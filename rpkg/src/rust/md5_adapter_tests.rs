//! MD5 adapter tests.
//!
//! MD5 is cryptographically broken — exposed for interop/checksums only.
use miniextendr_api::miniextendr;

/// MD5 hash of a string, as 32-char hex. Interop only — MD5 is broken crypto.
/// @param input String to hash.
#[miniextendr]
pub fn md5_str(input: String) -> String {
    miniextendr_api::md5_impl::md5_str(&input)
}

/// MD5 hash of a raw vector, as a 16-byte raw digest.
/// @param data Raw vector to hash.
#[miniextendr]
pub fn md5_bytes(data: Vec<u8>) -> Vec<u8> {
    miniextendr_api::md5_impl::md5_bytes(&data)
}

/// MD5 hash of a raw vector, as 32-char hex.
/// @param data Raw vector to hash.
#[miniextendr]
pub fn md5_hex(data: Vec<u8>) -> String {
    miniextendr_api::md5_impl::md5_hex(&data)
}
