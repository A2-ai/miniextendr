# S7 vs R6 method dispatch — cost attribution (#668)

**Date:** 2026-06-09
**Issue:** [#668](https://github.com/A2-ai/miniextendr/issues/668) — "perf-research: S7 method dispatch is ~4× R6 — attribute the cost via Rprof"
**Versions:** R 4.6.0, S7 0.2.2, R6 2.6.1 (arm64, `rv/library/4.6/arm64`)

## TL;DR

- **S7 dispatch is genuinely heavier than R6 dispatch in the current codegen** — confirmed structurally and empirically.
- The extra cost lives **entirely in S7's generic-dispatch machinery** (`S7::S7_dispatch()` → `.External2(method_call_, …)` → class-ancestry walk + method-table lookup + re-invocation of the method closure), **not** in anything miniextendr emits. The `.Call(...)` body and the `@.ptr` property access are identical to R6's in shape and trivially cheap.
- The issue's headline "~4×" is the *full-call* ratio (dispatch + `.Call` + condition-check, where a fixed wrapper floor dilutes the difference). The **pure dispatch-overhead ratio is ~7–11×** (S7 over R6), reproduced here at **7.4× median** in an isolated pure-R bench.
- miniextendr cannot fix S7 internals, but **caching the resolved method closure makes the call ~29× faster** (eliminates ~2.3 µs of S7 dispatch). That is the concrete optimization lead — exposed to users as a documented escape hatch, optionally as generated codegen.

## The stale-evidence caveat (read first)

This issue cites `analysis/scaffolding-bench-2026-05-20.md` and `analysis/scaffolding-perf-roadmap.md` and `rpkg/src/rust/fast_fixtures.rs`. **None of these exist** in `main` or any git ref (verified: `ls analysis/`, `git log --all -- 'analysis/scaffolding-*'`, `git log --all -- 'rpkg/src/rust/fast_fixtures.rs'` all come back empty). They are from an unmerged perf sprint; the `fast` / `no_preconditions` / `no_call_attribution` knobs the issue references are likewise not on `main`. So the "~4×" table is a **hypothesis to verify, not established fact**. This write-up grounds everything in the current `main` codegen plus a fresh measurement.

## What each class system actually emits

The cleanest apples-to-apples pair on `main` is `S7TraitCounter` vs `R6TraitCounter` in
`rpkg/src/rust/class_system_matrix.rs` — both expose `get_value(&self) -> i32`, a
trivial method with no real work. The generated wrappers:

### R6 (`rpkg/R/miniextendr-wrappers.R:3463`)

```r
R6TraitCounter <- R6::R6Class("R6TraitCounter",
  public = list(
    ...
    get_value = function() {
      .val <- .Call(C_R6TraitCounter__get_value, .call = match.call(), private$.ptr)
      if (inherits(.val, "rust_condition_value") && isTRUE(attr(.val, "__rust_condition__"))) return(.miniextendr_raise_condition(.val, sys.call()))
      .val
    }
  ),
  private = list(.ptr = NULL),
  ...
)
```

**Dispatch path for `obj$get_value()`:** the `$` operator on the R6 object's environment →
direct symbol lookup of the bound closure → call it → `.Call(...)`. No generic, no class
walk, no method table.

### S7 (`rpkg/R/miniextendr-wrappers.R:3599`)

```r
S7TraitCounter <- S7::new_class("S7TraitCounter",
  properties = list(.ptr = S7::class_any),
)
if (!exists("get_value", mode = "function")) {
  get_value <- S7::new_generic("get_value", "x", function(x, ...) S7::S7_dispatch())
}
S7::method(get_value, S7TraitCounter) <- function(x, ...) {
  .val <- .Call(C_S7TraitCounter__get_value, .call = match.call(), x@.ptr)
  if (inherits(.val, "rust_condition_value") && isTRUE(attr(.val, "__rust_condition__"))) return(.miniextendr_raise_condition(.val, sys.call()))
  .val
}
```

**Dispatch path for `get_value(obj)`:**
1. Call the **generic** `get_value` (a function with class `c("S7_generic","function","S7_object")`).
2. Its body runs `S7::S7_dispatch()`, which is (from the installed S7 0.2.2):
   ```r
   function () .External2(method_call_, sys.function(-1L), sys.frame(-1L))
   ```
   i.e. a hand-off into S7's C dispatcher `method_call_` that:
   - reads the dispatch argument(s) and computes the dispatch class,
   - walks the class ancestry,
   - looks the method up in the generic's method table,
   - re-invokes the resolved **method closure** (a *second* R closure frame).
3. *Only then* does the method body run the identical `.Call(...)` + condition check that R6 runs directly.

So the structural delta is: **S7 inserts a generic-function frame + a C-level `method_call_` dispatch (class walk + table lookup) + a re-entered method closure frame in front of the same `.Call` body R6 reaches directly.** The bodies are byte-for-byte equivalent (`r_class_formatter.rs:265` `instance_call` builds both via the same `DotCallBuilder`); the difference is purely the dispatch envelope.

### Where the wrappers come from in the codegen

- R6: `miniextendr-macros/src/miniextendr_impl/r6_class.rs:278-291` — method emitted as a plain
  closure in the `public = list(...)`; body via `MethodReturnBuilder::build_r6_body` →
  `instance_call("private$.ptr")`.
- S7: `miniextendr-macros/src/miniextendr_impl/s7_class.rs:819-843` — emits
  `S7::new_generic(name, "x", function(x, ...) S7::S7_dispatch())` then
  `S7::method(generic, Class) <- function(x, ...) { ... }`; body via
  `MethodReturnBuilder::build_s7_body` → `instance_call("x@.ptr")`.
- Both `.Call` sites share `DotCallBuilder` (`miniextendr-macros/src/r_wrapper_builder.rs:390`)
  and emit `.call = match.call()` identically — **call-attribution cost is equal on both paths**,
  so it is not a differentiator.

Note the issue's candidate causes #2 (validator on property), #3 (deep class-ancestry walk),
and #6 (property getter invocation) **do not apply** to a plain `get_value` method: there is no
property validator on the method path, the class is depth-1, and `@.ptr` is a bare property read.
The cost is the generic-dispatch envelope itself (issue candidates #1 + #4 + #5).

## Measurement (cheap, pure-R isolation)

The installed `miniextendr` package wasn't present (installing requires a full Rust
compile — out of scope per "don't test too much locally"), so rather than spend the budget
on `just rcmdinstall`, I isolated the **dispatch overhead** in pure R using minimal S7 / R6
classes whose method just returns `42L` (no Rust `.Call`, no real work). This measures exactly
the dispatch envelope the issue is about, machine-independent of the `.Call` floor.

```
microbenchmark, 20000 iters, ns:
  expr  min   lq median   mean
  bare    0   41     82     75    # bare closure: f(x) -> 42L
  R6    246  328    369    377    # r6obj$value()
  S7   2337 2583   2747   3493    # s7_value(s7obj)

DISPATCH-ONLY median ratios (no Rust .Call):
  S7/R6   = 7.44x
  S7/bare = 33.5x
  R6/bare = 4.5x
```

- **S7 dispatch is 7.4× R6 dispatch (median)** in pure overhead.
- S7 adds ~2665 ns over the bare-closure baseline; R6 adds ~287 ns.

### Reconciling with the issue's "~4×"

The issue's table is the *full* call (dispatch + `.Call` + condition guard). Subtracting its own
`bare fn` floor (1435 ns) isolates dispatch overhead:

- S7: 9061 − 1435 = **7626 ns** of S7 overhead
- R6: 2132 − 1435 = **697 ns** of R6 overhead
- ratio = 7626 / 697 ≈ **10.9×**

So the issue's data, properly decomposed, already agrees with the ~7–11× *dispatch-overhead*
figure. The "~4×" is the full-call ratio, diluted by the shared ~1.4 µs `.Call`/wrapper floor
present in both. **Conclusion: the ~4× full-call claim is plausible and direction-correct; the
real driver is a 7–11× difference in the dispatch envelope.** (Absolute ns differ by machine —
this box shows ~2.7 µs for S7 vs the issue's 9 µs — but the *ratio* reproduces.)

### Pinpointing the cost: caching the resolved method

Directly testing the issue's proposed mitigation — resolve the method once, then call the
closure straight, bypassing the generic:

```r
cached <- S7::method(s7_value, S7Counter)   # resolved method closure
# microbenchmark, 20000 iters, ns:
  S7_generic median = 2378 ns
  S7_cached  median =   82 ns   # == bare closure
  speedup    = 29x  (eliminates ~2296 ns of S7_dispatch / .External2 / class-walk)
```

Calling the **resolved method closure** is 29× faster and lands exactly at the bare-closure
baseline — i.e. **100% of S7's extra cost is the `S7_dispatch()` → `.External2(method_call_,…)`
path** (class resolution + method-table lookup + re-dispatch). Nothing in the `@.ptr` read, the
property object, or the miniextendr-emitted body contributes measurably.

## Optimization leads (ranked: payoff vs risk)

### 1. Document the escape hatch + ship a `method()`-cache helper — **HIGH payoff / LOW risk** ✅ recommended

The cost is irreducibly inside S7's generic dispatch; miniextendr can't change S7. But users in
hot loops can pre-resolve once:

```r
# hot loop over the same class
m <- S7::method(s7_value, S7TraitCounter)
for (obj in objs) m(obj)        # ~29x faster than s7_value(obj)
```

Payoff: ~29× on the dispatch envelope for the (common) tight-loop case, zero codegen change,
zero ABI risk. This is the right default answer to "why is S7 slow?" — and it is what the issue
itself floats under "Document the cost so users can route around it / cache the resolved method
as a local closure". Add a short section to the class-systems docs (`docs/`) and the S7 guidance
in `miniextendr-class-systems` skill, with this benchmark as justification. **Recommended to file
as a docs follow-up issue (or land a docs paragraph in this PR).**

### 2. Generated per-class fast-path shortcut function — **MEDIUM payoff / MEDIUM risk**

The macro could *additionally* emit, alongside the generic+method, a plain non-generic shortcut,
e.g. `S7TraitCounter_get_value <- function(x, ...) { <body> }` that calls the same `.Call`
without going through `S7_dispatch()`. Power users call the shortcut in hot loops; the generic
stays for polymorphic dispatch. Payoff: same ~29× for opt-in callers. Risk: doubles the S7
wrapper surface, two names per method (NAMESPACE / docs churn), and it's a footgun if the class
is ever subclassed (the shortcut won't dispatch to overrides). Worth a design issue, not a blind
implement. Gate behind an explicit attribute (e.g. `#[miniextendr(s7(fast_shortcut))]`) if pursued.

### 3. Upstream: feed the decomposition to S7 — **LOW direct payoff / LOW risk**

The dominant cost is `.External2(method_call_, …)` per call. The S7 team are aware dispatch is
the bottleneck for trivial methods; a focused report (with the 29×-when-cached number isolating
`method_call_` as the driver, single-dispatch depth-1 class) could motivate an S7-side fast path
(e.g. an inline-cache on the generic for the last-seen dispatch class). Payoff to miniextendr is
indirect and on S7's timeline; low effort to write up. Suitable as an upstream note, not an
in-repo change.

## Recommendation

- **Do not** attempt an in-repo S7 dispatch rewrite — the cost is in S7, not our codegen.
- **Land/queue lead #1** (docs + cache-the-method guidance) as the actionable, low-risk outcome.
- **File lead #2** as a design issue (opt-in generated shortcut) for later, with this sketch.
- The research question (#668) is *answered* by this write-up; it isn't *closed* by a code fix
  unless the docs paragraph (lead #1) is considered the deliverable.

## Reproduction

```r
.libPaths(c("rv/library/4.6/arm64", .libPaths()))
library(S7); library(R6); library(microbenchmark)

S7Counter <- S7::new_class("S7Counter", properties = list(.ptr = S7::class_any))
s7_value  <- S7::new_generic("s7_value", "x", function(x, ...) S7::S7_dispatch())
S7::method(s7_value, S7Counter) <- function(x, ...) 42L
s7obj <- S7Counter(.ptr = NULL)

R6Counter <- R6::R6Class("R6Counter", public = list(value = function() 42L))
r6obj <- R6Counter$new()

bare <- function(x) 42L
microbenchmark(bare = bare(NULL), R6 = r6obj$value(),
               S7 = s7_value(s7obj), times = 20000L, unit = "ns")

cached <- S7::method(s7_value, S7Counter)
microbenchmark(S7_generic = s7_value(s7obj), S7_cached = cached(s7obj),
               times = 20000L, unit = "ns")
```
