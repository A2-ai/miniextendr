use super::*;

/// `NA_real_ * 1` as R computes it: the NaN is quieted, the low word stays 1954.
const COMPUTED_NA: f64 = f64::from_bits(0x7FF8_0000_0000_07A2);

#[test]
fn f64_null_buffer_follows_r_is_na() {
    let nulls = build_null_buffer(&[1.0, COMPUTED_NA, f64::NAN, NA_REAL])
        .expect("the NAs need a null buffer");
    // Only the two NAs are null; the plain NaN stays a valid value.
    assert_eq!(nulls.null_count(), 2);
    assert!(nulls.is_valid(0) && nulls.is_null(1) && nulls.is_valid(2) && nulls.is_null(3));
    assert!(build_null_buffer(&[1.0, f64::NAN]).is_none());
}

#[test]
fn float64_array_no_na_counts_nan() {
    let clean = Float64Array::from(vec![1.0, 2.0]);
    assert_eq!(AltRealData::no_na(&clean), Some(true));
    let with_nan = Float64Array::from(vec![1.0, f64::NAN]);
    assert_eq!(AltRealData::no_na(&with_nan), Some(false));
    let with_null = Float64Array::from(vec![Some(1.0), None]);
    assert_eq!(AltRealData::no_na(&with_null), Some(false));
}
