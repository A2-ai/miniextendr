//! TinyVec adapter tests
use miniextendr_api::miniextendr;
use miniextendr_api::tinyvec_impl::{ArrayVec, TinyVec};

/// Test `TinyVec<[i32; 8]>` roundtrip through R.
/// @param v Integer vector from R.
#[miniextendr]
pub fn tinyvec_roundtrip_int(v: TinyVec<[i32; 8]>) -> TinyVec<[i32; 8]> {
    v
}

/// Test `TinyVec<[f64; 8]>` roundtrip through R.
/// @param v Numeric vector from R.
#[miniextendr]
pub fn tinyvec_roundtrip_dbl(v: TinyVec<[f64; 8]>) -> TinyVec<[f64; 8]> {
    v
}

/// Test getting the length of a TinyVec.
/// @param v Integer vector from R.
#[miniextendr]
pub fn tinyvec_len(v: TinyVec<[i32; 8]>) -> i32 {
    v.len() as i32
}

/// Test `ArrayVec<[i32; 8]>` roundtrip through R.
/// @param v Integer vector from R (max 8 elements).
#[miniextendr]
pub fn arrayvec_roundtrip_int(v: ArrayVec<[i32; 8]>) -> ArrayVec<[i32; 8]> {
    v
}

/// Test `ArrayVec<[f64; 4]>` roundtrip through R.
/// @param v Numeric vector from R (max 4 elements).
#[miniextendr]
pub fn arrayvec_roundtrip_dbl(v: ArrayVec<[f64; 4]>) -> ArrayVec<[f64; 4]> {
    v
}

/// Test creating and roundtripping an empty TinyVec.
#[miniextendr]
pub fn tinyvec_empty() -> TinyVec<[i32; 8]> {
    TinyVec::new()
}

/// Test TinyVec at inline capacity (8 elements fits inline).
#[miniextendr]
pub fn tinyvec_at_capacity() -> TinyVec<[i32; 8]> {
    let mut tv = TinyVec::new();
    for i in 1..=8 {
        tv.push(i);
    }
    tv
}

/// Test TinyVec exceeding inline capacity (spills to heap).
#[miniextendr]
pub fn tinyvec_over_capacity() -> TinyVec<[i32; 8]> {
    let mut tv = TinyVec::new();
    for i in 1..=20 {
        tv.push(i);
    }
    tv
}

/// Test creating and roundtripping an empty ArrayVec.
#[miniextendr]
pub fn arrayvec_empty() -> ArrayVec<[i32; 8]> {
    ArrayVec::new()
}
