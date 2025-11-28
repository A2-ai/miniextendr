//! Unsafe ALTREP trampolines and installers bridging safe traits to R's C ABI.
//!
//! This module owns the generic extern "C-unwind" functions (trampolines) that call
//! into the safe trait implementations in `altrep_traits`, and the helpers that
//! register/install only the methods that are implemented (`HAS_*`).

use crate::altrep_traits as traits;
use crate::ffi::altrep::*;
use crate::ffi::*;
use core::ffi::c_void;

// ========= Generic trampolines for Altrep/AltVec families =========

/// # Safety
/// Called by R for an ALTREP instance of the class backed by `T`.
/// `x` must be a valid SEXP of that class. Must run on the R thread.
pub unsafe extern "C-unwind" fn t_length<T: traits::Altrep>(x: SEXP) -> R_xlen_t {
    T::length(x)
}
/// # Safety
/// Called by R; `x` must be a valid ALTREP vector for `T`. Returns a data pointer
/// contingent on writability flag `w`. Must run on the R thread.
pub unsafe extern "C-unwind" fn t_dataptr<T: traits::AltVec>(x: SEXP, w: Rboolean) -> *mut c_void {
    T::dataptr(x, matches!(w, Rboolean::TRUE))
}
/// # Safety
/// Called by R; `x` must be a valid ALTREP vector for `T`. Must run on the R thread.
pub unsafe extern "C-unwind" fn t_dataptr_or_null<T: traits::AltVec>(x: SEXP) -> *const c_void {
    T::dataptr_or_null(x)
}
/// # Safety
/// Called by R; `x` must be a valid ALTREP vector for `T`. `indx` and `call` are valid
/// R objects provided by R. Must run on the R thread.
pub unsafe extern "C-unwind" fn t_extract_subset<T: traits::AltVec>(
    x: SEXP,
    indx: SEXP,
    call: SEXP,
) -> SEXP {
    T::extract_subset(x, indx, call)
}

// Integer family
/// # Safety
/// Called by R; `x` must be a valid ALTREP INTSXP for `T` and `i` within bounds.
pub unsafe extern "C-unwind" fn t_int_elt<T: traits::AltInteger>(x: SEXP, i: R_xlen_t) -> i32 {
    T::elt(x, i)
}
/// # Safety
/// Called by R; `x` must be a valid ALTREP INTSXP and `out` must point to a
/// writable buffer of at least `n` elements. Indices `i..i+n` must be in-bounds
/// or will be truncated per `get_region` semantics. Must run on the R thread.
pub unsafe extern "C-unwind" fn t_int_get_region<T: traits::AltInteger>(
    x: SEXP,
    i: R_xlen_t,
    n: R_xlen_t,
    out: *mut i32,
) -> R_xlen_t {
    T::get_region(x, i, n, out)
}
/// # Safety
/// Called by R; `x` must be a valid ALTREP INTSXP.
pub unsafe extern "C-unwind" fn t_int_is_sorted<T: traits::AltInteger>(x: SEXP) -> i32 {
    T::is_sorted(x)
}
/// # Safety
/// Called by R; `x` must be a valid ALTREP INTSXP.
pub unsafe extern "C-unwind" fn t_int_no_na<T: traits::AltInteger>(x: SEXP) -> i32 {
    T::no_na(x)
}
/// # Safety
/// Called by R; `x` must be a valid ALTREP INTSXP.
pub unsafe extern "C-unwind" fn t_int_sum<T: traits::AltInteger>(x: SEXP, narm: Rboolean) -> SEXP {
    T::sum(x, matches!(narm, Rboolean::TRUE))
}
/// # Safety
/// Called by R; `x` must be a valid ALTREP INTSXP.
pub unsafe extern "C-unwind" fn t_int_min<T: traits::AltInteger>(x: SEXP, narm: Rboolean) -> SEXP {
    T::min(x, matches!(narm, Rboolean::TRUE))
}
/// # Safety
/// Called by R; `x` must be a valid ALTREP INTSXP.
pub unsafe extern "C-unwind" fn t_int_max<T: traits::AltInteger>(x: SEXP, narm: Rboolean) -> SEXP {
    T::max(x, matches!(narm, Rboolean::TRUE))
}

