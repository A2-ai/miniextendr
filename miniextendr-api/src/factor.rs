//! Factor support for enum ↔ R factor conversions.
//!
//! R factors are integer vectors with a `levels` attribute (character vector)
//! and a `class` attribute set to `"factor"`. The integer payload uses 1-based
//! indexing into the levels, with `NA_INTEGER` for missing values.
//!
//! # Usage
//!
//! ```ignore
//! use miniextendr_api::RFactor;
//!
//! #[derive(Copy, Clone, RFactor)]
//! enum Color { Red, Green, Blue }
//!
//! // Enum values convert to/from R factors automatically
//! #[miniextendr]
//! fn describe(c: Color) -> &'static str {
//!     match c {
//!         Color::Red => "red",
//!         Color::Green => "green",
//!         Color::Blue => "blue",
//!     }
//! }
//! ```

use std::ffi::CString;
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::OnceLock;

use crate::altrep_traits::NA_INTEGER;
use crate::ffi::{
    INTEGER, INTEGER_ELT, PRINTNAME, R_ClassSymbol, R_LevelsSymbol, Rboolean, Rf_allocVector,
    Rf_getAttrib, Rf_install, Rf_isFactor, Rf_setAttrib, Rf_xlength, SET_STRING_ELT, SEXP,
    SEXPTYPE, STRING_ELT, SexpExt,
};
use crate::from_r::{SexpError, TryFromSexp, charsxp_to_str};
use crate::into_r::IntoR;

// =============================================================================
// Cached "factor" class STRSXP
// =============================================================================

static FACTOR_CLASS: OnceLock<SEXP> = OnceLock::new();

fn factor_class_sexp() -> SEXP {
    *FACTOR_CLASS.get_or_init(|| unsafe {
        let class_sexp = Rf_allocVector(SEXPTYPE::STRSXP, 1);
        crate::preserve::insert(class_sexp);
        // Use symbol PRINTNAME for permanent CHARSXP
        let sym = Rf_install(c"factor".as_ptr());
        SET_STRING_ELT(class_sexp, 0, PRINTNAME(sym));
        class_sexp
    })
}

// =============================================================================
// RFactor trait
// =============================================================================

/// Trait for mapping Rust enums to R factors.
///
/// Typically implemented via `#[derive(RFactor)]` for C-style enums.
/// The derive macro also generates `IntoR` and `TryFromSexp` implementations.
pub trait RFactor: Copy + 'static {
    /// Level names for this enum (order matches index values).
    const LEVELS: &'static [&'static str];

    /// Convert variant to 1-based level index.
    fn to_level_index(self) -> i32;

    /// Convert 1-based level index to variant, or `None` if out of range.
    fn from_level_index(idx: i32) -> Option<Self>;
}

// =============================================================================
// Core building functions
// =============================================================================

/// Build a levels STRSXP using symbol PRINTNAMEs for permanent CHARSXP protection.
///
/// The returned STRSXP is NOT protected - caller must protect or preserve it.
pub fn build_levels_sexp(levels: &[&str]) -> SEXP {
    unsafe {
        let sexp = Rf_allocVector(SEXPTYPE::STRSXP, levels.len() as isize);
        for (i, level) in levels.iter().enumerate() {
            // Install as symbol - symbols and their PRINTNAMEs are never GC'd
            let c_str = CString::new(*level).expect("level name contains null byte");
            let sym = Rf_install(c_str.as_ptr());
            SET_STRING_ELT(sexp, i as isize, PRINTNAME(sym));
        }
        sexp
    }
}

/// Build a levels STRSXP and preserve it permanently (for caching).
pub fn build_levels_sexp_cached(levels: &[&str]) -> SEXP {
    unsafe {
        let sexp = build_levels_sexp(levels);
        crate::preserve::insert(sexp);
        sexp
    }
}

