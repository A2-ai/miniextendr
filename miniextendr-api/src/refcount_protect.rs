//! Reference-counted GC protection using a `BTreeMap` + VECSXP backing.
//!
//! This module provides an alternative to [`gc_protect`](crate::gc_protect) that uses
//! reference counting instead of R's LIFO protect stack. This allows releasing
//! protections in any order and avoids the `--max-ppsize` limit.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │  RefCountedArena / ThreadLocalArena                                 │
//! │  ┌─────────────────────────┐   ┌───────────────────────────────────┐│
//! │  │  BTreeMap<usize, Entry> │   │  VECSXP (R_PreserveObject'd)      ││
//! │  │  ────────────────────── │   │  ─────────────────────────────    ││
//! │  │  sexp_a → {count:2, i:0}│◄──┤  [0]: sexp_a                      ││
//! │  │  sexp_b → {count:1, i:1}│◄──┤  [1]: sexp_b                      ││
//! │  │  sexp_c → {count:1, i:2}│◄──┤  [2]: sexp_c                      ││
//! │  └─────────────────────────┘   │  [3]: <free>                      ││
//! │                                └───────────────────────────────────┘│
//! └─────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Available Types
//!
//! | Type | Storage | Use Case |
//! |------|---------|----------|
//! | [`RefCountedArena`] | `RefCell` | General purpose, ordered, ppsize-scale workloads |
//! | [`ThreadLocalArena`] | `thread_local` | Lowest overhead — no `RefCell` borrow checking |
//!
//! Only these two flavors are instantiated anywhere in the tree today. This
//! module previously also shipped HashMap- and ahash-backed variants
//! (`HashMapArena`, `ThreadLocalHashArena`, `FastHashMapArena`,
//! `ThreadLocalFastHashArena`) behind a generic `MapStorage` abstraction; they
//! were removed for having zero production or test consumers (see
//! <https://github.com/a2-ai/miniextendr/issues/ISSUE_NUMBER_PLACEHOLDER>).
//! Re-add a flavor (and the generic `MapStorage` plumbing needed to support
//! more than one map type again) when a real consumer appears.

use crate::sys::{R_PreserveObject, R_ReleaseObject, Rf_allocVector, Rf_protect, Rf_unprotect};
use crate::{R_xlen_t, SEXP, SEXPTYPE, SexpExt};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::rc::Rc;

// region: Entry type

/// Entry in the reference count map.
#[derive(Debug, Clone, Copy)]
struct Entry {
    /// Reference count (how many times this SEXP has been protected)
    count: usize,
    /// Index in the backing VECSXP
    index: usize,
}
// endregion

// region: Core arena state (shared between RefCell and thread_local variants)

/// Core arena state without interior mutability.
///
/// This is used internally by both [`RefCountedArena`] (with `RefCell`) and
/// [`ThreadLocalArena`] (with `UnsafeCell`).
struct ArenaState {
    /// Map from SEXP pointer to entry
    map: MaybeUninit<BTreeMap<usize, Entry>>,
    /// Backing VECSXP (preserved via R_PreserveObject)
    backing: SEXP,
    /// Current capacity
    capacity: usize,
    /// Number of active entries
    len: usize,
    /// Free list for slot reuse
    free_list: Vec<usize>,
    /// Monotonic write cursor: next index to hand out on the fresh-slot path.
    /// Distinct from `len` (live count) — after release cycles `len` can be
    /// lower than the highest index ever handed out, so using `len` as the
    /// cursor can return an index that is already on the free-list.
    next_slot: usize,
}

impl ArenaState {
    /// Initial capacity for the backing VECSXP.
    ///
    /// This is suitable for light usage (a handful of protected values).
    /// For ppsize-scale workloads (hundreds or thousands of protected values),
    /// use [`RefCountedArena::with_capacity`] or
    /// [`ThreadLocalArena::init_with_capacity`] to avoid repeated backing
    /// VECSXP growth and map rehashing.
    const INITIAL_CAPACITY: usize = 16;

