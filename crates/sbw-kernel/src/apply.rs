//! `apply`: the kernel's second operation. Given a state and an event, it
//! produces the new state using only the event's effects — never the
//! author's rules. This is what makes replaying the event log independent
//! of rule changes: every effect that carries a "before" value is verified
//! against the current state before anything is modified, and effects
//! without one are checked for the entity they act on — the element, the
//! container, the incarnation, the thread or vigil, the deadline.
//!
//! Pure: no I/O, no clock, no randomness, and the rules are never invoked.
//! The input state is never mutated: `apply` works on a clone and returns
//! the modified copy, or an error with nothing modified. Dice outcomes
//! recorded in the event are never re-rolled: they don't reach the state.

use crate::effect::Effect;
use crate::error::ApplyError;
use crate::event::Event;
use crate::id::ElementId;
use crate::state::{
    AttributeKey, AttributeOwner, ElementSkeleton, State, ThreadSkeleton, VigilSkeleton,
};

/// Applies an event's effects, in order, to a state. Returns the new
/// state, or an error that leaves the caller's state untouched. Every
/// "before" value in the effects must match the current state: a
/// mismatch is a reported error, never a silent modification. The rules
/// are never invoked: the effects alone rebuild the state, which is why
/// replaying the log stays correct when the rules change.
pub fn apply(state: &State, event: &Event) -> Result<State, ApplyError> {
    let mut next = state.clone();
    for (index, effect) in event.effects().iter().enumerate() {
        apply_effect(&mut next, effect, index)?;
    }
    Ok(next)
}

