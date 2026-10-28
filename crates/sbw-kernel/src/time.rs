//! Time and position types (spec-01 §3, §4).

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
