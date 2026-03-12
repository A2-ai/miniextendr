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
use std::ffi::CStr;

/// Base type for ALTREP vectors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RBase {
    /// Integer vectors (`INTSXP`).
    Int,
    /// Double vectors (`REALSXP`).
    Real,
    /// Logical vectors (`LGLSXP`).
    Logical,
    /// Raw byte vectors (`RAWSXP`).
    Raw,
    /// Character vectors (`STRSXP`).
    String,
    /// Generic list vectors (`VECSXP`).
    List,
    /// Complex vectors (`CPLXSXP`).
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

// region: Runtime dispatch helper for class creation

/// Validate that an ALTREP class handle was successfully created.
///
/// Panics with a descriptive message if the class handle is null, indicating
/// that `R_make_alt*_class()` failed during registration.
///
/// # Arguments
/// * `cls` - The class handle returned by `R_make_alt*_class()`
/// * `class_name` - The name of the ALTREP class (for diagnostics)
/// * `base` - The base R type (for diagnostics)
pub fn validate_altrep_class(
    cls: R_altrep_class_t,
    class_name: &CStr,
    base: RBase,
) -> R_altrep_class_t {
    if cls.ptr.is_null() {
        panic!(
            "ALTREP class registration failed: R_make_alt{base:?}_class() returned NULL \
             for class {:?}",
            class_name
        );
    }
    cls
}

/// Create an ALTREP class handle based on the runtime base type.
///
/// Validates the returned handle and panics if registration fails.
///
/// # Safety
/// Must be called during R initialization.
pub unsafe fn make_class_by_base(
    class_name: *const i8,
    pkg_name: *const i8,
    base: RBase,
) -> R_altrep_class_t {
    let cls = unsafe {
        match base {
            RBase::Int => R_make_altinteger_class(class_name, pkg_name, core::ptr::null_mut()),
            RBase::Real => R_make_altreal_class(class_name, pkg_name, core::ptr::null_mut()),
            RBase::Logical => R_make_altlogical_class(class_name, pkg_name, core::ptr::null_mut()),
            RBase::Raw => R_make_altraw_class(class_name, pkg_name, core::ptr::null_mut()),
            RBase::String => R_make_altstring_class(class_name, pkg_name, core::ptr::null_mut()),
            RBase::List => R_make_altlist_class(class_name, pkg_name, core::ptr::null_mut()),
            RBase::Complex => R_make_altcomplex_class(class_name, pkg_name, core::ptr::null_mut()),
        }
    };
    // SAFETY: class_name was passed to R, so it's still a valid C string
    let name_cstr = unsafe { CStr::from_ptr(class_name) };
    validate_altrep_class(cls, name_cstr, base)
}
// endregion