/// Applies one effect to a working copy of the state, verifying its
/// "before" value and referenced entities first.
fn apply_effect(state: &mut State, effect: &Effect, index: usize) -> Result<(), ApplyError> {
    match effect {
        Effect::CreateElement {
            id,
            element_type,
            container,
            attributes,
        } => {
            if state.elements().contains_key(id) {
                return Err(ApplyError::DuplicateElement {
                    effect_index: index,
                    id: *id,
                });
            }
            if let Some(container) = container
                && !state.elements().contains_key(container)
            {
                return Err(ApplyError::UnknownContainer {
                    effect_index: index,
                    id: *container,
                });
            }
            state
                .add_element(ElementSkeleton::new(*id, element_type.clone(), *container))
                .map_err(|_| ApplyError::DuplicateElement {
                    effect_index: index,
                    id: *id,
                })?;
            for (name, value) in attributes {
                state.set_attribute(AttributeOwner::Element(*id), name.clone(), value.clone());
            }
        }
        Effect::RemoveElement { id } => {
            if !state.elements().contains_key(id) {
                return Err(ApplyError::UnknownElement {
                    effect_index: index,
                    id: *id,
                });
            }
            state.remove_element(*id);
        }
        Effect::MoveElement { id, from, to } => {
            let current = state
                .elements()
                .get(id)
                .map(|element| element.container())
                .ok_or(ApplyError::UnknownElement {
                    effect_index: index,
                    id: *id,
                })?;
            if current != *from {
                return Err(ApplyError::BeforeMismatch {
                    effect_index: index,
                    expected: format!("{from:?}"),
                    actual: format!("{current:?}"),
                });
            }
            if let Some(to) = to {
                if !state.elements().contains_key(to) {
                    return Err(ApplyError::UnknownContainer {
                        effect_index: index,
                        id: *to,
                    });
                }
                // the world is a single-container tree: moving an element
                // into itself or into one of its own descendants would
                // create a cycle
                if would_cycle(state, *id, *to) {
                    return Err(ApplyError::ContainerCycle {
                        effect_index: index,
                        id: *id,
                        container: *to,
                    });
                }
            }
            state.move_element(*id, *to);
        }
        Effect::SetAttribute {
            owner,
            name,
            old,
            new,
        } => {
            match owner {
                AttributeOwner::Element(id) if !state.elements().contains_key(id) => {
                    return Err(ApplyError::UnknownElement {
                        effect_index: index,
                        id: *id,
                    });
                }
                AttributeOwner::Incarnation(id) if !state.incarnations().contains_key(id) => {
                    return Err(ApplyError::UnknownIncarnation {
                        effect_index: index,
                        id: *id,
                    });
                }
                _ => {}
            }
            let current = state.attribute(&AttributeKey::new(owner.clone(), name.clone()));
            if current != old.as_ref() {
                return Err(ApplyError::BeforeMismatch {
                    effect_index: index,
                    expected: format!("{old:?}"),
                    actual: format!("{current:?}"),
                });
            }
            state.set_attribute(owner.clone(), name.clone(), new.clone());
        }
        Effect::OpenThread {
            id,
            elements,
            incarnations,
        } => {
            if state.open_threads().contains_key(id) {
                return Err(ApplyError::ThreadAlreadyOpen {
                    effect_index: index,
                    id: *id,
                });
            }
            state.open_thread(ThreadSkeleton::new(
                *id,
                elements.iter().copied(),
                incarnations.iter().copied(),
            ));
        }
        Effect::CloseThread { id } => {
            if !state.open_threads().contains_key(id) {
                return Err(ApplyError::ThreadNotOpen {
                    effect_index: index,
                    id: *id,
                });
            }
            state.close_thread(*id);
        }
        Effect::ScheduleDeadline { deadline } => {
            state.schedule_deadline(deadline.clone());
        }
        Effect::CancelDeadline { deadline } => {
            if !state
                .deadlines()
                .get(&deadline.world_time())
                .is_some_and(|list| list.contains(deadline))
            {
                return Err(ApplyError::DeadlineNotScheduled {
                    effect_index: index,
                    world_time: deadline.world_time(),
                });
            }
            state.cancel_deadline(deadline);
        }
        Effect::SetTimeAnchor { old, new } => {
            if state.time_anchor() != *old {
                return Err(ApplyError::BeforeMismatch {
                    effect_index: index,
                    expected: format!("{old:?}"),
                    actual: format!("{:?}", state.time_anchor()),
                });
            }
            state.set_time_anchor(*new);
        }
        Effect::OpenVigil { id, incarnation } => {
            if state.open_vigils().contains_key(id) {
                return Err(ApplyError::VigilAlreadyOpen {
                    effect_index: index,
                    id: *id,
                });
            }
            if !state.incarnations().contains_key(incarnation) {
                return Err(ApplyError::UnknownIncarnation {
                    effect_index: index,
                    id: *incarnation,
                });
            }
            state.open_vigil(VigilSkeleton::new(*id, *incarnation));
        }
        Effect::CloseVigil { id } => {
            if !state.open_vigils().contains_key(id) {
                return Err(ApplyError::VigilNotOpen {
                    effect_index: index,
                    id: *id,
                });
            }
            state.close_vigil(*id);
        }
    }
    Ok(())
}

