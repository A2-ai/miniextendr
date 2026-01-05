//! Factor support for enum ↔ R factor conversions.
//!
//! This module provides the [`RFactor`] trait for converting Rust enums to/from R factors.
//! Factors in R are integer vectors with a `levels` attribute (character vector) and a
//! `class` attribute set to `"factor"`.
//!
//! # Example
//!
//! ```ignore
//! use miniextendr_api::RFactor;
//!
//! #[derive(Copy, Clone, RFactor)]
//! enum Color {
//!     Red,
//!     Green,
//!     Blue,
//! }
//!
//! // Color values can now be converted to/from R factors automatically
//! #[miniextendr]
//! fn get_color(c: Color) -> &'static str {
//!     match c {
//!         Color::Red => "it's red!",
//!         Color::Green => "it's green!",
//!         Color::Blue => "it's blue!",
//!     }
//! }
//! ```
//!
//! # R Factor Structure
//!
//! A factor in R is an `INTSXP` with:
//! - `levels` attribute: `STRSXP` containing level names (1-based indexing)
//! - `class` attribute: `"factor"`
//!
//! Integer payload uses `NA_INTEGER` for missing values.

use std::sync::OnceLock;

use crate::altrep_traits::NA_INTEGER;
use crate::ffi::{
    INTEGER, INTEGER_ELT, R_ClassSymbol, R_LevelsSymbol, Rboolean, Rf_allocVector, Rf_getAttrib,
    Rf_isFactor, Rf_mkCharLenCE, Rf_setAttrib, Rf_xlength, SET_STRING_ELT, SEXP, SEXPTYPE,
    STRING_ELT, SexpExt, cetype_t,
};
use crate::from_r::{SexpError, charsxp_to_str};

// =============================================================================
// Global factor symbol (cached SYMSXP)
// =============================================================================

/// Cached "factor" class string SEXP.
static FACTOR_CLASS: OnceLock<SEXP> = OnceLock::new();

/// Get or initialize the factor class STRSXP.
///
/// This creates a length-1 STRSXP containing "factor".
fn factor_class_sexp() -> SEXP {
    *FACTOR_CLASS.get_or_init(|| unsafe {
        let class_sexp = Rf_allocVector(SEXPTYPE::STRSXP, 1);
        crate::preserve::insert(class_sexp);
        let factor_str = "factor";
        let charsxp = Rf_mkCharLenCE(
            factor_str.as_ptr().cast(),
            factor_str.len() as i32,
            cetype_t::CE_UTF8,
        );
        SET_STRING_ELT(class_sexp, 0, charsxp);
        class_sexp
    })
}

// =============================================================================
// RFactor trait
// =============================================================================

/// Trait for mapping Rust enums to R factors.
///
/// This trait is typically implemented via `#[derive(RFactor)]` for C-style enums.
///
/// # Safety
///
/// The `LEVELS` array must match the implementation of `to_level_index` and
/// `from_level_index` - specifically, indices must be in the range 1..=LEVELS.len().
///
/// # Example (manual implementation)
///
/// ```ignore
/// use miniextendr_api::factor::RFactor;
///
/// #[derive(Copy, Clone)]
/// enum Status { Active, Inactive, Pending }
///
/// impl RFactor for Status {
///     const LEVELS: &'static [&'static str] = &["Active", "Inactive", "Pending"];
///
///     fn to_level_index(self) -> i32 {
///         match self {
///             Status::Active => 1,
///             Status::Inactive => 2,
///             Status::Pending => 3,
///         }
///     }
///
///     fn from_level_index(idx: i32) -> Option<Self> {
///         match idx {
///             1 => Some(Status::Active),
///             2 => Some(Status::Inactive),
///             3 => Some(Status::Pending),
///             _ => None,
///         }
///     }
/// }
/// ```
pub trait RFactor: Copy + 'static {
    /// Level names for this enum.
    ///
    /// The order must match the indices returned by `to_level_index`.
    const LEVELS: &'static [&'static str];

    /// Convert enum variant to 1-based level index.
    ///
    /// Returns an integer in the range 1..=LEVELS.len().
    fn to_level_index(self) -> i32;

    /// Convert 1-based level index to enum variant.
    ///
    /// Returns `None` for out-of-range indices.
    fn from_level_index(idx: i32) -> Option<Self>;
}

