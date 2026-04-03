//! Large integer types → REALSXP (f64).
//!
//! R doesn't have native 64-bit integers. These types convert to f64 (REALSXP)
//! which may lose precision for values outside the "safe integer" range.

use crate::altrep_traits::{NA_INTEGER, NA_LOGICAL, NA_REAL};
use crate::into_r::IntoR;
//
// **Precision Loss Warning:**
// - f64 can exactly represent integers in range [-2^53, 2^53] (±9,007,199,254,740,992)
// - Values outside this range may be rounded to the nearest representable f64
// - This is silent - no error or warning is raised
//
// **Alternatives for exact 64-bit integers:**
// - Use the `bit64` R package (stores as REALSXP but interprets as int64)
// - Store as character strings and parse in R
// - Split into high/low 32-bit parts
//
// For most use cases (counters, IDs, timestamps), values fit within 2^53.

/// Convert `i64` to R integer (INTSXP) or numeric (REALSXP).
///
/// Uses smart conversion: values in `(i32::MIN, i32::MAX]` are returned as
/// R integers for exact representation. Values outside that range (including
/// `i32::MIN` which is `NA_integer_` in R) fall back to R doubles.
///
/// ```ignore
/// let small: i64 = 42;
/// small.into_sexp(); // R integer 42L
///
/// let big: i64 = 3_000_000_000;
/// big.into_sexp(); // R double 3e9
///
/// let na_trap: i64 = i32::MIN as i64;
/// na_trap.into_sexp(); // R double (not NA_integer_!)
/// ```
impl IntoR for i64 {
    type Error = std::convert::Infallible;
    #[inline]
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        // i32::MIN is NA_integer_ in R, so exclude it from the integer range
        if self > i32::MIN as i64 && self <= i32::MAX as i64 {
            // Range guard verified — cast is safe
            (self as i32).into_sexp()
        } else {
            // R has no 64-bit integer; f64 loses precision > 2^53
            (self as f64).into_sexp()
        }
    }
    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        if self > i32::MIN as i64 && self <= i32::MAX as i64 {
            unsafe { (self as i32).into_sexp_unchecked() }
        } else {
            unsafe { (self as f64).into_sexp_unchecked() }
        }
    }
}

/// Convert `u64` to R integer (INTSXP) or numeric (REALSXP).
///
/// Values in `[0, i32::MAX]` are returned as R integers. Larger values
/// fall back to R doubles (which may lose precision above 2^53).
impl IntoR for u64 {
    type Error = std::convert::Infallible;
    #[inline]
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        if self <= i32::MAX as u64 {
            (self as i32).into_sexp()
        } else {
            (self as f64).into_sexp()
        }
    }
    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        if self <= i32::MAX as u64 {
            unsafe { (self as i32).into_sexp_unchecked() }
        } else {
            unsafe { (self as f64).into_sexp_unchecked() }
        }
    }
}

/// Convert `isize` to R integer (INTSXP) or numeric (REALSXP).
///
/// On 64-bit platforms, uses the same smart conversion as [`i64`](impl IntoR for i64).
/// On 32-bit platforms, `isize` fits in i32 so conversion is always exact.
impl IntoR for isize {
    type Error = std::convert::Infallible;
    #[inline]
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok((self as i64).into_sexp())
    }
    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        (self as i64).into_sexp()
    }
    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe { (self as i64).into_sexp_unchecked() }
    }
}

/// Convert `usize` to R integer (INTSXP) or numeric (REALSXP).
///
/// Values in `[0, i32::MAX]` are returned as R integers. Larger values
/// fall back to R doubles.
impl IntoR for usize {
    type Error = std::convert::Infallible;
    #[inline]
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok((self as u64).into_sexp())
    }
    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        (self as u64).into_sexp()
    }
    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe { (self as u64).into_sexp_unchecked() }
    }
}

/// Macro for logical IntoR via Rf_ScalarLogical with conversion to i32.
macro_rules! impl_logical_into_r {
    ($ty:ty, $to_i32:expr) => {
        impl IntoR for $ty {
            type Error = std::convert::Infallible;
            #[inline]
            fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
                Ok(unsafe { crate::ffi::Rf_ScalarLogical($to_i32(self)) })
            }
            #[inline]
            unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
                Ok(unsafe { self.into_sexp_unchecked() })
            }
            #[inline]
            fn into_sexp(self) -> crate::ffi::SEXP {
                unsafe { crate::ffi::Rf_ScalarLogical($to_i32(self)) }
            }
            #[inline]
            unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
                unsafe { crate::ffi::Rf_ScalarLogical_unchecked($to_i32(self)) }
            }
        }
    };
}

impl_logical_into_r!(bool, |v: bool| i32::from(v));
impl_logical_into_r!(crate::ffi::Rboolean, |v: crate::ffi::Rboolean| v as i32);
impl_logical_into_r!(crate::ffi::RLogical, crate::ffi::RLogical::to_i32);

impl IntoR for Option<i32> {
    type Error = std::convert::Infallible;
    #[inline]
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => v.into_sexp(),
            None => unsafe { crate::ffi::Rf_ScalarInteger(NA_INTEGER) },
        }
    }
    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => unsafe { v.into_sexp_unchecked() },
            None => unsafe { crate::ffi::Rf_ScalarInteger_unchecked(NA_INTEGER) },
        }
    }
}

