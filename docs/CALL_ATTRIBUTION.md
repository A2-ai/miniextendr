# Call Attribution and `match.call()`

Generated R wrappers pass `.call = match.call()` into every `.Call()` so that errors raised from Rust are attributed to the user's call frame, with formal parameters matched by name. This page shows the difference using a real, runnable fixture.

## The fixture

`rpkg/src/rust/call_attribution_demo.rs` defines two functions that raise the same error message. Only the wrapper differs.

```rust
// Wrapped path. Generated R wrapper passes `.call = match.call()` into the
// C entry; on panic, `Rf_errorcall(call, msg)` shows the user's call frame.
#[miniextendr]
pub fn call_attr_with(_left: i32, _right: i32) -> i32 {
    panic!("left + right is too risky")
}

// Unwrapped path. `extern "C-unwind"` bypasses the wrapper entirely — there is
// no call slot and no `with_r_unwind_protect`. We raise an R error directly
// with `Rf_error`, which carries no call attribution.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_call_attr_without(_left: SEXP, _right: SEXP) -> SEXP {
    unsafe {
        ::miniextendr_api::sys::Rf_error(
            c"%s".as_ptr(),
            c"left + right is too risky".as_ptr(),
        )
    }
}
```

Generated R wrappers (excerpted from `rpkg/R/miniextendr-wrappers.R`):

```r
call_attr_with <- function(left, right) {
  # ... preconditions ...
  .val <- .Call(C_miniextendr_call_attr_with, .call = match.call(), left, right)
  # ... tagged-condition demux ...
}

unsafe_C_call_attr_without <- function(left, right) {
  .val <- .Call(C_call_attr_without, left, right)   # no .call slot
  # ...
}
```

The `extern "C-unwind"` path is registered directly as the `.Call` symbol, so there is nowhere to thread a call SEXP.

## The transcript

These two functions are internal demo fixtures (not exported); the `:::` prefix is intentional.

```r
> library(miniextendr)

> miniextendr:::call_attr_with(1L, 2L)
Error in miniextendr:::call_attr_with(left = 1L, right = 2L) :
  left + right is too risky

> miniextendr:::unsafe_C_call_attr_without(1L, 2L)
Error in miniextendr:::unsafe_C_call_attr_without(1L, 2L) :
  left + right is too risky
```

Wrapped inside another function so the difference is sharper:

```r
> outer_with <- function(x) miniextendr:::call_attr_with(x, x + 1L)
> outer_with(5L)
Error in miniextendr:::call_attr_with(left = x, right = x + 1L) :
  left + right is too risky

> outer_without <- function(x) miniextendr:::unsafe_C_call_attr_without(x, x + 1L)
> outer_without(5L)
Error in miniextendr:::unsafe_C_call_attr_without(x, x + 1L) :
  left + right is too risky
```

Programmatic comparison via `tryCatch`:

```r
> e_with    <- tryCatch(outer_with(5L),    error = identity)
> e_without <- tryCatch(outer_without(5L), error = identity)

> class(e_with)
[1] "rust_error"   "simpleError"  "error"        "condition"
> class(e_without)
[1] "simpleError"  "error"        "condition"

> conditionCall(e_with)
miniextendr:::call_attr_with(left = x, right = x + 1L)
> conditionCall(e_without)
miniextendr:::unsafe_C_call_attr_without(x, x + 1L)
```

## What `.call = match.call()` buys you

1. **Formal parameter names matched.** `call_attr_with(left = 1L, right = 2L)` reads better than `call_attr_with(1L, 2L)` and is robust to positional vs. named call style.
2. **Public function name, not internal symbol.** Errors blame `call_attr_with`, not `miniextendr:::unsafe_C_call_attr_without`. Triple-colon paths in error messages are a leak of internals.
3. **Structured `rust_error` class.** The wrapped path goes through the tagged-condition decoder and returns a condition with class `rust_error`, which downstream handlers can catch specifically. The unwrapped path produces a plain `simpleError`.
4. **Stable across nesting.** Whether the user calls the function directly or from another function, `match.call()` always captures the *immediate* caller's expression, with the call written as the user wrote it.

## How it flows end to end

