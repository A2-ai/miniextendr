//! Integration tests for the `r!` / `r_str!` eval macros and the underlying
//! `r_eval_str` parse + eval helper (issue #687).
//!
//! These exercise the real R runtime: parse a string of R source, evaluate it
//! with full GC protection, and verify the parse-error path returns `Err`
//! rather than segfaulting or silently producing a wrong value.
//!
//! The `r_lowering_*` tests specifically exercise the `Rf_lang*` lowering path
//! added in PR #938 item 2: they verify that lowered calls produce results
//! identical to the string-eval fallback path.

mod r_test_utils;

use miniextendr_api::condition::RError;
use miniextendr_api::expression::{RCall, REnv, r_eval_str};
use miniextendr_api::sys::R_GlobalEnv;
use miniextendr_api::{SEXP, SEXPTYPE, SexpExt, r, r_str};

#[test]
fn r_eval_suite() {
    r_test_utils::with_r_thread(|| {
        test_r_str_arithmetic();
        test_r_macro_arithmetic();
        test_assignment_and_lookup();
        test_env_form();
        test_empty_source();
        test_parse_error_is_err();
        test_eval_error_is_err();
        test_dynamic_format_string();
    });
}

#[test]
fn r_lowering_suite() {
    r_test_utils::with_r_thread(|| {
        test_lowering_c_integers();
        test_lowering_sum();
        test_lowering_paste();
        test_lowering_named_arg();
        test_lowering_nested_call();
        test_lowering_namespaced();
        test_lowering_identity_true();
        test_lowering_is_null_null();
        test_lowering_bare_na_is_logical();
        test_lowering_plain_numeric_is_double();
        test_lowering_fallback_arithmetic();
        test_lowering_fallback_assignment_sequence();
    });
}

/// `r_str!("1L + 2L")` → INTSXP 3.
fn test_r_str_arithmetic() {
    let result = r_str!("1L + 2L").expect("1L + 2L should evaluate");
    assert_eq!(result.as_integer(), Some(3), "expected INTSXP 3");
}

/// `r!(1L + 2L)` (token form) lowers to the same static string and yields 3.
fn test_r_macro_arithmetic() {
    let result = r!(1L + 2L).expect("r!(1L + 2L) should evaluate");
    assert_eq!(result.as_integer(), Some(3), "expected INTSXP 3");

    // A more elaborate single-expression call tree (the issue's motivating
    // shape): nested calls + string args, all in one token stream.
    let nchars = r!(nchar("hello")).expect("nchar(\"hello\") should evaluate");
    assert_eq!(nchars.as_integer(), Some(5));
}

/// Symbol / environment case: assign into the global env, then read it back.
/// Verifies that side-effecting statements take effect and the *last*
/// expression's value is what comes back.
fn test_assignment_and_lookup() {
    // Assignment is not a valid Rust `expr`, so it must go through the token
    // stream (`r!`) or the string form (`r_str!`).
    let assigned = r!(.mx687_x <- 41L + 1L).expect("assignment should evaluate");
    // `<-` returns its value invisibly; we get 42 back.
    assert_eq!(assigned.as_integer(), Some(42));

    let looked_up = r_str!(".mx687_x").expect("lookup should evaluate");
    assert_eq!(looked_up.as_integer(), Some(42));
}

/// Explicit environment form: evaluate in a fresh child environment and confirm
/// the binding lands there, not in the global env.
fn test_env_form() {
    // Build a new environment and bind it in R, then evaluate against it.
    let env = r_str!("new.env()").expect("new.env() should evaluate");
    // env is unprotected; protect it across the next eval (which allocates).
    let _guard = unsafe { miniextendr_api::OwnedProtect::new(env) };

    let v = r!(env: env; local_val <- 7L; local_val * 6L).expect("env eval should work");
    assert_eq!(v.as_integer(), Some(42));

    // The binding must NOT have leaked into the global env.
    let exists = r_str!("exists(\"local_val\", envir = globalenv(), inherits = FALSE)")
        .expect("exists() should evaluate");
    assert_eq!(exists.as_logical(), Some(false));
}

/// Blank source yields `R_NilValue`.
fn test_empty_source() {
    let nil = r_str!("   ").expect("whitespace-only source should be Ok(NULL)");
    assert!(nil.is_nil(), "blank source should evaluate to NULL");
}

