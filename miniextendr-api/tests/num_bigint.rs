mod r_test_utils;

#[cfg(feature = "num-bigint")]
use miniextendr_api::coerce::{Coerce, CoerceError, TryCoerce};
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
        let sexp = vec![Some("1".to_string()), None, Some("42".to_string())].into_sexp();
        let values: Vec<Option<BigUint>> = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert_eq!(values.len(), 3);
        assert_eq!(values[0].as_ref().unwrap().to_string(), "1");
        assert!(values[1].is_none());
        assert_eq!(values[2].as_ref().unwrap().to_string(), "42");
    });
}

// region: Coerce/TryCoerce tests for BigInt/BigUint

#[cfg(feature = "num-bigint")]
#[test]
fn coerce_i32_to_bigint() {
    // i32 -> BigInt: lossless
    let bi: BigInt = 42i32.coerce();
    assert_eq!(bi.to_string(), "42");

    // Negative values
    let neg: BigInt = (-100i32).coerce();
    assert_eq!(neg.to_string(), "-100");

    // Edge cases
    let max: BigInt = i32::MAX.coerce();
    assert_eq!(max.to_string(), i32::MAX.to_string());

    let min: BigInt = i32::MIN.coerce();
    assert_eq!(min.to_string(), i32::MIN.to_string());
}

#[cfg(feature = "num-bigint")]
#[test]
fn coerce_i64_to_bigint() {
    // i64 -> BigInt: lossless
    let bi: BigInt = 1234567890123456789i64.coerce();
    assert_eq!(bi.to_string(), "1234567890123456789");

    // Negative large value
    let neg: BigInt = (-999999999999999999i64).coerce();
    assert_eq!(neg.to_string(), "-999999999999999999");
}

#[cfg(feature = "num-bigint")]
#[test]
fn coerce_u32_to_biguint() {
    // u32 -> BigUint: lossless
    let bu: BigUint = 42u32.coerce();
    assert_eq!(bu.to_string(), "42");

    let max: BigUint = u32::MAX.coerce();
    assert_eq!(max.to_string(), u32::MAX.to_string());
}

#[cfg(feature = "num-bigint")]
#[test]
fn coerce_u64_to_biguint() {
    // u64 -> BigUint: lossless
    let bu: BigUint = 12345678901234567890u64.coerce();
    assert_eq!(bu.to_string(), "12345678901234567890");
}

#[cfg(feature = "num-bigint")]
#[test]
fn try_coerce_f64_to_bigint_success() {
    // f64 -> BigInt: succeeds for integral values
    let result: Result<BigInt, _> = 42.0f64.try_coerce();
    assert!(result.is_ok());
    assert_eq!(result.unwrap().to_string(), "42");

    // Negative integer
    let neg_result: Result<BigInt, _> = (-100.0f64).try_coerce();
    assert!(neg_result.is_ok());
    assert_eq!(neg_result.unwrap().to_string(), "-100");

    // Zero
    let zero_result: Result<BigInt, _> = 0.0f64.try_coerce();
    assert!(zero_result.is_ok());
    assert_eq!(zero_result.unwrap().to_string(), "0");
}

#[cfg(feature = "num-bigint")]
#[test]
fn try_coerce_f64_to_bigint_fractional_error() {
    // f64 -> BigInt: errors on fractional values
    let result: Result<BigInt, CoerceError> = 42.5f64.try_coerce();
    assert_eq!(result, Err(CoerceError::PrecisionLoss));

    let small_frac: Result<BigInt, CoerceError> = 0.1f64.try_coerce();
    assert_eq!(small_frac, Err(CoerceError::PrecisionLoss));
}

#[cfg(feature = "num-bigint")]
#[test]
fn try_coerce_f64_to_bigint_nan_error() {
    // f64 -> BigInt: errors on NaN
    let result: Result<BigInt, CoerceError> = f64::NAN.try_coerce();
    assert_eq!(result, Err(CoerceError::NaN));
}

#[cfg(feature = "num-bigint")]
#[test]
fn try_coerce_f64_to_bigint_inf_error() {
    // f64 -> BigInt: errors on infinity
    let result: Result<BigInt, CoerceError> = f64::INFINITY.try_coerce();
    assert_eq!(result, Err(CoerceError::Overflow));

    let neg_inf: Result<BigInt, CoerceError> = f64::NEG_INFINITY.try_coerce();
    assert_eq!(neg_inf, Err(CoerceError::Overflow));
}

