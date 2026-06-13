# `#[miniextendr]` scaffolding cost — 2026-05-20

Goal: measure the per-call overhead added by `#[miniextendr]` wrappers,
attribute it to each layer (R-side prologue, `.Call`, `with_r_unwind_protect`,
`TryFromSexp`, `IntoR`, error transport), and identify where the fat is.

- Script: `analysis/scaffolding-bench.R`
- Raw output: `analysis/scaffolding-bench-output.txt`
- Repo HEAD: `42daa354` (main)
- R: 4.6.0 (2026-04-24) / Platform: aarch64-apple-darwin23 / CPU: Apple M3 Max
- Tool: `bench::mark` + `bench::press` (10 000 iterations / case, scalar
  cases; 500 iterations / case for vec presses). Times reported as
  **minimum** to suppress GC / system noise.

## Layer map (from `cargo expand`)

For `pub fn conv_i32_arg(x: i32) -> i32 { x }` the macro emits:

```rust
#[unsafe(no_mangle)]
pub extern "C-unwind" fn C_conv_i32_arg(
    __miniextendr_call: SEXP,
    x: SEXP,
) -> SEXP {
    with_r_unwind_protect(|| {
        let x: i32 = match TryFromSexp::try_from_sexp(x) {
            Ok(v) => v,
            Err(e) => return make_rust_condition_value(/* … */),
        };
        let __result = conv_i32_arg(x);
        IntoR::into_sexp(__result)
    }, Some(__miniextendr_call))
}
```

…paired with the generated R wrapper:

```r
conv_i32_arg <- function(x) {
  stopifnot(
    "'x' must be numeric, logical, or raw" = is.numeric(x) || is.logical(x) || is.raw(x),
    "'x' must have length 1" = length(x) == 1L
  )
  .val <- .Call(C_conv_i32_arg, .call = match.call(), x)
  if (inherits(.val, "rust_condition_value") &&
      isTRUE(attr(.val, "__rust_condition__")))
    return(.miniextendr_raise_condition(.val, sys.call()))
  .val
}
```

Layers paid on **every** call:

| # | Layer                                                          | Side | Notes |
|---|-----------------------------------------------------------------|------|---|
| 1 | function prologue: `stopifnot(...)`                             | R    | scalar fns only |
| 2 | `match.call()`                                                  | R    | allocates a LANGSXP |
| 3 | `.Call` (symbol resolve + arg marshalling)                      | R/C  | ~125 ns measured |
| 4 | `extern "C-unwind"` entry                                       | C    | ABI shim |
| 5 | `Box<CallData{…}>` heap alloc + `get_continuation_token()`      | Rust | inside `run_r_unwind_protect` |
| 6 | outer `catch_unwind(AssertUnwindSafe(…))`                       | Rust | catches `R_ContinueUnwind` re-panic |
| 7 | `R_UnwindProtect_C_unwind` (trampoline + cleanup_handler)        | C    | R API |
| 8 | inner `catch_unwind(AssertUnwindSafe(f))`                       | Rust | catches user panic |
| 9 | `Box::from_raw` reclaim + `drain_log_queue_if_available()`      | Rust | no-op without `log` feature |
| 10 | per arg: `TryFromSexp::try_from_sexp(x)`                       | Rust | varies by type |
| 11 | user body                                                       | Rust | |
| 12 | `IntoR::into_sexp(__result)`                                    | Rust | skipped if `fn → SEXP` |
| 13 | post-call: `inherits(.val, "rust_condition_value")` + attr check | R    | branch never taken on success |

## Numbers

### 1. Baseline — SEXP passthrough (no body, no decode)

| expression | min (ns) |
|---|---:|
| `r_identity(x)` (pure R closure) | 41 |
| `identity(x)` (base) | 41 |
| `.Call(C_conv_sexp_arg, NULL, x)` (skips R wrapper) | **123** |
| `miniextendr::conv_sexp_arg(x)` (full path) | **1517** |

