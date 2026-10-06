# Call attribution

Generated R wrappers pass `.call = sys.call()` into every `.Call()` so that errors raised from Rust are attributed to the user's call, as the user wrote it. This page shows the difference using a real, runnable fixture.

## The fixture

`rpkg/src/rust/call_attribution_demo.rs` defines two functions that raise the same error message. Only the wrapper differs.

```rust
// Wrapped path. Generated R wrapper passes `.call = sys.call()` into the
// C entry; on panic, `Rf_errorcall(call, msg)` shows the user's call.
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
  .val <- .Call(C_miniextendr_call_attr_with, .call = sys.call(), left, right)
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
Error in miniextendr:::call_attr_with(1L, 2L) :
  left + right is too risky

> miniextendr:::unsafe_C_call_attr_without(1L, 2L)
Error in miniextendr:::unsafe_C_call_attr_without(1L, 2L) :
  left + right is too risky
```

Wrapped inside another function so the difference is sharper:

```r
> outer_with <- function(x) miniextendr:::call_attr_with(x, x + 1L)
> outer_with(5L)
Error in miniextendr:::call_attr_with(x, x + 1L) :
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
miniextendr:::call_attr_with(x, x + 1L)
> conditionCall(e_without)
miniextendr:::unsafe_C_call_attr_without(x, x + 1L)
```

## Which call: the call as written

The call slot carries the call the way the user wrote it, `sys.call()`, not
the call with its formals matched (`match.call()`):

1. **The R convention.** `stop()`, `stopifnot()` and base R's argument errors
   report the call as written, and so does rlang / vctrs'
   `call = caller_env()`, which reports that frame's `sys.call()`.
2. **One form on every path.** The R-side checks (type and length guards,
   `no_na`, `inherits`, `not_inherits`, `match_arg` / `choices`), a Rust conversion error, an
   `Err` or panic from the body, a warning or message deferred from Rust, and
   the `Call` / `CallerCall` markers all see the same call. A handler that
   compares `conditionCall(e)` does not have to know which side refused the
   input.
3. **Forwarded arguments stay as written.** A call through
   `function(...) f(...)` reports `f(...)`, and `lapply(xs, f)` reports
   `FUN(X[[i]], ...)`, the frames' own calls; there is no `...` to expand.
4. **The cheaper form.** The slot is evaluated on every call, not only when
   one raises. `sys.call()` is a closure with one default argument around an
   `.Internal()`; `match.call()` has four, and matches the formals.
5. **Structured `rust_error` class.** The wrapped path goes through the
   tagged-condition decoder and returns a condition with class `rust_error`,
   which downstream handlers can catch specifically. The unwrapped path
   produces a plain `simpleError`.

A positional call therefore reports positional arguments:
`call_attr_with(1L, 2L)`, not `call_attr_with(left = 1L, right = 2L)`. A
partial name stays partial (`f(val = 1)`), and a call that `do.call()` built
reports the arguments `do.call()` was given.

## How it flows end to end

```text
R user calls:  call_attr_with(1L, 2L)
                       │
                       ▼
R wrapper:     .Call(C_miniextendr_call_attr_with, .call = sys.call(), left, right)
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
                     R error: "Error in call_attr_with(1L, 2L) : ..."
```

For the unwrapped extern `"C-unwind"` path, the call slot does not exist, so the panic-to-error path becomes `Rf_error(msg)` and R falls back to `sys.call()` of the wrapper frame.

## Internal entry points: caller attribution

`noexport` exists for package-internal entry points, and the natural layout is a
hand-written R function that validates its arguments and delegates to the
generated wrapper. With the default attribution the user then sees the bridge:

```text
Error in call_attr_self_impl(value) : x must be positive, got -1
```

