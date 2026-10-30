//! Route Rust `log` crate macros to R's console output.
//!
//! When the `log` feature is enabled, this module provides an R-aware logger
//! that routes `log::info!()`, `log::warn!()`, `log::error!()` etc. to R's
//! console via `Rprintf` / `REprintf`.
//!
//! # Thread safety
//!
//! The logger is safe to call from any thread. A record logged on a thread
//! other than R's main thread (rayon, `std::thread`) is sent on a bounded
//! `std::sync::mpsc` channel (capacity 1024). Any thread sends on it; R's main
//! thread owns the receiving end, which `install_r_logger()` keeps in a
//! thread-local of the thread it runs on (`package_init()` runs it on R's main
//! thread). The main thread prints the waiting records the next time it passes
//! through the FFI trampoline, each thread's records in the order that thread
//! sent them. A record logged on the main thread prints the waiting records
//! first, then itself.
//!
//! If the channel is full, the record being sent is dropped (a sender cannot
//! remove the oldest one) and an overflow counter is incremented. The next
//! call to `drain_log_queue()` from the main thread prints the records kept,
//! then a single `WARN`-level message ("N log records dropped due to queue
//! overflow"), and resets the counter to 0.
//!
//! # Level mapping
//!
//! | Rust level | R output |
//! |------------|----------|
//! | `error!()` | `REprintf` (stderr, non-interrupting) |
//! | `warn!()`  | `REprintf` (stderr, non-interrupting) |
//! | `info!()`  | `Rprintf` (stdout/console) |
//! | `debug!()` | `Rprintf` (stdout/console) |
//! | `trace!()` | `Rprintf` (stdout/console) |
//!
//! # Default level
//!
//! `install_r_logger()` sets the max level to `Off`. Downstream packages must
//! explicitly opt in by calling `set_log_level("info")` (or any other level).
//! This prevents packages from unexpectedly producing console output when users
//! do not expect it.
//!
//! # Drain at FFI exit
//!
//! The `drain_log_queue()` function is called automatically from the unwind
//! protect trampoline (`unwind_protect.rs`) on every FFI exit path — normal
//! return, Rust panic, and R longjmp. It is a no-op when not on the main
//! thread. Each call prints at most 1024 records, so threads that keep logging
//! cannot hold R's main thread there; the rest print at the next drain.
//!
//! # Example
//!
//! ```rust,ignore
//! use log::{info, warn};
//!
//! #[miniextendr]
//! fn process(path: &str) -> Vec<f64> {
//!     info!("Loading file: {path}");
//!     warn!("3 missing values filled with NA");
//!     vec![1.0, 2.0, 3.0]
//! }
//! ```

pub use log;

use log::{Level, LevelFilter, Log, Metadata, Record};
use std::cell::OnceCell;
#[cfg(not(test))]
use std::ffi::CString;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};

/// Capacity of the cross-thread log channel.
const QUEUE_CAPACITY: usize = 1024;

/// A formatted record on its way from another thread to R's main thread.
type Entry = (Level, String);

/// Sending end of the cross-thread log channel. Any thread sends on it.
static SENDER: OnceLock<SyncSender<Entry>> = OnceLock::new();

thread_local! {
    /// Receiving end of the cross-thread log channel: set on the thread that
    /// opened the channel (R's main thread), empty on every other thread.
    static RECEIVER: OnceCell<Receiver<Entry>> = const { OnceCell::new() };
}

/// Number of records dropped because the channel was full, since the last drain.
static DROPPED: AtomicUsize = AtomicUsize::new(0);

/// Open the cross-thread log channel and keep its receiving end on the
/// current thread. Only the first call opens it.
fn open_channel() {
    SENDER.get_or_init(|| {
        let (tx, rx) = sync_channel(QUEUE_CAPACITY);
        RECEIVER.with(|cell| {
            cell.get_or_init(|| rx);
        });
        tx
    });
}

