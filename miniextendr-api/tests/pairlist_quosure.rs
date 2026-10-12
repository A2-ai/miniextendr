//! Public pairlist accessors (`PairListExt`, `args()`) and the public
//! `Quosure` constructors (`from_sexp`, `new`, `args`) (#1903).
//!
//! Each R object is bound in the global environment, which roots it for the
//! test. Quosures are built by hand (`structure(quote(~x), class = ...)`), so
//! only `Quosure::new` and `Quosure::args` on a bare expression need rlang;
//! those parts skip when it is not installed.

mod r_test_utils;

use std::panic::{AssertUnwindSafe, catch_unwind};

use miniextendr_api::gc_protect::ProtectScope;
use miniextendr_api::sys::{R_EmptyEnv, R_GlobalEnv};
use miniextendr_api::{PairListExt, Quosure, SEXP, SEXPTYPE, SexpExt, r_str};

/// Evaluate R source in the global environment.
fn r(code: &str) -> SEXP {
    r_str!(code).unwrap_or_else(|e| panic!("`{code}` failed: {e}"))
}

/// The names and values `args()` yields, the names owned.
fn args_of(x: SEXP) -> Vec<(Option<String>, SEXP)> {
    x.args()
        .map(|(name, value)| (name.map(str::to_owned), value))
        .collect()
}

fn rlang_installed() -> bool {
    r("requireNamespace('rlang', quietly = TRUE)").as_logical() == Some(true)
}

#[test]
fn pairlist_suite() {
    r_test_utils::with_r_thread(|| {
        call_args_skip_the_function();
        pairlist_args_are_every_element();
        null_has_no_args();
        args_refuses_other_types();
        setters_change_a_duplicate_only();
    });
}

#[test]
fn quosure_suite() {
    r_test_utils::with_r_thread(|| {
        from_sexp_reads_a_quosure_only();
        from_sexp_looks_through_nested_quosures();
        from_sexp_reads_a_missing_argument_quosure();
        if rlang_installed() {
            new_wraps_a_bare_expression();
            new_returns_a_quosure_as_it_is();
            args_keep_embedded_quosures();
        }
        new_refuses_a_non_environment();
        args_of_a_non_call_is_none();
    });
}

// region: PairListExt

/// A call's arguments, in order with their names; the function is not one.
/// `NULL` and empty arguments are yielded, not taken as the end.
fn call_args_skip_the_function() {
    let call = r(".pq_call <- quote(f(a, b = 2L, NULL, c = , 'd'))");
    assert_eq!(call.car(), r("quote(f)"));
    let args = args_of(call);
    assert_eq!(args.len(), 5);
    let names: Vec<_> = args.iter().map(|(name, _)| name.as_deref()).collect();
    assert_eq!(names, [None, Some("b"), None, Some("c"), None]);
    assert_eq!(args[0].1, r("quote(a)"));
    assert_eq!(args[1].1.as_integer(), Some(2));
    assert!(args[2].1.is_nil());
    assert_eq!(args[3].1, SEXP::missing_arg());
    assert_eq!(args[4].1.string_elt_str(0), Some("d"));
    // The call's own nodes, read with car / cdr / tag.
    let b = call.cdr().cdr();
    assert_eq!(b.tag(), r("quote(b)"));
    assert!(call.cdr().tag().is_nil());
}

/// A pairlist's elements are all its arguments, the first one included.
fn pairlist_args_are_every_element() {
    let list = r(".pq_list <- pairlist(a = 1, 2)");
    assert_eq!(list.type_of(), SEXPTYPE::LISTSXP);
    let args = args_of(list);
    let names: Vec<_> = args.iter().map(|(name, _)| name.as_deref()).collect();
    assert_eq!(names, [Some("a"), None]);
    assert_eq!(args[0].1.as_real(), Some(1.0));
    assert_eq!(args[1].1.as_real(), Some(2.0));

    let formals = r(".pq_formals <- formals(function(x, y = 1L) NULL)");
    let args = args_of(formals);
    let names: Vec<_> = args.iter().map(|(name, _)| name.as_deref()).collect();
    assert_eq!(names, [Some("x"), Some("y")]);
    assert_eq!(args[0].1, SEXP::missing_arg());
    assert_eq!(args[1].1.as_integer(), Some(1));

    // A call's tail is a pairlist of its arguments.
    let tail = r(".pq_call2 <- quote(g(1L, k = 2L))").cdr();
    assert_eq!(tail.type_of(), SEXPTYPE::LISTSXP);
    assert_eq!(args_of(tail).len(), 2);
}

fn null_has_no_args() {
    assert_eq!(SEXP::nil().args().count(), 0);
    assert_eq!(r("quote(f())").args().count(), 0);
}

fn args_refuses_other_types() {
    let x = r(".pq_int <- 1:3");
    let err = catch_unwind(AssertUnwindSafe(|| x.args().count())).unwrap_err();
    let msg = err.downcast_ref::<String>().expect("a formatted panic");
    assert_eq!(
        msg,
        "`args()` reads a call, a pairlist or NULL, not integer"
    );
}

/// `set_car` / `set_tag` on a `shallow_duplicate()` of a call change the
/// copy; the original keeps its nodes.
fn setters_change_a_duplicate_only() {
    let call = r(".pq_subset <- quote(subset(x, a > 1))");
    let scope = unsafe { ProtectScope::new() };
    let copy = unsafe { scope.protect_raw(call.shallow_duplicate()) };
    copy.cdr().set_car(r("quote(y)"));
    copy.cdr().set_tag(r("quote(data)"));
    assert!(copy.is_identical(r("quote(subset(data = y, a > 1))")));
    assert!(call.is_identical(r("quote(subset(x, a > 1))")));
}

