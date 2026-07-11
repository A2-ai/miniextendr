---
name: miniextendr-getting-started
description: Use when a new user asks how to start a miniextendr-backed R package from zero, how to scaffold via minirextendr::use_miniextendr(), what their first Rust function should look like, what the dev loop is, or "can I add Rust to an existing R package". Also use when someone is confused about which package is which (miniextendr vs minirextendr).
---

# Getting Started with miniextendr

miniextendr lets you write R packages with Rust backends. From an empty R
package to a callable Rust function is five steps. This skill walks a developer
who knows R and basic Rust — but has never written R-Rust FFI — through those
steps.

## When to use this skill

- "I'm new to miniextendr, where do I start?"
- "Can I add Rust to an existing R package?"
- "What does the dev loop look like?"
- "Where do I put my Rust code?"
- "How do I scaffold a new package?"
- "What is the difference between miniextendr and minirextendr?"
- "What does `#[miniextendr]` actually do?"
- "How do I call my Rust function from R?"

## Key concepts

### miniextendr vs minirextendr

These are two separate packages with different audiences:

- **miniextendr** (this framework) is the runtime and macro system your Rust
  code links against. It lives in `miniextendr-api/` and `miniextendr-macros/`
  in this repo. End users add it as a Cargo dependency in their package's
  `Cargo.toml`.

- **minirextendr** is a pure-R scaffolding helper. It generates the directory
  structure, `configure.ac`, `Makevars.in`, and boilerplate that wires an R
  package to its Rust backend. End users run
  `minirextendr::use_miniextendr()` once to set up a new package and never
  touch it again during normal development.

In short: minirextendr sets up the scaffolding; miniextendr is the framework
the scaffolded Rust code uses at build and runtime.

### Source mode vs tarball mode

During local development, the package builds in source mode: cargo resolves
miniextendr from git or a local path, and the `configure` script writes
appropriate `[patch]` or `[source]` entries in `.cargo/config.toml`.

When shipping to CRAN, you run `just vendor` to bundle all cargo dependencies
into `rpkg/inst/vendor.tar.xz`. The presence of that tarball is the single
signal that flips `configure` into offline (CRAN-safe) tarball mode.

For a full explanation of the latch and why it works this way, see the
`miniextendr-architecture` skill. For now: if you are just developing locally,
you never need to worry about this unless `minirextendr_doctor()` reports a
stale tarball.

### What `#[miniextendr]` does

Placing `#[miniextendr]` on a `pub fn` does three things automatically:

1. Generates a `extern "C-unwind"` C wrapper that receives SEXPs, converts them
   to Rust types via `TryFromSexp`, calls your function, and converts the
   return value back to an SEXP via `IntoR`.
2. Emits an entry into the `MX_CALL_DEFS` distributed_slice so the C wrapper
   is registered with R at package load time via `R_init_<pkg>`.
3. Emits an entry into the `MX_R_WRAPPERS` distributed_slice so that a
   corresponding `.Call(C_my_fn, ...)` R wrapper is generated into
   `R/miniextendr-wrappers.R` at build time.

You write plain Rust. The macro handles all the FFI plumbing.

## How it works: the five-step walkthrough

### Step 1: Create the package skeleton

If you are starting from scratch, use the scaffolding helper:

```r
install.packages("minirextendr")  # or devtools::install_github("...")
library(minirextendr)
use_miniextendr()
```

Run this from your new package's root directory (which should already have a
`DESCRIPTION` file — create it first with `usethis::create_package("mypkg")`
if needed).

Before running `./configure`, initialize git with `usethis::use_git()` or
`git init`. Without a `.git` ancestor, configure assumes a CRAN-style/offline
build context, auto-vendors, and latches into tarball mode.

`use_miniextendr()` writes the following into your package:

- `src/Makevars.in` — build configuration template. `configure` writes
  `src/rust/.cargo/config.toml` inline per install mode (no `.in` template
  for that file).
- `configure.ac` — the autoconf source for install-mode detection.
- `configure` — the generated configure script (via `autoconf`).
- `src/rust/Cargo.toml` — a Rust crate depending on `miniextendr-api` and
  `miniextendr-macros`.
- `src/rust/src/lib.rs` — a minimal Rust library with `miniextendr_init!`.
- `src/stub.c` — the minimal C file that R's build system requires.

The key entry point is `minirextendr/R/create.R`. If you want to see what the
scaffolded package looks like before running the function, the canonical live
example is `rpkg/` in this repository.

### Step 2: Add a Rust function

