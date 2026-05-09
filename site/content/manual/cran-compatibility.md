+++
title = "CRAN compatibility and vendoring"
weight = 50
description = "How miniextendr keeps the CRAN install path working without polluting day-to-day development."
+++

How miniextendr keeps the CRAN install path working without polluting day-to-day
development.

## TL;DR

There are exactly two install modes. Configure auto-detects based on a single
signal and configures cargo accordingly:

| Mode | Triggered when | Cargo behavior |
|---|---|---|
| **Source install** | `inst/vendor.tar.xz` is **absent** in the package being installed | Cargo resolves dependencies normally. In monorepo dev, configure writes a `[patch."git+url"]` block in `.cargo/config.toml` that points the three workspace crates at sibling paths. Otherwise cargo fetches the git URL declared in `Cargo.toml`. |
| **Tarball install** | `inst/vendor.tar.xz` is **present** | Configure unpacks the tarball into `vendor/`, writes a `.cargo/config.toml` with `[source.crates-io]` and `[source."git+..."]` redirected to `vendored-sources`, and cargo builds offline. |

That's the entire decision tree. There is no `NOT_CRAN` env var, no
`PREPARE_CRAN`, no `FORCE_VENDOR`, no auto-detected "build context"; just the
file-existence test.

## Self-repair: configure auto-vendors when needed

Before the file-existence test, configure runs an **auto-vendor** block that
produces `inst/vendor.tar.xz` on the fly when ALL of these hold:

1. `inst/vendor.tar.xz` is absent.
2. `cargo-revendor` is on PATH.
3. Source tree has no `.git` ancestor (i.e., we are not in a developer's
   checkout — we are in a build-staging dir or an install-extraction dir).

This is what makes the scaffolding **self-repairing and self-coherent**:

- **Build phase, pkgbuild path**: `devtools::build()` / `pkgbuild::build()` /
  `r-lib/actions/check-r-package` honor `Config/build/bootstrap: TRUE` →
  `bootstrap.R` runs in the staging dir → invokes `./configure` → no `.git`
  ancestor → auto-vendor fires → `inst/vendor.tar.xz` is sealed into the
  tarball. No explicit `just vendor` needed.
- **Install phase, end users**: a tarball that arrives missing
  `inst/vendor.tar.xz` (e.g. published from a raw `R CMD build` that bypassed
  `bootstrap.R`) is repaired at install time — configure runs, no `.git`,
  `cargo-revendor` available → vendor produced → tarball mode → offline build.
- **Dev iteration**: `bash ./configure` from the source tree finds `.git` in
  an ancestor → auto-vendor block is **skipped** → fast `just configure`,
  source-mode dev iteration with monorepo path overrides. Use `just vendor`
  / `miniextendr_vendor()` explicitly when producing a release artifact.
- **CRAN's offline farm**: `cargo-revendor` is not installed, so the auto-vendor
  branch is short-circuited → falls through to source mode → `cargo` tries the
  network → fails loudly. This is the canary: CRAN bouncing a tarball means the
  maintainer shipped one without vendor inside.

## Where each install path lands

| You ran | Mode | Vendor used? | How vendor was produced |
|---|---|---|---|
| `R CMD INSTALL .` (rpkg source dir) | Source | No | n/a (`.git` ancestor → skip) |
| `devtools::install("rpkg")` / `load_all` / `install_local` | Source | No | n/a (`.git` ancestor → skip) |
| `remotes::install_github("A2-ai/miniextendr", subdir = "rpkg")` | Source | No | n/a (`.git` ancestor in fetched repo) |
| `R CMD build rpkg` directly (no bootstrap.R) | Tarball | Yes | configure auto-vendor at install time on user's machine |
| `devtools::build("rpkg")` / `pkgbuild::build()` | Tarball | Yes | bootstrap.R → configure auto-vendor at build time (staging dir, no `.git`) |
| `just r-cmd-build` / `just r-cmd-check` | Tarball | Yes | explicit `just vendor` (recipe dependency) before R CMD build |
| CRAN's autobuilder on a submitted tarball | Tarball | Yes | maintainer's `just vendor` baked it into the tarball |

