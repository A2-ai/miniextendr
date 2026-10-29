use super::*;

fn postfix(text: &str) -> Result<Option<String>, String> {
    parse_crate_config(text).map(|c| c.noexport_postfix)
}

#[test]
fn header_form_is_read() {
    let text = r#"
[package]
name = "demo"

[package.metadata.miniextendr]
noexport_postfix = "_impl" # trailing comment

[dependencies]
miniextendr-api = "0.1"
"#;
    assert_eq!(postfix(text), Ok(Some("_impl".to_string())));
}

#[test]
fn dotted_forms_are_read() {
    let under_metadata = "[package.metadata]\nminiextendr.noexport_postfix = '_impl'\n";
    assert_eq!(postfix(under_metadata), Ok(Some("_impl".to_string())));

    let under_package = "[package]\nmetadata.miniextendr.noexport_postfix = \"_rs\"\n";
    assert_eq!(postfix(under_package), Ok(Some("_rs".to_string())));

    let quoted_header =
        "[ \"package\" . metadata . 'miniextendr' ]\nnoexport_postfix = \"_impl\"\n";
    assert_eq!(postfix(quoted_header), Ok(Some("_impl".to_string())));
}

#[test]
fn unrelated_tables_and_keys_are_ignored() {
    let text = r#"
[package.metadata.docs.rs]
noexport_postfix = "not-ours"

[package.metadata.miniextendr]
other_key = "ignored"

[[bin]]
name = "x"
noexport_postfix = "also not ours"
"#;
    assert_eq!(postfix(text), Ok(None));
    assert_eq!(postfix(""), Ok(None));
    assert_eq!(postfix("[package]\nname = \"demo\"\n"), Ok(None));
}

#[test]
fn source_tags_is_read_and_defaults_off() {
    let source_tags = |text: &str| parse_crate_config(text).map(|c| c.source_tags);
    assert_eq!(source_tags(""), Ok(false));
    assert_eq!(
        source_tags("[package.metadata.miniextendr]\nsource_tags = true # provenance\n"),
        Ok(true)
    );
    assert_eq!(
        source_tags("[package.metadata]\nminiextendr.source_tags = false\n"),
        Ok(false)
    );
    // Both keys in one table.
    let both = "[package.metadata.miniextendr]\nnoexport_postfix = \"_impl\"\nsource_tags = true\n";
    let config = parse_crate_config(both).unwrap();
    assert_eq!(config.noexport_postfix.as_deref(), Some("_impl"));
    assert!(config.source_tags);
    assert!(
        source_tags("[package.metadata.miniextendr]\nsource_tags = \"yes\"\n")
            .unwrap_err()
            .contains("must be `true` or `false`")
    );
    assert!(
        source_tags("[package.metadata.miniextendr]\nsource_tags = true\nsource_tags = true\n")
            .unwrap_err()
            .contains("more than once")
    );
}

#[test]
fn preconditions_is_read_and_defaults_unset() {
    let preconditions = |text: &str| parse_crate_config(text).map(|c| c.preconditions);
    assert_eq!(preconditions(""), Ok(None));
    assert_eq!(
        preconditions("[package.metadata.miniextendr]\npreconditions = false # hot paths\n"),
        Ok(Some(false))
    );
    assert_eq!(
        preconditions("[package.metadata]\nminiextendr.preconditions = true\n"),
        Ok(Some(true))
    );
    assert!(
        preconditions("[package.metadata.miniextendr]\npreconditions = \"no\"\n")
            .unwrap_err()
            .contains("`preconditions` must be `true` or `false`, found `\"no\"`")
    );
    assert!(
        preconditions(
            "[package.metadata.miniextendr]\npreconditions = false\npreconditions = true\n"
        )
        .unwrap_err()
        .contains("`preconditions` is set more than once")
    );
    // Next to the other keys.
    let all = "[package.metadata.miniextendr]\nsource_tags = true\npreconditions = false\ncall_attribution = \"caller\"\n";
    let config = parse_crate_config(all).unwrap();
    assert_eq!(config.preconditions, Some(false));
    assert!(config.source_tags);
}

