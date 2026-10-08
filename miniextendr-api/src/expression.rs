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
//! | [`REvalError`] | An R error caught by [`RCall::eval`] / [`r_eval_str`] |
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

use crate::condition::ConditionClass;
use crate::gc_protect::{OwnedProtect, ProtectScope};
use crate::protect_pool::ProtectPool;
use crate::sexp_ext::PairListExt;
use crate::sys::{
    self, ParseStatus, R_BaseEnv, R_EmptyEnv, R_GlobalEnv, R_ParseVector, Rf_install,
};
use crate::{SEXP, SexpExt};
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::os::raw::c_void;

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

    /// The base namespace (`SEXP::base_namespace()`).
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
    pub fn base_namespace() -> Self {
        REnv {
            sexp: SEXP::base_namespace(),
        }
    }

    /// A package's namespace environment.
    ///
    /// Finds the namespace for a loaded package. Use this to evaluate functions
    /// that live in a specific package (e.g., `slot()` from `methods`).
    ///
    /// This is a safe wrapper around `getNamespace(name)`, evaluated through
    /// [`RCall::eval`], so a missing namespace returns `Err` instead of
    /// longjmping through Rust frames.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    ///
    /// # Errors
    ///
    /// R's error when the namespace cannot be loaded (a
    /// `packageNotFoundError` for a package that is not installed).
    pub unsafe fn package_namespace(name: &str) -> Result<Self, REvalError> {
        unsafe {
            let name_sexp = OwnedProtect::new(SEXP::scalar_string_from_str(name));
            RCall::new("getNamespace")
                .arg(name_sexp.get())
                .eval(R_BaseEnv)
                .map(|sexp| REnv { sexp })
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
            sexp: unsafe { sys::R_GetCurrentEnv() },
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
/// use miniextendr_api::sys;
///
/// unsafe {
///     // seq_len(10)
///     let result = RCall::new("seq_len")
///         .arg(SEXP::scalar_integer(10))
///         .eval_base()?;
///
///     // paste(x, collapse = ", ")
///     let result = RCall::new("paste")
///         .arg(some_sexp)
///         .named_arg("collapse", sys::Rf_mkString(c", ".as_ptr()))
///         .eval_base()?;
/// }
/// ```
pub struct RCall {
    /// Function symbol or SEXP.
    fun: SEXP,
    /// Arguments as (optional_name, value) pairs.
    args: Vec<(Option<CString>, SEXP)>,
    /// Roots the callable and every argument for the builder's lifetime,
    /// except symbols, which need no root ([`needs_root`]).
    ///
    /// Besides keeping inline allocations alive, `ProtectPool` makes `RCall`
    /// `!Send + !Sync`: construction, mutation, and drop stay on R's thread.
    roots: ProtectPool,
}

impl RCall {
    /// Initialize an empty builder calling `fun`, rooted unless it is a symbol.
    ///
    /// `fun` may be a freshly allocated, unprotected closure. The temporary
    /// stack root keeps it alive while the pool allocates its backing VECSXP.
    #[inline]
    unsafe fn from_callable(fun: SEXP) -> Self {
        unsafe {
            if !needs_root(fun) {
                return RCall {
                    fun,
                    args: Vec::new(),
                    roots: ProtectPool::new(4),
                };
            }
            let fun_guard = OwnedProtect::new(fun);
            let mut roots = ProtectPool::new(4);
            roots.insert(fun_guard.get());
            RCall {
                fun,
                args: Vec::new(),
                roots,
            }
        }
    }

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
        unsafe { Self::from_callable(Rf_install(c_name.as_ptr())) }
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
        unsafe { Self::from_callable(Rf_install(fun_name.as_ptr())) }
    }

    /// Start building a call with a function SEXP (closure, builtin, etc.).
    ///
    /// # Safety
    ///
    /// `fun` must be a valid SEXP representing a callable R object.
    #[inline]
    pub unsafe fn from_sexp(fun: SEXP) -> Self {
        unsafe { Self::from_callable(fun) }
    }

    /// Root an argument for the builder's lifetime, unless it is a symbol.
    ///
    /// The temporary stack root covers `ProtectPool::insert` if growing the
    /// backing VECSXP allocates. The pool then owns the lasting root.
    ///
    /// # Safety
    ///
    /// R's main thread.
    #[inline]
    unsafe fn root(&mut self, value: SEXP) {
        if needs_root(value) {
            unsafe {
                let value_guard = OwnedProtect::new(value);
                self.roots.insert(value_guard.get());
            }
        }
    }

    /// Add a positional argument.
    #[inline]
    pub fn arg(mut self, value: SEXP) -> Self {
        unsafe { self.root(value) };
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
        unsafe { self.root(value) };
        self.args.push((Some(c_name), value));
        self
    }

    /// Add a positional argument passed as is: `quote(<value>)`.
    ///
    /// The call evaluates its arguments, so a symbol, a call or a quosure
    /// added with [`arg`](Self::arg) is evaluated before the function sees
    /// it. Use this for a language object the function must receive as a
    /// value: a call for an `error_call` / `call` argument, an expression
    /// for `eval()`. Any other value passes through `quote()` unchanged.
    #[inline]
    pub fn quoted_arg(self, value: SEXP) -> Self {
        let quoted = unsafe { quote(value) };
        self.arg(quoted)
    }

    /// Add a named argument passed as is: `name = quote(<value>)`. See
    /// [`quoted_arg`](Self::quoted_arg).
    ///
    /// # Panics
    ///
    /// Panics if `name` contains a null byte.
    #[inline]
    pub fn named_quoted_arg(self, name: &str, value: SEXP) -> Self {
        let quoted = unsafe { quote(value) };
        self.named_arg(name, quoted)
    }

    /// Build the LANGSXP without evaluating it.
    ///
    /// The returned SEXP is **unprotected**. The caller must protect it if
    /// further allocations will occur before use.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread. The builder keeps the callable
    /// and every argument rooted until it is dropped.
    pub unsafe fn build(&self) -> SEXP {
        unsafe {
            // Build the argument pairlist from back to front using Rf_cons.
            // ProtectScope tracks all intermediate cons cells and the final
            // LANGSXP head, then unprotects them all on drop. The returned
            // call is unprotected — caller protects it if needed.
            let scope = ProtectScope::new();

            let mut tail = SEXP::nil();
            for (name, value) in self.args.iter().rev() {
                tail = scope.protect_raw(value.cons(tail));
                if let Some(c_name) = name {
                    tail.set_tag(Rf_install(c_name.as_ptr()));
                }
            }

            // Prepend the function as LANGSXP head.
            // ProtectScope drops here; the call is unprotected on return
            // (callers re-protect via OwnedProtect before invoking eval).
            scope.protect_raw(self.fun.lcons(tail))
        }
    }

    /// Evaluate the call in the given environment, catching an R error.
    ///
    /// The call runs in a new top-level context (`R_ToplevelExec`) under an
    /// exiting handler for `error` conditions (`R_tryCatchError`). So the
    /// caller's condition handlers and restarts don't see it, and an R error
    /// comes back as `Err` holding R's condition (its message, call, classes
    /// and the object itself) instead of jumping through the Rust frames.
    /// Use [`eval_with_handlers`](Self::eval_with_handlers) for a call whose
    /// conditions the user should see as raised.
    ///
    /// ```ignore
    /// match RCall::new("print").arg(df).named_arg("na.print", one).eval_base() {
    ///     Ok(_) => {}
    ///     // `e` is R's condition: "invalid 'na.print' specification",
    ///     // without the "Error in print.default(...) :" prefix.
    ///     Err(e) => rust_error!(class = e.reraise_class("pkg_print_error"), "{e}"),
    /// }
    /// ```
    ///
    /// # Safety
    ///
    /// - Must be called from the R main thread.
    /// - `env` must be a valid ENVSXP.
    /// - The builder must not outlive the active R session.
    ///
    /// # Returns
    ///
    /// - `Ok(SEXP)` with the result (unprotected — caller should protect if needed)
    /// - `Err(REvalError)` with R's error condition on failure
    pub unsafe fn eval(&self, env: SEXP) -> Result<SEXP, REvalError> {
        unsafe {
            let call = OwnedProtect::new(self.build());
            try_eval(call.get(), env)
        }
    }

    /// Evaluate in `R_BaseEnv`.
    ///
    /// # Safety
    ///
    /// Same as [`eval`](Self::eval).
    #[inline]
    pub unsafe fn eval_base(&self) -> Result<SEXP, REvalError> {
        unsafe { self.eval(R_BaseEnv) }
    }

    /// Evaluate the call in `env` in the caller's R context, keeping R's
    /// condition handlers: see [`eval_with_handlers`].
    ///
    /// Use it to call R code that signals conditions the user should see, with
    /// their classes, through their own `withCallingHandlers()` /
    /// `tryCatch()`. A tidyselect selection, say:
    ///
    /// ```ignore
    /// // `cols` is a `Quosure` parameter, `data` a data frame.
    /// let selected = unsafe {
    ///     RCall::namespaced("tidyselect", "eval_select")?
    ///         .arg(cols.sexp())
    ///         .arg(data)
    ///         .eval_with_handlers(R_BaseEnv)
    /// };
    /// ```
    ///
    /// A column that does not exist raises tidyselect's own error (class
    /// `vctrs_error_subscript_oob`, with its call), which unwinds through the
    /// Rust frames and reaches the user's `tryCatch()` as it was raised.
    ///
    /// # Safety
    ///
    /// As [`eval_with_handlers`]: R's main thread, inside a miniextendr
    /// boundary, `env` an environment. The result is unprotected.
    pub unsafe fn eval_with_handlers(&self, env: SEXP) -> SEXP {
        unsafe {
            let call = OwnedProtect::new(self.build());
            eval_with_handlers(call.get(), env)
        }
    }

    /// Start building a namespaced call: `pkg::fun(args…)`.
    ///
    /// Looks up `pkg::fun_name` in the base environment and uses the resolved
    /// function closure as the call target. This respects R's namespace lookup
    /// rules (exported + non-exported via `::` / `:::`).
    ///
    /// This is the runtime counterpart of the lowered `pkg::fn(args…)` form
    /// in `r!(pkg::fn(args…))`.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    ///
    /// # Panics
    ///
    /// Panics if `pkg` or `fun_name` contain a null byte.
    ///
    /// # Errors
    ///
    /// R's error if the namespace lookup fails (package not installed, or no
    /// such function).
    pub unsafe fn namespaced(pkg: &str, fun_name: &str) -> Result<Self, REvalError> {
        unsafe {
            // Build (:: pkg fun_name) as a LANGSXP and evaluate it to get the closure.
            let ns_op = Rf_install(c"::".as_ptr());
            let pkg_sym = {
                let c = CString::new(pkg).expect("pkg name must not contain null bytes");
                Rf_install(c.as_ptr())
            };
            let fun_sym = {
                let c = CString::new(fun_name).expect("fun name must not contain null bytes");
                Rf_install(c.as_ptr())
            };

            // Build `(:: pkg fun_name)` pairlist from back to front.
            let scope = crate::gc_protect::ProtectScope::new();
            let nil = crate::sys::R_NilValue;
            let fun_cons = scope.protect_raw(fun_sym.cons(nil));
            let pkg_cons = scope.protect_raw(pkg_sym.cons(fun_cons));
            let ns_call = scope.protect_raw(ns_op.lcons(pkg_cons));

            // Evaluate to resolve the function closure.
            let fun_sexp = try_eval(ns_call, R_BaseEnv)?;

            // Root the resolved closure for the builder lifetime. This is
            // load-bearing even for closures that are not namespace bindings
            // (the same constructor backs `RCall::from_sexp`).
            Ok(RCall::from_sexp(fun_sexp))
        }
    }
}

/// Build and evaluate `target$name` — the R `$` extraction operator.
///
/// This is a convenience wrapper that avoids hand-rolling
/// `Rf_install("$") + Rf_lang3(...) + R_tryEval(...)` ladders.
/// Equivalent to:
///
/// ```ignore
/// RCall::new("$")
///     .arg(target)
///     .arg(SEXP::scalar_string_from_str(name))
///     .eval_base()
/// ```
///
/// with the name argument protected while the call is built.
///
/// # Safety
///
/// - Must be called from the R main thread.
/// - `target` must be a valid SEXP (typically a list, environment, or S4
///   object that supports `$` extraction).
///
/// # Returns
///
/// - `Ok(SEXP)` with the extracted value (unprotected — caller should protect if needed).
/// - `Err(REvalError)` with R's error if `$` extraction fails.
pub unsafe fn dollar_extract(target: SEXP, name: &str) -> Result<SEXP, REvalError> {
    unsafe {
        let name_sexp = OwnedProtect::new(SEXP::scalar_string_from_str(name));
        RCall::new("$").arg(target).arg(name_sexp.get()).eval_base()
    }
}
// endregion

/// Whether `value` needs a GC root while Rust holds it across an allocation.
///
/// A symbol never does: R never frees one. `install()` links every new symbol
/// into `R_SymbolTable` (`src/main/names.c`, `install`, and `installNoTrChar`
/// likewise), nothing ever unlinks it, and the collector marks every bucket of
/// the table on every collection (`src/main/memory.c`, `RunGenCollect`: the
/// `R_SymbolTable` loop). The marker symbols outside the table
/// (`R_MissingArg`, `R_UnboundValue`, `R_RestartToken` and the rest, made by
/// `mkSymMarker` in `InitNames`) are forwarded as roots of their own in the
/// same function.
#[inline]
fn needs_root(value: SEXP) -> bool {
    value.type_of() != crate::SEXPTYPE::SYMSXP
}

/// `quote(<value>)`, unprotected. `value` is rooted across the allocation
/// unless it is a symbol ([`needs_root`]).
///
/// # Safety
///
/// R's main thread.
unsafe fn quote(value: SEXP) -> SEXP {
    unsafe {
        let quote = Rf_install(c"quote".as_ptr());
        if !needs_root(value) {
            return crate::sys::Rf_lang2(quote, value);
        }
        let value = OwnedProtect::new(value);
        crate::sys::Rf_lang2(quote, value.get())
    }
}

// region: eval_with_handlers (evaluation in the caller's R context)

/// Evaluate `expr` in `env` in the caller's R context, the way R code calling
/// `eval(expr, env)` does.
///
/// | | [`RCall::eval`], [`r_eval_str`] | `eval_with_handlers` |
/// |---|---|---|
/// | R entry point | `R_tryCatchError` inside `R_ToplevelExec` | `Rf_eval` in its own `R_UnwindProtect` |
/// | caller's `withCallingHandlers()`, `suppressWarnings()` | not seen (`R_ToplevelExec` empties the handler and restart stacks) | see every warning, message and condition |
/// | R error | `Err(`[`REvalError`]`)`: R's condition, with its message (no `Error in` prefix, no `Calls:` line), call and classes; the Rust code decides what to raise | unwinds as the R condition, class and call kept |
/// | caller's `tryCatch(warning = )`, restarts, interrupts | not seen (an interrupt ends the evaluation with an `Err`) | exit through the Rust frames |
///
/// An R error, or any other exit from the evaluation (an exiting handler of
/// the caller's `tryCatch()`, `invokeRestart()`, an interrupt), stops at this
/// function's own `R_UnwindProtect` frame, so it never jumps over a Rust
/// frame. It then leaves as a Rust unwind: the frames between here and the
/// `#[miniextendr]` boundary drop their values (a guard's `Drop` runs), and
/// the boundary hands the exit back to R, which carries on to the handler or
/// restart it was going to, with the original condition object. A warning or
/// message the caller muffles returns here and the evaluation continues.
///
/// So this is the evaluator for code a user wrote: an argument passed
/// unevaluated ([`crate::Quoted`], [`crate::Quosure`]), a callback, a call into
/// a package whose conditions are part of its interface. Keep [`RCall::eval`]
/// for internal calls whose failure the Rust code handles itself, for
/// instance by raising its own condition with the caught message
/// ([`REvalError::reraise_class`]).
///
/// The unwind must reach a miniextendr boundary. Don't catch it with
/// `std::panic::catch_unwind` in between, or the R exit is abandoned (R
/// carries on as after `R_tryEval`, without running the caller's handler);
/// resume a caught payload with `std::panic::resume_unwind` on the same
/// thread before the call returns. An exit resumed in a later call has lost
/// its target: the boundary raises an error instead. Don't call this from a
/// `Drop` implementation: an exit there would start an unwind during an
/// unwind, which aborts.
///
/// # Safety
///
/// - R's main thread, inside a miniextendr boundary: a `#[miniextendr]`
///   function body, a [`with_r_unwind_protect`](crate::unwind_protect::with_r_unwind_protect)
///   closure, an ALTREP callback with the `r_unwind` or `rust_unwind` guard
///   (not `unsafe`, which catches nothing), a connection callback, a
///   [`with_r_thread`](crate::worker::with_r_thread) closure.
/// - `expr` and `env` stay rooted for the call; `env` is an environment.
///
/// # Returns
///
/// The value, **unprotected**: protect it before the next allocation.
pub unsafe fn eval_with_handlers(expr: SEXP, env: SEXP) -> SEXP {
    unsafe { crate::unwind_protect::eval_unwinding(expr, env) }
}

// endregion

// region: r_eval_str (runtime string parse + eval)

/// Parse a string of R source and evaluate it in `env`.
///
/// This is the runtime workhorse behind the [`r_str!`](crate::r_str) and
/// [`r!`](crate::r) macros. It performs the full
/// `R_ParseVector` → check status → `Rf_eval` ladder with correct GC
/// protection on every intermediate SEXP, so callers never have to hand-roll
/// `OwnedProtect` around the parse tree.
///
/// Only the **last** top-level expression's value is returned (matching R's
/// `eval(parse(text = ...))` semantics): each parsed expression is evaluated in
/// order so that side effects (assignments, `library()`, …) take effect, and
/// the value of the final one is returned. An empty / whitespace-only string
/// yields `R_NilValue`.
///
/// # Safety
///
/// - Must be called from (or routed to) the R main thread. The parse and eval
///   FFI calls go through the checked `#[r_ffi_checked]` variants, which
///   serialize onto the R thread via `with_r_thread`, so calling from a
///   worker thread is sound — but the returned SEXP must not outlive the R
///   session.
/// - `env` must be a valid ENVSXP.
///
/// # Returns
///
/// - `Ok(SEXP)` with the value of the last expression (**unprotected** — the
///   caller should protect it if further allocations will occur before use).
/// - `Err(REvalError)` if evaluation raises an R error: R's condition, caught
///   as in [`RCall::eval`], so it never longjmps through Rust frames. A parse
///   failure (syntax error / incomplete input) has no R condition; it comes
///   back as a `simpleError` whose message names the failure and the source.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::expression::r_eval_str;
/// use miniextendr_api::sys::R_GlobalEnv;
///
/// unsafe {
///     let three = r_eval_str("1L + 2L", R_GlobalEnv)?;
///     // three is an INTSXP holding 3
/// }
/// ```
pub unsafe fn r_eval_str(code: &str, env: SEXP) -> Result<SEXP, REvalError> {
    unsafe {
        // 1. Wrap the source in a length-1 STRSXP. scalar_string_from_str
        //    allocates a CHARSXP + STRSXP; protect it across the parse, which
        //    allocates again.
        let code_sexp = OwnedProtect::new(SEXP::scalar_string_from_str(code));

        // 2. Parse. R_ParseVector returns an EXPRSXP (a vector of expressions).
        //    Protect it across the subsequent VECTOR_ELT / Rf_eval allocations.
        let mut status = ParseStatus::PARSE_NULL;
        let parsed = R_ParseVector(code_sexp.get(), -1, &mut status, sys::R_NilValue);

        let parse_failure = match status {
            ParseStatus::PARSE_OK => None,
            ParseStatus::PARSE_INCOMPLETE => Some(format!(
                "incomplete R expression (unbalanced delimiter?): {code}"
            )),
            ParseStatus::PARSE_ERROR => Some(format!("R syntax error while parsing: {code}")),
            ParseStatus::PARSE_EOF => {
                Some(format!("unexpected end of input while parsing: {code}"))
            }
            ParseStatus::PARSE_NULL => {
                Some(format!("R_ParseVector returned PARSE_NULL for: {code}"))
            }
        };
        if let Some(message) = parse_failure {
            return Err(REvalError::simple_error(&message));
        }

        let parsed = OwnedProtect::new(parsed);

        // 3. Evaluate each parsed expression in order; return the value of the
        //    last one. An empty EXPRSXP (blank source) yields R_NilValue.
        let n = parsed.get().xlength();
        let mut result = sys::R_NilValue;
        for i in 0..n {
            // VECTOR_ELT borrows from `parsed` (still protected). The element
            // is part of the protected EXPRSXP, so it stays reachable.
            let expr = parsed.get().vector_elt(i);
            result = try_eval(expr, env)?;
        }

        Ok(result)
    }
}

/// Parse and evaluate a string of R source in `R_GlobalEnv`.
///
/// Convenience wrapper over [`r_eval_str`] for the common case. See that
/// function for safety and return semantics.
///
/// # Safety
///
/// Same as [`r_eval_str`].
#[inline]
pub unsafe fn r_eval_str_global(code: &str) -> Result<SEXP, REvalError> {
    unsafe { r_eval_str(code, R_GlobalEnv) }
}
// endregion

// region: REvalError (an R error caught by RCall::eval / r_eval_str)

/// The trailing classes [`REvalError::specific_classes`] leaves out: R's base
/// error layers, and the `rust_error` layer a miniextendr function adds.
/// `error!` and a `Result` return add them again.
const BASE_ERROR_CLASSES: [&str; 4] = ["rust_error", "simpleError", "error", "condition"];

/// The message of the `simpleError` that stands for an evaluation that jumped
/// to the top level without an error condition.
const EXIT_MESSAGE: &str =
    "R evaluation was interrupted (it jumped to the top level without an error condition)";

/// An R error caught by [`RCall::eval`] / [`RCall::eval_base`], [`r_eval_str`]
/// and the helpers built on them ([`REnv::package_namespace`],
/// [`RCall::namespaced`], [`dollar_extract`]).
///
/// It holds R's condition object, rooted while this value lives, and what R
/// reports about it:
///
/// - [`message`](Self::message): `conditionMessage(cond)`, the text R prints
///   after `Error in <call> :`, without that prefix and without the `Calls:`
///   traceback line of a non-interactive session;
/// - [`call`](Self::call): `conditionCall(cond)`, `None` when it is `NULL`;
/// - [`classes`](Self::classes): the class vector, most specific first;
/// - [`condition`](Self::condition): the condition object itself.
///
/// `Display` writes the message alone. Every value holds a condition: a parse
/// failure in [`r_eval_str`] and an evaluation that jumps to the top level
/// without an error (an interrupt) come back as a `simpleError` built for
/// them, with no call.
///
/// # Raising your own condition with it
///
/// The condition macros take the message through `Display`, and
/// [`reraise_class`](Self::reraise_class) puts the package's own classes
/// ahead of the caught ones:
///
/// ```ignore
/// use miniextendr_api::rust_error;
///
/// match RCall::new("print").arg(x).named_arg("na.print", na).eval_base() {
///     Ok(_) => {}
///     // The package's own class with R's message:
///     //   class = "pkg_print_error"
///     // or with the caught classes kept after it, so a handler for those
///     // still matches:
///     Err(e) => rust_error!(class = e.reraise_class("pkg_print_error"), "{e}"),
/// }
/// ```
///
/// A `#[miniextendr]` function returning `Result<T, REvalError>` raises the
/// caught message with the caught [specific classes](Self::specific_classes)
/// (`REvalError` implements [`RConditionError`](crate::condition::RConditionError)).
/// For a `Result<T, condition::RError>` return, `RError::from(e)` takes the
/// message (it is a `std::error::Error`) and `.class(...)` adds the classes.
///
/// # Threads
///
/// `!Send + !Sync`: dropping the value releases its root on R's precious list,
/// which is R main-thread state, and [`condition`](Self::condition) /
/// [`call`](Self::call) hand out SEXPs. To carry the failure to another thread
/// or into a `Send` error type (`anyhow::Error`, `Box<dyn Error + Send>`),
/// take [`message`](Self::message) or convert into
/// [`condition::RError`](crate::condition::RError).
pub struct REvalError {
    /// `conditionMessage(cond)`.
    message: String,
    /// `class(cond)`.
    classes: Vec<String>,
    /// `list(cond, call)`, on R's precious list while this value lives, where
    /// `call` is `conditionCall(cond)` or `NULL` (see [`REvalError::call`]).
    roots: SEXP,
    /// `!Send + !Sync`, see the type docs.
    _not_send: PhantomData<*mut ()>,
}

impl REvalError {
    /// The condition's message (`conditionMessage(cond)`), without R's
    /// `Error in <call> :` prefix and `Calls:` line.
    #[inline]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// The condition's call (`conditionCall(cond)`), `None` when it is `NULL`.
    ///
    /// Also `None` for an error raised at the top of the evaluated expression
    /// (`stop()` called directly rather than from a function): R gives such
    /// an error the call of the `tryCatch()` frame the evaluation runs in,
    /// `doTryCatch(return(expr), name, parentenv, handler)`, which is no call
    /// of the evaluated code. The [condition object](Self::condition) keeps
    /// what R recorded.
    ///
    /// The SEXP is rooted while this value lives; protect it to keep it longer.
    #[inline]
    pub fn call(&self) -> Option<SEXP> {
        let call = self.roots.vector_elt(1);
        (!call.is_nil()).then_some(call)
    }

    /// The condition's class vector, most specific first: for `stop("boom")`,
    /// `["simpleError", "error", "condition"]`.
    #[inline]
    pub fn classes(&self) -> &[String] {
        &self.classes
    }

    /// Whether the condition's class vector contains `class`.
    #[inline]
    pub fn inherits(&self, class: &str) -> bool {
        self.classes.iter().any(|c| c == class)
    }

    /// The classes ahead of the base layers: the class vector without its
    /// trailing `rust_error`, `simpleError`, `error` and `condition`, which the
    /// condition macros add again. Empty for `stop("boom")`; `["my_class"]`
    /// for `stop(errorCondition("boom", class = "my_class"))` or for a
    /// miniextendr function's `error!(class = "my_class", "boom")`.
    pub fn specific_classes(&self) -> &[String] {
        let end = self
            .classes
            .iter()
            .rposition(|c| !BASE_ERROR_CLASSES.contains(&c.as_str()))
            .map_or(0, |last| last + 1);
        &self.classes[..end]
    }

    /// The class vector for raising this error again as the package's own
    /// condition: `own` first (one class or several, most specific first),
    /// then the caught [specific classes](Self::specific_classes) not already
    /// in `own`.
    ///
    /// ```ignore
    /// // stop(errorCondition("boom", class = "vctrs_error")) caught as `e`:
    /// rust_error!(class = e.reraise_class(["pkg_print_error", "pkg_error"]), "{e}");
    /// // class: pkg_print_error, pkg_error, vctrs_error, rust_error,
    /// //        simpleError, error, condition
    /// ```
    pub fn reraise_class(&self, own: impl ConditionClass) -> Vec<String> {
        let mut classes = own.into_condition_class();
        for class in self.specific_classes() {
            if !classes.contains(class) {
                classes.push(class.clone());
            }
        }
        classes
    }

    /// The condition object.
    ///
    /// The SEXP is rooted while this value lives; protect it to keep it longer.
    #[inline]
    pub fn condition(&self) -> SEXP {
        self.roots.vector_elt(0)
    }

    /// Take a caught condition: read its classes, message and call, and root
    /// the condition with its call.
    ///
    /// # Safety
    ///
    /// R's main thread; `cond` is rooted by the caller for the call.
    unsafe fn from_condition(cond: SEXP) -> Self {
        unsafe {
            let classes = string_vector(cond.get_class());
            let message = condition_message(cond);
            let call = OwnedProtect::new(condition_call(cond));
            Self::adopt(cond, call.get(), message, classes)
        }
    }

    /// A `simpleError` with `message` and no call, for a failure that raised
    /// no R condition (a parse failure, a jump to the top level).
    ///
    /// # Safety
    ///
    /// R's main thread.
    unsafe fn simple_error(message: &str) -> Self {
        const CLASSES: [&str; 3] = ["simpleError", "error", "condition"];
        unsafe {
            let scope = ProtectScope::new();
            // list(message = <message>, call = NULL): `alloc_list` fills
            // `NULL`, and each fresh element is stored before the next
            // allocation.
            let cond = scope.protect_raw(SEXP::alloc_list(2));
            cond.set_vector_elt(0, SEXP::scalar_string_from_str(message));
            let names = scope.protect_raw(SEXP::alloc_strsxp(2));
            names.set_string_elt(0, SEXP::charsxp("message"));
            names.set_string_elt(1, SEXP::charsxp("call"));
            cond.set_names(names);
            let class = scope.protect_raw(SEXP::alloc_strsxp(3));
            for (i, name) in (0..).zip(CLASSES) {
                class.set_string_elt(i, SEXP::charsxp(name));
            }
            cond.set_class(class);
            Self::adopt(
                cond,
                SEXP::nil(),
                message.to_owned(),
                CLASSES.map(String::from).to_vec(),
            )
        }
    }

    /// Root `cond` and `call` together for the value's lifetime.
    ///
    /// # Safety
    ///
    /// R's main thread; `cond` and `call` are rooted by the caller for the
    /// call.
    unsafe fn adopt(cond: SEXP, call: SEXP, message: String, classes: Vec<String>) -> Self {
        unsafe {
            let roots = OwnedProtect::new(SEXP::alloc_list(2));
            roots.get().set_vector_elt(0, cond);
            roots.get().set_vector_elt(1, call);
            sys::R_PreserveObject(roots.get());
            REvalError {
                message,
                classes,
                roots: roots.get(),
                _not_send: PhantomData,
            }
        }
    }
}

impl Drop for REvalError {
    fn drop(&mut self) {
        // SAFETY: the value never left the thread it was made on (`!Send`):
        // R's main thread, or a worker whose checked calls route there.
        unsafe { sys::R_ReleaseObject(self.roots) };
    }
}

impl std::fmt::Display for REvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::fmt::Debug for REvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("REvalError")
            .field("message", &self.message)
            .field("classes", &self.classes)
            .finish_non_exhaustive()
    }
}

