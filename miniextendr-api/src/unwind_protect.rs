//! Safe API for R's `R_UnwindProtect`
//!
//! This module provides [`with_r_unwind_protect`] for handling R errors with Rust cleanup.
//! It automatically runs Rust destructors when R errors occur.
//!
//! **Important**: R uses `longjmp` for error handling, which normally bypasses Rust destructors.
//! Use this API to ensure cleanup happens even when R errors occur.
//!
//! ## When to reach for this
//!
//! - **Calling R APIs that can error from a body you wrote yourself**
//!   (custom ALTREP, custom connection trampoline, hand-rolled FFI shim).
//!   Wrap the R-calling section in [`with_r_unwind_protect`] so Rust
//!   destructors run if R longjmps.
//! - **Inside a [`with_r_unwind_protect`] body it is safe to use `*_unchecked`
//!   variants of the R FFI** — see the [`crate::sys`] module doc. The lint
//!   **MXL301** recognises this as one of the three contexts where bypassing
//!   the main-thread assertion is valid (the other two being ALTREP callbacks
//!   and [`crate::worker::with_r_thread`] bodies).
//!
//! ## You probably don't need this from a `#[miniextendr]` body
//!
//! The proc-macro already wraps every function and method in a guard that
//! converts panics into the tagged-condition transport ([`crate::error_value`]).
//! Returning `Result::Err`, `Option::None`, or calling `panic!()` /
//! [`crate::error!`] / [`crate::warning!`] / [`crate::message!`] is the
//! idiomatic path. Direct [`with_r_unwind_protect`] use inside that body is
//! almost always wrong — you'd be nesting an `R_UnwindProtect` inside another
//! `R_UnwindProtect`, paying the longjmp-leak cost twice (see "Leaks" below).
//!
//! ## Don't use `Rf_error`
//!
//! `Rf_error` and `Rf_errorcall` longjmp directly, skipping every Rust
//! destructor on the stack. The lint **MXL300** forbids them in user code.
//! Panic instead (or call [`crate::error!`]) and the framework raises the
//! corresponding R condition for you.
//!
//! ## Leaks
//!
//! On the R longjmp path (when R unwinds out of the protected body),
//! `with_r_unwind_protect` leaks ~8 bytes (an `RErrorMarker` + `Box` header)
//! because the cleanup handler can't reclaim them via
//! `Box::from_raw`. Regular Rust panics from inside the body don't leak.
//! This is the cost MXL300 is buying off: every direct `Rf_error()` would
//! incur the same leak with no observability.
//!
//! ## Log drain
//!
//! Every call to `with_r_unwind_protect` (and its variants) drains the
//! cross-thread log queue via the crate-private `drain_log_queue_if_available`
//! helper before returning or re-raising an R error. This ensures that records
//! buffered by worker threads are flushed to R's console on every FFI exit —
//! including error paths.
//!
//! ## Cross references
//!
//! - [`crate::worker::with_r_thread`] — routes a closure to R's main thread.
//! - [`crate::ffi_guard`] — unified panic-catching trampoline that consumes
//!   `with_r_unwind_protect_sourced` for ALTREP `RUnwind` mode.
//! - [`crate::error_value`] / [`mod@crate::condition`] — panic → R condition
//!   transport.
use std::{
    any::Any,
    borrow::Cow,
    cell::Cell,
    ffi::c_void,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::OnceLock,
};

// region: raise_rust_condition_via_stop — Approach 3 for ALTREP RUnwind path

