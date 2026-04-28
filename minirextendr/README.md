# minirextendr

minirextendr is an R helper package for scaffolding and maintaining R packages
that use miniextendr for Rust <-> R interop. It provides templates, autoconf /
configure wiring, vendoring helpers, and cargo command wrappers tailored to R
package workflows.

## Installation

From GitHub:

```r
remotes::install_github("A2-ai/miniextendr", subdir = "minirextendr")
```

From a local checkout:

```r
devtools::install("minirextendr")
```

## Requirements

- Rust toolchain (>= 1.85) on `PATH`
- `autoconf`
- A working R build toolchain

## Quick start

Create a new standalone miniextendr package:

```r
library(minirextendr)
create_miniextendr_package("path/to/pkg")
```

Create a monorepo with a Rust workspace and embedded R package:

```r
library(minirextendr)
create_miniextendr_monorepo("path/to/project")
```

Or add miniextendr scaffolding to an existing package/project:

```r
library(minirextendr)
use_miniextendr()
```

## Workflow helpers

Generate build files and run the standard package workflow:

```r
miniextendr_autoconf()
miniextendr_configure()
miniextendr_build()
```

`miniextendr_build()` runs the normal pipeline:

1. `autoconf`
2. `./configure`
3. package install (`devtools::install()` when available)
4. roxygen regeneration (`devtools::document()`)

The generated wrapper file is written during install/build and then committed as
part of the R package sources.

## Templates

See `inst/templates/README.md` for the standalone-package and monorepo layouts
used by the scaffolder.

The templates are derived from this repo's `rpkg/` example package and checked
with `just templates-check` / `just templates-approve` at the repo root.

Both standalone and monorepo templates use git dependencies for miniextendr
crates. For monorepo development, where you want to use local miniextendr
crates instead of git sources, add a `[patch."https://..."]` section to your
`Cargo.toml`:

```toml
[patch."https://github.com/A2-ai/miniextendr"]
miniextendr-api = { path = "../path/to/miniextendr-api" }
miniextendr-macros = { path = "../path/to/miniextendr-macros" }
miniextendr-lint = { path = "../path/to/miniextendr-lint" }
```

## Diagnostics

```r
miniextendr_doctor()       # Comprehensive project health check
miniextendr_check()        # Full R CMD check workflow
```

## Vendoring

Day-to-day development does **not** need vendoring — `R CMD INSTALL .`
resolves the miniextendr git URL and crates.io deps on first build,
then reuses the cargo registry cache.

For a CRAN submission, run `miniextendr_vendor()` once before
`R CMD build`. It bundles every transitive dep into
`inst/vendor.tar.xz`:

```r
miniextendr_vendor()
```

The presence of `inst/vendor.tar.xz` is the single signal that flips
`./configure` into offline tarball mode. After producing the release
tarball, delete `inst/vendor.tar.xz` to return to source-mode dev —
otherwise subsequent installs stay locked to the vendored snapshot.

## Cargo helpers

These wrappers automatically use `src/rust/Cargo.toml` in the current project.

```r
cargo_add("serde", features = "derive")
cargo_rm("serde")
cargo_update()

cargo_init()
cargo_build()
cargo_check()
cargo_test()
cargo_clippy()
cargo_fmt()
cargo_doc(no_deps = TRUE)

cargo_search("json")
cargo_deps(depth = 2)
```

## What minirextendr generates

- Autoconf scripts and configure wrappers (`configure.ac`, `configure`,
  `configure.win`)
- Rust files (`Cargo.toml`, `lib.rs`, `build.rs`)
- Build templates (`src/Makevars.in`, `src/stub.c`)
- Package docs and ignore files (`.Rbuildignore`, `.gitignore`)
- Vendored miniextendr crates under `vendor/`

## License

MIT (see `LICENSE`).
