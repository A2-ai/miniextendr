# Type Coercion in miniextendr

This document describes the `Coerce<R>` trait system for converting Rust types to R's native scalar types.

## R's Native Scalar Types

R has a fixed set of native scalar types that can appear in vectors:

| R Type | Rust Type | SEXPTYPE |
|--------|-----------|----------|
| integer | `i32` | `INTSXP` |
| numeric/double | `f64` | `REALSXP` |
| logical | `Rboolean` | `LGLSXP` |
| raw | `u8` | `RAWSXP` |
| complex | `Rcomplex` | `CPLXSXP` |

The `RNative` marker trait identifies these types:

```rust
pub trait RNative: Copy + 'static {
    const SEXP_TYPE: SEXPTYPE;
}
```

## Core Traits

### `Coerce<R>` - Infallible Coercion

For conversions that always succeed (identity, widening):

```rust
pub trait Coerce<R> {
    fn coerce(self) -> R;
}
```

**Scalar implementations:**

| From | To | Notes |
|------|----|-------|
| `i32` | `i32` | Identity |
| `f64` | `f64` | Identity |
| `Rboolean` | `Rboolean` | Identity |
| `u8` | `u8` | Identity |
| `Rcomplex` | `Rcomplex` | Identity |
| `i8`, `i16`, `u8`, `u16` | `i32` | Widening to R integer |
| `f32`, `i8`..`u32` | `f64` | Widening to R real |
| `u8` | `u16`, `i16`, `u32` | Widening |
| `i8` | `i16` | Widening |
| `u16` | `u32` | Widening |
| `bool` | `Rboolean` | `true` → `TRUE`, `false` → `FALSE` |
| `bool` | `i32` | `true` → `1`, `false` → `0` |
| `bool` | `f64` | `true` → `1.0`, `false` → `0.0` |
| `Rboolean` | `i32` | Direct cast |

**Slice/Vec implementations (element-wise):**

| From | To | Notes |
|------|----|-------|
| `&[T]` | `Vec<R>` | Where `T: Copy + Coerce<R>` |
| `Vec<T>` | `Vec<R>` | Where `T: Coerce<R>` |

```rust
let slice: &[i8] = &[1, 2, 3];
let vec: Vec<i32> = slice.coerce();  // [1, 2, 3]

let v: Vec<i16> = vec![10, 20, 30];
let result: Vec<f64> = v.coerce();   // [10.0, 20.0, 30.0]
```

### `TryCoerce<R>` - Fallible Coercion

For conversions that may fail (narrowing, overflow, precision loss):

```rust
pub trait TryCoerce<R> {
    type Error;
    fn try_coerce(self) -> Result<R, Self::Error>;
}

pub enum CoerceError {
    Overflow,       // Value out of range
    PrecisionLoss,  // Would lose significant digits
    NaN,            // NaN cannot be converted to integer
}
```

**Built-in implementations:**

| From | To | Failure Condition |
|------|----|-------------------|
| `u32`, `u64`, `i64`, `usize`, `isize` | `i32` | Value outside `i32` range |
| `f64`, `f32` | `i32` | NaN, out of range, or has fractional part |
| `i64`, `u64`, `isize`, `usize` | `f64` | Value outside ±2^53 (precision loss) |
| All integers except `u8` | `u8` | Value outside 0..255 |
| `i8`, `i16`, `i32`, `i64`, `u32`, `u64`, `usize`, `isize` | `u16` | Value outside 0..65535 |
| `i32`, `i64`, `u16`, `u32`, `u64`, `usize`, `isize` | `i16` | Value outside `i16` range |
| `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`, `usize`, `isize` | `i8` | Value outside `i8` range |
| `f64` | `u16`, `i16`, `i8` | NaN, out of range, or has fractional part |

**Blanket impl:** `Coerce<R>` automatically implements `TryCoerce<R>` with `Error = Infallible`.

**Slice coercion:** Slices/Vecs get `TryCoerce` automatically via the blanket impl when elements have `Coerce`. For fallible element-wise coercion, use manual iteration:

```rust
// R integer slice → Rust u16 vec (common use case)
let r_ints: &[i32] = &[1, 100, 1000];
let result: Result<Vec<u16>, _> = r_ints
    .iter()
    .copied()
    .map(TryCoerce::try_coerce)
    .collect();
assert_eq!(result, Ok(vec![1u16, 100, 1000]));

// Failure case - negative values can't become u16
let bad: &[i32] = &[1, -5, 1000];
let result: Result<Vec<u16>, _> = bad
    .iter()
    .copied()
    .map(TryCoerce::try_coerce)
    .collect();
// Err(CoerceError::Overflow) - fails on -5
```

## Trait Bounds

For use in `where` clauses:

```rust
pub trait CanCoerceToInteger: Coerce<i32> {}
pub trait CanCoerceToReal: Coerce<f64> {}
pub trait CanCoerceToLogical: Coerce<Rboolean> {}
pub trait CanCoerceToRaw: Coerce<u8> {}
```

Example:

```rust
fn process_as_integer<T: CanCoerceToInteger>(value: T) -> i32 {
    value.coerce()
}

// Works with any type that can infallibly coerce to i32
process_as_integer(42i8);   // i8 → i32
process_as_integer(true);   // bool → i32
process_as_integer(100u16); // u16 → i32
```

## Usage with `#[miniextendr]`

### The `coerce` Attribute (Recommended)

Use `#[miniextendr(coerce)]` to enable automatic type coercion for non-R-native parameter types:

```rust
// Scalar coercion: R integer (i32) → u16
#[miniextendr(coerce)]
fn process_u16(x: u16) -> i32 {
    x as i32
}

// Vec coercion: R integer vector (&[i32]) → Vec<u16>
#[miniextendr(coerce)]
fn sum_u16_vec(x: Vec<u16>) -> i32 {
    x.iter().map(|&v| v as i32).sum()
}

// Float narrowing: R double (f64) → f32
#[miniextendr(coerce)]
fn process_f32(x: f32) -> f64 {
    x as f64
}
```

**Supported coercions:**

| Parameter Type | R Type | Coercion |
|----------------|--------|----------|
| `u16`, `i16`, `i8` | integer | `TryCoerce` (overflow → panic) |
| `u32`, `u64`, `i64` | integer | `TryCoerce` (overflow → panic) |
| `f32` | numeric | `Coerce` (may lose precision) |
| `Vec<u16>`, `Vec<i16>`, etc. | integer vector | element-wise `TryCoerce` |
| `Vec<f32>` | numeric vector | element-wise `Coerce` |

**Example in R:**

```r
# Works - value fits in u16
process_u16(100L)  # Returns 100

# Errors - value doesn't fit in u16
process_u16(-1L)   # Error: coercion to u16 failed: Overflow
process_u16(70000L)  # Error: coercion to u16 failed: Overflow

# Vec coercion
sum_u16_vec(c(1L, 2L, 3L))  # Returns 6
sum_u16_vec(c(1L, -1L, 3L)) # Error: coercion to Vec<u16> failed: Overflow
```

**Combining with other attributes:**

```rust
#[miniextendr(coerce, invisible)]
fn process_silently(x: u16) -> i32 {
    x as i32  // Returns invisibly
}
```

### Per-Parameter Coercion with `#[miniextendr(coerce)]`

For selective coercion, add `#[miniextendr(coerce)]` to individual parameters:

```rust
// Only coerce the first parameter
#[miniextendr]
fn process_mixed(#[miniextendr(coerce)] x: u16, y: i32) -> i32 {
    x as i32 + y  // x is coerced from R integer, y is used directly
}

// Coerce multiple specific parameters
#[miniextendr]
fn process_both(#[miniextendr(coerce)] x: u16, #[miniextendr(coerce)] y: i16, z: i32) -> i32 {
    x as i32 + y as i32 + z  // x and y coerced, z is direct R integer
}

// Coerce Vec parameter
#[miniextendr]
fn sum_u16(#[miniextendr(coerce)] values: Vec<u16>, offset: i32) -> i32 {
    values.iter().map(|&v| v as i32).sum::<i32>() + offset
}
```

**Example in R:**

```r
# x is coerced to u16, y is used as-is
process_mixed(100L, 5L)  # Returns 105

# Overflow only affects coerced parameter
process_mixed(-1L, 5L)   # Error: coercion to u16 failed
```

This is useful when you have a mix of R-native types and types that need coercion.

### Manual Coercion (Alternative)

For more control, accept R native types and coerce manually:

```rust
#[miniextendr]
fn widen_to_real(x: i32) -> f64 {
    x.coerce()  // i32 → f64, always succeeds
}

#[miniextendr]
fn try_narrow(x: f64) -> i32 {
    match TryCoerce::<i32>::try_coerce(x) {
        Ok(v) => v,
        Err(_) => i32::MIN,  // Return NA on failure
    }
}
```

**Helper functions with generic bounds:**

```rust
fn internal_helper<T: CanCoerceToInteger>(x: T) -> i32 {
    x.coerce()
}

#[miniextendr]
fn from_i8(x: i8) -> i32 {
    internal_helper(x)  // Concrete type at call site
}
```

### What Doesn't Work

**Generic `#[miniextendr]` functions:**

```rust
// THIS DOES NOT COMPILE
#[miniextendr]
fn generic_coerce<T: Coerce<i32>>(x: T) -> i32 {
    x.coerce()
}
```

**Why:** The macro generates `TryFromSexp::try_from_sexp(arg)` which requires knowing the concrete type `T` at compile time. A trait bound alone doesn't tell the macro what R type to expect.

## No Automatic R-Side Coercion

miniextendr does **NOT** automatically insert `as.integer()`, `as.numeric()`, etc. in generated R wrappers.

### Why Not?

**R has no scalars - everything is a vector (length-1 slice).**

Consider a function that modifies data in place:

```rust
#[miniextendr]
fn double_first(x: &mut [i32]) {
    x[0] *= 2;
}
```

