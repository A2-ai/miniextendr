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
use std::cell::LazyCell;

use crate::ffi;

thread_local! {
    static R_CONTINUATION_TOKEN: LazyCell<crate::ffi::SEXP> = LazyCell::new(|| unsafe {
        let token = crate::ffi::R_MakeUnwindCont();
        ffi::R_PreserveObject(token);
        token
    });
}

struct ClosureContext<FunClosure, CleanClosure> {
    fun: Option<FunClosure>,
    clean: Option<CleanClosure>,
}

unsafe extern "C" fn fun_tramp<F, C>(data: *mut std::ffi::c_void) -> ffi::SEXP
where
    F: FnOnce() -> ffi::SEXP,
{
    let ctx = unsafe { data.cast::<ClosureContext<F, C>>().as_mut().unwrap() };
    let f = ctx.fun.take().unwrap();
    f()
}

unsafe extern "C" fn clean_tramp<F, C>(data: *mut std::ffi::c_void, jump: ffi::Rboolean)
where
    C: FnOnce(bool),
{
    let closure_ctx = unsafe { Box::from_raw(data.cast::<ClosureContext<F, C>>()) };
    let ClosureContext { fun, clean } = *closure_ctx;
    if let Some(fun) = fun {
        drop(fun)
    }
    if let Some(clean) = clean {
        clean(jump != ffi::Rboolean::FALSE)
    }
}

/// Wrap a Rust closure with `R_UnwindProtect`.
/// `clean` sees `true` if a non-local jump happened, `false` on normal return.
///
///
/// # Safety
/// Must be called from the R dispatcher thread. Closures must not unwind across FFI.
pub unsafe fn with_unwind_protect<FunClosure, CleanClosure>(
    fun: FunClosure,
    clean: CleanClosure,
) -> ffi::SEXP
where
    FunClosure: FnOnce() -> ffi::SEXP,
    CleanClosure: FnOnce(bool),
{
    let data = Box::into_raw(Box::new(ClosureContext {
        fun: Some(fun),
        clean: Some(clean),
    }));

    unsafe {
        ffi::R_UnwindProtect(
            Some(fun_tramp::<FunClosure, CleanClosure>),
            data.cast(),
            Some(clean_tramp::<FunClosure, CleanClosure>),
            data.cast(),
            R_CONTINUATION_TOKEN.with(|x| **x),
        )
    }
}

// =============================================================================
// Higher-level API with automatic cleanup
// =============================================================================

