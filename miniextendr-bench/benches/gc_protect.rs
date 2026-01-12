//! GC protection benchmarks.
//!
//! Compares:
//! - `ProtectScope` vs raw `Rf_protect/Rf_unprotect`
//! - `OwnedProtect` vs `ProtectScope::protect`
//! - `ReprotectSlot::set` vs re-protect patterns
//! - `List::set_elt` vs `set_elt_unchecked`
//! - `ListBuilder` vs manual list construction
//! - `StrVecBuilder` vs manual string vector construction

use miniextendr_api::ffi::{self, Rf_allocVector, Rf_protect, Rf_unprotect, SEXPTYPE};
use miniextendr_api::gc_protect::{OwnedProtect, ProtectScope};
use miniextendr_api::list::{List, ListBuilder};
use miniextendr_api::strvec::{StrVec, StrVecBuilder};

fn main() {
    miniextendr_bench::init();
    divan::main();
}

// =============================================================================
// ProtectScope overhead
// =============================================================================

/// Baseline: raw Rf_protect/Rf_unprotect
#[divan::bench]
fn raw_protect_unprotect() {
    unsafe {
        let sexp = ffi::Rf_ScalarInteger(42);
        Rf_protect(sexp);
        Rf_unprotect(1);
        divan::black_box(sexp);
    }
}

/// OwnedProtect RAII guard
#[divan::bench]
fn owned_protect() {
    unsafe {
        let guard = OwnedProtect::new(ffi::Rf_ScalarInteger(42));
        divan::black_box(guard.get());
    }
}

/// ProtectScope with single protect
#[divan::bench]
fn protect_scope_single() {
    unsafe {
        let scope = ProtectScope::new();
        let root = scope.protect(ffi::Rf_ScalarInteger(42));
        divan::black_box(root.get());
    }
}

/// ProtectScope with multiple protects
#[divan::bench(args = [4, 16, 64])]
fn protect_scope_multiple(n: usize) {
    unsafe {
        let scope = ProtectScope::new();
        for _ in 0..n {
            let _ = scope.protect(ffi::Rf_ScalarInteger(42));
        }
        divan::black_box(scope.count());
    }
}

/// Raw protect multiple (for comparison)
#[divan::bench(args = [4, 16, 64])]
fn raw_protect_multiple(n: usize) {
    unsafe {
        for _ in 0..n {
            Rf_protect(ffi::Rf_ScalarInteger(42));
        }
        Rf_unprotect(n as i32);
    }
}

// =============================================================================
// ReprotectSlot benchmarks
// =============================================================================

/// ReprotectSlot: repeated set() calls
#[divan::bench(args = [10, 100, 1000])]
fn reprotect_slot_set(iterations: usize) {
    unsafe {
        let scope = ProtectScope::new();
        let slot = scope.protect_with_index(ffi::Rf_ScalarInteger(0));

        for i in 0..iterations {
            slot.set(ffi::Rf_ScalarInteger(i as i32));
        }

        divan::black_box(slot.get());
    }
}

/// Alternative: repeated OwnedProtect (WRONG pattern but shows cost)
#[divan::bench(args = [10, 100, 1000])]
fn owned_protect_repeated(iterations: usize) {
    unsafe {
        let scope = ProtectScope::new();

        // Start with a protected value
        let mut current = scope.protect_raw(ffi::Rf_ScalarInteger(0));

        for i in 0..iterations {
            // Each iteration: protect new, forget old protection
            // This grows the stack (bad pattern, but shows cost)
            current = scope.protect_raw(ffi::Rf_ScalarInteger(i as i32));
        }

        divan::black_box(current);
    }
}

// =============================================================================
// List construction benchmarks
// =============================================================================

const LIST_SIZES: [isize; 3] = [10, 100, 1000];

/// ListBuilder: allocate list and set elements
#[divan::bench(args = [0usize, 1, 2])]
fn list_builder_construction(size_idx: usize) {
    let n = LIST_SIZES[size_idx];
    unsafe {
        let scope = ProtectScope::new();
        let builder = ListBuilder::new(&scope, n);

        for i in 0..n {
            let child = scope.protect_raw(ffi::Rf_ScalarInteger(i as i32));
            builder.set(i, child);
        }

        divan::black_box(builder.into_sexp());
    }
}

/// Manual list construction with scope
#[divan::bench(args = [0usize, 1, 2])]
fn list_manual_construction(size_idx: usize) {
    let n = LIST_SIZES[size_idx];
    unsafe {
        let scope = ProtectScope::new();
        let list = scope.protect_raw(Rf_allocVector(SEXPTYPE::VECSXP, n));

        for i in 0..n {
            let child = scope.protect_raw(ffi::Rf_ScalarInteger(i as i32));
            ffi::SET_VECTOR_ELT(list, i, child);
        }

        divan::black_box(list);
    }
}

