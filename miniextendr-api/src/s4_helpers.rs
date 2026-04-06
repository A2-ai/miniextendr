//! S4 slot access and class checking helpers.
//!
//! For new packages, [`S7`](https://rconsortium.github.io/S7/) is the
//! recommended class system (use `#[miniextendr(s7)]`). These S4 helpers
//! exist for interoperating with existing S4 packages — for example,
//! reading slots from Bioconductor objects passed as function arguments.
//!
//! Since R's C API for S4 slot access (`R_has_slot`, `R_do_slot`,
//! `R_do_slot_assign`) is not exposed in miniextendr's FFI bindings,
//! these helpers use R expression evaluation via [`RCall`](crate::expression::RCall)
//! as a fallback.
//!
//! All functions require being called from the R main thread and operate
/// on raw SEXP values.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::ffi::SEXP;
/// use miniextendr_api::s4_helpers;
///
/// unsafe {
///     if s4_helpers::s4_is(obj) {
///         if let Some(class) = s4_helpers::s4_class_name(obj) {
///             println!("S4 class: {class}");
///         }
///         let slot_val = s4_helpers::s4_get_slot(obj, "data")?;
///     }
/// }
/// ```
use crate::expression::{RCall, REnv};
use crate::ffi::{self, Rf_protect, Rf_unprotect, SEXP, SexpExt};
use std::ffi::CStr;

/// Get the `methods` package namespace for evaluating S4 functions.
///
/// # Safety
///
/// Must be called from the R main thread.
unsafe fn methods_namespace() -> Result<SEXP, String> {
    unsafe {
        RCall::new("getNamespace")
            .arg(scalar_string("methods"))
            .eval_base()
    }
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
/// Uses R's `slot(obj, name)` to access the slot value.
///
/// # Safety
///
/// - `obj` must be a valid S4 SEXP with the named slot.
/// - Must be called from the R main thread.
///
/// # Returns
///
/// - `Ok(SEXP)` with the slot value (unprotected).
/// - `Err(String)` if the slot doesn't exist or another R error occurs.
pub unsafe fn s4_get_slot(obj: SEXP, slot_name: &str) -> Result<SEXP, String> {
    unsafe {
        let ns = methods_namespace()?;
        let env = REnv::from_sexp(ns);
        RCall::new("slot")
            .arg(obj)
            .named_arg("name", scalar_string(slot_name))
            .eval(env.as_sexp())
    }
}

/// Set the value of a named slot on an S4 object.
///
/// Uses R's `slot(obj, name) <- value` to assign the slot value.
///
/// # Safety
///
/// - `obj` must be a valid S4 SEXP with the named slot.
/// - `value` must be a valid SEXP of the appropriate type for the slot.
/// - Must be called from the R main thread.
///
/// # Returns
///
/// - `Ok(())` on success.
/// - `Err(String)` if the slot doesn't exist or the value type is incompatible.
pub unsafe fn s4_set_slot(obj: SEXP, slot_name: &str, value: SEXP) -> Result<(), String> {
    unsafe {
        // slot(obj, name) <- value  is equivalent to `slot<-`(obj, name, value)
        let ns = methods_namespace()?;
        let env = REnv::from_sexp(ns);
        RCall::new("slot<-")
            .arg(obj)
            .named_arg("name", scalar_string(slot_name))
            .named_arg("value", value)
            .eval(env.as_sexp())?;
        Ok(())
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
        if class_attr.is_null_or_nil() || ffi::Rf_xlength(class_attr) == 0 {
            return None;
        }

        let first = class_attr.string_elt(0);
        if first.is_null_or_nil() {
            return None;
        }

        let ptr = ffi::R_CHAR(first);
        if ptr.is_null() {
            return None;
        }

        Some(CStr::from_ptr(ptr).to_string_lossy().into_owned())
    }
}

// region: Internal helpers

/// Create a scalar R character string from a Rust `&str`.
///
/// The returned SEXP is unprotected.
unsafe fn scalar_string(s: &str) -> SEXP {
    use std::ffi::CString;
    unsafe {
        let c_str = CString::new(s).expect("slot name must not contain null bytes");
        let charsxp = ffi::Rf_mkChar(c_str.as_ptr());
        Rf_protect(charsxp);
        let strsxp = SEXP::scalar_string(charsxp);
        Rf_unprotect(1);
        strsxp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn s4_is_compiles() {
        // Verify the function signature compiles.
        // Actual testing requires the R runtime.
        fn assert_fn<F: Fn(SEXP) -> bool>(_f: F) {}
        assert_fn(|s| unsafe { s4_is(s) });
    }

    #[test]
    fn s4_class_name_compiles() {
        fn assert_fn<F: Fn(SEXP) -> Option<String>>(_f: F) {}
        assert_fn(|s| unsafe { s4_class_name(s) });
    }
}
// endregion
