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

/// R-backed global allocator.
///
/// All allocations are backed by R RAWSXP objects and protected from
/// garbage collection. The allocator stores metadata before the returned
/// pointer to enable proper deallocation.
///
/// # Example
///
/// ```ignore
/// #[global_allocator]
/// static R_ALLOCATOR: RAllocator = RAllocator;
/// ```
///
/// # Safety
///
/// This allocator is ONLY safe to use from the R main thread. Using it
/// from other threads will cause undefined behavior because:
/// - `Rf_allocVector` must be called from the main thread
/// - The preserve mechanism is thread-local
///
/// If you need allocations from worker threads, use the standard system
/// allocator on those threads.
#[derive(Debug)]
pub struct RAllocator;

unsafe impl alloc::GlobalAlloc for RAllocator {
    unsafe fn alloc(&self, layout: alloc::Layout) -> *mut u8 {
        unsafe {
            let size = layout.size();
            let align = layout.align();

            // Zero-sized allocations return null (per GlobalAlloc contract)
            if size == 0 {
                return ptr::null_mut();
            }

            // Calculate layout for RBox header + user data
            // The RBox struct has:
            // - cell: SEXP (8 bytes on 64-bit, 4 bytes on 32-bit)
            // - data: [u8; size] with alignment `align`
            let cell_size = std::mem::size_of::<SEXP>();

            // Calculate padding needed after cell to achieve user alignment
            let padding = (align - (cell_size % align)) % align;
            let total_size = cell_size + padding + size;

            // Allocate RAWSXP
            let sexp = Rf_allocVector(SEXPTYPE::RAWSXP, total_size as isize);
            if sexp.is_null() {
                return ptr::null_mut();
            }

            // Protect from GC
            let cell = insert(sexp);

            // Get pointer to RAWSXP payload
            let raw_ptr = RAW(sexp);

            // Write the cell at the start
            ptr::write(raw_ptr as *mut SEXP, cell);

            // Return pointer to data (after cell + padding)
            let data_offset = cell_size + padding;
            let data_ptr = raw_ptr.add(data_offset);

            // Verify alignment
            debug_assert_eq!(
                data_ptr as usize % align,
                0,
                "data pointer not properly aligned"
            );

            data_ptr
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: alloc::Layout) {
        unsafe {
            if ptr.is_null() {
                return;
            }

            let align = layout.align();
            let cell_size = std::mem::size_of::<SEXP>();

            // Calculate padding (same as in alloc)
            let padding = (align - (cell_size % align)) % align;
            let data_offset = cell_size + padding;

            // Recover the cell pointer
            let raw_ptr = ptr.sub(data_offset);
            let cell = ptr::read(raw_ptr as *const SEXP);

            // Release from preserve list
            release(cell);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::alloc::{GlobalAlloc, Layout};

    #[test]
    fn test_alloc_dealloc() {
        unsafe {
            // Allocate 64 bytes with 8-byte alignment
            let layout = Layout::from_size_align(64, 8).unwrap();
            let ptr = RAllocator.alloc(layout);
            assert!(!ptr.is_null());
            assert_eq!(ptr as usize % 8, 0, "pointer should be 8-byte aligned");

            // Write some data
            std::ptr::write_bytes(ptr, 0x42, 64);

            // Deallocate
            RAllocator.dealloc(ptr, layout);
        }
    }

    #[test]
    fn test_zero_size_alloc() {
        unsafe {
            let layout = Layout::from_size_align(0, 1).unwrap();
            let ptr = RAllocator.alloc(layout);
            assert!(ptr.is_null(), "zero-size alloc should return null");
        }
    }

    #[test]
    fn test_various_alignments() {
        unsafe {
            for align in [1, 2, 4, 8, 16, 32, 64] {
                let layout = Layout::from_size_align(32, align).unwrap();
                let ptr = RAllocator.alloc(layout);
                assert!(!ptr.is_null());
                assert_eq!(
                    ptr as usize % align,
                    0,
                    "pointer should be {}-byte aligned",
                    align
                );
                RAllocator.dealloc(ptr, layout);
            }
        }
    }

    #[test]
    fn test_multiple_allocations() {
        unsafe {
            let layout = Layout::from_size_align(16, 8).unwrap();

            let mut ptrs = Vec::new();
            for _ in 0..10 {
                let ptr = RAllocator.alloc(layout);
                assert!(!ptr.is_null());
                ptrs.push(ptr);
            }

            // Deallocate in reverse order
            for ptr in ptrs.into_iter().rev() {
                RAllocator.dealloc(ptr, layout);
            }
        }
    }
}
