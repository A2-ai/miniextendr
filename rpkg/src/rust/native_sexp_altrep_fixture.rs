//! Native-SEXP `AltrepExtract` proof-of-concept.
//!
//! Demonstrates that `AltrepExtract` is not limited to `ExternalPtr`-backed
//! storage.  Here the backing data lives directly in the ALTREP `data1` slot
//! as a plain `INTSXP` — the same pattern used by R's own `compact_intseq`
//! in `src/main/altclasses.c`.
//!
//! # Storage layout
//!
//! | Slot   | Contents                                                   |
//! |--------|------------------------------------------------------------|
//! | data1  | plain `INTSXP` holding the integer values (the actual data)|
//! | data2  | `R_NilValue` — no materialization cache needed;            |
//! |        | `data1` IS the materialization                             |
//!
//! Because the backing data is already a native R vector, the `Dataptr`
//! callback can simply return `DATAPTR_RO(data1)` without building a
//! separate copy in `data2`.
//!
//! # `AltrepExtract` implementation
//!
//! `NativeSexpIntAltrep` is a zero-sized type (ZST) — all state lives in
//! `data1` on the ALTREP SEXP.  `altrep_extract_ref` / `altrep_extract_mut`
//! return references to a static singleton, which is sound because the
//! singleton carries no mutable state.
//!
//! Pointer provenance for `altrep_extract_mut`: we must NOT write `&mut INSTANCE`
//! directly (Stacked Borrows UB on aliased mutable refs to statics).  We use
//! `std::ptr::addr_of_mut!` to produce a raw pointer and then call
//! `.as_mut().unwrap_unchecked()` — the provenance-clean pattern described in
//! CLAUDE.md.
//!
//! # Registration path
//!
//! This PoC does NOT use `#[derive(AltrepInteger)]`.  Instead it manually
//! implements the required trait hierarchy:
//!
//! 1. `AltrepExtract` — singleton extraction (custom impl, no ExternalPtr)
//! 2. `Altrep` + `AltVec` + `AltInteger` — low-level method tables (manual)
//! 3. `InferBase` — maps the type to `INTSXP` + installs method tables
//! 4. `RegisterAltrep` — class registration via `OnceLock`
//! 5. `IntoR` — constructs the ALTREP SEXP with a plain INTSXP in `data1`
//!
//! The approach avoids `ExternalPtr` entirely: `data1` IS the integer data.
//! This is the `compact_intseq`-style native-SEXP pattern.

use std::sync::OnceLock;

use miniextendr_api::altrep::RegisterAltrep;
use miniextendr_api::altrep_data::{AltIntegerData, AltrepDataptr, AltrepExtract, AltrepLen};
use miniextendr_api::ffi::{
    DATAPTR_RO, R_xlen_t, Rf_allocVector, Rf_protect, Rf_unprotect, SEXP, SEXPTYPE, SexpExt as _,
};
use miniextendr_api::into_r::IntoR;
use miniextendr_api::{impl_inferbase_integer, miniextendr};

// region: NativeSexpIntAltrep — ZST, all data in ALTREP data1

/// Zero-sized ALTREP implementation where all element storage is a plain
/// `INTSXP` in the ALTREP `data1` slot.
///
/// This type is NOT a registered R class via `#[miniextendr(r6)]` or similar.
/// R-visible entry points are `native_sexp_altrep_new()` (constructor) and
/// `gc_stress_native_sexp_altrep()` (GC-torture fixture).
pub struct NativeSexpIntAltrep;

/// Static singleton — the ZST has no state, so one instance suffices.
static mut INSTANCE: NativeSexpIntAltrep = NativeSexpIntAltrep;

// endregion

// region: AltrepExtract — return static singleton instead of ExternalPtr

impl AltrepExtract for NativeSexpIntAltrep {
    unsafe fn altrep_extract_ref(_x: SEXP) -> &'static Self {
        // SAFETY: `NativeSexpIntAltrep` is a ZST with no interior mutability.
        // Shared references to a ZST are always safe.  R's GC keeps the ALTREP
        // SEXP (and thus `data1`) alive for the callback duration.
        unsafe { &*std::ptr::addr_of!(INSTANCE) }
    }

    unsafe fn altrep_extract_mut(_x: SEXP) -> &'static mut Self {
        // SAFETY: `NativeSexpIntAltrep` is a ZST — no data can be aliased.
        // `addr_of_mut!` avoids creating a mutable reference directly to a
        // static (Stacked Borrows UB on ZST statics).  The raw pointer is then
        // reborrrowed as `&mut`, valid because the ZST occupies no memory.
        unsafe { std::ptr::addr_of_mut!(INSTANCE).as_mut().unwrap_unchecked() }
    }
}

