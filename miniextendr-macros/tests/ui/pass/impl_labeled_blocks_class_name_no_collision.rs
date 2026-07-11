//! Compile-pass regression test for #1242.
//!
//! Two *labeled* inherent `#[miniextendr]` impl blocks on the same type — the
//! documented multi-impl pattern (docs/CLASS_SYSTEMS.md "Multiple Impl
//! Blocks"; labels are what MXL009 tells the user to add). Before the fix,
//! both blocks emitted a class-name registration static named
//! `__mx_class_name_entry_<type>` (keyed on the type alone, ignoring the
//! label) and collided with `error[E0428]: the name ... is defined multiple
//! times` — labels cleared the `R_WRAPPERS_IMPL_*` and C-wrapper names but
//! not this one. The label-aware name makes the pattern compile; the
//! duplicate (identical) `MX_CLASS_NAMES` entries the two blocks register are
//! collapsed at consumption by `build_class_name_index` in miniextendr-api.

#![allow(dead_code)]

use miniextendr_api::{ExternalPtr, miniextendr};

#[derive(ExternalPtr)]
pub struct SplitImpl {
    value: i32,
}

/// Constructor-side operations.
#[miniextendr(env, label = "ctor")]
impl SplitImpl {
    fn new(initial: i32) -> Self {
        Self { value: initial }
    }
}

/// Accessor-side operations.
#[miniextendr(env, label = "ops")]
impl SplitImpl {
    fn bump(&self) -> i32 {
        self.value + 1
    }
}

fn main() {}
