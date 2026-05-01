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
//! # Longjmp-safety contract
//!
//! **`WorkerPump::run` must be called from inside an `#[miniextendr]` FFI
//! body** (or any code already wrapped by `with_r_unwind_protect`).
//!
//! The reason: the pump closure may call into R's API (e.g. to render a
//! progress bar), and R can `longjmp` out of those calls at any time — on
//! interrupt, on allocation failure, etc.  miniextendr's macro layer wraps
//! every `#[miniextendr]` body in `R_UnwindProtect` via
//! `run_r_unwind_protect`.
//!
//! The `R_UnwindProtect` strategy used by miniextendr converts R longjmps
//! into Rust panics (via `cleanup_handler` → `std::panic::panic_any`) which
//! are then caught by an outer `catch_unwind`.  Because the panic travels
//! through normal Rust stack-unwinding, **all `Drop` glue runs on the way
//! out** — including `thread::scope`'s `Drop`, which joins the worker before
//! the scope exits.  The worker sees `tx` dropped (because `rx` dropped as
//! the scope cleaned up), so any blocked `tx.send` returns `Err` and the
//! worker can exit gracefully.  The panic is then re-raised as an R error via
//! `R_ContinueUnwind`.
//!
//! If you call `WorkerPump::run` *outside* of an `#[miniextendr]` body and
//! the pump triggers an R longjmp, the longjmp will bypass Rust destructors
//! entirely and the worker thread will be leaked.
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
/// See [the module documentation][self] for the longjmp-safety contract and a
/// usage example.
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
    /// drains a slot.  If the pump panics or a longjmp fires, `rx` is dropped
    /// as part of scope unwinding, which unblocks `tx.send` with an `Err` and
    /// lets the worker exit cleanly.
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
    /// If the pump closure panics, the panic propagates normally through
    /// `thread::scope`'s `Drop` (which joins the worker), and then out of
    /// `run`.  When called from inside an `#[miniextendr]` body the outer
    /// `R_UnwindProtect` catches it and converts it to an R error.
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
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};

    // -------------------------------------------------------------------------
    // happy_path
    // -------------------------------------------------------------------------

    /// Worker sends N messages, pump receives all, worker returns Ok.
    #[test]
    fn happy_path() {
        let received = Arc::new(Mutex::new(Vec::<i64>::new()));
        let received2 = Arc::clone(&received);

        let result = WorkerPump::new()
            .drain_logs_each_tick(false)
            .run(
                |tx| {
                    for i in 0..5i64 {
                        tx.send(i).unwrap();
                    }
                    Ok(42i64)
                },
                |msg| {
                    received2.lock().unwrap().push(msg);
                },
            )
            .expect("run failed");

        assert_eq!(result, 42);
        let got = received.lock().unwrap();
        assert_eq!(*got, vec![0, 1, 2, 3, 4]);
    }

    // -------------------------------------------------------------------------
    // worker_returns_err
    // -------------------------------------------------------------------------

    /// Worker returns Err early; pump exits cleanly; run returns the worker's Err.
    #[test]
    fn worker_returns_err() {
        let pump_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let pump_count2 = Arc::clone(&pump_count);

        let result: Result<(), WorkerError> = WorkerPump::new().drain_logs_each_tick(false).run(
            |_tx| {
                // drop tx immediately, send no messages
                Err("deliberate worker error".into())
            },
            |_msg: ()| {
                pump_count2.fetch_add(1, Ordering::Relaxed);
            },
        );

        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(
            msg.contains("deliberate worker error"),
            "unexpected error: {msg}"
        );
        assert_eq!(
            pump_count.load(Ordering::Relaxed),
            0,
            "pump must not run if worker sends nothing"
        );
    }

    // -------------------------------------------------------------------------
    // pump_panics
    // -------------------------------------------------------------------------

    /// Pump callback panics; worker is joined (no thread leak).
    ///
    /// We instrument the worker with a `Drop` guard that flips an `AtomicBool`
    /// to confirm it ran after the scope unwinds.
    #[test]
    fn pump_panics() {
        // Instrument: flipped to true when the worker's guard is dropped.
        let worker_dropped = Arc::new(AtomicBool::new(false));
        let worker_dropped2 = Arc::clone(&worker_dropped);

        struct DropGuard(Arc<AtomicBool>);
        impl Drop for DropGuard {
            fn drop(&mut self) {
                self.0.store(true, Ordering::Relaxed);
            }
        }

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            WorkerPump::<u8>::new().drain_logs_each_tick(false).run(
                move |tx| {
                    let _guard = DropGuard(worker_dropped2);
                    // Send one message to ensure pump fires at least once,
                    // then block until rx is dropped (pump panic).
                    let _ = tx.send(1u8);
                    // The second send will return Err when rx drops on pump panic.
                    let _ = tx.send(2u8);
                    Ok(())
                },
                |_msg: u8| {
                    panic!("pump panic");
                },
            )
        }));

        // The outer catch_unwind should see the pump panic propagate.
        assert!(result.is_err(), "expected panic to propagate");
        // The worker's DropGuard must have run — confirms worker was joined.
        assert!(
            worker_dropped.load(Ordering::Relaxed),
            "worker Drop did not run — possible thread leak"
        );
    }

    // -------------------------------------------------------------------------
    // bounded_channel_no_deadlock_on_scope_drop
    // -------------------------------------------------------------------------

    /// Fill the bounded channel; break out of pump early; assert worker exits.
    ///
    /// The worker tries to send more messages than the channel can hold.  The
    /// pump breaks after the first message (simulating an early exit), which
    /// drops `rx`.  The worker's blocked `tx.send` must return `Err` (not
    /// deadlock), and the worker must finish.
    ///
    /// We verify no deadlock by asserting `run` completes before the test
    /// times out (Rust's test harness kills hanging tests).
    #[test]
    fn bounded_channel_no_deadlock_on_scope_drop() {
        // Use a channel capacity of 1 so the worker blocks quickly.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            WorkerPump::<u8>::new()
                .channel_capacity(1)
                .drain_logs_each_tick(false)
                .run(
                    |tx| {
                        // Try to send many messages — will block after slot 1 fills.
                        for i in 0u8..=10 {
                            if tx.send(i).is_err() {
                                // rx was dropped (pump broke out), exit cleanly.
                                break;
                            }
                        }
                        Ok(())
                    },
                    |_msg: u8| {
                        // Simulate early pump exit by panicking after first message.
                        panic!("early pump exit");
                    },
                )
        }));

        // We get a panic from the pump, but no deadlock (test would hang otherwise).
        assert!(result.is_err(), "expected pump panic");
    }

    // -------------------------------------------------------------------------
    // drain_logs_each_tick_default_on
    // -------------------------------------------------------------------------

    /// With the `log` feature enabled: a log record from the worker is flushed
    /// to the render sink by the time the pump processes the next message.
    ///
    /// Without the `log` feature this test reduces to a basic send/recv check.
    #[test]
    fn drain_logs_each_tick_default_on() {
        #[cfg(feature = "log")]
        {
            use crate::optionals::log_impl::{
                LOG_TEST_LOCK, install_r_logger, set_fake_main_thread, set_log_level, take_rendered,
            };

            // Acquire the same lock that log_impl tests use so we don't race on
            // the shared QUEUE / DROPPED / TEST_SINK globals.
            let _guard = LOG_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

            install_r_logger();
            set_log_level("trace");
            // Clear residual state from prior tests.
            take_rendered();

            let rendered_before_pump: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
            let rendered_before_pump2 = Arc::clone(&rendered_before_pump);

            // The pump loop (and drain_log_queue inside it) runs on *this* thread.
            // Mark this thread as main so drain_log_queue() actually drains.
            // The worker is a real scoped thread whose FAKE_IS_MAIN is None,
            // so is_main() → is_r_main_thread() → false → records go to queue.
            set_fake_main_thread(Some(true));

            WorkerPump::<u8>::new()
                .drain_logs_each_tick(true)
                .run(
                    |tx| {
                        // Runs on a real background thread; is_main() == false
                        // → log record is buffered in QUEUE, not rendered yet.
                        log::info!("hello from worker");
                        tx.send(1u8).ok();
                        Ok(())
                    },
                    move |msg: u8| {
                        // WorkerPump calls drain_log_queue() BEFORE calling us.
                        // Since is_main() == true on the pump thread, the queue
                        // was already drained → record is in TEST_SINK.
                        let rendered = take_rendered();
                        rendered_before_pump2.lock().unwrap().extend(rendered);
                        let _ = msg;
                    },
                )
                .expect("run failed");

            // Restore state.
            set_fake_main_thread(None);
            take_rendered();

            // The drain happened before pump(msg), so the record must have
            // appeared in the captured snapshot.
            let rendered = rendered_before_pump.lock().unwrap();
            assert!(
                rendered.iter().any(|m| m.contains("hello from worker")),
                "log record was not drained before pump tick; got: {rendered:?}"
            );
        }

        // When `log` feature is disabled: just verify run completes.
        #[cfg(not(feature = "log"))]
        {
            let result: Result<u8, WorkerError> = WorkerPump::new().drain_logs_each_tick(true).run(
                |tx| {
                    tx.send(99u8).ok();
                    Ok(99u8)
                },
                |_| {},
            );
            assert_eq!(result.unwrap(), 99u8);
        }
    }
}

// endregion
