# Environment Variables

All environment variables that affect miniextendr's build, configure, test, and lint processes.

## Install mode

There is no env var that controls install mode. configure auto-detects from a
single signal:

| Mode | When | Behavior |
|---|---|---|
| Source | `inst/vendor.tar.xz` absent | Cargo resolves via `[patch."git+url"]` to monorepo siblings, or fetches the git URL when no siblings are detected. |
| Tarball | `inst/vendor.tar.xz` present | Configure unpacks the tarball, writes `[source]` replacement to `vendored-sources`, build runs `--offline`. |

`NOT_CRAN`, `PREPARE_CRAN`, `FORCE_VENDOR`, `BUILD_CONTEXT` are removed. See
[CRAN Compatibility](CRAN_COMPATIBILITY.md) for the full table and rationale.

## Cargo & Rust

| Variable | Purpose | Default |
|----------|---------|---------|
| `MINIEXTENDR_FEATURES` | Comma-separated cargo features to enable | All features except `nonapi` |
| `CARGO_PROFILE` | Build profile: `dev` or `release` | `release` |
| `CARGO_TARGET_DIR` | Artifact directory (must be outside `src/`) | `${abs_top_srcdir}/rust-target` |
| `CARGO_BUILD_TARGET` | Rust target triple for cross-compilation | Empty (native); auto-detected from autoconf host |
| `CARGO_HOME` | Cargo registry/cache directory for offline or restricted environments | Empty (cargo's default, `~/.cargo`) |
| `RUST_TOOLCHAIN` | Toolchain selector (e.g., `+stable`, `+nightly`) | Empty (system default) |
| `ENV_RUSTFLAGS` | Rust compiler flags, passed as `RUSTFLAGS` to cargo | Value of `RUSTFLAGS` |

All of the above are declared as `AC_ARG_VAR` in `configure.ac` and can be set when invoking `./configure`:

```bash
cd rpkg && MINIEXTENDR_FEATURES="rayon,serde" CARGO_PROFILE=dev bash ./configure
```

## R Installation

| Variable | Purpose | Default |
|----------|---------|---------|
| `R_HOME` | Path to R installation | Auto-detected via `R RHOME` |
| `R_LIBS` | R library path for package installation | System default |

## Lint

| Variable | Purpose | Default |
|----------|---------|---------|
| `MINIEXTENDR_LINT` | Disable lint: `0`, `false`, `no`, or `off` | Enabled |

The lint runs automatically during `cargo build`/`cargo check` via `build.rs`. Disable with:

```bash
MINIEXTENDR_LINT=0 cargo check --manifest-path=rpkg/src/rust/Cargo.toml
```

## Runtime

| Variable | Purpose | Default |
|----------|---------|---------|
| `MINIEXTENDR_BACKTRACE` | Show full Rust backtraces on panic: `1` or `true` (case-insensitive) | Suppressed |
| `MINIEXTENDR_ENCODING_DEBUG` | Print encoding snapshot at init (any value enables) | Not set |

`MINIEXTENDR_BACKTRACE` is read at panic time, not at package load, so it can be toggled
during a session without restarting R. See [Error Handling: Panic Hook and Backtraces](ERROR_HANDLING.md#panic-hook-and-backtraces).

`MINIEXTENDR_ENCODING_DEBUG` is only useful when embedding R via `miniextendr-engine` or on platforms where non-API
encoding symbols are exported. See [Encoding](ENCODING.md).

## minirextendr (Scaffolding)

| Variable | Purpose | Default |
|----------|---------|---------|
| `MINIEXTENDR_LOCAL_PATH` | Path to local miniextendr monorepo for tests/scaffolding | Auto-detected |

## Bootstrap (Internal)

These are set automatically by `bootstrap.R` during `R CMD INSTALL` and shouldn't be set manually:

| Variable | Purpose |
|----------|---------|
| `CC`, `CFLAGS`, `CXX`, `CXXFLAGS`, `CPPFLAGS`, `LDFLAGS` | C/C++ toolchain from `R CMD config` |
| `_R_SHLIB_BUILD_OBJECTS_SYMBOL_TABLES_` | Symbol table generation (set to `false`) |

## Cargo-Internal (Set Automatically)

These are set by cargo/build.rs and not meant for manual use:

| Variable | Purpose |
|----------|---------|
| `CARGO_MANIFEST_DIR` | Directory containing Cargo.toml |
| `CARGO_CFG_TARGET_OS` | Target OS (windows, macos, linux) |
| `CARGO_CFG_TARGET_ENV` | Target environment (msvc, gnu, musl) |
| `CARGO_FEATURE_*` | One per enabled feature (uppercase + underscore) |
