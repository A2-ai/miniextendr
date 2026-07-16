//! Worker thread infrastructure for safe Rust-R FFI.
//!
//! ## Why a worker thread at all?
//!
//! R's error handling uses `longjmp`, which skips Rust destructors and leaks
//! resources. Generated wrappers therefore establish a destructor-safe error
//! boundary. The default boundary runs inline on R's main thread inside
//! `R_UnwindProtect`; `#[miniextendr(worker)]` (or the `worker-default`
//! feature) instead dispatches the Rust body to a dedicated worker.
//!
//! On the worker-dispatch path, **user code is off the R main thread**.
//! Merely enabling the `worker-thread` infrastructure feature does not select
//! that path. Anything in an opted-in worker body that calls R's C API
//! (allocating `SEXP`s, walking attributes, accessing `INTEGER(x)`) must cross
//! back to main via [`with_r_thread`].
//!
//! ## Public API
//!
//! - [`with_r_thread`] — Execute a closure on R's main thread. This is the
//!   bridge: call it from inside a `#[miniextendr]` body whenever you need
//!   to touch the R FFI.
//! - [`is_r_main_thread`] — Check if the current thread is R's main thread.
//! - [`Sendable`] — `#[doc(hidden)]` wrapper used by the macros to ferry
//!   `SEXP` (and other `!Send` types) across the worker channel. The author
//!   asserts the value is only consumed on the main thread.
//!
//! ## Tradeoffs
//!
//! - **Default to checked FFI variants** (`Rf_allocVector`, `INTEGER`, …) so
//!   an active worker call routes correctly and an arbitrary off-thread call
//!   fails instead of reaching R.
//! - **Inside a [`with_r_thread`] body, the check is redundant** — the
//!   `*_unchecked` variants in [`crate::sys`] are safe to call there
//!   (recognised by the lint **MXL301**, alongside ALTREP callbacks and
//!   [`crate::unwind_protect::with_r_unwind_protect`] bodies).
//! - **Don't raise R errors directly** from worker-thread code. `Rf_error`
//!   would longjmp through Rust frames on the wrong thread. Panic instead;
//!   the framework converts the panic into a structured R condition (see
//!   [`crate::error_value`]). The lint **MXL300** enforces this.
//!
//! ## Feature gate: `worker-thread`
//!
//! Without the `worker-thread` cargo feature, all calls execute inline on
//! R's main thread:
//! - `with_r_thread(f)` runs `f()` directly (panics if not on main thread)
//! - `run_on_worker(f)` runs `f()` directly, returns `Ok(f())`
//!
//! With the feature enabled, a dedicated worker thread is spawned at init time.
//! `with_r_thread` routes calls from the worker back to main, and `run_on_worker`
//! dispatches closures to the worker with bidirectional communication. The
//! worker has a 16 MB stack — keep `proptest!` invocations on it modest (see
//! the project `CLAUDE.md`).
//!
//! ## Initialization
//!
//! [`miniextendr_runtime_init`] must be called from R's main thread before any
//! R FFI APIs. Typically done in `R_init_<pkgname>()`.
//!
//! ## Cross references
//!
//! - [`crate::unwind_protect::with_r_unwind_protect`] — catch R errors with
//!   Rust cleanup; sibling to `with_r_thread`.
//! - [`crate::sys`] — checked vs `*_unchecked` FFI surface.
//! - [`crate::ffi_guard`] — guard taxonomy across boundaries.

use std::sync::OnceLock;
use std::thread;

use crate::SEXP;
use crate::sys::{self};

static R_MAIN_THREAD_ID: OnceLock<thread::ThreadId> = OnceLock::new();

// region: Public API

/// Wrapper to mark values as Send for main-thread routing.
///
/// Only safe if the value is not accessed on the worker thread and is
/// used exclusively on the main thread.
#[doc(hidden)]
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Sendable<T>(pub T);

unsafe impl<T> Send for Sendable<T> {}

/// Check if the current thread is R's main thread.
///
/// Returns `true` if called from the main R thread, `false` otherwise.
/// Before `miniextendr_runtime_init()` is called, always returns `false`.
#[inline(always)]
pub fn is_r_main_thread() -> bool {
    R_MAIN_THREAD_ID
        .get()
        .map(|&id| id == std::thread::current().id())
        .unwrap_or(false)
}

