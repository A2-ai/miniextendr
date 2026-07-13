# cargo_revendor v0.1.0

cargo-revendor: vendor Rust dependencies for R packages and monorepos.

`cargo-revendor` is a thin orchestrator over `cargo metadata`, `cargo
package`, and `cargo vendor`. It calls cargo for the parts cargo does
well, and only does the additional work cargo does not.

Capabilities beyond plain `cargo vendor`:

- Workspace path dependencies are packaged via `cargo package`, which
  resolves `*.workspace = true` inheritance into a standalone manifest.
- Inter-crate `path = "../sibling"` references are rewritten to the
  sibling's vendor directory.
- Opt-in stripping of `tests/`, `benches/`, `examples/`, and `[[bin]]`
  targets, plus the matching `Cargo.toml` sections.
- `--freeze` rewrites the target manifest to resolve everything from
  `vendor/` and regenerates `Cargo.lock` with `--offline`.
- `--compress` tars and xz-compresses `vendor/` for shipping.
- `--verify` is a CI-only check that asserts agreement between
  `Cargo.lock`, `vendor/`, and any compressed tarball.
- Three-tier cache (`.revendor-cache`, `.revendor-cache-external`,
  `.revendor-cache-local`) gates re-vendoring. Source files of local
  crates participate in the cache key because pure source edits leave
  `Cargo.lock` untouched.
- Phase modes (`--external-only`, `--local-only`) split the pipeline
  for CI cases where the external dep set rebuilds rarely.
- `--sync` mirrors `cargo vendor --sync`, unioning multiple disjoint
  workspaces into a single `vendor/` tree.
- JSON output for machine consumption.

See `README.md` in this crate for the full pipeline walkthrough and
flag reference.

---

## Structs

### `Verbosity`

```rust
pub struct Verbosity
```

Verbosity level (0=quiet, 1=-v, 2=-vv, 3=-vvv)

**Fields:**

- `0`: `u8`

**Inherent associated items:**

#### `debug`

```rust
fn debug(self: Self) -> bool
```

#### `info`

```rust
fn info(self: Self) -> bool
```

#### `trace`

```rust
fn trace(self: Self) -> bool
```

### `manifest_guard::ManifestGuard`

```rust
pub struct ManifestGuard
```

Snapshot + auto-restore of a single file.

Construct before any mutation to the file; the snapshotted bytes are
restored unconditionally when the guard is dropped — including on panic
unwind or early `?` return.

Use `finish()` to dismiss the guard when the mutation is intentional and
should persist (rare — typical cargo-revendor usage is always transient).

**Inherent associated items:**

#### `finish`

```rust
fn finish(self: Self)
```

Dismiss the guard so Drop does nothing. Use when the mutation is
intended to persist (cargo-revendor doesn't currently do this, but
the option is here for future callers).

#### `snapshot`

```rust
fn snapshot(path: &Path) -> Result<Self>
```

Capture the file's current bytes. The guard arms immediately — any
subsequent mutation to the file will be reverted on drop.

### `metadata::LocalPackage`

```rust
pub struct LocalPackage
```

A local (path-based) package discovered in the dependency tree

**Fields:**

- `name`: `String`
- `version`: `String`
- `path`: `std::path::PathBuf`
- `manifest_path`: `std::path::PathBuf`

### `strip::StripConfig`

```rust
pub struct StripConfig
```

Configuration for what to strip from vendored crates

**Fields:**

- `tests`: `bool`
- `benches`: `bool`
- `examples`: `bool`
- `bins`: `bool`
- `toml_only`: `bool`
  - Strip TOML sections only — leave source-related directories

**Inherent associated items:**

#### `all`

```rust
fn all() -> Self
```

Strip everything (directories and TOML sections)

#### `any`

```rust
fn any(self: &Self) -> bool
```

Whether any stripping is enabled

#### `toml_only`

```rust
fn toml_only() -> Self
```

Strip TOML sections for all source-target categories without
deleting `tests/` / `benches/` / `examples/` directories. See #330.

### `verify::LockPackage`

