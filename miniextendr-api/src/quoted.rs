//! Unevaluated arguments: the [`Quoted`] and [`Quosure`] parameter markers.
//!
//! A generated R wrapper forces every argument it hands to `.Call()`, so a
//! Rust body normally sees values only. A parameter typed `Quoted` or
//! `Quosure` is passed unevaluated instead: the body receives the argument's
//! expression and the environment to evaluate it in, and decides what to do
//! with them. That is what `subset()`-style methods and APIs taking bare
//! column names (`f(data, TIME)` as well as `f(data, "TIME")`) need.
//!
//! | marker | the wrapper passes | needs | `{{ x }}` / `!!x` |
//! |---|---|---|---|
//! | [`Quoted`] | `list(substitute(x), parent.frame())` | base R | no |
//! | [`Quosure`] | `rlang::enquo(x)` | rlang in `Imports` | yes |
//!
//! ```ignore
//! use miniextendr_api::{miniextendr, Quoted, SEXP};
//!
//! /// Rows of `data` where `cond` holds; `cond` sees the columns of `data`,
//! /// then the caller's variables.
//! #[miniextendr]
//! pub fn keep_rows(data: SEXP, cond: Quoted) -> Vec<i32> {
//!     let keep = cond.eval_in(data);
//!     // ... read the logical vector
//! #   vec![]
//! }
//! ```
//!
//! ```r
//! threshold <- 3
//! keep_rows(df, value > threshold)
//! ```
//!
//! The wrapper never forces the argument: a `Quoted` / `Quosure` parameter
//! takes no R-side checks and none of the per-parameter options (`coerce`,
//! `match_arg`, `choices`, `default`, `no_na`, `inherits`, ...), which are
//! compile errors on it. A function taking one runs on R's main thread.
//! It works on free functions, including standalone S3 methods
//! (`#[miniextendr(s3(generic = "subset", class = "..."))]`): in a method
//! reached through `UseMethod()`, `parent.frame()` is the generic's caller, as
//! in base R's `subset.data.frame()`.
//!
//! A missing argument is a conversion error naming the parameter
//! (`argument "cond" is missing, with no default`), like forcing it in R. Take
//! `Missing<Quoted>` / `Missing<Quosure>` to make the argument optional.
//!
//! Evaluation goes through [`crate::expression::eval_with_handlers`]: the
//! caller's `withCallingHandlers()` / `suppressWarnings()` see what the
//! expression signals, and an error, or the exit of a `tryCatch()` handler set
//! up around the call, unwinds through the Rust frames (running their
//! destructors) and carries on in R as the original condition.
//!
//! Both markers borrow from the `.Call()` arguments, which R keeps rooted for
//! the call; the lifetime keeps them from outliving it. They are `!Send`.

use std::marker::PhantomData;

use crate::from_r::charsxp_to_str;
use crate::gc_protect::{OwnedProtect, ProtectScope};
use crate::missing::Missing;
use crate::sexp_ext::PairListExt;
use crate::{SEXP, SEXPTYPE, SexpExt};

/// The name an argument expression stands for: a symbol's name, or the value
/// of a length-1 character literal. `None` for anything else, and for `NA`.
fn expr_name<'a>(expr: SEXP) -> Option<&'a str> {
    match expr.type_of() {
        // Symbols are never collected; their print names live for the session.
        SEXPTYPE::SYMSXP => {
            let name = unsafe { charsxp_to_str(expr.printname()) };
            (!name.is_empty()).then_some(name)
        }
        SEXPTYPE::STRSXP if expr.len() == 1 => {
            let elt = expr.string_elt(0);
            // The CHARSXP is held by `expr`, which the caller keeps rooted.
            (!elt.is_na_string()).then(|| unsafe { charsxp_to_str(elt) })
        }
        _ => None,
    }
}

// region: Quoted