/// Execute a closure on R's main thread, returning the result.
///
/// This function executes successfully in two contexts:
/// - From the main thread: executes the closure directly (re-entrant)
/// - From the worker thread (during `run_on_worker`): sends the work to
///   the main thread and blocks until completion
///
/// Calls from arbitrary spawned or Rayon threads panic; there is no active
/// main-thread event loop to receive their work.
///
/// The "main thread" the closure runs on is whichever thread called
/// [`run_on_worker`]. Per the [`run_on_worker`] contract, that must be
/// the R main thread (the thread that ran `miniextendr_runtime_init()`);
/// otherwise R API calls inside the closure happen on the wrong thread.
///
/// # Panics
///
/// - If `miniextendr_runtime_init()` hasn't been called yet
/// - If called from a non-main thread without the `worker-thread` feature
/// - If called from a non-main thread outside of a `run_on_worker` context
///   (even with the `worker-thread` feature)
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::with_r_thread;
///
/// // From worker thread, safely call R APIs:
/// let sexp = with_r_thread(|| {
///     // This runs on R's main thread
///     SEXP::nil()
/// });
/// ```
pub fn with_r_thread<F, R>(f: F) -> R
where
    F: FnOnce() -> R + 'static,
    R: Send + 'static,
{
    assert_runtime_initialized();

    if is_r_main_thread() {
        return f();
    }

    // Not on main thread — need worker-thread routing (absent on wasm, where
    // there is only the single R thread, so this branch is unreachable there).
    #[cfg(not(all(feature = "worker-thread", not(target_family = "wasm"))))]
    {
        panic!(
            "with_r_thread called from a non-main thread without the `worker-thread` feature.\n\
             \n\
             Without `worker-thread`, R API calls can only happen on the R main thread.\n\
             Either:\n\
             - Enable the `worker-thread` cargo feature to route calls from background threads, or\n\
             - Ensure this code only runs on the R main thread."
        );
    }

    #[cfg(all(feature = "worker-thread", not(target_family = "wasm")))]
    {
        worker_channel::route_to_main_thread(f)
    }
}
// endregion

// region: #[doc(hidden)] items for macro-generated code

/// Raise an R error from a panic message. Does not return.
///
/// If `call` is `Some(sexp)`, uses `Rf_errorcall` to include call context.
#[doc(hidden)]
pub fn panic_message_to_r_error(msg: String, call: Option<SEXP>) -> ! {
    let c_msg = std::ffi::CString::new(msg)
        .unwrap_or_else(|_| std::ffi::CString::new("Rust panic (invalid message)").unwrap());
    unsafe {
        match call {
            Some(call) => sys::Rf_errorcall_unchecked(call, c"%s".as_ptr(), c_msg.as_ptr()),
            None => sys::Rf_error_unchecked(c"%s".as_ptr(), c_msg.as_ptr()),
        }
    }
}

/// Run a closure on the worker thread with proper cleanup on panic.
///
/// Returns `Ok(T)` on success, `Err(String)` if the closure panicked.
/// The caller handles the error (either tagged error value or `Rf_errorcall`).
///
/// Without the `worker-thread` feature, runs inline on the current thread.
///
/// # Precondition: caller must be the R main thread
///
/// The main-thread event loop that drives [`with_r_thread`] callbacks
/// runs on whatever thread invokes `run_on_worker`. R API calls fired
/// from inside the closure are routed back to *that* thread — so if
/// the caller isn't the R main thread, the callbacks land on the
/// wrong thread silently.
///
/// In normal usage this contract is satisfied automatically:
/// `#[miniextendr]` entry points are reached via `.Call`, which is
/// always on R's main thread. Calling `run_on_worker` from a
/// Rust-spawned thread is a programming error.
///
/// In debug builds the precondition is asserted via `debug_assert!`.
/// Release builds skip the check (one fewer atomic load per dispatch);
/// the `.Call` invariant is relied on instead.
#[doc(hidden)]
pub fn run_on_worker<F, T>(f: F) -> Result<T, String>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    // On wasm there is no worker thread even when the feature is enabled
    // (R-on-wasm is single-threaded; emscripten has no usable pthreads), so we
    // run inline — identical to a non-`worker-thread` build. The feature stays
    // *enabled* on wasm so worker-gated routines still compile and the
    // pre-generated `wasm_registry.rs` (built with the feature) has no dangling
    // entries. See `worker_active` reasoning at the top of the worker functions.
    //
    // The inline path must uphold the same panic contract as the worker path:
    // a panicking closure yields `Err(message)`, never an unwind out of
    // `run_on_worker` (on the worker path the channel boundary guarantees
    // this). Without the catch, a panic here unwinds out of raw
    // `extern "C-unwind"` `.Call` entry points that rely on the Err contract —
    // under webR's wasm-exception unwinding that escapes R entirely and
    // reaches the JS host as an uncaught `WebAssembly.Exception`, killing the
    // session (observed in tier-3 via `unsafe_C_test_worker_panic_simple`).
    // The message rules mirror `worker_channel::fold_panic_message`:
    // `RCondition` payloads stringify verbatim, generic panics fold in the
    // recorded `(at file:line)` location — same thread, so the take-once
    // location slot is valid here too.
    #[cfg(not(all(feature = "worker-thread", not(target_family = "wasm"))))]
    {
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
            Ok(val) => Ok(val),
            Err(payload) => {
                let msg = if payload.is::<crate::condition::RCondition>() {
                    crate::unwind_protect::panic_payload_to_string(payload.as_ref()).into_owned()
                } else {
                    crate::unwind_protect::panic_message_with_location(payload.as_ref())
                };
                crate::panic_telemetry::fire(&msg, crate::panic_telemetry::PanicSource::Worker);
                Err(msg)
            }
        }
    }

    #[cfg(all(feature = "worker-thread", not(target_family = "wasm")))]
    {
        let result = worker_channel::dispatch_to_worker(f);
        if let Err(ref msg) = result {
            crate::panic_telemetry::fire(msg, crate::panic_telemetry::PanicSource::Worker);
        }
        result
    }
}

