//! Raw ALTREP C API bindings.
//!
//! This module mirrors `R_ext/Altrep.h` and is intentionally low-level.

#![allow(non_camel_case_types)]
use crate::ffi::{DllInfo, R_xlen_t, Rboolean, Rbyte, Rcomplex, SEXP, SEXPTYPE};

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
unsafe extern "C-unwind" {
    // ALTREP instance construction (encapsulated by R_altrep_class_t::new_altrep)
    fn R_new_altrep(aclass: R_altrep_class_t, data1: SEXP, data2: SEXP) -> SEXP;

    // ALTREP class constructors — pub because proc-macro generates calls from user crates
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
    // ALTREP class membership check (encapsulated by R_altrep_class_t::inherits)
    fn R_altrep_inherits(x: SEXP, aclass: R_altrep_class_t) -> Rboolean;

    // Method setters — private, encapsulated by R_altrep_class_t methods
    fn R_set_altrep_UnserializeEX_method(
        cls: R_altrep_class_t,
        fun: R_altrep_UnserializeEX_method_t,
    );
    fn R_set_altrep_Unserialize_method(cls: R_altrep_class_t, fun: R_altrep_Unserialize_method_t);
    fn R_set_altrep_Serialized_state_method(
        cls: R_altrep_class_t,
        fun: R_altrep_Serialized_state_method_t,
    );
    fn R_set_altrep_DuplicateEX_method(cls: R_altrep_class_t, fun: R_altrep_DuplicateEX_method_t);
    fn R_set_altrep_Duplicate_method(cls: R_altrep_class_t, fun: R_altrep_Duplicate_method_t);
    fn R_set_altrep_Coerce_method(cls: R_altrep_class_t, fun: R_altrep_Coerce_method_t);
    fn R_set_altrep_Inspect_method(cls: R_altrep_class_t, fun: R_altrep_Inspect_method_t);
    fn R_set_altrep_Length_method(cls: R_altrep_class_t, fun: R_altrep_Length_method_t);
    fn R_set_altvec_Dataptr_method(cls: R_altrep_class_t, fun: R_altvec_Dataptr_method_t);
    fn R_set_altvec_Dataptr_or_null_method(
        cls: R_altrep_class_t,
        fun: R_altvec_Dataptr_or_null_method_t,
    );
    fn R_set_altvec_Extract_subset_method(
        cls: R_altrep_class_t,
        fun: R_altvec_Extract_subset_method_t,
    );
    fn R_set_altinteger_Elt_method(cls: R_altrep_class_t, fun: R_altinteger_Elt_method_t);
    fn R_set_altinteger_Get_region_method(
        cls: R_altrep_class_t,
        fun: R_altinteger_Get_region_method_t,
    );
    fn R_set_altinteger_Is_sorted_method(
        cls: R_altrep_class_t,
        fun: R_altinteger_Is_sorted_method_t,
    );
    fn R_set_altinteger_No_NA_method(cls: R_altrep_class_t, fun: R_altinteger_No_NA_method_t);
    fn R_set_altinteger_Sum_method(cls: R_altrep_class_t, fun: R_altinteger_Sum_method_t);
    fn R_set_altinteger_Min_method(cls: R_altrep_class_t, fun: R_altinteger_Min_method_t);
    fn R_set_altinteger_Max_method(cls: R_altrep_class_t, fun: R_altinteger_Max_method_t);
    fn R_set_altreal_Elt_method(cls: R_altrep_class_t, fun: R_altreal_Elt_method_t);
    fn R_set_altreal_Get_region_method(cls: R_altrep_class_t, fun: R_altreal_Get_region_method_t);
    fn R_set_altreal_Is_sorted_method(cls: R_altrep_class_t, fun: R_altreal_Is_sorted_method_t);
    fn R_set_altreal_No_NA_method(cls: R_altrep_class_t, fun: R_altreal_No_NA_method_t);
    fn R_set_altreal_Sum_method(cls: R_altrep_class_t, fun: R_altreal_Sum_method_t);
    fn R_set_altreal_Min_method(cls: R_altrep_class_t, fun: R_altreal_Min_method_t);
    fn R_set_altreal_Max_method(cls: R_altrep_class_t, fun: R_altreal_Max_method_t);
    fn R_set_altlogical_Elt_method(cls: R_altrep_class_t, fun: R_altlogical_Elt_method_t);
    fn R_set_altlogical_Get_region_method(
        cls: R_altrep_class_t,
        fun: R_altlogical_Get_region_method_t,
    );
    fn R_set_altlogical_Is_sorted_method(
        cls: R_altrep_class_t,
        fun: R_altlogical_Is_sorted_method_t,
    );
    fn R_set_altlogical_No_NA_method(cls: R_altrep_class_t, fun: R_altlogical_No_NA_method_t);
    fn R_set_altlogical_Sum_method(cls: R_altrep_class_t, fun: R_altlogical_Sum_method_t);
    fn R_set_altraw_Elt_method(cls: R_altrep_class_t, fun: R_altraw_Elt_method_t);
    fn R_set_altraw_Get_region_method(cls: R_altrep_class_t, fun: R_altraw_Get_region_method_t);
    fn R_set_altcomplex_Elt_method(cls: R_altrep_class_t, fun: R_altcomplex_Elt_method_t);
    fn R_set_altcomplex_Get_region_method(
        cls: R_altrep_class_t,
        fun: R_altcomplex_Get_region_method_t,
    );
    fn R_set_altstring_Elt_method(cls: R_altrep_class_t, fun: R_altstring_Elt_method_t);
    fn R_set_altstring_Set_elt_method(cls: R_altrep_class_t, fun: R_altstring_Set_elt_method_t);
    fn R_set_altstring_Is_sorted_method(cls: R_altrep_class_t, fun: R_altstring_Is_sorted_method_t);
    fn R_set_altstring_No_NA_method(cls: R_altrep_class_t, fun: R_altstring_No_NA_method_t);
    fn R_set_altlist_Elt_method(cls: R_altrep_class_t, fun: R_altlist_Elt_method_t);
    fn R_set_altlist_Set_elt_method(cls: R_altrep_class_t, fun: R_altlist_Set_elt_method_t);
}

