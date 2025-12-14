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
- `Is_sorted(SEXP) -> i32` - sortedness hint (UNKNOWN=INT_MIN, unsorted=0, increasing=1, decreasing=-1, NA-first=±2)
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
| `Box<[i32]>` | `AltIntegerData` | `dataptr`, `get_region`, `sum`, `min`, `max` |
| `Box<[f64]>` | `AltRealData` | `dataptr`, `get_region`, `sum`, `min`, `max` |
| `Box<[bool]>` | `AltLogicalData` | `no_na` (always true), `sum` |
| `Box<[u8]>` | `AltRawData` | `dataptr`, `get_region` |
| `Box<[String]>` | `AltStringData` | `no_na` (always true) |
| `Range<i32>` | `AltIntegerData` | O(1) `sum`, `min`, `max`, `is_sorted` |
| `Range<f64>` | `AltRealData` | O(1) `sum`, `min`, `max`, `is_sorted` |
| `[T; N]` | (same as Vec) | Fixed-size arrays |
| `&'static [T]` | (same as Vec) | Static slices with `dataptr` |

### `Box<[T]>`, `&'static [T]`, and `&[T]`

**R vectors are more like `Box<[T]>` than `Vec<T>`** - fixed size, heap-allocated, no capacity overhead.

| Type | Can use with ALTREP? | Reason |
|------|---------------------|--------|
| `Vec<T>` | ✅ Yes | Owned, sized, works with ExternalPtr |
| `[T; N]` | ✅ Yes | Owned, sized (compile-time length) |
| `&'static [T]` | ✅ Yes | Static lifetime, sized (fat pointer) |
| `Box<[T]>` | ✅ Yes | Owned, sized (fat pointer: ptr + len) |
| `&[T]` | ❌ No | Borrowed - ALTREP must own its data |

**`Box<[T]>` (owned slice):** Works because `Box<[T]>` is actually `Sized` - it's a fat pointer (2 words: ptr + len), same as `&[T]`. Use this when you want fixed-size data with no reallocation capability and no capacity overhead:

```rust
#[miniextendr(class = "BoxedInts", pkg = "mypkg")]
pub struct BoxedIntsClass(Box<[i32]>);

fn create_boxed(v: Vec<i32>) -> SEXP {
    let boxed: Box<[i32]> = v.into_boxed_slice();
    unsafe { BoxedIntsClass::into_altrep(boxed) }
}
```

**`&'static [T]` (static slice):** Works because the fat pointer itself is `Sized` (2 words: ptr + len) and `'static` lifetime means the data lives forever - no dangling references. Use cases:

```rust
// Const arrays
static DATA: [i32; 5] = [1, 2, 3, 4, 5];

#[miniextendr(class = "StaticInts", pkg = "mypkg")]
pub struct StaticIntsClass(&'static [i32]);

fn create_static() -> SEXP {
    unsafe { StaticIntsClass::into_altrep(&DATA[..]) }
}

// Leaked data (intentional memory leak for process lifetime)
fn create_leaked(v: Vec<i32>) -> SEXP {
    let leaked: &'static [i32] = Box::leak(v.into_boxed_slice());
    unsafe { StaticIntsClass::into_altrep(leaked) }
}

// String literals
static NAMES: [&'static str; 3] = ["alpha", "beta", "gamma"];

#[miniextendr(class = "StaticNames", pkg = "mypkg")]
pub struct StaticNamesClass(&'static [&'static str]);
```

**`&[T]` (borrowed slice):** Cannot work at all. ALTREP objects are R objects that can be stored, serialized, passed around - they must *own* their data. A borrowed slice would become invalid when the borrow ends.

**Practical recommendation:**

- Use `Vec<T>` when you need to build data dynamically
- Use `Box<[T]>` when you have a fixed-size collection (saves capacity field overhead)
- Use `&'static [T]` for compile-time constants or leaked data

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

### Auto-Inferred Base Type

The `base` attribute is **optional**. The base type and method installation are automatically inferred from the inner data type via the `InferBase` trait:

```rust
// Base type "Real" is automatically inferred from Vec<f64>
#[miniextendr(class = "MyReals", pkg = "mypkg")]
pub struct MyRealsClass(Vec<f64>);

// Equivalent to:
// #[miniextendr(class = "MyReals", pkg = "mypkg", base = "Real")]
```

Supported auto-inference mappings:

| Inner Type | Inferred Base |
|------------|---------------|
| `Vec<i32>` | `Int` |
| `Vec<f64>` | `Real` |
| `Vec<bool>` | `Logical` |
| `Vec<u8>` | `Raw` |
| `Vec<String>` | `String` |
| `Range<i32>`, `Range<i64>` | `Int` |
| `Range<f64>` | `Real` |
| `[i32; N]`, `[f64; N]`, etc. | Corresponding type |
| `&'static [i32]`, `&'static [f64]`, etc. | Corresponding type |
| `&'static [&'static str]` | `String` |

For custom data types, just use the appropriate `impl_alt*_from_data!` macro - it automatically enables base type inference:

```rust
// The impl_altinteger_from_data! macro automatically enables base type inference
miniextendr_api::impl_altinteger_from_data!(MyCustomData);

// Now you can omit base:
#[miniextendr(class = "MyCustom", pkg = "mypkg")]  // No base needed!
pub struct MyCustomClass(MyCustomData);
```

