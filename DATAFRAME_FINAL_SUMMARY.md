# DataFrameRow: Heterogeneous Types & Collection Support - Complete

## Executive Summary

The `DataFrameRow` derive macro now fully supports:
1. ✅ **Heterogeneous field types** (String, i32, f64, bool in the same struct)
2. ✅ **Multiple collection types** (Vec, Box<[]>, arrays, HashSet, BTreeSet)
3. ✅ **Mixed collections** (different collection types in the same struct)

**Status:** All tests passing (20 R tests + 6 Rust tests)

## Problem & Resolution

### Original Issue
The macro failed for structs with different field types (heterogeneous), only working for homogeneous types (all fields the same type).

**Error Pattern:**
```
error[E0308]: expected `Vec<String>`, found `Vec<i32>`
```
All fields expected the first field's type, despite correct-looking generated code.

### Root Cause Discovered
The bug was in the `IntoDataFrame` implementation:

```rust
// ❌ BROKEN: from_pairs is generic over ONE type T
List::from_pairs(vec![
    ("name", self.name),    // Vec<String>
    ("age", self.age),      // Vec<i32> ← Error! Expected Vec<String>
])
```

`List::from_pairs<T>()` is generic over a single type `T: IntoR`, forcing all values to be the same type. The compiler inferred `T = Vec<String>` from the first field, causing mismatches for other fields.

### Solution Applied
Switch to `List::from_raw_pairs()` which accepts heterogeneous `SEXP` values:

```rust
// ✅ FIXED: Each field converted independently to SEXP
List::from_raw_pairs(vec![
    ("name", IntoR::into_sexp(self.name)),    // Vec<String> → SEXP
    ("age", IntoR::into_sexp(self.age)),      // Vec<i32> → SEXP
])
```

Each field is independently converted to `SEXP`, avoiding the type unification constraint.

