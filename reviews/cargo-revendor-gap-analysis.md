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
| Strip test/bench dirs | opt-in | always | — |
| TOML section stripping | Y | Y | — |
| Empty checksum gen | Y | Y | — |
| Cargo.lock checksum strip | Y | Y | Y |
| .cargo/config.toml gen | Y | — | template |
| Git source replacement | Y (auto-detect) | — | Y (sed scan) |
| Caching | Y (Cargo.lock hash) | — | — |
| Source tracking (.vendor-source) | **NO** | Y | — |
| JSON output | Y | — | — |
| Compress to tarball | **NO** | Y | Y (if missing) |
| Unpack vendor.tar.xz | **NO** | — | Y |
| CRAN Cargo.toml rewriting | **NO** | — | Y (git→path, strip [patch]) |
| Lockfile regeneration | **NO** | — | Y (generate-lockfile --offline) |
| .md file blanking | **NO** | Y | — |
| Fallback for package failure | Y (direct copy) | — | — |
| Workspace inheritance (fallback) | Y (toml_edit) | regex | — |
| Atomic vendor swap | — | Y (backup+rename) | — |

## Gaps to close

### 1. Tarball output (critical)
cargo-revendor produces vendor/ but doesn't compress to inst/vendor.tar.xz.
**Fix**: Add `--compress` flag that produces the tarball directly.

### 2. CRAN mode (critical)
configure.ac does three things cargo-revendor doesn't:
- Rewrites `git = "..."` deps to `path = "../../vendor/<name>"` in Cargo.toml
- Strips `[patch.crates-io]` section
- Adds `[patch.crates-io]` with vendor path entries for transitive deps
- Regenerates Cargo.lock from vendored sources offline

**Fix**: Add `--cran` flag that performs these rewrites on the vendored output.
Or: generate a CRAN-ready Cargo.toml alongside the vendor directory.

### 3. Source tracking
vendor-crates.R writes `.vendor-source` marker to record where vendor/ came from.
cargo-revendor doesn't track provenance.
**Fix**: Write source info to `.revendor-cache` or a separate marker.

### 4. Unpack tarball
configure.ac unpacks inst/vendor.tar.xz into vendor/ during CRAN builds.
cargo-revendor doesn't have an unpack mode.
**Fix**: Not needed — this stays in configure.ac (it's a build-time concern).

### 5. .md blanking
vendor-crates.R blanks .md files before tarball (avoids CRAN NOTEs).
cargo-revendor doesn't touch .md files.
**Fix**: Add to the `--compress` implementation.

### 6. Lockfile regeneration
configure.ac runs `cargo generate-lockfile --offline` in CRAN mode.
cargo-revendor only strips checksums.
**Fix**: Add `--regenerate-lockfile` flag or include in `--cran` mode.

## What can cargo-revendor replace today?

**Can replace**: vendor-crates.R's `sync` command (dependency discovery, packaging,
extraction, stripping, path rewriting, checksum clearing).

**Cannot replace yet**:
- Tarball creation (vendor-crates.R's `pack` = sync + compress)
- CRAN Cargo.toml rewriting (configure.ac)
- Lockfile regeneration (configure.ac)
- Tarball unpacking (configure.ac / Makevars)

## Recommended next steps

1. Add `--compress <path>` to produce inst/vendor.tar.xz directly
2. Add `--cran` mode that rewrites Cargo.toml for offline builds
3. Add `--source-marker` to write .vendor-source
4. Then: vendor-crates.R `pack` becomes `cargo revendor --compress inst/vendor.tar.xz --strip-all`
5. configure.ac cargo-vendor block becomes: unpack tarball + `cargo revendor --cran` (or stays as-is)
