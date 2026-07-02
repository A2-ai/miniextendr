---
name: miniextendr-conversions
description: Use when passing values between R and Rust in a miniextendr package — choosing argument/return types for #[miniextendr] functions, NA handling, Option<T>, logical three-state values, integer widening, strict mode, or errors like "expected INTSXP" / unexpected NA / doubles coming back where integers went in.
---

# R ↔ Rust type conversions

Every argument of a `#[miniextendr]` function is converted from an R value
(SEXP) to its Rust type; every return value is converted back. You never call
the conversion traits (`TryFromSexp` in, `IntoR` out) directly — you just
pick types, and the framework does the rest. Getting the types right is the
whole game.

## The basic mapping

| R type | Rust argument | Notes |
|---|---|---|
| `integer()` | `i32`, `Vec<i32>`, `&[i32]` | `NA_integer_` is an error unless `Option` |
| `double()` | `f64`, `Vec<f64>`, `&[f64]` | `NA_real_` passes through as an NA NaN |
| `logical()` | `bool` (errors on NA) or `RLogical` (three-state) | |
| `character()` | `&str`, `String`, `Vec<String>` | `NA_character_` errors unless `Option` |
| `raw()` | `u8`, `Vec<u8>` | |
| any + NA/NULL | `Option<T>` | NA and NULL both become `None` |
| `list()` | `Dots`, typed structs via derives, `SEXP` | see the dataframe/classes skills |

Return types mirror the same table. `&[T]` / `&str` arguments borrow the R
data with no copy — prefer them for read-only inputs. No lifetime annotations
needed; explicit `<'a>` lifetimes are also fine (type/const generics are not).

## Three conversion modes

- **Normal** — exact types. `i32` accepts only `integer()`, `f64` only
  `double()`. Mismatch → clear R error.
- **Coerce** (automatic for non-native widths) — `i8`, `i16`, `u16`, `u32`,
  `f32`, `i64`, `u64`, `isize`, `usize` accept integer, double, raw, or
  logical input and convert with range checking. Out-of-range → error, not
  truncation.
- **Strict** — opt in with `#[miniextendr(strict)]`. Large integer types then
  accept only integer/double input, and a returned `i64` that doesn't fit in
  R's integer range raises an error instead of silently widening to double.

Large-integer output without strict mode uses smart widening: fits in i32 →
`integer()`, otherwise → `double()` (precision loss possible above 2^53).

## NA handling — the part everyone trips on

R's NAs are type-specific sentinels, and Rust types must opt in to them:

- `NA_integer_` **is** `i32::MIN`. A plain `i32` argument rejects it with an
  NA error; `Option<i32>` receives `None`. Never return `i32::MIN` as a
  "real" value — R prints it as `NA`.
- `NA_real_` is one specific NaN bit pattern. A plain `f64` receives it (it
  *is* a valid f64); use `Option<f64>` to make NA explicit as `None`.
  `f64::is_nan()` cannot distinguish NA from an ordinary NaN.
- Logical NA is a **third state** — R logicals are ints underneath. `bool`
  errors on NA; use `RLogical` (`True` / `False` / `Na`) or `Option<bool>`.
- `NA_character_` errors for `String`/`&str`; use `Option<String>`.

Vectors with NAs: `Vec<Option<T>>` preserves element-wise NA. `Vec<String>`
from a character vector containing NA is an error — use
`Vec<Option<String>>`.

Output direction: scalar `Option<T>` → `None` becomes the matching NA;
`Option<Vec<T>>` → `None` becomes `NULL` (not NA). Downstream R code checking
`is.null()` vs `is.na()` cares about this difference.

## Choosing a signature (decision tree)

```
Need to accept NA?                 → wrap in Option<T> (or Vec<Option<T>>)
Read-only vector input?            → &[f64] / &[i32] / &str   (zero-copy)
Mutating or keeping the data?      → Vec<T> / String          (owned copy)
Logical that can be NA?            → RLogical
Caller may pass integer OR double? → i64 / f64 (coerce mode handles both)
Exact-type contract?               → i32/f64 + optionally #[miniextendr(strict)]
Heterogeneous / structured data?   → see miniextendr-dataframe (rows) or
                                     miniextendr-classes (stateful objects)
```

## Errors and panics

Conversion failures and `panic!()` in your Rust code both surface as ordinary
R errors (classed conditions carrying the message). Idiomatic error handling:

```rust
#[miniextendr]
pub fn checked_sqrt(x: f64) -> f64 {
    if x < 0.0 {
        panic!("x must be non-negative, got {x}");
    }
    x.sqrt()
}
```

```r
tryCatch(checked_sqrt(-1), error = conditionMessage)
#> "x must be non-negative, got -1"
```

Returning `Result<T, E>` (with `E: Display`) also works: `Err` becomes an R
error. Never call R's C error functions directly from Rust (the MXL300 lint
blocks it) — `panic!` is the supported path and runs Rust destructors first.

## Pitfalls

- **`1` in R is a double.** `f(1)` fails a strict `i32` argument; `f(1L)`
  passes. Coerce-mode types (`i64` etc.) accept both.
- **`i32::MIN` is NA** — see above; it is excluded from valid `i32` range.
- **`bool` in generic code**: R logicals are i32-based three-state, so `bool`
  is deliberately not part of the native-type blanket impls. Generic code
  over native numeric types won't cover it; handle logicals explicitly.
- **Coerce mode accepts logicals**: `i64` happily receives `TRUE` as `1`.
  If that's too loose for your API, add `#[miniextendr(strict)]`.
- **Empty vectors are fine** as `&[T]` / `Vec<T>` (you get an empty
  slice/vec). If you write `unsafe` code against raw R data pointers
  yourself, beware: R returns a non-null sentinel pointer for length-0
  vectors — always go through the framework's slice helpers instead.
- **Factors are integers + levels**: a factor arrives as its integer codes
  unless you take it as a dedicated factor-aware type or convert in R first
  (`as.character()` / `as.integer()` at the call site is often simplest).

## Where to look things up

- Full conversion matrix (every R type × Rust type × mode):
  https://a2-ai.github.io/miniextendr — "Conversion matrix" manual page.
- Structured data (data.frames of rows): `miniextendr-dataframe` skill.
- Stateful Rust objects held by R: `miniextendr-classes` skill.
