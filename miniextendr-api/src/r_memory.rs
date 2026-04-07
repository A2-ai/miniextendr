//! Utilities for recovering R SEXPs from raw data pointers.
//!
//! R stores vector data at a fixed offset after the SEXPREC header. Given a
//! pointer into that data region, we can subtract the header size to recover
//! the SEXP — then verify it by reading raw memory fields (type tag, ALTREP
//! bit, and vecsxp.length) without calling any R functions.
//!
//! This is used by:
//! - Arrow integration: zero-copy IntoR when the buffer is R-backed
//! - `Cow<[T]>` IntoR round-trip
//!
//! # Initialization
//!
//! [`init_sexprec_data_offset`] must be called during package init (before any
//! recovery attempts). It measures the offset on a real R vector, so it works
//! across R versions and platforms.
//!
//! # R's VECTOR_SEXPREC layout
//!
//! ```text
//! // From R's Defn.h:
//! typedef struct VECTOR_SEXPREC {
//!     SEXPREC_HEADER;           // sxpinfo(8) + attrib(8) + gengc_next(8) + gengc_prev(8)
//!     struct vecsxp_struct {    // length(8) + truelength(8)
//!         R_xlen_t length;
//!         R_xlen_t truelength;
//!     } vecsxp;
//! } VECTOR_SEXPREC;
//!
//! typedef union { VECTOR_SEXPREC s; double align; } SEXPREC_ALIGN;
//! #define STDVEC_DATAPTR(x) ((void *)(((SEXPREC_ALIGN *)(x)) + 1))
//! ```
//!
//! On 64-bit: `sizeof(VECTOR_SEXPREC)` = 48 bytes, `sizeof(SEXPREC_ALIGN)` = 48.
//! Data starts at `sexp + 48`. All vector types (REALSXP, INTSXP, RAWSXP,
//! STRSXP, VECSXP) use the same `VECTOR_SEXPREC` header.
//!
//! # Why not `#[repr(C)]` mirror struct?
//!
//! A Rust `#[repr(C)]` struct mirroring `VECTOR_SEXPREC` would give a
//! compile-time `size_of` instead of runtime measurement. However:
//! - R's layout can vary by version and compile options (32-bit, padding)
//! - The runtime measurement is one allocation at init — negligible
//! - A `repr(C)` mirror struct doesn't help with the real safety issue:
//!   reading from a speculative pointer. `addr_of!` computes field addresses
//!   without dereferencing, but we still need to `read()` the type tag — and
//!   that read is from potentially invalid memory for non-R pointers.
//!
//! The verification (type tag + ALTREP check + XLENGTH) prevents false
//! positives. Only the type tag requires a raw sxpinfo read; ALTREP and
//! XLENGTH use R's public C API.
//!
//! # Safety of speculative reads
//!
//! The candidate pointer is computed from pointer arithmetic on the input
//! data_ptr. For Rust-owned buffers (not R-backed), this points into
//! arbitrary heap memory. We must be careful about which R functions we
//! call on it:
//!
//! - **`ALTREP(x)`** — safe: just reads `x->sxpinfo.alt` (a single bit).
//! - **`XLENGTH(x)`** on non-ALTREP — safe: reads `STDVEC_LENGTH` (struct
//!   field, no dispatch, no error).
//! - **`LENGTH(x)`** — UNSAFE: wraps XLENGTH with `> INT_MAX` check that
//!   calls `R_BadLongVector()` (throws R error on garbage with large length).
//! - **`DATAPTR_RO(x)`** — UNSAFE on ALTREP: dispatches through class vtable
//!   (bogus function pointers on garbage). On non-ALTREP: `STDVEC_DATAPTR`
//!   which also checks for long vectors.
//!
//! The verification sequence is:
//! 1. Raw sxpinfo type tag (bits 0-4) — no public TYPEOF that's safe on garbage
//! 2. `ALTREP(candidate)` — gates step 3 (rejects ALTREP before XLENGTH dispatch)
//! 3. `XLENGTH(candidate)` — safe for non-ALTREP (STDVEC_LENGTH, no errors)

use std::sync::atomic::{AtomicUsize, Ordering};

use crate::ffi::{self, SEXP, SEXPTYPE, SexpExt};

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
/// verifies it:
/// 1. The SEXP type tag (bits 0-4 of sxpinfo) matches `expected_type`
/// 2. `ALTREP(candidate)` is false (only non-ALTREP vectors have fixed-offset data)
/// 3. `XLENGTH(candidate)` matches `expected_len` (safe for non-ALTREP)
///
/// Returns `None` if:
/// - The offset hasn't been initialized yet
/// - The pointer doesn't come from an R vector
/// - The candidate SEXP has the wrong type or length
/// - The candidate is an ALTREP vector (data not at fixed offset from SEXP)
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

    // Zero-length vectors can't be recovered (R uses sentinel pointer 0x1,
    // and empty Arrow buffers use dangling pointers).
    if expected_len == 0 {
        return None;
    }

    let data_addr = data_ptr as usize;

    // Reject pointers that would wraparound or are in invalid ranges.
    // R's sentinel for empty vectors is 0x1; wrapping_byte_sub on small
    // addresses produces huge values (top of address space) → segfault.
    // The 4096 threshold also guards against speculative reads near page
    // boundaries — for non-R pointers (e.g. Rust-allocated Arrow buffers),
    // subtracting the offset could land before the start of mapped memory.
    if data_addr < offset.saturating_add(4096) {
        return None;
    }

    // Compute candidate SEXP by subtracting header size.
    // wrapping_byte_sub is defined behavior for all pointer arithmetic.
    let candidate_ptr = (data_ptr as *mut ffi::SEXPREC).wrapping_byte_sub(offset);

    let candidate = SEXP(candidate_ptr);

    // Quick check: type tag (bits 0-4 of sxpinfo, which is the first field).
    // For Rust-allocated buffers this reads arbitrary heap memory, but
    // wrapping_sub ensures the pointer arithmetic itself is defined.
    // The read is a plain u32 load from mapped heap.
    // No public R API reads TYPEOF without side effects on invalid pointers,
    // so a raw sxpinfo read is unavoidable here.
    let sxpinfo_bits = unsafe { *(candidate.0 as *const u32) };
    let type_bits = sxpinfo_bits & 0x1f;
    if type_bits != expected_type as u32 {
        return None;
    }

    // Reject ALTREP via R's public API (SexpExt::is_altrep → ALTREP(x)).
    // ALTREP vectors store data via indirection — can't recover them.
    // This also gates the xlength() call below: for non-ALTREP, XLENGTH is
    // just STDVEC_LENGTH (a struct field read, no dispatch, no error).
    // For ALTREP, XLENGTH dispatches through the class vtable which would
    // crash on a garbage pointer.
    if candidate.is_altrep() {
        return None;
    }

    // For non-ALTREP, xlength() → Rf_xlength → STDVEC_LENGTH — a direct
    // struct field read with no dispatch and no "long vectors not supported"
    // error. (LENGTH() wraps XLENGTH with an INT_MAX check that can
    // R_BadLongVector; XLENGTH itself never errors for non-ALTREP vectors.)
    if candidate.len() != expected_len {
        return None;
    }

    // No DATAPTR_RO round-trip check needed: for non-ALTREP vectors,
    // STDVEC_DATAPTR(x) == (char*)x + SEXPREC_DATA_OFFSET, and we
    // constructed candidate = data_ptr - offset, so the round-trip
    // is tautologically true. The type + ALTREP + length checks above
    // are the actual discriminators.

    Some(candidate)
}
