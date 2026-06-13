# Scaffolding-cost investigation plan — 2026-05-20

Companion to `analysis/scaffolding-bench-2026-05-20.md`. The first pass said
*the R wrapper is ~1.4 μs and `TryFromSexp`/`IntoR` is ~1.3 μs.* Before we
add knobs to `#[miniextendr(...)]`, we need (a) finer attribution within
each layer, (b) confirmation of which knobs are safe to flip without
breaking semantics. This plan lists what we'd need to learn and how.

## 0. Existing macro option surface (so we don't propose duplicates)

### Fn-level boolean flags (`#[miniextendr(...)]`)

| Option            | Codegen effect                                      | Defaultable via cargo feature |
|-------------------|------------------------------------------------------|-------------------------------|
| `invisible` / `visible` | wraps R return in `invisible(...)` or not       | —                             |
| `check_interrupt` | inserts `R_CheckUserInterrupt()` in C wrapper        | —                             |
| `worker` / `no_worker` | forces / forbids worker-thread routing          | `default-worker`              |
| `coerce` / `no_coerce` | enables `coerce` for **all** params              | `default-coerce`              |
| `rng`             | wraps body in `GetRNGstate()` / `PutRNGstate()`      | —                             |
| `unwrap_in_r`     | returns `Result<T, E>` to R without unwrapping       | —                             |
| `strict` / `no_strict` | strict-mode IntoR conversions                   | `default-strict`              |
| `internal`        | `@keywords internal`, suppresses `@export`           | —                             |
| `noexport`        | suppresses `@export` only                            | —                             |
| `export`          | force `@export` on non-pub fns                       | —                             |

### Fn-level key-value flags

| Option         | What it does                                                 |
|----------------|--------------------------------------------------------------|
| `prefer = "auto"\|"list"\|"externalptr"\|"vector"\|"native"` | force a specific `IntoR` path |
| `dots = typed_list!(...)` | typed dots validation                              |
| `lifecycle = "stage"` (or `lifecycle(...)`) | deprecation/experimental tagging |
| `doc = "..."`  | override roxygen body                                        |
| `c_symbol = "..."` | override generated C symbol name                          |
| `r_name = "..."` | override generated R function name                          |
| `r_entry = "..."` | inject R code at the very top of the wrapper body          |
| `r_post_checks = "..."` | inject R code after built-in checks, before `.Call` |
| `r_on_exit = "..."` (or `r_on_exit(expr, add, after)`) | register `on.exit()` |

### Fn-level nested

- `s3(generic = "...", class = "...")` — S3 method registration
- `lifecycle(stage, when, ...)` — full deprecation spec

### Per-parameter (`#[miniextendr(...)]` on a `fn` arg)

- `coerce` — per-param coercion (vs fn-level `coerce`)
- `match_arg` — emit `match.arg(...)` for string args
- `default = "<R expr>"` — default value
- `choices("a", "b", ...)` — restrict + emit match.arg
- `several_ok` — multi-value `match.arg(several.ok = TRUE)`

### Cargo features that flip codegen project-wide

- `default-strict`, `default-coerce`, `default-r6`, `default-s7`, `default-worker`
- `worker-thread` (gates `run_on_worker(f) → Ok(f())` inline vs real worker)
- `nonapi` — exposes non-API R fns
- `log` — drains worker-thread log queue at unwind exits

**Observation**: there is **no existing option that lets a user opt out of
any of the perf-relevant scaffolding**. Closest is `null_call_attribution`,
but that is an internal mechanism (used only for R6 finalizer / S7 lambda
contexts), not user-exposed via attribute.

## 1. What we don't know yet

The first pass measured *aggregate* costs per layer. To choose knobs
intelligently we need finer attribution within each layer. Specifically:

### R-side wrapper (~1.4 μs aggregate)

The wrapper for a 1-arg scalar fn does:

```r
fn <- function(x) {
  stopifnot(
    "'x' must be numeric, logical, or raw" = is.numeric(x) || is.logical(x) || is.raw(x),
    "'x' must have length 1" = length(x) == 1L
  )
  .val <- .Call(C_fn, .call = match.call(), x)
  if (inherits(.val, "rust_condition_value") && isTRUE(attr(.val, "__rust_condition__")))
    return(.miniextendr_raise_condition(.val, sys.call()))
  .val
}
```

