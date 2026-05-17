+++
title = "Static R CMD check for miniextendr (no Rust compile, no cargo network)"
description = "Design rationale for miniextendr_check_static(): --install=fake approach, naming, tests, and docs."
+++

# Static R CMD check for minirextendr (no Rust compile, no cargo network)

## Motivation

`minirextendr::miniextendr_check()` (workflow.R) wraps
`rcmdcheck::rcmdcheck()` and produces a full R CMD check. The flow is
`R CMD build` -> `R CMD INSTALL` -> `R CMD check`, and the install step
runs `./configure`, which runs `cargo` (directly or via
`cargo revendor`). In offline / sandboxed / no-network environments
(CRAN's --no-internet incoming checks, devcluster, Claude Code
sandbox), the cargo step fails with "failed to download ..." and the
rest of the check never runs.

We want a companion function that runs every R-level check *without*
triggering cargo at all, so the package's DESCRIPTION / NAMESPACE /
R code / Rd docs / subdirs can still be validated in those
environments.

## Approach -- `R CMD check --install=fake`

`R CMD check --install=fake` is exactly the right primitive:

- **`R CMD INSTALL --fake`** (invoked by check) sets `use_configure <- FALSE`
  (r-svn `tools/R/install.R:2424`). `./configure` is **not run** => no
  cargo invocation, no `.so` build, no network.
- `R CMD check` automatically sets
  `do_examples <- do_tests <- do_vignettes <- do_build_vignettes <- 0`
  when `install == "fake"` (r-svn `tools/R/check.R:7373`). Anything that
  requires loading the installed package is skipped.
- Every *static* phase still runs: DESCRIPTION, NAMESPACE, R code
  syntax, packages-used analysis, S3 method registration, code-usage
  (with `codetools`), Rd parsing / cross-refs / metadata, sub-dirs,
  top-level files, ASCII checks, dataset metadata checks.

`R CMD build` is also safe: it normalises line endings in `configure`
(`build.R:683`) but never executes it.

## Deliverable

New exported function `miniextendr_check_static()` in
`minirextendr/R/check.R`. Wraps `rcmdcheck::rcmdcheck()` with
`--install=fake`; returns the same result object as `miniextendr_check()`.

```r
miniextendr_check_static <- function(path = ".",
                                     args = c("--install=fake", "--no-manual"),
                                     build_args = c("--no-build-vignettes"),
                                     error_on = "warning",
                                     roxygenize = TRUE,
                                     ...) {
  with_project(path)
  if (!requireNamespace("rcmdcheck", quietly = TRUE)) {
    cli::cli_abort(c(
      "rcmdcheck is required for miniextendr_check_static()",
      "i" = 'Install it with: install.packages("rcmdcheck")'
    ))
  }
  # Guarantee --install=fake is in args (and warn if the user overrode it).
  if (!any(grepl("^--install=", args))) {
    args <- c(args, "--install=fake")
  } else if (!any(grepl("^--install=fake$", args))) {
    cli::cli_warn(c(
      "miniextendr_check_static() requires --install=fake to skip Rust compile",
      "i" = "Override detected in {.arg args}; check may trigger cargo"
    ))
  }

  cli::cli_h1("miniextendr static check (no Rust compile)")

  if (isTRUE(roxygenize) && requireNamespace("roxygen2", quietly = TRUE)) {
    roxygen2::roxygenize(usethis::proj_get())
  }

  cli::cli_alert_info("Skipped phases: install (configure/cargo), examples, tests, vignettes")
  cli::cli_alert_info("Running phases: DESCRIPTION, NAMESPACE, R code, Rd, subdirs")

  invisible(rcmdcheck::rcmdcheck(
    usethis::proj_get(),
    args = args,
    build_args = build_args,
    error_on = error_on,
    ...
  ))
}
```

## Naming

