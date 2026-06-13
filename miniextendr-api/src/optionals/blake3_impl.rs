//! Integration with the `blake3` crate for cryptographic hashing.
//!
//! Provides BLAKE3 hashing helpers for R.
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
//! use miniextendr_api::blake3_impl::{blake3_str, blake3_bytes};
//!
//! #[miniextendr]
//! fn fingerprint(payload: &str) -> String {
//!     blake3_str(payload)
//! }
//! ```
//!
//! # Functions
//!
//! | Function | Input | Output |
//! |----------|-------|--------|
//! | `blake3_bytes` | `&[u8]` | 32-byte digest (`Vec<u8>`) |
//! | `blake3_bytes_hex` | `&[u8]` | 64-char hex string |
//! | `blake3_str` | `&str` | 64-char hex string |

// Re-export core types for advanced usage
pub use blake3::{Hash, Hasher};

// region: Core hashing functions

/// Compute the BLAKE3 hash of raw bytes.
///
/// Returns the 32-byte digest.
///
/// # Example
///
/// ```ignore
/// let digest = blake3_bytes(b"hello world");
/// assert_eq!(digest.len(), 32);
/// ```
#[inline]
pub fn blake3_bytes(data: &[u8]) -> Vec<u8> {
    blake3::hash(data).as_bytes().to_vec()
}

/// Compute the BLAKE3 hash of raw bytes as a lowercase hex string.
///
/// Returns a 64-character lowercase hex string.
///
/// # Example
///
/// ```ignore
/// let hash = blake3_bytes_hex(b"hello world");
/// assert_eq!(hash.len(), 64);
/// ```
#[inline]
pub fn blake3_bytes_hex(data: &[u8]) -> String {
    blake3::hash(data).to_hex().to_string()
}

/// Compute the BLAKE3 hash of a UTF-8 string.
///
/// Returns a 64-character lowercase hex string.
///
/// # Example
///
/// ```ignore
/// let hash = blake3_str("hello world");
/// assert_eq!(hash, "d74981efa70a0c880b8d8c1985d075dbcbf679b99a5f9914e5aaf96b831a9e24");
/// ```
#[inline]
pub fn blake3_str(s: &str) -> String {
    blake3_bytes_hex(s.as_bytes())
}
// endregion

// region: Vector helpers

/// Compute BLAKE3 hex digests for a vector of byte slices.
///
/// Returns a vector of 64-character lowercase hex strings.
pub fn blake3_bytes_hex_vec(data: &[&[u8]]) -> Vec<String> {
    data.iter().map(|d| blake3_bytes_hex(d)).collect()
}

/// Compute BLAKE3 hex digests for a vector of strings.
///
/// Returns a vector of 64-character lowercase hex strings.
pub fn blake3_str_vec(strings: &[&str]) -> Vec<String> {
    strings.iter().map(|s| blake3_str(s)).collect()
}
// endregion

// region: Unit tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blake3_str() {
        // Known test vector
        let hash = blake3_str("hello world");
        assert_eq!(
            hash,
            "d74981efa70a0c880b8d8c1985d075dbcbf679b99a5f9914e5aaf96b831a9e24"
        );
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_blake3_bytes_hex() {
        let hash = blake3_bytes_hex(b"hello world");
        assert_eq!(
            hash,
            "d74981efa70a0c880b8d8c1985d075dbcbf679b99a5f9914e5aaf96b831a9e24"
        );
    }

    #[test]
    fn test_blake3_bytes() {
        let digest = blake3_bytes(b"hello world");
        assert_eq!(digest.len(), 32);
        // Raw digest must hex-encode to the same well-known value.
        assert_eq!(
            blake3_bytes_hex(b"hello world"),
            "d74981efa70a0c880b8d8c1985d075dbcbf679b99a5f9914e5aaf96b831a9e24"
        );
    }

    #[test]
    fn test_blake3_empty() {
        // Well-known BLAKE3 digest of the empty input.
        let hash = blake3_str("");
        assert_eq!(
            hash,
            "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262"
        );
    }

    #[test]
    fn test_blake3_str_vec() {
        let hashes = blake3_str_vec(&["a", "b", "c"]);
        assert_eq!(hashes.len(), 3);
        // blake3("a")
        assert_eq!(
            hashes[0],
            "17762fddd969a453925d65717ac3eea21320b66b54342fde15128d6caf21215f"
        );
    }

    #[test]
    fn test_blake3_bytes_hex_vec() {
        let hashes = blake3_bytes_hex_vec(&[b"a", b"b"]);
        assert_eq!(hashes.len(), 2);
        assert_eq!(hashes[0].len(), 64);
        assert_eq!(hashes[1].len(), 64);
    }
}
// endregion
