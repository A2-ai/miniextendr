# Build System: Test Plan

**Date**: 2026-05-11
**Companion**: `analysis/build-system-investigation-2026-05-11.md` (read first
for context and §-references used here).

This plan converts the corners and decisions identified in the investigation
into concrete, executable tests. Each test names: trigger, prior state,
expected outcome, expected install mode, observable side effects.

The plan is structured for incremental adoption:

1. **§T0 quick gates** — sanity checks first PRs run.
2. **§T1–T8 functional matrix** — every installer × every project structure ×
   every realistic prior state.
3. **§T9–T11 regression nets** — guard the known-bug fixes once landed.
4. **§T12 implementation strategy** — where each test should live and how to
   run them.
5. **§T13 the matrix in compact form** — single-page cell view.

Glossary:

- **M** = monorepo project structure (rust workspace top + rpkg subdir).
- **S** = standalone rpkg structure (R package with `src/rust/`).
- **T** = a built tarball (`R CMD build` output, includes `inst/vendor.tar.xz`).
- **dev mode** = source mode with `[patch."git+url"]` from monorepo OR with
  network resolution from `git = "..."` in standalone.

---

## T0 — Quick gates (mandatory pre-PR)

These are fast smoke tests. Failures block PRs.

| Test | What | Where it runs | Time |
|---|---|---|---|
| T0.1 | `just configure` is idempotent (run twice, same output) | local + CI | < 5 s |
| T0.2 | `just lock-shape-check` passes | local + CI | < 1 s |
| T0.3 | `just vendor-sync-check` passes | local + CI | < 5 s |
| T0.4 | `minirextendr_doctor()` reports clean | local + CI | < 2 s |
| T0.5 | `git status` shows no uncommitted generated files after `just configure && just rcmdinstall && just devtools-document` (smoke for "configure shouldn't touch tracked files") | local | < 90 s |

---

## T1 — Project structure parity (M vs S identical install behavior)

**Goal**: prove monorepo and standalone packages have identical
configure/Makevars output at install time.

| # | Setup | Trigger | Expected |
|---|---|---|---|
| T1.1 | M: `create_miniextendr_monorepo("/tmp/m_test")` + `just configure` in rpkg | diff `src/Makevars` against S equivalent | files differ only in `PACKAGE_NAME`, `ABS_TOP_SRCDIR`, `MONOREPO_ROOT` |
| T1.2 | M + S separate scaffolds | `R CMD build` both; tarballs install | both `R CMD INSTALL <tarball>` succeed with identical install logs (modulo paths) |
| T1.3 | M + S separate scaffolds | `devtools::load_all` both | both compile clean, both `library(<pkg>)` works |
| T1.4 | M scaffolded with `use_native_package("cli")` | `R CMD INSTALL <path>` | compiles without `r_shim.h not found`  (this currently fails — §2.4) |

T1.4 is the regression test for the §2.4 r_shim.h gap. Until fixed, this is
expected to fail and serves as the bug repro.

---

## T2 — Fresh-clone smoke (every installer × M, S)

**Goal**: prove a user who freshly clones a scaffolded package can install
via every supported entry point.

For each project structure (M, S), starting from a fresh `git clone` (or
equivalent fresh scaffold):

| # | Installer | Expected mode | Expected to succeed |
|---|---|---|---|
| T2.1 | `R CMD INSTALL <path>` | source + monorepo patch (M) / source-network (S) | Yes (M: offline; S: needs network) |
| T2.2 | `R CMD build <path>` → `R CMD INSTALL <tarball>` | bootstrap.R vendors, install tarball-offline | Yes |
| T2.3 | `R CMD check <tarball>` | bootstrap.R vendors, check passes | Yes (full --as-cran clean) |
| T2.4 | `devtools::install(path)` | bootstrap.R vendors, tarball install | Yes |
| T2.5 | `devtools::load_all(path)` | source + monorepo patch / source-network | Yes |
| T2.6 | `devtools::document(path)` | source + monorepo patch / source-network | Yes; produces `R/<pkg>-wrappers.R` + NAMESPACE |
| T2.7 | `devtools::build(path)` then `R CMD INSTALL` of result | tarball install offline | Yes |
| T2.8 | `devtools::check(path)` | tarball check | Yes |
| T2.9 | `pkgbuild::build(path)` then install | tarball install offline | Yes |
| T2.10 | `remotes::install_local(path)` | tarball stage offline | Yes |
| T2.11 | `remotes::install_local(<tarball>)` | tarball install offline | Yes |
| T2.12 | `pak::local_install(path)` | tarball stage offline | Yes |

