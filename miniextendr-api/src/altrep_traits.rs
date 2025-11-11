//! Safe, idiomatic ALTREP trait hierarchy mirroring R's method tables.
//!
//! These traits model the inheritance structure of ALTREP classes:
//! - Altrep (base)
//! - AltVec (extends Altrep)
//! - AltInteger, AltReal, AltLogical, AltRaw, AltString, AltList (extend AltVec)
//!
//! Each method returns `Option<...>` and defaults to `None`, which corresponds
//! to “no override” so that R’s default behavior applies. Implementors can
//! override just the methods they need without dealing with `unsafe`.
//!
//! Bridging from these traits to R’s C ABI is handled elsewhere by unsafe
//! trampolines that only set the methods which are `Some(...)`.

use crate::ffi::{R_xlen_t, SEXP, SEXPTYPE};
use core::ffi::c_void;

/// Base ALTREP methods (Length, Duplicate, Coerce, Inspect, Serialization).
pub trait Altrep {
    const HAS_LENGTH: bool = false;
    fn length(_x: SEXP) -> R_xlen_t {
        unreachable!("HAS_LENGTH = false")
    }

    const HAS_SERIALIZED_STATE: bool = false;
    fn serialized_state(_x: SEXP) -> SEXP {
        unreachable!()
    }
    fn unserialize_ex(_class: SEXP, _state: SEXP, _attr: SEXP, _objf: i32, _levs: i32) -> SEXP {
        unreachable!()
    }
    const HAS_UNSERIALIZE_EX: bool = false;
    fn unserialize(_class: SEXP, _state: SEXP) -> SEXP {
        unreachable!()
    }
    const HAS_UNSERIALIZE: bool = false;

    const HAS_DUPLICATE: bool = false;
    fn duplicate(_x: SEXP, _deep: bool) -> SEXP {
        unreachable!()
    }
    const HAS_DUPLICATE_EX: bool = false;
    fn duplicate_ex(_x: SEXP, _deep: bool) -> SEXP {
        unreachable!()
    }

    const HAS_COERCE: bool = false;
    fn coerce(_x: SEXP, _to_type: SEXPTYPE) -> SEXP {
        unreachable!()
    }

    /// Return `Some(true/false)` to override; `None` to keep default.
    const HAS_INSPECT: bool = false;
    fn inspect(_x: SEXP, _pre: i32, _deep: i32, _pvec: i32) -> bool {
        unreachable!()
    }
}

/// Vector-level hooks.
pub trait AltVec: Altrep {
    const HAS_DATAPTR: bool = false;
    fn dataptr(_x: SEXP, _writable: bool) -> *mut c_void {
        unreachable!()
    }
    const HAS_DATAPTR_OR_NULL: bool = false;
    fn dataptr_or_null(_x: SEXP) -> *const c_void {
        unreachable!()
    }
    const HAS_EXTRACT_SUBSET: bool = false;
    fn extract_subset(_x: SEXP, _indx: SEXP, _call: SEXP) -> SEXP {
        unreachable!()
    }
}

/// Integer vector methods.
pub trait AltInteger: AltVec {
    const HAS_ELT: bool = false;
    fn elt(_x: SEXP, _i: R_xlen_t) -> i32 {
        unreachable!()
    }
    const HAS_GET_REGION: bool = false;
    fn get_region(_x: SEXP, _i: R_xlen_t, _n: R_xlen_t, _buf: *mut i32) -> R_xlen_t {
        unreachable!()
    }
    const HAS_IS_SORTED: bool = false;
    fn is_sorted(_x: SEXP) -> i32 {
        unreachable!()
    }
    const HAS_NO_NA: bool = false;
    fn no_na(_x: SEXP) -> i32 {
        unreachable!()
    }
    const HAS_SUM: bool = false;
    fn sum(_x: SEXP, _narm: bool) -> SEXP {
        unreachable!()
    }
    const HAS_MIN: bool = false;
    fn min(_x: SEXP, _narm: bool) -> SEXP {
        unreachable!()
    }
    const HAS_MAX: bool = false;
    fn max(_x: SEXP, _narm: bool) -> SEXP {
        unreachable!()
    }
}

/// Real vector methods.
pub trait AltReal: AltVec {
    const HAS_ELT: bool = false;
    fn elt(_x: SEXP, _i: R_xlen_t) -> f64 {
        unreachable!()
    }
    const HAS_GET_REGION: bool = false;
    fn get_region(_x: SEXP, _i: R_xlen_t, _n: R_xlen_t, _buf: *mut f64) -> R_xlen_t {
        unreachable!()
    }
    const HAS_IS_SORTED: bool = false;
    fn is_sorted(_x: SEXP) -> i32 {
        unreachable!()
    }
    const HAS_NO_NA: bool = false;
    fn no_na(_x: SEXP) -> i32 {
        unreachable!()
    }
    const HAS_SUM: bool = false;
    fn sum(_x: SEXP, _narm: bool) -> SEXP {
        unreachable!()
    }
    const HAS_MIN: bool = false;
    fn min(_x: SEXP, _narm: bool) -> SEXP {
        unreachable!()
    }
    const HAS_MAX: bool = false;
    fn max(_x: SEXP, _narm: bool) -> SEXP {
        unreachable!()
    }
}

/// Logical vector methods.
pub trait AltLogical: AltVec {
    const HAS_ELT: bool = false;
    fn elt(_x: SEXP, _i: R_xlen_t) -> i32 {
        unreachable!()
    }
    const HAS_GET_REGION: bool = false;
    fn get_region(_x: SEXP, _i: R_xlen_t, _n: R_xlen_t, _buf: *mut i32) -> R_xlen_t {
        unreachable!()
    }
    const HAS_IS_SORTED: bool = false;
    fn is_sorted(_x: SEXP) -> i32 {
        unreachable!()
    }
    const HAS_NO_NA: bool = false;
    fn no_na(_x: SEXP) -> i32 {
        unreachable!()
    }
    const HAS_SUM: bool = false;
    fn sum(_x: SEXP, _narm: bool) -> SEXP {
        unreachable!()
    }
}

/// Raw vector methods.
pub trait AltRaw: AltVec {
    const HAS_ELT: bool = false;
    fn elt(_x: SEXP, _i: R_xlen_t) -> u8 {
        unreachable!()
    }
    const HAS_GET_REGION: bool = false;
    fn get_region(_x: SEXP, _i: R_xlen_t, _n: R_xlen_t, _buf: *mut u8) -> R_xlen_t {
        unreachable!()
    }
}

/// String vector methods.
pub trait AltString: AltVec {
    const HAS_ELT: bool = false;
    fn elt(_x: SEXP, _i: R_xlen_t) -> SEXP {
        unreachable!()
    }
    const HAS_SET_ELT: bool = false;
    fn set_elt(_x: SEXP, _i: R_xlen_t, _v: SEXP) {
        unreachable!()
    }
    const HAS_IS_SORTED: bool = false;
    fn is_sorted(_x: SEXP) -> i32 {
        unreachable!()
    }
    const HAS_NO_NA: bool = false;
    fn no_na(_x: SEXP) -> i32 {
        unreachable!()
    }
}

/// List (VECSXP) methods.
pub trait AltList: AltVec {
    const HAS_ELT: bool = false;
    fn elt(_x: SEXP, _i: R_xlen_t) -> SEXP {
        unreachable!()
    }
    const HAS_SET_ELT: bool = false;
    fn set_elt(_x: SEXP, _i: R_xlen_t, _v: SEXP) {
        unreachable!()
    }
}
