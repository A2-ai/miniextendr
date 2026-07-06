//! Tests for ndarray integration.
//!
//! This module provides wrapper types that expose ndarray arrays to R
//! along with adapter trait methods for testing.

use miniextendr_api::ndarray_impl::{
    Array1, Array2, ArrayD, ArrayView1, ArrayView2, IxDyn, RNdArrayOps, RNdIndex, RNdSlice,
    RNdSlice2D,
};
use miniextendr_api::{ExternalPtr, miniextendr};

// region: NdVec - Wrapper for Array1<f64>

/// A 1D numeric array wrapper for testing RNdArrayOps and RNdSlice.
#[derive(ExternalPtr)]
pub struct NdVec(Array1<f64>);

/// NdVec methods: 1D numeric array operations, slicing, and conversion.
/// @param data Numeric vector of array elements.
/// @details `get`, `get_many`, `is_valid_index`, and `slice_1d` take 1-based
///   indices (R-idiomatic, matching the rest of the package). `get` errors on
///   an out-of-bounds or non-positive index; `get_many` returns `NA` per
///   out-of-bounds element instead (vectorized contract). `slice_1d` takes
///   1-based inclusive bounds (R's `x[start:end]` convention) and errors
///   unless `1 <= start <= end <= length`.
// env pinned: method names collide with base R non-generics (var/get/row/col/
// diag/reshape), which the s7-default flip cannot register S7 methods on (#1114).
#[miniextendr(env)]
impl NdVec {
    fn new(data: Vec<f64>) -> Self {
        NdVec(Array1::from_vec(data))
    }

    fn from_range(start: f64, end: f64, step: f64) -> Self {
        let mut data = Vec::new();
        let mut val = start;
        while val < end {
            data.push(val);
            val += step;
        }
        NdVec(Array1::from_vec(data))
    }

    // --- RNdArrayOps methods ---
    fn len(&self) -> i32 {
        RNdArrayOps::len(&self.0)
    }

    fn is_empty(&self) -> bool {
        RNdArrayOps::is_empty(&self.0)
    }

    fn ndim(&self) -> i32 {
        RNdArrayOps::ndim(&self.0)
    }

    fn shape(&self) -> Vec<i32> {
        RNdArrayOps::shape(&self.0)
    }

    fn sum(&self) -> f64 {
        RNdArrayOps::sum(&self.0)
    }

    fn mean(&self) -> f64 {
        RNdArrayOps::mean(&self.0)
    }

    fn min(&self) -> f64 {
        RNdArrayOps::min(&self.0)
    }

    fn max(&self) -> f64 {
        RNdArrayOps::max(&self.0)
    }

    fn product(&self) -> f64 {
        RNdArrayOps::product(&self.0)
    }

    fn var(&self) -> f64 {
        RNdArrayOps::var(&self.0)
    }

    fn std(&self) -> f64 {
        RNdArrayOps::std(&self.0)
    }

    // --- RNdSlice methods ---
    /// Get the element at the given 1-based index.
    /// @param index 1-based index. Errors if out of `1..=length`.
    fn get(&self, index: i32) -> f64 {
        let len = self.0.len();
        if index < 1 {
            panic!(
                "index {index} is out of bounds (must be a positive 1-based index, length {len})"
            );
        }
        RNdSlice::get(&self.0, index - 1)
            .unwrap_or_else(|| panic!("index {index} out of bounds (length {len})"))
    }

    fn first(&self) -> Option<f64> {
        RNdSlice::first(&self.0)
    }

    fn last(&self) -> Option<f64> {
        RNdSlice::last(&self.0)
    }

    /// Extract the elements `start:end` (1-based, inclusive on both ends —
    /// R's `x[start:end]` convention).
    /// @param start 1-based inclusive start index.
    /// @param end 1-based inclusive end index. Errors unless `1 <= start <= end <= length`.
    fn slice_1d(&self, start: i32, end: i32) -> Vec<f64> {
        let len = RNdArrayOps::len(&self.0);
        if start < 1 || start > end || end > len {
            panic!(
                "slice bounds [{start}, {end}] are out of bounds (must satisfy 1 <= start <= end <= length, length {len})"
            );
        }
        // 1-based inclusive -> trait's 0-based half-open: [start - 1, end)
        RNdSlice::slice_1d(&self.0, start - 1, end)
    }

    /// Get elements at the given 1-based indices.
    /// @param indices 1-based indices. Out-of-bounds indices yield `NA`.
    fn get_many(&self, indices: Vec<i32>) -> Vec<Option<f64>> {
        RNdSlice::get_many(&self.0, indices.into_iter().map(|i| i - 1).collect())
    }

