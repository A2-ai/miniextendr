# cargo-revendor

Vendor Rust dependencies for offline/hermetic builds, including workspace and
path dependencies that plain `cargo vendor` skips.

This tool lives in `cargo-revendor/` and is intentionally excluded from the
main workspace so vendoring logic can evolve independently of the runtime
crates.

## Build and install

From the repo root:

```sh
just revendor-build
just revendor-test
cargo install --path cargo-revendor
```

Or directly with Cargo:

```sh
cargo build --manifest-path cargo-revendor/Cargo.toml
cargo test --manifest-path cargo-revendor/Cargo.toml
```

## Usage

```sh
# Basic: vendor all deps into vendor/
cargo revendor --manifest-path src/rust/Cargo.toml

# Full CRAN/offline prep: vendor, strip, freeze, compress
cargo revendor \
  --manifest-path src/rust/Cargo.toml \
  --output vendor \
  --strip-all \
  --freeze \
  --compress inst/vendor.tar.xz \
  --blank-md \
  --source-marker \
  -v
```

## What it does

1. Discovers dependencies via `cargo metadata`
2. Packages local crates via `cargo package`
3. Vendors external deps via `cargo vendor`
4. Extracts local crates into `vendor/`
5. Strips test/bench/example directories and TOML sections (opt-in)
6. Rewrites inter-crate path dependencies
7. Clears vendored checksums
8. Generates `.cargo/config.toml` with source replacement entries
9. Strips `Cargo.lock` checksums for vendored compatibility

## Flags

| Flag | Description |
|---|---|
| `--manifest-path` | Path to `Cargo.toml` (default: `src/rust/Cargo.toml`) |
| `--output` | Vendor directory (default: `vendor`) |
| `--source-root` | Workspace root for path-dependency discovery |
| `--strip-tests` | Strip `tests/` directories |
| `--strip-benches` | Strip `benches/` directories |
| `--strip-examples` | Strip `examples/` directories |
| `--strip-bins` | Strip binary targets |
| `--strip-all` | Strip all of the above |
| `--freeze` | Rewrite `Cargo.toml` to resolve everything from `vendor/` |
| `--compress <path>` | Compress `vendor/` into a `.tar.xz` tarball |
| `--blank-md` | Blank `.md` files before compression |
| `--source-marker` | Write `.vendor-source` provenance file |
| `--json` | Machine-readable JSON output |
| `--force` | Bypass cache and re-vendor unconditionally |
| `-v` / `-vv` / `-vvv` | Verbosity levels |

## `--freeze`

Rewrites `Cargo.toml` so workspace path deps resolve from `vendor/`.
Specifically it:

- rewrites `path = "..."` entries for local workspace crates to point at
  `vendor/<name>-<version>/`
- strips `[patch.*]` sections that reference external sources
- adds `[patch.crates-io]` entries for local vendored dependencies
- regenerates `Cargo.lock` from the frozen manifest with `--offline`

External `git = "..."` dependencies (deps that aren't workspace members)
are **not** rewritten — they remain as `git =` entries in the frozen
manifest and rely on cargo's source replacement
(`vendor/.cargo-config.toml`, which cargo-revendor always writes) to
resolve offline. Copy that file to `.cargo/config.toml` in the build
directory, or point cargo at it via `CARGO_HOME`, for `cargo build
--offline` to succeed.

Pass `--strict-freeze` to fail fast if any external git dep remains after
the freeze pass — useful for CI guards where the frozen manifest alone
must be buildable offline.

## Caching

cargo-revendor hashes `Cargo.lock` plus `Cargo.toml` and skips re-vendoring
when unchanged. Use `--force` to override.

## Path dependency handling

Unlike `cargo vendor`, cargo-revendor:

1. tries `cargo package` first
2. falls back to direct copy if packaging fails
3. resolves `*.workspace = true` fields in the fallback path
4. adds `version = "*"` to path-only dependencies that do not declare one
