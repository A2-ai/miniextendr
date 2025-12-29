//! GC protection tools built on R's PROTECT stack.
//!
//! This module provides RAII wrappers around R's GC protection primitives.
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
//! ## When to Use Each
//!
//! **Use `gc_protect` (this module) when:**
//! - You allocate R objects during a `.Call` and need them protected until return
//! - You want RAII-based automatic balancing of PROTECT/UNPROTECT
//! - Protection is short-lived (within a single function)
//!
//! **Use [`preserve`](crate::preserve) when:**
//! - Objects must survive across multiple `.Call` invocations
//! - You need to release protections in arbitrary order
//! - Example: [`RAllocator`](crate::RAllocator) backing memory
//!
//! **Use [`ExternalPtr`](struct@crate::ExternalPtr) when:**
//! - You want R to own a Rust value
//! - The Rust value should be dropped when R garbage collects the pointer
//! - You're exposing Rust structs to R code
//!
//! ## Visual Overview
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │  .Call("my_func", x)                                            │
//! │  ┌──────────────────────────────────────────────────────────┐   │
//! │  │  ProtectScope::new()                                     │   │
//! │  │  ├── protect(Rf_allocVector(...))  // temp allocation    │   │
//! │  │  ├── protect(Rf_allocVector(...))  // another temp       │   │
//! │  │  └── UNPROTECT(n) on scope drop                          │   │
//! │  └──────────────────────────────────────────────────────────┘   │
//! │                          ↓ return SEXP                          │
//! └─────────────────────────────────────────────────────────────────┘
//!
//! ┌─────────────────────────────────────────────────────────────────┐
//! │  preserve (objects surviving across .Calls)                     │
//! │  ├── preserve::insert(sexp)   // add to linked list             │
//! │  ├── ... multiple .Calls ...  // object stays protected         │
//! │  └── preserve::release(cell)  // remove when done               │
//! └─────────────────────────────────────────────────────────────────┘
//!
//! ┌─────────────────────────────────────────────────────────────────┐
//! │  ExternalPtr<MyStruct> (R owns Rust data)                       │
//! │  ├── Construction: temporary Rf_protect                         │
//! │  ├── Return to R → R owns the EXTPTRSXP                         │
//! │  └── R GC → finalizer runs → Rust Drop executes                 │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Types in This Module
//!
//! This module provides RAII wrappers around R's GC protection primitives:
//!
//! | Type | Purpose |
//! |------|---------|
//! | [`ProtectScope`] | Batch protection with automatic `UNPROTECT(n)` on drop |
//! | [`Root<'scope>`] | Lightweight handle tied to a scope's lifetime |
//! | [`OwnedProtect`] | Single-value RAII guard for simple cases |
//! | [`ReprotectSlot<'scope>`] | Protected slot supporting replace-under-protection |
//!
//! # Design Principles
//!
//! - `ProtectScope` owns the responsibility of calling `UNPROTECT(n)`
//! - `Root<'a>` is a move-friendly, non-dropping handle whose lifetime ties to the scope
//! - `ReprotectSlot<'a>` supports replace-under-protection via `PROTECT_WITH_INDEX`/`REPROTECT`
//!
//! # Safety Model
//!
//! These tools are `unsafe` to create because they require:
//!
//! 1. **Running on the R main thread** - R's API is not thread-safe
//! 2. **No panics across FFI** - Rust panics must not unwind across C boundary
//! 3. **Understanding R errors** - If R raises an error (`longjmp`), Rust destructors
//!    will not run, so scope-based unprotection will leak
//!
//! For cleanup that survives R errors, use `R_UnwindProtect` boundaries in your
//! `.Call` trampoline (see [`unwind_protect`](crate::unwind_protect)).
//!
//! # Example
//!
//! ```ignore
//! use miniextendr_api::gc_protect::ProtectScope;
//! use miniextendr_api::ffi::SEXP;
//!
//! unsafe fn process_vectors(x: SEXP, y: SEXP) -> SEXP {
//!     let scope = ProtectScope::new();
//!
//!     // Protect multiple values
//!     let x = scope.protect(x);
//!     let y = scope.protect(y);
//!
//!     // Work with protected values...
//!     let result = scope.protect(some_r_function(x.get(), y.get()));
//!
//!     result.into_raw()
//! } // UNPROTECT(3) called automatically
//! ```

use crate::ffi::{R_ProtectWithIndex, R_Reprotect, Rf_protect, Rf_unprotect, SEXP};
use core::cell::Cell;
use core::marker::PhantomData;
use std::rc::Rc;

/// R's PROTECT_INDEX type (just `c_int` under the hood).
pub type ProtectIndex = ::std::os::raw::c_int;

