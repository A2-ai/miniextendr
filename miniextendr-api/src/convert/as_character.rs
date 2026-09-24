//! [`AsCharacter`] / [`AsCharacterVec`]: a label argument read the way R's
//! `as.character()` reads it, from any atomic vector, a factor by its labels.
//! The reading rules are documented on [`AsCharacterVec`].

use crate::expression::{RCall, RSymbol};
use crate::from_r::{SexpError, SexpLengthError, SexpTypeError, TryFromSexp};
use crate::gc_protect::OwnedProtect;
use crate::impl_option_try_from_sexp;
use crate::{SEXP, SEXPTYPE, SexpExt};

// region: Marker types

/// A character scalar read like R's `as.character()`: from an atomic vector
/// of length 1 of any type, a factor by its label. `NA` of any type is `None`.
///
/// It follows the same reading rules as [`AsCharacterVec`], and also requires
/// length 1 ([`SexpError::Length`] otherwise), both of the argument and of what
/// a class's `as.character()` method returns for it.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::{miniextendr, AsCharacter};
///
/// #[miniextendr]
/// fn subject_label(id: AsCharacter) -> String {
///     format!("subject {}", id.0.as_deref().unwrap_or("unknown"))
/// }
/// // R: subject_label("S-01")        → "subject S-01"
/// //    subject_label(101L)          → "subject 101"
/// //    subject_label(factor("S-02")) → "subject S-02"   (the label, not the code)
/// //    subject_label(NA)            → "subject unknown"
/// //    subject_label(c(1, 2))       → error: 'id' must have length 1
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsCharacter(pub Option<String>);

/// A character vector read like R's `as.character()`: from an atomic vector of
/// any type, a factor by its labels. `NA` of any type is `None`.
///
/// Identifiers in R data often arrive as numbers or factors: subject IDs,
/// visit numbers, or grouping columns read by `read.csv()`. A `Vec<String>`
/// parameter accepts character input only, so a function that treats such
/// values as labels would need an R wrapper calling `as.character()` first.
/// This marker does that conversion itself, and the strings are exactly the
/// ones R produces elsewhere, because R makes them.
///
/// # Reading rules
///
/// | R input | Result |
/// |---------|--------|
/// | character | as is; `NA_character_` → `None`. `"NA"` and `""` stay values |
/// | integer, double | R's own formatting (doubles to 15 significant digits): `0.1 + 0.2` → `"0.3"`, `1e6` → `"1e+06"`, `100` → `"100"`, `NaN` → `"NaN"`, `Inf` → `"Inf"`; `NA` → `None` |
/// | logical | `"TRUE"` / `"FALSE"`; `NA` → `None` |
/// | complex | `"1+2i"`; `NA` → `None` |
/// | raw | two hex digits: `as.raw(255)` → `"ff"` |
/// | classed object (factor, `Date`, `POSIXct`, …) | its `as.character()` method: factor labels (never the codes), `"2024-01-15"` for a `Date`, `"2024-01-15 10:30:00"` for a `POSIXct` |
/// | list, data frame, `NULL`, … | [`SexpError::Type`] |
///
/// A plain atomic vector is converted with R's `coerceVector()` (what the
/// `as.character()` primitive does for it); the text never comes from Rust
/// formatting, which would give `"0.30000000000000004"` for `0.1 + 0.2`. A
/// vector with a class attribute (`is.object(x)`) is passed to
/// `as.character(x)`, evaluated as if at top level: base R's `as.character`
/// (a user binding of that name does not interfere), with S3 and S4 methods
/// registered by packages or defined in the global environment. A `Date` is a
/// double underneath, and `coerceVector()` alone would give its day count
/// (`"19737"`). An error in the method becomes the conversion error, as does a
/// method that returns something other than a character vector.
///
/// Names and `dim` are dropped, as `as.character()` drops them: a matrix is
/// read in column-major order. Zero-length input gives an empty vector.
///
/// Unlike [`AsNumericVec`](crate::convert::AsNumericVec), which reads the
/// token `"NA"` and blank strings as missing, only `NA` itself is `None`
/// here: `as.character()` keeps `"NA"` and `""` as strings. `NaN` becomes
/// `"NaN"` (`no_na` refuses it in R, as `anyNA()` counts it).
///
/// `Option<AsCharacterVec>` (and `Option<AsCharacter>`) also accept `NULL` as
/// `None`. The markers are input-only: there is no `IntoR`. Return the inner
/// `Vec<Option<String>>` / `Option<String>`, which already converts to a
/// character vector with `NA`.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::{miniextendr, AsCharacterVec};
/// use std::collections::HashSet;
///
/// #[miniextendr]
/// fn n_subjects(ids: AsCharacterVec) -> i32 {
///     let distinct: HashSet<String> = ids.0.into_iter().flatten().collect();
///     i32::try_from(distinct.len()).expect("fewer than 2^31 subjects")
/// }
/// // R: n_subjects(c(101L, 102L, 101L))         → 2
/// //    n_subjects(factor(c("S1", "S2", NA)))    → 2   (labels, not codes)
/// //    n_subjects(c(0.1 + 0.2, 0.3))            → 1   (both read as "0.3")
/// //    n_subjects(list(1))                      → error: 'ids' must be atomic
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AsCharacterVec(pub Vec<Option<String>>);

