# Plan: #1261 item 3 — migrate `Rf_findVarInFrame` to API-blessed `R_getVarEx`

Date: 2026-07-11. Anchors verified against main @ 17f634d8.
Branch: `fix/1261-findvarinframe-api`.

Scope: one of #1261's four `R CMD check` WARNINGs — `checking compiled code`:
`miniextendr.so` contains a non-API call to `Rf_findVarInFrame` ("this entry
point may be removed soon"). Items 1/2/4 are handled by other PRs
(`plans/2026-07-11-issue-1248-s3-nongeneric-collision.md`,
`plans/2026-07-11-issue-1261-items-2-4-rd-dedup-int-literal.md`). Do NOT
re-fix them here.

**AMENDED 2026-07-11 (maintainer decision review): this PR is the #1261
CLOSER.** The R-floor bump to 4.5 is APPROVED (veto window closed). Beyond
item 3, this PR also carries: (a) the two additional pre-existing doc
WARNINGs inventoried on #1261 (comment 4945939625) — `ImplDotsS3.Rd`
undocumented `seed` argument + undocumented `impl_dots_s3_*` objects — see
new work item 9; (b) the CI `error-on` flip from `'"error"'` to
`'"warning"'` so future WARNING regressions gate PRs — see new work item 10;
(c) `Fixes #1261` in the PR body (supersedes the "partial" instruction in
work item 8).

**Precondition**: dispatch only after the items-2+4 PR (branch
`fix/1261-rd-arg-dedup-int-literal`) and item 1 (PR #1278, merged) are on
main. First step on checkout: `git merge origin/main` (do not rebase), so
the check-log baseline already lacks the other WARNINGs.

## Facts (verified)

- The ONLY compiled reference is `env_binding()` at
  `miniextendr-api/src/externalptr.rs:356` (import at `:173`), used by the
  class-handle unwrapping path (audit A9: Env/R6 `.ptr` /
  `.__enclos_env__`→`private`→`.ptr`, S4 `ptr` slot, `.ptr` attribute).
- Declarations live at `miniextendr-api/src/sys.rs:900-905`: `Rf_findVar`,
  `Rf_findVarInFrame`, `Rf_findVarInFrame3`. `Rf_findVar` and
  `Rf_findVarInFrame3` have **zero callers** anywhere in the repo.
- The blessed replacement per `background/r-svn/doc/manual/R-exts.texi:17653`
  ("Use `R_getVar` or `R_getVarEx`, added in R 4.5.0"):
  `SEXP R_getVarEx(SEXP sym, SEXP rho, Rboolean inherits, SEXP ifnotfound)`
  (`Rinternals.h:539`, impl `envir.c:2287`). With `inherits = FALSE` it calls
  `R_findVarInFrame` internally — exactly the current single-frame semantics.
  **Note the argument order: `(sym, rho, ...)` — REVERSED vs
  `Rf_findVarInFrame(rho, sym)`.**
- Semantics deltas vs the old call, all acceptable here:
  - Raises an R error (longjmp) if `rho` is not an environment — the existing
    `is_environment()` guard at `externalptr.rs:352-354` already prevents
    that path; KEEP it.
  - Raises an R error if the binding is `R_MissingArg` — pathological for
    `.ptr`/`.__enclos_env__`/`private` lookups; the conversion path runs
    under the framework's unwind protection. Accept; note in the rustdoc.
  - Forces promises (evaluates them) instead of returning them raw — strictly
    better for handle unwrapping.
- **R-version floor**: `R_getVarEx` exists only on R >= 4.5.0.
  `rpkg/DESCRIPTION:4` currently says `Depends: R (>= 4.4)`. The webR image
  bundles R 4.6.0 (`Dockerfile.webr:28`), the repo pin is R 4.6, and the
  project is unreleased (no-backcompat principle). **Decision baked into this
  plan: bump the floor to `R (>= 4.5)`** — no shim, no dlsym dodge. (This is
  flagged to the maintainer in the dispatch inventory; if vetoed, this plan
  is re-cut, per the escalation rule below.) The floor line exists ONLY in
  `rpkg/DESCRIPTION` (verified — templates and minirextendr declare none).

## Work items (flat order)

1. `sys.rs`: add the declaration beside the environment-operations group
   (around `:900`), matching the style of its neighbors (doc alias, same
   extern block / `#[r_ffi_checked]` treatment as `Rf_findVar` has today):
   ```rust
   /// Single-frame (`inherits = FALSE`) or inherited variable lookup with an
   /// `ifnotfound` default — the API-blessed replacement for
   /// `Rf_findVarInFrame` (R-exts; added in R 4.5.0, hence
   /// `Depends: R (>= 4.5)`). **Longjmps** if `rho` is not an environment or
   /// the binding is `R_MissingArg`; forces promises.
   #[doc(alias = "getVarEx")]
   pub fn R_getVarEx(sym: SEXP, rho: SEXP, inherits: Rboolean, ifnotfound: SEXP) -> SEXP;
   ```
   Do NOT add `R_getVar`/`R_existsVarInFrame` — unused (simple over complex).
2. `externalptr.rs:356`: replace
   `let val = Rf_findVarInFrame(env, sym);` with
   `let val = R_getVarEx(sym, env, Rboolean::FALSE, R_UnboundValue);`
   (match however `Rboolean` false is spelled elsewhere in the file/crate —
   grep `Rboolean` usage and copy the idiom). Update the import at `:173`
   and the function rustdoc at `:341` (it names `Rf_findVarInFrame`; describe
   the new call + the missing-arg/promise notes from above).
3. `sys.rs`: DELETE the now-unused `Rf_findVar`, `Rf_findVarInFrame`,
   `Rf_findVarInFrame3` declarations (`:900-905`) — project principle:
   remove, don't shim. First grep the whole repo (including `_unchecked`
   suffixed forms) to confirm zero remaining callers; if any exist, stop and
   report per the escalation rule.
4. Update the `R_UnboundValue` doc comment at `sys.rs:289-291` — it cites
   `Rf_findVarInFrame`/`Rf_findVarInFrame3` as its producers; re-word to cite
   `R_getVarEx`'s `ifnotfound` usage.
5. `docs/NONAPI.md:139` lists `Rf_findVarInFrame3` — remove it from that list
   (the declaration is gone) and, if the page has prose about variable
   lookup, mention `R_getVarEx` as the API path.
6. `rpkg/DESCRIPTION:4`: `Depends: R (>= 4.4)` → `Depends: R (>= 4.5)`.
7. Regen + verify (commands below). The class-handle unwrapping tests
   (existing testthat coverage from audit A9 — R6/S4/S7/Env handle args to
   `ExternalPtr<T>` params) are the regression surface; they must stay green.
8. Confirm in the `just r-cmd-check` log: the `checking compiled code`
   WARNING about `Rf_findVarInFrame` is GONE — and, with items 9+10 below
   and the merged prerequisites, the check should report **zero WARNINGs**.
   If any WARNING remains that is not explained by this plan, stop and
   report per the escalation rule. PR body: `Fixes #1261` (this is the
   closer — amended from the original "partial" instruction).
9. **Doc WARNINGs (amendment)**: fix the two pre-existing documentation
   WARNINGs in `rpkg/src/rust/impl_dots_tests.rs`:
   - `ImplDotsS3.Rd`: undocumented `seed` argument. The `@param seed` line
     currently sits ONLY on the struct doc (`impl_dots_tests.rs:44-46`);
     the `#[miniextendr(s3)] impl ImplDotsS3` block (`:53`) has no doc
     comment and `new` (`:55-56`) documents neither param. Mirror the
     doc-comment layout of the `S3NonGenericCollision` fixture in
     `rpkg/src/rust/s3_tests.rs` (the reference S3 fixture pattern): impl
     block gets a description + `@param seed Integer base value.` +
     `@param ... Additional constructor arguments counted by Rust.`, and
     `new`'s doc gets the same `@param` lines.
   - Undocumented `impl_dots_s3_*` objects: give the two methods
     (`impl_dots_s3_ctor_dots` `:63-64`, `impl_dots_s3_add_with_dots`
     `:68-71`) roxygen matching how `s3_tests.rs` methods document theirs
     (real descriptions — never `@noRd`/`@keywords internal` silencers, per
     the "roxygen warnings are bugs to fix" principle).
   - Regen loop, then verify BOTH WARNING lines are gone from the check
     log. If either persists after mirroring the reference pattern, stop
     and report per the escalation rule (the emission path may differ for
     dots-taking methods — that would need a macro-side look, not fixture
     hacks).
10. **CI flip (amendment)**: in `.github/workflows/ci.yml`, change
    `error-on: '"error"'` to `error-on: '"warning"'` at the three ACTIVE
    sites (lines 628, 774, 1421 as of main @ 6de43e9b; line ~930 is inside
    a commented-out block — leave it). Do not touch anything else in the
    workflow. Note: if another PR has moved these lines, locate them by
    grepping `error-on` — three active occurrences expected.

## Exact commands (worktree)

```bash
rig default 4.6 && R --version | head -1        # must print 4.6.x
just worktree-sync                               # FIRST
just configure && just rcmdinstall && just force-document
# (no new exports — single install suffices)
just test 2>&1 > /tmp/1261i3-rust-test.log       # Read the log
just devtools-test 2>&1 > /tmp/1261i3-devtools.log
grep -E '\[ FAIL [0-9]+' /tmp/1261i3-devtools.log   # devtools::test always exits 0
just r-cmd-check 2>&1 > /tmp/1261i3-rcmdcheck.log   # Read; verify WARNING gone
cargo clippy --workspace --all-targets --locked -- -D warnings  # + all/all_s7 legs per ci.yml
cargo fmt --all
```

No snapshot churn expected (no macro output changes). If any snapshot or
trybuild `.stderr` changes: stop and report (never `TRYBUILD=overwrite`
locally — #1239).

## Must NOT touch

- `error_value.rs` or any PROTECT-sensitive file — this change is confined
  to `sys.rs`, `externalptr.rs`, `docs/NONAPI.md`, `rpkg/DESCRIPTION`,
  `rpkg/src/rust/impl_dots_tests.rs` (item 9), and the three `error-on`
  lines in `.github/workflows/ci.yml` (item 10).
- The other three #1261 items land via their own PRs — do not re-fix them;
  merge origin/main instead (see precondition).
- No `nonapi` feature-gating of the deleted declarations — delete outright.
- Generated files (`wrappers.R`, `wasm_registry.rs`). `NAMESPACE`/`man` WILL
  change for item 9 (`ImplDotsS3.Rd` etc.) — commit those regenerated files;
  the item-3 FFI change itself has no wrapper-visible effect.

## Done criteria

- Default build contains no `Rf_findVarInFrame` reference; `R CMD check`
  reports ZERO WARNINGs (compiled-code + both doc WARNINGs gone, prereq PRs
  merged in); A9 handle-unwrapping tests green; suites + three clippy legs
  green; `Depends: R (>= 4.5)` committed with the change; `error-on`
  flipped at the three active sites; PR body `Fixes #1261`.

## Escalation rule

If reality diverges from this plan — an anchor doesn't match, a caller of the
deleted declarations exists, a WARNING remains that the plan doesn't explain,
a test fails unexpectedly — **stop, commit nothing further, and report back.
Do not improvise.**
