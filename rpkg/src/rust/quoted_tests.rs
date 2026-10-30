//! Fixtures for arguments passed unevaluated (`Quoted`, `Quosure`) and for
//! evaluating R code from Rust in the caller's R context
//! (`eval_with_handlers`, `RCall::eval_with_handlers`) (#1835), with an R
//! error returned as `Err` (`try_eval_with_handlers`,
//! `RCall::try_eval_with_handlers`) (#1893). Pairlist arguments
//! (`PairListExt::args`) and quosures read from or built for a call
//! (`Quosure::from_sexp`, `Quosure::new`, `Quosure::args`) (#1903).
//!
//! Each evaluating fixture holds a [`DropSentinel`] across the evaluation, so
//! the tests can tell that an R exit (an error, a `tryCatch()` handler) ran
//! the Rust destructors on its way out: `quoted_sentinel_drops()` counts them.

use std::sync::atomic::{AtomicI32, Ordering};

use miniextendr_api::expression::{RCall, try_eval_with_handlers};
use miniextendr_api::from_r::TryFromSexp;
use miniextendr_api::gc_protect::ProtectScope;
use miniextendr_api::prelude::{Call, List, OwnedProtect, PairListExt, SEXP, SexpExt};
use miniextendr_api::{
    Missing, NamedListBuilder, Quosure, Quoted, SEXPTYPE, miniextendr, rust_error,
};

use crate::expression_tests::caught_parts;

// region: drop sentinel

static SENTINEL_DROPS: AtomicI32 = AtomicI32::new(0);

/// Counts its drops in [`SENTINEL_DROPS`].
struct DropSentinel;

impl Drop for DropSentinel {
    fn drop(&mut self) {
        SENTINEL_DROPS.fetch_add(1, Ordering::SeqCst);
    }
}

/// How many fixture sentinels have been dropped so far in this session.
#[miniextendr(noexport)]
pub fn quoted_sentinel_drops() -> i32 {
    SENTINEL_DROPS.load(Ordering::SeqCst)
}

// endregion

// region: Quoted

/// Evaluate `expr` in the caller's frame.
#[miniextendr(noexport)]
pub fn quoted_eval(expr: Quoted) -> SEXP {
    let _sentinel = DropSentinel;
    expr.eval()
}

/// The environment `expr` was captured in.
#[miniextendr(noexport)]
pub fn quoted_env(expr: Quoted) -> SEXP {
    expr.env()
}

/// The expression as written.
#[miniextendr(noexport)]
pub fn quoted_expr(expr: Quoted) -> SEXP {
    expr.expr()
}

/// `expr` as a name: a symbol or a string, else `NA`.
#[miniextendr(noexport)]
pub fn quoted_name(expr: Quoted) -> Option<String> {
    expr.as_name().map(str::to_owned)
}

/// `"absent"` when `expr` was omitted, else `typeof()` of the expression.
#[miniextendr(noexport)]
pub fn quoted_optional(expr: Missing<Quoted>) -> String {
    match expr {
        Missing::Absent => "absent".to_owned(),
        Missing::Present(expr) => expr.expr().type_of().type_name().to_owned(),
    }
}

/// `"absent"` when `x` was omitted, else `typeof()` of the expression. The
/// `NULL` default only shows in the usage; the wrapper never evaluates it.
///
/// @param x An expression, passed unevaluated. Omitted by default.
#[miniextendr(internal)]
pub fn quoted_default(#[miniextendr(default = "NULL")] x: Missing<Quoted>) -> String {
    match x {
        Missing::Absent => "absent".to_owned(),
        Missing::Present(x) => x.expr().type_of().type_name().to_owned(),
    }
}

/// `"absent"` when `x` was omitted, though its default would raise an error
/// if evaluated; else `typeof()` of the expression.
#[miniextendr(noexport)]
pub fn quoted_default_unevaluated(
    #[miniextendr(default = "stop(\"the default was evaluated\")")] x: Missing<Quoted>,
) -> String {
    match x {
        Missing::Absent => "absent".to_owned(),
        Missing::Present(x) => x.expr().type_of().type_name().to_owned(),
    }
}

