+++
title = "Conversion Behavior Matrix"
weight = 22
description = "This document describes how miniextendr converts between R types and Rust types. Conversions are governed by three modes (normal, coerce, strict) and apply to both directions: R-to-Rust (TryFromSexp) and Rust-to-R (IntoR)."
+++

This document describes how miniextendr converts between R types and Rust types. Conversions are governed by three modes (**normal**, **coerce**, **strict**) and apply to both directions: R-to-Rust (`TryFromSexp`) and Rust-to-R (`IntoR`).

**See also**: `miniextendr-api/src/from_r.rs`, `miniextendr-api/src/into_r.rs`, `miniextendr-api/src/strict.rs`, `miniextendr-api/src/coerce.rs`

---

## Conversion Modes

### Normal Mode (default)

Each Rust type accepts exactly one R type. For example, `i32` only accepts `INTSXP`, `f64` only accepts `REALSXP`. A type mismatch produces an error.

### Coerce Mode

Coerced types (like `i64`, `u64`, `isize`, `usize`, and sub-integer types `i8`, `i16`, `u16`, `u32`, `f32`) accept multiple R types: `INTSXP`, `REALSXP`, `RAWSXP`, and `LGLSXP`. The value is extracted as the R native type, then converted to the target Rust type via `TryCoerce`. This is the default for these types -- no attribute is needed.

### Strict Mode (`#[miniextendr(strict)]`)

Only `INTSXP` and `REALSXP` are accepted. `RAWSXP` and `LGLSXP` are rejected. Additionally, output values that don't fit in R's integer range (`i32`) cause a panic (R error) instead of silently widening to `REALSXP` (`f64`).

---

## R-to-Rust Conversions (Input: TryFromSexp)

### Native Scalar Types (Normal Mode)

These types require an exact R type match. Length must be 1.

| Rust Type | Accepted R Type | On NA | On Type Mismatch |
|-----------|----------------|-------|------------------|
| `i32` | INTSXP | Error (`SexpError::Na`) — use `Option<i32>` for NA | Error |
| `f64` | REALSXP | Returns NA_real_ (specific NaN) | Error |
| `u8` | RAWSXP | No NA concept in raw | Error |
| `Rcomplex` | CPLXSXP | Returns `Rcomplex { r: NA_real_, i: NA_real_ }` | Error |
| `bool` | LGLSXP | Error (NA is not true/false) | Error |
| `Rboolean` | LGLSXP | Error (NA not representable) | Error |
| `RLogical` | LGLSXP | Returns `RLogical::Na` | Error |
| `String` | STRSXP | Error (NA_character_) | Error |
| `&str` | STRSXP | Error (NA_character_) | Error |

### Option Wrappers (Normal Mode)

`Option<T>` maps NA to `None` and NULL to `None`:

| Rust Type | Accepted R Type | On NA | On NULL |
|-----------|----------------|-------|---------|
| `Option<i32>` | INTSXP | `None` | `None` |
| `Option<f64>` | REALSXP | `None` | `None` |
| `Option<u8>` | RAWSXP | `Some(val)` (raw has no NA) | `None` |
| `Option<Rcomplex>` | CPLXSXP | `None` | `None` |
| `Option<bool>` | LGLSXP | `None` | `None` |
| `Option<Rboolean>` | LGLSXP | `None` | `None` |
| `Option<String>` | STRSXP | `None` | `None` |

### Coerced Scalar Types (Multi-Source)

These types accept `INTSXP`, `REALSXP`, `RAWSXP`, and `LGLSXP`:

| Rust Type | INTSXP | REALSXP | RAWSXP | LGLSXP | STRSXP |
|-----------|--------|---------|--------|--------|--------|
| `i8` | Narrow i32->i8 | f64->i8 (reject frac/NaN) | u8->i8 | logical->i32->i8 | Error |
| `i16` | Narrow i32->i16 | f64->i16 (reject frac/NaN) | u8->i16 | logical->i32->i16 | Error |
| `u16` | i32->u16 (reject neg) | f64->u16 (reject frac/neg/NaN) | u8->u16 | logical->i32->u16 | Error |
| `u32` | i32->u32 (reject neg) | f64->u32 (reject frac/neg/NaN) | u8->u32 | logical->i32->u32 | Error |
| `f32` | i32 as f32 | f64 as f32 | u8 as f32 | logical as f32 | Error |
| `i64` | Widen i32->i64 | f64->i64 (reject frac/NaN/Inf) | u8->i64 | logical->i32->i64 | Error |
| `u64` | i32->u64 (reject neg) | f64->u64 (reject frac/neg/NaN) | u8->u64 | logical->i32->u64 | Error |
| `isize` | Widen i32->isize | f64->i64->isize (reject frac) | u8->isize | logical->isize | Error |
| `usize` | i32->usize (reject neg) | f64->u64->usize (reject frac/neg) | u8->usize | logical->i32->usize | Error |

