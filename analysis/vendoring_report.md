# Vendoring Behavior Report

Last updated: 2026-02-18

## Executive Summary

There are **three separate vendoring systems** in miniextendr, each serving different purposes:

1. **`just vendor`** (rpkg only) — CRAN release prep. Runs `cargo vendor` for all deps.
2. **`vendor_miniextendr()`** (scaffolded packages) — Downloads miniextendr crates from GitHub into `vendor/`. Does NOT run `cargo vendor` for external deps.
3. **`use_vendor_lib()`** (monorepo packages) — Configures a monorepo crate as a vendored dependency via `cargo package` + tarball extraction.

The key question: **when does `cargo vendor` actually run?**

---

## System 1: rpkg (the demo package in the monorepo)

rpkg uses **git dependencies** in Cargo.toml with a `[patch]` section for local development:

```toml
[dependencies]
miniextendr-api = { git = "https://github.com/CGMossa/miniextendr" }

[build-dependencies]
miniextendr-lint = { git = "https://github.com/CGMossa/miniextendr" }

[patch."https://github.com/CGMossa/miniextendr"]
miniextendr-api = { path = "../../../miniextendr-api" }
miniextendr-macros = { path = "../../../miniextendr-macros" }
miniextendr-macros-core = { path = "../../../miniextendr-macros-core" }
miniextendr-lint = { path = "../../../miniextendr-lint" }
```

### Build Context Resolution (rpkg/configure.ac)

rpkg's configure resolves one of 4 build contexts:

| Context | Trigger | `cargo vendor` runs? | Cargo config | vendor/ state |
|---------|---------|---------------------|--------------|---------------|
| `dev-monorepo` | monorepo detected (auto or `NOT_CRAN=true`) | **NO** | Deleted | Cleaned if exists |
| `dev-detached` | no monorepo + `NOT_CRAN=true` | **NO** | Deleted | Cleaned if exists |
| `vendored-install` | `NOT_CRAN=false` explicit, or auto+vendor hint | **ERROR** (if no vendor exists) | Kept | Unpacked from tarball or error |
| `prepare-cran` | `PREPARE_CRAN=true` | **NO** (assumes `just vendor` was already run) | Kept | Must pre-exist |

#### Decision tree:

```
PREPARE_CRAN=true?
  └─ YES → prepare-cran (assumes vendor exists from `just vendor`)
  └─ NO
     NOT_CRAN explicitly set?
       └─ YES + NOT_CRAN=true
          └─ monorepo? → dev-monorepo (uses [patch] paths)
          └─ no monorepo? → dev-detached (uses git/network deps)
       └─ YES + NOT_CRAN=false → vendored-install
       └─ NOT SET (auto-detect)
          └─ monorepo? → dev-monorepo
          └─ vendor hint? → vendored-install
          └─ neither? → dev-detached
```

### Scenario test results (rpkg in monorepo)

| Command | BUILD_CONTEXT | `cargo vendor` runs? | Notes |
|---------|---------------|---------------------|-------|
| `bash ./configure` | dev-monorepo | NO | Auto-detected monorepo |
| `NOT_CRAN=true bash ./configure` | dev-monorepo | NO | Same as above |
| `NOT_CRAN=false bash ./configure` | vendored-install | **ERROR** | No tarball found → error: "Run 'just vendor' first" |
| `PREPARE_CRAN=true bash ./configure` | prepare-cran | NO | Requires pre-existing vendor/ |

### When `cargo vendor` actually runs in rpkg configure:

**Only in `vendored-install` context when no vendor/ and no inst/vendor.tar.xz exist.**

This happens when:
- Installing from GitHub via `remotes::install_github()` where `NOT_CRAN` is unset
- The package has no pre-built vendor tarball
- Configure falls through to the network vendoring path (lines 529-554 of configure.ac)

### `just vendor` (explicit CRAN prep)

`just vendor` is the only supported way to create `inst/vendor.tar.xz` for CRAN:

1. `cargo package` each workspace crate → `.crate` archives
2. `cargo vendor` all external crates.io deps → `rpkg/vendor/`
3. Extract workspace `.crate` files on top of vendor/
4. Strip tests/benches/docs
5. Compress to `rpkg/inst/vendor.tar.xz`

This is **never called automatically**. It's a manual step for CRAN submission.

---

## System 2: Scaffolded packages (via minirextendr)

### Standalone R package (`create_miniextendr_package` / `use_miniextendr`)

