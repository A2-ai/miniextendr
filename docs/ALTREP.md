# ALTREP in miniextendr

ALTREP (Alternative Representations) is R's system for creating custom vector implementations. miniextendr provides a powerful, safe abstraction for creating ALTREP vectors from Rust.

## What is ALTREP?

ALTREP allows you to create R vectors with custom internal representations. Instead of storing data in R's native format, you can:

- **Compute elements on demand** (lazy sequences)
- **Reference external data** without copying (zero-copy views)
- **Use compact representations** (constant vectors, arithmetic sequences)
- **Provide optimized operations** (O(1) sum for arithmetic sequences)

## Quick Start

Here's a minimal ALTREP example - a constant integer vector:

```rust
use miniextendr_api::{miniextendr, ffi::SEXP, IntoR};
use miniextendr_api::altrep_data::{AltrepLen, AltIntegerData};

// 1. Define your data type
#[derive(miniextendr_api::ExternalPtr)]
pub struct ConstantIntData {
    value: i32,
    len: usize,
}

// 2. Implement required traits
impl AltrepLen for ConstantIntData {
    fn len(&self) -> usize {
        self.len
    }
}

impl AltIntegerData for ConstantIntData {
    fn elt(&self, _i: usize) -> i32 {
        self.value  // All elements are the same
    }
}

// 3. Generate low-level trait implementations
miniextendr_api::impl_altinteger_from_data!(ConstantIntData);

// 4. Create the ALTREP wrapper struct
#[miniextendr(class = "ConstantInt", pkg = "mypkg")]
pub struct ConstantInt(pub ConstantIntData);

// 5. Export a constructor
#[miniextendr]
pub fn constant_int(value: i32, n: i32) -> SEXP {
    let data = ConstantIntData { value, len: n as usize };
    ConstantInt(data).into_sexp()
}
```

Usage in R:
```r
x <- constant_int(42L, 1000000L)  # Creates 1M-element vector using O(1) memory
x[1]     # 42
x[500]   # 42
sum(x)   # 42000000 (uses default R sum)
```

---

## Architecture Overview

miniextendr's ALTREP system has three layers:

```
┌─────────────────────────────────────────────────────────────────┐
│  Layer 3: ALTREP Wrapper Struct                                 │
│  #[miniextendr(class = "...", pkg = "...")]                     │
│  Registers class with R, handles SEXP conversion                │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│  Layer 2: Low-Level Traits (auto-generated)                     │
│  impl_altinteger_from_data!(MyType)                             │
│  Implements Altrep, AltVec, AltInteger traits                   │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│  Layer 1: High-Level Data Traits (you implement)                │
│  AltrepLen, AltIntegerData, AltRealData, etc.                   │
│  Safe, idiomatic Rust - no raw SEXP handling                    │
└─────────────────────────────────────────────────────────────────┘
```

---

## High-Level Data Traits

### Core Trait: `AltrepLen`

Every ALTREP type must implement `AltrepLen`:

```rust
impl AltrepLen for MyData {
    fn len(&self) -> usize {
        // Return the vector length
    }
}
```

### Type-Specific Traits

| R Vector Type | Rust Trait | Required Method |
|---------------|------------|-----------------|
| `integer` | `AltIntegerData` | `fn elt(&self, i: usize) -> i32` |
| `numeric` | `AltRealData` | `fn elt(&self, i: usize) -> f64` |
| `logical` | `AltLogicalData` | `fn elt(&self, i: usize) -> Logical` |
| `raw` | `AltRawData` | `fn elt(&self, i: usize) -> u8` |
| `character` | `AltStringData` | `fn elt(&self, i: usize) -> Option<&str>` |
| `complex` | `AltComplexData` | `fn elt(&self, i: usize) -> Rcomplex` |
| `list` | `AltListData` | `fn elt(&self, i: usize) -> SEXP` |

### Optional Methods

Each trait provides optional methods you can override:

```rust
impl AltIntegerData for MyData {
    fn elt(&self, i: usize) -> i32 {
        // Required: element access
    }

    fn no_na(&self) -> Option<bool> {
        // Optional: NA hint (enables optimizations)
        Some(true)  // No NAs in this vector
    }

    fn is_sorted(&self) -> Option<Sortedness> {
        // Optional: sortedness hint
        Some(Sortedness::Increasing)
    }

    fn sum(&self, na_rm: bool) -> Option<i64> {
        // Optional: O(1) sum (instead of element-by-element)
        Some(self.formula_based_sum())
    }

    fn min(&self, na_rm: bool) -> Option<i32> {
        // Optional: O(1) min
        Some(self.known_minimum())
    }

    fn max(&self, na_rm: bool) -> Option<i32> {
        // Optional: O(1) max
        Some(self.known_maximum())
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [i32]) -> usize {
        // Optional: bulk element access (can be more efficient)
        // Default uses elt() in a loop
    }
}
```

