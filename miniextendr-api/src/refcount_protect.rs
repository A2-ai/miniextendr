//! Reference-counted GC protection using a map + VECSXP backing.
//!
//! This module provides an alternative to [`gc_protect`](crate::gc_protect) that uses
//! reference counting instead of R's LIFO protect stack. This allows releasing
//! protections in any order and avoids the `--max-ppsize` limit.
//!
//! # Architecture
//!
//! The module is built around two key abstractions:
//!
//! 1. **[`MapStorage`]** - Trait abstracting over map implementations (BTreeMap, HashMap)
//! 2. **[`Arena`]** - Generic arena using RefCell for interior mutability
//!
//! For thread-local storage without RefCell overhead, use the [`define_thread_local_arena!`] macro.
//!
//! # How It Works
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │  Arena<M: MapStorage>                                               │
//! │  ┌─────────────────────────┐   ┌───────────────────────────────────┐│
//! │  │  Map<usize, Entry>      │   │  VECSXP (R_PreserveObject'd)      ││
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
//! | Type | Map | Storage | Use Case |
//! |------|-----|---------|----------|
//! | [`RefCountedArena`] | BTreeMap | RefCell | General purpose, ordered |
//! | [`HashMapArena`] | HashMap | RefCell | Large collections |
//! | [`ThreadLocalArena`] | BTreeMap | thread_local | Lowest overhead |
//! | [`ThreadLocalHashArena`] | HashMap | thread_local | Large + low overhead |
//!
//! ## Fast Hash (feature-gated)
//!
//! With the `refcount-fast-hash` feature enabled, additional types become available:
//!
//! | Type | Map | Storage | Use Case |
//! |------|-----|---------|----------|
//! | [`FastHashMapArena`] | ahash HashMap | RefCell | Faster for large collections |
//! | [`ThreadLocalFastHashArena`] | ahash HashMap | thread_local | Fastest for large + hot loops |
//!
//! These use ahash instead of SipHash for improved throughput. Not DOS-resistant,
//! but suitable for local, non-hostile environments.

use crate::ffi::{
    R_NilValue, R_PreserveObject, R_ReleaseObject, R_xlen_t, Rf_allocVector, Rf_protect,
    Rf_unprotect, SET_VECTOR_ELT, SEXP, SEXPTYPE, VECTOR_ELT,
};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::rc::Rc;

// =============================================================================
// Entry type
// =============================================================================

/// Entry in the reference count map.
///
/// This is an implementation detail exposed for generic type bounds.
#[derive(Debug, Clone, Copy)]
#[doc(hidden)]
pub struct Entry {
    /// Reference count (how many times this SEXP has been protected)
    count: usize,
    /// Index in the backing VECSXP
    index: usize,
}

// =============================================================================
// MapStorage trait
// =============================================================================

/// Trait abstracting over map implementations for arena storage.
///
/// This allows [`Arena`] to be generic over the underlying map type,
/// supporting both `BTreeMap` and `HashMap`.
pub trait MapStorage: Default {
    /// Get an entry by key.
    fn get(&self, key: &usize) -> Option<&Entry>;

    /// Get a mutable entry by key.
    fn get_mut(&mut self, key: &usize) -> Option<&mut Entry>;

    /// Insert an entry, returning the old value if present.
    fn insert(&mut self, key: usize, entry: Entry) -> Option<Entry>;

    /// Remove an entry by key.
    fn remove(&mut self, key: &usize) -> Option<Entry>;

    /// Check if a key exists.
    fn contains_key(&self, key: &usize) -> bool;

    /// Iterate over all entries.
    fn for_each_entry<F: FnMut(&Entry)>(&self, f: F);

    /// Clear all entries.
    fn clear(&mut self);

    /// Reserve capacity for additional entries.
    ///
    /// This is a no-op for ordered maps (BTreeMap) but can improve performance
    /// for hash maps by avoiding rehashing during bulk inserts.
    #[inline]
    fn reserve(&mut self, _additional: usize) {
        // Default no-op for maps that don't support reservation
    }