// Real family
/// # Safety
/// Called by R; `x` must be a valid ALTREP REALSXP for `T` and `i` within bounds.
pub unsafe extern "C-unwind" fn t_real_elt<T: traits::AltReal>(x: SEXP, i: R_xlen_t) -> f64 {
    T::elt(x, i)
}
/// # Safety
/// Called by R; `x` must be a valid ALTREP REALSXP and `out` must point to a
/// writable buffer of at least `n` elements. Must run on the R thread.
pub unsafe extern "C-unwind" fn t_real_get_region<T: traits::AltReal>(
    x: SEXP,
    i: R_xlen_t,
    n: R_xlen_t,
    out: *mut f64,
) -> R_xlen_t {
    T::get_region(x, i, n, out)
}
/// # Safety
/// Called by R; `x` must be a valid ALTREP REALSXP.
pub unsafe extern "C-unwind" fn t_real_is_sorted<T: traits::AltReal>(x: SEXP) -> i32 {
    T::is_sorted(x)
}
/// # Safety
/// Called by R; `x` must be a valid ALTREP REALSXP.
pub unsafe extern "C-unwind" fn t_real_no_na<T: traits::AltReal>(x: SEXP) -> i32 {
    T::no_na(x)
}
/// # Safety
/// Called by R; `x` must be a valid ALTREP REALSXP.
pub unsafe extern "C-unwind" fn t_real_sum<T: traits::AltReal>(x: SEXP, narm: Rboolean) -> SEXP {
    T::sum(x, matches!(narm, Rboolean::TRUE))
}
/// # Safety
/// Called by R; `x` must be a valid ALTREP REALSXP.
pub unsafe extern "C-unwind" fn t_real_min<T: traits::AltReal>(x: SEXP, narm: Rboolean) -> SEXP {
    T::min(x, matches!(narm, Rboolean::TRUE))
}
/// # Safety
/// Called by R; `x` must be a valid ALTREP REALSXP.
pub unsafe extern "C-unwind" fn t_real_max<T: traits::AltReal>(x: SEXP, narm: Rboolean) -> SEXP {
    T::max(x, matches!(narm, Rboolean::TRUE))
}

// Logical family
/// # Safety
/// Called by R; `x` must be a valid ALTREP LGLSXP for `T` and `i` within bounds.
pub unsafe extern "C-unwind" fn t_lgl_elt<T: traits::AltLogical>(x: SEXP, i: R_xlen_t) -> i32 {
    T::elt(x, i)
}
/// # Safety
/// Called by R; `x` must be a valid ALTREP LGLSXP and `out` must point to a
/// writable buffer of at least `n` elements. Must run on the R thread.
pub unsafe extern "C-unwind" fn t_lgl_get_region<T: traits::AltLogical>(
    x: SEXP,
    i: R_xlen_t,
    n: R_xlen_t,
    out: *mut i32,
) -> R_xlen_t {
    T::get_region(x, i, n, out)
}
/// # Safety
/// Called by R; `x` must be a valid ALTREP LGLSXP.
pub unsafe extern "C-unwind" fn t_lgl_is_sorted<T: traits::AltLogical>(x: SEXP) -> i32 {
    T::is_sorted(x)
}
/// # Safety
/// Called by R; `x` must be a valid ALTREP LGLSXP.
pub unsafe extern "C-unwind" fn t_lgl_no_na<T: traits::AltLogical>(x: SEXP) -> i32 {
    T::no_na(x)
}

// Raw family
/// # Safety
/// Called by R; `x` must be a valid ALTREP RAWSXP for `T` and `i` within bounds.
pub unsafe extern "C-unwind" fn t_raw_elt<T: traits::AltRaw>(x: SEXP, i: R_xlen_t) -> Rbyte {
    T::elt(x, i) as Rbyte
}
/// # Safety
/// Called by R; `x` must be a valid ALTREP RAWSXP and `out` must point to a
/// writable buffer of at least `n` elements. Must run on the R thread.
pub unsafe extern "C-unwind" fn t_raw_get_region<T: traits::AltRaw>(
    x: SEXP,
    i: R_xlen_t,
    n: R_xlen_t,
    out: *mut Rbyte,
) -> R_xlen_t {
    T::get_region(x, i, n, out)
}

// String family
/// # Safety
/// Called by R; `x` must be a valid ALTREP STRSXP for `T` and `i` within bounds.
pub unsafe extern "C-unwind" fn t_str_elt<T: traits::AltString>(x: SEXP, i: R_xlen_t) -> SEXP {
    T::elt(x, i)
}
/// # Safety
/// Called by R; `x` must be a valid ALTREP STRSXP.
pub unsafe extern "C-unwind" fn t_str_is_sorted<T: traits::AltString>(x: SEXP) -> i32 {
    T::is_sorted(x)
}
/// # Safety
/// Called by R; `x` must be a valid ALTREP STRSXP.
pub unsafe extern "C-unwind" fn t_str_no_na<T: traits::AltString>(x: SEXP) -> i32 {
    T::no_na(x)
}
/// # Safety
/// Called by R; `x` must be a valid ALTREP STRSXP and `v` a valid CHARSXP.
pub unsafe extern "C-unwind" fn t_str_set_elt<T: traits::AltString>(x: SEXP, i: R_xlen_t, v: SEXP) {
    T::set_elt(x, i, v)
}

