//! Rust handle for R's `utils::txtProgressBar`.
//!
//! [`RTxtProgressBar`] constructs and drives R's built-in text progress bar
//! entirely from Rust. The bar is created via `utils::txtProgressBar()` and
//! pinned on R's precious list for the lifetime of the struct. It
//! auto-closes on [`Drop`] (calls `pb$kill()` via `R_tryEvalSilent` so that
//! the close path cannot unwind through a Rust destructor stack).
//!
//! # Feature gate
//!
//! This module is compiled under the `connections` Cargo feature (alongside
//! `connection.rs`). Enable `connections` to use it.
//!
//! # Examples
//!
//! ```ignore
//! use miniextendr_api::txt_progress_bar::RTxtProgressBar;
//!
//! // Style-3 bar from 0 to 100 — closes automatically on drop.
//! let pb = RTxtProgressBar::builder(0.0, 100.0).style(3).build();
//! for i in 0..=100 {
//!     pb.set(i as f64).ok();
//! }
//! // pb drops here; auto-close fires.
//! ```

use std::marker::PhantomData;

use crate::ffi::{R_PreserveObject, R_ReleaseObject, Rf_protect, Rf_unprotect, SEXP, SexpExt};

// region: RTxtProgressBar struct

/// A Rust-owned handle to an R `utils::txtProgressBar`.
///
/// Constructed via [`RTxtProgressBar::builder`] → [`RTxtProgressBarBuilder::build`].
/// The bar is auto-closed when the struct is dropped; call [`RTxtProgressBar::close`]
/// to close it explicitly before that.
///
/// `RTxtProgressBar` is intentionally `!Send + !Sync`: R's runtime is
/// single-threaded and the underlying SEXP must only be touched from the R
/// main thread.
pub struct RTxtProgressBar {
    /// The protected SEXP for the `txtProgressBar` object.
    ///
    /// Stored on the precious list so R's GC cannot collect it while Rust
    /// holds the struct.
    sexp: SEXP,
    /// Whether the bar is still open. Guards `Drop` against double-close.
    open: bool,
    /// Makes `RTxtProgressBar` `!Send + !Sync` without a runtime cost.
    _not_send: PhantomData<*const ()>,
}

impl std::fmt::Debug for RTxtProgressBar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RTxtProgressBar")
            .field("open", &self.open)
            .finish_non_exhaustive()
    }
}

impl RTxtProgressBar {
    /// Start building a new `txtProgressBar` with the given `min` and `max`.
    ///
    /// # Panics
    ///
    /// Panics if `min > max`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let pb = RTxtProgressBar::builder(0.0, 50.0).style(3).build();
    /// ```
    pub fn builder(min: f64, max: f64) -> RTxtProgressBarBuilder {
        RTxtProgressBarBuilder::new(min, max)
    }

    /// The underlying R progress-bar SEXP.
    #[inline]
    pub fn sexp(&self) -> SEXP {
        self.sexp
    }

    /// Construct from a SEXP that has **already been added to R's precious list**.
    ///
    /// Used by `TryFromSexp for RTxtProgressBar`, which calls
    /// `R_PreserveObject` before handing the SEXP to this constructor.
    /// The `Drop` impl will call `R_ReleaseObject` exactly once.
    ///
    /// # Safety
    ///
    /// - `sexp` must be a list with R class `"txtProgressBar"`.
    /// - Caller must have called `R_PreserveObject(sexp)` before this call.
    pub(crate) unsafe fn from_preserved_sexp(sexp: SEXP) -> Self {
        RTxtProgressBar {
            sexp,
            open: true,
            _not_send: PhantomData,
        }
    }

    /// Set the progress bar to `value`.
    ///
    /// Calls `utils::setTxtProgressBar(pb, value)` in R.
    ///
    /// # Errors
    ///
    /// Returns `Err(String)` if the bar is closed or if the R call fails.
    /// Note: R may print the error message to stderr (via `R_WriteConsoleEx`)
    /// before the `Err` is returned. Callers in a tight update loop who want to
    /// suppress that output should route the bar's `file` to `RNullConnection`
    /// or call via `R_tryEvalSilent` directly.
    pub fn set(&self, value: f64) -> Result<(), String> {
        if !self.open {
            return Err("RTxtProgressBar is closed".to_string());
        }
        unsafe { set_txt_progress_bar_inner(self.sexp, value) }
    }

    /// Get the current progress bar value.
    ///
    /// Calls `utils::getTxtProgressBar(pb)` in R.
    ///
    /// # Errors
    ///
    /// Returns `Err(String)` if the bar is closed or if the R call fails.
    /// Note: R may print the error message to stderr (via `R_WriteConsoleEx`)
    /// before the `Err` is returned.
    pub fn get(&self) -> Result<f64, String> {
        if !self.open {
            return Err("RTxtProgressBar is closed".to_string());
        }
        unsafe { get_txt_progress_bar_inner(self.sexp) }
    }

