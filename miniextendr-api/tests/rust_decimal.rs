mod r_test_utils;

#[cfg(feature = "rust_decimal")]
use miniextendr_api::Decimal;
#[cfg(feature = "rust_decimal")]
use miniextendr_api::from_r::TryFromSexp;
#[cfg(feature = "rust_decimal")]
use miniextendr_api::into_r::IntoR;

#[cfg(feature = "rust_decimal")]
#[test]
fn decimal_roundtrip() {
    r_test_utils::with_r_thread(|| {
        let s = "12345.6789";
        let sexp = s.to_string().into_sexp();
        let value: Decimal = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert_eq!(value.to_string(), s);

        let back = value.into_sexp();
        let back_str: String = TryFromSexp::try_from_sexp(back).unwrap();
        assert_eq!(back_str, s);
    });
}

#[cfg(feature = "rust_decimal")]
#[test]
fn decimal_vector_with_na() {
    r_test_utils::with_r_thread(|| {
        let sexp = vec![Some("1.5"), None, Some("2.25")].into_sexp();
        let values: Vec<Option<Decimal>> = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert_eq!(values.len(), 3);
        assert_eq!(values[0].as_ref().unwrap().to_string(), "1.5");
        assert!(values[1].is_none());
        assert_eq!(values[2].as_ref().unwrap().to_string(), "2.25");
    });
}
