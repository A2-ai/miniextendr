//! Integration with the `num-bigint` crate.
//!
//! Provides conversions between R character vectors and `BigInt` / `BigUint`.
//!
//! # Features
//!
//! Enable this module with the `num-bigint` feature:
//!
//! ```toml
//! [dependencies]
//! miniextendr-api = { version = "0.1", features = ["num-bigint"] }
//! ```

pub use num_bigint::{BigInt, BigUint};

use crate::coerce::{Coerce, CoerceError, TryCoerce};
use crate::ffi::{SEXP, SEXPTYPE};
use crate::from_r::{SexpError, SexpNaError, TryFromSexp};
use crate::into_r::IntoR;
use std::str::FromStr;

// =============================================================================
// Coerce/TryCoerce impls for BigInt and BigUint
// =============================================================================

/// `i32` → `BigInt`: lossless
impl Coerce<BigInt> for i32 {
    #[inline(always)]
    fn coerce(self) -> BigInt {
        BigInt::from(self)
    }
}

/// `i64` → `BigInt`: lossless
impl Coerce<BigInt> for i64 {
    #[inline(always)]
    fn coerce(self) -> BigInt {
        BigInt::from(self)
    }
}

/// `u32` → `BigUint`: lossless
impl Coerce<BigUint> for u32 {
    #[inline(always)]
    fn coerce(self) -> BigUint {
        BigUint::from(self)
    }
}

/// `u64` → `BigUint`: lossless
impl Coerce<BigUint> for u64 {
    #[inline(always)]
    fn coerce(self) -> BigUint {
        BigUint::from(self)
    }
}

/// `f64` → `BigInt`: fallible, must be integer value (no fraction) and not NaN/Inf
impl TryCoerce<BigInt> for f64 {
    type Error = CoerceError;

    #[inline]
    fn try_coerce(self) -> Result<BigInt, CoerceError> {
        if self.is_nan() {
            return Err(CoerceError::NaN);
        }
        if self.is_infinite() {
            return Err(CoerceError::Overflow);
        }
        if self.fract() != 0.0 {
            return Err(CoerceError::PrecisionLoss);
        }
        // Convert via i64 if in range, otherwise use string for larger values
        if self >= i64::MIN as f64 && self <= i64::MAX as f64 {
            Ok(BigInt::from(self as i64))
        } else {
            // For very large values, convert via string representation
            let s = format!("{:.0}", self);
            BigInt::from_str(&s).map_err(|_| CoerceError::Overflow)
        }
    }
}

/// `f64` → `BigUint`: fallible, must be non-negative integer (no fraction, not NaN/Inf)
impl TryCoerce<BigUint> for f64 {
    type Error = CoerceError;

    #[inline]
    fn try_coerce(self) -> Result<BigUint, CoerceError> {
        if self.is_nan() {
            return Err(CoerceError::NaN);
        }
        if self.is_infinite() {
            return Err(CoerceError::Overflow);
        }
        if self < 0.0 {
            return Err(CoerceError::Overflow);
        }
        if self.fract() != 0.0 {
            return Err(CoerceError::PrecisionLoss);
        }
        // Convert via u64 if in range, otherwise use string for larger values
        if self <= u64::MAX as f64 {
            Ok(BigUint::from(self as u64))
        } else {
            let s = format!("{:.0}", self);
            BigUint::from_str(&s).map_err(|_| CoerceError::Overflow)
        }
    }
}

/// `BigInt` → `i32`: fallible, may not fit
impl TryCoerce<i32> for BigInt {
    type Error = CoerceError;

    #[inline]
    fn try_coerce(self) -> Result<i32, CoerceError> {
        use num_bigint::TryFromBigIntError;
        i32::try_from(self).map_err(|_: TryFromBigIntError<BigInt>| CoerceError::Overflow)
    }
}

/// `BigInt` → `i64`: fallible, may not fit
impl TryCoerce<i64> for BigInt {
    type Error = CoerceError;

    #[inline]
    fn try_coerce(self) -> Result<i64, CoerceError> {
        use num_bigint::TryFromBigIntError;
        i64::try_from(self).map_err(|_: TryFromBigIntError<BigInt>| CoerceError::Overflow)
    }
}

/// `BigUint` → `u32`: fallible, may not fit
impl TryCoerce<u32> for BigUint {
    type Error = CoerceError;

