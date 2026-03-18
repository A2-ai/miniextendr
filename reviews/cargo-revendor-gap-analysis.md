# cargo-revendor gap analysis

Comparison of cargo-revendor vs vendor-crates.R + configure.ac for vendor responsibilities.

## Responsibility matrix

| Responsibility | cargo-revendor | vendor-crates.R | configure.ac |
|---|---|---|---|
| Dependency discovery | cargo metadata | cargo tree (text) | — |
| Local crate detection | metadata filter | tree + path filter | — |
| cargo package (workspace resolution) | Y | Y | — |
| cargo vendor (externals) | Y | Y | — |
| Extract .crate archives | Y | Y | — |
| Path dep rewriting | toml_edit | regex | sed |
| Strip test/bench dirs | opt-in (`--strip-*`) | always | — |
| TOML section stripping | Y | Y | — |
| Empty checksum gen | Y | Y | — |
| Cargo.lock checksum strip | Y | Y | Y |
| .cargo/config.toml gen | Y | — | template |
| Git source replacement | Y (auto-detect) | — | Y (sed scan) |
| Caching | Y (Cargo.lock hash) | — | — |
| Source tracking (.vendor-source) | Y (`--source-marker`) | Y | — |
| JSON output | Y (`--json`) | — | — |
| Compress to tarball | Y (`--compress`) | Y | Y (if missing) |
| Unpack vendor.tar.xz | — (build-time) | — | Y |
| Freeze manifest for offline | Y (`--freeze`) | — | Y (git→path, strip [patch]) |
| Lockfile regeneration | Y (part of `--freeze`) | — | Y (generate-lockfile --offline) |
| .md file blanking | Y (`--blank-md`) | Y | — |
| Fallback for package failure | Y (direct copy) | — | — |
| Workspace inheritance (fallback) | Y (toml_edit) | regex | — |
| Atomic vendor swap | — | Y (backup+rename) | — |

## Status

All critical gaps are closed. cargo-revendor can now fully replace `vendor-crates.R pack`:

```sh
cargo revendor \
  --manifest-path src/rust/Cargo.toml \
  --strip-all --freeze \
  --compress inst/vendor.tar.xz \
  --blank-md --source-marker -v
```

## What stays in configure.ac

- **Unpack** vendor.tar.xz at build time (CRAN installs don't have vendor/)
- **Dev/CRAN mode detection** (NOT_CRAN env var)
- **Git source replacement** in .cargo/config.toml (CRAN mode, dynamic scan)
- **Cargo.lock compatibility** check (older cargo versions)

These are build-time concerns, not vendor-time. cargo-revendor handles vendor-time;
configure.ac handles build-time.
