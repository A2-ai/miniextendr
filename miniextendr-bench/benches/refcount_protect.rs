//! Benchmarks for RefCountedArena vs ProtectScope.
//!
//! These benchmarks include SEXP allocation in the timed region, measuring
//! end-to-end arena usage cost (allocation + protection + release). For
//! isolated protection-only timings, see `gc_protection_compare.rs`.
//!
//! Compares:
//! - Protection overhead (single value)
//! - Multiple protections
//! - Reference counting (same value protected multiple times)
//! - Release order flexibility
//! - High iteration counts

use miniextendr_api::SEXPTYPE;
use miniextendr_api::gc_protect::ProtectScope;
use miniextendr_api::refcount_protect::{RefCountedArena, ThreadLocalArena};
use miniextendr_api::sys::{self, Rf_allocVector};
use miniextendr_bench::raw_ffi;

fn main() {
    miniextendr_bench::init();
    divan::main();
}

// region: Raw R_PreserveObject/R_ReleaseObject baseline
// NOTE: R_ReleaseObject is O(n) - it scans the precious list to find the object.
// This makes protect+release cycles O(n²) at scale, which is why RefCountedArena
// (with hash table for O(1) lookup) is much faster for large numbers of objects.

/// Raw R_PreserveObject + R_ReleaseObject (checked)
#[divan::bench]
fn raw_preserve_release_single() {
    unsafe {
        let x = raw_ffi::Rf_ScalarInteger(42);
        raw_ffi::R_PreserveObject(x);
        raw_ffi::R_ReleaseObject(x);
        divan::black_box(x);
    }
}

/// Raw R_PreserveObject + R_ReleaseObject (unchecked)
#[divan::bench]
fn raw_preserve_release_unchecked_single() {
    unsafe {
        let x = raw_ffi::Rf_ScalarInteger(42);
        sys::R_PreserveObject_unchecked(x);
        sys::R_ReleaseObject_unchecked(x);
        divan::black_box(x);
    }
}

/// Raw R_PreserveObject + R_ReleaseObject: N values (checked)
/// WARNING: O(n²) due to R_ReleaseObject scanning
#[divan::bench(args = [10, 100, 1000])]
fn raw_preserve_release_multiple(n: usize) {
    unsafe {
        let mut values = Vec::with_capacity(n);
        for i in 0..n {
            let x = raw_ffi::Rf_ScalarInteger(i as i32);
            raw_ffi::R_PreserveObject(x);
            values.push(x);
        }
        for x in values {
            raw_ffi::R_ReleaseObject(x);
        }
    }
}

/// Raw R_PreserveObject + R_ReleaseObject: N values (unchecked)
/// WARNING: O(n²) due to R_ReleaseObject scanning
#[divan::bench(args = [10, 100, 1000])]
fn raw_preserve_release_unchecked_multiple(n: usize) {
    unsafe {
        let mut values = Vec::with_capacity(n);
        for i in 0..n {
            let x = raw_ffi::Rf_ScalarInteger(i as i32);
            sys::R_PreserveObject_unchecked(x);
            values.push(x);
        }
        for x in values {
            sys::R_ReleaseObject_unchecked(x);
        }
    }
}

/// Raw R_PreserveObject: scale test (protect only, no release)
/// This isolates preserve cost from the O(n) release cost
#[divan::bench(args = [1000, 5000, 10000])]
fn raw_preserve_only(n: usize) {
    unsafe {
        for i in 0..n {
            let x = raw_ffi::Rf_ScalarInteger((i % 100) as i32);
            sys::R_PreserveObject_unchecked(x);
            divan::black_box(x);
        }
        // NOTE: Not releasing - this pollutes the precious list but shows true preserve cost
    }
}

/// Raw R_PreserveObject + R_ReleaseObject: scale test
/// WARNING: Very slow at scale due to O(n²) release cost
#[divan::bench(args = [100, 500, 1000, 2000])]
fn raw_preserve_release_scale(n: usize) {
    unsafe {
        let mut values = Vec::with_capacity(n);
        for i in 0..n {
            let x = raw_ffi::Rf_ScalarInteger((i % 100) as i32);
            sys::R_PreserveObject_unchecked(x);
            values.push(x);
        }
        for x in values {
            sys::R_ReleaseObject_unchecked(x);
        }
    }
}
// endregion

// region: Single value protection

