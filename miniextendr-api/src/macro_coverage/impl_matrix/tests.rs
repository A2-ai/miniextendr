use super::*;

/// Verify that the generated wrapper's data_offset is correct for
/// the default-aligned CovEnv type.
#[test]
fn test_data_offset_default_aligned() {
    let wrapped = __mx_wrap_CovEnv(CovEnv { v: 42 });
    let base = unsafe { (*wrapped).base };
    let data_offset = unsafe { (*base).data_offset };

    // For default-aligned types, offset should equal size_of::<mx_erased>()
    assert_eq!(data_offset, std::mem::size_of::<crate::abi::mx_erased>());

    // Verify we can read the data correctly through the offset
    let data_ptr = unsafe {
        wrapped
            .cast::<u8>()
            .add(data_offset)
            .cast::<std::cell::RefCell<CovEnv>>()
            .cast_const()
    };
    let value = unsafe { (*data_ptr).borrow().v };
    assert_eq!(value, 42);

    // Clean up
    unsafe {
        let drop_fn = (*base).drop;
        drop_fn(wrapped);
    }
}