// List family
/// # Safety
/// Called by R; `x` must be a valid ALTREP VECSXP for `T` and `i` within bounds.
pub unsafe extern "C-unwind" fn t_list_elt<T: traits::AltList>(x: SEXP, i: R_xlen_t) -> SEXP {
    T::elt(x, i)
}
/// # Safety
/// Called by R; `x` must be a valid ALTREP VECSXP and `v` a valid SEXP.
pub unsafe extern "C-unwind" fn t_list_set_elt<T: traits::AltList>(x: SEXP, i: R_xlen_t, v: SEXP) {
    T::set_elt(x, i, v)
}

// ========= Installers per family =========

/// # Safety
/// Must be called during R initialization with a valid ALTREP class handle.
pub unsafe fn install_base<T: traits::Altrep>(cls: R_altrep_class_t) {
    if <T as traits::Altrep>::HAS_LENGTH {
        unsafe { R_set_altrep_Length_method(cls, Some(t_length::<T>)) };
    }
    if <T as traits::Altrep>::HAS_SERIALIZED_STATE {
        unsafe { R_set_altrep_Serialized_state_method(cls, None) };
    }
    if <T as traits::Altrep>::HAS_UNSERIALIZE_EX {
        unsafe { R_set_altrep_UnserializeEX_method(cls, None) };
    }
    if <T as traits::Altrep>::HAS_UNSERIALIZE {
        unsafe { R_set_altrep_Unserialize_method(cls, None) };
    }
    if <T as traits::Altrep>::HAS_DUPLICATE_EX {
        unsafe { R_set_altrep_DuplicateEX_method(cls, None) };
    }
    if <T as traits::Altrep>::HAS_DUPLICATE {
        unsafe { R_set_altrep_Duplicate_method(cls, None) };
    }
    if <T as traits::Altrep>::HAS_COERCE {
        unsafe { R_set_altrep_Coerce_method(cls, None) };
    }
    if <T as traits::Altrep>::HAS_INSPECT {
        unsafe { R_set_altrep_Inspect_method(cls, None) };
    }
}

/// # Safety
/// Must be called during R initialization with a valid ALTREP class handle.
pub unsafe fn install_vec<T: traits::AltVec>(cls: R_altrep_class_t) {
    if <T as traits::AltVec>::HAS_DATAPTR {
        unsafe { R_set_altvec_Dataptr_method(cls, Some(t_dataptr::<T>)) };
    }
    if <T as traits::AltVec>::HAS_DATAPTR_OR_NULL {
        unsafe { R_set_altvec_Dataptr_or_null_method(cls, Some(t_dataptr_or_null::<T>)) };
    }
    if <T as traits::AltVec>::HAS_EXTRACT_SUBSET {
        unsafe { R_set_altvec_Extract_subset_method(cls, Some(t_extract_subset::<T>)) };
    }
}

/// # Safety
/// Must be called during R initialization with a valid ALTREP class handle.
pub unsafe fn install_int<T: traits::AltInteger>(cls: R_altrep_class_t) {
    if <T as traits::AltInteger>::HAS_ELT {
        unsafe { R_set_altinteger_Elt_method(cls, Some(t_int_elt::<T>)) };
    }
    if <T as traits::AltInteger>::HAS_GET_REGION {
        unsafe { R_set_altinteger_Get_region_method(cls, Some(t_int_get_region::<T>)) };
    }
    if <T as traits::AltInteger>::HAS_IS_SORTED {
        unsafe { R_set_altinteger_Is_sorted_method(cls, Some(t_int_is_sorted::<T>)) };
    }
    if <T as traits::AltInteger>::HAS_NO_NA {
        unsafe { R_set_altinteger_No_NA_method(cls, Some(t_int_no_na::<T>)) };
    }
    if <T as traits::AltInteger>::HAS_SUM {
        unsafe { R_set_altinteger_Sum_method(cls, Some(t_int_sum::<T>)) };
    }
    if <T as traits::AltInteger>::HAS_MIN {
        unsafe { R_set_altinteger_Min_method(cls, Some(t_int_min::<T>)) };
    }
    if <T as traits::AltInteger>::HAS_MAX {
        unsafe { R_set_altinteger_Max_method(cls, Some(t_int_max::<T>)) };
    }
}