/// Execute a closure with R unwind protection.
///
/// **Important**: R uses `longjmp` for error handling, which normally bypasses Rust
/// destructors. This function ensures that Rust destructors run even when R errors occur.
///
/// # How it works
///
/// 1. The closure is executed within `R_UnwindProtect`
/// 2. If an R error occurs (e.g., `Rf_error()`), R calls our cleanup callback
/// 3. The cleanup callback drops all captured data before R continues the error
/// 4. `R_ContinueUnwind` propagates the R error after cleanup
///
/// # Example
///
/// ```ignore
/// struct MyResource;
/// impl Drop for MyResource {
///     fn drop(&mut self) { eprintln!("Cleaned up!"); }
/// }
///
/// #[no_mangle]
/// pub extern "C" fn my_r_function() -> SEXP {
///     with_r_unwind_protect(|| {
///         let _resource = MyResource;  // Will be dropped even on R error!
///         unsafe { some_r_api_that_might_error() };
///         create_result()
///     })
/// }
/// ```
///
/// # Safety
///
/// Must be called from the R dispatcher thread.
/// Execute a closure with R unwind protection and explicit cleanup data.
///
/// **This is the recommended API** for guaranteed cleanup. Resources passed via
/// `cleanup_data` WILL be dropped when an R error occurs.
///
/// # How it works
///
/// 1. The closure is executed within `R_UnwindProtect`
/// 2. If an R error occurs, R calls our cleanup callback
/// 3. The cleanup callback drops `cleanup_data` (running its `Drop` impl)
/// 4. `R_ContinueUnwind` propagates the R error
///
/// # Example
///
/// ```ignore
/// struct MyResource;
/// impl Drop for MyResource {
///     fn drop(&mut self) { eprintln!("Cleaned up!"); }
/// }
///
/// let resource = MyResource;
/// with_r_unwind_protect_cleanup(resource, |res| {
///     // Use res here - it will be dropped on R error
///     unsafe { some_r_api_that_might_error() };
///     create_result()
/// })
/// ```
pub fn with_r_unwind_protect_cleanup<C, F, T>(cleanup_data: C, f: F) -> T
where
    C: 'static,
    F: FnOnce(&mut C) -> T,
{
    struct CallData<C, F, T> {
        cleanup: Option<C>,
        f: Option<F>,
        result: Option<T>,
        jump_occurred: bool,
    }

    unsafe extern "C" fn trampoline<C, F, T>(data: *mut std::ffi::c_void) -> ffi::SEXP
    where
        F: FnOnce(&mut C) -> T,
    {
        let data = unsafe { &mut *(data as *mut CallData<C, F, T>) };
        let f = data.f.take().unwrap();
        let cleanup = data.cleanup.as_mut().unwrap();
        data.result = Some(f(cleanup));
        std::ptr::null_mut()
    }

    unsafe extern "C" fn cleanup_cb<C, F, T>(data: *mut std::ffi::c_void, jump: ffi::Rboolean) {
        let data = unsafe { &mut *(data as *mut CallData<C, F, T>) };
        if jump != ffi::Rboolean::FALSE {
            // R error occurred - mark it and drop cleanup data
            data.jump_occurred = true;
            data.cleanup.take(); // This drops the cleanup data!
            data.f.take();
            data.result.take();
        }
    }

    unsafe {
        let cont = ffi::Rf_protect(ffi::R_MakeUnwindCont());

        let mut data = CallData {
            cleanup: Some(cleanup_data),
            f: Some(f),
            result: None,
            jump_occurred: false,
        };

        ffi::R_UnwindProtect(
            Some(trampoline::<C, F, T>),
            &mut data as *mut _ as *mut std::ffi::c_void,
            Some(cleanup_cb::<C, F, T>),
            &mut data as *mut _ as *mut std::ffi::c_void,
            cont,
        );

        if data.jump_occurred {
            ffi::R_ContinueUnwind(cont);
        }

        ffi::Rf_unprotect(1);
        data.result.unwrap()
    }
}

/// Convenience wrapper that takes a simple closure without explicit cleanup data.
///
/// **Warning**: Resources created inside the closure will NOT be dropped if an R
/// error occurs. Use [`with_r_unwind_protect_cleanup`] for guaranteed cleanup.
pub fn with_r_unwind_protect<F, T>(f: F) -> T
where
    F: FnOnce() -> T,
{
    with_r_unwind_protect_cleanup((), |_| f())
}

// =============================================================================
// Helper functions for error handling
// =============================================================================

/// Convert a panic payload to a string message.
#[doc(hidden)]
pub fn panic_payload_to_string(
    panic: Box<dyn std::any::Any + Send>,
) -> std::borrow::Cow<'static, str> {
    match panic.downcast::<String>() {
        Ok(message) => std::borrow::Cow::Owned(*message),
        Err(panic) => match panic.downcast::<&'static str>() {
            Ok(ref message) => std::borrow::Cow::Borrowed(message),
            Err(_) => std::borrow::Cow::Borrowed("panic payload could not be unpacked"),
        },
    }
}

/// Raise an R error with a call context (does not return).
#[doc(hidden)]
pub unsafe fn raise_r_error_call(call: ffi::SEXP, message: &str) -> ! {
    let c_message = std::ffi::CString::new(message)
        .unwrap_or_else(|_| std::ffi::CString::new("<invalid panic message>").unwrap());
    unsafe {
        ffi::Rf_errorcall(call, c"%s".as_ptr(), c_message.as_ptr());
    }
}

/// Raise an R error (does not return).
#[doc(hidden)]
pub fn raise_r_error(message: &str) -> ! {
    let c_message = std::ffi::CString::new(message)
        .unwrap_or_else(|_| std::ffi::CString::new("<invalid panic message>").unwrap());
    unsafe {
        ffi::Rf_error(c"%s".as_ptr(), c_message.as_ptr());
    }
}

