//! Container conversions for newtypes and string-parsed types.
//!
//! `#[derive(TryFromSexp)]` / `#[derive(IntoR)]` on a single-field newtype
//! (`struct UserId(Uuid)`) emit the scalar forwarding impls *and* a small marker
//! impl from this module. The container blankets here then light up
//! `Vec<UserId>`, `Option<UserId>`, and `Vec<Option<UserId>>` automatically — the
//! newtype inherits the inner type's exact SEXPTYPE checks, NA policy, error
//! text and `TryFromSexp::CHARACTER_ONLY` in every shape.
//!
//! A type parsed from an R string gets the same three shapes from
//! [`try_from_sexp_via_str_parse!`](crate::try_from_sexp_via_str_parse), which
//! implements [`TryFromSexpElement`] with
//! [`ParsedRStr`](crate::from_r::ParsedRStr) as the inner type (#1766).
//!
//! # The containers forward; a check refuses
//!
//! The containers never call the element's own `TryFromSexp::try_from_sexp`.
//! `Option<T>` reads `Option<T::Inner>`, `Vec<T>` reads `Vec<T::Inner>` and
//! `Vec<Option<T>>` reads `Vec<Option<T::Inner>>`, each wrapping what it
//! read with [`TryFromSexpElement::from_inner`]. So an element's scalar
//! `try_from_sexp` must be the inner type's followed by `from_inner`, which
//! is what the derive emits, and refusing values a type's inner type
//! accepts belongs in [`TryFromSexpElement::check_sexp`], not in
//! `try_from_sexp`. The check runs on the R value before it is read, in the
//! scalar `try_from_sexp`, on a non-`NULL` input for `Option<T>` (`NULL` is
//! "not given"), and for `Vec<T>` / `Vec<Option<T>>` where R keeps the
//! element's class. `#[derive(TryFromSexp)]` takes it as
//! `#[try_from_sexp(validate = path)]` (#1815).
//!
//! # Where a `Vec`'s check runs: the inner container decides
//!
//! An atomic vector keeps one class for all its elements, on the vector; a
//! list of R objects keeps a class on each element. So the `Vec` blankets
//! don't run the check themselves: they pass `T::check_sexp` to the inner
//! container's hidden
//! [`TryFromSexp::__mx_try_from_sexp_with_check`](crate::TryFromSexp::__mx_try_from_sexp_with_check)
//! (#1837). Its default runs the check once on the whole input, which every
//! atomic inner container keeps. `Vec<List>` and `Vec<Option<List>>` override
//! it to run the check on each element (but `NULL`) and to report every
//! failing element in one error, with its position. The blankets override it
//! too, adding their own check after the one they are given, so a newtype of a
//! newtype over `List` checks each element, outer check first, as its scalar
//! does.
//!
//! This keeps one blanket per container slot and one element trait: the
//! choice follows the inner type, through an impl that only `miniextendr-api`
//! can write (`Vec<List>`), not a second marker trait a downstream type could
//! implement next to `TryFromSexpElement`. The `Option<T>` blanket runs the
//! check itself: a scalar keeps its class on itself whatever its inner type.
//!
//! # Why the markers live here and not in the derive
//!
//! A downstream crate cannot write `impl TryFromSexp for Vec<MyNewtype>`: `Vec` /
//! `Option` are not `#[fundamental]`, so the local newtype is "covered" and the
//! orphan rule (E0117) forbids it. The container impls must live in
//! `miniextendr-api`, keyed on a marker trait the derive *can* legally implement
//! downstream (foreign trait + local `Self`). The blankets below are those
//! container impls (#844).
//!
//! [`TryFromSexpElement`] / [`IntoRNewtype`] / [`IntoRVecElement`] are
//! **plumbing**: they are emitted by the derives and by
//! `try_from_sexp_via_str_parse!`, not implemented by hand. Implementing them
//! manually is supported but unusual. A hand-written [`TryFromSexpElement`]
//! also needs a `TryFromSexp` impl on the type for `Option<T>` to convert (the
//! derive and the macro emit both), and that impl must keep the forwarding
//! rule above: call [`TryFromSexpElement::check_sexp`], then convert the inner
//! type and wrap it, and nothing else, since no container ever runs it.
//!
//! # One element trait per direction
//!
//! Each `TryFromSexp` container slot (`Vec<T>`, `Option<T>`, `Vec<Option<T>>`)
//! holds exactly one blanket, keyed on [`TryFromSexpElement`]. A second blanket
//! on another public trait (say, a parse trait) would collide with it (E0119):
//! a downstream type could implement both, so coherence cannot prove them
//! disjoint. So every non-built-in element goes through this one trait, and
//! what differs is the inner type the containers read first: the newtype's
//! field type, or `ParsedRStr<T>` for a string-parsed type, whose container
//! impls in `from_r` hold the NA policy and the batched element errors.
//!
//! Every container's error is [`SexpError`]: the inner container's error
//! converts into it, and so does a check's refusal
//! ([`SexpError::Condition`](crate::from_r::SexpError::Condition), carrying
//! its own classes). The inner containers that exist are all `miniextendr-api`
//! impls (the orphan rule keeps the rest out), and each one's error is a
//! `SexpError` or converts into one.
//!
//! The blankets bound the inner type through the hidden `VecInner` /
//! `OptionInner` / `VecOptionInner` helpers rather than
//! `Vec<T::Inner>: TryFromSexp`. For a type that is not an element,
//! `T::Inner` cannot be normalized; with the projection as the bound's self
//! type, the solver gives up on it at once and reports
//! `T: TryFromSexpElement` as unsatisfied (E0277). Spelled
//! `Vec<T::Inner>: TryFromSexp`, the same bound sent the solver through
//! every `Vec<_>` impl (`Vec<Vec<_>>` nests one level per step) until it
//! overflowed (E0275, #1682).
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
//! The concrete `TryFromSexp for Vec<List>` / `Vec<Option<List>>` impls sit
//! beside them because `List` is not a `TryFromSexpElement`, and no other
//! crate can make it one (both are `miniextendr-api`'s).

