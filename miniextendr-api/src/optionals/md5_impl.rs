//! Integration with the `md5` crate for legacy hashing.
//!
//! Provides MD5 hashing helpers for R.
//!
//! # Security
//!
//! **MD5 is cryptographically broken** (practical collision attacks exist).
//! It is exposed strictly for interoperability: ETags, cache keys,
//! content-addressed storage, and compatibility with existing R tooling
//! (`digest::digest(algo = "md5")`). Do NOT use it for passwords, signatures,
//! or any security boundary — use `sha2` or `blake3` instead.
//!
//! # Features
//!
//! Enable this module with the `md5` feature:
//!
//! ```toml
//! [dependencies]
//! miniextendr-api = { version = "0.1", features = ["md5"] }
//! ```
//!
//! # Functions
//!
//! | Function | Input | Output |
//! |----------|-------|--------|
//! | `md5_str` | `&str` | 32-char lowercase hex string |
//! | `md5_bytes` | `&[u8]` | 16-byte raw digest (`Vec<u8>`) |
//! | `md5_hex` | `&[u8]` | 32-char lowercase hex string |

// Re-export core types for advanced usage
pub use md5::Digest;

// region: Core hashing functions

/// Compute the MD5 hash of a UTF-8 string.
///
/// Returns a 32-character lowercase hex string.
///
/// # Example
///
/// ```ignore
/// assert_eq!(md5_str(""), "d41d8cd98f00b204e9800998ecf8427e");
/// assert_eq!(md5_str("abc"), "900150983cd24fb0d6963f7d28e17f72");
/// ```
#[inline]
pub fn md5_str(s: &str) -> String {
    md5_hex(s.as_bytes())
}

/// Compute the MD5 hash of raw bytes, hex-encoded.
///
/// Returns a 32-character lowercase hex string.
#[inline]
pub fn md5_hex(data: &[u8]) -> String {
    format!("{:x}", md5::compute(data))
}

/// Compute the MD5 hash of raw bytes.
///
/// Returns the 16-byte raw digest.
#[inline]
pub fn md5_bytes(data: &[u8]) -> Vec<u8> {
    md5::compute(data).0.to_vec()
}
// endregion

// region: Vector helpers

/// Compute MD5 hex digests for a vector of strings.
pub fn md5_str_vec(strings: &[&str]) -> Vec<String> {
    strings.iter().map(|s| md5_str(s)).collect()
}
// endregion

// region: Unit tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_md5_empty() {
        // RFC 1321 test vector
        assert_eq!(md5_str(""), "d41d8cd98f00b204e9800998ecf8427e");
    }

    #[test]
    fn test_md5_abc() {
        // RFC 1321 test vector
        assert_eq!(md5_str("abc"), "900150983cd24fb0d6963f7d28e17f72");
    }

    #[test]
    fn test_md5_bytes_len_and_hex_agreement() {
        let raw = md5_bytes(b"abc");
        assert_eq!(raw.len(), 16);
        let hex: String = raw.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(hex, md5_str("abc"));
    }

    #[test]
    fn test_md5_str_vec() {
        let hashes = md5_str_vec(&["a", "b"]);
        assert_eq!(hashes.len(), 2);
        assert_eq!(hashes[0].len(), 32);
        assert_ne!(hashes[0], hashes[1]);
    }
}
// endregion
