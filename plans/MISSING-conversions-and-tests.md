# Missing Conversions and Tests Audit

## Executive Summary

After comprehensive blanket impl migration, systematic audit reveals:
- **Std collections**: 6/12 have impls (50% coverage)
- **Std numerics**: 3/14 have RNativeType (21% coverage)
- **Optional crates**: 18/24 have conversions (75% coverage)
- **Test coverage**: Good for implemented features, gaps for new blanket impls

## 1. Missing Std Type Conversions

### Collections (Missing: 6)

| Type | IntoR | TryFromSexp | Why Missing / Should Add? |
|------|-------|-------------|---------------------------|
| ✅ `Vec<T>` | ✅ | ✅ | Done |
| ✅ `VecDeque<T>` | ✅ | ✅ | **NEW** - Just added! |
| ✅ `HashMap<K, V>` | ✅ | ✅ | Done |
| ✅ `BTreeMap<K, V>` | ✅ | ✅ | Done |
| ✅ `HashSet<T>` | ✅ | ✅ | Done |
| ✅ `BTreeSet<T>` | ✅ | ✅ | Done |
| ❌ `LinkedList<T>` | ❌ | ❌ | **Low priority** - O(n) access, rarely useful in R context |
| ❌ `BinaryHeap<T>` | ❌ | ❌ | **Medium priority** - Priority queue, could be useful |
| ❌ `Box<T>` | ❌ | ❌ | **Can't add** - Conflicts with `IntoExternalPtr` blanket impl |
| ❌ `Rc<T>` | ❌ | ❌ | **Low priority** - Could add but unclear R semantics |
| ❌ `Arc<T>` | ❌ | ❌ | **Low priority** - Could add but unclear R semantics |
| ❌ `Cow<'_, T>` | ❌ | ❌ | **Medium priority** - Could be useful for zero-copy |

**Recommendation**: Add `BinaryHeap<T>` and `Cow<'_, T>`. Skip the rest.

### Smart Pointers - Analysis

**Why `Rc<T>` and `Arc<T>` are problematic**:
- `IntoR` would need to clone the inner value (defeating the purpose)
- OR wrap in ExternalPtr (but then no type-safe access)
- R doesn't have reference counting semantics that match Rust

**Why `Box<T>` can't be added**:
- Conflicts with blanket `impl<T: IntoExternalPtr> IntoR for T`
- If downstream implements `IntoExternalPtr for Box<SomeType>`, overlap occurs
- Users can manually unbox: `(*my_box).into_sexp()`

### Numerics (Missing: 11/14)

| Type | RNativeType | Coerce → i32 | Coerce → f64 | Should Add? |
|------|-------------|--------------|--------------|-------------|
| ❌ `i8` | ❌ | ✅ (via marker) | ✅ (via marker) | **HIGH PRIORITY** |
| ❌ `i16` | ❌ | ✅ (via marker) | ✅ (via marker) | **HIGH PRIORITY** |
| ✅ `i32` | ✅ | ✅ (identity) | ✅ (via marker) | Done |
| ❌ `i64` | ❌ | ❌ | ❌ | **MEDIUM PRIORITY** |
| ❌ `i128` | ❌ | ❌ | ❌ | **LOW** - No R equivalent |
| ❌ `isize` | ❌ | ❌ | ❌ | **LOW** - Platform-specific |
| ✅ `u8` | ✅ | ✅ (via marker) | ✅ (via marker) | Done |
| ❌ `u16` | ❌ | ✅ (via marker) | ✅ (via marker) | **MEDIUM** |
| ❌ `u32` | ❌ | ❌ | ✅ (via marker) | **MEDIUM** |
| ❌ `u64` | ❌ | ❌ | ❌ | **LOW** |
| ❌ `u128` | ❌ | ❌ | ❌ | **LOW** - No R equivalent |
| ❌ `usize` | ❌ | ❌ | ❌ | **LOW** - Platform-specific |
| ❌ `f32` | ❌ | ❌ | ✅ (via marker) | **MEDIUM** |
| ✅ `f64` | ✅ | ❌ | ✅ (identity) | Done |

**Key insight**: `i8`, `i16`, `f32` work via **coercion** but don't have **direct** RNativeType impls.

**Recommendation**:
- Add `RNativeType` for `i8`, `i16`, `f32` (high value)
- Add `RNativeType` for `i64` (medium value)
- Skip i128/u128 (no R equivalent)

**Impact of adding**: Each new RNativeType automatically gets:
- Vec, VecDeque, HashMap, HashSet, etc.
- TinyVec, ArrayVec
- ndarray arrays (Array0-6, ArrayD)
- nalgebra vectors/matrices
- ~100+ conversions per type

