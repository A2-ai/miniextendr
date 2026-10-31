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

use std::ops::Deref;
use std::sync::OnceLock;

use crate::altrep_traits::NA_INTEGER;
use crate::from_r::{
    BatchedErrors, SexpError, SexpLengthError, SexpNaError, TryFromSexp, charsxp_to_cow_lossy,
};
use crate::gc_protect::OwnedProtect;
use crate::into_r::IntoR;
use crate::sys::Rf_allocVector;
use crate::{IsNA, SEXP, SEXPTYPE, SexpExt};

// region: Cached "factor" class STRSXP

static FACTOR_CLASS: OnceLock<SEXP> = OnceLock::new();

pub(crate) fn factor_class_sexp() -> SEXP {
    *FACTOR_CLASS.get_or_init(|| build_levels_sexp_cached(&["factor"]))
}
// endregion

// region: RFactor trait

/// Trait for mapping Rust enums to R factors.
///
/// Typically implemented via `#[derive(RFactor)]` for C-style enums.
/// The derive macro also generates `IntoR` and `TryFromSexp` implementations.
pub trait RFactor: crate::match_arg::MatchArg + Copy + 'static {
    /// Convert variant to 1-based level index.
    fn to_level_index(self) -> i32;

    /// Convert 1-based level index to variant, or `None` if out of range.
    fn from_level_index(idx: i32) -> Option<Self>;
}
// endregion

// region: Core building functions

/// Build a levels STRSXP of UTF-8 CHARSXPs.
///
/// Level names are not interned as R symbols, so names built from data stay
/// collectable. The returned STRSXP is NOT protected - caller must protect or
/// preserve it.
///
/// # Panics
///
/// If a level name contains a null byte.
pub fn build_levels_sexp(levels: &[&str]) -> SEXP {
    build_levels_sexp_protected(levels).get()
}

/// Keep the container rooted while each level's CHARSXP is allocated.
fn build_levels_sexp_protected(levels: &[&str]) -> OwnedProtect {
    // `Rf_mkCharLenCE` raises an R error on an embedded nul, which would
    // longjmp over the container's protection.
    if levels.iter().any(|level| level.as_bytes().contains(&0)) {
        panic!("level name contains null byte");
    }
    let len = levels.len().try_into().expect("too many factor levels");
    unsafe {
        let sexp = OwnedProtect::new(Rf_allocVector(SEXPTYPE::STRSXP, len));
        for (i, level) in (0..len).zip(levels) {
            // A new CHARSXP is unrooted until it is stored in the protected
            // container. R's CHARSXP cache shares repeated names.
            sexp.set_string_elt(i, SEXP::charsxp(level));
        }
        sexp
    }
}

/// Build a levels STRSXP and preserve it permanently (for caching).
pub fn build_levels_sexp_cached(levels: &[&str]) -> SEXP {
    unsafe {
        let sexp = build_levels_sexp_protected(levels);
        crate::sys::R_PreserveObject(sexp.get());
        sexp.get()
    }
}

/// Build a factor SEXP from indices and a levels STRSXP.
///
/// The caller must keep `levels` rooted across this call. The factor is rooted
/// during construction; the returned SEXP must be protected or returned to R.
pub fn build_factor(indices: &[i32], levels: SEXP) -> SEXP {
    unsafe {
        let (sexp, dst) = crate::into_r::alloc_r_vector::<i32>(indices.len());
        // Attribute assignment and the cold class-cache initializer allocate.
        let sexp = OwnedProtect::new(sexp);
        dst.copy_from_slice(indices);
        sexp.set_levels(levels);
        sexp.set_class(factor_class_sexp());
        sexp.get()
    }
}

/// Build a factor SEXP from indices and level names in a single call.
///
/// Keeps the levels STRSXP rooted from allocation through [`build_factor`],
/// so callers don't need to manage
/// the levels protection themselves. The returned factor SEXP is **not**
/// protected — caller must protect or return it.
///
/// This is the recommended path for one-shot factor construction; for
/// repeated calls with the same levels prefer caching via
/// [`build_levels_sexp_cached`] (no protection needed because the cached
/// SEXP is on R's precious list).
///
/// The levels container roots its strings; the container and the factor
/// payload are rooted while being built.
pub fn build_factor_with_levels(indices: &[i32], level_names: &[&str]) -> SEXP {
    let levels = build_levels_sexp_protected(level_names);
    build_factor(indices, levels.get())
}
// endregion

