//! Multi-workspace vendor scenarios for cargo-revendor (#230).
//!
//! The user's original ask behind this matrix: "a monorepo-like setup where
//! two incompatible versions of the same crate are used in one crate, but
//! another disjoint crate (like an rpkg crate) has another version, and they
//! both have to work off the same vendor directory."
//!
//! Three shapes:
//!
//! - **M1** — single workspace, two locked versions of the same crate
//!   coexisting as separate versioned dirs.
//! - **M2** — two disjoint workspaces vendored serially into the same
//!   `--output`; verifies current (pre-#229) behavior: the second run
//!   clobbers the first.
//! - **M3** — two disjoint workspaces using `--sync` (added in #229) to
//!   share one vendor/; union of both graphs materializes.

mod common;

use common::{assert_vendor_has, assert_vendor_missing, revendor_cmd};

/// **M1** — one workspace where two internal crates pin incompatible
/// versions of a shared dep. Cargo resolves both versions; verify that
/// cargo-revendor vendors both.
///
/// Uses `autocfg` — a leaf crate with no deps of its own — so the graph
/// is tiny and two locked versions can coexist without triggering
/// resolver issues. `autocfg 0.1` vs `autocfg 1.x` are incompatible
/// semver.
#[test]
#[ignore] // network
fn two_locked_versions_of_same_crate_in_one_workspace() {
    let work = tempfile::TempDir::new().unwrap();
    let root = work.path().join("ws");
    std::fs::create_dir_all(root.join("consumer_a/src")).unwrap();
    std::fs::create_dir_all(root.join("consumer_b/src")).unwrap();

    std::fs::write(
        root.join("Cargo.toml"),
        r#"[workspace]
members = ["consumer_a", "consumer_b"]
resolver = "2"
"#,
    )
    .unwrap();
    // Pin autocfg 0.1 — cargo resolves this as a separate major from 1.x.
    std::fs::write(
        root.join("consumer_a/Cargo.toml"),
        r#"[package]
name = "consumer_a"
version = "0.1.0"
edition = "2021"
publish = false

[dependencies]
autocfg = "=0.1.7"
"#,
    )
    .unwrap();
    std::fs::write(root.join("consumer_a/src/lib.rs"), "").unwrap();
    std::fs::write(
        root.join("consumer_b/Cargo.toml"),
        r#"[package]
name = "consumer_b"
version = "0.1.0"
edition = "2021"
publish = false

[dependencies]
autocfg = "=1.5.0"
"#,
    )
    .unwrap();
    std::fs::write(root.join("consumer_b/src/lib.rs"), "").unwrap();
    common::git_init(&root);

    let vendor = root.join("vendor");
    revendor_cmd()
        .arg("revendor")
        .arg("--manifest-path")
        .arg(root.join("consumer_a/Cargo.toml"))
        .arg("--output")
        .arg(&vendor)
        .assert()
        .success();

    // Multi-version layout: without `--versioned-dirs` (not yet wired up;
    // see #215), cargo vendor flattens one version to `vendor/<name>/` and
    // puts other versions in `vendor/<name>-<version>/`. Check that BOTH
    // versions are materialized regardless of which slot holds which.
    assert_autocfg_version_present(&vendor, "0.1.7");
    assert_autocfg_version_present(&vendor, "1.5.0");

    // Also the lockfile must record both versions.
    let lock = std::fs::read_to_string(root.join("Cargo.lock")).unwrap();
    assert!(
        lock.contains("version = \"0.1.7\"") && lock.contains("version = \"1.5.0\""),
        "Cargo.lock should pin both autocfg versions"
    );
}