1. **Template Cargo.toml uses path deps** (not git deps):
   ```toml
   [dependencies]
   miniextendr-api = { path = "../../vendor/miniextendr-api" }
   [build-dependencies]
   miniextendr-lint = { path = "../../vendor/miniextendr-lint" }
   ```

2. **`vendor_miniextendr()` is called during scaffolding** — downloads miniextendr crates from GitHub (or copies from local path) into `vendor/`:
   - miniextendr-api
   - miniextendr-macros
   - miniextendr-macros-core
   - miniextendr-lint
   - miniextendr-engine

3. **External crates.io deps are NOT vendored** at scaffold time.

4. **`vendor_crates_io()` exists** as a separate function but is never called automatically.

### Template configure.ac behavior

The template uses a simple `NOT_CRAN` binary:

| NOT_CRAN | FORCE_VENDOR | vendor/ empty? | Behavior |
|----------|-------------|----------------|----------|
| true | unset | any | "dev mode — using vendor/ from scaffolding" (no vendoring) |
| true | 1 | any | **Runs `cargo vendor`** (explicit override) |
| false | any | no | "CRAN build — vendor ready" (uses existing vendor/) |
| false | any | yes + tarball exists | Unpacks inst/vendor.tar.xz |
| false | any | yes + no tarball | **ERROR**: "Run 'just vendor' before CRAN submission" |

### Key observation for scaffolded packages:

At scaffold time, `vendor/` contains only miniextendr crates (5 crates). It does NOT contain external deps (proc-macro2, syn, quote, etc.).

**In dev mode (`NOT_CRAN=true`):**
- Cargo config is deleted
- Cargo.toml uses `path = "../../vendor/miniextendr-api"` directly
- External deps (proc-macro2, syn, etc.) are fetched from crates.io via network
- This works because `--offline` is not set in dev mode

**In CRAN mode (`NOT_CRAN=false` or unset):**
- Cargo config is KEPT (source replacement: all sources → `vendor/`)
- `--offline` flag is set
- If vendor/ only has miniextendr crates but not external deps → **build fails**
- User must run `vendor_crates_io()` or equivalent before CRAN submission

### Monorepo template (`create_miniextendr_monorepo`)

Identical configure.ac to standalone template. Same behavior.

Additionally:
- `vendor_miniextendr()` is called with `dest = rpkg_name/vendor/`
- The main Rust crate (e.g., `my-crate/`) is NOT vendored — it's referenced via `[patch.crates-io]` (dev) or VENDOR_LIB (CRAN)

---

## System 3: VENDOR_LIB (monorepo library dependency)

