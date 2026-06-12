//! BLAKE3 adapter tests
use miniextendr_api::miniextendr;

/// BLAKE3 hash of a string, as 64-char hex.
/// @param input String to hash.
#[miniextendr]
pub fn blake3_str(input: String) -> String {
    miniextendr_api::blake3_impl::blake3_str(&input)
}

/// BLAKE3 hash of a raw vector, as a 32-byte raw digest.
/// @param data Raw vector to hash.
#[miniextendr]
pub fn blake3_bytes(data: Vec<u8>) -> Vec<u8> {
    miniextendr_api::blake3_impl::blake3_bytes(&data)
}

/// BLAKE3 hash of a raw vector, as 64-char hex.
/// @param data Raw vector to hash.
#[miniextendr]
pub fn blake3_hex(data: Vec<u8>) -> String {
    miniextendr_api::blake3_impl::blake3_hex(&data)
}
