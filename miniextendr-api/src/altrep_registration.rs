//! ALTREP registration traits and helpers.

use crate::ffi::altrep::R_altrep_class_t;

/// Registration trait: implemented per type by the macro on struct items.
pub trait RegisterAltrep {
    fn register() -> R_altrep_class_t;
}

/// Macro-generated types implement this to install only the methods they need.
pub trait MethodRegistrar {
    /// Safety: must be called with a valid class handle and from R-init context.
    unsafe fn install(cls: R_altrep_class_t);
}
