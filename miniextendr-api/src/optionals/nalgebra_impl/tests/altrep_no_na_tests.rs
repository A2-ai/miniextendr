use super::super::*;
use crate::altrep_traits::{NA_INTEGER, NA_REAL};

/// `no_na` is R's `ISNAN` hint: `anyNA()` trusts it, so NA and NaN both count.
#[test]
fn dvector_no_na_scans_the_data() {
    let clean = DVector::from_vec(vec![1.0, 2.0]);
    assert_eq!(AltRealData::no_na(&clean), Some(true));
    let na = DVector::from_vec(vec![1.0, NA_REAL]);
    assert_eq!(AltRealData::no_na(&na), Some(false));
    let nan = DVector::from_vec(vec![1.0, f64::NAN]);
    assert_eq!(AltRealData::no_na(&nan), Some(false));
    let ints = DVector::from_vec(vec![1, NA_INTEGER]);
    assert_eq!(AltIntegerData::no_na(&ints), Some(false));
    let clean_ints = DVector::from_vec(vec![1, 2]);
    assert_eq!(AltIntegerData::no_na(&clean_ints), Some(true));
}
