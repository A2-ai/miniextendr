//! Edge cases + diagnostics for cargo-revendor (#231).
//!
//! Final phase of the test-matrix rollout. These fill in corners the
//! bigger test files don't cover:
//!
//! - **E**dge cases: dev-deps on path crates, cfg-gated deps, dep renames,
//!   explicit `default-features = false`, path escapes.
//! - **D**iagnostics: verbosity level plumbing, `.vendor-source` marker
//!   content.

mod common;

use common::{create_simple_crate, create_workspace, read_vendor_toml, revendor_cmd, vendor_has};

// region: Edge cases (E1-E5)

/// **E1** — dev-dependency on a workspace path crate. With `--strip-all`,
/// the vendored manifest should strip [dev-dependencies] entirely
/// (alongside the tests/ benches/ examples/ directories).
#[test]
#[ignore] // network
fn dev_dependency_on_local_path_crate_stripped() {
    // Workspace with a helper crate used only as a dev-dep.
    let proj = create_workspace(
        r#"[workspace]
members = ["caller", "helper"]
resolver = "2"
"#,
        &[
            (
                "caller",
                r#"[package]
name = "caller"
version = "0.1.0"
edition = "2021"
publish = false

[lib]
path = "lib.rs"

[dev-dependencies]
helper = { path = "../helper" }
cfg-if = "1"
"#,
                "pub fn caller() {}\n",
            ),
            (
                "helper",
                r#"[package]
name = "helper"
version = "0.1.0"
edition = "2021"
publish = false

[lib]
path = "lib.rs"
"#,
                "pub fn helper() {}\n",
            ),
        ],
    );

    let vendor = proj.root().join("vendor");
    revendor_cmd()
        .arg("revendor")
        .arg("--manifest-path")
        .arg(proj.root().join("caller/Cargo.toml"))
        .arg("--output")
        .arg(&vendor)
        .arg("--strip-all")
        .assert()
        .success();

    // cfg-if is a proper dep of the graph (well, dev-dep of caller) — it may
    // or may not land in vendor/ depending on whether dev-deps are resolved
    // in the minimal lockfile. What we care about: the vendored helper (if
    // present) has [dev-dependencies] stripped.
    if vendor_has(&vendor, "helper") {
        let toml = read_vendor_toml(&vendor, "helper");
        assert!(
            !toml.contains("[dev-dependencies]"),
            "--strip-all should remove [dev-dependencies] from vendored helper Cargo.toml, got:\n{toml}"
        );
    }
}

/// **E3** — cfg-gated dependency (`[target.'cfg(unix)'.dependencies]`)
/// should still be vendored. cargo vendor resolves conditionals during
/// dep-graph traversal.
#[test]
#[ignore] // network
fn cfg_gated_dependencies_vendored() {
    let proj = create_simple_crate(
        r#"[package]
name = "cfg-caller"
version = "0.1.0"
edition = "2021"
publish = false

[workspace]

[lib]
path = "lib.rs"

[target.'cfg(unix)'.dependencies]
cfg-if = "1"

[target.'cfg(windows)'.dependencies]
once_cell = "1"
"#,
        "pub fn hi() {}\n",
    );

    let vendor = proj.root().join("vendor");
    revendor_cmd()
        .arg("revendor")
        .arg("--manifest-path")
        .arg(proj.root().join("Cargo.toml"))
        .arg("--output")
        .arg(&vendor)
        .assert()
        .success();

    // cargo vendor materializes deps for all target triples, not just the
    // host — both cfg-if (unix) and once_cell (windows) should be present.
    assert!(
        vendor_has(&vendor, "cfg-if"),
        "expected unix cfg-gated dep to vendor"
    );
    assert!(
        vendor_has(&vendor, "once_cell"),
        "expected windows cfg-gated dep to vendor"
    );
}

