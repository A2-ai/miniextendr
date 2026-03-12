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

// region: ALTREP GUARD MODE

/// Controls the panic/error guard used around ALTREP trampoline callbacks.
///
/// Each mode trades off safety vs performance:
///
/// - [`Unsafe`](AltrepGuard::Unsafe): No protection. If the callback panics,
///   behavior is undefined (unwinding through C frames). Use only for trivial
///   callbacks that cannot panic.
///
/// - [`RustUnwind`](AltrepGuard::RustUnwind): Wraps in `catch_unwind`, converting
///   Rust panics to R errors. This is the **default** and safe for all pure-Rust
///   callbacks. Overhead: ~1-2ns per call.
///
/// - [`RUnwind`](AltrepGuard::RUnwind): Wraps in `R_UnwindProtect`, catching both
///   Rust panics and R `longjmp` errors. Use when ALTREP callbacks invoke R API
///   functions that might error (e.g., `Rf_allocVector`, `Rf_eval`).
///
/// The guard is selected via the `const GUARD` associated constant on the [`Altrep`]
/// trait. Since it is a const, the compiler eliminates dead branches at
/// monomorphization time — zero runtime overhead for the chosen mode.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AltrepGuard {
    /// No protection. Fastest, but if the callback panics, behavior is undefined.
    Unsafe,
    /// `catch_unwind` — catches Rust panics, converts to R errors. Default.
    RustUnwind,
    /// `with_r_unwind_protect` — catches both Rust panics and R longjmps.
    /// Use when ALTREP callbacks invoke R API functions that might error.
    RUnwind,
}
// endregion

// region: ALTREP BASE

/// Base ALTREP methods.
///
/// `length` is REQUIRED (no default). All other methods are optional with HAS_* gating.
pub trait Altrep {
    /// The guard mode for all ALTREP trampolines on this type.
    ///
    /// Defaults to [`AltrepGuard::RustUnwind`] (catches Rust panics).
    /// Override to [`AltrepGuard::Unsafe`] for maximum performance or
    /// [`AltrepGuard::RUnwind`] when callbacks call R API functions.
    const GUARD: AltrepGuard = AltrepGuard::RustUnwind;

    // --- REQUIRED ---
    /// Returns the length of the ALTREP vector.
    /// This is REQUIRED - R cannot determine vector length without it.
    fn length(x: SEXP) -> R_xlen_t;

    // --- OPTIONAL: Serialization ---
    /// Set to `true` to register [`serialized_state`](Self::serialized_state).
    const HAS_SERIALIZED_STATE: bool = false;
    /// Return serialization state.
    fn serialized_state(_x: SEXP) -> SEXP {
        unreachable!("HAS_SERIALIZED_STATE = false")
    }

    /// Set to `true` to register [`unserialize`](Self::unserialize).
    const HAS_UNSERIALIZE: bool = false;
    /// Reconstruct ALTREP from serialized state.
    fn unserialize(_class: SEXP, _state: SEXP) -> SEXP {
        unreachable!("HAS_UNSERIALIZE = false")
    }

    /// Set to `true` to register [`unserialize_ex`](Self::unserialize_ex).
    const HAS_UNSERIALIZE_EX: bool = false;
    /// Extended unserialization with attributes.
    fn unserialize_ex(_class: SEXP, _state: SEXP, _attr: SEXP, _objf: i32, _levs: i32) -> SEXP {
        unreachable!("HAS_UNSERIALIZE_EX = false")
    }

    // --- OPTIONAL: Duplication ---
    /// Set to `true` to register [`duplicate`](Self::duplicate).
    const HAS_DUPLICATE: bool = false;
    /// Duplicate the ALTREP object.
    fn duplicate(_x: SEXP, _deep: bool) -> SEXP {
        unreachable!("HAS_DUPLICATE = false")
    }

    /// Set to `true` to register [`duplicate_ex`](Self::duplicate_ex).
    const HAS_DUPLICATE_EX: bool = false;
    /// Extended duplication.
    fn duplicate_ex(_x: SEXP, _deep: bool) -> SEXP {
        unreachable!("HAS_DUPLICATE_EX = false")
    }

    // --- OPTIONAL: Coercion ---
    /// Set to `true` to register [`coerce`](Self::coerce).
    const HAS_COERCE: bool = false;
    /// Coerce to another type.
    fn coerce(_x: SEXP, _to_type: SEXPTYPE) -> SEXP {
        unreachable!("HAS_COERCE = false")
    }

    // --- OPTIONAL: Inspection ---
    /// Set to `true` to register [`inspect`](Self::inspect).
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
// endregion

// region: ALTVEC - Vector-level methods (extends Altrep)

/// Vector-level methods.
///
/// All methods are optional with HAS_* gating.
pub trait AltVec: Altrep {
    /// Set to `true` to register [`dataptr`](Self::dataptr).
    const HAS_DATAPTR: bool = false;
    /// Get raw data pointer.
    fn dataptr(_x: SEXP, _writable: bool) -> *mut c_void {
        unreachable!("HAS_DATAPTR = false")
    }

