# Final Audit: Missing Conversions and Tests

## Quick Summary

**Missing High-Priority Conversions**: 5 items
**Missing Critical Tests**: 7 test files
**Estimated Effort**: 8-10 hours for high-priority items

---

## Part 1: Missing Conversions

### 🔴 HIGH PRIORITY

#### 1. i8, i16, f32 as RNativeType
**Current**: Work via coercion only
**Missing**: Direct RNativeType implementations

**Impact**: Each would unlock ~100+ automatic conversions:
```rust
// Add ONE impl:
impl RNativeType for i16 {
    const SEXP_TYPE: SEXPTYPE = SEXPTYPE::INTSXP;
    unsafe fn dataptr_mut(sexp: SEXP) -> *mut Self {
        INTEGER(sexp) as *mut i16
    }
}

// Automatically get ALL of these FREE:
// - Vec<i16>, VecDeque<i16>
// - TinyVec<[i16; N]>, ArrayVec<[i16; N]>
// - DVector<i16>, DMatrix<i16>
// - Array0-6<i16>, ArrayD<i16>
// - HashMap<String, i16>, HashSet<i16>
// - &[i16], &mut [i16], Option<&[i16]>
// - Plus all Option<> and Vec<Option<>> variants
```

**Caveat for f32**:
- R only has f64 (REALSXP)
- f32 → f64 widening on read
- f64 → f32 narrowing on write (precision loss)
- Still valuable for memory-efficient storage

**Effort**: 30 mins per type

#### 2. Bytes/BytesMut ↔ Raw Vector
**Current**: bytes_impl.rs only has adapter traits
**Missing**: Direct conversions to/from RAWSXP

```rust
impl IntoR for Bytes {
    fn into_sexp(self) -> SEXP {
        Vec::from(self.as_ref()).into_sexp()  // to RAWSXP
    }
}

impl TryFromSexp for Bytes {
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let vec: Vec<u8> = TryFromSexp::try_from_sexp(sexp)?;
        Ok(Bytes::from(vec))
    }
}

// Same for BytesMut
```

**Why**: bytes crate is fundamental for efficient byte handling
**Effort**: 1 hour

#### 3. Fixed-Size Arrays [T; N]
**Current**: Only works via .as_slice()
**Missing**: Direct IntoR for [u8; N], [i32; N], etc.

```rust
// Blanket impl for fixed-size arrays
impl<T: RNativeType, const N: usize> IntoR for [T; N] {
    fn into_sexp(self) -> SEXP {
        self.as_slice().into_sexp()
    }
}

impl<T: RNativeType + Copy, const N: usize> TryFromSexp for [T; N] {
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let slice: &[T] = TryFromSexp::try_from_sexp(sexp)?;
        if slice.len() != N {
            return Err(SexpLengthError { expected: N, actual: slice.len() }.into());
        }
        let mut arr = [T::default(); N];  // Requires Default
        arr.copy_from_slice(slice);
        Ok(arr)
    }
}
```

**Challenge**: Requires `T: Default` or unsafe `MaybeUninit`
**Use case**: SHA256 → [u8; 32], fixed-size patterns
**Effort**: 2 hours

#### 4. nalgebra SVector/SMatrix
**Current**: Only DVector/DMatrix
**Missing**: Static-size vectors and matrices

```rust
impl<T: RNativeType + Scalar + Copy, const N: usize> TryFromSexp for SVector<T, N> {
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let slice: &[T] = TryFromSexp::try_from_sexp(sexp)?;
        if slice.len() != N {
            return Err(SexpLengthError { expected: N, actual: slice.len() }.into());
        }
        Ok(SVector::from_column_slice(slice))
    }
}

impl<T: RNativeType + Scalar, const N: usize> IntoR for SVector<T, N> {
    fn into_sexp(self) -> SEXP {
        self.as_slice().into_sexp()
    }
}

// Similar for SMatrix<T, R, C>
```

**Use case**: Small linear algebra on stack, very common in graphics/physics
**Effort**: 2 hours

### 🟡 MEDIUM PRIORITY

