//! Tests for R return value visibility (visible/invisible returns).

use miniextendr_api::miniextendr;

#[miniextendr]
/// @title Visibility and Interrupt Tests
/// @name rpkg_visibility_interrupts
/// @noRd
/// @description Visibility and interrupt checks
/// @examples
/// invisibly_return_no_arrow()
/// force_invisible_i32()
/// with_interrupt_check(2L)
/// try(invisibly_option_return_none())
/// \dontrun{
/// unsafe_C_check_interupt_after()
/// }
/// @aliases invisibly_return_no_arrow invisibly_return_arrow invisibly_option_return_none
///   invisibly_option_return_some invisibly_result_return_ok force_invisible_i32 force_visible_unit
///   with_interrupt_check unsafe_C_check_interupt_after unsafe_C_check_interupt_unwind
pub fn invisibly_return_no_arrow() {}

/// @noRd
#[miniextendr]
#[allow(clippy::unused_unit)]
pub fn invisibly_return_arrow() -> () {}

/// @noRd
#[miniextendr]
pub fn invisibly_option_return_none() -> Option<()> {
    None // expectation: error!
}

/// @noRd
#[miniextendr]
pub fn invisibly_option_return_some() -> Option<()> {
    Some(())
}

/// @noRd
#[miniextendr]
#[allow(clippy::result_unit_err)]
pub fn invisibly_result_return_ok() -> Result<(), ()> {
    Ok(())
}

/// @noRd
#[miniextendr]
#[allow(clippy::result_unit_err)]
pub fn result_null_on_err(x: i32) -> Result<i32, ()> {
    if x >= 0 {
        Ok(x * 2)
    } else {
        Err(()) // Should return NULL
    }
}

// Test explicit invisible attribute (force i32 return to be invisible)
/// @noRd
#[miniextendr(invisible)]
pub fn force_invisible_i32() -> i32 {
    42
}

// Test explicit visible attribute (force () return to be visible)
/// @noRd
#[miniextendr(visible)]
pub fn force_visible_unit() {}

// Test check_interrupt attribute - checks for Ctrl+C before executing
/// @noRd
#[miniextendr(check_interrupt)]
pub fn with_interrupt_check(x: i32) -> i32 {
    x * 2
}

/// @noRd
#[miniextendr(unwrap_in_r, no_error_in_r)]
pub fn result_unwrap_in_r(x: i32) -> Result<i32, String> {
    if x >= 0 {
        Ok(x * 2)
    } else {
        Err(format!("negative input: {}", x))
    }
}
