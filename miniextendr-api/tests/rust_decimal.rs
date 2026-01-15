mod r_test_utils;

#[cfg(feature = "rust_decimal")]
use miniextendr_api::Decimal;
#[cfg(feature = "rust_decimal")]
use miniextendr_api::coerce::{Coerce, CoerceError, TryCoerce};
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
        let sexp = vec![Some("1.5".to_string()), None, Some("2.25".to_string())].into_sexp();
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

// =============================================================================
// Coerce/TryCoerce tests for Decimal
// =============================================================================

#[cfg(feature = "rust_decimal")]
#[test]
fn coerce_i32_to_decimal() {
    // i32 -> Decimal: lossless
    let d: Decimal = 42i32.coerce();
    assert_eq!(d.to_string(), "42");

    // Negative values
    let neg: Decimal = (-100i32).coerce();
    assert_eq!(neg.to_string(), "-100");

    // Edge cases
    let max: Decimal = i32::MAX.coerce();
    assert_eq!(max.to_string(), i32::MAX.to_string());

    let min: Decimal = i32::MIN.coerce();
    assert_eq!(min.to_string(), i32::MIN.to_string());
}

#[cfg(feature = "rust_decimal")]
#[test]
fn coerce_i64_to_decimal() {
    // i64 -> Decimal: lossless (Decimal has 28-29 digit precision)
    let d: Decimal = 1234567890123456i64.coerce();
    assert_eq!(d.to_string(), "1234567890123456");

    let neg: Decimal = (-999999999999999i64).coerce();
    assert_eq!(neg.to_string(), "-999999999999999");
}

#[cfg(feature = "rust_decimal")]
#[test]
fn try_coerce_f64_to_decimal_success() {
    // f64 -> Decimal: succeeds for finite values
    let result: Result<Decimal, _> = 123.456f64.try_coerce();
    assert!(result.is_ok());
    // Note: f64 representation may not be exact
    let d = result.unwrap();
    assert!(d.to_string().starts_with("123.456"));

    // Integer values
    let int_result: Result<Decimal, _> = 42.0f64.try_coerce();
    assert!(int_result.is_ok());
    assert_eq!(int_result.unwrap().to_string(), "42");

    // Zero
    let zero_result: Result<Decimal, _> = 0.0f64.try_coerce();
    assert!(zero_result.is_ok());
    assert_eq!(zero_result.unwrap().to_string(), "0");
}

#[cfg(feature = "rust_decimal")]
#[test]
fn try_coerce_f64_to_decimal_nan_error() {
    // f64 -> Decimal: errors on NaN
    let result: Result<Decimal, CoerceError> = f64::NAN.try_coerce();
    assert_eq!(result, Err(CoerceError::NaN));
}

#[cfg(feature = "rust_decimal")]
#[test]
fn try_coerce_f64_to_decimal_inf_error() {
    // f64 -> Decimal: errors on infinity
    let result: Result<Decimal, CoerceError> = f64::INFINITY.try_coerce();
    assert_eq!(result, Err(CoerceError::Overflow));

    let neg_inf_result: Result<Decimal, CoerceError> = f64::NEG_INFINITY.try_coerce();
    assert_eq!(neg_inf_result, Err(CoerceError::Overflow));
}

#[cfg(feature = "rust_decimal")]
#[test]
fn try_coerce_decimal_to_f64() {
    use std::str::FromStr;

    // Decimal -> f64: may lose precision for high-precision decimals
    let d = Decimal::from_str("123.456").unwrap();
    let result: Result<f64, _> = d.try_coerce();
    assert!(result.is_ok());
    assert!((result.unwrap() - 123.456).abs() < 0.0001);

    // Large integer part
    let large = Decimal::from_str("1234567890").unwrap();
    let large_result: Result<f64, _> = large.try_coerce();
    assert!(large_result.is_ok());
    assert_eq!(large_result.unwrap(), 1234567890.0);
}

#[cfg(feature = "rust_decimal")]
#[test]
fn try_coerce_decimal_to_i64() {
    use std::str::FromStr;

    // Decimal -> i64: succeeds for integer values
    let d = Decimal::from_str("12345").unwrap();
    let result: Result<i64, _> = d.try_coerce();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 12345);

    // Negative
    let neg = Decimal::from_str("-999").unwrap();
    let neg_result: Result<i64, _> = neg.try_coerce();
    assert!(neg_result.is_ok());
    assert_eq!(neg_result.unwrap(), -999);
}

#[cfg(feature = "rust_decimal")]
#[test]
fn try_coerce_decimal_to_i64_precision_loss() {
    use std::str::FromStr;

    // Decimal -> i64: errors on fractional part
    let d = Decimal::from_str("123.456").unwrap();
    let result: Result<i64, CoerceError> = d.try_coerce();
    assert_eq!(result, Err(CoerceError::PrecisionLoss));
}

#[cfg(feature = "rust_decimal")]
#[test]
fn try_coerce_decimal_to_i32() {
    use std::str::FromStr;

    // Decimal -> i32: succeeds for small integer values
    let d = Decimal::from_str("42").unwrap();
    let result: Result<i32, _> = d.try_coerce();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);

    // Errors on fractional
    let frac = Decimal::from_str("42.5").unwrap();
    let frac_result: Result<i32, CoerceError> = frac.try_coerce();
    assert_eq!(frac_result, Err(CoerceError::PrecisionLoss));

    // Errors on overflow
    let large = Decimal::from_str("3000000000").unwrap();
    let large_result: Result<i32, CoerceError> = large.try_coerce();
    assert_eq!(large_result, Err(CoerceError::Overflow));
}
