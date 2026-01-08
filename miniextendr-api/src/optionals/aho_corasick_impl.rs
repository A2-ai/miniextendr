//! Integration with the `aho-corasick` crate for multi-pattern string search.
//!
//! Provides fast multi-pattern search using the Aho-Corasick algorithm.
//!
//! # Features
//!
//! Enable this module with the `aho-corasick` feature:
//!
//! ```toml
//! [dependencies]
//! miniextendr-api = { version = "0.1", features = ["aho-corasick"] }
//! ```
//!
//! # Overview
//!
//! The Aho-Corasick algorithm enables searching for multiple patterns simultaneously
//! in a single pass over the haystack. This is much faster than searching for each
//! pattern individually.
//!
//! # Example
//!
//! ```ignore
//! use miniextendr_api::aho_corasick_impl::{aho_compile, aho_find_all};
//!
//! #[miniextendr]
//! fn find_keywords(patterns: Vec<String>, text: &str) -> Vec<(i32, i32, i32)> {
//!     let ac = aho_compile(&patterns).unwrap();
//!     aho_find_all(&ac, text)
//!         .into_iter()
//!         .map(|(pid, start, end)| ((pid + 1) as i32, start as i32, end as i32))
//!         .collect()
//! }
//! ```
//!
//! # Match Representation
//!
//! - Pattern IDs are **0-based** in Rust; convert to **1-based** for R wrappers
//! - `start`/`end` are **byte offsets** (not character indices)
//! - For UTF-8 text, character indices require additional conversion

pub use aho_corasick::{AhoCorasick, MatchKind};

use crate::ffi::{Rf_xlength, SEXP, SEXPTYPE, STRING_ELT, SexpExt};
use crate::from_r::charsxp_to_str;
use crate::from_r::{SexpError, TryFromSexp};
use crate::{
    impl_option_try_from_sexp, impl_vec_option_try_from_sexp_list, impl_vec_try_from_sexp_list,
};

// =============================================================================
// TryFromSexp for AhoCorasick (from Vec<String> patterns)
// =============================================================================

/// Build an `AhoCorasick` automaton from a character vector of patterns.
///
/// # Errors
///
/// Returns an error if:
/// - Input is not a character vector (STRSXP)
/// - Pattern vector is empty
/// - Building the automaton fails (e.g., invalid patterns)
impl TryFromSexp for AhoCorasick {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::STRSXP {
            return Err(SexpError::Type(crate::from_r::SexpTypeError {
                expected: SEXPTYPE::STRSXP,
                actual,
            }));
        }

        let len = unsafe { Rf_xlength(sexp) } as usize;
        if len == 0 {
            return Err(SexpError::InvalidValue(
                "aho-corasick requires at least one pattern".to_string(),
            ));
        }

        let mut patterns = Vec::with_capacity(len);
        for i in 0..len {
            let charsxp = unsafe { STRING_ELT(sexp, i as crate::ffi::R_xlen_t) };
            if charsxp == unsafe { crate::ffi::R_NaString } {
                return Err(SexpError::InvalidValue(format!(
                    "NA at index {} not allowed in patterns",
                    i
                )));
            }
            let s = unsafe { charsxp_to_str(charsxp) };
            patterns.push(s.to_string());
        }

        AhoCorasick::new(&patterns)
            .map_err(|e| SexpError::InvalidValue(format!("failed to build AhoCorasick: {}", e)))
    }
}

// Use macros to implement Option/Vec conversions
impl_option_try_from_sexp!(AhoCorasick);
impl_vec_try_from_sexp_list!(AhoCorasick);
impl_vec_option_try_from_sexp_list!(AhoCorasick);

// =============================================================================
// Helper functions
// =============================================================================

/// Compile an Aho-Corasick automaton from patterns.
///
/// # Errors
///
/// Returns an error if building the automaton fails.
///
/// # Example
///
/// ```ignore
/// let ac = aho_compile(&["foo".to_string(), "bar".to_string()])?;
/// ```
#[inline]
pub fn aho_compile(patterns: &[String]) -> Result<AhoCorasick, SexpError> {
    if patterns.is_empty() {
        return Err(SexpError::InvalidValue(
            "aho-corasick requires at least one pattern".to_string(),
        ));
    }
    AhoCorasick::new(patterns)
        .map_err(|e| SexpError::InvalidValue(format!("failed to build AhoCorasick: {}", e)))
}

