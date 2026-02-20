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
//! | `#[derive(PreferList)]` | [`crate::markers::PrefersList`] |
//! | `#[derive(PreferExternalPtr)]` | [`crate::markers::PrefersExternalPtr`] |
//! | `#[derive(PreferRNativeType)]` | [`crate::markers::PrefersRNativeType`] |
//! | `#[derive(PreferDataFrame)]` | [`crate::markers::PrefersDataFrame`] |

/// Marker trait for types that should be converted to R lists via `IntoR`.
///
/// Implemented by the `PreferList` derive; you can also implement it manually.
pub trait PrefersList: crate::list::IntoList {}

/// Marker trait for types that should be converted to R data frames via `IntoR`.
///
/// Implemented by the `PreferDataFrame` derive; you can also implement it manually.
pub trait PrefersDataFrame: crate::convert::IntoDataFrame {}

/// Marker trait for types that prefer `ExternalPtr` conversion.
///
/// Implemented by the `PreferExternalPtr` derive; currently informational.
pub trait PrefersExternalPtr: crate::externalptr::IntoExternalPtr {}

/// Marker trait for types that prefer native SEXP conversion.
///
/// Implemented by the `PreferRNativeType` derive; currently informational.
pub trait PrefersRNativeType: crate::ffi::RNativeType {}

// =============================================================================
// Coercion marker traits
// =============================================================================

/// Marker trait for types that can widen to `i32` without loss.
///
/// Manually implemented for specific types to avoid conflicts with identity/
/// special-case conversions. Used by blanket Coerce implementations.
pub trait WidensToI32: Into<i32> + Copy {}

/// Marker trait for types that can widen to `f64` without loss.
///
/// Manually implemented for specific types to avoid conflicts with identity/
/// special-case conversions. Used by blanket Coerce implementations.
pub trait WidensToF64: Into<f64> + Copy {}

// Explicit marker impls for widening conversions (no blanket impl to avoid conflicts)
impl WidensToI32 for i8 {}
impl WidensToI32 for i16 {}
impl WidensToI32 for u8 {}
impl WidensToI32 for u16 {}

impl WidensToF64 for f32 {}
impl WidensToF64 for i8 {}
impl WidensToF64 for i16 {}
impl WidensToF64 for i32 {}
impl WidensToF64 for u8 {}
impl WidensToF64 for u16 {}
impl WidensToF64 for u32 {}
