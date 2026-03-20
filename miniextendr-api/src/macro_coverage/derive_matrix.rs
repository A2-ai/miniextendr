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
//! ## Excluded (marker-only, require complex prerequisites)
//!
//! - `PreferExternalPtr`: needs `IntoExternalPtr` (from `ExternalPtr`) but combined
//!   with `ExternalPtr` creates conflicting blanket `IntoR` impls.
//! - `PreferDataFrame`: needs `IntoDataFrame` which only exists on generated
//!   `*DataFrame` companion types from `DataFrameRow`.

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
#[derive(miniextendr_api::RNativeType, miniextendr_api::PreferRNativeType, Clone, Copy)]
pub struct CovPreferNative(pub i32);

// DataFrameRow requires IntoList
#[derive(miniextendr_api::IntoList, miniextendr_api::DataFrameRow)]
pub struct CovRow {
    pub x: i32,
}

// RFactor derive on C-style enum
#[derive(miniextendr_api::RFactor, Copy, Clone)]
pub enum CovFactor {
    A,
    B,
}
