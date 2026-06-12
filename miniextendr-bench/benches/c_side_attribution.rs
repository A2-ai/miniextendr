//! M2 — C-side wrapper attribution.
//!
//! Decompose the ~287 ns floor that the R-side bench measures for
//! `fast_i32_fast(42L)` (a wrapper with stopifnot + match.call stripped) into:
//!
//! 1. raw closure call (Rust noop)
//! 2. `catch_unwind` only (no R API contact)
//! 3. `with_r_unwind_protect_or_raise` (legacy raise-as-R-error path)
//! 4. `with_r_unwind_protect` (the tagged-SEXP transport used by `#[miniextendr]`)
//! 5. `with_r_unwind_protect` + `TryFromSexp<i32>` + `IntoR<i32>`
//!    (the full body of a `fast`-mode wrapper)
//!
//! Each step adds one layer. Deltas attribute cost to that layer.
//!
//! Run:
//!   cargo bench --manifest-path=miniextendr-bench/Cargo.toml \
//!     --bench c_side_attribution

use miniextendr_api::SEXP;
use miniextendr_api::unwind_protect::{with_r_unwind_protect, with_r_unwind_protect_or_raise};
use miniextendr_api::{IntoR, TryFromSexp};
use miniextendr_bench::raw_ffi;

fn main() {
    miniextendr_bench::init();
    divan::main();
}

// ---------------------------------------------------------------------------
// Cached SEXP inputs — built once at startup, reused across iterations.
// Using `Rf_protect` once keeps them pinned for the lifetime of the bench.
// ---------------------------------------------------------------------------

fn cached_i32_42() -> SEXP {
    use std::sync::OnceLock;
    static CACHE: OnceLock<SEXPPtr> = OnceLock::new();
    struct SEXPPtr(SEXP);
    unsafe impl Send for SEXPPtr {}
    unsafe impl Sync for SEXPPtr {}
    CACHE
        .get_or_init(|| unsafe {
            let s = raw_ffi::Rf_ScalarInteger(42);
            raw_ffi::Rf_protect(s);
            SEXPPtr(s)
        })
        .0
}

// ---------------------------------------------------------------------------
// Layer 1: bare closure call. The Rust floor — no unwind protection at all.
// ---------------------------------------------------------------------------

#[divan::bench]
fn l1_closure_only() -> i32 {
    let f = || 42i32;
    divan::black_box(f())
}

// ---------------------------------------------------------------------------
// Layer 2: catch_unwind only — sets up a Rust panic landing pad. No R API.
// ---------------------------------------------------------------------------

#[divan::bench]
fn l2_catch_unwind_only() -> i32 {
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| 42i32));
    divan::black_box(r.unwrap())
}

// ---------------------------------------------------------------------------
// Layer 3: legacy `with_r_unwind_protect_or_raise`.
//
// This is the panics-as-R-error variant: `Box<CallData>` alloc +
// `R_UnwindProtect_C_unwind` + 2× catch_unwind + Box::from_raw.
// Returns a generic `R`.
// ---------------------------------------------------------------------------

#[divan::bench]
fn l3_unwind_raise_noop() -> i32 {
    let out = with_r_unwind_protect_or_raise(|| 42i32, None);
    divan::black_box(out)
}

// ---------------------------------------------------------------------------
// Layer 4: `with_r_unwind_protect` — the tagged-condition transport used
// by every generated #[miniextendr] wrapper. Same machinery as L3 but
// constrained to `FnOnce() -> SEXP` and handling RCondition + tagged-SEXP
// transport on the panic path.
// ---------------------------------------------------------------------------

#[divan::bench]
fn l4_unwind_tagged_noop() -> SEXP {
    let out = with_r_unwind_protect(SEXP::nil, None);
    divan::black_box(out)
}

// ---------------------------------------------------------------------------
// Layer 5: full body of a `fast`-mode wrapper:
// with_r_unwind_protect { TryFromSexp<i32> → IntoR<i32> → SEXP }.
// ---------------------------------------------------------------------------

#[divan::bench]
fn l5_unwind_tagged_i32_roundtrip(bencher: divan::Bencher) {
    let input = cached_i32_42();
    bencher.bench_local(|| {
        let out = with_r_unwind_protect(
            || {
                let x: i32 = match TryFromSexp::try_from_sexp(input) {
                    Ok(v) => v,
                    Err(_) => return SEXP::nil(),
                };
                x.into_sexp()
            },
            None,
        );
        divan::black_box(out);
    });
}

// ---------------------------------------------------------------------------
// Layer 5b: just TryFromSexp + IntoR, no with_r_unwind_protect.
// Isolates the conversion cost without the unwind machinery.
// ---------------------------------------------------------------------------

#[divan::bench]
fn l5b_i32_roundtrip_no_unwind(bencher: divan::Bencher) {
    let input = cached_i32_42();
    bencher.bench_local(|| {
        let x: i32 = TryFromSexp::try_from_sexp(input).unwrap();
        let out = x.into_sexp();
        divan::black_box(out);
    });
}

// ---------------------------------------------------------------------------
// Layer 6: full R-callable C wrapper, exercised via Rf_eval of an
// installed wrapper. Closes the loop with the R-side measurement.
// ---------------------------------------------------------------------------

// (omitted — the R-side bench `scaffolding-fast-bench.R` already covers this)
