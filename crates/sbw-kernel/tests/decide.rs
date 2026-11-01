//! `decide` tests: the kernel's response cycle — check, execute, reactions,
//! depth limit, determinism — exercised through test-only rule fixtures
//! (the lantern rules, the nailed lantern, the endless loop, the quiet
//! time command). Fixtures live here and are never reachable from non-test
//! code.

mod common;

use common::{
    ADA, CANTINA, DOOR, INVENTORY, LANTERN, element_id, incarnation_id, take_lantern,
    world_with_lantern, world_with_lantern_elsewhere,
};
use sbw_kernel::command::{Command, CommandParam, CommandSource};
use sbw_kernel::decide::{REACTION_DEPTH_LIMIT, decide};
use sbw_kernel::effect::{AttributeValue, Effect};
use sbw_kernel::error::DecideError;
use sbw_kernel::event::{DiceOutcome, Event};
use sbw_kernel::meaning::Meaning;
use sbw_kernel::refusal::Refusal;
use sbw_kernel::rule::{ExecutionOutput, Rule, StateView};
use sbw_kernel::state::AttributeOwner;
use sbw_kernel::time::{Seed, WallTime};

fn time_command() -> Command {
    Command::new(
        CommandSource::World,
        "time-passes",
        Vec::new(),
        WallTime::new(5),
        Seed::new(Vec::new()),
    )
}

/// The ordinary-case rules: Ada can take the lantern while they share a
/// location; a trap springs when the lantern is taken.
struct LanternRules;

impl LanternRules {
    fn take_command(who: &CommandSource) -> Command {
        Command::new(
            who.clone(),
            "take",
            vec![CommandParam::Element(element_id(LANTERN))],
            WallTime::new(0),
            Seed::new(Vec::new()),
        )
    }

    fn lantern_with(who: &CommandSource, state: &StateView<'_>) -> bool {
        if let CommandSource::Incarnation(inc) = who
            && let Some(incarnation) = state.incarnations().get(inc)
            && let Some(lantern) = state.elements().get(&element_id(LANTERN))
        {
            return lantern.container() == Some(incarnation.location());
        }
        false
    }
}

impl Rule for LanternRules {
    fn available_commands(&self, state: &StateView<'_>, who: &CommandSource) -> Vec<Command> {
        if Self::lantern_with(who, state) {
            vec![Self::take_command(who)]
        } else {
            Vec::new()
        }
    }

    fn check(&self, state: &StateView<'_>, command: &Command) -> Result<(), Refusal> {
        if command.action() == "take"
            && command.params() == [CommandParam::Element(element_id(LANTERN))]
            && state.elements().contains_key(&element_id(LANTERN))
        {
            Ok(())
        } else {
            Err(Refusal::new("unknown-action", Vec::new()))
        }
    }

    fn execute(
        &self,
        state: &StateView<'_>,
        _command: &Command,
        _dice: &[DiceOutcome],
    ) -> Option<ExecutionOutput> {
        let from = state
            .elements()
            .get(&element_id(LANTERN))
            .and_then(|lantern| lantern.container());
        Some(ExecutionOutput::new(
            Meaning::new("preso"),
            vec![Effect::MoveElement {
                id: element_id(LANTERN),
                from,
                to: Some(element_id(INVENTORY)),
            }],
        ))
    }

    fn react(&self, _state: &StateView<'_>, trigger: &Event) -> Vec<ExecutionOutput> {
        if trigger.meaning().as_str() == "preso" {
            vec![ExecutionOutput::new(
                Meaning::new("trap"),
                vec![Effect::SetAttribute {
                    owner: AttributeOwner::Element(element_id(DOOR)),
                    name: "alarmed".into(),
                    old: None,
                    new: AttributeValue::Boolean(true),
                }],
            )]
        } else {
            Vec::new()
        }
    }
}

/// The nailed-lantern rules: taking the lantern is refused with a
/// structured reason.
struct NailedLanternRules;

impl Rule for NailedLanternRules {
    fn available_commands(&self, state: &StateView<'_>, who: &CommandSource) -> Vec<Command> {
        LanternRules.available_commands(state, who)
    }

    fn check(&self, _state: &StateView<'_>, command: &Command) -> Result<(), Refusal> {
        if command.action() == "take"
            && command.params() == [CommandParam::Element(element_id(LANTERN))]
        {
            Err(Refusal::new(
                "nailed-down",
                vec![CommandParam::Element(element_id(LANTERN))],
            ))
        } else {
            Err(Refusal::new("unknown-action", Vec::new()))
        }
    }

