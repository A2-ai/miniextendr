# Vendor only inside R CMD build → R CMD INSTALL <tarball>

## Goal

Vendoring (`rpkg/vendor/` + `rpkg/inst/vendor.tar.xz`) becomes a CRAN-tarball-only artifact.

- Source installs (`R CMD INSTALL .`, `devtools::install()`, `devtools::load_all()`,
  `remotes::install_github()`, `rv sync` on a git checkout) **never** touch vendor.
- The `NOT_CRAN` env var is removed. There are exactly two install paths:
  1. **Source mode** (no `Packaged:` in DESCRIPTION) — cargo resolves deps normally
     (workspace path overrides in monorepo, git URLs otherwise).
  2. **Tarball mode** (`Packaged:` present) — `inst/vendor.tar.xz` is unpacked,
     `.cargo/config.toml` redirects `[source.crates-io]` to `vendor/`, build runs
     `--offline`.

## Why

- 8–12 minute revendor on every `just configure` + sccache cache poisoning from
  per-invocation Cargo.lock hashes makes dev iteration painful.
- "First-install from github" path forces network + cargo-revendor + git clone of
  miniextendr just to bootstrap `path = "../../vendor/..."` deps.
- The `--freeze` mode of `just vendor` writes `path = "../../vendor/..."` into
  `rpkg/src/rust/Cargo.toml`, which then *requires* vendor/ to exist in every
  context — including dev. This is the load-bearing hack we are pulling out.
- `NOT_CRAN` is overloaded (CRAN-vs-not, vendor-or-not, network-or-not, lock-or-not)
  and several CI knobs disagree about its intended meaning.

## Tarball detection

`grep -q '^Packaged:' DESCRIPTION` in `configure.ac`.

R CMD build calls `add_build_stamp_to_description_file()` (build.R:174–189) which
unconditionally writes `Packaged: <date>; <user>` to DESCRIPTION right before
tarballing. A raw source tree never has this field. The `inst/vendor.tar.xz`
existence test serves as a redundant fallback.

`Built:` is *different* — added by `R CMD INSTALL --build` for binary packages.
Don't conflate. `R_BUILD_TAR` is read but not set by build itself. Path-pattern
heuristics (`Rbuild`/`Rinst` tempdirs) are fragile.

## Cargo.toml dep shape

Committed `rpkg/src/rust/Cargo.toml` uses **git URLs** for the three workspace
crates (matching `minirextendr/inst/templates/rpkg/Cargo.toml.tmpl`):

```toml
[dependencies]
miniextendr-api = { git = "https://github.com/A2-ai/miniextendr" }

[build-dependencies]
miniextendr-lint = { git = "https://github.com/A2-ai/miniextendr" }

[patch.crates-io]
miniextendr-api    = { git = "https://github.com/A2-ai/miniextendr" }
miniextendr-macros = { git = "https://github.com/A2-ai/miniextendr" }
miniextendr-lint   = { git = "https://github.com/A2-ai/miniextendr" }
```

Resolution per mode:

- **Tarball**: `vendor.tar.xz` unpacked, `.cargo/config.toml` writes
  `[source."git+https://github.com/A2-ai/miniextendr"] replace-with = "vendored-sources"`
  (the existing `dev-cargo-config` block already handles this scan). Build is offline.
- **Monorepo dev**: `configure.ac` walks up from `rpkg/`, finds
  `miniextendr-api/Cargo.toml`, substitutes a `MONOREPO_PATCH_CONFIG` variable into
  `Makevars`. `Makevars` passes those overrides on every cargo invocation:
  ```
  $(CARGO) build --config 'patch.crates-io.miniextendr-api.path="<root>/miniextendr-api"' \
                 --config 'patch.crates-io.miniextendr-macros.path="<root>/miniextendr-macros"' \
                 --config 'patch.crates-io.miniextendr-lint.path="<root>/miniextendr-lint"' \
                 ...
  ```
  Same trick the `just check`/`build`/`test` recipes already use. Workspace edits
  propagate immediately, no copy step, no vendor.
