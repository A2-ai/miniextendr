//! Error handling helpers for R API calls.
//!
//! ## Default path: `error_in_r` (the standard for `#[miniextendr]`)
//!
//! Every `#[miniextendr]` function runs inside
//! [`with_r_unwind_protect_error_in_r`](crate::unwind_protect::with_r_unwind_protect_error_in_r).
//! Rust panics and user-raised conditions (`error!()`, `warning!()`, `message!()`,
//! `condition!()`) are caught, packaged as a tagged SEXP, and returned normally.
//! The generated R wrapper inspects the SEXP and raises the appropriate R condition
//! with `rust_*` class layering. **No `Rf_error` longjmp happens on this path.**
//!
//! User code should use:
//! - `panic!()` — for unrecoverable Rust errors (becomes `rust_error` in R)
//! - `error!()` / `warning!()` / `message!()` / `condition!()` — for structured R conditions
//!   (see `crate::condition`)
//!
//! ## When `Rf_error` fires
//!
//! `Rf_error` / `Rf_errorcall` (longjmp) is only used on three paths where there
//! is no R wrapper to inspect a tagged SEXP:
//!
//! 1. **Trait-ABI vtable shims** — cross-package C-ABI calls go through
//!    `with_r_unwind_protect` (non-error_in_r).
//! 2. **ALTREP `RUnwind` guard** — ALTREP callbacks invoked from R's GC / vector
//!    dispatch use `with_r_unwind_protect_sourced`.
//! 3. **Explicit opt-out** — `#[miniextendr(no_error_in_r)]` / `unwrap_in_r`.
//!
//! `r_stop` (this module) is the internal Rust wrapper around `Rf_error`. It is
//! used by proc-macro generated argument validation and return-type unwrapping,
//! which run *before* the closure enters `with_r_unwind_protect`. **User code
//! should never call `r_stop` directly.**
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

/// Raise an R error via `Rf_error` (longjmp). **Do not call from user code.**
///
/// User code should use `panic!()` (caught by the framework and converted to a
/// `rust_error` R condition) or the structured condition macros `error!()` /
/// `warning!()` / `message!()` / `condition!()`.
///
/// `r_stop` is for internal use only: generated argument validation, return-type
/// unwrapping, and the FFI guard layer — all of which run in contexts where
/// `Rf_error` longjmp is safe (inside `R_UnwindProtect` or after `catch_unwind`).
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