    #[inline]
    fn try_coerce(self) -> Result<u32, CoerceError> {
        use num_bigint::TryFromBigIntError;
        u32::try_from(self).map_err(|_: TryFromBigIntError<BigUint>| CoerceError::Overflow)
    }
}

/// `BigUint` → `u64`: fallible, may not fit
impl TryCoerce<u64> for BigUint {
    type Error = CoerceError;

    #[inline]
    fn try_coerce(self) -> Result<u64, CoerceError> {
        use num_bigint::TryFromBigIntError;
        u64::try_from(self).map_err(|_: TryFromBigIntError<BigUint>| CoerceError::Overflow)
    }
}

/// `BigInt` → `f64`: fallible, may lose precision for large values
impl TryCoerce<f64> for BigInt {
    type Error = CoerceError;

    #[inline]
    fn try_coerce(self) -> Result<f64, CoerceError> {
        // Check if value is within f64's exact integer representation range (2^53)
        const MAX_SAFE: i64 = 1 << 53;
        const MIN_SAFE: i64 = -(1 << 53);

        // Try to fit in i64 first for the range check
        if let Ok(i) = i64::try_from(self.clone()) {
            if (MIN_SAFE..=MAX_SAFE).contains(&i) {
                return Ok(i as f64);
            }
        }

        // For larger values, convert and check round-trip
        let f = self
            .to_string()
            .parse::<f64>()
            .map_err(|_| CoerceError::Overflow)?;
        let roundtrip = BigInt::from_str(&format!("{:.0}", f));
        if let Ok(rt) = roundtrip {
            if rt == self {
                return Ok(f);
            }
        }
        Err(CoerceError::PrecisionLoss)
    }
}

fn parse_bigint(s: &str) -> Result<BigInt, SexpError> {
    BigInt::from_str(s).map_err(|e| SexpError::InvalidValue(e.to_string()))
}

fn parse_biguint(s: &str) -> Result<BigUint, SexpError> {
    BigUint::from_str(s).map_err(|e| SexpError::InvalidValue(e.to_string()))
}

impl TryFromSexp for BigInt {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let s: Option<String> = TryFromSexp::try_from_sexp(sexp)?;
        let s = s.ok_or(SexpError::Na(SexpNaError {
            sexp_type: SEXPTYPE::STRSXP,
        }))?;
        parse_bigint(&s)
    }
}

impl TryFromSexp for BigUint {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let s: Option<String> = TryFromSexp::try_from_sexp(sexp)?;
        let s = s.ok_or(SexpError::Na(SexpNaError {
            sexp_type: SEXPTYPE::STRSXP,
        }))?;
        parse_biguint(&s)
    }
}

impl TryFromSexp for Option<BigInt> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let s: Option<String> = TryFromSexp::try_from_sexp(sexp)?;
        match s {
            Some(s) => parse_bigint(&s).map(Some),
            None => Ok(None),
        }
    }
}

impl TryFromSexp for Option<BigUint> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let s: Option<String> = TryFromSexp::try_from_sexp(sexp)?;
        match s {
            Some(s) => parse_biguint(&s).map(Some),
            None => Ok(None),
        }
    }
}

impl TryFromSexp for Vec<BigInt> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let values: Vec<Option<String>> = TryFromSexp::try_from_sexp(sexp)?;
        values
            .into_iter()
            .map(|opt| {
                let s = opt.ok_or(SexpError::Na(SexpNaError {
                    sexp_type: SEXPTYPE::STRSXP,
                }))?;
                parse_bigint(&s)
            })
            .collect()
    }
}

impl TryFromSexp for Vec<BigUint> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let values: Vec<Option<String>> = TryFromSexp::try_from_sexp(sexp)?;
        values
            .into_iter()
            .map(|opt| {
                let s = opt.ok_or(SexpError::Na(SexpNaError {
                    sexp_type: SEXPTYPE::STRSXP,
                }))?;
                parse_biguint(&s)
            })
            .collect()
    }
}

impl TryFromSexp for Vec<Option<BigInt>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let values: Vec<Option<String>> = TryFromSexp::try_from_sexp(sexp)?;
        values
            .into_iter()
            .map(|opt| match opt {
                Some(s) => parse_bigint(&s).map(Some),
                None => Ok(None),
            })
            .collect()
    }
}

impl TryFromSexp for Vec<Option<BigUint>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let values: Vec<Option<String>> = TryFromSexp::try_from_sexp(sexp)?;
        values
            .into_iter()
            .map(|opt| match opt {
                Some(s) => parse_biguint(&s).map(Some),
                None => Ok(None),
            })
            .collect()
    }
}