    /// Check if the given 1-based index is valid.
    fn is_valid_index(&self, index: i32) -> bool {
        index >= 1 && RNdSlice::is_valid_index(&self.0, index - 1)
    }

    // --- Conversion ---
    fn to_r(&self) -> Array1<f64> {
        self.0.clone()
    }

    fn view_to_r(&self) -> Vec<f64> {
        // Return the view converted to R (tests ArrayView1::IntoR)
        let view: ArrayView1<f64> = self.0.view();
        view.iter().cloned().collect()
    }
}
// endregion

// region: NdMatrix - Wrapper for Array2<f64>

/// A 2D numeric array (matrix) wrapper for testing RNdArrayOps and RNdSlice2D.
#[derive(ExternalPtr)]
pub struct NdMatrix(Array2<f64>);

/// NdMatrix methods: 2D numeric matrix operations, row/col access, and conversion.
/// @param nrow Integer number of rows.
/// @param ncol Integer number of columns.
/// @param data Numeric vector of matrix elements (in row-major order).
/// @details `get_2d`, `row`, and `col` take 1-based indices (R-idiomatic,
///   matching the rest of the package) and error on an out-of-bounds or
///   non-positive index.
// env pinned: method names collide with base R non-generics (var/get/row/col/
// diag/reshape), which the s7-default flip cannot register S7 methods on (#1114).
#[miniextendr(env)]
impl NdMatrix {
    fn new(data: Array2<f64>) -> Self {
        NdMatrix(data)
    }

    fn from_rows(nrow: i32, ncol: i32, data: Vec<f64>) -> Self {
        let arr =
            Array2::from_shape_vec((nrow as usize, ncol as usize), data).expect("Shape mismatch");
        NdMatrix(arr)
    }

    fn identity(n: i32) -> Self {
        let mut arr = Array2::zeros((n as usize, n as usize));
        for i in 0..n as usize {
            arr[[i, i]] = 1.0;
        }
        NdMatrix(arr)
    }

    // --- RNdArrayOps methods ---
    fn len(&self) -> i32 {
        RNdArrayOps::len(&self.0)
    }

    fn is_empty(&self) -> bool {
        RNdArrayOps::is_empty(&self.0)
    }

    fn ndim(&self) -> i32 {
        RNdArrayOps::ndim(&self.0)
    }

    fn shape(&self) -> Vec<i32> {
        RNdArrayOps::shape(&self.0)
    }

    fn sum(&self) -> f64 {
        RNdArrayOps::sum(&self.0)
    }

    fn mean(&self) -> f64 {
        RNdArrayOps::mean(&self.0)
    }

    fn min(&self) -> f64 {
        RNdArrayOps::min(&self.0)
    }

    fn max(&self) -> f64 {
        RNdArrayOps::max(&self.0)
    }

    fn product(&self) -> f64 {
        RNdArrayOps::product(&self.0)
    }

    fn var(&self) -> f64 {
        RNdArrayOps::var(&self.0)
    }

    fn std(&self) -> f64 {
        RNdArrayOps::std(&self.0)
    }

    // --- RNdSlice2D methods ---
    /// Get the element at (row, col) (1-based).
    /// @param row 1-based row index. Errors if out of `1..=nrows()`.
    /// @param col 1-based column index. Errors if out of `1..=ncols()`.
    fn get_2d(&self, row: i32, col: i32) -> f64 {
        let nrows = RNdSlice2D::nrows(&self.0);
        let ncols = RNdSlice2D::ncols(&self.0);
        if row < 1 || col < 1 {
            panic!(
                "[{row}, {col}] is out of bounds (must be positive 1-based indices, matrix is {nrows} x {ncols})"
            );
        }
        RNdSlice2D::get_2d(&self.0, row - 1, col - 1)
            .unwrap_or_else(|| panic!("[{row}, {col}] out of bounds (matrix is {nrows} x {ncols})"))
    }

    /// Get a row as a vector (1-based).
    /// @param row 1-based row index. Errors if out of `1..=nrows()`.
    fn row(&self, row: i32) -> Vec<f64> {
        let nrows = RNdSlice2D::nrows(&self.0);
        if row < 1 || row > nrows {
            panic!(
                "row {row} is out of bounds (must be a positive 1-based index, matrix has {nrows} rows)"
            );
        }
        RNdSlice2D::row(&self.0, row - 1)
    }

