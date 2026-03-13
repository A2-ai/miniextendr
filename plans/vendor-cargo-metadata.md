# Plan: Switch vendor-crates.R from cargo tree to cargo metadata

## Problem

`vendor-crates.R` discovers local path-dependencies by parsing `cargo tree
--format {p}` text output. This format is not guaranteed stable across Cargo
versions — if the output layout changes, `parse_tree_packages()` breaks
silently.

`cargo metadata --format-version=1` provides the same information as stable,
versioned JSON. It's the recommended machine-readable interface.

## Constraint

`vendor-crates.R` is copied into scaffolded packages at `tools/vendor-crates.R`
and must remain **zero R-package-dependency** (no jsonlite, no other CRAN
packages). Base R has no JSON parser.

## Approach: Use jq as a system tool

`jq` is widely available on developer machines and CI environments. The R
script can shell out to `jq` to extract the needed fields from
`cargo metadata` JSON output.

### What we need from the dependency graph

For each local (path-based) crate reachable from the package manifest:
- `name`
- `version`
- `manifest_path` (absolute path to its Cargo.toml)

And for each crate being packaged, its local dependencies (to generate
per-crate `[patch.crates-io]` configs).

### Implementation sketch

```r
cargo_metadata_packages <- function(manifest_path) {
  # Run cargo metadata
  result <- run_command("cargo", c(
    "metadata", "--format-version=1",
    "--manifest-path", manifest_path,
    "--no-deps"
  ))
  json <- paste(result$output, collapse = "\n")

  # Use jq to extract package info
  jq_filter <- '.packages[] | {name, version, manifest_path, source}'
  jq_result <- run_command("jq", c("-c", jq_filter), input = json)

  # Parse jq's line-delimited JSON output (one object per line)
  packages <- lapply(jq_result$output, function(line) {
    # Minimal JSON field extraction with regex (jq already flattened it)
    name <- sub('.*"name":"([^"]*)".*', '\\1', line)
    version <- sub('.*"version":"([^"]*)".*', '\\1', line)
    manifest <- sub('.*"manifest_path":"([^"]*)".*', '\\1', line)
    source <- sub('.*"source":("([^"]*)"|null).*', '\\2', line)
    list(name = name, version = version,
         manifest_path = manifest, source = source)
  })

  # Filter to local packages (source == null means path dep)
  local <- Filter(function(p) !nzchar(p$source), packages)

  data.frame(
    name = vapply(local, `[[`, "", "name"),
    version = vapply(local, `[[`, "", "version"),
    manifest_path = vapply(local, `[[`, "", "manifest_path"),
    crate_dir = dirname(vapply(local, `[[`, "", "manifest_path")),
    stringsAsFactors = FALSE
  )
}
```

### jq filters needed

```bash
# All local (non-registry) packages:
cargo metadata --format-version=1 --manifest-path Cargo.toml \
  | jq -c '.packages[] | select(.source == null) | {name, version, manifest_path}'

# Full dependency graph (with edges):
cargo metadata --format-version=1 --manifest-path Cargo.toml \
  | jq -c '.resolve.nodes[] | {id, deps: [.deps[].pkg]}'
```

### Fallback strategy

If `jq` is not found, fall back to the current `cargo tree` parsing. This
keeps the script working on minimal systems while preferring the stable API
when available:

```r
discover_packages <- function(manifest_path, ...) {
  jq <- Sys.which("jq")
  if (nzchar(jq)) {
    cargo_metadata_packages(manifest_path, ...)
  } else {
    # existing cargo tree approach
    cargo_tree_packages(manifest_path, ...)
  }
}
```

## Alternatives considered

| Approach | Pros | Cons |
|----------|------|------|
| jsonlite R package | Proper JSON parsing | Adds R dependency; breaks zero-dep constraint |
| Base R regex on raw JSON | No external deps | Fragile on nested/escaped JSON; worse than cargo tree |
| Python json module | Robust parsing | Adds Python dependency |
| **jq (chosen)** | Stable, widely available, compact | Requires jq on PATH; needs fallback |
| cargo tree (current) | Zero deps, works now | Unstable output format |

## Files to change

1. `rpkg/tools/vendor-crates.R` — add `cargo_metadata_packages()`, update
   `discover_local_roots()` to prefer metadata when jq is available
2. `minirextendr/inst/templates/rpkg/tools/vendor-crates.R` — same
3. `minirextendr/inst/templates/monorepo/tools/vendor-crates.R` — same

All three files are identical copies. Consider whether a sync-check mechanism
(like templates-check) should cover these too.

## Scope

Small — the core change is one new function + a dispatch wrapper. The fallback
ensures no breakage if jq is missing.
