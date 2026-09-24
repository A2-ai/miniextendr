//! Classed `Result` errors, class vectors and the reserved condition-data
//! names (#1434, #1435, #1440).
//!
//! - `RConditionError` on a package error enum: every `Err` raised from a
//!   `Result<T, PkgError>` return carries the member + family classes and the
//!   variant's fields as `e$<name>`, while the Rust side keeps `?` composition.
//! - `RError`: the ready-made classed value, built from a `std::error::Error`
//!   with `?` / `From` and decorated with `.class(..)` / `.data(..)`.
//! - `class = [..]` vectors on `rust_error!` / `warning!` / `rust_condition!`.
//! - Data fields named `kind` / `message` / `call` are rejected: a computed
//!   reserved name fails at runtime (literal names fail to compile, see the
//!   `compile_fail` doctest on
//!   `miniextendr_api::condition::is_reserved_condition_field`).

use miniextendr_api::condition::{ConditionData, RConditionError, RError};
use miniextendr_api::{RValue, miniextendr, rust_condition, rust_error, warning};

// region: RConditionError on a package error enum

/// Stand-in for a downstream package's thiserror-style error enum.
#[derive(Debug)]
pub enum PkgError {
    MissingField {
        field: String,
    },
    OutOfRange {
        value: f64,
        max: f64,
    },
    /// A variant whose data carries an explicitly absent value: `e$optional`
    /// must exist in `names(e)` and be `NULL`, next to a present field.
    NoValue {
        present: f64,
    },
}

impl std::fmt::Display for PkgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PkgError::MissingField { field } => write!(f, "field `{field}` is missing"),
            PkgError::OutOfRange { value, max } => write!(f, "{value} exceeds the maximum {max}"),
            PkgError::NoValue { present } => write!(f, "no value alongside {present}"),
        }
    }
}

impl std::error::Error for PkgError {}

impl RConditionError for PkgError {
    fn message(&self) -> String {
        self.to_string()
    }

    fn class(&self) -> Vec<String> {
        let member = match self {
            PkgError::MissingField { .. } => "pkg_error_missing_field",
            PkgError::OutOfRange { .. } => "pkg_error_out_of_range",
            PkgError::NoValue { .. } => "pkg_error_no_value",
        };
        vec![member.to_string(), "pkg_error".to_string()]
    }

    fn data(&self) -> Option<ConditionData> {
        Some(match self {
            PkgError::MissingField { field } => vec![("field".to_string(), field.as_str().into())],
            PkgError::OutOfRange { value, max } => vec![
                ("value".to_string(), (*value).into()),
                ("max".to_string(), (*max).into()),
            ],
            PkgError::NoValue { present } => vec![
                ("optional".to_string(), RValue::Null),
                ("present".to_string(), (*present).into()),
            ],
        })
    }
}

fn require_field(field: &str) -> Result<i32, PkgError> {
    if field.is_empty() {
        Err(PkgError::MissingField {
            field: "id".to_string(),
        })
    } else {
        Ok(field.len() as i32)
    }
}

/// `Result<i32, PkgError>`: empty `field` raises `pkg_error_missing_field`
/// with `e$field == "id"`; otherwise returns the length.
#[miniextendr]
pub fn classed_result_missing(field: &str) -> Result<i32, PkgError> {
    let n = require_field(field)?;
    Ok(n)
}

/// `Result<f64, PkgError>`: values above 100 raise `pkg_error_out_of_range`
/// with `e$value` and `e$max`.
#[miniextendr]
pub fn classed_result_range(value: f64) -> Result<f64, PkgError> {
    if value > 100.0 {
        return Err(PkgError::OutOfRange { value, max: 100.0 });
    }
    Ok(value)
}

/// Unit-returning `Result<(), PkgError>` (a different codegen arm).
#[miniextendr]
pub fn classed_result_unit(value: f64) -> Result<(), PkgError> {
    classed_result_range(value).map(|_| ())
}

/// Always raises `pkg_error_no_value` whose data has an explicit NULL field:
/// `"optional" %in% names(e)` is `TRUE`, `e$optional` is `NULL`, `e$present`
/// is `1`. Guards the `keep.null = TRUE` in the shared raise helper.
#[miniextendr]
pub fn classed_result_null_field() -> Result<(), PkgError> {
    Err(PkgError::NoValue { present: 1.0 })
}

/// The same error type raised from an S3 method (impl-block codegen arm).
#[derive(miniextendr_api::ExternalPtr)]
pub struct ClassedChecker {
    max: f64,
}

#[miniextendr(s3)]
impl ClassedChecker {
    /// A checker with an upper bound.
    /// @param max Upper bound.
    pub fn new(max: f64) -> Self {
        ClassedChecker { max }
    }

    /// Check `value` against the bound; raises `pkg_error_out_of_range`.
    /// @param value Value to check.
    pub fn check_bound(&self, value: f64) -> Result<f64, PkgError> {
        if value > self.max {
            return Err(PkgError::OutOfRange {
                value,
                max: self.max,
            });
        }
        Ok(value)
    }
}

// endregion

// region: RError

