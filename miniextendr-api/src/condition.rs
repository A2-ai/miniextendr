//! Condition macros and signal enum for the Rust→R condition pipeline.
//!
//! This module provides two things:
//!
//! 1. **[`RCondition`] enum** — the internal panic payload used by `error!()`,
//!    `warning!()`, `message!()`, and `condition!()` macros. Caught by
//!    [`crate::unwind_protect::with_r_unwind_protect`] before the generic
//!    panic→error path, then forwarded to R as a structured condition with
//!    `rust_*` class layering via
//!    [`crate::error_value::make_rust_condition_value`].
//!
//! 2. **[`AsRError`] struct** — wraps any `E: std::error::Error` and
//!    preserves the full error chain (cause/source) when converting to an R
//!    error message. Use as the `Err` type in `Result` returns.
//!
//! # When to reach for what
//!
//! There are three Rust→R error-emission paths and they are not
//! interchangeable. The crate-level rationale (why tagged-SEXP at all, what
//! `error_in_r` defaults imply, and the `with_r_unwind_protect` leak) lives
//! on [`crate::error_value`]; the practical picking-one summary:
//!
//! - **`panic!`** — escape hatch. Becomes class `c("rust_error",
//!   "simpleError", "error", "condition")` with `kind = "panic"`. Use for
//!   genuine bugs or impossible states. Cheapest in source; coarsest in R
//!   (callers can only match `rust_error` / `error`, not a specific class).
//! - **`error!` / `warning!` / `message!` / `condition!`** (this module) —
//!   typed conditions. Same transport, but allow an optional `class =
//!   "name"` so R-side `tryCatch` can route by class. `warning!` /
//!   `message!` / `condition!` are the only way to emit non-error
//!   conditions; `panic!` is always an error.
//! - **`Result<T, E>` with [`AsRError<E>`]** — value-style propagation
//!   through Rust code. Converts at the boundary; `kind = "result_err"`.
//!   Best when the failure path is real-and-recoverable in Rust and the
//!   error chain (`std::error::Error::source`) is worth preserving.
//!
//! `Rf_error` is *not* on this list. Direct `Rf_error` skips Rust
//! destructors unconditionally and is forbidden by lint MXL300; see
//! [`crate::error_value`] for the full reasoning.
//!
//! # Macro-vs-module name collision
//!
//! `#[macro_export]` puts each macro at the *crate root*, where `error!` and
//! `condition!` collide with the same-named modules `pub mod error` and `pub
//! mod condition`. The practical implication: `use miniextendr_api::{error,
//! condition}` imports the *modules*, not the macros, and a subsequent
//! `error!(...)` call fails to resolve.
//!
//! Workarounds, in rough order of ergonomics:
//!
//! 1. Invoke via fully-qualified path: `miniextendr_api::error!("...")`.
//! 2. `use miniextendr_api as mx;` then `mx::error!("...")`.
//! 3. `warning!` and `message!` have no module conflict — `use
//!    miniextendr_api::{warning, message};` works directly.
//!
//! See the individual macro docs for the per-macro reminder.
//!
//! # Condition macros
//!
//! The four macros are the user-facing API for raising non-panic conditions from
//! Rust. They ride the tagged-condition transport that every `#[miniextendr]`
//! function uses:
//!
//! ```ignore
//! use miniextendr_api::{error, warning, message, condition};
//!
//! #[miniextendr]
//! fn demo_error() {
//!     error!("something went wrong: {}", 42);
//! }
//!
//! #[miniextendr]
//! fn demo_warning() {
//!     warning!("something looks suspicious");
//! }
//!
//! #[miniextendr]
//! fn demo_message() {
//!     message!("progress: {} of {}", 1, 10);
//! }
//!
//! #[miniextendr]
//! fn demo_condition() {
//!     condition!("a signallable condition");
//! }
//! ```
//!
//! Optional `class =` extension for programmatic catching:
//!
//! ```ignore
//! #[miniextendr]
//! fn typed_error(name: &str) {
//!     error!(class = "my_error", "missing field: {name}");
//! }
//! ```
//!
//! ```r
//! tryCatch(typed_error("x"), my_error = function(e) "caught!")
//! # [1] "caught!"
//! ```
//!
//! Optional `data =` extension attaches structured fields readable as
//! `e$<name>` in handlers (rlang-`abort()`-style):
//!
//! ```ignore
//! #[miniextendr]
//! fn validate(value: i32) {
//!     if !(0..=100).contains(&value) {
//!         miniextendr_api::error!(
//!             class = "validation_error",
//!             data = [("value", value), ("min", 0), ("max", 100)],
//!             "value {value} out of range"
//!         );
//!     }
//! }
//! ```
//!
//! ```r
//! tryCatch(validate(150L), validation_error = function(e) c(e$value, e$min, e$max))
//! # [1] 150   0 100
//! ```
//!
//! Supported `data` value types (v1): scalars and `Vec`s of `i32`, `f64`,
//! `bool`, `String` (see [`ConditionDataValue`]). The payload is built as a
//! Send-safe owned value at the call site and materialised as R objects on
//! the main thread — so `data =` works from worker-thread code too.
//!
//! # `AsRError`
//!
//! ```ignore
//! use miniextendr_api::condition::AsRError;
//!
//! #[miniextendr]
//! fn parse_config(path: &str) -> Result<i32, AsRError<std::io::Error>> {
//!     let content = std::fs::read_to_string(path).map_err(AsRError)?;
//!     Ok(content.len() as i32)
//! }
//! ```

