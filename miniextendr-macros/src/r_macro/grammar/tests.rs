use super::*;
use proc_macro2::TokenStream;

fn ok(src: &str) {
    let ts: TokenStream = src.parse().expect("not a valid Rust token stream");
    assert!(
        validate(&ts).is_ok(),
        "expected Ok for {src:?}, got: {:?}",
        validate(&ts)
    );
}

fn err(src: &str) {
    let ts: TokenStream = src.parse().expect("not a valid Rust token stream");
    assert!(validate(&ts).is_err(), "expected Err for {src:?}, got Ok");
}

// region: Trailing binary operators (should be rejected)

#[test]
fn trailing_multiply() {
    err("x *");
}

#[test]
fn trailing_divide() {
    err("x /");
}

#[test]
fn trailing_caret() {
    err("x ^");
}

// region: Things that should pass

#[test]
fn arithmetic_expression() {
    ok("1L + 2L");
}

#[test]
fn assignment_arrow() {
    // `<-` tokenises as `<` then `-` in proc_macro2 (joint)
    ok(".x <- 41L + 1L");
}

#[test]
fn forward_arrow_assignment() {
    ok("41L + 1L -> .y");
}

#[test]
fn semicolon_sequence() {
    ok("a <- 7L ; a * 6L");
}

#[test]
fn multi_statement_with_env_form() {
    // The env form is stripped before validation, but the body `local_val <- 7L; local_val * 6L`
    // should be valid.
    ok("local_val <- 7L ; local_val * 6L");
}

#[test]
fn trailing_semicolon_ok() {
    ok("x <- 1L ;");
}

#[test]
fn nchar_call() {
    ok(r#"nchar("hello")"#);
}

#[test]
fn index_with_empty_first_arg() {
    // x[,1] — valid R matrix indexing; second comma-slot is non-empty
    // This is a bracket group, not a function call.
    ok("x[,1]");
}

#[test]
fn empty_call_args_are_valid_missing_args() {
    // R's sublist grammar allows empty slots in every call form — they
    // become the missing-arg sentinel. `matrix(, 2, 2)` is idiomatic.
    ok("f(, x)");
    ok("f(x,,y)");
    ok("matrix(, 2, 2)");
}

#[test]
fn double_star_is_r_power_synonym() {
    // R's parser accepts `**` as an undocumented synonym for `^`.
    ok("2 ** 3");
}

#[test]
fn empty_function_call() {
    // f() is valid
    ok("f()");
}

#[test]
fn percent_in_percent() {
    // %in% tokenises as `%` ident `%` — must pass
    ok("x % in % y");
}

#[test]
fn tilde_formula() {
    ok("y ~ x");
}

#[test]
fn logical_operators() {
    ok("x && y");
    ok("x || y");
}

// region: Control flow

#[test]
fn if_else_valid() {
    ok("if (x > 0) x else - x");
}

#[test]
fn while_valid() {
    ok("while (i < 10) i <- i + 1L");
}

#[test]
fn for_valid() {
    ok("for (i in 1:10) print(i)");
}

// region: Control flow errors

#[test]
fn if_empty_condition() {
    err("if () x");
}

#[test]
fn if_no_body() {
    err("if (x)");
}

#[test]
fn for_missing_in() {
    err("for (x) {}");
}

#[test]
fn for_empty() {
    err("for () {}");
}

// region: Consecutive non-unary binary operators

#[test]
fn double_star() {
    err("x * * y");
}

#[test]
fn double_slash() {
    err("x / / y");
}

#[test]
fn trailing_empty_call_arg_ok() {
    ok("f(x,)");
}

#[test]
fn empty_paren_after_binary_at() {
    // `x @ ()` — `@` is a pure binary operator
    err("x @ ()");
}
