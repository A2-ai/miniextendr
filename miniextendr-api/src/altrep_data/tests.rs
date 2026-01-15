use super::*;
use std::ops::Range;

#[test]
fn logical_roundtrip() {
    assert_eq!(Logical::False.to_r_int(), 0);
    assert_eq!(Logical::True.to_r_int(), 1);
    assert_eq!(Logical::Na.to_r_int(), i32::MIN);

    assert_eq!(Logical::from_r_int(0), Logical::False);
    assert_eq!(Logical::from_r_int(1), Logical::True);
    assert_eq!(Logical::from_r_int(i32::MIN), Logical::Na);
}

#[test]
fn sortedness_roundtrip() {
    let cases = [
        Sortedness::Unknown,
        Sortedness::KnownUnsorted,
        Sortedness::Increasing,
        Sortedness::Decreasing,
        Sortedness::IncreasingNaFirst,
        Sortedness::DecreasingNaFirst,
    ];
    for s in cases {
        let code = s.to_r_int();
        assert_eq!(Sortedness::from_r_int(code), s);
    }
}

#[test]
fn range_i32_no_na_normal() {
    // Normal ranges should not contain NA
    let r: Range<i32> = 1..10;
    assert_eq!(AltIntegerData::no_na(&r), Some(true));

    let r: Range<i32> = -100..100;
    assert_eq!(AltIntegerData::no_na(&r), Some(true));

    let r: Range<i32> = 0..1;
    assert_eq!(AltIntegerData::no_na(&r), Some(true));
}

#[test]
fn range_i32_no_na_at_min() {
    // Range starting at i32::MIN (NA_INTEGER) contains NA
    let r: Range<i32> = i32::MIN..0;
    assert_eq!(AltIntegerData::no_na(&r), Some(false));

    // Range starting at i32::MIN with length 1 also contains NA
    let r: Range<i32> = i32::MIN..(i32::MIN + 1);
    assert_eq!(AltIntegerData::no_na(&r), Some(false));

    // Empty range starting at i32::MIN does NOT contain NA (no elements)
    let r: Range<i32> = i32::MIN..i32::MIN;
    assert_eq!(AltIntegerData::no_na(&r), Some(true));
}

#[test]
fn range_i32_sum_with_na() {
    // Range with NA and na_rm=false should return None
    let r: Range<i32> = i32::MIN..(i32::MIN + 3);
    assert_eq!(AltIntegerData::sum(&r, false), None);

    // Range with NA and na_rm=true should exclude NA
    // Elements are: NA, NA+1, NA+2 -> sum of (NA+1) + (NA+2) = 2*(NA+1.5)
    // But actually: start+1 to end-1 = (i32::MIN+1) to (i32::MIN+2)
    // Sum = 2 * ((i32::MIN+1) + (i32::MIN+2)) / 2 = (i32::MIN+1) + (i32::MIN+2)
    let expected = (i32::MIN as i64 + 1) + (i32::MIN as i64 + 2);
    assert_eq!(AltIntegerData::sum(&r, true), Some(expected));
}

#[test]
fn range_i32_min_with_na() {
    // Range with NA at start
    let r: Range<i32> = i32::MIN..(i32::MIN + 3);

    // min with na_rm=false returns None (propagates NA)
    assert_eq!(AltIntegerData::min(&r, false), None);

    // min with na_rm=true returns second element
    assert_eq!(AltIntegerData::min(&r, true), Some(i32::MIN + 1));
}