// region: ConditionDataValue — Send-safe owned condition-data payload

/// A single condition-data field value.
///
/// This is the Send-safe owned representation of a value attached to a
/// condition via the macros' `data = ...` form. It exists because the
/// condition payload travels through `std::panic::panic_any` (which requires
/// `Send`) and the macro may fire on the worker thread, where a live `SEXP`
/// would be illegal to carry. The actual R object is materialised lazily in
/// [`crate::error_value::make_rust_condition_value`], which always runs on R's
/// main thread.
///
/// # Supported value types (v1)
///
/// Scalars and homogeneous vectors of `i32`, `f64`, `bool`, and `String`:
///
/// | Rust value passed to `data = (..)` | R element type |
/// |---|---|
/// | `i32` | `integer(1)` |
/// | `f64` | `double(1)` |
/// | `bool` | `logical(1)` |
/// | `&str` / `String` | `character(1)` |
/// | `Vec<i32>` | `integer(n)` |
/// | `Vec<f64>` | `double(n)` |
/// | `Vec<bool>` | `logical(n)` |
/// | `Vec<String>` | `character(n)` |
///
/// Anything outside this set is not supported in v1 — stringify it at the
/// call site (e.g. `format!("{x:?}")`) or attach the scalar fields you need
/// individually. Richer / arbitrary `IntoR` payloads across the thread
/// boundary are tracked as a follow-up; see the macro docs.
///
/// Users normally do not name this type — the `From` impls let the macros
/// accept the bare Rust value (`data = ("count", 7i32)`). It is `#[doc(hidden)]`
/// on the enum variants for that reason but the type itself is public so the
/// macro expansion can reference it.
#[doc(hidden)]
#[derive(Debug, Clone)]
pub enum ConditionDataValue {
    Int(i32),
    Real(f64),
    Bool(bool),
    Str(String),
    IntVec(Vec<i32>),
    RealVec(Vec<f64>),
    BoolVec(Vec<bool>),
    StrVec(Vec<String>),
}

impl ConditionDataValue {
    /// Materialise this value as an R SEXP.
    ///
    /// # Safety
    ///
    /// Must be called from R's main thread (delegates to `IntoR`). The
    /// returned SEXP is unprotected — the caller must protect it before the
    /// next allocation.
    pub fn into_sexp(self) -> crate::SEXP {
        use crate::IntoR;
        match self {
            ConditionDataValue::Int(v) => v.into_sexp(),
            ConditionDataValue::Real(v) => v.into_sexp(),
            ConditionDataValue::Bool(v) => v.into_sexp(),
            ConditionDataValue::Str(v) => v.into_sexp(),
            ConditionDataValue::IntVec(v) => v.into_sexp(),
            ConditionDataValue::RealVec(v) => v.into_sexp(),
            ConditionDataValue::BoolVec(v) => v.into_sexp(),
            ConditionDataValue::StrVec(v) => v.into_sexp(),
        }
    }
}

impl From<i32> for ConditionDataValue {
    fn from(v: i32) -> Self {
        ConditionDataValue::Int(v)
    }
}
impl From<f64> for ConditionDataValue {
    fn from(v: f64) -> Self {
        ConditionDataValue::Real(v)
    }
}
impl From<bool> for ConditionDataValue {
    fn from(v: bool) -> Self {
        ConditionDataValue::Bool(v)
    }
}
impl From<String> for ConditionDataValue {
    fn from(v: String) -> Self {
        ConditionDataValue::Str(v)
    }
}
impl From<&str> for ConditionDataValue {
    fn from(v: &str) -> Self {
        ConditionDataValue::Str(v.to_string())
    }
}
impl From<Vec<i32>> for ConditionDataValue {
    fn from(v: Vec<i32>) -> Self {
        ConditionDataValue::IntVec(v)
    }
}
impl From<Vec<f64>> for ConditionDataValue {
    fn from(v: Vec<f64>) -> Self {
        ConditionDataValue::RealVec(v)
    }
}
impl From<Vec<bool>> for ConditionDataValue {
    fn from(v: Vec<bool>) -> Self {
        ConditionDataValue::BoolVec(v)
    }
}
impl From<Vec<String>> for ConditionDataValue {
    fn from(v: Vec<String>) -> Self {
        ConditionDataValue::StrVec(v)
    }
}
impl From<Vec<&str>> for ConditionDataValue {
    fn from(v: Vec<&str>) -> Self {
        ConditionDataValue::StrVec(v.into_iter().map(|s| s.to_string()).collect())
    }
}

/// Named condition-data payload: an ordered list of `(name, value)` pairs.
///
/// Produced by the macros' `data = ...` form and consumed by
/// [`crate::error_value::make_rust_condition_value`]. Send-safe by
/// construction (every field is a [`ConditionDataValue`]).
pub type ConditionData = Vec<(String, ConditionDataValue)>;

// endregion

// region: RCondition enum — internal panic payload

/// Internal panic payload for structured R conditions.
///
/// Raised by the `error!()`, `warning!()`, `message!()`, and `condition!()`
/// macros via `std::panic::panic_any`. Caught by `with_r_unwind_protect`
/// before the generic panic→string path and forwarded to R as a tagged SEXP
/// with `rust_*` class layering.
///
/// This type is `#[doc(hidden)]` because users interact with the macros,
/// not the enum directly.
#[doc(hidden)]
#[derive(Debug)]
pub enum RCondition {
    /// Raised by `error!(...)` / `error!(class = "...", ...)`.
    Error {
        message: String,
        class: Option<String>,
        data: Option<ConditionData>,
    },
    /// Raised by `warning!(...)` / `warning!(class = "...", ...)`.
    Warning {
        message: String,
        class: Option<String>,
        data: Option<ConditionData>,
    },
    /// Raised by `message!(...)`.
    Message {
        message: String,
        data: Option<ConditionData>,
    },
    /// Raised by `condition!(...)` / `condition!(class = "...", ...)`.
    Condition {
        message: String,
        class: Option<String>,
        data: Option<ConditionData>,
    },
}

