//! [`AsInteger`] / [`AsIntegerVec`]: an integer argument read the way R's
//! `as.integer()` reads it, from numbers, text, or factor labels, refusing the
//! values it would truncate or turn into `NA`. The reading rules are
//! documented on [`AsIntegerVec`]; the reader is [`AsNumericVec`]'s.
//!
//! [`AsNumericVec`]: super::AsNumericVec

use super::as_numeric::{ReadNumber, Refused, read_numeric, read_scalar};
use crate::SEXP;
use crate::SEXPTYPE;
use crate::coerce::TryCoerce;
use crate::from_r::{SexpError, TryFromSexp, impl_option_try_from_sexp};
use crate::into_r::IntoR;

// region: Marker types

/// An integer scalar read like R's `as.integer()`: from an integer, logical,
/// double, character, or factor (by its labels) of length 1. `NA` of any type
/// is `None`.
///
/// It follows the same reading rules as [`AsIntegerVec`], and also requires
/// length 1 ([`SexpError::Length`] otherwise). A value it can't store fails
/// with `non-integer value(s): "3.5" (element 1)`. Under
/// `#[miniextendr(no_na)]` it refuses every `None`, with `'x' must not be NA`.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::{miniextendr, AsInteger};
///
/// #[miniextendr]
/// fn next_id(x: AsInteger) -> Option<i32> {
///     x.0.map(|v| v + 1)
/// }
/// // R: next_id(3)            → 4L
/// //    next_id(" 7 ")        → 8L
/// //    next_id(factor("10")) → 11L  (the label, not the code)
/// //    next_id(NA_character_) → NA
/// //    next_id(3.5)          → error: non-integer value(s): "3.5" (element 1)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AsInteger(pub Option<i32>);

/// An integer vector read like R's `as.integer()`: from an integer, logical,
/// double, character, or factor (by its labels) vector. `NA` of any type is
/// `None`.
///
/// It is [`AsNumericVec`](super::AsNumericVec)'s integer sibling: the same
/// inputs, the same text parsing and the same batched error, with each number
/// then stored as an `i32`. Where `as.integer()` would lose information, the
/// marker refuses the value instead: `as.integer(3.5)` truncates to `3` and
/// `as.integer(3e10)` gives `NA` with a warning, both of which the marker
/// reports.
///
/// # Reading rules
///
/// | R input | Result |
/// |---------|--------|
/// | integer | as is; `NA` → `None` |
/// | logical | `TRUE` → `1`, `FALSE` → `0`; `NA` → `None` |
/// | double | a whole number in R's integer range as is; `NA` and `NaN` → `None` (`as.integer(NaN)` is `NA`) |
/// | character | parsed as [`AsNumericVec`](super::AsNumericVec) parses it, then read as a double |
/// | factor | each *label* (`levels(x)[x]`) read as character, never the codes; code `NA` → `None` |
/// | anything else (list, raw, complex, …) | [`SexpError::Type`] |
///
/// R's integer range is `-2147483647..=2147483647`: `-2147483648` (`i32::MIN`)
/// is `NA_integer_`'s bit pattern, so it is out of range, as for
/// `as.integer(-2^31)`.
///
/// A value is refused when it is not a number (`"n/a"`), not whole (`3.5`,
/// `"2.5"`), or out of range (`Inf`, `"1e10"`, `-2^31`). All of them are
/// collected and reported together, with the text as given or the double as R
/// would print it: `non-integer value(s): "n/a", "3.5", "Inf" (elements 2, 3,
/// 5)`, 1-based element numbers, at most 10 values listed (the rest as
/// `"; and N more"`).
///
/// # `no_na`
///
/// `#[miniextendr(no_na)]` refuses everything this marker reads as missing,
/// not only what R's `anyNA()` sees: the R guard refuses `NA` (and a numeric
/// `NaN`) before the call, and the C wrapper checks the converted value after
/// it, refusing every `None` (the text `"NA"` or `"NaN"`, blank strings,
/// factor labels among those). Both raise the same `'x' must not contain NA`
/// argument error. `NULL` given to an `Option<AsIntegerVec>` is still `None`,
/// as not given. A type alias or a `#[derive(TryFromSexp)]` newtype of the
/// marker is checked the same way.
///
/// `Option<AsIntegerVec>` (and `Option<AsInteger>`) also accept `NULL` as
/// `None`.
///
/// # `IntoR`
///
/// The markers convert to R as their inner `Vec<Option<i32>>` /
/// `Option<i32>`: an integer vector with `NA` for `None`, which the marker
/// reads back unchanged, and `Option<marker>` gives `NULL` for `None`. That is
/// what lets a `#[miniextendr]` trait method take a marker parameter: the
/// trait's View converts each argument with `IntoR` and the implementing side
/// reads it back with `TryFromSexp`. A `#[miniextendr]` function returning a
/// marker gives the same R value as returning `.0`; returning `.0` stays the
/// idiom.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::{miniextendr, AsIntegerVec};
///
/// #[miniextendr]
/// fn total(x: AsIntegerVec) -> i64 {
///     x.0.into_iter().flatten().map(i64::from).sum()
/// }
/// // R: total(c(" 1 ", "NA", "", "0x10")) → 17
/// //    total(factor(c("10", "2")))       → 12  (labels, not codes)
/// //    total(c(1, 2.5, 3e10))
/// //    → error: non-integer value(s): "2.5", "3e+10" (elements 2, 3)
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AsIntegerVec(pub Vec<Option<i32>>);

