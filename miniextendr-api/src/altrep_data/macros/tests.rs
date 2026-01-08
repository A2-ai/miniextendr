use crate::altrep_data::{
    AltComplexData, AltIntegerData, AltLogicalData, AltRawData, AltRealData, AltStringData,
    AltrepLen, IterComplexData, IterIntCoerceData, IterIntData, IterIntFromBoolData,
    IterLogicalData, IterRawData, IterRealCoerceData, IterRealData, IterStringData, Logical,
    Sortedness,
};
use crate::ffi::Rcomplex;

#[test]
pub(crate) fn test_logical_to_r_int() {
    assert_eq!(Logical::False.to_r_int(), 0);
    assert_eq!(Logical::True.to_r_int(), 1);
    assert_eq!(Logical::Na.to_r_int(), i32::MIN);
}

#[test]
pub(crate) fn test_logical_from_r_int() {
    assert_eq!(Logical::from_r_int(0), Logical::False);
    assert_eq!(Logical::from_r_int(1), Logical::True);
    assert_eq!(Logical::from_r_int(42), Logical::True); // Non-zero is TRUE
    assert_eq!(Logical::from_r_int(-1), Logical::True);
    assert_eq!(Logical::from_r_int(i32::MIN), Logical::Na);
}

#[test]
pub(crate) fn test_logical_from_bool() {
    assert_eq!(Logical::from_bool(false), Logical::False);
    assert_eq!(Logical::from_bool(true), Logical::True);
}

#[test]
pub(crate) fn test_sortedness_to_r_int() {
    assert_eq!(Sortedness::Unknown.to_r_int(), i32::MIN);
    assert_eq!(Sortedness::KnownUnsorted.to_r_int(), 0);
    assert_eq!(Sortedness::Increasing.to_r_int(), 1);
    assert_eq!(Sortedness::Decreasing.to_r_int(), -1);
    assert_eq!(Sortedness::IncreasingNaFirst.to_r_int(), 2);
    assert_eq!(Sortedness::DecreasingNaFirst.to_r_int(), -2);
}

#[test]
pub(crate) fn test_sortedness_from_r_int() {
    assert_eq!(Sortedness::from_r_int(i32::MIN), Sortedness::Unknown);
    assert_eq!(Sortedness::from_r_int(0), Sortedness::KnownUnsorted);
    assert_eq!(Sortedness::from_r_int(1), Sortedness::Increasing);
    assert_eq!(Sortedness::from_r_int(-1), Sortedness::Decreasing);
    assert_eq!(Sortedness::from_r_int(2), Sortedness::IncreasingNaFirst);
    assert_eq!(Sortedness::from_r_int(-2), Sortedness::DecreasingNaFirst);
    // Invalid values map to Unknown
    assert_eq!(Sortedness::from_r_int(99), Sortedness::Unknown);
}

#[test]
pub(crate) fn test_vec_i32_len() {
    let v: Vec<i32> = vec![1, 2, 3, 4, 5];
    assert_eq!(AltrepLen::len(&v), 5);
    assert!(!AltrepLen::is_empty(&v));

    let empty: Vec<i32> = vec![];
    assert_eq!(AltrepLen::len(&empty), 0);
    assert!(AltrepLen::is_empty(&empty));
}

#[test]
pub(crate) fn test_vec_i32_elt() {
    let v = vec![10, 20, 30];
    assert_eq!(AltIntegerData::elt(&v, 0), 10);
    assert_eq!(AltIntegerData::elt(&v, 1), 20);
    assert_eq!(AltIntegerData::elt(&v, 2), 30);
}

#[test]
pub(crate) fn test_vec_i32_as_slice() {
    let v = vec![1, 2, 3];
    assert_eq!(AltIntegerData::as_slice(&v), Some(&[1, 2, 3][..]));
}

