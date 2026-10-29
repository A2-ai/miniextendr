use super::*;
use tempfile::TempDir;

fn write_toml(dir: &Path, content: &str) -> std::path::PathBuf {
    let path = dir.join("Cargo.toml");
    std::fs::write(&path, content).unwrap();
    path
}

#[test]
fn strip_toml_removes_bench_sections() {
    let dir = TempDir::new().unwrap();
    let path = write_toml(
        dir.path(),
        "[package]\nname = \"foo\"\n\n[dependencies]\nbar = \"1\"\n\n[[bench]]\nname = \"my_bench\"\nharness = false\n\n[lib]\nname = \"foo\"\n",
    );
    strip_toml_sections(&path, &["[[bench]]"]).unwrap();
    let result = std::fs::read_to_string(&path).unwrap();
    assert!(!result.contains("[[bench]]"));
    assert!(!result.contains("my_bench"));
    assert!(result.contains("[dependencies]"));
    assert!(result.contains("[lib]"));
}

#[test]
fn strip_toml_removes_test_sections() {
    let dir = TempDir::new().unwrap();
    let path = write_toml(
        dir.path(),
        "[package]\nname = \"foo\"\n\n[[test]]\nname = \"integration\"\n\n[dependencies]\nbar = \"1\"\n",
    );
    strip_toml_sections(&path, &["[[test]]"]).unwrap();
    let result = std::fs::read_to_string(&path).unwrap();
    assert!(!result.contains("[[test]]"));
    assert!(result.contains("[dependencies]"));
}

#[test]
fn strip_toml_removes_dev_dependencies() {
    let dir = TempDir::new().unwrap();
    let path = write_toml(
        dir.path(),
        "[package]\nname = \"foo\"\n\n[dependencies]\nbar = \"1\"\n\n[dev-dependencies]\ncriterion = \"0.5\"\n\n[features]\ndefault = []\n",
    );
    strip_toml_sections(&path, &["[dev-dependencies"]).unwrap();
    let result = std::fs::read_to_string(&path).unwrap();
    assert!(!result.contains("[dev-dependencies]"));
    assert!(!result.contains("criterion"));
    assert!(result.contains("[features]"));
    // No dangling refs to criterion exist in this fixture, default = [] survives
    assert!(result.contains("default = []"));
}

#[test]
fn strip_toml_removes_example_sections() {
    let dir = TempDir::new().unwrap();
    let path = write_toml(
        dir.path(),
        "[package]\nname = \"foo\"\n\n[[example]]\nname = \"demo\"\n\n[dependencies]\nbar = \"1\"\n",
    );
    strip_toml_sections(&path, &["[[example]]"]).unwrap();
    let result = std::fs::read_to_string(&path).unwrap();
    assert!(!result.contains("[[example]]"));
    assert!(result.contains("[dependencies]"));
}

#[test]
fn strip_toml_preserves_regular_sections() {
    let dir = TempDir::new().unwrap();
    let path = write_toml(
        dir.path(),
        "[package]\nname = \"foo\"\n\n[dependencies]\nbar = \"1\"\n\n[build-dependencies]\ncc = \"1\"\n\n[features]\ndefault = []\n\n[profile.release]\nopt-level = 3\n",
    );
    strip_toml_sections(&path, &["[[test]]", "[[bench]]", "[dev-dependencies"]).unwrap();
    let result = std::fs::read_to_string(&path).unwrap();
    assert!(result.contains("[package]"));
    assert!(result.contains("[dependencies]"));
    assert!(result.contains("[build-dependencies]"));
    assert!(result.contains("[features]"));
    assert!(result.contains("[profile.release]"));
}

#[test]
fn strip_toml_handles_multiple_stripped_sections() {
    let dir = TempDir::new().unwrap();
    let path = write_toml(
        dir.path(),
        "[package]\nname = \"foo\"\n\n[[test]]\nname = \"t1\"\n\n[[test]]\nname = \"t2\"\n\n[[bench]]\nname = \"b1\"\n\n[dev-dependencies]\ncriterion = \"0.5\"\n\n[features]\ndefault = []\n",
    );
    strip_toml_sections(&path, &["[[test]]", "[[bench]]", "[dev-dependencies"]).unwrap();
    let result = std::fs::read_to_string(&path).unwrap();
    assert!(!result.contains("[[test]]"));
    assert!(!result.contains("[[bench]]"));
    assert!(!result.contains("[dev-dependencies]"));
    assert!(result.contains("[features]"));
}

