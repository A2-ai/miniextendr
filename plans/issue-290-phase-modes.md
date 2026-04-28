+++
title = "cargo-revendor: --external-only / --local-only phase modes (#290)"
description = "Split vendoring pipeline into two independent phase modes for CI cacheability"
+++

# cargo-revendor phase modes

Closes #290.

---

## Context

`cargo revendor` does two things in one pass:
1. **External deps** (crates.io/git) — expensive, changes only when `Cargo.lock` changes
2. **Local workspace crates** — cheap, changes on every dev commit

Decoupling them lets CI cache the external layer on a `Cargo.lock` key, making
most PR runs a 2-second `--local-only` step instead of a 60-second full vendor.

The issue spec is detailed and well-specified at:
https://github.com/A2-ai/miniextendr/issues/290

---

## Key design decisions

### CLI

Add two mutually exclusive flags to `Cli` in `cargo-revendor/src/main.rs`:

```rust
/// Vendor external (crates.io/git) dependencies only.
/// Writes vendor/<name>-<version>/ dirs, never touches local crate dirs.
/// Incompatible with --freeze, --compress, --source-marker, --blank-md.
#[arg(long, conflicts_with = "local_only")]
external_only: bool,

/// Vendor local workspace crates only.
/// Writes vendor/<name>/ dirs (flat, single-version), never touches external dirs.
/// Requires externals to already be on disk for --freeze/--compress/--source-marker.
#[arg(long, conflicts_with = "external_only")]
local_only: bool,
```

`Mode` enum (private to main.rs):
```rust
#[derive(Clone, Copy, PartialEq)]
enum Mode { Full, ExternalOnly, LocalOnly }
```

Derive from `cli.external_only` / `cli.local_only`.

### Flag compatibility validation

Implement `validate_flag_compatibility(cli: &Cli, mode: Mode) -> Result<()>`:
- `ExternalOnly` + `--freeze` → error
- `ExternalOnly` + `--compress` → error
- `ExternalOnly` + `--source-marker` → error
- `ExternalOnly` + `--blank-md` → error
- `ExternalOnly` + `--strict-freeze` → error
- `LocalOnly` + `--freeze` → ok only if `vendor/` already has external dirs on disk; else error
- `LocalOnly` + `--compress` → same gating as --freeze
- `LocalOnly` + `--source-marker` → same
- `LocalOnly` + `--blank-md` → same
- All other flags: ✓ in all modes

"Externals on disk" check: `output.exists() && output.read_dir()?.any(|e| e?.file_name().to_string_lossy().contains('-'))`.
Versioned dirs always have a `-<version>` suffix; local crates are flat (`vendor/<name>/` no dash).
A simpler check: does `.revendor-cache-external` exist in output? If yes, externals were previously vendored.

Use the cache file check: `output.join(".revendor-cache-external").exists()`.

### Cache files

Add to `cache.rs`:

```rust
pub const CACHE_FILE_EXTERNAL: &str = ".revendor-cache-external";
pub const CACHE_FILE_LOCAL: &str = ".revendor-cache-local";
```

Keep `CACHE_FILE = ".revendor-cache"` for legacy compat — full mode still writes it.

New function signatures:
```rust
pub fn is_cached_external(lockfile, sync_manifests, vendor_dir) -> Result<bool>
pub fn is_cached_local(vendor_dir, local_crate_paths) -> Result<bool>
pub fn save_cache_external(lockfile, sync_manifests, vendor_dir) -> Result<()>
pub fn save_cache_local(vendor_dir, local_crate_paths) -> Result<()>
```

`is_cached_external` hashes: `Cargo.lock` + `Cargo.toml` + sync manifests/lockfiles.
`is_cached_local` hashes: each local crate's source tree only.

Full mode continues writing the legacy `.revendor-cache` (union of both hashes, or compute as before) for compatibility. Also writes both new files. Migration: if legacy `.revendor-cache` exists but new files don't, treat as miss for both new files.

### Step 8: merge-copy for phase modes

Current step 8 (full mode):
```rust
std::fs::remove_dir_all(&output)?;
std::fs::rename(&vendor_staging, &output)?;
```

Phase modes must NOT clobber the other phase's dirs. Implement `merge_copy_vendor(staging, output, mode)`:
- Collect dir names present in staging
- For each dir in staging: remove `output/<name>` if it exists, then copy/rename `staging/<name>` to `output/<name>`
- Dirs in output NOT in staging: untouched

```rust
fn merge_copy_vendor(staging: &Path, output: &Path, v: Verbosity) -> Result<()> {
    std::fs::create_dir_all(output)?;
    for entry in std::fs::read_dir(staging)? {
        let entry = entry?;
        let name = entry.file_name();
        let dst = output.join(&name);
        if dst.exists() {
            std::fs::remove_dir_all(&dst)?;
        }
        std::fs::rename(entry.path(), &dst)
            .or_else(|_| copy_dir_recursive(&entry.path(), &dst))?;
    }
    Ok(())
}
```

Full mode keeps the current `remove_dir_all + rename` (fast path, nothing to merge against).

### Pipeline mapping

**`run_external_only` steps:**
1b. Load metadata + partition (no git_overrides needed — skip source_root bootstrap)
3. `cargo vendor` for external deps into staging
5. Strip external dirs in staging (strip_cfg)
5.5. `strip_vendor_path_deps` on staging
7. `clear_checksums` on staging
8. `merge_copy_vendor(staging, output)` — only touches `<name>-<version>/` dirs
9. `generate_cargo_config` (rescans all of output, so includes any local dirs already there)
10. `strip_lock_checksums` (if --compress given — but --compress is gated; skip)
14. `save_cache_external`

