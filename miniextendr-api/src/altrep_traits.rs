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
    fn length(_x: SEXP) -> Option<R_xlen_t> { None }

    fn serialized_state(_x: SEXP) -> Option<SEXP> { None }
    fn unserialize_ex(
        _class: SEXP,
        _state: SEXP,
        _attr: SEXP,
        _objf: i32,
        _levs: i32,
    ) -> Option<SEXP> {
        None
    }
    fn unserialize(_class: SEXP, _state: SEXP) -> Option<SEXP> { None }

    fn duplicate(_x: SEXP, _deep: bool) -> Option<SEXP> { None }
    fn duplicate_ex(_x: SEXP, _deep: bool) -> Option<SEXP> { None }

    fn coerce(_x: SEXP, _to_type: SEXPTYPE) -> Option<SEXP> { None }

    /// Return `Some(true/false)` to override; `None` to keep default.
    fn inspect(_x: SEXP, _pre: i32, _deep: i32, _pvec: i32) -> Option<bool> { None }
}

/// Vector-level hooks.
pub trait AltVec: Altrep {
    fn dataptr(_x: SEXP, _writable: bool) -> Option<*mut c_void> { None }
    fn dataptr_or_null(_x: SEXP) -> Option<*const c_void> { None }
    fn extract_subset(_x: SEXP, _indx: SEXP, _call: SEXP) -> Option<SEXP> { None }
}

/// Integer vector methods.
pub trait AltInteger: AltVec {
    fn elt(_x: SEXP, _i: R_xlen_t) -> Option<i32> { None }
    fn get_region(_x: SEXP, _i: R_xlen_t, _n: R_xlen_t, _buf: *mut i32) -> Option<R_xlen_t> {
        None
    }
    fn is_sorted(_x: SEXP) -> Option<i32> { None }
    fn no_na(_x: SEXP) -> Option<i32> { None }
    fn sum(_x: SEXP, _narm: bool) -> Option<SEXP> { None }
    fn min(_x: SEXP, _narm: bool) -> Option<SEXP> { None }
    fn max(_x: SEXP, _narm: bool) -> Option<SEXP> { None }
}

/// Real vector methods.
pub trait AltReal: AltVec {
    fn elt(_x: SEXP, _i: R_xlen_t) -> Option<f64> { None }
    fn get_region(_x: SEXP, _i: R_xlen_t, _n: R_xlen_t, _buf: *mut f64) -> Option<R_xlen_t> {
        None
    }
    fn is_sorted(_x: SEXP) -> Option<i32> { None }
    fn no_na(_x: SEXP) -> Option<i32> { None }
    fn sum(_x: SEXP, _narm: bool) -> Option<SEXP> { None }
    fn min(_x: SEXP, _narm: bool) -> Option<SEXP> { None }
    fn max(_x: SEXP, _narm: bool) -> Option<SEXP> { None }
}

/// Logical vector methods.
pub trait AltLogical: AltVec {
    fn elt(_x: SEXP, _i: R_xlen_t) -> Option<i32> { None }
    fn get_region(_x: SEXP, _i: R_xlen_t, _n: R_xlen_t, _buf: *mut i32) -> Option<R_xlen_t> {
        None
    }
    fn is_sorted(_x: SEXP) -> Option<i32> { None }
    fn no_na(_x: SEXP) -> Option<i32> { None }
    fn sum(_x: SEXP, _narm: bool) -> Option<SEXP> { None }
}

/// Raw vector methods.
pub trait AltRaw: AltVec {
    fn elt(_x: SEXP, _i: R_xlen_t) -> Option<u8> { None }
    fn get_region(_x: SEXP, _i: R_xlen_t, _n: R_xlen_t, _buf: *mut u8) -> Option<R_xlen_t> {
        None
    }
}

/// String vector methods.
pub trait AltString: AltVec {
    fn elt(_x: SEXP, _i: R_xlen_t) -> Option<SEXP> { None }
    fn set_elt(_x: SEXP, _i: R_xlen_t, _v: SEXP) -> Option<()> { None }
    fn is_sorted(_x: SEXP) -> Option<i32> { None }
    fn no_na(_x: SEXP) -> Option<i32> { None }
}

/// List (VECSXP) methods.
pub trait AltList: AltVec {
    fn elt(_x: SEXP, _i: R_xlen_t) -> Option<SEXP> { None }
    fn set_elt(_x: SEXP, _i: R_xlen_t, _v: SEXP) -> Option<()> { None }
}

