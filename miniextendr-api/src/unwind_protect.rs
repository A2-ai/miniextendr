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

// region: raise_rust_condition_via_stop — Approach 3 for ALTREP RUnwind path

/// Cached `stop` symbol (permanently interned via `Rf_install`).
fn stop_sym() -> crate::ffi::SEXP {
    static CACHE: OnceLock<crate::ffi::SEXP> = OnceLock::new();
    *CACHE.get_or_init(|| unsafe { crate::ffi::Rf_install(c"stop".as_ptr()) })
}

/// Raise an R condition with `rust_*` class layering by evaluating
/// `stop(structure(list(message = msg, call = call), class = c(...)))`.
///
/// This is **Approach 3** from the issue-345 plan: the `Rf_eval(stop(...))` pattern
/// that works in any context where there is no outer R wrapper to inspect a tagged SEXP.
/// It is the only viable option for ALTREP callbacks, which are invoked directly by
/// R's runtime (no `.Call` frame, no R wrapper).
///
/// The `stop()` call longjmps, so this function never returns — declared `-> !`.
///
/// ## Class layering
///
/// - If `class` is `Some("my_class")`, the resulting R condition has class:
///   `c("my_class", "rust_error", "simpleError", "error", "condition")`.
/// - Without a custom class: `c("rust_error", "simpleError", "error", "condition")`.
///
/// ## MXL300 compliance
///
/// This function raises an R error via `Rf_eval(stop(...))`, not via direct
/// `Rf_error`/`Rf_errorcall`. MXL300 does not flag `Rf_eval`.
///
/// # Safety
///
/// Must be called from R's main thread inside an `R_UnwindProtect` cleanup
/// or equivalent context where R longjmps are safe. In practice, always called
/// from `with_r_unwind_protect_sourced` on the ALTREP guard path.
pub(crate) unsafe fn raise_rust_condition_via_stop(
    message: &str,
    class: Option<&str>,
    call: Option<crate::ffi::SEXP>,
) -> ! {
    use crate::ffi::{
        CE_UTF8, R_BaseEnv, Rf_allocVector, Rf_eval, Rf_lang2, Rf_mkCharCE, Rf_protect, SEXP,
        SEXPTYPE, SexpExt,
    };

    unsafe {
        // Build the class vector: c([custom_class,] "rust_error", "simpleError", "error", "condition")
        let base_classes: &[&std::ffi::CStr] =
            &[c"rust_error", c"simpleError", c"error", c"condition"];
        let class_count = if class.is_some() {
            base_classes.len() + 1
        } else {
            base_classes.len()
        };

        let class_vec = Rf_allocVector(SEXPTYPE::STRSXP, class_count as isize);
        Rf_protect(class_vec);

        let mut idx = 0isize;
        if let Some(custom) = class {
            let custom_cstr = std::ffi::CString::new(custom)
                .unwrap_or_else(|_| std::ffi::CString::new("rust_error").unwrap());
            let custom_charsxp = Rf_mkCharCE(custom_cstr.as_ptr(), CE_UTF8);
            class_vec.set_string_elt(idx, custom_charsxp);
            idx += 1;
        }
        for base in base_classes {
            let charsxp = crate::cached_class::permanent_charsxp(base);
            class_vec.set_string_elt(idx, charsxp);
            idx += 1;
        }

        // Build the message SEXP
        let msg_cstr = std::ffi::CString::new(message)
            .unwrap_or_else(|_| std::ffi::CString::new("<invalid error message>").unwrap());
        let msg_charsxp = Rf_mkCharCE(msg_cstr.as_ptr(), CE_UTF8);
        let msg_sexp = SEXP::scalar_string(msg_charsxp);
        Rf_protect(msg_sexp);

        let call_sexp = call.unwrap_or(SEXP::nil());

        // Build a 2-element named list: list(message = msg, call = call_sexp)
        let err_list = Rf_allocVector(SEXPTYPE::VECSXP, 2);
        Rf_protect(err_list);
        err_list.set_vector_elt(0, msg_sexp);
        err_list.set_vector_elt(1, call_sexp);

        // Set names: c("message", "call")
        let names_vec = Rf_allocVector(SEXPTYPE::STRSXP, 2);
        Rf_protect(names_vec);
        names_vec.set_string_elt(0, crate::cached_class::permanent_charsxp(c"message"));
        names_vec.set_string_elt(1, crate::cached_class::permanent_charsxp(c"call"));
        err_list.set_names(names_vec);

        // Set the class attribute directly (no structure() call needed)
        err_list.set_class(class_vec);

        // Build stop(err_list) as a language object: lang2(stop_sym, err_list)
        // stop() accepts a condition object directly
        let stop_call = Rf_lang2(stop_sym(), err_list);
        Rf_protect(stop_call);

        // Rf_eval(stop_call, R_BaseEnv) longjmps — never returns
        // The protect stack is cleaned up by R's longjmp unwind
        Rf_eval(stop_call, R_BaseEnv);

        // Never reached — Rf_eval(stop(...), ...) always longjmps
        std::hint::unreachable_unchecked()
    }
}

