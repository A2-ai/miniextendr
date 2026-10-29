use super::*;

#[test]
fn test_borsh_serialize_roundtrip_vec() {
    let original: Vec<u8> = vec![1, 2, 3, 4, 5];
    let bytes = original.borsh_serialize();
    let recovered: Vec<u8> = RBorshOps::borsh_deserialize(&bytes).unwrap();
    assert_eq!(original, recovered);
}

#[test]
fn test_borsh_serialize_roundtrip_tuple() {
    let original: (f64, f64, f64) = (1.5, 2.5, 3.5);
    let bytes = original.borsh_serialize();
    let recovered: (f64, f64, f64) = RBorshOps::borsh_deserialize(&bytes).unwrap();
    assert_eq!(original, recovered);
}

#[test]
fn test_borsh_serialize_roundtrip_string() {
    let original = String::from("hello borsh");
    let bytes = original.borsh_serialize();
    let recovered: String = RBorshOps::borsh_deserialize(&bytes).unwrap();
    assert_eq!(original, recovered);
}

#[test]
fn test_borsh_serialize_empty_vec() {
    let original: Vec<i32> = vec![];
    let bytes = original.borsh_serialize();
    let recovered: Vec<i32> = RBorshOps::borsh_deserialize(&bytes).unwrap();
    assert_eq!(original, recovered);
}

#[test]
fn test_borsh_size() {
    let value: u32 = 42;
    let size = value.borsh_size();
    assert_eq!(size, 4);
}

#[test]
fn test_borsh_size_string() {
    let value = String::from("hi");
    let size = value.borsh_size();
    // borsh encodes strings as: 4-byte length prefix + utf8 bytes
    assert_eq!(size, 4 + 2);
}

#[test]
fn test_borsh_deserialize_invalid_data() {
    let bad_bytes: &[u8] = &[0xff, 0xff];
    let result: Result<String, _> = RBorshOps::borsh_deserialize(bad_bytes);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("borsh deserialization failed"));
}

#[test]
fn test_borsh_roundtrip_nested() {
    let original: Vec<(String, Vec<u8>)> = vec![
        ("first".to_string(), vec![1, 2, 3]),
        ("second".to_string(), vec![4, 5]),
    ];
    let bytes = original.borsh_serialize();
    let recovered: Vec<(String, Vec<u8>)> = RBorshOps::borsh_deserialize(&bytes).unwrap();
    assert_eq!(original, recovered);
}

#[test]
fn test_borsh_roundtrip_option() {
    let some_val: Option<u64> = Some(42);
    let none_val: Option<u64> = None;

    let some_bytes = some_val.borsh_serialize();
    let none_bytes = none_val.borsh_serialize();

    let recovered_some: Option<u64> = RBorshOps::borsh_deserialize(&some_bytes).unwrap();
    let recovered_none: Option<u64> = RBorshOps::borsh_deserialize(&none_bytes).unwrap();

    assert_eq!(some_val, recovered_some);
    assert_eq!(none_val, recovered_none);
}
