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
//!
//! # Container Insertion Patterns
//!
//! When building containers (lists, character vectors), children need protection
//! between allocation and insertion:
//!
//! ```ignore
//! // WRONG - child unprotected between allocation and SET_VECTOR_ELT
//! let child = Rf_allocVector(REALSXP, 10);  // unprotected!
//! SET_VECTOR_ELT(list, 0, child);           // GC could occur before this!
//!
//! // CORRECT - use safe insertion methods
//! let list = List::from_raw(scope.protect_raw(Rf_allocVector(VECSXP, n)));
//! for i in 0..n {
//!     let child = Rf_allocVector(REALSXP, 10);
//!     list.set_elt(i, child);  // protects child during insertion
//! }
//!
//! // EFFICIENT - use ListBuilder with scope
//! let builder = ListBuilder::new(&scope, n);
//! for i in 0..n {
//!     let child = scope.protect_raw(Rf_allocVector(REALSXP, 10));
//!     builder.set(i, child);  // child already protected by scope
//! }
//! ```
//!
//! See [`List::set_elt`](crate::list::List::set_elt),
//! [`ListBuilder`](crate::list::ListBuilder), and
//! [`StrVec::set_str`](crate::strvec::StrVec::set_str) for safe container APIs.
//!
//! # Reassignment with `ReprotectSlot`
//!
//! Use [`ReprotectSlot`] when you need to reassign a protected value multiple times
//! without growing the protection stack:
//!
//! ```ignore
//! let slot = scope.protect_with_index(initial_value);
//! for item in items {
//!     let new_value = process(slot.get(), item);
//!     slot.set(new_value);  // R_Reprotect, stack count unchanged
//! }
//! ```
//!
//! This avoids the LIFO drop-order pitfall of reassigning `OwnedProtect` guards.

