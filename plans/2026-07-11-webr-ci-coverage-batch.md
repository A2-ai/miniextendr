# Plan: webR CI coverage batch — #1270 local parity, #1255 testthat-under-wasm, #1271 monorepo leg

Date: 2026-07-11. Anchors re-verified against main @ 17f634d8. **PR #1272 has
MERGED** — the scaffold leg (incl. `SMOKE_SCAFFOLD_PKG`, the `mxsmoke` rename
workaround at `.github/workflows/webr.yml:364-366`, git-init guard at `:373`)
is now on main; this batch is unblocked.

Branches: PR 1 `ci/1270-webr-smoke-scaffold-phase` · PR 2
`ci/1255-webr-smoke-testthat` · PR 3 `ci/1271-webr-monorepo-wasm-leg`.

Three sibling gaps in webR/wasm coverage, all touching the same three files
(`.github/workflows/webr.yml`, `tests/webr-smoke.sh`,
`tests/webr-node-smoke/smoke.mjs`). Batch as 2–3 PRs in the order below;
each is independently shippable.

**Sequencing / blockers**
- ~~All three build on the scaffold leg from PR #1272 — merge #1272 first.~~
  DONE — #1272 merged 2026-07-11. Order within the batch: PR 1 → PR 2
  (independent code, same files — whichever lands second rebases) → PR 3
  (**extends PR 1's `phase_scaffold`; dispatch only after PR 1 merges**, and
  start by merging `origin/main` into the seeded branch).
- #1273's fix (plans/2026-07-10-issue-1273-c-symbol-crate-prefix.md, step 15)
  will later edit the same webr.yml leg to revert the mxsmoke rename
  workaround. No structural conflict, but whichever lands second rebases.
- The smoke-script header (`tests/webr-smoke.sh:7`) already cites #1255 as
  the tracking issue for the dropped testthat coverage — the load-only gate
  is documented as interim, not contract, so restoring coverage (not
  recording a decision) is the right resolution.

## PR 1 — #1270: `phase_scaffold` in tests/webr-smoke.sh (local parity)

The local docker smoke mirrors CI tier-2/3 step-for-step
(`phase_native_install` at `:195`, `phase_wasm_build` at `:234`,
`phase_webr_session` at `:282`, chained at `:327-329`) but has no scaffold
leg, so a scaffold-leg CI failure can't be reproduced locally without
hand-driving the container.

1. Add `phase_scaffold()` between `phase_wasm_build` and `phase_webr_session`,
   mirroring the four CI steps from #1272's leg: install minirextendr +
   roxygen tooling into a temp lib; `create_miniextendr_package()` to
   `${SMOKE_TMP}/scaffold/mxsmoke` + `use_local_miniextendr()`; native
   install → `roxygen2::roxygenise()` → native reinstall (the scaffolded
   NAMESPACE is a stub until roxygenise, so the first `library()` would load
   no DLL); `CC=emcc` install into the wasm lib.
2. Carry over the two traps recorded in the issue: set `HTTPUserAgent` to the
   binary-serving format so P3M serves binaries inside the container, and
   `git init` the scaffold before configure so a cargo-revendor-on-PATH can
   never flip it into tarball mode (silently building published git sources
   instead of the checkout).
