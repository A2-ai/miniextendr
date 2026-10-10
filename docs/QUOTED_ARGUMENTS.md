# Unevaluated arguments

A `#[miniextendr]` function can take an argument the way `subset()`,
`with()` or `dplyr::filter()` do: as the expression the caller wrote, plus the
environment it was written in, without evaluating it. Rust then evaluates it
when and where it wants, against a data frame's columns say, and R conditions
raised on the way reach the caller's handlers as R code would raise them
(#1835).

Two parameter markers do this, both in `miniextendr_api`:

| Marker | The wrapper passes | Holds | Evaluate with |
|---|---|---|---|
| `Quoted` | `list(substitute(x), parent.frame())` | the expression and the caller's frame | `eval()`, `eval_in(data)` |
| `Quosure` | `rlang::enquo(x)` | an rlang quosure | `eval_tidy(data)`, or any rlang / tidyselect function through `RCall` |

Either may be wrapped in `Missing<..>` for an optional argument. Evaluating R
code from Rust in the caller's R context is also available on its own, for
calls built in Rust: [`eval_with_handlers`](#handler-keeping-evaluation).

## `Quoted`

```rust
use miniextendr_api::{Quoted, SEXP, miniextendr};

/// Rows of `data` where `cond` is TRUE, with the columns in scope.
#[miniextendr]
pub fn keep_rows(data: SEXP, cond: Quoted) -> Vec<i32> {
    let keep = cond.eval_in(data);   // a logical vector, unprotected
    /* ... */
}
```

The generated wrapper never forces `cond`:

```r
keep_rows <- function(data, cond) {
  .val <- .Call(C_pkg_keep_rows, .call = sys.call(), data,
    if (missing(cond)) quote(expr=) else list(substitute(cond), parent.frame()))
  ...
}
```

```r
limit <- 5
keep_rows(df, b > limit)     # `b` is a column, `limit` the caller's variable
```

| Method | Returns |
|---|---|
| `expr()` | the expression as written: a symbol, a call or a constant |
| `env()` | the environment it was written in (`parent.frame()` of the wrapper) |
| `as_name()` | `Some(name)` for a symbol (`x`) or a single string (`"x"`), else `None` |
| `eval()` | `eval(expr, env)` |
| `eval_in(data)` | `eval(expr, data, env)`: `data` a list or data frame (its named elements in front of `env`, the first of two same-named ones winning), an environment (used as is) or `NULL` (`env`). Anything else panics. |

`as_name()` borrows the symbol's or string's characters, which R keeps for
the call. The evaluated values are unprotected: protect one before the next
allocation (`OwnedProtect::new(cond.eval_in(data))`).

A standalone S3 method takes one too; `UseMethod()` keeps the generic's
promise, so `substitute()` in the method still sees what the user wrote and
`parent.frame()` is the generic's caller:

```rust
/// @param x A table.
/// @param subset A logical expression over the columns.
/// @param ... Ignored.
/// @export
#[miniextendr(s3(generic = "subset", class = "my_tbl"))]
pub fn my_tbl_subset(x: SEXP, subset: Quoted, _dots: ...) -> SEXP { /* ... */ }
```

`substitute()` reads one level: a function that forwards its own argument
(`f <- function(x) keep_rows(df, x)`) passes the symbol `x`, evaluated in
`f`'s frame. That is base R's semantics; use `Quosure` to forward through
user-level functions.

## `Quosure`

`Quosure` is the tidy-evaluation variant. The wrapper captures the argument
with `rlang::enquo()`, so `{{ x }}` and `!!x` in the caller work, and Rust
gets a quosure that rlang and tidyselect take as is:

```rust
use miniextendr_api::{Quosure, SEXP, miniextendr};

#[miniextendr]
pub fn pull_col(data: SEXP, col: Quosure) -> SEXP {
    col.eval_tidy(data)   // rlang::eval_tidy(col, data)
}
```

```r
pull_col <- function(data, col) {
  .val <- .Call(C_pkg_pull_col, .call = sys.call(), data,
    if (missing(col)) quote(expr=) else rlang::enquo(col))
  ...
}

my_pull <- function(data, col) pull_col(data, {{ col }})
my_pull(df, TIME)              # the `TIME` column
pull_col(df, !!rlang::sym("TIME"))
```

**A package with a `Quosure` parameter needs rlang in its `Imports:`.** The
wrapper calls `rlang::enquo()` (and `eval_tidy()` calls `rlang::eval_tidy()`),
so without rlang installed the call fails at run time with R's
`there is no package called 'rlang'`, and `R CMD check` warns about the
undeclared `rlang::` use. miniextendr itself does not depend on rlang;
`Quoted` needs nothing.

| Method | Returns |
|---|---|
| `sexp()` | the quosure (a classed formula), to pass to an R function |
| `expr()` | its expression (`rlang::quo_get_expr()`), read natively |
| `env()` | its environment (`rlang::quo_get_env()`), read natively |
| `as_name()` | `Some(name)` for a symbol or a single string, looking through nested quosures, else `None` |
| `eval_tidy(data)` | `rlang::eval_tidy(quo, data)`: columns win over the quosure's variables, `.data$x` / `.env$x` pick a side, `data` may be `NULL` |

Other tidy-evaluation functions take the quosure through
[`RCall`](#calls-built-in-rust), tidyselect for one:

```rust
use miniextendr_api::{Call, Quosure, SEXP, expression::RCall, miniextendr, sys::R_BaseEnv};

#[miniextendr]
pub fn select_cols(data: SEXP, cols: Quosure, call: Call) -> SEXP {
    unsafe {
        RCall::namespaced("tidyselect", "eval_select")
            .unwrap_or_else(|e| panic!("{e}"))
            .quoted_arg(cols.sexp())
            .quoted_arg(data)
            .named_arg("allow_rename", SEXP::scalar_logical(false))
            .named_quoted_arg("error_call", call.sexp())
            .eval_with_handlers(R_BaseEnv)
    }
}
```

`select_cols(df, c(TIME, ID))` returns the named positions; a column that
does not exist raises tidyselect's `vctrs_error_subscript_oob` with
`select_cols(df, c(TIME, ID))` as its call (the `Call` marker,
[CALL_ATTRIBUTION.md](CALL_ATTRIBUTION.md)).

## Omitted arguments

| The caller | `x: Quoted` / `x: Quosure` | `x: Missing<Quoted>` / `x: Missing<Quosure>` |
|---|---|---|
| omits `x` | argument error (below) | `Missing::Absent` |
| passes `NULL` | `expr()` is `NULL` | `Present`, `expr()` is `NULL` |
| passes a name | `expr()` is the symbol | `Present`, `expr()` is the symbol |

An omitted bare marker raises R's own message, `argument "x" is missing, with
no default`, as the package's argument-error condition (`kind =
"conversion"`, `e$param = "x"`, the wrapper's call; [CONDITIONS.md](CONDITIONS.md)),
before the function body runs. An argument forwarded from a function whose
own argument was omitted is omitted too: `g <- function(y) keep_rows(df, y);
g()`, and for `Quosure`, `{{ col }}` with `col` omitted.

`Missing<..>` takes a `default` for the formal, so the usage reads `x = NULL`:

```rust
#[miniextendr]
pub fn summarise_by(
    data: SEXP,
    #[miniextendr(default = "NULL")] by: Missing<Quosure>,
) -> SEXP { /* ... */ }
```

The default is never evaluated, and Rust never sees it: `missing(by)` is
`TRUE` for an omitted argument with a default, so the wrapper passes the
missing-argument sentinel (`if (missing(by)) quote(expr=) else
rlang::enquo(by)`) and the function gets `Missing::Absent`. A default that
would raise an error if evaluated is as good as `NULL`. Only `default` is
accepted, and only under `Missing<..>`.

## Restrictions

These are compile errors, each because it would force the argument or has no
argument to act on:

- per-parameter options (`coerce`, `match_arg`, `choices`, `no_na`,
  `inherits`, `not_inherits`, `preconditions`) on the parameter, and
  `default` on a bare marker or next to another option (`default` alone on
  `Missing<..>` writes the formal only, above);
- `Checked<Quoted>` / `Unchecked<..>`: the parameter has no R-side checks;
- the marker anywhere but the whole type or `Missing<..>`'s type argument
  (`Option<Quoted>`, `Vec<Quoted>`);
- a class or trait method taking one (only standalone functions, standalone
  S3 methods included, pass arguments unevaluated; methods are #1839).

A function taking either marker runs on R's main thread (also under
`worker`), and a marker borrows the `.Call()` argument: it cannot be stored
or sent to another thread. A written lifetime (`Quoted<'a>`) is accepted.

## Handler-keeping evaluation

`Quoted::eval()`, `eval_in()` and `Quosure::eval_tidy()` evaluate through
`miniextendr_api::expression::eval_with_handlers(expr, env)`, which is also
public for any expression or call built in Rust. It differs from
`RCall::eval` / `r_eval_str`, and `try_eval_with_handlers` combines the two:

| | `RCall::eval`, `r_eval_str` | `eval_with_handlers`, `RCall::eval_with_handlers` | `try_eval_with_handlers`, `RCall::try_eval_with_handlers` |
|---|---|---|---|
| R entry point | `R_tryCatchError` inside `R_ToplevelExec` | `Rf_eval` in its own `R_UnwindProtect` | `R_tryCatchError` in its own `R_UnwindProtect` |
| caller's `withCallingHandlers()`, `suppressWarnings()` | not seen (`R_ToplevelExec` empties the handler and restart stacks) | see every warning, message and condition | see every warning, message and condition except an error |
| R error | `Err(REvalError)`: R's condition, with its message (no `Error in` prefix, no `Calls:` line), call and classes; the Rust code decides what to raise | reaches the caller's `tryCatch()` as raised: class, call, fields | `Err(REvalError)`, as from `RCall::eval` |
| caller's `tryCatch(warning = )`, restarts | not seen | exit through the Rust frames | exit through the Rust frames |

An R error, or any other jump out of the evaluation (an exiting handler of
the caller's `tryCatch()`, `invokeRestart()`, an interrupt), stops at the
evaluator's own `R_UnwindProtect` frame and leaves it as a Rust unwind: the
Rust frames up to the `#[miniextendr]` boundary drop their values (a guard's
`Drop` runs), and the boundary hands the jump back to R, which carries on to
the handler or restart it was going to, with the original condition. A
warning or message the caller muffles just returns, and evaluation goes on.
So use it for code the user wrote or whose conditions are part of an
interface (a callback, a tidyselect selection, a deprecation warning), and
keep `RCall::eval` for internal calls whose failure the Rust code handles
itself; `REvalError::reraise_class` raises a caught error again under the
package's class with R's classes kept
([CONDITIONS.md](CONDITIONS.md#raising-a-caught-r-error-as-your-own)). The
framework's own calls into user R code (`as.character()` dispatch, S4
`slot()`) still go through `RCall::eval` (#1840).

Two rules follow from the unwind:

- Don't stop it with `std::panic::catch_unwind` between the evaluation and the
  boundary: the R jump would be dropped, and R would carry on as after
  `R_tryEval`. A caught payload must be resumed (`resume_unwind`) on the same
  thread before the call returns. The jump's target is an R frame of that
  call, so a payload kept and resumed in a later call cannot continue: the
  boundary raises an error instead.
- Don't evaluate from a `Drop` implementation: a jump there would start an
  unwind during an unwind, which aborts.

It must run inside a miniextendr boundary on R's main thread: a
`#[miniextendr]` body, a `with_r_unwind_protect` closure, an ALTREP callback
with the `r_unwind` or `rust_unwind` guard (the `unsafe` guard catches
nothing), a connection callback, a `with_r_thread` closure.

### Calls built in Rust

`RCall::eval_with_handlers(env)` builds the call and evaluates it the same
way. A call evaluates its arguments, so a language object meant as a value (a
call for `error_call`, an expression, a quosure, a symbol) goes in through
`quoted_arg(value)` / `named_quoted_arg(name, value)`, which wrap it in
`quote()`; other values pass through `quote()` unchanged.

```rust
// lifecycle-style warning: the caller's handlers see its class
RCall::namespaced("rlang", "warn")?
    .named_arg("message", SEXP::scalar_string_from_str("`old()` is deprecated"))
    .named_arg("class", SEXP::scalar_string_from_str("lifecycle_warning_deprecated"))
    .eval_with_handlers(R_BaseEnv);
```

### An R error returned as `Err`

`try_eval_with_handlers(expr, env)` and `RCall::try_eval_with_handlers(env)`
keep the caller's handlers the same way, but catch an R error at the
evaluation and return it as `Err(REvalError)`, built as `RCall::eval` builds
it. It suits a method that forwards the user's arguments to an R function and
raises the function's errors under the package's own class: the user's
`suppressWarnings()` and `tryCatch(warning = )` still see the function's
warnings.

```rust
// a `[` method of a data-frame subclass, forwarding to `[.data.frame`
let out = unsafe {
    RCall::new("[")
        .arg(frame)
        .arg(i)
        .named_arg("drop", drop)
        .try_eval_with_handlers(R_BaseEnv)
}
.unwrap_or_else(|e| rust_error!(class = e.reraise_class("pkg_subset_error"), "{e}"));
```

```r
x <- suppressWarnings(s[1, drop = FALSE])   # "'drop' argument will be ignored" silenced
tryCatch(s["zz"], pkg_subset_error = function(e) conditionMessage(e))
#> [1] "undefined columns selected"
```

Only a condition of class `error` is caught. A warning, message or other
condition reaches the caller's handlers, and an exit other than an error (the
caller's `tryCatch(warning = )`, `invokeRestart()`, an interrupt) leaves
through the Rust frames as under `eval_with_handlers`, so the two rules above
apply to it too. Because the error handler is the innermost one, the caller's
own error handlers (`withCallingHandlers(error = )`) don't see an error the
evaluation raises. An error raised by a handler of the caller's, as in
`withCallingHandlers(warning = function(w) stop("no warnings"))`, is the
caller's: R runs that handler outside the evaluation's handlers, so the error
reaches the caller's `tryCatch()` as raised. As under `RCall::eval`, the
evaluation runs inside R's `tryCatch()`, so a `stop()` at the top of the
expression has no `call()`
([EXPRESSION_EVAL.md](EXPRESSION_EVAL.md#error-handling)).

## Tests

`rpkg/src/rust/quoted_tests.rs` (fixtures, including the `subset()` method,
the `quoted_try_*` fixtures of `try_eval_with_handlers` and
`gc_stress_quoted()`) and `rpkg/tests/testthat/test-quoted.R`; the rlang and
tidyselect blocks skip when those packages are not installed.
`gc_stress_try_eval_with_handlers()` is in `rpkg/src/rust/gc_stress_fixtures.rs`.

## See also

- [EXPRESSION_EVAL.md](EXPRESSION_EVAL.md) -- `RCall`, `RSymbol`, `REnv`
- [CALL_ATTRIBUTION.md](CALL_ATTRIBUTION.md) -- the `Call` marker
- [CONDITIONS.md](CONDITIONS.md) -- the argument-error condition
- [MACRO_ERRORS.md](MACRO_ERRORS.md) -- the compile errors
