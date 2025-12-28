use super::*;
use std::mem::{align_of, size_of};

/// Ensure ABI types have expected sizes and alignments.
///
/// These tests catch accidental ABI breakage.
#[test]
fn test_abi_layout() {
    // mx_tag: 16 bytes (2 × u64), 8-byte aligned
    assert_eq!(size_of::<mx_tag>(), 16);
    assert_eq!(align_of::<mx_tag>(), 8);

    // mx_erased: pointer-sized, pointer-aligned
    assert_eq!(size_of::<mx_erased>(), size_of::<*const ()>());
    assert_eq!(align_of::<mx_erased>(), align_of::<*const ()>());

    // mx_base_vtable: 3 fields (2 pointers + mx_tag)
    // On 64-bit: drop(8) + concrete_tag(16) + query(8) = 32 bytes
    // Note: actual layout may differ due to padding
    assert!(size_of::<mx_base_vtable>() >= 32);
}

#[test]
fn test_mx_tag_equality() {
    let tag1 = mx_tag::new(1, 2);
    let tag2 = mx_tag::new(1, 2);
    let tag3 = mx_tag::new(1, 3);

    assert_eq!(tag1, tag2);
    assert_ne!(tag1, tag3);
}