#[test]
fn roxygen_prose_links_is_read_and_validated() {
    let links = |text: &str| parse_crate_config(text).map(|c| c.roxygen_prose_links);
    assert_eq!(links(""), Ok(None));
    assert_eq!(
        links("[package.metadata.miniextendr]\nroxygen_prose_links = \"keep\" # R-first docs\n"),
        Ok(Some(ProseLinks::Keep))
    );
    assert_eq!(
        links("[package.metadata]\nminiextendr.roxygen_prose_links = 'strip'\n"),
        Ok(Some(ProseLinks::Strip))
    );
    assert_eq!(
        links("[package]\nmetadata.miniextendr.roxygen_prose_links = \"keep\"\n"),
        Ok(Some(ProseLinks::Keep))
    );
    for value in ["\"preserve\"", "true", "keep"] {
        let bad = links(&format!(
            "[package.metadata.miniextendr]\nroxygen_prose_links = {value}\n"
        ))
        .unwrap_err();
        assert!(
            bad.contains("`roxygen_prose_links` must be one of \"strip\", \"keep\"")
                && bad.contains(&format!("found `{value}`")),
            "{bad}"
        );
    }
    let twice = links(
        "[package.metadata.miniextendr]\nroxygen_prose_links = \"keep\"\nroxygen_prose_links = \"strip\"\n",
    )
    .unwrap_err();
    assert!(
        twice.contains("`roxygen_prose_links` is set more than once"),
        "{twice}"
    );
    // Alongside the other keys.
    let all = "[package.metadata.miniextendr]\nnoexport_postfix = \"_impl\"\nroxygen_prose_links = \"keep\"\nsource_tags = true\n";
    let config = parse_crate_config(all).unwrap();
    assert_eq!(config.noexport_postfix.as_deref(), Some("_impl"));
    assert_eq!(config.roxygen_prose_links, Some(ProseLinks::Keep));
    assert!(config.source_tags);
}

#[test]
fn call_attribution_is_read_and_validated() {
    let attribution = |text: &str| parse_crate_config(text).map(|c| c.call_attribution);
    assert_eq!(attribution(""), Ok(None));
    assert_eq!(
        attribution(
            "[package.metadata.miniextendr]\ncall_attribution = \"caller\" # internal entry points\n"
        ),
        Ok(Some(CallAttribution::Caller))
    );
    assert_eq!(
        attribution("[package.metadata]\nminiextendr.call_attribution = 'wrapper'\n"),
        Ok(Some(CallAttribution::Wrapper))
    );
    assert_eq!(
        attribution("[package]\nmetadata.miniextendr.call_attribution = \"wrapper\"\n"),
        Ok(Some(CallAttribution::Wrapper))
    );
    assert_eq!(
        attribution("[package.metadata.miniextendr]\ncall_attribution = 'none'\n"),
        Ok(Some(CallAttribution::NoCall))
    );
    for value in ["\"parent\"", "'self'"] {
        let bad = attribution(&format!(
            "[package.metadata.miniextendr]\ncall_attribution = {value}\n"
        ))
        .unwrap_err();
        assert!(
            bad.contains("must be one of \"wrapper\", \"caller\", \"none\"")
                && bad.contains(&format!("`{value}`")),
            "{bad}"
        );
    }
    let bare =
        attribution("[package.metadata.miniextendr]\ncall_attribution = caller\n").unwrap_err();
    assert!(bare.contains("must be one of"), "{bare}");
    let twice = attribution(
        "[package.metadata.miniextendr]\ncall_attribution = \"wrapper\"\ncall_attribution = \"caller\"\n",
    )
    .unwrap_err();
    assert!(twice.contains("more than once"), "{twice}");
    // Alongside the other keys.
    let all = "[package.metadata.miniextendr]\nnoexport_postfix = \"_impl\"\nsource_tags = true\ncall_attribution = \"caller\"\n";
    let config = parse_crate_config(all).unwrap();
    assert_eq!(config.noexport_postfix.as_deref(), Some("_impl"));
    assert!(config.source_tags);
    assert_eq!(config.call_attribution, Some(CallAttribution::Caller));
}