// =============================================================================
// Helper functions
// =============================================================================

/// Build a levels STRSXP from level names.
pub fn build_levels_sexp(levels: &[&str]) -> SEXP {
    unsafe {
        let sexp = Rf_allocVector(SEXPTYPE::STRSXP, levels.len() as isize);
        for (i, level) in levels.iter().enumerate() {
            let charsxp =
                Rf_mkCharLenCE(level.as_ptr().cast(), level.len() as i32, cetype_t::CE_UTF8);
            SET_STRING_ELT(sexp, i as isize, charsxp);
        }
        sexp
    }
}

/// Build a factor SEXP from indices and level names.
///
/// # Arguments
///
/// * `indices` - 1-based integer indices (use NA_INTEGER for NA)
/// * `levels` - Level names
pub fn build_factor(indices: &[i32], levels: &[&str]) -> SEXP {
    unsafe {
        let sexp = Rf_allocVector(SEXPTYPE::INTSXP, indices.len() as isize);
        let ptr = INTEGER(sexp);
        std::ptr::copy_nonoverlapping(indices.as_ptr(), ptr, indices.len());

        // Set levels attribute
        let levels_sexp = build_levels_sexp(levels);
        Rf_setAttrib(sexp, R_LevelsSymbol, levels_sexp);

        // Set class attribute to "factor"
        Rf_setAttrib(sexp, R_ClassSymbol, factor_class_sexp());

        sexp
    }
}

/// Validate that a SEXP is a factor with expected levels.
///
/// Returns an error if:
/// - The SEXP is not a factor
/// - The levels don't match expected (order-sensitive)
pub fn validate_factor_levels(sexp: SEXP, expected_levels: &[&str]) -> Result<(), SexpError> {
    // Check it's a factor
    if unsafe { Rf_isFactor(sexp) } == Rboolean::FALSE {
        return Err(SexpError::InvalidValue(
            "expected a factor, got non-factor".into(),
        ));
    }

    // Get and validate levels
    let levels = unsafe { Rf_getAttrib(sexp, R_LevelsSymbol) };
    if levels.type_of() != SEXPTYPE::STRSXP {
        return Err(SexpError::InvalidValue(
            "factor levels attribute is not a character vector".into(),
        ));
    }

    let n_levels = unsafe { Rf_xlength(levels) } as usize;
    if n_levels != expected_levels.len() {
        return Err(SexpError::InvalidValue(format!(
            "factor has {} levels, expected {}",
            n_levels,
            expected_levels.len()
        )));
    }

    // Validate each level matches
    for (i, expected) in expected_levels.iter().enumerate() {
        let charsxp = unsafe { STRING_ELT(levels, i as isize) };
        let actual = unsafe { charsxp_to_str(charsxp) };
        if actual != *expected {
            return Err(SexpError::InvalidValue(format!(
                "level {} mismatch: expected '{}', got '{}'",
                i + 1,
                expected,
                actual
            )));
        }
    }

    Ok(())
}

// =============================================================================
// Conversion helpers (for use by derive macro)
// =============================================================================

/// Convert a single RFactor value to an R factor SEXP.
///
/// This is used by the derive macro to implement IntoR.
#[inline]
pub fn factor_to_sexp<T: RFactor>(value: T) -> SEXP {
    let idx = value.to_level_index();
    build_factor(&[idx], T::LEVELS)
}

/// Convert a Vec of RFactor values to an R factor SEXP.
///
/// This is used by the derive macro to implement IntoR for Vec<T>.
#[inline]
pub fn factor_vec_to_sexp<T: RFactor>(values: &[T]) -> SEXP {
    let indices: Vec<i32> = values.iter().map(|v| v.to_level_index()).collect();
    build_factor(&indices, T::LEVELS)
}

/// Convert a Vec of Option<RFactor> values to an R factor SEXP.
///
/// This is used by the derive macro to implement IntoR for Vec<Option<T>>.
#[inline]
pub fn factor_option_vec_to_sexp<T: RFactor>(values: &[Option<T>]) -> SEXP {
    let indices: Vec<i32> = values
        .iter()
        .map(|v| match v {
            Some(val) => val.to_level_index(),
            None => NA_INTEGER,
        })
        .collect();
    build_factor(&indices, T::LEVELS)
}

