//! Integration tests for cargo-revendor.
//!
//! Tests marked `#[ignore]` require network access (they run `cargo vendor`).
//! Run them with: `cargo test -p cargo-revendor -- --ignored`.
//!
//! Shared harness lives in `common/mod.rs` (imported below). Split-out test
//! files in `tests/*.rs` should do the same so all binaries share the same
//! helpers. See issue #226 for the extraction rationale.

mod common;

use common::{
    assert_valid_checksum, assert_vendor_has, assert_vendor_missing, create_monorepo,
    create_simple_crate, create_workspace, git_init, read_vendor_toml, revendor_cmd,
};

// =============================================================================
// Error cases (offline)
// =============================================================================

#[test]
fn error_missing_manifest() {
    revendor_cmd()
        .args(["revendor", "--manifest-path", "/tmp/nonexistent/Cargo.toml"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("manifest not found"));
}

// =============================================================================
// Simple single crate (network)
// =============================================================================

#[test]
#[ignore] // network
fn simple_crate_with_cratesio_dep() {
    let proj = create_simple_crate(
        r#"[package]
name = "testpkg"
version = "0.1.0"
edition = "2021"
[workspace]
[lib]
path = "lib.rs"
[dependencies]
cfg-if = "1"
"#,
        "pub fn hello() {}",
    );
    let vendor = proj.root().join("vendor");

    revendor_cmd()
        .args([
            "revendor",
            "--manifest-path",
            proj.root().join("Cargo.toml").to_str().unwrap(),
            "--output",
            vendor.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert_vendor_has(&vendor, "cfg-if");
    assert_valid_checksum(&vendor, "cfg-if");
    assert_vendor_missing(&vendor, "testpkg"); // target crate not vendored
}

// =============================================================================
// Workspace with sibling dep (network)
// =============================================================================

#[test]
#[ignore] // network
fn workspace_sibling_dep() {
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
        ])
        .assert()
        .success();

    assert_vendor_has(&vendor, "myhelper");
    assert_vendor_has(&vendor, "cfg-if");
    assert_valid_checksum(&vendor, "cfg-if");
    assert_vendor_missing(&vendor, "rpkg");
}

// =============================================================================
// Transitive local deps (network)
// =============================================================================

#[test]
#[ignore] // network
fn workspace_transitive_local_deps() {
    let proj = create_workspace(
        r#"[workspace]
members = ["app", "mid", "leaf"]
"#,
        &[
            (
                "app",
                r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"
[lib]
path = "lib.rs"
[dependencies]
mid = { path = "../mid" }
cfg-if = "1"
"#,
                "pub fn go() {}",
            ),
            (
                "mid",
                r#"[package]
name = "mid"
version = "0.1.0"
edition = "2021"
[lib]
path = "lib.rs"
[dependencies]
leaf = { path = "../leaf" }
"#,
                "pub fn middle() {}",
            ),
            (
                "leaf",
                r#"[package]
name = "leaf"
version = "0.1.0"
edition = "2021"
[lib]
path = "lib.rs"
"#,
                "pub fn bottom() {}",
            ),
        ],
    );
    let vendor = proj.root().join("vendor");

    revendor_cmd()
        .args([
            "revendor",
            "--manifest-path",
            proj.root().join("app/Cargo.toml").to_str().unwrap(),
            "--output",
            vendor.to_str().unwrap(),
            "--source-root",
            proj.root().to_str().unwrap(),
        ])
        .assert()
        .success();

    assert_vendor_has(&vendor, "mid");
    assert_vendor_has(&vendor, "leaf");
    assert_vendor_has(&vendor, "cfg-if");

    // Verify path rewriting: mid should reference leaf
    let mid_toml = read_vendor_toml(&vendor, "mid");
    assert!(
        mid_toml.contains("path = \"../leaf\""),
        "mid should have path dep to leaf:\n{}",
        mid_toml
    );
}

// =============================================================================
// patch.crates-io pattern (network)
// =============================================================================

#[test]
#[ignore] // network
fn patch_cratesio_pattern() {
    // Use a crate name that is guaranteed not to exist on crates.io so the
    // version from the local workspace (0.1.0) never conflicts with a
    // registry resolution.
    //
    // The [patch.crates-io] must be placed in the workspace root manifest,
    // not in a member manifest — cargo ignores member-level [patch] in
    // workspace context.
    let crate_name = "zzz-cargo-revendor-test-fixture-mylib";
    let proj = create_workspace(
        &format!(
            r#"[workspace]
members = ["rpkg", "{crate_name}"]
resolver = "2"

[patch.crates-io]
{crate_name} = {{ path = "{crate_name}" }}
"#
        ),
        &[
            (
                "rpkg",
                &format!(
                    r#"[package]
name = "rpkg"
version = "0.1.0"
edition = "2021"
[lib]
path = "lib.rs"
[dependencies]
{crate_name} = {{ version = "0.1.0" }}
cfg-if = "1"
"#
                ),
                "pub fn go() {}",
            ),
            (
                crate_name,
                &format!(
                    r#"[package]
name = "{crate_name}"
version = "0.1.0"
edition = "2021"
[lib]
path = "lib.rs"
"#
                ),
                "pub fn lib_fn() {}",
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
        ])
        .assert()
        .success();

    assert_vendor_has(&vendor, crate_name);
    assert_vendor_has(&vendor, "cfg-if");
}

// =============================================================================
// Monorepo nested rpkg (network)
// =============================================================================

#[test]
#[ignore] // network
fn monorepo_nested_rpkg() {
    // Use a crate name that doesn't exist on crates.io to avoid version
    // conflicts between the local workspace copy and the registry.
    let crate_name = "zzz-cargo-revendor-test-fixture-mylib";
    let proj = create_monorepo(
        &format!("[workspace]\nmembers = [\"{crate_name}\"]\n"),
        &[(
            crate_name,
            &format!(
                r#"[package]
name = "{crate_name}"
version = "0.1.0"
edition = "2021"
[lib]
path = "lib.rs"
"#
            ),
            "pub fn lib_fn() {}",
        )],
        &format!(
            r#"[package]
name = "mypkg"
version = "0.1.0"
edition = "2021"
[workspace]
[lib]
path = "lib.rs"
[dependencies]
{crate_name} = {{ version = "0.1.0" }}
cfg-if = "1"
[patch.crates-io]
{crate_name} = {{ path = "../../../{crate_name}" }}
"#
        ),
        "pub fn go() {}",
    );
    let vendor = proj.root().join("vendor");

    revendor_cmd()
        .args([
            "revendor",
            "--manifest-path",
            proj.root()
                .join("rpkg/src/rust/Cargo.toml")
                .to_str()
                .unwrap(),
            "--output",
            vendor.to_str().unwrap(),
            "--source-root",
            proj.root().to_str().unwrap(),
        ])
        .assert()
        .success();

    assert_vendor_has(&vendor, crate_name);
    assert_vendor_has(&vendor, "cfg-if");
}

// =============================================================================
// Workspace version inheritance (network)
// =============================================================================

#[test]
#[ignore] // network
fn workspace_version_inheritance() {
    let proj = create_workspace(
        r#"[workspace]
members = ["rpkg", "myhelper"]
[workspace.package]
version = "1.2.3"
edition = "2021"
"#,
        &[
            (
                "rpkg",
                r#"[package]
name = "rpkg"
version.workspace = true
edition.workspace = true
[lib]
path = "lib.rs"
[dependencies]
myhelper = { path = "../myhelper" }
"#,
                "pub fn go() {}",
            ),
            (
                "myhelper",
                r#"[package]
name = "myhelper"
version.workspace = true
edition.workspace = true
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
        ])
        .assert()
        .success();

    assert_vendor_has(&vendor, "myhelper");
    // Workspace inheritance should be resolved by the direct-copy fallback
    let helper_toml = read_vendor_toml(&vendor, "myhelper");
    assert!(
        helper_toml.contains("\"1.2.3\""),
        "workspace version should be resolved to 1.2.3:\n{}",
        helper_toml
    );
}

// =============================================================================
// Build dependencies (network)
// =============================================================================

#[test]
#[ignore] // network
fn build_dependencies_vendored() {
    let proj = create_simple_crate(
        r#"[package]
name = "testpkg"
version = "0.1.0"
edition = "2021"
[workspace]
[lib]
path = "lib.rs"
[dependencies]
cfg-if = "1"
[build-dependencies]
cfg-if = "1"
"#,
        "pub fn hello() {}",
    );
    let vendor = proj.root().join("vendor");

    revendor_cmd()
        .args([
            "revendor",
            "--manifest-path",
            proj.root().join("Cargo.toml").to_str().unwrap(),
            "--output",
            vendor.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert_vendor_has(&vendor, "cfg-if");
}

// =============================================================================
// Stripping via full pipeline (network)
// =============================================================================

#[test]
#[ignore] // network
fn stripping_removes_test_bench_dirs() {
    // Create workspace where local crate has tests/ and benches/
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

[[test]]
name = "integration"
path = "tests/integration.rs"

[[bench]]
name = "perf"
harness = false

[dev-dependencies]
criterion = "0.5"
"#,
                "pub fn help() {}",
            ),
        ],
    );
    // Create the actual test/bench directories
    let helper_dir = proj.root().join("myhelper");
    std::fs::create_dir_all(helper_dir.join("tests")).unwrap();
    std::fs::write(helper_dir.join("tests/integration.rs"), "#[test] fn t() {}").unwrap();
    std::fs::create_dir_all(helper_dir.join("benches")).unwrap();
    std::fs::write(helper_dir.join("benches/perf.rs"), "fn main() {}").unwrap();
    git_init(proj.root());

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
            "--strip-all",
        ])
        .assert()
        .success();

    assert_vendor_has(&vendor, "myhelper");
    assert!(
        !vendor.join("myhelper/tests").exists(),
        "tests/ should be stripped"
    );
    assert!(
        !vendor.join("myhelper/benches").exists(),
        "benches/ should be stripped"
    );
    let toml = read_vendor_toml(&vendor, "myhelper");
    assert!(!toml.contains("[[test]]"), "[[test]] should be stripped");
    assert!(!toml.contains("[[bench]]"), "[[bench]] should be stripped");
    assert!(
        !toml.contains("[dev-dependencies]"),
        "[dev-dependencies] should be stripped"
    );
}