impl R_altrep_class_t {
    /// Create from a raw SEXP pointer.
    ///
    /// Rust equivalent of C macro `R_SUBTYPE_INIT(x)`.
    #[inline(always)]
    pub const fn from_sexp(ptr: SEXP) -> Self {
        Self { ptr }
    }

    /// Get the underlying SEXP.
    ///
    /// Rust equivalent of C macro `R_SEXP(x)`.
    #[inline(always)]
    pub fn as_sexp(self) -> SEXP {
        self.ptr
    }

    /// Create a new ALTREP instance with data1 and data2 slots.
    ///
    /// # Safety
    /// Must be called on R's main thread. `data1` and `data2` must be valid SEXPs.
    #[inline]
    pub unsafe fn new_altrep(self, data1: SEXP, data2: SEXP) -> SEXP {
        unsafe { R_new_altrep(self, data1, data2) }
    }

    /// Create a new ALTREP instance (no thread check).
    ///
    /// # Safety
    /// Must be called on R's main thread.
    #[inline]
    pub unsafe fn new_altrep_unchecked(self, data1: SEXP, data2: SEXP) -> SEXP {
        unsafe { R_new_altrep_unchecked(self, data1, data2) }
    }

    /// Check if `x` is an instance of this ALTREP class.
    ///
    /// # Safety
    /// Must be called on R's main thread. `x` must be a valid SEXP.
    #[inline]
    pub unsafe fn inherits(self, x: SEXP) -> bool {
        unsafe { R_altrep_inherits(x, self) != Rboolean::FALSE }
    }

    // region: Base ALTREP method setters

    /// Set the Length method.
    /// # Safety
    /// Must be called during R initialization.
    #[inline]
    pub unsafe fn set_length_method(self, fun: R_altrep_Length_method_t) {
        unsafe { R_set_altrep_Length_method(self, fun) }
    }

