//! Built-in ring-buffer ALTREP classes retain their owner and R value semantics.
mod r_test_utils;

use std::collections::VecDeque;

use miniextendr_api::altrep_data::{AltrepExtract, AltrepSerialize};
use miniextendr_api::altrep_traits::{AltComplex, AltInteger, AltRaw, AltReal, AltVec, Altrep};
use miniextendr_api::{IntoR, OwnedProtect, Rcomplex, SexpExt, TryFromSexp};

#[test]
fn native_deque_classes_read_both_segments_and_serialize_pointer_writes() {
    r_test_utils::with_r_thread(|| {
        macro_rules! check {
            ($elem:ty, $family:ident, $values:expr, $replacement:expr) => {{
                let mut expected: Vec<$elem> = $values;
                let mut deque = VecDeque::from(expected.clone());
                deque.rotate_left(1);
                expected.rotate_left(1);
                assert!(!deque.as_slices().1.is_empty());
                let x = unsafe { OwnedProtect::new(miniextendr_api::Altrep(deque).into_sexp()) };
                assert!(x.get().is_altrep());
                assert!(<VecDeque<$elem> as AltVec>::dataptr_or_null(x.get()).is_null());
                let mut out = expected.clone();
                assert_eq!(
                    <VecDeque<$elem> as $family>::get_region(x.get(), 0, 99, &mut out),
                    isize::try_from(expected.len()).unwrap()
                );
                assert_eq!(out, expected);
                assert_eq!(<VecDeque<$elem> as $family>::elt(x.get(), 0), expected[0]);
                let pointer = <VecDeque<$elem> as AltVec>::dataptr(x.get(), true).cast::<$elem>();
                unsafe { *pointer = $replacement };
                expected[0] = $replacement;
                assert_eq!(<VecDeque<$elem> as $family>::elt(x.get(), 0), $replacement);
                let owner =
                    unsafe { <VecDeque<$elem> as AltrepExtract>::altrep_extract_ref(x.get()) };
                assert_eq!(owner.as_slices().0, expected.as_slice());
                assert!(owner.as_slices().1.is_empty());
                let state = unsafe {
                    OwnedProtect::new(<VecDeque<$elem> as Altrep>::serialized_state(x.get()))
                };
                let restored =
                    <VecDeque<$elem> as AltrepSerialize>::unserialize(state.get()).unwrap();
                drop(state);
                unsafe { miniextendr_api::sys::R_gc() };
                assert_eq!(restored.into_iter().collect::<Vec<_>>(), expected);
                assert_eq!(Vec::<$elem>::try_from_sexp(x.get()).unwrap(), expected);
            }};
        }
        check!(i32, AltInteger, vec![1, 2, 3, 4], 90);
        check!(f64, AltReal, vec![1.5, 2.5, 3.5, 4.5], 90.5);
        check!(u8, AltRaw, vec![1, 2, 3, 4], 255);
        check!(
            Rcomplex,
            AltComplex,
            vec![Rcomplex { r: 1.0, i: 2.0 }; 4],
            Rcomplex { r: 90.0, i: -1.0 }
        );
    });
}