// endregion

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
/// In these contexts there is no R wrapper to inspect a tagged SEXP. Panics
/// are routed through `raise_rust_condition_via_stop` so they still receive
/// `rust_*` class layering (issue #345). Trait-ABI shims use a separate
/// SEXP-returning variant that re-panics at the View boundary.
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
/// Handles [`crate::condition::RCondition`] payloads:
///
/// - `RCondition::Error` — routes through [`raise_rust_condition_via_stop`] which
///   `Rf_eval`s `stop(structure(..., class = c("rust_error", ...)))`. This gives
///   full `rust_*` class layering even in ALTREP callback context where there is
///   no R wrapper to inspect a tagged SEXP (Approach 3 from the issue-345 plan).
///   Custom `class = "..."` from `error!()` is preserved in the class vector.
///
/// - `Warning`, `Message`, `Condition` — convert to a plain R error with a
///   diagnostic message. `warning!()`/`message!()` from ALTREP context cannot
///   suspend execution for non-fatal signals; documented limitation.
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
                    crate::condition::RCondition::Error { message, class } => {
                        // Approach 3 (issue-345): raise via Rf_eval(stop(structure(...)))
                        // so tryCatch(rust_error = h, ...) and tryCatch(my_class = h, ...)
                        // both match. No R wrapper needed.
                        crate::panic_telemetry::fire(message, source);
                        unsafe { raise_rust_condition_via_stop(message, class.as_deref(), call) }
                    }
                    crate::condition::RCondition::Warning { .. }
                    | crate::condition::RCondition::Message { .. }
                    | crate::condition::RCondition::Condition { .. } => {
                        // warning!/message!/condition! cannot be cleanly raised from ALTREP
                        // context (no mechanism to suspend execution for non-fatal signals).
                        // Documented degradation: convert to a plain R error with a fixed
                        // diagnostic, but route through `raise_rust_condition_via_stop` so
                        // the resulting error gets `rust_error` class layering — consistent
                        // with the generic-panic branch a few lines below (issue #366).
                        let msg = "warning!/message!/condition! from ALTREP callback context \
                                   cannot be raised as non-fatal signals; use error!() instead. \
                                   This context has no R wrapper to handle signal restart.";
                        crate::panic_telemetry::fire(msg, source);
                        unsafe { raise_rust_condition_via_stop(msg, None, call) }
                    }
                }
            } else {
                // Generic panic — no class layering, plain error string.
                // Fire telemetry and raise via Approach 3 with rust_error class so
                // tryCatch(rust_error = h, ...) matches even for plain panics.
                let msg = panic_payload_to_string(payload.as_ref());
                crate::panic_telemetry::fire(&msg, source);
                unsafe { raise_rust_condition_via_stop(&msg, None, call) }
            }
            // endregion
        }
    }
}

/// Like [`with_r_unwind_protect`], but returns a tagged error SEXP on Rust panics
/// instead of raising an R error via `Rf_errorcall`.
///
/// **For trait-ABI vtable shims.** Same behaviour as
/// [`with_r_unwind_protect_error_in_r`] except it is intended for use in shim
/// functions that have no R wrapper of their own. The tagged SEXP is returned
/// to the View method wrapper, which calls
/// [`crate::condition::repanic_if_rust_error`] to re-panic with the
/// reconstructed [`crate::condition::RCondition`]. The outer
/// `with_r_unwind_protect_error_in_r` in the consumer's C entry point then
/// catches the re-panic and builds the final tagged SEXP for the consumer's R
/// wrapper.
///
/// R-origin errors (longjmp) still pass through via `R_ContinueUnwind` — the
/// outer `error_in_r` guard will catch them.
///
/// # PROTECT note
///
/// The returned SEXP is unprotected. The View method wrapper must not call any
/// R API functions between receiving it and passing it to
/// `repanic_if_rust_error`. `repanic_if_rust_error` reads the message/kind/class
/// strings immediately and then panics (or returns), so the SEXP does not need
/// protection beyond that window.
pub fn with_r_unwind_protect_shim<F>(f: F) -> SEXP
where
    F: FnOnce() -> SEXP,
{
    match run_r_unwind_protect(f) {
        Ok(result) => result,
        Err(payload) => {
            // region: RCondition recognition — same as error_in_r path
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
                return crate::error_value::make_rust_condition_value(msg, kind, class, None);
            }
            // endregion

            // Generic panic path
            let msg = panic_payload_to_string(payload.as_ref());
            crate::panic_telemetry::fire(&msg, crate::panic_telemetry::PanicSource::UnwindProtect);
            crate::error_value::make_rust_error_value(&msg, "panic", None)
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