// region: prune_dangling_feature_refs (#322)

#[test]
fn prune_dangling_feature_refs_removes_dep_slash_feature() {
    let toml = r#"[package]
name = "foo"
version = "0.1.0"

[dev-dependencies]
criterion = "0.5"

[features]
real_blackbox = ["criterion/real_blackbox"]
default = []
"#;
    let result = prune_dangling_feature_refs(toml, &["criterion".to_string()]);
    // criterion/real_blackbox should be gone
    assert!(
        !result.contains("criterion/real_blackbox"),
        "dangling dep-feature ref should be pruned, got:\n{result}"
    );
    // default = [] should survive
    assert!(
        result.contains("default = []"),
        "default feature should be preserved, got:\n{result}"
    );
    // real_blackbox feature key still exists but as empty array
    assert!(
        result.contains("real_blackbox"),
        "feature key should still be present, got:\n{result}"
    );
}

#[test]
fn prune_dangling_feature_refs_removes_optional_dep_feature() {
    // dep?/feature syntax (optional dep)
    let toml = r#"[features]
foo = ["criterion?/real_blackbox", "serde/derive"]
"#;
    let result = prune_dangling_feature_refs(toml, &["criterion".to_string()]);
    assert!(!result.contains("criterion?/real_blackbox"));
    // serde/derive is untouched (serde was not removed)
    assert!(result.contains("serde/derive"));
}

#[test]
fn prune_dangling_feature_refs_removes_exact_dep_name() {
    // Feature enabling a dep by exact name: `feature = ["criterion"]`
    let toml = r#"[features]
benchmarks = ["criterion"]
default = []
"#;
    let result = prune_dangling_feature_refs(toml, &["criterion".to_string()]);
    assert!(!result.contains("\"criterion\""));
    assert!(result.contains("default = []"));
}

#[test]
fn prune_dangling_feature_refs_keeps_non_removed_deps() {
    let toml = r#"[features]
full = ["serde/derive", "criterion/real_blackbox"]
default = []
"#;
    // Only criterion removed — serde/derive must survive
    let result = prune_dangling_feature_refs(toml, &["criterion".to_string()]);
    assert!(!result.contains("criterion/real_blackbox"));
    assert!(result.contains("serde/derive"));
}

#[test]
fn prune_dangling_feature_refs_empty_removed_deps_is_noop() {
    let toml = r#"[features]
full = ["serde/derive"]
"#;
    let result = prune_dangling_feature_refs(toml, &[]);
    assert_eq!(result, toml);
}

#[test]
fn strip_crate_dir_prunes_dangling_features_after_dev_dep_strip() {
    // End-to-end: strip_crate_dir with StripConfig { tests: true, ... }
    // should remove [dev-dependencies] AND prune features that reference them.
    let dir = TempDir::new().unwrap();
    let crate_dir = dir.path().join("mycrate");
    std::fs::create_dir_all(crate_dir.join("src")).unwrap();
    std::fs::write(crate_dir.join("src/lib.rs"), "").unwrap();
    std::fs::write(crate_dir.join(".cargo-checksum.json"), "{\"files\":{}}").unwrap();
    std::fs::write(
        crate_dir.join("Cargo.toml"),
        r#"[package]
name = "mycrate"
version = "0.1.0"

[dependencies]
serde = "1"

[dev-dependencies]
criterion = "0.5"

[features]
real_blackbox = ["criterion/real_blackbox"]
default = []
"#,
    )
    .unwrap();

    strip_crate_dir(&crate_dir, &StripConfig::all(), Verbosity(0)).unwrap();

    let result = std::fs::read_to_string(crate_dir.join("Cargo.toml")).unwrap();
    assert!(
        !result.contains("[dev-dependencies]"),
        "dev-deps should be removed"
    );
    assert!(
        !result.contains("criterion"),
        "criterion ref should be gone"
    );
    assert!(result.contains("serde"), "regular dep preserved");
    assert!(result.contains("default = []"), "default feature preserved");
    assert!(
        result.contains("real_blackbox"),
        "feature key still present but pruned"
    );
}

