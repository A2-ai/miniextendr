//! NA-aware vector conversions (`Vec<Option<T>>`, `Box<[Option<T>]>`).
//!
//! Maps R's NA values to `None` and non-NA values to `Some(v)`.
//! Covers native types (i32, f64, u8), logical (bool, Rboolean, RLogical),
//! string (`Option<String>`), complex (`Option<Rcomplex>`), and coerced
//! numeric types (`Option<i64>`, `Option<u64>`, etc.).

use crate::coerce::TryCoerce;
use crate::ffi::{RLogical, Rboolean, SEXP, SEXPTYPE, SexpExt};
use crate::from_r::{
    SexpError, SexpNaError, SexpTypeError, TryFromSexp, coerce_value, is_na_real, r_slice,
};

/// Macro for NA-aware `R vector → Vec<Option<T>>` conversions.
macro_rules! impl_vec_option_try_from_sexp {
    ($t:ty, $sexptype:ident, $dataptr:ident, $is_na:expr) => {
        impl TryFromSexp for Vec<Option<$t>> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let actual = sexp.type_of();
                if actual != SEXPTYPE::$sexptype {
                    return Err(SexpTypeError {
                        expected: SEXPTYPE::$sexptype,
                        actual,
                    }
                    .into());
                }

                let len = sexp.len();
                let ptr = unsafe { crate::ffi::$dataptr(sexp) };
                let slice = unsafe { r_slice(ptr, len) };

                Ok(slice
                    .iter()
                    .map(|&v| if $is_na(v) { None } else { Some(v) })
                    .collect())
            }
        }
    };
}

impl_vec_option_try_from_sexp!(f64, REALSXP, REAL_unchecked, is_na_real);
impl_vec_option_try_from_sexp!(i32, INTSXP, INTEGER_unchecked, |v: i32| v == i32::MIN);

/// Macro for NA-aware `R vector → Box<[Option<T>]>` conversions.
macro_rules! impl_boxed_slice_option_try_from_sexp {
    ($t:ty, $sexptype:ident, $dataptr:ident, $is_na:expr) => {
        impl TryFromSexp for Box<[Option<$t>]> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let actual = sexp.type_of();
                if actual != SEXPTYPE::$sexptype {
                    return Err(SexpTypeError {
                        expected: SEXPTYPE::$sexptype,
                        actual,
                    }
                    .into());
                }

                let len = sexp.len();
                let ptr = unsafe { crate::ffi::$dataptr(sexp) };
                let slice = unsafe { r_slice(ptr, len) };

                Ok(slice
                    .iter()
                    .map(|&v| if $is_na(v) { None } else { Some(v) })
                    .collect())
            }
        }
    };
}

impl_boxed_slice_option_try_from_sexp!(f64, REALSXP, REAL_unchecked, is_na_real);
impl_boxed_slice_option_try_from_sexp!(i32, INTSXP, INTEGER_unchecked, |v: i32| v == i32::MIN);

/// Convert R logical vector (LGLSXP) to `Vec<Option<bool>>` with NA support.
impl TryFromSexp for Vec<Option<bool>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::LGLSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::LGLSXP,
                actual,
            }
            .into());
        }

        let len = sexp.len();
        let ptr = unsafe { crate::ffi::LOGICAL_unchecked(sexp) };
        let slice = unsafe { r_slice(ptr, len) };

        Ok(slice
            .iter()
            .map(|&v| RLogical::from_i32(v).to_option_bool())
            .collect())
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::LGLSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::LGLSXP,
                actual,
            }
            .into());
        }

        let len = unsafe { sexp.len_unchecked() };
        let ptr = unsafe { crate::ffi::LOGICAL_unchecked(sexp) };
        let slice = unsafe { r_slice(ptr, len) };

        Ok(slice
            .iter()
            .map(|&v| RLogical::from_i32(v).to_option_bool())
            .collect())
    }
}

