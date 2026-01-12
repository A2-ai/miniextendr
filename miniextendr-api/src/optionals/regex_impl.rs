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
// Vec conversions
// =============================================================================

impl TryFromSexp for Vec<Regex> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let patterns: Vec<String> = TryFromSexp::try_from_sexp(sexp)?;
        patterns
            .into_iter()
            .enumerate()
            .map(|(i, pattern)| {
                Regex::new(&pattern).map_err(|e| {
                    SexpError::InvalidValue(format!("invalid regex pattern at index {}: {}", i, e))
                })
            })
            .collect()
    }
}

impl TryFromSexp for Vec<Option<Regex>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let patterns: Vec<Option<String>> = TryFromSexp::try_from_sexp(sexp)?;
        patterns
            .into_iter()
            .enumerate()
            .map(|(i, opt)| match opt {
                None => Ok(None),
                Some(pattern) => Regex::new(&pattern).map(Some).map_err(|e| {
                    SexpError::InvalidValue(format!("invalid regex pattern at index {}: {}", i, e))
                }),
            })
            .collect()
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

// =============================================================================
// RRegexOps adapter trait
// =============================================================================

/// Adapter trait for [`Regex`] operations.
///
/// Provides string replacement and matching operations from R.
/// Automatically implemented for `regex::Regex`.
///
/// # Example
///
/// ```rust,ignore
/// use regex::Regex;
/// use miniextendr_api::regex_impl::RRegexOps;
///
/// #[derive(ExternalPtr)]
/// struct MyPattern(Regex);
///
/// #[miniextendr]
/// impl RRegexOps for MyPattern {}
///
/// miniextendr_module! {
///     mod mymodule;
///     impl RRegexOps for MyPattern;
/// }
/// ```
///
/// In R:
/// ```r
/// pat <- compile_regex("\\d+")
/// pat$replace_first("abc123def456", "X")  # "abcXdef456"
/// pat$replace_all("abc123def456", "X")    # "abcXdefX"
/// pat$is_match("test123")                 # TRUE
/// pat$find("test123")                     # "123"
/// ```
pub trait RRegexOps {
    /// Replace the first match in the text.
    fn replace_first(&self, text: &str, replacement: &str) -> String;

    /// Replace all matches in the text.
    fn replace_all(&self, text: &str, replacement: &str) -> String;

    /// Check if the pattern matches anywhere in the text.
    fn is_match(&self, text: &str) -> bool;

    /// Find the first match and return it, or None if no match.
    fn find(&self, text: &str) -> Option<String>;

    /// Find all non-overlapping matches.
    fn find_all(&self, text: &str) -> Vec<String>;

    /// Split the text by the pattern.
    fn split(&self, text: &str) -> Vec<String>;

    /// Get the number of capture groups (including the whole match).
    fn captures_len(&self) -> i32;
}

impl RRegexOps for Regex {
    fn replace_first(&self, text: &str, replacement: &str) -> String {
        self.replace(text, replacement).into_owned()
    }

    fn replace_all(&self, text: &str, replacement: &str) -> String {
        Regex::replace_all(self, text, replacement).into_owned()
    }

    fn is_match(&self, text: &str) -> bool {
        Regex::is_match(self, text)
    }

    fn find(&self, text: &str) -> Option<String> {
        Regex::find(self, text).map(|m| m.as_str().to_string())
    }

    fn find_all(&self, text: &str) -> Vec<String> {
        Regex::find_iter(self, text)
            .map(|m| m.as_str().to_string())
            .collect()
    }

    fn split(&self, text: &str) -> Vec<String> {
        Regex::split(self, text).map(|s| s.to_string()).collect()
    }

    fn captures_len(&self) -> i32 {
        self.captures_len() as i32
    }
}

// =============================================================================
// RCaptures adapter trait
// =============================================================================

/// Wrapper for regex capture groups.
///
/// This type wraps `regex::Captures` for access from R.
/// It holds owned copies of capture group strings for safe access.
#[derive(Debug, Clone)]
pub struct CaptureGroups {
    groups: Vec<Option<String>>,
    names: std::collections::HashMap<String, usize>,
}

impl CaptureGroups {
    /// Create capture groups from a regex and text.
    pub fn capture(re: &Regex, text: &str) -> Option<Self> {
        re.captures(text).map(|caps| {
            let groups: Vec<Option<String>> = caps
                .iter()
                .map(|m| m.map(|m| m.as_str().to_string()))
                .collect();

            let names: std::collections::HashMap<String, usize> = re
                .capture_names()
                .enumerate()
                .filter_map(|(i, name)| name.map(|n| (n.to_string(), i)))
                .collect();

            CaptureGroups { groups, names }
        })
    }
}