// =============================================================================
// Path rewriting (network)
// =============================================================================

#[test]
#[ignore] // network
fn path_rewriting_inline_and_section() {
    let proj = create_workspace(
        r#"[workspace]
members = ["app", "liba", "libb"]
"#,
        &[
            (
                "app",
                r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"
[lib]
path = "lib.rs"
[dependencies]
liba = { path = "../liba" }
libb = { path = "../libb" }
"#,
                "pub fn go() {}",
            ),
            (
                "liba",
                r#"[package]
name = "liba"
version = "0.1.0"
edition = "2021"
[lib]
path = "lib.rs"
[dependencies]
libb = { path = "../libb" }
"#,
                "pub fn a() {}",
            ),
            (
                "libb",
                r#"[package]
name = "libb"
version = "0.1.0"
edition = "2021"
[lib]
path = "lib.rs"
"#,
                "pub fn b() {}",
            ),
        ],
    );
    let vendor = proj.root().join("vendor");

    revendor_cmd()
        .args([
            "revendor",
            "--manifest-path",
            proj.root().join("app/Cargo.toml").to_str().unwrap(),
            "--output",
            vendor.to_str().unwrap(),
            "--source-root",
            proj.root().to_str().unwrap(),
        ])
        .assert()
        .success();

    // liba should reference libb with path
    let liba_toml = read_vendor_toml(&vendor, "liba");
    assert!(
        liba_toml.contains("path = \"../libb\""),
        "liba should have path dep to libb:\n{}",
        liba_toml
    );
}