    /// Get a column as a vector (1-based).
    /// @param col 1-based column index. Errors if out of `1..=ncols()`.
    fn col(&self, col: i32) -> Vec<f64> {
        let ncols = RNdSlice2D::ncols(&self.0);
        if col < 1 || col > ncols {
            panic!(
                "col {col} is out of bounds (must be a positive 1-based index, matrix has {ncols} columns)"
            );
        }
        RNdSlice2D::col(&self.0, col - 1)
    }

    fn diag(&self) -> Vec<f64> {
        RNdSlice2D::diag(&self.0)
    }

    fn nrows(&self) -> i32 {
        RNdSlice2D::nrows(&self.0)
    }

    fn ncols(&self) -> i32 {
        RNdSlice2D::ncols(&self.0)
    }

    // --- Conversion ---
    fn to_r(&self) -> Array2<f64> {
        self.0.clone()
    }

    fn view_to_r(&self) -> Array2<f64> {
        // Return the view converted to R (tests ArrayView2::IntoR)
        let view: ArrayView2<f64> = self.0.view();
        view.to_owned()
    }
}
// endregion

// region: NdArrayDyn - Wrapper for ArrayD<f64>

/// An N-dimensional array wrapper for testing RNdIndex.
#[derive(ExternalPtr)]
pub struct NdArrayDyn(ArrayD<f64>);

/// NdArrayDyn methods: N-dimensional array operations, indexing, and reshaping.
/// @param shape Integer vector specifying array dimensions.
/// @param data Numeric vector of array elements.
/// @details `get_nd`, `is_valid_nd`, `slice_nd`, and `axis_slice` take
///   1-based indices (R-idiomatic, matching the rest of the package).
///   `slice_nd` takes 1-based inclusive per-dimension bounds (R's
///   `x[s1:e1, s2:e2, ...]` convention); `axis_slice`'s `axis` follows R's
///   `MARGIN` convention (1 = first dimension). `get_nd`, `reshape`,
///   `slice_nd`, and `axis_slice` error on an out-of-bounds/invalid argument
///   instead of silently yielding no value.
// env pinned: method names collide with base R non-generics (var/get/row/col/
// diag/reshape), which the s7-default flip cannot register S7 methods on (#1114).
#[miniextendr(env)]
impl NdArrayDyn {
    fn new(shape: Vec<i32>, data: Vec<f64>) -> Self {
        let shape_usize: Vec<usize> = shape.iter().map(|&d| d as usize).collect();
        let arr =
            ArrayD::from_shape_vec(IxDyn(&shape_usize), data).expect("Shape/data length mismatch");
        NdArrayDyn(arr)
    }

    fn zeros(shape: Vec<i32>) -> Self {
        let shape_usize: Vec<usize> = shape.iter().map(|&d| d as usize).collect();
        NdArrayDyn(ArrayD::zeros(IxDyn(&shape_usize)))
    }

    fn ones(shape: Vec<i32>) -> Self {
        let shape_usize: Vec<usize> = shape.iter().map(|&d| d as usize).collect();
        NdArrayDyn(ArrayD::ones(IxDyn(&shape_usize)))
    }

    // --- RNdArrayOps methods ---
    fn len(&self) -> i32 {
        RNdArrayOps::len(&self.0)
    }

    fn is_empty(&self) -> bool {
        RNdArrayOps::is_empty(&self.0)
    }

    fn sum(&self) -> f64 {
        RNdArrayOps::sum(&self.0)
    }

    fn mean(&self) -> f64 {
        RNdArrayOps::mean(&self.0)
    }

    fn min(&self) -> f64 {
        RNdArrayOps::min(&self.0)
    }

    fn max(&self) -> f64 {
        RNdArrayOps::max(&self.0)
    }

    fn product(&self) -> f64 {
        RNdArrayOps::product(&self.0)
    }

    fn var(&self) -> f64 {
        RNdArrayOps::var(&self.0)
    }

    fn std(&self) -> f64 {
        RNdArrayOps::std(&self.0)
    }

    // --- RNdIndex methods ---
    /// Get the element at the given n-dimensional 1-based index.
    /// @param indices 1-based indices, one per dimension. Errors on a non-positive index, an out-of-bounds index, or a length mismatch with the array's dimensionality.
    fn get_nd(&self, indices: Vec<i32>) -> f64 {
        let shape = RNdIndex::shape_nd(&self.0);
        if indices.iter().any(|&i| i < 1) {
            panic!("indices {indices:?} must be positive 1-based indices (shape {shape:?})");
        }
        let zero_based: Vec<i32> = indices.iter().map(|&i| i - 1).collect();
        RNdIndex::get_nd(&self.0, zero_based)
            .unwrap_or_else(|| panic!("indices {indices:?} out of bounds (shape {shape:?})"))
    }