---

## Example: Arithmetic Sequence

A lazy arithmetic sequence that computes elements on demand:

```rust
#[derive(miniextendr_api::ExternalPtr)]
pub struct ArithSeqData {
    start: f64,
    step: f64,
    len: usize,
}

impl AltrepLen for ArithSeqData {
    fn len(&self) -> usize {
        self.len
    }
}

impl AltRealData for ArithSeqData {
    fn elt(&self, i: usize) -> f64 {
        self.start + (i as f64) * self.step
    }

    fn no_na(&self) -> Option<bool> {
        Some(true)  // Arithmetic sequences never produce NA
    }

    fn is_sorted(&self) -> Option<Sortedness> {
        if self.step < 0.0 {
            Some(Sortedness::Decreasing)
        } else {
            Some(Sortedness::Increasing)
        }
    }

    fn sum(&self, _na_rm: bool) -> Option<f64> {
        // O(1) sum using arithmetic series formula: n*(first+last)/2
        let last = self.start + (self.len - 1) as f64 * self.step;
        Some(self.len as f64 * (self.start + last) / 2.0)
    }
}

miniextendr_api::impl_altreal_from_data!(ArithSeqData);

#[miniextendr(class = "ArithSeq", pkg = "mypkg")]
pub struct ArithSeq(pub ArithSeqData);

#[miniextendr]
pub fn arith_seq(from: f64, to: f64, length_out: i32) -> SEXP {
    let len = length_out as usize;
    let step = if len > 1 { (to - from) / (len - 1) as f64 } else { 0.0 };
    ArithSeq(ArithSeqData { start: from, step, len }).into_sexp()
}
```

---

## Lazy Materialization

For cases where you want lazy computation but also need to support `DATAPTR`:

```rust
use miniextendr_api::altrep_data::AltrepDataptr;

#[derive(miniextendr_api::ExternalPtr)]
pub struct LazyIntSeqData {
    start: i32,
    step: i32,
    len: usize,
    materialized: Option<Vec<i32>>,  // Lazily allocated
}

impl AltrepLen for LazyIntSeqData {
    fn len(&self) -> usize { self.len }
}

impl AltIntegerData for LazyIntSeqData {
    fn elt(&self, i: usize) -> i32 {
        // Compute on-the-fly (no materialization needed)
        self.start.saturating_add((i as i32).saturating_mul(self.step))
    }
}

impl AltrepDataptr<i32> for LazyIntSeqData {
    fn dataptr(&mut self, _writable: bool) -> Option<*mut i32> {
        // Materialize on first DATAPTR access
        if self.materialized.is_none() {
            let data: Vec<i32> = (0..self.len)
                .map(|i| self.start.saturating_add((i as i32).saturating_mul(self.step)))
                .collect();
            self.materialized = Some(data);
        }
        self.materialized.as_mut().map(|v| v.as_mut_ptr())
    }

    fn dataptr_or_null(&self) -> Option<*const i32> {
        // Return pointer only if already materialized
        // Returning None tells R to use elt() instead
        self.materialized.as_ref().map(|v| v.as_ptr())
    }
}

// Enable dataptr support in macro
miniextendr_api::impl_altinteger_from_data!(LazyIntSeqData, dataptr);
```

**Key behaviors:**
- `elt()` always works, no allocation needed
- `dataptr_or_null()` returns `None` until materialized
- `dataptr()` allocates on first call, caches result
- Operations like `x + y` trigger `dataptr()`, causing materialization

---

## Serialization Support

To make ALTREP objects serializable (for `saveRDS`/`readRDS`):

