//! Route Rust `log` crate macros to R's console output.
//!
//! When the `log` feature is enabled, this module provides an R-aware logger
//! that routes `log::info!()`, `log::warn!()`, `log::error!()` etc. to R's
//! console via `Rprintf` / `REprintf`.
//!
//! # Thread safety
//!
//! The logger only outputs on R's main thread. Messages logged from other
//! threads are silently dropped (R API calls are not thread-safe).
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
use std::ffi::CString;

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

        // Only output on R's main thread — R API is not thread-safe.
        if !crate::worker::is_r_main_thread() {
            return;
        }

        let msg = format!("{}\n", record.args());
        let Ok(cmsg) = CString::new(msg) else {
            return; // message contains null byte, skip
        };

        unsafe {
            let fmt = c"%s".as_ptr();
            match record.level() {
                Level::Error | Level::Warn => {
                    crate::ffi::REprintf_unchecked(fmt, cmsg.as_ptr());
                }
                Level::Info | Level::Debug | Level::Trace => {
                    crate::ffi::Rprintf_unchecked(fmt, cmsg.as_ptr());
                }
            }
        }
    }

    fn flush(&self) {
        if crate::worker::is_r_main_thread() {
            unsafe {
                crate::ffi::R_FlushConsole_unchecked();
            }
        }
    }
}

/// Install the R console logger.
///
/// Call this once during package initialization (from `package_init()`).
/// If a logger is already installed (by another package or the user),
/// this is a no-op.
///
/// Default level: `Info` (shows info, warn, error; hides debug, trace).
pub fn install_r_logger() {
    static LOGGER: RLogger = RLogger;
    // ok() — ignore AlreadySet error if another logger is installed
    log::set_logger(&LOGGER).ok();
    log::set_max_level(LevelFilter::Info);
}

/// Set the log level filter from a string.
///
/// Valid levels: "error", "warn", "info", "debug", "trace", "off".
/// Invalid strings default to "info".
pub fn set_log_level(level: &str) {
    let filter = match level {
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        "off" => LevelFilter::Off,
        _ => LevelFilter::Info,
    };
    log::set_max_level(filter);
}
