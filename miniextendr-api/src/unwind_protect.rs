//! Safe API for R's `R_UnwindProtect`
//!
//! This module provides [`with_r_unwind_protect`] for handling R errors with Rust cleanup.
//! It automatically runs Rust destructors when R errors occur.
//!
//! **Important**: R uses `longjmp` for error handling, which normally bypasses Rust destructors.
//! Use this API to ensure cleanup happens even when R errors occur.
//!
//! ## Log drain
//!
//! Every call to `with_r_unwind_protect` (and its variants) drains the
//! cross-thread log queue via [`drain_log_queue_if_available`] before
//! returning or re-raising an R error. This ensures that records buffered by
//! worker threads are flushed to R's console on every FFI exit — including
//! error paths.
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
    *R_CONTINUATION_TOKEN.get_or_init(|| {
        // The continuation token must be created on R's main thread
        // (R_MakeUnwindCont is an R API call). OnceLock ensures it is
        // only created once and safely shared.
        unsafe {
            let token = ffi::R_MakeUnwindCont();
            ffi::R_PreserveObject(token);
            token
        }
    })
}

/// Extract a message from a panic payload.
///
/// Handles `&str`, `String`, and `&String` payloads consistently.
/// Returns a descriptive fallback for unrecognised payload types.
pub fn panic_payload_to_string(payload: &(dyn Any + Send)) -> String {
    if let Some(&s) = payload.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else if let Some(s) = payload.downcast_ref::<&String>() {
        (*s).clone()
    } else {
        "unknown panic".to_string()
    }
}

/// Convert a Rust panic payload into an R error and continue unwinding on the R side.
pub(crate) unsafe fn panic_payload_to_r_error(
    payload: Box<dyn Any + Send>,
    call: Option<SEXP>,
    source: crate::panic_telemetry::PanicSource,
) -> ! {
    let error_message = panic_payload_to_string(payload.as_ref());

    crate::panic_telemetry::fire(&error_message, source);

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

// region: Log drain integration

/// Drain the cross-thread log queue if the `log` feature is enabled.
///
/// This is called at every exit point of `run_r_unwind_protect` (normal
/// return, Rust panic, and immediately before `R_ContinueUnwind`) so that
/// worker-thread log records always reach R's console before the FFI call
/// returns or re-raises an R error.
///
/// When the `log` feature is disabled this compiles to a no-op; there is
/// no runtime overhead.
#[inline]
fn drain_log_queue_if_available() {
    #[cfg(feature = "log")]
    crate::optionals::log_impl::drain_log_queue();
}

// endregion

/// Core R_UnwindProtect wrapper. Returns `Ok(result)` on success,
/// `Err(payload)` on Rust panic, or diverges via `R_ContinueUnwind` on R longjmp.
///
/// Handles: CallData boxing, trampoline, cleanup handler, continuation token,
/// `Box::from_raw` reclamation on all non-diverging paths.
///
/// Drains the cross-thread log queue (when the `log` feature is enabled) at
/// each exit point so worker-thread records reach R's console before the FFI
/// boundary is crossed.
fn run_r_unwind_protect<F, R>(f: F) -> Result<R, Box<dyn Any + Send>>
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
        assert!(!data.is_null(), "trampoline: data pointer is null");
        let data = unsafe { &mut *data.cast::<CallData<F, R>>() };
        let f = data.f.take().expect("trampoline: closure already consumed");

        match catch_unwind(AssertUnwindSafe(f)) {
            Ok(result) => {
                data.result = Some(result);
                crate::ffi::SEXP::nil()
            }
            Err(payload) => {
                data.panic_payload = Some(payload);
                crate::ffi::SEXP::nil()
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
                    // Drain worker-thread log records before returning the panic
                    // payload to the caller (which will convert it to an R error).
                    drain_log_queue_if_available();
                    Err(payload)
                } else {
                    // Normal completion - return the result
                    let result = data
                        .result
                        .take()
                        .expect("result not set after successful completion");
                    drop(data);
                    // Drain worker-thread log records on the normal success path.
                    drain_log_queue_if_available();
                    Ok(result)
                }
            }
            Err(payload) => {
                // Drop data first to run destructors
                drop(data);
                // Check if this was an R error or a Rust panic
                if payload.downcast_ref::<RErrorMarker>().is_some() {
                    // R error - drain log records before re-raising so worker
                    // thread output is not lost even on error exits.
                    drain_log_queue_if_available();
                    // Continue R's unwind (diverges, never returns)
                    R_ContinueUnwind(token);
                } else {
                    // Rust panic — drain before returning the payload.
                    drain_log_queue_if_available();
                    Err(payload)
                }
            }
        }
    }
}

/// Execute a closure with R unwind protection (non-`error_in_r` path).
///
/// If the closure panics, the panic is caught and converted to an R error via
/// `Rf_errorcall` (longjmp). If R raises an error (longjmp), all Rust RAII
/// resources are properly dropped before R continues unwinding.
///
/// **This is NOT the default path for `#[miniextendr]` functions.** The default
/// is [`with_r_unwind_protect_error_in_r`], which returns a tagged SEXP instead
/// of longjmping, preserving `rust_*` class layering.
///
/// This function is used by:
/// - Trait-ABI vtable shims (cross-package C-ABI calls)
/// - ALTREP `RUnwind` guard callbacks
/// - Explicit `#[miniextendr(no_error_in_r)]` / `unwrap_in_r` opt-out
///
/// In these contexts there is no R wrapper to inspect a tagged SEXP, so panics
/// must be converted to R errors directly via `Rf_errorcall`.
///
/// # Arguments
///
/// * `f` - The closure to execute
/// * `call` - Optional R call SEXP for better error messages
///
/// # Example
///
/// ```ignore
/// // Typical use: inside a trait-ABI shim where no R wrapper exists.
/// // For user-facing #[miniextendr] fns, prefer with_r_unwind_protect_error_in_r.
/// let result: i32 = with_r_unwind_protect_error_in_r(|| {
///     // Rust code that may panic or raise conditions
///     SEXP::nil()
/// }, Some(call));
/// ```
pub fn with_r_unwind_protect<F, R>(f: F, call: Option<SEXP>) -> R
where
    F: FnOnce() -> R,
{
    with_r_unwind_protect_sourced(f, call, crate::panic_telemetry::PanicSource::UnwindProtect)
}