```rust
use miniextendr_api::altrep_data::AltrepSerialize;

impl AltrepSerialize for LazyIntSeqData {
    fn serialized_state(&self) -> SEXP {
        // Return a serializable representation (typically a simple vector)
        unsafe {
            use miniextendr_api::ffi::{Rf_allocVector, SET_INTEGER_ELT, SEXPTYPE};
            let state = Rf_allocVector(SEXPTYPE::INTSXP, 3);
            SET_INTEGER_ELT(state, 0, self.start);
            SET_INTEGER_ELT(state, 1, self.step);
            SET_INTEGER_ELT(state, 2, self.len as i32);
            state
        }
    }

    fn unserialize(state: SEXP) -> Option<Self> {
        unsafe {
            use miniextendr_api::ffi::INTEGER_ELT;
            Some(LazyIntSeqData {
                start: INTEGER_ELT(state, 0),
                step: INTEGER_ELT(state, 1),
                len: INTEGER_ELT(state, 2) as usize,
                materialized: None,  // Fresh - not materialized
            })
        }
    }
}

// Enable serialization in macro
miniextendr_api::impl_altinteger_from_data!(LazyIntSeqData, dataptr, serialize);
```

---

## Standard Type Support

miniextendr provides built-in ALTREP support for common Rust types:

### `Vec<T>` (Owned Data)

```rust
// Vec<i32> has built-in ALTREP traits
#[miniextendr(class = "SimpleVecInt", pkg = "mypkg")]
pub struct SimpleVecInt(pub Vec<i32>);

#[miniextendr]
pub fn simple_vec_int(values: Vec<i32>) -> SEXP {
    SimpleVecInt(values).into_sexp()
}
```

### `Box<[T]>` (Immutable Owned Slice)

```rust
#[miniextendr(class = "BoxedInts", pkg = "mypkg")]
pub struct BoxedInts(pub Box<[i32]>);

#[miniextendr]
pub fn boxed_ints(n: i32) -> SEXP {
    let data: Box<[i32]> = (1..=n).collect();
    BoxedInts(data).into_sexp()
}
```

### Static Slices (`&'static [T]`)

```rust
static MY_DATA: &[i32] = &[10, 20, 30, 40, 50];

#[miniextendr(class = "StaticInts", pkg = "mypkg")]
pub struct StaticInts(pub &'static [i32]);

#[miniextendr]
pub fn static_ints() -> SEXP {
    StaticInts(MY_DATA).into_sexp()
}
```

**Note:** Static ALTREPs are read-only and cannot support writable DATAPTR.

---

## Complex Numbers

```rust
use miniextendr_api::ffi::Rcomplex;
use miniextendr_api::altrep_data::AltComplexData;

#[derive(miniextendr_api::ExternalPtr)]
pub struct UnitCircleData {
    n: usize,  // Number of points on unit circle
}

impl AltrepLen for UnitCircleData {
    fn len(&self) -> usize { self.n }
}

impl AltComplexData for UnitCircleData {
    fn elt(&self, i: usize) -> Rcomplex {
        let theta = 2.0 * std::f64::consts::PI * (i as f64) / (self.n as f64);
        Rcomplex { r: theta.cos(), i: theta.sin() }
    }
}

miniextendr_api::impl_altcomplex_from_data!(UnitCircleData);

#[miniextendr(class = "UnitCircle", pkg = "mypkg")]
pub struct UnitCircle(pub UnitCircleData);

#[miniextendr]
pub fn unit_circle(n: i32) -> SEXP {
    UnitCircle(UnitCircleData { n: n as usize }).into_sexp()
}
```

---

## Logical Vectors

Use the `Logical` enum for proper NA handling:

```rust
use miniextendr_api::altrep_data::{AltLogicalData, Logical};

#[derive(miniextendr_api::ExternalPtr)]
pub struct LogicalVecData {
    data: Vec<Logical>,
}

impl AltrepLen for LogicalVecData {
    fn len(&self) -> usize { self.data.len() }
}

impl AltLogicalData for LogicalVecData {
    fn elt(&self, i: usize) -> Logical {
        self.data[i]
    }

    fn no_na(&self) -> Option<bool> {
        Some(!self.data.iter().any(|v| matches!(v, Logical::Na)))
    }

    fn sum(&self, na_rm: bool) -> Option<i64> {
        // Sum = count of TRUE values
        let mut total = 0i64;
        for v in &self.data {
            match v {
                Logical::True => total += 1,
                Logical::False => {}
                Logical::Na => if !na_rm { return None; }
            }
        }
        Some(total)
    }
}

miniextendr_api::impl_altlogical_from_data!(LogicalVecData);
```

---

## String Vectors

String ALTREPs return `Option<&str>` where `None` represents `NA`:

