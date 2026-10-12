//! Sparse iterator ALTREP adaptors: the values a vector reads once its source ends.

use miniextendr_api::altrep_traits::NA_INTEGER;
use miniextendr_api::{
    AltComplexData, AltIntegerData, AltRealData, IsNA, Rcomplex, SparseIterComplexData,
    SparseIterIntData, SparseIterRealData,
};

/// Yields `1..=produced`, then `None` once, then `100, 101, ...` forever.
fn resuming(produced: i32) -> impl Iterator<Item = i32> {
    let mut next = 1;
    let mut ended = false;
    std::iter::from_fn(move || {
        if next > produced && !ended {
            ended = true;
            next = 100;
            return None;
        }
        next += 1;
        Some(next - 1)
    })
}

#[test]
fn an_index_that_read_as_missing_stays_missing_when_the_source_resumes() {
    let data = SparseIterIntData::from_iter(resuming(1), 4);
    assert_eq!(data.elt(0), 1);
    assert_eq!(data.elt(2), NA_INTEGER); // The source ends while skipping index 1.
    assert_eq!(data.elt(1), NA_INTEGER);
    assert_eq!(data.elt(2), NA_INTEGER);
    assert_eq!(data.elt(3), NA_INTEGER);
    assert_eq!(data.elt(0), 1);
}

#[test]
fn a_region_read_stops_at_the_first_none() {
    let data = SparseIterIntData::from_iter(resuming(2), 5);
    let mut buf = [0; 5];
    assert_eq!(data.get_region(0, 5, &mut buf), 5);
    assert_eq!(buf, [1, 2, NA_INTEGER, NA_INTEGER, NA_INTEGER]);
}

#[test]
fn reads_past_a_short_source_are_missing() {
    let data = SparseIterIntData::from_iter(1..=3, 6);
    assert_eq!(data.elt(1), 2);
    assert_eq!(data.elt(4), NA_INTEGER); // The skip consumes 3, then the source ends.
    assert_eq!(data.elt(4), NA_INTEGER);
    assert_eq!(data.elt(2), NA_INTEGER);
    assert_eq!(data.elt(5), NA_INTEGER);
    assert_eq!(data.elt(1), 2);
}

#[test]
fn skipped_real_and_complex_elements_are_ordinary_nan() {
    let real = SparseIterRealData::from_iter((0..4).map(f64::from), 4);
    assert_eq!(real.elt(3), 3.0);
    let skipped = real.elt(0);
    assert!(skipped.is_nan() && !skipped.is_na());

    let values = (0..4).map(|i| Rcomplex {
        r: f64::from(i),
        i: -f64::from(i),
    });
    let complex = SparseIterComplexData::from_iter(values, 4);
    assert_eq!(complex.elt(3).r, 3.0);
    let skipped = complex.elt(0);
    for part in [skipped.r, skipped.i] {
        assert!(part.is_nan() && !part.is_na());
    }
}
