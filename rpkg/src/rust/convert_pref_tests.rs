//! Tests for explicit return-shaping wrappers (`AsList`, `AsExternalPtr`, `AsRNative`).
//!
//! These cover combinations where a type implements multiple conversion traits
//! (ExternalPtr, IntoList, RNativeType) to ensure the wrapper forces the
//! desired representation.

use miniextendr_api::convert::{AsExternalPtr, AsList, AsRNative};
use miniextendr_api::{miniextendr, miniextendr_module};

#[derive(
    Copy,
    Clone,
    Debug,
    miniextendr_api::ExternalPtr,
    miniextendr_api::RNativeType,
    miniextendr_api::IntoList,
)]
pub struct Hybrid(pub i32);

#[derive(Debug, miniextendr_api::ExternalPtr, miniextendr_api::IntoList)]
pub struct PtrList(pub i32);

#[derive(Copy, Clone, Debug, miniextendr_api::RNativeType, miniextendr_api::IntoList)]
pub struct NativeList(pub i32);

// All traits present
#[miniextendr]
/// @title Prefer list conversion when multiple IntoR paths exist
/// @name convert_pref_tests
/// @rdname convert_pref_tests
/// @description Wraps a type implementing ExternalPtr, IntoList, and RNativeType with `AsList` to force VECSXP.
/// @keywords internal
/// @examples
/// hybrid_as_list(1L)
pub fn hybrid_as_list(x: i32) -> AsList<Hybrid> {
    AsList(Hybrid(x))
}

#[miniextendr]
/// @title Prefer external pointer conversion when multiple IntoR paths exist
/// @rdname convert_pref_tests
/// @description Uses `AsExternalPtr` to force EXTPTRSXP even though list/native are available.
/// @keywords internal
/// @examples
/// hybrid_as_ptr(1L)
pub fn hybrid_as_ptr(x: i32) -> AsExternalPtr<Hybrid> {
    AsExternalPtr(Hybrid(x))
}

#[miniextendr]
/// @title Prefer native scalar conversion when multiple IntoR paths exist
/// @rdname convert_pref_tests
/// @description Uses `AsRNative` to force a length-1 integer vector.
/// @keywords internal
/// @examples
/// hybrid_as_native(1L)
pub fn hybrid_as_native(x: i32) -> AsRNative<Hybrid> {
    AsRNative(Hybrid(x))
}

// ExternalPtr + IntoList
#[miniextendr]
/// @title Prefer list when both ExternalPtr and IntoList exist
/// @rdname convert_pref_tests
/// @description `AsList` wins over the automatic ExternalPtr `IntoR` impl.
/// @keywords internal
/// @examples
/// ptr_list_as_list(2L)
pub fn ptr_list_as_list(x: i32) -> AsList<PtrList> {
    AsList(PtrList(x))
}

#[miniextendr]
/// @title Prefer external pointer when both ExternalPtr and IntoList exist
/// @rdname convert_pref_tests
/// @description `AsExternalPtr` wins over list conversion.
/// @keywords internal
/// @examples
/// ptr_list_as_ptr(2L)
pub fn ptr_list_as_ptr(x: i32) -> AsExternalPtr<PtrList> {
    AsExternalPtr(PtrList(x))
}

// RNativeType + IntoList
#[miniextendr]
/// @title Prefer list when both RNativeType and IntoList exist
/// @rdname convert_pref_tests
/// @description Forces VECSXP even though a native vector would be possible.
/// @keywords internal
/// @examples
/// native_list_as_list(3L)
pub fn native_list_as_list(x: i32) -> AsList<NativeList> {
    AsList(NativeList(x))
}

#[miniextendr]
/// @title Prefer native vector when both RNativeType and IntoList exist
/// @rdname convert_pref_tests
/// @description Forces an integer vector via `AsRNative`.
/// @keywords internal
/// @examples
/// native_list_as_native(3L)
pub fn native_list_as_native(x: i32) -> AsRNative<NativeList> {
    AsRNative(NativeList(x))
}

miniextendr_module! {
    mod convert_pref_tests;
    fn hybrid_as_list;
    fn hybrid_as_ptr;
    fn hybrid_as_native;
    fn ptr_list_as_list;
    fn ptr_list_as_ptr;
    fn native_list_as_list;
    fn native_list_as_native;
}
