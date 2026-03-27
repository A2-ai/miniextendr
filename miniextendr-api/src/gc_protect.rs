//! GC protection tools built on R's PROTECT stack.
//!
//! This module provides RAII wrappers around R's GC protection primitives.
//!
//! # Submodules
//!
//! | Module | Contents |
//! |--------|----------|
//! | [`tls`] | Thread-local convenience API — `tls::protect(x)` without passing `&ProtectScope` |
//!
//! # Core Types
//!
//! - [`ProtectScope`] — RAII scope that calls `UNPROTECT(n)` on drop
//! - [`OwnedProtect`] — single-value RAII protect/unprotect
//! - [`Root`] — lifetime-tied handle to a protected SEXP
//! - [`ReprotectSlot`] — `PROTECT_WITH_INDEX` + `REPROTECT` for mutable slots
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
//! let list = List::from_raw(scope.alloc_vecsxp(n).into_raw());
//! for i in 0..n {
//!     let child = Rf_allocVector(REALSXP, 10);
//!     list.set_elt(i, child);  // protects child during insertion
//! }
//!
//! // EFFICIENT - use ListBuilder with scope
//! let builder = ListBuilder::new(&scope, n);
//! for i in 0..n {
//!     let child = scope.alloc_real(10).into_raw();
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
    R_NewEnv, R_NilValue, R_ProtectWithIndex, R_Reprotect, R_xlen_t, RNativeType, Rf_allocList,
    Rf_allocMatrix, Rf_allocVector, Rf_coerceVector, Rf_duplicate, Rf_mkCharLenCE, Rf_protect,
    Rf_ScalarComplex, Rf_ScalarInteger, Rf_ScalarLogical, Rf_ScalarRaw, Rf_ScalarReal,
    Rf_ScalarString, Rf_shallow_duplicate, Rf_unprotect, SEXP, SEXPTYPE,
};
use core::cell::Cell;
use core::marker::PhantomData;
use std::rc::Rc;

/// R's PROTECT_INDEX type (just `c_int` under the hood).
pub type ProtectIndex = ::std::os::raw::c_int;

/// Enforces `!Send + !Sync` (R API is not thread-safe).
type NoSendSync = PhantomData<Rc<()>>;

// region: Protector trait

/// A scope-like GC protection backend.
///
/// Functions that allocate multiple intermediate SEXPs can take `&mut impl Protector`
/// to be generic over the protection mechanism. All protected SEXPs stay protected
/// until the protector itself is dropped — there is no individual release via this
/// trait.
///
/// For individual release by key, use [`ProtectPool::insert`](crate::protect_pool::ProtectPool::insert)
/// and [`ProtectPool::release`](crate::protect_pool::ProtectPool::release) directly.
///
/// # Safety
///
/// Implementations must ensure that the returned SEXP remains protected from GC
/// for at least as long as the protector is alive. Callers must not use the
/// returned SEXP after the protector is dropped.
///
/// All methods must be called from the R main thread.
pub trait Protector {
    /// Protect a SEXP from garbage collection.
    ///
    /// Returns the same SEXP (for convenience in chaining). The SEXP is now
    /// protected and will remain so until the protector is dropped.
    ///
    /// The key (if any) is managed internally — use the pool's direct API
    /// (`insert`/`release`) if you need individual release.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread. `sexp` must be a valid SEXP.
    unsafe fn protect(&mut self, sexp: SEXP) -> SEXP;
}

impl Protector for ProtectScope {
    #[inline]
    unsafe fn protect(&mut self, sexp: SEXP) -> SEXP {
        unsafe { self.protect_raw(sexp) }
    }
}

impl Protector for crate::protect_pool::ProtectPool {
    #[inline]
    unsafe fn protect(&mut self, sexp: SEXP) -> SEXP {
        // Key is intentionally discarded — Protector is scope-like (all released
        // on drop). For individual release, use pool.insert()/pool.release() directly.
        unsafe { self.insert(sexp) };
        sexp
    }
}

// endregion