/// Initialize the miniextendr runtime.
///
/// Records the main thread ID and (with `worker-thread`) spawns the worker.
/// Must be called from R's main thread, typically from `R_init_<pkgname>`.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub extern "C-unwind" fn miniextendr_runtime_init() {
    static RUN_ONCE: std::sync::Once = std::sync::Once::new();

    // wasm: never spawn a worker (single-threaded; spawning traps at load).
    // Falls through to the inline init path below even with the feature on.
    #[cfg(all(feature = "worker-thread", not(target_family = "wasm")))]
    {
        RUN_ONCE.call_once_force(|x| {
            if x.is_poisoned() {
                eprintln!(
                    "warning: miniextendr worker init is retrying after a previous failed attempt"
                );
            }

            let current_id = std::thread::current().id();
            if let Some(&existing_id) = R_MAIN_THREAD_ID.get() {
                if existing_id != current_id {
                    panic!(
                        "miniextendr_runtime_init called from thread {:?}, but R_MAIN_THREAD_ID \
                         was already set to {:?}. This indicates incorrect initialization order.",
                        current_id, existing_id
                    );
                }
            } else {
                let _ = R_MAIN_THREAD_ID.set(current_id);
            }

            worker_channel::init_worker();

            // NB: no libc `atexit` registration.
            //
            // `atexit` stores a function pointer into the DLL's code. If the
            // package is unloaded (e.g. via `library.dynam.unload` / dyn.unload
            // / devtools::load_all(reset = TRUE)) before libc runs its atexit
            // registry at process exit, the handler jumps to an unmapped
            // address and tears down the process's SEH state on Windows —
            // which manifests as "fatal runtime error: failed to initiate
            // panic, error 5" in the next DLL that tries to unwind. #277.
            //
            // The normal path (package unload → `R_unload_<pkg>` →
            // `miniextendr_runtime_shutdown`) already joins the worker
            // cleanly. The abnormal path (process exit without unload, e.g.
            // `q("no")`) relies on the OS to reap the worker thread, which
            // it does — we don't need graceful shutdown for a dying process.
        });
    }

    #[cfg(not(all(feature = "worker-thread", not(target_family = "wasm"))))]
    {
        RUN_ONCE.call_once(|| {
            let _ = R_MAIN_THREAD_ID.set(std::thread::current().id());
        });
    }
}

/// Shut down the miniextendr worker thread synchronously.
///
/// Called from `R_unload_<pkg>` (generated by `miniextendr_init!`). Sends a
/// `Shutdown` message to the worker, drops the sender, and blocks on
/// `JoinHandle::join()` until the worker thread has fully exited. Must block:
/// `library.dynam.unload` unmaps the DLL's code pages as soon as this returns,
/// and a still-live worker would resume execution in freed memory (see #277).
///
/// Idempotent. After the first call, the join handle is taken and subsequent
/// calls are no-ops. Safe to call from any thread, though R only ever calls it
/// from the main thread.
///
/// Additionally uninstalls the process panic hook that this DLL registered
/// (also in DLL code — see `backtrace::miniextendr_panic_hook_uninstall`).
///
/// Without the `worker-thread` feature this is (mostly) a no-op: only the
/// panic hook uninstall runs.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub extern "C-unwind" fn miniextendr_runtime_shutdown() {
    // wasm never spawns a worker, so there is nothing to join (and the channel
    // was never initialised) — skip shutdown there even with the feature on.
    #[cfg(all(feature = "worker-thread", not(target_family = "wasm")))]
    {
        worker_channel::shutdown();
    }
    crate::backtrace::miniextendr_panic_hook_uninstall();
}
// endregion

// region: pub(crate) internals

/// Check whether the current thread has a worker routing context.
pub(crate) fn has_worker_context() -> bool {
    #[cfg(all(feature = "worker-thread", not(target_family = "wasm")))]
    {
        worker_channel::has_context()
    }
    #[cfg(not(all(feature = "worker-thread", not(target_family = "wasm"))))]
    {
        false
    }
}

/// Panic if the runtime hasn't been initialized.
fn assert_runtime_initialized() {
    if R_MAIN_THREAD_ID.get().is_none() {
        panic!(
            "miniextendr_runtime_init() must be called before using R FFI APIs.\n\
             \n\
             This is typically done in R_init_<pkgname>() via:\n\
             \n\
             void R_init_pkgname(DllInfo *dll) {{\n\
             miniextendr_runtime_init();\n\
             R_registerRoutines(dll, NULL, CallEntries, NULL, NULL);\n\
             }}\n\
             \n\
             If you're embedding R in Rust, call miniextendr_runtime_init() from the main thread \
             before any R API calls."
        );
    }
}
// endregion

