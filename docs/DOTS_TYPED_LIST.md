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
- `try_list(&self) -> Result<List, SexpTypeError>` - Type-checked conversion (a list, or a pairlist coerced to one). Names are not checked: `f(a = 1, a = 2)` reaches the body, and `List::first_duplicate_name` finds the repeat
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

- A function takes at most one `...`. A second `&Dots` parameter, or Rust `...` next to an explicit `&Dots`, is a compile error.
- Rust `...` with another parameter after it is a compile error that names the `&Dots` spelling, and `miniextendr-lint`'s "failed to parse" warning for that file ends with the same hint.
- The dots parameter takes no default, `match_arg` or check; `Missing<&Dots>` is refused. The dots are always present.
- On a `call = caller` or `call_arg` wrapper, `.call` goes last, after the dots and after any formal that follows them: `function(x, ..., overwrite = FALSE, .call = NULL)`.
- `#[miniextendr(dots = typed_list!(...))]` reads an explicit `&Dots` the same way it reads `...`.
- A function with dots runs on the R main thread, even under `worker`: the dots are an R list.

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
| `list()` | List (VECSXP or pairlist) |
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
