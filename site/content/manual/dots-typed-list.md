+++
title = "Dots and typed_list! Validation"
weight = 27
description = "This document describes miniextendr's support for R's ... (dots) arguments and the typed_list! macro for structured validation."
+++

This document describes miniextendr's support for R's `...` (dots) arguments and the `typed_list!` macro for structured validation.

## Overview

When an R function accepts `...`, miniextendr converts it to a `&Dots` parameter. The `typed_list!` macro provides compile-time specification of expected list structure, with runtime validation.

## Basic Dots Usage

When you use `...` in a function signature, miniextendr creates a parameter named `_dots` of type `&Dots`:

```rust
#[miniextendr]
pub fn my_func(...) -> i32 {
    // `_dots: &Dots` is automatically created from `...`
    // Access the underlying list:
    let list = _dots.as_list();
    list.len() as i32
}
```

The underscore prefix (`_dots`) suppresses unused variable warnings if you don't use it.

### Dots Methods

- `as_list(&self) -> List` - Fast unchecked conversion to List
- `try_list(&self) -> Result<List, ListFromSexpError>` - Validated conversion
- `typed(&self, spec: TypedListSpec) -> Result<TypedList, TypedListError>` - Validate against a spec

### Named Dots

You can give dots a custom name using the `name @ ...` syntax (instead of the default `_dots`):

```rust
#[miniextendr]
pub fn my_func(args @ ...) -> i32 {
    // Use `args` instead of `_dots`
    args.as_list().len() as i32
}
```

This is useful when you want a more descriptive name or need to avoid the underscore prefix.

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
pub fn validate_args(...) -> Result<String, String> {
    let args = _dots.typed(typed_list!(
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
pub fn my_func(...) -> String {
    // `dots_typed` is automatically created and validated
    let x: f64 = dots_typed.get("x").expect("x");
    let y: f64 = dots_typed.get("y").expect("y");
    format!("x={}, y={}", x, y)
}
```

This injects validation at the start of the function body:
```rust
let dots_typed = _dots.typed(typed_list!(...)).expect("dots validation failed");
```

### With Optional Fields

```rust
#[miniextendr(dots = typed_list!(
    name => character(),
    greeting? => character()
))]
pub fn greet(...) -> String {
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