    /// Decrement the count for a key and remove if zero.
    ///
    /// Returns `Some(true)` if entry was found and removed,
    /// `Some(false)` if entry was found but count > 0 after decrement,
    /// `None` if entry was not found.
    ///
    /// This uses entry API when available for single-lookup performance.
    fn decrement_and_maybe_remove(&mut self, key: &usize) -> Option<(bool, usize)> {
        if let Some(entry) = self.get_mut(key) {
            entry.count -= 1;
            if entry.count == 0 {
                let index = entry.index;
                self.remove(key);
                Some((true, index))
            } else {
                Some((false, entry.index))
            }
        } else {
            None
        }
    }
}

impl MapStorage for BTreeMap<usize, Entry> {
    #[inline]
    fn get(&self, key: &usize) -> Option<&Entry> {
        BTreeMap::get(self, key)
    }

    #[inline]
    fn get_mut(&mut self, key: &usize) -> Option<&mut Entry> {
        BTreeMap::get_mut(self, key)
    }

    #[inline]
    fn insert(&mut self, key: usize, entry: Entry) -> Option<Entry> {
        BTreeMap::insert(self, key, entry)
    }

    #[inline]
    fn remove(&mut self, key: &usize) -> Option<Entry> {
        BTreeMap::remove(self, key)
    }

    #[inline]
    fn contains_key(&self, key: &usize) -> bool {
        BTreeMap::contains_key(self, key)
    }

    #[inline]
    fn for_each_entry<F: FnMut(&Entry)>(&self, mut f: F) {
        for entry in self.values() {
            f(entry);
        }
    }

    #[inline]
    fn clear(&mut self) {
        BTreeMap::clear(self);
    }
}

impl MapStorage for HashMap<usize, Entry> {
    #[inline]
    fn get(&self, key: &usize) -> Option<&Entry> {
        HashMap::get(self, key)
    }

    #[inline]
    fn get_mut(&mut self, key: &usize) -> Option<&mut Entry> {
        HashMap::get_mut(self, key)
    }

    #[inline]
    fn insert(&mut self, key: usize, entry: Entry) -> Option<Entry> {
        HashMap::insert(self, key, entry)
    }

    #[inline]
    fn remove(&mut self, key: &usize) -> Option<Entry> {
        HashMap::remove(self, key)
    }

    #[inline]
    fn contains_key(&self, key: &usize) -> bool {
        HashMap::contains_key(self, key)
    }

    #[inline]
    fn for_each_entry<F: FnMut(&Entry)>(&self, mut f: F) {
        for entry in self.values() {
            f(entry);
        }
    }

    #[inline]
    fn clear(&mut self) {
        HashMap::clear(self);
    }

    #[inline]
    fn reserve(&mut self, additional: usize) {
        HashMap::reserve(self, additional);
    }

    /// Entry-based decrement for single-lookup performance.
    #[inline]
    fn decrement_and_maybe_remove(&mut self, key: &usize) -> Option<(bool, usize)> {
        use std::collections::hash_map::Entry as HashEntry;

        match self.entry(*key) {
            HashEntry::Occupied(mut occupied) => {
                let entry = occupied.get_mut();
                entry.count -= 1;
                if entry.count == 0 {
                    let index = entry.index;
                    occupied.remove();
                    Some((true, index))
                } else {
                    Some((false, entry.index))
                }
            }
            HashEntry::Vacant(_) => None,
        }
    }
}

// =============================================================================
// Core arena state (shared between RefCell and thread_local variants)
// =============================================================================

/// Core arena state without interior mutability.
///
/// This is used internally by both [`Arena`] (with RefCell) and
/// thread-local arenas (with UnsafeCell).
#[doc(hidden)]
pub struct ArenaState<M> {
    /// Map from SEXP pointer to entry
    pub map: MaybeUninit<M>,
    /// Backing VECSXP (preserved via R_PreserveObject)
    pub backing: SEXP,
    /// Current capacity
    pub capacity: usize,
    /// Number of active entries
    pub len: usize,
    /// Free list for slot reuse
    pub free_list: Vec<usize>,
}