impl IntoR for Option<f64> {
    type Error = std::convert::Infallible;
    #[inline]
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => v.into_sexp(),
            None => unsafe { crate::ffi::Rf_ScalarReal(NA_REAL) },
        }
    }
    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => unsafe { v.into_sexp_unchecked() },
            None => unsafe { crate::ffi::Rf_ScalarReal_unchecked(NA_REAL) },
        }
    }
}

impl IntoR for Option<crate::ffi::Rboolean> {
    type Error = std::convert::Infallible;
    #[inline]
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        match self {
            // Rboolean is repr(i32), `as i32` is a no-op transmute
            Some(v) => unsafe { crate::ffi::Rf_ScalarLogical(v as i32) },
            None => unsafe { crate::ffi::Rf_ScalarLogical(NA_LOGICAL) },
        }
    }
    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => unsafe { crate::ffi::Rf_ScalarLogical_unchecked(v as i32) },
            None => unsafe { crate::ffi::Rf_ScalarLogical_unchecked(NA_LOGICAL) },
        }
    }
}

impl IntoR for Option<crate::ffi::RLogical> {
    type Error = std::convert::Infallible;
    #[inline]
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => unsafe { crate::ffi::Rf_ScalarLogical(v.to_i32()) },
            None => unsafe { crate::ffi::Rf_ScalarLogical(NA_LOGICAL) },
        }
    }
    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => unsafe { crate::ffi::Rf_ScalarLogical_unchecked(v.to_i32()) },
            None => unsafe { crate::ffi::Rf_ScalarLogical_unchecked(NA_LOGICAL) },
        }
    }
}

impl IntoR for Option<bool> {
    type Error = std::convert::Infallible;
    #[inline]
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => v.into_sexp(),
            None => unsafe { crate::ffi::Rf_ScalarLogical(NA_LOGICAL) },
        }
    }
    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        match self {
            Some(v) => unsafe { v.into_sexp_unchecked() },
            None => unsafe { crate::ffi::Rf_ScalarLogical_unchecked(NA_LOGICAL) },
        }
    }
}

/// Macro for NA-aware `Option<T> → R` smart scalar conversion.
/// Checks if value fits i32 → INTSXP with NA_INTEGER for None,
/// otherwise REALSXP with NA_REAL for None.
macro_rules! impl_option_smart_i64_into_r {
    ($t:ty, $fits_i32:expr) => {
        impl IntoR for Option<$t> {
            type Error = std::convert::Infallible;
            #[inline]
            fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
                Ok(self.into_sexp())
            }
            #[inline]
            unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
                self.try_into_sexp()
            }
            #[inline]
            fn into_sexp(self) -> crate::ffi::SEXP {
                match self {
                    Some(x) if $fits_i32(x) => (x as i32).into_sexp(),
                    Some(x) => (x as f64).into_sexp(),
                    None => unsafe { crate::ffi::Rf_ScalarInteger(NA_INTEGER) },
                }
            }
        }
    };
}

impl_option_smart_i64_into_r!(i64, |x: i64| x > i32::MIN as i64 && x <= i32::MAX as i64);
impl_option_smart_i64_into_r!(u64, |x: u64| x <= i32::MAX as u64);
impl_option_smart_i64_into_r!(isize, |x: isize| x > i32::MIN as isize
    && x <= i32::MAX as isize);
impl_option_smart_i64_into_r!(usize, |x: usize| x <= i32::MAX as usize);

/// Macro for `Option<T>` where `T` coerces to a type with existing Option impl.
macro_rules! impl_option_coerce_into_r {
    ($from:ty => $to:ty) => {
        impl IntoR for Option<$from> {
            type Error = std::convert::Infallible;
            #[inline]
            fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
                Ok(self.map(|x| x as $to).into_sexp())
            }
            #[inline]
            unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
                self.try_into_sexp()
            }
            #[inline]
            fn into_sexp(self) -> crate::ffi::SEXP {
                self.map(|x| x as $to).into_sexp()
            }
        }
    };
}

impl_option_coerce_into_r!(i8 => i32);
impl_option_coerce_into_r!(i16 => i32);
impl_option_coerce_into_r!(u16 => i32);
impl_option_coerce_into_r!(u32 => i64); // delegates to smart i64 path
impl_option_coerce_into_r!(f32 => f64);

impl<T: crate::externalptr::TypedExternal> IntoR for crate::externalptr::ExternalPtr<T> {
    type Error = std::convert::Infallible;
    #[inline]
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.as_sexp())
    }
    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.as_sexp())
    }
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        self.as_sexp()
    }
}

/// Blanket impl: Types marked with `IntoExternalPtr` get automatic `IntoR`.
///
/// This wraps the value in `ExternalPtr<T>` automatically, so you can return
/// `MyType` directly from `#[miniextendr]` functions instead of `ExternalPtr<MyType>`.
impl<T: crate::externalptr::IntoExternalPtr> IntoR for T {
    type Error = std::convert::Infallible;
    #[inline]
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        crate::externalptr::ExternalPtr::new(self).into_sexp()
    }
    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe { crate::externalptr::ExternalPtr::new_unchecked(self).into_sexp() }
    }
}

