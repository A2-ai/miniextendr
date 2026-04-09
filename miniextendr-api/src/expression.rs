//! Safe wrappers for R expression evaluation.
//!
//! This module provides ergonomic types for building and evaluating R function
//! calls from Rust, handling GC protection and error propagation automatically.
//!
//! # Types
//!
//! | Type | Purpose |
//! |------|---------|
//! | [`RSymbol`] | Interned R symbol (SYMSXP) |
//! | [`RCall`] | Builder for R function calls (LANGSXP) |
//! | [`REnv`] | Well-known R environments |
//!
//! # Example
//!
//! ```ignore
//! use miniextendr_api::expression::{RCall, REnv};
//!
//! unsafe {
//!     // Call paste0("hello", " world") in the base environment
//!     let result = RCall::new("paste0")
//!         .arg(Rf_mkString(c"hello".as_ptr()))
//!         .arg(Rf_mkString(c" world".as_ptr()))
//!         .eval(REnv::base().as_sexp())?;
//! }
//! ```

use crate::ffi::{
    self, R_BaseEnv, R_BaseNamespace, R_EmptyEnv, R_GlobalEnv, R_tryEvalSilent, Rf_install,
    Rf_lcons, Rf_protect, Rf_unprotect, SET_TAG, SEXP, SexpExt,
};
use std::ffi::{CStr, CString};

// region: RSymbol

/// A safe wrapper around R symbols (SYMSXP).
///
/// R symbols are interned strings used as variable and function names.
/// They are never garbage collected, so `RSymbol` does not need GC protection.
///
/// # Example
///
/// ```ignore
/// let sym = RSymbol::new("paste0");
/// // sym.as_sexp() is a SYMSXP that can be used in call construction
/// ```
pub struct RSymbol {
    sexp: SEXP,
}

impl RSymbol {
    /// Create or retrieve an interned R symbol.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    ///
    /// # Panics
    ///
    /// Panics if `name` contains a null byte.
    #[inline]
    pub unsafe fn new(name: &str) -> Self {
        let c_name = CString::new(name).expect("symbol name must not contain null bytes");
        RSymbol {
            sexp: unsafe { Rf_install(c_name.as_ptr()) },
        }
    }

    /// Create a symbol from a C string literal.
    ///
    /// This avoids the allocation needed by [`new`](Self::new) when you have
    /// a `&CStr` available (e.g., from `c"name"` literals).
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn from_cstr(name: &CStr) -> Self {
        RSymbol {
            sexp: unsafe { Rf_install(name.as_ptr()) },
        }
    }

    /// Get the underlying SEXP.
    #[inline]
    pub fn as_sexp(&self) -> SEXP {
        self.sexp
    }
}
// endregion

// region: REnv

/// Handle to a well-known R environment.
///
/// Provides access to R's standard environments without raw FFI calls.
pub struct REnv {
    sexp: SEXP,
}

impl REnv {
    /// The global environment (`R_GlobalEnv`).
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn global() -> Self {
        REnv {
            sexp: unsafe { R_GlobalEnv },
        }
    }

    /// The base environment (`R_BaseEnv`).
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn base() -> Self {
        REnv {
            sexp: unsafe { R_BaseEnv },
        }
    }

    /// The empty environment (`R_EmptyEnv`).
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn empty() -> Self {
        REnv {
            sexp: unsafe { R_EmptyEnv },
        }
    }

    /// The base namespace (`R_BaseNamespace`).
    ///
    /// Unlike [`base()`](Self::base) which is the base *environment* (exported
    /// functions visible to users), this is the base *namespace* (includes
    /// internal helpers). Rarely needed — prefer [`base()`](Self::base) unless
    /// you specifically need unexported base internals.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn base_namespace() -> Self {
        REnv {
            sexp: unsafe { R_BaseNamespace },
        }
    }

    /// A package's namespace environment.
    ///
    /// Finds the namespace for a loaded package. Use this to evaluate functions
    /// that live in a specific package (e.g., `slot()` from `methods`).
    ///
    /// This is a safe wrapper around `R_FindNamespace` — it uses
    /// `R_tryEvalSilent` internally so that a missing namespace returns
    /// `Err` instead of longjmping through Rust frames.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the package namespace is not found (package not loaded).
    pub unsafe fn package_namespace(name: &str) -> Result<Self, String> {
        unsafe {
            let name_sexp = SEXP::scalar_string_from_str(name);
            Rf_protect(name_sexp);
            let result = RCall::new("getNamespace").arg(name_sexp).eval(R_BaseEnv);
            Rf_unprotect(1);
            result.map(|sexp| REnv { sexp })
        }
    }

    /// The current execution environment.
    ///
    /// Returns the environment of the innermost active closure on R's call
    /// stack, or the global environment if no closure is active.
    ///
    /// Useful when you need to evaluate an expression in the caller's context
    /// rather than a fixed well-known environment.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn caller() -> Self {
        REnv {
            sexp: unsafe { ffi::R_GetCurrentEnv() },
        }
    }

    /// Wrap an arbitrary environment SEXP.
    ///
    /// # Safety
    ///
    /// `sexp` must be a valid ENVSXP.
    #[inline]
    pub unsafe fn from_sexp(sexp: SEXP) -> Self {
        REnv { sexp }
    }

    /// Get the underlying SEXP.
    #[inline]
    pub fn as_sexp(&self) -> SEXP {
        self.sexp
    }
}
// endregion

