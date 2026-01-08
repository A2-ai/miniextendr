//! Integration with the `bitflags` crate.
//!
//! Provides conversions between R integers and bitflags types.
//!
//! # Features
//!
//! Enable this module with the `bitflags` feature:
//!
//! ```toml
//! [dependencies]
//! miniextendr-api = { version = "0.1", features = ["bitflags"] }
//! ```
//!
//! # Usage
//!
//! Since `bitflags!` generates unique types, this module provides wrapper types
//! and helper functions rather than blanket implementations.
//!
//! ## Wrapper Type
//!
//! Use [`RFlags<T>`] to wrap any bitflags type for R interop:
//!
//! ```ignore
//! use bitflags::bitflags;
//! use miniextendr_api::bitflags_impl::RFlags;
//!
//! bitflags! {
//!     #[derive(Clone, Copy, Debug)]
//!     pub struct Permissions: u32 {
//!         const READ = 0b001;
//!         const WRITE = 0b010;
//!         const EXECUTE = 0b100;
//!     }
//! }
//!
//! #[miniextendr]
//! fn check_permission(flags: RFlags<Permissions>, perm: i32) -> bool {
//!     let check = Permissions::from_bits_truncate(perm as u32);
//!     flags.contains(check)
//! }
//! ```
//!
//! ## Direct Conversion
//!
//! For manual control, use the helper functions:
//!
//! ```ignore
//! use miniextendr_api::bitflags_impl::{flags_from_i32, flags_to_i32};
//!
//! let flags: Permissions = flags_from_i32(0b011)?;
//! let int: i32 = flags_to_i32(flags);
//! ```
//!
//! # Bit Width
//!
//! R integers are 32-bit signed (`i32`). This module requires that flag values
//! fit within `i32::MAX` (0x7FFFFFFF). Use [`RFlags64<T>`] for 64-bit flags.

pub use bitflags::Flags;

use crate::altrep_traits::NA_INTEGER;
use crate::ffi::{SEXP, SEXPTYPE, SexpExt};
use crate::from_r::{SexpError, SexpLengthError, SexpTypeError, TryFromSexp};
use crate::into_r::IntoR;
use std::fmt;
use std::ops::{BitAnd, BitOr, BitXor, Deref, Not};

// =============================================================================
// RFlags<T> wrapper type
// =============================================================================

/// Wrapper for bitflags types that implements R conversions.
///
/// `RFlags<T>` wraps any type `T` that implements `bitflags::Flags` and provides
/// `TryFromSexp` and `IntoR` implementations for R interop.
///
/// # Example
///
/// ```ignore
/// bitflags! {
///     #[derive(Clone, Copy, Debug)]
///     pub struct Options: u8 {
///         const VERBOSE = 0b0001;
///         const DEBUG = 0b0010;
///     }
/// }
///
/// #[miniextendr]
/// fn process(opts: RFlags<Options>) -> String {
///     if opts.contains(Options::VERBOSE) {
///         "verbose mode".to_string()
///     } else {
///         "quiet mode".to_string()
///     }
/// }
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct RFlags<T: Flags>(pub T);

impl<T: Flags> RFlags<T> {
    /// Create a new `RFlags` wrapper.
    #[inline]
    pub fn new(flags: T) -> Self {
        RFlags(flags)
    }

    /// Get the wrapped flags.
    #[inline]
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T: Flags> Deref for RFlags<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Flags> From<T> for RFlags<T> {
    fn from(flags: T) -> Self {
        RFlags(flags)
    }
}

impl<T: Flags + fmt::Debug> fmt::Debug for RFlags<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("RFlags").field(&self.0).finish()
    }
}

impl<T: Flags + fmt::Display> fmt::Display for RFlags<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Forward bitwise operations
impl<T: Flags + BitAnd<Output = T>> BitAnd for RFlags<T> {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        RFlags(self.0 & rhs.0)
    }
}

impl<T: Flags + BitOr<Output = T>> BitOr for RFlags<T> {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        RFlags(self.0 | rhs.0)
    }
}

impl<T: Flags + BitXor<Output = T>> BitXor for RFlags<T> {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        RFlags(self.0 ^ rhs.0)
    }
}

impl<T: Flags + Not<Output = T>> Not for RFlags<T> {
    type Output = Self;
    fn not(self) -> Self::Output {
        RFlags(!self.0)
    }
}

// =============================================================================
// TryFromSexp / IntoR for RFlags<T>
// =============================================================================