/// `Result<i32, RError>` built from a `ParseIntError` via `From`, then classed.
#[miniextendr]
pub fn rerror_parse(s: &str) -> Result<i32, RError> {
    let n: i32 = s.parse().map_err(|e| {
        RError::from(e)
            .class(["pkg_bad_number", "pkg_error"])
            .data("input", s)
    })?;
    Ok(n)
}

/// `RError` whose field name is computed at runtime and reserved: raises a
/// plain `rust_error` about the reserved name instead of overwriting `e$kind`.
#[miniextendr]
pub fn rerror_reserved_runtime(name: &str) -> Result<i32, RError> {
    Err(RError::new("reserved field").data(name, 1))
}

/// Classless `RError`: same rendering as a plain `Result<_, String>` error.
#[miniextendr]
pub fn rerror_plain() -> Result<i32, RError> {
    Err(RError::new("plain rerror"))
}

// endregion

// region: class vectors on the macros

/// `rust_error!(class = [member, family], ...)`.
#[miniextendr]
pub fn classed_error_vec(member: &str, family: &str) {
    rust_error!(class = [member, family], "layered error");
}

/// `warning!` with a two-element class vector and data. Like every condition
/// macro it unwinds, so nothing follows it.
#[miniextendr]
pub fn classed_warning_vec(n: i32) {
    warning!(
        class = ["pkg_warning_dropped", "pkg_warning"],
        data = ("dropped", n),
        "dropped {n} rows"
    );
}

/// `rust_condition!` with a class vector supplied as a `Vec<String>`.
#[miniextendr]
pub fn classed_condition_vec(classes: Vec<String>) {
    rust_condition!(class = classes, "signalled");
}

/// Computed reserved field name: the runtime reserved-name check fires.
#[miniextendr]
pub fn reserved_data_macro_runtime(name: &str) {
    rust_error!(data = (name, 1), "should not reach R with this field");
}

// endregion

// region: classed argument-conversion errors

/// Named, non-negative hyperparameters from an R named numeric vector,
/// `c(alpha = 1, beta = 0.5)`. Its `TryFromSexp::Error` implements
/// `RConditionError` (via the derive), so an argument that fails to convert
/// raises the error's classes and fields, with the parameter context in the
/// message and the parameter's R name as `e$param`.
pub struct Hyperparams(std::collections::BTreeMap<String, f64>);

/// Why an argument is not a valid [`Hyperparams`].
#[derive(Debug, RConditionError)]
#[condition(class = "mx_fixture_error")]
pub enum HyperparamError {
    /// Not a named numeric vector. The member class is the
    /// `mx_fixture_bad_arg` of every argument this fixture family rejects.
    #[condition(
        class = "mx_fixture_bad_arg",
        message = "expected a named numeric vector ({reason})"
    )]
    NotNamedNumeric {
        /// The underlying conversion error.
        reason: String,
    },
    /// A negative entry. Its data has a `param` field of its own (the
    /// hyperparameter's name), which wins over the framework's `e$param`.
    #[condition(
        class = "mx_fixture_negative_hyperparam",
        message = "hyperparameter '{param}' must be non-negative, got {value}"
    )]
    Negative {
        /// The hyperparameter's name.
        param: String,
        /// Its value.
        value: f64,
    },
}

impl miniextendr_api::TryFromSexp for Hyperparams {
    type Error = HyperparamError;

    fn try_from_sexp(sexp: miniextendr_api::SEXP) -> Result<Self, Self::Error> {
        use miniextendr_api::named_vector::NamedVector;
        let NamedVector(map) =
            NamedVector::<std::collections::BTreeMap<String, f64>>::try_from_sexp(sexp).map_err(
                |e| HyperparamError::NotNamedNumeric {
                    reason: e.to_string(),
                },
            )?;
        if let Some((param, value)) = map.iter().find(|(_, value)| **value < 0.0) {
            return Err(HyperparamError::Negative {
                param: param.clone(),
                value: *value,
            });
        }
        Ok(Hyperparams(map))
    }
}

/// Sum of the hyperparameters; a bad `hyper` raises a classed conversion
/// error (`mx_fixture_bad_arg` / `mx_fixture_negative_hyperparam`, then
/// `mx_fixture_error`).
///
/// @param hyper A named numeric vector of non-negative values.
#[miniextendr]
pub fn hyperparams_total(hyper: Hyperparams) -> f64 {
    hyper.0.values().sum()
}

#[cfg(feature = "worker-thread")]
mod worker {
    use super::Hyperparams;
    use miniextendr_api::miniextendr;

    /// `hyperparams_total()` on the worker thread: the argument converts on
    /// the main thread before dispatch, through the split conversion path.
    ///
    /// @param hyper A named numeric vector of non-negative values.
    #[miniextendr(worker)]
    pub fn hyperparams_total_worker(hyper: Hyperparams) -> f64 {
        hyper.0.values().sum()
    }
}

/// Internal entry point behind the hand-written `hyperparams_total_caller()`
/// in `R/call_attribution.R`: a conversion error is attributed to that
/// caller's call.
///
/// @param hyper A named numeric vector of non-negative values.
/// @noRd
#[miniextendr(noexport, call = caller)]
pub fn hyperparams_total_caller_impl(hyper: Hyperparams) -> f64 {
    hyper.0.values().sum()
}

// endregion
