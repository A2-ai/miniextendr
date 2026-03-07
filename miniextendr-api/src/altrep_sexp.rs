//! `AltrepSexp` — a `!Send + !Sync` wrapper for ALTREP vectors.
//!
//! ALTREP vectors must not have `DATAPTR_RO` called on non-R threads because
//! materialization dispatches into R internals (C callbacks, GC toggling,
//! allocation). `AltrepSexp` makes this a compile-time guarantee: it wraps
//! a SEXP known to be ALTREP and is `!Send + !Sync`, preventing it from
//! crossing thread boundaries.
//!
//! Plain (non-ALTREP) SEXPs remain `Send + Sync` and are unaffected.
//!
//! # Usage
//!
//! ```ignore
//! use miniextendr_api::AltrepSexp;
//!
//! // Wrap a SEXP that might be ALTREP:
//! if let Some(altrep) = AltrepSexp::try_wrap(sexp) {
//!     // Must materialize on R main thread before accessing data
//!     let materialized: SEXP = unsafe { altrep.materialize() };
//! }
//!
//! // Or use the convenience helper:
//! let safe_sexp = unsafe { ensure_materialized(sexp) };
//! ```

use crate::ffi::{self, Rcomplex, SEXP, SEXPTYPE};
use crate::from_r::r_slice;
use std::marker::PhantomData;
use std::rc::Rc;

/// A SEXP known to be ALTREP. `!Send + !Sync` — must be materialized on the
/// R main thread before data can be accessed or sent to other threads.
///
/// This type prevents ALTREP vectors from being accidentally sent to rayon
/// or other worker threads where `DATAPTR_RO` would invoke R internals
/// (undefined behavior).
///
/// # Construction
///
/// - [`AltrepSexp::try_wrap`] — runtime check, returns `None` if not ALTREP
/// - [`AltrepSexp::from_raw`] — unsafe, caller asserts `ALTREP(sexp) != 0`
///
/// # Materialization
///
/// - [`AltrepSexp::materialize`] — forces R to materialize, returns plain SEXP
/// - [`AltrepSexp::materialize_real`] — materialize and return `&[f64]`
/// - [`AltrepSexp::materialize_integer`] — materialize and return `&[i32]`
/// - [`AltrepSexp::materialize_strings`] — materialize STRSXP to `Vec<Option<String>>`
pub struct AltrepSexp {
    sexp: SEXP,
    /// PhantomData<Rc<()>> makes this type !Send + !Sync.
    _not_send: PhantomData<Rc<()>>,
}

impl AltrepSexp {
    /// Wrap a SEXP that is known to be ALTREP.
    ///
    /// # Safety
    ///
    /// Caller must ensure `ALTREP(sexp)` is true (non-zero).
    #[inline]
    pub unsafe fn from_raw(sexp: SEXP) -> Self {
        debug_assert!(unsafe { ffi::ALTREP(sexp) } != 0);
        Self {
            sexp,
            _not_send: PhantomData,
        }
    }

    /// Check a SEXP and wrap if ALTREP. Returns `None` if not ALTREP.
    #[inline]
    pub fn try_wrap(sexp: SEXP) -> Option<Self> {
        if unsafe { ffi::ALTREP(sexp) } != 0 {
            Some(Self {
                sexp,
                _not_send: PhantomData,
            })
        } else {
            None
        }
    }

    /// Force materialization and return the (now materialized) SEXP.
    ///
    /// For contiguous types (INTSXP, REALSXP, LGLSXP, RAWSXP, CPLXSXP),
    /// calls `DATAPTR_RO` to trigger ALTREP materialization.
    /// For STRSXP, iterates `STRING_ELT` to force element materialization.
    ///
    /// After this call, the SEXP's data pointer is stable and can be safely
    /// accessed from any thread (the SEXP itself is still `Send + Sync`).
    ///
    /// # Safety
    ///
    /// Must be called on the R main thread.
    pub unsafe fn materialize(self) -> SEXP {
        let typ = unsafe { ffi::TYPEOF(self.sexp) } as SEXPTYPE;
        match typ {
            SEXPTYPE::STRSXP => {
                let n = unsafe { ffi::Rf_xlength(self.sexp) };
                for i in 0..n {
                    let _ = unsafe { ffi::STRING_ELT(self.sexp, i) };
                }
            }
            SEXPTYPE::INTSXP
            | SEXPTYPE::REALSXP
            | SEXPTYPE::LGLSXP
            | SEXPTYPE::RAWSXP
            | SEXPTYPE::CPLXSXP => {
                let _ = unsafe { ffi::DATAPTR_RO(self.sexp) };
            }
            _ => {} // non-vector types, nothing to materialize
        }
        self.sexp
    }

    /// Materialize and return a typed slice of `f64` (REALSXP).
    ///
    /// # Safety
    ///
    /// Must be called on the R main thread. The SEXP must be REALSXP.
    pub unsafe fn materialize_real(&self) -> &[f64] {
        let ptr = unsafe { ffi::DATAPTR_RO(self.sexp) } as *const f64;
        let len = unsafe { ffi::Rf_xlength(self.sexp) } as usize;
        unsafe { r_slice(ptr, len) }
    }

