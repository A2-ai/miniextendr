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

// =============================================================================
// Channel saturation — many small requests in sequence through the worker
// =============================================================================

#[divan::bench]
fn worker_channel_saturation(bencher: divan::Bencher) {
    bencher.bench_local(|| {
        // 20 sequential worker round-trips with minimal work.
        for i in 0..20 {
            let out: i32 = run_on_worker(move || i);
            divan::black_box(out);
        }
    });
}

// =============================================================================
// Batching — N with_r_thread calls inside one worker closure
// =============================================================================

#[divan::bench]
fn worker_batching(bencher: divan::Bencher) {
    bencher.bench_local(|| {
        // Single worker hop, 10 R thread requests batched inside.
        run_on_worker(|| {
            let mut sum = 0i32;
            for _ in 0..10 {
                let sexp = with_r_thread(|| unsafe { ffi::Rf_ScalarInteger(1) });
                sum += unsafe { ffi::Rf_asInteger(sexp) } as i32;
            }
            divan::black_box(sum);
        });
    });
}
