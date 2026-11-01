//! The effect vocabulary: the closed set of state changes an event can
//! carry, each with its before/after values where applicable, plus the
//! value, rate, anchor and deadline types those effects reference. The
//! vocabulary is fixed: a change to it is a new framework version, which is
//! the price of replaying the log without the author's rules.

use crate::id::{ElementId, IncarnationId, ThreadId, VigilId};
use crate::state::AttributeOwner;
use crate::time::{WallTime, WorldTime};

/// The value an attribute can hold: integers, text, booleans or a reference
/// to an element. No floating point anywhere.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AttributeValue {
    Integer(i64),
    Text(String),
    Boolean(bool),
    Id(ElementId),
}

/// Error returned when a `Rate` is built with a non-positive denominator.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RateError {
    /// A ratio with a zero or negative denominator is not a rate.
    #[error("rate denominator must be positive, got {denominator}")]
    NonPositiveDenominator { denominator: i64 },
}

/// How fast fiction time advances per unit of wall time, as a ratio of two
/// integers. A numerator of zero means fiction time is paused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Rate {
    numerator: i64,
    denominator: i64,
}

impl Rate {
    /// Creates a rate. Returns a structured error if `denominator` is zero
    /// or negative.
    pub fn new(numerator: i64, denominator: i64) -> Result<Self, RateError> {
        if denominator <= 0 {
            return Err(RateError::NonPositiveDenominator { denominator });
        }
        Ok(Self {
            numerator,
            denominator,
        })
    }

    /// The numerator: fiction-time units.
    pub fn numerator(self) -> i64 {
        self.numerator
    }

    /// The denominator: wall-time units.
    pub fn denominator(self) -> i64 {
        self.denominator
    }
}

/// The fiction-time anchor: the fiction time that held at a given wall
/// time, and the rate at which fiction time advances from there. The
/// current fiction time is computed from the anchor and the wall time of
/// each time command; the anchor only changes through an event (pause, rate
/// change, rewind, jump), which records old and new.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeAnchor {
    world_time: WorldTime,
    wall_time: WallTime,
    rate: Rate,
}

impl TimeAnchor {
    /// Creates an anchor.
    pub fn new(world_time: WorldTime, wall_time: WallTime, rate: Rate) -> Self {
        Self {
            world_time,
            wall_time,
            rate,
        }
    }

    /// The fiction time that held at the anchor.
    pub fn world_time(self) -> WorldTime {
        self.world_time
    }

    /// The wall time the anchor refers to.
    pub fn wall_time(self) -> WallTime {
        self.wall_time
    }

    /// The rate fiction time advances at from the anchor.
    pub fn rate(self) -> Rate {
        self.rate
    }
}

/// What a deadline wants evaluated when it matures: an open-vocabulary
/// action label plus parameters, both defined by the rules. The kernel
/// stores and forwards them without interpreting them.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DeadlinePayload {
    action: String,
    params: Vec<AttributeValue>,
}

impl DeadlinePayload {
    /// Creates a payload. Takes ownership: the payload is stored with the
    /// deadline.
    pub fn new(action: impl Into<String>, params: Vec<AttributeValue>) -> Self {
        Self {
            action: action.into(),
            params,
        }
    }

    /// The action label.
    pub fn action(&self) -> &str {
        &self.action
    }

    /// The parameters.
    pub fn params(&self) -> &[AttributeValue] {
        &self.params
    }
}

/// A scheduled wake-up: at fiction time `world_time`, the payload is
/// evaluated. Deadlines always refer to fiction time, never to wall time,
/// so moving fiction time (rewind included) keeps them coherent.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Deadline {
    world_time: WorldTime,
    payload: DeadlinePayload,
}

impl Deadline {
    /// Creates a deadline.
    pub fn new(world_time: WorldTime, payload: DeadlinePayload) -> Self {
        Self {
            world_time,
            payload,
        }
    }

    /// The fiction time at which the deadline matures.
    pub fn world_time(&self) -> WorldTime {
        self.world_time
    }

    /// What gets evaluated when the deadline matures.
    pub fn payload(&self) -> &DeadlinePayload {
        &self.payload
    }
}

