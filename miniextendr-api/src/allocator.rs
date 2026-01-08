//! R-backed global allocator for Rust.
//!
//! Allocations are backed by R RAWSXP objects and protected from GC via the
//! crate's [`preserve`](crate::preserve) mechanism.
//!
//! # Protection Strategy
//!
//! This allocator uses the **preserve list** (not the PROTECT stack) because:
//! - Allocations may need to survive across multiple `.Call` invocations
//! - Deallocations can happen in any order (not LIFO like the PROTECT stack)
//! - The preserve list supports arbitrary-order release
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
//! - Contexts where `R_UnwindProtect` is active (e.g., inside `run_on_worker`)
//!
//! For long-lived allocations or critical cleanup requirements, consider using
//! Rust's standard allocator instead.

use crate::ffi::{SEXP, SEXPTYPE};
use crate::preserve::{insert, release};
use crate::worker::{has_worker_context, is_r_main_thread, with_r_thread};
use core::{
    alloc::{GlobalAlloc, Layout},
    mem, ptr,
};

// ============================================================================
// SendableDataPtr - Thread-safe wrapper for allocator pointers
// ============================================================================

/// Wrapper to make `*mut u8` pointers `Send` for cross-thread routing.
///
/// Unlike `SendablePtr<T>` in externalptr, this allows null pointers
/// since allocator operations can fail and return null.
///
/// # Safety
///
/// Same safety model as `Sendable<T>` and `SendablePtr`:
/// - The pointer value (memory address) is safely transmitted between threads
/// - The pointer is only dereferenced on R's main thread
/// - This is guaranteed by the `with_r_thread_or_inline` routing mechanism
type SendableDataPtr = crate::worker::Sendable<*mut u8>;

#[inline]
const fn sendable_data_ptr_new(ptr: *mut u8) -> SendableDataPtr {
    crate::worker::Sendable(ptr)
}

#[inline]
const fn sendable_data_ptr_get(ptr: SendableDataPtr) -> *mut u8 {
    ptr.0
}

#[inline]
const fn sendable_data_ptr_is_null(ptr: SendableDataPtr) -> bool {
    ptr.0.is_null()
}

#[inline]
const fn sendable_data_ptr_null() -> SendableDataPtr {
    crate::worker::Sendable(ptr::null_mut())
}

// ============================================================================
// Thread routing helper
// ============================================================================

/// Routes a closure to the R main thread if not already there.
///
/// - If on main thread: executes directly
/// - If in worker context: routes via `with_r_thread`
/// - Otherwise: panics (R API calls from arbitrary threads are unsafe)
///
/// # Panics
///
/// Panics if called from a non-main thread without worker context.
/// This prevents unsafe R API calls from arbitrary threads (e.g., Rayon).
#[inline]
fn with_r_thread_or_inline<R: Send + 'static, F: FnOnce() -> R + Send + 'static>(f: F) -> R {
    if is_r_main_thread() {
        f()
    } else if has_worker_context() {
        with_r_thread(f)
    } else {
        panic!(
            "RAllocator: cannot allocate from non-main thread without worker context. \
             Ensure miniextendr_worker_init() was called and you're within run_on_worker()."
        )
    }
}

// ============================================================================
// Header and constants
// ============================================================================

/// Metadata stored immediately before the returned user pointer.
#[repr(C)]
#[derive(Copy, Clone)]
struct Header {
    preserve_tag: SEXP,
}

const HEADER_SIZE: usize = mem::size_of::<Header>();
const HEADER_ALIGN: usize = mem::align_of::<Header>();

// ============================================================================
// RAllocator
// ============================================================================

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
/// This allocator is safe to use from any thread. R API calls are automatically
/// routed to the main thread via `with_r_thread_or_inline`.
#[derive(Debug)]
pub struct RAllocator;

unsafe impl GlobalAlloc for RAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        sendable_data_ptr_get(with_r_thread_or_inline(move || unsafe {
            alloc_main_thread(layout)
        }))
    }

    unsafe fn dealloc(&self, data: *mut u8, _layout: Layout) {
        if data.is_null() {
            return;
        }
        let ptr = sendable_data_ptr_new(data);
        with_r_thread_or_inline(move || unsafe {
            dealloc_main_thread(ptr);
        });
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

        let old_ptr = sendable_data_ptr_new(old);
        let old_size = layout.size();
        let align = layout.align();

        let new_ptr = with_r_thread_or_inline(move || unsafe {
            realloc_main_thread(old_ptr, old_size, align, new_size)
        });
        sendable_data_ptr_get(new_ptr)
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let p = unsafe { self.alloc(layout) };
        if !p.is_null() {
            unsafe { ptr::write_bytes(p, 0, layout.size()) };
        }
        p
    }
}

// ============================================================================
// Main-thread helpers
// ============================================================================

