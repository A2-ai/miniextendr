//! [`NonNa<T>`]: a value that holds no R `NA`.
//!
//! As a `#[miniextendr]` parameter type, `NonNa<T>` is `#[miniextendr(no_na)]`
//! written in the type: the parameter converts as `T`, an argument holding an
//! `NA` is refused with `no_na`'s argument error, and the body receives a
//! value that says so in its type.
//!
//! ```ignore
//! use miniextendr_api::{NonNa, miniextendr};
//!
//! /// `xs` holds no `NA` (and no `NaN`); `sum()` needs no check of its own.
//! #[miniextendr]
//! pub fn total(xs: NonNa<Vec<f64>>) -> f64 {
//!     xs.iter().sum()
//! }
//!
//! /// A string argument: `NA_character_` is refused instead of reaching the
//! /// body as `""`, as it would for a plain `String`.
//! #[miniextendr]
//! pub fn greet(name: NonNa<String>) -> String {
//!     format!("hello {}", name.unwrap())
//! }
//! ```
//!
//! "`NA`" is what R's `anyNA()` counts: `NA` in every type, and `NaN` in
//! doubles and complex numbers. The generated R wrapper checks it first
//! (`!anyNA(x)`, the `no_na` guard), and the C wrapper checks the argument
//! again before converting it, so a `.Call()` that skips the R wrapper cannot
//! hand the body an `NA` either. A reading marker (`AsNumeric`, ...) is also
//! asked after its conversion, as under `no_na` (`"NA"` strings, blank
//! cells). The custom message of `no_na(message = "...")` applies when both
//! are written.
//!
//! The marker composes with the precondition markers
//! (`Unchecked<NonNa<Vec<f64>>>` drops the type checks and keeps this one).
//! Like them it goes outermost (`NonNa<Option<T>>` is refused, since a value
//! without `NA` makes `Option` mean only `NULL`: write `Option<T>` with
//! `#[miniextendr(no_na)]` for that), it is a parameter type only, and it
//! cannot appear in a `#[miniextendr]` trait's method signature (spell
//! `no_na(x)` on the impl) or on an `extern "C-unwind"` function.
//!
//! # Construction
//!
//! The generated wrapper builds a `NonNa` after its check. Rust code builds
//! one with [`NonNa::new`], which checks the value ([`MaybeNa`]) and returns
//! `None` when it holds an `NA`, or with [`NonNa::new_unchecked`], whose
//! caller promises there is none. There is no other way in: the field is
//! private, and there is no `DerefMut` / `AsMut`, which would let code write
//! an `NA` into it.

use std::borrow::Borrow;
use std::ops::Deref;

use crate::sexp_types::{IsNA, RLogical, Rcomplex};

/// A value that holds no R `NA`: a `#[miniextendr]` parameter refuses an
/// argument with one (see the [module docs](self)). Transparent over `T`.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NonNa<T>(T);

impl<T: MaybeNa> NonNa<T> {
    /// `value` as a `NonNa`, or `None` when it holds an `NA`
    /// ([`MaybeNa::has_na`]).
    #[inline]
    pub fn new(value: T) -> Option<Self> {
        if value.has_na() {
            None
        } else {
            Some(NonNa(value))
        }
    }
}

impl<T> NonNa<T> {
    /// `value` as a `NonNa`, without checking it.
    ///
    /// # Safety
    ///
    /// `value` must hold no `NA` in the sense of `anyNA()`: code receiving a
    /// `NonNa` relies on that.
    #[inline]
    pub const unsafe fn new_unchecked(value: T) -> Self {
        NonNa(value)
    }

    /// The value.
    #[inline]
    pub fn into_inner(self) -> T {
        self.0
    }

    /// The value; the same as [`into_inner`](Self::into_inner). It never
    /// panics: a `NonNa` always holds one.
    #[inline]
    pub fn unwrap(self) -> T {
        self.0
    }
}

impl<T: Copy> NonNa<T> {
    /// A copy of the value, as `NonZero::get` gives one.
    #[inline]
    pub fn get(self) -> T {
        self.0
    }
}

impl<T> Deref for NonNa<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> AsRef<T> for NonNa<T> {
    #[inline]
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> Borrow<T> for NonNa<T> {
    #[inline]
    fn borrow(&self) -> &T {
        &self.0
    }
}

/// A type whose values can hold R's `NA`, with the check [`NonNa::new`]
/// makes.
///
/// [`has_na`](Self::has_na) counts what `anyNA()` counts: `NA_integer_`,
/// `NA` logicals, `NaN` doubles and complex numbers (R's `NA_real_` is one),
/// and `None`, which is how an `Option` element holds `NA`. A `String` never
/// holds one: R's `NA_character_` has no `String` form.
pub trait MaybeNa {
    /// Whether the value holds an `NA`.
    fn has_na(&self) -> bool;
}

impl MaybeNa for i32 {
    #[inline]
    fn has_na(&self) -> bool {
        IsNA::is_na(self)
    }
}

impl MaybeNa for RLogical {
    #[inline]
    fn has_na(&self) -> bool {
        IsNA::is_na(self)
    }
}

impl MaybeNa for f64 {
    #[inline]
    fn has_na(&self) -> bool {
        self.is_nan()
    }
}

impl MaybeNa for f32 {
    #[inline]
    fn has_na(&self) -> bool {
        self.is_nan()
    }
}

impl MaybeNa for Rcomplex {
    #[inline]
    fn has_na(&self) -> bool {
        self.r.is_nan() || self.i.is_nan()
    }
}

/// Types with no `NA` form: an R `NA` is refused when converting to them, or
/// (strings) has no value of the type.
macro_rules! never_na {
    ($($ty:ty),* $(,)?) => {$(
        impl MaybeNa for $ty {
            #[inline]
            fn has_na(&self) -> bool {
                false
            }
        }
    )*};
}

never_na!(
    bool, u8, i8, u16, i16, u32, i64, u64, isize, usize, String, str
);

impl<T: MaybeNa> MaybeNa for Option<T> {
    #[inline]
    fn has_na(&self) -> bool {
        self.as_ref().is_none_or(MaybeNa::has_na)
    }
}

impl<T: MaybeNa> MaybeNa for [T] {
    #[inline]
    fn has_na(&self) -> bool {
        self.iter().any(MaybeNa::has_na)
    }
}

impl<T: MaybeNa, const N: usize> MaybeNa for [T; N] {
    #[inline]
    fn has_na(&self) -> bool {
        self.as_slice().has_na()
    }
}

impl<T: MaybeNa> MaybeNa for Vec<T> {
    #[inline]
    fn has_na(&self) -> bool {
        self.as_slice().has_na()
    }
}

impl<T: MaybeNa + ?Sized> MaybeNa for Box<T> {
    #[inline]
    fn has_na(&self) -> bool {
        (**self).has_na()
    }
}

impl<T: MaybeNa + ?Sized> MaybeNa for &T {
    #[inline]
    fn has_na(&self) -> bool {
        (**self).has_na()
    }
}

#[cfg(test)]
mod tests;
