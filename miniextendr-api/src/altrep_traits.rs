//! Safe, idiomatic ALTREP trait hierarchy mirroring R's method tables.
//!
//! ## Design: Required vs Optional Methods
//!
//! Each ALTREP type family has:
//! - **Required methods** (no defaults) - compiler enforces implementation
//! - **Optional methods** with `HAS_*` const gating - defaults to `false` (not installed)
//!
//! When `HAS_*` is false, the method is NOT installed with R, so R uses its own default behavior.
//!
//! ## Required Methods by Type
//!
//! | Type | Required Methods |
//! |------|------------------|
//! | All | `length` |
//! | ALTSTRING | `length` + `elt` |
//! | ALTLIST | `length` + `elt` |
//! | Numeric types | `length` + (`elt` OR `dataptr` via HAS_*) |

use crate::ffi::{R_xlen_t, Rcomplex, SEXP, SEXPTYPE};
use core::ffi::c_void;

// =============================================================================
// ALTREP BASE
// =============================================================================

/// Base ALTREP methods.
///
/// `length` is REQUIRED (no default). All other methods are optional with HAS_* gating.
pub trait Altrep {
    // --- REQUIRED ---
    /// Returns the length of the ALTREP vector.
    /// This is REQUIRED - R cannot determine vector length without it.
    fn length(x: SEXP) -> R_xlen_t;

    // --- OPTIONAL: Serialization ---
    const HAS_SERIALIZED_STATE: bool = false;
    /// Return serialization state.
    fn serialized_state(_x: SEXP) -> SEXP {
        unreachable!("HAS_SERIALIZED_STATE = false")
    }

    const HAS_UNSERIALIZE: bool = false;
    /// Reconstruct ALTREP from serialized state.
    fn unserialize(_class: SEXP, _state: SEXP) -> SEXP {
        unreachable!("HAS_UNSERIALIZE = false")
    }

    const HAS_UNSERIALIZE_EX: bool = false;
    /// Extended unserialization with attributes.
    fn unserialize_ex(_class: SEXP, _state: SEXP, _attr: SEXP, _objf: i32, _levs: i32) -> SEXP {
        unreachable!("HAS_UNSERIALIZE_EX = false")
    }

    // --- OPTIONAL: Duplication ---
    const HAS_DUPLICATE: bool = false;
    /// Duplicate the ALTREP object.
    fn duplicate(_x: SEXP, _deep: bool) -> SEXP {
        unreachable!("HAS_DUPLICATE = false")
    }

    const HAS_DUPLICATE_EX: bool = false;
    /// Extended duplication.
    fn duplicate_ex(_x: SEXP, _deep: bool) -> SEXP {
        unreachable!("HAS_DUPLICATE_EX = false")
    }

    // --- OPTIONAL: Coercion ---
    const HAS_COERCE: bool = false;
    /// Coerce to another type.
    fn coerce(_x: SEXP, _to_type: SEXPTYPE) -> SEXP {
        unreachable!("HAS_COERCE = false")
    }

    // --- OPTIONAL: Inspection ---
    const HAS_INSPECT: bool = false;
    /// Custom inspection for `.Internal(inspect())`.
    fn inspect(
        _x: SEXP,
        _pre: i32,
        _deep: i32,
        _pvec: i32,
        _inspect_subtree: Option<unsafe extern "C-unwind" fn(SEXP, i32, i32, i32)>,
    ) -> bool {
        unreachable!("HAS_INSPECT = false")
    }
}

// =============================================================================
// ALTVEC - Vector-level methods (extends Altrep)
// =============================================================================

/// Vector-level methods.
///
/// All methods are optional with HAS_* gating.
pub trait AltVec: Altrep {
    const HAS_DATAPTR: bool = false;
    /// Get raw data pointer.
    fn dataptr(_x: SEXP, _writable: bool) -> *mut c_void {
        unreachable!("HAS_DATAPTR = false")
    }

    const HAS_DATAPTR_OR_NULL: bool = false;
    /// Get data pointer without forcing materialization.
    fn dataptr_or_null(_x: SEXP) -> *const c_void {
        unreachable!("HAS_DATAPTR_OR_NULL = false")
    }

    const HAS_EXTRACT_SUBSET: bool = false;
    /// Optimized subsetting.
    fn extract_subset(_x: SEXP, _indx: SEXP, _call: SEXP) -> SEXP {
        unreachable!("HAS_EXTRACT_SUBSET = false")
    }
}

