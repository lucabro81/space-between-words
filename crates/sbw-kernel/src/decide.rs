//! `decide`: the kernel's main operation. Given a state, a command and the
//! author's rules, it runs the response cycle — check, execute, then
//! reactions on each new event until none are produced or the depth limit
//! is exceeded — and returns the response's events, or a refusal, or an
//! error.
//!
//! Pure: no I/O, no clock, no randomness — all external data arrives in the
//! command. The caller's state is never mutated: the cycle works on a copy.
//! Replay never runs the rules: `apply` works only on the effects stored in
//! the events this function returns.

use std::collections::{BTreeMap, BTreeSet};

use crate::command::{Command, CommandParam, CommandSource};
use crate::error::DecideError;
use crate::event::{DiceOutcome, Event, InvolvedEntity};
use crate::id::{ThreadId, VigilId};
use crate::rule::{ExecutionOutput, Rule, StateView};
use crate::state::{State, ThreadSkeleton, VigilSkeleton};
use crate::time::Seed;

/// The maximum reaction depth of one response. One mechanism and one limit
/// for reactions and thread generators alike: a chain whose depth *exceeds*
/// the limit is a reported error and the whole response is discarded — no
/// events, state unchanged. Never a silent truncation.
pub const REACTION_DEPTH_LIMIT: usize = 100;

/// Runs the response cycle for a command against a state and a ruleset.
///
/// - `Err(DecideError::Refusal)`: the command makes no sense in this state;
///   no events, the state is unchanged.
/// - `Err(DecideError::DepthLimitExceeded)`: the reaction chain went deeper
///   than the limit; no events, the state is unchanged.
/// - `Ok(events)`: the response group, in evaluation order. Empty when the
///   accepted command produced no event — a command that changes nothing
///   produces no events.
pub fn decide<R: Rule>(
    state: &State,
    command: &Command,
    rules: &R,
) -> Result<Vec<Event>, DecideError> {
    let view = StateView::new(state);
    if let Err(refusal) = rules.check(&view, command) {
        return Err(DecideError::Refusal(refusal));
    }
    let dice = expand_dice_material(command.dice_material());
    // the working copy: applying the produced events to it is `apply`'s job;
    // until then reactions see the copy as it stands
    let working = state.clone();
    let mut events: Vec<Event> = Vec::new();
    let mut pending: Vec<Event> = Vec::new();
    if let Some(output) = rules.execute(&view, command, &dice) {
        let event = build_event(&working, command, &output, &dice);
        events.push(event.clone());
        pending.push(event);
    }
    // reaction rounds in breadth-first order: deterministic, and at parity
    // by IDs, since every structure iterated here is ordered
    let mut depth = 0usize;
    while !pending.is_empty() {
        let triggers = std::mem::take(&mut pending);
        let mut next: Vec<Event> = Vec::new();
        for trigger in &triggers {
            for output in rules.react(&StateView::new(&working), trigger) {
                let event = build_event(&working, command, &output, &dice);
                events.push(event.clone());
                next.push(event);
            }
        }
        if !next.is_empty() {
            depth += 1;
            if depth > REACTION_DEPTH_LIMIT {
                return Err(DecideError::DepthLimitExceeded {
                    limit: REACTION_DEPTH_LIMIT,
                });
            }
        }
        pending = next;
    }
    Ok(events)
}

/// Builds the event for an execution output. Every output becomes an event,
/// even one with no effects — a failure is an execution too and is recorded
/// like any event. The kernel stamps the fields it owns: involved entities
/// from the command, the actor's open vigil, thread membership, wall time,
/// dice outcomes. `exposed` is always `false`: the event constructor
/// enforces it.
fn build_event(
    state: &State,
    command: &Command,
    output: &ExecutionOutput,
    dice: &[DiceOutcome],
) -> Event {
    let involved = involved_from_command(command);
    let vigil = vigil_of(state.open_vigils(), command.who());
    let threads = threads_for(state.open_threads(), &involved);
    Event::new(
        output.meaning().clone(),
        output.effects().to_vec(),
        involved,
        vigil,
        threads,
        command.wall_time(),
        dice.to_vec(),
    )
}

/// The entities a command involves: its source plus the elements and
/// incarnations in its parameters, deduplicated and in order.
fn involved_from_command(command: &Command) -> Vec<InvolvedEntity> {
    let mut involved: BTreeSet<InvolvedEntity> = BTreeSet::new();
    if let CommandSource::Incarnation(id) = command.who() {
        involved.insert(InvolvedEntity::Incarnation(*id));
    }
    for param in command.params() {
        match param {
            CommandParam::Element(id) => {
                involved.insert(InvolvedEntity::Element(*id));
            }
            CommandParam::Incarnation(id) => {
                involved.insert(InvolvedEntity::Incarnation(*id));
            }
            _ => {}
        }
    }
    involved.into_iter().collect()
}

/// The open vigil of the acting incarnation, if any.
fn vigil_of(
    open_vigils: &BTreeMap<VigilId, VigilSkeleton>,
    who: &CommandSource,
) -> Option<VigilId> {
    if let CommandSource::Incarnation(id) = who {
        for (vigil_id, vigil) in open_vigils {
            if vigil.incarnation() == *id {
                return Some(*vigil_id);
            }
        }
    }
    None
}

/// The open threads that follow any involved entity, in ID order: an event
/// that involves one of their elements or incarnations belongs to them.
fn threads_for(
    open_threads: &BTreeMap<ThreadId, ThreadSkeleton>,
    involved: &[InvolvedEntity],
) -> Vec<ThreadId> {
    open_threads
        .iter()
        .filter(|(_, thread)| {
            involved.iter().any(|entity| match entity {
                InvolvedEntity::Element(id) => thread.elements().contains(id),
                InvolvedEntity::Incarnation(id) => thread.incarnations().contains(id),
            })
        })
        .map(|(id, _)| *id)
        .collect()
}