/// List::set_elt (safe, protects each child)
#[divan::bench(args = [0usize, 1, 2])]
fn list_set_elt_safe(size_idx: usize) {
    let n = LIST_SIZES[size_idx];
    unsafe {
        let scope = ProtectScope::new();
        let list = List::from_raw(scope.protect_raw(Rf_allocVector(SEXPTYPE::VECSXP, n)));

        for i in 0..n {
            // Child is unprotected - set_elt handles protection
            let child = ffi::Rf_ScalarInteger(i as i32);
            list.set_elt(i, child);
        }

        divan::black_box(list.as_sexp());
    }
}

/// List::set_elt_unchecked (unsafe, no per-child protection)
#[divan::bench(args = [0usize, 1, 2])]
fn list_set_elt_unchecked(size_idx: usize) {
    let n = LIST_SIZES[size_idx];
    unsafe {
        let scope = ProtectScope::new();
        let list = List::from_raw(scope.protect_raw(Rf_allocVector(SEXPTYPE::VECSXP, n)));

        for i in 0..n {
            // Pre-protect child, then use unchecked
            let child = scope.protect_raw(ffi::Rf_ScalarInteger(i as i32));
            list.set_elt_unchecked(i, child);
        }

        divan::black_box(list.as_sexp());
    }
}

// =============================================================================
// StrVec construction benchmarks
// =============================================================================

/// StrVecBuilder: allocate and set strings
#[divan::bench(args = [0usize, 1, 2])]
fn strvec_builder_construction(size_idx: usize) {
    let n = LIST_SIZES[size_idx];
    unsafe {
        let scope = ProtectScope::new();
        let builder = StrVecBuilder::new(&scope, n);

        for i in 0..n {
            builder.set_str(i, "hello");
        }

        divan::black_box(builder.into_sexp());
    }
}

/// Manual string vector construction
#[divan::bench(args = [0usize, 1, 2])]
fn strvec_manual_construction(size_idx: usize) {
    let n = LIST_SIZES[size_idx];
    unsafe {
        let scope = ProtectScope::new();
        let vec = scope.protect_raw(Rf_allocVector(SEXPTYPE::STRSXP, n));

        for i in 0..n {
            let charsxp = ffi::Rf_mkCharLenCE("hello".as_ptr().cast(), 5, ffi::CE_UTF8);
            ffi::SET_STRING_ELT(vec, i, charsxp);
        }

        divan::black_box(vec);
    }
}

/// StrVec::set_str (safe, protects each CHARSXP)
#[divan::bench(args = [0usize, 1, 2])]
fn strvec_set_str_safe(size_idx: usize) {
    let n = LIST_SIZES[size_idx];
    unsafe {
        let scope = ProtectScope::new();
        let strvec = StrVec::from_raw(scope.protect_raw(Rf_allocVector(SEXPTYPE::STRSXP, n)));

        for i in 0..n {
            strvec.set_str(i, "hello");
        }

        divan::black_box(strvec.as_sexp());
    }
}

// =============================================================================
// Mixed workload benchmarks
// =============================================================================

/// Realistic: build a named list with mixed types
#[divan::bench]
fn build_named_list_realistic() {
    unsafe {
        let scope = ProtectScope::new();
        let builder = ListBuilder::new(&scope, 5);

        // Integer
        builder.set(0, scope.protect_raw(ffi::Rf_ScalarInteger(42)));
        // Real
        builder.set(1, scope.protect_raw(ffi::Rf_ScalarReal(3.14)));
        // Logical
        builder.set(2, scope.protect_raw(ffi::Rf_ScalarLogical(1)));
        // String
        let s = ffi::Rf_mkCharLenCE("test".as_ptr().cast(), 4, ffi::CE_UTF8);
        builder.set(3, scope.protect_raw(ffi::Rf_ScalarString(s)));
        // Nested list
        let inner = ListBuilder::new(&scope, 2);
        inner.set(0, scope.protect_raw(ffi::Rf_ScalarInteger(1)));
        inner.set(1, scope.protect_raw(ffi::Rf_ScalarInteger(2)));
        builder.set(4, inner.into_sexp());

        divan::black_box(builder.into_sexp());
    }
}

/// Realistic: accumulator pattern with ReprotectSlot
#[divan::bench(args = [10, 100])]
fn accumulator_pattern(iterations: usize) {
    unsafe {
        let scope = ProtectScope::new();

        // Accumulate into a growing vector
        let slot = scope.protect_with_index(Rf_allocVector(SEXPTYPE::INTSXP, 1));

        for i in 1..=iterations {
            // Simulate growing result
            let new_vec = Rf_allocVector(SEXPTYPE::INTSXP, i as isize);
            slot.set(new_vec);
        }

        divan::black_box(slot.get());
    }
}

