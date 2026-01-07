//! Integration with the `bitvec` crate.
//!
//! Provides conversions between R logical vectors and `BitVec` types.
//!
//! # Features
//!
//! Enable this module with the `bitvec` feature:
//!
//! ```toml
//! [dependencies]
//! miniextendr-api = { version = "0.1", features = ["bitvec"] }
//! ```
//!
//! # Type Aliases
//!
//! This module provides a standard `RBitVec` type alias for consistent bit ordering:
//!
//! ```ignore
//! pub type RBitVec = BitVec<u8, Lsb0>;
//! ```
//!
//! # Example
//!
//! ```ignore
//! use miniextendr_api::bitvec_impl::RBitVec;
//!
//! #[miniextendr]
//! fn count_true(bits: RBitVec) -> i32 {
//!     bits.count_ones() as i32
//! }
//!
//! #[miniextendr]
//! fn make_mask(len: i32) -> RBitVec {
//!     let mut bits = RBitVec::repeat(false, len as usize);
//!     bits.set(0, true);
//!     bits.set(len as usize - 1, true);
//!     bits
//! }
//! ```
//!
//! # NA Handling
//!
//! NA values in logical vectors cause an error during conversion to `RBitVec`.
//! Use `RBitVecNA` (with `Option<bool>` semantics) if you need to preserve NA values.

pub use bitvec::order::{Lsb0, Msb0};
pub use bitvec::vec::BitVec;

use crate::altrep_traits::NA_LOGICAL;
use crate::ffi::{SEXP, SEXPTYPE, SexpExt};
use crate::from_r::{SexpError, SexpTypeError, TryFromSexp};
use crate::impl_option_try_from_sexp;
use crate::into_r::IntoR;

/// Standard bit vector type for R interop.
///
/// Uses `u8` storage with LSB-first ordering for consistent behavior.
pub type RBitVec = BitVec<u8, Lsb0>;

// =============================================================================
// TryFromSexp for RBitVec
// =============================================================================

impl TryFromSexp for RBitVec {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        use crate::ffi::LOGICAL_ELT;

        let actual = sexp.type_of();
        if actual != SEXPTYPE::LGLSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::LGLSXP,
                actual,
            }
            .into());
        }

        let len = sexp.len();
        let mut bits = RBitVec::with_capacity(len);

        for i in 0..len {
            let val = unsafe { LOGICAL_ELT(sexp, i as crate::ffi::R_xlen_t) };
            if val == NA_LOGICAL {
                return Err(SexpError::InvalidValue(format!(
                    "NA at index {} not allowed for RBitVec",
                    i
                )));
            }
            bits.push(val != 0);
        }

        Ok(bits)
    }
}

impl_option_try_from_sexp!(RBitVec);

// =============================================================================
// IntoR for RBitVec
// =============================================================================

impl IntoR for RBitVec {
    fn into_sexp(self) -> SEXP {
        use crate::ffi::{Rf_allocVector, SET_LOGICAL_ELT};

        let len = self.len();
        let sexp = unsafe { Rf_allocVector(SEXPTYPE::LGLSXP, len as crate::ffi::R_xlen_t) };

        for (i, bit) in self.iter().enumerate() {
            let val = if *bit { 1 } else { 0 };
            unsafe { SET_LOGICAL_ELT(sexp, i as crate::ffi::R_xlen_t, val) };
        }

        sexp
    }
}

impl IntoR for Option<RBitVec> {
    fn into_sexp(self) -> SEXP {
        match self {
            Some(bits) => bits.into_sexp(),
            None => unsafe { crate::ffi::R_NilValue },
        }
    }
}

// =============================================================================
// TryFromSexp / IntoR for BitVec<u8, Msb0>
// =============================================================================