// =============================================================================
// ALTINTEGER - Integer vector methods
// =============================================================================

/// Integer vector methods.
///
/// For ALTINTEGER, you must provide EITHER:
/// - `HAS_ELT = true` with `elt()` implementation, OR
/// - `HAS_DATAPTR = true` with `dataptr()` implementation
///
/// If neither is provided, R will error at runtime when accessing elements.
pub trait AltInteger: AltVec {
    const HAS_ELT: bool = false;
    /// Get element at index.
    fn elt(_x: SEXP, _i: R_xlen_t) -> i32 {
        unreachable!("HAS_ELT = false")
    }

    const HAS_GET_REGION: bool = false;
    /// Bulk read elements into buffer.
    fn get_region(_x: SEXP, _i: R_xlen_t, _n: R_xlen_t, _buf: *mut i32) -> R_xlen_t {
        unreachable!("HAS_GET_REGION = false")
    }

    const HAS_IS_SORTED: bool = false;
    /// Sortedness hint.
    fn is_sorted(_x: SEXP) -> i32 {
        unreachable!("HAS_IS_SORTED = false")
    }

    const HAS_NO_NA: bool = false;
    /// NA-free hint.
    fn no_na(_x: SEXP) -> i32 {
        unreachable!("HAS_NO_NA = false")
    }

    const HAS_SUM: bool = false;
    /// Optimized sum.
    fn sum(_x: SEXP, _narm: bool) -> SEXP {
        unreachable!("HAS_SUM = false")
    }

    const HAS_MIN: bool = false;
    /// Optimized min.
    fn min(_x: SEXP, _narm: bool) -> SEXP {
        unreachable!("HAS_MIN = false")
    }

    const HAS_MAX: bool = false;
    /// Optimized max.
    fn max(_x: SEXP, _narm: bool) -> SEXP {
        unreachable!("HAS_MAX = false")
    }
}

// =============================================================================
// ALTREAL - Real (double) vector methods
// =============================================================================

/// Real vector methods.
pub trait AltReal: AltVec {
    const HAS_ELT: bool = false;
    fn elt(_x: SEXP, _i: R_xlen_t) -> f64 {
        unreachable!("HAS_ELT = false")
    }

    const HAS_GET_REGION: bool = false;
    fn get_region(_x: SEXP, _i: R_xlen_t, _n: R_xlen_t, _buf: *mut f64) -> R_xlen_t {
        unreachable!("HAS_GET_REGION = false")
    }

    const HAS_IS_SORTED: bool = false;
    fn is_sorted(_x: SEXP) -> i32 {
        unreachable!("HAS_IS_SORTED = false")
    }

    const HAS_NO_NA: bool = false;
    fn no_na(_x: SEXP) -> i32 {
        unreachable!("HAS_NO_NA = false")
    }

    const HAS_SUM: bool = false;
    fn sum(_x: SEXP, _narm: bool) -> SEXP {
        unreachable!("HAS_SUM = false")
    }

    const HAS_MIN: bool = false;
    fn min(_x: SEXP, _narm: bool) -> SEXP {
        unreachable!("HAS_MIN = false")
    }

    const HAS_MAX: bool = false;
    fn max(_x: SEXP, _narm: bool) -> SEXP {
        unreachable!("HAS_MAX = false")
    }
}

// =============================================================================
// ALTLOGICAL - Logical vector methods
// =============================================================================

/// Logical vector methods.
pub trait AltLogical: AltVec {
    const HAS_ELT: bool = false;
    /// Returns i32: 0=FALSE, 1=TRUE, NA_LOGICAL=NA
    fn elt(_x: SEXP, _i: R_xlen_t) -> i32 {
        unreachable!("HAS_ELT = false")
    }

    const HAS_GET_REGION: bool = false;
    fn get_region(_x: SEXP, _i: R_xlen_t, _n: R_xlen_t, _buf: *mut i32) -> R_xlen_t {
        unreachable!("HAS_GET_REGION = false")
    }

    const HAS_IS_SORTED: bool = false;
    fn is_sorted(_x: SEXP) -> i32 {
        unreachable!("HAS_IS_SORTED = false")
    }

    const HAS_NO_NA: bool = false;
    fn no_na(_x: SEXP) -> i32 {
        unreachable!("HAS_NO_NA = false")
    }

    const HAS_SUM: bool = false;
    /// Sum for logical = count of TRUE values.
    fn sum(_x: SEXP, _narm: bool) -> SEXP {
        unreachable!("HAS_SUM = false")
    }
    // Note: R's ALTREP API does not expose min/max for logical vectors
}

