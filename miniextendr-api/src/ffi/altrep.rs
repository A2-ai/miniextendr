#![allow(non_camel_case_types)]
use crate::ffi::{DllInfo, R_xlen_t, Rboolean, Rbyte, Rcomplex, SEXP, SEXPTYPE};

#[allow(non_camel_case_types)]
pub type R_altrep_Coerce_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP, rtype: SEXPTYPE) -> SEXP>;

pub type R_altrep_UnserializeEX_method_t = ::std::option::Option<
    unsafe extern "C-unwind" fn(
        class: SEXP,
        state: SEXP,
        attr: SEXP,
        objf: ::std::os::raw::c_int,
        levs: ::std::os::raw::c_int,
    ) -> SEXP,
>;
pub type R_altrep_Unserialize_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(class: SEXP, state: SEXP) -> SEXP>;
pub type R_altrep_Serialized_state_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP) -> SEXP>;
pub type R_altrep_DuplicateEX_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP, deep: Rboolean) -> SEXP>;
pub type R_altrep_Duplicate_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP, deep: Rboolean) -> SEXP>;
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
pub type R_altrep_Length_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP) -> R_xlen_t>;
pub type R_altvec_Dataptr_method_t = ::std::option::Option<
    unsafe extern "C-unwind" fn(x: SEXP, writable: Rboolean) -> *mut ::std::os::raw::c_void,
>;
pub type R_altvec_Dataptr_or_null_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP) -> *const ::std::os::raw::c_void>;
pub type R_altvec_Extract_subset_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP, indx: SEXP, call: SEXP) -> SEXP>;
pub type R_altinteger_Elt_method_t = ::std::option::Option<
    unsafe extern "C-unwind" fn(x: SEXP, i: R_xlen_t) -> ::std::os::raw::c_int,
>;
pub type R_altinteger_Get_region_method_t = ::std::option::Option<
    unsafe extern "C-unwind" fn(
        sx: SEXP,
        i: R_xlen_t,
        n: R_xlen_t,
        buf: *mut ::std::os::raw::c_int,
    ) -> R_xlen_t,
>;
pub type R_altinteger_Is_sorted_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP) -> ::std::os::raw::c_int>;
pub type R_altinteger_No_NA_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP) -> ::std::os::raw::c_int>;
pub type R_altinteger_Sum_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP, narm: Rboolean) -> SEXP>;
pub type R_altinteger_Min_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP, narm: Rboolean) -> SEXP>;
pub type R_altinteger_Max_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP, narm: Rboolean) -> SEXP>;
pub type R_altreal_Elt_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP, i: R_xlen_t) -> f64>;
pub type R_altreal_Get_region_method_t = ::std::option::Option<
    unsafe extern "C-unwind" fn(sx: SEXP, i: R_xlen_t, n: R_xlen_t, buf: *mut f64) -> R_xlen_t,
>;
pub type R_altreal_Is_sorted_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP) -> ::std::os::raw::c_int>;
pub type R_altreal_No_NA_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP) -> ::std::os::raw::c_int>;
pub type R_altreal_Sum_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP, narm: Rboolean) -> SEXP>;
pub type R_altreal_Min_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP, narm: Rboolean) -> SEXP>;
pub type R_altreal_Max_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP, narm: Rboolean) -> SEXP>;
pub type R_altlogical_Elt_method_t = ::std::option::Option<
    unsafe extern "C-unwind" fn(x: SEXP, i: R_xlen_t) -> ::std::os::raw::c_int,
>;
pub type R_altlogical_Get_region_method_t = ::std::option::Option<
    unsafe extern "C-unwind" fn(
        sx: SEXP,
        i: R_xlen_t,
        n: R_xlen_t,
        buf: *mut ::std::os::raw::c_int,
    ) -> R_xlen_t,