We don't yet have per-line numbers for:

1. `stopifnot(...)` with 2 assertions
2. `match.call()` evaluation + LANGSXP allocation
3. The `inherits()` + `isTRUE(attr(...))` post-call guard
4. The implicit R function-call setup itself (arg matching, env creation)

### C-side wrapper (~125 ns aggregate)

The `with_r_unwind_protect` C wrapper does:

1. `Box::into_raw(Box::new(CallData{...}))` heap alloc
2. `get_continuation_token()`
3. Outer `catch_unwind(AssertUnwindSafe(|| R_UnwindProtect_C_unwind(...)))`
4. R-side `R_UnwindProtect_C_unwind` (trampoline + cleanup_handler install)
5. Inner `catch_unwind(AssertUnwindSafe(f))` for the user closure
6. `Box::from_raw(data)` reclaim
7. `drain_log_queue_if_available()` (compile-time no-op without `log`)

We don't know how the 125 ns splits across these. The first 6 are pure
Rust ABI cost and could in principle be inlined/stack-allocated; #4 is
the only one that's structurally required (it's R's API).

### TryFromSexp + IntoR (~1.3 μs aggregate per scalar)

- Per-element copy cost is sub-nanosecond. **Where is the 1.3 μs going?**
- Hypothesis: trait-method dispatch via vtable + branching on type-check
  (`is.numeric || is.logical || is.raw`) + NA detection (`x == NA_INTEGER`)
- Need: per-impl cost in isolation. Strip `with_r_unwind_protect` etc.

### Error-path (~14 μs over `stop()`)

- We know `make_rust_condition_value` allocates a 4-element VECSXP +
  class STRSXP + `__rust_condition__` attribute. **Which allocation is
  the bottleneck?**
- We know the R wrapper then does `stop(structure(...))` which itself
  allocates another condition object. **Could the C side raise directly?**

### Class system (~1–8 μs over bare fn)

- R6 method dispatch was 2.1 μs, S7 was 9 μs. **Why is S7 4× slower?**
  Need to walk through `S7::S7_dispatch()` once to identify the cost
  driver — is it argument matching, validator running, class ancestry
  walk, or method-table lookup?

## 2. Measurements to close those gaps

