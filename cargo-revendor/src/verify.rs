//! Verification: guarantee Cargo.lock, `vendor/`, and `inst/vendor.tar.xz` agree.
//!
//! Issue #157: the failure we guard against is a committed tarball that
//! disagrees with `Cargo.lock`, e.g.
//!
//! ```text
//! error: failed to select a version for the requirement `rayon = "^1.10"` (locked to 1.12.0)
//! candidate versions found which didn't match: 1.11.0
//! ```
//!
//! Two orthogonal checks:
//! 1. [`verify_lock_matches_vendor`] — every non-local lockfile entry has a
//!    corresponding `vendor/<name>/` or `vendor/<name>-<version>/` whose
//!    `Cargo.toml` reports the same version.
//! 2. [`verify_tarball_matches_vendor`] — extracting the tarball reproduces
//!    the file set and byte-for-byte content of `vendor/`.

use anyhow::{Context, Result, bail};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// One `[[package]]` entry from a Cargo.lock.
#[derive(Debug, Clone)]
pub struct LockPackage {
    pub name: String,
    pub version: String,
    /// Empty string for local path/workspace packages; `registry+...` or
    /// `git+...` for foreign sources that must be vendored.
    pub source: String,
}

impl LockPackage {
    fn is_vendored_source(&self) -> bool {
        !self.source.is_empty()
    }
}

/// Parse a Cargo.lock into its `[[package]]` entries.
pub fn parse_lockfile(lockfile: &Path) -> Result<Vec<LockPackage>> {
    let content = std::fs::read_to_string(lockfile)
        .with_context(|| format!("failed to read {}", lockfile.display()))?;
    let doc: toml_edit::DocumentMut = content
        .parse()
        .with_context(|| format!("failed to parse {}", lockfile.display()))?;

    let packages = doc
        .get("package")
        .and_then(|v| v.as_array_of_tables())
        .context("Cargo.lock has no [[package]] entries")?;

    let mut out = Vec::with_capacity(packages.len());
    for pkg in packages {
        let name = pkg
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let version = pkg
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let source = pkg
            .get("source")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if name.is_empty() || version.is_empty() {
            continue;
        }
        out.push(LockPackage {
            name,
            version,
            source,
        });
    }
    Ok(out)
}

/// Assert that every foreign-source package in `lockfile` has a corresponding
/// directory in `vendor_dir` whose `Cargo.toml` reports the same version.
///
/// Local path packages (empty `source`) are skipped — they are the crates
/// being vendored *from*, not *into*.
pub fn verify_lock_matches_vendor(lockfile: &Path, vendor_dir: &Path) -> Result<()> {
    if !vendor_dir.is_dir() {
        bail!(
            "vendor directory does not exist: {}\n\
             Run `just vendor` to populate it.",
            vendor_dir.display()
        );
    }

    let packages = parse_lockfile(lockfile)?;
    let mut missing: Vec<String> = Vec::new();
    let mut mismatched: Vec<(String, String, String)> = Vec::new();

    for pkg in &packages {
        if !pkg.is_vendored_source() {
            continue;
        }

        // cargo vendor flattens to `vendor/<name>/` when the crate appears
        // once; multi-version resolutions get `vendor/<name>-<version>/`.
        let versioned_dir = vendor_dir.join(format!("{}-{}", pkg.name, pkg.version));
        let flat_dir = vendor_dir.join(&pkg.name);

        let candidate = if versioned_dir.is_dir() {
            versioned_dir
        } else if flat_dir.is_dir() {
            flat_dir
        } else {
            missing.push(format!("{} v{} ({})", pkg.name, pkg.version, pkg.source));
            continue;
        };

        let manifest = candidate.join("Cargo.toml");
        let vendored_version = read_manifest_version(&manifest).with_context(|| {
            format!(
                "failed to read vendored manifest {} for locked crate {} v{}",
                manifest.display(),
                pkg.name,
                pkg.version
            )
        })?;
        if vendored_version != pkg.version {
            mismatched.push((pkg.name.clone(), pkg.version.clone(), vendored_version));
        }
    }

    if missing.is_empty() && mismatched.is_empty() {
        return Ok(());
    }

    let mut msg = format!(
        "Cargo.lock ({}) disagrees with {}:\n",
        lockfile.display(),
        vendor_dir.display()
    );
    if !missing.is_empty() {
        msg.push_str(&format!(
            "\n  Missing vendor/<name>/ for {} locked crate(s):\n",
            missing.len()
        ));
        for m in missing.iter().take(20) {
            msg.push_str(&format!("    - {}\n", m));
        }
        if missing.len() > 20 {
            msg.push_str(&format!("    ... and {} more\n", missing.len() - 20));
        }
    }
    if !mismatched.is_empty() {
        msg.push_str(&format!("\n  Version mismatches ({}):\n", mismatched.len()));
        for (name, locked, vendored) in mismatched.iter().take(20) {
            msg.push_str(&format!(
                "    - {}: Cargo.lock says {}, vendor/ says {}\n",
                name, locked, vendored
            ));
        }
        if mismatched.len() > 20 {
            msg.push_str(&format!("    ... and {} more\n", mismatched.len() - 20));
        }
    }
    msg.push_str(
        "\nRun `just vendor` to regenerate vendor/ and inst/vendor.tar.xz from Cargo.lock.",
    );
    bail!(msg);
}

