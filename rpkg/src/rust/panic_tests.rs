//! Panic, drop, and R error handling tests.

use miniextendr_api::ffi::{Rf_error, SEXP};
use miniextendr_api::miniextendr;

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

/// @noRd
#[miniextendr]
pub fn drop_message_on_success() -> i32 {
    let _a = MsgOnDrop;
    42
}

/// @noRd
#[miniextendr]
pub fn drop_on_panic() {
    let _a = MsgOnDrop;
    panic!()
}

/// @noRd
#[miniextendr]
pub fn drop_on_panic_with_move() {
    let _a = MsgOnDrop;
    unsafe {
        Rf_error(c"%s".as_ptr(), c"an r error occurred".as_ptr()); // mxl::allow(MXL300)
    }
}

// endregion

// region: panics, (), and Result

/// @noRd
#[miniextendr]
#[allow(clippy::unused_unit)]
pub fn take_and_return_nothing() -> () {}

#[miniextendr]
/// @title Arithmetic Tests
/// @name rpkg_arithmetic
/// @description Arithmetic and return-value tests
/// @param left Integer input.
/// @param right Integer input.
/// @return A scalar integer result.
/// @examples
/// add(1L, 2L)
pub fn add(left: i32, right: i32) -> i32 {
    left + right
}

/// @noRd
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

/// @noRd
#[miniextendr]
pub fn add3(left: i32, right: i32, _dummy: ()) -> Result<i32, RustError> {
    left.checked_add(right).ok_or(().into())
}

/// @noRd
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

/// @noRd
#[miniextendr]
pub fn nested_panic() {
    middle_function();
}

#[miniextendr]
/// @title Panic and Error Handling Tests
/// @name rpkg_panic_tests
/// @noRd
/// @description Panic and error handling tests (unsafe)
/// @examples
/// try(add_panic(1L, 2L))
/// try(add_r_error(1L, 2L))
/// \dontrun{
/// # These use checked FFI wrappers that detect wrong-thread usage,
/// # but thread panic propagation causes runtime errors.
/// miniextendr:::unsafe_C_r_error_in_thread()
/// miniextendr:::unsafe_C_r_print_in_thread()
/// }
/// @aliases nested_panic add_panic add_panic_heap add_r_error add_r_error_heap
///   drop_message_on_success drop_on_panic drop_on_panic_with_move unsafe_C_just_panic
///   unsafe_C_panic_and_catch unsafe_C_r_error unsafe_C_r_error_in_catch unsafe_C_r_error_in_thread
///   unsafe_C_r_print_in_thread
pub fn add_panic(_left: i32, _right: i32) -> i32 {
    let _a = MsgOnDrop;
    panic!("we cannot add right now! ")
}

/// @noRd
#[miniextendr]
pub fn add_r_error(_left: i32, _right: i32) -> i32 {
    let _a = MsgOnDrop;
    unsafe {
        // mxl::allow(MXL300)
        ::miniextendr_api::ffi::Rf_error(c"%s".as_ptr(), c"r error in `add_r_error`".as_ptr())
    }
}

/// @noRd
#[miniextendr]
pub fn add_panic_heap(_left: i32, _right: i32) -> i32 {
    let _a = Box::new(MsgOnDrop);
    panic!("we cannot add right now! ")
}

/// @noRd
#[miniextendr]
pub fn add_r_error_heap(_left: i32, _right: i32) -> i32 {
    let _a = Box::new(MsgOnDrop);
    unsafe {
        // mxl::allow(MXL300)
        ::miniextendr_api::ffi::Rf_error(c"%s".as_ptr(), c"r error in `add_r_error`".as_ptr())
    }
}

// endregion

// region: `mut` checks

/// Note: `mut` tests macro handling of mutable parameters; becomes unused after expansion.
#[allow(unused_mut)]
/// @noRd
#[miniextendr]
pub fn add_left_mut(mut left: i32, right: i32) -> i32 {
    let left = &mut left;
    *left + right
}

/// @noRd
#[miniextendr]
pub fn add_right_mut(left: i32, right: i32) -> i32 {
    left + right
}

/// @noRd
#[miniextendr]
pub fn add_left_right_mut(left: i32, right: i32) -> i32 {
    left + right
}

// endregion

// region: panic printing

/// @noRd
#[unsafe(no_mangle)]
#[miniextendr]
pub extern "C-unwind" fn C_just_panic() -> SEXP {
    panic!("just panic, no capture");
}

/// @noRd
#[miniextendr]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C-unwind" fn C_panic_and_catch() -> SEXP {
    let result = std::panic::catch_unwind(|| panic!("just panic, no capture"));
    let _ = dbg!(result);
    ::miniextendr_api::ffi::SEXP::nil()
}

/// @noRd
#[miniextendr]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C-unwind" fn C_r_error() -> SEXP {
    // mxl::allow(MXL300, MXL301)
    unsafe { crate::raw_ffi::Rf_error(c"arg1".as_ptr()) }
}

/// @noRd
#[miniextendr]
#[allow(non_snake_case)]
#[allow(clippy::diverging_sub_expression)]
#[unsafe(no_mangle)]
pub extern "C-unwind" fn C_r_error_in_catch() -> SEXP {
    unsafe {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            // mxl::allow(MXL300, MXL301)
            crate::raw_ffi::Rf_error(c"arg1".as_ptr())
        }))
        .unwrap();
        miniextendr_api::ffi::SEXP::nil()
    }
}

/// Extract panic message from a thread join error.
fn extract_panic_message(e: Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = e.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = e.downcast_ref::<String>() {
        s.clone()
    } else {
        "thread panicked".to_string()
    }
}

/// @noRd
#[miniextendr]
#[allow(non_snake_case)]
#[allow(clippy::diverging_sub_expression)]
#[unsafe(no_mangle)]
pub extern "C-unwind" fn C_r_error_in_thread() -> SEXP {
    // Use checked Rf_error - will panic with clear message about wrong thread.
    // Since Rf_error returns !, the thread always panics, so unwrap_err is safe.
    let e = std::thread::spawn(|| unsafe {
        miniextendr_api::ffi::Rf_error(c"%s".as_ptr(), c"arg1".as_ptr()) // mxl::allow(MXL300)
    })
    .join()
    .unwrap_err();

    panic!("{}", extract_panic_message(e));
}

/// @noRd
#[miniextendr]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C-unwind" fn C_r_print_in_thread() -> SEXP {
    // Use checked Rprintf - will panic with clear message about wrong thread.
    let result = std::thread::spawn(|| unsafe {
        miniextendr_api::ffi::Rprintf(c"%s".as_ptr(), c"arg1".as_ptr())
    })
    .join();

    match result {
        Ok(()) => miniextendr_api::ffi::SEXP::nil(),
        Err(e) => panic!("{}", extract_panic_message(e)),
    }
}

// endregion
