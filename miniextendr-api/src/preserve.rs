//! R object preservation using a circular doubly-linked list.
//!
//! This module provides a protection scheme for R objects (SEXPs) that need to
//! survive R's garbage collection **across multiple `.Call` invocations**.
//!
//! # Protection Strategies in miniextendr
//!
//! miniextendr provides three complementary protection mechanisms for different scenarios:
//!
//! | Strategy | Module | Lifetime | Release Order | Use Case |
//! |----------|--------|----------|---------------|----------|
//! | **PROTECT stack** | [`gc_protect`](crate::gc_protect) | Within `.Call` | LIFO (stack) | Temporary allocations |
//! | **Preserve list** | [`preserve`](crate::preserve) | Across `.Call`s | Any order | Long-lived R objects |
//! | **R ownership** | [`ExternalPtr`](struct@crate::ExternalPtr) | Until R GCs | R decides | Rust data owned by R |
//!
//! ## When to Use This Module
//!
//! **Use `preserve` (this module) when:**
//! - Objects must survive across multiple `.Call` invocations
//! - You need to release protections in arbitrary order (not LIFO)
//! - Example: [`RAllocator`](crate::RAllocator) backing memory
//!
//! **Use [`gc_protect`](crate::gc_protect) instead when:**
//! - Protection is short-lived (within a single `.Call`)
//! - You want RAII-based automatic PROTECT/UNPROTECT balancing
//!
//! **Use [`ExternalPtr`](struct@crate::ExternalPtr) instead when:**
//! - You want R to own a Rust value with automatic cleanup
//!
//! # Architecture
//!
//! This module uses a cpp11-style circular doubly-linked list approach,
//! which has advantages over the R protect stack:
//!
//! - No balance requirement (PROTECT/UNPROTECT pairs must be balanced)
//! - Can release protections in any order
//! - Thread-local storage (each thread has its own preserve list)
//! - More ergonomic with RAII patterns
//!
//! The preservation list is a circular doubly-linked cons list where:
//! - The list itself is preserved with `R_PreserveObject` (never GC'd)
//! - Each protected SEXP is stored as the TAG of a cell
//! - CAR points to previous cell, CDR points to next cell
//! - Head and tail are sentinel nodes
//!
//! # Safety
//!
//! All functions in this module are unsafe and must be called from the R main thread.

use crate::ffi::{
    CAR, CDR, R_NilValue, R_PreserveObject, R_xlen_t, Rf_cons, Rf_protect, Rf_unprotect,
    Rf_xlength, SET_TAG, SETCAR, SETCDR, SEXP,
};
use std::cell::OnceCell;

thread_local! {
    /// The per-thread preservation list.
    ///
    /// Initialized on first use with a circular doubly-linked list
    /// that is preserved from R's GC via `R_PreserveObject`.
    static PRESERVE_LIST: OnceCell<SEXP> = const { OnceCell::new() };
}

/// Initialize the preservation list.
///
/// Creates a circular doubly-linked list: `(head -> sentinel -> head)`
/// and preserves it with `R_PreserveObject` so it's never GC'd.
///
/// # Safety
///
/// Must be called from the R main thread.
#[inline]
unsafe fn init() -> SEXP {
    unsafe {
        let out = Rf_cons(R_NilValue, Rf_cons(R_NilValue, R_NilValue));
        R_PreserveObject(out);
        out
    }
}

/// Initialize the preservation list (unchecked version).
///
/// Skips thread safety checks for performance-critical paths.
///
/// # Safety
///
/// Must be called from the R main thread. Only use in contexts where
/// you're certain you're on the main thread.
#[inline]
unsafe fn init_unchecked() -> SEXP {
    use crate::ffi::{R_PreserveObject_unchecked, Rf_cons_unchecked};

    unsafe {
        let out = Rf_cons_unchecked(R_NilValue, Rf_cons_unchecked(R_NilValue, R_NilValue));
        R_PreserveObject_unchecked(out);
        out
    }
}

/// Get the current thread's preservation list, initializing if needed.
///
/// # Safety
///
/// Must be called from the R main thread.
#[inline]
pub(crate) unsafe fn get() -> SEXP {
    // One global preserve list per thread.
    PRESERVE_LIST.with(|x| *x.get_or_init(|| unsafe { init() }))
}

/// Get the current thread's preservation list (unchecked version).
///
/// Skips thread safety checks for performance-critical paths.
///
/// # Safety
///
/// Must be called from the R main thread. Only use in contexts where
/// you're certain you're on the main thread (ALTREP callbacks, extern "C-unwind" functions).
#[inline]
pub(crate) unsafe fn get_unchecked() -> SEXP {
    // Use unchecked init for full consistency
    PRESERVE_LIST.with(|x| *x.get_or_init(|| unsafe { init_unchecked() }))
}

/// Count the number of currently protected objects.
///
/// This is useful for debugging and testing, but not typically needed
/// in production code.
///
/// # Safety
///
/// Must be called from the R main thread.
#[allow(dead_code)]
#[inline]
pub unsafe fn count() -> R_xlen_t {
    unsafe {
        let head: R_xlen_t = 1;
        let tail: R_xlen_t = 1;
        let list = get();
        Rf_xlength(list) - head - tail
    }
}