## 2. Missing Optional Crate Conversions

### Crates with 0 Conversions (Should Have Some)

| Crate | Public Types | Missing Conversions | Why/Should Add? |
|-------|--------------|---------------------|-----------------|
| `bytes` | Bytes, BytesMut, Buf, BufMut | Bytes ↔ Raw vector | **HIGH** - Bytes ↔ `Vec<u8>` |
| `num-traits` | (traits only) | N/A | **Correct** - No conversions needed |
| `rand` | (RNG traits) | N/A | **Correct** - Adapter traits only |
| `sha2` | Sha256, Sha512, digest types | Hash → Raw vector | **MEDIUM** - `[u8; N]` ↔ raw |
| `rayon` | (parallel iter) | N/A | **Correct** - Bridge code only |

### Bytes - HIGH PRIORITY

**Missing**:
```rust
// Should add:
impl IntoR for Bytes {
    fn into_sexp(self) -> SEXP {
        self.to_vec().into_sexp()  // Convert to RAWSXP
    }
}

impl TryFromSexp for Bytes {
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let vec: Vec<u8> = TryFromSexp::try_from_sexp(sexp)?;
        Ok(Bytes::from(vec))
    }
}

// Similar for BytesMut
```

### Sha2 - MEDIUM PRIORITY

**Missing**:
```rust
// Should add for fixed-size hash outputs:
impl IntoR for [u8; 32] {  // SHA256 output
    fn into_sexp(self) -> SEXP {
        self.as_slice().into_sexp()  // Convert to RAWSXP
    }
}

// Or more generally:
impl<const N: usize> IntoR for [u8; N] {
    fn into_sexp(self) -> SEXP {
        self.as_slice().into_sexp()
    }
}
```

## 3. Missing ndarray/nalgebra Types

### Ndarray Coverage

| Type | IntoR | TryFromSexp | Notes |
|------|-------|-------------|-------|
| ✅ `Array0-6<T>` | ✅ Blanket | ✅ Blanket | Done |
| ✅ `ArrayD<T>` | ✅ Blanket | ✅ Blanket | Done |
| ✅ `ArcArray1<T>` | ✅ | ✅ | Shared ownership |
| ✅ `ArcArray2<T>` | ✅ | ✅ | Shared ownership |
| ❌ `ArrayView0-6<T>` | ✅ | ❌ | **Views are read-only from SEXP** |
| ❌ `ArrayViewMut<T>` | ❌ | ❌ | **Unclear semantics** |

**Views analysis**: ArrayView types already have IntoR but not TryFromSexp because:
- Views are zero-copy borrows
- TryFromSexp would need to return a view into SEXP memory
- This is what `from_r_slice()` does - **it's already there via helper functions!**

**Recommendation**: ArrayView conversions are already optimal via helper functions.

### Nalgebra Coverage

| Type | IntoR | TryFromSexp | Notes |
|------|-------|-------------|-------|
| ✅ `DVector<T>` | ✅ Blanket | ✅ Blanket | Done |
| ✅ `DMatrix<T>` | ✅ Blanket | ✅ Blanket | Done |
| ❌ `SVector<T, const N: usize>` | ❌ | ❌ | **HIGH** - Fixed-size vectors |
| ❌ `SMatrix<T, const R: usize, const C: usize>` | ❌ | ❌ | **MEDIUM** - Fixed-size matrices |
| ❌ `Vector1-6<T>` | ❌ | ❌ | **LOW** - Type aliases for SVector |
| ❌ `Matrix1-6<T>` | ❌ | ❌ | **LOW** - Type aliases for SMatrix |

**SVector/SMatrix are important** - they're stack-allocated, zero-cost abstractions.

**Recommendation**: Add blanket impls for SVector and SMatrix.

## 4. Missing Tests

### New Blanket Impls Lacking Tests

**Critical (Just Added, Untested)**:
1. ❌ **VecDeque conversions** - No integration tests
   - Need: R → VecDeque → R roundtrip
   - Need: VecDeque with all RNativeType elements

2. ❌ **Arbitrary lifetime slices** - No tests for `&[T]` vs `&'static [T]`
   - Need: Verify non-static lifetime works

3. ❌ **TinyVec blanket impl** - Only unit tests, no integration tests
   - Need: R → TinyVec → R roundtrip
   - Need: Capacity overflow tests for ArrayVec

4. ❌ **Nalgebra blanket impl** - No tests for non-concrete types
   - Existing tests only use `DVector<f64>`, not generic `T: RNativeType`

