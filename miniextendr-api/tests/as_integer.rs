//! `AsInteger` / `AsIntegerVec` against a real R runtime: every accepted input
//! type, `NA` of every type, factor labels (not codes), R's string parsing via
//! `R_strtod`, the refusal of fractions and out-of-range values, the batched
//! `non-integer value(s)` error, and each marker's `IntoR` value read back by
//! its `TryFromSexp`.

mod r_test_utils;

use miniextendr_api::from_r::{SexpError, TryFromSexp};
use miniextendr_api::gc_protect::OwnedProtect;
use miniextendr_api::{AsInteger, AsIntegerVec, IntoR, SEXP, SEXPTYPE, SexpExt, r_str};

/// Evaluate `src` and convert the result right away (conversion allocates no
/// R memory, so the unprotected result stays valid).
fn vec_of(src: &str) -> Result<Vec<Option<i32>>, SexpError> {
    let sexp: SEXP = r_str!(src).expect("R source should evaluate");
    AsIntegerVec::try_from_sexp(sexp).map(|v| v.0)
}

fn scalar_of(src: &str) -> Result<Option<i32>, SexpError> {
    let sexp: SEXP = r_str!(src).expect("R source should evaluate");
    AsInteger::try_from_sexp(sexp).map(|v| v.0)
}

/// `value` to R and back, the way a trait's View hands an argument to the
/// implementing method: the R value's type, and the value read back from it.
fn round_trip<T: IntoR + TryFromSexp<Error = SexpError>>(value: T) -> (SEXPTYPE, T) {
    let sexp = unsafe { OwnedProtect::new(value.into_sexp()) };
    let back = T::try_from_sexp(sexp.get()).expect("a marker reads its own R value back");
    (sexp.get().type_of(), back)
}

fn invalid_message(err: SexpError) -> String {
    match err {
        SexpError::InvalidValue(msg) => msg,
        other => panic!("expected InvalidValue, got {other:?}"),
    }
}

#[test]
fn as_integer_suite() {
    r_test_utils::with_r_thread(|| {
        integers_and_logicals_pass();
        whole_doubles_pass();
        character_parses_like_as_numeric();
        factors_read_their_labels();
        refusals_are_batched();
        other_types_are_refused();
        scalar_checks_length();
        markers_round_trip();
    });
}

fn integers_and_logicals_pass() {
    assert_eq!(
        vec_of("c(1L, NA, -3L, 2147483647L)").unwrap(),
        vec![Some(1), None, Some(-3), Some(i32::MAX)]
    );
    assert_eq!(
        vec_of("c(TRUE, NA, FALSE)").unwrap(),
        vec![Some(1), None, Some(0)]
    );
    assert_eq!(vec_of("integer(0)").unwrap(), Vec::<Option<i32>>::new());
}

fn whole_doubles_pass() {
    assert_eq!(
        vec_of("c(3, -0, NA, NaN, 2147483647, -2147483647)").unwrap(),
        vec![
            Some(3),
            Some(0),
            None,
            None,
            Some(i32::MAX),
            Some(-i32::MAX)
        ]
    );
    // A computed NA has a different bit pattern from `NA_real_`; R still
    // reads it as NA, and so must the marker.
    assert_eq!(vec_of("c(1, NA) * 2").unwrap(), vec![Some(2), None]);
}

