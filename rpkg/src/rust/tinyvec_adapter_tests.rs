//! TinyVec adapter tests
use miniextendr_api::tinyvec_impl::{ArrayVec, TinyVec};
use miniextendr_api::{miniextendr, miniextendr_module};

/// @noRd
#[miniextendr]
pub fn tinyvec_roundtrip_int(v: TinyVec<[i32; 8]>) -> TinyVec<[i32; 8]> {
    v
}

/// @noRd
#[miniextendr]
pub fn tinyvec_roundtrip_dbl(v: TinyVec<[f64; 8]>) -> TinyVec<[f64; 8]> {
    v
}

/// @noRd
#[miniextendr]
pub fn tinyvec_len(v: TinyVec<[i32; 8]>) -> i32 {
    v.len() as i32
}

/// @noRd
#[miniextendr]
pub fn arrayvec_roundtrip_int(v: ArrayVec<[i32; 8]>) -> ArrayVec<[i32; 8]> {
    v
}

/// @noRd
#[miniextendr]
pub fn arrayvec_roundtrip_dbl(v: ArrayVec<[f64; 4]>) -> ArrayVec<[f64; 4]> {
    v
}

miniextendr_module! {
    mod tinyvec_adapter_tests;
    fn tinyvec_roundtrip_int;
    fn tinyvec_roundtrip_dbl;
    fn tinyvec_len;
    fn arrayvec_roundtrip_int;
    fn arrayvec_roundtrip_dbl;
}
