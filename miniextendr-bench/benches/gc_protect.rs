//! GC protection benchmarks.
//!
//! **NOTE**: Protection-specific benchmarks here include SEXP allocation in the
//! timed region. For accurate protection-only timings, see `gc_protection_compare.rs`
//! which pre-allocates SEXPs outside the timed loop. The list/strvec construction
//! benchmarks below are still valid — they measure end-to-end construction cost
//! where allocation is part of the workload.
//!
//! Compares:
//! - `ProtectScope` vs raw `Rf_protect/Rf_unprotect`
//! - `OwnedProtect` vs `ProtectScope::protect`
//! - `ReprotectSlot::set` vs re-protect patterns
//! - `List::set_elt` vs `set_elt_unchecked`
//! - `ListBuilder` vs manual list construction
//! - `StrVecBuilder` vs manual string vector construction

use miniextendr_api::ffi::{self, Rf_allocVector, Rf_protect, Rf_unprotect, SEXPTYPE};
use miniextendr_api::gc_protect::{OwnedProtect, ProtectIndex, ProtectScope};
use miniextendr_api::list::{List, ListAccumulator, ListBuilder, collect_list};
use miniextendr_api::preserve;
use miniextendr_api::strvec::{StrVec, StrVecBuilder};

fn main() {
    miniextendr_bench::init();
    divan::main();
}

// region: ProtectScope overhead

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

/// Preserve list: insert + release
#[divan::bench]
fn preserve_insert_release() {
    unsafe {
        let sexp = ffi::Rf_ScalarInteger(42);
        let cell = preserve::insert(sexp);
        preserve::release(cell);
        divan::black_box(cell);
    }
}

/// Preserve list (unchecked): insert + release
#[divan::bench]
fn preserve_insert_release_unchecked() {
    unsafe {
        let sexp = ffi::Rf_ScalarInteger(42);
        let cell = preserve::insert_unchecked(sexp);
        preserve::release_unchecked(cell);
        divan::black_box(cell);
    }
}

/// Preserve list: insert N values then release all (doubly-linked list O(n))
#[divan::bench(args = [10, 100, 1000])]
fn preserve_multiple(n: usize) {
    unsafe {
        let mut cells = Vec::with_capacity(n);
        for i in 0..n {
            let sexp = ffi::Rf_ScalarInteger(i as i32);
            cells.push(preserve::insert_unchecked(sexp));
        }
        // Release in reverse order (typical LIFO pattern)
        for cell in cells.into_iter().rev() {
            preserve::release_unchecked(cell);
        }
    }
}

/// Preserve list: insert N values then release in arbitrary order
/// Shows O(1) release advantage of doubly-linked list
#[divan::bench(args = [10, 100, 1000])]
fn preserve_release_arbitrary_order(n: usize) {
    unsafe {
        let mut cells = Vec::with_capacity(n);
        for i in 0..n {
            let sexp = ffi::Rf_ScalarInteger(i as i32);
            cells.push(preserve::insert_unchecked(sexp));
        }
        // Release in "random" order (every 3rd, then every 2nd, then rest)
        for i in (0..n).step_by(3) {
            preserve::release_unchecked(cells[i]);
        }
        for i in (1..n).step_by(3) {
            preserve::release_unchecked(cells[i]);
        }
        for i in (2..n).step_by(3) {
            preserve::release_unchecked(cells[i]);
        }
    }
}

/// Preserve list: count check
#[cfg(feature = "debug-preserve")]
#[divan::bench]
fn preserve_count() {
    unsafe {
        divan::black_box(preserve::count());
    }
}

/// Preserve list: large scale test (matches ppsize_* benchmarks in refcount_protect)
#[divan::bench(args = [10000, 50000, 100000, 200000, 300000, 400000, 500000])]
fn preserve_ppsize_scale(n: usize) {
    unsafe {
        let mut cells = Vec::with_capacity(n);
        for i in 0..n {
            let sexp = ffi::Rf_ScalarInteger((i % 1000) as i32);
            cells.push(preserve::insert_unchecked(sexp));
        }
        // Release all
        for cell in cells {
            preserve::release_unchecked(cell);
        }
    }
}