/// Helper to convert a string slice to R CHARSXP.
/// Uses UTF-8 encoding. Empty strings return R_BlankString (static, no allocation).
#[inline]
pub(crate) fn str_to_charsxp(s: &str) -> crate::ffi::SEXP {
    unsafe {
        if s.is_empty() {
            crate::ffi::R_BlankString
        } else {
            let _len: i32 = s.len().try_into().expect("string exceeds i32::MAX bytes");
            crate::ffi::SEXP::charsxp(s)
        }
    }
}

/// Unchecked version of [`str_to_charsxp`].
#[inline]
pub(crate) unsafe fn str_to_charsxp_unchecked(s: &str) -> crate::ffi::SEXP {
    unsafe {
        if s.is_empty() {
            crate::ffi::R_BlankString
        } else {
            let len: i32 = s.len().try_into().expect("string exceeds i32::MAX bytes");
            crate::ffi::Rf_mkCharLenCE_unchecked(s.as_ptr().cast(), len, crate::ffi::CE_UTF8)
        }
    }
}

impl IntoR for String {
    type Error = crate::into_r_error::IntoRError;
    #[inline]
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        self.as_str().try_into_sexp()
    }
    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        self.as_str().into_sexp()
    }
    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe { self.as_str().into_sexp_unchecked() }
    }
}

impl IntoR for char {
    type Error = std::convert::Infallible;
    #[inline]
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        // Convert char to a single-character string — always ≤ 4 bytes, cannot overflow i32
        let mut buf = [0u8; 4];
        let s = self.encode_utf8(&mut buf);
        unsafe {
            let charsxp = str_to_charsxp(s);
            crate::ffi::Rf_ScalarString(charsxp)
        }
    }
    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        let mut buf = [0u8; 4];
        let s = self.encode_utf8(&mut buf);
        unsafe { s.into_sexp_unchecked() }
    }
}

impl IntoR for &str {
    type Error = crate::into_r_error::IntoRError;
    #[inline]
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        let _len = i32::try_from(self.len())
            .map_err(|_| crate::into_r_error::IntoRError::StringTooLong { len: self.len() })?;
        Ok(unsafe {
            let charsxp = str_to_charsxp(self);
            crate::ffi::Rf_ScalarString(charsxp)
        })
    }
    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe {
            let charsxp = str_to_charsxp(self);
            crate::ffi::Rf_ScalarString(charsxp)
        }
    }
    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe {
            let charsxp = str_to_charsxp_unchecked(self);
            crate::ffi::Rf_ScalarString_unchecked(charsxp)
        }
    }
}

impl IntoR for Option<&str> {
    type Error = crate::into_r_error::IntoRError;
    #[inline]
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        match self {
            Some(s) => {
                let _len = i32::try_from(s.len())
                    .map_err(|_| crate::into_r_error::IntoRError::StringTooLong { len: s.len() })?;
                Ok(unsafe {
                    let charsxp = str_to_charsxp(s);
                    crate::ffi::Rf_ScalarString(charsxp)
                })
            }
            None => Ok(unsafe { crate::ffi::Rf_ScalarString(crate::ffi::R_NaString) }),
        }
    }
    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        unsafe {
            let charsxp = match self {
                Some(s) => str_to_charsxp(s),
                None => crate::ffi::R_NaString,
            };
            crate::ffi::Rf_ScalarString(charsxp)
        }
    }
    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe {
            let charsxp = match self {
                Some(s) => str_to_charsxp_unchecked(s),
                None => crate::ffi::R_NaString,
            };
            crate::ffi::Rf_ScalarString_unchecked(charsxp)
        }
    }
}

impl IntoR for Option<String> {
    type Error = crate::into_r_error::IntoRError;
    #[inline]
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        self.as_deref().try_into_sexp()
    }
    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        self.as_deref().into_sexp()
    }
    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe { self.as_deref().into_sexp_unchecked() }
    }
}

/// Convert `Option<&T>` to R SEXP by copying the value.
///
/// - `Some(&v)` → copies `v` and converts to R
/// - `None` → returns `NULL` (R_NilValue)
///
/// Note: This returns NULL for None, not NA, since there's no reference to return.
/// Use `Option<T>` directly if you want NA semantics for scalar types.
impl<T> IntoR for Option<&T>
where
    T: Copy + IntoR,
{
    type Error = crate::into_r_error::IntoRError;
    #[inline]
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        match self {
            Some(&v) => v
                .try_into_sexp()
                .map_err(|e| crate::into_r_error::IntoRError::Inner(e.to_string())),
            None => Ok(crate::ffi::SEXP::null()),
        }
    }
    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        match self {
            Some(&v) => v.into_sexp(),
            None => crate::ffi::SEXP::null(),
        }
    }
    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        match self {
            Some(&v) => unsafe { v.into_sexp_unchecked() },
            None => crate::ffi::SEXP::null(),
        }
    }
}

// endregion
