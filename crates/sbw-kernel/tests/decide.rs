//! `decide` tests: the kernel's response cycle — check, execute, reactions,
//! depth limit, determinism — exercised through test-only rule fixtures
//! (the lantern rules, the nailed lantern, the endless loop, the quiet
//! time command). Fixtures live here and are never reachable from non-test
//! code.

mod common;

use common::{
    ADA, CANTINA, DOOR, INVENTORY, LANTERN, anchor, element_id, incarnation_id, take_lantern,
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
    ) -> ExecutionOutput {
        let from = state
            .elements()
            .get(&element_id(LANTERN))
            .and_then(|lantern| lantern.container());
        ExecutionOutput::new(
            Meaning::new("preso"),
            vec![Effect::MoveElement {
                id: element_id(LANTERN),
                from,
                to: Some(element_id(INVENTORY)),
            }],
        )
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
    ) -> ExecutionOutput {
        LanternRules.execute(state, command, dice)
    }

    fn react(&self, _state: &StateView<'_>, _trigger: &Event) -> Vec<ExecutionOutput> {
        Vec::new()
    }
}

/// The endless-loop rules: every "loop" event triggers another one.
struct LoopRules;

impl LoopRules {
    fn loop_output() -> ExecutionOutput {
        ExecutionOutput::new(
            Meaning::new("loop"),
            vec![Effect::SetAttribute {
                owner: AttributeOwner::Element(element_id(DOOR)),
                name: "count".into(),
                old: None,
                new: AttributeValue::Integer(1),
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
    ) -> ExecutionOutput {
        Self::loop_output()
    }

    fn react(&self, _state: &StateView<'_>, trigger: &Event) -> Vec<ExecutionOutput> {
        if trigger.meaning().as_str() == "loop" {
            vec![Self::loop_output()]
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
    ) -> ExecutionOutput {
        ExecutionOutput::new(Meaning::new("niente"), Vec::new())
    }

    fn react(&self, _state: &StateView<'_>, _trigger: &Event) -> Vec<ExecutionOutput> {
        Vec::new()
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

#[test]
fn anchor_helper_is_used_by_fixtures() {
    // keep the shared anchor in use: the worlds above are built on it
    assert_eq!(world_with_lantern().time_anchor(), anchor());
}
