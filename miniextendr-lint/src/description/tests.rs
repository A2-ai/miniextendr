use super::*;

#[test]
fn parses_package_and_dependencies_across_continuation_lines() {
    let d = Description::parse(
        "Package: mypkg\nImports: cli (>= 3.0),\n    rlang,\n    vctrs\nSuggests: testthat\nLinkingTo: cpp11\nTitle: x, y\n",
    )
    .unwrap();
    assert_eq!(d.package, "mypkg");
    for dep in ["cli", "rlang", "vctrs", "testthat", "cpp11"] {
        assert!(d.declared.contains(dep), "{dep}");
    }
    assert!(!d.declared.contains("x"));
    assert!(d.resolves("mypkg") && d.resolves("stats") && d.resolves("MASS"));
    assert!(!d.resolves("Sources") && !d.resolves("dplyr"));
}

#[test]
fn a_file_without_a_package_field_is_none() {
    assert_eq!(Description::parse("Title: x\n"), None);
}