    /// Maximum capacity: the backing VECSXP is indexed by `R_xlen_t` (isize),
    /// so the capacity must fit in a non-negative `R_xlen_t`.
    const MAX_CAPACITY: usize = R_xlen_t::MAX as usize;

    /// Convert a `usize` capacity to `R_xlen_t`, panicking on overflow.
    #[inline]
    fn capacity_as_r_xlen(cap: usize) -> R_xlen_t {
        R_xlen_t::try_from(cap).unwrap_or_else(|_| {
            panic!(
                "arena capacity {} exceeds R_xlen_t::MAX ({})",
                cap,
                R_xlen_t::MAX
            )
        })
    }

    /// Create uninitialized state (for thread_local).
    const fn uninit() -> Self {
        Self {
            map: MaybeUninit::uninit(),
            backing: SEXP(std::ptr::null_mut()),
            capacity: 0,
            len: 0,
            free_list: Vec::new(),
            next_slot: 0,
        }
    }

    /// Initialize the state.
    ///
    /// # Safety
    ///
    /// Must be called exactly once before using the state.
    unsafe fn init(&mut self, capacity: usize) {
        let capacity = capacity.max(1);
        assert!(
            capacity <= Self::MAX_CAPACITY,
            "arena capacity {} exceeds R_xlen_t::MAX ({})",
            capacity,
            R_xlen_t::MAX
        );
        unsafe {
            let r_cap = Self::capacity_as_r_xlen(capacity);
            let backing = Rf_protect(Rf_allocVector(SEXPTYPE::VECSXP, r_cap));
            R_PreserveObject(backing);
            Rf_unprotect(1);

            self.map.write(BTreeMap::new());
            self.backing = backing;
            self.capacity = capacity;
            self.len = 0;
            self.next_slot = 0;
            self.free_list = Vec::with_capacity(capacity);
        }
    }

    /// Create initialized state.
    unsafe fn new(capacity: usize) -> Self {
        let capacity = capacity.max(1);
        let mut state = Self {
            map: MaybeUninit::new(BTreeMap::new()),
            backing: SEXP(std::ptr::null_mut()),
            capacity: 0,
            len: 0,
            next_slot: 0,
            free_list: Vec::with_capacity(capacity),
        };
        unsafe { state.init_backing(capacity) };
        state
    }

    /// Initialize just the backing (map already initialized).
    unsafe fn init_backing(&mut self, capacity: usize) {
        let capacity = capacity.max(1);
        assert!(
            capacity <= Self::MAX_CAPACITY,
            "arena capacity {} exceeds R_xlen_t::MAX ({})",
            capacity,
            R_xlen_t::MAX
        );
        unsafe {
            let r_cap = Self::capacity_as_r_xlen(capacity);
            let backing = Rf_protect(Rf_allocVector(SEXPTYPE::VECSXP, r_cap));
            R_PreserveObject(backing);
            Rf_unprotect(1);

            self.backing = backing;
            self.capacity = capacity;
        }
    }

    /// Get a reference to the map.
    #[inline]
    fn map(&self) -> &BTreeMap<usize, Entry> {
        // SAFETY: Map is initialized before any access
        unsafe { self.map.assume_init_ref() }
    }

    /// Get a mutable reference to the map.
    #[inline]
    fn map_mut(&mut self) -> &mut BTreeMap<usize, Entry> {
        // SAFETY: Map is initialized before any access
        unsafe { self.map.assume_init_mut() }
    }

    /// Decrement the count for a key and remove if zero.
    ///
    /// Returns `Some((true, index))` if the entry was found and removed,
    /// `Some((false, index))` if the entry was found but count > 0 after
    /// decrement, `None` if the entry was not found.
    #[inline]
    fn decrement_and_maybe_remove(&mut self, key: &usize) -> Option<(bool, usize)> {
        let map = self.map_mut();
        if let Some(entry) = map.get_mut(key) {
            entry.count -= 1;
            if entry.count == 0 {
                let index = entry.index;
                map.remove(key);
                Some((true, index))
            } else {
                Some((false, entry.index))
            }
        } else {
            None
        }
    }