#[test]
fn strip_keeps_default_feature_ref_when_name_collides_with_dev_dep() {
    // Mirrors the `toml` crate footgun: `serde` is BOTH a [dev-dependencies]
    // entry AND a [features] key. `default = ["std", "serde", ...]` refers
    // to the feature, not the dev-dep, so the prune must not drop it.
    let dir = TempDir::new().unwrap();
    let crate_dir = dir.path().join("toml_like");
    std::fs::create_dir_all(crate_dir.join("src")).unwrap();
    std::fs::write(crate_dir.join("src/lib.rs"), "").unwrap();
    std::fs::write(
        crate_dir.join("Cargo.toml"),
        r#"[package]
name = "toml_like"
version = "0.1.0"

[dependencies]
serde_core = { version = "1", optional = true }

[dev-dependencies]
serde = { version = "1", features = ["derive"] }

[features]
default = ["std", "serde", "parse"]
std = []
parse = []
serde = ["dep:serde_core"]
"#,
    )
    .unwrap();
    std::fs::write(crate_dir.join(".cargo-checksum.json"), "{\"files\":{}}").unwrap();

    strip_crate_dir(&crate_dir, &StripConfig::all(), Verbosity(0)).unwrap();

    let result = std::fs::read_to_string(crate_dir.join("Cargo.toml")).unwrap();
    assert!(!result.contains("[dev-dependencies]"));
    // 'serde' in default list is the FEATURE, not the dev-dep — keep it.
    assert!(
        result.contains("\"serde\""),
        "default's 'serde' (feature ref) must survive: {result}"
    );
    // The serde feature definition itself is intact.
    assert!(result.contains("dep:serde_core"));
}

#[test]
fn strip_keeps_feature_refs_for_deps_in_both_dev_and_regular() {
    // Mirrors the `time` crate: `time-macros` appears in BOTH
    // `[dev-dependencies]` and `[dependencies]` (as optional). Stripping
    // dev-dependencies must not prune `time-macros?/formatting` from
    // `[features]`, because the optional dep entry is still there.
    let dir = TempDir::new().unwrap();
    let crate_dir = dir.path().join("time_like");
    std::fs::create_dir_all(crate_dir.join("src")).unwrap();
    std::fs::write(crate_dir.join("src/lib.rs"), "").unwrap();
    std::fs::write(
        crate_dir.join("Cargo.toml"),
        r#"[package]
name = "time_like"
version = "0.1.0"

[dependencies]
time-macros = { version = "0.2", optional = true }

[dev-dependencies]
time-macros = "0.2"

[features]
formatting = ["time-macros?/formatting"]
parsing    = ["time-macros?/parsing"]
macros     = ["dep:time-macros"]
"#,
    )
    .unwrap();
    std::fs::write(crate_dir.join(".cargo-checksum.json"), "{\"files\":{}}").unwrap();

    strip_crate_dir(&crate_dir, &StripConfig::all(), Verbosity(0)).unwrap();

    let result = std::fs::read_to_string(crate_dir.join("Cargo.toml")).unwrap();
    assert!(!result.contains("[dev-dependencies]"));
    assert!(
        result.contains("time-macros?/formatting"),
        "feature ref to optional dep must survive: {result}"
    );
    assert!(result.contains("time-macros?/parsing"));
    assert!(result.contains("dep:time-macros"));
}

// endregion

// region: --strip-toml-sections (#330)

#[test]
fn toml_only_dirs_to_strip_excludes_source_dirs() {
    let cfg = StripConfig::toml_only();
    let dirs = cfg.dirs_to_strip();
    // Always-safe base dirs survive
    assert!(dirs.contains(&".github"));
    assert!(dirs.contains(&".circleci"));
    assert!(dirs.contains(&"ci"));
    assert!(dirs.contains(&"target"));
    // Source-related dirs are NOT stripped (zerocopy include_str! safety)
    assert!(!dirs.contains(&"tests"));
    assert!(!dirs.contains(&"benches"));
    assert!(!dirs.contains(&"examples"));
}

#[test]
fn toml_only_still_strips_toml_sections() {
    let cfg = StripConfig::toml_only();
    let sections = cfg.toml_sections_to_strip();
    assert!(sections.contains(&"[[test]]"));
    assert!(sections.contains(&"[[bench]]"));
    assert!(sections.contains(&"[[example]]"));
    assert!(sections.contains(&"[[bin]]"));
    assert!(sections.contains(&"[dev-dependencies"));
}

