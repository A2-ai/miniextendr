use super::*;
use bitflags::bitflags;

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct TestFlags: u8 {
        const A = 0b0001;
        const B = 0b0010;
        const C = 0b0100;
        const AB = Self::A.bits() | Self::B.bits();
    }
}

#[test]
fn test_rflags_new() {
    let flags = RFlags::new(TestFlags::A | TestFlags::B);
    assert!(flags.contains(TestFlags::A));
    assert!(flags.contains(TestFlags::B));
    assert!(!flags.contains(TestFlags::C));
}

#[test]
fn test_rflags_deref() {
    let flags = RFlags::new(TestFlags::A);
    // Deref allows calling methods on inner type
    assert!(flags.contains(TestFlags::A));
}

#[test]
fn test_rflags_into_inner() {
    let flags = RFlags::new(TestFlags::AB);
    let inner = flags.into_inner();
    assert_eq!(inner, TestFlags::A | TestFlags::B);
}

#[test]
fn test_rflags_bitwise_ops() {
    let a = RFlags::new(TestFlags::A);
    let b = RFlags::new(TestFlags::B);

    let or = a | b;
    assert!(or.contains(TestFlags::A));
    assert!(or.contains(TestFlags::B));

    let and = RFlags::new(TestFlags::AB) & a;
    assert!(and.contains(TestFlags::A));
    assert!(!and.contains(TestFlags::B));
}

#[test]
fn test_flags_from_i32_strict() {
    let flags: Option<TestFlags> = flags_from_i32_strict(0b0011);
    assert_eq!(flags, Some(TestFlags::A | TestFlags::B));

    // Unknown bit should fail
    let invalid: Option<TestFlags> = flags_from_i32_strict(0b1000);
    assert_eq!(invalid, None);
}

#[test]
fn test_flags_from_i32_truncate() {
    let flags: TestFlags = flags_from_i32_truncate(0b1111);
    // Only A, B, C are defined (0b0111)
    assert_eq!(flags, TestFlags::A | TestFlags::B | TestFlags::C);
}

#[test]
fn test_flags_to_i32() {
    let flags = TestFlags::A | TestFlags::B;
    let int = flags_to_i32(flags);
    assert_eq!(int, Some(0b0011));
}
