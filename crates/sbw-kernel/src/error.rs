//! Errors returned by the kernel's operations.

use crate::id::{ElementId, IncarnationId, ThreadId, VigilId};
use crate::refusal::Refusal;
use crate::time::WorldTime;

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

/// Why an event did not apply. The state the caller holds is never
/// modified on error: a mismatched "before" value is reported, never
/// silently overwritten.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ApplyError {
    /// An effect's recorded "before" value does not match the current
    /// state: the effect's position in the event, plus the recorded and
    /// current values.
    #[error("before value mismatch at effect {effect_index}: expected {expected}, got {actual}")]
    BeforeMismatch {
        effect_index: usize,
        expected: String,
        actual: String,
    },
    /// The effect references an element that is not in the world.
    #[error("unknown element at effect {effect_index}: {id:?}")]
    UnknownElement { effect_index: usize, id: ElementId },
    /// The effect's target container is not in the world.
    #[error("unknown container at effect {effect_index}: {id:?}")]
    UnknownContainer { effect_index: usize, id: ElementId },
    /// The effect would make an element its own direct or indirect
    /// container, breaking the single-container tree.
    #[error("container cycle at effect {effect_index}: {id:?} into {container:?}")]
    ContainerCycle {
        effect_index: usize,
        id: ElementId,
        container: ElementId,
    },
    /// The effect references an incarnation that is not in the world.
    #[error("unknown incarnation at effect {effect_index}: {id:?}")]
    UnknownIncarnation {
        effect_index: usize,
        id: IncarnationId,
    },
    /// The effect creates an element whose ID already exists.
    #[error("element already exists at effect {effect_index}: {id:?}")]
    DuplicateElement { effect_index: usize, id: ElementId },
    /// The effect opens a thread that is already open.
    #[error("thread already open at effect {effect_index}: {id:?}")]
    ThreadAlreadyOpen { effect_index: usize, id: ThreadId },
    /// The effect closes a thread that is not open.
    #[error("thread not open at effect {effect_index}: {id:?}")]
    ThreadNotOpen { effect_index: usize, id: ThreadId },
    /// The effect opens a vigil that is already open.
    #[error("vigil already open at effect {effect_index}: {id:?}")]
    VigilAlreadyOpen { effect_index: usize, id: VigilId },
    /// The effect closes a vigil that is not open.
    #[error("vigil not open at effect {effect_index}: {id:?}")]
    VigilNotOpen { effect_index: usize, id: VigilId },
    /// The effect cancels a deadline that is not scheduled.
    #[error("deadline not scheduled at effect {effect_index}: {world_time:?}")]
    DeadlineNotScheduled {
        effect_index: usize,
        world_time: WorldTime,
    },
}

impl ApplyError {
    /// The position in the event's effects of the effect that failed.
    pub fn effect_index(&self) -> usize {
        match self {
            ApplyError::BeforeMismatch { effect_index, .. }
            | ApplyError::UnknownElement { effect_index, .. }
            | ApplyError::UnknownContainer { effect_index, .. }
            | ApplyError::ContainerCycle { effect_index, .. }
            | ApplyError::UnknownIncarnation { effect_index, .. }
            | ApplyError::DuplicateElement { effect_index, .. }
            | ApplyError::ThreadAlreadyOpen { effect_index, .. }
            | ApplyError::ThreadNotOpen { effect_index, .. }
            | ApplyError::VigilAlreadyOpen { effect_index, .. }
            | ApplyError::VigilNotOpen { effect_index, .. }
            | ApplyError::DeadlineNotScheduled { effect_index, .. } => *effect_index,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::CommandParam;

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
            expected: "Some(1)".into(),
            actual: "None".into(),
        });
        assert!(apply_failed.to_string().contains("before value mismatch"));
        assert_ne!(refusal, limit);
        assert_ne!(refusal, apply_failed);
    }

    #[test]
    fn apply_error_always_names_its_effect_index() {
        let error = ApplyError::UnknownContainer {
            effect_index: 3,
            id: ElementId::from_raw(9),
        };
        assert_eq!(error.effect_index(), 3);
        assert!(error.to_string().contains("unknown container"));
    }
}
