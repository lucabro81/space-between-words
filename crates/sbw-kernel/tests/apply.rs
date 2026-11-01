//! `apply` integration tests: replay determinism, independence from the
//! rules, the rewind case, replay without re-rolling, and before-value
//! coherence. The tests here construct events directly — `apply` is tested
//! in isolation; one closing test checks that `decide` + `apply` and replay
//! agree.

mod common;

use common::{ADA, CANTINA, INVENTORY, LANTERN, element_id, incarnation_id, world_with_lantern};
use sbw_kernel::apply::apply;
use sbw_kernel::command::{Command, CommandParam, CommandSource};
use sbw_kernel::effect::{AttributeValue, Deadline, DeadlinePayload, Effect, Rate, TimeAnchor};
use sbw_kernel::error::ApplyError;
use sbw_kernel::event::{DiceOutcome, Event, InvolvedEntity};
use sbw_kernel::id::{ThreadId, VigilId};
use sbw_kernel::meaning::Meaning;
use sbw_kernel::rule::{ExecutionOutput, Rule, StateView};
use sbw_kernel::state::{AttributeKey, AttributeOwner, State};
use sbw_kernel::time::{WallTime, WorldTime};

fn taken_event(from: u64, to: u64) -> Event {
    Event::new(
        Meaning::new("preso"),
        vec![Effect::MoveElement {
            id: element_id(LANTERN),
            from: Some(element_id(from)),
            to: Some(element_id(to)),
        }],
        vec![
            InvolvedEntity::Incarnation(incarnation_id(ADA)),
            InvolvedEntity::Element(element_id(LANTERN)),
        ],
        None,
        Vec::new(),
        WallTime::new(1_000),
        Vec::new(),
    )
}

fn rewind_event(from: u64, to: u64, old: TimeAnchor, new: TimeAnchor) -> Event {
    Event::new(
        Meaning::new("riavvolto"),
        vec![
            Effect::MoveElement {
                id: element_id(LANTERN),
                from: Some(element_id(from)),
                to: Some(element_id(to)),
            },
            Effect::SetTimeAnchor { old, new },
        ],
        vec![
            InvolvedEntity::Incarnation(incarnation_id(ADA)),
            InvolvedEntity::Element(element_id(LANTERN)),
        ],
        None,
        Vec::new(),
        WallTime::new(2_000),
        Vec::new(),
    )
}

fn lantern_container(state: &State) -> Option<[u8; 8]> {
    state
        .elements()
        .get(&element_id(LANTERN))
        .expect("lantern")
        .container()
        .map(|id| id.to_bytes())
}

fn thread_id(raw: u64) -> ThreadId {
    ThreadId::from_bytes(&raw.to_be_bytes()).expect("8 bytes")
}

fn vigil_id(raw: u64) -> VigilId {
    VigilId::from_bytes(&raw.to_be_bytes()).expect("8 bytes")
}

fn anchor_at(world_time: i64, wall_time: i64) -> TimeAnchor {
    TimeAnchor::new(
        WorldTime::new(world_time),
        WallTime::new(wall_time),
        Rate::new(1, 1).expect("positive denominator"),
    )
}

/// Replays a log against the world's initial state.
fn replay(log: &[Event]) -> State {
    let mut state = world_with_lantern();
    for event in log {
        state = apply(&state, event).expect("the log is coherent");
    }
    state
}

#[test]
fn replaying_the_same_log_twice_produces_the_same_state() {
    let log = vec![taken_event(CANTINA, INVENTORY)];
    let first = replay(&log);
    let second = replay(&log);
    assert_eq!(first, second);
    assert_eq!(
        lantern_container(&first),
        Some(element_id(INVENTORY).to_bytes())
    );
}

#[test]
fn replay_does_not_depend_on_the_rules() {
    // two versions of the rules name the same taking differently — the
    // meaning is an open-vocabulary label the author owns — and agree on
    // the effects; replaying either log rebuilds the same state, because
    // the effects alone rebuild it
    let taken = taken_event(CANTINA, INVENTORY);
    let renamed = Event::new(
        Meaning::new("raccolta"),
        taken.effects().to_vec(),
        taken.involved().to_vec(),
        taken.vigil(),
        taken.threads().to_vec(),
        taken.wall_time(),
        Vec::new(),
    );
    assert_ne!(taken.meaning(), renamed.meaning());
    assert_eq!(replay(&[taken]), replay(&[renamed]));
}

