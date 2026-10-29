use super::*;
use crate::Verbosity;

// region: collect_git_urls (#256)
//
// These cover the problematic shapes called out in the review: the old
// line-regex in generate_cargo_config missed `git=\"...\"` (no spaces),
// non-https schemes, and inline tables with trailing fields. Each test
// asserts the structural walker handles one of those shapes correctly.

#[test]
fn git_inline_table_with_spaces() {
    let toml = r#"[dependencies]
foo = { git = "https://github.com/bar/foo", rev = "abc123" }
"#;
    let urls = collect_git_urls(toml).unwrap();
    assert_eq!(urls.len(), 1);
    assert!(urls.contains("https://github.com/bar/foo"));
}

#[test]
fn git_inline_table_no_spaces() {
    // The old line-regex required `git = \"` literally; this shape
    // broke it. toml_edit accepts either.
    let toml = r#"[dependencies]
foo={git="https://github.com/bar/foo"}
"#;
    let urls = collect_git_urls(toml).unwrap();
    assert!(urls.contains("https://github.com/bar/foo"));
}

#[test]
fn git_non_https_schemes_preserved() {
    let toml = r#"[dependencies]
a = { git = "ssh://git@github.com/bar/a" }
b = { git = "http://example.com/bar/b" }
c = { git = "git+https://gitlab.com/bar/c" }
"#;
    let urls = collect_git_urls(toml).unwrap();
    assert!(urls.contains("ssh://git@github.com/bar/a"));
    assert!(urls.contains("http://example.com/bar/b"));
    assert!(urls.contains("git+https://gitlab.com/bar/c"));
}

#[test]
fn git_table_form_dependency() {
    let toml = r#"[dependencies.foo]
git = "https://github.com/bar/foo"
branch = "main"
"#;
    let urls = collect_git_urls(toml).unwrap();
    assert!(urls.contains("https://github.com/bar/foo"));
}

#[test]
fn git_target_gated_dependency() {
    let toml = r#"[target.'cfg(windows)'.dependencies]
foo = { git = "https://github.com/bar/foo-win" }

[target.'cfg(unix)'.build-dependencies]
bar = { git = "https://github.com/baz/bar-unix" }
"#;
    let urls = collect_git_urls(toml).unwrap();
    assert!(urls.contains("https://github.com/bar/foo-win"));
    assert!(urls.contains("https://github.com/baz/bar-unix"));
}

#[test]
fn git_across_multiple_dep_tables() {
    let toml = r#"[dependencies]
a = { git = "https://github.com/a/a" }

[dev-dependencies]
b = { git = "https://github.com/b/b" }

[build-dependencies]
c = { git = "https://github.com/c/c" }
"#;
    let urls = collect_git_urls(toml).unwrap();
    assert_eq!(urls.len(), 3);
}

#[test]
fn no_git_deps_returns_empty() {
    let toml = r#"[dependencies]
serde = "1"
anyhow = { version = "1", default-features = false }
"#;
    let urls = collect_git_urls(toml).unwrap();
    assert!(urls.is_empty());
}

#[test]
fn duplicate_git_urls_deduplicated() {
    // Two deps from the same git URL collapse to one entry.
    let toml = r#"[dependencies]
foo = { git = "https://github.com/x/repo" }
bar = { git = "https://github.com/x/repo" }
"#;
    let urls = collect_git_urls(toml).unwrap();
    assert_eq!(urls.len(), 1);
}

#[test]
fn malformed_toml_returns_empty_safely() {
    // Parse error shouldn't panic — caller's other paths will surface
    // the real error.
    let toml = r#"[dependencies
this-is = "broken"
"#;
    let urls = collect_git_urls(toml).unwrap();
    assert!(urls.is_empty());
}

// endregion

// region: --strict-freeze (#252)
//
// freeze_manifest rewrites local_pkgs deps to vendor paths but leaves
// external `git = "..."` deps alone. --strict-freeze turns any residual
// git dep into an error instead of a silent trust-the-config-replacement.

#[test]
fn collect_remaining_git_deps_finds_inline_table() {
    let toml = r#"[dependencies]
local = { path = "../local" }
external = { git = "https://example.com/ext" }
"#;
    let doc: toml_edit::DocumentMut = toml.parse().unwrap();
    let deps = collect_remaining_git_deps(&doc);
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].0, "external");
    assert_eq!(deps[0].1, "https://example.com/ext");
}