/// Enforces `!Send + !Sync` (R API is not thread-safe).
type NoSendSync = PhantomData<Rc<()>>;

// =============================================================================
// ProtectScope
// =============================================================================

/// A scope that automatically balances `UNPROTECT(n)` on drop.
///
/// This is the primary tool for managing GC protection in batch operations.
/// Each call to [`protect`][Self::protect] or [`protect_with_index`][Self::protect_with_index]
/// increments an internal counter; when the scope is dropped, `UNPROTECT(n)` is called.
///
/// # Example
///
/// ```ignore
/// unsafe fn my_call(x: SEXP, y: SEXP) -> SEXP {
///     let scope = ProtectScope::new();
///     let x = scope.protect(x);
///     let y = scope.protect(y);
///
///     // Both x and y are protected until scope drops
///     let result = scope.protect(some_operation(x.get(), y.get()));
///     result.get()
/// } // UNPROTECT(3)
/// ```
///
/// # Nested Scopes
///
/// Scopes can be nested. Each scope tracks only its own protections:
///
/// ```ignore
/// unsafe fn outer(x: SEXP) -> SEXP {
///     let scope = ProtectScope::new();
///     let x = scope.protect(x);
///
///     let result = helper(&scope, x.get());
///     scope.protect(result).get()
/// } // UNPROTECT(2)
///
/// unsafe fn helper(_parent: &ProtectScope, x: SEXP) -> SEXP {
///     let scope = ProtectScope::new();
///     let temp = scope.protect(allocate_something());
///     combine(x, temp.get())
/// } // UNPROTECT(1) - only this scope's protections
/// ```
pub struct ProtectScope {
    n: Cell<i32>,
    armed: Cell<bool>,
    _nosend: NoSendSync,
}

impl ProtectScope {
    /// Create a new protection scope.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn new() -> Self {
        Self {
            n: Cell::new(0),
            armed: Cell::new(true),
            _nosend: PhantomData,
        }
    }

    /// Protect `x` and return a rooted handle tied to this scope.
    ///
    /// This always calls `Rf_protect`. The protection is released when
    /// the scope is dropped (along with all other protections in this scope).
    ///
    /// # Safety
    ///
    /// - Must be called from the R main thread
    /// - `x` must be a valid SEXP
    #[inline]
    pub unsafe fn protect<'a>(&'a self, x: SEXP) -> Root<'a> {
        let y = unsafe { Rf_protect(x) };
        self.n.set(self.n.get() + 1);
        Root {
            sexp: y,
            _scope: PhantomData,
        }
    }

    /// Protect and return the raw `SEXP` (sometimes more convenient).
    ///
    /// # Safety
    ///
    /// Same as [`protect`][Self::protect].
    #[inline]
    pub unsafe fn protect_raw(&self, x: SEXP) -> SEXP {
        let y = unsafe { Rf_protect(x) };
        self.n.set(self.n.get() + 1);
        y
    }

    /// Protect `x` with an index slot so it can be replaced later via [`R_Reprotect`].
    ///
    /// Use this when you need to update a protected value in-place without
    /// growing the protection stack.
    ///
    /// # Safety
    ///
    /// - Must be called from the R main thread
    /// - `x` must be a valid SEXP
    ///
    /// # Example
    ///
    /// ```ignore
    /// unsafe fn accumulate(values: &[SEXP]) -> SEXP {
    ///     let scope = ProtectScope::new();
    ///     let slot = scope.protect_with_index(values[0]);
    ///
    ///     for &v in &values[1..] {
    ///         let combined = combine(slot.get(), v);
    ///         slot.set(combined);  // Reprotect without growing stack
    ///     }
    ///
    ///     slot.get()
    /// }
    /// ```
    #[inline]
    pub unsafe fn protect_with_index<'a>(&'a self, x: SEXP) -> ReprotectSlot<'a> {
        let mut idx: ProtectIndex = 0;
        unsafe { R_ProtectWithIndex(x, &mut idx as *mut ProtectIndex) };
        self.n.set(self.n.get() + 1);
        ReprotectSlot {
            idx,
            cur: Cell::new(x),
            _scope: PhantomData,
            _nosend: PhantomData,
        }
    }

    /// Protect two values at once (convenience method).
    ///
    /// # Safety
    ///
    /// Same as [`protect`][Self::protect].
    #[inline]
    pub unsafe fn protect2<'a>(&'a self, a: SEXP, b: SEXP) -> (Root<'a>, Root<'a>) {
        // SAFETY: caller guarantees R main thread and valid SEXPs
        unsafe { (self.protect(a), self.protect(b)) }
    }

    /// Protect three values at once (convenience method).
    ///
    /// # Safety
    ///
    /// Same as [`protect`][Self::protect].
    #[inline]
    pub unsafe fn protect3<'a>(
        &'a self,
        a: SEXP,
        b: SEXP,
        c: SEXP,
    ) -> (Root<'a>, Root<'a>, Root<'a>) {
        // SAFETY: caller guarantees R main thread and valid SEXPs
        unsafe { (self.protect(a), self.protect(b), self.protect(c)) }
    }

    /// Return the current protection count.
    #[inline]
    pub fn count(&self) -> i32 {
        self.n.get()
    }

    /// Escape hatch: disable `UNPROTECT` on drop.
    ///
    /// After calling this, the scope will **not** unprotect its values when dropped.
    /// You become responsible for ensuring correct unprotection.
    ///
    /// # Safety
    ///
    /// You must ensure the protects performed in this scope are correctly
    /// unprotected elsewhere, or you will leak protect stack entries.
    #[inline]
    pub unsafe fn disarm(&self) {
        self.armed.set(false);
    }

    /// Re-arm a previously disarmed scope.
    ///
    /// # Safety
    ///
    /// Only call if you know the scope was disarmed and you want to restore
    /// automatic unprotection. Be careful not to double-unprotect.
    #[inline]
    pub unsafe fn rearm(&self) {
        self.armed.set(true);
    }
}

