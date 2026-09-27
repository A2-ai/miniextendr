//! Per-parameter `inherits` / `no_na` checks, driven by
//! `tests/testthat/test-param-checks.R`.
//!
//! Standalone fns spell them on the parameter
//! (`#[miniextendr(inherits = "cls", no_na)]`); impl and trait methods on the
//! method (`#[miniextendr(inherits(x = "cls"), no_na(y))]`). Both land in the
//! generated precondition guards after the type checks and survive
//! `no_preconditions` / `fast`. Either spelling takes an optional
//! `message = "..."`, the condition message of a failure, used verbatim
//! (`inherits(class = "cls", message = "...")`, `no_na(message = "...")`,
//! method level `inherits(x(class = "cls", message = "..."))`).
//!
//! On a reading marker (`AsNumeric*`, `AsCharacter*`, and aliases or derived
//! newtypes of them) `no_na` also checks the converted value, for what the
//! marker reads as `NA` beyond `anyNA()`.

use std::collections::HashMap;

use miniextendr_api::{
    AsCharacter, AsNumeric, AsNumericVec, List, Missing, SEXP, TryFromSexp, miniextendr,
};

// region: standalone fns

/// `inherits = "cls"`: the argument must inherit from the class.
/// @param x An object of class `mx_obj`.
#[miniextendr(noexport)]
pub fn param_inherits_one(#[miniextendr(inherits = "mx_obj")] x: List) -> i32 {
    i32::try_from(x.len()).expect("list length fits i32")
}

/// `inherits("a", "b")`: any of the classes.
/// @param x An object of class `mx_a` or `mx_b`.
#[miniextendr(noexport)]
pub fn param_inherits_any(#[miniextendr(inherits("mx_a", "mx_b"))] x: SEXP) -> bool {
    let _ = x;
    true
}

/// `inherits` on an `Option<List>`: `NULL` passes.
/// @param x `NULL` or an object of class `mx_obj`.
#[miniextendr(noexport)]
pub fn param_inherits_optional(
    #[miniextendr(inherits = "mx_obj", default = "NULL")] x: Option<List>,
) -> bool {
    x.is_some()
}

/// `no_na` on a double scalar: `NA_real_` and `NaN` are refused.
/// @param x A non-NA double.
#[miniextendr(noexport)]
pub fn param_no_na_scalar(#[miniextendr(no_na)] x: f64) -> f64 {
    x
}

/// `no_na` on a double vector.
/// @param x A double vector without NA.
#[miniextendr(noexport)]
pub fn param_no_na_vec(#[miniextendr(no_na)] x: Vec<f64>) -> f64 {
    x.iter().sum()
}

/// `no_na` on a `Missing<f64>`: an omitted argument passes.
/// @param x A non-NA double, or omitted.
#[miniextendr(noexport)]
pub fn param_no_na_missing(#[miniextendr(no_na)] x: Missing<f64>) -> bool {
    x.is_present()
}

/// Both checks on one parameter, split over two attributes.
/// @param x A classed double without NA.
#[miniextendr(noexport)]
pub fn param_checks_both(
    #[miniextendr(no_na)]
    #[miniextendr(inherits = "mx_num")]
    x: Vec<f64>,
) -> f64 {
    x.iter().sum()
}

/// `fast` drops the type checks but keeps `no_na`: `"a"` fails in Rust,
/// `NA_real_` in R.
/// @param x A non-NA double.
#[miniextendr(noexport, fast)]
pub fn param_no_na_fast(#[miniextendr(no_na)] x: f64) -> f64 {
    x
}

/// Under `call = caller` the checks are attributed to the calling function
/// (`R/param_checks.R`, `param_checks_caller()`).
/// @param x An object of class `mx_obj`.
/// @param y A non-NA double.
/// @noRd
#[miniextendr(noexport, call = caller)]
pub fn param_checks_caller_impl(
    #[miniextendr(inherits = "mx_obj")] x: List,
    #[miniextendr(no_na)] y: f64,
) -> f64 {
    let _ = x;
    y
}
// endregion

