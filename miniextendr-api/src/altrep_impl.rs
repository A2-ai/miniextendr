//! ALTREP implementation utilities.
//!
//! This module provides helper functions for implementing ALTREP classes.
//! The proc-macro uses these to generate trait implementations.
//!
//! Use `crate::altrep_data1_as` (re-exported from externalptr) to extract
//! data from an ALTREP's data1 slot.

// =============================================================================
// Macros for generating trait implementations
// =============================================================================

/// Generate ALTREP trait implementations for a type that implements AltIntegerData.
///
/// This macro generates `impl Altrep`, `impl AltVec`, and `impl AltInteger` for the type,
/// delegating to the high-level `AltIntegerData` trait methods.
///
/// **Requires**: The type must implement `TypedExternal` (use `#[derive(ExternalPtr)]`).
#[macro_export]
macro_rules! impl_altinteger_from_data {
    ($ty:ty) => {
        impl $crate::altrep_traits::Altrep for $ty {
            fn length(x: $crate::ffi::SEXP) -> $crate::ffi::R_xlen_t {
                unsafe { $crate::altrep_data1_as::<$ty>(x) }
                    .map(|d| <$ty as $crate::altrep_data::AltrepLen>::len(&*d) as $crate::ffi::R_xlen_t)
                    .unwrap_or(0)
            }
        }

        impl $crate::altrep_traits::AltVec for $ty {}

        impl $crate::altrep_traits::AltInteger for $ty {
            const HAS_ELT: bool = true;

            fn elt(x: $crate::ffi::SEXP, i: $crate::ffi::R_xlen_t) -> i32 {
                unsafe { $crate::altrep_data1_as::<$ty>(x) }
                    .map(|d| <$ty as $crate::altrep_data::AltIntegerData>::elt(&*d, i as usize))
                    .unwrap_or(i32::MIN)
            }

            const HAS_GET_REGION: bool = true;

            fn get_region(
                x: $crate::ffi::SEXP,
                start: $crate::ffi::R_xlen_t,
                len: $crate::ffi::R_xlen_t,
                buf: *mut i32,
            ) -> $crate::ffi::R_xlen_t {
                unsafe { $crate::altrep_data1_as::<$ty>(x) }
                    .map(|d| {
                        let slice = unsafe { std::slice::from_raw_parts_mut(buf, len as usize) };
                        <$ty as $crate::altrep_data::AltIntegerData>::get_region(
                            &*d,
                            start as usize,
                            len as usize,
                            slice,
                        ) as $crate::ffi::R_xlen_t
                    })
                    .unwrap_or(0)
            }

            const HAS_IS_SORTED: bool = true;

            fn is_sorted(x: $crate::ffi::SEXP) -> i32 {
                unsafe { $crate::altrep_data1_as::<$ty>(x) }
                    .and_then(|d| <$ty as $crate::altrep_data::AltIntegerData>::is_sorted(&*d))
                    .map(|s| s.to_r_int())
                    .unwrap_or(i32::MIN)
            }

            const HAS_NO_NA: bool = true;

            fn no_na(x: $crate::ffi::SEXP) -> i32 {
                unsafe { $crate::altrep_data1_as::<$ty>(x) }
                    .and_then(|d| <$ty as $crate::altrep_data::AltIntegerData>::no_na(&*d))
                    .map(|b| if b { 1 } else { 0 })
                    .unwrap_or(0)
            }

            const HAS_SUM: bool = true;

            fn sum(x: $crate::ffi::SEXP, narm: bool) -> $crate::ffi::SEXP {
                unsafe { $crate::altrep_data1_as::<$ty>(x) }
                    .and_then(|d| <$ty as $crate::altrep_data::AltIntegerData>::sum(&*d, narm))
                    .map(|s| {
                        if s >= i32::MIN as i64 && s <= i32::MAX as i64 {
                            unsafe { $crate::ffi::Rf_ScalarInteger(s as i32) }
                        } else {
                            unsafe { $crate::ffi::Rf_ScalarReal(s as f64) }
                        }
                    })
                    .unwrap_or(unsafe { $crate::ffi::R_NilValue })
            }

            const HAS_MIN: bool = true;

            fn min(x: $crate::ffi::SEXP, narm: bool) -> $crate::ffi::SEXP {
                unsafe { $crate::altrep_data1_as::<$ty>(x) }
                    .and_then(|d| <$ty as $crate::altrep_data::AltIntegerData>::min(&*d, narm))
                    .map(|m| unsafe { $crate::ffi::Rf_ScalarInteger(m) })
                    .unwrap_or(unsafe { $crate::ffi::R_NilValue })
            }

            const HAS_MAX: bool = true;

            fn max(x: $crate::ffi::SEXP, narm: bool) -> $crate::ffi::SEXP {
                unsafe { $crate::altrep_data1_as::<$ty>(x) }
                    .and_then(|d| <$ty as $crate::altrep_data::AltIntegerData>::max(&*d, narm))
                    .map(|m| unsafe { $crate::ffi::Rf_ScalarInteger(m) })
                    .unwrap_or(unsafe { $crate::ffi::R_NilValue })
            }
        }
    };
}

