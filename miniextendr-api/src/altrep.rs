//! Core ALTREP types for the proc-macro approach.
//!
//! This module provides minimal types needed by `#[miniextendr]` for ALTREP classes.
//! Individual ALTREP classes are created via the `#[miniextendr]` proc-macro.

use crate::ffi::{R_xlen_t, SEXP};

/// Initialize ALTREP subsystem.
///
/// Called automatically during package initialization.
/// This is a no-op since ALTREP classes are registered lazily
/// via the `#[miniextendr]` proc-macro approach.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn miniextendr_altrep_init() {
    // No-op: ALTREP classes are registered lazily via proc-macro generated code
}

/// Base type for ALTREP vectors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RBase {
    Int,
    Real,
    Logical,
    Raw,
    String,
    List,
    Complex,
}

/// Trait implemented by ALTREP classes via `#[miniextendr]`.
///
/// This trait is automatically implemented when using the proc-macro with
/// ALTREP attributes (class, pkg, base).
pub trait AltrepClass {
    /// The class name (null-terminated C string).
    const CLASS_NAME: &'static std::ffi::CStr;
    /// The package name (null-terminated C string).
    const PKG_NAME: &'static std::ffi::CStr;
    /// The base R type (Int, Real, Logical, etc.).
    const BASE: RBase;

    /// Returns the length of the ALTREP object.
    ///
    /// # Safety
    /// Caller must ensure `x` is a valid SEXP from R.
    unsafe fn length(x: SEXP) -> R_xlen_t;
}
