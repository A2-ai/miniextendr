//! Iterator-backed ALTREP benchmarks.

use miniextendr_api::altrep_data::{AltIntegerData, AltrepLen};
use miniextendr_api::ffi::SexpExt;
use miniextendr_api::{IntoR, IterIntData};
use miniextendr_bench::raw_ffi;

const SIZE_INDICES: &[usize] = &[0, 2, 4];

#[derive(miniextendr_api::Altrep)]
#[altrep_derive_opts(class = "BenchIterInt")]
pub struct BenchIterIntData {
    inner: IterIntData<std::ops::Range<i32>>,
}

impl AltrepLen for BenchIterIntData {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl AltIntegerData for BenchIterIntData {
    fn elt(&self, i: usize) -> i32 {
        self.inner.elt(i)
    }

    fn as_slice(&self) -> Option<&[i32]> {
        self.inner.as_slice()
    }
}

miniextendr_api::impl_altinteger_from_data!(BenchIterIntData);

fn main() {
    miniextendr_bench::init();
    divan::main();
}

#[divan::bench(args = SIZE_INDICES)]
fn altrep_iter_int_elt(size_idx: usize) {
    let len = miniextendr_bench::SIZES[size_idx];
    let inner = IterIntData::from_iter(0..len as i32, len);
    let sexp = (BenchIterIntData { inner }).into_sexp();
    let val = sexp.integer_elt(0);
    divan::black_box(val);
}

#[divan::bench(args = SIZE_INDICES)]
fn altrep_iter_int_xlength(size_idx: usize) {
    let len = miniextendr_bench::SIZES[size_idx];
    let inner = IterIntData::from_iter(0..len as i32, len);
    let sexp = (BenchIterIntData { inner }).into_sexp();
    unsafe {
        let len = raw_ffi::Rf_xlength(sexp);
        divan::black_box(len);
    }
}