>;
pub type R_altlogical_Is_sorted_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP) -> ::std::os::raw::c_int>;
pub type R_altlogical_No_NA_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP) -> ::std::os::raw::c_int>;
pub type R_altlogical_Sum_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP, narm: Rboolean) -> SEXP>;
pub type R_altraw_Elt_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP, i: R_xlen_t) -> Rbyte>;
pub type R_altraw_Get_region_method_t = ::std::option::Option<
    unsafe extern "C-unwind" fn(sx: SEXP, i: R_xlen_t, n: R_xlen_t, buf: *mut Rbyte) -> R_xlen_t,
>;
pub type R_altcomplex_Elt_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP, i: R_xlen_t) -> Rcomplex>;
pub type R_altcomplex_Get_region_method_t = ::std::option::Option<
    unsafe extern "C-unwind" fn(sx: SEXP, i: R_xlen_t, n: R_xlen_t, buf: *mut Rcomplex) -> R_xlen_t,
>;
pub type R_altstring_Elt_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP, i: R_xlen_t) -> SEXP>;
pub type R_altstring_Set_elt_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP, i: R_xlen_t, v: SEXP)>;
pub type R_altstring_Is_sorted_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP) -> ::std::os::raw::c_int>;
pub type R_altstring_No_NA_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP) -> ::std::os::raw::c_int>;
pub type R_altlist_Elt_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP, i: R_xlen_t) -> SEXP>;
pub type R_altlist_Set_elt_method_t =
    ::std::option::Option<unsafe extern "C-unwind" fn(x: SEXP, i: R_xlen_t, v: SEXP)>;
#[derive(Clone, Copy)]
#[repr(C)]
pub struct R_altrep_class_t {
    pub ptr: SEXP,
}

// SAFETY: R_altrep_class_t is only used on R's main thread.
// The class is created once during package init and stored in a static.
unsafe impl Send for R_altrep_class_t {}
unsafe impl Sync for R_altrep_class_t {}

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

// region: ALTREP Class Registration Builder

/// Builder for registering ALTREP classes with type-safe method setters.
///
/// # Example
///
/// ```ignore
/// let class = AltrepClassBuilder::new_integer(cname, pname, info)
///     .elt(my_elt_method)
///     .get_region(my_get_region_method)
///     .length(my_length_method)
///     .build();
/// ```
pub struct AltrepClassBuilder {
    class: R_altrep_class_t,
}

impl AltrepClassBuilder {
    /// Create a new ALTINTEGER class builder.
    ///
    /// # Safety
    ///
    /// - `cname` and `pname` must be valid null-terminated C strings
    /// - `info` must be a valid `DllInfo` pointer
    /// - Must be called during package initialization
    #[inline]
    pub unsafe fn new_integer(
        cname: *const ::std::os::raw::c_char,
        pname: *const ::std::os::raw::c_char,
        info: *mut DllInfo,
    ) -> Self {
        Self {
            class: unsafe { R_make_altinteger_class(cname, pname, info) },
        }
    }

    /// Create a new ALTREAL class builder.
    ///
    /// # Safety
    ///
    /// - `cname` and `pname` must be valid null-terminated C strings
    /// - `info` must be a valid `DllInfo` pointer
    /// - Must be called during package initialization
    #[inline]
    pub unsafe fn new_real(
        cname: *const ::std::os::raw::c_char,
        pname: *const ::std::os::raw::c_char,
        info: *mut DllInfo,
    ) -> Self {
        Self {
            class: unsafe { R_make_altreal_class(cname, pname, info) },
        }
    }

    /// Create a new ALTLOGICAL class builder.
    ///
    /// # Safety
    ///
    /// - `cname` and `pname` must be valid null-terminated C strings
    /// - `info` must be a valid `DllInfo` pointer
    /// - Must be called during package initialization
    #[inline]
    pub unsafe fn new_logical(
        cname: *const ::std::os::raw::c_char,
        pname: *const ::std::os::raw::c_char,
        info: *mut DllInfo,
    ) -> Self {
        Self {
            class: unsafe { R_make_altlogical_class(cname, pname, info) },
        }
    }

