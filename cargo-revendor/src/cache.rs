//! Caching: skip re-vendoring when all inputs are unchanged
//!
//! Stores a hash in `vendor/.revendor-cache` covering:
//! - The caller's `Cargo.lock` and `Cargo.toml`
//! - The source tree of each local workspace crate that gets vendored
//!   (Cargo.toml + every file under `src/`, `tests/`, `examples/`, `benches/`)
//!
//! Hashing the lockfile alone misses pure source-file edits to workspace
//! crates (see issue #150), which leaves a stale `vendor/` copy on disk.
//!
//! Phase-mode caches (#290): `.revendor-cache-external` hashes only external
//! inputs (Cargo.lock + Cargo.toml + sync manifests); written by
//! `--external-only` and checked to gate `--local-only` flag compatibility.
//! `.revendor-cache-local` hashes only local crate source trees; written by
//! `--local-only`. Full mode writes all three cache files.

use anyhow::Result;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const CACHE_FILE: &str = ".revendor-cache";
pub const CACHE_FILE_EXTERNAL: &str = ".revendor-cache-external";
pub const CACHE_FILE_LOCAL: &str = ".revendor-cache-local";

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

// region: phase-mode caches (#290)

/// Check whether the external deps are up to date relative to `lockfile` and
/// sync manifests. Ignores local crate source trees — pure source edits don't
/// change external deps.
pub fn is_cached_external(
    lockfile: &Path,
    sync_manifests: &[PathBuf],
    vendor_dir: &Path,
) -> Result<bool> {
    let cache_path = vendor_dir.join(CACHE_FILE_EXTERNAL);
    if !cache_path.exists() || !vendor_dir.exists() {
        return Ok(false);
    }
    let current = compute_hash_external(lockfile, sync_manifests)?;
    let cached = std::fs::read_to_string(&cache_path)?;
    Ok(current.trim() == cached.trim())
}

/// Check whether the local crates are up to date relative to their source
/// trees. Ignores Cargo.lock / sync manifests — lockfile changes don't affect
/// the local crate packaging output.
pub fn is_cached_local(vendor_dir: &Path, local_crate_paths: &[PathBuf]) -> Result<bool> {
    let cache_path = vendor_dir.join(CACHE_FILE_LOCAL);
    if !cache_path.exists() || !vendor_dir.exists() {
        return Ok(false);
    }
    let current = compute_hash_local(local_crate_paths)?;
    let cached = std::fs::read_to_string(&cache_path)?;
    Ok(current.trim() == cached.trim())
}

/// Save the external-only cache file.
pub fn save_cache_external(
    lockfile: &Path,
    sync_manifests: &[PathBuf],
    vendor_dir: &Path,
) -> Result<()> {
    let hash = compute_hash_external(lockfile, sync_manifests)?;
    std::fs::write(vendor_dir.join(CACHE_FILE_EXTERNAL), &hash)?;
    Ok(())
}

/// Save the local-only cache file.
pub fn save_cache_local(vendor_dir: &Path, local_crate_paths: &[PathBuf]) -> Result<()> {
    let hash = compute_hash_local(local_crate_paths)?;
    std::fs::write(vendor_dir.join(CACHE_FILE_LOCAL), &hash)?;
    Ok(())
}

// endregion

/// FNV-1a 64-bit streaming hasher.
///
/// Stable by construction across Rust toolchain versions — the only
/// way the output changes is if the FNV prime / offset basis change,
/// which are part of the published FNV spec. See the frozen-vector test
/// at the bottom of this module: any change to the hash output breaks
/// it loudly, which is exactly the behavior we want (cache-compat
/// breakage should be a conscious choice, not accidental).
///
/// Replaces the previous `std::collections::hash_map::DefaultHasher`,
/// whose implementation Rust explicitly reserves the right to change
/// between releases — hashed values computed on one toolchain can
/// silently mismatch on another, causing unnecessary cache misses on
/// every rustup update.
struct Fnv64 {
    state: u64,
}

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0100_0000_01b3;

impl Fnv64 {
    fn new() -> Self {
        Self {
            state: FNV_OFFSET_BASIS,
        }
    }

    fn update(&mut self, data: &[u8]) {
        for &b in data {
            self.state ^= u64::from(b);
            self.state = self.state.wrapping_mul(FNV_PRIME);
        }
    }

    fn finish(self) -> u64 {
        self.state
    }
}

