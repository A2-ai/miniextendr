//! Cached field values: `#[r_data(cache)]` (#2271).
//!
//! The R getter of a struct field converts a clone of the Rust value on every
//! read, so a large field costs a copy and a conversion per `$`. The getter of
//! a field marked `#[r_data(cache)]` keeps the R object its first read makes,
//! and later reads return that same object, until the value is borrowed
//! exclusively.
//!
//! # Storage
//!
//! The cached objects live in the pool that roots owning `ExternalPtr`
//! handles ([`root_owned`]), and their keys in a table keyed by the address of
//! the value's `RefCell<T>`: one entry per object, one slot per cached field,
//! empty until the field's first read. Not in the pointer's `prot` list, which
//! `saveRDS()` writes: a save never carries a cache, and an object
//! `readRDS()` brings back starts with none. A clone has a `RefCell` of its
//! own, so it starts with none too.
//!
//! # Invalidation
//!
//! Every exclusive borrow of the value drops its entry
//! ([`borrow_value_mut`](super::borrow_value_mut)): a `&mut self` method, a
//! `&mut T` or [`ExternalPtrMut`](super::ExternalPtrMut) argument,
//! [`ExternalPtr::borrow_mut`](super::ExternalPtr::borrow_mut), an R setter,
//! a consuming `self` method, `into_inner`. The check is a constant
//! ([`TypedExternal::CACHED_FIELD_COUNT`](super::TypedExternal::CACHED_FIELD_COUNT)),
//! so a type without cached fields pays one constant branch. While the borrow
//! lasts, a read can't fill the entry again: the getter's shared borrow
//! raises `miniextendr_borrow_conflict`. The finalizer of such a type drops
//! the entry before it frees the value ([`super::finalizer`]), so a later
//! value at the same address never finds it.
//!
//! A change no exclusive borrow covers is not seen, and the field reads its
//! old value: interior mutability behind `&self` (a `Cell`, `RefCell`,
//! `Mutex` or atomic in the field, or anything it reaches), and the ALTREP
//! `data1` paths that read and write without a borrow
//! ([`as_mut_unchecked`](super::ExternalPtr::as_mut_unchecked)). The value
//! is stale, not unsound: the cached object is a complete R value, rooted
//! until its entry drops.
//!
//! # Sharing
//!
//! A read returns the cached object itself, so two reads give the same R
//! object. It is marked not mutable (`MARK_NOT_MUTABLE`): R code that
//! modifies it (`v[1] <- 0`, `attr(v, "a") <- 1`) copies it first. C code that
//! writes its argument in place (`data.table::set()`) changes it for every
//! later read, as it would any shared R object.

use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;

use super::{pooled, root_owned, unroot_owned};
use crate::SEXP;
use crate::protect_pool::ProtectKey;
use crate::sys::MARK_NOT_MUTABLE;

thread_local! {
    /// The cached field values of each object that has any, by the address
    /// of its value's `RefCell`: the pool key of each cached field's value,
    /// in cache-index order, `None` until the field's first read.
    static FIELD_CACHES: RefCell<HashMap<usize, Box<[Option<ProtectKey>]>>> =
        RefCell::new(HashMap::new());
}

/// The table key of a stored value: the address of its `RefCell`.
#[inline]
pub(super) fn cell_key<T>(cell: &RefCell<T>) -> usize {
    std::ptr::from_ref(cell).addr()
}

/// The table key of the value an `ExternalPtr`'s address holds: the
/// `RefCell<T>` its `Box<dyn Any>` points to, so the same as [`cell_key`] of
/// that cell.
#[inline]
pub(super) fn stored_key(stored: &dyn Any) -> usize {
    std::ptr::from_ref(stored).cast::<()>().addr()
}

/// The cached value of the field at `index` of the value at `key`, if a read
/// stored one since the value's last exclusive borrow.
#[inline]
pub(super) fn cached(key: usize, index: usize) -> Option<SEXP> {
    let pool_key =
        FIELD_CACHES.with_borrow(|caches| caches.get(&key)?.get(index).copied().flatten())?;
    pooled(pool_key)
}

/// Keeps `value`, the converted value of the field at `index` of the value
/// at `key`, whose type has `count` cached fields. Marks it not mutable and
/// roots it in the pool, which `value` may need no protection for (the pool
/// keeps it reachable while it grows). On R's main thread.
pub(super) fn store(key: usize, count: usize, index: usize, value: SEXP) {
    debug_assert!(index < count, "cache index {index} out of {count}");
    // SAFETY: on R's main thread (the getter's); `value` is a live object.
    unsafe { MARK_NOT_MUTABLE(value) };
    let pool_key = root_owned(value);
    // No R call runs while the table is borrowed, so no finalizer can reach
    // it meanwhile.
    let replaced = FIELD_CACHES.with_borrow_mut(|caches| {
        let slots = caches
            .entry(key)
            .or_insert_with(|| vec![None; count].into_boxed_slice());
        slots[index].replace(pool_key)
    });
    if let Some(old) = replaced {
        unroot_owned(old);
    }
}

/// Drops the cached field values of the value at `key`, if it has any.
/// Allocates nothing and calls no R code, so a finalizer can call it.
pub(super) fn forget(key: usize) {
    let Some(slots) = FIELD_CACHES.with_borrow_mut(|caches| caches.remove(&key)) else {
        return;
    };
    for &pool_key in slots.iter().flatten() {
        unroot_owned(pool_key);
    }
}