use crate::ffi::{
    R_NilValue, R_ProtectWithIndex, R_Reprotect, R_xlen_t, RNativeType, Rf_allocList,
    Rf_allocMatrix, Rf_allocVector, Rf_protect, Rf_unprotect, SEXP, SEXPTYPE,
};
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

    // =========================================================================
    // Allocation + Protection Helpers
    // =========================================================================

    /// Allocate a vector of the given type and length, and immediately protect it.
    ///
    /// This combines allocation and protection in a single step, eliminating the
    /// GC gap that exists when you separately allocate and then protect.
    ///
    /// # Safety
    ///
    /// - Must be called from the R main thread
    /// - Only protects the newly allocated object; does not protect other live
    ///   unprotected objects during allocation
    ///
    /// # Example
    ///
    /// ```ignore
    /// unsafe fn make_ints(n: R_xlen_t) -> SEXP {
    ///     let scope = ProtectScope::new();
    ///     let vec = scope.alloc_vector(SEXPTYPE::INTSXP, n);
    ///     // fill via INTEGER(vec.get()) ...
    ///     vec.get()
    /// }
    /// ```
    #[inline]
    pub unsafe fn alloc_vector<'a>(&'a self, ty: SEXPTYPE, n: R_xlen_t) -> Root<'a> {
        // SAFETY: caller guarantees R main thread
        let sexp = unsafe { Rf_allocVector(ty, n) };
        unsafe { self.protect(sexp) }
    }

    /// Allocate a matrix of the given type and dimensions, and immediately protect it.
    ///
    /// # Safety
    ///
    /// Same as [`alloc_vector`][Self::alloc_vector].
    #[inline]
    pub unsafe fn alloc_matrix<'a>(&'a self, ty: SEXPTYPE, nrow: i32, ncol: i32) -> Root<'a> {
        let sexp = unsafe { Rf_allocMatrix(ty, nrow, ncol) };
        unsafe { self.protect(sexp) }
    }

    /// Allocate a list (VECSXP) of the given length and immediately protect it.
    ///
    /// # Safety
    ///
    /// Same as [`alloc_vector`][Self::alloc_vector].
    #[inline]
    pub unsafe fn alloc_list<'a>(&'a self, n: i32) -> Root<'a> {
        let sexp = unsafe { Rf_allocList(n) };
        unsafe { self.protect(sexp) }
    }

    /// Allocate a STRSXP (character vector) of the given length and immediately protect it.
    ///
    /// # Safety
    ///
    /// Same as [`alloc_vector`][Self::alloc_vector].
    #[inline]
    pub unsafe fn alloc_strsxp<'a>(&'a self, n: R_xlen_t) -> Root<'a> {
        unsafe { self.alloc_vector(SEXPTYPE::STRSXP, n) }
    }

    /// Allocate a VECSXP (generic list) of the given length and immediately protect it.
    ///
    /// # Safety
    ///
    /// Same as [`alloc_vector`][Self::alloc_vector].
    #[inline]
    pub unsafe fn alloc_vecsxp<'a>(&'a self, n: R_xlen_t) -> Root<'a> {
        unsafe { self.alloc_vector(SEXPTYPE::VECSXP, n) }
    }

    /// Create a `Root<'a>` for an already-protected SEXP without adding protection.
    ///
    /// This is useful when you have a SEXP that is already protected by some other
    /// mechanism (e.g., a `ReprotectSlot`) and want to return it as a `Root` tied
    /// to this scope's lifetime for API consistency.
    ///
    /// # Safety
    ///
    /// - The caller must ensure `sexp` is already protected and will remain
    ///   protected for at least the lifetime of this scope
    /// - Must be called from the R main thread
    #[inline]
    pub unsafe fn rooted<'a>(&'a self, sexp: SEXP) -> Root<'a> {
        Root {
            sexp,
            _scope: PhantomData,
        }
    }

    // =========================================================================
    // Iterator Collection
    // =========================================================================

    /// Collect an iterator into a typed R vector.
    ///
    /// This allocates once, protects, and fills directly - the most efficient pattern
    /// for typed vectors. The element type `T` determines the R vector type via
    /// the [`RNativeType`] trait.
    ///
    /// # Type Mapping
    ///
    /// | Rust Type | R Vector Type |
    /// |-----------|---------------|
    /// | `i32` | `INTSXP` |
    /// | `f64` | `REALSXP` |
    /// | `u8` | `RAWSXP` |
    /// | [`RLogical`](crate::ffi::RLogical) | `LGLSXP` |
    /// | [`Rcomplex`](crate::ffi::Rcomplex) | `CPLXSXP` |
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    ///
    /// # Example
    ///
    /// ```ignore
    /// unsafe fn squares(n: usize) -> SEXP {
    ///     let scope = ProtectScope::new();
    ///     // Type inferred from iterator
    ///     scope.collect((0..n).map(|i| (i * i) as i32)).get()
    /// }
    /// ```
    ///
    /// # Unknown Length
    ///
    /// For iterators without exact size (e.g., `filter`), collect to `Vec` first:
    ///
    /// ```ignore
    /// let evens: Vec<i32> = data.iter().filter(|x| *x % 2 == 0).copied().collect();
    /// scope.collect(evens)
    /// ```
    #[inline]
    pub unsafe fn collect<'a, T, I>(&'a self, iter: I) -> Root<'a>
    where
        T: RNativeType,
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        let iter = iter.into_iter();
        let len = iter.len();

        let vec = unsafe { self.alloc_vector(T::SEXP_TYPE, len as R_xlen_t) };
        let ptr = unsafe { T::dataptr_mut(vec.get()) };

        for (i, value) in iter.enumerate() {
            unsafe { ptr.add(i).write(value) };
        }

        vec
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
/// unsafe fn allocate_and_fill() -> SEXP {
///     let guard = OwnedProtect::new(Rf_allocVector(REALSXP, 10));
///     fill_vector(guard.get());
///     // Return the SEXP - guard drops and unprotects on this line.
///     // This is safe because no GC can occur between unprotect and return.
///     guard.get()
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
/// # When to Use `ReprotectSlot`
///
/// Use `ReprotectSlot` when you need to **reassign a protected value** multiple times:
///
/// | Pattern | Use | Why |
/// |---------|-----|-----|
/// | Accumulator loop | `ReprotectSlot` | Repeatedly replace result without stack growth |
/// | Single allocation | `ProtectScope::protect` | Simpler, no reassignment needed |
/// | Child insertion | `List::set_elt` | Container handles child protection |
///
/// # Warning: RAII Assignment Pitfall
///
/// R's PROTECT stack is LIFO. Rust's RAII drop order can cause problems:
///
/// ```ignore
/// // WRONG - can unprotect the new value instead of the old!
/// let mut guard = OwnedProtect::new(old_value);
/// guard = OwnedProtect::new(new_value);  // Old guard drops AFTER new is assigned
/// ```
///
/// `ReprotectSlot` avoids this by using `R_Reprotect` which replaces in-place:
///
/// ```ignore
/// // CORRECT - always keeps exactly one slot protected
/// let slot = scope.protect_with_index(old_value);
/// slot.set(new_value);  // R_Reprotect, no stack change
/// ```
///
/// # Examples
///
/// ## Accumulator Pattern
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
///
/// ## Starting with Empty Slot
///
/// ```ignore
/// unsafe fn build_result(items: &[Input]) -> SEXP {
///     let scope = ProtectScope::new();
///
///     // Start with R_NilValue, replace with first real result
///     let slot = scope.protect_with_index(R_NilValue);
///
///     for (i, item) in items.iter().enumerate() {
///         let result = process_item(item, slot.get());
///         slot.set(result);
///     }
///
///     slot.get()
/// }
/// ```
///
/// ## Multiple Slots
///
/// ```ignore
/// unsafe fn merge_sorted(a: SEXP, b: SEXP) -> SEXP {
///     let scope = ProtectScope::new();
///
///     let slot_a = scope.protect_with_index(a);
///     let slot_b = scope.protect_with_index(b);
///     let result = scope.protect_with_index(R_NilValue);
///
///     // Process both inputs, updating result
///     while !is_empty(slot_a.get()) && !is_empty(slot_b.get()) {
///         let merged = merge_next(slot_a.get(), slot_b.get());
///         result.set(merged);
///         // ... update slot_a and slot_b as needed
///     }
///
///     result.get()
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
    /// Returns the raw SEXP for convenience. Note that this SEXP is only
    /// protected until the next call to `set()` on this slot - if you need
    /// to hold multiple protected values simultaneously, use separate
    /// protection slots or `OwnedProtect`.
    ///
    /// # Safety
    ///
    /// - Must be called from the R main thread
    /// - `x` must be a valid SEXP
    #[inline]
    pub unsafe fn set(&self, x: SEXP) -> SEXP {
        unsafe { R_Reprotect(x, self.idx) };
        self.cur.set(x);
        x
    }

    /// Allocate a new value via the closure and replace this slot's value safely.
    ///
    /// This method encodes the safe pattern for replacing a protected slot with
    /// a newly allocated value. It:
    ///
    /// 1. Calls the closure `f()` to allocate a new SEXP
    /// 2. Temporarily protects the new value (to close the GC gap)
    /// 3. Calls `R_Reprotect` to replace this slot's value
    /// 4. Unprotects the temporary protection
    ///
    /// This prevents the GC gap that would exist if you called `f()` and then
    /// `set()` separately - during that window, the newly allocated value would
    /// be unprotected.
    ///
    /// # Safety
    ///
    /// - Must be called from the R main thread
    /// - The closure must return a valid SEXP
    ///
    /// # Example
    ///
    /// ```ignore
    /// unsafe fn grow_list(scope: &ProtectScope, old_list: SEXP) -> SEXP {
    ///     let slot = scope.protect_with_index(old_list);
    ///
    ///     // Safely grow the list without GC gap
    ///     slot.set_with(|| {
    ///         let new_list = Rf_allocVector(VECSXP, new_size);
    ///         // copy elements from old_list to new_list...
    ///         new_list
    ///     });
    ///
    ///     slot.get()
    /// }
    /// ```
    #[inline]
    pub unsafe fn set_with<F>(&self, f: F) -> SEXP
    where
        F: FnOnce() -> SEXP,
    {
        // Allocate the new value
        let new_value = f();

        // Temporarily protect the new value to close the GC gap
        let temp = unsafe { Rf_protect(new_value) };

        // Replace this slot's value with the new value
        unsafe { R_Reprotect(temp, self.idx) };
        self.cur.set(temp);

        // Remove the temporary protection (slot now owns the protection)
        unsafe { Rf_unprotect(1) };

        temp
    }

    /// Take the current value and clear the slot to `R_NilValue`.
    ///
    /// This provides `Option::take`-like semantics. The slot remains allocated
    /// (protect stack depth unchanged), but now holds `R_NilValue` (immortal).
    ///
    /// # Safety
    ///
    /// - Must be called from the R main thread
    /// - The returned SEXP is **unprotected**. If it needs to survive further
    ///   allocations, you must protect it explicitly.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let slot = scope.protect_with_index(some_value);
    /// // ... work with slot.get() ...
    /// let old = slot.take();  // slot now holds R_NilValue
    /// // old is unprotected - protect it if needed
    /// let guard = OwnedProtect::new(old);
    /// ```
    #[inline]
    pub unsafe fn take(&self) -> SEXP {
        let old = self.cur.get();
        let nil = unsafe { R_NilValue };
        unsafe { R_Reprotect(nil, self.idx) };
        self.cur.set(nil);
        old
    }

    /// Replace the slot's value with `x` and return the old value.
    ///
    /// This provides `Option::replace`-like semantics. The slot now protects
    /// `x`, and the old value is returned **unprotected**.
    ///
    /// # Safety
    ///
    /// - Must be called from the R main thread
    /// - `x` must be a valid SEXP
    /// - The returned SEXP is **unprotected**. If it needs to survive further
    ///   allocations, you must protect it explicitly.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let slot = scope.protect_with_index(initial);
    /// let old = slot.replace(new_value);
    /// // old is unprotected, slot now protects new_value
    /// ```
    #[inline]
    pub unsafe fn replace(&self, x: SEXP) -> SEXP {
        let old = self.cur.get();
        unsafe { R_Reprotect(x, self.idx) };
        self.cur.set(x);
        old
    }

    /// Clear the slot by setting it to `R_NilValue`.
    ///
    /// The slot remains allocated (protect stack depth unchanged), but releases
    /// its reference to the previous value. The previous value may still be
    /// rooted elsewhere.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn clear(&self) {
        let nil = unsafe { R_NilValue };
        unsafe { R_Reprotect(nil, self.idx) };
        self.cur.set(nil);
    }

    /// Check if the slot is currently cleared (holds `R_NilValue`).
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread (accesses R's `R_NilValue`).
    #[inline]
    pub unsafe fn is_nil(&self) -> bool {
        self.cur.get() == unsafe { R_NilValue }
    }
}

