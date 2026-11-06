//! An `Either` argument's alias check follows the arm its conversion takes
//! (#1504).
//!
//! `wrapper_call` runs the generated wrapper's protocol
//! (`rust_conversion_builder::two_pass_alias_guard` in miniextendr-macros) on
//! two arguments: the check with both arms of the `Either`, the preparation
//! of the marked argument, the check with the arm it takes, then the
//! conversion from the prepared state. rpkg's `test-either-borrow.R` drives
//! the generated wrappers themselves.
#![cfg(feature = "either")]

mod r_test_utils;

use std::cell::Cell;

use miniextendr_api::condition::RCondition;
use miniextendr_api::either_impl::Either;
use miniextendr_api::from_r::borrow::{assert_no_aliases, assert_or_select, needs_check};
use miniextendr_api::from_r::prepared::{self, Prepared};
use miniextendr_api::from_r::{NativeBorrow, SexpError, TryFromSexp};
use miniextendr_api::gc_protect::ProtectScope;
use miniextendr_api::list::ListBuilder;
use miniextendr_api::unwind_protect::with_r_unwind_protect;
use miniextendr_api::{IntoR, Missing, SEXP};

/// The wrapper's check of `a: T` next to `b` (which borrows `b_borrow`),
/// then `a`'s conversion: the alias error, or `a`'s value.
fn wrapper_call<T: TryFromSexp + 'static>(
    a: SEXP,
    b: SEXP,
    b_borrow: Option<NativeBorrow>,
) -> Result<Result<T, T::Error>, String> {
    let mut converted = None;
    let result = with_r_unwind_protect(
        || {
            let mut prepared = Prepared::<T>::Unprepared;
            let mut borrows = [T::NATIVE_BORROW, b_borrow];
            if needs_check(&borrows) {
                let args = [(a, "a"), (b, "b")];
                let select = unsafe { assert_or_select(args, borrows) };
                if select.contains(&true) {
                    if NativeBorrow::selects(T::NATIVE_BORROW) && select[0] {
                        prepared = T::__mx_prepare(a);
                        borrows[0] = prepared.borrow(borrows[0]);
                    }
                    unsafe { assert_no_aliases(args, borrows) };
                }
            }
            converted = Some(prepared::convert(prepared, a));
            SEXP::nil()
        },
        None,
    );
    let condition = unsafe {
        let scope = ProtectScope::new();
        let result = scope.protect(result);
        RCondition::from_tagged_sexp(result.get()).map(|c| c.message().to_owned())
    };
    match condition {
        Some(message) => Err(message),
        None => Ok(converted.expect("the conversion ran")),
    }
}

#[track_caller]
fn assert_alias(result: Result<impl Sized, String>) {
    let Err(message) = result else {
        panic!("expected an alias error");
    };
    assert!(
        message.contains("parameters `a` and `b` borrow the same non-empty R object"),
        "{message}"
    );
}

fn list(scope: &ProtectScope, elements: &[SEXP]) -> SEXP {
    let builder = unsafe { ListBuilder::new(scope, elements.len()) };
    for (index, &element) in elements.iter().enumerate() {
        unsafe { builder.set(isize::try_from(index).unwrap(), element) };
    }
    builder.into_sexp()
}

fn shared_i32() -> Option<NativeBorrow> {
    <&i32>::NATIVE_BORROW
}

#[test]
fn the_issue_cases() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let v = scope.protect(7i32.into_sexp()).get();
        // Left converts and borrows mutably what `b` borrows.
        assert_alias(wrapper_call::<Either<&mut i32, String>>(v, v, shared_i32()));
        // Left fails, Right converts and borrows it.
        assert_alias(wrapper_call::<Either<String, &mut i32>>(v, v, shared_i32()));
        // Left copies the scalar: the unused Right arm borrows nothing.
        match wrapper_call::<Either<i32, &mut i32>>(v, v, <&mut i32>::NATIVE_BORROW) {
            Ok(Ok(Either::Left(7))) => {}
            Ok(_) => panic!("expected Left(7)"),
            Err(message) => panic!("rejected an owned Left: {message}"),
        }
    });
}

