use super::IterState;
use std::panic::{AssertUnwindSafe, catch_unwind};

#[test]
fn materialization_never_probes_past_the_declared_prefix() {
    for len in [0, 3] {
        let source = (0..len)
            .map(|i| i.to_string())
            .chain(std::iter::once_with(|| panic!("read beyond the prefix")));
        let state = IterState::new(source, len);
        let expected: Vec<_> = (0..len).map(|i| i.to_string()).collect();
        assert_eq!(state.materialize_all(), expected);
        assert_eq!(state.materialize_all(), expected);
    }
}

#[test]
fn materialization_continues_after_a_cached_prefix() {
    let source = (0..4).chain(std::iter::once_with(|| panic!("read beyond the prefix")));
    let state = IterState::new(source, 4);
    assert_eq!(state.get_element(1), Some(1));
    assert_eq!(state.materialize_all(), [0, 1, 2, 3]);
}

#[test]
fn an_exhausted_source_is_never_polled_again() {
    // Yields 0, then None, then 1 if polled again (not fused).
    struct Flaky(u8);
    impl Iterator for Flaky {
        type Item = i32;
        fn next(&mut self) -> Option<i32> {
            self.0 += 1;
            match self.0 {
                1 => Some(0),
                3 => Some(1),
                _ => None,
            }
        }
    }
    let state = IterState::new(Flaky(0), 2);
    assert_eq!(state.get_element(1), None);
    assert_eq!(state.get_element(1), None);
    assert_eq!(state.get_element(0), Some(0));
}

#[test]
fn early_exhaustion_never_publishes_a_partial_buffer() {
    let state = IterState::new(0..2, 3);
    for _ in 0..2 {
        assert!(catch_unwind(AssertUnwindSafe(|| state.materialize_all())).is_err());
        assert!(state.as_materialized().is_none());
        assert_eq!(state.get_element(0), Some(0));
        assert_eq!(state.get_element(2), None);
    }
    assert_eq!(state.try_materialize_all(), Err(2));
}

#[test]
fn clone_preserves_a_partial_prefix_without_advancing_the_source() {
    let original = IterState::new(10..20i32, 6);
    assert_eq!(original.get_element(2), Some(12));
    let cloned = original.clone();
    assert!(original.as_materialized().is_none());
    assert!(cloned.as_materialized().is_none());
    assert_eq!(cloned.materialize_all(), &[10, 11, 12, 13, 14, 15]);
    assert!(original.as_materialized().is_none());
    assert_eq!(original.get_element(3), Some(13));
    assert_eq!(original.materialize_all(), cloned.materialize_all());
}

#[test]
fn restored_values_need_no_iterator_constructor() {
    type Source = std::iter::Take<std::iter::Repeat<i32>>;
    let restored = IterState::<Source, i32>::from_values(vec![4, 5, 6]);
    assert_eq!(restored.len(), 3);
    assert_eq!(restored.get_element(1), Some(5));
    assert_eq!(restored.materialize_all(), &[4, 5, 6]);
}