#[cfg(feature = "num-bigint")]
#[test]
fn try_coerce_f64_to_biguint_success() {
    // f64 -> BigUint: succeeds for non-negative integral values
    let result: Result<BigUint, _> = 42.0f64.try_coerce();
    assert!(result.is_ok());
    assert_eq!(result.unwrap().to_string(), "42");

    let zero_result: Result<BigUint, _> = 0.0f64.try_coerce();
    assert!(zero_result.is_ok());
    assert_eq!(zero_result.unwrap().to_string(), "0");
}

#[cfg(feature = "num-bigint")]
#[test]
fn try_coerce_f64_to_biguint_negative_error() {
    // f64 -> BigUint: errors on negative values
    let result: Result<BigUint, CoerceError> = (-1.0f64).try_coerce();
    assert_eq!(result, Err(CoerceError::Overflow));
}

#[cfg(feature = "num-bigint")]
#[test]
fn try_coerce_f64_to_biguint_fractional_error() {
    // f64 -> BigUint: errors on fractional values
    let result: Result<BigUint, CoerceError> = 42.5f64.try_coerce();
    assert_eq!(result, Err(CoerceError::PrecisionLoss));
}

#[cfg(feature = "num-bigint")]
#[test]
fn try_coerce_bigint_to_i32() {
    use std::str::FromStr;

    // BigInt -> i32: succeeds for values in range
    let bi = BigInt::from_str("42").unwrap();
    let result: Result<i32, _> = bi.try_coerce();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);

    // Negative
    let neg = BigInt::from_str("-100").unwrap();
    let neg_result: Result<i32, _> = neg.try_coerce();
    assert!(neg_result.is_ok());
    assert_eq!(neg_result.unwrap(), -100);

    // Overflow - too large
    let large = BigInt::from_str("3000000000").unwrap();
    let large_result: Result<i32, CoerceError> = large.try_coerce();
    assert_eq!(large_result, Err(CoerceError::Overflow));

    // Overflow - too small
    let small = BigInt::from_str("-3000000000").unwrap();
    let small_result: Result<i32, CoerceError> = small.try_coerce();
    assert_eq!(small_result, Err(CoerceError::Overflow));
}

#[cfg(feature = "num-bigint")]
#[test]
fn try_coerce_bigint_to_i64() {
    use std::str::FromStr;

    // BigInt -> i64: succeeds for values in range
    let bi = BigInt::from_str("1234567890123").unwrap();
    let result: Result<i64, _> = bi.try_coerce();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1234567890123i64);

    // Overflow
    let huge = BigInt::from_str("12345678901234567890123456789").unwrap();
    let huge_result: Result<i64, CoerceError> = huge.try_coerce();
    assert_eq!(huge_result, Err(CoerceError::Overflow));
}

#[cfg(feature = "num-bigint")]
#[test]
fn try_coerce_biguint_to_u32() {
    use std::str::FromStr;

    // BigUint -> u32: succeeds for values in range
    let bu = BigUint::from_str("42").unwrap();
    let result: Result<u32, _> = bu.try_coerce();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);

    // Overflow
    let large = BigUint::from_str("5000000000").unwrap();
    let large_result: Result<u32, CoerceError> = large.try_coerce();
    assert_eq!(large_result, Err(CoerceError::Overflow));
}

#[cfg(feature = "num-bigint")]
#[test]
fn try_coerce_bigint_to_f64() {
    use std::str::FromStr;

    // BigInt -> f64: succeeds for values exactly representable
    let bi = BigInt::from_str("12345").unwrap();
    let result: Result<f64, _> = bi.try_coerce();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 12345.0);

    // Large values may lose precision
    let huge = BigInt::from_str("123456789012345678901234567890").unwrap();
    let huge_result: Result<f64, CoerceError> = huge.try_coerce();
    // This may succeed (f64 can represent it, just with precision loss)
    // or fail depending on implementation
    // The current impl uses ToPrimitive which succeeds but loses precision
    assert!(huge_result.is_ok() || huge_result == Err(CoerceError::PrecisionLoss));
}
// endregion
