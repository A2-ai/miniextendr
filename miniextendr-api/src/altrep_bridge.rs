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
    AltrepGuard,
};
use crate::ffi::altrep::*;
use crate::ffi::*;
use core::ffi::c_void;

/// Dispatch an ALTREP callback through the guard mode selected by `T::GUARD`.
///
/// Since `T::GUARD` is a const, the compiler eliminates the unreachable branches
/// at monomorphization time — zero runtime overhead for the chosen mode.
///
/// - `Unsafe`: No protection — the closure runs directly.
/// - `RustUnwind`: Wraps in [`catch_unwind`](std::panic::catch_unwind), converting
///   panics to `Rf_error` so they don't unwind through C frames.
/// - `RUnwind`: Wraps in [`R_UnwindProtect`](crate::ffi::R_UnwindProtect), catching
///   both Rust panics and R longjmps safely.
#[inline(always)]
fn guarded_altrep_call<T: Altrep, F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    match T::GUARD {
        AltrepGuard::Unsafe => f(),
        AltrepGuard::RustUnwind => crate::ffi_guard::guarded_ffi_call(
            f,
            crate::ffi_guard::GuardMode::CatchUnwind,
            crate::panic_telemetry::PanicSource::Altrep,
        ),
        AltrepGuard::RUnwind => crate::ffi_guard::guarded_ffi_call(
            f,
            crate::ffi_guard::GuardMode::RUnwind,
            crate::panic_telemetry::PanicSource::Altrep,
        ),
    }
}

// region: ALTREP BASE TRAMPOLINES

/// Trampoline for Length method.
/// # Safety
/// `x` must be a valid SEXP for the ALTREP class backed by `T`.
pub unsafe extern "C-unwind" fn t_length<T: Altrep>(x: SEXP) -> R_xlen_t {
    guarded_altrep_call::<T, _, _>(|| T::length(x))
}

/// Trampoline for Duplicate method.
/// # Safety
/// `x` must be a valid SEXP for the ALTREP class backed by `T`.
pub unsafe extern "C-unwind" fn t_duplicate<T: Altrep>(x: SEXP, deep: Rboolean) -> SEXP {
    guarded_altrep_call::<T, _, _>(|| T::duplicate(x, matches!(deep, Rboolean::TRUE)))
}

/// Trampoline for DuplicateEX method (extended duplication).
/// # Safety
/// `x` must be a valid SEXP for the ALTREP class backed by `T`.
pub unsafe extern "C-unwind" fn t_duplicate_ex<T: Altrep>(x: SEXP, deep: Rboolean) -> SEXP {
    guarded_altrep_call::<T, _, _>(|| T::duplicate_ex(x, matches!(deep, Rboolean::TRUE)))
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
    guarded_altrep_call::<T, _, _>(|| {
        if T::inspect(x, pre, deep, pvec, inspect_subtree) {
            Rboolean::TRUE
        } else {
            Rboolean::FALSE
        }
    })
}

/// Trampoline for Serialized_state method.
/// # Safety
/// `x` must be a valid SEXP for the ALTREP class backed by `T`.
pub unsafe extern "C-unwind" fn t_serialized_state<T: Altrep>(x: SEXP) -> SEXP {
    guarded_altrep_call::<T, _, _>(|| T::serialized_state(x))
}

/// Trampoline for Unserialize method.
/// # Safety
/// `class` and `state` must be valid SEXPs from R.
pub unsafe extern "C-unwind" fn t_unserialize<T: Altrep>(class: SEXP, state: SEXP) -> SEXP {
    guarded_altrep_call::<T, _, _>(|| T::unserialize(class, state))
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
    guarded_altrep_call::<T, _, _>(|| T::unserialize_ex(class, state, attr, objf, levs))
}

/// Trampoline for Coerce method.
/// # Safety
/// `x` must be a valid SEXP for the ALTREP class backed by `T`.
pub unsafe extern "C-unwind" fn t_coerce<T: Altrep>(x: SEXP, to_type: SEXPTYPE) -> SEXP {
    guarded_altrep_call::<T, _, _>(|| T::coerce(x, to_type))
}
// endregion

