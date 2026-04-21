+++
title = "ALTREP Quick Reference"
weight = 30
description = "One-page reference for miniextendr's ALTREP system."
+++

One-page reference for miniextendr's ALTREP system.

## Conversion Methods

### IntoR vs IntoRAltrep

| Method | Conversion | Memory | Speed (1M elements) | Use When |
|--------|------------|--------|---------------------|----------|
| `.into_sexp()` | Copy to R | R heap +3.8 MB | 0.44 ms | Small data, R needs DATAPTR |
| `.into_sexp_altrep()` | Zero-copy wrap | R heap +0 MB | 0.20 ms (2.2x faster) | Large data, lazy eval |
| `Altrep(x)` | Same as above | R heap +0 MB | 0.20 ms (2.2x faster) | Explicit wrapper style |

**Measured on Apple M-series, R 4.5*

### Quick Decision Guide

```text
Is your data > 1000 elements?
├─ Yes → Use .into_sexp_altrep()
└─ No
   └─ Will R modify it?
      ├─ Yes → Use .into_sexp() (copy)
      └─ No → Either works, .into_sexp() is simpler
```

### Examples

```rust
// Small data - copy is fine
#[miniextendr]
fn get_config() -> Vec<i32> {
    vec![1, 2, 3]  // Automatically copies via IntoR
}

// Large data - use ALTREP
#[miniextendr]
fn get_large_data() -> SEXP {
    let data = vec![0; 1_000_000];
    data.into_sexp_altrep()  // Zero-copy!
}

// Lazy computation - definitely ALTREP
#[miniextendr]
fn fibonacci_seq(n: i32) -> SEXP {
    fibonacci_iterator(n as usize)
        .collect::<Vec<_>>()
        .into_sexp_altrep()
}

// Range - already lazy
#[miniextendr]
fn int_range(from: i32, to: i32) -> SEXP {
    (from..to).collect::<Vec<_>>().into_sexp_altrep()
}
```

## The 7 Vector Types

| R Type | Trait | Element Type | Required Method |
|--------|-------|--------------|-----------------|
| integer | `AltIntegerData` | `i32` | `fn elt(&self, i: usize) -> i32` |
| numeric | `AltRealData` | `f64` | `fn elt(&self, i: usize) -> f64` |
| logical | `AltLogicalData` | `Logical` | `fn elt(&self, i: usize) -> Logical` |
| raw | `AltRawData` | `u8` | `fn elt(&self, i: usize) -> u8` |
| complex | `AltComplexData` | `Rcomplex` | `fn elt(&self, i: usize) -> Rcomplex` |
| character | `AltStringData` | `Option<&str>` | `fn elt(&self, i: usize) -> Option<&str>` |
| list | `AltListData` | `SEXP` | `fn elt(&self, i: usize) -> SEXP` |

## Minimal Example: Field-Based Derive (1 step)

```rust
// Everything generated from the derive - no trait impls needed
#[derive(miniextendr_api::AltrepInteger)]
#[altrep(len = "len", elt = "value", class = "MyClass")]
pub struct MyData { value: i32, len: usize }
```

## Minimal Example: Manual Traits (3 steps)

```rust
// 1. Define data struct with registration derive
#[derive(miniextendr_api::Altrep)]
#[altrep(class = "MyClass")]
pub struct MyData { value: i32, len: usize }

// 2. Implement required traits
impl AltrepLen for MyData {
    fn len(&self) -> usize { self.len }
}

impl AltIntegerData for MyData {
    fn elt(&self, _i: usize) -> i32 { self.value }
}

// 3. Generate low-level traits
miniextendr_api::impl_altinteger_from_data!(MyData);
```

In both cases, use `MyData { value: 42, len: 100 }.into_sexp()` to create the ALTREP vector.

## Optional Methods

| Feature | Trait Method | Macro Option | When to Use |
|---------|--------------|--------------|-------------|
| **Bulk access** | `get_region()` | - | Faster than looping `elt()` |
| **Dataptr** | `AltrepDataptr<T>` | `dataptr` | Data in contiguous memory |
| **Serialization** | `AltrepSerialize` | `serialize` | Save/load support needed |
| **Subsetting** | `extract_subset()` | `subset` | O(1) subset possible |
| **Mutation** | `set_elt()` | `set_elt` | Mutable String/List |
| **NA hint** | `no_na()` | - | Enables optimizations |
| **Sorted hint** | `is_sorted()` | - | Enables optimizations |
| **Sum** | `sum()` | - | O(1) computation possible |
| **Min** | `min()` | - | O(1) computation possible |
| **Max** | `max()` | - | O(1) computation possible |

## Derive Syntax

### Field-based derive (generates everything)

```rust
#[derive(miniextendr_api::AltrepInteger)]
#[altrep(len = "len_field", elt = "value_field", class = "ClassName")]
pub struct MyType { value_field: i32, len_field: usize }

// Optional: dataptr, serialize, subset via #[altrep(...)] options
#[derive(miniextendr_api::AltrepInteger)]
#[altrep(len = "len", elt_delegate = "data", dataptr, class = "DelegateType")]
pub struct MyType2 { data: Vec<i32>, len: usize }
```

### Manual traits macro syntax

```rust
// Basic (elt only)
impl_altinteger_from_data!(MyType);

// With dataptr
impl_altinteger_from_data!(MyType, dataptr);

// With serialization
impl_altinteger_from_data!(MyType, serialize);

// Multiple options
impl_altinteger_from_data!(MyType, dataptr, serialize, subset, set_elt);
```