/// Send a record logged off R's main thread to the main thread.
///
/// When the channel is full (or the main thread is gone), the record is
/// dropped and counted in `DROPPED`.
fn send_to_main(level: Level, msg: String) {
    let sent = SENDER
        .get()
        .is_some_and(|tx| tx.try_send((level, msg)).is_ok());
    if !sent {
        DROPPED.fetch_add(1, Ordering::Relaxed);
    }
}

// region: Test infrastructure (cfg(test) overrides)

// In tests, `is_r_main_thread()` is overridden by a thread-local boolean so
// individual tests can pretend to be on or off the main thread without
// actually running R.
#[cfg(test)]
thread_local! {
    static FAKE_IS_MAIN: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
}

/// Whether the current thread is R's main thread.
///
/// In tests: returns the thread-local override when set; falls back to the real
/// `crate::is_r_main_thread()` otherwise.
#[cfg(not(test))]
#[inline(always)]
fn is_main() -> bool {
    crate::is_r_main_thread()
}

#[cfg(test)]
fn is_main() -> bool {
    FAKE_IS_MAIN
        .with(|c| c.get())
        .unwrap_or_else(crate::is_r_main_thread)
}

/// Override the main-thread predicate for the duration of the current test.
///
/// Call with `None` to restore the real predicate.
#[cfg(test)]
pub(crate) fn set_fake_main_thread(v: Option<bool>) {
    FAKE_IS_MAIN.with(|c| c.set(v));
}

/// Run `f` on the thread that plays R's main thread in tests, and return its
/// result.
///
/// The channel's receiving end stays on the thread that installed the logger,
/// as it stays on R's main thread in production, while cargo runs each test on
/// a thread of its own. So a test that logs or drains runs its whole body here:
/// one long-lived thread that installs the logger before any body and runs one
/// body at a time, which also keeps tests from sharing the channel, `DROPPED`
/// and the render sink at once. Each body starts as the "main thread", with
/// nothing waiting in the channel, `DROPPED` at 0 and the sink empty. A panic in
/// the body panics the caller with the same message: the runner's own panic
/// output goes to the capture of the test that started it, which may be over.
#[cfg(test)]
#[track_caller]
pub(crate) fn on_main_thread<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    use std::panic::{AssertUnwindSafe, catch_unwind};
    use std::sync::mpsc::{Sender, channel};

    type Job = Box<dyn FnOnce() + Send>;
    static JOBS: OnceLock<Sender<Job>> = OnceLock::new();
    let jobs = JOBS.get_or_init(|| {
        let (tx, rx) = channel::<Job>();
        std::thread::spawn(move || {
            install_r_logger();
            rx.into_iter().for_each(|job| job());
        });
        tx
    });

    let (tx, rx) = sync_channel(1);
    jobs.send(Box::new(move || {
        // Start clean even after a body that panicked.
        set_fake_main_thread(Some(true));
        RECEIVER.with(|cell| {
            if let Some(rx) = cell.get() {
                rx.try_iter().for_each(drop);
            }
        });
        DROPPED.store(0, Ordering::Relaxed);
        take_rendered();
        tx.send(catch_unwind(AssertUnwindSafe(f))).ok();
    }))
    .expect("the test main thread is running");
    match rx.recv().expect("the test main thread ran the body") {
        Ok(value) => value,
        Err(payload) => panic!(
            "{}",
            crate::unwind_protect::panic_payload_to_string(payload.as_ref())
        ),
    }
}

// endregion

// region: Render abstraction (cfg(test) indirection)

