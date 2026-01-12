# Storage‑Directed Conversion Semantics (Design Note)

Date: 2026-01-12

## Goal
Provide a **strict-by-default** conversion layer so users can pick an R storage
(`integer`, `numeric`, `logical`, `raw`, `character`) and avoid manual coercions.
This layer should compose the existing `TryCoerce` + `IntoR` machinery.

The intent is:

- If the user chooses storage, **conversions happen automatically**.
- **Strict by default** (errors on precision loss, non-finite, out-of-range, etc.).
- **Lossy escape hatch** keeps current `IntoR` behavior when desired.

## Existing Building Blocks

- `coerce.rs`: `Coerce<T>` (infallible) + `TryCoerce<T>` (strict/fallible).
- `ffi::RNativeType`: maps Rust element types to R storage.
- `IntoR`: constructs R vectors/scalars once element storage is decided.

## Proposed API Surface (plan-only)

### 1) Storage‑directed conversion trait

```rust
pub trait IntoRAs<Target> {
    type Error;
    fn into_r_as(self) -> Result<SEXP, Self::Error>;
}
```

- `Target` is a storage type: `i32` (integer), `f64` (numeric), `RLogical` (logical), `u8` (raw), `String` (character).
- Implementations should be provided for scalars and slices/Vecs.
- Implementations **delegate** to `TryCoerce` then `IntoR`.

### 2) Lossy escape hatch

```rust
pub trait IntoRAsLossy<Target> {
    fn into_r_as_lossy(self) -> SEXP;
}
```

- Uses existing `IntoR` or `Coerce` paths (current behavior).
- No new error types; explicit opt-in to lossy conversions.

### 3) Convenience adapters

- `as_r_*` style on common wrappers if desired (optional, not required).
- Keep the core of the system minimal: `TryCoerce + IntoR`.

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
- integer types **only if** exactly representable (strict)

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

Add a minimal error enum to map strict failures:

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
// Strict integer storage
let x = vec![1_i64, 2, 3];
let sexp = x.into_r_as::<i32>()?; // error if out of range

// Strict numeric storage
let y = vec![1_i64 << 60];
let sexp = y.into_r_as::<f64>()?; // error: precision loss

// Character storage (stringify NaN/Inf)
let z = vec![f64::NAN, f64::INFINITY, -f64::INFINITY];
let sexp = z.into_r_as::<String>()?; // "NaN", "Inf", "-Inf"
```

## Documentation Updates

- Add a short section to `docs/docs.md` describing storage‑directed conversion.
- Add a note to `docs/COERCE.md` or `docs/COERCE_AND_INTO_R_REVIEW.md` clarifying strict vs lossy.
- Add this file as the definitive semantic reference.

