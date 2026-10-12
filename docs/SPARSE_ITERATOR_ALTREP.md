# Sparse Iterator ALTREP Guide

Sparse iterator ALTREP vectors use `Iterator::nth()` to skip to requested indices, caching only the elements that are actually accessed. They suit increasing-index access to a small part of a large source. Skipping consumes values permanently, so they do not provide ordinary random-access vector semantics.

**See also**: [ALTREP.md](ALTREP.md) for the full ALTREP system documentation, [ALTREP_QUICKREF.md](ALTREP_QUICKREF.md) for a quick reference.

---

## What Are Sparse Iterators?

Standard (prefix-caching) iterator ALTREP vectors cache every element from index 0 up to the highest accessed index. If you access element 999,999, the prefix-caching variant must materialize and store all 1,000,000 elements in a contiguous `Vec<T>`.

Sparse iterators take a different approach: they use `Iterator::nth()` to skip directly to the requested index, consuming (discarding) intermediate elements. Only the actually-accessed elements are stored in a `BTreeMap<usize, T>`. This means:

- **Accessing element 999,999** caches only that element (O(1) cache memory for a single access). The iterator may still compute every skipped value.
- **Skipped elements are permanently lost** -- they return the type's NA/default value
- **Memory scales with access count**, not with the highest accessed index

### The Trade-off

Sparse iterators trade **completeness for efficiency**. You get O(num_accessed) memory instead of O(max_index), but skipped elements can never be retrieved. This is the right trade-off when:

- The vector is very large (millions of elements)
- You only need a small subset of elements
- You know the access pattern in advance (or don't care about missed elements)

---

## How It Works Internally

### Core Data Structure

The `SparseIterState<I, T>` struct (defined in `miniextendr-api/src/altrep_data/iter/sparse.rs`) holds three fields:

```rust
pub struct SparseIterState<I, T> {
    len: usize,                             // Total vector length
    iter: RefCell<Option<(I, usize)>>,      // (iterator, next_index)
    cache: RefCell<BTreeMap<usize, T>>,     // Sparse cache of accessed elements
}
```

### Element Access Algorithm (`get_element`)

When R requests element at index `i`:

1. **Bounds check**: If `i >= len`, return `None`.
2. **Cache lookup**: If `cache` contains `i`, return the cached value immediately.
3. **Forward skip**: If `i >= next_index`, call `iter.nth(i - next_index)` to skip forward. This consumes all elements between `next_index` and `i` (exclusive), then yields element `i`.
4. **Backward request**: If `i < next_index`, the element was already skipped -- return `None`.
5. **Cache the result**: Store the element in the `BTreeMap` for future lookups.

`None` is mapped to the type-appropriate NA/default by each concrete data type. Once `nth()` returns `None`, the source is dropped and never polled again, even if it is not fused. Reaching the declared length also drops the source without requesting another element; a zero-length prefix never polls it. Cached values remain readable.

### Iterator::nth() -- The Key Mechanism

Rust's `Iterator::nth(n)` method advances the iterator by `n+1` positions and returns the next element, or `None` if the iterator is exhausted. For many iterators (like `Range`), `nth()` is O(1) rather than O(n), making the skip virtually free.

For custom iterators, `nth()` defaults to calling `next()` N times, but you can override it for better performance if your data structure supports random access.

---

## Available Types

miniextendr provides five sparse iterator data types:

| Type | Element | NA/Default Value | Trait |
|------|---------|-----------------|-------|
| `SparseIterIntData<I>` | `i32` | `NA_INTEGER` (`i32::MIN`) | `AltIntegerData` |
| `SparseIterRealData<I>` | `f64` | `NA_REAL` (R's `NA_real_`) | `AltRealData` |
| `SparseIterLogicalData<I>` | `bool` | `Logical::Na` (NA_logical) | `AltLogicalData` |
| `SparseIterRawData<I>` | `u8` | `0` (raw has no NA concept) | `AltRawData` |
| `SparseIterComplexData<I>` | `Rcomplex` | `NA_REAL` in both parts (R's `NA_complex_`) | `AltComplexData` |

All types share the same behavior:

- `as_slice()` always returns `None` (sparse storage cannot provide a contiguous slice)
- `get_region()` fills a buffer element-by-element via `elt()`
- `len()` returns the declared length

Delegation wrappers (Step 2 below) must forward `get_region()` to the inner
type — a wrapper that implements only `elt()` silently reverts bulk scans to
the default per-element loop. See
[ALTREP_GET_REGION.md](ALTREP_GET_REGION.md) for when R calls `get_region`
and how to fill regions efficiently.

---

## Complete Example: Sparse Integer Iterator

This example from `rpkg/src/rust/lib.rs` shows the full pattern for creating a sparse integer ALTREP vector.

### Step 1: Define a Wrapper Type

The generic `SparseIterIntData<I>` needs a concrete iterator type. Use a boxed
trait object for flexibility. The `#[derive(AltrepInteger)]` with
`#[altrep(manual)]` registers the ALTREP class and leaves the data traits
(`AltrepLen` + `AltIntegerData`) for you to implement by hand in the next step:

```rust
use miniextendr_api::prelude::*;
use miniextendr_api::altrep_data::SparseIterIntData;

/// Type alias for boxed iterator producing i32
type BoxedIntIter = Box<dyn Iterator<Item = i32>>;

/// Wrapper for sparse integer iterator ALTREP
#[derive(AltrepInteger)]
#[altrep(class = "SparseIntIter", manual)]
pub struct SparseIntIterData {
    inner: SparseIterIntData<BoxedIntIter>,
}
```

### Step 2: Delegate Trait Implementations

```rust
impl AltrepLen for SparseIntIterData {
    fn altrep_len(&self) -> usize {
        self.inner.altrep_len()
    }
}

impl AltIntegerData for SparseIntIterData {
    fn elt(&self, i: usize) -> i32 {
        self.inner.elt(i)
    }

    fn as_slice(&self) -> Option<&[i32]> {
        None  // Sparse storage cannot provide contiguous slice
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [i32]) -> usize {
        self.inner.get_region(start, len, buf)
    }
}
```

### Step 3: Registration Is Automatic

The derive in Step 1 already emitted the low-level trait impls
(`impl_altinteger_from_data!`), the `RegisterAltrep` impl, and the
registration entry. There is no macro to call and no wrapper class struct to
define — `#[miniextendr]` on a 1-field struct is removed.

### Step 4: Write the Constructor Function

```rust
#[miniextendr]
pub fn sparse_iter_int(from: i32, to: i32) -> SEXP {
    let len = (to - from).max(0) as usize;
    let start = from;
    let iter: BoxedIntIter = Box::new((0..len as i32).map(move |i| start + i));
    let data = SparseIntIterData {
        inner: SparseIterIntData::from_iter(iter, len),
    };
    data.into_sexp()
}
```

Registration is automatic via the derive's registration entry -- no manual module declarations needed.

---

## Behavior from R

### Sequential Access -- Everything Works

When elements are accessed in order, sparse iterators behave identically to prefix-caching ones:

```r
x <- sparse_iter_int(0L, 5L)
x[1]  #> 0
x[2]  #> 1
x[3]  #> 2
x[4]  #> 3
x[5]  #> 4
```

### Forward Skip -- Skipped Elements Become NA

Accessing a later element first causes all earlier elements to be permanently skipped:

```r
x <- sparse_iter_int(1L, 101L)

# Access element 50 first -- elements 1-49 are skipped
x[50]   #> 50

# Skipped elements return NA
x[1]    #> NA
x[25]   #> NA
x[49]   #> NA

# Accessed element is cached and still works
x[50]   #> 50

# Elements after 50 are still available
x[51]   #> 51
x[100]  #> 100
```

### Impact on Aggregate Operations

Because skipped elements become NA, aggregate functions like `sum()` are affected:

```r
x <- sparse_iter_int(1L, 6L)  # Elements: 1, 2, 3, 4, 5

# Access only element 5
x[5]  #> 5

# sum() sees NAs for elements 1-4
sum(x)             #> NA
sum(x, na.rm = TRUE)  #> 5  (only cached element)
```

### Comparison with Prefix-Caching Iterators

```r
# Prefix-caching: accessing element 50 caches elements 1-50
regular <- iter_int_range(1L, 101L)
regular[50]  #> 50
regular[1]   #> 1   (cached in prefix)

# Sparse: accessing element 50 skips elements 1-49
sparse <- sparse_iter_int(1L, 101L)
sparse[50]   #> 50
sparse[1]    #> NA  (skipped, gone forever)
```

---

## Sparse Real Iterator Example

Real-valued sparse iterators return R's `NA_real_` for skipped elements, not an ordinary `NaN`: `is.na()` is `TRUE` and `is.nan()` is `FALSE`, and Rust's `IsNA` recognizes it:

```rust
type BoxedRealIter = Box<dyn Iterator<Item = f64>>;

#[derive(AltrepReal)]
#[altrep(class = "SparseRealIter", manual)]
pub struct SparseRealIterData {
    inner: SparseIterRealData<BoxedRealIter>,
}

// ... trait delegation same as integer variant ...

#[miniextendr]
pub fn sparse_iter_real(from: f64, step: f64, length_out: i32) -> SEXP {
    let len = length_out.max(0) as usize;
    let iter: BoxedRealIter = Box::new(
        (0..len).map(move |i| from + (i as f64) * step)
    );
    let data = SparseRealIterData {
        inner: SparseIterRealData::from_iter(iter, len),
    };
    data.into_sexp()
}
```

```r
x <- sparse_iter_real(0, 0.5, 100L)
x[50]   #> 24.5  (0 + 49 * 0.5)
x[1]    #> NA    (skipped)
x[51]   #> 25.0  (still available)
```

---

## When to Use Sparse vs Prefix-Caching

### Use Sparse Iterators When:

- **Sampling in increasing order**: Sort selected indices before reading a fresh sparse vector (e.g., `x[sort(sample(1e6, 100))]`). An unordered sample can lose requested values when a higher index is read first.
- **Tail access**: You primarily access elements near the end of a large vector
- **Memory-constrained**: You cannot afford to cache millions of elements
- **One-shot access**: Each element is accessed at most once, in roughly increasing order
- **Large vectors with few accesses**: A 10M-element vector where you only read 100 elements

### Use Prefix-Caching (IterState) When:

- **Sequential scan**: You'll iterate through all or most elements (`for`, `sapply`, etc.)
- **Random access**: You need to access elements in any order, including backward
- **Aggregate operations**: You'll call `sum()`, `mean()`, etc. (these touch every element)
- **Multiple passes**: Data will be read more than once
- **Slice access**: You need `as_slice()` for zero-copy operations

### Decision Table

| Access Pattern | Prefix-Caching | Sparse |
|---------------|----------------|--------|
| `x[1:n]` (full scan) | Best | Equivalent but wastes BTreeMap overhead |
| `x[n]` (single tail access) | Caches 0..n, O(n) memory | O(1) memory |
| `x[c(10, 20, 30)]` (few scattered) | Caches 0..30, O(30) memory | O(3) memory |
| `sum(x)` / `mean(x)` | Preserves all source values | Includes the defaults for skipped elements |
| `x[sort(sample(n, k))]` on a fresh vector | Caches through the largest index | O(k) cache memory |
| `x[sample(n, k)]` (unordered sample) | Preserves all requested values | Can return defaults for skipped indices |

---

## Default Values for Skipped Elements

Each R type has a different representation for "no value":

| Type | Skipped Element Value | R Display |
|------|----------------------|-----------|
| Integer (`SparseIterIntData`) | `NA_INTEGER` (`i32::MIN`) | `NA` |
| Real (`SparseIterRealData`) | `NA_REAL` | `NA` |
| Logical (`SparseIterLogicalData`) | `Logical::Na` | `NA` |
| Raw (`SparseIterRawData`) | `0` | `00` |
| Complex (`SparseIterComplexData`) | `NA_REAL` in both parts | `NA` |

Note that raw vectors have no NA concept in R, so skipped raw elements return `0x00`. This means you cannot distinguish a skipped element from a legitimately-zero element in a raw sparse iterator.

---

## Limitations

1. **Skipped values cannot be recovered**: Backward reads of cached elements work. An element discarded by a skip returns NA/default forever; there is no way to rewind the iterator.

2. **No `as_slice()`**: Sparse storage uses `BTreeMap`, not a contiguous array. When R needs a data pointer (`DATAPTR`, for example to duplicate or save the vector), the vector is copied into an ordinary R vector by reading every element in increasing order: cached values, the NA/default for skipped elements, and newly computed values for elements the iterator has not reached yet. Later reads come from that copy.

3. **Serialization loses the iterator**: Without custom serialization hooks, saving materializes the vector (see above) and R writes it as an ordinary vector; `readRDS()` restores that vector. Cached values survive, skipped positions keep their NA/default values, and elements the iterator had not reached are computed when the vector is saved. Preserving the ALTREP class requires an explicit serialization and reconstruction policy on the concrete class.

4. **Aggregates see the skipped-position defaults**: `sum()` and `mean()` include the NA of each skipped element. `na.rm = TRUE` ignores these, but cannot recover the discarded values. Use prefix-caching iterators when an aggregate must include every source value.

5. **Raw vectors cannot signal "missing"**: Since `u8` has no NA concept, `SparseIterRawData` returns `0` for skipped elements. This is indistinguishable from a real `0` value.

6. **`Iterator::nth()` performance varies**: For iterators like `Range`, `nth()` is O(1). For chained iterators or complex transformations, `nth()` may be O(n) as it calls `next()` repeatedly. Profile your iterator if skip performance matters.

---

## Construction Reference

All sparse data types support two constructors:

```rust
// Explicit length -- use when iterator doesn't implement ExactSizeIterator
let data = SparseIterIntData::from_iter(my_iter, 1_000_000);

// Auto-detect length -- use when iterator implements ExactSizeIterator
let data = SparseIterIntData::from_exact_iter(0..1_000_000);
```

### Introspection Methods on `SparseIterState`

These methods are available on the underlying `SparseIterState<I, T>`:

| Method | Returns | Description |
|--------|---------|-------------|
| `len()` | `usize` | Total declared vector length |
| `is_empty()` | `bool` | Whether length is 0 |
| `get_element(i)` | `Option<T>` | Get element (may skip forward) |
| `iterator_position()` | `Option<usize>` | Next source index, or `None` after EOF or completion of the declared prefix |
| `is_cached(i)` | `bool` | Whether element `i` has been cached |
| `cached_count()` | `usize` | Number of elements currently in cache |