    fn execute(
        &self,
        state: &StateView<'_>,
        command: &Command,
        dice: &[DiceOutcome],
    ) -> Option<ExecutionOutput> {
        LanternRules.execute(state, command, dice)
    }

    fn react(&self, _state: &StateView<'_>, _trigger: &Event) -> Vec<ExecutionOutput> {
        Vec::new()
    }
}

/// Reads the `count` an event just set, if it set one.
fn count_of(trigger: &Event) -> Option<i64> {
    trigger.effects().iter().find_map(|effect| match effect {
        Effect::SetAttribute {
            new: AttributeValue::Integer(count),
            ..
        } => Some(*count),
        _ => None,
    })
}

/// The endless-loop rules: every "loop" event triggers another one.
struct LoopRules;

impl LoopRules {
    fn loop_output(old: Option<AttributeValue>, new: i64) -> ExecutionOutput {
        ExecutionOutput::new(
            Meaning::new("loop"),
            vec![Effect::SetAttribute {
                owner: AttributeOwner::Element(element_id(DOOR)),
                name: "count".into(),
                old,
                new: AttributeValue::Integer(new),
            }],
        )
    }
}

impl Rule for LoopRules {
    fn available_commands(&self, _state: &StateView<'_>, _who: &CommandSource) -> Vec<Command> {
        Vec::new()
    }

    fn check(&self, _state: &StateView<'_>, _command: &Command) -> Result<(), Refusal> {
        Ok(())
    }

    fn execute(
        &self,
        _state: &StateView<'_>,
        _command: &Command,
        _dice: &[DiceOutcome],
    ) -> Option<ExecutionOutput> {
        Some(Self::loop_output(None, 1))
    }

    fn react(&self, _state: &StateView<'_>, trigger: &Event) -> Vec<ExecutionOutput> {
        if let Some(count) = count_of(trigger) {
            vec![Self::loop_output(
                Some(AttributeValue::Integer(count)),
                count + 1,
            )]
        } else {
            Vec::new()
        }
    }
}

/// The quiet time command: accepted, but it produces no effects.
struct QuietTimeRules;

impl Rule for QuietTimeRules {
    fn available_commands(&self, _state: &StateView<'_>, _who: &CommandSource) -> Vec<Command> {
        Vec::new()
    }

    fn check(&self, _state: &StateView<'_>, _command: &Command) -> Result<(), Refusal> {
        Ok(())
    }

    fn execute(
        &self,
        _state: &StateView<'_>,
        _command: &Command,
        _dice: &[DiceOutcome],
    ) -> Option<ExecutionOutput> {
        // accepted, but nothing happened: no event
        None
    }

    fn react(&self, _state: &StateView<'_>, _trigger: &Event) -> Vec<ExecutionOutput> {
        Vec::new()
    }
}

/// A reaction chain of a fixed length: every "loop" event carries its depth
/// in the effect, and the chain stops at `max`.
struct CountingLoopRules {
    max: usize,
}

impl CountingLoopRules {
    fn counted_output(count: i64) -> ExecutionOutput {
        Self::counted_set(None, count)
    }

    fn counted_next(previous: i64) -> ExecutionOutput {
        Self::counted_set(Some(AttributeValue::Integer(previous)), previous + 1)
    }

    fn counted_set(old: Option<AttributeValue>, new: i64) -> ExecutionOutput {
        ExecutionOutput::new(
            Meaning::new("loop"),
            vec![Effect::SetAttribute {
                owner: AttributeOwner::Element(element_id(DOOR)),
                name: "count".into(),
                old,
                new: AttributeValue::Integer(new),
            }],
        )
    }

    fn count_of(trigger: &Event) -> Option<i64> {
        count_of(trigger)
    }
}

impl Rule for CountingLoopRules {
    fn available_commands(&self, _state: &StateView<'_>, _who: &CommandSource) -> Vec<Command> {
        Vec::new()
    }

    fn check(&self, _state: &StateView<'_>, _command: &Command) -> Result<(), Refusal> {
        Ok(())
    }

    fn execute(
        &self,
        _state: &StateView<'_>,
        _command: &Command,
        _dice: &[DiceOutcome],
    ) -> Option<ExecutionOutput> {
        Some(Self::counted_output(1))
    }

    fn react(&self, _state: &StateView<'_>, trigger: &Event) -> Vec<ExecutionOutput> {
        if trigger.meaning().as_str() == "loop"
            && let Some(count) = Self::count_of(trigger)
            && (count as usize) < self.max
        {
            return vec![Self::counted_next(count)];
        }
        Vec::new()
    }
}

/// Rules with several reactions per round, to pin the evaluation order.
struct OrderingRules;