/// Compile an Aho-Corasick automaton with custom options.
///
/// # Arguments
///
/// * `patterns` - Patterns to search for
/// * `ascii_case_insensitive` - Enable case-insensitive matching (ASCII only)
/// * `match_kind` - Match semantics: "standard", "leftmost-first", or "leftmost-longest"
///
/// # Errors
///
/// Returns an error if:
/// - Pattern vector is empty
/// - Unknown match_kind value
/// - Building the automaton fails
///
/// # Example
///
/// ```ignore
/// let ac = aho_builder(&["Foo".to_string()], true, "leftmost-longest")?;
/// ```
pub fn aho_builder(
    patterns: &[String],
    ascii_case_insensitive: bool,
    match_kind: &str,
) -> Result<AhoCorasick, SexpError> {
    if patterns.is_empty() {
        return Err(SexpError::InvalidValue(
            "aho-corasick requires at least one pattern".to_string(),
        ));
    }

    let kind = match match_kind {
        "standard" => MatchKind::Standard,
        "leftmost-first" => MatchKind::LeftmostFirst,
        "leftmost-longest" => MatchKind::LeftmostLongest,
        _ => {
            return Err(SexpError::InvalidValue(format!(
                "unknown match_kind '{}': expected 'standard', 'leftmost-first', or 'leftmost-longest'",
                match_kind
            )));
        }
    };

    aho_corasick::AhoCorasick::builder()
        .ascii_case_insensitive(ascii_case_insensitive)
        .match_kind(kind)
        .build(patterns)
        .map_err(|e| SexpError::InvalidValue(format!("failed to build AhoCorasick: {}", e)))
}

/// Find all matches in a haystack.
///
/// Returns a vector of `(pattern_id, start, end)` tuples where:
/// - `pattern_id` is 0-based (convert to 1-based for R)
/// - `start` and `end` are byte offsets into the haystack
///
/// # Example
///
/// ```ignore
/// let ac = aho_compile(&["foo".to_string(), "bar".to_string()])?;
/// let matches = aho_find_all(&ac, "foo and bar");
/// // matches: [(0, 0, 3), (1, 8, 11)]
/// ```
#[inline]
pub fn aho_find_all(ac: &AhoCorasick, haystack: &str) -> Vec<(usize, usize, usize)> {
    ac.find_iter(haystack)
        .map(|m| (m.pattern().as_usize(), m.start(), m.end()))
        .collect()
}

/// Find all matches and return as a flattened vector suitable for R.
///
/// Returns a flat vector: `[pid1, start1, end1, pid2, start2, end2, ...]`
/// where pattern IDs are **1-based** for R compatibility.
///
/// This format can be easily reshaped to a matrix in R:
/// ```r
/// result <- matrix(matches, ncol = 3, byrow = TRUE)
/// colnames(result) <- c("pattern_id", "start", "end")
/// ```
#[inline]
pub fn aho_find_all_flat(ac: &AhoCorasick, haystack: &str) -> Vec<i32> {
    let mut result = Vec::new();
    for m in ac.find_iter(haystack) {
        // +1 for 1-based pattern ID, +1 for 1-based start position
        result.push((m.pattern().as_usize() + 1) as i32);
        result.push((m.start() + 1) as i32); // 1-based position for R
        result.push((m.end()) as i32); // end is exclusive, so keep as-is for length
    }
    result
}

/// Check if any pattern matches in the haystack.
#[inline]
pub fn aho_is_match(ac: &AhoCorasick, haystack: &str) -> bool {
    ac.is_match(haystack)
}

/// Count total number of matches.
#[inline]
pub fn aho_count_matches(ac: &AhoCorasick, haystack: &str) -> usize {
    ac.find_iter(haystack).count()
}

/// Find the first (leftmost) match.
///
/// Returns `Some((pattern_id, start, end))` or `None` if no match.
/// Pattern ID is 0-based.
#[inline]
pub fn aho_find_first(ac: &AhoCorasick, haystack: &str) -> Option<(usize, usize, usize)> {
    ac.find(haystack)
        .map(|m| (m.pattern().as_usize(), m.start(), m.end()))
}

/// Replace all matches with a single replacement string.
#[inline]
pub fn aho_replace_all(ac: &AhoCorasick, haystack: &str, replacement: &str) -> String {
    ac.replace_all(haystack, &vec![replacement; ac.patterns_len()])
}

/// Replace matches with pattern-specific replacements.
///
/// `replacements` must have the same length as the number of patterns.
#[inline]
pub fn aho_replace_all_with(
    ac: &AhoCorasick,
    haystack: &str,
    replacements: &[String],
) -> Result<String, SexpError> {
    if replacements.len() != ac.patterns_len() {
        return Err(SexpError::InvalidValue(format!(
            "replacements length ({}) != patterns length ({})",
            replacements.len(),
            ac.patterns_len()
        )));
    }
    let refs: Vec<&str> = replacements.iter().map(|s| s.as_str()).collect();
    Ok(ac.replace_all(haystack, &refs))
}

// =============================================================================
// Adapter trait for AhoCorasick
// =============================================================================

/// Adapter trait for exposing `AhoCorasick` operations to R.
///
/// Since `AhoCorasick` doesn't implement `Clone` or `Copy`, it's typically
/// wrapped in an `ExternalPtr` for reuse across R calls.
///
/// # Registration
///
/// ```ignore
/// use miniextendr_api::aho_corasick_impl::{AhoCorasick, RAhoCorasickOps};
///
/// miniextendr_module! {
///     mod mymodule;
///     impl RAhoCorasickOps for AhoCorasick;
/// }
/// ```
pub trait RAhoCorasickOps {
    /// Number of patterns in the automaton.
    fn patterns_len(&self) -> i32;

