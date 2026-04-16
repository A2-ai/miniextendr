//! Caching: skip re-vendoring when all inputs are unchanged
//!
//! Stores a hash in `vendor/.revendor-cache` covering:
//! - The caller's `Cargo.lock` and `Cargo.toml`
//! - The source tree of each local workspace crate that gets vendored
//!   (Cargo.toml + every file under `src/`, `tests/`, `examples/`, `benches/`)
//!
//! Hashing the lockfile alone misses pure source-file edits to workspace
//! crates (see issue #150), which leaves a stale `vendor/` copy on disk.

use anyhow::Result;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const CACHE_FILE: &str = ".revendor-cache";

/// Check whether `vendor_dir` is up to date relative to `lockfile` plus the
/// source trees of `local_crate_paths`.
pub fn is_cached(
    lockfile: &Path,
    vendor_dir: &Path,
    local_crate_paths: &[PathBuf],
) -> Result<bool> {
    let cache_path = vendor_dir.join(CACHE_FILE);
    if !cache_path.exists() || !vendor_dir.exists() {
        return Ok(false);
    }

    let current = compute_hash(lockfile, local_crate_paths)?;
    let cached = std::fs::read_to_string(&cache_path)?;

    Ok(current.trim() == cached.trim())
}

/// Save the current hash to the cache file.
pub fn save_cache(
    lockfile: &Path,
    vendor_dir: &Path,
    local_crate_paths: &[PathBuf],
) -> Result<()> {
    let hash = compute_hash(lockfile, local_crate_paths)?;
    let cache_path = vendor_dir.join(CACHE_FILE);
    std::fs::write(&cache_path, &hash)?;
    Ok(())
}

/// Compute a hash over `Cargo.lock`, the sibling `Cargo.toml`, and the source
/// tree of each local workspace crate.
fn compute_hash(lockfile: &Path, local_crate_paths: &[PathBuf]) -> Result<String> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();

    if lockfile.exists() {
        std::fs::read(lockfile)?.hash(&mut hasher);
    }

    let manifest = lockfile.with_file_name("Cargo.toml");
    if manifest.exists() {
        std::fs::read(&manifest)?.hash(&mut hasher);
    }

    // Hash each local crate's source tree in a deterministic order so the
    // cache key is stable across runs.
    for crate_path in local_crate_paths {
        let entries = collect_crate_files(crate_path)?;
        for (rel, bytes) in entries {
            rel.hash(&mut hasher);
            bytes.hash(&mut hasher);
        }
    }

    Ok(format!("{:x}", hasher.finish()))
}

/// Collect the files under a local crate that should influence the cache
/// key: `Cargo.toml` plus everything under `src/`, `tests/`, `examples/`,
/// `benches/`, and `build.rs`. Returns a sorted map of `(relative path,
/// file bytes)`.
fn collect_crate_files(crate_path: &Path) -> Result<BTreeMap<String, Vec<u8>>> {
    let mut out = BTreeMap::new();

    let root_files = ["Cargo.toml", "build.rs"];
    for name in root_files {
        let p = crate_path.join(name);
        if p.is_file() {
            out.insert(name.to_string(), std::fs::read(&p)?);
        }
    }

    for sub in ["src", "tests", "examples", "benches"] {
        let dir = crate_path.join(sub);
        if dir.is_dir() {
            walk_dir(&dir, crate_path, &mut out)?;
        }
    }

    Ok(out)
}

fn walk_dir(
    dir: &Path,
    root: &Path,
    out: &mut BTreeMap<String, Vec<u8>>,
) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            walk_dir(&path, root, out)?;
        } else if file_type.is_file() {
            let rel = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .into_owned();
            out.insert(rel, std::fs::read(&path)?);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup() -> (TempDir, PathBuf, PathBuf) {
        let tmp = TempDir::new().unwrap();
        let caller = tmp.path().join("caller");
        std::fs::create_dir_all(&caller).unwrap();
        std::fs::write(caller.join("Cargo.toml"), "[package]\nname = \"c\"").unwrap();
        std::fs::write(caller.join("Cargo.lock"), "version = 3").unwrap();

        let crate_path = tmp.path().join("libx");
        std::fs::create_dir_all(crate_path.join("src")).unwrap();
        std::fs::write(crate_path.join("Cargo.toml"), "[package]\nname = \"libx\"").unwrap();
        std::fs::write(crate_path.join("src/lib.rs"), "// v1").unwrap();

        let vendor = tmp.path().join("vendor");
        std::fs::create_dir_all(&vendor).unwrap();

        let lockfile = caller.join("Cargo.lock");
        (tmp, lockfile, vendor)
    }

    #[test]
    fn cache_invalidates_on_local_crate_source_change() {
        let (tmp, lockfile, vendor) = setup();
        let crate_path = tmp.path().join("libx");
        let locals = vec![crate_path.clone()];

        save_cache(&lockfile, &vendor, &locals).unwrap();
        assert!(is_cached(&lockfile, &vendor, &locals).unwrap());

        std::fs::write(crate_path.join("src/lib.rs"), "// v2 changed").unwrap();
        assert!(
            !is_cached(&lockfile, &vendor, &locals).unwrap(),
            "cache should invalidate when a local crate source file changes"
        );
    }

    #[test]
    fn cache_invalidates_on_local_crate_manifest_change() {
        let (tmp, lockfile, vendor) = setup();
        let crate_path = tmp.path().join("libx");
        let locals = vec![crate_path.clone()];

        save_cache(&lockfile, &vendor, &locals).unwrap();
        std::fs::write(
            crate_path.join("Cargo.toml"),
            "[package]\nname = \"libx\"\nversion = \"0.2.0\"",
        )
        .unwrap();
        assert!(!is_cached(&lockfile, &vendor, &locals).unwrap());
    }

    #[test]
    fn cache_stable_when_nothing_changes() {
        let (_tmp, lockfile, vendor) = setup();
        let crate_path = _tmp.path().join("libx");
        let locals = vec![crate_path];

        save_cache(&lockfile, &vendor, &locals).unwrap();
        assert!(is_cached(&lockfile, &vendor, &locals).unwrap());
        // Repeat hit.
        assert!(is_cached(&lockfile, &vendor, &locals).unwrap());
    }
}
