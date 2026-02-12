//! Unwind protection benchmarks.

use miniextendr_api::ffi;
use miniextendr_api::unwind_protect::with_r_unwind_protect;

fn main() {
    miniextendr_bench::init();
    divan::main();
}

#[divan::bench]
fn unwind_protect_noop() {
    let out: i32 = with_r_unwind_protect(|| 42, None);
    divan::black_box(out);
}

#[divan::bench]
fn direct_noop() {
    let out: i32 = 42;
    divan::black_box(out);
}

// =============================================================================
// Closure with a trivial R API call inside unwind protection
// =============================================================================

#[divan::bench]
fn unwind_r_call() {
    let out = with_r_unwind_protect(|| unsafe { ffi::Rf_ScalarInteger(1) }, None);
    divan::black_box(out);
}

// TODO: panic_path — benchmark the cost of catching a Rust panic inside
// with_r_unwind_protect. Requires subprocess isolation to avoid contaminating
// the benchmark process state (panic hook, unwinding side effects).

// TODO: r_error_path — benchmark the cost of catching an R error (Rf_error)
// via R_UnwindProtect. Also requires subprocess isolation since R errors
// may leave the R session in a partially-reset state.