/// Raise the R condition for a tagged condition value by evaluating the
/// shared `.miniextendr_raise_condition` helper on it, the helper every
/// generated R wrapper calls on the value its `.Call()` returns.
///
/// This is **Approach 3** from the issue-345 plan: an `Rf_eval` that ends in
/// `stop(structure(...))`, for contexts with no outer R wrapper to inspect a
/// tagged SEXP. It is the only viable option for ALTREP callbacks, which are
/// invoked directly by R's runtime (no `.Call` frame, no R wrapper).
///
/// `tagged_value` builds the value (with [`crate::error_value`]'s
/// constructors). Because the helper raises it, the condition is the one the
/// wrapper raises for the same value, field for field (#1768): `message`,
/// `call`, `kind`, then the `data =` fields spliced with `utils::modifyList`,
/// so a data field named `message`, `call` or `kind` replaces the base field
/// on both transports (#1315); the classes ahead of the `rust_*` layering.
/// The copy evaluated here lives in the base namespace, so the caller
/// resolves `arg_error!`'s class marker in Rust first.
///
/// The value must be of an error kind (`stop()` longjmps, so this function
/// never returns); the helper's warning / message / condition arms return.
///
/// ## Nothing Rust-owned survives the longjmp
///
/// The longjmp skips every Rust frame above R's, so no destructor runs past
/// this point. `tagged_value` consumes what it captures (message, classes,
/// data) and is itself dropped before `Rf_eval`; the SEXPs are rooted with
/// plain `Rf_protect`, whose stack R resets on the jump, rather than a
/// drop guard that would never run.
///
/// ## MXL300 compliance
///
/// This function raises an R error via `Rf_eval(stop(...))`, not via direct
/// `Rf_error`/`Rf_errorcall`. MXL300 does not flag `Rf_eval`.
///
/// # Safety
///
/// Must be called from R's main thread in a context where R longjmps are
/// safe, with no Rust value that needs dropping left on the caller's frames.
/// In practice, always called from `with_r_unwind_protect_sourced` (ALTREP
/// `RUnwind` guards, [`with_r_unwind_protect_or_raise`]) after its guarded
/// closure has returned.
pub(crate) unsafe fn raise_rust_condition_via_stop(
    tagged_value: impl FnOnce() -> SEXP,
    call: Option<SEXP>,
) -> ! {
    use crate::sys::{R_BaseEnv, Rf_eval, Rf_lang3, Rf_protect};

    unsafe {
        let helper = crate::deferred_condition::raise_condition_helper();
        let tagged = Rf_protect(tagged_value());
        let expr = Rf_protect(Rf_lang3(helper, tagged, call.unwrap_or(SEXP::nil())));
        // Longjmps through `stop()`; R pops the protect stack on the way out.
        Rf_eval(expr, R_BaseEnv);
    }
    // Only a non-error tagged value (a bug in the caller) gets here, and
    // unwinding out of an ALTREP callback would cross R's C frames.
    std::process::abort()
}

/// The tagged value of a generic panic, `kind = "panic"`: what
/// [`with_r_unwind_protect`] returns for one.
fn panic_condition_value(message: &str, call: Option<SEXP>) -> SEXP {
    // SAFETY: only called by `raise_rust_condition_via_stop`, on the main thread.
    unsafe {
        crate::error_value::make_rust_condition_value(
            message,
            crate::error_value::kind::PANIC,
            None,
            call,
        )
    }
}

// endregion

use crate::sys::{self, R_ContinueUnwind, R_UnwindProtect_C_unwind};
use crate::{Rboolean, SEXP};

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
            let token = sys::R_MakeUnwindCont();
            sys::R_PreserveObject(token);
            token
        }
    })
}

/// Panic payload whose message is already final (location folded, or
/// deliberately location-free): downstream folds must use it verbatim and
/// must NOT append the current thread's recorded panic location (#1245).
///
/// Produced by `worker::route_to_main_thread`'s re-panic when a `with_r_thread`
/// closure panics on the main thread: the main-thread stringify point already
/// folded the *true* origin location into the message before it crossed back
/// to the worker, so the worker's own re-panic (needed to unwind out of
/// `run_on_worker`) must carry that message forward untouched rather than
/// re-fold its own relay call site on top.
pub(crate) struct PreLocatedPanic(pub(crate) String);

/// Extract a message from a panic payload.
///
/// Handles `&str`, `String`, `&String`, `PreLocatedPanic` and
/// [`RCondition`](crate::condition::RCondition) payloads (the condition's
/// message) consistently. The borrowed variants are returned as
/// `Cow::Borrowed`, so the common `panic!("literal")` case avoids the heap
/// allocation that a `String` return would force. Unrecognised payload types
/// fall back to a `Cow::Borrowed` static string.
///
/// Call `.into_owned()` (or `.to_string()`) at sites that need an owned
/// `String`.
pub fn panic_payload_to_string(payload: &(dyn Any + Send)) -> Cow<'_, str> {
    if let Some(&s) = payload.downcast_ref::<&str>() {
        Cow::Borrowed(s)
    } else if let Some(s) = payload.downcast_ref::<String>() {
        Cow::Borrowed(s.as_str())
    } else if let Some(s) = payload.downcast_ref::<&String>() {
        Cow::Borrowed(s.as_str())
    } else if let Some(pre) = payload.downcast_ref::<PreLocatedPanic>() {
        Cow::Borrowed(pre.0.as_str())
    } else if let Some(condition) = payload.downcast_ref::<crate::condition::RCondition>() {
        Cow::Borrowed(condition.message())
    } else {
        Cow::Borrowed("unknown panic")
    }
}

