//! Regex adapter tests
use miniextendr_api::miniextendr;
use miniextendr_api::regex_impl::Regex;

/// Test whether a regex pattern matches the given text.
/// @param pattern Regular expression pattern.
/// @param text String to match against.
#[miniextendr]
pub fn regex_is_match(pattern: &str, text: &str) -> bool {
    Regex::new(pattern)
        .map(|re| re.is_match(text))
        .unwrap_or(false)
}

/// Test finding the first regex match in the text.
/// @param pattern Regular expression pattern.
/// @param text String to search within.
#[miniextendr]
pub fn regex_find(pattern: &str, text: &str) -> Option<String> {
    Regex::new(pattern)
        .ok()?
        .find(text)
        .map(|m| m.as_str().to_string())
}

/// Test finding all regex matches in the text.
/// @param pattern Regular expression pattern.
/// @param text String to search within.
#[miniextendr]
pub fn regex_find_all(pattern: &str, text: &str) -> Vec<String> {
    Regex::new(pattern)
        .map(|re| re.find_iter(text).map(|m| m.as_str().to_string()).collect())
        .unwrap_or_default()
}

/// Test replacing the first regex match with a replacement string.
/// @param pattern Regular expression pattern.
/// @param text String to search within.
/// @param replacement Replacement string.
#[miniextendr]
pub fn regex_replace_first(pattern: &str, text: &str, replacement: &str) -> String {
    Regex::new(pattern)
        .map(|re| re.replace(text, replacement).into_owned())
        .unwrap_or_else(|_| text.to_string())
}

/// Test replacing all regex matches with a replacement string.
/// @param pattern Regular expression pattern.
/// @param text String to search within.
/// @param replacement Replacement string.
#[miniextendr]
pub fn regex_replace_all(pattern: &str, text: &str, replacement: &str) -> String {
    Regex::new(pattern)
        .map(|re| re.replace_all(text, replacement).into_owned())
        .unwrap_or_else(|_| text.to_string())
}

/// Test splitting a string by a regex pattern.
/// @param pattern Regular expression pattern to split on.
/// @param text String to split.
#[miniextendr]
pub fn regex_split(pattern: &str, text: &str) -> Vec<String> {
    Regex::new(pattern)
        .map(|re| re.split(text).map(|s| s.to_string()).collect())
        .unwrap_or_else(|_| vec![text.to_string()])
}

// region: Upstream example-derived fixtures

/// Extract all capture group matches from a regex.
/// Returns all captured groups (including the full match at index 0) for the first match.
/// @param pattern Regular expression pattern with capture groups.
/// @param text String to search within.
#[miniextendr]
pub fn regex_captures(pattern: &str, text: &str) -> Vec<String> {
    Regex::new(pattern)
        .ok()
        .and_then(|re| {
            re.captures(text).map(|caps| {
                caps.iter()
                    .map(|m| m.map(|m| m.as_str().to_string()).unwrap_or_default())
                    .collect()
            })
        })
        .unwrap_or_default()
}

/// Count the number of non-overlapping matches.
/// @param pattern Regular expression pattern.
/// @param text String to search within.
#[miniextendr]
pub fn regex_count(pattern: &str, text: &str) -> i32 {
    Regex::new(pattern)
        .map(|re| re.find_iter(text).count() as i32)
        .unwrap_or(0)
}

// endregion

// region: RRegexOps + CaptureGroups adapter traits

/// Drive a compiled `Regex` through the `RRegexOps` adapter trait (audit A7 —
/// the free-function fixtures above call inherent `Regex` methods; the trait
/// was unexercised). Calls are trait-qualified.
/// @param pattern Regular expression pattern.
/// @param text String to operate on.
/// @param replacement Replacement string.
#[miniextendr]
pub fn regex_ops_via_trait(pattern: &str, text: &str, replacement: &str) -> Vec<String> {
    use miniextendr_api::regex_impl::RRegexOps;

    let Ok(re) = Regex::new(pattern) else {
        return vec!["INVALID_PATTERN".to_string()];
    };
    vec![
        RRegexOps::is_match(&re, text).to_string(),
        RRegexOps::find(&re, text).unwrap_or_default(),
        RRegexOps::replace_first(&re, text, replacement),
        RRegexOps::replace_all(&re, text, replacement),
        RRegexOps::find_all(&re, text).join(","),
        RRegexOps::split(&re, text).join("|"),
        RRegexOps::captures_len(&re).to_string(),
    ]
}

/// Capture the first match into a `CaptureGroups` value and return all groups
/// (index 0 = whole match; `NA` for non-participating groups).
/// @param pattern Regular expression pattern with capture groups.
/// @param text String to search within.
#[miniextendr]
pub fn regex_capture_groups_all(pattern: &str, text: &str) -> Vec<Option<String>> {
    use miniextendr_api::regex_impl::{CaptureGroups, RCaptureGroups};

    let Ok(re) = Regex::new(pattern) else {
        return vec![];
    };
    match CaptureGroups::capture(&re, text) {
        Some(caps) => RCaptureGroups::all_groups(&caps),
        None => vec![],
    }
}

/// Look up capture groups by index and by name via `RCaptureGroups`.
/// @param pattern Regular expression pattern with a named group.
/// @param text String to search within.
/// @param name Name of the capture group to fetch.
#[miniextendr]
pub fn regex_capture_group_named(pattern: &str, text: &str, name: &str) -> Vec<Option<String>> {
    use miniextendr_api::regex_impl::{CaptureGroups, RCaptureGroups};

    let Ok(re) = Regex::new(pattern) else {
        return vec![];
    };
    match CaptureGroups::capture(&re, text) {
        Some(caps) => vec![
            RCaptureGroups::get(&caps, 0),
            RCaptureGroups::get_named(&caps, name),
            Some(RCaptureGroups::len(&caps).to_string()),
        ],
        None => vec![],
    }
}

// endregion
