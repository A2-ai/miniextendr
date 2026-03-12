# Sparse Iterator ALTREP Guide

Sparse iterator ALTREP vectors use `Iterator::nth()` to skip directly to requested indices, caching only the elements that are actually accessed. This makes them ideal for large vectors where only a few elements are needed.

**See also**: [ALTREP.md](ALTREP.md) for the full ALTREP system documentation, [ALTREP_QUICKREF.md](ALTREP_QUICKREF.md) for a quick reference.

---

## What Are Sparse Iterators?

Standard (prefix-caching) iterator ALTREP vectors cache every element from index 0 up to the highest accessed index. If you access element 999,999, the prefix-caching variant must materialize and store all 1,000,000 elements in a contiguous `Vec<T>`.

Sparse iterators take a different approach: they use `Iterator::nth()` to skip directly to the requested index, consuming (discarding) intermediate elements. Only the actually-accessed elements are stored in a `BTreeMap<usize, T>`. This means:

- **Accessing element 999,999** generates only that one element (O(1) memory for a single access)
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

The `SparseIterState<I, T>` struct (defined in `miniextendr-api/src/altrep_data/iter.rs`) holds three fields:

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

`None` is mapped to the type-appropriate NA/default by each concrete data type.

### Iterator::nth() -- The Key Mechanism

Rust's `Iterator::nth(n)` method advances the iterator by `n+1` positions and returns the next element, or `None` if the iterator is exhausted. For many iterators (like `Range`), `nth()` is O(1) rather than O(n), making the skip virtually free.

For custom iterators, `nth()` defaults to calling `next()` N times, but you can override it for better performance if your data structure supports random access.

---

## Available Types

miniextendr provides four sparse iterator data types:

| Type | Element | NA/Default Value | Trait |
|------|---------|-----------------|-------|
| `SparseIterIntData<I>` | `i32` | `NA_INTEGER` (`i32::MIN`) | `AltIntegerData` |
| `SparseIterRealData<I>` | `f64` | `NaN` (which R displays as `NA`) | `AltRealData` |
| `SparseIterLogicalData<I>` | `bool` | `Logical::Na` (NA_logical) | `AltLogicalData` |
| `SparseIterRawData<I>` | `u8` | `0` (raw has no NA concept) | `AltRawData` |

There is also `SparseIterComplexData<I>` for complex number vectors.

All types share the same behavior:
- `as_slice()` always returns `None` (sparse storage cannot provide a contiguous slice)
- `get_region()` fills a buffer element-by-element via `elt()`
- `len()` returns the declared length

---

## Complete Example: Sparse Integer Iterator

This example from `rpkg/src/rust/lib.rs` shows the full pattern for creating a sparse integer ALTREP vector.

### Step 1: Define a Wrapper Type

The generic `SparseIterIntData<I>` needs a concrete iterator type. Use a boxed trait object for flexibility:

```rust
use miniextendr_api::altrep_data::{
    AltrepLen, AltIntegerData, SparseIterIntData,
};

/// Type alias for boxed iterator producing i32
type BoxedIntIter = Box<dyn Iterator<Item = i32>>;

/// Wrapper for sparse integer iterator ALTREP
#[derive(miniextendr_api::ExternalPtr)]
pub struct SparseIntIterData {
    inner: SparseIterIntData<BoxedIntIter>,
}
```

### Step 2: Delegate Trait Implementations

```rust
impl AltrepLen for SparseIntIterData {
    fn len(&self) -> usize {
        self.inner.len()
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

### Step 3: Generate Low-Level Trait Impls and Create the ALTREP Class

```rust
miniextendr_api::impl_altinteger_from_data!(SparseIntIterData);

/// The ALTREP class wrapper
#[miniextendr(class = "SparseIntIter")]
pub struct SparseIntIterClass(pub SparseIntIterData);
```

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
    SparseIntIterClass(data).into_sexp()
}
```

Registration is automatic via `#[miniextendr]` -- no manual module declarations needed.

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

Real-valued sparse iterators work the same way, with `NaN` (displayed as `NA` in R) for skipped elements:

```rust
type BoxedRealIter = Box<dyn Iterator<Item = f64>>;

#[derive(miniextendr_api::ExternalPtr)]
pub struct SparseRealIterData {
    inner: SparseIterRealData<BoxedRealIter>,
}

// ... trait delegation same as integer variant ...

#[miniextendr]
pub fn sparse_iter_real(from: f64, step: f64, n: i32) -> SEXP {
    let len = n.max(0) as usize;
    let iter: BoxedRealIter = Box::new(
        (0..len).map(move |i| from + (i as f64) * step)
    );
    let data = SparseRealIterData {
        inner: SparseIterRealData::from_iter(iter, len),
    };
    SparseRealIterClass(data).into_sexp()
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

- **Sampling**: You need random elements from a large range (e.g., `x[sample(1e6, 100)]`)
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
| `sum(x)` / `mean(x)` | Best (materializes once) | NAs for all skipped elements |
| `x[sample(n, k)]` (random sample) | Caches 0..max(sample), O(max) | O(k) memory |

---

## Default Values for Skipped Elements

Each R type has a different representation for "no value":

| Type | Skipped Element Value | R Display |
|------|----------------------|-----------|
| Integer (`SparseIterIntData`) | `NA_INTEGER` (`i32::MIN`) | `NA` |
| Real (`SparseIterRealData`) | `NaN` | `NA` |
| Logical (`SparseIterLogicalData`) | `Logical::Na` | `NA` |
| Raw (`SparseIterRawData`) | `0` | `00` |

Note that raw vectors have no NA concept in R, so skipped raw elements return `0x00`. This means you cannot distinguish a skipped element from a legitimately-zero element in a raw sparse iterator.

---

## Limitations

1. **No backward access**: Once an element is skipped, it returns NA/default forever. There is no way to "rewind" the iterator.

2. **No `as_slice()`**: Sparse storage uses `BTreeMap`, not a contiguous array. Operations requiring a contiguous slice (like `DATAPTR` materialization) will trigger R's default materialization path.

3. **No serialization**: Like all iterator-backed ALTREP, sparse iterators cannot be serialized with `saveRDS()` because the iterator state is not reconstructable. Serialization will trigger materialization first.

4. **Aggregate functions see NAs**: Calling `sum()`, `mean()`, or similar functions on a partially-accessed sparse vector will include NAs for skipped elements. Use `na.rm = TRUE` to ignore them, or use prefix-caching iterators if you need aggregates.

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
| `iterator_position()` | `Option<usize>` | Next index the iterator will produce |
| `is_cached(i)` | `bool` | Whether element `i` has been cached |
| `cached_count()` | `usize` | Number of elements currently in cache |
