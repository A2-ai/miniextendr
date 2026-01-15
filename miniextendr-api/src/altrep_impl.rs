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
///
/// ## Variants
///
/// ```ignore
/// // Basic (no dataptr, no serialization):
/// impl_altinteger_from_data!(MyType);
///
/// // With dataptr (type must implement AltrepDataptr<i32>):
/// impl_altinteger_from_data!(MyType, dataptr);
///
/// // With serialization (type must implement AltrepSerialize):
/// impl_altinteger_from_data!(MyType, serialize);
///
/// // With subset optimization (type must implement AltrepExtractSubset):
/// impl_altinteger_from_data!(MyType, subset);
///
/// // Combine multiple options:
/// impl_altinteger_from_data!(MyType, dataptr, serialize);
/// impl_altinteger_from_data!(MyType, subset, serialize);
/// ```
#[macro_export]
macro_rules! impl_altinteger_from_data {
    ($ty:ty) => {
        $crate::__impl_altrep_base!($ty);
        impl $crate::altrep_traits::AltVec for $ty {}
        $crate::__impl_altinteger_methods!($ty);
        $crate::impl_inferbase_integer!($ty);
    };
    ($ty:ty, dataptr) => {
        $crate::__impl_altrep_base!($ty);
        $crate::__impl_altvec_dataptr!($ty, i32);
        $crate::__impl_altinteger_methods!($ty);
        $crate::impl_inferbase_integer!($ty);
    };
    ($ty:ty, serialize) => {
        $crate::__impl_altrep_base_with_serialize!($ty);
        impl $crate::altrep_traits::AltVec for $ty {}
        $crate::__impl_altinteger_methods!($ty);
        $crate::impl_inferbase_integer!($ty);
    };
    ($ty:ty, subset) => {
        $crate::__impl_altrep_base!($ty);
        $crate::__impl_altvec_extract_subset!($ty);
        $crate::__impl_altinteger_methods!($ty);
        $crate::impl_inferbase_integer!($ty);
    };
    ($ty:ty, dataptr, serialize) => {
        $crate::__impl_altrep_base_with_serialize!($ty);
        $crate::__impl_altvec_dataptr!($ty, i32);
        $crate::__impl_altinteger_methods!($ty);
        $crate::impl_inferbase_integer!($ty);
    };
    ($ty:ty, serialize, dataptr) => {
        $crate::impl_altinteger_from_data!($ty, dataptr, serialize);
    };
    ($ty:ty, subset, serialize) => {
        $crate::__impl_altrep_base_with_serialize!($ty);
        $crate::__impl_altvec_extract_subset!($ty);
        $crate::__impl_altinteger_methods!($ty);
        $crate::impl_inferbase_integer!($ty);
    };
    ($ty:ty, serialize, subset) => {
        $crate::impl_altinteger_from_data!($ty, subset, serialize);
    };
}

/// Internal macro: impl Altrep with just length
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_altrep_base {
    ($ty:ty) => {
        #[allow(clippy::not_unsafe_ptr_arg_deref)]
        impl $crate::altrep_traits::Altrep for $ty {
            fn length(x: $crate::ffi::SEXP) -> $crate::ffi::R_xlen_t {
                unsafe { $crate::altrep_data1_as::<$ty>(x) }
                    .map(|d| {
                        <$ty as $crate::altrep_data::AltrepLen>::len(&*d) as $crate::ffi::R_xlen_t
                    })
                    .unwrap_or(0)
            }
        }
    };
}

/// Internal macro: impl Altrep with length + serialization
///
/// This implements both:
/// - `serialized_state(x)` (save-side)
/// - `unserialize(class, state)` (load-side)
///
/// The `unserialize` implementation reconstructs the backing Rust value via
/// [`AltrepSerialize::unserialize`] and then creates a fresh ALTREP instance via
/// `R_new_altrep(class, data1, R_NilValue)` where `data1` is an `ExternalPtr<$ty>`.
///
/// This matches the proc-macro-generated `IntoR::into_sexp` behavior (data is stored in `data1`,
/// and `data2` is `R_NilValue`).
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_altrep_base_with_serialize {
    ($ty:ty) => {
        #[allow(clippy::not_unsafe_ptr_arg_deref)]
        impl $crate::altrep_traits::Altrep for $ty {
            fn length(x: $crate::ffi::SEXP) -> $crate::ffi::R_xlen_t {
                unsafe { $crate::altrep_data1_as::<$ty>(x) }
                    .map(|d| {
                        <$ty as $crate::altrep_data::AltrepLen>::len(&*d) as $crate::ffi::R_xlen_t
                    })
                    .unwrap_or(0)
            }

            const HAS_SERIALIZED_STATE: bool = true;

            fn serialized_state(x: $crate::ffi::SEXP) -> $crate::ffi::SEXP {
                unsafe { $crate::altrep_data1_as::<$ty>(x) }
                    .map(|d| <$ty as $crate::altrep_data::AltrepSerialize>::serialized_state(&*d))
                    .unwrap_or($crate::ffi::SEXP::null())
            }

            const HAS_UNSERIALIZE: bool = true;

            fn unserialize(
                class: $crate::ffi::SEXP,
                state: $crate::ffi::SEXP,
            ) -> $crate::ffi::SEXP {
                let Some(data) = <$ty as $crate::altrep_data::AltrepSerialize>::unserialize(state)
                else {
                    $crate::r_error!(
                        "ALTREP unserialize failed for {}",
                        core::any::type_name::<$ty>()
                    );
                };

                // SAFETY: Unserialize is called by R on the main thread.
                unsafe {
                    use $crate::externalptr::ExternalPtr;
                    use $crate::ffi::altrep::{R_altrep_class_t, R_new_altrep};
                    use $crate::ffi::{R_NilValue, Rf_protect_unchecked, Rf_unprotect_unchecked};

                    let ext_ptr = ExternalPtr::new_unchecked(data);
                    let data1 = ext_ptr.as_sexp();
                    // Protect across the allocation in R_new_altrep.
                    Rf_protect_unchecked(data1);
                    let out = R_new_altrep(R_altrep_class_t { ptr: class }, data1, R_NilValue);
                    Rf_unprotect_unchecked(1);
                    out
                }
            }
        }
    };
}

/// Internal macro: impl AltVec with dataptr support
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_altvec_dataptr {
    ($ty:ty, $elem:ty) => {
        #[allow(clippy::not_unsafe_ptr_arg_deref)]
        impl $crate::altrep_traits::AltVec for $ty {
            const HAS_DATAPTR: bool = true;

            fn dataptr(x: $crate::ffi::SEXP, writable: bool) -> *mut core::ffi::c_void {
                unsafe { $crate::altrep_data1_mut::<$ty>(x) }
                    .and_then(|d| {
                        <$ty as $crate::altrep_data::AltrepDataptr<$elem>>::dataptr(d, writable)
                    })
                    .map(|p| p as *mut core::ffi::c_void)
                    .unwrap_or(core::ptr::null_mut())
            }

            const HAS_DATAPTR_OR_NULL: bool = true;

            fn dataptr_or_null(x: $crate::ffi::SEXP) -> *const core::ffi::c_void {
                unsafe { $crate::altrep_data1_as::<$ty>(x) }
                    .and_then(|d| {
                        <$ty as $crate::altrep_data::AltrepDataptr<$elem>>::dataptr_or_null(&*d)
                    })
                    .map(|p| p as *const core::ffi::c_void)
                    .unwrap_or(core::ptr::null())
            }
        }
    };
}

