# miniextendr_build(): document() after install corrupts under R 4.6 libdeflate — root cause was devtools reload

**Date**: 2026-06-11
**Context**: adding the #898 single-pass-export e2e test, which drives the real
`miniextendr_build()` against a temp-libpath monorepo scaffold.

## What was attempted

A compile-gated test: scaffold → add `#[miniextendr] pub fn mx_new_fn` → one
`miniextendr_build()` → assert `getNamespaceExports()`.

## What went wrong

The build aborted inside `devtools::document()`:

```
internal error 1 in R_decompress1 with libdeflate
Error in env_get_list(...): lazy-load database '.../library/<pkg>/R/<pkg>.rdb' is corrupt
```

Backtrace: `document()` → `roxygen2::roxygenise` → `pkgload::load_all` →
`pkgload:::unregister_namespace` → reading the installed namespace's `.rdb`.
The pre-existing standalone e2e (`test-templates.R:565`, mxroundtrip) failed
identically on clean `main` — pre-existing, deterministic on this machine
(R 4.6.0, libdeflate-compressed lazy-load DBs), not introduced by the new test.

## Root cause

`install_pkg()` called `devtools::install()` with the default `reload = TRUE`,
which re-registers the just-installed package's namespace in the *building*
session. The subsequent `document()` runs `pkgload::load_all()`, whose
`unregister()` step forces that namespace's active bindings — reading the
freshly-written `.rdb`. On R ≥ 4.6 that read intermittently dies with the
libdeflate decompress error. The reload is pointless in a build workflow:
`document()` works from source, and nothing in `miniextendr_build()` needs the
package attached.

A minimal R-only package does **not** reproduce it — it needs the compiled
miniextendr wrapper shape (larger `.rdb`, native-symbol bindings), so the only
real repro is the e2e harness itself.

## Fix

Pass `reload = FALSE` in all three `devtools::install()` call sites in
`minirextendr/R/workflow.R` (`install_pkg()` and both `bootstrap_fresh_wrappers()`
installs). With the fix the #898 e2e passes; before it, it failed with the
corrupt-rdb error on every run.

This likely also removes the `.rdb`-corrupt failure mode of the known-flaky
standalone e2e (#1000) — its remaining failure modes observed this session were
environmental (disk exhaustion, offline cargo resolution), not the rdb error.
