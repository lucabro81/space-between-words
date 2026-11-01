//! The command: a structured request entering the kernel from the runtime.
//! Not free text — who asks, which action, on what — plus the data only the
//! runtime can supply: the wall time and the dice material. Nothing a model
//! produces enters the kernel directly: a human's input is first translated
//! into one of the available commands.

use crate::id::{ElementId, IncarnationId};
use crate::time::{Seed, WallTime};

/// Who issues a command.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CommandSource {
    /// A player character, by incarnation ID.
    Incarnation(IncarnationId),
    /// An editor or game master. Acts only via commands, like everyone else.
    Curator,
    /// The world itself: the passage of time arrives as a command from it.
    World,
}

/// One parameter of a command: an element, an incarnation, or a plain value.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CommandParam {
    /// An element of the world.
    Element(ElementId),
    /// An incarnation.
    Incarnation(IncarnationId),
    /// An integer value.
    Integer(i64),
    /// A text value.
    Text(String),
    /// A boolean value.
    Boolean(bool),
}

/// A structured request: who, what action, on what, plus the runtime data.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Command {
    who: CommandSource,
    action: String,
    params: Vec<CommandParam>,
    wall_time: WallTime,
    dice_material: Seed,
}

impl Command {
    /// Creates a command. Takes ownership of the action and the parameters:
    /// they are stored.
    pub fn new(
        who: CommandSource,
        action: impl Into<String>,
        params: Vec<CommandParam>,
        wall_time: WallTime,
        dice_material: Seed,
    ) -> Self {
        Self {
            who,
            action: action.into(),
            params,
            wall_time,
            dice_material,
        }
    }

    /// Who issued the command.
    pub fn who(&self) -> &CommandSource {
        &self.who
    }

    /// The action type ("take"). Not a closed list: the rules define them.
    pub fn action(&self) -> &str {
        &self.action
    }

    /// The parameters: what the action is about.
    pub fn params(&self) -> &[CommandParam] {
        &self.params
    }

    /// The wall time the runtime supplied. Chronicle only: it decides
    /// nothing.
    pub fn wall_time(&self) -> WallTime {
        self.wall_time
    }

    /// The dice material the runtime supplied. The kernel expands it into
    /// rolls and records them in the event; it is never stored in the state.
    pub fn dice_material(&self) -> &Seed {
        &self.dice_material
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_source_covers_the_three_issuers() {
        let inc = CommandSource::Incarnation(IncarnationId::from_raw(7));
        assert_ne!(inc, CommandSource::Curator);
        assert_ne!(CommandSource::Curator, CommandSource::World);
        assert!(CommandSource::Curator < CommandSource::World);
    }

    #[test]
    fn command_param_covers_elements_incarnations_and_values() {
        let element = CommandParam::Element(ElementId::from_raw(1));
        let incarnation = CommandParam::Incarnation(IncarnationId::from_raw(2));
        assert_ne!(element, incarnation);
        assert_eq!(CommandParam::Integer(3), CommandParam::Integer(3));
        assert_ne!(
            CommandParam::Text("a".into()),
            CommandParam::Text("b".into())
        );
        assert_ne!(CommandParam::Boolean(true), CommandParam::Boolean(false));
    }

    #[test]
    fn command_keeps_all_its_parts() {
        let command = Command::new(
            CommandSource::World,
            "time-passes",
            vec![CommandParam::Integer(60)],
            WallTime::new(1_000),
            Seed::new(vec![1, 2, 3]),
        );
        assert_eq!(command.who(), &CommandSource::World);
        assert_eq!(command.action(), "time-passes");
        assert_eq!(command.params(), &[CommandParam::Integer(60)]);
        assert_eq!(command.wall_time(), WallTime::new(1_000));
        assert_eq!(command.dice_material().as_bytes(), &[1, 2, 3]);
        let copied = command.clone();
        assert_eq!(command, copied);
    }
}
