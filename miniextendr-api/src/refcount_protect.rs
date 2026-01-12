//! Reference-counted GC protection using a BTreeMap + VECSXP backing.
//!
//! This module provides an alternative to [`gc_protect`](crate::gc_protect) that uses
//! reference counting instead of R's LIFO protect stack. This allows releasing
//! protections in any order and avoids the `--max-ppsize` limit.
//!
//! # How It Works
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │  RefCountedArena                                                     │
//! │  ┌─────────────────────────┐   ┌───────────────────────────────────┐│
//! │  │  BTreeMap<SEXP, Entry>  │   │  VECSXP (R_PreserveObject'd)      ││
//! │  │  ────────────────────── │   │  ─────────────────────────────    ││
//! │  │  sexp_a → {count:2, i:0}│◄──┤  [0]: sexp_a                      ││
//! │  │  sexp_b → {count:1, i:1}│◄──┤  [1]: sexp_b                      ││
//! │  │  sexp_c → {count:1, i:2}│◄──┤  [2]: sexp_c                      ││
//! │  └─────────────────────────┘   │  [3]: <free>                      ││
//! │                                │  [4]: <free>                      ││
//! │                                └───────────────────────────────────┘│
//! └─────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! - The `BTreeMap` tracks each SEXP's reference count and index in the VECSXP
//! - The `VECSXP` is preserved via `R_PreserveObject` so it's never GC'd
//! - When ref count drops to 0, the entry is removed and the slot freed
//! - Free slots are reused (swap-with-last for O(1) removal)
//!
//! # When to Use This
//!
//! Use `RefCountedArena` when:
//! - You need to protect many objects without worrying about stack limits
//! - You want to release protections in arbitrary order
//! - You're building data structures where the same SEXP may be referenced multiple times
//!
//! Use [`ProtectScope`](crate::gc_protect::ProtectScope) instead when:
//! - Protection is short-lived (within a single function)
//! - You want minimal overhead (direct R API calls)
//! - You're okay with LIFO release order
//!
//! # Example
//!
//! ```ignore
//! use miniextendr_api::refcount_protect::RefCountedArena;
//!
//! unsafe fn complex_operation() -> SEXP {
//!     let arena = RefCountedArena::new();
//!
//!     // Protect objects - can be released in any order
//!     let a = arena.protect(allocate_a());
//!     let b = arena.protect(allocate_b());
//!     let c = arena.protect(allocate_c());
//!
//!     // Same SEXP can be protected multiple times (ref count increases)
//!     arena.protect(a);  // a now has ref count 2
//!
//!     // Release in any order
//!     arena.unprotect(b);  // b removed (count was 1)
//!     arena.unprotect(a);  // a count now 1
//!     arena.unprotect(a);  // a removed (count was 1)
//!
//!     c  // c still protected until arena drops
//! }
//! ```
//!
//! # Performance Characteristics
//!
//! | Operation | Time Complexity | Notes |
//! |-----------|-----------------|-------|
//! | `protect` | O(log n) | BTreeMap insert/update |
//! | `unprotect` | O(log n) | BTreeMap lookup + possible removal |
//! | Memory | O(n) | BTreeMap entries + VECSXP slots |
//!
//! The overhead per operation is higher than raw `Rf_protect`/`Rf_unprotect`,
//! but the system is more flexible and doesn't have stack size limits.

use crate::ffi::{
    R_NilValue, R_PreserveObject, R_ReleaseObject, R_xlen_t, Rf_allocVector, Rf_protect,
    Rf_unprotect, SET_VECTOR_ELT, SEXP, SEXPTYPE, VECTOR_ELT,
};
use std::cell::{Cell, RefCell, UnsafeCell};
use std::collections::{BTreeMap, HashMap};
use std::marker::PhantomData;
use std::rc::Rc;

/// Entry in the reference count map.
#[derive(Debug, Clone, Copy)]
struct Entry {
    /// Reference count (how many times this SEXP has been protected)
    count: usize,
    /// Index in the backing VECSXP
    index: usize,
}

/// Enforces `!Send + !Sync` (R API is not thread-safe).
type NoSendSync = PhantomData<Rc<()>>;

