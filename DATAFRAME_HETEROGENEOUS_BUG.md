# DataFrameRow Macro: Heterogeneous Type Inference Issue

## Summary

The `DataFrameRow` derive macro successfully compiles for structs with homogeneous field types (e.g., all `f64`), but fails with type mismatch errors for structs with heterogeneous field types (e.g., `String`, `i32`, `f64`, `bool`).

## Problem Statement

When deriving `DataFrameRow` for a struct with different field types, the Rust compiler reports type mismatches where all fields expect the first field's type (`Vec<String>`), even though the generated code appears to specify the correct types for each field.

## Working Example (Homogeneous Types)

```rust
#[derive(Clone, Debug, IntoList, DataFrameRow)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}
```

**Status:** ✅ Compiles successfully

## Failing Example (Heterogeneous Types)

```rust
#[derive(Clone, Debug, DataFrameRow)]
pub struct SimplePerson {
    pub name: String,
    pub age: i32,
}
```

**Status:** ❌ Fails with:
```
error[E0308]: mismatched types
  --> dataframe_examples.rs:26:24
   |
26 | #[derive(Clone, Debug, DataFrameRow)]
   |                        ^^^^^^^^^^^^ expected `Vec<String>`, found `Vec<i32>`
```

## Generated Code (Via Debug Output)

The macro generates what appears to be correct code:

### DataFrame Struct
```rust
pub struct SimplePersonDataFrame {
    pub name: Vec<String>,
    pub age: Vec<i32>
}
```

### From Implementation
```rust
impl From<Vec<SimplePerson>> for SimplePersonDataFrame {
    fn from(rows: Vec<SimplePerson>) -> Self {
        SimplePersonDataFrame {
            name: rows.iter().map(|r| r.name.clone()).collect::<Vec<String>>(),
            age: rows.iter().map(|r| r.age.clone()).collect::<Vec<i32>>()
        }
    }
}
```

### Iterator Implementation
```rust
pub struct SimplePersonDataFrameIterator {
    name: std::vec::IntoIter<String>,
    age: std::vec::IntoIter<i32>
}

impl IntoIterator for SimplePersonDataFrame {
    type Item = SimplePerson;
    type IntoIter = SimplePersonDataFrameIterator;

    fn into_iter(self) -> Self::IntoIter {
        let SimplePersonDataFrame { name, age } = self;
        SimplePersonDataFrameIterator {
            name: <Vec<String>>::into_iter(name),
            age: <Vec<i32>>::into_iter(age)
        }
    }
}

impl Iterator for SimplePersonDataFrameIterator {
    type Item = SimplePerson;

    fn next(&mut self) -> Option<Self::Item> {
        Some(SimplePerson {
            name: self.name.next()?,
            age: self.age.next()?
        })
    }
}
```

## Manual Implementation (Control Test)

A manual implementation with identical structure compiles successfully:

```rust
pub struct ManualTest {
    pub name: String,
    pub age: i32,
}

pub struct ManualTestDataFrame {
    pub name: Vec<String>,
    pub age: Vec<i32>,
}

impl From<Vec<ManualTest>> for ManualTestDataFrame {
    fn from(rows: Vec<ManualTest>) -> Self {
        ManualTestDataFrame {
            name: rows.iter().map(|r| r.name.clone()).collect::<Vec<String>>(),
            age: rows.iter().map(|r| r.age.clone()).collect::<Vec<i32>>(),
        }
    }
}

pub struct ManualTestDataFrameIterator {
    name: std::vec::IntoIter<String>,
    age: std::vec::IntoIter<i32>,
}

impl IntoIterator for ManualTestDataFrame {
    type Item = ManualTest;
    type IntoIter = ManualTestDataFrameIterator;

    fn into_iter(self) -> Self::IntoIter {
        let ManualTestDataFrame { name, age } = self;
        ManualTestDataFrameIterator {
            name: name.into_iter(),
            age: age.into_iter(),
        }
    }
}

impl Iterator for ManualTestDataFrameIterator {
    type Item = ManualTest;

    fn next(&mut self) -> Option<Self::Item> {
        Some(ManualTest {
            name: self.name.next()?,
            age: self.age.next()?,
        })
    }
}
```

**Status:** ✅ Compiles successfully

## Approaches Attempted

All of the following approaches have been tried without success:

### 1. Explicit Type Annotations
```rust
.collect::<Vec<#ty>>()
```

### 2. Concrete Iterator Types
```rust
#name: std::vec::IntoIter<#ty>
```

### 3. Fully Qualified Call Syntax
```rust
<Vec<#ty>>::into_iter(#name)
```

### 4. Destructuring with Type Annotations
```rust
let #df_name { #name: Vec<#ty>, ... } = self;
```
(Note: This is invalid Rust syntax in patterns)