    /// Create a new ALTSTRING class builder.
    ///
    /// # Safety
    ///
    /// - `cname` and `pname` must be valid null-terminated C strings
    /// - `info` must be a valid `DllInfo` pointer
    /// - Must be called during package initialization
    #[inline]
    pub unsafe fn new_string(
        cname: *const ::std::os::raw::c_char,
        pname: *const ::std::os::raw::c_char,
        info: *mut DllInfo,
    ) -> Self {
        Self {
            class: unsafe { R_make_altstring_class(cname, pname, info) },
        }
    }

    /// Create a new ALTRAW class builder.
    ///
    /// # Safety
    ///
    /// - `cname` and `pname` must be valid null-terminated C strings
    /// - `info` must be a valid `DllInfo` pointer
    /// - Must be called during package initialization
    #[inline]
    pub unsafe fn new_raw(
        cname: *const ::std::os::raw::c_char,
        pname: *const ::std::os::raw::c_char,
        info: *mut DllInfo,
    ) -> Self {
        Self {
            class: unsafe { R_make_altraw_class(cname, pname, info) },
        }
    }

    /// Create a new ALTCOMPLEX class builder.
    ///
    /// # Safety
    ///
    /// - `cname` and `pname` must be valid null-terminated C strings
    /// - `info` must be a valid `DllInfo` pointer
    /// - Must be called during package initialization
    #[inline]
    pub unsafe fn new_complex(
        cname: *const ::std::os::raw::c_char,
        pname: *const ::std::os::raw::c_char,
        info: *mut DllInfo,
    ) -> Self {
        Self {
            class: unsafe { R_make_altcomplex_class(cname, pname, info) },
        }
    }

    /// Create a new ALTLIST class builder.
    ///
    /// # Safety
    ///
    /// - `cname` and `pname` must be valid null-terminated C strings
    /// - `info` must be a valid `DllInfo` pointer
    /// - Must be called during package initialization
    #[inline]
    pub unsafe fn new_list(
        cname: *const ::std::os::raw::c_char,
        pname: *const ::std::os::raw::c_char,
        info: *mut DllInfo,
    ) -> Self {
        Self {
            class: unsafe { R_make_altlist_class(cname, pname, info) },
        }
    }

    /// Set the UnserializeEX method.
    #[inline]
    pub fn unserialize_ex(self, fun: R_altrep_UnserializeEX_method_t) -> Self {
        unsafe { R_set_altrep_UnserializeEX_method(self.class, fun) };
        self
    }

    /// Set the Unserialize method.
    #[inline]
    pub fn unserialize(self, fun: R_altrep_Unserialize_method_t) -> Self {
        unsafe { R_set_altrep_Unserialize_method(self.class, fun) };
        self
    }

    /// Set the Serialized_state method.
    #[inline]
    pub fn serialized_state(self, fun: R_altrep_Serialized_state_method_t) -> Self {
        unsafe { R_set_altrep_Serialized_state_method(self.class, fun) };
        self
    }

    /// Set the DuplicateEX method.
    #[inline]
    pub fn duplicate_ex(self, fun: R_altrep_DuplicateEX_method_t) -> Self {
        unsafe { R_set_altrep_DuplicateEX_method(self.class, fun) };
        self
    }

    /// Set the Duplicate method.
    #[inline]
    pub fn duplicate(self, fun: R_altrep_Duplicate_method_t) -> Self {
        unsafe { R_set_altrep_Duplicate_method(self.class, fun) };
        self
    }

    /// Set the Coerce method.
    #[inline]
    pub fn coerce(self, fun: R_altrep_Coerce_method_t) -> Self {
        unsafe { R_set_altrep_Coerce_method(self.class, fun) };
        self
    }

    /// Set the Inspect method.
    #[inline]
    pub fn inspect(self, fun: R_altrep_Inspect_method_t) -> Self {
        unsafe { R_set_altrep_Inspect_method(self.class, fun) };
        self
    }

    /// Set the Length method.
    #[inline]
    pub fn length(self, fun: R_altrep_Length_method_t) -> Self {
        unsafe { R_set_altrep_Length_method(self.class, fun) };
        self
    }

    /// Set the Dataptr method.
    #[inline]
    pub fn dataptr(self, fun: R_altvec_Dataptr_method_t) -> Self {
        unsafe { R_set_altvec_Dataptr_method(self.class, fun) };
        self
    }

