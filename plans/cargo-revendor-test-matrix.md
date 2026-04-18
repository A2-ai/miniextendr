# cargo-revendor test matrix

Gap analysis vs the existing 23 integration tests in `cargo-revendor/tests/integration.rs`, plus a harness proposal for elaborate multi-workspace scenarios.

## Covered today (23 tests, all `#[ignore]` network-gated)

| Feature | Test |
|---|---|
| Error: missing manifest | `error_missing_manifest` |
| Simple crates-io dep | `simple_crate_with_cratesio_dep` |
| Workspace sibling path dep | `workspace_sibling_dep` |
| Transitive path deps (A→B→C) | `path_dep_chain_a_to_b_to_c`, `workspace_transitive_local_deps` |
| `[patch.crates-io]` preserved | `patch_cratesio_pattern` |
| Monorepo w/ nested rpkg-shaped crate | `monorepo_nested_rpkg` |
| Workspace `version = workspace = true` inheritance | `workspace_version_inheritance`, `workspace_dep_inheritance` |
| `[build-dependencies]` vendored | `build_dependencies_vendored` |
| Strip tests/benches/examples | `stripping_removes_test_bench_dirs`, `strip_tests_only`, `no_strip_preserves_directories` |
| Intra-vendor path rewriting | `path_rewriting_inline_and_section` |
| Broken crate → direct-copy fallback | `broken_crate_still_packages` |
| Raw path deps get `version = "*"` | `raw_path_deps_auto_versioned` |
| JSON output shape | `json_output_structure` |
| Cache hit (no-op on 2nd run) | `caching_skips_second_run` |
| `--force` bypasses cache | `force_bypasses_cache` |
| No external deps | `empty_external_deps` |
| Generates `.cargo/config.toml` + strips lockfile checksums | `generates_cargo_config_and_stripped_lockfile` |
| Optional deps | `optional_dependencies` |
| Features on path deps | `features_on_path_deps` |
| Output dir clobbered on rerun | `output_dir_replaced_cleanly` |

Also unit-tested elsewhere: cache-key hashing (`cache.rs`), freeze patch-sort determinism (`vendor.rs::freeze_manifest_*`), strip mechanics (`strip.rs`), verify lock↔vendor and tarball↔vendor (`verify.rs`).

## Missing coverage

### Git dependencies (zero coverage)

cargo-revendor claims git-dep support via delegation to `cargo vendor`, but **no test exercises `source = "git+..."`**.

- [ ] **G1** — `git_dep_from_local_bare_repo`: local bare git repo as source, verify `vendor/<crate>/` materializes (no checksum, correct version, clean `Cargo.toml`).
- [ ] **G2** — `git_dep_pinned_by_rev`: `git = "...", rev = "abc123"` pin survives vendoring; lockfile preserves the rev.
- [ ] **G3** — `git_dep_with_branch_and_tag`: separate pins for branch/tag variants; no divergence.
- [ ] **G4** — `git_monorepo_multi_crate`: single git repo providing 2+ crates, only one referenced by caller — vendor extracts just the referenced one.
- [ ] **G5** — `git_dep_overridden_by_patch`: `[patch."https://github.com/..."]` entries make the patched git source resolve from a workspace path.

### `--freeze` end-to-end

Unit-tested in `vendor.rs`, not exercised through an integration test.

- [ ] **F1** — `freeze_rewrites_manifest_to_vendor_paths`: after freeze, every `[dependencies]` entry resolves from `vendor/<name>/`; no git or version-only entries remain.
- [ ] **F2** — `freeze_regenerates_lockfile_offline`: `cargo build --offline` from the frozen state succeeds.
- [ ] **F3** — `freeze_sorts_patch_crates_io_deterministically`: regression for #206 at the integration level.

### `--verify` end-to-end

Added in #217, only unit-tested.

- [ ] **V1** — `verify_clean_after_vendor`: full `vendor` flow + immediate `--verify` passes.
- [ ] **V2** — `verify_catches_lock_vendor_mismatch`: hand-edit `Cargo.lock` to a version not present in `vendor/`; verify fails with actionable message. This is the #157 failure shape.
- [ ] **V3** — `verify_catches_stale_tarball`: modify a file inside `vendor/` without regenerating the tarball; verify fails.
- [ ] **V4** — `verify_ignores_revendor_cache_byproduct`: after `--compress`, a `.revendor-cache` exists only in `vendor/` (written post-compress) — verify must not flag it. Regression for #218.
- [ ] **V5** — `verify_only_skips_vendoring`: `--verify` without prior `vendor/` errors cleanly; does NOT try to re-vendor.

### `--compress` end-to-end

- [ ] **C1** — `compress_roundtrip_matches_vendor`: compress, extract to a fresh temp dir, diff against original vendor tree — bit-for-bit identical (modulo `.revendor-cache`).
- [ ] **C2** — `compress_blank_md_zeroes_markdown`: with `--blank-md`, every `.md` in the tarball is empty; without the flag, contents preserved.
- [ ] **C3** — `compress_suppresses_macos_xattrs`: no `._*` entries, no `LIBARCHIVE.xattr.*` PAX headers.

