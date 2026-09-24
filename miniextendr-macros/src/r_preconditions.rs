//! R-side precondition generation for type checking.
//!
//! Generates argument checks in R wrapper functions that run BEFORE the `.Call()` boundary.
//! This gives users clear, idiomatic R error messages with proper stack traces instead of
//! Rust panic messages.
//!
//! Each check tests ONE thing and names the requirement it enforces:
//!
//! ```r
//! add <- function(a, b) {
//!   if (!isTRUE(is.integer(a))) .miniextendr_arg_error("a", "must be integer")
//!   if (!isTRUE(length(a) == 1L)) .miniextendr_arg_error("a", "must have length 1")
//!   if (!isTRUE(is.integer(b))) .miniextendr_arg_error("b", "must be integer")
//!   if (!isTRUE(length(b) == 1L)) .miniextendr_arg_error("b", "must have length 1")
//!   .Call(C_add, .call = match.call(), a, b)
//! }
//! ```
//!
//! A failing check calls the preamble helper `.miniextendr_arg_error`
//! (`miniextendr-api/src/registry.rs`), which raises the same condition as a
//! failed Rust conversion: the crate's `conversion_error_class`, `rust_error`,
//! `kind = "conversion"` and `e$param`, with the message `'a' must be integer`
//! (#1591). A passing check costs one `isTRUE()` test; the condition is only
//! built on failure. `isTRUE()` keeps `stopifnot()`'s failure semantics (`NA`
//! fails too), and the guards are cheaper than the `stopifnot()` call they
//! replace. [`conversion_expectation`] gives the Rust conversion the same
//! vocabulary for its messages.

use std::collections::{HashMap, HashSet};

/// A single R-side check on one parameter: a requirement and the R condition
/// enforcing it.
///
/// Formatted as a guard (see [`RAssertion::to_guard`]):
/// `if (!isTRUE(is.numeric(x))) .miniextendr_arg_error("x", "must be numeric")`.
struct RAssertion {
    /// R name of the checked parameter (`e$param` on failure).
    param: String,
    /// What the parameter must satisfy, without the parameter name
    /// (`must be numeric`). The condition message is `'<param>' <requirement>`.
    requirement: String,
    /// R expression that must evaluate to `TRUE` for the check to pass.
    condition: String,
    /// The author's own condition message (`message = "..."` on `inherits` /
    /// `no_na`), used verbatim instead of `'<param>' <requirement>`.
    message: Option<String>,
}

impl RAssertion {
    /// Create a new assertion on `param` with its requirement and R condition expression.
    fn new(
        param: impl Into<String>,
        requirement: impl Into<String>,
        condition: impl Into<String>,
    ) -> Self {
        Self {
            param: param.into(),
            requirement: requirement.into(),
            condition: condition.into(),
            message: None,
        }
    }

    /// Use `message` verbatim as the condition message of a failure.
    fn with_message(mut self, message: Option<&String>) -> Self {
        self.message = message.cloned();
        self
    }

    /// The condition message a failure raises: the author's message, else
    /// `'<param>' <requirement>`.
    #[cfg(test)]
    fn message(&self) -> String {
        match &self.message {
            Some(message) => message.clone(),
            None => format!("'{}' {}", self.param, self.requirement),
        }
    }

    /// Format as a guard that raises an argument error on failure:
    /// `if (!isTRUE(condition)) .miniextendr_arg_error("param", "requirement")`,
    /// or with the author's message
    /// `if (!isTRUE(condition)) .miniextendr_arg_error("param", message = "...")`.
    ///
    /// `call` is the call the error is attributed to. `None` leaves the
    /// helper's default, the wrapper's own call (what `stopifnot()` reported);
    /// a `call = caller` wrapper passes `.mx_call` (#1548). Next to a named
    /// `message` the call is named too: positionally it would bind to `what`.
    fn to_guard(&self, call: Option<&str>) -> String {
        match &self.message {
            None => {
                let call_arg = call.map(|c| format!(", {c}")).unwrap_or_default();
                format!(
                    "if (!isTRUE({})) .miniextendr_arg_error(\"{}\", \"{}\"{})",
                    self.condition, self.param, self.requirement, call_arg
                )
            }
            Some(message) => {
                let call_arg = call.map(|c| format!(", call = {c}")).unwrap_or_default();
                format!(
                    "if (!isTRUE({})) .miniextendr_arg_error(\"{}\", message = \"{}\"{})",
                    self.condition,
                    self.param,
                    r_string_escape(message),
                    call_arg
                )
            }
        }
    }

    /// Wrap for nullable: prepend `is.null(param) || ` to the condition,
    /// and adjust the requirement to mention NULL.
    fn nullable(self) -> Self {
        let requirement = if let Some(rest) = self.requirement.strip_prefix("must be ") {
            // "must be character" → "must be NULL or character"
            format!("must be NULL or {rest}")
        } else if let Some(rest) = self.requirement.strip_prefix("must have ") {
            // "must have length 1" → "must be NULL or have length 1"
            format!("must be NULL or have {rest}")
        } else {
            format!("{} (or NULL)", self.requirement)
        };
        Self {
            condition: format!("is.null({}) || {}", self.param, self.condition),
            param: self.param,
            requirement,
            message: self.message,
        }
    }
}

/// Per-function knobs that influence precondition codegen.
///
/// Coercion preserves numeric input types, extends `bool` / `Vec<bool>`
/// checks to accept integers as well as logicals, and widens the native
/// `i32` / `Vec<i32>` gates to whole-number doubles. Other types keep their checks.
/// Strict input conversion remains enforced in Rust, where range and precision
/// failures can carry contextual diagnostics.
#[derive(Clone, Default)]
pub struct PreconditionOptions {
    /// `coerce` knob is active for all parameters (`coerce_all`).
    pub coerce_all: bool,
    /// R-normalized names of parameters with a per-param `coerce` attribute.
    pub coerce_params: HashSet<String>,
    /// Checks the author named per parameter (`inherits`, `no_na`), keyed by
    /// R-normalized parameter name.
    pub explicit: HashMap<String, ExplicitChecks>,
    /// `no_preconditions` / `fast`: drop the checks derived from parameter
    /// types. The [`explicit`](Self::explicit) checks are still emitted.
    pub no_type_checks: bool,
}

impl PreconditionOptions {
    /// Returns `true` if the parameter `r_name` is coerced (function-wide or
    /// per-param).
    fn is_coerced(&self, r_name: &str) -> bool {
        self.coerce_all || self.coerce_params.contains(r_name)
    }
}

/// R-side checks the author asked for by name on one parameter, rather than
/// ones derived from its Rust type.
///
/// Spelled `#[miniextendr(inherits = "cls", no_na)]` on a standalone fn
/// parameter, or `inherits(x = "cls")` / `no_na(x)` on an impl or trait
/// method. They run after the type checks, as the same kind of guards
/// raising the same argument error, and survive `no_preconditions` / `fast`: the
/// Rust conversion cannot check them, so dropping them would change what the
/// function accepts.
///
/// Each check can carry the author's own condition message
/// (`inherits(class = "cls", message = "...")`, `no_na(message = "...")`;
/// method level `inherits(x(class = "cls", message = "..."))`,
/// `no_na(x(message = "..."))`), used verbatim in place of the generated
/// `'x' must inherit from 'cls'`. The condition is otherwise the same.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ExplicitChecks {
    /// `inherits = "cls"` / `inherits("a", "b")`: the argument must inherit
    /// from at least one of these classes (`inherits(x, c(...))`).
    pub inherits: Option<Vec<String>>,
    /// `message = "..."` in `inherits(...)`: the message of a failed class
    /// check, for all of its classes. Only set together with `inherits`.
    pub inherits_message: Option<String>,
    /// `no_na`: the argument must not contain `NA` (`!anyNA(x)`, so `NaN`
    /// is refused too, as `is.na()` does).
    pub no_na: bool,
    /// `no_na(message = "...")`: the message of a failed NA check. Only set
    /// together with `no_na`.
    pub no_na_message: Option<String>,
}

impl ExplicitChecks {
    /// Whether any check is requested.
    pub fn is_empty(&self) -> bool {
        self.inherits.is_none() && !self.no_na
    }

    /// Merge `other` into `self` (a parameter may carry several attributes).
    ///
    /// Fails when both give a message for the same check; the error names
    /// the check.
    pub fn merge(&mut self, other: ExplicitChecks) -> Result<(), String> {
        merge_message(
            &mut self.inherits_message,
            other.inherits_message,
            "inherits",
        )?;
        merge_message(&mut self.no_na_message, other.no_na_message, "no_na")?;
        if let Some(classes) = other.inherits {
            self.inherits.get_or_insert_with(Vec::new).extend(classes);
        }
        self.no_na |= other.no_na;
        Ok(())
    }

