//! [`AsNumeric`] / [`AsNumericVec`]: a number argument read the way R's
//! `as.numeric()` reads it, from numbers, text, or factor labels. The reading
//! rules are documented on [`AsNumericVec`].

use crate::altrep_traits::NA_INTEGER;
use crate::from_r::{
    BatchedErrors, SexpError, SexpLengthError, SexpTypeError, TryFromSexp, charsxp_to_str,
    impl_option_try_from_sexp, is_na_real, map_strsxp_with,
};
use crate::into_r::IntoR;
use crate::{RLogical, SEXP, SEXPTYPE, SexpExt};

// region: Marker types

/// A numeric scalar read like R's `as.numeric()`: from a double, integer,
/// logical, character, or factor (by its labels) of length 1. `NA` of any
/// type is `None`.
///
/// It follows the same reading rules as [`AsNumericVec`], and also requires
/// length 1 ([`SexpError::Length`] otherwise). A value that is not a number
/// fails with `non-numeric value(s): "n/a" (element 1)`. Under
/// `#[miniextendr(no_na)]` it refuses the same values as the vector marker
/// (see [`AsNumericVec`], "`no_na`"), with `'x' must not be NA`.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::{miniextendr, AsNumeric};
///
/// #[miniextendr]
/// fn half(x: AsNumeric) -> Option<f64> {
///     x.0.map(|v| v / 2.0)
/// }
/// // R: half(3L)            → 1.5
/// //    half(" 1e3 ")       → 500
/// //    half(factor("10"))  → 5   (the label, not the code)
/// //    half(NA_character_) → NA
/// //    half("n/a")         → error: non-numeric value(s): "n/a" (element 1)
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AsNumeric(pub Option<f64>);

/// A numeric vector read like R's `as.numeric()`: from a double, integer,
/// logical, character, or factor (by its labels) vector. `NA` of any type is
/// `None`.
///
/// Data read from CSV files or spreadsheets often arrives as character, with
/// numbers mixed with tokens such as `"n/a"` or `"<0.1"`, or as a factor whose
/// labels are the numbers. This marker takes any of those, maps every kind of
/// `NA` to `None`, and names the values that are not numbers in one error.
///
/// # Reading rules
///
/// | R input | Result |
/// |---------|--------|
/// | double | as is; `NA_real_` → `None`, `NaN` stays `Some(NaN)` |
/// | integer, logical | widened to `f64`; `NA` → `None` |
/// | character | parsed as `as.numeric()` does (see below) |
/// | factor | each *label* (`levels(x)[x]`) parsed as character, never the codes; code `NA` → `None` |
/// | anything else (list, raw, complex, …) | [`SexpError::Type`] |
///
/// Character parsing follows R's `String2Real` (`RealFromString` in
/// `src/main/coerce.c`):
///
/// - `NA_character_` → `None`.
/// - A blank string (empty, or whitespace only) → `None`, as `as.numeric("")`
///   gives `NA` without a warning.
/// - The token `"NA"`, with blanks around it at most, → `None`. This follows
///   `scan()` / `type.convert()`, which read `"NA"` as missing;
///   `as.numeric("NA")` also gives `NA`, but with a coercion warning.
/// - Otherwise R's own `R_strtod` reads a number, and the rest of the string
///   must be blank. That accepts surrounding spaces, `"Inf"`, `"-inf"`,
///   `"Infinity"`, `"NaN"`, `"1e3"`, `"+.5"`, `"5."`, and hex such as `"0x1A"`
///   or `"0x1p3"`, exactly as the running R does.
/// - Anything else fails. All failures are collected and reported together:
///   `non-numeric value(s): "n/a", "<0.1" (elements 2, 5)`, with 1-based
///   element numbers and at most 10 values listed (the rest as `"; and N more"`).
///
/// # `no_na`
///
/// `#[miniextendr(no_na)]` refuses everything this marker reads as missing,
/// not only what R's `anyNA()` sees: the R guard refuses `NA` (and a numeric
/// `NaN`) before the call, and the C wrapper checks the converted value
/// after it, refusing `None` (the token `"NA"`, blank strings, factor labels
/// among those) and `NaN` (from the text `"NaN"` too). Both raise the same
/// `'x' must not contain NA` argument error. `NULL` given to an
/// `Option<AsNumericVec>` is still `None`, as not given. A type alias or a
/// `#[derive(TryFromSexp)]` newtype of the marker is checked the same way.
///
/// `Option<AsNumericVec>` (and `Option<AsNumeric>`) also accept `NULL` as
/// `None`.
///
/// # `IntoR`
///
/// The markers convert to R as their inner `Vec<Option<f64>>` /
/// `Option<f64>`: a double vector with `NA` for `None`, which the marker
/// reads back unchanged (`NaN` stays `Some(NaN)`, though a NaN carrying R's
/// NA payload reads back as `None`, as `R_IsNA` says), and `Option<marker>`
/// gives `NULL` for `None`. That is what lets a `#[miniextendr]` trait method
/// take a marker parameter: the trait's View converts each argument with
/// `IntoR` and the implementing side reads it back with `TryFromSexp`. A
/// `#[miniextendr]` function returning a marker gives the same R value as
/// returning `.0`; returning `.0` stays the idiom.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::{miniextendr, AsNumericVec};
///
/// #[miniextendr]
/// fn total(x: AsNumericVec) -> f64 {
///     x.0.into_iter().flatten().sum()
/// }
/// // R: total(c(" 1.5 ", "NA", "", "0x10")) → 17.5
/// //    total(factor(c("10", "2")))         → 12  (labels, not codes)
/// //    total(c("1", "n/a", "3", "4", "<0.1"))
/// //    → error: non-numeric value(s): "n/a", "<0.1" (elements 2, 5)
/// ```
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AsNumericVec(pub Vec<Option<f64>>);

