# Call Convention Review: `#[miniextendr]` Standalone Functions

Date: 2026-03-07
Scope: Full R → C → Rust → R path for `#[miniextendr]` standalone functions

## Issues

### 1. RNG + non-error_in_r: `PutRNGstate` skipped on panic/R error

**Severity: Medium (bug)**

For `#[miniextendr(rng, no_error_in_r)]`, the generated code is:

```rust
extern "C-unwind" fn C_fn(...) -> SEXP {
    GetRNGstate();
    let __result = catch_unwind(|| {
        with_r_unwind_protect(|| { /* user code */ }, Some(call))
    });
    PutRNGstate();   // <-- may never execute
    match __result {
        Ok(sexp) => sexp,
        Err(payload) => resume_unwind(payload),
    }
}
```

If the user function panics, `with_r_unwind_protect` catches it and calls
`Rf_errorcall` which longjmps. The longjmp skips both `PutRNGstate()` and the
`catch_unwind`. Same for R errors inside `f` — `R_ContinueUnwind` longjmps
past `PutRNGstate`.

The `resume_unwind` path in the outer match is unreachable (all error paths
from `with_r_unwind_protect` diverge via longjmp), but the `PutRNGstate` skip
is real.

**Impact**: R's `.Random.seed` not updated. Next `GetRNGstate` replays the old
seed, producing repeated random sequences. Data correctness bug, not a crash.

**Scope**: Only `#[miniextendr(rng, no_error_in_r)]` — rare combination.
`error_in_r` (the default) returns tagged SEXP on panic, so `PutRNGstate`
runs normally.

**Fix options**:
- A: Move `GetRNGstate`/`PutRNGstate` inside the `R_UnwindProtect` closure
  (guaranteed to bracket the user code, but `PutRNGstate` would be skipped on
  R longjmp too — same as current behavior for error_in_r path since
  `R_ContinueUnwind` diverges)
- B: Use the error_in_r path unconditionally when `rng` is set (panic →
  tagged SEXP → PutRNGstate runs → return → R wrapper raises condition)
- C: Add a dedicated `with_r_unwind_protect_rng` that wraps
  Get/PutRNGstate inside the protection boundary with a cleanup callback

Recommendation: **B** — simplest, and RNG functions almost certainly want
error_in_r semantics anyway. Could enforce `rng` implies `error_in_r` in
the attribute parser.

**Resolution**: Fixed via option B. `rng + no_error_in_r` now rejected at
parse time in both `miniextendr_fn.rs` and `miniextendr_impl/mod.rs`. The
non-error_in_r RNG codegen path (with dead `resume_unwind`) removed from
`lib.rs` and `c_wrapper_builder.rs`. RNG always uses error_in_r now, so
`PutRNGstate` is guaranteed to execute.

### 2. `.expect()` in input conversions — two-hop error path

**Severity: Low (code quality / UX)**

Input conversions use `.expect(msg)` which panics. The panic is caught by
`catch_unwind` inside `R_UnwindProtect`, then goes through:

```
panic → panic_payload_to_string → make_rust_error_value → R condition
```

The R preconditions (`stopifnot()` in the R wrapper) catch most type errors
before crossing into Rust. But edge cases still reach the panic path:

- NA in non-nullable `i32` (R preconditions don't check for NA)
- Factor passed where integer expected (same R type predicate)
- Overflow in `u32` (R precondition checks `>= 0` but not max)

The error messages from `.expect()` are decent:
`"failed to convert parameter 'x' to i32: wrong type, length, or contains NA"`

But the two-hop path (panic → string → tagged SEXP → R condition) loses the
original error type. A `TryFromSexp::Error` could carry structured info
(which parameter, expected type, actual SEXPTYPE) that gets flattened to a
string by `panic_payload_to_string`.

**Fix options**:
- A: Return `Result` from conversions, build tagged error SEXP directly
  (avoids panic overhead, preserves structure). Requires refactoring the
  conversion builder to produce `let x = try_from_sexp(s)?;` instead of
  `.expect()`, and wrapping the whole conversion block in a
  match/error-return.
- B: Leave as-is — the double-hop adds ~microseconds, and the messages
  are already useful. Preconditions catch 90%+ of cases before Rust.

Recommendation: **B for now**. The preconditions do the heavy lifting.
Consider A as a future refinement if users report confusing error messages.

**Resolution**: Fixed via option A. `RustConversionBuilder` gained
`with_error_in_r()` — when enabled (main-thread + error_in_r), input
conversions generate `match` with early `return make_rust_error_value(...,
"conversion", Some(call))` instead of `.expect()`. Worker-thread and
non-error_in_r paths keep `.expect()` (caught by `catch_unwind`).

### 3. Worker thread: double `catch_unwind`

**Severity: Low (code quality)**

The worker path nests catch_unwind layers:

```
Layer 1: catch_unwind(|| {           // catches conversion panics
    // SEXP → Rust conversion
    match run_on_worker(|| {         // dispatches to worker
        catch_unwind(f)              // Layer 2: catches user fn panics
    }) {
        Ok(result) => with_r_unwind_protect(|| into_sexp(result)),
        Err(msg) => make_rust_error_value(...)
    }
})
```

Layer 1 catches panics during SEXP→Rust conversion (main thread, before
dispatch). Layer 2 (inside `run_on_worker` / `dispatch_to_worker`) catches
panics in the user function (worker thread).

This is **correct** — the two catch_unwinds protect different code on
different threads. But it's worth noting:

- Without `worker-thread` feature, `run_on_worker` is just `Ok(f())` with
  no catch_unwind. Layer 1 catches everything. So the double-catch only
  exists with the feature enabled.
- A conversion panic on the main thread produces a slightly different error
  path than a user function panic on the worker: both end up as tagged SEXP,
  but conversion panics go through Layer 1's handler while user panics go
  through `run_on_worker`'s Result::Err path.

No action needed — this is inherent to the two-thread design.

**Resolution**: No action needed. The double `catch_unwind` is correct by
design: Layer 1 protects main-thread conversions, Layer 2 protects worker-
thread user code. They operate on different threads.

### 4. `with_r_unwind_protect_error_in_r` is a near-copy of `with_r_unwind_protect`

**Severity: Low (code quality)**

The two functions share ~100 lines of identical structure:
- Same `CallData` struct
- Same `trampoline` function
- Same `cleanup_handler` function
- Same `Box::into_raw` / `catch_unwind(R_UnwindProtect)` / `Box::from_raw` pattern

They differ only in the terminal handling:

| Path | `with_r_unwind_protect` | `_error_in_r` |
|------|------------------------|---------------|
| Rust panic | `Rf_errorcall` (diverges) | `make_rust_error_value` (returns) |
| R longjmp | `R_ContinueUnwind` (diverges) | `R_ContinueUnwind` (diverges) |

**Fix options**:
- A: Extract common trampoline/cleanup into a shared inner function that
  returns an enum `{Ok(R), RustPanic(Box<dyn Any>), RLongjmp}`, then each
  variant handles the terminal case differently.
- B: Make `_error_in_r` a thin wrapper that calls a generic
  `with_r_unwind_protect_impl<F, R, H>(f, call, panic_handler: H)` where
  `H: FnOnce(Box<dyn Any+Send>) -> R` handles panics.
- C: Leave as-is — the duplication is self-contained and the functions are
  unlikely to diverge further.

Recommendation: **B** — the two functions will need to stay in sync forever
otherwise. A strategy parameter eliminates that maintenance burden.

**Resolution**: Fixed via a variant of option B. Extracted shared mechanics
into `run_r_unwind_protect<F, R>(f) -> Result<R, Box<dyn Any + Send>>`.
Both `with_r_unwind_protect_sourced` and `with_r_unwind_protect_error_in_r`
are now thin wrappers that call `run_r_unwind_protect` and handle the
`Err(payload)` case differently. ~100 lines of duplication eliminated.

### 5. `.call = match.call()` convention — vestigial in error_in_r path

**Severity: Negligible**

Every `.Call()` passes `.call = match.call()` as the first SEXP argument.
This is forwarded to `with_r_unwind_protect` as the `call` parameter for
`Rf_errorcall`.

In the `error_in_r` path (the default), the call SEXP is passed as `_call`
to `with_r_unwind_protect_error_in_r` and **never used**. The R wrapper
uses `sys.call()` instead when raising the condition:

```r
stop(structure(
  class = c("rust_error", "simpleError", "error", "condition"),
  list(message = .val$error, call = sys.call(), kind = .val$kind)
))
```

The `match.call()` overhead is minimal (R caches the call object), and
keeping it allows switching back to non-error_in_r without codegen changes.
But it's a wasted SEXP argument on every `.Call()`.

**Fix options**:
- A: Remove `.call = match.call()` from error_in_r wrappers, don't pass
  the call SEXP to the C wrapper. Saves one argument per `.Call()`.
- B: Use the call SEXP in `make_rust_error_value` — embed it in the tagged
  error list so the R wrapper can use it instead of `sys.call()`. Slightly
  better error context (original call vs. wrapper call).
- C: Leave as-is — the overhead is negligible and it's a consistent pattern.

Recommendation: **C** for now. The consistency is worth more than the micro-
optimization. Consider B if error reporting improvements are pursued.

**Resolution**: Fixed via option B. `make_rust_error_value` now takes
`call: Option<SEXP>` and embeds it in the tagged error list as a third
element. R wrapper uses `.val$call %||% sys.call()` for the condition's
`call` field. All call sites updated to pass the call SEXP.

### 6. Invisible handling — static determination from return type

**Severity: Informational**

`()` and `Result<(), E>` return types generate `invisible(.val)` in the R
wrapper. The `is_invisible` flag is determined statically by
`return_type_analysis.rs` and can be overridden with
`#[miniextendr(invisible)]` or `#[miniextendr(visible)]`.

This works well. No issues found. Noting for completeness.

**Resolution**: No action needed. Working correctly.

### 7. Global continuation token reuse with nested `R_UnwindProtect`

**Severity: Low (theoretical)**

A single `R_CONTINUATION_TOKEN` (OnceLock<SEXP>) is shared across all
`with_r_unwind_protect` calls. `R_UnwindProtect` stores jump target/mask
into the token at SETJMP time. If nested calls share the token, the inner
call overwrites the outer's data.

**Why this is safe in practice**: Two `R_UnwindProtect` calls with the same
token never both try to `R_ContinueUnwind` for the same R error. The inner
`R_ContinueUnwind` longjmps past the outer entirely (R's context chain
handles the nesting). The outer's cleanup handler never fires because the
longjmp goes to a context above both.

Nesting occurs in:
- Worker path: `dispatch_to_worker`'s event loop has its own
  `R_UnwindProtect` for `with_r_thread` callbacks, plus the outer
  `with_r_unwind_protect_error_in_r` for `into_sexp` conversion
- These don't nest in the R-error path — if the inner errors,
  `R_ContinueUnwind` diverges before the outer is ever entered

**Safe by construction, but the reasoning is subtle.** Per-call tokens would
eliminate the reasoning burden at the cost of one `R_PreserveObject` per
`with_r_unwind_protect` invocation (expensive).

No action recommended.

**Resolution**: No action needed. Safe by construction; the reasoning is
subtle but correct. Per-call tokens would add unnecessary overhead.

### 8. `f`'s stack locals leaked on R longjmp

**Severity: Inherent limitation**

When `f` (the user closure) calls an R API that `Rf_error`s, the longjmp
goes to `R_UnwindProtect`'s SETJMP, abandoning the trampoline and `f`'s
stack frames. The cleanup handler then panics `RErrorMarker`, but this
panic starts ABOVE the abandoned frames — it unwinds from the cleanup
handler up through the outer `catch_unwind`, not back through `f`.

Any `Drop` types that `f` had on its stack are NOT dropped. Only:
- `CallData` on the heap (reclaimed by `Box::from_raw`) ✅
- Captured variables in the closure `F` (dropped with CallData) ✅
- Frames above `R_UnwindProtect` (unwound by the panic) ✅

This is the fundamental limitation documented in R's `R_UnwindProtect` API.
Users who have critical RAII cleanup inside R API-calling code should use
nested `R_UnwindProtect` or ensure cleanup is in heap-allocated structures.

No fix possible — this is inherent to the longjmp/unwind interaction.

**Resolution**: No action needed. Inherent R limitation, well-documented.

### 9. Channel leak on R error during `dispatch_to_worker`

**Severity: Negligible**

When R errors during a `with_r_thread` callback in the worker event loop:
1. Cleanup handler sends error to worker via `response_tx`
2. `R_ContinueUnwind` longjmps past `dispatch_to_worker`'s stack frame
3. `worker_rx` (Receiver) on the abandoned stack never drops
4. Its Arc to the shared channel state never decrements

The worker side completes normally (sends `Done`), dropping its end.
But the main-thread's `worker_rx` Arc leaks (~100-200 bytes per R error
during worker dispatch).