/// Internal macro: impl AltVec with extract_subset support
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_altvec_extract_subset {
    ($ty:ty) => {
        #[allow(clippy::not_unsafe_ptr_arg_deref)]
        impl $crate::altrep_traits::AltVec for $ty {
            const HAS_EXTRACT_SUBSET: bool = true;

            fn extract_subset(
                x: $crate::ffi::SEXP,
                indx: $crate::ffi::SEXP,
                _call: $crate::ffi::SEXP,
            ) -> $crate::ffi::SEXP {
                // Validate that indx is an integer vector before calling INTEGER().
                // Return NULL to signal R to use default subsetting if not.
                if unsafe { $crate::ffi::TYPEOF(indx) } != $crate::ffi::SEXPTYPE::INTSXP {
                    return core::ptr::null_mut();
                }

                // Convert indx SEXP to slice
                let len = unsafe { $crate::ffi::Rf_xlength(indx) } as usize;
                let indices =
                    unsafe { std::slice::from_raw_parts($crate::ffi::INTEGER(indx), len) };

                unsafe { $crate::altrep_data1_as::<$ty>(x) }
                    .and_then(|d| {
                        <$ty as $crate::altrep_data::AltrepExtractSubset>::extract_subset(
                            &*d, indices,
                        )
                    })
                    .unwrap_or($crate::ffi::SEXP::null())
            }
        }
    };
}

/// Internal macro for AltInteger method implementations.
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_altinteger_methods {
    ($ty:ty) => {
        #[allow(clippy::not_unsafe_ptr_arg_deref)]
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
                    .unwrap_or($crate::ffi::SEXP::null())
            }

            const HAS_MIN: bool = true;

            fn min(x: $crate::ffi::SEXP, narm: bool) -> $crate::ffi::SEXP {
                unsafe { $crate::altrep_data1_as::<$ty>(x) }
                    .and_then(|d| <$ty as $crate::altrep_data::AltIntegerData>::min(&*d, narm))
                    .map(|m| unsafe { $crate::ffi::Rf_ScalarInteger(m) })
                    .unwrap_or($crate::ffi::SEXP::null())
            }

            const HAS_MAX: bool = true;

            fn max(x: $crate::ffi::SEXP, narm: bool) -> $crate::ffi::SEXP {
                unsafe { $crate::altrep_data1_as::<$ty>(x) }
                    .and_then(|d| <$ty as $crate::altrep_data::AltIntegerData>::max(&*d, narm))
                    .map(|m| unsafe { $crate::ffi::Rf_ScalarInteger(m) })
                    .unwrap_or($crate::ffi::SEXP::null())
            }
        }
    };
}

/// Generate ALTREP trait implementations for a type that implements AltRealData.
///
/// ## Variants
///
/// ```ignore
/// // Basic (no dataptr, no serialization):
/// impl_altreal_from_data!(MyType);
///
/// // With dataptr (type must implement AltrepDataptr<f64>):
/// impl_altreal_from_data!(MyType, dataptr);
///
/// // With serialization (type must implement AltrepSerialize):
/// impl_altreal_from_data!(MyType, serialize);
///
/// // With both dataptr and serialization:
/// impl_altreal_from_data!(MyType, dataptr, serialize);
/// ```
#[macro_export]
macro_rules! impl_altreal_from_data {
    ($ty:ty) => {
        $crate::__impl_altrep_base!($ty);
        impl $crate::altrep_traits::AltVec for $ty {}
        $crate::__impl_altreal_methods!($ty);
        $crate::impl_inferbase_real!($ty);
    };
    ($ty:ty, dataptr) => {
        $crate::__impl_altrep_base!($ty);
        $crate::__impl_altvec_dataptr!($ty, f64);
        $crate::__impl_altreal_methods!($ty);
        $crate::impl_inferbase_real!($ty);
    };
    ($ty:ty, serialize) => {
        $crate::__impl_altrep_base_with_serialize!($ty);
        impl $crate::altrep_traits::AltVec for $ty {}
        $crate::__impl_altreal_methods!($ty);
        $crate::impl_inferbase_real!($ty);
    };
    ($ty:ty, dataptr, serialize) => {
        $crate::__impl_altrep_base_with_serialize!($ty);
        $crate::__impl_altvec_dataptr!($ty, f64);
        $crate::__impl_altreal_methods!($ty);
        $crate::impl_inferbase_real!($ty);
    };
    ($ty:ty, serialize, dataptr) => {
        $crate::impl_altreal_from_data!($ty, dataptr, serialize);
    };
}

/// Internal macro for AltReal method implementations.
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_altreal_methods {
    ($ty:ty) => {
        #[allow(clippy::not_unsafe_ptr_arg_deref)]
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
                    .unwrap_or($crate::ffi::SEXP::null())
            }

            const HAS_MIN: bool = true;

            fn min(x: $crate::ffi::SEXP, narm: bool) -> $crate::ffi::SEXP {
                unsafe { $crate::altrep_data1_as::<$ty>(x) }
                    .and_then(|d| <$ty as $crate::altrep_data::AltRealData>::min(&*d, narm))
                    .map(|m| unsafe { $crate::ffi::Rf_ScalarReal(m) })
                    .unwrap_or($crate::ffi::SEXP::null())
            }

            const HAS_MAX: bool = true;

            fn max(x: $crate::ffi::SEXP, narm: bool) -> $crate::ffi::SEXP {
                unsafe { $crate::altrep_data1_as::<$ty>(x) }
                    .and_then(|d| <$ty as $crate::altrep_data::AltRealData>::max(&*d, narm))
                    .map(|m| unsafe { $crate::ffi::Rf_ScalarReal(m) })
                    .unwrap_or($crate::ffi::SEXP::null())
            }
        }
    };
}

/// Generate ALTREP trait implementations for a type that implements AltLogicalData.
///
/// ## Variants
///
/// ```ignore
/// // Basic (no dataptr, no serialization):
/// impl_altlogical_from_data!(MyType);
///
/// // With dataptr (type must implement AltrepDataptr<i32>):
/// impl_altlogical_from_data!(MyType, dataptr);
///
/// // With serialization (type must implement AltrepSerialize):
/// impl_altlogical_from_data!(MyType, serialize);
///
/// // With both dataptr and serialization:
/// impl_altlogical_from_data!(MyType, dataptr, serialize);
/// ```
#[macro_export]
macro_rules! impl_altlogical_from_data {
    ($ty:ty) => {
        $crate::__impl_altrep_base!($ty);
        impl $crate::altrep_traits::AltVec for $ty {}
        $crate::__impl_altlogical_methods!($ty);
        $crate::impl_inferbase_logical!($ty);
    };
    ($ty:ty, dataptr) => {
        $crate::__impl_altrep_base!($ty);
        $crate::__impl_altvec_dataptr!($ty, i32);
        $crate::__impl_altlogical_methods!($ty);
        $crate::impl_inferbase_logical!($ty);
    };
    ($ty:ty, serialize) => {
        $crate::__impl_altrep_base_with_serialize!($ty);
        impl $crate::altrep_traits::AltVec for $ty {}
        $crate::__impl_altlogical_methods!($ty);
        $crate::impl_inferbase_logical!($ty);
    };
    ($ty:ty, dataptr, serialize) => {
        $crate::__impl_altrep_base_with_serialize!($ty);
        $crate::__impl_altvec_dataptr!($ty, i32);
        $crate::__impl_altlogical_methods!($ty);
        $crate::impl_inferbase_logical!($ty);
    };
    ($ty:ty, serialize, dataptr) => {
        $crate::impl_altlogical_from_data!($ty, dataptr, serialize);
    };
}

/// Internal macro: impl AltLogical methods from AltLogicalData
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_altlogical_methods {
    ($ty:ty) => {
        #[allow(clippy::not_unsafe_ptr_arg_deref)]
        impl $crate::altrep_traits::AltLogical for $ty {
            const HAS_ELT: bool = true;

            fn elt(x: $crate::ffi::SEXP, i: $crate::ffi::R_xlen_t) -> i32 {
                unsafe { $crate::altrep_data1_as::<$ty>(x) }
                    .map(|d| {
                        <$ty as $crate::altrep_data::AltLogicalData>::elt(&*d, i as usize)
                            .to_r_int()
                    })
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
                    .unwrap_or($crate::ffi::SEXP::null())
            }
        }
    };
}

