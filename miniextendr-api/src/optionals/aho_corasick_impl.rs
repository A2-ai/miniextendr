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

use crate::from_r::{
    SexpError, TryFromSexp, impl_option_try_from_sexp, impl_vec_option_try_from_sexp_list,
    impl_vec_try_from_sexp_list, try_charsxp_to_str,
};
use crate::{SEXP, SEXPTYPE, SexpExt};

// region: TryFromSexp for AhoCorasick (from Vec<String> patterns)

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
    const CHARACTER_ONLY: bool = true;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::STRSXP {
            return Err(SexpError::Type(crate::from_r::SexpTypeError {
                expected: SEXPTYPE::STRSXP,
                actual,
            }));
        }

        let len = sexp.len();
        if len == 0 {
            return Err(SexpError::InvalidValue(
                "aho-corasick requires at least one pattern".to_string(),
            ));
        }

        let mut patterns = Vec::with_capacity(len);
        let mut errors = crate::from_r::BatchedErrors::default();
        for i in 0..len {
            let charsxp = sexp.string_elt(i as crate::R_xlen_t);
            if charsxp == SEXP::na_string() {
                errors.push(i, || "NA is not allowed as a pattern".to_string());
                continue;
            }
            match unsafe { try_charsxp_to_str(charsxp) } {
                Ok(s) => patterns.push(s.to_string()),
                Err(e) => errors.push(i, || e.r_reason(false)),
            }
        }
        if !errors.is_empty() {
            return Err(errors.into_element_error());
        }

        AhoCorasick::new(&patterns)
            .map_err(|e| SexpError::InvalidValue(format!("failed to build AhoCorasick: {}", e)))
    }
}

// Note: IntoR is intentionally not implemented for AhoCorasick.
// A compiled automaton cannot be meaningfully converted back to R.
// If you need the patterns, keep the original character vector.

// Use macros to implement Option/Vec conversions
impl_option_try_from_sexp!(AhoCorasick);
impl_vec_try_from_sexp_list!(AhoCorasick);
impl_vec_option_try_from_sexp_list!(AhoCorasick);
// endregion

// region: Helper functions

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
        result.push(i32::try_from(m.pattern().as_usize() + 1).unwrap_or(i32::MAX));
        result.push(i32::try_from(m.start() + 1).unwrap_or(i32::MAX)); // 1-based position for R
        result.push(i32::try_from(m.end()).unwrap_or(i32::MAX)); // end is exclusive, so keep as-is for length
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
// endregion

// region: Adapter trait for AhoCorasick

/// Adapter trait for exposing `AhoCorasick` operations to R.
///
/// Since `AhoCorasick` doesn't implement `Clone` or `Copy`, it's typically
/// wrapped in an `ExternalPtr` for reuse across R calls.
///
/// # Registration
///
/// Registration is automatic when you annotate `impl RAhoCorasickOps for AhoCorasick`
/// with `#[miniextendr]`.
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
        i32::try_from(self.patterns_len()).unwrap_or(i32::MAX)
    }

    fn is_match(&self, haystack: &str) -> bool {
        aho_is_match(self, haystack)
    }

    fn count_matches(&self, haystack: &str) -> i32 {
        i32::try_from(aho_count_matches(self, haystack)).unwrap_or(i32::MAX)
    }

    fn find_all_flat(&self, haystack: &str) -> Vec<i32> {
        aho_find_all_flat(self, haystack)
    }

    fn find_first(&self, haystack: &str) -> Vec<i32> {
        match aho_find_first(self, haystack) {
            Some((pid, start, end)) => {
                vec![
                    i32::try_from(pid + 1).unwrap_or(i32::MAX), // 1-based pattern ID
                    i32::try_from(start + 1).unwrap_or(i32::MAX), // 1-based position
                    i32::try_from(end).unwrap_or(i32::MAX),     // end is exclusive
                ]
            }
            None => vec![],
        }
    }

    fn replace_all(&self, haystack: &str, replacement: &str) -> String {
        aho_replace_all(self, haystack, replacement)
    }
}
// endregion

// region: Unit tests

#[cfg(test)]
mod tests;
// endregion
