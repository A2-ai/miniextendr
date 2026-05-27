//! Built-in ALTREP class instantiations for `Vec<T>`, `Box<[T]>`, `Cow<T>`,
//! and `Range<T>`.
//!
//! This file contains two crate-private declarative macros that are *not*
//! `#[macro_export]`ed: `impl_builtin_altrep_family!` (emits the family impl
//! plus linkme registration) and `impl_register_altrep_builtin!` (emits the
//! `RegisterAltrep` impl with the cached `OnceLock` class handle).
//!
//! User crates do not invoke these macros directly — they call
//! `impl_alt<family>_from_data!` plus a hand-rolled `RegisterAltrep` impl, or
//! use the `#[derive(Altrep<Family>)]` proc-macro which emits the equivalent
//! code.

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
// The `materializing` arm expands to `($ty, serialize)` in the underlying
// per-family macro — the `($ty, serialize)` arm IS the materializing path
// (it routes through `__impl_altvec_<family>_dataptr!` which provides a
// trivial `AltrepDataptr<T>` returning `None`, falling through to data2
// materialisation for bool→i32 conversion, Range compute-on-access, etc.).
// Both arms preserve `const GUARD = AltrepGuard::RUnwind` — the default from
// `__impl_altrep_base!`'s default and `with_serialize` arms.
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
    //
    // The materializing path is the *default* arm of `impl_alt<family>_from_data!`
    // (the per-family `__impl_altvec_<family>_dataptr!` provides a trivial
    // `AltrepDataptr<T>` returning `None`, falling through to materialise into
    // data2). `serialize` is added to all builtin instantiations.
    ($ty:ty, integer, materializing, $reg_fn:ident, $entry_ident:ident) => {
        $crate::impl_altinteger_from_data!($ty, serialize);
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
        $crate::impl_altreal_from_data!($ty, serialize);
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
        $crate::impl_altlogical_from_data!($ty, serialize);
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
        $crate::impl_altraw_from_data!($ty, serialize);
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
        $crate::impl_altstring_from_data!($ty, serialize);
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
        $crate::impl_altcomplex_from_data!($ty, serialize);
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
// Guard: RUnwind (default from __impl_altrep_base!'s arms — not overridden).

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
    Vec<crate::Rcomplex>,
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
    Box<[crate::Rcomplex]>,
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
    std::borrow::Cow<'static, [crate::Rcomplex]>,
    complex,
    dataptr,
    __mx_altrep_reg_builtin_Cow_Rcomplex,
    __MX_ALTREP_REG_ENTRY_builtin_Cow_Rcomplex
);

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
            fn get_or_init_class() -> crate::sys::altrep::R_altrep_class_t {
                use std::sync::OnceLock;
                static CLASS: OnceLock<crate::sys::altrep::R_altrep_class_t> = OnceLock::new();
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
impl_register_altrep_builtin!(Vec<crate::Rcomplex>, "Vec_Rcomplex");

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
impl_register_altrep_builtin!(Box<[crate::Rcomplex]>, "Box_Rcomplex");

// Cow types - RegisterAltrep for zero-copy borrow from R
impl_register_altrep_builtin!(std::borrow::Cow<'static, [i32]>, "Cow_i32");
impl_register_altrep_builtin!(std::borrow::Cow<'static, [f64]>, "Cow_f64");
impl_register_altrep_builtin!(std::borrow::Cow<'static, [u8]>, "Cow_u8");
impl_register_altrep_builtin!(std::borrow::Cow<'static, [crate::Rcomplex]>, "Cow_Rcomplex");

// Cow string vector types
impl_register_altrep_builtin!(Vec<std::borrow::Cow<'static, str>>, "Vec_Cow_str");
impl_register_altrep_builtin!(
    Vec<Option<std::borrow::Cow<'static, str>>>,
    "Vec_Option_Cow_str"
);
// endregion
