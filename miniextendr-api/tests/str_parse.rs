//! Tests for the `try_from_sexp_via_str_parse!` string-parse conversions
//! (uuid / url / regex / num-bigint), including the NA-policy standardization:
//! scalar NA errors (`SexpError::Na`), `Option` NA maps to `None`, `Vec<T>`
//! rejects NA with an indexed error, `Vec<Option<T>>` maps NA to `None`.

mod r_test_utils;

#[cfg(any(feature = "uuid", feature = "regex"))]
use miniextendr_api::from_r::{SexpError, TryFromSexp};
#[cfg(any(feature = "uuid", feature = "regex"))]
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
