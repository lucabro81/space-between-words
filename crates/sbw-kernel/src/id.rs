//! Opaque ID newtypes (AGENTS.md principle 8).

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