// region: RCall

/// Builder for constructing and evaluating R function calls.
///
/// `RCall` constructs a LANGSXP (R language object) from a function name or
/// SEXP and a sequence of arguments (optionally named). It handles GC
/// protection during construction and evaluation.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::expression::RCall;
/// use miniextendr_api::ffi;
///
/// unsafe {
///     // seq_len(10)
///     let result = RCall::new("seq_len")
///         .arg(ffi::Rf_ScalarInteger(10))
///         .eval_base()?;
///
///     // paste(x, collapse = ", ")
///     let result = RCall::new("paste")
///         .arg(some_sexp)
///         .named_arg("collapse", ffi::Rf_mkString(c", ".as_ptr()))
///         .eval_base()?;
/// }
/// ```
pub struct RCall {
    /// Function symbol or SEXP.
    fun: SEXP,
    /// Arguments as (optional_name, value) pairs.
    args: Vec<(Option<CString>, SEXP)>,
}

impl RCall {
    /// Start building a call to a named R function.
    ///
    /// The function is looked up via `Rf_install`, which returns an interned symbol.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    ///
    /// # Panics
    ///
    /// Panics if `fun_name` contains a null byte.
    #[inline]
    pub unsafe fn new(fun_name: &str) -> Self {
        let c_name = CString::new(fun_name).expect("function name must not contain null bytes");
        RCall {
            fun: unsafe { Rf_install(c_name.as_ptr()) },
            args: Vec::new(),
        }
    }

    /// Start building a call to a function given as a C string literal.
    ///
    /// More efficient than [`new`](Self::new) when a `&CStr` is available.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn from_cstr(fun_name: &CStr) -> Self {
        RCall {
            fun: unsafe { Rf_install(fun_name.as_ptr()) },
            args: Vec::new(),
        }
    }

    /// Start building a call with a function SEXP (closure, builtin, etc.).
    ///
    /// # Safety
    ///
    /// `fun` must be a valid SEXP representing a callable R object.
    #[inline]
    pub unsafe fn from_sexp(fun: SEXP) -> Self {
        RCall {
            fun,
            args: Vec::new(),
        }
    }

    /// Add a positional argument.
    #[inline]
    pub fn arg(mut self, value: SEXP) -> Self {
        self.args.push((None, value));
        self
    }

    /// Add a named argument.
    ///
    /// # Panics
    ///
    /// Panics if `name` contains a null byte.
    #[inline]
    pub fn named_arg(mut self, name: &str, value: SEXP) -> Self {
        let c_name = CString::new(name).expect("argument name must not contain null bytes");
        self.args.push((Some(c_name), value));
        self
    }

    /// Build the LANGSXP without evaluating it.
    ///
    /// The returned SEXP is **unprotected**. The caller must protect it if
    /// further allocations will occur before use.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread. All argument SEXPs must still
    /// be valid (protected or otherwise reachable by R's GC).
    pub unsafe fn build(&self) -> SEXP {
        unsafe {
            // Build the argument pairlist from back to front using Rf_cons.
            // We protect intermediate results as we go.
            let mut n_protect: i32 = 0;

            let mut tail = SEXP::nil();
            for (name, value) in self.args.iter().rev() {
                tail = ffi::Rf_cons(*value, tail);
                Rf_protect(tail);
                n_protect += 1;

                if let Some(c_name) = name {
                    SET_TAG(tail, Rf_install(c_name.as_ptr()));
                }
            }

            // Prepend the function as LANGSXP head
            let call = Rf_lcons(self.fun, tail);
            Rf_protect(call);
            n_protect += 1;

            // Clean up all intermediate protections; caller is responsible
            // for protecting the returned call.
            Rf_unprotect(n_protect);
            call
        }
    }

    /// Evaluate the call in the given environment.
    ///
    /// Uses `R_tryEvalSilent` so that R errors are captured as `Err(String)`
    /// rather than causing a longjmp through Rust frames.
    ///
    /// # Safety
    ///
    /// - Must be called from the R main thread.
    /// - `env` must be a valid ENVSXP.
    /// - All argument SEXPs must still be valid.
    ///
    /// # Returns
    ///
    /// - `Ok(SEXP)` with the result (unprotected — caller should protect if needed)
    /// - `Err(String)` with the R error message on failure
    pub unsafe fn eval(&self, env: SEXP) -> Result<SEXP, String> {
        unsafe {
            let call = self.build();
            Rf_protect(call);

            let mut error_occurred: std::os::raw::c_int = 0;
            let result = R_tryEvalSilent(call, env, &mut error_occurred);

            Rf_unprotect(1); // call

            if error_occurred != 0 {
                Err(get_r_error_message())
            } else {
                Ok(result)
            }
        }
    }

    /// Evaluate in `R_BaseEnv`.
    ///
    /// # Safety
    ///
    /// Same as [`eval`](Self::eval).
    #[inline]
    pub unsafe fn eval_base(&self) -> Result<SEXP, String> {
        unsafe { self.eval(R_BaseEnv) }
    }
}
// endregion