| # | Measurement                                                                                                                                              | How                                                                                                                                                                                                                                                                                                                                  | Outcome                                                                                |
|---|---|---|---|
| M1 | Per-line R-wrapper cost breakdown                                                                                                                       | Hand-write 5 R wrappers around `.Call(C_conv_sexp_arg, NULL, x)`, each missing one piece (no `match.call`, no `stopifnot`, no post-call guard, etc.). `bench::mark` each.                                                                                                                                                            | Knowing whether `match.call` (~400 ns?), `stopifnot` (~700 ns?), or post-check (~300 ns?) is the biggest item. Determines which one is worth a fast-path option first. |
| M2 | C-wrapper layer attribution                                                                                                                              | Build `analysis/scaffolding-c-attribution.rs` — small Rust bin that calls (a) bare `R_UnwindProtect_C_unwind`, (b) `with_r_unwind_protect` with empty body, (c) variant with stack-alloc'd CallData via `MaybeUninit`. Bench via `divan`. Optionally Instruments / `cargo flamegraph`.                                              | Knowing whether `Box<CallData>` alloc is meaningful or noise. If it's <20 ns, skip; if it's 50+ ns, prototype a stack variant. |
| M3 | TryFromSexp / IntoR isolation                                                                                                                            | Extend `miniextendr-bench/benches/from_r.rs` and `into_r.rs` (already exist) — bench `TryFromSexp::try_from_sexp(x)` for `i32`/`f64`/etc directly, no wrapper. Subtract from full-wrapper number to validate the ~1.3 μs.                                                                                                              | Confirm whether dispatch infra or per-element work dominates. |
| M4 | Error-path attribution                                                                                                                                   | Add benches that exercise `make_rust_condition_value` alone (no R wrapper) vs full `demo_error` flow vs direct `Rf_error()` (test-only). Compare allocation count via `bench::mark`'s `mem_alloc`.                                                                                                                                  | Pinpoint whether the cost is the C-side tagged SEXP build or the R-side `stop(structure(...))` re-raise. |
| M5 | S7 vs R6 dispatch deep-dive                                                                                                                              | `Rprof()` over a tight loop of `s7_value(obj)` calls; extract the call-graph hot path inside `S7_dispatch`. Compare to R6's `$value()` resolution.                                                                                                                                                                                  | Decide whether to recommend caching the resolved method in user code, or whether miniextendr could emit a hand-cached dispatch closure. |
| M6 | Knob impact upper bound                                                                                                                                  | Hand-edit one generated wrapper in `rpkg/R/miniextendr-wrappers.R` to drop `match.call()`, `stopifnot`, and post-call check. Re-bench. **This is a one-off — not a real codegen change** — to find the *theoretical maximum speedup* if we added all knobs.                                                                            | Sets the ceiling. If hand-stripped wrapper is still >500 ns we're chasing ghosts. |
| M7 | Multi-arg scaling under different decode paths                                                                                                          | Extend bench: fns with 2, 3, 5 args of mixed types (i32 + f64 + String + Vec<f64>). Bench each vs `n×` single-arg cost. Looking for per-arg overhead beyond `TryFromSexp`.                                                                                                                                                          | Confirm linearity or find argument-count-dependent cost (likely none, but cheap to verify). |
| M8 | Worker hop cost                                                                                                                                          | Force `force_worker = true` on a copy of `conv_i32_arg`, bench worker round-trip. Subtract main-thread cost to isolate the worker hop.                                                                                                                                                                                              | Quantify what `worker` costs; potentially recommend it for I/O-bound fns only. |

## 3. Candidate new options (post-measurement)

Ranked by *expected ROI* (estimated speedup × likelihood of safety). To be
proposed only after the matching measurement confirms the gain is real.

### A. `#[miniextendr(no_call_attribution)]` — **probably high ROI**

Codegen change: emit `.Call(C_fn, .call = NULL, x)` instead of
`.Call(C_fn, .call = match.call(), x)`. Internal infrastructure already
exists (`DotCallBuilder::null_call_attribution`) — it's currently only
used for R6 finalizer / S7 lambda contexts.

- **Effect**: drops `match.call()` LANGSXP alloc. M1 will tell us how big.
- **Trade-off**: error `.call` slot falls back to `sys.call()` of the
  wrapper invocation rather than the user's literal call. Users see
  `error in conv_i32_arg(...)` instead of `error in conv_i32_arg(42L)`.
  Acceptable for hot-path internal fns.
- **Compat**: behaviour-preserving for the success path. Error UX
  degrades slightly. Decide if `internal` should imply this.

### B. `#[miniextendr(no_preconditions)]` — **probably high ROI**

Codegen change: drop the `stopifnot(...)` block. `TryFromSexp` already
raises a typed Rust error if the SEXP is the wrong type/length, so the
information isn't lost — just routed through Rust's error message rather
than R's stopifnot text.

- **Effect**: drops 2–N assertions per call.
- **Trade-off**: error messages come from Rust's `TryFromSexp`
  formatting, e.g. `"failed to convert parameter 'x' to i32: wrong type,
  length, or contains NA"`. Slightly less helpful than R's "must be
  numeric, logical, or raw" for misuse, but more precise about the
  actual failure.
- **Compat**: behaviour-preserving. Could be implied by `internal`.

### C. `#[miniextendr(infallible)]` — **conditional ROI**

Annotation that asserts the function body cannot panic (no `?`, no
`unwrap`, no `panic!`). Codegen change: skip both the
`with_r_unwind_protect` wrapping in C AND the
`inherits(.val, "rust_condition_value")` check in R.

- **Effect**: removes the entire C-side unwind machinery and the R-side
  post-call branch.
- **Trade-off**: a buggy user fn that panics will abort or unwind into R
  unprotected. Probably needs `unsafe(infallible)`.