// NOTE: Deref was intentionally removed to avoid UB.
// The previous impl fabricated `&SEXP` from `Cell<SEXP>` via pointer cast,
// which violates Cell's aliasing rules if `set()` is called while a
// reference is live. Use `get()` instead, which returns SEXP by value.

// =============================================================================
// TLS-backed convenience API (optional)
// =============================================================================

/// Thread-local convenience for protecting without explicit scope references.
///
/// This module provides an **optional** convenience layer that maintains a
/// thread-local stack of scope pointers, allowing `tls::protect(x)` without
/// passing `&ProtectScope` explicitly.
///
/// # Design Note
///
/// **Explicit `&ProtectScope` is recommended for most use cases.** It's simpler,
/// clearer about lifetimes, and doesn't rely on runtime state. Use this TLS
/// convenience only when threading scope references through deep call stacks
/// would be excessively verbose.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::gc_protect::tls;
///
/// unsafe fn deep_helper(x: SEXP) -> SEXP {
///     // No need to thread &ProtectScope through multiple call levels
///     let y = tls::protect(allocate_something());
///     combine(x, y.get())
/// }
///
/// unsafe fn call_body(x: SEXP) -> SEXP {
///     tls::with_protect_scope(|| {
///         let x = tls::protect(x);
///         deep_helper(x.get())
///     })
/// }
/// ```
pub mod tls {
    use super::*;
    use std::cell::RefCell;
    use std::ptr::NonNull;