#[test]
fn rewind_moves_the_lantern_back_and_resets_the_anchor_with_swapped_values() {
    let taken = taken_event(CANTINA, INVENTORY);
    let before_rewind = replay(std::slice::from_ref(&taken));
    assert_eq!(
        lantern_container(&before_rewind),
        Some(element_id(INVENTORY).to_bytes())
    );

    // the rewind is a move with before/after swapped plus an anchor change
    // with before/after swapped; it is a NEW event — the taken event still
    // applies in its original form to the state it was recorded against
    let rewind = rewind_event(INVENTORY, CANTINA, anchor_at(800, 0), anchor_at(790, 2_000));
    let after = apply(&before_rewind, &rewind).expect("rewind applies");
    assert_eq!(
        lantern_container(&after),
        Some(element_id(CANTINA).to_bytes())
    );
    assert_eq!(after.time_anchor(), anchor_at(790, 2_000));

    // replaying the full log from scratch reproduces the same final state,
    // without losing or rewriting the earlier events
    let full = vec![taken, rewind];
    assert_eq!(replay(&full), after);
}

#[test]
fn replay_never_re_rolls_dice() {
    // two logs identical except for the recorded dice outcomes replay to
    // the same state: the outcomes are chronicle in the event, never reach
    // the state, and applying a log invokes no randomness
    let with_high_roll = with_dice(taken_event(CANTINA, INVENTORY), 14);
    let with_low_roll = with_dice(taken_event(CANTINA, INVENTORY), 3);
    assert_eq!(with_high_roll.dice_outcomes()[0].result(), 14);
    assert_eq!(with_low_roll.dice_outcomes()[0].result(), 3);
    let first = replay(&[with_high_roll]);
    let second = replay(&[with_low_roll]);
    assert_eq!(first, second);
    assert_eq!(
        lantern_container(&first),
        Some(element_id(INVENTORY).to_bytes())
    );
}

/// The same event with a recorded dice outcome attached.
fn with_dice(event: Event, result: i64) -> Event {
    Event::new(
        event.meaning().clone(),
        event.effects().to_vec(),
        event.involved().to_vec(),
        event.vigil(),
        event.threads().to_vec(),
        event.wall_time(),
        vec![DiceOutcome::new("skill check, 1d20", result)],
    )
}

#[test]
fn a_mismatched_before_value_is_reported_and_changes_nothing() {
    let world = world_with_lantern();
    let original = world.clone();
    // the lantern is in the Cantina, not in the inventory: the "before"
    // value of this event does not match the state
    let bad = rewind_event(INVENTORY, CANTINA, anchor_at(800, 0), anchor_at(790, 2_000));
    let error = apply(&world, &bad).expect_err("before value mismatch");
    assert!(matches!(
        error,
        ApplyError::BeforeMismatch {
            effect_index: 0,
            ..
        }
    ));
    // the input state is untouched
    assert_eq!(world, original);
}

#[test]
fn a_mismatch_deeper_in_the_event_reports_its_effect_index() {
    let world = world_with_lantern();
    let original = world.clone();
    // the first effect applies cleanly, the second one's "before" anchor
    // does not match: the error names the second effect and the state the
    // caller holds is still the original
    let event = rewind_event(CANTINA, INVENTORY, anchor_at(999, 0), anchor_at(790, 2_000));
    let error = apply(&world, &event).expect_err("anchor mismatch");
    assert!(matches!(
        error,
        ApplyError::BeforeMismatch {
            effect_index: 1,
            ..
        }
    ));
    assert_eq!(world, original);
}

