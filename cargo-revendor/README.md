# cargo-revendor

A `cargo` subcommand that vendors Rust dependencies for offline and hermetic
builds, with first-class support for the cases plain `cargo vendor` does not
handle: workspace path dependencies, monorepo siblings, R package layouts,
and CRAN-grade trim and freeze passes.

## Why this exists

Plain `cargo vendor` was designed for a single workspace whose dependencies
all resolve from `crates.io` or git. It has three blind spots that matter for
R packages and monorepos:

1. **Path dependencies are skipped.** A workspace member at `path = "../foo"`
   stays as a path reference; nothing is copied into `vendor/`. When the
   package is shipped to CRAN or built from a tarball, the path no longer
   resolves and the build fails.
2. **Workspace inheritance survives in the output.** Each vendored
   `Cargo.toml` keeps `version.workspace = true` and similar inherited
   fields, so the vendored crate cannot build standalone outside its
   original workspace.
3. **No trimming, freezing, or compression.** CRAN tarballs need to be
   small, self-contained, and reproducible from `Cargo.lock` alone. `cargo
   vendor` gives you a directory; the rest is up to you.

`cargo-revendor` is a superset that fills those gaps while remaining a thin
orchestrator over `cargo metadata`, `cargo package`, and `cargo vendor`. It
calls cargo for the parts cargo does well, and only does the additional
work cargo does not.

## Layout in this repo

`cargo-revendor/` is a **standalone Cargo workspace**, deliberately excluded
from the root miniextendr workspace. This keeps its dependency graph
independent of the runtime crates so vendoring logic can evolve without
touching the runtime `Cargo.lock`.

```
cargo-revendor/
├── Cargo.toml         # standalone workspace, not part of miniextendr
├── README.md          # this file
└── src/
    ├── main.rs            # CLI parsing, mode dispatch, full pipeline orchestration
    ├── metadata.rs        # cargo metadata wrapper, package partitioning, dup-source detection
    ├── package.rs         # invokes cargo package per local crate, resolves workspace inheritance
    ├── vendor.rs          # invokes cargo vendor, extracts archives, freeze + compress
    ├── strip.rs           # removes tests/benches/examples/bins and matching TOML sections
    ├── verify.rs          # CI-only check: lockfile vs vendor/ vs tarball agreement
    ├── cache.rs           # FNV-hash gate over Cargo.lock and local crate sources
    └── manifest_guard.rs  # RAII restore for transient Cargo.toml mutations
```

`tests/` contains an integration matrix exercising every CLI flag combination
against fixture workspaces (registry deps, git deps, multi-workspace `--sync`,
phase modes, freeze, compress, verify).

## Build and install

From the miniextendr repo root:

```sh
just revendor-build           # cargo build for the standalone workspace
just revendor-test            # full integration test matrix
cargo install --path cargo-revendor
```

Or directly:

```sh
cargo build  --manifest-path cargo-revendor/Cargo.toml
cargo test   --manifest-path cargo-revendor/Cargo.toml
```

After `cargo install`, invoke as a cargo subcommand: `cargo revendor ...`.

## Quick start

The common case is one command:

```sh
cargo revendor --manifest-path src/rust/Cargo.toml
```

This populates `vendor/` and writes `vendor/.cargo-config.toml` with the
source-replacement entries needed to build offline. To use it, copy that
file to `.cargo/config.toml` in the build root, or point `CARGO_HOME` at
it.

The full CRAN release recipe pulls in trim, freeze, compression, and
provenance:

