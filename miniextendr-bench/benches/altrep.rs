//! ALTREP benchmarks.

use miniextendr_api::ffi;
use miniextendr_api::ffi::SexpExt;
use miniextendr_api::{IntoR, miniextendr};
use miniextendr_bench::raw_ffi;

const SIZE_INDICES: &[usize] = &[0, 2, 4];

#[miniextendr(class = "BenchInt", pkg = "miniextendr.bench")]
struct BenchInt(Vec<i32>);

#[miniextendr(class = "BenchReal", pkg = "miniextendr.bench")]
struct BenchReal(Vec<f64>);

fn main() {
    miniextendr_bench::init();
    divan::main();
}

#[divan::bench(args = SIZE_INDICES)]
fn altrep_int_elt(size_idx: usize) {
    let len = miniextendr_bench::SIZES[size_idx];
    let data: Vec<i32> = (0..len as i32).collect();
    let sexp = BenchInt::from(data).into_sexp();
    let val = sexp.integer_elt(0);
    divan::black_box(val);
}

#[divan::bench(args = SIZE_INDICES)]
fn altrep_int_dataptr(size_idx: usize) {
    let len = miniextendr_bench::SIZES[size_idx];
    let data: Vec<i32> = (0..len as i32).collect();
    let sexp = BenchInt::from(data).into_sexp();
    unsafe {
        let ptr = ffi::DATAPTR_RO(sexp).cast::<i32>();
        divan::black_box(ptr);
    }
}

#[divan::bench(args = SIZE_INDICES)]
fn plain_int_elt(size_idx: usize) {
    let sexp = miniextendr_bench::fixtures().int_vec(size_idx);
    let val = sexp.integer_elt(0);
    divan::black_box(val);
}

#[divan::bench(args = SIZE_INDICES)]
fn plain_int_dataptr(size_idx: usize) {
    let sexp = miniextendr_bench::fixtures().int_vec(size_idx);
    unsafe {
        let ptr = raw_ffi::INTEGER(sexp);
        divan::black_box(ptr);
    }
}

#[divan::bench(args = SIZE_INDICES)]
fn altrep_real_elt(size_idx: usize) {
    let len = miniextendr_bench::SIZES[size_idx];
    let data: Vec<f64> = (0..len).map(|x| x as f64).collect();
    let sexp = BenchReal::from(data).into_sexp();
    let val = sexp.real_elt(0);
    divan::black_box(val);
}