**Notes on coercion checks**:
- **Fractional check**: `f64` values with a non-zero fractional part are rejected (e.g., `3.14` fails)
- **NaN/Inf**: Both are rejected when converting `f64` to integer types
- **Range check**: Values outside the target type's range are rejected (e.g., 300 fails for `i8`)
- **NA propagation**: NA_integer_ and NA_real_ produce errors for non-Option types; `Option<i64>` etc. map NA to `None`

### Strict Mode Scalar Types

Only `INTSXP` and `REALSXP` accepted; `RAWSXP` and `LGLSXP` are rejected:

| Rust Type | INTSXP | REALSXP | RAWSXP | LGLSXP |
|-----------|--------|---------|--------|--------|
| `i64` (strict) | Widen i32->i64 | f64->i64 (reject frac/NaN) | **Panic** | **Panic** |
| `u64` (strict) | i32->u64 (reject neg) | f64->u64 (reject frac/neg) | **Panic** | **Panic** |
| `isize` (strict) | Delegates to i64 | Delegates to i64 | **Panic** | **Panic** |
| `usize` (strict) | Delegates to u64 | Delegates to u64 | **Panic** | **Panic** |

### Vector Types

Vector conversions (`Vec<T>`) follow the same source-type rules as scalars:

| Rust Type | Accepted R Type(s) | Element Behavior |
|-----------|--------------------|------------------|
| `Vec<i32>` / `&[i32]` | INTSXP only | Direct memcpy |
| `Vec<f64>` / `&[f64]` | REALSXP only | Direct memcpy |
| `Vec<u8>` / `&[u8]` | RAWSXP only | Direct memcpy |
| `Vec<bool>` | LGLSXP only | Each logical->bool; NA causes error |
| `Vec<String>` | STRSXP only | Each CHARSXP->String; NA causes error |
| `Vec<Option<i32>>` | INTSXP only | NA_integer_ -> None |
| `Vec<Option<f64>>` | REALSXP only | NA_real_ -> None |
| `Vec<Option<bool>>` | LGLSXP only | NA_logical -> None |
| `Vec<Option<String>>` | STRSXP only | NA_character_ -> None |
| `Vec<i64>` (strict) | INTSXP or REALSXP | Per-element checked coercion; RAWSXP/LGLSXP rejected |
| `Vec<u64>` (strict) | INTSXP or REALSXP | Per-element checked coercion; RAWSXP/LGLSXP rejected |

---

## Rust-to-R Conversions (Output: IntoR)

### Scalar Types

| Rust Type | R Output Type | Notes |
|-----------|--------------|-------|
| `i32` | INTSXP | Direct via `Rf_ScalarInteger` |
| `f64` | REALSXP | Direct via `Rf_ScalarReal` |
| `u8` | RAWSXP | Direct via `Rf_ScalarRaw` |
| `bool` | LGLSXP | `true`->1, `false`->0 |
| `Rboolean` | LGLSXP | Direct |
| `RLogical` | LGLSXP | Includes NA support |
| `String` / `&str` | STRSXP | UTF-8 encoding via `Rf_mkCharLenCE` |
| `char` | STRSXP | Single UTF-8 character as string |
| `()` | NILSXP | Returns R NULL |

### Widening Scalar Types

| Rust Type | R Output Type | Notes |
|-----------|--------------|-------|
| `i8`, `i16`, `u16` | INTSXP | Infallible widening to i32 |
| `f32`, `u32` | REALSXP | Infallible widening to f64 |

### Smart Scalar Conversion (i64, u64, isize, usize)

These types use a **smart** conversion strategy: fit in i32 -> INTSXP, otherwise -> REALSXP.

| Rust Type | Condition | R Output Type | Notes |
|-----------|-----------|--------------|-------|
| `i64` | `i32::MIN < val <= i32::MAX` | INTSXP | Exact representation |
| `i64` | Otherwise (incl. `i32::MIN`) | REALSXP | May lose precision >2^53 |
| `u64` | `val <= i32::MAX` | INTSXP | Exact representation |
| `u64` | `val > i32::MAX` | REALSXP | May lose precision >2^53 |
| `isize` | Delegates to i64 | INTSXP or REALSXP | Same rules as i64 |
| `usize` | Delegates to u64 | INTSXP or REALSXP | Same rules as u64 |

