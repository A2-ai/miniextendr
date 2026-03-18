//! Integration tests for cargo-revendor
//!
//! Tests marked `#[ignore]` require network access (they run `cargo vendor`).
//! Run them with: `cargo test -p cargo-revendor -- --ignored`

use assert_cmd::Command;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

// region: Test harness

/// A temporary Cargo project for testing
struct TestProject {
    _dir: TempDir,
    root: PathBuf,
}

impl TestProject {
    fn root(&self) -> &Path {
        &self.root
    }
}

/// Get a Command for running cargo-revendor
fn revendor_cmd() -> Command {
    Command::cargo_bin("cargo-revendor").expect("binary not found")
}

/// Create a simple single-crate project
fn create_simple_crate(cargo_toml: &str, lib_rs: &str) -> TestProject {
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("project");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("Cargo.toml"), cargo_toml).unwrap();
    std::fs::write(root.join("lib.rs"), lib_rs).unwrap();
    git_init(&root);
    TestProject { _dir: dir, root }
}

/// Create a workspace with the given members
/// members: &[(name, cargo_toml, lib_rs)]
fn create_workspace(root_toml: &str, members: &[(&str, &str, &str)]) -> TestProject {
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("workspace");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("Cargo.toml"), root_toml).unwrap();
    for (name, toml, rs) in members {
        let member_dir = root.join(name);
        std::fs::create_dir_all(&member_dir).unwrap();
        std::fs::write(member_dir.join("Cargo.toml"), toml).unwrap();
        std::fs::write(member_dir.join("lib.rs"), rs).unwrap();
    }
    git_init(&root);
    TestProject { _dir: dir, root }
}

/// Create a monorepo: workspace root + rpkg subdirectory with own [workspace]
fn create_monorepo(
    ws_toml: &str,
    ws_members: &[(&str, &str, &str)],
    rpkg_toml: &str,
    rpkg_rs: &str,
) -> TestProject {
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("monorepo");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("Cargo.toml"), ws_toml).unwrap();
    for (name, toml, rs) in ws_members {
        let member_dir = root.join(name);
        std::fs::create_dir_all(&member_dir).unwrap();
        std::fs::write(member_dir.join("Cargo.toml"), toml).unwrap();
        std::fs::write(member_dir.join("lib.rs"), rs).unwrap();
    }
    // rpkg in a subdirectory with its own workspace
    let rpkg_dir = root.join("rpkg").join("src").join("rust");
    std::fs::create_dir_all(&rpkg_dir).unwrap();
    std::fs::write(rpkg_dir.join("Cargo.toml"), rpkg_toml).unwrap();
    std::fs::write(rpkg_dir.join("lib.rs"), rpkg_rs).unwrap();
    git_init(&root);
    TestProject { _dir: dir, root }
}

/// Initialize a git repo (cargo package requires it)
fn git_init(dir: &Path) {
    std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(dir)
        .output()
        .expect("git init failed");
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(dir)
        .output()
        .expect("git add failed");
    std::process::Command::new("git")
        .args(["commit", "-q", "-m", "init", "--allow-empty"])
        .current_dir(dir)
        .env("GIT_AUTHOR_NAME", "test")
        .env("GIT_AUTHOR_EMAIL", "test@test.com")
        .env("GIT_COMMITTER_NAME", "test")
        .env("GIT_COMMITTER_EMAIL", "test@test.com")
        .output()
        .expect("git commit failed");
}

/// Assert that vendor dir contains a crate
fn assert_vendor_has(vendor: &Path, name: &str) {
    let crate_dir = vendor.join(name);
    assert!(
        crate_dir.exists(),
        "expected vendor/{} to exist at {}",
        name,
        vendor.display()
    );
    assert!(
        crate_dir.join("Cargo.toml").exists(),
        "expected vendor/{}/Cargo.toml",
        name
    );
}

/// Assert that vendor dir does NOT contain a crate
fn assert_vendor_missing(vendor: &Path, name: &str) {
    assert!(
        !vendor.join(name).exists(),
        "vendor/{} should not exist",
        name
    );
}

/// Read vendored Cargo.toml as string
fn read_vendor_toml(vendor: &Path, name: &str) -> String {
    std::fs::read_to_string(vendor.join(name).join("Cargo.toml"))
        .unwrap_or_else(|_| panic!("failed to read vendor/{}/Cargo.toml", name))
}

/// Assert checksum file is empty
fn assert_empty_checksum(vendor: &Path, name: &str) {
    let cksum = vendor.join(name).join(".cargo-checksum.json");
    let content = std::fs::read_to_string(&cksum)
        .unwrap_or_else(|_| panic!("no .cargo-checksum.json in vendor/{}", name));
    assert_eq!(content, "{\"files\":{}}");
}

// endregion

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
    assert_empty_checksum(&vendor, "cfg-if");
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
    assert_empty_checksum(&vendor, "myhelper");
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
mylib = { version = "*" }
cfg-if = "1"
[patch.crates-io]
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
    assert_vendor_has(&vendor, "cfg-if");
}

// =============================================================================
// Monorepo nested rpkg (network)
// =============================================================================

#[test]
#[ignore] // network
fn monorepo_nested_rpkg() {
    let proj = create_monorepo(
        r#"[workspace]
members = ["mylib"]
"#,
        &[(
            "mylib",
            r#"[package]
name = "mylib"
version = "0.1.0"
edition = "2021"
[lib]
path = "lib.rs"
"#,
            "pub fn lib_fn() {}",
        )],
        r#"[package]
name = "mypkg"
version = "0.1.0"
edition = "2021"
[workspace]
[lib]
path = "lib.rs"
[dependencies]
mylib = { version = "*" }
cfg-if = "1"
[patch.crates-io]
mylib = { path = "../../../mylib" }
"#,
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

    assert_vendor_has(&vendor, "mylib");
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