/// An argument passed unevaluated: its expression and the environment it was
/// written in, as base R's `substitute(x)` and `parent.frame()` give them.
///
/// Take it as a `#[miniextendr]` parameter. The generated wrapper keeps the
/// parameter as an R formal and passes
/// `if (missing(x)) quote(expr=) else list(substitute(x), parent.frame())`,
/// which never forces the argument. See the [module docs](self).
///
/// ```ignore
/// use miniextendr_api::{miniextendr, Quoted, SEXP};
///
/// #[miniextendr]
/// pub fn column_name(col: Quoted) -> Option<String> {
///     col.as_name().map(str::to_owned)   // column_name(TIME), column_name("TIME")
/// }
///
/// #[miniextendr]
/// pub fn evaluate(expr: Quoted) -> SEXP {
///     expr.eval()                          // eval(expr, env)
/// }
/// ```
///
/// `parent.frame()` is the environment of the function that called the
/// wrapper, which is where the expression was written when the user calls the
/// wrapper directly or through `UseMethod()`. A wrapper reached through a
/// function that forwards its own argument (`g <- function(y) f(y)`) receives
/// `y` and `g`'s frame, as `subset()` does. To resolve forwarding, `{{ x }}`
/// and `!!x`, take a [`Quosure`].
pub struct Quoted<'a> {
    expr: SEXP,
    env: SEXP,
    /// Borrows the `.Call()` argument that roots `expr` and `env`.
    _arg: PhantomData<&'a SEXP>,
    /// R objects stay on R's main thread.
    _not_send: PhantomData<*const ()>,
}

impl<'a> Quoted<'a> {
    /// Read the argument a generated wrapper passed for a `Quoted` parameter:
    /// `None` when the argument is missing (`quote(expr=)`). Generated code
    /// calls this; user code has no reason to.
    ///
    /// # Panics
    ///
    /// When `arg` is not the wrapper's `list(<expr>, <environment>)`.
    ///
    /// # Safety
    ///
    /// R's main thread; `arg` stays rooted for `'a`.
    #[doc(hidden)]
    pub unsafe fn from_wrapper_arg(arg: &'a SEXP) -> Option<Self> {
        let arg = *arg;
        if arg == SEXP::missing_arg() {
            return None;
        }
        assert!(
            arg.type_of() == SEXPTYPE::VECSXP
                && arg.len() == 2
                && arg.vector_elt(1).is_environment(),
            "a `Quoted` argument is passed as `list(substitute(x), parent.frame())`, got {}",
            arg.type_of().type_name(),
        );
        Some(Quoted {
            expr: arg.vector_elt(0),
            env: arg.vector_elt(1),
            _arg: PhantomData,
            _not_send: PhantomData,
        })
    }

    /// [`Self::from_wrapper_arg`] for a `Missing<Quoted>` parameter.
    ///
    /// # Safety
    ///
    /// As [`Self::from_wrapper_arg`].
    #[doc(hidden)]
    pub unsafe fn missing_from_wrapper_arg(arg: &'a SEXP) -> Missing<Self> {
        match unsafe { Self::from_wrapper_arg(arg) } {
            Some(quoted) => Missing::Present(quoted),
            None => Missing::Absent,
        }
    }

    /// The expression as written: a symbol, a call, or a constant
    /// (`substitute(x)`).
    #[inline]
    pub fn expr(&self) -> SEXP {
        self.expr
    }

    /// The environment the expression was written in (`parent.frame()`).
    #[inline]
    pub fn env(&self) -> SEXP {
        self.env
    }

    /// The name the argument stands for: `TIME` and `"TIME"` both give
    /// `"TIME"`. `None` for any other expression (a call, a number, `NA`).
    #[inline]
    pub fn as_name(&self) -> Option<&'a str> {
        expr_name(self.expr)
    }

    /// Evaluate the expression in its environment, `eval(expr, env)`, keeping
    /// the caller's condition handlers
    /// ([`eval_with_handlers`](crate::expression::eval_with_handlers)).
    ///
    /// The value is unprotected: protect it before the next allocation.
    pub fn eval(&self) -> SEXP {
        // SAFETY: a `Quoted` exists only inside the `#[miniextendr]` call that
        // received it, on R's main thread; its argument roots both SEXPs.
        unsafe { crate::expression::eval_with_handlers(self.expr, self.env) }
    }

    /// Evaluate the expression with `data` as a data mask, like
    /// `eval(expr, data, env)`: the elements of a list or the columns of a
    /// data frame are visible by name, ahead of the expression's environment.
    /// The first of two elements with one name wins, and unnamed elements are
    /// not visible. An environment is used as is (`env` is not consulted),
    /// and `NULL` evaluates in the expression's environment.
    ///
    /// The value is unprotected: protect it before the next allocation.
    ///
    /// # Panics
    ///
    /// When `data` is not a list, a data frame, an environment or `NULL`.
    pub fn eval_in(&self, data: SEXP) -> SEXP {
        // SAFETY: as in `eval`; `data` is rooted by the caller.
        unsafe {
            match data.type_of() {
                SEXPTYPE::ENVSXP => crate::expression::eval_with_handlers(self.expr, data),
                SEXPTYPE::NILSXP => crate::expression::eval_with_handlers(self.expr, self.env),
                SEXPTYPE::VECSXP => {
                    // Rooted ahead of the evaluation's own protect frame, so an
                    // R exit leaves the protect stack as this guard expects.
                    let mask = OwnedProtect::new(data_mask(data, self.env));
                    crate::expression::eval_with_handlers(self.expr, mask.get())
                }
                other => panic!(
                    "`data` must be a list, a data frame, an environment or NULL, not {}",
                    other.type_name()
                ),
            }
        }
    }
}