/// 1-based rows of `data` where `cond`, evaluated with `data`'s columns in
/// scope, is `TRUE` (`NA` counts as `FALSE`).
#[miniextendr(noexport)]
pub fn quoted_rows(data: SEXP, cond: Quoted) -> Vec<i32> {
    let _sentinel = DropSentinel;
    // SAFETY: the value is rooted before the next allocation.
    let keep = unsafe { OwnedProtect::new(cond.eval_in(data)) };
    true_rows(keep.get())
}

/// Keep the rows of an `mx_quoted_tbl` where `subset` is `TRUE`, evaluated
/// with the table's columns in scope and the caller's variables behind them.
///
/// @param x An `mx_quoted_tbl`: a data frame with that class in front.
/// @param subset A logical expression over the columns, unevaluated.
/// @param ... Ignored.
/// @export
#[miniextendr(s3(generic = "subset", class = "mx_quoted_tbl"))]
pub fn mx_quoted_tbl_subset(x: SEXP, subset: Quoted, _dots: ...) -> SEXP {
    let _sentinel = DropSentinel;
    // SAFETY: R's main thread; every intermediate value is rooted before the
    // next allocation (`RCall` roots its arguments).
    unsafe {
        let keep = OwnedProtect::new(subset.eval_in(x));
        let rows = OwnedProtect::new(miniextendr_api::IntoR::into_sexp(true_rows(keep.get())));
        // x[rows, , drop = FALSE]
        RCall::new("[")
            .quoted_arg(x)
            .arg(rows.get())
            .arg(SEXP::missing_arg())
            .named_arg("drop", SEXP::scalar_logical(false))
            .eval_with_handlers(miniextendr_api::sys::R_BaseEnv)
    }
}

/// An R vector index.
fn idx(i: usize) -> isize {
    isize::try_from(i).expect("an R vector index fits in isize")
}

/// The 1-based positions of the `TRUE` elements of a logical vector.
fn true_rows(keep: SEXP) -> Vec<i32> {
    assert!(
        keep.type_of() == SEXPTYPE::LGLSXP,
        "the condition must evaluate to a logical vector, not {}",
        keep.type_of().type_name()
    );
    (0..keep.len())
        .filter(|&i| keep.logical_elt(idx(i)) == 1)
        .map(|i| i32::try_from(i + 1).expect("row number fits an R integer"))
        .collect()
}

// endregion

// region: Quosure

/// `rlang::eval_tidy(x, data)`.
#[miniextendr(noexport)]
pub fn quosure_eval_tidy(data: SEXP, x: Quosure) -> SEXP {
    let _sentinel = DropSentinel;
    x.eval_tidy(data)
}

/// `rlang::eval_tidy(x, data)` through `Quosure::try_eval_tidy` (#2163): the
/// value or the caught error's parts, as `quoted_try_call()`.
#[miniextendr(noexport)]
pub fn quosure_try_eval_tidy(data: SEXP, x: Quosure) -> SEXP {
    let _sentinel = DropSentinel;
    caught_parts(x.try_eval_tidy(data))
}

/// The quosure's expression.
#[miniextendr(noexport)]
pub fn quosure_expr(x: Quosure) -> SEXP {
    x.expr()
}

/// The quosure's environment.
#[miniextendr(noexport)]
pub fn quosure_env(x: Quosure) -> SEXP {
    x.env()
}

/// The quosure itself.
#[miniextendr(noexport)]
pub fn quosure_sexp(x: Quosure) -> SEXP {
    x.sexp()
}

/// `x` as a name: a symbol or a string, else `NA`.
#[miniextendr(noexport)]
pub fn quosure_name(x: Quosure) -> Option<String> {
    x.as_name().map(str::to_owned)
}

