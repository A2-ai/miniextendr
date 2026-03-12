//! Bytes adapter tests
use miniextendr_api::bytes_impl::{Bytes, BytesMut};
use miniextendr_api::miniextendr;

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

/// Empty bytes roundtrip
/// @noRd
#[miniextendr]
pub fn bytes_empty() -> Bytes {
    Bytes::new()
}

/// Empty bytes length
/// @noRd
#[miniextendr]
pub fn bytes_empty_len() -> i32 {
    Bytes::new().len() as i32
}

/// Large buffer roundtrip (1000 bytes)
/// @noRd
#[miniextendr]
pub fn bytes_large() -> Bytes {
    Bytes::from(vec![0xABu8; 1000])
}

/// Binary roundtrip: all byte values 0x00..0xFF
/// @noRd
#[miniextendr]
pub fn bytes_all_values() -> Bytes {
    let data: Vec<u8> = (0..=255).collect();
    Bytes::from(data)
}
