//! Opaque ID newtypes (AGENTS.md principle 8): compare, order and serialize,
//! nothing else. The internal representation is a placeholder `u64` — the
//! real structure is designed in block 13, and these newtypes hide it.

macro_rules! opaque_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $name(u64);

        impl $name {
            /// Test-only constructor (AGENTS.md principle 6): IDs have no
            /// public constructor until their structure is designed.
            #[cfg(test)]
            pub fn from_raw(raw: u64) -> Self {
                Self(raw)
            }

            /// Serializes the ID as 8 big-endian bytes.
            pub fn to_bytes(self) -> [u8; 8] {
                self.0.to_be_bytes()
            }

            /// Deserializes an ID from its 8 big-endian bytes; `None` on any
            /// other length.
            pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
                let raw: [u8; 8] = bytes.try_into().ok()?;
                Some(Self(u64::from_be_bytes(raw)))
            }
        }
    };
}

opaque_id!(
    /// A player. Opaque; structure not designed yet (block 13).
    PlayerId
);
opaque_id!(
    /// An incarnation: a character's presence in a world. Opaque; structure
    /// not designed yet (block 13).
    IncarnationId
);
opaque_id!(
    /// A world. Opaque; structure not designed yet (block 13).
    WorldId
);
opaque_id!(
    /// An element: anything notable defined by the setting, with a stable ID.
    ElementId
);
opaque_id!(
    /// A thread: a story (a quest, a feud) spanning multiple events.
    ThreadId
);
opaque_id!(
    /// A vigil: the period in which an incarnation is inside a world.
    VigilId
);

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_id_type {
        ($name:ident, $t:ty) => {
            #[test]
            fn $name() {
                // compare
                let a = <$t>::from_raw(1);
                let b = <$t>::from_raw(2);
                assert_eq!(a, <$t>::from_raw(1));
                assert_ne!(a, b);
                // order
                assert!(a < b);
                assert!(b > a);
                // copy
                let copied = a;
                assert_eq!(a, copied);
                // serialize: fixed-width big-endian roundtrip
                assert_eq!(a.to_bytes(), 1u64.to_be_bytes());
                assert_eq!(<$t>::from_bytes(&a.to_bytes()), Some(a));
                // serialize: wrong lengths are rejected, not panicked on
                assert!(<$t>::from_bytes(&[0u8; 7]).is_none());
                assert!(<$t>::from_bytes(&[0u8; 9]).is_none());
            }
        };
    }

    test_id_type!(player_id_traits, PlayerId);
    test_id_type!(incarnation_id_traits, IncarnationId);
    test_id_type!(world_id_traits, WorldId);
    test_id_type!(element_id_traits, ElementId);
    test_id_type!(thread_id_traits, ThreadId);
    test_id_type!(vigil_id_traits, VigilId);
}
