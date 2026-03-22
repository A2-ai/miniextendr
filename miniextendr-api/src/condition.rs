//! Structured error adapter for `std::error::Error`.
//!
//! [`RCondition`] wraps any `E: std::error::Error` and preserves the full
//! error chain (cause/source) when converting to an R error message.
//!
//! # Usage
//!
//! Use as the `Err` type in `Result` returns from `#[miniextendr]` functions:
//!
//! ```ignore
//! use miniextendr_api::condition::RCondition;
//!
//! #[miniextendr]
//! fn parse_config(path: &str) -> Result<i32, RCondition<std::io::Error>> {
//!     let content = std::fs::read_to_string(path).map_err(RCondition)?;
//!     Ok(content.len() as i32)
//! }
//! ```
//!
//! The R error message includes the full cause chain:
//! ```r
//! tryCatch(parse_config("/nonexistent"), error = function(e) e$message)
//! # "No such file or directory (os error 2)\n  caused by: ..."
//! ```

/// Structured error wrapper that preserves the `std::error::Error` cause chain.
///
/// When displayed, formats the error message with its full source chain:
/// ```text
/// top-level message
///   caused by: middle error
///   caused by: root cause
/// ```
///
/// Implements `From<E>` so it works with `?` and `.map_err(RCondition)`.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::condition::RCondition;
/// use std::num::ParseIntError;
///
/// #[miniextendr]
/// fn parse_number(s: &str) -> Result<i32, RCondition<ParseIntError>> {
///     s.parse::<i32>().map_err(RCondition)
/// }
/// ```
pub struct RCondition<E: std::error::Error>(pub E);

impl<E: std::error::Error> From<E> for RCondition<E> {
    #[inline]
    fn from(err: E) -> Self {
        RCondition(err)
    }
}

impl<E: std::error::Error> std::fmt::Display for RCondition<E> {
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

impl<E: std::error::Error> std::fmt::Debug for RCondition<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Debug shows the type name + Display output
        write!(
            f,
            "RCondition<{}>({})",
            std::any::type_name::<E>(),
            self
        )
    }
}

impl<E: std::error::Error> RCondition<E> {
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
