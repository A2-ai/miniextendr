//! Integration tests for auto-reading `[patch."<url>"]` from `.cargo/config.toml`.
//!
//! These tests exercise the feature described in plans/lockfile-mode-unification.md
//! item 1: cargo-revendor reads `[patch."<url>"] crate = { path = "..." }` entries
//! from `.cargo/config.toml` and vendors the patched local source *without* the
//! caller passing `--source-root`.
//!
//! All tests are gated behind `#[ignore]` because they invoke `cargo vendor`
//! (which may touch the registry for a full dep-graph resolve even with pure-git
//! or pure-path deps). Run with `cargo test -p cargo-revendor -- --ignored`.

mod common;

use common::{assert_vendor_has, create_local_git_crate, git_init, revendor_cmd, vendor_dir_for};
use std::path::Path;

/// Build a small workspace with one member and write a `.cargo/config.toml`
/// that patches the given git URL to point at the workspace member.
///
/// Layout:
///   root/
///     Cargo.toml          ← workspace root: resolver = "2", member = ["lib"]
///     lib/
///       Cargo.toml        ← the local crate being patched in
///       src/lib.rs        ← contains MARKER so we can assert the version
///     pkg/
///       Cargo.toml        ← the crate that depends on the git source
///       lib.rs
///     .cargo/config.toml  ← [patch."git_url"] lib = { path = "lib" }
fn build_patch_workspace(
    git_url: &str,
    crate_name: &str,
    crate_version: &str,
    marker: &str,
) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::TempDir::new().unwrap();
    let root = dir.path().to_path_buf();

    // Workspace root
    std::fs::write(
        root.join("Cargo.toml"),
        r#"[workspace]
members = ["lib", "pkg"]
resolver = "2"
"#,
    )
    .unwrap();

    // Local lib crate (the one that overrides the git source)
    std::fs::create_dir_all(root.join("lib/src")).unwrap();
    std::fs::write(
        root.join("lib/Cargo.toml"),
        format!(
            r#"[package]
name = "{crate_name}"
version = "{crate_version}"
edition = "2021"
publish = false

[lib]
path = "src/lib.rs"
"#
        ),
    )
    .unwrap();
    std::fs::write(
        root.join("lib/src/lib.rs"),
        format!("// PATCH_MARKER: {marker}\npub fn patched() -> &'static str {{ \"{marker}\" }}\n"),
    )
    .unwrap();

    // The rpkg-style crate that depends on the git source
    std::fs::create_dir_all(root.join("pkg")).unwrap();
    std::fs::write(
        root.join("pkg/Cargo.toml"),
        format!(
            r#"[package]
name = "test-pkg"
version = "0.1.0"
edition = "2021"
publish = false

[workspace]

[lib]
path = "lib.rs"

[dependencies]
{crate_name} = {{ git = "{git_url}" }}
"#
        ),
    )
    .unwrap();
    std::fs::write(
        root.join("pkg/lib.rs"),
        format!(
            "pub use {lib}::patched;\n",
            lib = crate_name.replace('-', "_")
        ),
    )
    .unwrap();

    // .cargo/config.toml at workspace root with the [patch] override
    std::fs::create_dir_all(root.join(".cargo")).unwrap();
    std::fs::write(
        root.join(".cargo/config.toml"),
        format!(
            "[patch.\"{}\"]\n{crate_name} = {{ path = \"lib\" }}\n",
            git_url
        ),
    )
    .unwrap();

    git_init(&root);

    let manifest = root.join("pkg/Cargo.toml");
    (dir, manifest)
}

/// **P1** — basic: `.cargo/config.toml` with `[patch."file://..."]` and a
/// `path = "..."` entry. cargo-revendor should pick up the local override
/// *without* `--source-root`, and the vendored copy should contain the
/// patched source.
#[test]
#[ignore] // invokes cargo vendor
fn patch_from_config_basic() {
    // Bare git repo as the "upstream" we are going to override.
    let git_upstream = create_local_git_crate(
        "upstream-lib",
        r#"[package]
name = "upstream-lib"
version = "0.1.0"
edition = "2021"
publish = false
"#,
        "pub fn upstream() -> u32 { 1 }\npub fn patched() -> &'static str { \"GIT\" }\n",
    );

    let (_work, manifest) = build_patch_workspace(
        &git_upstream.url(),
        "upstream-lib",
        "0.1.0",
        "LOCAL_EDIT_123",
    );
    let vendor = manifest.parent().unwrap().join("vendor");

    revendor_cmd()
        .arg("revendor")
        .arg("--manifest-path")
        .arg(&manifest)
        .arg("--output")
        .arg(&vendor)
        // No --source-root — auto-detection from .cargo/config.toml must do it.
        .assert()
        .success();

    assert_vendor_has(&vendor, "upstream-lib");

    // The vendored copy must be the patched LOCAL version, not the git snapshot.
    let crate_dir = vendor_dir_for(&vendor, "upstream-lib", None);
    let lib_rs = std::fs::read_to_string(crate_dir.join("src/lib.rs"))
        .expect("src/lib.rs not found in vendored upstream-lib");
    assert!(
        lib_rs.contains("LOCAL_EDIT_123"),
        "vendored upstream-lib should contain the local override, got:\n{lib_rs}"
    );
}

