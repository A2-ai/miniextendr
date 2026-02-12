# minirextendr

minirextendr is an R helper package for scaffolding and maintaining R packages
that use miniextendr for Rust <-> R interop. It provides templates, autoconf /
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

Both standalone and monorepo templates use git dependencies for miniextendr crates.
For monorepo development (where you want to use local miniextendr crates instead of
git), add a `[patch."https://..."]` section to your Cargo.toml:

```toml
[patch."https://github.com/CGMossa/miniextendr"]
miniextendr-api = { path = "../path/to/miniextendr-api" }
miniextendr-macros = { path = "../path/to/miniextendr-macros" }
miniextendr-macros-core = { path = "../path/to/miniextendr-macros-core" }
miniextendr-lint = { path = "../path/to/miniextendr-lint" }
```

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

Downloaded archives are cached to avoid repeated downloads:

```r
miniextendr_cache_info()      # Show cached versions
miniextendr_clear_cache()     # Clear all cached archives
vendor_miniextendr("main", refresh = TRUE)  # Force re-download
```

Vendoring external crates.io dependencies:

```r
vendor_crates_io()
```

### Vendor tarball and Git LFS

For CRAN submission, the vendored crates are compressed into `inst/vendor.tar.xz`.
This file is typically 5-10MB depending on enabled features.

**Git LFS is NOT required** for this file because:
- The file size (~7MB) is well below GitHub's 100MB limit
- It changes infrequently (only when updating vendored crate versions)
- Binary diffs work reasonably well for compressed archives

If your vendor tarball grows significantly larger (>50MB), consider:
1. Reducing enabled features in `Cargo.toml` to minimize dependencies
2. Using Git LFS: `git lfs track "inst/vendor.tar.xz"`
3. Hosting the tarball externally and downloading during `R CMD build`

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
- Rust files (Cargo.toml, lib.rs, build.rs, document.rs.in)
- Build templates (src/Makevars.in, src/entrypoint.c.in)
- Package docs and ignore files (.Rbuildignore, .gitignore)
- Vendored miniextendr crates under vendor/

## License

MIT (see LICENSE).