/// Build a factor SEXP from indices and a levels STRSXP.
pub fn build_factor(indices: &[i32], levels: SEXP) -> SEXP {
    unsafe {
        let sexp = Rf_allocVector(SEXPTYPE::INTSXP, indices.len() as isize);
        let ptr = INTEGER(sexp);
        std::ptr::copy_nonoverlapping(indices.as_ptr(), ptr, indices.len());
        Rf_setAttrib(sexp, R_LevelsSymbol, levels);
        Rf_setAttrib(sexp, R_ClassSymbol, factor_class_sexp());
        sexp
    }
}

// =============================================================================
// Factor - view into an R factor's data
// =============================================================================

/// A borrowed view into an R factor's integer indices.
///
/// Provides `Deref` to `&[i32]` for direct slice access to the factor's
/// underlying integer data. The indices are 1-based (matching R's convention)
/// with `NA_INTEGER` for missing values.
///
/// # Example
///
/// ```ignore
/// let factor = Factor::try_new(sexp)?;
/// for &idx in factor.iter() {
///     if idx == NA_INTEGER {
///         println!("NA");
///     } else {
///         println!("level index: {}", idx);
///     }
/// }
/// ```
pub struct Factor<'a> {
    indices: &'a [i32],
    levels_sexp: SEXP,
    _marker: PhantomData<&'a ()>,
}

impl<'a> Factor<'a> {
    /// Create a Factor from a factor SEXP.
    ///
    /// Returns an error if the SEXP is not a factor.
    pub fn try_new(sexp: SEXP) -> Result<Self, SexpError> {
        if unsafe { Rf_isFactor(sexp) } == Rboolean::FALSE {
            return Err(SexpError::InvalidValue("expected a factor".into()));
        }

        let len = unsafe { Rf_xlength(sexp) } as usize;
        let ptr = unsafe { INTEGER(sexp) };
        let indices = unsafe { std::slice::from_raw_parts(ptr, len) };
        let levels_sexp = unsafe { Rf_getAttrib(sexp, R_LevelsSymbol) };

        Ok(Self {
            indices,
            levels_sexp,
            _marker: PhantomData,
        })
    }

    /// Number of elements in the factor.
    #[inline]
    pub fn len(&self) -> usize {
        self.indices.len()
    }

    /// Whether the factor is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    /// The levels STRSXP.
    #[inline]
    pub fn levels_sexp(&self) -> SEXP {
        self.levels_sexp
    }

    /// Number of levels.
    #[inline]
    pub fn n_levels(&self) -> usize {
        unsafe { Rf_xlength(self.levels_sexp) as usize }
    }

    /// Get level string at 0-based index.
    #[inline]
    pub fn level(&self, idx: usize) -> &'a str {
        let charsxp = unsafe { STRING_ELT(self.levels_sexp, idx as isize) };
        unsafe { charsxp_to_str(charsxp) }
    }
}

impl Deref for Factor<'_> {
    type Target = [i32];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.indices
    }
}

impl<'a> TryFromSexp for Factor<'a> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        Self::try_new(sexp)
    }
}

// =============================================================================
// FactorMut - mutable view into an R factor's data
// =============================================================================

/// A mutable borrowed view into an R factor's integer indices.
///
/// Provides `DerefMut` to `&mut [i32]` for direct mutable slice access.
/// The indices are 1-based (matching R's convention) with `NA_INTEGER` for NA.
///
/// # Example
///
/// ```ignore
/// let mut factor_mut = FactorMut::try_new(sexp)?;
/// // Set all values to level 1
/// for idx in factor_mut.iter_mut() {
///     *idx = 1;
/// }
/// ```
pub struct FactorMut<'a> {
    indices: &'a mut [i32],
    levels_sexp: SEXP,
    _marker: PhantomData<&'a mut ()>,
}