5. ❌ **Ndarray blanket impl** - No tests for non-concrete types
   - Existing tests only use specific types (i32, f64)

6. ❌ **Coerce with marker traits** - No tests for WidensToI32/WidensToF64
   - Need: Verify blanket impl works for all marked types

7. ❌ **TinyVec/ArrayVec element-wise coercion** - Just added, no tests
   - Need: `TinyVec<[i8; N]>.coerce() → TinyVec<[i32; N]>`

### Existing Features with Weak Coverage

8. ❌ **Bytes** (if we add conversions) - No tests
9. ❌ **Sha2** (if we add conversions) - No tests
10. ❌ **SVector/SMatrix** (if we add) - No tests

### Good Coverage (Existing)

✅ **Coerce.rs unit tests** - 41 tests for scalar coercion
✅ **IndexMap** - Has dedicated test file
✅ **Uuid** - Has dedicated test file
✅ **Time** - Has dedicated test file
✅ **Regex** - Has dedicated test file
✅ **BigInt** - Has dedicated test file
✅ **OrderedFloat** - Has dedicated test file
✅ **Rust Decimal** - Has dedicated test file

## 5. Prioritized Action Items

### HIGH PRIORITY - Add These Conversions

1. **Bytes ↔ Raw vector**
   ```rust
   impl IntoR for Bytes
   impl IntoR for BytesMut
   impl TryFromSexp for Bytes
   impl TryFromSexp for BytesMut
   ```
   **Why**: Very common use case, bytes are fundamental

2. **Fixed-size arrays → Raw vector**
   ```rust
   impl<const N: usize> IntoR for [u8; N]
   impl<const N: usize> TryFromSexp for [u8; N]
   ```
   **Why**: Enables SHA hashes, fixed-size byte patterns

3. **RNativeType for i8, i16, f32**
   ```rust
   impl RNativeType for i8
   impl RNativeType for i16
   impl RNativeType for f32
   ```
   **Why**: Currently only work via coercion, would unlock all containers

4. **nalgebra SVector/SMatrix**
   ```rust
   impl<T: RNativeType + Scalar, const N: usize> TryFromSexp for SVector<T, N>
   impl<T: RNativeType + Scalar, const R: usize, const C: usize> TryFromSexp for SMatrix<T, R, C>
   ```
   **Why**: Stack-allocated, zero-cost, commonly used

### MEDIUM PRIORITY - Consider Adding

5. **BinaryHeap<T>**
   ```rust
   impl<T: RNativeType + Ord> IntoR for BinaryHeap<T>
   impl<T: RNativeType + Ord> TryFromSexp for BinaryHeap<T>
   ```
   **Why**: Priority queue could be useful for algorithms

6. **Cow<'_, T>**
   ```rust
   impl<T: IntoR + Clone> IntoR for Cow<'_, T>
   impl<T: TryFromSexp> TryFromSexp for Cow<'static, T>
   ```
   **Why**: Zero-copy patterns

7. **RNativeType for i64**
   ```rust
   impl RNativeType for i64  // Maps to REALSXP (loses precision > 2^53)
   ```
   **Why**: Timestamps, large integers (with caveat about precision)

### LOW PRIORITY - Skip

8. **LinkedList<T>** - O(n) access, not useful for R vectors
9. **i128/u128** - No R equivalent
10. **isize/usize** - Platform-dependent, unclear R mapping
11. **Rc/Arc smart pointers** - Unclear R semantics

## 6. Missing Tests - Priority Order

### CRITICAL - Test New Blanket Impls

1. **VecDeque roundtrip tests** (miniextendr-api/tests/vecdeque.rs)
   ```rust
   #[test]
   fn test_vecdeque_i32_roundtrip() { ... }
   #[test]
   fn test_vecdeque_f64_roundtrip() { ... }
   #[test]
   fn test_vecdeque_to_r_preserves_order() { ... }
   ```

2. **Slice arbitrary lifetime tests** (miniextendr-api/tests/from_r.rs)
   ```rust
   #[test]
   fn test_slice_non_static_lifetime() { ... }
   #[test]
   fn test_slice_mut_non_static() { ... }
   ```

3. **TinyVec blanket impl tests** (miniextendr-api/tests/tinyvec.rs - NEW FILE)
   ```rust
   #[test]
   fn test_tinyvec_i8_to_r() { ... }
   #[test]
   fn test_arrayvec_capacity_error() { ... }
   #[test]
   fn test_tinyvec_inline_vs_heap() { ... }
   ```

