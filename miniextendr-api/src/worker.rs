//! Worker thread for safe Rust-R FFI with longjmp protection.
//!
//! Execute Rust code on a separate worker thread. If a panic occurs,
//! `catch_unwind` catches it and the stack unwinds naturally, running all Drops.
//! The main thread then converts the result to SEXP or raises an R error.

use std::any::Any;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::OnceLock;
use std::sync::mpsc::{self, SyncSender};
use std::thread;

use crate::ffi;

static R_MAIN_THREAD_ID: OnceLock<thread::ThreadId> = OnceLock::new();

type AnyJob = Box<dyn FnOnce() + Send>;

static JOB_TX: OnceLock<SyncSender<AnyJob>> = OnceLock::new();

/// Check if the current thread is R's main thread.
#[inline(always)]
pub(crate) fn is_r_main_thread() -> bool {
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

/// Run a Rust closure on the worker thread with proper cleanup on panic.
///
/// Panics are caught and converted to R errors. Destructors run properly.
pub fn run_on_worker<F, T>(f: F) -> T
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    let job_tx = JOB_TX
        .get()
        .expect("worker not initialized - call miniextendr_worker_init first");

    let (result_tx, result_rx) = mpsc::sync_channel::<Result<T, String>>(0);

    let job: AnyJob = Box::new(move || {
        let result = catch_unwind(AssertUnwindSafe(f));
        let to_send = match result {
            Ok(val) => Ok(val),
            Err(payload) => Err(panic_payload_to_string(&payload)),
        };
        let _ = result_tx.send(to_send);
    });

    job_tx.send(job).expect("worker thread dead");

    match result_rx.recv().expect("worker channel closed") {
        Ok(val) => val,
        Err(msg) => panic_message_to_r_error(msg),
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
                loop {
                    match job_rx.recv() {
                        Ok(job) => job(),
                        Err(_) => break,
                    }
                }
            })
            .expect("failed to spawn worker thread");

        JOB_TX.set(job_tx).expect("worker already initialized");
    });
}
