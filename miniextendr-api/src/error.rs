//! Error handling helpers for R API calls.
//!
//! Inside `#[miniextendr]` functions, R API calls are automatically protected
//! by `with_r_unwind_protect`. This means:
//!
//! - R errors (via `Rf_error`) will trigger proper Rust destructor cleanup
//! - After cleanup, the error propagates to R normally
//!
//! # Example
//!
//! ```ignore
//! use miniextendr_api::{miniextendr, r_error};
//!
//! #[miniextendr]
//! fn validate_input(x: i32) -> i32 {
//!     if x < 0 {
//!         r_error!("x must be non-negative, got {}", x);
//!     }
//!     x * 2
//! }
//! ```
/// Debug-only check that we're on R's main thread.
#[inline]
fn debug_assert_main_thread(#[allow(unused)] fn_name: &str) {
    #[cfg(debug_assertions)]
    if !crate::worker::is_r_main_thread() {
        panic!("{} called from non-main thread", fn_name);
    }
}

/// Raise an R error with the given message.
///
/// This function does not return - it triggers R's error handling mechanism.
/// When called inside a `#[miniextendr]` function, Rust destructors will run
/// before the error propagates to R.
///
/// # Panics
///
/// Panics if the message contains null bytes, or in debug builds if called
/// from a non-main thread.
#[inline]
pub fn r_stop(msg: &str) -> ! {
    debug_assert_main_thread("r_stop");
    let c_msg = std::ffi::CString::new(msg).expect("r_stop: message contains null bytes");
    unsafe {
        crate::ffi::Rf_error_unchecked(c"%s".as_ptr(), c_msg.as_ptr());
    }
}

/// Raise an R error with a formatted message.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::r_error;
///
/// r_error!("Invalid value: {}", value);
/// ```
#[macro_export]
macro_rules! r_error {
    ($($arg:tt)*) => {
        $crate::error::r_stop(&format!($($arg)*))
    };
}

/// Raise an R warning with the given message.
///
/// Unlike `r_stop`, this returns normally after issuing the warning.
#[inline]
pub fn r_warning(msg: &str) {
    debug_assert_main_thread("r_warning");
    let c_msg = std::ffi::CString::new(msg).expect("r_warning: message contains null bytes");
    unsafe {
        crate::ffi::Rf_warning_unchecked(c"%s".as_ptr(), c_msg.as_ptr());
    }
}

/// Print a message to R's console (internal implementation).
#[doc(hidden)]
#[inline]
pub fn _r_print_str(msg: &str) {
    debug_assert_main_thread("r_print!");
    let c_msg = std::ffi::CString::new(msg).expect("r_print!: message contains null bytes");
    unsafe {
        crate::ffi::Rprintf_unchecked(c"%s".as_ptr(), c_msg.as_ptr());
    }
}

/// Print a newline to R's console (internal implementation).
#[doc(hidden)]
#[inline]
pub fn _r_print_newline() {
    debug_assert_main_thread("r_println!");
    unsafe {
        crate::ffi::Rprintf_unchecked(c"\n".as_ptr());
    }
}

/// Print to R's console (like `print!`).
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::r_print;
///
/// r_print!("Hello ");
/// r_print!("value: {}", 42);
/// ```
#[macro_export]
macro_rules! r_print {
    () => {};
    ($($arg:tt)*) => {
        $crate::error::_r_print_str(&format!($($arg)*))
    };
}

/// Print to R's console with a newline (like `println!`).
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::r_println;
///
/// r_println!();  // just a newline
/// r_println!("Hello, world!");
/// r_println!("value: {}", 42);
/// ```
#[macro_export]
macro_rules! r_println {
    () => {
        $crate::error::_r_print_newline()
    };
    ($($arg:tt)*) => {{
        $crate::error::_r_print_str(&format!($($arg)*));
        $crate::error::_r_print_newline();
    }};
}