/// Generate ALTREP trait implementations for a type that implements AltRawData.
///
/// ## Variants
///
/// ```ignore
/// // Basic (no dataptr, no serialization):
/// impl_altraw_from_data!(MyType);
///
/// // With dataptr (type must implement AltrepDataptr<u8>):
/// impl_altraw_from_data!(MyType, dataptr);
///
/// // With serialization (type must implement AltrepSerialize):
/// impl_altraw_from_data!(MyType, serialize);
///
/// // With both dataptr and serialization:
/// impl_altraw_from_data!(MyType, dataptr, serialize);
/// ```
#[macro_export]
macro_rules! impl_altraw_from_data {
    ($ty:ty) => {
        $crate::__impl_altrep_base!($ty);
        impl $crate::altrep_traits::AltVec for $ty {}
        $crate::__impl_altraw_methods!($ty);
        $crate::impl_inferbase_raw!($ty);
    };
    ($ty:ty, dataptr) => {
        $crate::__impl_altrep_base!($ty);
        $crate::__impl_altvec_dataptr!($ty, u8);
        $crate::__impl_altraw_methods!($ty);
        $crate::impl_inferbase_raw!($ty);
    };
    ($ty:ty, serialize) => {
        $crate::__impl_altrep_base_with_serialize!($ty);
        impl $crate::altrep_traits::AltVec for $ty {}
        $crate::__impl_altraw_methods!($ty);
        $crate::impl_inferbase_raw!($ty);
    };
    ($ty:ty, dataptr, serialize) => {
        $crate::__impl_altrep_base_with_serialize!($ty);
        $crate::__impl_altvec_dataptr!($ty, u8);
        $crate::__impl_altraw_methods!($ty);
        $crate::impl_inferbase_raw!($ty);
    };
    ($ty:ty, serialize, dataptr) => {
        $crate::impl_altraw_from_data!($ty, dataptr, serialize);
    };
}

/// Internal macro for AltRaw method implementations.
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_altraw_methods {
    ($ty:ty) => {
        #[allow(clippy::not_unsafe_ptr_arg_deref)]
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
///
/// ## Variants
///
/// ```ignore
/// // Basic (no serialization):
/// impl_altstring_from_data!(MyType);
///
/// // With serialization (type must implement AltrepSerialize):
/// impl_altstring_from_data!(MyType, serialize);
/// ```
#[macro_export]
macro_rules! impl_altstring_from_data {
    ($ty:ty) => {
        $crate::__impl_altrep_base!($ty);
        impl $crate::altrep_traits::AltVec for $ty {}
        $crate::__impl_altstring_methods!($ty);
        $crate::impl_inferbase_string!($ty);
    };
    ($ty:ty, serialize) => {
        $crate::__impl_altrep_base_with_serialize!($ty);
        impl $crate::altrep_traits::AltVec for $ty {}
        $crate::__impl_altstring_methods!($ty);
        $crate::impl_inferbase_string!($ty);
    };
}

/// Internal macro for AltString method implementations.
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_altstring_methods {
    ($ty:ty) => {
        #[allow(clippy::not_unsafe_ptr_arg_deref)]
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
        #[allow(clippy::not_unsafe_ptr_arg_deref)]
        impl $crate::altrep_traits::Altrep for $ty {
            fn length(x: $crate::ffi::SEXP) -> $crate::ffi::R_xlen_t {
                unsafe { $crate::altrep_data1_as::<$ty>(x) }
                    .map(|d| {
                        <$ty as $crate::altrep_data::AltrepLen>::len(&*d) as $crate::ffi::R_xlen_t
                    })
                    .unwrap_or(0)
            }
        }

        impl $crate::altrep_traits::AltVec for $ty {}

        #[allow(clippy::not_unsafe_ptr_arg_deref)]
        impl $crate::altrep_traits::AltList for $ty {
            fn elt(x: $crate::ffi::SEXP, i: $crate::ffi::R_xlen_t) -> $crate::ffi::SEXP {
                unsafe { $crate::altrep_data1_as::<$ty>(x) }
                    .map(|d| <$ty as $crate::altrep_data::AltListData>::elt(&*d, i as usize))
                    .unwrap_or(unsafe { $crate::ffi::R_NilValue })
            }
        }

        $crate::impl_inferbase_list!($ty);
    };
}

/// Internal macro: impl AltComplex methods (elt, get_region)
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_altcomplex_methods {
    ($ty:ty) => {
        #[allow(clippy::not_unsafe_ptr_arg_deref)]
        impl $crate::altrep_traits::AltComplex for $ty {
            const HAS_ELT: bool = true;

            fn elt(x: $crate::ffi::SEXP, i: $crate::ffi::R_xlen_t) -> $crate::ffi::Rcomplex {
                unsafe { $crate::altrep_data1_as::<$ty>(x) }
                    .map(|d| <$ty as $crate::altrep_data::AltComplexData>::elt(&*d, i as usize))
                    .unwrap_or($crate::ffi::Rcomplex {
                        r: f64::NAN,
                        i: f64::NAN,
                    })
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

/// Generate ALTREP trait implementations for a type that implements AltComplexData.
///
/// Optional features can be enabled by passing additional arguments:
/// - `dataptr`: Enable `Dataptr` and `Dataptr_or_null` methods (requires `AltrepDataptr<Rcomplex>`)
/// - `serialize`: Enable serialization support (requires `AltrepSerialize`)
/// - `subset`: Enable optimized subsetting (requires `AltrepExtractSubset`)
#[macro_export]
macro_rules! impl_altcomplex_from_data {
    ($ty:ty) => {
        $crate::__impl_altrep_base!($ty);
        impl $crate::altrep_traits::AltVec for $ty {}
        $crate::__impl_altcomplex_methods!($ty);
        $crate::impl_inferbase_complex!($ty);
    };
    ($ty:ty, dataptr) => {
        $crate::__impl_altrep_base!($ty);
        $crate::__impl_altvec_dataptr!($ty, $crate::ffi::Rcomplex);
        $crate::__impl_altcomplex_methods!($ty);
        $crate::impl_inferbase_complex!($ty);
    };
    ($ty:ty, serialize) => {
        $crate::__impl_altrep_base_with_serialize!($ty);
        impl $crate::altrep_traits::AltVec for $ty {}
        $crate::__impl_altcomplex_methods!($ty);
        $crate::impl_inferbase_complex!($ty);
    };
    ($ty:ty, subset) => {
        $crate::__impl_altrep_base!($ty);
        $crate::__impl_altvec_extract_subset!($ty);
        $crate::__impl_altcomplex_methods!($ty);
        $crate::impl_inferbase_complex!($ty);
    };
    ($ty:ty, dataptr, serialize) => {
        $crate::__impl_altrep_base_with_serialize!($ty);
        $crate::__impl_altvec_dataptr!($ty, $crate::ffi::Rcomplex);
        $crate::__impl_altcomplex_methods!($ty);
        $crate::impl_inferbase_complex!($ty);
    };
    ($ty:ty, serialize, dataptr) => {
        $crate::impl_altcomplex_from_data!($ty, dataptr, serialize);
    };
    ($ty:ty, subset, serialize) => {
        $crate::__impl_altrep_base_with_serialize!($ty);
        $crate::__impl_altvec_extract_subset!($ty);
        $crate::__impl_altcomplex_methods!($ty);
        $crate::impl_inferbase_complex!($ty);
    };
    ($ty:ty, serialize, subset) => {
        $crate::impl_altcomplex_from_data!($ty, subset, serialize);
    };
}

// =============================================================================
// Built-in implementations for standard types
// =============================================================================
// These implementations are provided here to satisfy the orphan rules.
// User crates can use these types directly with delegate_data.
//
// All types that implement AltrepSerialize get the `serialize` option enabled,
// which allows proper saveRDS/readRDS round-trips.

// Integer types - Vec<i32> supports dataptr, ranges don't (computed on demand)
impl_altinteger_from_data!(Vec<i32>, dataptr, serialize);
impl_altinteger_from_data!(std::ops::Range<i32>, serialize);
impl_altinteger_from_data!(std::ops::Range<i64>, serialize);

// Real types - Vec<f64> supports dataptr, ranges don't
impl_altreal_from_data!(Vec<f64>, dataptr, serialize);
impl_altreal_from_data!(std::ops::Range<f64>, serialize);

// Logical types
impl_altlogical_from_data!(Vec<bool>, serialize);

// Raw types
impl_altraw_from_data!(Vec<u8>, serialize);

// String types
impl_altstring_from_data!(Vec<String>, serialize);

// Complex types - Vec<Rcomplex> supports dataptr
impl_altcomplex_from_data!(Vec<crate::ffi::Rcomplex>, dataptr, serialize);

// =============================================================================
// Box<[T]> implementations
// =============================================================================
// Box<[T]> is a fat pointer (Sized) that wraps a DST slice.
// Unlike Vec<T>, it has no capacity field - just ptr + len (2 words).
// Useful for fixed-size heap allocations.

impl_altinteger_from_data!(Box<[i32]>, dataptr, serialize);
impl_altreal_from_data!(Box<[f64]>, dataptr, serialize);
impl_altlogical_from_data!(Box<[bool]>, serialize);
impl_altraw_from_data!(Box<[u8]>, serialize);
impl_altstring_from_data!(Box<[String]>, serialize);
impl_altcomplex_from_data!(Box<[crate::ffi::Rcomplex]>, dataptr, serialize);

// =============================================================================
// Array implementations (const generics - can't use macros)
// =============================================================================

// Integer arrays
impl<const N: usize> crate::altrep_traits::Altrep for [i32; N] {
    fn length(x: crate::ffi::SEXP) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<[i32; N]>(x) }
            .map(|d| crate::altrep_data::AltrepLen::len(&*d) as crate::ffi::R_xlen_t)
            .unwrap_or(0)
    }
}

impl<const N: usize> crate::altrep_traits::AltVec for [i32; N] {
    const HAS_DATAPTR: bool = true;

    fn dataptr(x: crate::ffi::SEXP, _writable: bool) -> *mut std::ffi::c_void {
        unsafe { crate::altrep_data1_as::<[i32; N]>(x) }
            .and_then(|d| {
                <[i32; N] as crate::altrep_data::AltIntegerData>::as_slice(&*d)
                    .map(|s| s.as_ptr() as *mut std::ffi::c_void)
            })
            .unwrap_or(std::ptr::null_mut())
    }
}

impl<const N: usize> crate::altrep_traits::AltInteger for [i32; N] {
    const HAS_ELT: bool = true;

    fn elt(x: crate::ffi::SEXP, i: crate::ffi::R_xlen_t) -> i32 {
        unsafe { crate::altrep_data1_as::<[i32; N]>(x) }
            .map(|d| <[i32; N] as crate::altrep_data::AltIntegerData>::elt(&*d, i as usize))
            .unwrap_or(i32::MIN)
    }

    const HAS_GET_REGION: bool = true;

    fn get_region(
        x: crate::ffi::SEXP,
        start: crate::ffi::R_xlen_t,
        len: crate::ffi::R_xlen_t,
        buf: *mut i32,
    ) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<[i32; N]>(x) }
            .map(|d| {
                let slice = unsafe { std::slice::from_raw_parts_mut(buf, len as usize) };
                <[i32; N] as crate::altrep_data::AltIntegerData>::get_region(
                    &*d,
                    start as usize,
                    len as usize,
                    slice,
                ) as crate::ffi::R_xlen_t
            })
            .unwrap_or(0)
    }

    const HAS_NO_NA: bool = true;

    fn no_na(x: crate::ffi::SEXP) -> i32 {
        unsafe { crate::altrep_data1_as::<[i32; N]>(x) }
            .and_then(|d| <[i32; N] as crate::altrep_data::AltIntegerData>::no_na(&*d))
            .map(|b| if b { 1 } else { 0 })
            .unwrap_or(0)
    }
}

// Real arrays
impl<const N: usize> crate::altrep_traits::Altrep for [f64; N] {
    fn length(x: crate::ffi::SEXP) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<[f64; N]>(x) }
            .map(|d| crate::altrep_data::AltrepLen::len(&*d) as crate::ffi::R_xlen_t)
            .unwrap_or(0)
    }
}

