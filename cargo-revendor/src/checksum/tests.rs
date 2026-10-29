use super::*;
use std::fs;
use tempfile::TempDir;

fn make_vendor_crate(dir: &TempDir, name: &str, files: &[(&str, &str)]) -> std::path::PathBuf {
    let crate_dir = dir.path().join(name);
    fs::create_dir_all(&crate_dir).unwrap();
    for (rel, content) in files {
        let path = crate_dir.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&path, content).unwrap();
    }
    crate_dir
}

fn write_checksum_json(crate_dir: &std::path::Path, package: Option<&str>, files: &[(&str, &str)]) {
    let files_map: serde_json::Map<String, serde_json::Value> = files
        .iter()
        .map(|(k, v)| (k.to_string(), serde_json::Value::String(v.to_string())))
        .collect();
    let json = serde_json::json!({
        "package": package,
        "files": files_map,
    });
    fs::write(crate_dir.join(".cargo-checksum.json"), json.to_string()).unwrap();
}

fn read_checksum_json(crate_dir: &std::path::Path) -> serde_json::Value {
    let raw = fs::read_to_string(crate_dir.join(".cargo-checksum.json")).unwrap();
    serde_json::from_str(&raw).unwrap()
}

// region: single-file crate

/// A crate with a single file and a pre-existing package hash should keep
/// the package hash and recompute the file entry.
#[test]
fn single_file_preserves_package_hash() {
    let dir = TempDir::new().unwrap();
    let crate_dir = make_vendor_crate(&dir, "mycrate", &[("src/lib.rs", "pub fn hello() {}")]);

    let original_package = "deadbeef1234deadbeef1234deadbeef1234deadbeef1234deadbeef1234dead";
    write_checksum_json(&crate_dir, Some(original_package), &[]);

    recompute_cargo_checksum_json(&crate_dir).unwrap();

    let result = read_checksum_json(&crate_dir);

    // package field must be unchanged
    assert_eq!(
        result["package"].as_str().unwrap(),
        original_package,
        "package hash must be preserved"
    );

    // files map must have an entry for src/lib.rs
    let files = result["files"].as_object().unwrap();
    assert!(
        files.contains_key("src/lib.rs"),
        "files map missing src/lib.rs"
    );

    // hash must be the actual SHA-256 of the content
    let expected = sha256_hex(b"pub fn hello() {}");
    assert_eq!(files["src/lib.rs"].as_str().unwrap(), expected);

    // .cargo-checksum.json must NOT appear in the files map
    assert!(!files.contains_key(".cargo-checksum.json"));
}

// endregion

// region: multi-file crate

/// Multi-file crate: all files appear in the map, paths use forward slashes.
#[test]
fn multi_file_all_files_hashed() {
    let dir = TempDir::new().unwrap();
    let crate_dir = make_vendor_crate(
        &dir,
        "multi",
        &[
            ("src/lib.rs", "// lib"),
            ("src/util/helper.rs", "// helper"),
            (
                "Cargo.toml",
                "[package]\nname = \"multi\"\nversion = \"0.1.0\"",
            ),
        ],
    );
    write_checksum_json(&crate_dir, Some("aabbcc"), &[]);

    recompute_cargo_checksum_json(&crate_dir).unwrap();

    let result = read_checksum_json(&crate_dir);
    let files = result["files"].as_object().unwrap();

    assert!(files.contains_key("src/lib.rs"), "missing src/lib.rs");
    assert!(
        files.contains_key("src/util/helper.rs"),
        "missing nested file"
    );
    assert!(files.contains_key("Cargo.toml"), "missing Cargo.toml");
    assert_eq!(files.len(), 3, "unexpected extra entries");

    // Verify one hash
    assert_eq!(files["src/lib.rs"].as_str().unwrap(), sha256_hex(b"// lib"));
}

// endregion

// region: null package hash (git/path sources)