#[test]
fn collect_remaining_git_deps_finds_target_gated() {
    let toml = r#"[target.'cfg(unix)'.dependencies]
foo = { git = "https://example.com/unix-foo" }

[target.'cfg(windows)'.build-dependencies]
bar = { git = "https://example.com/win-bar" }
"#;
    let doc: toml_edit::DocumentMut = toml.parse().unwrap();
    let deps = collect_remaining_git_deps(&doc);
    assert_eq!(deps.len(), 2);
    assert!(deps.iter().any(|(n, _)| n == "foo"));
    assert!(deps.iter().any(|(n, _)| n == "bar"));
}

#[test]
fn collect_remaining_git_deps_ignores_path_and_version() {
    let toml = r#"[dependencies]
from_path = { path = "../x" }
from_crates_io = "1.0"
from_crates_io_inline = { version = "1.0", default-features = false }
"#;
    let doc: toml_edit::DocumentMut = toml.parse().unwrap();
    assert!(collect_remaining_git_deps(&doc).is_empty());
}

#[test]
fn collect_remaining_git_deps_handles_table_form() {
    let toml = r#"[dependencies.foo]
git = "https://example.com/foo"
rev = "abc"
"#;
    let doc: toml_edit::DocumentMut = toml.parse().unwrap();
    let deps = collect_remaining_git_deps(&doc);
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].0, "foo");
}

#[test]
fn freeze_manifest_strict_errors_on_external_git() {
    // Full-flow test: build a fixture with an external git dep,
    // freeze with strict=true, assert error.
    let dir = tempfile::tempdir().unwrap();
    let manifest = dir.path().join("Cargo.toml");
    std::fs::write(
        &manifest,
        r#"[package]
name = "x"
version = "0.1.0"
edition = "2021"

[dependencies]
external = { git = "https://example.com/ext" }
"#,
    )
    .unwrap();
    let vendor = dir.path().join("vendor");
    std::fs::create_dir_all(&vendor).unwrap();

    let err = freeze_manifest(
        &manifest,
        &vendor,
        &[],
        false,
        /* strict */ true,
        Verbosity(0),
    )
    .unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("--strict-freeze") && msg.contains("external"),
        "expected strict-freeze error naming the dep, got:\n{msg}"
    );
}

#[test]
fn freeze_manifest_writes_prefreeze_sidecar_once() {
    let dir = tempfile::tempdir().unwrap();
    let vendor = dir.path().join("vendor");
    let binding = vendor.join("binding");
    std::fs::create_dir_all(&binding).unwrap();
    std::fs::create_dir_all(vendor.join("core")).unwrap();
    let manifest = binding.join("Cargo.toml");
    // Deliberately odd formatting: the snapshot must be byte-exact.
    let original = "[package]\nname = \"binding\"\nversion = \"0.1.0\"\r\n\n[dependencies]\ncore   = { path = \"../../../core\" }  # sibling\n";
    std::fs::write(&manifest, original).unwrap();
    let packages = vec![LocalPackage {
        name: "core".into(),
        version: "0.1.0".into(),
        path: dir.path().join("core"),
        manifest_path: dir.path().join("core/Cargo.toml"),
    }];
    let sidecar = prefreeze_sidecar_path(&manifest);
    assert_eq!(sidecar, binding.join(".Cargo.toml.prefreeze"));
    assert!(!sidecar.exists());

    freeze_manifest(&manifest, &vendor, &packages, false, false, Verbosity(0)).unwrap();
    assert_eq!(std::fs::read_to_string(&sidecar).unwrap(), original);
    let frozen = std::fs::read_to_string(&manifest).unwrap();
    assert_ne!(frozen, original);
    assert!(frozen.contains("../core"));

    // Freezing the frozen manifest again keeps the first snapshot.
    freeze_manifest(&manifest, &vendor, &packages, false, false, Verbosity(0)).unwrap();
    assert_eq!(std::fs::read_to_string(&sidecar).unwrap(), original);
    assert_eq!(std::fs::read_to_string(&manifest).unwrap(), frozen);
}

#[test]
fn freeze_manifest_without_changes_writes_no_sidecar() {
    let dir = tempfile::tempdir().unwrap();
    let manifest = dir.path().join("Cargo.toml");
    let original =
        "[package]\nname = \"x\"\nversion = \"0.1.0\"\n\n[dependencies]\ncfg-if = \"1\"\n";
    std::fs::write(&manifest, original).unwrap();
    let vendor = dir.path().join("vendor");
    std::fs::create_dir_all(&vendor).unwrap();

    freeze_manifest(&manifest, &vendor, &[], false, false, Verbosity(0)).unwrap();
    assert_eq!(std::fs::read_to_string(&manifest).unwrap(), original);
    assert!(!prefreeze_sidecar_path(&manifest).exists());
}