impl IntoR for BigInt {
    fn into_sexp(self) -> SEXP {
        self.to_string().into_sexp()
    }
}

impl IntoR for BigUint {
    fn into_sexp(self) -> SEXP {
        self.to_string().into_sexp()
    }
}

impl IntoR for Option<BigInt> {
    fn into_sexp(self) -> SEXP {
        self.map(|v| v.to_string()).into_sexp()
    }
}

impl IntoR for Option<BigUint> {
    fn into_sexp(self) -> SEXP {
        self.map(|v| v.to_string()).into_sexp()
    }
}

impl IntoR for Vec<BigInt> {
    fn into_sexp(self) -> SEXP {
        self.into_iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .into_sexp()
    }
}

impl IntoR for Vec<BigUint> {
    fn into_sexp(self) -> SEXP {
        self.into_iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .into_sexp()
    }
}

impl IntoR for Vec<Option<BigInt>> {
    fn into_sexp(self) -> SEXP {
        self.into_iter()
            .map(|v| v.map(|val| val.to_string()))
            .collect::<Vec<_>>()
            .into_sexp()
    }
}

impl IntoR for Vec<Option<BigUint>> {
    fn into_sexp(self) -> SEXP {
        self.into_iter()
            .map(|v| v.map(|val| val.to_string()))
            .collect::<Vec<_>>()
            .into_sexp()
    }
}

// =============================================================================
// RBigIntOps adapter trait
// =============================================================================

use num_bigint::Sign;

/// Adapter trait for [`BigInt`] operations.
///
/// Provides arbitrary-precision integer arithmetic from R.
/// Automatically implemented for `num_bigint::BigInt`.
///
/// # Example
///
/// ```rust,ignore
/// use num_bigint::BigInt;
/// use miniextendr_api::num_bigint_impl::RBigIntOps;
///
/// #[derive(ExternalPtr)]
/// struct MyBigInt(BigInt);
///
/// #[miniextendr]
/// impl RBigIntOps for MyBigInt {}
///
/// miniextendr_module! {
///     mod mymodule;
///     impl RBigIntOps for MyBigInt;
/// }
/// ```
///
/// In R:
/// ```r
/// x <- MyBigInt$from_str("12345678901234567890")
/// y <- MyBigInt$from_str("98765432109876543210")
/// x$add(y)$as_string()  # "111111111011111111100"
/// x$is_positive()       # TRUE
/// x$bit_length()        # 64
/// ```
pub trait RBigIntOps {
    /// Convert to string representation.
    fn as_string(&self) -> String;

    /// Check if zero.
    fn is_zero(&self) -> bool;

    /// Check if positive (> 0).
    fn is_positive(&self) -> bool;

    /// Check if negative (< 0).
    fn is_negative(&self) -> bool;

    /// Get the sign as an integer: -1, 0, or 1.
    fn sign(&self) -> i32;

    /// Get the number of bits needed to represent this number.
    fn bit_length(&self) -> i64;

    /// Get the absolute value.
    fn abs(&self) -> BigInt;

    /// Negate the value.
    fn neg(&self) -> BigInt;

    /// Add another BigInt (passed as string).
    fn add_str(&self, other: &str) -> Result<BigInt, String>;

    /// Subtract another BigInt (passed as string).
    fn sub_str(&self, other: &str) -> Result<BigInt, String>;

    /// Multiply by another BigInt (passed as string).
    fn mul_str(&self, other: &str) -> Result<BigInt, String>;

    /// Divide by another BigInt (passed as string).
    fn div_str(&self, other: &str) -> Result<BigInt, String>;

    /// Remainder after division (passed as string).
    fn rem_str(&self, other: &str) -> Result<BigInt, String>;

    /// Raise to a power (u32 exponent).
    fn pow(&self, exp: u32) -> BigInt;

    /// Get the greatest common divisor with another BigInt.
    fn gcd_str(&self, other: &str) -> Result<BigInt, String>;

    /// Convert to bytes (big-endian).
    fn to_bytes_be(&self) -> Vec<u8>;

    /// Convert to bytes (little-endian).
    fn to_bytes_le(&self) -> Vec<u8>;
}

impl RBigIntOps for BigInt {
    fn as_string(&self) -> String {
        ToString::to_string(self)
    }

    fn is_zero(&self) -> bool {
        BigInt::sign(self) == Sign::NoSign
    }

