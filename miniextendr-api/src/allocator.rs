//! R-backed global allocator for Rust.
//!
//! Allocations are backed by R RAWSXP objects and protected from GC via
//! `R_PreserveObject`/`R_ReleaseObject` (R's precious list).
//!
//! # Protection Strategy
//!
//! This allocator uses `R_PreserveObject` directly because:
//! - Allocations may need to survive across multiple `.Call` invocations
//! - The SEXP (RAWSXP) is its own handle — zero Rust-side bookkeeping
//! - LIFO release pattern (recently allocated = first freed) means O(1) release
//!   in practice (R's precious list scans from head)
//!
//! See the [crate-level documentation](crate#gc-protection-strategies) for an
//! overview of miniextendr's protection mechanisms.
//!
//! # Layout
//!
//! Layout inside the RAWSXP (bytes):
//!   \[optional leading pad\]\[Header\]\[user bytes...\]
//!
//! We always return a pointer aligned to at least:
//!   `max(requested_align, align_of::<Header>())`
//! so the `Header` placed immediately before the user pointer is always aligned.
//!
//! # ⚠️ Warning: longjmp Risk
//!
//! R's `Rf_allocVector` can longjmp on allocation failure instead of returning
//! NULL. If this happens, Rust destructors will NOT run, potentially causing:
//! - Resource leaks (files, locks, etc.)
//! - Corrupted state if allocation happens mid-operation
//!
//! This allocator is best suited for:
//! - Short-lived operations within a single R API call
//! - Contexts where `R_UnwindProtect` is active (e.g., inside a `#[miniextendr]` body)
//!
//! For long-lived allocations or critical cleanup requirements, consider using
//! Rust's standard allocator instead.

use crate::sys::{R_PreserveObject_unchecked, R_ReleaseObject_unchecked};
use crate::{SEXP, SEXPTYPE, SexpExt, with_r_thread};
use core::{
    alloc::{GlobalAlloc, Layout},
    mem, ptr,
};

// region: Header and constants

/// Metadata stored immediately before the returned user pointer.
#[repr(C)]
#[derive(Copy, Clone)]
struct Header {
    /// The RAWSXP itself, preserved via R_PreserveObject.
    sexp: SEXP,
}

const HEADER_SIZE: usize = mem::size_of::<Header>();
const HEADER_ALIGN: usize = mem::align_of::<Header>();
// endregion

// region: RAllocator

/// R-backed global allocator.
///
/// All allocations are backed by R RAWSXP objects and protected from
/// garbage collection. The allocator stores metadata before the returned
/// pointer to enable proper deallocation.
///
/// **Note:** This should NOT be used as `#[global_allocator]` in R package
/// library crates, as it would be invoked during compilation/build time when
/// R isn't available. Instead, use it explicitly in standalone binaries that
/// embed R, or use arena-style allocation APIs.
///
/// # Thread Safety
///
/// This allocator is usable on R's main thread only: every call goes through
/// [`with_r_thread`], which panics on spawned or Rayon threads.
#[derive(Debug)]
pub struct RAllocator;

unsafe impl GlobalAlloc for RAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        with_r_thread(|| unsafe { alloc_main_thread(layout) })
    }

    unsafe fn dealloc(&self, data: *mut u8, _layout: Layout) {
        if data.is_null() {
            return;
        }
        with_r_thread(|| unsafe { dealloc_main_thread(data) });
    }

    unsafe fn realloc(&self, old: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // Handle null input (acts like alloc)
        if old.is_null() {
            let Ok(new_layout) = Layout::from_size_align(new_size, layout.align()) else {
                return ptr::null_mut();
            };
            return unsafe { self.alloc(new_layout) };
        }

        // Handle zero size (acts like dealloc)
        if new_size == 0 {
            unsafe { self.dealloc(old, layout) };
            return ptr::null_mut();
        }

        with_r_thread(|| unsafe {
            realloc_main_thread(old, layout.size(), layout.align(), new_size)
        })
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let p = unsafe { self.alloc(layout) };
        if !p.is_null() {
            unsafe { ptr::write_bytes(p, 0, layout.size()) };
        }
        p
    }
}
// endregion

// region: Main-thread helpers

