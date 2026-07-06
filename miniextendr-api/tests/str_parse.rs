//! Tests for the `try_from_sexp_via_str_parse!` string-parse conversions
//! (uuid / url / regex / num-bigint), including the NA-policy standardization:
//! scalar NA errors (`SexpError::Na`), `Option` NA maps to `None`, `Vec<T>`
//! rejects NA with an indexed error, `Vec<Option<T>>` maps NA to `None`.
//!
//! The `Vec<T>` / `Vec<Option<T>>` arms batch all per-element failures into
//! one diagnostic (capped at 10 + "and N more") instead of bailing on the
//! first (#1143).

mod r_test_utils;

#[cfg(any(
    feature = "uuid",
    feature = "regex",
    feature = "url",
    feature = "num-bigint"
))]
use miniextendr_api::from_r::{SexpError, TryFromSexp};
#[cfg(any(
    feature = "uuid",
    feature = "regex",
    feature = "url",
    feature = "num-bigint"
))]
use miniextendr_api::into_r::IntoR;

#[cfg(feature = "uuid")]
use miniextendr_api::Uuid;

#[cfg(feature = "uuid")]
const VALID_UUID: &str = "67e55044-10b1-426f-9247-bb680e5fe0c8";

#[cfg(feature = "uuid")]
#[test]
fn uuid_scalar_valid_and_invalid() {
    r_test_utils::with_r_thread(|| {
        let sexp = VALID_UUID.to_string().into_sexp();
        let u: Uuid = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert_eq!(u.to_string(), VALID_UUID);

        let bad = "not-a-uuid".to_string().into_sexp();
        let err = <Uuid as TryFromSexp>::try_from_sexp(bad).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("invalid UUID"), "got: {msg}");
    });
}

#[cfg(feature = "uuid")]
#[test]
fn uuid_scalar_na_is_na_error_not_parse_error() {
    r_test_utils::with_r_thread(|| {
        let na = Option::<String>::None.into_sexp();
        let err = <Uuid as TryFromSexp>::try_from_sexp(na).unwrap_err();
        assert!(matches!(err, SexpError::Na(_)), "got: {err:?}");
    });
}

#[cfg(feature = "uuid")]
#[test]
fn uuid_option_na_is_none() {
    r_test_utils::with_r_thread(|| {
        let na = Option::<String>::None.into_sexp();
        let opt: Option<Uuid> = TryFromSexp::try_from_sexp(na).unwrap();
        assert!(opt.is_none());
    });
}

#[cfg(feature = "uuid")]
#[test]
fn uuid_vec_rejects_na_with_index() {
    r_test_utils::with_r_thread(|| {
        let sexp = vec![Some(VALID_UUID.to_string()), None].into_sexp();
        let err = <Vec<Uuid> as TryFromSexp>::try_from_sexp(sexp).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("NA at index 1 not allowed for Vec<Uuid>"),
            "got: {msg}"
        );
    });
}

#[cfg(feature = "uuid")]
#[test]
fn uuid_vec_parse_error_carries_index() {
    r_test_utils::with_r_thread(|| {
        let sexp = vec![VALID_UUID.to_string(), "nope".to_string()].into_sexp();
        let err = <Vec<Uuid> as TryFromSexp>::try_from_sexp(sexp).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("invalid UUID at index 1"), "got: {msg}");
    });
}

#[cfg(feature = "uuid")]
#[test]
fn uuid_vec_option_na_is_none() {
    r_test_utils::with_r_thread(|| {
        let sexp = vec![Some(VALID_UUID.to_string()), None].into_sexp();
        let v: Vec<Option<Uuid>> = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert_eq!(v.len(), 2);
        assert!(v[0].is_some());
        assert!(v[1].is_none());
    });
}

// Batched diagnostics (#1143): the vector arms collect every failing element
// into one error instead of bailing on the first.

#[cfg(feature = "uuid")]
#[test]
fn uuid_vec_batches_all_na_and_parse_failures() {
    r_test_utils::with_r_thread(|| {
        let sexp = vec![
            Some(VALID_UUID.to_string()),
            None,
            Some("bad".to_string()),
            Some(VALID_UUID.to_string()),
            Some("worse".to_string()),
        ]
        .into_sexp();
        let err = <Vec<Uuid> as TryFromSexp>::try_from_sexp(sexp).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("Vec<Uuid> conversion failed"), "got: {msg}");
        assert!(
            msg.contains("NA at index 1 not allowed for Vec<Uuid>"),
            "got: {msg}"
        );
        assert!(msg.contains("invalid UUID at index 2"), "got: {msg}");
        assert!(msg.contains("invalid UUID at index 4"), "got: {msg}");
    });
}

