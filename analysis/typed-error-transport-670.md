# Typed-error transport for `Result<T, E>` (#670)

**Status:** design analysis. Scoping the proposal in #670 against the *current*
codegen. Gated on #665 (the R-side half).

**Date:** 2026-06-09

## TL;DR

The proposal in #670 — "let `Result<T,E>` build the tagged SEXP directly on the
`Err` path *without* `std::panic::catch_unwind`" — is **partly already done and
partly mis-scoped against today's code**:

- The `Err` arm **already** calls `make_rust_condition_value` directly. It does
  **not** `panic_any` the error. (Issue text "routes through panic transport"
  was true of a pre-#344 design; it is no longer true — see STILL_VALID below.)
- What remains is that the whole wrapper body (both `Ok` and `Err` arms) is
  enclosed in `with_r_unwind_protect`, i.e. `run_r_unwind_protect`, which sets
  up an `R_UnwindProtect` frame + a `catch_unwind` trampoline on **every** call.
  That fixed scaffolding cost is paid on the happy path too, not just on error.
- So the realistic win is **not** "skip panic infra on the error path" (the
  error path doesn't panic). It is **"skip the `R_UnwindProtect`/`catch_unwind`
  frame entirely for functions whose body is panic-free"** — which a
  `Result<T,E>` signature does **not** guarantee, because `?`, indexing,
  `.unwrap()`, integer overflow, allocation failure, and any called code can
  still panic inside the body.

Conclusion: the *headline* mechanism in #670 (a typed-error path that sidesteps
the panic machinery) is sound only if we keep a panic guard around the body for
genuine panics. Once you keep that guard, the `R_UnwindProtect`/`catch_unwind`
frame is back and the projected ~5.7 μs saving largely evaporates. The
defensible, smaller win is to replace the *R-API-using* `R_UnwindProtect`
(`with_r_unwind_protect`) with a *pure-Rust* `catch_unwind` guard for bodies
that provably make no R allocations that can longjmp — but that is a narrow
class and a separate analysis. **Recommendation: not worth it as specified;
land #665 first, re-measure with a committed bench, then reconsider a narrower
"panic-only guard, no R_UnwindProtect" knob.** See "Recommendation" at the end.

---

## 1. Current error-path data flow (grounded)

For a standalone `#[miniextendr] fn f(...) -> Result<T, E>` the macro lowers
through, in order:

1. `miniextendr-macros/src/lib.rs:943-958` — picks `ReturnHandling`. Without
   `unwrap_in_r`, `detect_return_handling_standalone_fn(output)` classifies
   `Result<T,E>` into one of `ResultUnit` / `ResultSexp` / `ResultIntoR` /
   `ResultNullOnErr`.
2. `miniextendr-macros/src/c_wrapper_builder.rs:375-456`
   (`generate_main_thread_wrapper`) — emits the `extern "C-unwind"` C wrapper.
   The body is built by `generate_return_handling` (`c_wrapper_builder.rs:570`)
   and the **entire** body is wrapped in
   `with_r_unwind_protect(|| { …conversions…; …return_handling… }, Some(call))`.
3. `generate_return_handling`'s `ResultIntoR` arm (`c_wrapper_builder.rs:685`)
   emits:

   ```rust
   let __result = f(arg_0, …);          // the user call — may panic
   let __result = match __result {
       Ok(v) => v,
       Err(e) => return make_rust_condition_value(   // <-- direct, no panic
           &format!("{:?}", e), kind::RESULT_ERR, None, Some(__miniextendr_call),
       ),
   };
   IntoR::into_sexp(__result)
   ```

   The standalone-fn analogue lives in
   `return_type_analysis.rs:211-293` (`analyze_result_type`) and produces the
   same `match … Err(e) => make_rust_condition_value(…)` shape.

4. At runtime, `with_r_unwind_protect` (`miniextendr-api/src/unwind_protect.rs:541`)
   calls `run_r_unwind_protect` (`unwind_protect.rs:260`), which:
   - `Box::into_raw`s a `CallData`,
   - sets up the global continuation token (`get_continuation_token`),
   - calls `R_UnwindProtect_C_unwind(trampoline, …, cleanup_handler, …, token)`,
   - the `trampoline` does its own inner `catch_unwind(AssertUnwindSafe(f))`,
   - reclaims the `Box`, drains the log queue, and returns the SEXP (which on
     the `Err` arm is the tagged condition value built in step 3).

5. The generated R wrapper (`method_return_builder.rs:64`, `standalone_body`)
   runs:

   ```r
   .val <- .Call(C_f, .call = match.call(), …)
   if (inherits(.val, "rust_condition_value") && isTRUE(attr(.val, "__rust_condition__")))
     return(.miniextendr_raise_condition(.val, sys.call()))
   .val
   ```

   `.miniextendr_raise_condition` (`registry.rs:738`) `switch`es on `.val$kind`
   and calls `stop(structure(..., class = c("rust_error", …)))`. **This R-side
   `stop()` re-raise is the ~7 μs half that #665 targets — not #670.**

### Where the cost actually is

The issue decomposes the C-side ~7 μs into "L2 (`panic!()` +
`with_r_unwind_protect`) = 5833 ns vs L1 (`make_rust_condition_value` alone) =
125 ns → 5708 ns gap = cost of `panic_any` transport."