```text
R user calls:  call_attr_with(1L, 2L)
                       │
                       ▼
R wrapper:     .Call(C_miniextendr_call_attr_with, .call = match.call(), left, right)
                       │
                       ▼
C wrapper:     extern "C-unwind" fn(__miniextendr_call: SEXP, left: SEXP, right: SEXP)
                  │
                  └── with_r_unwind_protect(closure, Some(__miniextendr_call))
                          │
                          ▼
                     Rust panic caught
                          │
                          ▼
                     Rf_errorcall(__miniextendr_call, "left + right is too risky")
                          │
                          ▼
                     R error: "Error in call_attr_with(left = 1L, right = 2L) : ..."
```

For the unwrapped extern `"C-unwind"` path, the call slot does not exist, so the panic-to-error path becomes `Rf_error(msg)` and R falls back to `sys.call()` of the wrapper frame.

## Internal entry points: caller attribution

`noexport` exists for package-internal entry points, and the natural layout is a
hand-written R function that validates its arguments and delegates to the
generated wrapper. With the default attribution the user then sees the bridge:

```text
Error in call_attr_self_impl(x = value) : x must be positive, got -1
```

`#[miniextendr(noexport, call = caller)]` moves the attribution one frame up.
The wrapper resolves its caller's call as the first thing in its body, then
hands that call to every R-side check, to `.Call()` and to the raise fallback:

```r
call_attr_caller_impl <- function(x) {
  .mx_call <- .miniextendr_caller_call()
  if (!isTRUE(is.integer(x))) .miniextendr_arg_error("x", "must be integer", .mx_call)
  if (!isTRUE(length(x) == 1L)) .miniextendr_arg_error("x", "must have length 1", .mx_call)
  .val <- .Call(C_mypkg_call_attr_caller_impl, .call = .mx_call, x)
  if (inherits(.val, "rust_condition_value") && ...) return(.miniextendr_raise_condition(.val, .mx_call))
  .val
}
```

so the condition carries the hand-written function's call with *its* formals
matched, whether Rust or the R-side checks raised it:

```text
Error in call_attr_caller(value = -1L) : x must be positive, got -1
Error in call_attr_caller(value = 1.5) : 'x' must be integer
```