/// Stringify a panic payload and fold in the Rust source location the panic
/// hook recorded on the *current* thread, producing the final R-facing message.
///
/// Returns `panic_payload_to_string(payload)` with a `\n(at file:line)` suffix
/// when [`crate::backtrace::take_last_panic_location`] has a location for this
/// thread, otherwise the bare message. The location is captured in the process
/// panic hook (`backtrace.rs`), which fires on the panicking thread — so this
/// **must be called on the same thread that ran the panicking closure** (main
/// for main-thread `#[miniextendr]` fns; the worker for worker-dispatched fns).
///
/// Only the *generic panic* path uses this. `error!`/`warning!`/`message!`/
/// `condition!` (and `Result::Err` / `Option::None`) travel the typed
/// `RCondition` / tagged-value branches and are deliberately left byte-for-byte
/// unchanged — they carry no location suffix.
pub(crate) fn panic_message_with_location(payload: &(dyn Any + Send)) -> String {
    let msg = panic_payload_to_string(payload);
    match crate::backtrace::take_last_panic_location() {
        Some((file, line)) => format!("{msg}\n(at {file}:{line})"),
        None => msg.into_owned(),
    }
}

// region: R exits carried as a Rust unwind (#1835)

thread_local! {
    /// The innermost [`Boundary`] on this thread; `0` outside any.
    static BOUNDARY: Cell<u64> = const { Cell::new(0) };
    /// The last boundary id handed out on this thread.
    static LAST_BOUNDARY: Cell<u64> = const { Cell::new(0) };
}

/// A miniextendr boundary that resumes R exits ([`RUnwind`]): the innermost on
/// its thread from [`enter`](Self::enter) until it is dropped or
/// [`leave`](Self::leave)s.
///
/// An exit belongs to the boundary that was innermost when its evaluation
/// started, and only that boundary may resume it. Its jump target is an R
/// context on the C stack, alive only while that boundary's call is: an exit
/// caught with `catch_unwind`, kept, and resumed after the call returned would
/// jump into a frame that no longer exists. [`resume_if_r_unwind`] refuses it.
pub(crate) struct Boundary {
    id: u64,
    outer: u64,
}

impl Boundary {
    /// Become the innermost boundary on this thread.
    #[inline]
    pub(crate) fn enter() -> Self {
        let id = LAST_BOUNDARY.with(|last| {
            let id = last.get() + 1;
            last.set(id);
            id
        });
        let outer = BOUNDARY.with(|current| current.replace(id));
        Boundary { id, outer }
    }

    /// Make the enclosing boundary the innermost again. Call it before leaving
    /// by an R jump, which skips `Drop`.
    #[inline]
    pub(crate) fn leave(&self) {
        BOUNDARY.with(|current| current.set(self.outer));
    }
}

impl Drop for Boundary {
    #[inline]
    fn drop(&mut self) {
        self.leave();
    }
}

/// An R non-local exit that [`call_unwinding`] carries out of its callback
/// as a Rust unwind: an error, a condition caught by an exiting handler set up
/// outside the call (`tryCatch(warning = )`), a restart, an interrupt.
///
/// The unwind runs the destructors of the Rust frames between the evaluation
/// and the boundary that catches it. The boundary ([`run_r_unwind_protect`],
/// the worker's main-thread loop, the FFI guards) then resumes the exit with
/// `R_ContinueUnwind`, and R carries on to the context it was jumping to, with
/// the condition object and call it was jumping with.
///
/// The continuation token is private to one evaluation and preserved until
/// the exit resumes, so R code that a destructor runs during the unwind (a
/// protected call of its own, which reuses the shared token) cannot overwrite
/// the pending jump or let the collector reclaim it (#1507).
pub(crate) struct RUnwind {
    /// `R_MakeUnwindCont()`, preserved: the jump target and the value R was
    /// jumping with.
    token: SEXP,
    /// The [`Boundary`] that was innermost when the evaluation started.
    boundary: u64,
}

