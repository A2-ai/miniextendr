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

// =============================================================================
// Core hashing functions
// =============================================================================

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

// =============================================================================
// Vector helpers
// =============================================================================

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

// =============================================================================
// Helper functions
// =============================================================================

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

// =============================================================================
// Unit tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_str() {
        // Known test vector
        let hash = sha256_str("hello world");
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_sha256_bytes() {
        let hash = sha256_bytes(b"hello world");
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_sha256_empty() {
        let hash = sha256_str("");
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_sha512_str() {
        let hash = sha512_str("hello world");
        assert_eq!(hash.len(), 128);
        // Known test vector
        assert_eq!(
            hash,
            "309ecc489c12d6eb4cc40f50c902f2b4d0ed77ee511a7c7a9bcd3ca86d4cd86f989dd35bc5ff499670da34255b45b0cfd830e81f605dcf7dc5542e93ae9cd76f"
        );
    }

    #[test]
    fn test_sha512_bytes() {
        let hash = sha512_bytes(b"hello world");
        assert_eq!(hash.len(), 128);
    }

    #[test]
    fn test_sha512_empty() {
        let hash = sha512_str("");
        assert_eq!(
            hash,
            "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e"
        );
    }

    #[test]
    fn test_sha256_str_vec() {
        let hashes = sha256_str_vec(&["a", "b", "c"]);
        assert_eq!(hashes.len(), 3);
        assert_eq!(
            hashes[0],
            "ca978112ca1bbdcafac231b39a23dc4da786eff8147c4e72b9807785afee48bb"
        ); // sha256("a")
    }

    #[test]
    fn test_sha512_str_vec() {
        let hashes = sha512_str_vec(&["a", "b"]);
        assert_eq!(hashes.len(), 2);
        assert_eq!(hashes[0].len(), 128);
        assert_eq!(hashes[1].len(), 128);
    }

    #[test]
    fn test_hex_encode() {
        assert_eq!(hex_encode(&[0x00, 0xff, 0x10]), "00ff10");
        assert_eq!(hex_encode(&[]), "");
    }
}