/// The closed vocabulary of state changes an event can produce. Exactly
/// eleven primitives; each carries its before value, its after value, or
/// both, so replaying the log needs no rule to re-run. There is no
/// primitive for timelines: the world has one, and splitting it is an act
/// of design, never an event.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Effect {
    /// Create an element: its id, type, initial attributes and its
    /// container (`None` = the world root).
    CreateElement {
        id: ElementId,
        element_type: String,
        container: Option<ElementId>,
        attributes: Vec<(String, AttributeValue)>,
    },
    /// Remove an element from the world.
    RemoveElement { id: ElementId },
    /// Move an element from one container to another (`None` = the world
    /// root).
    MoveElement {
        id: ElementId,
        from: Option<ElementId>,
        to: Option<ElementId>,
    },
    /// Set an attribute of an element or an incarnation. `old` is `None`
    /// when the attribute did not exist before.
    SetAttribute {
        owner: AttributeOwner,
        name: String,
        old: Option<AttributeValue>,
        new: AttributeValue,
    },
    /// Open a thread: its id plus the elements and incarnations it
    /// follows. The followed entities travel with the effect so replay
    /// rebuilds the thread exactly as it was opened — the log is the only
    /// source of truth.
    OpenThread {
        id: ThreadId,
        elements: Vec<ElementId>,
        incarnations: Vec<IncarnationId>,
    },
    /// Close a thread.
    CloseThread { id: ThreadId },
    /// Schedule a deadline.
    ScheduleDeadline { deadline: Deadline },
    /// Cancel a deadline.
    CancelDeadline { deadline: Deadline },
    /// Set the fiction-time anchor: the old anchor and the new one. Rewind
    /// is a move plus an anchor set with before/after swapped — no extra
    /// primitive.
    SetTimeAnchor { old: TimeAnchor, new: TimeAnchor },
    /// Open a vigil: its id plus the incarnation it belongs to. The
    /// incarnation travels with the effect so replay rebuilds the vigil
    /// exactly as it was opened.
    OpenVigil {
        id: VigilId,
        incarnation: IncarnationId,
    },
    /// Close a vigil.
    CloseVigil { id: VigilId },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_rejects_non_positive_denominator() {
        assert_eq!(
            Rate::new(1, 0),
            Err(RateError::NonPositiveDenominator { denominator: 0 })
        );
        assert_eq!(
            Rate::new(1, -3),
            Err(RateError::NonPositiveDenominator { denominator: -3 })
        );
        // a zero numerator is valid: paused fiction time
        let paused = Rate::new(0, 2).expect("positive denominator");
        assert_eq!(paused.numerator(), 0);
        assert_eq!(paused.denominator(), 2);
        let running = Rate::new(2, 1).expect("positive denominator");
        assert!(paused < running);
    }

    #[test]
    fn time_anchor_holds_its_three_parts() {
        let anchor = TimeAnchor::new(
            WorldTime::new(800),
            WallTime::new(1_000),
            Rate::new(1, 1).expect("positive denominator"),
        );
        assert_eq!(anchor.world_time(), WorldTime::new(800));
        assert_eq!(anchor.wall_time(), WallTime::new(1_000));
        assert_eq!(anchor.rate().numerator(), 1);
        // Copy: the anchor moves freely
        let copied = anchor;
        assert_eq!(copied, anchor);
    }

    #[test]
    fn deadline_carries_time_and_payload() {
        let deadline = Deadline::new(
            WorldTime::new(900),
            DeadlinePayload::new("wake", vec![AttributeValue::Integer(3)]),
        );
        assert_eq!(deadline.world_time(), WorldTime::new(900));
        assert_eq!(deadline.payload().action(), "wake");
        assert_eq!(deadline.payload().params(), &[AttributeValue::Integer(3)]);
    }

    #[test]
    fn attribute_values_cover_the_four_kinds() {
        let a = ElementId::from_bytes(&1u64.to_be_bytes()).expect("8 bytes");
        let b = ElementId::from_bytes(&2u64.to_be_bytes()).expect("8 bytes");
        // each kind compares within itself: equal values are equal,
        // different values are not
        assert_eq!(AttributeValue::Integer(42), AttributeValue::Integer(42));
        assert_ne!(AttributeValue::Integer(42), AttributeValue::Integer(43));
        assert_eq!(
            AttributeValue::Text("a".into()),
            AttributeValue::Text("a".into())
        );
        assert_ne!(
            AttributeValue::Text("a".into()),
            AttributeValue::Text("b".into())
        );
        assert_eq!(AttributeValue::Boolean(true), AttributeValue::Boolean(true));
        assert_ne!(
            AttributeValue::Boolean(true),
            AttributeValue::Boolean(false)
        );
        assert_eq!(AttributeValue::Id(a), AttributeValue::Id(a));
        assert_ne!(AttributeValue::Id(a), AttributeValue::Id(b));
        // the four kinds are distinct from each other and ordered
        assert_ne!(
            AttributeValue::Integer(42),
            AttributeValue::Text("a".into())
        );
        assert_ne!(AttributeValue::Boolean(true), AttributeValue::Id(a));
        assert!(AttributeValue::Boolean(true) < AttributeValue::Id(a));
        assert!(AttributeValue::Integer(42) < AttributeValue::Text("a".into()));
    }
}