impl<const N: usize> crate::altrep_traits::AltVec for [f64; N] {
    const HAS_DATAPTR: bool = true;

    fn dataptr(x: crate::ffi::SEXP, _writable: bool) -> *mut std::ffi::c_void {
        unsafe { crate::altrep_data1_as::<[f64; N]>(x) }
            .and_then(|d| {
                <[f64; N] as crate::altrep_data::AltRealData>::as_slice(&*d)
                    .map(|s| s.as_ptr() as *mut std::ffi::c_void)
            })
            .unwrap_or(std::ptr::null_mut())
    }
}

impl<const N: usize> crate::altrep_traits::AltReal for [f64; N] {
    const HAS_ELT: bool = true;

    fn elt(x: crate::ffi::SEXP, i: crate::ffi::R_xlen_t) -> f64 {
        unsafe { crate::altrep_data1_as::<[f64; N]>(x) }
            .map(|d| <[f64; N] as crate::altrep_data::AltRealData>::elt(&*d, i as usize))
            .unwrap_or(f64::NAN)
    }

    const HAS_GET_REGION: bool = true;

    fn get_region(
        x: crate::ffi::SEXP,
        start: crate::ffi::R_xlen_t,
        len: crate::ffi::R_xlen_t,
        buf: *mut f64,
    ) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<[f64; N]>(x) }
            .map(|d| {
                let slice = unsafe { std::slice::from_raw_parts_mut(buf, len as usize) };
                <[f64; N] as crate::altrep_data::AltRealData>::get_region(
                    &*d,
                    start as usize,
                    len as usize,
                    slice,
                ) as crate::ffi::R_xlen_t
            })
            .unwrap_or(0)
    }

    const HAS_NO_NA: bool = true;

    fn no_na(x: crate::ffi::SEXP) -> i32 {
        unsafe { crate::altrep_data1_as::<[f64; N]>(x) }
            .and_then(|d| <[f64; N] as crate::altrep_data::AltRealData>::no_na(&*d))
            .map(|b| if b { 1 } else { 0 })
            .unwrap_or(0)
    }
}

// Logical arrays
impl<const N: usize> crate::altrep_traits::Altrep for [bool; N] {
    fn length(x: crate::ffi::SEXP) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<[bool; N]>(x) }
            .map(|d| crate::altrep_data::AltrepLen::len(&*d) as crate::ffi::R_xlen_t)
            .unwrap_or(0)
    }
}

impl<const N: usize> crate::altrep_traits::AltVec for [bool; N] {}

impl<const N: usize> crate::altrep_traits::AltLogical for [bool; N] {
    const HAS_ELT: bool = true;

    fn elt(x: crate::ffi::SEXP, i: crate::ffi::R_xlen_t) -> i32 {
        unsafe { crate::altrep_data1_as::<[bool; N]>(x) }
            .map(|d| {
                <[bool; N] as crate::altrep_data::AltLogicalData>::elt(&*d, i as usize).to_r_int()
            })
            .unwrap_or(crate::altrep_traits::NA_LOGICAL)
    }

    const HAS_NO_NA: bool = true;