impl TryFromSexp for AsNumericVec {
    type Error = SexpError;
    #[inline]
    fn __mx_has_na(&self) -> bool {
        self.0.iter().copied().any(refused_by_no_na)
    }

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        read_numeric(sexp).map(AsNumericVec)
    }
}

impl TryFromSexp for AsNumeric {
    type Error = SexpError;
    #[inline]
    fn __mx_has_na(&self) -> bool {
        refused_by_no_na(self.0)
    }

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        // The type is checked before the length, so a length-1 list reports
        // its type rather than passing the length check.
        check_type(sexp.type_of())?;
        let len = sexp.len();
        if len != 1 {
            return Err(SexpLengthError {
                expected: 1,
                actual: len,
            }
            .into());
        }
        let values = read_numeric(sexp)?;
        Ok(AsNumeric(values[0]))
    }
}

impl_option_try_from_sexp!(AsNumeric);
impl_option_try_from_sexp!(AsNumericVec);

/// Whether `#[miniextendr(no_na)]` refuses a value the markers read: `None`
/// (every kind of `NA`, the text `"NA"`, a blank string) and `NaN` (also read
/// from the text `"NaN"`), as R's `anyNA()` refuses a numeric `NaN`.
fn refused_by_no_na(value: Option<f64>) -> bool {
    value.is_none_or(f64::is_nan)
}
// endregion

// region: IntoR (the inner value)

// A marker converts to R as its inner value, which the marker reads back
// unchanged: `None` is `NA_real_` and `Some(NaN)` stays `NaN` (a NaN carrying
// R's NA payload, as from a plain `f64` argument given `NA_real_`, reads back
// as `None`). A trait's View passes a marker argument to the implementing
// method this way.