### Multi-workspace shared vendor (the scenario the user called out)

**Current behavior**: cargo-revendor clobbers `--output` on each run (step 8 in `main.rs`). Running against two disjoint workspaces with the same `--output` erases the first.

Three test shapes, each illuminates a different aspect:

- [ ] **M1 — single-workspace with two locked versions**: one workspace, where two internal crates pin incompatible versions of a third-party dep (e.g. `rayon = "1.10"` and `rayon = "1.12"`). Cargo resolves both. Verify `vendor/` contains both `rayon-1.10.0/` and `rayon-1.12.0/` under `--versioned-dirs` (once #215 lands) or a mix of flat + versioned under today's behavior.

- [ ] **M2 — disjoint workspaces, sequential vendor, same output path (current behavior)**: vendor ws1 → ws2 with the same `--output`; assert ws2's vendor dir replaces ws1's (regression for the current "clobbers" behavior, documented as intentional). Pairs with…

- [ ] **M3 — disjoint workspaces sharing vendor via `cargo vendor --sync`** (*feature gap*): cargo-revendor currently has no `--sync` flag. Real monorepo scenarios — e.g. an rpkg workspace and a benchmarks workspace that want one vendor/ — need it. Propose: `cargo revendor --manifest-path ws1/Cargo.toml --sync ws2/Cargo.toml --output vendor/`. **File as new issue**; test covers the eventual implementation.

### Diagnostics

- [ ] **D1** — `verbosity_levels`: `-v` / `-vv` / `-vvv` stderr deltas are non-trivial.
- [ ] **D2** — `source_marker_content`: `.vendor-source` contents when `--source-root` is explicit vs auto-detected.

### Edge cases

- [ ] **E1** — `dev_dependency_on_local_path_crate`: should still strip `[dev-dependencies]` sections.
- [ ] **E2** — `path_dep_outside_source_root_errors`: a path dep pointing to `../../escape-hatch` should fail or warn predictably.
- [ ] **E3** — `cfg_gated_dependencies`: `[target.'cfg(unix)'.dependencies]` correctly vendored.
- [ ] **E4** — `rename_deps`: `foo = { package = "bar", ... }` — rename survives vendoring.
- [ ] **E5** — `no_default_features_deps`: vendored crate's Cargo.toml preserves `default-features = false` on deps.

## Harness proposal

The existing helpers (`create_simple_crate`, `create_workspace`, `create_monorepo`) handle single-shape scenarios well. The proposals above need three new primitives:

```rust
// --- git source materialized locally, no network ---
struct LocalGitRepo {
    path: PathBuf,           // bare repo
    rev: String,             // first-commit OID
}

fn create_local_git_crate(name: &str, cargo_toml: &str, lib_rs: &str) -> LocalGitRepo {
    // 1. Create a temp dir, write crate contents.
    // 2. `git init && git add . && git commit -m init`.
    // 3. Clone to a sibling `.bare` with `git clone --bare`.
    // 4. Return bare path + HEAD OID.
}

// Callers reference it as:
//   foo = { git = "file:///path/to/bare.git" }
```

```rust
// --- multi-workspace orchestration for M1/M2/M3 ---
struct MultiWorkspace {
    root: TempDir,
    workspaces: Vec<PathBuf>, // ordered
}

impl MultiWorkspace {
    fn add(&mut self, name: &str, manifest: &str) -> &Path { ... }
    fn vendor_shared(&self, output: &Path, extra_args: &[&str]) { ... }
}
```

```rust
// --- round-trip tar utility for C1 ---
fn extract_tarball(xz: &Path, into: &Path) { /* `tar -xJf` */ }
fn diff_trees(a: &Path, b: &Path) -> Vec<TreeDiff> { /* recursive file-set + byte diff */ }
```

All three live in a shared `tests/common/mod.rs` (new) so both the existing tests and the new scenarios benefit. The existing tests would opt in incrementally — no big-bang refactor.

## Suggested rollout

1. **PR 1 — harness**: extract current helpers into `tests/common/mod.rs`; add `LocalGitRepo` + `extract_tarball` + `diff_trees`. No new test coverage yet.
2. **PR 2 — git coverage**: implement G1–G5 on top of the harness. This is the biggest correctness-relevant gap.
3. **PR 3 — `--verify` / `--freeze` / `--compress` end-to-end**: V1–V5, F1–F3, C1–C3.
4. **PR 4 — multi-workspace**: M1 + M2 first (current behavior). M3 blocks on a design decision about `--sync` support; file as a separate feature issue with the test matrix as acceptance criteria.
5. **PR 5 — edge cases + diagnostics**: E1–E5, D1–D2.

Each PR is self-contained, adds network-gated tests (matching existing convention), and can land independently.

## Explicit non-goals

- Performance benchmarks (that's what `criterion` would be for; separate concern).
- Cross-platform harness — all existing tests assume a POSIX shell for `tar`, `git`, etc. Windows coverage via CI only.
- Proptest/fuzzing. The test matrix targets concrete scenarios; randomization yields marginal value against cargo vendor's well-defined behavior.