// region: ALTVEC TRAMPOLINES

/// Trampoline for Dataptr method.
/// # Safety
/// `x` must be a valid SEXP for the ALTREP class backed by `T`.
pub unsafe extern "C-unwind" fn t_dataptr<T: AltVec>(x: SEXP, w: Rboolean) -> *mut c_void {
    guarded_altrep_call::<T, _, _>(|| T::dataptr(x, matches!(w, Rboolean::TRUE)))
}

/// Trampoline for Dataptr_or_null method.
/// # Safety
/// `x` must be a valid SEXP for the ALTREP class backed by `T`.
pub unsafe extern "C-unwind" fn t_dataptr_or_null<T: AltVec>(x: SEXP) -> *const c_void {
    guarded_altrep_call::<T, _, _>(|| T::dataptr_or_null(x))
}

/// Trampoline for Extract_subset method.
/// # Safety
/// `x`, `indx`, and `call` must be valid SEXPs.
pub unsafe extern "C-unwind" fn t_extract_subset<T: AltVec>(
    x: SEXP,
    indx: SEXP,
    call: SEXP,
) -> SEXP {
    guarded_altrep_call::<T, _, _>(|| T::extract_subset(x, indx, call))
}
// endregion

// region: ALTINTEGER TRAMPOLINES

/// Trampoline for integer Elt method.
/// # Safety
/// `x` must be a valid ALTREP INTSXP and `i` within bounds.
pub unsafe extern "C-unwind" fn t_int_elt<T: AltInteger>(x: SEXP, i: R_xlen_t) -> i32 {
    guarded_altrep_call::<T, _, _>(|| T::elt(x, i))
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
    guarded_altrep_call::<T, _, _>(|| T::get_region(x, i, n, out))
}

/// Trampoline for integer Is_sorted method.
/// # Safety
/// `x` must be a valid ALTREP INTSXP.
pub unsafe extern "C-unwind" fn t_int_is_sorted<T: AltInteger>(x: SEXP) -> i32 {
    guarded_altrep_call::<T, _, _>(|| T::is_sorted(x))
}

/// Trampoline for integer No_NA method.
/// # Safety
/// `x` must be a valid ALTREP INTSXP.
pub unsafe extern "C-unwind" fn t_int_no_na<T: AltInteger>(x: SEXP) -> i32 {
    guarded_altrep_call::<T, _, _>(|| T::no_na(x))
}

/// Trampoline for integer Sum method.
/// # Safety
/// `x` must be a valid ALTREP INTSXP.
pub unsafe extern "C-unwind" fn t_int_sum<T: AltInteger>(x: SEXP, narm: Rboolean) -> SEXP {
    guarded_altrep_call::<T, _, _>(|| T::sum(x, matches!(narm, Rboolean::TRUE)))
}

/// Trampoline for integer Min method.
/// # Safety
/// `x` must be a valid ALTREP INTSXP.
pub unsafe extern "C-unwind" fn t_int_min<T: AltInteger>(x: SEXP, narm: Rboolean) -> SEXP {
    guarded_altrep_call::<T, _, _>(|| T::min(x, matches!(narm, Rboolean::TRUE)))
}

/// Trampoline for integer Max method.
/// # Safety
/// `x` must be a valid ALTREP INTSXP.
pub unsafe extern "C-unwind" fn t_int_max<T: AltInteger>(x: SEXP, narm: Rboolean) -> SEXP {
    guarded_altrep_call::<T, _, _>(|| T::max(x, matches!(narm, Rboolean::TRUE)))
}
// endregion

// region: ALTREAL TRAMPOLINES

/// Trampoline for real Elt method.
/// # Safety
/// `x` must be a valid ALTREP REALSXP and `i` within bounds.
pub unsafe extern "C-unwind" fn t_real_elt<T: AltReal>(x: SEXP, i: R_xlen_t) -> f64 {
    guarded_altrep_call::<T, _, _>(|| T::elt(x, i))
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
    guarded_altrep_call::<T, _, _>(|| T::get_region(x, i, n, out))
}

