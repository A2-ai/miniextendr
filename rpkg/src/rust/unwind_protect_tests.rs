//! Tests for `with_r_unwind_protect` mechanism.

use miniextendr_api::ffi::SEXP;
use miniextendr_api::unwind_protect::with_r_unwind_protect;
use miniextendr_api::{miniextendr, miniextendr_module};

/// Simple RAII type that prints when dropped (without using with_r to avoid deadlocks).
/// This is used across multiple test modules.
pub(crate) struct SimpleDropMsg(pub &'static str);

impl Drop for SimpleDropMsg {
    fn drop(&mut self) {
        eprintln!("[Rust] Dropped: {}", self.0);
    }
}

/// Test that with_r_unwind_protect works for normal (non-error) path.
/// Destructors should run normally when the closure completes successfully.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_unwind_protect_normal() -> SEXP {
    with_r_unwind_protect(
        || {
            let _a = SimpleDropMsg("stack resource");
            let _b = Box::new(SimpleDropMsg("heap resource"));
            unsafe { ::miniextendr_api::ffi::Rf_ScalarInteger(42) }
        },
        None,
    )
}

/// Test that with_r_unwind_protect cleans up on R error.
/// Resources captured by the closure ARE dropped when an R error occurs.
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

            // Now trigger R error - cleanup should drop a and b
            unsafe {
                ::miniextendr_api::ffi::Rf_error(
                    c"%s".as_ptr(),
                    c"intentional R error for testing".as_ptr(),
                )
            };
            #[allow(unreachable_code)]
            unsafe {
                // This is never reached, but we need to "use" a and b
                // to prevent the compiler from moving them earlier
                drop(a);
                drop(b);
                ::miniextendr_api::ffi::R_NilValue
            }
        },
        None,
    )
}

/// Minimal test using low-level with_unwind_protect
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_unwind_protect_lowlevel_test() -> SEXP {
    eprintln!("[Rust] Starting low-level unwind protect test");
    unsafe {
        with_r_unwind_protect(
            || {
                eprintln!("[Rust] Inside protected function, about to trigger R error");
                ::miniextendr_api::ffi::Rf_error(c"%s".as_ptr(), c"test R error".as_ptr());
                #[allow(unreachable_code)]
                ::miniextendr_api::ffi::R_NilValue
            },
            None,
        )
    }
}

miniextendr_module! {
    mod unwind_protect_tests;

    extern "C-unwind" fn C_unwind_protect_normal;
    extern "C-unwind" fn C_unwind_protect_r_error;
    extern "C-unwind" fn C_unwind_protect_lowlevel_test;
}
