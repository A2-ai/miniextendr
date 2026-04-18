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
/// source trees of `local_crate_paths`, plus any additional manifests
/// supplied via `--sync` (#229). Each sync manifest's sibling `Cargo.lock`
/// and `Cargo.toml` are hashed into the key.
pub fn is_cached(
    lockfile: &Path,
    sync_manifests: &[PathBuf],
    vendor_dir: &Path,
    local_crate_paths: &[PathBuf],
) -> Result<bool> {
    let cache_path = vendor_dir.join(CACHE_FILE);
    if !cache_path.exists() || !vendor_dir.exists() {
        return Ok(false);
    }

    let current = compute_hash(lockfile, sync_manifests, local_crate_paths)?;
    let cached = std::fs::read_to_string(&cache_path)?;

    Ok(current.trim() == cached.trim())
}

/// Save the current hash to the cache file.
pub fn save_cache(
    lockfile: &Path,
    sync_manifests: &[PathBuf],
    vendor_dir: &Path,
    local_crate_paths: &[PathBuf],
) -> Result<()> {
    let hash = compute_hash(lockfile, sync_manifests, local_crate_paths)?;
    let cache_path = vendor_dir.join(CACHE_FILE);
    std::fs::write(&cache_path, &hash)?;
    Ok(())
}

/// Compute a hash over `Cargo.lock`, the sibling `Cargo.toml`, the source
/// tree of each local workspace crate, and each `--sync` manifest's
/// Cargo.toml + Cargo.lock pair (#229).
fn compute_hash(
    lockfile: &Path,
    sync_manifests: &[PathBuf],
    local_crate_paths: &[PathBuf],
) -> Result<String> {
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

    // Each --sync manifest contributes its own Cargo.toml + Cargo.lock pair.
    // Sort by path so ordering of --sync args doesn't affect the key — two
    // equivalent sync sets pass cache regardless of CLI order.
    let mut sorted_sync: Vec<&PathBuf> = sync_manifests.iter().collect();
    sorted_sync.sort();
    for sync_manifest in sorted_sync {
        if sync_manifest.exists() {
            std::fs::read(sync_manifest)?.hash(&mut hasher);
        }
        let sync_lock = sync_manifest.with_file_name("Cargo.lock");
        if sync_lock.exists() {
            std::fs::read(&sync_lock)?.hash(&mut hasher);
        }
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

        save_cache(&lockfile, &[], &vendor, &locals).unwrap();
        assert!(is_cached(&lockfile, &[], &vendor, &locals).unwrap());

        std::fs::write(crate_path.join("src/lib.rs"), "// v2 changed").unwrap();
        assert!(
            !is_cached(&lockfile, &[], &vendor, &locals).unwrap(),
            "cache should invalidate when a local crate source file changes"
        );
    }

    #[test]
    fn cache_invalidates_on_local_crate_manifest_change() {
        let (tmp, lockfile, vendor) = setup();
        let crate_path = tmp.path().join("libx");
        let locals = vec![crate_path.clone()];

        save_cache(&lockfile, &[], &vendor, &locals).unwrap();
        std::fs::write(
            crate_path.join("Cargo.toml"),
            "[package]\nname = \"libx\"\nversion = \"0.2.0\"",
        )
        .unwrap();
        assert!(!is_cached(&lockfile, &[], &vendor, &locals).unwrap());
    }

    #[test]
    fn cache_stable_when_nothing_changes() {
        let (_tmp, lockfile, vendor) = setup();
        let crate_path = _tmp.path().join("libx");
        let locals = vec![crate_path];

        save_cache(&lockfile, &[], &vendor, &locals).unwrap();
        assert!(is_cached(&lockfile, &[], &vendor, &locals).unwrap());
        // Repeat hit.
        assert!(is_cached(&lockfile, &[], &vendor, &locals).unwrap());
    }

    #[test]
    fn cache_invalidates_on_sync_manifest_change() {
        // #229: when --sync manifests are passed, edits to their
        // Cargo.toml or Cargo.lock must invalidate the cache — otherwise
        // cargo-revendor will serve a stale vendor/ tree after a sync'd
        // workspace's dep graph changes.
        let (tmp, lockfile, vendor) = setup();
        let crate_path = tmp.path().join("libx");
        let locals = vec![crate_path];

        // Prime a sync'd workspace with a manifest + lockfile.
        let sync_ws = tmp.path().join("sync-ws");
        std::fs::create_dir_all(&sync_ws).unwrap();
        let sync_manifest = sync_ws.join("Cargo.toml");
        let sync_lock = sync_ws.join("Cargo.lock");
        std::fs::write(&sync_manifest, "[package]\nname = \"syncpkg\"").unwrap();
        std::fs::write(&sync_lock, "version = 3").unwrap();

        let sync = vec![sync_manifest.clone()];
        save_cache(&lockfile, &sync, &vendor, &locals).unwrap();
        assert!(is_cached(&lockfile, &sync, &vendor, &locals).unwrap());

        // Bump the sync manifest; cache should invalidate.
        std::fs::write(&sync_manifest, "[package]\nname = \"syncpkg\"\nversion = \"0.2.0\"").unwrap();
        assert!(
            !is_cached(&lockfile, &sync, &vendor, &locals).unwrap(),
            "cache should invalidate when a --sync manifest changes"
        );

        // Restore + bump the sync lockfile; again cache should invalidate.
        std::fs::write(&sync_manifest, "[package]\nname = \"syncpkg\"").unwrap();
        save_cache(&lockfile, &sync, &vendor, &locals).unwrap();
        std::fs::write(&sync_lock, "version = 4").unwrap();
        assert!(
            !is_cached(&lockfile, &sync, &vendor, &locals).unwrap(),
            "cache should invalidate when a --sync Cargo.lock changes"
        );
    }

    #[test]
    fn cache_stable_when_sync_order_differs() {
        // Re-ordering the --sync arg list must NOT invalidate the cache;
        // cargo vendor doesn't care about order either.
        let (tmp, lockfile, vendor) = setup();
        let locals: Vec<std::path::PathBuf> = Vec::new();

        let a = tmp.path().join("a.toml");
        let b = tmp.path().join("b.toml");
        std::fs::write(&a, "[package]\nname = \"a\"").unwrap();
        std::fs::write(&b, "[package]\nname = \"b\"").unwrap();
        std::fs::write(a.with_file_name("a.lock"), "1").ok();
        std::fs::write(b.with_file_name("b.lock"), "2").ok();

        save_cache(&lockfile, &[a.clone(), b.clone()], &vendor, &locals).unwrap();
        assert!(
            is_cached(&lockfile, &[b, a], &vendor, &locals).unwrap(),
            "swapped --sync order should still hit the cache"
        );
    }
}
