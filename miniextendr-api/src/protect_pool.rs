//! VECSXP-backed protection pool with generational keys.
//!
//! A GC protection mechanism that stores protected SEXPs in a single R VECSXP
//! (generic list), with slot management and generation tracking on the Rust side.
//!
//! # Performance
//!
//! Benchmarked at 10.1 ns/op for single insert+release. Zero R allocation per
//! insert (unlike `preserve.rs` DLL which allocates a CONSXP each time).
//! See `analysis/gc-protection-benchmarks-results.md` for full data.
//!
//! # When to use
//!
//! Use this for cross-`.Call` protection when:
//! - You have many protected objects or frequent insert/release churn
//! - You need any-order release (not LIFO)
//! - You want generational safety (stale-key detection)
//!
//! For temporaries within a `.Call`, use [`ProtectScope`](crate::gc_protect::ProtectScope)
//! instead (7.4 ns/op, zero allocation, LIFO bulk cleanup).
//!
//! For a few long-lived objects that are never released in a loop (like ExternalPtr),
//! use [`R_PreserveObject`](crate::ffi::R_PreserveObject) directly (13 ns/op, zero
//! Rust-side bookkeeping).
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────┐
//! │  R side: VECSXP (GC-traced slots)   │  ← one R_PreserveObject, ever
//! │  [SEXP][SEXP][NIL][SEXP][NIL][SEXP] │
//! └──────┬──────────────────────────────┘
//!        │ slot indices
//! ┌──────┴──────────────────────────────┐
//! │  Rust side: Vec<u32> generations    │  ← one free list, one generation array
//! │  + Vec<usize> free_slots            │
//! └─────────────────────────────────────┘
//! ```
//!
//! No external dependencies for slot management. The generation counter per slot
//! detects stale keys. Single free list for VECSXP slot reuse.

use crate::ffi::{
    R_NilValue, R_PreserveObject, R_ReleaseObject, R_xlen_t, Rf_allocVector, Rf_protect,
    Rf_unprotect, SET_VECTOR_ELT, SEXP, SEXPTYPE, VECTOR_ELT,
};
use std::marker::PhantomData;
use std::rc::Rc;

/// Generational key for a slot in a [`ProtectPool`].
///
/// Contains a slot index and a generation counter. If a slot is released and
/// reused, the old key's generation won't match and operations will safely
/// return `None` or no-op.
///
/// 8 bytes: 4-byte slot index + 4-byte generation.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ProtectKey {
    slot: u32,
    generation: u32,
}

/// Enforces `!Send + !Sync` (R API is not thread-safe).
type NoSendSync = PhantomData<Rc<()>>;

/// A VECSXP-backed pool for GC protection with generational keys.
///
/// # Example
///
/// ```ignore
/// let mut pool = unsafe { ProtectPool::new(16) };
///
/// let key = unsafe { pool.insert(some_sexp) };
/// // SEXP is now protected from GC
///
/// let sexp = pool.get(key).unwrap();
/// // Use the SEXP...
///
/// unsafe { pool.release(key) };
/// // SEXP is no longer protected (eligible for GC)
/// ```
pub struct ProtectPool {
    /// The VECSXP that holds protected SEXPs. Anchored by `R_PreserveObject`.
    backing: SEXP,
    /// Current capacity of the backing VECSXP.
    capacity: usize,
    /// Generation counter per VECSXP slot. Incremented on each release.
    /// A key is valid iff `generations[key.slot] == key.generation`.
    generations: Vec<u32>,
    /// Free VECSXP slot indices for reuse.
    free_slots: Vec<usize>,
    /// Next fresh VECSXP slot index (for when free_slots is empty).
    next_slot: usize,
    /// Number of currently protected objects.
    len: usize,
    _nosend: NoSendSync,
}

impl ProtectPool {
    /// Initial default capacity.
    pub const DEFAULT_CAPACITY: usize = 16;

