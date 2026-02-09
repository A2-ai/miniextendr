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

    // mx_base_vtable: 4 fields (2 pointers + mx_tag + usize)
    // On 64-bit: drop(8) + concrete_tag(16) + query(8) + data_offset(8) = 40 bytes
    // Note: actual layout may differ due to padding
    assert!(size_of::<mx_base_vtable>() >= 40);
}

#[test]
fn test_mx_tag_equality() {
    let tag1 = mx_tag::new(1, 2);
    let tag2 = mx_tag::new(1, 2);
    let tag3 = mx_tag::new(1, 3);

    assert_eq!(tag1, tag2);
    assert_ne!(tag1, tag3);
}

/// Verify that data offset computation is alignment-correct for `#[repr(C)]`
/// wrapper structs of the form `{ erased: mx_erased, data: T }`.
///
/// When `T` has stricter alignment than `mx_erased`, padding is inserted.
/// The correct offset is `size_of::<mx_erased>().next_multiple_of(align_of::<T>())`.
#[test]
fn test_wrapper_data_offset_alignment() {
    // Helper: compute what the data offset would be in a
    // #[repr(C)] struct { erased: mx_erased, data: T }
    fn expected_data_offset<T>() -> usize {
        size_of::<mx_erased>().next_multiple_of(align_of::<T>())
    }

    // For types with alignment <= pointer alignment, offset == size_of::<mx_erased>()
    assert_eq!(expected_data_offset::<u8>(), size_of::<mx_erased>());
    assert_eq!(expected_data_offset::<u32>(), size_of::<mx_erased>());
    assert_eq!(expected_data_offset::<u64>(), size_of::<mx_erased>());
    assert_eq!(expected_data_offset::<*const ()>(), size_of::<mx_erased>());

    // For types with alignment > pointer alignment, padding is needed.
    // mx_erased is pointer-sized (8 bytes on 64-bit), so align(16) needs
    // the data at offset 16 (8 bytes of padding after the 8-byte header).
    #[repr(align(16))]
    struct Align16 {
        _data: [u8; 16],
    }
    assert_eq!(expected_data_offset::<Align16>(), 16);

    #[repr(align(32))]
    struct Align32 {
        _data: [u8; 32],
    }
    assert_eq!(expected_data_offset::<Align32>(), 32);

    #[repr(align(64))]
    struct Align64 {
        _data: [u8; 64],
    }
    assert_eq!(expected_data_offset::<Align64>(), 64);

    // Verify against actual repr(C) struct layout using offset_of!
    #[repr(C)]
    struct Wrapper16 {
        erased: mx_erased,
        data: Align16,
    }
    assert_eq!(
        std::mem::offset_of!(Wrapper16, data),
        expected_data_offset::<Align16>()
    );

    #[repr(C)]
    struct Wrapper32 {
        erased: mx_erased,
        data: Align32,
    }
    assert_eq!(
        std::mem::offset_of!(Wrapper32, data),
        expected_data_offset::<Align32>()
    );

    #[repr(C)]
    struct Wrapper64 {
        erased: mx_erased,
        data: Align64,
    }
    assert_eq!(
        std::mem::offset_of!(Wrapper64, data),
        expected_data_offset::<Align64>()
    );
}

/// Verify that the old (buggy) offset computation would fail for over-aligned types.
/// This test documents the bug that was fixed: using `size_of::<mx_erased>()` as the
/// data offset is wrong when `T` has alignment > `align_of::<mx_erased>()`.
#[test]
fn test_old_offset_wrong_for_overaligned() {
    let erased_size = size_of::<mx_erased>();

    #[repr(align(32))]
    struct Align32 {
        _data: [u8; 32],
    }

    #[repr(C)]
    struct Wrapper32 {
        _erased: mx_erased,
        data: Align32,
    }

    let actual_offset = std::mem::offset_of!(Wrapper32, data);

    // The old code used erased_size directly -- this is WRONG for align(32)
    assert_ne!(
        erased_size, actual_offset,
        "size_of::<mx_erased>() should NOT equal the actual data offset for align(32) types"
    );

    // The correct offset accounts for alignment
    assert_eq!(actual_offset, erased_size.next_multiple_of(align_of::<Align32>()));
}
