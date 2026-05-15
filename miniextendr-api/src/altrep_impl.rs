//! ALTREP implementation utilities.
//!
//! This module provides helper functions for implementing ALTREP classes.
//! The proc-macro uses these to generate trait implementations.
//!
//! Use `crate::altrep_data1_as` (re-exported from externalptr) to extract
//! data from an ALTREP's data1 slot.

// region: Checked string-to-CHARSXP helper

/// Create a CHARSXP from a Rust string, with checked length conversion.
///
/// # Safety
///
/// Must be called from R's main thread.
///
/// # Panics
///
/// Panics if `s.len() > i32::MAX`.
#[inline]
pub unsafe fn checked_mkchar(s: &str) -> crate::ffi::SEXP {
    let _len = i32::try_from(s.len()).unwrap_or_else(|_| {
        panic!(
            "string length {} exceeds i32::MAX for Rf_mkCharLenCE",
            s.len()
        )
    });
    crate::ffi::SEXP::charsxp(s)
}
// endregion

// region: Centralized ALTREP buffer access helper

/// Create a mutable slice from an ALTREP `get_region` output buffer pointer.
///
/// Called by the bridge trampolines (`altrep_bridge.rs`) to convert the raw
/// `*mut T` buffer from R's ALTREP dispatch into a `&mut [T]` before passing
/// it to the trait methods.
///
/// # Safety
///
/// - `buf` must be a valid, aligned, writable pointer to at least `len` elements of `T`.
/// - The caller must ensure no aliasing references to the same memory exist.
/// - This is guaranteed when called from R's ALTREP `Get_region` dispatch, which
///   provides a freshly allocated buffer.
#[inline]
pub unsafe fn altrep_region_buf<T>(buf: *mut T, len: usize) -> &'static mut [T] {
    unsafe { std::slice::from_raw_parts_mut(buf, len) }
}
// endregion

// region: Macros for generating trait implementations

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
        // Default: materializing DATAPTR — allocates INTSXP in data2 on first DATAPTR call.
        // Without this, R's default DATAPTR errors with "cannot access data pointer".
        $crate::__impl_altrep_base!($ty);
        $crate::__impl_altvec_integer_dataptr!($ty);
        $crate::__impl_altinteger_methods!($ty);
        $crate::impl_inferbase_integer!($ty);
    };
    ($ty:ty, dataptr) => {
        $crate::__impl_alt_from_data!(
            $ty,
            __impl_altinteger_methods,
            impl_inferbase_integer,
            dataptr(i32)
        );
    };
    ($ty:ty, serialize) => {
        // Serialize + materializing DATAPTR (default for computed types)
        $crate::__impl_altrep_base_with_serialize!($ty);
        $crate::__impl_altvec_integer_dataptr!($ty);
        $crate::__impl_altinteger_methods!($ty);
        $crate::impl_inferbase_integer!($ty);
    };
    ($ty:ty, subset) => {
        $crate::__impl_alt_from_data!(
            $ty,
            __impl_altinteger_methods,
            impl_inferbase_integer,
            subset
        );
    };
    ($ty:ty, dataptr, serialize) => {
        $crate::__impl_alt_from_data!(
            $ty,
            __impl_altinteger_methods,
            impl_inferbase_integer,
            dataptr(i32),
            serialize
        );
    };
    ($ty:ty, serialize, dataptr) => {
        $crate::impl_altinteger_from_data!($ty, dataptr, serialize);
    };
    ($ty:ty, subset, serialize) => {
        $crate::__impl_alt_from_data!(
            $ty,
            __impl_altinteger_methods,
            impl_inferbase_integer,
            subset,
            serialize
        );
    };
    ($ty:ty, serialize, subset) => {
        $crate::impl_altinteger_from_data!($ty, subset, serialize);
    };
    // Materializing dataptr only (no serialization)
    ($ty:ty, materializing_dataptr) => {
        $crate::__impl_altrep_base!($ty);
        $crate::__impl_altvec_integer_dataptr!($ty);
        $crate::__impl_altinteger_methods!($ty);
        $crate::impl_inferbase_integer!($ty);
    };
    // Materializing dataptr + serialize (for computed types like Range<i32>)
    ($ty:ty, materializing_dataptr, serialize) => {
        $crate::__impl_altrep_base_with_serialize!($ty);
        $crate::__impl_altvec_integer_dataptr!($ty);
        $crate::__impl_altinteger_methods!($ty);
        $crate::impl_inferbase_integer!($ty);
    };
}

