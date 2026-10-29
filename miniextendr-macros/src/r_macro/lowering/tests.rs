use super::*;

fn parse(s: &str) -> Vec<TokenTree> {
    let ts: TokenStream = s.parse().expect("failed to parse");
    ts.into_iter().collect()
}

fn is_lowerable(s: &str) -> bool {
    classify_call(&parse(s)).is_some()
}

// --- Positive cases: all should be lowerable ---

#[test]
fn simple_call_no_args() {
    assert!(is_lowerable("noop()"));
}

#[test]
fn simple_call_integer_l_suffix() {
    assert!(is_lowerable("identity(42L)"));
}

#[test]
fn simple_call_negative_integer() {
    assert!(is_lowerable("identity(-1L)"));
}

#[test]
fn simple_call_string() {
    assert!(is_lowerable(r#"paste("hello", "world")"#));
}

#[test]
fn simple_call_float() {
    assert!(is_lowerable("identity(3.14)"));
}

#[test]
fn simple_call_true_false() {
    assert!(is_lowerable("identity(TRUE)"));
    assert!(is_lowerable("identity(FALSE)"));
}

#[test]
fn simple_call_null() {
    assert!(is_lowerable("is.null(NULL)"));
}

#[test]
fn simple_call_na_forms() {
    assert!(is_lowerable("identity(NA)"));
    assert!(is_lowerable("identity(NA_integer_)"));
    assert!(is_lowerable("identity(NA_real_)"));
    assert!(is_lowerable("identity(NA_character_)"));
}

#[test]
fn simple_call_inf_nan() {
    assert!(is_lowerable("identity(Inf)"));
    assert!(is_lowerable("identity(NaN)"));
}

#[test]
fn simple_call_neg_inf() {
    assert!(is_lowerable("identity(-Inf)"));
}

#[test]
fn simple_call_bare_symbol() {
    assert!(is_lowerable("identity(x)"));
}

#[test]
fn named_arg() {
    assert!(is_lowerable("seq(1L, 10L, by = 2L)"));
}

#[test]
fn nested_call() {
    assert!(is_lowerable("c(1L, c(2L, 3L))"));
}

#[test]
fn namespaced_call() {
    assert!(is_lowerable("base::sum(1L, 2L)"));
}

#[test]
fn plain_integer_lowers_as_double() {
    // R parses unsuffixed numeric literals as double: typeof(42) is
    // "double". Only `42L` is INTSXP.
    let call = classify_call(&parse("identity(42)")).unwrap();
    assert!(matches!(call.args[0].value, LowerAtom::RealLit(v) if v == 42.0));
    let call = classify_call(&parse("identity(42L)")).unwrap();
    assert!(matches!(call.args[0].value, LowerAtom::IntLit(42)));
}

#[test]
fn bare_na_is_logical_na() {
    // typeof(NA) is "logical", not integer.
    let call = classify_call(&parse("identity(NA)")).unwrap();
    assert!(matches!(call.args[0].value, LowerAtom::Na));
}

// --- Negative cases: all should fall back ---

#[test]
fn arithmetic_not_lowerable() {
    assert!(!is_lowerable("1L + 2L"));
}

#[test]
fn assignment_not_lowerable() {
    assert!(!is_lowerable("x <- 5L"));
}

#[test]
fn semicolon_sequence_not_lowerable() {
    // Two-statement sequence — only a call is lowerable.
    assert!(!is_lowerable("x <- 5L; x"));
}

#[test]
fn indexing_not_lowerable() {
    assert!(!is_lowerable("x[1L]"));
}

#[test]
fn dollar_not_lowerable() {
    assert!(!is_lowerable("x$y"));
}

#[test]
fn bare_symbol_not_lowerable() {
    // A bare symbol at top level is not a call — fallback.
    assert!(!is_lowerable("x"));
}

#[test]
fn na_complex_arg_not_lowerable() {
    // NA_complex_ has no constructor → not lowerable.
    assert!(!is_lowerable("identity(NA_complex_)"));
}

#[test]
fn if_not_lowerable() {
    assert!(!is_lowerable("if (TRUE) 1L else 2L"));
}

#[test]
fn empty_arg_slot_not_lowerable() {
    // `matrix(, 2, 2)` is valid R (missing arg) and passes the grammar
    // validator — it must fall back to the string path, never lower with
    // the empty slot silently dropped.
    assert!(!is_lowerable("matrix(, 2, 2)"));
    assert!(!is_lowerable("f(x,,y)"));
}

#[test]
fn spaced_colons_not_namespaced() {
    // `pkg : : fn(x)` is an R parse error — must not lower to a working
    // namespaced call; fallback lets the string path reject it.
    assert!(!is_lowerable("pkg : : fn(1L)"));
}
