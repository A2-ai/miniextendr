//! Unsafe ALTREP trampolines and installers bridging safe traits to R's C ABI.
//!
//! This module provides:
//! - Generic `extern "C-unwind"` trampolines that call into safe trait methods
//! - Installer functions that register methods with R based on `HAS_*` consts
//!
//! ## Design
//!
//! Trampolines are only installed when `HAS_*` is true. When false, the method
//! is NOT installed with R, so R uses its own default behavior.

use crate::altrep_traits::{
    AltComplex, AltInteger, AltList, AltLogical, AltRaw, AltReal, AltString, AltVec, Altrep,
};
use crate::ffi::altrep::*;
use crate::ffi::*;
use core::ffi::c_void;

// =============================================================================
// ALTREP BASE TRAMPOLINES
// =============================================================================

/// Trampoline for Length method.
/// # Safety
/// `x` must be a valid SEXP for the ALTREP class backed by `T`.
pub unsafe extern "C-unwind" fn t_length<T: Altrep>(x: SEXP) -> R_xlen_t {
    T::length(x)
}

/// Trampoline for Duplicate method.
/// # Safety
/// `x` must be a valid SEXP for the ALTREP class backed by `T`.
pub unsafe extern "C-unwind" fn t_duplicate<T: Altrep>(x: SEXP, deep: Rboolean) -> SEXP {
    T::duplicate(x, matches!(deep, Rboolean::TRUE))
}

/// Trampoline for DuplicateEX method (extended duplication).
/// # Safety
/// `x` must be a valid SEXP for the ALTREP class backed by `T`.
pub unsafe extern "C-unwind" fn t_duplicate_ex<T: Altrep>(x: SEXP, deep: Rboolean) -> SEXP {
    T::duplicate_ex(x, matches!(deep, Rboolean::TRUE))
}

/// Trampoline for Inspect method.
/// # Safety
/// `x` must be a valid SEXP for the ALTREP class backed by `T`.
pub unsafe extern "C-unwind" fn t_inspect<T: Altrep>(
    x: SEXP,
    pre: i32,
    deep: i32,
    pvec: i32,
    inspect_subtree: Option<unsafe extern "C-unwind" fn(SEXP, i32, i32, i32)>,
) -> Rboolean {
    if T::inspect(x, pre, deep, pvec, inspect_subtree) {
        Rboolean::TRUE
    } else {
        Rboolean::FALSE
    }
}

/// Trampoline for Serialized_state method.
/// # Safety
/// `x` must be a valid SEXP for the ALTREP class backed by `T`.
pub unsafe extern "C-unwind" fn t_serialized_state<T: Altrep>(x: SEXP) -> SEXP {
    T::serialized_state(x)
}

/// Trampoline for Unserialize method.
/// # Safety
/// `class` and `state` must be valid SEXPs from R.
pub unsafe extern "C-unwind" fn t_unserialize<T: Altrep>(class: SEXP, state: SEXP) -> SEXP {
    T::unserialize(class, state)
}

/// Trampoline for UnserializeEX method (extended unserialization with attributes).
/// # Safety
/// `class`, `state`, and `attr` must be valid SEXPs from R.
pub unsafe extern "C-unwind" fn t_unserialize_ex<T: Altrep>(
    class: SEXP,
    state: SEXP,
    attr: SEXP,
    objf: ::std::os::raw::c_int,
    levs: ::std::os::raw::c_int,
) -> SEXP {
    T::unserialize_ex(class, state, attr, objf, levs)
}

/// Trampoline for Coerce method.
/// # Safety
/// `x` must be a valid SEXP for the ALTREP class backed by `T`.
pub unsafe extern "C-unwind" fn t_coerce<T: Altrep>(x: SEXP, to_type: SEXPTYPE) -> SEXP {
    T::coerce(x, to_type)
}

// =============================================================================
// ALTVEC TRAMPOLINES
// =============================================================================