    fn is_positive(&self) -> bool {
        BigInt::sign(self) == Sign::Plus
    }

    fn is_negative(&self) -> bool {
        BigInt::sign(self) == Sign::Minus
    }

    fn sign(&self) -> i32 {
        match BigInt::sign(self) {
            Sign::Minus => -1,
            Sign::NoSign => 0,
            Sign::Plus => 1,
        }
    }

    fn bit_length(&self) -> i64 {
        self.bits() as i64
    }

    fn abs(&self) -> BigInt {
        if BigInt::sign(self) == Sign::Minus {
            -self.clone()
        } else {
            self.clone()
        }
    }

    fn neg(&self) -> BigInt {
        -self.clone()
    }

    fn add_str(&self, other: &str) -> Result<BigInt, String> {
        let other = BigInt::from_str(other).map_err(|e| e.to_string())?;
        Ok(self + other)
    }

    fn sub_str(&self, other: &str) -> Result<BigInt, String> {
        let other = BigInt::from_str(other).map_err(|e| e.to_string())?;
        Ok(self - other)
    }

    fn mul_str(&self, other: &str) -> Result<BigInt, String> {
        let other = BigInt::from_str(other).map_err(|e| e.to_string())?;
        Ok(self * other)
    }

    fn div_str(&self, other: &str) -> Result<BigInt, String> {
        let other = BigInt::from_str(other).map_err(|e| e.to_string())?;
        if other.sign() == Sign::NoSign {
            return Err("division by zero".to_string());
        }
        Ok(self / other)
    }

    fn rem_str(&self, other: &str) -> Result<BigInt, String> {
        let other = BigInt::from_str(other).map_err(|e| e.to_string())?;
        if other.sign() == Sign::NoSign {
            return Err("division by zero".to_string());
        }
        Ok(self % other)
    }

    fn pow(&self, exp: u32) -> BigInt {
        num_bigint::BigInt::pow(self, exp)
    }

    fn gcd_str(&self, other: &str) -> Result<BigInt, String> {
        let other = BigInt::from_str(other).map_err(|e| e.to_string())?;
        Ok(num_integer::Integer::gcd(self, &other))
    }

    fn to_bytes_be(&self) -> Vec<u8> {
        let (_, bytes) = self.to_bytes_be();
        bytes
    }

    fn to_bytes_le(&self) -> Vec<u8> {
        let (_, bytes) = self.to_bytes_le();
        bytes
    }
}

// =============================================================================
// RBigUintOps adapter trait
// =============================================================================

/// Adapter trait for [`BigUint`] operations.
///
/// Provides arbitrary-precision unsigned integer arithmetic from R.
/// Automatically implemented for `num_bigint::BigUint`.
pub trait RBigUintOps {
    /// Convert to string representation.
    fn as_string(&self) -> String;

    /// Check if zero.
    fn is_zero(&self) -> bool;

    /// Check if this is one.
    fn is_one(&self) -> bool;

    /// Get the number of bits needed to represent this number.
    fn bit_length(&self) -> i64;

    /// Add another BigUint (passed as string).
    fn add_str(&self, other: &str) -> Result<BigUint, String>;

    /// Subtract another BigUint (passed as string). Returns error if result would be negative.
    fn sub_str(&self, other: &str) -> Result<BigUint, String>;

    /// Multiply by another BigUint (passed as string).
    fn mul_str(&self, other: &str) -> Result<BigUint, String>;

    /// Divide by another BigUint (passed as string).
    fn div_str(&self, other: &str) -> Result<BigUint, String>;

    /// Remainder after division (passed as string).
    fn rem_str(&self, other: &str) -> Result<BigUint, String>;

    /// Raise to a power (u32 exponent).
    fn pow(&self, exp: u32) -> BigUint;

    /// Get the greatest common divisor with another BigUint.
    fn gcd_str(&self, other: &str) -> Result<BigUint, String>;

    /// Convert to bytes (big-endian).
    fn to_bytes_be(&self) -> Vec<u8>;

    /// Convert to bytes (little-endian).
    fn to_bytes_le(&self) -> Vec<u8>;
}

impl RBigUintOps for BigUint {
    fn as_string(&self) -> String {
        ToString::to_string(self)
    }

    fn is_zero(&self) -> bool {
        use num_bigint::ToBigUint;
        *self == 0u32.to_biguint().unwrap()
    }