impl<M: MapStorage> ArenaState<M> {
    /// Initial capacity for the backing VECSXP.
    pub const INITIAL_CAPACITY: usize = 16;

    /// Create uninitialized state (for thread_local).
    pub const fn uninit() -> Self {
        Self {
            map: MaybeUninit::uninit(),
            backing: SEXP(std::ptr::null_mut()),
            capacity: 0,
            len: 0,
            free_list: Vec::new(),
        }
    }

    /// Initialize the state.
    ///
    /// # Safety
    ///
    /// Must be called exactly once before using the state.
    pub unsafe fn init(&mut self, capacity: usize) {
        let capacity = capacity.max(1);
        unsafe {
            let backing = Rf_protect(Rf_allocVector(SEXPTYPE::VECSXP, capacity as R_xlen_t));
            R_PreserveObject(backing);
            Rf_unprotect(1);

            let mut map = M::default();
            map.reserve(capacity);
            self.map.write(map);
            self.backing = backing;
            self.capacity = capacity;
            self.len = 0;
            self.free_list = Vec::with_capacity(capacity);
        }
    }

    /// Create initialized state.
    unsafe fn new(capacity: usize) -> Self {
        let capacity = capacity.max(1);
        let mut map = M::default();
        map.reserve(capacity);
        let mut state = Self {
            map: MaybeUninit::new(map),
            backing: SEXP(std::ptr::null_mut()),
            capacity: 0,
            len: 0,
            free_list: Vec::with_capacity(capacity),
        };
        unsafe { state.init_backing(capacity) };
        state
    }

    /// Initialize just the backing (map already initialized).
    unsafe fn init_backing(&mut self, capacity: usize) {
        let capacity = capacity.max(1);
        unsafe {
            let backing = Rf_protect(Rf_allocVector(SEXPTYPE::VECSXP, capacity as R_xlen_t));
            R_PreserveObject(backing);
            Rf_unprotect(1);

            self.backing = backing;
            self.capacity = capacity;
        }
    }

    /// Get a reference to the map.
    #[inline]
    fn map(&self) -> &M {
        // SAFETY: Map is initialized before any access
        unsafe { self.map.assume_init_ref() }
    }

    /// Get a mutable reference to the map.
    #[inline]
    fn map_mut(&mut self) -> &mut M {
        // SAFETY: Map is initialized before any access
        unsafe { self.map.assume_init_mut() }
    }

    /// Protect a SEXP from garbage collection.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread. The SEXP must be valid.
    #[inline]
    pub unsafe fn protect(&mut self, x: SEXP) -> SEXP {
        unsafe {
            if std::ptr::eq(x.0, R_NilValue.0) {
                return x;
            }

            let key = x.0 as usize;

            if let Some(entry) = self.map_mut().get_mut(&key) {
                entry.count += 1;
            } else {
                let index = self.allocate_slot();
                SET_VECTOR_ELT(self.backing, index as R_xlen_t, x);
                self.map_mut().insert(key, Entry { count: 1, index });
                self.len += 1;
            }

            x
        }
    }

