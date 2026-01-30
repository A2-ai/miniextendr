# Trait Pattern Review: Marker Types vs Macro-Generated Impls

This document reviews how traits are used across miniextendr-api to guide future implementations.

## Executive Summary

The codebase uses **three main patterns**:

1. **Blanket impls with trait bounds** - For `IntoR` on `Vec<T>`, `&[T]`, `Array1<T>` where `T: RNativeType`
2. **Macro-generated impls** - For `TryFromSexp` and when you need per-type control
3. **Generic helper functions + macro impls** - Combines code reuse with explicit type coverage

**Key insight**: `IntoR` uses blanket impls **for core containers** (Vec, slice, HashMap); optional integrations use explicit/macro impls. `TryFromSexp` always uses macros.

**Tinyvec status**: Uses pattern #2 (macro-only). Pattern #3 would add generic helpers.

---

## Pattern 1: Blanket Implementations with Trait Bounds

Used when you want a single impl to cover all types satisfying a trait bound.

### Core Example: `IntoR` for `Vec<T>` and `&[T]`

```rust
// into_r.rs
impl<T> IntoR for Vec<T>
where
    T: crate::ffi::RNativeType,
{
    fn into_sexp(self) -> SEXP {
        unsafe { vec_to_sexp(&self) }
    }
}

impl<T> IntoR for &[T]
where
    T: crate::ffi::RNativeType,
{
    fn into_sexp(self) -> SEXP {
        unsafe { vec_to_sexp(self) }
    }
}
```

### Key Trait: `RNativeType`

```rust
// ffi.rs
pub trait RNativeType: Sized + Copy + 'static {
    const SEXP_TYPE: SEXPTYPE;
    unsafe fn dataptr_mut(sexp: SEXP) -> *mut Self;
}
```

Implemented for: `i32`, `f64`, `u8`, `RLogical`, `Rcomplex`

### Where Blanket Impls Work

| Type | IntoR | TryFromSexp | Notes |
|------|-------|-------------|-------|
| `Vec<T: RNativeType>` | ✅ Blanket | ❌ Macro | TryFromSexp uses `impl_vec_try_from_sexp_native!` |
| `&[T: RNativeType]` | ✅ Blanket | ❌ Macro | TryFromSexp uses `impl_ref_conversions_for!` |
| `Array1<T: RNativeType>` | ✅ Generic per-dim | ❌ Macro | IntoR is generic but explicit per dimension type |

**Why this asymmetry?** `IntoR` has a simple, uniform implementation (delegate to slice). `TryFromSexp` needs more control over error handling and type checking per element type.

**Note on ndarray**: While ndarray IntoR impls are generic over `T: RNativeType`, they're written explicitly for each dimension variant (Array0-6, ArrayD) because each has different conversion semantics (Array0 → scalar, Array1 → vector, Array2+ → matrix with dims). This is not a violation of the pattern—it's intentional design.

---

## Pattern 2: Macro-Generated Implementations

Used for `TryFromSexp` and when you need explicit per-type impls.

### Example: TryFromSexp for Slices

```rust
// from_r.rs
macro_rules! impl_ref_conversions_for {
    ($t:ty) => {
        impl TryFromSexp for &'static [$t] { ... }
        impl TryFromSexp for &'static mut [$t] { ... }
        impl TryFromSexp for Option<&'static [$t]> { ... }
        // etc.
    };
}

impl_ref_conversions_for!(i32);
impl_ref_conversions_for!(f64);
impl_ref_conversions_for!(u8);
impl_ref_conversions_for!(RLogical);
impl_ref_conversions_for!(crate::ffi::Rcomplex);
```

### Example: ndarray TryFromSexp

```rust
// ndarray_impl.rs
macro_rules! impl_array_try_from_sexp_native {
    ($t:ty) => {
        impl TryFromSexp for Array0<$t> { ... }
        impl TryFromSexp for Array1<$t> { ... }
        impl TryFromSexp for Array2<$t> { ... }
        // etc.
    };
}

impl_array_try_from_sexp_native!(i32);
impl_array_try_from_sexp_native!(f64);
impl_array_try_from_sexp_native!(u8);
impl_array_try_from_sexp_native!(RLogical);
impl_array_try_from_sexp_native!(Rcomplex);
```

### When to Use Macros