/// Trampoline for Dataptr method.
/// # Safety
/// `x` must be a valid SEXP for the ALTREP class backed by `T`.
pub unsafe extern "C-unwind" fn t_dataptr<T: AltVec>(x: SEXP, w: Rboolean) -> *mut c_void {
    T::dataptr(x, matches!(w, Rboolean::TRUE))
}

/// Trampoline for Dataptr_or_null method.
/// # Safety
/// `x` must be a valid SEXP for the ALTREP class backed by `T`.
pub unsafe extern "C-unwind" fn t_dataptr_or_null<T: AltVec>(x: SEXP) -> *const c_void {
    T::dataptr_or_null(x)
}

/// Trampoline for Extract_subset method.
/// # Safety
/// `x`, `indx`, and `call` must be valid SEXPs.
pub unsafe extern "C-unwind" fn t_extract_subset<T: AltVec>(
    x: SEXP,
    indx: SEXP,
    call: SEXP,
) -> SEXP {
    T::extract_subset(x, indx, call)
}

// =============================================================================
// ALTINTEGER TRAMPOLINES
// =============================================================================

/// Trampoline for integer Elt method.
/// # Safety
/// `x` must be a valid ALTREP INTSXP and `i` within bounds.
pub unsafe extern "C-unwind" fn t_int_elt<T: AltInteger>(x: SEXP, i: R_xlen_t) -> i32 {
    T::elt(x, i)
}

/// Trampoline for integer Get_region method.
/// # Safety
/// `x` must be a valid ALTREP INTSXP and `out` a valid buffer of at least `n` elements.
pub unsafe extern "C-unwind" fn t_int_get_region<T: AltInteger>(
    x: SEXP,
    i: R_xlen_t,
    n: R_xlen_t,
    out: *mut i32,
) -> R_xlen_t {
    T::get_region(x, i, n, out)
}

/// Trampoline for integer Is_sorted method.
/// # Safety
/// `x` must be a valid ALTREP INTSXP.
pub unsafe extern "C-unwind" fn t_int_is_sorted<T: AltInteger>(x: SEXP) -> i32 {
    T::is_sorted(x)
}

/// Trampoline for integer No_NA method.
/// # Safety
/// `x` must be a valid ALTREP INTSXP.
pub unsafe extern "C-unwind" fn t_int_no_na<T: AltInteger>(x: SEXP) -> i32 {
    T::no_na(x)
}

/// Trampoline for integer Sum method.
/// # Safety
/// `x` must be a valid ALTREP INTSXP.
pub unsafe extern "C-unwind" fn t_int_sum<T: AltInteger>(x: SEXP, narm: Rboolean) -> SEXP {
    T::sum(x, matches!(narm, Rboolean::TRUE))
}

/// Trampoline for integer Min method.
/// # Safety
/// `x` must be a valid ALTREP INTSXP.
pub unsafe extern "C-unwind" fn t_int_min<T: AltInteger>(x: SEXP, narm: Rboolean) -> SEXP {
    T::min(x, matches!(narm, Rboolean::TRUE))
}

/// Trampoline for integer Max method.
/// # Safety
/// `x` must be a valid ALTREP INTSXP.
pub unsafe extern "C-unwind" fn t_int_max<T: AltInteger>(x: SEXP, narm: Rboolean) -> SEXP {
    T::max(x, matches!(narm, Rboolean::TRUE))
}

// =============================================================================
// ALTREAL TRAMPOLINES
// =============================================================================

/// Trampoline for real Elt method.
/// # Safety
/// `x` must be a valid ALTREP REALSXP and `i` within bounds.
pub unsafe extern "C-unwind" fn t_real_elt<T: AltReal>(x: SEXP, i: R_xlen_t) -> f64 {
    T::elt(x, i)
}

/// Trampoline for real Get_region method.
/// # Safety
/// `x` must be a valid ALTREP REALSXP and `out` a valid buffer of at least `n` elements.
pub unsafe extern "C-unwind" fn t_real_get_region<T: AltReal>(
    x: SEXP,
    i: R_xlen_t,
    n: R_xlen_t,
    out: *mut f64,
) -> R_xlen_t {
    T::get_region(x, i, n, out)
}

