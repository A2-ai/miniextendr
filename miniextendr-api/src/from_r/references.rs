//! Reference conversions (borrowed views into R vectors).
//!
//! Provides zero-copy access to R vector data through references.
//! Callers must keep the source SEXP rooted for the entire reference lifetime.
//!
//! Covers: `&T`, `&mut T`, `Option<&T>`, `Vec<&T>`, `Vec<&[T]>`, and
//! mutable variants for all `RNativeType` types.
//!
//! # Tradeoff
//!
//! Prefer borrowed `&[T]` over owned `Vec<T>` when you only need read access —
//! this is the fast path (no allocation, no copy). Reach for `&mut [T]` only
//! when you genuinely need to mutate R-owned data in place; the R caller will
//! observe those writes (ALTREP MAYBE_REFERENCED rules still apply). Failure
//! mode: taking `&mut [T]` from a shared SEXP that R has handed to multiple
//! callers writes through that aliased buffer.
//!
//! For NA-aware reads use [`crate::from_r::na_vectors`] (`Vec<Option<T>>`) —
//! borrowed slices cannot express NA without losing the sentinel encoding.
//! Outbound borrowed slices copy into R-owned storage through [`crate::IntoR`].
//! Mutable list inputs preflight all raw ranges before creating any references;
//! this does not establish the root lifetime or exclude borrows held elsewhere
//! by a direct caller (the remaining lifetime-boundary work is #1427).

use crate::from_r::{
    NativeBorrow, SexpError, SexpLengthError, SexpTypeError, TryFromSexp, map_vecsxp_batched,
    map_vecsxp_with, r_slice_mut,
};
use crate::{RLogical, RNativeType, SEXP, SEXPTYPE, SexpExt};

/// An unborrowed native buffer. No Rust reference exists during list preflight.
#[derive(Clone, Copy)]
struct MutableRange<T> {
    data: *mut T,
    len: usize,
}

impl<T> MutableRange<T> {
    fn overlaps(&self, other: &Self) -> bool {
        if self.len == 0 || other.len == 0 {
            return false;
        }
        let a = self.data.addr();
        let b = other.data.addr();
        // Each range describes an existing R allocation. Compare integer
        // addresses only; retain the original mutable pointer for borrowing.
        if a <= b {
            b - a < self.len * core::mem::size_of::<T>()
        } else {
            a - b < other.len * core::mem::size_of::<T>()
        }
    }
}

/// Validate every element and every raw buffer before creating any references.
/// Different ALTREP objects can expose identical or partially overlapping
/// storage, so comparing SEXP identities (or just starting pointers) is not
/// enough here. NULL is a missing element only for the optional forms.
fn map_mutable_list<T: RNativeType, U>(
    sexp: SEXP,
    scalar: bool,
    optional: bool,
    map: impl FnMut(Option<MutableRange<T>>) -> U,
) -> Result<Vec<U>, SexpError> {
    let mut ranges: Vec<MutableRange<T>> = Vec::new();
    let raw = map_vecsxp_batched(sexp, |element| {
        let actual = element.type_of();
        if optional && actual == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        if actual != T::SEXP_TYPE {
            return Err(SexpTypeError {
                expected: T::SEXP_TYPE,
                actual,
            }
            .into());
        }
        let len = element.len();
        if scalar && len != 1 {
            return Err(SexpLengthError {
                expected: 1,
                actual: len,
            }
            .into());
        }
        let range = MutableRange {
            data: unsafe { T::dataptr_mut(element) },
            len,
        };
        let overlaps = ranges.iter().any(|previous| range.overlaps(previous));
        ranges.push(range);
        if overlaps {
            Err(SexpError::InvalidValue(
                "list contains overlapping native buffers; cannot create multiple mutable references"
                    .to_string(),
            ))
        } else {
            Ok(Some(range))
        }
    })?;
    // Only Rust allocations remain; a failed preflight never constructs &mut.
    Ok(raw.into_iter().map(map).collect())
}

