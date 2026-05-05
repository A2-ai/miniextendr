# Condition system unification: #316 + #319 + #317 + Rf_error doc cleanup

**Goal:** Fold three pending issues + a doc-correctness cleanup into one coherent
PR. Result: a single, well-documented Rust→R condition pipeline with structured
`rust_*` class layering for *every* signal Rust raises (errors, warnings,
messages, conditions), correct call attribution at every site, and honest docs.

**Branch:** `feat/condition-system-unification` (built on `main`, supersedes #316)
**Worktree:** `.claude/worktrees/conditions/`

---

## Why one PR

Tightly coupled file overlap:

| File | #316 | #319 | #317 | Docs |
|---|---|---|---|---|
| `miniextendr-api/src/error_value.rs` | – | – | extend `kind` | comment |
| `miniextendr-api/src/unwind_protect.rs` | – | – | RCondition recognition | comment |
| `miniextendr-api/src/error.rs` | – | – | – | rewrite preamble |
| `miniextendr-api/src/condition.rs` (new) | – | – | yes | – |
| `miniextendr-macros/src/method_return_builder.rs` | – | – | switch on kind | – |
| `miniextendr-macros/src/r_wrapper_builder.rs` (DotCallBuilder) | yes | drop_call() | – | – |
| `miniextendr-macros/src/miniextendr_impl/{r6,s7}_class.rs` | – | five sites | – | – |
| `rpkg/src/rust/*` + `rpkg/tests/testthat/*` | demo | – | conditions test | – |
| `docs/CALL_ATTRIBUTION.md` | yes | update | – | – |
| `docs/CONDITIONS.md` (new) | – | – | yes | – |

Sequenced PRs would block each other on `error_in_r_check_lines` and
`DotCallBuilder` rewrites; one branch is cheaper.

---

## Background: what's actually happening today

### Two error transports

1. **`with_r_unwind_protect_error_in_r`** (default for all `#[miniextendr]` fns
   and methods — `miniextendr_fn.rs:1611`, `miniextendr_impl.rs:1677` both
   `error_in_r.unwrap_or(true)`). On Rust panic: `make_rust_error_value(msg,
   "panic", Some(__miniextendr_call))` returns a tagged SEXP normally. R
   wrapper inspects `.val` and raises `stop(structure(class = c("rust_error",
   "simpleError", "error", "condition"), …))`. **No `Rf_error` longjmp.**

2. **`with_r_unwind_protect`** (non-error_in_r). On Rust panic:
   `panic_payload_to_r_error` → `Rf_errorcall(call, ...)` longjmp. Hits
   `R_UnwindProtect`'s cleanup handler, drops Rust frames, then
   `R_ContinueUnwind`. Surfaces as a plain `simpleError` in R — no `rust_*`
   class.

### Where path 2 still fires

- **Trait-ABI vtable shims** (`miniextendr_trait.rs:808`,
  `miniextendr_impl_trait/vtable.rs:489`) — cross-package C-ABI calls.
- **ALTREP `RUnwind` guard** (`ffi_guard.rs:71` →
  `with_r_unwind_protect_sourced`) — protects callbacks invoked from R's GC /
  vector dispatch.
- User opts in via `no_error_in_r` or `unwrap_in_r`.

### Why it needs `Rf_error`

For paths 2 (trait shims, ALTREP), the Rust panic must surface as an R error
*from inside an `extern "C-unwind"` function*, with no R wrapper sitting
between us and the user. `Rf_errorcall` is the only mechanism that lets us
signal an error to R via the existing `R_UnwindProtect` token (which catches
the longjmp, runs Rust destructors, then `R_ContinueUnwind`s). There isn't a
"return a tagged SEXP" path here because there's no R wrapper to inspect it.

### Consequence: today's quiet inconsistency

A `panic!` from an ALTREP elt callback or a cross-package trait method surfaces
as `simpleError`, *losing* the `rust_*` class layering that direct
`#[miniextendr]` panics get. `tryCatch(rust_error = …)` won't catch it.

We accept this for now — fixing it would require either (a) wrapping every
trait-ABI / ALTREP callback in R-side R wrappers (huge cost, dubious value), or
(b) walking back up to find the nearest `error_in_r` frame and attaching class
information out-of-band. Both are out of scope.

### `.call` plumbing — what it actually does

Not for `Rf_errorcall`. The `__miniextendr_call: SEXP` first parameter on
every C wrapper is forwarded into `make_rust_error_value(..., Some(call))`
(`error_value.rs:54`) and read back in
`error_in_r_check_lines` (`method_return_builder.rs:28`) as
`.val$call %||% sys.call()`. It populates the `call` slot of the structured
condition so `conditionCall(e)` shows the user's expression in handlers.

That makes #316 a real bug regardless of #317: the C signature is
`(__miniextendr_call: SEXP, …)`. Sidecar getter/setter R wrappers were calling
`.Call(C_x_get_field, x)` instead of
`.Call(C_x_get_field, .call = match.call(), x)`, which slid the externalptr
`x` into the `__miniextendr_call` slot and dropped the first formal entirely.

### `.call` in lambda contexts (#319)

In R6 finalizer / `deep_clone` / S7 property getter/setter/validator, the R
wrapper code is a closure that R6/S7 dispatch calls on the user's behalf.
`match.call()` evaluated *inside* the closure captures the dispatch frame, not
the user's `obj$x` access. Result: `conditionCall(e)` returns garbage.

Fix is option 1 from #319: drop `.call = match.call()` from those five sites.
The `%||% sys.call()` fallback in `error_in_r_check_lines` then surfaces the
nearest meaningful frame (the externalptr accessor) instead of the dispatch
internals.

---

## Work breakdown

### Phase 1 — Rebase #316 sidecar arity fix

**Status:** PR #316 is open with checks 10/13. Cherry-pick its commits into
the new branch as the foundation.

- `git cherry-pick origin/<#316 head>~..origin/<#316 head>` (single commit
  range) — preserves the sidecar test, doc page, and DotCallBuilder
  unification.
- Verify wrapper diff is still 46 lines + demo.
- Do **not** rebase #316 itself; close it in favor of the new PR with a
  cross-link.

### Phase 2 — #319 drop `.call` from lambda sites

Five sites identified in #319:

1. R6 finalizer — `miniextendr_impl/r6_class.rs:382-389`
2. R6 `deep_clone` — `r6_class.rs:393-407`
3. S7 property getter — `s7_class.rs` via `MethodContext::instance_call("self@.ptr")`
4. S7 property setter — same
5. S7 property validator — `s7_class.rs:430-437`

**API change:** `DotCallBuilder` gains a flag:

```rust
impl DotCallBuilder {
    /// Skip `.call = match.call()`. For lambdas where match.call() captures
    /// internal dispatch frames, not the user's call.
    pub fn without_call_attribution(mut self) -> Self {
        self.skip_call = true;
        self
    }
}
```

`build()` emits `.Call(C_…, args…)` (no `.call` named arg) when `skip_call`
is set. The C wrapper still receives `__miniextendr_call`, but it'll be
whatever R passes positionally — which means... we *also* need the C wrapper
to handle this.

**Re-analysis:** The C wrapper signature is fixed —
`(__miniextendr_call: SEXP, …)`. If R `.Call`s it with no named `.call` arg,
the first positional SEXP slides into `__miniextendr_call`. That's a bug
(it's exactly the #316 bug, replayed).

**Correct fix:** Pass `.call = NULL` explicitly. R still names the arg, the
C wrapper gets `R_NilValue` for `__miniextendr_call`, and
`error_in_r_check_lines` falls back via `%||% sys.call()`.

```rust
pub fn null_call_attribution(mut self) -> Self {
    self.call_expr = Some("NULL".into());  // becomes ".call = NULL"
    self
}
```

Existing `build()` defaults to `match.call()`. Five lambda sites switch to
`null_call_attribution()`.

**Sanity check:** does `Some(R_NilValue)` in `make_rust_error_value` produce
sensible output? Today `call.unwrap_or(SEXP::nil())` already maps `None` to
`R_NilValue`, so `Some(R_NilValue)` is identical to `None` for the receiver.
Verified — both store `R_NilValue` in the tagged SEXP, and `%||% sys.call()`
catches it on the R side because `NULL %||% x == x`.

Update `docs/CALL_ATTRIBUTION.md` "Where it is intentionally absent" section
with the rationale and the five-site list.

### Phase 3 — #317 condition macros

#### 3a. Rust API

New file `miniextendr-api/src/condition.rs`:

```rust
/// Internal panic payload tagged so `with_r_unwind_protect_error_in_r`
/// recognises it before the generic panic-to-error path.
#[doc(hidden)]
#[derive(Debug)]
pub enum RCondition {
    Condition { message: String, class: Option<String> },
    Error     { message: String, class: Option<String> },
    Message   { message: String },
    Warning   { message: String, class: Option<String> },
}

#[macro_export]
macro_rules! condition {
    ($($arg:tt)*) => {
        ::std::panic::panic_any($crate::condition::RCondition::Condition {
            message: format!($($arg)*), class: None,
        })
    };
}

// analogous: error!, warning!, message!
```

Class-extension form (matches `rlang::abort(class = ...)`):

```rust
error!(class = "my_error", "missing field: {name}");
```

Implemented as a separate `match` arm inside the macro body, parsed via
`macro_rules!` token-tree matching:

```rust
macro_rules! error {
    (class = $class:expr, $($arg:tt)*) => { ... };
    ($($arg:tt)*) => { ... };
}
```

#### 3b. Unwind path recognition

`with_r_unwind_protect_error_in_r` and `with_r_unwind_protect_sourced` both
need to recognise `RCondition` payloads before falling through to the generic
panic handler.

In `unwind_protect.rs::with_r_unwind_protect_error_in_r`:

```rust
Err(payload) => {
    if let Some(cond) = payload.downcast_ref::<RCondition>() {
        let (kind, msg, class) = match cond {
            RCondition::Error { message, class }     => ("error",     message, class.clone()),
            RCondition::Warning { message, class }   => ("warning",   message, class.clone()),
            RCondition::Message { message }          => ("message",   message, None),
            RCondition::Condition { message, class } => ("condition", message, class.clone()),
        };
        crate::error_value::make_rust_condition_value(msg, kind, class.as_deref(), call)
    } else {
        // generic panic path — unchanged
        let msg = panic_payload_to_string(payload.as_ref());
        ...
        crate::error_value::make_rust_error_value(&msg, "panic", call)
    }
}
```

For the non-error_in_r path (`with_r_unwind_protect_sourced`), the situation
is different — there's no R wrapper to inspect a tagged SEXP. The choice is:

- **Option A (chosen):** in non-error_in_r mode, `condition!`/`message!`/
  `warning!` route directly to `Rf_eval` of `signalCondition()`/`message()`/
  `warning()` with the `rust_*` class layered, then **return normally** (or
  in the case of `warning`, return whatever the closure was meant to return —
  but at this point we don't have a default value, so we'd need the macro to
  fail compilation in non-error_in_r contexts). Pragmatic: only `error!`
  works in non-error_in_r mode (still goes via `Rf_errorcall` for path
  consistency with current behavior). Other variants degrade to a panic with
  a "must be in error_in_r mode" message.

- **Option B:** route warning/message/condition to direct R primitives even
  in non-error_in_r mode and continue execution. Hard because the Rust
  closure doesn't get a chance to resume after `panic_any` — by definition
  the panic unwound the stack.

Going with **Option A**: document that `condition!`/`message!`/`warning!`
require `error_in_r` (the default), and in non-error_in_r contexts they
behave like `error!`. Keep the surface uniform.

#### 3c. R-side switch

Update `error_in_r_check_lines` (`method_return_builder.rs:16`) and the two
sibling helpers (`error_in_r_inline_block`, `error_in_r_standalone_body`) to
emit:

```r
.val <- <call_expr>
if (inherits(.val, "rust_error_value") && isTRUE(attr(.val, "__rust_error__"))) {
  .msg <- .val$error
  .call <- .val$call %||% sys.call()
  .class <- .val$class  # NULL if no custom class
  .layered <- function(base) c(.class, paste0("rust_", base), paste0("simple", capitalize(base)), base, "condition")
  switch(.val$kind,
    error     = stop(structure(list(message = .msg, call = .call), class = .layered("error"))),
    warning   = warning(structure(list(message = .msg, call = .call), class = .layered("warning"))),
    message   = message(structure(list(message = paste0(.msg, "\n"), call = NULL), class = .layered("message"))),
    condition = signalCondition(structure(list(message = .msg, call = .call), class = .layered("condition"))),
    panic     = stop(structure(list(message = .msg, call = .call), class = .layered("error"))),
    # default: legacy
    stop(structure(list(message = .msg, call = .call), class = c("rust_error", "simpleError", "error", "condition")))
  )
}
```

Notes:
- `paste0("simple", capitalize(base))` produces `simpleError` /
  `simpleWarning` / `simpleMessage` / `simpleCondition` matching the
  primitive's standard class.
- For `message`, `paste0(msg, "\n")` matches `simpleMessage` semantics
  (trailing newline). Comment on the line so it isn't "cleaned up".
- For `condition`, no default action — `signalCondition` with no handler is
  a silent no-op, which matches the issue's spec.
- The `.class` slot in the tagged SEXP carries the optional user-supplied
  custom class (from `condition!(class = "my_class", ...)`) and prepends to
  the layered vector.

The R-side helper `capitalize` (or just inline the four explicit names) lives
inline in the wrapper — keep generated R code dependency-free.

Actually: avoid runtime helper. Generate the four cases explicitly. Cleaner
and keeps R wrapper output self-contained.

#### 3d. Tagged SEXP carries `class`

Extend `make_rust_error_value` (or add `make_rust_condition_value`) to write
a `class` element (length-1 character or `NULL`) into the list. Keep
backward compat: existing `kind = "panic" / "result_err" / "none_err"` paths
pass `class = None` and the R switch falls to the default `c("rust_error",
"simpleError", "error", "condition")` branch — identical to today.

#### 3e. Doc + tests

- `docs/CONDITIONS.md` — runnable side-by-side R transcripts of all four
  macros + `tryCatch` / `withCallingHandlers` examples.
- `rpkg/src/rust/condition_demo.rs` — fixture functions.
- `rpkg/tests/testthat/test-conditions.R` — assert class layering, handler
  dispatch, `withCallingHandlers(warning = h)` continuation,
  `suppressMessages` muffling via `muffleMessage` restart, custom-class
  matching.

### Phase 4 — Doc + comment cleanup

- `miniextendr-api/src/error.rs` preamble: clarify that `error_in_r` is the
  default and that user code should use `panic!()` (existing) or the new
  `error!()` / `warning!()` / `message!()` / `condition!()` macros.
  `r_stop` is internal; `Rf_error` only fires in non-error_in_r paths
  (trait shims, ALTREP guard, opt-out).
- `miniextendr-api/src/unwind_protect.rs:269`: doc comment for
  `with_r_unwind_protect_error_in_r` — "DEFAULT for `#[miniextendr]` fns".
  The plain `with_r_unwind_protect` doc — "used by trait shims, ALTREP
  RUnwind guard, and explicit opt-out via `no_error_in_r`".
- `unwind_protect.rs:243` example shows `with_r_unwind_protect` directly —
  rewrite to use `with_r_unwind_protect_error_in_r` since that's the actual
  default user-facing path.

---

## Acceptance criteria

- [ ] PR #316 closed in favor of the new PR; sidecar arity fix preserved.
- [ ] Five lambda sites use `null_call_attribution` (or equivalent);
  `conditionCall(e)` from a panicking R6 finalizer surfaces the user's frame
  via `sys.call()`, not the dispatch internals.
- [ ] `condition!` / `error!` / `warning!` / `message!` macros compile;
  optional `class = "..."` form works.
- [ ] `class()` of each emitted condition starts with `rust_*`:
  `c("rust_error", "simpleError", "error", "condition")` and analogous.
- [ ] `tryCatch(rust_warning = h, fn_warning())` invokes `h`.
- [ ] `withCallingHandlers(warning = h, fn_warning())` invokes `h` and
  continues — verified by post-call assertion.
- [ ] `tryCatch(condition = h, fn_condition())` catches a `rust_condition`.
- [ ] `suppressMessages(fn_message())` muffles via `muffleMessage` restart
  (no extra wiring needed — falls out of `message()`'s default handling).
- [ ] `tryCatch(my_class = h, error!(class = "my_class", "..."))` matches.
- [ ] `condition!`/`message!`/`warning!` in non-error_in_r mode degrade
  cleanly (panic with "use error_in_r mode" message).
- [ ] `docs/CONDITIONS.md` exists with runnable transcripts.
- [ ] `docs/CALL_ATTRIBUTION.md` updated; the five lambda sites are no
  longer flagged as "debatable".
- [ ] `error.rs` / `unwind_protect.rs` preambles tell the truth about
  `error_in_r` being the default.
- [ ] No regression on existing `panic!` → `rust_error` path (verified by
  unchanged class assertions in existing tests).
- [ ] CI green: `just clippy` (workspace + cross-package + rpkg),
  `just devtools-document`, `just rcmdcheck`, `cargo ltest -p
  miniextendr-macros`, `just devtools-test`.

---

## Out of scope (track separately)

- Routing trait-ABI / ALTREP `RUnwind` panics through the tagged-SEXP path
  to gain `rust_*` layering. Substantial — file as follow-up issue.
- Structured `data = list(...)` payloads in conditions (`rlang::abort`-style).
  The class extension covers most use cases; fold structured data into a
  later PR if user demand surfaces.
- Removing the non-`error_in_r` path entirely. Trait shims and ALTREP guard
  rely on it; removal would require a much bigger redesign.