/// `"absent"` when `x` was omitted, else `typeof()` of the expression.
#[miniextendr(noexport)]
pub fn quosure_optional(x: Missing<Quosure>) -> String {
    match x {
        Missing::Absent => "absent".to_owned(),
        Missing::Present(x) => x.expr().type_of().type_name().to_owned(),
    }
}

/// `"absent"` when `x` was omitted, else `typeof()` of the quosure's
/// expression. The `NULL` default only shows in the usage; the wrapper never
/// evaluates it.
///
/// @param x An expression, captured as a quosure. Omitted by default.
#[miniextendr(internal)]
pub fn quosure_default(#[miniextendr(default = "NULL")] x: Missing<Quosure>) -> String {
    match x {
        Missing::Absent => "absent".to_owned(),
        Missing::Present(x) => x.expr().type_of().type_name().to_owned(),
    }
}

/// `tidyselect::eval_select(cols, data, allow_rename = FALSE,
/// allow_empty = TRUE, error_call = <this call>)`: the named positions of
/// the selected columns.
#[miniextendr(noexport)]
pub fn quosure_select(data: SEXP, cols: Quosure, call: Call) -> SEXP {
    let _sentinel = DropSentinel;
    // SAFETY: R's main thread; `RCall` roots every argument.
    unsafe {
        RCall::namespaced("tidyselect", "eval_select")
            .unwrap_or_else(|e| panic!("{e}"))
            .quoted_arg(cols.sexp())
            .quoted_arg(data)
            .named_arg("allow_rename", SEXP::scalar_logical(false))
            .named_arg("allow_empty", SEXP::scalar_logical(true))
            .named_quoted_arg("error_call", call.sexp())
            .eval_with_handlers(miniextendr_api::sys::R_BaseEnv)
    }
}

// endregion

// region: calls built in Rust

/// Call `pkg::fun(<args>)` through `RCall::eval_with_handlers`. `args` is a
/// list; its named elements become named arguments. Every argument is passed
/// as is (`quote()`d), so a call in `args` reaches `fun` as a call.
#[miniextendr(noexport)]
pub fn quoted_call_ns(pkg: &str, fun: &str, args: SEXP) -> SEXP {
    let _sentinel = DropSentinel;
    assert!(args.type_of() == SEXPTYPE::VECSXP, "`args` must be a list");
    // SAFETY: R's main thread; `args` is the rooted `.Call()` argument and
    // `RCall` roots every argument it is given.
    unsafe {
        let names = args.get_names();
        let mut call = RCall::namespaced(pkg, fun).unwrap_or_else(|e| panic!("{e}"));
        for i in 0..args.len() {
            let value = args.vector_elt(idx(i));
            let name = if names.is_nil() {
                None
            } else {
                names.string_elt_str(idx(i)).filter(|n| !n.is_empty())
            };
            call = match name {
                Some(name) => call.named_quoted_arg(name, value),
                None => call.quoted_arg(value),
            };
        }
        call.eval_with_handlers(miniextendr_api::sys::R_BaseEnv)
    }
}

// endregion

// region: an R error returned as Err, the caller's handlers kept (#1893)

/// Call `f()` through `RCall::try_eval_with_handlers`: `list(value = <value>)`,
/// or the caught `REvalError`'s parts,
/// `list(message, classes, specific_classes, call, condition)`.
#[miniextendr(noexport)]
pub fn quoted_try_call(f: SEXP) -> SEXP {
    let _sentinel = DropSentinel;
    // SAFETY: R's main thread, inside the `.Call()`; `f` is the rooted
    // `.Call()` argument.
    caught_parts(unsafe {
        RCall::from_sexp(f).try_eval_with_handlers(miniextendr_api::sys::R_BaseEnv)
    })
}