/// A reference-counted arena for GC protection.
///
/// This provides an alternative to R's PROTECT stack that:
/// - Uses reference counting for each SEXP
/// - Allows releasing protections in any order
/// - Has no stack size limit (uses heap allocation)
///
/// The arena owns a preserved VECSXP that holds all protected SEXPs.
/// When the arena is dropped, all protections are released.
pub struct RefCountedArena {
    /// Map from SEXP pointer to entry (count + index)
    map: RefCell<BTreeMap<usize, Entry>>,
    /// Backing VECSXP (preserved via R_PreserveObject)
    backing: Cell<SEXP>,
    /// Current capacity of the backing VECSXP
    capacity: Cell<usize>,
    /// Number of active entries
    len: Cell<usize>,
    /// Free list: indices that can be reused
    free_list: RefCell<Vec<usize>>,
    /// Marker for !Send + !Sync
    _nosend: NoSendSync,
}

impl RefCountedArena {
    /// Initial capacity for the backing VECSXP.
    const INITIAL_CAPACITY: usize = 16;

    /// Create a new reference-counted arena.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    pub unsafe fn new() -> Self {
        unsafe { Self::with_capacity(Self::INITIAL_CAPACITY) }
    }

    /// Create a new arena with a specific initial capacity.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    pub unsafe fn with_capacity(capacity: usize) -> Self {
        let capacity = capacity.max(1);

        unsafe {
            // Allocate and preserve the backing VECSXP
            let backing = Rf_protect(Rf_allocVector(SEXPTYPE::VECSXP, capacity as R_xlen_t));
            R_PreserveObject(backing);
            Rf_unprotect(1);

            Self {
                map: RefCell::new(BTreeMap::new()),
                backing: Cell::new(backing),
                capacity: Cell::new(capacity),
                len: Cell::new(0),
                free_list: RefCell::new(Vec::new()),
                _nosend: PhantomData,
            }
        }
    }

    /// Protect a SEXP, incrementing its reference count.
    ///
    /// If the SEXP is already protected, its reference count is incremented.
    /// Otherwise, it's added to the arena with count 1.
    ///
    /// Returns the SEXP unchanged (for chaining convenience).
    ///
    /// # Safety
    ///
    /// - Must be called from the R main thread
    /// - `x` must be a valid SEXP
    #[inline]
    pub unsafe fn protect(&self, x: SEXP) -> SEXP {
        unsafe {
            // R_NilValue doesn't need protection
            if std::ptr::eq(x.0, R_NilValue.0) {
                return x;
            }

            let key = x.0 as usize;
            let mut map = self.map.borrow_mut();

            if let Some(entry) = map.get_mut(&key) {
                // Already protected - increment count
                entry.count += 1;
            } else {
                // New entry - find a slot
                let index = self.allocate_slot();

                // Store in backing VECSXP
                SET_VECTOR_ELT(self.backing.get(), index as R_xlen_t, x);

                // Add to map
                map.insert(key, Entry { count: 1, index });
                self.len.set(self.len.get() + 1);
            }

            x
        }
    }

    /// Unprotect a SEXP, decrementing its reference count.
    ///
    /// If the reference count reaches 0, the SEXP is removed from the arena.
    ///
    /// # Safety
    ///
    /// - Must be called from the R main thread
    /// - `x` must have been previously protected by this arena
    ///
    /// # Panics
    ///
    /// Panics if `x` was not protected by this arena.
    #[inline]
    pub unsafe fn unprotect(&self, x: SEXP) {
        unsafe {
            // R_NilValue doesn't need unprotection
            if std::ptr::eq(x.0, R_NilValue.0) {
                return;
            }

            let key = x.0 as usize;
            let mut map = self.map.borrow_mut();

            let entry = map
                .get_mut(&key)
                .expect("unprotect called on SEXP not protected by this arena");

            entry.count -= 1;

            if entry.count == 0 {
                let index = entry.index;

                // Clear the slot in backing VECSXP
                SET_VECTOR_ELT(self.backing.get(), index as R_xlen_t, R_NilValue);

                // Add index to free list
                self.free_list.borrow_mut().push(index);

                // Remove from map
                map.remove(&key);
                self.len.set(self.len.get() - 1);
            }
        }
    }

    /// Try to unprotect a SEXP, returning `true` if it was protected.
    ///
    /// Unlike [`unprotect`](Self::unprotect), this does not panic if the SEXP
    /// was not protected.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn try_unprotect(&self, x: SEXP) -> bool {
        unsafe {
            if std::ptr::eq(x.0, R_NilValue.0) {
                return false;
            }

            let key = x.0 as usize;
            let mut map = self.map.borrow_mut();

            if let Some(entry) = map.get_mut(&key) {
                entry.count -= 1;

                if entry.count == 0 {
                    let index = entry.index;

                    SET_VECTOR_ELT(self.backing.get(), index as R_xlen_t, R_NilValue);

                    self.free_list.borrow_mut().push(index);
                    map.remove(&key);
                    self.len.set(self.len.get() - 1);
                }

                true
            } else {
                false
            }
        }
    }

    /// Check if a SEXP is currently protected by this arena.
    #[inline]
    pub fn is_protected(&self, x: SEXP) -> bool {
        // SAFETY: R_NilValue is always valid
        if std::ptr::eq(x.0, unsafe { R_NilValue.0 }) {
            return false;
        }

        let key = x.0 as usize;
        self.map.borrow().contains_key(&key)
    }

    /// Get the reference count for a SEXP (0 if not protected).
    #[inline]
    pub fn ref_count(&self, x: SEXP) -> usize {
        // SAFETY: R_NilValue is always valid
        if std::ptr::eq(x.0, unsafe { R_NilValue.0 }) {
            return 0;
        }

        let key = x.0 as usize;
        self.map
            .borrow()
            .get(&key)
            .map(|e| e.count)
            .unwrap_or(0)
    }

    /// Get the number of distinct SEXPs currently protected.
    #[inline]
    pub fn len(&self) -> usize {
        self.len.get()
    }

    /// Check if the arena is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len.get() == 0
    }

    /// Get the current capacity of the backing storage.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity.get()
    }

    /// Allocate a slot in the backing VECSXP.
    ///
    /// Reuses free slots if available, otherwise grows the backing if needed.
    fn allocate_slot(&self) -> usize {
        // Try to reuse a free slot
        if let Some(index) = self.free_list.borrow_mut().pop() {
            return index;
        }

        // Need a new slot - check if we need to grow
        let len = self.len.get();
        let capacity = self.capacity.get();

        if len >= capacity {
            unsafe {
                self.grow();
            }
        }

        // Return the next slot
        len
    }

    /// Grow the backing VECSXP (doubles capacity).
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    unsafe fn grow(&self) {
        let old_capacity = self.capacity.get();
        let new_capacity = old_capacity * 2;
        let old_backing = self.backing.get();

        unsafe {
            // Allocate new backing
            let new_backing =
                Rf_protect(Rf_allocVector(SEXPTYPE::VECSXP, new_capacity as R_xlen_t));
            R_PreserveObject(new_backing);

            // Copy entries from old to new
            for i in 0..old_capacity {
                let elt = VECTOR_ELT(old_backing, i as R_xlen_t);
                SET_VECTOR_ELT(new_backing, i as R_xlen_t, elt);
            }

            // Release old backing
            R_ReleaseObject(old_backing);

            Rf_unprotect(1);

            // Update state
            self.backing.set(new_backing);
            self.capacity.set(new_capacity);
        }
    }

    /// Clear all protections.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    pub unsafe fn clear(&self) {
        let mut map = self.map.borrow_mut();
        let backing = self.backing.get();

        // Clear all slots
        for entry in map.values() {
            unsafe {
                SET_VECTOR_ELT(backing, entry.index as R_xlen_t, R_NilValue);
            }
        }

        map.clear();
        self.free_list.borrow_mut().clear();
        self.len.set(0);
    }
}