impl<'a> FactorMut<'a> {
    /// Create a FactorMut from a factor SEXP.
    ///
    /// Returns an error if the SEXP is not a factor.
    pub fn try_new(sexp: SEXP) -> Result<Self, SexpError> {
        if unsafe { Rf_isFactor(sexp) } == Rboolean::FALSE {
            return Err(SexpError::InvalidValue("expected a factor".into()));
        }

        let len = unsafe { Rf_xlength(sexp) } as usize;
        let ptr = unsafe { INTEGER(sexp) };
        let indices = unsafe { std::slice::from_raw_parts_mut(ptr, len) };
        let levels_sexp = unsafe { Rf_getAttrib(sexp, R_LevelsSymbol) };

        Ok(Self {
            indices,
            levels_sexp,
            _marker: PhantomData,
        })
    }

    /// Number of elements in the factor.
    #[inline]
    pub fn len(&self) -> usize {
        self.indices.len()
    }

    /// Whether the factor is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    /// The levels STRSXP.
    #[inline]
    pub fn levels_sexp(&self) -> SEXP {
        self.levels_sexp
    }

    /// Number of levels.
    #[inline]
    pub fn n_levels(&self) -> usize {
        unsafe { Rf_xlength(self.levels_sexp) as usize }
    }

    /// Get level string at 0-based index.
    #[inline]
    pub fn level(&self, idx: usize) -> &'a str {
        let charsxp = unsafe { STRING_ELT(self.levels_sexp, idx as isize) };
        unsafe { charsxp_to_str(charsxp) }
    }
}

impl Deref for FactorMut<'_> {
    type Target = [i32];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.indices
    }
}

impl std::ops::DerefMut for FactorMut<'_> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.indices
    }
}

// =============================================================================
// Validation helper
// =============================================================================

/// Validate that a factor has the expected levels.
pub(crate) fn validate_factor_levels(sexp: SEXP, expected: &[&str]) -> Result<(), SexpError> {
    if unsafe { Rf_isFactor(sexp) } == Rboolean::FALSE {
        return Err(SexpError::InvalidValue("expected a factor".into()));
    }

    let levels = unsafe { Rf_getAttrib(sexp, R_LevelsSymbol) };
    if levels.type_of() != SEXPTYPE::STRSXP {
        return Err(SexpError::InvalidValue("levels is not STRSXP".into()));
    }

    let n = unsafe { Rf_xlength(levels) } as usize;
    if n != expected.len() {
        return Err(SexpError::InvalidValue(format!(
            "expected {} levels, got {}",
            expected.len(),
            n
        )));
    }

    for (i, exp) in expected.iter().enumerate() {
        let charsxp = unsafe { STRING_ELT(levels, i as isize) };
        let actual = unsafe { charsxp_to_str(charsxp) };
        if actual != *exp {
            return Err(SexpError::InvalidValue(format!(
                "level {}: expected '{}', got '{}'",
                i + 1,
                exp,
                actual
            )));
        }
    }

    Ok(())
}

// =============================================================================
// Conversion helpers (used by derive macro)
// =============================================================================

/// Convert an R factor SEXP to a single enum value.
#[inline]
pub fn factor_from_sexp<T: RFactor>(sexp: SEXP) -> Result<T, SexpError> {
    validate_factor_levels(sexp, T::LEVELS)?;

    let len = unsafe { Rf_xlength(sexp) };
    if len != 1 {
        return Err(SexpError::InvalidValue(format!(
            "expected length 1, got {}",
            len
        )));
    }

    let idx = unsafe { INTEGER_ELT(sexp, 0) };
    if idx == NA_INTEGER {
        return Err(SexpError::InvalidValue("unexpected NA".into()));
    }

    T::from_level_index(idx).ok_or_else(|| SexpError::InvalidValue("index out of range".into()))
}

/// Convert an R factor SEXP to a Vec of enum values.
#[inline]
pub(crate) fn factor_vec_from_sexp<T: RFactor>(sexp: SEXP) -> Result<Vec<T>, SexpError> {
    validate_factor_levels(sexp, T::LEVELS)?;

    let len = unsafe { Rf_xlength(sexp) } as usize;
    let mut result = Vec::with_capacity(len);

    for i in 0..len {
        let idx = unsafe { INTEGER_ELT(sexp, i as isize) };
        if idx == NA_INTEGER {
            return Err(SexpError::InvalidValue(format!("NA at index {}", i)));
        }
        result.push(
            T::from_level_index(idx)
                .ok_or_else(|| SexpError::InvalidValue("index out of range".into()))?,
        );
    }

    Ok(result)
}

