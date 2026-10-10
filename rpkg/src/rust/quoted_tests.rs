//! Fixtures for arguments passed unevaluated (`Quoted`, `Quosure`) and for
//! evaluating R code from Rust in the caller's R context
//! (`eval_with_handlers`, `RCall::eval_with_handlers`) (#1835), with an R
//! error returned as `Err` (`try_eval_with_handlers`,
//! `RCall::try_eval_with_handlers`) (#1893).
//!
//! Each evaluating fixture holds a [`DropSentinel`] across the evaluation, so
//! the tests can tell that an R exit (an error, a `tryCatch()` handler) ran
//! the Rust destructors on its way out: `quoted_sentinel_drops()` counts them.

use std::sync::atomic::{AtomicI32, Ordering};

use miniextendr_api::expression::{RCall, try_eval_with_handlers};
use miniextendr_api::prelude::{Call, List, OwnedProtect, SEXP, SexpExt};
use miniextendr_api::{Missing, Quosure, Quoted, SEXPTYPE, miniextendr, rust_error};

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

/// Call `pkg::fun(arg)` from the worker thread, through `with_r_thread` and
/// `RCall::eval_with_handlers`: the value as a string (`""` when it is not
/// one). An R exit in the callback continues to the caller's handler, and the
/// worker answers the next call.
#[cfg(any(feature = "worker-thread", feature = "worker-default"))]
#[miniextendr(noexport, worker)]
pub fn quoted_worker_call(pkg: String, fun: String, arg: String) -> String {
    miniextendr_api::worker::with_r_thread(move || {
        // SAFETY: `with_r_thread` runs this on R's main thread, inside the
        // `.Call()`; `RCall` roots its arguments.
        unsafe {
            let arg = OwnedProtect::new(SEXP::scalar_string_from_str(&arg));
            let value = RCall::namespaced(&pkg, &fun)
                .unwrap_or_else(|e| panic!("{e}"))
                .arg(arg.get())
                .eval_with_handlers(miniextendr_api::sys::R_BaseEnv);
            if value.type_of() == SEXPTYPE::STRSXP && value.len() > 0 {
                value.string_elt_str(0).unwrap_or_default().to_owned()
            } else {
                String::new()
            }
        }
    })
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

// endregion
