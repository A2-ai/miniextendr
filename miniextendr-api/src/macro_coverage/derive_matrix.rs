#![allow(dead_code)]

//! Derive macro coverage.
//!
//! One minimal type per derive entrypoint in miniextendr-macros.
//! Some preference derives require their base trait derive as a prerequisite.
//!
//! ## Covered derives
//!
//! | Derive | Fixture |
//! |--------|---------|
//! | `ExternalPtr` | `CovPtr` |
//! | `IntoList` | `CovList` |
//! | `TryFromList` | `CovList` |
//! | `PreferList` | `CovPreferList` |
//! | `RNativeType` | `CovPreferNative` |
//! | `PreferRNativeType` | `CovPreferNative` |
//! | `DataFrameRow` | `CovRow` |
//! | `RFactor` | `CovFactor` |
//!
//! ## Covered elsewhere
//!
//! - `PreferExternalPtr`: needs a hand-written `TypedExternal` impl; the `Boxed`
//!   fixture in `miniextendr-macros/tests/ui/pass/assoc_item_variant_names.rs`
//!   covers it. `ExternalPtr` derive already supplies `IntoR`.

// ExternalPtr derive
#[derive(miniextendr_api::ExternalPtr)]
pub struct CovPtr {
    pub x: i32,
}

// IntoList + TryFromList derives
#[derive(miniextendr_api::IntoList, miniextendr_api::TryFromList)]
pub struct CovList {
    pub x: i32,
}

// PreferList requires IntoList
#[derive(miniextendr_api::IntoList, miniextendr_api::PreferList)]
pub struct CovPreferList {
    pub x: i32,
}

// RNativeType + PreferRNativeType (newtype over native R scalar)
#[repr(transparent)]
#[derive(miniextendr_api::RNativeType, miniextendr_api::PreferRNativeType, Clone, Copy)]
pub struct CovPreferNative(pub i32);

// DataFrameRow supplies the companion conversion without IntoList on the row.
#[derive(miniextendr_api::DataFrameRow)]
pub struct CovRow {
    pub x: i32,
}

// RFactor derive on C-style enum
#[derive(miniextendr_api::RFactor, Copy, Clone)]
pub enum CovFactor {
    A,
    B,
}