#[test]
pub(crate) fn test_vec_i32_get_region() {
    let v = vec![10, 20, 30, 40, 50];
    let mut buf = [0i32; 3];

    // Normal region
    let n = AltIntegerData::get_region(&v, 1, 3, &mut buf);
    assert_eq!(n, 3);
    assert_eq!(buf, [20, 30, 40]);

    // Region at end (partial)
    let n = AltIntegerData::get_region(&v, 3, 5, &mut buf);
    assert_eq!(n, 2);
    assert_eq!(buf[..2], [40, 50]);

    // Start beyond length
    let n = AltIntegerData::get_region(&v, 10, 3, &mut buf);
    assert_eq!(n, 0);
}

#[test]
pub(crate) fn test_vec_i32_no_na() {
    let v = vec![1, 2, 3];
    assert_eq!(AltIntegerData::no_na(&v), Some(true));

    let v_with_na = vec![1, i32::MIN, 3]; // i32::MIN is NA
    assert_eq!(AltIntegerData::no_na(&v_with_na), Some(false));
}

#[test]
pub(crate) fn test_vec_i32_sum() {
    let v = vec![1, 2, 3, 4, 5];
    assert_eq!(AltIntegerData::sum(&v, false), Some(15));
    assert_eq!(AltIntegerData::sum(&v, true), Some(15));

    // With NA
    let v_na = vec![1, 2, i32::MIN, 4, 5];
    assert_eq!(AltIntegerData::sum(&v_na, false), None); // NA propagates
    assert_eq!(AltIntegerData::sum(&v_na, true), Some(12)); // na.rm=TRUE
}

#[test]
pub(crate) fn test_vec_i32_min_max() {
    let v = vec![5, 2, 8, 1, 9];
    assert_eq!(AltIntegerData::min(&v, false), Some(1));
    assert_eq!(AltIntegerData::max(&v, false), Some(9));

    // With NA
    let v_na = vec![5, 2, i32::MIN, 1, 9];
    assert_eq!(AltIntegerData::min(&v_na, false), None);
    assert_eq!(AltIntegerData::max(&v_na, false), None);
    assert_eq!(AltIntegerData::min(&v_na, true), Some(1));
    assert_eq!(AltIntegerData::max(&v_na, true), Some(9));
}

#[test]
pub(crate) fn test_vec_f64_sum() {
    let v = vec![1.0, 2.0, 3.0];
    assert_eq!(AltRealData::sum(&v, false), Some(6.0));

    let v_nan = vec![1.0, f64::NAN, 3.0];
    assert!(AltRealData::sum(&v_nan, false).unwrap().is_nan());
    assert_eq!(AltRealData::sum(&v_nan, true), Some(4.0));
}

#[test]
pub(crate) fn test_vec_f64_min_max() {
    let v = vec![3.0, 1.0, 4.0, 1.5];
    assert_eq!(AltRealData::min(&v, false), Some(1.0));
    assert_eq!(AltRealData::max(&v, false), Some(4.0));
}

#[test]
pub(crate) fn test_box_slice_i32() {
    let b: Box<[i32]> = vec![1, 2, 3, 4, 5].into_boxed_slice();
    assert_eq!(AltrepLen::len(&b), 5);
    assert_eq!(AltIntegerData::elt(&b, 2), 3);
    assert_eq!(AltIntegerData::sum(&b, false), Some(15));
    assert_eq!(AltIntegerData::min(&b, false), Some(1));
    assert_eq!(AltIntegerData::max(&b, false), Some(5));
}

#[test]
pub(crate) fn test_box_slice_f64() {
    let b: Box<[f64]> = vec![1.0, 2.0, 3.0].into_boxed_slice();
    assert_eq!(AltrepLen::len(&b), 3);
    assert_eq!(AltRealData::elt(&b, 1), 2.0);
    assert_eq!(AltRealData::sum(&b, false), Some(6.0));
}

#[test]
#[allow(clippy::reversed_empty_ranges)] // Intentionally testing empty range handling
pub(crate) fn test_range_i32_len() {
    let r = 1..10;
    assert_eq!(AltrepLen::len(&r), 9);

    let empty = 10..5;
    assert_eq!(AltrepLen::len(&empty), 0);
}

