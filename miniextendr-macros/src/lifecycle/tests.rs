use super::*;

#[test]
fn test_stage_from_str() {
    assert_eq!(
        LifecycleStage::from_str("experimental"),
        Some(LifecycleStage::Experimental)
    );
    assert_eq!(
        LifecycleStage::from_str("deprecated"),
        Some(LifecycleStage::Deprecated)
    );
    assert_eq!(
        LifecycleStage::from_str("soft-deprecated"),
        Some(LifecycleStage::SoftDeprecated)
    );
    assert_eq!(LifecycleStage::from_str("unknown"), None);
}

#[test]
fn test_r_prelude_deprecated() {
    let spec = LifecycleSpec {
        stage: LifecycleStage::Deprecated,
        when: Some("0.4.0".into()),
        what: None,
        with: Some("new_fn()".into()),
        details: None,
        id: None,
    };
    let prelude = spec.r_prelude("old_fn").unwrap();
    assert!(prelude.contains("lifecycle::deprecate_warn"));
    assert!(prelude.contains("0.4.0"));
    assert!(prelude.contains("old_fn()"));
    assert!(prelude.contains("new_fn()"));
}

/// User text lands in R string literals: `\` and `"` are escaped, `\` first.
#[test]
fn test_r_prelude_escapes_string_arguments() {
    let spec = LifecycleSpec {
        stage: LifecycleStage::Deprecated,
        when: Some("0.4.0".into()),
        what: None,
        with: Some("new_fn()".into()),
        details: Some(r#"Use "C:\data" instead."#.into()),
        id: None,
    };
    let prelude = spec.r_prelude("old_fn").unwrap();
    assert!(
        prelude.contains(r#"details = "Use \"C:\\data\" instead.""#),
        "{prelude}"
    );
}

#[test]
fn test_r_prelude_experimental() {
    let spec = LifecycleSpec::new(LifecycleStage::Experimental);
    let prelude = spec.r_prelude("my_fn").unwrap();
    assert!(prelude.contains("lifecycle::signal_stage"));
    assert!(prelude.contains("experimental"));
    assert!(prelude.contains("my_fn()"));
}

#[test]
fn test_from_deprecated_note() {
    let spec = LifecycleSpec::from_deprecated(Some("1.0.0"), Some("Use bar() instead"));
    assert_eq!(spec.stage, LifecycleStage::Deprecated);
    assert_eq!(spec.when, Some("1.0.0".into()));
    assert_eq!(spec.with, Some("bar()".into()));
}

#[test]
fn test_import_from_fn() {
    assert_eq!(
        LifecycleStage::Experimental.import_from_fn(),
        Some("signal_stage")
    );
    assert_eq!(LifecycleStage::Stable.import_from_fn(), None);
    assert_eq!(
        LifecycleStage::Superseded.import_from_fn(),
        Some("signal_stage")
    );
    assert_eq!(
        LifecycleStage::SoftDeprecated.import_from_fn(),
        Some("deprecate_soft")
    );
    assert_eq!(
        LifecycleStage::Deprecated.import_from_fn(),
        Some("deprecate_warn")
    );
    assert_eq!(
        LifecycleStage::Defunct.import_from_fn(),
        Some("deprecate_stop")
    );
}

#[test]
fn test_inject_lifecycle_imports_deprecated() {
    let spec = LifecycleSpec::new(LifecycleStage::Deprecated);
    let mut tags = vec!["@title My function".to_string()];
    inject_lifecycle_badge(&mut tags, &spec);
    // Should have @importFrom lifecycle badge deprecate_warn
    let import_tag = tags
        .iter()
        .find(|t| t.starts_with("@importFrom lifecycle"))
        .expect("should have @importFrom lifecycle tag");
    assert!(import_tag.contains("badge"));
    assert!(import_tag.contains("deprecate_warn"));
}

#[test]
fn test_inject_lifecycle_imports_experimental() {
    let spec = LifecycleSpec::new(LifecycleStage::Experimental);
    let mut tags = vec!["@title My function".to_string()];
    inject_lifecycle_badge(&mut tags, &spec);
    let import_tag = tags
        .iter()
        .find(|t| t.starts_with("@importFrom lifecycle"))
        .expect("should have @importFrom lifecycle tag");
    assert!(import_tag.contains("badge"));
    assert!(import_tag.contains("signal_stage"));
}

#[test]
fn test_inject_lifecycle_imports_stable_no_import() {
    let spec = LifecycleSpec::new(LifecycleStage::Stable);
    let mut tags = vec!["@title My function".to_string()];
    inject_lifecycle_badge(&mut tags, &spec);
    // Stable stage should not add any @importFrom
    assert!(!tags.iter().any(|t| t.starts_with("@importFrom")));
}

#[test]
fn test_inject_lifecycle_imports_no_duplicates() {
    let spec = LifecycleSpec::new(LifecycleStage::Deprecated);
    let mut tags = vec![
        "@title My function".to_string(),
        "@importFrom lifecycle deprecate_warn badge".to_string(),
    ];
    inject_lifecycle_badge(&mut tags, &spec);
    // Should not add a duplicate @importFrom tag
    let import_count = tags
        .iter()
        .filter(|t| t.starts_with("@importFrom lifecycle"))
        .count();
    assert_eq!(import_count, 1);
}

#[test]
fn test_collect_lifecycle_imports_mixed_methods() {
    let specs = [
        LifecycleSpec::new(LifecycleStage::Deprecated),
        LifecycleSpec::new(LifecycleStage::Experimental),
        LifecycleSpec::new(LifecycleStage::Stable),
    ];
    let result = collect_lifecycle_imports(specs.iter());
    let import = result.expect("should produce import tag");
    // BTreeSet gives sorted order: badge, deprecate_warn, signal_stage
    assert_eq!(
        import,
        "@importFrom lifecycle badge deprecate_warn signal_stage"
    );
}

#[test]
fn test_collect_lifecycle_imports_no_lifecycle() {
    let specs: Vec<LifecycleSpec> = vec![];
    let result = collect_lifecycle_imports(specs.iter());
    assert!(result.is_none());
}

#[test]
fn test_collect_lifecycle_imports_only_stable() {
    let specs = [LifecycleSpec::new(LifecycleStage::Stable)];
    let result = collect_lifecycle_imports(specs.iter());
    assert!(result.is_none());
}
