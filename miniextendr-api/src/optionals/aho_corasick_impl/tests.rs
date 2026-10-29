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