#[test]
fn freeze_and_vendor_rewrites_preserve_renamed_path_dependencies() {
    let dir = tempfile::tempdir().unwrap();
    let vendor = dir.path().join("vendor");
    let binding = vendor.join("binding");
    std::fs::create_dir_all(&binding).unwrap();
    std::fs::create_dir_all(vendor.join("core")).unwrap();
    let manifest = binding.join("Cargo.toml");
    std::fs::write(
        &manifest,
        r#"[package]
name = "binding"
version = "0.1.0"
[dependencies]
core_library = { package = "core", path = "../../../core", optional = true }
[build-dependencies.build_core]
package = "core"
path = "../../../core"
"#,
    )
    .unwrap();
    let packages = vec![LocalPackage {
        name: "core".into(),
        version: "0.1.0".into(),
        path: dir.path().join("core"),
        manifest_path: dir.path().join("core/Cargo.toml"),
    }];
    freeze_manifest(&manifest, &vendor, &packages, false, false, Verbosity(0)).unwrap();
    let frozen: toml_edit::DocumentMut =
        std::fs::read_to_string(&manifest).unwrap().parse().unwrap();
    for (section, alias) in [
        ("dependencies", "core_library"),
        ("build-dependencies", "build_core"),
    ] {
        assert_eq!(frozen[section][alias]["package"].as_str(), Some("core"));
        assert!(frozen[section][alias].get("path").is_none());
        assert_eq!(frozen[section][alias]["version"].as_str(), Some("*"));
    }
    assert_eq!(
        frozen["dependencies"]["core_library"]["optional"].as_bool(),
        Some(true)
    );
    assert_eq!(
        frozen["patch"]["crates-io"]["core"]["path"].as_str(),
        Some("../core")
    );
    let mut stale = frozen;
    stale["dependencies"]["core_library"]["path"] = toml_edit::value("/old/core");
    std::fs::write(&manifest, stale.to_string()).unwrap();
    rewrite_local_path_deps(&vendor, &packages, Verbosity(0)).unwrap();
    let repaired: toml_edit::DocumentMut =
        std::fs::read_to_string(&manifest).unwrap().parse().unwrap();
    assert_eq!(
        repaired["dependencies"]["core_library"]["path"].as_str(),
        Some("../core")
    );
    assert_eq!(
        repaired["dependencies"]["core_library"]["package"].as_str(),
        Some("core")
    );
}

#[test]
fn freeze_manifest_non_strict_succeeds_on_external_git() {
    // Same fixture but strict=false: freeze proceeds, leaving the
    // external git dep in place. The verified property here is that
    // the error path doesn't fire.
    let dir = tempfile::tempdir().unwrap();
    let manifest = dir.path().join("Cargo.toml");
    std::fs::write(
        &manifest,
        r#"[package]
name = "x"
version = "0.1.0"
edition = "2021"

[dependencies]
external = { git = "https://example.com/ext" }
"#,
    )
    .unwrap();
    let vendor = dir.path().join("vendor");
    std::fs::create_dir_all(&vendor).unwrap();

    freeze_manifest(
        &manifest,
        &vendor,
        &[],
        false,
        /* strict */ false,
        Verbosity(0),
    )
    .unwrap();
}

// endregion

// region: blank_md (#828)