All `impl_alt*_from_data!` macros automatically enable base type inference - no additional macro call needed.

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
            Sortedness::Increasing
        } else if self.step < 0 {
            Sortedness::Decreasing
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

// Generate low-level traits (automatically enables base type inference)
miniextendr_api::impl_altinteger_from_data!(ArithSeq);

// Register class - no `base` attribute needed!
#[miniextendr(class = "ArithSeq", pkg = "mypkg")]
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

## Dataptr with Lazy Materialization

Some ALTREP types compute values on-demand (e.g., arithmetic sequences, Fibonacci). They can benefit from **lazy materialization**: the full data buffer is only allocated when `Dataptr` is called, but individual element access via `Elt` or `Get_region` can compute values without materialization.

### The Pattern

```rust
use miniextendr_api::altrep_data::{AltIntegerData, AltrepLen, AltrepDataptr};

#[derive(miniextendr_api::ExternalPtr)]
pub struct LazySequence {
    start: i32,
    step: i32,
    len: usize,
    /// Lazily-allocated buffer - None until Dataptr is called
    materialized: Option<Vec<i32>>,
}

impl AltrepLen for LazySequence {
    fn len(&self) -> usize { self.len }
}

impl AltIntegerData for LazySequence {
    fn elt(&self, i: usize) -> i32 {
        // Compute on-the-fly without materializing
        self.start + (i as i32) * self.step
    }
    // ... other methods (sum, min, max can be O(1))
}

/// Implement AltrepDataptr for lazy materialization
impl AltrepDataptr<i32> for LazySequence {
    fn dataptr(&mut self, _writable: bool) -> Option<*mut i32> {
        // Materialize on first Dataptr access
        if self.materialized.is_none() {
            let data: Vec<i32> = (0..self.len)
                .map(|i| self.start + (i as i32) * self.step)
                .collect();
            self.materialized = Some(data);
        }
        self.materialized.as_mut().map(|v| v.as_mut_ptr())
    }

    fn dataptr_or_null(&self) -> Option<*const i32> {
        // Only return pointer if already materialized
        // Returning None allows R to use Elt/Get_region for unmaterialized data
        self.materialized.as_ref().map(|v| v.as_ptr())
    }
}

// Use the `dataptr` variant to enable Dataptr methods (also enables base type inference)
miniextendr_api::impl_altinteger_from_data!(LazySequence, dataptr);

// Register ALTREP class - no `base` attribute needed!
#[miniextendr(class = "LazySequence", pkg = "mypkg")]
pub struct LazySequenceClass(LazySequence);
```

### Key Points

1. **`Elt` computes on-the-fly**: No allocation needed for individual element access
2. **`Dataptr` triggers materialization**: Full buffer is allocated and populated
3. **`Dataptr_or_null` returns `None` until materialized**: R will use `Elt` if available
4. **Use `dataptr` variant**: Pass `, dataptr` to `impl_alt*_from_data!` macro to enable these methods
5. **Base type is auto-inferred**: All `impl_alt*_from_data!` macros enable inference, no `base = "..."` needed

### When to Use Lazy Materialization

- Arithmetic/geometric sequences where formulas allow O(1) element access
- Fibonacci or other recurrence relations with memoization
- Computed sequences that might never need full materialization
- Memory-mapped files that shouldn't be loaded entirely into memory

### Testing Materialization

R code to test:

```r
# Create lazy sequence - not materialized yet
x <- lazy_int_seq(1L, 100L, 1L)

# Element access doesn't materialize
x[1]  # Uses Elt method

# Sum uses optimized method (O(1) for arithmetic sequences)
sum(x)

# Force materialization
y <- x + 0L  # Dataptr is called
```

## Serialization Support

ALTREP objects can be saved/loaded via `saveRDS()`/`readRDS()` by implementing the `AltrepSerialize` trait.

### The Pattern

```rust
use miniextendr_api::altrep_data::AltrepSerialize;
use miniextendr_api::ffi::{SEXP, Rf_allocVector, SET_INTEGER_ELT, INTEGER_ELT, INTSXP};

impl AltrepSerialize for MySeqData {
    fn serialized_state(&self) -> SEXP {
        // Store parameters needed to reconstruct the ALTREP
        unsafe {
            let state = Rf_allocVector(INTSXP, 3);
            SET_INTEGER_ELT(state, 0, self.start);
            SET_INTEGER_ELT(state, 1, self.step);
            SET_INTEGER_ELT(state, 2, self.len as i32);
            state
        }
    }

    fn unserialize(state: SEXP) -> Option<Self> {
        unsafe {
            let start = INTEGER_ELT(state, 0);
            let step = INTEGER_ELT(state, 1);
            let len = INTEGER_ELT(state, 2) as usize;
            Some(MySeqData { start, step, len, materialized: None })
        }
    }
}

// Enable serialization with the `serialize` variant
miniextendr_api::impl_altinteger_from_data!(MySeqData, serialize);

// Or combine with dataptr:
miniextendr_api::impl_altinteger_from_data!(MySeqData, dataptr, serialize);
```

### Key Points

1. **`serialized_state()`**: Convert your data to an R object (list, vector, etc.)
2. **`unserialize()`**: Reconstruct your data from that R object
3. **Use `serialize` variant**: Pass `, serialize` to `impl_alt*_from_data!` macro
4. **Don't serialize ephemeral state**: Computed caches, file handles, etc. should be reconstructed

### What to Serialize

- **Parameters**: Values that define the ALTREP (start, step, length for sequences)
- **NOT materialized buffers**: Let them be recomputed on demand
- **NOT pointers/handles**: They won't survive the save/load cycle

## Extract_subset Optimization

ALTREP types can provide optimized subsetting (e.g., `x[1:10]`) by implementing `AltrepExtractSubset`.

### When to Use

- **Arithmetic sequences**: `seq(1, 1000000)[1:10]` can return a new sequence without materializing
- **Lazy types**: Return another lazy object covering just the subset
- **Memory-mapped files**: Return a view without loading everything

### Implementation

```rust
use miniextendr_api::altrep_data::AltrepExtractSubset;

impl AltrepExtractSubset for ArithSeqData {
    fn extract_subset(&self, indices: &[i32]) -> Option<SEXP> {
        // Check for contiguous range like 1:10
        if is_contiguous_range(indices) {
            let start_idx = indices[0] as usize - 1; // Convert to 0-based
            let new_start = self.start + (start_idx as i32) * self.step;
            let new_len = indices.len();

            // Create a new ArithSeq for the subset
            let new_data = ArithSeqData {
                start: new_start,
                step: self.step,
                len: new_len,
            };
            return Some(unsafe { ArithSeqClass::into_altrep(new_data) });
        }

        // For non-contiguous subsets, return None to let R handle it
        None
    }
}

// Enable with the `subset` variant
miniextendr_api::impl_altinteger_from_data!(ArithSeqData, subset);
```

### Notes

- `indices` are 1-based R indices (NA represented as `i32::MIN`)
- Return `None` to fall back to R's default subsetting
- Return `Some(sexp)` with the optimized subset result

## C NULL vs R_NilValue Return Semantics

**Critical distinction**: for many ALTREP callbacks, C `NULL` (a null `SEXP` pointer) is a sentinel
that means “not implemented / fall back to R’s default behavior”. `R_NilValue` is a real R object
(the `NULL` object) and is **not** the same thing.

| Method | Return `NULL` | Return `R_NilValue` |
|--------|---------------|---------------------|
| `Serialized_state` | Fall back to standard serialization | Serialize with `state = NULL` (rare; only if your class supports it) |
| `Coerce` | Fall back to standard coercion | Coercion result is the `NULL` object (almost never what you want) |
| `Duplicate` | Fall back to standard duplication | Duplication result is the `NULL` object (almost never what you want) |
| `Extract_subset` | Fall back to standard subsetting | Subset result is the `NULL` object (almost never what you want) |
| `Sum/Min/Max` | Fall back to standard summary | Summary result is the `NULL` object (use scalar `NA_*` to return NA) |
| `Dataptr_or_null` | No pointer (use Elt) | — |
| `Elt` (list) | — (must return a valid `SEXP`) | Element is `NULL` (valid list element) |

The bridge macros use `NULL` when they need to signal “fall back to R”.

## Sortedness Values

```rust
pub const UNKNOWN_SORTEDNESS: i32 = i32::MIN;
pub const KNOWN_UNSORTED: i32 = 0;
pub const SORTED_INCR: i32 = 1;          // Increasing (may have ties)
pub const SORTED_DECR: i32 = -1;         // Decreasing (may have ties)
pub const SORTED_INCR_NA_1ST: i32 = 2;   // Increasing, NAs first
pub const SORTED_DECR_NA_1ST: i32 = -2;  // Decreasing, NAs first
```

Use the `Sortedness` enum in high-level code:

```rust
pub enum Sortedness {
    Unknown,
    KnownUnsorted,
    Increasing,
    Decreasing,
    IncreasingNaFirst,
    DecreasingNaFirst,
}
```

## Methods Using R Defaults

miniextendr has full trait infrastructure for optional ALTREP methods, but built-in types use R's defaults:

| Method | Trait Support | Built-in Types | R Default Behavior |
|--------|---------------|----------------|-------------------|
| `Coerce` | `HAS_COERCE` + `coerce()` | Use default | Standard type conversion (e.g., int→real) |
| `Duplicate` | `HAS_DUPLICATE` + `duplicate()` | Use default | Standard vector duplication |
| `Set_elt` | `HAS_SET_ELT` + `set_elt()` | Use default | Standard element assignment |
| `Serialized_state` | `AltrepSerialize` trait | Optional | Serialize as regular vector |

**When to implement these in custom ALTREP:**

- **Coerce**: Only if your ALTREP can produce a more efficient representation when coerced (e.g., R's compact sequences coerce int→real without expanding)
- **Duplicate**: Only if you can duplicate more efficiently than element-by-element copy
- **Set_elt**: Only for mutable string/list ALTREP (rare use case)
- **Serialized_state**: When you want ALTREP to survive serialization/deserialization

For most use cases, R's defaults are appropriate. The built-in implementations (`Vec<T>`, `Range<T>`, etc.) rely on defaults for these methods.

### Example: Custom Coerce Implementation

```rust
impl Altrep for MyRangeClass {
    const HAS_COERCE: bool = true;

    fn coerce(x: SEXP, to_type: SEXPTYPE) -> SEXP {
        // Return C NULL to use R's default coercion
        // Or return a custom SEXP for optimized conversion
        core::ptr::null_mut()
    }
    // ... other methods
}
```

## Testing

### Rust Unit Tests

The `altrep_data` module includes unit tests for pure Rust functionality. Run with:

```bash
cargo test --manifest-path miniextendr-api/Cargo.toml
```

Tests cover:

- **Enum conversions**: `Logical` (to/from R's int), `Sortedness` (to/from R's int)
- **Vec implementations**: `AltIntegerData`, `AltRealData`, `AltLogicalData`, `AltRawData`, `AltStringData`
- **Box<[T]> implementations**: Same traits as Vec, for owned slices
- **Range implementations**: O(1) `sum`, `min`, `max` for arithmetic sequences
- **Static slice implementations**: `&'static [T]` with NA handling
- **Array implementations**: `[T; N]` for fixed-size arrays
- **Edge cases**: Empty vectors, single elements, overflow handling, NA propagation

### R Integration Tests

The `rpkg` package includes testthat tests that verify ALTREP behavior from R. Run with:

```r
testthat::test_file("rpkg/tests/testthat/test-altrep.R")
```

Tests cover:

- **Proc-macro ALTREP**: `ConstantIntClass` - element access, sum, length
- **Complex ALTREP**: `UnitCircle` - roots of unity computation
- **Lazy materialization**: `LazyIntSeqClass` - deferred computation, O(1) sum, materialization trigger
- **Static slices**: `StaticIntsClass`, `StaticStringsClass` - const array ALTREP
- **Leaked data**: `leaked_ints()` - `Box::leak()` pattern for process-lifetime data
- **Box<[T]>**: `BoxedIntsClass` - owned slice ALTREP with dataptr support

### Testing Custom ALTREP Classes

When implementing custom ALTREP:

1. **Test element access**: Verify `elt(i)` returns correct values
2. **Test length**: Verify `len()` matches expected size
3. **Test optional methods**: If you implement `sum`, `min`, `max`, verify correctness
4. **Test NA handling**: Verify `na_rm` parameter works correctly
5. **Test materialization**: If using lazy materialization, verify:
   - Operations work before materialization
   - Dataptr triggers materialization
   - Values are correct after materialization

Example R test:

```r
test_that("my ALTREP works", {
  x <- my_altrep_constructor(10L)

  # Basic operations

  expect_equal(length(x), 10L)
  expect_equal(x[1], expected_first_element)
  expect_equal(x[10], expected_last_element)

  # Optimized methods
  expect_equal(sum(x), expected_sum)

  # Dataptr operations (triggers materialization for lazy types)
  y <- x + 0L
  expect_equal(y, expected_vector)
})
```

## Implementation Status

### Completed

- [x] Two-layer trait hierarchy (data traits + FFI traits)
- [x] Bridge macros for all ALTREP types
- [x] Built-in implementations for `Vec<T>`, `Range<T>`, `[T; N]`, `&'static [T]`
- [x] Proc-macro ALTREP class registration
- [x] `into_altrep()` instance creation
- [x] Trampoline generation with `HAS_*` gating
- [x] ExternalPtr integration for data storage
- [x] FFI bindings for all R ALTREP APIs

- [x] `Dataptr` with lazy materialization pattern
- [x] Auto-inferred base types for standard inner types
- [x] Serialization support (`AltrepSerialize` trait)
- [x] `Extract_subset` optimization (`AltrepExtractSubset` trait)
- [x] Complex number ALTREP example (`UnitCircle` - roots of unity)
- [x] Static slice support (`&'static [T]`) for const arrays and leaked data
- [x] Owned slice support (`Box<[T]>`) for fixed-size heap data
- [x] Rust unit tests for ALTREP data traits
- [x] R integration tests (testthat) for ALTREP functionality

### Not Yet Implemented

- [ ] Memory-mapped file ALTREP

## References

- R source: `src/main/altrep.c`, `src/main/altclasses.c`
- R headers: `src/include/R_ext/Altrep.h`
- Example packages: `simplemmap`, `mutable`
- Luke Tierney's ALTREP design document (DSC 2017)
