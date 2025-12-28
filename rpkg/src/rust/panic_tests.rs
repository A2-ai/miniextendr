//! Panic, drop, and R error handling tests.

use miniextendr_api::ffi::{Rf_error, SEXP};
use miniextendr_api::{miniextendr, miniextendr_module};

// region: MsgOnDrop for testing drop behavior

#[derive(Debug)]
/// RAII helper that logs a message when dropped, used to verify destructor paths.
pub(crate) struct MsgOnDrop;

impl Drop for MsgOnDrop {
    fn drop(&mut self) {
        use std::io::Write;
        let mut stdout = std::io::stdout().lock();
        writeln!(stdout, "[Rust] Dropped `MsgOnDrop`!").unwrap();
        stdout.flush().unwrap();
    }
}

#[miniextendr]
pub fn drop_message_on_success() -> i32 {
    let _a = MsgOnDrop;
    42
}

#[miniextendr]
pub fn drop_on_panic() {
    let _a = MsgOnDrop;
    panic!()
}

#[miniextendr]
pub fn drop_on_panic_with_move() {
    let _a = MsgOnDrop;
    unsafe {
        Rf_error(c"%s".as_ptr(), c"an r error occurred".as_ptr());
    }
}

// endregion

// region: panics, (), and Result

#[miniextendr]
#[allow(clippy::unused_unit)]
pub fn take_and_return_nothing() -> () {}

#[miniextendr]
/// @title Arithmetic Tests
/// @name rpkg_arithmetic
/// @description Arithmetic and return-value tests
/// @param left Integer input.
/// @param right Integer input.
/// @return A scalar integer result, or invisibly returns nothing for take_and_return_nothing().
/// @examples
/// add(1L, 2L)
/// add2(1L, 2L, NULL)
/// add3(1L, 2L, NULL)
/// add4(10L, 2L)
/// add_left_mut(1L, 2L)
/// take_and_return_nothing()
/// @aliases add add2 add3 add4 add_left_mut add_right_mut add_left_right_mut
///   take_and_return_nothing
pub fn add(left: i32, right: i32) -> i32 {
    left + right
}

#[miniextendr]
pub fn add2(left: i32, right: i32, _dummy: ()) -> i32 {
    left + right
}

#[derive(Debug)]
pub struct RustError;

impl From<()> for RustError {
    fn from(_value: ()) -> Self {
        Self
    }
}

#[miniextendr]
pub fn add3(left: i32, right: i32, _dummy: ()) -> Result<i32, RustError> {
    left.checked_add(right).ok_or(().into())
}

#[miniextendr]
pub fn add4(left: i32, right: i32) -> Result<i32, &'static str> {
    left.checked_div(right).ok_or("don't divide by zero dude")
}

fn inner_panicking_function() {
    let x: Option<i32> = None;
    #[allow(clippy::unnecessary_literal_unwrap)]
    x.unwrap();
}

fn middle_function() {
    inner_panicking_function();
}

#[miniextendr]
pub fn nested_panic() {
    middle_function();
}

#[miniextendr]
/// @title Panic and Error Handling Tests
/// @name rpkg_panic_tests
/// @keywords internal
/// @description Panic and error handling tests (unsafe)
/// @examples
/// try(add_panic(1L, 2L))
/// try(add_r_error(1L, 2L))
/// \dontrun{
/// unsafe_C_r_error_in_thread()
/// }
/// @aliases nested_panic add_panic add_panic_heap add_r_error add_r_error_heap
///   drop_message_on_success drop_on_panic drop_on_panic_with_move unsafe_C_just_panic
///   unsafe_C_panic_and_catch unsafe_C_r_error unsafe_C_r_error_in_catch unsafe_C_r_error_in_thread
///   unsafe_C_r_print_in_thread
pub fn add_panic(_left: i32, _right: i32) -> i32 {
    let _a = MsgOnDrop;
    panic!("we cannot add right now! ");
    #[allow(unreachable_code)]
    {
        _left + _right
    }
}

#[miniextendr]
pub fn add_r_error(_left: i32, _right: i32) -> i32 {
    let _a = MsgOnDrop;
    unsafe {
        ::miniextendr_api::ffi::Rf_error(c"%s".as_ptr(), c"r error in `add_r_error`".as_ptr())
    };
    #[allow(unreachable_code)]
    {
        _left + _right
    }
}

