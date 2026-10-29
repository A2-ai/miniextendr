//! Recompute `.cargo-checksum.json` for a vendored crate after CRAN-trim.
//!
//! ## Background
//!
//! `cargo vendor` writes `.cargo-checksum.json` with two fields:
//!
//! - `"package"`: SHA-256 of the original `.crate` tarball from the registry.
//!   This value is also what appears in `Cargo.lock`'s `checksum = "..."` line.
//! - `"files"`: a map from each file's POSIX-relative path to its SHA-256.
//!   Cargo verifies these at build time via `DirectorySource::verify()`.
//!
//! When cargo resolves with a directory source (vendored crates via
//! `[source.crates-io] replace-with = "vendored-sources"`), it reads the
//! `package` field and stores it as the summary checksum. During `merge_from`
//! (which reconciles the new resolution against the committed `Cargo.lock`),
//! cargo compares:
//!
//! - `Cargo.lock`'s `checksum = "..."` line for a package, and
//! - the `package` field from that package's `.cargo-checksum.json`.
//!
//! If the lock has `Some(hash)` but the vendored checksum is `None`, cargo
//! errors: "checksum for X could not be calculated, but a checksum is listed
//! in the existing lock file — unable to verify that X is the same as when
//! the lockfile was generated."
//!
//! If both are `Some` and differ, cargo errors: "checksum for X changed
//! between lock files."
//!
//! If both are the same `Some`, or both are `None` — all good.
//!
//! ## Strategy
//!
//! CRAN-trim removes files (test suites, benchmarks, docs, etc.) from each
//! vendored crate. After trim the `files` map in `.cargo-checksum.json` would
//! no longer match the actual disk contents, and a stale `package` field that
//! was computed from the *original* tarball would be referencing files that no
//! longer exist.
//!
//! We **preserve the original `package` field** (which matches the committed
//! `Cargo.lock`'s `checksum =` line), and **recompute the `files` map** from
//! the post-trim disk contents.  This means:
//!
//! 1. The lockfile's `checksum = "..."` line still matches `package` in
//!    `.cargo-checksum.json`, so `merge_from` succeeds.
//! 2. The `files` map reflects the trimmed state, so `DirectorySource::verify()`
//!    succeeds for the files that remain.
//!
//! For crates whose `.cargo-checksum.json` has `"package": null` (git-source
//! or path-source crates that cargo vendor emits without a registry hash), we
//! preserve `null` — no lockfile checksum line exists for them anyway.
//!
//! For crates whose `.cargo-checksum.json` is completely absent (shouldn't
//! happen in a well-formed vendor directory, but defensive), we write a minimal
//! `{"files": {...}, "package": null}`.

use anyhow::{Context, Result};
use sha2::Digest;
use std::collections::BTreeMap;
use std::path::Path;

/// Recompute `.cargo-checksum.json` for a single vendored crate directory.
///
/// Preserves the original `package` field (the registry `.crate` SHA-256 that
/// matches the committed `Cargo.lock`'s `checksum =` line) and rewrites the
/// `files` map with SHA-256s of every regular file currently present in
/// `crate_dir/`, excluding `.cargo-checksum.json` itself.
///
/// POSIX-relative paths (forward slashes) are used in the `files` map, as
/// required by cargo's directory source format.
pub fn recompute_cargo_checksum_json(crate_dir: &Path) -> Result<()> {
    let cksum_path = crate_dir.join(".cargo-checksum.json");

    // Read the existing checksum file (if present) to extract the original
    // `package` field. If absent, default to null.
    let existing_package: Option<String> = if cksum_path.exists() {
        let raw = std::fs::read_to_string(&cksum_path).with_context(|| {
            format!(
                "failed to read .cargo-checksum.json in {}",
                crate_dir.display()
            )
        })?;
        let parsed: serde_json::Value = serde_json::from_str(&raw).with_context(|| {
            format!(
                "failed to parse .cargo-checksum.json in {}",
                crate_dir.display()
            )
        })?;
        parsed
            .get("package")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    } else {
        None
    };

    // Walk every regular file in crate_dir, computing SHA-256 for each.
    // Skip .cargo-checksum.json itself (cargo doesn't include it in the
    // files map — only the source files are hashed).
    let files = collect_file_checksums(crate_dir)?;

    // Serialise: keys sorted (BTreeMap), values as lowercase hex strings.
    // Matches cargo vendor's own output format.
    let json = if let Some(pkg_hash) = existing_package {
        serde_json::json!({
            "package": pkg_hash,
            "files": files,
        })
    } else {
        serde_json::json!({
            "package": null,
            "files": files,
        })
    };

    std::fs::write(&cksum_path, json.to_string()).with_context(|| {
        format!(
            "failed to write .cargo-checksum.json in {}",
            crate_dir.display()
        )
    })?;

    Ok(())
}

/// Walk `crate_dir` recursively, hash every regular file (excluding
/// `.cargo-checksum.json`), and return a `BTreeMap<posix_relative_path, hex_sha256>`.
fn collect_file_checksums(crate_dir: &Path) -> Result<BTreeMap<String, String>> {
    let mut files = BTreeMap::new();

    for entry in walkdir::WalkDir::new(crate_dir)
        .follow_links(false)
        .sort_by_file_name()
    {
        let entry =
            entry.with_context(|| format!("failed to walk directory {}", crate_dir.display()))?;

        // Only hash regular files.
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();

        // Skip .cargo-checksum.json itself — cargo doesn't include it in the
        // files map (see cargo/src/cargo/ops/vendor.rs).
        if path
            .file_name()
            .is_some_and(|n| n == ".cargo-checksum.json")
        {
            continue;
        }

        let contents =
            std::fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;

        let hex = sha256_hex(&contents);

        // Relative path with POSIX (forward) slashes.
        let rel = path
            .strip_prefix(crate_dir)
            .with_context(|| {
                format!(
                    "path {} is not under {}",
                    path.display(),
                    crate_dir.display()
                )
            })?
            .to_string_lossy()
            .replace('\\', "/");

        files.insert(rel, hex);
    }

    Ok(files)
}

/// Compute the SHA-256 of `data` and return it as a lowercase hex string.
pub(crate) fn sha256_hex(data: &[u8]) -> String {
    let digest = sha2::Sha256::digest(data);
    format!("{digest:x}")
}

/// Recompute `.cargo-checksum.json` for every crate directory in `vendor_dir`.
///
/// This replaces `clear_checksums`: instead
/// of writing `{"files":{}}` (empty files map, null package), we preserve the
/// original `package` hash (matching the committed `Cargo.lock`) and recompute
/// the `files` map from the trimmed disk contents.
///
/// Called after CRAN-trim so that cargo's offline source-replacement can verify
/// both the lockfile consistency (via the `package` field) and the file
/// integrity (via the `files` map).
pub fn recompute_checksums(vendor_dir: &Path) -> Result<()> {
    for entry in std::fs::read_dir(vendor_dir)
        .with_context(|| format!("failed to read vendor dir {}", vendor_dir.display()))?
    {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            recompute_cargo_checksum_json(&entry.path()).with_context(|| {
                format!(
                    "failed to recompute checksums for {}",
                    entry.path().display()
                )
            })?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
