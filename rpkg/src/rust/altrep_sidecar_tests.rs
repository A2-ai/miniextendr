//! ALTREP callbacks read R-rooted fields through the explicit data1 owner.
use miniextendr_api::altrep_data::{materialize_altrep_data2, materialized_altrep_data2};
use miniextendr_api::altrep_traits::{AltInteger, AltVec, Altrep};
use miniextendr_api::externalptr::Sidecar;
use miniextendr_api::{R_xlen_t, RNativeType, SEXP, SexpExt, miniextendr};

#[derive(miniextendr_api::AltrepInteger)]
#[altrep(manual, no_lowlevel)]
pub struct SidecarVector {
    #[r_data(ref)]
    values: Sidecar<SEXP>,
}

impl SidecarVector {
    fn source(x: SEXP) -> SEXP {
        if let Some(cache) = unsafe { materialized_altrep_data2(x) } {
            return cache;
        }
        let owner = unsafe { miniextendr_api::altrep_data1_as_unchecked::<Self>(x) }
            .expect("SidecarVector data1 owner");
        Self::values(&owner)
    }
}

impl Altrep for SidecarVector {
    fn length(x: SEXP) -> R_xlen_t {
        R_xlen_t::try_from(Self::source(x).len()).unwrap()
    }
}

impl AltInteger for SidecarVector {
    const HAS_ELT: bool = true;
    fn elt(x: SEXP, i: R_xlen_t) -> i32 {
        i32::elt(Self::source(x), i)
    }
}

impl AltVec for SidecarVector {
    const HAS_DATAPTR: bool = true;
    fn dataptr(x: SEXP, _writable: bool) -> *mut core::ffi::c_void {
        // Never expose the sidecar's possibly shared vector as writable.
        // Once copied, data2 is authoritative for length and element reads.
        unsafe {
            match materialized_altrep_data2(x) {
                Some(cache) => miniextendr_api::sys::DATAPTR_RO_unchecked(cache).cast_mut(),
                None => materialize_altrep_data2::<i32>(x),
            }
        }
    }
    const HAS_DATAPTR_OR_NULL: bool = true;
    fn dataptr_or_null(x: SEXP) -> *const core::ffi::c_void {
        unsafe {
            materialized_altrep_data2(x).map_or(core::ptr::null(), |cache| {
                miniextendr_api::sys::DATAPTR_RO_unchecked(cache)
            })
        }
    }
}

miniextendr_api::impl_inferbase_integer!(SidecarVector);

/// @noRd
#[miniextendr(noexport)]
pub fn make_sidecar_altrep(values: SEXP) -> SidecarVector {
    assert_eq!(values.type_of(), miniextendr_api::SEXPTYPE::INTSXP);
    SidecarVector {
        values: Sidecar::new(values),
    }
}