impl RUnwind {
    /// Whether `boundary` may resume this exit: the boundary its evaluation
    /// ran in, still on the stack.
    #[inline]
    pub(crate) fn belongs_to(&self, boundary: &Boundary) -> bool {
        self.boundary == boundary.id
    }

    /// Continue the R exit where `R_UnwindProtect` stopped it. Never returns.
    ///
    /// # Safety
    ///
    /// R's main thread, at the boundary the exit [`belongs_to`](Self::belongs_to),
    /// after its [`Boundary::leave`], with no Rust value left to drop between
    /// this call and the R context that owns the frame (the C entry point R
    /// called).
    pub(crate) unsafe fn resume(self) -> ! {
        let token = self.token;
        std::mem::forget(self);
        unsafe {
            // `R_jumpctxt` roots the value it jumps with only after the cleanups
            // of the contexts it passes have run (it sets `R_ReturnedValue`
            // after `R_run_onexits`), and those may allocate. The protection
            // keeps the token, and with it the value, alive until then; the
            // jump resets the protect stack (`R_restore_globals`), which ends it.
            sys::Rf_protect(token);
            sys::R_ReleaseObject(token);
            R_ContinueUnwind(token)
        }
    }

    /// The payload a boundary raises in place of an exit it may not resume
    /// (see [`Boundary`]). The exit is dropped.
    pub(crate) fn stray(self: Box<Self>) -> Box<dyn Any + Send> {
        drop(self);
        Box::new(
            "an R exit (an error, a condition handler's exit or a restart) was caught with \
             `catch_unwind` and resumed outside the call that raised it; it cannot continue",
        )
    }
}

impl Drop for RUnwind {
    /// Reached when something other than the exit's boundary caught the unwind
    /// and dropped it, or the boundary refused it ([`RUnwind::stray`]). The R
    /// exit is abandoned: R carries on from where the evaluation stopped, as
    /// after `R_tryEval`. Release the token, on R's main thread: dropped on
    /// another thread (code that moved the caught payload there), it waits in
    /// [`ABANDONED_OFF_MAIN`] for the next evaluation.
    fn drop(&mut self) {
        if crate::worker::is_r_main_thread() {
            unsafe { sys::R_ReleaseObject(self.token) }
        } else {
            ABANDONED_OFF_MAIN
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(self.token);
            HAS_ABANDONED_OFF_MAIN.store(true, std::sync::atomic::Ordering::Release);
        }
    }
}

/// Tokens of R exits dropped off R's main thread, where `R_ReleaseObject`
/// would race R. [`call_unwinding`] releases them on the main thread.
static ABANDONED_OFF_MAIN: std::sync::Mutex<Vec<SEXP>> = std::sync::Mutex::new(Vec::new());

