# `miniextendr`

Experimental repository.

## Setup / Configuration

It is necessary to run

```shell
just configure
```

## Developer configuration

### `justfile`

To ensure that both the `miniextendr-*` crates are updated together with the
accompanying R-package `rpkg` and its embedded Rust crate `rpkg` we have a `justfile` runner, with a few noteworthy commands:

```shell
just --list
Available recipes:
    build *cargo_flags           # [alias: cargo-build]
    check *cargo_flags           # [alias: cargo-check]
    clean *cargo_flags           # [alias: cargo-clean]
    clippy *cargo_flags          # [alias: cargo-clippy]
    configure                    # Vendor rpkg deps and run ./configure
    default
    devtools-check               # Check rpkg with devtools::check
    devtools-document            # Document rpkg with devtools::document
    devtools-install             # Install rpkg with devtools::install
    devtools-load                # [alias: devtools-load_all]
    devtools-test FILTER=""      # Load and test rpkg with devtools
    doc *cargo_flags             # [alias: cargo-doc]
    doc-check *cargo_flags       # [alias: cargo-doc-check]
    expand *cargo_flags          # [alias: cargo-expand]
    fmt *cargo_flags             # [alias: cargo-fmt]
    fmt-check *cargo_flags       # [alias: cargo-fmt-check]
    r-cmd-build *args            # [alias: rcmdbuild]
    r-cmd-check *args            # [alias: rcmdcheck]
    r-cmd-install *args          # [alias: rcmdinstall]
    test *args                   # [alias: cargo-test]
    test-r-build                 # Build R package tarball
    tree *cargo_flags            # [alias: cargo-tree]
    vendor                       # [alias: cargo-vendor]
    vendor-rpkg                  # Patches Cargo.toml to remove workspace inheritance (not available when vendored)
```
