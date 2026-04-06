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

use miniextendr_api::ffi::{self, Rf_allocVector, SEXPTYPE};
use miniextendr_api::gc_protect::ProtectScope;
use miniextendr_api::refcount_protect::{
    HashMapArena, RefCountedArena, ThreadLocalArena, ThreadLocalArenaOps, ThreadLocalHashArena,
};
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
        ffi::R_PreserveObject_unchecked(x);
        ffi::R_ReleaseObject_unchecked(x);
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
            ffi::R_PreserveObject_unchecked(x);
            values.push(x);
        }
        for x in values {
            ffi::R_ReleaseObject_unchecked(x);
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
            ffi::R_PreserveObject_unchecked(x);
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
            ffi::R_PreserveObject_unchecked(x);
            values.push(x);
        }
        for x in values {
            ffi::R_ReleaseObject_unchecked(x);
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

// region: BTreeMap vs HashMap comparison

/// BTreeMap (RefCountedArena): single protect
#[divan::bench]
fn btreemap_single() {
    unsafe {
        let arena = RefCountedArena::new();
        let x = arena.protect(raw_ffi::Rf_ScalarInteger(42));
        divan::black_box(x);
    }
}

/// HashMap (HashMapArena): single protect
#[divan::bench]
fn hashmap_single() {
    unsafe {
        let arena = HashMapArena::new();
        let x = arena.protect(raw_ffi::Rf_ScalarInteger(42));
        divan::black_box(x);
    }
}

/// BTreeMap: protect N distinct values
#[divan::bench(args = [10, 100, 1000])]
fn btreemap_multiple(n: usize) {
    unsafe {
        let arena = RefCountedArena::new();
        for i in 0..n {
            arena.protect(raw_ffi::Rf_ScalarInteger(i as i32));
        }
        divan::black_box(arena.len());
    }
}

/// HashMap: protect N distinct values
#[divan::bench(args = [10, 100, 1000])]
fn hashmap_multiple(n: usize) {
    unsafe {
        let arena = HashMapArena::new();
        for i in 0..n {
            arena.protect(raw_ffi::Rf_ScalarInteger(i as i32));
        }
        divan::black_box(arena.len());
    }
}

/// BTreeMap: protect same value N times (ref count)
#[divan::bench(args = [10, 100, 1000])]
fn btreemap_same_value(n: usize) {
    unsafe {
        let arena = RefCountedArena::new();
        let x = raw_ffi::Rf_ScalarInteger(42);
        for _ in 0..n {
            arena.protect(x);
        }
        divan::black_box(arena.ref_count(x));
    }
}

/// HashMap: protect same value N times (ref count)
#[divan::bench(args = [10, 100, 1000])]
fn hashmap_same_value(n: usize) {
    unsafe {
        let arena = HashMapArena::new();
        let x = raw_ffi::Rf_ScalarInteger(42);
        for _ in 0..n {
            arena.protect(x);
        }
        divan::black_box(arena.ref_count(x));
    }
}

/// BTreeMap: protect then unprotect N values
#[divan::bench(args = [10, 100, 1000])]
fn btreemap_protect_unprotect(n: usize) {
    unsafe {
        let arena = RefCountedArena::new();
        let mut values = Vec::with_capacity(n);

        for i in 0..n {
            values.push(arena.protect(raw_ffi::Rf_ScalarInteger(i as i32)));
        }

        for x in values.into_iter().rev() {
            arena.unprotect(x);
        }

        divan::black_box(arena.is_empty());
    }
}

/// HashMap: protect then unprotect N values
#[divan::bench(args = [10, 100, 1000])]
fn hashmap_protect_unprotect(n: usize) {
    unsafe {
        let arena = HashMapArena::new();
        let mut values = Vec::with_capacity(n);

        for i in 0..n {
            values.push(arena.protect(raw_ffi::Rf_ScalarInteger(i as i32)));
        }

        for x in values.into_iter().rev() {
            arena.unprotect(x);
        }

        divan::black_box(arena.is_empty());
    }
}

/// BTreeMap: many values stress test
#[divan::bench(args = [1000, 5000, 10000])]
fn btreemap_many(n: usize) {
    unsafe {
        let arena = RefCountedArena::new();
        for i in 0..n {
            arena.protect(raw_ffi::Rf_ScalarInteger((i % 100) as i32));
        }
        divan::black_box(arena.len());
    }
}

/// HashMap: many values stress test
#[divan::bench(args = [1000, 5000, 10000])]
fn hashmap_many(n: usize) {
    unsafe {
        let arena = HashMapArena::new();
        for i in 0..n {
            arena.protect(raw_ffi::Rf_ScalarInteger((i % 100) as i32));
        }
        divan::black_box(arena.len());
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

// region: ThreadLocalHashArena benchmarks

/// ThreadLocalHashArena: single protect
#[divan::bench]
fn thread_local_hash_single() {
    unsafe {
        let x = ThreadLocalHashArena::protect(raw_ffi::Rf_ScalarInteger(42));
        divan::black_box(x);
        ThreadLocalHashArena::unprotect(x);
    }
}

/// ThreadLocalHashArena: protect N distinct values
#[divan::bench(args = [10, 100, 1000])]
fn thread_local_hash_multiple(n: usize) {
    unsafe {
        for i in 0..n {
            ThreadLocalHashArena::protect(raw_ffi::Rf_ScalarInteger(i as i32));
        }
        divan::black_box(ThreadLocalHashArena::len());
        ThreadLocalHashArena::clear();
    }
}

/// ThreadLocalHashArena: protect same value N times (ref count)
#[divan::bench(args = [10, 100, 1000])]
fn thread_local_hash_same_value(n: usize) {
    unsafe {
        let x = raw_ffi::Rf_ScalarInteger(42);
        for _ in 0..n {
            ThreadLocalHashArena::protect(x);
        }
        divan::black_box(ThreadLocalHashArena::ref_count(x));
        ThreadLocalHashArena::clear();
    }
}

/// ThreadLocalHashArena: protect then unprotect N values
#[divan::bench(args = [10, 100, 1000])]
fn thread_local_hash_protect_unprotect(n: usize) {
    unsafe {
        let mut values = Vec::with_capacity(n);

        for i in 0..n {
            values.push(ThreadLocalHashArena::protect(raw_ffi::Rf_ScalarInteger(
                i as i32,
            )));
        }

        for x in values.into_iter().rev() {
            ThreadLocalHashArena::unprotect(x);
        }

        divan::black_box(ThreadLocalHashArena::is_empty());
    }
}

/// ThreadLocalHashArena: many values stress test
#[divan::bench(args = [1000, 5000, 10000])]
fn thread_local_hash_many(n: usize) {
    unsafe {
        for i in 0..n {
            ThreadLocalHashArena::protect(raw_ffi::Rf_ScalarInteger((i % 100) as i32));
        }
        divan::black_box(ThreadLocalHashArena::len());
        ThreadLocalHashArena::clear();
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

/// HashMapArena: fine-grained ppsize testing (HashMap + RefCell)
#[divan::bench(args = [10000, 20000, 30000, 40000, 50000, 60000, 70000, 80000, 90000, 100000, 200000, 300000, 400000, 500000])]
fn ppsize_hashmap_arena(n: usize) {
    unsafe {
        let arena = HashMapArena::new();
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

/// ThreadLocalHashArena: fine-grained ppsize testing (HashMap + thread_local)
#[divan::bench(args = [10000, 20000, 30000, 40000, 50000, 60000, 70000, 80000, 90000, 100000, 200000, 300000, 400000, 500000])]
fn ppsize_thread_local_hash(n: usize) {
    unsafe {
        for i in 0..n {
            ThreadLocalHashArena::protect(raw_ffi::Rf_ScalarInteger((i % 1000) as i32));
        }
        divan::black_box(ThreadLocalHashArena::len());
        ThreadLocalHashArena::clear();
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

/// ThreadLocalHashArena: protect_fast (no init check) vs protect
#[divan::bench(args = [10, 100, 1000])]
fn thread_local_hash_protect_fast(n: usize) {
    unsafe {
        ThreadLocalHashArena::init();
        for i in 0..n {
            ThreadLocalHashArena::protect_fast(raw_ffi::Rf_ScalarInteger(i as i32));
        }
        divan::black_box(ThreadLocalHashArena::len());
        ThreadLocalHashArena::clear();
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

/// ThreadLocalHashArena: protect_fast + unprotect_fast cycle
#[divan::bench(args = [10, 100, 1000])]
fn thread_local_hash_fast_cycle(n: usize) {
    unsafe {
        ThreadLocalHashArena::init();
        let mut values = Vec::with_capacity(n);

        for i in 0..n {
            values.push(ThreadLocalHashArena::protect_fast(
                raw_ffi::Rf_ScalarInteger(i as i32),
            ));
        }

        for x in values.into_iter().rev() {
            ThreadLocalHashArena::unprotect_fast(x);
        }

        divan::black_box(ThreadLocalHashArena::is_empty());
    }
}
// endregion

// region: init_with_capacity benchmarks

/// ThreadLocalArena: default init vs init_with_capacity (10000)
#[divan::bench(args = [100, 1000, 10000])]
fn thread_local_init_with_capacity(n: usize) {
    unsafe {
        ThreadLocalHashArena::init_with_capacity(n);
        for i in 0..n {
            ThreadLocalHashArena::protect_fast(raw_ffi::Rf_ScalarInteger(i as i32));
        }
        divan::black_box(ThreadLocalHashArena::len());
        ThreadLocalHashArena::clear();
    }
}

/// HashMapArena: with_capacity vs default
#[divan::bench(args = [100, 1000, 10000])]
fn hashmap_with_capacity(n: usize) {
    unsafe {
        let arena = HashMapArena::with_capacity(n);
        for i in 0..n {
            arena.protect(raw_ffi::Rf_ScalarInteger(i as i32));
        }
        divan::black_box(arena.len());
    }
}
// endregion

// region: Fast hash arena benchmarks (feature-gated)

#[cfg(feature = "refcount-fast-hash")]
mod fast_hash_benches {
    use super::*;
    use miniextendr_api::refcount_protect::{
        FastHashMapArena, ThreadLocalArenaOps, ThreadLocalFastHashArena,
    };

    /// FastHashMapArena: single protect
    #[divan::bench]
    fn fast_hash_single() {
        unsafe {
            let arena = FastHashMapArena::new();
            let x = arena.protect(raw_ffi::Rf_ScalarInteger(42));
            divan::black_box(x);
        }
    }

    /// FastHashMapArena: protect N distinct values
    #[divan::bench(args = [10, 100, 1000])]
    fn fast_hash_multiple(n: usize) {
        unsafe {
            let arena = FastHashMapArena::new();
            for i in 0..n {
                arena.protect(raw_ffi::Rf_ScalarInteger(i as i32));
            }
            divan::black_box(arena.len());
        }
    }

    /// FastHashMapArena: protect same value N times (ref count)
    #[divan::bench(args = [10, 100, 1000])]
    fn fast_hash_same_value(n: usize) {
        unsafe {
            let arena = FastHashMapArena::new();
            let x = raw_ffi::Rf_ScalarInteger(42);
            for _ in 0..n {
                arena.protect(x);
            }
            divan::black_box(arena.ref_count(x));
        }
    }

    /// FastHashMapArena: protect then unprotect N values
    #[divan::bench(args = [10, 100, 1000])]
    fn fast_hash_protect_unprotect(n: usize) {
        unsafe {
            let arena = FastHashMapArena::new();
            let mut values = Vec::with_capacity(n);

            for i in 0..n {
                values.push(arena.protect(raw_ffi::Rf_ScalarInteger(i as i32)));
            }

            for x in values.into_iter().rev() {
                arena.unprotect(x);
            }

            divan::black_box(arena.is_empty());
        }
    }

    /// FastHashMapArena: many values stress test
    #[divan::bench(args = [1000, 5000, 10000])]
    fn fast_hash_many(n: usize) {
        unsafe {
            let arena = FastHashMapArena::new();
            for i in 0..n {
                arena.protect(raw_ffi::Rf_ScalarInteger((i % 100) as i32));
            }
            divan::black_box(arena.len());
        }
    }

    /// FastHashMapArena: fine-grained ppsize testing
    #[divan::bench(args = [10000, 20000, 30000, 40000, 50000, 60000, 70000, 80000, 90000, 100000, 200000, 300000, 400000, 500000])]
    fn ppsize_fast_hash(n: usize) {
        unsafe {
            let arena = FastHashMapArena::new();
            for i in 0..n {
                arena.protect(raw_ffi::Rf_ScalarInteger((i % 1000) as i32));
            }
            divan::black_box(arena.len());
        }
    }

    /// ThreadLocalFastHashArena: single protect
    #[divan::bench]
    fn thread_local_fast_hash_single() {
        unsafe {
            let x = ThreadLocalFastHashArena::protect(raw_ffi::Rf_ScalarInteger(42));
            divan::black_box(x);
            ThreadLocalFastHashArena::unprotect(x);
        }
    }

    /// ThreadLocalFastHashArena: protect N distinct values
    #[divan::bench(args = [10, 100, 1000])]
    fn thread_local_fast_hash_multiple(n: usize) {
        unsafe {
            for i in 0..n {
                ThreadLocalFastHashArena::protect(raw_ffi::Rf_ScalarInteger(i as i32));
            }
            divan::black_box(ThreadLocalFastHashArena::len());
            ThreadLocalFastHashArena::clear();
        }
    }

    /// ThreadLocalFastHashArena: protect_fast (no init check)
    #[divan::bench(args = [10, 100, 1000])]
    fn thread_local_fast_hash_protect_fast(n: usize) {
        unsafe {
            ThreadLocalFastHashArena::init();
            for i in 0..n {
                ThreadLocalFastHashArena::protect_fast(raw_ffi::Rf_ScalarInteger(i as i32));
            }
            divan::black_box(ThreadLocalFastHashArena::len());
            ThreadLocalFastHashArena::clear();
        }
    }

    /// ThreadLocalFastHashArena: fine-grained ppsize testing
    #[divan::bench(args = [10000, 20000, 30000, 40000, 50000, 60000, 70000, 80000, 90000, 100000, 200000, 300000, 400000, 500000])]
    fn ppsize_thread_local_fast_hash(n: usize) {
        unsafe {
            for i in 0..n {
                ThreadLocalFastHashArena::protect(raw_ffi::Rf_ScalarInteger((i % 1000) as i32));
            }
            divan::black_box(ThreadLocalFastHashArena::len());
            ThreadLocalFastHashArena::clear();
        }
    }
}
// endregion