#### 5. BinaryHeap<T: Ord>
```rust
impl<T: RNativeType + Ord> IntoR for BinaryHeap<T> {
    fn into_sexp(self) -> SEXP {
        self.into_vec().into_sexp()
    }
}
```
**Use case**: Priority queues, algorithmic work
**Effort**: 30 mins

#### 6. Cow<'_, T>
```rust
impl<T: IntoR + ToOwned> IntoR for Cow<'_, T>
where
    T::Owned: IntoR,
{
    fn into_sexp(self) -> SEXP {
        match self {
            Cow::Borrowed(b) => b.into_sexp(),
            Cow::Owned(o) => o.into_sexp(),
        }
    }
}
```
**Use case**: Zero-copy string/slice patterns
**Effort**: 1 hour

#### 7. i64 as RNativeType (with caveats)
```rust
// WARNING: R's REALSXP (f64) can only represent integers up to 2^53 exactly
impl RNativeType for i64 {
    const SEXP_TYPE: SEXPTYPE = SEXPTYPE::REALSXP;  // Maps to double!
    unsafe fn dataptr_mut(sexp: SEXP) -> *mut Self {
        REAL(sexp) as *mut i64  // Requires transmute or custom impl
    }
}
```
**Challenge**: Precision loss for large i64 values
**Use case**: Timestamps (which fit in 2^53), large counters
**Effort**: 1 hour + documentation

### 🟢 LOW PRIORITY (Skip)

- LinkedList - Not useful for R vectors
- i128/u128 - No R equivalent
- isize/usize - Platform-dependent
- Rc/Arc - Unclear semantics

---

## Part 2: Missing Tests

### 🔴 CRITICAL - No Tests for New Blanket Impls!

#### Test File 1: `miniextendr-api/tests/vecdeque.rs` (NEW)

**What to test**:
```rust
#[test]
fn test_vecdeque_i32_roundtrip() {
    // R vector → VecDeque<i32> → R vector
}

#[test]
fn test_vecdeque_f64_preserves_na() {
    // Verify NA handling in VecDeque<f64>
}

#[test]
fn test_vecdeque_empty() {
    // Empty R vector → empty VecDeque
}

#[test]
fn test_vecdeque_coerce_i8_to_i32() {
    // VecDeque<i8>.coerce() → VecDeque<i32>
}
```

**Effort**: 30 mins

#### Test File 2: `miniextendr-api/tests/tinyvec.rs` (NEW)

**What to test**:
```rust
#[test]
fn test_tinyvec_i32_roundtrip() {
    // R vector → TinyVec<[i32; 8]> → R vector
}

#[test]
fn test_tinyvec_stays_inline() {
    // Small vector stays inline (no heap allocation)
}

#[test]
fn test_tinyvec_spills_to_heap() {
    // Large vector spills to heap
}

#[test]
fn test_arrayvec_capacity_error() {
    // R vector too large → ArrayVec capacity error
}

#[test]
fn test_tinyvec_blanket_impl_all_types() {
    // Verify TinyVec works for i32, f64, u8, RLogical
}

#[test]
fn test_tinyvec_coerce_i8_to_i32() {
    // TinyVec<[i8; N]>.coerce() → TinyVec<[i32; N]>
}
```

**Effort**: 1 hour

#### Test File 3: `miniextendr-api/tests/nalgebra_blanket.rs` (NEW)

**What to test**:
```rust
#[test]
fn test_dvector_i32() {
    // Verify blanket impl works for i32
}

#[test]
fn test_dvector_u8() {
    // Verify blanket impl works for u8
}

#[test]
fn test_dmatrix_rlogical() {
    // Verify blanket impl works for RLogical
}

#[test]
fn test_dvector_generic_coverage() {
    // Verify all T: RNativeType + Scalar work
}
```

**Effort**: 30 mins

#### Test File 4: `miniextendr-api/tests/ndarray_blanket.rs` (NEW)

**What to test**:
```rust
#[test]
fn test_array1_i32() { ... }

#[test]
fn test_array2_u8() { ... }

#[test]
fn test_array0_scalar() { ... }

#[test]
fn test_arrayd_dynamic_dims() { ... }

#[test]
fn test_array_blanket_coverage() {
    // Verify Array0-6, ArrayD work for all T: RNativeType
}
```