/// Scan every `autocfg*` directory in `vendor/` and return success if any
/// has a Cargo.toml reporting the expected version. Accommodates cargo
/// vendor's flat-vs-versioned layout choice.
fn assert_autocfg_version_present(vendor: &std::path::Path, version: &str) {
    let mut seen_versions = Vec::new();
    for entry in std::fs::read_dir(vendor).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.starts_with("autocfg") {
            continue;
        }
        let toml = entry.path().join("Cargo.toml");
        if !toml.exists() {
            continue;
        }
        let content = std::fs::read_to_string(&toml).unwrap();
        // Extract the package-level version line.
        if let Some(v) = content.lines().find_map(|l| {
            let l = l.trim();
            l.strip_prefix("version = \"")
                .and_then(|s| s.strip_suffix('"'))
        }) {
            if v == version {
                return;
            }
            seen_versions.push(format!("{name} → {v}"));
        }
    }
    panic!(
        "expected autocfg version {version} under {} (saw: {:?})",
        vendor.display(),
        seen_versions
    );
}

/// **M2** — running cargo-revendor twice with the same `--output` but
/// different `--manifest-path` clobbers the first output. This is the
/// current behavior (main.rs step 8 does remove_dir_all + rename); the
/// test documents it so a future refactor doesn't silently regress.
///
/// Once callers want true shared-vendor, they use `--sync` (covered by
/// M3 below).
#[test]
#[ignore] // network
fn disjoint_workspaces_serial_vendor_clobbers_output() {
    let work = tempfile::TempDir::new().unwrap();

    let ws_a = work.path().join("ws_a");
    let ws_b = work.path().join("ws_b");
    std::fs::create_dir_all(&ws_a).unwrap();
    std::fs::create_dir_all(&ws_b).unwrap();

    std::fs::write(
        ws_a.join("Cargo.toml"),
        r#"[package]
name = "pkg_a"
version = "0.1.0"
edition = "2021"
publish = false

[workspace]

[lib]
path = "lib.rs"

[dependencies]
cfg-if = "1"
"#,
    )
    .unwrap();
    std::fs::write(ws_a.join("lib.rs"), "").unwrap();
    common::git_init(&ws_a);

    std::fs::write(
        ws_b.join("Cargo.toml"),
        r#"[package]
name = "pkg_b"
version = "0.1.0"
edition = "2021"
publish = false

[workspace]

[lib]
path = "lib.rs"

[dependencies]
once_cell = "1"
"#,
    )
    .unwrap();
    std::fs::write(ws_b.join("lib.rs"), "").unwrap();
    common::git_init(&ws_b);

    let shared_vendor = work.path().join("vendor");

    revendor_cmd()
        .arg("revendor")
        .arg("--manifest-path")
        .arg(ws_a.join("Cargo.toml"))
        .arg("--output")
        .arg(&shared_vendor)
        .assert()
        .success();
    assert_vendor_has(&shared_vendor, "cfg-if");

    revendor_cmd()
        .arg("revendor")
        .arg("--manifest-path")
        .arg(ws_b.join("Cargo.toml"))
        .arg("--output")
        .arg(&shared_vendor)
        .assert()
        .success();

    // Second run clobbered the first: cfg-if is gone, once_cell is there.
    assert_vendor_missing(&shared_vendor, "cfg-if");
    assert_vendor_has(&shared_vendor, "once_cell");
}

/// **M3** — two disjoint workspaces, single `--sync`-backed vendor. Both
/// graphs materialize. If ws_a pins `cfg-if` and ws_b pins `once_cell`,
/// both end up in the shared vendor tree.
///
/// This is the scenario that unblocks rpkg + miniextendr-bench sharing
/// one offline artifact.
#[test]
#[ignore] // network
fn disjoint_workspaces_sync_merges_into_shared_vendor() {
    let work = tempfile::TempDir::new().unwrap();

    let ws_a = work.path().join("ws_a");
    let ws_b = work.path().join("ws_b");
    std::fs::create_dir_all(&ws_a).unwrap();
    std::fs::create_dir_all(&ws_b).unwrap();

    std::fs::write(
        ws_a.join("Cargo.toml"),
        r#"[package]
name = "pkg_a"
version = "0.1.0"
edition = "2021"
publish = false

[workspace]

[lib]
path = "lib.rs"

[dependencies]
cfg-if = "1"
"#,
    )
    .unwrap();
    std::fs::write(ws_a.join("lib.rs"), "").unwrap();
    common::git_init(&ws_a);

    std::fs::write(
        ws_b.join("Cargo.toml"),
        r#"[package]
name = "pkg_b"
version = "0.1.0"
edition = "2021"
publish = false

[workspace]

[lib]
path = "lib.rs"

[dependencies]
once_cell = "1"
"#,
    )
    .unwrap();
    std::fs::write(ws_b.join("lib.rs"), "").unwrap();
    common::git_init(&ws_b);

    let shared_vendor = work.path().join("vendor");

    revendor_cmd()
        .arg("revendor")
        .arg("--manifest-path")
        .arg(ws_a.join("Cargo.toml"))
        .arg("--sync")
        .arg(ws_b.join("Cargo.toml"))
        .arg("--output")
        .arg(&shared_vendor)
        .assert()
        .success();

    // Both crates present after a single invocation — no clobbering.
    assert_vendor_has(&shared_vendor, "cfg-if");
    assert_vendor_has(&shared_vendor, "once_cell");
}