impl OrderingRules {
    fn output(meaning: &str, count: i64) -> ExecutionOutput {
        // each event sets its own attribute, so every "before" is None
        // and holds however many events the chain produces
        ExecutionOutput::new(
            Meaning::new(meaning),
            vec![Effect::SetAttribute {
                owner: AttributeOwner::Element(element_id(DOOR)),
                name: meaning.into(),
                old: None,
                new: AttributeValue::Integer(count),
            }],
        )
    }
}

impl Rule for OrderingRules {
    fn available_commands(&self, _state: &StateView<'_>, _who: &CommandSource) -> Vec<Command> {
        Vec::new()
    }

    fn check(&self, _state: &StateView<'_>, _command: &Command) -> Result<(), Refusal> {
        Ok(())
    }

    fn execute(
        &self,
        _state: &StateView<'_>,
        _command: &Command,
        _dice: &[DiceOutcome],
    ) -> Option<ExecutionOutput> {
        Some(Self::output("start", 0))
    }

    fn react(&self, _state: &StateView<'_>, trigger: &Event) -> Vec<ExecutionOutput> {
        match trigger.meaning().as_str() {
            // two reactions in the same round, in the rule's order
            "start" => vec![Self::output("a", 1), Self::output("b", 2)],
            "a" => vec![Self::output("a1", 3)],
            "b" => vec![Self::output("b1", 4)],
            _ => Vec::new(),
        }
    }
}

#[test]
fn same_state_command_and_rules_produce_the_same_event_sequence() {
    let state = world_with_lantern();
    let command = take_lantern();
    let first = decide(&state, &command, &LanternRules).expect("decided");
    let second = decide(&state, &command, &LanternRules).expect("decided");
    assert!(!first.is_empty());
    assert_eq!(first, second);
}

#[test]
fn nailed_lantern_command_is_refused_with_structured_reason_and_no_events() {
    let state = world_with_lantern();
    let before = state.clone();
    let command = take_lantern();
    let error = decide(&state, &command, &NailedLanternRules).expect_err("refused");
    match error {
        DecideError::Refusal(refusal) => {
            assert_eq!(refusal.code(), "nailed-down");
            assert_eq!(
                refusal.params(),
                &[CommandParam::Element(element_id(LANTERN))]
            );
        }
        other => panic!("expected a refusal, got {other:?}"),
    }
    // a refusal produces no events and leaves the state untouched
    assert_eq!(state, before);
}

#[test]
fn taking_the_lantern_produces_the_taken_event_with_the_move_effect() {
    let state = world_with_lantern();
    let command = take_lantern();
    let events = decide(&state, &command, &LanternRules).expect("decided");
    let taken = &events[0];
    assert_eq!(taken.meaning().as_str(), "preso");
    assert_eq!(
        taken.effects(),
        &[Effect::MoveElement {
            id: element_id(LANTERN),
            from: Some(element_id(CANTINA)),
            to: Some(element_id(INVENTORY)),
        }]
    );
    // the kernel stamps the rest: wall time from the command, never exposed,
    // the command's entities involved
    assert_eq!(taken.wall_time(), WallTime::new(1_000));
    assert!(!taken.exposed());
    assert_eq!(
        taken.involved(),
        &[
            sbw_kernel::event::InvolvedEntity::Incarnation(incarnation_id(ADA)),
            sbw_kernel::event::InvolvedEntity::Element(element_id(LANTERN)),
        ]
    );
}

#[test]
fn available_commands_offer_take_only_while_the_lantern_is_with_ada() {
    let rules = LanternRules;
    let who = CommandSource::Incarnation(incarnation_id(ADA));
    let state = world_with_lantern();
    let view = StateView::new(&state);
    let offered = rules.available_commands(&view, &who);
    assert_eq!(offered.len(), 1);
    assert_eq!(offered[0].action(), "take");
    let elsewhere = world_with_lantern_elsewhere();
    let elsewhere_view = StateView::new(&elsewhere);
    assert!(rules.available_commands(&elsewhere_view, &who).is_empty());
}

#[test]
fn trap_reaction_adds_its_event_after_the_taken_event_in_the_same_response() {
    let state = world_with_lantern();
    let command = take_lantern();
    let events = decide(&state, &command, &LanternRules).expect("decided");
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].meaning().as_str(), "preso");
    assert_eq!(events[1].meaning().as_str(), "trap");
    // reaction events are stamped by the kernel exactly like the first one
    let trap = &events[1];
    assert_eq!(trap.wall_time(), WallTime::new(1_000));
    assert!(!trap.exposed());
    assert_eq!(trap.involved(), events[0].involved());
}

