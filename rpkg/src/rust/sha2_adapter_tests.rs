//! SHA-2 adapter tests
use miniextendr_api::sha2_impl::{sha256_str, sha512_str};
use miniextendr_api::{miniextendr, miniextendr_module};

/// @noRd
#[miniextendr]
pub fn sha2_sha256(input: String) -> String {
    sha256_str(&input)
}

/// @noRd
#[miniextendr]
pub fn sha2_sha512(input: String) -> String {
    sha512_str(&input)
}

/// @noRd
#[miniextendr]
pub fn sha2_sha256_len() -> i32 {
    // SHA-256 hex string is always 64 chars
    sha256_str("test").len() as i32
}

/// @noRd
#[miniextendr]
pub fn sha2_sha512_len() -> i32 {
    // SHA-512 hex string is always 128 chars
    sha512_str("test").len() as i32
}

miniextendr_module! {
    mod sha2_adapter_tests;
    fn sha2_sha256;
    fn sha2_sha512;
    fn sha2_sha256_len;
    fn sha2_sha512_len;
}
