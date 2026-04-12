//! Async handle for background Rust computations.
//!
//! `MxAsyncHandle` is returned to R when a `#[miniextendr(async)]` function is called.
//! It wraps a one-shot channel receiver and provides non-blocking status checks
//! and blocking result collection.
//!
//! The handle is stored as an `ExternalPtr` in R, so R's GC manages its lifetime.
//! If R garbage-collects the handle before the background thread completes,
//! the sender gets a `SendError` — this is safe (the result is simply dropped).

use std::any::Any;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use crate::externalptr::TypedExternal;

/// Type-erased result from an async computation.
///
/// `Ok(Box<dyn Any + Send>)` holds the successful result (downcast by the generated
/// `$value()` C wrapper which knows the concrete type).
/// `Err(String)` holds a panic message or error description.
pub type AsyncResult = Result<Box<dyn Any + Send>, String>;

/// Handle for an in-flight async computation, visible to R via `ExternalPtr`.
///
/// Created by the generated C wrapper for `#[miniextendr(async)]` functions.
/// The R wrapper exposes `$is_resolved()` and `$value()` methods.
pub struct MxAsyncHandle {
    /// Set to `true` when the background thread sends its result.
    resolved: Arc<AtomicBool>,
    /// One-shot receiver for the background thread's result.
    /// `None` after the result has been consumed.
    receiver: Mutex<Option<mpsc::Receiver<AsyncResult>>>,
    /// Cached result after first `collect_result()` call.
    cached: Mutex<Option<AsyncResult>>,
}

impl MxAsyncHandle {
    /// Create a new handle paired with a sender for the background thread.
    ///
    /// Returns `(handle, sender)`. The background thread should call
    /// `sender.send(result)` exactly once when the computation completes.
    pub fn new() -> (Self, AsyncSender) {
        let (tx, rx) = mpsc::sync_channel::<AsyncResult>(1);
        let resolved = Arc::new(AtomicBool::new(false));
        let resolved_clone = Arc::clone(&resolved);
        (
            Self {
                resolved,
                receiver: Mutex::new(Some(rx)),
                cached: Mutex::new(None),
            },
            AsyncSender {
                sender: tx,
                resolved: resolved_clone,
            },
        )
    }

    /// Non-blocking check: has the background thread completed?
    pub fn is_resolved(&self) -> bool {
        // First check the atomic flag (set by AsyncSender::send)
        if self.resolved.load(Ordering::Acquire) {
            return true;
        }
        // Also check if the receiver has data (handles edge case where
        // sender dropped without sending — e.g. thread panicked past catch_unwind)
        let rx_guard = self.receiver.lock().unwrap();
        if let Some(rx) = rx_guard.as_ref() {
            // try_recv is non-blocking
            match rx.try_recv() {
                Ok(_) => {
                    // Data available but we can't consume it here without moving
                    // out of the mutex. Just report resolved.
                    // The actual data will be collected by collect_result().
                    true
                }
                Err(mpsc::TryRecvError::Disconnected) => true,
                Err(mpsc::TryRecvError::Empty) => false,
            }
        } else {
            // Receiver already consumed → result was already collected
            true
        }
    }

    /// Block until the result is available, then return it.
    ///
    /// Returns the type-erased result. The caller (generated C wrapper) must
    /// downcast to the concrete type and convert via `IntoR`.
    ///
    /// Can only be called once — subsequent calls return an error.
    pub fn collect_result(&self) -> AsyncResult {
        // Check cache first (in case is_resolved + collect_result race)
        {
            let mut cached = self.cached.lock().unwrap();
            if let Some(result) = cached.take() {
                return result;
            }
        }

        // Take the receiver (one-shot consumption)
        let rx = {
            let mut rx_guard = self.receiver.lock().unwrap();
            rx_guard.take()
        };

        match rx {
            Some(rx) => {
                // Block until the background thread sends
                let result = rx
                    .recv()
                    .unwrap_or_else(|_| Err("async task failed: channel closed".into()));
                self.resolved.store(true, Ordering::Release);
                result
            }
            None => Err("async result already consumed".into()),
        }
    }
}

impl TypedExternal for MxAsyncHandle {
    const TYPE_NAME: &'static str = "MxAsyncHandle";
    const TYPE_NAME_CSTR: &'static [u8] = b"MxAsyncHandle\0";
    const TYPE_ID_CSTR: &'static [u8] = concat!(module_path!(), "::MxAsyncHandle\0").as_bytes();
}

/// Sender half given to the background thread.
///
/// Wraps `mpsc::SyncSender` and sets the `resolved` flag atomically
/// after sending, so `is_resolved()` returns `true` immediately.
pub struct AsyncSender {
    sender: mpsc::SyncSender<AsyncResult>,
    resolved: Arc<AtomicBool>,
}

impl AsyncSender {
    /// Send the computation result and mark the handle as resolved.
    ///
    /// If the receiver (R-side handle) has been dropped, the result is silently
    /// discarded — this is expected when R GC's the handle before completion.
    pub fn send(self, result: AsyncResult) {
        let _ = self.sender.send(result);
        self.resolved.store(true, Ordering::Release);
    }
}
