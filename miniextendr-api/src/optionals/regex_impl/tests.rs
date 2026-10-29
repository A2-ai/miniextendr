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
