//! Integration tests for GC protection utilities.
//!
//! These tests require R to be initialized and test the actual PROTECT/UNPROTECT
//! behavior. For best results, run with gctorture() enabled to catch imbalances.

mod r_test_utils;

use miniextendr_api::ffi::{Rf_ScalarInteger, Rf_ScalarReal, Rf_allocVector, SEXPTYPE};
use miniextendr_api::gc_protect::{OwnedProtect, ProtectScope, tls};

// =============================================================================
// Balance tests
// =============================================================================

#[test]
fn protect_scope_balance_basic() {
    // Test: protect N values, verify they're all unprotected on scope drop
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();

        let _a = scope.protect(Rf_ScalarInteger(1));
        let _b = scope.protect(Rf_ScalarReal(2.0));
        let _c = scope.protect(Rf_ScalarInteger(3));

        assert_eq!(scope.count(), 3);
        // scope drops here, UNPROTECT(3) called
    });
}

#[test]
fn protect_scope_balance_empty() {
    // Test: empty scope should not unprotect anything
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        assert_eq!(scope.count(), 0);
        // scope drops, UNPROTECT(0) is a no-op
    });
}

#[test]
fn protect_scope_balance_large() {
    // Test: protect many values
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();

        for i in 0..100 {
            let _ = scope.protect(Rf_ScalarInteger(i));
        }

        assert_eq!(scope.count(), 100);
    });
}

// =============================================================================
// Nested scope tests
// =============================================================================

#[test]
fn nested_scopes_independent() {
    // Test: inner scope protections don't affect outer scope count
    r_test_utils::with_r_thread(|| unsafe {
        let outer = ProtectScope::new();
        let _a = outer.protect(Rf_ScalarInteger(1));

        {
            let inner = ProtectScope::new();
            let _b = inner.protect(Rf_ScalarInteger(2));
            let _c = inner.protect(Rf_ScalarInteger(3));

            assert_eq!(inner.count(), 2);
            // inner drops, UNPROTECT(2)
        }

        assert_eq!(outer.count(), 1);
        // outer drops, UNPROTECT(1)
    });
}

#[test]
fn deeply_nested_scopes() {
    r_test_utils::with_r_thread(|| unsafe {
        let s1 = ProtectScope::new();
        let _a = s1.protect(Rf_ScalarInteger(1));

        {
            let s2 = ProtectScope::new();
            let _b = s2.protect(Rf_ScalarInteger(2));

            {
                let s3 = ProtectScope::new();
                let _c = s3.protect(Rf_ScalarInteger(3));
                assert_eq!(s3.count(), 1);
            }

            assert_eq!(s2.count(), 1);
        }

        assert_eq!(s1.count(), 1);
    });
}

// =============================================================================
// Reprotect slot tests
// =============================================================================

#[test]
fn reprotect_slot_count_stays_one() {
    // Test: calling set() multiple times keeps protection count at +1
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();

        let slot = scope.protect_with_index(Rf_ScalarInteger(0));
        assert_eq!(scope.count(), 1);

        // Replace many times
        for i in 1..10 {
            let _ = slot.set(Rf_ScalarInteger(i));
            assert_eq!(scope.count(), 1, "count should stay at 1 after set()");
        }

        assert!(!slot.get().is_null());
    });
}

#[test]
fn reprotect_slot_value_updates() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();

        let initial = Rf_allocVector(SEXPTYPE::REALSXP, 10);
        let slot = scope.protect_with_index(initial);

        // The slot should track the current value
        assert!(std::ptr::eq(slot.get().0, initial.0));

        let updated = Rf_allocVector(SEXPTYPE::INTSXP, 5);
        let root = slot.set(updated);

        assert!(std::ptr::eq(slot.get().0, updated.0));
        assert!(std::ptr::eq(root.get().0, updated.0));
    });
}

// =============================================================================
// Escape hatch tests
// =============================================================================