/// Trampoline for real Is_sorted method.
/// # Safety
/// `x` must be a valid ALTREP REALSXP.
pub unsafe extern "C-unwind" fn t_real_is_sorted<T: AltReal>(x: SEXP) -> i32 {
    guarded_altrep_call::<T, _, _>(|| T::is_sorted(x))
}

/// Trampoline for real No_NA method.
/// # Safety
/// `x` must be a valid ALTREP REALSXP.
pub unsafe extern "C-unwind" fn t_real_no_na<T: AltReal>(x: SEXP) -> i32 {
    guarded_altrep_call::<T, _, _>(|| T::no_na(x))
}

/// Trampoline for real Sum method.
/// # Safety
/// `x` must be a valid ALTREP REALSXP.
pub unsafe extern "C-unwind" fn t_real_sum<T: AltReal>(x: SEXP, narm: Rboolean) -> SEXP {
    guarded_altrep_call::<T, _, _>(|| T::sum(x, matches!(narm, Rboolean::TRUE)))
}

/// Trampoline for real Min method.
/// # Safety
/// `x` must be a valid ALTREP REALSXP.
pub unsafe extern "C-unwind" fn t_real_min<T: AltReal>(x: SEXP, narm: Rboolean) -> SEXP {
    guarded_altrep_call::<T, _, _>(|| T::min(x, matches!(narm, Rboolean::TRUE)))
}

/// Trampoline for real Max method.
/// # Safety
/// `x` must be a valid ALTREP REALSXP.
pub unsafe extern "C-unwind" fn t_real_max<T: AltReal>(x: SEXP, narm: Rboolean) -> SEXP {
    guarded_altrep_call::<T, _, _>(|| T::max(x, matches!(narm, Rboolean::TRUE)))
}
// endregion

// region: ALTLOGICAL TRAMPOLINES

/// Trampoline for logical Elt method.
/// # Safety
/// `x` must be a valid ALTREP LGLSXP and `i` within bounds.
pub unsafe extern "C-unwind" fn t_lgl_elt<T: AltLogical>(x: SEXP, i: R_xlen_t) -> i32 {
    guarded_altrep_call::<T, _, _>(|| T::elt(x, i))
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
    guarded_altrep_call::<T, _, _>(|| T::get_region(x, i, n, out))
}

/// Trampoline for logical Is_sorted method.
/// # Safety
/// `x` must be a valid ALTREP LGLSXP.
pub unsafe extern "C-unwind" fn t_lgl_is_sorted<T: AltLogical>(x: SEXP) -> i32 {
    guarded_altrep_call::<T, _, _>(|| T::is_sorted(x))
}

/// Trampoline for logical No_NA method.
/// # Safety
/// `x` must be a valid ALTREP LGLSXP.
pub unsafe extern "C-unwind" fn t_lgl_no_na<T: AltLogical>(x: SEXP) -> i32 {
    guarded_altrep_call::<T, _, _>(|| T::no_na(x))
}

/// Trampoline for logical Sum method.
/// # Safety
/// `x` must be a valid ALTREP LGLSXP.
pub unsafe extern "C-unwind" fn t_lgl_sum<T: AltLogical>(x: SEXP, narm: Rboolean) -> SEXP {
    guarded_altrep_call::<T, _, _>(|| T::sum(x, matches!(narm, Rboolean::TRUE)))
}

// Note: R's ALTREP API does not expose min/max for logical vectors
// endregion

// region: ALTRAW TRAMPOLINES

/// Trampoline for raw Elt method.
/// # Safety
/// `x` must be a valid ALTREP RAWSXP and `i` within bounds.
pub unsafe extern "C-unwind" fn t_raw_elt<T: AltRaw>(x: SEXP, i: R_xlen_t) -> Rbyte {
    guarded_altrep_call::<T, _, _>(|| T::elt(x, i))
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
    guarded_altrep_call::<T, _, _>(|| T::get_region(x, i, n, out))
}
// endregion

// region: ALTCOMPLEX TRAMPOLINES

