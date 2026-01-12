//! Tests for ndarray integration.
//!
//! This module provides wrapper types that expose ndarray arrays to R
//! along with adapter trait methods for testing.

use miniextendr_api::ndarray_impl::{
    Array1, Array2, ArrayD, ArrayView1, ArrayView2, IxDyn, RNdArrayOps, RNdIndex, RNdSlice,
    RNdSlice2D,
};
use miniextendr_api::{ExternalPtr, miniextendr, miniextendr_module};

// =============================================================================
// NdVec - Wrapper for Array1<f64>
// =============================================================================

/// A 1D numeric array wrapper for testing RNdArrayOps and RNdSlice.
#[derive(ExternalPtr)]
pub struct NdVec(Array1<f64>);

/// NdVec: 1D ndarray wrapper
///
/// @name NdVec
/// @title 1D ndarray wrapper for testing
/// @description Wraps a 1D ndarray and exposes adapter trait methods.
/// @return An ExternalPtr to an NdVec.
/// @examples
/// v <- NdVec$new(c(1, 2, 3, 4, 5))
/// v$len()
/// v$sum()
/// v$get(2L)
#[miniextendr]
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
    fn get(&self, index: i32) -> Option<f64> {
        RNdSlice::get(&self.0, index)
    }

    fn first(&self) -> Option<f64> {
        RNdSlice::first(&self.0)
    }

    fn last(&self) -> Option<f64> {
        RNdSlice::last(&self.0)
    }

    fn slice_1d(&self, start: i32, end: i32) -> Vec<f64> {
        RNdSlice::slice_1d(&self.0, start, end)
    }

    fn get_many(&self, indices: Vec<i32>) -> Vec<Option<f64>> {
        RNdSlice::get_many(&self.0, indices)
    }

    fn is_valid_index(&self, index: i32) -> bool {
        RNdSlice::is_valid_index(&self.0, index)
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

// =============================================================================
// NdMatrix - Wrapper for Array2<f64>
// =============================================================================

/// A 2D numeric array (matrix) wrapper for testing RNdArrayOps and RNdSlice2D.
#[derive(ExternalPtr)]
pub struct NdMatrix(Array2<f64>);

/// NdMatrix: 2D ndarray wrapper
///
/// @name NdMatrix
/// @title 2D ndarray wrapper for testing
/// @description Wraps a 2D ndarray matrix and exposes adapter trait methods.
/// @return An ExternalPtr to an NdMatrix.
/// @examples
/// m <- NdMatrix$new(matrix(c(1, 2, 3, 4, 5, 6), nrow=2, ncol=3))
/// m$nrows()
/// m$ncols()
/// m$row(0L)
#[miniextendr]
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
    fn get_2d(&self, row: i32, col: i32) -> Option<f64> {
        RNdSlice2D::get_2d(&self.0, row, col)
    }

    fn row(&self, row: i32) -> Vec<f64> {
        RNdSlice2D::row(&self.0, row)
    }

    fn col(&self, col: i32) -> Vec<f64> {
        RNdSlice2D::col(&self.0, col)
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

// =============================================================================
// NdArrayDyn - Wrapper for ArrayD<f64>
// =============================================================================

/// An N-dimensional array wrapper for testing RNdIndex.
#[derive(ExternalPtr)]
pub struct NdArrayDyn(ArrayD<f64>);

/// NdArrayDyn: N-dimensional ndarray wrapper
///
/// @name NdArrayDyn
/// @title N-dimensional ndarray wrapper for testing
/// @description Wraps an n-dimensional ndarray and exposes RNdIndex methods.
/// @return An ExternalPtr to an NdArrayDyn.
/// @examples
/// arr <- NdArrayDyn$new(c(2L, 3L, 4L), as.double(1:24))
/// arr$ndim()
/// arr$shape_nd()
/// arr$get_nd(c(0L, 1L, 2L))
#[miniextendr]
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
    fn get_nd(&self, indices: Vec<i32>) -> Option<f64> {
        RNdIndex::get_nd(&self.0, indices)
    }

    fn slice_nd(&self, start: Vec<i32>, end: Vec<i32>) -> Option<Vec<f64>> {
        RNdIndex::slice_nd(&self.0, start, end)
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

    fn is_valid_nd(&self, indices: Vec<i32>) -> bool {
        RNdIndex::is_valid_nd(&self.0, indices)
    }

    fn axis_slice(&self, axis: i32, index: i32) -> Vec<f64> {
        RNdIndex::axis_slice(&self.0, axis, index)
    }

    fn reshape(&self, new_shape: Vec<i32>) -> Option<Vec<f64>> {
        RNdIndex::reshape(&self.0, new_shape)
    }

    // --- Conversion ---
    fn to_r(&self) -> ArrayD<f64> {
        self.0.clone()
    }
}

// =============================================================================
// NdIntVec - Wrapper for Array1<i32> (tests i32 implementations)
// =============================================================================

/// A 1D integer array wrapper for testing i32 adapter traits.
#[derive(ExternalPtr)]
pub struct NdIntVec(Array1<i32>);

/// NdIntVec: 1D integer ndarray wrapper
///
/// @name NdIntVec
/// @title 1D integer ndarray wrapper for testing
/// @description Wraps a 1D integer ndarray and exposes adapter trait methods.
/// @return An ExternalPtr to an NdIntVec.
/// @examples
/// v <- NdIntVec$new(1:10)
/// v$sum()
/// v$mean()
#[miniextendr]
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
    fn get(&self, index: i32) -> Option<i32> {
        RNdSlice::get(&self.0, index)
    }

    fn first(&self) -> Option<i32> {
        RNdSlice::first(&self.0)
    }

    fn last(&self) -> Option<i32> {
        RNdSlice::last(&self.0)
    }

    fn slice_1d(&self, start: i32, end: i32) -> Vec<i32> {
        RNdSlice::slice_1d(&self.0, start, end)
    }

    // --- Conversion ---
    fn to_r(&self) -> Array1<i32> {
        self.0.clone()
    }
}

// =============================================================================
// Helper functions for testing conversions
// =============================================================================

/// Round-trip test: R vector -> Array1 -> R vector
#[miniextendr]
pub fn ndarray_roundtrip_vec(data: Array1<f64>) -> Array1<f64> {
    data
}

/// Round-trip test: R matrix -> Array2 -> R matrix
#[miniextendr]
pub fn ndarray_roundtrip_matrix(data: Array2<f64>) -> Array2<f64> {
    data
}

/// Round-trip test: R array -> ArrayD -> R array
#[miniextendr]
pub fn ndarray_roundtrip_array(data: ArrayD<f64>) -> ArrayD<f64> {
    data
}

/// Test integer array round-trip
#[miniextendr]
pub fn ndarray_roundtrip_int_vec(data: Array1<i32>) -> Array1<i32> {
    data
}

/// Test integer matrix round-trip
#[miniextendr]
pub fn ndarray_roundtrip_int_matrix(data: Array2<i32>) -> Array2<i32> {
    data
}

miniextendr_module! {
    mod ndarray_tests;

    impl NdVec;
    impl NdMatrix;
    impl NdArrayDyn;
    impl NdIntVec;

    fn ndarray_roundtrip_vec;
    fn ndarray_roundtrip_matrix;
    fn ndarray_roundtrip_array;
    fn ndarray_roundtrip_int_vec;
    fn ndarray_roundtrip_int_matrix;
}