/// Whether [`ABANDONED_OFF_MAIN`] may hold a token: the evaluation's fast path.
static HAS_ABANDONED_OFF_MAIN: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Release the tokens of exits dropped off the main thread.
///
/// # Safety
///
/// R's main thread.
unsafe fn release_abandoned_off_main() {
    if HAS_ABANDONED_OFF_MAIN.swap(false, std::sync::atomic::Ordering::Acquire) {
        let tokens = std::mem::take(
            &mut *ABANDONED_OFF_MAIN
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        for token in tokens {
            unsafe { sys::R_ReleaseObject(token) };
        }
    }
}

/// Resume `payload` if it carries an R exit ([`RUnwind`]) that belongs to
/// `boundary`; return it otherwise, or [`RUnwind::stray`] for an exit from
/// elsewhere. Conditions deferred since `deferred_mark` belong to the call the
/// exit abandons and are discarded first, as on the R-error path.
///
/// # Safety
///
/// As [`RUnwind::resume`]: R's main thread, at `boundary`, with nothing left
/// to drop above the C entry point.
pub(crate) unsafe fn resume_if_r_unwind(
    payload: Box<dyn Any + Send>,
    deferred_mark: usize,
    boundary: &Boundary,
) -> Box<dyn Any + Send> {
    match payload.downcast::<RUnwind>() {
        Ok(exit) if exit.belongs_to(boundary) => {
            crate::deferred_condition::discard(deferred_mark);
            boundary.leave();
            unsafe { exit.resume() }
        }
        Ok(exit) => exit.stray(),
        Err(payload) => payload,
    }
}

/// Evaluate `expr` in `env` with `Rf_eval`, in the caller's R context: the
/// condition handlers and restarts established around the `.Call()` stay in
/// place, unlike under `R_tryEval`, whose `R_ToplevelExec` empties the handler
/// and restart stacks for the duration. An R exit leaves as [`RUnwind`]
/// ([`call_unwinding`]).
///
/// # Safety
///
/// As [`call_unwinding`]; `expr` and `env` are rooted for the call, `env` an
/// environment. The result is unprotected.
pub(crate) unsafe fn eval_unwinding(expr: SEXP, env: SEXP) -> SEXP {
    struct EvalData {
        expr: SEXP,
        env: SEXP,
    }

    unsafe extern "C-unwind" fn eval(data: *mut c_void) -> SEXP {
        let data = unsafe { &*data.cast::<EvalData>() };
        unsafe { sys::Rf_eval(data.expr, data.env) }
    }

    let mut data = EvalData { expr, env };
    unsafe { call_unwinding(eval, (&raw mut data).cast()) }
}

/// Call `fun(data)` in its own `R_UnwindProtect`, in the caller's R context,
/// and return its value.
///
/// An R exit from `fun` (an error, an exiting handler of the caller's
/// `tryCatch()`, a restart, an interrupt) stops at that frame, never jumping
/// over a Rust frame. Its cleanup turns the exit into a Rust unwind
/// ([`RUnwind`]); the boundary resumes it.
///
/// # Safety
///
/// R's main thread, inside a miniextendr boundary (a `#[miniextendr]` body, a
/// `with_r_unwind_protect*` closure, a guarded callback). `fun` holds nothing
/// to drop across an R jump (it is called from C), and what `data` points to
/// outlives the call. The result is unprotected.
pub(crate) unsafe fn call_unwinding(
    fun: unsafe extern "C-unwind" fn(*mut c_void) -> SEXP,
    data: *mut c_void,
) -> SEXP {
    struct CallData {
        fun: unsafe extern "C-unwind" fn(*mut c_void) -> SEXP,
        data: *mut c_void,
        token: SEXP,
        boundary: u64,
    }

    unsafe extern "C-unwind" fn trampoline(data: *mut c_void) -> SEXP {
        let data = unsafe { &*data.cast::<CallData>() };
        // An R exit from here jumps to `R_UnwindProtect`, over no Rust frame
        // that holds anything to drop.
        unsafe { (data.fun)(data.data) }
    }

    unsafe extern "C-unwind" fn cleanup(data: *mut c_void, jump: Rboolean) {
        if jump != Rboolean::FALSE {
            let data = unsafe { &*data.cast::<CallData>() };
            // A boundary the R code entered and an R jump left skipped its
            // `leave`; this evaluation's boundary is the innermost again.
            BOUNDARY.with(|current| current.set(data.boundary));
            // R stopped its jump at our frame and would continue it after this
            // returns. Carry it out as a Rust unwind instead. `resume_unwind`
            // skips the panic hook: this is no panic.
            std::panic::resume_unwind(Box::new(RUnwind {
                token: data.token,
                boundary: data.boundary,
            }));
        }
    }

    unsafe {
        release_abandoned_off_main();
        let token = sys::R_MakeUnwindCont();
        sys::R_PreserveObject(token);
        let boundary = BOUNDARY.with(Cell::get);
        let mut data = CallData {
            fun,
            data,
            token,
            boundary,
        };
        let value = R_UnwindProtect_C_unwind(
            Some(trampoline),
            (&raw mut data).cast(),
            Some(cleanup),
            (&raw mut data).cast(),
            token,
        );
        BOUNDARY.with(|current| current.set(boundary));
        sys::R_ReleaseObject(token);
        value
    }
}

// endregion

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
                crate::SEXP::nil()
            }
            Err(payload) => {
                data.panic_payload = Some(payload);
                crate::SEXP::nil()
            }
        }
    }

    unsafe extern "C-unwind" fn cleanup_handler(_data: *mut c_void, jump: Rboolean) {
        if jump != Rboolean::FALSE {
            // R is about to longjmp - trigger a Rust panic so we can unwind properly
            std::panic::panic_any(RErrorMarker);
        }
    }

    let deferred_mark = crate::deferred_condition::mark();
    let boundary = Boundary::enter();
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
                    // An R exit carried out of `call_unwinding` (#1835): the Rust
                    // frames it crossed have dropped their values; continue it.
                    let payload = resume_if_r_unwind(payload, deferred_mark, &boundary);
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
                    // No signalling while resuming an R error or exiting handler.
                    // Conditions from the abandoned call must not survive it.
                    crate::deferred_condition::discard(deferred_mark);
                    boundary.leave();
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

