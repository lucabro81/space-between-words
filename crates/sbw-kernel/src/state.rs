//! The world state: a fixed skeleton — elements, incarnations, vigils,
//! threads, deadlines, the fiction-time anchor — plus the attributes the
//! rules declare. The state is not the source of truth: it is rebuilt at
//! any time by applying the event log in order. Attributes are stored and
//! modified without interpreting them: names and meanings are the rules'.

use std::collections::{BTreeMap, BTreeSet};

use crate::effect::{AttributeValue, Deadline, TimeAnchor};
use crate::id::{ElementId, IncarnationId, ThreadId, VigilId};
use crate::time::WorldTime;

/// Identifies one attribute of one owner: an element or an incarnation plus
/// the attribute's name.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AttributeKey {
    owner: AttributeOwner,
    name: String,
}

impl AttributeKey {
    /// Creates a key. Takes ownership of the name: keys are stored.
    pub fn new(owner: AttributeOwner, name: impl Into<String>) -> Self {
        Self {
            owner,
            name: name.into(),
        }
    }

    /// The owner of the attribute.
    pub fn owner(&self) -> &AttributeOwner {
        &self.owner
    }

    /// The attribute's name.
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Who owns an attribute: an element or an incarnation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AttributeOwner {
    Element(ElementId),
    Incarnation(IncarnationId),
}

/// An element: anything notable defined by the setting, with a stable ID,
/// a type, and exactly one container. `None` means the world root: every
/// element is in one place at a time (world → location → object →
/// contained object).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ElementSkeleton {
    id: ElementId,
    element_type: String,
    container: Option<ElementId>,
}

impl ElementSkeleton {
    /// Creates an element. Takes ownership of the type: the skeleton is
    /// stored in the state.
    pub fn new(
        id: ElementId,
        element_type: impl Into<String>,
        container: Option<ElementId>,
    ) -> Self {
        Self {
            id,
            element_type: element_type.into(),
            container,
        }
    }

    /// The element's stable ID.
    pub fn id(&self) -> ElementId {
        self.id
    }

    /// The element's type, as declared by the setting.
    pub fn element_type(&self) -> &str {
        &self.element_type
    }

    /// The element's single container; `None` = the world root.
    pub fn container(&self) -> Option<ElementId> {
        self.container
    }
}

/// An incarnation: a character's presence in a world, located in an
/// element.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct IncarnationSkeleton {
    id: IncarnationId,
    location: ElementId,
}

impl IncarnationSkeleton {
    /// Creates an incarnation.
    pub fn new(id: IncarnationId, location: ElementId) -> Self {
        Self { id, location }
    }

    /// The incarnation's stable ID.
    pub fn id(&self) -> IncarnationId {
        self.id
    }

    /// Where the incarnation is.
    pub fn location(&self) -> ElementId {
        self.location
    }
}

/// An open vigil: the period an incarnation spends inside a world.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct VigilSkeleton {
    id: VigilId,
    incarnation: IncarnationId,
}

impl VigilSkeleton {
    /// Creates a vigil.
    pub fn new(id: VigilId, incarnation: IncarnationId) -> Self {
        Self { id, incarnation }
    }

    /// The vigil's stable ID.
    pub fn id(&self) -> VigilId {
        self.id
    }

    /// The incarnation the vigil belongs to.
    pub fn incarnation(&self) -> IncarnationId {
        self.incarnation
    }
}

/// An open thread, with the elements and incarnations it follows. An
/// event that involves one of them belongs to the thread.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ThreadSkeleton {
    id: ThreadId,
    elements: BTreeSet<ElementId>,
    incarnations: BTreeSet<IncarnationId>,
}

impl ThreadSkeleton {
    /// Creates a thread from the elements and incarnations it follows.
    pub fn new(
        id: ThreadId,
        elements: impl IntoIterator<Item = ElementId>,
        incarnations: impl IntoIterator<Item = IncarnationId>,
    ) -> Self {
        Self {
            id,
            elements: elements.into_iter().collect(),
            incarnations: incarnations.into_iter().collect(),
        }
    }

    /// The thread's stable ID.
    pub fn id(&self) -> ThreadId {
        self.id
    }

    /// The elements the thread follows.
    pub fn elements(&self) -> &BTreeSet<ElementId> {
        &self.elements
    }

    /// The incarnations the thread follows.
    pub fn incarnations(&self) -> &BTreeSet<IncarnationId> {
        &self.incarnations
    }
}

/// Error returned when an insertion violates the skeleton invariants.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum StateError {
    /// An element with this ID already exists.
    #[error("element already exists: {id:?}")]
    DuplicateElement { id: ElementId },
    /// The container must already be an element of the world.
    #[error("unknown container: {id:?}")]
    UnknownContainer { id: ElementId },
    /// An incarnation with this ID already exists.
    #[error("incarnation already exists: {id:?}")]
    DuplicateIncarnation { id: IncarnationId },
    /// The location must already be an element of the world.
    #[error("unknown location: {id:?}")]
    UnknownLocation { id: ElementId },
}