/// Trampoline for complex Elt method.
/// # Safety
/// `x` must be a valid ALTREP CPLXSXP and `i` within bounds.
pub unsafe extern "C-unwind" fn t_cplx_elt<T: AltComplex>(x: SEXP, i: R_xlen_t) -> Rcomplex {
    guarded_altrep_call::<T, _, _>(|| T::elt(x, i))
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
    guarded_altrep_call::<T, _, _>(|| T::get_region(x, i, n, out))
}
// endregion

// region: ALTSTRING TRAMPOLINES

/// Trampoline for string Elt method (REQUIRED for ALTSTRING).
/// # Safety
/// `x` must be a valid ALTREP STRSXP and `i` within bounds.
pub unsafe extern "C-unwind" fn t_str_elt<T: AltString>(x: SEXP, i: R_xlen_t) -> SEXP {
    guarded_altrep_call::<T, _, _>(|| T::elt(x, i))
}

/// Trampoline for string Set_elt method.
/// # Safety
/// `x` must be a valid ALTREP STRSXP and `v` a valid CHARSXP.
pub unsafe extern "C-unwind" fn t_str_set_elt<T: AltString>(x: SEXP, i: R_xlen_t, v: SEXP) {
    guarded_altrep_call::<T, _, _>(|| T::set_elt(x, i, v))
}

/// Trampoline for string Is_sorted method.
/// # Safety
/// `x` must be a valid ALTREP STRSXP.
pub unsafe extern "C-unwind" fn t_str_is_sorted<T: AltString>(x: SEXP) -> i32 {
    guarded_altrep_call::<T, _, _>(|| T::is_sorted(x))
}

/// Trampoline for string No_NA method.
/// # Safety
/// `x` must be a valid ALTREP STRSXP.
pub unsafe extern "C-unwind" fn t_str_no_na<T: AltString>(x: SEXP) -> i32 {
    guarded_altrep_call::<T, _, _>(|| T::no_na(x))
}
// endregion

// region: ALTLIST TRAMPOLINES

/// Trampoline for list Elt method (REQUIRED for ALTLIST).
/// # Safety
/// `x` must be a valid ALTREP VECSXP and `i` within bounds.
pub unsafe extern "C-unwind" fn t_list_elt<T: AltList>(x: SEXP, i: R_xlen_t) -> SEXP {
    guarded_altrep_call::<T, _, _>(|| T::elt(x, i))
}

/// Trampoline for list Set_elt method.
/// # Safety
/// `x` must be a valid ALTREP VECSXP and `v` a valid SEXP.
pub unsafe extern "C-unwind" fn t_list_set_elt<T: AltList>(x: SEXP, i: R_xlen_t, v: SEXP) {
    guarded_altrep_call::<T, _, _>(|| T::set_elt(x, i, v))
}
// endregion

// region: INSTALLERS - Only install methods where HAS_* = true

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

