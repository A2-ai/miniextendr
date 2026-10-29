//! [`ProtectPool`]: an owned root set for R objects, held in one R list.
//!
//! The pool stores the objects it roots in a single VECSXP (generic list) that
//! R's collector traces, and keeps the slot bookkeeping and a generation per
//! slot on the Rust side. One `R_PreserveObject` roots that list. Dropping the
//! pool releases it, and with it every object the pool holds.
//!
//! # The per-call root set
//!
//! Code that makes intermediate R objects and needs them alive until its
//! `.Call` returns (attribute vectors, results of R calls, sliced columns)
//! roots each one with [`root`](ProtectPool::root), which returns the object:
//!
//! ```ignore
//! struct Columns {
//!     roots: ProtectPool,
//!     cols: Vec<(String, SEXP)>,
//! }
//!
//! impl Columns {
//!     fn push(&mut self, name: &str, value: impl IntoR) {
//!         let sexp = self.roots.root(value);
//!         self.cols.push((name.to_owned(), sexp));
//!     }
//! }
//! ```
//!
//! A struct can own the pool and drop in any order relative to other pools,
//! scopes and guards. A [`ProtectScope`](crate::ProtectScope) moved into a
//! struct cannot: it unprotects with `UNPROTECT(n)`, which pops the top `n`
//! entries of R's protect stack whoever pushed them, so dropping it out of
//! order unprotects another scope's values. The pool takes no protect-stack
//! entry per value.
//!
//! # The cross-call pool
//!
//! Objects that outlive the `.Call` that made them and are released one at a
//! time, in any order: [`insert`](ProtectPool::insert) returns a generational
//! [`ProtectKey`], and [`release`](ProtectPool::release) frees its slot in
//! O(1). Every owning [`ExternalPtr`](struct@crate::ExternalPtr) is rooted in
//! one process-wide pool.
//!
//! # Contract
//!
//! The pool has the contract of [`Preserved`](crate::Preserved) and the other
//! owned roots: its functions are safe to call on R's main thread, and the type
//! is `!Send + !Sync`, so it is made, used and dropped there. An object stays
//! rooted while it occupies a slot: until its key is released, its slot is
//! replaced, or the pool drops. A [`SEXP`] copied out of the pool is not rooted
//! on its own after that.
//!
//! # Performance
//!
//! Benchmarked at 10.1 ns/op for single insert+release. Zero R allocation per
//! insert (unlike `R_PreserveObject`, which allocates a CONSXP each time). The
//! backing list doubles when it is full.
//! See `analysis/gc-protection-benchmarks-results.md` for full data.
//!
//! # When to use
//!
//! - A temporary within one call: [`ProtectScope`](crate::ProtectScope)
//!   (7.4 ns/op, LIFO bulk cleanup).
//! - Many values, any drop order, or owned by a struct: this pool.
//! - One long-lived object: [`Preserved`](crate::Preserved).
//!
//! `R_PreserveObject` is only appropriate for a *singleton* object that is
//! released (if ever) in LIFO order relative to other preserved objects — e.g.
//! this pool's own backing VECSXP. It is **not** appropriate for objects held
//! in bulk and released in arbitrary or FIFO order: `R_ReleaseObject`'s O(n)
//! precious-list scan degrades to O(n²) on exactly that pattern (60–65× slower
//! than this pool at 10k objects; see
//! `analysis/gc-protection-benchmarks-results.md`). `ExternalPtr` roots through
//! this pool for that reason — a `Vec<ExternalPtr>` releases its roots
//! front-to-back, the worst case for the precious list.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────┐
//! │  R side: VECSXP (GC-traced slots)   │  ← one R_PreserveObject at a time
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

use crate::gc_protect::OwnedProtect;
use crate::into_r::IntoR;
use crate::sys::{R_PreserveObject, R_ReleaseObject, Rf_allocVector, Rf_protect, Rf_unprotect};
use crate::{R_xlen_t, SEXP, SEXPTYPE, SexpExt};
use std::marker::PhantomData;
use std::rc::Rc;

/// Generational key for a slot in a [`ProtectPool`].
///
/// Contains a slot index and a generation counter. Once a slot is released,
/// its old keys are stale: the pool's generation for the slot no longer
/// matches, so operations with them return `None` or do nothing. A key does
/// not record its pool. Use it only with the pool that returned it: another
/// pool treats it as a key of its own slot.
///
/// 8 bytes: 4-byte slot index + 4-byte generation.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ProtectKey {
    slot: u32,
    generation: u32,
}

