//! Worker thread benchmarks.

use miniextendr_api::ffi;
use miniextendr_api::worker::{run_on_worker, with_r_thread};

fn main() {
    miniextendr_bench::init();
    divan::main();
}

#[divan::bench]
fn run_on_worker_no_r() {
    let out: i32 = run_on_worker(|| 7).unwrap();
    divan::black_box(out);
}

#[divan::bench]
fn run_on_worker_with_r_thread() {
    let out = run_on_worker(|| with_r_thread(|| unsafe { ffi::Rf_ScalarInteger(1) })).unwrap();
    divan::black_box(out);
}

#[divan::bench]
fn with_r_thread_main() {
    let out = with_r_thread(|| unsafe { ffi::Rf_ScalarInteger(1) });
    divan::black_box(out);
}

// region: Channel saturation — many small requests in sequence through the worker

const SATURATION_COUNTS: &[usize] = &[1, 5, 20, 100];

#[divan::bench(args = SATURATION_COUNTS)]
fn worker_channel_saturation(n: usize) {
    for i in 0..n {
        let out: i32 = run_on_worker(move || i as i32).unwrap();
        divan::black_box(out);
    }
}
// endregion

// region: Batching — N with_r_thread calls inside one worker closure

const BATCH_COUNTS: &[usize] = &[1, 5, 10, 50];

#[divan::bench(args = BATCH_COUNTS)]
fn worker_batching(n: usize) {
    run_on_worker(move || {
        let mut sum = 0i32;
        for _ in 0..n {
            let sexp = with_r_thread(|| unsafe { ffi::Rf_ScalarInteger(1) });
            sum += unsafe { ffi::Rf_asInteger(sexp) } as i32;
        }
        divan::black_box(sum);
    })
    .unwrap();
}
// endregion

// region: Payload sizes — measure cost as data size increases

const PAYLOAD_SIZES: &[usize] = &[8, 256, 4096, 65536];

#[divan::bench(args = PAYLOAD_SIZES)]
fn worker_payload_size(size: usize) {
    let data = vec![0u8; size];
    let out: usize = run_on_worker(move || data.len()).unwrap();
    divan::black_box(out);
}
// endregion