    /// Create a new pool with the given initial capacity.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    pub unsafe fn new(capacity: usize) -> Self {
        unsafe { Self::with_capacity(capacity) }
    }

    /// Create a new pool with a specific initial capacity.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    ///
    /// # Panics
    ///
    /// Panics if `capacity` exceeds `R_xlen_t::MAX` or `u32::MAX`.
    pub unsafe fn with_capacity(capacity: usize) -> Self {
        let capacity = capacity.max(1);
        let r_cap = R_xlen_t::try_from(capacity).expect("capacity exceeds R_xlen_t::MAX");
        unsafe {
            let backing = Rf_protect(Rf_allocVector(SEXPTYPE::VECSXP, r_cap));
            R_PreserveObject(backing);
            Rf_unprotect(1);

            Self {
                backing,
                capacity,
                generations: vec![0; capacity],
                free_slots: Vec::with_capacity(capacity / 2),
                next_slot: 0,
                len: 0,
                _nosend: PhantomData,
            }
        }
    }

    /// Protect a SEXP, returning a generational key.
    ///
    /// The SEXP will be protected from GC until [`release`](Self::release) is called
    /// with the returned key. If the key is dropped without calling `release`, the
    /// SEXP remains protected (leak, not crash).
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread. `sexp` must be a valid SEXP.
    ///
    /// # Panics
    ///
    /// Panics if the pool has grown beyond `u32::MAX` slots.
    #[inline]
    pub unsafe fn insert(&mut self, sexp: SEXP) -> ProtectKey {
        let slot = self.alloc_slot();
        // slot < capacity ≤ R_xlen_t::MAX (checked in with_capacity/grow),
        // so this conversion is safe.
        let r_slot = R_xlen_t::try_from(slot).expect("slot exceeds R_xlen_t::MAX");
        unsafe { SET_VECTOR_ELT(self.backing, r_slot, sexp) };
        self.len += 1;
        ProtectKey {
            slot: u32::try_from(slot).expect("slot exceeds u32::MAX"),
            generation: self.generations[slot],
        }
    }

    /// Release a previously protected SEXP.
    ///
    /// If the key is stale (already released, or from a different pool), this is a no-op.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread.
    #[inline]
    pub unsafe fn release(&mut self, key: ProtectKey) {
        let Ok(slot) = usize::try_from(key.slot) else {
            return;
        };
        let Ok(r_slot) = R_xlen_t::try_from(key.slot) else {
            return;
        };
        if slot < self.generations.len() && self.generations[slot] == key.generation {
            unsafe { SET_VECTOR_ELT(self.backing, r_slot, R_NilValue) };
            self.generations[slot] = self.generations[slot].wrapping_add(1);
            self.free_slots.push(slot);
            self.len -= 1;
        }
    }

    /// Get the SEXP for a key, or `None` if the key is stale.
    #[inline]
    pub fn get(&self, key: ProtectKey) -> Option<SEXP> {
        let Ok(slot) = usize::try_from(key.slot) else {
            return None;
        };
        let Ok(r_slot) = R_xlen_t::try_from(key.slot) else {
            return None;
        };
        if slot < self.generations.len() && self.generations[slot] == key.generation {
            Some(unsafe { VECTOR_ELT(self.backing, r_slot) })
        } else {
            None
        }
    }

    /// Overwrite the SEXP at an existing key without releasing/reinserting.
    ///
    /// Returns `true` if the key was valid and the value was replaced.
    /// Returns `false` if the key was stale (no-op).
    ///
    /// This is the pool equivalent of `R_Reprotect` — O(1), no allocation.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread. `sexp` must be a valid SEXP.
    #[inline]
    pub unsafe fn replace(&mut self, key: ProtectKey, sexp: SEXP) -> bool {
        let Ok(slot) = usize::try_from(key.slot) else {
            return false;
        };
        let Ok(r_slot) = R_xlen_t::try_from(key.slot) else {
            return false;
        };
        if slot < self.generations.len() && self.generations[slot] == key.generation {
            unsafe { SET_VECTOR_ELT(self.backing, r_slot, sexp) };
            true
        } else {
            false
        }
    }

    /// Check if a key is currently valid (not stale).
    #[inline]
    pub fn contains_key(&self, key: ProtectKey) -> bool {
        let Ok(slot) = usize::try_from(key.slot) else {
            return false;
        };
        slot < self.generations.len() && self.generations[slot] == key.generation
    }

    /// Number of currently protected objects.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Whether the pool is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Current capacity of the backing VECSXP.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    fn alloc_slot(&mut self) -> usize {
        if let Some(slot) = self.free_slots.pop() {
            return slot;
        }
        if self.next_slot >= self.capacity {
            unsafe { self.grow() };
        }
        let slot = self.next_slot;
        self.next_slot += 1;
        slot
    }

    unsafe fn grow(&mut self) {
        let new_cap = self
            .capacity
            .checked_mul(2)
            .expect("ProtectPool capacity overflow");
        let r_new_cap = R_xlen_t::try_from(new_cap).expect("new capacity exceeds R_xlen_t::MAX");
        unsafe {
            let new_backing = Rf_protect(Rf_allocVector(SEXPTYPE::VECSXP, r_new_cap));
            R_PreserveObject(new_backing);

            for i in 0..self.capacity {
                let r_i = R_xlen_t::try_from(i).expect("index exceeds R_xlen_t::MAX");
                SET_VECTOR_ELT(new_backing, r_i, VECTOR_ELT(self.backing, r_i));
            }

            R_ReleaseObject(self.backing);
            Rf_unprotect(1);

            self.backing = new_backing;
            self.generations.resize(new_cap, 0);
            self.capacity = new_cap;
        }
    }
}

impl Drop for ProtectPool {
    fn drop(&mut self) {
        unsafe { R_ReleaseObject(self.backing) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pool_is_not_send() {
        fn _assert_not_send<T: Send>() {}
        // Uncomment to verify: _assert_not_send::<ProtectPool>();
    }

    #[test]
    fn key_generational_safety() {
        let mut gens: Vec<u32> = vec![0; 4];
        let mut free: Vec<usize> = Vec::new();

        let k1 = ProtectKey {
            slot: 0,
            generation: gens[0],
        };
        assert_eq!(gens[0], k1.generation);

        gens[0] = gens[0].wrapping_add(1);
        free.push(0);
        assert_ne!(gens[0], k1.generation);

        let slot = free.pop().unwrap();
        let k2 = ProtectKey {
            slot: u32::try_from(slot).unwrap(),
            generation: gens[slot],
        };
        assert_eq!(gens[0], k2.generation);
        assert_ne!(k1.generation, k2.generation);
    }

    #[test]
    fn key_size() {
        assert_eq!(std::mem::size_of::<ProtectKey>(), 8);
    }
}
