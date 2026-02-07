//! Raw ALTREP C API bindings.
//!
//! This module mirrors `R_ext/Altrep.h` and is intentionally low-level.

#![allow(non_camel_case_types)]
use crate::ffi::{DllInfo, R_xlen_t, Rboolean, Rbyte, Rcomplex, SEXP, SEXPTYPE};

#[allow(non_camel_case_types)]
/// Signature for ALTREP `coerce` method.
pub type R_altrep_Coerce_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP, rtype: SEXPTYPE) -> SEXP>;

/// Signature for ALTREP extended `unserialize` method.
pub type R_altrep_UnserializeEX_method_t = ::std::option::Option<
    unsafe extern "C-unwind" fn(
        class: SEXP,
        state: SEXP,
        attr: SEXP,
        objf: ::std::os::raw::c_int,
        levs: ::std::os::raw::c_int,
    ) -> SEXP,
>;
/// Signature for ALTREP `unserialize` method.
pub type R_altrep_Unserialize_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(class: SEXP, state: SEXP) -> SEXP>;
/// Signature for ALTREP `serialized_state` method.
pub type R_altrep_Serialized_state_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP) -> SEXP>;
/// Signature for ALTREP extended `duplicate` method.
pub type R_altrep_DuplicateEX_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP, deep: Rboolean) -> SEXP>;
/// Signature for ALTREP `duplicate` method.
pub type R_altrep_Duplicate_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP, deep: Rboolean) -> SEXP>;
/// Signature for ALTREP `inspect` method.
pub type R_altrep_Inspect_method_t = ::std::option::Option<
    unsafe extern "C-unwind" fn(
        x: SEXP,
        pre: ::std::os::raw::c_int,
        deep: ::std::os::raw::c_int,
        pvec: ::std::os::raw::c_int,
        inspect_subtree: ::std::option::Option<
            unsafe extern "C-unwind" fn(
                x: SEXP,
                pre: ::std::os::raw::c_int,
                deep: ::std::os::raw::c_int,
                pvec: ::std::os::raw::c_int,
            ),
        >,
    ) -> Rboolean,
>;
/// Signature for ALTREP `length` method.
pub type R_altrep_Length_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP) -> R_xlen_t>;
/// Signature for ALTVEC `dataptr` method.
pub type R_altvec_Dataptr_method_t = ::std::option::Option<
    unsafe extern "C-unwind" fn(x: SEXP, writable: Rboolean) -> *mut ::std::os::raw::c_void,
>;
/// Signature for ALTVEC `dataptr_or_null` method.
pub type R_altvec_Dataptr_or_null_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP) -> *const ::std::os::raw::c_void>;
/// Signature for ALTVEC `extract_subset` method.
pub type R_altvec_Extract_subset_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP, indx: SEXP, call: SEXP) -> SEXP>;
/// Signature for ALTINTEGER `elt` method.
pub type R_altinteger_Elt_method_t = ::std::option::Option<
    unsafe extern "C-unwind" fn(x: SEXP, i: R_xlen_t) -> ::std::os::raw::c_int,
>;
/// Signature for ALTINTEGER `get_region` method.
pub type R_altinteger_Get_region_method_t = ::std::option::Option<
    unsafe extern "C-unwind" fn(
        sx: SEXP,
        i: R_xlen_t,
        n: R_xlen_t,
        buf: *mut ::std::os::raw::c_int,
    ) -> R_xlen_t,
>;
/// Signature for ALTINTEGER `is_sorted` method.
pub type R_altinteger_Is_sorted_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP) -> ::std::os::raw::c_int>;
/// Signature for ALTINTEGER `no_na` method.
pub type R_altinteger_No_NA_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP) -> ::std::os::raw::c_int>;
/// Signature for ALTINTEGER `sum` method.
pub type R_altinteger_Sum_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP, narm: Rboolean) -> SEXP>;
/// Signature for ALTINTEGER `min` method.
pub type R_altinteger_Min_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP, narm: Rboolean) -> SEXP>;
/// Signature for ALTINTEGER `max` method.
pub type R_altinteger_Max_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP, narm: Rboolean) -> SEXP>;
/// Signature for ALTREAL `elt` method.
pub type R_altreal_Elt_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP, i: R_xlen_t) -> f64>;
/// Signature for ALTREAL `get_region` method.
pub type R_altreal_Get_region_method_t = ::std::option::Option<
    unsafe extern "C-unwind" fn(sx: SEXP, i: R_xlen_t, n: R_xlen_t, buf: *mut f64) -> R_xlen_t,
