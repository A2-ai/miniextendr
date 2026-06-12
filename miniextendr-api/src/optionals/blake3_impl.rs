//! Integration with the `blake3` crate for fast cryptographic hashing.
//!
//! Provides BLAKE3 hashing helpers for R. BLAKE3 produces 32-byte digests and
//! is SIMD-accelerated on modern hardware.
//!
//! # Features
//!
//! Enable this module with the `blake3` feature:
//!
//! ```toml
//! [dependencies]
//! miniextendr-api = { version = "0.1", features = ["blake3"] }
//! ```
//!
//! # Example
//!
//! ```ignore
//! use miniextendr_api::blake3_impl::blake3_str;
//!
//! #[miniextendr]
//! fn content_key(s: &str) -> String {
//!     blake3_str(s)
//! }
//! ```
//!
//! # Functions
//!
//! | Function | Input | Output |
//! |----------|-------|--------|
//! | `blake3_str` | `&str` | 64-char lowercase hex string |
//! | `blake3_bytes` | `&[u8]` | 32-byte raw digest (`Vec<u8>`) |
//! | `blake3_hex` | `&[u8]` | 64-char lowercase hex string |

// Re-export core types for advanced usage
pub use blake3::{Hash, Hasher};

// region: Core hashing functions

/// Compute the BLAKE3 hash of a UTF-8 string.
///
/// Returns a 64-character lowercase hex string.
///
/// # Example
///
/// ```ignore
/// let hash = blake3_str("");
/// assert_eq!(hash, "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262");
/// ```
#[inline]
pub fn blake3_str(s: &str) -> String {
    blake3_hex(s.as_bytes())
}

/// Compute the BLAKE3 hash of raw bytes, hex-encoded.
///
/// Returns a 64-character lowercase hex string.
#[inline]
pub fn blake3_hex(data: &[u8]) -> String {
    blake3::hash(data).to_hex().to_string()
}

/// Compute the BLAKE3 hash of raw bytes.
///
/// Returns the 32-byte raw digest.
#[inline]
pub fn blake3_bytes(data: &[u8]) -> Vec<u8> {
    blake3::hash(data).as_bytes().to_vec()
}
// endregion

// region: Vector helpers

/// Compute BLAKE3 hex digests for a vector of strings.
pub fn blake3_str_vec(strings: &[&str]) -> Vec<String> {
    strings.iter().map(|s| blake3_str(s)).collect()
}
// endregion

// region: Unit tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blake3_empty() {
        // Official BLAKE3 test vector for empty input
        assert_eq!(
            blake3_str(""),
            "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262"
        );
    }

    #[test]
    fn test_blake3_bytes_len_and_hex_agreement() {
        let raw = blake3_bytes(b"abc");
        assert_eq!(raw.len(), 32);
        let hex: String = raw.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(hex, blake3_str("abc"));
    }

    #[test]
    fn test_blake3_deterministic_and_distinct() {
        assert_eq!(blake3_str("hello"), blake3_str("hello"));
        assert_ne!(blake3_str("hello"), blake3_str("world"));
    }

    #[test]
    fn test_blake3_str_vec() {
        let hashes = blake3_str_vec(&["a", "b"]);
        assert_eq!(hashes.len(), 2);
        assert_eq!(hashes[0].len(), 64);
        assert_ne!(hashes[0], hashes[1]);
    }
}
// endregion
