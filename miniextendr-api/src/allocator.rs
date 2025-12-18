//! R-backed global allocator for Rust.
//!
//! Allocations are backed by R RAWSXP objects and protected from GC via the
//! crate's preserve mechanism.
//!
//! Layout inside the RAWSXP (bytes):
//!   \[optional leading pad\]\[Header\]\[user bytes...\]
//!
//! We always return a pointer aligned to at least:
//!   max(requested_align, align_of::<Header>())
//! so the `Header` placed immediately before the user pointer is always aligned.

use crate::ffi::{RAW, Rf_allocVector, SEXP, SEXPTYPE};
use crate::preserve::{insert, release};
use core::{
    alloc::{GlobalAlloc, Layout},
    mem, ptr,
};

/// Metadata stored immediately before the returned user pointer.
#[repr(C)]
#[derive(Copy, Clone)]
struct Header {
    preserve_tag: SEXP,
}

const HEADER_SIZE: usize = mem::size_of::<Header>();
const HEADER_ALIGN: usize = mem::align_of::<Header>();

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

unsafe impl GlobalAlloc for RAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe {
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

            let sexp = Rf_allocVector(SEXPTYPE::RAWSXP, total_isize);
            if sexp.is_null() {
                return ptr::null_mut();
            }

            // Protect from GC (must stay valid until dealloc()).
            let preserve_tag = insert(sexp);

            let raw_base = RAW(sexp).cast::<u8>();

            // Calculate header and data pointers with alignment
            let after_header = raw_base.add(HEADER_SIZE);
            let pad = after_header.align_offset(align);
            if pad == usize::MAX {
                // Alignment failed (extremely unlikely)
                release(preserve_tag);
                return ptr::null_mut();
            }

            let data = after_header.add(pad);
            let header = data.sub(HEADER_SIZE).cast::<Header>();

            header.write(Header { preserve_tag });

            debug_assert_eq!(data.align_offset(layout.align()), 0);
            data
        }
    }

    unsafe fn dealloc(&self, data: *mut u8, _layout: Layout) {
        unsafe {
            if data.is_null() {
                return;
            }

            let header = data.sub(HEADER_SIZE).cast::<Header>();
            let preserve_tag = (*header).preserve_tag;

            release(preserve_tag);
        }
    }

    unsafe fn realloc(&self, old: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        unsafe {
            if old.is_null() {
                let Ok(new_layout) = Layout::from_size_align(new_size, layout.align()) else {
                    return ptr::null_mut();
                };
                return self.alloc(new_layout);
            }

            if new_size == 0 {
                self.dealloc(old, layout);
                return ptr::null_mut();
            }

            // Recover RAWSXP via preserve tag.
            let header = old.sub(HEADER_SIZE).cast::<Header>();
            let preserve_tag = (*header).preserve_tag;
            let sexp = crate::ffi::TAG(preserve_tag);

            // Exact available capacity from `old` to end of the RAWSXP.
            let raw_base = RAW(sexp).cast::<u8>();
            let cap: usize = match crate::ffi::Rf_xlength(sexp).try_into() {
                Ok(n) => n,
                Err(_) => return ptr::null_mut(),
            };

            let used = (old as *const u8).offset_from(raw_base as *const u8);
            if used < 0 {
                // Should be impossible if `old` came from this allocator, but don't UB.
                return ptr::null_mut();
            }
            let available = cap.saturating_sub(used as usize);

            if new_size <= available {
                return old;
            }

            let Ok(new_layout) = Layout::from_size_align(new_size, layout.align()) else {
                return ptr::null_mut();
            };

            let new_ptr = self.alloc(new_layout);
            if new_ptr.is_null() {
                // On realloc failure, the old allocation must remain valid.
                return ptr::null_mut();
            }

            ptr::copy_nonoverlapping(old, new_ptr, layout.size().min(new_size));
            self.dealloc(old, layout);

            new_ptr
        }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        unsafe {
            // Don't rely on RAWSXP being zeroed; make it explicit.
            let p = self.alloc(layout);
            if !p.is_null() {
                ptr::write_bytes(p, 0, layout.size());
            }
            p
        }
    }
}
