//! Logical type conversions (Rboolean, bool, Option variants).
//!
//! Handles the three R logical states (TRUE, FALSE, NA) and maps them to Rust:
//!
//! | Rust Type | NA Handling |
//! |-----------|-------------|
//! | `Rboolean` | Error on NA |
//! | `bool` | Error on NA |
//! | `Option<Rboolean>` | `None` on NA |
//! | `Option<bool>` | `None` on NA |

use crate::ffi::{RLogical, Rboolean, SEXP, SEXPTYPE, SexpExt};
use crate::from_r::{SexpError, SexpNaError, TryFromSexp, is_na_real};

impl TryFromSexp for Rboolean {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let raw: RLogical = TryFromSexp::try_from_sexp(sexp)?;
        match raw.to_option_bool() {
            Some(false) => Ok(Rboolean::FALSE),
            Some(true) => Ok(Rboolean::TRUE),
            None => Err(SexpNaError {
                sexp_type: SEXPTYPE::LGLSXP,
            }
            .into()),
        }
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        let raw: RLogical = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
        match raw.to_option_bool() {
            Some(false) => Ok(Rboolean::FALSE),
            Some(true) => Ok(Rboolean::TRUE),
            None => Err(SexpNaError {
                sexp_type: SEXPTYPE::LGLSXP,
            }
            .into()),
        }
    }
}

impl TryFromSexp for Option<Rboolean> {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        let raw: RLogical = TryFromSexp::try_from_sexp(sexp)?;
        match raw.to_option_bool() {
            Some(false) => Ok(Some(Rboolean::FALSE)),
            Some(true) => Ok(Some(Rboolean::TRUE)),
            None => Ok(None),
        }
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        let raw: RLogical = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
        match raw.to_option_bool() {
            Some(false) => Ok(Some(Rboolean::FALSE)),
            Some(true) => Ok(Some(Rboolean::TRUE)),
            None => Ok(None),
        }
    }
}

impl TryFromSexp for bool {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let raw: RLogical = TryFromSexp::try_from_sexp(sexp)?;
        raw.to_option_bool().ok_or_else(|| {
            SexpNaError {
                sexp_type: SEXPTYPE::LGLSXP,
            }
            .into()
        })
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        let raw: RLogical = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
        raw.to_option_bool().ok_or_else(|| {
            SexpNaError {
                sexp_type: SEXPTYPE::LGLSXP,
            }
            .into()
        })
    }
}

impl TryFromSexp for Option<bool> {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        // NULL -> None
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        let raw: RLogical = TryFromSexp::try_from_sexp(sexp)?;
        Ok(raw.to_option_bool())
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        // NULL -> None
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        let raw: RLogical = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
        Ok(raw.to_option_bool())
    }
}

impl TryFromSexp for Option<RLogical> {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        let raw: RLogical = TryFromSexp::try_from_sexp(sexp)?;
        if raw.is_na() { Ok(None) } else { Ok(Some(raw)) }
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        let raw: RLogical = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
        if raw.is_na() { Ok(None) } else { Ok(Some(raw)) }
    }
}

impl TryFromSexp for Option<i32> {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        // NULL -> None
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        // The i32 TryFromSexp impl now returns SexpNaError for NA_integer_.
        // Treat that as None here; propagate all other errors.
        match TryFromSexp::try_from_sexp(sexp) {
            Ok(value) => Ok(Some(value)),
            Err(SexpError::Na(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        // NULL -> None
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        match unsafe { TryFromSexp::try_from_sexp_unchecked(sexp) } {
            Ok(value) => Ok(Some(value)),
            Err(SexpError::Na(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

impl TryFromSexp for Option<f64> {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        // NULL -> None
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        let value: f64 = TryFromSexp::try_from_sexp(sexp)?;
        if is_na_real(value) {
            Ok(None)
        } else {
            Ok(Some(value))
        }
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        // NULL -> None
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        let value: f64 = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
        if is_na_real(value) {
            Ok(None)
        } else {
            Ok(Some(value))
        }
    }
}

impl TryFromSexp for Option<u8> {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        let value: u8 = TryFromSexp::try_from_sexp(sexp)?;
        Ok(Some(value))
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        let value: u8 = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
        Ok(Some(value))
    }
}

impl TryFromSexp for Option<crate::ffi::Rcomplex> {
    type Error = SexpError;

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        use crate::altrep_traits::NA_REAL;

        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        let value: crate::ffi::Rcomplex = TryFromSexp::try_from_sexp(sexp)?;
        let na_bits = NA_REAL.to_bits();
        if value.r.to_bits() == na_bits || value.i.to_bits() == na_bits {
            Ok(None)
        } else {
            Ok(Some(value))
        }
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        use crate::altrep_traits::NA_REAL;

        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        let value: crate::ffi::Rcomplex = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
        let na_bits = NA_REAL.to_bits();
        if value.r.to_bits() == na_bits || value.i.to_bits() == na_bits {
            Ok(None)
        } else {
            Ok(Some(value))
        }
    }
}
// endregion