**`run_local_only` steps:**
1a. Bootstrap-seed from source_root (needed for metadata resolution)
1b. Load metadata + partition (WITH git_overrides = source_root_members)
2. `package_local_crates`
4. `extract_crate_archive` for each packaged local crate
5. Strip local dirs in staging (strip_cfg)
6. `rewrite_local_path_deps` on staging
7. `clear_checksums` on staging
8. `merge_copy_vendor(staging, output)` — only touches `<name>/` (flat) dirs
9. `generate_cargo_config` (rescans all of output)
14. `save_cache_local`

Skip: cargo vendor (step 3), strip_vendor_path_deps (5.5), strip_lock_checksums (10), source_marker (11), freeze (12), compress (13).

**`run_full` (existing behavior):**
Keep the current 14-step sequence in main(). At step 0 (cache check), check both new cache files if available. At step 14, write all three cache files.

Optionally: add early exit if `is_cached_external` AND `is_cached_local` both fresh (replaces current `is_cached` check). For backward compat, also check legacy `.revendor-cache`.

### Code structure

Refactor `main()` into:
```rust
fn main() -> Result<()> {
    let cli = Cli::parse();
    let mode = if cli.external_only { Mode::ExternalOnly }
               else if cli.local_only { Mode::LocalOnly }
               else { Mode::Full };
    validate_flag_compatibility(&cli, mode, &output)?;
    match mode {
        Mode::Full => run_full(&cli, ...),
        Mode::ExternalOnly => run_external_only(&cli, ...),
        Mode::LocalOnly => run_local_only(&cli, ...),
    }
}
```

Each `run_*` function is self-contained with its own numbered step sequence.
Shared code stays in `vendor.rs`, `strip.rs`, `package.rs`, `cache.rs`.

---

## Files to modify

| File | Change |
|---|---|
| `cargo-revendor/src/main.rs` | Add CLI flags, `Mode` enum, `validate_flag_compatibility`, refactor main into `run_full` + `run_external_only` + `run_local_only`, add `merge_copy_vendor` |
| `cargo-revendor/src/cache.rs` | Add phase-specific cache functions: `is_cached_external`, `is_cached_local`, `save_cache_external`, `save_cache_local` |

No changes needed to `vendor.rs`, `strip.rs`, `metadata.rs`, `package.rs` — they're pure helpers.

---

## Tests

### Unit tests (cache.rs)

- `external_cache_ignores_local_source_edits`: save external cache, edit a local crate source file, assert `is_cached_external` still returns true
- `local_cache_ignores_lockfile_changes`: save local cache, edit Cargo.lock, assert `is_cached_local` still returns true
- `external_cache_invalidates_on_lockfile_change`: edit Cargo.lock, assert `is_cached_external` returns false

### Unit tests (main.rs)

- `validate_flag_compatibility_external_only_freeze_errors`
- `validate_flag_compatibility_external_only_compress_errors`
- `validate_flag_compatibility_local_only_flags_ok_with_externals_present`: create `.revendor-cache-external`, assert local-only + compress ok
- `validate_flag_compatibility_local_only_compress_errors_without_externals`

### Integration tests (new file: `tests/phase_modes.rs`)

- `external_only_produces_only_versioned_dirs`: fresh workspace, `--external-only` → assert vendor/ has only `<name>-<version>/` dirs, no flat dirs, `.revendor-cache-external` written, no `.revendor-cache-local`
- `local_only_produces_only_flat_dirs`: fresh workspace, first run `--external-only`, then `--local-only` → assert new flat dirs appear, versioned dirs untouched
- `phase_modes_compose_to_full`: `--external-only` then `--local-only` → vendor/ tree equals what `full` would produce
- `external_only_does_not_clobber_local_dirs`: `--external-only` after a `--local-only` run → local dirs in vendor/ untouched
- `flag_compat_external_only_freeze_exits_nonzero`: `--external-only --freeze` → exits 1 with message
- `flag_compat_local_only_compress_without_externals_exits_nonzero`: `--local-only --compress` on empty vendor/ → exits 1
- `external_cache_hit_skips_cargo_vendor`: second `--external-only` with unchanged lockfile → no-op (check by timing or by asserting vendor mtime unchanged)

These integration tests follow the same pattern as `tests/integration.rs` (using `tempfile`, creating toy Cargo workspaces). Most will be `#[ignore]` (requires network or is slow) with a few that can run without cargo available.

---

## Build/test commands

```bash
just revendor-build    # verify compilation
just revendor-test     # all 57+ tests pass
```

---

## Commit

Single commit on `feat/issue-290-phase-modes`:
```
feat(cargo-revendor): --external-only / --local-only phase modes

Closes #290
```

---

## Gotchas

- Step 8 for phase modes uses merge-copy (not rename). If `output/` doesn't exist yet, create it first.
- `generate_cargo_config` in step 9 rescans ALL of `output/` — so after `--external-only`, it won't see local dirs; after `--local-only`, it will see both. The config it generates is always correct for the CURRENT state of vendor/, which is the desired behavior.
- `strip_vendor_path_deps` (step 5.5) is only needed after `cargo vendor` (step 3) which is external-only. Local-only skips it.
- Legacy `.revendor-cache` migration: on first run after upgrade, both new cache files are written; old file is left as-is (don't delete it, existing CI might check its mtime).
- `--sync` manifests: external cache includes their lockfiles in the hash; local cache doesn't (sync manifests are external dep graphs).
