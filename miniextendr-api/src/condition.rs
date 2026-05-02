//! Condition macros and signal enum for the Rust→R condition pipeline.
//!
//! This module provides two things:
//!
//! 1. **[`RCondition`] enum** — the internal panic payload used by `error!()`,
//!    `warning!()`, `message!()`, and `condition!()` macros. Caught by
//!    `with_r_unwind_protect_error_in_r` before the generic panic→error path,
//!    then forwarded to R as a structured condition with `rust_*` class layering.
//!
//! 2. **[`RErrorAdapter`] struct** — wraps any `E: std::error::Error` and
//!    preserves the full error chain (cause/source) when converting to an R
//!    error message. Use as the `Err` type in `Result` returns.
//!
//! # Condition macros
//!
//! The four macros are the user-facing API for raising non-panic conditions from
//! Rust. They all require `error_in_r` mode (the default for `#[miniextendr]`
//! functions):
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
//! # `RErrorAdapter`
//!
//! ```ignore
//! use miniextendr_api::condition::RErrorAdapter;
//!
//! #[miniextendr]
//! fn parse_config(path: &str) -> Result<i32, RErrorAdapter<std::io::Error>> {
//!     let content = std::fs::read_to_string(path).map_err(RErrorAdapter)?;
//!     Ok(content.len() as i32)
//! }
//! ```

// region: RCondition enum — internal panic payload

/// Internal panic payload for structured R conditions.
///
/// Raised by the `error!()`, `warning!()`, `message!()`, and `condition!()`
/// macros via `std::panic::panic_any`. Caught by
/// `with_r_unwind_protect_error_in_r` before the generic panic→string path
/// and forwarded to R as a tagged SEXP with `rust_*` class layering.
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
    },
    /// Raised by `warning!(...)` / `warning!(class = "...", ...)`.
    Warning {
        message: String,
        class: Option<String>,
    },
    /// Raised by `message!(...)`.
    Message { message: String },
    /// Raised by `condition!(...)` / `condition!(class = "...", ...)`.
    Condition {
        message: String,
        class: Option<String>,
    },
}

// endregion

// region: Macros

/// Raise an R error from Rust with `rust_error` class layering.
///
/// Requires `error_in_r` mode (the default for `#[miniextendr]` functions).
/// The raised condition has class `c("rust_error", "simpleError", "error", "condition")`.
///
/// An optional `class = "name"` form prepends a custom class for programmatic catching:
/// `c("name", "rust_error", "simpleError", "error", "condition")`.
///
/// # Examples
///
/// ```ignore
/// use miniextendr_api::error;
///
/// #[miniextendr]
/// fn fail() {
///     error!("something went wrong: {}", 42);
/// }
///
/// // With a custom class for tryCatch:
/// #[miniextendr]
/// fn typed_fail(name: &str) {
///     error!(class = "my_error", "missing field: {name}");
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
    (class = $class:expr, $($arg:tt)*) => {
        ::std::panic::panic_any($crate::condition::RCondition::Error {
            message: ::std::format!($($arg)*),
            class: ::std::option::Option::Some($class.to_string()),
        })
    };
    ($($arg:tt)*) => {
        ::std::panic::panic_any($crate::condition::RCondition::Error {
            message: ::std::format!($($arg)*),
            class: ::std::option::Option::None,
        })
    };
}

/// Raise an R warning from Rust with `rust_warning` class layering.
///
/// Requires `error_in_r` mode (the default for `#[miniextendr]` functions).
/// Unlike `panic!`, execution continues after `warning!` is caught by a handler.
/// The raised condition has class `c("rust_warning", "simpleWarning", "warning", "condition")`.
///
/// An optional `class = "name"` form prepends a custom class.
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
    (class = $class:expr, $($arg:tt)*) => {
        ::std::panic::panic_any($crate::condition::RCondition::Warning {
            message: ::std::format!($($arg)*),
            class: ::std::option::Option::Some($class.to_string()),
        })
    };
    ($($arg:tt)*) => {
        ::std::panic::panic_any($crate::condition::RCondition::Warning {
            message: ::std::format!($($arg)*),
            class: ::std::option::Option::None,
        })
    };
}

/// Emit an R message from Rust with `rust_message` class layering.
///
/// Requires `error_in_r` mode (the default for `#[miniextendr]` functions).
/// The raised condition has class `c("rust_message", "simpleMessage", "message", "condition")`.
/// Muffled by `suppressMessages()` automatically (standard R restart mechanism).
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
    ($($arg:tt)*) => {
        ::std::panic::panic_any($crate::condition::RCondition::Message {
            message: ::std::format!($($arg)*),
        })
    };
}

/// Signal a generic R condition from Rust with `rust_condition` class layering.
///
/// Requires `error_in_r` mode (the default for `#[miniextendr]` functions).
/// Unlike `error!`, a bare condition is a silent no-op if there is no handler.
/// The raised condition has class `c("rust_condition", "simpleCondition", "condition")`.
///
/// An optional `class = "name"` form prepends a custom class.
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
    (class = $class:expr, $($arg:tt)*) => {
        ::std::panic::panic_any($crate::condition::RCondition::Condition {
            message: ::std::format!($($arg)*),
            class: ::std::option::Option::Some($class.to_string()),
        })
    };
    ($($arg:tt)*) => {
        ::std::panic::panic_any($crate::condition::RCondition::Condition {
            message: ::std::format!($($arg)*),
            class: ::std::option::Option::None,
        })
    };
}

// endregion

// region: RErrorAdapter struct — wraps std::error::Error for Result returns

/// Structured error wrapper that preserves the `std::error::Error` cause chain.
///
/// When displayed, formats the error message with its full source chain:
/// ```text
/// top-level message
///   caused by: middle error
///   caused by: root cause
/// ```
///
/// Implements `From<E>` so it works with `?` and `.map_err(RErrorAdapter)`.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::condition::RErrorAdapter;
/// use std::num::ParseIntError;
///
/// #[miniextendr]
/// fn parse_number(s: &str) -> Result<i32, RErrorAdapter<ParseIntError>> {
///     s.parse::<i32>().map_err(RErrorAdapter)
/// }
/// ```
pub struct RErrorAdapter<E: std::error::Error>(pub E);

impl<E: std::error::Error> From<E> for RErrorAdapter<E> {
    #[inline]
    fn from(err: E) -> Self {
        RErrorAdapter(err)
    }
}

impl<E: std::error::Error> std::fmt::Display for RErrorAdapter<E> {
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

impl<E: std::error::Error> std::fmt::Debug for RErrorAdapter<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RErrorAdapter<{}>({})", std::any::type_name::<E>(), self)
    }
}

impl<E: std::error::Error> RErrorAdapter<E> {
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