/// Call `f()` through `RCall::try_eval_with_handlers` and, when it raises an
/// R error, raise the package's own `mx_reraised` error with R's message, the
/// caught classes kept after it (`REvalError::reraise_class`).
#[miniextendr(noexport)]
pub fn quoted_try_call_reraise(f: SEXP) -> SEXP {
    let _sentinel = DropSentinel;
    // SAFETY: as `quoted_try_call`.
    match unsafe { RCall::from_sexp(f).try_eval_with_handlers(miniextendr_api::sys::R_BaseEnv) } {
        Ok(value) => value,
        Err(e) => rust_error!(class = e.reraise_class("mx_reraised"), "{e}"),
    }
}

/// Evaluate `expr` in the caller's frame through `try_eval_with_handlers`:
/// the value or the caught error's parts, as `quoted_try_call()`.
#[miniextendr(noexport)]
pub fn quoted_try_eval(expr: Quoted) -> SEXP {
    let _sentinel = DropSentinel;
    // SAFETY: R's main thread, inside the `.Call()`; the expression and its
    // environment are rooted by the `.Call()` argument.
    caught_parts(unsafe { try_eval_with_handlers(expr.expr(), expr.env()) })
}

/// `cond` with `data` as a data mask through `Quoted::try_eval_in` (#2163):
/// the value or the caught error's parts, as `quoted_try_call()`.
#[miniextendr(noexport)]
pub fn quoted_try_rows(data: SEXP, cond: Quoted) -> SEXP {
    let _sentinel = DropSentinel;
    caught_parts(cond.try_eval_in(data))
}

/// Call `f()` through `RCall::try_eval_with_handlers` and raise a caught
/// error again as R raised it (`REvalError::reraise`, #2163).
#[miniextendr(noexport)]
pub fn quoted_try_call_reraise_as_is(f: SEXP) -> SEXP {
    let _sentinel = DropSentinel;
    // SAFETY: as `quoted_try_call`.
    match unsafe { RCall::from_sexp(f).try_eval_with_handlers(miniextendr_api::sys::R_BaseEnv) } {
        Ok(value) => value,
        // SAFETY: inside the `.Call()`.
        Err(e) => unsafe { e.reraise() },
    }
}

/// Evaluate `expr` in the caller's frame through `try_eval_with_handlers`
/// and raise a caught error again as R raised it (`REvalError::reraise`,
/// #2163).
#[miniextendr(noexport)]
pub fn quoted_try_eval_reraise(expr: Quoted) -> SEXP {
    let _sentinel = DropSentinel;
    // SAFETY: as `quoted_try_eval`.
    match unsafe { try_eval_with_handlers(expr.expr(), expr.env()) } {
        Ok(value) => value,
        // SAFETY: inside the `.Call()`.
        Err(e) => unsafe { e.reraise() },
    }
}

/// The rows of `x` where `subset` is `TRUE`, as `quoted_rows()`, with an
/// error explained when `subset` names a column `x` doesn't have: class
/// `mx_unknown_column`. Any other error is raised again as R raised it
/// (`Quoted::try_eval_in`, `REvalError::reraise`, #2163).
#[miniextendr(noexport)]
pub fn quoted_try_subset(x: SEXP, subset: Quoted) -> Vec<i32> {
    let _sentinel = DropSentinel;
    let keep = match subset.try_eval_in(x) {
        Ok(keep) => keep,
        Err(e) => {
            let unknown = unknown_columns(&subset, x);
            if unknown.is_empty() {
                // SAFETY: inside the `.Call()`.
                unsafe { e.reraise() }
            }
            rust_error!(
                class = "mx_unknown_column",
                "no column `{}` in the table",
                unknown.join("`, `")
            )
        }
    };
    // SAFETY: the value is rooted before the next allocation.
    let keep = unsafe { OwnedProtect::new(keep) };
    true_rows(keep.get())
}

