//! End-to-end tests for `--verify`, `--freeze`, and `--compress` (#228).
//!
//! These exercise cargo-revendor through the CLI with realistic inputs, vs
//! the existing unit tests in `verify.rs` / `vendor.rs` which operate on
//! in-memory fixtures. Each test sets up a tiny crate, invokes cargo-revendor
//! once to produce a vendor tree / tarball, then a second invocation that
//! either verifies, frozen-builds, or extract-diffs the result.
//!
//! Gated behind `#[ignore]` because the first invocation is a full
//! `cargo vendor` (network-touching in general).

mod common;

use common::{
    create_simple_crate, diff_trees, extract_tarball, revendor_cmd, vendor_dir_for, TreeDiff,
};

// region: --verify end-to-end (V1-V5)

/// **V1** — full vendor flow produces a tree that `--verify` accepts.
#[test]
#[ignore] // network
fn verify_clean_after_vendor() {
    let proj = make_simple_project();
    let vendor = proj.root().join("vendor");

    // Step 1: vendor + compress.
    let tarball = proj.root().join("vendor.tar.xz");
    revendor_cmd()
        .arg("revendor")
        .arg("--manifest-path")
        .arg(proj.root().join("Cargo.toml"))
        .arg("--output")
        .arg(&vendor)
        .arg("--compress")
        .arg(&tarball)
        .assert()
        .success();

    // Step 2: verify both Lock↔vendor and vendor↔tarball.
    revendor_cmd()
        .arg("revendor")
        .arg("--verify")
        .arg("--manifest-path")
        .arg(proj.root().join("Cargo.toml"))
        .arg("--output")
        .arg(&vendor)
        .arg("--compress")
        .arg(&tarball)
        .assert()
        .success();
}

/// **V2** — hand-edited Cargo.lock pinning a version not present in
/// `vendor/` should fail verify with a clear error. Shape of failure
/// matches the #157 motivating example.
#[test]
#[ignore] // network
fn verify_catches_lock_vendor_mismatch() {
    let proj = make_simple_project();
    let vendor = proj.root().join("vendor");

    revendor_cmd()
        .arg("revendor")
        .arg("--manifest-path")
        .arg(proj.root().join("Cargo.toml"))
        .arg("--output")
        .arg(&vendor)
        .assert()
        .success();

    // Corrupt Cargo.lock: find the cfg-if entry (version changes over time
    // as cfg-if releases) and pin it to a nonexistent version.
    let lock_path = proj.root().join("Cargo.lock");
    let lock = std::fs::read_to_string(&lock_path).unwrap();
    // Look for: `name = "cfg-if"\nversion = "1.x.y"`. Keep things simple —
    // locate the line after `name = "cfg-if"` and rewrite its value.
    let needle = "name = \"cfg-if\"\nversion = \"";
    let start = lock.find(needle).expect("cfg-if entry not in Cargo.lock");
    let version_start = start + needle.len();
    let version_end = version_start + lock[version_start..].find('"').unwrap();
    let mut corrupted = lock.clone();
    corrupted.replace_range(version_start..version_end, "99.99.99");
    assert_ne!(lock, corrupted, "cfg-if version substitution didn't fire");
    std::fs::write(&lock_path, corrupted).unwrap();

    let assert = revendor_cmd()
        .arg("revendor")
        .arg("--verify")
        .arg("--manifest-path")
        .arg(proj.root().join("Cargo.toml"))
        .arg("--output")
        .arg(&vendor)
        .assert()
        .failure();

    let stderr = String::from_utf8_lossy(&assert.get_output().stderr).to_string();
    assert!(
        stderr.contains("cfg-if") && stderr.contains("99.99.99"),
        "expected mismatch error to name cfg-if + 99.99.99, got:\n{stderr}"
    );
}