Open `src/rust/src/lib.rs` (inside your R package's source tree) and add:

```rust
use miniextendr_api::miniextendr;

/// Add two integers together.
///
/// @param x First integer.
/// @param y Second integer.
/// @return Integer sum.
/// @export
#[miniextendr]
pub fn add_integers(x: i32, y: i32) -> i32 {
    x + y
}
```

A few rules:

- The function must be `pub`.
- The module containing the function must be reachable via `mod` from `lib.rs`.
- Argument and return types must implement `TryFromSexp` and `IntoR`
  respectively. Common types (integers, floats, strings, logical values,
  vectors) all work out of the box.
- The roxygen comment above the function is real: the macros pick it up and
  include it in the generated R wrapper.

For the simplest live example, look at the small helper functions in
`rpkg/src/rust/lib.rs` around the `small_vec_copy` and `large_vec_altrep`
functions.

### Step 3: Build and install

For a standalone package, use the single command `use_miniextendr()` already
told you to run next:

```r
minirextendr::miniextendr_build()
```

This runs `autoconf` + `./configure`, compiles Rust, generates the R
wrappers, installs the package, and regenerates `NAMESPACE`/`man/` via
roxygen2 — and it sets `MINIEXTENDR_FORCE_WRAPPER_GEN` so wrapper generation
happens even if the package has been flipped into offline tarball mode (see
the pitfall below). **Do not** substitute a bare `R CMD INSTALL .` /
`devtools::install()` / `devtools::document()` for this — those skip the
`MINIEXTENDR_FORCE_WRAPPER_GEN` override and can silently install a package
with an empty namespace.

If you are working inside the miniextendr monorepo instead, use the maintainer
recipes:

```bash
just configure          # wraps bash ./configure with workspace settings
just rcmdinstall        # R CMD INSTALL with correct flags
just force-document     # run roxygen2 to regenerate NAMESPACE and man/
```

The manual equivalent (useful for understanding what's happening, or when
`minirextendr` isn't installed) is `bash ./configure && R CMD INSTALL .` —
always `bash ./configure`, not `./configure`: the script uses `#!/bin/sh` as
its shebang, and `AC_CONFIG_COMMANDS` passthrough produces spurious errors
under that shell.

**Pitfall — configure before `git init`**: if you scaffold a package and run
`configure`/`miniextendr_build()` before ever running `git init`, the
"no `.git` ancestor" auto-vendor self-repair fires and silently flips the
package into CRAN-style offline tarball mode (you'll see `configure: install
mode = tarball install (offline, vendored)` with no further explanation).
This is otherwise harmless, but avoid the confusion by running `git init`
right after `use_miniextendr()`, before the first `configure`/build.

### Step 4: Call from R and iterate

After installing, load the package and call your function:

```r
library(mypkg)
add_integers(3L, 4L)
# [1] 7
```

The iteration loop for Rust changes is:

1. Edit `.rs` source files.
2. `minirextendr::miniextendr_build()` (or `just configure && just rcmdinstall
   && just force-document` in the monorepo).
3. **Adding, removing, or renaming a `#[miniextendr]` item** (function, class,
   trait impl — anything that changes the *set* of exported names) hits a
   chicken-and-egg ordering: the package is installed *before* roxygen2 rewrites
   `NAMESPACE` from the freshly generated `R/miniextendr-wrappers.R`, so the
   installed image lags the on-disk `NAMESPACE` by one build and the new export
   isn't callable yet (`could not find function "..."`).
   - `minirextendr::miniextendr_build()` **handles the additive case for you
     in a single call**: it snapshots `NAMESPACE` before and after `document()`
     and, if the export set changed, reinstalls once so the installed image
     matches. No second pass needed.
   - The monorepo raw recipes do *not* self-heal — after `just rcmdinstall &&
     just force-document` you must run `just rcmdinstall` once more so the new
     export becomes runtime-callable.
   - **Removing or renaming an export fails harder**
     (A2-ai/miniextendr#1288): the first install validates the stale
     `NAMESPACE` — now a *superset* of the regenerated wrappers — and aborts
     with `undefined exports: <old_name>` *before* `document()` can reconcile
     it. (The additive case self-heals because a subset `NAMESPACE` still
     installs; closed #860 covered only that side.) The aborting install has
     already regenerated `R/<pkg>-wrappers.R`, so recovery is: run
     `devtools::document()` / `roxygen2::roxygenize()` directly — it loads via
     `pkgload::load_all()`, which only *warns* on the mismatch — then install
     again. Alternatively, reset `NAMESPACE` to the roxygen header plus the
     `useDynLib` line before the first install.
   Pure signature/body edits to *existing* exports need neither — only changes
   to the set of exported names do.
4. Restart R (or reload the package) and test.

Generated files (`R/miniextendr-wrappers.R`, `NAMESPACE`, `man/*.Rd`) must be
committed in the same PR as the Rust changes that produced them. The pre-commit
hook in this repo enforces this for wrappers.R.

### Step 5: Build a tarball for CRAN

```bash
just vendor             # bundles all cargo dependencies into inst/vendor.tar.xz
R CMD build .           # produces the tarball
R CMD check mypkg_*.tar.gz --as-cran
```

`just vendor` is the maintainer-only step. It calls `cargo-revendor` to produce
a `vendor/` tree, compresses it, and places it at `inst/vendor.tar.xz`. Once
that file is present, `configure` automatically switches to offline tarball mode
for any subsequent `R CMD INSTALL` or check.

The tarball recipes in the justfile trap-clean `inst/vendor.tar.xz` on exit so
it does not linger after a build. If it does linger (for example, after a
failed build), run `just clean-vendor-leak` to remove it.

For a detailed account of the CRAN compatibility requirements, see
`docs/CRAN_COMPATIBILITY.md`.

## Decision trees

### I want to create a new package vs add Rust to an existing one

Starting from scratch:

1. `usethis::create_package("mypkg")` (creates the R package skeleton).
2. `minirextendr::use_miniextendr()` from the package root (adds Rust
   scaffolding).
3. Write Rust functions with `#[miniextendr]`.
4. `bash ./configure && R CMD INSTALL .`.

Adding Rust to an existing R package:

1. From the package root, run `minirextendr::use_miniextendr()`. It detects an
   existing `DESCRIPTION` and adds scaffolding without overwriting your existing
   R code.
2. The scaffolded `Makevars.in` and `configure.ac` coexist with any existing
   `src/` files.
3. Same build steps as above.

### I am shipping to CRAN vs internal use only

For internal use:

- No vendoring needed. Source mode (no `inst/vendor.tar.xz`) is the default.
- Users install via `remotes::install_github()` or similar. Cargo fetches
  dependencies from crates.io or git at install time.

For CRAN submission:

- Run `just vendor` before `R CMD build`. This bundles all dependencies.
- Check the built tarball: `R CMD check mypkg_*.tar.gz --as-cran`. Always check
  the tarball, not the source directory — the source directory skips
  `Authors@R` to `Author/Maintainer` conversion and misses real CRAN failures.
- See `docs/CRAN_COMPATIBILITY.md` for the full set of requirements.

## Key files

- `minirextendr/R/create.R` — `use_miniextendr()` and
  `create_miniextendr_package()` entry points.
- `rpkg/src/rust/lib.rs` — the canonical live example of a miniextendr Rust
  crate. The simplest exported functions near the `small_vec_copy` region show
  the minimal `#[miniextendr] pub fn` pattern.
- `rpkg/configure.ac` — the per-install entrypoint that detects source vs
  tarball mode and writes the cargo config.
- `rpkg/src/Makevars.in` — the Makevars template that drives the cdylib to
  staticlib double-link build.
- `docs/CRAN_COMPATIBILITY.md` — vendoring requirements, offline build
  verification, and CRAN submission checklist.

## Common pitfalls

- **`bash ./configure`, not `./configure`**: the script uses `#!/bin/sh`,
  and `AC_CONFIG_COMMANDS` passthrough produces spurious errors under that
  shell. Always use `bash ./configure` or run the `just configure` recipe.

- **`just` is maintainer-only**: the `just` recipes are for working inside this
  repository. End-user packages must build via `configure.ac`, `tools/*.R`, and
  standard R mechanisms (`R CMD INSTALL`, `devtools::install()`). Never require
  `just` in scaffolded package instructions.

- **Generated wrappers must be committed**: `R/miniextendr-wrappers.R` is
  generated during `R CMD INSTALL`. If you add or remove `#[miniextendr]`
  functions, regenerate and commit this file in the same PR. The pre-commit
  hook in this repo blocks commits where `*-wrappers.R` is staged without a
  matching updated `NAMESPACE`.

- **Install-mode latch leak**: if `inst/vendor.tar.xz` is present during local
  development (left over from a previous `R CMD build` that did not clean up),
  configure writes vendored mode and your edits to miniextendr workspace crates
  have no effect. Run `just clean-vendor-leak` or delete the file manually. Use
  `minirextendr_doctor()` to detect the condition automatically.

- **Modules must be reachable from `lib.rs`**: if you add a new `.rs` file with
  `#[miniextendr]` functions, you must add `mod my_module;` to `lib.rs`.
  Functions in unreachable modules are silently excluded from registration.

- **Tests stay green under a stale superset `NAMESPACE`**: after removing or
  renaming an export, `pkgload::load_all()` — used by `devtools::document()`
  and `testthat::test_local()` — only *warns* ("Objects listed as exports, but
  not present in namespace"), while `R CMD INSTALL` / `library()` hard-fail on
  the same mismatch. So the test suite can pass while `R CMD check` and fresh
  installs break. If you removed or renamed a `#[miniextendr]` export, confirm
  `NAMESPACE` was regenerated (see the dev-loop step above and
  A2-ai/miniextendr#1288).

- **`configure.ac` must not be called via `minirextendr::*`**: configure-time
  helpers belong in `tools/*.R` and are invoked via `Rscript tools/foo.R`. Do
  not call minirextendr functions from configure — configure runs in a minimal
  environment without the R library available.

## Related skills

- `miniextendr-architecture` — how the cdylib to staticlib double-link works,
  the distributed_slice registration system, and the install-mode latch in
  depth.
- `miniextendr-build` — configure.ac, Makevars.in, vendor pipeline, and the
  justfile recipes.
- `miniextendr-scaffolding` — minirextendr templates, template sync workflow,
  `minirextendr_doctor()`, and upgrade mechanics.
- `miniextendr-conversions` — `TryFromSexp`, `IntoR`, `Coerce`, NA handling,
  and the full type conversion matrix.
- `miniextendr-macros` — deep dive into what `#[miniextendr]` generates and
  how to use advanced attributes.