/// Generate ALTREP trait implementations for a type that implements AltRealData.
#[macro_export]
macro_rules! impl_altreal_from_data {
    ($ty:ty) => {
        impl $crate::altrep_traits::Altrep for $ty {
            fn length(x: $crate::ffi::SEXP) -> $crate::ffi::R_xlen_t {
                unsafe { $crate::altrep_data1_as::<$ty>(x) }
                    .map(|d| <$ty as $crate::altrep_data::AltrepLen>::len(&*d) as $crate::ffi::R_xlen_t)
                    .unwrap_or(0)
            }
        }

        impl $crate::altrep_traits::AltVec for $ty {}

        impl $crate::altrep_traits::AltReal for $ty {
            const HAS_ELT: bool = true;

            fn elt(x: $crate::ffi::SEXP, i: $crate::ffi::R_xlen_t) -> f64 {
                unsafe { $crate::altrep_data1_as::<$ty>(x) }
                    .map(|d| <$ty as $crate::altrep_data::AltRealData>::elt(&*d, i as usize))
                    .unwrap_or(f64::NAN)
            }

            const HAS_GET_REGION: bool = true;

            fn get_region(
                x: $crate::ffi::SEXP,
                start: $crate::ffi::R_xlen_t,
                len: $crate::ffi::R_xlen_t,
                buf: *mut f64,
            ) -> $crate::ffi::R_xlen_t {
                unsafe { $crate::altrep_data1_as::<$ty>(x) }
                    .map(|d| {
                        let slice = unsafe { std::slice::from_raw_parts_mut(buf, len as usize) };
                        <$ty as $crate::altrep_data::AltRealData>::get_region(
                            &*d,
                            start as usize,
                            len as usize,
                            slice,
                        ) as $crate::ffi::R_xlen_t
                    })
                    .unwrap_or(0)
            }

            const HAS_IS_SORTED: bool = true;

            fn is_sorted(x: $crate::ffi::SEXP) -> i32 {
                unsafe { $crate::altrep_data1_as::<$ty>(x) }
                    .and_then(|d| <$ty as $crate::altrep_data::AltRealData>::is_sorted(&*d))
                    .map(|s| s.to_r_int())
                    .unwrap_or(i32::MIN)
            }

            const HAS_NO_NA: bool = true;

            fn no_na(x: $crate::ffi::SEXP) -> i32 {
                unsafe { $crate::altrep_data1_as::<$ty>(x) }
                    .and_then(|d| <$ty as $crate::altrep_data::AltRealData>::no_na(&*d))
                    .map(|b| if b { 1 } else { 0 })
                    .unwrap_or(0)
            }

            const HAS_SUM: bool = true;

            fn sum(x: $crate::ffi::SEXP, narm: bool) -> $crate::ffi::SEXP {
                unsafe { $crate::altrep_data1_as::<$ty>(x) }
                    .and_then(|d| <$ty as $crate::altrep_data::AltRealData>::sum(&*d, narm))
                    .map(|s| unsafe { $crate::ffi::Rf_ScalarReal(s) })
                    .unwrap_or(unsafe { $crate::ffi::R_NilValue })
            }

            const HAS_MIN: bool = true;

            fn min(x: $crate::ffi::SEXP, narm: bool) -> $crate::ffi::SEXP {
                unsafe { $crate::altrep_data1_as::<$ty>(x) }
                    .and_then(|d| <$ty as $crate::altrep_data::AltRealData>::min(&*d, narm))
                    .map(|m| unsafe { $crate::ffi::Rf_ScalarReal(m) })
                    .unwrap_or(unsafe { $crate::ffi::R_NilValue })
            }

            const HAS_MAX: bool = true;

            fn max(x: $crate::ffi::SEXP, narm: bool) -> $crate::ffi::SEXP {
                unsafe { $crate::altrep_data1_as::<$ty>(x) }
                    .and_then(|d| <$ty as $crate::altrep_data::AltRealData>::max(&*d, narm))
                    .map(|m| unsafe { $crate::ffi::Rf_ScalarReal(m) })
                    .unwrap_or(unsafe { $crate::ffi::R_NilValue })
            }
        }
    };
}