    /// Extract a subarray bounded by `start` and `end` (1-based, inclusive on
    /// both ends per dimension — R's `x[s1:e1, s2:e2, ...]` convention).
    /// Elements are returned flattened in column-major (R) order.
    /// @param start 1-based inclusive start indices, one per dimension.
    /// @param end 1-based inclusive end indices, one per dimension. Errors on a dimensionality mismatch or when any dimension violates `1 <= start <= end <= extent`.
    fn slice_nd(&self, start: Vec<i32>, end: Vec<i32>) -> Vec<f64> {
        let shape = RNdIndex::shape_nd(&self.0);
        if start.len() != shape.len() || end.len() != shape.len() {
            panic!(
                "start {start:?} and end {end:?} must each have one bound per dimension (shape {shape:?})"
            );
        }
        for (i, &extent) in shape.iter().enumerate() {
            let (s, e) = (start[i], end[i]);
            if s < 1 || s > e || e > extent {
                panic!(
                    "slice bounds start {start:?}, end {end:?} are out of bounds (each dimension must satisfy 1 <= start <= end <= extent, shape {shape:?})"
                );
            }
        }
        // 1-based inclusive -> trait's 0-based half-open: [start - 1, end)
        let zero_based_start: Vec<i32> = start.iter().map(|&s| s - 1).collect();
        RNdIndex::slice_nd(&self.0, zero_based_start, end).expect("bounds validated above")
    }

    fn shape_nd(&self) -> Vec<i32> {
        RNdIndex::shape_nd(&self.0)
    }

    fn ndim(&self) -> i32 {
        RNdIndex::ndim(&self.0)
    }

    fn len_nd(&self) -> i32 {
        RNdIndex::len_nd(&self.0)
    }

    fn flatten(&self) -> Vec<f64> {
        RNdIndex::flatten(&self.0)
    }

    fn flatten_c(&self) -> Vec<f64> {
        RNdIndex::flatten_c(&self.0)
    }

    /// Check if the given 1-based n-dimensional index is valid.
    fn is_valid_nd(&self, indices: Vec<i32>) -> bool {
        indices.iter().all(|&i| i >= 1)
            && RNdIndex::is_valid_nd(&self.0, indices.into_iter().map(|i| i - 1).collect())
    }

    /// Get all elements where dimension `axis` is fixed at position `index`.
    /// Both arguments are 1-based: `axis` follows R's `MARGIN` convention
    /// (1 = first dimension), and `index` matches `row`/`col`. Elements are
    /// returned in column-major (R) order.
    /// @param axis 1-based dimension selector (R's `MARGIN` convention). Errors if out of `1..=ndim`.
    /// @param index 1-based position along that dimension. Errors if out of `1..=extent`.
    fn axis_slice(&self, axis: i32, index: i32) -> Vec<f64> {
        let shape = RNdIndex::shape_nd(&self.0);
        let ndim = RNdIndex::ndim(&self.0);
        if axis < 1 || axis > ndim {
            panic!(
                "axis {axis} is out of bounds (must be a positive 1-based dimension selector, array has {ndim} dimensions)"
            );
        }
        let extent = shape[usize::try_from(axis - 1).expect("axis >= 1 checked above")];
        if index < 1 || index > extent {
            panic!(
                "index {index} is out of bounds along axis {axis} (must be a positive 1-based index, extent {extent})"
            );
        }
        RNdIndex::axis_slice(&self.0, axis - 1, index - 1)
    }

    /// Reshape the array to new dimensions (data must fit exactly).
    /// @param new_shape Target dimensions. Errors if the total element count doesn't match the array's current length.
    fn reshape(&self, new_shape: Vec<i32>) -> Vec<f64> {
        let len = RNdIndex::len_nd(&self.0);
        RNdIndex::reshape(&self.0, new_shape.clone()).unwrap_or_else(|| {
            panic!("cannot reshape to {new_shape:?}: element count does not match length {len}")
        })
    }

    // --- Conversion ---
    fn to_r(&self) -> ArrayD<f64> {
        self.0.clone()
    }
}
// endregion

// region: NdIntVec - Wrapper for Array1<i32> (tests i32 implementations)

/// A 1D integer array wrapper for testing i32 adapter traits.
#[derive(ExternalPtr)]
pub struct NdIntVec(Array1<i32>);

