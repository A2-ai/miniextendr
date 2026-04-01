//! Test fixtures for ExternalSlice.

use miniextendr_api::externalptr::{ExternalPtr, ExternalSlice};
use miniextendr_api::prelude::*;

/// Newtype wrapper to satisfy orphan rule.
#[derive(ExternalPtr)]
pub struct F64Slice(ExternalSlice<f64>);

/// Create an ExternalSlice from a Vec, return its length.
#[miniextendr]
pub fn external_slice_len(data: Vec<f64>) -> i32 {
    let slice = F64Slice(ExternalSlice::new(data));
    let ptr = ExternalPtr::new(slice);
    ptr.0.len() as i32
}

/// Create an ExternalSlice, access elements via as_slice.
#[miniextendr]
pub fn external_slice_sum(data: Vec<f64>) -> f64 {
    let slice = ExternalSlice::new(data);
    slice.as_slice().iter().sum()
}