That benchmark (`L2` deliberately `panic!()`s) measures the **panic path**, not
the `Result::Err` path. On a `Result<T,E>` return the `Err` arm **does not
panic** — it `return`s `make_rust_condition_value(...)` directly (step 3). So
on the Err path the 5708 ns is **already not paid** for the panic-payload
alloc / `panic_payload_to_string` / panic-hook. What *is* still paid on every
call (Ok or Err) is the `run_r_unwind_protect` fixed cost:

| Cost component | Paid on Ok? | Paid on Err? | Removable by #670 as specified? |
|---|---|---|---|
| `Box<dyn Any+Send>` panic payload alloc | only on real panic | no | n/a (Err doesn't panic) |
| `panic_payload_to_string` | only on real panic | no | n/a |
| panic-hook (`panic_telemetry::fire`) | only on real panic | no | n/a |
| `R_UnwindProtect_C_unwind` frame setup | **yes** | **yes** | only if guard removed |
| inner `catch_unwind` trampoline setup | **yes** | **yes** | only if guard removed |
| `Box::into_raw`/`from_raw` of `CallData` | **yes** | **yes** | only if guard removed |
| `get_continuation_token` (after 1st call) | cheap (OnceLock) | cheap | — |
| `make_rust_condition_value` (4 SEXP allocs + protects) | no | **yes** | no (still needed) |

So #670's "skip `catch_unwind` on the Err path" is only meaningful if we also
remove the `R_UnwindProtect` frame — and that frame is what protects against
**genuine panics in the body**, which a `Result` return type does not preclude.

## 2. STILL_VALID check

- **Does an erroring `Result<T,E>` function still go through `panic_any` +
  `catch_unwind`?** **No** — the `Err` arm calls `make_rust_condition_value`
  directly (`c_wrapper_builder.rs:692`, `return_type_analysis.rs:264/276/287`).
  The only `catch_unwind` involvement is the *blanket* `run_r_unwind_protect`
  frame that wraps every wrapper body regardless of return type. There is no
  `panic_any` of the user's `E`. (The issue's "routes through panic transport"
  is stale; the panic-transport path is reserved for `panic!()` / `error!()`,
  not `Result::Err`.)
- **Does `unwrap_in_r` already bypass `with_r_unwind_protect`?** **No, and it's
  orthogonal.** `unwrap_in_r` is the *opposite* of an error boundary: it passes
  the whole `Result<T,E>` to R as a value via `IntoR for Result` (→
  `list(error = …)`, `into_r/result.rs:69`). It still runs inside
  `with_r_unwind_protect`; it just changes `Err` from "raise a condition" to
  "return a list". The macro CLAUDE.md confirms: "`unwrap_in_r` is semantically
  distinct ... and orthogonal to transport."

