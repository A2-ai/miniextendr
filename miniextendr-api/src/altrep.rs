//! Core ALTREP types and registration traits.
//!
//! ## Architecture
//!
//! - **FFI**: Raw setters/types in `crate::ffi::altrep`
//! - **Traits**: Safe traits in `crate::altrep_traits` (`Altrep`, `AltVec`, `AltInteger`, etc.)
//!   - Required methods: Compiler-enforced by trait definition
//!   - Optional methods: Gated by HAS_* constants, defaults provided
//! - **Bridge**: Generic `extern "C-unwind"` trampolines in `crate::altrep_bridge`
//! - **Macro**: `#[miniextendr]` on a struct emits `impl RegisterAltrep` that:
//!   - Creates the class handle via `R_make_alt*`
//!   - Installs methods based on trait bounds and HAS_* consts

use crate::ffi::altrep::*;
use crate::ffi::{R_xlen_t, SEXP};

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
    /// The base R type (Int, Real, Logical, etc.).
    const BASE: RBase;

    /// Returns the length of the ALTREP object.
    ///
    /// # Safety
    /// Caller must ensure `x` is a valid SEXP from R.
    unsafe fn length(x: SEXP) -> R_xlen_t;
}

/// Registration trait: implemented per type by the macro on struct items.
///
/// The `get_or_init_class` method returns the ALTREP class handle, initializing
/// it on first call and returning the cached handle on subsequent calls.
///
/// This trait combines class creation and method installation into a single
/// `get_or_init_class` call that caches the result.
pub trait RegisterAltrep {
    /// Get the ALTREP class handle, initializing it if this is the first call.
    ///
    /// The implementation should:
    /// 1. Create the class handle via `R_make_alt*` (or via `InferBase::make_class`)
    /// 2. Install methods via `install_*` functions from `altrep_bridge`
    /// 3. Cache the result in a static `OnceLock`
    fn get_or_init_class() -> R_altrep_class_t;
}

// =============================================================================
// Runtime dispatch helper for class creation
// =============================================================================

/// Create an ALTREP class handle based on the runtime base type.
///
/// # Safety
/// Must be called during R initialization.
pub unsafe fn make_class_by_base(
    class_name: *const i8,
    pkg_name: *const i8,
    base: RBase,
) -> R_altrep_class_t {
    unsafe {
        match base {
            RBase::Int => R_make_altinteger_class(class_name, pkg_name, core::ptr::null_mut()),
            RBase::Real => R_make_altreal_class(class_name, pkg_name, core::ptr::null_mut()),
            RBase::Logical => R_make_altlogical_class(class_name, pkg_name, core::ptr::null_mut()),
            RBase::Raw => R_make_altraw_class(class_name, pkg_name, core::ptr::null_mut()),
            RBase::String => R_make_altstring_class(class_name, pkg_name, core::ptr::null_mut()),
            RBase::List => R_make_altlist_class(class_name, pkg_name, core::ptr::null_mut()),
            RBase::Complex => R_make_altcomplex_class(class_name, pkg_name, core::ptr::null_mut()),
        }
    }
}
