//! The event: a fact that happened in the world, decided by the kernel and
//! stored in the event log. It never changes once written. An event carries
//! its meaning (open vocabulary), its effects (closed vocabulary), who was
//! involved, the wall time the runtime supplied, and the recorded dice
//! outcomes.

use crate::effect::Effect;
use crate::id::{ElementId, IncarnationId, ThreadId, VigilId};
use crate::meaning::Meaning;
use crate::time::WallTime;

/// An incarnation or an element an event involves.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum InvolvedEntity {
    Incarnation(IncarnationId),
    Element(ElementId),
}

/// A recorded dice outcome: why the roll was made and its result. The dice
/// machinery has its own unit of work; here the event only carries the
/// record, so replay never re-rolls.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DiceOutcome {
    reason: String,
    result: i64,
}

impl DiceOutcome {
    /// Records an outcome. Takes ownership: the record is stored with the
    /// event.
    pub fn new(reason: impl Into<String>, result: i64) -> Self {
        Self {
            reason: reason.into(),
            result,
        }
    }

    /// Why the roll was made ("attack, 1d20").
    pub fn reason(&self) -> &str {
        &self.reason
    }

    /// The outcome of the roll.
    pub fn result(&self) -> i64 {
        self.result
    }
}

/// A fact that happened in the world. Built only by the kernel: `exposed`
/// is not a constructor parameter and there is no way to set it — every
/// event the kernel produces starts not exposed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    meaning: Meaning,
    effects: Vec<Effect>,
    involved: Vec<InvolvedEntity>,
    vigil: Option<VigilId>,
    threads: Vec<ThreadId>,
    wall_time: WallTime,
    dice_outcomes: Vec<DiceOutcome>,
    exposed: bool,
}

impl Event {
    /// Builds an event: what it means, the state changes it carries, who
    /// was involved, at most one vigil, any threads it belongs to, the
    /// wall time the runtime supplied, and the recorded dice outcomes.
    pub fn new(
        meaning: Meaning,
        effects: Vec<Effect>,
        involved: Vec<InvolvedEntity>,
        vigil: Option<VigilId>,
        threads: Vec<ThreadId>,
        wall_time: WallTime,
        dice_outcomes: Vec<DiceOutcome>,
    ) -> Self {
        Self {
            meaning,
            effects,
            involved,
            vigil,
            threads,
            wall_time,
            dice_outcomes,
            // not a parameter: an event the kernel produces is never
            // exposed, and nothing in the API can change that
            exposed: false,
        }
    }

    /// The meaning of the event, in the author's open vocabulary.
    pub fn meaning(&self) -> &Meaning {
        &self.meaning
    }

    /// The state changes the event produces, from the closed vocabulary.
    pub fn effects(&self) -> &[Effect] {
        &self.effects
    }

    /// The incarnations and elements the event involves.
    pub fn involved(&self) -> &[InvolvedEntity] {
        &self.involved
    }

    /// The vigil the event happened in, if any: an event without players
    /// (a monster moving at midnight) has none.
    pub fn vigil(&self) -> Option<VigilId> {
        self.vigil
    }

    /// The threads the event belongs to, assigned by the kernel when it
    /// produces the event.
    pub fn threads(&self) -> &[ThreadId] {
        &self.threads
    }

    /// The wall time the runtime supplied with the command; chronicle
    /// only, it decides nothing.
    pub fn wall_time(&self) -> WallTime {
        self.wall_time
    }

    /// The recorded dice outcomes; replay reads them and never re-rolls.
    pub fn dice_outcomes(&self) -> &[DiceOutcome] {
        &self.dice_outcomes
    }

    /// Whether the event may ever be sent to other worlds. Always `false`
    /// for now: who decides exposure is an open design question, and the
    /// kernel offers no way to set it.
    pub fn exposed(&self) -> bool {
        self.exposed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_event() -> Event {
        Event::new(
            Meaning::new("preso"),
            vec![],
            vec![InvolvedEntity::Element(ElementId::from_bytes(&1u64.to_be_bytes())
                .expect("8 bytes"))],
            Some(VigilId::from_bytes(&7u64.to_be_bytes()).expect("8 bytes")),
            vec![],
            WallTime::new(1_000),
            vec![DiceOutcome::new("attack, 1d20", 14)],
        )
    }

    #[test]
    fn event_is_never_exposed() {
        // the constructor takes no `exposed` and there is no setter:
        // every event starts not exposed
        let event = sample_event();
        assert!(!event.exposed());
        assert!(!event.clone().exposed());
    }

    #[test]
    fn event_carries_its_parts() {
        let event = sample_event();
        assert_eq!(event.meaning().as_str(), "preso");
        assert!(event.effects().is_empty());
        assert_eq!(event.involved().len(), 1);
        assert_eq!(
            event.vigil(),
            Some(VigilId::from_bytes(&7u64.to_be_bytes()).expect("8 bytes"))
        );
        assert!(event.threads().is_empty());
        assert_eq!(event.wall_time(), WallTime::new(1_000));
        assert_eq!(event.dice_outcomes()[0].result(), 14);
        assert_eq!(event.dice_outcomes()[0].reason(), "attack, 1d20");
    }

    #[test]
    fn involved_entity_covers_both_kinds() {
        let inc = InvolvedEntity::Incarnation(IncarnationId::from_bytes(&1u64.to_be_bytes())
            .expect("8 bytes"));
        let el = InvolvedEntity::Element(ElementId::from_bytes(&2u64.to_be_bytes())
            .expect("8 bytes"));
        assert_ne!(inc, el);
    }
}
