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
    std::fs::write(
        &sync_manifest,
        "[package]\nname = \"syncpkg\"\nversion = \"0.2.0\"",
    )
    .unwrap();
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

// region: FNV-1a stability regression
//
// These vectors are load-bearing: a change to Fnv64's output means all
// cached vendor trees in the wild will miss once on first use. If you
// must change them, note the cache invalidation in the commit message
// and update both the test expectations and the values here.

#[test]
fn fnv1a_empty_input_is_offset_basis() {
    // FNV-1a of empty input is defined as the initial offset basis.
    let h = Fnv64::new();
    assert_eq!(h.finish(), FNV_OFFSET_BASIS);
}

#[test]
fn fnv1a_known_vectors() {
    // Standard FNV-1a 64-bit test vectors from
    // http://www.isthe.com/chongo/tech/comp/fnv/#FNV-test-vectors
    let cases: &[(&[u8], u64)] = &[
        (b"", 0xcbf2_9ce4_8422_2325),
        (b"a", 0xaf63_dc4c_8601_ec8c),
        (b"foobar", 0x8594_4171_f739_67e8),
        (b"a" as &[u8], 0xaf63_dc4c_8601_ec8c),
    ];
    for (input, expected) in cases {
        let mut h = Fnv64::new();
        h.update(input);
        assert_eq!(h.finish(), *expected, "FNV-1a mismatch for {input:?}");
    }
}

#[test]
fn compute_hash_is_deterministic_across_calls() {
    // Identical inputs must always produce identical output (the
    // property that DefaultHasher did not guarantee across Rust
    // versions). Two calls in the same process is a weaker check than
    // cross-process, but it's the right level for unit testing.
    let (_tmp, lockfile, _vendor) = setup();
    let crate_path = _tmp.path().join("libx");
    let locals = vec![crate_path];

    let h1 = compute_hash(&lockfile, &[], &locals).unwrap();
    let h2 = compute_hash(&lockfile, &[], &locals).unwrap();
    assert_eq!(h1, h2);

    // Also: format is 16 hex chars (u64 as 16-char zero-padded hex).
    assert_eq!(h1.len(), 16, "hash format changed: {h1}");
    assert!(
        h1.chars().all(|c| c.is_ascii_hexdigit()),
        "hash is not hex: {h1}"
    );
}

// endregion

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

// region: phase-mode cache tests (#290)

#[test]
fn external_cache_ignores_local_source_edits() {
    // Editing a local crate's source file must NOT invalidate the external
    // cache — external deps don't change when workspace source changes.
    let (tmp, lockfile, vendor) = setup();
    let crate_path = tmp.path().join("libx");

    save_cache_external(&lockfile, &[], &vendor).unwrap();
    assert!(is_cached_external(&lockfile, &[], &vendor).unwrap());

    // Edit the local crate source
    std::fs::write(crate_path.join("src/lib.rs"), "// v2 changed").unwrap();

    assert!(
        is_cached_external(&lockfile, &[], &vendor).unwrap(),
        "external cache should remain valid when only local crate source changes"
    );
}

#[test]
fn local_cache_ignores_lockfile_change() {
    // Changing Cargo.lock must NOT invalidate the local cache — the
    // packaged output of local workspace crates doesn't depend on the lock.
    let (tmp, lockfile, vendor) = setup();
    let crate_path = tmp.path().join("libx");
    let locals = vec![crate_path];

    save_cache_local(&vendor, &locals).unwrap();
    assert!(is_cached_local(&vendor, &locals).unwrap());

    // Bump the lockfile
    std::fs::write(&lockfile, "version = 4").unwrap();

    assert!(
        is_cached_local(&vendor, &locals).unwrap(),
        "local cache should remain valid when only Cargo.lock changes"
    );
}

#[test]
fn external_cache_invalidates_on_lockfile_change() {
    let (_tmp, lockfile, vendor) = setup();

    save_cache_external(&lockfile, &[], &vendor).unwrap();
    assert!(is_cached_external(&lockfile, &[], &vendor).unwrap());

    std::fs::write(&lockfile, "version = 99").unwrap();

    assert!(
        !is_cached_external(&lockfile, &[], &vendor).unwrap(),
        "external cache should invalidate when Cargo.lock changes"
    );
}

#[test]
fn external_cache_miss_when_file_absent() {
    let (_tmp, lockfile, vendor) = setup();
    // Never saved — must be a miss.
    assert!(!is_cached_external(&lockfile, &[], &vendor).unwrap());
}

#[test]
fn local_cache_miss_when_file_absent() {
    let (tmp, _lockfile, vendor) = setup();
    let locals = vec![tmp.path().join("libx")];
    assert!(!is_cached_local(&vendor, &locals).unwrap());
}

// endregion