// endregion

// region: Macros

/// Internal: normalise a macro `data = ...` argument into
/// `Option<ConditionData>`. Not part of the public API.
///
/// Two forms are accepted:
/// - a single pair: `("name", value)`
/// - a bracketed list of pairs: `[("a", v1), ("b", v2)]`
///
/// Each `value` is converted via `ConditionDataValue::from`, so any type with
/// a `From` impl (the v1 scalar/vector set) works without ceremony.
#[doc(hidden)]
#[macro_export]
macro_rules! __mx_condition_data {
    (($name:expr, $value:expr) $(,)?) => {
        ::std::option::Option::Some(::std::vec![(
            ($name).to_string(),
            $crate::condition::ConditionDataValue::from($value),
        )])
    };
    ([ $(($name:expr, $value:expr)),* $(,)? ]) => {
        ::std::option::Option::Some(::std::vec![
            $(
                (
                    ($name).to_string(),
                    $crate::condition::ConditionDataValue::from($value),
                ),
            )*
        ])
    };
}

/// Raise an R error from Rust with `rust_error` class layering.
///
/// Rides the tagged-condition transport that every `#[miniextendr]` function uses.
/// The raised condition has class `c("rust_error", "simpleError", "error", "condition")`.
///
/// An optional `class = "name"` form prepends a custom class for programmatic catching:
/// `c("name", "rust_error", "simpleError", "error", "condition")`.
///
/// # Structured `data = ...` payloads
///
/// An optional `data = ...` form (after `class`, before the message) attaches
/// named fields to the condition object, rlang-`abort()`-style. Handlers read
/// them as `e$<name>` instead of parsing the message string:
///
/// ```ignore
/// // Single field:
/// mx::error!(class = "range_error", data = ("value", value), "value {value} out of range");
///
/// // Multiple fields (bracketed list of pairs):
/// mx::error!(
///     class = "validation_error",
///     data = [("value", value), ("min", 0), ("max", 100)],
///     "value {value} out of range"
/// );
/// ```
///
/// ```r
/// tryCatch(validate(150L), validation_error = function(e) c(e$value, e$min, e$max))
/// # [1] 150   0 100
/// ```
///
/// Argument order is fixed: `class = ...` (optional), then `data = ...`
/// (optional), then the format message.
///
/// **Supported value types (v1)**: scalars and `Vec`s of `i32`, `f64`, `bool`,
/// and `String` (plus `&str` / `Vec<&str>`, converted to owned). The payload
/// must be `Send` — it travels through `panic_any` and may cross the
/// worker→main thread boundary, so live `SEXP`s cannot ride along; the R
/// objects are materialised on the main thread at the unwind boundary.
/// Anything richer: stringify at the call site (`format!("{x:?}")`) or attach
/// the individual scalar fields you need. See
/// [`crate::condition::ConditionDataValue`] for the full conversion table.
///
/// # See also
///
/// - [`crate::warning!`] / [`crate::message!`] / [`crate::condition!`] — the
///   non-error sibling kinds (warning continues execution; message is muffled
///   by `suppressMessages`; condition is silent without a handler).
/// - [`std::panic!`] — escape hatch with the same `rust_error` class layering
///   but no custom-class slot. Use for true bugs / impossible states; reach for
///   `error!` when callers might want to route by class.
/// - [`AsRError`] — wraps `Result<_, E: std::error::Error>` for value-style
///   propagation through Rust code; converts at the boundary.
/// - [`crate::error_value`] — module-level rationale for the tagged-SEXP
///   transport and the `error_in_r` default.
///
/// **Name-collision note.** Because `pub mod error` exists at the crate root,
/// `use miniextendr_api::error` imports the module rather than this macro.
/// Invoke via `miniextendr_api::error!(...)` (fully qualified) or via
/// `mx::error!(...)` after `use miniextendr_api as mx;`.
///
/// # Examples
///
/// ```ignore
/// use miniextendr_api as mx;
///
/// #[miniextendr]
/// fn fail() {
///     mx::error!("something went wrong: {}", 42);
/// }
///
/// // With a custom class for tryCatch:
/// #[miniextendr]
/// fn typed_fail(name: &str) {
///     mx::error!(class = "my_error", "missing field: {name}");
/// }
/// ```
///
/// ```r
/// tryCatch(fail(), rust_error = function(e) conditionMessage(e))
/// # [1] "something went wrong: 42"
///
/// tryCatch(typed_fail("x"), my_error = function(e) "caught!")
/// # [1] "caught!"
/// ```
#[macro_export]
macro_rules! error {
    (class = $class:expr, data = $data:tt, $($arg:tt)*) => {
        ::std::panic::panic_any($crate::condition::RCondition::Error {
            message: ::std::format!($($arg)*),
            class: ::std::option::Option::Some($class.to_string()),
            data: $crate::__mx_condition_data!($data),
        })
    };
    (data = $data:tt, $($arg:tt)*) => {
        ::std::panic::panic_any($crate::condition::RCondition::Error {
            message: ::std::format!($($arg)*),
            class: ::std::option::Option::None,
            data: $crate::__mx_condition_data!($data),
        })
    };
    (class = $class:expr, $($arg:tt)*) => {
        ::std::panic::panic_any($crate::condition::RCondition::Error {
            message: ::std::format!($($arg)*),
            class: ::std::option::Option::Some($class.to_string()),
            data: ::std::option::Option::None,
        })
    };
    ($($arg:tt)*) => {
        ::std::panic::panic_any($crate::condition::RCondition::Error {
            message: ::std::format!($($arg)*),
            class: ::std::option::Option::None,
            data: ::std::option::Option::None,
        })
    };
}

