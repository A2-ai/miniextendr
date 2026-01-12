mod r_test_utils;

#[cfg(feature = "ordered-float")]
use miniextendr_api::OrderedFloat;
#[cfg(feature = "ordered-float")]
use miniextendr_api::coerce::{Coerce, CoerceError, TryCoerce};
#[cfg(feature = "ordered-float")]
use miniextendr_api::from_r::TryFromSexp;
#[cfg(feature = "ordered-float")]
use miniextendr_api::into_r::IntoR;

#[cfg(feature = "ordered-float")]
#[test]
fn ordered_float_scalar_roundtrip() {
    r_test_utils::with_r_thread(|| {
        let sexp = 1.25f64.into_sexp();
        let value: OrderedFloat<f64> = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert_eq!(value.0, 1.25);

        let back = value.into_sexp();
        let back_value: f64 = TryFromSexp::try_from_sexp(back).unwrap();
        assert_eq!(back_value, 1.25);
    });
}

#[cfg(feature = "ordered-float")]
#[test]
fn ordered_float_vector_and_option() {
    r_test_utils::with_r_thread(|| {
        let sexp = vec![1.0f64, 2.5f64, -3.0f64].into_sexp();
        let values: Vec<OrderedFloat<f64>> = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert_eq!(values.len(), 3);
        assert_eq!(values[1].0, 2.5);

        let opt_sexp = Option::<f64>::None.into_sexp();
        let opt: Option<OrderedFloat<f64>> = TryFromSexp::try_from_sexp(opt_sexp).unwrap();
        assert!(opt.is_none());
    });
}

// =============================================================================
// Coerce/TryCoerce tests for OrderedFloat
// =============================================================================

#[cfg(feature = "ordered-float")]
#[test]
fn coerce_f64_to_ordered_float_f64() {
    // f64 -> OrderedFloat<f64>: infallible wrapping
    let of: OrderedFloat<f64> = 3.14f64.coerce();
    assert_eq!(of.0, 3.14);

    // NaN is allowed
    let nan: OrderedFloat<f64> = f64::NAN.coerce();
    assert!(nan.0.is_nan());

    // Infinity is allowed
    let inf: OrderedFloat<f64> = f64::INFINITY.coerce();
    assert!(inf.0.is_infinite());
}

#[cfg(feature = "ordered-float")]
#[test]
fn coerce_f32_to_ordered_float_f32() {
    // f32 -> OrderedFloat<f32>: infallible wrapping
    let of: OrderedFloat<f32> = 2.5f32.coerce();
    assert_eq!(of.0, 2.5);
}

#[cfg(feature = "ordered-float")]
#[test]
fn try_coerce_f64_to_ordered_float_f32_success() {
    // f64 -> OrderedFloat<f32>: narrowing that fits exactly
    let result: Result<OrderedFloat<f32>, _> = 1.5f64.try_coerce();
    assert!(result.is_ok());
    assert_eq!(result.unwrap().0, 1.5f32);

    // Small integer values should work
    let int_result: Result<OrderedFloat<f32>, _> = 42.0f64.try_coerce();
    assert!(int_result.is_ok());
    assert_eq!(int_result.unwrap().0, 42.0f32);
}

#[cfg(feature = "ordered-float")]
#[test]
fn try_coerce_f64_to_ordered_float_f32_precision_loss() {
    // f64 -> OrderedFloat<f32>: errors on precision loss
    // This value can't be exactly represented in f32
    let precise_f64 = 1.0000001f64;
    let result: Result<OrderedFloat<f32>, CoerceError> = precise_f64.try_coerce();
    assert_eq!(result, Err(CoerceError::PrecisionLoss));

    // Very large f64 that exceeds f32 precision
    let large = 1e30f64;
    let large_result: Result<OrderedFloat<f32>, CoerceError> = large.try_coerce();
    // This may overflow or lose precision depending on value
    // (1e30 fits in f32 range but may lose precision)
    assert!(large_result.is_err());
}

#[cfg(feature = "ordered-float")]
#[test]
fn try_coerce_f64_to_ordered_float_f32_overflow() {
    // f64 -> OrderedFloat<f32>: errors on overflow (beyond f32 range)
    let too_large = f64::MAX;
    let result: Result<OrderedFloat<f32>, CoerceError> = too_large.try_coerce();
    assert_eq!(result, Err(CoerceError::Overflow));

    let too_small = -f64::MAX;
    let result2: Result<OrderedFloat<f32>, CoerceError> = too_small.try_coerce();
    assert_eq!(result2, Err(CoerceError::Overflow));
}

#[cfg(feature = "ordered-float")]
#[test]
fn try_coerce_f64_to_ordered_float_f32_special_values() {
    // NaN narrows to NaN
    let nan_result: Result<OrderedFloat<f32>, _> = f64::NAN.try_coerce();
    assert!(nan_result.is_ok());
    assert!(nan_result.unwrap().0.is_nan());

    // Infinity narrows to infinity
    let inf_result: Result<OrderedFloat<f32>, _> = f64::INFINITY.try_coerce();
    assert!(inf_result.is_ok());
    assert!(inf_result.unwrap().0.is_infinite());

    let neg_inf_result: Result<OrderedFloat<f32>, _> = f64::NEG_INFINITY.try_coerce();
    assert!(neg_inf_result.is_ok());
    assert!(neg_inf_result.unwrap().0.is_infinite());
    assert!(neg_inf_result.unwrap().0.is_sign_negative());
}

#[cfg(feature = "ordered-float")]
#[test]
fn coerce_i32_to_ordered_float_f64() {
    // i32 -> OrderedFloat<f64>: widening
    let of: OrderedFloat<f64> = 42i32.coerce();
    assert_eq!(of.0, 42.0);

    // Negative values
    let neg: OrderedFloat<f64> = (-100i32).coerce();
    assert_eq!(neg.0, -100.0);

    // Edge values
    let max: OrderedFloat<f64> = i32::MAX.coerce();
    assert_eq!(max.0, i32::MAX as f64);
}

#[cfg(feature = "ordered-float")]
#[test]
fn try_coerce_i32_to_ordered_float_f32() {
    // Small i32 values fit in f32
    let result: Result<OrderedFloat<f32>, _> = 1000i32.try_coerce();
    assert!(result.is_ok());
    assert_eq!(result.unwrap().0, 1000.0f32);

    // Large i32 values may lose precision in f32
    // f32 can only exactly represent integers up to 2^24 = 16777216
    let large = 16777217i32; // One beyond f32's exact integer range
    let large_result: Result<OrderedFloat<f32>, CoerceError> = large.try_coerce();
    assert_eq!(large_result, Err(CoerceError::PrecisionLoss));
}

#[cfg(feature = "ordered-float")]
#[test]
fn coerce_ordered_float_to_f64() {
    // OrderedFloat<f64> -> f64: unwrapping
    let of = OrderedFloat(3.14f64);
    let f: f64 = of.coerce();
    assert_eq!(f, 3.14);

    // OrderedFloat<f32> -> f64: widening unwrap
    let of32 = OrderedFloat(2.5f32);
    let f2: f64 = of32.coerce();
    assert_eq!(f2, 2.5);
}