    /// Set the Serialized_state method.
    /// # Safety
    /// Must be called during R initialization.
    #[inline]
    pub unsafe fn set_serialized_state_method(self, fun: R_altrep_Serialized_state_method_t) {
        unsafe { R_set_altrep_Serialized_state_method(self, fun) }
    }

    /// Set the Unserialize method.
    /// # Safety
    /// Must be called during R initialization.
    #[inline]
    pub unsafe fn set_unserialize_method(self, fun: R_altrep_Unserialize_method_t) {
        unsafe { R_set_altrep_Unserialize_method(self, fun) }
    }

    /// Set the UnserializeEX method.
    /// # Safety
    /// Must be called during R initialization.
    #[inline]
    pub unsafe fn set_unserialize_ex_method(self, fun: R_altrep_UnserializeEX_method_t) {
        unsafe { R_set_altrep_UnserializeEX_method(self, fun) }
    }

    /// Set the Duplicate method.
    /// # Safety
    /// Must be called during R initialization.
    #[inline]
    pub unsafe fn set_duplicate_method(self, fun: R_altrep_Duplicate_method_t) {
        unsafe { R_set_altrep_Duplicate_method(self, fun) }
    }

    /// Set the DuplicateEX method.
    /// # Safety
    /// Must be called during R initialization.
    #[inline]
    pub unsafe fn set_duplicate_ex_method(self, fun: R_altrep_DuplicateEX_method_t) {
        unsafe { R_set_altrep_DuplicateEX_method(self, fun) }
    }

    /// Set the Coerce method.
    /// # Safety
    /// Must be called during R initialization.
    #[inline]
    pub unsafe fn set_coerce_method(self, fun: R_altrep_Coerce_method_t) {
        unsafe { R_set_altrep_Coerce_method(self, fun) }
    }

    /// Set the Inspect method.
    /// # Safety
    /// Must be called during R initialization.
    #[inline]
    pub unsafe fn set_inspect_method(self, fun: R_altrep_Inspect_method_t) {
        unsafe { R_set_altrep_Inspect_method(self, fun) }
    }

    // endregion

    // region: Vector-level method setters

    /// Set the Dataptr method.
    /// # Safety
    /// Must be called during R initialization.
    #[inline]
    pub unsafe fn set_dataptr_method(self, fun: R_altvec_Dataptr_method_t) {
        unsafe { R_set_altvec_Dataptr_method(self, fun) }
    }

    /// Set the Dataptr_or_null method.
    /// # Safety
    /// Must be called during R initialization.
    #[inline]
    pub unsafe fn set_dataptr_or_null_method(self, fun: R_altvec_Dataptr_or_null_method_t) {
        unsafe { R_set_altvec_Dataptr_or_null_method(self, fun) }
    }

    /// Set the Extract_subset method.
    /// # Safety
    /// Must be called during R initialization.
    #[inline]
    pub unsafe fn set_extract_subset_method(self, fun: R_altvec_Extract_subset_method_t) {
        unsafe { R_set_altvec_Extract_subset_method(self, fun) }
    }

    // endregion

    // region: Integer method setters

    /// Set the integer Elt method.
    /// # Safety
    /// Must be called during R initialization.
    #[inline]
    pub unsafe fn set_integer_elt_method(self, fun: R_altinteger_Elt_method_t) {
        unsafe { R_set_altinteger_Elt_method(self, fun) }
    }

    /// Set the integer Get_region method.
    /// # Safety
    /// Must be called during R initialization.
    #[inline]
    pub unsafe fn set_integer_get_region_method(self, fun: R_altinteger_Get_region_method_t) {
        unsafe { R_set_altinteger_Get_region_method(self, fun) }
    }

    /// Set the integer Is_sorted method.
    /// # Safety
    /// Must be called during R initialization.
    #[inline]
    pub unsafe fn set_integer_is_sorted_method(self, fun: R_altinteger_Is_sorted_method_t) {
        unsafe { R_set_altinteger_Is_sorted_method(self, fun) }
    }