/// Convert an R factor SEXP to a single RFactor value.
///
/// This is used by the derive macro to implement TryFromSexp.
#[inline]
pub fn factor_from_sexp<T: RFactor>(sexp: SEXP) -> Result<T, SexpError> {
    // Validate factor structure
    validate_factor_levels(sexp, T::LEVELS)?;

    // Check length
    let len = unsafe { Rf_xlength(sexp) };
    if len != 1 {
        return Err(SexpError::InvalidValue(format!(
            "expected factor of length 1, got length {}",
            len
        )));
    }

    // Get index
    let idx = unsafe { INTEGER_ELT(sexp, 0) };
    if idx == NA_INTEGER {
        return Err(SexpError::InvalidValue(
            "NA value in non-Option factor".into(),
        ));
    }

    T::from_level_index(idx)
        .ok_or_else(|| SexpError::InvalidValue(format!("factor index {} out of range", idx)))
}

/// Convert an R factor SEXP to an Option<RFactor> value.
///
/// This is used by the derive macro to implement TryFromSexp for Option<T>.
#[inline]
pub fn factor_option_from_sexp<T: RFactor>(sexp: SEXP) -> Result<Option<T>, SexpError> {
    // Validate factor structure
    validate_factor_levels(sexp, T::LEVELS)?;

    // Check length
    let len = unsafe { Rf_xlength(sexp) };
    if len != 1 {
        return Err(SexpError::InvalidValue(format!(
            "expected factor of length 1, got length {}",
            len
        )));
    }

    // Get index
    let idx = unsafe { INTEGER_ELT(sexp, 0) };
    if idx == NA_INTEGER {
        return Ok(None);
    }

    T::from_level_index(idx)
        .map(Some)
        .ok_or_else(|| SexpError::InvalidValue(format!("factor index {} out of range", idx)))
}

/// Convert an R factor SEXP to a Vec<RFactor>.
///
/// This is used by the derive macro to implement TryFromSexp for Vec<T>.
#[inline]
pub fn factor_vec_from_sexp<T: RFactor>(sexp: SEXP) -> Result<Vec<T>, SexpError> {
    // Validate factor structure
    validate_factor_levels(sexp, T::LEVELS)?;

    let len = unsafe { Rf_xlength(sexp) } as usize;
    let mut result = Vec::with_capacity(len);

    for i in 0..len {
        let idx = unsafe { INTEGER_ELT(sexp, i as isize) };
        if idx == NA_INTEGER {
            return Err(SexpError::InvalidValue(format!(
                "NA at index {} in non-Option factor",
                i
            )));
        }
        let val = T::from_level_index(idx).ok_or_else(|| {
            SexpError::InvalidValue(format!(
                "factor index {} out of range at position {}",
                idx, i
            ))
        })?;
        result.push(val);
    }

    Ok(result)
}

/// Convert an R factor SEXP to a Vec<Option<RFactor>>.
///
/// This is used by the derive macro to implement TryFromSexp for Vec<Option<T>>.
#[inline]
pub fn factor_option_vec_from_sexp<T: RFactor>(sexp: SEXP) -> Result<Vec<Option<T>>, SexpError> {
    // Validate factor structure
    validate_factor_levels(sexp, T::LEVELS)?;

    let len = unsafe { Rf_xlength(sexp) } as usize;
    let mut result = Vec::with_capacity(len);

    for i in 0..len {
        let idx = unsafe { INTEGER_ELT(sexp, i as isize) };
        if idx == NA_INTEGER {
            result.push(None);
        } else {
            let val = T::from_level_index(idx).ok_or_else(|| {
                SexpError::InvalidValue(format!(
                    "factor index {} out of range at position {}",
                    idx, i
                ))
            })?;
            result.push(Some(val));
        }
    }

    Ok(result)
}

// =============================================================================
// Newtype wrappers for ergonomic trait impls
// =============================================================================
//
// Due to Rust's orphan rules, we cannot implement IntoR for Vec<T: RFactor>
// because neither IntoR nor Vec is local when the impl is generated by the
// derive macro in the user's crate. The newtype wrapper approach gives us
// a local type that can implement IntoR.

use crate::from_r::TryFromSexp;
use crate::into_r::IntoR;