impl std::error::Error for REvalError {}

/// Returned as a `#[miniextendr]` function's `Err`, the caught error is raised
/// again with its message and its [specific classes](REvalError::specific_classes),
/// ahead of the `rust_error` layers; the call is the function's own.
impl crate::condition::RConditionError for REvalError {
    fn message(&self) -> String {
        self.message.clone()
    }

    fn class(&self) -> Vec<String> {
        self.specific_classes().to_vec()
    }
}

/// The elements of a character vector (`NA` as `"NA"`); empty for anything
/// else. Does not allocate.
///
/// # Safety
///
/// R's main thread.
unsafe fn string_vector(x: SEXP) -> Vec<String> {
    if x.type_of() != crate::SEXPTYPE::STRSXP {
        return Vec::new();
    }
    (0..x.xlength())
        .map(|i| {
            // SAFETY: `x` is a STRSXP, so its elements are CHARSXPs.
            unsafe { crate::from_r::charsxp_to_string_lossy(x.string_elt(i)) }
                .unwrap_or_else(|| "NA".to_owned())
        })
        .collect()
}

/// `conditionMessage(cond)`, through S3 dispatch, so a class with its own
/// method (rlang's errors, say) words its message. If the method fails or
/// returns no string: the condition's `message` field, then a placeholder.
///
/// # Safety
///
/// R's main thread; `cond` rooted.
unsafe fn condition_message(cond: SEXP) -> String {
    unsafe {
        let call = OwnedProtect::new(RCall::new("conditionMessage").quoted_arg(cond).build());
        if let Caught::Value(message) = try_eval_raw(call.get(), R_BaseEnv) {
            let lines = string_vector(message);
            if !lines.is_empty() {
                return lines.join("\n");
            }
        }
        if cond.type_of() == crate::SEXPTYPE::VECSXP {
            let names = string_vector(cond.get_names());
            if let Some(i) = names.iter().position(|name| name == "message") {
                let i = isize::try_from(i).expect("list index exceeds isize::MAX");
                let lines = string_vector(cond.vector_elt(i));
                if !lines.is_empty() {
                    return lines.join("\n");
                }
            }
        }
        "R error with no message".to_owned()
    }
}

