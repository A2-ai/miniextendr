//! Utilities for recovering R SEXPs from raw data pointers.
//!
//! R stores vector data at a fixed offset after the SEXPREC header. Given a
//! pointer into that data region, we can subtract the header size to recover
//! the SEXP — then verify it by checking the type tag, length, and round-trip
//! through `DATAPTR_RO`.
//!
//! This is used by:
//! - Arrow integration: zero-copy IntoR when the buffer is R-backed
//! - Future: Cow<[T]> round-trip, allocator validation
//!
//! # Initialization
//!
//! [`init_sexprec_data_offset`] must be called during package init (before any
//! recovery attempts). It measures the offset on a real R vector, so it works
//! across R versions and platforms.

use std::sync::atomic::{AtomicUsize, Ordering};

use crate::ffi::{self, SEXP, SEXPTYPE};

/// Offset in bytes from SEXP address to data pointer for standard (non-ALTREP) vectors.
///
/// `DATAPTR_RO(sexp) == (sexp as *const u8).add(SEXPREC_DATA_OFFSET)`
///
/// Zero means not yet initialized.
static SEXPREC_DATA_OFFSET: AtomicUsize = AtomicUsize::new(0);

/// Get the computed SEXPREC data offset.
///
/// Returns 0 if not yet initialized.
#[inline]
pub fn sexprec_data_offset() -> usize {
    SEXPREC_DATA_OFFSET.load(Ordering::Relaxed)
}

/// Compute and store the SEXPREC data offset by measuring a real R vector.
///
/// Must be called from R's main thread during package init.
///
/// # Safety
///
/// Must be called on R's main thread with R initialized.
pub unsafe fn init_sexprec_data_offset() {
    unsafe {
        let test = ffi::Rf_protect(ffi::Rf_allocVector(SEXPTYPE::REALSXP, 1));
        let sexp_addr = test.0 as usize;
        let data_addr = ffi::DATAPTR_RO(test) as usize;
        SEXPREC_DATA_OFFSET.store(data_addr - sexp_addr, Ordering::Relaxed);
        ffi::Rf_unprotect(1);
    }
}

/// Try to recover the source R SEXP from a data pointer.
///
/// Given a pointer that may point into an R vector's data area, this
/// subtracts the known SEXPREC header size to get a candidate SEXP, then
/// verifies it by checking:
/// 1. The SEXP type tag (bits 0-4 of sxpinfo) matches `expected_type`
/// 2. The vector length matches `expected_len`
/// 3. `DATAPTR_RO(candidate)` round-trips to the original pointer
///
/// Returns `None` if:
/// - The offset hasn't been initialized yet
/// - The pointer doesn't come from an R vector
/// - The candidate SEXP has the wrong type or length
/// - ALTREP vectors (data not at fixed offset from SEXP)
///
/// # Safety
///
/// Must be called on R's main thread. The data pointer must be valid
/// (i.e., it must point to readable memory for at least `expected_len`
/// elements, which is guaranteed if it came from an Arrow buffer).
pub unsafe fn try_recover_r_sexp(
    data_ptr: *const u8,
    expected_type: SEXPTYPE,
    expected_len: usize,
) -> Option<SEXP> {
    let offset = SEXPREC_DATA_OFFSET.load(Ordering::Relaxed);
    if offset == 0 {
        return None;
    }

    // Compute candidate SEXP by subtracting header size.
    // Use wrapping_sub: defined behavior for all pointer arithmetic (no
    // requirement that the result be within the same allocation, unlike sub).
    let candidate_ptr = (data_ptr as *mut ffi::SEXPREC).wrapping_byte_sub(offset);

    // Reject obviously invalid pointers (null, low addresses from sentinel 0x1)
    if (candidate_ptr as usize) < 4096 {
        return None;
    }

    let candidate = SEXP(candidate_ptr);

    // Quick check: type tag (bits 0-4 of sxpinfo, which is the first field).
    // For Rust-allocated buffers this reads arbitrary heap memory, but
    // wrapping_sub ensures the pointer arithmetic itself is defined.
    // The read is a plain u32 load from mapped heap — no UB from the
    // pointer derivation (wrapping arithmetic doesn't create provenance).
    let sxpinfo_bits = unsafe { *(candidate.0 as *const u32) };
    let type_bits = sxpinfo_bits & 0x1f;
    if type_bits != expected_type as u32 {
        return None;
    }

    // Length check — uses R's LENGTH which is safe for valid SEXPs of the
    // correct type (we verified the type tag above).
    let candidate_len: usize = match unsafe { ffi::LENGTH(candidate) }.try_into() {
        Ok(len) => len,
        Err(_) => return None,
    };
    if candidate_len != expected_len {
        return None;
    }

    // Final verification: DATAPTR_RO round-trip.
    // This catches ALTREP vectors (where data isn't at fixed offset) and any
    // false positives from the type/length checks.
    let actual_ptr = unsafe { ffi::DATAPTR_RO(candidate) } as *const u8;
    if actual_ptr != data_ptr {
        return None;
    }

    Some(candidate)
}