/// Wrapper for `Vec<T>` that enables `IntoR` and `TryFromSexp` implementations
/// for factor vectors.
///
/// Due to Rust's orphan rules, we cannot implement `IntoR for Vec<T>` where
/// `T: RFactor`. This newtype wrapper provides ergonomic trait-based conversion.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::{FactorVec, RFactor};
///
/// #[derive(Copy, Clone, RFactor)]
/// enum Color { Red, Green, Blue }
///
/// // Convert to R factor using trait method
/// let colors = FactorVec(vec![Color::Red, Color::Green]);
/// let sexp = colors.into_sexp();
/// ```
#[derive(Debug, Clone)]
pub struct FactorVec<T>(pub Vec<T>);

impl<T> FactorVec<T> {
    /// Create a new FactorVec from a Vec.
    pub fn new(vec: Vec<T>) -> Self {
        Self(vec)
    }

    /// Unwrap into the inner Vec.
    pub fn into_inner(self) -> Vec<T> {
        self.0
    }
}

impl<T> From<Vec<T>> for FactorVec<T> {
    fn from(vec: Vec<T>) -> Self {
        Self(vec)
    }
}

impl<T> std::ops::Deref for FactorVec<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for FactorVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: RFactor> IntoR for FactorVec<T> {
    fn into_sexp(self) -> SEXP {
        factor_vec_to_sexp(&self.0)
    }
}

impl<T: RFactor> TryFromSexp for FactorVec<T> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        factor_vec_from_sexp(sexp).map(FactorVec)
    }
}

/// Wrapper for `Vec<Option<T>>` that enables `IntoR` and `TryFromSexp`
/// implementations for factor vectors with NA support.
#[derive(Debug, Clone)]
pub struct FactorOptionVec<T>(pub Vec<Option<T>>);

impl<T> FactorOptionVec<T> {
    /// Create a new FactorOptionVec from a Vec.
    pub fn new(vec: Vec<Option<T>>) -> Self {
        Self(vec)
    }

    /// Unwrap into the inner Vec.
    pub fn into_inner(self) -> Vec<Option<T>> {
        self.0
    }
}

impl<T> From<Vec<Option<T>>> for FactorOptionVec<T> {
    fn from(vec: Vec<Option<T>>) -> Self {
        Self(vec)
    }
}

impl<T> std::ops::Deref for FactorOptionVec<T> {
    type Target = Vec<Option<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for FactorOptionVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: RFactor> IntoR for FactorOptionVec<T> {
    fn into_sexp(self) -> SEXP {
        factor_option_vec_to_sexp(&self.0)
    }
}

impl<T: RFactor> TryFromSexp for FactorOptionVec<T> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        factor_option_vec_from_sexp(sexp).map(FactorOptionVec)
    }
}

// =============================================================================
// Unit tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Copy, Clone, Debug, PartialEq)]
    enum TestColor {
        Red,
        Green,
        Blue,
    }

    impl RFactor for TestColor {
        const LEVELS: &'static [&'static str] = &["Red", "Green", "Blue"];

        fn to_level_index(self) -> i32 {
            match self {
                TestColor::Red => 1,
                TestColor::Green => 2,
                TestColor::Blue => 3,
            }
        }

        fn from_level_index(idx: i32) -> Option<Self> {
            match idx {
                1 => Some(TestColor::Red),
                2 => Some(TestColor::Green),
                3 => Some(TestColor::Blue),
                _ => None,
            }
        }
    }

    #[test]
    fn test_level_index_roundtrip() {
        assert_eq!(
            TestColor::from_level_index(TestColor::Red.to_level_index()),
            Some(TestColor::Red)
        );
        assert_eq!(
            TestColor::from_level_index(TestColor::Green.to_level_index()),
            Some(TestColor::Green)
        );
        assert_eq!(
            TestColor::from_level_index(TestColor::Blue.to_level_index()),
            Some(TestColor::Blue)
        );
    }

    #[test]
    fn test_invalid_index() {
        assert_eq!(TestColor::from_level_index(0), None);
        assert_eq!(TestColor::from_level_index(4), None);
        assert_eq!(TestColor::from_level_index(-1), None);
    }

    #[test]
    fn test_levels_array() {
        assert_eq!(TestColor::LEVELS, &["Red", "Green", "Blue"]);
    }
}