// region: Worker channel infrastructure (only with worker-thread feature)
//
// Compiled only on non-wasm worker-thread builds. On wasm the feature stays
// enabled (so worker-gated routines compile and `wasm_registry.rs` matches),
// but every caller above takes the inline path, so this module is omitted to
// avoid dead-code and to keep the single-threaded wasm side-module clean.

#[cfg(all(feature = "worker-thread", not(target_family = "wasm")))]
mod worker_channel {
    use std::any::Any;
    use std::cell::RefCell;
    use std::panic::{AssertUnwindSafe, catch_unwind};
    use std::sync::Mutex;
    use std::sync::mpsc::{self, Receiver, SyncSender};
    use std::thread;

    use super::Sendable;
    use crate::sys;
    use crate::{Rboolean, SEXP};

    type AnyJob = Box<dyn FnOnce() + Send>;

    /// Tagged messages on the main→worker channel.
    ///
    /// Plain `Box<AnyJob>` transport with an atomic shutdown flag + `recv_timeout`
    /// polling used to sit here. That shape left the worker asleep in
    /// `recv_timeout` when `R_unload_<pkg>` fired, so `library.dynam.unload`
    /// unmapped the DLL's code pages while the worker was still about to wake
    /// up inside them — producing the "failed to initiate panic, error 5"
    /// SEH corruption documented in #277.
    ///
    /// `Shutdown` is a proper message instead: the worker blocks on `recv()`
    /// (no timeout, no polling), `shutdown()` delivers the message, and
    /// `recv()` returns immediately. Combined with dropping the sender and
    /// a blocking `JoinHandle::join()`, this makes `R_unload_<pkg>`
    /// synchronous — the DLL can't unmap until the worker thread has truly
    /// exited.
    enum WorkerMsg {
        Job(AnyJob),
        Shutdown,
    }

    /// Single place that owns the worker's lifetime.
    ///
    /// `Mutex<Option<...>>` (rather than `OnceLock`) because `shutdown()`
    /// needs to `.take()` both the sender (to drop it, closing the channel as
    /// a second-path wake-up) and the join handle (to `.join()` it). After
    /// `shutdown()`, `dispatch_to_worker` sees `None` and returns a
    /// structured "worker shut down" error instead of relying on the old
    /// `send` returning `SendError`.
    struct WorkerState {
        tx: SyncSender<WorkerMsg>,
        handle: thread::JoinHandle<()>,
    }

    static WORKER: Mutex<Option<WorkerState>> = Mutex::new(None);

    /// Shut the worker down synchronously.
    ///
    /// Send `Shutdown`, drop the sender (so `recv()` returns `Err` even if
    /// the `Shutdown` message is somehow missed — defense in depth), then
    /// block on `JoinHandle::join()`. No timeout: if the worker wedges, we
    /// want the hang to surface directly rather than mask it with an
    /// arbitrary deadline that then races DLL unmap. Idempotent — after the
    /// first call, `WORKER` is `None` and the function is a no-op.
    pub(super) fn shutdown() {
        let Some(state) = WORKER.lock().unwrap().take() else {
            return;
        };
        // If the worker already exited (e.g. it panicked), `send` errors —
        // the drop below is what matters in that case.
        let _ = state.tx.send(WorkerMsg::Shutdown);
        drop(state.tx);
        // We're being called from `R_unload_<pkg>`, which is `extern
        // "C-unwind"` — so a panic here would unwind through R's dyn.unload
        // handler. If the worker itself panicked in a way we can't catch,
        // logging the payload and continuing is safer than re-raising
        // across the R FFI boundary. We still join (unwrap the Err) so the
        // handle is consumed and OS thread resources are released.
        if let Err(payload) = state.handle.join() {
            let msg = crate::unwind_protect::panic_payload_to_string(&*payload);
            eprintln!("miniextendr: worker thread panicked during shutdown: {msg}");
        }
    }

    // Type-erased main thread work: closure that returns boxed result
    type MainThreadWork = Sendable<Box<dyn FnOnce() -> Box<dyn Any + Send> + 'static>>;

    // Response from main thread: Ok(result) or Err(panic_message)
    type MainThreadResponse = Result<Box<dyn Any + Send>, String>;

    /// Messages from worker to main thread
    enum WorkerMessage<T> {
        /// Worker requests main thread to execute some work, then send response back
        WorkRequest(MainThreadWork),
        /// Worker is done, here's the final result
        Done(Result<T, String>),
    }

    type TypeErasedWorkerMessage = WorkerMessage<Box<dyn Any + Send>>;
    type WorkerToMainSender = RefCell<Option<SyncSender<TypeErasedWorkerMessage>>>;
    type MainResponseReceiver = RefCell<Option<Receiver<MainThreadResponse>>>;

    // Thread-local channels for worker -> main communication during run_on_worker
    thread_local! {
        static WORKER_TO_MAIN_TX: WorkerToMainSender = const { RefCell::new(None) };
        static MAIN_RESPONSE_RX: MainResponseReceiver = const { RefCell::new(None) };
    }

