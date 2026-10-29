//! Registry of Arrow buffers that are R memory (provenance for zero-copy
//! Arrow → R recovery).
//!
//! An Arrow → R conversion hands back the source vector instead of copying
//! only on an exact hit, so the answer never depends on reading memory the
//! buffer does not own. An earlier version guessed instead: it read the bytes
//! in front of the data pointer as an R vector header and accepted them when
//! the type, ALTREP bit and length looked right, behind a gate on the buffer's
//! shape (unsliced, capacity exactly the data length). The gate assumed that
//! buffers Arrow allocates itself have capacity rounded up to 64 bytes, but a
//! `Vec`-backed buffer has its exact length as capacity. DataFusion builds a
//! one-row aggregate column as `PrimitiveArray::from_value(v, 1)`, a
//! `vec![v; 1]`, so every such conversion read heap bytes in front of the
//! `Vec`, and when they looked like a header the data.frame got a SEXP
//! pointing into Rust memory freed with the batch.
//!
//! Only standard (non-ALTREP) vectors are registered. A standard vector's
//! data lies inside its own allocation, so while it is preserved no other
//! live vector has that data address, and a key names exactly one SEXP. An
//! ALTREP vector's data pointer can be another object's memory: the vector R's
//! wrapper class wraps, the expansion a compact sequence materializes, or the
//! Rust buffer behind one of this crate's Arrow ALTREP classes. Two such
//! vectors can then share a key, and an entry kept for the first would outlive
//! its guard while the second's guard is still counted. A buffer over an
//! ALTREP vector is therefore always copied on the way back, as it was when
//! recovery read the header.
//!
//! The map itself is plain Rust ([`RBackedBuffers`]): it stores addresses and
//! type tags and never reads R memory, so its unit tests run without R (and
//! under Miri, see `.github/workflows/miri-nightly.yml`).

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex, MutexGuard, PoisonError};

use crate::{RNativeType, SEXP, SEXPREC, SEXPTYPE, SexpExt};

/// (data address, byte length) of a buffer.
pub(super) type BufferKey = (usize, usize);

/// A registered vector: its address, its type, and how many live guards
/// cover it (the same SEXP converted to Arrow more than once).
struct Entry {
    sexp: usize,
    sexptype: SEXPTYPE,
    guards: usize,
}

/// (data address, byte length) of every buffer `sexp_to_arrow_buffer` made
/// over a standard R vector → that vector.
#[derive(Default)]
struct RBackedBuffers {
    entries: HashMap<BufferKey, Entry>,
}

impl RBackedBuffers {
    /// Record one more guard over the vector at `sexp`, whose data the buffer
    /// `key` covers.
    fn remember(&mut self, key: BufferKey, sexp: usize, sexptype: SEXPTYPE) {
        self.entries
            .entry(key)
            .and_modify(|entry| {
                debug_assert_eq!(entry.sexp, sexp, "two live vectors share a data address");
                entry.guards += 1;
            })
            .or_insert(Entry {
                sexp,
                sexptype,
                guards: 1,
            });
    }

    /// Give up one guard's entry for `key`.
    fn forget(&mut self, key: BufferKey) {
        if let Some(entry) = self.entries.get_mut(&key) {
            entry.guards -= 1;
            if entry.guards == 0 {
                self.entries.remove(&key);
            }
        }
    }

    /// The address of the vector whose data is exactly `len` elements of `T`
    /// starting at `buffer`'s data pointer, if one is registered.
    fn lookup<T: RNativeType>(&self, buffer: &arrow_buffer::Buffer, len: usize) -> Option<usize> {
        let byte_len = len.checked_mul(size_of::<T>())?;
        if byte_len == 0 {
            return None;
        }
        let entry = self.entries.get(&(buffer.as_ptr() as usize, byte_len))?;
        (entry.sexptype == T::SEXP_TYPE).then_some(entry.sexp)
    }
}

/// Guards drop wherever Arrow drops the buffer, possibly off R's main thread,
/// hence the lock. Addresses are stored as `usize` so the map is `Send`.
static R_BACKED_BUFFERS: LazyLock<Mutex<RBackedBuffers>> = LazyLock::new(Default::default);

fn registry() -> MutexGuard<'static, RBackedBuffers> {
    // Every update is a single insert, decrement or remove, so a lock poisoned
    // by a panic elsewhere still guards a consistent map.
    R_BACKED_BUFFERS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
}

/// Record a new guard over the standard vector `sexp`, whose data the buffer
/// `key` covers.
///
/// Main thread only (reads the type of the live vector).
pub(super) fn remember_r_backed_buffer(key: BufferKey, sexp: SEXP) {
    let sexptype = sexp.type_of();
    registry().remember(key, sexp.0 as usize, sexptype);
}

/// Give up one guard's entry for `key`. Any thread.
pub(super) fn forget_r_backed_buffer(key: BufferKey) {
    registry().forget(key);
}

/// The registered R vector whose data is exactly `len` elements of `T` at
/// `buffer`'s data pointer.
///
/// A hit is live: the buffer being converted points into its data, which a
/// standard vector keeps inside its own allocation, so whatever keeps the
/// buffer valid keeps the preserved vector alive. Main thread only, since
/// the result goes to R.
pub(super) fn r_backed_buffer_sexp<T: RNativeType>(
    buffer: &arrow_buffer::Buffer,
    len: usize,
) -> Option<SEXP> {
    registry()
        .lookup::<T>(buffer, len)
        .map(|sexp| SEXP(sexp as *mut SEXPREC))
}

#[cfg(test)]
mod tests;
