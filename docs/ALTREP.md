# ALTREP in miniextendr

ALTREP (Alternative Representations) is R's system for creating custom vector implementations. miniextendr provides a powerful, safe abstraction for creating ALTREP vectors from Rust.

**Additional Resources**:
- **[Quick Reference](ALTREP_QUICKREF.md)** - One-page cheat sheet
- **[Receiving ALTREP from R](ALTREP_SEXP.md)** - How `SEXP` and `AltrepSexp` parameters handle ALTREP input
- **[Practical Examples](ALTREP_EXAMPLES.md)** - Real-world use cases
- **[Test Suite](../rpkg/tests/testthat/test-altrep*.R)** - Working examples

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

## Choosing ALTREP vs Regular Conversion

miniextendr offers two conversion paths for Rust data:

### Regular Conversion (IntoR) - Copy to R

```rust
#[miniextendr]
fn get_data() -> Vec<i32> {
    vec![1, 2, 3, 4, 5]
}
// Or explicitly: vec.into_sexp()
```

**Behavior**:
- Data is copied to R's heap
- Original Vec is dropped
- R owns a regular integer vector (INTSXP)
- O(n) memory copy, O(n) memory allocation

**Best for**:
- Small data (<1000 elements)
- Data R will modify
- Temporary results
- When simplicity matters

### ALTREP Conversion (IntoRAltrep) - Zero-Copy

```rust
use miniextendr_api::IntoRAltrep;

#[miniextendr]
fn get_data() -> SEXP {
    let vec = vec![1, 2, 3, 4, 5];
    vec.into_sexp_altrep()
}
// Or: Altrep(vec).into_sexp()
```

**Behavior**:
- Data stays in Rust (ExternalPtr wrapper)
- No copying, no duplication
- R accesses via ALTREP callbacks
- O(1) creation, ~10ns per element overhead

**Best for**:
- Large vectors (>1000 elements)
- Lazy evaluation (compute on access)
- External data (files, APIs, databases)
- Zero-copy requirements

### Performance Comparison (Measured)

**Pure Creation (No Access)**:
| Size | Copy | ALTREP | Speedup |
|------|------|--------|---------|
| 100 | 0.33 ms | 0.42 ms | 0.8x (copy faster) |
| 1,000 | 0.43 ms | 0.50 ms | 0.9x (similar) |
| 100,000 | 0.44 ms | 0.42 ms | 1.0x (similar) |
| 1,000,000 | 0.44 ms | 0.20 ms | **2.2x faster** |
| 10,000,000 | 4.16 ms | 1.90 ms | **2.2x faster** |

**Partial Access (Create 1M, Access First 10)**:
| Size | Copy | ALTREP | Speedup |
|------|------|--------|---------|
| 10,000 | 0.02 ms | 0.02 ms | 1.0x |
| 100,000 | 0.06 ms | 0.02 ms | **3.0x faster** |
| 1,000,000 | 0.42 ms | 0.20 ms | **2.1x faster** |
| 10,000,000 | 4.28 ms | 0.08 ms | **53.5x faster** |

**Memory**:
- Copy (1M elements): R heap +3.8 MB
- ALTREP (1M elements): R heap +0.0 MB (data in Rust heap)

*Benchmarks run on Apple M-series, R 4.5. Your results may vary.*

### Decision Guide

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
use miniextendr_api::{miniextendr, IntoRAltrep, ffi::SEXP};

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
    (0..n as usize)
        .map(|i| fibonacci(i))
        .collect::<Vec<i32>>()
        .into_sexp_altrep()
}

// Range - already lazy, use ALTREP
#[miniextendr]
fn int_range(from: i32, to: i32) -> SEXP {
    (from..to)
        .collect::<Vec<_>>()
        .into_sexp_altrep()
}
```

### Migration from `Altrep(...)` to `.into_sexp_altrep()`

Both forms are equivalent and compile to identical code:

```rust
// Old style (still works!)
return Altrep(vec).into_sexp();

// New style (more explicit)
return vec.into_sexp_altrep();

// Both are valid - use whichever is clearer
```

---

## Architecture Overview

miniextendr's ALTREP system has three layers:

```text
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

## Class Registration and Cross-Session readRDS

