//! Test fixtures for s4_helpers (S4 slot access).

use miniextendr_api::ffi::SEXP;
use miniextendr_api::prelude::*;
use miniextendr_api::s4_helpers;

/// Check if an object is S4.
#[miniextendr]
pub fn s4_is_s4(obj: SEXP) -> bool {
    unsafe { s4_helpers::s4_is(obj) }
}

/// Check if an S4 object has a named slot.
#[miniextendr]
pub fn s4_has_slot_test(obj: SEXP, slot_name: String) -> bool {
    unsafe { s4_helpers::s4_has_slot(obj, &slot_name) }
}

/// Get the class name of an S4 object.
#[miniextendr]
pub fn s4_class_name_test(obj: SEXP) -> String {
    unsafe { s4_helpers::s4_class_name(obj) }.unwrap_or_else(|| "NA".to_string())
}