**Net STILL_VALID:** the *numeric* premise (panic infra dominates the error
path) is **not valid for `Result::Err`** — only for `panic!()`/`error!()`. The
*structural* observation (every wrapper pays a fixed `R_UnwindProtect` +
`catch_unwind` setup cost) is valid and is the only thing #670 could remove.
The referenced bench `miniextendr-bench/benches/error_path_attribution.rs` and
`analysis/scaffolding-perf-roadmap.md` / `scaffolding-deep-findings-2026-05-20.md`
**do not exist in the repo or its git history** — the L1/L2/L3 numbers are
uncommitted M4 figures and cannot be reproduced from the tree as-is. Any
implementation must first re-establish a committed bench.

## 3. Proposed typed-error-transport codegen

### What #670 literally asks for

> emit a C wrapper that, on `Err`, builds the tagged SEXP directly without
> `std::panic::catch_unwind`, on `Ok` returns the value normally.

The Err-arm half is **already implemented** (§1.3). The novel part is dropping
`with_r_unwind_protect` from the body. Before/after for `fn f(x:i32) ->
Result<i32, MyError>`:

**Today (`ResultIntoR`):**

```rust
#[no_mangle]
pub extern "C-unwind" fn C_f(__miniextendr_call: SEXP, x: SEXP) -> SEXP {
    with_r_unwind_protect(|| {                 // <-- R_UnwindProtect + catch_unwind frame
        let x = <i32 as TryFromSexp>::try_from_sexp(x, "x") /* panics on bad arg */;
        let __result = f(x);                   // user body — may panic (?, unwrap, overflow…)
        let __result = match __result {
            Ok(v) => v,
            Err(e) => return make_rust_condition_value(
                &format!("{e:?}"), kind::RESULT_ERR, None, Some(__miniextendr_call)),
        };
        IntoR::into_sexp(__result)
    }, Some(__miniextendr_call))
}
```

**Proposed "typed-error" path (`#[miniextendr(typed_error)]` or auto for
`Result` returns):**

```rust
#[no_mangle]
pub extern "C-unwind" fn C_f(__miniextendr_call: SEXP, x: SEXP) -> SEXP {
    // Still need a panic catch for genuine panics inside conversions / body /
    // make_rust_condition_value's R allocations. But it can be a *pure-Rust*
    // catch_unwind rather than an R_UnwindProtect frame, IF the body provably
    // never triggers an R longjmp that must run Rust destructors.
    let r = std::panic::catch_unwind(AssertUnwindSafe(|| {
        let x = <i32 as TryFromSexp>::try_from_sexp(x, "x");      // may panic
        match f(x) {                                              // may panic via ?
            Ok(v) => IntoR::into_sexp(v),                         // happy path: NO R_UnwindProtect
            Err(e) => make_rust_condition_value(                 // typed error: direct, no panic
                &format!("{e:?}"), kind::RESULT_ERR, None, Some(__miniextendr_call)),
        }
    }));
    match r {
        Ok(sexp) => sexp,
        Err(payload) => make_rust_condition_value(               // genuine panic → tagged value
            &panic_payload_to_string(&*payload), kind::PANIC, None, Some(__miniextendr_call)),
    }
}
```

The only thing removed vs today is the `R_UnwindProtect_C_unwind` frame
(`run_r_unwind_protect`); a `catch_unwind` is retained. **This is the crux of
the design and its biggest hazard:** `with_r_unwind_protect` exists precisely so
that an **R longjmp** raised by an R API call *inside* the body (e.g.
`Rf_allocVector` triggering an allocation error, a coercion that errors,
`R_CheckUserInterrupt`) unwinds through Rust drops via `R_ContinueUnwind`
instead of skipping them. A bare `catch_unwind` does **not** catch an R
longjmp — it would let R `longjmp` straight past every Rust destructor on the
stack (the exact UB MXL300 forbids). So the proposed wrapper is only sound if
the body makes **no R API call that can longjmp** — and `TryFromSexp`
conversions, `IntoR::into_sexp`, and `make_rust_condition_value` itself all call
`Rf_*` allocators that can longjmp under memory pressure.