```rust
use miniextendr_api::altrep_data::AltStringData;

#[derive(miniextendr_api::ExternalPtr)]
pub struct StringVecData {
    data: Vec<Option<String>>,
}

impl AltrepLen for StringVecData {
    fn len(&self) -> usize { self.data.len() }
}

impl AltStringData for StringVecData {
    fn elt(&self, i: usize) -> Option<&str> {
        self.data[i].as_deref()  // None = NA
    }

    fn no_na(&self) -> Option<bool> {
        Some(!self.data.iter().any(|v| v.is_none()))
    }
}

miniextendr_api::impl_altstring_from_data!(StringVecData);
```

---

## Raw Vectors

```rust
use miniextendr_api::altrep_data::AltRawData;

#[derive(miniextendr_api::ExternalPtr)]
pub struct RepeatingRawData {
    pattern: Vec<u8>,
    total_len: usize,
}

impl AltrepLen for RepeatingRawData {
    fn len(&self) -> usize { self.total_len }
}

impl AltRawData for RepeatingRawData {
    fn elt(&self, i: usize) -> u8 {
        self.pattern[i % self.pattern.len()]
    }
}

miniextendr_api::impl_altraw_from_data!(RepeatingRawData);
```

---

## Reference Types

When you need to pass an ALTREP back to Rust functions:

```rust
// The #[miniextendr] macro generates these automatically:
// - ConstantIntRef: immutable reference to ALTREP data
// - ConstantIntMut: mutable reference to ALTREP data

#[miniextendr]
pub fn inspect_constant_int(x: ConstantIntRef) -> String {
    format!("value={}, len={}", x.value, x.len)
}

#[miniextendr]
pub fn double_constant_int(mut x: ConstantIntMut) {
    x.value *= 2;
}
```

---

## Low-Level Trait Macros

The `impl_alt*_from_data!` macros accept options:

```rust
// Basic (element access only)
miniextendr_api::impl_altinteger_from_data!(MyType);

// With dataptr support (enables DATAPTR method)
miniextendr_api::impl_altinteger_from_data!(MyType, dataptr);

// With serialization (enables saveRDS/readRDS)
miniextendr_api::impl_altinteger_from_data!(MyType, serialize);

// With subset optimization (enables optimized x[i] for index vectors)
miniextendr_api::impl_altinteger_from_data!(MyType, subset);

// Multiple options
miniextendr_api::impl_altinteger_from_data!(MyType, dataptr, serialize, subset);
```

| Option | What it does | Requires |
|--------|--------------|----------|
| `dataptr` | Enables `DATAPTR` method | `impl AltrepDataptr<T>` |
| `serialize` | Enables serialization | `impl AltrepSerialize` |
| `subset` | Enables optimized subsetting | `impl AltrepSubset` |

---

## Sortedness and NA Hints

Providing hints enables R to optimize operations:

```rust
use miniextendr_api::altrep_data::Sortedness;

impl AltIntegerData for MyData {
    fn is_sorted(&self) -> Option<Sortedness> {
        match self.ordering {
            Ordering::Ascending => Some(Sortedness::Increasing),
            Ordering::Descending => Some(Sortedness::Decreasing),
            Ordering::Unknown => None,  // Don't know
        }
    }

    fn no_na(&self) -> Option<bool> {
        Some(true)  // Enables R to skip NA checks
    }
}
```

---

## Performance Tips

1. **Implement `sum`/`min`/`max`** when you can compute them in O(1)
2. **Use `no_na()` hint** when you know there are no NAs
3. **Use `is_sorted()` hint** for sorted data
4. **Implement `get_region()`** for efficient bulk access
5. **Delay materialization** - prefer `elt()` over `dataptr()`
6. **Return `None` from `dataptr_or_null()`** until actually materialized

---

## Common Patterns

### Pattern 1: Constant Vector

```rust
struct Constant<T> { value: T, len: usize }
// All elements return the same value
fn elt(&self, _i: usize) -> T { self.value }
```

### Pattern 2: Computed Sequence

```rust
struct Sequence { start: T, step: T, len: usize }
// Elements computed from formula
fn elt(&self, i: usize) -> T { self.start + i * self.step }
```

### Pattern 3: External Data View

```rust
struct ExternalView<'a> { data: &'a [T] }
// Zero-copy view into external data
fn elt(&self, i: usize) -> T { self.data[i] }
```

### Pattern 4: Lazy Computation with Cache

```rust
struct Lazy { params: Params, cache: Option<Vec<T>> }
// Compute and cache on first access
fn dataptr(&mut self) -> *mut T {
    if self.cache.is_none() { self.cache = Some(self.compute()); }
    self.cache.as_mut().unwrap().as_mut_ptr()
}
```

