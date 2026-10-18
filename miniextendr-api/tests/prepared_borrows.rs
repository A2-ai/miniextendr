//! Native references are formed only after the selected conversion is checked.
#![cfg(feature = "either")]

mod r_test_utils;

use either::Either;
use miniextendr_api::from_r::prepared::{Prepared, assert_no_aliases};
use miniextendr_api::from_r::{NativeBorrow, SexpError};
use miniextendr_api::{IntoR, Missing, ProtectScope, Root, SEXP, TryFromSexp};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicUsize, Ordering};

static LEFT_CALLS: AtomicUsize = AtomicUsize::new(0);
static RIGHT_CALLS: AtomicUsize = AtomicUsize::new(0);
static MATERIALIZED: AtomicUsize = AtomicUsize::new(0);

struct Even(i32);
impl TryFromSexp for Even {
    type Error = SexpError;
    fn try_from_sexp(input: SEXP) -> Result<Self, Self::Error> {
        LEFT_CALLS.fetch_add(1, Ordering::Relaxed);
        let value = i32::try_from_sexp(input)?;
        if value % 2 == 0 {
            Ok(Self(value))
        } else {
            Err(SexpError::InvalidValue("must be even".into()))
        }
    }
}

struct Counted<'a>(&'a mut i32);
impl<'a> TryFromSexp for Counted<'a> {
    type Error = SexpError;
    const NATIVE_BORROW: Option<NativeBorrow> = <&mut i32>::NATIVE_BORROW;
    fn try_from_sexp(_: SEXP) -> Result<Self, Self::Error> {
        panic!("preparation must not construct a borrowed value");
    }
    fn __mx_prepare<'root>(root: Root<'root>) -> Result<Prepared<'root, Self>, Self::Error>
    where
        Self: 'root,
    {
        RIGHT_CALLS.fetch_add(1, Ordering::Relaxed);
        <&mut i32>::__mx_prepare(root).map(|value| unsafe {
            value.map(|reference| {
                MATERIALIZED.fetch_add(1, Ordering::Relaxed);
                Self(reference)
            })
        })
    }
}

#[test]
fn rejects_left_and_right_borrowing_before_materializing_either() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let root = scope.protect(7_i32.into_sexp());
        MATERIALIZED.store(0, Ordering::Relaxed);
        let shared = <&i32>::__mx_prepare(root).unwrap();
        let left = <Either<Counted<'_>, String>>::__mx_prepare(root).unwrap();
        let right = <Either<String, Counted<'_>>>::__mx_prepare(root).unwrap();
        for choice in [left.borrows(), right.borrows()] {
            assert!(
                catch_unwind(AssertUnwindSafe(|| {
                    assert_no_aliases(&[("choice", choice), ("shared", shared.borrows())]);
                }))
                .is_err()
            );
        }
        // Nothing was materialized; simply dropping the plans is safe.
        assert_eq!(MATERIALIZED.load(Ordering::Relaxed), 0);
        assert_eq!(i32::try_from_sexp(root.get()).unwrap(), 7);
    });
}

#[test]
fn owned_left_success_runs_once_and_never_prepares_the_borrowing_right() {
    r_test_utils::with_r_thread(|| unsafe {
        LEFT_CALLS.store(0, Ordering::Relaxed);
        RIGHT_CALLS.store(0, Ordering::Relaxed);
        MATERIALIZED.store(0, Ordering::Relaxed);
        let scope = ProtectScope::new();
        let root = scope.protect(8_i32.into_sexp());
        let choice = <Either<Even, Counted<'_>>>::__mx_prepare(root).unwrap();
        let mutable = <&mut i32>::__mx_prepare(root).unwrap();
        assert_no_aliases(&[("choice", choice.borrows()), ("mutable", mutable.borrows())]);
        assert_eq!(LEFT_CALLS.load(Ordering::Relaxed), 1);
        assert_eq!(RIGHT_CALLS.load(Ordering::Relaxed), 0);
        assert_eq!(MATERIALIZED.load(Ordering::Relaxed), 0);
        let Either::Left(Even(value)) = choice.finish() else {
            panic!("owned left")
        };
        *mutable.finish() += 1;
        assert_eq!(value, 8);
        assert_eq!(i32::try_from_sexp(root.get()).unwrap(), 9);
    });
}

#[test]
fn custom_left_failure_prepares_right_once_and_keeps_both_errors() {
    r_test_utils::with_r_thread(|| unsafe {
        LEFT_CALLS.store(0, Ordering::Relaxed);
        RIGHT_CALLS.store(0, Ordering::Relaxed);
        MATERIALIZED.store(0, Ordering::Relaxed);
        let scope = ProtectScope::new();
        let root = scope.protect(7_i32.into_sexp());
        let choice = <Either<Even, Counted<'_>>>::__mx_prepare(root).unwrap();
        assert_no_aliases(&[("choice", choice.borrows())]);
        let Either::Right(value) = choice.finish() else {
            panic!("borrowed right")
        };
        *value.0 += 2;
        assert_eq!(LEFT_CALLS.load(Ordering::Relaxed), 1);
        assert_eq!(RIGHT_CALLS.load(Ordering::Relaxed), 1);
        assert_eq!(MATERIALIZED.load(Ordering::Relaxed), 1);
        let wrong = scope.protect(1.5_f64.into_sexp());
        let error = match <Either<Even, String>>::__mx_prepare(wrong) {
            Err(error) => error,
            Ok(_) => panic!("both converters must refuse"),
        };
        assert!(matches!(error, SexpError::EitherConversion { .. }));
        assert!(error.to_string().contains("Left failed"));
        assert!(error.to_string().contains("Right failed"));
    });
}