For `saveRDS`/`readRDS` to work across R sessions, R must be able to find the
ALTREP class by name when unserializing. This requires two things:

### 1. DllInfo — associating the class with a package

`R_make_alt*_class(class_name, pkg_name, dll_info)` takes a `DllInfo*` that
tells R which package owns the class. The serialized stream stores the class
name and package name. On `readRDS`, R looks up the class by
`(class_name, pkg_name)` — this lookup requires the DllInfo to have been
provided at registration time.

miniextendr stores the DllInfo from `package_init` in a global and passes it
to all `R_make_alt*_class` calls:

```rust
// In init.rs — during R_init_<pkg>:
crate::set_altrep_dll_info(dll);

// In __impl_inferbase macro — when creating the class:
let dll = $crate::altrep_dll_info();
let cls = R_make_altreal_class(class_name, pkg_name, dll);
```

Without DllInfo (`NULL`), R can't find the class during deserialization, even
if it's registered. This was a bug — all classes were registered with `NULL`.

### 2. Eager registration — classes must exist before readRDS runs

ALTREP classes are registered in two ways:

**Proc-macro-generated classes** (user `#[miniextendr]` structs) register via
linkme's `#[distributed_slice]`. Each ALTREP struct emits an entry that's
called during `R_init`:

```rust
// Generated by proc-macro:
#[distributed_slice(MX_ALTREP_REGISTRATIONS)]
fn register_my_class() {
    MyType::get_or_init_class();
}

// Called during R_init:
for reg_fn in MX_ALTREP_REGISTRATIONS.iter() {
    reg_fn();
}
```

**Built-in classes** (`Vec<f64>`, `Box<[i32]>`, Arrow arrays, etc.) use
`OnceLock` inside `RegisterAltrep::get_or_init_class()`. These are lazy —
the class is created on first use (e.g., when `.into_sexp_altrep()` is
called). This is a problem for `readRDS`: R tries to find the class *during
deserialization*, before any miniextendr code has called `into_sexp_altrep`.

The fix: `register_builtin_altrep_classes()` is called during `R_init` and
eagerly calls `get_or_init_class()` for every built-in type:

```rust
// In registry.rs — during R_init:
register_builtin_altrep_classes();  // Vec, Box, Range
#[cfg(feature = "arrow")]
register_arrow_altrep_classes();    // Float64Array, Int32Array, etc.
```

```rust
pub(crate) fn register_builtin_altrep_classes() {
    use crate::altrep::RegisterAltrep;
    Vec::<i32>::get_or_init_class();
    Vec::<f64>::get_or_init_class();
    Vec::<bool>::get_or_init_class();
    Vec::<u8>::get_or_init_class();
    Vec::<String>::get_or_init_class();
    Vec::<Option<String>>::get_or_init_class();
    // ... all built-in types
}
```

### What happens during readRDS

```text
Session A: saveRDS(altrep_vec, "data.rds")
  → ALTREP serialize hook fires
  → serialized_state() materializes data to plain R vector
  → Stream contains: class_name="miniextendr_Vec_f64", pkg_name="miniextendr", state=<REALSXP>

Session B: library(miniextendr); readRDS("data.rds")
  → R_init_miniextendr runs → registers all ALTREP classes (with DllInfo)
  → readRDS parses stream → finds class "miniextendr_Vec_f64" in package "miniextendr"
  → R calls unserialize(class, state) → reconstructs Vec<f64> from the REALSXP
  → Returns a live ALTREP vector backed by Rust data

Session C: readRDS("data.rds")  # WITHOUT library(miniextendr)
  → ALTREP class not registered → R falls back to the serialized state
  → Returns a plain R numeric vector (the materialized data)
  → Works correctly — just not an ALTREP anymore
```

### Adding serialization to new types

When you add a new `impl_alt*_from_data!` with `serialize`:

1. Implement `AltrepSerialize` for the type
2. Add the `serialize` option: `impl_altreal_from_data!(MyType, serialize);`
3. If it's a built-in type (in miniextendr-api, not user code), add it to
   `register_builtin_altrep_classes()` so it's eagerly registered at init

User types don't need step 3 — the proc-macro generates `#[distributed_slice]`
entries automatically.

---

## Mutable Vectors (Set_elt)

String and List vectors can be made mutable by implementing the `set_elt()` method. This allows R code to modify elements in-place.