/// Preserve list: large scale with arbitrary release order
#[divan::bench(args = [10000, 50000, 100000])]
fn preserve_ppsize_arbitrary_order(n: usize) {
    unsafe {
        let mut cells = Vec::with_capacity(n);
        for i in 0..n {
            let sexp = ffi::Rf_ScalarInteger((i % 1000) as i32);
            cells.push(preserve::insert_unchecked(sexp));
        }
        // Release in "random" order
        for i in (0..n).step_by(3) {
            preserve::release_unchecked(cells[i]);
        }
        for i in (1..n).step_by(3) {
            preserve::release_unchecked(cells[i]);
        }
        for i in (2..n).step_by(3) {
            preserve::release_unchecked(cells[i]);
        }
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
// endregion

// region: Raw R API reference (expensive variants)

/// Reference: direct R API calls that higher-level helpers stack together.
#[divan::bench]
fn raw_expensive_reference() {
    unsafe {
        // Preserve / release (precious list)
        let preserved = Rf_allocVector(SEXPTYPE::INTSXP, 1);
        ffi::R_PreserveObject(preserved);
        ffi::R_ReleaseObject(preserved);

        // Protect-with-index + reprotect (replace-in-place)
        let mut idx: ProtectIndex = 0;
        let slot_value = Rf_allocVector(SEXPTYPE::INTSXP, 1);
        ffi::R_ProtectWithIndex(slot_value, std::ptr::from_mut(&mut idx));

        // Protect new value temporarily to avoid GC gap before reprotect
        let replaced = Rf_allocVector(SEXPTYPE::INTSXP, 1);
        Rf_protect(replaced);
        ffi::R_Reprotect(replaced, idx);
        Rf_unprotect(1);

        // Unprotect by pointer (linear scan)
        ffi::Rf_unprotect_ptr(replaced);

        divan::black_box((preserved, replaced));
    }
}
// endregion

// region: ReprotectSlot benchmarks

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
// endregion

// region: List construction benchmarks

const LIST_SIZES: [isize; 3] = [10, 100, 1000];

/// ListBuilder: allocate list and set elements
#[divan::bench(args = [0usize, 1, 2])]
fn list_builder_construction(size_idx: usize) {
    let n = LIST_SIZES[size_idx];
    unsafe {
        let scope = ProtectScope::new();
        let builder = ListBuilder::new(&scope, n as usize);

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
// endregion

// region: StrVec construction benchmarks

/// StrVecBuilder: allocate and set strings
#[divan::bench(args = [0usize, 1, 2])]
fn strvec_builder_construction(size_idx: usize) {
    let n = LIST_SIZES[size_idx];
    unsafe {
        let scope = ProtectScope::new();
        let builder = StrVecBuilder::new(&scope, n as usize);

        for i in 0..n {
            builder.set_str(i, "hello");
        }

        divan::black_box(builder.into_sexp());
    }
}

/// Manual string vector construction
///
/// # Safety Warning
///
/// This benchmark is **intentionally unsafe** to measure the cost difference
/// vs. properly protected approaches. The CHARSXP from `Rf_mkCharLenCE` is
/// NOT protected before `SET_STRING_ELT`, creating a GC window. In real code,
/// use `StrVecBuilder::set_str` or protect the CHARSXP.
///
/// **DO NOT copy this pattern into production code.**
#[divan::bench(args = [0usize, 1, 2])]
fn strvec_manual_construction(size_idx: usize) {
    let n = LIST_SIZES[size_idx];
    unsafe {
        let scope = ProtectScope::new();
        let vec = scope.protect_raw(Rf_allocVector(SEXPTYPE::STRSXP, n));

        for i in 0..n {
            // UNSAFE: charsxp unprotected - GC risk! See doc comment above.
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
// endregion

// region: Mixed workload benchmarks

/// Realistic: build a named list with mixed types
#[divan::bench]
fn build_named_list_realistic() {
    unsafe {
        let scope = ProtectScope::new();
        let builder = ListBuilder::new(&scope, 5);

        // Integer
        builder.set(0, scope.protect_raw(ffi::Rf_ScalarInteger(42)));
        // Real
        builder.set(1, scope.protect_raw(ffi::Rf_ScalarReal(1.5)));
        // Logical
        builder.set(2, scope.protect_raw(ffi::Rf_ScalarLogical(1)));
        // String - protect CHARSXP before passing to Rf_ScalarString
        let s = scope.protect_raw(ffi::Rf_mkCharLenCE("test".as_ptr().cast(), 4, ffi::CE_UTF8));
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
// endregion

// region: Stack pressure benchmarks (ppsize stress tests)
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
        let outer = ListBuilder::new(&scope, outer_count);

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
        let builder = ListBuilder::new(&scope, n);

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
        let list1 =
            List::from_raw(scope1.protect_raw(Rf_allocVector(SEXPTYPE::VECSXP, n as isize)));
        for i in 0..n {
            list1.set_elt(i as isize, ffi::Rf_ScalarInteger(i as i32));
        }
        divan::black_box(list1.as_sexp());
        // scope1 drops: unprotect 1

        // Growing stack pattern (protect_raw + unchecked)
        let scope2 = ProtectScope::new();
        let list2 =
            List::from_raw(scope2.protect_raw(Rf_allocVector(SEXPTYPE::VECSXP, n as isize)));
        for i in 0..n {
            let child = scope2.protect_raw(ffi::Rf_ScalarInteger(i as i32));
            list2.set_elt_unchecked(i as isize, child);
        }
        divan::black_box(list2.as_sexp());
        // scope2 drops: unprotect 1 + n
    }
}
// endregion

// region: New ergonomic features benchmarks

/// ProtectScope::alloc_vector vs manual allocate + protect
#[divan::bench]
fn alloc_vector_helper() {
    unsafe {
        let scope = ProtectScope::new();
        let vec = scope.alloc_vector(SEXPTYPE::INTSXP, 100);
        divan::black_box(vec.get());
    }
}

/// Manual: allocate then protect separately
#[divan::bench]
fn alloc_then_protect() {
    unsafe {
        let scope = ProtectScope::new();
        let vec = scope.protect(Rf_allocVector(SEXPTYPE::INTSXP, 100));
        divan::black_box(vec.get());
    }
}

/// ReprotectSlot::set_with vs manual protect+set pattern
#[divan::bench(args = [10, 100, 1000])]
fn reprotect_set_with(iterations: usize) {
    unsafe {
        let scope = ProtectScope::new();
        let slot = scope.protect_with_index(ffi::Rf_ScalarInteger(0));

        for i in 0..iterations {
            // Safe pattern: set_with handles temp protection internally
            slot.set_with(|| ffi::Rf_ScalarInteger(i as i32));
        }

        divan::black_box(slot.get());
    }
}

/// Manual pattern: protect temp, reprotect, unprotect temp
#[divan::bench(args = [10, 100, 1000])]
fn reprotect_manual_pattern(iterations: usize) {
    unsafe {
        let scope = ProtectScope::new();
        let slot = scope.protect_with_index(ffi::Rf_ScalarInteger(0));

        for i in 0..iterations {
            // Manual pattern - more verbose, same result
            let new_val = ffi::Rf_ScalarInteger(i as i32);
            Rf_protect(new_val);
            slot.set(new_val);
            Rf_unprotect(1);
        }

        divan::black_box(slot.get());
    }
}
// endregion

// region: ListAccumulator benchmarks (unknown-length list construction)

/// ListAccumulator: unknown-length list with bounded stack
#[divan::bench(args = [10, 100, 1000])]
fn list_accumulator_push(n: usize) {
    unsafe {
        let scope = ProtectScope::new();
        let mut acc = ListAccumulator::new(&scope, 4);

        for i in 0..n {
            acc.push(i as i32);
        }

        divan::black_box(acc.into_sexp());
    }
}

/// ListAccumulator with exact initial capacity (no growth)
#[divan::bench(args = [10, 100, 1000])]
fn list_accumulator_exact_cap(n: usize) {
    unsafe {
        let scope = ProtectScope::new();
        let mut acc = ListAccumulator::new(&scope, n);

        for i in 0..n {
            acc.push(i as i32);
        }

        divan::black_box(acc.into_sexp());
    }
}

/// collect_list from iterator
#[divan::bench(args = [10, 100, 1000])]
fn collect_list_iterator(n: usize) {
    unsafe {
        let scope = ProtectScope::new();
        let list = collect_list(&scope, (0..n).map(|i| i as i32));
        divan::black_box(list.get());
    }
}

/// Naive pattern: Vec<SEXP> then build list (pre-collect for comparison)
/// Note: This is actually a decent pattern when length is known after iteration
#[divan::bench(args = [10, 100, 1000])]
fn collect_into_vec_then_list(n: usize) {
    unsafe {
        let scope = ProtectScope::new();

        // First collect into Vec (no R protection yet)
        let values: Vec<i32> = (0..n).map(|i| i as i32).collect();

        // Then build list with known size
        let builder = ListBuilder::new(&scope, values.len());
        for (i, v) in values.into_iter().enumerate() {
            builder.set_protected(i as isize, ffi::Rf_ScalarInteger(v));
        }

        divan::black_box(builder.into_sexp());
    }
}

/// Compare ListAccumulator vs ListBuilder (known size)
/// Shows the overhead of dynamic growth
#[divan::bench(args = [100, 500, 1000])]
fn accumulator_vs_builder_known_size(n: usize) {
    unsafe {
        // ListBuilder (optimal for known size)
        let scope1 = ProtectScope::new();
        let builder = ListBuilder::new(&scope1, n);
        for i in 0..n {
            builder.set_protected(i as isize, ffi::Rf_ScalarInteger(i as i32));
        }
        divan::black_box(builder.into_sexp());

        // ListAccumulator (for comparison - handles unknown size)
        let scope2 = ProtectScope::new();
        let mut acc = ListAccumulator::new(&scope2, n); // same initial cap
        for i in 0..n {
            acc.push(i as i32);
        }
        divan::black_box(acc.into_sexp());
    }
}

/// Stack pressure test: ListAccumulator maintains O(1) stack
#[divan::bench(args = [1000, 5000, 10000])]
fn list_accumulator_stack_pressure(n: usize) {
    unsafe {
        let scope = ProtectScope::new();
        let mut acc = ListAccumulator::new(&scope, 4);

        for i in 0..n {
            acc.push(i as i32);
        }

        // Stack usage: always 2 (list slot + temp slot)
        // regardless of n
        divan::black_box(acc.into_sexp());
    }
}
// endregion

// region: Typed vector collection (scope.collect)
//
// For typed vectors (INTSXP, REALSXP, etc.), there's no need for complex
// protection during construction. You allocate once, protect once, then
// fill by writing to the data pointer - no GC can occur during fills.
//
// For unknown-length iterators, just collect to Vec<T> first, then use
// scope.collect(vec) which accepts any IntoIterator + ExactSizeIterator.

/// scope.collect: exact-size iterator to typed vector
#[divan::bench(args = [100, 1000, 10000])]
fn scope_collect_exact(n: usize) {
    unsafe {
        let scope = ProtectScope::new();
        let vec = scope.collect((0..n).map(|i| i as i32));
        divan::black_box(vec.get());
    }
}

/// For unknown length: collect to Vec first, then scope.collect
#[divan::bench(args = [100, 1000, 10000])]
fn vec_then_scope_collect(n: usize) {
    unsafe {
        let scope = ProtectScope::new();
        // Collect to Vec first (non-exact iterator)
        let items: Vec<i32> = (0..n * 2)
            .filter(|x| x % 2 == 0)
            .map(|i| i as i32)
            .collect();
        // Then use scope.collect
        let vec = scope.collect(items);
        divan::black_box(vec.get());
    }
}

/// Baseline: manual allocation + fill via dataptr
#[divan::bench(args = [100, 1000, 10000])]
fn manual_typed_vector(n: usize) {
    unsafe {
        let scope = ProtectScope::new();
        let vec = scope.protect(Rf_allocVector(SEXPTYPE::INTSXP, n as isize));
        let ptr = ffi::INTEGER(vec.get());

        for i in 0..n {
            *ptr.add(i) = i as i32;
        }

        divan::black_box(vec.get());
    }
}

/// Compare: typed vector vs list for same data
/// Shows the efficiency gain of direct memory access vs boxing each element
#[divan::bench(args = [100, 1000])]
fn typed_vector_vs_list(n: usize) {
    unsafe {
        // Typed vector (direct memory via scope.collect)
        let scope1 = ProtectScope::new();
        let vec = scope1.collect((0..n).map(|i| i as i32));
        divan::black_box(vec.get());

        // List (boxed scalars via collect_list)
        let scope2 = ProtectScope::new();
        let list = collect_list(&scope2, (0..n).map(|i| i as i32));
        divan::black_box(list.get());
    }
}

/// scope.collect with f64 (REALSXP)
#[divan::bench(args = [100, 1000, 10000])]
fn scope_collect_f64(n: usize) {
    unsafe {
        let scope = ProtectScope::new();
        let vec = scope.collect((0..n).map(|i| i as f64 * 1.5));
        divan::black_box(vec.get());
    }
}

/// scope.collect from Vec (shows Vec -> R vector conversion)
#[divan::bench(args = [100, 1000, 10000])]
fn scope_collect_from_vec(n: usize) {
    unsafe {
        let scope = ProtectScope::new();
        let items: Vec<i32> = (0..n).map(|i| i as i32).collect();
        let vec = scope.collect(items);
        divan::black_box(vec.get());
    }
}
// endregion
