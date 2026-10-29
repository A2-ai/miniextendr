use super::*;

#[derive(Copy, Clone, Pod, Zeroable, Debug, PartialEq)]
#[repr(C)]
struct TestVec3 {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Copy, Clone, Pod, Zeroable, Debug, PartialEq)]
#[repr(C)]
struct TestPoint {
    x: i32,
    y: i32,
}

#[test]
fn test_raw_bytes_roundtrip() {
    let value = TestVec3 {
        x: 1.0,
        y: 2.0,
        z: 3.0,
    };
    let bytes = raw_to_bytes(&value);
    let decoded: TestVec3 = raw_from_bytes(&bytes).unwrap();
    assert_eq!(value, decoded);
}

#[test]
fn test_raw_slice_bytes_roundtrip() {
    let values = vec![
        TestPoint { x: 1, y: 2 },
        TestPoint { x: 3, y: 4 },
        TestPoint { x: 5, y: 6 },
    ];
    let bytes = raw_slice_to_bytes(&values);
    let decoded: Vec<TestPoint> = raw_slice_from_bytes(&bytes).unwrap();
    assert_eq!(values, decoded);
}

#[test]
fn test_raw_length_mismatch() {
    let bytes = vec![1u8, 2, 3]; // Not enough for TestVec3 (12 bytes)
    let result = raw_from_bytes::<TestVec3>(&bytes);
    assert!(matches!(result, Err(RawError::LengthMismatch { .. })));
}

#[test]
fn test_raw_slice_length_mismatch() {
    let bytes = vec![1u8, 2, 3, 4, 5]; // Not a multiple of 8 (TestPoint size)
    let result = raw_slice_from_bytes::<TestPoint>(&bytes);
    assert!(matches!(result, Err(RawError::LengthMismatch { .. })));
}

#[test]
fn test_header_validation() {
    let header = RawHeader::new_single::<TestVec3>();
    assert!(header.validate::<TestVec3>(Some(1)).is_ok());
    assert!(header.validate::<TestVec3>(Some(2)).is_err()); // Wrong count
    assert!(header.validate::<TestPoint>(Some(1)).is_err()); // Wrong size
}

#[test]
fn test_header_invalid_magic() {
    let mut header = RawHeader::new_single::<TestVec3>();
    header.magic = *b"XXXX";
    assert!(matches!(
        header.validate::<TestVec3>(Some(1)),
        Err(RawError::InvalidHeader(_))
    ));
}

#[test]
fn test_header_invalid_version() {
    let mut header = RawHeader::new_single::<TestVec3>();
    header.version = 99;
    assert!(matches!(
        header.validate::<TestVec3>(Some(1)),
        Err(RawError::InvalidHeader(_))
    ));
}

#[test]
fn test_raw_wrapper() {
    let raw = Raw(TestVec3 {
        x: 1.0,
        y: 2.0,
        z: 3.0,
    });
    assert_eq!(raw.inner().x, 1.0);
    let inner = raw.into_inner();
    assert_eq!(inner.y, 2.0);
}

#[test]
fn test_raw_slice_wrapper() {
    let raw = RawSlice(vec![TestPoint { x: 1, y: 2 }, TestPoint { x: 3, y: 4 }]);
    assert_eq!(raw.len(), 2);
    assert!(!raw.is_empty());
    assert_eq!(raw.inner()[0].x, 1);
    let inner = raw.into_inner();
    assert_eq!(inner[1].y, 4);
}

#[test]
fn test_raw_tagged_wrapper() {
    let raw = RawTagged(TestVec3 {
        x: 1.0,
        y: 2.0,
        z: 3.0,
    });
    assert_eq!(raw.inner().z, 3.0);
    let inner = raw.into_inner();
    assert_eq!(inner.x, 1.0);
}

#[test]
fn test_raw_slice_tagged_wrapper() {
    let raw = RawSliceTagged(vec![TestPoint { x: 1, y: 2 }]);
    assert_eq!(raw.len(), 1);
    assert!(!raw.is_empty());
    let inner = raw.into_inner();
    assert_eq!(inner[0], TestPoint { x: 1, y: 2 });
}

#[test]
fn test_empty_slice() {
    let raw: RawSlice<TestPoint> = RawSlice(vec![]);
    assert!(raw.is_empty());
    assert_eq!(raw.len(), 0);
}
