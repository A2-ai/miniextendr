//! Integration tests for RefCountedArena.

mod r_test_utils;

use miniextendr_api::ffi::{Rf_ScalarInteger, Rf_ScalarReal, Rf_allocVector, SEXPTYPE};
use miniextendr_api::refcount_protect::RefCountedArena;

// =============================================================================
// Basic protection tests
// =============================================================================

#[test]
fn protect_single_value() {
    r_test_utils::with_r_thread(|| unsafe {
        let arena = RefCountedArena::new();

        let x = Rf_ScalarInteger(42);
        let protected = arena.protect(x);

        assert!(std::ptr::eq(x.0, protected.0));
        assert!(arena.is_protected(x));
        assert_eq!(arena.ref_count(x), 1);
        assert_eq!(arena.len(), 1);
    });
}

#[test]
fn protect_multiple_values() {
    r_test_utils::with_r_thread(|| unsafe {
        let arena = RefCountedArena::new();

        let a = arena.protect(Rf_ScalarInteger(1));
        let b = arena.protect(Rf_ScalarReal(2.0));
        let c = arena.protect(Rf_ScalarInteger(3));

        assert!(arena.is_protected(a));
        assert!(arena.is_protected(b));
        assert!(arena.is_protected(c));
        assert_eq!(arena.len(), 3);
    });
}

#[test]
fn unprotect_removes_value() {
    r_test_utils::with_r_thread(|| unsafe {
        let arena = RefCountedArena::new();

        let x = arena.protect(Rf_ScalarInteger(42));
        assert!(arena.is_protected(x));
        assert_eq!(arena.len(), 1);

        arena.unprotect(x);
        assert!(!arena.is_protected(x));
        assert_eq!(arena.len(), 0);
    });
}

// =============================================================================
// Reference counting tests
// =============================================================================

#[test]
fn protect_same_value_increments_count() {
    r_test_utils::with_r_thread(|| unsafe {
        let arena = RefCountedArena::new();

        let x = Rf_ScalarInteger(42);
        arena.protect(x);
        assert_eq!(arena.ref_count(x), 1);

        arena.protect(x);
        assert_eq!(arena.ref_count(x), 2);

        arena.protect(x);
        assert_eq!(arena.ref_count(x), 3);

        // Still only one unique SEXP
        assert_eq!(arena.len(), 1);
    });
}

#[test]
fn unprotect_decrements_count() {
    r_test_utils::with_r_thread(|| unsafe {
        let arena = RefCountedArena::new();

        let x = Rf_ScalarInteger(42);
        arena.protect(x);
        arena.protect(x);
        arena.protect(x);
        assert_eq!(arena.ref_count(x), 3);

        arena.unprotect(x);
        assert_eq!(arena.ref_count(x), 2);
        assert!(arena.is_protected(x));

        arena.unprotect(x);
        assert_eq!(arena.ref_count(x), 1);
        assert!(arena.is_protected(x));

        arena.unprotect(x);
        assert_eq!(arena.ref_count(x), 0);
        assert!(!arena.is_protected(x));
    });
}

// =============================================================================
// Release order tests
// =============================================================================

#[test]
fn release_in_any_order() {
    r_test_utils::with_r_thread(|| unsafe {
        let arena = RefCountedArena::new();

        let a = arena.protect(Rf_ScalarInteger(1));
        let b = arena.protect(Rf_ScalarInteger(2));
        let c = arena.protect(Rf_ScalarInteger(3));

        // Release in different order than protection
        arena.unprotect(b); // middle first
        assert!(!arena.is_protected(b));
        assert!(arena.is_protected(a));
        assert!(arena.is_protected(c));

        arena.unprotect(a); // first
        assert!(!arena.is_protected(a));
        assert!(arena.is_protected(c));

        arena.unprotect(c); // last
        assert!(!arena.is_protected(c));

        assert!(arena.is_empty());
    });
}

// =============================================================================
// try_unprotect tests
// =============================================================================

#[test]
fn try_unprotect_returns_true_for_protected() {
    r_test_utils::with_r_thread(|| unsafe {
        let arena = RefCountedArena::new();

        let x = arena.protect(Rf_ScalarInteger(42));
        assert!(arena.try_unprotect(x));
        assert!(!arena.is_protected(x));
    });
}

#[test]
fn try_unprotect_returns_false_for_unprotected() {
    r_test_utils::with_r_thread(|| unsafe {
        let arena = RefCountedArena::new();

        let x = Rf_ScalarInteger(42);
        assert!(!arena.try_unprotect(x));
    });
}

// =============================================================================
// Guard tests
// =============================================================================