    /// The assertions for parameter `param` of type `ty`.
    ///
    /// An `Option<T>` parameter passes `NULL`, and a `Missing<T>` parameter
    /// passes when the argument was omitted. The failure message names only
    /// the failed requirement: it is shown only for a present, non-`NULL`
    /// value.
    fn assertions(&self, param: &str, ty: &syn::Type) -> Vec<RAssertion> {
        let mut out = Vec::new();
        let value_ty = crate::miniextendr_fn::get_missing_inner_type(ty).unwrap_or(ty);
        let value_ty = crate::type_inspect::option_inner_type(value_ty).unwrap_or(value_ty);
        if self.no_na {
            let verb = if crate::miniextendr_fn::is_vector_like_type(value_ty) {
                "contain"
            } else {
                "be"
            };
            out.push(
                RAssertion::new(
                    param,
                    format!("must not {verb} NA"),
                    format!("!anyNA({param})"),
                )
                .with_message(self.no_na_message.as_ref()),
            );
        }
        if let Some(classes) = &self.inherits {
            let quoted: Vec<String> = classes
                .iter()
                .map(|c| format!("'{}'", r_string_escape(c)))
                .collect();
            let literals: Vec<String> = classes
                .iter()
                .map(|c| format!("\"{}\"", r_string_escape(c)))
                .collect();
            let what = match quoted.as_slice() {
                [one] => one.clone(),
                [init @ .., last] => format!("{} or {last}", init.join(", ")),
                [] => unreachable!("inherits() is parsed as a non-empty class list"),
            };
            let class_arg = match literals.as_slice() {
                [one] => one.clone(),
                _ => format!("c({})", literals.join(", ")),
            };
            out.push(
                RAssertion::new(
                    param,
                    format!("must inherit from {what}"),
                    format!("inherits({param}, {class_arg})"),
                )
                .with_message(self.inherits_message.as_ref()),
            );
        }
        let guard = if crate::miniextendr_fn::is_missing_type(ty) {
            Some(format!("missing({param})"))
        } else if crate::type_inspect::is_option_type(ty) {
            Some(format!("is.null({param})"))
        } else {
            None
        };
        if let Some(guard) = guard {
            for a in &mut out {
                a.condition = format!("{guard} || {}", a.condition);
            }
        }
        out
    }
}

/// Record `from` as the message of `check`, which may have only one.
fn merge_message(
    into: &mut Option<String>,
    from: Option<String>,
    check: &str,
) -> Result<(), String> {
    match (into.is_some(), from) {
        (true, Some(_)) => Err(format!(
            "`{check}` is given more than one `message` on this parameter; give it once"
        )),
        (false, Some(message)) => {
            *into = Some(message);
            Ok(())
        }
        (_, None) => Ok(()),
    }
}

/// Escape `s` for the inside of an R double-quoted string literal (the
/// requirement text, the class names in `inherits()` and the author's
/// messages).
///
/// Besides `\` and `"`, a newline, carriage return or tab becomes its escape
/// (a guard stays on one line of the wrappers file), any other control
/// character and every non-ASCII character a `\u{..}` / `\U{..}` escape: R
/// code in a package must be ASCII, and R reads these escapes back to the
/// same UTF-8 text. A NUL cannot be written (R strings cannot hold one);
/// the attribute parser rejects it in a message.
fn r_string_escape(s: &str) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_ascii() && !c.is_ascii_control() => out.push(c),
            c if u32::from(c) <= 0xFFFF => {
                let _ = write!(out, "\\u{{{:x}}}", u32::from(c));
            }
            c => {
                let _ = write!(out, "\\U{{{:x}}}", u32::from(c));
            }
        }
    }
    out
}