impl ReadNumber for i32 {
    const STORAGE: SEXPTYPE = SEXPTYPE::INTSXP;
    const REFUSED: &'static str = "non-integer";
    #[inline]
    fn from_number(value: f64) -> Result<Option<Self>, Refused> {
        // `as.integer(NaN)` is `NA`, with no warning.
        if value.is_nan() {
            return Ok(None);
        }
        // Refuses `Inf`, values outside R's integer range (`i32::MIN`
        // included) and fractions.
        value.try_coerce().map(Some).map_err(|_| Refused)
    }
}

impl TryFromSexp for AsIntegerVec {
    type Error = SexpError;
    #[inline]
    fn __mx_has_na(&self) -> bool {
        self.0.contains(&None)
    }

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        read_numeric::<i32>(sexp).map(AsIntegerVec)
    }
}

impl TryFromSexp for AsInteger {
    type Error = SexpError;
    #[inline]
    fn __mx_has_na(&self) -> bool {
        self.0.is_none()
    }

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        read_scalar::<i32>(sexp).map(AsInteger)
    }
}

impl_option_try_from_sexp!(AsInteger);
impl_option_try_from_sexp!(AsIntegerVec);
// endregion

// region: IntoR (the inner value)

// A marker converts to R as its inner value, which the marker reads back
// unchanged: `None` is `NA_integer_`. A trait's View passes a marker argument
// to the implementing method this way.

impl IntoR for AsInteger {
    type Error = <Option<i32> as IntoR>::Error;
    #[inline]
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        self.0.try_into_sexp()
    }
    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        unsafe { self.0.try_into_sexp_unchecked() }
    }
    #[inline]
    fn into_sexp(self) -> SEXP {
        self.0.into_sexp()
    }
    #[inline]
    unsafe fn into_sexp_unchecked(self) -> SEXP {
        unsafe { self.0.into_sexp_unchecked() }
    }
}

impl IntoR for AsIntegerVec {
    type Error = <Vec<Option<i32>> as IntoR>::Error;
    #[inline]
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        self.0.try_into_sexp()
    }
    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        unsafe { self.0.try_into_sexp_unchecked() }
    }
    #[inline]
    fn into_sexp(self) -> SEXP {
        self.0.into_sexp()
    }
    #[inline]
    unsafe fn into_sexp_unchecked(self) -> SEXP {
        unsafe { self.0.into_sexp_unchecked() }
    }
}

impl_option_into_r_null!(AsInteger);
impl_option_into_r_null!(AsIntegerVec);
// endregion

#[cfg(test)]
mod tests;
