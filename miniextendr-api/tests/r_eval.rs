//! Integration tests for the `r!` / `r_str!` eval macros and the underlying
//! `r_eval_str` parse + eval helper (issue #687).
//!
//! These exercise the real R runtime: parse a string of R source, evaluate it
//! with full GC protection, and verify the parse-error path returns `Err`
//! rather than segfaulting or silently producing a wrong value.

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
