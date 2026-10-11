use miniextendr_api::adapter_traits::RIterator;
use std::cell::RefCell;

#[test]
fn blanket_iterator_accepts_arbitrary_items_and_bounds_consumption() {
    let iter = RefCell::new(
        ["a", "b", "c", "d"]
            .into_iter()
            .enumerate()
            .map(|(i, s)| (i, s.to_owned())),
    );
    assert_eq!(iter.collect_n(-1), []);
    assert_eq!(iter.nth(-1), None);
    assert_eq!(iter.size_hint(), (4, Some(4)));
    assert_eq!(
        iter.collect_n(2),
        [(0, "a".to_owned()), (1, "b".to_owned())]
    );
    assert_eq!(iter.skip(1), 1);
    assert_eq!(iter.next(), Some((3, "d".to_owned())));
    assert_eq!(iter.next(), None);
}

#[test]
fn blanket_iterator_uses_specialized_nth() {
    struct Specialized;
    impl Iterator for Specialized {
        type Item = usize;
        fn next(&mut self) -> Option<usize> {
            panic!("specialized operations must not fall back to next")
        }
        fn nth(&mut self, n: usize) -> Option<usize> {
            Some(n)
        }
    }
    let iter = RefCell::new(Specialized);
    assert_eq!(iter.nth(17), Some(17));
    let range = RefCell::new(0..42);
    assert_eq!(range.count(), 42);
    assert_eq!(range.next(), None);
}

#[test]
fn infinite_size_hint_does_not_wrap_to_negative() {
    let iter = RefCell::new(std::iter::repeat(7));
    let (lower, upper) = iter.size_hint();
    assert!(lower > 0);
    assert_eq!(upper, None);
    assert_eq!(iter.collect_n(3), [7, 7, 7]);
}
