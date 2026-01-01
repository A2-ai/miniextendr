mod r_test_utils;

#[cfg(feature = "num-bigint")]
use miniextendr_api::from_r::TryFromSexp;
#[cfg(feature = "num-bigint")]
use miniextendr_api::into_r::IntoR;
#[cfg(feature = "num-bigint")]
use miniextendr_api::{BigInt, BigUint};

#[cfg(feature = "num-bigint")]
#[test]
fn bigint_roundtrip() {
    r_test_utils::with_r_thread(|| {
        let s = "123456789012345678901234567890";
        let sexp = s.to_string().into_sexp();
        let value: BigInt = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert_eq!(value.to_string(), s);

        let back = value.into_sexp();
        let back_str: String = TryFromSexp::try_from_sexp(back).unwrap();
        assert_eq!(back_str, s);
    });
}

#[cfg(feature = "num-bigint")]
#[test]
fn biguint_vector_with_na() {
    r_test_utils::with_r_thread(|| {
        let sexp = vec![Some("1"), None, Some("42")].into_sexp();
        let values: Vec<Option<BigUint>> = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert_eq!(values.len(), 3);
        assert_eq!(values[0].as_ref().unwrap().to_string(), "1");
        assert!(values[1].is_none());
        assert_eq!(values[2].as_ref().unwrap().to_string(), "42");
    });
}
