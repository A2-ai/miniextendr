//! Iterator-backed ALTREP benchmarks.

use miniextendr_api::ffi;
use miniextendr_api::{IntoR, IterIntData, miniextendr};

const SIZE_INDICES: &[usize] = &[0, 2, 4];

#[miniextendr(class = "BenchIterInt", pkg = "miniextendr.bench")]
struct BenchIterInt(IterIntData<std::ops::Range<i32>>);

fn main() {
    miniextendr_bench::init();
    divan::main();
}

#[divan::bench(args = SIZE_INDICES)]
fn altrep_iter_int_elt(size_idx: usize) {
    let len = miniextendr_bench::SIZES[size_idx];
    let data = IterIntData::from_iter(0..len as i32, len);
    let sexp = BenchIterInt::from(data).into_sexp();
    unsafe {
        let val = ffi::INTEGER_ELT(sexp, 0);
        divan::black_box(val);
    }
}

#[divan::bench(args = SIZE_INDICES)]
fn altrep_iter_int_xlength(size_idx: usize) {
    let len = miniextendr_bench::SIZES[size_idx];
    let data = IterIntData::from_iter(0..len as i32, len);
    let sexp = BenchIterInt::from(data).into_sexp();
    unsafe {
        let len = ffi::Rf_xlength(sexp);
        divan::black_box(len);
    }
}