#[cfg(feature = "uuid")]
#[test]
fn uuid_vec_batch_caps_at_ten_and_summarizes_rest() {
    r_test_utils::with_r_thread(|| {
        let sexp: Vec<String> = (0..15).map(|i| format!("bad-{i}")).collect();
        let err = <Vec<Uuid> as TryFromSexp>::try_from_sexp(sexp.into_sexp()).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("invalid UUID at index 0"), "got: {msg}");
        assert!(msg.contains("invalid UUID at index 9"), "got: {msg}");
        assert!(!msg.contains("at index 10"), "got: {msg}");
        assert!(msg.contains("and 5 more"), "got: {msg}");
    });
}

#[cfg(feature = "uuid")]
#[test]
fn uuid_vec_option_na_stays_none_while_parse_failures_batch() {
    r_test_utils::with_r_thread(|| {
        let sexp = vec![
            Some(VALID_UUID.to_string()),
            None,
            Some("bad".to_string()),
            None,
            Some("worse".to_string()),
        ]
        .into_sexp();
        let err = <Vec<Option<Uuid>> as TryFromSexp>::try_from_sexp(sexp).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("Vec<Option<Uuid>> conversion failed"),
            "got: {msg}"
        );
        assert!(msg.contains("invalid UUID at index 2"), "got: {msg}");
        assert!(msg.contains("invalid UUID at index 4"), "got: {msg}");
        // NA elements are still allowed as None — they must not appear as errors.
        assert!(!msg.contains("NA at index"), "got: {msg}");
    });
}

#[cfg(feature = "url")]
#[test]
fn url_vec_batches_all_parse_failures() {
    use miniextendr_api::Url;
    r_test_utils::with_r_thread(|| {
        let sexp = vec![
            "https://example.com".to_string(),
            "not a url".to_string(),
            "::also-bad::".to_string(),
        ]
        .into_sexp();
        let err = <Vec<Url> as TryFromSexp>::try_from_sexp(sexp).unwrap_err();
        assert!(matches!(err, SexpError::InvalidValue(_)), "got: {err:?}");
        let msg = err.to_string();
        assert!(msg.contains("Vec<Url> conversion failed"), "got: {msg}");
        assert!(msg.contains("invalid URL at index 1"), "got: {msg}");
        assert!(msg.contains("invalid URL at index 2"), "got: {msg}");
    });
}

#[cfg(feature = "num-bigint")]
#[test]
fn bigint_vec_batches_all_parse_failures() {
    use miniextendr_api::BigInt;
    r_test_utils::with_r_thread(|| {
        let sexp = vec![
            "123".to_string(),
            "xyz".to_string(),
            "456".to_string(),
            "1.5".to_string(),
        ]
        .into_sexp();
        let err = <Vec<BigInt> as TryFromSexp>::try_from_sexp(sexp).unwrap_err();
        assert!(matches!(err, SexpError::InvalidValue(_)), "got: {err:?}");
        let msg = err.to_string();
        assert!(msg.contains("Vec<BigInt> conversion failed"), "got: {msg}");
        assert!(msg.contains("invalid BigInt at index 1"), "got: {msg}");
        assert!(msg.contains("invalid BigInt at index 3"), "got: {msg}");
    });
}

// NA regression: `NA_character_` used to flow through `String`'s NA -> ""
// mapping and silently compile as an empty regex (which matches everything).
// It must now be rejected as a missing value.
#[cfg(feature = "regex")]
#[test]
fn regex_na_rejected_not_silently_empty_pattern() {
    use miniextendr_api::Regex;
    r_test_utils::with_r_thread(|| {
        let na = Option::<String>::None.into_sexp();
        let err = <Regex as TryFromSexp>::try_from_sexp(na).unwrap_err();
        assert!(matches!(err, SexpError::Na(_)), "got: {err:?}");

        let sexp = vec![Some("^a+$".to_string()), None].into_sexp();
        let err = <Vec<Regex> as TryFromSexp>::try_from_sexp(sexp).unwrap_err();
        assert!(
            err.to_string()
                .contains("NA at index 1 not allowed for Vec<Regex>"),
            "got: {err}"
        );
    });
}