>;
/// Signature for ALTREAL `is_sorted` method.
pub type R_altreal_Is_sorted_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP) -> ::std::os::raw::c_int>;
/// Signature for ALTREAL `no_na` method.
pub type R_altreal_No_NA_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP) -> ::std::os::raw::c_int>;
/// Signature for ALTREAL `sum` method.
pub type R_altreal_Sum_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP, narm: Rboolean) -> SEXP>;
/// Signature for ALTREAL `min` method.
pub type R_altreal_Min_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP, narm: Rboolean) -> SEXP>;
/// Signature for ALTREAL `max` method.
pub type R_altreal_Max_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP, narm: Rboolean) -> SEXP>;
/// Signature for ALTLOGICAL `elt` method.
pub type R_altlogical_Elt_method_t = ::std::option::Option<
    unsafe extern "C-unwind" fn(x: SEXP, i: R_xlen_t) -> ::std::os::raw::c_int,
>;
/// Signature for ALTLOGICAL `get_region` method.
pub type R_altlogical_Get_region_method_t = ::std::option::Option<
    unsafe extern "C-unwind" fn(
        sx: SEXP,
        i: R_xlen_t,
        n: R_xlen_t,
        buf: *mut ::std::os::raw::c_int,
    ) -> R_xlen_t,
>;
/// Signature for ALTLOGICAL `is_sorted` method.
pub type R_altlogical_Is_sorted_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP) -> ::std::os::raw::c_int>;
/// Signature for ALTLOGICAL `no_na` method.
pub type R_altlogical_No_NA_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP) -> ::std::os::raw::c_int>;
/// Signature for ALTLOGICAL `sum` method.
pub type R_altlogical_Sum_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP, narm: Rboolean) -> SEXP>;
/// Signature for ALTRAW `elt` method.
pub type R_altraw_Elt_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP, i: R_xlen_t) -> Rbyte>;
/// Signature for ALTRAW `get_region` method.
pub type R_altraw_Get_region_method_t = ::std::option::Option<
    unsafe extern "C-unwind" fn(sx: SEXP, i: R_xlen_t, n: R_xlen_t, buf: *mut Rbyte) -> R_xlen_t,
>;
/// Signature for ALTCOMPLEX `elt` method.
pub type R_altcomplex_Elt_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP, i: R_xlen_t) -> Rcomplex>;
/// Signature for ALTCOMPLEX `get_region` method.
pub type R_altcomplex_Get_region_method_t = ::std::option::Option<
    unsafe extern "C-unwind" fn(sx: SEXP, i: R_xlen_t, n: R_xlen_t, buf: *mut Rcomplex) -> R_xlen_t,
>;
/// Signature for ALTSTRING `elt` method.
pub type R_altstring_Elt_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP, i: R_xlen_t) -> SEXP>;
/// Signature for ALTSTRING `set_elt` method.
pub type R_altstring_Set_elt_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP, i: R_xlen_t, v: SEXP)>;
/// Signature for ALTSTRING `is_sorted` method.
pub type R_altstring_Is_sorted_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP) -> ::std::os::raw::c_int>;
/// Signature for ALTSTRING `no_na` method.
pub type R_altstring_No_NA_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP) -> ::std::os::raw::c_int>;
/// Signature for ALTLIST `elt` method.
pub type R_altlist_Elt_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP, i: R_xlen_t) -> SEXP>;
/// Signature for ALTLIST `set_elt` method.
pub type R_altlist_Set_elt_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP, i: R_xlen_t, v: SEXP)>;
#[derive(Clone, Copy)]
#[repr(C)]
/// Opaque ALTREP class handle.
pub struct R_altrep_class_t {
    /// Underlying class object SEXP.
    pub ptr: SEXP,
}

// SAFETY: R_altrep_class_t is only used on R's main thread.
// The class is created once during package init and stored in a static.
unsafe impl Send for R_altrep_class_t {}
unsafe impl Sync for R_altrep_class_t {}