### 5. Building Tokens Explicitly Instead of Repetition
```rust
let mut tokens = TokenStream::new();
for (i, (name, ty)) in field_info.iter().enumerate() {
    if i > 0 {
        tokens.extend(quote! { , });
    }
    tokens.extend(quote! { #name: <Vec<#ty>>::into_iter(#name) });
}
```

### 6. Associated Type vs Concrete Type
Both `<Vec<#ty> as IntoIterator>::IntoIter` and `std::vec::IntoIter<#ty>` were tried.

### 7. Disabling Trait Bounds Check
The compile-time assertion for `IntoList` was temporarily disabled.

## Current Implementation Location

**File:** `miniextendr-macros/src/dataframe_derive.rs`
**Function:** `derive_dataframe_row()`

Key sections (lines 107-189):
- From impl: Lines 107-126
- Iterator struct and impls: Lines 128-189

## Error Pattern

All type errors follow the same pattern:
- **Expected:** `Vec<String>` (the first field's type)
- **Found:** `Vec<i32>`, `Vec<f64>`, `Vec<bool>` (the actual field types)
- **Location:** All errors point to the derive attribute location

This suggests that somehow all fields are being inferred as having the first field's type, despite explicit type annotations throughout the generated code.

## User Hints

Two hints were provided:
1. "the container too needs to be inferred I think"
2. "I think if the DataFrame'd struct has let's say a slice, then the field has to be set to a slice, and so on.."

These suggest the issue may be related to how container types are being inferred or specified in the macro output, but the exact meaning and solution remain unclear.

## Hypotheses

### H1: Quote Macro Repetition Issue
The `quote!` macro's repetition syntax `#(#tokens),*` may have issues with type inference when fields have different types. However, attempts to build tokens explicitly without repetition also failed.

### H2: Macro Hygiene/Span Issue
There may be an issue with how spans or hygiene markers are being set that causes Rust to incorrectly associate types across fields.

### H3: Unknown Type Inference Edge Case
There may be a subtle type inference rule in Rust that applies differently to proc-macro-generated code versus hand-written code.

### H4: Token Stream Structure
The structure of how tokens are assembled might matter in ways that aren't obvious from the printed output.

## Environment

- **Rust version:** 1.93.0 (2026-01-19)
- **Project:** miniextendr (R-Rust interop framework)
- **Macro crate:** miniextendr-macros v0.1.0
- **Dependencies:** proc-macro2, quote, syn

## Questions for Investigation

1. Is there a way to see the exact token stream structure (not just the printed representation) to verify field types are truly distinct?

2. Could this be related to how `proc-macro2::TokenStream` handles insertions and extensions?

3. Is there a known issue with `quote!` macro when dealing with heterogeneous types in struct initializations?

4. Would using `quote_spanned!` with specific spans help Rust's type inference?

5. Is there something about procedural macro expansion order that could cause type information to be lost?

## Reproduction Steps

1. Clone repository: `https://github.com/anthropics/miniextendr` (assuming public)
2. Navigate to: `miniextendr-macros/src/dataframe_derive.rs`
3. Examine the `derive_dataframe_row()` function
4. Try to compile `rpkg/src/rust/dataframe_examples.rs` which uses the macro
5. Observe type errors for `SimplePerson` and `Person` structs

## Request for Help

We need assistance understanding why the macro-generated code fails type checking despite appearing structurally identical to working manual implementations. Any insights into:
- Proc macro best practices for heterogeneous type handling
- Debugging techniques for proc macro type inference issues
- Known issues with `quote!` and type inference
- Alternative code generation strategies

Would be greatly appreciated.

---

## Resolution / Fix Applied (2026-01-30)

**Root cause:** The generated `IntoDataFrame` implementation used
`List::from_pairs(vec![(name, self.field), ...])`. `from_pairs` is
*homogeneous* because it is generic over a single `T: IntoR`, so the compiler
infers **one** `T` for the entire vector. That forces all columns to the first
field’s type (e.g., `Vec<String>`), which explains the “expected `Vec<String>`,
found `Vec<i32>`” errors even though the macro output looks correct.

**Fix:** Switch the macro to build data frames with raw SEXPs:

- Convert each column to a `SEXP` using `IntoR::into_sexp`.
- Use `List::from_raw_pairs(...)` (heterogeneous-friendly).

This mirrors the existing `IntoList` derive behavior and avoids the single‑`T`
inference trap.

### Code change (macro)

```rust
let df_pairs = field_names.iter().map(|name| {
    let name_str = name.to_string();
    quote! { (#name_str, ::miniextendr_api::IntoR::into_sexp(self.#name)) }
});

::miniextendr_api::list::List::from_raw_pairs(vec![
    #(#df_pairs),*
])
.set_class_str(&["data.frame"])
.set_row_names_int(n_rows)
```

**Location:** `miniextendr-macros/src/dataframe_derive.rs` in the
`IntoDataFrame` impl generation.