1. **Explicit type coverage** - Only support types you've tested
2. **Per-type logic** - Different coercion paths (`i8` from `i32`, `f32` from `f64`)
3. **Avoiding overlap** - When a blanket impl would conflict with existing explicit impls
4. **Convention** - `TryFromSexp` consistently uses macros throughout the codebase

---

## Pattern 3: Generic Helper Functions + Macro Impls

Combines code reuse (generic helpers) with explicit type coverage (macro impls).

### Example: ndarray

```rust
// Generic helper (internal) - code reuse
fn dmatrix_from_sexp<T: RNativeType>(sexp: SEXP) -> Result<DMatrix<T>, SexpError> {
    let slice: &[T] = unsafe { sexp.as_slice() };
    let (nrow, ncol) = get_matrix_dims(sexp)?;
    Ok(DMatrix::from_column_slice(nrow, ncol, slice))
}

// Macro generates explicit impls that call the generic helper
macro_rules! impl_nalgebra_try_from_sexp {
    ($t:ty) => {
        impl TryFromSexp for DMatrix<$t> {
            type Error = SexpError;
            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                dmatrix_from_sexp::<$t>(sexp)
            }
        }
    };
}

// But IntoR uses blanket impl (no overlap concerns going this direction)
impl<T: RNativeType + Clone> IntoR for DMatrix<T> {
    fn into_sexp(self) -> SEXP {
        // ... delegates to slice
    }
}
```