    pub(super) fn has_context() -> bool {
        WORKER_TO_MAIN_TX.with(|tx_cell| tx_cell.borrow().is_some())
    }

    /// Route a closure from the worker thread to the main thread.
    pub(super) fn route_to_main_thread<F, R>(f: F) -> R
    where
        F: FnOnce() -> R + 'static,
        R: Send + 'static,
    {
        WORKER_TO_MAIN_TX.with(|tx_cell| {
            let tx = tx_cell
                .borrow()
                .as_ref()
                .expect("`with_r_thread` called outside of `run_on_worker` context")
                .clone();

            let work: MainThreadWork =
                Sendable(Box::new(move || Box::new(f()) as Box<dyn Any + Send>));

            tx.send(WorkerMessage::WorkRequest(work))
                .expect("main thread channel closed");
        });

        MAIN_RESPONSE_RX.with(|rx_cell| {
            let rx = rx_cell.borrow();
            let rx = rx.as_ref().expect("response channel not set");
            let response = rx.recv().expect("main thread response channel closed");
            match response {
                Ok(boxed) => *boxed
                    .downcast::<R>()
                    .expect("type mismatch in `with_r_thread` response"),
                Err(panic_msg) => {
                    // `panic_msg` is already final: the main-thread stringify
                    // point in `dispatch_to_worker` (below) folded the *true*
                    // origin location into it before sending it back over the
                    // channel. Re-panicking here (needed to unwind out of
                    // `run_on_worker`) must NOT let this thread's panic hook
                    // fold ITS OWN call site (this line, in `worker.rs`) on
                    // top — wrap in `PreLocatedPanic` so the catch in
                    // `dispatch_to_worker` uses the message verbatim (#1245).
                    std::panic::panic_any(crate::unwind_protect::PreLocatedPanic(format!(
                        "panic in `with_r_thread`: {panic_msg}"
                    )))
                }
            }
        })
    }

    /// Stringify a caught panic payload for cross-thread transport, folding in
    /// the *current* thread's recorded panic location for generic panics.
    ///
    /// User conditions (`error!`/`warning!`/`message!`/`condition!`) travel as
    /// `RCondition` payloads and must stay location-free — stringified
    /// verbatim. Genuine generic panics get the `(at file:line)` suffix via
    /// [`crate::unwind_protect::panic_message_with_location`], which reads the
    /// current thread's take-once slot — correct only because every call site
    /// below runs on the same thread whose hook caught this exact panic
    /// (main thread for both uses in `dispatch_to_worker`'s main-thread event
    /// loop, #1245).
    fn fold_panic_message(payload: &(dyn Any + Send)) -> String {
        if payload.is::<crate::condition::RCondition>() {
            crate::unwind_protect::panic_payload_to_string(payload).into_owned()
        } else {
            crate::unwind_protect::panic_message_with_location(payload)
        }
    }

    /// Dispatch a closure to the worker thread.
    /// Returns Ok(T) or Err(panic_message).
    pub(super) fn dispatch_to_worker<F, T>(f: F) -> Result<T, String>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        /// Marker type for R errors caught by R_UnwindProtect's cleanup handler.
        struct RErrorMarker;

        // Re-entry guard: if we're already on the worker thread (inside a
        // run_on_worker job), a nested run_on_worker would deadlock because the
        // single worker thread can't pick up a new job while running the current one.
        //
        // Checked before the main-thread debug_assert so re-entry from the worker
        // produces its specific message rather than the more general
        // "must be called from the R main thread" assert (worker thread isn't main).
        if has_context() {
            panic!(
                "run_on_worker called re-entrantly from within a worker context.\n\
                 \n\
                 The single worker thread is already executing a job, so a nested \
                 run_on_worker would deadlock. To call R APIs from worker code, \
                 use with_r_thread() instead."
            );
        }

        // Precondition: the caller is the R main thread. `dispatch_to_worker`
        // runs the main-thread event loop on whatever thread invokes it, so
        // a non-main caller silently routes `with_r_thread` callbacks to the
        // wrong thread. `.Call` always lands here on R's main thread, so this
        // is a programming error rather than a runtime condition — debug-only
        // assert, no atomic load in release. See #730.
        debug_assert!(
            super::is_r_main_thread(),
            "run_on_worker must be called from the R main thread \
             (the thread that ran miniextendr_runtime_init); see #730"
        );

        // Clone the worker's sender while holding the mutex briefly. The
        // clone outlives the lock, so sends happen without blocking other
        // callers on the mutex. If `WORKER` is `None`, the package has
        // already been unloaded (or never initialized) — return a
        // structured error instead of panicking.
        let job_tx = {
            let guard = WORKER.lock().unwrap();
            match guard.as_ref() {
                Some(state) => state.tx.clone(),
                None => {
                    return Err(
                        "miniextendr worker is not running (runtime not initialized, \
                         or package has been unloaded)"
                            .to_string(),
                    );
                }
            }
        };

