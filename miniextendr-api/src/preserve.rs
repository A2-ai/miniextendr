//! R object preservation using a circular doubly-linked list.
//!
//! This module provides a protection scheme for R objects (SEXPs) that need to
//! survive R's garbage collection. It uses a circular doubly-linked list approach
//! similar to cpp11, which has advantages over the R protect stack:
//!
//! - No balance requirement (PROTECT/UNPROTECT pairs must be balanced)
//! - Can release protections in any order
//! - Thread-local storage (each thread has its own preserve list)
//! - More ergonomic with RAII patterns
//!
//! ## Architecture
//!
//! The preservation list is a circular doubly-linked cons list where:
//! - The list itself is preserved with `R_PreserveObject` (never GC'd)
//! - Each protected SEXP is stored as the TAG of a cell
//! - CAR points to previous cell, CDR points to next cell
//! - Head and tail are sentinel nodes
//!
//! ## Safety
//!
//! All functions in this module are unsafe and must be called from the R main thread.
//! Use the safe [`Protected`] wrapper for RAII-style protection.

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
        if x == R_NilValue {
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
        if cell == R_NilValue {
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

/// RAII wrapper for a protected R object.
///
/// Automatically protects the SEXP on creation and releases it on drop.
/// This ensures that the object won't be garbage collected while the
/// `Protected` value is in scope.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::preserve::Protected;
///
/// unsafe {
///     let protected = Protected::new(Rf_allocVector(INTSXP, 10));
///     // The vector is now protected from GC
///     // ... use the SEXP ...
///     // Automatically released when `protected` goes out of scope
/// }
/// ```
///
/// # Safety
///
/// Must only be used from the R main thread. The inner SEXP should not
/// be used after the `Protected` is dropped.
pub struct Protected {
    cell: SEXP,
}

impl Protected {
    /// Protect a SEXP from garbage collection.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn new(sexp: SEXP) -> Self {
        unsafe {
            let cell = insert(sexp);
            Self { cell }
        }
    }

    /// Get the protected SEXP.
    ///
    /// The returned SEXP is valid as long as this `Protected` value
    /// hasn't been dropped.
    #[inline]
    pub fn get(&self) -> SEXP {
        unsafe {
            if self.cell == R_NilValue {
                R_NilValue
            } else {
                crate::ffi::TAG(self.cell)
            }
        }
    }

    /// Consume this protection and return the inner SEXP.
    ///
    /// After calling this, the SEXP is no longer protected.
    ///
    /// # Safety
    ///
    /// The caller must ensure the returned SEXP is either:
    /// - Immediately protected by other means, or
    /// - Stored in a location that R knows about (e.g., returned to R)
    #[inline]
    pub unsafe fn into_inner(self) -> SEXP {
        let sexp = self.get();
        // Prevent drop from releasing
        std::mem::forget(self);
        sexp
    }
}

impl Drop for Protected {
    fn drop(&mut self) {
        unsafe {
            release(self.cell);
        }
    }
}

// Safety: Protected is only used on the R main thread
unsafe impl Send for Protected {}

// Tests for this module require R runtime and should be run via R CMD check.
// They are located in rpkg/tests/ as integration tests.