/// Classification of an R-side type check for a function parameter.
///
/// Each variant maps to a specific set of R-side checks. Numeric checks
/// use a broad predicate (`is.numeric || is.logical || is.raw`) because R coerces
/// logical to numeric freely and raw to integer is valid for byte-sized types.
/// Borderline cases (e.g., raw to i64 in strict mode) pass the precondition and
/// reach Rust's strict checker, which produces better contextual error messages.
enum RTypeCheck {
    /// Numeric scalar: type check + length-1 check (2 assertions).
    /// Used for the multi-source scalars `f32`, `i8`, `i16`, `i64`, `isize`, and
    /// for a coerced native `f64`. Native `i32` / `f64` without coerce use
    /// [`RTypeCheck::Scalar`] with `"integer"` / `"double"`.
    ScalarNumeric,
    /// Coerced bool: logical or integer, with length one.
    ScalarLogicalOrInteger,
    /// Coerced bool vector: logical or integer.
    VectorLogicalOrInteger,
    /// Coerced native `i32` scalar: integer, logical, raw, or a whole-number
    /// double, with length one. The vector form is [`RTypeCheck::VectorIntegerWide`].
    ScalarIntegerWide,
    /// Non-negative numeric scalar: type + length-1 + `>= 0` (3 assertions).
    /// Used for `u16`, `u32`, `u64`, `usize`.
    ScalarNonNeg,
    /// Non-numeric scalar: `is.<type>(x)` + length-1 check (2 assertions).
    /// The string is the R type predicate name (e.g., `"logical"`, `"character"`).
    Scalar(&'static str),
    /// Floating-point numeric vector: loose `is.numeric || is.logical || is.raw`
    /// (1 assertion). Used for `Vec<f32>` (multi-source converter) and for a
    /// coerced native `Vec<f64>`; `Vec<f64>` / `&[f64]` without coerce use
    /// [`RTypeCheck::Vector`] with `"double"` because the native path reads
    /// REALSXP only.
    VectorNumeric,
    /// **INTSXP-only** integer vector: `is.integer(x)` (1 assertion). Used *only*
    /// for `Vec<i32>` / `&[i32]` (issue #616). These use the native `RNativeType`
    /// inbound path (`impl_vec_try_from_sexp_native!(i32)`), which requires
    /// `INTSXP` and rejects `REALSXP` outright. The loose `is.numeric` predicate
    /// previously let a `double` like `c(1, 2)` pass the R gate only to fail with
    /// a cryptic "expected INTSXP, got REALSXP". `is.integer(x)` rejects every
    /// `double` (whole or fractional) at the boundary with a clean message,
    /// matching the actual Rust behaviour and closing the silent-truncation gap.
    VectorIntegerStrict,
    /// **Wide** integer vector accepting `REALSXP` whole-number values: the
    /// lossless whole-number predicate (1 assertion). Used for every non-`i32`
    /// integer element type — `Vec<i8>` / `Vec<i16>` / `Vec<u16>` / `Vec<u32>`
    /// and the 64-bit family `Vec<i64>` / `Vec<u64>` / `Vec<isize>` / `Vec<usize>`
    /// (issue #616). These use the coercing inbound path
    /// (`impl_vec_try_from_sexp_numeric!` → `from_numeric_vec_with`) which accepts
    /// INTSXP/REALSXP/RAWSXP/LGLSXP and rejects fractional doubles element-wise
    /// (`f64: TryCoerce<T>` checks `self.fract() != 0`). We accept
    /// integer/logical/raw **and** whole-number doubles, and reject genuinely
    /// lossy fractional doubles (`1.5`) at the boundary.
    VectorIntegerWide,
    /// Non-numeric vector: `is.<type>(x)` only (1 assertion).
    /// The string is the R type predicate name.
    Vector(&'static str),
    /// `AsNumeric` scalar: numeric, logical, character, or factor, with length
    /// one (2 assertions). The marker parses character and factor labels the way
    /// `as.numeric()` does, so the gate admits exactly the types it reads.
    ScalarNumericOrText,
    /// `AsNumericVec`: numeric, logical, character, or factor (1 assertion).
    VectorNumericOrText,
    /// Nullable wrapper around an inner check: prepends `is.null(x) ||` to each assertion
    /// and adjusts messages to mention NULL.
    Nullable(Box<RTypeCheck>),
    /// List check: `is.list(x)` (1 assertion).
    /// Used for `HashMap`, `BTreeMap`, `NamedList`, `List`, `ListMut`.
    List,
}

/// Build the R expression for the numeric type predicate.
///
/// Returns `"is.numeric(p) || is.logical(p) || is.raw(p)"` for a given parameter `p`.
/// This broad predicate matches R's coercion rules: logical coerces to numeric freely,
/// and raw is accepted because it represents byte-level data.
fn numeric_type_check(param: &str) -> String {
    format!(
        "is.numeric({p}) || is.logical({p}) || is.raw({p})",
        p = param
    )
}

/// Build the lossless whole-number predicate for a `REALSXP`-accepting integer
/// vector (the `i64` / `u64` / `isize` / `usize` family).
///
/// Accepts integer/logical/raw and whole-number doubles, rejects fractional
/// doubles. The whole-number test is NA-safe (`is.na(p) | p == trunc(p)`):
/// `NA_real_` passes (maps to `NA_integer_`), `1.5` fails. `Inf`/`NaN` satisfy
/// `x == trunc(x)` in R but are caught by the Rust-side range/NaN conversion
/// check instead.
fn integer_vector_wide_check(param: &str) -> String {
    format!(
        "is.integer({p}) || is.logical({p}) || is.raw({p}) || \
         (is.numeric({p}) && all(is.na({p}) | {p} == trunc({p})))",
        p = param
    )
}

/// Build the predicate for the `AsNumeric` / `AsNumericVec` markers.
///
/// `is.numeric()` is `FALSE` for a factor, so `is.factor()` is listed on its
/// own: the marker reads a factor by its labels. Raw and complex stay out, as
/// the marker refuses them.
fn numeric_or_text_check(param: &str) -> String {
    format!(
        "is.numeric({p}) || is.logical({p}) || is.character({p}) || is.factor({p})",
        p = param
    )
}

impl RTypeCheck {
    /// Produce the individual checks for this type check.
    ///
    /// Returns one or more `RAssertion` values, each one requirement on
    /// `param` (the R parameter name used in the conditions and messages).
    fn assertions(&self, param: &str) -> Vec<RAssertion> {
        let length_one = || {
            RAssertion::new(
                param,
                "must have length 1",
                format!("length({param}) == 1L"),
            )
        };
        match self {
            RTypeCheck::ScalarNumeric => vec![
                RAssertion::new(
                    param,
                    "must be numeric, logical, or raw",
                    numeric_type_check(param),
                ),
                length_one(),
            ],
            RTypeCheck::ScalarLogicalOrInteger => vec![
                RAssertion::new(
                    param,
                    "must be logical or integer",
                    format!("is.logical({p}) || is.integer({p})", p = param),
                ),
                length_one(),
            ],
            RTypeCheck::VectorLogicalOrInteger => vec![RAssertion::new(
                param,
                "must be logical or integer",
                format!("is.logical({p}) || is.integer({p})", p = param),
            )],
            RTypeCheck::ScalarIntegerWide => vec![
                RAssertion::new(
                    param,
                    "must be integer or whole-number numeric",
                    integer_vector_wide_check(param),
                ),
                length_one(),
            ],
            RTypeCheck::ScalarNonNeg => vec![
                RAssertion::new(
                    param,
                    "must be numeric, logical, or raw",
                    numeric_type_check(param),
                ),
                length_one(),
                RAssertion::new(
                    param,
                    "must be non-negative",
                    // raw is always non-negative; guard with is.raw() to avoid
                    // "comparison not implemented" error for raw values
                    format!("is.raw({p}) || {p} >= 0", p = param),
                ),
            ],
            RTypeCheck::Scalar(r_type) => vec![
                RAssertion::new(
                    param,
                    format!("must be {r_type}"),
                    format!("is.{r_type}({param})"),
                ),
                length_one(),
            ],
            RTypeCheck::VectorNumeric => vec![RAssertion::new(
                param,
                "must be numeric, logical, or raw",
                numeric_type_check(param),
            )],
            RTypeCheck::VectorIntegerStrict => vec![RAssertion::new(
                param,
                "must be an integer vector",
                format!("is.integer({param})"),
            )],
            RTypeCheck::VectorIntegerWide => vec![RAssertion::new(
                param,
                "must be integer or whole-number numeric",
                integer_vector_wide_check(param),
            )],
            RTypeCheck::Vector(r_type) => vec![RAssertion::new(
                param,
                format!("must be {r_type}"),
                format!("is.{r_type}({param})"),
            )],
            RTypeCheck::ScalarNumericOrText => vec![
                RAssertion::new(
                    param,
                    "must be numeric, logical, character, or factor",
                    numeric_or_text_check(param),
                ),
                length_one(),
            ],
            RTypeCheck::VectorNumericOrText => vec![RAssertion::new(
                param,
                "must be numeric, logical, character, or factor",
                numeric_or_text_check(param),
            )],
            RTypeCheck::Nullable(inner) => inner
                .assertions(param)
                .into_iter()
                .map(RAssertion::nullable)
                .collect(),
            RTypeCheck::List => vec![RAssertion::new(
                param,
                "must be a list",
                format!("is.list({param})"),
            )],
        }
    }

    /// What an argument of this type must be, in R terms: the `<expected>` of
    /// a Rust conversion failure's message, `'<p>' must be <expected>: <reason>`
    /// (#1591).
    ///
    /// The R-side checks test one requirement at a time (type, then length);
    /// a conversion failure names the whole expectation once. Scalars read
    /// `a single <noun>` (`a single integer`, `a single string`, `TRUE or
    /// FALSE`), vectors name the R type (`numeric`, `character`, `a list`).
    /// `AsNumeric` / `AsNumericVec` say `a single number` / `numeric`, the
    /// value they produce, rather than the inputs they read.
    fn expectation(&self) -> String {
        match self {
            RTypeCheck::ScalarNumeric | RTypeCheck::ScalarNumericOrText => "a single number".into(),
            RTypeCheck::ScalarLogicalOrInteger => "a single logical or integer".into(),
            RTypeCheck::VectorLogicalOrInteger => "logical or integer".into(),
            RTypeCheck::ScalarIntegerWide => "a single whole number".into(),
            RTypeCheck::ScalarNonNeg => "a single non-negative number".into(),
            RTypeCheck::Scalar(r_type) => match *r_type {
                "logical" => "TRUE or FALSE".into(),
                "character" => "a single string".into(),
                "integer" | "double" => format!("a single {r_type}"),
                other => format!("a single {other} value"),
            },
            RTypeCheck::VectorNumeric | RTypeCheck::VectorNumericOrText => "numeric".into(),
            RTypeCheck::VectorIntegerStrict => "integer".into(),
            RTypeCheck::VectorIntegerWide => "integer or whole-number numeric".into(),
            RTypeCheck::Vector(r_type) => (*r_type).into(),
            RTypeCheck::Nullable(inner) => match inner.expectation().as_str() {
                "TRUE or FALSE" => "NULL, TRUE or FALSE".into(),
                inner => format!("NULL or {inner}"),
            },
            RTypeCheck::List => "a list".into(),
        }
    }
}

/// The R-facing expectation for a parameter of Rust type `ty`, for the
/// message of its Rust conversion failure: `'<p>' must be <expected>: ...`
/// (see [`RTypeCheck::expectation`]).
///
/// Derived from the same type classification as the R-side checks, including
/// the `coerce` widening (`coerced`) and the `Missing<T>` / `Option<T>`
/// wrappers, so the two paths describe an argument the same way. `None` for
/// a type without an R-side check (a custom `TryFromSexp` type, `Either`,
/// the `AsFromStr` family, ...): the conversion then reports
/// `invalid '<p>' argument: <reason>`.
pub(crate) fn conversion_expectation(ty: &syn::Type, coerced: bool) -> Option<String> {
    let ty = crate::miniextendr_fn::get_missing_inner_type(ty).unwrap_or(ty);
    let check = r_check_for_type(ty)?;
    let check = if coerced {
        coerce_widened(check, ty)
    } else {
        check
    };
    Some(check.expectation())
}

/// Keep the R gate in sync with the conversion actually selected for this type.
///
/// Native `i32` widens to whole-number doubles (plus logical/raw), so its gate
/// becomes the lossless whole-number predicate; `Vec<i32>` moves from the
/// INTSXP-only gate (#616) to the same wide predicate the other integer vectors
/// use. Native `f64` / `Vec<f64>` move from the `is.double` gate to the loose
/// numeric predicate. `Option<bool>` has no coercion mapping and stays nullable
/// logical.
fn coerce_widened(check: RTypeCheck, ty: &syn::Type) -> RTypeCheck {
    use crate::miniextendr_fn::CoercionMapping;
    match CoercionMapping::from_type(ty) {
        Some(CoercionMapping::Bool) => RTypeCheck::ScalarLogicalOrInteger,
        Some(CoercionMapping::BoolVec) => RTypeCheck::VectorLogicalOrInteger,
        Some(CoercionMapping::NativeInt) => RTypeCheck::ScalarIntegerWide,
        Some(CoercionMapping::NativeIntVec) => RTypeCheck::VectorIntegerWide,
        Some(CoercionMapping::NativeReal) => RTypeCheck::ScalarNumeric,
        Some(CoercionMapping::NativeRealVec) => RTypeCheck::VectorNumeric,
        _ => check,
    }
}

/// Map a Rust type to its R-side type check, if applicable.
///
/// Returns `None` for types that should skip precondition checks (SEXP, Dots, ExternalPtr, etc.).
fn r_check_for_type(ty: &syn::Type) -> Option<RTypeCheck> {
    match ty {
        syn::Type::Path(type_path) => r_check_for_type_path(type_path),
        syn::Type::Reference(type_ref) => r_check_for_reference(type_ref),
        _ => None,
    }
}

/// Map a `syn::TypePath` to its R-side type check.
///
/// Handles the most common case: simple types (`i32`, `String`, `bool`),
/// generic wrappers (`Vec<T>`, `Option<T>`), map types, and skip types.
/// Returns `None` for types that cannot be prechecked from R.
fn r_check_for_type_path(type_path: &syn::TypePath) -> Option<RTypeCheck> {
    let segment = type_path.path.segments.last()?;
    let ident = segment.ident.to_string();

    match ident.as_str() {
        // Native scalars convert from exactly one SEXPTYPE (`i32::try_from_sexp`
        // is INTSXP-only, `f64` REALSXP-only), so the R gate names that storage
        // instead of letting a double reach Rust and fail with "expected INTSXP,
        // got REALSXP" (#1112). Under `coerce` the gate widens with the conversion
        // (see `coerce_widened`).
        "i32" => Some(RTypeCheck::Scalar("integer")),
        "f64" => Some(RTypeCheck::Scalar("double")),

        // Non-native numeric scalars use the multi-source converters (integer,
        // double, logical, raw), so the loose predicate matches what Rust accepts.
        "f32" | "i8" | "i16" | "i64" | "isize" => Some(RTypeCheck::ScalarNumeric),

        // Unsigned numeric scalars (non-negative constraint)
        "u16" | "u32" | "u64" | "usize" => Some(RTypeCheck::ScalarNonNeg),

        // Logical scalar
        "bool" | "Rbool" | "Rboolean" => Some(RTypeCheck::Scalar("logical")),

        // Character scalar
        "String" | "char" | "PathBuf" => Some(RTypeCheck::Scalar("character")),

        // Raw scalar
        "u8" => Some(RTypeCheck::Scalar("raw")),

        // Complex scalar
        "Rcomplex" => Some(RTypeCheck::Scalar("complex")),

        // `as.numeric()`-style markers: numbers, text, and factor labels.
        "AsNumeric" => Some(RTypeCheck::ScalarNumericOrText),
        "AsNumericVec" => Some(RTypeCheck::VectorNumericOrText),

        // `as.character()`-style markers: any atomic vector (a list or data
        // frame is refused, as `as.character()` would flatten it), with length
        // one for the scalar. "'x' must be atomic" is base R's own wording.
        "AsCharacter" => Some(RTypeCheck::Scalar("atomic")),
        "AsCharacterVec" => Some(RTypeCheck::Vector("atomic")),

        // Option<T> → Nullable
        "Option" => {
            let inner_ty = extract_single_generic_arg(segment)?;
            r_check_for_type(inner_ty).map(|inner| RTypeCheck::Nullable(Box::new(inner)))
        }

        // Vec<T> → Vector (depends on element type)
        "Vec" => {
            let inner_ty = extract_single_generic_arg(segment)?;
            r_check_for_vec_element(inner_ty)
        }

        // Map types and named list → List
        "HashMap" | "BTreeMap" | "NamedList" => Some(RTypeCheck::List),

        // List (bare) → List
        "List" | "ListMut" => Some(RTypeCheck::List),

        // Skip types: SEXP, Dots, Missing, ExternalPtr, RLogical, etc.
        "SEXP" | "Dots" | "Missing" | "ExternalPtr" | "OwnedProtect" => None,

        // Unknown type → skip (let Rust side validate)
        _ => None,
    }
}

/// Map a reference type to its R-side type check.
///
/// Handles `&str` and `&Path` (character scalar), `&[T]` (vector based on element type),
/// and `&Dots` (skipped). Returns `None` for unrecognized reference types.
fn r_check_for_reference(type_ref: &syn::TypeReference) -> Option<RTypeCheck> {
    match type_ref.elem.as_ref() {
        // &str → character scalar
        syn::Type::Path(tp) => {
            let seg = tp.path.segments.last()?;
            match seg.ident.to_string().as_str() {
                "str" => Some(RTypeCheck::Scalar("character")),
                "Path" => Some(RTypeCheck::Scalar("character")),
                "Dots" => None,
                _ => None,
            }
        }
        // &[T] → vector check based on element type
        syn::Type::Slice(slice) => r_check_for_vec_element(&slice.elem),
        _ => None,
    }
}

/// Map a `Vec<T>` or `&[T]` element type to the appropriate vector type check.
///
/// Numeric elements produce `VectorNumeric`, `bool` produces `Vector("logical")`,
/// `String` produces `Vector("character")`, etc. Handles nested `Option<T>` for
/// nullable element types (e.g., `Vec<Option<String>>` becomes character vector).
fn r_check_for_vec_element(elem_ty: &syn::Type) -> Option<RTypeCheck> {
    let syn::Type::Path(tp) = elem_ty else {
        return None;
    };
    let seg = tp.path.segments.last()?;
    let ident = seg.ident.to_string();

    match ident.as_str() {
        // `Vec<f64>` / `&[f64]` read the REALSXP payload directly (native path),
        // so the gate is `is.double`; `Vec<f32>` coerces from every numeric source.
        "f64" => Some(RTypeCheck::Vector("double")),
        "f32" => Some(RTypeCheck::VectorNumeric),

        // `Vec<i32>` / `&[i32]` is the *only* INTSXP-only integer vector (issue
        // #616): its inbound conversion uses the native `RNativeType` path
        // (`impl_vec_try_from_sexp_native!(i32)` in `from_r/collections.rs`),
        // which rejects REALSXP outright. So the R gate is `is.integer` — a
        // `double` like `c(1, 2)` fails cleanly at the boundary instead of
        // producing a cryptic "expected INTSXP, got REALSXP". `u8` is handled
        // below as a raw vector.
        "i32" => Some(RTypeCheck::VectorIntegerStrict),

        // Every other integer-element vector (`i8`/`i16`/`u16`/`u32` and the
        // 64-bit family `i64`/`u64`/`isize`/`usize`) uses the coercing inbound
        // path (`impl_vec_try_from_sexp_numeric!` → `from_numeric_vec_with`),
        // which accepts INTSXP/REALSXP/RAWSXP/LGLSXP and rejects fractional
        // doubles element-wise via `f64: TryCoerce<T>` (the `self.fract() != 0`
        // check in `coerce.rs`). The R gate mirrors that: accept integer/logical/
        // raw + whole-number doubles, reject fractional. (64-bit ints also arrive
        // as REALSXP since R has no native 64-bit integer type.)
        "i8" | "i16" | "u16" | "u32" | "i64" | "u64" | "isize" | "usize" => {
            Some(RTypeCheck::VectorIntegerWide)
        }

        // Logical vector
        "bool" => Some(RTypeCheck::Vector("logical")),

        // Character vector
        "String" => Some(RTypeCheck::Vector("character")),

        // Raw vector
        "u8" => Some(RTypeCheck::Vector("raw")),

        // Complex vector
        "Rcomplex" => Some(RTypeCheck::Vector("complex")),

        // Vec<Option<T>> — e.g., Vec<Option<String>> for nullable strings
        "Option" => {
            let inner = extract_single_generic_arg(seg)?;
            // Vec<Option<String>> → character, Vec<Option<i32>> → numeric, etc.
            r_check_for_vec_element(inner)
        }

        _ => None,
    }
}

/// Extract the single generic type argument from a path segment.
///
/// e.g., `Option<String>` → `String`, `Vec<i32>` → `i32`
fn extract_single_generic_arg(segment: &syn::PathSegment) -> Option<&syn::Type> {
    if let syn::PathArguments::AngleBracketed(ref args) = segment.arguments
        && let Some(syn::GenericArgument::Type(ty)) = args.args.first()
    {
        return Some(ty);
    }
    None
}

/// A parameter whose Rust type is not in the static type table.
///
/// Currently, fallback params are recorded but no R-side validation is generated
/// for them -- the Rust-side conversion handles type errors with its own messages.
#[allow(dead_code)] // Read in tests
pub struct FallbackParam {
    /// R-normalized parameter name (e.g., `_dots` becomes `.dots`).
    pub r_name: String,
}

/// Output of precondition analysis for a function's parameters.
///
/// Holds the R-side checks for known types and a list of parameters with
/// unknown types that were not statically prechecked.
pub struct PreconditionOutput {
    /// The checks, in parameter order, each parameter's type checks followed
    /// by its [`ExplicitChecks`]. Rendered by [`PreconditionOutput::guards`].
    assertions: Vec<RAssertion>,
    /// Parameters with unknown custom types that were not prechecked.
    #[allow(dead_code)] // Read in tests
    pub fallback_params: Vec<FallbackParam>,
}

impl PreconditionOutput {
    /// One guard line per check, raising an argument error through
    /// `.miniextendr_arg_error` when it fails (#1591); empty when there is
    /// nothing to check.
    ///
    /// `call` is the call every failure is attributed to: `None` for the
    /// wrapper's own call (the helper's default, the call `stopifnot()`
    /// reported), or `.mx_call` for a `call = caller` wrapper, which binds the
    /// caller's matched call before the checks run (#1548).
    pub fn guards(&self, call: Option<&str>) -> Vec<String> {
        self.assertions.iter().map(|a| a.to_guard(call)).collect()
    }
}

/// Returns `true` for types that should never get a fallback precheck.
///
/// These types are either handled specially by the FFI layer (`SEXP`),
/// consumed by the macro infrastructure (`Dots`, `Missing`), or managed
/// internally (`ExternalPtr`, `OwnedProtect`).
fn is_skip_type(ident: &str) -> bool {
    matches!(
        ident,
        "SEXP" | "Dots" | "Missing" | "ExternalPtr" | "OwnedProtect"
    )
}

/// Returns `true` if a type is unknown to the static type table and should
/// be recorded as a fallback parameter.
///
/// Returns `false` for skip types (SEXP, Dots, etc.) and reference types
/// (which are handled by the static table or skipped).
fn needs_fallback(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Path(tp) => {
            let Some(seg) = tp.path.segments.last() else {
                return false;
            };
            !is_skip_type(&seg.ident.to_string())
        }
        // References (&str, &[T], &Dots) are handled by static table or skipped
        syn::Type::Reference(_) => false,
        _ => false,
    }
}

/// Build precondition checks for a function's parameters.
///
/// Returns the checks for known types (render them with
/// [`PreconditionOutput::guards`]) and the parameters with unknown custom
/// types (`fallback_params`). The guards read:
/// ```r
/// if (!isTRUE(is.integer(a))) .miniextendr_arg_error("a", "must be integer")
/// if (!isTRUE(length(a) == 1L)) .miniextendr_arg_error("a", "must have length 1")
/// ```
///
/// Skips:
/// - `self`/`&self`/`&mut self` (receiver args)
/// - Parameters in `skip_params` (e.g., match_arg params already validated)
/// - Skip types (SEXP, Dots, ExternalPtr, etc.)
/// - Every type-derived check under `opts.no_type_checks`
///
/// A parameter's [`ExplicitChecks`] follow its type checks and are never
/// skipped.
pub fn build_precondition_checks(
    inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
    skip_params: &HashSet<String>,
    opts: &PreconditionOptions,
) -> PreconditionOutput {
    let mut assertions = Vec::new();
    let mut fallback_params = Vec::new();

    for arg in inputs {
        // Skip receiver (self/&self/&mut self)
        let syn::FnArg::Typed(pt) = arg else {
            continue;
        };

        // Extract parameter name
        let syn::Pat::Ident(pat_ident) = pt.pat.as_ref() else {
            continue;
        };

        // Use the R-normalized name for the check (matches the R formal)
        let r_name = crate::r_wrapper_builder::normalize_r_arg_ident(&pat_ident.ident).to_string();

        // Type-derived checks: skipped for match_arg params (already validated
        // by match.arg()) and under `no_preconditions` / `fast`.
        if !opts.no_type_checks && !skip_params.contains(&r_name) {
            // Preserve the ordinary input domain and include any coercion extensions.
            if let Some(mut check) = r_check_for_type(pt.ty.as_ref()) {
                if opts.is_coerced(&r_name) {
                    check = coerce_widened(check, pt.ty.as_ref());
                }
                assertions.extend(check.assertions(&r_name));
            } else if needs_fallback(pt.ty.as_ref()) {
                // Unknown type → record for potential future validation
                fallback_params.push(FallbackParam {
                    r_name: r_name.clone(),
                });
            }
        }

        if let Some(checks) = opts.explicit.get(&r_name) {
            assertions.extend(checks.assertions(&r_name, pt.ty.as_ref()));
        }
    }

    PreconditionOutput {
        assertions,
        fallback_params,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to parse a type string into syn::Type
    fn parse_type(s: &str) -> syn::Type {
        syn::parse_str(s).unwrap()
    }

    /// Helper to get assertions for a type.
    fn assertions_for(ty_str: &str, param: &str) -> Vec<RAssertion> {
        let ty = parse_type(ty_str);
        r_check_for_type(&ty).unwrap().assertions(param)
    }

    #[test]
    fn scalar_numeric_produces_two_assertions() {
        let asserts = assertions_for("i64", "x");
        assert_eq!(asserts.len(), 2);
        assert_eq!(asserts[0].message(), "'x' must be numeric, logical, or raw");
        assert_eq!(
            asserts[0].condition,
            "is.numeric(x) || is.logical(x) || is.raw(x)"
        );
        assert_eq!(asserts[1].message(), "'x' must have length 1");
        assert_eq!(asserts[1].condition, "length(x) == 1L");
    }

    #[test]
    fn native_scalars_use_their_storage_gate() {
        // `i32` / `f64` convert from one SEXPTYPE, so the gate says so (#1112).
        let asserts = assertions_for("i32", "x");
        assert_eq!(asserts.len(), 2);
        assert_eq!(asserts[0].message(), "'x' must be integer");
        assert_eq!(asserts[0].condition, "is.integer(x)");
        let asserts = assertions_for("f64", "x");
        assert_eq!(asserts[0].message(), "'x' must be double");
        assert_eq!(asserts[0].condition, "is.double(x)");
        let asserts = assertions_for("Vec<f64>", "x");
        assert_eq!(asserts.len(), 1);
        assert_eq!(asserts[0].condition, "is.double(x)");
        let asserts = assertions_for("&[f64]", "x");
        assert_eq!(asserts[0].condition, "is.double(x)");
    }

    #[test]
    fn all_signed_numeric_types_use_scalar_numeric() {
        for ty_str in &["f32", "i8", "i16", "i64", "isize"] {
            let asserts = assertions_for(ty_str, "x");
            assert_eq!(asserts.len(), 2, "{} should produce 2 assertions", ty_str);
            assert!(
                asserts[0].condition.contains("is.numeric(x)"),
                "{} type check",
                ty_str
            );
            assert!(
                asserts[0].condition.contains("is.logical(x)"),
                "{} accepts logical",
                ty_str
            );
            assert!(
                asserts[0].condition.contains("is.raw(x)"),
                "{} accepts raw",
                ty_str
            );
        }
    }

    #[test]
    fn scalar_non_neg_produces_three_assertions() {
        let asserts = assertions_for("u32", "n");
        assert_eq!(asserts.len(), 3);
        assert_eq!(asserts[0].message(), "'n' must be numeric, logical, or raw");
        assert_eq!(asserts[1].message(), "'n' must have length 1");
        assert_eq!(asserts[2].message(), "'n' must be non-negative");
        assert_eq!(asserts[2].condition, "is.raw(n) || n >= 0");
    }

    #[test]
    fn all_unsigned_types_use_scalar_non_neg() {
        for ty_str in &["u16", "u32", "u64", "usize"] {
            let asserts = assertions_for(ty_str, "x");
            assert_eq!(asserts.len(), 3, "{} should produce 3 assertions", ty_str);
            assert!(
                asserts[2].condition.contains(">= 0"),
                "{} non-neg check",
                ty_str
            );
        }
    }

    #[test]
    fn scalar_logical() {
        let asserts = assertions_for("bool", "x");
        assert_eq!(asserts.len(), 2);
        assert_eq!(asserts[0].message(), "'x' must be logical");
        assert_eq!(asserts[0].condition, "is.logical(x)");
        assert_eq!(asserts[1].condition, "length(x) == 1L");
    }

    #[test]
    fn scalar_character() {
        for ty_str in &["String", "char", "PathBuf"] {
            let asserts = assertions_for(ty_str, "s");
            assert_eq!(asserts.len(), 2);
            assert_eq!(asserts[0].message(), "'s' must be character");
            assert_eq!(asserts[0].condition, "is.character(s)");
        }
    }

    #[test]
    fn ref_str() {
        let ty: syn::Type = syn::parse_str("& str").unwrap();
        let asserts = r_check_for_type(&ty).unwrap().assertions("s");
        assert_eq!(asserts.len(), 2);
        assert_eq!(asserts[0].condition, "is.character(s)");
    }

    #[test]
    fn scalar_raw() {
        let asserts = assertions_for("u8", "x");
        assert_eq!(asserts.len(), 2);
        assert_eq!(asserts[0].message(), "'x' must be raw");
        assert_eq!(asserts[0].condition, "is.raw(x)");
    }

    #[test]
    fn vector_float_stays_loose() {
        // `Vec<f32>` coerces from every numeric source, so it keeps the loose predicate.
        let asserts = assertions_for("Vec<f32>", "x");
        assert_eq!(asserts.len(), 1);
        assert_eq!(
            asserts[0].condition,
            "is.numeric(x) || is.logical(x) || is.raw(x)"
        );
    }

    #[test]
    fn vector_i32_is_intsxp_strict() {
        // `Vec<i32>` is the only INTSXP-only integer vector: the native inbound
        // conversion rejects REALSXP, so the R gate is `is.integer(x)` — a clean
        // boundary rejection for any double (issue #616).
        let asserts = assertions_for("Vec<i32>", "x");
        assert_eq!(asserts.len(), 1);
        assert_eq!(asserts[0].condition, "is.integer(x)");
        assert_eq!(asserts[0].message(), "'x' must be an integer vector");
    }

    #[test]
    fn vector_integer_wide_is_lossless() {
        // Every non-i32 integer element type uses the coercing inbound path that
        // accepts whole REALSXP and rejects fractional → lossless whole-number gate.
        for ty_str in &[
            "Vec<i8>",
            "Vec<i16>",
            "Vec<u16>",
            "Vec<u32>",
            "Vec<i64>",
            "Vec<u64>",
            "Vec<isize>",
            "Vec<usize>",
        ] {
            let asserts = assertions_for(ty_str, "x");
            assert_eq!(asserts.len(), 1, "{} should produce 1 assertion", ty_str);
            assert_eq!(
                asserts[0].condition,
                "is.integer(x) || is.logical(x) || is.raw(x) || \
                 (is.numeric(x) && all(is.na(x) | x == trunc(x)))",
                "{}",
                ty_str
            );
            assert_eq!(
                asserts[0].message(),
                "'x' must be integer or whole-number numeric"
            );
        }
    }

    #[test]
    fn slice_int_is_strict() {
        let ty: syn::Type = syn::parse_str("& [i32]").unwrap();
        let asserts = r_check_for_type(&ty).unwrap().assertions("x");
        assert_eq!(asserts.len(), 1);
        assert_eq!(asserts[0].condition, "is.integer(x)");
    }

    #[test]
    fn coerced_native_int_widens_to_whole_number_gate() {
        let ty = parse_type("i32");
        let asserts = coerce_widened(r_check_for_type(&ty).unwrap(), &ty).assertions("x");
        assert_eq!(asserts.len(), 2);
        assert_eq!(
            asserts[0].message(),
            "'x' must be integer or whole-number numeric"
        );
        assert!(asserts[0].condition.contains("x == trunc(x)"));
        assert_eq!(asserts[1].condition, "length(x) == 1L");

        let ty = parse_type("Vec<i32>");
        let asserts = coerce_widened(r_check_for_type(&ty).unwrap(), &ty).assertions("x");
        assert_eq!(asserts.len(), 1);
        assert!(asserts[0].condition.contains("x == trunc(x)"));

        // Native doubles move from `is.double` to the loose numeric gate;
        // borrowed slices have no mapping and stay strict.
        for name in ["f64", "Vec<f64>"] {
            let ty = parse_type(name);
            let actual = coerce_widened(r_check_for_type(&ty).unwrap(), &ty).assertions("x");
            assert_eq!(
                actual[0].condition, "is.numeric(x) || is.logical(x) || is.raw(x)",
                "{name}"
            );
        }
        let ty = parse_type("&[i32]");
        let actual = coerce_widened(r_check_for_type(&ty).unwrap(), &ty).assertions("x");
        assert_eq!(actual[0].condition, "is.integer(x)");
    }

    #[test]
    fn coerced_numeric_and_optional_checks_are_preserved() {
        for name in ["Vec<u16>", "Vec<f32>", "Option<bool>"] {
            let ty = parse_type(name);
            let expected = r_check_for_type(&ty).unwrap().assertions("x");
            let actual = coerce_widened(r_check_for_type(&ty).unwrap(), &ty).assertions("x");
            assert_eq!(actual.len(), expected.len());
            for (actual, expected) in actual.iter().zip(&expected) {
                assert_eq!(actual.condition, expected.condition);
            }
        }
    }

    #[test]
    fn coerced_bool_checks_accept_logical_and_integer() {
        for name in ["bool", "Vec<bool>"] {
            let ty = parse_type(name);
            let actual = coerce_widened(r_check_for_type(&ty).unwrap(), &ty).assertions("x");
            assert_eq!(actual[0].condition, "is.logical(x) || is.integer(x)");
            assert_eq!(actual.len(), if name == "bool" { 2 } else { 1 });
        }
    }

    #[test]
    fn build_checks_coerced_param_preserves_whole_number_check() {
        let sig: syn::Signature = syn::parse_str("fn f(x: Vec<u16>)").unwrap();
        let mut coerce_params = HashSet::new();
        coerce_params.insert("x".to_string());
        let opts = PreconditionOptions {
            coerce_params,
            ..Default::default()
        };
        let output = build_precondition_checks(&sig.inputs, &HashSet::new(), &opts);
        let joined = output.guards(None).join("\n");
        assert!(joined.contains("is.integer(x)"));
        assert!(joined.contains("trunc(x)"));
    }

    /// Every check is one guard raising the shared argument error (#1591):
    /// the wrapper's own call by default, the captured `.mx_call` under
    /// `call = caller` (#1548). The requirement leaves out the parameter name,
    /// which the helper puts in front.
    #[test]
    fn guards_raise_the_argument_error_with_the_given_call() {
        let sig: syn::Signature = syn::parse_str("fn f(n: i32, xs: Vec<f64>)").unwrap();
        let output = build_precondition_checks(
            &sig.inputs,
            &HashSet::new(),
            &PreconditionOptions::default(),
        );
        assert_eq!(
            output.guards(None),
            vec![
                "if (!isTRUE(is.integer(n))) .miniextendr_arg_error(\"n\", \"must be integer\")",
                "if (!isTRUE(length(n) == 1L)) .miniextendr_arg_error(\"n\", \"must have length 1\")",
                "if (!isTRUE(is.double(xs))) .miniextendr_arg_error(\"xs\", \"must be double\")",
            ]
        );
        assert_eq!(
            output.guards(Some(".mx_call")),
            vec![
                "if (!isTRUE(is.integer(n))) .miniextendr_arg_error(\"n\", \"must be integer\", .mx_call)",
                "if (!isTRUE(length(n) == 1L)) .miniextendr_arg_error(\"n\", \"must have length 1\", .mx_call)",
                "if (!isTRUE(is.double(xs))) .miniextendr_arg_error(\"xs\", \"must be double\", .mx_call)",
            ]
        );
        let no_params: syn::Signature = syn::parse_str("fn g()").unwrap();
        let empty = build_precondition_checks(
            &no_params.inputs,
            &HashSet::new(),
            &PreconditionOptions::default(),
        );
        assert!(empty.guards(None).is_empty());
        assert!(empty.guards(Some(".mx_call")).is_empty());
    }

    /// The conversion-side expectation (#1591) reads the same classification:
    /// scalars name one value, vectors the R type, `Option` adds `NULL`,
    /// `coerce` widens with the gate, and types without a check have none.
    #[test]
    fn conversion_expectation_follows_the_type_check() {
        let exp = |ty: &str, coerced: bool| conversion_expectation(&parse_type(ty), coerced);
        let some = |s: &str| Some(s.to_string());
        assert_eq!(exp("AsNumericVec", false), some("numeric"));
        assert_eq!(exp("AsNumeric", false), some("a single number"));
        assert_eq!(
            exp("Option<AsNumeric>", false),
            some("NULL or a single number")
        );
        assert_eq!(exp("i32", false), some("a single integer"));
        assert_eq!(exp("i32", true), some("a single whole number"));
        assert_eq!(exp("f64", false), some("a single double"));
        assert_eq!(exp("f64", true), some("a single number"));
        assert_eq!(exp("i64", false), some("a single number"));
        assert_eq!(exp("u32", false), some("a single non-negative number"));
        assert_eq!(exp("bool", false), some("TRUE or FALSE"));
        assert_eq!(exp("Option<bool>", false), some("NULL, TRUE or FALSE"));
        assert_eq!(exp("bool", true), some("a single logical or integer"));
        assert_eq!(exp("String", false), some("a single string"));
        assert_eq!(exp("&str", false), some("a single string"));
        assert_eq!(exp("u8", false), some("a single raw value"));
        assert_eq!(exp("Rcomplex", false), some("a single complex value"));
        assert_eq!(exp("Vec<i32>", false), some("integer"));
        assert_eq!(
            exp("Vec<i32>", true),
            some("integer or whole-number numeric")
        );
        assert_eq!(
            exp("Vec<u16>", false),
            some("integer or whole-number numeric")
        );
        assert_eq!(exp("Vec<f64>", false), some("double"));
        assert_eq!(exp("&[f64]", false), some("double"));
        assert_eq!(exp("Vec<Option<String>>", false), some("character"));
        assert_eq!(exp("Vec<bool>", true), some("logical or integer"));
        assert_eq!(exp("HashMap<String, i32>", false), some("a list"));
        assert_eq!(exp("Missing<f64>", false), some("a single double"));
        // Choice parameters, including the `Missing` / `Option` / `Either`
        // layers of #1551, have no expectation: their conversion errors read
        // `invalid '<p>' argument: <reason>`.
        for unknown in [
            "Hyperparams",
            "Either<i32, String>",
            "AsFromStrVec<i32>",
            "SEXP",
            "Mode",
            "Option<Mode>",
            "Missing<Option<Mode>>",
            "Missing<Vec<Mode>>",
            "Either<Route, DataFrame>",
            "Option<Either<Route, DataFrame>>",
            "Missing<Option<Either<Mode, List>>>",
        ] {
            assert_eq!(exp(unknown, false), None, "{unknown}");
        }
    }

    #[test]
    fn vector_character() {
        let asserts = assertions_for("Vec<String>", "x");
        assert_eq!(asserts.len(), 1);
        assert_eq!(asserts[0].condition, "is.character(x)");
    }

    #[test]
    fn vector_optional_string() {
        let asserts = assertions_for("Vec<Option<String>>", "x");
        assert_eq!(asserts.len(), 1);
        assert_eq!(asserts[0].condition, "is.character(x)");
    }

    #[test]
    fn slice_u8() {
        let ty: syn::Type = syn::parse_str("& [u8]").unwrap();
        let asserts = r_check_for_type(&ty).unwrap().assertions("x");
        assert_eq!(asserts.len(), 1);
        assert_eq!(asserts[0].condition, "is.raw(x)");
    }

    #[test]
    fn nullable_wraps_inner_assertions() {
        let asserts = assertions_for("Option<i64>", "x");
        assert_eq!(asserts.len(), 2);
        assert_eq!(
            asserts[0].message(),
            "'x' must be NULL or numeric, logical, or raw"
        );
        assert_eq!(
            asserts[0].condition,
            "is.null(x) || is.numeric(x) || is.logical(x) || is.raw(x)"
        );
        let asserts = assertions_for("Option<i32>", "x");
        assert_eq!(asserts[0].condition, "is.null(x) || is.integer(x)");
        assert_eq!(asserts[1].message(), "'x' must be NULL or have length 1");
        assert_eq!(asserts[1].condition, "is.null(x) || length(x) == 1L");
    }

    #[test]
    fn nullable_character() {
        let asserts = assertions_for("Option<String>", "s");
        assert_eq!(asserts.len(), 2);
        assert_eq!(asserts[0].message(), "'s' must be NULL or character");
        assert_eq!(asserts[0].condition, "is.null(s) || is.character(s)");
        assert_eq!(asserts[1].message(), "'s' must be NULL or have length 1");
    }

    #[test]
    fn as_numeric_markers_admit_numbers_text_and_factors() {
        let condition = "is.numeric(x) || is.logical(x) || is.character(x) || is.factor(x)";
        let asserts = assertions_for("AsNumeric", "x");
        assert_eq!(asserts.len(), 2);
        assert_eq!(
            asserts[0].message(),
            "'x' must be numeric, logical, character, or factor"
        );
        assert_eq!(asserts[0].condition, condition);
        assert_eq!(asserts[1].message(), "'x' must have length 1");
        assert_eq!(asserts[1].condition, "length(x) == 1L");

        // Path-qualified spellings resolve by their last segment.
        let asserts = assertions_for("miniextendr_api::AsNumericVec", "x");
        assert_eq!(asserts.len(), 1);
        assert_eq!(
            asserts[0].message(),
            "'x' must be numeric, logical, character, or factor"
        );
        assert_eq!(asserts[0].condition, condition);
    }

    #[test]
    fn optional_as_numeric_is_nullable() {
        let asserts = assertions_for("Option<AsNumeric>", "x");
        assert_eq!(asserts.len(), 2);
        assert_eq!(
            asserts[0].message(),
            "'x' must be NULL or numeric, logical, character, or factor"
        );
        assert_eq!(
            asserts[0].condition,
            "is.null(x) || is.numeric(x) || is.logical(x) || is.character(x) || is.factor(x)"
        );
        assert_eq!(asserts[1].message(), "'x' must be NULL or have length 1");
        let asserts = assertions_for("Option<AsNumericVec>", "x");
        assert_eq!(asserts.len(), 1);
        assert!(asserts[0].condition.starts_with("is.null(x) || "));
    }

    #[test]
    fn as_character_markers_admit_atomic_vectors() {
        let asserts = assertions_for("AsCharacter", "x");
        assert_eq!(asserts.len(), 2);
        assert_eq!(asserts[0].message, "'x' must be atomic");
        assert_eq!(asserts[0].condition, "is.atomic(x)");
        assert_eq!(asserts[1].message, "'x' must have length 1");
        assert_eq!(asserts[1].condition, "length(x) == 1L");

        // Path-qualified spellings resolve by their last segment.
        let asserts = assertions_for("miniextendr_api::AsCharacterVec", "x");
        assert_eq!(asserts.len(), 1);
        assert_eq!(asserts[0].message, "'x' must be atomic");
        assert_eq!(asserts[0].condition, "is.atomic(x)");
    }

    #[test]
    fn optional_as_character_is_nullable() {
        let asserts = assertions_for("Option<AsCharacter>", "x");
        assert_eq!(asserts.len(), 2);
        assert_eq!(asserts[0].message, "'x' must be NULL or atomic");
        assert_eq!(asserts[0].condition, "is.null(x) || is.atomic(x)");
        assert_eq!(asserts[1].message, "'x' must be NULL or have length 1");
        let asserts = assertions_for("Option<AsCharacterVec>", "x");
        assert_eq!(asserts.len(), 1);
        assert_eq!(asserts[0].message, "'x' must be NULL or atomic");
        assert_eq!(asserts[0].condition, "is.null(x) || is.atomic(x)");
    }

    #[test]
    fn as_character_vec_with_no_na_checks_type_then_na() {
        let out = explicit_output("fn f(x: AsCharacterVec)", &[("x", no_na())], false);
        assert_eq!(
            out.static_checks,
            vec![
                "stopifnot(",
                "  \"'x' must be atomic\" = is.atomic(x),",
                "  \"'x' must not be NA\" = !anyNA(x)",
                ")",
            ]
        );
    }

    #[test]
    fn coerce_keeps_the_as_numeric_gate() {
        // `AsNumeric` has no coercion mapping: `coerce` leaves its gate alone.
        let ty = parse_type("AsNumericVec");
        let check = coerce_widened(r_check_for_type(&ty).unwrap(), &ty);
        let asserts = check.assertions("x");
        assert_eq!(asserts.len(), 1);
        assert!(asserts[0].condition.contains("is.factor(x)"));
    }

    #[test]
    fn map_types() {
        for ty_str in &["HashMap<String, i32>", "BTreeMap<String, f64>"] {
            let ty = parse_type(ty_str);
            let asserts = r_check_for_type(&ty).unwrap().assertions("x");
            assert_eq!(asserts.len(), 1);
            assert_eq!(asserts[0].condition, "is.list(x)");
        }
    }

    #[test]
    fn skip_types() {
        for ty_str in &["SEXP", "ExternalPtr<MyType>"] {
            let ty = parse_type(ty_str);
            assert!(
                r_check_for_type(&ty).is_none(),
                "{} should be skipped",
                ty_str
            );
        }
    }

    #[test]
    fn scalar_param_produces_one_guard_per_check() {
        let sig: syn::Signature = syn::parse_str("fn f(n: i32)").unwrap();
        let output = build_precondition_checks(
            &sig.inputs,
            &HashSet::new(),
            &PreconditionOptions::default(),
        );
        let checks = output.guards(None);
        assert_eq!(checks.len(), 2);
        assert!(checks[0].contains("\"must be integer\""));
        assert!(checks[1].contains("\"must have length 1\""));
        assert!(output.fallback_params.is_empty());
    }

    #[test]
    fn vector_param_single_guard() {
        // Vec<f64> produces 1 check → one guard line
        let sig: syn::Signature = syn::parse_str("fn f(x: Vec<f64>)").unwrap();
        let output = build_precondition_checks(
            &sig.inputs,
            &HashSet::new(),
            &PreconditionOptions::default(),
        );
        assert_eq!(
            output.guards(None),
            vec!["if (!isTRUE(is.double(x))) .miniextendr_arg_error(\"x\", \"must be double\")"]
        );
    }

    #[test]
    fn two_scalar_params_produce_four_guards_in_order() {
        let sig: syn::Signature = syn::parse_str("fn f(a: i32, b: f64)").unwrap();
        let output = build_precondition_checks(
            &sig.inputs,
            &HashSet::new(),
            &PreconditionOptions::default(),
        );
        let checks = output.guards(None);
        // 2 checks per param
        assert_eq!(checks.len(), 4);
        assert!(checks[0].contains("(\"a\", ") && checks[0].contains("integer"));
        assert!(checks[1].contains("(\"a\", ") && checks[1].contains("length 1"));
        assert!(checks[2].contains("(\"b\", ") && checks[2].contains("double"));
        assert!(checks[3].contains("(\"b\", ") && checks[3].contains("length 1"));
    }

    #[test]
    fn build_checks_skips_match_arg() {
        let sig: syn::Signature = syn::parse_str("fn f(n: i32, mode: String)").unwrap();
        let mut skip = HashSet::new();
        skip.insert("mode".to_string());
        let output = build_precondition_checks(&sig.inputs, &skip, &PreconditionOptions::default());
        // Only n's 2 checks remain
        let joined = output.guards(None).join("\n");
        assert!(joined.contains("(\"n\", "));
        assert!(!joined.contains("mode"));
    }

    #[test]
    fn unknown_type_produces_fallback() {
        let sig: syn::Signature = syn::parse_str("fn f(x: MyCustomType)").unwrap();
        let output = build_precondition_checks(
            &sig.inputs,
            &HashSet::new(),
            &PreconditionOptions::default(),
        );
        assert!(output.guards(None).is_empty());
        assert_eq!(output.fallback_params.len(), 1);
        assert_eq!(output.fallback_params[0].r_name, "x");
    }

    #[test]
    fn mixed_known_and_unknown_types() {
        let sig: syn::Signature = syn::parse_str("fn f(a: i32, b: MyType, c: String)").unwrap();
        let output = build_precondition_checks(
            &sig.inputs,
            &HashSet::new(),
            &PreconditionOptions::default(),
        );
        // a (i32) and c (String) are known → checks
        let joined = output.guards(None).join("\n");
        assert!(joined.contains("(\"a\", "));
        assert!(joined.contains("(\"c\", "));
        assert!(!joined.contains("(\"b\", "));
        // b (MyType) is unknown → fallback
        assert_eq!(output.fallback_params.len(), 1);
        assert_eq!(output.fallback_params[0].r_name, "b");
    }

    #[test]
    fn sexp_not_fallback() {
        let sig: syn::Signature = syn::parse_str("fn f(x: SEXP)").unwrap();
        let output = build_precondition_checks(
            &sig.inputs,
            &HashSet::new(),
            &PreconditionOptions::default(),
        );
        assert!(output.guards(None).is_empty());
        assert!(output.fallback_params.is_empty());
    }

    /// Build checks for `sig` with the given explicit checks keyed by R name.
    fn explicit_output(
        sig: &str,
        explicit: &[(&str, ExplicitChecks)],
        no_type_checks: bool,
    ) -> PreconditionOutput {
        let sig: syn::Signature = syn::parse_str(sig).unwrap();
        let opts = PreconditionOptions {
            explicit: explicit
                .iter()
                .map(|(name, checks)| (name.to_string(), checks.clone()))
                .collect(),
            no_type_checks,
            ..Default::default()
        };
        build_precondition_checks(&sig.inputs, &HashSet::new(), &opts)
    }

    fn inherits(classes: &[&str]) -> ExplicitChecks {
        ExplicitChecks {
            inherits: Some(classes.iter().map(|c| c.to_string()).collect()),
            ..Default::default()
        }
    }

    fn no_na() -> ExplicitChecks {
        ExplicitChecks {
            no_na: true,
            ..Default::default()
        }
    }

    fn with_inherits_message(classes: &[&str], message: &str) -> ExplicitChecks {
        ExplicitChecks {
            inherits_message: Some(message.to_string()),
            ..inherits(classes)
        }
    }

    fn with_no_na_message(message: &str) -> ExplicitChecks {
        ExplicitChecks {
            no_na_message: Some(message.to_string()),
            ..no_na()
        }
    }

    #[test]
    fn inherits_follows_the_type_check() {
        let out = explicit_output("fn f(x: List)", &[("x", inherits(&["pkg_obj"]))], false);
        assert_eq!(
            out.guards(None),
            vec![
                "if (!isTRUE(is.list(x))) .miniextendr_arg_error(\"x\", \"must be a list\")",
                "if (!isTRUE(inherits(x, \"pkg_obj\"))) .miniextendr_arg_error(\"x\", \"must inherit from 'pkg_obj'\")",
            ]
        );
    }

    #[test]
    fn inherits_any_of_several_classes() {
        let out = explicit_output("fn f(x: SEXP)", &[("x", inherits(&["a", "b", "c"]))], false);
        assert_eq!(
            out.guards(None),
            vec![
                "if (!isTRUE(inherits(x, c(\"a\", \"b\", \"c\")))) .miniextendr_arg_error(\"x\", \"must inherit from 'a', 'b' or 'c'\")"
            ]
        );
    }

    #[test]
    fn inherits_escapes_quotes_and_backslashes() {
        let out = explicit_output("fn f(x: SEXP)", &[("x", inherits(&["a\"b\\c"]))], false);
        assert_eq!(
            out.guards(None),
            vec![
                "if (!isTRUE(inherits(x, \"a\\\"b\\\\c\"))) .miniextendr_arg_error(\"x\", \"must inherit from 'a\\\"b\\\\c'\")"
            ]
        );
    }

    #[test]
    fn no_na_wording_follows_the_shape() {
        let scalar = explicit_output("fn f(x: f64)", &[("x", no_na())], true);
        assert_eq!(
            scalar.guards(None),
            vec!["if (!isTRUE(!anyNA(x))) .miniextendr_arg_error(\"x\", \"must not be NA\")"]
        );
        let vector = explicit_output("fn f(x: Vec<f64>)", &[("x", no_na())], true);
        assert_eq!(
            vector.guards(None),
            vec!["if (!isTRUE(!anyNA(x))) .miniextendr_arg_error(\"x\", \"must not contain NA\")"]
        );
        let slice = explicit_output("fn f(x: &[f64])", &[("x", no_na())], true);
        assert!(slice.guards(None)[0].contains("must not contain NA"));
    }

    #[test]
    fn explicit_checks_pass_null_and_missing() {
        let optional = explicit_output(
            "fn f(x: Option<List>)",
            &[("x", inherits(&["pkg_obj"]))],
            true,
        );
        assert_eq!(
            optional.guards(None),
            vec![
                "if (!isTRUE(is.null(x) || inherits(x, \"pkg_obj\"))) .miniextendr_arg_error(\"x\", \"must inherit from 'pkg_obj'\")"
            ]
        );
        let missing = explicit_output("fn f(x: Missing<f64>)", &[("x", no_na())], true);
        assert_eq!(
            missing.guards(None),
            vec![
                "if (!isTRUE(missing(x) || !anyNA(x))) .miniextendr_arg_error(\"x\", \"must not be NA\")"
            ]
        );
        let optional_vec = explicit_output("fn f(x: Option<Vec<f64>>)", &[("x", no_na())], true);
        assert!(optional_vec.guards(None)[0].contains("must not contain NA"));
    }

    #[test]
    fn explicit_checks_survive_no_type_checks() {
        let both = ExplicitChecks {
            inherits: Some(vec!["pkg_obj".into()]),
            no_na: true,
            ..Default::default()
        };
        let out = explicit_output("fn f(n: i32, x: List)", &[("x", both)], true);
        // `n`'s type checks and `x`'s `is.list()` are gone; the named checks stay,
        // NA first.
        assert_eq!(
            out.guards(None),
            vec![
                "if (!isTRUE(!anyNA(x))) .miniextendr_arg_error(\"x\", \"must not be NA\")",
                "if (!isTRUE(inherits(x, \"pkg_obj\"))) .miniextendr_arg_error(\"x\", \"must inherit from 'pkg_obj'\")",
            ]
        );
        let none = explicit_output("fn f(n: i32)", &[], true);
        assert!(none.guards(None).is_empty());
    }

    #[test]
    fn explicit_checks_apply_to_match_arg_params() {
        let sig: syn::Signature = syn::parse_str("fn f(mode: String)").unwrap();
        let opts = PreconditionOptions {
            explicit: [("mode".to_string(), no_na())].into_iter().collect(),
            ..Default::default()
        };
        let skip: HashSet<String> = ["mode".to_string()].into_iter().collect();
        let out = build_precondition_checks(&sig.inputs, &skip, &opts);
        assert_eq!(
            out.guards(None),
            vec!["if (!isTRUE(!anyNA(mode))) .miniextendr_arg_error(\"mode\", \"must not be NA\")"]
        );
    }

    #[test]
    fn explicit_checks_take_the_caller_call() {
        let out = explicit_output("fn f(x: f64)", &[("x", no_na())], true);
        assert_eq!(
            out.guards(Some(".mx_call")),
            vec![
                "if (!isTRUE(!anyNA(x))) .miniextendr_arg_error(\"x\", \"must not be NA\", .mx_call)"
            ]
        );
    }

    #[test]
    fn explicit_checks_merge() {
        let mut a = inherits(&["a"]);
        a.merge(no_na()).unwrap();
        a.merge(inherits(&["b"])).unwrap();
        assert_eq!(
            a,
            ExplicitChecks {
                inherits: Some(vec!["a".into(), "b".into()]),
                no_na: true,
                ..Default::default()
            }
        );
        assert!(ExplicitChecks::default().is_empty());

        // One message per check: it may come from either attribute, not both.
        let mut b = inherits(&["a"]);
        b.merge(with_inherits_message(&["b"], "one")).unwrap();
        assert_eq!(b.inherits_message.as_deref(), Some("one"));
        assert_eq!(b.inherits, Some(vec!["a".into(), "b".into()]));
        b.merge(with_no_na_message("two")).unwrap();
        assert_eq!(b.no_na_message.as_deref(), Some("two"));
        let err = b.merge(with_inherits_message(&["c"], "three")).unwrap_err();
        assert!(
            err.contains("`inherits`") && err.contains("more than one `message`"),
            "{err}"
        );
        let err = b.merge(with_no_na_message("four")).unwrap_err();
        assert!(err.contains("`no_na`"), "{err}");
    }

    /// The author's message replaces `'<p>' <requirement>` verbatim: the
    /// helper gets it as `message = `, and the call, when there is one, by
    /// name too (after a named `message` a positional call would bind to
    /// `what`). One message covers every class of the check.
    #[test]
    fn custom_messages_are_passed_verbatim_by_name() {
        let checks = ExplicitChecks {
            inherits: Some(vec!["pkg_a".into(), "pkg_b".into()]),
            inherits_message: Some("`model` must be a `pkg_model`; see pkg_model().".into()),
            no_na: true,
            no_na_message: Some("no NA in `model`".into()),
        };
        let out = explicit_output("fn f(model: List)", &[("model", checks)], false);
        assert_eq!(
            out.guards(None),
            vec![
                "if (!isTRUE(is.list(model))) .miniextendr_arg_error(\"model\", \"must be a list\")",
                "if (!isTRUE(!anyNA(model))) .miniextendr_arg_error(\"model\", message = \"no NA in `model`\")",
                "if (!isTRUE(inherits(model, c(\"pkg_a\", \"pkg_b\")))) .miniextendr_arg_error(\"model\", message = \"`model` must be a `pkg_model`; see pkg_model().\")",
            ]
        );
        let caller = out.guards(Some(".mx_call"));
        assert!(
            caller[0].ends_with(".miniextendr_arg_error(\"model\", \"must be a list\", .mx_call)"),
            "{}",
            caller[0]
        );
        assert!(
            caller[1].ends_with(
                ".miniextendr_arg_error(\"model\", message = \"no NA in `model`\", call = .mx_call)"
            ),
            "{}",
            caller[1]
        );
        // The `Option` / `Missing` guards are unchanged.
        let optional = explicit_output(
            "fn f(x: Option<List>)",
            &[("x", with_inherits_message(&["pkg_obj"], "need a pkg_obj"))],
            true,
        );
        assert_eq!(
            optional.guards(None),
            vec![
                "if (!isTRUE(is.null(x) || inherits(x, \"pkg_obj\"))) .miniextendr_arg_error(\"x\", message = \"need a pkg_obj\")"
            ]
        );
        let asserts = with_no_na_message("mine").assertions("x", &parse_type("f64"));
        assert_eq!(asserts[0].message(), "mine");
    }

    /// A message is an R string literal: quotes, backslashes, newlines, tabs
    /// and control characters are escaped, and non-ASCII text becomes
    /// `\u{..}` / `\U{..}` escapes (R code in a package must be ASCII). `%`
    /// needs nothing: the message never goes through `sprintf()`.
    #[test]
    fn custom_messages_are_escaped_as_r_string_literals() {
        let message = "say \"hi\" \\ back\nline\ttab 100% caf\u{e9} \u{1F600} bell\u{7}";
        let out = explicit_output("fn f(x: f64)", &[("x", with_no_na_message(message))], true);
        assert_eq!(
            out.guards(None),
            vec![
                r#"if (!isTRUE(!anyNA(x))) .miniextendr_arg_error("x", message = "say \"hi\" \\ back\nline\ttab 100% caf\u{e9} \U{1f600} bell\u{7}")"#
            ]
        );
        assert_eq!(r_string_escape("a\rb"), r"a\rb");
        assert_eq!(r_string_escape("plain 'text'"), "plain 'text'");
        // Class names take the same escaping.
        let out = explicit_output("fn f(x: SEXP)", &[("x", inherits(&["caf\u{e9}"]))], true);
        assert_eq!(
            out.guards(None),
            vec![
                r#"if (!isTRUE(inherits(x, "caf\u{e9}"))) .miniextendr_arg_error("x", "must inherit from 'caf\u{e9}'")"#
            ]
        );
    }
}
