//! Shared storage for the pointers and optional R owner of a generated view.

use crate::SEXP;
use crate::gc_protect::OwnedProtect;
use std::ffi::c_void;

/// The checked construction boundary of a generated trait view.
///
/// Pointer fields live here, rather than in the expansion's module: even
/// code beside the generated view cannot construct or overwrite them safely.
/// An R owner is preserved independently of the PROTECT stack, allowing views
/// to outlive their creating scope and to be dropped in arbitrary order.
/// Raw pointers keep this type (and every generated view) `!Send` and `!Sync`.
#[doc(hidden)]
pub struct TraitViewData<V> {
    data: *mut c_void,
    vtable: *const V,
    owner: Option<SEXP>,
}

impl<V> TraitViewData<V> {
    /// Construct a view's backing storage.
    ///
    /// # Safety
    ///
    /// `data` and `vtable` must be valid, non-null pointers matching the
    /// generated trait view. The vtable must outlive this value. When supplied,
    /// `owner` must own `data`; otherwise the caller must keep `data` alive.
    /// Calls must respect Rust's aliasing rules. Construction, use and drop
    /// must run on R's main thread, inside an R unwind boundary when allocating.
    pub unsafe fn new(data: *mut c_void, vtable: *const c_void, owner: Option<SEXP>) -> Self {
        if let Some(owner) = owner {
            // Preserve can allocate. Protect before creating the independent
            // root, then leave the caller's stack unchanged.
            let root = unsafe { OwnedProtect::new(owner) };
            unsafe { crate::sys::R_PreserveObject(root.get()) };
        }
        Self {
            data,
            vtable: vtable.cast(),
            owner,
        }
    }

    /// The concrete data pointer, retaining its original mutable provenance.
    pub fn data(&self) -> *mut c_void {
        self.data
    }

    /// The interface vtable. Dereferencing it remains an unsafe operation.
    pub fn vtable(&self) -> *const V {
        self.vtable
    }

    /// The rooted R receiver, absent for a raw Rust-owned view.
    pub fn owner(&self) -> Option<SEXP> {
        self.owner
    }
}

impl<V> Drop for TraitViewData<V> {
    fn drop(&mut self) {
        if let Some(owner) = self.owner {
            // SAFETY: construction preserved it, and this type cannot move
            // to another thread. Release does not allocate or call finalizers.
            unsafe { crate::sys::R_ReleaseObject(owner) };
        }
    }
}
