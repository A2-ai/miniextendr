# Session Changes Summary

This document summarizes all changes made during the debugging session for heterogeneous DataFrameRow support.

## Files Modified

### 1. `miniextendr-macros/src/dataframe_derive.rs`

**Changes:**
- Removed unused `CollectionType` enum variants (`BoxedSlice`, `Slice`, `Array`)
- Simplified to only support `Vec<T>` collections
- Changed from using `#(#repetitions),*` to explicitly building `TokenStream` in loops
- Added concrete `std::vec::IntoIter<#ty>` types instead of associated types
- Used fully qualified call syntax: `<Vec<#ty>>::into_iter(name)`
- Temporarily disabled `IntoList` trait assertion for debugging
- Removed helper functions `parse_collection_type()` and simplified `wrap_in_vec()`

**Current Status:** Compiles but heterogeneous types still fail type checking

### 2. `rpkg/src/rust/dataframe_examples.rs`

**Added:**
- `SimplePerson` struct with `String` and `i32` fields (heterogeneous test case)
- `Person` struct with `String`, `i32`, `f64`, `bool` fields (complex heterogeneous test)
- `create_people_df()` function to test heterogeneous DataFrames
- Manual `IntoList` impl for `SimplePerson` (for debugging)

**Current Status:** `Point` (homogeneous) compiles, `SimplePerson` and `Person` (heterogeneous) fail

### 3. `rpkg/tests/testthat/test-dataframe.R` (Created)

**Contents:**
- Tests for homogeneous DataFrame (`create_points_df`)
- Tests for heterogeneous DataFrame (`create_people_df`)
- Checks for correct types, dimensions, and values

**Current Status:** File created but tests cannot run until compilation succeeds

## Files Created for Debugging

### 4. `rpkg/src/rust/manual_dataframe_test.rs` (Created)

**Purpose:** Manual implementation to verify the approach works
**Status:** ✅ Compiles successfully
**Conclusion:** The code structure is valid; issue is macro-specific

### 5. `rpkg/src/rust/debug_test.rs` (Created)

**Purpose:** Minimal non-macro test case
**Status:** ✅ Compiles successfully
**Conclusion:** Confirms the issue is with proc macro expansion

## Documentation Files Created

### 6. `DATAFRAME_HETEROGENEOUS_BUG.md` (Created)

Comprehensive documentation of:
- Problem statement
- Working vs failing examples
- Generated code inspection
- All attempted solutions
- Hypotheses about root cause
- Environment details

### 7. `MINIMAL_REPRO.md` (Created)

Minimal reproduction case with:
- Simplest failing example
- Core macro logic
- Test variations to try
- Debugging questions

### 8. `SESSION_CHANGES.md` (This file)

Summary of all changes made during the session.

## What Works

✅ Homogeneous types (e.g., `Point { x: f64, y: f64 }`)
✅ Manual implementation of heterogeneous DataFrames
✅ Basic DataFrame struct generation
✅ `From<Vec<Row>>` impl generation (code looks correct)
✅ Iterator implementation generation (code looks correct)

## What Doesn't Work

❌ Macro-generated heterogeneous types
❌ Type inference for fields beyond the first field
❌ Compilation despite apparently correct generated code

## Error Pattern

All errors show:
```
expected `Vec<String>`, found `Vec<i32>` (or f64, bool, etc.)
```

Where `String` is always the first field's type, suggesting all fields are somehow being inferred as the first field's type.

## Approaches That Were Tried (All Failed)

1. Explicit turbofish: `.collect::<Vec<#ty>>()`
2. Concrete types: `std::vec::IntoIter<#ty>`
3. Fully qualified syntax: `<Vec<#ty>>::into_iter()`
4. Building tokens explicitly (no repetition)
5. Destructuring patterns
6. Associated types vs concrete types
7. Disabling trait bounds
8. Different TokenStream assembly methods

## ✅ RESOLVED (2026-01-30)

**Root Cause Identified:** The issue was in the `IntoDataFrame` impl, not the iterator code!

The macro was using:
```rust
List::from_pairs(vec![("name", self.name), ("age", self.age), ...])
```

`List::from_pairs()` is generic over **one type** `T: IntoR`, so Rust infers a single type for all values in the vector (the first field's type), causing type mismatches.

**Fix Applied:** Switch to `List::from_raw_pairs()` with explicit SEXP conversion:
```rust
List::from_raw_pairs(vec![
    ("name", IntoR::into_sexp(self.name)),
    ("age", IntoR::into_sexp(self.age)),
    ...
])
```

This allows each field to be independently converted, avoiding the homogeneous type constraint.

**Result:** ✅ All tests pass! Heterogeneous DataFrames now work correctly.

## Cleanup Needed (Optional)

The following temporary files can be removed once the issue is resolved:
- `rpkg/src/rust/manual_dataframe_test.rs`
- `rpkg/src/rust/debug_test.rs`
- References to these modules in `rpkg/src/rust/lib.rs`

## Configuration to Run Tests

Once compilation works, tests can be run with:
```bash
NOT_CRAN=true just configure
NOT_CRAN=true just rcmdinstall
NOT_CRAN=true just devtools-document
NOT_CRAN=true just devtools-test
```

**Note:** All R CMD operations require `dangerouslyDisableSandbox: true` due to sandbox restrictions on compilation.

## Git Status

Uncommitted changes in:
- `miniextendr-macros/src/dataframe_derive.rs`
- `rpkg/src/rust/dataframe_examples.rs`
- `rpkg/tests/testthat/test-dataframe.R` (new)
- Documentation files (new)

Consider creating a feature branch before committing:
```bash
git checkout -b feature/dataframe-heterogeneous-types
git add <files>
git commit -m "WIP: Add heterogeneous type support to DataFrameRow (needs debugging)"
```

## Key Insights

1. **The generated code structure is valid** - Manual implementation proves this
2. **The issue is proc-macro-specific** - Same code fails when generated by macro
3. **Type inference fails across fields** - All fields expect first field's type
4. **Multiple remediation strategies failed** - Suggesting a fundamental issue with token generation or expansion

## Questions for Expert Review

1. Could this be a known issue with `quote!` macro and heterogeneous struct initialization?
2. Is there something about `TokenStream` assembly that affects type inference?
3. Would a completely different code generation strategy (e.g., using `syn::parse_quote!` differently) help?
4. Is this potentially a Rust compiler bug in how it handles proc-macro-generated code with multiple type parameters?