/// A new environment enclosed by `enclos` that binds each named element of
/// the list `data`, as `eval()` builds one for a list `envir`. Bound in
/// reverse so the first of two same-named elements wins, as in `eval()`.
///
/// # Safety
///
/// R's main thread; `data` a rooted list, `enclos` an environment. The result
/// is unprotected.
unsafe fn data_mask(data: SEXP, enclos: SEXP) -> SEXP {
    unsafe {
        let n = data.len();
        let size = i32::try_from(n).unwrap_or(i32::MAX);
        let mask = OwnedProtect::new(crate::sys::R_NewEnv(enclos, crate::Rboolean::TRUE, size));
        let names = data.get_names();
        if names.type_of() == SEXPTYPE::STRSXP {
            for i in (0..n).rev() {
                let i = isize::try_from(i).expect("list index fits in isize");
                let name = names.string_elt(i);
                if name.is_na_string() || name.len() == 0 {
                    continue;
                }
                // `names` is held by `data`; the symbol table keeps the symbol.
                let sym = crate::sys::Rf_installChar(name);
                crate::sys::Rf_defineVar(sym, data.vector_elt(i), mask.get());
            }
        }
        mask.get()
    }
}

// endregion

// region: Quosure

/// An argument captured as an rlang quosure, `rlang::enquo(x)`: its expression
/// and environment, with rlang's argument forwarding and injection resolved.
///
/// Take it as a `#[miniextendr]` parameter. The generated wrapper keeps the
/// parameter as an R formal and passes
/// `if (missing(x)) quote(expr=) else rlang::enquo(x)`, which never forces the
/// argument. Unlike [`Quoted`], a wrapper reached through
/// `function(col) f(data, {{ col }})` or called as `f(data, !!sym)` receives
/// the expression the user wrote, in the user's environment.
///
/// **The package must list rlang in `Imports`.** The macro cannot see
/// `DESCRIPTION`; without rlang the wrapper fails when it is called.
///
/// The expression and environment are read from the quosure directly (a
/// quosure is a one-sided formula, `~expr`, of class `quosure` whose
/// environment is its `.Environment` attribute), so [`Self::expr`],
/// [`Self::env`] and [`Self::as_name`] call no R code.
/// [`Self::eval_tidy`] evaluates through `rlang::eval_tidy()`, which resolves
/// nested quosures and the `.data` / `.env` pronouns. Hand [`Self::sexp`] to
/// other quosure-aware R functions (`tidyselect::eval_select()`) with
/// [`RCall::eval_with_handlers`](crate::expression::RCall::eval_with_handlers).
pub struct Quosure<'a> {
    quo: SEXP,
    expr: SEXP,
    env: SEXP,
    /// Borrows the `.Call()` argument that roots the quosure.
    _arg: PhantomData<&'a SEXP>,
    /// R objects stay on R's main thread.
    _not_send: PhantomData<*const ()>,
}