/// **M3b** — a cross-workspace version conflict resolved into two
/// versioned dirs under `--sync`. ws_a pins `autocfg = "=0.1.7"`,
/// ws_b pins `autocfg = "=1.5.0"`; the shared vendor holds both.
///
/// This is the exact scenario the issue's user called out: one workspace
/// on version X, another disjoint workspace on incompatible version Y,
/// both resolving off the same vendor/.
#[test]
#[ignore] // network
fn disjoint_workspaces_with_version_conflict_both_versions_vendored() {
    let work = tempfile::TempDir::new().unwrap();

    let ws_a = work.path().join("ws_a");
    let ws_b = work.path().join("ws_b");
    std::fs::create_dir_all(&ws_a).unwrap();
    std::fs::create_dir_all(&ws_b).unwrap();

    std::fs::write(
        ws_a.join("Cargo.toml"),
        r#"[package]
name = "pkg_a"
version = "0.1.0"
edition = "2021"
publish = false

[workspace]

[lib]
path = "lib.rs"

[dependencies]
autocfg = "=0.1.7"
"#,
    )
    .unwrap();
    std::fs::write(ws_a.join("lib.rs"), "").unwrap();
    common::git_init(&ws_a);

    std::fs::write(
        ws_b.join("Cargo.toml"),
        r#"[package]
name = "pkg_b"
version = "0.1.0"
edition = "2021"
publish = false

[workspace]

[lib]
path = "lib.rs"

[dependencies]
autocfg = "=1.5.0"
"#,
    )
    .unwrap();
    std::fs::write(ws_b.join("lib.rs"), "").unwrap();
    common::git_init(&ws_b);

    let shared_vendor = work.path().join("vendor");

    revendor_cmd()
        .arg("revendor")
        .arg("--manifest-path")
        .arg(ws_a.join("Cargo.toml"))
        .arg("--sync")
        .arg(ws_b.join("Cargo.toml"))
        .arg("--output")
        .arg(&shared_vendor)
        .assert()
        .success();

    // Both versions present. cargo vendor may flatten one to `autocfg/`.
    assert_autocfg_version_present(&shared_vendor, "0.1.7");
    assert_autocfg_version_present(&shared_vendor, "1.5.0");
}