impl Drop for RefCountedArena {
    fn drop(&mut self) {
        // Release the preserved backing VECSXP
        unsafe {
            R_ReleaseObject(self.backing.get());
        }
    }
}

impl Default for RefCountedArena {
    fn default() -> Self {
        // SAFETY: This is a foot-gun but matches the pattern of other R interop code.
        unsafe { Self::new() }
    }
}

// =============================================================================
// RAII Guard
// =============================================================================

/// An RAII guard that unprotects a SEXP when dropped.
///
/// This provides automatic cleanup for arena-protected SEXPs.
///
/// # Example
///
/// ```ignore
/// unsafe {
///     let arena = RefCountedArena::new();
///     let guard = arena.guard(some_sexp);
///     // use guard.get()...
/// } // automatically unprotected when guard drops
/// ```
pub struct RefCountedGuard<'a> {
    arena: &'a RefCountedArena,
    sexp: SEXP,
}

impl<'a> RefCountedGuard<'a> {
    /// Create a new guard that will unprotect the SEXP when dropped.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn new(arena: &'a RefCountedArena, sexp: SEXP) -> Self {
        unsafe {
            arena.protect(sexp);
        }
        Self { arena, sexp }
    }

    /// Get the protected SEXP.
    #[inline]
    pub fn get(&self) -> SEXP {
        self.sexp
    }
}

