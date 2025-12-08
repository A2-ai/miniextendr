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

### ALTREP is Vector-Only

**ALTREP only supports vector types.** The available class constructors are:

| Function | SEXPTYPE | R Type |
|----------|----------|--------|
| `R_make_altinteger_class` | INTSXP | integer |
| `R_make_altreal_class` | REALSXP | double/numeric |
| `R_make_altlogical_class` | LGLSXP | logical |
| `R_make_altraw_class` | RAWSXP | raw |
| `R_make_altcomplex_class` | CPLXSXP | complex |
| `R_make_altstring_class` | STRSXP | character |
| `R_make_altlist_class` | VECSXP | list |

**NOT supported:** NILSXP, SYMSXP, LISTSXP (pairlist), CLOSXP, ENVSXP, PROMSXP, LANGSXP, EXPRSXP, etc.

### Required vs Optional Methods

From `altrep.c` defaults - methods that error if not provided vs those with safe defaults:

#### REQUIRED Methods (error if missing)

| Method | Type | Why Required |
|--------|------|--------------|
| `Length` | All | R cannot determine vector length without it |
| `Elt` | ALTSTRING | No default - `ALTREP_ERROR_IN_CLASS("No Elt method found")` |
| `Elt` | ALTLIST | No default - `ALTREP_ERROR_IN_CLASS("must provide an Elt method")` |
| `Set_elt` | ALTSTRING | No default if you want mutability |
| `Set_elt` | ALTLIST | No default if you want mutability |
| `Dataptr` | All | Errors if called without override (but may never be called) |

#### RECOMMENDED Methods (have defaults, but you likely want to override)

| Method | Default Behavior | Why Override |
|--------|------------------|--------------|
| `Elt` (numeric) | Falls back to `DATAPTR()[i]` | Avoid forcing materialization |
| `Get_region` | Loops over `Elt` | Bulk copy is faster |
| `Duplicate` | Returns NULL → standard copy | Keep ALTREP representation |
| `Serialized_state` | Returns NULL → expand & serialize | Preserve compact form |
| `Unserialize` | Errors | Required if you provide `Serialized_state` |

#### OPTIONAL Methods (safe defaults, override for optimization)

| Method | Default | When to Override |
|--------|---------|------------------|
| `Dataptr_or_null` | Returns NULL | If you have contiguous memory |
| `Is_sorted` | `UNKNOWN_SORTEDNESS` | If you know sortedness (e.g., Range) |
| `No_NA` | `0` (unknown) | If you guarantee no NAs |
| `Sum` | NULL → R loops | O(1) formula available (e.g., arithmetic series) |
| `Min/Max` | NULL → R loops | O(1) if sorted or computed |
| `Coerce` | NULL → standard coercion | Custom type conversion |
| `Inspect` | Returns FALSE → default output | Custom debug printing |
| `Extract_subset` | NULL → standard `[` | Optimized slicing |
| `DuplicateEX` | Calls `Duplicate` + handles attrs | Usually not needed |
| `UnserializeEX` | Calls `Unserialize` + restores attrs | Usually not needed |

#### Minimum Viable ALTREP by Type

**ALTINTEGER/ALTREAL/ALTLOGICAL/ALTRAW/ALTCOMPLEX:**

```rust
// Minimum: Length + (Elt OR Dataptr)
impl AltInteger for MyType {
    const HAS_LENGTH: bool = true;
    fn length(x: SEXP) -> R_xlen_t { ... }

    // Option A: Element access (lazy, no materialization)
    const HAS_ELT: bool = true;
    fn elt(x: SEXP, i: R_xlen_t) -> i32 { ... }

    // Option B: Data pointer (if contiguous memory available)
    // const HAS_DATAPTR: bool = true;
    // fn dataptr(x: SEXP, w: bool) -> *mut c_void { ... }
}
```

**ALTSTRING:**

```rust
// Minimum: Length + Elt (REQUIRED, no default!)
impl AltString for MyType {
    const HAS_LENGTH: bool = true;
    fn length(x: SEXP) -> R_xlen_t { ... }

    const HAS_ELT: bool = true;  // REQUIRED
    fn elt(x: SEXP, i: R_xlen_t) -> SEXP { ... }  // Return CHARSXP
}
```

**ALTLIST:**

```rust
// Minimum: Length + Elt (REQUIRED, no default!)
impl AltList for MyType {
    const HAS_LENGTH: bool = true;
    fn length(x: SEXP) -> R_xlen_t { ... }

    const HAS_ELT: bool = true;  // REQUIRED
    fn elt(x: SEXP, i: R_xlen_t) -> SEXP { ... }  // Return element SEXP
}
```

## R's ALTREP Usage Patterns

From `altclasses.c` and example packages:

### Pattern 1: Compact Sequences (`compact_intseq`, `compact_realseq`)

- **data1**: REALSXP with [n, start, inc]
- **data2**: R_NilValue initially, becomes expanded vector if needed
- **Length**: Returns n from data1
- **Elt**: Returns start + i * inc
- **Dataptr**: Expands to full vector on first call, stores in data2

### Pattern 2: Memory-mapped Files (`simplemmap`)

- **data1**: ExternalPtr to mmap address
- **data2**: LISTSXP state (file, size, flags)
- **Dataptr**: Returns mmap address if ptrOK
- **Elt/Get_region**: Read directly from mmap

### Pattern 3: Wrapper/Mutable (`mutable.c`)

- **data1**: The underlying R vector
- **data2**: R_NilValue
- **All methods**: Delegate to underlying vector
- **Duplicate**: Returns new wrapper sharing same data

