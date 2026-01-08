//! Trait ABI benchmarks.
//!
//! Measures the cost of type-erased trait dispatch (mx_erased + vtable query).

use std::os::raw::c_void;

use miniextendr_api::abi::mx_erased;
use miniextendr_api::trait_abi::TraitView;
use miniextendr_api::{ExternalPtr, miniextendr, miniextendr_module};

fn main() {
    miniextendr_bench::init();
    divan::main();
}

// =============================================================================
// Test trait + type
// =============================================================================

#[miniextendr]
pub trait Counter {
    fn value(&self) -> i32;
    fn increment(&mut self);
}

#[derive(ExternalPtr)]
pub struct SimpleCounter {
    value: i32,
}

#[miniextendr]
impl Counter for SimpleCounter {
    fn value(&self) -> i32 {
        self.value
    }

    fn increment(&mut self) {
        self.value += 1;
    }
}

miniextendr_module! {
    mod trait_abi_bench;
    impl Counter for SimpleCounter;
}

// =============================================================================
// Helpers
// =============================================================================

struct ErasedCounter {
    ptr: *mut mx_erased,
}

impl ErasedCounter {
    fn new(start: i32) -> Self {
        let ptr = __mx_wrap_simplecounter(SimpleCounter { value: start });
        Self { ptr }
    }

    #[inline(always)]
    unsafe fn query_counter_vtable(&self) -> *const c_void {
        let base = unsafe { (*self.ptr).base };
        unsafe { ((*base).query)(self.ptr, TAG_COUNTER) }
    }

    #[inline(always)]
    unsafe fn data_ptr(&self) -> *mut c_void {
        let data_offset = std::mem::size_of::<mx_erased>();
        unsafe { (self.ptr as *mut u8).add(data_offset) as *mut c_void }
    }
}

impl Drop for ErasedCounter {
    fn drop(&mut self) {
        if self.ptr.is_null() {
            return;
        }
        unsafe {
            let base = (*self.ptr).base;
            ((*base).drop)(self.ptr);
        }
    }
}

struct OwnedCounterView {
    _erased: ErasedCounter,
    view: CounterView,
}

impl OwnedCounterView {
    fn new(start: i32) -> Self {
        let erased = ErasedCounter::new(start);
        let vtable = unsafe { erased.query_counter_vtable() };
        let data = unsafe { erased.data_ptr() };
        let view = unsafe { <CounterView as TraitView>::from_raw_parts(data, vtable) };
        Self {
            _erased: erased,
            view,
        }
    }
}

// =============================================================================
// Benchmarks
// =============================================================================

#[divan::bench]
fn baseline_direct_value(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| SimpleCounter { value: 1 })
        .bench_local_refs(|counter| {
            let v = counter.value();
            divan::black_box(v);
        });
}

#[divan::bench]
fn baseline_direct_increment(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| SimpleCounter { value: 1 })
        .bench_local_refs(|counter| {
            let mut c = SimpleCounter {
                value: counter.value,
            };
            c.increment();
            divan::black_box(c.value);
        });
}

#[divan::bench]
fn mx_query_vtable(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| ErasedCounter::new(1))
        .bench_local_refs(|erased| unsafe {
            let vtable = erased.query_counter_vtable();
            divan::black_box(vtable);
        });
}

#[divan::bench]
fn view_value_only(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| OwnedCounterView::new(1))
        .bench_local_refs(|owned| {
            let v = owned.view.value();
            divan::black_box(v);
        });
}

#[divan::bench]
fn query_view_value(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| ErasedCounter::new(1))
        .bench_local_refs(|erased| unsafe {
            let vtable = erased.query_counter_vtable();
            let data = erased.data_ptr();
            let view = <CounterView as TraitView>::from_raw_parts(data, vtable);
            divan::black_box(view.value());
        });
}
