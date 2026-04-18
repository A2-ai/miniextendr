# Plan: `--freeze` documents non-local git dep constraint; adds `--strict-freeze` (#252)

Scope: approach A from the review (document + strict-mode opt-in). Approach B
(rewrite external git deps to vendor paths) is out of scope — file as follow-up
if reviewer requests.

## Problem

`cargo-revendor/src/vendor.rs:709–718` in `freeze_manifest` rewrites deps only
when they match `local_pkgs`. External (non-workspace) `git =` deps are left
untouched. The README claims `--freeze` makes `cargo build --offline` work
with only `vendor/`, which is true for miniextendr's configure.ac flow (it
sets up source-replacement) but false for any other user invoking
`cargo revendor --freeze` standalone.

## Files to change

- `cargo-revendor/README.md` — clarify the `--freeze` contract.
- `cargo-revendor/src/main.rs` — add `--strict-freeze` flag.
- `cargo-revendor/src/vendor.rs` — `freeze_manifest` gains an optional
  strict-mode check that errors on external git deps.
- `cargo-revendor/tests/verify_freeze_compress.rs` — one regression test.

## Implementation steps

1. **README update**: in the `--freeze` section, add:
   > `--freeze` rewrites path deps (`path = "..."`) to vendor paths. External
   > git deps (`git = "..."`) are preserved as `git =` entries and rely on
   > `.cargo/config.toml` source replacement (which `cargo revendor` writes
   > to `vendor/.cargo-config.toml`) for offline build. Users who want
   > `cargo build --offline` to succeed with the frozen manifest alone should
   > either copy `vendor/.cargo-config.toml` to `.cargo/config.toml` under
   > their build dir or pass `--strict-freeze` to fail-fast on this case.

2. **Add `--strict-freeze` CLI flag** in `main.rs`:
   ```rust
   /// Error if `--freeze` cannot fully resolve the manifest to vendor/ alone.
   /// Catches external git deps that would silently require a .cargo/config.toml.
   #[arg(long)]
   strict_freeze: bool,
   ```
   Plumb to `freeze_manifest` as a parameter.

3. **Implement the check in `freeze_manifest`**: after rewriting local-pkg
   deps, walk every `[dependencies]`, `[target.*.dependencies]`, and
   `[build-dependencies]` table. For each entry with a `git` field that
   **wasn't** matched by a local_pkg rewrite, collect into a `Vec<String>`
   (name of the unresolved git dep).

4. If the collection is non-empty AND `strict_freeze`, return:
   ```
   error: --strict-freeze: external git deps remain after freeze:
     - foo (git = "https://github.com/bar/foo")
     - baz (git = "https://github.com/qux/baz", rev = "abc123")
   Either drop --strict-freeze, or vendor these git deps first.
   ```

5. If non-empty and NOT `strict_freeze`, emit a warning at verbosity >= 1
   listing the unresolved git deps so the user knows their manifest still
   requires `.cargo/config.toml` source replacement.

6. **Test**: `tests/verify_freeze_compress.rs` — add `F4_strict_freeze_errors_on_external_git`:
   - construct a workspace whose Cargo.toml has `foo = { git = "https://..." }`
   - run `cargo revendor --freeze --strict-freeze ...`
   - assert exit code != 0 and stderr contains "external git deps remain"
   - opposite test: run WITHOUT `--strict-freeze`; expect success, warning in
     stderr at `-v`

## Verification

```bash
just revendor-test
cd cargo-revendor && cargo test --test verify_freeze_compress strict_freeze
# manual: existing miniextendr workflow still works (no external git deps →
# strict_freeze path is never exercised by internal use)
```

## Out of scope

- **Approach B**: rewriting external git deps to vendor paths. Requires
  resolved commit → vendor-dir lookup, more test surface, and changes
  user-visible manifest semantics. File as follow-up issue if pursued.
- Changing the `--freeze` default behavior — non-strict stays default for
  backwards-compat with current miniextendr rpkg flow.

## Risk

Low. Pure addition; no behavior change unless `--strict-freeze` passed. Read
path (freeze_manifest) already walks these tables for local-pkg rewriting,
so the extra collection step is incremental.

## PR expectations

- Branch: `fix/issue-252-strict-freeze`
- No merge — CR review
- If reviewer pushes back, can be split into (a) docs-only PR, (b)
  strict-freeze flag PR