/// A genuine R syntax error must return `Err`, not crash or wrong-answer. It
/// has no R condition, so it comes back as a `simpleError` built for it.
fn test_parse_error_is_err() {
    // Unbalanced paren — a classic parse failure.
    let err =
        unsafe { r_eval_str("1 + (2", R_GlobalEnv) }.expect_err("unbalanced paren must be an Err");
    assert_eq!(
        err.message(),
        "incomplete R expression (unbalanced delimiter?): 1 + (2"
    );
    assert_eq!(err.classes(), ["simpleError", "error", "condition"]);
    assert!(err.call().is_none());
    assert!(err.condition().inherits_class(c"error"));

    // Outright garbage tokens.
    let err2 = unsafe { r_eval_str("if if if", R_GlobalEnv) }.expect_err("garbage must be an Err");
    assert!(err2.message().contains("syntax error"), "got: {err2}");
}

/// A runtime R error (valid syntax, failing eval) is captured as `Err`
/// holding R's condition: the message alone, no call for a `stop()` at the
/// top of the source.
fn test_eval_error_is_err() {
    let err = r_str!("stop(\"boom from R\")").expect_err("stop() should surface as Err");
    assert_eq!(err.message(), "boom from R");
    assert_eq!(
        err.to_string(),
        "boom from R",
        "Display is the message alone"
    );
    assert_eq!(err.classes(), ["simpleError", "error", "condition"]);
    assert!(err.specific_classes().is_empty());
    assert!(
        err.call().is_none(),
        "a top-level stop() has no call of its own"
    );
    assert!(err.condition().inherits_class(c"simpleError"));

    // Calling an undefined function is also an eval-time error.
    let err2 = r_str!("this_function_does_not_exist_687()")
        .expect_err("undefined function should surface as Err");
    assert!(
        err2.message()
            .contains("could not find function \"this_function_does_not_exist_687\""),
        "got: {err2}"
    );
}

#[test]
fn r_eval_error_suite() {
    r_test_utils::with_r_thread(|| {
        test_classed_condition_from_function();
        test_nested_error_has_no_traceback();
        test_reraise_class();
        test_missing_package_condition();
        test_error_survives_gc();
    });
}

/// A fresh child of the global environment, rooted for the caller.
fn fresh_env() -> miniextendr_api::OwnedProtect {
    let env = r_str!("new.env()").expect("new.env() evaluates");
    unsafe { miniextendr_api::OwnedProtect::new(env) }
}

/// The head of a call: its function name, for `f(...)`.
fn head(call: SEXP) -> SEXP {
    unsafe { miniextendr_api::sys::CAR(call) }
}

/// A classed condition raised by the called function arrives with its class
/// vector, its message and its call.
fn test_classed_condition_from_function() {
    let env = fresh_env();
    r_str!(
        r#"check_input <- function(x) {
            stop(errorCondition("bad input", class = "my_input_error", call = sys.call()))
        }"#,
        env = env.get()
    )
    .expect("defining the function works");

    let err = unsafe {
        let one = miniextendr_api::OwnedProtect::new(SEXP::scalar_integer(1));
        RCall::new("check_input").arg(one.get()).eval(env.get())
    }
    .expect_err("check_input() raises");
    assert_eq!(err.message(), "bad input");
    assert_eq!(err.classes(), ["my_input_error", "error", "condition"]);
    assert_eq!(err.specific_classes(), ["my_input_error"]);
    assert!(err.inherits("my_input_error"));
    assert!(!err.inherits("simpleError"));

    let call = err.call().expect("the condition has a call");
    assert_eq!(call.type_of(), SEXPTYPE::LANGSXP);
    assert_eq!(head(call), SEXP::symbol("check_input"));
    assert!(err.condition().inherits_class(c"my_input_error"));
}

/// A `stop()` two functions deep, with `showErrorCalls` on (as in `Rscript`):
/// R's printed error would read `Error in g() : boom` plus a `Calls: f -> g`
/// line; the caught message is `boom` and the call `g()`.
fn test_nested_error_has_no_traceback() {
    let env = fresh_env();
    r_str!(
        r#"f <- function() g(); g <- function() stop("boom")"#,
        env = env.get()
    )
    .expect("defining the functions works");

    let old = r_str!("options(showErrorCalls = TRUE)").expect("options() evaluates");
    let old = unsafe { miniextendr_api::OwnedProtect::new(old) };
    let err = unsafe { RCall::new("f").eval(env.get()) }.expect_err("f() raises");
    unsafe { RCall::new("options").arg(old.get()).eval_base() }.expect("options restored");

    assert_eq!(err.message(), "boom");
    let call = err.call().expect("stop() inside g() has a call");
    assert_eq!(head(call), SEXP::symbol("g"));
}

