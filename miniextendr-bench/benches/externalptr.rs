//! ExternalPtr benchmarks.
//!
//! Measures the cost of creating, accessing, and managing ExternalPtr values.

use miniextendr_api::externalptr::{ErasedExternalPtr, ExternalPtr};

fn main() {
    miniextendr_bench::init();
    divan::main();
}

// region: Test types for ExternalPtr benchmarks

/// Small payload type (8 bytes).
#[derive(miniextendr_api::ExternalPtr)]
struct SmallPayload {
    value: i64,
}

/// Medium payload type (~1KB).
#[derive(miniextendr_api::ExternalPtr)]
pub struct MediumPayload {
    pub data: [u8; 1024],
}

/// Large payload type (~64KB).
#[derive(miniextendr_api::ExternalPtr)]
pub struct LargePayload {
    pub data: Box<[u8; 65536]>,
}

/// Another small payload type to benchmark downcast miss paths.
#[derive(miniextendr_api::ExternalPtr)]
struct OtherPayload {
    _value: i64,
}
// endregion

// region: ExternalPtr creation benchmarks

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
// endregion

// region: ExternalPtr access benchmarks (using bench_local_refs for !Sync types)

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
// endregion

// region: ExternalPtr SEXP conversion benchmarks

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
// endregion

// region: From trait usage benchmarks

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
// endregion

// region: Protection/tagging benchmarks

#[divan::bench]
fn set_protected(bencher: divan::Bencher) {
    use miniextendr_bench::raw_ffi;
    bencher
        .with_inputs(|| {
            let payload = SmallPayload { value: 42 };
            ExternalPtr::new(payload)
        })
        .bench_local_refs(|ptr| unsafe {
            let nil = raw_ffi::R_NilValue;
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
// endregion

// region: Comparison: Box vs ExternalPtr creation

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
// endregion

// region: Type-erased checks + downcasts

struct ProtectedSexp {
    sexp: miniextendr_api::SEXP,
}

impl Drop for ProtectedSexp {
    fn drop(&mut self) {
        unsafe { miniextendr_bench::raw_ffi::Rf_unprotect(1) };
    }
}

#[divan::bench]
fn erased_is_hit(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| unsafe {
            let ptr = ExternalPtr::new(SmallPayload { value: 42 });
            let sexp = ptr.as_sexp();
            miniextendr_bench::raw_ffi::Rf_protect(sexp);
            ProtectedSexp { sexp }
        })
        .bench_local_refs(|p| {
            let erased = unsafe { ErasedExternalPtr::from_sexp(p.sexp) };
            divan::black_box(erased.is::<SmallPayload>());
        });
}

#[divan::bench]
fn erased_is_miss(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| unsafe {
            let ptr = ExternalPtr::new(SmallPayload { value: 42 });
            let sexp = ptr.as_sexp();
            miniextendr_bench::raw_ffi::Rf_protect(sexp);
            ProtectedSexp { sexp }
        })
        .bench_local_refs(|p| {
            let erased = unsafe { ErasedExternalPtr::from_sexp(p.sexp) };
            divan::black_box(erased.is::<OtherPayload>());
        });
}

#[divan::bench]
fn erased_downcast_ref_hit(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| unsafe {
            let ptr = ExternalPtr::new(SmallPayload { value: 42 });
            let sexp = ptr.as_sexp();
            miniextendr_bench::raw_ffi::Rf_protect(sexp);
            ProtectedSexp { sexp }
        })
        .bench_local_refs(|p| {
            let erased = unsafe { ErasedExternalPtr::from_sexp(p.sexp) };
            let r = erased.downcast_ref::<SmallPayload>().unwrap();
            divan::black_box(r.value);
        });
}

#[divan::bench]
fn erased_downcast_mut_hit(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| unsafe {
            let ptr = ExternalPtr::new(SmallPayload { value: 42 });
            let sexp = ptr.as_sexp();
            miniextendr_bench::raw_ffi::Rf_protect(sexp);
            ProtectedSexp { sexp }
        })
        .bench_local_refs(|p| {
            let mut erased = unsafe { ErasedExternalPtr::from_sexp(p.sexp) };
            let r = erased.downcast_mut::<SmallPayload>().unwrap();
            r.value += 1;
            divan::black_box(r.value);
        });
}
// endregion

// region: Parameterized payload creation (A11: payload size scaling)

const PAYLOAD_BYTES: &[usize] = &[8, 64, 512, 4096, 65536];

/// Measure ExternalPtr creation cost as payload size increases.
#[divan::bench(args = PAYLOAD_BYTES)]
fn create_sized_payload(size: usize) {
    let data = vec![0u8; size];
    let ptr = ExternalPtr::new(data);
    divan::black_box(ptr);
}

/// Measure Box creation cost for comparison (same sizes).
#[divan::bench(args = PAYLOAD_BYTES)]
fn baseline_box_sized(size: usize) {
    let data = vec![0u8; size];
    let boxed = Box::new(data);
    divan::black_box(boxed);
}
// endregion

// region: Multiple ExternalPtrs (collection stress)

const COLLECTION_COUNTS: &[usize] = &[1, 10, 100, 1000];

/// Create N ExternalPtrs in sequence (measures allocation throughput).
#[divan::bench(args = COLLECTION_COUNTS)]
fn create_n_ptrs(n: usize) {
    for _ in 0..n {
        let ptr = ExternalPtr::new(SmallPayload { value: 42 });
        divan::black_box(ptr);
    }
}

