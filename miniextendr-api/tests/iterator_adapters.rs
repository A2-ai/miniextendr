mod r_test_utils;

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

#[test]
fn native_altrep_construction_is_lazy_and_reads_only_a_finite_prefix() {
    use miniextendr_api::{AltIntegerData, AltrepIteratorExt, AltrepLen};
    use std::cell::Cell;

    let draws = Cell::new(0i32);
    let data = std::iter::from_fn(|| {
        let value = draws.get();
        draws.set(value + 1);
        Some(value)
    })
    .into_altrep_data(3);
    assert_eq!(draws.get(), 0);
    assert_eq!(data.len(), 3);
    assert_eq!(data.elt(1), 1);
    assert_eq!(data.elt(0), 0);
    assert_eq!(draws.get(), 2);
    assert_eq!(data.elt(2), 2);
    assert_eq!(data.as_slice(), Some([0, 1, 2].as_slice()));
    assert_eq!(draws.get(), 3);
}

#[test]
fn native_altrep_accepts_composed_and_unknown_length_iterators() {
    use miniextendr_api::{AltIntegerData, AltrepIteratorExt, AltrepLen};

    let exact = [1i32, 2, 3, 4].into_iter().rev().map(|x| x * 2).take(3);
    let data = exact.into_altrep_data_exact();
    assert_eq!(data.len(), 3);
    assert_eq!(data.elt(2), 4);

    let filtered = (0..10i32).filter(|x| x % 2 == 0).into_altrep_data(5);
    assert_eq!(filtered.elt(4), 8);
    assert_eq!(filtered.elt(1), 2);
}

#[test]
fn native_logical_iterator_retains_na_and_noncanonical_true_storage() {
    use miniextendr_api::{AltLogicalData, AltrepIteratorExt, RLogical};

    let data = [RLogical::FALSE, RLogical::from_i32(42), RLogical::NA]
        .into_iter()
        .into_altrep_data_exact();
    let mut out = [0; 3];
    assert_eq!(data.get_region(0, 3, &mut out), 3);
    assert_eq!(out, [0, 42, i32::MIN]);
    assert_eq!(data.as_r_slice(), Some(out.as_slice()));
}

#[test]
fn a_local_element_type_can_choose_its_data_family() {
    use miniextendr_api::coerce::Coerce;
    use miniextendr_api::{
        AltIntegerData, AltrepIteratorExt, AltrepIteratorItem, IterIntCoerceData,
    };

    #[derive(Clone, Copy)]
    struct Small(u16);
    impl Coerce<i32> for Small {
        fn coerce(self) -> i32 {
            i32::from(self.0)
        }
    }
    impl AltrepIteratorItem for Small {
        type Data<I: Iterator<Item = Self>> = IterIntCoerceData<I, Self>;
        fn into_data<I: Iterator<Item = Self>>(iter: I, len: usize) -> Self::Data<I> {
            IterIntCoerceData::from_iter(iter, len)
        }
    }
    let data = [Small(3), Small(7)].into_iter().into_altrep_data_exact();
    assert_eq!(data.elt(1), 7);
}

#[test]
fn tuple_iterator_results_cross_the_trait_abi_conversion_layer() {
    use miniextendr_api::from_r::TryFromSexp;
    use miniextendr_api::gc_protect::ProtectScope;
    use miniextendr_api::trait_abi::conv::to_sexp;
    use miniextendr_api::{SEXPTYPE, SexpExt};

    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let iter = RefCell::new(
            ["a".to_owned(), "b".to_owned(), "c".to_owned()]
                .into_iter()
                .enumerate(),
        );
        let hint = scope.protect(to_sexp(iter.size_hint())).get();
        assert_eq!(
            <(i64, Option<i64>)>::try_from_sexp(hint).unwrap(),
            (3, Some(3))
        );
        let first = scope.protect(to_sexp(iter.next())).get();
        assert_eq!(
            Option::<(usize, String)>::try_from_sexp(first).unwrap(),
            Some((0, "a".into()))
        );
        let rest = scope.protect(to_sexp(iter.collect_n(10))).get();
        assert_eq!(
            Vec::<(usize, String)>::try_from_sexp(rest).unwrap(),
            vec![(1, "b".into()), (2, "c".into())]
        );
        let done = scope.protect(to_sexp(iter.next())).get();
        assert_eq!(done.type_of(), SEXPTYPE::NILSXP);

        let unknown = RefCell::new(std::iter::from_fn(|| Some((7i32, "x".to_string()))));
        let hint = scope.protect(to_sexp(unknown.size_hint())).get();
        assert_eq!(
            <(i64, Option<i64>)>::try_from_sexp(hint).unwrap(),
            (0, None)
        );
    });
}
