//! `AsNumeric` / `AsNumericVec` against a real R runtime: every accepted input
//! type, `NA` of every type, factor labels (not codes), R's string parsing via
//! `R_strtod`, and the batched `non-numeric value(s)` error. Also the
//! `AsFromStr` / `AsFromStrVec` NA and quoting behaviour.

mod r_test_utils;

use miniextendr_api::from_r::{SexpError, TryFromSexp};
use miniextendr_api::{AsFromStr, AsFromStrVec, AsNumeric, AsNumericVec, SEXP, r_str};

/// Evaluate `src` and convert the result right away (conversion allocates no
/// R memory, so the unprotected result stays valid).
fn vec_of(src: &str) -> Result<Vec<Option<f64>>, SexpError> {
    let sexp: SEXP = r_str!(src).expect("R source should evaluate");
    AsNumericVec::try_from_sexp(sexp).map(|v| v.0)
}

fn scalar_of(src: &str) -> Result<Option<f64>, SexpError> {
    let sexp: SEXP = r_str!(src).expect("R source should evaluate");
    AsNumeric::try_from_sexp(sexp).map(|v| v.0)
}

fn invalid_message(err: SexpError) -> String {
    match err {
        SexpError::InvalidValue(msg) => msg,
        other => panic!("expected InvalidValue, got {other:?}"),
    }
}

#[test]
fn as_numeric_suite() {
    r_test_utils::with_r_thread(|| {
        doubles_pass_through();
        integers_and_logicals_widen();
        character_parses_like_as_numeric();
        factors_read_their_labels();
        failures_are_batched();
        other_types_are_refused();
        scalar_checks_length();
        from_str_reports_na_and_quotes_values();
    });
}

fn doubles_pass_through() {
    let v = vec_of("c(1.5, NA, NaN, -Inf)").unwrap();
    assert_eq!(v[0], Some(1.5));
    assert_eq!(v[1], None);
    assert!(v[2].is_some_and(f64::is_nan), "plain NaN stays Some(NaN)");
    assert_eq!(v[3], Some(f64::NEG_INFINITY));
    // A computed NA has a different bit pattern from `NA_real_`; R still
    // reads it as NA, and so must the marker.
    assert_eq!(vec_of("c(1, NA) * 2").unwrap(), vec![Some(2.0), None]);
    assert_eq!(vec_of("double(0)").unwrap(), Vec::<Option<f64>>::new());
}

fn integers_and_logicals_widen() {
    assert_eq!(
        vec_of("c(1L, NA, -3L)").unwrap(),
        vec![Some(1.0), None, Some(-3.0)]
    );
    assert_eq!(
        vec_of("c(TRUE, NA, FALSE)").unwrap(),
        vec![Some(1.0), None, Some(0.0)]
    );
}

fn character_parses_like_as_numeric() {
    let v = vec_of(
        r#"c(" 1.5 ", "Inf", "-inf", "1e3", "0x1A", "0x1p3", "+3", "5.", "NA", " NA ", "", "   ", NA)"#,
    )
    .unwrap();
    assert_eq!(
        v,
        vec![
            Some(1.5),
            Some(f64::INFINITY),
            Some(f64::NEG_INFINITY),
            Some(1000.0),
            Some(26.0),
            Some(8.0),
            Some(3.0),
            Some(5.0),
            None,
            None,
            None,
            None,
            None,
        ]
    );
    let nan = vec_of(r#""NaN""#).unwrap();
    assert!(nan[0].is_some_and(f64::is_nan));
}

fn factors_read_their_labels() {
    assert_eq!(
        vec_of(r#"factor(c("10", "2", NA, "10"))"#).unwrap(),
        vec![Some(10.0), Some(2.0), None, Some(10.0)]
    );
    // A bad label is reported at every element that uses it.
    let msg = invalid_message(vec_of(r#"factor(c("1", "n/a", "n/a"))"#).unwrap_err());
    assert_eq!(msg, r#"non-numeric value(s): "n/a", "n/a" (elements 2, 3)"#);
    // Codes outside the levels: a malformed factor.
    let msg = invalid_message(
        vec_of(r#"structure(c(1L, 3L), levels = c("1", "2"), class = "factor")"#).unwrap_err(),
    );
    assert!(msg.contains("malformed factor"), "{msg}");
}

fn failures_are_batched() {
    let msg = invalid_message(vec_of(r#"c("1", "n/a", "3", "4", "<0.1")"#).unwrap_err());
    assert_eq!(
        msg,
        r#"non-numeric value(s): "n/a", "<0.1" (elements 2, 5)"#
    );

    let msg = invalid_message(vec_of(r#"c(paste0("x", 1:12), "7")"#).unwrap_err());
    assert!(
        msg.ends_with(r#""x10" (elements 1, 2, 3, 4, 5, 6, 7, 8, 9, 10); and 2 more"#),
        "{msg}"
    );
}

fn other_types_are_refused() {
    for src in ["list(1)", "as.raw(1)", "1i", "NULL"] {
        let err = vec_of(src).unwrap_err();
        assert!(matches!(err, SexpError::Type(_)), "{src}: {err:?}");
    }
}

fn scalar_checks_length() {
    assert_eq!(scalar_of("2L").unwrap(), Some(2.0));
    assert_eq!(scalar_of(r#"factor("10")"#).unwrap(), Some(10.0));
    assert_eq!(scalar_of("NA_character_").unwrap(), None);
    assert!(matches!(
        scalar_of("c(1, 2)").unwrap_err(),
        SexpError::Length(_)
    ));
    assert!(matches!(
        scalar_of("list(1)").unwrap_err(),
        SexpError::Type(_)
    ));
    let msg = invalid_message(scalar_of(r#""n/a""#).unwrap_err());
    assert_eq!(msg, r#"non-numeric value(s): "n/a" (element 1)"#);

    // `Option<AsNumeric>`: NULL is `None`, anything else goes through the marker.
    let sexp = r_str!("NULL").unwrap();
    assert_eq!(Option::<AsNumeric>::try_from_sexp(sexp).unwrap(), None);
    let sexp = r_str!(r#""4""#).unwrap();
    assert_eq!(
        Option::<AsNumeric>::try_from_sexp(sexp).unwrap(),
        Some(AsNumeric(Some(4.0)))
    );
}

fn from_str_reports_na_and_quotes_values() {
    let sexp = r_str!("NA_character_").unwrap();
    let err = AsFromStr::<i32>::try_from_sexp(sexp).unwrap_err();
    assert!(matches!(err, SexpError::Na(_)), "{err:?}");

    let sexp = r_str!(r#""n/a""#).unwrap();
    let msg = invalid_message(AsFromStr::<i32>::try_from_sexp(sexp).unwrap_err());
    assert_eq!(msg, r#""n/a": invalid digit found in string"#);

    let sexp = r_str!(r#"c("1", "n/a", NA)"#).unwrap();
    let msg = invalid_message(AsFromStrVec::<i32>::try_from_sexp(sexp).unwrap_err());
    assert_eq!(
        msg,
        r#""n/a": invalid digit found in string (element 2); NA is not allowed (element 3)"#
    );
}