/// Raise an R warning from Rust with `rust_warning` class layering.
///
/// Rides the tagged-condition transport that every `#[miniextendr]` function uses.
/// Unlike `panic!`, execution continues after `warning!` is caught by a handler.
/// The raised condition has class `c("rust_warning", "simpleWarning", "warning", "condition")`.
///
/// An optional `class = "name"` form prepends a custom class. An optional
/// `data = ...` form (after `class`, before the message) attaches named fields
/// readable as `w$<name>` in handlers — same grammar and supported value types
/// as [`crate::error!`] (see there for details):
///
/// ```ignore
/// warning!(class = "truncation", data = ("dropped", n), "dropped {n} rows");
/// ```
///
/// # See also
///
/// - [`crate::error!`] — fatal sibling; aborts the call instead of continuing.
/// - [`crate::message!`] / [`crate::condition!`] — softer signal kinds (muffled
///   by `suppressMessages` / silent without handler, respectively).
/// - [`std::panic!`] — escape hatch when "continue after this" is not a sensible
///   semantic.
/// - [`crate::error_value`] — tagged-SEXP transport rationale.
///
/// No name-collision caveat: there is no `pub mod warning`, so
/// `use miniextendr_api::warning;` then `warning!(...)` works directly.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::warning;
///
/// #[miniextendr]
/// fn maybe_warn(x: i32) -> i32 {
///     if x > 100 {
///         warning!("x is large: {x}");
///     }
///     x * 2
/// }
/// ```
///
/// ```r
/// withCallingHandlers(
///   maybe_warn(200L),
///   warning = function(w) { cat("saw:", conditionMessage(w)); invokeRestart("muffleWarning") }
/// )
/// # saw: x is large: 200
/// # [1] 400
/// ```
#[macro_export]
macro_rules! warning {
    (class = $class:expr, data = $data:tt, $($arg:tt)*) => {
        ::std::panic::panic_any($crate::condition::RCondition::Warning {
            message: ::std::format!($($arg)*),
            class: ::std::option::Option::Some($class.to_string()),
            data: $crate::__mx_condition_data!($data),
        })
    };
    (data = $data:tt, $($arg:tt)*) => {
        ::std::panic::panic_any($crate::condition::RCondition::Warning {
            message: ::std::format!($($arg)*),
            class: ::std::option::Option::None,
            data: $crate::__mx_condition_data!($data),
        })
    };
    (class = $class:expr, $($arg:tt)*) => {
        ::std::panic::panic_any($crate::condition::RCondition::Warning {
            message: ::std::format!($($arg)*),
            class: ::std::option::Option::Some($class.to_string()),
            data: ::std::option::Option::None,
        })
    };
    ($($arg:tt)*) => {
        ::std::panic::panic_any($crate::condition::RCondition::Warning {
            message: ::std::format!($($arg)*),
            class: ::std::option::Option::None,
            data: ::std::option::Option::None,
        })
    };
}

/// Emit an R message from Rust with `rust_message` class layering.
///
/// Rides the tagged-condition transport that every `#[miniextendr]` function uses.
/// The raised condition has class `c("rust_message", "simpleMessage", "message", "condition")`.
/// Muffled by `suppressMessages()` automatically (standard R restart mechanism).
///
/// An optional `data = ...` form (before the message) attaches named fields
/// readable as `m$<name>` in `withCallingHandlers` — same grammar and
/// supported value types as [`crate::error!`] (see there for details). There
/// is no `class =` form for `message!`.
///
/// # See also
///
/// - [`crate::warning!`] / [`crate::condition!`] — louder/quieter sibling kinds.
/// - [`crate::error!`] — for fatal failures.
/// - [`std::panic!`] — escape hatch.
/// - [`crate::error_value`] — tagged-SEXP transport rationale.
///
/// No name-collision caveat: there is no `pub mod message`, so
/// `use miniextendr_api::message;` then `message!(...)` works directly.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::message;
///
/// #[miniextendr]
/// fn log_step(step: i32) {
///     message!("step {} complete", step);
/// }
/// ```
///
/// ```r
/// log_step(3L)
/// # step 3 complete
///
/// suppressMessages(log_step(3L))  # no output
/// ```
#[macro_export]
macro_rules! message {
    (data = $data:tt, $($arg:tt)*) => {
        ::std::panic::panic_any($crate::condition::RCondition::Message {
            message: ::std::format!($($arg)*),
            data: $crate::__mx_condition_data!($data),
        })
    };
    ($($arg:tt)*) => {
        ::std::panic::panic_any($crate::condition::RCondition::Message {
            message: ::std::format!($($arg)*),
            data: ::std::option::Option::None,
        })
    };
}

