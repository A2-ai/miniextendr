//! Integration with the `regex` crate.
//!
//! Provides compiled regular expressions from R character vectors.
//!
//! | R Type | Rust Type | Notes |
//! |--------|-----------|-------|
//! | `character(1)` | `Regex` | Compiles pattern |
//! | `NA_character_` | `Option<Regex>` | NA maps to None |
//!
//! **Note:** `Regex` does not implement `IntoR` since a compiled regex cannot be
//! meaningfully converted back to R. Use the original pattern string if needed.
//!
//! # Features
//!
//! Enable this module with the `regex` feature:
//!
//! ```toml
//! [dependencies]
//! miniextendr-api = { version = "0.1", features = ["regex"] }
//! ```
//!
//! # Example: One-shot matching
//!
//! ```ignore
//! use regex::Regex;
//!
//! #[miniextendr]
//! fn is_valid_email(pattern: Regex, emails: Vec<String>) -> Vec<bool> {
//!     emails.iter().map(|e| pattern.is_match(e)).collect()
//! }
//! ```
//!
//! # Example: Cached regex for repeated calls
//!
//! For functions called repeatedly in loops, compile the regex once and
//! return an `ExternalPtr<Regex>` for reuse:
//!
//! ```ignore
//! use regex::Regex;
//! use miniextendr_api::ExternalPtr;
//!
//! #[miniextendr]
//! fn compile_pattern(pattern: String) -> ExternalPtr<Regex> {
//!     let re = Regex::new(&pattern).expect("invalid regex");
//!     ExternalPtr::new(re)
//! }
//!
//! #[miniextendr]
//! fn match_with_cached(re: ExternalPtr<Regex>, text: String) -> bool {
//!     re.is_match(&text)
//! }
//! ```
//!
//! # Performance Notes
//!
//! - Regex compilation is expensive. For single calls, the overhead is acceptable.
//! - For loops or `lapply`, use `ExternalPtr<Regex>` to compile once.
//! - The `regex` crate's `Regex::new` already uses internal caching, but
//!   `ExternalPtr` avoids re-parsing the pattern string on each call.

pub use regex::Regex;

use crate::ffi::SEXP;
use crate::from_r::{SexpError, TryFromSexp};

// =============================================================================
// Scalar conversions
// =============================================================================

impl TryFromSexp for Regex {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let pattern: String = TryFromSexp::try_from_sexp(sexp)?;
        Regex::new(&pattern)
            .map_err(|e| SexpError::InvalidValue(format!("invalid regex pattern: {}", e)))
    }
}

// Note: IntoR is intentionally not implemented for Regex.
// A compiled regex cannot be meaningfully converted back to R.
// If you need the pattern, keep the original String.

// =============================================================================
// Option conversions (NA support)
// =============================================================================

impl TryFromSexp for Option<Regex> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let opt: Option<String> = TryFromSexp::try_from_sexp(sexp)?;
        match opt {
            None => Ok(None),
            Some(pattern) => Regex::new(&pattern)
                .map(Some)
                .map_err(|e| SexpError::InvalidValue(format!("invalid regex pattern: {}", e))),
        }
    }
}

// =============================================================================
// Helper functions
// =============================================================================

/// Compile a regex pattern and return an error-typed Result.
///
/// This is a convenience function for use in `#[miniextendr]` functions
/// that want to handle regex compilation errors gracefully.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::regex_impl::try_compile;
///
/// #[miniextendr]
/// fn safe_compile(pattern: String) -> Result<ExternalPtr<Regex>, String> {
///     try_compile(&pattern)
///         .map(ExternalPtr::new)
///         .map_err(|e| e.to_string())
/// }
/// ```
#[inline]
pub fn try_compile(pattern: &str) -> Result<Regex, regex::Error> {
    Regex::new(pattern)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regex_can_be_compiled() {
        let re = Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();
        assert!(re.is_match("2024-01-15"));
        assert!(!re.is_match("not-a-date"));
    }

    #[test]
    fn regex_invalid_pattern() {
        let result = Regex::new(r"[invalid");
        assert!(result.is_err());
    }

    #[test]
    fn try_compile_works() {
        assert!(try_compile(r"\d+").is_ok());
        assert!(try_compile(r"[invalid").is_err());
    }
}