// Imported ALTREP constructor and method-registration symbols.
#[allow(missing_docs)]
#[miniextendr_macros::r_ffi_checked]
#[allow(non_snake_case)]
unsafe extern "C-unwind" {
    pub fn R_new_altrep(aclass: R_altrep_class_t, data1: SEXP, data2: SEXP) -> SEXP;

    // ALTREP class constructors
    pub fn R_make_altstring_class(
        cname: *const ::std::os::raw::c_char,
        pname: *const ::std::os::raw::c_char,
        info: *mut DllInfo,
    ) -> R_altrep_class_t;
    pub fn R_make_altinteger_class(
        cname: *const ::std::os::raw::c_char,
        pname: *const ::std::os::raw::c_char,
        info: *mut DllInfo,
    ) -> R_altrep_class_t;
    pub fn R_make_altreal_class(
        cname: *const ::std::os::raw::c_char,
        pname: *const ::std::os::raw::c_char,
        info: *mut DllInfo,
    ) -> R_altrep_class_t;
    pub fn R_make_altlogical_class(
        cname: *const ::std::os::raw::c_char,
        pname: *const ::std::os::raw::c_char,
        info: *mut DllInfo,
    ) -> R_altrep_class_t;
    pub fn R_make_altraw_class(
        cname: *const ::std::os::raw::c_char,
        pname: *const ::std::os::raw::c_char,
        info: *mut DllInfo,
    ) -> R_altrep_class_t;
    pub fn R_make_altcomplex_class(
        cname: *const ::std::os::raw::c_char,
        pname: *const ::std::os::raw::c_char,
        info: *mut DllInfo,
    ) -> R_altrep_class_t;
    pub fn R_make_altlist_class(
        cname: *const ::std::os::raw::c_char,
        pname: *const ::std::os::raw::c_char,
        info: *mut DllInfo,
    ) -> R_altrep_class_t;
    pub fn R_altrep_inherits(x: SEXP, aclass: R_altrep_class_t) -> Rboolean;
    pub fn R_set_altrep_UnserializeEX_method(
        cls: R_altrep_class_t,
        fun: R_altrep_UnserializeEX_method_t,
    );
    pub fn R_set_altrep_Unserialize_method(
        cls: R_altrep_class_t,
        fun: R_altrep_Unserialize_method_t,
    );
    pub fn R_set_altrep_Serialized_state_method(
        cls: R_altrep_class_t,
        fun: R_altrep_Serialized_state_method_t,
    );
    pub fn R_set_altrep_DuplicateEX_method(
        cls: R_altrep_class_t,
        fun: R_altrep_DuplicateEX_method_t,
    );
    pub fn R_set_altrep_Duplicate_method(cls: R_altrep_class_t, fun: R_altrep_Duplicate_method_t);
    pub fn R_set_altrep_Coerce_method(cls: R_altrep_class_t, fun: R_altrep_Coerce_method_t);
    pub fn R_set_altrep_Inspect_method(cls: R_altrep_class_t, fun: R_altrep_Inspect_method_t);
    pub fn R_set_altrep_Length_method(cls: R_altrep_class_t, fun: R_altrep_Length_method_t);
    pub fn R_set_altvec_Dataptr_method(cls: R_altrep_class_t, fun: R_altvec_Dataptr_method_t);
    pub fn R_set_altvec_Dataptr_or_null_method(
        cls: R_altrep_class_t,
        fun: R_altvec_Dataptr_or_null_method_t,
    );
    pub fn R_set_altvec_Extract_subset_method(
        cls: R_altrep_class_t,
        fun: R_altvec_Extract_subset_method_t,
    );
    pub fn R_set_altinteger_Elt_method(cls: R_altrep_class_t, fun: R_altinteger_Elt_method_t);
    pub fn R_set_altinteger_Get_region_method(
        cls: R_altrep_class_t,
        fun: R_altinteger_Get_region_method_t,
    );
    pub fn R_set_altinteger_Is_sorted_method(
        cls: R_altrep_class_t,
        fun: R_altinteger_Is_sorted_method_t,
    );
    pub fn R_set_altinteger_No_NA_method(cls: R_altrep_class_t, fun: R_altinteger_No_NA_method_t);
    pub fn R_set_altinteger_Sum_method(cls: R_altrep_class_t, fun: R_altinteger_Sum_method_t);
    pub fn R_set_altinteger_Min_method(cls: R_altrep_class_t, fun: R_altinteger_Min_method_t);
    pub fn R_set_altinteger_Max_method(cls: R_altrep_class_t, fun: R_altinteger_Max_method_t);
    pub fn R_set_altreal_Elt_method(cls: R_altrep_class_t, fun: R_altreal_Elt_method_t);
    pub fn R_set_altreal_Get_region_method(
        cls: R_altrep_class_t,
        fun: R_altreal_Get_region_method_t,
    );
    pub fn R_set_altreal_Is_sorted_method(cls: R_altrep_class_t, fun: R_altreal_Is_sorted_method_t);
    pub fn R_set_altreal_No_NA_method(cls: R_altrep_class_t, fun: R_altreal_No_NA_method_t);
    pub fn R_set_altreal_Sum_method(cls: R_altrep_class_t, fun: R_altreal_Sum_method_t);
    pub fn R_set_altreal_Min_method(cls: R_altrep_class_t, fun: R_altreal_Min_method_t);
    pub fn R_set_altreal_Max_method(cls: R_altrep_class_t, fun: R_altreal_Max_method_t);
    pub fn R_set_altlogical_Elt_method(cls: R_altrep_class_t, fun: R_altlogical_Elt_method_t);
    pub fn R_set_altlogical_Get_region_method(
        cls: R_altrep_class_t,
        fun: R_altlogical_Get_region_method_t,
    );
    pub fn R_set_altlogical_Is_sorted_method(
        cls: R_altrep_class_t,
        fun: R_altlogical_Is_sorted_method_t,
    );
    pub fn R_set_altlogical_No_NA_method(cls: R_altrep_class_t, fun: R_altlogical_No_NA_method_t);
    pub fn R_set_altlogical_Sum_method(cls: R_altrep_class_t, fun: R_altlogical_Sum_method_t);
    pub fn R_set_altraw_Elt_method(cls: R_altrep_class_t, fun: R_altraw_Elt_method_t);
    pub fn R_set_altraw_Get_region_method(cls: R_altrep_class_t, fun: R_altraw_Get_region_method_t);
    pub fn R_set_altcomplex_Elt_method(cls: R_altrep_class_t, fun: R_altcomplex_Elt_method_t);
    pub fn R_set_altcomplex_Get_region_method(
        cls: R_altrep_class_t,
        fun: R_altcomplex_Get_region_method_t,
    );
    pub fn R_set_altstring_Elt_method(cls: R_altrep_class_t, fun: R_altstring_Elt_method_t);
    pub fn R_set_altstring_Set_elt_method(cls: R_altrep_class_t, fun: R_altstring_Set_elt_method_t);
    pub fn R_set_altstring_Is_sorted_method(
        cls: R_altrep_class_t,
        fun: R_altstring_Is_sorted_method_t,
    );
    pub fn R_set_altstring_No_NA_method(cls: R_altrep_class_t, fun: R_altstring_No_NA_method_t);
    pub fn R_set_altlist_Elt_method(cls: R_altrep_class_t, fun: R_altlist_Elt_method_t);
    pub fn R_set_altlist_Set_elt_method(cls: R_altrep_class_t, fun: R_altlist_Set_elt_method_t);
}

// region: ALTREP Helper Functions (Rust equivalents of R's ALTREP macros)

/// Extracts the `ptr` field from `R_altrep_class_t`.
///
/// Rust equivalent of the C macro `R_SEXP(x)` which expands to `(x).ptr`.
#[inline(always)]
pub fn sexp(class: R_altrep_class_t) -> SEXP {
    class.ptr
}

/// Creates an `R_altrep_class_t` from a SEXP pointer.
///
/// Rust equivalent of the C macro `R_SUBTYPE_INIT(x)` which expands to `{ x }`.
#[inline(always)]
pub const fn subtype_init(ptr: SEXP) -> R_altrep_class_t {
    R_altrep_class_t { ptr }
}

// endregion
