//! ALTREP owners with `#[r_data] Sidecar<T>` fields, for `test-altrep-sidecar.R`.
use miniextendr_api::altrep_data::{
    AltIntegerData, AltrepLen, materialize_altrep_data2, materialized_altrep_data2,
};
use miniextendr_api::altrep_registration::RegisterAltrep;
use miniextendr_api::altrep_traits::{AltInteger, AltVec, Altrep};
use miniextendr_api::externalptr::Sidecar;
use miniextendr_api::{R_xlen_t, RNativeType, SEXP, SexpExt, miniextendr};

// region: low-level callbacks reading a sidecar

/// Integers read from an R vector that the data1 owner roots in a sidecar.
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
                Some(cache) => miniextendr_api::sys::DATAPTR_RO(cache).cast_mut(),
                None => materialize_altrep_data2::<i32>(x),
            }
        }
    }
    const HAS_DATAPTR_OR_NULL: bool = true;
    fn dataptr_or_null(x: SEXP) -> *const core::ffi::c_void {
        unsafe {
            materialized_altrep_data2(x).map_or(core::ptr::null(), |cache| {
                miniextendr_api::sys::DATAPTR_RO(cache)
            })
        }
    }
}

miniextendr_api::impl_inferbase_integer!(SidecarVector);

/// An ALTREP view of the integer vector `values`, which it keeps alive.
#[miniextendr(noexport)]
pub fn make_sidecar_altrep(values: SEXP) -> SidecarVector {
    assert_eq!(values.type_of(), miniextendr_api::SEXPTYPE::INTSXP);
    SidecarVector {
        values: Sidecar::new(values),
    }
}
// endregion

// region: a clone-policy copy shares the sidecars

/// Integers from a Rust field, with an R value in a sidecar. A clone-policy
/// copy clones the handle, so it holds the same R value.
#[derive(Clone, miniextendr_api::AltrepInteger)]
#[altrep(manual, duplicate = "clone", class = "ClonedSidecarInts")]
pub struct ClonedSidecarInts {
    values: Vec<i32>,
    #[r_data(ref)]
    tag: Sidecar<SEXP>,
}

impl AltrepLen for ClonedSidecarInts {
    fn len(&self) -> usize {
        self.values.len()
    }
}

impl AltIntegerData for ClonedSidecarInts {
    fn elt(&self, i: usize) -> i32 {
        self.values[i]
    }
}

/// An integer ALTREP of `values` whose owner keeps `tag` alive.
#[miniextendr(noexport)]
pub fn altrep_cloned_sidecar_ints(values: Vec<i32>, tag: SEXP) -> ClonedSidecarInts {
    ClonedSidecarInts {
        values,
        tag: Sidecar::new(tag),
    }
}

/// The `tag` sidecar of a vector from `altrep_cloned_sidecar_ints()`.
#[miniextendr(noexport)]
pub fn altrep_cloned_sidecar_tag(x: SEXP) -> SEXP {
    let class = ClonedSidecarInts::get_or_init_class();
    assert!(
        x.is_altrep() && unsafe { class.inherits(x) },
        "not a ClonedSidecarInts vector"
    );
    let owner = unsafe { miniextendr_api::altrep_data1_as::<ClonedSidecarInts>(x) }
        .expect("ClonedSidecarInts owner");
    ClonedSidecarInts::tag(&owner)
}
// endregion
