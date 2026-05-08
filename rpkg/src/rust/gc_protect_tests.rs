//! Tests for GC protection patterns.
//!
//! These tests verify that the protection APIs work correctly.

use miniextendr_api::ffi::{Rf_allocVector, SEXPTYPE, SexpExt};
use miniextendr_api::gc_protect::ProtectScope;
use miniextendr_api::list::{List, ListBuilder};
use miniextendr_api::miniextendr;
use miniextendr_api::strvec::{StrVec, StrVecBuilder};

// region: ListBuilder tests

/// Test that ListBuilder reports the correct length.
/// @param n Number of list elements to allocate.
#[miniextendr]
pub fn test_list_builder_length(n: i32) -> i32 {
    unsafe {
        let scope = ProtectScope::new();
        let builder = ListBuilder::new(&scope, n as usize);
        builder.len() as i32
    }
}

/// Test setting heterogeneous child vectors in a ListBuilder.
#[miniextendr]
pub fn test_list_builder_set() -> List {
    unsafe {
        let scope = ProtectScope::new();
        let builder = ListBuilder::new(&scope, 3);

        // Create and set child vectors
        let child1 = scope.protect_raw(Rf_allocVector(SEXPTYPE::INTSXP, 1));
        let child2 = scope.protect_raw(Rf_allocVector(SEXPTYPE::REALSXP, 2));
        let child3 = scope.protect_raw(Rf_allocVector(SEXPTYPE::STRSXP, 3));

        builder.set(0, child1);
        builder.set(1, child2);
        builder.set(2, child3);

        builder.into_list()
    }
}

/// Test List::set_elt with unprotected child vectors (set_elt handles protection).
#[miniextendr]
pub fn test_list_set_elt() -> List {
    unsafe {
        let scope = ProtectScope::new();
        let list = List::from_raw(scope.protect_raw(Rf_allocVector(SEXPTYPE::VECSXP, 2)));

        // These children are unprotected - set_elt should handle protection
        let child1 = Rf_allocVector(SEXPTYPE::INTSXP, 5);
        let child2 = Rf_allocVector(SEXPTYPE::REALSXP, 10);

        list.set_elt(0, child1);
        list.set_elt(1, child2);

        list
    }
}

/// Test List::set_elt_with using closure-based lazy allocation of child vectors.
#[miniextendr]
pub fn test_list_set_elt_with() -> List {
    unsafe {
        let scope = ProtectScope::new();
        let list = List::from_raw(scope.protect_raw(Rf_allocVector(SEXPTYPE::VECSXP, 2)));

        list.set_elt_with(0, || Rf_allocVector(SEXPTYPE::INTSXP, 3));
        list.set_elt_with(1, || Rf_allocVector(SEXPTYPE::REALSXP, 4));

        list
    }
}
// endregion

// region: StrVecBuilder tests

/// Test that StrVecBuilder reports the correct length.
/// @param n Number of string elements to allocate.
#[miniextendr]
pub fn test_strvec_builder_length(n: i32) -> i32 {
    unsafe {
        let scope = ProtectScope::new();
        let builder = StrVecBuilder::new(&scope, n as usize);
        builder.len() as i32
    }
}

/// Test StrVecBuilder set_str, set_na, and set_opt_str methods.
#[miniextendr]
pub fn test_strvec_builder_set() -> Vec<Option<String>> {
    unsafe {
        let scope = ProtectScope::new();
        let builder = StrVecBuilder::new(&scope, 4);

        builder.set_str(0, "hello");
        builder.set_str(1, "world");
        builder.set_na(2);
        builder.set_opt_str(3, Some("test"));

        // Convert back to Vec for verification
        let strvec = builder.into_strvec();
        let mut result = Vec::with_capacity(4);
        for i in 0..4 {
            result.push(strvec.get_str(i).map(|s| s.to_string()));
        }
        result
    }
}