/// The names `subset` uses that are neither columns of `x` nor visible from
/// where it was written: `all.vars()` less `names(x)` and what `exists()`
/// finds.
fn unknown_columns(subset: &Quoted, x: SEXP) -> Vec<String> {
    // SAFETY: R's main thread, inside the `.Call()`; `RCall` roots its
    // arguments, and each result is read before the next allocation.
    unsafe {
        let vars = RCall::new("all.vars")
            .quoted_arg(subset.expr())
            .eval_base()
            .expect("all.vars() of an expression");
        let vars: Vec<String> = TryFromSexp::try_from_sexp(vars).expect("all.vars() gives strings");
        let columns: Vec<String> = TryFromSexp::try_from_sexp(x.get_names()).unwrap_or_default();
        vars.into_iter()
            .filter(|var| !columns.contains(var))
            .filter(|var| {
                let found = RCall::new("exists")
                    .arg(SEXP::scalar_string_from_str(var))
                    .named_arg("envir", subset.env())
                    .eval_base()
                    .expect("exists() of a name");
                !found.as_logical().unwrap_or(false)
            })
            .collect()
    }
}

// endregion

// region: an R exit caught with catch_unwind

thread_local! {
    /// A caught unwind, kept across calls by `quoted_eval_stash()`.
    static STASHED: std::cell::RefCell<Option<Box<dyn std::any::Any + Send>>> =
        const { std::cell::RefCell::new(None) };
}

/// Evaluate `expr`, catching the unwind an R exit starts and dropping it:
/// `TRUE` when there was one. R carries on from the evaluation, as after
/// `R_tryEval`; the caller's handler never runs.
#[miniextendr(noexport)]
pub fn quoted_eval_abandoned(expr: Quoted) -> bool {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _sentinel = DropSentinel;
        expr.eval();
    }))
    .is_err()
}

/// Evaluate `expr`, catching the unwind an R exit starts and dropping it on
/// another thread: `TRUE` when there was one. The next evaluation releases
/// what the exit held, on R's main thread.
#[miniextendr(noexport)]
pub fn quoted_eval_abandoned_elsewhere(expr: Quoted) -> bool {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        expr.eval();
    })) {
        Ok(()) => false,
        Err(payload) => {
            std::thread::spawn(move || drop(payload))
                .join()
                .expect("the dropping thread does not panic");
            true
        }
    }
}

/// Evaluate `expr`, catching the unwind an R exit starts and keeping it for
/// `quoted_resume_stashed()`: `TRUE` when there was one.
#[miniextendr(noexport)]
pub fn quoted_eval_stash(expr: Quoted) -> bool {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        expr.eval();
    })) {
        Ok(()) => false,
        Err(payload) => {
            STASHED.with(|stashed| *stashed.borrow_mut() = Some(payload));
            true
        }
    }
}

/// Resume the unwind `quoted_eval_stash()` kept, in this later call. Its R
/// exit cannot continue (its target is gone), so the boundary raises an error.
#[miniextendr(noexport)]
pub fn quoted_resume_stashed() -> bool {
    match STASHED.with(|stashed| stashed.borrow_mut().take()) {
        Some(payload) => std::panic::resume_unwind(payload),
        None => false,
    }
}

// endregion

// region: pairlists and quosures in a call (#1903)

/// The arguments of `x`, a call, a pairlist or `NULL`, as a list named by
/// their tags (`""` for an unnamed one).
#[miniextendr(noexport)]
pub fn pairlist_args(x: SEXP) -> NamedListBuilder {
    let mut out = NamedListBuilder::new();
    for (name, value) in x.args() {
        out.push_sexp(name.unwrap_or_default(), value);
    }
    out
}

/// A copy of `call` whose first argument is `value`: `set_car` on a node of
/// a `shallow_duplicate()`, which leaves `call` as it was.
#[miniextendr(noexport)]
pub fn pairlist_replace_first_arg(call: SEXP, value: SEXP) -> SEXP {
    assert!(call.is_language(), "`call` must be a call");
    // SAFETY: R's main thread; the copy is rooted until it is returned.
    unsafe {
        let scope = ProtectScope::new();
        let copy = scope.protect_raw(call.shallow_duplicate());
        let first = copy.cdr();
        assert!(!first.is_nil(), "`call` has no argument");
        first.set_car(value);
        copy
    }
}