impl<T> TryFromSexp for RFlags<T>
where
    T: Flags,
    T::Bits: TryFrom<i32>,
{
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        use crate::ffi::INTEGER_ELT;

        let actual = sexp.type_of();
        if actual != SEXPTYPE::INTSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::INTSXP,
                actual,
            }
            .into());
        }

        let len = sexp.len();
        if len != 1 {
            return Err(SexpError::Length(SexpLengthError {
                expected: 1,
                actual: len,
            }));
        }

        let int_val = unsafe { INTEGER_ELT(sexp, 0) };
        if int_val == NA_INTEGER {
            return Err(SexpError::InvalidValue(
                "NA not allowed for bitflags".to_string(),
            ));
        }

        // Convert i32 to the flags' Bits type
        let bits = T::Bits::try_from(int_val).map_err(|_| {
            SexpError::InvalidValue(format!("value {} out of range for bitflags", int_val))
        })?;

        // Try strict conversion (reject unknown bits)
        T::from_bits(bits).map(RFlags).ok_or_else(|| {
            SexpError::InvalidValue(format!("invalid bits 0x{:x} for flags", int_val))
        })
    }
}

impl<T> TryFromSexp for Option<RFlags<T>>
where
    T: Flags,
    T::Bits: TryFrom<i32>,
{
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        use crate::ffi::INTEGER_ELT;

        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }

        let actual = sexp.type_of();
        if actual != SEXPTYPE::INTSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::INTSXP,
                actual,
            }
            .into());
        }

        let len = sexp.len();
        if len != 1 {
            return Err(SexpError::Length(SexpLengthError {
                expected: 1,
                actual: len,
            }));
        }

        let int_val = unsafe { INTEGER_ELT(sexp, 0) };
        if int_val == NA_INTEGER {
            return Ok(None);
        }

        let bits = T::Bits::try_from(int_val).map_err(|_| {
            SexpError::InvalidValue(format!("value {} out of range for bitflags", int_val))
        })?;

        T::from_bits(bits)
            .map(RFlags)
            .ok_or_else(|| SexpError::InvalidValue(format!("invalid bits 0x{:x}", int_val)))
            .map(Some)
    }
}

impl<T> TryFromSexp for Vec<RFlags<T>>
where
    T: Flags,
    T::Bits: TryFrom<i32>,
{
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::INTSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::INTSXP,
                actual,
            }
            .into());
        }

        let slice: &[i32] = unsafe { sexp.as_slice() };
        slice
            .iter()
            .enumerate()
            .map(|(i, &int_val)| {
                if int_val == NA_INTEGER {
                    return Err(SexpError::InvalidValue(format!(
                        "NA at index {} not allowed for bitflags",
                        i
                    )));
                }

                let bits = T::Bits::try_from(int_val).map_err(|_| {
                    SexpError::InvalidValue(format!(
                        "value {} out of range for bitflags at index {}",
                        int_val, i
                    ))
                })?;

                T::from_bits(bits).map(RFlags).ok_or_else(|| {
                    SexpError::InvalidValue(format!("invalid bits 0x{:x} at index {}", int_val, i))
                })
            })
            .collect()
    }
}

impl<T> TryFromSexp for Vec<Option<RFlags<T>>>
where
    T: Flags,
    T::Bits: TryFrom<i32>,
{
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::INTSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::INTSXP,
                actual,
            }
            .into());
        }

        let slice: &[i32] = unsafe { sexp.as_slice() };
        slice
            .iter()
            .enumerate()
            .map(|(i, &int_val)| {
                if int_val == NA_INTEGER {
                    return Ok(None);
                }

                let bits = T::Bits::try_from(int_val).map_err(|_| {
                    SexpError::InvalidValue(format!(
                        "value {} out of range for bitflags at index {}",
                        int_val, i
                    ))
                })?;

                T::from_bits(bits)
                    .map(RFlags)
                    .ok_or_else(|| {
                        SexpError::InvalidValue(format!(
                            "invalid bits 0x{:x} at index {}",
                            int_val, i
                        ))
                    })
                    .map(Some)
            })
            .collect()
    }
}

impl<T> IntoR for RFlags<T>
where
    T: Flags,
    T::Bits: TryInto<i32>,
{
    fn into_sexp(self) -> SEXP {
        let bits = self.0.bits();
        match bits.try_into() {
            Ok(int_val) => int_val.into_sexp(),
            Err(_) => {
                // Value too large for i32 - return NA
                NA_INTEGER.into_sexp()
            }
        }
    }
}

impl<T> IntoR for Option<RFlags<T>>
where
    T: Flags,
    T::Bits: TryInto<i32>,
{
    fn into_sexp(self) -> SEXP {
        match self {
            Some(flags) => flags.into_sexp(),
            None => NA_INTEGER.into_sexp(),
        }
    }
}

