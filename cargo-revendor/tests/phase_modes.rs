//! Integration tests for --external-only / --local-only phase modes (#290).
//!
//! Tests marked `#[ignore]` require network access or a full cargo/rustc
//! toolchain for vendoring. Offline tests (flag compatibility) run without
//! any network or package-compilation.
//!
//! Run all tests including ignored ones:
//!   cargo test -p cargo-revendor -- --ignored

mod common;

use common::{create_workspace, git_init, revendor_cmd};

// =============================================================================
// Flag compatibility (offline — no cargo vendor invoked)
// =============================================================================

/// --external-only --freeze must exit non-zero with a clear error.
#[test]
fn flag_compat_external_only_freeze_exits_nonzero() {
    let proj = create_workspace(
        r#"[workspace]
members = ["mypkg"]
"#,
        &[(
            "mypkg",
            r#"[package]
name = "mypkg"
version = "0.1.0"
edition = "2021"
[lib]
path = "lib.rs"
"#,
            "pub fn hello() {}",
        )],
    );

    revendor_cmd()
        .args([
            "revendor",
            "--manifest-path",
            proj.root().join("mypkg/Cargo.toml").to_str().unwrap(),
            "--external-only",
            "--freeze",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "--external-only is incompatible with --freeze",
        ));
}

/// --external-only --compress must exit non-zero.
#[test]
fn flag_compat_external_only_compress_exits_nonzero() {
    let proj = create_workspace(
        r#"[workspace]
members = ["mypkg"]
"#,
        &[(
            "mypkg",
            r#"[package]
name = "mypkg"
version = "0.1.0"
edition = "2021"
[lib]
path = "lib.rs"
"#,
            "pub fn hello() {}",
        )],
    );

    revendor_cmd()
        .args([
            "revendor",
            "--manifest-path",
            proj.root().join("mypkg/Cargo.toml").to_str().unwrap(),
            "--external-only",
            "--compress",
            "vendor.tar.xz",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "--external-only is incompatible with --compress",
        ));
}

/// --local-only --compress without a prior --external-only run must exit non-zero.
#[test]
fn flag_compat_local_only_compress_without_externals_exits_nonzero() {
    let proj = create_workspace(
        r#"[workspace]
members = ["mypkg"]
"#,
        &[(
            "mypkg",
            r#"[package]
name = "mypkg"
version = "0.1.0"
edition = "2021"
[lib]
path = "lib.rs"
"#,
            "pub fn hello() {}",
        )],
    );

    // Vendor dir is empty (no .revendor-cache-external) — should fail.
    let vendor = proj.root().join("vendor");

    revendor_cmd()
        .args([
            "revendor",
            "--manifest-path",
            proj.root().join("mypkg/Cargo.toml").to_str().unwrap(),
            "--output",
            vendor.to_str().unwrap(),
            "--local-only",
            "--compress",
            "vendor.tar.xz",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains(".revendor-cache-external not found"));
}

/// --external-only and --local-only are mutually exclusive (clap enforces this).
#[test]
fn flag_compat_external_and_local_only_are_exclusive() {
    let proj = create_workspace(
        r#"[workspace]
members = ["mypkg"]
"#,
        &[(
            "mypkg",
            r#"[package]
name = "mypkg"
version = "0.1.0"
edition = "2021"
[lib]
path = "lib.rs"
"#,
            "pub fn hello() {}",
        )],
    );

    revendor_cmd()
        .args([
            "revendor",
            "--manifest-path",
            proj.root().join("mypkg/Cargo.toml").to_str().unwrap(),
            "--external-only",
            "--local-only",
        ])
        .assert()
        .failure();
}

// =============================================================================
// Phase mode vendoring (network required)
// =============================================================================

/// --external-only produces versioned dirs and writes .revendor-cache-external.
#[test]
#[ignore] // network
fn external_only_produces_versioned_dirs_and_cache_file() {
    let proj = create_workspace(
        r#"[workspace]
members = ["rpkg", "myhelper"]
"#,
        &[
            (
                "rpkg",
                r#"[package]
name = "rpkg"
version = "0.1.0"
edition = "2021"
[lib]
path = "lib.rs"
[dependencies]
myhelper = { path = "../myhelper" }
cfg-if = "1"
"#,
                "pub fn go() {}",
            ),
            (
                "myhelper",
                r#"[package]
name = "myhelper"
version = "0.1.0"
edition = "2021"
[lib]
path = "lib.rs"
"#,
                "pub fn help() {}",
            ),
        ],
    );
    let vendor = proj.root().join("vendor");

    revendor_cmd()
        .args([
            "revendor",
            "--manifest-path",
            proj.root().join("rpkg/Cargo.toml").to_str().unwrap(),
            "--output",
            vendor.to_str().unwrap(),
            "--source-root",
            proj.root().to_str().unwrap(),
            "--external-only",
        ])
        .assert()
        .success();

    // External cache file must exist.
    assert!(
        vendor.join(".revendor-cache-external").exists(),
        ".revendor-cache-external should be written by --external-only"
    );

    // Local-only cache file must NOT exist (we didn't run --local-only).
    assert!(
        !vendor.join(".revendor-cache-local").exists(),
        ".revendor-cache-local should not exist after --external-only"
    );

    // cfg-if (external) must be present as a versioned dir.
    let has_cfg_if = std::fs::read_dir(&vendor)
        .unwrap()
        .flatten()
        .any(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("cfg-if-")
        });
    assert!(has_cfg_if, "cfg-if versioned dir should be in vendor/");

    // myhelper (local) must NOT be present (we skipped local packaging).
    assert!(
        !common::vendor_has(&vendor, "myhelper"),
        "myhelper (local) should not be in vendor/ after --external-only"
    );
}

