# Standalone `miniextendr_build()` skips wrapper-gen in tarball mode

**Date:** 2026-05-29
**Status:** Bug CONFIRMED on the pre-#757 code path; #757 closes the
`miniextendr_build()` entry point. One residual template defect found and fixed
here; one deferred follow-up filed.

## What was attempted

Reproduce the maintainer's report: a *standalone* (non-monorepo) package
scaffolded by `minirextendr::create_miniextendr_package()` and then built with
`minirextendr::miniextendr_build()` ends up with no R wrappers, so
`library(pkg)` exposes no functions. Suspected cause: `miniextendr_build()`
flips the build into tarball mode (writes `inst/vendor.tar.xz`), and tarball
mode skips the cdylib wrapper-generation pass.

## Reproduction setup

- Worktree HEAD `c98cc3f9` (includes PR #757).
- Built/installed `minirextendr` from the worktree.
- Scaffolded a standalone package in `/tmp/wraptest/wrapdemo` — **no `.git`
  ancestor**, so it behaves like a real end-user package (auto-vendor fires).
- The scaffold ships two trivial exports out of the box:
  `add(a,b) -> f64` and `hello(name: &str) -> String`.

## Observed: the mechanism is real

`miniextendr_build()` itself never calls `miniextendr_vendor()`. The tarball
latch is set *during* the `devtools::install()` step:

1. `devtools::install()` → pkgbuild runs `bootstrap.R`
   (`Config/build/bootstrap: TRUE`) in a build-staging dir with no `.git`
   ancestor.
2. `bootstrap.R` sees no `inst/vendor.tar.xz` → runs `cargo-revendor` →
   **writes `inst/vendor.tar.xz`** (1.9 MB).
3. `./configure` then detects the tarball and prints
   `install mode = tarball install (offline, vendored)` and writes
   `.cargo/config.toml` in tarball mode.
4. `src/Makevars` (`IS_TARBALL_INSTALL=true`) gates the `$(WRAPPERS_R)` target:
   absent `MINIEXTENDR_FORCE_WRAPPER_GEN`, it takes the "use pre-shipped
   wrappers" branch and skips the cdylib build.

So the maintainer's claim is exactly right: **`miniextendr_build()` does run in
tarball mode**, and tarball mode skips wrapper-gen.

## The bug, demonstrated (pre-#757 behavior)

Reset the scaffold (delete `R/wrapdemo-wrappers.R`, `inst/vendor.tar.xz`,
`vendor/`), then ran autoconf + configure + `devtools::install()` **without**
`MINIEXTENDR_FORCE_WRAPPER_GEN` (i.e. what `miniextendr_build()` did before
#757):

```
configure: install mode         = tarball install (offline, vendored)
tarball install: using pre-shipped R/-wrappers.R
ERROR: tarball is missing pre-generated .../wrapdemo/R/wrapdemo-wrappers.R
make: *** [.../R/wrapdemo-wrappers.R] Error 1
ERROR: compilation failed for package 'wrapdemo'
```

The wrapper file never existed (fresh build), tarball mode refused to generate
it, and the existence guard turned it into a hard build failure. Before that
guard was added it would have installed a wrappers-less package silently
(library() exposes nothing) — same root cause, quieter symptom. Either way:
**bug reproduced**.

## The fix (PR #757) verified working

The installed `minirextendr` (with #757) sets
`MINIEXTENDR_FORCE_WRAPPER_GEN="1"` around the `devtools::install()` call (and
restores it on exit). Running the real `miniextendr_build()`:

```
configure: install mode         = tarball install (offline, vendored)
MINIEXTENDR_FORCE_WRAPPER_GEN set: regenerating wrappers from cdylib (tarball mode override)...
```

Post-build observables:
- `R/wrapdemo-wrappers.R` exists, contains `add <- function(a, b)` and
  `hello <- function(name)`.
- `inst/vendor.tar.xz` present (tarball mode WAS engaged — proves the latch
  fires, and that the FORCE override is what salvages wrapper-gen).
- `NAMESPACE` exports `add`, `hello`.
- After install: `library(wrapdemo); hello("world")` → `"Hello, world!"`;
  `add(2,3)` → `5`; `getNamespaceExports("wrapdemo")` → `hello, add`.
- `MINIEXTENDR_FORCE_WRAPPER_GEN` is `<unset>` after the build (no env leak —
  #757's restore works).

**Conclusion: the `miniextendr_build()` entry point is fixed by #757.**

## Residual gap (fixed in this PR)

The scaffolded standalone `lib.rs` template
(`minirextendr/inst/templates/rpkg/lib.rs`) told users a *different*, broken
workflow:

```
//      Rscript -e 'devtools::document()'  # Compiles Rust + generates R wrappers
//      Rscript -e 'devtools::install()'   # Install the package
//    devtools::document() handles everything in one step ...
```

Empirically false. Reproduced: a clean `devtools::document(".")` on the
standalone package does **not** generate wrappers (it only runs roxygen via
pkgload's `load_all`, never the cdylib pass), and emits:

```
Warning: Objects listed as exports, but not present in namespace:
• add
• hello
Deleting 'add.Rd' and 'hello.Rd'
```

And a bare `devtools::install()` hits the identical tarball-mode skip the
maintainer reported (it doesn't set the FORCE env). So users following the
scaffolded instructions reproduce the bug even though `miniextendr_build()`
itself is fixed.

**Fix:** rewrote the `lib.rs` template comment to point at
`minirextendr::miniextendr_build()` (the one entry point that sets the FORCE
env) and to explicitly warn that bare `devtools::install()` / `R CMD INSTALL .`
/ `devtools::document()` skip wrapper generation in tarball mode. The monorepo
`lib.rs` template needs no change (monorepo builds via `just rcmdinstall` in
source mode — no auto-vendor flip, no skip). The `create.R` "Next steps"
message already (correctly) recommends `miniextendr_build()`.

## Root cause (one line)

Tarball mode (`inst/vendor.tar.xz` latch) skips cdylib wrapper-gen; any build
entry point that doesn't set `MINIEXTENDR_FORCE_WRAPPER_GEN` produces a
wrappers-less install on a fresh standalone package. `miniextendr_build()` was
fixed in #757; the scaffolded `lib.rs` still advertised the unfixed
`devtools::document()`/`install()` path.

## Deferred follow-up

No automated regression test exercises the full standalone build → wrapper-gen
→ `library()` round-trip (it needs cargo + autoconf + a network fetch of
`miniextendr` `main`, minutes per run — unsuitable for the testthat suite).
Filed as a `just`-recipe / nightly-CI candidate rather than a unit test.