    /// Materialize and return a typed slice of `i32` (INTSXP).
    ///
    /// # Safety
    ///
    /// Must be called on the R main thread. The SEXP must be INTSXP.
    pub unsafe fn materialize_integer(&self) -> &[i32] {
        let ptr = unsafe { ffi::DATAPTR_RO(self.sexp) } as *const i32;
        let len = unsafe { ffi::Rf_xlength(self.sexp) } as usize;
        unsafe { r_slice(ptr, len) }
    }

    /// Materialize and return a typed slice of `i32` (LGLSXP, R's internal logical storage).
    ///
    /// # Safety
    ///
    /// Must be called on the R main thread. The SEXP must be LGLSXP.
    pub unsafe fn materialize_logical(&self) -> &[i32] {
        let ptr = unsafe { ffi::DATAPTR_RO(self.sexp) } as *const i32;
        let len = unsafe { ffi::Rf_xlength(self.sexp) } as usize;
        unsafe { r_slice(ptr, len) }
    }

    /// Materialize and return a typed slice of `u8` (RAWSXP).
    ///
    /// # Safety
    ///
    /// Must be called on the R main thread. The SEXP must be RAWSXP.
    pub unsafe fn materialize_raw(&self) -> &[u8] {
        let ptr = unsafe { ffi::DATAPTR_RO(self.sexp) } as *const u8;
        let len = unsafe { ffi::Rf_xlength(self.sexp) } as usize;
        unsafe { r_slice(ptr, len) }
    }

    /// Materialize and return a typed slice of `Rcomplex` (CPLXSXP).
    ///
    /// # Safety
    ///
    /// Must be called on the R main thread. The SEXP must be CPLXSXP.
    pub unsafe fn materialize_complex(&self) -> &[Rcomplex] {
        let ptr = unsafe { ffi::DATAPTR_RO(self.sexp) } as *const Rcomplex;
        let len = unsafe { ffi::Rf_xlength(self.sexp) } as usize;
        unsafe { r_slice(ptr, len) }
    }

    /// Materialize strings into owned Rust data.
    ///
    /// Each element is `None` for `NA_character_`, or `Some(String)` otherwise.
    ///
    /// # Safety
    ///
    /// Must be called on the R main thread. The SEXP must be STRSXP.
    pub unsafe fn materialize_strings(&self) -> Vec<Option<String>> {
        let n = unsafe { ffi::Rf_xlength(self.sexp) } as usize;
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            let elt = unsafe { ffi::STRING_ELT(self.sexp, i as ffi::R_xlen_t) };
            if elt == unsafe { ffi::R_NaString } {
                out.push(None);
            } else {
                let cstr = unsafe { ffi::Rf_translateCharUTF8(elt) };
                let s = unsafe { std::ffi::CStr::from_ptr(cstr) }
                    .to_string_lossy()
                    .into_owned();
                out.push(Some(s));
            }
        }
        out
    }

    /// Get the inner SEXP without materializing.
    ///
    /// # Safety
    ///
    /// The returned SEXP is still ALTREP. Do not call `DATAPTR_RO` on it
    /// from a non-R thread.
    #[inline]
    pub unsafe fn as_raw(&self) -> SEXP {
        self.sexp
    }

    /// Get the SEXPTYPE of the underlying vector.
    #[inline]
    pub fn sexptype(&self) -> SEXPTYPE {
        (unsafe { ffi::TYPEOF(self.sexp) }) as SEXPTYPE
    }

    /// Get the length of the underlying vector.
    #[inline]
    pub fn len(&self) -> usize {
        (unsafe { ffi::Rf_xlength(self.sexp) }) as usize
    }

    /// Check if the underlying vector is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl std::fmt::Debug for AltrepSexp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AltrepSexp")
            .field("sexptype", &self.sexptype())
            .field("len", &self.len())
            .finish()
    }
}

/// If `sexp` is ALTREP, materialize it in place and return the SEXP.
/// If not ALTREP, return as-is.
///
/// This is the main entry point for ensuring a SEXP is safe to access
/// from non-R threads. After materialization, the data pointer is stable
/// and the SEXP can be freely sent across threads.
///
/// # Safety
///
/// Must be called on the R main thread.
#[inline]
pub unsafe fn ensure_materialized(sexp: SEXP) -> SEXP {
    if unsafe { ffi::ALTREP(sexp) } != 0 {
        unsafe { AltrepSexp::from_raw(sexp).materialize() }
    } else {
        sexp
    }
}

// Compile-time assertions: SEXP must remain Send + Sync.
const _: () = {
    fn _assert_send<T: Send>() {}
    fn _assert_sync<T: Sync>() {}

    fn _sexp_is_send_sync() {
        _assert_send::<SEXP>();
        _assert_sync::<SEXP>();
    }
};

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify AltrepSexp is !Send and !Sync at compile time.
    /// SEXP IS Send + Sync.
    fn _assert_send_sync_properties() {
        fn requires_send<T: Send>() {}
        fn requires_sync<T: Sync>() {}

        // These must NOT compile — uncomment to verify:
        // requires_send::<AltrepSexp>();
        // requires_sync::<AltrepSexp>();

        // SEXP IS Send + Sync:
        requires_send::<SEXP>();
        requires_sync::<SEXP>();
    }
}