fn character_parses_like_as_numeric() {
    assert_eq!(
        vec_of(r#"c(" 15 ", "1e3", "0x1A", "+3", "5.", "-0", "NA", " NA ", "", "   ", NA, "NaN")"#)
            .unwrap(),
        vec![
            Some(15),
            Some(1000),
            Some(26),
            Some(3),
            Some(5),
            Some(0),
            None,
            None,
            None,
            None,
            None,
            None,
        ]
    );
}

fn factors_read_their_labels() {
    assert_eq!(
        vec_of(r#"factor(c("10", "2", NA, "10"))"#).unwrap(),
        vec![Some(10), Some(2), None, Some(10)]
    );
    // A refused label is reported at every element that uses it, numbered by
    // element, not by level.
    let msg = invalid_message(vec_of(r#"factor(c("2", "2.5", "2", "2.5"))"#).unwrap_err());
    assert_eq!(msg, r#"non-integer value(s): "2.5", "2.5" (elements 2, 4)"#);
}

fn refusals_are_batched() {
    // Every kind in one message, in element order: text that is not a number,
    // a fraction, and values out of range, as text or as doubles.
    let msg = invalid_message(
        vec_of(r#"c("1", "n/a", "3.5", "1e10", "-Inf", "-2147483648")"#).unwrap_err(),
    );
    assert_eq!(
        msg,
        r#"non-integer value(s): "n/a", "3.5", "1e10", "-Inf", "-2147483648" (elements 2, 3, 4, 5, 6)"#
    );
    // Doubles are shown as `as.character()` shows them: `2^-1000` has no
    // short decimal form, so it shows 15 significant digits. (Not a literal
    // like `1e-300`: R's parser reads that exactly on x86_64 Linux but a
    // little below it on aarch64 macOS, whose `long double` is a `double`.)
    let msg = invalid_message(
        vec_of("c(1, 3.5, Inf, -Inf, 2^31, -2^31, 3e10, 1e300, 2^-1000, 0.1 + 0.2, 1e-5)")
            .unwrap_err(),
    );
    assert_eq!(
        msg,
        r#"non-integer value(s): "3.5", "Inf", "-Inf", "2147483648", "-2147483648", "3e+10", "1e+300", "9.33263618503219e-302", "0.3", "1e-05" (elements 2, 3, 4, 5, 6, 7, 8, 9, 10, 11)"#
    );
    let msg = invalid_message(vec_of("c(123456.5, 1e10)").unwrap_err());
    assert_eq!(
        msg,
        r#"non-integer value(s): "123456.5", "1e+10" (elements 1, 2)"#
    );

    let msg = invalid_message(vec_of("c(seq_len(12) + 0.5, 7)").unwrap_err());
    assert!(
        msg.ends_with(r#""10.5" (elements 1, 2, 3, 4, 5, 6, 7, 8, 9, 10); and 2 more"#),
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
    assert_eq!(scalar_of("2").unwrap(), Some(2));
    assert_eq!(scalar_of(r#"factor("10")"#).unwrap(), Some(10));
    assert_eq!(scalar_of("NA_character_").unwrap(), None);
    assert!(matches!(
        scalar_of("c(1, 2)").unwrap_err(),
        SexpError::Length(_)
    ));
    assert!(matches!(
        scalar_of("list(1)").unwrap_err(),
        SexpError::Type(_)
    ));
    let msg = invalid_message(scalar_of("3.5").unwrap_err());
    assert_eq!(msg, r#"non-integer value(s): "3.5" (element 1)"#);

    // `Option<AsInteger>`: NULL is `None`, anything else goes through the marker.
    let sexp = r_str!("NULL").unwrap();
    assert_eq!(Option::<AsInteger>::try_from_sexp(sexp).unwrap(), None);
    let sexp = r_str!(r#""4""#).unwrap();
    assert_eq!(
        Option::<AsInteger>::try_from_sexp(sexp).unwrap(),
        Some(AsInteger(Some(4)))
    );
}

fn markers_round_trip() {
    // `Some(marker(None))` is `NA`, not `NULL`, so it comes back as itself.
    assert_eq!(
        round_trip(Some(AsInteger(None))),
        (SEXPTYPE::INTSXP, Some(AsInteger(None)))
    );
    assert_eq!(round_trip(None::<AsInteger>), (SEXPTYPE::NILSXP, None));
    let v = AsIntegerVec(vec![Some(-4), None, Some(i32::MAX)]);
    assert_eq!(round_trip(Some(v.clone())), (SEXPTYPE::INTSXP, Some(v)));
    assert_eq!(round_trip(None::<AsIntegerVec>), (SEXPTYPE::NILSXP, None));
    assert_eq!(
        round_trip(AsIntegerVec(vec![])),
        (SEXPTYPE::INTSXP, AsIntegerVec(vec![]))
    );
}