/// Compute a hash over `Cargo.lock`, the sibling `Cargo.toml`, the source
/// tree of each local workspace crate, and each `--sync` manifest's
/// Cargo.toml + Cargo.lock pair (#229).
///
/// Uses FNV-1a so the cache key is stable across Rust toolchain upgrades
/// (unlike `DefaultHasher`, whose implementation Rust reserves the right
/// to change between releases).
///
/// Fields are separated by `b"|"` bytes so no concatenation-ambiguity
/// collision is possible. Without separators, the inputs
/// `("foo", "bar")` and `("foob", "ar")` would hash identically.
fn compute_hash(
    lockfile: &Path,
    sync_manifests: &[PathBuf],
    local_crate_paths: &[PathBuf],
) -> Result<String> {
    let mut hasher = Fnv64::new();

    if lockfile.exists() {
        hasher.update(&std::fs::read(lockfile)?);
    }
    hasher.update(b"|");

    let manifest = lockfile.with_file_name("Cargo.toml");
    if manifest.exists() {
        hasher.update(&std::fs::read(&manifest)?);
    }
    hasher.update(b"|");

    // Each --sync manifest contributes its own Cargo.toml + Cargo.lock pair.
    // Sort by path so ordering of --sync args doesn't affect the key — two
    // equivalent sync sets pass cache regardless of CLI order.
    let mut sorted_sync: Vec<&PathBuf> = sync_manifests.iter().collect();
    sorted_sync.sort();
    for sync_manifest in sorted_sync {
        if sync_manifest.exists() {
            hasher.update(&std::fs::read(sync_manifest)?);
        }
        hasher.update(b"|");
        let sync_lock = sync_manifest.with_file_name("Cargo.lock");
        if sync_lock.exists() {
            hasher.update(&std::fs::read(&sync_lock)?);
        }
        hasher.update(b"|");
    }

    // Hash each local crate's source tree in a deterministic order so the
    // cache key is stable across runs.
    for crate_path in local_crate_paths {
        let entries = collect_crate_files(crate_path)?;
        for (rel, bytes) in entries {
            hasher.update(rel.as_bytes());
            hasher.update(b":");
            hasher.update(&bytes);
            hasher.update(b"|");
        }
    }

    Ok(format!("{:016x}", hasher.finish()))
}

/// Compute a hash covering only the external inputs: `Cargo.lock`, the sibling
/// `Cargo.toml`, and each `--sync` manifest's Cargo.toml + Cargo.lock pair.
/// Local crate source trees are excluded — external deps don't change when
/// workspace crate sources change.
fn compute_hash_external(lockfile: &Path, sync_manifests: &[PathBuf]) -> Result<String> {
    let mut hasher = Fnv64::new();

    if lockfile.exists() {
        hasher.update(&std::fs::read(lockfile)?);
    }
    hasher.update(b"|");

    let manifest = lockfile.with_file_name("Cargo.toml");
    if manifest.exists() {
        hasher.update(&std::fs::read(&manifest)?);
    }
    hasher.update(b"|");

    let mut sorted_sync: Vec<&PathBuf> = sync_manifests.iter().collect();
    sorted_sync.sort();
    for sync_manifest in sorted_sync {
        if sync_manifest.exists() {
            hasher.update(&std::fs::read(sync_manifest)?);
        }
        hasher.update(b"|");
        let sync_lock = sync_manifest.with_file_name("Cargo.lock");
        if sync_lock.exists() {
            hasher.update(&std::fs::read(&sync_lock)?);
        }
        hasher.update(b"|");
    }

    Ok(format!("{:016x}", hasher.finish()))
}

/// Compute a hash covering only the local crate source trees. Cargo.lock and
/// sync manifests are excluded — they don't influence the packaged output of
/// local workspace crates.
fn compute_hash_local(local_crate_paths: &[PathBuf]) -> Result<String> {
    let mut hasher = Fnv64::new();

    for crate_path in local_crate_paths {
        let entries = collect_crate_files(crate_path)?;
        for (rel, bytes) in entries {
            hasher.update(rel.as_bytes());
            hasher.update(b":");
            hasher.update(&bytes);
            hasher.update(b"|");
        }
    }

    Ok(format!("{:016x}", hasher.finish()))
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

fn walk_dir(dir: &Path, root: &Path, out: &mut BTreeMap<String, Vec<u8>>) -> Result<()> {
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
mod tests;
