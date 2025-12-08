//! ALTREP registration traits and helpers.
//!
//! ## Architecture
//!
//! - **FFI**: Raw setters/types in `crate::ffi::altrep`
//! - **Traits**: Safe Core/Opt traits in `crate::altrep_traits`
//!   - Core traits: Required methods (no defaults, compiler enforces)
//!   - Opt traits: Optional methods (HAS_* gated, defaults to not installed)
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

/// Marker trait for types that can be used as ALTREP integer vectors.
///
/// Requires `AltrepCore` (for `length`) and `AltIntegerCore` marker.
/// The type must provide element access via either:
/// - `AltIntegerOpt::HAS_ELT = true` with `elt()` implementation, OR
/// - `AltVecOpt::HAS_DATAPTR = true` with `dataptr()` implementation
pub trait AltIntegerRegistrar:
    crate::altrep_traits::AltIntegerOpt + crate::altrep_traits::AltIntegerCore
{
}
impl<T: crate::altrep_traits::AltIntegerOpt + crate::altrep_traits::AltIntegerCore>
    AltIntegerRegistrar for T
{
}

/// Marker trait for types that can be used as ALTREP real vectors.
pub trait AltRealRegistrar:
    crate::altrep_traits::AltRealOpt + crate::altrep_traits::AltRealCore
{
}
impl<T: crate::altrep_traits::AltRealOpt + crate::altrep_traits::AltRealCore> AltRealRegistrar
    for T
{
}

/// Marker trait for types that can be used as ALTREP logical vectors.
pub trait AltLogicalRegistrar:
    crate::altrep_traits::AltLogicalOpt + crate::altrep_traits::AltLogicalCore
{
}
impl<T: crate::altrep_traits::AltLogicalOpt + crate::altrep_traits::AltLogicalCore>
    AltLogicalRegistrar for T
{
}

/// Marker trait for types that can be used as ALTREP raw vectors.
pub trait AltRawRegistrar:
    crate::altrep_traits::AltRawOpt + crate::altrep_traits::AltRawCore
{
}
impl<T: crate::altrep_traits::AltRawOpt + crate::altrep_traits::AltRawCore> AltRawRegistrar for T {}

/// Marker trait for types that can be used as ALTREP complex vectors.
pub trait AltComplexRegistrar:
    crate::altrep_traits::AltComplexOpt + crate::altrep_traits::AltComplexCore
{
}
impl<T: crate::altrep_traits::AltComplexOpt + crate::altrep_traits::AltComplexCore>
    AltComplexRegistrar for T
{
}

/// Marker trait for types that can be used as ALTREP string vectors.
///
/// Requires `AltStringCore` which mandates `elt()` implementation (no default).
pub trait AltStringRegistrar:
    crate::altrep_traits::AltStringOpt + crate::altrep_traits::AltStringCore
{
}
impl<T: crate::altrep_traits::AltStringOpt + crate::altrep_traits::AltStringCore>
    AltStringRegistrar for T
{
}

/// Marker trait for types that can be used as ALTREP list vectors.
///
/// Requires `AltListCore` which mandates `elt()` implementation (no default).
pub trait AltListRegistrar:
    crate::altrep_traits::AltListOpt + crate::altrep_traits::AltListCore
{
}
impl<T: crate::altrep_traits::AltListOpt + crate::altrep_traits::AltListCore> AltListRegistrar
    for T
{
}
