//! Benchmarks for RefCountedArena vs ProtectScope.
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
    HashMapArena, RefCountedArena, ThreadLocalArena, ThreadLocalHashArena,
};

fn main() {
    miniextendr_bench::init();
    divan::main();
}

// =============================================================================
// Single value protection
// =============================================================================

/// ProtectScope: protect single value
#[divan::bench]
fn protect_scope_single() {
    unsafe {
        let scope = ProtectScope::new();
        let x = scope.protect(ffi::Rf_ScalarInteger(42));
        divan::black_box(x.get());
    }
}

/// RefCountedArena: protect single value
#[divan::bench]
fn refcount_arena_single() {
    unsafe {
        let arena = RefCountedArena::new();
        let x = arena.protect(ffi::Rf_ScalarInteger(42));
        divan::black_box(x);
    }
}

/// RefCountedArena with guard: protect single value
#[divan::bench]
fn refcount_arena_guard_single() {
    unsafe {
        let arena = RefCountedArena::new();
        let guard = arena.guard(ffi::Rf_ScalarInteger(42));
        divan::black_box(guard.get());
    }
}

// =============================================================================
// Multiple value protection
// =============================================================================

/// ProtectScope: protect N values
#[divan::bench(args = [10, 100, 1000])]
fn protect_scope_multiple(n: usize) {
    unsafe {
        let scope = ProtectScope::new();
        for i in 0..n {
            let _ = scope.protect(ffi::Rf_ScalarInteger(i as i32));
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
            arena.protect(ffi::Rf_ScalarInteger(i as i32));
        }
        divan::black_box(arena.len());
    }
}

// =============================================================================
// Reference counting (same value multiple times)
// =============================================================================

/// RefCountedArena: protect same value N times
#[divan::bench(args = [10, 100, 1000])]
fn refcount_arena_same_value(n: usize) {
    unsafe {
        let arena = RefCountedArena::new();
        let x = ffi::Rf_ScalarInteger(42);

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
        let x = ffi::Rf_ScalarInteger(42);

        for _ in 0..n {
            let _ = scope.protect(x);
        }

        divan::black_box(scope.count());
    }
}

// =============================================================================
// Protect + unprotect cycles
// =============================================================================

/// RefCountedArena: protect then unprotect N values
#[divan::bench(args = [10, 100, 1000])]
fn refcount_arena_protect_unprotect(n: usize) {
    unsafe {
        let arena = RefCountedArena::new();
        let mut values = Vec::with_capacity(n);

        // Protect all
        for i in 0..n {
            values.push(arena.protect(ffi::Rf_ScalarInteger(i as i32)));
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
            values.push(arena.protect(ffi::Rf_ScalarInteger(i as i32)));
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

// =============================================================================
// Large scale tests
// =============================================================================

/// RefCountedArena: protect many values (stress test)
#[divan::bench(args = [1000, 5000, 10000])]
fn refcount_arena_many_values(n: usize) {
    unsafe {
        let arena = RefCountedArena::new();

        for i in 0..n {
            arena.protect(ffi::Rf_ScalarInteger((i % 100) as i32));
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
            let _ = scope.protect(ffi::Rf_ScalarInteger((i % 100) as i32));
        }

        divan::black_box(scope.count());
    }
}

// =============================================================================
// Guard vs manual protect/unprotect
// =============================================================================

/// RefCountedArena: guard pattern
#[divan::bench(args = [10, 100])]
fn refcount_arena_guards(n: usize) {
    unsafe {
        let arena = RefCountedArena::new();

        for i in 0..n {
            let _guard = arena.guard(ffi::Rf_ScalarInteger(i as i32));
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
            let x = arena.protect(ffi::Rf_ScalarInteger(i as i32));
            arena.unprotect(x);
        }

        divan::black_box(arena.is_empty());
    }
}

// =============================================================================
// Mixed workload
// =============================================================================

/// RefCountedArena: realistic workload with vectors
#[divan::bench]
fn refcount_arena_realistic() {
    unsafe {
        let arena = RefCountedArena::new();

        // Protect a list
        let list = arena.protect(Rf_allocVector(SEXPTYPE::VECSXP, 10));

        // Protect some children
        for i in 0..10 {
            let child = arena.protect(ffi::Rf_ScalarInteger(i));
            ffi::SET_VECTOR_ELT(list, i as isize, child);
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
            let child = scope.protect_raw(ffi::Rf_ScalarInteger(i));
            ffi::SET_VECTOR_ELT(list, i as isize, child);
        }

        divan::black_box(list);
        // All unprotected on scope drop
    }
}

// =============================================================================
// BTreeMap vs HashMap comparison
// =============================================================================

/// BTreeMap (RefCountedArena): single protect
#[divan::bench]
fn btreemap_single() {
    unsafe {
        let arena = RefCountedArena::new();
        let x = arena.protect(ffi::Rf_ScalarInteger(42));
        divan::black_box(x);
    }
}

/// HashMap (HashMapArena): single protect
#[divan::bench]
fn hashmap_single() {
    unsafe {
        let arena = HashMapArena::new();
        let x = arena.protect(ffi::Rf_ScalarInteger(42));
        divan::black_box(x);
    }
}

/// BTreeMap: protect N distinct values
#[divan::bench(args = [10, 100, 1000])]
fn btreemap_multiple(n: usize) {
    unsafe {
        let arena = RefCountedArena::new();
        for i in 0..n {
            arena.protect(ffi::Rf_ScalarInteger(i as i32));
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
            arena.protect(ffi::Rf_ScalarInteger(i as i32));
        }
        divan::black_box(arena.len());
    }
}

/// BTreeMap: protect same value N times (ref count)
#[divan::bench(args = [10, 100, 1000])]
fn btreemap_same_value(n: usize) {
    unsafe {
        let arena = RefCountedArena::new();
        let x = ffi::Rf_ScalarInteger(42);
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
        let x = ffi::Rf_ScalarInteger(42);
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
            values.push(arena.protect(ffi::Rf_ScalarInteger(i as i32)));
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
            values.push(arena.protect(ffi::Rf_ScalarInteger(i as i32)));
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
            arena.protect(ffi::Rf_ScalarInteger((i % 100) as i32));
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
            arena.protect(ffi::Rf_ScalarInteger((i % 100) as i32));
        }
        divan::black_box(arena.len());
    }
}

// =============================================================================
// ThreadLocalArena benchmarks
// =============================================================================

/// ThreadLocalArena: single protect
#[divan::bench]
fn thread_local_single() {
    unsafe {
        let x = ThreadLocalArena::protect(ffi::Rf_ScalarInteger(42));
        divan::black_box(x);
        ThreadLocalArena::unprotect(x);
    }
}

/// ThreadLocalArena: protect N distinct values
#[divan::bench(args = [10, 100, 1000])]
fn thread_local_multiple(n: usize) {
    unsafe {
        for i in 0..n {
            ThreadLocalArena::protect(ffi::Rf_ScalarInteger(i as i32));
        }
        divan::black_box(ThreadLocalArena::len());
        ThreadLocalArena::clear();
    }
}

/// ThreadLocalArena: protect same value N times (ref count)
#[divan::bench(args = [10, 100, 1000])]
fn thread_local_same_value(n: usize) {
    unsafe {
        let x = ffi::Rf_ScalarInteger(42);
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
            values.push(ThreadLocalArena::protect(ffi::Rf_ScalarInteger(i as i32)));
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
            ThreadLocalArena::protect(ffi::Rf_ScalarInteger((i % 100) as i32));
        }
        divan::black_box(ThreadLocalArena::len());
        ThreadLocalArena::clear();
    }
}

// =============================================================================
// ThreadLocalHashArena benchmarks
// =============================================================================

/// ThreadLocalHashArena: single protect
#[divan::bench]
fn thread_local_hash_single() {
    unsafe {
        let x = ThreadLocalHashArena::protect(ffi::Rf_ScalarInteger(42));
        divan::black_box(x);
        ThreadLocalHashArena::unprotect(x);
    }
}

/// ThreadLocalHashArena: protect N distinct values
#[divan::bench(args = [10, 100, 1000])]
fn thread_local_hash_multiple(n: usize) {
    unsafe {
        for i in 0..n {
            ThreadLocalHashArena::protect(ffi::Rf_ScalarInteger(i as i32));
        }
        divan::black_box(ThreadLocalHashArena::len());
        ThreadLocalHashArena::clear();
    }
}

/// ThreadLocalHashArena: protect same value N times (ref count)
#[divan::bench(args = [10, 100, 1000])]
fn thread_local_hash_same_value(n: usize) {
    unsafe {
        let x = ffi::Rf_ScalarInteger(42);
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
            values.push(ThreadLocalHashArena::protect(ffi::Rf_ScalarInteger(i as i32)));
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
            ThreadLocalHashArena::protect(ffi::Rf_ScalarInteger((i % 100) as i32));
        }
        divan::black_box(ThreadLocalHashArena::len());
        ThreadLocalHashArena::clear();
    }
}

