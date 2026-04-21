+++
title = "Type Conversions"
weight = 3
description = "R-Rust type mappings, NA handling, and coercion rules"
+++

## Core Traits

| Trait | Direction | Notes |
|-------|-----------|-------|
| `TryFromSexp` | R -> Rust | Fallible; `try_from_sexp` (checked) or `try_from_sexp_unchecked` |
| `IntoR` | Rust -> R | `into_sexp` (checked) or `into_sexp_unchecked` |
| `IntoRAs<Target>` | Rust -> R | Explicit target type (e.g., force `i64` -> `INTSXP`) |
| `Coerce<R>` | Rust -> Rust | Infallible widening before R conversion |

## Scalar Types

| R Type | Rust Type | Notes |
|--------|-----------|-------|
| `integer` (length 1) | `i32` | NA -> panic |
| `numeric` (length 1) | `f64` | NA preserved as `NA_REAL` |
| `logical` (length 1) | `bool` | NA -> panic |
| `character` (length 1) | `String`, `&str` | NA -> panic |
| `raw` (length 1) | `u8` | No NA in raw |
| `complex` (length 1) | `Rcomplex` | Has real/imag NA |

## Vector Types

| R Type | Rust Type | Notes |
|--------|-----------|-------|
| `integer` | `Vec<i32>`, `&[i32]` | NA = `i32::MIN` |
| `numeric` | `Vec<f64>`, `&[f64]` | NA = special bit pattern |
| `logical` | `Vec<i32>` | TRUE=1, FALSE=0, NA=`i32::MIN` |
| `character` | `Vec<String>` | NA -> panic |
| `raw` | `Vec<u8>`, `&[u8]` | No NA |
| `list` | Various | See below |

## NA Handling with `Option<T>`

Use `Option<T>` to handle NA values safely:

```rust
#[miniextendr]
pub fn replace_na(x: Vec<Option<f64>>, replacement: f64) -> Vec<f64> {
    x.into_iter()
        .map(|v| v.unwrap_or(replacement))
        .collect()
}
```

## Zero-Copy Slices

For read-only access, use slice references for zero-copy access to R's vector data:

```rust
#[miniextendr]
pub fn sum_slice(x: &[f64]) -> f64 {
    x.iter().sum()
}
```

## Container Types

Multiple container types are supported:

| Container | Notes |
|-----------|-------|
| `Vec<T>` | Owned, heap-allocated |
| `&[T]` | Zero-copy slice into R's data |
| `Box<[T]>` | Owned boxed slice |
| `Cow<[T]>` | Copy-on-write, ALTREP-backed |

## Coercion

The `Coerce` trait handles element-wise widening on slices:

```rust
use miniextendr_api::coerce::Coerce;

// Element-wise i8 -> i32
let slice: &[i8] = &[1, 2, 3];
let vec: Vec<i32> = slice.coerce();
```

Coercion is available for:
- `i32` -> `f64`
- `bool` -> `i32`, `f64`
- `u8` -> `i32`, `f64`

## Strict Mode

Use `#[miniextendr(strict)]` to reject lossy conversions for wide integer types:

```rust
#[miniextendr(strict)]
pub fn exact_i64(x: i64) -> i64 {
    x
}
```

In strict mode, only `INTSXP` and `REALSXP` are accepted as input -- `RAWSXP` and `LGLSXP` are rejected.

## Conversion-Preference Derives

For custom types, derive macros select the `IntoR` conversion path:

| Derive | `IntoR` result |
|--------|----------------|
| `#[derive(PreferList)]` | R named list |
| `#[derive(PreferDataFrame)]` | R data frame |
| `#[derive(PreferExternalPtr)]` | R external pointer |
| `#[derive(PreferRNativeType)]` | Native SEXP (e.g., `INTSXP`) |

## serde: Native Serialization

The `serde` feature provides direct Rust-to-R serialization without JSON intermediaries:

```rust
#[derive(serde::Serialize, serde::Deserialize)]
struct Point { x: f64, y: f64 }

#[miniextendr]
pub fn make_point() -> Point {
    Point { x: 1.0, y: 2.0 }
}
// Returns: list(x = 1.0, y = 2.0) in R
```

| Feature | `serde_json` (JSON string) | `serde` (native R) |
|---------|---------------------------|-------------------|
| Intermediate format | JSON string | None |
| Type preservation | No (all numbers -> f64) | Yes (i32 stays i32) |
| NA handling | Limited | Full support via `Option<T>` |
| Performance | Extra parse/stringify | Direct conversion |
