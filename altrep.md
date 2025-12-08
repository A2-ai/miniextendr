# ALTREP Design for miniextendr

## Overview

ALTREP (Alternative Representation) is R's mechanism for custom vector implementations that can provide lazy evaluation, compact storage, or special access patterns without materializing a full R vector.

## R's ALTREP Hierarchy

From the R source (`altrep.c`, `Altrep.h`):

```default
ALTREP (base)
└── ALTVEC (vector base)
    ├── ALTINTEGER  (i32)
    ├── ALTREAL     (f64)
    ├── ALTLOGICAL  (i32 tristate)
    ├── ALTRAW      (u8)
    ├── ALTCOMPLEX  (Rcomplex)
    ├── ALTSTRING   (CHARSXP elements)
    └── ALTLIST     (SEXP elements)
```

### Method Tables (from `altrep.c`)

**ALTREP_METHODS** (base, all types):

- `Length(SEXP) -> R_xlen_t` - vector length
- `Duplicate(SEXP, deep) -> SEXP` - shallow/deep copy
- `DuplicateEX(SEXP, deep) -> SEXP` - extended duplicate
- `Coerce(SEXP, type) -> SEXP` - type coercion
- `Inspect(SEXP, pre, deep, pvec, subtree_fn) -> bool` - R's `.Internal(inspect())`
- `Serialized_state(SEXP) -> SEXP` - state for serialization
- `Unserialize(class, state) -> SEXP` - reconstruct from state
- `UnserializeEX(class, state, attr, objf, levs) -> SEXP` - extended unserialize

**ALTVEC_METHODS** (extends ALTREP):

- `Dataptr(SEXP, writable) -> *void` - get raw data pointer
- `Dataptr_or_null(SEXP) -> *const void` - non-allocating pointer (or NULL)
- `Extract_subset(SEXP, indx, call) -> SEXP` - optimized subsetting

**ALTINTEGER_METHODS** (extends ALTVEC):

- `Elt(SEXP, i) -> i32` - element access
- `Get_region(SEXP, i, n, buf) -> R_xlen_t` - bulk read
- `Is_sorted(SEXP) -> i32` - sortedness hint (UNKNOWN=INT_MIN, no=-1, increasing=0, decreasing=1, etc.)
- `No_NA(SEXP) -> i32` - NA-free hint (0=unknown, 1=no NAs)
- `Sum(SEXP, narm) -> SEXP` - optimized sum
- `Min(SEXP, narm) -> SEXP` - optimized min
- `Max(SEXP, narm) -> SEXP` - optimized max

**ALTREAL_METHODS** (extends ALTVEC):

- Same as ALTINTEGER but `Elt` returns `f64`

**ALTLOGICAL_METHODS** (extends ALTVEC):

- `Elt`, `Get_region`, `Is_sorted`, `No_NA`, `Sum` (no Min/Max)

**ALTRAW_METHODS** (extends ALTVEC):

- `Elt` (returns `u8`), `Get_region` only

**ALTCOMPLEX_METHODS** (extends ALTVEC):

- `Elt` (returns `Rcomplex`), `Get_region` only

**ALTSTRING_METHODS** (extends ALTVEC):

- `Elt(SEXP, i) -> SEXP` - returns CHARSXP
- `Set_elt(SEXP, i, v)` - set element (for mutable strings)
- `Is_sorted`, `No_NA`

**ALTLIST_METHODS** (extends ALTVEC):

- `Elt(SEXP, i) -> SEXP` - returns element SEXP
- `Set_elt(SEXP, i, v)` - set element

### Required vs Optional Methods

| Type | Required Methods |
|------|------------------|
| All | `length` |
| ALTSTRING | `length` + `elt` |
| ALTLIST | `length` + `elt` |
| Numeric types | `length` + (`elt` OR `dataptr`) |

## miniextendr ALTREP Architecture

### Two-Layer Design

miniextendr uses a two-layer trait design:

1. **High-level data traits** (`altrep_data.rs`): User-friendly `&self` methods
2. **Low-level FFI traits** (`altrep_traits.rs`): Raw `SEXP` callbacks for R

