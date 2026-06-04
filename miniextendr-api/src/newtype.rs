//! Container conversions for forwarding newtypes.
//!
//! `#[derive(TryFromSexp)]` / `#[derive(IntoR)]` on a single-field newtype
//! (`struct UserId(Uuid)`) emit the scalar forwarding impls *and* a small marker
//! impl from this module. The container blankets here then light up
//! `Vec<UserId>`, `Option<UserId>`, and `Vec<Option<UserId>>` automatically — the
//! newtype inherits the inner type's exact SEXPTYPE checks, NA policy, and error
//! text in every shape.
//!
//! # Why the markers live here and not in the derive
//!
//! A downstream crate cannot write `impl TryFromSexp for Vec<MyNewtype>`: `Vec` /
//! `Option` are not `#[fundamental]`, so the local newtype is "covered" and the
//! orphan rule (E0117) forbids it. The container impls must live in
//! `miniextendr-api`, keyed on a marker trait the derive *can* legally implement
//! downstream (foreign trait + local `Self`). The blankets below are those
//! container impls. See `analysis/rconvert-containers-coherence-2026-06-04.md`
//! (issue #844) for the full coherence analysis.
//!
//! [`FromRNewtype`] / [`IntoRNewtype`] / [`IntoRVecElement`] are **plumbing**:
//! they are emitted by the derives, not implemented by hand. Implementing them
//! manually is supported but unusual.
//!
//! # The asymmetries
//!
//! Five of the six container shapes are granted. Two are not, for two different
//! coherence reasons:
//!
//! - **`IntoR for Vec<T>` (granted, but shared).** This slot has exactly one
//!   blanket and `MatchArg` already needs it (`Vec<MyEnum>` → STRSXP). Rather
//!   than a second, conflicting `Vec<T>` blanket (E0119), both paths funnel
//!   through [`IntoRVecElement`]: `MatchArg` types reach it via a bridge blanket
//!   in `match_arg.rs`, newtypes via a concrete impl emitted by
//!   `#[derive(IntoR)]`. A type that is *both* a `MatchArg` enum and an `IntoR`
//!   newtype is a coherence error — don't derive both on one type.
//! - **`IntoR for Option<T>` (not granted).** A bare `Option<T>` blanket
//!   collides with the pre-existing `impl<T: Copy + IntoR> IntoR for Option<&T>`:
//!   `&T` is `#[fundamental]`, so a downstream crate could impl `IntoRNewtype`
//!   for `&LocalType` and coherence cannot prove the two disjoint. Return
//!   `Option<Inner>` (`opt.map(|x| x.0)`) instead. See the note on the missing
//!   blanket below.
//!
//! `TryFromSexp for Vec<T>` / `Option<T>` / `Vec<Option<T>>` and `IntoR for
//! Vec<Option<T>>` are coherence-free: no other blanket occupies those slots.

use crate::SEXP;
use crate::from_r::TryFromSexp;
use crate::into_r::IntoR;

// region: marker traits (emitted by the derives, not hand-written)

/// Construct a forwarding newtype from its inner value (R → Rust side).
///
/// Emitted by `#[derive(TryFromSexp)]`. Powers the `TryFromSexp` container
/// blankets for `Vec<T>` / `Option<T>` / `Vec<Option<T>>` in this module.
pub trait FromRNewtype: Sized {
    /// The wrapped inner type, whose conversions are forwarded to.
    type Inner;

    /// Wrap an inner value into the newtype.
    fn from_inner(inner: Self::Inner) -> Self;
}

/// Unwrap a forwarding newtype into its inner value (Rust → R side).
///
/// Emitted by `#[derive(IntoR)]`. Powers the `IntoR` container blankets for
/// `Option<T>` / `Vec<Option<T>>` in this module.
pub trait IntoRNewtype {
    /// The wrapped inner type, whose conversions are forwarded to.
    type Inner;

    /// Unwrap the newtype into its inner value.
    fn into_inner(self) -> Self::Inner;
}

/// How a `Vec<Self>` becomes a single R vector SEXP.
///
/// This is the shared element-marker behind the **one** `impl<T: …> IntoR for
/// Vec<T>` blanket slot. Implemented concretely per type — by `#[derive(IntoR)]`
/// for newtypes (forwarding to `Vec<Inner>`), and by the `MatchArg` bridge in
/// `match_arg.rs` for `match.arg` enums (STRSXP by variant name). See the module
/// docs for why this cannot be two competing blankets.
pub trait IntoRVecElement: Sized {
    /// Convert all elements into one R vector SEXP.
    fn elements_into_sexp(values: Vec<Self>) -> SEXP;
}

