//! Safe API for R's `R_UnwindProtect`
//!
//! This module provides [`with_r_unwind_protect`] for handling R errors with Rust cleanup.
//! It automatically runs Rust destructors when R errors occur.
//!
//! **Important**: R uses `longjmp` for error handling, which normally bypasses Rust destructors.
//! Use this API to ensure cleanup happens even when R errors occur.
//!
use std::{
    any::Any,
    ffi::c_void,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::OnceLock,
};

use crate::ffi::{self, R_ContinueUnwind, R_UnwindProtect_C_unwind, Rboolean, SEXP};

/// Global continuation token for R_UnwindProtect.
///
/// Using a single global token instead of thread-local tokens avoids leaking
/// one token per thread that uses `with_r_unwind_protect`.
///
/// # Safety
///
/// The token is created and preserved once during first use. It remains valid
/// for the entire R session.
static R_CONTINUATION_TOKEN: OnceLock<SEXP> = OnceLock::new();

/// Get or create the global continuation token.
///
/// This is public for use by the worker module.
pub(crate) fn get_continuation_token() -> SEXP {
    *R_CONTINUATION_TOKEN.get_or_init(|| unsafe {
        let token = ffi::R_MakeUnwindCont();
        ffi::R_PreserveObject(token);
        token
    })
}

/// Convert a Rust panic payload into an R error and continue unwinding on the R side.
pub(crate) unsafe fn panic_payload_to_r_error(
    payload: Box<dyn Any + Send>,
    call: Option<SEXP>,
) -> ! {
    let error_message: &str = if let Some(&message) = payload.downcast_ref::<&str>() {
        message
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.as_str()
    } else if let Some(message) = payload.downcast_ref::<&String>() {
        message.as_str()
    } else {
        "panic payload could not be unpacked"
    };

    let c_error_message = std::ffi::CString::new(error_message)
        .unwrap_or_else(|_| std::ffi::CString::new("<invalid panic message>").unwrap());

    unsafe {
        if let Some(call) = call {
            ::miniextendr_api::ffi::Rf_errorcall_unchecked(
                call,
                c"%s".as_ptr(),
                c_error_message.as_ptr(),
            );
        } else {
            ::miniextendr_api::ffi::Rf_error_unchecked(c"%s".as_ptr(), c_error_message.as_ptr());
        }
    }
}

/// Execute a closure with R unwind protection.
///
/// If the closure panics, the panic is caught and converted to an R error.
/// If R raises an error (longjmp), all Rust RAII resources are properly dropped
/// before R continues unwinding.
///
/// # Arguments
///
/// * `f` - The closure to execute
/// * `call` - Optional R call SEXP for better error messages
///
/// # Example
///
/// ```ignore
/// let result: i32 = with_r_unwind_protect(|| {
///     // Code that might call R APIs that can error
///     42
/// }, None);
/// ```
pub fn with_r_unwind_protect<F, R>(f: F, call: Option<SEXP>) -> R
where
    F: FnOnce() -> R,
{
    /// Marker type for R errors caught by R_UnwindProtect's cleanup handler.
    struct RErrorMarker;

    struct CallData<F, R> {
        f: Option<F>,
        result: Option<R>,
        panic_payload: Option<Box<dyn Any + Send>>,
    }

    unsafe extern "C-unwind" fn trampoline<F, R>(data: *mut c_void) -> SEXP
    where
        F: FnOnce() -> R,
    {
        let data = unsafe { data.cast::<CallData<F, R>>().as_mut().unwrap() };
        let f = data.f.take().expect("trampoline: closure already consumed");

        match catch_unwind(AssertUnwindSafe(f)) {
            Ok(result) => {
                data.result = Some(result);
                unsafe { crate::ffi::R_NilValue }
            }
            Err(payload) => {
                data.panic_payload = Some(payload);
                unsafe { crate::ffi::R_NilValue }
            }
        }
    }

    unsafe extern "C-unwind" fn cleanup_handler(_data: *mut c_void, jump: Rboolean) {
        if jump != Rboolean::FALSE {
            // R is about to longjmp - trigger a Rust panic so we can unwind properly
            std::panic::panic_any(RErrorMarker);
        }
    }

    unsafe {
        let token = get_continuation_token();

        let data = Box::into_raw(Box::new(CallData::<F, R> {
            f: Some(f),
            result: None,
            panic_payload: None,
        }));

        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            R_UnwindProtect_C_unwind(
                Some(trampoline::<F, R>),
                data.cast(),
                Some(cleanup_handler),
                std::ptr::null_mut(),
                token,
            )
        }));

        let mut data = Box::from_raw(data);

        match panic_result {
            Ok(_) => {
                // Check if trampoline caught a panic
                if let Some(payload) = data.panic_payload.take() {
                    drop(data);
                    panic_payload_to_r_error(payload, call);
                }
                // Normal completion - return the result
                data.result
                    .take()
                    .expect("result not set after successful completion")
            }
            Err(payload) => {
                // Drop data first to run destructors
                drop(data);
                // Check if this was an R error or a Rust panic
                if payload.downcast_ref::<RErrorMarker>().is_some() {
                    // R error - continue R's unwind
                    R_ContinueUnwind(token);
                } else {
                    // Rust panic - convert to R error
                    panic_payload_to_r_error(payload, call);
                }
            }
        }
    }
}
