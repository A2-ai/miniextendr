# minirextendr

minirextendr is an R helper package for scaffolding and maintaining R packages
that use the miniextendr Rust integration. It provides templates, autoconf /
configure wiring, vendoring helpers, and cargo command wrappers tailored to
R package workflows.

## Installation

From GitHub:

```r
remotes::install_github("CGMossa/miniextendr", subdir = "minirextendr")
```

If you're using a fork, replace the owner/repo in the install command.

From a local checkout:

```r
devtools::install("minirextendr")
```

## Requirements

- Rust toolchain (>= 1.85) on PATH
- autoconf
- A working R build toolchain (R, headers, compiler)

## Quick start

Create a new miniextendr-enabled package:

```r
library(minirextendr)
create_miniextendr_package("path/to/pkg")
```

Or add miniextendr scaffolding to an existing package:

```r
library(minirextendr)
use_miniextendr()
```

Generate build files and wrappers:

```r
miniextendr_autoconf()
miniextendr_configure()
miniextendr_document()
```

Or run the full workflow:

```r
miniextendr_build()
```

## Templates

See `inst/templates/README.md` for the standalone package and monorepo layouts
used by the scaffolder.

## Status and validation

```r
miniextendr_status()
miniextendr_check()
```

## Vendoring

Vendoring miniextendr crates:

```r
miniextendr_available_versions()
vendor_miniextendr("main")
```

Vendoring external crates.io dependencies:

```r
vendor_crates_io()
```

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

- Autoconf scripts and configure wrappers (configure.ac, configure, configure.win)
- Rust templates (Cargo.toml.in, lib.rs, build.rs, document.rs.in)
- Build templates (src/Makevars.in, src/entrypoint.c.in)
- Package docs and ignore files (.Rbuildignore, .gitignore)
- Vendored miniextendr crates under src/vendor/

## License

MIT (see LICENSE).