impl IntoR for AsNumeric {
    type Error = <Option<f64> as IntoR>::Error;
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

impl IntoR for AsNumericVec {
    type Error = <Vec<Option<f64>> as IntoR>::Error;
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

impl_option_into_r_null!(AsNumeric);
impl_option_into_r_null!(AsNumericVec);
// endregion

// region: SEXP dispatch

/// A value that `as.numeric()` would turn into `NA` with a coercion warning.
struct NotNumeric;

/// The rejected-element accumulator: 0-based index and the offending text.
type Rejected = BatchedErrors;

/// Refuse every SEXPTYPE the markers do not read. `REALSXP` is named as the
/// expected type because that is the storage the value ends up in.
fn check_type(actual: SEXPTYPE) -> Result<(), SexpError> {
    match actual {
        SEXPTYPE::REALSXP | SEXPTYPE::INTSXP | SEXPTYPE::LGLSXP | SEXPTYPE::STRSXP => Ok(()),
        _ => Err(SexpTypeError {
            expected: SEXPTYPE::REALSXP,
            actual,
        }
        .into()),
    }
}

/// Read any accepted input into `Vec<Option<f64>>`, collecting every
/// non-numeric element into one error.
fn read_numeric(sexp: SEXP) -> Result<Vec<Option<f64>>, SexpError> {
    let actual = sexp.type_of();
    check_type(actual)?;
    match actual {
        SEXPTYPE::REALSXP => {
            let values: &[f64] = unsafe { sexp.as_slice() };
            Ok(values
                .iter()
                .map(|&v| (!is_na_real(v)).then_some(v))
                .collect())
        }
        SEXPTYPE::INTSXP if sexp.is_factor() => read_factor_labels(sexp),
        SEXPTYPE::INTSXP => {
            let values: &[i32] = unsafe { sexp.as_slice() };
            Ok(values
                .iter()
                .map(|&v| (v != NA_INTEGER).then(|| f64::from(v)))
                .collect())
        }
        SEXPTYPE::LGLSXP => {
            let values: &[RLogical] = unsafe { sexp.as_slice() };
            Ok(values
                .iter()
                .map(|v| (!v.is_na()).then(|| f64::from(v.to_i32())))
                .collect())
        }
        // STRSXP: `check_type` admitted nothing else.
        _ => {
            let mut rejected = Rejected::default();
            let values = map_strsxp_with(sexp, |charsxp, i| {
                Ok(parse_charsxp(charsxp).unwrap_or_else(|NotNumeric| {
                    rejected.push(i, || unsafe { charsxp_to_str(charsxp) }.to_owned());
                    None
                }))
            })?;
            finish(values, rejected)
        }
    }
}

/// Read a factor by its labels: each level is parsed once, then every code
/// looks its level up. Code `NA` is `None`; a code outside the levels is a
/// malformed factor (R's `as.character()` refuses it too).
fn read_factor_labels(sexp: SEXP) -> Result<Vec<Option<f64>>, SexpError> {
    let levels = sexp.get_levels();
    if levels.type_of() != SEXPTYPE::STRSXP {
        return Err(SexpError::InvalidValue(
            "malformed factor: levels are not a character vector".to_string(),
        ));
    }
    let parsed: Vec<Result<Option<f64>, NotNumeric>> =
        map_strsxp_with(levels, |charsxp, _| Ok(parse_charsxp(charsxp)))?;

    let codes: &[i32] = unsafe { sexp.as_slice() };
    let mut values = Vec::with_capacity(codes.len());
    let mut rejected = Rejected::default();
    for (i, &code) in codes.iter().enumerate() {
        if code == NA_INTEGER {
            values.push(None);
            continue;
        }
        let level = usize::try_from(code)
            .ok()
            .and_then(|c| c.checked_sub(1))
            .filter(|&k| k < parsed.len());
        let Some(k) = level else {
            return Err(SexpError::InvalidValue(format!(
                "malformed factor: code {code} at element {} is outside levels 1..={}",
                i + 1,
                parsed.len()
            )));
        };
        match parsed[k] {
            Ok(v) => values.push(v),
            Err(NotNumeric) => {
                rejected.push(i, || level_label(levels, k));
                values.push(None);
            }
        }
    }
    finish(values, rejected)
}

/// The text of level `k` (0-based) of a factor's `levels` STRSXP.
fn level_label(levels: SEXP, k: usize) -> String {
    let charsxp = levels.string_elt(isize::try_from(k).expect("level index fits in R_xlen_t"));
    // A rejected label is never `NA_character_` (that parses as `None`).
    unsafe { charsxp_to_str(charsxp) }.to_owned()
}

/// Return the values, or the one batched error naming every rejected element.
fn finish(values: Vec<Option<f64>>, rejected: Rejected) -> Result<Vec<Option<f64>>, SexpError> {
    if rejected.is_empty() {
        Ok(values)
    } else {
        Err(rejected.into_value_error("non-numeric"))
    }
}
// endregion

// region: String parsing (R's `String2Real`)

/// Parse one CHARSXP the way `as.numeric()` does.
fn parse_charsxp(charsxp: SEXP) -> Result<Option<f64>, NotNumeric> {
    if charsxp == SEXP::na_string() {
        return Ok(None);
    }
    // SAFETY: a non-NA CHARSXP; its data is NUL-terminated, which is what
    // `parse_nul_terminated` needs.
    unsafe { parse_nul_terminated(charsxp_to_str(charsxp)) }
}

/// Parse `s` as R's `RealFromString` does, plus the `"NA"` token.
///
/// # Safety
///
/// The byte after `s` must be NUL (R's `CHAR()` data is), because `R_strtod`
/// reads a C string.
unsafe fn parse_nul_terminated(s: &str) -> Result<Option<f64>, NotNumeric> {
    // Also covers `""`, whose `as_ptr()` would not point at R's data.
    if is_blank(s) || is_na_token(s) {
        return Ok(None);
    }
    let start = s.as_ptr().cast::<std::os::raw::c_char>();
    let mut end: *mut std::os::raw::c_char = std::ptr::null_mut();
    // SAFETY: `start` is a NUL-terminated C string (caller contract).
    // `R_strtod` allocates nothing and never signals an R condition.
    let value = unsafe { crate::sys::R_strtod(start, &mut end) };
    // SAFETY: `R_strtod` leaves `end` inside `s` (it stops at or before the NUL).
    let consumed = unsafe { end.cast_const().offset_from(start) };
    let rest_blank = usize::try_from(consumed)
        .ok()
        .and_then(|n| s.as_bytes().get(n..))
        .is_some_and(rest_is_blank);
    if rest_blank {
        Ok(Some(value))
    } else {
        Err(NotNumeric)
    }
}

/// Whether `c` counts as blank for R's `isBlankString`, which uses `iswspace`
/// in a UTF-8 locale: Unicode whitespace except NEL (U+0085) and the no-break
/// spaces U+00A0, U+2007 and U+202F, which neither glibc nor macOS classes as
/// space.
fn is_blank_char(c: char) -> bool {
    c.is_whitespace() && !matches!(c, '\u{0085}' | '\u{00A0}' | '\u{2007}' | '\u{202F}')
}

/// R's `isBlankString`: empty, or only blank characters.
fn is_blank(s: &str) -> bool {
    s.chars().all(is_blank_char)
}

/// The missing-value token `"NA"`, with blanks around it at most.
fn is_na_token(s: &str) -> bool {
    s.trim_matches(is_blank_char) == "NA"
}

/// Whether the bytes `R_strtod` left unread are blank. They follow an ASCII
/// prefix of a valid UTF-8 string, so a failed UTF-8 check only happens for a
/// split multi-byte character, which is not blank either.
fn rest_is_blank(rest: &[u8]) -> bool {
    std::str::from_utf8(rest).is_ok_and(is_blank)
}
// endregion

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_strings_match_r_is_blank_string() {
        for s in [
            "",
            " ",
            "   ",
            "\t\n",
            "\u{0B}\u{0C}\r",
            "\u{3000}",
            "\u{2028}",
            "\u{2003}",
        ] {
            assert!(is_blank(s), "{s:?} should be blank");
        }
        // Neither glibc nor macOS `iswspace` accepts NEL or the no-break spaces.
        for s in [
            "\u{0085}", "\u{00A0}", "\u{2007}", "\u{202F}", "x", " 1 ", "\0",
        ] {
            assert!(!is_blank(s), "{s:?} should not be blank");
        }
    }