/// Allocate memory on the R main thread.
///
/// # Safety
///
/// Must be called from R's main thread.
unsafe fn alloc_main_thread(layout: Layout) -> *mut u8 {
    // ZST allocations: return null since we can't meaningfully track them
    // (dangling pointer would crash in dealloc when we try to read the header)
    if layout.size() == 0 {
        return ptr::null_mut();
    }

    let align = layout.align().max(HEADER_ALIGN);

    // Calculate total size needed with overflow checking
    let total = {
        let Some(align_minus_1) = align.checked_sub(1) else {
            return ptr::null_mut();
        };
        let Some(temp) = HEADER_SIZE.checked_add(align_minus_1) else {
            return ptr::null_mut();
        };
        let Some(total) = temp.checked_add(layout.size()) else {
            return ptr::null_mut();
        };
        total
    };

    let total_isize: isize = match total.try_into() {
        Ok(n) => n,
        Err(_) => return ptr::null_mut(),
    };

    // NOTE: Rf_allocVector can longjmp on failure instead of returning NULL.
    // Inside a `#[miniextendr]` body, R_UnwindProtect catches it. Outside of
    // that context, Rust destructors may be skipped.
    // Use _unchecked since `with_r_thread` checked for R's main thread.
    // RAWSXP literal is the source of truth (#882): this is an opaque R-managed
    // byte arena used as a `*mut u8` allocation, not a typed vector — there is no
    // `T: RNativeType` element to derive the tag from.
    let sexp = unsafe { crate::sys::Rf_allocVector_unchecked(SEXPTYPE::RAWSXP, total_isize) };
    if sexp.is_null_ptr() {
        return ptr::null_mut();
    }

    // Protect from GC (must stay valid until dealloc()).
    // Uses R_PreserveObject — LIFO means recently allocated objects are found fast on release.
    unsafe { R_PreserveObject_unchecked(sexp) };

    // Use _unchecked since we're guaranteed to be on R main thread.
    let raw_base = unsafe { crate::sys::RAW_unchecked(sexp) }.cast::<u8>();

    // Calculate header and data pointers with alignment
    let after_header = unsafe { raw_base.add(HEADER_SIZE) };
    let pad = after_header.align_offset(align);
    if pad == usize::MAX {
        // Alignment failed (extremely unlikely)
        unsafe { R_ReleaseObject_unchecked(sexp) };
        return ptr::null_mut();
    }

    let data = unsafe { after_header.add(pad) };
    let header = unsafe { data.sub(HEADER_SIZE) }.cast::<Header>();

    unsafe { header.write(Header { sexp }) };

    debug_assert_eq!(data.align_offset(layout.align()), 0);
    data
}

/// Deallocate memory on the R main thread.
///
/// # Safety
///
/// Must be called from R's main thread.
/// The pointer must have been allocated by this allocator.
unsafe fn dealloc_main_thread(data: *mut u8) {
    let header = unsafe { data.sub(HEADER_SIZE) }.cast::<Header>();
    let sexp = unsafe { (*header).sexp };
    unsafe { R_ReleaseObject_unchecked(sexp) };
}

/// Reallocate memory on the R main thread.
///
/// # Safety
///
/// Must be called from R's main thread.
/// The old pointer must have been allocated by this allocator.
unsafe fn realloc_main_thread(
    old: *mut u8,
    old_size: usize,
    align: usize,
    new_size: usize,
) -> *mut u8 {
    // Recover RAWSXP from header (stored directly, no DLL cell indirection).
    // Use _unchecked since `with_r_thread` checked for R's main thread.
    let header = unsafe { old.sub(HEADER_SIZE) }.cast::<Header>();
    let sexp = unsafe { (*header).sexp };

    // Check if existing allocation has capacity
    let raw_base = unsafe { crate::sys::RAW_unchecked(sexp) }.cast::<u8>();
    let cap: usize = match unsafe { sexp.xlength_unchecked() }.try_into() {
        Ok(n) => n,
        Err(_) => return ptr::null_mut(),
    };

    let used = unsafe { old.cast_const().offset_from(raw_base.cast_const()) };
    let Ok(used_usize) = usize::try_from(used) else {
        // Should be impossible if `old` came from this allocator, but don't UB.
        return ptr::null_mut();
    };
    let available = cap.saturating_sub(used_usize);

    if new_size <= available {
        return old; // Reuse existing allocation
    }

    // Need new allocation
    let Ok(new_layout) = Layout::from_size_align(new_size, align) else {
        return ptr::null_mut();
    };

    let new_ptr = unsafe { alloc_main_thread(new_layout) };
    if new_ptr.is_null() {
        // On realloc failure, the old allocation must remain valid.
        return ptr::null_mut();
    }

    unsafe { ptr::copy_nonoverlapping(old, new_ptr, old_size.min(new_size)) };
    unsafe { R_ReleaseObject_unchecked(sexp) }; // Free old allocation

    new_ptr
}
// endregion
