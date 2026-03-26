//! Error handling helpers for R API calls.
//!
//! **In `#[miniextendr]` functions, use `panic!()` instead of `r_stop`.**
//! Panics are caught by `catch_unwind` and propagated cleanly as R errors.
//!
//! `r_stop` calls `Rf_error` (longjmp). It is used internally by:
//! - The proc-macro generated argument validation / return type unwrapping
//! - The `CatchUnwind` guard (after catch_unwind has caught a panic)
//! - Trait ABI shims
//!
//! These are all inside `R_UnwindProtect` or after `catch_unwind`, where
//! `Rf_error` longjmp is safe. User code should never call `r_stop` directly.
//!
//! # Example
//!
//! ```ignore
//! use miniextendr_api::miniextendr;
//!
//! #[miniextendr]
//! fn validate_input(x: i32) -> i32 {
//!     assert!(x >= 0, "x must be non-negative, got {x}");
//!     x * 2
//! }
//! ```

/// Raise an R error via `Rf_error` (longjmp). **Do not call from user code** —
/// use `panic!()` instead, which is caught by the framework.
///
/// This is used internally by generated code and the FFI guard layer.
///
/// # Panics
///
/// Panics if the message contains null bytes.
#[inline]
pub fn r_stop(msg: &str) -> ! {
    let c_msg = std::ffi::CString::new(msg).expect("r_stop: message contains null bytes");

    if crate::worker::is_r_main_thread() {
        unsafe {
            crate::ffi::Rf_error_unchecked(c"%s".as_ptr(), c_msg.as_ptr());
        }
    } else {
        // Route to main thread
        crate::worker::with_r_thread(move || unsafe {
            crate::ffi::Rf_error_unchecked(c"%s".as_ptr(), c_msg.as_ptr());
        })
    }
}

/// Raise an R warning with the given message.
///
/// Unlike `r_stop`, this returns normally after issuing the warning.
/// Automatically routes to R's main thread if called from a worker thread.
#[inline]
pub fn r_warning(msg: &str) {
    let c_msg = std::ffi::CString::new(msg).expect("r_warning: message contains null bytes");

    if crate::worker::is_r_main_thread() {
        unsafe {
            crate::ffi::Rf_warning_unchecked(c"%s".as_ptr(), c_msg.as_ptr());
        }
    } else {
        crate::worker::with_r_thread(move || unsafe {
            crate::ffi::Rf_warning_unchecked(c"%s".as_ptr(), c_msg.as_ptr());
        });
    }
}

/// Print a message to R's console (internal implementation).
/// Automatically routes to R's main thread if called from a worker thread.
#[doc(hidden)]
#[inline]
pub fn _r_print_str(msg: &str) {
    let c_msg = std::ffi::CString::new(msg).expect("r_print!: message contains null bytes");

    if crate::worker::is_r_main_thread() {
        unsafe {
            crate::ffi::Rprintf_unchecked(c"%s".as_ptr(), c_msg.as_ptr());
        }
    } else {
        crate::worker::with_r_thread(move || unsafe {
            crate::ffi::Rprintf_unchecked(c"%s".as_ptr(), c_msg.as_ptr());
        });
    }
}

/// Print a newline to R's console (internal implementation).
/// Automatically routes to R's main thread if called from a worker thread.
#[doc(hidden)]
#[inline]
pub fn _r_print_newline() {
    if crate::worker::is_r_main_thread() {
        unsafe {
            crate::ffi::Rprintf_unchecked(c"\n".as_ptr());
        }
    } else {
        crate::worker::with_r_thread(|| unsafe {
            crate::ffi::Rprintf_unchecked(c"\n".as_ptr());
        });
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