/// Whether making `container` the container of `element` would create a
/// cycle: walking up from `container`, do we reach `element`? The walk is
/// bounded by the number of elements: on a well-formed tree it reaches the
/// root well within that, so a longer walk means the state itself is
/// already cyclic — and the move is rejected either way. `apply` never
/// creates a cycle itself, but the signature accepts any `&State`, so the
/// walk must always terminate.
fn would_cycle(state: &State, element: ElementId, container: ElementId) -> bool {
    let mut cursor = Some(container);
    let mut steps = 0usize;
    while let Some(current) = cursor {
        if current == element {
            return true;
        }
        steps += 1;
        if steps > state.elements().len() {
            return true;
        }
        cursor = state
            .elements()
            .get(&current)
            .and_then(|skeleton| skeleton.container());
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::{AttributeValue, Rate, TimeAnchor};
    use crate::id::{ElementId, IncarnationId, ThreadId, VigilId};
    use crate::meaning::Meaning;
    use crate::time::{WallTime, WorldTime};

    fn element(raw: u64) -> ElementId {
        ElementId::from_raw(raw)
    }

    fn anchor() -> TimeAnchor {
        TimeAnchor::new(
            WorldTime::new(800),
            WallTime::new(0),
            Rate::new(1, 1).expect("positive denominator"),
        )
    }

    fn event(effects: Vec<Effect>) -> Event {
        Event::new(
            Meaning::new("preso"),
            effects,
            Vec::new(),
            None,
            Vec::new(),
            WallTime::new(1_000),
            Vec::new(),
        )
    }

    fn world_with_lantern() -> State {
        let mut state = State::new(anchor());
        state
            .add_element(ElementSkeleton::new(element(1), "room", None))
            .expect("room");
        state
            .add_element(ElementSkeleton::new(
                element(2),
                "lantern",
                Some(element(1)),
            ))
            .expect("lantern");
        state
            .add_element(ElementSkeleton::new(element(3), "door", Some(element(1))))
            .expect("door");
        state
            .add_element(ElementSkeleton::new(element(8), "inventory", None))
            .expect("inventory");
        state
            .add_incarnation(crate::state::IncarnationSkeleton::new(
                IncarnationId::from_raw(7),
                element(1),
            ))
            .expect("ada");
        state
    }

    #[test]
    fn apply_moves_the_lantern_without_mutating_the_input() {
        let state = world_with_lantern();
        let event = event(vec![Effect::MoveElement {
            id: element(2),
            from: Some(element(1)),
            to: Some(element(8)),
        }]);
        let next = apply(&state, &event).expect("applies");
        assert_eq!(
            next.elements()
                .get(&element(2))
                .expect("lantern")
                .container(),
            Some(element(8))
        );
        assert_eq!(
            state
                .elements()
                .get(&element(2))
                .expect("lantern")
                .container(),
            Some(element(1))
        );
    }

    #[test]
    fn apply_reports_a_mismatched_before_value() {
        let state = world_with_lantern();
        let event = event(vec![Effect::MoveElement {
            id: element(2),
            from: Some(element(3)),
            to: Some(element(8)),
        }]);
        let error = apply(&state, &event).expect_err("mismatched before value");
        assert_eq!(error.effect_index(), 0);
        assert!(matches!(
            error,
            ApplyError::BeforeMismatch {
                effect_index: 0,
                ..
            }
        ));
        assert!(error.to_string().contains("expected"));
    }

    #[test]
    fn apply_rejects_creating_a_duplicate_element() {
        let state = world_with_lantern();
        let event = event(vec![Effect::CreateElement {
            id: element(2),
            element_type: "lantern".into(),
            container: Some(element(1)),
            attributes: Vec::new(),
        }]);
        assert!(matches!(
            apply(&state, &event),
            Err(ApplyError::DuplicateElement { .. })
        ));
    }

    #[test]
    fn apply_rejects_a_move_from_a_wrong_container() {
        let state = world_with_lantern();
        let event = event(vec![Effect::MoveElement {
            id: element(2),
            from: Some(element(8)),
            to: Some(element(1)),
        }]);
        assert!(matches!(
            apply(&state, &event),
            Err(ApplyError::BeforeMismatch { .. })
        ));
    }

    #[test]
    fn apply_rejects_a_stale_attribute_before_value() {
        let mut state = world_with_lantern();
        state.set_attribute(
            AttributeOwner::Element(element(2)),
            "lit",
            AttributeValue::Boolean(true),
        );
        let event = event(vec![Effect::SetAttribute {
            owner: AttributeOwner::Element(element(2)),
            name: "lit".into(),
            old: None,
            new: AttributeValue::Boolean(false),
        }]);
        assert!(matches!(
            apply(&state, &event),
            Err(ApplyError::BeforeMismatch { .. })
        ));
    }

    #[test]
    fn apply_rejects_closing_an_unopen_vigil_or_thread() {
        let state = world_with_lantern();
        let vigil = event(vec![Effect::CloseVigil {
            id: VigilId::from_raw(5),
        }]);
        assert!(apply(&state, &vigil).is_err());
        let thread = event(vec![Effect::CloseThread {
            id: ThreadId::from_raw(9),
        }]);
        assert!(apply(&state, &thread).is_err());
    }

    #[test]
    fn apply_rejects_opening_an_already_open_vigil_or_thread() {
        let mut state = world_with_lantern();
        state.open_vigil(VigilSkeleton::new(
            VigilId::from_raw(5),
            IncarnationId::from_raw(7),
        ));
        state.open_thread(ThreadSkeleton::new(
            ThreadId::from_raw(9),
            [element(2)],
            [IncarnationId::from_raw(7)],
        ));
        let vigil = event(vec![Effect::OpenVigil {
            id: VigilId::from_raw(5),
            incarnation: IncarnationId::from_raw(7),
        }]);
        assert!(apply(&state, &vigil).is_err());
        let thread = event(vec![Effect::OpenThread {
            id: ThreadId::from_raw(9),
            elements: vec![element(2)],
            incarnations: vec![IncarnationId::from_raw(7)],
        }]);
        assert!(apply(&state, &thread).is_err());
    }

    #[test]
    fn apply_rejects_a_stale_time_anchor() {
        let state = world_with_lantern();
        let event = event(vec![Effect::SetTimeAnchor {
            old: TimeAnchor::new(
                WorldTime::new(999),
                WallTime::new(0),
                Rate::new(1, 1).expect("positive denominator"),
            ),
            new: anchor(),
        }]);
        assert!(matches!(
            apply(&state, &event),
            Err(ApplyError::BeforeMismatch { .. })
        ));
    }

    #[test]
    fn apply_rejects_a_move_into_itself_or_a_descendant() {
        let state = world_with_lantern();
        // direct self-container
        let self_move = event(vec![Effect::MoveElement {
            id: element(1),
            from: None,
            to: Some(element(1)),
        }]);
        assert!(matches!(
            apply(&state, &self_move),
            Err(ApplyError::ContainerCycle { .. })
        ));
        // indirect: the room into the lantern it contains
        let cycle = event(vec![Effect::MoveElement {
            id: element(1),
            from: None,
            to: Some(element(2)),
        }]);
        assert!(matches!(
            apply(&state, &cycle),
            Err(ApplyError::ContainerCycle { .. })
        ));
        // moving to the world root is fine instead
        let to_root = event(vec![Effect::MoveElement {
            id: element(2),
            from: Some(element(1)),
            to: None,
        }]);
        assert!(apply(&state, &to_root).is_ok());
    }

    #[test]
    fn apply_rejects_unknown_referenced_entities() {
        let state = world_with_lantern();
        let create = event(vec![Effect::CreateElement {
            id: element(11),
            element_type: "coin".into(),
            container: Some(element(99)),
            attributes: Vec::new(),
        }]);
        assert!(matches!(
            apply(&state, &create),
            Err(ApplyError::UnknownContainer { .. })
        ));
        let move_to = event(vec![Effect::MoveElement {
            id: element(2),
            from: Some(element(1)),
            to: Some(element(99)),
        }]);
        assert!(matches!(
            apply(&state, &move_to),
            Err(ApplyError::UnknownContainer { .. })
        ));
        let vigil = event(vec![Effect::OpenVigil {
            id: VigilId::from_raw(5),
            incarnation: IncarnationId::from_raw(42),
        }]);
        assert!(matches!(
            apply(&state, &vigil),
            Err(ApplyError::UnknownIncarnation { .. })
        ));
    }

    #[test]
    fn apply_rejects_an_attribute_on_a_missing_owner() {
        let state = world_with_lantern();
        let on_element = event(vec![Effect::SetAttribute {
            owner: AttributeOwner::Element(element(99)),
            name: "lit".into(),
            old: None,
            new: AttributeValue::Boolean(true),
        }]);
        assert!(matches!(
            apply(&state, &on_element),
            Err(ApplyError::UnknownElement { .. })
        ));
        let on_incarnation = event(vec![Effect::SetAttribute {
            owner: AttributeOwner::Incarnation(IncarnationId::from_raw(42)),
            name: "hunger".into(),
            old: None,
            new: AttributeValue::Integer(1),
        }]);
        assert!(matches!(
            apply(&state, &on_incarnation),
            Err(ApplyError::UnknownIncarnation { .. })
        ));
    }

    #[test]
    fn apply_rejects_an_unscheduled_deadline_cancellation() {
        use crate::effect::{Deadline, DeadlinePayload};
        let state = world_with_lantern();
        let deadline = Deadline::new(
            WorldTime::new(900),
            DeadlinePayload::new("wake", vec![AttributeValue::Integer(3)]),
        );
        let event = event(vec![Effect::CancelDeadline { deadline }]);
        assert!(apply(&state, &event).is_err());
    }
}
