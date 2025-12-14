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
use std::mem;
use std::ptr;

/// Wrapper that stores the protection tag alongside user data.
///
/// Using `repr(C)` ensures stable layout where `tag` is at offset 0
/// and `data` follows with proper padding for its alignment.
#[repr(C)]
struct WithProtectionTag<T: ?Sized> {
    tag: SEXP,
    data: T,
}

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

            // For repr(C), data field offset = (sizeof(tag) + align(data) - 1) & ~(align(data) - 1)
            let tag_size = mem::size_of::<SEXP>();
            let data_offset = (tag_size + layout.align() - 1) & !(layout.align() - 1);
            let total_size = data_offset + layout.size();

            // Allocate RAWSXP to hold the whole thing
            let sexp = Rf_allocVector(SEXPTYPE::RAWSXP, total_size as isize);
            if sexp.is_null() {
                return ptr::null_mut();
            }

            // Protect the RAWSXP from GC
            let protection_tag = insert(sexp);

            // Cast RAWSXP payload to our wrapper type
            let wrapper = RAW(sexp) as *mut WithProtectionTag<[u8; 0]>;

            // Write the tag field
            ptr::addr_of_mut!((*wrapper).tag).write(protection_tag);

            // Return pointer to the data field
            let data_ptr = (wrapper as *mut u8).add(data_offset);

            debug_assert_eq!(data_ptr as usize % layout.align(), 0);

            data_ptr
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: alloc::Layout) {
        unsafe {
            if ptr.is_null() {
                return;
            }

            // Calculate where wrapper starts (same math as alloc)
            let tag_size = mem::size_of::<SEXP>();
            let data_offset = (tag_size + layout.align() - 1) & !(layout.align() - 1);

            let wrapper = ptr.sub(data_offset) as *mut WithProtectionTag<[u8; 0]>;

            // Read the tag field
            let protection_tag = ptr::addr_of!((*wrapper).tag).read();

            // Release from preserve list
            release(protection_tag);
        }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: alloc::Layout, new_size: usize) -> *mut u8 {
        unsafe {
            if ptr.is_null() {
                // Equivalent to alloc
                return self.alloc(alloc::Layout::from_size_align_unchecked(new_size, layout.align()));
            }

            if new_size == 0 {
                // Equivalent to dealloc + return null
                self.dealloc(ptr, layout);
                return ptr::null_mut();
            }

            // Calculate where the wrapper starts
            let tag_size = mem::size_of::<SEXP>();
            let data_offset = (tag_size + layout.align() - 1) & !(layout.align() - 1);
            let wrapper = ptr.sub(data_offset) as *mut WithProtectionTag<[u8; 0]>;

            // Read the protection tag to get the RAWSXP
            let protection_tag = ptr::addr_of!((*wrapper).tag).read();

            // Get the SEXP from the protection tag
            // The tag is a cons cell, the actual RAWSXP is stored in its TAG
            let sexp = crate::ffi::TAG(protection_tag);

            // Calculate old RAWSXP capacity
            let old_rawsxp_size = crate::ffi::Rf_xlength(sexp) as usize;
            let new_data_offset = (tag_size + layout.align() - 1) & !(layout.align() - 1);
            let new_total_needed = new_data_offset + new_size;

            // Optimization: if new size fits in old RAWSXP, just return same pointer
            if new_total_needed <= old_rawsxp_size {
                return ptr;
            }

            // Need to allocate new RAWSXP
            let new_layout = alloc::Layout::from_size_align_unchecked(new_size, layout.align());
            let new_ptr = self.alloc(new_layout);

            if new_ptr.is_null() {
                return ptr::null_mut();
            }

            // Copy data from old to new (min of old and new sizes)
            let copy_size = layout.size().min(new_size);
            ptr::copy_nonoverlapping(ptr, new_ptr, copy_size);

            // Deallocate old
            self.dealloc(ptr, layout);

            new_ptr
        }
    }
}

// Tests for this module require R runtime and should be run via R CMD check.
// They are located in rpkg/tests/ as integration tests.
