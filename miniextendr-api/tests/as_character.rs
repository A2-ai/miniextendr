//! `AsCharacter` / `AsCharacterVec` against a real R runtime: every atomic
//! type gives exactly the strings `as.character()` gives, factors read by
//! their labels, classed vectors go through their `as.character()` method,
//! and lists or other non-atomic input are refused.

mod r_test_utils;

use miniextendr_api::from_r::{SexpError, TryFromSexp};
use miniextendr_api::gc_protect::OwnedProtect;
use miniextendr_api::{AsCharacter, AsCharacterVec, SEXP, r_str};

/// Evaluate `src` and convert the result. The conversion allocates (the
/// coerced vector, deferred-string elements, the dispatch call), so the
/// evaluated input stays protected throughout.
fn vec_of(src: &str) -> Result<Vec<Option<String>>, SexpError> {
    let sexp: SEXP = r_str!(src).expect("R source should evaluate");
    let sexp = unsafe { OwnedProtect::new(sexp) };
    AsCharacterVec::try_from_sexp(sexp.get()).map(|v| v.0)
}

fn scalar_of(src: &str) -> Result<Option<String>, SexpError> {
    let sexp: SEXP = r_str!(src).expect("R source should evaluate");
    let sexp = unsafe { OwnedProtect::new(sexp) };
    AsCharacter::try_from_sexp(sexp.get()).map(|v| v.0)
}

/// `as.character(<src>)` as R computes it, for comparison.
fn r_as_character(src: &str) -> Vec<Option<String>> {
    vec_of(&format!("as.character({src})")).expect("as.character() output is character")
}

fn strings(values: &[Option<&str>]) -> Vec<Option<String>> {
    values.iter().map(|v| v.map(str::to_owned)).collect()
}

fn invalid_message(err: SexpError) -> String {
    match err {
        SexpError::InvalidValue(msg) => msg,
        other => panic!("expected InvalidValue, got {other:?}"),
    }
}

#[test]
fn as_character_suite() {
    r_test_utils::with_r_thread(|| {
        matches_as_character_for_atomic_types();
        doubles_use_r_formatting();
        factors_read_their_labels();
        classed_vectors_dispatch();
        method_failures_are_conversion_errors();
        dims_and_names_are_dropped();
        other_types_are_refused();
        scalar_checks_length();
    });
}

fn matches_as_character_for_atomic_types() {
    for src in [
        r#"c("a", NA, "NA", "")"#,
        "c(1L, NA, -3L)",
        "c(0.1 + 0.2, 1e6, 100, 1e5, 1/3, NaN, Inf, -Inf, NA)",
        "c(1, NA) * 2",
        "c(TRUE, NA, FALSE)",
        "c(1+2i, NA)",
        "as.raw(c(0, 255))",
        "1:5",
        "character(0)",
        "integer(0)",
        "double(0)",
    ] {
        assert_eq!(vec_of(src).unwrap(), r_as_character(src), "{src}");
    }
}