/// Convert R logical vector (LGLSXP) to `Box<[Option<bool>]>` with NA support.
impl TryFromSexp for Box<[Option<bool>]> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let vec: Vec<Option<bool>> = TryFromSexp::try_from_sexp(sexp)?;
        Ok(vec.into_boxed_slice())
    }
}

/// Convert R logical vector (LGLSXP) to `Vec<Rboolean>` (errors on NA).
impl TryFromSexp for Vec<Rboolean> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::LGLSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::LGLSXP,
                actual,
            }
            .into());
        }

        let len = sexp.len();
        let ptr = unsafe { crate::ffi::LOGICAL_unchecked(sexp) };
        let slice = unsafe { r_slice(ptr, len) };

        slice
            .iter()
            .map(|&v| {
                let raw = RLogical::from_i32(v);
                match raw.to_option_bool() {
                    Some(false) => Ok(Rboolean::FALSE),
                    Some(true) => Ok(Rboolean::TRUE),
                    None => Err(SexpNaError {
                        sexp_type: SEXPTYPE::LGLSXP,
                    }
                    .into()),
                }
            })
            .collect()
    }
}

/// Convert R logical vector (LGLSXP) to `Vec<Option<Rboolean>>` with NA support.
impl TryFromSexp for Vec<Option<Rboolean>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::LGLSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::LGLSXP,
                actual,
            }
            .into());
        }

        let len = sexp.len();
        let ptr = unsafe { crate::ffi::LOGICAL_unchecked(sexp) };
        let slice = unsafe { r_slice(ptr, len) };

        Ok(slice
            .iter()
            .map(|&v| match RLogical::from_i32(v).to_option_bool() {
                Some(false) => Some(Rboolean::FALSE),
                Some(true) => Some(Rboolean::TRUE),
                None => None,
            })
            .collect())
    }
}

/// Convert R logical vector (LGLSXP) to `Vec<Logical>` (ALTREP-compatible).
///
/// This converts R's logical vector to a vector of [`Logical`](crate::altrep_data::Logical) values,
/// which is the native representation used by ALTREP logical vectors.
/// Unlike `Vec<bool>`, this preserves NA values.
impl TryFromSexp for Vec<crate::altrep_data::Logical> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::LGLSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::LGLSXP,
                actual,
            }
            .into());
        }

        let len = sexp.len();
        let ptr = unsafe { crate::ffi::LOGICAL_unchecked(sexp) };
        let slice = unsafe { r_slice(ptr, len) };

        Ok(slice
            .iter()
            .map(|&v| crate::altrep_data::Logical::from_r_int(v))
            .collect())
    }
}

/// Convert R logical vector (LGLSXP) to `Vec<Option<RLogical>>` with NA support.
impl TryFromSexp for Vec<Option<RLogical>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::LGLSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::LGLSXP,
                actual,
            }
            .into());
        }

        let len = sexp.len();
        let ptr = unsafe { crate::ffi::LOGICAL_unchecked(sexp) };
        let slice = unsafe { r_slice(ptr, len) };

        Ok(slice
            .iter()
            .map(|&v| {
                let raw = RLogical::from_i32(v);
                if raw.is_na() { None } else { Some(raw) }
            })
            .collect())
    }
}

/// Convert R character vector (STRSXP) to `Vec<Option<String>>` with NA support.
///
/// `NA_character_` elements are converted to `None`.
impl TryFromSexp for Vec<Option<String>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        use crate::ffi::Rf_translateCharUTF8;

        let actual = sexp.type_of();
        if actual != SEXPTYPE::STRSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::STRSXP,
                actual,
            }
            .into());
        }

        let len = sexp.len();
        let mut result = Vec::with_capacity(len);

        for i in 0..len {
            let charsxp = sexp.string_elt(i as crate::ffi::R_xlen_t);

            if charsxp == unsafe { crate::ffi::R_NaString } {
                result.push(None);
            } else {
                let c_str = unsafe { Rf_translateCharUTF8(charsxp) };
                if c_str.is_null() {
                    result.push(Some(String::new()));
                } else {
                    let rust_str = unsafe { std::ffi::CStr::from_ptr(c_str) };
                    result.push(Some(rust_str.to_str().map(|s| s.to_owned()).map_err(
                        |_| SexpTypeError {
                            expected: SEXPTYPE::STRSXP,
                            actual: SEXPTYPE::STRSXP,
                        },
                    )?));
                }
            }
        }

        Ok(result)
    }
}

