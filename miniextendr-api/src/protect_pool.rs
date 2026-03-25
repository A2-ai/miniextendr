//! VECSXP-backed protection pool with generational keys.
//!
//! A GC protection mechanism that stores protected SEXPs in a single R VECSXP
//! (generic list), with slot management handled by `slotmap` on the Rust side.
//!
//! # Performance
//!
//! Benchmarked at 9.6 ns/op (Vec) or 11.4 ns/op (slotmap) for single insert+release.
//! Zero R allocation per insert (unlike `preserve.rs` DLL which allocates a CONSXP).
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
//!        │ slot indices (stored as slotmap values)
//! ┌──────┴──────────────────────────────┐
//! │  Rust side: SlotMap<Key, usize>     │  ← generational key → VECSXP index
//! │  manages which slots are occupied   │
//! └─────────────────────────────────────┘
//! ```

use crate::ffi::{
    R_NilValue, R_PreserveObject, R_ReleaseObject, R_xlen_t, Rf_allocVector, Rf_protect,
    Rf_unprotect, SET_VECTOR_ELT, SEXP, SEXPTYPE, VECTOR_ELT,
};
use slotmap::{new_key_type, SlotMap};
use std::cell::Cell;
use std::marker::PhantomData;
use std::rc::Rc;

new_key_type! {
    /// Generational key for a slot in a [`ProtectPool`].
    ///
    /// Contains both a slot index and a generation counter. If a slot is
    /// released and reused, the old key's generation won't match and
    /// operations will safely return `None` or no-op.
    pub struct ProtectKey;
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
    /// Generational slot management. Value = VECSXP slot index.
    /// The slotmap provides generational keys for safe access and
    /// tracks which keys are alive. The usize value maps key → VECSXP index.
    slots: SlotMap<ProtectKey, usize>,
    /// Free VECSXP slot indices for reuse.
    free_slots: Vec<usize>,
    /// Next fresh VECSXP slot index (for when free_slots is empty).
    next_slot: usize,
    /// Number of currently protected objects.
    len: Cell<usize>,
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
    pub unsafe fn with_capacity(capacity: usize) -> Self {
        let capacity = capacity.max(1);
        unsafe {
            let backing = Rf_protect(Rf_allocVector(SEXPTYPE::VECSXP, capacity as R_xlen_t));
            R_PreserveObject(backing);
            Rf_unprotect(1);

            Self {
                backing,
                capacity,
                slots: SlotMap::with_capacity_and_key(capacity),
                free_slots: Vec::with_capacity(capacity / 2),
                next_slot: 0,
                len: Cell::new(0),
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
    #[inline]
    pub unsafe fn insert(&mut self, sexp: SEXP) -> ProtectKey {
        let slot = self.alloc_slot();
        unsafe { SET_VECTOR_ELT(self.backing, slot as R_xlen_t, sexp) };
        self.len.set(self.len.get() + 1);
        self.slots.insert(slot)
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
        if let Some(slot) = self.slots.remove(key) {
            unsafe { SET_VECTOR_ELT(self.backing, slot as R_xlen_t, R_NilValue) };
            self.free_slots.push(slot);
            self.len.set(self.len.get() - 1);
        }
    }

    /// Get the SEXP for a key, or `None` if the key is stale.
    #[inline]
    pub fn get(&self, key: ProtectKey) -> Option<SEXP> {
        let &slot = self.slots.get(key)?;
        Some(unsafe { VECTOR_ELT(self.backing, slot as R_xlen_t) })
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
        if let Some(&slot) = self.slots.get(key) {
            unsafe { SET_VECTOR_ELT(self.backing, slot as R_xlen_t, sexp) };
            true
        } else {
            false
        }
    }

    /// Check if a key is currently valid (not stale).
    #[inline]
    pub fn contains_key(&self, key: ProtectKey) -> bool {
        self.slots.contains_key(key)
    }

    /// Number of currently protected objects.
    #[inline]
    pub fn len(&self) -> usize {
        self.len.get()
    }

    /// Whether the pool is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len.get() == 0
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
        unsafe {
            let new_backing = Rf_protect(Rf_allocVector(SEXPTYPE::VECSXP, new_cap as R_xlen_t));
            R_PreserveObject(new_backing);

            // Copy existing elements
            for i in 0..self.capacity {
                SET_VECTOR_ELT(
                    new_backing,
                    i as R_xlen_t,
                    VECTOR_ELT(self.backing, i as R_xlen_t),
                );
            }

            R_ReleaseObject(self.backing);
            Rf_unprotect(1);

            self.backing = new_backing;
            self.capacity = new_cap;
        }
    }
}

impl Drop for ProtectPool {
    fn drop(&mut self) {
        // Release the backing VECSXP. All protected SEXPs become eligible for GC.
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
        // Should fail because of PhantomData<Rc<()>>
    }

    #[test]
    fn slotmap_generational_safety() {
        // Verify slotmap detects stale keys after removal + reuse
        let mut sm = SlotMap::<ProtectKey, usize>::with_key();
        let k1 = sm.insert(10);
        let k2 = sm.insert(20);
        assert_eq!(sm.get(k1), Some(&10));
        assert_eq!(sm.get(k2), Some(&20));

        sm.remove(k1);
        let k3 = sm.insert(30);
        // k1 is stale — same slot but new generation
        assert!(sm.get(k1).is_none());
        assert_eq!(sm.get(k3), Some(&30));
        assert_eq!(sm.get(k2), Some(&20));
    }
}