#[test]
fn reaction_rounds_are_evaluated_in_a_deterministic_order() {
    let state = world_with_lantern();
    let command = time_command();
    let events = decide(&state, &command, &OrderingRules).expect("decided");
    let meanings: Vec<&str> = events.iter().map(|e| e.meaning().as_str()).collect();
    assert_eq!(meanings, ["start", "a", "b", "a1", "b1"]);
}

#[test]
fn a_reaction_chain_at_exactly_the_depth_limit_is_accepted() {
    let state = world_with_lantern();
    let command = time_command();
    let rules = CountingLoopRules {
        max: REACTION_DEPTH_LIMIT + 1,
    };
    let events = decide(&state, &command, &rules).expect("within the limit");
    assert_eq!(events.len(), REACTION_DEPTH_LIMIT + 1);
}

#[test]
fn a_reaction_chain_beyond_the_depth_limit_is_rejected() {
    let state = world_with_lantern();
    let command = time_command();
    let rules = CountingLoopRules {
        max: REACTION_DEPTH_LIMIT + 2,
    };
    let error = decide(&state, &command, &rules).expect_err("beyond the limit");
    assert_eq!(
        error,
        DecideError::DepthLimitExceeded {
            limit: REACTION_DEPTH_LIMIT
        }
    );
}

#[test]
fn an_endless_reaction_chain_fails_with_depth_limit_error_and_no_events() {
    let state = world_with_lantern();
    let before = state.clone();
    let command = time_command();
    let error = decide(&state, &command, &LoopRules).expect_err("depth limit exceeded");
    assert_eq!(
        error,
        DecideError::DepthLimitExceeded {
            limit: REACTION_DEPTH_LIMIT
        }
    );
    // the whole response is discarded and the state is untouched
    assert_eq!(state, before);
}

#[test]
fn an_accepted_command_with_no_effects_returns_an_empty_response() {
    let state = world_with_lantern();
    let command = time_command();
    let events = decide(&state, &command, &QuietTimeRules).expect("accepted");
    assert!(events.is_empty());
}

/// A failed execution: the output has a meaning but no effects, and it is
/// still an event.
struct FailedExecutionRules;

impl Rule for FailedExecutionRules {
    fn available_commands(&self, _state: &StateView<'_>, _who: &CommandSource) -> Vec<Command> {
        Vec::new()
    }

    fn check(&self, _state: &StateView<'_>, _command: &Command) -> Result<(), Refusal> {
        Ok(())
    }

    fn execute(
        &self,
        _state: &StateView<'_>,
        _command: &Command,
        _dice: &[DiceOutcome],
    ) -> Option<ExecutionOutput> {
        Some(ExecutionOutput::new(Meaning::new("fallito"), Vec::new()))
    }

    fn react(&self, _state: &StateView<'_>, _trigger: &Event) -> Vec<ExecutionOutput> {
        Vec::new()
    }
}

#[test]
fn an_effect_free_failure_execution_still_produces_its_event() {
    let state = world_with_lantern();
    let command = time_command();
    let events = decide(&state, &command, &FailedExecutionRules).expect("decided");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].meaning().as_str(), "fallito");
    assert!(events[0].effects().is_empty());
    assert!(!events[0].exposed());
}

/// Rules whose execution produces an effect whose "before" value does not
/// match the state: the lantern is in the Cantina, not in the inventory.
struct MismatchingRules;

impl Rule for MismatchingRules {
    fn available_commands(&self, _state: &StateView<'_>, _who: &CommandSource) -> Vec<Command> {
        Vec::new()
    }

    fn check(&self, _state: &StateView<'_>, _command: &Command) -> Result<(), Refusal> {
        Ok(())
    }

    fn execute(
        &self,
        _state: &StateView<'_>,
        _command: &Command,
        _dice: &[DiceOutcome],
    ) -> Option<ExecutionOutput> {
        Some(ExecutionOutput::new(
            Meaning::new("preso"),
            vec![Effect::MoveElement {
                id: element_id(LANTERN),
                from: Some(element_id(INVENTORY)),
                to: Some(element_id(CANTINA)),
            }],
        ))
    }

    fn react(&self, _state: &StateView<'_>, _trigger: &Event) -> Vec<ExecutionOutput> {
        Vec::new()
    }
}

#[test]
fn a_response_whose_event_does_not_apply_is_discarded_with_an_error() {
    let state = world_with_lantern();
    let before = state.clone();
    let command = take_lantern();
    let result = decide(&state, &command, &MismatchingRules);
    let error = result.expect_err("the event does not apply");
    assert!(matches!(error, DecideError::ApplyFailed(_)));
    // the whole response is discarded and the caller's state is untouched
    assert_eq!(state, before);
}