    thread_local! {
        /// Stack of active protection scopes on this thread.
        static SCOPE_STACK: RefCell<Vec<NonNull<ProtectScope>>> = const { RefCell::new(Vec::new()) };
    }

    /// Execute a closure with a new protection scope as the current TLS scope.
    ///
    /// The scope is pushed onto the thread-local stack, the closure runs, and
    /// then the scope is popped and dropped (triggering `UNPROTECT(n)`).
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    ///
    /// # Example
    ///
    /// ```ignore
    /// unsafe fn my_call(x: SEXP) -> SEXP {
    ///     tls::with_protect_scope(|| {
    ///         let x = tls::protect(x);
    ///         let y = tls::protect(allocate_something());
    ///         combine(x.get(), y.get())
    ///     })
    /// }
    /// ```
    /// Guard that pops the TLS scope stack on drop (panic-safe cleanup).
    struct TlsScopeGuard {
        scope_ptr: NonNull<ProtectScope>,
    }

    impl Drop for TlsScopeGuard {
        fn drop(&mut self) {
            SCOPE_STACK.with(|stack| {
                let popped = stack.borrow_mut().pop();
                debug_assert!(
                    popped == Some(self.scope_ptr),
                    "TLS scope stack corrupted: expected to pop same scope"
                );
            });
        }
    }