/// Write a formatted log message to R's console.
///
/// In production: calls `Rprintf` (stdout) or `REprintf` (stderr) based on
/// level. In tests: appends to a thread-local sink to avoid calling the R FFI
/// (rendering only happens on the main thread).
#[cfg(not(test))]
fn render(level: Level, msg: &str) {
    let line = format!("{msg}\n");
    let Ok(cmsg) = CString::new(line) else {
        return; // message contains null byte, skip
    };
    unsafe {
        let fmt = c"%s".as_ptr();
        match level {
            Level::Error | Level::Warn => {
                crate::sys::REprintf_unchecked(fmt, cmsg.as_ptr());
            }
            Level::Info | Level::Debug | Level::Trace => {
                crate::sys::Rprintf_unchecked(fmt, cmsg.as_ptr());
            }
        }
    }
}

#[cfg(test)]
thread_local! {
    static TEST_SINK: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
}

#[cfg(test)]
fn render(_level: Level, msg: &str) {
    TEST_SINK.with_borrow_mut(|sink| sink.push(msg.to_string()));
}

#[cfg(test)]
pub(crate) fn take_rendered() -> Vec<String> {
    TEST_SINK.take()
}

// endregion

// region: Logger implementation

/// R-aware logger that routes to `Rprintf`/`REprintf`.
struct RLogger;

impl Log for RLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        // Format the message immediately — `record.args()` borrows `record`
        // which cannot escape this function.
        let msg = format!("{}", record.args());

        if is_main() {
            // On the main thread: first drain the records other threads sent,
            // then render this record immediately.
            drain_log_queue();
            render(record.level(), &msg);
        } else {
            // On another thread: send to the main thread.
            send_to_main(record.level(), msg);
        }
    }

    fn flush(&self) {
        if is_main() {
            drain_log_queue();
            #[cfg(not(test))]
            unsafe {
                crate::sys::R_FlushConsole_unchecked();
            }
        }
        // No-op on non-main threads: cannot call R API.
    }
}

// endregion

// region: Public API

/// Install the R console logger.
///
/// Call this once during package initialization (from `package_init()`), on
/// R's main thread: the first call opens the cross-thread log channel and keeps
/// its receiving end on the calling thread, the only thread that drains it.
/// If a logger is already installed (by another package or the user),
/// installing this one is a no-op.
///
/// Default level: `Off` (all output suppressed until the downstream package
/// calls `set_log_level("info")` or similar).
pub fn install_r_logger() {
    static LOGGER: RLogger = RLogger;
    // Open the channel first, so the logger never runs without a sender.
    open_channel();
    // ok() — ignore AlreadySet error if another logger is installed
    log::set_logger(&LOGGER).ok();
    log::set_max_level(LevelFilter::Off);
}

/// Set the log level filter from a string.
///
/// Valid levels: "error", "warn", "info", "debug", "trace", "off"
/// (case-insensitive). Invalid strings default to `"info"`.
pub fn set_log_level(level: &str) {
    use std::str::FromStr;
    let filter = LevelFilter::from_str(level).unwrap_or(LevelFilter::Info);
    log::set_max_level(filter);
}

/// Drain the log records other threads sent to R's console.
///
/// Must be called from R's main thread — silently returns otherwise.
/// Idempotent: if nothing is waiting, this is a no-op. Prints at most 1024
/// records per call; records sent while it prints may wait for the next call.
///
/// If records were dropped because the channel was full, a single `WARN`-level
/// message ("N log records dropped due to queue overflow") is emitted after the
/// records kept (the dropped ones were sent after them), and the overflow
/// counter is reset to zero.
///
/// The FFI trampoline in `unwind_protect.rs` calls this automatically on every
/// FFI exit path (normal return, Rust panic, and caught R longjmp). Direct
/// callers are rare but supported (e.g. end-of-batch flush points).
pub fn drain_log_queue() {
    if !is_main() {
        return;
    }

    // Bounded: other threads keep sending while this renders.
    RECEIVER.with(|cell| {
        if let Some(rx) = cell.get() {
            for (level, msg) in rx.try_iter().take(QUEUE_CAPACITY) {
                render(level, &msg);
            }
        }
    });

    // Report overflow after the records kept: a full channel drops the newest.
    let dropped = DROPPED.swap(0, Ordering::Relaxed);
    if dropped > 0 {
        render(
            Level::Warn,
            &format!("{dropped} log records dropped due to queue overflow"),
        );
    }
}