These should be parameterized over (M, S) so the matrix is 2 × 12 = 24 cells.

---

## T3 — End-user GitHub install (no .git, no local toolchain)

**Goal**: validate that a user installing from a GitHub archive gets a
working package whether or not they have `cargo-revendor`.

For both M and S (where applicable — note M's monorepo siblings won't be
available via `install_github`, so this is mostly S):

| # | Setup | Trigger | Expected |
|---|---|---|---|
| T3.1 | cargo-revendor on PATH; no `inst/vendor.tar.xz` in archive | `remotes::install_github("user/repo")` | configure auto-vendor fires → tarball mode → offline build succeeds |
| T3.2 | No cargo-revendor; archive lacks `inst/vendor.tar.xz` | `remotes::install_github("user/repo")` with network | source mode; cargo network-fetches; succeeds if online |
| T3.3 | No cargo-revendor; no network | `remotes::install_github("user/repo")` | fails loudly with cargo network error (intended) |
| T3.4 | Archive includes `inst/vendor.tar.xz` (maintainer pre-bundled) | `remotes::install_github("user/repo")` no cargo-revendor, no network | succeeds offline |
| T3.5 | Same as T3.4 via `pak::pkg_install("github::user/repo")` | tarball mode offline | succeeds |

T3.4 is the "CRAN-style end-user" case. T3.3 is the "intended canary" — it
must fail with a clear error pointing at the absent tarball or cargo-revendor.

---

## T4 — Warm dev iteration (incremental compilation)

**Goal**: prove dev workflows are truly incremental.

| # | Setup | Trigger | Expected |
|---|---|---|---|
| T4.1 | `just rcmdinstall` once (warm); no source changes | `just rcmdinstall` again | `cargo build` exits ≤ 2 s; no recompilation log lines; no network |
| T4.2 | Warm; touch one `.rs` file (no semantic change) | `just rcmdinstall` | cargo recompiles only that crate + downstream; ≤ 30 s |
| T4.3 | Warm | `devtools::load_all("rpkg")` | cargo: zero rebuilds, package loads in ≤ 5 s |
| T4.4 | Warm | `devtools::document("rpkg")` | cargo: zero rebuilds; roxygen2 still runs over wrappers.R (known limitation, time bound: ≤ 30 s) |
| T4.5 | Warm | `devtools::test("rpkg")` | cargo: zero rebuilds; tests run |
| T4.6 | `R/foo.R` touched (no Rust change) | `devtools::document` | cargo: zero rebuilds; NAMESPACE regenerated |
| T4.7 | After a successful `just devtools-document`, then 2nd run with no changes | configure should be a near-no-op; entire run ≤ 3 s | (currently ~5 s — see §6.6 — flag if regresses) |

Observability: parse cargo's output for lines like
`Compiling miniextendr-api v...` — none should appear on no-op runs.

---

## T5 — Latch leak guards

**Goal**: prove the §3.2 / §7.6 latch-leak failure modes are caught.

