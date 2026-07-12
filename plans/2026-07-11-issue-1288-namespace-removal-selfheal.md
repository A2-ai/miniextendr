# Plan: #1288 — dev-loop NAMESPACE removal/rename self-heal (mirror of #860)

Date: 2026-07-11. Anchors verified against main @ 372841eccc44aed3a6aa9fe62886e84ffd72b0cd.
Branch: `fix/1288-namespace-removal-selfheal`.

The additive case (#860, PR #899) self-heals via the Step-3 install →
Step-4 `document()` → Step-5 conditional-reinstall dance. The removal/rename
case dies at Step 3: the on-disk NAMESPACE is a **superset** of the freshly
regenerated wrappers, and `R CMD INSTALL`'s test-load aborts with
`undefined exports: <old_name>` before `document()` can drop the stale entry.

## Issue verdicts (independently verified)

- Line anchors `:189-190` / `:202` / `:214-222` — **confirmed exactly** on
  this checkout (see Verified anchors).
- Failure mechanics — **confirmed, with one nuance**: `install_pkg()` uses
  `devtools::install(build = TRUE)` (tarball path). The tarball ships the
  *stale* wrappers; it is the `MINIEXTENDR_FORCE_WRAPPER_GEN=1` regen
  (workflow.R:248-249) in the unpacked staging tree that drops `old_name`
  from the collated code while NAMESPACE (also from the tarball) still
  exports it. Test-load then hard-errors (`namespaceExport`,
  `background/r-svn/src/library/base/R/namespace.R:1240`; test-load hook
  `.../tools/R/install.R:1890-1896`) and the install is rolled back.
  Empirically reproduced with an R-only probe package (superset NAMESPACE):
  `R CMD INSTALL` aborts at "testing if installed package can be loaded from
  temporary location" with `undefined exports: bar`; `--no-test-load`
  installs it; byte-compile/lazy-load prep passes (partial load skips
  exports), so test-load is the *only* tripwire.
- Wrinkle 1 (pkgload warn-only) — **confirmed empirically and in source**:
  `pkgload:::setup_ns_exports` emits `cli_warn("Objects listed as exports,
  but not present in namespace: ...")`, intersects, and proceeds. So
  `document()` / `load_all()` / `testthat` survive the superset.
- Wrinkle 2 (background job shows outer exit 0 while inner step exited 1) —
  **refuted as a minirextendr defect**: `install_pkg()`'s
  `tryCatch → cli_abort` chain propagates, `miniextendr_build()` errors,
  `Rscript` exits non-zero. The masking is in the reporter's job harness,
  not this repo. No code change; state this in the PR body.
- PR #899's claim that a "wrapper-gen → document → install" reorder is
  unreachable without `just` — **refuted**: `devtools::document()` →
  roxygen2 `load_pkgload` → `pkgload::load_all(compile = NA)` →
  `pkgbuild::compile_dll()` → `install_min(components = "libs",
  "--no-test-load")` runs the Makevars `all: $(SHLIB) $(WRAPPERS_R)` target
  **in-place in the source tree** (and `pkgbuild:::sources()` globs `.rs` +
  `Cargo.toml`, so Rust edits trigger it). This is in fact the mechanism
  that makes Step 4 self-healing today — the Step-3 tarball install never
  touches the source tree's wrappers.R. We do NOT adopt the reorder (see
  rejected alternatives), but the plan relies on this verified mechanism:
  after a rename, Step 4 alone regenerates source wrappers.R *and* rewrites
  NAMESPACE correctly, tolerating the superset with a warning.

## Verified anchors

All in `minirextendr/R/workflow.R` unless noted:

- Step 3 install: `:189` (`cli_h2`), `:190` (`install_pkg(pkg_path)`),
  `:191` (success alert). Guarded by `has_devtools` (`:172`).
- Step 4: `:195` header; `namespace_before` `:199-200`;
  `devtools::document(pkg_path)` `:202`; `namespace_after` `:205`.
