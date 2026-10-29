use super::*;

/// Verify that the generated wrapper's data_offset is correct for
/// a 32-byte-aligned type. This catches the bug where `size_of::<mx_erased>()`
/// was used directly (which is 8 on 64-bit, but the data field starts at 32).
#[test]
fn test_data_offset_align32() {
    let wrapped = __mx_wrap_Aligned32Counter(Aligned32Counter { value: 99 });
    let base = unsafe { (*wrapped).base };
    let data_offset = unsafe { (*base).data_offset };

    // For align(32) types on 64-bit, offset should be 32 (not 8)
    assert_eq!(data_offset, 32);
    assert_ne!(
        data_offset,
        std::mem::size_of::<crate::abi::mx_erased>(),
        "data_offset must NOT equal size_of::<mx_erased>() for over-aligned types"
    );

    // Verify we can read the data correctly
    let data_ptr = unsafe {
        wrapped
            .cast::<u8>()
            .add(data_offset)
            .cast::<std::cell::RefCell<Aligned32Counter>>()
            .cast_const()
    };
    let value = unsafe { (*data_ptr).borrow().value };
    assert_eq!(value, 99);

    // Verify vtable query succeeds (non-null = trait is implemented)
    let vtable = unsafe { ((*base).query)(wrapped, TAG_CovTrait) };
    assert!(!vtable.is_null());

    unsafe {
        let drop_fn = (*base).drop;
        drop_fn(wrapped);
    }
}

/// Verify that the generated wrapper's data_offset is correct for
/// a 64-byte-aligned type.
#[test]
fn test_data_offset_align64() {
    let wrapped = __mx_wrap_Aligned64Counter(Aligned64Counter { value: 77 });
    let base = unsafe { (*wrapped).base };
    let data_offset = unsafe { (*base).data_offset };

    // For align(64) types, offset should be 64
    assert_eq!(data_offset, 64);

    // Verify we can read the data correctly
    let data_ptr = unsafe {
        wrapped
            .cast::<u8>()
            .add(data_offset)
            .cast::<std::cell::RefCell<Aligned64Counter>>()
            .cast_const()
    };
    let value = unsafe { (*data_ptr).borrow().value };
    assert_eq!(value, 77);

    unsafe {
        let drop_fn = (*base).drop;
        drop_fn(wrapped);
    }
}