    /// Protect a SEXP from garbage collection.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread. The SEXP must be valid.
    #[inline]
    unsafe fn protect(&mut self, x: SEXP) -> SEXP {
        if x.is_nil() {
            return x;
        }

        let key = x.0 as usize;

        if let Some(entry) = self.map_mut().get_mut(&key) {
            entry.count += 1;
        } else {
            let index = self.allocate_slot();
            self.backing.set_vector_elt(index as R_xlen_t, x);
            self.map_mut().insert(key, Entry { count: 1, index });
            self.len += 1;
        }

        x
    }

    /// Unprotect a SEXP, allowing garbage collection when refcount reaches zero.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread. The SEXP must have been
    /// previously protected by this arena.
    #[inline]
    unsafe fn unprotect(&mut self, x: SEXP) {
        if x.is_nil() {
            return;
        }

        let key = x.0 as usize;

        match self.decrement_and_maybe_remove(&key) {
            Some((true, index)) => {
                // Entry was removed (count reached 0)
                self.backing.set_vector_elt(index as R_xlen_t, SEXP::nil());
                self.free_list.push(index);
                self.len -= 1;
            }
            Some((false, _)) => {
                // Entry still exists (count > 0)
            }
            None => {
                panic!("unprotect called on SEXP not protected by this arena");
            }
        }
    }

    /// Try to unprotect a SEXP. Returns false if not protected by this arena.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    unsafe fn try_unprotect(&mut self, x: SEXP) -> bool {
        if x.is_nil() {
            return false;
        }

        let key = x.0 as usize;

        match self.decrement_and_maybe_remove(&key) {
            Some((true, index)) => {
                // Entry was removed (count reached 0)
                self.backing.set_vector_elt(index as R_xlen_t, SEXP::nil());
                self.free_list.push(index);
                self.len -= 1;
                true
            }
            Some((false, _)) => {
                // Entry still exists (count > 0)
                true
            }
            None => false,
        }
    }

    #[inline]
    /// Returns true if this arena currently protects `x`.
    fn is_protected(&self, x: SEXP) -> bool {
        if x.is_nil() {
            return false;
        }
        let key = x.0 as usize;
        self.map().contains_key(&key)
    }

    #[inline]
    /// Returns the current reference count for `x` in this arena.
    ///
    /// Returns 0 if `x` is not protected or is `SEXP::nil()`.
    fn ref_count(&self, x: SEXP) -> usize {
        if x.is_nil() {
            return 0;
        }
        let key = x.0 as usize;
        self.map().get(&key).map(|e| e.count).unwrap_or(0)
    }

    fn allocate_slot(&mut self) -> usize {
        if let Some(index) = self.free_list.pop() {
            return index;
        }

        if self.next_slot >= self.capacity {
            unsafe { self.grow() };
        }

        let idx = self.next_slot;
        self.next_slot += 1;
        idx
    }

    unsafe fn grow(&mut self) {
        let old_capacity = self.capacity;
        let new_capacity = old_capacity
            .checked_mul(2)
            .expect("arena capacity overflow during growth");
        assert!(
            new_capacity <= Self::MAX_CAPACITY,
            "arena capacity {} would exceed R_xlen_t::MAX ({}) after growth",
            new_capacity,
            R_xlen_t::MAX
        );
        let old_backing = self.backing;

        unsafe {
            let r_new_cap = Self::capacity_as_r_xlen(new_capacity);
            let new_backing = Rf_protect(Rf_allocVector(SEXPTYPE::VECSXP, r_new_cap));
            R_PreserveObject(new_backing);

            for i in 0..old_capacity {
                let r_i = Self::capacity_as_r_xlen(i);
                let elt = old_backing.vector_elt(r_i);
                new_backing.set_vector_elt(r_i, elt);
            }

            R_ReleaseObject(old_backing);
            Rf_unprotect(1);

            self.backing = new_backing;
            self.capacity = new_capacity;
        }
    }

    /// Clear all protected values from the arena.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    unsafe fn clear(&mut self) {
        for entry in self.map().values() {
            self.backing
                .set_vector_elt(entry.index as R_xlen_t, SEXP::nil());
        }
        self.map_mut().clear();
        self.free_list.clear();
        self.len = 0;
        self.next_slot = 0;
    }

    unsafe fn release_backing(&mut self) {
        if !self.backing.0.is_null() {
            unsafe { R_ReleaseObject(self.backing) };
            self.backing = SEXP(std::ptr::null_mut());
        }
    }
}
// endregion