/// Trampoline for real Is_sorted method.
/// # Safety
/// `x` must be a valid ALTREP REALSXP.
pub unsafe extern "C-unwind" fn t_real_is_sorted<T: AltReal>(x: SEXP) -> i32 {
    T::is_sorted(x)
}

/// Trampoline for real No_NA method.
/// # Safety
/// `x` must be a valid ALTREP REALSXP.
pub unsafe extern "C-unwind" fn t_real_no_na<T: AltReal>(x: SEXP) -> i32 {
    T::no_na(x)
}

/// Trampoline for real Sum method.
/// # Safety
/// `x` must be a valid ALTREP REALSXP.
pub unsafe extern "C-unwind" fn t_real_sum<T: AltReal>(x: SEXP, narm: Rboolean) -> SEXP {
    T::sum(x, matches!(narm, Rboolean::TRUE))
}

/// Trampoline for real Min method.
/// # Safety
/// `x` must be a valid ALTREP REALSXP.
pub unsafe extern "C-unwind" fn t_real_min<T: AltReal>(x: SEXP, narm: Rboolean) -> SEXP {
    T::min(x, matches!(narm, Rboolean::TRUE))
}

/// Trampoline for real Max method.
/// # Safety
/// `x` must be a valid ALTREP REALSXP.
pub unsafe extern "C-unwind" fn t_real_max<T: AltReal>(x: SEXP, narm: Rboolean) -> SEXP {
    T::max(x, matches!(narm, Rboolean::TRUE))
}

// =============================================================================
// ALTLOGICAL TRAMPOLINES
// =============================================================================

/// Trampoline for logical Elt method.
/// # Safety
/// `x` must be a valid ALTREP LGLSXP and `i` within bounds.
pub unsafe extern "C-unwind" fn t_lgl_elt<T: AltLogical>(x: SEXP, i: R_xlen_t) -> i32 {
    T::elt(x, i)
}

/// Trampoline for logical Get_region method.
/// # Safety
/// `x` must be a valid ALTREP LGLSXP and `out` a valid buffer of at least `n` elements.
pub unsafe extern "C-unwind" fn t_lgl_get_region<T: AltLogical>(
    x: SEXP,
    i: R_xlen_t,
    n: R_xlen_t,
    out: *mut i32,
) -> R_xlen_t {
    T::get_region(x, i, n, out)
}

/// Trampoline for logical Is_sorted method.
/// # Safety
/// `x` must be a valid ALTREP LGLSXP.
pub unsafe extern "C-unwind" fn t_lgl_is_sorted<T: AltLogical>(x: SEXP) -> i32 {
    T::is_sorted(x)
}

/// Trampoline for logical No_NA method.
/// # Safety
/// `x` must be a valid ALTREP LGLSXP.
pub unsafe extern "C-unwind" fn t_lgl_no_na<T: AltLogical>(x: SEXP) -> i32 {
    T::no_na(x)
}

/// Trampoline for logical Sum method.
/// # Safety
/// `x` must be a valid ALTREP LGLSXP.
pub unsafe extern "C-unwind" fn t_lgl_sum<T: AltLogical>(x: SEXP, narm: Rboolean) -> SEXP {
    T::sum(x, matches!(narm, Rboolean::TRUE))
}

// Note: R's ALTREP API does not expose min/max for logical vectors

// =============================================================================
// ALTRAW TRAMPOLINES
// =============================================================================

/// Trampoline for raw Elt method.
/// # Safety
/// `x` must be a valid ALTREP RAWSXP and `i` within bounds.
pub unsafe extern "C-unwind" fn t_raw_elt<T: AltRaw>(x: SEXP, i: R_xlen_t) -> Rbyte {
    T::elt(x, i)
}