impl Drop for RefCountedGuard<'_> {
    fn drop(&mut self) {
        unsafe {
            self.arena.unprotect(self.sexp);
        }
    }
}

impl std::ops::Deref for RefCountedGuard<'_> {
    type Target = SEXP;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.sexp
    }
}

// =============================================================================
// Arena extension methods
// =============================================================================

impl RefCountedArena {
    /// Protect a SEXP and return an RAII guard.
    ///
    /// The SEXP is automatically unprotected when the guard is dropped.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn guard(&self, x: SEXP) -> RefCountedGuard<'_> {
        unsafe { RefCountedGuard::new(self, x) }
    }
}

// =============================================================================
// HashMap-based variant (for benchmarking comparison)
// =============================================================================

/// A HashMap-based variant of [`RefCountedArena`] for benchmarking.
///
/// This is identical to `RefCountedArena` except it uses `HashMap` instead of
/// `BTreeMap` for the internal map. Use this to compare performance
/// characteristics of the two data structures.
pub struct HashMapArena {
    map: RefCell<HashMap<usize, Entry>>,
    backing: Cell<SEXP>,
    capacity: Cell<usize>,
    len: Cell<usize>,
    free_list: RefCell<Vec<usize>>,
    _nosend: NoSendSync,
}

impl HashMapArena {
    const INITIAL_CAPACITY: usize = 16;

