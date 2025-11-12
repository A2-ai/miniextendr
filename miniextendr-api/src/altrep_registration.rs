//! ALTREP registration traits and helpers.

use crate::ffi::altrep::*;

/// Registration trait: implement for a type to produce an ALTREP class handle.
pub trait RegisterAltrep {
    fn register() -> R_altrep_class_t;
}