R errors during worker dispatch are rare in practice. The leak is bounded
by the number of such errors per session.

No action recommended.

**Resolution**: No action needed. Bounded leak (~100-200 bytes per R error
during worker dispatch), rare in practice.

## Plan (completed 2026-03-07)

### Phase 1: Fix the RNG bug (Issue 1)

**Approach**: Enforce `error_in_r` when `rng` is set.

In `miniextendr_fn.rs` attribute parsing (~line 986):
```rust
// rng implies error_in_r — PutRNGstate must run after the .Call returns,
// and non-error_in_r diverges via Rf_errorcall/R_ContinueUnwind which
// skips PutRNGstate.
if rng && error_in_r == Some(false) {
    // emit compile error: rng + no_error_in_r is unsound
}
let error_in_r = if rng { true } else { error_in_r.unwrap_or(true) };
```

Then simplify the RNG codegen in `lib.rs`: remove the non-error_in_r RNG
path entirely (the `resume_unwind` branch). The RNG path always uses
error_in_r, so the `catch_unwind` + `PutRNGstate` pattern is always safe.

**Files**: `miniextendr-macros/src/miniextendr_fn.rs`, `miniextendr-macros/src/lib.rs`

**Verification**: `cargo test -p miniextendr-macros`, `just devtools-test`,
check that `#[miniextendr(rng, no_error_in_r)]` produces a compile error.

