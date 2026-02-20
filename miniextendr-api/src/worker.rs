//! Worker thread for safe Rust-R FFI with longjmp protection.
//!
//! Execute Rust code on a separate worker thread. If a panic occurs,
//! `catch_unwind` catches it and the stack unwinds naturally, running all Drops.
//! The main thread then converts the result to SEXP or raises an R error.
//!
//! ## Initialization
//!
//! Before using any R FFI APIs, [`miniextendr_worker_init`] must be called from R's
//! main thread. This is typically done in `R_init_<pkgname>()`:
//!
//! ```c
//! void R_init_pkgname(DllInfo *dll) {
//!     miniextendr_worker_init();
//!     R_registerRoutines(dll, NULL, CallEntries, NULL, NULL);
//! }
//! ```
//!
//! Calling R FFI APIs before initialization will panic with a descriptive error.
//!
//! ## Bidirectional Communication
//!
//! The worker can call R APIs via [`with_r_thread`], which sends work back to the
//! main thread. The main thread processes these requests while waiting for the
//! worker's final result.
//!
//! ## API Categories
//!
//! R FFI functions wrapped with `#[r_ffi_checked]` fall into two categories:
//!
//! 1. **Value-returning functions** (e.g., `Rf_ScalarInteger`, `Rf_allocVector`):
//!    These are automatically routed to the main thread via [`with_r_thread`] when
//!    called from a worker thread. The result is sent back to the worker.
//!
//! 2. **Pointer-returning functions** (e.g., `INTEGER`, `REAL`, `DATAPTR`):
//!    These MUST be called from the main thread and will panic if called from a
//!    worker thread. Raw pointers cannot be safely routed because the pointed-to
//!    memory could be garbage collected before the worker uses it.
//!
//! ## R Error Handling
//!
//! When R errors occur during routed calls, the error is sent to the worker thread,
//! which then panics with the error message.
//!
//! With the `nonapi` feature enabled, the actual R error message is captured via
//! `R_curErrorBuf()`, preserving diagnostic information across the thread boundary.
//! Without this feature (CRAN-compatible builds), a generic "R error occurred"
//! message is used instead.

use std::any::Any;
use std::cell::RefCell;
use std::fmt;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::OnceLock;
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::thread;

use crate::ffi::{self, Rboolean, SEXP};

// ---------------------------------------------------------------------------
// PanicError: structured panic payload wrapper
// ---------------------------------------------------------------------------

/// A structured representation of a Rust panic payload with diagnostics.
///
/// Wraps the raw `Box<dyn Any + Send>` panic payload and extracts a human-readable
/// message. Also records the [`PanicSource`] (worker, ALTREP, unwind_protect, or
/// connection) so callers can distinguish where the panic originated.
///
/// # Examples
///
/// ```ignore
/// use miniextendr_api::worker::PanicError;
/// use miniextendr_api::panic_telemetry::PanicSource;
///
/// let err = PanicError::from_panic_payload(
///     Box::new("something went wrong"),
///     PanicSource::Worker,
/// );
/// assert_eq!(err.message(), "something went wrong");
/// assert_eq!(err.source(), PanicSource::Worker);
/// ```
pub struct PanicError {
    message: String,
    source: crate::panic_telemetry::PanicSource,
    /// The original type name of the panic payload (for diagnostics).
    payload_type: &'static str,
}

impl PanicError {
    /// Create a `PanicError` from a raw panic payload.
    ///
    /// Extracts the message from `&str`, `String`, and `&String` payloads.
    /// For unrecognised types, falls back to `"unknown panic"`.
    pub fn from_panic_payload(
        payload: Box<dyn Any + Send>,
        source: crate::panic_telemetry::PanicSource,
    ) -> Self {
        let payload_type = payload_type_name(payload.as_ref());
        let message = crate::unwind_protect::panic_payload_to_string(payload.as_ref());
        Self {
            message,
            source,
            payload_type,
        }
    }

    /// Create a `PanicError` from a string message directly.
    pub fn from_message(message: String, source: crate::panic_telemetry::PanicSource) -> Self {
        Self {
            message,
            source,
            payload_type: "String",
        }
    }

    /// The human-readable panic message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Which panic→R-error boundary caught this panic.
    pub fn source(&self) -> crate::panic_telemetry::PanicSource {
        self.source
    }