/// Create N ExternalPtrs and check type on each (measures type check throughput).
#[divan::bench(args = COLLECTION_COUNTS)]
fn create_and_check_n_ptrs(bencher: divan::Bencher, n: usize) {
    bencher.bench_local(|| {
        for _ in 0..n {
            let ptr = ExternalPtr::new(SmallPayload { value: 42 });
            let sexp = ptr.as_sexp();
            let erased = unsafe { ErasedExternalPtr::from_sexp(sexp) };
            divan::black_box(erased.is::<SmallPayload>());
        }
    });
}
// endregion

// region: Vec<ExternalPtr> lifecycle (#836 — concurrent rooting + bulk release)
//
// Unlike `create_n_ptrs` (which drops each handle before allocating the next),
// these hold all N handles alive *simultaneously* in a `Vec` and then release
// them front-to-back when the `Vec` drops. This is the exact pattern that made
// `R_PreserveObject` rooting O(N²) (release scans R's precious list per element)
// and motivated rooting `ExternalPtr` through the O(1)-release `ProtectPool`
// instead (#836). See `analysis/gc-protection-benchmarks-results.md` for the
// mechanism-level precious-list vs pool comparison.

const VEC_LIFECYCLE_COUNTS: &[usize] = &[100, 1_000, 10_000];

/// Build a `Vec<ExternalPtr>` holding N handles concurrently, then drop it
/// (build + bulk release — the realistic end-to-end `Vec<ExternalPtr>` cost).
#[divan::bench(args = VEC_LIFECYCLE_COUNTS)]
fn vec_lifecycle(bencher: divan::Bencher, n: usize) {
    bencher.bench_local(|| {
        let v: Vec<ExternalPtr<SmallPayload>> = (0..n)
            .map(|i| ExternalPtr::new(SmallPayload { value: i as i64 }))
            .collect();
        divan::black_box(&v);
        drop(v);
    });
}

/// Isolate the *release* cost: build the `Vec` outside the timed region, time
/// only its drop (the front-to-back root release). This is the operation whose
/// asymptotics differ between `R_PreserveObject` (O(N²)) and `ProtectPool` (O(N)).
#[divan::bench(args = VEC_LIFECYCLE_COUNTS)]
fn vec_drop_release(bencher: divan::Bencher, n: usize) {
    bencher
        .with_inputs(|| {
            (0..n)
                .map(|i| ExternalPtr::new(SmallPayload { value: i as i64 }))
                .collect::<Vec<ExternalPtr<SmallPayload>>>()
        })
        .bench_local_values(drop);
}
// endregion

// region: Vec<ExternalPtr> → R list (#827/#836 — pool-rooted vs destination-rooted)
//
// The end-to-end question behind a `Vec<ExternalPtr<T>>` → `list()` conversion:
// is it cheaper to (a) build a pool-rooted `Vec<ExternalPtr>` and then copy each
// handle into an R list, or (b) build each `EXTPTRSXP` straight into the
// protected result list with `collect_into_r_list` (no pool, no copy pass)?
// Both produce the *same* artifact — a protected `VECSXP` of N external pointers
// — so the delta is exactly the per-element pool insert + release + copy that
// (b) elides. This is the data behind preferring `collect_into_r_list` over a
// `new_unprotected` escape hatch for the bulk path.

/// (a) Pool path: build `Vec<ExternalPtr>` (roots N in the pool), copy each into
/// an R list, then drop the `Vec` (releases N pool roots). The naive conversion.
#[divan::bench(args = VEC_LIFECYCLE_COUNTS)]
fn vec_pool_then_list(bencher: divan::Bencher, n: usize) {
    use miniextendr_api::{R_xlen_t, SEXPTYPE};
    use miniextendr_bench::raw_ffi;
    bencher.bench_local(|| {
        let v: Vec<ExternalPtr<SmallPayload>> = (0..n)
            .map(|i| ExternalPtr::new(SmallPayload { value: i as i64 }))
            .collect();
        let list = unsafe { raw_ffi::Rf_allocVector(SEXPTYPE::VECSXP, n as R_xlen_t) };
        unsafe { raw_ffi::Rf_protect(list) };
        for (i, p) in v.iter().enumerate() {
            unsafe { raw_ffi::SET_VECTOR_ELT(list, i as R_xlen_t, p.as_sexp()) };
        }
        divan::black_box(list);
        // Drop the Vec while the list still protects every element: this is the
        // front-to-back pool-root release the pool makes O(N).
        drop(v);
        unsafe { raw_ffi::Rf_unprotect(1) };
    });
}

/// (b) Destination path: build each `EXTPTRSXP` straight into the protected list
/// via `collect_into_r_list` — no pool traffic, no copy pass.
#[divan::bench(args = VEC_LIFECYCLE_COUNTS)]
fn vec_collect_into_list(bencher: divan::Bencher, n: usize) {
    use miniextendr_bench::raw_ffi;
    bencher.bench_local(|| {
        let list = ExternalPtr::<SmallPayload>::collect_into_r_list(
            (0..n).map(|i| SmallPayload { value: i as i64 }),
        );
        unsafe { raw_ffi::Rf_protect(list) };
        divan::black_box(list);
        unsafe { raw_ffi::Rf_unprotect(1) };
    });
}
// endregion
