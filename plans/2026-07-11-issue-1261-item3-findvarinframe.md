# Plan: #1261 item 3 — migrate `Rf_findVarInFrame` to API-blessed `R_getVarEx`

Date: 2026-07-11. Anchors verified against main @ 17f634d8.
Branch: `fix/1261-findvarinframe-api`.

Scope: one of #1261's four `R CMD check` WARNINGs — `checking compiled code`:
`miniextendr.so` contains a non-API call to `Rf_findVarInFrame` ("this entry
point may be removed soon"). Items 1/2/4 are handled by other PRs
(`plans/2026-07-11-issue-1248-s3-nongeneric-collision.md`,
`plans/2026-07-11-issue-1261-items-2-4-rd-dedup-int-literal.md`). Do NOT
touch them here.

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
   WARNING about `Rf_findVarInFrame` is GONE. The log will still show the
   other #1261 WARNINGs not yet merged — expected; reference their PRs in
   the PR body. PR body references #1261 (partial — do NOT `Fixes #1261`).

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

- `error_value.rs` or any PROTECT-sensitive file — this change is confined to
  `sys.rs`, `externalptr.rs`, `docs/NONAPI.md`, `rpkg/DESCRIPTION`.
- The other three #1261 items. Do not flip CI `error-on:` — that's the
  maintainer's close-out after all four items land.
- No `nonapi` feature-gating of the deleted declarations — delete outright.
- Generated files (`wrappers.R`, `wasm_registry.rs`); `NAMESPACE`/`man`
  expected unchanged (no wrapper-visible change).

## Done criteria

- Default build contains no `Rf_findVarInFrame` reference; `R CMD check`
  compiled-code WARNING gone; A9 handle-unwrapping tests green; suites +
  three clippy legs green; `Depends: R (>= 4.5)` committed with the change.

## Escalation rule

If reality diverges from this plan — an anchor doesn't match, a caller of the
deleted declarations exists, the maintainer has vetoed the R-floor bump, a
test fails unexpectedly — **stop, commit nothing further, and report back.
Do not improvise.**