        // Single channel for worker -> main (work requests + final result).
        // Capacity 1: each run_on_worker sends exactly one request at a time and blocks
        // for a response, so no accumulation is possible. The extra slot ensures the
        // worker's final Done message doesn't block if the main thread longjmped away.
        let (worker_tx, worker_rx) = mpsc::sync_channel::<TypeErasedWorkerMessage>(1);

        // Channel for main -> worker responses to work requests.
        // Capacity 1: the worker blocks on recv after each with_r_thread call, so at most
        // one response is in flight. The extra slot lets the cleanup handler send an error
        // without blocking (it runs mid-longjmp and cannot wait).
        let (response_tx, response_rx) = mpsc::sync_channel::<MainThreadResponse>(1);

        let job: AnyJob = Box::new(move || {
            // Set up thread-local channels for with_r_thread
            WORKER_TO_MAIN_TX.with(|tx_cell| {
                *tx_cell.borrow_mut() = Some(worker_tx.clone());
            });
            MAIN_RESPONSE_RX.with(|rx_cell| {
                *rx_cell.borrow_mut() = Some(response_rx);
            });

            let result = catch_unwind(AssertUnwindSafe(f));

            // Clear thread-locals
            WORKER_TO_MAIN_TX.with(|tx_cell| {
                *tx_cell.borrow_mut() = None;
            });
            MAIN_RESPONSE_RX.with(|rx_cell| {
                *rx_cell.borrow_mut() = None;
            });

            // Send final result back to the main thread's recv loop. The capacity-1
            // buffer ensures this doesn't block even if the main thread already exited
            // the loop (e.g., after an R longjmp consumed the last WorkRequest).
            let to_send: Result<Box<dyn Any + Send>, String> = match result {
                Ok(val) => Ok(Box::new(val)),
                Err(payload) => {
                    // Fold the panic location HERE, on the worker (origin)
                    // thread: the hook fired on the worker for this panic, so
                    // its take-once slot holds the real `panic!` site. The main
                    // side then treats the already-final message verbatim
                    // (`make_rust_condition_value(&__panic_msg, PANIC, …)` in the
                    // generated worker wrapper) — no second fold, no clobber.
                    //
                    // Only genuine generic panics get the `(at …)` suffix. User
                    // conditions (error!/warning!/message!/condition!) travel as
                    // `RCondition` payloads and must stay location-free — mirror
                    // the main-thread RCondition branch by stringifying verbatim.
                    //
                    // `PreLocatedPanic` is a THIRD case (#1245): the re-panic in
                    // `route_to_main_thread` (a `with_r_thread` closure that
                    // panicked on the MAIN thread, relayed back here) already
                    // carries a final, correctly-folded message from the true
                    // origin. This thread's panic hook still fired for the
                    // `panic_any` relay call itself, though, and recorded ITS
                    // OWN call site (in `worker.rs`, not the user's) into this
                    // thread's take-once slot — stale. Use the payload's
                    // message verbatim and drain-and-discard that stale slot so
                    // it can't leak into a later, unrelated fold on this same
                    // (reused) worker thread.
                    let msg = if let Some(pre) =
                        payload.downcast_ref::<crate::unwind_protect::PreLocatedPanic>()
                    {
                        let _ = crate::backtrace::take_last_panic_location();
                        pre.0.clone()
                    } else {
                        fold_panic_message(&*payload)
                    };
                    Err(msg)
                }
            };
            let _ = worker_tx.send(WorkerMessage::Done(to_send));
        });

        job_tx
            .send(WorkerMsg::Job(job))
            .expect("worker thread dead");

