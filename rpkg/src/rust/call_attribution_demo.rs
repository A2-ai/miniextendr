//! Side-by-side fixture for `docs/CALL_ATTRIBUTION.md`.
//!
//! Two functions raise the same error message. One goes through the standard
//! `#[miniextendr]` wrapper (which emits `.call = match.call()`); the other is
//! `extern "C-unwind"`, which has no generated R wrapper and so no call slot.
//! The R-side error rendering is dramatically different.

use miniextendr_api::ffi::SEXP;
use miniextendr_api::miniextendr;

/// Wrapped path. The generated R wrapper passes `.call = match.call()` into the
/// C entry; on panic, `Rf_errorcall(call, msg)` shows the user's call frame.
///
/// @param left Ignored.
/// @param right Ignored.
/// @export
#[miniextendr]
pub fn call_attr_with(_left: i32, _right: i32) -> i32 {
    panic!("left + right is too risky")
}

/// Unwrapped path. `extern "C-unwind"` bypasses the wrapper entirely — there is
/// no call slot and no `with_r_unwind_protect`. We raise an R error directly
/// with `Rf_error`, which carries no call attribution.
///
/// @param left Ignored.
/// @param right Ignored.
/// @export
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_call_attr_without(_left: SEXP, _right: SEXP) -> SEXP {
    unsafe {
        ::miniextendr_api::ffi::Rf_error(c"%s".as_ptr(), c"left + right is too risky".as_ptr()) // mxl::allow(MXL300)
    }
}
