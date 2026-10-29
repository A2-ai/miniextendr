use super::*;
use crate::altrep_data::AltRealData;
use crate::altrep_traits::NA_REAL;

/// `NA_real_ * 1` as R computes it: arithmetic quiets the NaN, so the
/// high word differs from `NA_REAL`'s while the low word stays 1954.
const COMPUTED_NA: f64 = f64::from_bits(0x7FF8_0000_0000_07A2);

// region: NA_REAL bit pattern tests

/// Verify NA_REAL has the expected R bit pattern.
#[test]
fn na_real_bit_pattern() {
    assert_eq!(NA_REAL.to_bits(), 0x7FF0_0000_0000_07A2);
}

/// Regular NaN is NOT the same bit pattern as NA_real_.
#[test]
fn nan_is_not_na_real() {
    let nan = f64::NAN;
    assert!(nan.is_nan());
    assert_ne!(nan.to_bits(), NA_REAL.to_bits());
}

// endregion

// region: Vec<f64> no_na — NA vs NaN

/// A vector holding a regular NaN reports no_na = Some(false): R's
/// `anyNA()` trusts the hint and counts NaN as missing (`ISNAN`).
#[test]
fn vec_f64_no_na_with_regular_nan_is_false() {
    let v: Vec<f64> = vec![1.0, f64::NAN, 3.0];
    assert_eq!(AltRealData::no_na(&v), Some(false));
}

/// A computed NA (`NA_real_ * 1` quiets the NaN) is still NA.
#[test]
fn vec_f64_no_na_with_computed_na_is_false() {
    let v: Vec<f64> = vec![1.0, COMPUTED_NA, 3.0];
    assert_eq!(AltRealData::no_na(&v), Some(false));
}

/// A vector containing NA_real_ should report no_na = Some(false).
#[test]
fn vec_f64_no_na_with_na_real_is_false() {
    let v: Vec<f64> = vec![1.0, NA_REAL, 3.0];
    assert_eq!(AltRealData::no_na(&v), Some(false));
}

/// A vector with no NaN and no NA should report no_na = Some(true).
#[test]
fn vec_f64_no_na_all_finite_is_true() {
    let v: Vec<f64> = vec![1.0, 2.0, 3.0];
    assert_eq!(AltRealData::no_na(&v), Some(true));
}

// endregion

// region: Vec<f64> sum — NA vs NaN

/// sum with na_rm=false and NA_real_ present should return Some(NA_REAL).
#[test]
fn vec_f64_sum_na_propagates() {
    let v: Vec<f64> = vec![1.0, NA_REAL, 3.0];
    let result = AltRealData::sum(&v, false);
    assert!(result.is_some());
    let bits = result.unwrap().to_bits();
    assert_eq!(
        bits,
        NA_REAL.to_bits(),
        "sum with NA should return NA_real_"
    );
}

/// sum with na_rm=true and NA_real_ present should skip the NA and sum the rest.
#[test]
fn vec_f64_sum_na_rm_skips_na() {
    let v: Vec<f64> = vec![1.0, NA_REAL, 3.0];
    let result = AltRealData::sum(&v, true);
    assert_eq!(result, Some(4.0));
}

/// sum with na_rm=false and regular NaN present should return Some(NaN) — NOT NA.
#[test]
fn vec_f64_sum_nan_propagates_as_nan_not_na() {
    let v: Vec<f64> = vec![1.0, f64::NAN, 3.0];
    let result = AltRealData::sum(&v, false);
    assert!(result.is_some());
    let val = result.unwrap();
    assert!(val.is_nan(), "sum with NaN (not NA) should return NaN");
    assert_ne!(
        val.to_bits(),
        NA_REAL.to_bits(),
        "sum with regular NaN should NOT return NA_real_"
    );
}

/// sum with na_rm=true and regular NaN present should skip the NaN.
#[test]
fn vec_f64_sum_nan_rm_skips_nan() {
    let v: Vec<f64> = vec![1.0, f64::NAN, 3.0];
    let result = AltRealData::sum(&v, true);
    assert_eq!(result, Some(4.0));
}