/// `--blank-md` must blank documentation `.md` (crate-root README, etc.)
/// but leave `.md` files that Rust source pulls in via `include_str!`
/// intact — blanking those produces a `format!("", …)` compile error and
/// breaks every vendored build whose deps include such a crate (#828).
///
/// The intact file's checksum must also stay correct: because we don't
/// touch it, the post-blank `recompute_checksums` (hashing actual disk
/// contents) yields its unchanged SHA-256 automatically.
#[test]
fn blank_md_skips_include_str_referenced_files() {
    use sha2::Digest;

    let dir = tempfile::tempdir().unwrap();
    let vendor = dir.path().join("vendor");
    let crate_dir = vendor.join("tplcrate-0.1.0");
    std::fs::create_dir_all(crate_dir.join("src/doc_tpl")).unwrap();

    // Rust source that pulls a .md template into source via include_str!
    // inside a format! call — the exact derive_builder_core shape.
    std::fs::write(
        crate_dir.join("src/lib.rs"),
        r#"pub fn doc(name: &str) -> String {
    format!(include_str!("doc_tpl/builder_struct.md"), struct_name = name)
}
"#,
    )
    .unwrap();
    let tpl_body = "Builder for {struct_name}.\n";
    std::fs::write(crate_dir.join("src/doc_tpl/builder_struct.md"), tpl_body).unwrap();

    // A crate-root README that SHOULD be blanked (pure docs).
    std::fs::write(crate_dir.join("README.md"), "# tplcrate\n\nDocs.\n").unwrap();

    // Minimal .cargo-checksum.json with a package hash, so the recompute
    // path is exercised end-to-end.
    let pkg = "deadbeef1234deadbeef1234deadbeef1234deadbeef1234deadbeef1234dead";
    std::fs::write(
        crate_dir.join(".cargo-checksum.json"),
        serde_json::json!({ "package": pkg, "files": {} }).to_string(),
    )
    .unwrap();

    // Run the blank step (no tar).
    let skipped = blank_md_files(&vendor).unwrap();
    assert_eq!(skipped, 1, "exactly one source-referenced .md kept intact");

    // The include_str! template must be byte-for-byte intact.
    let tpl_after =
        std::fs::read_to_string(crate_dir.join("src/doc_tpl/builder_struct.md")).unwrap();
    assert_eq!(tpl_after, tpl_body, "include_str! .md must not be blanked");

    // The crate-root README must be blanked.
    let readme_after = std::fs::read_to_string(crate_dir.join("README.md")).unwrap();
    assert_eq!(readme_after, "", "crate-root README must be blanked");

    // Now recompute checksums (the production sequence) and confirm the
    // intact template's recorded SHA-256 matches its real content, while
    // the blanked README's recorded SHA-256 matches the empty string.
    crate::checksum::recompute_checksums(&vendor).unwrap();
    let raw = std::fs::read_to_string(crate_dir.join(".cargo-checksum.json")).unwrap();
    let cksum: serde_json::Value = serde_json::from_str(&raw).unwrap();
    let files = cksum["files"].as_object().unwrap();

    let tpl_hash = format!("{:x}", sha2::Sha256::digest(tpl_body.as_bytes()));
    assert_eq!(
        files["src/doc_tpl/builder_struct.md"].as_str().unwrap(),
        tpl_hash,
        "checksum of intact template must match its real (unchanged) content"
    );

    let empty_hash = format!("{:x}", sha2::Sha256::digest(b""));
    assert_eq!(
        files["README.md"].as_str().unwrap(),
        empty_hash,
        "checksum of blanked README must match the empty string"
    );

    // package field preserved.
    assert_eq!(cksum["package"].as_str().unwrap(), pkg);
}

// endregion

#[test]
fn strip_vendor_path_deps_removes_relative_paths() {
    let dir = tempfile::tempdir().unwrap();
    let vendor = dir.path().join("vendor");
    std::fs::create_dir_all(vendor.join("crate-a")).unwrap();
    std::fs::create_dir_all(vendor.join("crate-b")).unwrap();

    // crate-a has a relative path dep to crate-b
    std::fs::write(
        vendor.join("crate-a/Cargo.toml"),
        r#"[package]
name = "crate-a"
version = "0.1.0"

[dependencies.crate-b]
version = "*"
path = "../crate-b"
"#,
    )
    .unwrap();

    // crate-b has no path deps
    std::fs::write(
        vendor.join("crate-b/Cargo.toml"),
        r#"[package]
name = "crate-b"
version = "0.1.0"
"#,
    )
    .unwrap();

    strip_vendor_path_deps(&vendor, Verbosity(0)).unwrap();

    let result = std::fs::read_to_string(vendor.join("crate-a/Cargo.toml")).unwrap();
    assert!(result.contains("crate-b"));
    assert!(result.contains("version"));
    assert!(!result.contains("path"));
}