`#[miniextendr(noexport, call = caller)]` moves the attribution one frame up.
The wrapper resolves its caller's call as the first thing in its body, then
hands that call to every R-side check, to `.Call()` and to the raise fallback.
Its last formal, `.call = NULL`, lets a helper in between pass on a different
call ([below](#a-helper-in-between-call)):

```r
call_attr_caller_impl <- function(x, .call = NULL) {
  .mx_call <- .miniextendr_caller_call(.call)
  if (!isTRUE(is.integer(x))) .miniextendr_arg_error("x", "must be integer", .mx_call)
  if (!isTRUE(length(x) == 1L)) .miniextendr_arg_error("x", "must have length 1", .mx_call)
  .val <- .Call(C_mypkg_call_attr_caller_impl, .call = .mx_call, x)
  if (inherits(.val, "rust_condition_value") && ...) return(.miniextendr_raise_condition(.val, .mx_call))
  .val
}
```

so the condition carries the hand-written function's call as written, whether
Rust or the R-side checks raised it:

```text
Error in call_attr_caller(-1L) : x must be positive, got -1
Error in call_attr_caller(1.5) : 'x' must be integer
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
`several_ok`, behind `if (is.character(kinds) || is.factor(kinds))` when the
list sits in an `Either<Vec<T>, R>` (#1612). Default-attribution wrappers emit
the same statements without the call argument, so the helper reports the
wrapper's own call (#1552).
Both helpers name the argument (`'kind' should be one of "a", "b"`). If a
downstream package's tests pinned `conditionCall()` for such a failure to
the `_impl` wrapper or to `match.arg()`, they now see the public call.

`.miniextendr_caller_call()` is defined once at the top of the generated
wrappers file. Called from the wrapper's body with the wrapper's `.call`, it
returns that argument's call when one was passed (see
[below](#a-helper-in-between-call)). For the default `NULL` it looks two
frames up for the wrapper's caller, and returns that frame's call as written.
It falls back to the wrapper's own call, without `.call`, in two cases: there
is no parent frame (called from top level, also under `tryCatch()` there), or
the parent frame's function is not a closure. The second case covers
`eval()`'d code (a testthat block, `source()`, `local()`): R gives such a
frame the `eval` primitive as its function, and its call would name `eval`.
A caller that forwards its dots (`function(...) call_attr_caller(...)`)
reports `call_attr_caller(...)`, as written.

The option requires `noexport` or `internal` (an exported function's caller is
arbitrary user code) and applies to standalone functions only; class methods
keep their own attribution. A `CallerCall` parameter is the type-level spelling
of the same option, and `[package.metadata.miniextendr] call_attribution =
"caller"` in `Cargo.toml` makes it the default for every internal entry point
(see [the next section](#choosing-the-attribution-marker-attribute-crate-default)). The frame is the *calling* frame, so an entry point reached
through `lapply()` reports `FUN(X[[i]], ...)`, and one reached through
`do.call()` reports the call `do.call()` built (`call_attr_caller(-1L)` for a
function name, the deparsed function for a function object), the same way
`sys.call(-1)` would. A helper that calls the entry point through `do.call()`
on behalf of a public function passes that function's frame as `.call`
instead (next subsection). Fixture pair:
`call_attr_caller_impl` / `call_attr_self_impl` in
`rpkg/src/rust/call_attribution_demo.rs` with the delegates in
`rpkg/R/call_attribution.R`, verified by `test-call-attribution.R`.

### A helper in between: `.call`

The calling frame is one frame. When a hand-written helper sits between the
public function and the entry point, the entry point's caller is the helper,
and the condition names it:

```r
.prepare <- function(value) call_attr_caller_impl(value)
call_attr_via_plain_helper <- function(value) .prepare(value)
# Error in .prepare(value) : x must be positive, got -1
```

Every wrapper whose attribution resolves to `caller` (the attribute, the
`CallerCall` marker or the crate default) therefore ends its formals with
`.call = NULL`. The helper takes its own caller's frame, following the vctrs /
rlang `call = caller_env()` convention, and passes it on:

```r
.prepare <- function(value, call = parent.frame()) {
  call_attr_caller_impl(value, .call = call)
}
call_attr_via_helper <- function(value) .prepare(value)
# Error in call_attr_via_helper(-1L) : x must be positive, got -1
```

What `.call` accepts:

- `NULL`, the default: the wrapper's caller, resolved as above.
- An environment: the call of the closure whose frame it is, as written, as
  for the default. `parent.frame()` in a helper's formals names the helper's
  caller; `environment()` names the function that passes it. A frame
  passes unchanged through nested helpers (each forwards `call = call`),
  `lapply()`, `...` forwarding, `local()` and `do.call()`. An environment
  that is no closure's live frame, such as `globalenv()`, counts as `NULL`.
- A call object: used as is, `.call = quote(verb(x = 1))`.
- Anything else: an argument error on `.call` (`e$param == ".call"`, class
  `rust_error`) reported against the wrapper's own call. This also catches a
  positional argument too many, `call_attr_caller_impl(-1L, 2)`, which used to
  be R's `unused argument` error.

**Pass a frame, not a call.** `do.call()` evaluates the arguments it is given
as a call, so a call object in the argument list runs the function it names
again:

```r
f <- function(x, .call = NULL) { force(.call); x }   # the wrapper forces .call first
g <- function() { n <<- n + 1; if (n > 3) stop("re-entered"); do.call(f, list(1, .call = quote(g()))) }
n <- 0; try(g())   # Error in g() : re-entered. Passing environment() instead returns 1.
```

An environment is a value, so it evaluates to itself. A call object needs
`do.call(..., quote = TRUE)`.

On a wrapper with `...`, `.call` follows the dots and any formal after them
(`function(x, ..., .call = NULL)`, `function(x, ..., flag = FALSE, .call = NULL)`):
positional extras land in the dots, and `.call` is matched by name only, as R
requires for any formal after `...`. Pass
it by name everywhere. Two cases keep today's shape. An S3 method has no
`.call`, because a formal the generic lacks breaks generic/method consistency;
its conditions name the generic's caller. A `CallerCall` body receives the
resolved call, which is still `is.call()`. An `internal` wrapper, which
renders a page, gets a generated `@param .call` line. Fixtures: the
`call_attr_via_*` delegates in `rpkg/R/call_attribution.R`,
`call_attr_internal_impl` / `call_attr_dots_impl` in
`rpkg/src/rust/call_attribution_demo.rs`, and `producer_via_helper` in
`tests/cross-package/producer.pkg` (the crate default), verified by the
`test-call-attribution.R` files.

## Choosing the attribution: marker, attribute, crate default

A standalone `#[miniextendr]` function picks one of two attributions, in
three equivalent spellings (#1566). Most specific wins:

| Spelling | `wrapper` | `caller` |
|----------|-----------|----------|
| **Marker parameter** (`miniextendr_api::{Call, CallerCall}`) | `call: Call` | `call: CallerCall` |
| **Attribute** | `call = wrapper` | `call = caller` |
| **Crate default** (`Cargo.toml`) | `call_attribution = "wrapper"` | `call_attribution = "caller"` |

Below those, `wrapper`. The attribute accepts the path and string forms
(`call = caller`, `call = "caller"`). The attribution is independent of
`no_preconditions`: a wrapper without its R-side checks still passes the call,
and `no_preconditions, call = caller` or a `Call` parameter under
`no_preconditions` work as without it.

```rust
use miniextendr_api::{Call, CallerCall, miniextendr};

/// `.call = sys.call()`, and the body gets to see it.
#[miniextendr]
pub fn scale(x: f64, call: Call) -> f64 { let _ = call.sexp(); x * 2.0 }

/// Internal entry point behind a hand-written `scale2()`: the caller's call,
/// as written, reaches Rust as `call`.
#[miniextendr(noexport)]
pub fn scale2_impl(x: f64, _call: CallerCall) -> f64 { x * 2.0 }
```

```toml
# Cargo.toml: every noexport / internal entry point reports its caller.
[package.metadata.miniextendr]
call_attribution = "caller"
```

The marker is **not an R formal**: the generated wrapper's formals are the
other parameters (`scale <- function(x)`), and the C wrapper binds the marker
from its hidden `__miniextendr_call` slot, so the body receives exactly the
SEXP the wrapper passed as `.call = ...`. A `caller` wrapper, `CallerCall`
included, still takes its own trailing `.call = NULL` formal
(`scale2_impl <- function(x, .call = NULL)`, see
[A helper in between](#a-helper-in-between-call)); the body then receives the
call that resolves to. Both markers are `repr(transparent)`
newtypes over `SEXP` (`.sexp()`, `Deref`, `From<CallerCall> for Call`), and a
function taking one runs on R's main thread like one taking `SEXP`. The marker
selects the attribution the same way the attribute does: `Call` is `wrapper`,
`CallerCall` is `caller` and therefore needs `noexport` / `internal` too.
Per-parameter options (`coerce`, `match_arg`, `choices`, `default`) do not
apply to it.

A crate default of `"caller"` applies to `noexport` / `internal` free
functions only: an exported function's caller is arbitrary user code, so it
keeps `wrapper`. `"wrapper"` applies to every standalone function. A per-item
spelling beats the crate default; a `call = wrapper` on one entry point
restores the wrapper's own call under a `"caller"` default. Class and trait
methods are untouched by all three spellings (a marker on a method is a
compile error) and always report their own call.

Two spellings on one function must agree. A `Call` parameter with
`call = caller`, two markers, a `CallerCall` on an exported function,
per-parameter options on a marker and `call = parent` are all compile errors
listed in [MACRO_ERRORS.md](MACRO_ERRORS.md#common-proc-macro-errors).
Fixtures: `call_marker_wrapper_impl` / `call_marker_caller_impl` /
`call_marker_checked_impl` in
`rpkg/src/rust/call_attribution_demo.rs` (verified by `test-call-attribution.R`),
and the crate default in `tests/cross-package/producer.pkg`
(`test-call-attribution.R` there).

## Where this is emitted

Every `.Call()` inside generated R wrappers puts the call slot first. Class and trait methods go through `DotCallBuilder` in `miniextendr-macros/src/r_wrapper_builder.rs`, which prepends `.call = sys.call()` (or `.call = NULL` for the lambda frames below, `null_call_attribution()`). Standalone functions take the argument from `CallAttribution::dot_call_arg()` in the same file: `.call = sys.call()` for `wrapper`, `.call = .mx_call` for `caller`. The C wrapper builder in `miniextendr-macros/src/c_wrapper_builder.rs` always declares `__miniextendr_call: SEXP` as the first parameter, so the convention is symmetric.

It applies uniformly to:

- Standalone `#[miniextendr]` functions
- All six class systems (R6, S3, S4, S7, Env, Vctrs) — constructors, instance methods, static methods, active bindings (the R6 finalizer and `deep_clone` pass `.call = NULL` instead, see below)
- All trait implementations across all class systems
- `match_arg` choices helper calls
- Warnings and messages deferred from Rust, which carry the slot's call

## Where it is intentionally absent

- **`extern "C-unwind"` functions** registered directly with `#[miniextendr]`. The function *is* the C entry point — there is no generated wrapper and no call slot. This is the demo above. Use only for low-level fixtures and tests where you control the error path manually.
- **`vctrs_derive` boilerplate** — `format.<class>`, `vec_ptype2.<class>.<class>`, etc. — pure R, no `.Call()`.
- **Sidecar `Type_get_field` / `Type_set_field` accessors** generated by `#[derive(ExternalPtr)]`. Their C functions take only `x` (and `value`), so the `.Call()` passes no `.call` argument; the guard's `sys.call()` supplies the call.

## Where `.call = NULL` is used instead of `sys.call()`

No attribute selects it. Five lambda dispatch sites cannot use `sys.call()` because the lambda is invoked by R6/S7 dispatch machinery, not by user code. `sys.call()` inside those lambdas would name the dispatch frame (e.g., `R6$finalize()`, `S7::prop_get()`), not the user's `obj$field` access. The generated `.Call()` instead passes `.call = NULL`. The raise helper's fallback, `if (isFALSE(.val$call)) NULL else if (is.null(.val$call)) .call_default else .val$call` with the wrapper's `sys.call()` as `.call_default` (`condition_check_lines`), then surfaces the nearest meaningful frame. (`FALSE` is the marker of a condition raised with `call = none`, which keeps no call at all; see [Conditions without a call](CONDITIONS.md#conditions-without-a-call).)

The five sites are:

1. **R6 finalizer** — `finalize = function() .Call(C_mypkg_Type__finalize, .call = NULL, private$.ptr)`
2. **R6 `deep_clone`** — `deep_clone = function(name, value) .Call(C_mypkg_Type__deep_clone, .call = NULL, private$.ptr, name, value)`
3. **S7 property validator** — `validator = function(value) .Call(C_mypkg_Type__validate_prop, .call = NULL, value)`
4. **S7 property getter** — `getter = function(self) .Call(C_mypkg_Type__get_prop, .call = NULL, self@.ptr)`
5. **S7 property setter** — `setter = function(self, value) { .Call(C_mypkg_Type__set_prop, .call = NULL, self@.ptr, value); self }`

This is implemented via `DotCallBuilder::null_call_attribution()` in `miniextendr-macros/src/r_wrapper_builder.rs`. The C wrapper still receives `__miniextendr_call: SEXP` (it always does) and gets `R_NilValue`; `make_rust_condition_value` stores it, and the raise helper's `.call_default` fallback (the wrapper's `sys.call()`) recovers the user's frame.

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
