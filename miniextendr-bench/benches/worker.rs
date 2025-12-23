//! Worker thread benchmarks.

use miniextendr_api::ffi;
use miniextendr_api::worker::{run_on_worker, with_r_thread};

fn main() {
    miniextendr_bench::init();
    divan::main();
}

#[divan::bench]
fn run_on_worker_no_r() {
    let out: i32 = run_on_worker(|| 7);
    divan::black_box(out);
}

#[divan::bench]
fn run_on_worker_with_r_thread() {
    let out = run_on_worker(|| with_r_thread(|| unsafe { ffi::Rf_ScalarInteger(1) }));
    divan::black_box(out);
}

#[divan::bench]
fn with_r_thread_main() {
    let out = with_r_thread(|| unsafe { ffi::Rf_ScalarInteger(1) });
    divan::black_box(out);
}