The second column maps directly to the file-existence test. The third column
shows which trigger produced the vendor — there are three layered triggers
(`just vendor`, `bootstrap.R`-via-pkgbuild, configure auto-vendor at install),
all converging on the same single signal.

## Why

The previous design fired vendoring on every `just configure` so that the
`path = "../../vendor/..."` deps frozen into `Cargo.toml` would resolve. That
meant:

- 8–12 minutes of `cargo revendor` on every dev iteration.
- sccache hit rates collapsed because per-invocation Cargo.lock churn poisoned
  the cache keys.
- `remotes::install_github("A2-ai/miniextendr", subdir = "rpkg")` couldn't run
  without `cargo-revendor` installed and network access to clone the monorepo
  itself for path-dep bootstrap.
- Four overlapping flags (`NOT_CRAN`, `FORCE_VENDOR`, `PREPARE_CRAN`, the
  Rbuild-tempdir heuristic) disagreed about what mode any given invocation was
  in.

Lifting vendoring to a CRAN-prep-only step deletes all of that. Day-to-day
development uses cargo's normal resolution and the `[patch.crates-io]`-style
override that `just check`/`build`/`test` recipes have always done. CRAN
release prep stays self-contained: maintainer runs `just vendor`, ships the
resulting tarball.

## Maintainer release workflow

```bash
just vendor             # 1) Regenerate Cargo.lock in tarball-shape, vendor
                        #    deps to rpkg/vendor/, compress to inst/vendor.tar.xz.
                        #    Dirties Cargo.lock + writes inst/vendor.tar.xz.
just r-cmd-build        # 2) R CMD build rpkg → miniextendr_X.Y.Z.tar.gz.
                        #    Depends on `just vendor` so the tarball ships
                        #    inst/vendor.tar.xz.
just r-cmd-check        # 3) R CMD check the built tarball (--as-cran).
```

Day-to-day commands (`just rcmdinstall`, `just devtools-install`,
`just devtools-test`, `just devtools-document`, `just devtools-load`) do **not**
depend on `just vendor`. They install via source mode, which doesn't need a
vendor tarball at all. Run `just vendor` only when you're producing a build
artifact for CRAN.

## What `just vendor` actually does

```text
1. Move src/rust/.cargo/config.toml aside (so the [patch] override is inactive).
2. Delete and regenerate src/rust/Cargo.lock with cargo against the bare git
   URL — entries for miniextendr-{api,lint,macros} get
   `source = "git+https://github.com/A2-ai/miniextendr#<commit>"`.
3. Restore .cargo/config.toml.
4. Run `cargo revendor` against the freshly regenerated lockfile, producing
   rpkg/vendor/ and rpkg/inst/vendor.tar.xz.
   cargo-revendor recomputes `.cargo-checksum.json` after CRAN-trim: the
   original `package` hash (matching the lockfile's `checksum = ...` line) is
   preserved and the `files` map is refreshed to reflect the trimmed files.
   The committed Cargo.lock can therefore retain its `checksum = ...` lines.
```

Steps 1–3 ensure the lockfile carries the git source for the workspace crates,
which cargo's source replacement needs to redirect to vendor at install time.
Without that, source replacement reports "the source git+... requires a lock
file to be present first before it can be used against vendored source code".

## Cargo.lock shape, drift, and why dev iteration may dirty it

> **See [Cargo.lock shape](./CARGO_LOCK_SHAPE.md)** for a dedicated walkthrough
> of the invariants, the failure modes when they're violated, and the manual
> steps `just vendor` / `miniextendr_vendor()` automate. Summary below.

The committed `rpkg/src/rust/Cargo.lock` is in tarball-shape: workspace crates
have `source = "git+https://github.com/A2-ai/miniextendr#<hash>"`. Registry
`checksum = ...` lines are now **retained** — cargo-revendor writes valid
`.cargo-checksum.json` files that match them.

When you run `cargo build` / `cargo check` in source mode, cargo silently
rewrites the lockfile in place: it re-resolves the workspace crates through
the `[patch."git+url"]` override (so they become `path` sources). **This drift
is expected and harmless for local iteration.** Don't commit it; run
`just vendor` to restore the canonical shape.

If you ever see CI complain that the committed lockfile is in source-shape
instead of tarball-shape, run `just vendor` and commit the regenerated
artifact.