/// The complete snapshot of a world at an instant. Cloneable: the response
/// cycle works on a copy and only keeps it if the events apply cleanly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    elements: BTreeMap<ElementId, ElementSkeleton>,
    incarnations: BTreeMap<IncarnationId, IncarnationSkeleton>,
    open_vigils: BTreeMap<VigilId, VigilSkeleton>,
    open_threads: BTreeMap<ThreadId, ThreadSkeleton>,
    deadlines: BTreeMap<WorldTime, Vec<Deadline>>,
    time_anchor: TimeAnchor,
    attributes: BTreeMap<AttributeKey, AttributeValue>,
}

impl State {
    /// The empty world at a given time anchor. The log's genesis events
    /// populate it from there.
    pub fn new(time_anchor: TimeAnchor) -> Self {
        Self {
            elements: BTreeMap::new(),
            incarnations: BTreeMap::new(),
            open_vigils: BTreeMap::new(),
            open_threads: BTreeMap::new(),
            deadlines: BTreeMap::new(),
            time_anchor,
            attributes: BTreeMap::new(),
        }
    }

    /// Adds an element. The ID must be new and, when the element has a
    /// container, the container must already exist.
    pub fn add_element(&mut self, element: ElementSkeleton) -> Result<(), StateError> {
        if self.elements.contains_key(&element.id()) {
            return Err(StateError::DuplicateElement { id: element.id() });
        }
        if let Some(container) = element.container()
            && !self.elements.contains_key(&container)
        {
            return Err(StateError::UnknownContainer { id: container });
        }
        self.elements.insert(element.id(), element);
        Ok(())
    }

    /// Adds an incarnation. The ID must be new and the location must
    /// already be an element of the world.
    pub fn add_incarnation(&mut self, incarnation: IncarnationSkeleton) -> Result<(), StateError> {
        if self.incarnations.contains_key(&incarnation.id()) {
            return Err(StateError::DuplicateIncarnation {
                id: incarnation.id(),
            });
        }
        if !self.elements.contains_key(&incarnation.location()) {
            return Err(StateError::UnknownLocation {
                id: incarnation.location(),
            });
        }
        self.incarnations.insert(incarnation.id(), incarnation);
        Ok(())
    }

    /// Returns the value of one attribute, if it exists.
    pub fn attribute(&self, key: &AttributeKey) -> Option<&AttributeValue> {
        self.attributes.get(key)
    }

    /// Removes an element from the world. Its recorded attributes stay in
    /// the map, still keyed to the removed element's ID — preserved so a
    /// later recreation can pick them up. The existence check is the
    /// caller's: `apply` verifies it before calling.
    pub fn remove_element(&mut self, id: ElementId) {
        self.elements.remove(&id);
    }

    /// Moves an element to another container. `None` is the world root.
    /// The coherence of the move is the caller's: `apply` verifies the
    /// recorded "before" container first.
    pub fn move_element(&mut self, id: ElementId, container: Option<ElementId>) {
        if let Some(element) = self.elements.get_mut(&id) {
            element.container = container;
        }
    }

    /// Sets an attribute. The coherence of the recorded "before" value
    /// is the caller's: `apply` verifies it first.
    pub fn set_attribute(
        &mut self,
        owner: AttributeOwner,
        name: impl Into<String>,
        value: AttributeValue,
    ) {
        self.attributes
            .insert(AttributeKey::new(owner, name), value);
    }

    /// Opens a vigil, replacing any skeleton with the same ID.
    pub fn open_vigil(&mut self, vigil: VigilSkeleton) {
        self.open_vigils.insert(vigil.id(), vigil);
    }

    /// Closes a vigil. Closing an unknown vigil is a no-op: the coherence
    /// of the log is `apply`'s to verify before calling.
    pub fn close_vigil(&mut self, id: VigilId) {
        self.open_vigils.remove(&id);
    }

    /// Opens a thread, replacing any skeleton with the same ID.
    pub fn open_thread(&mut self, thread: ThreadSkeleton) {
        self.open_threads.insert(thread.id(), thread);
    }

    /// Closes a thread. Closing an unknown thread is a no-op: the
    /// coherence of the log is `apply`'s to verify before calling.
    pub fn close_thread(&mut self, id: ThreadId) {
        self.open_threads.remove(&id);
    }

    /// Schedules a deadline, appending it to the fiction time it matures
    /// at.
    pub fn schedule_deadline(&mut self, deadline: Deadline) {
        self.deadlines
            .entry(deadline.world_time())
            .or_default()
            .push(deadline);
    }

    /// Cancels one scheduled deadline: the first equal to it at its
    /// fiction time. Cancelling an unscheduled deadline is a no-op.
    pub fn cancel_deadline(&mut self, deadline: &Deadline) {
        if let Some(list) = self.deadlines.get_mut(&deadline.world_time()) {
            if let Some(position) = list.iter().position(|scheduled| scheduled == deadline) {
                list.remove(position);
            }
            if list.is_empty() {
                self.deadlines.remove(&deadline.world_time());
            }
        }
    }

