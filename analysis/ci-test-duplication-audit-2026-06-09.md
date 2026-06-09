# CI ↔ test-suite duplication audit (#805)

Date: 2026-06-09
Scope: every job / `run:` block across `.github/workflows/{ci,webr,pages,mirror-webr}.yml`,
the composite action `.github/actions/setup-vendor/action.yml`, and the standalone
`tests/*.sh` harnesses. Classify what each *asserts* and whether the assertion lives
in (or belongs in) a test the suite runs, vs. genuinely infra-level CI logic.

## Principle

CI should *run* the test suites, not carry its own assertion logic. When a CI job
reimplements a check a test already performs (or could), the assertion lives in two
places and drifts. The two acceptable end states per job:

- **(a) Fold into suite** — the assertion moves into a test the suite runs, and CI
  just invokes the suite (`just <recipe>` / `testthat::test_local()` / the bash
  harness behind a `just` recipe).
- **(b) Keep at CI level** — the check is genuinely infrastructure-level (toolchain
  bring-up, packaging, network bootstrap, image-digest mirroring) and stays in CI
  with a one-line justification.

## Cross-cutting finding (the headline)

The motivating duplication from #805 — `tests/standalone-build-roundtrip.sh` + a
bespoke `Standalone Build Round-Trip` CI job — **was already resolved by PR #803**.
That PR deleted the script, the `just test-standalone-build` recipe, and the dedicated
job, and folded the round-trip into a gated `test_that()` in
`minirextendr/tests/testthat/test-templates.R:521`
("standalone scaffolding builds in tarball mode and exposes functions"), sitting
next to the monorepo round-trips. The optional additive test contemplated in this
ticket therefore **already exists** — no new test was added by this audit.

What remains is the *real* cross-cutting gap, and it is a **coverage hole, not a
duplication**: the heavy end-to-end round-trips in `test-templates.R` are gated by
`skip_e2e()` (`test-templates.R:70`), which calls `skip_on_ci()` **first**:

```r
skip_e2e <- function() {
  skip_on_ci()                                   # <- always skips under CI
  skip_if_not(nzchar(Sys.which("autoconf")), ...)
  skip_if_not(nzchar(Sys.which("cargo")), ...)
  skip_if_not(nzchar(Sys.which("R")), ...)
  skip_if_no_local_repo()
}
```

The per-PR `minirextendr` job (`ci.yml:975`) **does** export
`MINIEXTENDR_LOCAL_PATH: ${{ github.workspace }}` (`ci.yml:1017`) — so
`skip_if_no_local_repo()` would pass — but because `skip_on_ci()` fires unconditionally
under `CI=true`, every `skip_e2e()` test (the monorepo round-trip at line 455 and the
standalone round-trip at line 521, plus `test-scaffold-smoke.R`'s `skip_on_ci()`
tests) **never executes in any CI job**. The coverage exists in-suite but is dark in
CI. That is precisely the condition that "tempted a bespoke job" in the first place.

**The fix is not to fork logic into shell** — it is to give a *scheduled* (not per-PR)
CI job what the suite needs and let it run the suite: a checkout (already present),
`MINIEXTENDR_LOCAL_PATH` (already present in the per-PR job), `cargo` + `autoconf` +
`cargo-revendor` on PATH, and a way to bypass `skip_on_ci()` for that run. There is no
such scheduled job today (`ci.yml`'s only `schedule:` consumers are `r-check-macos`
and the heap-check/Valgrind steps inside `r-tests`; neither runs the minirextendr e2e
suite). This is what #775 tracks; the concrete step is in the table below.

## Audit table

Legend — **Action**: `FOLD` = move/keep assertion in a suite test, CI invokes the
suite; `RUN-SUITE` = CI already correctly just runs a suite/recipe (no bespoke
assertion); `INFRA` = genuinely CI-level, keep as-is.

### `.github/workflows/ci.yml`