/// Generate ALTREP trait implementations for a type that implements AltLogicalData.
#[macro_export]
macro_rules! impl_altlogical_from_data {
    ($ty:ty) => {
        impl $crate::altrep_traits::Altrep for $ty {
            fn length(x: $crate::ffi::SEXP) -> $crate::ffi::R_xlen_t {
                unsafe { $crate::altrep_data1_as::<$ty>(x) }
                    .map(|d| <$ty as $crate::altrep_data::AltrepLen>::len(&*d) as $crate::ffi::R_xlen_t)
                    .unwrap_or(0)
            }
        }

        impl $crate::altrep_traits::AltVec for $ty {}

        impl $crate::altrep_traits::AltLogical for $ty {
            const HAS_ELT: bool = true;

            fn elt(x: $crate::ffi::SEXP, i: $crate::ffi::R_xlen_t) -> i32 {
                unsafe { $crate::altrep_data1_as::<$ty>(x) }
                    .map(|d| <$ty as $crate::altrep_data::AltLogicalData>::elt(&*d, i as usize).to_r_int())
                    .unwrap_or(i32::MIN)
            }

            const HAS_GET_REGION: bool = true;

            fn get_region(
                x: $crate::ffi::SEXP,
                start: $crate::ffi::R_xlen_t,
                len: $crate::ffi::R_xlen_t,
                buf: *mut i32,
            ) -> $crate::ffi::R_xlen_t {
                unsafe { $crate::altrep_data1_as::<$ty>(x) }
                    .map(|d| {
                        let slice = unsafe { std::slice::from_raw_parts_mut(buf, len as usize) };
                        <$ty as $crate::altrep_data::AltLogicalData>::get_region(
                            &*d,
                            start as usize,
                            len as usize,
                            slice,
                        ) as $crate::ffi::R_xlen_t
                    })
                    .unwrap_or(0)
            }

            const HAS_IS_SORTED: bool = true;

            fn is_sorted(x: $crate::ffi::SEXP) -> i32 {
                unsafe { $crate::altrep_data1_as::<$ty>(x) }
                    .and_then(|d| <$ty as $crate::altrep_data::AltLogicalData>::is_sorted(&*d))
                    .map(|s| s.to_r_int())
                    .unwrap_or(i32::MIN)
            }

            const HAS_NO_NA: bool = true;

            fn no_na(x: $crate::ffi::SEXP) -> i32 {
                unsafe { $crate::altrep_data1_as::<$ty>(x) }
                    .and_then(|d| <$ty as $crate::altrep_data::AltLogicalData>::no_na(&*d))
                    .map(|b| if b { 1 } else { 0 })
                    .unwrap_or(0)
            }

            const HAS_SUM: bool = true;

            fn sum(x: $crate::ffi::SEXP, narm: bool) -> $crate::ffi::SEXP {
                unsafe { $crate::altrep_data1_as::<$ty>(x) }
                    .and_then(|d| <$ty as $crate::altrep_data::AltLogicalData>::sum(&*d, narm))
                    .map(|s| {
                        if s >= i32::MIN as i64 && s <= i32::MAX as i64 {
                            unsafe { $crate::ffi::Rf_ScalarInteger(s as i32) }
                        } else {
                            unsafe { $crate::ffi::Rf_ScalarReal(s as f64) }
                        }
                    })
                    .unwrap_or(unsafe { $crate::ffi::R_NilValue })
            }
        }
    };
}