    fn no_na(x: crate::ffi::SEXP) -> i32 {
        unsafe { crate::altrep_data1_as::<[bool; N]>(x) }
            .and_then(|d| <[bool; N] as crate::altrep_data::AltLogicalData>::no_na(&*d))
            .map(|b| if b { 1 } else { 0 })
            .unwrap_or(0)
    }
}

// Raw arrays
impl<const N: usize> crate::altrep_traits::Altrep for [u8; N] {
    fn length(x: crate::ffi::SEXP) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<[u8; N]>(x) }
            .map(|d| crate::altrep_data::AltrepLen::len(&*d) as crate::ffi::R_xlen_t)
            .unwrap_or(0)
    }
}

impl<const N: usize> crate::altrep_traits::AltVec for [u8; N] {
    const HAS_DATAPTR: bool = true;

    fn dataptr(x: crate::ffi::SEXP, _writable: bool) -> *mut std::ffi::c_void {
        unsafe { crate::altrep_data1_as::<[u8; N]>(x) }
            .and_then(|d| {
                <[u8; N] as crate::altrep_data::AltRawData>::as_slice(&*d)
                    .map(|s| s.as_ptr() as *mut std::ffi::c_void)
            })
            .unwrap_or(std::ptr::null_mut())
    }
}

impl<const N: usize> crate::altrep_traits::AltRaw for [u8; N] {
    const HAS_ELT: bool = true;

    fn elt(x: crate::ffi::SEXP, i: crate::ffi::R_xlen_t) -> crate::ffi::Rbyte {
        unsafe { crate::altrep_data1_as::<[u8; N]>(x) }
            .map(|d| <[u8; N] as crate::altrep_data::AltRawData>::elt(&*d, i as usize))
            .unwrap_or(0)
    }

    const HAS_GET_REGION: bool = true;

    fn get_region(
        x: crate::ffi::SEXP,
        start: crate::ffi::R_xlen_t,
        len: crate::ffi::R_xlen_t,
        buf: *mut crate::ffi::Rbyte,
    ) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<[u8; N]>(x) }
            .map(|d| {
                let slice = unsafe { std::slice::from_raw_parts_mut(buf, len as usize) };
                <[u8; N] as crate::altrep_data::AltRawData>::get_region(
                    &*d,
                    start as usize,
                    len as usize,
                    slice,
                ) as crate::ffi::R_xlen_t
            })
            .unwrap_or(0)
    }
}

// String arrays
impl<const N: usize> crate::altrep_traits::Altrep for [String; N] {
    fn length(x: crate::ffi::SEXP) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<[String; N]>(x) }
            .map(|d| crate::altrep_data::AltrepLen::len(&*d) as crate::ffi::R_xlen_t)
            .unwrap_or(0)
    }
}

impl<const N: usize> crate::altrep_traits::AltVec for [String; N] {}

impl<const N: usize> crate::altrep_traits::AltString for [String; N] {
    fn elt(x: crate::ffi::SEXP, i: crate::ffi::R_xlen_t) -> crate::ffi::SEXP {
        match unsafe { crate::altrep_data1_as::<[String; N]>(x) } {
            Some(d) => {
                match <[String; N] as crate::altrep_data::AltStringData>::elt(&*d, i as usize) {
                    Some(s) => unsafe {
                        crate::ffi::Rf_mkCharLenCE(
                            s.as_ptr().cast(),
                            s.len() as i32,
                            crate::ffi::cetype_t::CE_UTF8,
                        )
                    },
                    None => unsafe { crate::ffi::R_NaString },
                }
            }
            None => unsafe { crate::ffi::R_NaString },
        }
    }
}

// Complex arrays
impl<const N: usize> crate::altrep_traits::Altrep for [crate::ffi::Rcomplex; N] {
    fn length(x: crate::ffi::SEXP) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<[crate::ffi::Rcomplex; N]>(x) }
            .map(|d| crate::altrep_data::AltrepLen::len(&*d) as crate::ffi::R_xlen_t)
            .unwrap_or(0)
    }
}

impl<const N: usize> crate::altrep_traits::AltVec for [crate::ffi::Rcomplex; N] {
    const HAS_DATAPTR: bool = true;

    fn dataptr(x: crate::ffi::SEXP, _writable: bool) -> *mut std::ffi::c_void {
        unsafe { crate::altrep_data1_as::<[crate::ffi::Rcomplex; N]>(x) }
            .and_then(|d| {
                <[crate::ffi::Rcomplex; N] as crate::altrep_data::AltComplexData>::as_slice(&*d)
                    .map(|s| s.as_ptr() as *mut std::ffi::c_void)
            })
            .unwrap_or(std::ptr::null_mut())
    }
}

impl<const N: usize> crate::altrep_traits::AltComplex for [crate::ffi::Rcomplex; N] {
    const HAS_ELT: bool = true;

    fn elt(x: crate::ffi::SEXP, i: crate::ffi::R_xlen_t) -> crate::ffi::Rcomplex {
        unsafe { crate::altrep_data1_as::<[crate::ffi::Rcomplex; N]>(x) }
            .map(|d| {
                <[crate::ffi::Rcomplex; N] as crate::altrep_data::AltComplexData>::elt(
                    &*d, i as usize,
                )
            })
            .unwrap_or(crate::ffi::Rcomplex {
                r: f64::NAN,
                i: f64::NAN,
            })
    }

    const HAS_GET_REGION: bool = true;

    fn get_region(
        x: crate::ffi::SEXP,
        start: crate::ffi::R_xlen_t,
        len: crate::ffi::R_xlen_t,
        buf: *mut crate::ffi::Rcomplex,
    ) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<[crate::ffi::Rcomplex; N]>(x) }
            .map(|d| {
                let slice = unsafe { std::slice::from_raw_parts_mut(buf, len as usize) };
                <[crate::ffi::Rcomplex; N] as crate::altrep_data::AltComplexData>::get_region(
                    &*d,
                    start as usize,
                    len as usize,
                    slice,
                ) as crate::ffi::R_xlen_t
            })
            .unwrap_or(0)
    }
}

// =============================================================================
// InferBase implementations for arrays (const generics)
// =============================================================================
//
// These allow arrays to be registered as ALTREP classes.
// Note: Macros don't work with const generics, so these are hand-written.

impl<const N: usize> crate::altrep_data::InferBase for [i32; N] {
    const BASE: crate::altrep::RBase = crate::altrep::RBase::Int;

    unsafe fn make_class(
        class_name: *const i8,
        pkg_name: *const i8,
    ) -> crate::ffi::altrep::R_altrep_class_t {
        unsafe {
            crate::ffi::altrep::R_make_altinteger_class(class_name, pkg_name, core::ptr::null_mut())
        }
    }

    unsafe fn install_methods(cls: crate::ffi::altrep::R_altrep_class_t) {
        unsafe { crate::altrep_bridge::install_base::<Self>(cls) };
        unsafe { crate::altrep_bridge::install_vec::<Self>(cls) };
        unsafe { crate::altrep_bridge::install_int::<Self>(cls) };
    }
}

impl<const N: usize> crate::altrep_data::InferBase for [f64; N] {
    const BASE: crate::altrep::RBase = crate::altrep::RBase::Real;

    unsafe fn make_class(
        class_name: *const i8,
        pkg_name: *const i8,
    ) -> crate::ffi::altrep::R_altrep_class_t {
        unsafe {
            crate::ffi::altrep::R_make_altreal_class(class_name, pkg_name, core::ptr::null_mut())
        }
    }

    unsafe fn install_methods(cls: crate::ffi::altrep::R_altrep_class_t) {
        unsafe { crate::altrep_bridge::install_base::<Self>(cls) };
        unsafe { crate::altrep_bridge::install_vec::<Self>(cls) };
        unsafe { crate::altrep_bridge::install_real::<Self>(cls) };
    }
}

