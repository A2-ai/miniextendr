//! Test fixtures for ExternalSlice.

use miniextendr_api::externalptr::{ExternalPtr, ExternalSlice};
use miniextendr_api::prelude::*;

/// Create an ExternalSlice from a Vec, return its length.
#[miniextendr]
pub fn external_slice_len(data: Vec<f64>) -> i32 {
    let slice = ExternalSlice::new(data);
    let ptr = ExternalPtr::new(slice);
    i32::try_from(ptr.borrow().len()).expect("slice length fits an R integer")
}

/// Create an ExternalSlice, access elements via as_slice.
#[miniextendr]
pub fn external_slice_sum(data: Vec<f64>) -> f64 {
    let ptr = ExternalPtr::new(ExternalSlice::new(data));
    ptr.borrow().as_slice().iter().sum()
}
