//! Registered iterator classes must preserve lazy evaluation and writable storage.
mod r_test_utils;

use std::cell::Cell;
use std::rc::Rc;

use miniextendr_api::altrep_data::{AltrepIterator, AltrepSerialize};
use miniextendr_api::altrep_traits::{AltComplex, AltInteger, AltRaw, AltReal, AltVec, Altrep};
use miniextendr_api::{IntoR, OwnedProtect, Rcomplex, SexpExt, TryFromSexp};

#[test]
fn reads_cache_one_prefix_and_dataptr_preserves_writes() {
    r_test_utils::with_r_thread(|| {
        let calls = Rc::new(Cell::new(0));
        let count = calls.clone();
        let iter = (0..8).inspect(move |_| count.set(count.get() + 1));
        let x = unsafe {
            OwnedProtect::new(miniextendr_api::Altrep::from_exact_iter(iter).into_sexp())
        };
        type Data = AltrepIterator<i32>;
        assert!(x.get().is_altrep());
        assert_eq!(calls.get(), 0);
        assert!(<Data as AltVec>::dataptr_or_null(x.get()).is_null());
        assert_eq!(<Data as AltInteger>::elt(x.get(), 3), 3);
        assert_eq!(calls.get(), 4);
        assert_eq!(<Data as AltInteger>::elt(x.get(), 0), 0);
        assert_eq!(calls.get(), 4);
        let mut out = [-1; 3];
        assert_eq!(<Data as AltInteger>::get_region(x.get(), 5, 9, &mut out), 3);
        assert_eq!(out, [5, 6, 7]);
        assert_eq!(calls.get(), 8);
        let pointer = <Data as AltVec>::dataptr(x.get(), true).cast::<i32>();
        unsafe { *pointer = 90 };
        assert_eq!(<Data as AltInteger>::elt(x.get(), 0), 90);
        let state = unsafe { OwnedProtect::new(<Data as Altrep>::serialized_state(x.get())) };
        assert_eq!(
            Vec::<i32>::try_from_sexp(state.get()).unwrap(),
            [90, 1, 2, 3, 4, 5, 6, 7]
        );
        assert_eq!(calls.get(), 8);
    });
}

#[test]
fn bounded_iterators_never_probe_beyond_the_declared_prefix() {
    r_test_utils::with_r_thread(|| {
        for len in [0usize, 3] {
            let calls = Rc::new(Cell::new(0));
            let count = calls.clone();
            let iter = (0..).inspect(move |_| count.set(count.get() + 1));
            let x = unsafe {
                OwnedProtect::new(
                    miniextendr_api::Altrep::from_iter_with_len(iter, len).into_sexp(),
                )
            };
            let state = unsafe {
                OwnedProtect::new(<AltrepIterator<i32> as Altrep>::serialized_state(x.get()))
            };
            assert_eq!(calls.get(), len);
            assert_eq!(Vec::<i32>::try_from_sexp(state.get()).unwrap().len(), len);
        }
    });
}

#[test]
fn native_iterator_families_restore_owned_mutable_buffers() {
    r_test_utils::with_r_thread(|| {
        macro_rules! check {
            ($elem:ty, $family:ident, $values:expr, $replacement:expr) => {{
                let expected: Vec<$elem> = $values;
                let x = unsafe {
                    OwnedProtect::new(
                        miniextendr_api::Altrep::from_exact_iter(expected.clone().into_iter())
                            .into_sexp(),
                    )
                };
                assert!(x.get().is_altrep());
                let state = unsafe {
                    OwnedProtect::new(<AltrepIterator<$elem> as Altrep>::serialized_state(x.get()))
                };
                let restored =
                    <AltrepIterator<$elem> as AltrepSerialize>::unserialize(state.get()).unwrap();
                drop(state);
                unsafe { miniextendr_api::sys::R_gc() };
                let y = unsafe { OwnedProtect::new(miniextendr_api::Altrep(restored).into_sexp()) };
                assert_eq!(Vec::<$elem>::try_from_sexp(y.get()).unwrap(), expected);
                let pointer =
                    <AltrepIterator<$elem> as AltVec>::dataptr(y.get(), true).cast::<$elem>();
                unsafe { *pointer = $replacement };
                assert_eq!(
                    <AltrepIterator<$elem> as $family>::elt(y.get(), 0),
                    $replacement
                );
            }};
        }
        check!(i32, AltInteger, vec![1, 2], 90);
        check!(f64, AltReal, vec![1.5, 2.5], 90.5);
        check!(u8, AltRaw, vec![1, 2], 255);
        check!(
            Rcomplex,
            AltComplex,
            vec![Rcomplex { r: 1.0, i: 2.0 }; 2],
            Rcomplex { r: 90.0, i: -1.0 }
        );
    });
}
