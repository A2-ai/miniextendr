//! `SexpExt::has_attributes` and `SexpExt::is_identical`, checked against
//! R's own `attributes()` and `identical()`.

mod r_test_utils;

use miniextendr_api::gc_protect::ProtectScope;
use miniextendr_api::{SEXP, SexpExt, r_str};

/// Evaluate `code` in the global environment and root the value in `scope`.
fn eval(scope: &ProtectScope, code: &str) -> SEXP {
    let value = r_str!(code).unwrap_or_else(|err| panic!("evaluating `{code}`: {err}"));
    unsafe { scope.protect_raw(value) }
}

/// Evaluate an R expression that yields `TRUE` or `FALSE`.
fn eval_flag(code: &str) -> bool {
    let value = r_str!(code).unwrap_or_else(|err| panic!("evaluating `{code}`: {err}"));
    value
        .as_logical()
        .unwrap_or_else(|| panic!("`{code}` returned NA"))
}

/// (R expression, whether it has attributes)
const ATTRIBUTE_CASES: &[(&str, bool)] = &[
    ("NULL", false),
    ("FALSE", false),
    ("1:3", false),
    ("list(1, 'a')", false),
    ("c(a = 1, b = 2)", true),
    ("matrix(1:4, 2L)", true),
    ("factor(c('x', 'y'))", true),
    ("structure(list(), class = 'mx_thing')", true),
    ("structure(FALSE, foo = 1)", true),
    // A pairlist's tag names count, as `attributes()` reports them.
    ("pairlist(1)", false),
    ("pairlist(a = 1)", true),
    // Calls and symbols are passed through `quote()`, not evaluated.
    ("quote(undefined_fn(undefined_arg))", false),
    (
        "structure(quote(undefined_fn(undefined_arg)), foo = 1)",
        true,
    ),
    ("quote(undefined_symbol)", false),
    ("new.env()", false),
];

#[test]
fn has_attributes_matches_attributes() {
    r_test_utils::with_r_thread(|| {
        for &(code, expected) in ATTRIBUTE_CASES {
            let scope = unsafe { ProtectScope::new() };
            let x = eval(&scope, code);
            assert_eq!(x.has_attributes(), expected, "has_attributes({code})");
            let r_answer = eval_flag(&format!("!is.null(attributes({code}))"));
            assert_eq!(expected, r_answer, "!is.null(attributes({code})) in R");
        }
    });
}

/// A CHARSXP's attribute slot holds the string-cache chain, not attributes.
#[test]
fn has_attributes_is_false_for_charsxp() {
    r_test_utils::with_r_thread(|| {
        let scope = unsafe { ProtectScope::new() };
        // Enough distinct strings that some cache buckets hold a chain.
        let strings = eval(&scope, "c(as.character(1:5000), 'a')");
        for i in [0, 4999, 5000] {
            let charsxp = strings.string_elt(i);
            assert!(
                !charsxp.has_attributes(),
                "string_elt({i}) has no attributes"
            );
        }
    });
}

/// R expressions compared with `FALSE`, and whether `identical(x, FALSE)`.
const IDENTICAL_FALSE_CASES: &[(&str, bool)] = &[
    ("FALSE", true),
    ("!TRUE", true),
    ("as.logical(0L)", true),
    ("TRUE", false),
    ("NA", false),
    ("0L", false),
    ("0", false),
    ("logical(0)", false),
    ("c(FALSE, FALSE)", false),
    ("c(a = FALSE)", false),
    ("structure(FALSE, foo = 1)", false),
    ("list(FALSE)", false),
    ("factor('FALSE')", false),
    ("NULL", false),
];

#[test]
fn is_identical_matches_identical_false() {
    r_test_utils::with_r_thread(|| {
        for &(code, expected) in IDENTICAL_FALSE_CASES {
            let scope = unsafe { ProtectScope::new() };
            let x = eval(&scope, code);
            assert_eq!(
                x.is_identical(SEXP::scalar_logical(false)),
                expected,
                "is_identical({code}, FALSE)"
            );
            let r_answer = eval_flag(&format!("identical({code}, FALSE)"));
            assert_eq!(expected, r_answer, "identical({code}, FALSE) in R");
        }
    });
}

/// Pairs of R expressions and whether `identical(x, y)` under its defaults.
const IDENTICAL_PAIR_CASES: &[(&str, &str, bool)] = &[
    ("1:3", "c(1L, 2L, 3L)", true),
    ("1:3", "c(1, 2, 3)", false),
    ("c(a = 1)", "c(a = 1)", true),
    ("c(a = 1)", "c(b = 1)", false),
    // attrib.as.set = TRUE: attribute order does not matter.
    (
        "structure(1, a = 1, b = 2)",
        "structure(1, b = 2, a = 1)",
        true,
    ),
    // single.NA = TRUE still tells NA_real_ and NaN apart.
    ("NA_real_", "NaN", false),
    ("list(1, 'a')", "list(1, 'a')", true),
];

#[test]
fn is_identical_matches_identical_pairs() {
    r_test_utils::with_r_thread(|| {
        for &(lhs, rhs, expected) in IDENTICAL_PAIR_CASES {
            let scope = unsafe { ProtectScope::new() };
            let x = eval(&scope, lhs);
            let y = eval(&scope, rhs);
            assert_ne!(x, y, "`{lhs}` and `{rhs}` should be distinct objects");
            assert_eq!(x.is_identical(y), expected, "is_identical({lhs}, {rhs})");
            assert!(x.is_identical(x), "is_identical({lhs}, itself)");
            let r_answer = eval_flag(&format!("identical({lhs}, {rhs})"));
            assert_eq!(expected, r_answer, "identical({lhs}, {rhs}) in R");
        }
    });
}