/// **#215** — `--versioned-dirs` forces every crate into
/// `vendor/<name>-<version>/` instead of flattening single-version crates
/// to `vendor/<name>/`. Regression test for the opt-in flag.
#[test]
#[ignore] // network
fn versioned_dirs_flag_forces_versioned_layout() {
    let proj = common::create_simple_crate(
        r#"[package]
name = "vd-test"
version = "0.1.0"
edition = "2021"
publish = false

[workspace]

[lib]
path = "lib.rs"

[dependencies]
cfg-if = "1"
"#,
        "",
    );

    let vendor = proj.root().join("vendor");
    revendor_cmd()
        .arg("revendor")
        .arg("--manifest-path")
        .arg(proj.root().join("Cargo.toml"))
        .arg("--output")
        .arg(&vendor)
        .arg("--versioned-dirs")
        .assert()
        .success();

    // Without --versioned-dirs, cargo vendor would flatten cfg-if (only one
    // version in the graph) to `vendor/cfg-if/`. With the flag, the dir is
    // `vendor/cfg-if-<version>/`.
    let flat = vendor.join("cfg-if");
    let entries: Vec<String> = std::fs::read_dir(&vendor)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert!(
        !flat.exists(),
        "--versioned-dirs should prevent the flat `cfg-if/` slot, saw:\n{entries:#?}"
    );
    let versioned = entries
        .iter()
        .find(|e| e.starts_with("cfg-if-"))
        .unwrap_or_else(|| panic!("no cfg-if-<version>/ found in vendor, saw:\n{entries:#?}"));
    assert!(versioned.starts_with("cfg-if-1."));
}

/// **M3c** — `--verify` against a shared-sync vendor checks both primary
/// and sync'd Cargo.lock. Hand-corrupt one of the sync'd lockfiles → the
/// verify step should flag it.
#[test]
#[ignore] // network
fn verify_of_shared_vendor_checks_all_sync_lockfiles() {
    let work = tempfile::TempDir::new().unwrap();

    let ws_a = work.path().join("ws_a");
    let ws_b = work.path().join("ws_b");
    std::fs::create_dir_all(&ws_a).unwrap();
    std::fs::create_dir_all(&ws_b).unwrap();

    for (ws, pkg, dep) in [
        (ws_a.as_path(), "pkg_a", "cfg-if"),
        (ws_b.as_path(), "pkg_b", "once_cell"),
    ] {
        std::fs::write(
            ws.join("Cargo.toml"),
            format!(
                r#"[package]
name = "{pkg}"
version = "0.1.0"
edition = "2021"
publish = false

[workspace]

[lib]
path = "lib.rs"

[dependencies]
{dep} = "1"
"#
            ),
        )
        .unwrap();
        std::fs::write(ws.join("lib.rs"), "").unwrap();
        common::git_init(ws);
    }

    let shared_vendor = work.path().join("vendor");

    revendor_cmd()
        .arg("revendor")
        .arg("--manifest-path")
        .arg(ws_a.join("Cargo.toml"))
        .arg("--sync")
        .arg(ws_b.join("Cargo.toml"))
        .arg("--output")
        .arg(&shared_vendor)
        .assert()
        .success();

    // Verify passes on the clean post-vendor state.
    revendor_cmd()
        .arg("revendor")
        .arg("--verify")
        .arg("--manifest-path")
        .arg(ws_a.join("Cargo.toml"))
        .arg("--sync")
        .arg(ws_b.join("Cargo.toml"))
        .arg("--output")
        .arg(&shared_vendor)
        .assert()
        .success();

    // Corrupt ws_b's Cargo.lock by pinning once_cell to a bogus version.
    let lock_b_path = ws_b.join("Cargo.lock");
    let lock_b = std::fs::read_to_string(&lock_b_path).unwrap();
    let needle = "name = \"once_cell\"\nversion = \"";
    let start = lock_b.find(needle).expect("once_cell entry missing");
    let version_start = start + needle.len();
    let version_end = version_start + lock_b[version_start..].find('"').unwrap();
    let mut corrupted = lock_b.clone();
    corrupted.replace_range(version_start..version_end, "99.99.99");
    std::fs::write(&lock_b_path, corrupted).unwrap();

    let assert = revendor_cmd()
        .arg("revendor")
        .arg("--verify")
        .arg("--manifest-path")
        .arg(ws_a.join("Cargo.toml"))
        .arg("--sync")
        .arg(ws_b.join("Cargo.toml"))
        .arg("--output")
        .arg(&shared_vendor)
        .assert()
        .failure();

    let stderr = String::from_utf8_lossy(&assert.get_output().stderr).to_string();
    assert!(
        stderr.contains("once_cell") && stderr.contains("99.99.99"),
        "expected mismatch error to name once_cell + bogus version, got:\n{stderr}"
    );
}