/// Convert an R factor SEXP to a Vec of Option enum values (NA → None).
#[inline]
pub(crate) fn factor_option_vec_from_sexp<T: RFactor>(
    sexp: SEXP,
) -> Result<Vec<Option<T>>, SexpError> {
    validate_factor_levels(sexp, T::LEVELS)?;

    let len = unsafe { Rf_xlength(sexp) } as usize;
    let mut result = Vec::with_capacity(len);

    for i in 0..len {
        let idx = unsafe { INTEGER_ELT(sexp, i as isize) };
        if idx == NA_INTEGER {
            result.push(None);
        } else {
            result.push(Some(T::from_level_index(idx).ok_or_else(|| {
                SexpError::InvalidValue("index out of range".into())
            })?));
        }
    }

    Ok(result)
}

// =============================================================================
// Newtype wrappers (for orphan rule workaround)
// =============================================================================

/// Wrapper for `Vec<T: RFactor>` enabling `IntoR`/`TryFromSexp`.
#[derive(Debug, Clone)]
pub struct FactorVec<T>(pub Vec<T>);

impl<T> FactorVec<T> {
    /// Wrap a `Vec<T>` so it can be converted to and from R factors.
    pub fn new(vec: Vec<T>) -> Self {
        Self(vec)
    }

    /// Extract the inner vector.
    pub fn into_inner(self) -> Vec<T> {
        self.0
    }
}

impl<T> From<Vec<T>> for FactorVec<T> {
    fn from(vec: Vec<T>) -> Self {
        Self(vec)
    }
}

impl<T> Deref for FactorVec<T> {
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
        let indices: Vec<i32> = self.0.iter().map(|v| v.to_level_index()).collect();
        build_factor(&indices, build_levels_sexp(T::LEVELS))
    }
}

impl<T: RFactor> TryFromSexp for FactorVec<T> {
    type Error = SexpError;
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        factor_vec_from_sexp(sexp).map(FactorVec)
    }
}

/// Wrapper for `Vec<Option<T: RFactor>>` with NA support.
#[derive(Debug, Clone)]
pub struct FactorOptionVec<T>(pub Vec<Option<T>>);

impl<T> FactorOptionVec<T> {
    /// Wrap a `Vec<Option<T>>` so it can be converted to and from R factors with NA support.
    pub fn new(vec: Vec<Option<T>>) -> Self {
        Self(vec)
    }

    /// Extract the inner vector.
    pub fn into_inner(self) -> Vec<Option<T>> {
        self.0
    }
}

impl<T> From<Vec<Option<T>>> for FactorOptionVec<T> {
    fn from(vec: Vec<Option<T>>) -> Self {
        Self(vec)
    }
}

impl<T> Deref for FactorOptionVec<T> {
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
        let indices: Vec<i32> = self
            .0
            .iter()
            .map(|v| v.map_or(NA_INTEGER, |x| x.to_level_index()))
            .collect();
        build_factor(&indices, build_levels_sexp(T::LEVELS))
    }
}

impl<T: RFactor> TryFromSexp for FactorOptionVec<T> {
    type Error = SexpError;
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        factor_option_vec_from_sexp(sexp).map(FactorOptionVec)
    }
}