/// **V3** — tarball on disk that no longer reflects the current vendor/
/// state. verify should fail at the tarball↔vendor step, pointing at the
/// specific file(s).
#[test]
#[ignore] // network
fn verify_catches_stale_tarball() {
    let proj = make_simple_project();
    let vendor = proj.root().join("vendor");
    let tarball = proj.root().join("vendor.tar.xz");

    revendor_cmd()
        .arg("revendor")
        .arg("--manifest-path")
        .arg(proj.root().join("Cargo.toml"))
        .arg("--output")
        .arg(&vendor)
        .arg("--compress")
        .arg(&tarball)
        .assert()
        .success();

    // Modify a vendored file after the tarball was built.
    let target = vendor_dir_for(&vendor, "cfg-if", None).join("Cargo.toml");
    let content = std::fs::read_to_string(&target).unwrap();
    std::fs::write(&target, format!("{content}\n# drift\n")).unwrap();

    let assert = revendor_cmd()
        .arg("revendor")
        .arg("--verify")
        .arg("--manifest-path")
        .arg(proj.root().join("Cargo.toml"))
        .arg("--output")
        .arg(&vendor)
        .arg("--compress")
        .arg(&tarball)
        .assert()
        .failure();

    let stderr = String::from_utf8_lossy(&assert.get_output().stderr).to_string();
    assert!(
        stderr.contains("out of sync") || stderr.contains("differing content"),
        "expected tarball-drift error, got:\n{stderr}"
    );
}

/// **V4** — regression for #218. `.revendor-cache` is written at the top
/// of `vendor/` AFTER `--compress`, so it's never in the tarball by design.
/// verify must not flag it as a diff.
#[test]
#[ignore] // network
fn verify_ignores_revendor_cache_byproduct() {
    let proj = make_simple_project();
    let vendor = proj.root().join("vendor");
    let tarball = proj.root().join("vendor.tar.xz");

    revendor_cmd()
        .arg("revendor")
        .arg("--manifest-path")
        .arg(proj.root().join("Cargo.toml"))
        .arg("--output")
        .arg(&vendor)
        .arg("--compress")
        .arg(&tarball)
        .assert()
        .success();

    // The cache file MUST exist after a vendor run, otherwise this test is
    // a tautology.
    assert!(
        vendor.join(".revendor-cache").exists(),
        "expected vendor/.revendor-cache to be written after vendor run"
    );

    revendor_cmd()
        .arg("revendor")
        .arg("--verify")
        .arg("--manifest-path")
        .arg(proj.root().join("Cargo.toml"))
        .arg("--output")
        .arg(&vendor)
        .arg("--compress")
        .arg(&tarball)
        .assert()
        .success();
}

/// **V5** — running `--verify` without a prior vendor tree should fail with
/// a clean error, not try to re-vendor.
#[test]
fn verify_only_errors_without_vendor_tree() {
    let proj = make_simple_project();
    let vendor = proj.root().join("vendor-does-not-exist");

    let assert = revendor_cmd()
        .arg("revendor")
        .arg("--verify")
        .arg("--manifest-path")
        .arg(proj.root().join("Cargo.toml"))
        .arg("--output")
        .arg(&vendor)
        .assert()
        .failure();

    // Don't care about the exact message — just that it didn't silently try
    // to vendor or succeed on an empty dir.
    assert!(
        !vendor.exists(),
        "--verify must not create the vendor directory"
    );
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr).to_string();
    assert!(
        stderr.contains("vendor") || stderr.contains("not"),
        "expected an error mentioning the missing vendor, got:\n{stderr}"
    );
}

// endregion

// region: --freeze end-to-end (F1-F3)

