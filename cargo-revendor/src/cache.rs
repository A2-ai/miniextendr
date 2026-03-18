//! Caching: skip re-vendoring when Cargo.lock is unchanged
//!
//! Stores a hash of Cargo.lock + Cargo.toml in vendor/.revendor-cache.
//! If the hash matches on next run, vendoring is skipped.

use anyhow::Result;
use std::path::Path;

const CACHE_FILE: &str = ".revendor-cache";

/// Check if the vendor directory is cached and up to date
pub fn is_cached(lockfile: &Path, vendor_dir: &Path) -> Result<bool> {
    let cache_path = vendor_dir.join(CACHE_FILE);
    if !cache_path.exists() || !vendor_dir.exists() {
        return Ok(false);
    }

    let current = compute_hash(lockfile)?;
    let cached = std::fs::read_to_string(&cache_path)?;

    Ok(current.trim() == cached.trim())
}

/// Save the current hash to the cache file
pub fn save_cache(lockfile: &Path, vendor_dir: &Path) -> Result<()> {
    let hash = compute_hash(lockfile)?;
    let cache_path = vendor_dir.join(CACHE_FILE);
    std::fs::write(&cache_path, &hash)?;
    Ok(())
}

/// Compute a hash from Cargo.lock (and Cargo.toml for good measure)
fn compute_hash(lockfile: &Path) -> Result<String> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();

    // Hash Cargo.lock content
    if lockfile.exists() {
        let content = std::fs::read(lockfile)?;
        content.hash(&mut hasher);
    }

    // Also hash Cargo.toml (catches dep changes before lock update)
    let manifest = lockfile.with_file_name("Cargo.toml");
    if manifest.exists() {
        let content = std::fs::read(&manifest)?;
        content.hash(&mut hasher);
    }

    Ok(format!("{:x}", hasher.finish()))
}