#[test]
fn distinct_objects_and_shared_arms_are_accepted() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let v = scope.protect(7i32.into_sexp()).get();
        let w = scope.protect(8i32.into_sexp()).get();
        match wrapper_call::<Either<String, &mut i32>>(v, w, shared_i32()) {
            Ok(Ok(Either::Right(r))) => assert_eq!(*r, 7),
            _ => panic!("distinct objects convert"),
        }
        // A shared arm next to a shared argument never conflicts.
        assert!(matches!(
            wrapper_call::<Either<&i32, String>>(v, v, shared_i32()),
            Ok(Ok(Either::Left(7)))
        ));
    });
}

#[test]
fn slices_follow_the_selected_arm() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let x = scope.protect(vec![1.0f64, 2.0, 3.0].into_sexp()).get();
        let n = scope.protect(1i32.into_sexp()).get();
        let shared = <&[f64]>::NATIVE_BORROW;
        assert_alias(wrapper_call::<Either<i32, &mut [f64]>>(x, x, shared));
        assert!(matches!(
            wrapper_call::<Either<i32, &mut [f64]>>(n, x, shared),
            Ok(Ok(Either::Left(1)))
        ));
        // An owned Left on the same vector is fine.
        assert!(matches!(
            wrapper_call::<Either<Vec<f64>, &mut [f64]>>(x, x, shared),
            Ok(Ok(Either::Left(_)))
        ));
        // An empty vector borrows nothing.
        let empty = scope.protect(Vec::<f64>::new().into_sexp()).get();
        assert!(matches!(
            wrapper_call::<Either<String, &mut [f64]>>(empty, empty, shared),
            Ok(Ok(Either::Right(_)))
        ));
    });
}

#[test]
fn nested_and_wrapped_eithers_keep_their_arm() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let v = scope.protect(7i32.into_sexp()).get();
        let d = scope.protect(2.5f64.into_sexp()).get();
        let e = scope.protect(3.5f64.into_sexp()).get();
        type Nested = Either<String, Either<i32, &'static mut f64>>;
        // String fails, the inner i32 converts: owned.
        assert!(matches!(
            wrapper_call::<Either<String, Either<i32, &mut i32>>>(v, v, shared_i32()),
            Ok(Ok(Either::Right(Either::Left(7))))
        ));
        // Right, Right borrows the double.
        assert_alias(wrapper_call::<Nested>(d, d, <&f64>::NATIVE_BORROW));
        // Distinct doubles: the path through both arms builds the reference.
        match wrapper_call::<Nested>(d, e, <&f64>::NATIVE_BORROW) {
            Ok(Ok(Either::Right(Either::Right(r)))) => assert_eq!(*r, 2.5),
            _ => panic!("the nested arm materializes"),
        }
        // `Option`, `Missing` and `Result<_, ()>` prepare as the `Either`.
        assert_alias(wrapper_call::<Option<Either<String, &mut i32>>>(
            v,
            v,
            shared_i32(),
        ));
        assert!(matches!(
            wrapper_call::<Option<Either<i32, &mut i32>>>(v, v, <&mut i32>::NATIVE_BORROW),
            Ok(Ok(Some(Either::Left(7))))
        ));
        assert_alias(wrapper_call::<Missing<Either<String, &mut i32>>>(
            v,
            v,
            shared_i32(),
        ));
        assert!(matches!(
            wrapper_call::<Missing<Either<i32, &mut i32>>>(v, v, <&mut i32>::NATIVE_BORROW),
            Ok(Ok(Missing::Present(Either::Left(7))))
        ));
        assert_alias(wrapper_call::<Result<Either<String, &mut i32>, ()>>(
            v,
            v,
            shared_i32(),
        ));
        assert!(matches!(
            wrapper_call::<Result<Either<i32, &mut i32>, ()>>(v, v, <&mut i32>::NATIVE_BORROW),
            Ok(Ok(Ok(Either::Left(7))))
        ));
    });
}

