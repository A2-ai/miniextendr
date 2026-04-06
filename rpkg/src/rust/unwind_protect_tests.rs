//! Tests for `with_r_unwind_protect` mechanism.

use miniextendr_api::ffi::SEXP;
use miniextendr_api::miniextendr;
use miniextendr_api::unwind_protect::with_r_unwind_protect;

/// Simple RAII type that prints when dropped (without using with_r to avoid deadlocks).
/// This is used across multiple test modules.
pub(crate) struct SimpleDropMsg(pub &'static str);

impl Drop for SimpleDropMsg {
    fn drop(&mut self) {
        eprintln!("[Rust] Dropped: {}", self.0);
    }
}

/// @noRd
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
/// @name rpkg_unwind_protect
/// @noRd
/// @examples
/// \dontrun{
/// unsafe_C_unwind_protect_normal()
/// unsafe_C_unwind_protect_r_error()
/// unsafe_C_unwind_protect_lowlevel_test()
/// }
/// @aliases unsafe_C_unwind_protect_normal unsafe_C_unwind_protect_r_error
///   unsafe_C_unwind_protect_lowlevel_test
pub extern "C-unwind" fn C_unwind_protect_normal() -> SEXP {
    with_r_unwind_protect(
        || {
            let _a = SimpleDropMsg("stack resource");
            let _b = Box::new(SimpleDropMsg("heap resource"));
            ::miniextendr_api::ffi::SEXP::scalar_integer(42)
        },
        None,
    )
}

/// @noRd
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_unwind_protect_r_error() -> SEXP {
    // Create resources BEFORE the protected region
    let a = SimpleDropMsg("captured resource 1");
    let b = Box::new(SimpleDropMsg("captured resource 2 (boxed)"));

    with_r_unwind_protect(
        move || {
            // Access resources without moving them out of closure's captured state
            eprintln!("[Rust] Inside closure, using captured resources");
            eprintln!("[Rust] a.0 = {}", a.0);
            eprintln!("[Rust] b.0 = {}", b.0);

            // Now trigger R error - cleanup should drop a and b.
            // a and b are captured by the `move` closure, so they remain
            // alive at this point. Rf_error diverges (returns !).
            unsafe {
                // mxl::allow(MXL300)
                ::miniextendr_api::ffi::Rf_error(
                    c"%s".as_ptr(),
                    c"intentional R error for testing".as_ptr(),
                )
            }
        },
        None,
    )
}

/// @noRd
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_unwind_protect_lowlevel_test() -> SEXP {
    eprintln!("[Rust] Starting low-level unwind protect test");
    unsafe {
        with_r_unwind_protect(
            || {
                eprintln!("[Rust] Inside protected function, about to trigger R error");
                // mxl::allow(MXL300)
                ::miniextendr_api::ffi::Rf_error(c"%s".as_ptr(), c"test R error".as_ptr())
            },
            None,
        )
    }
}