/// NdIntVec methods: 1D integer array operations, slicing, and conversion.
/// @param data Integer vector of array elements.
/// @details `get` and `slice_1d` take 1-based indices (R-idiomatic, matching
///   the rest of the package) and error on out-of-bounds or non-positive
///   indices. `slice_1d` takes 1-based inclusive bounds (R's `x[start:end]`
///   convention) and errors unless `1 <= start <= end <= length`.
// env pinned: method names collide with base R non-generics (var/get/row/col/
// diag/reshape), which the s7-default flip cannot register S7 methods on (#1114).
#[miniextendr(env)]
impl NdIntVec {
    fn new(data: Vec<i32>) -> Self {
        NdIntVec(Array1::from_vec(data))
    }

    // --- RNdArrayOps methods (returns f64) ---
    fn len(&self) -> i32 {
        RNdArrayOps::len(&self.0)
    }

    fn is_empty(&self) -> bool {
        RNdArrayOps::is_empty(&self.0)
    }

    fn ndim(&self) -> i32 {
        RNdArrayOps::ndim(&self.0)
    }

    fn shape(&self) -> Vec<i32> {
        RNdArrayOps::shape(&self.0)
    }

    fn sum(&self) -> f64 {
        RNdArrayOps::sum(&self.0)
    }

    fn mean(&self) -> f64 {
        RNdArrayOps::mean(&self.0)
    }

    fn min(&self) -> f64 {
        RNdArrayOps::min(&self.0)
    }

    fn max(&self) -> f64 {
        RNdArrayOps::max(&self.0)
    }

    fn product(&self) -> f64 {
        RNdArrayOps::product(&self.0)
    }

    fn var(&self) -> f64 {
        RNdArrayOps::var(&self.0)
    }

    fn std(&self) -> f64 {
        RNdArrayOps::std(&self.0)
    }

    // --- RNdSlice methods ---
    /// Get the element at the given 1-based index.
    /// @param index 1-based index. Errors if out of `1..=length`.
    fn get(&self, index: i32) -> i32 {
        let len = self.0.len();
        if index < 1 {
            panic!(
                "index {index} is out of bounds (must be a positive 1-based index, length {len})"
            );
        }
        RNdSlice::get(&self.0, index - 1)
            .unwrap_or_else(|| panic!("index {index} out of bounds (length {len})"))
    }

    fn first(&self) -> Option<i32> {
        RNdSlice::first(&self.0)
    }

    fn last(&self) -> Option<i32> {
        RNdSlice::last(&self.0)
    }

    /// Extract the elements `start:end` (1-based, inclusive on both ends —
    /// R's `x[start:end]` convention).
    /// @param start 1-based inclusive start index.
    /// @param end 1-based inclusive end index. Errors unless `1 <= start <= end <= length`.
    fn slice_1d(&self, start: i32, end: i32) -> Vec<i32> {
        let len = RNdArrayOps::len(&self.0);
        if start < 1 || start > end || end > len {
            panic!(
                "slice bounds [{start}, {end}] are out of bounds (must satisfy 1 <= start <= end <= length, length {len})"
            );
        }
        // 1-based inclusive -> trait's 0-based half-open: [start - 1, end)
        RNdSlice::slice_1d(&self.0, start - 1, end)
    }

    // --- Conversion ---
    fn to_r(&self) -> Array1<i32> {
        self.0.clone()
    }
}
// endregion

// region: Helper functions for testing conversions

/// Test roundtripping a 1D numeric array through R and back.
/// @param data A numeric vector to roundtrip.
#[miniextendr]
pub fn ndarray_roundtrip_vec(data: Array1<f64>) -> Array1<f64> {
    data
}

/// Test roundtripping a 2D numeric matrix through R and back.
/// @param data A numeric matrix to roundtrip.
#[miniextendr]
pub fn ndarray_roundtrip_matrix(data: Array2<f64>) -> Array2<f64> {
    data
}

/// Test roundtripping an N-dimensional numeric array through R and back.
/// @param data An N-dimensional numeric array to roundtrip.
#[miniextendr]
pub fn ndarray_roundtrip_array(data: ArrayD<f64>) -> ArrayD<f64> {
    data
}

/// Test roundtripping a 1D integer array through R and back.
/// @param data An integer vector to roundtrip.
#[miniextendr]
pub fn ndarray_roundtrip_int_vec(data: Array1<i32>) -> Array1<i32> {
    data
}

/// Test roundtripping a 2D integer matrix through R and back.
/// @param data An integer matrix to roundtrip.
#[miniextendr]
pub fn ndarray_roundtrip_int_matrix(data: Array2<i32>) -> Array2<i32> {
    data
}
// endregion