// =============================================================================
// Tests
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

    // Test interaction factor (manual impl to verify logic)
    #[derive(Copy, Clone, Debug, PartialEq)]
    enum Size {
        Small,
        Large,
    }

    impl RFactor for Size {
        const LEVELS: &'static [&'static str] = &["Small", "Large"];

        fn to_level_index(self) -> i32 {
            match self {
                Size::Small => 1,
                Size::Large => 2,
            }
        }

        fn from_level_index(idx: i32) -> Option<Self> {
            match idx {
                1 => Some(Size::Small),
                2 => Some(Size::Large),
                _ => None,
            }
        }
    }

    // Manual interaction factor impl (what derive should generate)
    #[derive(Copy, Clone, Debug, PartialEq)]
    enum ColorSize {
        Red(Size),
        Green(Size),
        Blue(Size),
    }

    impl RFactor for ColorSize {
        // Levels: Red.Small, Red.Large, Green.Small, Green.Large, Blue.Small, Blue.Large
        const LEVELS: &'static [&'static str] = &[
            "Red.Small",
            "Red.Large",
            "Green.Small",
            "Green.Large",
            "Blue.Small",
            "Blue.Large",
        ];

        fn to_level_index(self) -> i32 {
            match self {
                Self::Red(inner) => {
                    let inner_idx_0 = inner.to_level_index() - 1;
                    inner_idx_0 + 1
                }
                Self::Green(inner) => {
                    let inner_idx_0 = inner.to_level_index() - 1;
                    2 + inner_idx_0 + 1
                }
                Self::Blue(inner) => {
                    let inner_idx_0 = inner.to_level_index() - 1;
                    2 * 2 + inner_idx_0 + 1
                }
            }
        }

        fn from_level_index(idx: i32) -> Option<Self> {
            match idx {
                1..=2 => {
                    let inner_idx_1 = (idx - 1) % 2 + 1;
                    Size::from_level_index(inner_idx_1).map(Self::Red)
                }
                3..=4 => {
                    let inner_idx_1 = (idx - 1) % 2 + 1;
                    Size::from_level_index(inner_idx_1).map(Self::Green)
                }
                5..=6 => {
                    let inner_idx_1 = (idx - 1) % 2 + 1;
                    Size::from_level_index(inner_idx_1).map(Self::Blue)
                }
                _ => None,
            }
        }
    }

    #[test]
    fn test_interaction_levels() {
        assert_eq!(
            ColorSize::LEVELS,
            &[
                "Red.Small",
                "Red.Large",
                "Green.Small",
                "Green.Large",
                "Blue.Small",
                "Blue.Large"
            ]
        );
    }

    #[test]
    fn test_interaction_to_index() {
        assert_eq!(ColorSize::Red(Size::Small).to_level_index(), 1);
        assert_eq!(ColorSize::Red(Size::Large).to_level_index(), 2);
        assert_eq!(ColorSize::Green(Size::Small).to_level_index(), 3);
        assert_eq!(ColorSize::Green(Size::Large).to_level_index(), 4);
        assert_eq!(ColorSize::Blue(Size::Small).to_level_index(), 5);
        assert_eq!(ColorSize::Blue(Size::Large).to_level_index(), 6);
    }

    #[test]
    fn test_interaction_from_index() {
        assert_eq!(
            ColorSize::from_level_index(1),
            Some(ColorSize::Red(Size::Small))
        );
        assert_eq!(
            ColorSize::from_level_index(2),
            Some(ColorSize::Red(Size::Large))
        );
        assert_eq!(
            ColorSize::from_level_index(3),
            Some(ColorSize::Green(Size::Small))
        );
        assert_eq!(
            ColorSize::from_level_index(4),
            Some(ColorSize::Green(Size::Large))
        );
        assert_eq!(
            ColorSize::from_level_index(5),
            Some(ColorSize::Blue(Size::Small))
        );
        assert_eq!(
            ColorSize::from_level_index(6),
            Some(ColorSize::Blue(Size::Large))
        );
        assert_eq!(ColorSize::from_level_index(0), None);
        assert_eq!(ColorSize::from_level_index(7), None);
    }

    #[test]
    fn test_interaction_roundtrip() {
        for i in 1..=6 {
            let color_size = ColorSize::from_level_index(i).unwrap();
            assert_eq!(color_size.to_level_index(), i);
        }
    }
}
