//! VECSXP pool prototypes for benchmarking.
//!
//! These are standalone implementations used to benchmark different pool
//! strategies head-to-head. The winner will be integrated into miniextendr-api.

use miniextendr_api::ffi::{
    self, R_NilValue, R_PreserveObject, R_ReleaseObject, Rf_protect, Rf_unprotect, SEXP,
};
use std::collections::VecDeque;

// region: VecPool — Vec<usize> free list (LIFO reuse)

/// VECSXP pool with Vec<usize> free list (LIFO slot reuse).
///
/// Simplest possible pool. Stale handles are not detected.
pub struct VecPool {
    pub backing: SEXP,
    pub capacity: usize,
    pub len: usize,
    free_list: Vec<usize>,
}

impl VecPool {
    pub unsafe fn new(capacity: usize) -> Self {
        let capacity = capacity.max(1);
        unsafe {
            let backing =
                ffi::Rf_allocVector(ffi::SEXPTYPE::VECSXP, capacity as ffi::R_xlen_t);
            R_PreserveObject(backing);
            Self {
                backing,
                capacity,
                len: 0,
                free_list: Vec::with_capacity(capacity / 2),
            }
        }
    }

    #[inline]
    pub unsafe fn insert(&mut self, sexp: SEXP) -> usize {
        let slot = if let Some(s) = self.free_list.pop() {
            s
        } else {
            if self.len >= self.capacity {
                unsafe { self.grow() };
            }
            let s = self.len;
            self.len += 1;
            s
        };
        unsafe { ffi::SET_VECTOR_ELT(self.backing, slot as ffi::R_xlen_t, sexp) };
        slot
    }

    #[inline]
    pub unsafe fn release(&mut self, slot: usize) {
        unsafe { ffi::SET_VECTOR_ELT(self.backing, slot as ffi::R_xlen_t, R_NilValue) };
        self.free_list.push(slot);
    }

    #[inline]
    pub unsafe fn get(&self, slot: usize) -> SEXP {
        unsafe { ffi::VECTOR_ELT(self.backing, slot as ffi::R_xlen_t) }
    }

    unsafe fn grow(&mut self) {
        let new_cap = self.capacity * 2;
        unsafe {
            let new_backing =
                ffi::Rf_allocVector(ffi::SEXPTYPE::VECSXP, new_cap as ffi::R_xlen_t);
            Rf_protect(new_backing);
            R_PreserveObject(new_backing);
            for i in 0..self.capacity {
                ffi::SET_VECTOR_ELT(
                    new_backing,
                    i as ffi::R_xlen_t,
                    ffi::VECTOR_ELT(self.backing, i as ffi::R_xlen_t),
                );
            }
            R_ReleaseObject(self.backing);
            Rf_unprotect(1);
            self.backing = new_backing;
            self.capacity = new_cap;
        }
    }
}

impl Drop for VecPool {
    fn drop(&mut self) {
        unsafe { R_ReleaseObject(self.backing) };
    }
}

// endregion

// region: DequePool — VecDeque<usize> free list (FIFO reuse)

/// VECSXP pool with VecDeque<usize> free list (FIFO slot reuse).
///
/// Released slots go to the back; allocations come from the front.
/// Delays reuse of recently-freed slots.
pub struct DequePool {
    pub backing: SEXP,
    pub capacity: usize,
    pub len: usize,
    free_list: VecDeque<usize>,
}

impl DequePool {
    pub unsafe fn new(capacity: usize) -> Self {
        let capacity = capacity.max(1);
        unsafe {
            let backing =
                ffi::Rf_allocVector(ffi::SEXPTYPE::VECSXP, capacity as ffi::R_xlen_t);
            R_PreserveObject(backing);
            Self {
                backing,
                capacity,
                len: 0,
                free_list: VecDeque::with_capacity(capacity / 2),
            }
        }
    }

    #[inline]
    pub unsafe fn insert(&mut self, sexp: SEXP) -> usize {
        let slot = if let Some(s) = self.free_list.pop_front() {
            s
        } else {
            if self.len >= self.capacity {
                unsafe { self.grow() };
            }
            let s = self.len;
            self.len += 1;
            s
        };
        unsafe { ffi::SET_VECTOR_ELT(self.backing, slot as ffi::R_xlen_t, sexp) };
        slot
    }