- **Downstream git install** (no monorepo siblings): no `MONOREPO_PATCH_CONFIG`
  set, cargo follows the git URL. First build downloads miniextendr from github;
  subsequent builds use cargo's git cache. Same as any rust crate with git deps.
- **CRAN tarball install**: tarball mode, vendored.

This eliminates the `path = "../../vendor/..."` frozen-deps trap entirely.

## File-by-file changes

### `rpkg/configure.ac`

- Replace `NOT_CRAN` normalization block with `IS_TARBALL` derived from
  `grep -q '^Packaged:' DESCRIPTION`. Export `IS_TARBALL` for `AC_CONFIG_COMMANDS`.
- `cargo-vendor` block:
  - Tarball mode: unpack `inst/vendor.tar.xz`, strip Cargo.lock checksums, error
    if tarball is missing.
  - Source mode: no-op. Do not call cargo-revendor. Do not touch `vendor/`.
- Drop `FORCE_VENDOR` (vestigial).
- Drop the auto-vendor + git-clone bootstrap fallback (~50 lines, only triggered
  in the now-unsupported "downstream install with no vendor and no git deps" case).
- `dev-cargo-config` block: rename to `cargo-config`, write the file in tarball
  mode only; remove it in source mode (so cargo's normal resolution is used).
- Compute `MONOREPO_PATCH_CONFIG`: in source mode, walk up looking for
  `miniextendr-api/Cargo.toml`; emit a single shell-quoted string of three
  `--config 'patch.crates-io.X.path="..."'` flags. AC_SUBST for Makevars.
- `cargo-lockfile-compat`: keep, but only consult vendor in tarball mode.
- `post-vendor`: keep `cargo generate-lockfile` fallback and the `touch
  Cargo.toml` line. Drop the NOT_CRAN branch.

### `rpkg/src/Makevars.in`

- Replace `NOT_CRAN_FLAG` with `IS_TARBALL`.
- Add `MONOREPO_PATCH_CONFIG = @MONOREPO_PATCH_CONFIG@` substitution.
- Append `$(MONOREPO_PATCH_CONFIG)` to every `cargo build` / `cargo rustc`
  invocation (`$(CARGO_AR)` and `$(CARGO_CDYLIB)` recipes).
- `CARGO_OFFLINE_FLAG`: set to `--offline` in tarball mode, empty in source mode.
- Drop the `unpack vendor.tar.xz from Makevars` step (configure handles it).
- The `all:` cleanup-target-dirs branch: keep, but gate on `IS_TARBALL` instead
  of `NOT_CRAN_FLAG != true`. (Cleaning up monorepo's rust-target/ during dev
  was already broken — gated on `inst/vendor.tar.xz` existing, which doesn't in dev.)
- Touch-Cargo.toml branch: gate on source mode.

### `rpkg/src/rust/Cargo.toml`

Revert `path = "../../vendor/..."` to git URLs. Three deps: `[dependencies]
miniextendr-api`, `[build-dependencies] miniextendr-lint`, `[patch.crates-io]`
× 3. After the change, regenerate Cargo.lock via plain `cargo generate-lockfile`.

### `rpkg/src/rust/cargo-config.toml.in`

Unchanged shape. Just no longer auto-deleted in dev — configure decides whether
to write it.

### `rpkg/cleanup` / `cleanup.win` / `cleanup.ucrt`

Keep `rm -rf vendor` only when `inst/vendor.tar.xz` exists (CRAN tarball case).
Already correct. Document why.

### `justfile`

- `configure`: drop `NOT_CRAN=true`. Just `bash ./configure`.
- `configure-cran`: **delete**. There's one configure now.
- `vendor`: drop `--freeze`. Drop `--source-root .` (only needed because
  Cargo.toml had frozen path deps; with git deps cargo metadata works without
  pre-seeding). Keep `--strip-all`, `--compress`, `--blank-md`, `--source-marker`,
  `--force`.
- `r-cmd-build`: depend on `vendor` (so the tarball ships a fresh
  `inst/vendor.tar.xz`).
- `r-cmd-check`: already depends on `vendor`. Keep.
- `devtools-build`, `devtools-check`: depend on `vendor`. devtools::check
  internally does R CMD build, so the tarball it produces needs vendor.tar.xz
  in inst/.
- `rcmdinstall`, `devtools-install`, `devtools-test`, `devtools-document`,
  `devtools-load`: do **not** depend on vendor. Pure source mode.
- `clean`: remove `NOT_CRAN=false` from cleanup invocation.
- `vendor-sync-check`, `vendor-sync-diff`: keep but document they only run
  meaningfully after `just vendor`.

### `.github/workflows/ci.yml`

- Drop `NOT_CRAN` env on every job.
- `r-tests` job: drop `just vendor`, drop `cargo install --path cargo-revendor`,
  install rpkg via `R CMD INSTALL .` directly. Pure source mode.
- `r-check-linux`: keep `just vendor` (R CMD check builds a tarball internally).
  Add an `actions/cache` step keyed on `hashFiles('Cargo.lock', '**/Cargo.toml')`
  that caches `rpkg/inst/vendor.tar.xz` so the vendor step is a no-op on
  cache hit.
- `cran-check`: keep `just vendor`, keep `actions/cache`, drop `NOT_CRAN: false`
  override (now redundant — tarball detection drives mode).
- `cross-package-tests`: drop `just vendor` if the cross-package configure.ac
  uses the same source-mode logic (it should, after templates port).
- `sync-checks`: keep `just vendor` (still need to validate the artifact is
  reproducible from sources for releases).
- Add a smoke-test job: `R CMD INSTALL rpkg` from a clean checkout with no
  cargo-revendor installed, asserts the source-mode path doesn't regress.

### `minirextendr/inst/templates/rpkg/configure.ac`

Mirror all rpkg/configure.ac changes. Same `Packaged:` detection. Drop NOT_CRAN.
Add MONOREPO_PATCH_CONFIG (still useful: scaffolded packages embedded in a
miniextendr monorepo checkout — uncommon but supported today).

### `minirextendr/inst/templates/rpkg/Makevars.in`

Mirror rpkg/src/Makevars.in changes. NOT_CRAN_FLAG → IS_TARBALL,
MONOREPO_PATCH_CONFIG substitution, cargo invocations updated.

### `minirextendr/inst/templates/monorepo/rpkg/{configure.ac,Makevars.in}`

Same as above (these are 1:1 ports of rpkg/).

### `minirextendr` R sources

- `minirextendr/R/vendor.R`: remove any code paths that emit `NOT_CRAN=true`
  into scaffolded files or scripts. Keep the strip-toml-sections logic
  (still needed for vendoring).
- `minirextendr/R/use-configure.R`, `config.R`, `workflow.R`: grep for
  `NOT_CRAN`, remove or invert.
- `minirextendr/R/upgrade.R`: ensure upgrade flow rewrites the new templates
  cleanly without leaving stale NOT_CRAN references.
- `minirextendr/tests/testthat/`: snapshot tests will need refreshing once
  templates change. Run `just minirextendr-test` and accept new snapshots if
  they reflect the new source-mode-by-default behavior.

### `patches/templates.patch`

Regenerate via `just templates-approve` after rpkg/ and templates/ are both updated.

### `tests/cross-package/{producer.pkg,consumer.pkg}/configure.ac` + Makevars.in

Cross-package configure.ac and Makevars.in were last synced before this change.
Apply the same changes (likely a smaller diff because the cross-package
fixtures don't use vendor.tar.xz today). Verify `just cross-install` still works.

### `docs/CRAN_COMPATIBILITY.md`

New page covering:

- The two modes and the `Packaged:` detection.
- Decision rationale: 8–12 min revendor + sccache poisoning + cache invalidation
  on every dev iteration was untenable.
- "Where does each install path land?" table:
  - `R CMD INSTALL .` → source
  - `devtools::install()`, `load_all()`, `install_github()` → source
  - `R CMD INSTALL <tarball>` (from `R CMD build` or CRAN) → tarball
  - `R CMD check` on a source dir → tarball (it builds internally)
- Maintainer release flow: `just vendor` → `just r-cmd-build` → upload tarball.
- CI strategy: cache vendor.tar.xz keyed on Cargo.lock + Cargo.toml hashes.
- Constraint reminders: rpkg/inst/vendor.tar.xz is gitignored (regenerated by CI
  before R CMD check), Cargo.toml uses git deps not path-to-vendor.

### `docs/INDEX.md`

Add link to `CRAN_COMPATIBILITY.md`.

### CLAUDE.md

Update sections that reference NOT_CRAN: "Build contexts" table, "Configure is
mandatory", "After Rust changes", "inst/vendor.tar.xz is not tracked",
"Common Issues". Replace the four-context table with two: source / tarball.

## Open design points (flag in PR)

1. **Cargo.toml shape: git URLs vs path-to-monorepo-sibling**.
   Plan picks git URLs for symmetry with the scaffolded-package template.
   Alternative: `path = "../../../miniextendr-api"` works in monorepo but breaks
   `install_github("A2-ai/miniextendr", subdir = "rpkg")` for downstream consumers
   who clone only rpkg/ via subdir extraction (it's unclear whether `install_github`
   with subdir copies the parent siblings — needs to be tested if we go this way).
   Git URLs sidestep the question.

2. **Caching `vendor/` itself or just `inst/vendor.tar.xz`**.
   Plan caches the tarball (small, deterministic). vendor/ unpacking is fast
   so re-extracting on each CI run is fine; caching vendor/ would save the xz
   decompression but add cache-key complexity.

3. **Drop `vendor-sync-check` from required CI**?
   If vendoring only fires for releases, drift between workspace and vendor/
   is by definition the release-prep step's problem. Move it to a release-only
   workflow. Keep `vendor-verify` (cargo-revendor's reproducibility check)
   in `sync-checks` so CI guarantees the tarball is regenerable from sources.

4. **Remove `cleanup` vendor-rm logic?**
   Today `cleanup` deletes `vendor/` if `inst/vendor.tar.xz` exists. Useful when
   R re-runs install and we want fresh unpacking. Probably keep — costs nothing
   and makes failure modes simpler.

## Implementation order

Files in dependency order so each step leaves the tree buildable:

1. Cargo.toml: revert path → git deps. Regenerate Cargo.lock.
2. configure.ac: rewrite with `Packaged:` detection + MONOREPO_PATCH_CONFIG.
3. Makevars.in: add MONOREPO_PATCH_CONFIG, swap NOT_CRAN_FLAG → IS_TARBALL.
4. Verify `R CMD INSTALL .` works with vendor/ deleted, cargo-revendor uninstalled.
5. Verify `just vendor && R CMD build rpkg && R CMD INSTALL miniextendr_*.tar.gz` works.
6. justfile cleanup.
7. Port to `minirextendr/inst/templates/{rpkg,monorepo/rpkg}/configure.ac + Makevars.in`.
8. Port to `tests/cross-package/*` if applicable.
9. Regenerate `patches/templates.patch`.
10. CI workflow updates + new smoke-test job.
11. minirextendr R helpers: drop NOT_CRAN emission.
12. CLAUDE.md + docs/CRAN_COMPATIBILITY.md + docs/INDEX.md.
13. Run full `just devtools-test`, `just minirextendr-test`, `just rcmdinstall`,
    `just rcmdcheck`. Push, watch CI.

## Out-of-scope (file follow-ups as separate issues)

- cargo-revendor's `--freeze` mode is still useful for non-monorepo consumers
  who want reproducible vendored Cargo.toml. Don't delete; just stop using it
  inside this repo.
- Eventually the dev-mode MONOREPO_PATCH_CONFIG could be replaced by a
  cargo-style per-workspace `[patch]` section if cargo gains conditional patches.
  Not today.
