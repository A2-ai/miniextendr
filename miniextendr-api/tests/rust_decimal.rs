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
        let sexp = vec![
            Some("1.5".to_string()),
            None,
            Some("2.25".to_string()),
        ]
        .into_sexp();
        let values: Vec<Option<Decimal>> = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert_eq!(values.len(), 3);
        assert_eq!(values[0].as_ref().unwrap().to_string(), "1.5");
        assert!(values[1].is_none());
        assert_eq!(values[2].as_ref().unwrap().to_string(), "2.25");
    });
}

#[cfg(feature = "rust_decimal")]
#[test]
fn decimal_from_numeric() {
    r_test_utils::with_r_thread(|| {
        // Test numeric (f64) fast path
        let sexp = 123.456f64.into_sexp();
        let value: Decimal = TryFromSexp::try_from_sexp(sexp).unwrap();
        // f64 -> Decimal conversion
        assert!(value.to_string().starts_with("123.456"));
    });
}

#[cfg(feature = "rust_decimal")]
#[test]
fn decimal_from_integer() {
    r_test_utils::with_r_thread(|| {
        // Test integer (i32) path - lossless for i32 range
        let sexp = 42i32.into_sexp();
        let value: Decimal = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert_eq!(value.to_string(), "42");
    });
}

#[cfg(feature = "rust_decimal")]
#[test]
fn decimal_vector_from_numeric() {
    r_test_utils::with_r_thread(|| {
        let sexp = vec![1.5f64, 2.25, 3.125].into_sexp();
        let values: Vec<Decimal> = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert_eq!(values.len(), 3);
        assert!(values[0].to_string().starts_with("1.5"));
        assert!(values[1].to_string().starts_with("2.25"));
        assert!(values[2].to_string().starts_with("3.125"));
    });
}

#[cfg(feature = "rust_decimal")]
#[test]
fn decimal_vector_from_integer() {
    r_test_utils::with_r_thread(|| {
        let sexp = vec![1i32, 2, 3].into_sexp();
        let values: Vec<Decimal> = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert_eq!(values.len(), 3);
        assert_eq!(values[0].to_string(), "1");
        assert_eq!(values[1].to_string(), "2");
        assert_eq!(values[2].to_string(), "3");
    });
}

#[cfg(feature = "rust_decimal")]
#[test]
fn decimal_option_from_numeric_na() {
    r_test_utils::with_r_thread(|| {
        let sexp = vec![Some(1.5f64), None, Some(2.5)].into_sexp();
        let values: Vec<Option<Decimal>> = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert_eq!(values.len(), 3);
        assert!(values[0].is_some());
        assert!(values[1].is_none());
        assert!(values[2].is_some());
    });
}