// endregion

// region: High-level data traits (stubs required by impl_inferbase_integer! bound)
//
// The low-level `Altrep` / `AltInteger` impls below access `data1` directly
// from the ALTREP `x: SEXP`, bypassing these high-level methods entirely.
// The stubs are only here to satisfy the `AltIntegerData` bound on
// `impl_inferbase_integer!`.

impl AltrepLen for NativeSexpIntAltrep {
    fn len(&self) -> usize {
        // Stub — the low-level `Altrep::length` reads directly from `data1`.
        0
    }
}

impl AltIntegerData for NativeSexpIntAltrep {
    fn elt(&self, _i: usize) -> i32 {
        // Stub — the low-level `AltInteger::elt` reads directly from `data1`.
        0
    }
}

impl AltrepDataptr<i32> for NativeSexpIntAltrep {
    fn dataptr(&mut self, _writable: bool) -> Option<*mut i32> {
        None // Stub — the low-level `AltVec::dataptr` goes direct.
    }
}

// endregion

// region: InferBase — maps NativeSexpIntAltrep to INTSXP + method installation

impl_inferbase_integer!(NativeSexpIntAltrep);

// endregion

// region: Low-level Altrep / AltVec / AltInteger trait impls
//
// These receive the ALTREP `x: SEXP` directly from R's trampoline, so they
// can access `data1` (the plain INTSXP) without any thread-local tricks.

use miniextendr_api::altrep_traits::{AltInteger, AltVec, Altrep, AltrepGuard};

impl Altrep for NativeSexpIntAltrep {
    const GUARD: AltrepGuard = AltrepGuard::RUnwind;

    fn length(x: SEXP) -> R_xlen_t {
        // SAFETY: ALTREP callback — on R's main thread; `x` is a valid ALTREP SEXP.
        // `data1` is the INTSXP we stored in `into_altrep_sexp`.
        let data1 = unsafe { x.altrep_data1_raw_unchecked() };
        data1.xlength()
    }
}

impl AltVec for NativeSexpIntAltrep {
    // Expose `Dataptr` so that `as.integer(v)`, `sum(v)`, bulk operations etc.
    // can use the fast contiguous-memory path.  `data1` IS the materialized
    // INTSXP, so this is trivial — no `data2` materialization step needed.
    const HAS_DATAPTR: bool = true;

    fn dataptr(x: SEXP, _writable: bool) -> *mut core::ffi::c_void {
        // SAFETY: ALTREP callback — R main thread; `x` valid; `data1` valid INTSXP.
        // `DATAPTR_RO(data1)` returns the contiguous integer buffer.  We cast the
        // const pointer to mut to satisfy the `*mut c_void` return type: R may
        // write through this pointer when `writable = true` and the vector is not
        // shared.  Because `data1` is a plain INTSXP we own (NAMED == 1 at
        // construction), writes are safe.
        let data1 = unsafe { x.altrep_data1_raw_unchecked() };
        // `DATAPTR_RO` gives us a read-only pointer; cast to mut for the Dataptr
        // contract.  An empty vector produces a dangling-but-non-null pointer.
        let ro = unsafe { DATAPTR_RO(data1) };
        if ro.is_null() {
            std::ptr::NonNull::<i32>::dangling().as_ptr().cast()
        } else {
            ro.cast_mut()
        }
    }

    const HAS_DATAPTR_OR_NULL: bool = true;

    fn dataptr_or_null(x: SEXP) -> *const core::ffi::c_void {
        // SAFETY: ALTREP callback; `data1` valid INTSXP.
        let data1 = unsafe { x.altrep_data1_raw_unchecked() };
        unsafe { DATAPTR_RO(data1) }
    }
}

impl AltInteger for NativeSexpIntAltrep {
    const HAS_ELT: bool = true;