macro_rules! impl_ref_conversions_for {
    ($t:ty) => {
        impl<'a> TryFromSexp for &'a $t {
            const NATIVE_BORROW: Option<NativeBorrow> =
                Some(NativeBorrow::scalar(<$t as RNativeType>::SEXP_TYPE, false));

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
                unsafe { sexp.as_slice::<$t>() }.first().ok_or_else(|| {
                    SexpLengthError {
                        expected: 1,
                        actual: 0,
                    }
                    .into()
                })
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
                    .ok_or_else(|| {
                        SexpLengthError {
                            expected: 1,
                            actual: 0,
                        }
                        .into()
                    })
            }
        }

        /// # Safety note (aliasing)
        ///
        /// This impl can produce aliased `&mut` references if the same R object
        /// is passed to multiple mutable parameters. The caller (generated wrapper)
        /// rejects overlapping native mutable/shared borrows before conversion.
        /// Direct conversion callers must enforce this borrowing contract themselves.
        impl<'a> TryFromSexp for &'a mut $t {
            const NATIVE_BORROW: Option<NativeBorrow> =
                Some(NativeBorrow::scalar(<$t as RNativeType>::SEXP_TYPE, true));

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

        impl<'a> TryFromSexp for Option<&'a $t> {
            const NATIVE_BORROW: Option<NativeBorrow> = <&'a $t as TryFromSexp>::NATIVE_BORROW;

            type Error = SexpError;

            #[inline]
            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                if sexp.type_of() == SEXPTYPE::NILSXP {
                    return Ok(None);
                }
                let value: &'a $t = TryFromSexp::try_from_sexp(sexp)?;
                Ok(Some(value))
            }

            #[inline]
            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                if sexp.type_of() == SEXPTYPE::NILSXP {
                    return Ok(None);
                }
                let value: &'a $t = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
                Ok(Some(value))
            }
        }

        impl<'a> TryFromSexp for Option<&'a mut $t> {
            const NATIVE_BORROW: Option<NativeBorrow> = <&'a mut $t as TryFromSexp>::NATIVE_BORROW;

            type Error = SexpError;

            #[inline]
            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                if sexp.type_of() == SEXPTYPE::NILSXP {
                    return Ok(None);
                }
                let value: &'a mut $t = TryFromSexp::try_from_sexp(sexp)?;
                Ok(Some(value))
            }

            #[inline]
            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                if sexp.type_of() == SEXPTYPE::NILSXP {
                    return Ok(None);
                }
                let value: &'a mut $t = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
                Ok(Some(value))
            }
        }

        // Option<&[T]> and Option<&mut [T]> impls removed - now use blanket impls

        impl<'a> TryFromSexp for Vec<&'a $t> {
            const NATIVE_BORROW: Option<NativeBorrow> =
                NativeBorrow::in_list(<&'a $t as TryFromSexp>::NATIVE_BORROW);

            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                map_vecsxp_with(sexp, |_i, elem| TryFromSexp::try_from_sexp(elem))
            }
        }

        impl<'a> TryFromSexp for Vec<Option<&'a $t>> {
            const NATIVE_BORROW: Option<NativeBorrow> =
                NativeBorrow::in_list(<Option<&'a $t> as TryFromSexp>::NATIVE_BORROW);

            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                map_vecsxp_with(sexp, |_i, elem| {
                    if elem.type_of() == SEXPTYPE::NILSXP {
                        Ok(None)
                    } else {
                        let value: &'a $t = TryFromSexp::try_from_sexp(elem)?;
                        Ok(Some(value))
                    }
                })
            }
        }

        impl<'a> TryFromSexp for Vec<&'a mut $t> {
            const NATIVE_BORROW: Option<NativeBorrow> =
                NativeBorrow::in_list(<&'a mut $t as TryFromSexp>::NATIVE_BORROW);

            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                map_mutable_list::<$t, _>(sexp, true, false, |range| {
                    // Non-optional scalar ranges were checked above.
                    unsafe { &mut *range.unwrap().data }
                })
            }
        }

        impl<'a> TryFromSexp for Vec<Option<&'a mut $t>> {
            const NATIVE_BORROW: Option<NativeBorrow> =
                NativeBorrow::in_list(<Option<&'a mut $t> as TryFromSexp>::NATIVE_BORROW);

            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                map_mutable_list::<$t, _>(sexp, true, true, |range| {
                    range.map(|range| unsafe { &mut *range.data })
                })
            }
        }

        impl<'a> TryFromSexp for Vec<&'a [$t]> {
            const NATIVE_BORROW: Option<NativeBorrow> =
                NativeBorrow::in_list(<&'a [$t] as TryFromSexp>::NATIVE_BORROW);

            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                map_vecsxp_with(sexp, |_i, elem| {
                    TryFromSexp::try_from_sexp(elem).map_err(SexpError::from)
                })
            }
        }

        impl<'a> TryFromSexp for Vec<Option<&'a [$t]>> {
            const NATIVE_BORROW: Option<NativeBorrow> =
                NativeBorrow::in_list(<Option<&'a [$t]> as TryFromSexp>::NATIVE_BORROW);

            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                map_vecsxp_with(sexp, |_i, elem| {
                    if elem.type_of() == SEXPTYPE::NILSXP {
                        Ok(None)
                    } else {
                        let slice: &'a [$t] =
                            TryFromSexp::try_from_sexp(elem).map_err(SexpError::from)?;
                        Ok(Some(slice))
                    }
                })
            }
        }

        impl<'a> TryFromSexp for Vec<&'a mut [$t]> {
            const NATIVE_BORROW: Option<NativeBorrow> =
                NativeBorrow::in_list(<&'a mut [$t] as TryFromSexp>::NATIVE_BORROW);

            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                map_mutable_list::<$t, _>(sexp, false, false, |range| {
                    let range = range.unwrap();
                    unsafe { r_slice_mut(range.data, range.len) }
                })
            }
        }

        impl<'a> TryFromSexp for Vec<Option<&'a mut [$t]>> {
            const NATIVE_BORROW: Option<NativeBorrow> =
                NativeBorrow::in_list(<Option<&'a mut [$t]> as TryFromSexp>::NATIVE_BORROW);

            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                map_mutable_list::<$t, _>(sexp, false, true, |range| {
                    range.map(|range| unsafe { r_slice_mut(range.data, range.len) })
                })
            }
        }
    };
}

impl_ref_conversions_for!(i32);
impl_ref_conversions_for!(f64);
impl_ref_conversions_for!(u8);
impl_ref_conversions_for!(RLogical);
impl_ref_conversions_for!(crate::Rcomplex);
// endregion