    /// Explicitly close the progress bar and disarm the `Drop` guard.
    ///
    /// Equivalent to `close(pb)` in R. The handle is consumed — the `Drop`
    /// impl becomes a no-op.
    ///
    /// # Errors
    ///
    /// Returns `Err(String)` if the R close call fails. The precious-list
    /// release still happens regardless.
    pub fn close(mut self) -> Result<(), String> {
        self.open = false;
        let result = unsafe { close_txt_progress_bar_inner(self.sexp) };
        // Release from precious list; transfer ownership back to R's GC.
        unsafe { R_ReleaseObject(self.sexp) };
        std::mem::forget(self); // Skip `Drop` — already cleaned up.
        result
    }
}

impl Drop for RTxtProgressBar {
    fn drop(&mut self) {
        if self.open {
            self.open = false;
            let sexp = self.sexp;
            // Use drop_catching_panic so a panic inside the R call aborts
            // rather than unwinding through the destructor (UB).
            crate::externalptr::drop_catching_panic(|| {
                unsafe {
                    // Call pb$kill() — the R-internal blank-and-close path.
                    // Errors are silently swallowed; we cannot propagate from Drop.
                    kill_txt_progress_bar_inner(sexp);
                    R_ReleaseObject(sexp);
                }
            });
        }
    }
}

// endregion

// region: RTxtProgressBarBuilder

/// Builder for [`RTxtProgressBar`].
///
/// Construct via [`RTxtProgressBar::builder`].
pub struct RTxtProgressBarBuilder {
    min: f64,
    max: f64,
    initial: f64,
    char: String,
    width: Option<u32>,
    style: u8,
    file: Option<SEXP>,
}

impl RTxtProgressBarBuilder {
    fn new(min: f64, max: f64) -> Self {
        assert!(
            min <= max,
            "RTxtProgressBar: min ({min}) must be <= max ({max}) — transposed arguments?",
        );
        RTxtProgressBarBuilder {
            min,
            max,
            initial: min,
            char: "=".to_string(),
            width: None,
            style: 3,
            file: None,
        }
    }

    /// Set the initial value of the progress bar (default: `min`).
    pub fn initial(mut self, initial: f64) -> Self {
        self.initial = initial;
        self
    }

    /// Set the fill character for the progress bar (default: `"="`).
    pub fn char(mut self, char: impl Into<String>) -> Self {
        self.char = char.into();
        self
    }

    /// Set the width of the bar in characters.
    ///
    /// When not set (the default) R uses `getOption("width")`.
    pub fn width(mut self, width: u32) -> Self {
        self.width = Some(width);
        self
    }

    /// Set the style of the progress bar: 1, 2, or 3 (default: 3).
    ///
    /// - Style 1: percentage only.
    /// - Style 2: percentage + basic bar.
    /// - Style 3: pipe-delimited bar with percentage (`|====  | 40%`).
    ///
    /// # Panics
    ///
    /// Panics if `style` is not 1, 2, or 3.
    pub fn style(mut self, style: u8) -> Self {
        assert!(
            matches!(style, 1..=3),
            "RTxtProgressBarBuilder::style: must be 1, 2, or 3, got {style}",
        );
        self.style = style;
        self
    }

    /// Route the progress bar output to a specific R connection.
    ///
    /// Accepts any R connection SEXP — e.g., `RStderr.into_sexp()` or
    /// `RNullConnection::new().sexp()`. When omitted the `file` argument is
    /// not passed to `txtProgressBar()`, so R defaults to its stdout.
    pub fn file(mut self, file: impl Into<SEXP>) -> Self {
        self.file = Some(file.into());
        self
    }

    /// Build the progress bar by evaluating `utils::txtProgressBar(...)` in R.
    ///
    /// # Panics
    ///
    /// Panics if `utils` is not loaded or if `txtProgressBar()` errors.
    pub fn build(self) -> RTxtProgressBar {
        unsafe { build_inner(self) }
    }
}

// endregion

// region: unsafe R-API helpers

