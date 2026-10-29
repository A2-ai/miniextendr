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
