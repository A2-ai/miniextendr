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
