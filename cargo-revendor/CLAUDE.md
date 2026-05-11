# cargo-revendor

Standalone `cargo` subcommand for R/CRAN-friendly vendoring. **Excluded from the miniextendr workspace** — has its own `Cargo.toml`/`Cargo.lock`/`target/`. See root `CLAUDE.md` for shared rules.

## Why standalone
End users install via `cargo install cargo-revendor`; it must build without dragging in the miniextendr workspace `[patch."git+url"]` table. Inclusion in the parent workspace would break that.

## Dev loop
- `just revendor-build` — builds against this crate's own manifest.
- `just revendor-test` — runs `cargo test` here.
- Never `cargo --workspace`-it from the root; the root manifest doesn't include it.

## Key features
- **`--freeze`** — resolves `Cargo.toml` against the local `vendor/` only (writes `path = "../../vendor/..."` into `[dependencies]` and `[patch.crates-io]`). Not invoked by `just vendor` (removed; see `docs/CRAN_COMPATIBILITY.md`).
- **`--sync`** — refreshes vendor/ from a Cargo.lock without re-resolving versions.
- **`--versioned-dirs`** — opt-in for now; #239 tracks making it default.
- **`cargo package` for workspace resolution** — let cargo expand workspace inheritance; never hard-code workspace dependency replacements.

## When the tarball arrives without `cargo-revendor` on PATH
CRAN's offline farm has no `cargo-revendor`. The configure.ac auto-vendor branch is short-circuited; a maintainer who shipped a tarball without `inst/vendor.tar.xz` fails CRAN's offline check loudly. Intended canary.

