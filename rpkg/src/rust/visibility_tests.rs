//! Tests for R return value visibility (visible/invisible returns).

use miniextendr_api::{miniextendr, miniextendr_module};

#[miniextendr]
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
    fn force_invisible_i32;
    fn force_visible_unit;
    fn with_interrupt_check;
}
