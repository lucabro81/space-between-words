//! Errors returned by the kernel's `decide` operation.

use crate::refusal::Refusal;

/// Why a command produced no response.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DecideError {
    /// The command makes no sense in the current state: refused with its
    /// structured reason. No events are produced and the state is unchanged.
    #[error("command refused: {0:?}")]
    Refusal(Refusal),
    /// The reaction chain exceeded the depth limit: the whole response is
    /// discarded — no events are produced and the state is unchanged.
    #[error("reaction depth limit exceeded: {limit}")]
    DepthLimitExceeded { limit: usize },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::CommandParam;
    use crate::id::ElementId;

    #[test]
    fn decide_error_carries_its_variants() {
        let refusal = DecideError::Refusal(Refusal::new(
            "nailed-down",
            vec![CommandParam::Element(ElementId::from_raw(2))],
        ));
        assert!(refusal.to_string().contains("nailed-down"));
        let limit = DecideError::DepthLimitExceeded { limit: 100 };
        assert_eq!(limit.to_string(), "reaction depth limit exceeded: 100");
        assert_ne!(refusal, limit);
    }
}