#[test]
fn absent_values_prepare_without_an_arm() {
    r_test_utils::with_r_thread(|| {
        assert!(matches!(
            <Option<Either<String, &mut i32>>>::__mx_prepare(SEXP::nil()),
            Prepared::Ready(None)
        ));
        assert!(matches!(
            <Missing<Either<String, &mut i32>>>::__mx_prepare(SEXP::missing_arg()),
            Prepared::Ready(Missing::Absent)
        ));
        assert!(matches!(
            <Result<Either<String, &mut i32>, ()>>::__mx_prepare(SEXP::nil()),
            Prepared::Ready(Err(()))
        ));
    });
}

#[test]
fn both_arms_failing_keeps_the_conversion_error() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let s = scope.protect("x".into_sexp()).get();
        let two = scope.protect(vec![1.0f64, 2.0].into_sexp()).get();
        for input in [s, two] {
            let expected = match <Either<i32, &mut f64>>::try_from_sexp(input) {
                Err(e) => e.to_string(),
                Ok(_) => panic!("neither arm converts"),
            };
            match <Either<i32, &mut f64>>::__mx_prepare(input) {
                Prepared::Failed(e) => assert_eq!(e.to_string(), expected),
                _ => panic!("a failed preparation keeps the error"),
            }
        }
    });
}

/// An owned type whose conversion accepts only positive integers: the
/// `Either`'s arm depends on the value, not on its R type.
struct Positive(i32);

impl TryFromSexp for Positive {
    type Error = SexpError;
    fn try_from_sexp(sexp: SEXP) -> Result<Self, SexpError> {
        let value = i32::try_from_sexp(sexp)?;
        if value > 0 {
            Ok(Positive(value))
        } else {
            Err(SexpError::InvalidValue("not positive".into()))
        }
    }
}

#[test]
fn a_custom_left_predicate_decides_the_arm() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let positive = scope.protect(5i32.into_sexp()).get();
        let negative = scope.protect((-5i32).into_sexp()).get();
        match wrapper_call::<Either<Positive, &mut i32>>(positive, positive, shared_i32()) {
            Ok(Ok(Either::Left(Positive(5)))) => {}
            _ => panic!("the positive Left is owned"),
        }
        assert_alias(wrapper_call::<Either<Positive, &mut i32>>(
            negative,
            negative,
            shared_i32(),
        ));
    });
}

thread_local! {
    static FLIPS: Cell<u32> = const { Cell::new(0) };
}

/// An owned conversion that succeeds on odd calls and fails on even ones.
struct Flip;

impl TryFromSexp for Flip {
    type Error = SexpError;
    fn try_from_sexp(_: SEXP) -> Result<Self, SexpError> {
        let calls = FLIPS.get() + 1;
        FLIPS.set(calls);
        if calls % 2 == 1 {
            Ok(Flip)
        } else {
            Err(SexpError::InvalidValue("flipped".into()))
        }
    }
}

#[test]
fn the_checked_arm_is_the_converted_arm() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let v = scope.protect(7i32.into_sexp()).get();
        // Call 1 succeeds: Left, owned, converted once and kept.
        FLIPS.set(0);
        assert!(matches!(
            wrapper_call::<Either<Flip, &mut i32>>(v, v, shared_i32()),
            Ok(Ok(Either::Left(Flip)))
        ));
        assert_eq!(FLIPS.get(), 1, "the Left arm converts once");
        // Call 2 fails: the Right arm borrows and is refused. A second Left
        // conversion that would succeed never runs, so an accepted Right
        // with an alias can't be observed.
        assert_alias(wrapper_call::<Either<Flip, &mut i32>>(v, v, shared_i32()));
        assert_eq!(FLIPS.get(), 2);
        // Distinct objects: call 3 succeeds, so Left; call 4 would fail.
        let w = scope.protect(8i32.into_sexp()).get();
        assert!(matches!(
            wrapper_call::<Either<Flip, &mut i32>>(v, w, shared_i32()),
            Ok(Ok(Either::Left(Flip)))
        ));
        assert_eq!(FLIPS.get(), 3);
        // Without a conflict nothing is prepared: one conversion as before.
        let d = scope.protect(1.5f64.into_sexp()).get();
        let _ = wrapper_call::<Either<Flip, &mut i32>>(v, d, <&f64>::NATIVE_BORROW);
        assert_eq!(FLIPS.get(), 4);
    });
}

