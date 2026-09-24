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

use miniextendr_api::{List, Missing, SEXP, miniextendr};

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
}
// endregion
