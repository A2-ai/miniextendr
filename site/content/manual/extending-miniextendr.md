+++
title = "Extending miniextendr"
weight = 44
description = "This guide explains how to extend miniextendr with custom types, enabling them to be passed between Rust and R."
+++

This guide explains how to extend miniextendr with custom types, enabling them to be passed between Rust and R.

## Quick Start

To make your type work with miniextendr, you have two main options:

1. **Implement `RNativeType`** - For types with the same memory layout as R's native types
2. **Implement `TryFromSexp`/`IntoR` directly** - For types requiring custom conversion logic

## Option 1: RNativeType (Recommended)

If your type has the same memory layout as `i32`, `f64`, `u8`, or `Rcomplex`, implement `RNativeType` to automatically get ~150+ conversions.

### Example: Newtype Wrapper

```rust
use miniextendr_api::ffi::{RNativeType, SEXP, SEXPTYPE};

/// A temperature in Celsius, stored as f64
#[repr(transparent)]
pub struct Celsius(pub f64);

impl RNativeType for Celsius {
    const SEXP_TYPE: SEXPTYPE = SEXPTYPE::REALSXP;

    unsafe fn dataptr_mut(sexp: SEXP) -> *mut Self {
        // Safe because Celsius is repr(transparent) over f64
        miniextendr_api::ffi::REAL(sexp) as *mut Self
    }
}
```

### What You Get Automatically

With just that impl, these all work:

```rust
// Scalar
fn get_temp() -> Celsius { ... }
fn set_temp(t: Celsius) { ... }

// Vectors
fn get_temps() -> Vec<Celsius> { ... }
fn process_temps(temps: &[Celsius]) { ... }

// Collections
fn temp_map() -> HashMap<String, Celsius> { ... }
fn temp_deque() -> VecDeque<Celsius> { ... }

// Optional integrations (with features enabled)
fn temp_tinyvec() -> TinyVec<[Celsius; 8]> { ... }
fn temp_nalgebra() -> DVector<Celsius> { ... }
fn temp_ndarray() -> Array1<Celsius> { ... }

// All Option<> variants
fn maybe_temp() -> Option<Celsius> { ... }
fn maybe_temps() -> Option<Vec<Celsius>> { ... }
```

### Requirements for RNativeType

Your type must:

1. **Be `#[repr(transparent)]`** over `i32`, `f64`, `u8`, or `Rcomplex`
2. **Implement `Copy`** (required by the trait bound)
3. **Be `'static`** (no borrowed data)

### Memory Layout Correspondence

| Rust Type | R Type | SEXPTYPE |
|-----------|--------|----------|
| `i32` | integer | `INTSXP` |
| `f64` | numeric | `REALSXP` |
| `u8` | raw | `RAWSXP` |
| `Rcomplex` | complex | `CPLXSXP` |

**Cannot be RNativeType**: `i8`, `i16`, `f32`, `i64`, `String` - no matching R storage type.

---

## Option 2: Direct TryFromSexp/IntoR Implementation

For types that don't match R's memory layout, implement the conversion traits directly.

### Example: Custom String Type

```rust
use miniextendr_api::ffi::SEXP;
use miniextendr_api::from_r::{SexpError, TryFromSexp};
use miniextendr_api::into_r::IntoR;

pub struct Username(String);

impl TryFromSexp for Username {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let s: String = TryFromSexp::try_from_sexp(sexp)?;
        if s.is_empty() {
            return Err(SexpError::InvalidValue("username cannot be empty".into()));
        }
        Ok(Username(s))
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        // Same logic - no unchecked fast path for this type
        Self::try_from_sexp(sexp)
    }
}

impl IntoR for Username {
    fn into_sexp(self) -> SEXP {
        self.0.into_sexp()
    }

    unsafe fn into_sexp_unchecked(self) -> SEXP {
        unsafe { self.0.into_sexp_unchecked() }
    }
}
```

### When to Use Direct Implementation

- Type requires validation (like `Username` above)
- Type stores borrowed data
- Conversion involves complex transformation
- Type maps to R list or other complex structure