#[test]
pub(crate) fn test_range_i32_elt() {
    let r = 5..10;
    assert_eq!(AltIntegerData::elt(&r, 0), 5);
    assert_eq!(AltIntegerData::elt(&r, 4), 9);
}

#[test]
pub(crate) fn test_range_i32_sum() {
    // Sum of 1..11 (1 to 10) = 55
    let r = 1..11;
    assert_eq!(AltIntegerData::sum(&r, false), Some(55));

    // Sum of 1..101 (1 to 100) = 5050
    let r = 1..101;
    assert_eq!(AltIntegerData::sum(&r, false), Some(5050));
}

#[test]
pub(crate) fn test_range_i32_min_max() {
    let r = 5..15;
    assert_eq!(AltIntegerData::min(&r, false), Some(5));
    assert_eq!(AltIntegerData::max(&r, false), Some(14)); // end is exclusive
}

#[test]
pub(crate) fn test_range_i32_is_sorted() {
    let r = 1..10;
    assert_eq!(AltIntegerData::is_sorted(&r), Some(Sortedness::Increasing));
}

#[test]
pub(crate) fn test_static_slice_i32() {
    static DATA: [i32; 5] = [10, 20, 30, 40, 50];
    let s: &[i32] = &DATA;

    assert_eq!(AltrepLen::len(&s), 5);
    assert_eq!(AltIntegerData::elt(&s, 0), 10);
    assert_eq!(AltIntegerData::elt(&s, 4), 50);
    assert_eq!(AltIntegerData::sum(&s, false), Some(150));
    assert_eq!(AltIntegerData::min(&s, false), Some(10));
    assert_eq!(AltIntegerData::max(&s, false), Some(50));
}

#[test]
pub(crate) fn test_static_slice_with_na() {
    let s: &[i32] = &[1, 2, i32::MIN, 4];
    assert_eq!(AltIntegerData::no_na(&s), Some(false));
    assert_eq!(AltIntegerData::sum(&s, false), None); // NA propagates
    assert_eq!(AltIntegerData::sum(&s, true), Some(7)); // na.rm=TRUE
}

#[test]
pub(crate) fn test_static_slice_f64() {
    static DATA: [f64; 4] = [1.5, 2.5, 3.5, 4.5];
    let s: &[f64] = &DATA;

    assert_eq!(AltrepLen::len(&s), 4);
    assert_eq!(AltRealData::sum(&s, false), Some(12.0));
    assert_eq!(AltRealData::min(&s, false), Some(1.5));
    assert_eq!(AltRealData::max(&s, false), Some(4.5));
}

#[test]
pub(crate) fn test_array_i32() {
    let arr: [i32; 3] = [100, 200, 300];
    assert_eq!(AltrepLen::len(&arr), 3);
    assert_eq!(AltIntegerData::elt(&arr, 1), 200);
    assert_eq!(AltIntegerData::as_slice(&arr), Some(&[100, 200, 300][..]));
}

#[test]
pub(crate) fn test_array_f64() {
    let arr: [f64; 2] = [1.1, 2.2];
    assert_eq!(AltrepLen::len(&arr), 2);
    assert_eq!(AltRealData::elt(&arr, 0), 1.1);
}

#[test]
pub(crate) fn test_vec_bool_logical() {
    let v = vec![true, false, true, true];
    assert_eq!(AltrepLen::len(&v), 4);
    assert_eq!(AltLogicalData::elt(&v, 0), Logical::True);
    assert_eq!(AltLogicalData::elt(&v, 1), Logical::False);
    assert_eq!(AltLogicalData::no_na(&v), Some(true));
    assert_eq!(AltLogicalData::sum(&v, false), Some(3)); // Count of TRUE
}