    fn is_one(&self) -> bool {
        use num_bigint::ToBigUint;
        *self == 1u32.to_biguint().unwrap()
    }

    fn bit_length(&self) -> i64 {
        self.bits() as i64
    }

    fn add_str(&self, other: &str) -> Result<BigUint, String> {
        let other = BigUint::from_str(other).map_err(|e| e.to_string())?;
        Ok(self + other)
    }

    fn sub_str(&self, other: &str) -> Result<BigUint, String> {
        let other = BigUint::from_str(other).map_err(|e| e.to_string())?;
        if self < &other {
            return Err("subtraction would result in negative number".to_string());
        }
        Ok(self - other)
    }

    fn mul_str(&self, other: &str) -> Result<BigUint, String> {
        let other = BigUint::from_str(other).map_err(|e| e.to_string())?;
        Ok(self * other)
    }

    fn div_str(&self, other: &str) -> Result<BigUint, String> {
        use num_bigint::ToBigUint;
        let other = BigUint::from_str(other).map_err(|e| e.to_string())?;
        if other == 0u32.to_biguint().unwrap() {
            return Err("division by zero".to_string());
        }
        Ok(self / other)
    }

    fn rem_str(&self, other: &str) -> Result<BigUint, String> {
        use num_bigint::ToBigUint;
        let other = BigUint::from_str(other).map_err(|e| e.to_string())?;
        if other == 0u32.to_biguint().unwrap() {
            return Err("division by zero".to_string());
        }
        Ok(self % other)
    }

    fn pow(&self, exp: u32) -> BigUint {
        num_bigint::BigUint::pow(self, exp)
    }

    fn gcd_str(&self, other: &str) -> Result<BigUint, String> {
        let other = BigUint::from_str(other).map_err(|e| e.to_string())?;
        Ok(num_integer::Integer::gcd(self, &other))
    }

    fn to_bytes_be(&self) -> Vec<u8> {
        BigUint::to_bytes_be(self)
    }

    fn to_bytes_le(&self) -> Vec<u8> {
        BigUint::to_bytes_le(self)
    }
}

// =============================================================================
// RBigIntBitOps adapter trait
// =============================================================================

/// Adapter trait for [`BigInt`] bitwise operations.
///
/// Provides bitwise operations on arbitrary-precision integers from R.
/// Note: Bitwise operations on negative BigInt use two's complement representation.
///
/// # Example
///
/// ```rust,ignore
/// use num_bigint::BigInt;
/// use miniextendr_api::num_bigint_impl::RBigIntBitOps;
///
/// #[derive(ExternalPtr)]
/// struct MyBigInt(BigInt);
///
/// #[miniextendr]
/// impl RBigIntBitOps for MyBigInt {}
///
/// miniextendr_module! {
///     mod mymodule;
///     impl RBigIntBitOps for MyBigInt;
/// }
/// ```
///
/// In R:
/// ```r
/// x <- MyBigInt$from_str("255")
/// x$bit_and_str("15")$as_string()  # "15"
/// x$shl(4)$as_string()             # "4080"
/// x$count_ones()                   # 8
/// ```
pub trait RBigIntBitOps {
    /// Bitwise AND with another BigInt (passed as string).
    fn bit_and_str(&self, other: &str) -> Result<BigInt, String>;

    /// Bitwise OR with another BigInt (passed as string).
    fn bit_or_str(&self, other: &str) -> Result<BigInt, String>;

    /// Bitwise XOR with another BigInt (passed as string).
    fn bit_xor_str(&self, other: &str) -> Result<BigInt, String>;

    /// Bitwise NOT (two's complement).
    fn bit_not(&self) -> BigInt;

    /// Left shift by n bits.
    fn shl(&self, n: u32) -> BigInt;

    /// Right shift by n bits (arithmetic shift for signed).
    fn shr(&self, n: u32) -> BigInt;

    /// Count the number of set bits (ones) in the absolute value.
    fn count_ones(&self) -> i64;

    /// Count trailing zeros in the absolute value.
    fn trailing_zeros(&self) -> Option<i64>;

    /// Get bit at position n (0-indexed from LSB).
    fn bit(&self, n: u64) -> bool;

    /// Set bit at position n (0-indexed from LSB).
    fn set_bit(&self, n: u64) -> BigInt;

    /// Clear bit at position n (0-indexed from LSB).
    fn clear_bit(&self, n: u64) -> BigInt;
}

