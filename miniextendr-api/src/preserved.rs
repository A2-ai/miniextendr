//! [`Preserved`]: an owned GC root for one R object, on R's precious list.

use std::marker::PhantomData;

use crate::SEXP;
use crate::into_r::IntoR;

/// An owned GC root for one R object: `R_PreserveObject` when made,
/// `R_ReleaseObject` when dropped.
///
/// Hold one where a Rust value keeps an R object across allocations or past
/// the `.Call()` that made it, such as a field of a struct that lives in an
/// [`ExternalPtr`](struct@crate::ExternalPtr) and drops with its finalizer.
/// The object stays reachable for as long as the handle lives, whatever the
/// drop order. [`ProtectScope`](crate::ProtectScope) and
/// [`OwnedProtect`](crate::OwnedProtect) use R's protect stack instead, which
/// unwinds in order and ends with the call.
///
/// ```ignore
/// struct Fit {
///     data: Preserved, // the data frame the model was fitted on
/// }
///
/// let fit = Fit { data: Preserved::new(df) };
/// // Any number of allocations and `.Call()`s later:
/// let n = fit.data.len();
/// ```
///
/// - [`Clone`] preserves the object again. Each handle releases its own entry,
///   so clones drop in any order.
/// - The handle dereferences to its [`SEXP`], so [`SexpExt`](crate::SexpExt)
///   methods apply to it. A `SEXP` copied out of it (`*handle`,
///   [`as_sexp`](Self::as_sexp)) stays rooted only while the handle lives.
/// - [`IntoR`] releases the root and hands the object to the caller: return
///   the handle from a `#[miniextendr]` function to return the object.
/// - `!Send + !Sync`: the precious list is R main-thread state. Make, clone
///   and drop the handle on R's main thread.
///
/// # Cost
///
/// R keeps the precious list as one linked list. Preserving conses a cell onto
/// its head, an allocation. Releasing searches the list from the head, one step
/// per object preserved after this one and not yet released. Starting R with
/// the environment variable `R_HASH_PRECIOUS` set splits the list into 1069
/// hashed buckets.
///
/// That suits a few long-lived objects. For many roots, or roots released in
/// no particular order, use a [`ProtectPool`](crate::ProtectPool): it preserves
/// one list and inserts and releases in O(1), and it roots every
/// [`ExternalPtr`](struct@crate::ExternalPtr), because a `Vec` of handles
/// dropped front to back makes each release scan the whole precious list.
/// [`RefCountedArena`](crate::RefCountedArena) does the same with a count per
/// object. Within one call, the protect stack ([`ProtectScope`]) allocates
/// nothing.
///
/// [`ProtectScope`]: crate::ProtectScope
pub struct Preserved {
    sexp: SEXP,
    /// `!Send + !Sync`: preserving and releasing edit R's precious list.
    _not_send: PhantomData<*mut ()>,
}

impl Preserved {
    /// Convert `value` to an R object and preserve it.
    ///
    /// A [`SEXP`] converts to itself, so `Preserved::new(sexp)` roots an
    /// existing object. Rooting is immediate: nothing allocates between the
    /// conversion and `R_PreserveObject`, which keeps its argument reachable
    /// while it allocates its cell, so a fresh, unprotected object is safe to
    /// pass. Call it on R's main thread.
    pub fn new(value: impl IntoR) -> Self {
        let sexp = value.into_sexp();
        // SAFETY: `into_sexp` just ran on R's main thread and returned a valid
        // SEXP; rooting is immediate (see above).
        unsafe { crate::sys::R_PreserveObject(sexp) };
        Self {
            sexp,
            _not_send: PhantomData,
        }
    }

    /// The preserved object, rooted for as long as this handle lives.
    #[inline]
    pub fn as_sexp(&self) -> SEXP {
        self.sexp
    }

    /// Release the root and return the object, now unprotected: the caller
    /// takes it over. `R_ReleaseObject` does not allocate, so returning the
    /// object straight to R (the `.Call()` return) is safe.
    fn release(self) -> SEXP {
        let sexp = self.sexp;
        // SAFETY: `!Send` keeps us on the thread that preserved it. Release the
        // entry added in `new`, and skip `Drop` so it is not released twice.
        unsafe { crate::sys::R_ReleaseObject(sexp) };
        std::mem::forget(self);
        sexp
    }
}

impl Clone for Preserved {
    /// Preserve the object again, for a handle with its own entry.
    fn clone(&self) -> Self {
        Self::new(self.sexp)
    }
}

impl Drop for Preserved {
    fn drop(&mut self) {
        // SAFETY: `!Send` keeps the drop on the thread that preserved it.
        // Release removes the entry added in `new` (precious-list entries
        // stack, so other handles to the object keep theirs) and allocates
        // nothing.
        unsafe { crate::sys::R_ReleaseObject(self.sexp) };
    }
}

impl std::ops::Deref for Preserved {
    type Target = SEXP;

    #[inline]
    fn deref(&self) -> &SEXP {
        &self.sexp
    }
}

impl IntoR for Preserved {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.release())
    }

    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        Ok(self.release())
    }

    #[inline]
    fn into_sexp(self) -> SEXP {
        self.release()
    }
}

impl std::fmt::Debug for Preserved {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Preserved").field(&self.sexp).finish()
    }
}
