# Dots and typed_list! Validation

This document describes miniextendr's support for R's `...` (dots) arguments and the `typed_list!` macro for structured validation.

## Overview

When an R function accepts `...`, miniextendr converts it to a `&Dots` parameter. The `typed_list!` macro provides compile-time specification of expected list structure, with runtime validation.

## Basic Dots Usage

When you use `name: ...` in a function signature, miniextendr replaces it with a `name: &Dots` parameter (see [Named Dots](#named-dots)):

```rust
#[miniextendr]
pub fn count_args(args: ...) -> i32 {
    // `args: &Dots` is created from `args: ...`.
    // Access the underlying list:
    let list = args.as_list();
    list.len() as i32
}
```

Write `_: ...` (or `_args: ...`) when the function accepts dots it ignores:

```rust
#[miniextendr]
pub fn first_only(x: i32, _: ...) -> i32 {
    x
}
```

`_: ...` binds the synthetic name `__miniextendr_dots`, which is not meant for your code; name the dots when the body reads them. The R formal is plain `...` either way.

Don't write a bare `...` with no pattern. rustc rejects it before `#[miniextendr]` runs (the `varargs_without_pattern` lint, deny by default, [rust-lang/rust#145544](https://github.com/rust-lang/rust/issues/145544)) and suggests `_: ...`.

### Dots Methods

- `as_list(&self) -> List` - Fast unchecked conversion to List
- `try_list(&self) -> Result<List, SexpTypeError>` - Type-checked conversion (a list; a pairlist is refused, as `List` refuses one). Names are not checked: `f(a = 1, a = 2)` reaches the body, and `List::first_duplicate_name` finds the repeat
- `typed(&self, spec: TypedListSpec) -> Result<TypedList, TypedListError>` - Validate against a spec

### Named Dots

You give dots a name with the `name: ...` syntax, which becomes a `name: &Dots` parameter:

```rust
#[miniextendr]
pub fn my_func(args: ...) -> i32 {
    args.as_list().len() as i32
}
```

Writing the parameter yourself, `args: &Dots`, is the same thing. The name stays on the Rust side: the R formal is always plain `...`.

### Formals after `...`

The parameter of type `&Dots` is R's `...` at its own position, so formals can follow it. Rust's `...` syntax only parses as the last parameter; spell a formal after the dots with an explicit `&Dots` parameter:

```rust
/// @param x A number.
/// @param ... Values to write.
/// @param overwrite Whether to overwrite.
#[miniextendr]
pub fn write_all(
    x: i32,
    rest: &Dots,
    #[miniextendr(default = "FALSE")] overwrite: bool,
) -> String {
    format!("x = {x}, {} values, overwrite = {overwrite}", rest.len())
}
```

```r
write_all <- function(x, ..., overwrite = FALSE) {
  # ...
  .Call(C_pkg_write_all, .call = sys.call(), x, list(...), overwrite)
}
```

The R formals and the `.Call()` arguments follow the Rust order. R matches a formal after `...` by its exact name only:

```r
write_all(1L, 2, 3, overwrite = TRUE)   # the dots hold 2 and 3
write_all(1L, over = TRUE)              # `over` is a dot; overwrite stays FALSE
write_all(1L, TRUE)                     # a positional extra is a dot too
```

The same holds for methods in every class system: `fn collect(&self, n: i32, rest: &Dots, flag: bool)` has the R formals `n, ..., flag` (after the receiver for S3, S4 and S7), with no second dispatch `...`.

- A function takes at most one `...`. A second `&Dots` parameter, Rust `...` next to an explicit `&Dots`, or a `&Dots` next to a [`LazyDots`](#dots-left-unforced-lazydots), is a compile error.
- Rust `...` with another parameter after it is a compile error that names the `&Dots` spelling, and `miniextendr-lint`'s "failed to parse" warning for that file ends with the same hint.
- The dots parameter takes no default, `match_arg` or check; `Missing<&Dots>` is refused. The dots are always present.
- On a `call = caller` or `call_arg` wrapper, `.call` goes last, after the dots and after any formal that follows them: `function(x, ..., overwrite = FALSE, .call = NULL)`.
- `#[miniextendr(dots = typed_list!(...))]` reads an explicit `&Dots` the same way it reads `...`.
- A function with dots runs on the R main thread, even under `worker`: the dots are an R list.

## Dots left unforced: `LazyDots`

A `&Dots` parameter receives `list(...)`, which R evaluates before the Rust body runs: every element is forced, and an empty argument (`f(a = )`, the empty positions of `x[1, , , ]`) stops the call with `argument is missing, with no default`. A parameter of type `LazyDots` is R's `...` at its position too, but the wrapper passes its own frame, `environment()`, and evaluates nothing on the way in (#1892). The body counts and names the dots, sees which are empty, reads what was written, and forces the elements it wants, each in its own environment.

```rust
use miniextendr_api::{LazyDots, Missing, Quoted, SEXP, miniextendr};

/// @export
#[miniextendr(s3(generic = "subset", class = "mx_lazy"))]
pub fn mx_lazy_subset(
    x: SEXP,
    subset: Missing<Quoted>,
    rest: LazyDots,
) -> Result<SEXP, LazyUnsupportedArgs> {
    if !rest.is_empty() {
        // Names the extra arguments; none of them is evaluated.
        return Err(unsupported("subset", &rest));
    }
    // ... keep the rows where `subset` is TRUE
}
```

```r
subset.mx_lazy <- function(x, subset, ...) {
  .Call(C_pkg_mx_lazy_subset, .call = environment(), x,
        if (missing(subset)) quote(expr=) else list(substitute(subset), parent.frame()),
        environment())
  # ...
}
```

`subset(x, TRUE, select = ID)` then raises the method's own `mx_lazy_unsupported_args` error naming `select`, where `list(...)` would have stopped on `object 'ID' not found`.

| Method | R equivalent | Forces |
|---|---|---|
| `len()` / `is_empty()` | `...length()` | nothing |
| `names() -> Vec<Option<&str>>` | `...names()`; `None` for an unnamed element | nothing |
| `is_missing_arg(i) -> bool` | element `i` of `substitute(list(...))` is the empty argument | nothing |
| `expr(i) -> SEXP` | element `i` of `substitute(list(...))` | nothing |
| `force(i) -> SEXP` | `..k`, with `k = i + 1` | element `i` |
| `try_force(i) -> Result<SEXP, REvalError>` | `..k`, an R error returned as `Err` | element `i` |

- **Indices are 0-based**, as in `List::get_index`: element `i` is R's `..k` with `k = i + 1`. An index past the end panics before anything is forced, as slice indexing does.
- **Forwarded dots.** Each element keeps the environment it was written in, also when another function passed its own dots on (`g <- function(...) f(x, ...)`): `force(i)` forces the element's promise there, and `expr(i)` gives what the caller of `g` wrote.
- **Empty and missing elements.** `is_missing_arg(i)` is `true` only for a literally empty slot, the element R stores as the missing argument: `f(a = )`, `update(x, select = )`, the empty positions of `x[1, , , ]`. That is what R 4.6's `R_GetDotType()` reports as missing. An element forwarded from a missing argument is not empty: in `g <- function(a) f(1, a)` called as `g()`, element 1 is the promise of `a`, so `is_missing_arg(1)` is `false`, and forcing it raises R's own `argument "a" is missing, with no default`, as `list(...)` would.
- **Forcing an empty element** raises `argument "..k" is missing, with no default` (classes `evalError`, `missingArgError` from R 4.6, `simpleError` before), with the generated function's own call as its call. A body that accepts empty arguments checks `is_missing_arg(i)` first.
- **Conditions.** `force(i)` evaluates through `eval_with_handlers` (see [QUOTED_ARGUMENTS.md](QUOTED_ARGUMENTS.md)): the caller's `withCallingHandlers()` / `suppressWarnings()` see what forcing signals, and an error, or the exit of a `tryCatch()` handler, unwinds through the Rust frames (running their destructors) and carries on in R as the original condition. `try_force(i)` evaluates through `try_eval_with_handlers`: warnings and messages still reach the caller's handlers, any other exit unwinds as under `force(i)`, and an R error comes back as `Err(REvalError)` for the body to handle (an empty element included).
- **Caching.** Forced values are not cached in Rust: R stores a promise's value in the promise, so forcing an element twice evaluates it once. The count and the expressions are read once per `LazyDots` value: the first `is_missing_arg` / `expr` call evaluates `substitute(list(...))` in the frame and the value keeps the list, so a loop over the elements copies the expressions once.
- **R versions.** Every method runs a base R call in the frame (`...length()`, `...names()`, `substitute(list(...))`, `..k`) through `eval_with_handlers`, so the same code runs on R 4.4 and later. It uses neither R 4.6's dots C API nor the non-API promise accessors; #1898 tracks moving to that API, and a per-element `env(i)`, once a build can target R 4.6.

Where it is accepted:

- Standalone functions, `s3(...)` functions, and the impl-block methods of every class system: env, R6, S3, S4 (also through S4's `.local` rewrite), S7 (dispatch and the `<Class>_<method>` shortcut) and vctrs (static methods and protocol methods such as `format()`). The R formals are the same as for `&Dots`; the `.Call()` passes `environment()` at the position of `...`.
- The parameter is taken by value with the whole type `LazyDots` (optionally `LazyDots<'_>` or a path ending in it). `&LazyDots`, `Option<LazyDots>` and `Missing<LazyDots>` are compile errors: the dots are always present (`len()` is 0 when the call passes none).
- It takes no per-parameter option (`default`, `coerce`, `match_arg`, `no_na`, ...) and no `dots = typed_list!(...)`, which validates a forced list.
- Trait methods and `extern "C-unwind"` functions refuse it: a trait method's arguments cross the trait ABI as converted values, and `.Call()` reaches an `extern` function without a generated wrapper.
- A function with a `LazyDots` parameter runs on the R main thread, even under `worker`, and the value is `!Send`.
- A function without one keeps its wrapper byte for byte.

`rpkg/src/rust/lazy_dots_tests.rs` has the fixtures (the readers and forcers, `subset.mx_lazy`, `update.mx_lazy`, `` `[.mx_lazy_grid` `` and one class per class system), and `rpkg/tests/testthat/test-lazy-dots.R` tests them.

## typed_list! Macro

The `typed_list!` macro creates a `TypedListSpec` for validating list structure.

### Basic Syntax

```rust
typed_list!(
    field_name => type_spec,
    another_field => type_spec,
    optional_field? => type_spec,  // ? marks optional
)
```

### Type Specifications

| Syntax | Description |
|--------|-------------|
| `numeric()` | Real/double vector, any length |
| `numeric(4)` | Real/double vector, exactly 4 elements |
| `integer()` | Integer vector |
| `logical()` | Logical vector |
| `character()` | Character vector |
| `raw()` | Raw vector |
| `complex()` | Complex vector |
| `list()` | List (VECSXP) or `NULL`; a pairlist is refused, as `List` refuses one |
| `"data.frame"` | Object inheriting from class |
| `"my_class"` | Any class name as string literal |

### Strict Mode

By default, extra fields are allowed. Use `@exact;` for strict validation:

```rust
typed_list!(@exact;
    x => numeric(),
    y => numeric()
)
// Extra fields like `z` will cause an error
```

## Manual Validation

Call `.typed()` explicitly in your function body:

```rust
use miniextendr_api::typed_list;

#[miniextendr]
pub fn validate_args(dots: ...) -> Result<String, String> {
    let args = dots.typed(typed_list!(
        alpha => numeric(4),
        beta => list(),
        gamma? => character()  // optional
    )).map_err(|e| e.to_string())?;

    let alpha: Vec<f64> = args.get("alpha").map_err(|e| e.to_string())?;
    let gamma: Option<String> = args.get_opt("gamma").map_err(|e| e.to_string())?;

    Ok(format!("alpha has {} elements", alpha.len()))
}
```

## Attribute Sugar (Recommended)

Use `#[miniextendr(dots = typed_list!(...))]` for automatic validation:

```rust
#[miniextendr(dots = typed_list!(x => numeric(), y => numeric()))]
pub fn my_func(_: ...) -> String {
    // `dots_typed` is automatically created and validated
    let x: f64 = dots_typed.get("x").expect("x");
    let y: f64 = dots_typed.get("y").expect("y");
    format!("x={}, y={}", x, y)
}
```

This injects validation at the start of the function body, over the dots binding (`__miniextendr_dots` for `_: ...`, otherwise the name of the `name: ...` or `name: &Dots` parameter, at any position):
```rust
let dots_typed = __miniextendr_dots
    .typed(typed_list!(...))
    .unwrap_or_else(|e| panic!("dots validation failed: {e}"));
```

### With Optional Fields

```rust
#[miniextendr(dots = typed_list!(
    name => character(),
    greeting? => character()
))]
pub fn greet(_: ...) -> String {
    let name: String = dots_typed.get("name").expect("name");
    let greeting: Option<String> = dots_typed.get_opt("greeting").expect("greeting");
    let greeting = greeting.unwrap_or_else(|| "Hello".to_string());
    format!("{}, {}!", greeting, name)
}
```

## TypedList Methods

After validation, `TypedList` provides typed accessors:

- `get<T>(&self, name: &str) -> Result<T, TypedListError>` - Get required field
- `get_opt<T>(&self, name: &str) -> Result<Option<T>, TypedListError>` - Get optional field
- `get_raw(&self, name: &str) -> Result<SEXP, TypedListError>` - Get raw SEXP
- `as_list(&self) -> List` - Get underlying List

`get::<T>()` and `get_opt::<T>()` accept **both** scalar and vector/collection
target types. `T` can be a scalar (`f64`, `i32`, `String`, `bool`, …) **or** a
vector (`Vec<f64>`, `Vec<i32>`, `Vec<String>`, `Vec<Option<T>>`, …) — anything
implementing `TryFromSexp` with a `Display` error type:

```rust
let survive: Vec<i32> = args.get("survive")?;       // integer() field -> Vec<i32>
let weights: Vec<f64> = args.get("weights")?;       // numeric() field -> Vec<f64>
let labels: Option<Vec<String>> = args.get_opt("labels")?;  // optional character()
```

## Error Types

`TypedListError` variants:

| Variant | Description |
|---------|-------------|
| `NotList` | Input was not a list |
| `Missing { name }` | Required field is missing |
| `WrongType { name, expected, actual }` | Field has wrong type |
| `WrongLen { name, expected, actual }` | Field has wrong length |
| `ExtraFields { names }` | Extra fields in strict mode |
| `DuplicateNames { name }` | Duplicate field names |

## R Usage

From R, call functions with named arguments:

```r
# Valid
validate_args(alpha = c(1.0, 2.0, 3.0, 4.0), beta = list(1, 2))

# Missing required field -> error
validate_args(beta = list(1, 2))
# Error: missing required field: "alpha"

# Wrong type -> error
validate_args(alpha = 1:4, beta = list())  # integer instead of numeric
# Error: field "alpha" has wrong type: expected numeric, got integer

# Wrong length -> error
validate_args(alpha = c(1.0, 2.0), beta = list())  # 2 elements instead of 4
# Error: field "alpha" has wrong length: expected 4, got 2
```
