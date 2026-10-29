# Maintainer Guide

This document covers maintenance tasks for the miniextendr project.

## Version Management

Versions are tracked in two places that must stay in sync:

- `Cargo.toml` (`[workspace.package].version`)
- `rpkg/DESCRIPTION` (`Version:`)

The workspace crates inherit theirs with `version.workspace = true`.

### Bumping Version

```bash
./scripts/bump-version.sh 0.2.0
```

This updates `Cargo.toml`, every `DESCRIPTION` (`rpkg`, `minirextendr` and the
two cross-package fixtures) and `tests/cross-package/shared-traits/Cargo.toml`.

R development versions (e.g., `0.2.0.9000`) are allowed and will match the base version `0.2.0` in CI checks.

## Regenerating Configure Scripts

After modifying `rpkg/configure.ac`, regenerate the configure script:

```bash
cd rpkg
autoreconf -vif
```

Or use the justfile:

```bash
just configure  # runs autoconf + ./configure
```

### When to Regenerate

- After editing `configure.ac`
- After editing `Makevars.in`
- After changing autoconf macros
- Before committing changes to configure.ac

### Dependencies

Requires GNU autotools:

```bash
# macOS
brew install autoconf automake

# Debian/Ubuntu
apt-get install autoconf automake
```

## Development Workflow

### Full Rebuild (after macro changes)

When changing proc-macros, the full sequence is:

```bash
just configure && just rcmdinstall && just devtools-document
```

`just devtools-document` always runs roxygen2; run it after `rcmdinstall` so it
sees the regenerated wrappers. For pure R/roxygen changes it can run on its own.

### Quick Iteration

For most changes:

```bash
just check              # Fast cargo check
just rcmdinstall        # Build and install
```

### Running Tests

```bash
# Rust tests
just test

# R tests
just devtools-test

# Full R CMD check
just r-cmd-check
```

## Rustdoc Maintenance

Public and internal APIs should stay documented as they evolve.

- Run a targeted doc lint snapshot when touching API-heavy modules:

```bash
RUSTFLAGS='-Wmissing-docs' cargo check -p miniextendr-api --lib
```

- Prefer documenting internals in-place with rustdoc comments (`///`) near trait
  constants, enum variants, and error fields so generated docs remain useful.
- For raw header-mirror FFI declarations, document key types and safety model;
  avoid duplicating full upstream header docs verbatim.

## CI and Release Gates

