# CI speed audit — dedupe stress tests, revive main-push CI

Date: 2026-07-10. Audited run: `29070047852` (PR CI, success, ~49 min wall).

## Measurements

Per-PR job durations (wall):

| Job | Duration | Dominant cost |
|---|---|---|
| R CMD check / Linux devel | 48.5 min | `checking tests` ≈ 31 min |
| R CMD check / Linux oldrel-1 | 45.0 min | same |
| R tests / Linux | 44.0 min | `testthat::test_local()` = 34.5 min |
| R CMD check / Linux release | 39.9 min | `checking tests [31m/30m]` |
| CRAN-like check | 39.1 min | check step 36 min (same tests) |
| Sync Checks | 17.0 min | `wrappers-sync-check` 13.2 min (no sccache!) |
| everything else | ≤ 4 min each | — |

Test-suite breakdown (2051 s total, 7084 tests):

| File | Time | Share |
|---|---|---|
| gc-stress-fixtures | 1360 s | 66% |
| externalptr-self-root | 233 s | 11% |
| iter-to-dataframe | 194 s | 9% |
| dataframe-deserialize | 147 s | 7% |
| all ~90 other files | ~117 s | 6% |

Key facts:
- Rust compile is NOT the bottleneck: sccache-warm install ≈ 5 min elapsed
  (`checking whether package can be installed ... [66s/301s]`).
- The four gctorture files (~32 min) run identically in **five** jobs
  → ~2.6 h of duplicated runner time per push event.
- Main-push `r-tests` = suite + 3× heap-check rounds ≈ 140 min > 90-min
  timeout → the last several main-push runs all cancelled at ~90+ min.
  The heap-check guard on main effectively never completes.
- `cancel-in-progress: true` applies to main pushes too, so merge trains
  cancel main-tip CI serially.
- Nightly `gctorture-nightly.yml` already runs the full suite under
  `gctorture2(step=100)` — the deep GC net exists independently of PR CI.
- `NOT_CRAN=true` is exported by r-lib actions in every job (including
  CRAN-like check), so `skip_on_cran()` alone cannot gate CI; an explicit
  env var is required.

## Changes (flat priority)

1. **Gate the four gctorture-heavy test files** behind
   `MINIEXTENDR_SKIP_STRESS` (new `helper-gc-stress.R`; gates only the
   torture blocks — cheap structure/value assertions keep running
   everywhere). Helper also calls `skip_on_cran()`: a 31-min test suite
   would be rejected by real CRAN anyway.
2. **New `r-stress-tests` job** (2 shards) is now the single place the
   gctorture files run per-PR. The dynamic no-arg-fixture sweep (77
   fixtures × 5 iters) splits by fixture index via
   `MINIEXTENDR_STRESS_SHARD=k/n`; the three other files are distributed
   across the shards by testthat `filter`. Coverage per PR is unchanged —
   same fixtures, same iteration counts, just not repeated 5×.
3. `r-check-linux`, `cran-check`, `r-tests` set `MINIEXTENDR_SKIP_STRESS=1`.
   Their `checking tests` drops ~31 min → ~4 min.
4. **`r-check-linux` matrix trims to `release` on PRs**; devel + oldrel-1
   still run on push-to-main / schedule / dispatch (dynamic matrix via
   `fromJSON`). devel is advisory (`continue-on-error`) and oldrel-1
   rarely diverges — post-merge signal is enough.
5. **Main-push r-tests fits its budget again**: heap-check rounds inherit
   the stress skip (3 × ~5 min instead of 3 × ~35 min). Main guard is alive.
6. **Concurrency: only PRs cancel in-progress runs** — main-push runs now
   queue instead of cancelling each other.
7. **Sync Checks gets sccache + rust-cache** (was completely uncached):
   13 min install → ~5 min.
8. Housekeeping: autoconf install steps skip apt-get when preinstalled;
   feature-legs' sccache-action pinned to v0.0.10 like everywhere else.

## Expected outcome

- PR wall-clock: ~50 min → ~30 min (critical path: stress shard).
- PR runner time: ~240 min → ~135 min.
- Main-push CI completes instead of timing out/cancelling.
- Per-PR GC guard preserved (all fixtures, same iterations, once).
- Real-CRAN test time drops below CRAN's patience threshold.

## Explicitly out of scope (tracked separately)

- The 4 standing `R CMD check` WARNINGs visible in every check log
  (R code for possible problems / Rd usage / compiled code / unstated
  test deps) — pre-existing, gated by `error-on: "error"`. → issue.
- webr.yml duration (~27 min) — under active work in PRs #1256–#1258.
- devel/oldrel-1 lose per-PR (not post-merge) stress coverage — accepted
  trade; nightly + main-push retain it on release R.
