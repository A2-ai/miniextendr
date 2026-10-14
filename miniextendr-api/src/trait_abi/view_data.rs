//! Shared storage for the pointers and optional R owner of a generated view.

use crate::SEXP;
use crate::externalptr::{root_owned, unroot_owned};
use crate::gc_protect::OwnedProtect;
use crate::protect_pool::ProtectKey;
use std::ffi::c_void;

/// The checked construction boundary of a generated trait view.
///
/// Pointer fields live here, rather than in the expansion's module: even
/// code beside the generated view cannot construct or overwrite them safely.
/// An R owner is rooted in the same pool as owning `ExternalPtr` handles,
/// independently of the PROTECT stack, so views can outlive their creating
/// scope and be dropped in any order.
/// Raw pointers keep this type (and every generated view) `!Send` and `!Sync`.
#[doc(hidden)]
pub struct TraitViewData<V> {
    data: *mut c_void,
    vtable: *const V,
    owner: Option<(SEXP, ProtectKey)>,
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
        let owner = owner.map(|owner| {
            // The pool can allocate while it grows. Protect the owner until
            // it holds its own root, then leave the caller's stack unchanged.
            let guard = unsafe { OwnedProtect::new(owner) };
            (owner, root_owned(guard.get()))
        });
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

    /// The rooted R receiver, which owner-receiver methods pass to their shim.
    /// `None` for a raw view built without an owner.
    pub fn owner(&self) -> Option<SEXP> {
        self.owner.map(|(owner, _)| owner)
    }
}

impl<V> Drop for TraitViewData<V> {
    fn drop(&mut self) {
        if let Some((_, key)) = self.owner.take() {
            // This type cannot leave the main thread it was built on, and
            // releasing a pool slot neither allocates nor runs finalizers.
            unroot_owned(key);
        }
    }
}
