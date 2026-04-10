//! `AltrepSexp` — a `!Send + !Sync` wrapper for ALTREP vectors.
//!
//! R uses ALTREP (Alternative Representations) for common idioms like `1:N`,
//! `seq_len(N)`, and `as.character(1:N)`. These vectors are lazily materialized:
//! calling `DATAPTR_RO` triggers allocation, GC, and C callbacks inside R's
//! runtime. This must only happen on the R main thread.
//!
//! This module provides two complementary tools:
//!
//! - **[`AltrepSexp`]** — a `!Send + !Sync` wrapper that holds an ALTREP SEXP
//!   and prevents it from crossing thread boundaries at compile time.
//! - **[`ensure_materialized`]** — a function that forces materialization if
//!   the SEXP is ALTREP, returning a SEXP with a stable data pointer.
//!
//! Plain (non-ALTREP) SEXPs are `Send + Sync` and are unaffected by either.
//!
//! # How ALTREP flows through miniextendr
//!
//! | Parameter type | ALTREP handling |
//! |---|---|
//! | Typed (`Vec<i32>`, `&[f64]`) | Auto-materialized via `DATAPTR_RO` in `TryFromSexp` |
//! | `SEXP` | Auto-materialized via [`ensure_materialized`] in `TryFromSexp` |
//! | [`AltrepSexp`] | Wrapped without materializing. `!Send + !Sync`. |
//! | `extern "C-unwind"` raw SEXP | No conversion — receives raw SEXP as-is |
//!
//! # Usage
//!
//! ```ignore
//! use miniextendr_api::AltrepSexp;
//!
//! // As a #[miniextendr] parameter — accepts only ALTREP vectors:
//! #[miniextendr]
//! pub fn altrep_length(x: AltrepSexp) -> usize {
//!     x.len()
//! }
//!
//! // Manual wrapping:
//! if let Some(altrep) = AltrepSexp::try_wrap(sexp) {
//!     // Must materialize on R main thread before accessing data
//!     let materialized: SEXP = unsafe { altrep.materialize() };
//! }
//!
//! // Or use the convenience helper on any SEXP:
//! let safe_sexp = unsafe { ensure_materialized(sexp) };
//! ```
//!
//! See also: `docs/ALTREP_SEXP.md` for the full guide on receiving ALTREP
//! vectors from R.

use crate::ffi::{self, Rcomplex, SEXP, SEXPTYPE, SexpExt};
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
/// # As a `#[miniextendr]` parameter
///
/// `AltrepSexp` implements [`TryFromSexp`](crate::from_r::TryFromSexp), so it
/// can be used directly as a function parameter. It **only accepts ALTREP
/// vectors** — non-ALTREP input produces an error.
///
/// ```ignore
/// #[miniextendr]
/// pub fn altrep_info(x: AltrepSexp) -> String {
///     format!("{:?}, len={}", x.sexptype(), x.len())
/// }
/// ```
///
/// ```r
/// altrep_info(1:10)          # OK — 1:10 is ALTREP
/// altrep_info(c(1L, 2L, 3L)) # Error: "expected an ALTREP vector"
/// ```
///
/// # Construction
///
/// - [`AltrepSexp::try_wrap`] — runtime check, returns `None` if not ALTREP
/// - [`AltrepSexp::from_raw`] — unsafe, caller asserts `ALTREP(sexp) != 0`
///
/// # Materialization
///
/// All materialization methods must be called on the R main thread.
///
/// - [`AltrepSexp::materialize`] — forces R to materialize, returns plain SEXP
/// - [`AltrepSexp::materialize_integer`] — materialize INTSXP and return `&[i32]`
/// - [`AltrepSexp::materialize_real`] — materialize REALSXP and return `&[f64]`
/// - [`AltrepSexp::materialize_logical`] — materialize LGLSXP and return `&[i32]`
/// - [`AltrepSexp::materialize_raw`] — materialize RAWSXP and return `&[u8]`
/// - [`AltrepSexp::materialize_complex`] — materialize CPLXSXP and return `&[Rcomplex]`
/// - [`AltrepSexp::materialize_strings`] — materialize STRSXP to `Vec<Option<String>>`
///
/// # Thread safety
///
/// `AltrepSexp` is `!Send + !Sync` (via `PhantomData<Rc<()>>`). This is a
/// compile-time guarantee: you cannot send an un-materialized ALTREP vector
/// to another thread. Call one of the `materialize_*` methods first to get
/// a `Send + Sync` slice or SEXP.
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
        debug_assert!(sexp.is_altrep());
        Self {
            sexp,
            _not_send: PhantomData,
        }
    }

    /// Check a SEXP and wrap if ALTREP. Returns `None` if not ALTREP.
    #[inline]
    pub fn try_wrap(sexp: SEXP) -> Option<Self> {
        if sexp.is_altrep() {
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
        let typ = self.sexp.type_of();
        match typ {
            SEXPTYPE::STRSXP => {
                let n = unsafe { ffi::Rf_xlength(self.sexp) };
                for i in 0..n {
                    let _ = self.sexp.string_elt(i);
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
            let elt = self.sexp.string_elt(i as ffi::R_xlen_t);
            if elt == SEXP::na_string() {
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
        self.sexp.type_of()
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

/// Conversion from R SEXP to `AltrepSexp`.
///
/// Only succeeds if the input is an ALTREP vector (`ALTREP(sexp) != 0`).
/// Non-ALTREP input produces `SexpError::InvalidValue`.
///
/// This is the inverse of [`TryFromSexp for SEXP`](crate::from_r::TryFromSexp),
/// which accepts any SEXP but auto-materializes ALTREP.
impl crate::from_r::TryFromSexp for AltrepSexp {
    type Error = crate::from_r::SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        AltrepSexp::try_wrap(sexp).ok_or_else(|| {
            crate::from_r::SexpError::InvalidValue(
                "expected an ALTREP vector but got a non-ALTREP SEXP".to_string(),
            )
        })
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Self::try_from_sexp(sexp)
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

/// If `sexp` is ALTREP, force materialization and return the SEXP.
/// If not ALTREP, return as-is (no-op).
///
/// This is the main entry point for ensuring a SEXP is safe to access
/// from non-R threads. After materialization, the data pointer is stable
/// and the SEXP can be freely sent across threads.
///
/// Called automatically by `TryFromSexp for SEXP` — you only need to call
/// this directly in `extern "C-unwind"` functions that receive raw SEXPs.
///
/// For contiguous types (INTSXP, REALSXP, LGLSXP, RAWSXP, CPLXSXP),
/// calls `DATAPTR_RO` to trigger materialization. For STRSXP, iterates
/// `STRING_ELT` to force each element to materialize.
///
/// # Safety
///
/// Must be called on the R main thread (materialization invokes R internals).
#[inline]
pub unsafe fn ensure_materialized(sexp: SEXP) -> SEXP {
    if sexp.is_altrep() {
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