- Step 5 (the #860 fix): condition + block `:214-222` — fires only on
  `!identical(namespace_before, namespace_after)`.
- `install_pkg()` `:247-269`; FORCE env `:248-254`; `tryCatch → cli_abort`
  `:255-268`; `reload = FALSE` (#1000) `:261`.
- `namespace_digest()` `:275-280`; `wrappers_file_exists()` `:291-297`.
- `bootstrap_fresh_wrappers()` `:320-391`: first install `:348-359`
  (`cli_abort("Bootstrap install failed")`), wrappers-exist gate `:361-369`,
  `document()` `:374`, second install `:375-387`.
- `miniextendr_sync()` (`minirextendr/R/sync.R:149,182-187`) delegates to
  `miniextendr_build()` — covered for free.
- e2e pattern to mirror: additive single-pass test
  `minirextendr/tests/testthat/test-templates.R:637-706` (#898);
  `skip_e2e()` `:82-93`. Mock pattern to mirror:
  `test-workflow-bootstrap.R:67-130` (`local_mocked_bindings` on
  `devtools::install`/`document` + internal helpers; `make_pkg_root()` `:4`).
- No template copy: `workflow.R` is not in `minirextendr/inst/templates/`
  (only comment mentions of `miniextendr_build()` there) — **no
  templates-approve/patch churn**.
- No open PR touches `minirextendr/R/workflow.R` (checked #1281–#1287).

## Design (all decisions resolved)

**Chosen: defer-and-retry (hardened form of the issue's fix 1).** Step 3's
install failure is caught and deferred with a loud warning; Step 4 runs
(it reconciles wrappers + NAMESPACE, tolerating the superset per pkgload);
Step 5 becomes **mandatory whenever Step 3 was deferred** (not just on
NAMESPACE change). Step 5 keeps its hard abort — so a build never reports
success without a final successful, test-loaded install. Invariant to state
in the roxygen and PR: *`miniextendr_build(install = TRUE)` returns `TRUE`
only if the last install attempt succeeded with test-load.*

- **No error classification.** Do NOT grep the condition message for
  "undefined exports" — the callr/devtools message shape is version-fragile.
  Deferring *any* Step-3 failure is safe because: a real compile error
  re-fails at Step 4 (`compile_dll` runs the same cargo build) or at the
  mandatory Step 5, both loudly; the only cost is one wasted document
  attempt on a genuinely broken build.
- Step 3 shape (replaces `:190-191`):
  ```r
  step3_error <- NULL
  tryCatch(
    {
      install_pkg(pkg_path)
      cli::cli_alert_success("Installed package")
    },
    error = function(e) step3_error <<- e
  )
  if (!is.null(step3_error)) {
    cli::cli_warn(c(
      "Install failed before NAMESPACE reconciliation; deferring.",
      "i" = "Expected when an exported #[miniextendr] function was removed or renamed: the stale NAMESPACE still lists the old export and R CMD INSTALL's load test aborts with 'undefined exports' (#1288).",
      "i" = "Continuing to the document step, then retrying the install once.",
      "x" = conditionMessage(step3_error)
    ))
  }
  ```
  (`step3_error` must be initialised before the `if (install)` block so the
  Step-5 condition can read it when `install = FALSE`/no-devtools paths run.)
- Step 5 condition (`:214`) becomes:
  `install && has_devtools && (!identical(namespace_before, namespace_after) || !is.null(step3_error))`
  with the alert message differentiating the retry case ("retrying install
  after NAMESPACE reconciliation" vs the existing exports-changed text).
  `install_pkg()` itself is **unchanged** (both call sites keep it); a
  Step-5 failure aborts with the real error as today.
- If Step 4's `document()` errors after a deferred Step 3, let it propagate
  (no condition chaining) — the Step-3 warning is already on screen and the
  previously installed image is intact (test-load failure rolls back).
- **Bootstrap path gets the same class of fix** (reachable when a mature
  package's wrappers.R was deleted while NAMESPACE retains exports of a
  since-removed/renamed fn): in `bootstrap_fresh_wrappers()`, the FIRST
  install (`:348-359`) defers its error; discriminator is the existing
  wrappers gate — `** libs` (which writes wrappers in-place) runs before
  test-load, so after a test-load-only failure `wrappers_file_exists()` is
  TRUE (defer + warn, `document()` + second install heal it); if wrappers
  are still absent the failure was real → re-raise
  `cli_abort("Bootstrap install failed", ...)` with the captured message.
  The second bootstrap install (`:375-387`) keeps its hard abort (NAMESPACE
  is fresh by then). Fresh scaffolds are unaffected
  (`mx_minimal_namespace()` exports nothing, no superset possible).
- Roxygen: extend `miniextendr_build`'s "Why a conditional reinstall"
  section (`:92-108`) with the removal/rename case + the forced-retry rule;
  regenerate `man/miniextendr_build.Rd`.

**Rejected alternatives:**

- *Issue fix 2 — pre-reset NAMESPACE to header + useDynLib before Step 3*:
  destructive mid-build mutation (an interrupt strands a gutted NAMESPACE);
  either forces a second install on every build (digest comparison becomes
  reset-vs-full) or needs a third snapshot; validates nothing at Step 3.
- *Issue fix 3 — document() before the first install*: mechanically works
  (verified above) and would give single-install builds, but silently keys
  the NAMESPACE's correctness on roxygen2's load strategy staying
  `load_pkgload` and on `pkgbuild:::needs_compile` mtime heuristics — a
  `Roxygen: list(load = "source")` package or an mtime miss yields a stale
  NAMESPACE with no downstream self-check. Restructuring the
  battle-tested #899/#1000/#1003 flow is not warranted by this bug.
- *`--no-test-load` on Step 3 only*: smallest diff, but drops the stock
  validation from the final install of every no-change build and can leave
  a broken-superset image installed if Step 4 then fails.
- *Grep-scoped tolerance ("undefined exports" only)*: message shape is not
  stable across devtools/callr versions; unconditional defer + mandatory
  retry gives the same safety without the parser.

## Work items (flat order)

1. `minirextendr/R/workflow.R`: Step-3 defer + warning; Step-5 condition
   `|| !is.null(step3_error)` + retry-flavoured alert; roxygen section
   update. Regenerate `man/miniextendr_build.Rd`
   (`just minirextendr-document`).
2. `bootstrap_fresh_wrappers()`: defer first-install error iff
   `wrappers_file_exists()` post-failure; re-raise otherwise (keep the
   existing abort wording, include the captured message).
3. New `minirextendr/tests/testthat/test-workflow-selfheal.R` (mock-based,
   runs in the normal suite — mirror `test-workflow-bootstrap.R:67`'s
   `local_mocked_bindings` pattern; mock `devtools::install` /
   `devtools::document` / `miniextendr_autoconf` / `miniextendr_configure` /
   `wrappers_file_exists`; `make_pkg_root()`-style tmp package):
   a. install fails once then succeeds, `document` leaves NAMESPACE
      **unchanged** → expect warning, expect install called 2×, returns
      TRUE (pins the forced-retry: digest alone would skip Step 5);
   b. install always fails → `expect_error`, install called exactly 2×
      (bounded — no loop);
   c. bootstrap: first install fails with wrappers present → heals
      (document + second install run); first install fails with wrappers
      absent → aborts with "Bootstrap install failed".
   These are behavioural-through-mocks, NOT `deparse(body)` greps (brittle
   per project memory).
4. e2e removal-case regression (skip_e2e-gated), sibling of the #898 test
   in `test-templates.R`: scaffold monorepo (`sprename`), `cargo vendor`,
   `miniextendr_build()` once; then rename via exact-string
   `sub("pub fn add(", "pub fn add_renamed(", fixed = TRUE)` in
   `src/rust/lib.rs` (NOT a regex over `add` — substring-rename gotcha);
   `miniextendr_build()` once more; assert `getNamespaceExports()` contains
   `add_renamed`, lacks `add`, and `add_renamed(2, 3) == 5`; also assert the
   deferral warning fired (wrap in `expect_warning`/`withCallingHandlers`).
5. Incidental docs drift (fix-what-you-see): `docs/MINIREXTENDR.md:106` and
   `:475` reference a nonexistent `miniextendr_document()` — replace the
   `:106` line with `devtools::document(...)` (per the file's own
   "Recommended" section) and drop it from the `:475` table.
6. Follow-up issue (concessions rule — implementer files it, references it
   in the PR body): "minirextendr_doctor(): detect NAMESPACE exports absent
   from R sources/generated wrappers (stale-export drift)" — covers the
   residual wrinkle-1 risk for users who only ever run
   `load_all()`/`testthat` (warn-only) and never `miniextendr_build()`.
   Wrinkle 2 is refuted (external harness), documented in the PR body, no
   issue.

## Exact commands (worktree)

```bash
rig default 4.6 && R --version | head -1
just worktree-sync                               # FIRST — fresh worktree rv/library
# unit loop (fast, no compile):
just minirextendr-test workflow-selfheal 2>&1 > /tmp/1288-selfheal.log
grep -E '\[ FAIL [0-9]+' /tmp/1288-selfheal.log  # devtools::test always exits 0
just minirextendr-document                        # regen man/miniextendr_build.Rd
# full suite — NOTE: e2e tests are NOT skipped locally (skip_on_ci is a no-op);
# they scaffold + cargo-compile → run with dangerouslyDisableSandbox, budget ~15-30 min:
just minirextendr-test 2>&1 > /tmp/1288-minir.log
grep -E '\[ FAIL [0-9]+' /tmp/1288-minir.log
# templates untouched, but cheap to prove:
just templates-check
```

No Rust changes → no clippy/fmt legs needed. If the full local e2e run is
infeasible (cold cargo cache in the worktree), run at minimum the new
removal-case e2e by filter (`just minirextendr-test templates` picks up
test-templates.R) and say in the PR which e2e legs ran where; CI's
label-gated `r-roundtrip-e2e` job (MINIEXTENDR_RUN_E2E=1) is the backstop.

(Planning note: the failure mechanics were verified live with an R-only
probe package — superset NAMESPACE → test-load abort / pkgload warn /
`--no-test-load` install — but the full Rust rename e2e exceeded the 15-min
planning budget and is left to work item 4.)

## Must NOT touch

- `install_pkg()` signature/behaviour — both call sites keep it; the defer
  lives at the Step-3 call site only.
- Step 5's and bootstrap-install-#2's hard aborts (the fail-loudly
  backstop).
- The `reload = FALSE` lines (#1000) and `MINIEXTENDR_FORCE_WRAPPER_GEN`
  plumbing (#757/#963/#911) in both installers.
- `namespace_digest()` semantics (NA-for-missing drives first-build
  reinstall).
- The existing #898/#911 e2e tests and `minirextendr/inst/templates/`
  (workflow.R is not templated).

## Done criteria

- Rename/removal of a `#[miniextendr]` export heals in a SINGLE
  `miniextendr_build()` pass (e2e work item 4 green); additive #898 e2e
  unchanged-green; unit tests (item 3) green in `just minirextendr-test`
  (`[ FAIL 0`); a genuinely failing install still fails the build loudly
  (test 3b); `man/miniextendr_build.Rd` regenerated; docs drift fixed;
  follow-up doctor issue filed and referenced; `just templates-check`
  clean; PR body carries the wrinkle-2 refutation and `Fixes #1288`.

## Escalation rule

If reality diverges from this plan — the mock-based tests can't drive
`miniextendr_build()` without real autoconf/configure (with_project/proj
machinery resists mocking), the e2e removal test still fails at Step 5's
test-load after `document()` (i.e. the source-tree wrapper regen via
`compile_dll` does not happen as verified here), the Step-3 defer trips
other suites' expectations on `miniextendr_build()` warnings, or an
open PR has started touching `minirextendr/R/workflow.R` — **stop, commit
nothing further, and report back. Do not improvise.**

---

## Amendment (2026-07-11, post-escalation)

The implementer hit the named escalation trigger: the e2e rename test fails
identically at Step 3 AND the Step-5 retry (`undefined exports: add`), with
`document()` leaving NAMESPACE unchanged. Root-caused; work items 1/2/3/5
stand as implemented (10/10 unit passes, full suite `[ FAIL 0` pre-item-4).

### Root cause (with evidence)

The original plan's claim "Step 4 alone regenerates source wrappers.R and
rewrites NAMESPACE" is **wrong for any package that has been through one
`build = TRUE` install in the same `miniextendr_build()` call**:

- Step 3's `devtools::install(build = TRUE)` → `pkgbuild::build()` runs the
  scaffolded `bootstrap.R` **in the source tree**, which seals
  `inst/vendor.tar.xz` there **by design** (bootstrap.R's own comment: the
  vendor step "leaves inst/vendor.tar.xz in the (git-tracked) source dir on
  purpose"; its only guard is `!file.exists("inst/vendor.tar.xz")` +
  cargo-revendor on PATH — **no `.git` check**; the `.git`-walk guard
  belongs to configure's auto-vendor, which is exactly why bootstrap.R
  exists). Diag log `/tmp/1288-diag.log:438-486`: `Running bootstrap.R...`
  → `bootstrap.R: generating inst/vendor.tar.xz via cargo-revendor` with
  paths inside the source scaffold.
- `miniextendr_build()` already knows (the snapshot/restore trap,
  workflow.R `:166-186` in the implementer's tree) — but the restore is
  `on.exit`, i.e. **function exit**. During Steps 4/5 the source tree is
  latched: Step 4's `compile_dll` re-runs `./configure` → `install mode =
  tarball install` (log `:596`) → Makevars #1022 guard → `tarball install:
  using pre-shipped R/<pkg>-wrappers.R (skipping wrapper-gen, #1022)`
  (log `:642`) because `MINIEXTENDR_FORCE_WRAPPER_GEN` is scoped to
  `install_pkg()` only. Wrappers stale → NAMESPACE unreconciled (log
  `:647-655`: still `export(add)`) → mandatory Step-5 retry re-fails.
- `--freeze` also rewrites `src/rust/Cargo.toml` (path-dep siblings →
  `vendor/`) and tarball-mode `.cargo/config.toml` points at vendored
  sources, so the mid-build window compiles against a vendor snapshot —
  wrong beyond NAMESPACE (arbitrarily stale if the latch was pre-existing).

**Two-bug verdict.** This is a distinct, pre-existing defect — filed as
**#1294** — that breaks the #860 self-heal for EVERY post-first-build
export change (additive too: second-build add → Step 4 skips regen →
NAMESPACE unchanged → Step 5 skipped → unexported install, the exact #860
symptom back). The additive #898 e2e only passes because its export is
added *before the first build*, so `bootstrap_fresh_wrappers()`
(`build = FALSE`, in-place, latch-free source mode) finalises everything
before Step 3; Steps 4/5 are fixpoints there. Both bugs must be fixed in
THIS PR (the #1288 e2e cannot pass otherwise); PR body carries
`Fixes #1288` and `Fixes #1294`.

### Amended design (decision)

**Restore the dev source tree immediately after Step 3, not only on exit.**

- New local closure inside `miniextendr_build()` (e.g.
  `restore_dev_tree()`), wrapping the existing restore body (`:180-184`):
  write back `snap_manifest`/`snap_lock`, delete `vendor_tarball` iff
  `!tarball_preexisting && fs::file_exists(...)`. `on.exit` calls the same
  closure (unchanged backstop; idempotent).
- Call it **unconditionally right after the `if (install) { ... }` block
  closes** (before the Step-4 header) — a no-op when nothing was sealed
  (`install = FALSE`, no devtools, bootstrap-only). With the latch gone,
  Step 4's `compile_dll` configure re-resolves **source mode**, rewrites
  `.cargo/config.toml` itself, and wrapper regen runs unconditionally (the
  #1022 skip is tarball-only) against the un-frozen manifest. Step 5's
  install then re-runs bootstrap.R, which re-vendors (one extra
  cargo-revendor per export-changing build, seconds — log shows 10 crates
  / 1.2 MB) and reseals for the staging tarball; `on.exit` cleans at exit.
- **Do NOT touch `vendor/` or `.cargo/` in the mid-build restore**:
  configure owns `.cargo/config.toml` (bootstrap's pre-vendor configure
  left it source-mode anyway, log `:441`), and `vendor/` may be
  user-provisioned (the e2e's manual `cargo vendor` for offline crates.io
  deps depends on it).
- **Never delete a pre-existing tarball mid-build** (unchanged semantics).
  Instead, when `tarball_preexisting` is TRUE at entry, emit a `cli_warn`
  up front: dev-loop self-heal is structurally disabled in a latched tree;
  delete `inst/vendor.tar.xz` (or run `minirextendr_doctor()`, which
  already detects the stale latch) to resume source-mode dev.
- **Rejected: holding `MINIEXTENDR_FORCE_WRAPPER_GEN=1` through Step 4.**
  It would regenerate wrappers (and make the e2e pass) but leaves Step 4
  compiling in tarball mode against the frozen manifest/vendor snapshot —
  content-current for a same-build seal, silently stale for a leaked one —
  and keeps the tree latched for the whole build. Symptom patch, not the
  class fix. Keep FORCE scoped to `install_pkg()` as today.
- **No framework-side change** (bootstrap.R/configure): sealing the source
  tree is by design (the built tarball must carry `inst/vendor.tar.xz`;
  see the #1029 leaked-tarball guard). The defect is `miniextendr_build()`
  not isolating Steps 4/5 from that documented side effect — the trap
  exists, only its granularity was wrong.

### Amended work items (flat; 1/2/3/5 stand as implemented)

7. `workflow.R`: extract the restore body into the local closure; `on.exit`
   delegates to it; explicit call after the `if (install)` block (before
   Step 4). Add the `tarball_preexisting` entry warning.
8. Unit test (append to `test-workflow-selfheal.R`, mock pattern as items
   3a-3c): mocked `devtools::install` **simulates bootstrap.R's side
   effect** (creates `inst/vendor.tar.xz` + mutates `src/rust/Cargo.toml`
   in the tmp pkg root); mocked `devtools::document` asserts, at
   document-time, that the latch is gone and the manifest matches the
   entry snapshot. Second case: pre-seeded `inst/vendor.tar.xz` before the
   call → still present at document-time (not deleted mid-build) and the
   new entry warning fired.
9. e2e (work item 4, unchanged scenario) gains two assertions per build:
   `inst/vendor.tar.xz` absent after `miniextendr_build()` returns, and
   `src/rust/Cargo.toml` byte-identical to its pre-build snapshot. Note in
   the test comment: `add_renamed %in% exports` after build 2 IS the
   additive-second-build regression pin for #1294 — no separate additive
   e2e needed.
10. Roxygen: one short paragraph in the miniextendr_build docs (beside the
    implemented "Removal/rename self-heal" section) documenting the
    mid-build restore + the latched-tree warning; regen
    `man/miniextendr_build.Rd`.
11. PR body: `Fixes #1288` + `Fixes #1294`; keep the wrinkle-2 refutation.

### Amended done criteria

As before, PLUS: the rename e2e passes (single second-build pass exports
`add_renamed`, drops `add`); its latch/manifest assertions pass; unit items
3 + 8 green; full `just minirextendr-test` `[ FAIL 0`; `Fixes #1288` and
`Fixes #1294` in the PR.

### Amended escalation triggers

If reality diverges — the e2e STILL fails after the mid-build restore
(would mean a third latch writer or a non-latch mode flip: capture
configure's `install mode` lines and stop); Step 5's install breaks
because bootstrap.R's re-vendor fails on the restored tree; the restore
demonstrably discards a legitimate same-build `Cargo.lock` update that the
existing exit-time restore did not (behavior must match on.exit semantics,
only earlier); or the entry warning breaks other suites — **stop, commit
nothing further, and report back. Do not improvise.**
