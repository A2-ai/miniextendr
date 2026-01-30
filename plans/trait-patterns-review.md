# Trait Pattern Review: Blanket Impls with Marker Traits

This document reviews how traits are used across miniextendr-api after the blanket impl migration (commit e91190e).

## Executive Summary

The codebase now uses **blanket implementations as the primary pattern**:

1. **Blanket impls with `RNativeType` bound** - For containers: `Vec<T>`, `&[T]`, `TinyVec<[T; N]>`, `DVector<T>`, `SMatrix<T, R, C>`, etc.
2. **Marker traits for behavior control** - `WidensToI32`, `WidensToF64` for coercion paths
3. **Macros for non-parametric types only** - Scalar coercions, specialized conversions

**Key insight**: Any type implementing `RNativeType` automatically gets ~150+ conversions across all containers (Vec, TinyVec, ndarray, nalgebra, HashMap, etc.) without modification.

**External extensibility**: External crates can implement `RNativeType` for custom types and immediately get full ecosystem integration.

---

## Pattern 1: Blanket Implementations (Primary Pattern)

Used for all container types. A single impl covers all `T: RNativeType`.

### Core Example: Slices and Vectors

```rust
// from_r.rs - works for ANY T: RNativeType
impl<T> TryFromSexp for &[T]
where
    T: RNativeType + Copy,
{
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        // Type checking via T::SEXP_TYPE
        // Direct pointer cast - no per-type code needed
    }
}

// into_r.rs
impl<T: RNativeType> IntoR for Vec<T> {
    fn into_sexp(self) -> SEXP {
        unsafe { vec_to_sexp(&self) }
    }
}
```

### Container Coverage via Blanket Impls

| Container | TryFromSexp | IntoR | Notes |
|-----------|-------------|-------|-------|
| `Vec<T>` | ✅ Blanket | ✅ Blanket | Core container |
| `&[T]`, `&mut [T]` | ✅ Blanket | ✅ Blanket | Arbitrary lifetimes |
| `[T; N]` | ✅ Blanket | ✅ Blanket | Fixed-size arrays |
| `VecDeque<T>` | ✅ Blanket | ✅ Blanket | Double-ended queue |
| `BinaryHeap<T>` | ✅ Blanket | ✅ Blanket | Requires `T: Ord` |
| `Cow<'_, [T]>` | ✅ Blanket | ✅ Blanket | Zero-copy wrapper |
| `HashMap<String, T>` | ✅ Blanket | ✅ Blanket | Named list |
| `TinyVec<[T; N]>` | ✅ Blanket | ✅ Blanket | Small-vec optimization |
| `ArrayVec<[T; N]>` | ✅ Blanket | ✅ Blanket | Stack-only vec |
| `DVector<T>` | ✅ Blanket | ✅ Blanket | Dynamic nalgebra vector |
| `DMatrix<T>` | ✅ Blanket | ✅ Blanket | Dynamic nalgebra matrix |
| `SVector<T, D>` | ✅ Blanket | ✅ Blanket | Static nalgebra vector |
| `SMatrix<T, R, C>` | ✅ Blanket | ✅ Blanket | Static nalgebra matrix |
| `Array0..6<T>` | ✅ Blanket | ✅ Blanket | ndarray types |
| `ArrayD<T>` | ✅ Blanket | ✅ Blanket | ndarray dynamic-dim |

### The Key Trait: `RNativeType`

```rust
// ffi.rs
pub trait RNativeType: Sized + Copy + 'static {
    /// The SEXPTYPE for vectors containing this element type.
    const SEXP_TYPE: SEXPTYPE;

    /// Get mutable pointer to vector data.
    unsafe fn dataptr_mut(sexp: SEXP) -> *mut Self;
}
```

**Implemented for**: `i32`, `f64`, `u8`, `RLogical`, `Rcomplex`

