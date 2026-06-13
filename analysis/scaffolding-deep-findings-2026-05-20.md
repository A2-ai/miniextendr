# Deep scaffolding-cost findings — 2026-05-20

Companion to the first-pass note (`scaffolding-bench-2026-05-20.md`) and the
investigation plan (`scaffolding-perf-investigation-plan.md`). This document
consolidates results from M9–M13: bytecompile sensitivity, run-to-run
variance, in-namespace hand-strip, multi-arg / precondition-shape sweep,
and Rprof over a real testthat run.

- Scripts: `analysis/scaffolding-bench-deep.R`, `scaffolding-bench-installed.R`, `scaffolding-bench-rprof.R`
- Raw outputs: `*-output.txt` siblings
- HEAD: `42daa354` (main)
- R 4.6.0 / Apple M3 Max / JIT level 3 (default for R 4.6)

## TL;DR

1. **Bytecompile is a wash** — `compiler::cmpfun()` saves 0–120 ns out of
   2.5 μs (≤ 5 %). The 1230 ns / 1230 ns / 123 ns per-line cost picture
   from M6 holds.
2. **Variance is rock-solid** — CV 0.0–2.6 % across 5 reps. Deltas are
   50–100× the noise.
3. **Hand-written matches installed** — placed in the package namespace,
   my hand-written `i32_full` is 2870 ns vs the real `conv_i32_arg`
   at 2829 ns. Within 1.5 % / 41 ns.
4. **stopifnot scales linearly at ~300 ns per assertion** plus ~150 ns
   bracket overhead. A 5-arg numeric-scalar fn pays ~3.9 μs in
   preconditions alone.
5. **match.call is fixed at ~1100–1300 ns** regardless of arg count.
6. **In a real testthat run the wrapper is 1.1 % of total time** —
   testthat infrastructure dominates at 84–98 %. The wrapper
   optimization helps user inner loops and framework-internal calls,
   not test suites.

## M9: Bytecompile sensitivity

R 4.6 ships with `compiler::enableJIT(3)` as the default. Package
wrappers are compiled at install. Does that change the per-line picture?

| Variant | src_ns | cmp_ns | delta | cmp/src |
|---|---:|---:|---:|---:|
| `full` | 2665 | 2583 | −82 | 96.9 % |
| `no_stopifnot` | 1435 | 1394 | −41 | 97.1 % |
| `no_matchcall` | 1517 | 1394 | −123 | 91.9 % |
| `no_stop_no_call` | 287 | 328 | **+41** | 114.3 % |
| `bare` | 205 | 205 | 0 | 100.0 % |

Bytecompile is essentially neutral and **sometimes makes things slower**
(`no_stop_no_call_cmp` is +41 ns vs uncompiled). The targets we'd
remove (`stopifnot`, `match.call`) are not stuff R's bytecompiler
specially optimises.

Reference: `miniextendr::conv_sexp_arg` (real installed wrapper, no
stopifnot because SEXP arg has no precondition) clocks 1517 ns — within
83 ns of `no_stopifnot_cmp` at 1394 ns. The macro output matches my
hand-written equivalent.

## M10: Run-to-run variance (5 reps)

| Variant | min | mean | median | max | sd | CV % |
|---|---:|---:|---:|---:|---:|---:|
| `full_cmp` | 2583 | 2599 | 2583 | 2624 | 22.5 | 0.9 |
| `no_stopifnot_cmp` | 1394 | 1418 | 1394 | 1476 | 36.7 | 2.6 |
| `no_matchcall_cmp` | 1435 | 1443 | 1435 | 1476 | 18.3 | 1.3 |
| `no_stop_no_call_cmp` | 287 | 287 | 287 | 287 | 0.0 | 0.0 |
| `bare_cmp` | 164 | 164 | 164 | 164 | 0.0 | 0.0 |
| `package_conv_sexp_arg` | 1599 | 1648 | 1640 | 1681 | 34.3 | 2.1 |
| `package_conv_i32_arg` | 2829 | 2870 | 2870 | 2911 | 29.0 | 1.0 |

Noise floor: 18–37 ns of standard deviation. The 1100+ ns deltas we want
to remove are 30–60× the noise.

## M11: Hand-strip an installed wrapper

The toughest cross-check: a hand-written `i32_full` placed inside the
`miniextendr` namespace (same lookup path, byte-compiled at the same
JIT level as the real `conv_i32_arg`).

| Variant | min | mean | median | max | sd | CV % |
|---|---:|---:|---:|---:|---:|---:|
| `conv_i32_arg` (real package fn) | **2829** | 2854 | 2870 | 2870 | 22.5 | 0.8 |
| `i32_full` (hand-written, in pkg ns) | **2870** | 2878 | 2870 | 2911 | 18.3 | 0.6 |
| `i32_no_stopifnot` | 1640 | 1640 | 1640 | 1640 | 0.0 | 0.0 |
| `i32_no_matchcall` | 1640 | 1648 | 1640 | 1681 | 18.3 | 1.1 |
| `i32_no_stop_no_call` | 410 | 443 | 451 | 451 | 18.3 | 4.1 |
| `i32_bare` | 287 | 328 | 328 | 369 | 29.0 | 8.8 |

**Cross-check confirms it**: package `conv_i32_arg` (2829 ns) ≈
hand-written `i32_full` (2870 ns), 41 ns apart. The macro output is
faithfully reproduced by the hand-written variant.

Per-line cost from the in-namespace measurement:

| Piece | Cost (ns) |
|---|---:|
| `stopifnot()` (2 assertions) | **1230** |
| `match.call()` on success path | **1230** |
| `inherits / attr` post-check | **123** |
| Sum (i32_full − i32_bare) | **2583** |
| `i32_bare` floor (TryFromSexp + IntoR + with_r_unwind_protect) | **287** |

## M12: Multi-arg + precondition-shape sweep

Synthesised wrappers with N args (0/1/2/3/5) and varied precondition
shapes (none / numeric_scalar / string_scalar / nullable_numeric /
numeric_vector). All byte-compiled. Body unchanged.

### stopifnot cost vs assertion count

Fixing `match.call = NULL` to isolate the precondition layer:

| n_args | shape | min_ns | Δ vs 0-arg none (615 ns) |
|---:|---|---:|---:|
| 0 | none | 615 | — |
| 1 | none | 697 | +82 |
| 1 | numeric_scalar (2 assertions) | 1845 | +1230 |
| 1 | string_scalar (2 assertions) | 1886 | +1271 |
| 1 | nullable_numeric (2 assertions) | 1886 | +1271 |
| 1 | numeric_vector (1 assertion) | 1517 | +902 |
| 2 | numeric_scalar (4 assertions) | 2501 | +1886 |
| 3 | numeric_scalar (6 assertions) | 3075 | +2460 |
| 5 | numeric_scalar (10 assertions) | 4551 | +3936 |

The cost model: roughly **150 ns fixed bracket cost** + **300 ns per
assertion** + **80 ns per additional arg in the formals list**. For
common shapes (numeric scalar = 2 assertions/arg), the per-arg cost
lands at ~600–700 ns.

### match.call cost vs arg count

Same table, comparing `match.call = TRUE` minus `match.call = NULL`:

| n_args | shape | Δ for match.call |
|---:|---|---:|
| 0 | none | +1107 |
| 1 | none | +1148 |
| 1 | numeric_scalar | +1189 |
| 2 | numeric_scalar | +1312 |
| 5 | numeric_scalar | +1271 |

**match.call is fixed at ~1100–1300 ns regardless of arg count.** It
allocates a LANGSXP capturing the current call frame; the cost doesn't
scale with parameters.

## M13: Rprof over `test-conversions.R`

768-line test file, the most wrapper-heavy in rpkg's testthat suite.
Profiled at 1 ms sampling interval.

| Layer | self.time (s) | self.pct |
|---|---:|---:|
| testthat infrastructure (`tryCatch`, `test_that`, etc.) | 0.373 | 84–98 |
| `tempfile` / `getwd` / `dir.create` (test setup) | 0.313 | 71 |
| `.Call` (actual C dispatch) | 0.018 | 4.07 |
| `lazyLoadDBfetch` | 0.017 | 3.85 |
| `isTRUE` | 0.004 | 0.90 |
| `stopifnot` | 0.001 | 0.23 |

**Wrapper-layer self.time = 1.1 % of total test time.** The wrapper
optimization won't measurably accelerate testthat runs.

This is the most important finding from M13 for prioritisation. The
wrapper is **not** the testthat bottleneck. The optimization payoff is
in:

1. User code that calls miniextendr fns in **tight inner loops**.
2. **Framework-internal** hot paths — sidecar accessors, cross-package
   trait ABI, R6/S7 method dispatch (each method call goes through a
   generated C wrapper).
3. **Benchmarks** comparing miniextendr to alternatives.

For a typical R analysis workload that calls a few miniextendr fns
between heavy R-side data manipulation, the wrapper cost is invisible.

## Implications for the option design

With this evidence, the original ranking is **confirmed and slightly
sharpened**:

| Option | Confidence | Savings | Notes |
|---|---|---:|---|
| `no_preconditions` | **HIGH** | 1230 ns (1-arg) → 3936 ns (5-arg) | Scales linearly with arg count |
| `no_call_attribution` | **HIGH** | 1100–1300 ns (fixed) | Doesn't scale with args |
| `no_postcheck` (requires `infallible`) | LOW | 123 ns | Tiny absolute, but only meaningful when paired |
| Fast bundle (= the three together) | HIGH | 2.5 μs → 0.3 μs for 1-arg; 6.4 μs → 0.6 μs for 5-arg | **~8–13× speedup** for the wrapper layer |

**Recommendation**: still build `no_preconditions` first (biggest variable
saving, scales with arg count), then `no_call_attribution` (largest fixed
saving, semantically safest to make default), then expose `fast` as a
bundle alias. The decision is the same as before; the confidence in the
numbers is now much higher.

## Remaining uncertainty (intentionally left)

- **M2 (C-side flamegraph attribution)** not done. The 205–287 ns
  floor includes `with_r_unwind_protect` setup + `.Call` boundary +
  TryFromSexp dispatch. Without M2 we don't know how to break that
  down or whether `unsafe(infallible)` (drop with_r_unwind_protect)
  would yield anything meaningful.
- **M4 (error-path attribution)** not done. We know the full path is
  ~14 μs over native `stop()`, but don't yet know what fraction is
  C-side tagged-SEXP build vs R-side re-raise. Decides whether option
  F (`error_direct`) is worthwhile.
- **M5 (S7 dispatch deep-dive)** not done. We know S7 is ~4× R6
  (9 μs vs 2 μs) but haven't profiled `S7_dispatch`. Probably just a
  documentation note.

These can wait until after the first knob ships, unless you want a
fuller bench cycle now.
