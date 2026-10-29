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