/// Signal a generic R condition from Rust with `rust_condition` class layering.
///
/// Rides the tagged-condition transport that every `#[miniextendr]` function uses.
/// Unlike `error!`, a bare condition is a silent no-op if there is no handler.
/// The raised condition has class `c("rust_condition", "simpleCondition", "condition")`.
///
/// An optional `class = "name"` form prepends a custom class. An optional
/// `data = ...` form (after `class`, before the message) attaches named fields
/// readable as `c$<name>` in handlers — same grammar and supported value types
/// as [`crate::error!`] (see there for details).
///
/// # See also
///
/// - [`crate::error!`] / [`crate::warning!`] / [`crate::message!`] — louder
///   condition kinds. Pick `condition!` when "no handler = silent" is the
///   right default (progress events, structured logging hooks).
/// - [`std::panic!`] — escape hatch when the failure cannot be ignored.
/// - [`crate::error_value`] — tagged-SEXP transport rationale.
///
/// **Name-collision note.** Because `pub mod condition` exists at the crate
/// root, `use miniextendr_api::condition` imports the module rather than this
/// macro. Invoke via `miniextendr_api::condition!(...)` (fully qualified) or
/// via `mx::condition!(...)` after `use miniextendr_api as mx;`.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::condition;
///
/// #[miniextendr]
/// fn signal_progress(n: i32) {
///     condition!(class = "my_progress", "processed {n} items");
/// }
/// ```
///
/// ```r
/// withCallingHandlers(
///   signal_progress(42L),
///   my_progress = function(c) cat("progress:", conditionMessage(c), "\n")
/// )
/// # progress: processed 42 items
/// ```
#[macro_export]
macro_rules! condition {
    (class = $class:expr, data = $data:tt, $($arg:tt)*) => {
        ::std::panic::panic_any($crate::condition::RCondition::Condition {
            message: ::std::format!($($arg)*),
            class: ::std::option::Option::Some($class.to_string()),
            data: $crate::__mx_condition_data!($data),
        })
    };
    (data = $data:tt, $($arg:tt)*) => {
        ::std::panic::panic_any($crate::condition::RCondition::Condition {
            message: ::std::format!($($arg)*),
            class: ::std::option::Option::None,
            data: $crate::__mx_condition_data!($data),
        })
    };
    (class = $class:expr, $($arg:tt)*) => {
        ::std::panic::panic_any($crate::condition::RCondition::Condition {
            message: ::std::format!($($arg)*),
            class: ::std::option::Option::Some($class.to_string()),
            data: ::std::option::Option::None,
        })
    };
    ($($arg:tt)*) => {
        ::std::panic::panic_any($crate::condition::RCondition::Condition {
            message: ::std::format!($($arg)*),
            class: ::std::option::Option::None,
            data: ::std::option::Option::None,
        })
    };
}

// endregion

// region: from_tagged_sexp + repanic_if_rust_error — shim re-panic helpers