### Phase 2: Consolidate unwind_protect duplication (Issue 4)

**Approach**: Extract a shared `with_r_unwind_protect_impl` that takes a
panic handler strategy.

```rust
enum PanicStrategy {
    /// Diverge via Rf_errorcall (used by trait ABI shims)
    RError { call: Option<SEXP>, source: PanicSource },
    /// Return tagged error SEXP (used by #[miniextendr] functions)
    TaggedSexp,
}

fn with_r_unwind_protect_impl<F, R>(f: F, strategy: PanicStrategy) -> R
where F: FnOnce() -> R
{
    // shared trampoline/cleanup/token logic
    // terminal handling dispatches on strategy
}
```

Then `with_r_unwind_protect` and `_error_in_r` become thin wrappers.

**Files**: `miniextendr-api/src/unwind_protect.rs`

**Verification**: `cargo test -p miniextendr-api`, `just devtools-test`

### Phase 3: (Optional future) Structured conversion errors (Issue 2)

Not prioritized. The current `.expect()` + preconditions approach works well.
Revisit if users report confusing error messages for edge cases.

### Priority

All phases completed:

1. **Phase 1** (RNG bug) — ✅ Fixed. `rng` implies `error_in_r`; dead codegen removed.
2. **Phase 2** (consolidation) — ✅ Fixed. `run_r_unwind_protect` extracted; thin wrappers.
3. **Phase 3** (structured conversions) — ✅ Fixed (promoted from deferred). `with_error_in_r()`
   builder generates `match` instead of `.expect()` for main-thread error_in_r path.
4. **Issue 5** (call SEXP in error value) — ✅ Fixed. `make_rust_error_value` takes
   `call: Option<SEXP>`, R wrapper uses `.val$call %||% sys.call()`.
5. **Issues 3, 6, 7, 8, 9** — No action needed (documented above).