// endregion

// region: MatchArg impl for LevelFilter

/// `MatchArg` implementation for [`log::LevelFilter`].
///
/// Allows downstream `#[miniextendr]` functions to accept a `log::LevelFilter`
/// argument with `#[miniextendr(match_arg)]` and have the generated R wrapper
/// validate the string via `base::match.arg(level, c("off", "error", "warn",
/// "info", "debug", "trace"))` — choices sourced from miniextendr, not
/// hand-typed at the consumer.  The first choice (`"off"`) is intentionally
/// the `match.arg` default, matching `install_r_logger()`'s default level so
/// that a caller who does not pass a level silently suppresses output rather
/// than unexpectedly enabling it.
impl crate::MatchArg for log::LevelFilter {
    /// Choices in the order produced by `log::LevelFilter::iter()`:
    /// `Off, Error, Warn, Info, Debug, Trace`.
    ///
    /// The first entry is the `match.arg` default when the R argument is `NULL`.
    /// `"off"` is intentional: a consumer that doesn't pass a level should get
    /// the silent default, matching `install_r_logger`'s default level since #339.
    const CHOICES: &'static [&'static str] = &["off", "error", "warn", "info", "debug", "trace"];

    fn from_choice(choice: &str) -> Option<Self> {
        // log::LevelFilter has a case-insensitive FromStr; we feed it
        // exactly the strings in CHOICES, but accept any casing.
        choice.parse().ok()
    }

    fn to_choice(self) -> &'static str {
        // Exhaustive match — if `log` ever adds a variant, this stops
        // compiling and forces CHOICES above to be updated in lockstep.
        match self {
            log::LevelFilter::Off => "off",
            log::LevelFilter::Error => "error",
            log::LevelFilter::Warn => "warn",
            log::LevelFilter::Info => "info",
            log::LevelFilter::Debug => "debug",
            log::LevelFilter::Trace => "trace",
        }
    }
}

/// `TryFromSexp` for [`log::LevelFilter`] — wires the `MatchArg` impl into the
/// FFI conversion layer so `#[miniextendr(match_arg)] level: log::LevelFilter`
/// compiles without requiring a local `#[derive(MatchArg)]` on the foreign type.
///
/// Conversion delegates to [`crate::match_arg_from_sexp`], which handles
/// `NULL` → first choice (default), exact and partial matching, NA, and wrong
/// SEXP type.
impl crate::TryFromSexp for log::LevelFilter {
    type Error = crate::SexpError;
    const CHARACTER_ONLY: bool = true;

    fn try_from_sexp(
        sexp: crate::SEXP,
    ) -> Result<Self, <log::LevelFilter as crate::TryFromSexp>::Error> {
        crate::match_arg_from_sexp(sexp).map_err(Into::into)
    }
}

/// `IntoR` for [`log::LevelFilter`] — converts a `LevelFilter` back to an R
/// character scalar via [`crate::MatchArg::to_choice`].  This is used when a
/// `#[miniextendr]` function returns a `log::LevelFilter` value.
impl crate::IntoR for log::LevelFilter {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<crate::SEXP, <log::LevelFilter as crate::IntoR>::Error> {
        Ok(self.into_sexp())
    }

    unsafe fn try_into_sexp_unchecked(
        self,
    ) -> Result<crate::SEXP, <log::LevelFilter as crate::IntoR>::Error> {
        self.try_into_sexp()
    }

    fn into_sexp(self) -> crate::SEXP {
        use crate::MatchArg;
        self.to_choice().into_sexp()
    }
}

// endregion

// region: Tests

#[cfg(test)]
mod tests;

// endregion