impl RCondition {
    /// Reconstruct an [`RCondition::Error`] from a tagged SEXP produced by
    /// [`crate::error_value::make_rust_condition_value`].
    ///
    /// Returns `Some(RCondition)` when `sexp` has class `"rust_condition_value"` AND
    /// the `"__rust_condition__"` attribute is `TRUE`. Returns `None` for all other
    /// SEXPs (normal return values, `R_NilValue`, etc.).
    ///
    /// Reconstructs the matching variant for each kind: `"error"`/`"panic"`/
    /// `"result_err"`/`"none_err"`/`"other_rust_error"` → [`RCondition::Error`];
    /// `"warning"` → [`RCondition::Warning`]; `"message"` → [`RCondition::Message`];
    /// `"condition"` → [`RCondition::Condition`]. Unknown kinds degrade to
    /// [`RCondition::Error`] with the kind string prefixed to the message.
    ///
    /// # Safety
    ///
    /// Must be called from R's main thread.
    pub unsafe fn from_tagged_sexp(sexp: crate::SEXP) -> Option<Self> {
        use crate::SexpExt;

        // Use SexpExt::inherits_class — wraps Rf_inherits, already main-thread.
        if !sexp.inherits_class(c"rust_condition_value") {
            return None;
        }

        // Belt-and-suspenders PROTECT across the full inspection window. The reads
        // below are nominally non-allocating, but R-devel's GC is aggressive enough
        // (see MEMORY.md "Common gotchas") that a defensive guard is cheap and
        // closes the door on subtle regressions if the read path ever changes.
        let _guard = unsafe { crate::gc_protect::OwnedProtect::new(sexp) };

        // Verify the __rust_condition__ marker attribute is TRUE (a length-1 LGLSXP
        // with value 1). This guards against coincidental class attribute collisions.
        let attr_sym = crate::cached_class::rust_condition_attr_symbol();
        let marker = sexp.get_attr(attr_sym);
        // marker should be a scalar logical TRUE: is_logical() and logical_elt(0) == 1
        if !marker.is_logical() || marker.logical_elt(0) != 1 {
            return None;
        }

        // It's a tagged SEXP. Read the elements.
        // Both 3-element (legacy) and 4-element (condition) forms have:
        //   [0] = error message (STRSXP)
        //   [1] = kind string (STRSXP)
        //   [2] = class name or NULL (only in 4-element form; absent in legacy)

        let len = sexp.len();

        // Defense-in-depth: a tagged SEXP must have at least the message and kind
        // slots. inherits_class + __rust_condition__ marker should already imply this,
        // but a corrupted/spoofed SEXP that satisfies both checks shouldn't OOB
        // the vector_elt reads below.
        if len < 2 {
            return None;
        }

        let msg_sexp = sexp.vector_elt(0);
        let msg: String = msg_sexp
            .string_elt_str(0)
            .unwrap_or("<invalid error message>")
            .to_string();

        let kind_sexp = sexp.vector_elt(1);
        let kind: &str = kind_sexp
            .string_elt_str(0)
            .unwrap_or(crate::error_value::kind::PANIC);

        // Class slot is element [2] in the 4-element form (NULL in legacy form)
        let class: Option<String> = if len >= 4 {
            let class_sexp = sexp.vector_elt(2);
            if class_sexp.is_nil() {
                None
            } else {
                class_sexp.string_elt_str(0).map(|s| s.to_string())
            }
        } else {
            None
        };

        use crate::error_value::kind as kind_const;

        // Slot [4] is the optional named-list condition data, present when `len >= 5`.
        //
        // We reverse-map the SEXP types back into `ConditionData` so that structured
        // fields survive the cross-package trait-ABI re-panic path
        // (`repanic_if_rust_error`). The consumer's outer `with_r_unwind_protect`
        // guard rebuilds the tagged SEXP from the reconstructed `RCondition`, which
        // now carries the data — so `e$field_name` is accessible in R handlers even
        // when the error crossed a package boundary.
        //
        // Type mapping (SEXPTYPE → ConditionDataValue):
        //   INTSXP  len=1  → Int(v)           (drop if NA_integer_)
        //   INTSXP  len>1  → IntVec(v)         (drop entire field if any element is NA_integer_)
        //   REALSXP len=1  → Real(v)           (drop if NA_real_)
        //   REALSXP len>1  → RealVec(v)        (drop entire field if any element is NA_real_)
        //   LGLSXP  len=1  → Bool(v)           (drop if NA_logical_)
        //   LGLSXP  len>1  → BoolVec(v)        (drop entire field if any element is NA_logical_)
        //   STRSXP  len=1  → Str(v)            (drop if NA_character_)
        //   STRSXP  len>1  → StrVec(v)         (drop entire field if any element is NA_character_)
        //   other SEXPTYPE  → drop the field   (lossy but safe — preserves message/class/kind)
        //
        // NA handling (v1): fields containing NA values are dropped rather than
        // emitted as bogus Rust values. Full NA fidelity via `Option`-bearing
        // `ConditionDataValue` variants is deferred to a follow-up PR.
        //
        // All reads here are non-allocating copies into owned Rust values, so
        // no new SEXPs are created and the existing `_guard` OwnedProtect suffices.
        let data: Option<ConditionData> = if len >= 5 {
            let data_sexp = sexp.vector_elt(4);
            if data_sexp.is_nil() || !data_sexp.is_list() {
                None
            } else {
                let data_len = data_sexp.len();
                let names_sexp = data_sexp.get_names();
                let mut fields: ConditionData = Vec::with_capacity(data_len);
                for i in 0..data_len {
                    // Read the field name from the names attribute. If missing/empty, skip.
                    let name: String = if names_sexp.is_nil() {
                        continue;
                    } else if names_sexp.is_character() {
                        match names_sexp.string_elt_str(i as isize) {
                            Some(s) if !s.is_empty() => s.to_string(),
                            _ => continue,
                        }
                    } else {
                        continue;
                    };
                    let elt = data_sexp.vector_elt(i as isize);
                    let elt_len = elt.len();
                    // Map SEXPTYPE → ConditionDataValue (v1 types only; drop unknown types).
                    // NA-bearing fields are dropped entirely (see NA handling note above).
                    let value: Option<ConditionDataValue> = if elt.is_integer() {
                        if elt_len == 1 {
                            let v = elt.integer_elt(0);
                            // i32::MIN == NA_integer_ in R — drop rather than emit bogus value.
                            if v == i32::MIN {
                                None
                            } else {
                                Some(ConditionDataValue::Int(v))
                            }
                        } else {
                            let mut vec: Vec<i32> = Vec::with_capacity(elt_len);
                            let mut has_na = false;
                            for j in 0..elt_len {
                                let v = elt.integer_elt(j as isize);
                                if v == i32::MIN {
                                    has_na = true;
                                    break;
                                }
                                vec.push(v);
                            }
                            if has_na {
                                None
                            } else {
                                Some(ConditionDataValue::IntVec(vec))
                            }
                        }
                    } else if elt.is_real() {
                        use crate::altrep_traits::NA_REAL;
                        if elt_len == 1 {
                            let v = elt.real_elt(0);
                            if v.to_bits() == NA_REAL.to_bits() {
                                None
                            } else {
                                Some(ConditionDataValue::Real(v))
                            }
                        } else {
                            let mut vec: Vec<f64> = Vec::with_capacity(elt_len);
                            let mut has_na = false;
                            for j in 0..elt_len {
                                let v = elt.real_elt(j as isize);
                                if v.to_bits() == NA_REAL.to_bits() {
                                    has_na = true;
                                    break;
                                }
                                vec.push(v);
                            }
                            if has_na {
                                None
                            } else {
                                Some(ConditionDataValue::RealVec(vec))
                            }
                        }
                    } else if elt.is_logical() {
                        // NA_logical_ == i32::MIN; 0 = FALSE, 1 = TRUE.
                        if elt_len == 1 {
                            let v = elt.logical_elt(0);
                            if v == i32::MIN {
                                None
                            } else {
                                Some(ConditionDataValue::Bool(v != 0))
                            }
                        } else {
                            let mut vec: Vec<bool> = Vec::with_capacity(elt_len);
                            let mut has_na = false;
                            for j in 0..elt_len {
                                let v = elt.logical_elt(j as isize);
                                if v == i32::MIN {
                                    has_na = true;
                                    break;
                                }
                                vec.push(v != 0);
                            }
                            if has_na {
                                None
                            } else {
                                Some(ConditionDataValue::BoolVec(vec))
                            }
                        }
                    } else if elt.is_character() {
                        if elt_len == 1 {
                            let charsxp = elt.string_elt(0);
                            if charsxp.is_na_string() {
                                None
                            } else {
                                elt.string_elt_str(0)
                                    .map(|s| ConditionDataValue::Str(s.to_string()))
                            }
                        } else {
                            let mut vec: Vec<String> = Vec::with_capacity(elt_len);
                            let mut has_na = false;
                            for j in 0..elt_len {
                                let charsxp = elt.string_elt(j as isize);
                                if charsxp.is_na_string() {
                                    has_na = true;
                                    break;
                                }
                                match elt.string_elt_str(j as isize) {
                                    Some(s) => vec.push(s.to_string()),
                                    None => {
                                        has_na = true;
                                        break;
                                    }
                                }
                            }
                            if has_na {
                                None
                            } else {
                                Some(ConditionDataValue::StrVec(vec))
                            }
                        }
                    } else {
                        // Unknown SEXPTYPE — drop the field (safe degradation).
                        None
                    };
                    if let Some(v) = value {
                        fields.push((name, v));
                    }
                }
                if fields.is_empty() {
                    None
                } else {
                    Some(fields)
                }
            }
        } else {
            None
        };

        let cond = match kind {
            kind_const::ERROR
            | kind_const::PANIC
            | kind_const::RESULT_ERR
            | kind_const::NONE_ERR
            | kind_const::OTHER_RUST_ERROR => RCondition::Error {
                message: msg,
                class,
                data,
            },
            kind_const::WARNING => RCondition::Warning {
                message: msg,
                class,
                data,
            },
            kind_const::MESSAGE => RCondition::Message { message: msg, data },
            kind_const::CONDITION => RCondition::Condition {
                message: msg,
                class,
                data,
            },
            other => {
                // Unknown kind — degrade to error
                RCondition::Error {
                    message: format!("[{other}] {msg}"),
                    class,
                    data,
                }
            }
        };
        Some(cond)
    }
}