/// Count the number of currently protected objects (unchecked version).
///
/// Skips thread safety checks for performance-critical paths.
///
/// # Safety
///
/// Must be called from the R main thread. Only use in contexts where
/// you're certain you're on the main thread.
#[allow(dead_code)]
#[inline]
pub unsafe fn count_unchecked() -> R_xlen_t {
    use crate::ffi::Rf_xlength_unchecked;

    unsafe {
        let head: R_xlen_t = 1;
        let tail: R_xlen_t = 1;
        let list = get_unchecked();
        Rf_xlength_unchecked(list) - head - tail
    }
}

/// Insert a SEXP into the preservation list, protecting it from GC.
///
/// Returns a "cell" (a cons cell) that can later be passed to [`release`]
/// to stop protecting the object.
///
/// If `x` is `R_NilValue`, returns `R_NilValue` without protection
/// (since NIL is never collected).
///
/// # Safety
///
/// Must be called from the R main thread. The returned cell must eventually
/// be passed to [`release`] to prevent leaking memory in the preserve list.
#[inline]
pub unsafe fn insert(x: SEXP) -> SEXP {
    unsafe {
        if std::ptr::addr_eq(x.0, R_NilValue.0) {
            return R_NilValue;
        }

        Rf_protect(x);

        let list = get();

        // head is the list itself; next is the node after head
        let head = list;
        let next = CDR(list);

        // New cell points to current head and next
        let cell = Rf_protect(Rf_cons(head, next));
        SET_TAG(cell, x);

        // Splice cell between head and next
        SETCDR(head, cell);
        SETCAR(next, cell);

        Rf_unprotect(2);

        cell
    }
}

/// Insert a SEXP into the preservation list (unchecked version).
///
/// Skips thread safety checks for performance-critical paths.
/// Otherwise identical to [`insert`].
///
/// # Safety
///
/// Must be called from the R main thread. Only use in contexts where
/// you're certain you're on the main thread (ALTREP callbacks, extern "C-unwind" functions).
/// The returned cell must eventually be passed to [`release_unchecked`].
#[inline]
pub unsafe fn insert_unchecked(x: SEXP) -> SEXP {
    use crate::ffi::{
        CDR_unchecked, Rf_cons_unchecked, Rf_protect_unchecked, Rf_unprotect_unchecked,
        SET_TAG_unchecked, SETCAR_unchecked, SETCDR_unchecked,
    };

    unsafe {
        if std::ptr::addr_eq(x.0, R_NilValue.0) {
            return R_NilValue;
        }

        Rf_protect_unchecked(x);

        let list = get_unchecked();

        // head is the list itself; next is the node after head
        let head = list;
        let next = CDR_unchecked(list);

        // New cell points to current head and next
        let cell = Rf_protect_unchecked(Rf_cons_unchecked(head, next));
        SET_TAG_unchecked(cell, x);

        // Splice cell between head and next
        SETCDR_unchecked(head, cell);
        SETCAR_unchecked(next, cell);

        Rf_unprotect_unchecked(2);

        cell
    }
}

/// Release a previously protected SEXP from the preservation list.
///
/// The `cell` parameter should be a value returned from [`insert`].
///
/// If `cell` is `R_NilValue`, this is a no-op.
///
/// # Safety
///
/// Must be called from the R main thread. The `cell` must be a valid
/// cell returned from [`insert`] and must not have been released already.
#[inline]
pub unsafe fn release(cell: SEXP) {
    unsafe {
        if std::ptr::addr_eq(cell.0, R_NilValue.0) {
            return;
        }

        // Neighbors around the cell
        let lhs = CAR(cell);
        let rhs = CDR(cell);

        // Bypass cell
        SETCDR(lhs, rhs);
        SETCAR(rhs, lhs);

        // Optional hygiene (unnecessary but can help catch bugs)
        // SET_TAG(cell, R_NilValue);
        // SETCAR(cell, R_NilValue);
        // SETCDR(cell, R_NilValue);
    }
}

/// Release a previously protected SEXP (unchecked version).
///
/// Skips thread safety checks for performance-critical paths.
/// Otherwise identical to [`release`].
///
/// # Safety
///
/// Must be called from the R main thread. Only use in contexts where
/// you're certain you're on the main thread. The `cell` must be a valid
/// cell returned from [`insert_unchecked`] and must not have been released already.
#[inline]
pub unsafe fn release_unchecked(cell: SEXP) {
    use crate::ffi::{CAR_unchecked, CDR_unchecked, SETCAR_unchecked, SETCDR_unchecked};

    unsafe {
        if std::ptr::addr_eq(cell.0, R_NilValue.0) {
            return;
        }

        // Neighbors around the cell
        let lhs = CAR_unchecked(cell);
        let rhs = CDR_unchecked(cell);

        // Bypass cell
        SETCDR_unchecked(lhs, rhs);
        SETCAR_unchecked(rhs, lhs);
    }
}