    /// Set to `true` to register [`dataptr_or_null`](Self::dataptr_or_null).
    const HAS_DATAPTR_OR_NULL: bool = false;
    /// Get data pointer without forcing materialization.
    fn dataptr_or_null(_x: SEXP) -> *const c_void {
        unreachable!("HAS_DATAPTR_OR_NULL = false")
    }

    /// Set to `true` to register [`extract_subset`](Self::extract_subset).
    const HAS_EXTRACT_SUBSET: bool = false;
    /// Optimized subsetting.
    fn extract_subset(_x: SEXP, _indx: SEXP, _call: SEXP) -> SEXP {
        unreachable!("HAS_EXTRACT_SUBSET = false")
    }
}
// endregion

// region: ALTINTEGER - Integer vector methods

/// Integer vector methods.
///
/// For ALTINTEGER, you must provide EITHER:
/// - `HAS_ELT = true` with `elt()` implementation, OR
/// - `HAS_DATAPTR = true` with `dataptr()` implementation
///
/// If neither is provided, R will error at runtime when accessing elements.
pub trait AltInteger: AltVec {
    /// Set to `true` to register [`elt`](Self::elt).
    const HAS_ELT: bool = false;
    /// Get element at index.
    fn elt(_x: SEXP, _i: R_xlen_t) -> i32 {
        unreachable!("HAS_ELT = false")
    }

    /// Set to `true` to register [`get_region`](Self::get_region).
    const HAS_GET_REGION: bool = false;
    /// Bulk read elements into buffer.
    fn get_region(_x: SEXP, _i: R_xlen_t, _n: R_xlen_t, _buf: *mut i32) -> R_xlen_t {
        unreachable!("HAS_GET_REGION = false")
    }

    /// Set to `true` to register [`is_sorted`](Self::is_sorted).
    const HAS_IS_SORTED: bool = false;
    /// Sortedness hint.
    fn is_sorted(_x: SEXP) -> i32 {
        unreachable!("HAS_IS_SORTED = false")
    }

    /// Set to `true` to register [`no_na`](Self::no_na).
    const HAS_NO_NA: bool = false;
    /// NA-free hint.
    fn no_na(_x: SEXP) -> i32 {
        unreachable!("HAS_NO_NA = false")
    }

    /// Set to `true` to register [`sum`](Self::sum).
    const HAS_SUM: bool = false;
    /// Optimized sum.
    fn sum(_x: SEXP, _narm: bool) -> SEXP {
        unreachable!("HAS_SUM = false")
    }

    /// Set to `true` to register [`min`](Self::min).
    const HAS_MIN: bool = false;
    /// Optimized min.
    fn min(_x: SEXP, _narm: bool) -> SEXP {
        unreachable!("HAS_MIN = false")
    }

    /// Set to `true` to register [`max`](Self::max).
    const HAS_MAX: bool = false;
    /// Optimized max.
    fn max(_x: SEXP, _narm: bool) -> SEXP {
        unreachable!("HAS_MAX = false")
    }
}
// endregion

// region: ALTREAL - Real (double) vector methods

/// Real vector methods.
pub trait AltReal: AltVec {
    /// Set to `true` to register [`elt`](Self::elt).
    const HAS_ELT: bool = false;
    /// Get element at index.
    fn elt(_x: SEXP, _i: R_xlen_t) -> f64 {
        unreachable!("HAS_ELT = false")
    }

    /// Set to `true` to register [`get_region`](Self::get_region).
    const HAS_GET_REGION: bool = false;
    /// Bulk read elements into buffer.
    fn get_region(_x: SEXP, _i: R_xlen_t, _n: R_xlen_t, _buf: *mut f64) -> R_xlen_t {
        unreachable!("HAS_GET_REGION = false")
    }

    /// Set to `true` to register [`is_sorted`](Self::is_sorted).
    const HAS_IS_SORTED: bool = false;
    /// Sortedness hint.
    fn is_sorted(_x: SEXP) -> i32 {
        unreachable!("HAS_IS_SORTED = false")
    }

    /// Set to `true` to register [`no_na`](Self::no_na).
    const HAS_NO_NA: bool = false;
    /// NA-free hint.
    fn no_na(_x: SEXP) -> i32 {
        unreachable!("HAS_NO_NA = false")
    }

    /// Set to `true` to register [`sum`](Self::sum).
    const HAS_SUM: bool = false;
    /// Optimized sum.
    fn sum(_x: SEXP, _narm: bool) -> SEXP {
        unreachable!("HAS_SUM = false")
    }

    /// Set to `true` to register [`min`](Self::min).
    const HAS_MIN: bool = false;
    /// Optimized min.
    fn min(_x: SEXP, _narm: bool) -> SEXP {
        unreachable!("HAS_MIN = false")
    }