/// A computed NA propagates as NA (not NaN) and is skipped by na_rm.
#[test]
fn vec_f64_sum_computed_na() {
    let v: Vec<f64> = vec![1.0, COMPUTED_NA, 3.0];
    let na = AltRealData::sum(&v, false).expect("sum is always computed");
    assert!(na.is_na(), "{:#x}", na.to_bits());
    assert_eq!(AltRealData::sum(&v, true), Some(4.0));
}

/// NA wins over NaN whatever the order, as in R's `rmin` / `rmax`.
#[test]
fn vec_f64_min_max_na_trumps_nan() {
    for v in [
        vec![f64::NAN, 1.0, COMPUTED_NA],
        vec![COMPUTED_NA, f64::NAN, 1.0],
        vec![1.0, f64::NAN, NA_REAL],
    ] {
        for extreme in [
            AltRealData::min(&v, false),
            AltRealData::max(&v, false),
            AltRealData::sum(&v, false),
        ] {
            let value = extreme.expect("missing values give a result");
            assert!(value.is_na(), "{v:?} gave {:#x}", value.to_bits());
        }
        assert_eq!(AltRealData::min(&v, true), Some(1.0));
        assert_eq!(AltRealData::max(&v, true), Some(1.0));
    }
    let nan_only = vec![3.0, f64::NAN, 1.0];
    let min = AltRealData::min(&nan_only, false).expect("NaN gives a result");
    assert!(min.is_nan() && !min.is_na());
    assert_eq!(AltRealData::min(&nan_only, true), Some(1.0));
    assert_eq!(AltRealData::max(&nan_only, true), Some(3.0));
    let all_missing = vec![NA_REAL, f64::NAN];
    assert_eq!(AltRealData::min(&all_missing, true), None);
}

// endregion

// region: Box<[f64]> no_na — NA vs NaN

#[test]
fn box_f64_no_na_with_regular_nan_is_false() {
    let v: Box<[f64]> = vec![1.0, f64::NAN, 3.0].into_boxed_slice();
    assert_eq!(AltRealData::no_na(&v), Some(false));
}

#[test]
fn box_f64_no_na_with_na_real_is_false() {
    let v: Box<[f64]> = vec![1.0, NA_REAL, 3.0].into_boxed_slice();
    assert_eq!(AltRealData::no_na(&v), Some(false));
}

// endregion

// region: &[f64] no_na — NA vs NaN

#[test]
fn slice_f64_no_na_with_regular_nan_is_false() {
    let data: &[f64] = &[1.0, f64::NAN, 3.0];
    assert_eq!(AltRealData::no_na(&data), Some(false));
}

#[test]
fn slice_f64_no_na_with_na_real_is_false() {
    let data: &[f64] = &[1.0, NA_REAL, 3.0];
    assert_eq!(AltRealData::no_na(&data), Some(false));
}

// endregion

// region: [f64; N] no_na — NA vs NaN

#[test]
fn array_f64_no_na_with_regular_nan_is_false() {
    let arr: [f64; 3] = [1.0, f64::NAN, 3.0];
    assert_eq!(AltRealData::no_na(&arr), Some(false));
}

#[test]
fn array_f64_no_na_with_na_real_is_false() {
    let arr: [f64; 3] = [1.0, NA_REAL, 3.0];
    assert_eq!(AltRealData::no_na(&arr), Some(false));
}

// endregion

// region: Cow<'static, [f64]> no_na — NA vs NaN

#[test]
fn cow_f64_no_na_with_regular_nan_is_false() {
    let v: Cow<'static, [f64]> = Cow::Owned(vec![1.0, f64::NAN, 3.0]);
    assert_eq!(AltRealData::no_na(&v), Some(false));
}

#[test]
fn cow_f64_no_na_with_na_real_is_false() {
    let v: Cow<'static, [f64]> = Cow::Owned(vec![1.0, NA_REAL, 3.0]);
    assert_eq!(AltRealData::no_na(&v), Some(false));
}

// endregion