/// `conditionCall(cond)`, or `NULL` if the method fails or the call is the
/// `tryCatch()` frame's ([`is_try_catch_frame_call`]). Unprotected.
///
/// # Safety
///
/// R's main thread; `cond` rooted.
unsafe fn condition_call(cond: SEXP) -> SEXP {
    unsafe {
        let call = OwnedProtect::new(RCall::new("conditionCall").quoted_arg(cond).build());
        match try_eval_raw(call.get(), R_BaseEnv) {
            Caught::Value(call) if !is_try_catch_frame_call(call) => call,
            Caught::Value(_) | Caught::Error(_) | Caught::Exit => SEXP::nil(),
        }
    }
}

/// Whether `call` is `doTryCatch(return(expr), name, parentenv, handler)`,
/// the frame of R's `tryCatch()`, which `R_tryCatchError` evaluates in. An
/// error raised at the top of the evaluated expression (`stop()` called
/// directly, not from a function) takes that frame's call as its own; it is
/// no call of the user's code, so [`REvalError::call`] reports `None` there.
/// Base R's `try()` recognises the frame by the same `doTryCatch` head.
fn is_try_catch_frame_call(call: SEXP) -> bool {
    call.type_of() == crate::SEXPTYPE::LANGSXP
        // SAFETY: `doTryCatch` is a literal without NUL; interning is main-thread work.
        && call.car() == unsafe { RSymbol::from_cstr(c"doTryCatch") }.as_sexp()
}

