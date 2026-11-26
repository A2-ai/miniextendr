//! S4 slot access and class checking helpers.
//!
//! For new packages, [`S7`](https://rconsortium.github.io/S7/) is the
//! recommended class system (use `#[miniextendr(s7)]`). These S4 helpers
//! exist for interoperating with existing S4 packages — for example,
//! reading slots from Bioconductor objects passed as function arguments.
//!
//! Since R's C API for S4 slot access (`R_has_slot`, `R_do_slot`,
//! `R_do_slot_assign`) is not exposed in miniextendr's FFI bindings,
//! these helpers use R expression evaluation via [`RCall`]
//! as a fallback.
//!
//! All functions require being called from the R main thread and operate
//! on raw SEXP values.
//!
//! # Example
//!
//! ```ignore
//! use miniextendr_api::SEXP;
//! use miniextendr_api::s4_helpers;
//!
//! unsafe {
//!     if s4_helpers::s4_is(obj) {
//!         if let Some(class) = s4_helpers::s4_class_name(obj) {
//!             println!("S4 class: {class}");
//!         }
//!         let slot_val = s4_helpers::s4_get_slot(obj, "data")?;
//!         // A copy with the slot set; `obj` keeps its value.
//!         let relabelled = s4_helpers::s4_with_slot(obj, "label", new_label)?;
//!     }
//! }
//! ```
use crate::expression::{RCall, REnv, REvalError};
use crate::gc_protect::OwnedProtect;
use crate::unwind_protect::Boundary;
use crate::{SEXP, SexpExt};

/// Get the `methods` package namespace for evaluating S4 functions.
///
/// # Safety
///
/// Must be called from the R main thread.
unsafe fn methods_namespace() -> Result<REnv, REvalError> {
    unsafe { REnv::package_namespace("methods") }
}

/// Check if a SEXP is an S4 object.
///
/// # Safety
///
/// - `obj` must be a valid SEXP.
/// - Must be called from the R main thread.
#[inline]
pub unsafe fn s4_is(obj: SEXP) -> bool {
    obj.is_s4()
}

/// Check if an S4 object has a named slot.
///
/// Attempts to access the slot via [`s4_get_slot`]. Returns `true` if the
/// slot exists and is accessible, `false` if accessing it errors (i.e.,
/// the slot does not exist).
///
/// # Safety
///
/// - `obj` must be a valid SEXP (typically an S4 object).
/// - Must be called from the R main thread.
pub unsafe fn s4_has_slot(obj: SEXP, slot_name: &str) -> bool {
    unsafe { s4_get_slot(obj, slot_name).is_ok() }
}

/// Get the value of a named slot from an S4 object.
///
/// Uses R's `slot(obj, name)` to access the slot value. Inside a miniextendr
/// boundary it is evaluated with the caller's condition handlers, as
/// [`s4_with_slot`] is; elsewhere isolated from them.
///
/// # Safety
///
/// - `obj` must be a valid S4 SEXP with the named slot.
/// - Must be called from the R main thread.
/// - Inside a miniextendr boundary, not from a `Drop` implementation: an exit
///   a caller's handler takes unwinds through the Rust frames, as under
///   [`RCall::try_eval_with_handlers`].
///
/// # Returns
///
/// - `Ok(SEXP)` with the slot value (unprotected).
/// - `Err(REvalError)` with R's error if the slot doesn't exist or another R
///   error occurs.
pub unsafe fn s4_get_slot(obj: SEXP, slot_name: &str) -> Result<SEXP, REvalError> {
    unsafe {
        let env = methods_namespace()?;
        let call = RCall::new("slot")
            .arg(obj)
            .named_arg("name", scalar_string(slot_name));
        if Boundary::active() {
            call.try_eval_with_handlers(env.as_sexp())
        } else {
            call.eval(env.as_sexp())
        }
    }
}

/// An S4 object with a named slot set: R's `obj@name <- value`.
///
/// Assigns into a copy of `obj` (a shallow one: the other slots are shared)
/// with R's `slot<-`, and returns it; `obj` keeps its value, as R's own
/// replacement leaves a shared object unchanged. `slot<-` itself assigns into
/// the object it is given, and for the `.Data` slot returns a new object
/// rather than changing it, so the result is the only place the value lands.
///
/// A value of a class that `setIs()` relates to the slot's class is coerced
/// first, by that relation's `coerce` function: user R code.
///
/// Inside a miniextendr boundary (a `#[miniextendr]` function body, say) the
/// assignment is evaluated with the caller's condition handlers, as
/// [`RCall::try_eval_with_handlers`] evaluates: `suppressWarnings()` and
/// `withCallingHandlers()` see the coercion's warnings and messages, and an
/// exit one of them takes (`tryCatch(warning = )`) unwinds to that boundary.
/// Elsewhere (code outside a boundary, an ALTREP callback with the `unsafe`
/// guard, a finalizer) it is isolated from them, as [`RCall::eval`] is.
///
/// # Safety
///
/// - `obj` must be a valid S4 SEXP with the named slot.
/// - `value` must be a valid SEXP of the appropriate type for the slot.
/// - Must be called from the R main thread.
/// - Inside a miniextendr boundary, not from a `Drop` implementation: an exit
///   a caller's handler takes unwinds through the Rust frames, as under
///   [`RCall::try_eval_with_handlers`].
///
/// # Returns
///
/// - `Ok(SEXP)` with the object, the slot set (unprotected).
/// - `Err(REvalError)` with R's error if the slot doesn't exist or the value
///   type is incompatible.
pub unsafe fn s4_with_slot(obj: SEXP, slot_name: &str, value: SEXP) -> Result<SEXP, REvalError> {
    unsafe {
        let env = methods_namespace()?;
        let copy = OwnedProtect::new(crate::sys::Rf_shallow_duplicate(obj));
        let call = RCall::new("slot<-")
            .arg(copy.get())
            .named_arg("name", scalar_string(slot_name))
            .named_arg("value", value);
        if Boundary::active() {
            call.try_eval_with_handlers(env.as_sexp())
        } else {
            call.eval(env.as_sexp())
        }
    }
}

/// Extract the S4 class name from an object.
///
/// Reads the `class` attribute and returns the first element as a `String`.
/// Returns `None` if the object has no class attribute or the attribute is empty.
///
/// # Safety
///
/// - `obj` must be a valid SEXP.
/// - Must be called from the R main thread.
pub unsafe fn s4_class_name(obj: SEXP) -> Option<String> {
    unsafe {
        let class_attr = obj.get_class();
        if class_attr.is_null_or_nil() || class_attr.xlength() == 0 {
            return None;
        }

        let first = class_attr.string_elt(0);
        crate::from_r::charsxp_to_string_lossy(first)
    }
}

// region: Internal helpers

/// Create a scalar R character string from a Rust `&str`.
///
/// The returned SEXP is unprotected — caller must protect if further
/// allocations will occur before use.
#[inline]
fn scalar_string(s: &str) -> SEXP {
    SEXP::scalar_string_from_str(s)
}

#[cfg(test)]
mod tests;
// endregion
