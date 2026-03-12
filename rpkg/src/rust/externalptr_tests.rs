//! Tests for ExternalPtr functionality.

use miniextendr_api::externalptr::ErasedExternalPtr;
use miniextendr_api::ffi::SEXP;
use miniextendr_api::miniextendr;

/// A simple test struct for ExternalPtr
#[derive(miniextendr_api::ExternalPtr, Debug)]
pub struct Counter {
    pub value: i32,
}

/// Another test struct to verify type safety
#[derive(miniextendr_api::ExternalPtr, Debug)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// @noRd
#[miniextendr]
/// @name rpkg_externalptr
/// @examples
/// ptr <- extptr_counter_new(1L)
/// unsafe_C_extptr_counter_get(ptr)
/// unsafe_C_extptr_counter_increment(ptr)
/// p <- extptr_point_new(0.1, 0.2)
/// unsafe_C_extptr_point_get_x(p)
/// test_extptr_on_main_thread()
/// @aliases extptr_counter_new extptr_point_new unsafe_C_extptr_counter_get
///   unsafe_C_extptr_counter_increment unsafe_C_extptr_point_get_x unsafe_C_extptr_point_get_y
///   unsafe_C_extptr_type_mismatch_test unsafe_C_extptr_null_test unsafe_C_extptr_is_counter
///   unsafe_C_extptr_is_point test_extptr_on_main_thread
/// @param initial Initial value for the counter.
pub fn extptr_counter_new(initial: i32) -> miniextendr_api::externalptr::ExternalPtr<Counter> {
    miniextendr_api::externalptr::ExternalPtr::new(Counter { value: initial })
}

/// @noRd
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn C_extptr_counter_get(ptr: SEXP) -> SEXP {
    use miniextendr_api::externalptr::ExternalPtr;
    use miniextendr_api::ffi::Rf_ScalarInteger;
    unsafe {
        match ExternalPtr::<Counter>::wrap_sexp(ptr) {
            Some(ext) => Rf_ScalarInteger(ext.value),
            None => Rf_ScalarInteger(i32::MIN), // NA_INTEGER equivalent
        }
    }
}

/// @noRd
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn C_extptr_counter_increment(ptr: SEXP) -> SEXP {
    use miniextendr_api::ffi::Rf_ScalarInteger;
    unsafe {
        // Get mutable access via downcast_mut on erased pointer
        let mut erased = ErasedExternalPtr::from_sexp(ptr);
        if let Some(counter) = erased.downcast_mut::<Counter>() {
            counter.value += 1;
            return Rf_ScalarInteger(counter.value);
        }
        Rf_ScalarInteger(i32::MIN) // NA_INTEGER equivalent
    }
}

/// @noRd
#[miniextendr]
pub fn extptr_point_new(x: f64, y: f64) -> miniextendr_api::externalptr::ExternalPtr<Point> {
    miniextendr_api::externalptr::ExternalPtr::new(Point { x, y })
}

/// @noRd
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn C_extptr_point_get_x(ptr: SEXP) -> SEXP {
    use miniextendr_api::externalptr::ExternalPtr;
    use miniextendr_api::ffi::Rf_ScalarReal;
    unsafe {
        match ExternalPtr::<Point>::wrap_sexp(ptr) {
            Some(ext) => Rf_ScalarReal(ext.x),
            None => Rf_ScalarReal(f64::NAN),
        }
    }
}

/// @noRd
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn C_extptr_point_get_y(ptr: SEXP) -> SEXP {
    use miniextendr_api::externalptr::ExternalPtr;
    use miniextendr_api::ffi::Rf_ScalarReal;
    unsafe {
        match ExternalPtr::<Point>::wrap_sexp(ptr) {
            Some(ext) => Rf_ScalarReal(ext.y),
            None => Rf_ScalarReal(f64::NAN),
        }
    }
}

/// @noRd
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn C_extptr_type_mismatch_test(ptr: SEXP) -> SEXP {
    use miniextendr_api::externalptr::ExternalPtr;
    use miniextendr_api::ffi::Rf_ScalarInteger;
    unsafe {
        // Try to interpret a Point as a Counter - should return None
        match ExternalPtr::<Counter>::wrap_sexp(ptr) {
            Some(_) => Rf_ScalarInteger(1), // Unexpected success
            None => Rf_ScalarInteger(0),    // Expected failure - type mismatch
        }
    }
}

/// @noRd
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn C_extptr_null_test(ptr: SEXP) -> SEXP {
    use miniextendr_api::externalptr::ExternalPtr;
    use miniextendr_api::ffi::Rf_ScalarInteger;
    unsafe {
        // R's new("externalptr") creates a null external pointer
        // Our wrap_sexp should return None for it
        match ExternalPtr::<Counter>::wrap_sexp(ptr) {
            Some(_) => Rf_ScalarInteger(1), // Unexpected - null pointer should fail
            None => Rf_ScalarInteger(0),    // Expected - null pointer detected
        }
    }
}

/// @noRd
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn C_extptr_is_counter(ptr: SEXP) -> SEXP {
    use miniextendr_api::ffi::Rf_ScalarInteger;
    unsafe {
        let erased = ErasedExternalPtr::from_sexp(ptr);
        if erased.is::<Counter>() {
            Rf_ScalarInteger(1)
        } else {
            Rf_ScalarInteger(0)
        }
    }
}

/// @noRd
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn C_extptr_is_point(ptr: SEXP) -> SEXP {
    use miniextendr_api::ffi::Rf_ScalarInteger;
    unsafe {
        let erased = ErasedExternalPtr::from_sexp(ptr);
        if erased.is::<Point>() {
            Rf_ScalarInteger(1)
        } else {
            Rf_ScalarInteger(0)
        }
    }
}

/// @noRd
#[miniextendr(unsafe(main_thread))]
pub fn test_extptr_on_main_thread() -> i32 {
    use miniextendr_api::externalptr::ExternalPtr;
    let ptr = ExternalPtr::new(Counter { value: 99 });
    ptr.value
}