// region: RefCountedArena - RefCell-based arena

/// Enforces `!Send + !Sync` (R API is not thread-safe).
type NoSendSync = PhantomData<Rc<()>>;

/// A reference-counted arena for GC protection, backed by a `BTreeMap`.
///
/// This provides an alternative to R's PROTECT stack that:
/// - Uses reference counting for each SEXP
/// - Allows releasing protections in any order
/// - Has no stack size limit (uses heap allocation)
pub struct RefCountedArena {
    state: RefCell<ArenaState>,
    _nosend: NoSendSync,
}

impl RefCountedArena {
    /// Create a new arena with default capacity (16 slots).
    ///
    /// For workloads protecting many distinct SEXPs (e.g., ppsize-scale loops),
    /// prefer [`with_capacity`](Self::with_capacity) to avoid backing VECSXP
    /// growth and map rehashing during operation.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    pub unsafe fn new() -> Self {
        unsafe { Self::with_capacity(ArenaState::INITIAL_CAPACITY) }
    }

    /// Create a new arena with specific initial capacity.
    ///
    /// Pre-sizing the arena avoids growth of the backing VECSXP and rehashing
    /// of the internal map. Use this when the expected number of distinct
    /// protected values is known or can be estimated.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    pub unsafe fn with_capacity(capacity: usize) -> Self {
        Self {
            state: RefCell::new(unsafe { ArenaState::new(capacity) }),
            _nosend: PhantomData,
        }
    }

    /// Protect a SEXP, incrementing its reference count.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn protect(&self, x: SEXP) -> SEXP {
        unsafe { self.state.borrow_mut().protect(x) }
    }

    /// Unprotect a SEXP, decrementing its reference count.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    ///
    /// # Panics
    ///
    /// Panics if `x` was not protected by this arena.
    #[inline]
    pub unsafe fn unprotect(&self, x: SEXP) {
        unsafe { self.state.borrow_mut().unprotect(x) };
    }

    /// Try to unprotect a SEXP, returning `true` if it was protected.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn try_unprotect(&self, x: SEXP) -> bool {
        unsafe { self.state.borrow_mut().try_unprotect(x) }
    }

    /// Check if a SEXP is currently protected by this arena.
    #[inline]
    pub fn is_protected(&self, x: SEXP) -> bool {
        self.state.borrow().is_protected(x)
    }

    /// Get the reference count for a SEXP (0 if not protected).
    #[inline]
    pub fn ref_count(&self, x: SEXP) -> usize {
        self.state.borrow().ref_count(x)
    }

    /// Get the number of distinct SEXPs currently protected.
    #[inline]
    pub fn len(&self) -> usize {
        self.state.borrow().len
    }

    /// Check if the arena is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.state.borrow().len == 0
    }

    /// Get the current capacity.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.state.borrow().capacity
    }

    /// Clear all protections.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    pub unsafe fn clear(&self) {
        unsafe { self.state.borrow_mut().clear() };
    }

    /// Protect a SEXP and return an RAII guard.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn guard(&self, x: SEXP) -> ArenaGuard<'_> {
        unsafe { ArenaGuard::new(self, x) }
    }
}

impl Drop for RefCountedArena {
    fn drop(&mut self) {
        let state = self.state.get_mut();
        // SAFETY: RefCountedArena always constructs via ArenaState::new(),
        // which initializes the map.
        unsafe { state.map.assume_init_drop() };
        unsafe { state.release_backing() };
    }
}

impl Default for RefCountedArena {
    fn default() -> Self {
        unsafe { Self::new() }
    }
}
// endregion

// region: RAII Guard