"Blocking" below is maintainer policy: don't merge or tag while an applicable
check is failing or still running. GitHub enforces none of it. As of
2026-10-14, `main` has no branch protection and no ruleset, so no status check
is required and an enabled merge button proves nothing (#1391).

### What `CI Success` covers

`.github/workflows/ci.yml` runs on pull requests, pushes to `main`, `v*` tag
pushes, a weekly schedule (Saturday 06:00 UTC) and `workflow_dispatch`. Its
`CI Success` job is the one check to require. It needs Generated Files Check,
Version Check, Detect changes, Sync Checks, Docs Link Check, Docs Examples
Check, Rust lint, Rust tests, R CMD check / Linux, R tests / Linux, the R GC-stress shards,
Cross-Package Trait ABI, minirextendr, CRAN-like check and Bootstrap Vendor
Test. A skipped job passes; a failed or cancelled one fails the aggregate. A
green `CI Success` therefore means only that nothing which ran failed:

- **Path filters.** On a pull request or a push to `main`, the `changes` job
  skips jobs whose inputs did not change. The Rust jobs need Rust, Cargo,
  workflow or `justfile` changes; the R jobs need R, package, scaffolding or
  framework-crate changes; Docs Link Check needs docs, README, site or rustdoc
  inputs. Scheduled, dispatched and tag runs skip the filter and run every
  path-filtered job.
- **R versions.** R CMD check / Linux checks R 4.4 and the current release on
  every run. Pushes, tags, the schedule and dispatched runs add oldrel and
  devel. On a pull request, a label named after an R version alias (`oldrel` or
  `oldrel/N`, `devel`, `next`) adds that leg, and `full-ci` adds oldrel and
  devel. The devel and next legs may fail without failing the run.
- **Label-gated jobs.** On a pull request, the three `MALLOC_CHECK_` rounds
  in R tests / Linux need `heap-check`; other events run them. The GC-stress
  shards need `MX_GC_STRESS_CI=true` on every event, plus `gc-stress` on a
  PR and matching R paths. The variable is currently unset, so the shards
  skip even on main, tags and dispatch (#2070). A label alone cannot enable
  them. See [GCTORTURE_TESTING.md](GCTORTURE_TESTING.md#how-ci-runs-the-gctorture-tests-opt-in-per-pr).
  A job that runs and fails blocks `CI Success`.
- **Cancellation.** A new push to a pull request cancels the superseded run. A
  run for a push to `main` is never cancelled once it starts, but while it
  runs only the newest waiting run is kept, so a burst of merges does not get
  one run per commit.

### Gate table

| Check | Merge | Release |
|---|---|---|
| `CI Success` | Blocking on every pull request. | Blocking on the commit to be tagged, from the dispatched release run. That run checks R 4.4, release, oldrel and devel and the heap-check rounds. Feature combos and Windows skip; the GC-stress shards also skip unless `MX_GC_STRESS_CI=true`. |
| CRAN-like check (inside `CI Success`) | Blocking when R inputs change. | Blocking for every `rpkg` release. It builds the vendored source tarball and checks the tarball with `--as-cran`. |
| Docs Link Check (inside `CI Success`) | Blocking when docs inputs change. | Blocking when the release changes docs. `just site-check` runs it locally. |
| Docs Examples Check (inside `CI Success`) | Blocking when docs/rustdoc inputs change. Compiles the marked examples in docs and end-user skills. | Blocking on the dispatched release run. |
| webR (`webr.yml`, not in `CI Success`) | Triggers only on pull requests and pushes that touch its paths (framework crates, `rpkg`, `minirextendr`, the cross-package and webR test fixtures, the webR Dockerfiles, Cargo manifests). On a pull request, every job except `pin-lockstep` also needs the `webr` label; apply it when the PR touches wasm-specific code (see [WEBR.md](WEBR.md#ci)). A labeled run must pass. Pushes to `main` run it in full. | Blocking when the release changes webR-covered paths: dispatch it and require every job, including the tier-2 install and the tier-3 session. |
| macOS R CMD check, arm64 and x86_64 (`ci.yml`, not in `CI Success`) | Never runs on pull requests or pushes; schedule and dispatch only. | Blocking for every `rpkg` release: both rows of the dispatched release run. |
| Feature runtime legs (`ci.yml`, not in `CI Success`) | Schedule and dispatch only. | Conditional: blocking when feature selection or code generation changed. They run in the dispatched release run. |
| Feature combos (`ci.yml`, not in `CI Success`) | Schedule only; dispatch does not run it. | Conditional, on the same changes: run `just check-features` locally. |
| Standalone round-trip e2e (`ci.yml`, not in `CI Success`) | Schedule, dispatch, or the `run-e2e` label on a pull request. | Conditional: blocking when scaffolding or build plumbing changed. It runs in the dispatched release run. |
| gctorture nightly (`gctorture-nightly.yml`) | Manual dispatch only: the nightly schedule is disabled pending #2070. | Conditional: blocking for FFI, allocation, GC-rooting, conversion or runtime changes. Dispatch it on the release branch; a skipped CI stress job is not stress-test evidence. |
| Miri nightly (`miri-nightly.yml`) | Informational: the test step is `continue-on-error`. | Informational: review the latest report. |
| Windows R CMD check (`ci.yml`, not in `CI Success`) | Runs only on pull requests with the `windows` label, and never blocks. | No gate: Windows support is deferred (#1654, which collects #94, #104 and #594). Say so in the release notes. |
| Pages deployment, webR image mirror | Operational, not gates. | Not gates: a failure affects publishing infrastructure, not the release artifacts. |

## Release Checklist

[RELEASING.md](RELEASING.md) covers versioning and building the release
artifacts. Every check below must run on the exact commit you tag, not on an
older green `main`.

1. **Land the version bump on `main` through a pull request**, then pin the
   merged commit to a branch, so the dispatched runs test it even if `main`
   moves on. Rebase and squash merges both rewrite the commit, so take the SHA
   from `main`, not from the pull request's branch, and confirm it is the bump:

   ```bash
   git fetch origin
   sha=$(git rev-parse origin/main)   # the bump commit, the commit you will tag
   git show --stat "$sha"
   git push origin "$sha:refs/heads/release/vX.Y.Z"
   ```

2. **Dispatch the release runs on that branch.** A dispatched `ci.yml` run
   skips the path filter and includes the oldrel and devel legs, heap-check
   rounds, both macOS architectures, feature runtime legs and standalone
   round-trip. Feature combos and Windows skip. The GC-stress shards also
   skip while `MX_GC_STRESS_CI` is unset; run `just devtools-test-stress` at
   the release SHA to cover them during the #2070 CI suspension.

   ```bash
   gh workflow run ci.yml --ref release/vX.Y.Z
   gh workflow run webr.yml --ref release/vX.Y.Z               # webR-covered paths changed
   gh workflow run gctorture-nightly.yml --ref release/vX.Y.Z  # runtime-sensitive code changed
   ```

3. **Check that every run tested `$sha` and passed.** `CI Success` must be
   green, and so must the jobs it leaves out: both macOS rows, the feature
   runtime legs and the round-trip. The gctorture sweep has six shards with
   a 300-minute timeout; some have hit that limit (#2070). A timeout is
   missing validation, not a passing release gate.

   ```bash
   gh run list --commit "$sha" --event workflow_dispatch --json workflowName,status,conclusion,url
   ```

4. **Run the local checks at `$sha`.** `just r-cmd-check` builds the vendored
   tarball and checks it. Run `just check-features` when feature selection
   changed, since CI runs it only on the weekly schedule. Miri is
   informational, but review its latest report:

   ```bash
   just r-cmd-check
   just check-features
   gh run list --workflow miri-nightly.yml --limit 1
   ```

5. **Tag `$sha` and push the tag**, then delete the release branch:

   ```bash
   git tag -a vX.Y.Z -m "vX.Y.Z" "$sha"
   git push origin vX.Y.Z
   git push origin --delete release/vX.Y.Z
   ```

   The tag push runs `ci.yml` once more, without the macOS, feature runtime and
   round-trip jobs. Its CRAN-like check uploads the checked, vendored source
   tarball as the `miniextendr-source-package` artifact. That tarball is for
   native installs only (the job does not regenerate the wasm registry), so
   build the release artifacts as [RELEASING.md](RELEASING.md#releasing-rpkg)
   describes.

6. **Record the evidence.** Put `$sha` and the run URLs in the release notes.
   While Windows support is deferred (#1654), state that the release has no
   Windows gate. The full release-path audit on every platform (vendoring,
   the built-tarball check, a clean-machine install and a scaffolded package)
   has not been run yet (#1335).

There is no crates.io publication step today; distribution is from the Git tag
and the attached R source tarball.

## Vendoring for CRAN

R packages submitted to CRAN must be self-contained. The `vendor` recipe (which
calls `cargo-revendor`) produces the offline bundle:

```bash
just vendor
```

This:

1. Regenerates `Cargo.lock` in tarball-shape (git-URL sources for the
   `miniextendr-{api,lint,macros}` workspace crates)
2. Vendors all crates.io dependencies (proc-macro2, quote, syn, etc.) into `rpkg/vendor/`
3. Trims each vendored crate with `--strip-all`, which drops test/bench/example/bin
   targets and dev-dependencies, the same trim `minirextendr::miniextendr_vendor()` applies
4. Compresses the result to `rpkg/inst/vendor.tar.xz`

`just configure` does **no** vendoring in dev mode — it only generates
`Makevars` and `.cargo/config.toml`, and auto-detects tarball mode from the
presence of `inst/vendor.tar.xz`. Once the tarball exists, `configure` unpacks
it and writes the offline `[source]` replacement; delete it with
`just clean-vendor-leak` to return to source-mode dev iteration.

## Useful Commands

```bash
# List all just recipes
just

# Clean build artifacts
just clean

# Check all crates compile
just check

# Format code
just fmt

# Run clippy
just clippy

# Build documentation
just doc

# Expand macros (requires cargo-expand)
just expand

# Run benchmarks
just bench
```

## File Locations

| Purpose | Location |
|---------|----------|
| Workspace config | `Cargo.toml` |
| R package | `rpkg/` |
| Configure script | `rpkg/configure.ac` |
| Makefile template | `rpkg/src/Makevars.in` |
| Vendored crates | `rpkg/vendor/` |
| CI workflow | `.github/workflows/ci.yml` |
| Version bump script | `scripts/bump-version.sh` |