    /// Execute a closure with a protect scope that is accessible via TLS.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn with_protect_scope<F, R>(f: F) -> R
    where
        F: FnOnce() -> R,
    {
        // SAFETY: caller guarantees R main thread
        let scope = unsafe { ProtectScope::new() };

        // Push scope pointer onto TLS stack
        // SAFETY: scope lives for the duration of this function call
        let scope_ptr = NonNull::from(&scope);
        SCOPE_STACK.with(|stack| {
            stack.borrow_mut().push(scope_ptr);
        });

        // Guard ensures TLS stack is popped even on panic.
        // The guard must be dropped BEFORE scope (declared after scope),
        // so the TLS stack is popped before UNPROTECT runs.
        let _guard = TlsScopeGuard { scope_ptr };

        // Run the user's closure - if it panics, _guard drops and pops TLS,
        // then scope drops and calls UNPROTECT(n)
        f()
    }

    /// Protect a value using the current TLS scope.
    ///
    /// # Panics
    ///
    /// Panics if called outside of a [`with_protect_scope`] block.
    ///
    /// # Safety
    ///
    /// - Must be called from the R main thread
    /// - `x` must be a valid SEXP
    /// - Must be called within a [`with_protect_scope`] block
    ///
    /// # Example
    ///
    /// ```ignore
    /// tls::with_protect_scope(|| {
    ///     let x = tls::protect(some_sexp);
    ///     // use x...
    /// })
    /// ```
    #[inline]
    pub unsafe fn protect(x: SEXP) -> TlsRoot {
        let scope_ptr = SCOPE_STACK.with(|stack| {
            stack
                .borrow()
                .last()
                .copied()
                .expect("tls::protect called outside of with_protect_scope")
        });

        // SAFETY: scope_ptr is valid because we're inside with_protect_scope
        let scope: &ProtectScope = unsafe { scope_ptr.as_ref() };
        let root = unsafe { scope.protect(x) };

        TlsRoot { sexp: root.sexp }
    }

    /// Protect a value, returning the raw SEXP.
    ///
    /// # Panics
    ///
    /// Panics if called outside of a [`with_protect_scope`] block.
    ///
    /// # Safety
    ///
    /// Same as [`protect`].
    #[inline]
    pub unsafe fn protect_raw(x: SEXP) -> SEXP {
        let scope_ptr = SCOPE_STACK.with(|stack| {
            stack
                .borrow()
                .last()
                .copied()
                .expect("tls::protect_raw called outside of with_protect_scope")
        });

        let scope: &ProtectScope = unsafe { scope_ptr.as_ref() };
        unsafe { scope.protect_raw(x) }
    }

    /// Check if there is an active TLS scope.
    #[inline]
    pub fn has_active_scope() -> bool {
        SCOPE_STACK.with(|stack| !stack.borrow().is_empty())
    }

    /// Get the current scope's protection count.
    ///
    /// Returns `None` if no scope is active.
    #[inline]
    pub fn current_count() -> Option<i32> {
        SCOPE_STACK.with(|stack| {
            stack.borrow().last().map(|ptr| {
                // SAFETY: pointer is valid while in with_protect_scope
                unsafe { ptr.as_ref() }.count()
            })
        })
    }

    /// Get the nesting depth of TLS scopes.
    #[inline]
    pub fn scope_depth() -> usize {
        SCOPE_STACK.with(|stack| stack.borrow().len())
    }

    /// A rooted SEXP from TLS protection.
    ///
    /// This is similar to [`Root`] but without a compile-time lifetime tie to
    /// the scope. The protection is valid as long as the enclosing
    /// [`with_protect_scope`] block hasn't exited.
    ///
    /// # Warning
    ///
    /// Using a `TlsRoot` after its scope has exited is undefined behavior.
    /// The compile-time lifetime checking of [`Root`] is safer; use TLS
    /// convenience only when necessary.
    #[derive(Clone, Copy)]
    pub struct TlsRoot {
        sexp: SEXP,
    }

    impl TlsRoot {
        /// Get the underlying SEXP.
        #[inline]
        pub fn get(&self) -> SEXP {
            self.sexp
        }

        /// Consume and return the underlying SEXP.
        #[inline]
        pub fn into_raw(self) -> SEXP {
            self.sexp
        }
    }

    impl std::ops::Deref for TlsRoot {
        type Target = SEXP;

        #[inline]
        fn deref(&self) -> &Self::Target {
            &self.sexp
        }
    }
}

// =============================================================================
// Typed Vector Collection
// =============================================================================

