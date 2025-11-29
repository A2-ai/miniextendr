# `miniextendr`

Experimental repository.

## Setup / Configuration

It is necessary to run

```shell
R CMD INSTALL .
```

to setup this project, as the `rpkg/configure` script in the embedded R-package
`{rpkg}` also sets up `cargo` configurations!

Or atleast run

```sh
./rpkg/configure
```

## Developer configuration

If you alter `configure.ac`, then a `/configure`-script has to be recompiled, and that is done via

```shell
autoreconf -vif
```

The `/configure`-script is supposed to be built during developer-time, and thus
users are not expected to run `autoconf`/`autoreconf`.

### `justfile`

To ensure that both the `miniextendr-*` crates are updated together with the
accompanying R-package `rpkg` and its embedded Rust crate `rpkg` we have a `justfile` runner, with a few noteworthy commands:

```shell
just --list
Available recipes:
    build *cargo_flags        # Build all crates
    check *cargo_flags        # Check all crates`` [alias: cargo-check]
    clean *cargo_flags        # Clean build artifacts
    clippy *cargo_flags       # Run clippy on all crates
    configure                 # Run configure for rpkg
    default
    devtools-check            # Check rpkg with devtools::check
    devtools-document         # Document rpkg with devtools::document
    devtools-install          # Install rpkg with devtools::install
    devtools-test FILTER=""   # Load and test rpkg with devtools
    doc *cargo_flags          # Build and open documentation [alias: cargo-doc]
    doc-check *cargo_flags    # Check documentation builds
    expand *cargo_flags       # Expand macros for rpkg (requires cargo-expand) [alias: cargo-expand]
    fmt *cargo_flags          # Format all code [alias: cargo-fmt]
    fmt-check *cargo_flags    # Check formatting
    load                      # Load rpkg with devtools::load_all
    r-cmd-build *args         # [alias: rcmdbuild]
    r-cmd-check *args         # [alias: rcmdcheck]
    r-cmd-install *args       # [alias: rcmdinstall]
    smoke-test                # Quick smoke test of rpkg functions
    test *args                # Run tests
    test-r-build              # Build R package tarball
    tree *cargo_flags         # Show dependency tree
    vendor                    # Vendor dependencies
```