---

## Module Registration

Register your ALTREP types in `miniextendr_module!`:

```rust
miniextendr_module! {
    mod mypkg;

    // Constructor functions
    fn constant_int;
    fn arith_seq;
    fn unit_circle;

    // ALTREP classes are registered automatically via #[miniextendr] attribute
}
```

---

## Troubleshooting

### "Error: could not find function"
- Ensure constructor function is listed in `miniextendr_module!`
- Run `just devtools-document` after adding new functions

### Elements return wrong values
- Check your `elt()` implementation
- Verify index bounds handling

### R crashes on access
- Ensure `ExternalPtr` derive is on your data type
- Check that `into_sexp()` is called to create the ALTREP object

### Serialization fails
- Implement `AltrepSerialize` trait
- Add `serialize` option to `impl_alt*_from_data!`

### DATAPTR operations crash
- Implement `AltrepDataptr` trait
- Add `dataptr` option to `impl_alt*_from_data!`
- Ensure returned pointer is valid for the vector's lifetime

---

## Iterator-Backed ALTREP

miniextendr provides two iterator-backed ALTREP variants:

### `IterState` (Prefix Caching)

The default iterator state caches elements as a contiguous prefix. When you access element `i`, all elements `0..=i` are generated and cached.

```rust
use miniextendr_api::altrep_data::IterIntData;

// Create from an iterator
let data = IterIntData::from_iter((0..1000).map(|x| x * 2), 1000);

// Access element 100 - generates and caches elements 0-100
let elem = data.elt(100);

// Access element 50 - returns from cache (no computation)
let elem = data.elt(50);
```

**Characteristics:**
- Cache is contiguous `Vec<T>`
- All elements up to max accessed index are cached
- `as_slice()` available after full materialization
- Memory usage: O(max_accessed_index)

### `SparseIterState` (Skipping)

For sparse access patterns, use the sparse variants that skip intermediate elements using `Iterator::nth()`:

```rust
use miniextendr_api::altrep_data::SparseIterIntData;

// Create from an iterator
let data = SparseIterIntData::from_iter((0..1_000_000).map(|x| x * 2), 1_000_000);

// Access element 999_999 - skips directly there
let elem = data.elt(999_999);  // Only this element is generated

// Element 0 was skipped and is now inaccessible
let first = data.elt(0);  // Returns NA_INTEGER
```

**Characteristics:**
- Cache is sparse `BTreeMap<usize, T>`
- Only accessed elements are cached
- Skipped elements return NA/default forever
- `as_slice()` always returns `None`
- Memory usage: O(num_accessed)

### Comparison

| Feature | `IterState` | `SparseIterState` |
|---------|-------------|-------------------|
| Cache storage | Contiguous `Vec<T>` | Sparse `BTreeMap<usize, T>` |
| Access pattern | Prefix (0..=i) cached | Only accessed indices cached |
| Skipped elements | All cached | Gone forever (return NA) |
| Memory for sparse access | O(max_index) | O(num_accessed) |
| `as_slice()` support | Yes (after full materialization) | No |

### Available Types

**Prefix caching (`IterState`):**
- `IterIntData<I>` - Integer vectors
- `IterRealData<I>` - Real (f64) vectors
- `IterLogicalData<I>` - Logical (bool) vectors
- `IterRawData<I>` - Raw (u8) vectors
- `IterStringData<I>` - Character vectors (forces full materialization)
- `IterComplexData<I>` - Complex number vectors
- `IterListData<I>` - List vectors (SEXP elements)
- `IterIntCoerceData<I, T>` - Integer with coercion from other types
- `IterRealCoerceData<I, T>` - Real with coercion from other types

**Sparse/skipping (`SparseIterState`):**
- `SparseIterIntData<I>` - Integer vectors
- `SparseIterRealData<I>` - Real (f64) vectors
- `SparseIterLogicalData<I>` - Logical (bool) vectors
- `SparseIterRawData<I>` - Raw (u8) vectors
- `SparseIterComplexData<I>` - Complex number vectors

### When to Use Which

**Use `IterState` (prefix caching) when:**
- Access is mostly sequential (0, 1, 2, ...)
- You'll eventually access most/all elements
- You need `as_slice()` or full materialization later

**Use `SparseIterState` (skipping) when:**
- Access is truly sparse (e.g., sampling)
- Vector is very large but you only need a few elements
- You don't need skipped elements ever again
- Memory is constrained