/// Inspect a SEXP returned by a trait-ABI vtable shim and, if it is a tagged
/// error value, re-panic with the reconstructed [`RCondition`].
///
/// This is the "re-panic at the View boundary" step of Approach 1 from the
/// issue-345 plan. The caller (a generated View method wrapper) does:
///
/// ```ignore
/// let result = { vtable_call };
/// ::miniextendr_api::trait_abi::repanic_if_rust_error(result);
/// // ... convert result normally if we reach here
/// ```
///
/// When `sexp` is a tagged error value:
/// - `RCondition::Error` / `RCondition::Warning` / etc. → `panic_any!(cond)`.
///   The outer `with_r_unwind_protect` in the consumer's C entry point will
///   catch this and produce a tagged SEXP for the consumer's R wrapper.
///
/// When `sexp` is a normal value: this is a no-op.
///
/// # Safety
///
/// Must be called from R's main thread. `sexp` must be a valid (possibly
/// tagged) SEXP.
pub unsafe fn repanic_if_rust_error(sexp: crate::SEXP) {
    if let Some(cond) = unsafe { RCondition::from_tagged_sexp(sexp) } {
        std::panic::panic_any(cond);
    }
}

// endregion

// region: AsRError struct — wraps std::error::Error for Result returns

/// Structured error wrapper that preserves the `std::error::Error` cause chain.
///
/// When displayed, formats the error message with its full source chain:
/// ```text
/// top-level message
///   caused by: middle error
///   caused by: root cause
/// ```
///
/// Implements `From<E>` so it works with `?` and `.map_err(AsRError)`.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::condition::AsRError;
/// use std::num::ParseIntError;
///
/// #[miniextendr]
/// fn parse_number(s: &str) -> Result<i32, AsRError<ParseIntError>> {
///     s.parse::<i32>().map_err(AsRError)
/// }
/// ```
pub struct AsRError<E: std::error::Error>(pub E);

impl<E: std::error::Error> From<E> for AsRError<E> {
    #[inline]
    fn from(err: E) -> Self {
        AsRError(err)
    }
}

impl<E: std::error::Error> std::fmt::Display for AsRError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Write the top-level message
        write!(f, "{}", self.0)?;

        // Walk the cause chain
        let mut current: &dyn std::error::Error = &self.0;
        while let Some(source) = current.source() {
            write!(f, "\n  caused by: {source}")?;
            current = source;
        }

        Ok(())
    }
}

impl<E: std::error::Error> std::fmt::Debug for AsRError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AsRError<{}>({})", std::any::type_name::<E>(), self)
    }
}

impl<E: std::error::Error> AsRError<E> {
    /// Get the inner error.
    #[inline]
    pub fn into_inner(self) -> E {
        self.0
    }

    /// Get the Rust type name of the wrapped error (for programmatic matching).
    #[inline]
    pub fn rust_type_name(&self) -> &'static str {
        std::any::type_name::<E>()
    }

    /// Collect the full cause chain as a `Vec<String>`.
    pub fn cause_chain(&self) -> Vec<String> {
        let mut chain = vec![self.0.to_string()];
        let mut current: &dyn std::error::Error = &self.0;
        while let Some(source) = current.source() {
            chain.push(source.to_string());
            current = source;
        }
        chain
    }
}

// endregion

// region: Tests — macro grammar + payload contents (no R runtime needed)