3. Export `SMOKE_SCAFFOLD_PKG=mxsmoke` for the `smoke.mjs` invocation in
   `phase_webr_session` (the runner is env-gated on #1272's branch; unset
   keeps today's behavior).
4. Gate behind `--scaffold` / `WEBR_SCAFFOLD=1` so the default local loop
   stays fast; extend the cleanup function's removal block (`:147-152`) to
   remove the scaffold dir + temp libs.
5. Done: `tests/webr-smoke.sh --scaffold` reproduces the CI scaffold leg
   end-to-end in the container; without the flag, behavior is byte-identical
   to today.

## PR 2 — #1255: opt-in testthat pass in smoke.mjs

Phase-3 unification onto `smoke.mjs` dropped the old testthat-under-wasm run;
nothing exercises the R suite under wasm (single-threaded, no fork, inline
worker) even informationally.

1. Add an env-gated (`SMOKE_TESTTHAT=1`) step to `smoke.mjs`: NODEFS-mount
   `rpkg/tests`, run `testthat::test_local()`, print pass/fail/skip counts.
   **Never fail the gate on test failures** — many tests legitimately fail
   under wasm (worker/fork/threading assumptions); tolerate-and-report is the
   old, intended semantics. Fail only if the harness itself errors before
   producing counts (that's a smoke-infrastructure regression, not a wasm
   incompatibility).
2. Respect `MINIEXTENDR_SKIP_STRESS` conventions: export it in the wasm
   session so the gctorture files don't run under the (much slower)
   interpreter — the stress suite has its own CI job.
3. Enable it by default in `tests/webr-smoke.sh`'s `phase_webr_session`
   (local runs want the information); leave CI tier-3 load-only per-PR.
   Optionally add it to the main-push/dispatch webr.yml variant where wall
   time doesn't gate PRs.
4. Update the stale prose: `tests/webr-smoke.sh` header (`:7`) and the
   corresponding `docs/WEBR.md` claims currently describe the old behavior —
   rewrite to describe the flag-gated reality.
5. Done: `SMOKE_TESTTHAT=1` prints a counts line from inside the webR
   session; a wasm-runtime-behavior regression that still loads fine is now
   *visible* in local smoke output and the opt-in CI variant.

## PR 3 — #1271: monorepo-template wasm coverage

The #1272 leg builds only the standalone template
(`minirextendr/inst/templates/rpkg/`); the monorepo tree
(`templates/monorepo/rpkg/`) carries its own `configure.ac` / `Makevars.in` /
`build.rs` copies whose wasm branches are CI-unbuilt — lockstep is currently
review + `just templates-check` only.

Recommended two-tier shape (the issue's "cheaper interim" plus the full leg,
gated differently):

1. **Per-PR (cheap)**: after scaffolding a monorepo
   (`create_miniextendr_monorepo()`), run `cargo check --target
   wasm32-unknown-emscripten` on the scaffolded rpkg crate. Covers the
   template `build.rs`/cfg-gating drift class without emcc link or R.
2. **Main-push + workflow_dispatch (full)**: extend the scaffold leg with a
   monorepo variant — scaffold, `use_local_miniextendr()` on the rpkg
   subdir, native → roxygenise → native → `CC=emcc` install, and load in
   tier 3 alongside the standalone package. Requires `SMOKE_SCAFFOLD_PKG` to
   accept a list (comma-split in `smoke.mjs`); keep package names distinct
   (`mxsmoke` / `mxmono`).
3. Watch the #1273 interaction: until the symbol-prefix fix lands, the
   monorepo package's stock `add`/`hello` need the same rename workaround as
   `mxsmoke` (three-way collision otherwise); drop both renames when #1273's
   fix merges.
4. Mirror the monorepo variant into `tests/webr-smoke.sh --scaffold` (extends
   PR 1's phase) so local parity holds for both templates.
5. Done: a monorepo-template-only wasm regression turns CI red on main-push
   at the latest, and `cargo check` catches the build.rs/cfg class per-PR.

## Exact commands (all three PRs)

```bash
rig default 4.6 && R --version | head -1
just worktree-sync            # FIRST; then dev installs if the PR needs R pkgs
# These PRs touch shell/YAML/mjs only — no Rust regen loop, no NAMESPACE churn.
bash -n tests/webr-smoke.sh   # syntax gate after editing
node --check tests/webr-node-smoke/smoke.mjs   # PR 2/3 only
# YAML sanity: ensure the workflow parses
ruby -ryaml -e 'YAML.load_file(".github/workflows/webr.yml")' 2>/dev/null \
  || python3 -c 'import yaml,sys; yaml.safe_load(open(".github/workflows/webr.yml"))'
```

Docker-based end-to-end verification of `tests/webr-smoke.sh --scaffold` is
NOT required before the PR (the container run is long); CI's webr workflow is
the authority. State in the PR body which verification was run locally.

## Must NOT touch

- `minirextendr/inst/templates/**` (PR 3 does not edit templates — it builds
  them; template edits belong to a different pipeline with `templates-approve`).
- `patches/templates.patch`.
- The tier-2/tier-3 job structure of webr.yml beyond the steps named here.
- No workflow-permission or credential changes (#747 is separate).

## Escalation rule

If reality diverges from this plan — an anchor doesn't match, the merged
#1272 leg differs from what a step assumes, a step is impossible as written —
**stop, commit nothing further, and report back. Do not improvise.**