#[test]
fn an_undecided_arm_is_checked_against_both_and_says_so() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let v = scope.protect(7i32.into_sexp()).get();
        let rows = list(&scope, &[v]);
        // A list of references has no preparation of its own.
        let Err(message) = wrapper_call::<Either<Vec<&mut i32>, String>>(rows, v, shared_i32())
        else {
            panic!("expected an alias error");
        };
        assert!(message.contains("checked against both arms"), "{message}");
        // A plain conflict carries no such hint.
        let Err(message) = wrapper_call::<&mut i32>(v, v, shared_i32()) else {
            panic!("expected an alias error");
        };
        assert!(!message.contains("both arms"), "{message}");
    });
}

thread_local! {
    static SLOT_CHECKS: Cell<u32> = const { Cell::new(0) };
}

/// A mutable scalar behind a check in user code that fails on odd calls and
/// passes on even ones, as a `#[try_from_sexp(validate = ...)]` newtype's
/// may: its preparation forwards the reference's, as the derive does.
struct FlakySlot(&'static mut i32);

fn flaky_check() -> Result<(), SexpError> {
    let calls = SLOT_CHECKS.get() + 1;
    SLOT_CHECKS.set(calls);
    if calls % 2 == 1 {
        Err(SexpError::InvalidValue("flaky".into()))
    } else {
        Ok(())
    }
}

impl TryFromSexp for FlakySlot {
    type Error = SexpError;
    const NATIVE_BORROW: Option<NativeBorrow> = <&'static mut i32>::NATIVE_BORROW;
    fn try_from_sexp(sexp: SEXP) -> Result<Self, SexpError> {
        flaky_check()?;
        <&'static mut i32>::try_from_sexp(sexp).map(FlakySlot)
    }
    fn __mx_prepare(sexp: SEXP) -> Prepared<Self> {
        if let Err(error) = flaky_check() {
            return Prepared::Failed(error);
        }
        <&'static mut i32>::__mx_prepare(sexp).map(FlakySlot, |e| e)
    }
    fn __mx_materialize(sexp: SEXP, arms: prepared::Arms) -> Result<Self, SexpError> {
        <&'static mut i32>::__mx_materialize(sexp, arms).map(FlakySlot)
    }
}

#[test]
fn a_failed_left_arm_stays_checked_when_the_right_arm_is_undecided() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let v = scope.protect(7i32.into_sexp()).get();
        // The preparation's Left check fails (call 1) and the list arm has no
        // preparation, so the conversion runs the whole `Either` after the
        // check, Left first, and its check passes then (call 2). The check
        // must still take the Left arm, or `a` would borrow `b` mutably.
        SLOT_CHECKS.set(0);
        let Err(message) = wrapper_call::<Either<FlakySlot, Vec<&mut i32>>>(v, v, shared_i32())
        else {
            panic!("expected an alias error");
        };
        assert!(message.contains("checked against both arms"), "{message}");
        assert_eq!(SLOT_CHECKS.get(), 1, "refused before converting");
        // Distinct objects: no conflict, so no preparation; the conversion's
        // check passes (call 2) and Left borrows `a`.
        let w = scope.protect(8i32.into_sexp()).get();
        let Ok(Ok(Either::Left(slot))) =
            wrapper_call::<Either<FlakySlot, Vec<&mut i32>>>(v, w, shared_i32())
        else {
            panic!("expected the Left arm");
        };
        assert_eq!(*slot.0, 7);
        assert_eq!(SLOT_CHECKS.get(), 2);
    });
}
