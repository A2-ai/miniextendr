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

/// Empty TinyVec roundtrip
/// @noRd
#[miniextendr]
pub fn tinyvec_empty() -> TinyVec<[i32; 8]> {
    TinyVec::new()
}

/// TinyVec at inline capacity (8 elements fits inline)
/// @noRd
#[miniextendr]
pub fn tinyvec_at_capacity() -> TinyVec<[i32; 8]> {
    let mut tv = TinyVec::new();
    for i in 1..=8 {
        tv.push(i);
    }
    tv
}

/// TinyVec over inline capacity (spills to heap)
/// @noRd
#[miniextendr]
pub fn tinyvec_over_capacity() -> TinyVec<[i32; 8]> {
    let mut tv = TinyVec::new();
    for i in 1..=20 {
        tv.push(i);
    }
    tv
}

/// Empty ArrayVec roundtrip
/// @noRd
#[miniextendr]
pub fn arrayvec_empty() -> ArrayVec<[i32; 8]> {
    ArrayVec::new()
}

miniextendr_module! {
    mod tinyvec_adapter_tests;
    fn tinyvec_roundtrip_int;
    fn tinyvec_roundtrip_dbl;
    fn tinyvec_len;
    fn arrayvec_roundtrip_int;
    fn arrayvec_roundtrip_dbl;
    fn tinyvec_empty;
    fn tinyvec_at_capacity;
    fn tinyvec_over_capacity;
    fn arrayvec_empty;
}
