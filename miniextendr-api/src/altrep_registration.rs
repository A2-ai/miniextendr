//! ALTREP registration traits and helpers.
//!
//! Layout overview
//! - FFI: raw setters/types live in `crate::ffi::altrep`.
//! - Traits: safe, opt‑in method surfaces in `crate::altrep_traits` (with HAS_* flags).
//! - Bridge: generic extern "C-unwind" trampolines in `crate::altrep_bridge` call trait methods.
//! - Macro: `#[miniextendr]` on a struct emits an `impl RegisterAltrep` that:
//!   - Creates the class handle via `R_make_alt*`.
//!   - Installs only methods whose `<T as Trait>::HAS_*` are true by wiring
//!     `R_set_alt*` to the bridge trampolines specialized on a chosen trampoline type
//!     (either the struct itself or a `delegate = Type`).
//! - Init: `miniextendr_altrep_init()` performs built‑in class registration and is
//!   called from `entrypoint.c(.in)` during package initialization.

use crate::ffi::altrep::R_altrep_class_t;

/// Registration trait: implemented per type by the macro on struct items.
pub trait RegisterAltrep {
    fn register() -> R_altrep_class_t;
}

/// Macro-generated types implement this to install only the methods they need.
pub trait MethodRegistrar {
    /// # Safety
    /// Must be invoked during R initialization with a valid ALTREP class handle.
    /// Callbacks are registered into the class and must match its base kind.
    unsafe fn install(cls: R_altrep_class_t);
}