The pre-commit hook (`.githooks/pre-commit`) blocks commits that would
introduce `path+` sources into `rpkg/src/rust/Cargo.lock`.
Run `just lock-shape-check` to verify the committed lockfile is in the correct
shape at any time.

## CI strategy

- **`r-tests`** (Linux): runs `R CMD INSTALL .` on the source dir. Tests source
  mode end-to-end. Does **not** install `cargo-revendor`. This job is the
  implicit smoke test for the source-only install path.
- **`r-check-linux` / `cran-check`**: runs `R CMD check`, which internally
  builds a tarball and tests offline install. Runs `just vendor` first.
  `inst/vendor.tar.xz` is cached across runs keyed on `Cargo.lock` and the
  workspace `Cargo.toml`s, so a no-op re-run skips the vendor step.
- **`sync-checks`**: runs `just vendor` *without* the cache, plus
  `just vendor-sync-check`, to guarantee the tarball is reproducible from
  workspace sources before merge.

## inst/vendor.tar.xz is gitignored

It used to be committed. That caused 22 MB/commit bloat, binary merge
conflicts on every PR that touched a workspace crate, and stale-after-rebase
drift. CI regenerates the tarball before every R CMD check; release tooling
regenerates it at version bump time. Don't try to commit it.

## Stale tarball warning

`inst/vendor.tar.xz` must not linger in the source tree after `just r-cmd-build`
or `just r-cmd-check` finish. Both recipes set `trap 'rm -f rpkg/inst/vendor.tar.xz' EXIT`,
but the trap does not fire on `SIGKILL`. If the file is left behind:

1. `just configure` sees it and sets `IS_TARBALL_INSTALL=true`.
2. The next `just rcmdinstall` (or `R CMD INSTALL rpkg`) runs `make` with
   `IS_TARBALL_INSTALL=true` and `ABS_RPKG_SRCDIR` pointing to the **source**
   `rpkg/src/`. The tarball-mode cleanup in `Makevars.in` then deletes
   `src/rust/.cargo/` from the source tree.
3. The monorepo `[patch."git+url"]` override is gone; cargo silently resolves
   the three workspace crates from `git+https://...#<sha>` instead of local siblings.

Recovery: use `just clean-vendor-leak` (monorepo) or
`miniextendr_clean_vendor_leak()` (scaffolded packages) to remove the stale
tarball, then `just configure` to regenerate `.cargo/config.toml`.
`miniextendr_doctor()` detects both the stale tarball and a missing
`config.toml` and prints the fix.

Dev-consume recipes (`just rcmdinstall`, `just devtools-test`,
`just devtools-load`, `just devtools-install`) will abort with an error if the
tarball is present in the source tree, preventing silent tarball-mode iteration.
See CLAUDE.md "Vendor tarball is a latch" for the full context and the
`just test-bootstrap-vendor` regression test (#441).

## Constraints, in case you're tempted

- `Cargo.toml` must keep miniextendr-{api,lint,macros} declared as `git = "..."`.
  Path deps to `../../vendor/...` would require `vendor/` to exist in source
  mode, which is exactly what we removed. Path deps to monorepo siblings
  (`../../../miniextendr-api`) would break tarball install (the tarball doesn't
  carry siblings).
- Configure must not mutate `Cargo.toml` or `*.rs` (CLAUDE.md project rule).
  Mutating `Cargo.lock` in tarball mode is acceptable — it's an artifact, not
  a source — but `just vendor` does that pre-build, not configure at install
  time.
- `[ -f inst/vendor.tar.xz ]` is the only source-vs-tarball signal. Don't add
  a second one. Maintenance load lives in the number of switches.

## Symbols cleanup, for grep-bait

Removed entirely from this codebase:

- `NOT_CRAN`
- `FORCE_VENDOR`
- `PREPARE_CRAN`
- `BUILD_CONTEXT` (the dev-monorepo / dev-detached / vendored-install /
  prepare-cran enum)
- `cargo revendor --freeze` invocations from `just vendor`
- The "auto-vendor on first install" + git-clone-bootstrap fallback in
  `configure.ac`
- The unpack-vendor-from-Makevars step in `Makevars.in`

If you find a stray reference, it's vestigial — delete it.
