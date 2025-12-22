//! ExternalPtr benchmarks.
//!
//! Measures the cost of creating, accessing, and managing ExternalPtr values.

use miniextendr_api::externalptr::ExternalPtr;

fn main() {
    miniextendr_bench::init();
    divan::main();
}

// =============================================================================
// Test types for ExternalPtr benchmarks
// =============================================================================

/// Small payload type (8 bytes).
#[derive(miniextendr_api::ExternalPtr)]
struct SmallPayload {
    value: i64,
}

/// Medium payload type (~1KB).
#[derive(miniextendr_api::ExternalPtr)]
struct MediumPayload {
    data: [u8; 1024],
}

/// Large payload type (~64KB).
#[derive(miniextendr_api::ExternalPtr)]
struct LargePayload {
    data: Box<[u8; 65536]>,
}

// =============================================================================
// ExternalPtr creation benchmarks
// =============================================================================

#[divan::bench]
fn create_small_payload() {
    let payload = SmallPayload { value: 42 };
    let ptr = ExternalPtr::new(payload);
    divan::black_box(ptr);
}

#[divan::bench]
fn create_medium_payload() {
    let payload = MediumPayload { data: [0u8; 1024] };
    let ptr = ExternalPtr::new(payload);
    divan::black_box(ptr);
}

#[divan::bench]
fn create_large_payload() {
    let payload = LargePayload {
        data: Box::new([0u8; 65536]),
    };
    let ptr = ExternalPtr::new(payload);
    divan::black_box(ptr);
}

// =============================================================================
// ExternalPtr access benchmarks (using bench_local_refs for !Sync types)
// =============================================================================

#[divan::bench]
fn access_as_ref(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| {
            let payload = SmallPayload { value: 42 };
            ExternalPtr::new(payload)
        })
        .bench_local_refs(|ptr| {
            let r = ptr.as_ref();
            divan::black_box(r);
        });
}

#[divan::bench]
fn access_deref(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| {
            let payload = SmallPayload { value: 42 };
            ExternalPtr::new(payload)
        })
        .bench_local_refs(|ptr| {
            let val = ptr.value;
            divan::black_box(val);
        });
}

#[divan::bench]
fn access_as_ptr(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| {
            let payload = SmallPayload { value: 42 };
            ExternalPtr::new(payload)
        })
        .bench_local_refs(|ptr| {
            let raw = ptr.as_ptr();
            divan::black_box(raw);
        });
}

// =============================================================================
// ExternalPtr SEXP conversion benchmarks
// =============================================================================

#[divan::bench]
fn as_sexp(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| {
            let payload = SmallPayload { value: 42 };
            ExternalPtr::new(payload)
        })
        .bench_local_refs(|ptr| {
            let sexp = ptr.as_sexp();
            divan::black_box(sexp);
        });
}

// =============================================================================
// From trait usage benchmarks
// =============================================================================

#[divan::bench]
fn from_value() {
    let payload = SmallPayload { value: 42 };
    let ptr: ExternalPtr<SmallPayload> = payload.into();
    divan::black_box(ptr);
}

#[divan::bench]
fn from_box() {
    let boxed = Box::new(SmallPayload { value: 42 });
    let ptr: ExternalPtr<SmallPayload> = boxed.into();
    divan::black_box(ptr);
}

// =============================================================================
// Protection/tagging benchmarks
// =============================================================================

#[divan::bench]
fn set_protected(bencher: divan::Bencher) {
    use miniextendr_api::ffi;
    bencher
        .with_inputs(|| {
            let payload = SmallPayload { value: 42 };
            ExternalPtr::new(payload)
        })
        .bench_local_refs(|ptr| unsafe {
            let nil = ffi::R_NilValue;
            ptr.set_protected(nil);
            divan::black_box(ptr);
        });
}

#[divan::bench]
fn get_tag(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| {
            let payload = SmallPayload { value: 42 };
            ExternalPtr::new(payload)
        })
        .bench_local_refs(|ptr| {
            let tag = ptr.tag();
            divan::black_box(tag);
        });
}

// =============================================================================
// Comparison: Box vs ExternalPtr creation
// =============================================================================

#[divan::bench]
fn baseline_box_small() {
    let boxed = Box::new(SmallPayload { value: 42 });
    divan::black_box(boxed);
}

#[divan::bench]
fn baseline_box_medium() {
    let boxed = Box::new(MediumPayload { data: [0u8; 1024] });
    divan::black_box(boxed);
}

#[divan::bench]
fn baseline_box_large() {
    let boxed = Box::new(LargePayload {
        data: Box::new([0u8; 65536]),
    });
    divan::black_box(boxed);
}
