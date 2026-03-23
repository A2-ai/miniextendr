//! Test fixtures for Lazy<T> ALTREP wrappers.

use miniextendr_api::into_r::Altrep;
type Lazy<T> = Altrep<T>;
use miniextendr_api::miniextendr;

// region: Lazy<Vec<T>> (already supported)

/// @export
#[miniextendr]
pub fn test_lazy_vec_f64(n: i32) -> Lazy<Vec<f64>> {
    Altrep((0..n).map(|i| (i as f64).sin()).collect())
}

/// @export
#[miniextendr]
pub fn test_lazy_vec_i32(n: i32) -> Lazy<Vec<i32>> {
    Altrep((0..n).map(|i| i * i).collect())
}

// endregion

// region: Lazy<Arrow arrays>

#[cfg(feature = "arrow")]
use miniextendr_api::optionals::arrow_impl::{Float64Array, Int32Array, BooleanArray};

/// @export
#[cfg(feature = "arrow")]
#[miniextendr]
pub fn test_lazy_arrow_f64(n: i32) -> Lazy<Float64Array> {
    let values: Vec<f64> = (0..n).map(|i| (i as f64) * 1.5).collect();
    Altrep(Float64Array::from(values))
}

/// @export
#[cfg(feature = "arrow")]
#[miniextendr]
pub fn test_lazy_arrow_i32(n: i32) -> Lazy<Int32Array> {
    let values: Vec<i32> = (0..n).map(|i| i * 10).collect();
    Altrep(Int32Array::from(values))
}

/// @export
#[cfg(feature = "arrow")]
#[miniextendr]
pub fn test_lazy_arrow_bool(n: i32) -> Lazy<BooleanArray> {
    let values: Vec<bool> = (0..n).map(|i| i % 2 == 0).collect();
    Altrep(BooleanArray::from(values))
}

/// @export
#[cfg(feature = "arrow")]
#[miniextendr]
pub fn test_lazy_arrow_f64_with_nulls() -> Lazy<Float64Array> {
    let values: Vec<Option<f64>> = vec![Some(1.0), None, Some(3.0), None, Some(5.0)];
    Altrep(Float64Array::from(values))
}

// endregion

// region: Lazy<ndarray::Array1<T>>

#[cfg(feature = "ndarray")]
use miniextendr_api::optionals::ndarray_impl::Array1;

/// @export
#[cfg(feature = "ndarray")]
#[miniextendr]
pub fn test_lazy_ndarray_f64(n: i32) -> Lazy<Array1<f64>> {
    Altrep(Array1::from_iter((0..n).map(|i| (i as f64) * 0.5)))
}

/// @export
#[cfg(feature = "ndarray")]
#[miniextendr]
pub fn test_lazy_ndarray_i32(n: i32) -> Lazy<Array1<i32>> {
    Altrep(Array1::from_iter(0..n))
}

// endregion

// region: Lazy<nalgebra::DVector<T>>

#[cfg(feature = "nalgebra")]
use miniextendr_api::optionals::nalgebra_impl::DVector;

/// @export
#[cfg(feature = "nalgebra")]
#[miniextendr]
pub fn test_lazy_nalgebra_f64(n: i32) -> Lazy<DVector<f64>> {
    Altrep(DVector::from_iterator(n as usize, (0..n).map(|i| (i as f64) * 2.0)))
}

/// @export
#[cfg(feature = "nalgebra")]
#[miniextendr]
pub fn test_lazy_nalgebra_i32(n: i32) -> Lazy<DVector<i32>> {
    Altrep(DVector::from_iterator(n as usize, 0..n))
}

// endregion
