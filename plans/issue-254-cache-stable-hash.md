# Plan: replace `DefaultHasher` in cache.rs with stable FNV-1a (#254)

## Problem

`cargo-revendor/src/cache.rs:59–62` uses `std::collections::hash_map::DefaultHasher`
for computing cache keys. The docs explicitly say `DefaultHasher`'s implementation
may change between Rust versions, so cache files written by one toolchain
may not be recognized by another — silent cache misses around toolchain
upgrades.

`cargo-revendor/src/verify.rs:302` also uses DefaultHasher, but only for
within-run tarball comparison where cross-process stability doesn't matter.
Leave that usage unchanged.

## Files to change

- `cargo-revendor/src/cache.rs` — replace `DefaultHasher` with FNV-1a.
- `cargo-revendor/tests/integration.rs` (or a new `cache_stability.rs`) — add
  a frozen-hash regression test so any future implementation change is
  caught.

## Implementation

Replace the `use std::collections::hash_map::DefaultHasher;` block and the
hasher construction in `compute_hash` with FNV-1a:

```rust
/// FNV-1a 64-bit. Stable by construction across Rust versions/platforms.
fn fnv1a_64(data: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in data {
        h ^= b as u64;
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

/// Streaming FNV-1a accumulator for multi-chunk hashing without materializing
/// the full input.
struct Fnv64 {
    state: u64,
}

impl Fnv64 {
    fn new() -> Self {
        Self { state: 0xcbf2_9ce4_8422_2325 }
    }
    fn update(&mut self, data: &[u8]) {
        for &b in data {
            self.state ^= b as u64;
            self.state = self.state.wrapping_mul(0x0100_0000_01b3);
        }
    }
    fn finish(self) -> u64 {
        self.state
    }
}
```

Rewrite `compute_hash` to use `Fnv64`:

```rust
fn compute_hash(
    lockfile: &Path,
    sync_manifests: &[PathBuf],
    local_crate_paths: &[PathBuf],
) -> Result<String> {
    let mut hasher = Fnv64::new();

    if lockfile.exists() {
        hasher.update(&std::fs::read(lockfile)?);
    }
    // length-prefixed separator between fields to prevent collision via
    // concatenation ambiguity
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
```

**Important**: include a `|` separator between fields so two different inputs
don't accidentally produce the same hash by concatenation ambiguity (e.g.
`manifest="foo"` + `lockfile="bar"` vs `manifest="foo" lockfile=""` + `"bar"`).

## Cache invalidation on upgrade

After this change, existing caches will mismatch on first run post-upgrade
(expected — that's the bug we're fixing). Downstream impact: every user will
see one "cache miss, re-vendoring" after their first run on the new
cargo-revendor. Acceptable. Note in PR description.

## Regression test

Add `tests/cache_stability.rs`:

```rust
//! Cache-hash stability regression test.
//!
//! If this test ever starts failing, it means the cache hash function changed
//! incompatibly. All existing caches in the wild will be invalidated. If
//! intentional, update the expected hash below and note in the commit
//! message.

#[test]
fn fnv1a_empty() {
    // FNV-1a of empty input is the initial offset basis.
    assert_eq!(cargo_revendor::cache::fnv1a_64(b""), 0xcbf2_9ce4_8422_2325);
}

#[test]
fn fnv1a_known_vectors() {
    // Standard FNV-1a test vectors from http://www.isthe.com/chongo/tech/comp/fnv/
    assert_eq!(cargo_revendor::cache::fnv1a_64(b"a"), 0xaf63_dc4c_8601_ec8c);
    assert_eq!(cargo_revendor::cache::fnv1a_64(b"foobar"), 0x85944171f73967e8);
}

#[test]
fn compute_hash_is_deterministic() {
    // Two runs with identical inputs produce identical hashes.
    let dir = tempfile::tempdir().unwrap();
    let lockfile = dir.path().join("Cargo.lock");
    std::fs::write(&lockfile, b"[[package]]\nname=\"x\"\n").unwrap();
    let manifest = dir.path().join("Cargo.toml");
    std::fs::write(&manifest, b"[package]\nname=\"x\"\n").unwrap();

    let h1 = cargo_revendor::cache::compute_hash_for_test(&lockfile, &[], &[]).unwrap();
    let h2 = cargo_revendor::cache::compute_hash_for_test(&lockfile, &[], &[]).unwrap();
    assert_eq!(h1, h2);
}
```

Expose `fnv1a_64` and a test-visible `compute_hash_for_test` wrapper with
`#[doc(hidden)] pub`.

## Verification

```bash
just revendor-test
cd cargo-revendor && cargo test --test cache_stability
# manual: touch Cargo.lock; run `cargo revendor`; run again; second should hit cache
```

## Out of scope

- `verify.rs:302` — leave alone (within-process, stability not required)
- Switching to a crypto hash (blake3, sha2) — FNV-1a is sufficient for cache
  keys; smaller binary
- Detecting / migrating old DefaultHasher cache files — a one-time miss on
  upgrade is acceptable

## Risk

Low. Pure implementation swap. New hash values differ from old, so existing
caches will miss once on first run; that's expected and benign.

## PR expectations

- Branch: `fix/issue-254-stable-cache-hash`
- No merge — CR review