// endregion

// region: IntoR for Vec<T> — the unified element-marker blanket

/// The single `IntoR for Vec<T>` blanket, shared by `MatchArg` enums and
/// `#[derive(IntoR)]` newtypes via [`IntoRVecElement`]. Coexists with the
/// concrete `impl IntoR for Vec<i32>` (etc.) impls: `IntoRVecElement` is
/// crate-local, so coherence proves the foreign R-native types do not implement
/// it.
impl<T: IntoRVecElement> IntoR for Vec<T> {
    type Error = std::convert::Infallible;

    #[inline]
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }

    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        self.try_into_sexp()
    }

    #[inline]
    fn into_sexp(self) -> SEXP {
        T::elements_into_sexp(self)
    }
}

// endregion

// region: TryFromSexp container blankets (R → Rust)

impl<T: FromRNewtype> TryFromSexp for Vec<T>
where
    Vec<T::Inner>: TryFromSexp,
{
    type Error = <Vec<T::Inner> as TryFromSexp>::Error;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        Ok(<Vec<T::Inner> as TryFromSexp>::try_from_sexp(sexp)?
            .into_iter()
            .map(T::from_inner)
            .collect())
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Ok(
            unsafe { <Vec<T::Inner> as TryFromSexp>::try_from_sexp_unchecked(sexp) }?
                .into_iter()
                .map(T::from_inner)
                .collect(),
        )
    }
}

impl<T: FromRNewtype> TryFromSexp for Option<T>
where
    Option<T::Inner>: TryFromSexp,
{
    type Error = <Option<T::Inner> as TryFromSexp>::Error;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        Ok(<Option<T::Inner> as TryFromSexp>::try_from_sexp(sexp)?.map(T::from_inner))
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Ok(
            unsafe { <Option<T::Inner> as TryFromSexp>::try_from_sexp_unchecked(sexp) }?
                .map(T::from_inner),
        )
    }
}

impl<T: FromRNewtype> TryFromSexp for Vec<Option<T>>
where
    Vec<Option<T::Inner>>: TryFromSexp,
{
    type Error = <Vec<Option<T::Inner>> as TryFromSexp>::Error;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        Ok(<Vec<Option<T::Inner>> as TryFromSexp>::try_from_sexp(sexp)?
            .into_iter()
            .map(|opt| opt.map(T::from_inner))
            .collect())
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Ok(
            unsafe { <Vec<Option<T::Inner>> as TryFromSexp>::try_from_sexp_unchecked(sexp) }?
                .into_iter()
                .map(|opt| opt.map(T::from_inner))
                .collect(),
        )
    }
}

// endregion

// region: IntoR container blankets (Rust → R) for Vec<Option>

// NOTE: there is deliberately no `impl<T: IntoRNewtype> IntoR for Option<T>`.
// That bare `Option<T>` blanket collides (E0119) with the pre-existing
// `impl<T: Copy + IntoR> IntoR for Option<&T>` (into_r/large_integers.rs): `&T`
// is `#[fundamental]`, so a downstream crate *could* implement `IntoRNewtype`
// for `&LocalType`, and coherence cannot prove the two disjoint. Returning a
// `Option<MyNewtype>` to R is the one shape the derive does not grant — map to
// the inner first (`opt.map(|x| x.0)` → `Option<Inner>`), which mirrors the
// NULL-vs-NA guidance already on the `Option<&T>` impl. See issue #844.

impl<T: IntoRNewtype> IntoR for Vec<Option<T>>
where
    Vec<Option<T::Inner>>: IntoR,
{
    type Error = <Vec<Option<T::Inner>> as IntoR>::Error;

    #[inline]
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        self.into_iter()
            .map(|opt| opt.map(T::into_inner))
            .collect::<Vec<Option<T::Inner>>>()
            .try_into_sexp()
    }

    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        unsafe {
            self.into_iter()
                .map(|opt| opt.map(T::into_inner))
                .collect::<Vec<Option<T::Inner>>>()
                .try_into_sexp_unchecked()
        }
    }

    #[inline]
    fn into_sexp(self) -> SEXP {
        self.into_iter()
            .map(|opt| opt.map(T::into_inner))
            .collect::<Vec<Option<T::Inner>>>()
            .into_sexp()
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> SEXP {
        unsafe {
            self.into_iter()
                .map(|opt| opt.map(T::into_inner))
                .collect::<Vec<Option<T::Inner>>>()
                .into_sexp_unchecked()
        }
    }
}

// endregion