#[test]
fn nested_eithers_and_absence_wrappers_keep_only_the_winning_ranges() {
    r_test_utils::with_r_thread(|| unsafe {
        type Choice<'a> = Missing<Result<Option<Either<String, Either<f64, &'a mut i32>>>, ()>>;
        let scope = ProtectScope::new();
        let root = scope.protect(4_i32.into_sexp());
        let prepared = Choice::__mx_prepare(root).unwrap();
        assert_eq!(prepared.borrows().len(), 1);
        assert_no_aliases(&[("choice", prepared.borrows())]);
        let Missing::Present(Ok(Some(Either::Right(Either::Right(value))))) = prepared.finish()
        else {
            panic!("nested right")
        };
        *value += 1;
        for input in [SEXP::missing_arg(), SEXP::nil()] {
            let absent = Choice::__mx_prepare(scope.protect(input)).unwrap();
            assert!(absent.borrows().is_empty());
        }
        let nullable =
            <Option<Either<String, &mut i32>>>::__mx_prepare(scope.protect(SEXP::nil())).unwrap();
        assert!(nullable.borrows().is_empty());
        assert!(nullable.finish().is_none());
        assert_eq!(i32::try_from_sexp(root.get()).unwrap(), 5);
    });
}

#[test]
fn ranges_distinguish_partial_overlap_shared_aliases_and_empty_buffers() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let root = scope.protect(vec![1_i32, 2, 3].into_sexp());
        let shared = <&[i32]>::__mx_prepare(root).unwrap();
        let also_shared = <&[i32]>::__mx_prepare(root).unwrap();
        assert_no_aliases(&[("a", shared.borrows()), ("b", also_shared.borrows())]);
        let pointer = <i32 as miniextendr_api::RNativeType>::dataptr_mut(root.get());
        // The second window starts inside the first, not at the same address.
        let first = Prepared::native(root, pointer, 2, true, move || {
            core::slice::from_raw_parts_mut(pointer, 2)
        });
        let second_pointer = pointer.add(1);
        let second = Prepared::native(root, second_pointer, 2, false, move || {
            core::slice::from_raw_parts(second_pointer, 2)
        });
        assert!(
            catch_unwind(AssertUnwindSafe(|| {
                assert_no_aliases(&[("first", first.borrows()), ("second", second.borrows())]);
            }))
            .is_err()
        );
        let empty = scope.protect(Vec::<i32>::new().into_sexp());
        let a = <&mut [i32]>::__mx_prepare(empty).unwrap();
        let b = <&mut [i32]>::__mx_prepare(empty).unwrap();
        assert_no_aliases(&[("a", a.borrows()), ("b", b.borrows())]);
        assert!(a.finish().is_empty());
        assert!(b.finish().is_empty());
    });
}

#[test]
fn borrowing_converters_without_preparation_stop_before_their_raw_converter() {
    struct Unprepared;
    impl TryFromSexp for Unprepared {
        type Error = SexpError;
        const NATIVE_BORROW: Option<NativeBorrow> = <&mut i32>::NATIVE_BORROW;
        fn try_from_sexp(_: SEXP) -> Result<Self, Self::Error> {
            panic!("raw converter must not be called");
        }
    }
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let root = scope.protect(1_i32.into_sexp());
        let error = catch_unwind(AssertUnwindSafe(|| Unprepared::__mx_prepare(root)));
        let payload = match error {
            Err(payload) => payload,
            Ok(_) => panic!("must refuse"),
        };
        assert!(
            payload
                .downcast_ref::<String>()
                .unwrap()
                .contains("must implement")
        );
    });
}

#[test]
fn length_failure_selects_the_slice_and_type_failure_keeps_conversion_errors() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let root = scope.protect(vec![1_i32, 2].into_sexp());
        let choice = <Either<&mut i32, &mut [i32]>>::__mx_prepare(root).unwrap();
        assert_no_aliases(&[("choice", choice.borrows())]);
        let Either::Right(slice) = choice.finish() else {
            panic!("scalar length failure must fall back to the slice")
        };
        slice[1] = 3;
        assert_eq!(Vec::<i32>::try_from_sexp(root.get()).unwrap(), [1, 3]);

        let wrong = scope.protect(1.5_f64.into_sexp());
        let prepared_error = match <Either<&mut i32, &mut [i32]>>::__mx_prepare(wrong) {
            Err(error) => error,
            Ok(_) => panic!("both native conversions must refuse the type"),
        };
        let raw_error = <Either<&mut i32, &mut [i32]>>::try_from_sexp(wrong.get()).unwrap_err();
        assert_eq!(prepared_error.to_string(), raw_error.to_string());
    });
}
