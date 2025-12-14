//! R-backed global allocator for Rust.
//!
//! This module provides a `GlobalAlloc` implementation that uses R's memory
//! management system to allocate memory. All allocations are backed by R RAWSXP
//! objects and protected from garbage collection.
//!
//! ## How it works
//!
//! 1. When `alloc()` is called, we allocate a RAWSXP large enough for:
//!    - A header (preserve cell + offset)
//!    - Alignment padding
//!    - The requested user data
//!
//! 2. The RAWSXP is protected using our preserve mechanism
//!
//! 3. We return an aligned pointer into the RAWSXP's data, with the header
//!    stored immediately before it
//!
//! 4. On `dealloc()`, we recover the header and release the preserve cell
//!
//! ## Memory layout
//!
//! Each allocation is wrapped in an `RBox<T>` structure that contains:
//! - The preserve cell (for GC protection)
//! - The actual data (properly aligned via `repr(C)`)
//!
//! The RAWSXP is large enough to hold the entire `RBox<T>`.
//!
//! ## Safety
//!
//! This allocator must only be used from the R main thread. Using it from
//! other threads will cause undefined behavior.

use crate::ffi::{RAW, Rf_allocVector, SEXP, SEXPTYPE};
use crate::preserve::{insert, release};
use std::alloc;
use std::ptr;

/// Header stored before user data in each allocation.
///
/// The header contains the protection tag and the offset from the RAWSXP
/// base to the user data pointer. We need the offset because RAW(sexp)
/// is not guaranteed to be aligned, so we can't reliably recalculate it.
#[repr(C)]
struct AllocationHeader {
    tag: SEXP,
    offset: u16, // Offset from RAW(sexp) to user data pointer
}

const HEADER_SIZE: usize = std::mem::size_of::<AllocationHeader>();

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
/// # Safety
///
/// This allocator is ONLY safe to use from the R main thread. Using it
/// from other threads will cause undefined behavior because:
/// - `Rf_allocVector` must be called from the main thread
/// - The preserve mechanism is thread-local
#[derive(Debug)]
pub struct RAllocator;

unsafe impl alloc::GlobalAlloc for RAllocator {
    unsafe fn alloc(&self, layout: alloc::Layout) -> *mut u8 {
        unsafe {
            if layout.size() == 0 {
                return ptr::null_mut();
            }

            let align = layout.align();
            let size = layout.size();

            // We need space for: header + padding + data
            // Worst case: HEADER_SIZE + (align - 1) + size
            let total_size = HEADER_SIZE + align - 1 + size;

            // Allocate RAWSXP
            let sexp = Rf_allocVector(SEXPTYPE::RAWSXP, total_size as isize);
            if sexp.is_null() {
                return ptr::null_mut();
            }

            // Protect from GC
            let protection_tag = insert(sexp);

            // Get raw base pointer
            let raw_base = RAW(sexp);

            // Find where header should go: immediately before aligned data
            // We want data at an address aligned to `align`
            let header_end = raw_base.add(HEADER_SIZE);

            // Align header_end up to the requested alignment
            let data_ptr = header_end.map_addr(|addr| (addr + align - 1) & !(align - 1));

            // Header goes immediately before data
            let header_ptr = data_ptr.sub(HEADER_SIZE).cast::<AllocationHeader>();

            // Write header
            ptr::write(
                header_ptr,
                AllocationHeader {
                    tag: protection_tag,
                    offset: data_ptr.offset_from(raw_base) as u16,
                },
            );

            // Verify alignment (without usize cast)
            debug_assert_eq!(data_ptr.align_offset(align), 0);

            data_ptr
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: alloc::Layout) {
        unsafe {
            if ptr.is_null() {
                return;
            }

            // Header is immediately before data
            let header_ptr = ptr.sub(HEADER_SIZE) as *const AllocationHeader;
            let header = ptr::read(header_ptr);

            // Release from preserve list
            release(header.tag);
        }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: alloc::Layout, new_size: usize) -> *mut u8 {
        unsafe {
            if ptr.is_null() {
                return self.alloc(alloc::Layout::from_size_align_unchecked(
                    new_size,
                    layout.align(),
                ));
            }

            if new_size == 0 {
                self.dealloc(ptr, layout);
                return ptr::null_mut();
            }

            // Read header to get RAWSXP
            let header_ptr = ptr.sub(HEADER_SIZE) as *const AllocationHeader;
            let header = ptr::read(header_ptr);
            let sexp = crate::ffi::TAG(header.tag);

            // Check if new size fits in existing RAWSXP
            let old_rawsxp_size = crate::ffi::Rf_xlength(sexp);
            let align = layout.align();
            let new_total_needed = HEADER_SIZE + align - 1 + new_size;

            if new_total_needed as isize <= old_rawsxp_size {
                // Fits! Return same pointer
                return ptr;
            }

            // Need new allocation
            let new_layout = alloc::Layout::from_size_align_unchecked(new_size, align);
            let new_ptr = self.alloc(new_layout);

            if new_ptr.is_null() {
                return ptr::null_mut();
            }

            // Copy data
            let copy_size = layout.size().min(new_size);
            ptr::copy_nonoverlapping(ptr, new_ptr, copy_size);

            // Free old
            self.dealloc(ptr, layout);

            new_ptr
        }
    }
}

// Tests for this module require R runtime and should be run via R CMD check.
// They are located in rpkg/tests/ as integration tests.