**Therefore the "no R_UnwindProtect" path is unsafe for the general case.** It
could be made safe only for a body that touches no R allocator on the success
path — which excludes essentially every real conversion. This is the
load-bearing reason #670 is "scoping unknown."

### Realistic, safe variant

Keep `with_r_unwind_protect` (R longjmp safety), but special-case the **Err
construction** so it never enters the panic machinery — which it already
doesn't. There is no further C-side win available without giving up R-longjmp
safety. The remaining lever is purely the **R-side** re-raise, which is #665.

## 4. The `MiniextendrError` trait decision

Today `Result<T,E>` only requires `E: Debug` (the Err arm does `format!("{e:?}",
e)`, `c_wrapper_builder.rs:692`). `unwrap_in_r` requires `E: Display`
(`into_r/result.rs:72`). `AsRError<E: std::error::Error>`
(`condition.rs:541`) is the opt-in for `?`-friendly value propagation.

If a typed-error path were built, the trait question is whether to enrich the
tagged value with **class** and **kind** from `E` (so R callers can
`tryCatch(my_rust_error = …)`), which `format!("{e:?}")` cannot do. Options:

| Option | Bound | Gives | Cost |
|---|---|---|---|
| **A. Keep `E: Debug`** | none new | message only, `kind = result_err`, no class | zero churn; status quo |
| **B. `E: Display`** | widens current `Debug` | nicer message, still no class | breaks fns relying on `Debug`-only `E`; no class still |
| **C. New `MiniextendrError` trait** | `fn message(&self)->Cow<str>; fn class(&self)->Option<&str>; fn kind(&self)->&str` | full class layering into `make_rust_condition_value(class=…)` | new trait, blanket impl for `Display`/`Error`, coherence work |
| **D. Reuse `std::error::Error` via `AsRError`** | `E: Error` | message; `source()` chain; no native class | already exists; class still absent |

`make_rust_condition_value` already takes `class: Option<&str>` and `kind:
&str`, so **Option C composes cleanly**: the Err arm would become
`make_rust_condition_value(&e.message(), e.kind(), e.class(), Some(call))`. A
blanket `impl<E: std::error::Error> MiniextendrError for E` (message via
`Display`, `class = None`, `kind = RESULT_ERR`) preserves today's behaviour and
lets users opt into richer errors by hand-implementing the trait — same shape
as `error!(class = "…")` already supports. **This is the only part of #670 with
clear standalone value** (it improves `tryCatch` ergonomics) and is independent
of the perf claim.

Tradeoff: introducing `MiniextendrError` risks a coherence collision with the
existing `IntoR for Result<T, E: Display>` (`unwrap_in_r`) and with `AsRError`.
It would need to be a distinct, sealed-ish trait used only by the macro's Err
arm, never a blanket that competes with `IntoR`.

## 5. `?`-operator interaction

`?` inside the body desugars to `return Err(From::from(e))`, so a `Result`
return with `?` propagates as a normal `Err` and lands in the macro's Err arm —
**no panic involved**, already handled. The hazard is `?` (or `.unwrap()`,
indexing, overflow, `From` conversions, called library code) that **panics**
rather than returns `Err`. The wrapper must therefore *always* keep a panic
guard regardless of return type. The two coexist exactly as today:

- `Err(e)` (including `?`-propagated) → macro Err arm → `make_rust_condition_value(kind=result_err)`.
- `panic!`/`unwrap`/overflow → caught by the body guard → `make_rust_condition_value(kind=panic)`.

Any "typed error" knob **must not** drop the panic guard. That is the design
contradiction at the heart of #670: you cannot both "skip the panic
infrastructure" and "keep the body panic-safe." You can only skip it for the
*value construction* of `Err` — which is already skipped.

## 6. Cross-package trait-ABI implications