impl TryFromSexp for AsCharacterVec {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        read_character(sexp).map(AsCharacterVec)
    }
}

impl TryFromSexp for AsCharacter {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        // The type is checked before the length, so a length-1 list reports
        // its type rather than passing the length check.
        check_type(sexp.type_of())?;
        check_length_one(sexp.len())?;
        let mut values = read_character(sexp)?;
        // A class's `as.character()` method may return another length.
        check_length_one(values.len())?;
        Ok(AsCharacter(values.swap_remove(0)))
    }
}

impl_option_try_from_sexp!(AsCharacter);
impl_option_try_from_sexp!(AsCharacterVec);
// endregion

// region: SEXP dispatch

/// Refuse every SEXPTYPE that is not an atomic vector. `STRSXP` is named as
/// the expected type because that is what the value becomes.
fn check_type(actual: SEXPTYPE) -> Result<(), SexpError> {
    match actual {
        SEXPTYPE::LGLSXP
        | SEXPTYPE::INTSXP
        | SEXPTYPE::REALSXP
        | SEXPTYPE::CPLXSXP
        | SEXPTYPE::STRSXP
        | SEXPTYPE::RAWSXP => Ok(()),
        _ => Err(SexpTypeError {
            expected: SEXPTYPE::STRSXP,
            actual,
        }
        .into()),
    }
}

fn check_length_one(actual: usize) -> Result<(), SexpError> {
    if actual == 1 {
        Ok(())
    } else {
        Err(SexpLengthError {
            expected: 1,
            actual,
        }
        .into())
    }
}

/// Read any atomic vector into `Vec<Option<String>>` through R's own
/// conversion to character.
fn read_character(sexp: SEXP) -> Result<Vec<Option<String>>, SexpError> {
    check_type(sexp.type_of())?;
    let strings = if sexp.is_object() {
        dispatch_as_character(sexp)?
    } else {
        // `coerceVector()` returns a character vector as is. For an integer or
        // double vector without attributes it returns a deferred-string ALTREP
        // whose elements are allocated as they are read, so the result stays
        // protected until every element has been copied out.
        // SAFETY: argument conversion runs on R's main thread.
        unsafe { OwnedProtect::new(sexp.coerce(SEXPTYPE::STRSXP)) }
    };
    Vec::<Option<String>>::try_from_sexp(strings.get())
}

/// Evaluate `as.character(x)` for a classed `x`, so its method decides the
/// text (factor labels, formatted dates, ...).
///
/// The call's head is the function bound in the base environment, and it is
/// evaluated in the global environment: R finds the methods a top-level
/// `as.character(x)` would find, and a global binding named `as.character`
/// does not replace the function. `R_tryEvalSilent` (inside [`RCall::eval`])
/// turns an R error into `Err`, so a failing method never unwinds through
/// Rust frames.
fn dispatch_as_character(sexp: SEXP) -> Result<OwnedProtect, SexpError> {
    // SAFETY: argument conversion runs on R's main thread, and `as.character`
    // is always bound in the base environment, so `Rf_findFun` cannot fail.
    unsafe {
        let fun = crate::sys::Rf_findFun(
            RSymbol::from_cstr(c"as.character").as_sexp(),
            crate::sys::R_BaseEnv,
        );
        let call = RCall::from_sexp(fun).arg(sexp);
        let result = call
            .eval(crate::sys::R_GlobalEnv)
            .map_err(|msg| SexpError::InvalidValue(format!("as.character() failed: {msg}")))?;
        // `eval` returns the value unprotected: root it before anything else
        // allocates.
        let result = OwnedProtect::new(result);
        let actual = result.get().type_of();
        if actual != SEXPTYPE::STRSXP {
            return Err(SexpError::InvalidValue(format!(
                "as.character() returned {}, not a character vector",
                crate::typed_list::sexptype_name(actual)
            )));
        }
        Ok(result)
    }
}
// endregion
