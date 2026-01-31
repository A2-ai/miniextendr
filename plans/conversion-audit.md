# Conversion Audit: Blanket Impl Coverage Analysis

## Status: In Progress

This document tracks the systematic conversion of all optional integrations to use blanket impls for maximum composability.

## ✅ Completed Conversions

### Core (from_r.rs)
- ✅ `&[T]` where `T: RNativeType` - **Blanket impl** (replaces macro)
- ✅ `&mut [T]` where `T: RNativeType` - **Blanket impl** (replaces macro)
- ✅ `Option<&[T]>` where `T: RNativeType` - **Blanket impl** (replaces macro)
- ✅ `Option<&mut [T]>` where `T: RNativeType` - **Blanket impl** (replaces macro)

### tinyvec_impl.rs
- ✅ `TinyVec<[T; N]>` where `T: RNativeType` - **Blanket impl** (replaced macro + helpers)
- ✅ `ArrayVec<[T; N]>` where `T: RNativeType` - **Blanket impl** (replaced macro + helpers)
- ✅ `Option<TinyVec<[T; N]>>` - **Blanket impl**
- ✅ `Option<ArrayVec<[T; N]>>` - **Blanket impl**
- ✅ `IntoR` for all above - **Blanket impl**

**Result**: 8 blanket impls covering infinite types, down from ~300 lines of macro code

### nalgebra_impl.rs
- ✅ `DVector<T>` where `T: RNativeType + Scalar` - **Blanket impl** (replaced macro + helpers)
- ✅ `DMatrix<T>` where `T: RNativeType + Scalar` - **Blanket impl** (replaced macro + helpers)
- ✅ `IntoR` for both - Already blanket impls

**Result**: 2 blanket impls covering infinite types

## 🚧 In Progress

### ndarray_impl.rs
**Status**: Partially converted, needs completion

Dimension types need individual impls due to different conversion semantics:
- 🚧 `Array0<T>` - Scalar (length = 1 check)
- 🚧 `Array1<T>` - 1D vector
- 🚧 `Array2<T>` - 2D matrix (dimension extraction)
- 🚧 `Array3<T>` - 3D array
- 🚧 `Array4<T>` - 4D array
- 🚧 `Array5<T>` - 5D array
- 🚧 `Array6<T>` - 6D array
- 🚧 `ArrayD<T>` - Dynamic dimensions

**Action**: Convert macro + helper pattern to 8 blanket impls (one per dimension type)

**Current**: `macro_rules! impl_array_try_from_sexp_native` + 5 invocations
**Target**: 8 blanket impls: `impl<T: RNativeType + Copy> TryFromSexp for ArrayN<T>`

### IntoR Impls Already Blanket
- ✅ `Array0<T>` through `ArrayD<T>` - Already have blanket `IntoR` impls

## 📋 Systematic Audit Needed

### Standard Library Types

**Numeric Types**:
- ✅ `i32`, `f64`, `u8` - Core RNativeType
- ⚠️  `i8`, `i16`, `i64`, `i128` - Coercion only
- ⚠️  `u16`, `u32`, `u64`, `u128` - Coercion only
- ⚠️  `f32` - Coercion only
- ⚠️  `isize`, `usize` - Coercion only

**Collections** (should all work via blanket impls now):
- ✅ `Vec<T>` where `T: RNativeType` - Blanket impl in `into_r.rs`
- ✅ `&[T]` where `T: RNativeType` - **NEW** Blanket impl
- ✅ `HashMap<String, T>` - Already blanket impl
- ❓ `HashSet<T>` - Check if exists
- ❓ `BTreeMap<K, V>` - Check if exists
- ❓ `BTreeSet<T>` - Check if exists
- ❓ `LinkedList<T>` - Probably not needed
- ❓ `VecDeque<T>` - Check if needed

**Other std types**:
- ✅ `String` - Exists
- ✅ `bool` - Exists via RLogical
- ✅ `Option<T>` for various T - Partially (no blanket)
- ❓ `Result<T, E>` - Check if exists
- ❓ `Box<T>` - Check if exists
- ❓ `Rc<T>`, `Arc<T>` - Check if exists
- ❓ `Cow<'_, T>` - Check if needed

### Optional Crate Types

**ndarray**:
- 🚧 `Array0-6<T: RNativeType>` - Converting to blanket
- 🚧 `ArrayD<T: RNativeType>` - Converting to blanket
- ❓ `ArrayView<T>` - Check if exists
- ❓ `ArrayViewMut<T>` - Check if exists

**nalgebra**:
- ✅ `DVector<T: RNativeType + Scalar>` - Done
- ✅ `DMatrix<T: RNativeType + Scalar>` - Done
- ❓ `SVector<T, const N: usize>` - Static vectors
- ❓ `SMatrix<T, const R: usize, const C: usize>` - Static matrices
- ❓ Other nalgebra types?

**tinyvec**:
- ✅ `TinyVec<[T; N]>` - Done
- ✅ `ArrayVec<[T; N]>` - Done

**Other optional crates** (23 total):
Need to audit each for:
1. What public types exist?
2. Which have conversions?
3. Which should have conversions?
4. Can they use blanket impls?

Files to audit:
- url_impl.rs
- toml_impl.rs
- bytes_impl.rs
- indexmap_impl.rs
- rust_decimal_impl.rs
- time_impl.rs
- num_bigint_impl.rs
- regex_impl.rs
- either_impl.rs
- ordered_float_impl.rs
- aho_corasick_impl.rs
- bitflags_impl.rs
- num_complex_impl.rs
- rand_impl.rs
- bitvec_impl.rs
- tabled_impl.rs
- uuid_impl.rs
- num_traits_impl.rs
- serde_impl.rs
- sha2_impl.rs
- rayon_bridge.rs

## 🎯 Strategy Going Forward

### Phase 1: Complete Current Conversions
1. Finish ndarray blanket impls
2. Verify all tests pass

### Phase 2: Systematic Std Audit
Use Rust std documentation to check every std collection/container:
- Which ones make sense for R conversion?
- Do they already have IntoR/TryFromSexp?
- Can we add blanket impls?

### Phase 3: Optional Crate Audit
For each optional crate:
1. Read crate documentation
2. List all public types
3. Check which have conversions
4. Identify gaps
5. Implement blanket impls where possible

### Phase 4: Documentation Update
Update `trait-patterns-review.md` to reflect:
- New blanket impl approach
- Coverage matrix (what's implemented vs what exists)
- Guidelines for adding new types/crates

## Questions for User

1. **Std collections priority**: Which std types are most important?
   - HashSet/BTreeMap/BTreeSet?
   - Result<T, E> conversion strategy?
   - Smart pointers (Box/Rc/Arc)?

2. **Coercion types**: Should non-RNativeType numerics get their own impls?
   - Currently i8/i16/i64/f32 only work via coercion
   - Could add explicit impls for direct conversion

3. **nalgebra coverage**: Beyond DVector/DMatrix?
   - Static-size vectors/matrices?
   - Other nalgebra types?

4. **Scope**: Should we aim for 100% coverage of all public types?
   - Or focus on "commonly useful" subset?

## Benefits of Blanket Impl Approach

**Achieved so far**:
- ~500 lines of macro code → ~150 lines of blanket impls
- 5 concrete types → ∞ types (any T: RNativeType)
- External crates can add RNativeType impls and get everything for free
- True composability - traits actually compose now

**Next**: Apply this pattern everywhere consistently