impl RBigIntBitOps for BigInt {
    fn bit_and_str(&self, other: &str) -> Result<BigInt, String> {
        let other = BigInt::from_str(other).map_err(|e| e.to_string())?;
        Ok(self & &other)
    }

    fn bit_or_str(&self, other: &str) -> Result<BigInt, String> {
        let other = BigInt::from_str(other).map_err(|e| e.to_string())?;
        Ok(self | &other)
    }

    fn bit_xor_str(&self, other: &str) -> Result<BigInt, String> {
        let other = BigInt::from_str(other).map_err(|e| e.to_string())?;
        Ok(self ^ &other)
    }

    fn bit_not(&self) -> BigInt {
        !self.clone()
    }

    fn shl(&self, n: u32) -> BigInt {
        self << n
    }

    fn shr(&self, n: u32) -> BigInt {
        self >> n
    }

    fn count_ones(&self) -> i64 {
        // Count ones in absolute value
        let (_, digits) = self.to_u32_digits();
        digits.iter().map(|d| d.count_ones() as i64).sum()
    }

    fn trailing_zeros(&self) -> Option<i64> {
        if BigInt::sign(self) == Sign::NoSign {
            return None; // Zero has no trailing zeros (or infinite)
        }
        let (_, digits) = self.to_u32_digits();
        let mut count: i64 = 0;
        for digit in &digits {
            if *digit == 0 {
                count += 32;
            } else {
                count += digit.trailing_zeros() as i64;
                break;
            }
        }
        Some(count)
    }

    fn bit(&self, n: u64) -> bool {
        let (_, digits) = self.to_u32_digits();
        let word_index = (n / 32) as usize;
        let bit_index = (n % 32) as u32;
        if word_index >= digits.len() {
            false
        } else {
            (digits[word_index] >> bit_index) & 1 == 1
        }
    }

    fn set_bit(&self, n: u64) -> BigInt {
        self | (BigInt::from(1) << n)
    }

    fn clear_bit(&self, n: u64) -> BigInt {
        self & !(BigInt::from(1) << n)
    }
}

/// Adapter trait for [`BigUint`] bitwise operations.
///
/// Provides bitwise operations on arbitrary-precision unsigned integers from R.
pub trait RBigUintBitOps {
    /// Bitwise AND with another BigUint (passed as string).
    fn bit_and_str(&self, other: &str) -> Result<BigUint, String>;

    /// Bitwise OR with another BigUint (passed as string).
    fn bit_or_str(&self, other: &str) -> Result<BigUint, String>;

    /// Bitwise XOR with another BigUint (passed as string).
    fn bit_xor_str(&self, other: &str) -> Result<BigUint, String>;

    /// Left shift by n bits.
    fn shl(&self, n: u32) -> BigUint;

    /// Right shift by n bits.
    fn shr(&self, n: u32) -> BigUint;

    /// Count the number of set bits (ones).
    fn count_ones(&self) -> i64;

    /// Count trailing zeros.
    fn trailing_zeros(&self) -> Option<i64>;

    /// Get bit at position n (0-indexed from LSB).
    fn bit(&self, n: u64) -> bool;

    /// Set bit at position n (0-indexed from LSB).
    fn set_bit(&self, n: u64) -> BigUint;

    /// Clear bit at position n (0-indexed from LSB).
    fn clear_bit(&self, n: u64) -> BigUint;
}

impl RBigUintBitOps for BigUint {
    fn bit_and_str(&self, other: &str) -> Result<BigUint, String> {
        let other = BigUint::from_str(other).map_err(|e| e.to_string())?;
        Ok(self & &other)
    }

    fn bit_or_str(&self, other: &str) -> Result<BigUint, String> {
        let other = BigUint::from_str(other).map_err(|e| e.to_string())?;
        Ok(self | &other)
    }

    fn bit_xor_str(&self, other: &str) -> Result<BigUint, String> {
        let other = BigUint::from_str(other).map_err(|e| e.to_string())?;
        Ok(self ^ &other)
    }

    fn shl(&self, n: u32) -> BigUint {
        self << n
    }

    fn shr(&self, n: u32) -> BigUint {
        self >> n
    }

    fn count_ones(&self) -> i64 {
        self.to_u32_digits()
            .iter()
            .map(|d| d.count_ones() as i64)
            .sum()
    }