#[miniextendr]
pub fn add_panic_heap(_left: i32, _right: i32) -> i32 {
    let _a = Box::new(MsgOnDrop);
    panic!("we cannot add right now! ")
}

#[miniextendr]
pub fn add_r_error_heap(_left: i32, _right: i32) -> i32 {
    let _a = Box::new(MsgOnDrop);
    unsafe {
        ::miniextendr_api::ffi::Rf_error(c"%s".as_ptr(), c"r error in `add_r_error`".as_ptr())
    }
}

// endregion

// region: `mut` checks

#[miniextendr]
pub fn add_left_mut(mut left: i32, right: i32) -> i32 {
    let left = &mut left;
    *left + right
}

#[miniextendr]
pub fn add_right_mut(left: i32, right: i32) -> i32 {
    left + right
}

#[miniextendr]
pub fn add_left_right_mut(left: i32, right: i32) -> i32 {
    left + right
}

// endregion

// region: panic printing

#[unsafe(no_mangle)]
#[miniextendr]
pub extern "C-unwind" fn C_just_panic() -> SEXP {
    panic!("just panic, no capture");
}

/// If you call a miniextendr function that panics, and then `C_panic_catch`,
/// you'll see that the panic hook was not reset.
#[miniextendr]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C-unwind" fn C_panic_and_catch() -> SEXP {
    let result = std::panic::catch_unwind(|| panic!("just panic, no capture"));
    let _ = dbg!(result);
    unsafe { ::miniextendr_api::ffi::R_NilValue }
}

#[miniextendr]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C-unwind" fn C_r_error() -> SEXP {
    // Use unchecked - this is testing raw R error behavior
    unsafe { miniextendr_api::ffi::Rf_error_unchecked(c"arg1".as_ptr()) }
}

#[miniextendr]
#[allow(non_snake_case)]
#[allow(clippy::diverging_sub_expression)]
#[unsafe(no_mangle)]
pub extern "C-unwind" fn C_r_error_in_catch() -> SEXP {
    unsafe {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            // Use unchecked - this is testing raw R error behavior
            miniextendr_api::ffi::Rf_error_unchecked(c"arg1".as_ptr())
        }))
        .unwrap();
        miniextendr_api::ffi::R_NilValue
    }
}

/// This crashes immediately. R is simply not present on the spawned thread, hence the present segfault.
/// With the checked `Rf_error`, this would panic instead (which is the correct behavior).
#[miniextendr]
#[allow(non_snake_case)]
#[allow(clippy::diverging_sub_expression)]
#[unsafe(no_mangle)]
pub extern "C-unwind" fn C_r_error_in_thread() -> SEXP {
    // Use checked Rf_error - will panic with clear message about wrong thread
    std::thread::spawn(|| unsafe {
        miniextendr_api::ffi::Rf_error(c"%s".as_ptr(), c"arg1".as_ptr())
    })
    .join()
    .unwrap();
    unsafe { miniextendr_api::ffi::R_NilValue }
}

/// This will segfault, as R is not present on the spawned thread.
#[miniextendr]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C-unwind" fn C_r_print_in_thread() -> SEXP {
    std::thread::spawn(|| unsafe { miniextendr_api::ffi::Rprintf_unchecked(c"arg1".as_ptr()) })
        .join()
        .unwrap();
    unsafe { miniextendr_api::ffi::R_NilValue }
}

// endregion

miniextendr_module! {
    mod panic_tests;

    fn add;
    fn add2;
    fn add3;
    fn add4;
    fn nested_panic;
    fn add_panic;
    fn add_r_error;

    fn add_panic_heap;
    fn add_r_error_heap;

    fn add_left_mut;
    fn add_right_mut;
    fn add_left_right_mut;

    fn take_and_return_nothing;

    extern "C-unwind" fn C_just_panic;
    extern "C-unwind" fn C_panic_and_catch;

    fn drop_message_on_success;
    fn drop_on_panic;
    fn drop_on_panic_with_move;

    extern fn C_r_error;
    extern fn C_r_error_in_catch;
    extern fn C_r_error_in_thread;
    extern fn C_r_print_in_thread;
}