```r
# Without coercion - works correctly
x <- c(1L, 2L, 3L)
double_first(x)
x[1]  # 2L - modified in place ✓

# With automatic coercion - BROKEN
x <- c(1.0, 2.0, 3.0)  # numeric, not integer
double_first(x)  # If wrapper did as.integer(x), it would create a COPY
x[1]  # Still 1.0 - user's data unchanged! ✗
```

Automatic coercion creates copies, silently breaking "modify in place" semantics.

### The Correct Approach

1. **Type mismatch = error** - Let users see the error and decide how to handle it
2. **Explicit coercion in R** - Users call `as.integer(x)` when they understand the copy implications
3. **Rust-side Coerce** - Use the trait for internal conversions and return values

```r
# User handles coercion explicitly
x <- c(1.0, 2.0, 3.0)
x_int <- as.integer(x)  # User knows this is a copy
double_first(x_int)
x_int[1]  # 2L - the copy was modified
```

## Newtype Wrappers with `#[derive(RNative)]`

For newtype wrappers around R native types, use the `RNative` derive macro.

### Supported Struct Forms

Both tuple structs and single-field named structs are supported:

```rust
use miniextendr_api::RNative;

// Tuple struct (most common)
#[derive(Clone, Copy, RNative)]
struct UserId(i32);

#[derive(Clone, Copy, RNative)]
struct Score(f64);

// Named single-field struct
#[derive(Clone, Copy, RNative)]
struct Temperature { celsius: f64 }
```

### Using with Coerce

The derive forwards the inner type's `SEXP_TYPE`. The newtype can then participate in coercion as a target type:

```rust
impl Coerce<UserId> for i32 {
    fn coerce(self) -> UserId {
        UserId(self)
    }
}

let id: UserId = 42.coerce();
```

### Requirements

- Must be a newtype struct (exactly one field, tuple or named)
- The inner type must implement `RNative` (`i32`, `f64`, `Rboolean`, `u8`, `Rcomplex`, or another derived type)
- Should also derive `Copy` (required by `RNative: Copy`)

## Implementing Coerce for Custom Types

```rust
use miniextendr_api::{Coerce, TryCoerce, CoerceError, RNative};

// Infallible coercion
impl Coerce<i32> for MyType {
    fn coerce(self) -> i32 {
        self.value as i32
    }
}

// Fallible coercion
impl TryCoerce<i32> for MyOtherType {
    type Error = CoerceError;

    fn try_coerce(self) -> Result<i32, CoerceError> {
        if self.value > i32::MAX as i64 {
            Err(CoerceError::Overflow)
        } else {
            Ok(self.value as i32)
        }
    }
}
```

## Comparison with R's Coercion

miniextendr's `TryCoerce` is **stricter** than R's `coerceVector()`. This is intentional - Rust-idiomatic explicit failure over silent data loss.

| Conversion | R Behavior | miniextendr Behavior |
|------------|------------|----------------------|
| `42.7` → integer | Truncates to `42` | `Err(PrecisionLoss)` |
| `1e20` → integer | `NA` with warning | `Err(Overflow)` |
| `NaN` → integer | `NA` | `Err(NaN)` |
| `300` → raw | `0` with warning | `Err(Overflow)` |
| `-5` → raw | `0` with warning | `Err(Overflow)` |
| `NA` → raw | `0` with warning | `Err(Overflow)` |

**R source reference** (`src/main/coerce.c`):

```c
// IntegerFromReal - just truncates, no fractional check
int IntegerFromReal(double x, int *warn) {
    if (ISNAN(x)) return NA_INTEGER;
    if (x >= INT_MAX+1. || x <= INT_MIN) {
        *warn |= WARN_INT_NA;
        return NA_INTEGER;
    }
    return (int) x;  // Truncates!
}

// coerceToRaw - out of range becomes 0
if (tmp == NA_INTEGER || tmp < 0 || tmp > 255) {
    tmp = 0;
    warn |= WARN_RAW;
}
```

**To match R's truncation behavior**, use `as` cast after bounds check:

```rust
fn r_style_to_int(x: f64) -> i32 {
    if x.is_nan() { return i32::MIN; }  // NA
    if x >= (i32::MAX as f64 + 1.0) || x <= i32::MIN as f64 {
        return i32::MIN;  // NA
    }
    x as i32  // Truncates like R
}
```

## Summary

| Use Case | Solution |
|----------|----------|
| Convert Rust types internally | `Coerce<R>` / `TryCoerce<R>` |
| Generic helper functions | Trait bounds (`CanCoerceToInteger`, etc.) |
| R → Rust at boundary | Explicit types, no auto-coercion |
| Rust → R return values | `Coerce<R>` works fine |
| R `i32` slice → Rust `u16` vec | `slice.iter().copied().map(TryCoerce::try_coerce).collect()` |
| Mutable slice parameters | **Never auto-coerce** - breaks semantics |
| Match R's truncation | Use `as` cast after bounds check |

The `Coerce<R>` trait system provides type-safe conversions within Rust while respecting R's copy-on-coerce semantics at the language boundary.