/// ProtectScope: protect single value
#[divan::bench]
fn protect_scope_single() {
    unsafe {
        let scope = ProtectScope::new();
        let x = scope.protect(raw_ffi::Rf_ScalarInteger(42));
        divan::black_box(x.get());
    }
}

/// RefCountedArena: protect single value
#[divan::bench]
fn refcount_arena_single() {
    unsafe {
        let arena = RefCountedArena::new();
        let x = arena.protect(raw_ffi::Rf_ScalarInteger(42));
        divan::black_box(x);
    }
}

/// RefCountedArena with guard: protect single value
#[divan::bench]
fn refcount_arena_guard_single() {
    unsafe {
        let arena = RefCountedArena::new();
        let guard = arena.guard(raw_ffi::Rf_ScalarInteger(42));
        divan::black_box(guard.get());
    }
}
// endregion

// region: Multiple value protection

/// ProtectScope: protect N values
#[divan::bench(args = [10, 100, 1000])]
fn protect_scope_multiple(n: usize) {
    unsafe {
        let scope = ProtectScope::new();
        for i in 0..n {
            let _ = scope.protect(raw_ffi::Rf_ScalarInteger(i as i32));
        }
        divan::black_box(scope.count());
    }
}

/// RefCountedArena: protect N distinct values
#[divan::bench(args = [10, 100, 1000])]
fn refcount_arena_multiple(n: usize) {
    unsafe {
        let arena = RefCountedArena::new();
        for i in 0..n {
            arena.protect(raw_ffi::Rf_ScalarInteger(i as i32));
        }
        divan::black_box(arena.len());
    }
}
// endregion

// region: Reference counting (same value multiple times)

/// RefCountedArena: protect same value N times
#[divan::bench(args = [10, 100, 1000])]
fn refcount_arena_same_value(n: usize) {
    unsafe {
        let arena = RefCountedArena::new();
        let x = raw_ffi::Rf_ScalarInteger(42);

        for _ in 0..n {
            arena.protect(x);
        }

        divan::black_box(arena.ref_count(x));
    }
}

/// ProtectScope: protect same value N times (for comparison)
#[divan::bench(args = [10, 100, 1000])]
fn protect_scope_same_value(n: usize) {
    unsafe {
        let scope = ProtectScope::new();
        let x = raw_ffi::Rf_ScalarInteger(42);

        for _ in 0..n {
            let _ = scope.protect(x);
        }

        divan::black_box(scope.count());
    }
}
// endregion

// region: Protect + unprotect cycles

/// RefCountedArena: protect then unprotect N values
#[divan::bench(args = [10, 100, 1000])]
fn refcount_arena_protect_unprotect(n: usize) {
    unsafe {
        let arena = RefCountedArena::new();
        let mut values = Vec::with_capacity(n);

        // Protect all
        for i in 0..n {
            values.push(arena.protect(raw_ffi::Rf_ScalarInteger(i as i32)));
        }

        // Unprotect in reverse order
        for x in values.into_iter().rev() {
            arena.unprotect(x);
        }

        divan::black_box(arena.is_empty());
    }
}

/// RefCountedArena: protect then unprotect in random order
#[divan::bench(args = [10, 100, 1000])]
fn refcount_arena_unprotect_random_order(n: usize) {
    unsafe {
        let arena = RefCountedArena::new();
        let mut values = Vec::with_capacity(n);

        // Protect all
        for i in 0..n {
            values.push(arena.protect(raw_ffi::Rf_ScalarInteger(i as i32)));
        }

        // Unprotect in "random" order (every 3rd, then every 2nd, then rest)
        for i in (0..n).step_by(3) {
            arena.unprotect(values[i]);
        }
        for i in (1..n).step_by(3) {
            arena.unprotect(values[i]);
        }
        for i in (2..n).step_by(3) {
            arena.unprotect(values[i]);
        }

        divan::black_box(arena.is_empty());
    }
}
// endregion

// region: Large scale tests

/// RefCountedArena: protect many values (stress test)
#[divan::bench(args = [1000, 5000, 10000])]
fn refcount_arena_many_values(n: usize) {
    unsafe {
        let arena = RefCountedArena::new();

        for i in 0..n {
            arena.protect(raw_ffi::Rf_ScalarInteger((i % 100) as i32));
        }

        divan::black_box(arena.len());
    }
}

/// ProtectScope: protect many values (stress test)
/// Note: This uses ProtectScope's stack, limited by --max-ppsize
#[divan::bench(args = [1000, 5000])]
fn protect_scope_many_values(n: usize) {
    unsafe {
        let scope = ProtectScope::new();

        for i in 0..n {
            let _ = scope.protect(raw_ffi::Rf_ScalarInteger((i % 100) as i32));
        }

        divan::black_box(scope.count());
    }
}
// endregion