impl<const N: usize> crate::altrep_data::InferBase for [bool; N] {
    const BASE: crate::altrep::RBase = crate::altrep::RBase::Logical;

    unsafe fn make_class(
        class_name: *const i8,
        pkg_name: *const i8,
    ) -> crate::ffi::altrep::R_altrep_class_t {
        unsafe {
            crate::ffi::altrep::R_make_altlogical_class(class_name, pkg_name, core::ptr::null_mut())
        }
    }

    unsafe fn install_methods(cls: crate::ffi::altrep::R_altrep_class_t) {
        unsafe { crate::altrep_bridge::install_base::<Self>(cls) };
        unsafe { crate::altrep_bridge::install_vec::<Self>(cls) };
        unsafe { crate::altrep_bridge::install_lgl::<Self>(cls) };
    }
}

impl<const N: usize> crate::altrep_data::InferBase for [u8; N] {
    const BASE: crate::altrep::RBase = crate::altrep::RBase::Raw;

    unsafe fn make_class(
        class_name: *const i8,
        pkg_name: *const i8,
    ) -> crate::ffi::altrep::R_altrep_class_t {
        unsafe {
            crate::ffi::altrep::R_make_altraw_class(class_name, pkg_name, core::ptr::null_mut())
        }
    }

    unsafe fn install_methods(cls: crate::ffi::altrep::R_altrep_class_t) {
        unsafe { crate::altrep_bridge::install_base::<Self>(cls) };
        unsafe { crate::altrep_bridge::install_vec::<Self>(cls) };
        unsafe { crate::altrep_bridge::install_raw::<Self>(cls) };
    }
}

impl<const N: usize> crate::altrep_data::InferBase for [String; N] {
    const BASE: crate::altrep::RBase = crate::altrep::RBase::String;

    unsafe fn make_class(
        class_name: *const i8,
        pkg_name: *const i8,
    ) -> crate::ffi::altrep::R_altrep_class_t {
        unsafe {
            crate::ffi::altrep::R_make_altstring_class(class_name, pkg_name, core::ptr::null_mut())
        }
    }

    unsafe fn install_methods(cls: crate::ffi::altrep::R_altrep_class_t) {
        unsafe { crate::altrep_bridge::install_base::<Self>(cls) };
        unsafe { crate::altrep_bridge::install_vec::<Self>(cls) };
        unsafe { crate::altrep_bridge::install_str::<Self>(cls) };
    }
}

impl<const N: usize> crate::altrep_data::InferBase for [crate::ffi::Rcomplex; N] {
    const BASE: crate::altrep::RBase = crate::altrep::RBase::Complex;

    unsafe fn make_class(
        class_name: *const i8,
        pkg_name: *const i8,
    ) -> crate::ffi::altrep::R_altrep_class_t {
        unsafe {
            crate::ffi::altrep::R_make_altcomplex_class(class_name, pkg_name, core::ptr::null_mut())
        }
    }

    unsafe fn install_methods(cls: crate::ffi::altrep::R_altrep_class_t) {
        unsafe { crate::altrep_bridge::install_base::<Self>(cls) };
        unsafe { crate::altrep_bridge::install_vec::<Self>(cls) };
        unsafe { crate::altrep_bridge::install_cplx::<Self>(cls) };
    }
}

// =============================================================================
// Static slice implementations (&'static [T])
// =============================================================================
//
// `&'static [T]` is Sized (fat pointer: ptr + len) and satisfies 'static,
// so it can be used DIRECTLY with ALTREP via ExternalPtr.
//
// Use cases:
// - Const arrays: `static DATA: [i32; 5] = [1, 2, 3, 4, 5]; create_altrep(&DATA[..])`
// - Leaked data: `let s: &'static [i32] = Box::leak(vec.into_boxed_slice());`
// - Memory-mapped files with 'static lifetime

// Integer static slices
impl crate::altrep_traits::Altrep for &'static [i32] {
    fn length(x: crate::ffi::SEXP) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<&'static [i32]>(x) }
            .map(|d| crate::altrep_data::AltrepLen::len(&*d) as crate::ffi::R_xlen_t)
            .unwrap_or(0)
    }
}

