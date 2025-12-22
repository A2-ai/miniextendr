//! SexpExt helper benchmarks.

use miniextendr_api::ffi::{self, SexpExt};

const SIZE_INDICES: &[usize] = &[0, 2, 4];

fn main() {
    miniextendr_bench::init();
    divan::main();
}

#[divan::bench(args = SIZE_INDICES)]
fn sexp_len_ext(size_idx: usize) {
    let sexp = miniextendr_bench::fixtures().int_vec(size_idx);
    let len = sexp.len();
    divan::black_box(len);
}

#[divan::bench(args = SIZE_INDICES)]
fn sexp_len_raw(size_idx: usize) {
    let sexp = miniextendr_bench::fixtures().int_vec(size_idx);
    unsafe {
        let len = ffi::Rf_xlength(sexp);
        divan::black_box(len);
    }
}

#[divan::bench]
fn sexp_is_integer_ext() {
    let sexp = miniextendr_bench::fixtures().int_vec(2);
    let out = sexp.is_integer();
    divan::black_box(out);
}

#[divan::bench]
fn sexp_is_integer_raw() {
    let sexp = miniextendr_bench::fixtures().int_vec(2);
    unsafe {
        let out = ffi::TYPEOF(sexp) == ffi::SEXPTYPE::INTSXP;
        divan::black_box(out);
    }
}

#[divan::bench(args = SIZE_INDICES)]
fn sexp_as_slice_ext(size_idx: usize) {
    let sexp = miniextendr_bench::fixtures().int_vec(size_idx);
    unsafe {
        let slice: &[i32] = sexp.as_slice();
        divan::black_box(slice.as_ptr());
    }
}

#[divan::bench(args = SIZE_INDICES)]
fn sexp_as_slice_raw(size_idx: usize) {
    let sexp = miniextendr_bench::fixtures().int_vec(size_idx);
    unsafe {
        let ptr = ffi::INTEGER(sexp);
        let len = ffi::Rf_xlength(sexp) as usize;
        let slice = std::slice::from_raw_parts(ptr, len);
        divan::black_box(slice.as_ptr());
    }
}