/// Trampoline for raw Get_region method.
/// # Safety
/// `x` must be a valid ALTREP RAWSXP and `out` a valid buffer of at least `n` elements.
pub unsafe extern "C-unwind" fn t_raw_get_region<T: AltRaw>(
    x: SEXP,
    i: R_xlen_t,
    n: R_xlen_t,
    out: *mut Rbyte,
) -> R_xlen_t {
    T::get_region(x, i, n, out)
}

// =============================================================================
// ALTCOMPLEX TRAMPOLINES
// =============================================================================

/// Trampoline for complex Elt method.
/// # Safety
/// `x` must be a valid ALTREP CPLXSXP and `i` within bounds.
pub unsafe extern "C-unwind" fn t_cplx_elt<T: AltComplex>(x: SEXP, i: R_xlen_t) -> Rcomplex {
    T::elt(x, i)
}

/// Trampoline for complex Get_region method.
/// # Safety
/// `x` must be a valid ALTREP CPLXSXP and `out` a valid buffer of at least `n` elements.
pub unsafe extern "C-unwind" fn t_cplx_get_region<T: AltComplex>(
    x: SEXP,
    i: R_xlen_t,
    n: R_xlen_t,
    out: *mut Rcomplex,
) -> R_xlen_t {
    T::get_region(x, i, n, out)
}

// =============================================================================
// ALTSTRING TRAMPOLINES
// =============================================================================

/// Trampoline for string Elt method (REQUIRED for ALTSTRING).
/// # Safety
/// `x` must be a valid ALTREP STRSXP and `i` within bounds.
pub unsafe extern "C-unwind" fn t_str_elt<T: AltString>(x: SEXP, i: R_xlen_t) -> SEXP {
    T::elt(x, i)
}

/// Trampoline for string Set_elt method.
/// # Safety
/// `x` must be a valid ALTREP STRSXP and `v` a valid CHARSXP.
pub unsafe extern "C-unwind" fn t_str_set_elt<T: AltString>(x: SEXP, i: R_xlen_t, v: SEXP) {
    T::set_elt(x, i, v)
}

/// Trampoline for string Is_sorted method.
/// # Safety
/// `x` must be a valid ALTREP STRSXP.
pub unsafe extern "C-unwind" fn t_str_is_sorted<T: AltString>(x: SEXP) -> i32 {
    T::is_sorted(x)
}

/// Trampoline for string No_NA method.
/// # Safety
/// `x` must be a valid ALTREP STRSXP.
pub unsafe extern "C-unwind" fn t_str_no_na<T: AltString>(x: SEXP) -> i32 {
    T::no_na(x)
}

// =============================================================================
// ALTLIST TRAMPOLINES
// =============================================================================

/// Trampoline for list Elt method (REQUIRED for ALTLIST).
/// # Safety
/// `x` must be a valid ALTREP VECSXP and `i` within bounds.
pub unsafe extern "C-unwind" fn t_list_elt<T: AltList>(x: SEXP, i: R_xlen_t) -> SEXP {
    T::elt(x, i)
}

/// Trampoline for list Set_elt method.
/// # Safety
/// `x` must be a valid ALTREP VECSXP and `v` a valid SEXP.
pub unsafe extern "C-unwind" fn t_list_set_elt<T: AltList>(x: SEXP, i: R_xlen_t, v: SEXP) {
    T::set_elt(x, i, v)
}

// =============================================================================
// INSTALLERS - Only install methods where HAS_* = true
// =============================================================================

