//! Error handling helpers for R API calls.
//!
//! ## User-facing path: tagged condition SEXP
//!
//! Every `#[miniextendr]` function runs inside
//! [`with_r_unwind_protect`](crate::unwind_protect::with_r_unwind_protect).
//! Rust panics and user-raised conditions (`error!()`, `warning!()`, `message!()`,
//! `condition!()`) are caught, packaged as a tagged SEXP, and returned normally.
//! The generated R wrapper inspects the SEXP and raises the appropriate R
//! condition with `rust_*` class layering. **No `Rf_error` longjmp happens on
//! this path.**
//!
//! User code should use:
//! - `panic!()` — for unrecoverable Rust errors (becomes `rust_error` in R)
//! - `error!()` / `warning!()` / `message!()` / `condition!()` — for structured
//!   R conditions (see [`mod@crate::condition`])
//!
//! See [`crate::error_value`] for the tagged-SEXP layout, the
//! `error_in_r` default + `no_error_in_r` / `unwrap_in_r` opt-outs, and the
//! PROTECT-discipline gotcha that R-devel surfaces.
//!
//! ## When `Rf_error` still fires (framework-internal)
//!
//! `Rf_error` (longjmp via `r_stop`) survives only at FFI guard sites where
//! there is no SEXP slot to return through:
//!
//! 1. **`ffi_guard::guarded_ffi_call(GuardMode::CatchUnwind, …)`** — worker
//!    thread panic conversion before the worker→main boundary returns a SEXP.
//! 2. **`trait_abi::check_arity`** — pre-shim arity check that runs before the
//!    vtable shim has a SEXP to return.
//!
//! ALTREP `RUnwind` callbacks now route through
//! `with_r_unwind_protect_sourced` → `raise_rust_condition_via_stop`, which
//! preserves `rust_*` class layering without going through `r_stop`.
//!
//! `r_stop` is `pub(crate)` — no user code should depend on it.
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

/// Build a NUL-safe [`CString`](std::ffi::CString) from a message.
///
/// R-bound messages are frequently user-derived (e.g. `panic!("…{user_input}…")`),
/// so they can contain interior NUL bytes. `CString::new` rejects those, and the
/// callers here run on error/warning/print paths where a panic would be
/// catastrophic (`r_stop` is already converting a panic to an R error — a second
/// panic there aborts the process). Replacing interior NUL bytes with U+FFFD
/// guarantees construction can never fail. NUL-free messages are passed through
/// unchanged, so normal output is byte-identical.
fn cstring_lossy(msg: &str) -> std::ffi::CString {
    match std::ffi::CString::new(msg) {
        Ok(c) => c,
        Err(_) => std::ffi::CString::new(msg.replace('\0', "\u{fffd}")).unwrap_or_default(),
    }
}

/// Raise an R error via `Rf_error` (longjmp). **Crate-internal only.**
///
/// Survives at two guard sites where there is no SEXP slot to return through:
/// - [`crate::ffi_guard::guarded_ffi_call`] with `GuardMode::CatchUnwind`
///   (worker thread panic conversion)
/// - [`crate::trait_abi::check_arity`] (pre-shim arity check)
///
/// User code should use `panic!()` (caught by the framework and converted to a
/// `rust_error` R condition) or the structured condition macros `error!()` /
/// `warning!()` / `message!()` / `condition!()`.
#[inline]
pub(crate) fn r_stop(msg: &str) -> ! {
    let c_msg = cstring_lossy(msg);

    if crate::worker::is_r_main_thread() {
        unsafe {
            crate::sys::Rf_error_unchecked(c"%s".as_ptr(), c_msg.as_ptr());
        }
    } else {
        // Route to main thread
        crate::worker::with_r_thread(move || unsafe {
            crate::sys::Rf_error_unchecked(c"%s".as_ptr(), c_msg.as_ptr());
        })
    }
}

/// Raise an R warning with the given message.
///
/// Unlike `r_stop`, this returns normally after issuing the warning.
/// Automatically routes to R's main thread if called from a worker thread.
#[inline]
pub fn r_warning(msg: &str) {
    let c_msg = cstring_lossy(msg);

    if crate::worker::is_r_main_thread() {
        unsafe {
            crate::sys::Rf_warning_unchecked(c"%s".as_ptr(), c_msg.as_ptr());
        }
    } else {
        crate::worker::with_r_thread(move || unsafe {
            crate::sys::Rf_warning_unchecked(c"%s".as_ptr(), c_msg.as_ptr());
        });
    }
}

/// Print a message to R's console (internal implementation).
/// Automatically routes to R's main thread if called from a worker thread.
#[doc(hidden)]
#[inline]
pub fn _r_print_str(msg: &str) {
    let c_msg = cstring_lossy(msg);

    if crate::worker::is_r_main_thread() {
        unsafe {
            crate::sys::Rprintf_unchecked(c"%s".as_ptr(), c_msg.as_ptr());
        }
    } else {
        crate::worker::with_r_thread(move || unsafe {
            crate::sys::Rprintf_unchecked(c"%s".as_ptr(), c_msg.as_ptr());
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
            crate::sys::Rprintf_unchecked(c"\n".as_ptr());
        }
    } else {
        crate::worker::with_r_thread(|| unsafe {
            crate::sys::Rprintf_unchecked(c"\n".as_ptr());
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

#[cfg(test)]
mod tests {
    use super::cstring_lossy;

    #[test]
    fn cstring_lossy_passes_through_nul_free() {
        assert_eq!(cstring_lossy("plain message").to_bytes(), b"plain message");
    }

    #[test]
    fn cstring_lossy_sanitizes_interior_nul() {
        // Must not panic, and the result must have no interior NUL byte.
        let c = cstring_lossy("a\0b");
        assert!(!c.to_bytes().contains(&0));
    }
}
