//! Rule-interface tests: `Command`, `Refusal` and the `Rule` trait,
//! exercised through test-only rule fixtures (the nailed lantern, the
//! lock-picking rule). The fixtures live here and are never reachable from
//! non-test code.

use sbw_kernel::command::{Command, CommandParam, CommandSource};
use sbw_kernel::effect::{Effect, Rate, TimeAnchor};
use sbw_kernel::event::{DiceOutcome, Event, InvolvedEntity};
use sbw_kernel::id::{ElementId, IncarnationId};
use sbw_kernel::meaning::Meaning;
use sbw_kernel::refusal::Refusal;
use sbw_kernel::rule::{ExecutionOutput, Rule, StateView};
use sbw_kernel::state::{ElementSkeleton, IncarnationSkeleton, State};
use sbw_kernel::time::{Seed, WallTime, WorldTime};

fn element_id(raw: u64) -> ElementId {
    ElementId::from_bytes(&raw.to_be_bytes()).expect("8 bytes")
}

fn incarnation_id(raw: u64) -> IncarnationId {
    IncarnationId::from_bytes(&raw.to_be_bytes()).expect("8 bytes")
}

const CANTINA: u64 = 1;
const LANTERN: u64 = 2;
const DOOR: u64 = 3;
const ADA: u64 = 7;

fn anchor() -> TimeAnchor {
    TimeAnchor::new(
        WorldTime::new(800),
        WallTime::new(0),
        Rate::new(1, 1).expect("positive denominator"),
    )
}

/// The world of the fixture rules: Ada in the Cantina, a lantern and a
/// locked door in the Cantina.
fn world() -> State {
    let mut state = State::new(anchor());
    state
        .add_element(ElementSkeleton::new(element_id(CANTINA), "room", None))
        .expect("room");
    state
        .add_element(ElementSkeleton::new(
            element_id(LANTERN),
            "lantern",
            Some(element_id(CANTINA)),
        ))
        .expect("lantern");
    state
        .add_element(ElementSkeleton::new(
            element_id(DOOR),
            "door",
            Some(element_id(CANTINA)),
        ))
        .expect("door");
    state
        .add_incarnation(IncarnationSkeleton::new(
            incarnation_id(ADA),
            element_id(CANTINA),
        ))
        .expect("ada");
    state
}

fn take_lantern() -> Command {
    Command::new(
        CommandSource::Incarnation(incarnation_id(ADA)),
        "take",
        vec![CommandParam::Element(element_id(LANTERN))],
        WallTime::new(1_000),
        Seed::new(vec![9, 9, 9]),
    )
}

fn taken_lantern_event() -> Event {
    Event::new(
        Meaning::new("preso"),
        vec![Effect::MoveElement {
            id: element_id(LANTERN),
            from: Some(element_id(CANTINA)),
            to: None,
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

/// Fixture rule: the lantern is nailed down, so taking it is refused with a
/// structured reason; a trap springs when the lantern is taken anyway.
struct NailedLantern;

impl Rule for NailedLantern {
    fn available_commands(&self, state: &StateView<'_>, who: &CommandSource) -> Vec<Command> {
        if state.elements().contains_key(&element_id(LANTERN)) {
            vec![Command::new(
                who.clone(),
                "take",
                vec![CommandParam::Element(element_id(LANTERN))],
                WallTime::new(0),
                Seed::new(Vec::new()),
            )]
        } else {
            Vec::new()
        }
    }

    fn check(&self, state: &StateView<'_>, command: &Command) -> Result<(), Refusal> {
        // the lantern is nailed down: "take the lantern" makes no sense in
        // this state — a refusal, not a failure
        if command.action() == "take"
            && command.params() == [CommandParam::Element(element_id(LANTERN))]
            && state.elements().contains_key(&element_id(LANTERN))
        {
            return Err(Refusal::new(
                "nailed-down",
                vec![CommandParam::Element(element_id(LANTERN))],
            ));
        }
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
                to: None,
            }],
        ))
    }

    fn react(&self, _state: &StateView<'_>, trigger: &Event) -> Vec<ExecutionOutput> {
        if trigger.meaning().as_str() == "preso" {
            vec![ExecutionOutput::new(Meaning::new("trap"), Vec::new())]
        } else {
            Vec::new()
        }
    }
}

/// Fixture rule: lock-picking requires a dice roll; a failed roll is a
/// failure (an event with meaning "fallito"), not a refusal.
struct LockPicking;

impl Rule for LockPicking {
    fn available_commands(&self, _state: &StateView<'_>, who: &CommandSource) -> Vec<Command> {
        vec![Command::new(
            who.clone(),
            "pick",
            vec![CommandParam::Element(element_id(DOOR))],
            WallTime::new(0),
            Seed::new(Vec::new()),
        )]
    }

    fn check(&self, _state: &StateView<'_>, _command: &Command) -> Result<(), Refusal> {
        Ok(())
    }

    fn execute(
        &self,
        _state: &StateView<'_>,
        _command: &Command,
        dice: &[DiceOutcome],
    ) -> Option<ExecutionOutput> {
        if dice.iter().any(|outcome| outcome.result() < 10) {
            Some(ExecutionOutput::new(Meaning::new("fallito"), Vec::new()))
        } else {
            Some(ExecutionOutput::new(Meaning::new("scassinato"), Vec::new()))
        }
    }