    #[test]
    fn na_token_allows_surrounding_blanks_only() {
        for s in ["NA", " NA", "NA ", "\tNA\n"] {
            assert!(is_na_token(s), "{s:?} should be the NA token");
        }
        for s in ["na", "N A", "NAN", "NA1", "<NA>", "\u{00A0}NA"] {
            assert!(!is_na_token(s), "{s:?} should not be the NA token");
        }
    }

    /// `no_na` refuses what the markers read as missing, and `NaN`.
    #[test]
    fn no_na_refuses_none_and_nan() {
        assert!(!AsNumeric(Some(2.5)).__mx_has_na());
        assert!(!AsNumeric(Some(f64::INFINITY)).__mx_has_na());
        assert!(AsNumeric(None).__mx_has_na());
        assert!(AsNumeric(Some(f64::NAN)).__mx_has_na());

        assert!(!AsNumericVec(vec![]).__mx_has_na());
        assert!(!AsNumericVec(vec![Some(1.0), Some(-0.0)]).__mx_has_na());
        assert!(AsNumericVec(vec![Some(1.0), None]).__mx_has_na());
        assert!(AsNumericVec(vec![Some(f64::NAN), Some(1.0)]).__mx_has_na());
    }

    /// The layers a marker comes in forward the check: `NULL` (`None`) and an
    /// omitted argument (`Absent`) pass, a present marker is asked. A type
    /// that is not a marker keeps the default `false`.
    #[test]
    fn no_na_is_forwarded_through_the_layers() {
        use crate::Missing;
        use std::collections::{BTreeMap, HashMap};

        assert!(!None::<AsNumeric>.__mx_has_na());
        assert!(!Some(AsNumeric(Some(1.0))).__mx_has_na());
        assert!(Some(AsNumeric(None)).__mx_has_na());
        assert!(Some(AsNumericVec(vec![None])).__mx_has_na());

        assert!(!Missing::<AsNumeric>::Absent.__mx_has_na());
        assert!(!Missing::Present(AsNumeric(Some(1.0))).__mx_has_na());
        assert!(Missing::Present(AsNumeric(None)).__mx_has_na());
        assert!(!Missing::<Option<AsNumeric>>::Present(None).__mx_has_na());
        assert!(Missing::Present(Some(AsNumeric(None))).__mx_has_na());

        assert!(!Err::<AsNumeric, ()>(()).__mx_has_na());
        assert!(!Ok::<AsNumeric, ()>(AsNumeric(Some(1.0))).__mx_has_na());
        assert!(Ok::<AsNumeric, ()>(AsNumeric(None)).__mx_has_na());

        let mut map = HashMap::new();
        map.insert("a".to_string(), AsNumeric(Some(1.0)));
        assert!(!map.__mx_has_na());
        map.insert("b".to_string(), AsNumeric(None));
        assert!(map.__mx_has_na());
        assert!(Some(map).__mx_has_na());
        assert!(!None::<HashMap<String, AsNumeric>>.__mx_has_na());

        let tree: BTreeMap<String, AsNumericVec> =
            [("a".to_string(), AsNumericVec(vec![Some(1.0), None]))].into();
        assert!(tree.__mx_has_na());

        assert!(!Some(f64::NAN).__mx_has_na());
        assert!(!None::<f64>.__mx_has_na());
        assert!(!Missing::<f64>::Absent.__mx_has_na());
        assert!(!vec![None::<f64>].__mx_has_na());
    }

    #[test]
    fn unread_rest_must_be_blank_utf8() {
        assert!(rest_is_blank(b""));
        assert!(rest_is_blank(b"  \t"));
        assert!(rest_is_blank("\u{3000}".as_bytes()));
        assert!(!rest_is_blank(b" x"));
        assert!(!rest_is_blank(b"\xa0"), "split multi-byte character");
    }
}