/// The quosure `x` holds (the innermost of nested ones), or `NULL` when `x`
/// is not a quosure.
#[miniextendr(noexport)]
pub fn quosure_from_sexp(x: SEXP) -> SEXP {
    // SAFETY: inside the `.Call()`, which roots `x`.
    unsafe { Quosure::from_sexp(&x) }.map_or(SEXP::nil(), |quosure| quosure.sexp())
}

/// `Quosure::new(expr, env)`: a quosure of a bare `expr`, or `expr` itself
/// when it is a quosure.
#[miniextendr(noexport)]
pub fn quosure_new(expr: SEXP, env: SEXP) -> SEXP {
    // SAFETY: inside the `.Call()`, which roots `expr` and `env`; the result
    // is returned before anything else allocates.
    unsafe {
        let scope = ProtectScope::new();
        Quosure::new(expr, env, &scope).sexp()
    }
}

/// The arguments of the call `x` captures, each as a quosure, in a list
/// named by their names (`""` for an unnamed one); `NULL` when `x` is not a
/// call.
#[miniextendr(noexport)]
pub fn quosure_args(x: Quosure) -> SEXP {
    // SAFETY: inside the `.Call()`.
    let scope = unsafe { ProtectScope::new() };
    match x.args(&scope) {
        Some(args) => {
            let mut out = NamedListBuilder::with_capacity(args.len());
            for (name, quosure) in args {
                out.push_sexp(name.unwrap_or_default(), quosure.sexp());
            }
            miniextendr_api::IntoR::into_sexp(out)
        }
        None => SEXP::nil(),
    }
}

/// A role map read entry by entry: `cols` is `c(role = <selection>, ...)`.
/// An `NA` or `NULL` entry is kept as written (tidyselect refuses both);
/// every other one is selected in `data` with `tidyselect::eval_select()`,
/// its named positions. An omitted `cols` gives an empty list.
#[miniextendr(noexport)]
pub fn quosure_roles(data: SEXP, cols: Missing<Quosure>, call: Call) -> NamedListBuilder {
    let mut out = NamedListBuilder::new();
    let Missing::Present(cols) = cols else {
        return out;
    };
    // SAFETY: inside the `.Call()`.
    let scope = unsafe { ProtectScope::new() };
    let roles = cols
        .args(&scope)
        .unwrap_or_else(|| panic!("`cols` must be a call, `c(role = column, ...)`"));
    for (role, quosure) in roles {
        let expr = quosure.expr();
        let kept = expr.is_nil()
            || (expr.type_of() == SEXPTYPE::LGLSXP
                && expr.len() == 1
                && expr.as_logical().is_none());
        let value = if kept {
            expr
        } else {
            // SAFETY: R's main thread; `RCall` roots every argument, and the
            // builder roots the selection as it is pushed.
            unsafe {
                RCall::namespaced("tidyselect", "eval_select")
                    .unwrap_or_else(|e| panic!("{e}"))
                    .quoted_arg(quosure.sexp())
                    .quoted_arg(data)
                    .named_arg("allow_rename", SEXP::scalar_logical(false))
                    .named_quoted_arg("error_call", call.sexp())
                    .eval_with_handlers(miniextendr_api::sys::R_BaseEnv)
            }
        };
        out.push_sexp(role.unwrap_or_default(), value);
    }
    out
}

// endregion

// region: gc stress