**Effort**: 1 hour

#### Test Extension: `miniextendr-api/tests/coerce.rs`

**Add tests for**:
```rust
#[test]
fn test_widen_to_i32_marker_i8()  { ... }

#[test]
fn test_widen_to_i32_marker_i16() { ... }

#[test]
fn test_widen_to_i32_marker_u8()  { ... }

#[test]
fn test_widen_to_i32_marker_u16() { ... }

#[test]
fn test_widen_to_f64_marker_f32() { ... }

#[test]
fn test_widen_to_f64_marker_all_types() {
    // Verify all 7 types work via marker blanket impl
}

#[test]
fn test_tinyvec_element_coerce() {
    // TinyVec<[i8; 8]>.coerce() → TinyVec<[i32; 8]>
}

#[test]
fn test_vecdeque_element_coerce() {
    // VecDeque<i8>.coerce() → VecDeque<i32>
}
```

**Effort**: 1 hour

#### Test Extension: `miniextendr-api/tests/from_r.rs`

**Add tests for**:
```rust
#[test]
fn test_slice_arbitrary_lifetime() {
    // Verify &[T] works (not just &'static [T])
}

#[test]
fn test_slice_mut_arbitrary_lifetime() {
    // Verify &mut [T] works (not just &'static mut [T])
}

#[test]
fn test_slice_option_arbitrary_lifetime() {
    // Verify Option<&[T]> works
}
```

**Effort**: 30 mins

### 🟡 HIGH PRIORITY - Tests for Planned Features

#### Test File 5: `miniextendr-api/tests/bytes.rs` (NEW - if conversions added)

```rust
#[test]
fn test_bytes_from_raw() { ... }

#[test]
fn test_bytes_to_raw() { ... }

#[test]
fn test_bytesmut_roundtrip() { ... }
```

**Effort**: 30 mins (if Bytes conversions implemented)

#### Test File 6: `miniextendr-api/tests/fixed_arrays.rs` (NEW - if conversions added)

```rust
#[test]
fn test_u8_array_32_roundtrip() {
    // [u8; 32] → R raw → [u8; 32]
}

#[test]
fn test_i32_array_length_mismatch_error() {
    // R vector wrong length → error
}
```

**Effort**: 30 mins (if fixed array conversions implemented)

---

## Part 3: What Exists vs What's Missing

### Std Collections: 6/12 Implemented (50%)

| Collection | Status | Priority | Effort |
|------------|--------|----------|--------|
| Vec | ✅ Done | - | - |
| VecDeque | ✅ Done | - | - |
| HashMap | ✅ Done | - | - |
| BTreeMap | ✅ Done | - | - |
| HashSet | ✅ Done | - | - |
| BTreeSet | ✅ Done | - | - |
| BinaryHeap | ❌ Missing | 🟡 Medium | 30 mins |
| LinkedList | ❌ Missing | ⚪ Low | Skip |
| Box | ❌ Can't add | - | Conflicts |
| Rc | ❌ Missing | ⚪ Low | Skip |
| Arc | ❌ Missing | ⚪ Low | Skip |
| Cow | ❌ Missing | 🟡 Medium | 1 hour |

### Std Numerics: 3/14 RNativeType (21%)

| Type | RNativeType | Coerce→i32 | Coerce→f64 | Priority | Effort |
|------|-------------|------------|------------|----------|--------|
| i8 | ❌ | ✅ | ✅ | 🔴 HIGH | 30 mins |
| i16 | ❌ | ✅ | ✅ | 🔴 HIGH | 30 mins |
| i32 | ✅ | ✅ | ✅ | - | - |
| i64 | ❌ | ❌ | ❌ | 🟡 Medium | 1 hour |
| i128 | ❌ | ❌ | ❌ | ⚪ Low | Skip |
| isize | ❌ | ❌ | ❌ | ⚪ Low | Skip |
| u8 | ✅ | ✅ | ✅ | - | - |
| u16 | ❌ | ✅ | ✅ | 🟡 Medium | 30 mins |
| u32 | ❌ | ❌ | ✅ | 🟡 Medium | 30 mins |
| u64 | ❌ | ❌ | ❌ | ⚪ Low | Skip |
| u128 | ❌ | ❌ | ❌ | ⚪ Low | Skip |
| usize | ❌ | ❌ | ❌ | ⚪ Low | Skip |
| f32 | ❌ | ❌ | ✅ | 🔴 HIGH | 30 mins |
| f64 | ✅ | ❌ | ✅ | - | - |

