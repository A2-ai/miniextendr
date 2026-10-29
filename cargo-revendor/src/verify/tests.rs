use super::*;
use std::fs;
use tempfile::TempDir;

fn write_lock(dir: &Path, body: &str) -> std::path::PathBuf {
    let p = dir.join("Cargo.lock");
    fs::write(&p, body).unwrap();
    p
}

fn write_vendored(vendor: &Path, name: &str, version: &str) {
    let d = vendor.join(name);
    fs::create_dir_all(&d).unwrap();
    fs::write(
        d.join("Cargo.toml"),
        format!("[package]\nname = \"{name}\"\nversion = \"{version}\"\n"),
    )
    .unwrap();
}

#[test]
fn lock_matches_vendor_passes_when_all_present() {
    let tmp = TempDir::new().unwrap();
    let vendor = tmp.path().join("vendor");
    fs::create_dir_all(&vendor).unwrap();
    write_vendored(&vendor, "rayon", "1.12.0");
    write_vendored(&vendor, "anyhow", "1.0.80");

    let lockfile = write_lock(
        tmp.path(),
        r#"version = 4

[[package]]
name = "rayon"
version = "1.12.0"
source = "registry+https://github.com/rust-lang/crates.io-index"

[[package]]
name = "anyhow"
version = "1.0.80"
source = "registry+https://github.com/rust-lang/crates.io-index"

[[package]]
name = "my-local"
version = "0.1.0"
"#,
    );

    verify_lock_matches_vendor(&lockfile, &vendor).unwrap();
}

#[test]
fn lock_matches_vendor_rejects_version_mismatch() {
    // The exact failure shape called out in #157:
    //   locked to rayon 1.12.0, but vendor/ only has rayon 1.11.0.
    let tmp = TempDir::new().unwrap();
    let vendor = tmp.path().join("vendor");
    fs::create_dir_all(&vendor).unwrap();
    write_vendored(&vendor, "rayon", "1.11.0");

    let lockfile = write_lock(
        tmp.path(),
        r#"version = 4

[[package]]
name = "rayon"
version = "1.12.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
"#,
    );

    let err = verify_lock_matches_vendor(&lockfile, &vendor).unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("rayon") && msg.contains("1.12.0") && msg.contains("1.11.0"),
        "unexpected error: {msg}"
    );
}

#[test]
fn lock_matches_vendor_rejects_missing_crate() {
    let tmp = TempDir::new().unwrap();
    let vendor = tmp.path().join("vendor");
    fs::create_dir_all(&vendor).unwrap();

    let lockfile = write_lock(
        tmp.path(),
        r#"version = 4

[[package]]
name = "rayon"
version = "1.12.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
"#,
    );

    let err = verify_lock_matches_vendor(&lockfile, &vendor).unwrap_err();
    assert!(format!("{err}").contains("rayon"));
}

#[test]
fn lock_matches_vendor_accepts_versioned_dir_layout() {
    // When two versions of the same crate are locked, cargo vendor uses
    // `vendor/<name>-<version>/` for each.
    let tmp = TempDir::new().unwrap();
    let vendor = tmp.path().join("vendor");
    fs::create_dir_all(&vendor).unwrap();
    write_vendored(&vendor, "ahash-0.7.8", "0.7.8");
    write_vendored(&vendor, "ahash-0.8.11", "0.8.11");

    // Fix the nested name/version in the ahash manifests.
    fs::write(
        vendor.join("ahash-0.7.8").join("Cargo.toml"),
        "[package]\nname = \"ahash\"\nversion = \"0.7.8\"\n",
    )
    .unwrap();
    fs::write(
        vendor.join("ahash-0.8.11").join("Cargo.toml"),
        "[package]\nname = \"ahash\"\nversion = \"0.8.11\"\n",
    )
    .unwrap();

    let lockfile = write_lock(
        tmp.path(),
        r#"version = 4

[[package]]
name = "ahash"
version = "0.7.8"
source = "registry+https://github.com/rust-lang/crates.io-index"

[[package]]
name = "ahash"
version = "0.8.11"
source = "registry+https://github.com/rust-lang/crates.io-index"
"#,
    );

    verify_lock_matches_vendor(&lockfile, &vendor).unwrap();
}
