//! Marker traits for proc-macro derived types.
//!
//! These marker traits identify types that have been derived with specific proc-macros.
//! They enable compile-time type checking and blanket implementations.
//!
//! # Pattern
//!
//! Each derive macro generates an impl of its corresponding marker trait:
//!
//! | Derive Macro | Marker Trait |
//! |--------------|--------------|
//! | `#[derive(ExternalPtr)]` | [`IntoExternalPtr`](crate::externalptr::IntoExternalPtr) |
//! | `#[derive(RNativeType)]` | [`IsRNativeType`] |
//! | `#[derive(AltrepInteger)]` | [`IsAltrepIntegerData`] |
//! | `#[derive(AltrepReal)]` | [`IsAltrepRealData`] |
//! | `#[derive(AltrepLogical)]` | [`IsAltrepLogicalData`] |
//! | `#[derive(AltrepRaw)]` | [`IsAltrepRawData`] |
//! | `#[derive(AltrepString)]` | [`IsAltrepStringData`] |
//! | `#[derive(AltrepComplex)]` | [`IsAltrepComplexData`] |
//! | `#[derive(AltrepList)]` | [`IsAltrepListData`] |

/// Marker trait for types derived with `#[derive(RNativeType)]`.
///
/// This marker indicates that a newtype wrapper implements [`RNativeType`](crate::ffi::RNativeType)
/// via the derive macro.
///
/// # Example
///
/// ```ignore
/// #[derive(Clone, Copy, RNativeType)]
/// struct UserId(i32);
///
/// // The derive macro generates:
/// // impl IsRNativeType for UserId {}
/// ```
pub trait IsRNativeType: crate::ffi::RNativeType {}

/// Marker trait for types derived with `#[derive(AltrepInteger)]`.
///
/// This marker indicates that a type implements [`AltIntegerData`](crate::altrep_data::AltIntegerData)
/// via the derive macro.
pub trait IsAltrepIntegerData: crate::altrep_data::AltIntegerData {}

/// Marker trait for types derived with `#[derive(AltrepReal)]`.
///
/// This marker indicates that a type implements [`AltRealData`](crate::altrep_data::AltRealData)
/// via the derive macro.
pub trait IsAltrepRealData: crate::altrep_data::AltRealData {}

/// Marker trait for types derived with `#[derive(AltrepLogical)]`.
///
/// This marker indicates that a type implements [`AltLogicalData`](crate::altrep_data::AltLogicalData)
/// via the derive macro.
pub trait IsAltrepLogicalData: crate::altrep_data::AltLogicalData {}

/// Marker trait for types derived with `#[derive(AltrepRaw)]`.
///
/// This marker indicates that a type implements [`AltRawData`](crate::altrep_data::AltRawData)
/// via the derive macro.
pub trait IsAltrepRawData: crate::altrep_data::AltRawData {}

/// Marker trait for types derived with `#[derive(AltrepString)]`.
///
/// This marker indicates that a type implements [`AltStringData`](crate::altrep_data::AltStringData)
/// via the derive macro.
pub trait IsAltrepStringData: crate::altrep_data::AltStringData {}

/// Marker trait for types derived with `#[derive(AltrepComplex)]`.
///
/// This marker indicates that a type implements [`AltComplexData`](crate::altrep_data::AltComplexData)
/// via the derive macro.
pub trait IsAltrepComplexData: crate::altrep_data::AltComplexData {}

/// Marker trait for types derived with `#[derive(AltrepList)]`.
///
/// This marker indicates that a type implements [`AltListData`](crate::altrep_data::AltListData)
/// via the derive macro.
pub trait IsAltrepListData: crate::altrep_data::AltListData {}
