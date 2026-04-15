//! `match.arg`-style enum conversion for R string arguments.
//!
//! This module provides the [`MatchArg`] trait for converting between Rust
//! fieldless enums and R character strings with `match.arg` semantics
//! (exact match or unique partial matching).
//!
//! # Usage
//!
//! ```ignore
//! use miniextendr_api::MatchArg;
//!
//! #[derive(Copy, Clone, MatchArg)]
//! #[match_arg(rename_all = "snake_case")]
//! enum Mode {
//!     Fast,
//!     Safe,
//!     Debug,
//! }
//!
//! #[miniextendr]
//! fn run(#[miniextendr(match_arg)] mode: Mode) -> String {
//!     format!("{mode:?}")
//! }
//! ```
//!
//! The generated R wrapper uses `base::match.arg()` for validation before
//! the main `.Call()`, giving users familiar R error messages and partial
//! matching.

use crate::ffi::{self, SEXP, SEXPTYPE, SexpExt};
use crate::from_r::charsxp_to_str;

/// Trait for enum types that support `match.arg`-style string conversion.
///
/// Implementors provide a fixed set of choice strings and bidirectional
/// conversion between enum variants and their string representations.
///
/// Use `#[derive(MatchArg)]` to auto-generate this implementation.
pub trait MatchArg: Sized + Copy + 'static {
    /// The canonical choice strings, in variant declaration order.
    ///
    /// The first choice is the default when the R argument is `NULL`.
    const CHOICES: &'static [&'static str];

    /// Convert a choice string to the corresponding enum variant.
    ///
    /// Returns `None` if the string doesn't match any choice exactly.
    fn from_choice(choice: &str) -> Option<Self>;

    /// Convert the enum variant to its canonical choice string.
    fn to_choice(self) -> &'static str;
}

/// Error type for `MatchArg` conversion failures.
#[derive(Debug, Clone)]
pub enum MatchArgError {
    /// The SEXP was not a character or factor type.
    InvalidType(SEXPTYPE),
    /// The input had length != 1.
    InvalidLength(usize),
    /// The input was NA.
    IsNa,
    /// No choice matched the input.
    NoMatch {
        /// The input string that didn't match.
        input: String,
        /// The valid choices.
        choices: &'static [&'static str],
    },
}

impl std::fmt::Display for MatchArgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MatchArgError::InvalidType(ty) => {
                write!(f, "match.arg: expected character or factor, got {:?}", ty)
            }
            MatchArgError::InvalidLength(len) => {
                write!(f, "match.arg: expected length 1, got {}", len)
            }
            MatchArgError::IsNa => write!(f, "match.arg: input is NA"),
            MatchArgError::NoMatch { input, choices } => {
                write!(
                    f,
                    "'arg' should be one of {}, got {:?}",
                    choices
                        .iter()
                        .map(|c| format!("{:?}", c))
                        .collect::<Vec<_>>()
                        .join(", "),
                    input,
                )
            }
        }
    }
}

impl std::error::Error for MatchArgError {}

impl From<MatchArgError> for crate::from_r::SexpError {
    fn from(e: MatchArgError) -> Self {
        crate::from_r::SexpError::InvalidValue(e.to_string())
    }
}

/// Build an R character vector (STRSXP) from the choices of a `MatchArg` type.
///
/// This is called by generated choices-helper C wrappers to provide the
/// choice list to `base::match.arg()` in the R wrapper.
pub fn choices_sexp<T: MatchArg>() -> SEXP {
    let choices = <T as MatchArg>::CHOICES;
    unsafe {
        let n = choices.len();
        let vec = ffi::Rf_allocVector(SEXPTYPE::STRSXP, n as ffi::R_xlen_t);
        ffi::Rf_protect(vec);
        for (i, s) in choices.iter().enumerate() {
            let charsxp = if s.is_empty() {
                SEXP::blank_string()
            } else {
                SEXP::charsxp(s)
            };
            vec.set_string_elt(i as ffi::R_xlen_t, charsxp);
        }
        ffi::Rf_unprotect(1);
        vec
    }
}

