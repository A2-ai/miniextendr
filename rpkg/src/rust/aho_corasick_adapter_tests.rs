//! Aho-Corasick adapter tests
use miniextendr_api::aho_corasick_impl::{aho_compile, aho_count_matches, aho_find_all_flat, aho_is_match, aho_replace_all};
use miniextendr_api::{miniextendr, miniextendr_module};

/// @noRd
#[miniextendr]
pub fn aho_test_is_match(patterns: Vec<String>, haystack: String) -> bool {
    let ac = aho_compile(&patterns).unwrap();
    aho_is_match(&ac, &haystack)
}

/// @noRd
#[miniextendr]
pub fn aho_test_count(patterns: Vec<String>, haystack: String) -> i32 {
    let ac = aho_compile(&patterns).unwrap();
    aho_count_matches(&ac, &haystack) as i32
}

/// @noRd
#[miniextendr]
pub fn aho_test_find_flat(patterns: Vec<String>, haystack: String) -> Vec<i32> {
    let ac = aho_compile(&patterns).unwrap();
    aho_find_all_flat(&ac, &haystack)
}

/// @noRd
#[miniextendr]
pub fn aho_test_replace(patterns: Vec<String>, haystack: String, replacement: String) -> String {
    let ac = aho_compile(&patterns).unwrap();
    aho_replace_all(&ac, &haystack, &replacement)
}

/// No match returns false
/// @noRd
#[miniextendr]
pub fn aho_test_no_match(patterns: Vec<String>, haystack: String) -> i32 {
    let ac = aho_compile(&patterns).unwrap();
    aho_count_matches(&ac, &haystack) as i32
}

/// Overlapping patterns - default standard semantics reports all
/// @noRd
#[miniextendr]
pub fn aho_test_overlapping(haystack: String) -> i32 {
    // "he" and "she" overlap in "she": standard semantics finds both
    let ac = aho_compile(&["he".to_string(), "she".to_string()]).unwrap();
    aho_count_matches(&ac, &haystack) as i32
}

/// Unicode patterns
/// @noRd
#[miniextendr]
pub fn aho_test_unicode(patterns: Vec<String>, haystack: String) -> bool {
    let ac = aho_compile(&patterns).unwrap();
    aho_is_match(&ac, &haystack)
}

/// Replace with empty string (deletion)
/// @noRd
#[miniextendr]
pub fn aho_test_replace_empty(patterns: Vec<String>, haystack: String) -> String {
    let ac = aho_compile(&patterns).unwrap();
    aho_replace_all(&ac, &haystack, "")
}

miniextendr_module! {
    mod aho_corasick_adapter_tests;
    fn aho_test_is_match;
    fn aho_test_count;
    fn aho_test_find_flat;
    fn aho_test_replace;
    fn aho_test_no_match;
    fn aho_test_overlapping;
    fn aho_test_unicode;
    fn aho_test_replace_empty;
}