    /// The Rust type name of the original panic payload (e.g., `"&str"`, `"String"`).
    pub fn payload_type(&self) -> &'static str {
        self.payload_type
    }

    /// Consume into the inner message string.
    pub fn into_message(self) -> String {
        self.message
    }
}

impl fmt::Display for PanicError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:?}] {}", self.source, self.message)
    }
}

impl fmt::Debug for PanicError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PanicError")
            .field("message", &self.message)
            .field("source", &self.source)
            .field("payload_type", &self.payload_type)
            .finish()
    }
}

impl std::error::Error for PanicError {}

/// Identify the type name of a panic payload for diagnostics.
fn payload_type_name(payload: &(dyn Any + Send)) -> &'static str {
    if payload.downcast_ref::<&str>().is_some() {
        "&str"
    } else if payload.downcast_ref::<String>().is_some() {
        "String"
    } else if payload.downcast_ref::<&String>().is_some() {
        "&String"
    } else {
        "unknown"
    }
}

static R_MAIN_THREAD_ID: OnceLock<thread::ThreadId> = OnceLock::new();

type AnyJob = Box<dyn FnOnce() + Send>;

static JOB_TX: OnceLock<SyncSender<AnyJob>> = OnceLock::new();

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

// Type alias for the common worker message type
type TypeErasedWorkerMessage = WorkerMessage<Box<dyn Any + Send>>;

// Type alias to avoid type_complexity lint
type WorkerToMainSender = RefCell<Option<SyncSender<TypeErasedWorkerMessage>>>;
type MainResponseReceiver = RefCell<Option<Receiver<MainThreadResponse>>>;

// Thread-local channels for worker -> main communication during run_on_worker
thread_local! {
    // Channel to send messages (work requests or done) to main thread
    static WORKER_TO_MAIN_TX: WorkerToMainSender = const { RefCell::new(None) };
    // Channel to receive responses from main thread
    static MAIN_RESPONSE_RX: MainResponseReceiver = const { RefCell::new(None) };
}

/// Check whether the current thread is running inside a `run_on_worker` context
/// (i.e., `with_r_thread` has routing channels available).
pub fn has_worker_context() -> bool {
    WORKER_TO_MAIN_TX.with(|tx_cell| tx_cell.borrow().is_some())
}

/// Wrapper to mark values as Send for main-thread routing.
///
/// This is only safe if the value is not accessed on the worker thread and is
/// used exclusively on the main thread.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Sendable<T>(pub T);

unsafe impl<T> Send for Sendable<T> {}

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

/// Assert that the current thread is R's main thread.
///
/// In debug builds this is always active. In release builds it is a no-op
/// unless the `release-thread-check` feature is enabled.
///
/// Use this at the top of functions that MUST run on the main thread
/// (e.g., raw pointer accessors like `INTEGER`, `REAL`).
///
/// # Panics
///
/// Panics with a descriptive message if called from a non-main thread.
#[inline(always)]
pub fn assert_r_main_thread(fn_name: &str) {
    if (cfg!(debug_assertions) || cfg!(feature = "release-thread-check")) && !is_r_main_thread() {
        panic!(
            "{fn_name} must be called on R's main thread (current: {:?})",
            std::thread::current().id()
        );
    }
}

/// Extract a message from a panic payload.
pub fn panic_payload_to_string(payload: &Box<dyn Any + Send>) -> String {
    crate::unwind_protect::panic_payload_to_string(payload.as_ref())
}

/// Raise an R error from a panic message. Does not return.
pub fn panic_message_to_r_error(msg: String) -> ! {
    let c_msg = std::ffi::CString::new(msg)
        .unwrap_or_else(|_| std::ffi::CString::new("Rust panic (invalid message)").unwrap());
    unsafe {
        ffi::Rf_error_unchecked(c"%s".as_ptr(), c_msg.as_ptr());
    }
}

