//! ALTREP impls for `&'static [T]`.
//!
//! Static slices are Sized (fat pointer: ptr + len) and satisfy `'static`, so
//! they work directly with the ExternalPtr-backed ALTREP machinery. These
//! impls are hand-written rather than macro-generated because the writable
//! `dataptr` path must assert `!writable` (static data is immutable).
//!
//! Use cases: const arrays via `&DATA[..]`, leaked vectors via
//! `Box::leak(vec.into_boxed_slice())`, memory-mapped files with `'static`
//! lifetime.

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
    fn length(x: crate::sys::SEXP) -> crate::sys::R_xlen_t {
        let data =
            unsafe { <&'static [i32] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        crate::altrep_data::AltrepLen::len(data) as crate::sys::R_xlen_t
    }
}

impl crate::altrep_traits::AltVec for &'static [i32] {
    const HAS_DATAPTR: bool = true;

    fn dataptr(x: crate::sys::SEXP, writable: bool) -> *mut std::ffi::c_void {
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

    fn dataptr_or_null(x: crate::sys::SEXP) -> *const std::ffi::c_void {
        let data =
            unsafe { <&'static [i32] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        data.as_ptr().cast::<std::ffi::c_void>()
    }
}

impl crate::altrep_traits::AltInteger for &'static [i32] {
    const HAS_ELT: bool = true;

    fn elt(x: crate::sys::SEXP, i: crate::sys::R_xlen_t) -> i32 {
        let data =
            unsafe { <&'static [i32] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        crate::altrep_data::AltIntegerData::elt(data, i.max(0) as usize)
    }

    const HAS_GET_REGION: bool = true;

    fn get_region(
        x: crate::sys::SEXP,
        start: crate::sys::R_xlen_t,
        len: crate::sys::R_xlen_t,
        buf: &mut [i32],
    ) -> crate::sys::R_xlen_t {
        if start < 0 || len <= 0 {
            return 0;
        }
        let data =
            unsafe { <&'static [i32] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        let len = len as usize;
        crate::altrep_data::AltIntegerData::get_region(data, start as usize, len, buf)
            as crate::sys::R_xlen_t
    }

    const HAS_NO_NA: bool = true;

    fn no_na(x: crate::sys::SEXP) -> i32 {
        let data =
            unsafe { <&'static [i32] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        crate::altrep_data::AltIntegerData::no_na(data)
            .map(|b| if b { 1 } else { 0 })
            .unwrap_or(0)
    }

    const HAS_SUM: bool = true;

    // ALTREP protocol: return C NULL (not R_NilValue) to signal "can't compute"
    fn sum(x: crate::sys::SEXP, narm: bool) -> crate::sys::SEXP {
        let data =
            unsafe { <&'static [i32] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        crate::altrep_data::AltIntegerData::sum(data, narm)
            .map(|s| {
                if s >= i32::MIN as i64 && s <= i32::MAX as i64 {
                    crate::sys::SEXP::scalar_integer(s as i32)
                } else {
                    crate::sys::SEXP::scalar_real(s as f64)
                }
            })
            .unwrap_or(crate::sys::SEXP::null())
    }

    const HAS_MIN: bool = true;

    fn min(x: crate::sys::SEXP, narm: bool) -> crate::sys::SEXP {
        let data =
            unsafe { <&'static [i32] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        crate::altrep_data::AltIntegerData::min(data, narm)
            .map(crate::sys::SEXP::scalar_integer)
            .unwrap_or(crate::sys::SEXP::null())
    }

    const HAS_MAX: bool = true;

    fn max(x: crate::sys::SEXP, narm: bool) -> crate::sys::SEXP {
        let data =
            unsafe { <&'static [i32] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        crate::altrep_data::AltIntegerData::max(data, narm)
            .map(crate::sys::SEXP::scalar_integer)
            .unwrap_or(crate::sys::SEXP::null())
    }
}

crate::impl_inferbase_integer!(&'static [i32]);

// Real static slices
impl crate::altrep_traits::Altrep for &'static [f64] {
    fn length(x: crate::sys::SEXP) -> crate::sys::R_xlen_t {
        let data =
            unsafe { <&'static [f64] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        crate::altrep_data::AltrepLen::len(data) as crate::sys::R_xlen_t
    }
}

impl crate::altrep_traits::AltVec for &'static [f64] {
    const HAS_DATAPTR: bool = true;

    fn dataptr(x: crate::sys::SEXP, writable: bool) -> *mut std::ffi::c_void {
        assert!(
            !writable,
            "cannot get writable DATAPTR for static ALTREP data"
        );
        let data =
            unsafe { <&'static [f64] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        data.as_ptr().cast::<std::ffi::c_void>().cast_mut()
    }

    const HAS_DATAPTR_OR_NULL: bool = true;

    fn dataptr_or_null(x: crate::sys::SEXP) -> *const std::ffi::c_void {
        let data =
            unsafe { <&'static [f64] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        data.as_ptr().cast::<std::ffi::c_void>()
    }
}

impl crate::altrep_traits::AltReal for &'static [f64] {
    const HAS_ELT: bool = true;

    fn elt(x: crate::sys::SEXP, i: crate::sys::R_xlen_t) -> f64 {
        let data =
            unsafe { <&'static [f64] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        crate::altrep_data::AltRealData::elt(data, i.max(0) as usize)
    }

    const HAS_GET_REGION: bool = true;

    fn get_region(
        x: crate::sys::SEXP,
        start: crate::sys::R_xlen_t,
        len: crate::sys::R_xlen_t,
        buf: &mut [f64],
    ) -> crate::sys::R_xlen_t {
        if start < 0 || len <= 0 {
            return 0;
        }
        let data =
            unsafe { <&'static [f64] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        let len = len as usize;
        crate::altrep_data::AltRealData::get_region(data, start as usize, len, buf)
            as crate::sys::R_xlen_t
    }

    const HAS_NO_NA: bool = true;

    fn no_na(x: crate::sys::SEXP) -> i32 {
        let data =
            unsafe { <&'static [f64] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        crate::altrep_data::AltRealData::no_na(data)
            .map(|b| if b { 1 } else { 0 })
            .unwrap_or(0)
    }

    const HAS_SUM: bool = true;

    // ALTREP protocol: return C NULL (not R_NilValue) to signal "can't compute"
    fn sum(x: crate::sys::SEXP, narm: bool) -> crate::sys::SEXP {
        let data =
            unsafe { <&'static [f64] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        crate::altrep_data::AltRealData::sum(data, narm)
            .map(crate::sys::SEXP::scalar_real)
            .unwrap_or(crate::sys::SEXP::null())
    }

    const HAS_MIN: bool = true;

    fn min(x: crate::sys::SEXP, narm: bool) -> crate::sys::SEXP {
        let data =
            unsafe { <&'static [f64] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        crate::altrep_data::AltRealData::min(data, narm)
            .map(crate::sys::SEXP::scalar_real)
            .unwrap_or(crate::sys::SEXP::null())
    }

    const HAS_MAX: bool = true;

    fn max(x: crate::sys::SEXP, narm: bool) -> crate::sys::SEXP {
        let data =
            unsafe { <&'static [f64] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        crate::altrep_data::AltRealData::max(data, narm)
            .map(crate::sys::SEXP::scalar_real)
            .unwrap_or(crate::sys::SEXP::null())
    }
}

crate::impl_inferbase_real!(&'static [f64]);

// Logical static slices
impl crate::altrep_traits::Altrep for &'static [bool] {
    fn length(x: crate::sys::SEXP) -> crate::sys::R_xlen_t {
        let data = unsafe {
            <&'static [bool] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x)
        };
        crate::altrep_data::AltrepLen::len(data) as crate::sys::R_xlen_t
    }
}

impl crate::altrep_traits::AltVec for &'static [bool] {}

impl crate::altrep_traits::AltLogical for &'static [bool] {
    const HAS_ELT: bool = true;

    fn elt(x: crate::sys::SEXP, i: crate::sys::R_xlen_t) -> i32 {
        let data = unsafe {
            <&'static [bool] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x)
        };
        crate::altrep_data::AltLogicalData::elt(data, i.max(0) as usize).to_r_int()
    }

    const HAS_NO_NA: bool = true;

    fn no_na(x: crate::sys::SEXP) -> i32 {
        let data = unsafe {
            <&'static [bool] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x)
        };
        crate::altrep_data::AltLogicalData::no_na(data)
            .map(|b| if b { 1 } else { 0 })
            .unwrap_or(0)
    }

    const HAS_SUM: bool = true;

    // ALTREP protocol: return C NULL (not R_NilValue) to signal "can't compute"
    fn sum(x: crate::sys::SEXP, narm: bool) -> crate::sys::SEXP {
        let data = unsafe {
            <&'static [bool] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x)
        };
        crate::altrep_data::AltLogicalData::sum(data, narm)
            .map(|s| {
                if s >= i32::MIN as i64 && s <= i32::MAX as i64 {
                    crate::sys::SEXP::scalar_integer(s as i32)
                } else {
                    crate::sys::SEXP::scalar_real(s as f64)
                }
            })
            .unwrap_or(crate::sys::SEXP::null())
    }
}

crate::impl_inferbase_logical!(&'static [bool]);

// Raw static slices
impl crate::altrep_traits::Altrep for &'static [u8] {
    fn length(x: crate::sys::SEXP) -> crate::sys::R_xlen_t {
        let data =
            unsafe { <&'static [u8] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        crate::altrep_data::AltrepLen::len(data) as crate::sys::R_xlen_t
    }
}

impl crate::altrep_traits::AltVec for &'static [u8] {
    const HAS_DATAPTR: bool = true;

    fn dataptr(x: crate::sys::SEXP, writable: bool) -> *mut std::ffi::c_void {
        assert!(
            !writable,
            "cannot get writable DATAPTR for static ALTREP data"
        );
        let data =
            unsafe { <&'static [u8] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        data.as_ptr().cast::<std::ffi::c_void>().cast_mut()
    }

    const HAS_DATAPTR_OR_NULL: bool = true;

    fn dataptr_or_null(x: crate::sys::SEXP) -> *const std::ffi::c_void {
        let data =
            unsafe { <&'static [u8] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        data.as_ptr().cast::<std::ffi::c_void>()
    }
}

impl crate::altrep_traits::AltRaw for &'static [u8] {
    const HAS_ELT: bool = true;

    fn elt(x: crate::sys::SEXP, i: crate::sys::R_xlen_t) -> crate::sys::Rbyte {
        let data =
            unsafe { <&'static [u8] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        crate::altrep_data::AltRawData::elt(data, i.max(0) as usize)
    }

    const HAS_GET_REGION: bool = true;

    fn get_region(
        x: crate::sys::SEXP,
        start: crate::sys::R_xlen_t,
        len: crate::sys::R_xlen_t,
        buf: &mut [u8],
    ) -> crate::sys::R_xlen_t {
        if start < 0 || len <= 0 {
            return 0;
        }
        let data =
            unsafe { <&'static [u8] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x) };
        let len = len as usize;
        crate::altrep_data::AltRawData::get_region(data, start as usize, len, buf)
            as crate::sys::R_xlen_t
    }
}

crate::impl_inferbase_raw!(&'static [u8]);

// String static slices (owned strings)
impl crate::altrep_traits::Altrep for &'static [String] {
    // String ALTREP elt calls Rf_mkCharLenCE (R API) — must use RUnwind.
    const GUARD: crate::altrep_traits::AltrepGuard = crate::altrep_traits::AltrepGuard::RUnwind;

    fn length(x: crate::sys::SEXP) -> crate::sys::R_xlen_t {
        let data = unsafe {
            <&'static [String] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x)
        };
        crate::altrep_data::AltrepLen::len(data) as crate::sys::R_xlen_t
    }
}

impl crate::altrep_traits::AltVec for &'static [String] {}

impl crate::altrep_traits::AltString for &'static [String] {
    fn elt(x: crate::sys::SEXP, i: crate::sys::R_xlen_t) -> crate::sys::SEXP {
        let data = unsafe {
            <&'static [String] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x)
        };
        match crate::altrep_data::AltStringData::elt(data, i.max(0) as usize) {
            Some(s) => unsafe { super::checked_mkchar(s) },
            None => crate::sys::SEXP::na_string(),
        }
    }

    const HAS_NO_NA: bool = true;

    fn no_na(x: crate::sys::SEXP) -> i32 {
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

    fn length(x: crate::sys::SEXP) -> crate::sys::R_xlen_t {
        let data = unsafe {
            <&'static [&'static str] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x)
        };
        crate::altrep_data::AltrepLen::len(data) as crate::sys::R_xlen_t
    }
}

impl crate::altrep_traits::AltVec for &'static [&'static str] {}

impl crate::altrep_traits::AltString for &'static [&'static str] {
    fn elt(x: crate::sys::SEXP, i: crate::sys::R_xlen_t) -> crate::sys::SEXP {
        let data = unsafe {
            <&'static [&'static str] as crate::altrep_data::AltrepExtract>::altrep_extract_ref(x)
        };
        match crate::altrep_data::AltStringData::elt(data, i.max(0) as usize) {
            Some(s) => unsafe { super::checked_mkchar(s) },
            None => crate::sys::SEXP::na_string(),
        }
    }

    const HAS_NO_NA: bool = true;

    fn no_na(x: crate::sys::SEXP) -> i32 {
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
