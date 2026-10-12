//! Borrow state shared by every R alias of an external pointer.
//!
//! The counter belongs to the owner's protection list, not a package-local
//! registry: trait views in another DLL must observe the same active borrow.
//! All transitions happen on R's main thread. A wrapper keeps its guard on
//! that thread while worker code runs and while R services its callbacks.

use super::{PROT_BORROW_INDEX, PROT_VEC_LEN, root_owned, unroot_owned};
use crate::protect_pool::ProtectKey;
use crate::{SEXP, SEXPTYPE, SexpExt};
use core::{marker::PhantomData, ptr::NonNull};

/// Initialize the counter before publishing a new external pointer.
///
/// # Safety
/// R's main thread, inside an unwind boundary. `owner` must be rooted and
/// have either no protection list (trait-ABI storage) or the framework layout.
/// An existing counter is retained when sidecar initialization already made it.
pub(crate) unsafe fn init_borrow_state(owner: SEXP) {
    unsafe {
        let mut prot = crate::sys::R_ExternalPtrProtected_unchecked(owner);
        if prot.type_of() == SEXPTYPE::NILSXP {
            prot = crate::sys::Rf_allocVector_unchecked(SEXPTYPE::VECSXP, PROT_VEC_LEN);
            crate::sys::R_SetExternalPtrProtected_unchecked(owner, prot);
        }
        if prot.vector_elt_unchecked(PROT_BORROW_INDEX).type_of() != SEXPTYPE::NILSXP {
            return;
        }
        // A fresh vector, not a possibly shared cached scalar integer.
        let counter = crate::gc_protect::OwnedProtect::new(crate::sys::Rf_allocVector_unchecked(
            SEXPTYPE::INTSXP,
            1,
        ));
        crate::sys::INTEGER_unchecked(counter.get()).write(0);
        prot.set_vector_elt_unchecked(PROT_BORROW_INDEX, counter.get());
    }
}

/// Call-scoped access to a Rust value owned by an R external pointer.
///
/// Generated receivers and field accessors acquire this before forming a
/// reference. Shared reads compose; a write conflicts with every other borrow.
/// Dropping the guard releases the counter on normal return and Rust/R unwind.
/// The guard is deliberately neither Send nor Sync: keep it on R's main thread
/// even when the borrowed value is passed to a worker.
#[doc(hidden)]
pub struct ExternalBorrow {
    owner: SEXP,
    owner_root: ProtectKey,
    counter: Option<NonNull<i32>>,
    root: Option<ProtectKey>,
    mutable: bool,
    _main_thread: PhantomData<*mut ()>,
}

impl ExternalBorrow {
    /// Borrow a live owner's value, or leave a cleared/restored owner to its
    /// ordinary receiver validation (which supplies the type-specific error).
    ///
    /// # Safety
    /// On R's main thread, `owner` must be a valid framework external pointer
    /// with initialized borrow state, live on entry. The guard roots it.
    /// Acquire before creating any reference; keep the guard until all such
    /// references, including return-value conversion borrows, have ended.
    pub unsafe fn acquire(owner: SEXP, mutable: bool) -> Self {
        assert!(
            crate::worker::is_r_main_thread(),
            "external-pointer borrows require R's main thread"
        );
        let owner = unsafe { crate::gc_protect::OwnedProtect::new(owner) };
        let mut borrow = Self {
            owner: owner.get(),
            owner_root: root_owned(owner.get()),
            counter: None,
            root: None,
            mutable,
            _main_thread: PhantomData,
        };
        let owner = owner.get();
        unsafe {
            if crate::sys::R_ExternalPtrAddr_unchecked(owner).is_null() {
                return borrow;
            }
            let prot = crate::sys::R_ExternalPtrProtected_unchecked(owner);
            let counter =
                crate::gc_protect::OwnedProtect::new(prot.vector_elt_unchecked(PROT_BORROW_INDEX));
            // Moving the value out clears the owner's protection list. Keep
            // the counter rooted separately until Drop, in arbitrary order.
            // The temporary also roots it while the pool might grow.
            borrow.root = Some(root_owned(counter.get()));
            if crate::sys::R_ExternalPtrAddr_unchecked(owner).is_null() {
                return borrow;
            }
            let counter = NonNull::new_unchecked(crate::sys::INTEGER_unchecked(counter.get()));
            // No Rust reference to the counter survives a transition, and all
            // transitions (including Drop) run on the one R thread. Read after
            // rooting, since pool growth can allocate and run R finalizers.
            let current = counter.as_ptr().read();
            if current < 0 || (mutable && current != 0) {
                crate::rust_error!(
                    class = "miniextendr_borrow_conflict",
                    "external pointer is already borrowed by a running call"
                );
            }
            let next = if mutable {
                -1
            } else {
                current
                    .checked_add(1)
                    .expect("external-pointer borrow count overflow")
            };
            counter.as_ptr().write(next);
            borrow.counter = Some(counter);
        }
        borrow
    }

    /// The resolved, rooted owner this guard protects. Do not resolve an
    /// active binding again between acquiring the guard and extracting data.
    pub fn owner(&self) -> SEXP {
        self.owner
    }
}

impl Drop for ExternalBorrow {
    fn drop(&mut self) {
        if let Some(counter) = self.counter {
            unsafe {
                let next = if self.mutable {
                    0
                } else {
                    counter.as_ptr().read() - 1
                };
                counter.as_ptr().write(next);
            }
        }
        if let Some(root) = self.root {
            unroot_owned(root);
        }
        unroot_owned(self.owner_root);
    }
}