// region: Error message extraction

/// Extract the most recent R error message.
///
/// Uses `geterrmessage()` which is public R API (unlike `R_curErrorBuf`
/// which is non-API). Falls back to a generic message if extraction fails.
unsafe fn get_r_error_message() -> String {
    unsafe {
        // Call geterrmessage() — a public R function that returns the last
        // error message as a character(1) string.
        let call = ffi::Rf_lang1(Rf_install(c"geterrmessage".as_ptr()));
        Rf_protect(call);

        let mut err: std::os::raw::c_int = 0;
        let msg_sexp = R_tryEvalSilent(call, R_BaseEnv, &mut err);

        if err != 0 || msg_sexp.is_null() {
            Rf_unprotect(1); // call
            return "R error occurred (could not retrieve message)".to_string();
        }

        Rf_protect(msg_sexp);

        // geterrmessage() returns character(1)
        let result = if ffi::Rf_xlength(msg_sexp) > 0 {
            let charsxp = msg_sexp.string_elt(0);
            if !charsxp.is_null() {
                let ptr = ffi::R_CHAR(charsxp);
                if !ptr.is_null() {
                    let msg = CStr::from_ptr(ptr).to_string_lossy().into_owned();
                    msg.trim_end().to_string()
                } else {
                    "R error occurred".to_string()
                }
            } else {
                "R error occurred".to_string()
            }
        } else {
            "R error occurred".to_string()
        };

        Rf_unprotect(2); // call + msg_sexp
        result
    }
}
// endregion

// region: Tests

#[cfg(test)]
mod tests {
    use super::*;

    // These tests verify compilation and basic invariants.
    // Full integration tests require the R runtime.

    #[test]
    fn rcall_arg_accumulation() {
        // Verify the builder pattern accumulates args correctly.
        // We can't call R functions without an R runtime, but we can
        // check that the Vec grows as expected.
        let call = RCall {
            fun: SEXP(std::ptr::null_mut()),
            args: Vec::new(),
        };
        let call = call
            .arg(SEXP(std::ptr::null_mut()))
            .arg(SEXP(std::ptr::null_mut()));
        assert_eq!(call.args.len(), 2);
        assert!(call.args[0].0.is_none());
        assert!(call.args[1].0.is_none());
    }

    #[test]
    fn rcall_named_arg() {
        let call = RCall {
            fun: SEXP(std::ptr::null_mut()),
            args: Vec::new(),
        };
        let call = call.named_arg("collapse", SEXP(std::ptr::null_mut()));
        assert_eq!(call.args.len(), 1);
        assert_eq!(
            call.args[0].0.as_ref().unwrap(),
            &CString::new("collapse").unwrap()
        );
    }

    #[test]
    fn renv_types_are_sized() {
        // Just verify types compile and are sized
        fn assert_sized<T: Sized>() {}
        assert_sized::<RSymbol>();
        assert_sized::<RCall>();
        assert_sized::<REnv>();
    }

    #[test]
    fn renv_constructors_compile() {
        // Verify all REnv constructor signatures compile.
        // Actual testing requires the R runtime.
        fn assert_env_fn<F: FnOnce() -> REnv>(_f: F) {}
        fn assert_env_result_fn<F: FnOnce() -> Result<REnv, String>>(_f: F) {}

        assert_env_fn(|| unsafe { REnv::global() });
        assert_env_fn(|| unsafe { REnv::base() });
        assert_env_fn(|| unsafe { REnv::empty() });
        assert_env_fn(|| unsafe { REnv::base_namespace() });
        assert_env_fn(|| unsafe { REnv::caller() });
        assert_env_result_fn(|| unsafe { REnv::package_namespace("base") });
    }
}
// endregion