    /// Set the integer No_NA method.
    /// # Safety
    /// Must be called during R initialization.
    #[inline]
    pub unsafe fn set_integer_no_na_method(self, fun: R_altinteger_No_NA_method_t) {
        unsafe { R_set_altinteger_No_NA_method(self, fun) }
    }

    /// Set the integer Sum method.
    /// # Safety
    /// Must be called during R initialization.
    #[inline]
    pub unsafe fn set_integer_sum_method(self, fun: R_altinteger_Sum_method_t) {
        unsafe { R_set_altinteger_Sum_method(self, fun) }
    }

    /// Set the integer Min method.
    /// # Safety
    /// Must be called during R initialization.
    #[inline]
    pub unsafe fn set_integer_min_method(self, fun: R_altinteger_Min_method_t) {
        unsafe { R_set_altinteger_Min_method(self, fun) }
    }

    /// Set the integer Max method.
    /// # Safety
    /// Must be called during R initialization.
    #[inline]
    pub unsafe fn set_integer_max_method(self, fun: R_altinteger_Max_method_t) {
        unsafe { R_set_altinteger_Max_method(self, fun) }
    }

    // endregion

    // region: Real method setters

    /// Set the real Elt method.
    /// # Safety
    /// Must be called during R initialization.
    #[inline]
    pub unsafe fn set_real_elt_method(self, fun: R_altreal_Elt_method_t) {
        unsafe { R_set_altreal_Elt_method(self, fun) }
    }

    /// Set the real Get_region method.
    /// # Safety
    /// Must be called during R initialization.
    #[inline]
    pub unsafe fn set_real_get_region_method(self, fun: R_altreal_Get_region_method_t) {
        unsafe { R_set_altreal_Get_region_method(self, fun) }
    }

    /// Set the real Is_sorted method.
    /// # Safety
    /// Must be called during R initialization.
    #[inline]
    pub unsafe fn set_real_is_sorted_method(self, fun: R_altreal_Is_sorted_method_t) {
        unsafe { R_set_altreal_Is_sorted_method(self, fun) }
    }

    /// Set the real No_NA method.
    /// # Safety
    /// Must be called during R initialization.
    #[inline]
    pub unsafe fn set_real_no_na_method(self, fun: R_altreal_No_NA_method_t) {
        unsafe { R_set_altreal_No_NA_method(self, fun) }
    }

    /// Set the real Sum method.
    /// # Safety
    /// Must be called during R initialization.
    #[inline]
    pub unsafe fn set_real_sum_method(self, fun: R_altreal_Sum_method_t) {
        unsafe { R_set_altreal_Sum_method(self, fun) }
    }

    /// Set the real Min method.
    /// # Safety
    /// Must be called during R initialization.
    #[inline]
    pub unsafe fn set_real_min_method(self, fun: R_altreal_Min_method_t) {
        unsafe { R_set_altreal_Min_method(self, fun) }
    }

    /// Set the real Max method.
    /// # Safety
    /// Must be called during R initialization.
    #[inline]
    pub unsafe fn set_real_max_method(self, fun: R_altreal_Max_method_t) {
        unsafe { R_set_altreal_Max_method(self, fun) }
    }

    // endregion

    // region: Logical method setters

    /// Set the logical Elt method.
    /// # Safety
    /// Must be called during R initialization.
    #[inline]
    pub unsafe fn set_logical_elt_method(self, fun: R_altlogical_Elt_method_t) {
        unsafe { R_set_altlogical_Elt_method(self, fun) }
    }

    /// Set the logical Get_region method.
    /// # Safety
    /// Must be called during R initialization.
    #[inline]
    pub unsafe fn set_logical_get_region_method(self, fun: R_altlogical_Get_region_method_t) {
        unsafe { R_set_altlogical_Get_region_method(self, fun) }
    }

