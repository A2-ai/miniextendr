//! Bytes adapter tests
use miniextendr_api::bytes_impl::{Bytes, BytesMut};
use miniextendr_api::{miniextendr, miniextendr_module};

/// @noRd
#[miniextendr]
pub fn bytes_roundtrip(data: Bytes) -> Bytes {
    data
}

/// @noRd
#[miniextendr]
pub fn bytes_len(data: Bytes) -> i32 {
    data.len() as i32
}

/// @noRd
#[miniextendr]
pub fn bytes_mut_roundtrip(data: BytesMut) -> BytesMut {
    data
}

/// @noRd
#[miniextendr]
pub fn bytes_concat(a: Bytes, b: Bytes) -> Bytes {
    let mut out = BytesMut::with_capacity(a.len() + b.len());
    out.extend_from_slice(&a);
    out.extend_from_slice(&b);
    out.freeze()
}

/// @noRd
#[miniextendr]
pub fn bytes_slice(data: Bytes, start: i32, end: i32) -> Bytes {
    data.slice(start as usize..end as usize)
}

miniextendr_module! {
    mod bytes_adapter_tests;
    fn bytes_roundtrip;
    fn bytes_len;
    fn bytes_mut_roundtrip;
    fn bytes_concat;
    fn bytes_slice;
}