/// A crate whose `.cargo-checksum.json` has `"package": null` (git/path
/// source, no registry hash) must keep `null` after recompute.
#[test]
fn null_package_stays_null() {
    let dir = TempDir::new().unwrap();
    let crate_dir = make_vendor_crate(&dir, "gitcrate", &[("src/lib.rs", "pub fn foo() {}")]);
    write_checksum_json(&crate_dir, None, &[]);

    recompute_cargo_checksum_json(&crate_dir).unwrap();

    let result = read_checksum_json(&crate_dir);
    assert!(result["package"].is_null(), "null package must stay null");
    let files = result["files"].as_object().unwrap();
    assert!(files.contains_key("src/lib.rs"));
}

// endregion

// region: missing checksum file

/// If `.cargo-checksum.json` is absent (shouldn't happen in practice but
/// defensive), we write one with `"package": null` and full files map.
#[test]
fn absent_checksum_file_created() {
    let dir = TempDir::new().unwrap();
    let crate_dir = make_vendor_crate(&dir, "nocsum", &[("src/lib.rs", "fn main() {}")]);
    // Do NOT write a .cargo-checksum.json

    recompute_cargo_checksum_json(&crate_dir).unwrap();

    let result = read_checksum_json(&crate_dir);
    assert!(result["package"].is_null());
    let files = result["files"].as_object().unwrap();
    assert!(files.contains_key("src/lib.rs"));
}

// endregion

// region: empty crate (no source files)

/// An empty crate dir (only .cargo-checksum.json) should produce an empty
/// `files` map and preserve the original `package` field.
#[test]
fn empty_crate_empty_files_map() {
    let dir = TempDir::new().unwrap();
    let crate_dir = dir.path().join("empty");
    fs::create_dir_all(&crate_dir).unwrap();
    let pkg_hash = "cafebabe00cafebabe00cafebabe00cafebabe00cafebabe00cafebabe00cafe";
    write_checksum_json(&crate_dir, Some(pkg_hash), &[]);

    recompute_cargo_checksum_json(&crate_dir).unwrap();

    let result = read_checksum_json(&crate_dir);
    assert_eq!(result["package"].as_str().unwrap(), pkg_hash);
    let files = result["files"].as_object().unwrap();
    assert!(
        files.is_empty(),
        "files map should be empty for empty crate"
    );
}

// endregion

// region: subdirectory structure

/// Paths in the files map use forward slashes regardless of platform.
#[test]
fn files_map_uses_forward_slashes() {
    let dir = TempDir::new().unwrap();
    let crate_dir = make_vendor_crate(&dir, "pathtest", &[("deep/nested/file.rs", "// deep")]);
    write_checksum_json(&crate_dir, None, &[]);

    recompute_cargo_checksum_json(&crate_dir).unwrap();

    let result = read_checksum_json(&crate_dir);
    let files = result["files"].as_object().unwrap();
    assert!(
        files.contains_key("deep/nested/file.rs"),
        "expected forward-slash path, got: {:?}",
        files.keys().collect::<Vec<_>>()
    );
}

// endregion

// region: recompute_checksums (whole vendor dir)

/// `recompute_checksums` processes every crate directory and updates each
/// `.cargo-checksum.json`.
#[test]
fn recompute_checksums_processes_all_crates() {
    let dir = TempDir::new().unwrap();
    let vendor = dir.path();

    for name in &["crate_a", "crate_b"] {
        let crate_dir = vendor.join(name);
        fs::create_dir_all(crate_dir.join("src")).unwrap();
        fs::write(crate_dir.join("src/lib.rs"), format!("// {name}")).unwrap();
        write_checksum_json(&crate_dir, Some("oldhash"), &[]);
    }
    // A file at vendor root (not a crate dir) should be ignored.
    fs::write(vendor.join("README.md"), "top-level readme").unwrap();

    recompute_checksums(vendor).unwrap();

    for name in &["crate_a", "crate_b"] {
        let result = read_checksum_json(&vendor.join(name));
        let files = result["files"].as_object().unwrap();
        assert!(
            files.contains_key("src/lib.rs"),
            "{name} missing src/lib.rs"
        );
        // package preserved
        assert_eq!(result["package"].as_str().unwrap(), "oldhash");
    }
}

// endregion