    /// Unprotect a SEXP, allowing garbage collection when refcount reaches zero.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread. The SEXP must have been
    /// previously protected by this arena.
    #[inline]
    pub unsafe fn unprotect(&mut self, x: SEXP) {
        unsafe {
            if std::ptr::eq(x.0, R_NilValue.0) {
                return;
            }

            let key = x.0 as usize;

            // Use entry-based single-lookup for HashMap, double-lookup for BTreeMap
            match self.map_mut().decrement_and_maybe_remove(&key) {
                Some((true, index)) => {
                    // Entry was removed (count reached 0)
                    SET_VECTOR_ELT(self.backing, index as R_xlen_t, R_NilValue);
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
    }

    /// Try to unprotect a SEXP. Returns false if not protected by this arena.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn try_unprotect(&mut self, x: SEXP) -> bool {
        unsafe {
            if std::ptr::eq(x.0, R_NilValue.0) {
                return false;
            }

            let key = x.0 as usize;

            // Use entry-based single-lookup for HashMap, double-lookup for BTreeMap
            match self.map_mut().decrement_and_maybe_remove(&key) {
                Some((true, index)) => {
                    // Entry was removed (count reached 0)
                    SET_VECTOR_ELT(self.backing, index as R_xlen_t, R_NilValue);
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
    }

    #[inline]
    /// Returns true if this arena currently protects `x`.
    pub fn is_protected(&self, x: SEXP) -> bool {
        if std::ptr::eq(x.0, unsafe { R_NilValue.0 }) {
            return false;
        }
        let key = x.0 as usize;
        self.map().contains_key(&key)
    }

    #[inline]
    /// Returns the current reference count for `x` in this arena.
    ///
    /// Returns 0 if `x` is not protected or is `R_NilValue`.
    pub fn ref_count(&self, x: SEXP) -> usize {
        if std::ptr::eq(x.0, unsafe { R_NilValue.0 }) {
            return 0;
        }
        let key = x.0 as usize;
        self.map().get(&key).map(|e| e.count).unwrap_or(0)
    }

    fn allocate_slot(&mut self) -> usize {
        if let Some(index) = self.free_list.pop() {
            return index;
        }

        if self.len >= self.capacity {
            unsafe { self.grow() };
        }

        self.len
    }

    unsafe fn grow(&mut self) {
        let old_capacity = self.capacity;
        let new_capacity = old_capacity * 2;
        let old_backing = self.backing;

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

            self.backing = new_backing;
            self.capacity = new_capacity;
        }
    }

    /// Clear all protected values from the arena.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    pub unsafe fn clear(&mut self) {
        unsafe {
            self.map().for_each_entry(|entry| {
                SET_VECTOR_ELT(self.backing, entry.index as R_xlen_t, R_NilValue);
            });
        }
        self.map_mut().clear();
        self.free_list.clear();
        self.len = 0;
    }

    unsafe fn release_backing(&mut self) {
        if !self.backing.0.is_null() {
            unsafe { R_ReleaseObject(self.backing) };
            self.backing = SEXP(std::ptr::null_mut());
        }
    }
}

// =============================================================================
// Arena<M> - RefCell-based generic arena
// =============================================================================

/// Enforces `!Send + !Sync` (R API is not thread-safe).
type NoSendSync = PhantomData<Rc<()>>;

/// A reference-counted arena for GC protection, generic over map type.
///
/// This provides an alternative to R's PROTECT stack that:
/// - Uses reference counting for each SEXP
/// - Allows releasing protections in any order
/// - Has no stack size limit (uses heap allocation)
///
/// # Type Aliases
///
/// - [`RefCountedArena`] = `Arena<BTreeMap<...>>` (ordered, good for ref counting)
/// - [`HashMapArena`] = `Arena<HashMap<...>>` (faster for large collections)
pub struct Arena<M: MapStorage> {
    state: RefCell<ArenaState<M>>,
    _nosend: NoSendSync,
}

impl<M: MapStorage> Arena<M> {
    /// Create a new arena with default capacity.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    pub unsafe fn new() -> Self {
        unsafe { Self::with_capacity(ArenaState::<M>::INITIAL_CAPACITY) }
    }

    /// Create a new arena with specific initial capacity.
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
    pub unsafe fn guard(&self, x: SEXP) -> ArenaGuard<'_, M> {
        unsafe { ArenaGuard::new(self, x) }
    }
}

impl<M: MapStorage> Drop for Arena<M> {
    fn drop(&mut self) {
        unsafe { self.state.borrow_mut().release_backing() };
    }
}

impl<M: MapStorage> Default for Arena<M> {
    fn default() -> Self {
        unsafe { Self::new() }
    }
}

// =============================================================================
// Type aliases for common arena types
// =============================================================================

/// BTreeMap-based arena (default, good for reference counting).
pub type RefCountedArena = Arena<BTreeMap<usize, Entry>>;

/// HashMap-based arena (faster for large collections).
pub type HashMapArena = Arena<HashMap<usize, Entry>>;

// =============================================================================
// Fast hash types (feature-gated)
// =============================================================================

#[cfg(feature = "refcount-fast-hash")]
mod fast_hash {
    use super::{Entry, MapStorage};
    use std::collections::hash_map::Entry as StdEntry;