// endregion

// region: try_eval (evaluation with R errors caught)

/// How [`try_eval_raw`] ended. The SEXPs are unprotected.
enum Caught {
    /// The expression's value.
    Value(SEXP),
    /// The `error` condition the evaluation raised.
    Error(SEXP),
    /// A jump to the top level without an error condition: an interrupt, an
    /// `abort` restart.
    Exit,
}

/// What [`try_eval_raw`] shares with its callbacks.
struct TryEvalData {
    expr: SEXP,
    env: SEXP,
    /// A protected list of one, which receives the value or the caught
    /// condition. The callbacks store into it without allocating, so what
    /// they hand back is rooted before R runs again.
    slot: SEXP,
    /// Whether the error handler ran.
    caught: bool,
}

/// Evaluate `expr` in `env`, hidden from the caller's handlers and restarts
/// (a new top-level context, `R_ToplevelExec`), with `error` conditions
/// caught (`R_tryCatchError`).
///
/// `R_tryCatchError` alone is not enough: it runs R's `tryCatch()`, so the
/// caller's calling handlers would see the evaluation's warnings, and an
/// exiting handler or restart of the caller would jump through the Rust
/// frames. `R_ToplevelExec` gives the isolation `R_tryEval` gives, and stops
/// any jump that is not an error.
///
/// # Safety
///
/// R's main thread (the checked FFI routes a worker's call there); `expr`
/// and `env` rooted.
unsafe fn try_eval_raw(expr: SEXP, env: SEXP) -> Caught {
    /// `R_ToplevelExec`'s callback. A jump out of it crosses no Rust frame
    /// holding anything to drop.
    unsafe extern "C-unwind" fn toplevel(data: *mut c_void) {
        let data = data.cast::<TryEvalData>();
        unsafe {
            let value = sys::R_tryCatchError(Some(body), data.cast(), Some(handler), data.cast());
            if !(*data).caught {
                (*data).slot.set_vector_elt(0, value);
            }
        }
    }

    /// `R_tryCatchError`'s body. An R error jumps from here to the
    /// `tryCatch()` frame, over no Rust frame holding anything to drop.
    unsafe extern "C-unwind" fn body(data: *mut c_void) -> SEXP {
        let data = data.cast::<TryEvalData>();
        unsafe { sys::Rf_eval((*data).expr, (*data).env) }
    }

    /// `R_tryCatchError`'s handler, run once R has unwound to the
    /// `tryCatch()` frame: keep the condition.
    unsafe extern "C-unwind" fn handler(cond: SEXP, data: *mut c_void) -> SEXP {
        let data = data.cast::<TryEvalData>();
        unsafe {
            (*data).slot.set_vector_elt(0, cond);
            (*data).caught = true;
        }
        SEXP::nil()
    }

    unsafe {
        let slot = OwnedProtect::new(SEXP::alloc_list(1));
        let mut data = TryEvalData {
            expr,
            env,
            slot: slot.get(),
            caught: false,
        };
        let completed = sys::R_ToplevelExec(Some(toplevel), (&raw mut data).cast());
        if completed == crate::sexp_types::Rboolean::FALSE {
            return Caught::Exit;
        }
        let value = slot.get().vector_elt(0);
        if data.caught {
            Caught::Error(value)
        } else {
            Caught::Value(value)
        }
    }
}

/// [`try_eval_raw`] with a caught error as an [`REvalError`]. The value is
/// unprotected.
///
/// # Safety
///
/// As [`try_eval_raw`].
unsafe fn try_eval(expr: SEXP, env: SEXP) -> Result<SEXP, REvalError> {
    unsafe {
        match try_eval_raw(expr, env) {
            Caught::Value(value) => Ok(value),
            Caught::Error(cond) => {
                let cond = OwnedProtect::new(cond);
                Err(REvalError::from_condition(cond.get()))
            }
            Caught::Exit => Err(REvalError::simple_error(EXIT_MESSAGE)),
        }
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
        fn assert_env_result_fn<F: FnOnce() -> Result<REnv, REvalError>>(_f: F) {}

        assert_env_fn(|| unsafe { REnv::global() });
        assert_env_fn(|| unsafe { REnv::base() });
        assert_env_fn(|| unsafe { REnv::empty() });
        assert_env_fn(REnv::base_namespace);
        assert_env_fn(|| unsafe { REnv::caller() });
        assert_env_result_fn(|| unsafe { REnv::package_namespace("base") });
    }
}
// endregion