/// Raise an R error (with call context) from a panic message. Does not return.
pub fn panic_message_to_r_errorcall(msg: String, call: ffi::SEXP) -> ! {
    let c_msg = std::ffi::CString::new(msg)
        .unwrap_or_else(|_| std::ffi::CString::new("Rust panic (invalid message)").unwrap());
    unsafe {
        ffi::Rf_errorcall_unchecked(call, c"%s".as_ptr(), c_msg.as_ptr());
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
    F: FnOnce() -> R + 'static,
    R: Send + 'static,
{
    // Check if worker system has been initialized
    if R_MAIN_THREAD_ID.get().is_none() {
        panic!(
            "miniextendr_worker_init() must be called before using R FFI APIs.\n\
             \n\
             This is typically done in R_init_<pkgname>() via:\n\
             \n\
             void R_init_pkgname(DllInfo *dll) {{\n\
             miniextendr_worker_init();\n\
             R_registerRoutines(dll, NULL, CallEntries, NULL, NULL);\n\
             }}\n\
             \n\
             If you're embedding R in Rust, call miniextendr_worker_init() from the main thread \
             before any R API calls."
        );
    }

    if is_r_main_thread() {
        // Already on main thread, just run it
        return f();
    }

    // On worker thread - send work request to main thread
    WORKER_TO_MAIN_TX.with(|tx_cell| {
        let tx = tx_cell
            .borrow()
            .as_ref()
            .expect("`with_r_thread` called outside of `run_on_worker` context")
            .clone();

        // Create type-erased work that boxes the result
        let work: MainThreadWork = Sendable(Box::new(move || Box::new(f()) as Box<dyn Any + Send>));

        // Send work request to main thread. The worker blocks until the main
        // thread's loop picks this up and sends a response on response_tx.
        tx.send(WorkerMessage::WorkRequest(work))
            .expect("main thread channel closed");
    });

    // Block until the main thread sends a response. Exactly one response is
    // produced per WorkRequest—either a result, a panic error, or an R error
    // from the cleanup handler.
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

/// Run a Rust closure on the worker thread with proper cleanup on panic.
///
/// Panics are caught and converted to R errors. Destructors run properly.
/// The closure can use [`with_r_thread`] to execute code on the main thread.
pub fn run_on_worker<F, T>(f: F) -> T
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    match run_on_worker_inner(f) {
        Ok(val) => val,
        Err(msg) => {
            crate::panic_telemetry::fire(&msg, crate::panic_telemetry::PanicSource::Worker);
            panic_message_to_r_error(msg)
        }
    }
}

/// Like [`run_on_worker`], but returns `Result<T, String>` instead of
/// diverging on worker-thread panics. Used by `#[miniextendr(error_in_r)]`
/// mode so the caller can convert panics to tagged error values.
///
/// R-origin errors (longjmp) still pass through via `R_ContinueUnwind`.
pub fn run_on_worker_result<F, T>(f: F) -> Result<T, String>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    let result = run_on_worker_inner(f);
    if let Err(ref msg) = result {
        crate::panic_telemetry::fire(msg, crate::panic_telemetry::PanicSource::Worker);
    }
    result
}

/// Shared implementation for [`run_on_worker`] and [`run_on_worker_result`].
fn run_on_worker_inner<F, T>(f: F) -> Result<T, String>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    /// Marker type for R errors caught by R_UnwindProtect's cleanup handler.
    struct RErrorMarker;

    let job_tx = JOB_TX
        .get()
        .expect("worker not initialized - call miniextendr_worker_init first");

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
            Err(payload) => Err(panic_payload_to_string(&payload)),
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
                    let data = unsafe { data.cast::<CallData>().as_mut().unwrap() };
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
                        let data = unsafe { data.cast::<CallData>().as_ref().unwrap() };
                        let response_tx = unsafe { &*data.response_tx_ptr };

                        // Try to capture R's actual error message before sending.
                        // R_curErrorBuf is non-API, so only available with `nonapi` feature.
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

                        // Send error response. Ignore send errors—we're about to unwind.
                        // The capacity-1 buffer guarantees this won't block.
                        let _ = response_tx.send(Err(error_msg));

                        // Trigger a Rust panic so catch_unwind in the caller can catch it
                        // and call R_ContinueUnwind to resume R's longjmp.
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
                                // R error—cleanup_handler already sent the error response
                                // to the worker, so we just resume R's longjmp.
                                drop(data);
                                ffi::R_ContinueUnwind(token);
                            }
                            // Rust panic - return as error response
                            Err(panic_payload_to_string(&payload))
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