/// **P2** — config.toml placed one level above the manifest dir (ancestor
/// directory). cargo-revendor should walk up and find it.
#[test]
#[ignore] // invokes cargo vendor
fn patch_from_config_in_ancestor_dir() {
    let git_upstream = create_local_git_crate(
        "ancestor-lib",
        r#"[package]
name = "ancestor-lib"
version = "0.2.0"
edition = "2021"
publish = false
"#,
        "pub fn ancestor() {}\npub fn patched() -> &'static str { \"GIT\" }\n",
    );

    let (_work, manifest) = build_patch_workspace(
        &git_upstream.url(),
        "ancestor-lib",
        "0.2.0",
        "ANCESTOR_OVERRIDE",
    );

    // The .cargo/config.toml is already written at the workspace root level
    // (one level above pkg/Cargo.toml), so this test exercises the ancestor
    // walk without any extra setup.
    let vendor = manifest.parent().unwrap().join("vendor");

    revendor_cmd()
        .arg("revendor")
        .arg("--manifest-path")
        .arg(&manifest)
        .arg("--output")
        .arg(&vendor)
        .assert()
        .success();

    assert_vendor_has(&vendor, "ancestor-lib");
    let crate_dir = vendor_dir_for(&vendor, "ancestor-lib", None);
    let lib_rs = std::fs::read_to_string(crate_dir.join("src/lib.rs"))
        .expect("src/lib.rs not found in vendored ancestor-lib");
    assert!(
        lib_rs.contains("ANCESTOR_OVERRIDE"),
        "vendored ancestor-lib should contain the ancestor-level override, got:\n{lib_rs}"
    );
}

/// **P3** — explicit `--source-root` wins over a `.cargo/config.toml` entry
/// for the same crate. Verifies the "explicit beats inferred" rule.
#[test]
#[ignore] // invokes cargo vendor
fn source_root_wins_over_patch_config() {
    let git_upstream = create_local_git_crate(
        "conflict-lib",
        r#"[package]
name = "conflict-lib"
version = "0.3.0"
edition = "2021"
publish = false
"#,
        "pub fn conflict() {}\npub fn patched() -> &'static str { \"GIT\" }\n",
    );

    let git_url = git_upstream.url();

    // Two competing workspaces:
    // - from_config_ws: what .cargo/config.toml points at (marker = FROM_CONFIG)
    // - source_root_ws: what --source-root discovers (marker = FROM_SOURCE_ROOT)

    let config_ws = tempfile::TempDir::new().unwrap();
    let config_root = config_ws.path().to_path_buf();
    build_lib_workspace(&config_root, "conflict-lib", "0.3.0", "FROM_CONFIG");
    git_init(&config_root);

    let sr_ws = tempfile::TempDir::new().unwrap();
    let sr_root = sr_ws.path().to_path_buf();
    build_lib_workspace(&sr_root, "conflict-lib", "0.3.0", "FROM_SOURCE_ROOT");
    git_init(&sr_root);

    // Project that depends on the git source; .cargo/config.toml points at config_ws.
    let proj_dir = tempfile::TempDir::new().unwrap();
    let proj_root = proj_dir.path().to_path_buf();
    std::fs::create_dir_all(proj_root.join(".cargo")).unwrap();
    std::fs::write(
        proj_root.join("Cargo.toml"),
        format!(
            r#"[package]
name = "proj"
version = "0.1.0"
edition = "2021"
publish = false

[workspace]

[lib]
path = "lib.rs"

[dependencies]
conflict-lib = {{ git = "{git_url}" }}
"#
        ),
    )
    .unwrap();
    std::fs::write(proj_root.join("lib.rs"), "pub use conflict_lib::patched;\n").unwrap();
    std::fs::write(
        proj_root.join(".cargo/config.toml"),
        format!(
            "[patch.\"{}\"]\nconflict-lib = {{ path = \"{}\" }}\n",
            git_url,
            config_root.join("conflict-lib").display()
        ),
    )
    .unwrap();
    git_init(&proj_root);

    let vendor = proj_root.join("vendor");
    revendor_cmd()
        .arg("revendor")
        .arg("--manifest-path")
        .arg(proj_root.join("Cargo.toml"))
        .arg("--output")
        .arg(&vendor)
        .arg("--source-root")
        .arg(&sr_root) // explicit; should win over config.toml entry
        .assert()
        .success();

    assert_vendor_has(&vendor, "conflict-lib");
    let crate_dir = vendor_dir_for(&vendor, "conflict-lib", None);
    let lib_rs = std::fs::read_to_string(crate_dir.join("src/lib.rs"))
        .expect("src/lib.rs not found in vendored conflict-lib");
    // The --source-root entry should win.
    assert!(
        lib_rs.contains("FROM_SOURCE_ROOT"),
        "vendored conflict-lib should come from --source-root, got:\n{lib_rs}"
    );
    assert!(
        !lib_rs.contains("FROM_CONFIG"),
        "vendored conflict-lib should NOT come from config.toml patch, got:\n{lib_rs}"
    );
}