#[test]
pub(crate) fn test_vec_string() {
    let v = vec!["hello".to_string(), "world".to_string()];
    assert_eq!(AltrepLen::len(&v), 2);
    assert_eq!(AltStringData::elt(&v, 0), Some("hello"));
    assert_eq!(AltStringData::elt(&v, 1), Some("world"));
    assert_eq!(AltStringData::no_na(&v), Some(true));
}

#[test]
pub(crate) fn test_vec_option_string() {
    let v: Vec<Option<String>> = vec![Some("a".to_string()), None, Some("b".to_string())];
    assert_eq!(AltrepLen::len(&v), 3);
    assert_eq!(AltStringData::elt(&v, 0), Some("a"));
    assert_eq!(AltStringData::elt(&v, 1), None); // NA
    assert_eq!(AltStringData::elt(&v, 2), Some("b"));
    assert_eq!(AltStringData::no_na(&v), Some(false)); // Has NA
}

#[test]
pub(crate) fn test_vec_u8() {
    let v: Vec<u8> = vec![0x01, 0x02, 0xFF];
    assert_eq!(AltrepLen::len(&v), 3);
    assert_eq!(AltRawData::elt(&v, 0), 0x01);
    assert_eq!(AltRawData::elt(&v, 2), 0xFF);
    assert_eq!(AltRawData::as_slice(&v), Some(&[0x01, 0x02, 0xFF][..]));
}

#[test]
pub(crate) fn test_empty_vec() {
    let v: Vec<i32> = vec![];
    assert_eq!(AltrepLen::len(&v), 0);
    assert!(AltrepLen::is_empty(&v));
    assert_eq!(AltIntegerData::sum(&v, false), Some(0));
    assert_eq!(AltIntegerData::min(&v, false), None);
    assert_eq!(AltIntegerData::max(&v, false), None);
}

#[test]
pub(crate) fn test_single_element() {
    let v = vec![42];
    assert_eq!(AltIntegerData::sum(&v, false), Some(42));
    assert_eq!(AltIntegerData::min(&v, false), Some(42));
    assert_eq!(AltIntegerData::max(&v, false), Some(42));
}

#[test]
pub(crate) fn test_large_sum_overflow() {
    // Sum that exceeds i32 range but fits in i64
    let v: Vec<i32> = vec![i32::MAX, i32::MAX];
    let sum = AltIntegerData::sum(&v, false).unwrap();
    assert_eq!(sum, 2 * i32::MAX as i64);
}

#[test]
pub(crate) fn test_iter_int_basic() {
    // Use from_iter with explicit length since Map doesn't preserve ExactSizeIterator
    let iter = (1..=5).map(|x| x * 2);
    let data = IterIntData::from_iter(iter, 5);

    assert_eq!(AltrepLen::len(&data), 5);
    assert_eq!(AltIntegerData::elt(&data, 0), 2);
    assert_eq!(AltIntegerData::elt(&data, 4), 10);

    // Out of bounds
    assert_eq!(
        AltIntegerData::elt(&data, 5),
        crate::altrep_traits::NA_INTEGER
    );
}

#[test]
pub(crate) fn test_iter_int_random_access() {
    let iter = (0..10).map(|x| x * x);
    let data = IterIntData::from_iter(iter, 10);

    // Access in non-sequential order (tests caching)
    assert_eq!(AltIntegerData::elt(&data, 5), 25);
    assert_eq!(AltIntegerData::elt(&data, 2), 4);
    assert_eq!(AltIntegerData::elt(&data, 5), 25); // Cached
    assert_eq!(AltIntegerData::elt(&data, 9), 81);
}

#[test]
pub(crate) fn test_iter_real_basic() {
    let iter = (1..=5).map(|x| x as f64 * 1.5);
    let data = IterRealData::from_iter(iter, 5);

    assert_eq!(AltrepLen::len(&data), 5);
    assert_eq!(AltRealData::elt(&data, 0), 1.5);
    assert_eq!(AltRealData::elt(&data, 4), 7.5);
}