/// Execute multiple closures on the R main thread in a single round-trip.
///
/// Each closure runs sequentially on the main thread. Results are collected
/// into a `Vec`. If any closure panics, remaining closures are skipped and
/// the panic propagates.
///
/// This amortizes the ~440us channel overhead across N calls instead of
/// paying it N times.
///
/// # Panics
///
/// Panics if the worker hasn't been initialized (see [`miniextendr_worker_init`]).
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::worker::with_r_thread_batch;
///
/// let results = with_r_thread_batch(vec![
///     Box::new(|| 1 + 1),
///     Box::new(|| 2 + 2),
///     Box::new(|| 3 + 3),
/// ]);
/// assert_eq!(results, vec![2, 4, 6]);
/// ```
pub fn with_r_thread_batch<F, T>(work_items: Vec<F>) -> Vec<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    with_r_thread(move || work_items.into_iter().map(|f| f()).collect())
}

/// A scope for batching multiple R thread calls into a single round-trip.
///
/// Instead of one channel round-trip per call, `RThreadScope` collects
/// closures and executes them all in a single `with_r_thread` call when
/// [`execute`](RThreadScope::execute) is invoked.
///
/// Results are returned as `Vec<Box<dyn Any + Send>>` because each closure
/// may return a different type. Use [`downcast`](Box::downcast) to recover
/// the concrete type.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::worker::RThreadScope;
///
/// let mut scope = RThreadScope::new();
/// let idx_a = scope.push(|| 42i32);
/// let idx_b = scope.push(|| String::from("hello"));
///
/// let results = scope.execute();
/// let a: i32 = *results[idx_a].downcast().unwrap();
/// let b: String = *results[idx_b].downcast().unwrap();
/// assert_eq!(a, 42);
/// assert_eq!(b, "hello");
/// ```
pub struct RThreadScope {
    work_items: Vec<Box<dyn FnOnce() -> Box<dyn Any + Send> + Send>>,
}

impl RThreadScope {
    /// Create a new empty scope.
    pub fn new() -> Self {
        Self {
            work_items: Vec::new(),
        }
    }

    /// Queue a closure to run on the R main thread.
    ///
    /// Returns an index that can be used to retrieve the result from the
    /// `Vec` returned by [`execute`](RThreadScope::execute).
    pub fn push<F, T>(&mut self, f: F) -> usize
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let idx = self.work_items.len();
        self.work_items
            .push(Box::new(move || Box::new(f()) as Box<dyn Any + Send>));
        idx
    }

    /// Execute all queued closures in a single round-trip to the R main thread.
    ///
    /// Returns results in the order they were pushed. Each result is a
    /// `Box<dyn Any + Send>` that can be downcast to the original return type.
    ///
    /// If any closure panics, remaining closures are skipped and the panic
    /// propagates.
    pub fn execute(self) -> Vec<Box<dyn Any + Send>> {
        with_r_thread(move || self.work_items.into_iter().map(|f| f()).collect())
    }
}

impl Default for RThreadScope {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize the miniextendr worker thread infrastructure.
///
/// # Requirements
///
/// This function **MUST** be called from R's main thread, typically from the
/// `R_init_<pkgname>` function in `entrypoint.c`. Calling from any other thread
/// will cause all subsequent thread checks to be incorrect, leading to unsafe
/// R API calls from wrong threads.
///
/// # Panics
///
/// Panics if called when `R_MAIN_THREAD_ID` was already set to a different thread,
/// as this indicates incorrect initialization order.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn miniextendr_worker_init() {
    static RUN_ONCE: std::sync::Once = std::sync::Once::new();
    RUN_ONCE.call_once_force(|x| {
        // On poisoned retry, attempt full re-initialization instead of
        // returning early. The previous init panicked before completing,
        // so worker infrastructure may be missing.
        if x.is_poisoned() {
            eprintln!(
                "warning: miniextendr worker init is retrying after a previous failed attempt"
            );
        }

        // Safety check: if R_MAIN_THREAD_ID was already set, verify it's the same thread
        let current_id = std::thread::current().id();
        if let Some(&existing_id) = R_MAIN_THREAD_ID.get() {
            if existing_id != current_id {
                panic!(
                    "miniextendr_worker_init called from thread {:?}, but R_MAIN_THREAD_ID \
                     was already set to {:?}. This indicates incorrect initialization order.",
                    current_id, existing_id
                );
            }
            // Thread ID already correct; fall through to ensure worker is set up
        } else {
            let _ = R_MAIN_THREAD_ID.set(current_id);
        }

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
    });
}

