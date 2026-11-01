//! Shared fixtures for the kernel's integration tests: ID helpers and the
//! ordinary-case world (Ada in the Cantina with a lantern, a door and an
//! inventory). Each test binary includes the whole module; helpers it does
//! not use are allowed to stay.

#![allow(dead_code)]

use sbw_kernel::command::{Command, CommandParam, CommandSource};
use sbw_kernel::effect::{Rate, TimeAnchor};
use sbw_kernel::id::{ElementId, IncarnationId};
use sbw_kernel::state::{ElementSkeleton, IncarnationSkeleton, State};
use sbw_kernel::time::{Seed, WallTime, WorldTime};

pub const CANTINA: u64 = 1;
pub const LANTERN: u64 = 2;
pub const DOOR: u64 = 3;
pub const ALTRO: u64 = 4;
pub const ADA: u64 = 7;
pub const INVENTORY: u64 = 8;

pub fn element_id(raw: u64) -> ElementId {
    ElementId::from_bytes(&raw.to_be_bytes()).expect("8 bytes")
}

pub fn incarnation_id(raw: u64) -> IncarnationId {
    IncarnationId::from_bytes(&raw.to_be_bytes()).expect("8 bytes")
}

pub fn anchor() -> TimeAnchor {
    TimeAnchor::new(
        WorldTime::new(800),
        WallTime::new(0),
        Rate::new(1, 1).expect("positive denominator"),
    )
}

/// The ordinary-case world: the Cantina (root) holds the lantern and the
/// door; Ada is in the Cantina; Ada's inventory is a root-level element.
pub fn world_with_lantern() -> State {
    let mut state = State::new(anchor());
    state
        .add_element(ElementSkeleton::new(element_id(CANTINA), "room", None))
        .expect("room");
    state
        .add_element(ElementSkeleton::new(
            element_id(INVENTORY),
            "inventory",
            None,
        ))
        .expect("inventory");
    state
        .add_element(ElementSkeleton::new(
            element_id(LANTERN),
            "lantern",
            Some(element_id(CANTINA)),
        ))
        .expect("lantern");
    state
        .add_element(ElementSkeleton::new(
            element_id(DOOR),
            "door",
            Some(element_id(CANTINA)),
        ))
        .expect("door");
    state
        .add_incarnation(IncarnationSkeleton::new(
            incarnation_id(ADA),
            element_id(CANTINA),
        ))
        .expect("ada");
    state
}

/// Same world, but the lantern is in another room: not available to Ada.
pub fn world_with_lantern_elsewhere() -> State {
    let mut state = State::new(anchor());
    state
        .add_element(ElementSkeleton::new(element_id(CANTINA), "room", None))
        .expect("room");
    state
        .add_element(ElementSkeleton::new(element_id(ALTRO), "room", None))
        .expect("other room");
    state
        .add_element(ElementSkeleton::new(
            element_id(LANTERN),
            "lantern",
            Some(element_id(ALTRO)),
        ))
        .expect("lantern");
    state
        .add_element(ElementSkeleton::new(
            element_id(DOOR),
            "door",
            Some(element_id(CANTINA)),
        ))
        .expect("door");
    state
        .add_incarnation(IncarnationSkeleton::new(
            incarnation_id(ADA),
            element_id(CANTINA),
        ))
        .expect("ada");
    state
}

/// Ada's command: take the lantern.
pub fn take_lantern() -> Command {
    Command::new(
        CommandSource::Incarnation(incarnation_id(ADA)),
        "take",
        vec![CommandParam::Element(element_id(LANTERN))],
        WallTime::new(1_000),
        Seed::new(vec![9, 9, 9]),
    )
}

/// A world whose IDs match the known raw values: the Cantina (1) holds the
/// lantern (2); Ada (7) is in the Cantina.
pub fn event_id_free_world() -> State {
    world_with_lantern()
}