// =============================================================================
// ALTRAW - Raw (byte) vector methods
// =============================================================================

/// Raw vector methods.
pub trait AltRaw: AltVec {
    const HAS_ELT: bool = false;
    fn elt(_x: SEXP, _i: R_xlen_t) -> u8 {
        unreachable!("HAS_ELT = false")
    }

    const HAS_GET_REGION: bool = false;
    fn get_region(_x: SEXP, _i: R_xlen_t, _n: R_xlen_t, _buf: *mut u8) -> R_xlen_t {
        unreachable!("HAS_GET_REGION = false")
    }
}

// =============================================================================
// ALTCOMPLEX - Complex vector methods
// =============================================================================

/// Complex vector methods.
pub trait AltComplex: AltVec {
    const HAS_ELT: bool = false;
    fn elt(_x: SEXP, _i: R_xlen_t) -> Rcomplex {
        unreachable!("HAS_ELT = false")
    }

    const HAS_GET_REGION: bool = false;
    fn get_region(_x: SEXP, _i: R_xlen_t, _n: R_xlen_t, _buf: *mut Rcomplex) -> R_xlen_t {
        unreachable!("HAS_GET_REGION = false")
    }
}

// =============================================================================
// ALTSTRING - String vector methods
// =============================================================================

/// String vector methods.
///
/// **REQUIRED**: `elt` must be implemented (no default).
/// R will error with "No Elt method found" if you don't provide it.
pub trait AltString: AltVec {
    // --- REQUIRED for ALTSTRING ---
    /// Get string element at index. Returns CHARSXP.
    /// This is REQUIRED for ALTSTRING - there is no default.
    fn elt(x: SEXP, i: R_xlen_t) -> SEXP;

    // --- OPTIONAL ---
    const HAS_SET_ELT: bool = false;
    /// Set element (for mutable strings).
    fn set_elt(_x: SEXP, _i: R_xlen_t, _v: SEXP) {
        unreachable!("HAS_SET_ELT = false")
    }

    const HAS_IS_SORTED: bool = false;
    fn is_sorted(_x: SEXP) -> i32 {
        unreachable!("HAS_IS_SORTED = false")
    }

    const HAS_NO_NA: bool = false;
    fn no_na(_x: SEXP) -> i32 {
        unreachable!("HAS_NO_NA = false")
    }
}

// =============================================================================
// ALTLIST - List (VECSXP) methods
// =============================================================================

/// List vector methods.
///
/// **REQUIRED**: `elt` must be implemented (no default).
/// R will error with "must provide an Elt method" if you don't provide it.
pub trait AltList: AltVec {
    // --- REQUIRED for ALTLIST ---
    /// Get list element at index. Returns any SEXP.
    /// This is REQUIRED for ALTLIST - there is no default.
    fn elt(x: SEXP, i: R_xlen_t) -> SEXP;

    // --- OPTIONAL ---
    const HAS_SET_ELT: bool = false;
    /// Set element (for mutable lists).
    fn set_elt(_x: SEXP, _i: R_xlen_t, _v: SEXP) {
        unreachable!("HAS_SET_ELT = false")
    }
}

// =============================================================================
// Constants
// =============================================================================

/// Unknown sortedness value (INT_MIN in R).
pub const UNKNOWN_SORTEDNESS: i32 = i32::MIN;

/// Known to be unsorted (`KNOWN_UNSORTED` in R).
pub const KNOWN_UNSORTED: i32 = 0;

/// Sorted in increasing order, possibly with ties (`SORTED_INCR` in R).
pub const SORTED_INCR: i32 = 1;

/// Sorted in decreasing order, possibly with ties (`SORTED_DECR` in R).
pub const SORTED_DECR: i32 = -1;

/// Sorted in increasing order, with NAs first (`SORTED_INCR_NA_1ST` in R).
pub const SORTED_INCR_NA_1ST: i32 = 2;

/// Sorted in decreasing order, with NAs first (`SORTED_DECR_NA_1ST` in R).
pub const SORTED_DECR_NA_1ST: i32 = -2;
/// NA value for integers.
pub const NA_INTEGER: i32 = i32::MIN;
/// NA value for logical (same as integer in R).
pub const NA_LOGICAL: i32 = i32::MIN;
/// NA value for reals (IEEE NaN with R's NA payload).
pub const NA_REAL: f64 = f64::from_bits(0x7FF0_0000_0000_07A2);