/// Internal macro: impl Altrep with just length
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_altrep_base {
    ($ty:ty) => {
        $crate::__impl_altrep_base!($ty, RUnwind);
    };
    ($ty:ty, $guard:ident) => {
        impl $crate::altrep_traits::Altrep for $ty {
            const GUARD: $crate::altrep_traits::AltrepGuard =
                $crate::altrep_traits::AltrepGuard::$guard;

            fn length(x: $crate::ffi::SEXP) -> $crate::ffi::R_xlen_t {
                let data =
                    unsafe { <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
                <$ty as $crate::altrep_data::AltrepLen>::len(data) as $crate::ffi::R_xlen_t
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
/// `R_new_altrep(class, data1, SEXP::nil())` where `data1` is an `ExternalPtr<$ty>`.
///
/// This matches the proc-macro-generated `IntoR::into_sexp` behavior (data is stored in `data1`,
/// and `data2` is `R_NilValue`).
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_altrep_base_with_serialize {
    ($ty:ty) => {
        $crate::__impl_altrep_base_with_serialize!($ty, RUnwind);
    };
    ($ty:ty, $guard:ident) => {
        impl $crate::altrep_traits::Altrep for $ty {
            const GUARD: $crate::altrep_traits::AltrepGuard =
                $crate::altrep_traits::AltrepGuard::$guard;

            fn length(x: $crate::ffi::SEXP) -> $crate::ffi::R_xlen_t {
                let data =
                    unsafe { <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
                <$ty as $crate::altrep_data::AltrepLen>::len(data) as $crate::ffi::R_xlen_t
            }

            const HAS_SERIALIZED_STATE: bool = true;

            fn serialized_state(x: $crate::ffi::SEXP) -> $crate::ffi::SEXP {
                let data =
                    unsafe { <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
                <$ty as $crate::altrep_data::AltrepSerialize>::serialized_state(data)
            }

            const HAS_UNSERIALIZE: bool = true;

            fn unserialize(
                class: $crate::ffi::SEXP,
                state: $crate::ffi::SEXP,
            ) -> $crate::ffi::SEXP {
                let Some(data) = <$ty as $crate::altrep_data::AltrepSerialize>::unserialize(state)
                else {
                    panic!(
                        "ALTREP unserialize failed for {}",
                        core::any::type_name::<$ty>()
                    );
                };

                // SAFETY: Unserialize is called by R on the main thread.
                unsafe {
                    use $crate::externalptr::ExternalPtr;
                    use $crate::ffi::altrep::R_altrep_class_t;
                    use $crate::ffi::{Rf_protect_unchecked, Rf_unprotect_unchecked, SEXP};

                    let ext_ptr = ExternalPtr::new_unchecked(data);
                    let data1 = ext_ptr.as_sexp();
                    // Protect across the allocation in new_altrep_unchecked.
                    Rf_protect_unchecked(data1);
                    let cls = R_altrep_class_t::from_sexp(class);
                    let out = cls.new_altrep_unchecked(data1, SEXP::nil());
                    Rf_unprotect_unchecked(1);
                    out
                }
            }
        }
    };
}

/// Internal macro: impl AltVec with dataptr support
///
/// When `writable = true`, obtains a mutable reference to the data via
/// `altrep_data1_mut` so that writes through the returned pointer modify
/// the Rust-owned data directly. When `writable = false`, uses the
/// immutable `altrep_data1_as` + `dataptr_or_null` path, avoiding
/// unnecessary mutable borrows (and, for `Cow`, avoiding a copy-on-write).
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_altvec_dataptr {
    ($ty:ty, $elem:ty) => {
        impl $crate::altrep_traits::AltVec for $ty {
            const HAS_DATAPTR: bool = true;

            fn dataptr(x: $crate::ffi::SEXP, writable: bool) -> *mut core::ffi::c_void {
                // Check data2 cache first (materialized by a prior call).
                unsafe {
                    let data2 = $crate::altrep_ext::AltrepSexpExt::altrep_data2_raw(&x);
                    if !data2.is_null()
                        && $crate::ffi::SexpExt::type_of(&data2)
                            == <$elem as $crate::ffi::RNativeType>::SEXP_TYPE
                    {
                        return $crate::ffi::DATAPTR_RO(data2).cast_mut();
                    }
                }

                // Try the fast path: direct pointer from the underlying data.
                let direct = if writable {
                    let d = unsafe {
                        <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_mut(x)
                    };
                    <$ty as $crate::altrep_data::AltrepDataptr<$elem>>::dataptr(d, true)
                        .map(|p| p.cast::<core::ffi::c_void>())
                } else {
                    // Read-only: try immutable access first to avoid &mut borrows
                    // and unnecessary copy-on-write for Cow types.
                    // Scoped block: the &T borrow must end before altrep_extract_mut
                    // to avoid aliasing &T / &mut T (Stacked Borrows UB).
                    let ro = {
                        let d = unsafe {
                            <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x)
                        };
                        <$ty as $crate::altrep_data::AltrepDataptr<$elem>>::dataptr_or_null(d)
                    };
                    if let Some(p) = ro {
                        return p.cast_mut().cast::<core::ffi::c_void>();
                    }
                    // dataptr_or_null returned None — try mutable path.
                    let d = unsafe {
                        <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_mut(x)
                    };
                    <$ty as $crate::altrep_data::AltrepDataptr<$elem>>::dataptr(d, false)
                        .map(|p| p.cast::<core::ffi::c_void>())
                };

                if let Some(p) = direct {
                    return p;
                }

                // The underlying data can't provide a contiguous pointer (e.g., Arrow
                // array with null bitmask). Materialize into data2 via Elt methods.
                // Must never return null — R doesn't fall back when custom Dataptr is set.
                unsafe { $crate::altrep_data::materialize_altrep_data2::<$elem>(x) }
            }

            const HAS_DATAPTR_OR_NULL: bool = true;

            fn dataptr_or_null(x: $crate::ffi::SEXP) -> *const core::ffi::c_void {
                // Check data2 cache first (may have been materialized by a prior dataptr call)
                unsafe {
                    let data2 = $crate::altrep_ext::AltrepSexpExt::altrep_data2_raw(&x);
                    if !data2.is_null()
                        && $crate::ffi::SexpExt::type_of(&data2)
                            == <$elem as $crate::ffi::RNativeType>::SEXP_TYPE
                    {
                        return $crate::ffi::DATAPTR_RO(data2);
                    }
                }
                let d =
                    unsafe { <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
                <$ty as $crate::altrep_data::AltrepDataptr<$elem>>::dataptr_or_null(d)
                    .map(|p| p.cast::<core::ffi::c_void>())
                    .unwrap_or(core::ptr::null())
            }
        }
    };
}

/// Internal macro: impl AltVec with dataptr support for string ALTREP.
///
/// String vectors (STRSXP) store CHARSXP pointers, not contiguous data. This macro
/// materializes remaining uncached elements into the data2 STRSXP cache (which may
/// already have some elements from prior `Elt` calls). Returns the cached STRSXP's
/// data pointer.
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_altvec_string_dataptr {
    ($ty:ty) => {
        impl $crate::altrep_traits::AltVec for $ty {
            const HAS_DATAPTR: bool = true;

            fn dataptr(x: $crate::ffi::SEXP, _writable: bool) -> *mut core::ffi::c_void {
                unsafe {
                    let n = <$ty as $crate::altrep_traits::Altrep>::length(x);

                    // Get or allocate the data2 cache STRSXP
                    let mut data2 = $crate::altrep_ext::AltrepSexpExt::altrep_data2_raw(&x);
                    let fresh_alloc = data2.is_null()
                        || $crate::ffi::SexpExt::type_of(&data2) != $crate::ffi::SEXPTYPE::STRSXP;
                    if fresh_alloc {
                        // Rf_allocVector(STRSXP, n) leaves elements UNINITIALIZED
                        // (garbage SEXP pointers). Must fill with R_NaString sentinel
                        // so cache lookups work. This is O(n) but unavoidable.
                        data2 = $crate::ffi::Rf_protect($crate::ffi::Rf_allocVector(
                            $crate::ffi::SEXPTYPE::STRSXP,
                            n,
                        ));
                        for j in 0..n {
                            $crate::ffi::SexpExt::set_string_elt(
                                &data2,
                                j,
                                $crate::ffi::SEXP::na_string(),
                            );
                        }
                        $crate::altrep_ext::AltrepSexpExt::set_altrep_data2(&x, data2);
                        $crate::ffi::Rf_unprotect(1);
                    }

                    // Fill uncached elements only — elements already cached by Elt
                    // are non-NA CHARSXPs and are skipped. NA elements are re-probed
                    // from Rust (O(1)) to handle mixed cached/uncached NA slots.
                    for i in 0..n {
                        let cached = $crate::ffi::SexpExt::string_elt(&data2, i);
                        if cached != $crate::ffi::SEXP::na_string() {
                            continue; // already cached by a prior Elt call
                        }
                        // Compute from Rust and store
                        let elt = <$ty as $crate::altrep_traits::AltString>::elt(x, i);
                        $crate::ffi::SexpExt::set_string_elt(&data2, i, elt);
                    }

                    $crate::ffi::DATAPTR_RO(data2).cast_mut()
                }
            }

            const HAS_DATAPTR_OR_NULL: bool = true;

            fn dataptr_or_null(x: $crate::ffi::SEXP) -> *const core::ffi::c_void {
                // Always return null. The data2 STRSXP may be partially cached
                // (Elt filled some slots, others are R_NaString sentinels).
                // Returning a pointer to a partial cache would expose sentinel
                // R_NaString as actual NAs. Returning null tells R to use
                // Elt-based access, which correctly handles the per-element cache.
                // Dataptr (not dataptr_or_null) is the full-materialization path.
                let _ = x;
                core::ptr::null()
            }
        }
    };
}

/// Internal macro: impl AltVec with materializing dataptr for logical ALTREP.
///
/// Thin wrapper: provides a trivial `AltrepDataptr` (no direct pointer) and
/// delegates to `__impl_altvec_dataptr` which materializes via `RNativeType::elt`.
/// R logicals are stored as `i32` but accessed through `RLogical` for type safety.
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_altvec_logical_dataptr {
    ($ty:ty) => {
        impl $crate::altrep_data::AltrepDataptr<$crate::ffi::RLogical> for $ty {
            fn dataptr(&mut self, _writable: bool) -> Option<*mut $crate::ffi::RLogical> {
                None
            }
        }
        $crate::__impl_altvec_dataptr!($ty, $crate::ffi::RLogical);
    };
}

/// Internal macro: impl AltVec with materializing dataptr for integer ALTREP.
///
/// Internal macro: impl AltVec with materializing dataptr for integer ALTREP.
///
/// Thin wrapper: provides a trivial `AltrepDataptr` (no direct pointer) and
/// delegates to `__impl_altvec_dataptr` which materializes via `RNativeType::elt`.
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_altvec_integer_dataptr {
    ($ty:ty) => {
        impl $crate::altrep_data::AltrepDataptr<i32> for $ty {
            fn dataptr(&mut self, _writable: bool) -> Option<*mut i32> {
                None
            }
        }
        $crate::__impl_altvec_dataptr!($ty, i32);
    };
}

/// Internal macro: impl AltVec with materializing dataptr for real ALTREP.
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_altvec_real_dataptr {
    ($ty:ty) => {
        impl $crate::altrep_data::AltrepDataptr<f64> for $ty {
            fn dataptr(&mut self, _writable: bool) -> Option<*mut f64> {
                None
            }
        }
        $crate::__impl_altvec_dataptr!($ty, f64);
    };
}

/// Internal macro: impl AltVec with materializing dataptr for raw ALTREP.
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_altvec_raw_dataptr {
    ($ty:ty) => {
        impl $crate::altrep_data::AltrepDataptr<u8> for $ty {
            fn dataptr(&mut self, _writable: bool) -> Option<*mut u8> {
                None
            }
        }
        $crate::__impl_altvec_dataptr!($ty, u8);
    };
}

/// Internal macro: impl AltVec with materializing dataptr for complex ALTREP.
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_altvec_complex_dataptr {
    ($ty:ty) => {
        impl $crate::altrep_data::AltrepDataptr<$crate::ffi::Rcomplex> for $ty {
            fn dataptr(&mut self, _writable: bool) -> Option<*mut $crate::ffi::Rcomplex> {
                None
            }
        }
        $crate::__impl_altvec_dataptr!($ty, $crate::ffi::Rcomplex);
    };
}

/// Internal macro: impl AltVec with extract_subset support
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_altvec_extract_subset {
    ($ty:ty) => {
        impl $crate::altrep_traits::AltVec for $ty {
            const HAS_EXTRACT_SUBSET: bool = true;

            fn extract_subset(
                x: $crate::ffi::SEXP,
                indx: $crate::ffi::SEXP,
                _call: $crate::ffi::SEXP,
            ) -> $crate::ffi::SEXP {
                // Validate that indx is an integer vector before calling INTEGER().
                // Return NULL to signal R to use default subsetting if not.
                if $crate::ffi::SexpExt::type_of(&indx) != $crate::ffi::SEXPTYPE::INTSXP {
                    return core::ptr::null_mut();
                }

                // Convert indx SEXP to slice using SexpExt (avoids raw-ptr-deref lint)
                let indices = unsafe { $crate::ffi::SexpExt::as_slice::<i32>(&indx) };

                let data =
                    unsafe { <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
                <$ty as $crate::altrep_data::AltrepExtractSubset>::extract_subset(data, indices)
                    .unwrap_or($crate::ffi::SEXP::nil())
            }
        }
    };
}
// endregion

// region: Shared building-block macros for ALTREP trait implementations
//
// These macros expand to associated items inside `impl` blocks. They are
// invoked by the per-family `__impl_alt*_methods!` macros to eliminate
// code duplication across the 7 ALTREP type families.

/// Shared `elt` implementation for ALTREP families with direct element access.
///
/// Generates `const HAS_ELT` and `fn elt(...)` inside an impl block.
/// Used by integer, real, raw, and complex families.
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_alt_elt {
    ($ty:ty, $trait:path, $elem:ty, $na:expr) => {
        const HAS_ELT: bool = true;

        fn elt(x: $crate::ffi::SEXP, i: $crate::ffi::R_xlen_t) -> $elem {
            let data =
                unsafe { <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
            <$ty as $trait>::elt(data, i.max(0) as usize)
        }
    };
}

/// Shared `get_region` implementation for ALTREP families.
///
/// Generates `const HAS_GET_REGION` and `fn get_region(...)` inside an impl block.
/// Used by integer, real, logical, raw, and complex families.
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_alt_get_region {
    ($ty:ty, $trait:path, $buf_ty:ty) => {
        const HAS_GET_REGION: bool = true;

        fn get_region(
            x: $crate::ffi::SEXP,
            start: $crate::ffi::R_xlen_t,
            len: $crate::ffi::R_xlen_t,
            buf: &mut [$buf_ty],
        ) -> $crate::ffi::R_xlen_t {
            if start < 0 || len <= 0 {
                return 0;
            }
            let data =
                unsafe { <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
            let len = len as usize;
            <$ty as $trait>::get_region(data, start as usize, len, buf) as $crate::ffi::R_xlen_t
        }
    };
}

/// Shared `is_sorted` implementation for ALTREP families.
///
/// Generates `const HAS_IS_SORTED` and `fn is_sorted(...)` inside an impl block.
/// Used by integer, real, logical, and string families.
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_alt_is_sorted {
    ($ty:ty, $trait:path) => {
        const HAS_IS_SORTED: bool = true;

        fn is_sorted(x: $crate::ffi::SEXP) -> i32 {
            let data =
                unsafe { <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
            <$ty as $trait>::is_sorted(data)
                .map(|s| s.to_r_int())
                .unwrap_or(i32::MIN)
        }
    };
}

/// Shared `no_na` implementation for ALTREP families.
///
/// Generates `const HAS_NO_NA` and `fn no_na(...)` inside an impl block.
/// Used by integer, real, logical, and string families.
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_alt_no_na {
    ($ty:ty, $trait:path) => {
        const HAS_NO_NA: bool = true;

        fn no_na(x: $crate::ffi::SEXP) -> i32 {
            let data =
                unsafe { <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
            <$ty as $trait>::no_na(data)
                .map(|b| if b { 1 } else { 0 })
                .unwrap_or(0)
        }
    };
}
// endregion

// region: Parametric macro: __impl_alt_from_data!
//
// This internal macro generates the standard ALTREP trait implementations
// (Altrep, AltVec, family-specific methods, InferBase) for a given type.
// The 7 public `impl_alt*_from_data!` macros delegate to this with
// family-specific parameters.

#[macro_export]
#[doc(hidden)]
macro_rules! __impl_alt_from_data {
    // Base: no options
    ($ty:ty, $methods:ident, $inferbase:ident) => {
        $crate::__impl_altrep_base!($ty);
        impl $crate::altrep_traits::AltVec for $ty {}
        $crate::$methods!($ty);
        $crate::$inferbase!($ty);
    };
    // Base: explicit guard
    ($ty:ty, $methods:ident, $inferbase:ident, @guard $guard:ident) => {
        $crate::__impl_altrep_base!($ty, $guard);
        impl $crate::altrep_traits::AltVec for $ty {}
        $crate::$methods!($ty);
        $crate::$inferbase!($ty);
    };
    // Dataptr with element type
    ($ty:ty, $methods:ident, $inferbase:ident, dataptr($elem:ty)) => {
        $crate::__impl_altrep_base!($ty);
        $crate::__impl_altvec_dataptr!($ty, $elem);
        $crate::$methods!($ty);
        $crate::$inferbase!($ty);
    };
    // String dataptr (materialization into STRSXP)
    ($ty:ty, $methods:ident, $inferbase:ident, string_dataptr) => {
        $crate::__impl_altrep_base!($ty);
        $crate::__impl_altvec_string_dataptr!($ty);
        $crate::$methods!($ty);
        $crate::$inferbase!($ty);
    };
    // String dataptr + explicit guard
    ($ty:ty, $methods:ident, $inferbase:ident, string_dataptr, @guard $guard:ident) => {
        $crate::__impl_altrep_base!($ty, $guard);
        $crate::__impl_altvec_string_dataptr!($ty);
        $crate::$methods!($ty);
        $crate::$inferbase!($ty);
    };
    // Serialize only
    ($ty:ty, $methods:ident, $inferbase:ident, serialize) => {
        $crate::__impl_altrep_base_with_serialize!($ty);
        impl $crate::altrep_traits::AltVec for $ty {}
        $crate::$methods!($ty);
        $crate::$inferbase!($ty);
    };
    // Serialize + explicit guard
    ($ty:ty, $methods:ident, $inferbase:ident, serialize, @guard $guard:ident) => {
        $crate::__impl_altrep_base_with_serialize!($ty, $guard);
        impl $crate::altrep_traits::AltVec for $ty {}
        $crate::$methods!($ty);
        $crate::$inferbase!($ty);
    };
    // Subset only
    ($ty:ty, $methods:ident, $inferbase:ident, subset) => {
        $crate::__impl_altrep_base!($ty);
        $crate::__impl_altvec_extract_subset!($ty);
        $crate::$methods!($ty);
        $crate::$inferbase!($ty);
    };
    // Dataptr + serialize
    ($ty:ty, $methods:ident, $inferbase:ident, dataptr($elem:ty), serialize) => {
        $crate::__impl_altrep_base_with_serialize!($ty);
        $crate::__impl_altvec_dataptr!($ty, $elem);
        $crate::$methods!($ty);
        $crate::$inferbase!($ty);
    };
    // String dataptr + serialize
    ($ty:ty, $methods:ident, $inferbase:ident, string_dataptr, serialize) => {
        $crate::__impl_altrep_base_with_serialize!($ty);
        $crate::__impl_altvec_string_dataptr!($ty);
        $crate::$methods!($ty);
        $crate::$inferbase!($ty);
    };
    // String dataptr + serialize + explicit guard
    ($ty:ty, $methods:ident, $inferbase:ident, string_dataptr, serialize, @guard $guard:ident) => {
        $crate::__impl_altrep_base_with_serialize!($ty, $guard);
        $crate::__impl_altvec_string_dataptr!($ty);
        $crate::$methods!($ty);
        $crate::$inferbase!($ty);
    };
    // Subset + serialize
    ($ty:ty, $methods:ident, $inferbase:ident, subset, serialize) => {
        $crate::__impl_altrep_base_with_serialize!($ty);
        $crate::__impl_altvec_extract_subset!($ty);
        $crate::$methods!($ty);
        $crate::$inferbase!($ty);
    };
}
// endregion

// region: Per-family method macros (using shared building blocks)

/// Internal macro for AltInteger method implementations.
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_altinteger_methods {
    ($ty:ty) => {
        impl $crate::altrep_traits::AltInteger for $ty {
            $crate::__impl_alt_elt!($ty, $crate::altrep_data::AltIntegerData, i32, i32::MIN);
            $crate::__impl_alt_get_region!($ty, $crate::altrep_data::AltIntegerData, i32);
            $crate::__impl_alt_is_sorted!($ty, $crate::altrep_data::AltIntegerData);
            $crate::__impl_alt_no_na!($ty, $crate::altrep_data::AltIntegerData);

            const HAS_SUM: bool = true;

            // ALTREP protocol: return C NULL (not R_NilValue) to signal "can't compute"
            fn sum(x: $crate::ffi::SEXP, narm: bool) -> $crate::ffi::SEXP {
                let data =
                    unsafe { <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
                match <$ty as $crate::altrep_data::AltIntegerData>::sum(data, narm) {
                    Some(s) => {
                        if s >= i32::MIN as i64 && s <= i32::MAX as i64 {
                            $crate::ffi::SEXP::scalar_integer(s as i32)
                        } else {
                            $crate::ffi::SEXP::scalar_real(s as f64)
                        }
                    }
                    None => $crate::ffi::SEXP::null(),
                }
            }

            const HAS_MIN: bool = true;

            fn min(x: $crate::ffi::SEXP, narm: bool) -> $crate::ffi::SEXP {
                let data =
                    unsafe { <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
                <$ty as $crate::altrep_data::AltIntegerData>::min(data, narm)
                    .map(|m| $crate::ffi::SEXP::scalar_integer(m))
                    .unwrap_or($crate::ffi::SEXP::null())
            }

            const HAS_MAX: bool = true;

            fn max(x: $crate::ffi::SEXP, narm: bool) -> $crate::ffi::SEXP {
                let data =
                    unsafe { <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
                <$ty as $crate::altrep_data::AltIntegerData>::max(data, narm)
                    .map(|m| $crate::ffi::SEXP::scalar_integer(m))
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
/// // With subset optimization (type must implement AltrepExtractSubset):
/// impl_altreal_from_data!(MyType, subset);
///
/// // Combine multiple options:
/// impl_altreal_from_data!(MyType, dataptr, serialize);
/// impl_altreal_from_data!(MyType, subset, serialize);
/// ```
#[macro_export]
macro_rules! impl_altreal_from_data {
    ($ty:ty) => {
        $crate::__impl_altrep_base!($ty);
        $crate::__impl_altvec_real_dataptr!($ty);
        $crate::__impl_altreal_methods!($ty);
        $crate::impl_inferbase_real!($ty);
    };
    ($ty:ty, dataptr) => {
        $crate::__impl_alt_from_data!(
            $ty,
            __impl_altreal_methods,
            impl_inferbase_real,
            dataptr(f64)
        );
    };
    ($ty:ty, serialize) => {
        $crate::__impl_altrep_base_with_serialize!($ty);
        $crate::__impl_altvec_real_dataptr!($ty);
        $crate::__impl_altreal_methods!($ty);
        $crate::impl_inferbase_real!($ty);
    };
    ($ty:ty, dataptr, serialize) => {
        $crate::__impl_alt_from_data!(
            $ty,
            __impl_altreal_methods,
            impl_inferbase_real,
            dataptr(f64),
            serialize
        );
    };
    ($ty:ty, serialize, dataptr) => {
        $crate::impl_altreal_from_data!($ty, dataptr, serialize);
    };
    ($ty:ty, subset) => {
        $crate::__impl_alt_from_data!($ty, __impl_altreal_methods, impl_inferbase_real, subset);
    };
    ($ty:ty, subset, serialize) => {
        $crate::__impl_alt_from_data!(
            $ty,
            __impl_altreal_methods,
            impl_inferbase_real,
            subset,
            serialize
        );
    };
    ($ty:ty, serialize, subset) => {
        $crate::impl_altreal_from_data!($ty, subset, serialize);
    };
    // Materializing dataptr only (no serialization)
    ($ty:ty, materializing_dataptr) => {
        $crate::__impl_altrep_base!($ty);
        $crate::__impl_altvec_real_dataptr!($ty);
        $crate::__impl_altreal_methods!($ty);
        $crate::impl_inferbase_real!($ty);
    };
    // Materializing dataptr + serialize (for computed types like Range<f64>)
    ($ty:ty, materializing_dataptr, serialize) => {
        $crate::__impl_altrep_base_with_serialize!($ty);
        $crate::__impl_altvec_real_dataptr!($ty);
        $crate::__impl_altreal_methods!($ty);
        $crate::impl_inferbase_real!($ty);
    };
}

/// Internal macro for AltReal method implementations.
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_altreal_methods {
    ($ty:ty) => {
        impl $crate::altrep_traits::AltReal for $ty {
            $crate::__impl_alt_elt!($ty, $crate::altrep_data::AltRealData, f64, f64::NAN);
            $crate::__impl_alt_get_region!($ty, $crate::altrep_data::AltRealData, f64);
            $crate::__impl_alt_is_sorted!($ty, $crate::altrep_data::AltRealData);
            $crate::__impl_alt_no_na!($ty, $crate::altrep_data::AltRealData);

            const HAS_SUM: bool = true;

            // ALTREP protocol: return C NULL (not R_NilValue) to signal "can't compute"
            fn sum(x: $crate::ffi::SEXP, narm: bool) -> $crate::ffi::SEXP {
                let data =
                    unsafe { <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
                <$ty as $crate::altrep_data::AltRealData>::sum(data, narm)
                    .map(|s| $crate::ffi::SEXP::scalar_real(s))
                    .unwrap_or($crate::ffi::SEXP::null())
            }

            const HAS_MIN: bool = true;

            fn min(x: $crate::ffi::SEXP, narm: bool) -> $crate::ffi::SEXP {
                let data =
                    unsafe { <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
                <$ty as $crate::altrep_data::AltRealData>::min(data, narm)
                    .map(|m| $crate::ffi::SEXP::scalar_real(m))
                    .unwrap_or($crate::ffi::SEXP::null())
            }

            const HAS_MAX: bool = true;

            fn max(x: $crate::ffi::SEXP, narm: bool) -> $crate::ffi::SEXP {
                let data =
                    unsafe { <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
                <$ty as $crate::altrep_data::AltRealData>::max(data, narm)
                    .map(|m| $crate::ffi::SEXP::scalar_real(m))
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
/// // With subset optimization (type must implement AltrepExtractSubset):
/// impl_altlogical_from_data!(MyType, subset);
///
/// // Combine multiple options:
/// impl_altlogical_from_data!(MyType, dataptr, serialize);
/// impl_altlogical_from_data!(MyType, subset, serialize);
/// ```
#[macro_export]
macro_rules! impl_altlogical_from_data {
    ($ty:ty) => {
        $crate::__impl_altrep_base!($ty);
        $crate::__impl_altvec_logical_dataptr!($ty);
        $crate::__impl_altlogical_methods!($ty);
        $crate::impl_inferbase_logical!($ty);
    };
    ($ty:ty, dataptr) => {
        $crate::__impl_alt_from_data!(
            $ty,
            __impl_altlogical_methods,
            impl_inferbase_logical,
            dataptr(i32)
        );
    };
    ($ty:ty, serialize) => {
        $crate::__impl_altrep_base_with_serialize!($ty);
        $crate::__impl_altvec_logical_dataptr!($ty);
        $crate::__impl_altlogical_methods!($ty);
        $crate::impl_inferbase_logical!($ty);
    };
    ($ty:ty, dataptr, serialize) => {
        $crate::__impl_alt_from_data!(
            $ty,
            __impl_altlogical_methods,
            impl_inferbase_logical,
            dataptr(i32),
            serialize
        );
    };
    ($ty:ty, serialize, dataptr) => {
        $crate::impl_altlogical_from_data!($ty, dataptr, serialize);
    };
    ($ty:ty, subset) => {
        $crate::__impl_alt_from_data!(
            $ty,
            __impl_altlogical_methods,
            impl_inferbase_logical,
            subset
        );
    };
    ($ty:ty, subset, serialize) => {
        $crate::__impl_alt_from_data!(
            $ty,
            __impl_altlogical_methods,
            impl_inferbase_logical,
            subset,
            serialize
        );
    };
    ($ty:ty, serialize, subset) => {
        $crate::impl_altlogical_from_data!($ty, subset, serialize);
    };
    // Materializing dataptr only (no serialization)
    ($ty:ty, materializing_dataptr) => {
        $crate::__impl_altrep_base!($ty);
        $crate::__impl_altvec_logical_dataptr!($ty);
        $crate::__impl_altlogical_methods!($ty);
        $crate::impl_inferbase_logical!($ty);
    };
    // Materializing dataptr + serialize (for bool types that need bool→i32 conversion)
    ($ty:ty, materializing_dataptr, serialize) => {
        $crate::__impl_altrep_base_with_serialize!($ty);
        $crate::__impl_altvec_logical_dataptr!($ty);
        $crate::__impl_altlogical_methods!($ty);
        $crate::impl_inferbase_logical!($ty);
    };
}

/// Internal macro: impl AltLogical methods from AltLogicalData
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_altlogical_methods {
    ($ty:ty) => {
        impl $crate::altrep_traits::AltLogical for $ty {
            // Logical elt is special: returns Logical → .to_r_int()
            const HAS_ELT: bool = true;

            fn elt(x: $crate::ffi::SEXP, i: $crate::ffi::R_xlen_t) -> i32 {
                let data =
                    unsafe { <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
                <$ty as $crate::altrep_data::AltLogicalData>::elt(data, i.max(0) as usize)
                    .to_r_int()
            }

            $crate::__impl_alt_get_region!($ty, $crate::altrep_data::AltLogicalData, i32);
            $crate::__impl_alt_is_sorted!($ty, $crate::altrep_data::AltLogicalData);
            $crate::__impl_alt_no_na!($ty, $crate::altrep_data::AltLogicalData);

            const HAS_SUM: bool = true;

            // ALTREP protocol: return C NULL (not R_NilValue) to signal "can't compute"
            fn sum(x: $crate::ffi::SEXP, narm: bool) -> $crate::ffi::SEXP {
                let data =
                    unsafe { <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
                match <$ty as $crate::altrep_data::AltLogicalData>::sum(data, narm) {
                    Some(s) => {
                        if s >= i32::MIN as i64 && s <= i32::MAX as i64 {
                            $crate::ffi::SEXP::scalar_integer(s as i32)
                        } else {
                            $crate::ffi::SEXP::scalar_real(s as f64)
                        }
                    }
                    None => $crate::ffi::SEXP::null(),
                }
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
/// // With subset optimization (type must implement AltrepExtractSubset):
/// impl_altraw_from_data!(MyType, subset);
///
/// // Combine multiple options:
/// impl_altraw_from_data!(MyType, dataptr, serialize);
/// impl_altraw_from_data!(MyType, subset, serialize);
/// ```
#[macro_export]
macro_rules! impl_altraw_from_data {
    ($ty:ty) => {
        $crate::__impl_altrep_base!($ty);
        $crate::__impl_altvec_raw_dataptr!($ty);
        $crate::__impl_altraw_methods!($ty);
        $crate::impl_inferbase_raw!($ty);
    };
    ($ty:ty, dataptr) => {
        $crate::__impl_alt_from_data!($ty, __impl_altraw_methods, impl_inferbase_raw, dataptr(u8));
    };
    ($ty:ty, serialize) => {
        $crate::__impl_altrep_base_with_serialize!($ty);
        $crate::__impl_altvec_raw_dataptr!($ty);
        $crate::__impl_altraw_methods!($ty);
        $crate::impl_inferbase_raw!($ty);
    };
    ($ty:ty, dataptr, serialize) => {
        $crate::__impl_alt_from_data!(
            $ty,
            __impl_altraw_methods,
            impl_inferbase_raw,
            dataptr(u8),
            serialize
        );
    };
    ($ty:ty, serialize, dataptr) => {
        $crate::impl_altraw_from_data!($ty, dataptr, serialize);
    };
    ($ty:ty, subset) => {
        $crate::__impl_alt_from_data!($ty, __impl_altraw_methods, impl_inferbase_raw, subset);
    };
    ($ty:ty, subset, serialize) => {
        $crate::__impl_alt_from_data!(
            $ty,
            __impl_altraw_methods,
            impl_inferbase_raw,
            subset,
            serialize
        );
    };
    ($ty:ty, serialize, subset) => {
        $crate::impl_altraw_from_data!($ty, subset, serialize);
    };
}

/// Internal macro for AltRaw method implementations.
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_altraw_methods {
    ($ty:ty) => {
        impl $crate::altrep_traits::AltRaw for $ty {
            $crate::__impl_alt_elt!($ty, $crate::altrep_data::AltRawData, u8, 0);
            $crate::__impl_alt_get_region!($ty, $crate::altrep_data::AltRawData, u8);
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
/// // With dataptr (materialized STRSXP):
/// impl_altstring_from_data!(MyType, dataptr);
///
/// // With serialization (type must implement AltrepSerialize):
/// impl_altstring_from_data!(MyType, serialize);
///
/// // With subset optimization (type must implement AltrepExtractSubset):
/// impl_altstring_from_data!(MyType, subset);
///
/// // Combine multiple options:
/// impl_altstring_from_data!(MyType, dataptr, serialize);
/// impl_altstring_from_data!(MyType, subset, serialize);
/// ```
#[macro_export]
macro_rules! impl_altstring_from_data {
    ($ty:ty) => {
        $crate::__impl_altrep_base!($ty);
        $crate::__impl_altvec_string_dataptr!($ty);
        $crate::__impl_altstring_methods!($ty);
        $crate::impl_inferbase_string!($ty);
    };
    ($ty:ty, dataptr) => {
        $crate::__impl_alt_from_data!(
            $ty,
            __impl_altstring_methods,
            impl_inferbase_string,
            string_dataptr
        );
    };
    ($ty:ty, serialize) => {
        $crate::__impl_altrep_base_with_serialize!($ty);
        $crate::__impl_altvec_string_dataptr!($ty);
        $crate::__impl_altstring_methods!($ty);
        $crate::impl_inferbase_string!($ty);
    };
    ($ty:ty, dataptr, serialize) => {
        $crate::__impl_alt_from_data!(
            $ty,
            __impl_altstring_methods,
            impl_inferbase_string,
            string_dataptr,
            serialize
        );
    };
    ($ty:ty, subset) => {
        $crate::__impl_alt_from_data!($ty, __impl_altstring_methods, impl_inferbase_string, subset);
    };
    ($ty:ty, subset, serialize) => {
        $crate::__impl_alt_from_data!(
            $ty,
            __impl_altstring_methods,
            impl_inferbase_string,
            subset,
            serialize
        );
    };
    ($ty:ty, serialize, subset) => {
        $crate::impl_altstring_from_data!($ty, subset, serialize);
    };
}

/// Internal macro for AltString method implementations.
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_altstring_methods {
    ($ty:ty) => {
        impl $crate::altrep_traits::AltString for $ty {
            // String elt with lazy per-element caching in data2 STRSXP.
            //
            // On first access, allocates a STRSXP in data2 (initialized to R_NaString).
            // Each element is computed from Rust on first access and cached. Subsequent
            // accesses return the cached CHARSXP directly.
            //
            // For NA elements (Rust elt returns None), data2[i] stays R_NaString — we
            // re-probe Rust each time (O(1) index, returns None immediately). This is
            // simpler than a separate materialization bitmap and the cost is negligible.
            fn elt(x: $crate::ffi::SEXP, i: $crate::ffi::R_xlen_t) -> $crate::ffi::SEXP {
                unsafe {
                    let idx = i.max(0) as usize;

                    // Get or allocate the data2 cache STRSXP
                    let mut data2 = $crate::altrep_ext::AltrepSexpExt::altrep_data2_raw(&x);
                    if data2.is_null()
                        || $crate::ffi::SexpExt::type_of(&data2) != $crate::ffi::SEXPTYPE::STRSXP
                    {
                        let n = <$ty as $crate::altrep_traits::Altrep>::length(x);
                        // Rf_allocVector(STRSXP, n) leaves elements UNINITIALIZED
                        // (garbage SEXP pointers). Must fill with R_NaString sentinel.
                        data2 = $crate::ffi::Rf_protect($crate::ffi::Rf_allocVector(
                            $crate::ffi::SEXPTYPE::STRSXP,
                            n,
                        ));
                        for j in 0..n {
                            $crate::ffi::SexpExt::set_string_elt(
                                &data2,
                                j,
                                $crate::ffi::SEXP::na_string(),
                            );
                        }
                        $crate::altrep_ext::AltrepSexpExt::set_altrep_data2(&x, data2);
                        $crate::ffi::Rf_unprotect(1);
                    }

                    // Check cache: non-NA means already materialized
                    let cached = $crate::ffi::SexpExt::string_elt(&data2, i);
                    if cached != $crate::ffi::SEXP::na_string() {
                        return cached;
                    }

                    // Cache miss (or genuine NA) — probe Rust source
                    let data = <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x);
                    match <$ty as $crate::altrep_data::AltStringData>::elt(data, idx) {
                        Some(s) => {
                            let charsxp = $crate::altrep_impl::checked_mkchar(s);
                            $crate::ffi::SexpExt::set_string_elt(&data2, i, charsxp);
                            charsxp
                        }
                        None => $crate::ffi::SEXP::na_string(),
                    }
                }
            }

            $crate::__impl_alt_is_sorted!($ty, $crate::altrep_data::AltStringData);
            $crate::__impl_alt_no_na!($ty, $crate::altrep_data::AltStringData);
        }
    };
}

/// Generate ALTREP trait implementations for a type that implements AltListData.
#[macro_export]
macro_rules! impl_altlist_from_data {
    ($ty:ty) => {
        $crate::impl_altlist_from_data!($ty, RUnwind);
    };
    ($ty:ty, $guard:ident) => {
        impl $crate::altrep_traits::Altrep for $ty {
            const GUARD: $crate::altrep_traits::AltrepGuard =
                $crate::altrep_traits::AltrepGuard::$guard;

            fn length(x: $crate::ffi::SEXP) -> $crate::ffi::R_xlen_t {
                let data =
                    unsafe { <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
                <$ty as $crate::altrep_data::AltrepLen>::len(data) as $crate::ffi::R_xlen_t
            }
        }

        impl $crate::altrep_traits::AltVec for $ty {}

        impl $crate::altrep_traits::AltList for $ty {
            fn elt(x: $crate::ffi::SEXP, i: $crate::ffi::R_xlen_t) -> $crate::ffi::SEXP {
                let data =
                    unsafe { <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
                <$ty as $crate::altrep_data::AltListData>::elt(data, i.max(0) as usize)
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
        impl $crate::altrep_traits::AltComplex for $ty {
            $crate::__impl_alt_elt!(
                $ty,
                $crate::altrep_data::AltComplexData,
                $crate::ffi::Rcomplex,
                $crate::ffi::Rcomplex {
                    r: f64::NAN,
                    i: f64::NAN
                }
            );
            $crate::__impl_alt_get_region!(
                $ty,
                $crate::altrep_data::AltComplexData,
                $crate::ffi::Rcomplex
            );
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
        $crate::__impl_altvec_complex_dataptr!($ty);
        $crate::__impl_altcomplex_methods!($ty);
        $crate::impl_inferbase_complex!($ty);
    };
    ($ty:ty, dataptr) => {
        $crate::__impl_alt_from_data!(
            $ty,
            __impl_altcomplex_methods,
            impl_inferbase_complex,
            dataptr($crate::ffi::Rcomplex)
        );
    };
    ($ty:ty, serialize) => {
        $crate::__impl_altrep_base_with_serialize!($ty);
        $crate::__impl_altvec_complex_dataptr!($ty);
        $crate::__impl_altcomplex_methods!($ty);
        $crate::impl_inferbase_complex!($ty);
    };
    ($ty:ty, subset) => {
        $crate::__impl_alt_from_data!(
            $ty,
            __impl_altcomplex_methods,
            impl_inferbase_complex,
            subset
        );
    };
    ($ty:ty, dataptr, serialize) => {
        $crate::__impl_alt_from_data!(
            $ty,
            __impl_altcomplex_methods,
            impl_inferbase_complex,
            dataptr($crate::ffi::Rcomplex),
            serialize
        );
    };
    ($ty:ty, serialize, dataptr) => {
        $crate::impl_altcomplex_from_data!($ty, dataptr, serialize);
    };
    ($ty:ty, subset, serialize) => {
        $crate::__impl_alt_from_data!(
            $ty,
            __impl_altcomplex_methods,
            impl_inferbase_complex,
            subset,
            serialize
        );
    };
    ($ty:ty, serialize, subset) => {
        $crate::impl_altcomplex_from_data!($ty, subset, serialize);
    };
}
// endregion

// region: Meta-macros for built-in ALTREP family instantiation
//
// `impl_builtin_altrep_family!` is a single declarative meta-macro that maps a
// (type, family, dataptr-mode, sym) 4-tuple to the correct per-family
// `impl_alt*_from_data!` call AND emits a linkme `MX_ALTREP_REGISTRATIONS` entry
// so the class is registered at `R_init_*` time (no hand-enumerated list needed).
//
// The `$sym:ident` parameter is a unique snake_case identifier used as the suffix
// for the `#[no_mangle]` registration fn and the linkme static.  It must be unique
// across all call sites — naming convention: `<container>_<elem>` (e.g. `Vec_i32`,
// `Box_bool`, `Cow_str`, `Range_i32`).
//
// ## Families
//
// | Token     | Delegates to               |
// |-----------|----------------------------|
// | `integer` | `impl_altinteger_from_data!` |
// | `real`    | `impl_altreal_from_data!`    |
// | `logical` | `impl_altlogical_from_data!` |
// | `raw`     | `impl_altraw_from_data!`     |
// | `string`  | `impl_altstring_from_data!`  |
// | `complex` | `impl_altcomplex_from_data!` |
//
// ## Dataptr modes
//
// | Token         | Meaning                                                     |
// |---------------|-------------------------------------------------------------|
// | `dataptr`     | Type has a direct contiguous pointer (`RNativeType` backed) |
// | `materializing` | No direct pointer; materializes into data2 on first access  |
//
// The `materializing` arm expands to `materializing_dataptr, serialize` in the
// underlying macro (bool→i32 conversion, Range compute-on-access, etc.).
// Both arms preserve `const GUARD = AltrepGuard::RUnwind` — the default from
// `__impl_altrep_base!` and `__impl_altrep_base_with_serialize!`.
//
// ## Corner cases NOT handled by this macro
//
// - `[T; N]` const-generic arrays: use const generics (`impl<const N>`); left in the
//   "Array implementations" region below.
// - `&'static [T]` static slices: unique lifetime + writable-assert pattern; left in
//   the "Static slice implementations" region below.

/// Generate ALTREP trait impls AND a linkme `MX_ALTREP_REGISTRATIONS` entry for a
/// builtin type.
///
/// ## Arguments
///
/// - `$ty:ty` — the builtin container type (e.g. `Vec<i32>`, `Box<[f64]>`)
/// - `$family:ident` — ALTREP family token: `integer`, `real`, `logical`, `raw`,
///   `string`, or `complex`
/// - `$mode:ident` — dataptr mode: `dataptr` or `materializing`
/// - `$reg_fn:ident` — unique `#[no_mangle]` name for the registration fn
///   (convention: `__mx_altrep_reg_builtin_<sym>`)
/// - `$entry_ident:ident` — unique name for the linkme static
///   (convention: `__MX_ALTREP_REG_ENTRY_builtin_<sym>`)
///
/// Both identifier arguments must be globally unique across all call sites.
/// The registration fn is always emitted; the `#[distributed_slice]` attribute
/// is guarded by `cfg_attr(not(target_arch = "wasm32"), ...)` so linkme's
/// compile-error arm is not reached on wasm32 targets.
#[doc(hidden)]
macro_rules! impl_builtin_altrep_family {
    // dataptr arm — type has a direct contiguous native pointer
    ($ty:ty, integer, dataptr, $reg_fn:ident, $entry_ident:ident) => {
        $crate::impl_altinteger_from_data!($ty, dataptr, serialize);
        #[doc(hidden)]
        #[unsafe(no_mangle)]
        pub extern "C" fn $reg_fn() {
            <$ty as $crate::altrep::RegisterAltrep>::get_or_init_class();
        }
        #[cfg_attr(
                    not(target_arch = "wasm32"),
                    $crate::linkme::distributed_slice($crate::registry::MX_ALTREP_REGISTRATIONS),
                    linkme(crate = $crate::linkme)
                )]
        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        static $entry_ident: $crate::registry::AltrepRegistration =
            $crate::registry::AltrepRegistration {
                register: $reg_fn,
                symbol: stringify!($reg_fn),
            };
    };
    ($ty:ty, real, dataptr, $reg_fn:ident, $entry_ident:ident) => {
        $crate::impl_altreal_from_data!($ty, dataptr, serialize);
        #[doc(hidden)]
        #[unsafe(no_mangle)]
        pub extern "C" fn $reg_fn() {
            <$ty as $crate::altrep::RegisterAltrep>::get_or_init_class();
        }
        #[cfg_attr(
                    not(target_arch = "wasm32"),
                    $crate::linkme::distributed_slice($crate::registry::MX_ALTREP_REGISTRATIONS),
                    linkme(crate = $crate::linkme)
                )]
        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        static $entry_ident: $crate::registry::AltrepRegistration =
            $crate::registry::AltrepRegistration {
                register: $reg_fn,
                symbol: stringify!($reg_fn),
            };
    };
    ($ty:ty, logical, dataptr, $reg_fn:ident, $entry_ident:ident) => {
        $crate::impl_altlogical_from_data!($ty, dataptr, serialize);
        #[doc(hidden)]
        #[unsafe(no_mangle)]
        pub extern "C" fn $reg_fn() {
            <$ty as $crate::altrep::RegisterAltrep>::get_or_init_class();
        }
        #[cfg_attr(
                    not(target_arch = "wasm32"),
                    $crate::linkme::distributed_slice($crate::registry::MX_ALTREP_REGISTRATIONS),
                    linkme(crate = $crate::linkme)
                )]
        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        static $entry_ident: $crate::registry::AltrepRegistration =
            $crate::registry::AltrepRegistration {
                register: $reg_fn,
                symbol: stringify!($reg_fn),
            };
    };
    ($ty:ty, raw, dataptr, $reg_fn:ident, $entry_ident:ident) => {
        $crate::impl_altraw_from_data!($ty, dataptr, serialize);
        #[doc(hidden)]
        #[unsafe(no_mangle)]
        pub extern "C" fn $reg_fn() {
            <$ty as $crate::altrep::RegisterAltrep>::get_or_init_class();
        }
        #[cfg_attr(
                    not(target_arch = "wasm32"),
                    $crate::linkme::distributed_slice($crate::registry::MX_ALTREP_REGISTRATIONS),
                    linkme(crate = $crate::linkme)
                )]
        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        static $entry_ident: $crate::registry::AltrepRegistration =
            $crate::registry::AltrepRegistration {
                register: $reg_fn,
                symbol: stringify!($reg_fn),
            };
    };
    ($ty:ty, string, dataptr, $reg_fn:ident, $entry_ident:ident) => {
        $crate::impl_altstring_from_data!($ty, dataptr, serialize);
        #[doc(hidden)]
        #[unsafe(no_mangle)]
        pub extern "C" fn $reg_fn() {
            <$ty as $crate::altrep::RegisterAltrep>::get_or_init_class();
        }
        #[cfg_attr(
                    not(target_arch = "wasm32"),
                    $crate::linkme::distributed_slice($crate::registry::MX_ALTREP_REGISTRATIONS),
                    linkme(crate = $crate::linkme)
                )]
        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        static $entry_ident: $crate::registry::AltrepRegistration =
            $crate::registry::AltrepRegistration {
                register: $reg_fn,
                symbol: stringify!($reg_fn),
            };
    };
    ($ty:ty, complex, dataptr, $reg_fn:ident, $entry_ident:ident) => {
        $crate::impl_altcomplex_from_data!($ty, dataptr, serialize);
        #[doc(hidden)]
        #[unsafe(no_mangle)]
        pub extern "C" fn $reg_fn() {
            <$ty as $crate::altrep::RegisterAltrep>::get_or_init_class();
        }
        #[cfg_attr(
                    not(target_arch = "wasm32"),
                    $crate::linkme::distributed_slice($crate::registry::MX_ALTREP_REGISTRATIONS),
                    linkme(crate = $crate::linkme)
                )]
        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        static $entry_ident: $crate::registry::AltrepRegistration =
            $crate::registry::AltrepRegistration {
                register: $reg_fn,
                symbol: stringify!($reg_fn),
            };
    };
    // materializing arm — no direct native pointer; materializes on DATAPTR access.
    // Used for: bool (bool→i32 via LGLSXP), Range<T> (compute-on-access).
    // Guard remains RUnwind (the default) — the underlying macros do not change it.
    ($ty:ty, integer, materializing, $reg_fn:ident, $entry_ident:ident) => {
        $crate::impl_altinteger_from_data!($ty, materializing_dataptr, serialize);
        #[doc(hidden)]
        #[unsafe(no_mangle)]
        pub extern "C" fn $reg_fn() {
            <$ty as $crate::altrep::RegisterAltrep>::get_or_init_class();
        }
        #[cfg_attr(
                    not(target_arch = "wasm32"),
                    $crate::linkme::distributed_slice($crate::registry::MX_ALTREP_REGISTRATIONS),
                    linkme(crate = $crate::linkme)
                )]
        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        static $entry_ident: $crate::registry::AltrepRegistration =
            $crate::registry::AltrepRegistration {
                register: $reg_fn,
                symbol: stringify!($reg_fn),
            };
    };
    ($ty:ty, real, materializing, $reg_fn:ident, $entry_ident:ident) => {
        $crate::impl_altreal_from_data!($ty, materializing_dataptr, serialize);
        #[doc(hidden)]
        #[unsafe(no_mangle)]
        pub extern "C" fn $reg_fn() {
            <$ty as $crate::altrep::RegisterAltrep>::get_or_init_class();
        }
        #[cfg_attr(
                    not(target_arch = "wasm32"),
                    $crate::linkme::distributed_slice($crate::registry::MX_ALTREP_REGISTRATIONS),
                    linkme(crate = $crate::linkme)
                )]
        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        static $entry_ident: $crate::registry::AltrepRegistration =
            $crate::registry::AltrepRegistration {
                register: $reg_fn,
                symbol: stringify!($reg_fn),
            };
    };
    ($ty:ty, logical, materializing, $reg_fn:ident, $entry_ident:ident) => {
        $crate::impl_altlogical_from_data!($ty, materializing_dataptr, serialize);
        #[doc(hidden)]
        #[unsafe(no_mangle)]
        pub extern "C" fn $reg_fn() {
            <$ty as $crate::altrep::RegisterAltrep>::get_or_init_class();
        }
        #[cfg_attr(
                    not(target_arch = "wasm32"),
                    $crate::linkme::distributed_slice($crate::registry::MX_ALTREP_REGISTRATIONS),
                    linkme(crate = $crate::linkme)
                )]
        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        static $entry_ident: $crate::registry::AltrepRegistration =
            $crate::registry::AltrepRegistration {
                register: $reg_fn,
                symbol: stringify!($reg_fn),
            };
    };
    ($ty:ty, raw, materializing, $reg_fn:ident, $entry_ident:ident) => {
        $crate::impl_altraw_from_data!($ty, materializing_dataptr, serialize);
        #[doc(hidden)]
        #[unsafe(no_mangle)]
        pub extern "C" fn $reg_fn() {
            <$ty as $crate::altrep::RegisterAltrep>::get_or_init_class();
        }
        #[cfg_attr(
                    not(target_arch = "wasm32"),
                    $crate::linkme::distributed_slice($crate::registry::MX_ALTREP_REGISTRATIONS),
                    linkme(crate = $crate::linkme)
                )]
        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        static $entry_ident: $crate::registry::AltrepRegistration =
            $crate::registry::AltrepRegistration {
                register: $reg_fn,
                symbol: stringify!($reg_fn),
            };
    };
    ($ty:ty, string, materializing, $reg_fn:ident, $entry_ident:ident) => {
        $crate::impl_altstring_from_data!($ty, materializing_dataptr, serialize);
        #[doc(hidden)]
        #[unsafe(no_mangle)]
        pub extern "C" fn $reg_fn() {
            <$ty as $crate::altrep::RegisterAltrep>::get_or_init_class();
        }
        #[cfg_attr(
                    not(target_arch = "wasm32"),
                    $crate::linkme::distributed_slice($crate::registry::MX_ALTREP_REGISTRATIONS),
                    linkme(crate = $crate::linkme)
                )]
        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        static $entry_ident: $crate::registry::AltrepRegistration =
            $crate::registry::AltrepRegistration {
                register: $reg_fn,
                symbol: stringify!($reg_fn),
            };
    };
    ($ty:ty, complex, materializing, $reg_fn:ident, $entry_ident:ident) => {
        $crate::impl_altcomplex_from_data!($ty, materializing_dataptr, serialize);
        #[doc(hidden)]
        #[unsafe(no_mangle)]
        pub extern "C" fn $reg_fn() {
            <$ty as $crate::altrep::RegisterAltrep>::get_or_init_class();
        }
        #[cfg_attr(
                    not(target_arch = "wasm32"),
                    $crate::linkme::distributed_slice($crate::registry::MX_ALTREP_REGISTRATIONS),
                    linkme(crate = $crate::linkme)
                )]
        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        static $entry_ident: $crate::registry::AltrepRegistration =
            $crate::registry::AltrepRegistration {
                register: $reg_fn,
                symbol: stringify!($reg_fn),
            };
    };
}

// region: Built-in implementations for standard types
// These implementations are provided here to satisfy the orphan rules.
// User crates can use these types directly with delegate_data.
//
// All types implement AltrepSerialize; serialize is injected by the meta-macro.
// Guard: RUnwind (default from __impl_altrep_base[_with_serialize]! — not overridden).

// Vec<T> — owned, heap-allocated contiguous storage.
// bool uses `materializing` because bool is not RNativeType (R stores logicals as i32).
impl_builtin_altrep_family!(
    Vec<i32>,
    integer,
    dataptr,
    __mx_altrep_reg_builtin_Vec_i32,
    __MX_ALTREP_REG_ENTRY_builtin_Vec_i32
);
impl_builtin_altrep_family!(
    Vec<f64>,
    real,
    dataptr,
    __mx_altrep_reg_builtin_Vec_f64,
    __MX_ALTREP_REG_ENTRY_builtin_Vec_f64
);
impl_builtin_altrep_family!(
    Vec<bool>,
    logical,
    materializing,
    __mx_altrep_reg_builtin_Vec_bool,
    __MX_ALTREP_REG_ENTRY_builtin_Vec_bool
);
impl_builtin_altrep_family!(
    Vec<u8>,
    raw,
    dataptr,
    __mx_altrep_reg_builtin_Vec_u8,
    __MX_ALTREP_REG_ENTRY_builtin_Vec_u8
);
impl_builtin_altrep_family!(
    Vec<String>,
    string,
    dataptr,
    __mx_altrep_reg_builtin_Vec_String,
    __MX_ALTREP_REG_ENTRY_builtin_Vec_String
);
impl_builtin_altrep_family!(
    Vec<Option<String>>,
    string,
    dataptr,
    __mx_altrep_reg_builtin_Vec_Option_String,
    __MX_ALTREP_REG_ENTRY_builtin_Vec_Option_String
);
// Cow string vectors — zero-copy from R, ALTREP output without copying back.
// Serialize: Rf_mkCharLenCE hits R's CHARSXP cache (no string data copy for borrowed).
// Unserialize: TryFromSexp uses charsxp_to_cow (zero-copy borrow for UTF-8).
impl_builtin_altrep_family!(
    Vec<std::borrow::Cow<'static, str>>,
    string,
    dataptr,
    __mx_altrep_reg_builtin_Vec_Cow_str,
    __MX_ALTREP_REG_ENTRY_builtin_Vec_Cow_str
);
impl_builtin_altrep_family!(
    Vec<Option<std::borrow::Cow<'static, str>>>,
    string,
    dataptr,
    __mx_altrep_reg_builtin_Vec_Option_Cow_str,
    __MX_ALTREP_REG_ENTRY_builtin_Vec_Option_Cow_str
);
impl_builtin_altrep_family!(
    Vec<crate::ffi::Rcomplex>,
    complex,
    dataptr,
    __mx_altrep_reg_builtin_Vec_Rcomplex,
    __MX_ALTREP_REG_ENTRY_builtin_Vec_Rcomplex
);

// Range<T> — compute-on-access (no direct pointer); materializes into data2 INTSXP/REALSXP.
impl_builtin_altrep_family!(
    std::ops::Range<i32>,
    integer,
    materializing,
    __mx_altrep_reg_builtin_Range_i32,
    __MX_ALTREP_REG_ENTRY_builtin_Range_i32
);
impl_builtin_altrep_family!(
    std::ops::Range<i64>,
    integer,
    materializing,
    __mx_altrep_reg_builtin_Range_i64,
    __MX_ALTREP_REG_ENTRY_builtin_Range_i64
);
impl_builtin_altrep_family!(
    std::ops::Range<f64>,
    real,
    materializing,
    __mx_altrep_reg_builtin_Range_f64,
    __MX_ALTREP_REG_ENTRY_builtin_Range_f64
);
// endregion

// region: Box<[T]> implementations
// Box<[T]> is a fat pointer (Sized) that wraps a DST slice.
// Unlike Vec<T>, it has no capacity field - just ptr + len (2 words).
// Useful for fixed-size heap allocations.
// bool uses `materializing` (same reason as Vec<bool>).

impl_builtin_altrep_family!(
    Box<[i32]>,
    integer,
    dataptr,
    __mx_altrep_reg_builtin_Box_i32,
    __MX_ALTREP_REG_ENTRY_builtin_Box_i32
);
impl_builtin_altrep_family!(
    Box<[f64]>,
    real,
    dataptr,
    __mx_altrep_reg_builtin_Box_f64,
    __MX_ALTREP_REG_ENTRY_builtin_Box_f64
);
impl_builtin_altrep_family!(
    Box<[bool]>,
    logical,
    materializing,
    __mx_altrep_reg_builtin_Box_bool,
    __MX_ALTREP_REG_ENTRY_builtin_Box_bool
);
impl_builtin_altrep_family!(
    Box<[u8]>,
    raw,
    dataptr,
    __mx_altrep_reg_builtin_Box_u8,
    __MX_ALTREP_REG_ENTRY_builtin_Box_u8
);
impl_builtin_altrep_family!(
    Box<[String]>,
    string,
    dataptr,
    __mx_altrep_reg_builtin_Box_String,
    __MX_ALTREP_REG_ENTRY_builtin_Box_String
);
impl_builtin_altrep_family!(
    Box<[crate::ffi::Rcomplex]>,
    complex,
    dataptr,
    __mx_altrep_reg_builtin_Box_Rcomplex,
    __MX_ALTREP_REG_ENTRY_builtin_Box_Rcomplex
);

// Cow<'static, [T]> — zero-copy borrow from R with copy-on-write dataptr.
// Borrowed variants expose R's data directly; Owned behaves like Vec.
impl_builtin_altrep_family!(
    std::borrow::Cow<'static, [i32]>,
    integer,
    dataptr,
    __mx_altrep_reg_builtin_Cow_i32,
    __MX_ALTREP_REG_ENTRY_builtin_Cow_i32
);
impl_builtin_altrep_family!(
    std::borrow::Cow<'static, [f64]>,
    real,
    dataptr,
    __mx_altrep_reg_builtin_Cow_f64,
    __MX_ALTREP_REG_ENTRY_builtin_Cow_f64
);
impl_builtin_altrep_family!(
    std::borrow::Cow<'static, [u8]>,
    raw,
    dataptr,
    __mx_altrep_reg_builtin_Cow_u8,
    __MX_ALTREP_REG_ENTRY_builtin_Cow_u8
);
impl_builtin_altrep_family!(
    std::borrow::Cow<'static, [crate::ffi::Rcomplex]>,
    complex,
    dataptr,
    __mx_altrep_reg_builtin_Cow_Rcomplex,
    __MX_ALTREP_REG_ENTRY_builtin_Cow_Rcomplex
);

// endregion

// region: Array implementations (const generics)
//
// Macro-generated for numeric families (i32, f64, u8, Rcomplex) that share
// the same pattern: Altrep + AltVec with dataptr + family trait + InferBase.
// Bool and String arrays are hand-written because they differ structurally
// (bool has no direct dataptr; String elt returns SEXP).

/// Generate all ALTREP trait impls + InferBase for a numeric [T; N] array family.
/// Pass optional extra items via `extra { ... }` to include in the family trait impl.
macro_rules! impl_altrep_array_numeric {
    (
        elem = $elem:ty,
        data_trait = $data_trait:path,
        alt_trait = $alt_trait:path,
        rbase = $rbase:expr,
        make_class_fn = $make_class_fn:path,
        install_family_fn = $install_family_fn:ident
        $(, extra { $($extra:tt)* } )?
        $(,)?
    ) => {
        impl<const N: usize> crate::altrep_traits::Altrep for [$elem; N] {
            fn length(x: crate::ffi::SEXP) -> crate::ffi::R_xlen_t {
                let data = unsafe {
                    <[$elem; N] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x)
                };
                crate::altrep_data::AltrepLen::len(data) as crate::ffi::R_xlen_t
            }
        }

        impl<const N: usize> crate::altrep_traits::AltVec for [$elem; N] {
            const HAS_DATAPTR: bool = true;

            fn dataptr(x: crate::ffi::SEXP, _writable: bool) -> *mut core::ffi::c_void {
                let data = unsafe {
                    <[$elem; N] as crate::altrep_data::AltrepExtract>::altrep_extract_mut(x)
                };
                data.as_mut_ptr().cast::<core::ffi::c_void>()
            }
        }

        impl<const N: usize> $alt_trait for [$elem; N] {
            const HAS_ELT: bool = true;

            fn elt(x: crate::ffi::SEXP, i: crate::ffi::R_xlen_t) -> $elem {
                let data = unsafe {
                    <[$elem; N] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x)
                };
                <[$elem; N] as $data_trait>::elt(data, i.max(0) as usize)
            }

            const HAS_GET_REGION: bool = true;

            fn get_region(
                x: crate::ffi::SEXP,
                start: crate::ffi::R_xlen_t,
                len: crate::ffi::R_xlen_t,
                buf: &mut [$elem],
            ) -> crate::ffi::R_xlen_t {
                if start < 0 || len <= 0 {
                    return 0;
                }
                let data = unsafe {
                    <[$elem; N] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x)
                };
                <[$elem; N] as $data_trait>::get_region(data, start as usize, len as usize, buf)
                    as crate::ffi::R_xlen_t
            }

            $($($extra)*)?
        }

        impl<const N: usize> crate::altrep_data::InferBase for [$elem; N] {
            const BASE: crate::altrep::RBase = $rbase;

            unsafe fn make_class(
                class_name: *const i8,
                pkg_name: *const i8,
            ) -> crate::ffi::altrep::R_altrep_class_t {
                let cls = unsafe { $make_class_fn(class_name, pkg_name, crate::altrep_dll_info()) };
                let name = unsafe { core::ffi::CStr::from_ptr(class_name) };
                crate::altrep::validate_altrep_class(cls, name, Self::BASE)
            }

            unsafe fn install_methods(cls: crate::ffi::altrep::R_altrep_class_t) {
                unsafe { crate::altrep_bridge::install_base::<Self>(cls) };
                unsafe { crate::altrep_bridge::install_vec::<Self>(cls) };
                unsafe { crate::altrep_bridge::$install_family_fn::<Self>(cls) };
            }
        }
    };
}

/// no_na fragment for families that support it (Integer, Real).
macro_rules! altrep_array_no_na {
    ($elem:ty, $data_trait:path) => {
        const HAS_NO_NA: bool = true;

        fn no_na(x: crate::ffi::SEXP) -> i32 {
            let data =
                unsafe { <[$elem; N] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
            <[$elem; N] as $data_trait>::no_na(data)
                .map(i32::from)
                .unwrap_or(0)
        }
    };
}

impl_altrep_array_numeric!(
    elem = i32,
    data_trait = crate::altrep_data::AltIntegerData,
    alt_trait = crate::altrep_traits::AltInteger,
    rbase = crate::altrep::RBase::Int,
    make_class_fn = crate::ffi::altrep::R_make_altinteger_class,
    install_family_fn = install_int,
    extra { altrep_array_no_na!(i32, crate::altrep_data::AltIntegerData); },
);
impl_altrep_array_numeric!(
    elem = f64,
    data_trait = crate::altrep_data::AltRealData,
    alt_trait = crate::altrep_traits::AltReal,
    rbase = crate::altrep::RBase::Real,
    make_class_fn = crate::ffi::altrep::R_make_altreal_class,
    install_family_fn = install_real,
    extra { altrep_array_no_na!(f64, crate::altrep_data::AltRealData); },
);
impl_altrep_array_numeric!(
    elem = u8,
    data_trait = crate::altrep_data::AltRawData,
    alt_trait = crate::altrep_traits::AltRaw,
    rbase = crate::altrep::RBase::Raw,
    make_class_fn = crate::ffi::altrep::R_make_altraw_class,
    install_family_fn = install_raw,
);
impl_altrep_array_numeric!(
    elem = crate::ffi::Rcomplex,
    data_trait = crate::altrep_data::AltComplexData,
    alt_trait = crate::altrep_traits::AltComplex,
    rbase = crate::altrep::RBase::Complex,
    make_class_fn = crate::ffi::altrep::R_make_altcomplex_class,
    install_family_fn = install_cplx,
);

// Logical arrays — bool != i32, no direct dataptr, elt returns i32 via to_r_int()
impl<const N: usize> crate::altrep_traits::Altrep for [bool; N] {
    fn length(x: crate::ffi::SEXP) -> crate::ffi::R_xlen_t {
        let data =
            unsafe { <[bool; N] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        crate::altrep_data::AltrepLen::len(data) as crate::ffi::R_xlen_t
    }
}

impl<const N: usize> crate::altrep_traits::AltVec for [bool; N] {}

impl<const N: usize> crate::altrep_traits::AltLogical for [bool; N] {
    const HAS_ELT: bool = true;

    fn elt(x: crate::ffi::SEXP, i: crate::ffi::R_xlen_t) -> i32 {
        let data =
            unsafe { <[bool; N] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        <[bool; N] as crate::altrep_data::AltLogicalData>::elt(data, i.max(0) as usize).to_r_int()
    }

    const HAS_NO_NA: bool = true;

    fn no_na(x: crate::ffi::SEXP) -> i32 {
        let data =
            unsafe { <[bool; N] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        <[bool; N] as crate::altrep_data::AltLogicalData>::no_na(data)
            .map(i32::from)
            .unwrap_or(0)
    }
}

impl<const N: usize> crate::altrep_data::InferBase for [bool; N] {
    const BASE: crate::altrep::RBase = crate::altrep::RBase::Logical;

    unsafe fn make_class(
        class_name: *const i8,
        pkg_name: *const i8,
    ) -> crate::ffi::altrep::R_altrep_class_t {
        let cls = unsafe {
            crate::ffi::altrep::R_make_altlogical_class(
                class_name,
                pkg_name,
                crate::altrep_dll_info(),
            )
        };
        let name = unsafe { core::ffi::CStr::from_ptr(class_name) };
        crate::altrep::validate_altrep_class(cls, name, Self::BASE)
    }

    unsafe fn install_methods(cls: crate::ffi::altrep::R_altrep_class_t) {
        unsafe { crate::altrep_bridge::install_base::<Self>(cls) };
        unsafe { crate::altrep_bridge::install_vec::<Self>(cls) };
        unsafe { crate::altrep_bridge::install_lgl::<Self>(cls) };
    }
}

// String arrays — no dataptr, elt returns SEXP via checked_mkchar
impl<const N: usize> crate::altrep_traits::Altrep for [String; N] {
    const GUARD: crate::altrep_traits::AltrepGuard = crate::altrep_traits::AltrepGuard::RUnwind;

    fn length(x: crate::ffi::SEXP) -> crate::ffi::R_xlen_t {
        let data =
            unsafe { <[String; N] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        crate::altrep_data::AltrepLen::len(data) as crate::ffi::R_xlen_t
    }
}

impl<const N: usize> crate::altrep_traits::AltVec for [String; N] {}

impl<const N: usize> crate::altrep_traits::AltString for [String; N] {
    fn elt(x: crate::ffi::SEXP, i: crate::ffi::R_xlen_t) -> crate::ffi::SEXP {
        let data =
            unsafe { <[String; N] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        match <[String; N] as crate::altrep_data::AltStringData>::elt(data, i.max(0) as usize) {
            Some(s) => unsafe { checked_mkchar(s) },
            None => crate::ffi::SEXP::na_string(),
        }
    }
}

impl<const N: usize> crate::altrep_data::InferBase for [String; N] {
    const BASE: crate::altrep::RBase = crate::altrep::RBase::String;

    unsafe fn make_class(
        class_name: *const i8,
        pkg_name: *const i8,
    ) -> crate::ffi::altrep::R_altrep_class_t {
        let cls = unsafe {
            crate::ffi::altrep::R_make_altstring_class(
                class_name,
                pkg_name,
                crate::altrep_dll_info(),
            )
        };
        let name = unsafe { core::ffi::CStr::from_ptr(class_name) };
        crate::altrep::validate_altrep_class(cls, name, Self::BASE)
    }

    unsafe fn install_methods(cls: crate::ffi::altrep::R_altrep_class_t) {
        unsafe { crate::altrep_bridge::install_base::<Self>(cls) };
        unsafe { crate::altrep_bridge::install_vec::<Self>(cls) };
        unsafe { crate::altrep_bridge::install_str::<Self>(cls) };
    }
}
// endregion

// region: Static slice implementations (&'static [T])
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
        let data =
            unsafe { <&'static [i32] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        crate::altrep_data::AltrepLen::len(data) as crate::ffi::R_xlen_t
    }
}

impl crate::altrep_traits::AltVec for &'static [i32] {
    const HAS_DATAPTR: bool = true;

    fn dataptr(x: crate::ffi::SEXP, writable: bool) -> *mut std::ffi::c_void {
        // Static data cannot be modified. Panic is caught by RUnwind guard.
        assert!(
            !writable,
            "cannot get writable DATAPTR for static ALTREP data"
        );
        let data =
            unsafe { <&'static [i32] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        data.as_ptr().cast::<std::ffi::c_void>().cast_mut()
    }

    const HAS_DATAPTR_OR_NULL: bool = true;

    fn dataptr_or_null(x: crate::ffi::SEXP) -> *const std::ffi::c_void {
        let data =
            unsafe { <&'static [i32] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        data.as_ptr().cast::<std::ffi::c_void>()
    }
}

impl crate::altrep_traits::AltInteger for &'static [i32] {
    const HAS_ELT: bool = true;

    fn elt(x: crate::ffi::SEXP, i: crate::ffi::R_xlen_t) -> i32 {
        let data =
            unsafe { <&'static [i32] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        crate::altrep_data::AltIntegerData::elt(data, i.max(0) as usize)
    }

    const HAS_GET_REGION: bool = true;

    fn get_region(
        x: crate::ffi::SEXP,
        start: crate::ffi::R_xlen_t,
        len: crate::ffi::R_xlen_t,
        buf: &mut [i32],
    ) -> crate::ffi::R_xlen_t {
        if start < 0 || len <= 0 {
            return 0;
        }
        let data =
            unsafe { <&'static [i32] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        let len = len as usize;
        crate::altrep_data::AltIntegerData::get_region(data, start as usize, len, buf)
            as crate::ffi::R_xlen_t
    }

    const HAS_NO_NA: bool = true;

    fn no_na(x: crate::ffi::SEXP) -> i32 {
        let data =
            unsafe { <&'static [i32] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        crate::altrep_data::AltIntegerData::no_na(data)
            .map(|b| if b { 1 } else { 0 })
            .unwrap_or(0)
    }

    const HAS_SUM: bool = true;

    // ALTREP protocol: return C NULL (not R_NilValue) to signal "can't compute"
    fn sum(x: crate::ffi::SEXP, narm: bool) -> crate::ffi::SEXP {
        let data =
            unsafe { <&'static [i32] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        crate::altrep_data::AltIntegerData::sum(data, narm)
            .map(|s| {
                if s >= i32::MIN as i64 && s <= i32::MAX as i64 {
                    crate::ffi::SEXP::scalar_integer(s as i32)
                } else {
                    crate::ffi::SEXP::scalar_real(s as f64)
                }
            })
            .unwrap_or(crate::ffi::SEXP::null())
    }

    const HAS_MIN: bool = true;

    fn min(x: crate::ffi::SEXP, narm: bool) -> crate::ffi::SEXP {
        let data =
            unsafe { <&'static [i32] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        crate::altrep_data::AltIntegerData::min(data, narm)
            .map(crate::ffi::SEXP::scalar_integer)
            .unwrap_or(crate::ffi::SEXP::null())
    }

    const HAS_MAX: bool = true;

    fn max(x: crate::ffi::SEXP, narm: bool) -> crate::ffi::SEXP {
        let data =
            unsafe { <&'static [i32] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        crate::altrep_data::AltIntegerData::max(data, narm)
            .map(crate::ffi::SEXP::scalar_integer)
            .unwrap_or(crate::ffi::SEXP::null())
    }
}

crate::impl_inferbase_integer!(&'static [i32]);

// Real static slices
impl crate::altrep_traits::Altrep for &'static [f64] {
    fn length(x: crate::ffi::SEXP) -> crate::ffi::R_xlen_t {
        let data =
            unsafe { <&'static [f64] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        crate::altrep_data::AltrepLen::len(data) as crate::ffi::R_xlen_t
    }
}

impl crate::altrep_traits::AltVec for &'static [f64] {
    const HAS_DATAPTR: bool = true;

    fn dataptr(x: crate::ffi::SEXP, writable: bool) -> *mut std::ffi::c_void {
        assert!(
            !writable,
            "cannot get writable DATAPTR for static ALTREP data"
        );
        let data =
            unsafe { <&'static [f64] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        data.as_ptr().cast::<std::ffi::c_void>().cast_mut()
    }

    const HAS_DATAPTR_OR_NULL: bool = true;

    fn dataptr_or_null(x: crate::ffi::SEXP) -> *const std::ffi::c_void {
        let data =
            unsafe { <&'static [f64] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        data.as_ptr().cast::<std::ffi::c_void>()
    }
}

impl crate::altrep_traits::AltReal for &'static [f64] {
    const HAS_ELT: bool = true;

    fn elt(x: crate::ffi::SEXP, i: crate::ffi::R_xlen_t) -> f64 {
        let data =
            unsafe { <&'static [f64] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        crate::altrep_data::AltRealData::elt(data, i.max(0) as usize)
    }

    const HAS_GET_REGION: bool = true;

    fn get_region(
        x: crate::ffi::SEXP,
        start: crate::ffi::R_xlen_t,
        len: crate::ffi::R_xlen_t,
        buf: &mut [f64],
    ) -> crate::ffi::R_xlen_t {
        if start < 0 || len <= 0 {
            return 0;
        }
        let data =
            unsafe { <&'static [f64] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        let len = len as usize;
        crate::altrep_data::AltRealData::get_region(data, start as usize, len, buf)
            as crate::ffi::R_xlen_t
    }

    const HAS_NO_NA: bool = true;

    fn no_na(x: crate::ffi::SEXP) -> i32 {
        let data =
            unsafe { <&'static [f64] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        crate::altrep_data::AltRealData::no_na(data)
            .map(|b| if b { 1 } else { 0 })
            .unwrap_or(0)
    }

    const HAS_SUM: bool = true;

    // ALTREP protocol: return C NULL (not R_NilValue) to signal "can't compute"
    fn sum(x: crate::ffi::SEXP, narm: bool) -> crate::ffi::SEXP {
        let data =
            unsafe { <&'static [f64] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        crate::altrep_data::AltRealData::sum(data, narm)
            .map(crate::ffi::SEXP::scalar_real)
            .unwrap_or(crate::ffi::SEXP::null())
    }

    const HAS_MIN: bool = true;

    fn min(x: crate::ffi::SEXP, narm: bool) -> crate::ffi::SEXP {
        let data =
            unsafe { <&'static [f64] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        crate::altrep_data::AltRealData::min(data, narm)
            .map(crate::ffi::SEXP::scalar_real)
            .unwrap_or(crate::ffi::SEXP::null())
    }

    const HAS_MAX: bool = true;

    fn max(x: crate::ffi::SEXP, narm: bool) -> crate::ffi::SEXP {
        let data =
            unsafe { <&'static [f64] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        crate::altrep_data::AltRealData::max(data, narm)
            .map(crate::ffi::SEXP::scalar_real)
            .unwrap_or(crate::ffi::SEXP::null())
    }
}

crate::impl_inferbase_real!(&'static [f64]);

// Logical static slices
impl crate::altrep_traits::Altrep for &'static [bool] {
    fn length(x: crate::ffi::SEXP) -> crate::ffi::R_xlen_t {
        let data = unsafe {
            <&'static [bool] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x)
        };
        crate::altrep_data::AltrepLen::len(data) as crate::ffi::R_xlen_t
    }
}

impl crate::altrep_traits::AltVec for &'static [bool] {}

impl crate::altrep_traits::AltLogical for &'static [bool] {
    const HAS_ELT: bool = true;

    fn elt(x: crate::ffi::SEXP, i: crate::ffi::R_xlen_t) -> i32 {
        let data = unsafe {
            <&'static [bool] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x)
        };
        crate::altrep_data::AltLogicalData::elt(data, i.max(0) as usize).to_r_int()
    }

    const HAS_NO_NA: bool = true;

    fn no_na(x: crate::ffi::SEXP) -> i32 {
        let data = unsafe {
            <&'static [bool] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x)
        };
        crate::altrep_data::AltLogicalData::no_na(data)
            .map(|b| if b { 1 } else { 0 })
            .unwrap_or(0)
    }

    const HAS_SUM: bool = true;

    // ALTREP protocol: return C NULL (not R_NilValue) to signal "can't compute"
    fn sum(x: crate::ffi::SEXP, narm: bool) -> crate::ffi::SEXP {
        let data = unsafe {
            <&'static [bool] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x)
        };
        crate::altrep_data::AltLogicalData::sum(data, narm)
            .map(|s| {
                if s >= i32::MIN as i64 && s <= i32::MAX as i64 {
                    crate::ffi::SEXP::scalar_integer(s as i32)
                } else {
                    crate::ffi::SEXP::scalar_real(s as f64)
                }
            })
            .unwrap_or(crate::ffi::SEXP::null())
    }
}

crate::impl_inferbase_logical!(&'static [bool]);

// Raw static slices
impl crate::altrep_traits::Altrep for &'static [u8] {
    fn length(x: crate::ffi::SEXP) -> crate::ffi::R_xlen_t {
        let data =
            unsafe { <&'static [u8] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        crate::altrep_data::AltrepLen::len(data) as crate::ffi::R_xlen_t
    }
}

impl crate::altrep_traits::AltVec for &'static [u8] {
    const HAS_DATAPTR: bool = true;

    fn dataptr(x: crate::ffi::SEXP, writable: bool) -> *mut std::ffi::c_void {
        assert!(
            !writable,
            "cannot get writable DATAPTR for static ALTREP data"
        );
        let data =
            unsafe { <&'static [u8] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        data.as_ptr().cast::<std::ffi::c_void>().cast_mut()
    }

    const HAS_DATAPTR_OR_NULL: bool = true;

    fn dataptr_or_null(x: crate::ffi::SEXP) -> *const std::ffi::c_void {
        let data =
            unsafe { <&'static [u8] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        data.as_ptr().cast::<std::ffi::c_void>()
    }
}

impl crate::altrep_traits::AltRaw for &'static [u8] {
    const HAS_ELT: bool = true;

    fn elt(x: crate::ffi::SEXP, i: crate::ffi::R_xlen_t) -> crate::ffi::Rbyte {
        let data =
            unsafe { <&'static [u8] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        crate::altrep_data::AltRawData::elt(data, i.max(0) as usize)
    }

    const HAS_GET_REGION: bool = true;

    fn get_region(
        x: crate::ffi::SEXP,
        start: crate::ffi::R_xlen_t,
        len: crate::ffi::R_xlen_t,
        buf: &mut [u8],
    ) -> crate::ffi::R_xlen_t {
        if start < 0 || len <= 0 {
            return 0;
        }
        let data =
            unsafe { <&'static [u8] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        let len = len as usize;
        crate::altrep_data::AltRawData::get_region(data, start as usize, len, buf)
            as crate::ffi::R_xlen_t
    }
}

crate::impl_inferbase_raw!(&'static [u8]);

// String static slices (owned strings)
impl crate::altrep_traits::Altrep for &'static [String] {
    // String ALTREP elt calls Rf_mkCharLenCE (R API) — must use RUnwind.
    const GUARD: crate::altrep_traits::AltrepGuard = crate::altrep_traits::AltrepGuard::RUnwind;

    fn length(x: crate::ffi::SEXP) -> crate::ffi::R_xlen_t {
        let data = unsafe {
            <&'static [String] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x)
        };
        crate::altrep_data::AltrepLen::len(data) as crate::ffi::R_xlen_t
    }
}

impl crate::altrep_traits::AltVec for &'static [String] {}

impl crate::altrep_traits::AltString for &'static [String] {
    fn elt(x: crate::ffi::SEXP, i: crate::ffi::R_xlen_t) -> crate::ffi::SEXP {
        let data = unsafe {
            <&'static [String] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x)
        };
        match crate::altrep_data::AltStringData::elt(data, i.max(0) as usize) {
            Some(s) => unsafe { checked_mkchar(s) },
            None => crate::ffi::SEXP::na_string(),
        }
    }

    const HAS_NO_NA: bool = true;

    fn no_na(x: crate::ffi::SEXP) -> i32 {
        let data = unsafe {
            <&'static [String] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x)
        };
        crate::altrep_data::AltStringData::no_na(data)
            .map(|b| if b { 1 } else { 0 })
            .unwrap_or(0)
    }
}

crate::impl_inferbase_string!(&'static [String]);

// String static slices (str references)
impl crate::altrep_traits::Altrep for &'static [&'static str] {
    // String ALTREP elt calls Rf_mkCharLenCE (R API) — must use RUnwind.
    const GUARD: crate::altrep_traits::AltrepGuard = crate::altrep_traits::AltrepGuard::RUnwind;

    fn length(x: crate::ffi::SEXP) -> crate::ffi::R_xlen_t {
        let data = unsafe {
            <&'static [&'static str] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x)
        };
        crate::altrep_data::AltrepLen::len(data) as crate::ffi::R_xlen_t
    }
}

impl crate::altrep_traits::AltVec for &'static [&'static str] {}

impl crate::altrep_traits::AltString for &'static [&'static str] {
    fn elt(x: crate::ffi::SEXP, i: crate::ffi::R_xlen_t) -> crate::ffi::SEXP {
        let data = unsafe {
            <&'static [&'static str] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x)
        };
        match crate::altrep_data::AltStringData::elt(data, i.max(0) as usize) {
            Some(s) => unsafe { checked_mkchar(s) },
            None => crate::ffi::SEXP::na_string(),
        }
    }

    const HAS_NO_NA: bool = true;

    fn no_na(x: crate::ffi::SEXP) -> i32 {
        let data = unsafe {
            <&'static [&'static str] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x)
        };
        crate::altrep_data::AltStringData::no_na(data)
            .map(|b| if b { 1 } else { 0 })
            .unwrap_or(0)
    }
}

crate::impl_inferbase_string!(&'static [&'static str]);
// endregion

// region: RegisterAltrep implementations for builtin types
//
// These implementations provide ALTREP class registration for Vec<T>, Box<[T]>,
// and Range<T> types. They allow using these types with ALTREP via wrapper structs.
//
// Note: IntoR is NOT implemented here for Vec types because there are already
// existing IntoR implementations that copy data to R eagerly. To get ALTREP
// behavior, use wrapper structs:
//   #[miniextendr(class = "MyVec")]
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
                            CLASS_NAME.as_ptr().cast::<std::ffi::c_char>(),
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
impl_register_altrep_builtin!(Vec<Option<String>>, "Vec_Option_String");
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

// Cow types - RegisterAltrep for zero-copy borrow from R
impl_register_altrep_builtin!(std::borrow::Cow<'static, [i32]>, "Cow_i32");
impl_register_altrep_builtin!(std::borrow::Cow<'static, [f64]>, "Cow_f64");
impl_register_altrep_builtin!(std::borrow::Cow<'static, [u8]>, "Cow_u8");
impl_register_altrep_builtin!(
    std::borrow::Cow<'static, [crate::ffi::Rcomplex]>,
    "Cow_Rcomplex"
);

// Cow string vector types
impl_register_altrep_builtin!(Vec<std::borrow::Cow<'static, str>>, "Vec_Cow_str");
impl_register_altrep_builtin!(
    Vec<Option<std::borrow::Cow<'static, str>>>,
    "Vec_Option_Cow_str"
);
// endregion
