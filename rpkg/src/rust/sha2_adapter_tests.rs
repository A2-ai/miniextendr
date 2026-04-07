//! SHA-2 adapter tests
use miniextendr_api::miniextendr;
use miniextendr_api::sha2_impl::{sha256_str, sha512_str};

/// Test SHA-256 hashing of a string input.
/// @param input String to hash.
#[miniextendr]
pub fn sha2_sha256(input: String) -> String {
    sha256_str(&input)
}

/// Test SHA-512 hashing of a string input.
/// @param input String to hash.
#[miniextendr]
pub fn sha2_sha512(input: String) -> String {
    sha512_str(&input)
}

/// Test that SHA-256 hex output is always 64 characters.
#[miniextendr]
pub fn sha2_sha256_len() -> i32 {
    // SHA-256 hex string is always 64 chars
    sha256_str("test").len() as i32
}

/// Test that SHA-512 hex output is always 128 characters.
#[miniextendr]
pub fn sha2_sha512_len() -> i32 {
    // SHA-512 hex string is always 128 chars
    sha512_str("test").len() as i32
}

/// Test SHA-256 hash of a known string for correctness verification.
#[miniextendr]
pub fn sha2_sha256_hello() -> String {
    sha256_str("hello world")
}

/// Test SHA-256 hashing of a large repeated input (100k characters).
#[miniextendr]
pub fn sha2_sha256_large() -> String {
    let large_input = "a".repeat(100_000);
    sha256_str(&large_input)
}

/// Test SHA-256 hashing of content with special characters.
#[miniextendr]
pub fn sha2_sha256_binary_content() -> String {
    sha256_str("\t\n\r ~!@#$%^&*()")
}

/// Test that two different inputs produce different SHA-256 hashes.
#[miniextendr]
pub fn sha2_different_inputs_differ() -> bool {
    sha256_str("input1") != sha256_str("input2")
}
