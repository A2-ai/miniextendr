# minirextendr/

Pure R scaffolding helper — generates new R packages with a `miniextendr` Rust backend. End users install this from CRAN/GitHub and run `minirextendr::use_miniextendr_package()`. See root `CLAUDE.md` for shared rules.

## Loaded name
Package loads as `library(minirextendr)`. The exemplar consumer (`rpkg/`) is its own package (`library(miniextendr)`).

## Dev loop
```bash
just minirextendr-install
just minirextendr-test
just minirextendr-check
```

## Templates pipeline
`inst/templates/` is **derived from `rpkg/`** (master source). Workflow:
1. Edit `rpkg/` first.
2. Port changes into `inst/templates/`.
3. `just templates-approve` locks the delta into `patches/templates.patch`.
4. `just templates-check` verifies no unexpected drift in CI.

Templates may have extra standalone-project logic (e.g., checking for miniextendr-api before applying path overrides, running `cargo vendor` for transitive deps). That delta is what `patches/templates.patch` records.

## Key R modules
- `R/vendor.R` — `vendor_crates_io()`: the `cargo revendor` call behind `miniextendr_vendor()`. It trims with `--strip-all` (TOML sections, dev-dependencies, dangling `[features]`, and every `tests/` / `benches/` / `examples/` directory that the crate's source doesn't `include_str!()` or `#[path]`). A caller's `revendor_args` are appended, and any `--strip-*` flag among them replaces the default.
- `R/use_*.R` — scaffolding (`use_miniextendr_package`, `use_release_workflow`, `use_template`).
- `R/upgrade.R` — `upgrade_miniextendr_package()`. It fingerprints every file in `upgrade_owned_files()` before the first write and after the last, and its closing summary (and invisible return value) lists the `changed` / `added` / `removed` ones (#1713). A new file the upgrade writes must be added to that list, or the summary misses it (`test-upgrade-fidelity.R` checks that every created file is reported). `R/upgrade-substitutions.R` holds its pre-write check (#1733): with `configure_ac = FALSE`, the kept `configure.ac` must substitute every `@VAR@` of the template `Makevars.in` / `win.def.in` (autoconf's `ac_subst_vars`, static scan without autoconf), or the upgrade aborts before writing anything.
- `R/doctor.R` — `miniextendr_doctor()`: detects stale `inst/vendor.tar.xz` leak + missing `.cargo/config.toml` + git-tracked generated files (`git rm --cached` advice, #1250) + a git-tracked `vendor.tar.xz` (same advice) + an S7 package whose `.onLoad()` lacks `S7::methods_register()` (`use_s7()` writes one to `R/zzz.R`); `webr = TRUE` adds the webR import lint.
- `R/webr-lint.R` — `miniextendr_webr_import_lint()`: flags namespace-level imports of compiled packages (static probe + curated fallback, #925).
- `R/check_static.R` — `miniextendr_check_static()` (revamp shipped in PR #296).

## Gotchas
- `usethis::write_over()` skips silently in non-interactive mode. `use_template()` deletes the target first so `upgrade_miniextendr_package()` actually overwrites.
- Cargo directory source can't find manually-extracted crates from `.crate` files — use `[patch.crates-io]` + path deps for workspace crates.
- Regression tests in `tests/testthat/` grep function source for literal strings (`deparse(body)` style). Don't inline a helper just to satisfy them — fix the test or accept the indirection.
- Every function in `R/` has an Rd page, internal helpers included: they get `@keywords internal` (never `@noRd`) with a title, a `@param` per formal and `@return`. Closely related helpers share a page via `@rdname` (e.g. `webr_lint_internals`, `prefreeze_manifest`). `tests/testthat/test-rd-coverage.R` fails on a function with no `\alias{}` in `man/`, or a page with an undocumented argument (#1727); R CMD check alone skips arguments on internal pages unless `_R_CHECK_RD_INTERNAL_TOO_=true`.
- `configure` and `bootstrap.R` never vendor. `bootstrap.R` only stages path dependencies outside the package; `inst/vendor.tar.xz` comes from `miniextendr_vendor()`, which `miniextendr_build_tarball()` / `miniextendr_check()` run and clean up.

## End-user contract
**`just` is maintainer-only.** Scaffolded packages must build via `configure.ac` / `tools/*.R` / standard R mechanisms. If a template requires `just`, fix the template.
