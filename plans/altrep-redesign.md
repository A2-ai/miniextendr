# ALTREP Redesign

## Goal

Redesign the ALTREP system so that:

1. One struct + one derive = complete ALTREP class (no wrapper struct, no macro calls)
2. The storage strategy (ExternalPtr vs native SEXP vs computed) is explicit and pluggable
3. The trait system matches R's actual runtime dispatch model
4. The derive macro generates everything: registration, bridging, IntoR, TryFromSexp

## R's ALTREP: What Actually Exists

From `altrep.c` and `Altrep.h`, an ALTREP object is:

- A normal SEXP (INTSXP, REALSXP, etc.) with the ALTREP bit set
- TAG field points to the class object
- Class object = RAWSXP whose DATAPTR holds a method struct (vtable)
- Two data slots: `R_altrep_data1(x)` and `R_altrep_data2(x)` — both SEXP, free-form

**7 ALTREP families** (one per vector SEXPTYPE):

| Family | SEXPTYPE | Element C type | Required methods |
|--------|----------|---------------|------------------|
| ALTINTEGER | INTSXP | `int` | Length |
| ALTREAL | REALSXP | `double` | Length |
| ALTLOGICAL | LGLSXP | `int` | Length |
| ALTRAW | RAWSXP | `Rbyte` | Length |
| ALTCOMPLEX | CPLXSXP | `Rcomplex` | Length |
| ALTSTRING | STRSXP | `SEXP` (CHARSXP) | Length, Elt |
| ALTLIST | VECSXP | `SEXP` | Length, Elt |

**Method tables** (from the C macros in altrep.c):

| Level | Methods | Notes |
|-------|---------|-------|
| altrep (base) | Length, Serialized_state, Unserialize, UnserializeEX, Duplicate, DuplicateEX, Coerce, Inspect | All have defaults |
| altvec | Dataptr, Dataptr_or_null, Extract_subset | Dataptr default = error; the rest = NULL |
| altinteger | Elt, Get_region, Is_sorted, No_NA, Sum, Min, Max | Elt default = `INTEGER(x)[i]` (materializes!) |
| altreal | Elt, Get_region, Is_sorted, No_NA, Sum, Min, Max | Elt default = `REAL(x)[i]` |
| altlogical | Elt, Get_region, Is_sorted, No_NA, Sum | No Min/Max in R's API |
| altraw | Elt, Get_region | Minimal |
| altcomplex | Elt, Get_region | Minimal |
| altstring | Elt, Set_elt, Is_sorted, No_NA | Elt **required** (default = error) |
| altlist | Elt, Set_elt | Elt **required** (default = error) |

Key R behaviors:

- **Elt default for numeric types falls back to DATAPTR** — `INTEGER(x)[i]` calls `DATAPTR(x)` which calls the Dataptr method. If neither Elt nor Dataptr is set, you get "cannot access data pointer" error.
- **Get_region default iterates Elt** — so providing Elt is sufficient.
- **Sum/Min/Max default returns NULL** — R falls back to its standard implementation.
- **Dataptr_or_null default returns NULL** — R falls back to element-wise access.
- **Extract_subset default returns NULL** — R falls back to standard subsetting.
- **Serialized_state default returns NULL** — R uses standard serialization (materializes).
- **GC is disabled during Dataptr, String Elt, and List Elt calls** — R sets `R_GCEnabled = FALSE`.

## User Stories

### Owned container types (Vec, Box<[T]>)

**US-01: Vec<i32> as ALTREP integer**
User has a `Vec<i32>` computed in Rust and wants to return it to R as an integer vector
without copying. The Vec is stored in an ExternalPtr in data1. Dataptr returns the Vec's
internal pointer directly. Elt indexes into the Vec. Get_region does a memcpy from the
Vec's buffer. This is zero-copy for contiguous access.

**US-02: Vec<f64> as ALTREP real**
Same as US-01 but for doubles. Identical pattern.

**US-03: Vec<Option<String>> as ALTREP string**
User has strings with possible NAs. Elt converts each `Option<&str>` to CHARSXP
(or NA_STRING for None). No dataptr possible for strings (STRSXP elements are CHARSXP
pointers, not contiguous chars). Materialization stores expanded STRSXP in data2.

**US-04: Vec<u8> as ALTREP raw**
Byte vectors. Same zero-copy pattern as US-01.

**US-05: Vec<bool> as ALTREP logical**
Bools are not R-native (R uses i32 for logicals). Elt converts bool→i32. No direct
dataptr (layout mismatch). Get_region converts element-by-element.