---

## Adding Coercion Support

If your type can be losslessly converted to/from R's numeric types, implement the marker traits:

```rust
use miniextendr_api::markers::{WidensToF64, WidensToI32};
use miniextendr_api::coerce::Coerce;

// If Celsius can be losslessly widened to f64
impl WidensToF64 for Celsius {}

// Now this works:
impl From<Celsius> for f64 {
    fn from(c: Celsius) -> f64 { c.0 }
}

// And you get automatic coercion:
// Vec<Celsius>.coerce() -> Vec<f64>
```

### Available Marker Traits

| Trait | Meaning | Use When |
|-------|---------|----------|
| `WidensToI32` | Losslessly converts to `i32` | 8/16-bit signed integers |
| `WidensToF64` | Losslessly converts to `f64` | Any numeric that fits in f64 |

---

## Working with ExternalPtr

For complex types that shouldn't be copied to R, use `ExternalPtr`:

```rust
use miniextendr_api::externalptr::{ExternalPtr, TypedExternal};

pub struct LargeDataset {
    data: Vec<f64>,
    metadata: HashMap<String, String>,
}

// Implement TypedExternal for type safety
impl TypedExternal for LargeDataset {
    const TYPE_NAME: &'static str = "LargeDataset";
    const TYPE_NAME_CSTR: &'static [u8] = b"LargeDataset\0";
    const TYPE_ID_CSTR: &'static [u8] = b"mypackage::LargeDataset\0";
}

// Now you can pass it by reference:
#[miniextendr]
fn create_dataset() -> ExternalPtr<LargeDataset> {
    ExternalPtr::new(LargeDataset { ... })
}

#[miniextendr]
fn process_dataset(data: &LargeDataset) -> f64 {
    data.data.iter().sum()
}
```

### When to Use ExternalPtr

- Large data structures (avoid copying)
- Mutable state between R calls
- Types that don't have R equivalents
- Opaque handles to Rust resources

---

## Complete Example: Custom Numeric Type

```rust
use miniextendr_api::ffi::{RNativeType, SEXP, SEXPTYPE};
use miniextendr_api::markers::WidensToF64;

/// Probability value in [0, 1]
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Probability(f64);

impl Probability {
    pub fn new(value: f64) -> Option<Self> {
        if (0.0..=1.0).contains(&value) {
            Some(Self(value))
        } else {
            None
        }
    }

    pub fn value(self) -> f64 {
        self.0
    }
}

// Enable automatic conversions for all containers
impl RNativeType for Probability {
    const SEXP_TYPE: SEXPTYPE = SEXPTYPE::REALSXP;

    unsafe fn dataptr_mut(sexp: SEXP) -> *mut Self {
        miniextendr_api::ffi::REAL(sexp) as *mut Self
    }
}

// Enable coercion to f64
impl WidensToF64 for Probability {}

impl From<Probability> for f64 {
    fn from(p: Probability) -> f64 {
        p.0
    }
}

// Now all these work:
// Vec<Probability>, &[Probability], DVector<Probability>, etc.
```

---

## Checklist for New Types

1. **Choose your approach**:
   - `#[repr(transparent)]` newtype over primitive? → `RNativeType`
   - Complex type or needs validation? → Direct `TryFromSexp`/`IntoR`
   - Large/mutable? → `ExternalPtr`

2. **Implement required traits**:
   - [ ] `RNativeType` OR `TryFromSexp` + `IntoR`
   - [ ] `Copy` (if using `RNativeType`)
   - [ ] `TypedExternal` (if using `ExternalPtr`)

3. **Optional enhancements**:
   - [ ] `WidensToI32`/`WidensToF64` for coercion
   - [ ] `Ord` for `BinaryHeap` support
   - [ ] `Hash` for `HashSet`/`HashMap` key support

4. **Test your type**:
   - [ ] Scalar round-trip: Rust → R → Rust
   - [ ] Vector round-trip: `Vec<T>` both directions
   - [ ] Option handling: `None` ↔ `NULL` or `NA`
   - [ ] Edge cases: empty vectors, single elements