/// `reraise_class` puts the package's classes first, then the caught specific
/// classes, without the base layers and without duplicates.
fn test_reraise_class() {
    let err = r_str!(r#"stop(errorCondition("x", class = c("vctrs_error_x", "rlang_error")))"#)
        .expect_err("raises");
    assert_eq!(err.specific_classes(), ["vctrs_error_x", "rlang_error"]);
    assert_eq!(
        err.reraise_class(["pkg_print_error", "pkg_error"]),
        [
            "pkg_print_error",
            "pkg_error",
            "vctrs_error_x",
            "rlang_error"
        ]
    );
    assert_eq!(
        err.reraise_class("rlang_error"),
        ["rlang_error", "vctrs_error_x"]
    );

    // Converted to `RError` (Send, takes data fields): R's message, the
    // classes given.
    let class = err.reraise_class("pkg_error");
    let carried = RError::from(err).class(class);
    assert_eq!(carried.message_str(), "x");
    assert_eq!(
        carried.classes(),
        ["pkg_error", "vctrs_error_x", "rlang_error"]
    );

    // A condition raised by a miniextendr function carries `rust_error` and
    // R's layers after its own classes; neither is specific.
    let err = r_str!(
        r#"stop(structure(class = c("pkg_bad", "rust_error", "simpleError", "error", "condition"),
                          list(message = "bad", call = NULL)))"#
    )
    .expect_err("raises");
    assert_eq!(err.specific_classes(), ["pkg_bad"]);
    assert!(err.call().is_none());
}

/// `REnv::package_namespace` for a package that is not installed returns R's
/// `packageNotFoundError`, message included.
fn test_missing_package_condition() {
    let err = unsafe { REnv::package_namespace("mxNoSuchPackage1861") }
        .map(|_| ())
        .expect_err("no such package");
    assert!(err.inherits("packageNotFoundError"), "{:?}", err.classes());
    assert!(
        err.message()
            .contains("there is no package called \u{2018}mxNoSuchPackage1861\u{2019}")
            || err
                .message()
                .contains("there is no package called 'mxNoSuchPackage1861'"),
        "got: {err}"
    );
}

/// The condition and its call stay rooted while the error is held across
/// collections.
fn test_error_survives_gc() {
    let env = fresh_env();
    r_str!(
        r#"h <- function() {
            stop(errorCondition(paste("held", "error"), class = "held_error", call = sys.call()))
        }"#,
        env = env.get()
    )
    .expect("defining the function works");
    let err = unsafe { RCall::new("h").eval(env.get()) }.expect_err("h() raises");
    for _ in 0..5 {
        r_str!("invisible(lapply(1:200, function(i) paste(i, 'x'))); gc()").expect("gc runs");
    }
    assert_eq!(err.message(), "held error");
    assert!(err.condition().inherits_class(c"held_error"));
    let message = err.condition().vector_elt(0);
    assert_eq!(message.string_elt_str(0), Some("held error"));
    assert_eq!(head(err.call().expect("h() call")), SEXP::symbol("h"));
}

/// The runtime-string use case from the issue: `format!`-built source.
fn test_dynamic_format_string() {
    let obj = "c(1L, 2L, 3L, 4L)";
    let code = format!("sum({obj})");
    let total = r_str!(&code).expect("dynamic sum should evaluate");
    assert_eq!(total.as_integer(), Some(10));
}

// region: Lowering equivalence tests

/// Helper: assert two SEXPs are identical via R's `identical()`.
fn assert_r_identical(a: miniextendr_api::SEXP, b: miniextendr_api::SEXP) {
    // R_compute_identical returns Rboolean (TRUE=1, FALSE=0).
    // Use From<Rboolean> for bool to convert without naming the private enum.
    let result = unsafe { miniextendr_api::sys::R_compute_identical(a, b, 15) };
    let is_ident: bool = result.into();
    assert!(
        is_ident,
        "R values are not identical: {:?} vs {:?}",
        a.as_integer(),
        b.as_integer()
    );
}

/// `r!(c(1L, 2L, 3L))` — lowered multi-arg call, equivalence with string path.
fn test_lowering_c_integers() {
    let lowered = r!(c(1L, 2L, 3L)).expect("lowered c(1L, 2L, 3L)");
    let string = r_str!("c(1L, 2L, 3L)").expect("string c(1L, 2L, 3L)");
    assert_r_identical(lowered, string);
    assert_eq!(lowered.xlength(), 3);
    assert_eq!(lowered.integer_elt(0), 1);
}