    /// Check if haystack matches any pattern.
    fn is_match(&self, haystack: &str) -> bool;

    /// Count total matches in haystack.
    fn count_matches(&self, haystack: &str) -> i32;

    /// Find all matches, returning flat vec: [pid, start, end, ...]
    /// Pattern IDs are 1-based, positions are 1-based (R convention).
    fn find_all_flat(&self, haystack: &str) -> Vec<i32>;

    /// Find first match, returning [pid, start, end] or empty vec.
    /// Pattern ID is 1-based, positions are 1-based (R convention).
    fn find_first(&self, haystack: &str) -> Vec<i32>;

    /// Replace all matches with a single replacement.
    fn replace_all(&self, haystack: &str, replacement: &str) -> String;
}

impl RAhoCorasickOps for AhoCorasick {
    fn patterns_len(&self) -> i32 {
        self.patterns_len() as i32
    }

    fn is_match(&self, haystack: &str) -> bool {
        aho_is_match(self, haystack)
    }

    fn count_matches(&self, haystack: &str) -> i32 {
        aho_count_matches(self, haystack) as i32
    }

    fn find_all_flat(&self, haystack: &str) -> Vec<i32> {
        aho_find_all_flat(self, haystack)
    }

    fn find_first(&self, haystack: &str) -> Vec<i32> {
        match aho_find_first(self, haystack) {
            Some((pid, start, end)) => {
                vec![
                    (pid + 1) as i32,   // 1-based pattern ID
                    (start + 1) as i32, // 1-based position
                    end as i32,         // end is exclusive
                ]
            }
            None => vec![],
        }
    }

    fn replace_all(&self, haystack: &str, replacement: &str) -> String {
        aho_replace_all(self, haystack, replacement)
    }
}

// =============================================================================
// Unit tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aho_compile() {
        let patterns = vec!["foo".to_string(), "bar".to_string()];
        let ac = aho_compile(&patterns).unwrap();
        assert_eq!(ac.patterns_len(), 2);
    }

    #[test]
    fn test_aho_compile_empty() {
        let patterns: Vec<String> = vec![];
        let result = aho_compile(&patterns);
        assert!(result.is_err());
    }

    #[test]
    fn test_aho_find_all() {
        let ac = aho_compile(&["foo".to_string(), "bar".to_string()]).unwrap();
        let matches = aho_find_all(&ac, "foo and bar and foobar");
        // Expected: foo at 0-3, bar at 8-11, foo at 16-19, bar at 19-22
        assert_eq!(matches.len(), 4);
        assert_eq!(matches[0], (0, 0, 3));
        assert_eq!(matches[1], (1, 8, 11));
    }

    #[test]
    fn test_aho_find_all_flat() {
        let ac = aho_compile(&["foo".to_string(), "bar".to_string()]).unwrap();
        let flat = aho_find_all_flat(&ac, "foo bar");
        // foo: pid=1, start=1, end=3; bar: pid=2, start=5, end=7
        assert_eq!(flat, vec![1, 1, 3, 2, 5, 7]);
    }

    #[test]
    fn test_aho_is_match() {
        let ac = aho_compile(&["needle".to_string()]).unwrap();
        assert!(aho_is_match(&ac, "haystack needle haystack"));
        assert!(!aho_is_match(&ac, "no match here"));
    }

    #[test]
    fn test_aho_count_matches() {
        let ac = aho_compile(&["a".to_string()]).unwrap();
        assert_eq!(aho_count_matches(&ac, "banana"), 3);
    }

    #[test]
    fn test_aho_find_first() {
        let ac = aho_compile(&["foo".to_string(), "bar".to_string()]).unwrap();
        let first = aho_find_first(&ac, "bar then foo");
        assert_eq!(first, Some((1, 0, 3))); // bar matches first
    }

    #[test]
    fn test_aho_replace_all() {
        let ac = aho_compile(&["cat".to_string(), "dog".to_string()]).unwrap();
        let result = aho_replace_all(&ac, "cat and dog", "pet");
        assert_eq!(result, "pet and pet");
    }

    #[test]
    fn test_aho_builder_case_insensitive() {
        let ac = aho_builder(&["FOO".to_string()], true, "standard").unwrap();
        assert!(aho_is_match(&ac, "foo"));
        assert!(aho_is_match(&ac, "FOO"));
        assert!(aho_is_match(&ac, "Foo"));
    }

    #[test]
    fn test_aho_builder_invalid_kind() {
        let result = aho_builder(&["foo".to_string()], false, "invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_adapter_trait() {
        let ac = aho_compile(&["x".to_string(), "y".to_string()]).unwrap();
        assert_eq!(RAhoCorasickOps::patterns_len(&ac), 2);
        assert!(RAhoCorasickOps::is_match(&ac, "xyz"));
        // "xyx" matches: x at 0, y at 1, x at 2 = 3 total matches
        assert_eq!(RAhoCorasickOps::count_matches(&ac, "xyx"), 3);
    }
}
