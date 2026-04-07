//! Aho-Corasick adapter tests
use miniextendr_api::aho_corasick_impl::{
    aho_compile, aho_count_matches, aho_find_all_flat, aho_is_match, aho_replace_all,
};
use miniextendr_api::miniextendr;

/// Test whether any pattern matches the haystack using Aho-Corasick.
/// @param patterns Character vector of patterns to search for.
/// @param haystack Single string to search within.
#[miniextendr]
pub fn aho_test_is_match(patterns: Vec<String>, haystack: String) -> bool {
    let ac = aho_compile(&patterns).unwrap();
    aho_is_match(&ac, &haystack)
}

/// Test counting the number of pattern matches in the haystack.
/// @param patterns Character vector of patterns to search for.
/// @param haystack Single string to search within.
#[miniextendr]
pub fn aho_test_count(patterns: Vec<String>, haystack: String) -> i32 {
    let ac = aho_compile(&patterns).unwrap();
    aho_count_matches(&ac, &haystack) as i32
}

/// Test finding all match positions as a flat integer vector.
/// @param patterns Character vector of patterns to search for.
/// @param haystack Single string to search within.
#[miniextendr]
pub fn aho_test_find_flat(patterns: Vec<String>, haystack: String) -> Vec<i32> {
    let ac = aho_compile(&patterns).unwrap();
    aho_find_all_flat(&ac, &haystack)
}

/// Test replacing all pattern matches with a replacement string.
/// @param patterns Character vector of patterns to search for.
/// @param haystack Single string to search within.
/// @param replacement Replacement string for each match.
#[miniextendr]
pub fn aho_test_replace(patterns: Vec<String>, haystack: String, replacement: String) -> String {
    let ac = aho_compile(&patterns).unwrap();
    aho_replace_all(&ac, &haystack, &replacement)
}

/// Test that non-matching patterns return a zero count.
/// @param patterns Character vector of patterns to search for.
/// @param haystack Single string to search within.
#[miniextendr]
pub fn aho_test_no_match(patterns: Vec<String>, haystack: String) -> i32 {
    let ac = aho_compile(&patterns).unwrap();
    aho_count_matches(&ac, &haystack) as i32
}

/// Test overlapping pattern detection with standard semantics.
/// @param haystack Single string to search within.
#[miniextendr]
pub fn aho_test_overlapping(haystack: String) -> i32 {
    // "he" and "she" overlap in "she": standard semantics finds both
    let ac = aho_compile(&["he".to_string(), "she".to_string()]).unwrap();
    aho_count_matches(&ac, &haystack) as i32
}

/// Test Aho-Corasick matching with Unicode patterns.
/// @param patterns Character vector of Unicode patterns to search for.
/// @param haystack Single string to search within.
#[miniextendr]
pub fn aho_test_unicode(patterns: Vec<String>, haystack: String) -> bool {
    let ac = aho_compile(&patterns).unwrap();
    aho_is_match(&ac, &haystack)
}

/// Test replacing matches with an empty string (deletion).
/// @param patterns Character vector of patterns to delete.
/// @param haystack Single string to search within.
#[miniextendr]
pub fn aho_test_replace_empty(patterns: Vec<String>, haystack: String) -> String {
    let ac = aho_compile(&patterns).unwrap();
    aho_replace_all(&ac, &haystack, "")
}
