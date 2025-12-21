//! ALTREP registration traits and helpers.
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
//! - **Init**: `miniextendr_altrep_init()` called from `entrypoint.c` during package init

use crate::ffi::altrep::R_altrep_class_t;

/// Registration trait: implemented per type by the macro on struct items.
///
/// The `get_or_init_class` method returns the ALTREP class handle, initializing
/// it on first call and returning the cached handle on subsequent calls.
pub trait RegisterAltrep {
    /// Get the ALTREP class handle, initializing it if this is the first call.
    fn get_or_init_class() -> R_altrep_class_t;
}

/// Macro-generated types implement this to install only the methods they need.
///
/// The installer checks trait bounds and HAS_* consts to determine which
/// methods to wire up to R.
pub trait MethodRegistrar {
    /// Install ALTREP methods into the class.
    ///
    /// # Safety
    /// Must be invoked during R initialization with a valid ALTREP class handle.
    /// Callbacks are registered into the class and must match its base kind.
    unsafe fn install(cls: R_altrep_class_t);
}

// =============================================================================
// Runtime dispatch helper for class creation
// =============================================================================

use crate::altrep::RBase;
use crate::ffi::altrep::*;

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
