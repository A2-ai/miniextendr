# Plan: #1244 — fast-default runtime CI leg + bare-fn fixture

Date: 2026-07-11. Anchors verified against main @ 17f634d8.
Branch: `ci/1244-fast-default-leg`.

Related: #1210 (added `fast-default`), #1113 (same family — broadening
r6/s7 legs; separate plan, do not fold), #667 (internal-implies-fast — open
decision, untouched here).

## The gap (verified)

`fast-default` exists at `miniextendr-macros/Cargo.toml:31` and
`miniextendr-api/Cargo.toml:130-132` (and rides the `full-codegen`/
`full-codegen-s7` aggregates at `:171,:181`, so clippy compiles it), but:

- **rpkg has NO passthrough**: `rpkg/src/rust/Cargo.toml:66-71` declares
  `strict-default`/`coerce-default`/`r6-default`/`s7-default`/`worker-default`
  passthroughs — `fast-default` is missing.
- **detect-features denylist** (`rpkg/tools/detect-features.R:73-79`) lists
  the other five default-flippers; `fast-default` is absent (harmless today
  only because the passthrough doesn't exist — adding the passthrough
  WITHOUT the denylist entry would silently enable fast codegen in every
  default build; the two edits must land together).
- **No runtime leg**: `.github/workflows/ci.yml` feature-legs matrix
  (`:1188-1207`) has `extras` / `worker-default` / `r6-default` /
  `s7-default` rows only.
- The two `#[cfg(feature = "fast-default")]` unit tests
  (`miniextendr-macros/src/tests.rs:1160,1175`) are compiled by clippy but
  run by NO job.

## Work items (flat order)

