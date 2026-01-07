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
//! | `#[derive(RNativeType)]` | [`crate::markers::IsRNativeType`] |
//! | `#[derive(AltrepInteger)]` | [`crate::markers::IsAltrepIntegerData`] |
//! | `#[derive(AltrepReal)]` | [`crate::markers::IsAltrepRealData`] |
//! | `#[derive(AltrepLogical)]` | [`crate::markers::IsAltrepLogicalData`] |
//! | `#[derive(AltrepRaw)]` | [`crate::markers::IsAltrepRawData`] |
//! | `#[derive(AltrepString)]` | [`crate::markers::IsAltrepStringData`] |
//! | `#[derive(AltrepComplex)]` | [`crate::markers::IsAltrepComplexData`] |
//! | `#[derive(AltrepList)]` | [`crate::markers::IsAltrepListData`] |
//! | `#[derive(IntoList)]` | [`crate::markers::IsIntoList`] |
//! | `#[derive(PreferList)]` | [`crate::markers::PrefersList`] |
//! | `#[derive(PreferExternalPtr)]` | [`crate::markers::PrefersExternalPtr`] |
//! | `#[derive(PreferRNativeType)]` | [`crate::markers::PrefersRNativeType`] |
//! | `#[derive(PreferList)]` | [`crate::markers::PrefersList`] |

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

/// Marker trait for types derived with `#[derive(IntoList)]`.
///
/// Indicates that a struct derives list conversion helpers.
pub trait IsIntoList: crate::list::IntoList {}

/// Marker trait for types that should be converted to R lists via `IntoR`.
///
/// Implemented by the `PreferList` derive; you can also implement it manually.
pub trait PrefersList: IsIntoList {}

/// Marker trait for types that prefer `ExternalPtr` conversion.
///
/// Implemented by the `PreferExternalPtr` derive; currently informational.
pub trait PrefersExternalPtr: crate::externalptr::IntoExternalPtr {}

/// Marker trait for types that prefer native SEXP conversion.
///
/// Implemented by the `PreferRNativeType` derive; currently informational.
pub trait PrefersRNativeType: IsRNativeType {}

// Blanket implementations: any type satisfying the underlying data trait
// automatically gets the marker trait. This keeps derived and manual impls
// consistent without requiring an explicit marker impl.
impl<T: crate::ffi::RNativeType> IsRNativeType for T {}
impl<T: crate::altrep_data::AltIntegerData> IsAltrepIntegerData for T {}
impl<T: crate::altrep_data::AltRealData> IsAltrepRealData for T {}
impl<T: crate::altrep_data::AltLogicalData> IsAltrepLogicalData for T {}
impl<T: crate::altrep_data::AltRawData> IsAltrepRawData for T {}
impl<T: crate::altrep_data::AltStringData> IsAltrepStringData for T {}
impl<T: crate::altrep_data::AltComplexData> IsAltrepComplexData for T {}
impl<T: crate::altrep_data::AltListData> IsAltrepListData for T {}
impl<T: crate::list::IntoList> IsIntoList for T {}
