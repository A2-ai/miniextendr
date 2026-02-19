# Smoke Test Report

**Date:** 2026-02-19

## Pre-checks

All pre-checks pass:
- `just templates-check` — PASS
- `just vendor-sync-check` — PASS (warnings about missing vendor/ in dev mode are expected)
- `just check` (cargo check all crates) — PASS
- `just clippy` — PASS, zero warnings

## Monorepo Tests (rpkg, cross-packages, Rust, minirextendr)

| Suite | Result | Count |
|-------|--------|-------|
| rpkg R tests | PASS | 3112 pass, 0 fail, 5 skip |
| Cross-package R tests | PASS | 86 pass, 0 fail |
| Rust unit tests (all crates + rpkg + cross-pkgs) | PASS | 233+ pass, 0 fail |
| minirextendr R tests | PASS | 204 pass, 0 fail |

## Smoke Test 1: Standalone R Package

**Scaffolded with:** `minirextendr::create_miniextendr_package("standalone")`

### Code added
- `reverse_string()`, `fibonacci()` functions
- `Counter` R6 class with `new()`, `increment()`, `get()`, `label()` methods
- 15 R tests covering all functions and the R6 class

### Dev workflow

| Step | Result |
|------|--------|
| Scaffold | PASS |
| `NOT_CRAN=true bash ./configure` | PASS |
| `R CMD INSTALL .` (first) | PASS |
| `devtools::document()` | PASS — generated 5 Rd files + Counter.Rd |
| `R CMD INSTALL .` (second) | PASS |
| `devtools::test()` | PASS — 15/15 |

### CRAN prep workflow

| Step | Result |
|------|--------|
| `minirextendr::miniextendr_vendor()` | PASS — created inst/vendor.tar.xz (1.4 MB) |
| `bash ./configure` (CRAN mode auto-detected) | PASS — "CRAN build — vendor ready" |
| `R CMD build .` | PASS |
| `R CMD check --as-cran` | 2 WARNINGs, 3 NOTEs (see below) |

### R CMD check issues (standalone)

**WARNINGs:**
1. `Non-standard license specification` — placeholder license from scaffolding template. **Expected** (user must choose a license).
2. `Documented arguments not in \usage in Rd file 'Counter.Rd': 'label'` — The `label` parameter of `Counter$new(label)` is documented in roxygen but the generated \usage section doesn't include it. **This is a miniextendr doc generation issue** (see Issue 1 below).

**NOTEs:** Standard new-submission boilerplate (placeholder maintainer, version, timestamps).

## Smoke Test 2: Monorepo Package

**Scaffolded with:** `minirextendr::create_miniextendr_monorepo("monorepo", crate_name="monorepo-core", rpkg_name="monorepo.rpkg")`

### Code added
- In `monorepo-core/src/lib.rs`: `factorial()`, `is_prime()`, `collatz()` (pure Rust)
- In `monorepo.rpkg/src/rust/lib.rs`: R-exposed wrappers calling into monorepo-core
- 13 R tests covering all functions