use crate::from_r::{ElementCheck, NativeBorrow, SexpError, TryFromSexp, no_element_check};
use crate::into_r::IntoR;
use crate::{SEXP, SEXPTYPE, SexpExt};

// region: marker traits (emitted by the derives, not hand-written)

/// The element side of the `TryFromSexp` impls for `Vec<T>`, `Option<T>` and
/// `Vec<Option<T>>` (R → Rust).
///
/// Each container of `Self` is read as the same container of
/// [`Inner`](Self::Inner), then every element is wrapped with
/// [`from_inner`](Self::from_inner): `Vec<Self>` reads `Vec<Self::Inner>`,
/// `Option<Self>` reads `Option<Self::Inner>`, and `Vec<Option<Self>>` reads
/// `Vec<Option<Self::Inner>>`. Each shape exists when the inner container
/// converts, independently of the other two, and keeps its NA policy and
/// metadata (`NATIVE_BORROW`, `CHARACTER_ONLY`); its error converts into
/// [`SexpError`].
///
/// # Forwarding
///
/// No container calls `<Self as TryFromSexp>::try_from_sexp`. They read the
/// inner type and wrap it, so the element's own `try_from_sexp`, when it has
/// one, must do the same: [`check_sexp`](Self::check_sexp), then the inner
/// type's conversion, then [`from_inner`](Self::from_inner). Anything else
/// it did would hold for a scalar argument and silently not for
/// `Option<Self>`, `Vec<Self>` or `Vec<Option<Self>>`. A value the inner type
/// accepts but `Self` refuses is refused in `check_sexp`, which every shape
/// runs.
///
/// Emitted by `#[derive(TryFromSexp)]` (the inner type is the newtype's field;
/// `#[try_from_sexp(validate = path)]` gives the check) and by
/// [`try_from_sexp_via_str_parse!`](crate::try_from_sexp_via_str_parse)
/// (the inner type is [`ParsedRStr<Self>`](crate::from_r::ParsedRStr)). See
/// the module docs for why both share this one trait.
#[diagnostic::on_unimplemented(
    message = "this `Vec` or `Option` of `{Self}` has no conversion from R",
    label = "`{Self}` does not implement `TryFromSexpElement`",
    note = "`Vec<T>`, `Option<T>` and `Vec<Option<T>>` convert from R for the built-in types, `#[derive(TryFromSexp)]` newtypes over a type whose container converts, and types given `try_from_sexp_via_str_parse!`"
)]
pub trait TryFromSexpElement: Sized {
    /// The type whose containers are read first.
    type Inner;

    /// Wrap an inner value into `Self`.
    fn from_inner(inner: Self::Inner) -> Self;