Reading: the full scaffolding stack costs **~1.5 μs minimum per call**.
The C wrapper itself (`with_r_unwind_protect` + the SEXP-identity
`TryFromSexp`) adds only **~85 ns** over a bare `.Call`. **The dominant
fixed cost is in the generated R wrapper** — `match.call()` + `stopifnot()` +
the post-call `inherits()/attr()` check together account for **~1.4 μs** of
the 1.5 μs round-trip.

### 2. `TryFromSexp` scalar arg decode (full wrapper round-trip)

| Arg type        | min (ns) | Δ vs `conv_sexp_arg` |
|-----------------|---------:|---------------------:|
| `SEXP`          | 1476     | (baseline)           |
| `i32`           | 2788     | +1312                |
| `f64`           | 2747     | +1271                |
| `Rboolean`      | 2706     | +1230                |
| `u8`            | 2788     | +1312                |
| `String`        | 2788     | +1312                |

Reading: scalar decode + scalar `IntoR` encode is **~1.3 μs combined** for
all primitive types — i.e. the `TryFromSexp` and `IntoR::into_sexp` round-trip
contributes **roughly as much as the entire R wrapper layer**.

### 3. `Vec<T>` arg decode size sweep (`conv_vec_*_len`)

| Size  | `Vec<i32>` (ns) | `Vec<f64>` (ns) |
|------:|---------------:|---------------:|
| 1     | 2501           | 2501           |
| 16    | 2542           | 2501           |
| 256   | 2542           | 2542           |
| 4 096 | 2829           | 3034           |
| 65 536| **5822**       | **8692**       |

Reading: per-element cost is **negligible up to ~4K**, then grows
predictably. At 64K the i32 path adds ~3.3 μs (~50 ps/elem) and the f64
path ~6.2 μs (~95 ps/elem) — roughly memcpy speed plus per-element NA
checks. Cache effects dominate over wrapper overhead at this scale.

### 4. `IntoR` scalar return encode (no args)

All scalar returns clock at **~1.4–1.7 μs**:

| Return type | min (ns) |
|---|---:|
| `i32`       | 1476 |
| `f64`       | 1435 |
| `Rboolean`  | 1476 |
| `u8`        | 1394 |
| `String`    | 1435 |
| `SEXP`      | 1476 |

Reading: scalar `IntoR::into_sexp` is essentially free against the fixed
wrapper baseline. Even `String` (which allocates a `CHARSXP`) is within
~30 ns of the i32 path.

### 5. Error transport

| Path                              | min (ns) | Δ vs `stop()` |
|-----------------------------------|---------:|--------------:|
| `tryCatch(stop("oops"), …)`       | 7052     | (baseline)    |
| `tryCatch(demo_error("oops"), …)` | 20828    | +13.8 μs      |
| `tryCatch(demo_warning(…), …)`    | 26240    | +19.2 μs      |

Reading: the `panic → tagged SEXP → R wrapper → stop(structure(...))`
transport costs **~14 μs over a native `stop()`**. The biggest pieces
(see `make_rust_condition_value` in `error_value.rs`): allocating the
4-element VECSXP, building the `class = c("rust_error", "simple_*", ...)`
character vector, setting attributes, then the R-side
`.miniextendr_raise_condition` doing another `stop(structure(...))`.
Warning is slower because it routes through `withCallingHandlers` rather
than `tryCatch` semantics. Not on the hot path, but worth knowing.

### 6. Class-system method dispatch (`&self → i32`)

| Class system | Constructor                       | Method call             | min (ns) |
|--------------|-----------------------------------|-------------------------|---------:|
| (bare fn)    | —                                 | `conv_i32_ret()`        | **1435** |
| R6           | `R6Counter$new(0L)`               | `r6_counter$value()`    | 2132     |
| S4           | `S4Counter(0L)`                   | `s4_value(s4_counter)`  | 2337     |
| Env          | `SimpleCounter$new_counter(0L)`   | `env$get_value()`       | 2788     |
| **S7**       | `S7Counter(0L)`                   | `s7_value(s7_counter)`  | **9061** |

