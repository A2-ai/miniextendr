//! Worker thread for safe Rust-R FFI with longjmp protection.
//!
//! Execute Rust code on a separate worker thread. If a panic occurs,
//! `catch_unwind` catches it and the stack unwinds naturally, running all Drops.
//! The main thread then converts the result to SEXP or raises an R error.
//!
//! ## Bidirectional Communication
//!
//! The worker can call R APIs via [`with_r_thread`], which sends work back to the
//! main thread. The main thread processes these requests while waiting for the
//! worker's final result.

use std::any::Any;
use std::cell::RefCell;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::OnceLock;
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::thread;

use crate::ffi::{self, Rboolean, SEXP};

static R_MAIN_THREAD_ID: OnceLock<thread::ThreadId> = OnceLock::new();

type AnyJob = Box<dyn FnOnce() + Send>;

static JOB_TX: OnceLock<SyncSender<AnyJob>> = OnceLock::new();

// Type-erased main thread work: closure that returns boxed result
type MainThreadWork = Box<dyn FnOnce() -> Box<dyn Any + Send> + Send>;

// Response from main thread: Ok(result) or Err(panic_message)
type MainThreadResponse = Result<Box<dyn Any + Send>, String>;

/// Messages from worker to main thread
enum WorkerMessage<T> {
    /// Worker requests main thread to execute some work, then send response back
    WorkRequest(MainThreadWork),
    /// Worker is done, here's the final result
    Done(Result<T, String>),
}

// Type alias for the common worker message type
type TypeErasedWorkerMessage = WorkerMessage<Box<dyn Any + Send>>;

// Thread-local channels for worker -> main communication during run_on_worker
thread_local! {
    static R_CONTINUATION_TOKEN: std::cell::LazyCell<SEXP> = std::cell::LazyCell::new(|| unsafe {
        let token = ffi::R_MakeUnwindCont();
        ffi::R_PreserveObject(token);
        token
    });
    // Channel to send messages (work requests or done) to main thread
    #[allow(clippy::type_complexity)]
    static WORKER_TO_MAIN_TX: RefCell<Option<SyncSender<TypeErasedWorkerMessage>>> = const { RefCell::new(None) };
    // Channel to receive responses from main thread
    static MAIN_RESPONSE_RX: RefCell<Option<Receiver<MainThreadResponse>>> = const { RefCell::new(None) };
}

/// Check whether the current thread is running inside a `run_on_worker` context
/// (i.e., `with_r_thread` has routing channels available).
pub fn has_worker_context() -> bool {
    WORKER_TO_MAIN_TX.with(|tx_cell| tx_cell.borrow().is_some())
}

/// Check if the current thread is R's main thread.
///
/// Returns `true` if called from the main R thread, `false` otherwise.
/// If the worker hasn't been initialized yet, returns `false` (safe default).
///
/// # Note
///
/// Before `miniextendr_worker_init()` is called, this always returns `false`.
/// This is intentional - it prevents R API calls from arbitrary threads before
/// the worker is properly set up. Ensure `miniextendr_worker_init()` is called
/// during R package initialization (typically in `R_init_<pkgname>`).
#[inline(always)]
pub fn is_r_main_thread() -> bool {
    R_MAIN_THREAD_ID
        .get()
        .map(|&id| id == std::thread::current().id())
        .unwrap_or(false) // Safe default: assume NOT main thread until initialized
}

/// Extract a message from a panic payload.
pub fn panic_payload_to_string(payload: &Box<dyn Any + Send>) -> String {
    if let Some(&s) = payload.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "unknown panic".to_string()
    }
}

/// Raise an R error from a panic message. Does not return.
pub fn panic_message_to_r_error(msg: String) -> ! {
    let c_msg = std::ffi::CString::new(msg)
        .unwrap_or_else(|_| std::ffi::CString::new("Rust panic (invalid message)").unwrap());
    unsafe {
        ffi::Rf_error_unchecked(c"%s".as_ptr(), c_msg.as_ptr());
    }
}