// region: Guard vs manual protect/unprotect

/// RefCountedArena: guard pattern
#[divan::bench(args = [10, 100])]
fn refcount_arena_guards(n: usize) {
    unsafe {
        let arena = RefCountedArena::new();

        for i in 0..n {
            let _guard = arena.guard(raw_ffi::Rf_ScalarInteger(i as i32));
            // guard drops at end of loop iteration
        }

        divan::black_box(arena.is_empty());
    }
}

/// RefCountedArena: manual protect/unprotect pattern
#[divan::bench(args = [10, 100])]
fn refcount_arena_manual(n: usize) {
    unsafe {
        let arena = RefCountedArena::new();

        for i in 0..n {
            let x = arena.protect(raw_ffi::Rf_ScalarInteger(i as i32));
            arena.unprotect(x);
        }

        divan::black_box(arena.is_empty());
    }
}
// endregion

// region: Mixed workload

/// RefCountedArena: realistic workload with vectors
#[divan::bench]
fn refcount_arena_realistic() {
    unsafe {
        let arena = RefCountedArena::new();

        // Protect a list
        let list = arena.protect(Rf_allocVector(SEXPTYPE::VECSXP, 10));

        // Protect some children
        for i in 0..10 {
            let child = arena.protect(raw_ffi::Rf_ScalarInteger(i));
            raw_ffi::SET_VECTOR_ELT(list, i as isize, child);
            // Children remain protected
        }

        // Unprotect in arbitrary order
        arena.unprotect(list);

        divan::black_box(arena.len());
    }
}

/// ProtectScope: equivalent realistic workload
#[divan::bench]
fn protect_scope_realistic() {
    unsafe {
        let scope = ProtectScope::new();

        // Protect a list
        let list = scope.protect_raw(Rf_allocVector(SEXPTYPE::VECSXP, 10));

        // Protect some children
        for i in 0..10 {
            let child = scope.protect_raw(raw_ffi::Rf_ScalarInteger(i));
            raw_ffi::SET_VECTOR_ELT(list, i as isize, child);
        }

        divan::black_box(list);
        // All unprotected on scope drop
    }
}
// endregion

// region: ThreadLocalArena benchmarks

/// ThreadLocalArena: single protect
#[divan::bench]
fn thread_local_single() {
    unsafe {
        let x = ThreadLocalArena::protect(raw_ffi::Rf_ScalarInteger(42));
        divan::black_box(x);
        ThreadLocalArena::unprotect(x);
    }
}

/// ThreadLocalArena: protect N distinct values
#[divan::bench(args = [10, 100, 1000])]
fn thread_local_multiple(n: usize) {
    unsafe {
        for i in 0..n {
            ThreadLocalArena::protect(raw_ffi::Rf_ScalarInteger(i as i32));
        }
        divan::black_box(ThreadLocalArena::len());
        ThreadLocalArena::clear();
    }
}

/// ThreadLocalArena: protect same value N times (ref count)
#[divan::bench(args = [10, 100, 1000])]
fn thread_local_same_value(n: usize) {
    unsafe {
        let x = raw_ffi::Rf_ScalarInteger(42);
        for _ in 0..n {
            ThreadLocalArena::protect(x);
        }
        divan::black_box(ThreadLocalArena::ref_count(x));
        ThreadLocalArena::clear();
    }
}

/// ThreadLocalArena: protect then unprotect N values
#[divan::bench(args = [10, 100, 1000])]
fn thread_local_protect_unprotect(n: usize) {
    unsafe {
        let mut values = Vec::with_capacity(n);

        for i in 0..n {
            values.push(ThreadLocalArena::protect(raw_ffi::Rf_ScalarInteger(
                i as i32,
            )));
        }

        for x in values.into_iter().rev() {
            ThreadLocalArena::unprotect(x);
        }

        divan::black_box(ThreadLocalArena::is_empty());
    }
}

/// ThreadLocalArena: many values stress test
#[divan::bench(args = [1000, 5000, 10000])]
fn thread_local_many(n: usize) {
    unsafe {
        for i in 0..n {
            ThreadLocalArena::protect(raw_ffi::Rf_ScalarInteger((i % 100) as i32));
        }
        divan::black_box(ThreadLocalArena::len());
        ThreadLocalArena::clear();
    }
}
// endregion