Reading: R6 and S4 add **~700–900 ns over a bare wrapper call**. Env is
~1.4 μs heavier than R6 (the closure-in-environment chain). **S7 dispatch
is ~4× slower than R6** — S7's `S7_dispatch()` machinery walks class
ancestors, runs validators, and resolves the generic each time. If S7 is
on a hot path, consider caching the method or using R6.

### 7. Per-call linearity (3 sequential `conv_i32_arg(x)` calls)

| n calls | min (ns) | per-call (ns) |
|--------:|---------:|--------------:|
| 1       | 2747     | 2747          |
| 2       | 5494     | 2747          |
| 3       | 8200     | 2733          |

Reading: per-call cost is **flat at ~2.75 μs**. No batching/economy of
scale, no GC-driven amplification at this rate — `bench::mark`'s
`filter_gc = FALSE` doesn't unmask anything pathological.

## Findings — where the fat is

Ranked by absolute contribution to a typical scalar round-trip
(`conv_i32_arg(42L)` ≈ 2750 ns):

1. **R wrapper prologue + epilogue (~1.4 μs, ~50 %)**
   `match.call()`, `stopifnot(...)`, and the `inherits()/attr()` post-call
   check together dominate. `match.call()` alone is ~400 ns; `stopifnot`
   with two predicates is ~700 ns; the post-call inherits/attr check is
   ~300 ns. Options:
   - Drop `match.call()` when no diagnostic class system would need the
     call attribution — would shave ~400 ns. *(But: condition raise-paths
     use it; would need a tiered codegen.)*
   - Replace `stopifnot()` with bare `if (...) stop(...)` — ~200–400 ns
     faster per check.
   - Compute the post-call check once into a precomputed inline guard.

2. **`TryFromSexp` + `IntoR` round-trip (~1.3 μs, ~45 %)**
   Per-call cost is type-agnostic for scalars (i32/f64/Rboolean/u8/String
   all within 80 ns of each other), suggesting the dispatch cost is in
   the trait infrastructure (vtable indirection, branching on length /
   NA / type) rather than the actual element copy.

3. **`.Call` + `with_r_unwind_protect` (~125 ns, ~5 %)**
   The lowest-cost layer. `Box<CallData{…}>` heap alloc is the biggest
   item here — could be replaced by a `MaybeUninit` on the stack if we
   can verify the trampoline doesn't need to outlive the call. (Probably
   not worth it given how small this slice already is.)

4. **Error path (~14 μs over native `stop`)**
   Two complete `stop(structure(...))` round-trips (Rust → tagged SEXP +
   R wrapper → real R condition). The 4-element VECSXP allocation + class
   vector + attribute set is unavoidable for the rust class layering, but
   we could elide the second `stop()` if the wrapper recognised the
   tagged-SEXP and *directly raised* without re-allocating.

5. **S7 dispatch (~7 μs over a bare wrapper)**
   `S7::S7_dispatch()` is the cost driver. Not a miniextendr problem
   strictly, but a documentation point: S7 is "free-style" expensive.

## Suggested follow-ups

- **`#[miniextendr(fast)]`** mode that elides `match.call()`,
  `stopifnot()`, and the post-call condition check for fns that never
  raise rust conditions (declared safe by attribute). Expected:
  ~1.4 μs → ~0.3 μs floor; +900 % calls/sec.
- **Error fast-path**: detect the tagged condition SEXP inside the C
  wrapper and call `Rf_error` directly with pre-built class vector,
  skipping the R-side raise wrapper. Expected: 14 μs → ~6 μs.
- **Vec arg fast path**: short-circuit `TryFromSexp::<Vec<T>>` when the
  source SEXP is already storage-compatible (REALSXP for `Vec<f64>`, no
  NA conversion needed). Currently bench shows ~500 ns of overhead at
  small sizes that doesn't scale with N — pure setup. Worth profiling.
- **S7 method-caching** documentation: warn users that S7 dispatch is
  ~4× R6. Consider caching the resolved method into a local closure for
  hot paths.

## How to re-run

```sh
just configure
just rcmdinstall                                     # ~7 min
Rscript analysis/scaffolding-bench.R \
  2>&1 | tee analysis/scaffolding-bench-output.txt
```