```
┌─────────────────────────────────────────────────────────────────┐
│                    User Code                                    │
│                                                                 │
│  impl AltrepLen for MyData { fn len(&self) -> usize { ... } }  │
│  impl AltIntegerData for MyData { fn elt(&self, i) -> i32 {...}}│
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼ impl_altinteger_from_data!(MyData)
┌─────────────────────────────────────────────────────────────────┐
│                Generated Low-Level Traits                       │
│                                                                 │
│  impl Altrep for MyData { fn length(x: SEXP) -> R_xlen_t {...} }│
│  impl AltInteger for MyData { fn elt(x: SEXP, i) -> i32 {...} } │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼ #[miniextendr(class=..., base=...)]
┌─────────────────────────────────────────────────────────────────┐
│                ALTREP Class Registration                        │
│                                                                 │
│  struct MyClass(MyData);  // 1-field wrapper                   │
│  MyClass::into_altrep(data) -> SEXP                            │
└─────────────────────────────────────────────────────────────────┘
```

### High-Level Data Traits

```rust
/// Base trait - all ALTREP types must provide length
pub trait AltrepLen {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool { self.len() == 0 }
}

/// Integer vector data
pub trait AltIntegerData: AltrepLen {
    fn elt(&self, i: usize) -> i32;

    // Optional methods with defaults
    fn as_slice(&self) -> Option<&[i32]> { None }
    fn get_region(&self, start: usize, len: usize, buf: &mut [i32]) -> usize { ... }
    fn is_sorted(&self) -> Option<Sortedness> { None }
    fn no_na(&self) -> Option<bool> { None }
    fn sum(&self, na_rm: bool) -> Option<i64> { None }
    fn min(&self, na_rm: bool) -> Option<i32> { None }
    fn max(&self, na_rm: bool) -> Option<i32> { None }
}

/// Real vector data
pub trait AltRealData: AltrepLen {
    fn elt(&self, i: usize) -> f64;
    // ... similar optional methods
}

/// Logical vector data (uses Logical enum: True, False, Na)
pub trait AltLogicalData: AltrepLen {
    fn elt(&self, i: usize) -> Logical;
    // ...
}

/// Raw byte vector data
pub trait AltRawData: AltrepLen {
    fn elt(&self, i: usize) -> u8;
    // ...
}

/// String vector data
pub trait AltStringData: AltrepLen {
    fn elt(&self, i: usize) -> Option<&str>;  // None = NA
    // ...
}

/// List vector data
pub trait AltListData: AltrepLen {
    fn elt(&self, i: usize) -> SEXP;
}
```

### Low-Level FFI Traits

```rust
/// Base ALTREP methods - length is REQUIRED
pub trait Altrep {
    fn length(x: SEXP) -> R_xlen_t;  // Required, no default

    // Optional methods gated by HAS_* constants
    const HAS_DUPLICATE: bool = false;
    fn duplicate(_x: SEXP, _deep: bool) -> SEXP { unreachable!() }

    const HAS_SERIALIZED_STATE: bool = false;
    fn serialized_state(_x: SEXP) -> SEXP { unreachable!() }
    // ...
}

/// Vector-level methods
pub trait AltVec: Altrep {
    const HAS_DATAPTR: bool = false;
    fn dataptr(_x: SEXP, _writable: bool) -> *mut c_void { unreachable!() }

    const HAS_DATAPTR_OR_NULL: bool = false;
    fn dataptr_or_null(_x: SEXP) -> *const c_void { unreachable!() }
    // ...
}

/// Integer methods
pub trait AltInteger: AltVec {
    const HAS_ELT: bool = false;
    fn elt(_x: SEXP, _i: R_xlen_t) -> i32 { unreachable!() }

    const HAS_GET_REGION: bool = false;
    fn get_region(_x: SEXP, _i: R_xlen_t, _n: R_xlen_t, _buf: *mut i32) -> R_xlen_t { unreachable!() }
    // ...
}
```

When `HAS_*` is `false`, the method is **not installed** with R, so R uses its default behavior.

### Bridge Macros