**Important**: Only String and List types support `set_elt`. Numeric vectors (Integer, Real, Logical, Raw, Complex) cannot be mutated through ALTREP.

### Mutable String Vectors

```rust
use miniextendr_api::altrep_data::{AltrepLen, AltStringData};
use miniextendr_api::ffi::SEXP;
use std::cell::RefCell;

#[derive(miniextendr_api::ExternalPtr)]
pub struct MutableStringData {
    strings: RefCell<Vec<Option<String>>>,
}

impl AltrepLen for MutableStringData {
    fn len(&self) -> usize {
        self.strings.borrow().len()
    }
}

impl AltStringData for MutableStringData {
    fn elt(&self, i: usize) -> Option<&str> {
        // SAFETY: This is unsafe - we're returning a reference into RefCell
        // In practice, you'd need to use a different strategy (e.g., cache in thread-local)
        // or return owned String and convert to SEXP
        unsafe {
            let ptr = self.strings.as_ptr();
            (*ptr).get(i).and_then(|s| s.as_deref())
        }
    }

    // Enable mutation
    fn set_elt(&mut self, i: usize, value: Option<&str>) {
        if let Some(s) = self.strings.get_mut().get_mut(i) {
            *s = value.map(|v| v.to_string());
        }
    }
}

miniextendr_api::impl_altstring_from_data!(MutableStringData, set_elt);
```

**Note**: The above example shows the concept but has lifetime issues. For production use, consider:
- Storing SEXPs directly instead of Rust strings
- Using thread-local storage for temporary string references
- Materializing to a regular R vector when mutations occur

### Mutable List Vectors

Lists are easier to make mutable since they already store SEXPs:

```rust
use miniextendr_api::altrep_data::{AltrepLen, AltListData};
use miniextendr_api::ffi::SEXP;
use std::cell::RefCell;

#[derive(miniextendr_api::ExternalPtr)]
pub struct MutableListData {
    // SEXPs need to be protected from GC
    elements: RefCell<Vec<SEXP>>,
}

impl AltrepLen for MutableListData {
    fn len(&self) -> usize {
        self.elements.borrow().len()
    }
}

impl AltListData for MutableListData {
    fn elt(&self, i: usize) -> SEXP {
        self.elements.borrow()[i]
    }

    fn set_elt(&mut self, i: usize, value: SEXP) {
        self.elements.borrow_mut()[i] = value;
    }
}

miniextendr_api::impl_altlist_from_data!(MutableListData, set_elt);
```

### Safety Considerations

**1. R's Copy-on-Write**: R may copy your vector before calling `set_elt`, so mutations may not affect the original vector reference.

**2. GC Protection**: When storing SEXPs in mutable lists:
   - SEXPs in the ALTREP data slot are automatically protected
   - If you create new SEXPs, ensure they're returned to R immediately
   - Don't store raw SEXP pointers that outlive their protection

**3. Thread Safety**:
   - ALTREP callbacks run on R's main thread
   - Use `RefCell` (not `Mutex`) for interior mutability
   - No async/threading allowed inside ALTREP methods

**4. Materialization**:
   - R may materialize (copy to regular vector) when it needs a `dataptr`
   - After materialization, mutations go to the copy, not your ALTREP

### When to Use Mutable ALTREP

**Good use cases**:
- Lazy evaluation with caching
- Proxying to external mutable data sources
- Implementing special data structures (e.g., sparse vectors)

**Avoid for**:
- Regular data storage (use `Vec<T>` instead)
- Situations where you need `dataptr` (forces materialization)
- Performance-critical code (mutations have overhead)

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

## List Vectors