#[test]
pub(crate) fn test_iter_logical_basic() {
    let iter = (0..5).map(|x| x % 2 == 0);
    let data = IterLogicalData::from_iter(iter, 5);

    assert_eq!(AltrepLen::len(&data), 5);
    assert_eq!(AltLogicalData::elt(&data, 0), Logical::True);
    assert_eq!(AltLogicalData::elt(&data, 1), Logical::False);
    assert_eq!(AltLogicalData::elt(&data, 2), Logical::True);
}

#[test]
pub(crate) fn test_iter_raw_basic() {
    let iter = (0..5u8).map(|x| x * 10);
    let data = IterRawData::from_iter(iter, 5);

    assert_eq!(AltrepLen::len(&data), 5);
    assert_eq!(AltRawData::elt(&data, 0), 0);
    assert_eq!(AltRawData::elt(&data, 2), 20);
    assert_eq!(AltRawData::elt(&data, 4), 40);
}

#[test]
pub(crate) fn test_iter_get_region() {
    let iter = (1..=10).map(|x| x * 10);
    let data = IterIntData::from_iter(iter, 10);

    let mut buf = [0i32; 5];
    let n = AltIntegerData::get_region(&data, 2, 5, &mut buf);

    assert_eq!(n, 5);
    assert_eq!(buf, [30, 40, 50, 60, 70]);
}

#[test]
pub(crate) fn test_iter_state_materialization() {
    let iter = (1..=3).map(|x| x * 2);
    let data = IterIntData::from_iter(iter, 3);

    // Access elements to force caching
    assert_eq!(AltIntegerData::elt(&data, 0), 2);
    assert_eq!(AltIntegerData::elt(&data, 2), 6);
    // as_slice should expose the materialized backing vector
    assert_eq!(data.as_slice(), Some(&[2, 4, 6][..]));
}

#[test]
pub(crate) fn test_iter_explicit_length() {
    // Create with explicit length (not ExactSizeIterator)
    let iter = vec![10, 20, 30].into_iter();
    let data = IterIntData::from_iter(iter, 3);

    assert_eq!(AltrepLen::len(&data), 3);
    assert_eq!(AltIntegerData::elt(&data, 1), 20);
}

#[test]
pub(crate) fn test_iter_int_coerce_u16() {
    // Iterator of u16 values coerced to i32
    let iter = (0..5u16).map(|x| x * 1000);
    let data = IterIntCoerceData::from_iter(iter, 5);

    assert_eq!(AltrepLen::len(&data), 5);
    assert_eq!(AltIntegerData::elt(&data, 0), 0);
    assert_eq!(AltIntegerData::elt(&data, 2), 2000);
    assert_eq!(AltIntegerData::elt(&data, 4), 4000);
}

#[test]
pub(crate) fn test_iter_int_coerce_i8() {
    // Iterator of i8 values coerced to i32
    let iter = -5i8..5i8;
    let data = IterIntCoerceData::from_exact_iter(iter);

    assert_eq!(AltrepLen::len(&data), 10);
    assert_eq!(AltIntegerData::elt(&data, 0), -5);
    assert_eq!(AltIntegerData::elt(&data, 5), 0);
    assert_eq!(AltIntegerData::elt(&data, 9), 4);
}

#[test]
pub(crate) fn test_iter_real_coerce_f32() {
    // Iterator of f32 values coerced to f64
    let iter = (0..5).map(|x| x as f32 * 1.5);
    let data = IterRealCoerceData::from_iter(iter, 5);

    assert_eq!(AltrepLen::len(&data), 5);
    assert!((AltRealData::elt(&data, 0) - 0.0).abs() < 0.001);
    assert!((AltRealData::elt(&data, 2) - 3.0).abs() < 0.001);
    assert!((AltRealData::elt(&data, 4) - 6.0).abs() < 0.001);
}

#[test]
pub(crate) fn test_iter_real_coerce_i32() {
    // Iterator of i32 values coerced to f64
    let iter = 1..=5;
    let data = IterRealCoerceData::from_iter(iter, 5);

    assert_eq!(AltrepLen::len(&data), 5);
    assert_eq!(AltRealData::elt(&data, 0), 1.0);
    assert_eq!(AltRealData::elt(&data, 4), 5.0);
}

