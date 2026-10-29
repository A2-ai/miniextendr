use super::*;

#[test]
fn test_error_display() {
    let err = RCoerceError::NotSupported {
        from: "MyType",
        to: "data.frame",
    };
    assert_eq!(err.to_string(), "cannot coerce MyType to data.frame");

    let err = RCoerceError::InvalidData {
        message: "column lengths differ".to_string(),
    };
    assert_eq!(err.to_string(), "invalid data: column lengths differ");

    let err = RCoerceError::Custom("something went wrong".to_string());
    assert_eq!(err.to_string(), "something went wrong");
}

#[test]
fn test_supported_generics() {
    assert!(is_supported_as_generic("data.frame"));
    assert!(is_supported_as_generic("list"));
    assert!(is_supported_as_generic("character"));
    assert!(is_supported_as_generic("numeric"));
    assert!(is_supported_as_generic("double"));
    assert!(!is_supported_as_generic("foo"));
    assert!(!is_supported_as_generic(""));
}

#[test]
fn test_generic_to_method() {
    assert_eq!(r_generic_to_method("data.frame"), Some("as_data_frame"));
    assert_eq!(r_generic_to_method("list"), Some("as_list"));
    assert_eq!(r_generic_to_method("numeric"), Some("as_numeric"));
    assert_eq!(r_generic_to_method("double"), Some("as_numeric"));
    assert_eq!(r_generic_to_method("foo"), None);
}