4. **Marker trait coercion tests** (miniextendr-api/tests/coerce.rs)
   ```rust
   #[test]
   fn test_widen_to_i32_marker_i8() { ... }
   #[test]
   fn test_widen_to_i32_marker_i16() { ... }
   #[test]
   fn test_widen_to_f64_marker_f32() { ... }
   #[test]
   fn test_tinyvec_element_wise_coerce() { ... }
   ```

5. **Nalgebra generic type tests** (miniextendr-api/tests/nalgebra.rs - NEW FILE)
   ```rust
   #[test]
   fn test_dvector_i32() { ... }
   #[test]
   fn test_dmatrix_u8() { ... }
   #[test]
   fn test_dvector_generic_works_for_all_rnative() { ... }
   ```

6. **Ndarray generic type tests** (miniextendr-api/tests/ndarray.rs - NEW FILE)
   ```rust
   #[test]
   fn test_array1_i32() { ... }
   #[test]
   fn test_array2_u8() { ... }
   #[test]
   fn test_array_blanket_impl_coverage() { ... }
   ```

### HIGH PRIORITY - Test Planned Additions

7. **Bytes conversions** (if added)
8. **Fixed-size array conversions** (if added)
9. **SVector/SMatrix conversions** (if added)

## 7. Test File Structure Recommendations

### Create New Test Files

```
miniextendr-api/tests/
├── vecdeque.rs          # NEW - VecDeque conversions
├── tinyvec.rs           # NEW - TinyVec/ArrayVec with all RNativeType
├── nalgebra.rs          # NEW - Generic nalgebra tests
├── ndarray.rs           # NEW - Generic ndarray tests
├── bytes.rs             # NEW (if conversions added)
├── fixed_arrays.rs      # NEW (if [T; N] conversions added)
└── blanket_impls.rs     # NEW - Comprehensive blanket impl verification
```

### Extend Existing Test Files

```
miniextendr-api/tests/coerce.rs
├── Add: test_widen_to_i32_marker_coverage
├── Add: test_widen_to_f64_marker_coverage
├── Add: test_tinyvec_coerce_i8_to_i32
└── Add: test_vecdeque_coerce_element_wise

miniextendr-api/tests/from_r.rs
├── Add: test_slice_arbitrary_lifetime
├── Add: test_slice_mut_arbitrary_lifetime
└── Add: test_option_slice_arbitrary_lifetime
```

## 8. Documentation Gaps

### Missing Documentation

1. **No guide for adding custom RNativeType**
   - Should document: "How to extend with your own numeric type"

2. **No blanket impl pattern guide**
   - Should document: "How blanket impls work and why they're better"

3. **No marker trait guide**
   - Should document: "How to use marker traits for conditional behavior"

4. **trait-patterns-review.md is outdated**
   - Still says "TryFromSexp uses macros" (now false)
   - Doesn't explain blanket impl as primary pattern
   - Needs complete rewrite

## 9. Recommended Implementation Order

### Phase 1: Fix Critical Test Gaps (1-2 hours)
1. Add VecDeque integration tests
2. Add TinyVec integration tests
3. Add marker trait coercion tests
4. Verify all new blanket impls have test coverage

### Phase 2: High-Priority Conversions (2-3 hours)
1. Add Bytes/BytesMut conversions
2. Add fixed-size array `[T; N]` conversions
3. Add RNativeType for i8, i16, f32
4. Add nalgebra SVector/SMatrix

### Phase 3: Medium-Priority Additions (1-2 hours)
1. Add BinaryHeap<T>
2. Add Cow<'_, T>
3. Add i64 RNativeType (with precision loss warning)

### Phase 4: Documentation (2 hours)
1. Update trait-patterns-review.md
2. Create "Extending miniextendr" guide
3. Create "Blanket Impl Pattern" guide
4. Create marker trait guide

## 10. Summary: What's Missing

### Missing Conversions (High Value)
- ✅ i8, i16, f32 as RNativeType (unlocks ~300+ conversions each)
- ❌ Bytes/BytesMut ↔ Raw vector
- ❌ [u8; N] ↔ Raw vector (fixed-size arrays)
- ❌ nalgebra SVector/SMatrix
- ❌ BinaryHeap<T>

### Missing Tests (Critical)
- ❌ VecDeque integration tests
- ❌ TinyVec integration tests
- ❌ Marker trait coverage tests
- ❌ Generic type tests for nalgebra/ndarray blanket impls
- ❌ Slice arbitrary lifetime tests

### Missing Documentation
- ❌ Blanket impl pattern guide
- ❌ Extending miniextendr guide
- ❌ Updated trait patterns review

**Total estimated effort**: 6-9 hours to complete all high/medium priority items
