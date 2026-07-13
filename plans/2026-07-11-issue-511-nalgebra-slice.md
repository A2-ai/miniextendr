# Plan: #511 slice 1 — nalgebra upstream-example fixtures

Date: 2026-07-11. Verified against main @ 17f634d8.
Branch: `test/511-nalgebra-fixtures`.

Scope: ONLY the nalgebra section (§1) of issue #511 — the issue is a large
sliceable backlog; nalgebra is its own designated first slice (17 upstream
examples, weakest coverage). Later slices are separate PRs, not this one.
PR references #511 (partial — do NOT `Fixes #511`; the issue is the umbrella).

## Content spec

Issue #511's body §1 ("nalgebra") enumerates the fixtures with exact
signatures (`nalgebra_solve_4x4`, matrix-construction, decompositions, etc.).
That list IS the spec — read the full issue body (`gh issue view 511`) and
implement every §1 item NOT already present in
`rpkg/src/rust/nalgebra_adapter_tests.rs` (currently 18 `pub fn`s — the
issue's table said 10, so some have landed since; diff by function name and
skip existing ones, listing skipped-as-present in the PR body).

## Rules

1. Each fixture: `#[miniextendr]` fn in `nalgebra_adapter_tests.rs`
   following the file's existing conversion idioms (DVector/DMatrix ↔ R
   vectors/matrices — copy the in-file patterns; do NOT invent new
   conversion shapes), `///` roxygen with `@param`/`@export` matching the
   file's style, feature-gated exactly as the file already is (the module's
   `#[cfg(feature = "nalgebra")]` gating lives at the `mod` declaration in
   `lib.rs` — verify, don't duplicate gates).
2. testthat: extend the existing nalgebra test file (grep
   `rpkg/tests/testthat/` for the current fixtures' tests); numeric
   assertions use `expect_equal(..., tolerance = 1e-10)` for decomposition
   results; each upstream example's mathematical property is the assertion
   (e.g. `a %*% x == b` for the solver; reconstruction `Q %*% R == A` for
   decompositions).
3. Pure compute, no SEXP storage across allocations → no gc-stress fixture
   (#430 not triggered).
4. If an issue-§1 item cannot be expressed with the crate's existing
   nalgebra conversions (e.g. needs a type the adapter doesn't convert),
   SKIP it and list it in the PR body under "needs conversion support"
   with one line on what's missing — do not extend the api crate's
   conversion surface in this PR.

## Exact commands (worktree)

```bash
rig default 4.6 && R --version | head -1
just worktree-sync                               # FIRST
cargo test -p miniextendr-api --features nalgebra 2>&1 > /tmp/511-api.log
just configure && just rcmdinstall && just force-document && just rcmdinstall
#                                   ^ ×2: new exports
just devtools-test 2>&1 > /tmp/511-devtools.log
grep -E '\[ FAIL [0-9]+' /tmp/511-devtools.log   # devtools::test always exits 0
cargo clippy --workspace --all-targets --locked -- -D warnings  # + all/all_s7 legs per ci.yml
cargo fmt --all
```

Commit regenerated `NAMESPACE`/`man` with the fixtures.

## Must NOT touch

- `miniextendr-api` conversion code (rule 4 above).
- Other crates' adapter test files (later slices).
- No new dependencies; no upstream example code vendored verbatim (rewrite
  minimal versions — these are fixtures, not ports).

## Done criteria

- Every implementable §1 item has a fixture + property-asserting test; the
  rest are listed with reasons; suites + three clippy legs green.

## Escalation rule

If reality diverges from this plan — the issue's §1 list conflicts with the
current adapter API in a way rule 4 can't absorb — **stop, commit nothing
further, and report back. Do not improvise.**
