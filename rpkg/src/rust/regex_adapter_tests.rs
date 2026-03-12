//! Regex adapter tests
use miniextendr_api::miniextendr;
use miniextendr_api::regex_impl::Regex;

/// @noRd
#[miniextendr]
pub fn regex_is_match(pattern: &str, text: &str) -> bool {
    Regex::new(pattern)
        .map(|re| re.is_match(text))
        .unwrap_or(false)
}

/// @noRd
#[miniextendr]
pub fn regex_find(pattern: &str, text: &str) -> Option<String> {
    Regex::new(pattern)
        .ok()?
        .find(text)
        .map(|m| m.as_str().to_string())
}

/// @noRd
#[miniextendr]
pub fn regex_find_all(pattern: &str, text: &str) -> Vec<String> {
    Regex::new(pattern)
        .map(|re| re.find_iter(text).map(|m| m.as_str().to_string()).collect())
        .unwrap_or_default()
}

/// @noRd
#[miniextendr]
pub fn regex_replace_first(pattern: &str, text: &str, replacement: &str) -> String {
    Regex::new(pattern)
        .map(|re| re.replace(text, replacement).into_owned())
        .unwrap_or_else(|_| text.to_string())
}

/// @noRd
#[miniextendr]
pub fn regex_replace_all(pattern: &str, text: &str, replacement: &str) -> String {
    Regex::new(pattern)
        .map(|re| re.replace_all(text, replacement).into_owned())
        .unwrap_or_else(|_| text.to_string())
}

/// @noRd
#[miniextendr]
pub fn regex_split(pattern: &str, text: &str) -> Vec<String> {
    Regex::new(pattern)
        .map(|re| re.split(text).map(|s| s.to_string()).collect())
        .unwrap_or_else(|_| vec![text.to_string()])
}