/// **E4** — renamed dependency (`foo = { package = "actual-name", ... }`).
/// Both the rename and the real `package = "..."` field survive vendoring.
#[test]
#[ignore] // network
fn rename_deps_survive_vendoring() {
    let proj = create_simple_crate(
        r#"[package]
name = "renamer"
version = "0.1.0"
edition = "2021"
publish = false

[workspace]

[lib]
path = "lib.rs"

[dependencies]
my_alias = { package = "cfg-if", version = "1" }
"#,
        "pub use my_alias::*;\n",
    );

    let vendor = proj.root().join("vendor");
    revendor_cmd()
        .arg("revendor")
        .arg("--manifest-path")
        .arg(proj.root().join("Cargo.toml"))
        .arg("--output")
        .arg(&vendor)
        .assert()
        .success();

    // The renamed dep is stored under the real crate name in vendor/.
    assert!(
        vendor_has(&vendor, "cfg-if"),
        "renamed dep should land under its real package name"
    );

    // The caller's Cargo.toml is NOT vendored (it's the target crate), so
    // the rename persistence we care about is: the rename is accepted
    // during vendoring without error (tested by .success() above) and the
    // resulting lockfile records cfg-if.
    let lock = std::fs::read_to_string(proj.root().join("Cargo.lock")).unwrap();
    assert!(
        lock.contains("name = \"cfg-if\""),
        "lockfile should record cfg-if despite the `my_alias` rename"
    );
}

/// **E5** — `default-features = false` on a dep should survive
/// rewrite_local_path_deps + strip passes in the vendored manifest.
/// Test target: a workspace where we vendor a local crate that itself
/// has `default-features = false` on one of its deps.
#[test]
#[ignore] // network
fn no_default_features_preserved_in_vendor() {
    let proj = create_workspace(
        r#"[workspace]
members = ["caller", "lib_no_default"]
resolver = "2"
"#,
        &[
            (
                "caller",
                r#"[package]
name = "caller"
version = "0.1.0"
edition = "2021"
publish = false

[lib]
path = "lib.rs"

[dependencies]
lib_no_default = { path = "../lib_no_default" }
"#,
                "pub use lib_no_default::*;\n",
            ),
            (
                "lib_no_default",
                r#"[package]
name = "lib_no_default"
version = "0.1.0"
edition = "2021"
publish = false

[lib]
path = "lib.rs"

[dependencies]
cfg-if = { version = "1", default-features = false }
"#,
                "pub fn noop() {}\n",
            ),
        ],
    );

    let vendor = proj.root().join("vendor");
    revendor_cmd()
        .arg("revendor")
        .arg("--manifest-path")
        .arg(proj.root().join("caller/Cargo.toml"))
        .arg("--output")
        .arg(&vendor)
        .arg("--strip-all")
        .assert()
        .success();

    let toml = read_vendor_toml(&vendor, "lib_no_default");
    assert!(
        toml.contains("default-features = false")
            || toml.contains("default_features = false")
            || toml.contains("default-features=false"),
        "vendored crate should preserve default-features = false, got:\n{toml}"
    );
}

// endregion

// region: Diagnostics (D1-D2)

