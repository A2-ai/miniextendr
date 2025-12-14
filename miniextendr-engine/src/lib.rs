//! miniextendr-engine: R runtime initialization and embedding
//!
//! This crate provides utilities for embedding the R runtime in Rust applications.
//! It handles initialization, configuration, and lifecycle management of R.
//!
//! ## Features
//!
//! - Initialize R runtime with custom arguments
//! - Configure R environment variables
//! - Manage R event loop
//! - Clean shutdown
//!
//! ## Safety
//!
//! This crate uses non-API R internals for runtime initialization. All functions
//! are inherently unsafe as they manipulate global R state.
//!
//! ## Example
//!
//! ```ignore
//! use miniextendr_engine::REngine;
//!
//! fn main() {
//!     let mut engine = REngine::new()
//!         .with_args(&["R", "--quiet", "--vanilla"])
//!         .init()
//!         .expect("Failed to initialize R");
//!
//!     // Use R here...
//!
//!     engine.shutdown();
//! }
//! ```

use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use std::process::Command;

unsafe extern "C" {
    // R initialization (from Rembedded.h)
    fn Rf_initialize_R(argc: c_int, argv: *mut *mut c_char) -> c_int;
    fn Rf_endEmbeddedR(fatal: c_int);

    // R event loop
    fn R_ProcessEvents();
    fn R_CheckUserInterrupt();

    // Setup functions
    fn setup_Rmainloop();

    // Global state
    static mut R_Interactive: c_int;
    static mut R_SignalHandlers: c_int;
}

/// Builder for configuring and initializing the R runtime.
///
/// # Example
///
/// ```ignore
/// let engine = REngine::new()
///     .with_args(&["R", "--quiet", "--no-save"])
///     .interactive(false)
///     .signal_handlers(false)
///     .init()?;
/// ```
pub struct REngineBuilder {
    args: Vec<String>,
    interactive: bool,
    signal_handlers: bool,
}

impl Default for REngineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl REngineBuilder {
    /// Create a new R engine builder with default settings.
    pub fn new() -> Self {
        Self {
            // Default to a non-interactive-safe setup: R requires an explicit
            // save/no-save choice when not running interactively.
            args: vec![
                "R".to_string(),
                "--quiet".to_string(),
                "--vanilla".to_string(),
            ],
            interactive: false,
            signal_handlers: false,
        }
    }

    /// Set the command-line arguments for R initialization.
    ///
    /// Default is `["R", "--quiet"]`.
    pub fn with_args(mut self, args: &[&str]) -> Self {
        self.args = args.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Set whether R should run in interactive mode.
    ///
    /// Default is `false`.
    pub fn interactive(mut self, interactive: bool) -> Self {
        self.interactive = interactive;
        self
    }

    /// Set whether R should install signal handlers.
    ///
    /// Default is `false`. Set to `true` if you want R to handle Ctrl+C etc.
    pub fn signal_handlers(mut self, enable: bool) -> Self {
        self.signal_handlers = enable;
        self
    }

    /// Initialize the R runtime with the configured settings.
    ///
    /// # Safety
    ///
    /// - Must only be called once per process
    /// - Must be called from the main thread
    /// - R cannot be safely shutdown and reinitialized
    ///
    /// # Errors
    ///
    /// Returns an error if R initialization fails.
    pub unsafe fn init(self) -> Result<REngine, REngineError> {
        ensure_r_home_env()?;

        // Convert args to C strings
        let c_args: Vec<CString> = self
            .args
            .iter()
            .map(|s| CString::new(s.as_str()).unwrap())
            .collect();

        let mut c_ptrs: Vec<*mut c_char> =
            c_args.iter().map(|s| s.as_ptr() as *mut c_char).collect();

        let argc = c_ptrs.len() as c_int;
        let argv = c_ptrs.as_mut_ptr();

        // Initialize R.
        //
        // Note: `Rf_initEmbeddedR()` already calls `setup_Rmainloop()`.
        // We want tighter control (and to avoid double-calling the setup),
        // so we call `Rf_initialize_R()` directly and then `setup_Rmainloop()`.
        let result = unsafe { Rf_initialize_R(argc, argv) };
        if result != 0 {
            return Err(REngineError::InitializationFailed);
        }

        unsafe {
            // Set global flags *after* initialization, mirroring R's own
            // `Rf_initEmbeddedR()` order (but respecting our builder flags).
            R_Interactive = if self.interactive { 1 } else { 0 };
            R_SignalHandlers = if self.signal_handlers { 1 } else { 0 };
            setup_Rmainloop();
        }

        Ok(REngine { initialized: true })
    }
}

/// Handle to an initialized R runtime.
///
/// Dropping this handle will shutdown R and run finalizers.
pub struct REngine {
    initialized: bool,
}

impl REngine {
    /// Create a new builder for configuring R initialization.
    pub fn new() -> REngineBuilder {
        REngineBuilder::new()
    }

    /// Process pending R events.
    ///
    /// Call this periodically to allow R to handle events, especially
    /// when running a long computation.
    ///
    /// # Safety
    ///
    /// Must be called from the thread that initialized R.
    pub unsafe fn process_events(&self) {
        unsafe {
            R_ProcessEvents();
        }
    }

    /// Check for user interrupts (Ctrl+C).
    ///
    /// # Safety
    ///
    /// Must be called from the thread that initialized R.
    pub unsafe fn check_interrupt(&self) {
        unsafe {
            R_CheckUserInterrupt();
        }
    }

    /// Explicitly shutdown R and run finalizers.
    ///
    /// This is called automatically on drop, but you can call it explicitly
    /// for better error handling.
    ///
    /// # Safety
    ///
    /// Must be called from the thread that initialized R.
    pub unsafe fn shutdown(mut self) {
        if self.initialized {
            unsafe {
                Rf_endEmbeddedR(0);
            }
            self.initialized = false;
        }
    }
}

impl Drop for REngine {
    fn drop(&mut self) {
        if self.initialized {
            unsafe {
                Rf_endEmbeddedR(0);
            }
        }
    }
}

/// Errors that can occur during R engine initialization.
#[derive(Debug)]
pub enum REngineError {
    /// Could not determine / set `R_HOME` for embedding.
    RHomeNotFound,
    /// R initialization failed.
    InitializationFailed,
}

impl std::fmt::Display for REngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            REngineError::RHomeNotFound => {
                write!(f, "R_HOME is not set and `R RHOME` could not be resolved")
            }
            REngineError::InitializationFailed => write!(f, "R initialization failed"),
        }
    }
}

impl std::error::Error for REngineError {}

fn ensure_r_home_env() -> Result<(), REngineError> {
    if std::env::var_os("R_HOME").is_some() {
        return Ok(());
    }

    let output = Command::new("R")
        .args(["RHOME"])
        .output()
        .map_err(|_| REngineError::RHomeNotFound)?;

    if !output.status.success() {
        return Err(REngineError::RHomeNotFound);
    }

    let r_home = String::from_utf8(output.stdout).map_err(|_| REngineError::RHomeNotFound)?;
    let r_home = r_home.trim();
    if r_home.is_empty() {
        return Err(REngineError::RHomeNotFound);
    }

    // SAFETY: We call this during single-threaded startup (before initializing
    // R and before spawning any worker threads).
    unsafe {
        std::env::set_var("R_HOME", r_home);
    }
    Ok(())
}
