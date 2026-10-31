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
//!
//! A `Quosure` can also be read from a quosure found in an R object
//! ([`Quosure::from_sexp`]), such as one a wrapper wrote into a call with
//! `c(time = {{ t }})`, or built from an expression and an environment
//! ([`Quosure::new`], [`Quosure::args`]). It then borrows the root the caller
//! holds: the SEXP it was read from, or a [`ProtectScope`].

use std::marker::PhantomData;

use crate::expression::RCall;
use crate::from_r::charsxp_to_str_or_panic;
use crate::gc_protect::{OwnedProtect, ProtectScope};
use crate::missing::Missing;
use crate::{PairListExt, SEXP, SEXPTYPE, SexpExt};

/// The name an argument expression stands for: a symbol's name, or the value
/// of a length-1 character literal. `None` for anything else, and for `NA`.
fn expr_name<'a>(expr: SEXP) -> Option<&'a str> {
    match expr.type_of() {
        // Symbols are never collected; their print names live for the session.
        SEXPTYPE::SYMSXP => {
            let name = unsafe { charsxp_to_str_or_panic(expr.printname()) };
            (!name.is_empty()).then_some(name)
        }
        SEXPTYPE::STRSXP if expr.len() == 1 => {
            let elt = expr.string_elt(0);
            // The CHARSXP is held by `expr`, which the caller keeps rooted.
            (!elt.is_na_string()).then(|| unsafe { charsxp_to_str_or_panic(elt) })
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
        unsafe { self.eval_masked(data, crate::expression::eval_with_handlers) }
    }

    /// [`eval_in`](Self::eval_in) with an R error returned as `Err`, as
    /// [`try_eval_with_handlers`](crate::expression::try_eval_with_handlers)
    /// returns it: the caller's handlers still see the expression's warnings
    /// and messages. The mask and the rules for `data` are `eval_in`'s.
    ///
    /// A `subset()` method can explain a column that doesn't exist and leave
    /// every other error as R raised it
    /// ([`REvalError::reraise`](crate::expression::REvalError::reraise)):
    ///
    /// ```ignore
    /// let keep = match subset.try_eval_in(table) {
    ///     Ok(keep) => keep,
    ///     // `unknown_column` compares the expression's names with the table's.
    ///     Err(e) if unknown_column(&subset, table) => {
    ///         rust_error!(class = "pkg_unknown_column", "{e}; the columns are ...")
    ///     }
    ///     // SAFETY: inside the `#[miniextendr]` call.
    ///     Err(e) => unsafe { e.reraise() },
    /// };
    /// ```
    ///
    /// The value is unprotected: protect it before the next allocation.
    ///
    /// # Panics
    ///
    /// When `data` is not a list, a data frame, an environment or `NULL`.
    pub fn try_eval_in(&self, data: SEXP) -> Result<SEXP, crate::expression::REvalError> {
        // SAFETY: as in `eval`; `data` is rooted by the caller.
        unsafe { self.eval_masked(data, crate::expression::try_eval_with_handlers) }
    }

    /// Evaluate the expression with `data` as a mask, as
    /// [`eval_in`](Self::eval_in) describes, through `eval`.
    ///
    /// # Safety
    ///
    /// As `eval`'s: R's main thread, inside a miniextendr boundary; `data`
    /// rooted by the caller.
    unsafe fn eval_masked<R>(&self, data: SEXP, eval: unsafe fn(SEXP, SEXP) -> R) -> R {
        unsafe {
            match data.type_of() {
                SEXPTYPE::ENVSXP => eval(self.expr, data),
                SEXPTYPE::NILSXP => eval(self.expr, self.env),
                SEXPTYPE::VECSXP => {
                    // Rooted ahead of the evaluation's own protect frame, so an
                    // R exit leaves the protect stack as this guard expects.
                    let mask = OwnedProtect::new(data_mask(data, self.env));
                    eval(self.expr, mask.get())
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
/// R's main thread, inside a miniextendr boundary; `data` a rooted list,
/// `enclos` an environment. The result is unprotected.
unsafe fn data_mask(data: SEXP, enclos: SEXP) -> SEXP {
    unsafe {
        let n = data.len();
        let size = i32::try_from(n).unwrap_or(i32::MAX);
        let mask = OwnedProtect::new(crate::sys::R_NewEnv(enclos, crate::Rboolean::TRUE, size));
        let env = crate::expression::REnv::from_sexp(mask.get());
        let names = data.get_names();
        if names.type_of() == SEXPTYPE::STRSXP {
            for i in (0..n).rev() {
                let i = isize::try_from(i).expect("list index fits in isize");
                let name = names.string_elt(i);
                if name.is_na_string() || name.len() == 0 {
                    continue;
                }
                // `names` is held by `data`; the symbol table keeps the symbol.
                env.define_symbol(SEXP::install_char(name), data.vector_elt(i));
            }
        }
        mask.get()
    }
}

// endregion

// region: Quosure

/// Whether `x` is an rlang quosure: a one-sided formula, `~expr`, of class
/// `quosure`.
fn is_quosure(x: SEXP) -> bool {
    x.is_language() && x.inherits_class(c"quosure")
}

/// The quosure `x` stands for: `x` itself, or, when its expression is a
/// quosure too (`rlang::new_quosure(<quosure>)`), the innermost one. `None`
/// when `x` is not a quosure.
fn innermost_quosure(x: SEXP) -> Option<SEXP> {
    if !is_quosure(x) {
        return None;
    }
    let mut quo = x;
    while is_quosure(quo.cdr().car()) {
        quo = quo.cdr().car();
    }
    Some(quo)
}

/// The function `rlang::new_quosure`, unprotected.
///
/// # Panics
///
/// When rlang cannot be loaded.
///
/// # Safety
///
/// R's main thread.
unsafe fn rlang_new_quosure() -> SEXP {
    use crate::sys::{R_BaseEnv, Rf_install};
    unsafe {
        // `rlang::new_quosure`; `::` takes both names unevaluated.
        RCall::new("::")
            .arg(Rf_install(c"rlang".as_ptr()))
            .arg(Rf_install(c"new_quosure".as_ptr()))
            .eval(R_BaseEnv)
            .unwrap_or_else(|e| {
                panic!(
                    "a `Quosure` of a bare expression is built with `rlang::new_quosure()`, \
                     which needs the rlang package: {e}"
                )
            })
    }
}

/// `new_quosure(quote(<expr>), env)`, unprotected.
///
/// # Safety
///
/// R's main thread; `new_quosure` (from [`rlang_new_quosure`]), `expr` and
/// `env` rooted, `env` an environment.
unsafe fn wrap_in_quosure(new_quosure: SEXP, expr: SEXP, env: SEXP) -> SEXP {
    unsafe {
        RCall::from_sexp(new_quosure)
            .quoted_arg(expr)
            .arg(env)
            .eval(crate::sys::R_BaseEnv)
            .unwrap_or_else(|e| panic!("`rlang::new_quosure()` failed: {e}"))
    }
}

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
///
/// An argument holding several selections, a role map such as
/// `cols = c(time = TAD, dose = NA)`, reads entry by entry with
/// [`Self::args`]: each argument of the call as a quosure of its own. A
/// quosure found in an R object reads with [`Self::from_sexp`], and
/// [`Self::new`] makes one from an expression and an environment.
pub struct Quosure<'a> {
    quo: SEXP,
    expr: SEXP,
    env: SEXP,
    /// Borrows the root that keeps the quosure alive: the `.Call()` argument,
    /// the SEXP it was read from, or a [`ProtectScope`].
    _arg: PhantomData<&'a SEXP>,
    /// R objects stay on R's main thread.
    _not_send: PhantomData<*const ()>,
}

impl<'a> Quosure<'a> {
    /// Read a quosure's expression and environment.
    ///
    /// # Safety
    ///
    /// R's main thread; `quo` is a quosure, rooted for `'a`.
    unsafe fn read(quo: SEXP) -> Self {
        let expr = quo.cdr().car();
        let env = quo.get_attr(unsafe { crate::sys::Rf_install(c".Environment".as_ptr()) });
        Quosure {
            quo,
            expr,
            env,
            _arg: PhantomData,
            _not_send: PhantomData,
        }
    }

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
            is_quosure(quo),
            "a `Quosure` argument is passed as `rlang::enquo(x)`, got {}",
            quo.type_of().type_name(),
        );
        let quosure = unsafe { Self::read(quo) };
        (quosure.expr != SEXP::missing_arg()).then_some(quosure)
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

    /// Read a quosure the caller holds: `None` when `sexp` is not one.
    ///
    /// A quosure whose expression is a quosure too, which
    /// `rlang::new_quosure(<quosure>)` makes, is looked through: the result is
    /// the innermost one, which is what the outer one evaluates to and what
    /// `tidyselect::eval_select()` accepts (it refuses the outer one). A
    /// quosure of the missing argument is read as it is: [`Self::expr`] is
    /// [`SEXP::missing_arg()`].
    ///
    /// ```ignore
    /// // `call` is the expression of a `Quosure` argument,
    /// // `c(time = {{ t }})` written by a forwarding wrapper.
    /// for (name, arg) in call.args() {
    ///     if let Some(quosure) = unsafe { Quosure::from_sexp(&arg) } {
    ///         // the quosure `{{ t }}` left in the call, as it is
    ///     }
    /// }
    /// ```
    ///
    /// # Safety
    ///
    /// R's main thread, inside a miniextendr boundary (a `#[miniextendr]`
    /// body, a `with_r_unwind_protect` closure; [`Self::eval_tidy`] needs
    /// one); `sexp` stays rooted for `'a`.
    pub unsafe fn from_sexp(sexp: &'a SEXP) -> Option<Self> {
        innermost_quosure(*sexp).map(|quo| unsafe { Self::read(quo) })
    }

    /// A quosure of `expr` in `env`, rooted by `scope`.
    ///
    /// A bare expression is wrapped with `rlang::new_quosure(expr, env)`. A
    /// quosure is returned as it is, through [`Self::from_sexp`] (`env` is
    /// not used): wrapping a quosure again makes a formula that
    /// `tidyselect::eval_select()` refuses. So an argument read from a call
    /// can go through `new()` whether or not a wrapper wrote a quosure there.
    /// The missing argument (an empty argument, `c(a = )`) is wrapped too:
    /// [`Self::expr`] is then [`SEXP::missing_arg()`].
    ///
    /// Each call protects one value in `scope`.
    ///
    /// # Panics
    ///
    /// When `env` is not an environment, or when `expr` is not a quosure and
    /// rlang cannot be loaded (a package taking a [`Quosure`] lists rlang in
    /// `Imports`).
    ///
    /// # Safety
    ///
    /// As [`Self::from_sexp`]; `expr` and `env` are rooted by the caller for
    /// the call.
    pub unsafe fn new(expr: SEXP, env: SEXP, scope: &'a ProtectScope) -> Self {
        assert!(
            env.is_environment(),
            "`Quosure::new()` takes an environment as `env`, not {}",
            env.type_of().type_name(),
        );
        let quo = match innermost_quosure(expr) {
            Some(quo) => quo,
            // The function's guard drops before `scope` protects the quosure,
            // so the protect stack unwinds in order; nothing allocates between.
            None => unsafe {
                let new_quosure = OwnedProtect::new(rlang_new_quosure());
                wrap_in_quosure(new_quosure.get(), expr, env)
            },
        };
        unsafe { Self::read(scope.protect_raw(quo)) }
    }

    /// The arguments of the quosure's call, each as a quosure with its name,
    /// rooted by `scope`; `None` when the expression is not a call.
    ///
    /// `c(time = TAD, dose = NA)` gives `(Some("time"), TAD)` and
    /// `(Some("dose"), NA)`, each in this quosure's environment. An argument
    /// a wrapper wrote as a quosure (`c(time = {{ t }})`) comes back as it is,
    /// in its own environment, as [`Self::new`] returns it; any other is
    /// wrapped in this quosure's environment with `rlang::new_quosure()`. The
    /// name is `None` for an unnamed argument; the function (`c`) is not one
    /// of the arguments.
    ///
    /// ```ignore
    /// #[miniextendr]
    /// pub fn roles(data: SEXP, cols: Quosure) -> Vec<String> {
    ///     let scope = unsafe { ProtectScope::new() };
    ///     let Some(args) = cols.args(&scope) else {
    ///         panic!("`cols` must be a call, `c(role = column, ...)`");
    ///     };
    ///     for (role, quosure) in args {
    ///         // `quosure.expr()` is `NA` / `NULL` as written; hand the others
    ///         // to `tidyselect::eval_select()`.
    ///     }
    /// #   vec![]
    /// }
    /// ```
    ///
    /// All the quosures are held in one list, which `scope` protects.
    ///
    /// # Panics
    ///
    /// As [`Self::new`]: when an argument is not a quosure and rlang cannot
    /// be loaded.
    pub fn args<'s>(
        &'s self,
        scope: &'s ProtectScope,
    ) -> Option<Vec<(Option<&'s str>, Quosure<'s>)>> {
        if !self.expr.is_language() {
            return None;
        }
        let args: Vec<(Option<&'s str>, SEXP)> = self.expr.args().collect();
        // SAFETY: a `Quosure` exists only on R's main thread, rooted for
        // `'a`: so are its call and the arguments in it. Each quosure is
        // stored in `held` as it is made, and `held` is protected by `scope`
        // before the function's guard, which drops first.
        unsafe {
            let held = scope.alloc_vecsxp(args.len()).into_raw();
            let mut new_quosure: Option<OwnedProtect> = None;
            for (i, &(_, arg)) in args.iter().enumerate() {
                let quo = match innermost_quosure(arg) {
                    Some(quo) => quo,
                    None => {
                        let new_quosure = new_quosure
                            .get_or_insert_with(|| OwnedProtect::new(rlang_new_quosure()));
                        wrap_in_quosure(new_quosure.get(), arg, self.env)
                    }
                };
                held.set_vector_elt(r_index(i), quo);
            }
            drop(new_quosure);
            Some(
                args.into_iter()
                    .enumerate()
                    .map(|(i, (name, _))| (name, Quosure::read(held.vector_elt(r_index(i)))))
                    .collect(),
            )
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
        while is_quosure(expr) {
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
        // SAFETY: a `Quosure` exists only inside a miniextendr boundary on
        // R's main thread (the `#[miniextendr]` call that received it, or
        // the caller of `from_sexp` / `new`); `data` is rooted by the caller.
        unsafe { self.eval_tidy_with(data, crate::expression::eval_with_handlers) }
    }

    /// [`eval_tidy`](Self::eval_tidy) with an R error returned as `Err`, as
    /// [`try_eval_with_handlers`](crate::expression::try_eval_with_handlers)
    /// returns it: the caller's handlers still see the quosure's warnings and
    /// messages. A missing rlang is an `Err` too.
    ///
    /// The value is unprotected: protect it before the next allocation.
    pub fn try_eval_tidy(&self, data: SEXP) -> Result<SEXP, crate::expression::REvalError> {
        // SAFETY: as in `eval_tidy`.
        unsafe { self.eval_tidy_with(data, crate::expression::try_eval_with_handlers) }
    }

    /// `rlang::eval_tidy(quo, data)` evaluated by `eval`.
    ///
    /// # Safety
    ///
    /// As `eval_tidy`'s: R's main thread, inside a miniextendr boundary;
    /// `data` rooted by the caller.
    unsafe fn eval_tidy_with<R>(&self, data: SEXP, eval: unsafe fn(SEXP, SEXP) -> R) -> R {
        use crate::sys::{R_BaseEnv, Rf_install, Rf_lang2, Rf_lang3};
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
            eval(call, R_BaseEnv)
        }
    }
}

/// An R vector index.
fn r_index(i: usize) -> isize {
    isize::try_from(i).expect("an R vector index fits in isize")
}

// endregion

#[cfg(test)]
mod tests;
