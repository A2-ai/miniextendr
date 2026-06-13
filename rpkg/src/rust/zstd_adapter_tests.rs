//! zstd adapter tests
use miniextendr_api::miniextendr;

/// Compress a raw vector with zstd.
/// @param data Raw vector to compress.
/// @param level Compression level (1-22); 0 or NA selects the default (3).
#[miniextendr]
pub fn zstd_compress(data: Vec<u8>, level: Option<i32>) -> Result<Vec<u8>, String> {
    miniextendr_api::zstd_impl::zstd_compress(&data, level.unwrap_or(0))
}

/// Decompress a zstd-compressed raw vector. Errors on invalid input.
/// @param data Raw vector of zstd-compressed bytes.
#[miniextendr]
pub fn zstd_decompress(data: Vec<u8>) -> Result<Vec<u8>, String> {
    miniextendr_api::zstd_impl::zstd_decompress(&data)
}

/// Round-trip helper: compress then decompress, returns input on success.
/// @param data Raw vector to round-trip.
/// @param level Compression level.
#[miniextendr]
pub fn zstd_roundtrip(data: Vec<u8>, level: i32) -> Result<Vec<u8>, String> {
    let compressed = miniextendr_api::zstd_impl::zstd_compress(&data, level)?;
    miniextendr_api::zstd_impl::zstd_decompress(&compressed)
}