| # | Setup | Trigger | Expected |
|---|---|---|---|
| T5.1 | Dev tree with `inst/vendor.tar.xz` present (leaked) | `just rcmdinstall` | hard fail with `_assert-no-vendor-leak` message |
| T5.2 | Same | `just devtools-test` | hard fail |
| T5.3 | Same | `just devtools-load` | hard fail |
| T5.4 | Same | `just clean-vendor-leak` | tarball removed; subsequent `just configure` restores `.cargo/config.toml` to source-mode config |
| T5.5 | Same | `minirextendr_doctor()` | reports both stale tarball and (post-cleanup) missing config |
| T5.6 | Same | bare `R CMD INSTALL rpkg/` from shell | currently silently goes into tarball mode → BROKEN. Expected to detect with R-side guard once added (§15 follow-up). |
| T5.7 | Same | `miniextendr_build()` (R wrapper) | hard fail (R-side guard — needs adding to match `just`) |

T5.6 and T5.7 are the holes in current coverage; tests should be added now
and remain `expect_failure` until the R-side guard lands.

---

## T6 — Wrapper-gen skip (once §6.4 is implemented)

**Goal**: prove the wrapper-skip optimization is correct.

| # | Setup | Trigger | Expected |
|---|---|---|---|
| T6.1 | Built tarball with committed `R/<pkg>-wrappers.R` | `R CMD INSTALL <tarball>` | install logs show "tarball install: using pre-shipped R/<pkg>-wrappers.R"; no cdylib build invocation; no `Rscript -e "dyn.load"` invocation |
| T6.2 | Same | Install timing | install completes ≥ 10 s faster than pre-optimization baseline |
| T6.3 | Same | After install, load package and call several `#[miniextendr]` functions | All work — wrapper file is correct |
| T6.4 | Tarball with intentionally-stale wrappers.R (test fixture) | `R CMD INSTALL <tarball>` | install succeeds (using stale wrappers); test fails when calling renamed function — proves the optimization is *not* a guard against pre-commit-hook bypass (that's the hook's job) |
| T6.5 | Source-mode install (`R CMD INSTALL <path>`) | configure → make | cdylib build still runs (skip does not fire) |
| T6.6 | Wasm install (CC=emcc) | configure → make | cdylib still skipped via wasm branch (no regression) |
| T6.7 | `MINIEXTENDR_FORCE_WRAPPER_GEN=1 R CMD INSTALL <tarball>` (if escape hatch added) | configure → make | cdylib build runs even in tarball mode |

---

## T7 — Vendor consistency

**Goal**: §7.6 and §7.7 — prove the two vendor paths agree, and prove
bootstrap.R produces a working tarball regardless of dev lock state.

| # | Setup | Trigger | Expected |
|---|---|---|---|
| T7.1 | Clean checkout | `just vendor`, capture `rpkg/vendor/` byte content | record baseline |
| T7.2 | Clean checkout | `Rscript -e 'minirextendr::miniextendr_vendor("rpkg")'`, capture vendor | byte-identical to T7.1 baseline (currently fails — §7.7) |
| T7.3 | Dev tree with source-shape lock (`source = "path+file:///..."` for framework crates) | `Rscript -e 'devtools::build("rpkg")'` | tarball ships tarball-shape lock (no `path+`); offline install succeeds (currently fails — §7.6) |
| T7.4 | After successful `just vendor` then `just r-cmd-check` | check passes including the lock-shape-check step | offline reproduction succeeds |
| T7.5 | Vendor tarball with checksums in `.cargo-checksum.json` and `Cargo.lock` | `R CMD INSTALL <tarball>` | cargo verifies checksums end-to-end without warnings |
| T7.6 | Vendor tarball with `{"files":{}}` checksums (legacy `miniextendr_vendor()` output) | `R CMD INSTALL <tarball>` | document whether cargo warns; if so, T7.2 becomes blocker |

---

## T8 — CRAN simulation

**Goal**: prove tarball install behaves like CRAN's offline farm.