/// **F1** — after `--freeze`, a `[patch.crates-io]` block is added with
/// path deps under `vendor/`. Crates-io entries in `[dependencies]` keep
/// their original form (cargo-revendor rewrites local-name deps only);
/// the patch block is what redirects resolution at build time.
#[test]
#[ignore] // network
fn freeze_rewrites_manifest_to_vendor_paths() {
    let proj = make_simple_project();
    let vendor = proj.root().join("vendor");

    revendor_cmd()
        .arg("revendor")
        .arg("--manifest-path")
        .arg(proj.root().join("Cargo.toml"))
        .arg("--output")
        .arg(&vendor)
        .arg("--freeze")
        .assert()
        .success();

    let frozen = std::fs::read_to_string(proj.root().join("Cargo.toml")).unwrap();
    // For a pure crates-io dep like cfg-if, --freeze adds a
    // [patch.crates-io] entry rather than rewriting [dependencies].
    // We confirm the manifest still resolves offline in F2; here we just
    // assert the patch section exists and mentions vendor paths.
    // Note: cargo-revendor's [patch.crates-io] entries only appear when there
    // are local path dep crates to patch. For the single-crate case, freeze
    // may leave [dependencies] alone and rely on .cargo/config.toml source
    // replacement. Accept either shape.
    let has_vendor_path = frozen.contains("path = \"vendor/")
        || frozen.contains("path = \"./vendor/")
        || frozen.contains("path = \"../vendor/");
    let config_toml_exists = proj
        .root()
        .join("vendor")
        .join(".cargo-config.toml")
        .exists()
        || proj.root().join(".cargo").join("config.toml").exists();
    assert!(
        has_vendor_path || config_toml_exists,
        "expected either a vendor/ path dep in Cargo.toml or a .cargo/config.toml \
         source replacement. Manifest:\n{frozen}"
    );
}

