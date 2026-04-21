+++
title = "RArray: N-Dimensional R Arrays"
weight = 34
description = "Compile-time dimensioned wrappers for R arrays, matrices, and vectors."
+++

Compile-time dimensioned wrappers for R arrays, matrices, and vectors.

## Overview

`RArray<T, NDIM>` wraps an R array SEXP with the dimension count tracked at compile
time via const generics. It provides safe, bounds-checked access to R's column-major
data with zero overhead (the wrapper is `#[repr(transparent)]` over a single SEXP).

## Table of Contents

- [Type Aliases](#type-aliases)
- [Quick Start](#quick-start)
- [Thread Safety](#thread-safety)
- [Memory Layout](#memory-layout)
- [Reading Data](#reading-data)
- [Creating Arrays](#creating-arrays)
- [Mutation](#mutation)
- [Coerced Types](#coerced-types)
- [Attributes](#attributes)
- [Performance](#performance)

## Type Aliases

| Alias | Type | R Equivalent |
|-------|------|--------------|
| `RVector<T>` | `RArray<T, 1>` | `vector` (with dim attribute) |
| `RMatrix<T>` | `RArray<T, 2>` | `matrix` |
| `RArray3D<T>` | `RArray<T, 3>` | `array(..., dim=c(a,b,c))` |

Supported element types (`T`) are those implementing `RNativeType`:

| Rust Type | R SEXP Type |
|-----------|-------------|
| `f64` | `REALSXP` |
| `i32` | `INTSXP` |
| `u8` | `RAWSXP` |
| `RLogical` | `LGLSXP` |
| `Rcomplex` | `CPLXSXP` |

## Quick Start

```rust
use miniextendr_api::rarray::RMatrix;

// RMatrix parameters require main_thread (RArray is !Send)
#[miniextendr(unsafe(main_thread))]
pub fn matrix_sum(m: RMatrix<f64>) -> f64 {
    unsafe { m.as_slice().iter().sum() }
}

#[miniextendr(unsafe(main_thread))]
pub fn column_means(m: RMatrix<f64>) -> Vec<f64> {
    let nrow = unsafe { m.nrow() };
    let ncol = unsafe { m.ncol() };
    (0..ncol)
        .map(|col| {
            let sum: f64 = unsafe { m.column(col) }.iter().sum();
            sum / nrow as f64
        })
        .collect()
}
```

From R:

```r
m <- matrix(1:12, nrow = 3, ncol = 4)
matrix_sum(m)
#> [1] 78

column_means(m)
#> [1] 2 5 8 11
```

## Thread Safety

`RArray` is **`!Send` and `!Sync`**. It cannot be transferred to or accessed from
other threads because the underlying R APIs (`DATAPTR_RO`, etc.) must be called on
the R main thread.

Functions that accept `RArray`, `RMatrix`, or `RVector` parameters must use
`#[miniextendr(unsafe(main_thread))]`:

```rust
#[miniextendr(unsafe(main_thread))]
pub fn process(m: RMatrix<f64>) -> f64 {
    // Runs on main thread -- RMatrix access is safe
    unsafe { m.as_slice().iter().sum() }
}
```

To use the data in worker threads or parallel code, copy it first with `to_vec()`:

```rust
#[miniextendr(unsafe(main_thread))]
pub fn parallel_process(m: RMatrix<f64>) -> f64 {
    // Copy on main thread -- Vec<f64> is Send
    let data: Vec<f64> = unsafe { m.to_vec() };
    // data can now be passed to rayon, worker threads, etc.
    data.iter().sum()
}
```

## Memory Layout

R arrays are stored in **column-major** (Fortran) order. For a 3x4 matrix:

```text
Logical layout:           Memory layout (contiguous):
      col0 col1 col2 col3
row0 [ 0    3    6    9 ]   [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]
row1 [ 1    4    7   10 ]    ^col0^  ^col1^  ^col2^  ^--col3--^
row2 [ 2    5    8   11 ]
```

Columns are contiguous in memory. The `column()` method returns a proper slice
with no striding. Rows are NOT contiguous -- row access requires stepping through
memory.

## Reading Data

### Slice Access (Fastest)

```rust
// Full buffer as a flat slice (column-major order)
let slice: &[f64] = unsafe { m.as_slice() };

// Iterate all elements
let sum: f64 = unsafe { m.as_slice().iter().sum() };
```

### Column Access (Fast, Matrix Only)

```rust
let ncol = unsafe { m.ncol() };
for col in 0..ncol {
    let col_data: &[f64] = unsafe { m.column(col) };
    // col_data is a contiguous slice of nrow elements
}
```

### Element Access (Slower)

```rust
// By N-dimensional indices (any NDIM)
let val: f64 = unsafe { m.get([row, col]) };

// By row/col (matrix convenience)
let val: f64 = unsafe { m.get_rc(row, col) };
```

### Dimensions

```rust
// All dimensions as array
let dims: [usize; 2] = unsafe { m.dims() };

// Individual dimension
let nrow = unsafe { m.dim(0) };
let ncol = unsafe { m.dim(1) };

// Matrix-specific helpers
let nrow = unsafe { m.nrow() };
let ncol = unsafe { m.ncol() };

// Total elements
let len = m.len();
let empty = m.is_empty();
```

### Copy to Vec

```rust
// Copy data to owned Vec (makes it Send)
let data: Vec<f64> = unsafe { m.to_vec() };
```

## Creating Arrays

### Allocate with Initializer

```rust
use miniextendr_api::rarray::{RMatrix, RArray3D};

// Matrix: 3 rows x 4 columns, initialized via closure
let matrix = unsafe {
    RMatrix::<f64>::new([3, 4], |slice| {
        for (i, v) in slice.iter_mut().enumerate() {
            *v = i as f64;
        }
    })
};

// 3D array: 2 x 3 x 4
let array = unsafe {
    RArray3D::<f64>::new([2, 3, 4], |slice| {
        slice.fill(1.0);
    })
};
```

### Allocate with Zeros

```rust
let matrix = unsafe { RMatrix::<f64>::zeros([100, 50]) };
```

### Important: GC Protection

`RArray::new()` and `RArray::zeros()` return an unprotected SEXP. The caller
must protect it if any further R allocations occur before returning:

```rust
let scope = ProtectScope::new();
let matrix = unsafe { RMatrix::<f64>::new([3, 4], |s| s.fill(0.0)) };
let protected = scope.protect(matrix.as_sexp());
```

## Mutation

### Mutable Slice

```rust
let mut m: RMatrix<f64> = /* ... */;
let slice: &mut [f64] = unsafe { m.as_slice_mut() };
slice[0] = 42.0;
```

### Element-Wise Set

```rust
// By N-dimensional indices
unsafe { m.set([row, col], 42.0) };

// Matrix convenience
unsafe { m.set_rc(row, col, 42.0) };
```

### Mutable Column

```rust
let col_data: &mut [f64] = unsafe { m.column_mut(col) };
col_data.fill(1.0);
```

## Coerced Types

`RArray` supports non-native Rust types via coercion from the underlying R type.
These wrap the source SEXP directly (zero-copy for construction), but `as_slice()`
is not available -- use `to_vec_coerced()` instead.

| Target Type | Source R Type | Coercion |
|-------------|--------------|----------|
| `i8`, `i16`, `i64`, `isize` | `INTSXP` (i32) | Integer narrowing/widening |
| `u16`, `u32`, `u64`, `usize` | `INTSXP` (i32) | Integer unsigned conversion |
| `f32` | `REALSXP` (f64) | Float narrowing |
| `bool` | `LGLSXP` (RLogical) | Logical conversion |

```rust
#[miniextendr(unsafe(main_thread))]
pub fn process_bool_matrix(m: RMatrix<bool>) -> Vec<bool> {
    unsafe { m.to_vec_coerced() }
}
```

Coercion validation happens at construction time (`TryFromSexp`). If any element
cannot be coerced (e.g., value out of range for narrowing), construction fails
with an error returned to R.

## Attributes

`RArray` provides access to standard R attributes:

### Getters

```rust
let names: Option<SEXP> = unsafe { m.get_names() };
let class: Option<SEXP> = unsafe { m.get_class() };
let dimnames: Option<SEXP> = unsafe { m.get_dimnames() };
let rownames: Option<SEXP> = unsafe { m.get_rownames() };
let colnames: Option<SEXP> = unsafe { m.get_colnames() };
```

### Setters

```rust
unsafe { m.set_names(names_sexp) };
unsafe { m.set_class(class_sexp) };
unsafe { m.set_dimnames(dimnames_sexp) };
```

## Performance

### Access Method Comparison

| Method | Speed | Use Case |
|--------|-------|----------|
| `as_slice()` | Fastest | Full-buffer iteration, SIMD |
| `column()` | Fast | Per-column operations (matrices) |
| `column_mut()` | Fast | Per-column mutation |
| `get()` / `get_rc()` | Slower | Single-element random access |

Per-element methods like `get()` perform index translation and bounds checks on
every call. For tight loops, this overhead dominates.

### Prefer Columns Over Rows

Columns are contiguous in R's column-major layout. Column iteration is a straight
memory scan; row iteration requires striding across columns.

```rust
// Fast: column-wise (contiguous memory)
for col in 0..ncol {
    for val in unsafe { m.column(col) } {
        // sequential memory access
    }
}

// Slow: row-wise (strided memory)
for row in 0..nrow {
    for col in 0..ncol {
        let val = unsafe { m.get_rc(row, col) };
        // jumping across columns in memory
    }
}
```

### Prefer Slice Iteration

For operations over all elements, use `as_slice()` instead of nested indexing:

```rust
// Fast
let sum: f64 = unsafe { m.as_slice() }.iter().sum();

// Slow
let mut sum = 0.0;
for row in 0..nrow {
    for col in 0..ncol {
        sum += unsafe { m.get_rc(row, col) };
    }
}
```

## See Also

- [Type Conversions](../type-conversions/) -- `TryFromSexp`/`IntoR` system
- [`#[miniextendr]` Attribute](../miniextendr-attribute/) -- `unsafe(main_thread)` and other options
- [Rayon Integration](../rayon/) -- `with_r_matrix` and `new_r_matrix` for parallel matrix construction
- [GC Protection](../gc-protect/) -- Protecting allocated arrays from garbage collection