1. `rpkg/src/rust/Cargo.toml` — insert
   `fast-default = ["miniextendr-api/fast-default"]` beside the other
   default-flippers (after line 67's `coerce-default`). Do NOT add it to any
   "all additive features" aggregate on line 72+ (it flips wrapper semantics;
   read that aggregate's comment and keep it excluded like the other
   `-default` selectors).
2. `rpkg/tools/detect-features.R` — add `"fast-default"` to the `deny` vector
   (`:76-79` block) plus a one-line comment in the block above it (mirror the
   `worker-default` comment style at `:70`): fast-default drops R-side
   preconditions crate-wide; runtime coverage lives in the weekly leg.
3. `rpkg/src/rust/lib.rs` (~`:2107-2112`, the `cfg!` feature list in
   `miniextendr_enabled_features`'s builder) — add the
   `if cfg!(feature = "fast-default") { features.push("fast-default"); }`
   arm so `miniextendr_has_feature("fast-default")` works from R.
4. Fixtures in `rpkg/src/rust/feature_default_fixtures.rs` (mirror the
   strict pair at `:40-48`, same `///` doc style; update the module doc at
   `:2` to mention fast-default):
   - `fdefault_fast_bare_i32(x: i32) -> i32` — **bare**, no knob attrs;
     returns `x` (same body shape as `fdefault_strict_i64`).
   - `#[miniextendr(no_fast)] fdefault_no_fast_i32(x: i32) -> i32` — opt-out
     probe (preconditions restored even under fast-default).
5. testthat in `rpkg/tests/testthat/test-feature-defaults.R` (mirror the
   strict/coerce branch style at `:21-46`):
   ```r
   fast_on <- miniextendr_has_feature("fast-default")
   ```
   - `fast_on == TRUE`: `fdefault_fast_bare_i32("nope")` raises a
     `rust_error` (Rust conversion error — the stopifnot precondition is
     gone); `fdefault_no_fast_i32("nope")` raises the R-side precondition
     error (NOT class `rust_error`). Pin both error classes; copy the exact
     expectation idioms used by `test-fast-fixtures.R` for the explicit-knob
     path (e.g. `expect_s3_class(tryCatch(..., error = identity), "rust_error")`).
   - `fast_on == FALSE` (every default build): `fdefault_fast_bare_i32("nope")`
     raises the stopifnot-shaped error; assert it is NOT a `rust_error`.
6. CI leg in `.github/workflows/ci.yml` — add a matrix row after
   `s7-default` (`:1205-1207`):
   ```yaml
   - leg: fast-default
     features: fast-default
     filter: feature-defaults
   ```
   (Bare fns intentionally lose their stopifnot text under the flip, so the
   full suite fails by design — `filter: feature-defaults` like the r6/s7
   rows.) Update the job's header comment block (`:1166-1178`) to mention the
   new leg. Then add a step inside the job, scoped
   `if: matrix.leg == 'fast-default'`, that runs
   `cargo test -p miniextendr-macros --features fast-default fast_default`
   so `tests.rs:1160/:1175` execute weekly (place it before the R build
   steps; keep sccache env inherited).
7. Update `docs/FEATURE_DEFAULTS.md` (or wherever the default-flipper table
   lives — grep `worker-default` in `docs/`) with a fast-default row noting
   the weekly-leg coverage, if the table exists and lacks it.

## Exact commands (worktree)

```bash
rig default 4.6 && R --version | head -1        # must print 4.6.x
just worktree-sync                               # FIRST
just configure && just rcmdinstall && just force-document && just rcmdinstall
#                                   ^ ×2: fdefault_* fixtures are NEW exports
just devtools-test 2>&1 > /tmp/1244-devtools.log     # default build: fast_on FALSE branch
grep -E '\[ FAIL [0-9]+' /tmp/1244-devtools.log      # devtools::test always exits 0
# Feature-on verification (this is the leg's local reproduction; one full rebuild).
# CARGO_FEATURES must be exported in the SAME shell as BOTH the install and
# the tests (configure re-runs re-detect defaults silently otherwise):
export CARGO_FEATURES=fast-default
just configure && just rcmdinstall
Rscript -e 'testthat::test_local("rpkg", filter = "feature-defaults")' 2>&1 > /tmp/1244-fastleg.log
grep -E '\[ FAIL [0-9]+' /tmp/1244-fastleg.log
unset CARGO_FEATURES && just configure && just rcmdinstall   # restore default build
cargo test -p miniextendr-macros --features fast-default fast_default 2>&1 > /tmp/1244-macrotests.log
cargo clippy --workspace --all-targets --locked -- -D warnings  # + all/all_s7 legs per ci.yml
cargo fmt --all
# YAML sanity:
python3 -c 'import yaml; yaml.safe_load(open(".github/workflows/ci.yml"))'
```

Commit regenerated `NAMESPACE`/`man/*.Rd` (new exports) with the Rust change.
If the exact `Rscript testthat` invocation above fails to load the project
library, run it as `cd rpkg && Rscript -e '...'` variants per the repo's
existing leg reproduction text in ci.yml (`:1277-1284` echoes it) — copy THAT
text's commands verbatim.

## Must NOT touch

- The r6/s7/strict/coerce leg rows and their filter semantics (#1113's
  broadening is a separate plan).
- `#[miniextendr(fast)]` explicit-knob fixtures (`fast_fixtures.rs`,
  `test-fast-fixtures.R`) — per-PR coverage, unrelated.
- `full-codegen` aggregates in miniextendr-api/Cargo.toml.
- Generated files (`wrappers.R`, `wasm_registry.rs`).

## Done criteria

- Weekly leg row exists and is reproducible locally per the commands above;
  fixture branches pass in BOTH default and fast-default builds; the two
  macros unit tests run in the leg; `fast-default` denylisted in
  detect-features (default builds unchanged — proven by the default-build
  `devtools-test` run); three clippy legs green; `Fixes #1244`.

## Escalation rule

If reality diverges from this plan — an anchor doesn't match, the fast-flip
changes MORE than precondition/call-attribution behavior in the fixtures'
errors, the default build's suite changes at all — **stop, commit nothing
further, and report back. Do not improvise.**