| CI job / step | What it asserts | Where the assertion lives | Action |
|---|---|---|---|
| `generated-files-check` | `rpkg/src/Makevars`, `.cargo/config.toml`, `target/`, `vendor/` are NOT tracked in git | Bespoke inline `git ls-files` loop | **INFRA** — VCS-hygiene gate on the *repo*, not package behaviour; testthat can't see git staging. Keep. (Low-value fold candidate at most; not worth shell→R churn.) |
| `version-check` | `Cargo.toml [workspace.package].version` base == `rpkg/DESCRIPTION` Version | Bespoke inline `sed`/`grep` | **INFRA-ish** → see follow-up. A cheap, deterministic, network-free invariant; reasonable to mirror as a `minirextendr`/build test (`expect_equal` on the two parsed versions) so a local `test_local()` catches drift before push. Filed as a fold candidate (low priority). |
| `changes` (paths-filter) | which paths changed (gates downstream jobs) | dorny/paths-filter config | **INFRA** — pure CI gating, no assertion. Keep. |
| `sync-checks` → `just wrappers-sync-check` / `vendor-sync-check` / `templates-check` / `templates-recipes-check` / `lint-sync-check` | generated R wrappers/NAMESPACE/man, vendored copies, templates delta, template justfile recipes, lint parser are all in sync | `just` recipes (each is the suite/tool for that invariant) | **RUN-SUITE** — already correct: CI invokes recipes, no inline grep. Keep. |
| `rust-lint` → `just fmt-check`, `cargo clippy …`, `cargo doc …` | rustfmt clean; clippy `-D warnings` (default + curated all-features + cargo-revendor); rustdoc `-D warnings` (cargo-revendor) | cargo/just toolchain (clippy *is* the suite) | **RUN-SUITE** — lint *is* the assertion engine; the `Fail if any Rust check failed` step is a result-aggregator, not a re-implemented check. Keep. (Curated feature-list duplication vs `clippy_all` is a separate known item, not a test-duplication.) |
| `r-check-linux` → `R CMD check --as-cran` | package builds, installs, passes R CMD check incl. its bundled `tests/` (testthat) on release/devel/oldrel-1 | `r-lib/actions/check-r-package` runs the package's own test suite | **RUN-SUITE** — canonical. The `Print failed tests` / `Print 00check.log` steps are diagnostics, not assertions. Keep. |
| `r-check-macos` | same as Linux, on macOS arm64/x86_64; SDK pin + CRAN syslibs | `check-r-package`; SDK/syslib steps are toolchain bring-up | **INFRA + RUN-SUITE** — the SDK pin / `/opt/R` syslib download are genuinely infra (toolchain ABI parity, #95). The check itself runs the suite. Keep. |
| `r-check-windows` (commented out) | — | — | **DISABLED** — n/a. |
| `r-tests` → `Install (source mode)` + `testthat::test_local()` | source-mode install works; full rpkg testthat suite passes | testthat suite | **RUN-SUITE** — correct. Keep. |
| `r-tests` → heap-check (3 randomized rounds, `MALLOC_CHECK_=3`) | rpkg suite passes under glibc heap-corruption detection | testthat suite, *run under a hardened allocator env* | **INFRA-wrapper over suite** — the *assertion* is the suite; the env (`MALLOC_CHECK_`/`MALLOC_PERTURB_`) is an infra-level GC-discipline guard that can't live in a portable testthat test. Keep. |
| `r-tests` → Valgrind ALTREP pass | ALTREP materialization has no invalid reads/writes/uninit | testthat filter `altrep-materialization` run under valgrind | **INFRA-wrapper over suite** — same shape: suite is the assertion, valgrind is the harness. The `grep -A5 "ERROR SUMMARY…"` is summary extraction, not a re-implemented check (valgrind's `--error-exitcode=1` is the real gate). Keep. |
| `cross-package-tests` → `just build-all && just test-all` | producer.pkg/consumer.pkg trait-ABI round-trip builds + tests pass | cross-package testthat suites behind `just test-all` | **RUN-SUITE** — correct. Keep. |
| `minirextendr` → `testthat::test_local()` (with `MINIEXTENDR_LOCAL_PATH` set) | minirextendr unit suite passes | testthat suite | **RUN-SUITE** for the unit suite — **but** `skip_on_ci()` darkens every `skip_e2e()`/`skip_on_ci()` test here (the round-trips at `test-templates.R:455`,`:521` and `test-scaffold-smoke.R`). See cross-cutting finding. **FOLD/RUN-SUITE gap → #775**. |
| `minirextendr` → `R CMD check --as-cran` | minirextendr package itself passes CRAN check | check-r-package (runs the package's tests on the *tarball*, where the relative-path repo probe fails and `MINIEXTENDR_LOCAL_PATH` is unset → e2e skip) | **RUN-SUITE** — correct for what it can see; reinforces why the e2e tests need a *separate* env-primed run (the tarball-check path structurally cannot run them). Keep; gap tracked by #775. |
| `cran-check` → `R CMD build` + `R CMD check --as-cran` from tarball | offline/tarball-mode install + CRAN check passes (vendor latch path) | check on built tarball, runs bundled tests | **RUN-SUITE** — correct. The `rm -rf rpkg/vendor` + `Build source tarball` are packaging steps (infra), the check runs the suite. Keep. |
| `bootstrap-vendor-test` → `just test-bootstrap-vendor` | (1) `bootstrap.R` regenerates `inst/vendor.tar.xz` from a clean tree (#441); (2) `just vendor` fails loudly on a git-vendored framework crate (#876); (3) cross-surface feature rename vendors from local workspace, lock stays tarball-shape (#883) | `tests/bootstrap-produces-vendor.sh`, `vendor-loud-fail.sh`, `vendor-cross-surface-rename.sh` behind the `just` recipe | **INFRA (keep as shell behind recipe)** — these drive `bootstrap.R` + `R CMD build` + `cargo-revendor` + raw `git`/`tar`/`Cargo.lock` inspection across the *build/packaging* boundary. They are not R-package behaviour and have no natural home in a testthat suite (no R session owns the assertion). CI correctly invokes them via one `just` recipe, not inline grep. Keep. |
| `ci-success` | all gating jobs succeeded/skipped | result aggregator | **INFRA** — branch-protection summary, no assertion. Keep. |

### `.github/workflows/webr.yml`

| CI job / step | What it asserts | Where the assertion lives | Action |
|---|---|---|---|
| `cargo-check` (tier 1) | `miniextendr-api` + cross-package crates compile on `wasm32-unknown-emscripten` | `cargo check --target wasm32-…` | **INFRA/RUN-SUITE** — compile-only guard for a target no local dev/CI suite exercises; cargo *is* the check. Keep. |
| `webr-install` Phase 1 → native install | native install regenerates `wasm_registry.rs` via the cdylib pass | `R CMD INSTALL` + `grep content-hash … != 0000…` | **INFRA** — toolchain/codegen bring-up; the `content-hash != all-zero` grep is a build-artifact sanity check (the cdylib pass either ran or it didn't), not a package-behaviour assertion. Keep. |
| `webr-install` Phase 2 → wasm install | rpkg installs on wasm32 under emcc side-module RUSTFLAGS; output is wasm not ELF (`file … | grep WebAssembly`) | `R CMD INSTALL` + `file` probe | **INFRA** — empirical validator for emcc link flags (#494/#745); no R session, no portable test home. Keep. |
| `webr-install` Tier 3 → Node + webR smoke (`node smoke.mjs`) | `library(miniextendr)` boots and runs inside a webR Node runtime | `tests/webr-node-smoke/smoke.mjs` | **INFRA/RUN-SUITE** — runtime smoke in a JS/wasm sandbox that no R/Rust suite can host; CI invokes the smoke driver, doesn't inline the assertion. Keep. (Mirrors `tests/webr-smoke.sh`, the local-dev counterpart behind `just docker-webr-build`.) |

### `.github/workflows/pages.yml`

| CI job / step | What it asserts | Where | Action |
|---|---|---|---|
| rustdoc builds (`cargo doc … -D warnings`), `just vendor`, `docs-to-site.sh`, `zola build`, roadmap data | docs/rustdoc/site build cleanly and deploy | cargo/zola/scripts | **INFRA** — pure build+deploy pipeline, no test assertion to fold. Keep. |

### `.github/workflows/mirror-webr.yml`

| CI job / step | What it asserts | Where | Action |
|---|---|---|---|
| `Read pinned digest` + skopeo copy | `WEBR_BASE` digest is extractable from `Dockerfile.webr`; mirror stays digest-aligned with upstream | inline `grep`/`sed` on Dockerfile + skopeo | **INFRA** — image-mirroring plumbing (#496); nothing a test suite could or should own. Keep. |

## Summary

- **Jobs / discrete `run:`-assertion blocks audited:** ~30 across 4 workflows + 1 composite action + 3 shell harnesses.
- **Genuinely infra-level (keep as-is, justified above):** the large majority —
  packaging (`cran-check` build steps), toolchain bring-up (macOS SDK/syslibs, wasm
  targets), allocator/valgrind harnesses wrapping the suite, VCS hygiene
  (`generated-files-check`), image mirroring, and the three vendor/bootstrap shell
  regressions behind `just test-bootstrap-vendor` (build-boundary, no R-session home).
- **Already correctly RUN-SUITE (no bespoke assertion):** all R CMD check jobs,
  `r-tests`, `cross-package-tests`, `sync-checks`, `rust-lint`.
- **Real action items (2):**
  1. **Coverage gap, not duplication (#775):** the `skip_e2e()` round-trips in
     `minirextendr/tests/testthat/test-templates.R` (monorepo `:455`, standalone
     `:521`) — and the `skip_on_ci()` tests in `test-scaffold-smoke.R` — never run in
     CI because `skip_e2e()` calls `skip_on_ci()` first. **Concrete step:** add a
     *scheduled* CI job (modeled on `bootstrap-vendor-test`: checkout + `just` + Rust
     toolchain + `cargo install --path cargo-revendor`) that exports
     `MINIEXTENDR_LOCAL_PATH=${{ github.workspace }}` **and** disables the on-CI skip
     for that run (cleanest: a `MINIEXTENDR_RUN_E2E=1` env probe that `skip_e2e()`
     honours to *not* call `skip_on_ci()`, instead of unsetting `CI`), then runs
     `Rscript -e 'testthat::test_local()'` in `minirextendr/`. This runs the existing
     in-suite round-trips rather than reviving `standalone-build-roundtrip.sh`. The
     standalone `test_that()` already exists (#803), so no new test is needed —
     only the scheduled runner + skip-gate honoring. Tracked by **#775** (and the
     #803 PR notes already point #775/#805 at this).
  2. **Low-priority fold candidate:** `version-check`'s `Cargo.toml`↔`DESCRIPTION`
     version-equality is a cheap, deterministic invariant that could be mirrored as a
     `test_that()` so `test_local()` catches drift pre-push (CI keeps the gate as a
     fast-fail). Filed as a focused follow-up; not worth doing inline in this audit
     PR. Everything else is infra-level and stays as-is.

No CI jobs or shell scripts were removed or restructured by this audit. The
deliverable is this document plus the tracked follow-ups.
