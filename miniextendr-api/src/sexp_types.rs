//! Core type vocabulary for R values: `SEXPTYPE`, `R_xlen_t`, `Rbyte`,
//! `Rcomplex`, `RLogical`, `Rboolean`, `cetype_t`, `R_CFinalizer_t`, and
//! the `RNativeType` marker trait + impls.
//!
//! These types are the bridge between raw R FFI (in `crate::sys`) and the
//! safe Rust API on `SEXP` (in `crate::sexp` and `crate::sexp_ext`).
//! Most user code reaches them via [`crate::prelude`].

use crate::SEXP;
use crate::altrep_traits::NA_REAL;
use crate::sys::{
    COMPLEX, COMPLEX_ELT, INTEGER, INTEGER_ELT, LOGICAL, LOGICAL_ELT, RAW, RAW_ELT, REAL, REAL_ELT,
    Rf_type2char, Rf_xlength,
};

#[allow(non_camel_case_types)]
/// R's extended vector length type (`R_xlen_t`).
pub type R_xlen_t = isize;
/// R byte element type used by `RAWSXP`.
pub type Rbyte = ::std::os::raw::c_uchar;

/// R's complex scalar layout (`Rcomplex`).
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Rcomplex {
    /// Real part.
    pub r: f64,
    /// Imaginary part.
    pub i: f64,
}

/// R S-expression tag values (`SEXPTYPE`).
#[repr(u32)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum SEXPTYPE {
    #[doc = " nil = NULL"]
    NILSXP = 0,
    #[doc = " symbols"]
    SYMSXP = 1,
    #[doc = " lists of dotted pairs"]
    LISTSXP = 2,
    #[doc = " closures"]
    CLOSXP = 3,
    #[doc = " environments"]
    ENVSXP = 4,
    #[doc = r" promises: \[un\]evaluated closure arguments"]
    PROMSXP = 5,
    #[doc = " language constructs (special lists)"]
    LANGSXP = 6,
    #[doc = " special forms"]
    SPECIALSXP = 7,
    #[doc = " builtin non-special forms"]
    BUILTINSXP = 8,
    #[doc = " \"scalar\" string type (internal only)"]
    CHARSXP = 9,
    #[doc = " logical vectors"]
    LGLSXP = 10,
    #[doc = " integer vectors"]
    INTSXP = 13,
    #[doc = " real variables"]
    REALSXP = 14,
    #[doc = " complex variables"]
    CPLXSXP = 15,
    #[doc = " string vectors"]
    STRSXP = 16,
    #[doc = " dot-dot-dot object"]
    DOTSXP = 17,
    #[doc = " make \"any\" args work"]
    ANYSXP = 18,
    #[doc = " generic vectors"]
    VECSXP = 19,
    #[doc = " expressions vectors"]
    EXPRSXP = 20,
    #[doc = " byte code"]
    BCODESXP = 21,
    #[doc = " external pointer"]
    EXTPTRSXP = 22,
    #[doc = " weak reference"]
    WEAKREFSXP = 23,
    #[doc = " raw bytes"]
    RAWSXP = 24,
    #[doc = " S4 non-vector"]
    S4SXP = 25,
    #[doc = " fresh node created in new page"]
    NEWSXP = 30,
    #[doc = " node released by GC"]
    FREESXP = 31,
    #[doc = " Closure or Builtin"]
    FUNSXP = 99,
}

impl SEXPTYPE {
    /// Alias for `S4SXP` (value 25).
    ///
    /// R defines both `OBJSXP` and `S4SXP` as value 25. `S4SXP` is retained
    /// for backwards compatibility; `OBJSXP` is the preferred name.
    pub const OBJSXP: SEXPTYPE = SEXPTYPE::S4SXP;

    /// Get R's name for this SEXPTYPE (e.g. `"double"`, `"integer"`, `"list"`).
    ///
    /// Returns the same string as R's `typeof()` function.
    #[inline]
    pub fn type_name(self) -> &'static str {
        let cstr = unsafe { Rf_type2char(self) };
        // SAFETY: R's type names are static ASCII strings
        unsafe { std::ffi::CStr::from_ptr(cstr) }
            .to_str()
            .unwrap_or("unknown")
    }
}