#[cfg(test)]
mod condition_macro_tests {
    use super::{ConditionData, ConditionDataValue, RCondition};

    /// Catch the `panic_any(RCondition)` raised by a macro invocation and
    /// return the payload. No R runtime needed — the macros panic before any
    /// R API call.
    fn catch(f: impl FnOnce() + std::panic::UnwindSafe) -> RCondition {
        let payload = std::panic::catch_unwind(f).expect_err("macro must panic");
        *payload
            .downcast::<RCondition>()
            .expect("payload must be RCondition")
    }

    fn assert_data(data: &Option<ConditionData>, expected: &[(&str, ConditionDataValue)]) {
        let data = data.as_ref().expect("data must be Some");
        assert_eq!(data.len(), expected.len());
        for ((name, value), (exp_name, exp_value)) in data.iter().zip(expected) {
            assert_eq!(name, exp_name);
            // ConditionDataValue has no PartialEq (f64); compare via Debug.
            assert_eq!(format!("{value:?}"), format!("{exp_value:?}"));
        }
    }

    #[test]
    fn error_message_only_backcompat() {
        let cond = catch(|| crate::error!("plain {}", 42));
        match cond {
            RCondition::Error {
                message,
                class,
                data,
            } => {
                assert_eq!(message, "plain 42");
                assert!(class.is_none());
                assert!(data.is_none());
            }
            other => panic!("wrong variant: {other:?}"),
        }
    }

    #[test]
    fn error_class_only_backcompat() {
        let cond = catch(|| crate::error!(class = "my_error", "missing field: {}", "x"));
        match cond {
            RCondition::Error {
                message,
                class,
                data,
            } => {
                assert_eq!(message, "missing field: x");
                assert_eq!(class.as_deref(), Some("my_error"));
                assert!(data.is_none());
            }
            other => panic!("wrong variant: {other:?}"),
        }
    }

    #[test]
    fn error_single_data_pair() {
        let value = 41_i32;
        let cond = catch(move || crate::error!(data = ("value", value), "v = {value}"));
        match cond {
            RCondition::Error {
                message,
                class,
                data,
            } => {
                assert_eq!(message, "v = 41");
                assert!(class.is_none());
                assert_data(&data, &[("value", ConditionDataValue::Int(41))]);
            }
            other => panic!("wrong variant: {other:?}"),
        }
    }

    #[test]
    fn error_class_and_data_list_all_value_types() {
        let cond = catch(|| {
            crate::error!(
                class = "validation_error",
                data = [
                    ("value", 1.5),
                    ("code", 7),
                    ("label", "lhs"),
                    ("fatal", false),
                    ("ints", vec![1, 2]),
                    ("reals", vec![0.5_f64]),
                    ("flags", vec![true]),
                    ("labels", vec!["a".to_string()])
                ],
                "out of range"
            )
        });
        match cond {
            RCondition::Error {
                message,
                class,
                data,
            } => {
                assert_eq!(message, "out of range");
                assert_eq!(class.as_deref(), Some("validation_error"));
                assert_data(
                    &data,
                    &[
                        ("value", ConditionDataValue::Real(1.5)),
                        ("code", ConditionDataValue::Int(7)),
                        ("label", ConditionDataValue::Str("lhs".into())),
                        ("fatal", ConditionDataValue::Bool(false)),
                        ("ints", ConditionDataValue::IntVec(vec![1, 2])),
                        ("reals", ConditionDataValue::RealVec(vec![0.5])),
                        ("flags", ConditionDataValue::BoolVec(vec![true])),
                        ("labels", ConditionDataValue::StrVec(vec!["a".into()])),
                    ],
                );
            }
            other => panic!("wrong variant: {other:?}"),
        }
    }

    #[test]
    fn warning_with_class_and_data() {
        let cond = catch(|| crate::warning!(class = "trunc", data = ("dropped", 3), "dropped"));
        match cond {
            RCondition::Warning {
                message,
                class,
                data,
            } => {
                assert_eq!(message, "dropped");
                assert_eq!(class.as_deref(), Some("trunc"));
                assert_data(&data, &[("dropped", ConditionDataValue::Int(3))]);
            }
            other => panic!("wrong variant: {other:?}"),
        }
    }

    #[test]
    fn message_with_data() {
        let cond = catch(|| crate::message!(data = ("step", 2), "step {}", 2));
        match cond {
            RCondition::Message { message, data } => {
                assert_eq!(message, "step 2");
                assert_data(&data, &[("step", ConditionDataValue::Int(2))]);
            }
            other => panic!("wrong variant: {other:?}"),
        }
    }

    #[test]
    fn condition_with_class_and_data() {
        let cond =
            catch(|| crate::condition!(class = "progress", data = [("n", 10)], "processed {}", 10));
        match cond {
            RCondition::Condition {
                message,
                class,
                data,
            } => {
                assert_eq!(message, "processed 10");
                assert_eq!(class.as_deref(), Some("progress"));
                assert_data(&data, &[("n", ConditionDataValue::Int(10))]);
            }
            other => panic!("wrong variant: {other:?}"),
        }
    }

    #[test]
    fn data_list_trailing_comma() {
        let cond = catch(|| crate::error!(data = [("a", 1), ("b", 2),], "msg"));
        match cond {
            RCondition::Error { data, .. } => {
                assert_data(
                    &data,
                    &[
                        ("a", ConditionDataValue::Int(1)),
                        ("b", ConditionDataValue::Int(2)),
                    ],
                );
            }
            other => panic!("wrong variant: {other:?}"),
        }
    }
}

// endregion
