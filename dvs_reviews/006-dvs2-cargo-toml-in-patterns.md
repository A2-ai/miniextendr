# DVS2 Cargo.toml.in Patterns Review

## Overview

DVS2's `Cargo.toml.in` shows how to set up a downstream miniextendr-based package.

## Key Patterns

### 1. Standalone Workspace

```toml
[package]
name = "dvs-rpkg"
version = "0.0.0-9000"
edition = "2021"
publish = false

# Make this a standalone workspace (prevents inheriting from parent)
[workspace]
```

The empty `[workspace]` section prevents this package from inheriting settings from a parent workspace. This is critical for CRAN builds where the package must be self-contained.

### 2. Document Binary

```toml
[[bin]]
name = "document"
path = "document.rs"
bench = false
```

Defines the `document` binary for R wrapper generation. `bench = false` excludes it from benchmarking.

### 3. Library Configuration

```toml
[lib]
path = "lib.rs"
crate-type = ["rlib", "staticlib"]
```

Produces both:
- `rlib`: Rust library format (for Rust-to-Rust deps)
- `staticlib`: C-compatible static library (for R linking)

### 4. Feature Flags

```toml
[features]
default = []
nonapi = ["miniextendr-api/nonapi"]
connections = ["miniextendr-api/connections"]
```

Defines optional features that forward to miniextendr-api features.

### 5. Git Dependencies

```toml
[dependencies]
miniextendr-api = { git = "https://github.com/CGMossa/miniextendr" }
dvs = { git = "https://github.com/A2-ai/dvs2" }
```

References upstream dependencies via git. These get vendored during configure.

### 6. Patch Section

```toml
[patch.crates-io]
miniextendr-api = { git = "https://github.com/CGMossa/miniextendr" }
miniextendr-macros = { git = "https://github.com/CGMossa/miniextendr" }
miniextendr-lint = { git = "https://github.com/CGMossa/miniextendr" }
dvs = { git = "https://github.com/A2-ai/dvs2" }
```

Patches crates-io to use git versions. This is necessary because:
1. miniextendr crates depend on each other via workspace dependencies
2. Workspace dependencies resolve to `version = "*"` when published
3. These patches redirect those `*` dependencies to git sources

**Important:** This is the pattern downstream projects should follow when depending on unreleased miniextendr.

## Comparison with Miniextendr rpkg

Miniextendr's rpkg/src/rust/Cargo.toml.in:
- Is part of the monorepo, so uses path dependencies in dev mode
- Doesn't need `[patch.crates-io]` because it IS the crates being patched
- Has many more features (since it's the demo package)

DVS2's approach is what external packages should follow.

## Recommendations for Documentation

Create a template `Cargo.toml.in` for downstream packages showing:
1. Standalone workspace pattern
2. Git dependency setup
3. Patch section for workspace resolution
4. Feature forwarding pattern