impl Drop for ProtectScope {
    #[inline]
    fn drop(&mut self) {
        if !self.armed.get() {
            return;
        }
        let n = self.n.replace(0);
        if n > 0 {
            unsafe { Rf_unprotect(n) };
        }
    }
}

impl Default for ProtectScope {
    /// Create a new scope. Equivalent to `unsafe { ProtectScope::new() }`.
    ///
    /// # Safety
    ///
    /// The caller must ensure this is called from the R main thread.
    #[inline]
    fn default() -> Self {
        // SAFETY: This is a foot-gun but matches the pattern of other R interop code.
        // Users should prefer `unsafe { ProtectScope::new() }` for clarity.
        unsafe { Self::new() }
    }
}

// =============================================================================
// Root
// =============================================================================

/// A rooted SEXP tied to the lifetime of a [`ProtectScope`].
///
/// This type has **no `Drop`**. The scope owns unprotection responsibility.
/// This makes `Root` cheap to move and copy (it's just a pointer + lifetime).
///
/// # Lifetime
///
/// The `'a` lifetime ties the root to its creating scope. The compiler ensures
/// you cannot use the root after the scope has been dropped.
#[derive(Clone, Copy)]
pub struct Root<'a> {
    sexp: SEXP,
    _scope: PhantomData<&'a ProtectScope>,
}

impl<'a> Root<'a> {
    /// Get the underlying SEXP.
    #[inline]
    pub fn get(&self) -> SEXP {
        self.sexp
    }

    /// Consume the root and return the underlying SEXP.
    ///
    /// The SEXP remains protected until the scope drops.
    #[inline]
    pub fn into_raw(self) -> SEXP {
        self.sexp
    }
}

impl<'a> std::ops::Deref for Root<'a> {
    type Target = SEXP;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.sexp
    }
}

// =============================================================================
// OwnedProtect
// =============================================================================

/// A single-object RAII guard: `PROTECT` on create, `UNPROTECT(1)` on drop.
///
/// Use this for simple cases where you're protecting a single value and
/// don't need the batching benefits of [`ProtectScope`].
///
/// # Example
///
/// ```ignore
/// unsafe fn allocate_and_return() -> SEXP {
///     let guard = OwnedProtect::new(Rf_allocVector(REALSXP, 10));
///     fill_vector(guard.get());
///     guard.into_inner()  // Still protected, but guard won't unprotect on drop
/// }
/// ```
///
/// # Warning: Stack Ordering
///
/// `OwnedProtect` uses `UNPROTECT(1)`, which removes the **top** of the protection
/// stack. If you have nested protections from other sources, the drop order matters!
///
/// For complex scenarios, prefer [`ProtectScope`] which unprotects all its values
/// at once when dropped.
pub struct OwnedProtect {
    sexp: SEXP,
    armed: bool,
    _nosend: NoSendSync,
}

impl OwnedProtect {
    /// Create a new protection guard for `x`.
    ///
    /// Calls `Rf_protect(x)` immediately.
    ///
    /// # Safety
    ///
    /// - Must be called from the R main thread
    /// - `x` must be a valid SEXP
    #[inline]
    pub unsafe fn new(x: SEXP) -> Self {
        let y = unsafe { Rf_protect(x) };
        Self {
            sexp: y,
            armed: true,
            _nosend: PhantomData,
        }
    }