The R-side checks take the caller's call under this option (#1548). Every
wrapper emits each precondition as a guard,
`if (!isTRUE(<check>)) .miniextendr_arg_error("<p>", "<requirement>")`, whose
helper raises the same argument error as a failed Rust conversion (#1591) and
by default reports the wrapper's own call, the frame `stopifnot()` used to
report (`isTRUE()` keeps `stopifnot()`'s failure semantics, and the guards are
cheaper than the `stopifnot()` call they replaced); the choice helpers default
to the same frame. With that default a Rust-side failure would name the public
function and a bad choice or a non-integer argument the bridge
(`verb_impl(...)`). A `call = caller` wrapper therefore passes `.mx_call` to
every precondition guard, and the caller's call to every choice parameter's
helper:
`.miniextendr_match_arg(kind, c(...), "kind", .mx_call)` for a scalar
`match_arg` / `choices` parameter, the same form inside `if (!is.null(kind))`
for an `Option<T>` choice (`if (!missing(kind) && ...)` when it is wrapped in
`Missing<..>`, #1551), and
`.miniextendr_match_arg_several(kinds, c(...), "kinds", .mx_call)` for
`several_ok`. Default-attribution wrappers emit the same statements without
the call argument, so the helper reports the wrapper's own call (#1552).
Both helpers name the argument (`'kind' should be one of "a", "b"`). If a
downstream package's tests pinned `conditionCall()` for such a failure to
the `_impl` wrapper or to `match.arg()`, they now see the public call.

`.miniextendr_caller_call()` is defined once at the top of the generated
wrappers file. Called from the wrapper's body, it looks two frames up for the
wrapper's caller, and returns that call with the caller's formals matched. It
falls back to the wrapper's own matched call in two cases: there is no parent
frame (called from top level, also under `tryCatch()` there), or the parent
frame's function is not a closure. The second case covers `eval()`'d code (a
testthat block, `source()`, `local()`): R gives such a frame the `eval`
primitive as its function, and `match.call()` rejects a non-closure
`definition`. The helper resolves the frames from its own body, never inside
a promise forced by `match.call()`, where `sys.call(0)` would resolve to
`match.call`'s frame.

The `envir` the helper hands to `match.call()` is the frame the caller's call
was evaluated in: the caller's caller. `match.call()` only consults `envir` to
expand a literal `...` in the call, and that is where the dots are bound when a
helper forwards them (`function(...) call_attr_caller(...)`) or when `lapply()`
evaluates `FUN(X[[i]], ...)` in its own frame. `match.call()`'s default is
the caller's frame, which has no `...`; before #1462 every call through such a
helper failed with `... used in a situation where it does not exist`, on the
success path too, since the call is matched before `.Call()`. The expansion
follows R's `match.call()` convention: constants forwarded through `...` are
inlined (`call_attr_caller(value = 0L)`), symbols and calls become `..1`,
`..2`.

The option requires `noexport` or `internal` (an exported function's caller is
arbitrary user code), cannot combine with `no_call_attribution` / `fast` (no
call slot to redirect), and applies to standalone functions only; class methods
keep their own attribution. A `CallerCall` parameter is the type-level spelling
of the same option, and `[package.metadata.miniextendr] call_attribution =
"caller"` in `Cargo.toml` makes it the default for every internal entry point
(see [the next section](#choosing-the-attribution-marker-attribute-crate-default)). The frame is the *calling* frame, so an entry point reached
through `lapply()` reports `FUN(value = X[[i]])`, and one reached through
`do.call()` reports the call `do.call()` built (`call_attr_caller(value = -1L)`
for a function name, the deparsed function for a function object), the same way
`sys.call(-1)` would. Fixture pair:
`call_attr_caller_impl` / `call_attr_self_impl` in
`rpkg/src/rust/call_attribution_demo.rs` with the delegates in
`rpkg/R/call_attribution.R`, verified by `test-call-attribution.R`.

## Choosing the attribution: marker, attribute, crate default

A standalone `#[miniextendr]` function picks one of three attributions, in
three equivalent spellings (#1566). Most specific wins:

| Spelling | `wrapper` | `caller` | `none` |
|----------|-----------|----------|--------|
| **Marker parameter** (`miniextendr_api::{Call, CallerCall}`) | `call: Call` | `call: CallerCall` | — |
| **Attribute** | `call = wrapper` (also `no_fast`) | `call = caller` | `call = none` (also `no_call_attribution`, `fast`) |
| **Crate default** (`Cargo.toml`) | `call_attribution = "wrapper"` | `call_attribution = "caller"` | `call_attribution = "none"` |

Below those come the `fast-default` cargo feature (`none`) and the framework
default, `wrapper`. The attribute accepts the path and string forms
(`call = caller`, `call = "caller"`).

```rust
use miniextendr_api::{Call, CallerCall, miniextendr};

/// `.call = match.call()`, and the body gets to see it.
#[miniextendr]
pub fn scale(x: f64, call: Call) -> f64 { let _ = call.sexp(); x * 2.0 }

/// Internal entry point behind a hand-written `scale2()`: the caller's call,
/// with the caller's formals matched, reaches Rust as `call`.
#[miniextendr(noexport)]
pub fn scale2_impl(x: f64, _call: CallerCall) -> f64 { x * 2.0 }

/// `.call = NULL`, spelled as an attribute.
#[miniextendr(call = none)]
pub fn hot_path(x: f64) -> f64 { x * 2.0 }
```

```toml
# Cargo.toml: every noexport / internal entry point reports its caller.
[package.metadata.miniextendr]
call_attribution = "caller"
```

The marker is **not an R formal**: the generated wrapper's formals are the
other parameters (`scale <- function(x)`), and the C wrapper binds the marker
from its hidden `__miniextendr_call` slot, so the body receives exactly the
SEXP the wrapper passed as `.call = ...`. Both markers are `repr(transparent)`
newtypes over `SEXP` (`.sexp()`, `Deref`, `From<CallerCall> for Call`), and a
function taking one runs on R's main thread like one taking `SEXP`. The marker
selects the attribution the same way the attribute does: `Call` is `wrapper`,
`CallerCall` is `caller` and therefore needs `noexport` / `internal` too.
Per-parameter options (`coerce`, `match_arg`, `choices`, `default`) do not
apply to it.

A crate default of `"caller"` applies to `noexport` / `internal` free
functions only: an exported function's caller is arbitrary user code, so it
keeps `wrapper`. `"none"` and `"wrapper"` apply to every standalone function.
The crate default beats the `fast-default` feature, and a per-item spelling
beats the crate default; a `call = wrapper` on one entry point restores the
wrapper's own call under a `"caller"` default. Class and trait methods are
untouched by all three spellings (a marker on a method is a compile error;
impl blocks keep their `fast` / `no_call_attribution` knobs).

Two spellings on one function must agree. A `Call` parameter with
`call = caller` (or with `fast`), two markers, a `CallerCall` on an exported
function, per-parameter options on a marker and `call = parent` are all
compile errors listed in [MACRO_ERRORS.md](MACRO_ERRORS.md#common-proc-macro-errors).
Fixtures: `call_marker_wrapper_impl` / `call_marker_caller_impl` /
`call_marker_checked_impl` / `call_attr_none_impl` in
`rpkg/src/rust/call_attribution_demo.rs` (verified by `test-call-attribution.R`),
and the crate default in `tests/cross-package/producer.pkg`
(`test-call-attribution.R` there).

## Where this is emitted

Every `.Call()` inside generated R wrappers goes through one source of truth: `DotCallBuilder` in `miniextendr-macros/src/r_wrapper_builder.rs`, which always prepends `.call = match.call()`. The C wrapper builder in `miniextendr-macros/src/c_wrapper_builder.rs` always declares `__miniextendr_call: SEXP` as the first parameter, so the convention is symmetric.

It applies uniformly to:

- Standalone `#[miniextendr]` functions
- All six class systems (R6, S3, S4, S7, Env, Vctrs) — constructors, instance methods, static methods, active bindings, finalizers, `deep_clone`
- All trait implementations across all class systems
- `match_arg` choices helper calls
- Sidecar `Type_get_field` / `Type_set_field` accessors generated by `#[derive(ExternalPtr)]`

## Where it is intentionally absent

- **`extern "C-unwind"` functions** registered directly with `#[miniextendr]`. The function *is* the C entry point — there is no generated wrapper and no call slot. This is the demo above. Use only for low-level fixtures and tests where you control the error path manually.
- **`vctrs_derive` boilerplate** — `format.<class>`, `vec_ptype2.<class>.<class>`, etc. — pure R, no `.Call()`.

## Where `.call = NULL` is used instead of `match.call()`

A standalone function opts into it with `call = none` (`no_call_attribution`,
`fast`) or through the crate default / `fast-default` feature, as described
above. Five lambda dispatch sites cannot use `match.call()` because the lambda is invoked by R6/S7 dispatch machinery, not by user code. `match.call()` inside those lambdas would capture the dispatch frame (e.g., `R6$finalize()`, `S7::prop_get()`), not the user's `obj$field` access. The generated `.Call()` instead passes `.call = NULL`. The `%||% sys.call()` fallback in `condition_check_lines` then surfaces the nearest meaningful frame.

The five sites are:

1. **R6 finalizer** — `finalize = function() .Call(C_mypkg_Type__finalize, .call = NULL, private$.ptr)`
2. **R6 `deep_clone`** — `deep_clone = function(name, value) .Call(C_mypkg_Type__deep_clone, .call = NULL, private$.ptr, name, value)`
3. **S7 property validator** — `validator = function(value) .Call(C_mypkg_Type__validate_prop, .call = NULL, value)`
4. **S7 property getter** — `getter = function(self) .Call(C_mypkg_Type__get_prop, .call = NULL, self@.ptr)`
5. **S7 property setter** — `setter = function(self, value) { .Call(C_mypkg_Type__set_prop, .call = NULL, self@.ptr, value); self }`

This is implemented via `DotCallBuilder::null_call_attribution()` in `miniextendr-macros/src/r_wrapper_builder.rs`. The C wrapper still receives `__miniextendr_call: SEXP` (it always does) and gets `R_NilValue`; `make_rust_condition_value` stores it and the R-side `%||% sys.call()` recovers the user's frame.

## Reproducing the transcript

```bash
just configure
just rcmdinstall
Rscript -e '
library(miniextendr)
try(miniextendr:::call_attr_with(1L, 2L))
try(miniextendr:::unsafe_C_call_attr_without(1L, 2L))
'
```

The fixture lives at `rpkg/src/rust/call_attribution_demo.rs`.