/// Convert R character vector to `Box<[Option<String>]>` with NA support.
impl TryFromSexp for Box<[Option<String>]> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let vec: Vec<Option<String>> = TryFromSexp::try_from_sexp(sexp)?;
        Ok(vec.into_boxed_slice())
    }
}

/// Convert R raw vector (RAWSXP) to `Vec<Option<u8>>`.
impl TryFromSexp for Vec<Option<u8>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::RAWSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::RAWSXP,
                actual,
            }
            .into());
        }

        let len = sexp.len();
        let ptr = unsafe { crate::ffi::RAW_unchecked(sexp) };
        let slice = unsafe { r_slice(ptr, len) };

        Ok(slice.iter().map(|&v| Some(v)).collect())
    }
}

#[inline]
fn try_from_sexp_numeric_option_vec<T>(sexp: SEXP) -> Result<Vec<Option<T>>, SexpError>
where
    i32: TryCoerce<T>,
    f64: TryCoerce<T>,
    u8: TryCoerce<T>,
    <i32 as TryCoerce<T>>::Error: std::fmt::Debug,
    <f64 as TryCoerce<T>>::Error: std::fmt::Debug,
    <u8 as TryCoerce<T>>::Error: std::fmt::Debug,
{
    let actual = sexp.type_of();
    match actual {
        SEXPTYPE::INTSXP => {
            let slice: &[i32] = unsafe { sexp.as_slice() };
            slice
                .iter()
                .map(|&v| {
                    if v == crate::altrep_traits::NA_INTEGER {
                        Ok(None)
                    } else {
                        coerce_value(v).map(Some)
                    }
                })
                .collect()
        }
        SEXPTYPE::REALSXP => {
            let slice: &[f64] = unsafe { sexp.as_slice() };
            slice
                .iter()
                .map(|&v| {
                    if is_na_real(v) {
                        Ok(None)
                    } else {
                        coerce_value(v).map(Some)
                    }
                })
                .collect()
        }
        SEXPTYPE::RAWSXP => {
            let slice: &[u8] = unsafe { sexp.as_slice() };
            slice.iter().map(|&v| coerce_value(v).map(Some)).collect()
        }
        SEXPTYPE::LGLSXP => {
            let slice: &[RLogical] = unsafe { sexp.as_slice() };
            slice
                .iter()
                .map(|&v| {
                    if v.is_na() {
                        Ok(None)
                    } else {
                        coerce_value(v.to_i32()).map(Some)
                    }
                })
                .collect()
        }
        _ => Err(SexpError::InvalidValue(format!(
            "expected integer, numeric, logical, or raw; got {:?}",
            actual
        ))),
    }
}

macro_rules! impl_vec_option_try_from_sexp_numeric {
    ($t:ty) => {
        impl TryFromSexp for Vec<Option<$t>> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                try_from_sexp_numeric_option_vec(sexp)
            }

            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                try_from_sexp_numeric_option_vec(sexp)
            }
        }
    };
}

impl_vec_option_try_from_sexp_numeric!(i8);
impl_vec_option_try_from_sexp_numeric!(i16);
impl_vec_option_try_from_sexp_numeric!(u16);
impl_vec_option_try_from_sexp_numeric!(u32);
impl_vec_option_try_from_sexp_numeric!(i64);
impl_vec_option_try_from_sexp_numeric!(u64);
impl_vec_option_try_from_sexp_numeric!(isize);
impl_vec_option_try_from_sexp_numeric!(usize);
impl_vec_option_try_from_sexp_numeric!(f32);
// endregion