**Why `i32::MIN` is excluded from INTSXP**: In R, `i32::MIN` (`-2147483648`) is `NA_integer_`. Returning it as INTSXP would create an unintended NA value.

### Strict Output Conversion

With `#[miniextendr(strict)]`, large integer types **panic** instead of falling back to REALSXP:

| Rust Type | Condition | Strict Behavior |
|-----------|-----------|-----------------|
| `i64` | Fits in `(i32::MIN, i32::MAX]` | INTSXP (same as normal) |
| `i64` | Outside range | **Panic** (R error) |
| `u64` | `val <= i32::MAX` | INTSXP (same as normal) |
| `u64` | `val > i32::MAX` | **Panic** (R error) |
| `Vec<i64>` | All elements fit | INTSXP vector |
| `Vec<i64>` | Any element outside range | **Panic** (R error) |

### Option Types (NA Mapping)

| Rust Type | `Some(val)` | `None` |
|-----------|-------------|--------|
| `Option<i32>` | INTSXP scalar | NA_integer_ |
| `Option<f64>` | REALSXP scalar | NA_real_ |
| `Option<bool>` | LGLSXP scalar | NA_logical |
| `Option<Rboolean>` | LGLSXP scalar | NA_logical |
| `Option<String>` | STRSXP scalar | NA_character_ |
| `Option<&str>` | STRSXP scalar | NA_character_ |
| `Option<Vec<T>>` | R vector | NULL (R_NilValue) |
| `Option<HashMap<...>>` | Named list | NULL (R_NilValue) |

### Vector Types

| Rust Type | R Output Type | Notes |
|-----------|--------------|-------|
| `Vec<i32>` / `&[i32]` | INTSXP | Bulk memcpy |
| `Vec<f64>` / `&[f64]` | REALSXP | Bulk memcpy |
| `Vec<u8>` / `&[u8]` | RAWSXP | Bulk memcpy |
| `Vec<bool>` / `&[bool]` | LGLSXP | Element-wise `bool as i32` |
| `Vec<String>` | STRSXP | Element-wise CHARSXP creation |
| `Vec<Option<i32>>` | INTSXP | None -> NA_integer_ |
| `Vec<Option<f64>>` | REALSXP | None -> NA_real_ |
| `Vec<Option<bool>>` | LGLSXP | None -> NA_logical |
| `Vec<Option<String>>` | STRSXP | None -> NA_character_ |

### Smart Vector Conversion (Vec of large integers)

`Vec<i64>`, `Vec<u64>`, `Vec<isize>`, `Vec<usize>` check whether **all** elements fit in i32. If yes, the entire vector is INTSXP; otherwise, the entire vector is REALSXP.

| Rust Type | All Fit in i32? | R Output Type |
|-----------|-----------------|--------------|
| `Vec<i64>` | Yes (all in `(i32::MIN, i32::MAX]`) | INTSXP |
| `Vec<i64>` | No (any element outside) | REALSXP |
| `Vec<u64>` | Yes (all `<= i32::MAX`) | INTSXP |
| `Vec<u64>` | No | REALSXP |

### Collection Types

| Rust Type | R Output Type |
|-----------|--------------|
| `HashMap<String, V>` | Named list (VECSXP) |
| `BTreeMap<String, V>` | Named list (VECSXP) |
| `HashSet<T>` / `BTreeSet<T>` | Vector (order may vary for HashSet) |
| `VecDeque<T>` | Vector (converted to Vec first) |
| `BinaryHeap<T>` | Vector (arbitrary order) |
| `Vec<Vec<T>>` | List of vectors (VECSXP) |
| `(A, B, ...)` | Unnamed list (VECSXP), up to 8 elements (IntoR only, no TryFromSexp) |
| `PathBuf` | STRSXP (lossy UTF-8 conversion) |
| `OsString` | STRSXP (lossy UTF-8 conversion) |

### Result and Error Types

| Rust Type | `Ok(val)` | `Err(e)` |
|-----------|-----------|----------|
| `Result<T, E: Debug>` (default) | `T::into_sexp()` | **Panic** -> R error |
| `Result<T, E: Display>` (`unwrap_in_r`) | `T::into_sexp()` | `list(error = msg)` |
| `Result<T, ()>` | `T::into_sexp()` | NULL (R_NilValue) |

---

## Raw/Bytemuck Conversions (Feature-Gated)

Enabled with `features = ["raw_conversions"]`. Uses R's `RAWSXP` for binary POD data.