/// Marker trait for types that correspond to R's native vector element types.
///
/// This enables blanket implementations for `TryFromSexp` and safe conversions.
pub trait RNativeType: Sized + Copy + 'static {
    /// The SEXPTYPE for vectors containing this element type.
    const SEXP_TYPE: SEXPTYPE;

    /// The per-type `NA` (missing-value) sentinel used when filling vector slots
    /// that have no source value (e.g. sparse scatter into a longer column).
    ///
    /// - `f64`   → `NA_REAL` (R's canonical NA double bit pattern, *not* a plain `NaN`)
    /// - `i32`   → `i32::MIN` (`NA_INTEGER`)
    /// - `RLogical` → `RLogical::NA` (`NA_LOGICAL`)
    /// - `Rcomplex` → both parts `NA_REAL`
    /// - `u8` (RAWSXP) → `0` — **R's raw type has no NA**, so absent positions
    ///   become `0x00` rather than a missing marker.
    const R_NA: Self;

    /// Get mutable pointer to vector data.
    ///
    /// For empty vectors (length 0), returns an aligned dangling pointer rather than
    /// R's internal 0x1 sentinel, which isn't properly aligned for most types.
    /// This allows safe creation of zero-length slices with `std::slice::from_raw_parts_mut`.
    ///
    /// # Safety
    ///
    /// - `sexp` must be a valid, non-null SEXP of the corresponding vector type.
    /// - For ALTREP vectors, this may trigger materialization.
    unsafe fn dataptr_mut(sexp: SEXP) -> *mut Self;

    /// Read the i-th element via the appropriate `*_ELT` accessor.
    ///
    /// Goes through R's ALTREP dispatch for ALTREP vectors.
    fn elt(sexp: SEXP, i: isize) -> Self;
}

/// R's logical element type (the contents of a `LGLSXP` vector).
///
/// In R, logical vectors are stored as `int` with possible values:
/// - `0` for FALSE
/// - `1` for TRUE
/// - `NA_LOGICAL` (typically `INT_MIN`) for NA
///
/// **Important:** R may also contain other non-zero values in logical vectors
/// (e.g., from low-level code). Those should be interpreted as TRUE.
///
/// This type is `repr(transparent)` over `i32` so *any* raw value is valid,
/// avoiding UB when viewing `LGLSXP` data as a slice.
#[repr(transparent)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct RLogical(i32);

impl RLogical {
    /// FALSE logical scalar.
    pub const FALSE: Self = Self(0);
    /// TRUE logical scalar.
    pub const TRUE: Self = Self(1);
    /// Missing logical scalar (`NA_LOGICAL`).
    pub const NA: Self = Self(i32::MIN);

    /// Construct directly from raw R logical storage.
    #[inline]
    pub const fn from_i32(raw: i32) -> Self {
        Self(raw)
    }

    /// Get raw R logical storage value.
    #[inline]
    pub const fn to_i32(self) -> i32 {
        self.0
    }

    /// Returns whether the value is `NA_LOGICAL`.
    #[inline]
    pub const fn is_na(self) -> bool {
        self.0 == i32::MIN
    }

    /// Convert to Rust `Option<bool>` (`None` for `NA`).
    #[inline]
    pub const fn to_option_bool(self) -> Option<bool> {
        match self.0 {
            0 => Some(false),
            i32::MIN => None,
            _ => Some(true),
        }
    }
}

impl From<bool> for RLogical {
    #[inline]
    fn from(value: bool) -> Self {
        if value { Self::TRUE } else { Self::FALSE }
    }
}

impl RNativeType for i32 {
    const SEXP_TYPE: SEXPTYPE = SEXPTYPE::INTSXP;
    const R_NA: Self = i32::MIN; // NA_INTEGER

    #[inline]
    unsafe fn dataptr_mut(sexp: SEXP) -> *mut Self {
        unsafe {
            if Rf_xlength(sexp) == 0 {
                std::ptr::NonNull::<Self>::dangling().as_ptr()
            } else {
                INTEGER(sexp)
            }
        }
    }

    #[inline]
    fn elt(sexp: SEXP, i: isize) -> Self {
        unsafe { INTEGER_ELT(sexp, i) }
    }
}

