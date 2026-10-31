//! Time and position types: plain data newtypes. Wall time enters the kernel
//! only as data in the command; `Position` is assigned by the event log, not
//! by the kernel; `WorldTime` is fiction time; `Seed` is dice material carried
//! by the command and never stored in state.

macro_rules! scalar_newtype {
    ($name:ident, $inner:ty, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $name($inner);

        impl $name {
            /// Wraps a raw value.
            pub const fn new(value: $inner) -> Self {
                Self(value)
            }

            /// Returns the wrapped value.
            pub const fn get(self) -> $inner {
                self.0
            }

            /// Serializes as 8 big-endian bytes.
            pub fn to_bytes(self) -> [u8; 8] {
                self.0.to_be_bytes()
            }

            /// Deserializes from 8 big-endian bytes; `None` on any other
            /// length.
            pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
                let raw: [u8; 8] = bytes.try_into().ok()?;
                Some(Self(<$inner>::from_be_bytes(raw)))
            }
        }
    };
}

scalar_newtype!(
    WallTime,
    i64,
    "Wall-clock time: the real-world clock. Read only by the runtime and \
     runtime and passed to the kernel as data in the command."
);
scalar_newtype!(
    Position,
    u64,
    "Ordinal position in the event log: 1, 2, 3, … Assigned by the event \
     the event log when an event is added, never chosen by the kernel."
);
scalar_newtype!(
    WorldTime,
    i64,
    "Fiction time: the in-game time. A datum in the state like any other; \
     any other; game mechanics can move it, rewind included."
);

/// Dice material: random bytes supplied by the runtime in the command.
/// Never stored in the state.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Seed(Vec<u8>);

impl Seed {
    /// Wraps raw dice material.
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// Returns the dice material as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Serializes as an 8-byte big-endian length prefix followed by the bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(8 + self.0.len());
        out.extend_from_slice(&(self.0.len() as u64).to_be_bytes());
        out.extend_from_slice(&self.0);
        out
    }

    /// Deserializes from the length-prefixed form; `None` if the prefix does
    /// not match the payload length.
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let (prefix, payload) = bytes.split_at_checked(8)?;
        let len = u64::from_be_bytes(prefix.try_into().ok()?);
        if payload.len() as u64 != len {
            return None;
        }
        Some(Self(payload.to_vec()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wall_time_is_ordered_and_roundtrips() {
        let t = WallTime::new(-5);
        assert_eq!(t.get(), -5);
        assert!(WallTime::new(-5) < WallTime::new(10));
        assert_eq!(t.to_bytes(), (-5i64).to_be_bytes());
        assert_eq!(WallTime::from_bytes(&t.to_bytes()), Some(t));
        assert!(WallTime::from_bytes(&[0u8; 7]).is_none());
    }

    #[test]
    fn position_is_ordered_and_roundtrips() {
        let p = Position::new(1);
        assert_eq!(p.get(), 1);
        assert!(Position::new(1) < Position::new(2));
        assert_eq!(p.to_bytes(), 1u64.to_be_bytes());
        assert_eq!(Position::from_bytes(&p.to_bytes()), Some(p));
        assert!(Position::from_bytes(&[0u8; 9]).is_none());
    }

    #[test]
    fn world_time_is_ordered_and_roundtrips() {
        let t = WorldTime::new(800);
        assert_eq!(t.get(), 800);
        assert!(WorldTime::new(800) < WorldTime::new(801));
        assert_eq!(t.to_bytes(), 800i64.to_be_bytes());
        assert_eq!(WorldTime::from_bytes(&t.to_bytes()), Some(t));
        assert!(WorldTime::from_bytes(&[0u8; 4]).is_none());
    }

    #[test]
    fn seed_preserves_bytes_and_roundtrips() {
        let s = Seed::new(vec![1, 2, 3]);
        assert_eq!(s.as_bytes(), &[1, 2, 3]);
        // serialized form: 8-byte big-endian length prefix + payload
        assert_eq!(&s.to_bytes()[..8], &3u64.to_be_bytes());
        assert_eq!(&s.to_bytes()[8..], &[1, 2, 3]);
        assert_eq!(Seed::from_bytes(&s.to_bytes()), Some(s.clone()));
        // empty seed is valid
        assert_eq!(
            Seed::from_bytes(&Seed::new(vec![]).to_bytes()),
            Some(Seed::new(vec![]))
        );
    }

    #[test]
    fn seed_rejects_malformed_bytes() {
        // length prefix longer than the payload
        let mut bad = 5u64.to_be_bytes().to_vec();
        bad.extend_from_slice(&[0u8; 2]);
        assert!(Seed::from_bytes(&bad).is_none());
        // truncated prefix
        assert!(Seed::from_bytes(&[0u8; 7]).is_none());
    }

    #[test]
    fn seed_is_ordered_by_bytes() {
        assert!(Seed::new(vec![1]) < Seed::new(vec![2]));
        assert!(Seed::new(vec![1]) < Seed::new(vec![1, 0]));
    }
}