**US-06: Vec<Rcomplex> as ALTREP complex**
Complex numbers. Direct dataptr works since Rcomplex has the same layout.

**US-07: Box<[i32]> as ALTREP integer**
Same as Vec but immutable (no reallocation). Dataptr can still return the pointer.
Useful for frozen data that came from deserialization or FFI.

### Computed/lazy sequences

**US-08: Arithmetic sequence (start, step, len)**
Like R's `1:1000000`. No storage — just three numbers. Elt computes `start + step * i`.
Length returns `len`. Is_sorted returns Increasing or Decreasing based on step sign.
No_NA returns true. Sum can be computed in O(1) with the arithmetic series formula.
Min/Max are just the endpoints. This is the compact_intseq pattern from R's own altclasses.c.

**US-09: Constant fill (value, len)**
`rep(42L, 1e9)`. Elt always returns the same value. Length is the stored len.
Sum = value * len. Min = Max = value. No_NA = (value != NA). Extremely compact.

**US-10: Fibonacci sequence (len)**
Computed on demand. Elt computes fib(i). No dataptr. Get_region computes a range.
Not sorted in the traditional sense but could declare sorted if the sequence is monotonic.

**US-11: Lazy real sequence with memoization**
Like US-08 but for reals, with a cache. First access computes; subsequent accesses
hit cache. Dataptr triggers full materialization into data2. Dataptr_or_null returns
NULL until materialized (forcing element-wise access via Elt).

**US-12: Random number generator (seed, len)**
Deterministic PRNG. Elt computes the i-th random number from the seed.
Not sorted. Has NAs: no. Serializable (just save seed + len).

### Views and wrappers

**US-13: Sorted-wrapper (wraps existing R vector + metadata)**
Like R's own `wrap_integer` class. data1 = the wrapped INTSXP. data2 = metadata
(sorted flag, no_na flag). All element access delegates to the inner vector.
Is_sorted and No_NA return the cached metadata. This is the only ALTREP class
where data1 is a plain R vector, not an ExternalPtr.

**US-14: Reversed view**
Wraps an existing integer vector and reverses index access. Elt(i) returns
inner[len-1-i]. Length delegates. Is_sorted flips. No storage duplication.
Could wrap either a native R vector or another ALTREP.

**US-15: Slice/window view**
View into a subrange [start, end) of an existing vector. Elt(i) returns
inner[start + i]. Length = end - start. Dataptr returns inner_ptr + start
(if inner has a dataptr). Extract_subset composes indices.

**US-16: Filtered view (indices)**
Holds an index vector + original vector. Elt(i) returns original[indices[i]].
Length = len(indices). No contiguous dataptr. Extract_subset can compose.

**US-17: Column view from a matrix**
View into column j of a matrix. Elt(i) returns matrix[i + j*nrow].
Dataptr returns matrix_ptr + j*nrow (contiguous column in column-major R).

### External data sources

**US-18: Memory-mapped file**
Like R's own mmap_integer class. ExternalPtr holds mmap handle + metadata.
Dataptr returns the mmap'd pointer. Elt indexes into mmap. Serialization
can save the file path. Finalization calls munmap.

**US-19: Arrow array (zero-copy)**
Arrow arrays have a data buffer + optional validity bitmap. For non-null
arrays, dataptr returns the data buffer directly. For nullable arrays,
dataptr returns NULL (no contiguous NA-aware representation), Elt checks
the validity bit and returns NA or the value. Get_region can be optimized
to check null bitmap in bulk.

**US-20: Database cursor / streaming results**
A lazy ALTREP that fetches rows on demand from a database connection.
Elt(i) might trigger a batch fetch. Length could be known upfront or
require a COUNT query. Not serializable (connection is ephemeral).
Dataptr triggers full materialization.

**US-21: Parquet column reader**
Reads a single column from a Parquet file. Length known from metadata.
Elt reads and decompresses the relevant page. Get_region does bulk
decompression. Serialization saves file path + column index.

**US-22: Network/REST-backed vector**
Lazy vector backed by a remote API. Elt triggers HTTP request for a
page of data. Aggressive caching in data2. Not practically useful
for random access but great for sequential iteration via Get_region.

### Transformations and pipelines

**US-23: Map/transform (f: T → U, inner: ALTREP)**
Applies a function to each element of an inner ALTREP. Elt(i) = f(inner.elt(i)).
Length delegates. Composes lazily. Could chain: `map(|x| x*2, map(|x| x+1, base))`.