    /// Replaces the fiction-time anchor. The coherence of the recorded
    /// "before" anchor is the caller's: `apply` verifies it first.
    pub fn set_time_anchor(&mut self, anchor: TimeAnchor) {
        self.time_anchor = anchor;
    }

    /// The elements of the world, in ID order.
    pub fn elements(&self) -> &BTreeMap<ElementId, ElementSkeleton> {
        &self.elements
    }

    /// The incarnations of the world, in ID order.
    pub fn incarnations(&self) -> &BTreeMap<IncarnationId, IncarnationSkeleton> {
        &self.incarnations
    }

    /// The open vigils, in ID order.
    pub fn open_vigils(&self) -> &BTreeMap<VigilId, VigilSkeleton> {
        &self.open_vigils
    }

    /// The open threads, in ID order.
    pub fn open_threads(&self) -> &BTreeMap<ThreadId, ThreadSkeleton> {
        &self.open_threads
    }

    /// The scheduled deadlines, indexed by fiction time and ordered by it.
    pub fn deadlines(&self) -> &BTreeMap<WorldTime, Vec<Deadline>> {
        &self.deadlines
    }

    /// The fiction-time anchor.
    pub fn time_anchor(&self) -> TimeAnchor {
        self.time_anchor
    }

    /// The attributes, in key order.
    pub fn attributes(&self) -> &BTreeMap<AttributeKey, AttributeValue> {
        &self.attributes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::Rate;
    use crate::time::WallTime;

    fn anchor() -> TimeAnchor {
        TimeAnchor::new(
            WorldTime::new(800),
            WallTime::new(0),
            Rate::new(1, 1).expect("positive denominator"),
        )
    }

    #[test]
    fn state_starts_empty_and_clones() {
        let state = State::new(anchor());
        assert!(state.elements().is_empty());
        assert!(state.incarnations().is_empty());
        assert!(state.open_vigils().is_empty());
        assert!(state.open_threads().is_empty());
        assert!(state.deadlines().is_empty());
        assert!(state.attributes().is_empty());
        assert_eq!(state.time_anchor(), anchor());
        let copy = state.clone();
        assert_eq!(state, copy);
    }

    #[test]
    fn add_element_enforces_its_invariants() {
        let mut state = State::new(anchor());
        let room = ElementSkeleton::new(ElementId::from_raw(1), "room", None);
        assert!(state.add_element(room.clone()).is_ok());
        // duplicate ID
        assert_eq!(
            state.add_element(room),
            Err(StateError::DuplicateElement {
                id: ElementId::from_raw(1)
            })
        );
        // container that does not exist yet
        assert_eq!(
            state.add_element(ElementSkeleton::new(
                ElementId::from_raw(2),
                "lantern",
                Some(ElementId::from_raw(99))
            )),
            Err(StateError::UnknownContainer {
                id: ElementId::from_raw(99)
            })
        );
        // once the container exists, the insertion succeeds
        assert!(
            state
                .add_element(ElementSkeleton::new(
                    ElementId::from_raw(3),
                    "shelf",
                    Some(ElementId::from_raw(1))
                ))
                .is_ok()
        );
    }

    #[test]
    fn add_incarnation_requires_a_known_location() {
        let mut state = State::new(anchor());
        state
            .add_element(ElementSkeleton::new(ElementId::from_raw(1), "room", None))
            .expect("room");
        let ada = IncarnationSkeleton::new(IncarnationId::from_raw(7), ElementId::from_raw(1));
        assert!(state.add_incarnation(ada.clone()).is_ok());
        // duplicate ID
        assert_eq!(
            state.add_incarnation(ada),
            Err(StateError::DuplicateIncarnation {
                id: IncarnationId::from_raw(7)
            })
        );
        // location that does not exist
        assert_eq!(
            state.add_incarnation(IncarnationSkeleton::new(
                IncarnationId::from_raw(8),
                ElementId::from_raw(42)
            )),
            Err(StateError::UnknownLocation {
                id: ElementId::from_raw(42)
            })
        );
    }

    #[test]
    fn cancel_deadline_removes_one_of_two_identical_deadlines() {
        use crate::effect::{Deadline, DeadlinePayload};
        let mut state = State::new(anchor());
        let deadline = Deadline::new(WorldTime::new(900), DeadlinePayload::new("wake", vec![]));
        state.schedule_deadline(deadline.clone());
        state.schedule_deadline(deadline.clone());
        state.cancel_deadline(&deadline);
        assert_eq!(
            state.deadlines().get(&WorldTime::new(900)),
            Some(&vec![deadline.clone()])
        );
        state.cancel_deadline(&deadline);
        assert!(state.deadlines().get(&WorldTime::new(900)).is_none());
    }

    #[test]
    fn attributes_have_no_values_before_the_log_is_applied() {
        // populating the attributes is `apply`'s job; here the map is only
        // reachable for reading and starts empty
        let state = State::new(anchor());
        assert!(state.attributes().is_empty());
        let key = AttributeKey::new(AttributeOwner::Element(ElementId::from_raw(1)), "lit");
        assert_eq!(state.attribute(&key), None);
    }
}