    /// HashMap with ahash for faster hashing (not DOS-resistant).
    pub type FastHashMap = std::collections::HashMap<usize, Entry, ahash::RandomState>;

    impl MapStorage for FastHashMap {
        #[inline]
        fn get(&self, key: &usize) -> Option<&Entry> {
            std::collections::HashMap::get(self, key)
        }

        #[inline]
        fn get_mut(&mut self, key: &usize) -> Option<&mut Entry> {
            std::collections::HashMap::get_mut(self, key)
        }

        #[inline]
        fn insert(&mut self, key: usize, entry: Entry) -> Option<Entry> {
            std::collections::HashMap::insert(self, key, entry)
        }

        #[inline]
        fn remove(&mut self, key: &usize) -> Option<Entry> {
            std::collections::HashMap::remove(self, key)
        }

        #[inline]
        fn contains_key(&self, key: &usize) -> bool {
            std::collections::HashMap::contains_key(self, key)
        }

        #[inline]
        fn for_each_entry<F: FnMut(&Entry)>(&self, mut f: F) {
            for entry in self.values() {
                f(entry);
            }
        }

        #[inline]
        fn clear(&mut self) {
            std::collections::HashMap::clear(self);
        }

        #[inline]
        fn reserve(&mut self, additional: usize) {
            std::collections::HashMap::reserve(self, additional);
        }

        /// Entry-based decrement for single-lookup performance.
        #[inline]
        fn decrement_and_maybe_remove(&mut self, key: &usize) -> Option<(bool, usize)> {
            match self.entry(*key) {
                StdEntry::Occupied(mut occupied) => {
                    let entry = occupied.get_mut();
                    entry.count -= 1;
                    if entry.count == 0 {
                        let index = entry.index;
                        occupied.remove();
                        Some((true, index))
                    } else {
                        Some((false, entry.index))
                    }
                }
                StdEntry::Vacant(_) => None,
            }
        }
    }
}

#[cfg(feature = "refcount-fast-hash")]
/// HashMap implementation used by `FastHashMapArena` when enabled.
pub use fast_hash::FastHashMap;

/// Fast hash arena using ahash (requires `refcount-fast-hash` feature).
///
/// Uses ahash instead of SipHash for improved throughput on large collections.
/// Not DOS-resistant, suitable for local, non-hostile environments.
#[cfg(feature = "refcount-fast-hash")]
pub type FastHashMapArena = Arena<FastHashMap>;

// =============================================================================
// RAII Guard
// =============================================================================

/// An RAII guard that unprotects a SEXP when dropped.
pub struct ArenaGuard<'a, M: MapStorage> {
    arena: &'a Arena<M>,
    sexp: SEXP,
}

impl<'a, M: MapStorage> ArenaGuard<'a, M> {
    /// Create a new guard that protects the SEXP and unprotects on drop.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread. The SEXP must be valid.
    #[inline]
    pub unsafe fn new(arena: &'a Arena<M>, sexp: SEXP) -> Self {
        unsafe { arena.protect(sexp) };
        Self { arena, sexp }
    }

    #[inline]
    /// Returns the protected SEXP.
    pub fn get(&self) -> SEXP {
        self.sexp
    }
}

impl<M: MapStorage> Drop for ArenaGuard<'_, M> {
    fn drop(&mut self) {
        unsafe { self.arena.unprotect(self.sexp) };
    }
}

impl<M: MapStorage> std::ops::Deref for ArenaGuard<'_, M> {
    type Target = SEXP;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.sexp
    }
}

/// Legacy type alias for backwards compatibility.
pub type RefCountedGuard<'a> = ArenaGuard<'a, BTreeMap<usize, Entry>>;

// =============================================================================
// Thread-local arena macro
// =============================================================================