    /// Set the Dataptr_or_null method.
    #[inline]
    pub fn dataptr_or_null(self, fun: R_altvec_Dataptr_or_null_method_t) -> Self {
        unsafe { R_set_altvec_Dataptr_or_null_method(self.class, fun) };
        self
    }

    /// Set the Extract_subset method.
    #[inline]
    pub fn extract_subset(self, fun: R_altvec_Extract_subset_method_t) -> Self {
        unsafe { R_set_altvec_Extract_subset_method(self.class, fun) };
        self
    }

    /// Set the Elt method (for integer vectors).
    #[inline]
    pub fn elt_integer(self, fun: R_altinteger_Elt_method_t) -> Self {
        unsafe { R_set_altinteger_Elt_method(self.class, fun) };
        self
    }

    /// Set the Get_region method (for integer vectors).
    #[inline]
    pub fn get_region_integer(self, fun: R_altinteger_Get_region_method_t) -> Self {
        unsafe { R_set_altinteger_Get_region_method(self.class, fun) };
        self
    }

    /// Set the Is_sorted method (for integer vectors).
    #[inline]
    pub fn is_sorted_integer(self, fun: R_altinteger_Is_sorted_method_t) -> Self {
        unsafe { R_set_altinteger_Is_sorted_method(self.class, fun) };
        self
    }

    /// Set the No_NA method (for integer vectors).
    #[inline]
    pub fn no_na_integer(self, fun: R_altinteger_No_NA_method_t) -> Self {
        unsafe { R_set_altinteger_No_NA_method(self.class, fun) };
        self
    }

    /// Set the Sum method (for integer vectors).
    #[inline]
    pub fn sum_integer(self, fun: R_altinteger_Sum_method_t) -> Self {
        unsafe { R_set_altinteger_Sum_method(self.class, fun) };
        self
    }

    /// Set the Min method (for integer vectors).
    #[inline]
    pub fn min_integer(self, fun: R_altinteger_Min_method_t) -> Self {
        unsafe { R_set_altinteger_Min_method(self.class, fun) };
        self
    }

    /// Set the Max method (for integer vectors).
    #[inline]
    pub fn max_integer(self, fun: R_altinteger_Max_method_t) -> Self {
        unsafe { R_set_altinteger_Max_method(self.class, fun) };
        self
    }

    /// Set the Elt method (for real vectors).
    #[inline]
    pub fn elt_real(self, fun: R_altreal_Elt_method_t) -> Self {
        unsafe { R_set_altreal_Elt_method(self.class, fun) };
        self
    }

    /// Set the Get_region method (for real vectors).
    #[inline]
    pub fn get_region_real(self, fun: R_altreal_Get_region_method_t) -> Self {
        unsafe { R_set_altreal_Get_region_method(self.class, fun) };
        self
    }

    /// Set the Is_sorted method (for real vectors).
    #[inline]
    pub fn is_sorted_real(self, fun: R_altreal_Is_sorted_method_t) -> Self {
        unsafe { R_set_altreal_Is_sorted_method(self.class, fun) };
        self
    }

    /// Set the No_NA method (for real vectors).
    #[inline]
    pub fn no_na_real(self, fun: R_altreal_No_NA_method_t) -> Self {
        unsafe { R_set_altreal_No_NA_method(self.class, fun) };
        self
    }

    /// Set the Sum method (for real vectors).
    #[inline]
    pub fn sum_real(self, fun: R_altreal_Sum_method_t) -> Self {
        unsafe { R_set_altreal_Sum_method(self.class, fun) };
        self
    }

    /// Set the Min method (for real vectors).
    #[inline]
    pub fn min_real(self, fun: R_altreal_Min_method_t) -> Self {
        unsafe { R_set_altreal_Min_method(self.class, fun) };
        self
    }

    /// Set the Max method (for real vectors).
    #[inline]
    pub fn max_real(self, fun: R_altreal_Max_method_t) -> Self {
        unsafe { R_set_altreal_Max_method(self.class, fun) };
        self
    }

