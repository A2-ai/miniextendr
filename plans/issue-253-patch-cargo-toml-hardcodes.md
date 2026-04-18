# Plan: `patch_cargo_toml` reads workspace values at runtime OR is deleted (#253)

## Problem

`minirextendr/R/vendor.R:225–255` (`patch_cargo_toml`) hardcodes
`edition = "2024"`, `version = "0.1.0"`, `license = "MIT"`. These values
silently diverge from the actual workspace `Cargo.toml` after any version
bump / edition upgrade, producing vendored copies that cargo then rejects at
resolve time.

## Decision tree

**Step 1: check whether `patch_cargo_toml` still has callers after #249 lands.**

After #249 (`vendor_crates_io` → `cargo revendor`) completes, cargo-revendor's
`cargo package` path emits a resolved Cargo.toml with workspace inheritance
already expanded and the correct version/edition. The custom R-side
`patch_cargo_toml` may become dead code.

Grep:
```bash
rg -n 'patch_cargo_toml' minirextendr/
```

**If zero callers remain: delete the function (approach B).**
**If callers remain: fix it to read workspace values at runtime (approach A).**

## Approach A — read workspace values at runtime

If `patch_cargo_toml` is still called:

1. Before any patch call, read the workspace `Cargo.toml` (e.g. via
   `jsonlite::fromJSON(system2("cargo", c("metadata", "--format-version=1"),
   stdout = TRUE))$workspace_root` → parse its Cargo.toml).
2. Extract `package.version`, `package.edition`, `package.license` (plus
   `package.rust-version` if referenced).
3. Pass these as parameters to `patch_cargo_toml`; use them in place of
   hardcoded values.
4. If any field is missing (package is `publish = false`), fall back to the
   current hardcoded defaults but emit a `warning()` calling out the
   assumption.

Choose the simplest toml parser available — if `blogdown` / `rstan` / another
R package with TOML support is already a dep, use it. Otherwise use a tiny
regex-based extractor for just `version = "..."`, `edition = "..."`,
`license = "..."` under `[package]`. Keep it under 30 lines.

## Approach B — delete the function

If `patch_cargo_toml` has no remaining callers after #249:

1. Delete the function body in vendor.R.
2. Delete any internal helpers that only `patch_cargo_toml` used.
3. Update the roxygen `@section` anywhere that referenced the function.
4. `just devtools-document` to regenerate NAMESPACE.

## Files to change

- `minirextendr/R/vendor.R` — either rewrite function body (A) or delete (B).
- `minirextendr/NAMESPACE` — regenerate if function deleted.
- `minirextendr/tests/testthat/test-vendor.R` — add regression: bump fixture
  workspace version to `0.9.99`, call `vendor_miniextendr()`, assert the
  vendored workspace crate's Cargo.toml has `version = "0.9.99"`.

## Verification

```bash
just minirextendr-test
rg -n 'patch_cargo_toml' minirextendr/  # zero hits if B; few hits if A
# manual: in a sandbox, bump version to 0.99.0 in workspace, run
# miniextendr_vendor, inspect extracted vendor/<crate>/Cargo.toml
```

## Coordination with #249

This issue depends on #249 landing first. If #249 is still in review when
an agent picks up #253, the agent should:
- Check out #249's branch locally to inspect the post-fix state
- Base the #253 branch on #249's branch (not on main)
- Note the dependency in the PR description

## Out of scope

- Broader vendor.R refactor
- Changes to the workspace Cargo.toml that would obviate this function
  (e.g. adding a `publish = true` flag)

## Risk

Low for B (dead-code removal, mechanical). Medium for A (parses TOML in R
without a well-tested TOML lib — need to be careful about quoting,
multi-line strings, `[package]` section scope).

## PR expectations

- Branch: `fix/issue-253-patch-cargo-toml`
- No merge — CR review
- Note the #249 dependency clearly in the PR body