// ---------------------------------------------------------------------------
// Unit tests for failure paths
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // PanicError tests
    // -----------------------------------------------------------------------

    #[test]
    fn panic_error_from_str_payload() {
        let payload: Box<dyn Any + Send> = Box::new("something broke");
        let err =
            PanicError::from_panic_payload(payload, crate::panic_telemetry::PanicSource::Worker);
        assert_eq!(err.message(), "something broke");
        assert_eq!(err.source(), crate::panic_telemetry::PanicSource::Worker);
        assert_eq!(err.payload_type(), "&str");
        assert_eq!(err.to_string(), "[Worker] something broke");
    }

    #[test]
    fn panic_error_from_string_payload() {
        let payload: Box<dyn Any + Send> = Box::new(String::from("owned message"));
        let err =
            PanicError::from_panic_payload(payload, crate::panic_telemetry::PanicSource::Altrep);
        assert_eq!(err.message(), "owned message");
        assert_eq!(err.source(), crate::panic_telemetry::PanicSource::Altrep);
        assert_eq!(err.payload_type(), "String");
    }

    #[test]
    fn panic_error_from_unknown_payload() {
        let payload: Box<dyn Any + Send> = Box::new(42i32);
        let err = PanicError::from_panic_payload(
            payload,
            crate::panic_telemetry::PanicSource::UnwindProtect,
        );
        assert_eq!(err.message(), "unknown panic");
        assert_eq!(err.payload_type(), "unknown");
    }

    #[test]
    fn panic_error_from_message() {
        let err = PanicError::from_message(
            "direct message".to_string(),
            crate::panic_telemetry::PanicSource::Connection,
        );
        assert_eq!(err.message(), "direct message");
        assert_eq!(
            err.source(),
            crate::panic_telemetry::PanicSource::Connection
        );
        assert_eq!(err.into_message(), "direct message");
    }

    #[test]
    fn panic_error_debug_format() {
        let err = PanicError::from_message(
            "test".to_string(),
            crate::panic_telemetry::PanicSource::Worker,
        );
        let debug = format!("{:?}", err);
        assert!(debug.contains("PanicError"));
        assert!(debug.contains("test"));
        assert!(debug.contains("Worker"));
    }

    #[test]
    fn panic_error_implements_error_trait() {
        let err = PanicError::from_message(
            "trait check".to_string(),
            crate::panic_telemetry::PanicSource::Worker,
        );
        // Verify it can be used as &dyn std::error::Error
        let _: &dyn std::error::Error = &err;
    }

    // -----------------------------------------------------------------------
    // payload_type_name tests
    // -----------------------------------------------------------------------

    #[test]
    fn payload_type_name_str() {
        let p: Box<dyn Any + Send> = Box::new("hello");
        assert_eq!(payload_type_name(p.as_ref()), "&str");
    }

    #[test]
    fn payload_type_name_string() {
        let p: Box<dyn Any + Send> = Box::new(String::from("hello"));
        assert_eq!(payload_type_name(p.as_ref()), "String");
    }

    #[test]
    fn payload_type_name_other() {
        let p: Box<dyn Any + Send> = Box::new(1.234f64);
        assert_eq!(payload_type_name(p.as_ref()), "unknown");
    }

    // -----------------------------------------------------------------------
    // assert_r_main_thread tests
    // -----------------------------------------------------------------------

    #[test]
    fn assert_r_main_thread_panics_when_uninitialized() {
        // R_MAIN_THREAD_ID hasn't been set, so is_r_main_thread() returns false.
        // In debug builds (which test runs are), this should panic.
        let result = std::panic::catch_unwind(|| {
            assert_r_main_thread("test_fn");
        });
        // In debug builds, this must panic
        if cfg!(debug_assertions) {
            assert!(result.is_err(), "should panic in debug mode");
        }
    }

    // -----------------------------------------------------------------------
    // Worker channel / failure path tests (no R required)
    // -----------------------------------------------------------------------

    #[test]
    fn panic_payload_to_string_handles_str() {
        let payload: Box<dyn Any + Send> = Box::new("test message");
        assert_eq!(panic_payload_to_string(&payload), "test message");
    }

    #[test]
    fn panic_payload_to_string_handles_string() {
        let payload: Box<dyn Any + Send> = Box::new(String::from("owned"));
        assert_eq!(panic_payload_to_string(&payload), "owned");
    }

    #[test]
    fn panic_payload_to_string_handles_unknown() {
        let payload: Box<dyn Any + Send> = Box::new(42u64);
        assert_eq!(panic_payload_to_string(&payload), "unknown panic");
    }

    #[test]
    fn with_r_thread_panics_before_init() {
        // Calling with_r_thread before miniextendr_worker_init should panic
        // with a descriptive message. We can't call init here (needs R), but
        // we can verify the panic path.
        let result = std::panic::catch_unwind(|| {
            with_r_thread(|| 42);
        });
        assert!(result.is_err());
        let payload = result.unwrap_err();
        let msg = crate::unwind_protect::panic_payload_to_string(payload.as_ref());
        assert!(
            msg.contains("miniextendr_worker_init"),
            "expected init error message, got: {msg}"
        );
    }

    #[test]
    fn has_worker_context_false_outside_worker() {
        // Outside of run_on_worker, there should be no worker context
        assert!(!has_worker_context());
    }

    #[test]
    fn sendable_is_send() {
        fn assert_send<T: Send>() {}
        // Verify Sendable makes non-Send types Send at the type level
        assert_send::<Sendable<*const u8>>();
    }

    /// Test that dropping a SyncSender for the response channel (simulating
    /// main thread disappearing) produces a recv error on the worker side.
    /// This validates that the channel invariants detect broken pipes.
    #[test]
    fn response_channel_closed_detected() {
        let (tx, rx) = mpsc::sync_channel::<MainThreadResponse>(1);
        drop(tx); // simulate main thread gone
        let result = rx.recv();
        assert!(result.is_err(), "recv should fail when sender is dropped");
    }

    /// Test that dropping the worker_rx (main side) causes worker_tx.send to fail.
    /// This validates the "worker thread dead" detection path.
    #[test]
    fn worker_channel_closed_detected() {
        let (tx, rx) = mpsc::sync_channel::<TypeErasedWorkerMessage>(1);
        drop(rx); // simulate main thread dropped receiver
        let result = tx.send(WorkerMessage::Done(Ok(Box::new(42i32))));
        assert!(result.is_err(), "send should fail when receiver is dropped");
    }

    /// Test that the job channel detects a dead worker thread.
    #[test]
    fn job_channel_dead_worker_detected() {
        let (tx, rx) = mpsc::sync_channel::<AnyJob>(0);
        drop(rx); // simulate worker thread exited
        let job: AnyJob = Box::new(|| {});
        let result = tx.send(job);
        assert!(result.is_err(), "send should fail when worker is dead");
    }

    // -----------------------------------------------------------------------
    // Batching API tests (no R required)
    // -----------------------------------------------------------------------

    #[test]
    fn with_r_thread_batch_panics_before_init() {
        // Like with_r_thread, batch should panic before worker init
        let result = std::panic::catch_unwind(|| {
            with_r_thread_batch(vec![Box::new(|| 42) as Box<dyn FnOnce() -> i32 + Send>]);
        });
        assert!(result.is_err());
        let payload = result.unwrap_err();
        let msg = crate::unwind_protect::panic_payload_to_string(payload.as_ref());
        assert!(
            msg.contains("miniextendr_worker_init"),
            "expected init error message, got: {msg}"
        );
    }

    #[test]
    fn with_r_thread_batch_empty_vec() {
        // Empty batch should also panic before init (goes through with_r_thread)
        let result = std::panic::catch_unwind(|| {
            with_r_thread_batch::<Box<dyn FnOnce() -> i32 + Send>, i32>(vec![]);
        });
        assert!(result.is_err());
    }

    #[test]
    fn r_thread_scope_new_is_empty() {
        let scope = RThreadScope::new();
        // Execute an empty scope — still panics before init, but validates construction
        let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
            scope.execute();
        }));
        assert!(result.is_err());
    }

    #[test]
    fn r_thread_scope_push_returns_sequential_indices() {
        let mut scope = RThreadScope::new();
        let idx0 = scope.push(|| 1);
        let idx1 = scope.push(|| 2);
        let idx2 = scope.push(|| 3);
        assert_eq!(idx0, 0);
        assert_eq!(idx1, 1);
        assert_eq!(idx2, 2);
    }

    #[test]
    fn r_thread_scope_default_trait() {
        let scope = RThreadScope::default();
        // Just verifies Default is implemented
        let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
            scope.execute();
        }));
        assert!(result.is_err()); // panics before init, expected
    }
}