Bridge macros generate low-level trait implementations from high-level data traits:

```rust
// Generate Altrep, AltVec, AltInteger impls from AltIntegerData
miniextendr_api::impl_altinteger_from_data!(MyData);

// Similar macros for other types:
// impl_altreal_from_data!(T)
// impl_altlogical_from_data!(T)
// impl_altraw_from_data!(T)
// impl_altstring_from_data!(T)
// impl_altlist_from_data!(T)
// impl_altcomplex_from_data!(T)
```

The macros:

- Extract data from SEXP via `altrep_data1_as::<T>(x)`
- Convert Rust types to R types (e.g., `Option<bool>` to R's `i32` sortedness)
- Set appropriate `HAS_*` flags based on trait methods

### Built-in Implementations

Standard types have built-in implementations:

| Rust Type | Data Trait | Features |
|-----------|------------|----------|
| `Vec<i32>` | `AltIntegerData` | `dataptr`, `get_region`, `sum`, `min`, `max` |
| `Vec<f64>` | `AltRealData` | `dataptr`, `get_region`, `sum`, `min`, `max` |
| `Vec<bool>` | `AltLogicalData` | `no_na` (always true), `sum` |
| `Vec<u8>` | `AltRawData` | `dataptr`, `get_region` |
| `Vec<String>` | `AltStringData` | `no_na` (always true) |
| `Range<i32>` | `AltIntegerData` | O(1) `sum`, `min`, `max`, `is_sorted` |
| `Range<f64>` | `AltRealData` | O(1) `sum`, `min`, `max`, `is_sorted` |
| `[T; N]` | (same as Vec) | Fixed-size arrays |

### ALTREP Class Registration

Use `#[miniextendr]` on a 1-field struct:

```rust
// 1. Define your data type with ExternalPtr derive
#[derive(miniextendr_api::ExternalPtr)]
pub struct FibonacciData {
    len: usize,
}

// 2. Implement high-level data traits
impl AltrepLen for FibonacciData {
    fn len(&self) -> usize { self.len }
}

impl AltIntegerData for FibonacciData {
    fn elt(&self, i: usize) -> i32 {
        // Compute fibonacci(i)
        fib(i) as i32
    }

    fn no_na(&self) -> Option<bool> { Some(true) }
}

// 3. Generate low-level traits
miniextendr_api::impl_altinteger_from_data!(FibonacciData);

// 4. Register ALTREP class with proc-macro
#[miniextendr(class = "Fibonacci", pkg = "mypkg", base = "Int")]
pub struct FibonacciClass(FibonacciData);

// 5. Create instances
fn create_fibonacci(n: i32) -> SEXP {
    let data = FibonacciData { len: n as usize };
    unsafe { FibonacciClass::into_altrep(data) }
}
```

The `#[miniextendr]` macro on the struct:

- Registers the ALTREP class with R via `R_make_altinteger_class`
- Installs trampolines based on `HAS_*` flags
- Generates `into_altrep(data) -> SEXP` for instance creation

### Using Standard Types Directly

For standard types, skip the data trait implementation:

```rust
// Vec<i32> already has built-in implementations
#[miniextendr(class = "SimpleVecInt", pkg = "mypkg", base = "Int")]
pub struct SimpleVecIntClass(Vec<i32>);

fn create_vec_int(data: Vec<i32>) -> SEXP {
    unsafe { SimpleVecIntClass::into_altrep(data) }
}
```

## Complete Example

```rust
use miniextendr_api::altrep_data::{AltIntegerData, AltrepLen, Sortedness};
use miniextendr_api::{miniextendr, ExternalPtr};

/// Arithmetic sequence: start, start+step, start+2*step, ...
#[derive(ExternalPtr)]
pub struct ArithSeq {
    start: i32,
    step: i32,
    len: usize,
}

impl AltrepLen for ArithSeq {
    fn len(&self) -> usize { self.len }
}

impl AltIntegerData for ArithSeq {
    fn elt(&self, i: usize) -> i32 {
        self.start + (i as i32) * self.step
    }

    fn is_sorted(&self) -> Option<Sortedness> {
        Some(if self.step > 0 {
            Sortedness::StrictlyIncreasing
        } else if self.step < 0 {
            Sortedness::StrictlyDecreasing
        } else {
            Sortedness::Increasing  // All equal
        })
    }

    fn no_na(&self) -> Option<bool> { Some(true) }

    fn sum(&self, _na_rm: bool) -> Option<i64> {
        if self.len == 0 { return Some(0); }
        let n = self.len as i64;
        let first = self.start as i64;
        let last = first + (n - 1) * self.step as i64;
        Some(n * (first + last) / 2)  // O(1)
    }

    fn min(&self, _na_rm: bool) -> Option<i32> {
        if self.len == 0 { None }
        else if self.step >= 0 { Some(self.start) }
        else { Some(self.elt(self.len - 1)) }
    }

    fn max(&self, _na_rm: bool) -> Option<i32> {
        if self.len == 0 { None }
        else if self.step <= 0 { Some(self.start) }
        else { Some(self.elt(self.len - 1)) }
    }
}

// Generate low-level traits
miniextendr_api::impl_altinteger_from_data!(ArithSeq);

// Register class
#[miniextendr(class = "ArithSeq", pkg = "mypkg", base = "Int")]
pub struct ArithSeqClass(ArithSeq);

// R-callable constructor
#[miniextendr]
fn arith_seq(from: i32, by: i32, length_out: i32) -> SEXP {
    let data = ArithSeq { start: from, step: by, len: length_out as usize };
    unsafe { ArithSeqClass::into_altrep(data) }
}

// Register in module
miniextendr_module! {
    mod mypkg;
    struct ArithSeqClass;
    fn arith_seq;
}
```

## C NULL vs R_NilValue Return Semantics

**Critical distinction**: ALTREP methods use C `NULL` and R's `R_NilValue` for different purposes.

| Method | Return `NULL` | Return `R_NilValue` |
|--------|---------------|---------------------|
| `Serialized_state` | Use default serialization | Serialize with nil state |
| `Duplicate` | Use default duplication | Return nil object |
| `Sum/Min/Max` | Use default implementation | Return NA |
| `Dataptr_or_null` | No pointer (use Elt) | — |
| `Elt` (list/string) | — | Element is NULL |

The bridge macros handle this correctly by returning `R_NilValue` for "use default" cases from optional methods.

## Sortedness Values

```rust
pub const UNKNOWN_SORTEDNESS: i32 = i32::MIN;
pub const SORTED_NONE: i32 = -1;      // Not sorted
pub const SORTED_INCR: i32 = 0;       // Increasing (may have ties)
pub const SORTED_DECR: i32 = 1;       // Decreasing (may have ties)
pub const SORTED_INCR_STRICT: i32 = 2;  // Strictly increasing
pub const SORTED_DECR_STRICT: i32 = -2; // Strictly decreasing
```

Use the `Sortedness` enum in high-level code:

```rust
pub enum Sortedness {
    Unknown, None, Increasing, Decreasing,
    StrictlyIncreasing, StrictlyDecreasing
}
```

## Implementation Status

### Completed

- [x] Two-layer trait hierarchy (data traits + FFI traits)
- [x] Bridge macros for all ALTREP types
- [x] Built-in implementations for `Vec<T>`, `Range<T>`, `[T; N]`
- [x] Proc-macro ALTREP class registration
- [x] `into_altrep()` instance creation
- [x] Trampoline generation with `HAS_*` gating
- [x] ExternalPtr integration for data storage
- [x] FFI bindings for all R ALTREP APIs

### Not Yet Implemented

- [ ] `Dataptr` with lazy materialization pattern
- [ ] Serialization/unserialization support
- [ ] `Extract_subset` optimization
- [ ] Memory-mapped file ALTREP
- [ ] Complex number ALTREP examples

## References

- R source: `src/main/altrep.c`, `src/main/altclasses.c`
- R headers: `src/include/R_ext/Altrep.h`
- Example packages: `simplemmap`, `mutable`
- Luke Tierney's ALTREP design document (DSC 2017)