#[test]
pub(crate) fn test_iter_int_from_bool() {
    // Iterator of bool values coerced to i32
    let iter = (0..10).map(|x| x % 3 == 0);
    let data = IterIntFromBoolData::from_iter(iter, 10);

    assert_eq!(AltrepLen::len(&data), 10);
    assert_eq!(AltIntegerData::elt(&data, 0), 1); // TRUE
    assert_eq!(AltIntegerData::elt(&data, 1), 0); // FALSE
    assert_eq!(AltIntegerData::elt(&data, 3), 1); // TRUE
}

#[test]
pub(crate) fn test_iter_coerce_get_region() {
    // Test get_region with coerced types
    let iter = (0..10u16).map(|x| x * 10);
    let data = IterIntCoerceData::from_iter(iter, 10);

    let mut buf = [0i32; 5];
    let n = AltIntegerData::get_region(&data, 3, 5, &mut buf);

    assert_eq!(n, 5);
    assert_eq!(buf, [30, 40, 50, 60, 70]);
}

#[test]
pub(crate) fn test_iter_real_coerce_option() {
    // Iterator of Option<f64> coerced to f64 with None → NA (NaN)
    let iter = (0..5).map(|x| if x % 2 == 0 { Some(x as f64) } else { None });
    let data = IterRealCoerceData::from_iter(iter, 5);

    assert_eq!(AltrepLen::len(&data), 5);
    assert_eq!(AltRealData::elt(&data, 0), 0.0); // Some(0.0)
    assert!(AltRealData::elt(&data, 1).is_nan()); // None → NaN
    assert_eq!(AltRealData::elt(&data, 2), 2.0); // Some(2.0)
    assert!(AltRealData::elt(&data, 3).is_nan()); // None → NaN
    assert_eq!(AltRealData::elt(&data, 4), 4.0); // Some(4.0)
}

#[test]
pub(crate) fn test_iter_int_coerce_option() {
    // Iterator of Option<i32> coerced to i32 with None → NA (i32::MIN)
    let iter = (0..5).map(|x| if x % 2 == 0 { Some(x) } else { None });
    let data = IterIntCoerceData::from_iter(iter, 5);

    assert_eq!(AltrepLen::len(&data), 5);
    assert_eq!(AltIntegerData::elt(&data, 0), 0); // Some(0)
    assert_eq!(AltIntegerData::elt(&data, 1), i32::MIN); // None → NA
    assert_eq!(AltIntegerData::elt(&data, 2), 2); // Some(2)
    assert_eq!(AltIntegerData::elt(&data, 3), i32::MIN); // None → NA
    assert_eq!(AltIntegerData::elt(&data, 4), 4); // Some(4)
}

#[test]
pub(crate) fn test_iter_string_basic() {
    let iter = (0..3).map(|x| format!("item_{}", x));
    let data = IterStringData::from_iter(iter, 3);

    assert_eq!(AltrepLen::len(&data), 3);
    assert_eq!(AltStringData::elt(&data, 0), Some("item_0"));
    assert_eq!(AltStringData::elt(&data, 1), Some("item_1"));
    assert_eq!(AltStringData::elt(&data, 2), Some("item_2"));
}

#[test]
pub(crate) fn test_iter_complex_basic() {
    let iter = (0..5).map(|x| Rcomplex {
        r: x as f64,
        i: (x * 2) as f64,
    });
    let data = IterComplexData::from_iter(iter, 5);

    assert_eq!(AltrepLen::len(&data), 5);

    let z0 = AltComplexData::elt(&data, 0);
    assert_eq!(z0.r, 0.0);
    assert_eq!(z0.i, 0.0);

    let z2 = AltComplexData::elt(&data, 2);
    assert_eq!(z2.r, 2.0);
    assert_eq!(z2.i, 4.0);
}
