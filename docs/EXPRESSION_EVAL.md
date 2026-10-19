# Expression Evaluation Helpers

Safe wrappers for building and evaluating R function calls from Rust.

## Types

| Type | Purpose |
|------|---------|
| `RSymbol` | Interned R symbol (SYMSXP) -- never GC'd |
| `RCall` | Builder for R function calls (LANGSXP) |
| `REnv` | Well-known R environments (Global, Base, Empty) and their bindings |
| `REvalError` | An R error caught by `RCall::eval` / `r_eval_str` / `try_eval_with_handlers`: R's condition |

Plus free functions: `r_eval_str` / `r_eval_str_global` (parse + evaluate a
string of R source), `dollar_extract` (the R `$` operator), and
`eval_with_handlers` / `try_eval_with_handlers` (evaluate in the caller's R
context, see [Handler-keeping evaluation](#handler-keeping-evaluation)). A
call's arguments read with `PairListExt` ([Reading a
call](#reading-a-call-pairlistext)).

## Quick Example

```rust
use miniextendr_api::expression::{RCall, REnv};
use miniextendr_api::sys::Rf_mkString;

unsafe {
    // Call paste0("hello", " world") in base
    let result = RCall::new("paste0")
        .arg(Rf_mkString(c"hello".as_ptr()))
        .arg(Rf_mkString(c" world".as_ptr()))
        .eval(REnv::base().as_sexp())?;
}
```

## RSymbol

Wraps R's `Rf_install()` for interned symbols. Symbols are never garbage collected, so `RSymbol` needs no GC protection.

```rust
use miniextendr_api::expression::RSymbol;

// From a Rust string (allocates a CString)
let sym = unsafe { RSymbol::new("my_var") };

// From a C string literal (zero allocation)
let sym = unsafe { RSymbol::from_cstr(c"my_var") };

// Use as SEXP
let sexp = sym.as_sexp();
```

## RCall

Builds R function calls with positional and named arguments.

```rust
use miniextendr_api::expression::RCall;

unsafe {
    // Positional arguments
    let result = RCall::new("sum")
        .arg(my_vector_sexp)
        .eval(env)?;

    // Named arguments
    let result = RCall::new("paste")
        .arg(x_sexp)
        .arg(y_sexp)
        .named_arg("sep", sep_sexp)
        .eval(env)?;
}
```

### Rust values as arguments, and reading the result

`arg_value(value)` / `named_value(name, value)` convert a Rust value with
`IntoR` and add it, and `eval_as::<T>(env)` / `eval_base_as::<T>()` evaluate
and read the result with `TryFromSexp`:

```rust
let q: Vec<Option<f64>> = unsafe {
    RCall::new("qt")
        .arg_value(p)
        .arg_value(df.to_vec())
        .eval_as(stats.as_sexp())?
};
```

Each converted value is rooted as it is added, and the result stays rooted
while it is converted, so no SEXP is ever held unrooted in between. Converting
the values first (`let p = p.into_sexp(); let df = df.into_sexp();`) would
leave `p` unrooted while `df`, and `RCall::new`, allocate. Reading a result
into an owned Rust value (`Vec<T>`, `String`, a struct) needs no root
afterwards; a view type (`SEXP`, `List`, `DataFrame`) comes back unrooted, as
from `eval()`. A result `T` can't read is an `Err` like an R error: a
`simpleError` with the conversion's message and no call.

### Error Handling

`eval()` returns `Result<SEXP, REvalError>`. An R error comes back as
`REvalError`, which holds R's condition object (rooted while the value lives)
and what R reports about it:

| Method | What it gives |
|---|---|
| `message()` | `conditionMessage(cond)`: no `Error in <call> :` prefix, no `Calls:` traceback line |
| `call()` | `conditionCall(cond)` as `Option<SEXP>` |
| `classes()` | the class vector, most specific first |
| `specific_classes()` | the classes ahead of `rust_error` / `simpleError` / `error` / `condition` |
| `inherits(class)` | whether `classes()` contains `class` |
| `condition()` | the condition object |
| `reraise_class(own)` | `own`, then the specific classes: the `class =` for raising it again |
| `reraise()` (`unsafe`) | raise the condition again as R raised it, with `call()` as its call; inside a miniextendr boundary |

`Display` writes the message alone, and `REvalError` is a `std::error::Error`.

```rust
match RCall::new("print").arg(df).named_arg("na.print", one).eval_base() {
    Ok(_) => {}
    // e.message() is "invalid 'na.print' specification", not
    // "Error in print.default(...) : invalid 'na.print' specification\nCalls: ..."
    Err(e) => rust_error!(class = e.reraise_class("pkg_print_error"), "{e}"),
}
```

[CONDITIONS.md](CONDITIONS.md#raising-a-caught-r-error-as-your-own) covers
raising the caught error again as the package's own condition.

The call runs in a new top-level context (`R_ToplevelExec`, as under
`R_tryEval`) under an exiting handler for `error` conditions
(`R_tryCatchError`). The top-level context hides the caller's condition
handlers and restarts: a warning goes to R's default handler rather than to
the caller's `withCallingHandlers()`, and an interrupt ends the evaluation
with an `Err`. For user code, or a call whose conditions the user should see
as raised, use [`eval_with_handlers`](#handler-keeping-evaluation) instead;
for a call whose warnings and messages the user should see but whose error the
Rust code raises itself, `try_eval_with_handlers`.

Two details follow from the `tryCatch()` underneath:

- An error raised at the top of the evaluated expression (`stop()` called
  directly, not from a function) gets the call of R's `tryCatch()` frame,
  `doTryCatch(return(expr), name, parentenv, handler)`; `call()` reports
  `None` for it, as base R's `try()` leaves it out. `condition()` keeps what
  R recorded.
- Code at the top of the evaluated expression that inspects the call stack
  (`sys.call()`, `sys.function()`) sees the `tryCatch()` frames.

`REvalError` is `!Send`: dropping it releases its root, which must happen on
R's main thread. Carry `message()` (a `String`) or a
`condition::RError` across threads instead.

### Arguments that are language objects

A call evaluates its arguments, so a symbol, call or quosure added with `arg()`
is evaluated before the function sees it. `quoted_arg(value)` /
`named_quoted_arg(name, value)` pass it as is (`quote(<value>)`), for an
`error_call` argument say.

### GC Protection

`RCall` roots the callable and every positional or named argument for the
builder's lifetime, including freshly allocated inline arguments
(`.arg(x.into_sexp())`, `.arg_value(x)`). Symbols are
the exception: R never frees a symbol (`install()` links it into the symbol
table, which the collector marks on every collection), so a symbol callable or
argument takes no root. It also
protects the call object and intermediate pairlist during construction. The
**returned SEXP is unprotected** -- caller must protect it if it will survive
across R API calls.

## REnv

Provides handles to R's well-known environments:

```rust
use miniextendr_api::expression::REnv;

unsafe {
    let global = REnv::global();                     // R_GlobalEnv
    let base = REnv::base();                         // R_BaseEnv
    let empty = REnv::empty();                       // R_EmptyEnv
    let base_ns = REnv::base_namespace();            // R_BaseNamespace (for .Internal etc.)
    let methods = REnv::package_namespace("methods")?; // getNamespace("methods")
    let caller = REnv::caller();                     // calling environment (R_GetCurrentEnv)

    // Use as SEXP
    let sexp = base.as_sexp();
}
```

Prefer `package_namespace(pkg)` over chasing symbols through
`R_GlobalEnv`. The former mirrors `getNamespace(pkg)` and resolves
against the package's own namespace regardless of what the user has
attached on the search path. `eval_global()` has been removed; evaluate
in `base()`, `base_namespace()`, or the caller's env instead.

### Reading and writing bindings

`get(name)` reads a binding of the environment's own frame and
`get_inherits(name)` searches the enclosing environments too, as
`get0(name, envir = env, inherits = FALSE / TRUE)` does. Both return `None`
for an unbound name and `Some` of `NULL` for a name bound to `NULL`.
`define(name, value)` binds `name` in the environment's own frame, as
`assign()` does.

```rust
unsafe {
    let ns = REnv::package_namespace("mypkg")?;
    let helper = ns.get("helper").expect("mypkg defines helper");
    let env = REnv::from_sexp(scope.new_env(REnv::base().as_sexp(), true, 2).get());
    env.define("x", x);
}
```

The same code runs on every R the crate supports: R 4.5 and later read with
`R_getVarEx`, R 4.4 evaluates `base::get0()`. Like `get0()`, a read forces a
promise binding (a lazy-loaded namespace function is one) and calls an active
binding, so it is not a check for whether a name is bound. An R error raised
while reading or binding (a failing promise or active binding, a locked
environment or binding, the empty environment) unwinds to the miniextendr
boundary as under `eval_with_handlers`, so call them inside one.

The value `get` returns is unprotected. A plain binding or a forced promise
keeps it reachable while the name stays bound to it; an active binding's value
is held by nothing. Protect it before the next allocation unless the binding is
known to be a plain value, and before anything that may rebind the name.

## r_eval_str

Parse a string of R source and evaluate it — the runtime workhorse behind the
`r_str!` / `r!` macros. Every top-level expression is evaluated in order
(so side effects take effect); the value of the **last** one is returned,
matching `eval(parse(text = ...))`. Empty / whitespace-only input yields
`R_NilValue`.

```rust
use miniextendr_api::expression::{r_eval_str, r_eval_str_global};

unsafe {
    // In a specific environment
    let three = r_eval_str("1L + 2L", env)?;

    // Convenience wrapper for R_GlobalEnv
    let six = r_eval_str_global("local({ x <- 2; x * 3 })")?;
}
```

An R evaluation error comes back as `Err(REvalError)`, caught as by
`RCall::eval`, so it never longjmps through Rust frames. A parse failure
(syntax error, incomplete input) raises no R condition; it comes back as an
`REvalError` holding a `simpleError` built for it, whose message names the
failure and the source (`incomplete R expression (unbalanced delimiter?): 1 +`).
The returned SEXP is unprotected.

## dollar_extract

Convenience wrapper for the R `$` extraction operator, replacing hand-rolled
`Rf_install("$")` + `Rf_lang3` + `R_tryEval` ladders. It evaluates through
`RCall::eval`, so a failure is an `REvalError`:

```rust
use miniextendr_api::expression::dollar_extract;

unsafe {
    let value = dollar_extract(list_sexp, "field_name")?;
}
```

## Reading a call: `PairListExt`

A call (`LANGSXP`) and a pairlist (`LISTSXP`, such as `formals(f)`) are chains
of nodes, each with a value (CAR), the rest (CDR) and a name (TAG).
`PairListExt`, at the crate root and in the prelude, reads and writes them on
a `SEXP`:

| Method | Returns |
|---|---|
| `car()` / `cdr()` / `tag()` | the node's value (on a call, the function), the rest (`NULL` after the last node), its name (a symbol, or `NULL`) |
| `args()` | an iterator over a call's arguments (the function left out) or a pairlist's elements, as `(Option<&str>, SEXP)`; `NULL` gives none, anything else panics |
| `set_car(v)` / `set_cdr(v)` / `set_tag(sym)` | change the node |
| `cons(cdr)` / `lcons(cdr)` | a new pairlist / call node, unprotected |

```rust
use miniextendr_api::{PairListExt, SEXP, miniextendr};

/// The names of a call's arguments: `quote(f(a, b = 2))` gives `NA`, `"b"`.
#[miniextendr]
pub fn arg_names(call: SEXP) -> Vec<Option<String>> {
    call.args().map(|(name, _)| name.map(str::to_owned)).collect()
}
```

The values are reachable from the pairlist, so they stay rooted while it does;
the names borrow it. An empty argument (`f(a = )`) is `SEXP::missing_arg()`,
and a `NULL` argument does not end the walk.

A call R hands you is shared (with the caller's code, a quosure, a promise), so
change only a copy you own: `shallow_duplicate()` copies the nodes and keeps
the values, so `set_car` on a node of the copy leaves the original as it was.

```rust
// `subset(x, cond)` with its first argument replaced; `call` is unchanged
let copy = scope.protect_raw(call.shallow_duplicate());
copy.cdr().set_car(data_symbol);
```

The arguments of a `Quosure` parameter's call read as quosures with
`Quosure::args()` ([QUOTED_ARGUMENTS.md](QUOTED_ARGUMENTS.md#quosures-in-a-call)).

## Handler-keeping evaluation

`eval_with_handlers(expr, env)` and `RCall::eval_with_handlers(env)` evaluate
in the caller's R context: `Rf_eval` inside the evaluator's own
`R_UnwindProtect`, so the caller's `withCallingHandlers()` and
`suppressWarnings()` see every condition, and an R error or a `tryCatch()`
exit unwinds the Rust frames (destructors run) and then reaches the caller's
handler with the original condition. The result is unprotected.

```rust
use miniextendr_api::expression::RCall;
use miniextendr_api::sys::R_BaseEnv;

unsafe {
    let selected = RCall::namespaced("tidyselect", "eval_select")?
        .quoted_arg(cols.sexp())      // a `Quosure` parameter
        .quoted_arg(data)
        .named_quoted_arg("error_call", call.sexp())
        .eval_with_handlers(R_BaseEnv);
}
```

`try_eval_with_handlers(expr, env)` and `RCall::try_eval_with_handlers(env)`
do the same, except that an R error is caught at the evaluation and comes back
as `Err(REvalError)`, as from `RCall::eval`: the caller's handlers see
warnings and messages, and the Rust code decides what to raise for an error
(`e.reraise_class(...)`). Other exits (`tryCatch(warning = )`, restarts) still
leave through the Rust frames.

```rust
let out = unsafe {
    RCall::new("[")
        .arg(frame)
        .arg(i)
        .named_arg("drop", drop)
        .try_eval_with_handlers(R_BaseEnv)
}
.unwrap_or_else(|e| rust_error!(class = e.reraise_class("pkg_subset_error"), "{e}"));
```

Both must run inside a miniextendr boundary on the main thread, and the unwind
they start must not be caught with `catch_unwind` on the way. Details:
[QUOTED_ARGUMENTS.md](QUOTED_ARGUMENTS.md#handler-keeping-evaluation).

## Safety Requirements

All functions in this module require:

- Being called from the **R main thread** (they use R API calls)
- `unsafe` blocks (they call into C)

Standalone `#[miniextendr]` functions already run on the main thread (they are the default), so these calls are safe there; they are also safe within ALTREP callbacks, which run on the main thread too.

## Use Cases

- **S4 slot access**: The `s4_helpers` module uses `RCall` internally
- **Calling R functions from ALTREP callbacks**: When `elt()` needs to call R
- **Dynamic dispatch**: Building R function calls based on runtime data
- **Package interop**: Calling functions from other R packages

## See Also

- [QUOTED_ARGUMENTS.md](QUOTED_ARGUMENTS.md) -- `Quoted` / `Quosure` parameters and `eval_with_handlers`
- [CLASS_SYSTEMS.md](CLASS_SYSTEMS.md#s4-helpers-module) -- S4 helpers built on RCall
- [THREADS.md](THREADS.md) -- Main thread requirements
- [GC_PROTECT.md](GC_PROTECT.md) -- Protecting returned SEXPs