    fn trailing_zeros(&self) -> Option<i64> {
        use num_bigint::ToBigUint;
        if *self == 0u32.to_biguint().unwrap() {
            return None;
        }
        let digits = self.to_u32_digits();
        let mut count: i64 = 0;
        for digit in &digits {
            if *digit == 0 {
                count += 32;
            } else {
                count += digit.trailing_zeros() as i64;
                break;
            }
        }
        Some(count)
    }

    fn bit(&self, n: u64) -> bool {
        let digits = self.to_u32_digits();
        let word_index = (n / 32) as usize;
        let bit_index = (n % 32) as u32;
        if word_index >= digits.len() {
            false
        } else {
            (digits[word_index] >> bit_index) & 1 == 1
        }
    }

    fn set_bit(&self, n: u64) -> BigUint {
        self | (BigUint::from(1u32) << n)
    }

    fn clear_bit(&self, n: u64) -> BigUint {
        // BigUint doesn't have NOT, so we compute it differently
        if !RBigUintBitOps::bit(self, n) {
            self.clone()
        } else {
            self ^ (BigUint::from(1u32) << n)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bigint_from_str() {
        let n = BigInt::from_str("12345678901234567890").unwrap();
        assert_eq!(ToString::to_string(&n), "12345678901234567890");
    }

    #[test]
    fn rbigintops_sign() {
        let pos = BigInt::from_str("123").unwrap();
        let neg = BigInt::from_str("-456").unwrap();
        let zero = BigInt::from_str("0").unwrap();

        assert!(RBigIntOps::is_positive(&pos));
        assert!(!RBigIntOps::is_negative(&pos));
        assert_eq!(RBigIntOps::sign(&pos), 1);

        assert!(!RBigIntOps::is_positive(&neg));
        assert!(RBigIntOps::is_negative(&neg));
        assert_eq!(RBigIntOps::sign(&neg), -1);

        assert!(RBigIntOps::is_zero(&zero));
        assert_eq!(RBigIntOps::sign(&zero), 0);
    }

    #[test]
    fn rbigintops_arithmetic() {
        let a = BigInt::from_str("1000000000000").unwrap();

        let sum = RBigIntOps::add_str(&a, "1").unwrap();
        assert_eq!(ToString::to_string(&sum), "1000000000001");

        let diff = RBigIntOps::sub_str(&a, "1").unwrap();
        assert_eq!(ToString::to_string(&diff), "999999999999");

        let prod = RBigIntOps::mul_str(&a, "2").unwrap();
        assert_eq!(ToString::to_string(&prod), "2000000000000");

        let quot = RBigIntOps::div_str(&a, "1000").unwrap();
        assert_eq!(ToString::to_string(&quot), "1000000000");

        let rem = RBigIntOps::rem_str(&a, "7").unwrap();
        assert_eq!(ToString::to_string(&rem), "1"); // 1000000000000 % 7 = 1
    }

    #[test]
    fn rbigintops_pow_gcd() {
        let n = BigInt::from_str("2").unwrap();
        let pow10 = RBigIntOps::pow(&n, 10);
        assert_eq!(ToString::to_string(&pow10), "1024");

        let a = BigInt::from_str("48").unwrap();
        let gcd = RBigIntOps::gcd_str(&a, "18").unwrap();
        assert_eq!(ToString::to_string(&gcd), "6");
    }

    #[test]
    fn rbigintops_abs_neg() {
        let neg = BigInt::from_str("-123").unwrap();
        assert_eq!(ToString::to_string(&RBigIntOps::abs(&neg)), "123");
        assert_eq!(ToString::to_string(&RBigIntOps::neg(&neg)), "123");

        let pos = BigInt::from_str("456").unwrap();
        assert_eq!(ToString::to_string(&RBigIntOps::abs(&pos)), "456");
        assert_eq!(ToString::to_string(&RBigIntOps::neg(&pos)), "-456");
    }

    #[test]
    fn rbigintops_bits() {
        let n = BigInt::from_str("255").unwrap(); // 8 bits
        assert_eq!(RBigIntOps::bit_length(&n), 8);

        let n = BigInt::from_str("256").unwrap(); // 9 bits
        assert_eq!(RBigIntOps::bit_length(&n), 9);
    }

    #[test]
    fn rbiguintops_basic() {
        let n = BigUint::from_str("12345").unwrap();
        assert!(!RBigUintOps::is_zero(&n));
        assert!(!RBigUintOps::is_one(&n));

        let zero = BigUint::from_str("0").unwrap();
        assert!(RBigUintOps::is_zero(&zero));

        let one = BigUint::from_str("1").unwrap();
        assert!(RBigUintOps::is_one(&one));
    }

    #[test]
    fn rbiguintops_arithmetic() {
        let a = BigUint::from_str("1000").unwrap();

        let sum = RBigUintOps::add_str(&a, "234").unwrap();
        assert_eq!(ToString::to_string(&sum), "1234");

        let diff = RBigUintOps::sub_str(&a, "100").unwrap();
        assert_eq!(ToString::to_string(&diff), "900");

        // Subtraction that would be negative should error
        let result = RBigUintOps::sub_str(&a, "2000");
        assert!(result.is_err());
    }

    #[test]
    fn rbigintops_division_by_zero() {
        let n = BigInt::from_str("100").unwrap();
        assert!(RBigIntOps::div_str(&n, "0").is_err());
        assert!(RBigIntOps::rem_str(&n, "0").is_err());
    }

    #[test]
    fn rbigintbitops_and_or_xor() {
        let a = BigInt::from_str("255").unwrap(); // 0xFF
        let b = "15"; // 0x0F

        let and = RBigIntBitOps::bit_and_str(&a, b).unwrap();
        assert_eq!(ToString::to_string(&and), "15"); // 0xFF & 0x0F = 0x0F

        let or = RBigIntBitOps::bit_or_str(&a, "256").unwrap();
        assert_eq!(ToString::to_string(&or), "511"); // 0xFF | 0x100 = 0x1FF

        let xor = RBigIntBitOps::bit_xor_str(&a, "255").unwrap();
        assert_eq!(ToString::to_string(&xor), "0"); // 0xFF ^ 0xFF = 0
    }

    #[test]
    fn rbigintbitops_shift() {
        let n = BigInt::from_str("1").unwrap();

        let shl = RBigIntBitOps::shl(&n, 8);
        assert_eq!(ToString::to_string(&shl), "256"); // 1 << 8

        let n256 = BigInt::from_str("256").unwrap();
        let shr = RBigIntBitOps::shr(&n256, 4);
        assert_eq!(ToString::to_string(&shr), "16"); // 256 >> 4
    }

    #[test]
    fn rbigintbitops_count() {
        let n = BigInt::from_str("255").unwrap(); // 0xFF = 8 ones
        assert_eq!(RBigIntBitOps::count_ones(&n), 8);

        let n8 = BigInt::from_str("8").unwrap(); // 0b1000 = trailing zeros = 3
        assert_eq!(RBigIntBitOps::trailing_zeros(&n8), Some(3));

        let zero = BigInt::from_str("0").unwrap();
        assert_eq!(RBigIntBitOps::trailing_zeros(&zero), None);
    }

    #[test]
    fn rbigintbitops_bit_access() {
        let n = BigInt::from_str("5").unwrap(); // 0b101

        assert!(RBigIntBitOps::bit(&n, 0)); // bit 0 is set
        assert!(!RBigIntBitOps::bit(&n, 1)); // bit 1 is not set
        assert!(RBigIntBitOps::bit(&n, 2)); // bit 2 is set
        assert!(!RBigIntBitOps::bit(&n, 3)); // bit 3 is not set

        let with_bit1 = RBigIntBitOps::set_bit(&n, 1);
        assert_eq!(ToString::to_string(&with_bit1), "7"); // 0b111

        let without_bit2 = RBigIntBitOps::clear_bit(&n, 2);
        assert_eq!(ToString::to_string(&without_bit2), "1"); // 0b001
    }

    #[test]
    fn rbiguintbitops_basic() {
        let a = BigUint::from_str("255").unwrap();

        let and = RBigUintBitOps::bit_and_str(&a, "15").unwrap();
        assert_eq!(ToString::to_string(&and), "15");

        let shl = RBigUintBitOps::shl(&a, 4);
        assert_eq!(ToString::to_string(&shl), "4080"); // 255 << 4

        assert_eq!(RBigUintBitOps::count_ones(&a), 8);
    }

    #[test]
    fn rbigintbitops_not() {
        let n = BigInt::from_str("0").unwrap();
        let not_n = RBigIntBitOps::bit_not(&n);
        assert_eq!(ToString::to_string(&not_n), "-1"); // ~0 = -1 in two's complement

        let one = BigInt::from_str("1").unwrap();
        let not_one = RBigIntBitOps::bit_not(&one);
        assert_eq!(ToString::to_string(&not_one), "-2"); // ~1 = -2
    }
}
