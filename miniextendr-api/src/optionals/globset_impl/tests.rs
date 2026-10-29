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