### Pattern 4: Deferred String Coercion

- **data1**: Source vector (to be coerced)
- **data2**: STRSXP cache (or R_NilValue)
- **Elt**: Coerce on demand, cache result

## miniextendr ALTREP Design

### Trait Hierarchy (already in `altrep_traits.rs`)

```rust
pub trait Altrep {
    const HAS_LENGTH: bool = false;
    fn length(_x: SEXP) -> R_xlen_t { unreachable!() }
    // ... other base methods
}

pub trait AltVec: Altrep {
    const HAS_DATAPTR: bool = false;
    fn dataptr(_x: SEXP, _writable: bool) -> *mut c_void { unreachable!() }
    // ...
}

pub trait AltInteger: AltVec {
    const HAS_ELT: bool = false;
    fn elt(_x: SEXP, _i: R_xlen_t) -> i32 { unreachable!() }
    // ...
}
// ... AltReal, AltLogical, AltRaw, AltComplex, AltString, AltList
```

The `HAS_*` constants control which methods are installed. Default is `false`, meaning "use R's default behavior."

### Class Registration Pattern

```rust
// Generated by proc-macro for each ALTREP type
pub struct AltrepClass<T> {
    class: R_altrep_class_t,
    _marker: PhantomData<T>,
}

impl<T: AltInteger> AltrepClass<T> {
    pub fn register(class_name: &CStr, pkg_name: &CStr, dll: *mut DllInfo) -> Self {
        let class = unsafe { R_make_altinteger_class(class_name.as_ptr(), pkg_name.as_ptr(), dll) };

        // Install methods based on HAS_* constants
        if T::HAS_LENGTH {
            unsafe { R_set_altrep_Length_method(class, Some(altrep_length_trampoline::<T>)) };
        }
        if T::HAS_ELT {
            unsafe { R_set_altinteger_Elt_method(class, Some(altinteger_elt_trampoline::<T>)) };
        }
        // ... etc

        Self { class, _marker: PhantomData }
    }

    pub fn new_instance(&self, data1: SEXP, data2: SEXP) -> SEXP {
        unsafe { R_new_altrep(self.class, data1, data2) }
    }
}
```

### Trampoline Functions

```rust
unsafe extern "C-unwind" fn altrep_length_trampoline<T: Altrep>(x: SEXP) -> R_xlen_t {
    T::length(x)
}

unsafe extern "C-unwind" fn altinteger_elt_trampoline<T: AltInteger>(x: SEXP, i: R_xlen_t) -> i32 {
    T::elt(x, i)
}
```

### ALTREP + ExternalPtr Pattern

For wrapping Rust types as ALTREP vectors:

```rust
/// Wraps a Rust type T as an ALTREP integer vector
pub trait AltIntegerExt: Sized + 'static {
    fn len(&self) -> usize;
    fn get(&self, i: usize) -> Option<i32>;

    // Optional optimizations
    fn sum(&self) -> Option<i64> { None }
    fn is_sorted(&self) -> Option<Sorted> { None }
    fn has_na(&self) -> Option<bool> { None }
}

// Bridge implementation
impl<T: AltIntegerExt> Altrep for AltrepWrapper<T> {
    const HAS_LENGTH: bool = true;
    fn length(x: SEXP) -> R_xlen_t {
        let ptr = R_altrep_data1(x);
        let wrapper: &T = ExternalPtr::from_sexp(ptr).unwrap();
        wrapper.len() as R_xlen_t
    }
}

impl<T: AltIntegerExt> AltInteger for AltrepWrapper<T> {
    const HAS_ELT: bool = true;
    fn elt(x: SEXP, i: R_xlen_t) -> i32 {
        let ptr = R_altrep_data1(x);
        let wrapper: &T = ExternalPtr::from_sexp(ptr).unwrap();
        wrapper.get(i as usize).unwrap_or(i32::MIN) // NA_INTEGER
    }
}
```

## Rust Type → ALTREP Mapping

### Type Mapping Table

| Rust Type | ALTREP Class | Dataptr? | Notes |
|-----------|--------------|----------|-------|
| `Vec<i32>` | ALTINTEGER | Yes | Direct pointer access |
| `Vec<f64>` | ALTREAL | Yes | Direct pointer access |
| `Vec<bool>` | ALTLOGICAL | No | Must convert bool→i32 |
| `Vec<u8>` | ALTRAW | Yes | Direct pointer access |
| `Vec<Rcomplex>` | ALTCOMPLEX | Yes | Direct pointer access |
| `Vec<String>` | ALTSTRING | No | Convert to CHARSXP on demand |
| `Vec<SEXP>` | ALTLIST | No | Element access only |
| `[T; N]` | (same as Vec) | Yes | Fixed-size arrays |
| `Box<[T]>` | (same as Vec) | Yes | Owned slices |
| `Range<i32>` | ALTINTEGER | No | Compact sequence |
| `RangeInclusive<i32>` | ALTINTEGER | No | Compact sequence |
| `Range<i64>` | ALTINTEGER | No | Check overflow to i32 |
| `Iterator<Item=T>` | (by T) | No | Lazy, cache on Dataptr |
| `HashMap<K,V>` | ALTLIST | No | Named list |
| `BTreeMap<K,V>` | ALTLIST | No | Sorted named list |
| `Option<T>` | (by T) | (by T) | None → NA |
| `Result<T,E>` | (by T) | (by T) | Err → NA or error |
| Custom struct | ALTLIST | No | Fields as named elements |

### Detailed Type Mappings

#### ALTINTEGER Types

