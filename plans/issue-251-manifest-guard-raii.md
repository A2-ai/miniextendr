# Plan: RAII `ManifestGuard` for workspace Cargo.toml mutations (#251)

Fix panic/SIGINT-unsafe manifest mutation in two hot paths. Small structural
refactor; drop-in replacement for both affected call sites.

## Problem

`cargo-revendor/src/vendor.rs:44–71` (`run_cargo_vendor`) and
`cargo-revendor/src/package.rs:60–91` (`package_local_crates`) both:

1. Read the workspace `Cargo.toml`.
2. Write a mutated version with `[patch.crates-io]` appended.
3. Shell out to cargo.
4. Write the original back.

If cargo panics, SIGINT lands, or the process is killed between (2) and (4),
the workspace Cargo.toml is left in a mutated state pointing at paths that
don't exist yet. The user then has to `git checkout Cargo.toml` manually.

## Files to change

- `cargo-revendor/src/vendor.rs` — replace inline restore logic with guard.
- `cargo-revendor/src/package.rs` — same.
- Optionally `cargo-revendor/src/lib.rs` or a new `manifest_guard.rs` — host
  the guard type.

## Guard design

```rust
// cargo-revendor/src/manifest_guard.rs  (new file)
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// RAII guard that restores a manifest's original bytes on drop.
///
/// Construct before mutating; the original is restored unconditionally when
/// the guard goes out of scope — including on panic/SIGINT unwind. Does NOT
/// restore on SIGKILL or std::process::abort; that residual gap is accepted.
pub struct ManifestGuard {
    path: PathBuf,
    original: Vec<u8>,
}

impl ManifestGuard {
    pub fn snapshot(path: &Path) -> Result<Self> {
        let original = std::fs::read(path)
            .with_context(|| format!("failed to snapshot {}", path.display()))?;
        Ok(Self {
            path: path.to_path_buf(),
            original,
        })
    }
}

impl Drop for ManifestGuard {
    fn drop(&mut self) {
        // Best-effort restore. Ignore errors — if we can't write the manifest
        // back, the alternative (panic in Drop) is worse.
        let _ = std::fs::write(&self.path, &self.original);
    }
}
```

Add `mod manifest_guard;` + `pub use manifest_guard::ManifestGuard;` to
lib.rs (if one exists) or main.rs.

## Implementation steps

1. Create `cargo-revendor/src/manifest_guard.rs` with the struct above.
2. Wire it up in `src/lib.rs` (or `src/main.rs` if no lib exists — check
   `Cargo.toml` for `[lib]` presence).
3. In `vendor.rs:run_cargo_vendor`:
   - Replace lines 44–46 + 70–71 with a `ManifestGuard::snapshot(&ws_manifest)?`
     early, then the explicit `fs::read_to_string` and `fs::write(&ws_manifest, &ws_original)`
     restore lines go away.
   - Still write the patched content with `fs::write(&ws_manifest, ...)`; the
     guard only owns the restore path.
4. Same pattern in `package.rs:package_local_crates` for both the workspace
   manifest AND the inner package manifest (if the current code restores
   both).
5. Confirm behavior: on success, guard runs last (after `bail!` for non-zero
   cargo exit); on error (cargo returns non-zero), `?` propagates and guard
   runs during unwind. On panic, guard runs during unwind.
6. Test — add an integration test that triggers a panic inside the vendor flow
   and asserts the workspace Cargo.toml is restored:

```rust
// cargo-revendor/tests/manifest_guard.rs  (new)
use cargo_revendor::manifest_guard::ManifestGuard;

#[test]
fn guard_restores_on_panic() {
    let tmp = tempfile::tempdir().unwrap();
    let manifest = tmp.path().join("Cargo.toml");
    std::fs::write(&manifest, b"[workspace]\n").unwrap();

    let result = std::panic::catch_unwind(|| {
        let _g = ManifestGuard::snapshot(&manifest).unwrap();
        std::fs::write(&manifest, b"mutated").unwrap();
        panic!("boom");
    });

    assert!(result.is_err());
    assert_eq!(std::fs::read(&manifest).unwrap(), b"[workspace]\n");
}
```

Also add one for the success path (guard still restores) and one for the
early-return path.

## Verification

```bash
just revendor-test                                          # all green
cargo test --manifest-path cargo-revendor/Cargo.toml manifest_guard  # new tests pass
# Manual: in a scratch workspace, run `cargo revendor ...`, send Ctrl-C
# during the cargo vendor phase, verify `git status` shows no changes.
```

## Out of scope

- Catching SIGKILL / abort — accept as unfixable in Rust's Drop model
- Guard for files other than Cargo.toml — not needed here
- Backing up to a sidecar file for extra safety — simpler is better

## Risk

Low. Guard makes behavior strictly better on failure paths. Success paths
behave identically (guard's Drop writes the original bytes, which already
matched the post-restore state).

## PR expectations

- Branch: `fix/issue-251-manifest-guard`
- No merge — CR review
