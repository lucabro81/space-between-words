//! The refusal: a command that makes no sense in the current state (the
//! lantern is nailed down, the sword is not here). A refusal is not a
//! failure — a failed dice roll is an execution that produces an event —
//! and it is never recorded: zero events, the state unchanged. The kernel
//! hands the structured reason to the runtime, which tells the player.

use crate::command::CommandParam;

/// A structured rejection: a machine-readable code plus the parameters the
/// code refers to (e.g. `"not-present"` and the sword's element ID).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Refusal {
    code: String,
    params: Vec<CommandParam>,
}

impl Refusal {
    /// Creates a refusal. Takes ownership of the code: the reason is stored.
    pub fn new(code: impl Into<String>, params: Vec<CommandParam>) -> Self {
        Self {
            code: code.into(),
            params,
        }
    }

    /// The machine-readable reason code.
    pub fn code(&self) -> &str {
        &self.code
    }

    /// The parameters the code refers to.
    pub fn params(&self) -> &[CommandParam] {
        &self.params
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::ElementId;

    #[test]
    fn refusal_carries_code_and_params() {
        let refusal = Refusal::new(
            "nailed-down",
            vec![CommandParam::Element(ElementId::from_raw(2))],
        );
        assert_eq!(refusal.code(), "nailed-down");
        assert_eq!(
            refusal.params(),
            &[CommandParam::Element(ElementId::from_raw(2))]
        );
        let copied = refusal.clone();
        assert_eq!(refusal, copied);
    }
}
