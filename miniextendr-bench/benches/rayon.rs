//! Rayon bridge benchmarks (feature-gated).

use miniextendr_api::ffi;
#[cfg(feature = "rayon")]
use miniextendr_api::rayon_bridge::rayon::prelude::*;
#[cfg(feature = "rayon")]
use miniextendr_api::rayon_bridge::{reduce, with_r_vec};
#[cfg(feature = "rayon")]
use miniextendr_bench::raw_ffi;

#[cfg(feature = "rayon")]
fn main() {
    miniextendr_bench::init();
    divan::main();
}

#[cfg(not(feature = "rayon"))]
fn main() {}

#[cfg(feature = "rayon")]
#[divan::bench(args = miniextendr_bench::SIZES)]
fn rayon_with_r_vec(len: usize) {
    let sexp = with_r_vec::<f64, _>(len, |chunk, offset| {
        for (i, slot) in chunk.iter_mut().enumerate() {
            *slot = (offset + i) as f64;
        }
    });
    divan::black_box(sexp);
}

#[cfg(feature = "rayon")]
#[divan::bench(args = miniextendr_bench::SIZES)]
fn rayon_collect_vec(len: usize) {
    let vec: Vec<i32> = (0..len as i32).into_par_iter().collect();
    divan::black_box(vec.len());
}

#[cfg(feature = "rayon")]
#[divan::bench(args = [0, 2, 4])]
fn rayon_reduce_sum(size_idx: usize) {
    let sexp = miniextendr_bench::fixtures().real_vec(size_idx);
    unsafe {
        let ptr = raw_ffi::REAL(sexp);
        let len = raw_ffi::Rf_xlength(sexp) as usize;
        let slice = std::slice::from_raw_parts(ptr, len);
        let out = reduce::sum(slice);
        divan::black_box(out);
    }
}