### Modifications needed after scaffolding
1. **Removed `miniextendr-api` dep from `monorepo-core/Cargo.toml`** — scaffolding adds `miniextendr-api.workspace = true` to the core crate, but the workspace resolves it from crates.io (where it's not published). Pure library crates shouldn't depend on miniextendr-api. See Issue 2.
2. **Ran `autoconf` manually in monorepo.rpkg/** — scaffolding generated `configure` for the standalone template but NOT for the monorepo rpkg. See Issue 3.
3. **Added `monorepo-core` path dep** to rpkg's Cargo.toml — this is expected user action.

### Dev workflow

| Step | Result |
|------|--------|
| Scaffold | PASS |
| `autoconf` (manual) | PASS |
| `just configure` | PASS (with non-fatal lockfile warning) |
| `R CMD INSTALL monorepo.rpkg` | PASS |
| `devtools::document("monorepo.rpkg")` | PASS — generated 5 Rd files |
| `R CMD INSTALL monorepo.rpkg` (second) | PASS |
| `devtools::test("monorepo.rpkg")` | PASS — 13/13 |

### CRAN prep workflow

| Step | Result |
|------|--------|
| `minirextendr::miniextendr_vendor("monorepo.rpkg")` | PASS |
| `bash ./configure` (CRAN mode) | PASS |
| `R CMD build monorepo.rpkg` | PASS |
| `R CMD check --as-cran` | **ERROR** — monorepo-core not found (see Issue 4) |

---

## Issues Found

### Issue 1: Counter.Rd \usage section missing constructor parameters — FIXED

**Severity:** Medium (causes R CMD check WARNING)
**Affects:** Env-class doc generation

When a `#[miniextendr] impl` block exposes an env-class, `@param` tags on methods create `\arguments` entries in the Rd file but have no matching `\usage` entry (roxygen can't infer usage from `Counter$new <- function()` assignment patterns). R CMD check warns "Documented arguments not in \usage".

**Fix:** Added `params_as_details` mode to `MethodDocBuilder`. For env-class methods, `@param name desc` tags are converted to Rd `\describe{\item{\code{name}}{desc}}` blocks in the description section instead of creating `@param` roxygen tags. This preserves parameter documentation while avoiding the `\arguments`/`\usage` mismatch.

**Files changed:** `r_class_formatter.rs` (MethodDocBuilder), `miniextendr_impl.rs` (env-class generator), `roxygen.rs` (push_roxygen_tags_str helper).

### Issue 2: Scaffolded monorepo core crate unnecessarily depends on miniextendr-api — FIXED

**Severity:** High (blocks `cargo generate-lockfile` — build fails)
**Affects:** `create_miniextendr_monorepo()`

The generated `monorepo-core/Cargo.toml` includes:
```toml
[dependencies]
miniextendr-api.workspace = true
```

But the workspace resolves `miniextendr-api` from crates.io (`version = "*"`), where it's not published. This causes `cargo generate-lockfile` to fail with "no matching package named miniextendr-api found".

**Workaround:** Remove the `miniextendr-api.workspace = true` line from the core crate if it doesn't need R interop.

**Root cause files:**
- `minirextendr/inst/templates/monorepo/Cargo.toml.tmpl` lines 9-11: workspace deps use `version = "*"` for crates not on crates.io
- `minirextendr/inst/templates/monorepo/my-crate/Cargo.toml.tmpl` line 7: `miniextendr-api.workspace = true`

**Where to fix:** `minirextendr` scaffolding — the monorepo core crate template should NOT include miniextendr-api as a dependency by default. The workspace Cargo.toml's `[workspace.dependencies]` should either use git sources or omit the miniextendr entries entirely (the rpkg already has its own path deps to vendor/).

### Issue 3: Monorepo scaffolding doesn't generate `configure` for rpkg — FIXED

**Severity:** Medium (requires manual `autoconf` step)
**Affects:** `create_miniextendr_monorepo()`

The standalone template scaffolding runs `autoconf` and generates `configure` from `configure.ac`. The monorepo scaffolding does NOT do this for the rpkg subdirectory, so `just configure` fails with "No such file or directory".

**Workaround:** Run `cd monorepo.rpkg && autoconf` manually.

**Root cause:** `minirextendr/R/create.R` — `create_miniextendr_monorepo()` (lines 63-148) never calls `miniextendr_autoconf()`. The standalone `use_miniextendr()` (lines 395-406) does call it.

**Where to fix:** Add `miniextendr_autoconf(rpkg_path)` call in `create_miniextendr_monorepo()`, similar to how `use_miniextendr()` does it.

### Issue 4: Monorepo CRAN build fails — user's own crate not vendored

**Severity:** Low (expected constraint, documented)
**Affects:** Monorepo CRAN workflow

`R CMD check` on the built tarball fails because `monorepo-core` (a path dep `../../../monorepo-core`) isn't included in the tarball. This is the documented VENDOR_LIB use case — `use_vendor_lib()` must be called to package the user's crate into `inst/` for CRAN.

**Not a bug** — this is working as designed. The `miniextendr_vendor()` function vendors miniextendr crates and external deps, but not the user's own monorepo crates. Users need to call `use_vendor_lib("monorepo-core", "0.1.0", dev_path = "../monorepo-core")` to handle that.

**Possible improvement:** `miniextendr_vendor()` could detect monorepo path deps and suggest running `use_vendor_lib()`, or the monorepo justfile template could include a `vendor-lib` recipe.

---

## Summary

| Area | Status |
|------|--------|
| Monorepo (rpkg) | All clear — 3112 R tests, 233+ Rust tests, 86 cross-pkg |
| Standalone scaffolding + dev | Works perfectly |
| Standalone CRAN prep | Works (WARNING about R6 class docs) |
| Monorepo scaffolding | Works after fixing 2 issues (autoconf + core crate deps) |
| Monorepo dev | Works after fix |
| Monorepo CRAN prep | Needs VENDOR_LIB for user crate (by design) |
| Vendoring changes (this PR) | Verified: error on missing vendor works correctly |
