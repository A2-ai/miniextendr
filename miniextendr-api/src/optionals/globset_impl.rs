//! Integration with the `globset` crate for shell-style glob matching.
//!
//! Provides multi-pattern glob matching (`*.R`, `**/*.rs`, `{foo,bar}/*`) —
//! the natural pattern language for R users coming from `list.files()` /
//! `fs::dir_ls(glob = ...)`. Complements `regex` (general regex) and
//! `aho-corasick` (multi-literal search).
//!
//! # Features
//!
//! Enable this module with the `globset` feature:
//!
//! ```toml
//! [dependencies]
//! miniextendr-api = { version = "0.1", features = ["globset"] }
//! ```
//!
//! # Default semantics
//!
//! [`GlobOptions::default()`] is **path-aware**: `literal_separator = true`,
//! so `*` / `?` / `[...]` never match `/` (`*.R` does not match `sub/a.R`).
//! This deliberately differs from raw `globset`, whose default lets `*`
//! cross path separators. `backslash_escape` is pinned to `true` on all
//! platforms (raw globset disables it on Windows).
//!
//! # Example
//!
//! ```ignore
//! use miniextendr_api::globset_impl::{GlobOptions, build_globset};
//!
//! #[miniextendr]
//! fn match_r_files(paths: Vec<String>) -> Result<Vec<bool>, String> {
//!     let set = build_globset(&["*.R".to_string()], &GlobOptions::default())
//!         .map_err(|e| e.to_string())?;
//!     Ok(paths.iter().map(|p| set.is_match(p)).collect())
//! }
//! ```

pub use globset::{Glob, GlobBuilder, GlobSet, GlobSetBuilder};

use crate::SEXP;
use crate::from_r::{SexpError, TryFromSexp};
use crate::{
    impl_option_try_from_sexp, impl_vec_option_try_from_sexp_list, impl_vec_try_from_sexp_list,
};

// region: Build options

/// Options forwarded to [`globset::GlobBuilder`] for every pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlobOptions {
    /// Require a literal `/` to match a path separator (`*` cannot cross `/`).
    /// Default: `true` (path-aware — differs from raw globset).
    pub literal_separator: bool,
    /// Case-insensitive matching. Default: `false`.
    pub case_insensitive: bool,
    /// Treat `\` as an escape character. Default: `true` on all platforms
    /// (raw globset disables this on Windows; we pin it for determinism).
    pub backslash_escape: bool,
}

impl Default for GlobOptions {
    fn default() -> Self {
        Self {
            literal_separator: true,
            case_insensitive: false,
            backslash_escape: true,
        }
    }
}
// endregion

// region: Helper functions

/// Build a [`GlobSet`] from patterns with the given options.
///
/// # Errors
///
/// Returns the underlying [`globset::Error`] if any pattern is invalid.
pub fn build_globset(patterns: &[String], opts: &GlobOptions) -> Result<GlobSet, globset::Error> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let glob = GlobBuilder::new(pattern)
            .literal_separator(opts.literal_separator)
            .case_insensitive(opts.case_insensitive)
            .backslash_escape(opts.backslash_escape)
            .build()?;
        builder.add(glob);
    }
    builder.build()
}

/// Match each path against the set: `TRUE` if ANY pattern matches.
pub fn globset_is_match(set: &GlobSet, paths: &[String]) -> Vec<bool> {
    paths.iter().map(|p| set.is_match(p)).collect()
}

/// Indices (1-based, for R) of the patterns that match `path`.
pub fn globset_matches(set: &GlobSet, path: &str) -> Vec<i32> {
    set.matches(path)
        .into_iter()
        .map(|i| i32::try_from(i + 1).expect("pattern index exceeds i32"))
        .collect()
}
// endregion

// region: TryFromSexp for GlobSet (from character vector of patterns)

/// Build a `GlobSet` from an R character vector of patterns using
/// [`GlobOptions::default()`] (path-aware).
///
/// # Errors
///
/// Returns an error if the input is not a character vector, is empty,
/// contains `NA`, or any pattern is an invalid glob.
impl TryFromSexp for GlobSet {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let patterns: Vec<String> = TryFromSexp::try_from_sexp(sexp)?;
        if patterns.is_empty() {
            return Err(SexpError::InvalidValue(
                "globset requires at least one pattern".to_string(),
            ));
        }
        build_globset(&patterns, &GlobOptions::default())
            .map_err(|e| SexpError::InvalidValue(format!("invalid glob pattern: {e}")))
    }
}

// Note: IntoR is intentionally not implemented for GlobSet.
// A compiled glob set cannot be meaningfully converted back to R.

impl_option_try_from_sexp!(GlobSet);
impl_vec_try_from_sexp_list!(GlobSet);
impl_vec_option_try_from_sexp_list!(GlobSet);
// endregion

// region: Unit tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_path_aware() {
        let set = build_globset(&["*.R".to_string()], &GlobOptions::default()).unwrap();
        let got = globset_is_match(
            &set,
            &[
                "a.R".to_string(),
                "a.Rmd".to_string(),
                "sub/a.R".to_string(),
            ],
        );
        assert_eq!(got, vec![true, false, false]);
    }

    #[test]
    fn non_literal_separator_crosses_slash() {
        let opts = GlobOptions {
            literal_separator: false,
            ..GlobOptions::default()
        };
        let set = build_globset(&["*.R".to_string()], &opts).unwrap();
        assert!(set.is_match("sub/a.R"));
    }

    #[test]
    fn case_insensitive() {
        let opts = GlobOptions {
            case_insensitive: true,
            ..GlobOptions::default()
        };
        let set = build_globset(&["*.r".to_string()], &opts).unwrap();
        assert!(set.is_match("A.R"));
    }

    #[test]
    fn multi_pattern_indices() {
        let set = build_globset(
            &["*.R".to_string(), "*.Rmd".to_string()],
            &GlobOptions::default(),
        )
        .unwrap();
        assert_eq!(globset_matches(&set, "a.Rmd"), vec![2]);
        assert!(set.is_match("a.R") && set.is_match("a.Rmd"));
        assert!(!set.is_match("a.py"));
    }

    #[test]
    fn invalid_glob_errors() {
        assert!(build_globset(&["a[".to_string()], &GlobOptions::default()).is_err());
    }

    #[test]
    fn recursive_glob() {
        let set = build_globset(&["**/*.rs".to_string()], &GlobOptions::default()).unwrap();
        assert!(set.is_match("src/lib.rs"));
        assert!(set.is_match("a/b/c.rs"));
    }
}
// endregion