    /// Refuse an R value before it is read: `Err` for a value the inner type
    /// would accept but `Self` must not, such as a number that carries a unit
    /// class (`difftime`, `Date`, `POSIXct`).
    ///
    /// Called with the input of every shape, before its inner conversion: by
    /// the scalar `TryFromSexp` (the derive's calls it), for `Option<Self>`
    /// on any input but `NULL`, which stays "not given", and for `Vec<Self>`
    /// and `Vec<Option<Self>>` where R keeps the element's class: once on the
    /// whole vector for an atomic inner type, on each element (but `NULL`)
    /// for a list of R objects (`Self::Inner` is `List`, or a newtype over
    /// it). It sees the value as R holds it, attributes and all, before
    /// anything is read from it.
    ///
    /// Return an [`RError`](crate::condition::RError), or any
    /// [`RConditionError`](crate::condition::RConditionError) type, with `?`
    /// or `.into()` to give the refusal its own classes and fields
    /// ([`SexpError::Condition`]). As an
    /// argument error it keeps the wrapper's context: `'<p>' must be
    /// <expected>: <message>`, the classes before the crate's
    /// `conversion_error_class`, `e$param` and `e$rust_type`. A plain
    /// `SexpError` (`SexpError::InvalidValue`) works too, with no classes.
    ///
    /// The default accepts every value; it is what the derive emits without
    /// `validate` and what a string-parsed type has.
    #[inline]
    fn check_sexp(_sexp: SEXP) -> Result<(), SexpError> {
        Ok(())
    }
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

// region: container helpers (bound on the inner type itself, #1682)

/// `Vec<Self>: TryFromSexp` with an error that converts into [`SexpError`],
/// with `Self` as the bound's self type.
///
/// Plumbing for the `Vec<T>` blanket below: see the module docs (#1682).
#[doc(hidden)]
pub trait VecInner: Sized {
    /// `<Vec<Self> as TryFromSexp>::NATIVE_BORROW`.
    const VEC_NATIVE_BORROW: Option<NativeBorrow>;
    /// `<Vec<Self> as TryFromSexp>::CHARACTER_ONLY`.
    const VEC_CHARACTER_ONLY: bool;
    /// `<Vec<Self> as TryFromSexp>::__mx_try_from_sexp_with_check`: the
    /// vector, with `check` run where `Vec<Self>` keeps an element's class.
    fn vec_try_from_sexp(sexp: SEXP, check: &ElementCheck<'_>) -> Result<Vec<Self>, SexpError>;
    /// `<Vec<Self> as TryFromSexp>::__mx_try_from_sexp_unchecked_with_check`.
    ///
    /// # Safety
    ///
    /// As [`TryFromSexp::try_from_sexp_unchecked`].
    unsafe fn vec_try_from_sexp_unchecked(
        sexp: SEXP,
        check: &ElementCheck<'_>,
    ) -> Result<Vec<Self>, SexpError>;
}

impl<I> VecInner for I
where
    Vec<I>: TryFromSexp,
    <Vec<I> as TryFromSexp>::Error: Into<SexpError>,
{
    const VEC_NATIVE_BORROW: Option<NativeBorrow> = <Vec<I> as TryFromSexp>::NATIVE_BORROW;
    const VEC_CHARACTER_ONLY: bool = <Vec<I> as TryFromSexp>::CHARACTER_ONLY;

    #[inline]
    fn vec_try_from_sexp(sexp: SEXP, check: &ElementCheck<'_>) -> Result<Vec<Self>, SexpError> {
        <Vec<I> as TryFromSexp>::__mx_try_from_sexp_with_check(sexp, check)
    }

    #[inline]
    unsafe fn vec_try_from_sexp_unchecked(
        sexp: SEXP,
        check: &ElementCheck<'_>,
    ) -> Result<Vec<Self>, SexpError> {
        unsafe { <Vec<I> as TryFromSexp>::__mx_try_from_sexp_unchecked_with_check(sexp, check) }
    }
}

/// `Option<Self>: TryFromSexp` with an error that converts into
/// [`SexpError`], with `Self` as the bound's self type.
///
/// Plumbing for the `Option<T>` blanket below: see the module docs (#1682).
#[doc(hidden)]
pub trait OptionInner: Sized {
    /// `<Option<Self> as TryFromSexp>::NATIVE_BORROW`.
    const OPTION_NATIVE_BORROW: Option<NativeBorrow>;
    /// `<Option<Self> as TryFromSexp>::CHARACTER_ONLY`.
    const OPTION_CHARACTER_ONLY: bool;
    /// `<Option<Self> as TryFromSexp>::try_from_sexp`, its error as a
    /// [`SexpError`].
    fn option_try_from_sexp(sexp: SEXP) -> Result<Option<Self>, SexpError>;
    /// `<Option<Self> as TryFromSexp>::try_from_sexp_unchecked`, its error as
    /// a [`SexpError`].
    ///
    /// # Safety
    ///
    /// As [`TryFromSexp::try_from_sexp_unchecked`].
    unsafe fn option_try_from_sexp_unchecked(sexp: SEXP) -> Result<Option<Self>, SexpError>;
}

impl<I> OptionInner for I
where
    Option<I>: TryFromSexp,
    <Option<I> as TryFromSexp>::Error: Into<SexpError>,
{
    const OPTION_NATIVE_BORROW: Option<NativeBorrow> = <Option<I> as TryFromSexp>::NATIVE_BORROW;
    const OPTION_CHARACTER_ONLY: bool = <Option<I> as TryFromSexp>::CHARACTER_ONLY;

    #[inline]
    fn option_try_from_sexp(sexp: SEXP) -> Result<Option<Self>, SexpError> {
        <Option<I> as TryFromSexp>::try_from_sexp(sexp).map_err(Into::into)
    }

    #[inline]
    unsafe fn option_try_from_sexp_unchecked(sexp: SEXP) -> Result<Option<Self>, SexpError> {
        unsafe { <Option<I> as TryFromSexp>::try_from_sexp_unchecked(sexp) }.map_err(Into::into)
    }
}

/// `Vec<Option<Self>>: TryFromSexp` with an error that converts into
/// [`SexpError`], with `Self` as the bound's self type.
///
/// Plumbing for the `Vec<Option<T>>` blanket below: see the module docs
/// (#1682).
#[doc(hidden)]
pub trait VecOptionInner: Sized {
    /// `<Vec<Option<Self>> as TryFromSexp>::NATIVE_BORROW`.
    const VEC_OPTION_NATIVE_BORROW: Option<NativeBorrow>;
    /// `<Vec<Option<Self>> as TryFromSexp>::CHARACTER_ONLY`.
    const VEC_OPTION_CHARACTER_ONLY: bool;
    /// `<Vec<Option<Self>> as TryFromSexp>::__mx_try_from_sexp_with_check`:
    /// the vector, with `check` run where `Vec<Option<Self>>` keeps an
    /// element's class.
    fn vec_option_try_from_sexp(
        sexp: SEXP,
        check: &ElementCheck<'_>,
    ) -> Result<Vec<Option<Self>>, SexpError>;
    /// `<Vec<Option<Self>> as TryFromSexp>::__mx_try_from_sexp_unchecked_with_check`.
    ///
    /// # Safety
    ///
    /// As [`TryFromSexp::try_from_sexp_unchecked`].
    unsafe fn vec_option_try_from_sexp_unchecked(
        sexp: SEXP,
        check: &ElementCheck<'_>,
    ) -> Result<Vec<Option<Self>>, SexpError>;
}

impl<I> VecOptionInner for I
where
    Vec<Option<I>>: TryFromSexp,
    <Vec<Option<I>> as TryFromSexp>::Error: Into<SexpError>,
{
    const VEC_OPTION_NATIVE_BORROW: Option<NativeBorrow> =
        <Vec<Option<I>> as TryFromSexp>::NATIVE_BORROW;
    const VEC_OPTION_CHARACTER_ONLY: bool = <Vec<Option<I>> as TryFromSexp>::CHARACTER_ONLY;

    #[inline]
    fn vec_option_try_from_sexp(
        sexp: SEXP,
        check: &ElementCheck<'_>,
    ) -> Result<Vec<Option<Self>>, SexpError> {
        <Vec<Option<I>> as TryFromSexp>::__mx_try_from_sexp_with_check(sexp, check)
    }

    #[inline]
    unsafe fn vec_option_try_from_sexp_unchecked(
        sexp: SEXP,
        check: &ElementCheck<'_>,
    ) -> Result<Vec<Option<Self>>, SexpError> {
        unsafe {
            <Vec<Option<I>> as TryFromSexp>::__mx_try_from_sexp_unchecked_with_check(sexp, check)
        }
    }
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

// The `Vec` blankets hand `T::check_sexp`, after the check of any enclosing
// newtype, to the inner container's `__mx_try_from_sexp_with_check`, which
// runs it where that container keeps an element's class: once on the whole
// input for an atomic vector, on each element for a list of R objects
// (#1837). They then wrap what was read with `T::from_inner`. `Option<T>`
// runs the check itself, on a non-`NULL` input. None calls
// `T::try_from_sexp`: see "Forwarding" on `TryFromSexpElement`.

impl<T: TryFromSexpElement> TryFromSexp for Vec<T>
where
    T::Inner: VecInner,
{
    type Error = SexpError;
    const NATIVE_BORROW: Option<NativeBorrow> = <T::Inner as VecInner>::VEC_NATIVE_BORROW;
    const CHARACTER_ONLY: bool = <T::Inner as VecInner>::VEC_CHARACTER_ONLY;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        Self::__mx_try_from_sexp_with_check(sexp, &no_element_check)
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        unsafe { Self::__mx_try_from_sexp_unchecked_with_check(sexp, &no_element_check) }
    }

    #[inline]
    fn __mx_try_from_sexp_with_check(
        sexp: SEXP,
        check: &ElementCheck<'_>,
    ) -> Result<Self, SexpError> {
        let check = |x: SEXP| {
            check(x)?;
            T::check_sexp(x)
        };
        Ok(<T::Inner as VecInner>::vec_try_from_sexp(sexp, &check)?
            .into_iter()
            .map(T::from_inner)
            .collect())
    }

    #[inline]
    unsafe fn __mx_try_from_sexp_unchecked_with_check(
        sexp: SEXP,
        check: &ElementCheck<'_>,
    ) -> Result<Self, SexpError> {
        let check = |x: SEXP| {
            check(x)?;
            T::check_sexp(x)
        };
        Ok(
            unsafe { <T::Inner as VecInner>::vec_try_from_sexp_unchecked(sexp, &check) }?
                .into_iter()
                .map(T::from_inner)
                .collect(),
        )
    }
}

// `T: TryFromSexp` lets `__mx_has_na` ask the element itself
// (`TryFromSexpElement` has no inner accessor, and an override cannot add a
// bound, E0276). The derive and the str-parse macro always emit both impls.
impl<T: TryFromSexpElement + TryFromSexp> TryFromSexp for Option<T>
where
    T::Inner: OptionInner,
{
    type Error = SexpError;
    const NATIVE_BORROW: Option<NativeBorrow> = <T::Inner as OptionInner>::OPTION_NATIVE_BORROW;
    const CHARACTER_ONLY: bool = <T::Inner as OptionInner>::OPTION_CHARACTER_ONLY;
    // `NULL` (`None`) is "not given" and passes `no_na`.
    #[inline]
    fn __mx_has_na(&self) -> bool {
        self.as_ref().is_some_and(T::__mx_has_na)
    }

    // `NULL` is "not given" too, so the check sees only a value.
    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() != SEXPTYPE::NILSXP {
            T::check_sexp(sexp)?;
        }
        Ok(<T::Inner as OptionInner>::option_try_from_sexp(sexp)?.map(T::from_inner))
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() != SEXPTYPE::NILSXP {
            T::check_sexp(sexp)?;
        }
        Ok(
            unsafe { <T::Inner as OptionInner>::option_try_from_sexp_unchecked(sexp) }?
                .map(T::from_inner),
        )
    }
}

impl<T: TryFromSexpElement> TryFromSexp for Vec<Option<T>>
where
    T::Inner: VecOptionInner,
{
    type Error = SexpError;
    const NATIVE_BORROW: Option<NativeBorrow> =
        <T::Inner as VecOptionInner>::VEC_OPTION_NATIVE_BORROW;
    const CHARACTER_ONLY: bool = <T::Inner as VecOptionInner>::VEC_OPTION_CHARACTER_ONLY;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        Self::__mx_try_from_sexp_with_check(sexp, &no_element_check)
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        unsafe { Self::__mx_try_from_sexp_unchecked_with_check(sexp, &no_element_check) }
    }

    #[inline]
    fn __mx_try_from_sexp_with_check(
        sexp: SEXP,
        check: &ElementCheck<'_>,
    ) -> Result<Self, SexpError> {
        let check = |x: SEXP| {
            check(x)?;
            T::check_sexp(x)
        };
        Ok(
            <T::Inner as VecOptionInner>::vec_option_try_from_sexp(sexp, &check)?
                .into_iter()
                .map(|opt| opt.map(T::from_inner))
                .collect(),
        )
    }

    #[inline]
    unsafe fn __mx_try_from_sexp_unchecked_with_check(
        sexp: SEXP,
        check: &ElementCheck<'_>,
    ) -> Result<Self, SexpError> {
        let check = |x: SEXP| {
            check(x)?;
            T::check_sexp(x)
        };
        Ok(unsafe {
            <T::Inner as VecOptionInner>::vec_option_try_from_sexp_unchecked(sexp, &check)
        }?
        .into_iter()
        .map(|opt| opt.map(T::from_inner))
        .collect())
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