/// Adapter trait for capture group access.
///
/// Provides methods to access regex capture groups from R.
///
/// # Example
///
/// ```rust,ignore
/// use regex::Regex;
/// use miniextendr_api::regex_impl::{CaptureGroups, RCaptureGroups};
///
/// let re = Regex::new(r"(\d{4})-(\d{2})-(\d{2})").unwrap();
/// let caps = CaptureGroups::capture(&re, "Date: 2024-01-15").unwrap();
///
/// assert_eq!(caps.get(0), Some("2024-01-15".to_string()));  // whole match
/// assert_eq!(caps.get(1), Some("2024".to_string()));        // year
/// assert_eq!(caps.get(2), Some("01".to_string()));          // month
/// assert_eq!(caps.get(3), Some("15".to_string()));          // day
/// ```
pub trait RCaptureGroups {
    /// Get a capture group by index (0 = whole match).
    fn get(&self, i: i32) -> Option<String>;

    /// Get a capture group by name.
    fn get_named(&self, name: &str) -> Option<String>;

    /// Get the number of capture groups.
    fn len(&self) -> i32;

    /// Check if there are no capture groups.
    fn is_empty(&self) -> bool;

    /// Get all capture groups as a vector (None for non-matching groups).
    fn all_groups(&self) -> Vec<Option<String>>;
}

impl RCaptureGroups for CaptureGroups {
    fn get(&self, i: i32) -> Option<String> {
        if i < 0 {
            return None;
        }
        self.groups.get(i as usize).and_then(|s| s.clone())
    }

    fn get_named(&self, name: &str) -> Option<String> {
        self.names
            .get(name)
            .and_then(|&i| self.groups.get(i).and_then(|s| s.clone()))
    }

    fn len(&self) -> i32 {
        self.groups.len() as i32
    }

    fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }

    fn all_groups(&self) -> Vec<Option<String>> {
        self.groups.clone()
    }
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
        let pattern = String::from("[invalid");
        let result = Regex::new(&pattern);
        assert!(result.is_err());
    }

    #[test]
    fn try_compile_works() {
        assert!(try_compile(r"\d+").is_ok());
        let pattern = String::from("[invalid");
        assert!(try_compile(&pattern).is_err());
    }

    #[test]
    fn rregexops_replace() {
        let re = Regex::new(r"\d+").unwrap();
        assert_eq!(re.replace_first("abc123def456", "X"), "abcXdef456");
        assert_eq!(re.replace_all("abc123def456", "X"), "abcXdefX");
    }

    #[test]
    fn rregexops_find() {
        let re = Regex::new(r"\d+").unwrap();
        assert_eq!(RRegexOps::find(&re, "abc123def"), Some("123".to_string()));
        assert_eq!(RRegexOps::find(&re, "no digits"), None);
        assert_eq!(RRegexOps::find_all(&re, "a1b2c3"), vec!["1", "2", "3"]);
    }

    #[test]
    fn rregexops_split() {
        let re = Regex::new(r"\s+").unwrap();
        assert_eq!(
            RRegexOps::split(&re, "hello world  test"),
            vec!["hello", "world", "test"]
        );
    }

    #[test]
    fn rregexops_match() {
        let re = Regex::new(r"\d+").unwrap();
        assert!(RRegexOps::is_match(&re, "has123numbers"));
        assert!(!RRegexOps::is_match(&re, "no numbers"));
    }

    #[test]
    fn rcapturegroups_basic() {
        let re = Regex::new(r"(\d{4})-(\d{2})-(\d{2})").unwrap();
        let caps = CaptureGroups::capture(&re, "Date: 2024-01-15 here").unwrap();

        assert_eq!(caps.get(0), Some("2024-01-15".to_string())); // whole match
        assert_eq!(caps.get(1), Some("2024".to_string())); // year
        assert_eq!(caps.get(2), Some("01".to_string())); // month
        assert_eq!(caps.get(3), Some("15".to_string())); // day
        assert_eq!(caps.get(4), None); // out of bounds
        assert_eq!(caps.get(-1), None); // negative index
        assert_eq!(caps.len(), 4);
        assert!(!caps.is_empty());
    }

    #[test]
    fn rcapturegroups_named() {
        let re = Regex::new(r"(?P<year>\d{4})-(?P<month>\d{2})-(?P<day>\d{2})").unwrap();
        let caps = CaptureGroups::capture(&re, "2024-01-15").unwrap();

        assert_eq!(caps.get_named("year"), Some("2024".to_string()));
        assert_eq!(caps.get_named("month"), Some("01".to_string()));
        assert_eq!(caps.get_named("day"), Some("15".to_string()));
        assert_eq!(caps.get_named("nonexistent"), None);
    }

    #[test]
    fn rcapturegroups_no_match() {
        let re = Regex::new(r"(\d+)").unwrap();
        let caps = CaptureGroups::capture(&re, "no digits");
        assert!(caps.is_none());
    }

    #[test]
    fn rcapturegroups_all_groups() {
        let re = Regex::new(r"(\w+)@(\w+)\.(\w+)").unwrap();
        let caps = CaptureGroups::capture(&re, "test@example.com").unwrap();
        let all = caps.all_groups();
        assert_eq!(all.len(), 4);
        assert_eq!(all[0], Some("test@example.com".to_string()));
        assert_eq!(all[1], Some("test".to_string()));
        assert_eq!(all[2], Some("example".to_string()));
        assert_eq!(all[3], Some("com".to_string()));
    }
}