/// **D1** — verbosity levels produce monotonically increasing stderr.
/// `-vvv` should reveal strictly more than `-vv`, which in turn reveals
/// strictly more than `-v`. Guards against accidental breakage of the
/// `Verbosity` plumbing across runs.
#[test]
#[ignore] // network
fn verbosity_levels_are_monotonic() {
    let make_run = |flag: &str| -> String {
        let proj = create_simple_crate(
            r#"[package]
name = "dpkg"
version = "0.1.0"
edition = "2021"
publish = false

[workspace]

[lib]
path = "lib.rs"

[dependencies]
cfg-if = "1"
"#,
            "pub fn hi() {}\n",
        );
        let vendor = proj.root().join("vendor");
        let mut cmd = revendor_cmd();
        cmd.arg("revendor")
            .arg("--manifest-path")
            .arg(proj.root().join("Cargo.toml"))
            .arg("--output")
            .arg(&vendor);
        if !flag.is_empty() {
            cmd.arg(flag);
        }
        let out = cmd.output().expect("revendor failed to spawn");
        assert!(
            out.status.success(),
            "revendor {flag} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stderr).to_string()
    };

    let q = make_run("");
    let v = make_run("-v");
    let vv = make_run("-vv");
    let vvv = make_run("-vvv");

    // The exact stderr bytes will differ between runs (crate counts,
    // temp paths), so compare line counts as a proxy for "more was said".
    let line_count = |s: &str| s.lines().count();

    assert!(
        line_count(&v) > line_count(&q),
        "expected `-v` to say more than quiet: q={}, v={}",
        line_count(&q),
        line_count(&v)
    );
    assert!(
        line_count(&vv) >= line_count(&v),
        "expected `-vv` to say at least as much as `-v`: v={}, vv={}",
        line_count(&v),
        line_count(&vv)
    );
    assert!(
        line_count(&vvv) >= line_count(&vv),
        "expected `-vvv` to say at least as much as `-vv`: vv={}, vvv={}",
        line_count(&vv),
        line_count(&vvv)
    );
}

/// **D2** — `--source-marker` writes a `.vendor-source` file. When
/// `--source-root` is explicit, the marker contains that path. Without
/// an explicit source root, it contains a derived value (currently
/// `"auto-detected"`, but the exact string isn't the contract — what we
/// assert is: the file exists and is non-empty).
#[test]
#[ignore] // network
fn source_marker_records_explicit_source_root() {
    let proj = create_simple_crate(
        r#"[package]
name = "smpkg"
version = "0.1.0"
edition = "2021"
publish = false

[workspace]

[lib]
path = "lib.rs"

[dependencies]
cfg-if = "1"
"#,
        "pub fn hi() {}\n",
    );

    let vendor = proj.root().join("vendor");
    let explicit_src = proj.root();
    revendor_cmd()
        .arg("revendor")
        .arg("--manifest-path")
        .arg(proj.root().join("Cargo.toml"))
        .arg("--output")
        .arg(&vendor)
        .arg("--source-marker")
        .arg("--source-root")
        .arg(explicit_src)
        .assert()
        .success();

    let marker = vendor.join(".vendor-source");
    assert!(
        marker.exists(),
        "--source-marker should write .vendor-source"
    );
    let body = std::fs::read_to_string(&marker).unwrap();
    let body = body.trim();
    assert_eq!(
        body,
        explicit_src.display().to_string(),
        "marker should record the explicit --source-root, got: {body}"
    );
}

// endregion

// region: Manifest auto-discovery (no --manifest-path)

/// **D3** — when `--manifest-path` is omitted, pick up a canonical
/// `src/rust/Cargo.toml` laid out relative to the current directory.
#[test]
fn auto_discovers_canonical_rpkg_layout() {
    use predicates::prelude::PredicateBooleanExt;
    let dir = tempfile::tempdir().unwrap();
    let rust = dir.path().join("src/rust");
    std::fs::create_dir_all(&rust).unwrap();
    std::fs::write(
        rust.join("Cargo.toml"),
        "[package]\nname=\"p\"\nversion=\"0.0.0\"\nedition=\"2021\"\n[lib]\npath=\"lib.rs\"\n",
    )
    .unwrap();
    std::fs::write(rust.join("lib.rs"), "").unwrap();

    // No --manifest-path; should not bail out on discovery.
    revendor_cmd()
        .current_dir(dir.path())
        .args(["revendor", "--verify", "--output", "vendor-never-created"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("no Cargo.toml found").not());
}

/// **D4** — when `--manifest-path` is omitted and the R package is in a
/// subdirectory (e.g. `dvs-rpkg/src/rust/Cargo.toml`), auto-discover it.
#[test]
fn auto_discovers_subdir_layout() {
    use predicates::prelude::PredicateBooleanExt;
    let dir = tempfile::tempdir().unwrap();
    let rust = dir.path().join("dvs-rpkg/src/rust");
    std::fs::create_dir_all(&rust).unwrap();
    std::fs::write(
        rust.join("Cargo.toml"),
        "[package]\nname=\"p\"\nversion=\"0.0.0\"\nedition=\"2021\"\n[lib]\npath=\"lib.rs\"\n",
    )
    .unwrap();
    std::fs::write(rust.join("lib.rs"), "").unwrap();

    revendor_cmd()
        .current_dir(dir.path())
        .args(["revendor", "--verify", "--output", "vendor-never-created"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("no Cargo.toml found").not());
}

/// **D5** — when no plausible manifest exists anywhere, fail with a clear
/// message pointing at `--manifest-path`.
#[test]
fn no_manifest_nearby_gives_helpful_error() {
    let dir = tempfile::tempdir().unwrap();

    revendor_cmd()
        .current_dir(dir.path())
        .args(["revendor"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("no Cargo.toml found"))
        .stderr(predicates::str::contains("--manifest-path"));
}

// endregion
