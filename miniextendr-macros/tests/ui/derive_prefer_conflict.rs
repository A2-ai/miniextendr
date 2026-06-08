//! Test: stacking two `Prefer*` derives on one type is a conflict (#870).
//!
//! A type carries exactly one `IntoR` default. Each `Prefer*` derive emits an
//! inherent associated const with the same self-describing name, so a second
//! `Prefer*` derive triggers a guided "duplicate definitions" error (E0592)
//! whose identifier names both the conflict and the remedy: choose a
//! representation per return value at the call site via the `As*` wrappers.
//! The raw conflicting-`IntoR`-impl error (E0119) co-fires.

use miniextendr_macros::{IntoList, PreferList, PreferRNativeType, RNativeType};

#[derive(Clone, Copy, IntoList, RNativeType, PreferList, PreferRNativeType)]
struct Model(i32);

fn main() {}