**Changed:** [dataframe_derive.rs:88-91](miniextendr-macros/src/dataframe_derive.rs#L88-L91)

## Collection Type Support Added

### IntoR Implementations Added
**File:** `miniextendr-api/src/into_r.rs` (lines 1517-1613)

| Type | Implementation | R Representation |
|------|----------------|------------------|
| `Vec<Box<[T]>>` (RNativeType) | Generic impl | List of typed vectors |
| `Vec<Box<[String]>>` | Specific impl | List of character vectors |
| `Vec<[T; N]>` (RNativeType) | Generic impl with const N | List of typed vectors |
| `Vec<HashSet<T>>` (RNativeType) | Generic impl | List of unordered vectors |
| `Vec<HashSet<String>>` | Specific impl | List of character vectors |
| `Vec<BTreeSet<T>>` (RNativeType) | Generic impl | List of ordered vectors |
| `Vec<BTreeSet<String>>` | Specific impl | List of character vectors |

### Design Pattern
All implementations follow the same pattern:
1. Allocate R list (VECSXP)
2. Convert each collection element to Vec
3. Convert Vec to SEXP
4. Set as list element

## Test Coverage

### Rust Tests (`dataframe_collections_test.rs`)
```
test dataframe_collections_test::tests::test_vec_dataframe ... ok
test dataframe_collections_test::tests::test_boxed_slice_dataframe ... ok
test dataframe_collections_test::tests::test_array_dataframe ... ok
test dataframe_collections_test::tests::test_hashset_dataframe ... ok
test dataframe_collections_test::tests::test_btreeset_dataframe ... ok
test dataframe_collections_test::tests::test_mixed_collections ... ok
```

### R Tests (`test-dataframe.R`)
```
✓ DataFrameRow works with homogeneous types (7 tests)
✓ DataFrameRow works with heterogeneous types (13 tests)
```

All 26 tests passing!

## Examples

### Heterogeneous Types
```rust
#[derive(Clone, Debug, IntoList, DataFrameRow)]
pub struct Person {
    pub name: String,        // character in R
    pub age: i32,            // integer in R
    pub height: f64,         // numeric in R
    pub is_student: bool,    // logical in R
}
```

**R Output:**
```r
> create_people_df()
     name age height is_student
1   Alice  25  165.5       TRUE
2     Bob  30  180.0      FALSE
3 Charlie  28  175.2       TRUE
```

### Collection Fields
```rust
#[derive(Clone, Debug, DataFrameRow)]
pub struct Measurement {
    pub tags: HashSet<String>,  // Each row has a set of tags
    pub coords: [f64; 3],        // Each row has 3D coordinates
    pub samples: Vec<i32>,       // Each row has variable samples
}
```

**DataFrame:** Each field becomes `Vec<CollectionType>`
- `tags: Vec<HashSet<String>>`
- `coords: Vec<[f64; 3]>`
- `samples: Vec<Vec<i32>>`

**R Representation:** List columns where each element is a vector/list

## Files Changed

### Core Implementation
1. **miniextendr-macros/src/dataframe_derive.rs** (230 lines)
   - Changed `from_pairs` → `from_raw_pairs` (line 97)
   - Simplified to Vec-only collections (removed unused variants)
   - Improved iterator type inference

2. **miniextendr-api/src/into_r.rs** (1613 lines, +97 lines added)
   - Added 7 new `IntoR` implementations for collection types

### Tests & Examples
3. **rpkg/src/rust/dataframe_examples.rs** (85 lines)
   - Added `SimplePerson` (2 fields: String, i32)
   - Added `Person` (4 fields: String, i32, f64, bool)
   - Added `create_people_df()` export

4. **rpkg/src/rust/dataframe_collections_test.rs** (142 lines, new)
   - 6 comprehensive collection type tests
   - Round-trip verification

5. **rpkg/tests/testthat/test-dataframe.R** (38 lines, new)
   - Tests for homogeneous DataFrames
   - Tests for heterogeneous DataFrames
   - Type checking and value verification

### Documentation
6. **DATAFRAME_HETEROGENEOUS_BUG.md** - Issue report with resolution
7. **DATAFRAME_COLLECTIONS_SUPPORT.md** - Collection type guide
8. **SESSION_CHANGES.md** - Complete change log
9. **MINIMAL_REPRO.md** - Minimal reproduction case

## Key Insights

### Why from_pairs Failed
`List::from_pairs<T: IntoR>(vec![(k, v), ...])` requires all values to be the same type `T`. The compiler picks the first field's type as `T`, causing all subsequent fields to fail type checking.

### Why from_raw_pairs Works
`List::from_raw_pairs(vec![(k, SEXP), ...])` accepts already-converted SEXPs. Since each field is independently converted via `IntoR::into_sexp()`, there's no type unification.

### Pattern for Heterogeneous Data
This pattern applies anywhere you need to handle heterogeneous types in proc macros:
```rust
// ❌ Forces single type T
vec![item1, item2, item3]  // All must be same type

// ✅ Allows different types
vec![
    item1.into_common_type(),
    item2.into_common_type(),  // Each independently converted
    item3.into_common_type(),
]
```

## Performance & Memory

### Cloning in From<Vec<Row>>
The generated `From<Vec<Row>>` implementation clones field values:
```rust
name: rows.iter().map(|r| r.name.clone()).collect()
```

For large data, consider:
- Pre-allocating DataFrame with capacity
- Using `Box<[T]>` for zero-copy moves
- Implementing custom conversions for very large datasets

### Collection Conversions
HashSet and BTreeSet conversions have overhead:
- HashSet → Vec loses hash benefits (O(1) lookup → O(n))
- BTreeSet → Vec loses tree benefits (O(log n) → O(n))

Use these types only when set semantics are needed in Rust.

## Future Work

### Potential Enhancements
1. **Custom collection attributes**
   ```rust
   #[dataframe(collection = "Box<[T]>")]
   pub struct Row { ... }
   ```

2. **Lazy/ALTREP support**
   - Zero-copy DataFrame views
   - On-demand materialization

3. **Slice references** with lifetime management
   ```rust
   pub struct Row<'a> {
       data: &'a [f64],
   }
   ```

4. **HashMap/IndexMap support**
   - Named nested data structures

### Completed ✅
- [x] Heterogeneous field types
- [x] Vec<T> collections
- [x] Box<[T]> collections
- [x] [T; N] arrays
- [x] HashSet<T> collections
- [x] BTreeSet<T> collections
- [x] Mixed collection types
- [x] Round-trip conversions
- [x] Comprehensive test suite

## Usage Recommendation

For most use cases, use the standard pattern:
```rust
#[derive(Clone, Debug, IntoList, DataFrameRow)]
pub struct MyRow {
    pub field1: Type1,
    pub field2: Type2,
    // ... any types that implement IntoR
}
```

The macro handles the rest automatically!

## Verification

```bash
# Rust tests
cargo test --lib dataframe

# R tests
Rscript -e "testthat::test_file('tests/testthat/test-dataframe.R')"

# Both should show: ALL TESTS PASSING ✅
```

---

**Implementation Date:** 2026-01-30
**Total Lines Changed:** ~400 lines
**Test Coverage:** 26 tests (20 R + 6 Rust)
**Status:** Production Ready ✅