/// An RAII guard that unprotects a SEXP when dropped.
pub struct ArenaGuard<'a> {
    arena: &'a RefCountedArena,
    sexp: SEXP,
}

impl<'a> ArenaGuard<'a> {
    /// Create a new guard that protects the SEXP and unprotects on drop.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread. The SEXP must be valid.
    #[inline]
    pub unsafe fn new(arena: &'a RefCountedArena, sexp: SEXP) -> Self {
        unsafe { arena.protect(sexp) };
        Self { arena, sexp }
    }

    #[inline]
    /// Returns the protected SEXP.
    pub fn get(&self) -> SEXP {
        self.sexp
    }
}

impl Drop for ArenaGuard<'_> {
    fn drop(&mut self) {
        unsafe { self.arena.unprotect(self.sexp) };
    }
}

impl std::ops::Deref for ArenaGuard<'_> {
    type Target = SEXP;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.sexp
    }
}
// endregion

// region: ThreadLocalArena

/// State wrapper for the thread-local arena.
struct ThreadLocalState {
    inner: ArenaState,
    initialized: bool,
}

impl ThreadLocalState {
    /// Create an uninitialized thread-local arena state.
    ///
    /// Call `init` or `init_with_capacity` before use.
    const fn uninit() -> Self {
        Self {
            inner: ArenaState::uninit(),
            initialized: false,
        }
    }

    /// Initialize with default capacity (16 slots).
    ///
    /// For ppsize-scale workloads, prefer [`init_with_capacity`](Self::init_with_capacity)
    /// to avoid backing VECSXP growth and map rehashing during operation.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread. Must only be called once.
    unsafe fn init(&mut self) {
        unsafe { self.inner.init(ArenaState::INITIAL_CAPACITY) };
        self.initialized = true;
    }

    /// Initialize with specific capacity.
    ///
    /// Pre-sizing avoids growth of the backing VECSXP and rehashing of the
    /// internal map. Use this when the expected number of distinct protected
    /// values is known or can be estimated (e.g., the length of an input vector).
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread. Must only be called once.
    unsafe fn init_with_capacity(&mut self, capacity: usize) {
        unsafe { self.inner.init(capacity) };
        self.initialized = true;
    }
}

impl Drop for ThreadLocalState {
    fn drop(&mut self) {
        if self.initialized {
            // SAFETY: The map was initialized in init() or init_with_capacity().
            // We must manually drop it because MaybeUninit does not run Drop.
            unsafe { self.inner.map.assume_init_drop() };
        }
        // R backing is released separately via release_backing() if needed.
        // Thread-local destructors may run after R has shut down, so we do NOT
        // call R_ReleaseObject here — the R runtime owns the backing VECSXP
        // lifetime via R_PreserveObject.
    }
}

thread_local! {
    static THREAD_LOCAL_STATE: std::cell::UnsafeCell<ThreadLocalState> =
        const { std::cell::UnsafeCell::new(ThreadLocalState::uninit()) };
}

/// Thread-local, `BTreeMap`-backed reference-counted GC protection arena.
///
/// This provides the lowest overhead for protection operations by
/// eliminating `RefCell` borrow checking — each thread gets its own
/// [`ThreadLocalState`] accessed through an `UnsafeCell`.
///
/// ```ignore
/// use miniextendr_api::refcount_protect::ThreadLocalArena;
/// unsafe { ThreadLocalArena::protect(x) };
/// ```
pub struct ThreadLocalArena;

impl ThreadLocalArena {
    /// Access the thread-local state.
    #[inline]
    fn with_state<R, F: FnOnce(&mut ThreadLocalState) -> R>(f: F) -> R {
        THREAD_LOCAL_STATE.with(|cell| f(unsafe { &mut *cell.get() }))
    }

    /// Initialize the arena with default capacity (called automatically on first use).
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    pub unsafe fn init() {
        Self::with_state(|s| {
            if !s.initialized {
                unsafe { s.init() };
            }
        });
    }

