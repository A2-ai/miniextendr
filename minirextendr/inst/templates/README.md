# miniextendr Templates

This directory contains the project templates consumed by `minirextendr`.
They are derived from the repo's `rpkg/` example package and kept in sync by
the template snapshot checks in the root `justfile`.

## Template types

### 1. `rpkg/` - standalone R package

A traditional R package with embedded Rust code in `src/rust/`.

```text
my.package/
├── DESCRIPTION
├── R/
│   └── my.package-package.R
├── src/
│   ├── Makevars.in
│   ├── stub.c
│   └── rust/
│       ├── Cargo.toml
│       ├── lib.rs
│       └── build.rs
├── configure.ac
└── bootstrap.R
```

Generated later during the build:

- `configure`
- `R/<package>-wrappers.R`
- `NAMESPACE` / man pages via roxygen

Use this template when:

- building an R package that happens to use Rust internally
- R is the primary interface
- you want the standard CRAN package workflow

Create it with:

```r
minirextendr::create_miniextendr_package("path/to/package")
minirextendr::use_miniextendr()
```

### 2. `monorepo/` - Rust workspace with embedded R package

A Rust workspace where the R package lives as a subdirectory, similar to this
repository.

```text
my-project/
├── Cargo.toml
├── justfile
├── my-crate/
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
└── rpkg/
    ├── DESCRIPTION
    ├── R/
    ├── src/
    │   ├── Makevars.in
    │   ├── stub.c
    │   └── rust/
    │       ├── Cargo.toml
    │       ├── lib.rs
    │       └── build.rs
    └── configure.ac
```

Use this template when:

- building a Rust project that also ships R bindings
- Rust is the primary project
- you want workspace-level dependency and tooling management

Create it with:

```r
minirextendr::create_miniextendr_monorepo("path/to/project")

minirextendr::create_miniextendr_monorepo(
  "path/to/project",
  crate_name = "my-lib",
  rpkg_name = "r-bindings"
)

minirextendr::use_miniextendr()
minirextendr::use_miniextendr(rpkg_name = "r-bindings")
```

## Template placeholders

Templates use mustache-style `{{variable}}` substitution:

- `{{package}}` - R package name
- `{{package_rs}}` - Rust-safe package name
- `{{Package}}` - title-cased package name
- `{{crate_name}}` - Rust crate name (monorepo only)
- `{{rpkg_name}}` - R package subdirectory name (monorepo only)
- `{{year}}` - current year

Additionally, `.in` files use autoconf `@PLACEHOLDER@` markers expanded during
`bash ./configure`:

- `@PACKAGE_TARNAME_RS@`
- `@PACKAGE_TARNAME_RS_UPPERCASE@`
- `@CARGO_STATICLIB_NAME@`

## Template snapshot testing

Use the root `justfile` to verify that templates have not drifted from `rpkg/`:

```bash
just templates-check
just templates-approve
```

`patches/templates.patch` captures the approved differences between the live
`rpkg/` package and the templated copies.

## Vendor tarball for CRAN

For offline/CRAN builds, vendored crates are compressed into
`inst/vendor.tar.xz`. In this repo the normal way to produce that tarball is
`just vendor`; in generated packages the equivalent helper is
`minirextendr::miniextendr_vendor()`.

During install, `configure` / `Makevars` use the tarball when `vendor/` is not
present, so end users do not need network access during build.
