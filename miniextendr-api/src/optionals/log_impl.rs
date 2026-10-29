//! Route Rust `log` crate macros to R's console output.
//!
//! When the `log` feature is enabled, this module provides an R-aware logger
//! that routes `log::info!()`, `log::warn!()`, `log::error!()` etc. to R's
//! console via `Rprintf` / `REprintf`.
//!
//! # Thread safety
//!
//! The logger is safe to call from any thread. Records logged from non-main
//! threads are buffered in a bounded MPSC queue (capacity 1024) and drained
//! to R's console the next time the main thread passes through the FFI
//! trampoline. Main-thread records drain the queue first, then render
//! immediately.
//!
//! If the queue fills up, the oldest record is dropped and an overflow counter
//! is incremented. The next call to `drain_log_queue()` from the main thread
//! emits a single `WARN`-level message ("N log records dropped due to queue
//! overflow") and resets the counter to 0.
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
//! thread.
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
use std::collections::VecDeque;
#[cfg(not(test))]
use std::ffi::CString;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};

/// Capacity of the cross-thread log queue.
const QUEUE_CAPACITY: usize = 1024;

/// Buffered records from non-main threads, pending drain on the next main-thread FFI exit.
static QUEUE: OnceLock<Mutex<VecDeque<(Level, String)>>> = OnceLock::new();

/// Number of records dropped due to queue overflow since the last drain.
static DROPPED: AtomicUsize = AtomicUsize::new(0);

fn queue() -> &'static Mutex<VecDeque<(Level, String)>> {
    QUEUE.get_or_init(|| Mutex::new(VecDeque::with_capacity(QUEUE_CAPACITY)))
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
/// `worker::is_r_main_thread()` otherwise.
#[cfg(not(test))]
#[inline(always)]
fn is_main() -> bool {
    crate::worker::is_r_main_thread()
}

#[cfg(test)]
fn is_main() -> bool {
    FAKE_IS_MAIN
        .with(|c| c.get())
        .unwrap_or_else(crate::worker::is_r_main_thread)
}

/// Override the main-thread predicate for the duration of the current test.
///
/// Call with `None` to restore the real predicate.
#[cfg(test)]
pub(crate) fn set_fake_main_thread(v: Option<bool>) {
    FAKE_IS_MAIN.with(|c| c.set(v));
}

// endregion

// region: Render abstraction (cfg(test) indirection)

/// Write a formatted log message to R's console.
///
/// In production: calls `Rprintf` (stdout) or `REprintf` (stderr) based on
/// level. In tests: appends to a `Mutex<Vec<String>>` sink to avoid calling
/// the R FFI.
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
pub(crate) static TEST_SINK: Mutex<Vec<String>> = Mutex::new(Vec::new());

#[cfg(test)]
fn render(_level: Level, msg: &str) {
    TEST_SINK.lock().unwrap().push(msg.to_string());
}

#[cfg(test)]
pub(crate) fn take_rendered() -> Vec<String> {
    std::mem::take(&mut *TEST_SINK.lock().unwrap())
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
            // On the main thread: drain any buffered worker records first,
            // then render this record immediately.
            drain_log_queue();
            render(record.level(), &msg);
        } else {
            // On a worker thread: push to the bounded queue.
            let mut q = queue().lock().unwrap();
            if q.len() >= QUEUE_CAPACITY {
                q.pop_front(); // drop oldest
                DROPPED.fetch_add(1, Ordering::Relaxed);
            }
            q.push_back((record.level(), msg));
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
/// Call this once during package initialization (from `package_init()`).
/// If a logger is already installed (by another package or the user),
/// this is a no-op.
///
/// Default level: `Off` (all output suppressed until the downstream package
/// calls `set_log_level("info")` or similar).
pub fn install_r_logger() {
    static LOGGER: RLogger = RLogger;
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

/// Drain all buffered cross-thread log records to R's console.
///
/// Must be called from R's main thread — silently returns otherwise.
/// Idempotent: if the queue is empty, this is a no-op.
///
/// If records were dropped due to queue overflow, a single `WARN`-level
/// message ("N log records dropped due to queue overflow") is emitted first,
/// and the overflow counter is reset to zero.
///
/// The FFI trampoline in `unwind_protect.rs` calls this automatically on every
/// FFI exit path (normal return, Rust panic, and caught R longjmp). Direct
/// callers are rare but supported (e.g. end-of-batch flush points).
pub fn drain_log_queue() {
    if !is_main() {
        return;
    }

    // Emit overflow warning before the buffered records.
    let dropped = DROPPED.swap(0, Ordering::Relaxed);
    if dropped > 0 {
        render(
            Level::Warn,
            &format!("{dropped} log records dropped due to queue overflow"),
        );
    }

    // Drain and render all buffered records.
    let records: Vec<(Level, String)> = {
        let mut q = queue().lock().unwrap();
        std::mem::take(&mut *q).into_iter().collect()
    };
    for (level, msg) in records {
        render(level, &msg);
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

/// Serialises all tests that touch the shared `QUEUE`/`DROPPED`/`TEST_SINK`
/// globals.  Exposed as `pub(crate)` so tests in sibling modules (e.g.
/// `pump`) can acquire the same lock and avoid races.
#[cfg(test)]
pub(crate) static LOG_TEST_LOCK: Mutex<()> = Mutex::new(());

#[cfg(test)]
mod tests;

// endregion
