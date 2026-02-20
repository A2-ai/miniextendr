# Storage‑Directed Conversion Semantics (Design Note)

Date: 2026-01-12

## Goal

Provide a **value-based** conversion layer so users can pick an R storage
(`integer`, `numeric`, `logical`, `raw`, `character`) and conversions happen
automatically when the actual values permit.

The intent is:

- If the user chooses storage, **conversions happen automatically**.
- **Runtime-checked**: if the actual value fits, convert it; if not, error.
- **No lossy escape hatch**: if you want lossy, cast it yourself first.

## Existing Building Blocks

- `coerce.rs`: `Coerce<T>` (infallible) + `TryCoerce<T>` (strict/fallible).
- `ffi::RNativeType`: maps Rust element types to R storage.
- `IntoR`: constructs R vectors/scalars once element storage is decided.

## Proposed API Surface (plan-only)

### Storage‑directed conversion trait

```rust
pub trait IntoRAs<Target> {
    type Error;
    fn into_r_as(self) -> Result<SEXP, Self::Error>;
}
```

- `Target` is a storage type: `i32` (integer), `f64` (numeric), `RLogical` (logical), `u8` (raw), `String` (character).
- Implementations provided for scalars and slices/Vecs.
- Implementations **delegate** to `TryCoerce` then `IntoR`.
- Conversion succeeds if **all values** fit the target; fails otherwise.

### No lossy escape hatch

If users want lossy conversion, they cast first:

```rust
// Value-based conversion (errors if any value doesn't fit)
vec![1_i64, 2, 3].into_r_as::<i32>()?           // OK
vec![1.5_f64].into_r_as::<i32>()?               // Error: not integral

// User wants lossy? Cast first - their responsibility.
vec![1.5_f64 as i32].into_r()                   // Truncates to 1
```

## Semantics by Storage

### Integer (`i32` / INTSXP)

Allowed:
- All integer types if within `i32` range
- `bool`/`RLogical` via 0/1 (or NA sentinel)
- `f32`/`f64` **only if** finite AND integral AND in range

Errors:
- non-finite floats
- non-integral floats
- out-of-range values

### Numeric (`f64` / REALSXP)

Allowed:
- `f32`/`f64` if finite
- integer types **only if** exactly representable in f64

Errors:
- non-finite floats
- precision loss (e.g., `i64` > 2^53)

### Logical (`RLogical` / LGLSXP)

Allowed:
- `bool` / `RLogical` (TRUE/FALSE/NA)
- integer types **only if** in {0,1,NA}

Errors:
- any other numeric values

### Raw (`u8` / RAWSXP)

Allowed:
- `u8`
- integer types within 0..=255

Errors:
- out-of-range
- missing values (raw has no NA)

### Character (`String` / STRSXP)

Allowed:
- `String`, `&str`, `Cow<str>`
- numeric and logical values via **stringification** (see below)

Errors:
- invalid UTF‑8 inputs (when applicable)

## Stringification Rules (numeric → character)

**Decision:** non-finite values are allowed and stringified.

Recommended canonical forms:

- `NaN` → "NaN"
- `+∞` → "Inf"
- `-∞` → "-Inf"

For finite values:

- Use Rust’s `to_string()` (default formatting) for now.
- If stable formatting is needed later, switch to `ryu`-style for floats.

For logical values:

- `TRUE` → "TRUE"
- `FALSE` → "FALSE"
- `NA` → "NA" (if represented via `RLogical`/Option)

## Error Type

Add a minimal error enum for conversion failures:

```rust
enum StorageCoerceError {
    Unsupported { from: &'static str, to: &'static str },
    OutOfRange { from: &'static str, to: &'static str },
    NonFinite { to: &'static str },
    PrecisionLoss { to: &'static str },
    NotIntegral { to: &'static str },
    MissingValue { to: &'static str },
    InvalidUtf8 { to: &'static str },
}
```

`TryCoerce` errors can be wrapped or mapped into this enum.

## Examples

```rust
// Integer storage - values must fit
let x = vec![1_i64, 2, 3];
let sexp = x.into_r_as::<i32>()?;               // OK: all values in range

let y = vec![1_i64 << 40];
let sexp = y.into_r_as::<i32>()?;               // Error: out of range

// Numeric storage - values must be exactly representable
let a = vec![1_i64, 2, 3];
let sexp = a.into_r_as::<f64>()?;               // OK: exactly representable

let b = vec![1_i64 << 60];
let sexp = b.into_r_as::<f64>()?;               // Error: precision loss

// Float to integer - must be integral and in range
let c = vec![1.0_f64, 2.0, 3.0];
let sexp = c.into_r_as::<i32>()?;               // OK: all integral

let d = vec![1.5_f64];
let sexp = d.into_r_as::<i32>()?;               // Error: not integral

// Character storage (stringify NaN/Inf)
let z = vec![f64::NAN, f64::INFINITY, -f64::INFINITY];
let sexp = z.into_r_as::<String>()?;            // "NaN", "Inf", "-Inf"

// User wants lossy? Cast first.
let lossy: Vec<i32> = vec![1.5_f64, 2.7].iter().map(|&x| x as i32).collect();
let sexp = lossy.into_r();                      // [1, 2] - user's responsibility
```

## Documentation Updates

- Add a short section to `docs/docs.md` describing storage‑directed conversion.
- Add this file as the definitive semantic reference.

