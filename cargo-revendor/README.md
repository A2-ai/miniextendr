# cargo-revendor

Vendor Rust dependencies for offline/hermetic builds, including workspace and path dependencies that `cargo vendor` skips.

## Install

```sh
cargo install --path cargo-revendor
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

1. **Discovers dependencies** via `cargo metadata`
2. **Packages local crates** via `cargo package` (resolves workspace inheritance)
3. **Vendors external deps** via `cargo vendor` (crates-io, git sources)
4. **Extracts local crates** into vendor/ (overwrites cargo vendor placeholders)
5. **Strips** test/bench/example directories and TOML sections (opt-in)
6. **Rewrites** inter-crate path dependencies (`path = "../sibling"`)
7. **Clears checksums** (`.cargo-checksum.json` → `{"files":{}}`)
8. **Generates** `.cargo/config.toml` with source replacement entries
9. **Strips** Cargo.lock checksums for vendored compatibility

## Flags

| Flag | Description |
|---|---|
| `--manifest-path` | Path to Cargo.toml (default: `src/rust/Cargo.toml`) |
| `--output` | Vendor directory (default: `vendor`) |
| `--source-root` | Workspace root for path dep discovery |
| `--strip-tests` | Strip `tests/` directories |
| `--strip-benches` | Strip `benches/` directories |
| `--strip-examples` | Strip `examples/` directories |
| `--strip-bins` | Strip binary targets |
| `--strip-all` | Strip all of the above |
| `--freeze` | Rewrite Cargo.toml to resolve everything from vendor/ |
| `--compress <path>` | Compress vendor/ into a `.tar.xz` tarball |
| `--blank-md` | Blank `.md` files before compression |
| `--source-marker` | Write `.vendor-source` provenance file |
| `--json` | Machine-readable JSON output |
| `--force` | Bypass cache, re-vendor unconditionally |
| `-v` / `-vv` / `-vvv` | Verbosity levels |

## `--freeze`

Rewrites `Cargo.toml` so the manifest is self-contained — all sources resolve from `vendor/` alone, with no network, git, or workspace context needed.

Specifically:
- Rewrites `git = "https://..."` deps to `path = "../../vendor/<name>"`
- Strips all `[patch.*]` sections (they reference external sources)
- Adds `[patch.crates-io]` with vendor paths for transitive local deps
- Regenerates `Cargo.lock` from the frozen manifest (`--offline`)

After `--freeze`, `cargo build --offline` works with only the vendor directory.

## Caching

cargo-revendor hashes `Cargo.lock` + `Cargo.toml` and skips re-vendoring when unchanged. Use `--force` to override.

## Path dependency handling

Unlike `cargo vendor` which silently skips path dependencies, cargo-revendor:
1. Tries `cargo package` (resolves workspace inheritance via the packaging pipeline)
2. Falls back to direct copy if packaging fails (unpublished deps)
3. Resolves `*.workspace = true` fields via `toml_edit` in the fallback path
4. Auto-adds `version = "*"` to path-only deps that lack a version
