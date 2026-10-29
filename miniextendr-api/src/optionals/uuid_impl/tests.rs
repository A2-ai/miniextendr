use super::*;

#[test]
fn uuid_can_be_created() {
    let uuid = Uuid::new_v4();
    assert_eq!(uuid.to_string().len(), 36); // UUID format: 8-4-4-4-12
}

#[test]
fn uuid_can_be_parsed() {
    let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
    assert_eq!(uuid.to_string(), "550e8400-e29b-41d4-a716-446655440000");
}

#[test]
fn ruuidops_version() {
    let v4 = Uuid::new_v4();
    assert_eq!(RUuidOps::version(&v4), 4);

    let nil = Uuid::nil();
    // Nil UUID has no version
    assert_eq!(RUuidOps::version(&nil), 0);
}

#[test]
fn ruuidops_nil_max() {
    let nil = Uuid::nil();
    assert!(RUuidOps::is_nil(&nil));
    assert!(!RUuidOps::is_max(&nil));

    let max = Uuid::max();
    assert!(!RUuidOps::is_nil(&max));
    assert!(RUuidOps::is_max(&max));

    let v4 = Uuid::new_v4();
    assert!(!RUuidOps::is_nil(&v4));
    assert!(!RUuidOps::is_max(&v4));
}

#[test]
fn ruuidops_bytes() {
    let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
    let bytes = RUuidOps::as_bytes(&uuid);
    assert_eq!(bytes.len(), 16);
    assert_eq!(bytes[0], 0x55);
}

#[test]
fn ruuidops_formats() {
    let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

    assert_eq!(
        RUuidOps::to_hyphenated(&uuid),
        "550e8400-e29b-41d4-a716-446655440000"
    );
    assert_eq!(
        RUuidOps::to_simple(&uuid),
        "550e8400e29b41d4a716446655440000"
    );
    assert_eq!(
        RUuidOps::to_urn(&uuid),
        "urn:uuid:550e8400-e29b-41d4-a716-446655440000"
    );
}

#[test]
fn uuid_helpers_work() {
    let v4 = uuid_helpers::new_v4();
    assert!(!v4.is_nil());

    let nil = uuid_helpers::nil();
    assert!(nil.is_nil());

    let max = uuid_helpers::max();
    assert!(max.is_max());

    let from_bytes = uuid_helpers::from_bytes(vec![0u8; 16]).unwrap();
    assert!(from_bytes.is_nil());

    let invalid_bytes = uuid_helpers::from_bytes(vec![0u8; 10]);
    assert!(invalid_bytes.is_err());

    let parsed = uuid_helpers::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
    assert_eq!(parsed.to_string(), "550e8400-e29b-41d4-a716-446655440000");
}