**US-24: Cumulative sum**
Wraps an integer/real ALTREP. Elt(i) computes sum of elements 0..=i.
Can cache intermediate results. Get_region computes incrementally.
Is_sorted depends on whether all elements are non-negative.

**US-25: Type coercion (int → real)**
ALTREP REALSXP backed by an INTSXP source. Elt(i) = inner_int_elt(i) as f64.
Length delegates. Coerce method could return this when asked to coerce
int→real (avoiding materialization). This is the deferred_string pattern
generalized.

**US-26: NA replacement (fill NAs with a default)**
Wraps a vector, replacing NA values with a fill value. Elt checks for NA
and substitutes. No_NA returns true. Is_sorted may or may not be preserved.

### String-specific

**US-27: Deferred string coercion (int/real → character)**
Like R's own deferred_string class. data1 = original numeric vector + config.
Elt converts one element to string on demand. Dataptr triggers full expansion.
Elements cached in data2 as they're computed (sparse expansion).

**US-28: Factor-like string ALTREP (levels + codes)**
Stores a small levels vector + integer codes vector. Elt(i) returns
levels[codes[i]]. Memory-efficient for categorical data with few levels.
Is_sorted could delegate to the codes vector's sortedness.

**US-29: Interned/deduplicated strings**
String pool + index array. Many duplicate strings share CHARSXP via R's
global cache. Elt(i) returns the pre-interned CHARSXP. No_NA checks the
index for sentinel value.

**US-30: Regex match results**
Lazy ALTREP string vector from regex captures. Elt(i) extracts the i-th
match from source text. Length = number of matches. Materialization builds
the full STRSXP.

### List-specific

**US-31: Chunked data frame columns**
ALTLIST where each element is a chunk (INTSXP, REALSXP, etc.) of a
partitioned column. Elt(i) returns chunk i. Length = number of chunks.
Used for chunked Arrow tables or partitioned Parquet datasets.

**US-32: Lazy list generation**
List where each element is computed on demand. Elt(i) calls a generator
function. Useful for test fixtures or simulation results.

### Zero-sized / stateless types

**US-33: All-NA vector**
`rep(NA_integer_, n)`. ZST data type. Elt always returns NA_integer_.
Length encoded in data1 as a scalar. No_NA = false. Sum = NA.
Min = Max = NA. Perfectly compact.

**US-34: Zero vector**
`rep(0L, n)`. Same as US-33 but value is 0. No_NA = true. Sum = 0.

**US-35: Iota / identity sequence**
`0:(n-1)`. Pure index vector. Elt(i) = i. Is_sorted = Increasing.
No storage at all beyond the length.

**US-36: Alternating pattern**
`rep(c(0L, 1L), n/2)`. Elt(i) = i % 2. Length = n. Compact.

### Serialization patterns

**US-37: Serializable computed type**
Arithmetic sequence that saves (start, step, len) as state. Unserialize
reconstructs the ArithSeq struct. Round-trips through saveRDS/readRDS.

**US-38: Non-serializable ephemeral type**
Database cursor ALTREP. Serialized_state returns NULL, causing R to
materialize and serialize as a standard vector. Data survives but laziness
is lost.

**US-39: Serializable with fallback**
Memory-mapped ALTREP that tries to serialize the file path. If the path
isn't portable, falls back to materializing. Unserialize tries to re-mmap
the file, falling back to the materialized data.

### Mutation patterns

