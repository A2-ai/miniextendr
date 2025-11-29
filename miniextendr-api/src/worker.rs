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

use crate::ffi;
use crate::unwind_protect::with_r_unwind_protect;

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

// Thread-local channels for worker -> main communication during run_on_worker
thread_local! {
    // Channel to send messages (work requests or done) to main thread
    static WORKER_TO_MAIN_TX: RefCell<Option<SyncSender<WorkerMessage<Box<dyn Any + Send>>>>> = const { RefCell::new(None) };
    // Channel to receive responses from main thread
    static MAIN_RESPONSE_RX: RefCell<Option<Receiver<MainThreadResponse>>> = const { RefCell::new(None) };
}

/// Check if the current thread is R's main thread.
///
/// Returns `true` if called from the main R thread, `false` otherwise.
/// If the worker hasn't been initialized yet, returns `true` (assumes main thread).
#[inline(always)]
pub fn is_r_main_thread() -> bool {
    R_MAIN_THREAD_ID
        .get()
        .map(|&id| id == std::thread::current().id())
        .unwrap_or(true) // If not initialized, assume we're on main thread
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
            Ok(boxed) => *boxed.downcast::<R>().expect("type mismatch in with_r_thread response"),
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
    let job_tx = JOB_TX
        .get()
        .expect("worker not initialized - call miniextendr_worker_init first");

    // Single channel for worker -> main (work requests + final result)
    let (worker_tx, worker_rx) =
        mpsc::sync_channel::<WorkerMessage<Box<dyn Any + Send>>>(0);

    // Channel for main -> worker responses to work requests
    let (response_tx, response_rx) = mpsc::sync_channel::<MainThreadResponse>(0);

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
        match worker_rx.recv().expect("worker channel closed unexpectedly") {
            WorkerMessage::WorkRequest(work) => {
                // Execute work on main thread with panic and R error protection
                let response = catch_unwind(AssertUnwindSafe(|| {
                    with_r_unwind_protect(work, None)
                }))
                .map_err(|payload| panic_payload_to_string(&payload));
                response_tx.send(response).expect("worker response channel closed");
            }
            WorkerMessage::Done(result) => {
                return match result {
                    Ok(boxed) => *boxed.downcast::<T>().expect("type mismatch in run_on_worker result"),
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