/// Generate ALTREP trait implementations for a type that implements AltRawData.
#[macro_export]
macro_rules! impl_altraw_from_data {
    ($ty:ty) => {
        impl $crate::altrep_traits::Altrep for $ty {
            fn length(x: $crate::ffi::SEXP) -> $crate::ffi::R_xlen_t {
                unsafe { $crate::altrep_data1_as::<$ty>(x) }
                    .map(|d| <$ty as $crate::altrep_data::AltrepLen>::len(&*d) as $crate::ffi::R_xlen_t)
                    .unwrap_or(0)
            }
        }

        impl $crate::altrep_traits::AltVec for $ty {}

        impl $crate::altrep_traits::AltRaw for $ty {
            const HAS_ELT: bool = true;

            fn elt(x: $crate::ffi::SEXP, i: $crate::ffi::R_xlen_t) -> u8 {
                unsafe { $crate::altrep_data1_as::<$ty>(x) }
                    .map(|d| <$ty as $crate::altrep_data::AltRawData>::elt(&*d, i as usize))
                    .unwrap_or(0)
            }

            const HAS_GET_REGION: bool = true;

            fn get_region(
                x: $crate::ffi::SEXP,
                start: $crate::ffi::R_xlen_t,
                len: $crate::ffi::R_xlen_t,
                buf: *mut u8,
            ) -> $crate::ffi::R_xlen_t {
                unsafe { $crate::altrep_data1_as::<$ty>(x) }
                    .map(|d| {
                        let slice = unsafe { std::slice::from_raw_parts_mut(buf, len as usize) };
                        <$ty as $crate::altrep_data::AltRawData>::get_region(
                            &*d,
                            start as usize,
                            len as usize,
                            slice,
                        ) as $crate::ffi::R_xlen_t
                    })
                    .unwrap_or(0)
            }
        }
    };
}

/// Generate ALTREP trait implementations for a type that implements AltStringData.
#[macro_export]
macro_rules! impl_altstring_from_data {
    ($ty:ty) => {
        impl $crate::altrep_traits::Altrep for $ty {
            fn length(x: $crate::ffi::SEXP) -> $crate::ffi::R_xlen_t {
                unsafe { $crate::altrep_data1_as::<$ty>(x) }
                    .map(|d| <$ty as $crate::altrep_data::AltrepLen>::len(&*d) as $crate::ffi::R_xlen_t)
                    .unwrap_or(0)
            }
        }

        impl $crate::altrep_traits::AltVec for $ty {}

        impl $crate::altrep_traits::AltString for $ty {
            fn elt(x: $crate::ffi::SEXP, i: $crate::ffi::R_xlen_t) -> $crate::ffi::SEXP {
                // Keep ExternalPtr alive while we use the string reference
                match unsafe { $crate::altrep_data1_as::<$ty>(x) } {
                    Some(d) => {
                        match <$ty as $crate::altrep_data::AltStringData>::elt(&*d, i as usize) {
                            Some(s) => unsafe {
                                $crate::ffi::Rf_mkCharLenCE(
                                    s.as_ptr().cast(),
                                    s.len() as i32,
                                    $crate::ffi::cetype_t::CE_UTF8,
                                )
                            },
                            None => unsafe { $crate::ffi::R_NaString },
                        }
                    }
                    None => unsafe { $crate::ffi::R_NaString },
                }
            }

            const HAS_IS_SORTED: bool = true;

            fn is_sorted(x: $crate::ffi::SEXP) -> i32 {
                unsafe { $crate::altrep_data1_as::<$ty>(x) }
                    .and_then(|d| <$ty as $crate::altrep_data::AltStringData>::is_sorted(&*d))
                    .map(|s| s.to_r_int())
                    .unwrap_or(i32::MIN)
            }

            const HAS_NO_NA: bool = true;

            fn no_na(x: $crate::ffi::SEXP) -> i32 {
                unsafe { $crate::altrep_data1_as::<$ty>(x) }
                    .and_then(|d| <$ty as $crate::altrep_data::AltStringData>::no_na(&*d))
                    .map(|b| if b { 1 } else { 0 })
                    .unwrap_or(0)
            }
        }
    };
}