/// Enforces `!Send + !Sync` (R API is not thread-safe).
type NoSendSync = PhantomData<Rc<()>>;

/// An owned root set for R objects, held in one R list, with generational keys.
///
/// See the [module docs](self) for the two uses: the per-call root set
/// ([`root`](Self::root)) and the cross-call pool ([`insert`](Self::insert) /
/// [`release`](Self::release)).
///
/// # Example
///
/// ```ignore
/// let mut pool = ProtectPool::new(ProtectPool::DEFAULT_CAPACITY);
///
/// // Root a value until the pool drops.
/// let ints = pool.root(vec![1, 2, 3]);
///
/// // Root a value with a key, to release it early.
/// let key = pool.insert(SEXP::scalar_integer(42));
/// let sexp = pool.get(key).unwrap();
/// pool.release(key);
/// assert!(pool.get(key).is_none());
/// ```
///
/// The pool is `!Send` and `!Sync`: R's API belongs to its main thread.
///
/// ```compile_fail
/// fn assert_send<T: Send>() {}
/// assert_send::<miniextendr_api::ProtectPool>();
/// ```
///
/// ```compile_fail
/// fn assert_sync<T: Sync>() {}
/// assert_sync::<miniextendr_api::ProtectPool>();
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

    /// Create a pool with room for `capacity` objects before it grows.
    ///
    /// Same as [`with_capacity`](Self::with_capacity). Call it on R's main
    /// thread.
    ///
    /// # Panics
    ///
    /// Panics if `capacity` exceeds `R_xlen_t::MAX`.
    pub fn new(capacity: usize) -> Self {
        Self::with_capacity(capacity)
    }

    /// Create a pool with room for `capacity` objects before it grows.
    ///
    /// Allocates the backing list and preserves it. Call it on R's main
    /// thread.
    ///
    /// # Panics
    ///
    /// Panics if `capacity` exceeds `R_xlen_t::MAX`.
    pub fn with_capacity(capacity: usize) -> Self {
        let capacity = capacity.max(1);
        let r_cap = R_xlen_t::try_from(capacity).expect("capacity exceeds R_xlen_t::MAX");
        // SAFETY: R's main thread (the pool's contract). The new list stays
        // protected while `R_PreserveObject` allocates its cell.
        let backing = unsafe {
            let backing = Rf_protect(Rf_allocVector(SEXPTYPE::VECSXP, r_cap));
            R_PreserveObject(backing);
            Rf_unprotect(1);
            backing
        };

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

    /// Convert `value` to an R object, root it until the pool drops, and
    /// return it.
    ///
    /// The "keep this alive and give it back" call of the per-call root set.
    /// A [`SEXP`] converts to itself, so `pool.root(sexp)` roots an existing
    /// object, fresh and unprotected or not. Nothing allocates between the
    /// conversion and the insert except the pool's own growth, which keeps the
    /// object reachable (see [`insert`](Self::insert)).
    ///
    /// There is no key: the object is released with the pool. Use
    /// [`insert`](Self::insert) to release it earlier. Call it on R's main
    /// thread.
    #[inline]
    pub fn root(&mut self, value: impl IntoR) -> SEXP {
        let sexp = value.into_sexp();
        self.insert(sexp);
        sexp
    }

    /// Root `sexp`, returning a generational key.
    ///
    /// `sexp` stays rooted until [`release`](Self::release) is called with
    /// the key, its slot is [`replace`](Self::replace)d, or the pool drops. A
    /// key dropped without `release` keeps it rooted until the pool drops.
    ///
    /// `sexp` may be a fresh, unprotected object. Inserting into a free slot
    /// allocates nothing. When the pool is full it grows, which allocates a
    /// new backing list; `sexp` stays on R's protect stack until then, so the
    /// collection that allocation can trigger cannot reclaim it. Call it on R's
    /// main thread.
    ///
    /// # Panics
    ///
    /// Panics if the pool has grown beyond `u32::MAX` slots.
    #[inline]
    pub fn insert(&mut self, sexp: SEXP) -> ProtectKey {
        // Only growth allocates. The guard drops at the end of this call, after
        // `grow` has popped its own entry, so the stack stays LIFO.
        let _pending = self.is_full().then(|| {
            // SAFETY: R's main thread (the pool's contract).
            unsafe { OwnedProtect::new(sexp) }
        });
        let slot = self.alloc_slot();
        // slot < capacity ≤ R_xlen_t::MAX (checked in with_capacity/grow),
        // so this conversion is safe.
        let r_slot = R_xlen_t::try_from(slot).expect("slot exceeds R_xlen_t::MAX");
        self.backing.set_vector_elt(r_slot, sexp);
        self.len += 1;
        ProtectKey {
            slot: u32::try_from(slot).expect("slot exceeds u32::MAX"),
            generation: self.generations[slot],
        }
    }

    /// Release the object `key` roots and free its slot for reuse.
    ///
    /// A stale key (already released) is a no-op. Allocates nothing. Call it
    /// on R's main thread.
    #[inline]
    pub fn release(&mut self, key: ProtectKey) {
        let Ok(slot) = usize::try_from(key.slot) else {
            return;
        };
        let Ok(r_slot) = R_xlen_t::try_from(key.slot) else {
            return;
        };
        if slot < self.generations.len() && self.generations[slot] == key.generation {
            self.backing.set_vector_elt(r_slot, SEXP::nil());
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
            Some(self.backing.vector_elt(r_slot))
        } else {
            None
        }
    }

    /// Root `sexp` in `key`'s slot, in place of the object there.
    ///
    /// Returns `true` if the key was valid and the value was replaced. The
    /// previous object is no longer rooted by this slot. Returns `false` for a
    /// stale key and does nothing: `sexp` is then not rooted.
    ///
    /// This is the pool equivalent of `R_Reprotect`: O(1), and it allocates
    /// nothing, so a fresh `sexp` is safe. Call it on R's main thread.
    #[inline]
    pub fn replace(&mut self, key: ProtectKey, sexp: SEXP) -> bool {
        let Ok(slot) = usize::try_from(key.slot) else {
            return false;
        };
        let Ok(r_slot) = R_xlen_t::try_from(key.slot) else {
            return false;
        };
        if slot < self.generations.len() && self.generations[slot] == key.generation {
            self.backing.set_vector_elt(r_slot, sexp);
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

    /// Whether the next insert grows the backing list.
    #[inline]
    fn is_full(&self) -> bool {
        self.free_slots.is_empty() && self.next_slot >= self.capacity
    }

    fn alloc_slot(&mut self) -> usize {
        if let Some(slot) = self.free_slots.pop() {
            return slot;
        }
        if self.next_slot >= self.capacity {
            self.grow();
        }
        let slot = self.next_slot;
        self.next_slot += 1;
        slot
    }

    /// Double the backing list. It allocates, so the caller keeps any object
    /// it is about to store reachable.
    fn grow(&mut self) {
        let new_cap = self
            .capacity
            .checked_mul(2)
            .expect("ProtectPool capacity overflow");
        let r_new_cap = R_xlen_t::try_from(new_cap).expect("new capacity exceeds R_xlen_t::MAX");
        // SAFETY: R's main thread (the pool's contract). The new list stays
        // protected while `R_PreserveObject` allocates; copying and releasing
        // allocate nothing.
        unsafe {
            let new_backing = Rf_protect(Rf_allocVector(SEXPTYPE::VECSXP, r_new_cap));
            R_PreserveObject(new_backing);

            for i in 0..self.capacity {
                let r_i = R_xlen_t::try_from(i).expect("index exceeds R_xlen_t::MAX");
                new_backing.set_vector_elt(r_i, self.backing.vector_elt(r_i));
            }

            R_ReleaseObject(self.backing);
            Rf_unprotect(1);

            self.backing = new_backing;
        }
        self.generations.resize(new_cap, 0);
        self.capacity = new_cap;
    }
}

impl Drop for ProtectPool {
    fn drop(&mut self) {
        // SAFETY: `!Send` keeps the drop on R's main thread. Releasing the
        // backing list releases every object the pool holds at once, whatever
        // order pools, scopes and guards drop in, and allocates nothing.
        unsafe { R_ReleaseObject(self.backing) };
    }
}

#[cfg(test)]
mod tests;
