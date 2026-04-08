+++
title = "ALTREP Practical Examples"
weight = 29
description = "Real-world examples of ALTREP usage patterns in miniextendr."
+++

Real-world examples of ALTREP usage patterns in miniextendr.

## Table of Contents

- [Database Result Set (List)](#database-result-set-list)
- [Memory-Mapped File (Raw)](#memory-mapped-file-raw)
- [Lazy Time Series (Real)](#lazy-time-series-real)
- [Sparse Matrix Row (Integer)](#sparse-matrix-row-integer)
- [External API Cache (String)](#external-api-cache-string)

---

## Database Result Set (List)

**Use case**: Lazy loading of database query results without materializing all rows.

```rust
use miniextendr_api::altrep_data::{AltrepLen, AltListData};
use miniextendr_api::ffi::SEXP;
use std::cell::RefCell;

#[derive(miniextendr_api::ExternalPtr)]
pub struct DatabaseResultSet {
    connection_id: usize,
    query: String,
    row_count: usize,
    // Cache fetched rows
    cache: RefCell<std::collections::HashMap<usize, SEXP>>,
}

impl AltrepLen for DatabaseResultSet {
    fn len(&self) -> usize {
        self.row_count
    }
}

impl AltListData for DatabaseResultSet {
    fn elt(&self, i: usize) -> SEXP {
        // Check cache first
        if let Some(&row) = self.cache.borrow().get(&i) {
            return row;
        }

        // Fetch from database (simplified)
        let row = fetch_row_from_db(self.connection_id, &self.query, i);

        // Cache the result
        self.cache.borrow_mut().insert(i, row);

        row
    }
}

miniextendr_api::impl_altlist_from_data!(DatabaseResultSet);
```

**Benefits**:
- Only fetches rows when accessed
- Caches fetched rows for repeated access
- Memory efficient for large result sets
- R users get familiar list interface

**Usage in R**:
```r
results <- db_query("SELECT * FROM large_table")
first_row <- results[[1]]      # Fetches row 1
summary <- results[[1:100]]    # Fetches first 100 rows
```

---

## Memory-Mapped File (Raw)

**Use case**: Zero-copy access to large binary files.

```rust
use miniextendr_api::altrep_data::{AltrepLen, AltRawData, AltrepDataptr};
use memmap2::Mmap;
use std::fs::File;

#[derive(miniextendr_api::ExternalPtr)]
pub struct MappedFile {
    _file: File,
    mmap: Mmap,
}

impl AltrepLen for MappedFile {
    fn len(&self) -> usize {
        self.mmap.len()
    }
}

impl AltRawData for MappedFile {
    fn elt(&self, i: usize) -> u8 {
        self.mmap[i]
    }
}

impl AltrepDataptr<u8> for MappedFile {
    fn dataptr(&mut self, _writable: bool) -> Option<*mut u8> {
        // Return pointer to mmap'd memory
        Some(self.mmap.as_ptr() as *mut u8)
    }

    fn dataptr_or_null(&self) -> Option<*const u8> {
        Some(self.mmap.as_ptr())
    }
}

miniextendr_api::impl_altraw_from_data!(MappedFile, dataptr);
```

**Benefits**:
- Zero-copy file access
- OS handles memory management
- Works with files larger than RAM
- Full dataptr support for fast operations

**Usage in R**:
```r
file <- mmap_file("large_data.bin")
header <- file[1:1024]          # Read first 1KB
checksum <- sum(as.integer(file))  # Process entire file
```

---

## Lazy Time Series (Real)

**Use case**: Generate time series data on-demand from formula.

```rust
use miniextendr_api::altrep_data::{AltrepLen, AltRealData};

#[derive(miniextendr_api::ExternalPtr)]
pub struct SinewaveTimeSeries {
    frequency: f64,
    amplitude: f64,
    phase: f64,
    sample_rate: f64,
    n_samples: usize,
}

impl AltrepLen for SinewaveTimeSeries {
    fn len(&self) -> usize {
        self.n_samples
    }
}

impl AltRealData for SinewaveTimeSeries {
    fn elt(&self, i: usize) -> f64 {
        let t = i as f64 / self.sample_rate;
        self.amplitude * (2.0 * std::f64::consts::PI * self.frequency * t + self.phase).sin()
    }

    fn no_na(&self) -> Option<bool> {
        Some(true)  // Mathematical function, no NAs
    }
}

miniextendr_api::impl_altreal_from_data!(SinewaveTimeSeries);
```

**Benefits**:
- O(1) memory regardless of length
- Perfect precision (no rounding from storage)
- Supports arbitrary length
- Can change parameters without regenerating

**Usage in R**:
```r
wave <- sinewave_ts(freq = 440, amplitude = 1.0, samples = 44100)
plot(wave[1:1000])  # Plot first second
max(wave)           # Find peak (should be ~1.0)
```

---

## Sparse Matrix Row (Integer)

**Use case**: Efficient representation of sparse data.

```rust
use miniextendr_api::altrep_data::{AltrepLen, AltIntegerData};
use std::collections::HashMap;

#[derive(miniextendr_api::ExternalPtr)]
pub struct SparseVector {
    length: usize,
    default_value: i32,
    // Only store non-default values
    values: HashMap<usize, i32>,
}

impl AltrepLen for SparseVector {
    fn len(&self) -> usize {
        self.length
    }
}

impl AltIntegerData for SparseVector {
    fn elt(&self, i: usize) -> i32 {
        *self.values.get(&i).unwrap_or(&self.default_value)
    }

    fn no_na(&self) -> Option<bool> {
        // Check if default or any stored value is NA
        if self.default_value == i32::MIN {
            return Some(false);
        }
        Some(!self.values.values().any(|&v| v == i32::MIN))
    }

    fn sum(&self, _na_rm: bool) -> Option<i64> {
        // Optimized sum: sum(stored values) + default * (n - n_stored)
        let stored_sum: i64 = self.values.values().map(|&v| v as i64).sum();
        let n_default = (self.length - self.values.len()) as i64;
        Some(stored_sum + (self.default_value as i64) * n_default)
    }
}

miniextendr_api::impl_altinteger_from_data!(SparseVector);
```

**Benefits**:
- Memory proportional to non-default values
- Fast sum() for sparse data
- Efficient for large mostly-zero vectors
- Transparent to R users

**Usage in R**:
```r
# 1M element vector, only 100 non-zero
sparse <- sparse_vector(length = 1e6, default = 0L,
                        indices = sample(1e6, 100),
                        values = sample(100, 100))

sum(sparse)        # O(n_stored) not O(n)
sparse[1:100]      # Access works normally
```

---

## External API Cache (String)

**Use case**: Lazy-load strings from external API with intelligent caching.

```rust
use miniextendr_api::altrep_data::{AltrepLen, AltStringData};
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(miniextendr_api::ExternalPtr)]
pub struct ApiStringCache {
    api_endpoint: String,
    item_ids: Vec<String>,
    cache: RefCell<HashMap<usize, String>>,
}

impl AltrepLen for ApiStringCache {
    fn len(&self) -> usize {
        self.item_ids.len()
    }
}

impl AltStringData for ApiStringCache {
    fn elt(&self, i: usize) -> Option<&str> {
        // Check cache
        if let Some(cached) = self.cache.borrow().get(&i) {
            // SAFETY: This is unsafe - in production use thread-local storage
            unsafe {
                return Some(std::mem::transmute::<&str, &str>(cached.as_str()));
            }
        }

        // Fetch from API
        let item_id = &self.item_ids[i];
        let value = fetch_from_api(&self.api_endpoint, item_id);

        // Cache it
        self.cache.borrow_mut().insert(i, value);

        // Return (in practice, use better pattern)
        self.cache.borrow().get(&i).map(|s| {
            unsafe { std::mem::transmute::<&str, &str>(s.as_str()) }
        })
    }

    fn no_na(&self) -> Option<bool> {
        Some(false)  // API might return NAs
    }
}

miniextendr_api::impl_altstring_from_data!(ApiStringCache);
```

**Benefits**:
- Lazy loading from API
- Automatic caching
- Familiar R string vector interface
- Network efficient (only fetch accessed items)

**Usage in R**:
```r
items <- api_string_cache("https://api.example.com/items",
                          ids = paste0("item_", 1:1000))

first <- items[1]        # Fetches item_1
subset <- items[1:10]    # Fetches items 1-10 (caches all)
first_again <- items[1]  # Returns from cache (no API call)
```

---

## Common Patterns

### Pattern 1: Lazy + Cache

```rust
struct LazyData {
    params: ComputeParams,
    cache: RefCell<Option<Vec<T>>>,
}

impl AltrepDataptr<T> for LazyData {
    fn dataptr(&mut self, _writable: bool) -> Option<*mut T> {
        if self.cache.borrow().is_none() {
            *self.cache.borrow_mut() = Some(compute_all(&self.params));
        }
        self.cache.borrow_mut().as_mut().map(|v| v.as_mut_ptr())
    }
}
```

**When**: Computation is expensive but eventual materialization is expected.

### Pattern 2: External Data View

```rust
struct ExternalView {
    source: Arc<ExternalData>,
    offset: usize,
    length: usize,
}

impl AltRealData for ExternalView {
    fn elt(&self, i: usize) -> f64 {
        self.source.get(self.offset + i)
    }
}
```

**When**: Wrapping external data source without copying.

### Pattern 3: Computed Sequence

```rust
struct ComputedSeq {
    f: Box<dyn Fn(usize) -> i32>,
    len: usize,
}

impl AltIntegerData for ComputedSeq {
    fn elt(&self, i: usize) -> i32 {
        (self.f)(i)
    }
}
```

**When**: Mathematical sequences or algorithmic generation.

---

## Performance Tips for Real-World Use

1. **Cache intelligently**: Don't cache everything, don't cache nothing
   - Cache accessed elements (LRU if memory constrained)
   - Clear cache if parameters change

2. **Provide optimization hints**:
   - `no_na()` if you know there are no NAs
   - `is_sorted()` if data is sorted
   - `sum()`, `min()`, `max()` if you can compute them efficiently

3. **Choose materialization strategy**:
   - Pure lazy: Mathematical sequences, infinite streams
   - Lazy + cache: Expensive computation, external data
   - Pre-materialized: Data already in memory

4. **Handle errors gracefully**:
   - API failures → return NA with warning
   - Out of bounds → R handles it
   - Invalid state → panic with clear message

5. **Test serialization**:
   - Implement `AltrepSerialize` if state is serializable
   - Skip if tied to external resources (files, network, DB)

---

## Anti-Patterns to Avoid

❌ **Don't**: Create ALTREP for small vectors (<1000 elements)
- Overhead outweighs benefits
- Use regular `Vec<T>` instead

❌ **Don't**: Implement dataptr() for truly lazy sequences
- Defeats purpose of laziness
- Let R materialize if needed

❌ **Don't**: Store non-Send data in ALTREP
- ALTREP must be `Send`
- Use interior mutability carefully

❌ **Don't**: Ignore memory in caches
- Implement cache eviction
- Consider LRU or size limits

❌ **Don't**: Assume sequential access
- Users may access `[1, 1000, 2]`
- Don't optimize only for forward iteration

---

## Testing Your ALTREP

```r
# Basic functionality
x <- your_altrep(...)
stopifnot(length(x) == expected_length)
stopifnot(x[1] == expected_first_element)

# Serialization
tmp <- tempfile()
saveRDS(x, tmp)
y <- readRDS(tmp)
stopifnot(identical(x, y))

# Subsetting
sub <- x[1:100]
stopifnot(length(sub) == 100)

# Operations
stopifnot(!is.na(sum(x)))   # Should not error
stopifnot(!is.na(min(x)))
stopifnot(!is.na(max(x)))
```

---

**See also**:
- [ALTREP.md](ALTREP.md) - Complete API reference
- [Background reference implementations](../background/) - simplemmap, mutable, vectorwindow
- [Test suite](../rpkg/tests/testthat/test-altrep*.R) - Examples in tests
