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
//! | [`REvalError`] | An R error caught by [`RCall::eval`] / [`r_eval_str`] / [`try_eval_with_handlers`] |
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
use crate::from_r::TryFromSexp;
use crate::gc_protect::{OwnedProtect, ProtectScope};
use crate::into_r::IntoR;
use crate::protect_pool::ProtectPool;
use crate::sexp_ext::PairListExt;
use crate::sys::{
    self, ParseStatus, R_BaseEnv, R_EmptyEnv, R_GlobalEnv, R_ParseVector, Rf_install,
};
use crate::{SEXP, SexpExt};
use std::ffi::{CStr, CString};
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

    /// The value bound to `name` in this environment's own frame, as R's
    /// `get0(name, envir = env, inherits = FALSE)` reads it, or `None` when
    /// the frame has no such binding. The enclosing environments are not
    /// searched; [`get_inherits`](Self::get_inherits) searches them.
    ///
    /// A binding to `NULL` is `Some` of `NULL`. Reading a binding does what
    /// `get0()` does:
    ///
    /// - A promise is forced, once, and its value is kept in the promise. A
    ///   function in a lazy-loaded namespace, a `delayedAssign()` and an
    ///   argument of an R function are promises. So this is not a check for
    ///   whether a name is bound without running anything.
    /// - An active binding (`makeActiveBinding()`) calls its function.
    /// - A binding to the missing argument raises R's error.
    ///
    /// R 4.5 and later read the binding with `R_getVarEx`; R 4.4 evaluates
    /// `base::get0()`. Both give the same results. An R error raised while
    /// reading, such as a promise or an active binding that fails, unwinds the
    /// Rust frames up to the miniextendr boundary, which hands it back to R,
    /// as under [`eval_with_handlers`].
    ///
    /// # Safety
    ///
    /// R's main thread, inside a miniextendr boundary: a `#[miniextendr]`
    /// function body, a [`with_r_unwind_protect`](crate::unwind_protect::with_r_unwind_protect)
    /// closure, an ALTREP callback with the `r_unwind` or `rust_unwind` guard,
    /// a connection callback, a [`with_r_thread`](crate::worker::with_r_thread)
    /// closure.
    ///
    /// # Returns
    ///
    /// The value, **unprotected**. A plain binding, or a forced promise,
    /// keeps it reachable while the name stays bound to it. An active
    /// binding's value is new and held by nothing. Protect the value
    /// ([`OwnedProtect`], [`Preserved`](crate::Preserved)) before the next
    /// allocation unless the binding is known to be a plain value, and before
    /// anything that may rebind the name: R code you evaluate, or
    /// [`define`](Self::define).
    ///
    /// # Panics
    ///
    /// Panics if `name` contains a null byte.
    pub unsafe fn get(&self, name: &str) -> Option<SEXP> {
        unsafe { self.lookup(name, false) }
    }

    /// The value bound to `name` in this environment or the nearest enclosing
    /// environment that binds it, as R's `get0(name, envir = env,
    /// inherits = TRUE)` reads it, or `None` when none does.
    ///
    /// Bindings are read as [`get`](Self::get) reads them: a promise is
    /// forced, an active binding is called. The search stops at the first
    /// binding of `name`, whatever its value.
    ///
    /// # Safety
    ///
    /// As [`get`](Self::get).
    ///
    /// # Returns
    ///
    /// The value, **unprotected**, as [`get`](Self::get) returns it.
    ///
    /// # Panics
    ///
    /// Panics if `name` contains a null byte.
    pub unsafe fn get_inherits(&self, name: &str) -> Option<SEXP> {
        unsafe { self.lookup(name, true) }
    }

    /// Bind `name` to `value` in this environment's own frame, as
    /// `assign(name, value, envir = env)` does: a binding of `name` in the
    /// frame is replaced, and the enclosing environments are not touched.
    ///
    /// R raises its error, which unwinds as under [`get`](Self::get), when
    /// the binding cannot be made: the empty environment, a locked
    /// environment that lacks the binding (a package namespace, once the
    /// package is loaded), or a locked binding. Assigning to an active binding
    /// calls its function with `value`.
    ///
    /// # Safety
    ///
    /// As [`get`](Self::get). `value` is a valid SEXP; it needs no root of
    /// its own, and the binding keeps it reachable afterwards.
    ///
    /// # Panics
    ///
    /// Panics if `name` contains a null byte.
    pub unsafe fn define(&self, name: &str, value: SEXP) {
        assert!(
            !name.contains('\0'),
            "a binding name must not contain null bytes"
        );
        unsafe { self.define_symbol(SEXP::symbol(name), value) }
    }

    /// [`define`](Self::define) for a name already interned as a symbol.
    ///
    /// # Safety
    ///
    /// As [`define`](Self::define); `symbol` is a SYMSXP.
    pub(crate) unsafe fn define_symbol(&self, symbol: SEXP, value: SEXP) {
        struct Define {
            symbol: SEXP,
            value: SEXP,
            env: SEXP,
        }

        unsafe extern "C-unwind" fn define(data: *mut c_void) -> SEXP {
            let data = unsafe { &*data.cast::<Define>() };
            unsafe { sys::Rf_defineVar(data.symbol, data.value, data.env) };
            SEXP::nil()
        }

        unsafe {
            // Rooted ahead of the evaluation's own protect frame, so an R exit
            // leaves the protect stack as the scope expects.
            let scope = ProtectScope::new();
            let env = scope.protect_raw(self.sexp);
            let value = scope.protect_raw(value);
            let mut data = Define { symbol, value, env };
            crate::unwind_protect::call_unwinding(define, (&raw mut data).cast());
        }
    }

    /// [`get`](Self::get) and [`get_inherits`](Self::get_inherits).
    ///
    /// # Safety
    ///
    /// As [`get`](Self::get).
    unsafe fn lookup(&self, name: &str, inherits: bool) -> Option<SEXP> {
        assert!(
            !name.contains('\0'),
            "a binding name must not contain null bytes"
        );

        // Each branch roots the environment ahead of the evaluation's own
        // protect frame, so an R exit leaves the protect stack as the guards
        // expect.
        #[cfg(r_4_5)]
        unsafe {
            struct Lookup {
                symbol: SEXP,
                env: SEXP,
                inherits: crate::Rboolean,
            }

            unsafe extern "C-unwind" fn lookup(data: *mut c_void) -> SEXP {
                let data = unsafe { &*data.cast::<Lookup>() };
                unsafe {
                    sys::R_getVarEx(data.symbol, data.env, data.inherits, sys::R_UnboundValue)
                }
            }

            let env = OwnedProtect::new(self.sexp);
            let mut data = Lookup {
                symbol: SEXP::symbol(name),
                env: env.get(),
                inherits: crate::Rboolean::from(inherits),
            };
            let value = crate::unwind_protect::call_unwinding(lookup, (&raw mut data).cast());
            (value != sys::R_UnboundValue).then_some(value)
        }

        #[cfg(not(r_4_5))]
        unsafe {
            let env = OwnedProtect::new(self.sexp);
            // A new object, so no binding holds it: `get0()` returns it only
            // for an unbound name, which tells an unbound name from one bound
            // to `NULL`.
            let unbound = OwnedProtect::new(SEXP::alloc_list(0));
            // Evaluated in the base environment, so `get0` is base's own.
            let value = RCall::new("get0")
                .arg_value(name)
                .named_arg("envir", env.get())
                .named_value("mode", "any")
                .named_value("inherits", inherits)
                .named_arg("ifnotfound", unbound.get())
                .eval_with_handlers(R_BaseEnv);
            (value != unbound.get()).then_some(value)
        }
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

    /// Convert a Rust value and add it as a positional argument.
    ///
    /// The converted value is rooted before anything else allocates, so a
    /// value made only for this call needs no root of its own. Write
    /// `RCall::new("qt").arg_value(p).arg_value(df)`, not
    /// `p.into_sexp()` and `df.into_sexp()` first: the second conversion, and
    /// [`new`](Self::new), allocate while the first value is not rooted.
    ///
    /// # Panics
    ///
    /// Panics if the conversion fails, as [`IntoR::into_sexp`] does.
    #[inline]
    pub fn arg_value(self, value: impl IntoR) -> Self {
        self.arg(value.into_sexp())
    }

    /// Convert a Rust value and add it as a named argument. See
    /// [`arg_value`](Self::arg_value).
    ///
    /// # Panics
    ///
    /// Panics if `name` contains a null byte, or if the conversion fails.
    #[inline]
    pub fn named_value(self, name: &str, value: impl IntoR) -> Self {
        self.named_arg(name, value.into_sexp())
    }

    /// Add R's `...` at this position: the dots of the frame the call is
    /// evaluated in, each passed on as the promise R matched, unevaluated.
    ///
    /// Only [`LazyDots::forward`](crate::LazyDots::forward) /
    /// [`try_forward`](crate::LazyDots::try_forward) evaluate a call in a
    /// frame that has dots (the generated wrapper's), and they add `...` last
    /// themselves, so this is for putting it elsewhere:
    /// `RCall::new("f").dots().named_value("na.rm", true)` is
    /// `f(..., na.rm = TRUE)`. Evaluated in an environment without dots, R
    /// raises "'...' used in an incorrect context".
    #[inline]
    pub fn dots(self) -> Self {
        // SAFETY: symbols are interned for the session and need no root.
        self.arg(unsafe { dots_symbol() })
    }

    /// Whether the call passes `...` ([`dots`](Self::dots)).
    pub(crate) fn has_dots(&self) -> bool {
        // SAFETY: as in `dots`.
        let dots = unsafe { dots_symbol() };
        self.args
            .iter()
            .any(|(name, value)| name.is_none() && *value == dots)
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
    /// conditions the user should see as raised, and
    /// [`try_eval_with_handlers`](Self::try_eval_with_handlers) for one whose
    /// warnings and messages the user should see but whose error the Rust
    /// code handles.
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

    /// Evaluate the call in `env`, as [`eval`](Self::eval) does, and read the
    /// result as `T`.
    ///
    /// The result stays rooted while it is converted, so the caller never
    /// holds it unrooted:
    ///
    /// ```ignore
    /// let q: Vec<Option<f64>> = RCall::new("qt")
    ///     .arg_value(p)
    ///     .arg_value(df)
    ///     .eval_as(stats.as_sexp())?;
    /// ```
    ///
    /// A result that `T` can't read is an `Err` too: a `simpleError`
    /// whose message is the conversion's, with no call. A `T` that only
    /// views the value (`SEXP`, `List`, `DataFrame`) comes back unrooted, as
    /// from [`eval`](Self::eval); read into an owned type (`Vec<T>`,
    /// `String`, a struct) or protect it.
    ///
    /// # Safety
    ///
    /// As [`eval`](Self::eval).
    pub unsafe fn eval_as<T>(&self, env: SEXP) -> Result<T, REvalError>
    where
        T: TryFromSexp,
        T::Error: std::fmt::Display,
    {
        unsafe {
            let value = OwnedProtect::new(self.eval(env)?);
            T::try_from_sexp(value.get()).map_err(|e| REvalError::simple_error(&e.to_string()))
        }
    }

    /// [`eval_as`](Self::eval_as) in `R_BaseEnv`.
    ///
    /// # Safety
    ///
    /// As [`eval`](Self::eval).
    #[inline]
    pub unsafe fn eval_base_as<T>(&self) -> Result<T, REvalError>
    where
        T: TryFromSexp,
        T::Error: std::fmt::Display,
    {
        unsafe { self.eval_as(R_BaseEnv) }
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
    /// Rust frames and reaches the user's `tryCatch()` as it was raised. To
    /// get the error back as `Err(REvalError)` instead, while the user's
    /// handlers still see warnings and messages, use
    /// [`try_eval_with_handlers`](Self::try_eval_with_handlers).
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

    /// Evaluate the call in `env` in the caller's R context, keeping R's
    /// condition handlers, with an R error returned as `Err`: see
    /// [`try_eval_with_handlers`].
    ///
    /// Use it for a call that forwards the user's arguments to an R function
    /// and raises the function's errors as the package's own: the user's
    /// `withCallingHandlers()`, `suppressWarnings()` and
    /// `tryCatch(warning = )` see the function's warnings and messages, and
    /// an R error comes back as [`REvalError`]. A `[` method of a data-frame
    /// subclass, say:
    ///
    /// ```ignore
    /// let out = unsafe {
    ///     RCall::new("[")
    ///         .arg(frame)
    ///         .arg(i)
    ///         .named_arg("drop", drop)
    ///         .try_eval_with_handlers(R_BaseEnv)
    /// }
    /// .unwrap_or_else(|e| rust_error!(class = e.reraise_class("pkg_subset_error"), "{e}"));
    /// ```
    ///
    /// `x[1, drop = FALSE]` warns that `drop` is ignored; the user's
    /// `suppressWarnings()` silences it. A column that does not exist
    /// ("undefined columns selected") comes back as `Err` and is raised again
    /// under `pkg_subset_error`.
    ///
    /// # Safety
    ///
    /// As [`try_eval_with_handlers`]: R's main thread, inside a miniextendr
    /// boundary, `env` an environment. The value is unprotected.
    pub unsafe fn try_eval_with_handlers(&self, env: SEXP) -> Result<SEXP, REvalError> {
        unsafe {
            let call = OwnedProtect::new(self.build());
            try_eval_with_handlers(call.get(), env)
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

/// The symbol `...`.
///
/// # Safety
///
/// R's main thread. Symbols are never collected.
unsafe fn dots_symbol() -> SEXP {
    unsafe { Rf_install(c"...".as_ptr()) }
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

// region: eval_with_handlers, try_eval_with_handlers (evaluation in the caller's R context)

/// Evaluate `expr` in `env` in the caller's R context, the way R code calling
/// `eval(expr, env)` does.
///
/// | | [`RCall::eval`], [`r_eval_str`] | `eval_with_handlers` | [`try_eval_with_handlers`] |
/// |---|---|---|---|
/// | R entry point | `R_tryCatchError` inside `R_ToplevelExec` | `Rf_eval` in its own `R_UnwindProtect` | `R_tryCatchError` in its own `R_UnwindProtect` |
/// | caller's `withCallingHandlers()`, `suppressWarnings()` | not seen (`R_ToplevelExec` empties the handler and restart stacks) | see every warning, message and condition | see every warning, message and condition except an error |
/// | R error | `Err(`[`REvalError`]`)`: R's condition, with its message (no `Error in` prefix, no `Calls:` line), call and classes; the Rust code decides what to raise | unwinds as the R condition, class and call kept | `Err(`[`REvalError`]`)`, as from [`RCall::eval`] |
/// | caller's `tryCatch(warning = )`, restarts, interrupts | not seen (an interrupt ends the evaluation with an `Err`) | exit through the Rust frames | exit through the Rust frames |
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
/// ([`REvalError::reraise_class`]). [`try_eval_with_handlers`] combines the
/// two: the caller's handlers see warnings and messages, and an R error comes
/// back as `Err` for the Rust code to raise.
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

/// Evaluate `expr` in `env` in the caller's R context, as
/// [`eval_with_handlers`] does, but return an R error as `Err` instead of
/// letting it unwind.
///
/// The caller's `withCallingHandlers()`, `suppressWarnings()` and
/// `suppressMessages()` see the evaluation's warnings, messages and other
/// conditions; one they muffle returns here and the evaluation goes on. A
/// condition of class `error` is caught at the evaluation, by an exiting
/// handler (`R_tryCatchError`), and comes back as `Err(`[`REvalError`]`)`
/// built as [`RCall::eval`] builds it: R's condition, with its message (no
/// `Error in` prefix, no `Calls:` line), call and classes. The Rust code
/// decides what to raise ([`REvalError::reraise_class`]).
///
/// Any other exit (an exiting handler of the caller's `tryCatch()`, such as
/// `tryCatch(warning = )`, `invokeRestart()`, an interrupt) leaves as under
/// [`eval_with_handlers`]: it stops at this function's own `R_UnwindProtect`
/// frame and goes on as a Rust unwind, the frames up to the `#[miniextendr]`
/// boundary drop their values, and the boundary hands the exit back to R.
///
/// The error handler is the innermost one, so a handler of the caller's for
/// errors (`withCallingHandlers(error = )`, `tryCatch(error = )`) does not see
/// an error the evaluation raises. An error raised by a handler of the
/// caller's, as in `withCallingHandlers(warning = function(w) stop("no"))`,
/// is not the evaluation's: R runs that handler outside the evaluation's
/// handlers, so the error unwinds to the caller as raised.
///
/// The evaluation runs inside R's `tryCatch()`, as under [`RCall::eval`]: an
/// error raised at the top of `expr` (`stop()` called directly, not from a
/// function) has no [`call`](REvalError::call), and code at the top of `expr`
/// that inspects the call stack (`sys.call()`) sees the `tryCatch()` frames.
///
/// The rules of [`eval_with_handlers`] for the unwind hold here too: don't
/// catch it with `std::panic::catch_unwind` on its way to the boundary, and
/// don't call this from a `Drop` implementation.
///
/// # Safety
///
/// As [`eval_with_handlers`]:
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
/// - `Ok(SEXP)` with the value, **unprotected**: protect it before the next
///   allocation.
/// - `Err(REvalError)` with R's error condition.
pub unsafe fn try_eval_with_handlers(expr: SEXP, env: SEXP) -> Result<SEXP, REvalError> {
    unsafe { into_result(try_eval_unwinding(expr, env)) }
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

/// An R error caught by [`RCall::eval`] / [`RCall::eval_base`], [`r_eval_str`],
/// the helpers built on them ([`REnv::package_namespace`],
/// [`RCall::namespaced`], [`dollar_extract`]), and [`try_eval_with_handlers`] /
/// [`RCall::try_eval_with_handlers`].
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
/// failure in [`r_eval_str`] and an evaluation under [`RCall::eval`] that
/// jumps to the top level without an error (an interrupt) come back as a
/// `simpleError` built for them, with no call.
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
/// [`reraise`](Self::reraise) raises the caught condition itself instead, as
/// R raised it.
///
/// A `#[miniextendr]` function returning `Result<T, REvalError>` raises the
/// caught message with the caught [specific classes](Self::specific_classes)
/// (`REvalError` implements [`RConditionData`](crate::condition::RConditionData)).
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
    /// Its handle makes the value `!Send + !Sync`, see the type docs.
    roots: crate::Preserved,
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

    /// Raise the caught condition again, as R raised it: its classes, message
    /// and fields are unchanged, and its call is [`call`](Self::call). So an
    /// error raised at the top of the evaluated expression reports no call,
    /// not R's `doTryCatch(...)` frame.
    ///
    /// Use it for the errors a body doesn't explain itself, next to a
    /// package's own condition for the ones it does
    /// ([`Quoted::try_eval_in`](crate::Quoted::try_eval_in) has an example).
    /// Unlike `rust_error!(class = e.reraise_class(..), "{e}")`, the condition
    /// gets no `rust_error` class and keeps R's call, not the wrapper's.
    ///
    /// The condition is signalled by base `stop(cond)`, evaluated as
    /// [`eval_with_handlers`] evaluates: the caller's handlers see it, and the
    /// exit stops at that evaluation's `R_UnwindProtect` frame and leaves as a
    /// Rust unwind. The frames up to the `#[miniextendr]` boundary drop their
    /// values, and the boundary hands the exit back to R, which goes on to the
    /// caller's handler or to the top level. The same rules hold as for
    /// [`eval_with_handlers`]: don't catch the unwind with
    /// `std::panic::catch_unwind`, and don't call this from a `Drop`
    /// implementation.
    ///
    /// # Safety
    ///
    /// As [`eval_with_handlers`]: R's main thread, inside a miniextendr
    /// boundary (a `#[miniextendr]` function body, a
    /// [`with_r_unwind_protect`](crate::unwind_protect::with_r_unwind_protect)
    /// closure, ...).
    pub unsafe fn reraise(self) -> ! {
        unsafe {
            let scope = ProtectScope::new();
            let cond = scope.protect_raw(without_try_catch_frame_call(self.condition()));
            // `stop(cond)` with the condition bound in a child of base, so a
            // traceback shows `stop(cond)` rather than the deparsed object.
            let env = scope.protect_raw(sys::R_NewEnv(R_BaseEnv, crate::Rboolean::FALSE, 1));
            let cond_sym = RSymbol::from_cstr(c"cond").as_sexp();
            REnv::from_sexp(env).define_symbol(cond_sym, cond);
            let call = scope.protect_raw(sys::Rf_lang2(
                RSymbol::from_cstr(c"stop").as_sexp(),
                cond_sym,
            ));
            eval_with_handlers(call, env);
        }
        unreachable!("stop() returned without signalling an error")
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
    /// no R condition (a parse failure, a jump to the top level, a result
    /// [`RCall::eval_as`] can't read).
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
            let roots = crate::Preserved::new(SEXP::alloc_list(2));
            roots.set_vector_elt(0, cond);
            roots.set_vector_elt(1, call);
            REvalError {
                message,
                classes,
                roots,
            }
        }
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
impl crate::condition::RConditionData for REvalError {
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

/// `cond`, or a shallow copy of it whose `call` element is `NULL` when that
/// element is the [`tryCatch()` frame's call](is_try_catch_frame_call): the
/// condition [`REvalError::reraise`] signals, with the call
/// [`REvalError::call`] reports.
///
/// # Safety
///
/// R's main thread; `cond` rooted by the caller. The result is unprotected.
unsafe fn without_try_catch_frame_call(cond: SEXP) -> SEXP {
    unsafe {
        if cond.type_of() != crate::SEXPTYPE::VECSXP {
            return cond;
        }
        let names = string_vector(cond.get_names());
        let Some(i) = names.iter().position(|name| name == "call") else {
            return cond;
        };
        let i = isize::try_from(i).expect("list index exceeds isize::MAX");
        if !is_try_catch_frame_call(cond.vector_elt(i)) {
            return cond;
        }
        let copy = sys::Rf_shallow_duplicate(cond);
        copy.set_vector_elt(i, SEXP::nil());
        copy
    }
}

// endregion

// region: try_eval (evaluation with R errors caught)

/// How an evaluation with R errors caught ended. The SEXPs are unprotected.
enum Caught {
    /// The expression's value.
    Value(SEXP),
    /// The `error` condition the evaluation raised.
    Error(SEXP),
    /// A jump to the top level without an error condition: an interrupt, an
    /// `abort` restart. Only [`try_eval_raw`] stops those.
    Exit,
}

/// A C callback run with R errors caught, given the caller's data. An R error
/// jumps out of it, so it holds nothing to drop.
type TryBody = unsafe extern "C-unwind" fn(*mut c_void) -> SEXP;

/// What [`catch_error`] shares with its callbacks.
struct TryEvalData {
    /// `R_tryCatchError`'s body: [`eval_body`] for an R expression.
    body: TryBody,
    body_data: *mut c_void,
    /// A protected list of one, which receives the value or the caught
    /// condition. The callbacks store into it without allocating, so what
    /// they hand back is rooted before R runs again.
    slot: SEXP,
    /// Whether the error handler ran.
    caught: bool,
}

impl TryEvalData {
    /// The call of `body(body_data)`; `slot` is a protected list of one.
    fn new(body: TryBody, body_data: *mut c_void, slot: SEXP) -> Self {
        TryEvalData {
            body,
            body_data,
            slot,
            caught: false,
        }
    }

    /// How [`catch_error`] ended, once it has returned. The SEXP is rooted by
    /// the slot.
    fn outcome(&self) -> Caught {
        let value = self.slot.vector_elt(0);
        if self.caught {
            Caught::Error(value)
        } else {
            Caught::Value(value)
        }
    }
}

/// An R expression and the environment to evaluate it in: [`eval_body`]'s
/// data.
struct EvalArgs {
    expr: SEXP,
    env: SEXP,
}

/// Evaluate an [`EvalArgs`]. An R error jumps from here to the `tryCatch()`
/// frame, over no Rust frame holding anything to drop.
unsafe extern "C-unwind" fn eval_body(data: *mut c_void) -> SEXP {
    let data = data.cast::<EvalArgs>();
    unsafe { sys::Rf_eval((*data).expr, (*data).env) }
}

/// Run a [`TryEvalData`]'s body with `error` conditions caught
/// (`R_tryCatchError`), and store its value, or the caught condition, in its
/// slot. Returns `R_tryCatchError`'s value: the body's, or `NULL` after an
/// error ([`TryEvalData::outcome`] tells them apart).
///
/// `R_tryCatchError` runs R's `tryCatch()`, so the caller's calling handlers
/// see the evaluation's other conditions, and any jump that is not an error
/// (an exiting handler or restart of the caller's) passes through. A jump out
/// of here crosses no Rust frame holding anything to drop.
unsafe extern "C-unwind" fn catch_error(data: *mut c_void) -> SEXP {
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

    let data = data.cast::<TryEvalData>();
    unsafe {
        let value = sys::R_tryCatchError(
            Some((*data).body),
            (*data).body_data,
            Some(handler),
            data.cast(),
        );
        if !(*data).caught {
            (*data).slot.set_vector_elt(0, value);
        }
        value
    }
}

/// Call `body(body_data)`, hidden from the caller's handlers and restarts
/// (a new top-level context, `R_ToplevelExec`), with `error` conditions
/// caught ([`catch_error`]).
///
/// `R_tryCatchError` alone is not enough here: the caller's calling handlers
/// would see the evaluation's warnings, and an exiting handler or restart of
/// the caller would jump through the Rust frames. `R_ToplevelExec` gives the
/// isolation `R_tryEval` gives, and stops any jump that is not an error.
///
/// # Safety
///
/// R's main thread (the checked FFI routes a worker's call there);
/// `body(body_data)` is sound to call, and what it reads is rooted.
unsafe fn try_call_raw(body: TryBody, body_data: *mut c_void) -> Caught {
    /// `R_ToplevelExec`'s callback. A jump out of it crosses no Rust frame
    /// holding anything to drop.
    unsafe extern "C-unwind" fn toplevel(data: *mut c_void) {
        unsafe { catch_error(data) };
    }

    unsafe {
        let slot = OwnedProtect::new(SEXP::alloc_list(1));
        let mut data = TryEvalData::new(body, body_data, slot.get());
        let completed = sys::R_ToplevelExec(Some(toplevel), (&raw mut data).cast());
        if completed == crate::sexp_types::Rboolean::FALSE {
            return Caught::Exit;
        }
        data.outcome()
    }
}

/// Evaluate `expr` in `env` as [`try_call_raw`] calls a body.
///
/// # Safety
///
/// R's main thread (the checked FFI routes a worker's call there); `expr`
/// and `env` rooted.
unsafe fn try_eval_raw(expr: SEXP, env: SEXP) -> Caught {
    let mut args = EvalArgs { expr, env };
    unsafe { try_call_raw(eval_body, (&raw mut args).cast()) }
}

/// Call `body(body_data)`, a C callback that may raise an R error, as
/// [`RCall::eval`] evaluates a call: hidden from the caller's handlers and
/// restarts, with an R error returned as `Err`. The value is unprotected.
///
/// For a C entry point that raises R errors, such as `R_WriteConnection`
/// (`connection/access.rs`); an R call needs no body of its own.
///
/// # Safety
///
/// As [`try_call_raw`]. An R error jumps out of `body`, so it must hold
/// nothing to drop.
#[cfg(all(feature = "connections", r_4_6))]
pub(crate) unsafe fn try_call(body: TryBody, body_data: *mut c_void) -> Result<SEXP, REvalError> {
    unsafe { into_result(try_call_raw(body, body_data)) }
}

/// Evaluate `expr` in `env` in the caller's R context, with `error`
/// conditions caught ([`catch_error`]): the caller's handlers and restarts
/// stay in place, and any other R exit leaves as a Rust unwind
/// ([`call_unwinding`](crate::unwind_protect::call_unwinding)). Never
/// [`Caught::Exit`].
///
/// The slot is protected before the evaluation and holds the caught condition
/// until the caller roots it. An exit jumps to `R_UnwindProtect`'s frame,
/// which resets the protect stack to its level when the frame was entered, so
/// the slot's protection is still the top one when the unwind drops it.
///
/// # Safety
///
/// As [`try_eval_with_handlers`].
unsafe fn try_eval_unwinding(expr: SEXP, env: SEXP) -> Caught {
    unsafe {
        let slot = OwnedProtect::new(SEXP::alloc_list(1));
        let mut args = EvalArgs { expr, env };
        let mut data = TryEvalData::new(eval_body, (&raw mut args).cast(), slot.get());
        crate::unwind_protect::call_unwinding(catch_error, (&raw mut data).cast());
        data.outcome()
    }
}

/// An evaluation's outcome with a caught error as an [`REvalError`]. The
/// value is unprotected.
///
/// # Safety
///
/// R's main thread, right after the evaluation: nothing has allocated since.
unsafe fn into_result(caught: Caught) -> Result<SEXP, REvalError> {
    unsafe {
        match caught {
            Caught::Value(value) => Ok(value),
            Caught::Error(cond) => {
                let cond = OwnedProtect::new(cond);
                Err(REvalError::from_condition(cond.get()))
            }
            Caught::Exit => Err(REvalError::simple_error(EXIT_MESSAGE)),
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
    unsafe { into_result(try_eval_raw(expr, env)) }
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
