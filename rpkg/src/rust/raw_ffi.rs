//! Raw FFI declarations for test fixtures that need direct R API access
//! without thread-checking overhead.
//!
//! These bypass miniextendr-api's `#[r_ffi_checked]` wrappers.

use miniextendr_api::ffi::SEXP;

#[allow(non_snake_case, dead_code)]
unsafe extern "C-unwind" {
    /// Direct binding to `Rf_ScalarInteger` (no thread check).
    pub fn Rf_ScalarInteger(x: i32) -> SEXP;

    /// Direct binding to `Rf_error` (no thread check, diverges).
    /// mxl::allow(MXL300)
    pub fn Rf_error(fmt: *const std::os::raw::c_char, ...) -> !;
}