impl crate::altrep_traits::AltVec for &'static [i32] {
    const HAS_DATAPTR: bool = true;

    fn dataptr(x: crate::ffi::SEXP, writable: bool) -> *mut std::ffi::c_void {
        // Static data cannot be modified. Error if writable access is requested.
        // This matches R's mmap behavior for read-only data (altclasses.c:1144-1153).
        if writable {
            unsafe {
                crate::ffi::Rf_error(
                    c"%s".as_ptr(),
                    c"cannot get writable DATAPTR for static ALTREP data".as_ptr(),
                );
            }
        }
        // For read-only access, return pointer to static data.
        // Use (*d).as_ptr() to get the slice's data pointer, not ExternalPtr::as_ptr()
        unsafe { crate::altrep_data1_as::<&'static [i32]>(x) }
            .map(|d| (*d).as_ptr() as *mut std::ffi::c_void)
            .unwrap_or(std::ptr::null_mut())
    }

    const HAS_DATAPTR_OR_NULL: bool = true;

    fn dataptr_or_null(x: crate::ffi::SEXP) -> *const std::ffi::c_void {
        unsafe { crate::altrep_data1_as::<&'static [i32]>(x) }
            .map(|d| (*d).as_ptr() as *const std::ffi::c_void)
            .unwrap_or(std::ptr::null())
    }
}

impl crate::altrep_traits::AltInteger for &'static [i32] {
    const HAS_ELT: bool = true;

    fn elt(x: crate::ffi::SEXP, i: crate::ffi::R_xlen_t) -> i32 {
        unsafe { crate::altrep_data1_as::<&'static [i32]>(x) }
            .map(|d| crate::altrep_data::AltIntegerData::elt(&*d, i as usize))
            .unwrap_or(i32::MIN)
    }

    const HAS_GET_REGION: bool = true;

    fn get_region(
        x: crate::ffi::SEXP,
        start: crate::ffi::R_xlen_t,
        len: crate::ffi::R_xlen_t,
        buf: *mut i32,
    ) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<&'static [i32]>(x) }
            .map(|d| {
                let slice = unsafe { std::slice::from_raw_parts_mut(buf, len as usize) };
                crate::altrep_data::AltIntegerData::get_region(
                    &*d,
                    start as usize,
                    len as usize,
                    slice,
                ) as crate::ffi::R_xlen_t
            })
            .unwrap_or(0)
    }

    const HAS_NO_NA: bool = true;

    fn no_na(x: crate::ffi::SEXP) -> i32 {
        unsafe { crate::altrep_data1_as::<&'static [i32]>(x) }
            .and_then(|d| crate::altrep_data::AltIntegerData::no_na(&*d))
            .map(|b| if b { 1 } else { 0 })
            .unwrap_or(0)
    }

    const HAS_SUM: bool = true;

    fn sum(x: crate::ffi::SEXP, narm: bool) -> crate::ffi::SEXP {
        unsafe { crate::altrep_data1_as::<&'static [i32]>(x) }
            .and_then(|d| crate::altrep_data::AltIntegerData::sum(&*d, narm))
            .map(|s| {
                if s >= i32::MIN as i64 && s <= i32::MAX as i64 {
                    unsafe { crate::ffi::Rf_ScalarInteger(s as i32) }
                } else {
                    unsafe { crate::ffi::Rf_ScalarReal(s as f64) }
                }
            })
            .unwrap_or(crate::ffi::SEXP::null())
    }

    const HAS_MIN: bool = true;

    fn min(x: crate::ffi::SEXP, narm: bool) -> crate::ffi::SEXP {
        unsafe { crate::altrep_data1_as::<&'static [i32]>(x) }
            .and_then(|d| crate::altrep_data::AltIntegerData::min(&*d, narm))
            .map(|m| unsafe { crate::ffi::Rf_ScalarInteger(m) })
            .unwrap_or(crate::ffi::SEXP::null())
    }

    const HAS_MAX: bool = true;

    fn max(x: crate::ffi::SEXP, narm: bool) -> crate::ffi::SEXP {
        unsafe { crate::altrep_data1_as::<&'static [i32]>(x) }
            .and_then(|d| crate::altrep_data::AltIntegerData::max(&*d, narm))
            .map(|m| unsafe { crate::ffi::Rf_ScalarInteger(m) })
            .unwrap_or(crate::ffi::SEXP::null())
    }
}

crate::impl_inferbase_integer!(&'static [i32]);

// Real static slices
impl crate::altrep_traits::Altrep for &'static [f64] {
    fn length(x: crate::ffi::SEXP) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<&'static [f64]>(x) }
            .map(|d| crate::altrep_data::AltrepLen::len(&*d) as crate::ffi::R_xlen_t)
            .unwrap_or(0)
    }
}

impl crate::altrep_traits::AltVec for &'static [f64] {
    const HAS_DATAPTR: bool = true;

    fn dataptr(x: crate::ffi::SEXP, writable: bool) -> *mut std::ffi::c_void {
        if writable {
            unsafe {
                crate::ffi::Rf_error(
                    c"%s".as_ptr(),
                    c"cannot get writable DATAPTR for static ALTREP data".as_ptr(),
                );
            }
        }
        unsafe { crate::altrep_data1_as::<&'static [f64]>(x) }
            .map(|d| (*d).as_ptr() as *mut std::ffi::c_void)
            .unwrap_or(std::ptr::null_mut())
    }

    const HAS_DATAPTR_OR_NULL: bool = true;

    fn dataptr_or_null(x: crate::ffi::SEXP) -> *const std::ffi::c_void {
        unsafe { crate::altrep_data1_as::<&'static [f64]>(x) }
            .map(|d| (*d).as_ptr() as *const std::ffi::c_void)
            .unwrap_or(std::ptr::null())
    }
}

impl crate::altrep_traits::AltReal for &'static [f64] {
    const HAS_ELT: bool = true;

    fn elt(x: crate::ffi::SEXP, i: crate::ffi::R_xlen_t) -> f64 {
        unsafe { crate::altrep_data1_as::<&'static [f64]>(x) }
            .map(|d| crate::altrep_data::AltRealData::elt(&*d, i as usize))
            .unwrap_or(f64::NAN)
    }

    const HAS_GET_REGION: bool = true;

    fn get_region(
        x: crate::ffi::SEXP,
        start: crate::ffi::R_xlen_t,
        len: crate::ffi::R_xlen_t,
        buf: *mut f64,
    ) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<&'static [f64]>(x) }
            .map(|d| {
                let slice = unsafe { std::slice::from_raw_parts_mut(buf, len as usize) };
                crate::altrep_data::AltRealData::get_region(
                    &*d,
                    start as usize,
                    len as usize,
                    slice,
                ) as crate::ffi::R_xlen_t
            })
            .unwrap_or(0)
    }

    const HAS_NO_NA: bool = true;

    fn no_na(x: crate::ffi::SEXP) -> i32 {
        unsafe { crate::altrep_data1_as::<&'static [f64]>(x) }
            .and_then(|d| crate::altrep_data::AltRealData::no_na(&*d))
            .map(|b| if b { 1 } else { 0 })
            .unwrap_or(0)
    }

    const HAS_SUM: bool = true;

    fn sum(x: crate::ffi::SEXP, narm: bool) -> crate::ffi::SEXP {
        unsafe { crate::altrep_data1_as::<&'static [f64]>(x) }
            .and_then(|d| crate::altrep_data::AltRealData::sum(&*d, narm))
            .map(|s| unsafe { crate::ffi::Rf_ScalarReal(s) })
            .unwrap_or(crate::ffi::SEXP::null())
    }

    const HAS_MIN: bool = true;

    fn min(x: crate::ffi::SEXP, narm: bool) -> crate::ffi::SEXP {
        unsafe { crate::altrep_data1_as::<&'static [f64]>(x) }
            .and_then(|d| crate::altrep_data::AltRealData::min(&*d, narm))
            .map(|m| unsafe { crate::ffi::Rf_ScalarReal(m) })
            .unwrap_or(crate::ffi::SEXP::null())
    }

    const HAS_MAX: bool = true;

    fn max(x: crate::ffi::SEXP, narm: bool) -> crate::ffi::SEXP {
        unsafe { crate::altrep_data1_as::<&'static [f64]>(x) }
            .and_then(|d| crate::altrep_data::AltRealData::max(&*d, narm))
            .map(|m| unsafe { crate::ffi::Rf_ScalarReal(m) })
            .unwrap_or(crate::ffi::SEXP::null())
    }
}

crate::impl_inferbase_real!(&'static [f64]);

// Logical static slices
impl crate::altrep_traits::Altrep for &'static [bool] {
    fn length(x: crate::ffi::SEXP) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<&'static [bool]>(x) }
            .map(|d| crate::altrep_data::AltrepLen::len(&*d) as crate::ffi::R_xlen_t)
            .unwrap_or(0)
    }
}

impl crate::altrep_traits::AltVec for &'static [bool] {}

impl crate::altrep_traits::AltLogical for &'static [bool] {
    const HAS_ELT: bool = true;

    fn elt(x: crate::ffi::SEXP, i: crate::ffi::R_xlen_t) -> i32 {
        unsafe { crate::altrep_data1_as::<&'static [bool]>(x) }
            .map(|d| crate::altrep_data::AltLogicalData::elt(&*d, i as usize).to_r_int())
            .unwrap_or(crate::altrep_traits::NA_LOGICAL)
    }

    const HAS_NO_NA: bool = true;

    fn no_na(x: crate::ffi::SEXP) -> i32 {
        unsafe { crate::altrep_data1_as::<&'static [bool]>(x) }
            .and_then(|d| crate::altrep_data::AltLogicalData::no_na(&*d))
            .map(|b| if b { 1 } else { 0 })
            .unwrap_or(0)
    }

    const HAS_SUM: bool = true;

    fn sum(x: crate::ffi::SEXP, narm: bool) -> crate::ffi::SEXP {
        unsafe { crate::altrep_data1_as::<&'static [bool]>(x) }
            .and_then(|d| crate::altrep_data::AltLogicalData::sum(&*d, narm))
            .map(|s| {
                if s >= i32::MIN as i64 && s <= i32::MAX as i64 {
                    unsafe { crate::ffi::Rf_ScalarInteger(s as i32) }
                } else {
                    unsafe { crate::ffi::Rf_ScalarReal(s as f64) }
                }
            })
            .unwrap_or(crate::ffi::SEXP::null())
    }
}

crate::impl_inferbase_logical!(&'static [bool]);

// Raw static slices
impl crate::altrep_traits::Altrep for &'static [u8] {
    fn length(x: crate::ffi::SEXP) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<&'static [u8]>(x) }
            .map(|d| crate::altrep_data::AltrepLen::len(&*d) as crate::ffi::R_xlen_t)
            .unwrap_or(0)
    }
}

impl crate::altrep_traits::AltVec for &'static [u8] {
    const HAS_DATAPTR: bool = true;

    fn dataptr(x: crate::ffi::SEXP, writable: bool) -> *mut std::ffi::c_void {
        if writable {
            unsafe {
                crate::ffi::Rf_error(
                    c"%s".as_ptr(),
                    c"cannot get writable DATAPTR for static ALTREP data".as_ptr(),
                );
            }
        }
        unsafe { crate::altrep_data1_as::<&'static [u8]>(x) }
            .map(|d| (*d).as_ptr() as *mut std::ffi::c_void)
            .unwrap_or(std::ptr::null_mut())
    }

    const HAS_DATAPTR_OR_NULL: bool = true;

    fn dataptr_or_null(x: crate::ffi::SEXP) -> *const std::ffi::c_void {
        unsafe { crate::altrep_data1_as::<&'static [u8]>(x) }
            .map(|d| (*d).as_ptr() as *const std::ffi::c_void)
            .unwrap_or(std::ptr::null())
    }
}

impl crate::altrep_traits::AltRaw for &'static [u8] {
    const HAS_ELT: bool = true;

    fn elt(x: crate::ffi::SEXP, i: crate::ffi::R_xlen_t) -> crate::ffi::Rbyte {
        unsafe { crate::altrep_data1_as::<&'static [u8]>(x) }
            .map(|d| crate::altrep_data::AltRawData::elt(&*d, i as usize))
            .unwrap_or(0)
    }

    const HAS_GET_REGION: bool = true;

    fn get_region(
        x: crate::ffi::SEXP,
        start: crate::ffi::R_xlen_t,
        len: crate::ffi::R_xlen_t,
        buf: *mut crate::ffi::Rbyte,
    ) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<&'static [u8]>(x) }
            .map(|d| {
                let slice = unsafe { std::slice::from_raw_parts_mut(buf, len as usize) };
                crate::altrep_data::AltRawData::get_region(&*d, start as usize, len as usize, slice)
                    as crate::ffi::R_xlen_t
            })
            .unwrap_or(0)
    }
}

crate::impl_inferbase_raw!(&'static [u8]);

// String static slices (owned strings)
impl crate::altrep_traits::Altrep for &'static [String] {
    fn length(x: crate::ffi::SEXP) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<&'static [String]>(x) }
            .map(|d| crate::altrep_data::AltrepLen::len(&*d) as crate::ffi::R_xlen_t)
            .unwrap_or(0)
    }
}

impl crate::altrep_traits::AltVec for &'static [String] {}

impl crate::altrep_traits::AltString for &'static [String] {
    fn elt(x: crate::ffi::SEXP, i: crate::ffi::R_xlen_t) -> crate::ffi::SEXP {
        match unsafe { crate::altrep_data1_as::<&'static [String]>(x) } {
            Some(d) => match crate::altrep_data::AltStringData::elt(&*d, i as usize) {
                Some(s) => unsafe {
                    crate::ffi::Rf_mkCharLenCE(
                        s.as_ptr().cast(),
                        s.len() as i32,
                        crate::ffi::cetype_t::CE_UTF8,
                    )
                },
                None => unsafe { crate::ffi::R_NaString },
            },
            None => unsafe { crate::ffi::R_NaString },
        }
    }

    const HAS_NO_NA: bool = true;

    fn no_na(x: crate::ffi::SEXP) -> i32 {
        unsafe { crate::altrep_data1_as::<&'static [String]>(x) }
            .and_then(|d| crate::altrep_data::AltStringData::no_na(&*d))
            .map(|b| if b { 1 } else { 0 })
            .unwrap_or(0)
    }
}

crate::impl_inferbase_string!(&'static [String]);

// String static slices (str references)
impl crate::altrep_traits::Altrep for &'static [&'static str] {
    fn length(x: crate::ffi::SEXP) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<&'static [&'static str]>(x) }
            .map(|d| crate::altrep_data::AltrepLen::len(&*d) as crate::ffi::R_xlen_t)
            .unwrap_or(0)
    }
}

impl crate::altrep_traits::AltVec for &'static [&'static str] {}

impl crate::altrep_traits::AltString for &'static [&'static str] {
    fn elt(x: crate::ffi::SEXP, i: crate::ffi::R_xlen_t) -> crate::ffi::SEXP {
        match unsafe { crate::altrep_data1_as::<&'static [&'static str]>(x) } {
            Some(d) => match crate::altrep_data::AltStringData::elt(&*d, i as usize) {
                Some(s) => unsafe {
                    crate::ffi::Rf_mkCharLenCE(
                        s.as_ptr().cast(),
                        s.len() as i32,
                        crate::ffi::cetype_t::CE_UTF8,
                    )
                },
                None => unsafe { crate::ffi::R_NaString },
            },
            None => unsafe { crate::ffi::R_NaString },
        }
    }

    const HAS_NO_NA: bool = true;

    fn no_na(x: crate::ffi::SEXP) -> i32 {
        unsafe { crate::altrep_data1_as::<&'static [&'static str]>(x) }
            .and_then(|d| crate::altrep_data::AltStringData::no_na(&*d))
            .map(|b| if b { 1 } else { 0 })
            .unwrap_or(0)
    }
}