    /// Set to `true` to register [`max`](Self::max).
    const HAS_MAX: bool = false;
    /// Optimized max.
    fn max(_x: SEXP, _narm: bool) -> SEXP {
        unreachable!("HAS_MAX = false")
    }
}
// endregion

// region: ALTLOGICAL - Logical vector methods

/// Logical vector methods.
pub trait AltLogical: AltVec {
    /// Set to `true` to register [`elt`](Self::elt).
    const HAS_ELT: bool = false;
    /// Returns i32: 0=FALSE, 1=TRUE, NA_LOGICAL=NA
    fn elt(_x: SEXP, _i: R_xlen_t) -> i32 {
        unreachable!("HAS_ELT = false")
    }

    /// Set to `true` to register [`get_region`](Self::get_region).
    const HAS_GET_REGION: bool = false;
    /// Bulk read elements into buffer.
    fn get_region(_x: SEXP, _i: R_xlen_t, _n: R_xlen_t, _buf: *mut i32) -> R_xlen_t {
        unreachable!("HAS_GET_REGION = false")
    }

    /// Set to `true` to register [`is_sorted`](Self::is_sorted).
    const HAS_IS_SORTED: bool = false;
    /// Sortedness hint.
    fn is_sorted(_x: SEXP) -> i32 {
        unreachable!("HAS_IS_SORTED = false")
    }

    /// Set to `true` to register [`no_na`](Self::no_na).
    const HAS_NO_NA: bool = false;
    /// NA-free hint.
    fn no_na(_x: SEXP) -> i32 {
        unreachable!("HAS_NO_NA = false")
    }

    /// Set to `true` to register [`sum`](Self::sum).
    const HAS_SUM: bool = false;
    /// Sum for logical = count of TRUE values.
    fn sum(_x: SEXP, _narm: bool) -> SEXP {
        unreachable!("HAS_SUM = false")
    }
    // Note: R's ALTREP API does not expose min/max for logical vectors
}
// endregion

// region: ALTRAW - Raw (byte) vector methods

/// Raw vector methods.
pub trait AltRaw: AltVec {
    /// Set to `true` to register [`elt`](Self::elt).
    const HAS_ELT: bool = false;
    /// Get element at index.
    fn elt(_x: SEXP, _i: R_xlen_t) -> u8 {
        unreachable!("HAS_ELT = false")
    }

    /// Set to `true` to register [`get_region`](Self::get_region).
    const HAS_GET_REGION: bool = false;
    /// Bulk read elements into buffer.
    fn get_region(_x: SEXP, _i: R_xlen_t, _n: R_xlen_t, _buf: *mut u8) -> R_xlen_t {
        unreachable!("HAS_GET_REGION = false")
    }
}
// endregion

// region: ALTCOMPLEX - Complex vector methods

/// Complex vector methods.
pub trait AltComplex: AltVec {
    /// Set to `true` to register [`elt`](Self::elt).
    const HAS_ELT: bool = false;
    /// Get element at index.
    fn elt(_x: SEXP, _i: R_xlen_t) -> Rcomplex {
        unreachable!("HAS_ELT = false")
    }

    /// Set to `true` to register [`get_region`](Self::get_region).
    const HAS_GET_REGION: bool = false;
    /// Bulk read elements into buffer.
    fn get_region(_x: SEXP, _i: R_xlen_t, _n: R_xlen_t, _buf: *mut Rcomplex) -> R_xlen_t {
        unreachable!("HAS_GET_REGION = false")
    }
}
// endregion

// region: ALTSTRING - String vector methods

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
    /// Set to `true` to register [`set_elt`](Self::set_elt).
    const HAS_SET_ELT: bool = false;
    /// Set element (for mutable strings).
    fn set_elt(_x: SEXP, _i: R_xlen_t, _v: SEXP) {
        unreachable!("HAS_SET_ELT = false")
    }

    /// Set to `true` to register [`is_sorted`](Self::is_sorted).
    const HAS_IS_SORTED: bool = false;
    /// Sortedness hint.
    fn is_sorted(_x: SEXP) -> i32 {
        unreachable!("HAS_IS_SORTED = false")
    }

    /// Set to `true` to register [`no_na`](Self::no_na).
    const HAS_NO_NA: bool = false;
    /// NA-free hint.
    fn no_na(_x: SEXP) -> i32 {
        unreachable!("HAS_NO_NA = false")
    }
}
// endregion

// region: ALTLIST - List (VECSXP) methods

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
    /// Set to `true` to register [`set_elt`](Self::set_elt).
    const HAS_SET_ELT: bool = false;
    /// Set element (for mutable lists).
    fn set_elt(_x: SEXP, _i: R_xlen_t, _v: SEXP) {
        unreachable!("HAS_SET_ELT = false")
    }
}
// endregion

// region: Constants

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
// endregion