**Why these types?** They have direct memory layout correspondence with R's internal storage:
- `i32` ↔ `INTSXP` (R's integer)
- `f64` ↔ `REALSXP` (R's numeric)
- `u8` ↔ `RAWSXP` (R's raw)
- `RLogical` ↔ `LGLSXP` (R's logical, stored as i32)
- `Rcomplex` ↔ `CPLXSXP` (R's complex)

**Cannot be RNativeType**: `i8`, `i16`, `f32`, `i64` - memory layout doesn't match any R type.

---

## Pattern 2: Marker Traits for Behavior Control

Marker traits declare capabilities that enable blanket impls.

### Coercion Markers

```rust
// markers.rs
/// Marker: type can be losslessly widened to i32
pub trait WidensToI32: Into<i32> + Copy {}

impl WidensToI32 for i8 {}
impl WidensToI32 for i16 {}
impl WidensToI32 for u8 {}
impl WidensToI32 for u16 {}
// NOT i32 - identity coercion handled separately

/// Marker: type can be losslessly widened to f64
pub trait WidensToF64: Into<f64> + Copy {}

impl WidensToF64 for f32 {}
impl WidensToF64 for i8 {}
impl WidensToF64 for i16 {}
impl WidensToF64 for i32 {}
impl WidensToF64 for u8 {}
impl WidensToF64 for u16 {}
impl WidensToF64 for u32 {}
```

### Blanket Impls Using Markers

```rust
// coerce.rs
impl<T: WidensToI32> Coerce<i32> for T {
    fn coerce(self) -> i32 {
        self.into()
    }
}

impl<T: WidensToF64> Coerce<f64> for T {
    fn coerce(self) -> f64 {
        self.into()
    }
}
```

### Why Markers Instead of Direct Bounds?

**Without markers** (doesn't work):
```rust
// This would conflict with other impls
impl<T: Into<i32> + Copy> Coerce<i32> for T { ... }
```

**With markers** (works):
```rust
// Opt-in via marker trait - no conflicts
impl<T: WidensToI32> Coerce<i32> for T { ... }
```

Markers provide **explicit opt-in** without blanket overlap problems.

---

## Pattern 3: Macros (Limited Use)

Macros are now only used for:

1. **Scalar coercions** - Per-type conversion paths (i8→i32, f32→f64)
2. **Non-parametric types** - Types without a generic parameter (String, bool)
3. **Specialized behavior** - When different types need fundamentally different logic

### Example: Scalar TryFromSexp

```rust
// from_r.rs - scalar reads need per-type NA handling
macro_rules! impl_scalar_try_from_sexp {
    ($t:ty, $na_check:expr) => {
        impl TryFromSexp for $t {
            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                // Per-type NA checking and conversion
            }
        }
    };
}
```

---

## Adding New Types

### External Crate Adding `RNativeType`

```rust
// In your crate (no miniextendr modification needed!)
use miniextendr_api::ffi::{RNativeType, SEXP, SEXPTYPE};

pub struct Temperature(f64);

impl RNativeType for Temperature {
    const SEXP_TYPE: SEXPTYPE = SEXPTYPE::REALSXP;

    unsafe fn dataptr_mut(sexp: SEXP) -> *mut Self {
        miniextendr_api::ffi::REAL(sexp) as *mut Self
    }
}

// Now automatically works:
// - Vec<Temperature>
// - &[Temperature]
// - TinyVec<[Temperature; N]>
// - DVector<Temperature>
// - HashMap<String, Temperature>
// - All Option<> variants
// - ~150+ total conversions
```

### Adding Coercion Support

```rust
// Your type widens to f64
impl WidensToF64 for Temperature {}

// Now you get:
// - Vec<Temperature>.coerce() -> Vec<f64>
// - Temperature as function parameter with auto-coercion
```

---

## Design Principles

### 1. Prefer Blanket Impls

**Before (macro approach)**:
```rust
// Must modify library for each type
impl_conversions_for!(MyType);
impl_tinyvec_for!(MyType);
impl_nalgebra_for!(MyType);
// ...repeat for every container
```

**After (blanket approach)**:
```rust
// One impl unlocks everything
impl RNativeType for MyType { ... }
```

### 2. Use Markers for Opt-in Behavior

Instead of complex trait bounds, use marker traits:
```rust
// Clear intent, no overlap issues
impl<T: WidensToI32> Coerce<i32> for T { ... }
```

### 3. Macros Only When Necessary

Use macros only when:
- Types have fundamentally different behavior (NA handling)
- No generic parameter to abstract over
- Specialization is required

---

## Option<T> Handling

`Option<T>` **cannot** have a blanket `IntoR` impl because:
- `Option<i32>`, `Option<f64>` use R's NA values
- `Option<MyCustomType>` uses R's NULL

This is handled by explicit impls for each container type:
```rust
impl<T: RNativeType> IntoR for Option<Vec<T>> { ... }
impl<T, const N: usize> IntoR for Option<TinyVec<[T; N]>> { ... }
// etc.
```

---

## Summary

| Pattern | When to Use | Example |
|---------|-------------|---------|
| Blanket impl | Containers over `T: RNativeType` | `Vec<T>`, `DVector<T>` |
| Marker trait | Opt-in capabilities | `WidensToI32`, `WidensToF64` |
| Macro | Scalar types, specialized behavior | `TryFromSexp for i32` |

The blanket impl approach transforms miniextendr from a **library** (fixed set of supported types) to a **framework** (extensible via trait implementation).