| # | Setup | Trigger | Expected |
|---|---|---|---|
| T8.1 | Built tarball with `inst/vendor.tar.xz`; isolate network | `R CMD INSTALL <tarball>` | succeeds offline |
| T8.2 | Built tarball *without* `inst/vendor.tar.xz`; no cargo-revendor; offline | `R CMD INSTALL <tarball>` | fails with clear error (cargo network blocked); this is the canary (§7.8) |
| T8.3 | Built tarball with `inst/vendor.tar.xz` but stale (vendor doesn't match Cargo.lock) | `R CMD INSTALL <tarball>` | fails with cargo verification error; not silent |
| T8.4 | Built tarball; cargo-revendor on PATH but no network; missing `inst/vendor.tar.xz` | `R CMD INSTALL <tarball>` | succeeds (auto-vendor branch in configure fires — needs cargo-revendor to succeed offline; verify) |

Implement T8 by setting `RUST_NET_DISABLED=1` or `HTTP_PROXY=http://0.0.0.0:1`
in the test env to break network.

---

## T9 — pak / remotes subprocess env propagation

**Goal**: §8.4 — verify env vars propagate.

| # | Setup | Trigger | Expected |
|---|---|---|---|
| T9.1 | `Sys.setenv(CARGO_HOME = "/tmp/test-cargo")` in R session | `pak::local_install("rpkg")` | cargo uses `/tmp/test-cargo`, not `~/.cargo` |
| T9.2 | Same | `remotes::install_local("rpkg")` | same — expected to use the override |
| T9.3 | Same | `devtools::install("rpkg")` | same |

If T9.1 fails (pak doesn't propagate), document as known limitation +
workaround (set via `.Renviron`).

---

## T10 — Cross-installer parity

**Goal**: same inputs → same installed package.

Define a reproducibility check:

| # | Setup | Trigger | Expected |
|---|---|---|---|
| T10.1 | Same source tree | Install via `devtools::install`, `remotes::install_local`, `pak::local_install`, `R CMD INSTALL <tarball>` (built once) | All four installed packages have identical: NAMESPACE, R sources, compiled .so symbol table |
| T10.2 | Hash the installed `.so` (release profile, no debug info) | Bit-identical across runs on same machine | (Cargo's deterministic build flag may be needed; if not bit-identical, document) |

T10.2 is aspirational — Rust isn't always bit-deterministic. T10.1 is the
practical check.

---

## T11 — Both project structures pass identical matrices

**Goal**: §1's claim "no path is monorepo-only or standalone-only."

| # | Setup | Trigger | Expected |
|---|---|---|---|
| T11.1 | Run T2 over (M, S) | full 24-cell pass | all pass |
| T11.2 | Run T4 over (M, S) | warm-iteration parity | both incremental in equivalent ways |
| T11.3 | Run T5 over (M, S) | latch-leak guards | both blocked equivalently |
| T11.4 | Run T8.1 over (M, S) | CRAN offline simulation | both succeed |

---

## T12 — Implementation strategy

### T12.1 Where each test should live

| Bucket | Location | Pattern |
|---|---|---|
| Quick gates (T0) | existing `justfile` recipes + pre-commit hook + CI step | `just check-all` aggregator already exists |
| Functional matrix (T2, T11) | `tests/build-system/` (new directory) | Bash + Rscript drivers, one script per cell, run via `just test-build-system` |
| Latch leak (T5) | `tests/build-system/latch-leak/` | bash scripts simulating leaked state; assert exit code + stderr substring |
| Wrapper-gen skip (T6) | `tests/build-system/wrapper-skip/` | once §6.4 lands; capture install logs and diff vs. baseline |
| Vendor consistency (T7) | `tests/build-system/vendor-parity/` | byte-diff vendor trees produced by two paths; can run as testthat in `minirextendr` |
| CRAN simulation (T8) | `tests/build-system/cran-sim/` | run with `HTTP_PROXY=http://0.0.0.0:1` to force offline |
| Subprocess env (T9) | `minirextendr/tests/testthat/test-subprocess-env.R` | testthat-style |
| Cross-installer parity (T10) | `tests/build-system/parity/` | bash + diff |
| Warm-iteration timings (T4) | local-only (timing varies); CI runs T4.1–4.6 as boolean "no recompile occurred" assertions, not timing |

### T12.2 Test fixture: a "fresh-clone" snapshot

Many of these tests need a "fresh clone" state. Two options:

- **Worktree-based**: `git worktree add /tmp/fresh main` for each test;
  tear down after. Robust but slow.
- **Tar archive**: pre-baked tarball of a known scaffolded package committed
  to `tests/build-system/fixtures/`. Fast but drifts with code.

**Recommendation**: worktree-based for monorepo tests (the repo itself is the
monorepo); freshly-scaffolded-via-minirextendr for standalone tests, with a
caching layer that re-scaffolds only if `minirextendr/` has changed.

### T12.3 What CI should actually run

CI is already constrained on runtime. Run:

- T0 always (already does)
- T2 always (subset — at least T2.1, T2.4, T2.5, T2.6, T2.8 for both M and S)
- T5 latch-leak guards (cheap, fast)
- T7 vendor consistency (T7.1–7.4)
- T8.1, T8.2 (CRAN simulation)
- T11 parity (a single representative cell, e.g., T11.1 with `devtools::install`)

Daily/nightly runs:

- T6 wrapper-skip (once landed)
- T9 subprocess env
- T10 cross-installer parity
- Full T4 (warm-iteration) — measure regressions

### T12.4 Test-failure clarity

Every test must emit, on failure:

- The installer command used.
- The project structure (M or S).
- The configured install mode (parse `configure` output for
  `install mode = ...`).
- The relevant excerpt of cargo/make/R logs around the failure point.

A common shell helper `tests/build-system/lib.sh` should standardize this.

---

## T13 — The matrix in compact form

A single-page view of (project × installer × prior state) → expected mode +
expected outcome.

Legend:

- `S+m` = source mode with monorepo `[patch."git+url"]`
- `S+n` = source mode with network (no monorepo)
- `T+o` = tarball mode (offline, vendored)
- `T+a` = tarball mode after auto-vendor at configure time
- `✓` = expected pass
- `✗` = expected fail
- `BUG-N` = currently broken; tracked at issue N
- `—` = not applicable

```
                                       │   monorepo (M)            │   standalone (S)
                                       │  fresh │ warm  │ leak  │  fresh │ warm  │ leak
─────────────────────────────────────  │────────│───────│───────│────────│───────│───────
R CMD INSTALL <path>                   │ S+m ✓  │ S+m ✓ │ T+o ✗ │ S+n ✓¹ │ S+n ✓ │ T+o ✗
R CMD INSTALL <tarball>                │ T+o ✓  │ T+o ✓ │  —    │ T+o ✓  │ T+o ✓ │  —
R CMD build <path> + INSTALL           │ T+o ✓  │ T+o ✓ │ BUG-A │ T+o ✓² │ T+o ✓ │ BUG-A
R CMD check <tarball>                  │ T+o ✓  │ T+o ✓ │  —    │ T+o ✓  │ T+o ✓ │  —
R CMD check <path>                     │  ✓³    │  ✓³   │  ✗    │  ✓³    │  ✓³   │  ✗
devtools::install(path)                │ T+o ✓  │ T+o ✓ │ BUG-A │ T+o ✓  │ T+o ✓ │ BUG-A
devtools::load_all(path)               │ S+m ✓  │ S+m ✓ │ JUST✗ │ S+n ✓  │ S+n ✓ │ JUST✗
devtools::document(path)               │ S+m ✓  │ S+m ✓ │ JUST✗ │ S+n ✓  │ S+n ✓ │ JUST✗
devtools::build(path)                  │ T+o ✓  │ T+o ✓ │ BUG-A │ T+o ✓  │ T+o ✓ │ BUG-A
devtools::check(path)                  │ T+o ✓  │ T+o ✓ │ BUG-A │ T+o ✓  │ T+o ✓ │ BUG-A
remotes::install_local(path)           │ T+o ✓  │ T+o ✓ │ BUG-A │ T+o ✓  │ T+o ✓ │ BUG-A
remotes::install_local(<tarball>)      │ T+o ✓  │ T+o ✓ │  —    │ T+o ✓  │ T+o ✓ │  —
remotes::install_github (cargo-rev)    │  —     │  —    │  —    │ T+a ✓  │ T+a ✓ │  —
remotes::install_github (no rev, net)  │  —     │  —    │  —    │ S+n ✓  │ S+n ✓ │  —
remotes::install_github (no rev, off)  │  —     │  —    │  —    │ ✗⁴     │ ✗⁴    │  —
pak::pkg_install("github::...")        │  —     │  —    │  —    │ T+a ✓  │ T+a ✓ │  —
pak::local_install(path)               │ T+o ✓  │ T+o ✓ │ BUG-A │ T+o ✓  │ T+o ✓ │ BUG-A
install.packages(repos=NULL, tarball)  │ T+o ✓  │ T+o ✓ │  —    │ T+o ✓  │ T+o ✓ │  —
```

Footnotes:

1. S+n requires network or cargo registry cache. Air-gapped + cold cache = fail.
2. T+o via S relies on `bootstrap.R` running `cargo-revendor`. Needs
   cargo-revendor on PATH.
3. `R CMD check <path>` is functionally OK but disrecommended (misses
   `Authors@R` → `Author/Maintainer`). Should not be the primary signal.
4. Intended canary failure — matches CRAN offline farm behavior.

`BUG-A` = §3.2 latch leak. Currently:

- `just`-routed installers → blocked by `_assert-no-vendor-leak`.
- Non-just installers (`R CMD ...`, bare `devtools::...`, `pak`, `remotes`) →
  silently go into tarball mode, then Makevars cleanup deletes
  `src/rust/.cargo/`. The next dev configure must re-create it.
- Cell shows `BUG-A` because nothing currently *prevents* the wrong mode for
  these paths.

`JUST✗` = `just devtools-{load,document,test}` blocks via
`_assert-no-vendor-leak`. Bare `devtools::*` does not. The cell shows the
just-recipe behavior. Bare-devtools behavior is `BUG-A`.

---

## T14 — Implementation order (suggested)

1. **Land T0 quick gates** as a `just check-build-system` aggregator
   (mostly existing recipes). One-day task.
2. **Land T5 latch-leak harness** including the currently-failing T5.6 / T5.7
   as `expect_failure` tests. This makes the bugs visible.
3. **Land T2 minimal subset** in CI: (M, devtools::install) and
   (S, R CMD INSTALL <tarball>) — covers the two most common end-user flows.
4. **Land T7 vendor consistency** as `expect_failure` for T7.2 and T7.3.
   These become the regression tests for §7.6 and §7.7 fixes.
5. **Land T6 wrapper-skip** alongside the §6.4 patch.
6. **Expand to full T11** parity matrix once the regression nets are in place.
7. **Add T8 CRAN simulation** to a separate CI job (needs network
   isolation).
8. **T9, T10** at low priority — useful but not load-bearing.

---

## T15 — What we cannot test

- **Real CRAN submission**. Best simulation is T8 plus `r-cmd-check` in CI
  with `--as-cran`.
- **All Linux distros**. CI matrix is finite. The `R CMD check --as-cran`
  WARNING for `compilation flags used` (§Common issues in CLAUDE.md) is
  glibc-version-sensitive.
- **Windows-specific path handling**. The `cygpath -m` conversion and
  `.dll.a` removal happen on Windows. CI has Windows runners; cover the
  install-path tests there.
- **Long-tail installers** like `BiocManager::install`. Not in scope.

---

## Glossary recap

- `T+o` requires `inst/vendor.tar.xz` to be inside the artifact being
  installed (a tarball or a path).
- `T+a` requires cargo-revendor on PATH at install time (auto-vendor).
- `S+m` requires the install path to be inside the monorepo, with
  `miniextendr-api/Cargo.toml` at ≤5 levels up.
- `S+n` requires either network access OR cargo-registry warm cache.

Read these as mode-determination axioms; the test plan should refuse to
fabricate state that contradicts them.
