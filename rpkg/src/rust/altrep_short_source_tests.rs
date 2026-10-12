//! Double and complex iterator/stream ALTREP fixtures whose source ends early.
//!
//! Each vector declares 4 elements, but its source yields only the first two
//! (`1, 2`, or `1+1i, 2+2i`), so the adaptor fills elements 3 and 4 with NA.

use miniextendr_api::Rcomplex;
use miniextendr_api::altrep_data::{
    IterComplexData, IterRealCoerceData, IterRealData, SparseIterComplexData, SparseIterRealData,
    StreamingRealData, WindowedIterRealData,
};
use miniextendr_api::prelude::*;

const DECLARED: usize = 4;

type RealSource = std::array::IntoIter<f64, 2>;
type ComplexSource = std::array::IntoIter<Rcomplex, 2>;

fn real_source() -> RealSource {
    [1.0, 2.0].into_iter()
}

fn complex_source() -> ComplexSource {
    [Rcomplex { r: 1.0, i: 1.0 }, Rcomplex { r: 2.0, i: 2.0 }].into_iter()
}

/// Writes only the first two elements, 1 and 2.
fn short_reader(start: usize, buf: &mut [f64]) -> usize {
    let count = buf.len().min(2usize.saturating_sub(start));
    for (i, slot) in buf[..count].iter_mut().enumerate() {
        *slot = f64::from(u8::try_from(start + i + 1).unwrap());
    }
    count
}

// region: Double wrappers

#[derive(miniextendr_api::AltrepReal)]
#[altrep(class = "ShortIterReal", manual)]
pub struct ShortIterReal {
    inner: IterRealData<RealSource>,
}

impl AltrepLen for ShortIterReal {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl AltRealData for ShortIterReal {
    fn elt(&self, i: usize) -> f64 {
        self.inner.elt(i)
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [f64]) -> usize {
        self.inner.get_region(start, len, buf)
    }
}

#[derive(miniextendr_api::AltrepReal)]
#[altrep(class = "ShortIterRealCoerce", manual)]
pub struct ShortIterRealCoerce {
    inner: IterRealCoerceData<std::array::IntoIter<i32, 2>, i32>,
}

impl AltrepLen for ShortIterRealCoerce {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl AltRealData for ShortIterRealCoerce {
    fn elt(&self, i: usize) -> f64 {
        self.inner.elt(i)
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [f64]) -> usize {
        self.inner.get_region(start, len, buf)
    }
}

#[derive(miniextendr_api::AltrepReal)]
#[altrep(class = "ShortWindowedReal", manual)]
pub struct ShortWindowedReal {
    inner: WindowedIterRealData<RealSource>,
}

impl AltrepLen for ShortWindowedReal {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl AltRealData for ShortWindowedReal {
    fn elt(&self, i: usize) -> f64 {
        self.inner.elt(i)
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [f64]) -> usize {
        self.inner.get_region(start, len, buf)
    }
}

#[derive(miniextendr_api::AltrepReal)]
#[altrep(class = "ShortSparseReal", manual)]
pub struct ShortSparseReal {
    inner: SparseIterRealData<RealSource>,
}

impl AltrepLen for ShortSparseReal {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl AltRealData for ShortSparseReal {
    fn elt(&self, i: usize) -> f64 {
        self.inner.elt(i)
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [f64]) -> usize {
        self.inner.get_region(start, len, buf)
    }
}

#[derive(miniextendr_api::AltrepReal)]
#[altrep(class = "ShortStreamReal", manual)]
pub struct ShortStreamReal {
    inner: StreamingRealData<fn(usize, &mut [f64]) -> usize>,
}

impl AltrepLen for ShortStreamReal {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl AltRealData for ShortStreamReal {
    fn elt(&self, i: usize) -> f64 {
        self.inner.elt(i)
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [f64]) -> usize {
        self.inner.get_region(start, len, buf)
    }
}

// endregion

// region: Complex wrappers

#[derive(miniextendr_api::AltrepComplex)]
#[altrep(class = "ShortIterComplex", manual)]
pub struct ShortIterComplex {
    inner: IterComplexData<ComplexSource>,
}

impl AltrepLen for ShortIterComplex {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl AltComplexData for ShortIterComplex {
    fn elt(&self, i: usize) -> Rcomplex {
        self.inner.elt(i)
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [Rcomplex]) -> usize {
        self.inner.get_region(start, len, buf)
    }
}

#[derive(miniextendr_api::AltrepComplex)]
#[altrep(class = "ShortSparseComplex", manual)]
pub struct ShortSparseComplex {
    inner: SparseIterComplexData<ComplexSource>,
}

impl AltrepLen for ShortSparseComplex {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl AltComplexData for ShortSparseComplex {
    fn elt(&self, i: usize) -> Rcomplex {
        self.inner.elt(i)
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [Rcomplex]) -> usize {
        self.inner.get_region(start, len, buf)
    }
}

// endregion

/// Create a double or complex ALTREP vector whose source ends early.
///
/// The vector declares 4 elements, but its source yields only 1 and 2
/// (`1+1i` and `2+2i` for complex), so elements 3 and 4 read as `NA`.
/// @param adaptor Data adaptor behind the vector: `"iter"`, `"iter_coerce"`,
///   `"windowed"`, `"sparse"`, `"stream"`, `"iter_complex"` or
///   `"sparse_complex"`.
/// @return An ALTREP double or complex vector of length 4.
/// @export
#[miniextendr]
pub fn altrep_short_source(adaptor: &str) -> SEXP {
    match adaptor {
        "iter" => ShortIterReal {
            inner: IterRealData::from_iter(real_source(), DECLARED),
        }
        .into_sexp(),
        "iter_coerce" => ShortIterRealCoerce {
            inner: IterRealCoerceData::from_iter([1, 2].into_iter(), DECLARED),
        }
        .into_sexp(),
        "windowed" => ShortWindowedReal {
            inner: WindowedIterRealData::from_iter(real_source(), DECLARED, DECLARED),
        }
        .into_sexp(),
        "sparse" => ShortSparseReal {
            inner: SparseIterRealData::from_iter(real_source(), DECLARED),
        }
        .into_sexp(),
        "stream" => ShortStreamReal {
            inner: StreamingRealData::new(DECLARED, DECLARED, short_reader),
        }
        .into_sexp(),
        "iter_complex" => ShortIterComplex {
            inner: IterComplexData::from_iter(complex_source(), DECLARED),
        }
        .into_sexp(),
        "sparse_complex" => ShortSparseComplex {
            inner: SparseIterComplexData::from_iter(complex_source(), DECLARED),
        }
        .into_sexp(),
        other => panic!("unknown adaptor {other:?}"),
    }
}