/// # Safety
/// Must be called during R initialization with a valid ALTREP class handle.
pub unsafe fn install_real<T: traits::AltReal>(cls: R_altrep_class_t) {
    if <T as traits::AltReal>::HAS_ELT {
        unsafe { R_set_altreal_Elt_method(cls, Some(t_real_elt::<T>)) };
    }
    if <T as traits::AltReal>::HAS_GET_REGION {
        unsafe { R_set_altreal_Get_region_method(cls, Some(t_real_get_region::<T>)) };
    }
    if <T as traits::AltReal>::HAS_IS_SORTED {
        unsafe { R_set_altreal_Is_sorted_method(cls, Some(t_real_is_sorted::<T>)) };
    }
    if <T as traits::AltReal>::HAS_NO_NA {
        unsafe { R_set_altreal_No_NA_method(cls, Some(t_real_no_na::<T>)) };
    }
    if <T as traits::AltReal>::HAS_SUM {
        unsafe { R_set_altreal_Sum_method(cls, Some(t_real_sum::<T>)) };
    }
    if <T as traits::AltReal>::HAS_MIN {
        unsafe { R_set_altreal_Min_method(cls, Some(t_real_min::<T>)) };
    }
    if <T as traits::AltReal>::HAS_MAX {
        unsafe { R_set_altreal_Max_method(cls, Some(t_real_max::<T>)) };
    }
}

/// # Safety
/// Must be called during R initialization with a valid ALTREP class handle.
pub unsafe fn install_lgl<T: traits::AltLogical>(cls: R_altrep_class_t) {
    if <T as traits::AltLogical>::HAS_ELT {
        unsafe { R_set_altlogical_Elt_method(cls, Some(t_lgl_elt::<T>)) };
    }
    if <T as traits::AltLogical>::HAS_GET_REGION {
        unsafe { R_set_altlogical_Get_region_method(cls, Some(t_lgl_get_region::<T>)) };
    }
    if <T as traits::AltLogical>::HAS_IS_SORTED {
        unsafe { R_set_altlogical_Is_sorted_method(cls, Some(t_lgl_is_sorted::<T>)) };
    }
    if <T as traits::AltLogical>::HAS_NO_NA {
        unsafe { R_set_altlogical_No_NA_method(cls, Some(t_lgl_no_na::<T>)) };
    }
}

/// # Safety
/// Must be called during R initialization with a valid ALTREP class handle.
pub unsafe fn install_raw<T: traits::AltRaw>(cls: R_altrep_class_t) {
    if <T as traits::AltRaw>::HAS_ELT {
        unsafe { R_set_altraw_Elt_method(cls, Some(t_raw_elt::<T>)) };
    }
    if <T as traits::AltRaw>::HAS_GET_REGION {
        unsafe { R_set_altraw_Get_region_method(cls, Some(t_raw_get_region::<T>)) };
    }
}

/// # Safety
/// Must be called during R initialization with a valid ALTREP class handle.
pub unsafe fn install_str<T: traits::AltString>(cls: R_altrep_class_t) {
    if <T as traits::AltString>::HAS_ELT {
        unsafe { R_set_altstring_Elt_method(cls, Some(t_str_elt::<T>)) };
    }
    if <T as traits::AltString>::HAS_IS_SORTED {
        unsafe { R_set_altstring_Is_sorted_method(cls, Some(t_str_is_sorted::<T>)) };
    }
    if <T as traits::AltString>::HAS_NO_NA {
        unsafe { R_set_altstring_No_NA_method(cls, Some(t_str_no_na::<T>)) };
    }
    if <T as traits::AltString>::HAS_SET_ELT {
        unsafe { R_set_altstring_Set_elt_method(cls, Some(t_str_set_elt::<T>)) };
    }
}

/// # Safety
/// Must be called during R initialization with a valid ALTREP class handle.
pub unsafe fn install_list<T: traits::AltList>(cls: R_altrep_class_t) {
    if <T as traits::AltList>::HAS_ELT {
        unsafe { R_set_altlist_Elt_method(cls, Some(t_list_elt::<T>)) };
    }
    if <T as traits::AltList>::HAS_SET_ELT {
        unsafe { R_set_altlist_Set_elt_method(cls, Some(t_list_set_elt::<T>)) };
    }
}
