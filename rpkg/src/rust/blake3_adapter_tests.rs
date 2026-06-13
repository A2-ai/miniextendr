//! BLAKE3 adapter tests
use miniextendr_api::blake3_impl::{blake3_bytes, blake3_str};
use miniextendr_api::miniextendr;

/// Test BLAKE3 hashing of a string input.
/// @param input String to hash.
#[miniextendr]
pub fn blake3_hash(input: String) -> String {
    blake3_str(&input)
}

/// Test BLAKE3 hashing of raw bytes, returning the 32-byte digest.
/// @param input Raw bytes to hash.
#[miniextendr]
pub fn blake3_hash_bytes(input: Vec<u8>) -> Vec<u8> {
    blake3_bytes(&input)
}

/// Test that BLAKE3 hex output is always 64 characters.
#[miniextendr]
pub fn blake3_len() -> i32 {
    // BLAKE3 hex string is always 64 chars
    blake3_str("test").len() as i32
}

/// Test that the BLAKE3 raw digest is always 32 bytes.
#[miniextendr]
pub fn blake3_bytes_len() -> i32 {
    // BLAKE3 digest is always 32 bytes
    blake3_bytes(b"test").len() as i32
}

/// Test BLAKE3 hash of a known string for correctness verification.
#[miniextendr]
pub fn blake3_hello() -> String {
    blake3_str("hello world")
}

/// Test BLAKE3 hashing of a large repeated input (100k characters).
#[miniextendr]
pub fn blake3_large() -> String {
    let large_input = "a".repeat(100_000);
    blake3_str(&large_input)
}

/// Test BLAKE3 hashing of content with special characters.
#[miniextendr]
pub fn blake3_binary_content() -> String {
    blake3_str("\t\n\r ~!@#$%^&*()")
}

/// Test that two different inputs produce different BLAKE3 hashes.
#[miniextendr]
pub fn blake3_different_inputs_differ() -> bool {
    blake3_str("input1") != blake3_str("input2")
}