    /// Initialize the arena with specific capacity.
    ///
    /// Use this when you know the expected number of distinct protected values
    /// to avoid backing VECSXP growth and map rehashing during operation.
    ///
    /// If already initialized, this is a no-op.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    pub unsafe fn init_with_capacity(capacity: usize) {
        Self::with_state(|s| {
            if !s.initialized {
                unsafe { s.init_with_capacity(capacity) };
            }
        });
    }

    /// Protect a SEXP, incrementing its reference count.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn protect(x: SEXP) -> SEXP {
        Self::with_state(|s| {
            if !s.initialized {
                unsafe { s.init() };
            }
            unsafe { s.inner.protect(x) }
        })
    }

    /// Unprotect a SEXP.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn unprotect(x: SEXP) {
        Self::with_state(|s| {
            // If the arena was never initialized, no SEXP could have been
            // protected by it, so there is nothing to unprotect.
            if !s.initialized {
                return;
            }
            unsafe { s.inner.unprotect(x) };
        });
    }

    /// Try to unprotect a SEXP.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn try_unprotect(x: SEXP) -> bool {
        Self::with_state(|s| {
            // If the arena was never initialized, no SEXP could have been
            // protected by it, so return false.
            if !s.initialized {
                return false;
            }
            unsafe { s.inner.try_unprotect(x) }
        })
    }

    /// Protect without checking initialization.
    ///
    /// For hot loops where `init()` or `init_with_capacity()` has already been called.
    ///
    /// # Safety
    ///
    /// - Must be called from the R main thread.
    /// - The arena must have been initialized via `init()` or `init_with_capacity()`.
    #[inline]
    pub unsafe fn protect_fast(x: SEXP) -> SEXP {
        Self::with_state(|s| {
            debug_assert!(s.initialized, "protect_fast called before init");
            unsafe { s.inner.protect(x) }
        })
    }

    /// Unprotect without checking initialization.
    ///
    /// For hot loops where `init()` or `init_with_capacity()` has already been called.
    ///
    /// # Safety
    ///
    /// - Must be called from the R main thread.
    /// - The arena must have been initialized via `init()` or `init_with_capacity()`.
    #[inline]
    pub unsafe fn unprotect_fast(x: SEXP) {
        Self::with_state(|s| {
            debug_assert!(s.initialized, "unprotect_fast called before init");
            unsafe { s.inner.unprotect(x) };
        });
    }

    /// Try to unprotect without checking initialization.
    ///
    /// For hot loops where `init()` or `init_with_capacity()` has already been called.
    ///
    /// # Safety
    ///
    /// - Must be called from the R main thread.
    /// - The arena must have been initialized via `init()` or `init_with_capacity()`.
    #[inline]
    pub unsafe fn try_unprotect_fast(x: SEXP) -> bool {
        Self::with_state(|s| {
            debug_assert!(s.initialized, "try_unprotect_fast called before init");
            unsafe { s.inner.try_unprotect(x) }
        })
    }

    /// Check if a SEXP is protected.
    #[inline]
    pub fn is_protected(x: SEXP) -> bool {
        Self::with_state(|s| {
            if !s.initialized {
                return false;
            }
            s.inner.is_protected(x)
        })
    }

    /// Get reference count.
    #[inline]
    pub fn ref_count(x: SEXP) -> usize {
        Self::with_state(|s| {
            if !s.initialized {
                return 0;
            }
            s.inner.ref_count(x)
        })
    }

    /// Number of protected SEXPs.
    #[inline]
    pub fn len() -> usize {
        Self::with_state(|s| s.inner.len)
    }

    /// Check if empty.
    #[inline]
    pub fn is_empty() -> bool {
        Self::len() == 0
    }

    /// Get capacity.
    #[inline]
    pub fn capacity() -> usize {
        Self::with_state(|s| s.inner.capacity)
    }

    /// Clear all protections.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    pub unsafe fn clear() {
        Self::with_state(|s| {
            if s.initialized {
                unsafe { s.inner.clear() };
            }
        });
    }
}

// Tests are in tests/refcount_protect.rs (requires R runtime via miniextendr-engine)
// endregion
