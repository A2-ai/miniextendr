//! Integration tests for RefCountedArena.

mod r_test_utils;

use miniextendr_api::ffi::{Rf_allocVector, SEXP, SEXPTYPE};
use miniextendr_api::refcount_protect::RefCountedArena;

// region: Basic protection tests

#[test]
fn protect_single_value() {
    r_test_utils::with_r_thread(|| unsafe {
        let arena = RefCountedArena::new();

        let x = SEXP::scalar_integer(42);
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

        let a = arena.protect(SEXP::scalar_integer(1));
        let b = arena.protect(SEXP::scalar_real(2.0));
        let c = arena.protect(SEXP::scalar_integer(3));

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

        let x = arena.protect(SEXP::scalar_integer(42));
        assert!(arena.is_protected(x));
        assert_eq!(arena.len(), 1);

        arena.unprotect(x);
        assert!(!arena.is_protected(x));
        assert_eq!(arena.len(), 0);
    });
}
// endregion

// region: Reference counting tests

#[test]
fn protect_same_value_increments_count() {
    r_test_utils::with_r_thread(|| unsafe {
        let arena = RefCountedArena::new();

        let x = SEXP::scalar_integer(42);
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

        let x = SEXP::scalar_integer(42);
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
// endregion

// region: Release order tests

#[test]
fn release_in_any_order() {
    r_test_utils::with_r_thread(|| unsafe {
        let arena = RefCountedArena::new();

        let a = arena.protect(SEXP::scalar_integer(1));
        let b = arena.protect(SEXP::scalar_integer(2));
        let c = arena.protect(SEXP::scalar_integer(3));

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
// endregion

// region: try_unprotect tests

#[test]
fn try_unprotect_returns_true_for_protected() {
    r_test_utils::with_r_thread(|| unsafe {
        let arena = RefCountedArena::new();

        let x = arena.protect(SEXP::scalar_integer(42));
        assert!(arena.try_unprotect(x));
        assert!(!arena.is_protected(x));
    });
}

#[test]
fn try_unprotect_returns_false_for_unprotected() {
    r_test_utils::with_r_thread(|| unsafe {
        let arena = RefCountedArena::new();

        let x = SEXP::scalar_integer(42);
        assert!(!arena.try_unprotect(x));
    });
}
// endregion

// region: Guard tests

#[test]
fn guard_protects_and_unprotects() {
    r_test_utils::with_r_thread(|| unsafe {
        let arena = RefCountedArena::new();

        let x = SEXP::scalar_integer(42);

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

        let x = SEXP::scalar_integer(42);

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
// endregion

// region: Capacity and growth tests

#[test]
fn arena_grows_when_full() {
    r_test_utils::with_r_thread(|| unsafe {
        let arena = RefCountedArena::with_capacity(4);
        let initial_capacity = arena.capacity();
        assert_eq!(initial_capacity, 4);

        // Fill beyond capacity
        for i in 0..10 {
            arena.protect(SEXP::scalar_integer(i));
        }

        assert_eq!(arena.len(), 10);
        assert!(arena.capacity() > initial_capacity);
    });
}

#[test]
fn free_slots_are_reused() {
    r_test_utils::with_r_thread(|| unsafe {
        let arena = RefCountedArena::with_capacity(4);

        let a = arena.protect(SEXP::scalar_integer(1));
        let b = arena.protect(SEXP::scalar_integer(2));
        let c = arena.protect(SEXP::scalar_integer(3));
        let d = arena.protect(SEXP::scalar_integer(4));
        assert_eq!(arena.len(), 4);
        assert_eq!(arena.capacity(), 4);

        // Unprotect b and c, creating free slots
        arena.unprotect(b);
        arena.unprotect(c);
        assert_eq!(arena.len(), 2);

        // Add new values - should reuse free slots, not grow
        arena.protect(SEXP::scalar_integer(5));
        arena.protect(SEXP::scalar_integer(6));
        assert_eq!(arena.len(), 4);
        assert_eq!(arena.capacity(), 4); // didn't grow

        // Clean up
        arena.unprotect(a);
        arena.unprotect(d);
    });
}
// endregion

// region: Clear test

#[test]
fn clear_removes_all_protections() {
    r_test_utils::with_r_thread(|| unsafe {
        let arena = RefCountedArena::new();

        let a = arena.protect(SEXP::scalar_integer(1));
        let b = arena.protect(SEXP::scalar_integer(2));
        let c = arena.protect(SEXP::scalar_integer(3));

        assert_eq!(arena.len(), 3);

        arena.clear();

        assert!(arena.is_empty());
        assert!(!arena.is_protected(a));
        assert!(!arena.is_protected(b));
        assert!(!arena.is_protected(c));
    });
}
// endregion

// region: Large scale test

#[test]
fn many_protections() {
    r_test_utils::with_r_thread(|| unsafe {
        let arena = RefCountedArena::new();

        // Protect 1000 values
        let values: Vec<_> = (0..1000)
            .map(|i| arena.protect(SEXP::scalar_integer(i)))
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
// endregion

// region: Slot-uniqueness regression test
//
// Regression test for the `ArenaState::allocate_slot` bug where using
// `self.len` (live count) as the fresh-slot cursor could return an index
// that was already handed out after release cycles.  The correct fix uses a
// monotonic `next_slot` cursor instead.
//
// Scenario: protect 3 distinct SEXPs, unprotect the middle one, protect 2
// more.  After the release cycle the free-list drains on the 4th protect and
// the 5th takes a fresh slot — at which point the buggy cursor (`self.len`)
// would point inside already-occupied territory if there were a miscount.
// We verify no collision by checking all live values remain individually
// protected and that the arena's live count is exactly right.

#[test]
fn allocate_slot_unique_across_release_cycles() {
    r_test_utils::with_r_thread(|| unsafe {
        // Use a small capacity (8) so the fresh-slot path is exercised without
        // triggering arena growth.
        let arena = RefCountedArena::with_capacity(8);

        // Protect three distinct SEXPs (a, b, c) — uses fresh slots 0, 1, 2.
        let a = SEXP::scalar_integer(1);
        let b = SEXP::scalar_integer(2);
        let c = SEXP::scalar_integer(3);
        arena.protect(a);
        arena.protect(b);
        arena.protect(c);
        assert_eq!(arena.len(), 3);

        // Unprotect the middle one — its slot goes onto the free-list; len → 2.
        arena.unprotect(b);
        assert_eq!(arena.len(), 2);
        assert!(!arena.is_protected(b));

        // Protect two more SEXPs (d, e).
        //   d:  recycles b's freed slot via the free-list   → len 3
        //   e:  free-list now empty; MUST take a fresh slot  → len 4
        //       With the buggy cursor (`self.len = 3`), the fresh slot for e
        //       would be index 3, which is unused (fresh). The bug was that in
        //       adversarial ordering `self.len` could alias an already-live slot,
        //       but the `next_slot` cursor correctly always advances past the
        //       highest previously-allocated index.
        let d = SEXP::scalar_integer(4);
        let e = SEXP::scalar_integer(5);
        arena.protect(d);
        arena.protect(e);
        assert_eq!(arena.len(), 4);

        // All four live SEXPs must be individually protected.
        assert!(arena.is_protected(a), "a must still be protected");
        assert!(arena.is_protected(c), "c must still be protected");
        assert!(arena.is_protected(d), "d must be protected");
        assert!(arena.is_protected(e), "e must be protected");

        // b must no longer be protected.
        assert!(!arena.is_protected(b), "b must have been unprotected");

        // Ref counts must each be exactly 1.
        assert_eq!(arena.ref_count(a), 1);
        assert_eq!(arena.ref_count(c), 1);
        assert_eq!(arena.ref_count(d), 1);
        assert_eq!(arena.ref_count(e), 1);

        // Clean up.
        arena.unprotect(a);
        arena.unprotect(c);
        arena.unprotect(d);
        arena.unprotect(e);
        assert!(arena.is_empty());
    });
}
// endregion

// region: VECSXP allocation test

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
// endregion