#[test]
fn every_effect_of_the_vocabulary_applies() {
    let world = world_with_lantern();
    let mut state = world.clone();

    // create with initial attributes
    let create = Event::new(
        Meaning::new("genesi"),
        vec![Effect::CreateElement {
            id: element_id(11),
            element_type: "coin".into(),
            container: Some(element_id(CANTINA)),
            attributes: vec![("shiny".into(), AttributeValue::Boolean(true))],
        }],
        Vec::new(),
        None,
        Vec::new(),
        WallTime::new(1_000),
        Vec::new(),
    );
    state = apply(&state, &create).expect("create applies");
    let key = AttributeKey::new(AttributeOwner::Element(element_id(11)), "shiny");
    assert_eq!(state.attribute(&key), Some(&AttributeValue::Boolean(true)));

    // set an attribute, then set it again with the recorded old value
    let set = Event::new(
        Meaning::new("acceso"),
        vec![Effect::SetAttribute {
            owner: AttributeOwner::Element(element_id(LANTERN)),
            name: "lit".into(),
            old: None,
            new: AttributeValue::Boolean(true),
        }],
        Vec::new(),
        None,
        Vec::new(),
        WallTime::new(1_000),
        Vec::new(),
    );
    state = apply(&state, &set).expect("set applies");
    let set_again = Event::new(
        Meaning::new("spento"),
        vec![Effect::SetAttribute {
            owner: AttributeOwner::Element(element_id(LANTERN)),
            name: "lit".into(),
            old: Some(AttributeValue::Boolean(true)),
            new: AttributeValue::Boolean(false),
        }],
        Vec::new(),
        None,
        Vec::new(),
        WallTime::new(1_000),
        Vec::new(),
    );
    state = apply(&state, &set_again).expect("set again applies");

    // open and close a vigil
    let vigil = Event::new(
        Meaning::new("veglia aperta"),
        vec![Effect::OpenVigil {
            id: vigil_id(5),
            incarnation: incarnation_id(ADA),
        }],
        Vec::new(),
        None,
        Vec::new(),
        WallTime::new(1_000),
        Vec::new(),
    );
    state = apply(&state, &vigil).expect("vigil applies");
    assert!(state.open_vigils().contains_key(&vigil_id(5)));
    let close_vigil = Event::new(
        Meaning::new("veglia chiusa"),
        vec![Effect::CloseVigil { id: vigil_id(5) }],
        Vec::new(),
        None,
        Vec::new(),
        WallTime::new(1_000),
        Vec::new(),
    );
    state = apply(&state, &close_vigil).expect("vigil closes");
    assert!(!state.open_vigils().contains_key(&vigil_id(5)));

    // open and close a thread
    let thread = Event::new(
        Meaning::new("filo aperto"),
        vec![Effect::OpenThread {
            id: thread_id(9),
            elements: vec![element_id(LANTERN)],
            incarnations: vec![incarnation_id(ADA)],
        }],
        Vec::new(),
        None,
        Vec::new(),
        WallTime::new(1_000),
        Vec::new(),
    );
    state = apply(&state, &thread).expect("thread applies");
    assert!(state.open_threads().contains_key(&thread_id(9)));
    let close_thread = Event::new(
        Meaning::new("filo chiuso"),
        vec![Effect::CloseThread { id: thread_id(9) }],
        Vec::new(),
        None,
        Vec::new(),
        WallTime::new(1_000),
        Vec::new(),
    );
    state = apply(&state, &close_thread).expect("thread closes");
    assert!(!state.open_threads().contains_key(&thread_id(9)));

    // schedule and cancel a deadline
    let deadline = Deadline::new(
        WorldTime::new(900),
        DeadlinePayload::new("wake", vec![AttributeValue::Integer(3)]),
    );
    let schedule = Event::new(
        Meaning::new("scadenza"),
        vec![Effect::ScheduleDeadline {
            deadline: deadline.clone(),
        }],
        Vec::new(),
        None,
        Vec::new(),
        WallTime::new(1_000),
        Vec::new(),
    );
    state = apply(&state, &schedule).expect("deadline applies");
    assert_eq!(
        state.deadlines().get(&WorldTime::new(900)),
        Some(&vec![deadline.clone()])
    );
    let cancel = Event::new(
        Meaning::new("scadenza annullata"),
        vec![Effect::CancelDeadline { deadline }],
        Vec::new(),
        None,
        Vec::new(),
        WallTime::new(1_000),
        Vec::new(),
    );
    state = apply(&state, &cancel).expect("deadline cancels");
    assert!(
        state
            .deadlines()
            .get(&WorldTime::new(900))
            .is_none_or(|list| list.is_empty())
    );

    // remove an element; its removal is recorded, so removing it again
    // reports an unknown-element error
    let remove = Event::new(
        Meaning::new("distrutto"),
        vec![Effect::RemoveElement {
            id: element_id(LANTERN),
        }],
        Vec::new(),
        None,
        Vec::new(),
        WallTime::new(1_000),
        Vec::new(),
    );
    state = apply(&state, &remove).expect("remove applies");
    assert!(!state.elements().contains_key(&element_id(LANTERN)));
    assert!(matches!(
        apply(&state, &remove),
        Err(ApplyError::UnknownElement { .. })
    ));
}