    #[inline]
    pub unsafe fn release(&mut self, slot: usize) {
        unsafe { ffi::SET_VECTOR_ELT(self.backing, slot as ffi::R_xlen_t, R_NilValue) };
        self.free_list.push_back(slot);
    }

    unsafe fn grow(&mut self) {
        let new_cap = self.capacity * 2;
        unsafe {
            let new_backing =
                ffi::Rf_allocVector(ffi::SEXPTYPE::VECSXP, new_cap as ffi::R_xlen_t);
            Rf_protect(new_backing);
            R_PreserveObject(new_backing);
            for i in 0..self.capacity {
                ffi::SET_VECTOR_ELT(
                    new_backing,
                    i as ffi::R_xlen_t,
                    ffi::VECTOR_ELT(self.backing, i as ffi::R_xlen_t),
                );
            }
            R_ReleaseObject(self.backing);
            Rf_unprotect(1);
            self.backing = new_backing;
            self.capacity = new_cap;
        }
    }
}

impl Drop for DequePool {
    fn drop(&mut self) {
        unsafe { R_ReleaseObject(self.backing) };
    }
}

// endregion

// region: SlotmapPool — slotmap with generational keys

use slotmap::{new_key_type, SlotMap};

new_key_type! {
    /// Generational key for VECSXP pool slots.
    pub struct ProtectKey;
}

/// VECSXP pool with slotmap generational index management.
///
/// Stale keys are safely detected via generation counter.
pub struct SlotmapPool {
    pub backing: SEXP,
    pub capacity: usize,
    slots: SlotMap<ProtectKey, usize>, // value = VECSXP slot index
    next_slot: usize,
    free_slots: Vec<usize>,
}

impl SlotmapPool {
    pub unsafe fn new(capacity: usize) -> Self {
        let capacity = capacity.max(1);
        unsafe {
            let backing =
                ffi::Rf_allocVector(ffi::SEXPTYPE::VECSXP, capacity as ffi::R_xlen_t);
            R_PreserveObject(backing);
            Self {
                backing,
                capacity,
                slots: SlotMap::with_capacity_and_key(capacity),
                next_slot: 0,
                free_slots: Vec::with_capacity(capacity / 2),
            }
        }
    }

    #[inline]
    pub unsafe fn insert(&mut self, sexp: SEXP) -> ProtectKey {
        let slot = if let Some(s) = self.free_slots.pop() {
            s
        } else {
            if self.next_slot >= self.capacity {
                unsafe { self.grow() };
            }
            let s = self.next_slot;
            self.next_slot += 1;
            s
        };
        unsafe { ffi::SET_VECTOR_ELT(self.backing, slot as ffi::R_xlen_t, sexp) };
        self.slots.insert(slot)
    }

    #[inline]
    pub unsafe fn release(&mut self, key: ProtectKey) {
        if let Some(slot) = self.slots.remove(key) {
            unsafe { ffi::SET_VECTOR_ELT(self.backing, slot as ffi::R_xlen_t, R_NilValue) };
            self.free_slots.push(slot);
        }
    }

    #[inline]
    pub fn get(&self, key: ProtectKey) -> Option<SEXP> {
        let &slot = self.slots.get(key)?;
        Some(unsafe { ffi::VECTOR_ELT(self.backing, slot as ffi::R_xlen_t) })
    }

    unsafe fn grow(&mut self) {
        let new_cap = self.capacity * 2;
        unsafe {
            let new_backing =
                ffi::Rf_allocVector(ffi::SEXPTYPE::VECSXP, new_cap as ffi::R_xlen_t);
            Rf_protect(new_backing);
            R_PreserveObject(new_backing);
            for i in 0..self.capacity {
                ffi::SET_VECTOR_ELT(
                    new_backing,
                    i as ffi::R_xlen_t,
                    ffi::VECTOR_ELT(self.backing, i as ffi::R_xlen_t),
                );
            }
            R_ReleaseObject(self.backing);
            Rf_unprotect(1);
            self.backing = new_backing;
            self.capacity = new_cap;
        }
    }
}

impl Drop for SlotmapPool {
    fn drop(&mut self) {
        unsafe { R_ReleaseObject(self.backing) };
    }
}

// endregion