/// Execute a closure with R unwind protection, raising any Rust panic as the R
/// error a generated wrapper would raise for it.
///
/// If the closure panics, the panic is caught and converted to an R error
/// (longjmp) with `rust_*` class layering. If R raises an error (longjmp), all
/// Rust RAII resources are properly dropped before R continues unwinding.
///
/// **This is NOT the user-facing path for `#[miniextendr]` functions.** That
/// path is [`with_r_unwind_protect`], which returns a tagged SEXP instead of
/// longjmping (the macro-generated R wrapper raises the structured condition).
///
/// This raising-variant exists for guard sites that have no R wrapper between
/// them and R's runtime:
/// - ALTREP `RUnwind` guard callbacks (via the crate-private
///   `with_r_unwind_protect_sourced`)
/// - FFI guard tests / benchmarks exercising the raw `R_UnwindProtect` mechanism
///
/// In those contexts there is no consumer-side R wrapper to inspect a tagged
/// SEXP. Panics are routed through `raise_rust_condition_via_stop` so they
/// still receive `rust_*` class layering (issue #345), the same `kind` and
/// the same fields as the wrapper's condition (#1768). Trait-ABI shims use a
/// separate SEXP-returning variant ([`with_r_unwind_protect_shim`]) that
/// re-panics at the View boundary.
///
/// # Arguments
///
/// * `f` - The closure to execute
/// * `call` - Optional R call SEXP for better error messages
///
/// Deferred conditions are signalled before the outcome is returned or raised.
/// Any R objects held by the generic return value must remain rooted during
/// signalling. Return an [`crate::OwnedProtect`] for a freshly allocated SEXP,
/// then call its `get()` after this guard returns. ALTREP's SEXP trampolines do
/// this automatically. An R-origin error discards this call's queued conditions.
pub fn with_r_unwind_protect_or_raise<F, R>(f: F, call: Option<SEXP>) -> R
where
    F: FnOnce() -> R,
{
    with_r_unwind_protect_sourced(f, call, crate::panic_telemetry::PanicSource::UnwindProtect)
}