| Wrapper | Direction | Format | Type Tag |
|---------|-----------|--------|----------|
| `Raw<T>` | Both | Headerless bytes | No |
| `RawSlice<T>` | Both | Headerless byte sequence | No |
| `RawTagged<T>` | Both | 16-byte header + bytes | Yes (`mx_raw_type` attr) |
| `RawSliceTagged<T>` | Both | 16-byte header + byte sequence | Yes (`mx_raw_type` attr) |

**Safety checks**: length validation, alignment (copy if misaligned), magic/version validation (tagged only), type name matching (tagged only).

---

## Special Values Quick Reference

| R Value | Rust Representation | Notes |
|---------|-------------------|-------|
| `NA_integer_` | `i32::MIN` (-2147483648) | Excluded from valid i32 range; inbound NA produces `SexpError::Na` on `i32` (use `Option<i32>` to receive NA) |
| `NA_real_` | `0x7FF0000000000007A2` (specific NaN bit pattern) | Distinguished from ordinary `f64::NAN` by bit-exact comparison; ALTREP `no_na`/`sum`/`min`/`max` treat only this bit pattern as NA |
| `NA_logical_` | `i32::MIN` | Same sentinel as NA_integer_ |
| `NA_character_` | R_NaString CHARSXP | Mapped to `None` in `Option<String>` |
| `NaN` | `f64::NAN` | **Not** the same as NA_real_; passes through as valid f64 |
| `Inf` / `-Inf` | `f64::INFINITY` / `f64::NEG_INFINITY` | Valid f64 values; rejected when coercing to integers |
| `NULL` | `R_NilValue` | Mapped to `None` in `Option<T>`; `()` produces NULL |

---

## Cookbook: Common Conversion Recipes

### "I have `Vec<Option<i64>>` — how does it convert to R?"

Each element uses the smart i64 conversion. If all `Some` values fit in i32, the whole vector is INTSXP; otherwise REALSXP. `None` values become `NA_integer_` or `NA_real_` accordingly.

```rust
#[miniextendr]
fn make_nullable_ids() -> Vec<Option<i64>> {
    vec![Some(1), None, Some(42), Some(i64::MAX)]
    // -> REALSXP because i64::MAX doesn't fit in i32
}
```

### "I want to accept either integer or numeric from R"

Use a coerced type (`i64`, `u64`, `f32`) — they accept INTSXP, REALSXP, RAWSXP, and LGLSXP automatically:

```rust
#[miniextendr]
fn flexible_input(x: i64) -> i64 {
    x * 2  // works with integer(1) or numeric(1) from R
}
```

Or use `#[miniextendr(strict)]` to only accept INTSXP and REALSXP (no raw/logical):

```rust
#[miniextendr(strict)]
fn strict_input(x: i64) -> i64 { x * 2 }
```

### "I want a named list from R as a HashMap"

```rust
use std::collections::HashMap;

#[miniextendr]
fn process_config(config: HashMap<String, f64>) -> f64 {
    config.get("threshold").copied().unwrap_or(0.5)
}
```

In R: `process_config(list(threshold = 0.9, alpha = 0.05))`

### "I want to return NA for missing values"

Wrap in `Option` — `None` becomes the appropriate NA:

```rust
#[miniextendr]
fn safe_divide(a: f64, b: f64) -> Option<f64> {
    if b == 0.0 { None } else { Some(a / b) }
}
```

### "I want to return NULL on failure, not an error"

Use `Result<T, ()>`:

```rust
#[miniextendr]
fn try_parse(s: String) -> Result<i32, ()> {
    s.parse::<i32>().map_err(|_| ())
    // Ok(42) -> 42L in R; Err(()) -> NULL in R
}
```

### "I have a struct and want to pass it to R and back"

Use `#[miniextendr]` on an impl block — the struct is wrapped in an ExternalPtr:

```rust
struct Counter { n: i32 }

#[miniextendr]
impl Counter {
    fn new() -> Self { Counter { n: 0 } }
    fn increment(&mut self) { self.n += 1; }
    fn get(&self) -> i32 { self.n }
}
```

### "I want to accept R's `...` (dots)"

Use `_dots: &Dots` as the last parameter:

```rust
#[miniextendr]
fn sum_all(x: f64, _dots: &Dots) -> f64 {
    // x is the first argument; _dots captures the rest
    x  // dots are validated but not directly accessible as Rust values
}
```

For typed dots validation, see [DOTS_TYPED_LIST.md](DOTS_TYPED_LIST.md).