    /// Set the Elt method (for logical vectors).
    #[inline]
    pub fn elt_logical(self, fun: R_altlogical_Elt_method_t) -> Self {
        unsafe { R_set_altlogical_Elt_method(self.class, fun) };
        self
    }

    /// Set the Get_region method (for logical vectors).
    #[inline]
    pub fn get_region_logical(self, fun: R_altlogical_Get_region_method_t) -> Self {
        unsafe { R_set_altlogical_Get_region_method(self.class, fun) };
        self
    }

    /// Set the Is_sorted method (for logical vectors).
    #[inline]
    pub fn is_sorted_logical(self, fun: R_altlogical_Is_sorted_method_t) -> Self {
        unsafe { R_set_altlogical_Is_sorted_method(self.class, fun) };
        self
    }

    /// Set the No_NA method (for logical vectors).
    #[inline]
    pub fn no_na_logical(self, fun: R_altlogical_No_NA_method_t) -> Self {
        unsafe { R_set_altlogical_No_NA_method(self.class, fun) };
        self
    }

    /// Set the Sum method (for logical vectors).
    #[inline]
    pub fn sum_logical(self, fun: R_altlogical_Sum_method_t) -> Self {
        unsafe { R_set_altlogical_Sum_method(self.class, fun) };
        self
    }

    /// Set the Elt method (for raw vectors).
    #[inline]
    pub fn elt_raw(self, fun: R_altraw_Elt_method_t) -> Self {
        unsafe { R_set_altraw_Elt_method(self.class, fun) };
        self
    }

    /// Set the Get_region method (for raw vectors).
    #[inline]
    pub fn get_region_raw(self, fun: R_altraw_Get_region_method_t) -> Self {
        unsafe { R_set_altraw_Get_region_method(self.class, fun) };
        self
    }

    /// Set the Elt method (for complex vectors).
    #[inline]
    pub fn elt_complex(self, fun: R_altcomplex_Elt_method_t) -> Self {
        unsafe { R_set_altcomplex_Elt_method(self.class, fun) };
        self
    }

    /// Set the Get_region method (for complex vectors).
    #[inline]
    pub fn get_region_complex(self, fun: R_altcomplex_Get_region_method_t) -> Self {
        unsafe { R_set_altcomplex_Get_region_method(self.class, fun) };
        self
    }

    /// Set the Elt method (for string vectors).
    #[inline]
    pub fn elt_string(self, fun: R_altstring_Elt_method_t) -> Self {
        unsafe { R_set_altstring_Elt_method(self.class, fun) };
        self
    }

    /// Set the Set_elt method (for string vectors).
    #[inline]
    pub fn set_elt_string(self, fun: R_altstring_Set_elt_method_t) -> Self {
        unsafe { R_set_altstring_Set_elt_method(self.class, fun) };
        self
    }

    /// Set the Is_sorted method (for string vectors).
    #[inline]
    pub fn is_sorted_string(self, fun: R_altstring_Is_sorted_method_t) -> Self {
        unsafe { R_set_altstring_Is_sorted_method(self.class, fun) };
        self
    }

    /// Set the No_NA method (for string vectors).
    #[inline]
    pub fn no_na_string(self, fun: R_altstring_No_NA_method_t) -> Self {
        unsafe { R_set_altstring_No_NA_method(self.class, fun) };
        self
    }

    /// Set the Elt method (for list vectors).
    #[inline]
    pub fn elt_list(self, fun: R_altlist_Elt_method_t) -> Self {
        unsafe { R_set_altlist_Elt_method(self.class, fun) };
        self
    }

    /// Set the Set_elt method (for list vectors).
    #[inline]
    pub fn set_elt_list(self, fun: R_altlist_Set_elt_method_t) -> Self {
        unsafe { R_set_altlist_Set_elt_method(self.class, fun) };
        self
    }

    /// Finalize and return the configured ALTREP class.
    #[inline]
    pub const fn build(self) -> R_altrep_class_t {
        self.class
    }
}

// endregion