    /// Get the protected SEXP.
    #[inline]
    pub fn get(&self) -> SEXP {
        self.sexp
    }

    /// Consume the guard, returning the SEXP.
    ///
    /// The SEXP is **still protected** after this call. The guard's drop
    /// will not run (it's consumed), so unprotection won't happen.
    ///
    /// Use this when returning a protected value from a function where the
    /// caller will take over protection responsibility.
    #[inline]
    pub fn into_inner(mut self) -> SEXP {
        // Disarm so drop doesn't unprotect
        self.armed = false;
        self.sexp
    }

    /// Escape hatch: do not `UNPROTECT(1)` on drop.
    ///
    /// # Safety
    ///
    /// Leaks one protection entry unless unprotected elsewhere.
    #[inline]
    pub unsafe fn forget(mut self) {
        self.armed = false;
        core::mem::forget(self);
    }
}

impl Drop for OwnedProtect {
    #[inline]
    fn drop(&mut self) {
        if self.armed {
            unsafe { Rf_unprotect(1) };
        }
    }
}

impl std::ops::Deref for OwnedProtect {
    type Target = SEXP;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.sexp
    }
}

// =============================================================================
// ReprotectSlot
// =============================================================================

/// A protected slot created with `R_ProtectWithIndex` and updated with `R_Reprotect`.
///
/// This allows updating a protected value in-place without growing the protection
/// stack. Useful for loops that repeatedly allocate and update a value.
///
/// The slot is valid only while the creating [`ProtectScope`] is alive.
///
/// # Example
///
/// ```ignore
/// unsafe fn sum_allocated_vectors(n: i32) -> SEXP {
///     let scope = ProtectScope::new();
///
///     // Initial allocation
///     let slot = scope.protect_with_index(Rf_allocVector(REALSXP, 10));
///
///     for i in 0..n {
///         // Each iteration allocates a new vector
///         let new_vec = compute_step(slot.get(), i);
///         slot.set(new_vec);  // Replace without growing protect stack
///     }
///
///     slot.get()
/// }
/// ```
pub struct ReprotectSlot<'a> {
    idx: ProtectIndex,
    cur: Cell<SEXP>,
    _scope: PhantomData<&'a ProtectScope>,
    _nosend: NoSendSync,
}

impl<'a> ReprotectSlot<'a> {
    /// Get the currently protected SEXP.
    #[inline]
    pub fn get(&self) -> SEXP {
        self.cur.get()
    }

    /// Replace the protected value in-place using `R_Reprotect`.
    ///
    /// The new value `x` becomes protected in this slot, and the old value
    /// is no longer protected (but may still be rooted elsewhere).
    ///
    /// Returns a [`Root`] for convenience (same lifetime as the slot).
    ///
    /// # Safety
    ///
    /// - Must be called from the R main thread
    /// - `x` must be a valid SEXP
    #[inline]
    pub unsafe fn set(&self, x: SEXP) -> Root<'a> {
        unsafe { R_Reprotect(x, self.idx) };
        self.cur.set(x);
        Root {
            sexp: x,
            _scope: PhantomData,
        }
    }

    /// Replace the protected value, returning the raw SEXP.
    ///
    /// # Safety
    ///
    /// Same as [`set`][Self::set].
    #[inline]
    pub unsafe fn set_raw(&self, x: SEXP) -> SEXP {
        unsafe { R_Reprotect(x, self.idx) };
        self.cur.set(x);
        x
    }
}

impl<'a> std::ops::Deref for ReprotectSlot<'a> {
    type Target = SEXP;

    #[inline]
    fn deref(&self) -> &Self::Target {
        // This is a bit awkward since we return &SEXP but SEXP is Copy.
        // The Cell prevents us from returning a reference to the inner SEXP.
        // This deref is mostly for ergonomics.
        unsafe { &*(&self.cur as *const Cell<SEXP> as *const SEXP) }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests primarily verify compilation and basic invariants.
    // Full integration testing requires R to be initialized.

    #[test]
    fn protect_scope_has_nosend_marker() {
        // Verify the NoSendSync marker type is present
        // (ProtectScope contains PhantomData<Rc<()>> which makes it !Send + !Sync)
        let _: NoSendSync = PhantomData;
    }

    #[test]
    fn protect_scope_default_count_is_zero() {
        let scope = ProtectScope::default();
        assert_eq!(scope.count(), 0);
    }

    #[test]
    fn root_is_copy() {
        fn assert_copy<T: Copy>() {}
        assert_copy::<Root<'static>>();
    }
}
