# Plan: test harness helpers tolerate versioned-dirs layout (#250)

Fix the flat-path assumptions that PR #239 broke. Mechanical but touches every
integration test file — needs a thorough sweep.

## Problem

`cargo-revendor/tests/common/mod.rs:237–273` and several call sites probe
`vendor/<name>/` (flat) instead of `vendor/<name>-<version>/` (versioned,
default since #239). Every `#[ignore]` network integration test asserting on
registry crates is latently broken; they only pass today because the `--ignored`
gate keeps them out of `just revendor-test`.

## Files to change

- `cargo-revendor/tests/common/mod.rs` — add a `vendor_dir_for()` helper;
  rewrite `assert_vendor_has`, `read_vendor_toml`, `assert_empty_checksum` to
  use it.
- `cargo-revendor/tests/integration.rs` — replace all hardcoded
  `vendor.join("<name>")` with helper calls.
- `cargo-revendor/tests/verify_freeze_compress.rs` — line 126 specifically, plus
  any other occurrences.
- `cargo-revendor/tests/multi_workspace.rs` — sweep.
- `cargo-revendor/tests/git_deps.rs` — sweep.
- `cargo-revendor/tests/edge_cases_diagnostics.rs` — sweep.

Grep gate before declaring done:
```bash
rg -n 'vendor\.join\("[a-z_]+"\)' cargo-revendor/tests/
```
Should return zero hits after the fix (every hardcoded-name join replaced by
the helper).

## Helper design

```rust
// In tests/common/mod.rs
pub fn vendor_dir_for(vendor: &Path, name: &str, version: Option<&str>) -> PathBuf {
    // 1. Exact versioned match if version given.
    if let Some(v) = version {
        let versioned = vendor.join(format!("{name}-{v}"));
        if versioned.is_dir() {
            return versioned;
        }
    }
    // 2. Glob for any <name>-* match (single result expected).
    if let Ok(entries) = std::fs::read_dir(vendor) {
        let prefix = format!("{name}-");
        let mut matches: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with(&prefix))
                    .unwrap_or(false)
            })
            .collect();
        if matches.len() == 1 {
            return matches.pop().unwrap();
        }
        if matches.len() > 1 {
            panic!(
                "vendor_dir_for({name}): ambiguous — multiple versioned matches: {matches:?}"
            );
        }
    }
    // 3. Flat fallback.
    let flat = vendor.join(name);
    if flat.is_dir() {
        return flat;
    }
    panic!("vendor_dir_for({name}): no matching directory in {}", vendor.display());
}
```

Pattern mirrors `cargo-revendor/src/verify.rs:105–115` (`verify_lock_matches_vendor`)
and `minirextendr/R/vendor.R:732–737` (`add_vendor_patches`) — both already do
probe-versioned-then-flat. Test code should inherit the same tolerance.

## Implementation steps

1. Add `vendor_dir_for()` to `tests/common/mod.rs` after existing helpers.
2. Update `assert_vendor_has(vendor, name)` to call
   `vendor_dir_for(vendor, name, None).exists()`.
3. Update `read_vendor_toml` to resolve the crate dir via `vendor_dir_for`
   before reading `Cargo.toml`.
4. Update `assert_empty_checksum` similarly.
5. Grep-and-replace every `vendor.join("<name>")` in tests/ with
   `vendor_dir_for(&vendor, "<name>", None)` (or pass the version if the test
   asserts on a specific one).
6. Remove/replace `PathBuf::from(format!("vendor/{name}/...",...))` variants.
7. Run `cargo test --manifest-path cargo-revendor/Cargo.toml -- --ignored`
   (requires network). Every previously-ignored test must pass.
8. Run `cargo test --manifest-path cargo-revendor/Cargo.toml -- --ignored`
   under `--flat-dirs` as well: add a test-only option or env var that forces
   flat-dir regeneration, or run one ignored test manually with `--flat-dirs`
   to confirm the fallback path still works.

## Verification

```bash
cd cargo-revendor
cargo test -- --ignored 2>&1 | tee /tmp/revendor-ignored.log
# expect: all green
rg -n 'vendor\.join\("[a-z_]+"\)' tests/ && exit 1  # should be empty
just revendor-test                   # non-ignored tests still green
```

## Out of scope

- Fixing the tests' reliance on the network — keep `#[ignore]` gate
- Adding new test cases — this is a pure helper refactor

## Risk

Low. Mechanical change; helper gives the same or better behavior than
hardcoded joins. Main risk: missing a hardcoded site. The rg gate above
catches those.

## PR expectations

- Branch: `fix/issue-250-test-harness-versioned-dirs`
- No merge — CR review