### Nalgebra: 2/4 Main Types (50%)

| Type | Status | Priority | Effort |
|------|--------|----------|--------|
| DVector<T> | ✅ Blanket | - | - |
| DMatrix<T> | ✅ Blanket | - | - |
| SVector<T, N> | ❌ | 🔴 HIGH | 1 hour |
| SMatrix<T, R, C> | ❌ | 🟡 Medium | 1 hour |

### Optional Crates: 18/24 (75%)

| Crate | Status | Missing | Priority |
|-------|--------|---------|----------|
| aho_corasick | ✅ 1 impl | - | - |
| bitflags | ✅ 8 impls | - | - |
| bitvec | ✅ 6 impls | - | - |
| **bytes** | ❌ 0 impls | Bytes, BytesMut | 🔴 HIGH |
| either | ✅ 2 impls | - | - |
| indexmap | ✅ 2 impls | - | - |
| nalgebra | ✅ 4 impls | SVector, SMatrix | 🔴 HIGH |
| ndarray | ✅ 22 impls | (Complete) | - |
| num_bigint | ✅ 16 impls | - | - |
| num_complex | ✅ 8 impls | - | - |
| **num_traits** | ✅ 0 impls | (Traits only - correct) | - |
| ordered_float | ✅ 16 impls | - | - |
| **rand** | ✅ 0 impls | (Traits only - correct) | - |
| **rayon** | ✅ 0 impls | (Bridge only - correct) | - |
| regex | ✅ 4 impls | - | - |
| rust_decimal | ✅ 8 impls | - | - |
| serde | ✅ 5 impls | - | - |
| **sha2** | ❌ 0 impls | Hash outputs? | 🟡 Medium |
| tabled | ✅ 1 impl | - | - |
| time | ✅ 16 impls | - | - |
| tinyvec | ✅ 8 impls | - | - |
| toml | ✅ 7 impls | - | - |
| url | ✅ 8 impls | - | - |
| uuid | ✅ 8 impls | - | - |

---

## Part 4: Missing Tests

### 🔴 CRITICAL - Zero Integration Tests for New Code

| Feature | Test File Needed | Priority | Effort |
|---------|------------------|----------|--------|
| **VecDeque** | tests/vecdeque.rs | 🔴 CRITICAL | 30 mins |
| **TinyVec blanket impl** | tests/tinyvec.rs | 🔴 CRITICAL | 1 hour |
| **Arbitrary lifetime slices** | tests/from_r.rs (extend) | 🔴 CRITICAL | 30 mins |
| **Marker trait coercion** | tests/coerce.rs (extend) | 🔴 CRITICAL | 1 hour |
| **Nalgebra blanket coverage** | tests/nalgebra_generic.rs | 🔴 CRITICAL | 30 mins |
| **Ndarray blanket coverage** | tests/ndarray_generic.rs | 🔴 CRITICAL | 30 mins |
| **Container element coercion** | tests/container_coerce.rs | 🔴 CRITICAL | 1 hour |

**Total critical test effort**: 5.5 hours

### 🟡 HIGH PRIORITY - Tests for Planned Features

| Feature | Test File | Effort |
|---------|-----------|--------|
| Bytes conversions | tests/bytes.rs | 30 mins |
| Fixed arrays | tests/fixed_arrays.rs | 30 mins |
| SVector/SMatrix | tests/nalgebra_static.rs | 1 hour |

**Total planned test effort**: 2 hours

### Test Coverage Summary

**Current**:
- ✅ Existing features well-tested (uuid, time, regex, etc.)
- ✅ Coerce unit tests (41 tests in lib)
- ❌ **NEW blanket impls have ZERO integration tests**

**This is dangerous!** We've added:
- VecDeque conversions - NOT TESTED
- Arbitrary lifetime slices - NOT TESTED
- TinyVec/nalgebra/ndarray blanket impls - NOT TESTED
- Marker trait coercion - NOT TESTED
- Container element coercion - NOT TESTED

