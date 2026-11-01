//! The rule interface: the only way author-declared rules interact with the
//! kernel. It mirrors the WIT rule interface (`wit/rules.wit`) one-to-one,
//! kept in sync by hand — no code generation. Rules run only inside
//! `decide`; replay never runs them, because `apply` works only on the
//! effects stored in the event.

use std::ops::Deref;

use crate::command::{Command, CommandSource};
use crate::effect::Effect;
use crate::event::{DiceOutcome, Event};
use crate::meaning::Meaning;
use crate::refusal::Refusal;
use crate::state::State;

/// A read-only view of the world state, handed to the rules. Placeholder:
/// it wraps a reference to the state and dereferences to it, giving the
/// rules the state's read API with no way to modify it.
#[derive(Debug, Clone, Copy)]
pub struct StateView<'a> {
    state: &'a State,
}

impl<'a> StateView<'a> {
    /// Views a state.
    pub fn new(state: &'a State) -> Self {
        Self { state }
    }
}

impl<'a> From<&'a State> for StateView<'a> {
    fn from(state: &'a State) -> Self {
        Self { state }
    }
}

impl Deref for StateView<'_> {
    type Target = State;

    fn deref(&self) -> &State {
        self.state
    }
}

/// What executing an accepted command produces: the meaning of the event
/// and the state changes it carries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionOutput {
    meaning: Meaning,
    effects: Vec<Effect>,
}

impl ExecutionOutput {
    /// Creates an output. Takes ownership: it is turned into an event.
    pub fn new(meaning: Meaning, effects: Vec<Effect>) -> Self {
        Self { meaning, effects }
    }

    /// The meaning of the produced event.
    pub fn meaning(&self) -> &Meaning {
        &self.meaning
    }

    /// The state changes the produced event carries.
    pub fn effects(&self) -> &[Effect] {
        &self.effects
    }
}

/// The four functions the kernel calls into author-declared rules.
pub trait Rule {
    /// Which commands are available to `who` in this state.
    fn available_commands(&self, state: &StateView<'_>, who: &CommandSource) -> Vec<Command>;

    /// Whether the command makes sense in this state: `Ok(())`, or a
    /// refusal carrying its structured reason.
    fn check(&self, state: &StateView<'_>, command: &Command) -> Result<(), Refusal>;

    /// Executes an accepted command: produces the meaning and the effects,
    /// given the recorded dice outcomes. `None` means the command changed
    /// nothing: no event is produced and the response may be empty.
    fn execute(
        &self,
        state: &StateView<'_>,
        command: &Command,
        dice: &[DiceOutcome],
    ) -> Option<ExecutionOutput>;

    /// Additional events triggered by an event just produced (reactions
    /// that run before, instead of, or after the action), in the same
    /// response. Rules return execution outputs; the kernel builds the
    /// events, stamping the fields it owns (wall time, dice outcomes,
    /// thread membership) on every one of them.
    fn react(&self, state: &StateView<'_>, trigger: &Event) -> Vec<ExecutionOutput>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::{Rate, TimeAnchor};
    use crate::id::ElementId;
    use crate::time::{Seed, WallTime, WorldTime};

    struct NoRules;

    impl Rule for NoRules {
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
            Some(ExecutionOutput::new(Meaning::new("niente"), Vec::new()))
        }

        fn react(&self, _state: &StateView<'_>, _trigger: &Event) -> Vec<ExecutionOutput> {
            Vec::new()
        }
    }

    fn anchor() -> TimeAnchor {
        TimeAnchor::new(
            WorldTime::new(800),
            WallTime::new(0),
            Rate::new(1, 1).expect("positive denominator"),
        )
    }

    #[test]
    fn state_view_reads_the_state_without_modifying_it() {
        let state = State::new(anchor());
        let view = StateView::new(&state);
        assert!(view.elements().is_empty());
        assert_eq!(view.time_anchor(), anchor());
        let via_from: StateView<'_> = (&state).into();
        assert!(via_from.attributes().is_empty());
    }

    #[test]
    fn execution_output_carries_meaning_and_effects() {
        let output = ExecutionOutput::new(
            Meaning::new("preso"),
            vec![Effect::RemoveElement {
                id: ElementId::from_raw(1),
            }],
        );
        assert_eq!(output.meaning().as_str(), "preso");
        assert_eq!(
            output.effects(),
            &[Effect::RemoveElement {
                id: ElementId::from_raw(1)
            }]
        );
    }

    #[test]
    fn rule_is_object_safe() {
        let state = State::new(anchor());
        let view = StateView::new(&state);
        let rule: Box<dyn Rule> = Box::new(NoRules);
        let command = Command::new(
            CommandSource::Curator,
            "look",
            Vec::new(),
            WallTime::new(1),
            Seed::new(Vec::new()),
        );
        assert!(rule.available_commands(&view, command.who()).is_empty());
        assert!(rule.check(&view, &command).is_ok());
        assert_eq!(
            rule.execute(&view, &command, &[])
                .expect("an output")
                .meaning()
                .as_str(),
            "niente"
        );
    }
}
