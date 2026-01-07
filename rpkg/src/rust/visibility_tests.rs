//! Tests for R return value visibility (visible/invisible returns).

use miniextendr_api::{miniextendr, miniextendr_module};

#[miniextendr]
/// @title Visibility and Interrupt Tests
/// @name rpkg_visibility_interrupts
/// @keywords internal
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

#[miniextendr]
#[allow(clippy::unused_unit)]
pub fn invisibly_return_arrow() -> () {}

#[miniextendr]
pub fn invisibly_option_return_none() -> Option<()> {
    None // expectation: error!
}

#[miniextendr]
pub fn invisibly_option_return_some() -> Option<()> {
    Some(())
}

#[miniextendr]
#[allow(clippy::result_unit_err)]
pub fn invisibly_result_return_ok() -> Result<(), ()> {
    Ok(())
}

/// Test Result<i32, ()> - Err returns NULL instead of erroring
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
#[miniextendr(invisible)]
pub fn force_invisible_i32() -> i32 {
    42
}

// Test explicit visible attribute (force () return to be visible)
#[miniextendr(visible)]
pub fn force_visible_unit() {}

// Test check_interrupt attribute - checks for Ctrl+C before executing
#[miniextendr(check_interrupt)]
pub fn with_interrupt_check(x: i32) -> i32 {
    x * 2
}

miniextendr_module! {
    mod visibility_tests;

    fn invisibly_return_no_arrow;
    fn invisibly_return_arrow;
    fn invisibly_option_return_none;
    fn invisibly_option_return_some;
    fn invisibly_result_return_ok;
    fn result_null_on_err;
    fn force_invisible_i32;
    fn force_visible_unit;
    fn with_interrupt_check;
}
