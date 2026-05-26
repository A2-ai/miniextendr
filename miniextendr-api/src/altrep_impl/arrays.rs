//! ALTREP impls for const-generic arrays `[T; N]`.
//!
//! Arrays use const generics (`impl<const N: usize>`), so they can't ride the
//! `impl_alt*_from_data!` macro path (which uses `$ty:ty` and needs a concrete
//! type to delegate through). Numeric families share enough structure to be
//! covered by the local `impl_altrep_array_numeric!` helper macro defined
//! below; bool and String arrays differ structurally and are hand-written.

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
            fn length(x: crate::sys::SEXP) -> crate::sys::R_xlen_t {
                let data = unsafe {
                    <[$elem; N] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x)
                };
                crate::altrep_data::AltrepLen::len(data) as crate::sys::R_xlen_t
            }
        }

        impl<const N: usize> crate::altrep_traits::AltVec for [$elem; N] {
            const HAS_DATAPTR: bool = true;

            fn dataptr(x: crate::sys::SEXP, _writable: bool) -> *mut core::ffi::c_void {
                let data = unsafe {
                    <[$elem; N] as crate::altrep_data::AltrepExtract>::altrep_extract_mut(x)
                };
                data.as_mut_ptr().cast::<core::ffi::c_void>()
            }
        }

        impl<const N: usize> $alt_trait for [$elem; N] {
            const HAS_ELT: bool = true;

            fn elt(x: crate::sys::SEXP, i: crate::sys::R_xlen_t) -> $elem {
                let data = unsafe {
                    <[$elem; N] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x)
                };
                <[$elem; N] as $data_trait>::elt(data, i.max(0) as usize)
            }

            const HAS_GET_REGION: bool = true;

            fn get_region(
                x: crate::sys::SEXP,
                start: crate::sys::R_xlen_t,
                len: crate::sys::R_xlen_t,
                buf: &mut [$elem],
            ) -> crate::sys::R_xlen_t {
                if start < 0 || len <= 0 {
                    return 0;
                }
                let data = unsafe {
                    <[$elem; N] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x)
                };
                <[$elem; N] as $data_trait>::get_region(data, start as usize, len as usize, buf)
                    as crate::sys::R_xlen_t
            }

            $($($extra)*)?
        }

        impl<const N: usize> crate::altrep_data::InferBase for [$elem; N] {
            const BASE: crate::altrep::RBase = $rbase;

            unsafe fn make_class(
                class_name: *const i8,
                pkg_name: *const i8,
            ) -> crate::sys::altrep::R_altrep_class_t {
                let cls = unsafe { $make_class_fn(class_name, pkg_name, crate::altrep_dll_info()) };
                let name = unsafe { core::ffi::CStr::from_ptr(class_name) };
                crate::altrep::validate_altrep_class(cls, name, Self::BASE)
            }

            unsafe fn install_methods(cls: crate::sys::altrep::R_altrep_class_t) {
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

        fn no_na(x: crate::sys::SEXP) -> i32 {
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
    make_class_fn = crate::sys::altrep::R_make_altinteger_class,
    install_family_fn = install_int,
    extra { altrep_array_no_na!(i32, crate::altrep_data::AltIntegerData); },
);
impl_altrep_array_numeric!(
    elem = f64,
    data_trait = crate::altrep_data::AltRealData,
    alt_trait = crate::altrep_traits::AltReal,
    rbase = crate::altrep::RBase::Real,
    make_class_fn = crate::sys::altrep::R_make_altreal_class,
    install_family_fn = install_real,
    extra { altrep_array_no_na!(f64, crate::altrep_data::AltRealData); },
);
impl_altrep_array_numeric!(
    elem = u8,
    data_trait = crate::altrep_data::AltRawData,
    alt_trait = crate::altrep_traits::AltRaw,
    rbase = crate::altrep::RBase::Raw,
    make_class_fn = crate::sys::altrep::R_make_altraw_class,
    install_family_fn = install_raw,
);
impl_altrep_array_numeric!(
    elem = crate::sys::Rcomplex,
    data_trait = crate::altrep_data::AltComplexData,
    alt_trait = crate::altrep_traits::AltComplex,
    rbase = crate::altrep::RBase::Complex,
    make_class_fn = crate::sys::altrep::R_make_altcomplex_class,
    install_family_fn = install_cplx,
);

// Logical arrays — bool != i32, no direct dataptr, elt returns i32 via to_r_int()
impl<const N: usize> crate::altrep_traits::Altrep for [bool; N] {
    fn length(x: crate::sys::SEXP) -> crate::sys::R_xlen_t {
        let data =
            unsafe { <[bool; N] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        crate::altrep_data::AltrepLen::len(data) as crate::sys::R_xlen_t
    }
}

impl<const N: usize> crate::altrep_traits::AltVec for [bool; N] {}

impl<const N: usize> crate::altrep_traits::AltLogical for [bool; N] {
    const HAS_ELT: bool = true;

    fn elt(x: crate::sys::SEXP, i: crate::sys::R_xlen_t) -> i32 {
        let data =
            unsafe { <[bool; N] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        <[bool; N] as crate::altrep_data::AltLogicalData>::elt(data, i.max(0) as usize).to_r_int()
    }

    const HAS_NO_NA: bool = true;

    fn no_na(x: crate::sys::SEXP) -> i32 {
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
    ) -> crate::sys::altrep::R_altrep_class_t {
        let cls = unsafe {
            crate::sys::altrep::R_make_altlogical_class(
                class_name,
                pkg_name,
                crate::altrep_dll_info(),
            )
        };
        let name = unsafe { core::ffi::CStr::from_ptr(class_name) };
        crate::altrep::validate_altrep_class(cls, name, Self::BASE)
    }

    unsafe fn install_methods(cls: crate::sys::altrep::R_altrep_class_t) {
        unsafe { crate::altrep_bridge::install_base::<Self>(cls) };
        unsafe { crate::altrep_bridge::install_vec::<Self>(cls) };
        unsafe { crate::altrep_bridge::install_lgl::<Self>(cls) };
    }
}

// String arrays — no dataptr, elt returns SEXP via checked_mkchar
impl<const N: usize> crate::altrep_traits::Altrep for [String; N] {
    const GUARD: crate::altrep_traits::AltrepGuard = crate::altrep_traits::AltrepGuard::RUnwind;

    fn length(x: crate::sys::SEXP) -> crate::sys::R_xlen_t {
        let data =
            unsafe { <[String; N] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        crate::altrep_data::AltrepLen::len(data) as crate::sys::R_xlen_t
    }
}

impl<const N: usize> crate::altrep_traits::AltVec for [String; N] {}

impl<const N: usize> crate::altrep_traits::AltString for [String; N] {
    fn elt(x: crate::sys::SEXP, i: crate::sys::R_xlen_t) -> crate::sys::SEXP {
        let data =
            unsafe { <[String; N] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        match <[String; N] as crate::altrep_data::AltStringData>::elt(data, i.max(0) as usize) {
            Some(s) => unsafe { super::checked_mkchar(s) },
            None => crate::sys::SEXP::na_string(),
        }
    }
}

impl<const N: usize> crate::altrep_data::InferBase for [String; N] {
    const BASE: crate::altrep::RBase = crate::altrep::RBase::String;

    unsafe fn make_class(
        class_name: *const i8,
        pkg_name: *const i8,
    ) -> crate::sys::altrep::R_altrep_class_t {
        let cls = unsafe {
            crate::sys::altrep::R_make_altstring_class(
                class_name,
                pkg_name,
                crate::altrep_dll_info(),
            )
        };
        let name = unsafe { core::ffi::CStr::from_ptr(class_name) };
        crate::altrep::validate_altrep_class(cls, name, Self::BASE)
    }

    unsafe fn install_methods(cls: crate::sys::altrep::R_altrep_class_t) {
        unsafe { crate::altrep_bridge::install_base::<Self>(cls) };
        unsafe { crate::altrep_bridge::install_vec::<Self>(cls) };
        unsafe { crate::altrep_bridge::install_str::<Self>(cls) };
    }
}
// endregion