// region: R ppsize range benchmarks (min=10000, default=50000, max=500000)
// These test the arena implementations at R's --max-ppsize boundaries.
// ProtectScope is limited by ppsize, arenas are not.
//
// Test ranges:
// - min*i (i=1..5): 10000, 20000, 30000, 40000, 50000
// - default+min*i (i=1..5): 60000, 70000, 80000, 90000, 100000
// - max: 500000
//
// NOTE: Benchmarks run alphabetically. ProtectScope tests are prefixed with
// "aaa_" to run first before other benchmarks consume protect stack space.

/// ProtectScope at ppsize boundaries (runs first via "aaa_" prefix)
/// Tests how much of the default 50000 ppsize is actually available.
/// R initialization uses ~30-40 protect slots, so max available is ~49960.
#[divan::bench(args = [10000, 20000, 30000, 40000, 49000, 49500, 49900])]
fn aaa_ppsize_protect_scope(n: usize) {
    unsafe {
        let scope = ProtectScope::new();
        for i in 0..n {
            let _ = scope.protect(raw_ffi::Rf_ScalarInteger((i % 1000) as i32));
        }
        divan::black_box(scope.count());
    }
}

/// RefCountedArena: fine-grained ppsize testing (BTreeMap + RefCell)
#[divan::bench(args = [10000, 20000, 30000, 40000, 50000, 60000, 70000, 80000, 90000, 100000, 200000, 300000, 400000, 500000])]
fn ppsize_refcount_arena(n: usize) {
    unsafe {
        let arena = RefCountedArena::new();
        for i in 0..n {
            arena.protect(raw_ffi::Rf_ScalarInteger((i % 1000) as i32));
        }
        divan::black_box(arena.len());
    }
}

/// ThreadLocalArena: fine-grained ppsize testing (BTreeMap + thread_local)
#[divan::bench(args = [10000, 20000, 30000, 40000, 50000, 60000, 70000, 80000, 90000, 100000, 200000, 300000, 400000, 500000])]
fn ppsize_thread_local(n: usize) {
    unsafe {
        for i in 0..n {
            ThreadLocalArena::protect(raw_ffi::Rf_ScalarInteger((i % 1000) as i32));
        }
        divan::black_box(ThreadLocalArena::len());
        ThreadLocalArena::clear();
    }
}

// endregion

// region: Fast API benchmarks (skip init check)

/// ThreadLocalArena: protect_fast (no init check) vs protect
#[divan::bench(args = [10, 100, 1000])]
fn thread_local_protect_fast(n: usize) {
    unsafe {
        ThreadLocalArena::init();
        for i in 0..n {
            ThreadLocalArena::protect_fast(raw_ffi::Rf_ScalarInteger(i as i32));
        }
        divan::black_box(ThreadLocalArena::len());
        ThreadLocalArena::clear();
    }
}

/// ThreadLocalArena: protect_fast + unprotect_fast cycle
#[divan::bench(args = [10, 100, 1000])]
fn thread_local_fast_cycle(n: usize) {
    unsafe {
        ThreadLocalArena::init();
        let mut values = Vec::with_capacity(n);

        for i in 0..n {
            values.push(ThreadLocalArena::protect_fast(raw_ffi::Rf_ScalarInteger(
                i as i32,
            )));
        }

        for x in values.into_iter().rev() {
            ThreadLocalArena::unprotect_fast(x);
        }

        divan::black_box(ThreadLocalArena::is_empty());
    }
}

// endregion

// region: init_with_capacity benchmarks

/// ThreadLocalArena: default init vs init_with_capacity (10000)
#[divan::bench(args = [100, 1000, 10000])]
fn thread_local_init_with_capacity(n: usize) {
    unsafe {
        ThreadLocalArena::init_with_capacity(n);
        for i in 0..n {
            ThreadLocalArena::protect_fast(raw_ffi::Rf_ScalarInteger(i as i32));
        }
        divan::black_box(ThreadLocalArena::len());
        ThreadLocalArena::clear();
    }
}

/// RefCountedArena: with_capacity vs default
#[divan::bench(args = [100, 1000, 10000])]
fn refcount_arena_with_capacity(n: usize) {
    unsafe {
        let arena = RefCountedArena::with_capacity(n);
        for i in 0..n {
            arena.protect(raw_ffi::Rf_ScalarInteger(i as i32));
        }
        divan::black_box(arena.len());
    }
}
// endregion