**US-40: Copy-on-write owned data**
Vec<i32> ALTREP. Dataptr(writable=false) returns the pointer directly.
Dataptr(writable=true) also returns the pointer (it's owned). R handles
COW semantics via reference counting — if MAYBE_SHARED, R duplicates
before requesting writable dataptr.

**US-41: Immutable borrowed data**
`&'static [i32]` ALTREP. Dataptr(writable=false) returns the slice pointer.
Dataptr(writable=true) must either panic or trigger materialization into
a mutable copy in data2.

**US-42: Mutable string set_elt**
String ALTREP that supports Set_elt for in-place modification. On set,
the new CHARSXP is stored in the expanded data2 STRSXP. Original lazy
computation is preserved for unmodified elements.

### Composition

**US-43: ALTREP wrapping another ALTREP**
Sorted-wrapper around a memory-mapped integer vector. The outer ALTREP
caches Is_sorted metadata; element access delegates through to the inner
ALTREP's Elt method (which reads from mmap). Two levels of ALTREP dispatch.

**US-44: ALTREP as ExternalPtr sidecar**
An ExternalPtr-based class (R6/S7/Env) that has an ALTREP field.
The ALTREP is stored in the sidecar. Methods on the class return the
ALTREP, which R can then access efficiently.

### Performance-sensitive patterns

**US-45: Get_region for bulk access**
Arrow array with 1M integers. Rather than calling Elt 1M times (1M function
pointer dispatches), R calls Get_region once to fill a buffer. The implementation
does one memcpy from the Arrow buffer. 100x faster for `sum()`.

**US-46: Pre-computed summary statistics**
ALTREP integer backed by a Vec, but the type also stores pre-computed sum,
min, max, is_sorted, no_na. The Sum/Min/Max methods return instantly from
cached values. Useful when the data comes from a source that provides statistics.

**US-47: Dataptr_or_null for fast-path detection**
R's `INTEGER_GET_REGION` checks `INTEGER_OR_NULL(sx)` first — if it gets
a non-null pointer, it does a direct memcpy instead of dispatching through
ALTREP. Providing Dataptr_or_null enables this fast path for types that
have contiguous memory (Vec, Box<[T]>, mmap).

## Current Architecture (What Exists)

```
Layer 3 (user):     AltIntegerData { fn elt(&self, i) -> i32 }
                         |
                    impl_altinteger_from_data!(T) macro
                         |
Layer 2 (bridge):   Altrep + AltVec + AltInteger  (fn(SEXP) -> ...)
                         |
                    altrep_bridge.rs trampolines
                         |
Layer 1 (R):        R_set_altinteger_Elt_method(cls, fn_ptr)
```

Problems:

1. **Two-struct pattern**: wrapper struct + data struct, always
2. **Three separate derives/macros**: `#[derive(ExternalPtr)]` + `#[derive(AltrepInteger)]` + `impl_altinteger_from_data!(T)` or `#[miniextendr]` on wrapper
3. **Always ExternalPtr**: no way to use native SEXP storage
4. **Guard mode is per-type, not per-method**
5. **`impl_alt*_from_data!` macro jungle**: 7 families × (base + variants) = many macro arms

## Proposed Architecture

### Design Principle: One Derive Does Everything

```rust
#[derive(Altrep)]          // NEW: generates everything
struct ArithSeq {
    #[altrep(len)]
    len: usize,
    start: i32,
    step: i32,
}

impl AltIntegerData for ArithSeq {
    fn elt(&self, i: usize) -> i32 {
        self.start + self.step * i as i32
    }
}
```

The `#[derive(Altrep)]` generates:

1. `impl TypedExternal for ArithSeq` (ExternalPtr storage)
2. `impl Altrep for ArithSeq` (bridges to AltrepLen)
3. `impl AltVec for ArithSeq` (bridges to AltrepDataptr if implemented)
4. `impl AltInteger for ArithSeq` (bridges to AltIntegerData)
5. `impl InferBase for ArithSeq` (inferred from which `Alt*Data` trait is implemented)
6. `impl RegisterAltrep for ArithSeq` (OnceLock + class creation + method installation)
7. `impl IntoR for ArithSeq` (creates ExternalPtr + R_new_altrep)
8. linkme `#[distributed_slice]` registration entry
9. `ArithSeqRef` / `ArithSeqMut` accessor types with TryFromSexp

The family-specific derives (`#[derive(AltrepInteger)]`, etc.) become **aliases** for
`#[derive(Altrep)]` that additionally generate the `Alt*Data` trait impl from field
attributes (`#[altrep(elt = "field")]`, `#[altrep(elt_delegate = "field")]`).

### Layer 2 Redesign: Trait-Based Bridge

The bridge between user traits (Layer 3) and R callbacks (Layer 1) uses const
dispatch on the traits the type implements:

```rust
// In the generated impl Altrep for T:
impl Altrep for ArithSeq {
    const GUARD: AltrepGuard = AltrepGuard::RustUnwind;

    fn length(x: SEXP) -> R_xlen_t {
        let data: &Self = unsafe { extract_ref(x) };
        data.len() as R_xlen_t
    }
}

impl AltInteger for ArithSeq {
    const HAS_ELT: bool = true;
    fn elt(x: SEXP, i: R_xlen_t) -> i32 {
        let data: &Self = unsafe { extract_ref(x) };
        AltIntegerData::elt(data, i as usize)
    }

    const HAS_GET_REGION: bool = true;
    fn get_region(x: SEXP, start: R_xlen_t, n: R_xlen_t, buf: &mut [i32]) -> R_xlen_t {
        let data: &Self = unsafe { extract_ref(x) };
        AltIntegerData::get_region(data, start as usize, n as usize, buf) as R_xlen_t
    }

    // HAS_IS_SORTED, HAS_NO_NA etc. detected from whether the data trait
    // method returns Some vs None:
    const HAS_IS_SORTED: bool = true;  // opt-in via #[altrep(is_sorted)]
    const HAS_NO_NA: bool = true;      // opt-in via #[altrep(no_na)]
    // ...
}
```

### Extraction trait

```rust
/// How to get &Self from an ALTREP SEXP.
/// Default impl uses ExternalPtr (generated by #[derive(Altrep)]).
/// Power users can override for native SEXP storage.
trait AltrepExtract: Sized {
    /// Extract a shared reference from the ALTREP data1 slot.
    unsafe fn extract_ref(x: SEXP) -> &Self;

    /// Extract a mutable reference from the ALTREP data1 slot.
    unsafe fn extract_mut(x: SEXP) -> &mut Self;
}
```

For ExternalPtr-backed types (the default):

```rust
impl<T: TypedExternal> AltrepExtract for T {
    unsafe fn extract_ref(x: SEXP) -> &T {
        // ExternalPtr extraction from data1
    }
    unsafe fn extract_mut(x: SEXP) -> &mut T {
        // Mutable ExternalPtr extraction from data1
    }
}
```

For power users who want native SEXP storage, they implement this trait manually
and the derive skips generating ExternalPtr storage.

### What about the family-specific derives?

`#[derive(AltrepInteger)]` stays but becomes simpler — it's `#[derive(Altrep)]` plus
auto-generated `AltIntegerData` from field annotations. For users who implement
`AltIntegerData` manually, they just use `#[derive(Altrep)]` directly.

### HAS_* detection

Currently, `HAS_IS_SORTED`, `HAS_NO_NA`, etc. must be set manually. In the new design:

- **Always-on methods** (elt, get_region): generated from trait impl, always `true`
- **Optional methods** (is_sorted, no_na, sum, min, max): default trait methods
  return `None`, which the bridge translates to `HAS_* = false`. When the user
  overrides to return `Some(...)`, the bridge sets `HAS_* = true`.

Problem: `HAS_*` must be a const, but whether the user overrode a method is not
detectable at compile time. Two solutions:

**Option A**: Keep explicit opt-in via `#[altrep(is_sorted, no_na)]` on the struct.
The derive reads these and sets the const.

**Option B**: Always install all methods. Methods that return None/NULL cause R to
fall back to its default behavior. This is safe — R checks return values and falls
back. Minor overhead: one extra function pointer dispatch for methods that always
return NULL/UNKNOWN.

Option B is simpler but wastes a few function pointer slots. Option A is precise.
**Recommend Option A** for consistency with current design.

But wait — for `sum`, `min`, `max`, R actually checks the return value:
`ALTINTEGER_SUM(x, narm)` returns NULL if the method returns NULL, and R's `sum()`
implementation checks for NULL and falls back to its own loop. So Option B actually
works correctly. The only overhead is the function call + NULL check.

For `is_sorted` and `no_na`, the default returns `UNKNOWN_SORTEDNESS` / `0` (false),
which are already the "I don't know" values. So these are also safe to always install.

**Decision: Option B** — always install all methods. The const `HAS_*` pattern becomes
internal implementation detail. Users never think about it. The bridge sets all to true
and generates methods that delegate to the data trait (which returns None/Unknown by
default). Zero user burden.

Exception: `dataptr`, `serialize`, `extract_subset` — these should remain opt-in because
they have semantic implications (materializing, serialization format, subset behavior).

### Capabilities as marker traits or attributes

```rust
#[derive(Altrep)]
#[altrep(dataptr, serialize)]   // opt-in capabilities
struct MyVec(Vec<i32>);
```

Or equivalently, implement the capability traits:

```rust
impl AltrepDataptr<i32> for MyVec { ... }
impl AltrepSerialize for MyVec { ... }
```

The derive detects which capability traits are implemented (via the attribute annotations)
and generates the corresponding bridge code.

## Implementation Order

1. Add `AltrepExtract` trait to `altrep_data/core.rs`
2. Implement blanket `AltrepExtract` for `TypedExternal` types
3. Modify `#[derive(Altrep)]` to generate everything (TypedExternal, low-level traits, registration, IntoR)
4. Migrate one rpkg test fixture (e.g., ArithSeqData) to the new pattern
5. Verify it compiles and tests pass
6. Migrate remaining fixtures
7. Remove `impl_alt*_from_data!` macros
8. Remove wrapper struct requirement from `#[miniextendr]` ALTREP path
9. Update documentation
10. Clean up dead code