crate::impl_inferbase_string!(&'static [&'static str]);

// =============================================================================
// RegisterAltrep implementations for builtin types
// =============================================================================
//
// These implementations provide ALTREP class registration for Vec<T>, Box<[T]>,
// and Range<T> types. They allow using these types with ALTREP via wrapper structs.
//
// Note: IntoR is NOT implemented here for Vec types because there are already
// existing IntoR implementations that copy data to R eagerly. To get ALTREP
// behavior, use wrapper structs:
//   #[miniextendr(class = "MyVec", pkg = "mypkg")]
//   pub struct MyVecClass(pub Vec<i32>);
//
// Each type uses a static OnceLock to cache the ALTREP class handle, which is
// registered on first use with the current package's name (from ALTREP_PKG_NAME).

use crate::altrep::RegisterAltrep;

/// Helper macro to implement RegisterAltrep for a builtin type.
macro_rules! impl_register_altrep_builtin {
    ($ty:ty, $class_name:expr) => {
        impl RegisterAltrep for $ty {
            fn get_or_init_class() -> crate::ffi::altrep::R_altrep_class_t {
                use std::sync::OnceLock;
                static CLASS: OnceLock<crate::ffi::altrep::R_altrep_class_t> = OnceLock::new();
                *CLASS.get_or_init(|| {
                    // Class name as null-terminated C string
                    const CLASS_NAME: &[u8] = concat!($class_name, "\0").as_bytes();
                    let cls = unsafe {
                        <$ty as crate::altrep_data::InferBase>::make_class(
                            CLASS_NAME.as_ptr() as *const std::ffi::c_char,
                            crate::AltrepPkgName::as_ptr(),
                        )
                    };
                    unsafe {
                        <$ty as crate::altrep_data::InferBase>::install_methods(cls);
                    }
                    cls
                })
            }
        }
    };
}

// Vec types - RegisterAltrep only (IntoR exists elsewhere, copies data)
impl_register_altrep_builtin!(Vec<i32>, "Vec_i32");
impl_register_altrep_builtin!(Vec<f64>, "Vec_f64");
impl_register_altrep_builtin!(Vec<bool>, "Vec_bool");
impl_register_altrep_builtin!(Vec<u8>, "Vec_u8");
impl_register_altrep_builtin!(Vec<String>, "Vec_String");
impl_register_altrep_builtin!(Vec<crate::ffi::Rcomplex>, "Vec_Rcomplex");

// Range types - RegisterAltrep only
impl_register_altrep_builtin!(std::ops::Range<i32>, "Range_i32");
impl_register_altrep_builtin!(std::ops::Range<i64>, "Range_i64");
impl_register_altrep_builtin!(std::ops::Range<f64>, "Range_f64");

// Box types - RegisterAltrep only
impl_register_altrep_builtin!(Box<[i32]>, "Box_i32");
impl_register_altrep_builtin!(Box<[f64]>, "Box_f64");
impl_register_altrep_builtin!(Box<[bool]>, "Box_bool");
impl_register_altrep_builtin!(Box<[u8]>, "Box_u8");
impl_register_altrep_builtin!(Box<[String]>, "Box_String");
impl_register_altrep_builtin!(Box<[crate::ffi::Rcomplex]>, "Box_Rcomplex");