/// Macro to define a thread-local arena with a specific map type.
///
/// This creates a zero-sized struct with static methods that access
/// thread-local storage, eliminating RefCell overhead.
///
/// # Example
///
/// ```ignore
/// define_thread_local_arena!(
///     /// My custom thread-local arena.
///     pub MyArena,
///     BTreeMap<usize, Entry>,
///     MY_ARENA_STATE
/// );
/// ```
#[macro_export]
macro_rules! define_thread_local_arena {
    (
        $(#[$meta:meta])*
        $vis:vis $name:ident,
        $map:ty,
        $state_name:ident
    ) => {
        thread_local! {
            static $state_name: std::cell::UnsafeCell<$crate::refcount_protect::ThreadLocalState<$map>> =
                const { std::cell::UnsafeCell::new($crate::refcount_protect::ThreadLocalState::uninit()) };
        }

        $(#[$meta])*
        $vis struct $name;

        impl $name {
            /// Initialize the arena with default capacity (called automatically on first use).
            ///
            /// # Safety
            ///
            /// Must be called from the R main thread.
            pub unsafe fn init() {
                $state_name.with(|cell| {
                    let state = unsafe { &mut *cell.get() };
                    if !state.initialized {
                        unsafe { state.init() };
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
                $state_name.with(|cell| {
                    let state = unsafe { &mut *cell.get() };
                    if !state.initialized {
                        unsafe { state.init_with_capacity(capacity) };
                    }
                });
            }

            /// Protect a SEXP, incrementing its reference count.
            ///
            /// # Safety
            ///
            /// Must be called from the R main thread.
            #[inline]
            pub unsafe fn protect(x: $crate::ffi::SEXP) -> $crate::ffi::SEXP {
                $state_name.with(|cell| {
                    let state = unsafe { &mut *cell.get() };
                    if !state.initialized {
                        unsafe { state.init() };
                    }
                    unsafe { state.inner.protect(x) }
                })
            }

            /// Unprotect a SEXP.
            ///
            /// # Safety
            ///
            /// Must be called from the R main thread.
            #[inline]
            pub unsafe fn unprotect(x: $crate::ffi::SEXP) {
                $state_name.with(|cell| {
                    let state = unsafe { &mut *cell.get() };
                    unsafe { state.inner.unprotect(x) };
                });
            }

            /// Try to unprotect a SEXP.
            ///
            /// # Safety
            ///
            /// Must be called from the R main thread.
            #[inline]
            pub unsafe fn try_unprotect(x: $crate::ffi::SEXP) -> bool {
                $state_name.with(|cell| {
                    let state = unsafe { &mut *cell.get() };
                    unsafe { state.inner.try_unprotect(x) }
                })
            }

            // =========================================================================
            // Fast API - skip initialization check for hot loops
            // =========================================================================

            /// Protect without checking initialization.
            ///
            /// For hot loops where `init()` or `init_with_capacity()` has already been called.
            ///
            /// # Safety
            ///
            /// - Must be called from the R main thread.
            /// - The arena must have been initialized via `init()` or `init_with_capacity()`.
            #[inline]
            pub unsafe fn protect_fast(x: $crate::ffi::SEXP) -> $crate::ffi::SEXP {
                $state_name.with(|cell| {
                    let state = unsafe { &mut *cell.get() };
                    debug_assert!(state.initialized, "protect_fast called before init");
                    unsafe { state.inner.protect(x) }
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
            pub unsafe fn unprotect_fast(x: $crate::ffi::SEXP) {
                $state_name.with(|cell| {
                    let state = unsafe { &mut *cell.get() };
                    debug_assert!(state.initialized, "unprotect_fast called before init");
                    unsafe { state.inner.unprotect(x) };
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
            pub unsafe fn try_unprotect_fast(x: $crate::ffi::SEXP) -> bool {
                $state_name.with(|cell| {
                    let state = unsafe { &mut *cell.get() };
                    debug_assert!(state.initialized, "try_unprotect_fast called before init");
                    unsafe { state.inner.try_unprotect(x) }
                })
            }

            /// Check if a SEXP is protected.
            #[inline]
            pub fn is_protected(x: $crate::ffi::SEXP) -> bool {
                $state_name.with(|cell| {
                    let state = unsafe { &*cell.get() };
                    if !state.initialized { return false; }
                    state.inner.is_protected(x)
                })
            }

            /// Get reference count.
            #[inline]
            pub fn ref_count(x: $crate::ffi::SEXP) -> usize {
                $state_name.with(|cell| {
                    let state = unsafe { &*cell.get() };
                    if !state.initialized { return 0; }
                    state.inner.ref_count(x)
                })
            }

            /// Number of protected SEXPs.
            #[inline]
            pub fn len() -> usize {
                $state_name.with(|cell| {
                    let state = unsafe { &*cell.get() };
                    state.inner.len
                })
            }

            /// Check if empty.
            #[inline]
            pub fn is_empty() -> bool {
                Self::len() == 0
            }

            /// Get capacity.
            #[inline]
            pub fn capacity() -> usize {
                $state_name.with(|cell| {
                    let state = unsafe { &*cell.get() };
                    state.inner.capacity
                })
            }

            /// Clear all protections.
            ///
            /// # Safety
            ///
            /// Must be called from the R main thread.
            pub unsafe fn clear() {
                $state_name.with(|cell| {
                    let state = unsafe { &mut *cell.get() };
                    if state.initialized {
                        unsafe { state.inner.clear() };
                    }
                });
            }
        }
    };
}

/// State wrapper for thread-local arenas (used by macro).
#[doc(hidden)]
pub struct ThreadLocalState<M: MapStorage> {
    pub inner: ArenaState<M>,
    pub initialized: bool,
}

impl<M: MapStorage> ThreadLocalState<M> {
    /// Create an uninitialized thread-local arena state.
    ///
    /// Call `init` or `init_with_capacity` before use.
    pub const fn uninit() -> Self {
        Self {
            inner: ArenaState::uninit(),
            initialized: false,
        }
    }

    /// Initialize with default capacity.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread. Must only be called once.
    pub unsafe fn init(&mut self) {
        unsafe { self.inner.init(ArenaState::<M>::INITIAL_CAPACITY) };
        self.initialized = true;
    }

    /// Initialize with specific capacity.
    ///
    /// Use this when you know the expected number of distinct protected values
    /// to avoid backing VECSXP growth and map rehashing.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread. Must only be called once.
    pub unsafe fn init_with_capacity(&mut self, capacity: usize) {
        unsafe { self.inner.init(capacity) };
        self.initialized = true;
    }
}

// =============================================================================
// Built-in thread-local arenas
// =============================================================================

define_thread_local_arena!(
    /// Thread-local BTreeMap-based arena.
    ///
    /// This provides the lowest overhead for protection operations by
    /// eliminating RefCell borrow checking.
    pub ThreadLocalArena,
    BTreeMap<usize, Entry>,
    THREAD_LOCAL_BTREE_STATE
);

define_thread_local_arena!(
    /// Thread-local HashMap-based arena.
    ///
    /// Combines HashMap's performance for large collections with
    /// thread-local storage's low overhead.
    pub ThreadLocalHashArena,
    HashMap<usize, Entry>,
    THREAD_LOCAL_HASH_STATE
);

// =============================================================================
// Fast hash thread-local arena (feature-gated)
// =============================================================================

#[cfg(feature = "refcount-fast-hash")]
define_thread_local_arena!(
    /// Thread-local fast hash arena using ahash.
    ///
    /// Combines ahash's faster hashing with thread-local storage's low overhead.
    /// Ideal for hot loops protecting many distinct values.
    ///
    /// Not DOS-resistant, suitable for local, non-hostile environments.
    ///
    /// Requires the `refcount-fast-hash` feature.
    pub ThreadLocalFastHashArena,
    FastHashMap,
    THREAD_LOCAL_FAST_HASH_STATE
);

// Tests are in tests/refcount_protect.rs (requires R runtime via miniextendr-engine)
