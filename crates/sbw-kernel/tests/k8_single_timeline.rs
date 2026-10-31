//! K8 acceptance test: the effect vocabulary contains no primitive for
//! creating, choosing or merging timelines.
//!
//! Observable outcome: one instance of every `Effect` variant is mapped to
//! its name by an exhaustive match, and the resulting set must be exactly
//! the eleven fixed primitives — none of which creates, chooses or merges
//! timelines. Adding any new variant (a timeline primitive included) breaks
//! the exhaustive match until it is listed here, and a name outside the
//! expected list fails the assertion.

use sbw_kernel::effect::{AttributeValue, Deadline, DeadlinePayload, Effect, Rate, TimeAnchor};
use sbw_kernel::id::{ElementId, ThreadId, VigilId};
use sbw_kernel::state::AttributeOwner;
use sbw_kernel::time::{WallTime, WorldTime};

fn element_id(raw: u64) -> ElementId {
    ElementId::from_bytes(&raw.to_be_bytes()).expect("8 bytes")
}

fn thread_id(raw: u64) -> ThreadId {
    ThreadId::from_bytes(&raw.to_be_bytes()).expect("8 bytes")
}

fn vigil_id(raw: u64) -> VigilId {
    VigilId::from_bytes(&raw.to_be_bytes()).expect("8 bytes")
}

fn anchor() -> TimeAnchor {
    TimeAnchor::new(
        WorldTime::new(800),
        WallTime::new(0),
        Rate::new(1, 1).expect("positive denominator"),
    )
}

fn deadline() -> Deadline {
    Deadline::new(WorldTime::new(900), DeadlinePayload::new("wake", vec![]))
}

/// Maps every possible effect to its variant name. Exhaustive by design:
/// a new variant does not compile until it is listed here.
fn variant_name(effect: &Effect) -> &'static str {
    match effect {
        Effect::CreateElement { .. } => "create_element",
        Effect::RemoveElement { .. } => "remove_element",
        Effect::MoveElement { .. } => "move_element",
        Effect::SetAttribute { .. } => "set_attribute",
        Effect::OpenThread { .. } => "open_thread",
        Effect::CloseThread { .. } => "close_thread",
        Effect::ScheduleDeadline { .. } => "schedule_deadline",
        Effect::CancelDeadline { .. } => "cancel_deadline",
        Effect::SetTimeAnchor { .. } => "set_time_anchor",
        Effect::OpenVigil { .. } => "open_vigil",
        Effect::CloseVigil { .. } => "close_vigil",
    }
}

#[test]
fn k8_effect_vocabulary_has_no_timeline_primitives() {
    let effects = vec![
        Effect::CreateElement {
            id: element_id(1),
            element_type: "lantern".into(),
            container: None,
            attributes: vec![("lit".into(), AttributeValue::Boolean(true))],
        },
        Effect::RemoveElement { id: element_id(1) },
        Effect::MoveElement {
            id: element_id(1),
            from: None,
            to: Some(element_id(2)),
        },
        Effect::SetAttribute {
            owner: AttributeOwner::Element(element_id(1)),
            name: "lit".into(),
            old: None,
            new: AttributeValue::Boolean(true),
        },
        Effect::OpenThread { id: thread_id(1) },
        Effect::CloseThread { id: thread_id(1) },
        Effect::ScheduleDeadline { deadline: deadline() },
        Effect::CancelDeadline { deadline: deadline() },
        Effect::SetTimeAnchor {
            old: anchor(),
            new: anchor(),
        },
        Effect::OpenVigil { id: vigil_id(1) },
        Effect::CloseVigil { id: vigil_id(1) },
    ];

    let mut names: Vec<&str> = effects.iter().map(variant_name).collect();
    names.sort_unstable();

    assert_eq!(
        names,
        [
            "cancel_deadline",
            "close_thread",
            "close_vigil",
            "create_element",
            "move_element",
            "open_thread",
            "open_vigil",
            "remove_element",
            "schedule_deadline",
            "set_attribute",
            "set_time_anchor",
        ]
    );
}