#[test]
fn conversion_error_class_is_read_and_validated() {
    let classes = |text: &str| parse_crate_config(text).map(|c| c.conversion_error_class);
    let owned = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    assert_eq!(classes(""), Ok(Vec::new()));
    // A single string and the array spellings.
    assert_eq!(
        classes("[package.metadata.miniextendr]\nconversion_error_class = \"pkg_error\"\n"),
        Ok(owned(&["pkg_error"]))
    );
    assert_eq!(
        classes(
            "[package.metadata.miniextendr]\nconversion_error_class = [\"pkg_error_argument\", 'pkg_error'] # family last\n"
        ),
        Ok(owned(&["pkg_error_argument", "pkg_error"]))
    );
    assert_eq!(
        classes("[package.metadata]\nminiextendr.conversion_error_class = [ \"a\" , \"b\" , ]\n"),
        Ok(owned(&["a", "b"]))
    );
    assert_eq!(
        classes("[package]\nmetadata.miniextendr.conversion_error_class = ['pkg.error']\n"),
        Ok(owned(&["pkg.error"]))
    );

    let err = |text: &str| classes(text).unwrap_err();
    let table = "[package.metadata.miniextendr]\n";
    for bad in [
        "conversion_error_class = pkg_error\n",
        "conversion_error_class = 1\n",
        "conversion_error_class = [\"a\", 1]\n",
        "conversion_error_class = [\"a\" \"b\"]\n",
        "conversion_error_class = [\"a\"] trailing\n",
        "conversion_error_class = [\"a\\tb\"]\n",
        "conversion_error_class = [\n",
    ] {
        let msg = err(&format!("{table}{bad}"));
        assert!(msg.contains("single-line array of strings"), "{bad}: {msg}");
    }
    assert!(err(&format!("{table}conversion_error_class = []\n")).contains("at least one"));
    assert!(err(&format!("{table}conversion_error_class = [\"\"]\n")).contains("not be empty"));
    let layered = err(&format!(
        "{table}conversion_error_class = [\"pkg_error\", \"rust_error\"]\n"
    ));
    assert!(layered.contains("must not name `rust_error`"), "{layered}");
    let dup = err(&format!("{table}conversion_error_class = [\"a\", \"a\"]\n"));
    assert!(dup.contains("more than once"), "{dup}");
    let twice = err(&format!(
        "{table}conversion_error_class = \"a\"\nconversion_error_class = \"b\"\n"
    ));
    assert!(twice.contains("is set more than once"), "{twice}");

    // Alongside the other keys.
    let all = "[package.metadata.miniextendr]\nnoexport_postfix = \"_impl\"\nconversion_error_class = [\"pkg_error_argument\", \"pkg_error\"]\ncall_attribution = \"caller\"\n";
    let config = parse_crate_config(all).unwrap();
    assert_eq!(config.noexport_postfix.as_deref(), Some("_impl"));
    assert_eq!(config.call_attribution, Some(CallAttribution::Caller));
    assert_eq!(
        config.conversion_error_class,
        owned(&["pkg_error_argument", "pkg_error"])
    );
}

#[test]
fn malformed_values_are_errors() {
    let empty = "[package.metadata.miniextendr]\nnoexport_postfix = \"\"\n";
    assert!(postfix(empty).unwrap_err().contains("must not be empty"));

    let bad_chars = "[package.metadata.miniextendr]\nnoexport_postfix = \"-impl\"\n";
    assert!(
        postfix(bad_chars)
            .unwrap_err()
            .contains("identifier fragment")
    );

    let not_a_string = "[package.metadata.miniextendr]\nnoexport_postfix = 1\n";
    assert!(
        postfix(not_a_string)
            .unwrap_err()
            .contains("must be a string")
    );

    let escaped = "[package.metadata.miniextendr]\nnoexport_postfix = \"a\\tb\"\n";
    assert!(postfix(escaped).unwrap_err().contains("must be a string"));

    let multiline = "[package.metadata.miniextendr]\nnoexport_postfix = \"\"\"_impl\"\"\"\n";
    assert!(postfix(multiline).unwrap_err().contains("must be a string"));

    let twice =
        "[package.metadata.miniextendr]\nnoexport_postfix = \"_a\"\nnoexport_postfix = \"_b\"\n";
    assert!(postfix(twice).unwrap_err().contains("more than once"));

    let inline = "[package.metadata]\nminiextendr = { noexport_postfix = \"_impl\" }\n";
    assert!(postfix(inline).unwrap_err().contains("inline tables"));
}

#[test]
fn manifest_dir_lookup_reads_and_caches() {
    let dir = std::env::temp_dir().join(format!(
        "miniextendr-crate-config-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or_default()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"demo\"\n\n[package.metadata.miniextendr]\nnoexport_postfix = \"_impl\"\n",
    )
    .unwrap();

    let first = crate_config_for_dir(&dir).unwrap();
    assert_eq!(first.noexport_postfix.as_deref(), Some("_impl"));

    // The cache answers subsequent lookups; a later edit is not observed
    // within one process (Cargo starts a new rustc per manifest change).
    std::fs::write(dir.join("Cargo.toml"), "[package]\nname = \"demo\"\n").unwrap();
    let second = crate_config_for_dir(&dir).unwrap();
    assert_eq!(second, first);

    let missing = crate_config_for_dir(&dir.join("does-not-exist")).unwrap();
    assert_eq!(missing, CrateConfig::default());

    let _ = std::fs::remove_dir_all(&dir);
}