/// Like [`with_r_unwind_protect`], but reports panics with a custom [`PanicSource`].
///
/// Used by `guarded_altrep_call` so that panics inside ALTREP callbacks with
/// `AltrepGuard::RUnwind` are still attributed to `PanicSource::Altrep`.
///
/// Also handles [`crate::condition::RCondition`] payloads in non-error_in_r mode
/// (Option A from the design): `RCondition::Error` routes to `Rf_errorcall` (call
/// attribution and `rust_*` class layering are lost — known limitation). The other
/// three variants (`Warning`, `Message`, `Condition`) panic with a diagnostic message
/// explaining that `error_in_r` mode is required for those signal kinds.
pub(crate) fn with_r_unwind_protect_sourced<F, R>(
    f: F,
    call: Option<SEXP>,
    source: crate::panic_telemetry::PanicSource,
) -> R
where
    F: FnOnce() -> R,
{
    match run_r_unwind_protect(f) {
        Ok(result) => result,
        Err(payload) => {
            // region: RCondition recognition in non-error_in_r context
            if let Some(cond) = payload.downcast_ref::<crate::condition::RCondition>() {
                match cond {
                    crate::condition::RCondition::Error { message, .. } => {
                        // Forward as Rf_errorcall. rust_* class layering is lost because
                        // there is no R wrapper to inspect the tagged SEXP — this is a
                        // known limitation of the non-error_in_r path (trait shims, ALTREP
                        // RUnwind guard). Documented in docs/CONDITIONS.md.
                        let owned = message.clone();
                        let box_payload: Box<dyn std::any::Any + Send> = Box::new(owned);
                        unsafe { panic_payload_to_r_error(box_payload, call, source) }
                    }
                    crate::condition::RCondition::Warning { .. }
                    | crate::condition::RCondition::Message { .. }
                    | crate::condition::RCondition::Condition { .. } => {
                        // warning!/message!/condition! require error_in_r mode (the
                        // default). This function is used by trait-ABI shims, ALTREP
                        // RUnwind guard, and explicit no_error_in_r/unwrap_in_r opt-out.
                        // Convert to a plain panic so the caller sees a diagnostic.
                        let msg = "warning!/message!/condition! require error_in_r mode (the \
                                   default); this function opted out via no_error_in_r/\
                                   unwrap_in_r or is a trait-ABI shim / ALTREP callback";
                        let box_payload: Box<dyn std::any::Any + Send> = Box::new(msg);
                        unsafe { panic_payload_to_r_error(box_payload, call, source) }
                    }
                }
            } else {
                unsafe { panic_payload_to_r_error(payload, call, source) }
            }
            // endregion
        }
    }
}

/// Like [`with_r_unwind_protect`], but returns a tagged error SEXP on Rust panics
/// instead of raising an R error via `Rf_errorcall`.
///
/// This is the **default** transport for all `#[miniextendr]` functions and
/// methods (both `error_in_r.unwrap_or(true)`). The error/condition SEXP is
/// inspected by the generated R wrapper which raises a proper R condition past
/// the Rust boundary, with `rust_*` class layering.
///
/// Recognises [`crate::condition::RCondition`] payloads (from `error!()`,
/// `warning!()`, `message!()`, `condition!()`) before falling through to the
/// generic panic→string path.
///
/// R-origin errors (longjmp) still pass through via `R_ContinueUnwind`.
pub fn with_r_unwind_protect_error_in_r<F>(f: F, call: Option<SEXP>) -> SEXP
where
    F: FnOnce() -> SEXP,
{
    match run_r_unwind_protect(f) {
        Ok(result) => result,
        Err(payload) => {
            // region: RCondition recognition — must come before generic panic path
            if let Some(cond) = payload.downcast_ref::<crate::condition::RCondition>() {
                let (kind, msg, class) = match cond {
                    crate::condition::RCondition::Error { message, class } => {
                        ("error", message.as_str(), class.as_deref())
                    }
                    crate::condition::RCondition::Warning { message, class } => {
                        ("warning", message.as_str(), class.as_deref())
                    }
                    crate::condition::RCondition::Message { message } => {
                        ("message", message.as_str(), None)
                    }
                    crate::condition::RCondition::Condition { message, class } => {
                        ("condition", message.as_str(), class.as_deref())
                    }
                };
                // No panic telemetry for user-raised conditions — they are intentional.
                return crate::error_value::make_rust_condition_value(msg, kind, class, call);
            }
            // endregion

            // Generic panic path — unchanged
            let msg = panic_payload_to_string(payload.as_ref());
            crate::panic_telemetry::fire(&msg, crate::panic_telemetry::PanicSource::UnwindProtect);
            crate::error_value::make_rust_error_value(&msg, "panic", call)
        }
    }
}