/// **F2** — after freeze, `cargo build --offline` succeeds: the resulting
/// manifest is self-contained.
#[test]
#[ignore] // network
fn freeze_builds_offline() {
    let proj = make_simple_project();
    let vendor = proj.root().join("vendor");

    revendor_cmd()
        .arg("revendor")
        .arg("--manifest-path")
        .arg(proj.root().join("Cargo.toml"))
        .arg("--output")
        .arg(&vendor)
        .arg("--freeze")
        .assert()
        .success();

    let out = std::process::Command::new("cargo")
        .args(["build", "--offline", "--manifest-path"])
        .arg(proj.root().join("Cargo.toml"))
        .env("CARGO_NET_OFFLINE", "true")
        .output()
        .expect("cargo build failed to spawn");
    assert!(
        out.status.success(),
        "cargo build --offline after freeze failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
}

/// **F3** — `[patch.crates-io]` entries emitted by `--freeze` are
/// alphabetically sorted. Regression for #206 at integration level;
/// the unit test in vendor.rs covered the sort function directly.
///
/// Uses the rpkg-style monorepo layout: outer workspace with two local
/// library crates, plus a separate leaf crate that vendors them (its own
/// `[workspace]` declaration prevents cargo from seeing the source crates
/// as duplicates of their vendored copies).
#[test]
#[ignore] // network
fn freeze_sorts_patch_crates_io_deterministically() {
    let work = tempfile::TempDir::new().unwrap();
    let root = work.path().join("ws");
    std::fs::create_dir_all(root.join("alpha/src")).unwrap();
    std::fs::create_dir_all(root.join("omega/src")).unwrap();
    std::fs::create_dir_all(root.join("leaf/src")).unwrap();

    std::fs::write(
        root.join("Cargo.toml"),
        r#"[workspace]
members = ["alpha", "omega"]
resolver = "2"
"#,
    )
    .unwrap();
    std::fs::write(
        root.join("alpha/Cargo.toml"),
        r#"[package]
name = "alpha"
version = "0.1.0"
edition = "2021"
publish = false
"#,
    )
    .unwrap();
    std::fs::write(root.join("alpha/src/lib.rs"), "pub fn alpha() {}\n").unwrap();
    std::fs::write(
        root.join("omega/Cargo.toml"),
        r#"[package]
name = "omega"
version = "0.1.0"
edition = "2021"
publish = false
"#,
    )
    .unwrap();
    std::fs::write(root.join("omega/src/lib.rs"), "pub fn omega() {}\n").unwrap();
    // Leaf crate: standalone workspace, depends on the two siblings. This is
    // the rpkg shape.
    std::fs::write(
        root.join("leaf/Cargo.toml"),
        r#"[package]
name = "leaf"
version = "0.1.0"
edition = "2021"
publish = false

[workspace]

[lib]
path = "src/lib.rs"

[dependencies]
alpha = { path = "../alpha" }
omega = { path = "../omega" }
cfg-if = "1"
"#,
    )
    .unwrap();
    std::fs::write(
        root.join("leaf/src/lib.rs"),
        "pub use alpha::*; pub use omega::*;\n",
    )
    .unwrap();
    common::git_init(&root);

    let vendor = root.join("leaf/vendor");
    revendor_cmd()
        .arg("revendor")
        .arg("--manifest-path")
        .arg(root.join("leaf/Cargo.toml"))
        .arg("--output")
        .arg(&vendor)
        .arg("--source-root")
        .arg(&root)
        .arg("--freeze")
        .assert()
        .success();

    let frozen = std::fs::read_to_string(root.join("leaf/Cargo.toml")).unwrap();
    let patch = frozen
        .split("[patch.crates-io]")
        .nth(1)
        .expect("frozen manifest should have [patch.crates-io]");
    let keys: Vec<String> = patch
        .lines()
        .take_while(|l| !l.trim().starts_with('[') || l.trim().is_empty())
        .filter_map(|l| {
            let l = l.trim();
            if l.is_empty() || l.starts_with('#') {
                return None;
            }
            l.split('=').next().map(|k| k.trim().to_string())
        })
        .filter(|k| !k.is_empty())
        .collect();
    assert!(
        keys.len() >= 2,
        "expected ≥2 [patch.crates-io] entries, got {keys:?}"
    );
    let mut sorted = keys.clone();
    sorted.sort();
    assert_eq!(
        keys, sorted,
        "[patch.crates-io] entries should be alphabetically sorted, got: {keys:?}"
    );
}

// endregion

// region: --compress round-trip (C1-C3)

/// **C1** — compressing `vendor/` and extracting the resulting tarball
/// into a fresh dir reproduces `vendor/` bit-for-bit (modulo the
/// `.revendor-cache` byproduct that's written after compress).
#[test]
#[ignore] // network
fn compress_roundtrip_matches_vendor() {
    let proj = make_simple_project();
    let vendor = proj.root().join("vendor");
    let tarball = proj.root().join("vendor.tar.xz");

    revendor_cmd()
        .arg("revendor")
        .arg("--manifest-path")
        .arg(proj.root().join("Cargo.toml"))
        .arg("--output")
        .arg(&vendor)
        .arg("--compress")
        .arg(&tarball)
        .assert()
        .success();

    // Extract to a fresh dir.
    let extract_dst = tempfile::TempDir::new().unwrap();
    extract_tarball(&tarball, extract_dst.path());

    // Tarball contains a single top-level `vendor/` dir.
    let extracted_root = std::fs::read_dir(extract_dst.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .find(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .map(|e| e.path())
        .expect("tarball should have a top-level dir");

    let mut diffs = diff_trees(&vendor, &extracted_root);
    // Filter out `.revendor-cache` — written after compress by design.
    diffs.retain(|d| !matches!(d,
        TreeDiff::OnlyInA(p) | TreeDiff::OnlyInB(p) | TreeDiff::ContentDiff(p)
            if p == ".revendor-cache"
    ));
    assert!(
        diffs.is_empty(),
        "vendor/ and tarball should match bit-for-bit (modulo .revendor-cache), diffs:\n{diffs:#?}"
    );
}

/// **C2** — `--blank-md` zeroes every `.md` file's content in the
/// compressed tarball; without the flag, contents are preserved.
#[test]
#[ignore] // network
fn compress_blank_md_zeroes_markdown() {
    fn read_first_readme(tarball: &std::path::Path) -> Vec<u8> {
        let extract_dst = tempfile::TempDir::new().unwrap();
        extract_tarball(tarball, extract_dst.path());
        // Find the first README.md anywhere in the extracted tree.
        for entry in walkdir::WalkDir::new(extract_dst.path()) {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            if entry.file_name() == "README.md" {
                return std::fs::read(entry.path()).unwrap();
            }
        }
        panic!("no README.md found in extracted tarball");
    }

    // Blanked run.
    let blanked_proj = make_simple_project();
    let blanked_tarball = blanked_proj.root().join("vendor.tar.xz");
    revendor_cmd()
        .arg("revendor")
        .arg("--manifest-path")
        .arg(blanked_proj.root().join("Cargo.toml"))
        .arg("--output")
        .arg(blanked_proj.root().join("vendor"))
        .arg("--compress")
        .arg(&blanked_tarball)
        .arg("--blank-md")
        .assert()
        .success();
    let blanked_content = read_first_readme(&blanked_tarball);

    // Preserved run (no --blank-md).
    let preserved_proj = make_simple_project();
    let preserved_tarball = preserved_proj.root().join("vendor.tar.xz");
    revendor_cmd()
        .arg("revendor")
        .arg("--manifest-path")
        .arg(preserved_proj.root().join("Cargo.toml"))
        .arg("--output")
        .arg(preserved_proj.root().join("vendor"))
        .arg("--compress")
        .arg(&preserved_tarball)
        .assert()
        .success();
    let preserved_content = read_first_readme(&preserved_tarball);

    assert!(
        blanked_content.is_empty(),
        "--blank-md should produce an empty README.md, got {} bytes",
        blanked_content.len()
    );
    assert!(
        !preserved_content.is_empty(),
        "without --blank-md the README.md should retain its content"
    );
}

/// **C3** — the produced tarball has no macOS xattr leakage. On macOS,
/// bsdtar will preserve `com.apple.*` xattrs unless we pass `--no-xattrs`
/// and set `COPYFILE_DISABLE=1`. Check there are no `._*` AppleDouble
/// entries and no `LIBARCHIVE.xattr.*` PAX headers.
#[test]
#[ignore] // network
fn compress_suppresses_macos_xattrs() {
    let proj = make_simple_project();
    let vendor = proj.root().join("vendor");
    let tarball = proj.root().join("vendor.tar.xz");

    revendor_cmd()
        .arg("revendor")
        .arg("--manifest-path")
        .arg(proj.root().join("Cargo.toml"))
        .arg("--output")
        .arg(&vendor)
        .arg("--compress")
        .arg(&tarball)
        .assert()
        .success();

    // `tar -tJf` lists contents; scan the names.
    let out = std::process::Command::new("tar")
        .arg("-tJf")
        .arg(&tarball)
        .output()
        .expect("tar -t failed");
    assert!(out.status.success(), "tar -t failed");
    let listing = String::from_utf8_lossy(&out.stdout);

    let appledouble: Vec<&str> = listing
        .lines()
        .filter(|l| l.split('/').any(|seg| seg.starts_with("._")))
        .collect();
    assert!(
        appledouble.is_empty(),
        "tarball should not contain AppleDouble (._) entries, found:\n{appledouble:#?}"
    );

    // PAX headers with xattr metadata appear as `./PaxHeader` or `@PaxHeader`
    // entries containing LIBARCHIVE.xattr keys in their body. Listing alone
    // won't show the body, but the presence of PaxHeader entries is itself
    // a signal on macOS.
    let pax: Vec<&str> = listing
        .lines()
        .filter(|l| l.contains("LIBARCHIVE.xattr") || l.contains("SCHILY.xattr"))
        .collect();
    assert!(
        pax.is_empty(),
        "tarball listing mentions xattr-carrying headers: {pax:#?}"
    );
}

// endregion

// region: helpers

/// The single-crate project shape used by most verify/freeze/compress
/// tests: depends on `cfg-if`, a small well-known crate with a README and
/// no build script.
fn make_simple_project() -> common::TestProject {
    create_simple_crate(
        r#"[package]
name = "vf-test"
version = "0.1.0"
edition = "2021"
publish = false

[workspace]

[lib]
path = "lib.rs"

[dependencies]
cfg-if = "1"
"#,
        "pub fn hello() {}\n",
    )
}

// endregion