```rust
// Vec<i32> - Full featured, direct memory access
impl AltInteger for Vec<i32> {
    // HAS_DATAPTR = true (contiguous memory)
    // HAS_ELT = true
    // HAS_GET_REGION = true (efficient bulk copy)
    // HAS_NO_NA = false (can't know without scanning)
    // HAS_IS_SORTED = false (can't know without scanning)
}

// Range<i32> - Compact sequence (like R's 1:n)
impl AltInteger for Range<i32> {
    // HAS_DATAPTR = false (would force materialization!)
    // HAS_ELT = true: start + i (O(1))
    // HAS_LENGTH = true: end - start
    // HAS_IS_SORTED = true: always INCREASING (or DECREASING if step < 0)
    // HAS_NO_NA = true: ranges never contain NA
    // HAS_SUM = true: arithmetic series formula O(1)
    // HAS_MIN = true: start (or end-1 if decreasing)
    // HAS_MAX = true: end-1 (or start if decreasing)
}

// Option<i32> - NA-aware scalar (for iterators)
// None maps to NA_INTEGER (i32::MIN in R)
```

#### ALTREAL Types

```rust
// Vec<f64> - Full featured
impl AltReal for Vec<f64> {
    // Same as Vec<i32> but f64 element type
    // NA is NaN with specific bit pattern (R_NaReal)
}

// Linspace/arithmetic sequences
struct RealSeq { start: f64, step: f64, len: usize }
impl AltReal for RealSeq {
    // HAS_DATAPTR = false
    // HAS_ELT = true: start + i * step
    // HAS_SUM/MIN/MAX = true (O(1) formulas)
    // HAS_IS_SORTED = true (by step sign)
    // HAS_NO_NA = true (computed values, no NA)
}
```

#### ALTLOGICAL Types

```rust
// Vec<bool> - Requires conversion (bool is 1 byte, R logical is 4 bytes)
impl AltLogical for Vec<bool> {
    // HAS_DATAPTR = false (layout mismatch!)
    // HAS_ELT = true: vec[i] as i32 (true=1, false=0)
    // HAS_GET_REGION = true (with conversion loop)
    // HAS_NO_NA = true (Rust bool can't be NA)
}

// Vec<Option<bool>> - NA-aware logical
impl AltLogical for Vec<Option<bool>> {
    // HAS_DATAPTR = false
    // HAS_ELT = true: Some(true)=1, Some(false)=0, None=NA_LOGICAL
    // HAS_NO_NA = false (can contain NA)
}

// BitVec (from bitvec crate) - Compact storage
impl AltLogical for BitVec {
    // HAS_DATAPTR = false (bit-packed, not i32 array)
    // HAS_ELT = true: extract bit
    // HAS_NO_NA = true (bits can't be NA)
    // Memory: 1 bit per element vs 4 bytes in R
}
```

#### ALTRAW Types

```rust
// Vec<u8> - Direct memory access
impl AltRaw for Vec<u8> {
    // HAS_DATAPTR = true
    // HAS_ELT = true
    // HAS_GET_REGION = true
    // No NA concept for raw vectors
}

// &[u8], Bytes, etc. - borrowed data
// Requires lifetime management via ExternalPtr prevent-drop mechanism
```

#### ALTCOMPLEX Types

```rust
// Vec<Rcomplex> or Vec<num::Complex<f64>>
impl AltComplex for Vec<Rcomplex> {
    // HAS_DATAPTR = true (if Rcomplex layout matches)
    // HAS_ELT = true
    // HAS_GET_REGION = true
}

// Note: Rcomplex is { r: f64, i: f64 }
// num::Complex<f64> should have same layout (verify with #[repr(C)])
```

#### ALTSTRING Types

```rust
// Vec<String> - Deferred CHARSXP creation
impl AltString for Vec<String> {
    // HAS_DATAPTR = false (no contiguous CHARSXP array)
    // HAS_ELT = true: Rf_mkCharLenCE(s.as_ptr(), s.len(), CE_UTF8)
    // Consider caching CHARSXP results in data2
}

// Vec<&str> - Borrowed strings (lifetime concerns)
// Vec<Option<String>> - NA support (None → R_NaString)

// Lazy string generation
struct DeferredStrings<F: Fn(usize) -> String> {
    len: usize,
    generator: F,
    cache: RefCell<Vec<Option<SEXP>>>, // Cached CHARSXP values
}
```

#### ALTLIST Types

```rust
// Vec<SEXP> - Generic list
impl AltList for Vec<SEXP> {
    // HAS_ELT = true: vec[i]
    // HAS_SET_ELT = true: vec[i] = v (if mutable)
}

// HashMap<String, SEXP> - Named list
impl AltList for HashMap<String, SEXP> {
    // HAS_ELT = true: iterate to index (O(n) but unavoidable)
    // Names attribute: keys as STRSXP
    // Consider: store iteration order for O(1) access
}

// Struct as named list
#[derive(ExternalPtr)]
struct MyStruct {
    x: i32,      // $x -> ScalarInteger
    y: f64,      // $y -> ScalarReal
    z: String,   // $z -> ScalarString
}
// AltList length = 3, names = ["x", "y", "z"]
// Elt(0) = x as SEXP, Elt(1) = y as SEXP, etc.
```

### Dataptr Considerations

**When to provide Dataptr:**