// =============================================================================
// Stack pressure benchmarks (ppsize stress tests)
// =============================================================================
// These benchmarks test patterns at high iteration counts to demonstrate
// the importance of bounded protect stack usage. R's --max-ppsize minimum
// is 10000, so patterns that stay bounded can handle arbitrary workloads
// while unbounded patterns would overflow.

/// ReprotectSlot with very high iterations - stays at 1 stack slot
#[divan::bench(args = [1000, 5000, 10000])]
fn reprotect_slot_high_iterations(n: usize) {
    unsafe {
        let scope = ProtectScope::new();
        let slot = scope.protect_with_index(ffi::Rf_ScalarInteger(0));

        for i in 0..n {
            slot.set(ffi::Rf_ScalarInteger((i % 1000) as i32));
        }

        // Stack usage: always 1
        divan::black_box(slot.get());
    }
}

/// List::set_elt with high element count - constant stack (1 for list)
#[divan::bench(args = [100, 500, 1000])]
fn list_set_elt_high_count(n: usize) {
    unsafe {
        let scope = ProtectScope::new();
        let list = List::from_raw(scope.protect_raw(Rf_allocVector(SEXPTYPE::VECSXP, n as isize)));

        for i in 0..n {
            // set_elt protects/unprotects internally - constant stack
            list.set_elt(i as isize, ffi::Rf_ScalarInteger(i as i32));
        }

        // Stack usage: always 1
        divan::black_box(list.as_sexp());
    }
}

/// StrVec::set_str with high element count - constant stack (1 for vector)
#[divan::bench(args = [100, 500, 1000])]
fn strvec_set_str_high_count(n: usize) {
    unsafe {
        let scope = ProtectScope::new();
        let vec = StrVec::from_raw(scope.protect_raw(Rf_allocVector(SEXPTYPE::STRSXP, n as isize)));

        for i in 0..n {
            // set_str protects/unprotects internally - constant stack
            vec.set_str(i as isize, "test");
        }

        // Stack usage: always 1
        divan::black_box(vec.as_sexp());
    }
}

/// Nested list construction using ReprotectSlot - bounded stack
#[divan::bench(args = [10, 50, 100])]
fn nested_lists_reprotect(outer_count: usize) {
    unsafe {
        let scope = ProtectScope::new();
        let outer = ListBuilder::new(&scope, outer_count as isize);

        for i in 0..outer_count {
            // Use reprotect slot for inner list - each iteration reuses slot
            let slot = scope.protect_with_index(Rf_allocVector(SEXPTYPE::VECSXP, 5));
            let inner = List::from_raw(slot.get());

            for j in 0..5isize {
                inner.set_elt(j, ffi::Rf_ScalarInteger((i * 5 + j as usize) as i32));
            }

            outer.set(i as isize, slot.get());
        }

        // Stack usage: 1 (outer) + outer_count (reprotect slots)
        divan::black_box(outer.into_sexp());
    }
}

/// Compare: ListBuilder with pre-protected children - grows stack
#[divan::bench(args = [100, 500, 1000])]
fn list_builder_grows_stack(n: usize) {
    unsafe {
        let scope = ProtectScope::new();
        let builder = ListBuilder::new(&scope, n as isize);

        for i in 0..n {
            // Each child adds to protect stack
            let child = scope.protect_raw(ffi::Rf_ScalarInteger(i as i32));
            builder.set(i as isize, child);
        }

        // Stack usage: 1 + n (grows with n)
        divan::black_box(builder.into_sexp());
    }
}

/// Compare: List::set_elt constant vs ListBuilder growing
/// Shows the trade-off: set_elt has per-element overhead but bounded stack
#[divan::bench(args = [100, 500, 1000])]
fn list_constant_vs_growing(n: usize) {
    unsafe {
        // Constant stack pattern (set_elt)
        let scope1 = ProtectScope::new();
        let list1 = List::from_raw(scope1.protect_raw(Rf_allocVector(SEXPTYPE::VECSXP, n as isize)));
        for i in 0..n {
            list1.set_elt(i as isize, ffi::Rf_ScalarInteger(i as i32));
        }
        divan::black_box(list1.as_sexp());
        // scope1 drops: unprotect 1

        // Growing stack pattern (protect_raw + unchecked)
        let scope2 = ProtectScope::new();
        let list2 = List::from_raw(scope2.protect_raw(Rf_allocVector(SEXPTYPE::VECSXP, n as isize)));
        for i in 0..n {
            let child = scope2.protect_raw(ffi::Rf_ScalarInteger(i as i32));
            list2.set_elt_unchecked(i as isize, child);
        }
        divan::black_box(list2.as_sexp());
        // scope2 drops: unprotect 1 + n
    }
}