// region: Validation helper

/// Validate that a factor has the expected levels.
pub(crate) fn validate_factor_levels(sexp: SEXP, expected: &[&str]) -> Result<(), SexpError> {
    if !sexp.is_factor() {
        return Err(SexpError::InvalidValue("expected a factor".into()));
    }

    let levels = sexp.get_levels();
    if levels.type_of() != SEXPTYPE::STRSXP {
        return Err(SexpError::InvalidValue("levels is not STRSXP".into()));
    }

    let n = levels.len();
    if n != expected.len() {
        return Err(SexpError::InvalidValue(format!(
            "expected {} levels, got {}",
            expected.len(),
            n
        )));
    }

    for (i, exp) in expected.iter().enumerate() {
        let charsxp = levels.string_elt(i as isize);
        let actual = unsafe { charsxp_to_cow_lossy(charsxp) };
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
// endregion

// region: Conversion helpers (used by derive macro)

/// The reason a factor code does not name one of the levels (a malformed
/// factor: [`validate_factor_levels`] has already checked the level set).
fn code_not_a_level(code: i32) -> String {
    format!("code {code} is not a level of the factor")
}

/// Read every code of a factor through `elt`, batching every failing element
/// (its reason and 1-based position) into one error instead of stopping at
/// the first.
fn read_factor_codes<U>(
    sexp: SEXP,
    elt: impl Fn(i32) -> Result<U, String>,
) -> Result<Vec<U>, SexpError> {
    let len = sexp.len();
    let mut result = Vec::with_capacity(len);
    let mut errors = BatchedErrors::default();
    for i in 0..len {
        match elt(sexp.integer_elt(i as isize)) {
            Ok(v) => result.push(v),
            Err(reason) => errors.push(i, || reason),
        }
    }
    if errors.is_empty() {
        Ok(result)
    } else {
        Err(errors.into_element_error())
    }
}

/// Convert an R factor SEXP to a single enum value.
#[inline]
pub fn factor_from_sexp<T: RFactor>(sexp: SEXP) -> Result<T, SexpError> {
    validate_factor_levels(sexp, T::CHOICES)?;

    let len = sexp.len();
    if len != 1 {
        return Err(SexpLengthError {
            expected: 1,
            actual: len,
        }
        .into());
    }

    let idx = sexp.integer_elt(0);
    if idx.is_na() {
        return Err(SexpNaError {
            sexp_type: SEXPTYPE::INTSXP,
        }
        .into());
    }

    T::from_level_index(idx).ok_or_else(|| SexpError::InvalidValue(code_not_a_level(idx)))
}

/// Convert an R factor SEXP to a Vec of enum values. Every `NA` and every
/// code outside the levels is reported, with its position.
#[inline]
pub(crate) fn factor_vec_from_sexp<T: UnitEnumFactor>(sexp: SEXP) -> Result<Vec<T>, SexpError> {
    validate_factor_levels(sexp, T::FACTOR_LEVELS)?;
    read_factor_codes(sexp, |idx| {
        if idx.is_na() {
            return Err("NA is not allowed".to_string());
        }
        T::from_factor_index(idx).ok_or_else(|| code_not_a_level(idx))
    })
}

/// Convert an R factor SEXP to a Vec of Option enum values (NA → None).
#[inline]
pub(crate) fn factor_option_vec_from_sexp<T: RFactor>(
    sexp: SEXP,
) -> Result<Vec<Option<T>>, SexpError> {
    validate_factor_levels(sexp, T::CHOICES)?;
    read_factor_codes(sexp, |idx| {
        if idx.is_na() {
            return Ok(None);
        }
        T::from_level_index(idx)
            .map(Some)
            .ok_or_else(|| code_not_a_level(idx))
    })
}

/// Convert an R factor SEXP to a `Vec<Option<T>>` using [`UnitEnumFactor`] (NA → `None`).
///
/// Used by the enum DataFrame reader to reconstruct `as_factor` columns.
/// Unlike `factor_option_vec_from_sexp` (which requires `RFactor + MatchArg`),
/// this accepts any `UnitEnumFactor` — including `#[derive(DataFrameRow)]`
/// unit-only enums that do not implement `RFactor`.
#[inline]
pub fn unit_factor_option_vec_from_sexp<T: UnitEnumFactor>(
    sexp: SEXP,
) -> Result<Vec<Option<T>>, SexpError> {
    validate_factor_levels(sexp, T::FACTOR_LEVELS)?;
    read_factor_codes(sexp, |idx| {
        if idx.is_na() {
            return Ok(None);
        }
        T::from_factor_index(idx)
            .map(Some)
            .ok_or_else(|| code_not_a_level(idx))
    })
}
// endregion

// region: Newtype wrappers (for orphan rule workaround)

/// Wrapper for `Vec<T: UnitEnumFactor>` enabling `IntoR`/`TryFromSexp`.
///
/// Accepts both `RFactor` types and unit-only `DataFrameRow` enums. Values are
/// moved during conversion; neither `Copy` nor `Clone` is required.
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

impl<T: UnitEnumFactor> IntoR for FactorVec<T> {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::SEXP, Self::Error> {
        self.try_into_sexp()
    }
    fn into_sexp(self) -> SEXP {
        let indices: Vec<i32> = self.0.into_iter().map(T::to_factor_index).collect();
        // build_factor_with_levels protects the levels STRSXP across the
        // build_factor allocation — see "PROTECT discipline against R-devel GC".
        build_factor_with_levels(&indices, T::FACTOR_LEVELS)
    }
}

impl<T: UnitEnumFactor> TryFromSexp for FactorVec<T> {
    type Error = SexpError;
    const CHARACTER_ONLY: bool = true;
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

// Blanket: every RFactor type is also a UnitEnumFactor (provides IntoR for FactorOptionVec<T>).
impl<T: RFactor + crate::match_arg::MatchArg> UnitEnumFactor for T {
    const FACTOR_LEVELS: &'static [&'static str] = T::CHOICES;
    fn to_factor_index(self) -> i32 {
        self.to_level_index()
    }
    fn from_factor_index(idx: i32) -> Option<Self> {
        T::from_level_index(idx)
    }
}

impl<T: RFactor> TryFromSexp for FactorOptionVec<T> {
    type Error = SexpError;
    const CHARACTER_ONLY: bool = true;
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        factor_option_vec_from_sexp(sexp).map(FactorOptionVec)
    }
}

// region: UnitEnumFactor — factor trait for DataFrameRow unit-only enums

/// Trait implemented by unit-only enums derived via `#[derive(DataFrameRow)]`.
///
/// Provides the level names and 1-based index needed to convert enum values
/// into R factor SEXPs. Unlike `RFactor`, this trait does **not** require
/// `Copy` or `MatchArg`, making it usable with `DataFrameRow`-derived types
/// that only need to participate as factor columns in data frames.
///
/// Implemented automatically by `#[derive(DataFrameRow)]` on unit-only enums.
/// `FactorVec<T>` and `FactorOptionVec<T>` provide the SEXP conversion used by the
/// companion struct's `into_data_frame` method.
///
/// # Safety contract
///
/// `to_factor_index` must return a value in `1..=FACTOR_LEVELS.len() as i32`
/// (or `NA_INTEGER` for missing) to produce a valid R factor SEXP.
pub trait UnitEnumFactor: Sized {
    /// Ordered level names (in the same order as the enum variants).
    const FACTOR_LEVELS: &'static [&'static str];

    /// Convert `self` to a 1-based R factor level index.
    fn to_factor_index(self) -> i32;

    /// Inverse: 1-based level index → variant, or `None` if out of range.
    ///
    /// Used by the enum DataFrame reader to reconstruct unit-only enum values
    /// from an R factor column. The default blanket impl delegates to the
    /// underlying `RFactor::from_level_index`.
    fn from_factor_index(idx: i32) -> Option<Self>;
}

impl<T: UnitEnumFactor> IntoR for FactorOptionVec<T> {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::SEXP, Self::Error> {
        self.try_into_sexp()
    }
    fn into_sexp(self) -> SEXP {
        // Generic statics aren't allowed in Rust, so levels are built per call.
        // build_factor_with_levels protects the levels STRSXP across the
        // build_factor allocation — see "PROTECT discipline against R-devel GC".
        let indices: Vec<i32> = self
            .0
            .into_iter()
            .map(|opt| match opt {
                None => NA_INTEGER,
                Some(v) => v.to_factor_index(),
            })
            .collect();
        build_factor_with_levels(&indices, T::FACTOR_LEVELS)
    }
}
// endregion: UnitEnumFactor

// region: Tests

#[cfg(test)]
mod tests;
// endregion