- **`miniextendr_check_static`** -- scope-describing (static checks only). Preferred.
- Rejected: `miniextendr_check_no_rust` (misleading -- it *does* still validate Rust-related scaffolding like Makevars presence, it just doesn't compile), `miniextendr_check_lite` (too vague), `miniextendr_check_offline` (describes *when* to use, not what it does).

## Tests (`minirextendr/tests/testthat/test-check.R`)

Three test cases, all guarded with `skip_on_cran()` + `skip_if_not_installed("rcmdcheck")`:

1. **Happy path on minimal R pkg** (no Rust scaffolding): scaffold a minimal
   R package in a tempdir, call `miniextendr_check_static(..., roxygenize = FALSE)`,
   assert `$errors` is empty, assert `src/rust/target/` is NOT created.

2. **Broken DESCRIPTION surfaces**: drop the `Version:` field, assert either an
   error throw or `result$errors`/`result$warnings` non-empty.

3. **Real miniextendr-shaped fixture**: a manually-created fixture under
   `_fixtures/static-check-rustpkg/` with `src/rust/Cargo.toml` and a stub
   `configure` script. Asserts: no errors, `src/rust/target/` absent,
   `inst/vendor.tar.xz` absent.

## Documentation

- Roxygen block on the new function with explicit "Catches:" and "Does not catch:" bullets.
- `@note` pointer to `miniextendr_check()` for full validation.
- `@seealso [miniextendr_check()]` cross-reference (and reverse seealso in workflow.R).
- One-line mention in `minirextendr/README.md` under the Diagnostics section.

## Files touched

| File | Why |
|---|---|
| `minirextendr/R/check.R` (new) | The new function and its roxygen |
| `minirextendr/NAMESPACE` | roxygen regenerate |
| `minirextendr/man/miniextendr_check_static.Rd` (new) | roxygen regenerate |
| `minirextendr/man/miniextendr_check.Rd` | `@seealso` regen |
| `minirextendr/tests/testthat/test-check.R` (new) | tests |
| `minirextendr/tests/testthat/_fixtures/static-check-rustpkg/` (new) | fixture for real-pkg test |
| `minirextendr/R/workflow.R` | small `@seealso` addition |
| `minirextendr/README.md` | one-line mention |
| `plans/check-static-no-compile.md` | this file |

No DESCRIPTION changes: `rcmdcheck` is already in Suggests.

## References

- `background/r-svn/src/library/tools/R/install.R:2424` -- `use_configure <- FALSE` under fake install
- `background/r-svn/src/library/tools/R/check.R:7371-7376` -- do_examples/tests/vignettes zeroed under fake install
- `background/r-svn/src/library/tools/R/check.R:5929` -- `--fake` passed to INSTALL_opts
- `background/r-svn/src/library/tools/R/build.R:683` -- build normalises but never runs `configure`

---

## Revamp 2026-05-04

This section records decisions confirmed during the revamp of PR #296
(branch: `feat/minirextendr-check-static-revamp`, based on
`origin/main` @ `b41c094f`).

The six open questions from `plans/revamp-minirextendr-check-static.md`
are resolved as follows:

**Q1 -- function or flag?**
Confirmed: separate function `miniextendr_check_static`. The static check
carries a different *guarantee* (no Rust compile, phases permanently
skipped) that is cleaner to express as a dedicated API than as a flag.

**Q2 -- scope**
Documented via explicit "Catches:" and "Does not catch:" bulleted lists
in the roxygen `@description`. Users see this in `?miniextendr_check_static`.

**Q3 -- always available, no gating**
The function is always exported. An `@note` in the roxygen points users
at `miniextendr_check()` for full CRAN-submission validation.

**Q4 -- require rcmdcheck, no shell-out fallback**
Consistent with `miniextendr_check()`. `rcmdcheck` is already in Suggests.

**Q5 -- real-miniextendr-pkg test**
Added as test 3. Uses a frozen fixture under
`tests/testthat/_fixtures/static-check-rustpkg/` rather than
`create_miniextendr_package()` because the scaffolding helper requires
autoconf + cargo + network (which defeats the purpose of a static-check test).
The fixture has `src/rust/Cargo.toml` and a stub `configure` script.

**Q6 -- roxygenize opt-in**
Added `roxygenize = TRUE` parameter. When TRUE and `roxygen2` is available,
calls `roxygen2::roxygenize()` before `rcmdcheck`. Silent skip when roxygen2
is absent. Tests pass `roxygenize = FALSE` to stay self-contained.

### Use-case framing (CGMossa's comment)

The primary use-case is: *R CMD check that doesn't complain about Rust-powered
R packages that have not yet vendored their dependencies* -- fast iteration /
pre-vendor sanity / sandbox environments. The function name
(`_static` suffix) and its `@description` lead with this framing.
