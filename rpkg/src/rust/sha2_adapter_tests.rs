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

/// Hash of a known string to verify correctness
/// @noRd
#[miniextendr]
pub fn sha2_sha256_hello() -> String {
    sha256_str("hello world")
}

/// SHA-256 of a large repeated input (deterministic)
/// @noRd
#[miniextendr]
pub fn sha2_sha256_large() -> String {
    let large_input = "a".repeat(100_000);
    sha256_str(&large_input)
}

/// SHA-256 of binary-like content (non-UTF8 safe string with special chars)
/// @noRd
#[miniextendr]
pub fn sha2_sha256_binary_content() -> String {
    sha256_str("\t\n\r ~!@#$%^&*()")
}

/// Two different inputs produce different hashes
/// @noRd
#[miniextendr]
pub fn sha2_different_inputs_differ() -> bool {
    sha256_str("input1") != sha256_str("input2")
}

miniextendr_module! {
    mod sha2_adapter_tests;
    fn sha2_sha256;
    fn sha2_sha512;
    fn sha2_sha256_len;
    fn sha2_sha512_len;
    fn sha2_sha256_hello;
    fn sha2_sha256_large;
    fn sha2_sha256_binary_content;
    fn sha2_different_inputs_differ;
}