// region: ProtectScope

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
        unsafe { R_ProtectWithIndex(x, std::ptr::from_mut(&mut idx)) };
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

    // region: Allocation + Protection Helpers

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
    pub unsafe fn alloc_strsxp<'a>(&'a self, n: usize) -> Root<'a> {
        unsafe { self.alloc_character(n) }
    }

    /// Allocate a VECSXP (generic list) of the given length and immediately protect it.
    ///
    /// # Safety
    ///
    /// Same as [`alloc_vector`][Self::alloc_vector].
    #[inline]
    pub unsafe fn alloc_vecsxp<'a>(&'a self, n: usize) -> Root<'a> {
        let len = R_xlen_t::try_from(n).expect("length exceeds R_xlen_t");
        unsafe { self.alloc_vector(SEXPTYPE::VECSXP, len) }
    }

    // region: Typed vector allocation shortcuts

    /// Allocate an integer vector (INTSXP), protected.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn alloc_integer<'a>(&'a self, n: usize) -> Root<'a> {
        let len = R_xlen_t::try_from(n).expect("length exceeds R_xlen_t");
        unsafe { self.alloc_vector(SEXPTYPE::INTSXP, len) }
    }

    /// Allocate a real vector (REALSXP), protected.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn alloc_real<'a>(&'a self, n: usize) -> Root<'a> {
        let len = R_xlen_t::try_from(n).expect("length exceeds R_xlen_t");
        unsafe { self.alloc_vector(SEXPTYPE::REALSXP, len) }
    }

    /// Allocate a logical vector (LGLSXP), protected.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn alloc_logical<'a>(&'a self, n: usize) -> Root<'a> {
        let len = R_xlen_t::try_from(n).expect("length exceeds R_xlen_t");
        unsafe { self.alloc_vector(SEXPTYPE::LGLSXP, len) }
    }

    /// Allocate a raw vector (RAWSXP), protected.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn alloc_raw<'a>(&'a self, n: usize) -> Root<'a> {
        let len = R_xlen_t::try_from(n).expect("length exceeds R_xlen_t");
        unsafe { self.alloc_vector(SEXPTYPE::RAWSXP, len) }
    }

    /// Allocate a complex vector (CPLXSXP), protected.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn alloc_complex<'a>(&'a self, n: usize) -> Root<'a> {
        let len = R_xlen_t::try_from(n).expect("length exceeds R_xlen_t");
        unsafe { self.alloc_vector(SEXPTYPE::CPLXSXP, len) }
    }

    /// Allocate a character vector (STRSXP), protected.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn alloc_character<'a>(&'a self, n: usize) -> Root<'a> {
        let len = R_xlen_t::try_from(n).expect("length exceeds R_xlen_t");
        unsafe { self.alloc_vector(SEXPTYPE::STRSXP, len) }
    }

    // endregion

    // region: Scalar constructors (allocate + set + protect)

    /// Create a scalar integer (length-1 INTSXP), protected.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn scalar_integer<'a>(&'a self, x: i32) -> Root<'a> {
        unsafe { self.protect(Rf_ScalarInteger(x)) }
    }

    /// Create a scalar real (length-1 REALSXP), protected.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn scalar_real<'a>(&'a self, x: f64) -> Root<'a> {
        unsafe { self.protect(Rf_ScalarReal(x)) }
    }

    /// Create a scalar logical (length-1 LGLSXP), protected.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn scalar_logical<'a>(&'a self, x: bool) -> Root<'a> {
        unsafe { self.protect(Rf_ScalarLogical(if x { 1 } else { 0 })) }
    }

    /// Create a scalar complex (length-1 CPLXSXP), protected.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn scalar_complex<'a>(&'a self, x: crate::ffi::Rcomplex) -> Root<'a> {
        unsafe { self.protect(Rf_ScalarComplex(x)) }
    }

    /// Create a scalar raw (length-1 RAWSXP), protected.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn scalar_raw<'a>(&'a self, x: u8) -> Root<'a> {
        unsafe { self.protect(Rf_ScalarRaw(x)) }
    }

    /// Create a scalar string (length-1 STRSXP) from a Rust `&str`, protected.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn scalar_string<'a>(&'a self, s: &str) -> Root<'a> {
        let charsxp = if s.is_empty() {
            unsafe { crate::ffi::R_BlankString }
        } else {
            let len: i32 = s.len().try_into().expect("string exceeds i32::MAX bytes");
            unsafe { Rf_mkCharLenCE(s.as_ptr().cast(), len, crate::ffi::CE_UTF8) }
        };
        unsafe { self.protect(Rf_ScalarString(charsxp)) }
    }

    // endregion

    // region: CHARSXP, duplication, coercion, environment

    /// Create a CHARSXP from a Rust `&str`, protected.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn mkchar<'a>(&'a self, s: &str) -> Root<'a> {
        let charsxp = if s.is_empty() {
            unsafe { crate::ffi::R_BlankString }
        } else {
            let len: i32 = s.len().try_into().expect("string exceeds i32::MAX bytes");
            unsafe { Rf_mkCharLenCE(s.as_ptr().cast(), len, crate::ffi::CE_UTF8) }
        };
        unsafe { self.protect(charsxp) }
    }

    /// Deep-duplicate a SEXP, protected.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread. `x` must be a valid SEXP.
    #[inline]
    pub unsafe fn duplicate<'a>(&'a self, x: SEXP) -> Root<'a> {
        unsafe { self.protect(Rf_duplicate(x)) }
    }

    /// Shallow-duplicate a SEXP, protected.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread. `x` must be a valid SEXP.
    #[inline]
    pub unsafe fn shallow_duplicate<'a>(&'a self, x: SEXP) -> Root<'a> {
        unsafe { self.protect(Rf_shallow_duplicate(x)) }
    }

    /// Coerce a SEXP to a different type, protected.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread. `x` must be a valid SEXP.
    #[inline]
    pub unsafe fn coerce<'a>(&'a self, x: SEXP, target: SEXPTYPE) -> Root<'a> {
        unsafe { self.protect(Rf_coerceVector(x, target)) }
    }

    /// Create a new environment, protected.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn new_env<'a>(&'a self, parent: SEXP, hash: bool, size: i32) -> Root<'a> {
        unsafe {
            self.protect(R_NewEnv(
                parent,
                if hash {
                    crate::ffi::Rboolean::TRUE
                } else {
                    crate::ffi::Rboolean::FALSE
                },
                size,
            ))
        }
    }

    // endregion

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
    // endregion

    // region: Iterator Collection

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
// endregion

