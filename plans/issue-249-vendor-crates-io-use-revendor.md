# Plan: `vendor_crates_io()` uses `cargo revendor`; drop redundant tar (#249)

Critical user-facing bug — S1.1 in the review. Also closes S2.4 (macOS tar xattrs)
because the fix replaces the offending `system2("tar", ...)` call entirely.

## Problem

`minirextendr/R/workflow.R:462–464` in `vendor_crates_io()` shells out to plain
`cargo vendor`. That call skips all cargo-revendor behavior: no workspace/path
crate extraction, no `--strip-all`, no `--freeze`, no `--compress`, no
`.vendor-source` marker. Users invoking the public `miniextendr_vendor()` get a
`vendor/` layout that fails to compile on CRAN for any package with workspace
path-deps.

The follow-up tar step in `miniextendr_vendor()` (workflow.R:212–215) also lacks
`COPYFILE_DISABLE=1` + `--no-xattrs`, contaminating macOS tarballs with
AppleDouble `._*` files.

## Files to change

- `minirextendr/R/workflow.R` — `vendor_crates_io()` body + the surrounding
  `miniextendr_vendor()` that calls it + its tar step.
- `minirextendr/R/vendor.R` — if `patch_cargo_toml` (lines 225–255) becomes dead
  code once we delegate to cargo-revendor, remove it and its callers. If it's
  still needed, leave for #253.
- `minirextendr/tests/testthat/` — add a regression test hitting a workspace
  with path deps; assert tarball contains vendored workspace crates.
- `minirextendr/NEWS.md` — one-line user-facing entry.

## Implementation order

1. **Read context**: `vendor_crates_io()` body + `miniextendr_vendor()` body in
   workflow.R; `patch_cargo_toml` in vendor.R; any other `cargo vendor`
   shell-outs across `minirextendr/R/` (grep `cargo.*vendor` — standalone
   `cargo vendor` invocation anywhere is a regression hazard).
2. **Replace the shell-out**: rewrite `vendor_crates_io()` to call
   `cargo revendor --strip-all --freeze --compress <out-tarball> --blank-md --source-marker -v`.
   Use `system2("cargo", c("revendor", ...))`, capture stdout/stderr with
   `stdout = TRUE, stderr = TRUE`, error on non-zero exit.
3. **Remove the follow-up tar step** in `miniextendr_vendor()` — cargo-revendor
   produces the tarball via `--compress`, so the manual `system2("tar", ...)`
   at workflow.R:212–215 is redundant. Delete it; delete the
   `COPYFILE_DISABLE` concern along with it (S2.4 resolved as byproduct).
4. **Check whether `patch_cargo_toml` is still called** after step 2. If no
   remaining callers, delete the function (one-liner follow-up, also helps
   close #253). If still called, leave it and note in the PR body.
5. **Flag/argument plumbing**: `vendor_crates_io()` signature likely passes
   `path`, `output_tarball`, and a verbosity arg. Keep the signature identical
   — only the body changes. Callers should not need updates.
6. **Verify cargo-revendor is installed**: before invoking, check via
   `Sys.which("cargo-revendor")` or `system2("cargo", c("revendor", "--help"))`
   and error with a clear "install with: cargo install --path cargo-revendor"
   message if missing.
7. **Add a regression test**: `minirextendr/tests/testthat/test-vendor.R` (new
   file or extend existing). Fixture: a minimal scaffolded monorepo project
   with one workspace crate. Call `miniextendr_vendor()`, assert:
   - produced tarball exists
   - `tar -tJf <out> | grep '<workspace-crate>/Cargo.toml'` — confirms
     workspace crate was extracted into the tarball
   - on macOS, `tar -tJf <out> | grep '^\\./\\._'` finds nothing
   Gate under `skip_e2e()` / `skip_on_ci()` if it requires a real cargo
   toolchain — use the same gating as the existing template smoke tests.
8. **NEWS entry**: `* `miniextendr_vendor()` now uses `cargo revendor` instead
   of `cargo vendor`, so workspace path-deps are correctly extracted into the
   produced tarball. Closes #249.`

## Verification

```bash
just minirextendr-test              # existing suite still green
NOT_CRAN=true just minirextendr-test # also green in dev mode
# manual: scaffold a test project via minirextendr::create_*, run
#   miniextendr_vendor(path = "."), inspect the tarball
```

## Out of scope

- Broader vendor.R cleanup (`patch_cargo_toml`) — covered by #253
- Adding new flags to cargo-revendor — out of scope here; we use existing ones
- Upgrading vendor.tar.xz compression level — not this issue

## Risk

Medium. `miniextendr_vendor()` is the primary user-facing CRAN-prep entry
point. Any regression here affects every downstream miniextendr package. The
new test should cover the golden path; manual verification on a non-trivial
fixture is essential before merge.

## PR expectations

- Branch: `fix/issue-249-vendor-crates-io-revendor`
- One PR, don't merge — CR review required
- Update memory on completion if patterns emerge worth remembering