- Type has contiguous memory layout matching R's expectation
- Memory is stable (won't move during R operations)
- Read-only access is safe without synchronization

**When NOT to provide Dataptr:**

- Layout mismatch (bool vs i32, bit-packed, etc.)
- Computed/lazy values (would force materialization)
- Non-contiguous storage (HashMap, etc.)
- Borrowed data with complex lifetime

**Dataptr fallback pattern:**

```rust
impl AltVec for MyLazyType {
    const HAS_DATAPTR: bool = true;
    const HAS_DATAPTR_OR_NULL: bool = true;

    fn dataptr(x: SEXP, _writable: bool) -> *mut c_void {
        // Check if already materialized
        let data2 = R_altrep_data2(x);
        if data2 != R_NilValue {
            return DATAPTR(data2);
        }
        // Materialize and cache
        let vec = materialize(x);
        R_set_altrep_data2(x, vec);
        DATAPTR(vec)
    }

    fn dataptr_or_null(x: SEXP) -> *const c_void {
        let data2 = R_altrep_data2(x);
        if data2 != R_NilValue {
            DATAPTR_RO(data2)
        } else {
            std::ptr::null() // Don't materialize
        }
    }
}
```

### NA Handling by Type

| R Type | NA Value | Rust Equivalent |
|--------|----------|-----------------|
| INTEGER | `NA_INTEGER` (i32::MIN) | `Option<i32>`, check for MIN |
| REAL | `R_NaReal` (special NaN) | `Option<f64>`, f64::is_nan() |
| LOGICAL | `NA_LOGICAL` (i32::MIN) | `Option<bool>` |
| STRING | `R_NaString` (SEXP) | `Option<String>` |
| COMPLEX | r=R_NaReal, i=R_NaReal | `Option<Complex<f64>>` |
| RAW | No NA | `u8` directly |
| LIST | Element can be anything | `Option<SEXP>` for NULL |

### C NULL vs R_NilValue Return Semantics

**Critical distinction**: ALTREP methods use C `NULL` and R's `R_NilValue` for different purposes.

#### Summary Table

| Method | Return `NULL` | Return `R_NilValue` | Return actual SEXP |
|--------|---------------|---------------------|-------------------|
| `Serialized_state` | Use default serialization (expand to regular vector) | State is empty/nil but ALTREP-serialize | Serialize with this state |
| `Duplicate` | Use default duplication | Return nil object (unusual) | Return duplicated object |
| `Coerce` | Use default coercion | Coercion result is nil | Return coerced object |
| `Sum/Min/Max` | Use default implementation | Result is NA/nil | Return computed result |
| `Extract_subset` | Use default subsetting | Empty result | Return subset |
| `Dataptr_or_null` | No pointer available (don't materialize) | — | Return data pointer |
| `Elt` (list/string) | — | Element is NULL/missing | Return element |

#### Detailed Semantics (from `altrep.c`)

**`Serialized_state` → NULL means "serialize as regular vector"**

```c
// serialize.c:1053-1054
SEXP state = ALTREP_SERIALIZED_STATE(s);
if (info != NULL && state != NULL) {
    // Serialize as ALTREP with state
} else {
    // Fall through: serialize as regular vector (expands ALTREP)
}
```

- Return `NULL`: ALTREP is expanded to regular vector, then serialized normally
- Return `R_NilValue`: Serialized as ALTREP with nil state (can reconstruct)
- Return SEXP: Serialized as ALTREP with given state

**`Duplicate` → NULL means "use default duplication"**

```c
// altrep.c:702-706
SEXP ans = ALTREP_DUPLICATE(x, deep);
if (ans != NULL && ans != x) {
    // Handle attributes on the duplicate
}
// If NULL, R falls back to standard vector duplication
```

- Return `NULL`: R creates a standard (non-ALTREP) copy
- Return original `x`: Share the object (no copy made)
- Return new SEXP: Use this as the duplicate

**`Coerce` → NULL means "use default coercion"**

```c
// Default: return NULL
static SEXP altrep_Coerce_default(SEXP x, int type) { return NULL; }
```

- Return `NULL`: R's standard coercion applies
- Return SEXP: Use this coerced value

**`Dataptr_or_null` → NULL means "don't materialize"**

```c
// Used by INTEGER_OR_NULL, REAL_OR_NULL, etc.
const int *x = INTEGER_OR_NULL(sx);
if (x != NULL) {
    // Fast path: direct memory access
} else {
    // Slow path: use ALTINTEGER_ELT per-element
}
```

- Return `NULL`: Caller must use Elt/Get_region (no pointer available)
- Return pointer: Caller can use direct memory access

**`Sum/Min/Max` → NULL means "use default implementation"**

```c
static SEXP altinteger_Sum_default(SEXP x, Rboolean narm) { return NULL; }
```

- Return `NULL`: R computes using standard loop over elements
- Return SEXP: Use this pre-computed result (optimization)

**`Extract_subset` → NULL means "use default subsetting"**

```c
static SEXP altvec_Extract_subset_default(SEXP x, SEXP indx, SEXP call) {
    return NULL;
}
```

- Return `NULL`: R performs standard `[` subsetting
- Return SEXP: Use this optimized subset

#### Rust Implementation Pattern

```rust
// Method that can opt-out (return NULL for default behavior)
const HAS_SUM: bool = true;
fn sum(x: SEXP, narm: bool) -> SEXP {
    let this = self_from(x);

    // Can compute efficiently?
    if let Some(result) = this.try_fast_sum(narm) {
        return Rf_ScalarReal(result);  // Return computed value
    }

    // Can't optimize: return NULL for R's default
    std::ptr::null_mut()  // C NULL, not R_NilValue!
}

// Method that returns an R value (R_NilValue is valid)
const HAS_ELT: bool = true;
fn elt(x: SEXP, i: R_xlen_t) -> SEXP {
    let this = self_from(x);

    match this.get(i as usize) {
        Some(value) => value.to_sexp(),
        None => R_NilValue,  // Valid "element is nil" return
    }
}
```

#### Common Mistakes

1. **Returning R_NilValue when you mean "no override"**

   ```rust
   // WRONG: This says "the serialized state is nil"
   fn serialized_state(x: SEXP) -> SEXP { R_NilValue }

   // RIGHT: This says "use default serialization"
   fn serialized_state(x: SEXP) -> SEXP { std::ptr::null_mut() }
   ```

2. **Returning NULL from Elt methods**

   ```rust
   // WRONG for ALTSTRING/ALTLIST Elt: NULL is not a valid element
   fn elt(x: SEXP, i: R_xlen_t) -> SEXP { std::ptr::null_mut() }

   // RIGHT: R_NilValue represents a nil/missing element
   fn elt(x: SEXP, i: R_xlen_t) -> SEXP { R_NilValue }
   ```

3. **Forgetting that Dataptr_or_null returning NULL is expected**

   ```rust
   // This is CORRECT - NULL means "use Elt instead"
   fn dataptr_or_null(x: SEXP) -> *const c_void {
       std::ptr::null()  // No pointer available, use element access
   }
   ```

#### Consequence Summary

| Return Value | Meaning | R's Behavior |
|--------------|---------|--------------|
| C `NULL` from optional method | "I don't handle this" | Use default/fallback implementation |
| C `NULL` from Dataptr_or_null | "No pointer, use Elt" | Element-by-element access |
| `R_NilValue` from Elt | "Element is nil/NULL" | Element value is R's NULL |
| `R_NilValue` from state method | "State is empty but valid" | Serialize as ALTREP with nil state |

### Optimization Opportunities

| Method | Rust Type | Optimization |
|--------|-----------|--------------|
| `Sum` | `Range<i32>` | `n * (start + end - 1) / 2` O(1) |
| `Sum` | `Vec<i32>` | SIMD via `iter().sum()` |
| `Min/Max` | `Range<i32>` | `start` or `end-1` O(1) |
| `Min/Max` | sorted `Vec` | First/last element O(1) |
| `Is_sorted` | `Range` | Always sorted |
| `Is_sorted` | `BTreeSet` | Always sorted |
| `No_NA` | `Range`, `Vec<bool>` | Always true |
| `Get_region` | `Vec<T>` | memcpy for contiguous |
| `Extract_subset` | `Range` | Return new Range if contiguous |

### Composite Types (Structs & Enums)

#### Decision: ALTREP vs ExternalPtr

Not every Rust type should be ALTREP. Use this guide:

| Scenario | Approach | Why |
|----------|----------|-----|
| Type represents vector data | ALTREP | R sees it as a native vector |
| Type is opaque handle | ExternalPtr only | No vector semantics |
| Type has one "main" vector field | ALTREP (delegate) | Expose the vector, hide metadata |
| Type is a "record" (row of data) | ExternalPtr or ALTLIST | Not a vector |
| Type is a collection of records | ALTREP per column | Data frame pattern |

#### Pattern 1: Single-Field Delegation

Struct wraps a vector with metadata. Expose the vector via ALTREP:

```rust
struct TaggedIntegers {
    tag: String,           // metadata, not exposed
    data: Vec<i32>,        // the "real" data
    sorted: bool,          // cached property
}

// ALTREP delegates to `data` field
impl AltInteger for TaggedIntegers {
    const HAS_LENGTH: bool = true;
    fn length(x: SEXP) -> R_xlen_t { self_from(x).data.len() as R_xlen_t }

    const HAS_ELT: bool = true;
    fn elt(x: SEXP, i: R_xlen_t) -> i32 { self_from(x).data[i as usize] }

    const HAS_DATAPTR: bool = true;
    fn dataptr(x: SEXP, _w: bool) -> *mut c_void {
        self_from(x).data.as_ptr() as *mut c_void
    }

    const HAS_IS_SORTED: bool = true;
    fn is_sorted(x: SEXP) -> i32 {
        if self_from(x).sorted { SORTED_INCR } else { UNKNOWN_SORTEDNESS }
    }
}
```

#### Pattern 2: Struct as Named ALTLIST

Struct fields become named list elements:

```rust
#[derive(ExternalPtr)]
#[miniextendr(altrep = "list")]
struct Person {
    name: String,    // $name -> character(1)
    age: i32,        // $age -> integer(1)
    scores: Vec<f64>, // $scores -> numeric vector
}

// Generated ALTREP implementation:
impl AltList for Person {
    const HAS_LENGTH: bool = true;
    fn length(_x: SEXP) -> R_xlen_t { 3 } // number of fields

    const HAS_ELT: bool = true;
    fn elt(x: SEXP, i: R_xlen_t) -> SEXP {
        let this = self_from(x);
        match i {
            0 => Rf_mkString(this.name.as_ptr()),
            1 => Rf_ScalarInteger(this.age),
            2 => this.scores.to_sexp(), // Vec<f64> -> REALSXP
            _ => R_NilValue,
        }
    }
}
// Plus names attribute: c("name", "age", "scores")
```

R usage: `person$name`, `person$age`, `person$scores`

#### Pattern 3: Vec<Struct> as Columnar ALTREP

Multiple structs → expose columns as ALTREP vectors:

```rust
struct Record { x: i32, y: f64, label: String }

struct DataFrame {
    records: Vec<Record>,
}

// Instead of one ALTREP, generate column accessors:
impl DataFrame {
    fn column_x(&self) -> impl AltInteger {
        ColumnView { data: &self.records, extract: |r| r.x }
    }
    fn column_y(&self) -> impl AltReal {
        ColumnView { data: &self.records, extract: |r| r.y }
    }
}

// Each column is lazy ALTREP - no data copying!
struct ColumnView<'a, T, F> {
    data: &'a [Record],
    extract: F,
}

impl<F: Fn(&Record) -> i32> AltInteger for ColumnView<'_, Record, F> {
    const HAS_ELT: bool = true;
    fn elt(x: SEXP, i: R_xlen_t) -> i32 {
        let this = self_from(x);
        (this.extract)(&this.data[i as usize])
    }
    // HAS_DATAPTR = false (not contiguous in memory)
}
```

#### Pattern 4: Enum as Tagged Union

```rust
enum Value {
    Int(i32),
    Real(f64),
    Text(String),
    Missing,
}

// Option 1: As ALTLIST where type varies by element
impl AltList for Vec<Value> {
    fn elt(x: SEXP, i: R_xlen_t) -> SEXP {
        match &self_from(x)[i as usize] {
            Value::Int(n) => Rf_ScalarInteger(*n),
            Value::Real(r) => Rf_ScalarReal(*r),
            Value::Text(s) => Rf_mkString(s.as_ptr()),
            Value::Missing => R_NilValue,
        }
    }
}

// Option 2: Homogeneous enum variants as typed ALTREP
enum IntOrNA { Value(i32), NA }
impl AltInteger for Vec<IntOrNA> {
    fn elt(x: SEXP, i: R_xlen_t) -> i32 {
        match self_from(x)[i as usize] {
            IntOrNA::Value(n) => n,
            IntOrNA::NA => NA_INTEGER,
        }
    }
}
```

#### Pattern 5: Nested Structures

```rust
struct Outer {
    metadata: Metadata,
    inner: Inner,
}

struct Inner {
    values: Vec<i32>,
}

// Options:
// A) Flatten: Outer exposes inner.values as ALTINTEGER
// B) Nest: Outer as ALTLIST with $metadata and $inner sub-lists
// C) Delegate: #[altrep(delegate)] on inner field
```

#### When NOT to Use ALTREP for Composites

1. **Opaque handles**: Database connections, file handles, thread pools
   - Use ExternalPtr only, no ALTREP
   - R shouldn't try to index/iterate

2. **Mutable state with invariants**:
   - ALTREP Elt/Dataptr bypass your API
   - Use ExternalPtr + explicit getter methods

3. **Complex lifetimes**:
   - Borrowed data (`&'a T`) in struct fields
   - ALTREP callbacks can't express lifetimes
   - Either own the data or use careful prevent-drop

4. **Non-vector semantics**:
   - Trees, graphs, state machines
   - Length/Elt don't make sense
   - ExternalPtr + method dispatch

#### Composition Summary

```
┌─────────────────────────────────────────────────────────────┐
│                    Composite Type                            │
├─────────────────────────────────────────────────────────────┤
│ Has single vector field?                                     │
│   YES → ALTREP delegates to that field                      │
│   NO  ↓                                                     │
├─────────────────────────────────────────────────────────────┤
│ Fields are independently useful?                             │
│   YES → ALTLIST with named elements                         │
│   NO  ↓                                                     │
├─────────────────────────────────────────────────────────────┤
│ Is Vec<Struct>?                                              │
│   YES → Columnar ALTREP (one per field)                     │
│   NO  ↓                                                     │
├─────────────────────────────────────────────────────────────┤
│ Opaque/stateful/complex?                                     │
│   YES → ExternalPtr only (no ALTREP)                        │
└─────────────────────────────────────────────────────────────┘
```

## Compile-Time Enforcement of Required Methods

The current `altrep_traits.rs` design with `HAS_*` constants is **insufficient** - it doesn't prevent users from forgetting required methods. The API should enforce correctness at compile time.

### Problem with Current Design

```rust
// Current: User can forget required methods, only fails at runtime
impl AltString for MyType {
    const HAS_LENGTH: bool = true;
    fn length(x: SEXP) -> R_xlen_t { 10 }
    // Forgot HAS_ELT and elt() - compiles fine, crashes at runtime!
}
```

### Solution 1: Builder Pattern with Type-State

```rust
// Type-state markers
struct NeedsLength;
struct NeedsElt;
struct Ready;

struct AltStringBuilder<State> {
    class_name: &'static str,
    pkg_name: &'static str,
    length_fn: Option<fn(SEXP) -> R_xlen_t>,
    elt_fn: Option<fn(SEXP, R_xlen_t) -> SEXP>,
    // ... optional methods
    _state: PhantomData<State>,
}

impl AltStringBuilder<NeedsLength> {
    pub fn new(name: &'static str, pkg: &'static str) -> Self { ... }

    // Must provide length - transitions state
    pub fn length(self, f: fn(SEXP) -> R_xlen_t) -> AltStringBuilder<NeedsElt> {
        AltStringBuilder {
            length_fn: Some(f),
            _state: PhantomData,
            ..self
        }
    }
}

impl AltStringBuilder<NeedsElt> {
    // Must provide elt - transitions to Ready
    pub fn elt(self, f: fn(SEXP, R_xlen_t) -> SEXP) -> AltStringBuilder<Ready> {
        AltStringBuilder {
            elt_fn: Some(f),
            _state: PhantomData,
            ..self
        }
    }
}

impl AltStringBuilder<Ready> {
    // Optional methods available in Ready state
    pub fn set_elt(mut self, f: fn(SEXP, R_xlen_t, SEXP)) -> Self { ... }
    pub fn is_sorted(mut self, f: fn(SEXP) -> i32) -> Self { ... }

    // Only Ready state can build!
    pub fn build(self, dll: *mut DllInfo) -> AltrepClass { ... }
}

// Usage - compile error if you forget required methods:
let class = AltStringBuilder::new("my_strings", "mypkg")
    .length(|x| my_len(x))
    .elt(|x, i| my_elt(x, i))      // Required!
    .is_sorted(|x| UNKNOWN)        // Optional
    .build(dll);

// This won't compile - can't call .build() without .elt():
// let class = AltStringBuilder::new("bad", "pkg")
//     .length(|x| 10)
//     .build(dll);  // ERROR: no method `build` on AltStringBuilder<NeedsElt>
```

### Solution 2: Separate Required + Optional Traits (Recommended)

The key insight: **C NULL tells R "use your default implementation"**. Optional methods should return NULL by default, which R interprets as "not overridden."

```rust
// Required methods - NO defaults, compiler enforces implementation
trait AltStringCore {
    fn length(x: SEXP) -> R_xlen_t;  // Must implement
    fn elt(x: SEXP, i: R_xlen_t) -> SEXP;  // Must implement
}

// Optional methods - defaults return NULL (R uses its fallback)
trait AltStringOpt: AltStringCore {
    // Returns NULL → R falls back to error (read-only string)
    fn set_elt(_x: SEXP, _i: R_xlen_t, _v: SEXP) -> *mut SEXPREC {
        std::ptr::null_mut()  // NULL = not implemented
    }

    // Returns NULL → R falls back to UNKNOWN_SORTEDNESS
    fn is_sorted(_x: SEXP) -> *mut SEXPREC {
        std::ptr::null_mut()
    }

    // Returns NULL → R falls back to "unknown" (0)
    fn no_na(_x: SEXP) -> *mut SEXPREC {
        std::ptr::null_mut()
    }
}

// For methods with non-pointer returns, use Option or sentinel
trait AltIntegerOpt: AltIntegerCore {
    // Option<i32> → None maps to NULL in registration
    fn sum(_x: SEXP, _narm: bool) -> Option<SEXP> { None }
    fn min(_x: SEXP, _narm: bool) -> Option<SEXP> { None }
    fn max(_x: SEXP, _narm: bool) -> Option<SEXP> { None }
}

// Blanket impl: anything implementing Core gets default Opt
impl<T: AltStringCore> AltStringOpt for T {}

// Registration checks for NULL and skips R_set_* calls
fn register_altstring<T: AltStringOpt>(
    name: &CStr,
    pkg: &CStr,
    dll: *mut DllInfo
) -> AltrepClass {
    let class = R_make_altstring_class(...);

    // Required methods - always install
    R_set_altrep_Length_method(class, Some(length_trampoline::<T>));
    R_set_altstring_Elt_method(class, Some(elt_trampoline::<T>));

    // Optional methods - trampoline handles NULL return
    // The trampoline checks if T's method returns null and returns appropriately
    R_set_altstring_Set_elt_method(class, Some(set_elt_trampoline::<T>));
    R_set_altstring_Is_sorted_method(class, Some(is_sorted_trampoline::<T>));

    class
}

// Usage - compiler enforces required methods:
struct MyStrings(Vec<String>);

impl AltStringCore for MyStrings {
    fn length(x: SEXP) -> R_xlen_t { /* must implement */ }
    fn elt(x: SEXP, i: R_xlen_t) -> SEXP { /* must implement */ }
}
// AltStringOpt auto-implemented with NULL-returning defaults
// User can override specific optional methods:
impl AltStringOpt for MyStrings {
    fn is_sorted(_x: SEXP) -> *mut SEXPREC {
        // Actually implement sorting check
        Rf_ScalarInteger(SORTED_INCR)
    }
    // set_elt and no_na remain default (NULL → use R's behavior)
}
```

#### Alternative: Const-based Method Gating

Instead of NULL returns, use const bools to control whether methods are installed:

```rust
trait AltStringOpt: AltStringCore {
    // Const flags control registration
    const HAS_SET_ELT: bool = false;
    const HAS_IS_SORTED: bool = false;
    const HAS_NO_NA: bool = false;

    // Methods only called if const is true
    fn set_elt(_x: SEXP, _i: R_xlen_t, _v: SEXP) { unreachable!() }
    fn is_sorted(_x: SEXP) -> i32 { unreachable!() }
    fn no_na(_x: SEXP) -> i32 { unreachable!() }
}

// Registration checks consts
fn register<T: AltStringOpt>(...) {
    // Required - always install
    R_set_altrep_Length_method(class, Some(length_trampoline::<T>));
    R_set_altstring_Elt_method(class, Some(elt_trampoline::<T>));

    // Optional - only install if implemented
    if T::HAS_SET_ELT {
        R_set_altstring_Set_elt_method(class, Some(set_elt_trampoline::<T>));
    }
    // Not installing = R uses its default (which is NULL in the method table)
}
```

This is cleaner because:

1. **No runtime cost**: Method not installed means R never calls it
2. **Clear intent**: `HAS_*` explicitly declares what's implemented
3. **Proc-macro friendly**: Macro can generate `const HAS_* = true` for implemented methods

### Solution 3: Per-Type Constructors

Different constructors for each ALTREP type, each requiring their specific methods:

```rust
// For numeric types (ALTINTEGER/ALTREAL/etc) - Elt OR Dataptr required
pub fn register_altinteger<T>(
    name: &CStr,
    pkg: &CStr,
    dll: *mut DllInfo,
    length: fn(SEXP) -> R_xlen_t,
    access: IntegerAccess,  // Enum: either Elt or Dataptr
) -> AltrepClass;

pub enum IntegerAccess {
    Elt(fn(SEXP, R_xlen_t) -> i32),
    Dataptr(fn(SEXP, bool) -> *mut c_void),
    Both {
        elt: fn(SEXP, R_xlen_t) -> i32,
        dataptr: fn(SEXP, bool) -> *mut c_void,
    },
}

// For ALTSTRING/ALTLIST - Elt is always required
pub fn register_altstring<T>(
    name: &CStr,
    pkg: &CStr,
    dll: *mut DllInfo,
    length: fn(SEXP) -> R_xlen_t,
    elt: fn(SEXP, R_xlen_t) -> SEXP,  // Required parameter!
) -> AltStringBuilder;  // Returns builder for optional methods

// Usage:
let class = register_altstring(
    c"my_strings",
    c"mypkg",
    dll,
    my_length,
    my_elt,
)
.set_elt(my_set_elt)  // Optional
.build();
```

### Recommended Approach

Combine **Solution 1 (type-state builder)** for explicit construction with **Solution 2 (split traits)** for derive macros:

```rust
// Manual construction: type-state builder enforces requirements
let class = AltIntegerBuilder::new("compact_seq", "mypkg")
    .length(compact_length)
    .elt(compact_elt)          // Required for lazy
    // OR .dataptr(compact_dataptr) for contiguous
    .sum(compact_sum)          // Optional optimization
    .build(dll);

// Derive macro: trait split enforces at impl level
#[derive(AltInteger)]
struct CompactSeq { start: i32, end: i32 }

impl AltIntegerCore for CompactSeq {
    fn length(&self) -> usize { (self.end - self.start) as usize }
    fn elt(&self, i: usize) -> i32 { self.start + i as i32 }
}
// Optional: impl AltIntegerOpt to override defaults
```

### Type-Specific Requirements Summary

| Type | Required | Pick One | Optional |
|------|----------|----------|----------|
| ALTINTEGER | `length` | `elt` OR `dataptr` | `get_region`, `sum`, `min`, `max`, `is_sorted`, `no_na` |
| ALTREAL | `length` | `elt` OR `dataptr` | `get_region`, `sum`, `min`, `max`, `is_sorted`, `no_na` |
| ALTLOGICAL | `length` | `elt` OR `dataptr` | `get_region`, `sum`, `is_sorted`, `no_na` |
| ALTRAW | `length` | `elt` OR `dataptr` | `get_region` |
| ALTCOMPLEX | `length` | `elt` OR `dataptr` | `get_region` |
| ALTSTRING | `length`, `elt` | — | `set_elt`, `is_sorted`, `no_na` |
| ALTLIST | `length`, `elt` | — | `set_elt` |

## Proc-Macro Composition

The `#[derive(ExternalPtr)]` macro should generate ALTREP support:

```rust
#[derive(ExternalPtr)]
#[miniextendr(altrep = "list")]  // Generate as ALTLIST
struct MyData {
    #[altrep(length)]
    values: Vec<i32>,           // This field provides length

    #[altrep(elt)]
    fn get_element(&self, i: usize) -> SEXP { ... }
}
```

### Composition Strategy

For structs with multiple fields, the macro could:

1. **Single-field delegation**: If one field is marked `#[altrep(delegate)]`, delegate all methods to that field's ALTREP impl

2. **Multi-field composition**: Generate a custom ALTREP that combines fields:
   - Length: Sum of field lengths, or user-specified
   - Elt: Route to appropriate field based on index ranges

3. **Custom methods**: Allow `#[altrep(method_name)]` on methods to override specific ALTREP methods

## Implementation Phases

### Phase 1: Core Infrastructure

- [x] ALTREP trait hierarchy (`altrep_traits.rs`)
- [ ] FFI bindings for R_make_alt**class, R_set** methods
- [ ] Trampoline generation (const generic dispatch)
- [ ] AltrepClass registration wrapper

### Phase 2: Primitive ALTREP Types

- [ ] AltInteger for `Vec<i32>`
- [ ] AltReal for `Vec<f64>`
- [ ] AltLogical for `Vec<bool>`
- [ ] AltRaw for `Vec<u8>`
- [ ] AltString for `Vec<String>`

### Phase 3: Advanced Types

- [ ] Range<i32> as compact integer sequence
- [ ] Lazy/deferred coercion
- [ ] Memory-mapped vectors (via memmap2 crate)

### Phase 4: Proc-Macro Integration

- [ ] `#[derive(Altrep)]` macro
- [ ] Field delegation
- [ ] Method composition
- [ ] ExternalPtr + ALTREP integration

## Key Design Decisions

1. **Const generics for method dispatch**: Use `HAS_*` constants to avoid installing NULL methods

2. **ExternalPtr in data1**: Store Rust data as ExternalPtr with proper Drop handling

3. **State in data2**: Use for caching materialized vectors or metadata

4. **Thread safety**: ALTREP callbacks may be called from any R thread; ensure Send/Sync bounds where needed

5. **Panic safety**: Trampolines must catch panics and convert to R errors

## References

- R source: `src/main/altrep.c`, `src/main/altclasses.c`
- R headers: `src/include/R_ext/Altrep.h`
- Example packages: `simplemmap`, `mutable`
- Luke Tierney's ALTREP design document (DSC 2017)