// =============================================================================
// R ppsize range benchmarks (min=10000, default=50000, max=500000)
// =============================================================================
// These test the arena implementations at R's --max-ppsize boundaries.
// ProtectScope is limited by ppsize, arenas are not.

/// ProtectScope at minimum ppsize (10000)
/// Note: Cannot test higher values - would hit protect stack overflow
/// This demonstrates why arenas are needed for large-scale protection
#[divan::bench]
fn ppsize_protect_scope_min() {
    unsafe {
        let scope = ProtectScope::new();
        for i in 0..10000 {
            let _ = scope.protect(ffi::Rf_ScalarInteger((i % 1000) as i32));
        }
        divan::black_box(scope.count());
    }
}

/// RefCountedArena at ppsize boundaries (no limit)
#[divan::bench(args = [10000, 50000, 500000])]
fn ppsize_refcount_arena(n: usize) {
    unsafe {
        let arena = RefCountedArena::new();
        for i in 0..n {
            arena.protect(ffi::Rf_ScalarInteger((i % 1000) as i32));
        }
        divan::black_box(arena.len());
    }
}

/// HashMapArena at ppsize boundaries (no limit)
#[divan::bench(args = [10000, 50000, 500000])]
fn ppsize_hashmap_arena(n: usize) {
    unsafe {
        let arena = HashMapArena::new();
        for i in 0..n {
            arena.protect(ffi::Rf_ScalarInteger((i % 1000) as i32));
        }
        divan::black_box(arena.len());
    }
}

/// ThreadLocalArena at ppsize boundaries (no limit)
#[divan::bench(args = [10000, 50000, 500000])]
fn ppsize_thread_local(n: usize) {
    unsafe {
        for i in 0..n {
            ThreadLocalArena::protect(ffi::Rf_ScalarInteger((i % 1000) as i32));
        }
        divan::black_box(ThreadLocalArena::len());
        ThreadLocalArena::clear();
    }
}

/// ThreadLocalHashArena at ppsize boundaries (no limit)
#[divan::bench(args = [10000, 50000, 500000])]
fn ppsize_thread_local_hash(n: usize) {
    unsafe {
        for i in 0..n {
            ThreadLocalHashArena::protect(ffi::Rf_ScalarInteger((i % 1000) as i32));
        }
        divan::black_box(ThreadLocalHashArena::len());
        ThreadLocalHashArena::clear();
    }
}
