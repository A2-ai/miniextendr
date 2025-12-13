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

// =============================================================================
// AltrepBase trait - maps Rust data types to R base types
// =============================================================================

/// Trait that maps a Rust data type to its R ALTREP base type.
///
/// This trait allows the proc-macro to infer the appropriate R base type
/// from the data type, eliminating the need for explicit `base = "..."` attributes.
///
/// # Example
///
/// ```ignore
/// // Vec<i32> implements AltrepBase with BASE = RBase::Int
/// #[miniextendr(class = "MyInts", pkg = "mypkg")]
/// pub struct MyInts(Vec<i32>);  // base type inferred automatically
/// ```
pub trait AltrepBase {
    /// The R base type this data maps to.
    const BASE: RBase;
}

// Standard Vec implementations
impl AltrepBase for Vec<i32> {
    const BASE: RBase = RBase::Int;
}

impl AltrepBase for Vec<f64> {
    const BASE: RBase = RBase::Real;
}

impl AltrepBase for Vec<bool> {
    const BASE: RBase = RBase::Logical;
}

impl AltrepBase for Vec<u8> {
    const BASE: RBase = RBase::Raw;
}

impl AltrepBase for Vec<String> {
    const BASE: RBase = RBase::String;
}

impl AltrepBase for Vec<crate::ffi::Rcomplex> {
    const BASE: RBase = RBase::Complex;
}

// NOTE: Box<[T]> cannot implement AltrepBase because slices are DSTs
// (dynamically sized types) and ExternalPtr requires Sized types.
// Use Vec<T> instead for ALTREP, which is semantically equivalent.

// Range implementations
impl AltrepBase for std::ops::Range<i32> {
    const BASE: RBase = RBase::Int;
}

impl AltrepBase for std::ops::Range<i64> {
    const BASE: RBase = RBase::Int;
}

impl AltrepBase for std::ops::Range<f64> {
    const BASE: RBase = RBase::Real;
}

// Array implementations
impl<const N: usize> AltrepBase for [i32; N] {
    const BASE: RBase = RBase::Int;
}

impl<const N: usize> AltrepBase for [f64; N] {
    const BASE: RBase = RBase::Real;
}

impl<const N: usize> AltrepBase for [bool; N] {
    const BASE: RBase = RBase::Logical;
}

impl<const N: usize> AltrepBase for [u8; N] {
    const BASE: RBase = RBase::Raw;
}

impl<const N: usize> AltrepBase for [String; N] {
    const BASE: RBase = RBase::String;
}

// Static slice implementations
// `&'static [T]` is Sized (fat pointer) and satisfies 'static, so it works with ExternalPtr.
// Use cases: const arrays, leaked data, memory-mapped files with 'static lifetime.
impl AltrepBase for &'static [i32] {
    const BASE: RBase = RBase::Int;
}

impl AltrepBase for &'static [f64] {
    const BASE: RBase = RBase::Real;
}

impl AltrepBase for &'static [bool] {
    const BASE: RBase = RBase::Logical;
}

impl AltrepBase for &'static [u8] {
    const BASE: RBase = RBase::Raw;
}

impl AltrepBase for &'static [String] {
    const BASE: RBase = RBase::String;
}

impl AltrepBase for &'static [&'static str] {
    const BASE: RBase = RBase::String;
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