/// Generate a family-specific installer function from a declarative spec.
///
/// Each entry maps a `HAS_*` const to a setter function and trampoline.
/// Optional `always` entries are installed unconditionally (e.g. required Elt).
macro_rules! def_installer {
    (
        $(#[$meta:meta])*
        $fn_name:ident < T: $trait:ident > {
            $( $has:ident => $setter:path, $tramp:ident; )*
        }
    ) => {
        $(#[$meta])*
        pub unsafe fn $fn_name<T: $trait>(cls: R_altrep_class_t) {
            $(
                if T::$has { unsafe { $setter(cls, Some($tramp::<T>)) } }
            )*
        }
    };
    (
        $(#[$meta:meta])*
        $fn_name:ident < T: $trait:ident > {
            $( $has:ident => $setter:path, $tramp:ident; )*
        }
        always { $( $always_setter:path, $always_tramp:ident; )* }
    ) => {
        $(#[$meta])*
        pub unsafe fn $fn_name<T: $trait>(cls: R_altrep_class_t) {
            $(
                unsafe { $always_setter(cls, Some($always_tramp::<T>)) }
            )*
            $(
                if T::$has { unsafe { $setter(cls, Some($tramp::<T>)) } }
            )*
        }
    };
}

def_installer! {
    /// Install integer-specific methods.
    /// # Safety
    /// Must be called during R initialization with a valid ALTREP class handle.
    install_int<T: AltInteger> {
        HAS_ELT => R_set_altinteger_Elt_method, t_int_elt;
        HAS_GET_REGION => R_set_altinteger_Get_region_method, t_int_get_region;
        HAS_IS_SORTED => R_set_altinteger_Is_sorted_method, t_int_is_sorted;
        HAS_NO_NA => R_set_altinteger_No_NA_method, t_int_no_na;
        HAS_SUM => R_set_altinteger_Sum_method, t_int_sum;
        HAS_MIN => R_set_altinteger_Min_method, t_int_min;
        HAS_MAX => R_set_altinteger_Max_method, t_int_max;
    }
}

def_installer! {
    /// Install real-specific methods.
    /// # Safety
    /// Must be called during R initialization with a valid ALTREP class handle.
    install_real<T: AltReal> {
        HAS_ELT => R_set_altreal_Elt_method, t_real_elt;
        HAS_GET_REGION => R_set_altreal_Get_region_method, t_real_get_region;
        HAS_IS_SORTED => R_set_altreal_Is_sorted_method, t_real_is_sorted;
        HAS_NO_NA => R_set_altreal_No_NA_method, t_real_no_na;
        HAS_SUM => R_set_altreal_Sum_method, t_real_sum;
        HAS_MIN => R_set_altreal_Min_method, t_real_min;
        HAS_MAX => R_set_altreal_Max_method, t_real_max;
    }
}

def_installer! {
    /// Install logical-specific methods.
    /// # Safety
    /// Must be called during R initialization with a valid ALTREP class handle.
    install_lgl<T: AltLogical> {
        HAS_ELT => R_set_altlogical_Elt_method, t_lgl_elt;
        HAS_GET_REGION => R_set_altlogical_Get_region_method, t_lgl_get_region;
        HAS_IS_SORTED => R_set_altlogical_Is_sorted_method, t_lgl_is_sorted;
        HAS_NO_NA => R_set_altlogical_No_NA_method, t_lgl_no_na;
        HAS_SUM => R_set_altlogical_Sum_method, t_lgl_sum;
    }
}

def_installer! {
    /// Install raw-specific methods.
    /// # Safety
    /// Must be called during R initialization with a valid ALTREP class handle.
    install_raw<T: AltRaw> {
        HAS_ELT => R_set_altraw_Elt_method, t_raw_elt;
        HAS_GET_REGION => R_set_altraw_Get_region_method, t_raw_get_region;
    }
}

def_installer! {
    /// Install complex-specific methods.
    /// # Safety
    /// Must be called during R initialization with a valid ALTREP class handle.
    install_cplx<T: AltComplex> {
        HAS_ELT => R_set_altcomplex_Elt_method, t_cplx_elt;
        HAS_GET_REGION => R_set_altcomplex_Get_region_method, t_cplx_get_region;
    }
}

def_installer! {
    /// Install string-specific methods.
    /// # Safety
    /// Must be called during R initialization with a valid ALTREP class handle.
    /// Note: Elt is always installed for ALTSTRING (required).
    install_str<T: AltString> {
        HAS_SET_ELT => R_set_altstring_Set_elt_method, t_str_set_elt;
        HAS_IS_SORTED => R_set_altstring_Is_sorted_method, t_str_is_sorted;
        HAS_NO_NA => R_set_altstring_No_NA_method, t_str_no_na;
    }
    always { R_set_altstring_Elt_method, t_str_elt; }
}

def_installer! {
    /// Install list-specific methods.
    /// # Safety
    /// Must be called during R initialization with a valid ALTREP class handle.
    /// Note: Elt is always installed for ALTLIST (required).
    install_list<T: AltList> {
        HAS_SET_ELT => R_set_altlist_Set_elt_method, t_list_set_elt;
    }
    always { R_set_altlist_Elt_method, t_list_elt; }
}
// endregion