Trait-ABI method shims route through `with_r_unwind_protect_shim`
(`unwind_protect.rs:483`) which returns a tagged SEXP that the consumer's View
method re-panics via `repanic_if_rust_error` into the consumer's *outer*
`with_r_unwind_protect`. The C ABI of an error-returning exported fn is "returns
a `SEXP` that is either the value or a `rust_condition_value`." A typed-error
path that **kept** this return contract (Err arm builds the same tagged SEXP)
would be **ABI-compatible** — the producer and consumer already speak
"tagged-SEXP-or-value." If instead a typed-error path tried to call `Rf_error`
directly from C (the #665 `error_direct` idea), it would **break** the shim
contract, because the shim's whole point is to *not* longjmp across the package
boundary (it returns a tagged SEXP so the consumer's guard owns the longjmp).
So: option C (richer tagged value, same ABI) is cross-package safe; any
"longjmp from C" variant is not.

## 7. Dependency on #665 + phased plan

#665 (R-side `error_direct`: call `Rf_error` from C with a pre-built class
vector, skip `.miniextendr_raise_condition` → `stop(structure(…))`) addresses
the R-side ~7 μs. It is the *easier, higher-confidence* half and is a
prerequisite for sensible measurement: until the R-side re-raise is removed, the
C-side frame is not the dominant cost on the `Result::Err` path (which doesn't
panic anyway).

Phased (flat priority):

1. **Land #665** (R-side direct raise). Independent; measurable; ~4–8 hrs.
2. **Commit a real `error_path_attribution` bench** under
   `miniextendr-bench/benches/` that separately times: (a) `Ok` return, (b)
   `Result::Err` return, (c) `panic!()`, (d) `error!()`. The current numbers are
   uncommitted; nothing in the tree reproduces them.
3. **Only if (2) shows the `Result::Err` path is dominated by the
   `R_UnwindProtect` frame** (not the R re-raise, not `make_rust_condition_value`'s
   SEXP allocs): prototype the `MiniextendrError` trait (option C) for richer
   classed errors — this ships value regardless of perf.
4. A "panic-only guard, no `R_UnwindProtect`" knob is **out of scope** unless we
   can statically prove a body makes no longjmping R call — which the macro
   cannot determine. Track separately if ever pursued.

## 8. Effort / risk

| Work item | Effort | Risk |
|---|---|---|
| `MiniextendrError` trait + blanket + Err-arm wiring | ~1–2 days | Medium (coherence vs `IntoR for Result`, vs `AsRError`); UI-test churn |
| Drop `R_UnwindProtect` for Result bodies | — | **Unacceptable** (R longjmp skips Rust drops; UB; MXL300 violation) |
| Committed error-path bench | ~half day | Low |
| Cross-package ABI: keep tagged-SEXP contract | included | Low if ABI unchanged; High if "longjmp from C" |

The headline perf win projected in #670 (~5.7 μs off the error path) is **not
achievable safely**, because it assumes removing the panic/longjmp guard, which
the body still needs. The achievable wins are: (a) #665's R-side ~7 μs, and (b)
ergonomic — classed typed errors — with negligible perf change.

## 9. Recommendation

**Not worth implementing #670 as specified ("skip the panic machinery on the
typed-error path"); the premise doesn't hold against current code** — the
`Result::Err` arm already builds the tagged SEXP without `panic_any`, and the
only remaining C-side cost is the `R_UnwindProtect`/`catch_unwind` frame that
**must** stay for body-panic and R-longjmp safety.

Concretely:

1. **Do #665 first** (R-side direct raise) — that is the real, safe C-vs-R
   saving and is the documented gate.
2. **Re-measure with a committed bench** distinguishing `Ok` / `Result::Err` /
   `panic!` / `error!`. The cited L1/L2 figures are uncommitted and the
   `Result::Err` path was never the panic path.
3. **Reframe #670** away from "skip panic infra" toward "richer typed errors via
   a `MiniextendrError` trait" (option C) — that has standalone ergonomic value
   (classed `tryCatch`) and is ABI-safe. If pursued, it is the actionable
   follow-up; otherwise #670 should be closed as superseded by #665 + this
   analysis.

The issue should **stay open as the design tracking item** (or be closed in
favour of the `MiniextendrError`-trait follow-up if the maintainer agrees the
perf framing is dead). It should **not** be auto-closed by this analysis.