---

## Part 5: Implementation Priority Matrix

### Immediate (Do First)

1. **Add critical tests** (5.5 hours)
   - VecDeque integration tests
   - TinyVec integration tests
   - Marker trait coercion tests
   - Blanket impl coverage tests

**Rationale**: We just rewrote 1200 lines of code with NO integration tests. This is risky.

### High Priority (Do Next)

2. **Add i8, i16, f32 as RNativeType** (1.5 hours)
   - Each unlocks ~100+ conversions
   - High value, low effort

3. **Add Bytes conversions** (1 hour)
   - Common use case
   - Simple implementation

4. **Add fixed-size array conversions** (2 hours)
   - Enables SHA hashes, fixed patterns
   - Moderate complexity (requires Default or MaybeUninit)

5. **Add nalgebra SVector/SMatrix** (2 hours)
   - Stack-allocated linear algebra
   - Common in scientific computing

**Total high-priority**: 7 hours

### Medium Priority (Consider)

6. **Add BinaryHeap** (30 mins)
7. **Add Cow** (1 hour)
8. **Add i64 RNativeType** (1 hour)
9. **Update documentation** (2 hours)

**Total medium-priority**: 4.5 hours

### Low Priority (Skip)

- LinkedList, Rc, Arc - Not useful for R
- i128, u128, isize, usize - No good R mapping

---

## Part 6: Quick Wins

### Can Implement in < 30 Minutes Each

1. **BinaryHeap<T>** - Trivial wrapper around Vec
2. **i8 RNativeType** - Copy i32 pattern, change pointer cast
3. **i16 RNativeType** - Copy i32 pattern, change pointer cast
4. **u16 RNativeType** - Copy u8 pattern, change SEXP_TYPE to INTSXP

### Moderate Effort (1-2 hours)

1. **f32 RNativeType** - Needs widening/narrowing logic
2. **Bytes conversions** - Straightforward Vec<u8> wrapper
3. **Fixed-size arrays** - Needs length check + copy logic
4. **SVector/SMatrix** - Needs dimension checking

---

## Final Recommendations

### Do This Week (Critical)

✅ **Phase 1**: Add critical tests (5.5 hours)
- Must verify new blanket impls work correctly
- Currently shipping untested code

✅ **Phase 2**: Add i8, i16, f32 as RNativeType (1.5 hours)
- Massive value (300+ conversions per type)
- Low effort, high impact

✅ **Phase 3**: Add Bytes conversions (1 hour)
- Common use case
- Simple to implement

**Total effort**: 8 hours
**Value**: Test coverage + 300+ new conversions + Bytes support

### Do Next Month (Nice to Have)

- Fixed-size arrays
- nalgebra SVector/SMatrix
- BinaryHeap/Cow
- i64 support (with docs about precision)
- Documentation updates

### Skip (Low Value)

- LinkedList, smart pointers (Rc/Arc)
- i128/u128, isize/usize
- Platform-specific types

---

## Test Coverage Gap Analysis

**Current State**:
- 181 unit tests in lib.rs ✅
- ~20 integration test files exist ✅
- **BUT**: New blanket impl code has 0 integration tests ❌

**Risk Level**: 🔴 HIGH
- Rewrote ~1200 lines with blanket impls
- No integration tests verify the new code works
- Could have subtle bugs in lifetime handling, type constraints, etc.

**Mitigation**: Add integration tests immediately (5.5 hours)

---

## Summary

**What's Left**:

1. 🔴 **CRITICAL**: Add integration tests for new blanket impls (5.5 hours)
2. 🔴 **HIGH**: Add i8, i16, f32 as RNativeType (1.5 hours)
3. 🔴 **HIGH**: Add Bytes/BytesMut conversions (1 hour)
4. 🟡 **MEDIUM**: Add fixed arrays, SVector/SMatrix, BinaryHeap/Cow (5 hours)
5. 🟡 **MEDIUM**: Update documentation (2 hours)

**Estimated total**: 15 hours for everything, 8 hours for critical path

**Next action**: Start with critical tests to verify the blanket impl migration is solid.
