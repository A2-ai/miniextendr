# Rayon `RDataFrameBuilder` — flattened vs column-granular fill

Date: 2026-05-29
Branch: `rayon-flatten-granularity` (stacked on #768 `with-r-dataframe-par`,
base commit `78c93b3c`).
Machine: macOS arm64, 14 logical cores (`sysctl hw.ncpu` = 14).
Harness: `cargo bench --manifest-path=miniextendr-bench/Cargo.toml --features rayon --bench rayon -- dataframe_few_long`
(`divan`, `sample_count = 30`).

## What is measured

A **few-long-columns** data.frame fill: 3 `f64` columns × 4,000,000 rows. Each
cell runs a small compute-bound transcendental kernel (`df_cell`: a `sqrt`
followed by 8 iterations of `sin`/`ln`/`sqrt`) so the fill is CPU-bound rather
than allocation/memcpy-bound (the scheduling difference is otherwise hidden in
allocation noise).

- `dataframe_few_long_flattened` — the production `RDataFrameBuilder::build()`,
  which flattens the job to one `(column, row-range)` work-list and runs a
  single `par_iter`. Each column shatters into `~nthreads*4` chunks, so all 14
  cores stay busy even with only 3 columns.
- `dataframe_few_long_column_granular` — a baseline that fans out one rayon task
  per column, each filling its whole column serially (no internal row chunking).
  With 3 columns it keeps only 3 of 14 cores busy. Allocation + assembly use the
  same PROTECT discipline as the builder, so the delta is purely the fill
  scheduling.

## Results (3 independent `cargo bench` runs)

`median` is the headline; `min` / `max` show the spread. All times in ms.

| Run | bench                | median | min   | max    |
|-----|----------------------|--------|-------|--------|
| 1   | column_granular      | 549.0  | 543.7 | 568.1  |
| 1   | flattened            | 151.6  | 148.5 | 155.2  |
| 2   | column_granular      | 548.8  | 546.2 | 585.5  |
| 2   | flattened            | 151.9  | 148.5 | 170.3  |
| 3   | column_granular      | 548.9  | 546.4 | 568.7  |
| 3   | flattened            | 152.8  | 148.2 | 165.1  |

### Summary

- **Median speedup: ~3.6×** (≈549 ms → ≈152 ms), stable across all 3 runs.
- The ratio tracks core utilisation: column-granular pins 3 of 14 cores
  (theoretical ceiling ≈ 14/3 ≈ 4.6×); the measured 3.6× is that minus
  scheduling/allocation overhead.
- **Variance is low** for both (column-granular CV < 1%, flattened CV ~1%).

### Earlier (pure-memcpy) kernel

With a trivial `(offset+i + j).sqrt()` fill (allocation-dominated), the medians
were ~1.45 ms (column-granular) vs ~1.21 ms (flattened) — only ~17% on the
median, but column-granular showed a catastrophic **tail**: max ~22–24 ms vs
~1.5–1.9 ms flattened (10–15× worse), the classic under-saturation straggler
signature. The compute-bound kernel above is the representative case for a fill
that does real per-row work.

## Reproduce

```bash
just bench-features --bench rayon -- dataframe_few_long
# or directly:
cargo bench --manifest-path=miniextendr-bench/Cargo.toml --features rayon \
  --bench rayon -- dataframe_few_long
```