/// --local-only after --external-only: only flat dirs appear; versioned dirs
/// are untouched.
#[test]
#[ignore] // network
fn local_only_after_external_only_does_not_clobber_versioned_dirs() {
    let proj = create_workspace(
        r#"[workspace]
members = ["rpkg", "myhelper"]
"#,
        &[
            (
                "rpkg",
                r#"[package]
name = "rpkg"
version = "0.1.0"
edition = "2021"
[lib]
path = "lib.rs"
[dependencies]
myhelper = { path = "../myhelper" }
cfg-if = "1"
"#,
                "pub fn go() {}",
            ),
            (
                "myhelper",
                r#"[package]
name = "myhelper"
version = "0.1.0"
edition = "2021"
[lib]
path = "lib.rs"
"#,
                "pub fn help() {}",
            ),
        ],
    );
    let vendor = proj.root().join("vendor");

    // Step 1: external-only
    revendor_cmd()
        .args([
            "revendor",
            "--manifest-path",
            proj.root().join("rpkg/Cargo.toml").to_str().unwrap(),
            "--output",
            vendor.to_str().unwrap(),
            "--source-root",
            proj.root().to_str().unwrap(),
            "--external-only",
        ])
        .assert()
        .success();

    // Record mtime of a versioned dir to verify it's untouched later.
    let cfg_if_dir = std::fs::read_dir(&vendor)
        .unwrap()
        .flatten()
        .find(|e| e.file_name().to_string_lossy().starts_with("cfg-if-"))
        .expect("cfg-if should be vendored")
        .path();
    let before_mtime = std::fs::metadata(&cfg_if_dir)
        .unwrap()
        .modified()
        .unwrap();

    // Step 2: local-only
    revendor_cmd()
        .args([
            "revendor",
            "--manifest-path",
            proj.root().join("rpkg/Cargo.toml").to_str().unwrap(),
            "--output",
            vendor.to_str().unwrap(),
            "--source-root",
            proj.root().to_str().unwrap(),
            "--local-only",
        ])
        .assert()
        .success();

    // myhelper (local) must now be present.
    assert!(
        common::vendor_has(&vendor, "myhelper"),
        "myhelper should be vendored after --local-only"
    );

    // cfg-if versioned dir must be untouched.
    let after_mtime = std::fs::metadata(&cfg_if_dir)
        .unwrap()
        .modified()
        .unwrap();
    assert_eq!(
        before_mtime, after_mtime,
        "cfg-if-* mtime should be unchanged after --local-only"
    );

    // .revendor-cache-local must now exist.
    assert!(
        vendor.join(".revendor-cache-local").exists(),
        ".revendor-cache-local should be written after --local-only"
    );
}