For when an R package depends on a Rust crate from the same monorepo (not miniextendr itself, but the user's own crate).

**Setup via `use_vendor_lib(crate, version, dev_path)`:**

1. Adds `crate = "version"` to `[dependencies]` in Cargo.toml
2. Adds `crate = { path = "dev_path" }` to `[patch.crates-io]` in Cargo.toml
3. Modifies configure.ac to add a `vendor-lib-{crate}` AC_CONFIG_COMMANDS block:

| Mode | What happens |
|------|-------------|
| Dev (`NOT_CRAN=true`) | `cargo package` the monorepo crate → `inst/{crate}-lib.tar.gz` (for future CRAN use) |
| CRAN (`NOT_CRAN=false`) | Extract `inst/{crate}-lib.tar.gz` → `vendor/{crate}`, rewrite `[patch.crates-io]` path |

**Key:** VENDOR_LIB creates a tarball from `cargo package`, not `cargo vendor`. It only handles the single monorepo crate, not its transitive dependencies.

---

## Issues Found

### Issue 1: `NOT_CRAN=false` in rpkg triggers surprise network vendoring — FIXED

**Problem:** Running `NOT_CRAN=false bash ./configure` in the monorepo (no vendor/, no tarball) triggered `cargo vendor` from network.

**Fix:** Replaced the network vendoring fallback in `rpkg/configure.ac` with a clear error message directing users to run `just vendor` first. The `vendored-install` context now requires pre-existing vendored sources (either `vendor/` directory or `inst/vendor.tar.xz`).

### Issue 2: Scaffolded packages have incomplete vendor/ for CRAN — FIXED

**Problem:** After `create_miniextendr_package()`, `vendor/` contains only miniextendr crates (5 crates). External deps (proc-macro2, syn, quote, etc.) are missing.

**Fix:** Added `just vendor` recipe to both template justfiles (rpkg and monorepo). This calls `minirextendr::miniextendr_vendor()` which runs `cargo vendor` for all deps and compresses to `inst/vendor.tar.xz`. Updated `cran-prep` to depend on `vendor`. Updated the comment about vendor/ state.

### Issue 3: Template cargo-config.toml.in has git source replacement — FIXED

**Problem:** All three cargo-config.toml.in files included a `[source."git+https://..."]` block for git source replacement that was dead code (templates use path deps, and rpkg's configure.ac rewrites git deps to path deps before this config is used).

**Fix:** Removed the git source replacement block from all three cargo-config.toml.in files. Also removed the now-unnecessary sed command in rpkg/configure.ac that stripped this block during vendored builds.

### Issue 4: No PREPARE_CRAN in templates — NOT AN ISSUE

**Observation:** The template configure.ac does not have `PREPARE_CRAN`. This is fine for scaffolded packages because:
- `NOT_CRAN=true` (dev): uses vendor/ from scaffolding, no vendoring
- `NOT_CRAN=false` (CRAN): requires pre-populated vendor/, no vendoring
- `FORCE_VENDOR=1` + `NOT_CRAN=true`: explicit escape hatch for re-vendoring

No change needed.

---

## Summary Matrix: When Does `cargo vendor` Run?

### rpkg (monorepo demo package)

| Scenario | `cargo vendor` | Why |
|----------|---------------|-----|
| `just configure` (monorepo) | NO | dev-monorepo: uses [patch] paths |
| `just configure-cran` (PREPARE_CRAN=true) | NO | prepare-cran: assumes `just vendor` was already run |
| `just vendor` | **YES** | Explicit CRAN prep command |
| `NOT_CRAN=false` (no vendor) | **ERROR** | vendored-install: requires pre-existing vendor |
| `NOT_CRAN=false` (with vendor) | NO | vendored-install: uses existing vendor/ |
| `NOT_CRAN=false` (with tarball) | NO | vendored-install: unpacks tarball |
| GitHub install (no tarball) | **ERROR** | vendored-install: requires pre-existing vendor |

### Scaffolded standalone package

| Scenario | `cargo vendor` | Why |
|----------|---------------|-----|
| After scaffold | NO | vendor/ has only miniextendr crates from GitHub download |
| `NOT_CRAN=true` configure | NO | Dev mode: "using vendor/ from scaffolding" |
| `NOT_CRAN=false` configure (with full vendor/) | NO | CRAN mode: uses existing vendor/ |
| `NOT_CRAN=false` configure (scaffolding vendor/ only) | NO | CRAN mode: **build will fail** (missing external deps) |
| `FORCE_VENDOR=1 NOT_CRAN=true` configure | **YES** | Explicit override |
| `vendor_crates_io()` from R | **YES** | Manual CRAN prep step |

### Scaffolded monorepo package

Same as standalone, plus:

| Scenario | `cargo vendor` | Why |
|----------|---------------|-----|
| `use_vendor_lib()` dev mode | NO | Uses `cargo package` for the lib crate only |
| `use_vendor_lib()` CRAN mode | NO | Extracts tarball from inst/ |

---

## Does Current Behavior Match Requirements?

| Requirement | Status | Notes |
|-------------|--------|-------|
| "Vendoring should NOT occur when created from minirextendr functions" | **PASS** | Scaffolding copies crates, doesn't run `cargo vendor` |
| "Should only happen when PREPARE_CRAN is set" | **PASS** | rpkg: `just vendor` is the only way to create vendor tarball. `NOT_CRAN=false` without vendor/ now errors instead of falling back to network. Templates: no PREPARE_CRAN exists, uses FORCE_VENDOR instead. |
| "If there IS a vendor, and NOT_CRAN is unset, should use vendor stuff" | **PASS** (rpkg) | Auto-detect: vendor hint → vendored-install context |
| "If there IS a vendor, and NOT_CRAN is unset, should use vendor stuff" | **PASS** (template) | NOT_CRAN defaults to false → CRAN mode → uses vendor/ |
| "Monorepo setup requires at least monorepo-rooted crates vendored (VENDOR_LIB)" | **PASS** | `use_vendor_lib()` handles this via cargo package + tarball |

**All requirements are now satisfied.** The network vendoring fallback has been removed — `vendored-install` context now requires pre-existing vendored sources or errors cleanly.