#[test]
fn freeze_manifest_patch_crates_io_is_alphabetical() {
    let dir = tempfile::tempdir().unwrap();
    let vendor = dir.path().join("vendor");
    for name in [
        "miniextendr-api",
        "miniextendr-macros",
        "miniextendr-lint",
        "miniextendr-macros-core",
    ] {
        std::fs::create_dir_all(vendor.join(name)).unwrap();
    }
    let manifest = dir.path().join("Cargo.toml");
    // Start with a [patch.crates-io] section whose keys are in non-sorted order
    // — this exercises the old HashSet nondeterminism bug (#205).
    std::fs::write(
        &manifest,
        r#"[package]
name = "example"
version = "0.1.0"

[dependencies]

[patch.crates-io]
miniextendr-api = { path = "/tmp/a" }
miniextendr-macros = { path = "/tmp/m" }
miniextendr-lint = { path = "/tmp/l" }
miniextendr-macros-core = { path = "/tmp/mc" }
"#,
    )
    .unwrap();

    freeze_manifest(&manifest, &vendor, &[], false, false, Verbosity(0)).unwrap();
    let result = std::fs::read_to_string(&manifest).unwrap();
    assert!(!result.lines().any(|line| line.trim() == "[patch]"));
    assert!(result.contains("[patch.crates-io]"));
    let api = result.find("miniextendr-api =").unwrap();
    let lint = result.find("miniextendr-lint =").unwrap();
    let macros = result.find("miniextendr-macros =").unwrap();
    let macros_core = result.find("miniextendr-macros-core =").unwrap();
    assert!(
        api < lint && lint < macros && macros < macros_core,
        "patch.crates-io entries not alphabetical: {}",
        result
    );
}

