//! Integration with the `sha2` crate for cryptographic hashing.
//!
//! Provides SHA-256 and SHA-512 hashing helpers for R.
//!
//! # Features
//!
//! Enable this module with the `sha2` feature:
//!
//! ```toml
//! [dependencies]
//! miniextendr-api = { version = "0.1", features = ["sha2"] }
//! ```
//!
//! # Example
//!
//! ```ignore
//! use miniextendr_api::sha2_impl::{sha256_str, sha512_str};
//!
//! #[miniextendr]
//! fn hash_password(password: &str) -> String {
//!     sha256_str(password)
//! }
//! ```
//!
//! # Functions
//!
//! | Function | Input | Output |
//! |----------|-------|--------|
//! | `sha256_bytes` | `&[u8]` | 64-char hex string |
//! | `sha256_str` | `&str` | 64-char hex string |
//! | `sha512_bytes` | `&[u8]` | 128-char hex string |
//! | `sha512_str` | `&str` | 128-char hex string |

// Re-export core types for advanced usage
pub use sha2::{Digest, Sha256, Sha512};

// region: Core hashing functions

/// Compute SHA-256 hash of raw bytes.
///
/// Returns a 64-character lowercase hex string.
///
/// # Example
///
/// ```ignore
/// let hash = sha256_bytes(b"hello world");
/// assert_eq!(hash.len(), 64);
/// ```
#[inline]
pub fn sha256_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex_encode(&result)
}

/// Compute SHA-256 hash of a UTF-8 string.
///
/// Returns a 64-character lowercase hex string.
///
/// # Example
///
/// ```ignore
/// let hash = sha256_str("hello world");
/// assert_eq!(hash, "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9");
/// ```
#[inline]
pub fn sha256_str(s: &str) -> String {
    sha256_bytes(s.as_bytes())
}

/// Compute SHA-512 hash of raw bytes.
///
/// Returns a 128-character lowercase hex string.
///
/// # Example
///
/// ```ignore
/// let hash = sha512_bytes(b"hello world");
/// assert_eq!(hash.len(), 128);
/// ```
#[inline]
pub fn sha512_bytes(data: &[u8]) -> String {
    let mut hasher = Sha512::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex_encode(&result)
}

/// Compute SHA-512 hash of a UTF-8 string.
///
/// Returns a 128-character lowercase hex string.
///
/// # Example
///
/// ```ignore
/// let hash = sha512_str("hello world");
/// assert_eq!(hash.len(), 128);
/// ```
#[inline]
pub fn sha512_str(s: &str) -> String {
    sha512_bytes(s.as_bytes())
}
// endregion

// region: Vector helpers

/// Compute SHA-256 hashes for a vector of byte slices.
///
/// Returns a vector of 64-character lowercase hex strings.
pub fn sha256_bytes_vec(data: &[&[u8]]) -> Vec<String> {
    data.iter().map(|d| sha256_bytes(d)).collect()
}

/// Compute SHA-256 hashes for a vector of strings.
///
/// Returns a vector of 64-character lowercase hex strings.
pub fn sha256_str_vec(strings: &[&str]) -> Vec<String> {
    strings.iter().map(|s| sha256_str(s)).collect()
}

/// Compute SHA-512 hashes for a vector of byte slices.
///
/// Returns a vector of 128-character lowercase hex strings.
pub fn sha512_bytes_vec(data: &[&[u8]]) -> Vec<String> {
    data.iter().map(|d| sha512_bytes(d)).collect()
}

/// Compute SHA-512 hashes for a vector of strings.
///
/// Returns a vector of 128-character lowercase hex strings.
pub fn sha512_str_vec(strings: &[&str]) -> Vec<String> {
    strings.iter().map(|s| sha512_str(s)).collect()
}
// endregion

// region: Helper functions

/// Encode bytes as lowercase hex string.
#[inline]
fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        use std::fmt::Write;
        let _ = write!(s, "{:02x}", b);
    }
    s
}
// endregion

// region: Unit tests

#[cfg(test)]
mod tests;
// endregion