**Why this pattern?** You get:
- Code reuse via generic helpers
- Explicit type coverage via macro invocations
- No overlap conflicts (macros don't overlap with each other)

---

## Current tinyvec Implementation

Uses **pattern #2** (macro-only, no generic helpers):

```rust
macro_rules! impl_tinyvec_native {
    ($t:ty) => {
        impl<const N: usize> TryFromSexp for TinyVec<[$t; N]> { ... }
        impl<const N: usize> IntoR for TinyVec<[$t; N]> { ... }
        impl<const N: usize> TryFromSexp for ArrayVec<[$t; N]> { ... }
        impl<const N: usize> IntoR for ArrayVec<[$t; N]> { ... }
    };
}

impl_tinyvec_native!(i32);
impl_tinyvec_native!(f64);
impl_tinyvec_native!(u8);
impl_tinyvec_native!(RLogical);
```

### Why Not Blanket IntoR?

A blanket impl like this:
```rust
impl<A> IntoR for TinyVec<A>
where
    A: Array,
    A::Item: RNativeType,
{ ... }
```

**Would conflict with the existing macro-generated impls** (`impl IntoR for TinyVec<[i32; N]>`, etc.). You can't have both a blanket impl and explicit impls for overlapping types.

**To use blanket impls**, you'd need to remove the macro impls first. This is a design choice, not a compiler limitation.

### Why No Blanket Option<T> IntoR?

You might expect a blanket impl like:
```rust
impl<T: IntoR> IntoR for Option<T> {
    fn into_sexp(self) -> SEXP {
        match self {
            Some(v) => v.into_sexp(),
            None => unsafe { R_NilValue },
        }
    }
}
```

**This doesn't exist** because it would conflict with specialized implementations:
- `Option<i32>`, `Option<f64>` use R's NA values, not NULL
- `Option<&T>` has different lifetime semantics

**Pattern for optional integrations**: Every custom type must explicitly implement both:
- `impl IntoR for MyType`
- `impl IntoR for Option<MyType>` (using NULL for None)

Copy-paste template:
```rust
impl IntoR for Option<MyType> {
    #[inline]
    fn into_sexp(self) -> SEXP {
        match self {
            Some(v) => v.into_sexp(),
            None => unsafe { crate::ffi::R_NilValue },
        }
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> SEXP {
        match self {
            Some(v) => unsafe { v.into_sexp_unchecked() },
            None => unsafe { crate::ffi::R_NilValue },
        }
    }
}
```

### Possible Enhancement

Could upgrade to pattern #3 by adding generic helpers:

```rust
// Generic helper
fn tinyvec_from_sexp<T: RNativeType, const N: usize>(sexp: SEXP) -> Result<TinyVec<[T; N]>, SexpTypeError>
where
    [T; N]: tinyvec::Array<Item = T>,
{
    let slice: &[T] = TryFromSexp::try_from_sexp(sexp)?;
    let mut tv = TinyVec::new();
    tv.extend_from_slice(slice);
    Ok(tv)
}

// Macro calls the helper
macro_rules! impl_tinyvec_native {
    ($t:ty) => {
        impl<const N: usize> TryFromSexp for TinyVec<[$t; N]> { ... }
            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                tinyvec_from_sexp::<$t, N>(sexp)
            }
        }
    };
}
```

This would reduce code duplication while keeping explicit type coverage.

---

## Marker Traits

### What Marker Traits Actually Do

Marker traits in `markers.rs` are **informational** - they identify types with certain capabilities:

```rust
pub trait IsRNativeType: crate::ffi::RNativeType {}
impl<T: crate::ffi::RNativeType> IsRNativeType for T {}  // Blanket impl
```

**Important**: Since `IsRNativeType` has a blanket impl from `RNativeType`, it does NOT distinguish "derived" vs "manual" impls. Any type implementing `RNativeType` automatically gets `IsRNativeType`.

### Marker Trait Categories

| Trait | Purpose | Blanket From |
|-------|---------|--------------|
| `IsRNativeType` | R native element types | `RNativeType` |
| `IsAltrepIntegerData` | ALTREP integer data | `AltIntegerData` |
| `IsAltrepRealData` | ALTREP real data | `AltRealData` |
| `IsAltrepLogicalData` | ALTREP logical data | `AltLogicalData` |
| `IsAltrepRawData` | ALTREP raw data | `AltRawData` |
| `IsAltrepStringData` | ALTREP string data | `AltStringData` |
| `IsAltrepComplexData` | ALTREP complex data | `AltComplexData` |
| `IsAltrepListData` | ALTREP list data | `AltListData` |
| `PrefersList` | Types preferring list conversion | `IsIntoList` |
| `PrefersExternalPtr` | Types preferring ExternalPtr | `IntoExternalPtr` |
| `PrefersDataFrame` | Types preferring DataFrame | (explicit impls) |
| `PrefersRNativeType` | Types preferring native vector | `IsRNativeType` |

---

## Core Conversion Traits

| Trait | Purpose | Implementation Strategy |
|-------|---------|------------------------|
| `IntoR` | Rust → SEXP | Blanket impls for containers of `T: RNativeType` |
| `TryFromSexp` | SEXP → Rust | Macro-generated per concrete type |
| `IntoRAs<Target>` | Rust → SEXP with coercion | Explicit impls |
| `Coerce<R>` / `TryCoerce<R>` | Scalar coercion | Explicit impls |

---

## Optional Crate Adapter Traits

| Trait | Crate | Purpose |
|-------|-------|---------|
| `RNdArrayOps` | ndarray | Array operations (sum, mean, etc.) |
| `RNdSlice` | ndarray | Slice/view operations |

(Other adapter traits like `RVectorOps`, `RRegexOps`, etc. are mentioned in docs but may not exist yet)

---

## Recommendations

### Pattern Decision Tree

```
Want impl for Wrapper<T> where T: RNativeType?
│
├─ Is this for IntoR (Rust → R)?
│   ├─ Yes → Try blanket impl first
│   │         └─ If overlap with existing impls → Use macro
│   └─ No (TryFromSexp) → Use macro (codebase convention)
│
├─ Do you already have explicit impls for this type?
│   ├─ Yes → Blanket impl will conflict; stick with macros
│   └─ No → Blanket impl may work
│
├─ Do you need per-type logic (different error handling, coercion)?
│   ├─ Yes → Use macro
│   └─ No → Blanket impl may work
│
└─ Does the blanket impl compile without overlap errors?
    ├─ Yes → Use it
    └─ No → Use macro
```

### For Future Optionals

1. **IntoR**: Try blanket impl first (`impl<T: RNativeType> IntoR for Container<T>`)
2. **TryFromSexp**: Use macros (matches codebase convention)
3. **Code reuse**: Add generic helper functions, call from macro impls (pattern #3)
4. **If blanket conflicts**: You probably have existing explicit impls; remove them or stick with macros

---

## Files to Reference

| File | Contains |
|------|----------|
| `markers.rs` | Marker trait definitions and blanket impls |
| `ffi.rs` | `RNativeType` trait definition |
| `into_r.rs` | `IntoR` trait with blanket impls for Vec/slice |
| `from_r.rs` | `TryFromSexp` trait with macro-generated impls |
| `optionals/ndarray_impl.rs` | Example of pattern #3 (helpers + macros + blanket IntoR) |
| `optionals/tinyvec_impl.rs` | Example of pattern #2 (macro-only) |
| `optionals/nalgebra_impl.rs` | Another pattern #3 example |