- **Compat**: dangerous; restrict via `unsafe(...)` syntax (parallel to
  the existing `_unchecked` FFI variants).

### D. `#[miniextendr(stack_callbox)]` — **probably low ROI**

C-side optimization: replace `Box<CallData>` with stack-allocated
`MaybeUninit<CallData>` for fns whose closure satisfies a sized bound.
Skip if M2 says heap alloc is <20 ns.

### E. `#[miniextendr(borrow_args)]` — **conditional ROI, complex**

Generate the C wrapper with `&[T]` / `&str` slice views instead of
copying into `Vec<T>` / `String`. Already partially supported (per
CLAUDE.md: `&[T]` / `&str` args work without lifetime annotations), but
the conversion still allocates internally. If we can keep the SEXP
protected for the closure scope, we can skip the copy.

- **Effect**: linear-time savings on `Vec<T>` arg decode (no copy).
- **Trade-off**: requires SEXP to outlive the call, which it does inside
  `with_r_unwind_protect`. Requires careful Send / GC discipline.
- **Compat**: only when body doesn't store the slice across allocations.

### F. `#[miniextendr(error_direct)]` — **probably medium ROI**

When the body raises an `RCondition::Error`, skip building the tagged
4-element SEXP and instead call `Rf_error` directly with a pre-built
class vector. Reverses the *current* design which prefers tagged SEXP
for traceability.

- **Effect**: 14 μs → ~5–7 μs error path (estimate).
- **Trade-off**: loses the R-side switch-based class layering. Would
  need a way to convey the `class = c(...)` vector through the C side.
- **Compat**: ABI-affecting; not enabled by default.

### G. Per-param `#[miniextendr(trust)]` — **probably low ROI individually**

Skip *only the precondition for this arg*. Useful when one arg is
trusted (e.g. passed by another miniextendr fn) but others aren't.

- **Effect**: drops 1–3 lines of `stopifnot` per arg.
- **Trade-off**: same as B, but per-arg granularity.

### H. Fast-path bundles

Probably the cleanest UX: instead of N independent knobs, ship 2–3
pre-baked bundles.

- `#[miniextendr(fast)]` = `no_call_attribution` + `no_preconditions` +
  the implied per-param trust. Drops in 1–2 μs.
- `#[miniextendr(internal_fast)]` = same + `internal`. For private
  framework fns where UX errors don't matter.
- `unsafe miniextendr(infallible_fast)` = `fast` + `infallible`. The
  "I know what I'm doing" mode.

## 4. Decision points needing user input

1. **Scope of "fast"**: is the goal to make user code faster, or to make
   the framework's own internal calls cheaper (e.g. cross-package trait
   ABI dispatch)? They want different knobs.
