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

use miniextendr_api::expression::r_eval_str;
use miniextendr_api::sys::R_GlobalEnv;
use miniextendr_api::{SexpExt, r, r_str};

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

/// A genuine R syntax error must return `Err`, not crash or wrong-answer.
fn test_parse_error_is_err() {
    // Unbalanced paren — a classic parse failure.
    let err = unsafe { r_eval_str("1 + (2", R_GlobalEnv) };
    assert!(err.is_err(), "unbalanced paren must be an Err, got {err:?}");

    // Outright garbage tokens.
    let err2 = unsafe { r_eval_str("if if if", R_GlobalEnv) };
    assert!(err2.is_err(), "garbage must be an Err, got {err2:?}");
}

/// A runtime R error (valid syntax, failing eval) is captured as `Err`.
fn test_eval_error_is_err() {
    let err = r_str!("stop(\"boom from R\")");
    assert!(err.is_err(), "stop() should surface as Err");
    let msg = err.unwrap_err();
    assert!(
        msg.contains("boom from R"),
        "error message should carry R's message, got: {msg}"
    );

    // Calling an undefined function is also an eval-time error.
    let err2 = r_str!("this_function_does_not_exist_687()");
    assert!(err2.is_err(), "undefined function should surface as Err");
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