// endregion

// region: Quosure

/// A quosure of `expr` in the global environment, built without rlang.
fn quosure_code(expr: &str) -> String {
    format!(
        "structure(as.call(list(as.name('~'), {expr})), class = c('quosure', 'formula'), \
         .Environment = globalenv())"
    )
}

fn from_sexp_reads_a_quosure_only() {
    for code in ["quote(x)", "1L", "~x", "NULL"] {
        let x = r(&format!(".pq_not <- {code}"));
        assert!(
            unsafe { Quosure::from_sexp(&x) }.is_none(),
            "`{code}` is not a quosure"
        );
    }
    let quo = r(&format!(".pq_quo <- {}", quosure_code("quote(TAD)")));
    let quosure = unsafe { Quosure::from_sexp(&quo) }.expect("a quosure");
    assert_eq!(quosure.sexp(), quo);
    assert_eq!(quosure.expr(), r("quote(TAD)"));
    assert_eq!(quosure.env(), unsafe { R_GlobalEnv });
    assert_eq!(quosure.as_name(), Some("TAD"));
}

/// A quosure wrapping a quosure reads as the inner one.
fn from_sexp_looks_through_nested_quosures() {
    let inner = r(&format!(".pq_inner <- {}", quosure_code("quote(TAD)")));
    let outer = r(
        ".pq_outer <- structure(as.call(list(as.name('~'), .pq_inner)), \
         class = c('quosure', 'formula'), .Environment = emptyenv())",
    );
    assert_eq!(outer.cdr().car(), inner);
    let quosure = unsafe { Quosure::from_sexp(&outer) }.expect("a quosure");
    assert_eq!(quosure.sexp(), inner);
    assert_eq!(quosure.expr(), r("quote(TAD)"));
    assert_eq!(quosure.env(), unsafe { R_GlobalEnv });
}

fn from_sexp_reads_a_missing_argument_quosure() {
    let quo = r(&format!(
        ".pq_missing <- {}",
        quosure_code("quote(expr = )")
    ));
    let quosure = unsafe { Quosure::from_sexp(&quo) }.expect("a quosure");
    assert_eq!(quosure.expr(), SEXP::missing_arg());
}

fn new_wraps_a_bare_expression() {
    unsafe {
        let scope = ProtectScope::new();
        let expr = r(".pq_expr <- quote(a + b)");
        let quosure = Quosure::new(expr, R_GlobalEnv, &scope);
        assert!(quosure.sexp().inherits_class(c"quosure"));
        assert_eq!(quosure.expr(), expr);
        assert_eq!(quosure.env(), R_GlobalEnv);
        assert_eq!(scope.count(), 1);

        let missing = Quosure::new(SEXP::missing_arg(), R_EmptyEnv, &scope);
        assert_eq!(missing.expr(), SEXP::missing_arg());
        assert_eq!(missing.env(), R_EmptyEnv);
    }
}

/// A quosure is not wrapped again; a nested one gives the innermost.
fn new_returns_a_quosure_as_it_is() {
    unsafe {
        let scope = ProtectScope::new();
        let inner = r(".pq_inner");
        let quosure = Quosure::new(inner, R_EmptyEnv, &scope);
        assert_eq!(quosure.sexp(), inner);
        assert_eq!(quosure.env(), R_GlobalEnv);
        let nested = Quosure::new(r(".pq_outer"), R_EmptyEnv, &scope);
        assert_eq!(nested.sexp(), inner);
    }
}

fn new_refuses_a_non_environment() {
    let err = catch_unwind(AssertUnwindSafe(|| unsafe {
        let scope = ProtectScope::new();
        Quosure::new(r("quote(x)"), SEXP::nil(), &scope);
    }))
    .unwrap_err();
    let msg = err.downcast_ref::<String>().expect("a formatted panic");
    assert_eq!(
        msg,
        "`Quosure::new()` takes an environment as `env`, not NULL"
    );
}

/// `c(time = <quosure>, dose = NA, NULL)`: the embedded quosure as it is,
/// the others wrapped in the call quosure's environment.
fn args_keep_embedded_quosures() {
    let quo = r(&format!(
        ".pq_roles <- {}",
        quosure_code("as.call(list(as.name('c'), time = .pq_inner, dose = NA, NULL))")
    ));
    unsafe {
        let call = Quosure::from_sexp(&quo).expect("a quosure");
        let scope = ProtectScope::new();
        let args = call.args(&scope).expect("a call");
        assert_eq!(scope.count(), 1, "one list holds every argument");
        let names: Vec<_> = args.iter().map(|(name, _)| *name).collect();
        assert_eq!(names, [Some("time"), Some("dose"), None]);

        let (_, time) = &args[0];
        assert_eq!(time.sexp(), r(".pq_inner"));
        assert_eq!(time.expr(), r("quote(TAD)"));

        let (_, dose) = &args[1];
        assert!(dose.sexp().inherits_class(c"quosure"));
        assert_eq!(dose.expr().as_logical(), None, "NA as written");
        assert_eq!(dose.expr().type_of(), SEXPTYPE::LGLSXP);
        assert_eq!(dose.env(), R_GlobalEnv);

        let (_, null) = &args[2];
        assert!(null.expr().is_nil());
        assert_eq!(null.env(), R_GlobalEnv);
    }
}

fn args_of_a_non_call_is_none() {
    let quo = r(&format!(".pq_sym <- {}", quosure_code("quote(TAD)")));
    unsafe {
        let quosure = Quosure::from_sexp(&quo).expect("a quosure");
        let scope = ProtectScope::new();
        assert!(quosure.args(&scope).is_none());
        assert_eq!(scope.count(), 0);
    }
}

// endregion
