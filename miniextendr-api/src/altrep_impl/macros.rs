//! Declarative macros generating ALTREP trait implementations.
//!
//! These `#[macro_export]` macros are the public surface used by the
//! `#[derive(Altrep*)]` proc-macros and by hand-rolled `optionals/*` impls.
//! They expand to `impl Altrep`, `impl AltVec`, `impl Alt<Family>`, and the
//! per-family `InferBase` impl for a given type.
//!
//! Each per-family `impl_alt<family>_from_data!` macro takes a type plus an
//! optional knob list of `dataptr` / `serialize` / `subset` (in canonical
//! alphabetical order). The default arm (no knobs) is the materialising
//! path — `__impl_altvec_<family>_dataptr!` provides a trivial
//! `AltrepDataptr<T>` returning `None`, falling through to data2
//! materialisation.

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
        $crate::__impl_altrep_base!($ty, with_serialize);
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
    ($ty:ty, subset, serialize) => {
        $crate::__impl_alt_from_data!(
            $ty,
            __impl_altinteger_methods,
            impl_inferbase_integer,
            subset,
            serialize
        );
    };
}

/// Internal macro: impl Altrep with just length, optionally plus serialization.
///
/// ## Arms
///
/// ```ignore
/// __impl_altrep_base!($ty);                            // length only, default RUnwind guard
/// __impl_altrep_base!($ty, $guard);                    // length only, explicit guard
/// __impl_altrep_base!($ty, with_serialize);            // length + serialize, default guard
/// __impl_altrep_base!($ty, $guard, with_serialize);    // length + serialize, explicit guard
/// ```
///
/// The `with_serialize` flag implements both:
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
macro_rules! __impl_altrep_base {
    ($ty:ty) => {
        $crate::__impl_altrep_base!($ty, RUnwind);
    };
    ($ty:ty, with_serialize) => {
        $crate::__impl_altrep_base!($ty, RUnwind, with_serialize);
    };
    ($ty:ty, $guard:ident) => {
        impl $crate::altrep_traits::Altrep for $ty {
            const GUARD: $crate::altrep_traits::AltrepGuard =
                $crate::altrep_traits::AltrepGuard::$guard;

            fn length(x: $crate::sys::SEXP) -> $crate::sys::R_xlen_t {
                let data =
                    unsafe { <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
                <$ty as $crate::altrep_data::AltrepLen>::len(data) as $crate::sys::R_xlen_t
            }
        }
    };
    ($ty:ty, $guard:ident, with_serialize) => {
        impl $crate::altrep_traits::Altrep for $ty {
            const GUARD: $crate::altrep_traits::AltrepGuard =
                $crate::altrep_traits::AltrepGuard::$guard;

            fn length(x: $crate::sys::SEXP) -> $crate::sys::R_xlen_t {
                let data =
                    unsafe { <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
                <$ty as $crate::altrep_data::AltrepLen>::len(data) as $crate::sys::R_xlen_t
            }

            const HAS_SERIALIZED_STATE: bool = true;

            fn serialized_state(x: $crate::sys::SEXP) -> $crate::sys::SEXP {
                let data =
                    unsafe { <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
                <$ty as $crate::altrep_data::AltrepSerialize>::serialized_state(data)
            }

            const HAS_UNSERIALIZE: bool = true;

            fn unserialize(
                class: $crate::sys::SEXP,
                state: $crate::sys::SEXP,
            ) -> $crate::sys::SEXP {
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
                    use $crate::sys::altrep::R_altrep_class_t;
                    use $crate::sys::{Rf_protect_unchecked, Rf_unprotect_unchecked, SEXP};

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

            fn dataptr(x: $crate::sys::SEXP, writable: bool) -> *mut core::ffi::c_void {
                // Check data2 cache first (materialized by a prior call).
                unsafe {
                    let data2 = $crate::altrep_ext::AltrepSexpExt::altrep_data2_raw(&x);
                    if !data2.is_null()
                        && $crate::sys::SexpExt::type_of(&data2)
                            == <$elem as $crate::sys::RNativeType>::SEXP_TYPE
                    {
                        return $crate::sys::DATAPTR_RO(data2).cast_mut();
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

            fn dataptr_or_null(x: $crate::sys::SEXP) -> *const core::ffi::c_void {
                // Check data2 cache first (may have been materialized by a prior dataptr call)
                unsafe {
                    let data2 = $crate::altrep_ext::AltrepSexpExt::altrep_data2_raw(&x);
                    if !data2.is_null()
                        && $crate::sys::SexpExt::type_of(&data2)
                            == <$elem as $crate::sys::RNativeType>::SEXP_TYPE
                    {
                        return $crate::sys::DATAPTR_RO(data2);
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

            fn dataptr(x: $crate::sys::SEXP, _writable: bool) -> *mut core::ffi::c_void {
                unsafe {
                    let n = <$ty as $crate::altrep_traits::Altrep>::length(x);

                    // Get or allocate the data2 cache STRSXP
                    let mut data2 = $crate::altrep_ext::AltrepSexpExt::altrep_data2_raw(&x);
                    let fresh_alloc = data2.is_null()
                        || $crate::sys::SexpExt::type_of(&data2) != $crate::sys::SEXPTYPE::STRSXP;
                    if fresh_alloc {
                        // Rf_allocVector(STRSXP, n) leaves elements UNINITIALIZED
                        // (garbage SEXP pointers). Must fill with R_NaString sentinel
                        // so cache lookups work. This is O(n) but unavoidable.
                        //
                        // Inside ALTREP dispatch — _unchecked variants skip the
                        // with_r_thread debug-assert (MXL301 permits).
                        data2 = $crate::sys::Rf_protect_unchecked(
                            $crate::sys::Rf_allocVector_unchecked($crate::sys::SEXPTYPE::STRSXP, n),
                        );
                        for j in 0..n {
                            $crate::sys::SexpExt::set_string_elt(
                                &data2,
                                j,
                                $crate::sys::SEXP::na_string(),
                            );
                        }
                        $crate::altrep_ext::AltrepSexpExt::set_altrep_data2(&x, data2);
                        $crate::sys::Rf_unprotect_unchecked(1);
                    }

                    // Fill uncached elements only — elements already cached by Elt
                    // are non-NA CHARSXPs and are skipped. NA elements are re-probed
                    // from Rust (O(1)) to handle mixed cached/uncached NA slots.
                    for i in 0..n {
                        let cached = $crate::sys::SexpExt::string_elt(&data2, i);
                        if cached != $crate::sys::SEXP::na_string() {
                            continue; // already cached by a prior Elt call
                        }
                        // Compute from Rust and store
                        let elt = <$ty as $crate::altrep_traits::AltString>::elt(x, i);
                        $crate::sys::SexpExt::set_string_elt(&data2, i, elt);
                    }

                    $crate::sys::DATAPTR_RO(data2).cast_mut()
                }
            }

            const HAS_DATAPTR_OR_NULL: bool = true;

            fn dataptr_or_null(x: $crate::sys::SEXP) -> *const core::ffi::c_void {
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
        impl $crate::altrep_data::AltrepDataptr<$crate::sys::RLogical> for $ty {
            fn dataptr(&mut self, _writable: bool) -> Option<*mut $crate::sys::RLogical> {
                None
            }
        }
        $crate::__impl_altvec_dataptr!($ty, $crate::sys::RLogical);
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
        impl $crate::altrep_data::AltrepDataptr<$crate::sys::Rcomplex> for $ty {
            fn dataptr(&mut self, _writable: bool) -> Option<*mut $crate::sys::Rcomplex> {
                None
            }
        }
        $crate::__impl_altvec_dataptr!($ty, $crate::sys::Rcomplex);
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
                x: $crate::sys::SEXP,
                indx: $crate::sys::SEXP,
                _call: $crate::sys::SEXP,
            ) -> $crate::sys::SEXP {
                // Validate that indx is an integer vector before calling INTEGER().
                // Return NULL to signal R to use default subsetting if not.
                if $crate::sys::SexpExt::type_of(&indx) != $crate::sys::SEXPTYPE::INTSXP {
                    return core::ptr::null_mut();
                }

                // Convert indx SEXP to slice using SexpExt (avoids raw-ptr-deref lint)
                let indices = unsafe { $crate::sys::SexpExt::as_slice::<i32>(&indx) };

                let data =
                    unsafe { <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
                <$ty as $crate::altrep_data::AltrepExtractSubset>::extract_subset(data, indices)
                    .unwrap_or($crate::sys::SEXP::nil())
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

        fn elt(x: $crate::sys::SEXP, i: $crate::sys::R_xlen_t) -> $elem {
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
            x: $crate::sys::SEXP,
            start: $crate::sys::R_xlen_t,
            len: $crate::sys::R_xlen_t,
            buf: &mut [$buf_ty],
        ) -> $crate::sys::R_xlen_t {
            if start < 0 || len <= 0 {
                return 0;
            }
            let data =
                unsafe { <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
            let len = len as usize;
            <$ty as $trait>::get_region(data, start as usize, len, buf) as $crate::sys::R_xlen_t
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

        fn is_sorted(x: $crate::sys::SEXP) -> i32 {
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

        fn no_na(x: $crate::sys::SEXP) -> i32 {
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
        $crate::__impl_altrep_base!($ty, with_serialize);
        impl $crate::altrep_traits::AltVec for $ty {}
        $crate::$methods!($ty);
        $crate::$inferbase!($ty);
    };
    // Serialize + explicit guard
    ($ty:ty, $methods:ident, $inferbase:ident, serialize, @guard $guard:ident) => {
        $crate::__impl_altrep_base!($ty, $guard, with_serialize);
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
        $crate::__impl_altrep_base!($ty, with_serialize);
        $crate::__impl_altvec_dataptr!($ty, $elem);
        $crate::$methods!($ty);
        $crate::$inferbase!($ty);
    };
    // String dataptr + serialize
    ($ty:ty, $methods:ident, $inferbase:ident, string_dataptr, serialize) => {
        $crate::__impl_altrep_base!($ty, with_serialize);
        $crate::__impl_altvec_string_dataptr!($ty);
        $crate::$methods!($ty);
        $crate::$inferbase!($ty);
    };
    // String dataptr + serialize + explicit guard
    ($ty:ty, $methods:ident, $inferbase:ident, string_dataptr, serialize, @guard $guard:ident) => {
        $crate::__impl_altrep_base!($ty, $guard, with_serialize);
        $crate::__impl_altvec_string_dataptr!($ty);
        $crate::$methods!($ty);
        $crate::$inferbase!($ty);
    };
    // Subset + serialize
    ($ty:ty, $methods:ident, $inferbase:ident, subset, serialize) => {
        $crate::__impl_altrep_base!($ty, with_serialize);
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
            fn sum(x: $crate::sys::SEXP, narm: bool) -> $crate::sys::SEXP {
                let data =
                    unsafe { <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
                match <$ty as $crate::altrep_data::AltIntegerData>::sum(data, narm) {
                    Some(s) => {
                        if s >= i32::MIN as i64 && s <= i32::MAX as i64 {
                            $crate::sys::SEXP::scalar_integer(s as i32)
                        } else {
                            $crate::sys::SEXP::scalar_real(s as f64)
                        }
                    }
                    None => $crate::sys::SEXP::null(),
                }
            }

            const HAS_MIN: bool = true;

            fn min(x: $crate::sys::SEXP, narm: bool) -> $crate::sys::SEXP {
                let data =
                    unsafe { <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
                <$ty as $crate::altrep_data::AltIntegerData>::min(data, narm)
                    .map(|m| $crate::sys::SEXP::scalar_integer(m))
                    .unwrap_or($crate::sys::SEXP::null())
            }

            const HAS_MAX: bool = true;

            fn max(x: $crate::sys::SEXP, narm: bool) -> $crate::sys::SEXP {
                let data =
                    unsafe { <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
                <$ty as $crate::altrep_data::AltIntegerData>::max(data, narm)
                    .map(|m| $crate::sys::SEXP::scalar_integer(m))
                    .unwrap_or($crate::sys::SEXP::null())
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
        $crate::__impl_altrep_base!($ty, with_serialize);
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
            fn sum(x: $crate::sys::SEXP, narm: bool) -> $crate::sys::SEXP {
                let data =
                    unsafe { <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
                <$ty as $crate::altrep_data::AltRealData>::sum(data, narm)
                    .map(|s| $crate::sys::SEXP::scalar_real(s))
                    .unwrap_or($crate::sys::SEXP::null())
            }

            const HAS_MIN: bool = true;

            fn min(x: $crate::sys::SEXP, narm: bool) -> $crate::sys::SEXP {
                let data =
                    unsafe { <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
                <$ty as $crate::altrep_data::AltRealData>::min(data, narm)
                    .map(|m| $crate::sys::SEXP::scalar_real(m))
                    .unwrap_or($crate::sys::SEXP::null())
            }

            const HAS_MAX: bool = true;

            fn max(x: $crate::sys::SEXP, narm: bool) -> $crate::sys::SEXP {
                let data =
                    unsafe { <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
                <$ty as $crate::altrep_data::AltRealData>::max(data, narm)
                    .map(|m| $crate::sys::SEXP::scalar_real(m))
                    .unwrap_or($crate::sys::SEXP::null())
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
        $crate::__impl_altrep_base!($ty, with_serialize);
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
}

/// Internal macro: impl AltLogical methods from AltLogicalData
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_altlogical_methods {
    ($ty:ty) => {
        impl $crate::altrep_traits::AltLogical for $ty {
            // Logical elt is special: returns Logical → .to_r_int()
            const HAS_ELT: bool = true;

            fn elt(x: $crate::sys::SEXP, i: $crate::sys::R_xlen_t) -> i32 {
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
            fn sum(x: $crate::sys::SEXP, narm: bool) -> $crate::sys::SEXP {
                let data =
                    unsafe { <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
                match <$ty as $crate::altrep_data::AltLogicalData>::sum(data, narm) {
                    Some(s) => {
                        if s >= i32::MIN as i64 && s <= i32::MAX as i64 {
                            $crate::sys::SEXP::scalar_integer(s as i32)
                        } else {
                            $crate::sys::SEXP::scalar_real(s as f64)
                        }
                    }
                    None => $crate::sys::SEXP::null(),
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
        $crate::__impl_altrep_base!($ty, with_serialize);
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
        $crate::__impl_altrep_base!($ty, with_serialize);
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
            fn elt(x: $crate::sys::SEXP, i: $crate::sys::R_xlen_t) -> $crate::sys::SEXP {
                unsafe {
                    let idx = i.max(0) as usize;

                    // Get or allocate the data2 cache STRSXP
                    let mut data2 = $crate::altrep_ext::AltrepSexpExt::altrep_data2_raw(&x);
                    if data2.is_null()
                        || $crate::sys::SexpExt::type_of(&data2) != $crate::sys::SEXPTYPE::STRSXP
                    {
                        let n = <$ty as $crate::altrep_traits::Altrep>::length(x);
                        // Rf_allocVector(STRSXP, n) leaves elements UNINITIALIZED
                        // (garbage SEXP pointers). Must fill with R_NaString sentinel.
                        //
                        // Inside ALTREP dispatch — _unchecked variants skip the
                        // with_r_thread debug-assert (MXL301 permits).
                        data2 = $crate::sys::Rf_protect_unchecked(
                            $crate::sys::Rf_allocVector_unchecked($crate::sys::SEXPTYPE::STRSXP, n),
                        );
                        for j in 0..n {
                            $crate::sys::SexpExt::set_string_elt(
                                &data2,
                                j,
                                $crate::sys::SEXP::na_string(),
                            );
                        }
                        $crate::altrep_ext::AltrepSexpExt::set_altrep_data2(&x, data2);
                        $crate::sys::Rf_unprotect_unchecked(1);
                    }

                    // Check cache: non-NA means already materialized
                    let cached = $crate::sys::SexpExt::string_elt(&data2, i);
                    if cached != $crate::sys::SEXP::na_string() {
                        return cached;
                    }

                    // Cache miss (or genuine NA) — probe Rust source
                    let data = <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x);
                    match <$ty as $crate::altrep_data::AltStringData>::elt(data, idx) {
                        Some(s) => {
                            let charsxp = $crate::altrep_impl::checked_mkchar(s);
                            $crate::sys::SexpExt::set_string_elt(&data2, i, charsxp);
                            charsxp
                        }
                        None => $crate::sys::SEXP::na_string(),
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

            fn length(x: $crate::sys::SEXP) -> $crate::sys::R_xlen_t {
                let data =
                    unsafe { <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
                <$ty as $crate::altrep_data::AltrepLen>::len(data) as $crate::sys::R_xlen_t
            }
        }

        impl $crate::altrep_traits::AltVec for $ty {}

        impl $crate::altrep_traits::AltList for $ty {
            fn elt(x: $crate::sys::SEXP, i: $crate::sys::R_xlen_t) -> $crate::sys::SEXP {
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
                $crate::sys::Rcomplex,
                $crate::sys::Rcomplex {
                    r: f64::NAN,
                    i: f64::NAN
                }
            );
            $crate::__impl_alt_get_region!(
                $ty,
                $crate::altrep_data::AltComplexData,
                $crate::sys::Rcomplex
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
            dataptr($crate::sys::Rcomplex)
        );
    };
    ($ty:ty, serialize) => {
        $crate::__impl_altrep_base!($ty, with_serialize);
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
            dataptr($crate::sys::Rcomplex),
            serialize
        );
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
}
// endregion
