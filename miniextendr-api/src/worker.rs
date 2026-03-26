//! Worker thread infrastructure for safe Rust-R FFI.
//!
//! ## Public API
//!
//! - [`with_r_thread`] — Execute a closure on R's main thread
//! - [`is_r_main_thread`] — Check if the current thread is R's main thread
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
//! dispatches closures to the worker with bidirectional communication.
//!
//! ## Initialization
//!
//! [`miniextendr_runtime_init`] must be called from R's main thread before any
//! R FFI APIs. Typically done in `R_init_<pkgname>()`.

use std::sync::OnceLock;
use std::thread;

use crate::ffi::{self, SEXP};

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
/// This function can be called from any thread:
/// - From the main thread: executes the closure directly (re-entrant)
/// - From the worker thread (during `run_on_worker`): sends the work to
///   the main thread and blocks until completion
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
///     unsafe { R_NilValue }
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

    // Not on main thread — need worker-thread feature for routing
    #[cfg(not(feature = "worker-thread"))]
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

    #[cfg(feature = "worker-thread")]
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
            Some(call) => ffi::Rf_errorcall_unchecked(call, c"%s".as_ptr(), c_msg.as_ptr()),
            None => ffi::Rf_error_unchecked(c"%s".as_ptr(), c_msg.as_ptr()),
        }
    }
}

/// Run a closure on the worker thread with proper cleanup on panic.
///
/// Returns `Ok(T)` on success, `Err(String)` if the closure panicked.
/// The caller handles the error (either tagged error value or `Rf_errorcall`).
///
/// Without the `worker-thread` feature, runs inline on the current thread.
#[doc(hidden)]
pub fn run_on_worker<F, T>(f: F) -> Result<T, String>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    #[cfg(not(feature = "worker-thread"))]
    {
        Ok(f())
    }

    #[cfg(feature = "worker-thread")]
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

    #[cfg(feature = "worker-thread")]
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
        });
    }

    #[cfg(not(feature = "worker-thread"))]
    {
        RUN_ONCE.call_once(|| {
            let _ = R_MAIN_THREAD_ID.set(std::thread::current().id());
        });
    }
}
// endregion

// region: pub(crate) internals

/// Check whether the current thread has a worker routing context.
pub(crate) fn has_worker_context() -> bool {
    #[cfg(feature = "worker-thread")]
    {
        worker_channel::has_context()
    }
    #[cfg(not(feature = "worker-thread"))]
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

#[cfg(feature = "worker-thread")]
mod worker_channel {
    use std::any::Any;
    use std::cell::RefCell;
    use std::panic::{AssertUnwindSafe, catch_unwind};
    use std::sync::mpsc::{self, Receiver, SyncSender};
    use std::thread;

    use super::Sendable;
    use crate::ffi::{self, Rboolean, SEXP};

    type AnyJob = Box<dyn FnOnce() + Send>;

    static JOB_TX: std::sync::OnceLock<SyncSender<AnyJob>> = std::sync::OnceLock::new();

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
                Err(panic_msg) => panic!("panic in `with_r_thread`: {}", panic_msg),
            }
        })
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
        if has_context() {
            panic!(
                "run_on_worker called re-entrantly from within a worker context.\n\
                 \n\
                 The single worker thread is already executing a job, so a nested \
                 run_on_worker would deadlock. To call R APIs from worker code, \
                 use with_r_thread() instead."
            );
        }

        let job_tx = JOB_TX
            .get()
            .expect("worker not initialized - call miniextendr_runtime_init first");

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
                Err(payload) => Err(crate::unwind_protect::panic_payload_to_string(&*payload)),
            };
            let _ = worker_tx.send(WorkerMessage::Done(to_send));
        });

        job_tx.send(job).expect("worker thread dead");

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
                                unsafe { ffi::R_NilValue }
                            }
                            Err(payload) => {
                                data.panic_payload = Some(payload);
                                unsafe { ffi::R_NilValue }
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
                                let buf = ffi::R_curErrorBuf();
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
                            ffi::R_UnwindProtect_C_unwind(
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
                                    Err(crate::unwind_protect::panic_payload_to_string(&*payload))
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
                                    ffi::R_ContinueUnwind(token);
                                }
                                // Rust panic - return as error response
                                Err(crate::unwind_protect::panic_payload_to_string(&*payload))
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

    /// Spawn the worker thread and set up the job channel.
    pub(super) fn init_worker() {
        if JOB_TX.get().is_some() {
            return; // Worker already running
        }
        // Capacity 0 (rendezvous): the main thread blocks until the worker picks
        // up the job, ensuring at most one job is in flight at a time.
        let (job_tx, job_rx) = mpsc::sync_channel::<AnyJob>(0);
        thread::Builder::new()
            .name("miniextendr-worker".into())
            .spawn(move || {
                while let Ok(job) = job_rx.recv() {
                    job();
                }
            })
            .expect("failed to spawn worker thread");

        JOB_TX.set(job_tx).expect("worker already initialized");
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
        #[test]
        fn run_on_worker_reentry_panics_not_deadlocks() {
            miniextendr_runtime_init();

            let (tx, rx) = std::sync::mpsc::sync_channel::<Result<String, String>>(1);

            std::thread::spawn(move || {
                let result = run_on_worker(|| {
                    // Re-entry: this is on the worker thread already.
                    run_on_worker(|| 42).unwrap();
                });
                match result {
                    Err(msg) => {
                        let _ = tx.send(Ok(msg));
                    }
                    Ok(()) => {
                        let _ = tx.send(Err("re-entry was not detected".into()));
                    }
                }
            });

            match rx.recv_timeout(std::time::Duration::from_secs(5)) {
                Ok(Ok(msg)) => {
                    assert!(
                        msg.contains("re-entr") || msg.contains("Re-entr"),
                        "expected re-entry error, got: {msg}"
                    );
                }
                Ok(Err(msg)) => panic!("{msg}"),
                Err(_) => {
                    panic!("DEADLOCK: run_on_worker re-entry caused the test to hang for 5 seconds")
                }
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
