//! An `Either` argument's native-borrow check follows the arm its conversion
//! takes (#1504). Driven by `tests/testthat/test-either-borrow.R`.
//!
//! Each fixture returns only after its conversions succeeded, so a refused
//! call never builds a reference. Where an arm borrows mutably the body
//! writes through it, so the tests can see which arm ran.

use std::cell::Cell;

use miniextendr_api::either_impl::Either;
use miniextendr_api::from_r::SexpError;
use miniextendr_api::{ExternalPtr, IntoR, Missing, SEXP, TryFromSexp, defer_warning, miniextendr};

fn len_i32(n: usize) -> i32 {
    i32::try_from(n).unwrap_or(i32::MAX)
}

// region: the issue's cases

/// A mutable Left arm next to a shared scalar: refused when both are one
/// vector, else the Left arm adds `b` in place.
/// @param a An integer scalar, or a string.
/// @param b An integer scalar.
/// @noRd
#[miniextendr(noexport)]
pub fn either_borrow_mut_left(a: Either<&mut i32, String>, b: &i32) -> i32 {
    match a {
        Either::Left(a) => {
            *a += *b;
            *a
        }
        Either::Right(s) => len_i32(s.len()),
    }
}

/// A mutable Right arm next to a shared scalar.
/// @param a A string, or an integer scalar.
/// @param b An integer scalar.
/// @noRd
#[miniextendr(noexport)]
pub fn either_borrow_mut_right(a: Either<String, &mut i32>, b: &i32) -> i32 {
    match a {
        Either::Left(s) => len_i32(s.len()),
        Either::Right(a) => {
            *a += *b;
            *a
        }
    }
}

/// An owned Left arm whose unused Right arm would borrow mutably, next to a
/// mutable scalar: one vector for both is valid, the Left arm is a copy.
/// @param a An integer scalar.
/// @param b An integer scalar, incremented in place.
/// @noRd
#[miniextendr(noexport)]
pub fn either_borrow_owned_left(a: Either<i32, &mut i32>, b: &mut i32) -> i32 {
    *b += 1;
    match a {
        Either::Left(n) => n,
        Either::Right(r) => {
            *r += 100;
            *r
        }
    }
}
// endregion

// region: slices, nesting and wrappers

/// An owned scalar or a mutable slice, next to a shared slice. The Right
/// arm adds the sum of `b` to every element.
/// @param a An integer scalar, or a double vector.
/// @param b A double vector.
/// @noRd
#[miniextendr(noexport)]
pub fn either_borrow_slice(a: Either<i32, &mut [f64]>, b: &[f64]) -> f64 {
    let total: f64 = b.iter().sum();
    match a {
        Either::Left(n) => f64::from(n) + total,
        Either::Right(v) => {
            for x in v.iter_mut() {
                *x += total;
            }
            v.iter().sum()
        }
    }
}

/// A nested `Either`: a string, an integer, or a mutable double.
/// @param a A string, an integer scalar or a double scalar.
/// @param b A double scalar.
/// @noRd
#[miniextendr(noexport)]
pub fn either_borrow_nested(a: Either<String, Either<i32, &mut f64>>, b: &f64) -> f64 {
    match a {
        Either::Left(s) => f64::from(len_i32(s.len())) + *b,
        Either::Right(Either::Left(n)) => f64::from(n) + *b,
        Either::Right(Either::Right(r)) => {
            *r += *b;
            *r
        }
    }
}

/// An optional `Either`: `NULL` borrows nothing.
/// @param a `NULL`, a string, or an integer scalar.
/// @param b An integer scalar.
/// @noRd
#[miniextendr(noexport)]
pub fn either_borrow_option(a: Option<Either<String, &mut i32>>, b: &i32) -> i32 {
    match a {
        None => -1,
        Some(Either::Left(s)) => len_i32(s.len()),
        Some(Either::Right(a)) => {
            *a += *b;
            *a
        }
    }
}

/// An omittable `Either`.
/// @param a A string, or an integer scalar; may be omitted.
/// @param b An integer scalar.
/// @noRd
#[miniextendr(noexport)]
pub fn either_borrow_missing(a: Missing<Either<String, &mut i32>>, b: &i32) -> i32 {
    match a {
        Missing::Absent => -1,
        Missing::Present(Either::Left(s)) => len_i32(s.len()),
        Missing::Present(Either::Right(a)) => {
            *a += *b;
            *a
        }
    }
}