/// Test StrVec set_str and set_na on a raw STRSXP allocation.
#[miniextendr]
pub fn test_strvec_set_str() -> Vec<Option<String>> {
    unsafe {
        let scope = ProtectScope::new();
        let strvec = StrVec::from_raw(scope.protect_raw(Rf_allocVector(SEXPTYPE::STRSXP, 3)));

        strvec.set_str(0, "first");
        strvec.set_str(1, "second");
        strvec.set_na(2);

        let mut result = Vec::with_capacity(3);
        for i in 0..3 {
            result.push(strvec.get_str(i).map(|s| s.to_string()));
        }
        result
    }
}
// endregion

// region: ReprotectSlot tests

/// Test ReprotectSlot by repeatedly replacing with larger vectors up to length n.
/// @param n Final expected vector length.
#[miniextendr]
pub fn test_reprotect_slot_accumulate(n: i32) -> i32 {
    unsafe {
        let scope = ProtectScope::new();

        // Start with a vector of length 1
        let slot = scope.protect_with_index(Rf_allocVector(SEXPTYPE::INTSXP, 1));

        // Repeatedly replace with longer vectors
        for i in 2..=n {
            let new_vec = Rf_allocVector(SEXPTYPE::INTSXP, i as isize);
            slot.set(new_vec);
        }

        // Final vector should have length n
        slot.get().xlength() as i32
    }
}

/// Test that ProtectScope counts protected objects correctly (slot + regular).
#[miniextendr]
pub fn test_reprotect_slot_count() -> i32 {
    unsafe {
        let scope = ProtectScope::new();

        // Create slot
        let _slot = scope.protect_with_index(Rf_allocVector(SEXPTYPE::INTSXP, 1));

        // Count should be 1
        let count_after_slot = scope.count();

        // Create another protected value
        let _other = scope.protect(Rf_allocVector(SEXPTYPE::REALSXP, 1));

        // Count should be 2
        let count_after_other = scope.count();

        // Verify: slot=1, other=1, total=2
        if count_after_slot == 1 && count_after_other == 2 {
            1 // success
        } else {
            0 // failure
        }
    }
}

/// Test that ReprotectSlot::set does not grow the protect stack over many iterations.
/// @param iterations Number of set() calls to perform.
#[miniextendr]
pub fn test_reprotect_slot_no_growth(iterations: i32) -> i32 {
    unsafe {
        let scope = ProtectScope::new();

        let slot = scope.protect_with_index(Rf_allocVector(SEXPTYPE::INTSXP, 1));
        let initial_count = scope.count();

        // Many set() calls
        for _ in 0..iterations {
            let new_vec = Rf_allocVector(SEXPTYPE::INTSXP, 1);
            slot.set(new_vec);
        }

        let final_count = scope.count();

        // Count should not have grown
        if initial_count == final_count {
            1 // success
        } else {
            0 // failure
        }
    }
}
// endregion

// region: Vec<T>::into_list / List::from_pairs UAF regression

/// Build a list from a Vec of allocating values via the blanket
/// `IntoList for Vec<T>` impl. Each `String::into_sexp()` allocates a fresh
/// STRSXP; pre-fix the buffer was held unrooted across allocations and crashed
/// under `gctorture(TRUE)` (same shape as the columnar UAF, issue #307).
#[miniextendr]
pub fn test_list_from_values_strings_gctorture() -> List {
    let v: Vec<String> = (0..16).map(|i| format!("element-{i}")).collect();
    List::from_values(v)
}

/// Build a named list from a Vec of allocating `(name, value)` pairs via
/// `List::from_pairs`. Same UAF shape as `test_list_from_values_strings_gctorture`.
#[miniextendr]
pub fn test_list_from_pairs_strings_gctorture() -> List {
    let pairs: Vec<(String, String)> = (0..16)
        .map(|i| (format!("k{i}"), format!("v{i}")))
        .collect();
    List::from_pairs(pairs)
}

// endregion

// region: Module registration
// endregion