        // Main thread event loop: processes WorkRequest messages (from with_r_thread)
        // until a Done message arrives. Invariant: each WorkRequest produces exactly
        // one response_tx.send, and the worker blocks until it receives that response.
        loop {
            match worker_rx
                .recv()
                .expect("worker channel closed unexpectedly")
            {
                WorkerMessage::WorkRequest(work) => {
                    // Execute work on main thread with R_UnwindProtect so we can:
                    // 1. Catch Rust panics and send them as errors to the worker
                    // 2. Catch R errors (longjmp) via cleanup handler and send error to worker
                    //    before R continues unwinding (function never returns in that case)

                    struct CallData {
                        work: Option<MainThreadWork>,
                        result: Option<Box<dyn Any + Send>>,
                        panic_payload: Option<Box<dyn Any + Send>>,
                        response_tx_ptr: *const SyncSender<MainThreadResponse>,
                    }

                    unsafe extern "C-unwind" fn trampoline(data: *mut std::ffi::c_void) -> SEXP {
                        assert!(!data.is_null(), "trampoline: data pointer is null");
                        let data = unsafe { &mut *data.cast::<CallData>() };
                        let work = data
                            .work
                            .take()
                            .expect("trampoline: work already consumed")
                            .0;

                        match catch_unwind(AssertUnwindSafe(work)) {
                            Ok(result) => {
                                data.result = Some(result);
                                SEXP::nil()
                            }
                            Err(payload) => {
                                data.panic_payload = Some(payload);
                                SEXP::nil()
                            }
                        }
                    }

                    unsafe extern "C-unwind" fn cleanup_handler(
                        data: *mut std::ffi::c_void,
                        jump: Rboolean,
                    ) {
                        if jump != Rboolean::FALSE {
                            // R is about to longjmp. We MUST send an error response to the worker
                            // before continuing the unwind—the worker is blocked on response_rx.recv()
                            // and would deadlock if we don't send something.
                            assert!(!data.is_null(), "cleanup_handler: data pointer is null");
                            let data = unsafe { &*data.cast::<CallData>() };
                            let response_tx = unsafe { &*data.response_tx_ptr };

                            #[cfg(feature = "nonapi")]
                            let error_msg = unsafe {
                                let buf = sys::R_curErrorBuf();
                                if buf.is_null() {
                                    "R error occurred".to_string()
                                } else {
                                    std::ffi::CStr::from_ptr(buf).to_string_lossy().into_owned()
                                }
                            };
                            #[cfg(not(feature = "nonapi"))]
                            let error_msg = "R error occurred".to_string();

                            let _ = response_tx.send(Err(error_msg));
                            std::panic::panic_any(RErrorMarker);
                        }
                    }

                    let response: MainThreadResponse = unsafe {
                        let token = crate::unwind_protect::get_continuation_token();

                        let data = Box::into_raw(Box::new(CallData {
                            work: Some(work),
                            result: None,
                            panic_payload: None,
                            response_tx_ptr: std::ptr::from_ref(&response_tx),
                        }));

                        let panic_result = catch_unwind(AssertUnwindSafe(|| {
                            sys::R_UnwindProtect_C_unwind(
                                Some(trampoline),
                                data.cast(),
                                Some(cleanup_handler),
                                data.cast(),
                                token,
                            )
                        }));

                        let mut data = Box::from_raw(data);

                        match panic_result {
                            Ok(_) => {
                                // Check if trampoline caught a panic
                                if let Some(payload) = data.panic_payload.take() {
                                    // This IS the real panic origin thread (main) —
                                    // fold its location in now (#1245), before the
                                    // message crosses back to the worker.
                                    Err(fold_panic_message(&*payload))
                                } else {
                                    // Normal completion - return the result
                                    Ok(data
                                        .result
                                        .take()
                                        .expect("result not set after successful completion"))
                                }
                            }
                            Err(payload) => {
                                // Check if this was an R error (cleanup handler already sent response)
                                if payload.downcast_ref::<RErrorMarker>().is_some() {
                                    drop(data);
                                    sys::R_ContinueUnwind(token);
                                }
                                // Rust panic - return as error response. Also
                                // the real origin thread (main) — fold its
                                // location in now (#1245).
                                Err(fold_panic_message(&*payload))
                            }
                        }
                    };

                    // Exactly one send per WorkRequest: either here (normal/panic) or
                    // in cleanup_handler (R error). Never both—R error path diverges
                    // via R_ContinueUnwind above and never reaches this line.
                    response_tx
                        .send(response)
                        .expect("worker response channel closed");
                }
                WorkerMessage::Done(result) => {
                    return match result {
                        Ok(boxed) => Ok(*boxed
                            .downcast::<T>()
                            .expect("type mismatch in run_on_worker result")),
                        Err(msg) => Err(msg),
                    };
                }
            }
        }
    }

    /// Spawn the worker thread and install it as the global `WORKER`.
    ///
    /// Idempotent — if the worker is already running, this is a no-op. We
    /// intentionally do NOT call this from a `OnceLock`: after `shutdown()`
    /// the slot is cleared, and a subsequent `dyn.load` on the same DLL
    /// (same statics, unchanged addresses) should be able to spawn a fresh
    /// worker. `std::sync::Once` would forbid that.
    pub(super) fn init_worker() {
        let mut guard = WORKER.lock().unwrap();
        if guard.is_some() {
            return;
        }
        // Capacity 0 (rendezvous): the main thread blocks until the worker picks
        // up the job, ensuring at most one job is in flight at a time.
        let (tx, rx) = mpsc::sync_channel::<WorkerMsg>(0);
        let handle = thread::Builder::new()
            .name("miniextendr-worker".into())
            .spawn(move || worker_loop(rx))
            .expect("failed to spawn worker thread");
        *guard = Some(WorkerState { tx, handle });
    }

    /// Worker thread body: blocking `recv()` loop.
    ///
    /// `recv()` blocks the thread in the OS until either a message arrives
    /// or the sender is dropped — no polling, no timeouts, no sleeps. On
    /// `Job`, run it. On `Shutdown` or `Err` (sender dropped), exit.
    fn worker_loop(rx: Receiver<WorkerMsg>) {
        while let Ok(msg) = rx.recv() {
            match msg {
                WorkerMsg::Job(job) => job(),
                WorkerMsg::Shutdown => break,
            }
        }
    }
}
// endregion