/// Allocate memory on the R main thread.
///
/// # Safety
///
/// Must be called from R's main thread (or routed via `with_r_thread`).
unsafe fn alloc_main_thread(layout: Layout) -> SendableDataPtr {
    // ZST allocations: return null since we can't meaningfully track them
    // (dangling pointer would crash in dealloc when we try to read the header)
    if layout.size() == 0 {
        return sendable_data_ptr_null();
    }

    let align = layout.align().max(HEADER_ALIGN);

    // Calculate total size needed with overflow checking
    let total = {
        let Some(align_minus_1) = align.checked_sub(1) else {
            return sendable_data_ptr_null();
        };
        let Some(temp) = HEADER_SIZE.checked_add(align_minus_1) else {
            return sendable_data_ptr_null();
        };
        let Some(total) = temp.checked_add(layout.size()) else {
            return sendable_data_ptr_null();
        };
        total
    };

    let total_isize: isize = match total.try_into() {
        Ok(n) => n,
        Err(_) => return sendable_data_ptr_null(),
    };

    // NOTE: Rf_allocVector can longjmp on failure instead of returning NULL.
    // If this happens inside run_on_worker, R_UnwindProtect will catch it.
    // Outside of that context, Rust destructors may be skipped.
    // Use _unchecked since we're guaranteed to be on R main thread via with_r_thread_or_inline.
    let sexp = unsafe { crate::ffi::Rf_allocVector_unchecked(SEXPTYPE::RAWSXP, total_isize) };
    if sexp.is_null() {
        return sendable_data_ptr_null();
    }

    // Protect from GC (must stay valid until dealloc()).
    let preserve_tag = unsafe { insert(sexp) };

    // Use _unchecked since we're guaranteed to be on R main thread.
    let raw_base = unsafe { crate::ffi::RAW_unchecked(sexp) }.cast::<u8>();

    // Calculate header and data pointers with alignment
    let after_header = unsafe { raw_base.add(HEADER_SIZE) };
    let pad = after_header.align_offset(align);
    if pad == usize::MAX {
        // Alignment failed (extremely unlikely)
        unsafe { release(preserve_tag) };
        return sendable_data_ptr_null();
    }

    let data = unsafe { after_header.add(pad) };
    let header = unsafe { data.sub(HEADER_SIZE) }.cast::<Header>();

    unsafe { header.write(Header { preserve_tag }) };

    debug_assert_eq!(data.align_offset(layout.align()), 0);
    sendable_data_ptr_new(data)
}

/// Deallocate memory on the R main thread.
///
/// # Safety
///
/// Must be called from R's main thread (or routed via `with_r_thread`).
/// The pointer must have been allocated by this allocator.
unsafe fn dealloc_main_thread(ptr: SendableDataPtr) {
    let data = sendable_data_ptr_get(ptr);
    let header = unsafe { data.sub(HEADER_SIZE) }.cast::<Header>();
    let preserve_tag = unsafe { (*header).preserve_tag };
    unsafe { release(preserve_tag) };
}

/// Reallocate memory on the R main thread.
///
/// # Safety
///
/// Must be called from R's main thread (or routed via `with_r_thread`).
/// The old pointer must have been allocated by this allocator.
unsafe fn realloc_main_thread(
    old_ptr: SendableDataPtr,
    old_size: usize,
    align: usize,
    new_size: usize,
) -> SendableDataPtr {
    let old = sendable_data_ptr_get(old_ptr);

    // Recover RAWSXP via preserve tag
    // Use _unchecked since we're guaranteed to be on R main thread via with_r_thread_or_inline.
    let header = unsafe { old.sub(HEADER_SIZE) }.cast::<Header>();
    let preserve_tag = unsafe { (*header).preserve_tag };
    let sexp = unsafe { crate::ffi::TAG_unchecked(preserve_tag) };

    // Check if existing allocation has capacity
    let raw_base = unsafe { crate::ffi::RAW_unchecked(sexp) }.cast::<u8>();
    let cap: usize = match unsafe { crate::ffi::Rf_xlength_unchecked(sexp) }.try_into() {
        Ok(n) => n,
        Err(_) => return sendable_data_ptr_null(),
    };

    let used = unsafe { (old as *const u8).offset_from(raw_base as *const u8) };
    if used < 0 {
        // Should be impossible if `old` came from this allocator, but don't UB.
        return sendable_data_ptr_null();
    }
    let available = cap.saturating_sub(used as usize);

    if new_size <= available {
        return old_ptr; // Reuse existing allocation
    }

    // Need new allocation
    let Ok(new_layout) = Layout::from_size_align(new_size, align) else {
        return sendable_data_ptr_null();
    };

    let new_ptr = unsafe { alloc_main_thread(new_layout) };
    if sendable_data_ptr_is_null(new_ptr) {
        // On realloc failure, the old allocation must remain valid.
        return sendable_data_ptr_null();
    }

    unsafe {
        ptr::copy_nonoverlapping(old, sendable_data_ptr_get(new_ptr), old_size.min(new_size))
    };
    unsafe { release(preserve_tag) }; // Free old allocation

    new_ptr
}