/// Generate ALTREP trait implementations for a type that implements AltListData.
#[macro_export]
macro_rules! impl_altlist_from_data {
    ($ty:ty) => {
        impl $crate::altrep_traits::Altrep for $ty {
            fn length(x: $crate::ffi::SEXP) -> $crate::ffi::R_xlen_t {
                unsafe { $crate::altrep_data1_as::<$ty>(x) }
                    .map(|d| <$ty as $crate::altrep_data::AltrepLen>::len(&*d) as $crate::ffi::R_xlen_t)
                    .unwrap_or(0)
            }
        }

        impl $crate::altrep_traits::AltVec for $ty {}

        impl $crate::altrep_traits::AltList for $ty {
            fn elt(x: $crate::ffi::SEXP, i: $crate::ffi::R_xlen_t) -> $crate::ffi::SEXP {
                unsafe { $crate::altrep_data1_as::<$ty>(x) }
                    .map(|d| <$ty as $crate::altrep_data::AltListData>::elt(&*d, i as usize))
                    .unwrap_or(unsafe { $crate::ffi::R_NilValue })
            }
        }
    };
}

/// Generate ALTREP trait implementations for a type that implements AltComplexData.
#[macro_export]
macro_rules! impl_altcomplex_from_data {
    ($ty:ty) => {
        impl $crate::altrep_traits::Altrep for $ty {
            fn length(x: $crate::ffi::SEXP) -> $crate::ffi::R_xlen_t {
                unsafe { $crate::altrep_data1_as::<$ty>(x) }
                    .map(|d| <$ty as $crate::altrep_data::AltrepLen>::len(&*d) as $crate::ffi::R_xlen_t)
                    .unwrap_or(0)
            }
        }

        impl $crate::altrep_traits::AltVec for $ty {}

        impl $crate::altrep_traits::AltComplex for $ty {
            const HAS_ELT: bool = true;

            fn elt(x: $crate::ffi::SEXP, i: $crate::ffi::R_xlen_t) -> $crate::ffi::Rcomplex {
                unsafe { $crate::altrep_data1_as::<$ty>(x) }
                    .map(|d| <$ty as $crate::altrep_data::AltComplexData>::elt(&*d, i as usize))
                    .unwrap_or($crate::ffi::Rcomplex { r: f64::NAN, i: f64::NAN })
            }

            const HAS_GET_REGION: bool = true;

            fn get_region(
                x: $crate::ffi::SEXP,
                start: $crate::ffi::R_xlen_t,
                len: $crate::ffi::R_xlen_t,
                buf: *mut $crate::ffi::Rcomplex,
            ) -> $crate::ffi::R_xlen_t {
                unsafe { $crate::altrep_data1_as::<$ty>(x) }
                    .map(|d| {
                        let slice = unsafe { std::slice::from_raw_parts_mut(buf, len as usize) };
                        <$ty as $crate::altrep_data::AltComplexData>::get_region(
                            &*d,
                            start as usize,
                            len as usize,
                            slice,
                        ) as $crate::ffi::R_xlen_t
                    })
                    .unwrap_or(0)
            }
        }
    };
}

// =============================================================================
// Built-in implementations for standard types
// =============================================================================
// These implementations are provided here to satisfy the orphan rules.
// User crates can use these types directly with delegate_data.

// Integer types
impl_altinteger_from_data!(Vec<i32>);
impl_altinteger_from_data!(std::ops::Range<i32>);
impl_altinteger_from_data!(std::ops::Range<i64>);

// Real types
impl_altreal_from_data!(Vec<f64>);
impl_altreal_from_data!(std::ops::Range<f64>);

// Logical types
impl_altlogical_from_data!(Vec<bool>);

// Raw types
impl_altraw_from_data!(Vec<u8>);

// String types
impl_altstring_from_data!(Vec<String>);