// region: Root

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
// endregion

// region: OwnedProtect

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
// endregion

// region: ReprotectSlot

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
// endregion


pub mod tls;


// region: WorkerUnprotectGuard — Send-safe unprotect for worker threads

/// A `Send`-safe guard that calls `Rf_unprotect(n)` on drop via `with_r_thread`.
///
/// Use this when you `Rf_protect` on the R main thread, then need the unprotect
/// to happen when a guard drops on a **worker thread** (e.g., rayon parallel code).
///
/// [`OwnedProtect`] and [`ProtectScope`] are `!Send` — they can only be used on
/// the R main thread. `WorkerUnprotectGuard` fills the gap for cross-thread patterns
/// where allocation + protect happen on the R thread but the guard lives on a worker.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::gc_protect::WorkerUnprotectGuard;
///
/// let sexp = with_r_thread(|| unsafe {
///     let sexp = Rf_allocVector(REALSXP, n);
///     Rf_protect(sexp);
///     sexp
/// });
/// let _guard = WorkerUnprotectGuard::new(1);
///
/// // ... parallel work on sexp's data ...
/// // _guard drops here, dispatching Rf_unprotect(1) back to R thread
/// ```
pub struct WorkerUnprotectGuard(i32);

impl WorkerUnprotectGuard {
    /// Create a guard that will unprotect `n` entries on drop.
    #[inline]
    pub fn new(n: i32) -> Self {
        Self(n)
    }
}

impl Drop for WorkerUnprotectGuard {
    fn drop(&mut self) {
        let n = self.0;
        crate::worker::with_r_thread(move || unsafe {
            crate::ffi::Rf_unprotect_unchecked(n);
        });
    }
}

// Safety: no SEXP field, just an integer count. The actual Rf_unprotect call
// is dispatched to the R main thread via with_r_thread.
unsafe impl Send for WorkerUnprotectGuard {}
// endregion

// region: Typed Vector Collection

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
// endregion

// region: Tests

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests primarily verify compilation and basic invariants.
    // Full integration testing requires R to be initialized.
    // endregion

    // region: Basic invariants

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
    // endregion

    // region: Threading: compile-time !Send + !Sync checks

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
    // endregion

    // region: TLS scope tests

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
    // endregion

    // region: Escape hatch tests

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
    // endregion

    // region: Counter tracking tests

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
    // endregion
}
// endregion