    /// Create a new HashMap-based arena.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    pub unsafe fn new() -> Self {
        unsafe { Self::with_capacity(Self::INITIAL_CAPACITY) }
    }

    /// Create a new arena with a specific initial capacity.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    pub unsafe fn with_capacity(capacity: usize) -> Self {
        let capacity = capacity.max(1);

        unsafe {
            let backing = Rf_protect(Rf_allocVector(SEXPTYPE::VECSXP, capacity as R_xlen_t));
            R_PreserveObject(backing);
            Rf_unprotect(1);

            Self {
                map: RefCell::new(HashMap::new()),
                backing: Cell::new(backing),
                capacity: Cell::new(capacity),
                len: Cell::new(0),
                free_list: RefCell::new(Vec::new()),
                _nosend: PhantomData,
            }
        }
    }

    /// Protect a SEXP, incrementing its reference count.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn protect(&self, x: SEXP) -> SEXP {
        unsafe {
            if std::ptr::eq(x.0, R_NilValue.0) {
                return x;
            }

            let key = x.0 as usize;
            let mut map = self.map.borrow_mut();

            if let Some(entry) = map.get_mut(&key) {
                entry.count += 1;
            } else {
                let index = self.allocate_slot();
                SET_VECTOR_ELT(self.backing.get(), index as R_xlen_t, x);
                map.insert(key, Entry { count: 1, index });
                self.len.set(self.len.get() + 1);
            }

            x
        }
    }

    /// Unprotect a SEXP, decrementing its reference count.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn unprotect(&self, x: SEXP) {
        unsafe {
            if std::ptr::eq(x.0, R_NilValue.0) {
                return;
            }

            let key = x.0 as usize;
            let mut map = self.map.borrow_mut();

            let entry = map
                .get_mut(&key)
                .expect("unprotect called on SEXP not protected by this arena");

            entry.count -= 1;

            if entry.count == 0 {
                let index = entry.index;
                SET_VECTOR_ELT(self.backing.get(), index as R_xlen_t, R_NilValue);
                self.free_list.borrow_mut().push(index);
                map.remove(&key);
                self.len.set(self.len.get() - 1);
            }
        }
    }

    /// Check if a SEXP is currently protected.
    #[inline]
    pub fn is_protected(&self, x: SEXP) -> bool {
        if std::ptr::eq(x.0, unsafe { R_NilValue.0 }) {
            return false;
        }
        let key = x.0 as usize;
        self.map.borrow().contains_key(&key)
    }

    /// Get the reference count for a SEXP.
    #[inline]
    pub fn ref_count(&self, x: SEXP) -> usize {
        if std::ptr::eq(x.0, unsafe { R_NilValue.0 }) {
            return 0;
        }
        let key = x.0 as usize;
        self.map.borrow().get(&key).map(|e| e.count).unwrap_or(0)
    }

    /// Get the number of distinct SEXPs currently protected.
    #[inline]
    pub fn len(&self) -> usize {
        self.len.get()
    }

    /// Check if the arena is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len.get() == 0
    }

    fn allocate_slot(&self) -> usize {
        if let Some(index) = self.free_list.borrow_mut().pop() {
            return index;
        }

        let len = self.len.get();
        let capacity = self.capacity.get();

        if len >= capacity {
            unsafe { self.grow(); }
        }

        len
    }

    unsafe fn grow(&self) {
        let old_capacity = self.capacity.get();
        let new_capacity = old_capacity * 2;
        let old_backing = self.backing.get();

        unsafe {
            let new_backing =
                Rf_protect(Rf_allocVector(SEXPTYPE::VECSXP, new_capacity as R_xlen_t));
            R_PreserveObject(new_backing);

            for i in 0..old_capacity {
                let elt = VECTOR_ELT(old_backing, i as R_xlen_t);
                SET_VECTOR_ELT(new_backing, i as R_xlen_t, elt);
            }

            R_ReleaseObject(old_backing);
            Rf_unprotect(1);

            self.backing.set(new_backing);
            self.capacity.set(new_capacity);
        }
    }
}

impl Drop for HashMapArena {
    fn drop(&mut self) {
        unsafe {
            R_ReleaseObject(self.backing.get());
        }
    }
}

impl Default for HashMapArena {
    fn default() -> Self {
        unsafe { Self::new() }
    }
}

// =============================================================================
// Thread-local arena (no RefCell/Cell overhead)
// =============================================================================

/// Internal state for the thread-local arena.
struct ThreadLocalState {
    /// Map from SEXP pointer to entry (count + index)
    map: BTreeMap<usize, Entry>,
    /// Backing VECSXP (preserved via R_PreserveObject)
    backing: SEXP,
    /// Current capacity of the backing VECSXP
    capacity: usize,
    /// Number of active entries
    len: usize,
    /// Free list: indices that can be reused
    free_list: Vec<usize>,
    /// Whether the state has been initialized
    initialized: bool,
}

impl ThreadLocalState {
    const fn uninit() -> Self {
        Self {
            map: BTreeMap::new(),
            backing: SEXP(std::ptr::null_mut()),
            capacity: 0,
            len: 0,
            free_list: Vec::new(),
            initialized: false,
        }
    }
}

thread_local! {
    static THREAD_LOCAL_STATE: UnsafeCell<ThreadLocalState> = const { UnsafeCell::new(ThreadLocalState::uninit()) };
}

