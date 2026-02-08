# miniextendr Templates

This directory contains two template types for creating projects with miniextendr.
These templates are consumed by the `minirextendr` package; see
`../../README.md` for usage.

## Template Types

### 1. `rpkg/` - Standalone R Package

A traditional R package with embedded Rust code in `src/rust/`.

**Structure:**

```
my.package/
├── DESCRIPTION
├── R/
│   └── my.package-package.R
├── src/
│   ├── Makevars.in
│   ├── entrypoint.c.in
│   └── rust/
│       ├── Cargo.toml
│       ├── lib.rs
│       ├── build.rs
│       └── document.rs.in
├── configure.ac
└── bootstrap.R
```

**When to use:**

- Building an R package that happens to use Rust internally
- R is the primary interface
- Standard CRAN submission workflow

**Create with:**

```r
# Create new standalone R package:
minirextendr::create_miniextendr_package("path/to/package")

# Add to existing R package:
minirextendr::use_miniextendr()  # Auto-detects rpkg template
```

### 2. `monorepo/` - Rust Workspace with Embedded R Package

A Rust workspace where the R package lives as a subdirectory. Similar to how this miniextendr repository itself is organized.

**Structure:**

```
my-project/
├── Cargo.toml              # Workspace root
├── justfile
├── my-crate/               # Main Rust library
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
└── rpkg/                   # R package
    ├── DESCRIPTION
    ├── R/
    ├── src/
    │   ├── Makevars.in
    │   ├── entrypoint.c.in
    │   └── rust/
    │       ├── Cargo.toml
    │       ├── lib.rs
    │       └── build.rs
    └── configure.ac
```

**When to use:**

- Building a Rust library that happens to have R bindings
- Rust is the primary project
- Want to use Rust workspace features
- Multiple Rust crates with one R package

**Create with:**

```r
# Create new monorepo from scratch:
minirextendr::create_miniextendr_monorepo("path/to/project")

# Customize names:
minirextendr::create_miniextendr_monorepo(
  "path/to/project",
  crate_name = "my-lib",
  rpkg_name = "r-bindings"  # Default: "rpkg"
)

# Add R package to existing Rust crate:
# (run from Rust crate directory)
minirextendr::use_miniextendr()  # Auto-detects monorepo template, creates rpkg/

# Customize R package directory name:
minirextendr::use_miniextendr(rpkg_name = "r-bindings")
```

## Template Placeholders

Templates use mustache-style `{{variable}}` substitution:

- `{{package}}` - R package name (e.g., "my.package")
- `{{package_rs}}` - Rust-safe variant (e.g., "my_package")
- `{{Package}}` - Title-cased (e.g., "My.Package")
- `{{crate_name}}` - Rust crate name (e.g., "my-crate") - monorepo only
- `{{rpkg_name}}` - R package subdirectory name (e.g., "rpkg", "r-bindings") - monorepo only
- `{{year}}` - Current year

Additionally, `.in` files use autoconf `@PLACEHOLDER@` markers that are expanded during `./configure`:

- `@PACKAGE_TARNAME_RS@` - Expands to Rust package name
- `@PACKAGE_TARNAME_RS_UPPERCASE@` - Uppercase version
- `@CARGO_STATICLIB_NAME@` - Extracted from Cargo.toml by configure

## Template Snapshot Testing

See the root `justfile` for snapshot testing recipes that verify templates stay in sync with the `rpkg/` source package:

```bash
just templates-check    # Verify templates haven't drifted
just templates-approve  # Accept current diffs as approved
```

The `patches/templates.patch` file captures approved differences between templates and rpkg sources (like placeholder markers).

## Vendor Tarball for CRAN

When submitting to CRAN, vendored crates are compressed into `inst/vendor.tar.xz`.
This file is typically 5-10MB and does NOT require Git LFS (which is for files >50MB).

The tarball is created during `./configure` when `NOT_CRAN` is not set, and
extracted during package installation. See the `minirextendr` README for details.
