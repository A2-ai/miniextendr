//! Safe API for R's `R_UnwindProtect`
//!
//! This module provides two approaches for handling R errors with Rust cleanup:
//!
//! - [`with_unwind_protect`]: Low-level closure-based API with explicit cleanup handler callback
//! - [`with_r_unwind_protect`]: Higher-level API that automatically runs Rust destructors
//!   when R errors occur
//!
//! **Important**: R uses `longjmp` for error handling, which normally bypasses Rust destructors.
//! Use these APIs to ensure cleanup happens even when R errors occur.
//!
#[allow(unused_imports)]
use std::panic::resume_unwind;
use std::{
    cell::LazyCell,
    ffi::c_void,
    panic::{AssertUnwindSafe, catch_unwind},
};

use crate::ffi::{self, R_ContinueUnwind, R_UnwindProtect_C_unwind, Rboolean, SEXP};

thread_local! {
    static R_CONTINUATION_TOKEN: LazyCell<crate::ffi::SEXP> = LazyCell::new(|| unsafe {
        let token = crate::ffi::R_MakeUnwindCont();
        ffi::R_PreserveObject(token);
        token
    });
}

pub fn with_r_unwind_protect<F>(f: F) -> SEXP
where
    F: FnOnce() -> SEXP,
{
    struct RError;

    unsafe extern "C-unwind" fn throw_r_error(_data: *mut c_void, jump: Rboolean) {
        if jump != Rboolean::FALSE {
            std::panic::panic_any(RError);
        }
    }

    struct CallData<F> {
        f: Option<F>,
    }

    unsafe extern "C-unwind" fn trampoline<F>(data: *mut c_void) -> SEXP
    where
        F: FnOnce() -> SEXP,
    {
        let data = unsafe { data.cast::<CallData<F>>().as_mut().unwrap() };
        let f = data.f.take().unwrap();
        f()
    }

    unsafe {
        let data = Box::into_raw(Box::new(CallData { f: Some(f) }));

        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            R_UnwindProtect_C_unwind(
                Some(trampoline::<F>),
                data.cast(),
                Some(throw_r_error),
                std::ptr::null_mut(),
                R_CONTINUATION_TOKEN.with(|x| **x),
            )
        }));

        match panic_result {
            Ok(result) => result,
            Err(payload) => {
                drop(Box::from_raw(data));
                if payload.downcast_ref::<RError>().is_some() {
                    R_ContinueUnwind(R_CONTINUATION_TOKEN.with(|x| **x));
                } else {
                    // FIRST APPROACH
                    // resume_unwind(payload);
                    // SECOND APPROACH
                    // convert every "real panic" to an R error..
                    let error_message: &str = if let Some(&message) = payload.downcast_ref::<&str>()
                    {
                        message
                    } else if let Some(message) = payload.downcast_ref::<String>() {
                        message.as_str()
                    } else if let Some(message) = payload.downcast_ref::<&String>() {
                        message.as_str()
                    } else {
                        "panic payload could not be unpacked"
                    };

                    let c_error_message =
                        std::ffi::CString::new(error_message).unwrap_or_else(|_| {
                            std::ffi::CString::new("<invalid panic message>").unwrap()
                        });

                    ::miniextendr_api::ffi::Rf_error(c"%s".as_ptr(), c_error_message.as_ptr());
                }
            }
        }
    }
}