impl<'a> Quosure<'a> {
    /// Read the argument a generated wrapper passed for a `Quosure` parameter:
    /// `None` when the argument is missing (`quote(expr=)`, or a quosure of
    /// the missing argument, which `{{ x }}` gives for a missing `x`).
    /// Generated code calls this; user code has no reason to.
    ///
    /// # Panics
    ///
    /// When `arg` is not a quosure.
    ///
    /// # Safety
    ///
    /// R's main thread; `arg` stays rooted for `'a`.
    #[doc(hidden)]
    pub unsafe fn from_wrapper_arg(arg: &'a SEXP) -> Option<Self> {
        let quo = *arg;
        if quo == SEXP::missing_arg() {
            return None;
        }
        assert!(
            quo.is_language() && quo.inherits_class(c"quosure"),
            "a `Quosure` argument is passed as `rlang::enquo(x)`, got {}",
            quo.type_of().type_name(),
        );
        let expr = quo.cdr().car();
        if expr == SEXP::missing_arg() {
            return None;
        }
        let env = quo.get_attr(unsafe { crate::sys::Rf_install(c".Environment".as_ptr()) });
        Some(Quosure {
            quo,
            expr,
            env,
            _arg: PhantomData,
            _not_send: PhantomData,
        })
    }

    /// [`Self::from_wrapper_arg`] for a `Missing<Quosure>` parameter.
    ///
    /// # Safety
    ///
    /// As [`Self::from_wrapper_arg`].
    #[doc(hidden)]
    pub unsafe fn missing_from_wrapper_arg(arg: &'a SEXP) -> Missing<Self> {
        match unsafe { Self::from_wrapper_arg(arg) } {
            Some(quosure) => Missing::Present(quosure),
            None => Missing::Absent,
        }
    }

    /// The quosure itself, for R functions that take one
    /// (`rlang::eval_tidy()`, `tidyselect::eval_select()`).
    #[inline]
    pub fn sexp(&self) -> SEXP {
        self.quo
    }

    /// The quosure's expression (`rlang::quo_get_expr()`).
    #[inline]
    pub fn expr(&self) -> SEXP {
        self.expr
    }

    /// The quosure's environment (`rlang::quo_get_env()`): the empty
    /// environment for a constant.
    #[inline]
    pub fn env(&self) -> SEXP {
        self.env
    }

    /// The name the argument stands for: `TIME`, `"TIME"`, `{{ col }}` with
    /// `col = TIME` and `!!rlang::sym("TIME")` all give `"TIME"`. A quosure
    /// nested in the expression is looked through. `None` for any other
    /// expression.
    pub fn as_name(&self) -> Option<&'a str> {
        let mut expr = self.expr;
        while expr.is_language() && expr.inherits_class(c"quosure") {
            expr = expr.cdr().car();
        }
        expr_name(expr)
    }

    /// Evaluate the quosure with `data` as a data mask,
    /// `rlang::eval_tidy(quo, data)`: columns of `data` win over variables of
    /// the same name in the quosure's environment, and `.data$x` / `.env$x`
    /// pick one side. `data` may be `NULL`.
    ///
    /// Evaluated with
    /// [`eval_with_handlers`](crate::expression::eval_with_handlers), so the
    /// caller's handlers see what the expression signals.
    ///
    /// The value is unprotected: protect it before the next allocation.
    pub fn eval_tidy(&self, data: SEXP) -> SEXP {
        use crate::sys::{R_BaseEnv, Rf_install, Rf_lang2, Rf_lang3};
        // SAFETY: a `Quosure` exists only inside the `#[miniextendr]` call
        // that received it, on R's main thread; `data` is rooted by the caller.
        unsafe {
            let scope = ProtectScope::new();
            // `rlang::eval_tidy(quote(<quo>), quote(<data>))`; a missing rlang raises
            // R's own error, through the caller's handlers.
            let fun = scope.protect_raw(Rf_lang3(
                Rf_install(c"::".as_ptr()),
                Rf_install(c"rlang".as_ptr()),
                Rf_install(c"eval_tidy".as_ptr()),
            ));
            // Both arguments pass through `quote()`, so neither the quosure
            // nor a language `data` is evaluated on the way in.
            let quote = Rf_install(c"quote".as_ptr());
            let quoted = scope.protect_raw(Rf_lang2(quote, self.quo));
            let data = scope.protect_raw(Rf_lang2(quote, data));
            let call = scope.protect_raw(Rf_lang3(fun, quoted, data));
            crate::expression::eval_with_handlers(call, R_BaseEnv)
        }
    }
}

// endregion

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markers_hold_their_sexps_only() {
        assert_eq!(
            std::mem::size_of::<Quoted<'static>>(),
            2 * std::mem::size_of::<SEXP>()
        );
        assert_eq!(
            std::mem::size_of::<Quosure<'static>>(),
            3 * std::mem::size_of::<SEXP>()
        );
    }
}