fn read_manifest_version(manifest: &Path) -> Result<String> {
    let content = std::fs::read_to_string(manifest)?;
    let doc: toml_edit::DocumentMut = content.parse()?;
    let version = doc
        .get("package")
        .and_then(|t| t.as_table())
        .and_then(|t| t.get("version"))
        .and_then(|v| v.as_str())
        .context("Cargo.toml has no [package].version")?
        .to_string();
    Ok(version)
}

/// Assert that extracting `tarball` yields the same files with the same
/// contents as `vendor_dir`.
///
/// Uses byte-level hashing so drift in a single vendored `.rs` is caught.
pub fn verify_tarball_matches_vendor(tarball: &Path, vendor_dir: &Path) -> Result<()> {
    if !tarball.exists() {
        bail!(
            "vendor tarball does not exist: {}\n\
             Run `just vendor` to create it.",
            tarball.display()
        );
    }
    if !vendor_dir.is_dir() {
        bail!(
            "vendor directory does not exist: {}\n\
             Run `just vendor` to populate it.",
            vendor_dir.display()
        );
    }

    let tmp = tempfile::tempdir().context("failed to create tempdir for tarball extraction")?;
    let status = std::process::Command::new("tar")
        .arg("-xJf")
        .arg(tarball)
        .arg("-C")
        .arg(tmp.path())
        .status()
        .context("failed to spawn tar")?;
    if !status.success() {
        bail!("tar failed to extract {}", tarball.display());
    }

    // Tarballs produced by `cargo revendor --compress` contain a single
    // top-level directory — the vendored tree's name (e.g. "vendor/").
    let extracted_root = std::fs::read_dir(tmp.path())?
        .filter_map(|e| e.ok())
        .find(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .map(|e| e.path())
        .context("tarball contains no top-level directory")?;

    let tarball_files = collect_file_hashes(&extracted_root)?;
    let vendor_files = collect_file_hashes(vendor_dir)?;

    let tarball_keys: BTreeSet<_> = tarball_files.keys().collect();
    let vendor_keys: BTreeSet<_> = vendor_files.keys().collect();

    let only_in_tarball: Vec<String> = tarball_keys
        .difference(&vendor_keys)
        .map(|s| (*s).clone())
        .collect();
    let only_in_vendor: Vec<String> = vendor_keys
        .difference(&tarball_keys)
        .map(|s| (*s).clone())
        .collect();
    let mut differing: Vec<String> = Vec::new();
    for k in tarball_keys.intersection(&vendor_keys) {
        if tarball_files.get(*k) != vendor_files.get(*k) {
            differing.push((*k).clone());
        }
    }

    if only_in_tarball.is_empty() && only_in_vendor.is_empty() && differing.is_empty() {
        return Ok(());
    }

    let mut msg = format!(
        "vendor tarball is out of sync with vendor tree:\n  tarball: {}\n  vendor: {}\n",
        tarball.display(),
        vendor_dir.display()
    );
    for (label, list) in [
        ("files only in tarball", &only_in_tarball),
        ("files only in vendor/", &only_in_vendor),
        ("files with differing content", &differing),
    ] {
        if list.is_empty() {
            continue;
        }
        msg.push_str(&format!("\n  {} ({}):\n", label, list.len()));
        for item in list.iter().take(20) {
            msg.push_str(&format!("    - {}\n", item));
        }
        if list.len() > 20 {
            msg.push_str(&format!("    ... and {} more\n", list.len() - 20));
        }
    }
    msg.push_str(
        "\nRun `just vendor` to regenerate inst/vendor.tar.xz from the current vendor/ tree.",
    );
    bail!(msg);
}

fn collect_file_hashes(root: &Path) -> Result<BTreeMap<String, u64>> {
    use std::hash::{Hash, Hasher};
    let mut out = BTreeMap::new();
    for entry in walkdir::WalkDir::new(root).min_depth(1) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(root)
            .unwrap_or(entry.path())
            .to_string_lossy()
            .into_owned();
        // cache.rs writes `.revendor-cache*` files at the top of vendor/ AFTER
        // the tarball is compressed, so they're never in the tarball by design —
        // skip them to avoid perpetual false-positive diffs.
        if rel == ".revendor-cache"
            || rel == ".revendor-cache-external"
            || rel == ".revendor-cache-local"
        {
            continue;
        }
        let bytes = std::fs::read(entry.path())?;
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        bytes.hash(&mut hasher);
        out.insert(rel, hasher.finish());
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
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
}
