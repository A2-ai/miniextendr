//! Bytes adapter tests
use miniextendr_api::bytes_impl::{Bytes, BytesMut};
use miniextendr_api::miniextendr;

/// Test Bytes roundtrip through R raw vector.
/// @param data Raw vector to roundtrip.
#[miniextendr]
pub fn bytes_roundtrip(data: Bytes) -> Bytes {
    data
}

/// Test getting the length of a Bytes buffer.
/// @param data Raw vector to measure.
#[miniextendr]
pub fn bytes_len(data: Bytes) -> i32 {
    data.len() as i32
}

/// Test BytesMut roundtrip through R raw vector.
/// @param data Raw vector to roundtrip.
#[miniextendr]
pub fn bytes_mut_roundtrip(data: BytesMut) -> BytesMut {
    data
}

/// Test concatenating two Bytes buffers.
/// @param a First raw vector.
/// @param b Second raw vector.
#[miniextendr]
pub fn bytes_concat(a: Bytes, b: Bytes) -> Bytes {
    let mut out = BytesMut::with_capacity(a.len() + b.len());
    out.extend_from_slice(&a);
    out.extend_from_slice(&b);
    out.freeze()
}

/// Test slicing a Bytes buffer by start and end indices.
/// @param data Raw vector to slice.
/// @param start Start index (0-based).
/// @param end End index (exclusive).
#[miniextendr]
pub fn bytes_slice(data: Bytes, start: i32, end: i32) -> Bytes {
    data.slice(start as usize..end as usize)
}

/// Test creating and roundtripping an empty Bytes buffer.
#[miniextendr]
pub fn bytes_empty() -> Bytes {
    Bytes::new()
}

/// Test that an empty Bytes buffer has length zero.
#[miniextendr]
pub fn bytes_empty_len() -> i32 {
    Bytes::new().len() as i32
}

/// Test roundtripping a large buffer (1000 bytes).
#[miniextendr]
pub fn bytes_large() -> Bytes {
    Bytes::from(vec![0xABu8; 1000])
}

/// Test roundtripping all 256 byte values (0x00 through 0xFF).
#[miniextendr]
pub fn bytes_all_values() -> Bytes {
    let data: Vec<u8> = (0..=255).collect();
    Bytes::from(data)
}