// region: custom messages

/// Two classes, the generated message: the baseline for
/// `param_model_custom`, which differs only in its message.
/// @param model An `mx_model` or `mx_model2` object.
#[miniextendr(noexport)]
pub fn param_model_default(#[miniextendr(inherits("mx_model", "mx_model2"))] model: List) -> i32 {
    i32::try_from(model.len()).expect("list length fits i32")
}

/// Two classes, one message for both, used verbatim.
/// @param model An `mx_model` or `mx_model2` object.
#[miniextendr(noexport)]
pub fn param_model_custom(
    #[miniextendr(inherits(
        "mx_model",
        "mx_model2",
        message = "`model` must be an `mx_model` object; create one with `mx_model()`."
    ))]
    model: List,
) -> i32 {
    i32::try_from(model.len()).expect("list length fits i32")
}

/// The keyed spelling `inherits(class = "cls", message = "...")`.
/// @param model An `mx_model` object.
#[miniextendr(noexport)]
pub fn param_model_class_key(
    #[miniextendr(inherits(class = "mx_model", message = "need an mx_model"))] model: List,
) -> i32 {
    i32::try_from(model.len()).expect("list length fits i32")
}

/// `no_na(message = "...")`, with quotes, backticks, a backslash, `%`, a
/// newline and a non-ASCII character, all of which reach R unchanged.
/// @param x A non-NA double.
#[miniextendr(noexport)]
pub fn param_no_na_custom(
    #[miniextendr(no_na(
        message = "`x` can't be NA: it's \"required\" \\ 100% sure\nsee caf\u{e9}()"
    ))]
    x: f64,
) -> f64 {
    x
}

/// `fast` keeps a check with a message, like any named check.
/// @param x A non-NA double.
#[miniextendr(noexport, fast)]
pub fn param_no_na_custom_fast(#[miniextendr(no_na(message = "no NA here"))] x: f64) -> f64 {
    x
}

/// `param_checks_caller_impl` with messages: under `call = caller` a custom
/// message keeps the caller's call (`R/call_attribution.R`,
/// `param_checks_caller_msg()`).
/// @param x An object of class `mx_obj`.
/// @param y A non-NA double.
/// @noRd
#[miniextendr(noexport, call = caller)]
pub fn param_checks_caller_msg_impl(
    #[miniextendr(inherits(class = "mx_obj", message = "`x` must be an `mx_obj`"))] x: List,
    #[miniextendr(no_na(message = "`y` must not be NA"))] y: f64,
) -> f64 {
    let _ = x;
    y
}
// endregion

// region: no_na on the reading markers
//
// `AsNumeric*` read the text `"NA"`, blank strings and `"NaN"` as missing
// (`NaN` as a value), `AsCharacter*` a factor `NA` level or an `NA` from a
// class's `as.character()`: inputs R's `anyNA()` passes. The C wrapper checks
// the converted value too and raises the R guard's condition, so every body
// below sees present, non-`NaN` values only.

/// A number read like `as.numeric()`, without `NA`.
/// @param x A number, string or factor label; not `NA`, `"NA"`, blank or `"NaN"`.
#[miniextendr(noexport)]
pub fn param_no_na_number(#[miniextendr(no_na)] x: AsNumeric) -> f64 {
    x.0.expect("no_na refuses a missing value")
}

/// Numbers read like `as.numeric()`: the count of missing values, always 0.
/// @param x Numbers, strings or factor labels, none of them missing.
#[miniextendr(noexport)]
pub fn param_no_na_numbers(#[miniextendr(no_na)] x: AsNumericVec) -> i32 {
    count_refused(&x.0)
}

/// `Option<AsNumeric>`: `NULL` is "not given" and passes as `None`.
/// @param x `NULL`, or a number that is not missing.
#[miniextendr(noexport)]
pub fn param_no_na_number_opt(
    #[miniextendr(no_na, default = "NULL")] x: Option<AsNumeric>,
) -> String {
    describe(x.map(|v| v.0))
}