/// Like [`with_r_unwind_protect_or_raise`], but reports panics with a custom
/// `PanicSource`.
///
/// Used by `guarded_altrep_call` so that panics inside ALTREP callbacks with
/// `AltrepGuard::RUnwind` are still attributed to `PanicSource::Altrep`.
///
/// Every outcome is raised through [`raise_rust_condition_via_stop`], which
/// hands the tagged value [`with_r_unwind_protect`] would return to the R
/// helper the generated wrappers call, so the condition is the wrapper's, with
/// its `kind` (#1768). This works even in ALTREP callback context, where there
/// is no R wrapper to inspect a tagged SEXP (Approach 3 from the issue-345
/// plan):
///
/// - `RCondition::Error` (`error!()`) — `kind = "error"`, its classes ahead of
///   the `rust_*` layering, its `data =` fields.
///
/// - `RCondition::Conversion` (`arg_error!`) — `kind = "conversion"`, with the
///   crate-class marker replaced in Rust by the classes `miniextendr_init!`
///   registered: no generated R is there to resolve it.
///
/// - A generic panic — `kind = "panic"`, with the `(at file:line)` suffix.
///
/// - `Warning`, `Message`, `Condition` — a plain R error with a diagnostic
///   message and `kind = "panic"`. `warning!()`/`message!()` from ALTREP
///   context cannot suspend execution for non-fatal signals; documented
///   limitation.
pub(crate) fn with_r_unwind_protect_sourced<F, R>(
    f: F,
    call: Option<SEXP>,
    source: crate::panic_telemetry::PanicSource,
) -> R
where
    F: FnOnce() -> R,
{
    let deferred_mark = crate::deferred_condition::mark();
    let outcome = run_r_unwind_protect(f);
    // Preserve the original panic site before condition handlers can re-enter
    // Rust and overwrite the panic hook's thread-local location.
    let panic_message = outcome.as_ref().err().and_then(|payload| {
        (!payload.is::<crate::condition::RCondition>())
            .then(|| panic_message_with_location(payload.as_ref()))
    });
    let queued = crate::deferred_condition::take_pending(deferred_mark);
    let (outcome, panic_message) = if queued.is_empty() {
        (outcome, panic_message)
    } else {
        // Move both the outcome and saved panic text INTO this guard: an
        // exiting R handler must drop all their Rust resources too.
        // SEXP-bearing outcomes must own their roots.
        match run_r_unwind_protect(move || {
            unsafe { crate::deferred_condition::signal_now(queued, call) };
            (outcome, panic_message)
        }) {
            Ok(outcome) => outcome,
            Err(payload) => (Err(payload), None),
        }
    };
    match outcome {
        Ok(result) => result,
        Err(payload) => {
            // region: RCondition recognition for the raising-variant path
            if payload.is::<crate::condition::RCondition>() {
                // Take ownership so the payload's parts move into the tagged
                // value without cloning — same idiom as `with_r_unwind_protect_shim`.
                let cond = *payload
                    .downcast::<crate::condition::RCondition>()
                    .expect("checked is::<RCondition> above");
                let error_kind = matches!(
                    cond,
                    crate::condition::RCondition::Error { .. }
                        | crate::condition::RCondition::Conversion { .. }
                );
                // The kind is the one the tagged path writes for this payload.
                let (kind, mut parts) = cond.into_parts();
                if error_kind {
                    // Approach 3 (issue-345): the helper the wrappers use raises
                    // the value, so `tryCatch(rust_error = h, ...)`,
                    // `tryCatch(my_class = h, ...)`, `e$kind` and the `data`
                    // fields (issue #996 path 2) all match the wrapper's
                    // condition. That copy of the helper cannot resolve an
                    // `arg_error!`'s class marker (#1740): Rust puts the
                    // package's registered classes in its place.
                    parts.class = crate::condition::resolve_conversion_error_class_marker(
                        parts.class,
                        crate::condition::crate_conversion_error_class(),
                    );
                    crate::panic_telemetry::fire(&parts.message, source);
                    unsafe {
                        raise_rust_condition_via_stop(
                            move || crate::error_value::condition_parts_value(kind, parts, call),
                            call,
                        )
                    }
                } else {
                    // warning!/message!/condition! cannot be cleanly raised from ALTREP
                    // context (no mechanism to suspend execution for non-fatal signals).
                    // Documented degradation: a plain R error with a fixed diagnostic,
                    // raised like the generic-panic branch below (`kind = "panic"`,
                    // `rust_error` class layering, issue #366): an error reporting the
                    // original kind would claim to be a warning. The classes and data
                    // fields are dropped along with the message; the call choice is
                    // kept (`call = none` drops the guard's call).
                    let call = parts.call.apply(call);
                    drop(parts);
                    let msg = "warning!/message!/condition! from ALTREP callback context \
                               cannot be raised as non-fatal signals; use error!() instead. \
                               This context has no R wrapper to handle signal restart.";
                    crate::panic_telemetry::fire(msg, source);
                    unsafe {
                        raise_rust_condition_via_stop(|| panic_condition_value(msg, call), call)
                    }
                }
            } else {
                // Generic panic — no class layering, plain error string plus the
                // `(at file:line)` suffix folded from the panic hook (this branch
                // runs on the panicking thread for the ALTREP/FFI-guard path).
                // Fire telemetry and raise via Approach 3 with rust_error class so
                // tryCatch(rust_error = h, ...) matches even for plain panics.
                let msg =
                    panic_message.unwrap_or_else(|| panic_message_with_location(payload.as_ref()));
                // The raise longjmps over this frame: free the payload first.
                drop(payload);
                crate::panic_telemetry::fire(&msg, source);
                unsafe {
                    raise_rust_condition_via_stop(move || panic_condition_value(&msg, call), call)
                }
            }
            // endregion
        }
    }
}