2. **Default policy**: should any of A–H become *the default* for `internal`
   functions (which don't need user-facing error UX)?
3. **Naming**: prefer N independent knobs (`no_call_attribution`,
   `no_preconditions`, ...) or H-style bundles (`fast` / `fast_unsafe`)?
4. **Error semantics for "fast" mode**: degrade the R `.call` slot to
   `sys.call()` (option A) or remove it entirely?
5. **Worker thread interaction**: most options assume main thread. Do
   we want a `worker_fast` flavor or is that overkill?
6. **Cargo-feature default**: should `default-strict` / `default-coerce`
   gain a `default-fast` sibling for project-wide opt-in?

## 5. Suggested order of attack

Recommendation: spend ~1 day on M1 + M6 first. Those two together set
the ceiling and tell us which knob to build first. The rest of the
measurements only make sense if M1+M6 say the gain is worth it.

```
M1  ──┐
      ├─→ pick the first knob (likely A or B)
M6  ──┘
M2 ──→ informs whether D is worth pursuing
M3 ──→ informs whether per-impl tuning of TryFromSexp is worth pursuing
M4 ──→ informs whether F is worth pursuing
M5 ──→ documentation note only (probably)
M7 ──→ confidence check
M8 ──→ separate axis (worker), independent of the above
```

## 5b. M1 + M6 RESULTS (run 2026-05-20)

Script: `analysis/scaffolding-strip-bench.R`
Raw output: `analysis/scaffolding-strip-output.txt`

### Variants (SEXP passthrough — no TryFromSexp work)

| Variant | min_ns | Δ vs A_full | What it has |
|---|---:|---:|---|
| A_full | 2583 | 0 | current generated wrapper |
| B_no_stopifnot | 1435 | −1148 | full minus `stopifnot()` |
| C_no_matchcall | 1476 | −1107 | full minus `match.call()` (.call = NULL) |
| D_no_postcheck | 2460 | −123 | full minus inherits/attr check |
| E_no_stop_no_call | 287 | −2296 | only post-check |
| F_only_matchcall | 1312 | −1271 | only match.call |
| **G_bare** | **205** | **−2378** | bare `.Call(C_, NULL, x)`, no wrapper |
| H_inline_call | 164 | −2419 | inline `.Call`, no closure |

### Variants (i32 path — with TryFromSexp + IntoR)

| Variant | min_ns | Notes |
|---|---:|---|
| i32_full | 2624 | macro-generated |
| **i32_bare** | **246** | `.Call(C_i32, NULL, x)` only |

### Per-line attribution

| Piece | Cost (min ns) | % of full |
|---|---:|---:|
| `stopifnot()` (2 assertions) | **1230** | **44 %** |
| `match.call()` on success path | **1148** | **41 %** |
| `inherits()/attr()` post-check | 123 | 4 % |
| **Sum of pieces** | **2501** | **92 %** |
| **G_bare floor (.Call only)** | **205** | 8 % |

Pieces are roughly additive: 1230 + 1148 + 123 = 2501 ≈ A_full − G_bare = 2378
(2 % drift, within bench noise).

### Confirmed: TryFromSexp/IntoR is fast

i32_bare − sexp_bare = 246 − 205 = **41 ns** for the full `TryFromSexp<i32>` +
`IntoR<i32>` round-trip. The first-pass measurement attributed ~1.3 μs to
conversions, but that was *wrapper overhead masquerading as conversion cost*.

### Confirmed: match.call() can become opt-in

The sanity probe ran the same `demo_error()` body through both
`.call = match.call()` and `.call = NULL`. Both produced:
- identical message text
- identical class layering (`rust_error`, `simpleError`, `error`, `condition`)
- a usable `call` slot — just positional (`demo_with_null_call("...")`) vs
  named (`demo_with_match_call(msg = "...")`)

For typical user error reporting the difference is cosmetic; the error
remains attributed to the user's invocation, not an internal frame.

### Implications for option ranking

The first-pass plan ranked `no_call_attribution` (A) above `no_preconditions`
(B). **The numbers reverse that**:

1. **`no_preconditions` (B): 1230 ns gain** — biggest single win
2. **`no_call_attribution` (A): 1148 ns gain** — second, and confirmed
   semantically safe to make default
3. `no_postcheck` (only with `infallible`): 123 ns gain — small but cheap
4. `infallible` (C): unlocks #3 plus the unwind-protect machinery itself
   (currently bundled into the 205 ns floor — would need M2 to separate)

### Theoretical ceiling

- Combined fast bundle (A+B+C, no postcheck either) → **205 ns** floor.
  **~13× speedup** over today's 2583 ns wrapper.
- With TryFromSexp/IntoR (i32 path): **246 ns** floor. ~11× speedup.
- The 205 ns floor includes: `.Call` symbol resolve + `with_r_unwind_protect`
  setup + return path. M2 (C-side flamegraph) would attribute *that*.

## 6. Open questions for the user

- **Are there specific user packages or hot paths** where these calls
  show up in profiling? If so, what's the workload? (Knowing the workload
  shape affects which knob matters — e.g. if it's 1M scalar calls,
  knobs A+B dominate; if it's a few large Vec calls, E dominates.)
- **How much UX degradation is acceptable** for the fast-path knobs?
  Specifically: is losing precise `error$call` information (option A)
  acceptable to ship as the default for `internal`?
- **Do we have a perf-budget target** (e.g. "scalar wrapper should be
  <500 ns") or are we just minimising?
- **Is M6 (hand-strip a wrapper to find the ceiling) worth doing first**
  or do you want to commit to a knob now and measure after?