/// Drive `Quoted` and `Quosure` through their conversions, evaluators and
/// accessors with values built here, under whatever GC pressure the caller
/// set (#430).
#[miniextendr(noexport)]
pub fn gc_stress_quoted() -> i32 {
    // SAFETY: R's main thread; every value is rooted across the allocations
    // that follow it.
    unsafe {
        use miniextendr_api::sys::{R_GlobalEnv, Rf_install, Rf_lang3};

        // The wrapper's `list(substitute(x), parent.frame())` for `a + b`.
        let expr = OwnedProtect::new(Rf_lang3(
            Rf_install(c"+".as_ptr()),
            Rf_install(c"a".as_ptr()),
            Rf_install(c"b".as_ptr()),
        ));
        let arg = OwnedProtect::new(List::from_raw_values(vec![expr.get(), R_GlobalEnv]).as_sexp());
        let arg_sexp = arg.get();
        let quoted = Quoted::from_wrapper_arg(&arg_sexp).expect("an argument was passed");
        assert!(quoted.as_name().is_none());

        // A data mask with both columns, from a named list built here.
        let a = OwnedProtect::new(miniextendr_api::IntoR::into_sexp(vec![1i32, 2, 3]));
        let b = OwnedProtect::new(miniextendr_api::IntoR::into_sexp(vec![10i32, 20, 30]));
        let data =
            OwnedProtect::new(List::from_raw_pairs(vec![("a", a.get()), ("b", b.get())]).as_sexp());
        let sum = OwnedProtect::new(quoted.eval_in(data.get()));
        let total: i32 = (0..sum.get().len())
            .map(|i| sum.get().integer_elt(idx(i)))
            .sum();
        assert_eq!(total, 66);

        // `Missing<Quoted>` on the missing-argument sentinel.
        let missing = SEXP::missing_arg();
        assert!(Quoted::missing_from_wrapper_arg(&missing).is_missing());

        // A string as a name.
        let name = OwnedProtect::new(SEXP::scalar_string_from_str("col"));
        let named =
            OwnedProtect::new(List::from_raw_values(vec![name.get(), R_GlobalEnv]).as_sexp());
        let named_sexp = named.get();
        let named = Quoted::from_wrapper_arg(&named_sexp).expect("an argument was passed");
        assert_eq!(named.as_name(), Some("col"));

        total
    }
}

/// Drive `Quosure::new`, `Quosure::from_sexp` and `Quosure::args` with a call
/// built here, `c(time = <quosure>, dose = NA, wt = WT, NULL)`, under
/// whatever GC pressure the caller set (#430): the number of arguments read.
/// Needs rlang.
#[miniextendr(noexport)]
pub fn gc_stress_quosure_args() -> i32 {
    // SAFETY: R's main thread; `scope` roots every quosure and the call, and
    // `RCall` roots each argument it is given.
    unsafe {
        use miniextendr_api::sys::{R_GlobalEnv, Rf_install};

        let scope = ProtectScope::new();
        let tad = Quosure::new(Rf_install(c"TAD".as_ptr()), R_GlobalEnv, &scope);
        let call = scope.protect_raw(
            RCall::new("c")
                .named_arg("time", tad.sexp())
                .named_value("dose", None::<bool>)
                .named_arg("wt", Rf_install(c"WT".as_ptr()))
                .arg(SEXP::nil())
                .build(),
        );
        let roles = Quosure::new(call, R_GlobalEnv, &scope);
        let roles_sexp = roles.sexp();
        let roles = Quosure::from_sexp(&roles_sexp).expect("a quosure");
        let args = roles.args(&scope).expect("a call");
        let names: Vec<_> = args.iter().map(|(name, _)| *name).collect();
        assert_eq!(names, [Some("time"), Some("dose"), Some("wt"), None]);
        assert_eq!(args[0].1.sexp(), tad.sexp());
        assert_eq!(args[1].1.expr().type_of(), SEXPTYPE::LGLSXP);
        assert_eq!(args[2].1.as_name(), Some("WT"));
        assert!(args[3].1.expr().is_nil());
        assert!(
            args.iter()
                .all(|(_, quosure)| quosure.sexp().inherits_class(c"quosure"))
        );
        i32::try_from(args.len()).expect("a handful of arguments")
    }
}

// endregion