/// After --external-only then --local-only, the vendor/ tree should equal
/// what a full run produces.
#[test]
#[ignore] // network
fn phase_modes_compose_to_full() {
    // Project A: two-phase run
    let proj_a = create_workspace(
        r#"[workspace]
members = ["rpkg", "myhelper"]
"#,
        &[
            (
                "rpkg",
                r#"[package]
name = "rpkg"
version = "0.1.0"
edition = "2021"
[lib]
path = "lib.rs"
[dependencies]
myhelper = { path = "../myhelper" }
cfg-if = "1"
"#,
                "pub fn go() {}",
            ),
            (
                "myhelper",
                r#"[package]
name = "myhelper"
version = "0.1.0"
edition = "2021"
[lib]
path = "lib.rs"
"#,
                "pub fn help() {}",
            ),
        ],
    );
    let vendor_a = proj_a.root().join("vendor");
    git_init(proj_a.root());

    revendor_cmd()
        .args([
            "revendor",
            "--manifest-path",
            proj_a.root().join("rpkg/Cargo.toml").to_str().unwrap(),
            "--output",
            vendor_a.to_str().unwrap(),
            "--source-root",
            proj_a.root().to_str().unwrap(),
            "--external-only",
        ])
        .assert()
        .success();

    revendor_cmd()
        .args([
            "revendor",
            "--manifest-path",
            proj_a.root().join("rpkg/Cargo.toml").to_str().unwrap(),
            "--output",
            vendor_a.to_str().unwrap(),
            "--source-root",
            proj_a.root().to_str().unwrap(),
            "--local-only",
        ])
        .assert()
        .success();

    // Project B: full run (same manifest)
    let proj_b = create_workspace(
        r#"[workspace]
members = ["rpkg", "myhelper"]
"#,
        &[
            (
                "rpkg",
                r#"[package]
name = "rpkg"
version = "0.1.0"
edition = "2021"
[lib]
path = "lib.rs"
[dependencies]
myhelper = { path = "../myhelper" }
cfg-if = "1"
"#,
                "pub fn go() {}",
            ),
            (
                "myhelper",
                r#"[package]
name = "myhelper"
version = "0.1.0"
edition = "2021"
[lib]
path = "lib.rs"
"#,
                "pub fn help() {}",
            ),
        ],
    );
    let vendor_b = proj_b.root().join("vendor");
    git_init(proj_b.root());

    revendor_cmd()
        .args([
            "revendor",
            "--manifest-path",
            proj_b.root().join("rpkg/Cargo.toml").to_str().unwrap(),
            "--output",
            vendor_b.to_str().unwrap(),
            "--source-root",
            proj_b.root().to_str().unwrap(),
        ])
        .assert()
        .success();

    // Compare the Cargo.toml of each crate (content, not mtime).
    let dirs_a: std::collections::BTreeSet<String> = std::fs::read_dir(&vendor_a)
        .unwrap()
        .flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| !n.starts_with('.'))
        .collect();

    let dirs_b: std::collections::BTreeSet<String> = std::fs::read_dir(&vendor_b)
        .unwrap()
        .flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| !n.starts_with('.'))
        .collect();

    let only_in_a: Vec<_> = dirs_a.difference(&dirs_b).collect();
    let only_in_b: Vec<_> = dirs_b.difference(&dirs_a).collect();

    assert!(
        only_in_a.is_empty() && only_in_b.is_empty(),
        "phase-composed vendor/ differs from full vendor/:\n  only in A: {only_in_a:?}\n  only in B: {only_in_b:?}"
    );
}

/// A second --external-only run with unchanged lockfile should be a cache hit.
#[test]
#[ignore] // network
fn external_cache_hit_skips_cargo_vendor() {
    let proj = create_workspace(
        r#"[workspace]
members = ["mypkg"]
"#,
        &[(
            "mypkg",
            r#"[package]
name = "mypkg"
version = "0.1.0"
edition = "2021"
[lib]
path = "lib.rs"
[dependencies]
cfg-if = "1"
"#,
            "pub fn hello() {}",
        )],
    );
    let vendor = proj.root().join("vendor");
    let manifest = proj.root().join("mypkg/Cargo.toml");

    // First run
    revendor_cmd()
        .args([
            "revendor",
            "--manifest-path",
            manifest.to_str().unwrap(),
            "--output",
            vendor.to_str().unwrap(),
            "--external-only",
        ])
        .assert()
        .success();

    // Record mtime of the cache file.
    let cache_mtime = std::fs::metadata(vendor.join(".revendor-cache-external"))
        .unwrap()
        .modified()
        .unwrap();

    // Second run — should be a no-op (cache hit)
    revendor_cmd()
        .args([
            "revendor",
            "--manifest-path",
            manifest.to_str().unwrap(),
            "--output",
            vendor.to_str().unwrap(),
            "--external-only",
        ])
        .assert()
        .success();

    // Cache file mtime must not change — nothing was written.
    let cache_mtime2 = std::fs::metadata(vendor.join(".revendor-cache-external"))
        .unwrap()
        .modified()
        .unwrap();
    assert_eq!(
        cache_mtime, cache_mtime2,
        ".revendor-cache-external should not be rewritten on a cache hit"
    );
}
