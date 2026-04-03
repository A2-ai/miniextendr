//! Reference conversions (borrowed views into R vectors).
//!
//! Provides zero-copy access to R vector data via `'static` references.
//! The lifetime is technically a lie — the data lives as long as R doesn't GC it.
//!
//! Covers: `&T`, `&mut T`, `Option<&T>`, `Vec<&T>`, `Vec<&[T]>`, and
//! mutable variants for all `RNativeType` types.

use crate::ffi::{RLogical, RNativeType, SEXP, SEXPTYPE, SexpExt};
use crate::from_r::{SexpError, SexpLengthError, SexpTypeError, TryFromSexp};

macro_rules! impl_ref_conversions_for {
    ($t:ty) => {
        impl TryFromSexp for &'static $t {
            type Error = SexpError;

            #[inline]
            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let actual = sexp.type_of();
                if actual != <$t as RNativeType>::SEXP_TYPE {
                    return Err(SexpTypeError {
                        expected: <$t as RNativeType>::SEXP_TYPE,
                        actual,
                    }
                    .into());
                }
                let len = sexp.len();
                if len != 1 {
                    return Err(SexpLengthError {
                        expected: 1,
                        actual: len,
                    }
                    .into());
                }
                unsafe { sexp.as_slice::<$t>() }
                    .first()
                    .ok_or_else(|| SexpLengthError { expected: 1, actual: 0 }.into())
            }

            #[inline]
            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                let actual = sexp.type_of();
                if actual != <$t as RNativeType>::SEXP_TYPE {
                    return Err(SexpTypeError {
                        expected: <$t as RNativeType>::SEXP_TYPE,
                        actual,
                    }
                    .into());
                }
                let len = unsafe { sexp.len_unchecked() };
                if len != 1 {
                    return Err(SexpLengthError {
                        expected: 1,
                        actual: len,
                    }
                    .into());
                }
                unsafe { sexp.as_slice_unchecked::<$t>() }
                    .first()
                    .ok_or_else(|| SexpLengthError { expected: 1, actual: 0 }.into())
            }
        }

        /// # Safety note (aliasing)
        ///
        /// This impl can produce aliased `&mut` references if the same R object
        /// is passed to multiple mutable parameters. The caller (generated wrapper)
        /// is responsible for ensuring no two `&mut` borrows alias the same SEXP.
        impl TryFromSexp for &'static mut $t {
            type Error = SexpError;

            #[inline]
            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let actual = sexp.type_of();
                if actual != <$t as RNativeType>::SEXP_TYPE {
                    return Err(SexpTypeError {
                        expected: <$t as RNativeType>::SEXP_TYPE,
                        actual,
                    }
                    .into());
                }
                let len = sexp.len();
                if len != 1 {
                    return Err(SexpLengthError {
                        expected: 1,
                        actual: len,
                    }
                    .into());
                }
                let ptr = unsafe { <$t as RNativeType>::dataptr_mut(sexp) };
                Ok(unsafe { &mut *ptr })
            }

            #[inline]
            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                let actual = sexp.type_of();
                if actual != <$t as RNativeType>::SEXP_TYPE {
                    return Err(SexpTypeError {
                        expected: <$t as RNativeType>::SEXP_TYPE,
                        actual,
                    }
                    .into());
                }
                let len = unsafe { sexp.len_unchecked() };
                if len != 1 {
                    return Err(SexpLengthError {
                        expected: 1,
                        actual: len,
                    }
                    .into());
                }
                let ptr = unsafe { <$t as RNativeType>::dataptr_mut(sexp) };
                Ok(unsafe { &mut *ptr })
            }
        }

        impl TryFromSexp for Option<&'static $t> {
            type Error = SexpError;

            #[inline]
            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                if sexp.type_of() == SEXPTYPE::NILSXP {
                    return Ok(None);
                }
                let value: &'static $t = TryFromSexp::try_from_sexp(sexp)?;
                Ok(Some(value))
            }

            #[inline]
            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                if sexp.type_of() == SEXPTYPE::NILSXP {
                    return Ok(None);
                }
                let value: &'static $t = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
                Ok(Some(value))
            }
        }

        impl TryFromSexp for Option<&'static mut $t> {
            type Error = SexpError;

            #[inline]
            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                if sexp.type_of() == SEXPTYPE::NILSXP {
                    return Ok(None);
                }
                let value: &'static mut $t = TryFromSexp::try_from_sexp(sexp)?;
                Ok(Some(value))
            }

            #[inline]
            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                if sexp.type_of() == SEXPTYPE::NILSXP {
                    return Ok(None);
                }
                let value: &'static mut $t =
                    unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
                Ok(Some(value))
            }
        }

        // Option<&[T]> and Option<&mut [T]> impls removed - now use blanket impls

        impl TryFromSexp for Vec<&'static $t> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let actual = sexp.type_of();
                if actual != SEXPTYPE::VECSXP {
                    return Err(SexpTypeError {
                        expected: SEXPTYPE::VECSXP,
                        actual,
                    }
                    .into());
                }

                let len = sexp.len();
                let mut out = Vec::with_capacity(len);

                for i in 0..len {
                    let elem = sexp.vector_elt(i as crate::ffi::R_xlen_t);
                    let value: &'static $t = TryFromSexp::try_from_sexp(elem)?;
                    out.push(value);
                }

                Ok(out)
            }
        }

        impl TryFromSexp for Vec<Option<&'static $t>> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let actual = sexp.type_of();
                if actual != SEXPTYPE::VECSXP {
                    return Err(SexpTypeError {
                        expected: SEXPTYPE::VECSXP,
                        actual,
                    }
                    .into());
                }

                let len = sexp.len();
                let mut out = Vec::with_capacity(len);

                for i in 0..len {
                    let elem = sexp.vector_elt(i as crate::ffi::R_xlen_t);
                    if elem.type_of() == SEXPTYPE::NILSXP {
                        out.push(None);
                    } else {
                        let value: &'static $t = TryFromSexp::try_from_sexp(elem)?;
                        out.push(Some(value));
                    }
                }

                Ok(out)
            }
        }

        impl TryFromSexp for Vec<&'static mut $t> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let actual = sexp.type_of();
                if actual != SEXPTYPE::VECSXP {
                    return Err(SexpTypeError {
                        expected: SEXPTYPE::VECSXP,
                        actual,
                    }
                    .into());
                }

                let len = sexp.len();
                let mut out = Vec::with_capacity(len);
                let mut ptrs: Vec<*mut $t> = Vec::new();

                for i in 0..len {
                    let elem = sexp.vector_elt(i as crate::ffi::R_xlen_t);
                    let value: &'static mut $t = TryFromSexp::try_from_sexp(elem)?;
                    let ptr = std::ptr::from_mut(value);
                    if ptrs.iter().any(|&p| p == ptr) {
                        return Err(SexpError::InvalidValue(
                            "list contains duplicate elements; cannot create multiple mutable references"
                                .to_string(),
                        ));
                    }
                    ptrs.push(ptr);
                    out.push(value);
                }

                Ok(out)
            }
        }

        impl TryFromSexp for Vec<Option<&'static mut $t>> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let actual = sexp.type_of();
                if actual != SEXPTYPE::VECSXP {
                    return Err(SexpTypeError {
                        expected: SEXPTYPE::VECSXP,
                        actual,
                    }
                    .into());
                }

                let len = sexp.len();
                let mut out = Vec::with_capacity(len);
                let mut ptrs: Vec<*mut $t> = Vec::new();

                for i in 0..len {
                    let elem = sexp.vector_elt(i as crate::ffi::R_xlen_t);
                    if elem.type_of() == SEXPTYPE::NILSXP {
                        out.push(None);
                        continue;
                    }
                    let value: &'static mut $t = TryFromSexp::try_from_sexp(elem)?;
                    let ptr = std::ptr::from_mut(value);
                    if ptrs.iter().any(|&p| p == ptr) {
                        return Err(SexpError::InvalidValue(
                            "list contains duplicate elements; cannot create multiple mutable references"
                                .to_string(),
                        ));
                    }
                    ptrs.push(ptr);
                    out.push(Some(value));
                }

                Ok(out)
            }
        }

        impl TryFromSexp for Vec<&'static [$t]> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let actual = sexp.type_of();
                if actual != SEXPTYPE::VECSXP {
                    return Err(SexpTypeError {
                        expected: SEXPTYPE::VECSXP,
                        actual,
                    }
                    .into());
                }

                let len = sexp.len();
                let mut out = Vec::with_capacity(len);

                for i in 0..len {
                    let elem = sexp.vector_elt(i as crate::ffi::R_xlen_t);
                    let slice: &'static [$t] =
                        TryFromSexp::try_from_sexp(elem).map_err(SexpError::from)?;
                    out.push(slice);
                }

                Ok(out)
            }
        }

        impl TryFromSexp for Vec<Option<&'static [$t]>> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let actual = sexp.type_of();
                if actual != SEXPTYPE::VECSXP {
                    return Err(SexpTypeError {
                        expected: SEXPTYPE::VECSXP,
                        actual,
                    }
                    .into());
                }

                let len = sexp.len();
                let mut out = Vec::with_capacity(len);

                for i in 0..len {
                    let elem = sexp.vector_elt(i as crate::ffi::R_xlen_t);
                    if elem.type_of() == SEXPTYPE::NILSXP {
                        out.push(None);
                    } else {
                        let slice: &'static [$t] =
                            TryFromSexp::try_from_sexp(elem).map_err(SexpError::from)?;
                        out.push(Some(slice));
                    }
                }

                Ok(out)
            }
        }

        impl TryFromSexp for Vec<&'static mut [$t]> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let actual = sexp.type_of();
                if actual != SEXPTYPE::VECSXP {
                    return Err(SexpTypeError {
                        expected: SEXPTYPE::VECSXP,
                        actual,
                    }
                    .into());
                }

                let len = sexp.len();
                let mut out = Vec::with_capacity(len);
                let mut ptrs: Vec<*mut $t> = Vec::new();

                for i in 0..len {
                    let elem = sexp.vector_elt(i as crate::ffi::R_xlen_t);
                    let slice: &'static mut [$t] =
                        TryFromSexp::try_from_sexp(elem).map_err(SexpError::from)?;
                    if !slice.is_empty() {
                        let ptr = slice.as_mut_ptr();
                        if ptrs.iter().any(|&p| p == ptr) {
                            return Err(SexpError::InvalidValue(
                                "list contains duplicate elements; cannot create multiple mutable references"
                                    .to_string(),
                            ));
                        }
                        ptrs.push(ptr);
                    }
                    out.push(slice);
                }

                Ok(out)
            }
        }

        impl TryFromSexp for Vec<Option<&'static mut [$t]>> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let actual = sexp.type_of();
                if actual != SEXPTYPE::VECSXP {
                    return Err(SexpTypeError {
                        expected: SEXPTYPE::VECSXP,
                        actual,
                    }
                    .into());
                }

                let len = sexp.len();
                let mut out = Vec::with_capacity(len);
                let mut ptrs: Vec<*mut $t> = Vec::new();

                for i in 0..len {
                    let elem = sexp.vector_elt(i as crate::ffi::R_xlen_t);
                    if elem.type_of() == SEXPTYPE::NILSXP {
                        out.push(None);
                        continue;
                    }
                    let slice: &'static mut [$t] =
                        TryFromSexp::try_from_sexp(elem).map_err(SexpError::from)?;
                    if !slice.is_empty() {
                        let ptr = slice.as_mut_ptr();
                        if ptrs.iter().any(|&p| p == ptr) {
                            return Err(SexpError::InvalidValue(
                                "list contains duplicate elements; cannot create multiple mutable references"
                                    .to_string(),
                            ));
                        }
                        ptrs.push(ptr);
                    }
                    out.push(Some(slice));
                }

                Ok(out)
            }
        }
    };
}

impl_ref_conversions_for!(i32);
impl_ref_conversions_for!(f64);
impl_ref_conversions_for!(u8);
impl_ref_conversions_for!(RLogical);
impl_ref_conversions_for!(crate::ffi::Rcomplex);
// endregion
