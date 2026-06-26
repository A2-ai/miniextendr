//! [`RCow`] — an R-aware copy-on-write slice.
//!
//! Like [`std::borrow::Cow<[T]>`](std::borrow::Cow), but the borrowed arm
//! carries the *source SEXP* it was read from. That single extra field is what
//! makes a zero-copy round-trip back to R **sound**, where `Cow<[T]>` could not
//! be (see #880).
//!
//! # The `Cow<[T]>` hazard this replaces
//!
//! `Cow<[T]>` is zero-copy on the way *in* — `TryFromSexp` hands back
//! `Cow::Borrowed(&[T])` pointing straight at R's vector data. But it cannot be
//! returned to R zero-copy *safely*. A bare `&[T]` carries no provenance, so a
//! borrowed **sub-slice** (`&cow[2..5]`) is byte-for-byte indistinguishable from
//! a full borrow, yet `as_ptr()` points into the *middle* of the R vector. The
//! old recovery path computed `data_ptr − SEXPREC_header` and speculatively read
//! there — landing mid-header for a sub-slice (false-positive corruption) or off
//! the front of a Rust-allocated buffer entirely (segfault — the same class of
//! bug fixed for Arrow in #867). Arrow could keep zero-copy because
//! `arrow::Buffer` *proves* it is unsliced and R-backed (`ptr_offset`/
//! `capacity`); a `&[T]` proves nothing.
//!
//! # How `RCow` fixes it structurally
//!
//! [`RCow::Borrowed`] wraps [`RBorrow`], whose fields are private. The *only*
//! constructor is [`RCow::try_from_sexp`], which always borrows a whole vector.
//! There is no way to build a borrowed `RCow` over a sub-range (slice via
//! [`Deref`] to get a plain `&[T]` instead). So a borrowed `RCow` always spans
//! its entire source vector, and [`IntoR`] simply returns the stored SEXP — no
//! pointer arithmetic, no speculative read, no hazard. Need to mutate or reshape?
//! [`to_mut`](RCow::to_mut) / [`into_owned`](RCow::into_owned) copy out into an
//! owned [`Vec<T>`].
//!
//! # Lifetime contract
//!
//! As with `Cow<[T]>`, a borrowed `RCow` is valid only for the duration of the
//! enclosing `.Call` (R protects the argument SEXP while Rust runs). Write
//! `RCow<'_, T>` at a `#[miniextendr]` boundary: the `'a` leashes the borrow to
//! the call, so sending it to another thread or storing it past the call return
//! is a *compile error*, not an honor-system pitfall.

use std::ops::Deref;

use crate::from_r::{SexpTypeError, TryFromSexp};
use crate::into_r::IntoR;
use crate::{RNativeType, SEXP};

/// Borrowed arm of [`RCow`]: a whole-vector view that remembers its source SEXP.
///
/// Fields are private by design — the only constructor is
/// [`RCow::try_from_sexp`], so a borrowed view can never be a sub-slice. That
/// invariant is what lets [`IntoR`] return the source SEXP zero-copy without the
/// provenance-free pointer probe that `Cow<[T]>` required (#880).
pub struct RBorrow<'a, T> {
    /// Source R vector. Valid for the duration of the enclosing `.Call`.
    sexp: SEXP,
    /// View over `sexp`'s data — always the whole vector (see invariant above).
    data: &'a [T],
}

impl<T> RBorrow<'_, T> {
    /// The borrowed view (the whole source vector).
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        self.data
    }

    /// The source R vector this view borrows from.
    #[inline]
    pub fn source_sexp(&self) -> SEXP {
        self.sexp
    }
}

/// An R-aware copy-on-write slice — the safe, zero-copy-round-trip alternative
/// to [`std::borrow::Cow<[T]>`](std::borrow::Cow).
///
/// See the [module docs](self) for why this exists and how it closes the #880
/// hazard. In brief: the [`Borrowed`](RCow::Borrowed) arm carries its source
/// SEXP, so returning it to R is a direct hand-back rather than a speculative
/// pointer recovery.
///
/// # Example
///
/// ```ignore
/// // Zero-copy in *and* out: the returned SEXP is the original R vector.
/// #[miniextendr]
/// pub fn passthrough(x: RCow<'_, f64>) -> RCow<'_, f64> {
///     x
/// }
///
/// // Mutating forces a copy (copy-on-write), then materializes a fresh vector.
/// #[miniextendr]
/// pub fn doubled(mut x: RCow<'_, f64>) -> RCow<'_, f64> {
///     for v in x.to_mut() {
///         *v *= 2.0;
///     }
///     x
/// }
/// ```
pub enum RCow<'a, T> {
    /// Zero-copy view of a whole R vector, carrying its source SEXP.
    Borrowed(RBorrow<'a, T>),
    /// Owned data; materializes a fresh R vector on [`IntoR`].
    Owned(Vec<T>),
}