// =============================================================================
// --no-strip flag (network)
// =============================================================================

#[test]
#[ignore] // network
fn no_strip_preserves_directories() {
    let proj = create_simple_crate(
        r#"[package]
name = "testpkg"
version = "0.1.0"
edition = "2021"
[workspace]
[lib]
path = "lib.rs"
[dependencies]
cfg-if = "1"
"#,
        "pub fn hello() {}",
    );
    let vendor = proj.root().join("vendor");

    revendor_cmd()
        .args([
            "revendor",
            "--manifest-path",
            proj.root().join("Cargo.toml").to_str().unwrap(),
            "--output",
            vendor.to_str().unwrap(),
            // No --strip-* flags = no stripping (opt-in)
        ])
        .assert()
        .success();

    assert_vendor_has(&vendor, "cfg-if");
    // With --no-strip, the cfg-if crate's files should be untouched
    // (cfg-if is tiny so it may not have tests, but the flag should work)
}

// =============================================================================
// Broken crate with --no-verify (network)
// =============================================================================

#[test]
#[ignore] // network
fn broken_crate_still_packages() {
    let proj = create_workspace(
        r#"[workspace]
members = ["rpkg", "broken"]
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
broken = { path = "../broken" }
"#,
                "pub fn go() {}",
            ),
            (
                "broken",
                r#"[package]
name = "broken"
version = "0.1.0"
edition = "2021"
[lib]
path = "lib.rs"
"#,
                "this is not valid rust!!!", // won't compile
            ),
        ],
    );
    let vendor = proj.root().join("vendor");

    // Should succeed because cargo package uses --no-verify
    revendor_cmd()
        .args([
            "revendor",
            "--manifest-path",
            proj.root().join("rpkg/Cargo.toml").to_str().unwrap(),
            "--output",
            vendor.to_str().unwrap(),
            "--source-root",
            proj.root().to_str().unwrap(),
        ])
        .assert()
        .success();

    assert_vendor_has(&vendor, "broken");
}