/// Extract a single string from an R SEXP and match it against a `MatchArg` type.
///
/// Used by the generated `TryFromSexp for T` implementation (single-value `match.arg`).
pub fn match_arg_from_sexp<T: MatchArg>(sexp: SEXP) -> Result<T, MatchArgError> {
    let actual_type = sexp.type_of();

    // Extract the string value
    let input = match actual_type {
        SEXPTYPE::STRSXP => {
            let len = sexp.len();
            if len != 1 {
                return Err(MatchArgError::InvalidLength(len));
            }
            let charsxp = sexp.string_elt(0);
            if charsxp == SEXP::na_string() {
                return Err(MatchArgError::IsNa);
            }
            // UTF-8 locale asserted at package init — R_CHAR is safe
            unsafe { charsxp_to_str(charsxp) }
        }
        // Accept factors: extract the level label
        SEXPTYPE::INTSXP => {
            // Check if it's a factor (has "levels" attribute)
            let levels = sexp.get_levels();
            if levels.is_nil() || levels.type_of() != SEXPTYPE::STRSXP {
                return Err(MatchArgError::InvalidType(actual_type));
            }
            let len = sexp.len();
            if len != 1 {
                return Err(MatchArgError::InvalidLength(len));
            }
            let idx = unsafe { *ffi::INTEGER(sexp) };
            if idx == i32::MIN {
                // NA_integer_
                return Err(MatchArgError::IsNa);
            }
            // R factor indices are 1-based
            let level_idx = (idx - 1) as ffi::R_xlen_t;
            if level_idx < 0 || level_idx >= levels.len() as ffi::R_xlen_t {
                return Err(MatchArgError::NoMatch {
                    input: format!("<factor index {}>", idx),
                    choices: <T as MatchArg>::CHOICES,
                });
            }
            let charsxp = levels.string_elt(level_idx);
            unsafe { charsxp_to_str(charsxp) }
        }
        SEXPTYPE::NILSXP => {
            // NULL → use first choice (match.arg default behavior)
            return T::from_choice(<T as MatchArg>::CHOICES[0]).ok_or_else(|| {
                MatchArgError::NoMatch {
                    input: String::new(),
                    choices: <T as MatchArg>::CHOICES,
                }
            });
        }
        _ => return Err(MatchArgError::InvalidType(actual_type)),
    };

    match_choice::<T>(input)
}

/// Match a string against the choices of a `MatchArg` type (exact or partial).
fn match_choice<T: MatchArg>(input: &str) -> Result<T, MatchArgError> {
    // Exact match
    if let Some(val) = T::from_choice(input) {
        return Ok(val);
    }

    // Unique partial match (like R's match.arg)
    let mut matches: Vec<(usize, &'static str)> = Vec::new();
    for (i, choice) in <T as MatchArg>::CHOICES.iter().enumerate() {
        if choice.starts_with(input) {
            matches.push((i, choice));
        }
    }

    match matches.len() {
        1 => T::from_choice(matches[0].1).ok_or(MatchArgError::NoMatch {
            input: input.to_string(),
            choices: <T as MatchArg>::CHOICES,
        }),
        _ => Err(MatchArgError::NoMatch {
            input: input.to_string(),
            choices: <T as MatchArg>::CHOICES,
        }),
    }
}

/// Extract multiple strings from an R SEXP (STRSXP) and match each against
/// the choices of a `MatchArg` type.
///
/// Used by the generated C wrapper for `match_arg + several_ok` parameters
/// (`match.arg` with `several.ok = TRUE`).
///
/// NULL input returns all variants (matching R's `match.arg` default with `several.ok = TRUE`).
///
/// Note: factors (INTSXP) are not handled here — the R wrapper coerces factors
/// to character before the `.Call()` boundary.
pub fn match_arg_vec_from_sexp<T: MatchArg>(sexp: SEXP) -> Result<Vec<T>, MatchArgError> {
    let actual_type = sexp.type_of();

    match actual_type {
        SEXPTYPE::STRSXP => {
            let len = sexp.len();
            let mut result = Vec::with_capacity(len);
            for i in 0..len {
                let charsxp = sexp.string_elt(i as ffi::R_xlen_t);
                if charsxp == SEXP::na_string() {
                    return Err(MatchArgError::IsNa);
                }
                let s = unsafe { charsxp_to_str(charsxp) };
                result.push(match_choice::<T>(s)?);
            }
            Ok(result)
        }
        SEXPTYPE::NILSXP => {
            // NULL → all choices (match.arg default with several.ok = TRUE)
            <T as MatchArg>::CHOICES
                .iter()
                .map(|c| match_choice::<T>(c))
                .collect()
        }
        _ => Err(MatchArgError::InvalidType(actual_type)),
    }
}
