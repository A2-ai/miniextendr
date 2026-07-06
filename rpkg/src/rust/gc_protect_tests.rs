//! Tests for GC protection patterns.
//!
//! These tests verify that the protection APIs work correctly.

use miniextendr_api::SEXPTYPE;
use miniextendr_api::gc_protect::ProtectScope;
use miniextendr_api::list::{List, ListBuilder};
use miniextendr_api::miniextendr;
use miniextendr_api::prelude::SexpExt;
use miniextendr_api::strvec::{StrVec, StrVecBuilder};
use miniextendr_api::sys::Rf_allocVector;

// region: ListBuilder tests

/// Test that ListBuilder reports the correct length.
/// @param n Number of list elements to allocate.
#[miniextendr(noexport)]
pub fn test_list_builder_length(n: i32) -> i32 {
    unsafe {
        let scope = ProtectScope::new();
        let builder = ListBuilder::new(&scope, n as usize);
        builder.len() as i32
    }
}

/// Test setting heterogeneous child vectors in a ListBuilder.
#[miniextendr(noexport)]
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
#[miniextendr(noexport)]
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
#[miniextendr(noexport)]
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
#[miniextendr(noexport)]
pub fn test_strvec_builder_length(n: i32) -> i32 {
    unsafe {
        let scope = ProtectScope::new();
        let builder = StrVecBuilder::new(&scope, n as usize);
        builder.len() as i32
    }
}

/// Test StrVecBuilder set_str, set_na, and set_opt_str methods.
#[miniextendr(noexport)]
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
#[miniextendr(noexport)]
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
#[miniextendr(noexport)]
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
#[miniextendr(noexport)]
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
#[miniextendr(noexport)]
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
#[miniextendr(noexport)]
pub fn test_list_from_values_strings_gctorture() -> List {
    let v: Vec<String> = (0..16).map(|i| format!("element-{i}")).collect();
    List::from_values(v)
}

/// Build a named list from a Vec of allocating `(name, value)` pairs via
/// `List::from_pairs`. Same UAF shape as `test_list_from_values_strings_gctorture`.
#[miniextendr(noexport)]
pub fn test_list_from_pairs_strings_gctorture() -> List {
    let pairs: Vec<(String, String)> = (0..16)
        .map(|i| (format!("k{i}"), format!("v{i}")))
        .collect();
    List::from_pairs(pairs)
}

// endregion

// region: Protected<'a, T> bundle

/// Exercise `Protected<'a, T>`: allocate a STRSXP, bundle it with a `StrVec`
/// view under `Protected::new`, then fill it — each `set_str` allocates a
/// CHARSXP while the bundle's guard is the only protection on the STRSXP
/// (gctorture-sensitive; no-arg so the fast gc_stress sweep picks it up, #430).
/// Reads back through `Deref` and `get()` before the guard drops.
#[miniextendr(noexport)]
pub fn test_protected_strvec_bundle() -> Vec<Option<String>> {
    use miniextendr_api::gc_protect::Protected;

    unsafe {
        let sexp = Rf_allocVector(SEXPTYPE::STRSXP, 3);
        let bundle = Protected::new(sexp, StrVec::from_raw(sexp));

        // Allocating writes: the STRSXP must survive each Rf_mkChar.
        bundle.set_str(0, "alpha");
        bundle.set_na(1);
        bundle.set_str(2, "gamma");

        let view = bundle.get();
        let mut out = Vec::with_capacity(3);
        for i in 0..3 {
            out.push(view.get_str(i).map(|s| s.to_string()));
        }
        out
        // bundle drops here → UNPROTECT(1)
    }
}

/// Exercise `Protected::from_trusted` + `into_inner`: bundle an
/// already-protected SEXP without double-protecting, then unwrap the view.
#[miniextendr(noexport)]
pub fn test_protected_from_trusted() -> Vec<Option<String>> {
    use miniextendr_api::gc_protect::Protected;

    unsafe {
        let scope = ProtectScope::new();
        let sexp = scope.protect_raw(Rf_allocVector(SEXPTYPE::STRSXP, 2));
        let bundle = Protected::from_trusted(sexp, StrVec::from_raw(sexp));

        bundle.set_str(0, "trusted");
        bundle.set_na(1);

        let view = bundle.into_inner();
        let mut out = Vec::with_capacity(2);
        for i in 0..2 {
            out.push(view.get_str(i).map(|s| s.to_string()));
        }
        out
    }
}
// endregion

// region: Module registration
// endregion