/// **P4** — no `.cargo/config.toml` present: behavior is unchanged from the
/// pre-patch-detection baseline. Should succeed without any local crate
/// pre-seeding (the dep is a pure git dep with no local override).
#[test]
#[ignore] // invokes cargo vendor
fn no_config_toml_behaves_as_before() {
    let git_upstream = create_local_git_crate(
        "no-config-lib",
        r#"[package]
name = "no-config-lib"
version = "0.4.0"
edition = "2021"
publish = false
"#,
        "pub fn no_config() {}\n",
    );

    let work = tempfile::TempDir::new().unwrap();
    let root = work.path().join("project");
    let git_url = git_upstream.url();

    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join("Cargo.toml"),
        format!(
            r#"[package]
name = "test-proj"
version = "0.1.0"
edition = "2021"
publish = false

[workspace]

[lib]
path = "lib.rs"

[dependencies]
no-config-lib = {{ git = "{git_url}" }}
"#
        ),
    )
    .unwrap();
    std::fs::write(root.join("lib.rs"), "pub use no_config_lib::no_config;\n").unwrap();
    git_init(&root);
    // No .cargo/config.toml written.

    let vendor = root.join("vendor");
    revendor_cmd()
        .arg("revendor")
        .arg("--manifest-path")
        .arg(root.join("Cargo.toml"))
        .arg("--output")
        .arg(&vendor)
        .assert()
        .success();

    // The git dep from the bare local repo should still be vendored.
    assert_vendor_has(&vendor, "no-config-lib");
    let crate_dir = vendor_dir_for(&vendor, "no-config-lib", None);
    let lib_rs = std::fs::read_to_string(crate_dir.join("src/lib.rs"))
        .expect("src/lib.rs not found in vendored no-config-lib");
    assert!(
        lib_rs.contains("no_config"),
        "vendored no-config-lib should have original content, got:\n{lib_rs}"
    );
}

// Helper: build a workspace with a single `conflict-lib` member (no pkg subdir),
// used for the `--source-root` test where the source root IS the workspace.
fn build_lib_workspace(root: &Path, crate_name: &str, version: &str, marker: &str) {
    std::fs::write(
        root.join("Cargo.toml"),
        format!("[workspace]\nmembers = [\"{crate_name}\"]\nresolver = \"2\"\n"),
    )
    .unwrap();
    std::fs::create_dir_all(root.join(format!("{crate_name}/src"))).unwrap();
    std::fs::write(
        root.join(format!("{crate_name}/Cargo.toml")),
        format!(
            r#"[package]
name = "{crate_name}"
version = "{version}"
edition = "2021"
publish = false

[lib]
path = "src/lib.rs"
"#
        ),
    )
    .unwrap();
    std::fs::write(
        root.join(format!("{crate_name}/src/lib.rs")),
        format!("// MARKER: {marker}\npub fn patched() -> &'static str {{ \"{marker}\" }}\n"),
    )
    .unwrap();
}