impl RNativeType for f64 {
    const SEXP_TYPE: SEXPTYPE = SEXPTYPE::REALSXP;
    const R_NA: Self = NA_REAL; // R's canonical NA double bit pattern, not f64::NAN

    #[inline]
    unsafe fn dataptr_mut(sexp: SEXP) -> *mut Self {
        unsafe {
            if Rf_xlength(sexp) == 0 {
                std::ptr::NonNull::<Self>::dangling().as_ptr()
            } else {
                REAL(sexp)
            }
        }
    }

    #[inline]
    fn elt(sexp: SEXP, i: isize) -> Self {
        unsafe { REAL_ELT(sexp, i) }
    }
}

impl RNativeType for u8 {
    const SEXP_TYPE: SEXPTYPE = SEXPTYPE::RAWSXP;
    // R's raw vectors have no NA; absent scatter positions become 0x00.
    const R_NA: Self = 0;

    #[inline]
    unsafe fn dataptr_mut(sexp: SEXP) -> *mut Self {
        unsafe {
            if Rf_xlength(sexp) == 0 {
                std::ptr::NonNull::<Self>::dangling().as_ptr()
            } else {
                RAW(sexp)
            }
        }
    }

    #[inline]
    fn elt(sexp: SEXP, i: isize) -> Self {
        unsafe { RAW_ELT(sexp, i) }
    }
}

impl RNativeType for RLogical {
    const SEXP_TYPE: SEXPTYPE = SEXPTYPE::LGLSXP;
    const R_NA: Self = RLogical::NA; // NA_LOGICAL

    #[inline]
    unsafe fn dataptr_mut(sexp: SEXP) -> *mut Self {
        // LOGICAL returns *mut c_int, RLogical is repr(transparent) over i32
        unsafe {
            if Rf_xlength(sexp) == 0 {
                std::ptr::NonNull::<Self>::dangling().as_ptr()
            } else {
                LOGICAL(sexp).cast()
            }
        }
    }

    #[inline]
    fn elt(sexp: SEXP, i: isize) -> Self {
        RLogical(unsafe { LOGICAL_ELT(sexp, i) })
    }
}

impl RNativeType for Rcomplex {
    const SEXP_TYPE: SEXPTYPE = SEXPTYPE::CPLXSXP;
    const R_NA: Self = Rcomplex {
        r: NA_REAL,
        i: NA_REAL,
    };

    #[inline]
    unsafe fn dataptr_mut(sexp: SEXP) -> *mut Self {
        unsafe {
            if Rf_xlength(sexp) == 0 {
                std::ptr::NonNull::<Self>::dangling().as_ptr()
            } else {
                COMPLEX(sexp)
            }
        }
    }

    #[inline]
    fn elt(sexp: SEXP, i: isize) -> Self {
        unsafe { COMPLEX_ELT(sexp, i) }
    }
}

#[repr(i32)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
/// Binary boolean used by many R C APIs.
pub enum Rboolean {
    /// False.
    FALSE = 0,
    /// True.
    TRUE = 1,
}

impl From<bool> for Rboolean {
    fn from(value: bool) -> Self {
        match value {
            true => Rboolean::TRUE,
            false => Rboolean::FALSE,
        }
    }
}

impl From<Rboolean> for bool {
    fn from(value: Rboolean) -> Self {
        match value {
            Rboolean::FALSE => false,
            Rboolean::TRUE => true,
        }
    }
}

#[allow(non_camel_case_types)]
/// C finalizer callback signature used by external pointers.
pub type R_CFinalizer_t = ::std::option::Option<unsafe extern "C-unwind" fn(s: SEXP)>;

#[repr(C)]
#[derive(Copy, Clone)]
#[allow(non_camel_case_types)]
/// Character encoding tag used by CHARSXP constructors.
pub enum cetype_t {
    /// Native locale encoding.
    CE_NATIVE = 0,
    /// UTF-8 encoding.
    CE_UTF8 = 1,
    /// Latin-1 encoding.
    CE_LATIN1 = 2,
    /// Raw bytes encoding.
    CE_BYTES = 3,
    /// Symbol encoding marker.
    CE_SYMBOL = 5,
    /// Any encoding accepted.
    CE_ANY = 99,
}
pub use cetype_t::CE_UTF8;
