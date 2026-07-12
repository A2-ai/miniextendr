# Plan: audit 2026-07-12 P1 — webR Tier 3 testthat leg traps with `unreachable`

Date: 2026-07-13. Anchors verified against main @ 656e5cdd.
Branch: `ci/audit-webr-tier3-testthat-unreachable`.

Covers 2026-07-12 audit worklist item 8. **Re-verified live on current
main** — this is NOT stale post-#1311: run
[29203614715](https://github.com/A2-ai/miniextendr/actions/runs/29203614715)
(main @ 656e5cdd) fails, and its log shows the #1311/#1273 cross-package
legs PASSING ("mxsmoke loaded; add(2, 3) == 5 via its own f64 path … #1273
cross-package symbol isolation") right before the testthat leg trips.

## Verified state

- **Regression window is exact**: webR was green through `e504646c`
  (2026-07-11) and red from `27689995` — which is PR **#1299**, "ci(webr):
  opt-in informational testthat pass in the tier-3 smoke (#1255)". The
  testthat leg has **never been green on main**; every main run since
  (`d10462ce`, `a88db764`, `79e24e51` = #1311, `656e5cdd`) fails
  identically. This is a born-red new leg, not a product regression.
- Failure line: `[tier3][testthat] FAIL: harness error before counts:
  unreachable` — emitted at `tests/webr-node-smoke/smoke.mjs:244`. The leg
  runs `testthat::test_dir` over the NODEFS-mounted `rpkg/tests` with
  `reporter = "silent"`, `stop_on_failure = FALSE`, and
  `Sys.setenv(MINIEXTENDR_SKIP_STRESS = "1", NOT_CRAN = "true")`
  (`smoke.mjs:302,321-326`) — so gctorture stress files are already
  excluded; the trap is elsewhere.
- Gating design (deliberate): test *failures* never gate; a harness error
  before the counts line, or the wall-clock cap, does
  (`tests/webr-smoke.sh:598-612`; `SMOKE_TESTTHAT` default ON, cap 2400s).
  `unreachable` is a wasm trap that kills the R session before testthat
  can print counts — precisely the gating condition.
- **No open issue covers this.** Adjacent-but-distinct open issues:
  #1312 (symbol residual guards), #1310 (api no_mangle GOT surface),
  #1309 (patch-block assert), #1307 (wasm_registry unused imports),
  #1290 (smoke.sh /tmp persistence), #1254 (arm64 image).

## Hypotheses (check in this order)

1. **Rust panic → abort → wasm `unreachable`.** rpkg's suite deliberately
   triggers Rust panics (e.g. `rpkg/src/rust/panic_tests.rs` fixtures)
   expecting the framework to convert them to R errors. Native relies on
   the worker thread / unwind machinery; under the wasm32 build (no worker
   thread, different panic strategy) a panic may compile to `abort` →
   `unreachable` instruction. Check what panic strategy the wasm cdylib is
   built with (Makevars/webr build flags, emscripten exception support)
   and whether `add(bad args)`-style coercion errors also route through
   panic on wasm.
2. **Stack exhaustion** — wasm default stack is far smaller than the 16 MB
   R thread; deep-recursion or big-locals tests could trap.
3. **Feature-set mismatch** — tests calling exports compiled out of the
   wasm build should produce plain R errors ("could not find function"),
   not traps; least likely, verify last.

## Work items (flat order)

1. **Reproduce locally**: `bash tests/webr-smoke.sh` in the container with
   `SMOKE_TESTTHAT=1` (export `LANG=LC_ALL=C.UTF-8` — known container
   locale trap). Capture the full tier-3 log to a file and Read it.
2. **Isolate the trapping file**: replace the single `test_dir` call with
   a per-file loop in a local experiment (iterate
   `testthat::find_test_scripts`, print the filename *before* each
   `test_file`, flush) — the last printed name is the trapping file; then
   bisect within it. Do this as a temporary local edit of `smoke.mjs` or
   an env-guarded mode; whether it ships is decided by item 4.
3. **Diagnose against the hypotheses** and fix at the right layer:
   - If (1): decide between making panic→R-error transport sound on wasm
     (preferred if tractable — check how webR/emscripten builds handle
     `panic = unwind`) or introducing a skip guard à la
     `MINIEXTENDR_SKIP_STRESS` (e.g. `MINIEXTENDR_WASM=1` set in
     `smoke.mjs` beside `smoke.mjs:321`, with `skip_if` helpers in
     `rpkg/tests/testthat/` mirroring `helper-gc-stress.R`) for tests whose
     *mechanism* (panic unwinding) cannot exist on wasm. Every skipped
     file/test gets a comment naming this plan and, if the transport fix
     is deferred, a tracked issue (implementer files it — repo policy).
   - If (2): grow the wasm stack in the build flags, or skip the specific
     test with rationale.
4. **Keep the leg meaningful**: after the fix, the testthat pass must print
   its counts line on main. Consider shipping the per-file progress line
   (item 2) permanently — it converts any future trap from "unreachable,
   no context" into "unreachable after test-foo.R", at negligible cost
   under the silent reporter.
5. **Gate-or-informational decision** (flag for maintainer, default
   proposed here): while the fix is in review, main's webR badge is red.
   Either merge this fix promptly, or — if diagnosis drags — a one-line
   interim PR flipping CI's main-push `SMOKE_TESTTHAT` to `0` (the
   knob already exists, `tests/webr-smoke.sh:49,603`) restores an honest
   green for the still-covered build/install/load/cross-package legs.
   Do NOT leave it red-and-ignored; that contradicts the no-known-issues
   principle.

## Exact commands (worktree)

```bash
bash tests/webr-smoke.sh 2>&1 > /tmp/audit-webr-smoke.log   # Read it; needs Docker + disk
# iterate with SMOKE_TESTTHAT=1 / per-file experiment as per items 1-2
just test                                                    # if any Rust changed
just devtools-test 2>&1 > /tmp/audit-webr-devtools.log       # if any rpkg test/helper changed
grep -E '\[ FAIL [0-9]+' /tmp/audit-webr-devtools.log
```

CI proof: the PR's webR workflow run must show the tier-3 testthat counts
line (or, for the interim option, the leg cleanly disabled on the main-push
path with the issue referenced).

## Must NOT touch

- The #1273/#1311 cross-package assertion legs in `smoke.mjs` (they pass;
  they are the regression test for the symbol-prefix fix).
- Native test suites' behavior (any wasm skip guard must be inert when the
  env var is unset).
- `webr.yml`'s build/install tiers (tier 1/2 are green).

## Done criteria

- Root cause of the `unreachable` trap identified and written down
  (PR body + `reviews/` entry if the mechanism was non-obvious).
- webR workflow green on main with the testthat leg either passing
  (counts line printed) or explicitly informational per the maintainer's
  gate decision — no silent red.
- Any skipped-on-wasm test carries a tracked issue if the underlying
  transport fix is deferred.

## Escalation rule

If reality diverges from this plan — the trap reproduces outside the
testthat leg, the per-file isolation shows *multiple* unrelated trapping
files, or fixing panic transport requires api-crate surgery beyond a
focused change — **stop, commit nothing further, and report back. Do not
improvise.**