/// Like [`with_r_unwind_protect`], but tailored for trait-ABI vtable shims.
///
/// Same tagged-SEXP behaviour as [`with_r_unwind_protect`], but intended for
/// shim functions that have no R wrapper of their own. The tagged SEXP is
/// returned to the View method wrapper, which calls
/// [`crate::condition::repanic_if_rust_error`] to re-panic with the
/// reconstructed [`crate::condition::RCondition`]. The outer
/// `with_r_unwind_protect` in the consumer's C entry point then catches the
/// re-panic and builds the final tagged SEXP for the consumer's R wrapper.
///
/// R-origin errors (longjmp) still pass through via `R_ContinueUnwind` — the
/// outer guard will catch them.
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
    // Conditions queued with `defer_warning` & co. inside the shim are signalled
    // at this boundary: the consumer's queue is a different static (every
    // package links its own copy of this crate), so nothing may be left behind.
    let deferred_mark = crate::deferred_condition::mark();
    let result = match run_r_unwind_protect(f) {
        Ok(result) => result,
        Err(payload) => {
            // region: RCondition recognition — same as the tagged-SEXP path
            if payload.is::<crate::condition::RCondition>() {
                // Take ownership of the payload so the `data` Vec can be moved
                // into `make_rust_condition_value` (consumed when materialised).
                // A call-less payload writes the "no call" marker, which
                // `from_tagged_sexp` reads back on the consumer's side.
                let cond = *payload
                    .downcast::<crate::condition::RCondition>()
                    .expect("checked is::<RCondition> above");
                // SAFETY: on the R main thread inside R_UnwindProtect.
                unsafe { crate::error_value::rust_condition_value(cond, None) }
            } else {
                // Generic panic path — fold the hook-captured `(at file:line)` into
                // the message (this shim runs on the panicking thread).
                let msg = panic_message_with_location(payload.as_ref());
                crate::panic_telemetry::fire(
                    &msg,
                    crate::panic_telemetry::PanicSource::UnwindProtect,
                );
                // SAFETY: on the R main thread inside R_UnwindProtect.
                unsafe {
                    crate::error_value::make_rust_condition_value(
                        &msg,
                        crate::error_value::kind::PANIC,
                        None,
                        None,
                    )
                }
            }
            // endregion
        }
    };
    // SAFETY: vtable shims run on R's main thread, inside the consumer's `.Call`.
    unsafe { crate::deferred_condition::finish(deferred_mark, result, None) }
}

/// Run a closure under `R_UnwindProtect`, returning a tagged condition SEXP on
/// Rust panics instead of raising an R error.
///
/// This is **the** transport for all `#[miniextendr]` functions and methods.
/// The returned error/condition SEXP is inspected by the generated R wrapper
/// which raises a proper R condition past the Rust boundary, with `rust_*`
/// class layering.
///
/// Recognises [`crate::condition::RCondition`] payloads (from `error!()`,
/// `warning!()`, `message!()`, `condition!()`) before falling through to the
/// generic panic→string path.
///
/// R-origin errors (longjmp) still pass through via `R_ContinueUnwind`.
///
/// For guard sites that have no R wrapper to inspect a tagged SEXP (ALTREP
/// `RUnwind` callbacks, FFI guard tests) see [`with_r_unwind_protect_or_raise`];
/// for trait-ABI vtable shims see [`with_r_unwind_protect_shim`].
pub fn with_r_unwind_protect<F>(f: F, call: Option<SEXP>) -> SEXP
where
    F: FnOnce() -> SEXP,
{
    match run_r_unwind_protect(f) {
        Ok(result) => result,
        Err(payload) => {
            // region: RCondition recognition — must come before generic panic path
            if payload.is::<crate::condition::RCondition>() {
                // Take ownership so the `data` payload can be moved into
                // `make_rust_condition_value` (consumed during materialisation).
                let cond = *payload
                    .downcast::<crate::condition::RCondition>()
                    .expect("checked is::<RCondition> above");
                // No panic telemetry for user-raised conditions — they are intentional.
                // SAFETY: on the R main thread inside R_UnwindProtect.
                return unsafe { crate::error_value::rust_condition_value(cond, call) };
            }
            // endregion

            // Generic panic path — the primary user-facing route for
            // `#[miniextendr]` fns running on the main thread. Fold the
            // hook-captured `(at file:line)` into the message. (The RCondition
            // branch above is deliberately untouched: error!/warning!/message!/
            // condition! and Err/None carry no location suffix.)
            let msg = panic_message_with_location(payload.as_ref());
            crate::panic_telemetry::fire(&msg, crate::panic_telemetry::PanicSource::UnwindProtect);
            // SAFETY: on the R main thread inside R_UnwindProtect.
            unsafe {
                crate::error_value::make_rust_condition_value(
                    &msg,
                    crate::error_value::kind::PANIC,
                    None,
                    call,
                )
            }
        }
    }
}