/// Install base ALTREP methods (always installs length, conditionally installs optional).
/// # Safety
/// Must be called during R initialization with a valid ALTREP class handle.
pub unsafe fn install_base<T: Altrep>(cls: R_altrep_class_t) {
    // Length is ALWAYS installed (required)
    unsafe { R_set_altrep_Length_method(cls, Some(t_length::<T>)) };

    // Optional methods - only install if HAS_* = true
    if T::HAS_SERIALIZED_STATE {
        unsafe { R_set_altrep_Serialized_state_method(cls, Some(t_serialized_state::<T>)) };
    }
    if T::HAS_UNSERIALIZE {
        unsafe { R_set_altrep_Unserialize_method(cls, Some(t_unserialize::<T>)) };
    }
    if T::HAS_UNSERIALIZE_EX {
        unsafe { R_set_altrep_UnserializeEX_method(cls, Some(t_unserialize_ex::<T>)) };
    }
    if T::HAS_DUPLICATE {
        unsafe { R_set_altrep_Duplicate_method(cls, Some(t_duplicate::<T>)) };
    }
    if T::HAS_DUPLICATE_EX {
        unsafe { R_set_altrep_DuplicateEX_method(cls, Some(t_duplicate_ex::<T>)) };
    }
    if T::HAS_COERCE {
        unsafe { R_set_altrep_Coerce_method(cls, Some(t_coerce::<T>)) };
    }
    if T::HAS_INSPECT {
        unsafe { R_set_altrep_Inspect_method(cls, Some(t_inspect::<T>)) };
    }
}

/// Install vector-level methods.
/// # Safety
/// Must be called during R initialization with a valid ALTREP class handle.
pub unsafe fn install_vec<T: AltVec>(cls: R_altrep_class_t) {
    if T::HAS_DATAPTR {
        unsafe { R_set_altvec_Dataptr_method(cls, Some(t_dataptr::<T>)) };
    }
    if T::HAS_DATAPTR_OR_NULL {
        unsafe { R_set_altvec_Dataptr_or_null_method(cls, Some(t_dataptr_or_null::<T>)) };
    }
    if T::HAS_EXTRACT_SUBSET {
        unsafe { R_set_altvec_Extract_subset_method(cls, Some(t_extract_subset::<T>)) };
    }
}

/// Install integer-specific methods.
/// # Safety
/// Must be called during R initialization with a valid ALTREP class handle.
pub unsafe fn install_int<T: AltInteger>(cls: R_altrep_class_t) {
    if T::HAS_ELT {
        unsafe { R_set_altinteger_Elt_method(cls, Some(t_int_elt::<T>)) };
    }
    if T::HAS_GET_REGION {
        unsafe { R_set_altinteger_Get_region_method(cls, Some(t_int_get_region::<T>)) };
    }
    if T::HAS_IS_SORTED {
        unsafe { R_set_altinteger_Is_sorted_method(cls, Some(t_int_is_sorted::<T>)) };
    }
    if T::HAS_NO_NA {
        unsafe { R_set_altinteger_No_NA_method(cls, Some(t_int_no_na::<T>)) };
    }
    if T::HAS_SUM {
        unsafe { R_set_altinteger_Sum_method(cls, Some(t_int_sum::<T>)) };
    }
    if T::HAS_MIN {
        unsafe { R_set_altinteger_Min_method(cls, Some(t_int_min::<T>)) };
    }
    if T::HAS_MAX {
        unsafe { R_set_altinteger_Max_method(cls, Some(t_int_max::<T>)) };
    }
}

/// Install real-specific methods.
/// # Safety
/// Must be called during R initialization with a valid ALTREP class handle.
pub unsafe fn install_real<T: AltReal>(cls: R_altrep_class_t) {
    if T::HAS_ELT {
        unsafe { R_set_altreal_Elt_method(cls, Some(t_real_elt::<T>)) };
    }
    if T::HAS_GET_REGION {
        unsafe { R_set_altreal_Get_region_method(cls, Some(t_real_get_region::<T>)) };
    }
    if T::HAS_IS_SORTED {
        unsafe { R_set_altreal_Is_sorted_method(cls, Some(t_real_is_sorted::<T>)) };
    }
    if T::HAS_NO_NA {
        unsafe { R_set_altreal_No_NA_method(cls, Some(t_real_no_na::<T>)) };
    }
    if T::HAS_SUM {
        unsafe { R_set_altreal_Sum_method(cls, Some(t_real_sum::<T>)) };
    }
    if T::HAS_MIN {
        unsafe { R_set_altreal_Min_method(cls, Some(t_real_min::<T>)) };
    }
    if T::HAS_MAX {
        unsafe { R_set_altreal_Max_method(cls, Some(t_real_max::<T>)) };
    }
}