/// `r!(sum(1L, 2L))` — lowered, result equals 3L.
fn test_lowering_sum() {
    let result = r!(sum(1L, 2L)).expect("lowered sum(1L, 2L)");
    let expected = r_str!("sum(1L, 2L)").expect("string sum");
    assert_r_identical(result, expected);
    assert_eq!(result.as_integer(), Some(3));
}

/// `r!(paste("a", "b"))` — string-arg lowering.
fn test_lowering_paste() {
    let lowered = r!(paste("a", "b")).expect("lowered paste");
    let string = r_str!(r#"paste("a", "b")"#).expect("string paste");
    assert_r_identical(lowered, string);
}

/// `r!(seq(1L, 10L, by = 2L))` — named arg lowering.
fn test_lowering_named_arg() {
    let lowered = r!(seq(1L, 10L, by = 2L)).expect("lowered seq with named arg");
    let string = r_str!("seq(1L, 10L, by = 2L)").expect("string seq");
    assert_r_identical(lowered, string);
    assert_eq!(lowered.xlength(), 5);
}

/// `r!(c(1L, c(2L, 3L)))` — nested call lowering.
fn test_lowering_nested_call() {
    let lowered = r!(c(1L, c(2L, 3L))).expect("lowered nested c");
    let string = r_str!("c(1L, c(2L, 3L))").expect("string nested c");
    assert_r_identical(lowered, string);
    assert_eq!(lowered.xlength(), 3);
    assert_eq!(lowered.integer_elt(1), 2);
    assert_eq!(lowered.integer_elt(2), 3);
}

/// `r!(base::sum(1L, 2L))` — pkg::fn lowering.
fn test_lowering_namespaced() {
    let lowered = r!(base::sum(1L, 2L)).expect("lowered base::sum");
    let string = r_str!("base::sum(1L, 2L)").expect("string base::sum");
    assert_r_identical(lowered, string);
    assert_eq!(lowered.as_integer(), Some(3));
}

/// `r!(identity(TRUE))` — bool atom lowering.
fn test_lowering_identity_true() {
    let lowered = r!(identity(TRUE)).expect("lowered identity(TRUE)");
    let string = r_str!("identity(TRUE)").expect("string identity(TRUE)");
    assert_r_identical(lowered, string);
    assert_eq!(lowered.as_logical(), Some(true));
}

/// `r!(is.null(NULL))` — NULL atom lowering.
fn test_lowering_is_null_null() {
    let lowered = r!(is.null(NULL)).expect("lowered is.null(NULL)");
    let string = r_str!("is.null(NULL)").expect("string is.null(NULL)");
    assert_r_identical(lowered, string);
    assert_eq!(lowered.as_logical(), Some(true));
}

/// `r!(identity(NA))` — bare NA must lower as LOGICAL NA (`typeof(NA)` is
/// "logical"), identical to what R's parser produces on the string path.
fn test_lowering_bare_na_is_logical() {
    // identical() distinguishes logical NA from NA_integer_, so this fails
    // if the lowering emits the wrong NA type.
    let lowered = r!(identity(NA)).expect("lowered identity(NA)");
    let string = r_str!("identity(NA)").expect("string identity(NA)");
    assert_r_identical(lowered, string);
}

/// `r!(identity(42))` — unsuffixed numeric literals are DOUBLE in R
/// (`typeof(42)` is "double"; only `42L` is integer).
fn test_lowering_plain_numeric_is_double() {
    let lowered = r!(identity(42)).expect("lowered identity(42)");
    let string = r_str!("identity(42)").expect("string identity(42)");
    assert_r_identical(lowered, string);
    assert_eq!(lowered.as_real(), Some(42.0));
}

/// `r!(1L + 2L)` — arithmetic falls back to string path, still evaluates correctly.
fn test_lowering_fallback_arithmetic() {
    let result = r!(1L + 2L).expect("fallback: 1L + 2L");
    assert_eq!(
        result.as_integer(),
        Some(3),
        "arithmetic fallback should give 3L"
    );
}

/// `r!(x <- 5L; x)` — statement sequence falls back, still evaluates correctly.
fn test_lowering_fallback_assignment_sequence() {
    let result = r!(.mx938_test_x <- 5L; .mx938_test_x).expect("fallback: assignment sequence");
    assert_eq!(
        result.as_integer(),
        Some(5),
        "assignment sequence fallback should give 5L"
    );
}

// endregion