    fn react(&self, _state: &StateView<'_>, _trigger: &Event) -> Vec<ExecutionOutput> {
        Vec::new()
    }
}

#[test]
fn command_carries_who_action_params_and_runtime_data() {
    let command = take_lantern();
    assert_eq!(
        command.who(),
        &CommandSource::Incarnation(incarnation_id(ADA))
    );
    assert_eq!(command.action(), "take");
    assert_eq!(
        command.params(),
        &[CommandParam::Element(element_id(LANTERN))]
    );
    assert_eq!(command.wall_time(), WallTime::new(1_000));
    assert_eq!(command.dice_material().as_bytes(), &[9, 9, 9]);
}

#[test]
fn refusal_carries_a_structured_code_and_params() {
    let refusal = Refusal::new("not-present", vec![CommandParam::Element(element_id(DOOR))]);
    assert_eq!(refusal.code(), "not-present");
    assert_eq!(refusal.params(), &[CommandParam::Element(element_id(DOOR))]);
}

#[test]
fn rule_trait_is_object_safe_and_callable_through_dyn() {
    let state = world();
    let view = StateView::new(&state);
    let rule: Box<dyn Rule> = Box::new(NailedLantern);
    let command = take_lantern();
    assert_eq!(rule.available_commands(&view, command.who()).len(), 1);
    assert!(rule.check(&view, &command).is_err());
    let output = rule.execute(&view, &command, &[]).expect("an output");
    assert_eq!(output.meaning().as_str(), "preso");
    assert_eq!(rule.react(&view, &taken_lantern_event()).len(), 1);
}

#[test]
fn nailed_lantern_take_is_refused_with_structured_reason_and_the_state_is_untouched() {
    let state = world();
    let before = state.clone();
    let view = StateView::new(&state);
    let rule = NailedLantern;
    let command = take_lantern();
    // the command is even offered, yet it makes no sense here: refused
    assert_eq!(rule.available_commands(&view, command.who()).len(), 1);
    let refusal = rule
        .check(&view, &command)
        .expect_err("the nailed lantern refuses the take");
    assert_eq!(refusal.code(), "nailed-down");
    assert_eq!(
        refusal.params(),
        &[CommandParam::Element(element_id(LANTERN))]
    );
    // a refusal produces no events and changes nothing
    assert_eq!(state, before);
}

#[test]
fn failed_lockpick_execution_yields_failure_meaning_and_the_event_records_the_roll() {
    let state = world();
    let view = StateView::new(&state);
    let rule = LockPicking;
    let command = Command::new(
        CommandSource::Incarnation(incarnation_id(ADA)),
        "pick",
        vec![CommandParam::Element(element_id(DOOR))],
        WallTime::new(2_000),
        Seed::new(vec![1, 2, 3]),
    );
    // a failed roll: a failure, not a refusal — the output says "fallito"
    let dice = [DiceOutcome::new("lockpick, 1d20", 3)];
    let output = rule.execute(&view, &command, &dice).expect("an output");
    assert_eq!(output.meaning().as_str(), "fallito");
    // the event recorded for this execution carries the roll, and no event
    // the kernel produces is ever exposed
    let event = Event::new(
        output.meaning().clone(),
        output.effects().to_vec(),
        vec![InvolvedEntity::Incarnation(incarnation_id(ADA))],
        None,
        Vec::new(),
        command.wall_time(),
        dice.to_vec(),
    );
    assert_eq!(event.dice_outcomes(), dice);
    assert_eq!(event.dice_outcomes()[0].reason(), "lockpick, 1d20");
    assert!(!event.exposed());
    // a successful roll succeeds instead
    let good_dice = [DiceOutcome::new("lockpick, 1d20", 17)];
    let output = rule
        .execute(&view, &command, &good_dice)
        .expect("an output");
    assert_eq!(output.meaning().as_str(), "scassinato");
}

#[test]
fn available_commands_offer_take_only_while_the_lantern_is_in_the_world() {
    let state = world();
    let view = StateView::new(&state);
    let rule = NailedLantern;
    let who = CommandSource::Incarnation(incarnation_id(ADA));
    assert_eq!(rule.available_commands(&view, &who).len(), 1);
    let empty = State::new(anchor());
    let empty_view = StateView::new(&empty);
    assert!(rule.available_commands(&empty_view, &who).is_empty());
}

#[test]
fn react_springs_the_trap_only_for_the_taken_lantern() {
    let state = world();
    let view = StateView::new(&state);
    let rule = NailedLantern;
    let trap = rule.react(&view, &taken_lantern_event());
    assert_eq!(trap.len(), 1);
    assert_eq!(trap[0].meaning().as_str(), "trap");
    let other = Event::new(
        Meaning::new("altro"),
        Vec::new(),
        Vec::new(),
        None,
        Vec::new(),
        WallTime::new(5),
        Vec::new(),
    );
    assert!(rule.react(&view, &other).is_empty());
}