impl<T> IntoR for Vec<RFlags<T>>
where
    T: Flags,
    T::Bits: TryInto<i32>,
{
    fn into_sexp(self) -> SEXP {
        let ints: Vec<i32> = self
            .into_iter()
            .map(|flags| flags.0.bits().try_into().unwrap_or(NA_INTEGER))
            .collect();
        ints.into_sexp()
    }
}

impl<T> IntoR for Vec<Option<RFlags<T>>>
where
    T: Flags,
    T::Bits: TryInto<i32>,
{
    fn into_sexp(self) -> SEXP {
        let ints: Vec<Option<i32>> = self
            .into_iter()
            .map(|opt| opt.and_then(|flags| flags.0.bits().try_into().ok()))
            .collect();
        ints.into_sexp()
    }
}

// =============================================================================
// Helper functions
// =============================================================================

/// Convert an `i32` to a bitflags type (strict - unknown bits cause error).
///
/// Returns `None` if the value contains bits not defined in the flags type.
///
/// # Example
///
/// ```ignore
/// let flags: Option<Permissions> = flags_from_i32_strict(0b011);
/// ```
#[inline]
pub fn flags_from_i32_strict<T>(value: i32) -> Option<T>
where
    T: Flags,
    T::Bits: TryFrom<i32>,
{
    let bits = T::Bits::try_from(value).ok()?;
    T::from_bits(bits)
}

/// Convert an `i32` to a bitflags type (truncating - unknown bits are ignored).
///
/// Returns empty flags if the value cannot be converted to the Bits type.
///
/// # Example
///
/// ```ignore
/// let flags: Permissions = flags_from_i32_truncate(0b111111);
/// // Only defined bits are kept
/// ```
#[inline]
pub fn flags_from_i32_truncate<T>(value: i32) -> T
where
    T: Flags,
    T::Bits: TryFrom<i32>,
{
    match T::Bits::try_from(value) {
        Ok(bits) => T::from_bits_truncate(bits),
        Err(_) => T::empty(),
    }
}

/// Convert a bitflags value to `i32`.
///
/// Returns `None` if the bits value doesn't fit in `i32`.
#[inline]
pub fn flags_to_i32<T>(flags: T) -> Option<i32>
where
    T: Flags,
    T::Bits: TryInto<i32>,
{
    flags.bits().try_into().ok()
}

// =============================================================================
// Unit tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use bitflags::bitflags;

    bitflags! {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        struct TestFlags: u8 {
            const A = 0b0001;
            const B = 0b0010;
            const C = 0b0100;
            const AB = Self::A.bits() | Self::B.bits();
        }
    }

    #[test]
    fn test_rflags_new() {
        let flags = RFlags::new(TestFlags::A | TestFlags::B);
        assert!(flags.contains(TestFlags::A));
        assert!(flags.contains(TestFlags::B));
        assert!(!flags.contains(TestFlags::C));
    }

    #[test]
    fn test_rflags_deref() {
        let flags = RFlags::new(TestFlags::A);
        // Deref allows calling methods on inner type
        assert!(flags.contains(TestFlags::A));
    }

    #[test]
    fn test_rflags_into_inner() {
        let flags = RFlags::new(TestFlags::AB);
        let inner = flags.into_inner();
        assert_eq!(inner, TestFlags::A | TestFlags::B);
    }

    #[test]
    fn test_rflags_bitwise_ops() {
        let a = RFlags::new(TestFlags::A);
        let b = RFlags::new(TestFlags::B);

        let or = a | b;
        assert!(or.contains(TestFlags::A));
        assert!(or.contains(TestFlags::B));

        let and = RFlags::new(TestFlags::AB) & a;
        assert!(and.contains(TestFlags::A));
        assert!(!and.contains(TestFlags::B));
    }

    #[test]
    fn test_flags_from_i32_strict() {
        let flags: Option<TestFlags> = flags_from_i32_strict(0b0011);
        assert_eq!(flags, Some(TestFlags::A | TestFlags::B));

        // Unknown bit should fail
        let invalid: Option<TestFlags> = flags_from_i32_strict(0b1000);
        assert_eq!(invalid, None);
    }

    #[test]
    fn test_flags_from_i32_truncate() {
        let flags: TestFlags = flags_from_i32_truncate(0b1111);
        // Only A, B, C are defined (0b0111)
        assert_eq!(flags, TestFlags::A | TestFlags::B | TestFlags::C);
    }

    #[test]
    fn test_flags_to_i32() {
        let flags = TestFlags::A | TestFlags::B;
        let int = flags_to_i32(flags);
        assert_eq!(int, Some(0b0011));
    }
}