#[test]
fn attributes_can_belong_to_an_incarnation() {
    let world = world_with_lantern();
    let event = Event::new(
        Meaning::new("set"),
        vec![Effect::SetAttribute {
            owner: AttributeOwner::Incarnation(incarnation_id(ADA)),
            name: "hunger".into(),
            old: None,
            new: AttributeValue::Integer(3),
        }],
        Vec::new(),
        None,
        Vec::new(),
        WallTime::new(1_000),
        Vec::new(),
    );
    let next = apply(&world, &event).expect("applies");
    let key = AttributeKey::new(AttributeOwner::Incarnation(incarnation_id(ADA)), "hunger");
    assert_eq!(next.attribute(&key), Some(&AttributeValue::Integer(3)));
}

#[test]
fn effects_within_one_event_apply_in_order() {
    let world = world_with_lantern();
    // the second effect depends on the first one's change: both must apply
    // in sequence within the same event
    let event = Event::new(
        Meaning::new("genesi"),
        vec![
            Effect::CreateElement {
                id: element_id(11),
                element_type: "coin".into(),
                container: Some(element_id(CANTINA)),
                attributes: Vec::new(),
            },
            Effect::SetAttribute {
                owner: AttributeOwner::Element(element_id(11)),
                name: "shiny".into(),
                old: None,
                new: AttributeValue::Boolean(true),
            },
        ],
        Vec::new(),
        None,
        Vec::new(),
        WallTime::new(1_000),
        Vec::new(),
    );
    let next = apply(&world, &event).expect("applies");
    let key = AttributeKey::new(AttributeOwner::Element(element_id(11)), "shiny");
    assert_eq!(next.attribute(&key), Some(&AttributeValue::Boolean(true)));
    // the same two effects in the opposite order fail: the element does
    // not exist yet when the removal runs
    let backwards = Event::new(
        Meaning::new("genesi-inversa"),
        vec![
            Effect::RemoveElement { id: element_id(11) },
            Effect::CreateElement {
                id: element_id(11),
                element_type: "coin".into(),
                container: Some(element_id(CANTINA)),
                attributes: Vec::new(),
            },
        ],
        Vec::new(),
        None,
        Vec::new(),
        WallTime::new(1_000),
        Vec::new(),
    );
    assert!(apply(&world, &backwards).is_err());
}

#[test]
fn an_event_without_effects_changes_nothing() {
    let world = world_with_lantern();
    let event = Event::new(
        Meaning::new("fallito"),
        Vec::new(),
        Vec::new(),
        None,
        Vec::new(),
        WallTime::new(1_000),
        Vec::new(),
    );
    let next = apply(&world, &event).expect("applies");
    assert_eq!(next, world);
}

/// A ruleset whose execution moves the lantern: used to show that the
/// state replay rebuilds matches the state the same response builds when
/// applied event by event — whatever rules produced the events.
struct TakeRules;

impl Rule for TakeRules {
    fn available_commands(&self, _state: &StateView<'_>, _who: &CommandSource) -> Vec<Command> {
        Vec::new()
    }

    fn check(
        &self,
        _state: &StateView<'_>,
        _command: &Command,
    ) -> Result<(), sbw_kernel::refusal::Refusal> {
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
                from: Some(element_id(CANTINA)),
                to: Some(element_id(INVENTORY)),
            }],
        ))
    }

    fn react(&self, _state: &StateView<'_>, _trigger: &Event) -> Vec<ExecutionOutput> {
        Vec::new()
    }
}

#[test]
fn the_state_replay_rebuilds_matches_the_state_decide_plus_apply_builds() {
    let world = world_with_lantern();
    let command = Command::new(
        CommandSource::Incarnation(incarnation_id(ADA)),
        "take",
        vec![CommandParam::Element(element_id(LANTERN))],
        WallTime::new(1_000),
        sbw_kernel::time::Seed::new(Vec::new()),
    );
    let events = sbw_kernel::decide::decide(&world, &command, &TakeRules).expect("decides");
    let mut applied = world.clone();
    for event in &events {
        applied = apply(&applied, event).expect("the response's events apply");
    }
    // the concrete outcome: the lantern ended up in Ada's inventory
    assert_eq!(
        lantern_container(&applied),
        Some(element_id(INVENTORY).to_bytes())
    );
    // and replay from the initial state rebuilds exactly that state
    assert_eq!(replay(&events), applied);
}