    /// Set the logical Is_sorted method.
    /// # Safety
    /// Must be called during R initialization.
    #[inline]
    pub unsafe fn set_logical_is_sorted_method(self, fun: R_altlogical_Is_sorted_method_t) {
        unsafe { R_set_altlogical_Is_sorted_method(self, fun) }
    }

    /// Set the logical No_NA method.
    /// # Safety
    /// Must be called during R initialization.
    #[inline]
    pub unsafe fn set_logical_no_na_method(self, fun: R_altlogical_No_NA_method_t) {
        unsafe { R_set_altlogical_No_NA_method(self, fun) }
    }

    /// Set the logical Sum method.
    /// # Safety
    /// Must be called during R initialization.
    #[inline]
    pub unsafe fn set_logical_sum_method(self, fun: R_altlogical_Sum_method_t) {
        unsafe { R_set_altlogical_Sum_method(self, fun) }
    }

    // endregion

    // region: Raw method setters

    /// Set the raw Elt method.
    /// # Safety
    /// Must be called during R initialization.
    #[inline]
    pub unsafe fn set_raw_elt_method(self, fun: R_altraw_Elt_method_t) {
        unsafe { R_set_altraw_Elt_method(self, fun) }
    }

    /// Set the raw Get_region method.
    /// # Safety
    /// Must be called during R initialization.
    #[inline]
    pub unsafe fn set_raw_get_region_method(self, fun: R_altraw_Get_region_method_t) {
        unsafe { R_set_altraw_Get_region_method(self, fun) }
    }

    // endregion

    // region: Complex method setters

    /// Set the complex Elt method.
    /// # Safety
    /// Must be called during R initialization.
    #[inline]
    pub unsafe fn set_complex_elt_method(self, fun: R_altcomplex_Elt_method_t) {
        unsafe { R_set_altcomplex_Elt_method(self, fun) }
    }

    /// Set the complex Get_region method.
    /// # Safety
    /// Must be called during R initialization.
    #[inline]
    pub unsafe fn set_complex_get_region_method(self, fun: R_altcomplex_Get_region_method_t) {
        unsafe { R_set_altcomplex_Get_region_method(self, fun) }
    }

    // endregion

    // region: String method setters

    /// Set the string Elt method.
    /// # Safety
    /// Must be called during R initialization.
    #[inline]
    pub unsafe fn set_string_elt_method(self, fun: R_altstring_Elt_method_t) {
        unsafe { R_set_altstring_Elt_method(self, fun) }
    }

    /// Set the string Set_elt method.
    /// # Safety
    /// Must be called during R initialization.
    #[inline]
    pub unsafe fn set_string_set_elt_method(self, fun: R_altstring_Set_elt_method_t) {
        unsafe { R_set_altstring_Set_elt_method(self, fun) }
    }

    /// Set the string Is_sorted method.
    /// # Safety
    /// Must be called during R initialization.
    #[inline]
    pub unsafe fn set_string_is_sorted_method(self, fun: R_altstring_Is_sorted_method_t) {
        unsafe { R_set_altstring_Is_sorted_method(self, fun) }
    }

    /// Set the string No_NA method.
    /// # Safety
    /// Must be called during R initialization.
    #[inline]
    pub unsafe fn set_string_no_na_method(self, fun: R_altstring_No_NA_method_t) {
        unsafe { R_set_altstring_No_NA_method(self, fun) }
    }

    // endregion

    // region: List method setters

    /// Set the list Elt method.
    /// # Safety
    /// Must be called during R initialization.
    #[inline]
    pub unsafe fn set_list_elt_method(self, fun: R_altlist_Elt_method_t) {
        unsafe { R_set_altlist_Elt_method(self, fun) }
    }

    /// Set the list Set_elt method.
    /// # Safety
    /// Must be called during R initialization.
    #[inline]
    pub unsafe fn set_list_set_elt_method(self, fun: R_altlist_Set_elt_method_t) {
        unsafe { R_set_altlist_Set_elt_method(self, fun) }
    }

    // endregion
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