/// `Missing<AsNumeric>`: an omitted argument passes.
/// @param x A number that is not missing, or omitted.
#[miniextendr(noexport)]
pub fn param_no_na_number_missing(#[miniextendr(no_na)] x: Missing<AsNumeric>) -> String {
    match x {
        Missing::Absent => "absent".to_string(),
        Missing::Present(v) => describe(Some(v.0)),
    }
}

/// `fast` drops the type checks; the `no_na` guard and the Rust check stay.
/// @param x Numbers, strings or factor labels, none of them missing.
#[miniextendr(noexport, fast)]
pub fn param_no_na_numbers_fast(#[miniextendr(no_na)] x: AsNumericVec) -> i32 {
    count_refused(&x.0)
}

/// A label read like `as.character()`, without `NA`.
/// @param x An atomic value or factor of length 1; not `NA`.
#[miniextendr(noexport)]
pub fn param_no_na_label(#[miniextendr(no_na)] x: AsCharacter) -> String {
    x.0.expect("no_na refuses a missing label")
}

/// The message of `param_no_na_custom`, on a marker: the Rust check passes
/// the same text verbatim.
/// @param dose A number that is not missing.
#[miniextendr(noexport)]
pub fn param_no_na_number_custom(
    #[miniextendr(no_na(
        message = "`x` can't be NA: it's \"required\" \\ 100% sure\nsee caf\u{e9}()"
    ))]
    dose: AsNumeric,
) -> f64 {
    dose.0.expect("no_na refuses a missing value")
}

/// On the worker path the check runs on the main thread, before dispatch.
/// @param x Numbers, strings or factor labels, none of them missing.
#[cfg(feature = "worker-thread")]
#[miniextendr(worker, noexport)]
pub fn param_no_na_numbers_worker(#[miniextendr(no_na)] x: AsNumericVec) -> i32 {
    count_refused(&x.0)
}

/// Under `call = caller` the Rust check names the caller, like the R guard
/// (`R/call_attribution.R`, `param_no_na_number_caller()`).
/// @param x A number that is not missing.
/// @noRd
#[miniextendr(noexport, call = caller)]
pub fn param_no_na_number_caller_impl(#[miniextendr(no_na)] x: AsNumeric) -> f64 {
    x.0.expect("no_na refuses a missing value")
}

/// A type alias of a marker: the check follows the type, not its name.
pub type Dose = AsNumeric;

/// `no_na` on an alias of `AsNumeric`.
/// @param x A number that is not missing.
#[miniextendr(noexport)]
pub fn param_no_na_alias(#[miniextendr(no_na)] x: Dose) -> f64 {
    x.0.expect("no_na refuses a missing value")
}

/// A `#[derive(TryFromSexp)]` newtype of a marker, which forwards the check.
#[derive(TryFromSexp)]
pub struct DoseNt(AsNumeric);

/// `no_na` on a derived newtype of `AsNumeric`.
/// @param x A number that is not missing.
#[miniextendr(noexport)]
pub fn param_no_na_newtype(#[miniextendr(no_na)] x: DoseNt) -> f64 {
    x.0.0.expect("no_na refuses a missing value")
}

/// `Option<DoseNt>` (the newtype container blanket): `NULL` passes.
/// @param x `NULL`, or a number that is not missing.
#[miniextendr(noexport)]
pub fn param_no_na_newtype_opt(
    #[miniextendr(no_na, default = "NULL")] x: Option<DoseNt>,
) -> String {
    describe(x.map(|v| v.0.0))
}

/// A map of markers: each top-level value is read by the marker.
/// @param x A named list of numbers, none of them missing.
#[miniextendr(noexport)]
pub fn param_no_na_map(#[miniextendr(no_na)] x: HashMap<String, AsNumeric>) -> i32 {
    let values: Vec<Option<f64>> = x.into_values().map(|v| v.0).collect();
    count_refused(&values)
}