fn doubles_use_r_formatting() {
    assert_eq!(
        vec_of("c(0.1 + 0.2, 1e6, 100, NaN, NA)").unwrap(),
        strings(&[Some("0.3"), Some("1e+06"), Some("100"), Some("NaN"), None])
    );
    assert_eq!(
        vec_of("c(TRUE, NA)").unwrap(),
        strings(&[Some("TRUE"), None])
    );
    // The token "NA" and blank strings are values, not missing.
    assert_eq!(
        vec_of(r#"c("NA", "", NA)"#).unwrap(),
        strings(&[Some("NA"), Some(""), None])
    );
}

fn factors_read_their_labels() {
    assert_eq!(
        vec_of(r#"factor(c("10", "2", NA, "10"))"#).unwrap(),
        strings(&[Some("10"), Some("2"), None, Some("10")])
    );
    assert_eq!(
        vec_of(r#"factor(c("b", "a"), levels = c("b", "a"), ordered = TRUE)"#).unwrap(),
        strings(&[Some("b"), Some("a")])
    );
    assert_eq!(
        vec_of("factor(character(0))").unwrap(),
        Vec::<Option<String>>::new()
    );
}

fn classed_vectors_dispatch() {
    // `coerceVector()` on the double underneath would give "19737".
    assert_eq!(
        vec_of(r#"as.Date(c("2024-01-15", NA))"#).unwrap(),
        strings(&[Some("2024-01-15"), None])
    );
    for src in [
        r#"as.Date(c("2024-01-15", NA))"#,
        r#"as.POSIXct(c("2024-01-15 10:30:00", "2024-01-16"), tz = "UTC")"#,
        r#"structure(c("x", "y"), class = "noquote")"#,
    ] {
        assert_eq!(vec_of(src).unwrap(), r_as_character(src), "{src}");
    }
    // A registered S3 method decides the text.
    r_str!(
        r#".S3method("as.character", "mx_test_label",
            function(x, ...) paste0("L", unclass(x)))"#
    )
    .unwrap();
    assert_eq!(
        vec_of(r#"structure(1:2, class = "mx_test_label")"#).unwrap(),
        strings(&[Some("L1"), Some("L2")])
    );
}

fn method_failures_are_conversion_errors() {
    r_str!(
        r#".S3method("as.character", "mx_test_boom",
            function(x, ...) stop("no labels here"))"#
    )
    .unwrap();
    let msg = invalid_message(vec_of(r#"structure(1L, class = "mx_test_boom")"#).unwrap_err());
    assert!(msg.starts_with("as.character() failed: "), "{msg}");
    assert!(msg.contains("no labels here"), "{msg}");

    r_str!(
        r#".S3method("as.character", "mx_test_numeric",
            function(x, ...) unclass(x))"#
    )
    .unwrap();
    let msg = invalid_message(vec_of(r#"structure(1L, class = "mx_test_numeric")"#).unwrap_err());
    assert_eq!(
        msg,
        "as.character() returned integer, not a character vector"
    );
}

fn dims_and_names_are_dropped() {
    assert_eq!(
        vec_of("matrix(1:4, 2)").unwrap(),
        strings(&[Some("1"), Some("2"), Some("3"), Some("4")])
    );
    assert_eq!(
        vec_of("array(c(1.5, 2), dim = 2)").unwrap(),
        strings(&[Some("1.5"), Some("2")])
    );
    assert_eq!(
        vec_of("c(a = 1L, b = 2L)").unwrap(),
        strings(&[Some("1"), Some("2")])
    );
}

fn other_types_are_refused() {
    for src in [
        "list(1)",
        "data.frame(a = 1)",
        "NULL",
        "quote(x)",
        "as.POSIXlt(\"2024-01-15\")",
    ] {
        let err = vec_of(src).unwrap_err();
        assert!(matches!(err, SexpError::Type(_)), "{src}: {err:?}");
    }
    // `Option<AsCharacterVec>`: NULL is `None`, anything else goes through the marker.
    let sexp = r_str!("NULL").unwrap();
    assert_eq!(Option::<AsCharacterVec>::try_from_sexp(sexp).unwrap(), None);
    let sexp = unsafe { OwnedProtect::new(r_str!("c(1L, 2L)").unwrap()) };
    assert_eq!(
        Option::<AsCharacterVec>::try_from_sexp(sexp.get()).unwrap(),
        Some(AsCharacterVec(strings(&[Some("1"), Some("2")])))
    );
}

fn scalar_checks_length() {
    assert_eq!(scalar_of("101L").unwrap().as_deref(), Some("101"));
    assert_eq!(scalar_of("0.1 + 0.2").unwrap().as_deref(), Some("0.3"));
    assert_eq!(
        scalar_of(r#"factor("S-02")"#).unwrap().as_deref(),
        Some("S-02")
    );
    assert_eq!(scalar_of("TRUE").unwrap().as_deref(), Some("TRUE"));
    assert_eq!(scalar_of("NA").unwrap(), None);
    assert!(matches!(
        scalar_of("c(1, 2)").unwrap_err(),
        SexpError::Length(_)
    ));
    assert!(matches!(
        scalar_of("character(0)").unwrap_err(),
        SexpError::Length(_)
    ));
    assert!(matches!(
        scalar_of("list(1)").unwrap_err(),
        SexpError::Type(_)
    ));
    // A method that returns another length than it was given.
    r_str!(
        r#".S3method("as.character", "mx_test_two",
            function(x, ...) c("a", "b"))"#
    )
    .unwrap();
    assert!(matches!(
        scalar_of(r#"structure(1L, class = "mx_test_two")"#).unwrap_err(),
        SexpError::Length(_)
    ));

    let sexp = r_str!("NULL").unwrap();
    assert_eq!(Option::<AsCharacter>::try_from_sexp(sexp).unwrap(), None);
}