impl<T> RCow<'_, T> {
    /// `true` if this is a borrowed (zero-copy) view of an R vector.
    #[inline]
    pub fn is_borrowed(&self) -> bool {
        matches!(self, RCow::Borrowed(_))
    }

    /// `true` if this owns its data.
    #[inline]
    pub fn is_owned(&self) -> bool {
        matches!(self, RCow::Owned(_))
    }
}

impl<T: Clone> RCow<'_, T> {
    /// Acquire a mutable reference to the owned data, cloning out of R first if
    /// borrowed (copy-on-write). After this the `RCow` is always
    /// [`Owned`](RCow::Owned).
    pub fn to_mut(&mut self) -> &mut Vec<T> {
        match self {
            RCow::Borrowed(b) => {
                *self = RCow::Owned(b.data.to_vec());
                match self {
                    RCow::Owned(v) => v,
                    // The line above just assigned `Owned`.
                    RCow::Borrowed(_) => unreachable!(),
                }
            }
            RCow::Owned(v) => v,
        }
    }

    /// Consume into an owned [`Vec<T>`], cloning out of R if borrowed.
    pub fn into_owned(self) -> Vec<T> {
        match self {
            RCow::Borrowed(b) => b.data.to_vec(),
            RCow::Owned(v) => v,
        }
    }
}

impl<T> Deref for RCow<'_, T> {
    type Target = [T];
    #[inline]
    fn deref(&self) -> &[T] {
        match self {
            RCow::Borrowed(b) => b.data,
            RCow::Owned(v) => v,
        }
    }
}

impl<T> From<Vec<T>> for RCow<'_, T> {
    #[inline]
    fn from(v: Vec<T>) -> Self {
        RCow::Owned(v)
    }
}

/// Reads an R vector zero-copy, remembering the source SEXP so it can be handed
/// straight back by [`IntoR`]. Rejects a mismatched [`SEXPTYPE`](crate::SEXPTYPE)
/// just like the `&[T]` impl it delegates to.
///
/// The impl is generic over `'a` so a borrowed `RCow<'_, T>` argument is leashed
/// to the enclosing `.Call` (R protects the argument SEXP for the call's
/// duration). Storing it past the call return — in an `ExternalPtr`, a global,
/// or another thread — is a compile error, not an honor-system hazard.
impl<'a, T: RNativeType> TryFromSexp for RCow<'a, T> {
    type Error = SexpTypeError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        // The `&[T]` impl fabricates &'static; covariance narrows it to &'a.
        let data: &'static [T] = TryFromSexp::try_from_sexp(sexp)?;
        Ok(RCow::Borrowed(RBorrow { sexp, data }))
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        let data: &'static [T] = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
        Ok(RCow::Borrowed(RBorrow { sexp, data }))
    }
}

/// Returns the source SEXP unchanged for the borrowed arm (true zero-copy), or
/// materializes a fresh R vector for the owned arm.
impl<T: RNativeType> IntoR for RCow<'_, T> {
    type Error = std::convert::Infallible;

    #[inline]
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }

    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }

    #[inline]
    fn into_sexp(self) -> SEXP {
        match self {
            // Invariant (enforced by `RBorrow`'s private fields): a borrowed
            // `RCow` always spans its entire source vector, so the source SEXP
            // *is* the correct result — hand it back directly. No
            // `data_ptr − header` probe, hence none of the #880 / #867
            // speculative-read hazard.
            RCow::Borrowed(b) => b.sexp,
            // Copies via the `&[T]` impl.
            RCow::Owned(v) => v.as_slice().into_sexp(),
        }
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> SEXP {
        match self {
            RCow::Borrowed(b) => b.sexp,
            RCow::Owned(v) => unsafe { v.as_slice().into_sexp_unchecked() },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The borrowed arm's behavior (TryFromSexp / zero-copy IntoR) requires a
    // live R session and is exercised by the rpkg `zero_copy_rcow_*` fixtures
    // and the `gc_stress_rcow_roundtrip` gctorture guard. These unit tests cover
    // the R-independent surface (the owned arm and the Cow-like helpers).

    #[test]
    fn owned_deref_and_into_owned() {
        let c: RCow<'static, i32> = RCow::Owned(vec![1, 2, 3]);
        assert!(c.is_owned());
        assert!(!c.is_borrowed());
        assert_eq!(&*c, &[1, 2, 3]);
        assert_eq!(c.into_owned(), vec![1, 2, 3]);
    }

    #[test]
    fn to_mut_on_owned_mutates_in_place() {
        let mut c: RCow<'static, f64> = RCow::Owned(vec![1.0]);
        c.to_mut().push(2.0);
        assert_eq!(&*c, &[1.0, 2.0]);
    }

    #[test]
    fn from_vec_is_owned() {
        let c: RCow<'static, i32> = vec![9, 8, 7].into();
        assert!(c.is_owned());
        assert_eq!(c.len(), 3);
    }
}
