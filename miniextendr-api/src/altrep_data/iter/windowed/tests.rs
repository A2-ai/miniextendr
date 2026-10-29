use super::WindowedIterState;
use std::panic::{AssertUnwindSafe, catch_unwind};

#[test]
fn a_read_behind_the_window_is_an_error() {
    let state = WindowedIterState::new(0..10, 10, 3);
    assert_eq!(state.get_element(0), Some(0));
    assert_eq!(state.get_element(5), Some(5));
    assert!(catch_unwind(AssertUnwindSafe(|| state.get_element(1))).is_err());
    assert_eq!(state.get_element(4), Some(4));
    assert_eq!(state.get_element(6), Some(6));
}

#[test]
fn materialization_before_the_window_moves_reads_exactly_len() {
    let source = || (0..4).chain(std::iter::once_with(|| panic!("read beyond the length")));
    let state = WindowedIterState::new(source(), 4, 2);
    assert_eq!(state.materialize_all(), [0, 1, 2, 3]);

    let state = WindowedIterState::new(source(), 4, 2);
    assert_eq!(state.get_element(1), Some(1));
    assert_eq!(state.materialize_all(), [0, 1, 2, 3]);
    assert_eq!(state.get_element(3), Some(3));
}

#[test]
fn materialization_after_the_window_moves_is_an_error() {
    let state = WindowedIterState::new(0..6, 6, 2);
    assert_eq!(state.get_element(4), Some(4));
    assert!(catch_unwind(AssertUnwindSafe(|| state.materialize_all())).is_err());
    assert!(state.as_materialized().is_none());
    assert_eq!(state.get_element(4), Some(4));
    assert_eq!(state.get_element(5), Some(5));
}

#[test]
fn early_exhaustion_never_publishes_a_partial_buffer() {
    let state = WindowedIterState::new(0..2, 3, 4);
    for _ in 0..2 {
        assert!(catch_unwind(AssertUnwindSafe(|| state.materialize_all())).is_err());
        assert!(state.as_materialized().is_none());
        assert_eq!(state.get_element(0), Some(0));
        assert_eq!(state.get_element(1), Some(1));
        assert_eq!(state.get_element(2), None);
    }
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

    // Exhausted while filling the window.
    let state = WindowedIterState::new(Flaky(0), 3, 2);
    assert_eq!(state.get_element(0), Some(0));
    assert_eq!(state.get_element(1), None);
    assert_eq!(state.get_element(1), None);

    // Exhausted while skipping to the window.
    let state = WindowedIterState::new(Flaky(0), 5, 1);
    assert_eq!(state.get_element(2), None);
    assert_eq!(state.get_element(2), None);
    assert!(catch_unwind(AssertUnwindSafe(|| state.get_element(0))).is_err());
}
