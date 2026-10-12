//! Iterator and stream ALTREP adaptors read a double or complex element their
//! source never produced as R's `NA`, not as an ordinary `NaN`.

use miniextendr_api::altrep_traits::NA_INTEGER;
use miniextendr_api::{
    AltComplexData, AltIntegerData, AltRealData, IsNA, IterComplexData, IterRealCoerceData,
    IterRealData, Rcomplex, SparseIterComplexData, SparseIterRealData, StreamingIntData,
    StreamingRealData, WindowedIterRealData,
};

/// A reader that writes only the first two elements, 1 and 2.
fn short_reader<T: From<u8>>(start: usize, buf: &mut [T]) -> usize {
    let count = buf.len().min(2usize.saturating_sub(start));
    for (i, slot) in buf[..count].iter_mut().enumerate() {
        *slot = T::from(u8::try_from(start + i + 1).unwrap());
    }
    count
}

/// Reads `1, 2, NA, NA` through `elt` and through `get_region` on a fresh vector.
fn check_short_real<D: AltRealData>(name: &str, make: impl Fn() -> D) {
    let data = make();
    assert_eq!([data.elt(0), data.elt(1)], [1.0, 2.0], "{name}");
    assert!(data.elt(2).is_na() && data.elt(3).is_na(), "{name}");

    let mut buf = [0.0; 4];
    assert_eq!(make().get_region(0, 4, &mut buf), 4, "{name}");
    assert_eq!(buf[..2], [1.0, 2.0], "{name}");
    assert!(buf[2].is_na() && buf[3].is_na(), "{name}: {buf:?}");
}

fn check_short_complex<D: AltComplexData>(name: &str, make: impl Fn() -> D) {
    let is_na = |z: Rcomplex| z.r.is_na() && z.i.is_na();
    let data = make();
    assert_eq!(data.elt(1).r, 2.0, "{name}");
    assert!(is_na(data.elt(2)) && is_na(data.elt(3)), "{name}");

    let zero = Rcomplex { r: 0.0, i: 0.0 };
    let mut buf = [zero; 4];
    assert_eq!(make().get_region(0, 4, &mut buf), 4, "{name}");
    assert!(is_na(buf[2]) && is_na(buf[3]), "{name}");
}

#[test]
fn double_adaptors_read_past_a_short_source_as_na() {
    let values = || [1.0, 2.0].into_iter();
    check_short_real("iter", || IterRealData::from_iter(values(), 4));
    check_short_real("iter_coerce", || {
        IterRealCoerceData::from_iter([1i32, 2].into_iter(), 4)
    });
    check_short_real("windowed", || {
        WindowedIterRealData::from_iter(values(), 4, 4)
    });
    check_short_real("sparse", || SparseIterRealData::from_iter(values(), 4));
    check_short_real("stream", || {
        StreamingRealData::new(4, 4, short_reader::<f64>)
    });
}

#[test]
fn complex_adaptors_read_past_a_short_source_as_na() {
    let values = || {
        (1..=2).map(|x| Rcomplex {
            r: f64::from(x),
            i: f64::from(x),
        })
    };
    check_short_complex("iter", || IterComplexData::from_iter(values(), 4));
    check_short_complex("sparse", || SparseIterComplexData::from_iter(values(), 4));
}

#[test]
fn skipped_sparse_elements_are_na() {
    let real = SparseIterRealData::from_iter((0..4).map(f64::from), 4);
    assert_eq!(real.elt(3), 3.0);
    assert!(real.elt(0).is_na());

    let values = (0..4).map(|i| Rcomplex {
        r: f64::from(i),
        i: -f64::from(i),
    });
    let complex = SparseIterComplexData::from_iter(values, 4);
    assert_eq!(complex.elt(3).r, 3.0);
    let skipped = complex.elt(0);
    assert!(skipped.r.is_na() && skipped.i.is_na());
}

#[test]
fn a_short_integer_stream_fills_its_region_with_na() {
    let data = StreamingIntData::new(4, 4, short_reader::<i32>);
    let mut buf = [0; 4];
    assert_eq!(data.get_region(0, 4, &mut buf), 4);
    assert_eq!(buf, [1, 2, NA_INTEGER, NA_INTEGER]);
}
