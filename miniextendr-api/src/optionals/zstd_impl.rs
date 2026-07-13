//! Integration with the `zstd` crate for whole-buffer compression.
//!
//! R has `memCompress(type = c("gzip", "bzip2", "xz"))` but no zstd. This
//! module exposes whole-buffer zstd compress/decompress (R `raw` in / out).
//!
//! # Features
//!
//! Enable this module with the `zstd` feature:
//!
//! ```toml
//! [dependencies]
//! miniextendr-api = { version = "0.1", features = ["zstd"] }
//! ```
//!
//! # Build note
//!
//! `zstd` compiles the bundled zstd C library via `zstd-sys` (the `cc` crate
//! with pregenerated bindings). No cmake or pkg-config is required; the C
//! toolchain R already mandates is sufficient.
//!
//! # Compression levels
//!
//! Valid levels are returned by `compression_level_range()`;
//! `0` selects zstd's `DEFAULT_COMPRESSION_LEVEL`.
//! Out-of-range levels return an error.
//!
//! # Caution
//!
//! [`zstd_decompress`] materializes the whole decompressed buffer in memory —
//! do not feed it untrusted data of unbounded decompressed size (decompression
//! bombs). A size-capped / streaming variant is a follow-up.

pub use zstd::DEFAULT_COMPRESSION_LEVEL;
pub use zstd::compression_level_range;

// region: Core compression functions

/// Compress a byte buffer with zstd at the given level.
///
/// `level == 0` selects zstd's default level (3). Other values must fall in
/// `compression_level_range()`.
///
/// # Errors
///
/// Returns an error if the level is out of range or compression fails.
pub fn zstd_compress(data: &[u8], level: i32) -> Result<Vec<u8>, String> {
    let range = compression_level_range();
    if level != 0 && !range.contains(&level) {
        return Err(format!(
            "zstd compression level {level} out of range {}..={} (0 = default)",
            range.start(),
            range.end()
        ));
    }
    zstd::stream::encode_all(data, level).map_err(|e| format!("zstd compression failed: {e}"))
}

/// Decompress a zstd-compressed byte buffer.
///
/// # Errors
///
/// Returns an error if the input is not valid zstd data.
pub fn zstd_decompress(data: &[u8]) -> Result<Vec<u8>, String> {
    zstd::stream::decode_all(data).map_err(|e| format!("zstd decompression failed: {e}"))
}
// endregion

// region: Unit tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let data: Vec<u8> = (0u8..=255).cycle().take(10_000).collect();
        let compressed = zstd_compress(&data, 3).unwrap();
        assert!(compressed.len() < data.len());
        assert_eq!(zstd_decompress(&compressed).unwrap(), data);
    }

    #[test]
    fn roundtrip_empty() {
        let compressed = zstd_compress(&[], 1).unwrap();
        assert_eq!(zstd_decompress(&compressed).unwrap(), Vec::<u8>::new());
    }

    #[test]
    fn level_zero_is_default() {
        let data = b"hello hello hello hello";
        assert_eq!(
            zstd_decompress(&zstd_compress(data, 0).unwrap()).unwrap(),
            data.to_vec()
        );
    }

    #[test]
    fn level_bounds() {
        let range = compression_level_range();
        // Range is -131072..=22 in zstd 0.13 (negative = fast levels)
        assert!(zstd_compress(b"x", *range.start()).is_ok());
        assert!(zstd_compress(b"x", *range.end()).is_ok());
        // Beyond max level
        assert!(zstd_compress(b"x", range.end() + 1).is_err());
        // Below min level
        assert!(zstd_compress(b"x", range.start() - 1).is_err());
    }

    #[test]
    fn decompress_garbage_errors() {
        assert!(zstd_decompress(&[0xDE, 0xAD, 0xBE, 0xEF]).is_err());
    }
}
// endregion