#[test]
fn guard_protects_and_unprotects() {
    r_test_utils::with_r_thread(|| unsafe {
        let arena = RefCountedArena::new();

        let x = Rf_ScalarInteger(42);

        {
            let _guard = arena.guard(x);
            assert!(arena.is_protected(x));
            assert_eq!(arena.ref_count(x), 1);
        } // guard drops here

        assert!(!arena.is_protected(x));
        assert_eq!(arena.ref_count(x), 0);
    });
}

#[test]
fn multiple_guards_for_same_value() {
    r_test_utils::with_r_thread(|| unsafe {
        let arena = RefCountedArena::new();

        let x = Rf_ScalarInteger(42);

        let g1 = arena.guard(x);
        assert_eq!(arena.ref_count(x), 1);

        {
            let _g2 = arena.guard(x);
            assert_eq!(arena.ref_count(x), 2);
        } // g2 drops

        assert_eq!(arena.ref_count(x), 1);
        assert!(arena.is_protected(x));

        drop(g1);
        assert!(!arena.is_protected(x));
    });
}

// =============================================================================
// Capacity and growth tests
// =============================================================================

#[test]
fn arena_grows_when_full() {
    r_test_utils::with_r_thread(|| unsafe {
        let arena = RefCountedArena::with_capacity(4);
        let initial_capacity = arena.capacity();
        assert_eq!(initial_capacity, 4);

        // Fill beyond capacity
        for i in 0..10 {
            arena.protect(Rf_ScalarInteger(i));
        }

        assert_eq!(arena.len(), 10);
        assert!(arena.capacity() > initial_capacity);
    });
}

#[test]
fn free_slots_are_reused() {
    r_test_utils::with_r_thread(|| unsafe {
        let arena = RefCountedArena::with_capacity(4);

        let a = arena.protect(Rf_ScalarInteger(1));
        let b = arena.protect(Rf_ScalarInteger(2));
        let c = arena.protect(Rf_ScalarInteger(3));
        let d = arena.protect(Rf_ScalarInteger(4));
        assert_eq!(arena.len(), 4);
        assert_eq!(arena.capacity(), 4);

        // Unprotect b and c, creating free slots
        arena.unprotect(b);
        arena.unprotect(c);
        assert_eq!(arena.len(), 2);

        // Add new values - should reuse free slots, not grow
        arena.protect(Rf_ScalarInteger(5));
        arena.protect(Rf_ScalarInteger(6));
        assert_eq!(arena.len(), 4);
        assert_eq!(arena.capacity(), 4); // didn't grow

        // Clean up
        arena.unprotect(a);
        arena.unprotect(d);
    });
}

// =============================================================================
// Clear test
// =============================================================================

#[test]
fn clear_removes_all_protections() {
    r_test_utils::with_r_thread(|| unsafe {
        let arena = RefCountedArena::new();

        let a = arena.protect(Rf_ScalarInteger(1));
        let b = arena.protect(Rf_ScalarInteger(2));
        let c = arena.protect(Rf_ScalarInteger(3));

        assert_eq!(arena.len(), 3);

        arena.clear();

        assert!(arena.is_empty());
        assert!(!arena.is_protected(a));
        assert!(!arena.is_protected(b));
        assert!(!arena.is_protected(c));
    });
}

// =============================================================================
// Large scale test
// =============================================================================

#[test]
fn many_protections() {
    r_test_utils::with_r_thread(|| unsafe {
        let arena = RefCountedArena::new();

        // Protect 1000 values
        let values: Vec<_> = (0..1000)
            .map(|i| arena.protect(Rf_ScalarInteger(i)))
            .collect();

        assert_eq!(arena.len(), 1000);

        // Unprotect in random-ish order
        for i in (0..1000).step_by(3) {
            arena.unprotect(values[i]);
        }

        // Some still protected
        assert!(arena.len() < 1000);
        assert!(!arena.is_empty());

        // Clear the rest
        arena.clear();
        assert!(arena.is_empty());
    });
}

// =============================================================================
// VECSXP allocation test
// =============================================================================

#[test]
fn protect_vecsxp() {
    r_test_utils::with_r_thread(|| unsafe {
        let arena = RefCountedArena::new();

        let vec = arena.protect(Rf_allocVector(SEXPTYPE::VECSXP, 100));
        assert!(arena.is_protected(vec));

        // The protected VECSXP survives GC triggers
        for _ in 0..10 {
            let _ = Rf_allocVector(SEXPTYPE::REALSXP, 10000);
        }

        assert!(arena.is_protected(vec));
        arena.unprotect(vec);
    });
}
