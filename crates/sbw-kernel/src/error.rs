//! Errors returned by the kernel's operations.

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
    /// An event produced by the rules did not apply cleanly to the working
    /// copy: its recorded "before" values disagree with the state the
    /// response built so far. The whole response is discarded — no events,
    /// the state is unchanged.
    #[error("an event did not apply: {0}")]
    ApplyFailed(#[from] ApplyError),
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
        let apply_failed = DecideError::ApplyFailed(ApplyError::BeforeMismatch {
            effect_index: 0,
            detail: "element missing".into(),
        });
        assert!(apply_failed.to_string().contains("before value mismatch"));
        assert_ne!(refusal, limit);
        assert_ne!(refusal, apply_failed);
    }
}

/// Why an event did not apply. The state the caller holds is never
/// modified on error: a mismatched "before" value is reported, never
/// silently overwritten.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ApplyError {
    /// An effect's recorded "before" value does not match the current
    /// state: the effect's position in the event and what differed.
    #[error("before value mismatch at effect {effect_index}: {detail}")]
    BeforeMismatch { effect_index: usize, detail: String },
}

impl ApplyError {
    /// The position in the event's effects of the effect that failed.
    pub fn effect_index(&self) -> Option<usize> {
        match self {
            ApplyError::BeforeMismatch { effect_index, .. } => Some(*effect_index),
        }
    }
}