#[test]
fn strip_crate_dir_toml_only_preserves_source_dirs() {
    // Mirrors the zerocopy footgun: bench source files referenced by
    // `include_str!()` from regular lib code must survive stripping.
    let dir = TempDir::new().unwrap();
    let crate_dir = dir.path().join("zerocopy_like");
    std::fs::create_dir_all(crate_dir.join("benches/formats")).unwrap();
    std::fs::create_dir_all(crate_dir.join("tests")).unwrap();
    std::fs::create_dir_all(crate_dir.join("examples")).unwrap();
    std::fs::create_dir_all(crate_dir.join(".github/workflows")).unwrap();
    std::fs::create_dir_all(crate_dir.join("src")).unwrap();
    std::fs::write(crate_dir.join("benches/formats/static_size.rs"), "// bench").unwrap();
    std::fs::write(crate_dir.join("src/lib.rs"), "").unwrap();
    std::fs::write(crate_dir.join(".cargo-checksum.json"), "{\"files\":{}}").unwrap();
    std::fs::write(
        crate_dir.join("Cargo.toml"),
        r#"[package]
name = "zerocopy_like"
version = "0.1.0"

[dependencies]
serde = "1"

[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "static_size"
harness = false

[features]
real_blackbox = ["criterion/real_blackbox"]
default = []
"#,
    )
    .unwrap();

    strip_crate_dir(&crate_dir, &StripConfig::toml_only(), Verbosity(0)).unwrap();

    // Source-related dirs preserved (the whole point of --strip-toml-sections)
    assert!(
        crate_dir.join("benches/formats/static_size.rs").exists(),
        "benches/ must survive — referenced by include_str! in some crates"
    );
    assert!(crate_dir.join("tests").exists(), "tests/ must survive");
    assert!(
        crate_dir.join("examples").exists(),
        "examples/ must survive"
    );
    // Always-safe base dirs still go
    assert!(
        !crate_dir.join(".github").exists(),
        ".github/ should still be stripped"
    );
    // TOML surgery still happens
    let manifest = std::fs::read_to_string(crate_dir.join("Cargo.toml")).unwrap();
    assert!(!manifest.contains("[dev-dependencies]"));
    assert!(!manifest.contains("[[bench]]"));
    assert!(!manifest.contains("criterion"));
    // Dangling [features] ref pruned
    assert!(!manifest.contains("criterion/real_blackbox"));
    // Regular content preserved
    assert!(manifest.contains("serde"));
    assert!(manifest.contains("default = []"));
}

// endregion

// region: include macro scanner tests

#[test]
fn extract_include_arg_literals_basic() {
    let src = r#"
            const A: &str = include_str!("../examples/css/parser.rs");
            const B: &[u8] = include_bytes!( "data/blob.bin" );
            include!("generated.rs");
        "#;
    let paths = extract_include_arg_literals(src);
    assert!(paths.iter().any(|p| p == "../examples/css/parser.rs"));
    assert!(paths.iter().any(|p| p == "data/blob.bin"));
    assert!(paths.iter().any(|p| p == "generated.rs"));
}

#[test]
fn extract_include_arg_literals_through_concat() {
    // zerocopy pattern: include_str!(concat!("../benches/formats/", x, ".rs"))
    let src = r#"
            #[doc = include_str!(concat!("../benches/formats/", $f, ".rs"))]
            #[doc = include_str!(concat!("../benches/", $b))]
        "#;
    let paths = extract_include_arg_literals(src);
    assert!(
        paths.iter().any(|p| p == "../benches/formats/"),
        "expected '../benches/formats/' literal: {:?}",
        paths
    );
    assert!(paths.iter().any(|p| p == "../benches/"));
    assert!(paths.iter().any(|p| p == ".rs"));
}

#[test]
fn referenced_md_files_resolves_relative_to_rs_file() {
    // derive_builder_core shape: src/x.rs does include_str!("doc_tpl/y.md").
    // referenced_md_files must return the canonical path of doc_tpl/y.md
    // (so --blank-md can skip it) but NOT an unrelated docs/README.md (#828).
    let dir = TempDir::new().unwrap();
    let crate_dir = dir.path().join("tplcrate");
    std::fs::create_dir_all(crate_dir.join("src/doc_tpl")).unwrap();
    std::fs::write(
        crate_dir.join("src/lib.rs"),
        r#"const X: &str = include_str!("doc_tpl/builder.md");"#,
    )
    .unwrap();
    let referenced = crate_dir.join("src/doc_tpl/builder.md");
    std::fs::write(&referenced, "template body").unwrap();
    // An unreferenced .md that should NOT be protected.
    std::fs::write(crate_dir.join("README.md"), "# docs").unwrap();

    let protected = referenced_md_files(&crate_dir);
    let canon_referenced = referenced.canonicalize().unwrap();
    assert!(
        protected.contains(&canon_referenced),
        "include_str! .md must be protected, got: {protected:?}"
    );
    let canon_readme = crate_dir.join("README.md").canonicalize().unwrap();
    assert!(
        !protected.contains(&canon_readme),
        "unreferenced README.md must NOT be protected"
    );
}

#[test]
fn referenced_md_files_ignores_non_md_includes() {
    // include_str! of a .rs or .txt file must not appear in the .md set.
    let dir = TempDir::new().unwrap();
    let crate_dir = dir.path().join("c");
    std::fs::create_dir_all(crate_dir.join("src")).unwrap();
    std::fs::write(
        crate_dir.join("src/lib.rs"),
        r#"const A: &str = include_str!("data.txt"); const B: &str = include_str!("gen.rs");"#,
    )
    .unwrap();
    std::fs::write(crate_dir.join("src/data.txt"), "x").unwrap();
    std::fs::write(crate_dir.join("src/gen.rs"), "// gen").unwrap();

    let protected = referenced_md_files(&crate_dir);
    assert!(
        protected.is_empty(),
        "no .md referenced, got: {protected:?}"
    );
}

#[test]
fn strip_all_preserves_dirs_referenced_by_include_str() {
    // Mirrors winnow's footgun: src/lib.rs has include_str!("../examples/...")
    // The strip must not delete examples/ even with --strip-all.
    let dir = TempDir::new().unwrap();
    let crate_dir = dir.path().join("winnow_like");
    std::fs::create_dir_all(crate_dir.join("src")).unwrap();
    std::fs::create_dir_all(crate_dir.join("examples/css")).unwrap();
    std::fs::create_dir_all(crate_dir.join("benches")).unwrap();
    std::fs::create_dir_all(crate_dir.join("tests")).unwrap();
    std::fs::write(crate_dir.join("examples/css/parser.rs"), "// example").unwrap();
    std::fs::write(crate_dir.join("benches/bench.rs"), "// bench").unwrap();
    std::fs::write(crate_dir.join("tests/test.rs"), "// test").unwrap();
    std::fs::write(
        crate_dir.join("src/lib.rs"),
        r#"#![doc = include_str!("../examples/css/parser.rs")]"#,
    )
    .unwrap();
    std::fs::write(
        crate_dir.join("Cargo.toml"),
        "[package]\nname = \"winnow_like\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    std::fs::write(crate_dir.join(".cargo-checksum.json"), "{\"files\":{}}").unwrap();

    strip_crate_dir(&crate_dir, &StripConfig::all(), Verbosity(0)).unwrap();

    // examples/ preserved because include_str! references it
    assert!(
        crate_dir.join("examples/css/parser.rs").exists(),
        "examples/ must survive — referenced by include_str!"
    );
    // tests/ and benches/ have no incoming references → still stripped
    assert!(!crate_dir.join("tests").exists());
    assert!(!crate_dir.join("benches").exists());
}

#[test]
fn strip_all_preserves_dirs_referenced_via_concat() {
    // Mirrors zerocopy's footgun:
    //   include_str!(concat!("../benches/formats/", x, ".rs"))
    // The literal alone doesn't resolve to a file, so substring detection
    // on '/benches/' has to kick in for the dir to survive.
    let dir = TempDir::new().unwrap();
    let crate_dir = dir.path().join("zerocopy_like");
    std::fs::create_dir_all(crate_dir.join("src/util")).unwrap();
    std::fs::create_dir_all(crate_dir.join("benches/formats")).unwrap();
    std::fs::create_dir_all(crate_dir.join("examples")).unwrap();
    std::fs::write(crate_dir.join("benches/formats/coco.rs"), "// bench").unwrap();
    std::fs::write(crate_dir.join("examples/ex.rs"), "// example").unwrap();
    std::fs::write(
        crate_dir.join("src/util/macros.rs"),
        r#"
                #[doc = include_str!(concat!("../benches/formats/", $format, ".rs"))]
                fn _hint() {}
            "#,
    )
    .unwrap();
    std::fs::write(crate_dir.join("src/lib.rs"), "pub mod util;").unwrap();
    std::fs::write(
        crate_dir.join("Cargo.toml"),
        "[package]\nname = \"zerocopy_like\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    std::fs::write(crate_dir.join(".cargo-checksum.json"), "{\"files\":{}}").unwrap();

    strip_crate_dir(&crate_dir, &StripConfig::all(), Verbosity(0)).unwrap();

    // benches/ preserved via concat-substring detection
    assert!(
        crate_dir.join("benches/formats/coco.rs").exists(),
        "benches/ must survive — referenced via concat!() in include_str!"
    );
    // examples/ has no reference → still stripped
    assert!(!crate_dir.join("examples").exists());
}

#[test]
fn extract_path_attr_literals_basic() {
    let src = r#"
            #[path = "../tests/common/shared.rs"]
            mod shared;
            #[path="imp/unix.rs"] mod imp;
            #[path_like = "../examples/not_a_path_attr.rs"]
            #[cfg(test)]
            mod t;
        "#;
    let paths = extract_path_attr_literals(src);
    assert_eq!(
        paths,
        vec![
            "../tests/common/shared.rs".to_string(),
            "imp/unix.rs".to_string()
        ]
    );
}

#[test]
fn strip_all_preserves_dirs_referenced_by_path_attr() {
    // A library module whose file lives under benches/ via #[path]: the
    // module is compiled into the lib, so benches/ has to survive.
    let dir = TempDir::new().unwrap();
    let crate_dir = dir.path().join("path_attr_like");
    std::fs::create_dir_all(crate_dir.join("src")).unwrap();
    std::fs::create_dir_all(crate_dir.join("benches")).unwrap();
    std::fs::create_dir_all(crate_dir.join("tests")).unwrap();
    std::fs::create_dir_all(crate_dir.join("examples")).unwrap();
    std::fs::write(crate_dir.join("benches/shared.rs"), "pub fn f() {}").unwrap();
    std::fs::write(crate_dir.join("tests/test.rs"), "// test").unwrap();
    std::fs::write(crate_dir.join("examples/ex.rs"), "// example").unwrap();
    std::fs::write(
        crate_dir.join("src/lib.rs"),
        "#[path = \"../benches/shared.rs\"]\npub mod shared;\n",
    )
    .unwrap();
    std::fs::write(
        crate_dir.join("Cargo.toml"),
        "[package]\nname = \"path_attr_like\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    std::fs::write(crate_dir.join(".cargo-checksum.json"), "{\"files\":{}}").unwrap();

    strip_crate_dir(&crate_dir, &StripConfig::all(), Verbosity(0)).unwrap();

    assert!(
        crate_dir.join("benches/shared.rs").exists(),
        "benches/ must survive — a lib module lives there via #[path]"
    );
    assert!(!crate_dir.join("tests").exists());
    assert!(!crate_dir.join("examples").exists());
}

// endregion

#[test]
fn strip_crate_dir_removes_directories() {
    let dir = TempDir::new().unwrap();
    let crate_dir = dir.path().join("mycrate");
    std::fs::create_dir_all(crate_dir.join("tests")).unwrap();
    std::fs::create_dir_all(crate_dir.join("benches")).unwrap();
    std::fs::create_dir_all(crate_dir.join("examples")).unwrap();
    std::fs::create_dir_all(crate_dir.join(".github")).unwrap();
    std::fs::create_dir_all(crate_dir.join("src")).unwrap();
    std::fs::write(crate_dir.join("tests/test.rs"), "").unwrap();
    std::fs::write(crate_dir.join("src/lib.rs"), "").unwrap();
    std::fs::write(
        crate_dir.join("Cargo.toml"),
        "[package]\nname = \"mycrate\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    std::fs::write(crate_dir.join(".cargo-checksum.json"), "{\"files\":{}}").unwrap();

    strip_crate_dir(&crate_dir, &StripConfig::all(), Verbosity(0)).unwrap();

    assert!(!crate_dir.join("tests").exists());
    assert!(!crate_dir.join("benches").exists());
    assert!(!crate_dir.join("examples").exists());
    assert!(!crate_dir.join(".github").exists());
    assert!(crate_dir.join("src").exists());
    assert!(crate_dir.join(".cargo-checksum.json").exists());
}