/// An `Either` behind a derived newtype: the derive forwards the
/// preparation, so the owned Left arm is known.
#[derive(miniextendr_api::TryFromSexp)]
pub struct EitherSlot(pub Either<i32, &'static mut i32>);

/// A derived newtype of an `Either` whose Right arm borrows.
#[derive(miniextendr_api::TryFromSexp)]
pub struct EitherSlotRight(pub Either<String, &'static mut i32>);

/// A newtype `Either` with an owned Left arm next to a mutable scalar.
/// @param a An integer scalar.
/// @param b An integer scalar, incremented in place.
/// @noRd
#[miniextendr(noexport)]
pub fn either_borrow_newtype_owned(a: EitherSlot, b: &mut i32) -> i32 {
    *b += 1;
    match a.0 {
        Either::Left(n) => n,
        Either::Right(r) => *r,
    }
}

/// A newtype `Either` whose Right arm borrows, next to a shared scalar.
/// @param a A string, or an integer scalar.
/// @param b An integer scalar.
/// @noRd
#[miniextendr(noexport)]
pub fn either_borrow_newtype_right(a: EitherSlotRight, b: &i32) -> i32 {
    match a.0 {
        Either::Left(s) => len_i32(s.len()),
        Either::Right(a) => {
            *a += *b;
            *a
        }
    }
}

/// A small integer (below 100), by mutable reference: the check runs before
/// the reference exists.
fn small_int(x: SEXP) -> Result<(), SexpError> {
    match i32::try_from_sexp(x) {
        Ok(n) if n < 100 => Ok(()),
        Ok(n) => Err(SexpError::InvalidValue(format!("{n} is not below 100"))),
        Err(e) => Err(e),
    }
}

/// A validated newtype of a mutable scalar.
#[derive(miniextendr_api::TryFromSexp)]
#[try_from_sexp(validate = small_int)]
pub struct SmallSlot(pub &'static mut i32);

/// A conversion error before an `Either` whose arms both fail is still the
/// first argument's: preparing the `Either` early keeps its error for its
/// own place. `first` is a `Positive`, which has no R-side check, so its
/// refusal comes from the C wrapper too.
/// @param first A positive integer scalar.
/// @param a A string, or an integer scalar below 100.
/// @param b An integer scalar.
/// @noRd
#[miniextendr(noexport)]
pub fn either_borrow_order(first: Positive, a: Either<String, SmallSlot>, b: &i32) -> i32 {
    match a {
        Either::Left(s) => first.0 + len_i32(s.len()),
        Either::Right(slot) => {
            *slot.0 += *b;
            first.0 + *slot.0
        }
    }
}
// endregion

// region: arms decided by the conversion

/// A positive integer: owned, its arm depends on the value.
pub struct Positive(pub i32);

impl IntoR for Positive {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.0.into_sexp())
    }
}

impl TryFromSexp for Positive {
    type Error = SexpError;
    fn try_from_sexp(sexp: SEXP) -> Result<Self, SexpError> {
        let value = i32::try_from_sexp(sexp)?;
        if value > 0 {
            Ok(Positive(value))
        } else {
            Err(SexpError::InvalidValue(format!("{value} is not positive")))
        }
    }
}

/// A positive integer, or a mutable integer scalar.
/// @param a An integer scalar.
/// @param b An integer scalar.
/// @noRd
#[miniextendr(noexport)]
pub fn either_borrow_positive(a: Either<Positive, &mut i32>, b: &i32) -> i32 {
    match a {
        Either::Left(Positive(n)) => n,
        Either::Right(r) => {
            *r += *b;
            *r
        }
    }
}

thread_local! {
    static FLIPS: Cell<i32> = const { Cell::new(0) };
}

/// A conversion that succeeds on odd calls and fails on even ones.
pub struct Flip;

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

/// `Flip`, or a mutable integer scalar: the arm the check takes is the arm
/// the conversion took.
/// @param a An integer scalar.
/// @param b An integer scalar.
/// @noRd
#[miniextendr(noexport)]
pub fn either_borrow_flip(a: Either<Flip, &mut i32>, b: &i32) -> i32 {
    match a {
        Either::Left(Flip) => 0,
        Either::Right(r) => {
            *r += *b;
            *r
        }
    }
}

/// Reset the `Flip` call count.
/// @noRd
#[miniextendr(noexport)]
pub fn either_borrow_flip_reset() {
    FLIPS.set(0);
}

/// The `Flip` call count.
/// @noRd
#[miniextendr(noexport)]
pub fn either_borrow_flip_calls() -> i32 {
    FLIPS.get()
}

/// An integer whose conversion queues one warning.
pub struct Noisy(pub i32);

impl TryFromSexp for Noisy {
    type Error = SexpError;
    fn try_from_sexp(sexp: SEXP) -> Result<Self, SexpError> {
        defer_warning!("converting a Noisy");
        i32::try_from_sexp(sexp).map(Noisy)
    }
}

/// `Noisy`, or a mutable integer scalar: its warning is raised once, though
/// the check prepares the argument before converting it.
/// @param a An integer scalar.
/// @param b An integer scalar.
/// @noRd
#[miniextendr(noexport)]
pub fn either_borrow_noisy(a: Either<Noisy, &mut i32>, b: &i32) -> i32 {
    match a {
        Either::Left(Noisy(n)) => n + *b,
        Either::Right(r) => *r,
    }
}
// endregion

// region: choices and containers

/// A choice or a mutable double vector: character input is the owned
/// choice, other input borrows.
/// @param level `"low"`, `"high"`, or a double vector.
/// @param b A double vector.
/// @noRd
#[miniextendr(noexport)]
pub fn either_borrow_choice(
    #[miniextendr(choices("low", "high"))] level: Either<String, &mut [f64]>,
    b: &[f64],
) -> f64 {
    let total: f64 = b.iter().sum();
    match level {
        Either::Left(level) => f64::from(len_i32(level.len())) + total,
        Either::Right(v) => {
            for x in v.iter_mut() {
                *x += total;
            }
            v.iter().sum()
        }
    }
}

/// An `Either` inside a tuple is checked against both arms: its arm is not
/// learned there.
/// @param a A list of an integer scalar and a string.
/// @param b An integer scalar.
/// @noRd
#[miniextendr(noexport)]
pub fn either_borrow_in_tuple(a: (Either<i32, &mut i32>, String), b: &i32) -> i32 {
    match a.0 {
        Either::Left(n) => n + *b,
        Either::Right(r) => *r,
    }
}
// endregion

// region: trait shims

/// A mutable integer scalar the trait ABI can pass: a View copies it into a
/// fresh vector.
#[derive(miniextendr_api::TryFromSexp)]
pub struct ScalarSlot(pub &'static mut i32);

impl IntoR for ScalarSlot {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok((*self.0).into_sexp())
    }
}

/// A shared integer scalar the trait ABI can pass.
#[derive(miniextendr_api::TryFromSexp)]
pub struct ScalarRead(pub &'static i32);

impl IntoR for ScalarRead {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok((*self.0).into_sexp())
    }
}

/// A trait method taking an `Either` whose Right arm borrows.
#[miniextendr]
pub trait EitherBorrowPick {
    /// A positive `a`, or `a` after adding `b` to it in place.
    fn pick(&self, a: Either<Positive, ScalarSlot>, b: ScalarRead) -> i32;

    /// `pick` as a default the impl leaves out: its wrapper converts through
    /// the types inferred from the copied signature (#2123).
    /// @param a An integer scalar.
    /// @param b An integer scalar.
    fn pick_default(&self, a: Either<Positive, ScalarSlot>, b: ScalarRead) -> i32 {
        pick(a, &b)
    }
}

/// The same methods on a generic trait, whose impls get concrete shims.
#[miniextendr]
pub trait EitherBorrowPickAs<T> {
    /// A positive `a`, or `a` after adding `b` to it in place.
    fn pick_as(&self, a: Either<Positive, T>, b: ScalarRead) -> i32;

    /// `a` when positive, else `b`, as a default the impl leaves out: its
    /// wrapper and concrete shim convert through the types inferred from
    /// the copied signature (#2123).
    /// @param a An integer scalar.
    /// @param b An integer scalar.
    fn pick_as_default(&self, a: Either<Positive, T>, b: ScalarRead) -> i32 {
        match a {
            Either::Left(Positive(n)) => n,
            Either::Right(_) => *b.0,
        }
    }
}

fn pick(a: Either<Positive, ScalarSlot>, b: &ScalarRead) -> i32 {
    match a {
        Either::Left(Positive(n)) => n,
        Either::Right(slot) => {
            *slot.0 += *b.0;
            *slot.0
        }
    }
}

/// Host for the `Either` trait methods.
#[derive(ExternalPtr)]
pub struct EitherBorrowHost;

#[miniextendr(env, noexport)]
impl EitherBorrowHost {
    pub fn new() -> Self {
        Self
    }
}

#[miniextendr(env, noexport)]
impl EitherBorrowPick for EitherBorrowHost {
    /// @param a An integer scalar.
    /// @param b An integer scalar.
    fn pick(&self, a: Either<Positive, ScalarSlot>, b: ScalarRead) -> i32 {
        pick(a, &b)
    }
}

#[miniextendr(env, noexport)]
impl EitherBorrowPickAs<ScalarSlot> for EitherBorrowHost {
    /// @param a An integer scalar.
    /// @param b An integer scalar.
    fn pick_as(&self, a: Either<Positive, ScalarSlot>, b: ScalarRead) -> i32 {
        pick(a, &b)
    }
}

/// Call a trait method's vtable shim with `a` and `b` as its arguments, as
/// another package's View would but without copying them, so the shim's
/// own alias check sees one vector passed twice. `generic` picks the
/// generic trait's concrete shim, `default` the default method's.
/// @param a An integer scalar.
/// @param b An integer scalar.
/// @param generic Whether to call the generic trait's shim.
/// @param default Whether to call the default method's shim.
/// @noRd
#[miniextendr(noexport)]
pub fn either_borrow_shim(a: SEXP, b: SEXP, generic: bool, default: bool) -> SEXP {
    let cell = std::cell::RefCell::new(EitherBorrowHost);
    let data = std::ptr::from_ref(&cell).cast_mut().cast();
    let argv = [a, b];
    let generic_table = &__VTABLE_miniextendr_EitherBorrowPickAs_FOR_EitherBorrowHost;
    let shim = match (generic, default) {
        (true, false) => generic_table.pick_as,
        (true, true) => generic_table.pick_as_default,
        (false, false) => <EitherBorrowHost as EitherBorrowPick>::__EitherBorrowPick_pick_shim,
        (false, true) => {
            <EitherBorrowHost as EitherBorrowPick>::__EitherBorrowPick_pick_default_shim
        }
    };
    let result = unsafe { shim(data, 2, argv.as_ptr()) };
    unsafe { miniextendr_api::trait_abi::repanic_if_rust_error(result) };
    result
}
// endregion

// region: gctorture

/// Drive the two-pass check through generated wrappers with fresh inputs:
/// an owned Left kept from its preparation, a Right arm built after the
/// second pass, and a refusal.
///
/// No arguments — picked up by the fast `gctorture(TRUE)` no-arg sweep (#430).
#[miniextendr(noexport)]
pub fn gc_stress_either_borrow() -> i32 {
    use miniextendr_api::condition::RCondition;
    use miniextendr_api::gc_protect::ProtectScope;

    unsafe {
        let scope = ProtectScope::new();
        let call = SEXP::nil();
        let x = scope.protect(7i32.into_sexp()).get();
        let owned = scope
            .protect(C_miniextendr_either_borrow_owned_left(call, x, x))
            .get();
        assert_eq!(i32::try_from_sexp(owned).ok(), Some(7));

        let v = scope.protect(vec![1.0f64, 2.0].into_sexp()).get();
        let y = scope.protect(vec![1.0f64, 1.0].into_sexp()).get();
        let right = scope
            .protect(C_miniextendr_either_borrow_slice(call, y, v))
            .get();
        assert_eq!(f64::try_from_sexp(right).ok(), Some(8.0));

        let refused = scope
            .protect(C_miniextendr_either_borrow_slice(call, v, v))
            .get();
        assert!(RCondition::from_tagged_sexp(refused).is_some());

        let label = scope.protect("a".into_sexp()).get();
        let d = scope.protect(2.5f64.into_sexp()).get();
        let nested = scope
            .protect(C_miniextendr_either_borrow_nested(call, label, d))
            .get();
        assert_eq!(f64::try_from_sexp(nested).ok(), Some(3.5));
        3
    }
}
// endregion