```sh
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

Each flag is explained below.

## Pipeline (what happens internally)

`run_full` in `main.rs` orchestrates the steps in order. The pipeline is the
same for every invocation; flags toggle individual steps on or off.

1. **Bootstrap seed** (only when `--source-root` is set). The frozen target
   manifest may contain `path = "../../vendor/<name>-<ver>/"` entries that
   do not resolve on a fresh clone. cargo-revendor copies the workspace
   source into the expected vendor paths so `cargo metadata` succeeds. The
   seed is later overwritten by canonical `cargo package` output.
2. **Load metadata.** `cargo metadata` resolves the dep graph and surfaces
   any duplicate source conflicts (two git URLs producing the same
   name+version are an error, mirroring upstream cargo).
3. **Partition packages.** Local workspace crates go to one bucket,
   external deps (registry + git) go to the other.
4. **Cache gate.** A 64-bit FNV hash over `Cargo.lock`, every `--sync`
   manifest's lockfile, and the source tree of every local crate is
   compared against `vendor/.revendor-cache`. On a hit, the run exits with
   a "cached" message and zero file system changes.
5. **`cargo package`** runs once per local crate. This is the step that
   resolves workspace inheritance; the `.crate` archive contains a flat,
   standalone `Cargo.toml`. A transient `[patch.crates-io]` block on the
   target manifest, guarded by `ManifestGuard`, lets local crates reference
   each other during packaging without polluting the user's manifest.
6. **`cargo vendor`** runs against the target manifest plus any `--sync`
   manifests. External crates land in the staging directory.
7. **Extract local archives.** Each `.crate` from step 5 is untarred into
   `vendor/<name>/`, replacing whatever placeholder `cargo vendor` left.
8. **Strip** (opt-in). Tests, benches, examples, bins, and matching TOML
   sections are removed. Always-safe directories (`.github`, `.circleci`,
   `ci`, `target`) are removed unconditionally.
9. **Strip vendor path deps.** `cargo vendor` preserves intra-workspace
   `path = "../sibling"` entries from git sources. These conflict with
   source replacement at build time, so they are removed.
10. **Rewrite local path deps.** `path = "../foo"` entries in vendored
    local crates are rewritten to point at sibling vendor directories.
11. **Clear checksums.** Vendored crates get empty `.cargo-checksum.json`
    files. Cargo accepts these for source-replaced builds.
12. **Move staging to output.** Atomic where possible (`rename`), recursive
    copy fallback for cross-filesystem moves.
13. **Generate `.cargo/config.toml`.** Source-replacement entries point
    cargo at `vendor/` instead of crates.io and any git remotes.
14. **Strip lockfile checksums.** Cargo refuses to build with `--offline`
    against vendored sources if the lockfile still carries registry
    checksums.
15. **Source marker** (opt-in). Writes `.vendor-source` recording where the
    vendor came from (explicit `--source-root` or auto-detected).
16. **Freeze** (opt-in). See the freeze section below.
17. **Compress** (opt-in). Tars and xz-compresses `vendor/` to the path
    given to `--compress`. `COPYFILE_DISABLE=1` is set to suppress macOS
    xattr warnings on Linux GNU tar consumers.
18. **Save cache.** Three cache files are written: full, external-only, and
    local-only. Subsequent runs short-circuit at step 4.

## Flags

### Required input

| Flag | Description |
|---|---|
| `--manifest-path <PATH>` | `Cargo.toml` of the target package. Defaults to `src/rust/Cargo.toml`, then `./Cargo.toml`, then any single `*/src/rust/Cargo.toml` subdirectory. The autodetection covers running from the R package root, the Rust crate root, and the monorepo root. |

### Output location

| Flag | Description |
|---|---|
| `--output <DIR>` | Vendor directory (default `vendor`). Relative paths resolve from CWD, not the manifest dir. |
| `--source-root <DIR>` | Workspace root containing path dependencies. Auto-detected from metadata when omitted. Set explicitly when bootstrapping a frozen tree on a fresh clone. |

### Trimming

These flags strip code that is not needed at build time. Stripping exists
because CRAN tarballs over a few megabytes draw reviewer attention, and
test fixtures sometimes carry extra licenses.

| Flag | Description |
|---|---|
| `--strip-tests` | Remove `tests/` directories and `[[test]]` sections from `Cargo.toml`. |
| `--strip-benches` | Remove `benches/` and `[[bench]]`. |
| `--strip-examples` | Remove `examples/` and `[[example]]`. |
| `--strip-bins` | Remove `[[bin]]` targets that are not needed when building a library. |
| `--strip-all` | Convenience: all four of the above plus `[dev-dependencies]`. |
| `--strip-toml-sections` | TOML-only variant. Strips `[[test]]`, `[[bench]]`, `[[example]]`, `[[bin]]`, `[dev-dependencies]` but **leaves the source directories on disk**. Use when a dep references files in `tests/` or `examples/` from regular library source via `include_str!()` (zerocopy is the canonical example). Mutually exclusive with the other strip flags. |

### Freeze

| Flag | Description |
|---|---|
| `--freeze` | Rewrite `Cargo.toml` so every source resolves from `vendor/`. See dedicated section below. |
| `--strict-freeze` | Fail fast if any external `git = "..."` dependency would survive the freeze pass. Requires `--freeze`. Useful as a CI guard. |

### Compression

| Flag | Description |
|---|---|
| `--compress <PATH>` | After vendoring, compress `vendor/` to a `.tar.xz` at the given path. Relative paths resolve from CWD. |
| `--blank-md` | Truncate `.md` files in `vendor/` to zero bytes before compression. Reduces tarball size by 5 to 15 percent on typical dep graphs. The on-disk `vendor/` directory is unaffected. |

### Phase modes

Phase modes split a vendor pass into independently cacheable halves. This
helps in CI where the external dep set rebuilds rarely (it changes only
with `Cargo.lock`) but local crate sources change every commit.

| Flag | Description |
|---|---|
| `--external-only` | Vendor only registry and git deps. Local crate directories are never written. Incompatible with `--freeze`, `--compress`, `--source-marker`, `--blank-md`, and `--strict-freeze` (those need a complete tree). |
| `--local-only` | Vendor only workspace crates. External dirs are left as-is. When combined with the flags above, requires that an `--external-only` pass has already populated externals (checked via `.revendor-cache-external`). |

### Multi-workspace

| Flag | Description |
|---|---|
| `--sync <PATH>` | Additional `Cargo.toml` to union into the same vendor tree. Mirrors `cargo vendor --sync`. Each sync manifest's `Cargo.lock` participates in the cache key and in `--verify`. Use case: one R package plus a separate benchmarks workspace sharing one offline artifact. |

### Layout

| Flag | Description |
|---|---|
| `--flat-dirs` | Use `vendor/<name>/` for every crate instead of the default `vendor/<name>-<version>/`. The versioned default is unambiguous across regenerations and aligns with what `cargo vendor --versioned-dirs` produces. Use the flag only when downstream tools hardcode flat paths. |

### Verification (CI mode)

| Flag | Description |
|---|---|
| `--verify` | Verify-only: do not vendor. Asserts that `Cargo.lock` matches the contents of `vendor/`, and (with `--compress`) that the tarball matches `vendor/` byte-for-byte. Exits non-zero on any drift. Run in CI before release to catch a stale committed tarball. |

### Operational

| Flag | Description |
|---|---|
| `--allow-dirty` | Forwarded to `cargo package`. Default `true` because vendor regeneration commonly happens on a working tree with edits. |
| `--source-marker` | Write `.vendor-source` recording provenance (the `--source-root` value or `auto-detected`). |
| `--json` | Machine-readable JSON output: counts, local crate names, cache hit, list of stripped directories. Goes to stdout; human output goes to stderr. |
| `--force` | Ignore the cache and re-vendor unconditionally. |
| `-v` / `-vv` / `-vvv` | Increasing verbosity. `info` reports steps, `debug` reports per-package decisions, `trace` reports per-file actions. |

## `--freeze` in detail

Freezing rewrites the target `Cargo.toml` so the build graph resolves
**entirely** from `vendor/`, with no further reliance on `crates.io`, git,
or workspace context.

Specifically, freeze:

1. Rewrites `path = "..."` entries for local workspace crates to point at
   `vendor/<name>-<version>/`.
2. Strips `[patch.*]` sections that reference external sources. Those
   patches were how the unfrozen build resolved git overrides; once
   everything lives in `vendor/`, patches are noise that confuses cargo.
3. Adds `[patch.crates-io]` entries for local vendored dependencies so
   transitive deps from external crates resolve to the vendored copy
   instead of attempting a registry fetch.
4. Regenerates `Cargo.lock` from the frozen manifest with `--offline`. This
   normalizes the lockfile into the shape it will have at build time and
   surfaces any unresolvable refs immediately.

External `git = "..."` dependencies (deps that are **not** workspace
members) are not rewritten. They remain `git =` entries in the frozen
manifest and rely on cargo's source replacement
(`vendor/.cargo-config.toml`, which cargo-revendor always writes) to
resolve offline. Copy that file to `.cargo/config.toml` in the build root,
or point `CARGO_HOME` at it, for `cargo build --offline` to succeed.

Pass `--strict-freeze` to fail fast when an external git dep would survive
the freeze pass. CI gates that need to guarantee the manifest alone is
buildable offline (without source replacement) should use `--strict-freeze`.

## Caching

cargo-revendor maintains three cache files in `vendor/`. Each stores a
64-bit FNV hash over its inputs.

| File | Hashed inputs | Written by | Checked by |
|---|---|---|---|
| `.revendor-cache` | `Cargo.lock` + `Cargo.toml` + every `--sync` manifest's lockfile + source tree of every local crate | full mode | full mode |
| `.revendor-cache-external` | `Cargo.lock` + `Cargo.toml` + sync manifests | full and `--external-only` | `--external-only`, plus `--local-only` flag-compat checks |
| `.revendor-cache-local` | source tree of every local crate | full and `--local-only` | `--local-only` |

The local source tree contributes to the cache key because pure source
edits to workspace crates leave `Cargo.lock` untouched (issue #150).
Hashing only the lockfile would silently serve a stale `vendor/`.

`--force` bypasses the cache check.

## Path dependency handling

This is the one area where cargo-revendor diverges meaningfully from
`cargo vendor`:

1. **`cargo package` first.** Each local crate is packaged individually,
   producing a `.crate` archive with workspace inheritance fully resolved
   in its `Cargo.toml`.
2. **Direct copy fallback.** If `cargo package` fails (for example, a
   crate with `publish = false` and an unresolvable `*.workspace = true`
   field under unusual feature flag combinations), the source tree is
   copied verbatim. The fallback path resolves workspace inheritance
   manually.
3. **Add missing versions.** Path-only dependencies that do not declare a
   `version` field have one synthesized (`version = "*"`). Cargo refuses
   to resolve a path dep without a version when source replacement is
   active.
4. **Rewrite intra-workspace paths.** After extraction, `path = "../sibling"`
   in vendored local crates is rewritten to point at the sibling's vendor
   directory.

## Common gotchas

These are the failure modes documented across the integration tests and
issue tracker:

- **macOS xattrs in tarballs.** `COPYFILE_DISABLE=1` is set automatically
  during compression to suppress Apple metadata that breaks Linux and
  Windows GNU tar.
- **Windows paths in TOML.** All paths are normalized to forward slashes
  and the `\\?\` extended-length prefix from `canonicalize()` is stripped
  before being written into any `.toml` file.
- **Dirty manifest after a crash.** `ManifestGuard` snapshots `Cargo.toml`
  before any transient `[patch.crates-io]` mutation and restores it on
  drop, including on panic and `?` propagation. The only uncovered case
  is `SIGKILL` and `std::process::abort()`, where Drop does not run.
- **Stale frozen vendor after merging main.** A frozen `Cargo.toml`
  carries `path = "../../vendor/..."` entries that go stale when the
  workspace diverges. To recover, reset the frozen path deps to `"*"`,
  delete `vendor/` and `Cargo.lock`, and re-run.
- **Crates that `include_str!()` from `tests/` or `examples/`.** Use
  `--strip-toml-sections` rather than `--strip-all` so the directories
  stay on disk and `cargo check --offline` keeps working.

## Module-by-module reference

### `main.rs`

CLI parsing (`Cli` via clap derive), phase-mode dispatch (`Mode::Full`,
`ExternalOnly`, `LocalOnly`), and the three top-level pipelines:
`run_full`, `run_external_only`, `run_local_only`. Also hosts the manifest
autodetect logic, JSON output struct, and the seeding helper that copies
workspace sources into vendor paths so `cargo metadata` can resolve frozen
path deps on a fresh clone.

### `metadata.rs`

Wraps `cargo_metadata::MetadataCommand`. Exposes `LocalPackage` (a thin
record of name, version, path, manifest path), `discover_workspace_members`
for source-root walks, `partition_packages` for the local-vs-external
split, and `check_duplicate_sources`, which mirrors upstream cargo's check
that two git sources cannot resolve to the same name+version pair.

### `package.rs`

`package_local_crates` invokes `cargo package --no-verify` per local crate
inside a temp directory whose `.cargo/config.toml` carries a synthesized
`[patch.crates-io]` block pointing every workspace crate at its on-disk
path. This is what lets a crate that depends on a sibling not published
to crates.io still package successfully. The fallback path (when
packaging fails) copies the source tree and runs
`add_versions_to_path_deps` so path deps without explicit versions still
resolve.

### `vendor.rs`

The largest module. Hosts:

- `run_cargo_vendor`: shells out to `cargo vendor` with the right manifest
  and `--sync` arguments.
- `extract_crate_archive`: untars a `.crate` into `vendor/<name>/`.
- `strip_vendor_path_deps`: clears intra-workspace `path = "../sibling"`
  entries that conflict with source replacement.
- `rewrite_local_path_deps`: rewrites local-crate path deps to sibling
  vendor directories.
- `clear_checksums`: writes empty `.cargo-checksum.json` files.
- `generate_cargo_config`: emits the source-replacement TOML.
- `strip_lock_checksums` and `strip_lockfile_inplace`: remove the
  `checksum = "..."` lines that block offline builds against vendored
  sources.
- `freeze_manifest` and `regenerate_lockfile`: the `--freeze` machinery.
- `compress_vendor`: tar + xz with `COPYFILE_DISABLE=1`.

### `strip.rs`

`StripConfig` describes which directories and TOML sections to remove.
`strip_vendor_dir` walks the tree, calling `strip_crate_dir` on each
package directory. Always-safe base directories (`.github`, `.circleci`,
`ci`, `target`) are removed regardless of config. The TOML editor uses
`toml_edit` so comments and formatting are preserved on round-trip.

### `verify.rs`

Two orthogonal checks for CI use:

1. `verify_lock_matches_vendor`: every non-local `[[package]]` in the
   lockfile has a matching `vendor/<name>/` or `vendor/<name>-<version>/`
   whose own `Cargo.toml` declares the same version.
2. `verify_tarball_matches_vendor`: extracting the tarball reproduces the
   file set and byte-for-byte content of `vendor/`.

### `cache.rs`

FNV-64 hashing of all relevant inputs (lockfile bytes plus, recursively,
every `.toml` and `.rs` under each local crate's `src/`, `tests/`,
`examples/`, `benches/`). Three cache files for the three modes; each is
written by the modes that produce a complete artifact for that scope and
checked by the modes that depend on that scope.

### `manifest_guard.rs`

`ManifestGuard::snapshot(path)` reads the file once, captures the bytes,
and restores them in `Drop`. `finish()` dismisses the guard for the rare
case where the mutation should persist. Used to wrap the transient
`[patch.crates-io]` blocks both `vendor.rs` and `package.rs` append before
shelling out to cargo.