```rust
pub struct LockPackage
```

One `[[package]]` entry from a Cargo.lock.

**Fields:**

- `name`: `String`
- `version`: `String`
- `source`: `String`
  - Empty string for local path/workspace packages; `registry+...` or

---

## Functions

### `cache::is_cached`

```rust
fn is_cached(lockfile: &std::path::Path, sync_manifests: &[std::path::PathBuf], vendor_dir: &std::path::Path, local_crate_paths: &[std::path::PathBuf]) -> anyhow::Result<bool>
```

Check whether `vendor_dir` is up to date relative to `lockfile` plus the
source trees of `local_crate_paths`, plus any additional manifests
supplied via `--sync` (#229). Each sync manifest's sibling `Cargo.lock`
and `Cargo.toml` are hashed into the key.

### `cache::is_cached_external`

```rust
fn is_cached_external(lockfile: &std::path::Path, sync_manifests: &[std::path::PathBuf], vendor_dir: &std::path::Path) -> anyhow::Result<bool>
```

Check whether the external deps are up to date relative to `lockfile` and
sync manifests. Ignores local crate source trees — pure source edits don't
change external deps.

### `cache::is_cached_local`

```rust
fn is_cached_local(vendor_dir: &std::path::Path, local_crate_paths: &[std::path::PathBuf]) -> anyhow::Result<bool>
```

Check whether the local crates are up to date relative to their source
trees. Ignores Cargo.lock / sync manifests — lockfile changes don't affect
the local crate packaging output.

### `cache::save_cache`

```rust
fn save_cache(lockfile: &std::path::Path, sync_manifests: &[std::path::PathBuf], vendor_dir: &std::path::Path, local_crate_paths: &[std::path::PathBuf]) -> anyhow::Result<()>
```

Save the current hash to the cache file.

### `cache::save_cache_external`

```rust
fn save_cache_external(lockfile: &std::path::Path, sync_manifests: &[std::path::PathBuf], vendor_dir: &std::path::Path) -> anyhow::Result<()>
```

Save the external-only cache file.

### `cache::save_cache_local`

```rust
fn save_cache_local(vendor_dir: &std::path::Path, local_crate_paths: &[std::path::PathBuf]) -> anyhow::Result<()>
```

Save the local-only cache file.

### `checksum::recompute_cargo_checksum_json`

```rust
fn recompute_cargo_checksum_json(crate_dir: &std::path::Path) -> anyhow::Result<()>
```

Recompute `.cargo-checksum.json` for a single vendored crate directory.

Preserves the original `package` field (the registry `.crate` SHA-256 that
matches the committed `Cargo.lock`'s `checksum =` line) and rewrites the
`files` map with SHA-256s of every regular file currently present in
`crate_dir/`, excluding `.cargo-checksum.json` itself.

POSIX-relative paths (forward slashes) are used in the `files` map, as
required by cargo's directory source format.

### `checksum::recompute_checksums`

```rust
fn recompute_checksums(vendor_dir: &std::path::Path) -> anyhow::Result<()>
```

Recompute `.cargo-checksum.json` for every crate directory in `vendor_dir`.

This replaces `clear_checksums`: instead
of writing `{"files":{}}` (empty files map, null package), we preserve the
original `package` hash (matching the committed `Cargo.lock`) and recompute
the `files` map from the trimmed disk contents.

Called after CRAN-trim so that cargo's offline source-replacement can verify
both the lockfile consistency (via the `package` field) and the file
integrity (via the `files` map).

### `find_workspace_root`

```rust
fn find_workspace_root(dir: &std::path::Path) -> anyhow::Result<std::path::PathBuf>
```

Find workspace root by walking up from a directory

### `metadata::check_duplicate_sources`

```rust
fn check_duplicate_sources(meta: &cargo_metadata::Metadata) -> anyhow::Result<()>
```

Error out when two different sources resolve to the same (name, version).

Mirrors upstream cargo/src/cargo/ops/vendor.rs's duplicate-source check:
two git repos that happen to publish the same crate name + version would
otherwise silently last-write-wins during extraction, making the vendored
contents depend on dep-graph iteration order. Upstream hard-errors; we do
too.

Common legitimate case this does NOT flag: the SAME (name, version) from
the SAME source appearing multiple times in `meta.packages` (cargo can
emit dupes when a package is reached via different dep paths). Only
DIFFERENT sources for the same (name, version) key are errors.

### `metadata::discover_from_patch_config`

```rust
fn discover_from_patch_config(manifest_path: &std::path::Path) -> anyhow::Result<Vec<LocalPackage>>
```

Discover local package overrides from `[patch."<url>"]` tables in
`.cargo/config.toml`.

Cargo's config search order walks up from the manifest directory, checking
`<dir>/.cargo/config.toml` at each level, then falls back to
`$HOME/.cargo/config.toml`. This function mirrors that walk.

For each `[patch."<url>"]` table (the URL may have or lack the `git+`
scheme prefix — both forms are accepted), entries of the form
`<crate-name> = { path = "<path>" }` are collected. The `path` is resolved
relative to the config file that declares it. For each entry, the target
crate's `Cargo.toml` is read to extract the version, and a `LocalPackage`
is returned.

Entries where the `path` does not contain a readable `Cargo.toml` are
silently skipped (the dep may not exist yet on this machine).

On TOML parse errors in a config file, returns an error with the file path
and position so the caller can report it loudly.

### `metadata::discover_patch_url_map`

```rust
fn discover_patch_url_map(manifest_path: &std::path::Path) -> anyhow::Result<std::collections::BTreeMap<String, String>>
```

Map each `[patch."<url>"]` crate to the git URL it is patched from.

This is the provenance the lockfile needs but `discover_from_patch_config`
discards: when cargo resolves with a `[patch."<url>"]` path override active,
the framework crates land in `Cargo.lock` as local (no `source`) entries.
For the offline tarball, those entries must instead carry
`source = "git+<url>#<sha>"` so cargo's `[source."git+<url>"]` replacement
can redirect them to `vendored-sources`. This function recovers the
`crate-name -> <url>` mapping from the same `.cargo/config.toml` walk so the
lockfile can be stamped after resolution. See [`crate::vendor::stamp_framework_git_sources`].

The returned URL is normalized: any leading `git+` scheme prefix is stripped
(cargo accepts `[patch."https://…"]` and `[patch."git+https://…"]`
interchangeably; the lockfile/source-replacement form is `git+<url>`, which
the stamper re-adds). Only entries whose `path` resolves to a readable
`Cargo.toml` are included — an unresolvable patch path means the crate is
vendored from its real git source, which is already lockfile-correct and
must not be stamped.

### `metadata::discover_workspace_members`

```rust
fn discover_workspace_members(workspace_root: &std::path::Path) -> anyhow::Result<Vec<LocalPackage>>
```

Discover all workspace members from a workspace root Cargo.toml

### `metadata::load_metadata`

```rust
fn load_metadata(manifest_path: &std::path::Path) -> anyhow::Result<cargo_metadata::Metadata>
```

Load cargo metadata for the given manifest.

Runs `cargo metadata` with the working directory set to the manifest's
parent so that cargo's CWD-relative config discovery picks up that crate's
`.cargo/config.toml`. For an R package in dev/source mode this carries the
`[patch."<git-url>"]` table that redirects the framework crates to the local
workspace checkout — so a cross-crate feature/dep rename (touching both a
framework crate and its consumer) resolves against the PR's sources instead
of git@main (#883). Without the CWD pin cargo would search upward from the
process CWD (the repo root for `just vendor`) and never find the patch.

### `metadata::partition_packages`

```rust
fn partition_packages(meta: &cargo_metadata::Metadata, target_manifest: &std::path::Path, git_overrides: &[LocalPackage]) -> anyhow::Result<(Vec<LocalPackage>, Vec<String>)>
```

Partition packages into local (path deps) and external (registry/git)

Local packages are those whose source is a local path and whose
manifest is NOT inside the target package's src/rust directory
(i.e., they're workspace siblings, not the package itself).

`git_overrides` allows callers to reclassify git-sourced deps as local
when the same crate is available in a local source root (e.g., a monorepo
where `--source-root` points at the workspace containing the git dep).
Any git dep whose name matches an entry in `git_overrides` is treated as
local and vendored from the local path rather than fetched from git.
Pass `&[]` when `--source-root` is not in use.

Returns an error if a git dep matches a `git_overrides` entry by name but
the resolved git version differs from the local version — a version mismatch
means the local checkout is not the same code the lockfile pinned, and
silently vendoring the wrong source would produce broken builds.

### `package::package_local_crates`

```rust
fn package_local_crates(local_pkgs: &[crate::metadata::LocalPackage], all_patch_pkgs: &[crate::metadata::LocalPackage], _target_manifest: &std::path::Path, staging_dir: &std::path::Path, allow_dirty: bool, v: crate::Verbosity) -> anyhow::Result<Vec<(String, std::path::PathBuf)>>
```

Package each local crate, returning (name, crate_archive_path) pairs

`local_pkgs` — crates to actually package
`all_patch_pkgs` — ALL workspace crates (for [patch.crates-io] config)

### `path_to_toml`

```rust
fn path_to_toml(path: &std::path::Path) -> String
```

Convert a path to a TOML-safe string (forward slashes, no \\?\ prefix)

### `strip::prune_dangling_feature_refs`

```rust
fn prune_dangling_feature_refs(content: &str, removed_deps: &[String]) -> String
```

Remove feature array entries that reference removed dependencies.

Cargo validates ALL `[features]` entries at parse time regardless of which
features are enabled. After dev-dependencies are stripped, any feature
referencing `"<dep>/..."`, `"<dep>?/..."`, or exactly `"<dep>"` for a
removed dep becomes a dangling reference that breaks every consumer.

Exact-match items (`"<dep>"`) are only pruned when no `[features]` entry
of the same name exists in this crate. Otherwise the string refers to the
crate's own feature, not the dep — e.g. toml ships
`default = ["std", "serde", "parse", "display"]` where `"serde"` is the
feature key, not the dev-dep.

If a feature's array becomes empty after pruning, it is kept as `[]`
(a valid, harmless feature flag). Non-referencing features are unchanged.

### `strip::strip_vendor_dir`

```rust
fn strip_vendor_dir(vendor_dir: &std::path::Path, config: &StripConfig, v: crate::Verbosity) -> anyhow::Result<Vec<String>>
```

Strip all vendored crates in a vendor directory.
Returns list of stripped items for reporting.

### `vendor::compress_vendor`

```rust
fn compress_vendor(vendor_dir: &std::path::Path, tarball_path: &std::path::Path, blank_md: bool, v: crate::Verbosity) -> anyhow::Result<()>
```

Compress vendor/ into a .tar.xz tarball

### `vendor::copy_lock_to_vendor`

```rust
fn copy_lock_to_vendor(lockfile: &std::path::Path, vendor_dir: &std::path::Path, v: crate::Verbosity) -> anyhow::Result<()>
```

Copy `Cargo.lock` to the vendor directory for use by `--freeze` /
`regenerate_lockfile`.

Checksums are retained — cargo-revendor now writes valid `.cargo-checksum.json`
files (with `package` fields matching the lockfile's `checksum = "..."` lines),
so the lock no longer needs to be stripped before copying.

### `vendor::extract_crate_archive`

```rust
fn extract_crate_archive(crate_path: &std::path::Path, vendor_dir: &std::path::Path, pkg_name: &str, pkg_version: Option<&str>, v: crate::Verbosity) -> anyhow::Result<()>
```

Extract a .crate archive OR copy a directory into the vendor directory.

Local workspace crates always land at flat `vendor/<name>/` — they are
single-version by construction, so the #214 flat-slot non-determinism
(which motivates `--versioned-dirs` for transitive deps) can't apply.
`pkg_version` is kept only to clean up versioned placeholders that
`cargo vendor --versioned-dirs` may have created for patched crates.

### `vendor::freeze_manifest`

```rust
fn freeze_manifest(manifest_path: &std::path::Path, vendor_dir: &std::path::Path, local_pkgs: &[crate::metadata::LocalPackage], versioned_dirs: bool, strict: bool, v: crate::Verbosity) -> anyhow::Result<()>
```

Freeze: rewrite Cargo.toml so sources resolve from vendor/.

1. Rewrites manifest-declared `path =` deps to vendor/ path deps. Deps
   declared `git =` are left untouched (external by declaration, even if a
   `[patch]` resolves them to a local crate during vendoring) and resolve
   offline via source replacement.
2. Strips all `[patch.*]` sections (they reference sources outside vendor/)
3. Adds `[patch.crates-io]` with vendor paths for the frozen path deps

After freezing, the manifest resolves from vendor/ for its path deps;
remaining git deps resolve via vendor/.cargo-config.toml source replacement.
`cargo build --offline` then works with only the vendor directory.

### `vendor::generate_cargo_config`

```rust
fn generate_cargo_config(manifest_path: &std::path::Path, vendor_dir: &std::path::Path, _local_pkgs: &[crate::metadata::LocalPackage]) -> anyhow::Result<String>
```

Generate a .cargo/config.toml for source replacement.

Returns the config content as a string. Also writes it to
`<vendor_dir>/../src/rust/.cargo/config.toml` if that path exists.

### `vendor::regenerate_lockfile`

```rust
fn regenerate_lockfile(manifest_path: &std::path::Path, vendor_dir: &std::path::Path, v: crate::Verbosity) -> anyhow::Result<()>
```

Regenerate Cargo.lock from vendored sources (freeze-consistent copy).

The vendor/ directory contains a Cargo.lock (with registry checksums
retained) produced by `copy_lock_to_vendor` during the same vendoring run.
Copying it directly to the manifest's Cargo.lock is the most reliable
approach: it is exactly consistent with what was vendored, avoiding
version-drift that can occur when `cargo generate-lockfile --offline`
resolves from the local index cache (which may have been updated by a
subsequent `cargo vendor` run).

### `vendor::resolve_framework_rev`

```rust
fn resolve_framework_rev(candidate_paths: &[std::path::PathBuf], v: crate::Verbosity) -> String
```

Resolve the commit sha to stamp as framework-crate provenance in Cargo.lock.

Tries `git rev-parse HEAD` in each candidate checkout (the local framework
crate dirs) in turn; falls back to [`PLACEHOLDER_GIT_REV`] with a warning.
The value is provenance only — cargo's `[source."git+<url>"]` replacement
keys on the URL, never the commit — so a placeholder still builds offline.

### `vendor::resolve_workspace_inheritance`

```rust
fn resolve_workspace_inheritance(vendor_crate_dir: &std::path::Path, original_crate_dir: &std::path::Path, v: crate::Verbosity) -> anyhow::Result<()>
```

Resolve `*.workspace = true` fields in a directly-copied crate's Cargo.toml

When cargo package can't run (unpublished deps), we copy the crate directly.
But workspace inheritance (`version.workspace = true`, etc.) won't resolve
outside the workspace. This function reads the workspace root's
`[workspace.package]` and replaces the inherited fields.

### `vendor::rewrite_local_path_deps`

```rust
fn rewrite_local_path_deps(vendor_dir: &std::path::Path, local_pkgs: &[crate::metadata::LocalPackage], v: crate::Verbosity) -> anyhow::Result<()>
```

Rewrite inter-crate path dependencies so local crates reference each other
in `vendor/`. Local crates always land at flat `vendor/<name>/` — they are
single-version by construction, so the #214 rationale for versioned dirs
doesn't apply.

### `vendor::run_cargo_vendor`

```rust
fn run_cargo_vendor(manifest_path: &std::path::Path, vendor_dir: &std::path::Path, local_pkgs: &[crate::metadata::LocalPackage], sync_manifests: &[std::path::PathBuf], versioned_dirs: bool, v: crate::Verbosity) -> anyhow::Result<()>
```

Run `cargo vendor` for external (registry/git) dependencies.

`sync_manifests` mirrors `cargo vendor --sync <path>`: additional
manifests whose dep graphs are unioned into the same output tree.
Use case (#229): one R-package workspace plus a disjoint benchmarks
workspace sharing one offline artifact; two packages pinning different
versions of the same transitive dep both coexist in `vendor/` as
separate dirs.

### `vendor::stamp_framework_git_sources`

```rust
fn stamp_framework_git_sources(lockfile: &std::path::Path, patch_url_map: &std::collections::BTreeMap<String, String>, rev: &str, v: crate::Verbosity) -> anyhow::Result<usize>
```

Stamp `source = "git+<url>#<rev>"` onto the framework crates' `[[package]]`
entries in `lockfile`.

Why this exists: the lock is resolved with the dev `[patch."<url>"]` path
override active (so a cross-crate feature rename resolves against the LOCAL
workspace, not git@main — see #883). That resolution records the framework
crates as local (no `source`) entries. The offline tarball install, however,
needs `source = "git+<url>#<sha>"` so cargo's `[source."git+<url>"]`
replacement can redirect them to `vendored-sources`. We reconstruct that
attribution here rather than re-resolving against the bare git URL (which is
exactly the step that fails on a cross-crate rename).

`patch_url_map` is `crate-name -> <url>` (no `git+` prefix; see
[`crate::metadata::discover_patch_url_map`]). Only packages named in the map
are touched; for those, any existing `source`/`path` is replaced and the new
`source` line is placed immediately after `version` to match cargo's own
canonical key order (and the `grep -A3` lock-shape check). Returns the number
of `[[package]]` entries stamped.

### `vendor::strip_vendor_path_deps`

```rust
fn strip_vendor_path_deps(vendor_dir: &std::path::Path, v: crate::Verbosity) -> anyhow::Result<()>
```

Strip relative path dependencies (`path = "../..."`) from all vendored crate manifests.

When `cargo vendor` vendors crates from a git workspace, the vendored Cargo.toml
files retain intra-workspace path deps (e.g., `path = "../sibling-crate"`). During
offline builds with cargo source replacement, these path deps cause cargo to resolve
siblings as path sources instead of through the directory source, which conflicts
with Cargo.lock entries that record them as git (or registry) sources. Stripping the
path keys forces cargo to resolve by name from the replaced source.

This runs BEFORE `rewrite_local_path_deps`, which adds back correct path deps
for local/workspace crates only.

### `verify::parse_lockfile`

```rust
fn parse_lockfile(lockfile: &std::path::Path) -> anyhow::Result<Vec<LockPackage>>
```

Parse a Cargo.lock into its `[[package]]` entries.

### `verify::verify_lock_matches_vendor`

```rust
fn verify_lock_matches_vendor(lockfile: &std::path::Path, vendor_dir: &std::path::Path) -> anyhow::Result<()>
```

Assert that every foreign-source package in `lockfile` has a corresponding
directory in `vendor_dir` whose `Cargo.toml` reports the same version.

Local path packages (empty `source`) are skipped — they are the crates
being vendored *from*, not *into*.

### `verify::verify_tarball_matches_vendor`

```rust
fn verify_tarball_matches_vendor(tarball: &std::path::Path, vendor_dir: &std::path::Path) -> anyhow::Result<()>
```

Assert that extracting `tarball` yields the same files with the same
contents as `vendor_dir`.

Uses byte-level hashing so drift in a single vendored `.rs` is caught.

---

## Constants

### `cache::CACHE_FILE`

```rust
pub const CACHE_FILE: &str = ".revendor-cache";
```

### `cache::CACHE_FILE_EXTERNAL`

```rust
pub const CACHE_FILE_EXTERNAL: &str = ".revendor-cache-external";
```

### `cache::CACHE_FILE_LOCAL`

```rust
pub const CACHE_FILE_LOCAL: &str = ".revendor-cache-local";
```