// =============================================================================
// Raw path deps (auto-versioned by cargo-revendor, fallback to direct copy)
// =============================================================================

#[test]
#[ignore] // network
fn raw_path_deps_auto_versioned() {
    let proj = create_workspace(
        r#"[workspace]
members = ["app", "liba", "libb"]
"#,
        &[
            (
                "app",
                r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"
[lib]
path = "lib.rs"
[dependencies]
liba = { path = "../liba" }
cfg-if = "1"
"#,
                "pub fn go() {}",
            ),
            (
                "liba",
                r#"[package]
name = "liba"
version = "0.1.0"
edition = "2021"
[lib]
path = "lib.rs"
[dependencies]
libb = { path = "../libb" }
"#,
                "pub fn a() {}",
            ),
            (
                "libb",
                r#"[package]
name = "libb"
version = "0.1.0"
edition = "2021"
[lib]
path = "lib.rs"
"#,
                "pub fn b() {}",
            ),
        ],
    );
    let vendor = proj.root().join("vendor");

    revendor_cmd()
        .args([
            "revendor",
            "--manifest-path",
            proj.root().join("app/Cargo.toml").to_str().unwrap(),
            "--output",
            vendor.to_str().unwrap(),
            "--source-root",
            proj.root().to_str().unwrap(),
        ])
        .assert()
        .success();

    assert_vendor_has(&vendor, "liba");
    assert_vendor_has(&vendor, "libb");
    assert_vendor_has(&vendor, "cfg-if");

    // Verify path rewriting
    let liba_toml = read_vendor_toml(&vendor, "liba");
    assert!(
        liba_toml.contains("path = \"../libb\""),
        "liba should have path dep to libb:\n{}",
        liba_toml
    );

    // Verify original Cargo.toml was restored
    let original = std::fs::read_to_string(proj.root().join("liba/Cargo.toml")).unwrap();
    assert!(
        !original.contains("version = \"*\""),
        "original should be restored:\n{}",
        original
    );
}

// =============================================================================
// Path dep chain A → B → C
// =============================================================================

#[test]
#[ignore] // network
fn path_dep_chain_a_to_b_to_c() {
    let proj = create_workspace(
        r#"[workspace]
members = ["rpkg", "a", "b", "c"]
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
a = { path = "../a" }
"#,
                "pub fn go() {}",
            ),
            (
                "a",
                r#"[package]
name = "a"
version = "0.1.0"
edition = "2021"
[lib]
path = "lib.rs"
[dependencies]
b = { path = "../b" }
"#,
                "pub fn a_fn() {}",
            ),
            (
                "b",
                r#"[package]
name = "b"
version = "0.1.0"
edition = "2021"
[lib]
path = "lib.rs"
[dependencies]
c = { path = "../c" }
"#,
                "pub fn b_fn() {}",
            ),
            (
                "c",
                r#"[package]
name = "c"
version = "0.1.0"
edition = "2021"
[lib]
path = "lib.rs"
"#,
                "pub fn c_fn() {}",
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
        ])
        .assert()
        .success();

    assert_vendor_has(&vendor, "a");
    assert_vendor_has(&vendor, "b");
    assert_vendor_has(&vendor, "c");
    assert_vendor_missing(&vendor, "rpkg");

    let a_toml = read_vendor_toml(&vendor, "a");
    assert!(
        a_toml.contains("path = \"../b\""),
        "a should ref b:\n{}",
        a_toml
    );

    let b_toml = read_vendor_toml(&vendor, "b");
    assert!(
        b_toml.contains("path = \"../c\""),
        "b should ref c:\n{}",
        b_toml
    );
}

// =============================================================================
// JSON output
// =============================================================================