/// `Result<AsNumeric, ()>`: `NULL` (`Err(())`) passes, a value is checked.
/// @param x `NULL`, or a number that is not missing.
#[miniextendr(noexport)]
pub fn param_no_na_number_result(
    #[miniextendr(no_na, default = "NULL")] x: Result<AsNumeric, ()>,
) -> String {
    describe(x.ok().map(|v| v.0))
}

/// `Either<AsNumericVec, Vec<u8>>`: the side the value converted to is
/// checked. Returns the count of missing numbers (always 0), or -1 for raw.
/// @param x Numbers, strings or factor labels, none of them missing, or a raw
///   vector.
#[cfg(feature = "either")]
#[miniextendr(noexport)]
pub fn param_no_na_numbers_either(
    #[miniextendr(no_na)] x: miniextendr_api::either_impl::Either<AsNumericVec, Vec<u8>>,
) -> i32 {
    match x {
        miniextendr_api::either_impl::Either::Left(v) => count_refused(&v.0),
        miniextendr_api::either_impl::Either::Right(_) => -1,
    }
}

/// How many values `no_na` should have refused: missing or `NaN`.
fn count_refused(values: &[Option<f64>]) -> i32 {
    let n = values.iter().filter(|v| v.is_none_or(f64::is_nan)).count();
    i32::try_from(n).expect("count fits i32")
}

/// `"NULL"` for `None`, the number for a value, `"missing"` for a value
/// `no_na` should have refused (a leak).
fn describe(value: Option<Option<f64>>) -> String {
    match value {
        None => "NULL".to_string(),
        Some(Some(v)) if !v.is_nan() => v.to_string(),
        Some(_) => "missing".to_string(),
    }
}
// endregion

// region: impl methods

/// Holder for the impl-method `inherits(...)` / `no_na(...)` fixture.
#[derive(miniextendr_api::ExternalPtr)]
pub struct ParamCheckHolder {
    total: f64,
}

/// Env class whose `add()` method checks its arguments with
/// `inherits(...)` / `no_na(...)`.
#[miniextendr(env)]
impl ParamCheckHolder {
    pub fn new() -> Self {
        Self { total: 0.0 }
    }

    /// Add `y` if `x` is an `mx_obj`.
    /// @param x An object of class `mx_obj` or `mx_other`.
    /// @param y A non-NA double.
    #[miniextendr(inherits(x = "mx_obj, mx_other"), no_na(y))]
    pub fn add(&mut self, x: List, y: f64) -> f64 {
        let _ = x;
        self.total += y;
        self.total
    }

    /// `add()` with the method-level messages.
    /// @param x An object of class `mx_obj` or `mx_other`.
    /// @param y A non-NA double.
    #[miniextendr(
        inherits(x(
            class = "mx_obj, mx_other",
            message = "`x` must be an `mx_obj` or an `mx_other`"
        )),
        no_na(y(message = "`y` must be a number, not NA"))
    )]
    pub fn add_checked(&mut self, x: List, y: f64) -> f64 {
        self.add(x, y)
    }

    /// Add a dose read like `as.numeric()`; `" NA "` is refused as `NA` is.
    /// @param dose A number, string or factor label that is not missing.
    #[miniextendr(no_na(dose))]
    pub fn add_dose(&mut self, dose: AsNumeric) -> f64 {
        self.total += dose.0.expect("no_na refuses a missing value");
        self.total
    }

    /// Add several doses, with a method-level message for a missing one.
    /// @param doses Numbers, strings or factor labels, none of them missing.
    #[miniextendr(no_na(doses(message = "every dose must be a number")))]
    pub fn add_doses(&mut self, doses: AsNumericVec) -> f64 {
        self.total += doses.0.into_iter().flatten().sum::<f64>();
        self.total
    }
}
// endregion