/// Expands the command's dice material into rolls. The deterministic PRNG
/// is a later unit of work; until it lands, no rolls are expanded and the
/// events record none.
fn expand_dice_material(_seed: &Seed) -> Vec<DiceOutcome> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::{Effect, Rate, TimeAnchor};
    use crate::id::{ElementId, IncarnationId};
    use crate::meaning::Meaning;
    use crate::time::{WallTime, WorldTime};

    fn anchor() -> TimeAnchor {
        TimeAnchor::new(
            WorldTime::new(800),
            WallTime::new(0),
            Rate::new(1, 1).expect("positive denominator"),
        )
    }

    #[test]
    fn involved_comes_from_source_and_params_deduped_and_ordered() {
        let command = Command::new(
            CommandSource::Incarnation(IncarnationId::from_raw(7)),
            "take",
            vec![
                CommandParam::Element(ElementId::from_raw(2)),
                CommandParam::Element(ElementId::from_raw(2)),
                CommandParam::Incarnation(IncarnationId::from_raw(7)),
                CommandParam::Integer(3),
            ],
            WallTime::new(1),
            Seed::new(Vec::new()),
        );
        assert_eq!(
            involved_from_command(&command),
            vec![
                InvolvedEntity::Incarnation(IncarnationId::from_raw(7)),
                InvolvedEntity::Element(ElementId::from_raw(2)),
            ]
        );
    }

    #[test]
    fn vigil_is_the_actors_open_vigil() {
        let mut vigils = BTreeMap::new();
        vigils.insert(
            VigilId::from_raw(5),
            VigilSkeleton::new(VigilId::from_raw(5), IncarnationId::from_raw(7)),
        );
        assert_eq!(
            vigil_of(
                &vigils,
                &CommandSource::Incarnation(IncarnationId::from_raw(7))
            ),
            Some(VigilId::from_raw(5))
        );
        assert_eq!(vigil_of(&vigils, &CommandSource::Curator), None);
        assert_eq!(
            vigil_of(
                &vigils,
                &CommandSource::Incarnation(IncarnationId::from_raw(8))
            ),
            None
        );
    }

    #[test]
    fn threads_follow_the_involved_entities_in_id_order() {
        let mut threads = BTreeMap::new();
        threads.insert(
            ThreadId::from_raw(9),
            ThreadSkeleton::new(ThreadId::from_raw(9), [ElementId::from_raw(3)], Vec::new()),
        );
        threads.insert(
            ThreadId::from_raw(2),
            ThreadSkeleton::new(
                ThreadId::from_raw(2),
                [ElementId::from_raw(2)],
                [IncarnationId::from_raw(7)],
            ),
        );
        let by_element = vec![InvolvedEntity::Element(ElementId::from_raw(2))];
        assert_eq!(
            threads_for(&threads, &by_element),
            vec![ThreadId::from_raw(2)]
        );
        let by_incarnation = vec![InvolvedEntity::Incarnation(IncarnationId::from_raw(7))];
        assert_eq!(
            threads_for(&threads, &by_incarnation),
            vec![ThreadId::from_raw(2)]
        );
        let other = vec![InvolvedEntity::Element(ElementId::from_raw(3))];
        assert_eq!(threads_for(&threads, &other), vec![ThreadId::from_raw(9)]);
        let unrelated = vec![InvolvedEntity::Element(ElementId::from_raw(42))];
        assert!(threads_for(&threads, &unrelated).is_empty());
    }

    #[test]
    fn an_output_with_no_effects_still_builds_an_event() {
        // a failure is an execution too: it is recorded like any event,
        // even when it carries no effects
        let state = State::new(anchor());
        let command = Command::new(
            CommandSource::World,
            "time-passes",
            Vec::new(),
            WallTime::new(5),
            Seed::new(Vec::new()),
        );
        let output = ExecutionOutput::new(Meaning::new("fallito"), Vec::new());
        let event = build_event(&state, &command, &output, &[]);
        assert_eq!(event.meaning().as_str(), "fallito");
        assert!(event.effects().is_empty());
    }

    #[test]
    fn built_events_are_stamped_by_the_kernel() {
        let state = State::new(anchor());
        let command = Command::new(
            CommandSource::Incarnation(IncarnationId::from_raw(7)),
            "take",
            vec![CommandParam::Element(ElementId::from_raw(2))],
            WallTime::new(1_000),
            Seed::new(vec![9]),
        );
        let output = ExecutionOutput::new(
            Meaning::new("preso"),
            vec![Effect::MoveElement {
                id: ElementId::from_raw(2),
                from: Some(ElementId::from_raw(1)),
                to: None,
            }],
        );
        let dice = [DiceOutcome::new("attack, 1d20", 14)];
        let event = build_event(&state, &command, &output, &dice);
        assert_eq!(event.meaning().as_str(), "preso");
        assert_eq!(event.wall_time(), WallTime::new(1_000));
        assert_eq!(event.dice_outcomes(), dice);
        assert!(!event.exposed());
        // an empty state has no open vigils or threads
        assert!(event.vigil().is_none());
        assert!(event.threads().is_empty());
        assert_eq!(
            event.involved(),
            [
                InvolvedEntity::Incarnation(IncarnationId::from_raw(7)),
                InvolvedEntity::Element(ElementId::from_raw(2)),
            ]
        );
    }
}