// region: Tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sendable_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Sendable<*const u8>>();
    }

    #[test]
    fn with_r_thread_panics_before_init() {
        // If another test already called miniextendr_runtime_init (via Once),
        // we can't test the pre-init path. Verify at least panics from wrong thread.
        if R_MAIN_THREAD_ID.get().is_some() {
            let handle = std::thread::spawn(|| std::panic::catch_unwind(|| with_r_thread(|| 42)));
            let result = handle.join().expect("thread panicked outside catch_unwind");
            assert!(
                result.is_err(),
                "with_r_thread should panic from non-main thread"
            );
            return;
        }
        let result = std::panic::catch_unwind(|| {
            with_r_thread(|| 42);
        });
        assert!(result.is_err());
        let payload = result.unwrap_err();
        let msg = crate::unwind_protect::panic_payload_to_string(payload.as_ref());
        assert!(
            msg.contains("miniextendr_runtime_init"),
            "expected init error message, got: {msg}"
        );
    }

    #[test]
    fn has_worker_context_false_outside_worker() {
        assert!(!has_worker_context());
    }

    // region: Feature-gated tests: worker-thread

    #[cfg(feature = "worker-thread")]
    mod worker_tests {
        use super::*;

        /// Calling `run_on_worker` from within worker code (re-entry) must be
        /// detected and panic, not deadlock.
        ///
        /// Dispatches from the current test thread rather than a spawned one.
        /// `run_on_worker` requires the R main thread (#730), and a spawned
        /// std::thread would trip the debug_assert before the re-entry check
        /// gets a chance to run. If this test happens to run on a thread that
        /// isn't `R_MAIN_THREAD_ID` (another test initialised first), skip —
        /// the reentry behaviour is checked elsewhere on each invocation of
        /// the suite.
        #[test]
        fn run_on_worker_reentry_panics_not_deadlocks() {
            miniextendr_runtime_init();
            if !is_r_main_thread() {
                return;
            }

            let result = run_on_worker(|| {
                // Re-entry: this runs on the worker thread.
                run_on_worker(|| 42).unwrap();
            });

            let msg = result.expect_err("re-entry should surface as Err");
            assert!(
                msg.contains("re-entr") || msg.contains("Re-entr"),
                "expected re-entry error, got: {msg}"
            );
        }

        /// `run_on_worker` from a non-main thread trips the debug-only
        /// precondition assert (#730). Spawning a fresh std::thread guarantees
        /// the caller isn't `R_MAIN_THREAD_ID`, regardless of which thread
        /// `miniextendr_runtime_init` first ran on.
        ///
        /// `cfg(debug_assertions)` only — the assert is compiled out in
        /// release, by design.
        #[cfg(debug_assertions)]
        #[test]
        fn run_on_worker_from_non_main_thread_asserts_in_debug() {
            miniextendr_runtime_init();

            let (tx, rx) = std::sync::mpsc::sync_channel::<Result<String, ()>>(1);
            std::thread::spawn(move || {
                let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    run_on_worker(|| 0i32)
                }));
                let report = match outcome {
                    Err(payload) => Ok(crate::unwind_protect::panic_payload_to_string(
                        payload.as_ref(),
                    )
                    .into_owned()),
                    Ok(_) => Err(()),
                };
                let _ = tx.send(report);
            });

            match rx.recv_timeout(std::time::Duration::from_secs(5)) {
                Ok(Ok(msg)) => {
                    assert!(
                        msg.contains("R main thread"),
                        "expected main-thread precondition assert, got: {msg}"
                    );
                }
                Ok(Err(())) => {
                    panic!("debug_assert did not fire when run_on_worker was called from non-main")
                }
                Err(_) => panic!("test deadlocked waiting for debug_assert panic"),
            }
        }
    }
    // endregion

    // region: Feature-gated tests: no worker-thread (stubs)

    #[cfg(not(feature = "worker-thread"))]
    mod stub_tests {
        use super::*;

        #[test]
        fn stub_with_r_thread_inline() {
            miniextendr_runtime_init();
            // If another parallel test already set R_MAIN_THREAD_ID to a
            // different thread (OnceLock), we won't be "main" and with_r_thread
            // will rightfully panic. Skip in that case.
            if !is_r_main_thread() {
                return;
            }
            let result = with_r_thread(|| 42);
            assert_eq!(result, 42);
        }

        #[test]
        fn stub_run_on_worker_inline() {
            let result = run_on_worker(|| 123);
            assert_eq!(result, Ok(123));
        }

        /// Without `worker-thread`, `with_r_thread` must panic when called from
        /// a non-main thread.
        #[test]
        fn stub_with_r_thread_panics_on_wrong_thread() {
            miniextendr_runtime_init();

            let handle = std::thread::spawn(|| {
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| with_r_thread(|| 42)))
            });

            let result = handle.join().expect("thread panicked outside catch_unwind");
            assert!(
                result.is_err(),
                "with_r_thread should panic when called from a non-main thread \
                 without the worker-thread feature, but it ran inline silently"
            );
        }
    }
    // endregion
}
// endregion