#[test]
#[ignore] // network
fn json_output_structure() {
    let proj = create_simple_crate(
        r#"[package]
name = "testpkg"
version = "0.1.0"
edition = "2021"
[workspace]
[lib]
path = "lib.rs"
[dependencies]
cfg-if = "1"
"#,
        "pub fn hello() {}",
    );
    let vendor = proj.root().join("vendor");

    let output = revendor_cmd()
        .args([
            "revendor",
            "--manifest-path",
            proj.root().join("Cargo.toml").to_str().unwrap(),
            "--output",
            vendor.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("invalid JSON output");
    assert!(json["vendor_dir"].is_string());
    assert!(json["total_crates"].is_number());
    assert!(json["cached"].is_boolean());
    assert_eq!(json["cached"], false);
}

// =============================================================================
// Caching (second run should be cached)
// =============================================================================

#[test]
#[ignore] // network
fn caching_skips_second_run() {
    let proj = create_simple_crate(
        r#"[package]
name = "testpkg"
version = "0.1.0"
edition = "2021"
[workspace]
[lib]
path = "lib.rs"
[dependencies]
cfg-if = "1"
"#,
        "pub fn hello() {}",
    );
    let vendor = proj.root().join("vendor");
    let manifest = proj.root().join("Cargo.toml");

    // First run
    revendor_cmd()
        .args([
            "revendor",
            "--manifest-path",
            manifest.to_str().unwrap(),
            "--output",
            vendor.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(
        vendor.join(".revendor-cache").exists(),
        "cache file should exist"
    );

    // Second run with --json to check cached flag
    let output = revendor_cmd()
        .args([
            "revendor",
            "--manifest-path",
            manifest.to_str().unwrap(),
            "--output",
            vendor.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("invalid JSON");
    assert_eq!(json["cached"], true, "second run should be cached");
}

// =============================================================================
// --force bypasses cache
// =============================================================================

#[test]
#[ignore] // network
fn force_bypasses_cache() {
    let proj = create_simple_crate(
        r#"[package]
name = "testpkg"
version = "0.1.0"
edition = "2021"
[workspace]
[lib]
path = "lib.rs"
[dependencies]
cfg-if = "1"
"#,
        "pub fn hello() {}",
    );
    let vendor = proj.root().join("vendor");
    let manifest = proj.root().join("Cargo.toml");

    // First run
    revendor_cmd()
        .args([
            "revendor",
            "--manifest-path",
            manifest.to_str().unwrap(),
            "--output",
            vendor.to_str().unwrap(),
        ])
        .assert()
        .success();

    // Second run with --force --json
    let output = revendor_cmd()
        .args([
            "revendor",
            "--manifest-path",
            manifest.to_str().unwrap(),
            "--output",
            vendor.to_str().unwrap(),
            "--force",
            "--json",
        ])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("invalid JSON");
    assert_eq!(json["cached"], false, "--force should bypass cache");
}

// =============================================================================
// Individual strip flags
// =============================================================================

#[test]
#[ignore] // network
fn strip_tests_only() {
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
    // Add tests and benches dirs to helper
    let helper_dir = proj.root().join("myhelper");
    std::fs::create_dir_all(helper_dir.join("tests")).unwrap();
    std::fs::write(helper_dir.join("tests/t.rs"), "").unwrap();
    std::fs::create_dir_all(helper_dir.join("benches")).unwrap();
    std::fs::write(helper_dir.join("benches/b.rs"), "").unwrap();
    git_init(proj.root());

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
            "--strip-tests", // only tests, NOT benches
        ])
        .assert()
        .success();

    assert_vendor_has(&vendor, "myhelper");
    assert!(
        !vendor.join("myhelper/tests").exists(),
        "tests/ should be stripped"
    );
    assert!(
        vendor.join("myhelper/benches").exists(),
        "benches/ should NOT be stripped (only --strip-tests)"
    );
}

// =============================================================================
// Empty vendor (no external deps)
// =============================================================================

#[test]
#[ignore] // network
fn empty_external_deps() {
    let proj = create_workspace(
        r#"[workspace]
members = ["rpkg", "mylib"]
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
mylib = { path = "../mylib" }
"#,
                "pub fn go() {}",
            ),
            (
                "mylib",
                r#"[package]
name = "mylib"
version = "0.1.0"
edition = "2021"
[lib]
path = "lib.rs"
"#,
                "pub fn lib_fn() {}",
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
        ])
        .assert()
        .success();

    assert_vendor_has(&vendor, "mylib");
}

// =============================================================================
// Config.toml and Cargo.lock output
// =============================================================================

#[test]
#[ignore] // network
fn generates_cargo_config_and_stripped_lockfile() {
    let proj = create_simple_crate(
        r#"[package]
name = "testpkg"
version = "0.1.0"
edition = "2021"
[workspace]
[lib]
path = "lib.rs"
[dependencies]
cfg-if = "1"
"#,
        "pub fn hello() {}",
    );
    let vendor = proj.root().join("vendor");

    revendor_cmd()
        .args([
            "revendor",
            "--manifest-path",
            proj.root().join("Cargo.toml").to_str().unwrap(),
            "--output",
            vendor.to_str().unwrap(),
        ])
        .assert()
        .success();

    // Check .cargo-config.toml was generated
    let config = std::fs::read_to_string(vendor.join(".cargo-config.toml"))
        .expect("should generate .cargo-config.toml");
    assert!(
        config.contains("[source.crates-io]"),
        "config should have crates-io source replacement"
    );
    assert!(
        config.contains("vendored-sources"),
        "config should reference vendored-sources"
    );

    // Check Cargo.lock was stripped and copied
    let lock = std::fs::read_to_string(vendor.join("Cargo.lock"))
        .expect("should copy stripped Cargo.lock");
    assert!(
        !lock.contains("checksum = "),
        "Cargo.lock should have checksums stripped"
    );
    assert!(
        lock.contains("cfg-if"),
        "Cargo.lock should still have dependency entries"
    );
}

// =============================================================================
// Optional dependencies
// =============================================================================

#[test]
#[ignore] // network
fn optional_dependencies() {
    let proj = create_simple_crate(
        r#"[package]
name = "testpkg"
version = "0.1.0"
edition = "2021"
[workspace]
[lib]
path = "lib.rs"
[dependencies]
cfg-if = { version = "1", optional = true }
[features]
default = ["cfg-if"]
"#,
        "pub fn hello() {}",
    );
    let vendor = proj.root().join("vendor");

    revendor_cmd()
        .args([
            "revendor",
            "--manifest-path",
            proj.root().join("Cargo.toml").to_str().unwrap(),
            "--output",
            vendor.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert_vendor_has(&vendor, "cfg-if");
}

// =============================================================================
// Features on path deps
// =============================================================================

#[test]
#[ignore] // network
fn features_on_path_deps() {
    let proj = create_workspace(
        r#"[workspace]
members = ["rpkg", "mylib"]
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
mylib = { path = "../mylib", features = ["extra"] }
"#,
                "pub fn go() {}",
            ),
            (
                "mylib",
                r#"[package]
name = "mylib"
version = "0.1.0"
edition = "2021"
[lib]
path = "lib.rs"
[features]
default = []
extra = []
"#,
                "pub fn lib_fn() {}",
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
        ])
        .assert()
        .success();

    assert_vendor_has(&vendor, "mylib");
}

// =============================================================================
// Workspace dependency inheritance (dep.workspace = true)
// =============================================================================

#[test]
#[ignore] // network
fn workspace_dep_inheritance() {
    let proj = create_workspace(
        r#"[workspace]
members = ["rpkg", "mylib"]
[workspace.dependencies]
cfg-if = "1"
mylib = { version = "0.1.0", path = "mylib" }
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
cfg-if.workspace = true
mylib.workspace = true
"#,
                "pub fn go() {}",
            ),
            (
                "mylib",
                r#"[package]
name = "mylib"
version = "0.1.0"
edition = "2021"
[lib]
path = "lib.rs"
"#,
                "pub fn lib_fn() {}",
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
        ])
        .assert()
        .success();

    assert_vendor_has(&vendor, "mylib");
    assert_vendor_has(&vendor, "cfg-if");
}

// =============================================================================
// Output directory already exists (should be replaced cleanly)
// =============================================================================

#[test]
#[ignore] // network
fn output_dir_replaced_cleanly() {
    let proj = create_simple_crate(
        r#"[package]
name = "testpkg"
version = "0.1.0"
edition = "2021"
[workspace]
[lib]
path = "lib.rs"
[dependencies]
cfg-if = "1"
"#,
        "pub fn hello() {}",
    );
    let vendor = proj.root().join("vendor");

    // Create a stale vendor dir with junk
    std::fs::create_dir_all(vendor.join("stale-crate")).unwrap();
    std::fs::write(vendor.join("stale-crate/Cargo.toml"), "[package]").unwrap();

    revendor_cmd()
        .args([
            "revendor",
            "--manifest-path",
            proj.root().join("Cargo.toml").to_str().unwrap(),
            "--output",
            vendor.to_str().unwrap(),
            "--force",
        ])
        .assert()
        .success();

    assert_vendor_has(&vendor, "cfg-if");
    assert_vendor_missing(&vendor, "stale-crate");
}