#[test]
fn freeze_manifest_leaves_git_patched_local_deps_as_git() {
    // A git dep resolved to a local crate via a `.cargo/config.toml`
    // [patch] (the monorepo dev override) lands in local_pkgs, but is
    // declared `git =` in the manifest. Freeze must NOT rewrite it to a
    // vendor/ path — it stays an external git dep, resolved offline via
    // source replacement. Only the genuinely path-declared sibling is
    // frozen. Regression guard for the path-dep-sibling support: a blunt
    // rewrite-all-local-pkgs freeze would corrupt framework git deps.
    let dir = tempfile::tempdir().unwrap();
    let vendor = dir.path().join("vendor");
    std::fs::create_dir_all(vendor.join("framework-crate")).unwrap();
    std::fs::create_dir_all(vendor.join("core-sibling")).unwrap();
    let manifest = dir.path().join("Cargo.toml");
    std::fs::write(
        &manifest,
        r#"[package]
name = "rpkg"
version = "0.1.0"
edition = "2021"

[dependencies]
framework-crate = { git = "https://github.com/example/framework" }
core-sibling = { path = "../../../core-sibling" }
"#,
    )
    .unwrap();

    // Both crates are discovered as local: framework-crate via a patch
    // entry, core-sibling via the path dep. freeze sees both in local_pkgs.
    let local_pkgs = vec![
        LocalPackage {
            name: "framework-crate".into(),
            version: "0.1.0".into(),
            path: dir.path().join("framework-crate"),
            manifest_path: dir.path().join("framework-crate/Cargo.toml"),
        },
        LocalPackage {
            name: "core-sibling".into(),
            version: "0.1.0".into(),
            path: dir.path().join("core-sibling"),
            manifest_path: dir.path().join("core-sibling/Cargo.toml"),
        },
    ];

    freeze_manifest(&manifest, &vendor, &local_pkgs, false, false, Verbosity(0)).unwrap();
    let result = std::fs::read_to_string(&manifest).unwrap();

    // git dep stays git, NOT rewritten to a vendor/ path.
    assert!(
        result.contains(r#"git = "https://github.com/example/framework""#),
        "git-declared dep lost its git source:\n{result}"
    );
    assert!(
        !result.contains("vendor/framework-crate"),
        "git-declared dep was wrongly frozen to a vendor/ path:\n{result}"
    );
    // path dep IS frozen to vendor/.
    assert!(
        result.contains("path = \"vendor/core-sibling\""),
        "path-declared sibling was not frozen:\n{result}"
    );

    // [patch.crates-io] covers the frozen path dep only, never the git dep.
    if let Some(idx) = result.find("[patch.crates-io]") {
        let patch_section = &result[idx..];
        assert!(
            !patch_section.contains("framework-crate"),
            "git dep leaked into [patch.crates-io]:\n{result}"
        );
        assert!(
            patch_section.contains("core-sibling"),
            "frozen path dep missing from [patch.crates-io]:\n{result}"
        );
    }
}

#[test]
fn strip_vendor_path_deps_keeps_internal_paths() {
    let dir = tempfile::tempdir().unwrap();
    let vendor = dir.path().join("vendor");
    std::fs::create_dir_all(vendor.join("mycrate")).unwrap();

    // path = "src/lib.rs" should NOT be stripped (it's internal, not ../...)
    std::fs::write(
        vendor.join("mycrate/Cargo.toml"),
        r#"[package]
name = "mycrate"
version = "0.1.0"

[lib]
path = "src/lib.rs"

[dependencies.sibling]
version = "*"
path = "../sibling"
"#,
    )
    .unwrap();

    strip_vendor_path_deps(&vendor, Verbosity(0)).unwrap();

    let result = std::fs::read_to_string(vendor.join("mycrate/Cargo.toml")).unwrap();
    // [lib] path is preserved (not a relative ../ path, and not in dependencies)
    assert!(result.contains("src/lib.rs"));
    // dependency path is stripped
    assert!(!result.contains("../sibling"));
}

// region: stamp_framework_git_sources (#883)

fn url_map(pairs: &[(&str, &str)]) -> std::collections::BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

#[test]
fn stamp_adds_git_source_after_version() {
    // A framework crate resolved as a local (no-source) entry gains
    // `source = "git+<url>#<rev>"` placed immediately after `version`.
    let dir = tempfile::tempdir().unwrap();
    let lock = dir.path().join("Cargo.lock");
    std::fs::write(
        &lock,
        r#"version = 4

[[package]]
name = "miniextendr-api"
version = "0.1.0"
dependencies = [
 "serde",
]

[[package]]
name = "serde"
version = "1.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
"#,
    )
    .unwrap();

    let map = url_map(&[("miniextendr-api", "https://github.com/A2-ai/miniextendr")]);
    let n = stamp_framework_git_sources(
        &lock,
        &map,
        "abc1230000000000000000000000000000000000",
        Verbosity(0),
    )
    .unwrap();
    assert_eq!(n, 1);

    let out = std::fs::read_to_string(&lock).unwrap();
    // source line present with git+url#rev
    assert!(out.contains(
        "source = \"git+https://github.com/A2-ai/miniextendr#abc1230000000000000000000000000000000000\""
    ));
    // placed right after version: the `-A 3` lock-shape grep must see it.
    let api_idx = out.find("name = \"miniextendr-api\"").unwrap();
    let after = &out[api_idx..];
    let version_line = after.find("version = ").unwrap();
    let source_line = after.find("source = ").unwrap();
    assert!(
        source_line > version_line && source_line - version_line < 30,
        "source must immediately follow version; got:\n{after}"
    );
    // The registry crate (not in the map) is untouched.
    assert!(out.contains("source = \"registry+https://github.com/rust-lang/crates.io-index\""));
}

#[test]
fn stamp_overwrites_existing_path_source() {
    // A drifted lock with `source = "path+..."` for a framework crate is
    // rewritten to the canonical git source (no duplicate source key).
    let dir = tempfile::tempdir().unwrap();
    let lock = dir.path().join("Cargo.lock");
    std::fs::write(
        &lock,
        r#"version = 4

[[package]]
name = "miniextendr-macros"
version = "0.1.0"
source = "path+file:///home/dev/miniextendr/miniextendr-macros"
dependencies = []
"#,
    )
    .unwrap();

    let map = url_map(&[("miniextendr-macros", "https://github.com/A2-ai/miniextendr")]);
    let n = stamp_framework_git_sources(&lock, &map, &"f".repeat(40), Verbosity(0)).unwrap();
    assert_eq!(n, 1);

    let out = std::fs::read_to_string(&lock).unwrap();
    assert!(
        !out.contains("path+file://"),
        "path source must be gone:\n{out}"
    );
    assert_eq!(
        out.matches("source = ").count(),
        1,
        "exactly one source key (no duplicate):\n{out}"
    );
    assert!(out.contains(&format!(
        "source = \"git+https://github.com/A2-ai/miniextendr#{}\"",
        "f".repeat(40)
    )));
}

#[test]
fn stamp_noop_when_map_empty_or_no_match() {
    let dir = tempfile::tempdir().unwrap();
    let lock = dir.path().join("Cargo.lock");
    let body = r#"version = 4

[[package]]
name = "serde"
version = "1.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
"#;
    std::fs::write(&lock, body).unwrap();

    // Empty map → 0 stamped, file unchanged.
    assert_eq!(
        stamp_framework_git_sources(&lock, &url_map(&[]), &"0".repeat(40), Verbosity(0)).unwrap(),
        0
    );
    // Map with a name not in the lock → 0 stamped.
    let map = url_map(&[("miniextendr-api", "https://github.com/A2-ai/miniextendr")]);
    assert_eq!(
        stamp_framework_git_sources(&lock, &map, &"0".repeat(40), Verbosity(0)).unwrap(),
        0
    );
    assert_eq!(std::fs::read_to_string(&lock).unwrap(), body);
}

// endregion
