//! ALTREP benchmarks.

use miniextendr_api::IntoR;
use miniextendr_api::altrep_data::{AltIntegerData, AltRealData, AltrepDataptr, AltrepLen};
use miniextendr_api::ffi;
use miniextendr_api::ffi::SexpExt;
use miniextendr_bench::raw_ffi;

const SIZE_INDICES: &[usize] = &[0, 2, 4];

#[derive(miniextendr_api::Altrep)]
#[altrep(class = "BenchInt", base = "Integer", dataptr)]
pub struct BenchInt {
    data: Vec<i32>,
}

impl AltrepLen for BenchInt {
    fn len(&self) -> usize {
        self.data.len()
    }
}

impl AltIntegerData for BenchInt {
    fn elt(&self, i: usize) -> i32 {
        self.data[i]
    }

    fn as_slice(&self) -> Option<&[i32]> {
        Some(&self.data)
    }
}

impl AltrepDataptr<i32> for BenchInt {
    fn dataptr(&mut self, _writable: bool) -> Option<*mut i32> {
        Some(self.data.as_mut_ptr())
    }

    fn dataptr_or_null(&self) -> Option<*const i32> {
        Some(self.data.as_ptr())
    }
}

#[derive(miniextendr_api::Altrep)]
#[altrep(class = "BenchReal", base = "Real", dataptr)]
pub struct BenchReal {
    data: Vec<f64>,
}

impl AltrepLen for BenchReal {
    fn len(&self) -> usize {
        self.data.len()
    }
}

impl AltRealData for BenchReal {
    fn elt(&self, i: usize) -> f64 {
        self.data[i]
    }

    fn as_slice(&self) -> Option<&[f64]> {
        Some(&self.data)
    }
}

impl AltrepDataptr<f64> for BenchReal {
    fn dataptr(&mut self, _writable: bool) -> Option<*mut f64> {
        Some(self.data.as_mut_ptr())
    }

    fn dataptr_or_null(&self) -> Option<*const f64> {
        Some(self.data.as_ptr())
    }
}

fn main() {
    miniextendr_bench::init();
    divan::main();
}

#[divan::bench(args = SIZE_INDICES)]
fn altrep_int_elt(size_idx: usize) {
    let len = miniextendr_bench::SIZES[size_idx];
    let data: Vec<i32> = (0..len as i32).collect();
    let sexp = (BenchInt { data }).into_sexp();
    let val = sexp.integer_elt(0);
    divan::black_box(val);
}

#[divan::bench(args = SIZE_INDICES)]
fn altrep_int_dataptr(size_idx: usize) {
    let len = miniextendr_bench::SIZES[size_idx];
    let data: Vec<i32> = (0..len as i32).collect();
    let sexp = (BenchInt { data }).into_sexp();
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
    let sexp = (BenchReal { data }).into_sexp();
    let val = sexp.real_elt(0);
    divan::black_box(val);
}
