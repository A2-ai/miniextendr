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
/// Raise an R error with the given message.
///
/// This function does not return - it triggers R's error handling mechanism.
/// When called inside a `#[miniextendr]` function, Rust destructors will run
/// before the error propagates to R.
///
/// Automatically routes to R's main thread if called from a worker thread.
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