/// A thread-local reference-counted arena for GC protection.
///
/// This is a zero-sized type that provides access to thread-local state.
/// Unlike [`RefCountedArena`], this uses thread-local storage to eliminate
/// the overhead of `RefCell`/`Cell` interior mutability.
///
/// # Usage
///
/// ```ignore
/// use miniextendr_api::refcount_protect::ThreadLocalArena;
///
/// unsafe {
///     // Initialize once at the start (optional - auto-initializes on first use)
///     ThreadLocalArena::init();
///
///     let x = ThreadLocalArena::protect(some_sexp);
///     let y = ThreadLocalArena::protect(another_sexp);
///
///     // Release in any order
///     ThreadLocalArena::unprotect(x);
///     ThreadLocalArena::unprotect(y);
///
///     // Or clear all at once
///     ThreadLocalArena::clear();
/// }
/// ```
///
/// # Performance
///
/// This implementation eliminates `RefCell` borrow checking overhead by using
/// thread-local storage directly. Since R is single-threaded, we can safely
/// access the state without interior mutability checks.
pub struct ThreadLocalArena;

impl ThreadLocalArena {
    const INITIAL_CAPACITY: usize = 16;

    /// Initialize the thread-local arena.
    ///
    /// This is called automatically on first use, but can be called explicitly
    /// for deterministic initialization.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    pub unsafe fn init() {
        THREAD_LOCAL_STATE.with(|cell| {
            let state = unsafe { &mut *cell.get() };
            if !state.initialized {
                unsafe { Self::init_state(state) };
            }
        });
    }

    /// Initialize the state (internal).
    unsafe fn init_state(state: &mut ThreadLocalState) {
        let capacity = Self::INITIAL_CAPACITY;
        unsafe {
            let backing = Rf_protect(Rf_allocVector(SEXPTYPE::VECSXP, capacity as R_xlen_t));
            R_PreserveObject(backing);
            Rf_unprotect(1);

            state.backing = backing;
            state.capacity = capacity;
            state.len = 0;
            state.map.clear();
            state.free_list.clear();
            state.initialized = true;
        }
    }

    /// Protect a SEXP, incrementing its reference count.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn protect(x: SEXP) -> SEXP {
        THREAD_LOCAL_STATE.with(|cell| {
            let state = unsafe { &mut *cell.get() };
            if !state.initialized {
                unsafe { Self::init_state(state) };
            }
            unsafe { Self::protect_impl(state, x) }
        })
    }

    #[inline]
    unsafe fn protect_impl(state: &mut ThreadLocalState, x: SEXP) -> SEXP {
        unsafe {
            if std::ptr::eq(x.0, R_NilValue.0) {
                return x;
            }

            let key = x.0 as usize;

            if let Some(entry) = state.map.get_mut(&key) {
                entry.count += 1;
            } else {
                let index = Self::allocate_slot_impl(state);
                SET_VECTOR_ELT(state.backing, index as R_xlen_t, x);
                state.map.insert(key, Entry { count: 1, index });
                state.len += 1;
            }

            x
        }
    }

    /// Unprotect a SEXP, decrementing its reference count.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn unprotect(x: SEXP) {
        THREAD_LOCAL_STATE.with(|cell| {
            let state = unsafe { &mut *cell.get() };
            unsafe { Self::unprotect_impl(state, x) };
        });
    }

    #[inline]
    unsafe fn unprotect_impl(state: &mut ThreadLocalState, x: SEXP) {
        unsafe {
            if std::ptr::eq(x.0, R_NilValue.0) {
                return;
            }

            let key = x.0 as usize;

            let entry = state
                .map
                .get_mut(&key)
                .expect("unprotect called on SEXP not protected by this arena");

            entry.count -= 1;

            if entry.count == 0 {
                let index = entry.index;
                SET_VECTOR_ELT(state.backing, index as R_xlen_t, R_NilValue);
                state.free_list.push(index);
                state.map.remove(&key);
                state.len -= 1;
            }
        }
    }

    /// Try to unprotect a SEXP, returning `true` if it was protected.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn try_unprotect(x: SEXP) -> bool {
        THREAD_LOCAL_STATE.with(|cell| {
            let state = unsafe { &mut *cell.get() };
            unsafe { Self::try_unprotect_impl(state, x) }
        })
    }

    #[inline]
    unsafe fn try_unprotect_impl(state: &mut ThreadLocalState, x: SEXP) -> bool {
        unsafe {
            if std::ptr::eq(x.0, R_NilValue.0) {
                return false;
            }

            let key = x.0 as usize;

            if let Some(entry) = state.map.get_mut(&key) {
                entry.count -= 1;

                if entry.count == 0 {
                    let index = entry.index;
                    SET_VECTOR_ELT(state.backing, index as R_xlen_t, R_NilValue);
                    state.free_list.push(index);
                    state.map.remove(&key);
                    state.len -= 1;
                }

                true
            } else {
                false
            }
        }
    }

    /// Check if a SEXP is currently protected.
    #[inline]
    pub fn is_protected(x: SEXP) -> bool {
        if std::ptr::eq(x.0, unsafe { R_NilValue.0 }) {
            return false;
        }

        THREAD_LOCAL_STATE.with(|cell| {
            let state = unsafe { &*cell.get() };
            let key = x.0 as usize;
            state.map.contains_key(&key)
        })
    }

    /// Get the reference count for a SEXP (0 if not protected).
    #[inline]
    pub fn ref_count(x: SEXP) -> usize {
        if std::ptr::eq(x.0, unsafe { R_NilValue.0 }) {
            return 0;
        }

        THREAD_LOCAL_STATE.with(|cell| {
            let state = unsafe { &*cell.get() };
            let key = x.0 as usize;
            state.map.get(&key).map(|e| e.count).unwrap_or(0)
        })
    }

    /// Get the number of distinct SEXPs currently protected.
    #[inline]
    pub fn len() -> usize {
        THREAD_LOCAL_STATE.with(|cell| {
            let state = unsafe { &*cell.get() };
            state.len
        })
    }

    /// Check if the arena is empty.
    #[inline]
    pub fn is_empty() -> bool {
        Self::len() == 0
    }

    /// Get the current capacity.
    #[inline]
    pub fn capacity() -> usize {
        THREAD_LOCAL_STATE.with(|cell| {
            let state = unsafe { &*cell.get() };
            state.capacity
        })
    }

    fn allocate_slot_impl(state: &mut ThreadLocalState) -> usize {
        if let Some(index) = state.free_list.pop() {
            return index;
        }

        if state.len >= state.capacity {
            unsafe { Self::grow_impl(state) };
        }

        state.len
    }

    unsafe fn grow_impl(state: &mut ThreadLocalState) {
        let old_capacity = state.capacity;
        let new_capacity = old_capacity * 2;
        let old_backing = state.backing;

        unsafe {
            let new_backing =
                Rf_protect(Rf_allocVector(SEXPTYPE::VECSXP, new_capacity as R_xlen_t));
            R_PreserveObject(new_backing);

            for i in 0..old_capacity {
                let elt = VECTOR_ELT(old_backing, i as R_xlen_t);
                SET_VECTOR_ELT(new_backing, i as R_xlen_t, elt);
            }

            R_ReleaseObject(old_backing);
            Rf_unprotect(1);

            state.backing = new_backing;
            state.capacity = new_capacity;
        }
    }

    /// Clear all protections.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    pub unsafe fn clear() {
        THREAD_LOCAL_STATE.with(|cell| {
            let state = unsafe { &mut *cell.get() };
            if !state.initialized {
                return;
            }

            unsafe {
                for entry in state.map.values() {
                    SET_VECTOR_ELT(state.backing, entry.index as R_xlen_t, R_NilValue);
                }
            }

            state.map.clear();
            state.free_list.clear();
            state.len = 0;
        });
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arena_starts_empty() {
        let arena = RefCountedArena::default();
        assert!(arena.is_empty());
        assert_eq!(arena.len(), 0);
    }

    #[test]
    fn arena_has_initial_capacity() {
        let arena = RefCountedArena::default();
        assert_eq!(arena.capacity(), RefCountedArena::INITIAL_CAPACITY);
    }

    #[test]
    fn nil_is_not_protected() {
        let arena = RefCountedArena::default();
        // SAFETY: R_NilValue is always valid
        let nil = unsafe { R_NilValue };
        assert!(!arena.is_protected(nil));
        assert_eq!(arena.ref_count(nil), 0);
    }

    // Integration tests with actual SEXPs require R to be initialized
    // and should be in miniextendr-api/tests/refcount_protect.rs
}
