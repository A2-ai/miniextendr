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

// =============================================================================
// catch_unwind overhead — measures the cost of panic-catching infrastructure
// =============================================================================

/// Baseline: std::panic::catch_unwind on a non-panicking closure.
#[divan::bench]
fn catch_unwind_success() {
    let result = std::panic::catch_unwind(|| 42i32);
    divan::black_box(result.unwrap());
}

/// Measure catch_unwind cost when a panic IS caught.
/// Uses AssertUnwindSafe to allow benchmarking the error path.
#[divan::bench]
fn catch_unwind_panic() {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> i32 {
        panic!("bench panic")
    }));
    divan::black_box(result.is_err());
}

// =============================================================================
// Nested unwind protection — measures cost of stacking protection layers
// =============================================================================

#[divan::bench]
fn unwind_nested_2() {
    let out = with_r_unwind_protect(
        || with_r_unwind_protect(|| unsafe { ffi::Rf_ScalarInteger(1) }, None),
        None,
    );
    divan::black_box(out);
}

#[divan::bench]
fn unwind_nested_5() {
    let out = with_r_unwind_protect(
        || {
            with_r_unwind_protect(
                || {
                    with_r_unwind_protect(
                        || {
                            with_r_unwind_protect(
                                || {
                                    with_r_unwind_protect(
                                        || unsafe { ffi::Rf_ScalarInteger(1) },
                                        None,
                                    )
                                },
                                None,
                            )
                        },
                        None,
                    )
                },
                None,
            )
        },
        None,
    );
    divan::black_box(out);
}

// NOTE: R error path (Rf_error inside with_r_unwind_protect) and panic-through-R-unwind
// benchmarks require subprocess isolation since the error/panic longjmps past the divan
// harness. These paths are tested for correctness in rpkg/tests/testthat/test-subprocess-isolated.R.
