//! [`WorkerPump<T>`] — safe main/worker thread coordination for FFI bodies.
//!
//! # Overview
//!
//! `WorkerPump` manages the common pattern of running a CPU-bound worker on a
//! background thread while the main R thread drives a "pump" loop — processing
//! progress events, rendering output, or any other operation that must happen
//! on R's main thread (e.g. calls into R's C API).
//!
//! It wraps [`std::thread::scope`] so the worker is always joined before
//! `run` returns, and exposes a builder API for common knobs (channel capacity,
//! log-drain cadence).
//!
//! # R exits in the pump
//!
//! Call `WorkerPump::run` from an `#[miniextendr]` body, on R's main thread,
//! and have the pump call R only through the APIs that turn an R exit into a
//! Rust unwind: [`RCall::eval_with_handlers`] /
//! [`RCall::try_eval_with_handlers`] (and `Quoted::eval` /
//! `Quosure::eval_tidy`, which use them).
//!
//! When R leaves such a call early (an error, a `tryCatch()` exit, an
//! interrupt while R runs), it unwinds as a Rust panic carrying the R exit.
//! The unwind runs through `run`: the receiver drops, so the worker's next
//! `send` returns `Err`, and [`std::thread::scope`] joins the worker before
//! the unwind leaves it. The `#[miniextendr]` boundary then continues the R
//! exit, so the condition reaches the function's caller.
//!
//! R must never `longjmp` over `run` instead: a bare `Rf_eval` / `Rf_error`
//! in the pump jumps past `run`'s frames without running their destructors,
//! so the worker, which borrows from them, is never joined.
//!
//! The worker should stop when `send` returns `Err`: one that keeps computing
//! delays the R exit until it returns.
//!
//! [`RCall::eval_with_handlers`]: crate::expression::RCall::eval_with_handlers
//! [`RCall::try_eval_with_handlers`]: crate::expression::RCall::try_eval_with_handlers
//!
//! # Error type
//!
//! `WorkerPump::run` uses `Result<R, Box<dyn Error + Send + Sync>>` so it
//! composes naturally with both `anyhow::Result` (via `?`) and `std::io::Error`
//! without requiring a hard dependency on any error-handling crate.
//!
//! # Example
//!
//! ```rust,ignore
//! use miniextendr_api::pump::WorkerPump;
//! use std::sync::mpsc::SyncSender;
//!
//! #[miniextendr]
//! fn compress_files(paths: Vec<String>) -> i64 {
//!     WorkerPump::new()
//!         .run(
//!             // worker: runs off-main-thread
//!             |tx: SyncSender<u64>| -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
//!                 let mut total = 0i64;
//!                 for path in &paths {
//!                     let bytes = compress_one(path)?;
//!                     tx.send(bytes).ok();
//!                     total += bytes as i64;
//!                 }
//!                 Ok(total)
//!             },
//!             // pump: runs on main thread, may call R API
//!             |bytes| render_progress(bytes),
//!         )
//!         .expect("compression failed")
//! }
//! ```

use std::error::Error;
use std::marker::PhantomData;
use std::sync::mpsc::{self, SyncSender};
use std::thread;

/// Boxed, thread-safe error type used by [`WorkerPump::run`].
///
/// Alias for `Box<dyn Error + Send + Sync>`. Compatible with `anyhow::Error`
/// via `?` and with standard library error types without requiring extra
/// dependencies.
pub type WorkerError = Box<dyn Error + Send + Sync>;

/// Runs a worker thread in parallel with a main-thread pump loop.
///
/// See [the module documentation][self] for how an R exit in the pump stops
/// the worker, and for a usage example.
pub struct WorkerPump<T> {
    /// Capacity of the bounded MPSC channel between worker and pump.
    capacity: usize,
    /// Whether to drain the cross-thread log queue on every pump tick.
    drain_logs_each_tick: bool,
    _marker: PhantomData<fn() -> T>,
}

impl<T: Send + 'static> WorkerPump<T> {
    /// Create a new `WorkerPump` with default settings.
    ///
    /// Defaults:
    /// - channel capacity: 64
    /// - `drain_logs_each_tick`: `true`
    pub fn new() -> Self {
        Self {
            capacity: 64,
            drain_logs_each_tick: true,
            _marker: PhantomData,
        }
    }

    /// Set the capacity of the bounded MPSC channel.
    ///
    /// The default is 64.  A larger capacity allows the worker to get further
    /// ahead of the pump; a capacity of 0 makes every send synchronous
    /// (rendezvous channel).
    ///
    /// When the channel is full the worker blocks on `tx.send` until the pump
    /// drains a slot.  If the pump panics, or an R exit unwinds out of it,
    /// `rx` is dropped as part of scope unwinding, which unblocks `tx.send`
    /// with an `Err` and lets the worker exit cleanly.
    pub fn channel_capacity(mut self, n: usize) -> Self {
        self.capacity = n;
        self
    }

    /// Control whether the cross-thread log queue is drained on every pump tick.
    ///
    /// Default: `true`.  Set to `false` if the consumer manages its own log
    /// drain cadence (e.g. it calls `drain_log_queue()` explicitly at
    /// coarser granularity).
    ///
    /// Has no effect when the `log` feature is disabled.
    pub fn drain_logs_each_tick(mut self, on: bool) -> Self {
        self.drain_logs_each_tick = on;
        self
    }

    /// Run the worker/pump pair and return the worker's result.
    ///
    /// - `worker` runs on a scoped background thread.  It receives a
    ///   [`SyncSender<T>`] and sends messages to the pump.  When `worker`
    ///   returns (success or error) it should drop `tx`; the pump's receive
    ///   loop then terminates naturally.
    /// - `pump` is called on the **current (main R) thread** for every message
    ///   the worker sends.
    ///
    /// `run` returns `Ok(R)` on success, or `Err` if the worker returned an
    /// error or panicked.
    ///
    /// # Panics
    ///
    /// If the worker thread panics, `run` returns
    /// `Err("WorkerPump worker panicked")`.
    ///
    /// If the pump closure panics (an R exit through
    /// [`RCall::eval_with_handlers`](crate::expression::RCall::eval_with_handlers)
    /// included), [`std::thread::scope`] joins the worker and the panic then
    /// propagates out of `run`. The `#[miniextendr]` boundary turns a panic
    /// into an R error and continues an R exit.
    pub fn run<R, W, P>(self, worker: W, mut pump: P) -> Result<R, WorkerError>
    where
        R: Send,
        W: FnOnce(SyncSender<T>) -> Result<R, WorkerError> + Send,
        P: FnMut(T),
    {
        thread::scope(|scope| {
            let (tx, rx) = mpsc::sync_channel(self.capacity);
            let handle = scope.spawn(move || worker(tx));
            for msg in rx {
                if self.drain_logs_each_tick {
                    #[cfg(feature = "log")]
                    crate::optionals::log_impl::drain_log_queue();
                }
                pump(msg);
            }
            handle
                .join()
                .map_err(|_| -> WorkerError { "WorkerPump worker panicked".into() })?
        })
    }
}

impl<T: Send + 'static> Default for WorkerPump<T> {
    fn default() -> Self {
        Self::new()
    }
}

// region: Tests

#[cfg(test)]
mod tests;

// endregion