/// Execute a closure on R's main thread, returning the result.
///
/// This function can be called from any thread:
/// - If called from the main thread, executes the closure directly
/// - If called from the worker thread (during `run_on_worker`), sends the work
///   to the main thread and blocks until completion
///
/// # Panics
///
/// Panics if called from the worker thread outside of a `run_on_worker` context.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::worker::with_r_thread;
///
/// // From worker thread, safely call R APIs:
/// let sexp = with_r_thread(|| {
///     // This runs on R's main thread
///     unsafe { R_NilValue }
/// });
/// ```
pub fn with_r_thread<F, R>(f: F) -> R
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    if is_r_main_thread() {
        // Already on main thread, just run it
        return f();
    }

    // On worker thread - send work request to main thread
    WORKER_TO_MAIN_TX.with(|tx_cell| {
        let tx = tx_cell
            .borrow()
            .as_ref()
            .expect("with_r_thread called outside of run_on_worker context")
            .clone();

        // Create type-erased work that boxes the result
        let work: MainThreadWork = Box::new(move || Box::new(f()) as Box<dyn Any + Send>);

        // Send work request to main thread
        tx.send(WorkerMessage::WorkRequest(work))
            .expect("main thread channel closed");
    });

    // Wait for response
    MAIN_RESPONSE_RX.with(|rx_cell| {
        let rx = rx_cell.borrow();
        let rx = rx.as_ref().expect("response channel not set");
        let response = rx.recv().expect("main thread response channel closed");
        match response {
            Ok(boxed) => *boxed
                .downcast::<R>()
                .expect("type mismatch in with_r_thread response"),
            Err(panic_msg) => panic!("panic in with_r_thread: {}", panic_msg),
        }
    })
}

/// Run a Rust closure on the worker thread with proper cleanup on panic.
///
/// Panics are caught and converted to R errors. Destructors run properly.
/// The closure can use [`with_r_thread`] to execute code on the main thread.
pub fn run_on_worker<F, T>(f: F) -> T
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    /// Marker type for R errors caught by R_UnwindProtect's cleanup handler.
    struct RErrorMarker;

    let job_tx = JOB_TX
        .get()
        .expect("worker not initialized - call miniextendr_worker_init first");

    // Single channel for worker -> main (work requests + final result)
    // Use buffer of 1 so the worker's final Done send doesn't block
    // if the main thread longjmps away (R error case)
    let (worker_tx, worker_rx) = mpsc::sync_channel::<TypeErasedWorkerMessage>(1);

    // Channel for main -> worker responses to work requests
    // Use buffer of 1 so cleanup handler's error send doesn't block
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

        // Send final result (type-erased)
        let to_send: Result<Box<dyn Any + Send>, String> = match result {
            Ok(val) => Ok(Box::new(val)),
            Err(payload) => Err(panic_payload_to_string(&payload)),
        };
        let _ = worker_tx.send(WorkerMessage::Done(to_send));
    });

    job_tx.send(job).expect("worker thread dead");

    // Main thread: block on single channel, handle work requests or final result
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
                    let data = unsafe { data.cast::<CallData>().as_mut().unwrap() };
                    let work = data.work.take().expect("trampoline: work already consumed");

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
                        // R is about to longjmp - send an error response to the worker first!
                        // This prevents the worker from blocking forever.
                        let data = unsafe { data.cast::<CallData>().as_ref().unwrap() };
                        let response_tx = unsafe { &*data.response_tx_ptr };

                        // Send error response - ignore send errors since we're about to unwind anyway
                        let _ = response_tx.send(Err("R error occurred".to_string()));

                        // Now trigger a Rust panic so catch_unwind below can catch it
                        // and we can properly continue R's unwind
                        std::panic::panic_any(RErrorMarker);
                    }
                }

                let response: MainThreadResponse = unsafe {
                    let token = R_CONTINUATION_TOKEN.with(|x| **x);

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
                            data.cast(), // Pass data to cleanup handler too
                            token,
                        )
                    }));

                    let mut data = Box::from_raw(data);

                    match panic_result {
                        Ok(_) => {
                            // Check if trampoline caught a panic
                            if let Some(payload) = data.panic_payload.take() {
                                Err(panic_payload_to_string(&payload))
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
                                // R error - drop data before continuing R's unwind
                                drop(data);
                                // Response was already sent in cleanup handler
                                ffi::R_ContinueUnwind(token);
                            }
                            // Rust panic - return as error response
                            Err(panic_payload_to_string(&payload))
                        }
                    }
                };

                response_tx
                    .send(response)
                    .expect("worker response channel closed");
            }
            WorkerMessage::Done(result) => {
                return match result {
                    Ok(boxed) => *boxed
                        .downcast::<T>()
                        .expect("type mismatch in run_on_worker result"),
                    Err(msg) => panic_message_to_r_error(msg),
                };
            }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn miniextendr_worker_init() {
    static RUN_ONCE: std::sync::Once = std::sync::Once::new();
    RUN_ONCE.call_once_force(|x| {
        // just ignore repeated calls to this function
        if x.is_poisoned() {
            println!("warning: miniextendr worker initialisation was done more than once");
            return;
        }
        let _ = R_MAIN_THREAD_ID.set(std::thread::current().id());

        if JOB_TX.get().is_some() {
            return;
        }
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
    });
}
