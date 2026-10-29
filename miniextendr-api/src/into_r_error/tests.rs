use super::*;

#[test]
fn display_string_too_long() {
    let e = IntoRError::StringTooLong { len: 3_000_000_000 };
    assert!(e.to_string().contains("3000000000"));
    assert!(e.to_string().contains("i32 limit"));
}

#[test]
fn display_length_overflow() {
    let e = IntoRError::LengthOverflow { len: 42 };
    assert!(e.to_string().contains("42"));
}

#[test]
fn display_inner() {
    let e = IntoRError::Inner("nested failure".to_string());
    assert_eq!(e.to_string(), "nested failure");
}