List vectors (R's `list` type / VECSXP) can contain any R objects. The `AltListData` trait allows you to create lists that compute or fetch elements on demand.

```rust
use miniextendr_api::altrep_data::{AltrepLen, AltListData};
use miniextendr_api::ffi::SEXP;
use miniextendr_api::{IntoR, Rf_ScalarInteger};

#[derive(miniextendr_api::ExternalPtr)]
pub struct IntegerSequenceList {
    n: usize,  // Number of elements in the list
}

impl AltrepLen for IntegerSequenceList {
    fn len(&self) -> usize {
        self.n
    }
}

impl AltListData for IntegerSequenceList {
    fn elt(&self, i: usize) -> SEXP {
        // Each element is a scalar integer equal to its index
        unsafe { Rf_ScalarInteger((i + 1) as i32) }
    }
}

miniextendr_api::impl_altlist_from_data!(IntegerSequenceList);

#[miniextendr(class = "IntegerSequenceList", pkg = "mypkg")]
pub struct IntSeqList(pub IntegerSequenceList);

#[miniextendr]
pub fn int_seq_list(n: i32) -> SEXP {
    let data = IntegerSequenceList { n: n as usize };
    IntSeqList(data).into_sexp()
}
```

Usage in R:
```r
lst <- int_seq_list(5L)
length(lst)  # 5
lst[[1]]     # 1L
lst[[3]]     # 3L
lst[[5]]     # 5L
```

### List Safety Considerations

**Important**: List elements are SEXPs that must be properly protected from garbage collection. When implementing `AltListData::elt()`:

1. **Return existing SEXPs**: If you store SEXPs in your data structure, they're already protected by being in the ALTREP object's data slot
2. **Create new SEXPs**: If you create SEXPs on-the-fly (like `Rf_ScalarInteger`), R will protect them when they're added to the list
3. **Avoid raw pointers**: Don't store raw SEXP pointers that might become invalid

### Practical List Examples

**Example 1: Repeating Element**
```rust
#[derive(miniextendr_api::ExternalPtr)]
pub struct RepeatedList {
    element: SEXP,  // Stored in data1 slot (protected)
    n: usize,
}

impl AltListData for RepeatedList {
    fn elt(&self, _i: usize) -> SEXP {
        self.element  // Same element for all indices
    }
}
```

**Example 2: List of Named Lists**
```rust
impl AltListData for NamedListGenerator {
    fn elt(&self, i: usize) -> SEXP {
        // Create a named list for each element
        let names = vec!["x", "y"];
        let values = vec![
            unsafe { Rf_ScalarInteger(i as i32) },
            unsafe { Rf_ScalarReal(i as f64) },
        ];
        // Use miniextendr's list builder
        miniextendr_api::list::named_list(&names, &values).into_sexp()
    }
}
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

## Subsetting Optimization (Extract_subset)

The `extract_subset()` method allows you to optimize R's subsetting operations (`x[indices]`). Instead of R extracting elements one-by-one, you can return a new ALTREP object or optimized representation.

### When R Calls Extract_subset

R calls `extract_subset(x, indices, call)` when:
- User writes `x[c(1, 3, 5)]` - integer vector indices
- User writes `x[condition]` - logical vector indices
- Subsetting with names: `x[c("a", "b")]`

**Note**: Single element access `x[i]` or `x[[i]]` uses `elt()`, not `extract_subset()`.

### Basic Example: Range Subsetting

```rust
use miniextendr_api::altrep_traits::AltVec;
use miniextendr_api::ffi::{SEXP, R_xlen_t};

impl AltVec for RangeData {
    const HAS_EXTRACT_SUBSET: bool = true;

    fn extract_subset(x: SEXP, indices: SEXP, _call: SEXP) -> SEXP {
        // Extract the RangeData from x
        let data = unsafe { altrep_data1_as::<RangeData>(x) }.unwrap();

        // For simple cases, return a new optimized Range
        // Example: Range(1..100)[1..10] = Range(1..10)

        // In practice, you'd:
        // 1. Parse indices SEXP
        // 2. Compute the subset
        // 3. Return new ALTREP or regular vector

        // Fallback to default R behavior for complex cases
        std::ptr::null_mut()  // R will use default elt-based extraction
    }
}
```

### Practical Example: Constant Vector Subset

For a constant vector, any subset is also constant:

```rust
impl AltVec for ConstantIntData {
    const HAS_EXTRACT_SUBSET: bool = true;

    fn extract_subset(x: SEXP, indices: SEXP, _call: SEXP) -> SEXP {
        use miniextendr_api::ffi::{Rf_xlength, TYPEOF, SEXPTYPE};

        let data = unsafe { altrep_data1_as::<ConstantIntData>(x) }?;

        // Get length of indices
        let n = unsafe { Rf_xlength(indices) };

        // Return new constant vector with same value, different length
        let subset = ConstantIntData {
            value: data.value,
            len: n as usize,
        };

        ConstantIntClass(subset).into_sexp()
    }
}
```

### Performance Benefits

**O(1) Subset Creation**:
```r
x <- range_int_altrep(1L, 1000000L)  # O(1) - no allocation
y <- x[1:100000]                       # O(1) - returns new Range(1, 100001)
```

Without `extract_subset`, R would:
1. Allocate a 100,000-element vector
2. Call `elt()` 100,000 times
3. Fill the new vector

With `extract_subset`:
1. Return a new `Range` object (few bytes)
2. No element extraction
3. Lazy evaluation continues

### When to Implement Extract_subset

**Good candidates**:
- ✅ **Mathematical sequences**: Range, arithmetic sequences (subset is another sequence)
- ✅ **Constant vectors**: Subset is constant with different length
- ✅ **Views/windows**: Subset adjusts the window bounds
- ✅ **External data**: Subset delegates to underlying data source
- ✅ **Sparse vectors**: Subset maintains sparsity

**Not worth it for**:
- ❌ **Materialized data** (Vec, Box): R's default is already efficient
- ❌ **Complex computations**: Unless subset is much simpler than original
- ❌ **Small vectors**: Overhead not worth the optimization

### Handling Different Index Types

```rust
fn extract_subset(x: SEXP, indices: SEXP, _call: SEXP) -> SEXP {
    use miniextendr_api::ffi::{TYPEOF, SEXPTYPE};

    unsafe {
        match TYPEOF(indices) {
            SEXPTYPE::INTSXP => {
                // Integer indices: x[c(1L, 3L, 5L)]
                // Extract and process integer vector
            }
            SEXPTYPE::REALSXP => {
                // Numeric indices: x[c(1, 3, 5)]
                // Convert to integers and process
            }
            SEXPTYPE::LGLSXP => {
                // Logical indices: x[c(TRUE, FALSE, TRUE)]
                // Find TRUE positions
            }
            SEXPTYPE::STRSXP => {
                // Named indices: x[c("a", "b")]
                // Match names (if your vector has names)
            }
            _ => {
                // Unknown type - let R handle it
                std::ptr::null_mut()
            }
        }
    }
}
```

### Fallback Strategy

**Always provide a fallback**: Return `NULL` (null_mut()) to let R use default element-by-element extraction:

```rust
fn extract_subset(x: SEXP, indices: SEXP, _call: SEXP) -> SEXP {
    // Try optimized path
    if let Some(result) = try_optimized_subset(x, indices) {
        return result;
    }

    // Fallback: R will call elt() for each index
    std::ptr::null_mut()
}
```

This ensures correctness even when optimization isn't possible.

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

## Advanced Methods

These methods are rarely needed but available for special use cases.

### Inspect - Custom Debug Output

The `inspect()` method customizes the output of `.Internal(inspect(x))`, R's internal debugging tool.

```rust
impl Altrep for MyData {
    const HAS_INSPECT: bool = true;

    fn inspect(
        x: SEXP,
        pre: i32,
        deep: i32,
        pvec: i32,
        inspect_subtree: Option<unsafe extern "C-unwind" fn(SEXP, i32, i32, i32)>,
    ) -> bool {
        // Print custom information
        eprintln!("  MyData ALTREP");
        eprintln!("  - custom_field: {}", /* access your data */);

        // Optionally inspect child objects
        if let Some(inspect) = inspect_subtree {
            unsafe { inspect(/* child SEXP */, pre, deep, pvec); }
        }

        true  // Return true if inspection succeeded
    }
}
```

**When to use**:
- Debugging complex ALTREP structures
- Showing internal state in `.Internal(inspect())`
- Documenting ALTREP design for users

**When to skip**:
- Most use cases (R's default inspection is fine)
- Production code (debugging feature)

### Duplicate - Custom Object Duplication

The `duplicate()` and `duplicate_ex()` methods customize how R duplicates your ALTREP object when copy-on-write semantics require it.

```rust
impl Altrep for LazyWithCache {
    const HAS_DUPLICATE: bool = true;

    fn duplicate(x: SEXP, deep: bool) -> SEXP {
        let data = unsafe { altrep_data1_as::<LazyWithCache>(x) }?;

        if deep {
            // Deep copy: clone cached data too
            let new_data = LazyWithCache {
                params: data.params.clone(),
                cache: RefCell::new(data.cache.borrow().clone()),
            };
            MyClass(new_data).into_sexp()
        } else {
            // Shallow copy: share cache (default R behavior)
            x  // Return self
        }
    }
}
```

**When to use**:
- Controlling what gets copied (cache vs params)
- Optimizing duplication for large cached data
- Implementing copy-on-write semantics
- Sharing immutable state across copies

**When to skip**:
- Default R duplication is correct
- No shared mutable state
- No expensive cached data

**Note**: `duplicate_ex()` is the newer extended version - prefer it over `duplicate()` if implementing both.

### Coerce - Custom Type Conversion

The `coerce()` method customizes how R converts your ALTREP to other types (e.g., integer → real, real → integer).

```rust
impl Altrep for ArithSeq {
    const HAS_COERCE: bool = true;

    fn coerce(x: SEXP, to_type: SEXPTYPE) -> SEXP {
        use SEXPTYPE::*;

        let data = unsafe { altrep_data1_as::<ArithSeq>(x) }?;

        match to_type {
            REALSXP => {
                // Convert integer sequence to real sequence
                // Instead of materializing, return a new Real ALTREP
                let real_seq = RealArithSeq {
                    start: data.start as f64,
                    step: data.step as f64,
                    len: data.len,
                };
                RealArithSeqClass(real_seq).into_sexp()
            }
            _ => {
                // Let R handle other conversions
                std::ptr::null_mut()
            }
        }
    }
}
```

**When to use**:
- Converting between related ALTREP types (IntSeq → RealSeq)
- Avoiding materialization during coercion
- Preserving ALTREP properties after conversion
- Optimizing common conversion paths

**When to skip**:
- Default R coercion is acceptable
- Conversion requires materialization anyway
- Rare conversion path

**Return values**:
- Return new SEXP: Your custom coercion
- Return `NULL` (null_mut()): Let R use default coercion

---

## Materialization and DATAPTR

### Understanding Materialization

**Materialization** is the process of converting your lazy/compact ALTREP representation into a standard R vector with contiguous memory. This happens when R needs direct memory access to your data.

### When R Requests DATAPTR

R calls the `dataptr()` or `dataptr_or_null()` methods when:

1. **Operations requiring contiguous memory**:
   - `sort()`, `order()`, `unique()`
   - `.C()` or `.Fortran()` calls passing the vector
   - `as.vector()` with specific types
   - Some vectorized operations (`x + y`, `x * 2`)

2. **Serialization** (unless you provide `serialize()`)

3. **Interop with other packages** expecting raw pointers

### The Three Dataptr Strategies

#### Strategy 1: No DATAPTR (Lazy Forever)

**When to use**: Pure lazy evaluation, external data sources, mathematical sequences

```rust
// Don't implement AltrepDataptr - only provide elt()

#[derive(miniextendr_api::ExternalPtr)]
pub struct LazySequence {
    start: i32,
    step: i32,
    len: usize,
}

impl AltIntegerData for LazySequence {
    fn elt(&self, i: usize) -> i32 {
        self.start + (i as i32) * self.step
    }
}

// No dataptr option
miniextendr_api::impl_altinteger_from_data!(LazySequence);
```

**Behavior**:
- ✅ O(1) creation
- ✅ O(1) element access
- ❌ Operations needing DATAPTR will materialize to regular R vector
- ❌ R owns the materialized copy (you lose control)

#### Strategy 2: Materialization on Demand

**When to use**: Lazy until needed, then cache the materialized form

```rust
use miniextendr_api::altrep_data::AltrepDataptr;
use std::cell::RefCell;

#[derive(miniextendr_api::ExternalPtr)]
pub struct LazyWithCache {
    // Computation parameters
    start: i32,
    step: i32,
    len: usize,

    // Materialized cache (initially None)
    materialized: RefCell<Option<Vec<i32>>>,
}

impl AltrepDataptr<i32> for LazyWithCache {
    fn dataptr(&mut self, _writable: bool) -> Option<*mut i32> {
        // Materialize on first call
        let mut mat = self.materialized.borrow_mut();
        if mat.is_none() {
            let vec: Vec<i32> = (0..self.len)
                .map(|i| self.start + (i as i32) * self.step)
                .collect();
            *mat = Some(vec);
        }

        // Return pointer to cached data
        mat.as_mut().map(|v| v.as_mut_ptr())
    }

    fn dataptr_or_null(&self) -> Option<*const i32> {
        // Return None if not yet materialized (saves memory)
        self.materialized
            .borrow()
            .as_ref()
            .map(|v| v.as_ptr())
    }
}

// Enable dataptr
miniextendr_api::impl_altinteger_from_data!(LazyWithCache, dataptr);
```

**Behavior**:
- ✅ Lazy until DATAPTR requested
- ✅ Subsequent DATAPTR calls are O(1)
- ✅ You control the materialized form
- ⚠️ Uses memory after materialization

#### Strategy 3: Pre-Materialized (Vec/Box)

**When to use**: Data already in memory, just wrapping existing vector

```rust
#[derive(miniextendr_api::ExternalPtr)]
pub struct VecWrapper {
    data: Vec<i32>,
}

impl AltrepDataptr<i32> for VecWrapper {
    fn dataptr(&mut self, _writable: bool) -> Option<*mut i32> {
        Some(self.data.as_mut_ptr())
    }

    fn dataptr_or_null(&self) -> Option<*const i32> {
        Some(self.data.as_ptr())
    }
}

miniextendr_api::impl_altinteger_from_data!(VecWrapper, dataptr);
```

**Behavior**:
- ✅ DATAPTR always available (O(1))
- ✅ No lazy evaluation overhead
- ❌ Memory used immediately
- ❌ No computation savings

### Materialization Trade-offs

| Aspect | No DATAPTR | On-Demand | Pre-Materialized |
|--------|------------|-----------|------------------|
| **Memory** | Minimal | Grows on use | Full upfront |
| **Speed** | Fast `elt()` | Fast after first | Fastest DATAPTR |
| **Use case** | Math sequences | Caching | Existing data |
| **Lazy eval** | ✅ Always | ✅ Until DATAPTR | ❌ Never |

### When to Provide DATAPTR

**Provide DATAPTR if**:
- ✅ Your data is already in memory (Vec, Box, slice)
- ✅ Users will frequently perform operations requiring contiguous memory
- ✅ You can efficiently materialize when needed
- ✅ You want to control the materialization process

**Skip DATAPTR if**:
- ✅ Data is external (database, file, network)
- ✅ Pure mathematical sequence (no need to materialize)
- ✅ Memory is at a premium
- ✅ R's default materialization is acceptable

### Safety Requirements

When implementing `dataptr()`:

1. **Pointer Validity**: The returned pointer must remain valid until the next GC or until the ALTREP object is collected

2. **Lifetime**: Store materialized data in the ALTREP object itself (in the data1 ExternalPtr)

3. **Mutability**: If `writable=true`, the pointer must be mutable. R may modify the data.

```rust
// ❌ WRONG - pointer becomes invalid
fn dataptr(&mut self, _writable: bool) -> Option<*mut i32> {
    let vec = vec![1, 2, 3];
    Some(vec.as_mut_ptr())  // vec is dropped! Pointer is now invalid!
}

// ✅ CORRECT - pointer remains valid
fn dataptr(&mut self, _writable: bool) -> Option<*mut i32> {
    self.cached_data.as_mut().map(|v| v.as_mut_ptr())
}
```

### Example: Controlling Materialization

```rust
#[derive(miniextendr_api::ExternalPtr)]
pub struct OptionallyMaterialized {
    generator: Box<dyn Fn(usize) -> i32>,
    len: usize,
    cache: RefCell<Option<Vec<i32>>>,
}

impl OptionallyMaterialized {
    pub fn is_materialized(&self) -> bool {
        self.cache.borrow().is_some()
    }

    pub fn force_materialize(&mut self) {
        if self.cache.borrow().is_none() {
            let vec = (0..self.len).map(|i| (self.generator)(i)).collect();
            *self.cache.borrow_mut() = Some(vec);
        }
    }
}
```

**Key Insight**: Materialization is a one-way door. Once materialized, you typically stay materialized. Plan your memory strategy accordingly.

---

## Troubleshooting

### "Error: could not find function"
- Ensure constructor function has `#[miniextendr]` and is `pub`
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