#[test]
fn disarm_scope_no_unprotect() {
    r_test_utils::with_r_thread(|| unsafe {
        // Create a scope, protect some values, then disarm
        // This would leak if not careful, but tests the escape hatch works
        let scope = ProtectScope::new();
        let _a = scope.protect(Rf_ScalarInteger(1));
        let _b = scope.protect(Rf_ScalarInteger(2));

        assert_eq!(scope.count(), 2);

        // Disarm - scope won't unprotect on drop
        scope.disarm();

        // We need to manually unprotect to avoid leaking
        // (In real code, this would be handled by some other mechanism)
        miniextendr_api::ffi::Rf_unprotect(2);
    });
}

#[test]
fn owned_protect_into_inner_no_unprotect() {
    r_test_utils::with_r_thread(|| unsafe {
        let guard = OwnedProtect::new(Rf_ScalarInteger(42));
        let sexp = guard.into_inner();

        // SEXP is still protected, guard didn't unprotect
        // We need to manually unprotect
        assert!(!sexp.is_null());
        miniextendr_api::ffi::Rf_unprotect(1);
    });
}

// =============================================================================
// TLS convenience tests
// =============================================================================

#[test]
fn tls_with_protect_scope_basic() {
    r_test_utils::with_r_thread(|| unsafe {
        let result = tls::with_protect_scope(|| {
            assert!(tls::has_active_scope());
            assert_eq!(tls::scope_depth(), 1);

            let x = tls::protect(Rf_ScalarInteger(42));
            assert_eq!(tls::current_count(), Some(1));

            x.get()
        });

        assert!(!tls::has_active_scope());
        assert!(!result.is_null());
    });
}

#[test]
fn tls_nested_scopes() {
    r_test_utils::with_r_thread(|| unsafe {
        tls::with_protect_scope(|| {
            assert_eq!(tls::scope_depth(), 1);
            let _a = tls::protect(Rf_ScalarInteger(1));

            tls::with_protect_scope(|| {
                assert_eq!(tls::scope_depth(), 2);
                let _b = tls::protect(Rf_ScalarInteger(2));

                // Inner scope count
                assert_eq!(tls::current_count(), Some(1));
            });

            // Back to outer scope
            assert_eq!(tls::scope_depth(), 1);
            assert_eq!(tls::current_count(), Some(1));
        });

        assert_eq!(tls::scope_depth(), 0);
    });
}

#[test]
fn tls_protect_multiple_values() {
    r_test_utils::with_r_thread(|| unsafe {
        tls::with_protect_scope(|| {
            for i in 0..10 {
                let _ = tls::protect(Rf_ScalarInteger(i));
            }

            assert_eq!(tls::current_count(), Some(10));
        });
    });
}

// =============================================================================
// OwnedProtect tests
// =============================================================================

#[test]
fn owned_protect_basic() {
    r_test_utils::with_r_thread(|| unsafe {
        {
            let guard = OwnedProtect::new(Rf_ScalarInteger(123));
            assert!(!guard.get().is_null());
            // guard drops, UNPROTECT(1) called
        }
    });
}

#[test]
fn owned_protect_deref() {
    r_test_utils::with_r_thread(|| unsafe {
        let guard = OwnedProtect::new(Rf_ScalarReal(std::f64::consts::PI));

        // Deref to get &SEXP
        let sexp: &miniextendr_api::ffi::SEXP = &guard;
        assert!(!sexp.is_null());
    });
}

// =============================================================================
// Convenience method tests
// =============================================================================

#[test]
fn protect2_convenience() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();

        let (a, b) = scope.protect2(Rf_ScalarInteger(1), Rf_ScalarReal(2.0));

        assert!(!a.get().is_null());
        assert!(!b.get().is_null());
        assert_eq!(scope.count(), 2);
    });
}

#[test]
fn protect3_convenience() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();

        let (a, b, c) =
            scope.protect3(Rf_ScalarInteger(1), Rf_ScalarReal(2.0), Rf_ScalarInteger(3));

        assert!(!a.get().is_null());
        assert!(!b.get().is_null());
        assert!(!c.get().is_null());
        assert_eq!(scope.count(), 3);
    });
}

#[test]
fn protect_raw_convenience() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();

        let sexp = scope.protect_raw(Rf_ScalarInteger(42));

        assert!(!sexp.is_null());
        assert_eq!(scope.count(), 1);
    });
}
