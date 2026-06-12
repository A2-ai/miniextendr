//! globset adapter tests
use miniextendr_api::globset_impl::{self, GlobOptions, build_globset};
use miniextendr_api::miniextendr;

/// Match paths against glob patterns (path-aware defaults).
/// Returns TRUE for each path matched by ANY pattern.
/// @param patterns Character vector of glob patterns.
/// @param paths Character vector of paths to test.
#[miniextendr]
pub fn globset_is_match(patterns: Vec<String>, paths: Vec<String>) -> Result<Vec<bool>, String> {
    let set = build_globset(&patterns, &GlobOptions::default()).map_err(|e| e.to_string())?;
    Ok(globset_impl::globset_is_match(&set, &paths))
}

/// Match paths with explicit builder options.
/// @param patterns Character vector of glob patterns.
/// @param paths Character vector of paths to test.
/// @param literal_separator If TRUE, `*` cannot match `/`.
/// @param case_insensitive If TRUE, matching ignores case.
/// @param backslash_escape If TRUE, backslash escapes meta-characters.
#[miniextendr]
pub fn globset_is_match_opts(
    patterns: Vec<String>,
    paths: Vec<String>,
    literal_separator: bool,
    case_insensitive: bool,
    backslash_escape: bool,
) -> Result<Vec<bool>, String> {
    let opts = GlobOptions {
        literal_separator,
        case_insensitive,
        backslash_escape,
    };
    let set = build_globset(&patterns, &opts).map_err(|e| e.to_string())?;
    Ok(globset_impl::globset_is_match(&set, &paths))
}

/// 1-based indices of the patterns matching a single path.
/// @param patterns Character vector of glob patterns.
/// @param path Single path to test.
#[miniextendr]
pub fn globset_which_match(patterns: Vec<String>, path: String) -> Result<Vec<i32>, String> {
    let set = build_globset(&patterns, &GlobOptions::default()).map_err(|e| e.to_string())?;
    Ok(globset_impl::globset_matches(&set, &path))
}
