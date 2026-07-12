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
//!
//! Each family also has an `impl_alt<family>_from_data_generic!` sibling
//! accepting a brace-delimited generic parameter list and where-clause
//! (`{$gen} $ty {$where}[, $knob]*`) for generic types like
//! `struct Foo<T> { .. }`. The plain macros forward to their `_generic!`
//! sibling with empty `{}` brackets, so each family has exactly one emission
//! body regardless of which form is called.
//!
//! Note the iterator/stream data adaptors in `altrep_data::iter` /
//! `altrep_data::stream` deliberately do **not** invoke these macros: they
//! implement only the data-level traits and are meant to be wrapped by a
//! concrete `#[derive(Altrep*)]` + `#[altrep(manual)]` struct (#1146).

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
    // Default (no knobs): materializing DATAPTR — allocates INTSXP in data2 on
    // first DATAPTR call. Without this, R's default DATAPTR errors with
    // "cannot access data pointer". See `__impl_alt_family!` for the knob matrix.
    ($ty:ty $(, $knob:ident)*) => {
        $crate::impl_altinteger_from_data_generic!({} $ty {} $(, $knob)*);
    };
}

/// Generic form of [`impl_altinteger_from_data!`]: accepts an optional
/// generic parameter list and where-clause (`{T, U} Foo<T, U> {T: Bound, ..}`)
/// so it can target `struct Foo<T> { .. }` types, not just
/// concrete/monomorphic ones. The non-generic macro above forwards here with
/// empty `{}` brackets so there is exactly one emission body for both call
/// shapes.
#[macro_export]
macro_rules! impl_altinteger_from_data_generic {
    ({$($gen:tt)*} $ty:ty {$($whr:tt)*} $(, $knob:ident)*) => {
        $crate::__impl_alt_family!(
            {$($gen)*} $ty {$($whr)*},
            __impl_altinteger_methods,
            impl_inferbase_integer,
            dataptr: dataptr(i32),
            default: materializing(i32)
            $(, $knob)*
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
    // Generic form: `{$gen} $ty {$where}, $guard[, with_serialize]`. Brace
    // (not bracket/paren) delimiters are deliberate: `{` can never start a
    // `$ty:ty` fragment, so a bare-`$ty:ty` arm asked to match a brace-led
    // invocation fails cleanly instead of committing to (and hard-erroring
    // on) a bogus type parse — `[]`/`()` are themselves valid (empty
    // slice-like / unit) type starts and would NOT fail cleanly here. The
    // non-generic arms above forward to this one with empty `{}` brackets so
    // there is exactly one emission body per (guard, serialize) combination.
    ({$($gen:tt)*} $ty:ty {$($whr:tt)*}, $guard:ident) => {
        impl<$($gen)*> $crate::altrep_traits::Altrep for $ty where $($whr)* {
            const GUARD: $crate::altrep_traits::AltrepGuard =
                $crate::altrep_traits::AltrepGuard::$guard;

            fn length(x: $crate::SEXP) -> $crate::R_xlen_t {
                let data =
                    unsafe { <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
                <$ty as $crate::altrep_data::AltrepLen>::len(data) as $crate::R_xlen_t
            }
        }
    };
    ({$($gen:tt)*} $ty:ty {$($whr:tt)*}, $guard:ident, with_serialize) => {
        impl<$($gen)*> $crate::altrep_traits::Altrep for $ty where $($whr)* {
            const GUARD: $crate::altrep_traits::AltrepGuard =
                $crate::altrep_traits::AltrepGuard::$guard;

            fn length(x: $crate::SEXP) -> $crate::R_xlen_t {
                let data =
                    unsafe { <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
                <$ty as $crate::altrep_data::AltrepLen>::len(data) as $crate::R_xlen_t
            }

            const HAS_SERIALIZED_STATE: bool = true;

            fn serialized_state(x: $crate::SEXP) -> $crate::SEXP {
                let data =
                    unsafe { <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
                <$ty as $crate::altrep_data::AltrepSerialize>::serialized_state(data)
            }

            const HAS_UNSERIALIZE: bool = true;

            fn unserialize(class: $crate::SEXP, state: $crate::SEXP) -> $crate::SEXP {
                let Some(data) = <$ty as $crate::altrep_data::AltrepSerialize>::unserialize(state)
                else {
                    panic!(
                        "ALTREP unserialize failed for {}",
                        core::any::type_name::<$ty>()
                    );
                };

                // SAFETY: Unserialize is called by R on the main thread.
                unsafe {
                    use $crate::SEXP;
                    use $crate::externalptr::ExternalPtr;
                    use $crate::sys::altrep::R_altrep_class_t;
                    use $crate::sys::{Rf_protect_unchecked, Rf_unprotect_unchecked};

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
    // Bare-`$ty:ty` forwarders (concrete/monomorphic types) — listed AFTER
    // the brace-generic arms above so a genuinely bare call (e.g. from the
    // `#[derive(Altrep*)]` proc-macro, which always targets concrete types)
    // still resolves here rather than being intercepted by the brace arms
    // (which require a literal leading `{...}` group and so never match a
    // bare invocation).
    ($ty:ty, $guard:ident) => {
        $crate::__impl_altrep_base!({} $ty {}, $guard);
    };
    ($ty:ty, $guard:ident, with_serialize) => {
        $crate::__impl_altrep_base!({} $ty {}, $guard, with_serialize);
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
        $crate::__impl_altvec_dataptr!({} $ty {}, $elem);
    };
    // Generic form: `[$gen] $ty [$where], $elem`. See `__impl_altrep_base!`
    // for the empty-brackets delegation pattern.
    ({$($gen:tt)*} $ty:ty {$($whr:tt)*}, $elem:ty) => {
        impl<$($gen)*> $crate::altrep_traits::AltVec for $ty where $($whr)* {
            const HAS_DATAPTR: bool = true;

            fn dataptr(x: $crate::SEXP, writable: bool) -> *mut core::ffi::c_void {
                // Check data2 cache first (materialized by a prior call).
                unsafe {
                    let data2 = $crate::altrep_ext::AltrepSexpExt::altrep_data2_raw(&x);
                    if !data2.is_null()
                        && $crate::SexpExt::type_of(&data2)
                            == <$elem as $crate::RNativeType>::SEXP_TYPE
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

            fn dataptr_or_null(x: $crate::SEXP) -> *const core::ffi::c_void {
                // Check data2 cache first (may have been materialized by a prior dataptr call)
                unsafe {
                    let data2 = $crate::altrep_ext::AltrepSexpExt::altrep_data2_raw(&x);
                    if !data2.is_null()
                        && $crate::SexpExt::type_of(&data2)
                            == <$elem as $crate::RNativeType>::SEXP_TYPE
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
        $crate::__impl_altvec_string_dataptr!({} $ty {});
    };
    // Generic form: `[$gen] $ty [$where]`.
    ({$($gen:tt)*} $ty:ty {$($whr:tt)*}) => {
        impl<$($gen)*> $crate::altrep_traits::AltVec for $ty where $($whr)* {
            const HAS_DATAPTR: bool = true;

            fn dataptr(x: $crate::SEXP, _writable: bool) -> *mut core::ffi::c_void {
                unsafe {
                    let n = <$ty as $crate::altrep_traits::Altrep>::length(x);

                    // Get or allocate the data2 cache STRSXP
                    let mut data2 = $crate::altrep_ext::AltrepSexpExt::altrep_data2_raw(&x);
                    let fresh_alloc = data2.is_null()
                        || $crate::SexpExt::type_of(&data2) != $crate::SEXPTYPE::STRSXP;
                    if fresh_alloc {
                        // Rf_allocVector(STRSXP, n) leaves elements UNINITIALIZED
                        // (garbage SEXP pointers). Must fill with R_NaString sentinel
                        // so cache lookups work. This is O(n) but unavoidable.
                        //
                        // Inside ALTREP dispatch — _unchecked variants skip the
                        // with_r_thread debug-assert (MXL301 permits).
                        data2 = $crate::sys::Rf_protect_unchecked(
                            $crate::sys::Rf_allocVector_unchecked($crate::SEXPTYPE::STRSXP, n),
                        );
                        for j in 0..n {
                            $crate::SexpExt::set_string_elt(&data2, j, $crate::SEXP::na_string());
                        }
                        $crate::altrep_ext::AltrepSexpExt::set_altrep_data2(&x, data2);
                        $crate::sys::Rf_unprotect_unchecked(1);
                    }

                    // Fill uncached elements only — elements already cached by Elt
                    // are non-NA CHARSXPs and are skipped. NA elements are re-probed
                    // from Rust (O(1)) to handle mixed cached/uncached NA slots.
                    for i in 0..n {
                        let cached = $crate::SexpExt::string_elt(&data2, i);
                        if cached != $crate::SEXP::na_string() {
                            continue; // already cached by a prior Elt call
                        }
                        // Compute from Rust and store
                        let elt = <$ty as $crate::altrep_traits::AltString>::elt(x, i);
                        $crate::SexpExt::set_string_elt(&data2, i, elt);
                    }

                    $crate::sys::DATAPTR_RO(data2).cast_mut()
                }
            }

            const HAS_DATAPTR_OR_NULL: bool = true;

            fn dataptr_or_null(x: $crate::SEXP) -> *const core::ffi::c_void {
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

/// Internal macro: impl AltVec with a *materializing* dataptr for a given element type.
///
/// Thin wrapper, parameterised by element type: installs a trivial
/// `AltrepDataptr<$elem>` (no direct pointer — `dataptr` returns `None`) and
/// delegates to [`__impl_altvec_dataptr`], which materializes into `data2` via
/// `RNativeType::elt`. The 5 per-family aliases below pin `$elem` so the derive
/// and the public `impl_alt*_from_data!` macros can reference them by a stable
/// per-family name.
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_altvec_materializing_dataptr {
    ($ty:ty, $elem:ty) => {
        $crate::__impl_altvec_materializing_dataptr!({} $ty {}, $elem);
    };
    // Generic form: `[$gen] $ty [$where], $elem`.
    ({$($gen:tt)*} $ty:ty {$($whr:tt)*}, $elem:ty) => {
        impl<$($gen)*> $crate::altrep_data::AltrepDataptr<$elem> for $ty where $($whr)* {
            fn dataptr(&mut self, _writable: bool) -> Option<*mut $elem> {
                None
            }
        }
        $crate::__impl_altvec_dataptr!({$($gen)*} $ty {$($whr)*}, $elem);
    };
}

/// Internal macro: materializing dataptr for logical ALTREP.
///
/// R logicals are stored as `i32` but accessed through `RLogical` for type safety.
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_altvec_logical_dataptr {
    ($ty:ty) => {
        $crate::__impl_altvec_materializing_dataptr!($ty, $crate::RLogical);
    };
}

/// Internal macro: materializing dataptr for integer ALTREP.
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_altvec_integer_dataptr {
    ($ty:ty) => {
        $crate::__impl_altvec_materializing_dataptr!($ty, i32);
    };
}

/// Internal macro: materializing dataptr for real ALTREP.
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_altvec_real_dataptr {
    ($ty:ty) => {
        $crate::__impl_altvec_materializing_dataptr!($ty, f64);
    };
}

/// Internal macro: materializing dataptr for raw ALTREP.
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_altvec_raw_dataptr {
    ($ty:ty) => {
        $crate::__impl_altvec_materializing_dataptr!($ty, u8);
    };
}

/// Internal macro: materializing dataptr for complex ALTREP.
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_altvec_complex_dataptr {
    ($ty:ty) => {
        $crate::__impl_altvec_materializing_dataptr!($ty, $crate::Rcomplex);
    };
}

/// Internal macro: impl AltVec with extract_subset support
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_altvec_extract_subset {
    ($ty:ty) => {
        $crate::__impl_altvec_extract_subset!({} $ty {});
    };
    // Generic form: `[$gen] $ty [$where]`.
    ({$($gen:tt)*} $ty:ty {$($whr:tt)*}) => {
        impl<$($gen)*> $crate::altrep_traits::AltVec for $ty where $($whr)* {
            const HAS_EXTRACT_SUBSET: bool = true;

            fn extract_subset(
                x: $crate::SEXP,
                indx: $crate::SEXP,
                _call: $crate::SEXP,
            ) -> $crate::SEXP {
                // Validate that indx is an integer vector before calling INTEGER().
                // Return C NULL (not R_NilValue) to signal R to use default
                // subsetting if not — R checks `!= NULL` here.
                if $crate::SexpExt::type_of(&indx) != $crate::SEXPTYPE::INTSXP {
                    return $crate::SEXP::null();
                }

                // Convert indx SEXP to slice using SexpExt (avoids raw-ptr-deref lint)
                let indices = unsafe { $crate::SexpExt::as_slice::<i32>(&indx) };

                let data =
                    unsafe { <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
                <$ty as $crate::altrep_data::AltrepExtractSubset>::extract_subset(data, indices)
                    .unwrap_or($crate::SEXP::nil())
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

        fn elt(x: $crate::SEXP, i: $crate::R_xlen_t) -> $elem {
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
            x: $crate::SEXP,
            start: $crate::R_xlen_t,
            len: $crate::R_xlen_t,
            buf: &mut [$buf_ty],
        ) -> $crate::R_xlen_t {
            if start < 0 || len <= 0 {
                return 0;
            }
            let data =
                unsafe { <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
            let len = len as usize;
            <$ty as $trait>::get_region(data, start as usize, len, buf) as $crate::R_xlen_t
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

        fn is_sorted(x: $crate::SEXP) -> i32 {
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

        fn no_na(x: $crate::SEXP) -> i32 {
            let data =
                unsafe { <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
            <$ty as $trait>::no_na(data)
                .map(|b| if b { 1 } else { 0 })
                .unwrap_or(0)
        }
    };
}
// endregion

// region: Canonical emission macros: __impl_altvec_flavor! / __impl_alt_from_data! / __impl_alt_family!
//
// Three layers, each with a single responsibility:
//
// 1. `__impl_altvec_flavor!` — maps an AltVec *flavor* token to the macro
//    that emits the `impl AltVec` (typed direct pointer, materializing,
//    STRSXP materialization, or extract_subset).
// 2. `__impl_alt_from_data!` — the canonical emitter: Altrep base
//    (with/without serialize) + AltVec flavor + family methods + InferBase.
//    Exactly two arms — every knob combination reduces to these.
// 3. `__impl_alt_family!` — the knob matrix: maps the public macros'
//    user-facing knob spellings (`dataptr` / `serialize` / `subset`, in
//    canonical order) to a flavor + optional serialize. Parameterised by the
//    family's `dataptr:` and `default:` flavors so one matrix serves all
//    six knob-bearing families.

/// Internal macro: emit the `impl AltVec` for a given flavor.
///
/// Flavors:
/// - `dataptr($elem)` — typed direct pointer via `AltrepDataptr<$elem>`,
///   falling back to data2 materialization.
/// - `materializing($elem)` — trivial `AltrepDataptr<$elem>` returning `None`
///   plus the same `__impl_altvec_dataptr!` (pure data2 materialization).
/// - `string_dataptr` — whole-vector STRSXP materialization (string family).
/// - `subset` — `Extract_subset` support via `AltrepExtractSubset`.
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_altvec_flavor {
    ($ty:ty, dataptr($elem:ty)) => {
        $crate::__impl_altvec_dataptr!($ty, $elem);
    };
    ($ty:ty, materializing($elem:ty)) => {
        $crate::__impl_altvec_materializing_dataptr!($ty, $elem);
    };
    ($ty:ty, string_dataptr) => {
        $crate::__impl_altvec_string_dataptr!($ty);
    };
    ($ty:ty, subset) => {
        $crate::__impl_altvec_extract_subset!($ty);
    };
    // Generic form: `[$gen] $ty [$where], $flavor`.
    ({$($gen:tt)*} $ty:ty {$($whr:tt)*}, dataptr($elem:ty)) => {
        $crate::__impl_altvec_dataptr!({$($gen)*} $ty {$($whr)*}, $elem);
    };
    ({$($gen:tt)*} $ty:ty {$($whr:tt)*}, materializing($elem:ty)) => {
        $crate::__impl_altvec_materializing_dataptr!({$($gen)*} $ty {$($whr)*}, $elem);
    };
    ({$($gen:tt)*} $ty:ty {$($whr:tt)*}, string_dataptr) => {
        $crate::__impl_altvec_string_dataptr!({$($gen)*} $ty {$($whr)*});
    };
    ({$($gen:tt)*} $ty:ty {$($whr:tt)*}, subset) => {
        $crate::__impl_altvec_extract_subset!({$($gen)*} $ty {$($whr)*});
    };
}

/// Internal macro: canonical ALTREP emission.
///
/// Generates the standard ALTREP trait implementations (Altrep base, AltVec
/// flavor, family-specific methods, InferBase) for a given type. The two arms
/// differ only in serialization support on the Altrep base impl.
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_alt_from_data {
    ($ty:ty, $methods:ident, $inferbase:ident, $flavor:ident $(($elem:ty))?) => {
        $crate::__impl_altrep_base!($ty);
        $crate::__impl_altvec_flavor!($ty, $flavor $(($elem))?);
        $crate::$methods!($ty);
        $crate::$inferbase!($ty);
    };
    ($ty:ty, $methods:ident, $inferbase:ident, $flavor:ident $(($elem:ty))?, serialize) => {
        $crate::__impl_altrep_base!($ty, with_serialize);
        $crate::__impl_altvec_flavor!($ty, $flavor $(($elem))?);
        $crate::$methods!($ty);
        $crate::$inferbase!($ty);
    };
    // Generic form: `[$gen] $ty [$where], $methods, $inferbase, $flavor`.
    ({$($gen:tt)*} $ty:ty {$($whr:tt)*}, $methods:ident, $inferbase:ident, $flavor:ident $(($elem:ty))?) => {
        $crate::__impl_altrep_base!({$($gen)*} $ty {$($whr)*}, RUnwind);
        $crate::__impl_altvec_flavor!({$($gen)*} $ty {$($whr)*}, $flavor $(($elem))?);
        $crate::$methods!({$($gen)*} $ty {$($whr)*});
        $crate::$inferbase!({$($gen)*} $ty {$($whr)*});
    };
    ({$($gen:tt)*} $ty:ty {$($whr:tt)*}, $methods:ident, $inferbase:ident, $flavor:ident $(($elem:ty))?, serialize) => {
        $crate::__impl_altrep_base!({$($gen)*} $ty {$($whr)*}, RUnwind, with_serialize);
        $crate::__impl_altvec_flavor!({$($gen)*} $ty {$($whr)*}, $flavor $(($elem))?);
        $crate::$methods!({$($gen)*} $ty {$($whr)*});
        $crate::$inferbase!({$($gen)*} $ty {$($whr)*});
    };
}

/// Internal macro: the per-family knob matrix.
///
/// Maps the user-facing knob list of the public `impl_alt*_from_data!` macros
/// (`dataptr` / `serialize` / `subset`, canonical order) to a canonical
/// [`__impl_alt_from_data!`] invocation. The family supplies:
/// - `dataptr:` — the AltVec flavor for the `dataptr` knob,
/// - `default:` — the AltVec flavor when no `dataptr`/`subset` knob is given
///   (the materializing/data2 path).
///
/// Valid knob combinations (anything else is a compile error):
///
/// ```ignore
/// ()                    // default flavor
/// (dataptr)             // direct-pointer flavor
/// (serialize)           // default flavor + serialize
/// (subset)              // extract_subset flavor
/// (dataptr, serialize)
/// (subset, serialize)
/// ```
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_alt_family {
    ($ty:ty, $methods:ident, $inferbase:ident,
     dataptr: $dpf:ident $(($dpe:ty))?, default: $dff:ident $(($dfe:ty))?) => {
        $crate::__impl_alt_from_data!($ty, $methods, $inferbase, $dff $(($dfe))?);
    };
    ($ty:ty, $methods:ident, $inferbase:ident,
     dataptr: $dpf:ident $(($dpe:ty))?, default: $dff:ident $(($dfe:ty))?, dataptr) => {
        $crate::__impl_alt_from_data!($ty, $methods, $inferbase, $dpf $(($dpe))?);
    };
    ($ty:ty, $methods:ident, $inferbase:ident,
     dataptr: $dpf:ident $(($dpe:ty))?, default: $dff:ident $(($dfe:ty))?, serialize) => {
        $crate::__impl_alt_from_data!($ty, $methods, $inferbase, $dff $(($dfe))?, serialize);
    };
    ($ty:ty, $methods:ident, $inferbase:ident,
     dataptr: $dpf:ident $(($dpe:ty))?, default: $dff:ident $(($dfe:ty))?, subset) => {
        $crate::__impl_alt_from_data!($ty, $methods, $inferbase, subset);
    };
    ($ty:ty, $methods:ident, $inferbase:ident,
     dataptr: $dpf:ident $(($dpe:ty))?, default: $dff:ident $(($dfe:ty))?, dataptr, serialize) => {
        $crate::__impl_alt_from_data!($ty, $methods, $inferbase, $dpf $(($dpe))?, serialize);
    };
    ($ty:ty, $methods:ident, $inferbase:ident,
     dataptr: $dpf:ident $(($dpe:ty))?, default: $dff:ident $(($dfe:ty))?, subset, serialize) => {
        $crate::__impl_alt_from_data!($ty, $methods, $inferbase, subset, serialize);
    };
    // Generic form: `[$gen] $ty [$where], $methods, $inferbase, dataptr: .., default: ..[, knobs]`.
    ({$($gen:tt)*} $ty:ty {$($whr:tt)*}, $methods:ident, $inferbase:ident,
     dataptr: $dpf:ident $(($dpe:ty))?, default: $dff:ident $(($dfe:ty))?) => {
        $crate::__impl_alt_from_data!({$($gen)*} $ty {$($whr)*}, $methods, $inferbase, $dff $(($dfe))?);
    };
    ({$($gen:tt)*} $ty:ty {$($whr:tt)*}, $methods:ident, $inferbase:ident,
     dataptr: $dpf:ident $(($dpe:ty))?, default: $dff:ident $(($dfe:ty))?, dataptr) => {
        $crate::__impl_alt_from_data!({$($gen)*} $ty {$($whr)*}, $methods, $inferbase, $dpf $(($dpe))?);
    };
    ({$($gen:tt)*} $ty:ty {$($whr:tt)*}, $methods:ident, $inferbase:ident,
     dataptr: $dpf:ident $(($dpe:ty))?, default: $dff:ident $(($dfe:ty))?, serialize) => {
        $crate::__impl_alt_from_data!({$($gen)*} $ty {$($whr)*}, $methods, $inferbase, $dff $(($dfe))?, serialize);
    };
    ({$($gen:tt)*} $ty:ty {$($whr:tt)*}, $methods:ident, $inferbase:ident,
     dataptr: $dpf:ident $(($dpe:ty))?, default: $dff:ident $(($dfe:ty))?, subset) => {
        $crate::__impl_alt_from_data!({$($gen)*} $ty {$($whr)*}, $methods, $inferbase, subset);
    };
    ({$($gen:tt)*} $ty:ty {$($whr:tt)*}, $methods:ident, $inferbase:ident,
     dataptr: $dpf:ident $(($dpe:ty))?, default: $dff:ident $(($dfe:ty))?, dataptr, serialize) => {
        $crate::__impl_alt_from_data!({$($gen)*} $ty {$($whr)*}, $methods, $inferbase, $dpf $(($dpe))?, serialize);
    };
    ({$($gen:tt)*} $ty:ty {$($whr:tt)*}, $methods:ident, $inferbase:ident,
     dataptr: $dpf:ident $(($dpe:ty))?, default: $dff:ident $(($dfe:ty))?, subset, serialize) => {
        $crate::__impl_alt_from_data!({$($gen)*} $ty {$($whr)*}, $methods, $inferbase, subset, serialize);
    };
}
// endregion

// region: Per-family method macros (using shared building blocks)

/// Internal macro for AltInteger method implementations.
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_altinteger_methods {
    ($ty:ty) => {
        $crate::__impl_altinteger_methods!({} $ty {});
    };
    // Generic form: `[$gen] $ty [$where]`.
    ({$($gen:tt)*} $ty:ty {$($whr:tt)*}) => {
        impl<$($gen)*> $crate::altrep_traits::AltInteger for $ty where $($whr)* {
            $crate::__impl_alt_elt!($ty, $crate::altrep_data::AltIntegerData, i32, i32::MIN);
            $crate::__impl_alt_get_region!($ty, $crate::altrep_data::AltIntegerData, i32);
            $crate::__impl_alt_is_sorted!($ty, $crate::altrep_data::AltIntegerData);
            $crate::__impl_alt_no_na!($ty, $crate::altrep_data::AltIntegerData);

            const HAS_SUM: bool = true;

            // ALTREP protocol: return C NULL (not R_NilValue) to signal "can't compute"
            fn sum(x: $crate::SEXP, narm: bool) -> $crate::SEXP {
                let data =
                    unsafe { <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
                match <$ty as $crate::altrep_data::AltIntegerData>::sum(data, narm) {
                    Some(s) => {
                        if s >= i32::MIN as i64 && s <= i32::MAX as i64 {
                            $crate::SEXP::scalar_integer(s as i32)
                        } else {
                            $crate::SEXP::scalar_real(s as f64)
                        }
                    }
                    None => $crate::SEXP::null(),
                }
            }

            const HAS_MIN: bool = true;

            fn min(x: $crate::SEXP, narm: bool) -> $crate::SEXP {
                let data =
                    unsafe { <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
                <$ty as $crate::altrep_data::AltIntegerData>::min(data, narm)
                    .map(|m| $crate::SEXP::scalar_integer(m))
                    .unwrap_or($crate::SEXP::null())
            }

            const HAS_MAX: bool = true;

            fn max(x: $crate::SEXP, narm: bool) -> $crate::SEXP {
                let data =
                    unsafe { <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
                <$ty as $crate::altrep_data::AltIntegerData>::max(data, narm)
                    .map(|m| $crate::SEXP::scalar_integer(m))
                    .unwrap_or($crate::SEXP::null())
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
    ($ty:ty $(, $knob:ident)*) => {
        $crate::impl_altreal_from_data_generic!({} $ty {} $(, $knob)*);
    };
}

/// Generic form of [`impl_altreal_from_data!`] — see
/// [`impl_altinteger_from_data_generic!`] for the calling convention.
#[macro_export]
macro_rules! impl_altreal_from_data_generic {
    ({$($gen:tt)*} $ty:ty {$($whr:tt)*} $(, $knob:ident)*) => {
        $crate::__impl_alt_family!(
            {$($gen)*} $ty {$($whr)*},
            __impl_altreal_methods,
            impl_inferbase_real,
            dataptr: dataptr(f64),
            default: materializing(f64)
            $(, $knob)*
        );
    };
}

/// Internal macro for AltReal method implementations.
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_altreal_methods {
    ($ty:ty) => {
        $crate::__impl_altreal_methods!({} $ty {});
    };
    // Generic form: `[$gen] $ty [$where]`.
    ({$($gen:tt)*} $ty:ty {$($whr:tt)*}) => {
        impl<$($gen)*> $crate::altrep_traits::AltReal for $ty where $($whr)* {
            $crate::__impl_alt_elt!($ty, $crate::altrep_data::AltRealData, f64, f64::NAN);
            $crate::__impl_alt_get_region!($ty, $crate::altrep_data::AltRealData, f64);
            $crate::__impl_alt_is_sorted!($ty, $crate::altrep_data::AltRealData);
            $crate::__impl_alt_no_na!($ty, $crate::altrep_data::AltRealData);

            const HAS_SUM: bool = true;

            // ALTREP protocol: return C NULL (not R_NilValue) to signal "can't compute"
            fn sum(x: $crate::SEXP, narm: bool) -> $crate::SEXP {
                let data =
                    unsafe { <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
                <$ty as $crate::altrep_data::AltRealData>::sum(data, narm)
                    .map(|s| $crate::SEXP::scalar_real(s))
                    .unwrap_or($crate::SEXP::null())
            }

            const HAS_MIN: bool = true;

            fn min(x: $crate::SEXP, narm: bool) -> $crate::SEXP {
                let data =
                    unsafe { <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
                <$ty as $crate::altrep_data::AltRealData>::min(data, narm)
                    .map(|m| $crate::SEXP::scalar_real(m))
                    .unwrap_or($crate::SEXP::null())
            }

            const HAS_MAX: bool = true;

            fn max(x: $crate::SEXP, narm: bool) -> $crate::SEXP {
                let data =
                    unsafe { <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
                <$ty as $crate::altrep_data::AltRealData>::max(data, narm)
                    .map(|m| $crate::SEXP::scalar_real(m))
                    .unwrap_or($crate::SEXP::null())
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
    // Note the asymmetry: the `dataptr` knob is typed `i32` (R's LGLSXP storage),
    // while the materializing default goes through `RLogical` for type safety.
    ($ty:ty $(, $knob:ident)*) => {
        $crate::impl_altlogical_from_data_generic!({} $ty {} $(, $knob)*);
    };
}

/// Generic form of [`impl_altlogical_from_data!`] — see
/// [`impl_altinteger_from_data_generic!`] for the calling convention.
#[macro_export]
macro_rules! impl_altlogical_from_data_generic {
    ({$($gen:tt)*} $ty:ty {$($whr:tt)*} $(, $knob:ident)*) => {
        $crate::__impl_alt_family!(
            {$($gen)*} $ty {$($whr)*},
            __impl_altlogical_methods,
            impl_inferbase_logical,
            dataptr: dataptr(i32),
            default: materializing($crate::RLogical)
            $(, $knob)*
        );
    };
}

/// Internal macro: impl AltLogical methods from AltLogicalData
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_altlogical_methods {
    ($ty:ty) => {
        $crate::__impl_altlogical_methods!({} $ty {});
    };
    // Generic form: `[$gen] $ty [$where]`.
    ({$($gen:tt)*} $ty:ty {$($whr:tt)*}) => {
        impl<$($gen)*> $crate::altrep_traits::AltLogical for $ty where $($whr)* {
            // Logical elt is special: returns Logical → .to_r_int()
            const HAS_ELT: bool = true;

            fn elt(x: $crate::SEXP, i: $crate::R_xlen_t) -> i32 {
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
            fn sum(x: $crate::SEXP, narm: bool) -> $crate::SEXP {
                let data =
                    unsafe { <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
                match <$ty as $crate::altrep_data::AltLogicalData>::sum(data, narm) {
                    Some(s) => {
                        if s >= i32::MIN as i64 && s <= i32::MAX as i64 {
                            $crate::SEXP::scalar_integer(s as i32)
                        } else {
                            $crate::SEXP::scalar_real(s as f64)
                        }
                    }
                    None => $crate::SEXP::null(),
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
    ($ty:ty $(, $knob:ident)*) => {
        $crate::impl_altraw_from_data_generic!({} $ty {} $(, $knob)*);
    };
}

/// Generic form of [`impl_altraw_from_data!`] — see
/// [`impl_altinteger_from_data_generic!`] for the calling convention.
#[macro_export]
macro_rules! impl_altraw_from_data_generic {
    ({$($gen:tt)*} $ty:ty {$($whr:tt)*} $(, $knob:ident)*) => {
        $crate::__impl_alt_family!(
            {$($gen)*} $ty {$($whr)*},
            __impl_altraw_methods,
            impl_inferbase_raw,
            dataptr: dataptr(u8),
            default: materializing(u8)
            $(, $knob)*
        );
    };
}

/// Internal macro for AltRaw method implementations.
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_altraw_methods {
    ($ty:ty) => {
        $crate::__impl_altraw_methods!({} $ty {});
    };
    // Generic form: `[$gen] $ty [$where]`.
    ({$($gen:tt)*} $ty:ty {$($whr:tt)*}) => {
        impl<$($gen)*> $crate::altrep_traits::AltRaw for $ty where $($whr)* {
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
    // String vectors have no contiguous typed pointer; the default and `dataptr`
    // knobs both route through `string_dataptr` (whole-vector STRSXP materialization).
    ($ty:ty $(, $knob:ident)*) => {
        $crate::impl_altstring_from_data_generic!({} $ty {} $(, $knob)*);
    };
}

/// Generic form of [`impl_altstring_from_data!`] — see
/// [`impl_altinteger_from_data_generic!`] for the calling convention.
#[macro_export]
macro_rules! impl_altstring_from_data_generic {
    ({$($gen:tt)*} $ty:ty {$($whr:tt)*} $(, $knob:ident)*) => {
        $crate::__impl_alt_family!(
            {$($gen)*} $ty {$($whr)*},
            __impl_altstring_methods,
            impl_inferbase_string,
            dataptr: string_dataptr,
            default: string_dataptr
            $(, $knob)*
        );
    };
}

/// Internal macro for AltString method implementations.
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_altstring_methods {
    ($ty:ty) => {
        $crate::__impl_altstring_methods!({} $ty {});
    };
    // Generic form: `[$gen] $ty [$where]`.
    ({$($gen:tt)*} $ty:ty {$($whr:tt)*}) => {
        impl<$($gen)*> $crate::altrep_traits::AltString for $ty where $($whr)* {
            // String elt with lazy per-element caching in data2 STRSXP.
            //
            // On first access, allocates a STRSXP in data2 (initialized to R_NaString).
            // Each element is computed from Rust on first access and cached. Subsequent
            // accesses return the cached CHARSXP directly.
            //
            // For NA elements (Rust elt returns None), data2[i] stays R_NaString — we
            // re-probe Rust each time (O(1) index, returns None immediately). This is
            // simpler than a separate materialization bitmap and the cost is negligible.
            fn elt(x: $crate::SEXP, i: $crate::R_xlen_t) -> $crate::SEXP {
                unsafe {
                    let idx = i.max(0) as usize;

                    // Get or allocate the data2 cache STRSXP
                    let mut data2 = $crate::altrep_ext::AltrepSexpExt::altrep_data2_raw(&x);
                    if data2.is_null()
                        || $crate::SexpExt::type_of(&data2) != $crate::SEXPTYPE::STRSXP
                    {
                        let n = <$ty as $crate::altrep_traits::Altrep>::length(x);
                        // Rf_allocVector(STRSXP, n) leaves elements UNINITIALIZED
                        // (garbage SEXP pointers). Must fill with R_NaString sentinel.
                        //
                        // Inside ALTREP dispatch — _unchecked variants skip the
                        // with_r_thread debug-assert (MXL301 permits).
                        data2 = $crate::sys::Rf_protect_unchecked(
                            $crate::sys::Rf_allocVector_unchecked($crate::SEXPTYPE::STRSXP, n),
                        );
                        for j in 0..n {
                            $crate::SexpExt::set_string_elt(&data2, j, $crate::SEXP::na_string());
                        }
                        $crate::altrep_ext::AltrepSexpExt::set_altrep_data2(&x, data2);
                        $crate::sys::Rf_unprotect_unchecked(1);
                    }

                    // Check cache: non-NA means already materialized.
                    //
                    // Must use the clamped `idx`, not the raw `i` — `data2` is a
                    // plain (non-ALTREP) STRSXP, so `string_elt`/`set_string_elt`
                    // go through R's own bounds-checked STRING_ELT/SET_STRING_ELT
                    // and would themselves error on a negative index instead of
                    // falling through to the clamp below (#1190).
                    let cached = $crate::SexpExt::string_elt(&data2, idx as $crate::R_xlen_t);
                    if cached != $crate::SEXP::na_string() {
                        return cached;
                    }

                    // Cache miss (or genuine NA) — probe Rust source
                    let data = <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x);
                    match <$ty as $crate::altrep_data::AltStringData>::elt(data, idx) {
                        Some(s) => {
                            let charsxp = $crate::altrep_impl::checked_mkchar(s);
                            $crate::SexpExt::set_string_elt(&data2, idx as $crate::R_xlen_t, charsxp);
                            charsxp
                        }
                        None => $crate::SEXP::na_string(),
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
        $crate::impl_altlist_from_data_generic!({} $ty {}, $guard);
    };
}

/// Generic form of [`impl_altlist_from_data!`]: accepts an optional generic
/// parameter list and where-clause so it can target `struct Foo<T> { .. }`
/// types, not just concrete ones. The non-generic macro above forwards
/// here with empty `{}` brackets so there is exactly one emission body.
#[macro_export]
macro_rules! impl_altlist_from_data_generic {
    ({$($gen:tt)*} $ty:ty {$($whr:tt)*}) => {
        $crate::impl_altlist_from_data_generic!({$($gen)*} $ty {$($whr)*}, RUnwind);
    };
    ({$($gen:tt)*} $ty:ty {$($whr:tt)*}, $guard:ident) => {
        impl<$($gen)*> $crate::altrep_traits::Altrep for $ty where $($whr)* {
            const GUARD: $crate::altrep_traits::AltrepGuard =
                $crate::altrep_traits::AltrepGuard::$guard;

            fn length(x: $crate::SEXP) -> $crate::R_xlen_t {
                let data =
                    unsafe { <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
                <$ty as $crate::altrep_data::AltrepLen>::len(data) as $crate::R_xlen_t
            }
        }

        impl<$($gen)*> $crate::altrep_traits::AltVec for $ty where $($whr)* {}

        impl<$($gen)*> $crate::altrep_traits::AltList for $ty where $($whr)* {
            fn elt(x: $crate::SEXP, i: $crate::R_xlen_t) -> $crate::SEXP {
                let data =
                    unsafe { <$ty as $crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
                <$ty as $crate::altrep_data::AltListData>::elt(data, i.max(0) as usize)
            }
        }

        $crate::impl_inferbase_list!({$($gen)*} $ty {$($whr)*});
    };
}

/// Internal macro: impl AltComplex methods (elt, get_region)
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_altcomplex_methods {
    ($ty:ty) => {
        $crate::__impl_altcomplex_methods!({} $ty {});
    };
    // Generic form: `[$gen] $ty [$where]`.
    ({$($gen:tt)*} $ty:ty {$($whr:tt)*}) => {
        impl<$($gen)*> $crate::altrep_traits::AltComplex for $ty where $($whr)* {
            $crate::__impl_alt_elt!(
                $ty,
                $crate::altrep_data::AltComplexData,
                $crate::Rcomplex,
                $crate::Rcomplex {
                    r: f64::NAN,
                    i: f64::NAN
                }
            );
            $crate::__impl_alt_get_region!(
                $ty,
                $crate::altrep_data::AltComplexData,
                $crate::Rcomplex
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
    ($ty:ty $(, $knob:ident)*) => {
        $crate::impl_altcomplex_from_data_generic!({} $ty {} $(, $knob)*);
    };
}

/// Generic form of [`impl_altcomplex_from_data!`] — see
/// [`impl_altinteger_from_data_generic!`] for the calling convention.
#[macro_export]
macro_rules! impl_altcomplex_from_data_generic {
    ({$($gen:tt)*} $ty:ty {$($whr:tt)*} $(, $knob:ident)*) => {
        $crate::__impl_alt_family!(
            {$($gen)*} $ty {$($whr)*},
            __impl_altcomplex_methods,
            impl_inferbase_complex,
            dataptr: dataptr($crate::Rcomplex),
            default: materializing($crate::Rcomplex)
            $(, $knob)*
        );
    };
}
// endregion