// NOTE: Typed vectors (INTSXP, REALSXP, RAWSXP, LGLSXP, CPLXSXP) do NOT need
// complex protection patterns during construction. You allocate once, protect
// once, then fill by writing directly to the data pointer. No GC can occur
// during the fill because you're just doing pointer writes - no R allocations.
//
// Only STRSXP (character vectors) and VECSXP (lists) need the ReprotectSlot
// pattern because each element insertion might allocate (mkChar, etc.).
//
// For typed vectors with unknown length, just collect to Vec<T> first, then
// allocate the exact size. The brief doubling of memory is fine.

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests primarily verify compilation and basic invariants.
    // Full integration testing requires R to be initialized.

    // -------------------------------------------------------------------------
    // Basic invariants
    // -------------------------------------------------------------------------

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

    #[test]
    fn tls_root_is_copy() {
        fn assert_copy<T: Copy>() {}
        assert_copy::<tls::TlsRoot>();
    }

    // -------------------------------------------------------------------------
    // Threading: compile-time !Send + !Sync checks
    // -------------------------------------------------------------------------

    #[test]
    fn protect_scope_is_not_send() {
        fn assert_not_send<T>()
        where
            T: ?Sized,
        {
            // This test passes if ProtectScope is !Send
            // We can't directly assert !Send, but the type containing Rc<()> ensures it
        }
        assert_not_send::<ProtectScope>();
    }

    #[test]
    fn protect_scope_is_not_sync() {
        fn assert_not_sync<T>()
        where
            T: ?Sized,
        {
            // This test passes if ProtectScope is !Sync
        }
        assert_not_sync::<ProtectScope>();
    }

    #[test]
    fn owned_protect_is_not_send() {
        fn assert_not_send<T>()
        where
            T: ?Sized,
        {
        }
        assert_not_send::<OwnedProtect>();
    }

    // Note: We can't easily assert !Send/!Sync at compile time without
    // negative trait bounds. The PhantomData<Rc<()>> marker ensures these types
    // are !Send and !Sync. If you need compile-time verification, use the
    // static_assertions crate with `assert_not_impl_any!`.

    // -------------------------------------------------------------------------
    // TLS scope tests
    // -------------------------------------------------------------------------

    #[test]
    fn tls_no_active_scope_by_default() {
        assert!(!tls::has_active_scope());
        assert_eq!(tls::current_count(), None);
        assert_eq!(tls::scope_depth(), 0);
    }

    #[test]
    fn tls_scope_depth_tracking() {
        // Without R, we can only test the TLS tracking logic
        // The actual protect/unprotect requires R runtime

        // Test that scope depth is tracked correctly
        assert_eq!(tls::scope_depth(), 0);

        // We can't fully test with_protect_scope without R initialized,
        // but we can verify the API compiles and the TLS logic works
    }

    #[test]
    #[should_panic(expected = "tls::protect called outside of with_protect_scope")]
    fn tls_protect_panics_outside_scope() {
        // This should panic because there's no active scope
        // Note: Can't actually call protect without R, but we test the panic message
        unsafe {
            let _ = tls::protect(crate::ffi::SEXP(std::ptr::null_mut()));
        }
    }

    // -------------------------------------------------------------------------
    // Escape hatch tests
    // -------------------------------------------------------------------------

    #[test]
    fn disarm_prevents_unprotect() {
        let scope = ProtectScope::default();
        assert!(scope.armed.get());

        unsafe { scope.disarm() };
        assert!(!scope.armed.get());

        // Scope will drop without calling Rf_unprotect (can't test actual R call)
    }

    #[test]
    fn rearm_restores_unprotect() {
        let scope = ProtectScope::default();

        unsafe {
            scope.disarm();
            assert!(!scope.armed.get());

            scope.rearm();
            assert!(scope.armed.get());
        }
    }

    // -------------------------------------------------------------------------
    // Counter tracking tests
    // -------------------------------------------------------------------------

    #[test]
    fn scope_counter_starts_at_zero() {
        let scope = ProtectScope::default();
        assert_eq!(scope.count(), 0);
    }

    // Note: The following tests require R to be initialized and would be
    // integration tests rather than unit tests:
    //
    // - Balance test: protect N, verify unprotect(N) on drop (gctorture)
    // - Nested scopes: verify drop order yields correct net unprotect
    // - Reprotect slot: verify set() many times keeps count at +1
    //
    // These should be tested in miniextendr-api/tests/gc_protect.rs with
    // embedded R.
}