/// Install logical-specific methods.
/// # Safety
/// Must be called during R initialization with a valid ALTREP class handle.
pub unsafe fn install_lgl<T: AltLogical>(cls: R_altrep_class_t) {
    if T::HAS_ELT {
        unsafe { R_set_altlogical_Elt_method(cls, Some(t_lgl_elt::<T>)) };
    }
    if T::HAS_GET_REGION {
        unsafe { R_set_altlogical_Get_region_method(cls, Some(t_lgl_get_region::<T>)) };
    }
    if T::HAS_IS_SORTED {
        unsafe { R_set_altlogical_Is_sorted_method(cls, Some(t_lgl_is_sorted::<T>)) };
    }
    if T::HAS_NO_NA {
        unsafe { R_set_altlogical_No_NA_method(cls, Some(t_lgl_no_na::<T>)) };
    }
    if T::HAS_SUM {
        unsafe { R_set_altlogical_Sum_method(cls, Some(t_lgl_sum::<T>)) };
    }
    // Note: R's ALTREP API does not expose min/max for logical vectors
}

/// Install raw-specific methods.
/// # Safety
/// Must be called during R initialization with a valid ALTREP class handle.
pub unsafe fn install_raw<T: AltRaw>(cls: R_altrep_class_t) {
    if T::HAS_ELT {
        unsafe { R_set_altraw_Elt_method(cls, Some(t_raw_elt::<T>)) };
    }
    if T::HAS_GET_REGION {
        unsafe { R_set_altraw_Get_region_method(cls, Some(t_raw_get_region::<T>)) };
    }
}

/// Install complex-specific methods.
/// # Safety
/// Must be called during R initialization with a valid ALTREP class handle.
pub unsafe fn install_cplx<T: AltComplex>(cls: R_altrep_class_t) {
    if T::HAS_ELT {
        unsafe { R_set_altcomplex_Elt_method(cls, Some(t_cplx_elt::<T>)) };
    }
    if T::HAS_GET_REGION {
        unsafe { R_set_altcomplex_Get_region_method(cls, Some(t_cplx_get_region::<T>)) };
    }
}

/// Install string-specific methods.
/// # Safety
/// Must be called during R initialization with a valid ALTREP class handle.
/// Note: Elt is always installed for ALTSTRING (required).
pub unsafe fn install_str<T: AltString>(cls: R_altrep_class_t) {
    // Elt is ALWAYS installed (required for ALTSTRING)
    unsafe { R_set_altstring_Elt_method(cls, Some(t_str_elt::<T>)) };

    // Optional methods
    if T::HAS_SET_ELT {
        unsafe { R_set_altstring_Set_elt_method(cls, Some(t_str_set_elt::<T>)) };
    }
    if T::HAS_IS_SORTED {
        unsafe { R_set_altstring_Is_sorted_method(cls, Some(t_str_is_sorted::<T>)) };
    }
    if T::HAS_NO_NA {
        unsafe { R_set_altstring_No_NA_method(cls, Some(t_str_no_na::<T>)) };
    }
}

/// Install list-specific methods.
/// # Safety
/// Must be called during R initialization with a valid ALTREP class handle.
/// Note: Elt is always installed for ALTLIST (required).
pub unsafe fn install_list<T: AltList>(cls: R_altrep_class_t) {
    // Elt is ALWAYS installed (required for ALTLIST)
    unsafe { R_set_altlist_Elt_method(cls, Some(t_list_elt::<T>)) };

    // Optional methods
    if T::HAS_SET_ELT {
        unsafe { R_set_altlist_Set_elt_method(cls, Some(t_list_set_elt::<T>)) };
    }
}
