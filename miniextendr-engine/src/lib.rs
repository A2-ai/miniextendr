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
//!     // SAFETY: Must be called once from main thread at startup.
//!     let engine = unsafe {
//!         REngine::build()
//!             .with_args(&["R", "--quiet", "--vanilla"])
//!             .init()
//!             .expect("Failed to initialize R")
//!     };
//!
//!     // Use R here...
//!
//!     // Note: No explicit shutdown needed. R cleanup is skipped intentionally
//!     // because Rf_endEmbeddedR is not reentrant-safe. The OS reclaims resources
//!     // when the process exits.
//! }
//! ```

use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use std::process::Command;

// Note: This entire crate uses non-API R functions (Rembedded.h, Rinterface.h)
// for embedding R. It is not intended for use in R packages.
unsafe extern "C" {
    // R initialization (from Rembedded.h - non-API)
    fn Rf_initialize_R(argc: c_int, argv: *mut *mut c_char) -> c_int;
    #[allow(dead_code)]
    fn Rf_endEmbeddedR(fatal: c_int);

    // R event loop
    fn R_ProcessEvents();
    fn R_CheckUserInterrupt();

    // Setup functions
    fn setup_Rmainloop();

    // Global state from Rinterface.h (non-API)
    // Use UnsafeCell for interior mutability without static mut
    static R_Interactive: std::cell::UnsafeCell<c_int>;
    static R_SignalHandlers: std::cell::UnsafeCell<c_int>;
    static R_CStackStart: usize;
    static R_CStackDir: c_int;
}

/// Write to R's global `R_Interactive` flag.
///
/// # Safety
/// Must be called from the main thread during R initialization.
#[inline]
unsafe fn set_r_interactive(value: c_int) {
    unsafe {
        *R_Interactive.get() = value;
    }
}

/// Write to R's global `R_SignalHandlers` flag.
///
/// # Safety
/// Must be called from the main thread during R initialization.
#[inline]
unsafe fn set_r_signal_handlers(value: c_int) {
    unsafe {
        *R_SignalHandlers.get() = value;
    }
}

/// Check whether `Rf_initialize_R` has run by inspecting stack sentinels.
///
/// `R_CStackStart`/`R_CStackDir` are set during R initialization on the main
/// thread. A zero or `usize::MAX` value indicates "not initialized".
#[inline]
pub fn r_initialized_sentinel() -> bool {
    unsafe {
        let start = R_CStackStart;
        let dir = R_CStackDir;
        dir != 0 && start != 0 && start != usize::MAX
    }
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
    /// Default is `["R", "--quiet", "--vanilla"]`.
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
        // Guard against re-initialization
        if r_initialized_sentinel() {
            return Err(REngineError::AlreadyInitialized);
        }

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
            set_r_interactive(if self.interactive { 1 } else { 0 });
            set_r_signal_handlers(if self.signal_handlers { 1 } else { 0 });
            setup_Rmainloop();

            // Note: We do NOT register an atexit handler for Rf_endEmbeddedR.
            // The R runtime cleanup operations (KillAllDevices, RunExitFinalizers, etc.)
            // are complex and can crash if other cleanup is happening concurrently.
            // For short-lived programs (tests, benchmarks), letting the OS reclaim
            // resources on process exit is safer and sufficient.
        }

        Ok(REngine)
    }
}

/// Handle to an initialized R runtime.
///
/// This is a marker type indicating R has been initialized for this process.
/// R cleanup (via `Rf_endEmbeddedR`) is intentionally NOT called because it
/// performs non-reentrant operations that can crash if called during Drop
/// or concurrent with other cleanup. The OS reclaims all resources on process exit.
pub struct REngine;

impl Drop for REngine {
    /// Implements drop such that `std::mem::forget` leaks `REngine` rather than
    /// dropping it, when `Drop` is absent.
    fn drop(&mut self) {}
}

impl REngine {
    /// Create a new builder for configuring R initialization.
    pub fn build() -> REngineBuilder {
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
}

// Note: We intentionally DO NOT provide shutdown or Drop implementations.
//
// Rf_endEmbeddedR performs non-reentrant cleanup operations.
// Here's what it does (from R 4.5.2 source):
//
// Unix/Linux version (src/unix/Rembedded.c):
// ```c
// void Rf_endEmbeddedR(int fatal)
// {
//     R_RunExitFinalizers();    // Runs .Last and exit handlers (NOT reentrant!)
//     CleanEd();                // Editor cleanup
//     if(!fatal) KillAllDevices();  // Graphics devices (NOT reentrant!)
//     R_CleanTempDir();         // File system cleanup
//     if(!fatal && R_CollectWarnings)
//         PrintWarnings();      // Console I/O
//     fpu_setup(FALSE);         // FPU state
// }
// ```
//
// Windows version (src/gnuwin32/embeddedR.c):
// ```c
// void Rf_endEmbeddedR(int fatal)
// {
//     R_RunExitFinalizers();
//     CleanEd();
//     R_CleanTempDir();
//     if(!fatal){
//         Rf_KillAllDevices();
//         AllDevicesKilled = TRUE;
//     }
//     if(!fatal && R_CollectWarnings)
//         PrintWarnings();
//     app_cleanup();           // Application-specific cleanup
// }
// ```
//
// These operations are NOT reentrant and must run exactly once at process exit.
// Calling during Drop (e.g., test cleanup) causes crashes.
//
// **Solution:** We intentionally do NOT call Rf_endEmbeddedR. For short-lived
// programs (tests, benchmarks), the OS reclaims all resources on process exit.
// This avoids crashes from double-cleanup or reentrant calls.

/// Errors that can occur during R engine initialization.
#[derive(Debug)]
pub enum REngineError {
    /// Could not determine / set `R_HOME` for embedding.
    RHomeNotFound,
    /// R initialization failed.
    InitializationFailed,
    /// R is already initialized. Re-initialization is not supported.
    AlreadyInitialized,
}

impl std::fmt::Display for REngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            REngineError::RHomeNotFound => {
                write!(f, "R_HOME is not set and `R RHOME` could not be resolved")
            }
            REngineError::InitializationFailed => write!(f, "R initialization failed"),
            REngineError::AlreadyInitialized => {
                write!(f, "R is already initialized. Multiple calls to REngineBuilder::init() are not supported.")
            }
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

#[cfg(test)]
mod tests;