// Evaluate utils::txtProgressBar(...) and return a preserved SEXP.
//
// # Safety
// Must be called from the R main thread.
unsafe fn build_inner(opts: RTxtProgressBarBuilder) -> RTxtProgressBar {
    use crate::expression::{RCall, REnv};
    use crate::ffi::{Rf_mkString, SEXP};

    unsafe {
        // Resolve the utils namespace — utils is always loaded (base default).
        let utils_ns =
            REnv::package_namespace("utils").expect("utils namespace not found — is utils loaded?");

        // Prepare scalar arguments; protect them across the RCall::build step.
        let min_val = SEXP::scalar_real(opts.min);
        Rf_protect(min_val);
        let max_val = SEXP::scalar_real(opts.max);
        Rf_protect(max_val);
        let initial_val = SEXP::scalar_real(opts.initial);
        Rf_protect(initial_val);

        let char_cstr = std::ffi::CString::new(opts.char.as_str())
            .unwrap_or_else(|_| std::ffi::CString::new("=").unwrap());
        let char_val = Rf_mkString(char_cstr.as_ptr());
        Rf_protect(char_val);

        // NA_integer_ when width is None (R interprets NA as "use getOption").
        let width_val = SEXP::scalar_integer(opts.width.map(|w| w as i32).unwrap_or(i32::MIN));
        Rf_protect(width_val);

        let style_val = SEXP::scalar_integer(opts.style as i32);
        Rf_protect(style_val);

        // Build the call: txtProgressBar(min=, max=, initial=, char=, width=, style= [, file=])
        let mut call = RCall::new("txtProgressBar")
            .named_arg("min", min_val)
            .named_arg("max", max_val)
            .named_arg("initial", initial_val)
            .named_arg("char", char_val)
            .named_arg("width", width_val)
            .named_arg("style", style_val);

        if let Some(file_sexp) = opts.file {
            call = call.named_arg("file", file_sexp);
        }

        let result = call.eval(utils_ns.as_sexp());

        // Release our scalar protections.
        Rf_unprotect(6); // style_val, width_val, char_val, initial_val, max_val, min_val

        let sexp = result.expect("utils::txtProgressBar(...) failed");

        // Pin on the precious list — GC cannot collect while Rust holds it.
        R_PreserveObject(sexp);

        RTxtProgressBar {
            sexp,
            open: true,
            _not_send: PhantomData,
        }
    }
}

// Call setTxtProgressBar(pb, value) in the utils namespace.
//
// # Safety
// Must be called from the R main thread.
unsafe fn set_txt_progress_bar_inner(sexp: SEXP, value: f64) -> Result<(), String> {
    use crate::expression::RCall;
    use crate::ffi::SEXP;

    unsafe {
        let val_sexp = SEXP::scalar_real(value);
        Rf_protect(val_sexp);

        let result = RCall::new("setTxtProgressBar")
            .arg(sexp)
            .arg(val_sexp)
            .eval_base();

        Rf_unprotect(1); // val_sexp
        result.map(|_| ())
    }
}

// Call getTxtProgressBar(pb) and return the numeric result.
//
// # Safety
// Must be called from the R main thread.
unsafe fn get_txt_progress_bar_inner(sexp: SEXP) -> Result<f64, String> {
    use crate::expression::RCall;

    unsafe {
        let result = RCall::new("getTxtProgressBar").arg(sexp).eval_base()?;
        // getTxtProgressBar returns a numeric(1); unwrap NA as 0.0.
        Ok(result.as_real().unwrap_or(0.0))
    }
}

// Call close(pb) — explicit close path.
// Errors are returned as Err(String). Release of the precious list is the
// caller's responsibility.
//
// # Safety
// Must be called from the R main thread.
unsafe fn close_txt_progress_bar_inner(sexp: SEXP) -> Result<(), String> {
    use crate::expression::RCall;

    unsafe { RCall::new("close").arg(sexp).eval_base().map(|_| ()) }
}

// Call pb$kill() — silent close used by the Drop impl.
// Errors are swallowed because Drop cannot propagate them.
//
// # Safety
// Must be called from the R main thread.
unsafe fn kill_txt_progress_bar_inner(sexp: SEXP) {
    use crate::ffi::{R_BaseEnv, Rf_install, Rf_lang1, Rf_lang3, Rf_mkString};

    unsafe {
        // Extract pb$kill — evaluates `"$"(pb, "kill")` to get the closure.
        let dollar_sym = Rf_install(c"$".as_ptr());
        let kill_str = Rf_mkString(c"kill".as_ptr());
        Rf_protect(kill_str);

        let extract = Rf_lang3(dollar_sym, sexp, kill_str);
        Rf_protect(extract);

        let mut err: std::os::raw::c_int = 0;
        let kill_fn = crate::ffi::R_tryEvalSilent(extract, R_BaseEnv, &mut err);
        Rf_unprotect(2); // extract, kill_str
        if err != 0 {
            return; // bar already closed or invalid — ignore
        }

        // Call kill()
        Rf_protect(kill_fn);
        let call = Rf_lang1(kill_fn);
        Rf_protect(call);
        let mut err2: std::os::raw::c_int = 0;
        crate::ffi::R_tryEvalSilent(call, R_BaseEnv, &mut err2);
        Rf_unprotect(2); // call, kill_fn
        // Ignore err2 — we're in Drop, cannot propagate.
    }
}

// endregion