impl TryFromSexp for BitVec<u8, Msb0> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        use crate::ffi::LOGICAL_ELT;

        let actual = sexp.type_of();
        if actual != SEXPTYPE::LGLSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::LGLSXP,
                actual,
            }
            .into());
        }

        let len = sexp.len();
        let mut bits = BitVec::<u8, Msb0>::with_capacity(len);

        for i in 0..len {
            let val = unsafe { LOGICAL_ELT(sexp, i as crate::ffi::R_xlen_t) };
            if val == NA_LOGICAL {
                return Err(SexpError::InvalidValue(format!(
                    "NA at index {} not allowed for BitVec",
                    i
                )));
            }
            bits.push(val != 0);
        }

        Ok(bits)
    }
}

impl_option_try_from_sexp!(BitVec<u8, Msb0>);

impl IntoR for BitVec<u8, Msb0> {
    fn into_sexp(self) -> SEXP {
        use crate::ffi::{Rf_allocVector, SET_LOGICAL_ELT};

        let len = self.len();
        let sexp = unsafe { Rf_allocVector(SEXPTYPE::LGLSXP, len as crate::ffi::R_xlen_t) };

        for (i, bit) in self.iter().enumerate() {
            let val = if *bit { 1 } else { 0 };
            unsafe { SET_LOGICAL_ELT(sexp, i as crate::ffi::R_xlen_t, val) };
        }

        sexp
    }
}

impl IntoR for Option<BitVec<u8, Msb0>> {
    fn into_sexp(self) -> SEXP {
        match self {
            Some(bits) => bits.into_sexp(),
            None => unsafe { crate::ffi::R_NilValue },
        }
    }
}

// =============================================================================
// Helper functions
// =============================================================================

/// Create a bit vector from a slice of booleans.
#[inline]
pub fn bitvec_from_bools(bools: &[bool]) -> RBitVec {
    bools.iter().collect()
}

/// Convert a bit vector to a Vec of booleans.
#[inline]
pub fn bitvec_to_bools(bits: &RBitVec) -> Vec<bool> {
    bits.iter().map(|b| *b).collect()
}

/// Count the number of set bits (ones).
#[inline]
pub fn bitvec_count_ones(bits: &RBitVec) -> usize {
    bits.count_ones()
}

/// Count the number of unset bits (zeros).
#[inline]
pub fn bitvec_count_zeros(bits: &RBitVec) -> usize {
    bits.count_zeros()
}

// =============================================================================
// Unit tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitvec_from_bools() {
        let bits = bitvec_from_bools(&[true, false, true, true, false]);
        assert_eq!(bits.len(), 5);
        assert!(bits[0]);
        assert!(!bits[1]);
        assert!(bits[2]);
        assert!(bits[3]);
        assert!(!bits[4]);
    }

    #[test]
    fn test_bitvec_to_bools() {
        let mut bits = RBitVec::new();
        bits.push(true);
        bits.push(false);
        bits.push(true);

        let bools = bitvec_to_bools(&bits);
        assert_eq!(bools, vec![true, false, true]);
    }

    #[test]
    fn test_bitvec_count() {
        let bits = bitvec_from_bools(&[true, false, true, true, false]);
        assert_eq!(bitvec_count_ones(&bits), 3);
        assert_eq!(bitvec_count_zeros(&bits), 2);
    }

    #[test]
    fn test_bitvec_empty() {
        let bits = RBitVec::new();
        assert_eq!(bits.len(), 0);
        assert_eq!(bitvec_count_ones(&bits), 0);
    }

    #[test]
    fn test_bitvec_repeat() {
        let bits = RBitVec::repeat(true, 10);
        assert_eq!(bits.len(), 10);
        assert_eq!(bitvec_count_ones(&bits), 10);

        let bits = RBitVec::repeat(false, 10);
        assert_eq!(bitvec_count_zeros(&bits), 10);
    }

    #[test]
    fn test_bitvec_msb0() {
        let mut bits = BitVec::<u8, Msb0>::new();
        bits.push(true);
        bits.push(false);
        bits.push(true);

        assert!(bits[0]);
        assert!(!bits[1]);
        assert!(bits[2]);
    }
}