    fn elt(x: SEXP, i: R_xlen_t) -> i32 {
        // SAFETY: ALTREP callback; `i` bounds-checked by R before dispatch;
        // `data1` is a valid INTSXP protected by the ALTREP SEXP.
        let data1 = unsafe { x.altrep_data1_raw_unchecked() };
        data1.integer_elt(i)
    }
}

// endregion

// region: RegisterAltrep — class registration via OnceLock

impl RegisterAltrep for NativeSexpIntAltrep {
    fn get_or_init_class() -> miniextendr_api::ffi::altrep::R_altrep_class_t {
        static CLASS: OnceLock<miniextendr_api::ffi::altrep::R_altrep_class_t> = OnceLock::new();
        *CLASS.get_or_init(|| {
            const CLASS_NAME: &[u8] = b"NativeSexpIntAltrep\0";
            let cls = unsafe {
                <NativeSexpIntAltrep as miniextendr_api::altrep_data::InferBase>::make_class(
                    CLASS_NAME.as_ptr().cast::<std::ffi::c_char>(),
                    miniextendr_api::AltrepPkgName::as_ptr(),
                )
            };
            unsafe {
                <NativeSexpIntAltrep as miniextendr_api::altrep_data::InferBase>::install_methods(
                    cls,
                );
            }
            cls
        })
    }
}

// endregion

// region: IntoR — build the ALTREP SEXP with data1 = plain INTSXP
//
// The standard macro-generated `IntoR` wraps `self` in an `ExternalPtr`.  For
// the native-SEXP pattern we want `data1` to hold a plain `INTSXP` carrying
// the integer values.  So we provide a manual `IntoR` on a value-carrying
// newtype (the ZST `NativeSexpIntAltrep` itself has nothing to put in IntoR).

impl IntoR for NativeSexpIntAltrep {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    /// Returns an empty (`length(0)`) ALTREP-backed integer vector.
    fn into_sexp(self) -> SEXP {
        build_native_sexp_altrep(&[])
    }
}

/// Build the ALTREP SEXP from a slice of `i32` values.
///
/// Allocates a plain `INTSXP` for `data1`, fills it, then calls
/// `R_new_altrep(class, data1, R_NilValue)`.  `data1` is protected across
/// the `new_altrep` call (which may allocate and trigger GC).
///
/// # Safety
///
/// Must be called from R's main thread.
fn build_native_sexp_altrep(values: &[i32]) -> SEXP {
    let cls = <NativeSexpIntAltrep as RegisterAltrep>::get_or_init_class();
    let n = values.len() as R_xlen_t;
    // SAFETY: on R's main thread; `Rf_protect` before `new_altrep` which may GC.
    unsafe {
        // Allocate and fill data1 (plain INTSXP).
        let data1 = Rf_allocVector(SEXPTYPE::INTSXP, n);
        // Copy values into the data1 buffer via SexpExt::set_integer_elt.
        for (i, &v) in values.iter().enumerate() {
            data1.set_integer_elt(i as isize, v);
        }
        // Protect data1 across new_altrep, which may allocate.
        Rf_protect(data1);
        let altrep = cls.new_altrep(data1, SEXP::nil());
        Rf_unprotect(1);
        altrep
    }
}

// endregion

// region: Exported R functions

/// Create a native-SEXP ALTREP integer vector backed by a plain `INTSXP`.
///
/// Unlike the usual ExternalPtr-backed ALTREP, the backing data (`data1`) is a
/// plain R integer vector stored directly in the ALTREP `data1` slot.
/// `AltrepExtract` is implemented via a static singleton: the ZST
/// `NativeSexpIntAltrep` has no state — all data lives in `data1` on the
/// ALTREP SEXP itself.
///
/// This mirrors R's own `compact_intseq` pattern (see
/// `src/main/altclasses.c` in the R source), where `data1` stores the
/// sequence parameters and all element-access methods derive their results
/// from `data1` directly.
///
/// @param values An integer vector.
/// @return An ALTREP-backed integer vector.
/// @export
#[miniextendr]
pub fn native_sexp_altrep_new(values: Vec<i32>) -> SEXP {
    build_native_sexp_altrep(&values)
}

// endregion
