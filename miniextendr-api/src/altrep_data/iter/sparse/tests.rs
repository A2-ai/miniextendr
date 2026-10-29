use super::SparseIterState;
use std::cell::Cell;
use std::rc::Rc;

#[test]
fn exhausted_sparse_sources_never_resume() {
    // The third poll would resurrect the source if we kept it after None.
    let polls = Cell::new(0);
    let source = std::iter::from_fn(|| {
        let poll = polls.get() + 1;
        polls.set(poll);
        match poll {
            1 => Some(10),
            3 => Some(20),
            _ => None,
        }
    });
    let state = SparseIterState::new(source, 4);
    assert_eq!(state.get_element(0), Some(10));
    assert_eq!(state.get_element(2), None); // Ends while nth skips index 1.
    assert_eq!(state.iterator_position(), None);
    assert_eq!(state.get_element(2), None);
    assert_eq!(state.get_element(1), None);
    assert_eq!(state.get_element(0), Some(10)); // Cache survives EOF.
    assert_eq!(polls.get(), 2);
}

#[test]
fn declared_prefix_drops_the_source_without_an_extra_poll() {
    for len in [0, 3] {
        // The closure owns the second handle until the state drops it.
        let owner = Rc::new(());
        let captured = Rc::clone(&owner);
        let polls = Cell::new(0);
        let poll_count = &polls;
        let source = std::iter::from_fn(move || {
            let _keep_alive = &captured;
            let poll = poll_count.get();
            assert!(poll < len, "read beyond the declared prefix");
            poll_count.set(poll + 1);
            Some(poll)
        });
        let state = SparseIterState::new(source, len);
        if len != 0 {
            assert_eq!(state.iterator_position(), Some(0));
            assert_eq!(state.get_element(len - 1), Some(len - 1));
            assert_eq!(state.get_element(0), None); // Skipped, not cached.
            assert_eq!(state.get_element(len - 1), Some(len - 1));
        }
        assert_eq!(state.iterator_position(), None);
        assert_eq!(state.get_element(len), None);
        assert_eq!(polls.get(), len);
        assert_eq!(Rc::strong_count(&owner), 1);
    }
}