## Available Macros

- `impl_altinteger_from_data!()` - Integer vectors
- `impl_altreal_from_data!()` - Real (numeric) vectors
- `impl_altlogical_from_data!()` - Logical vectors
- `impl_altraw_from_data!()` - Raw (byte) vectors
- `impl_altcomplex_from_data!()` - Complex vectors
- `impl_altstring_from_data!()` - String (character) vectors
- `impl_altlist_from_data!()` - List vectors

## Sortedness Values

```rust
use miniextendr_api::altrep_data::Sortedness;

fn is_sorted(&self) -> Option<Sortedness> {
    Some(Sortedness::Increasing)       // 1, 2, 3, ...
    Some(Sortedness::Decreasing)       // 3, 2, 1, ...
    Some(Sortedness::IncreasingNAsFirst)  // NA, 1, 2, 3
    Some(Sortedness::DecreasingNAsFirst)  // NA, 3, 2, 1
    Some(Sortedness::Unsorted)         // No pattern
    None  // Unknown (R will check if needed)
}
```

## Logical Values

```rust
use miniextendr_api::altrep_data::Logical;

impl AltLogicalData for MyData {
    fn elt(&self, i: usize) -> Logical {
        Logical::True      // TRUE (1)
        Logical::False     // FALSE (0)
        Logical::Na        // NA
    }
}
```

## Common Patterns

### Constant Vector
```rust
fn elt(&self, _i: usize) -> T { self.constant_value }
```

### Arithmetic Sequence
```rust
fn elt(&self, i: usize) -> T { self.start + i * self.step }
```

### External Data
```rust
fn elt(&self, i: usize) -> T { self.source[self.offset + i] }
```

### Computed
```rust
fn elt(&self, i: usize) -> T { (self.function)(i) }
```

## DATAPTR Decision Tree

```text
Do you have data in memory?
├─ Yes (Vec/Box/slice)
│  └─ Provide dataptr() → Fast operations
│
└─ No (computed/lazy)
   └─ Will users need DATAPTR often?
      ├─ Yes → Implement with cache
      └─ No → Skip dataptr()
```

## Debugging Checklist

- [ ] Function is `pub`?
- [ ] Data struct has `#[derive(AltrepInteger)]` (or `#[derive(Altrep)]` for manual pattern)?
- [ ] `#[altrep(class = "...")]` attribute present?
- [ ] For manual pattern: `impl AltrepLen` provided?
- [ ] For manual pattern: `impl Alt*Data` provided?
- [ ] For manual pattern: used correct `impl_alt*_from_data!` macro?
- [ ] Ran `devtools::document()`?
- [ ] Reinstalled package?

## Performance Checklist

- [ ] Provide `no_na()` if no NAs → Faster operations
- [ ] Provide `is_sorted()` if sorted → Faster unique/sort
- [ ] Provide `sum()/min()/max()` if O(1) possible → Much faster
- [ ] Provide `get_region()` if faster than looping → Faster bulk access
- [ ] Provide `extract_subset()` if O(1) subset → Faster subsetting
- [ ] Skip `dataptr()` if lazy → Save memory
- [ ] Implement `dataptr()` if in-memory → Faster operations

## Safety Checklist

- [ ] ALTREP derive (`#[derive(Altrep)]` or `#[derive(AltrepInteger)]` etc.) on data struct
- [ ] No raw pointers outliving their source
- [ ] `RefCell` (not Mutex) for interior mutability
- [ ] Returned pointers valid for object lifetime
- [ ] SEXPs in lists are protected
- [ ] Thread safety: no async/threading in callbacks

## Receiving ALTREP from R

R frequently passes ALTREP vectors to Rust (e.g., `1:10`, `seq_len(N)`).

| Rust parameter | ALTREP handling | Use when |
|---|---|---|
| `Vec<i32>`, `&[f64]`, etc. | Auto-materialized | You need the data |
| `SEXP` | Auto-materialized | You need the handle, not data |
| `AltrepSexp` | Not materialized, `!Send` | You need ALTREP metadata |
| `extern "C-unwind"` raw SEXP | No conversion at all | You inspect ALTREP state |

```rust
// Recommended: typed parameter handles everything
#[miniextendr]
pub fn sum_ints(x: Vec<i32>) -> i64 {
    x.iter().map(|&v| v as i64).sum()
}
// sum_ints(1:1000000)  ← works, ALTREP auto-materialized

// Explicit ALTREP handling
#[miniextendr]
pub fn altrep_len(x: AltrepSexp) -> usize { x.len() }
// altrep_len(1:10)      ← works
// altrep_len(c(1L, 2L)) ← error: not ALTREP
```

Full guide: [Receiving ALTREP from R](../altrep-sexp/)

## Further Reading

- **Complete guide**: [ALTREP.md](../altrep/)
- **Receiving ALTREP**: [ALTREP_SEXP.md](../altrep-sexp/)
- **Practical examples**: [ALTREP_EXAMPLES.md](../altrep-examples/)
- **Test suite**: [../rpkg/tests/testthat/test-altrep*.R](../rpkg/tests/testthat/)
- **Reference**: [R Altrep.h](../background/r-source-tags-R-4-5-2/src/include/R_ext/Altrep.h)
