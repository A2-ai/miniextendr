//! High-level ALTREP data traits.
//!
//! These traits let you implement ALTREP behavior using `&self` methods instead of
//! raw `SEXP` callbacks. The library provides blanket implementations that handle
//! the SEXP extraction automatically.
//!
//! ## Quick Start
//!
//! For common types, just use them directly:
//!
//! ```ignore
//! // Vec<i32> already implements AltIntegerData
//! let altrep = create_altinteger(vec![1, 2, 3, 4, 5]);
//! ```
//!
//! For custom types, implement the relevant trait:
//!
//! ```ignore
//! struct Fibonacci { len: usize }
//!
//! impl AltrepLen for Fibonacci {
//!     fn len(&self) -> usize { self.len }
//! }
//!
//! impl AltIntegerData for Fibonacci {
//!     fn elt(&self, i: usize) -> i32 {
//!         // Compute fibonacci(i)
//!         ...
//!     }
//! }
//! ```

use crate::ffi::{Rcomplex, SEXP};

// =============================================================================
// Core trait: length
// =============================================================================

/// Base trait for ALTREP data types. All ALTREP types must provide length.
pub trait AltrepLen {
    /// Returns the length of this ALTREP vector.
    fn len(&self) -> usize;

    /// Returns true if the vector is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

// =============================================================================
// Integer ALTREP
// =============================================================================

/// Trait for types that can back an ALTINTEGER vector.
///
/// Implement this to create custom integer ALTREP classes.
pub trait AltIntegerData: AltrepLen {
    /// Get the integer element at index `i`.
    fn elt(&self, i: usize) -> i32;

    /// Optional: return a pointer to contiguous data if available.
    /// Default returns None (no contiguous backing).
    fn as_slice(&self) -> Option<&[i32]> {
        None
    }

    /// Optional: bulk read into buffer. Returns number of elements read.
    /// Default uses `elt()` in a loop.
    fn get_region(&self, start: usize, len: usize, buf: &mut [i32]) -> usize {
        let actual_len = len.min(buf.len()).min(self.len().saturating_sub(start));
        for (i, slot) in buf.iter_mut().enumerate().take(actual_len) {
            *slot = self.elt(start + i);
        }
        actual_len
    }

    /// Optional: sortedness hint. Default is unknown.
    fn is_sorted(&self) -> Option<Sortedness> {
        None
    }

    /// Optional: does this vector contain any NA values?
    fn no_na(&self) -> Option<bool> {
        None
    }

    /// Optional: optimized sum. Default returns None (use R's default).
    fn sum(&self, _na_rm: bool) -> Option<i64> {
        None
    }

    /// Optional: optimized min. Default returns None (use R's default).
    fn min(&self, _na_rm: bool) -> Option<i32> {
        None
    }

    /// Optional: optimized max. Default returns None (use R's default).
    fn max(&self, _na_rm: bool) -> Option<i32> {
        None
    }
}

// =============================================================================
// Real ALTREP
// =============================================================================

/// Trait for types that can back an ALTREAL vector.
pub trait AltRealData: AltrepLen {
    /// Get the real element at index `i`.
    fn elt(&self, i: usize) -> f64;

    /// Optional: return a pointer to contiguous data if available.
    fn as_slice(&self) -> Option<&[f64]> {
        None
    }

    /// Optional: bulk read into buffer.
    fn get_region(&self, start: usize, len: usize, buf: &mut [f64]) -> usize {
        let actual_len = len.min(buf.len()).min(self.len().saturating_sub(start));
        for (i, slot) in buf.iter_mut().enumerate().take(actual_len) {
            *slot = self.elt(start + i);
        }
        actual_len
    }

    /// Optional: sortedness hint.
    fn is_sorted(&self) -> Option<Sortedness> {
        None
    }

    /// Optional: does this vector contain any NA values?
    fn no_na(&self) -> Option<bool> {
        None
    }

    /// Optional: optimized sum.
    fn sum(&self, _na_rm: bool) -> Option<f64> {
        None
    }

    /// Optional: optimized min.
    fn min(&self, _na_rm: bool) -> Option<f64> {
        None
    }

    /// Optional: optimized max.
    fn max(&self, _na_rm: bool) -> Option<f64> {
        None
    }
}

// =============================================================================
// Logical ALTREP
// =============================================================================

/// Logical value: TRUE, FALSE, or NA.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Logical {
    False,
    True,
    Na,
}

impl Logical {
    /// Convert to R's integer representation.
    pub fn to_r_int(self) -> i32 {
        match self {
            Logical::False => 0,
            Logical::True => 1,
            Logical::Na => i32::MIN,
        }
    }

    /// Convert from R's integer representation.
    pub fn from_r_int(i: i32) -> Self {
        match i {
            0 => Logical::False,
            i32::MIN => Logical::Na,
            _ => Logical::True,
        }
    }

    /// Convert from Rust bool (no NA representation).
    pub fn from_bool(b: bool) -> Self {
        if b {
            Logical::True
        } else {
            Logical::False
        }
    }
}

/// Trait for types that can back an ALTLOGICAL vector.
pub trait AltLogicalData: AltrepLen {
    /// Get the logical element at index `i`.
    fn elt(&self, i: usize) -> Logical;

    /// Optional: return a slice if data is contiguous i32 (R's internal format).
    fn as_r_slice(&self) -> Option<&[i32]> {
        None
    }

    /// Optional: bulk read into buffer.
    fn get_region(&self, start: usize, len: usize, buf: &mut [i32]) -> usize {
        let actual_len = len.min(buf.len()).min(self.len().saturating_sub(start));
        for (i, slot) in buf.iter_mut().enumerate().take(actual_len) {
            *slot = self.elt(start + i).to_r_int();
        }
        actual_len
    }

    /// Optional: sortedness hint.
    fn is_sorted(&self) -> Option<Sortedness> {
        None
    }

    /// Optional: does this vector contain any NA values?
    fn no_na(&self) -> Option<bool> {
        None
    }

    /// Optional: optimized sum (count of TRUE values).
    fn sum(&self, _na_rm: bool) -> Option<i64> {
        None
    }
}

// =============================================================================
// Raw ALTREP
// =============================================================================

/// Trait for types that can back an ALTRAW vector.
pub trait AltRawData: AltrepLen {
    /// Get the raw byte at index `i`.
    fn elt(&self, i: usize) -> u8;

    /// Optional: return a slice if data is contiguous.
    fn as_slice(&self) -> Option<&[u8]> {
        None
    }

    /// Optional: bulk read into buffer.
    fn get_region(&self, start: usize, len: usize, buf: &mut [u8]) -> usize {
        let actual_len = len.min(buf.len()).min(self.len().saturating_sub(start));
        for (i, slot) in buf.iter_mut().enumerate().take(actual_len) {
            *slot = self.elt(start + i);
        }
        actual_len
    }
}

// =============================================================================
// Complex ALTREP
// =============================================================================

/// Trait for types that can back an ALTCOMPLEX vector.
pub trait AltComplexData: AltrepLen {
    /// Get the complex element at index `i`.
    fn elt(&self, i: usize) -> Rcomplex;

    /// Optional: return a slice if data is contiguous.
    fn as_slice(&self) -> Option<&[Rcomplex]> {
        None
    }

    /// Optional: bulk read into buffer.
    fn get_region(&self, start: usize, len: usize, buf: &mut [Rcomplex]) -> usize {
        let actual_len = len.min(buf.len()).min(self.len().saturating_sub(start));
        for (i, slot) in buf.iter_mut().enumerate().take(actual_len) {
            *slot = self.elt(start + i);
        }
        actual_len
    }
}

// =============================================================================
// String ALTREP
// =============================================================================

/// Trait for types that can back an ALTSTRING vector.
///
/// Note: `elt` returns a `&str` which will be converted to CHARSXP.
pub trait AltStringData: AltrepLen {
    /// Get the string element at index `i`.
    ///
    /// Return `None` for NA values.
    fn elt(&self, i: usize) -> Option<&str>;

    /// Optional: sortedness hint.
    fn is_sorted(&self) -> Option<Sortedness> {
        None
    }

    /// Optional: does this vector contain any NA values?
    fn no_na(&self) -> Option<bool> {
        None
    }
}

// =============================================================================
// List ALTREP
// =============================================================================

/// Trait for types that can back an ALTLIST vector.
///
/// List elements are arbitrary SEXPs, so this trait works with raw SEXP.
pub trait AltListData: AltrepLen {
    /// Get the list element at index `i`.
    ///
    /// Returns a SEXP (any R object).
    fn elt(&self, i: usize) -> SEXP;
}

// =============================================================================
// Sortedness enum
// =============================================================================

/// Sortedness hint for ALTREP vectors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sortedness {
    /// Unknown sortedness.
    Unknown,
    /// Not sorted.
    None,
    /// Sorted in increasing order (may have ties).
    Increasing,
    /// Sorted in decreasing order (may have ties).
    Decreasing,
    /// Strictly increasing (no ties).
    StrictlyIncreasing,
    /// Strictly decreasing (no ties).
    StrictlyDecreasing,
}

impl Sortedness {
    /// Convert to R's integer representation.
    pub fn to_r_int(self) -> i32 {
        match self {
            Sortedness::Unknown => i32::MIN,
            Sortedness::None => 0,
            Sortedness::Increasing => 1,
            Sortedness::Decreasing => -1,
            Sortedness::StrictlyIncreasing => 2,
            Sortedness::StrictlyDecreasing => -2,
        }
    }

    /// Convert from R's integer representation.
    pub fn from_r_int(i: i32) -> Self {
        match i {
            i32::MIN => Sortedness::Unknown,
            0 => Sortedness::None,
            1 => Sortedness::Increasing,
            -1 => Sortedness::Decreasing,
            2 => Sortedness::StrictlyIncreasing,
            -2 => Sortedness::StrictlyDecreasing,
            _ => Sortedness::Unknown,
        }
    }
}

// =============================================================================
// Optional dataptr trait (separate from element access)
// =============================================================================

/// Trait for types that can provide a mutable data pointer.
///
/// This is separate from element access because some ALTREP types
/// compute elements on-the-fly but can materialize to a buffer.
///
/// ## Lazy Materialization Pattern
///
/// For types that compute values lazily (e.g., arithmetic sequences, Fibonacci),
/// you can implement lazy materialization by:
///
/// 1. Store an `Option<Vec<T>>` for the materialized buffer
/// 2. In `dataptr()`, compute all values and cache them
/// 3. In `dataptr_or_null()`, return `None` until materialized
///
/// ```ignore
/// struct LazySequence {
///     start: i32,
///     step: i32,
///     len: usize,
///     materialized: Option<Vec<i32>>,
/// }
///
/// impl AltrepDataptr<i32> for LazySequence {
///     fn dataptr(&mut self, _writable: bool) -> Option<*mut i32> {
///         // Materialize on first access
///         if self.materialized.is_none() {
///             let data: Vec<i32> = (0..self.len)
///                 .map(|i| self.start + (i as i32) * self.step)
///                 .collect();
///             self.materialized = Some(data);
///         }
///         self.materialized.as_mut().map(|v| v.as_mut_ptr())
///     }
///
///     fn dataptr_or_null(&self) -> Option<*const i32> {
///         // Only return pointer if already materialized
///         self.materialized.as_ref().map(|v| v.as_ptr())
///     }
/// }
/// ```
pub trait AltrepDataptr<T> {
    /// Get a mutable pointer to the underlying data.
    ///
    /// If `writable` is true, R may modify the data.
    /// Return `None` if data cannot be accessed as a contiguous buffer.
    ///
    /// This method may trigger materialization of lazy data.
    fn dataptr(&mut self, writable: bool) -> Option<*mut T>;

    /// Get a read-only pointer without forcing materialization.
    ///
    /// Return `None` if data is not already materialized or cannot provide
    /// a contiguous buffer. R will fall back to element-by-element access
    /// via `Elt` when this returns `None`.
    fn dataptr_or_null(&self) -> Option<*const T> {
        None
    }
}

// =============================================================================
// Serialization trait
// =============================================================================

/// Trait for ALTREP types that support serialization.
///
/// When an ALTREP object is saved (e.g., with `saveRDS()`), R calls `serialized_state`
/// to get a representation that can be saved. When loaded, R calls `unserialize`
/// to reconstruct the ALTREP object from that state.
///
/// ## How It Works
///
/// 1. **Saving**: R calls `serialized_state(x)` which should return an R object
///    (typically a list or vector) containing all data needed to reconstruct the ALTREP.
///
/// 2. **Loading**: R calls `unserialize(class, state)` where `state` is what
///    `serialized_state` returned. You reconstruct your ALTREP object from this.
///
/// ## Example
///
/// ```ignore
/// use miniextendr_api::ffi::{SEXP, Rf_allocVector, INTSXP, SET_INTEGER_ELT, INTEGER_ELT};
///
/// impl AltrepSerialize for ArithSeqData {
///     fn serialized_state(&self) -> SEXP {
///         // Store start, step, len in an integer vector
///         unsafe {
///             let state = Rf_allocVector(INTSXP, 3);
///             SET_INTEGER_ELT(state, 0, self.start);
///             SET_INTEGER_ELT(state, 1, self.step);
///             SET_INTEGER_ELT(state, 2, self.len as i32);
///             state
///         }
///     }
///
///     fn unserialize(state: SEXP) -> Option<Self> {
///         unsafe {
///             let start = INTEGER_ELT(state, 0);
///             let step = INTEGER_ELT(state, 1);
///             let len = INTEGER_ELT(state, 2) as usize;
///             Some(ArithSeqData { start, step, len })
///         }
///     }
/// }
/// ```
///
/// ## Notes
///
/// - The serialized state should be a standard R object (list, vector, etc.)
/// - Avoid storing pointers or handles that won't survive serialization
/// - For lazy types, decide whether to serialize the computed values or the parameters
pub trait AltrepSerialize: Sized {
    /// Convert the ALTREP data to a serializable R object.
    ///
    /// This is called when R needs to save the ALTREP (e.g., `saveRDS()`).
    /// Return an R object that contains all information needed to reconstruct
    /// the ALTREP on load.
    fn serialized_state(&self) -> SEXP;

    /// Reconstruct the ALTREP data from a serialized state.
    ///
    /// This is called when R loads a serialized ALTREP (e.g., `readRDS()`).
    /// The `state` parameter is what `serialized_state()` returned.
    ///
    /// Return `None` if the state is invalid or cannot be deserialized.
    fn unserialize(state: SEXP) -> Option<Self>;
}

/// Trait for creating an ALTREP SEXP from serialized state.
///
/// This is used by the `unserialize` callback to reconstruct the ALTREP object.
/// It combines data unserialization with ALTREP instance creation.
///
/// This trait is automatically implemented by the `#[miniextendr]` macro
/// for wrapper classes when the inner data type implements `AltrepSerialize`.
pub trait AltrepUnserialize {
    /// Create an ALTREP SEXP from serialized state.
    ///
    /// # Safety
    /// Must be called from the R main thread during unserialization.
    unsafe fn from_serialized_state(
        class: crate::ffi::altrep::R_altrep_class_t,
        state: SEXP,
    ) -> SEXP;
}

// =============================================================================
// Extract_subset optimization trait
// =============================================================================

/// Trait for ALTREP types that can provide optimized subsetting.
///
/// When R subsets an ALTREP (e.g., `x[1:10]`), it can call `Extract_subset` to get
/// an optimized result. This is useful for:
///
/// - **Arithmetic sequences**: `seq(1, 1000000)[1:10]` can return a new sequence
///   instead of materializing the full million elements
/// - **Lazy types**: Can return another lazy object covering just the subset
/// - **Memory-mapped files**: Can return a view without loading everything
///
/// ## Example
///
/// ```ignore
/// impl AltrepExtractSubset for ArithSeqData {
///     fn extract_subset(&self, indices: &[i32]) -> Option<SEXP> {
///         // For simple contiguous subsets like 1:10, we could return a new ArithSeq
///         // For general subsets, return None to let R handle it
///         None
///     }
/// }
/// ```
///
/// ## Notes
///
/// - `indices` contains 1-based R indices (may include NA as i32::MIN)
/// - Return `None` to let R use default subsetting
/// - Return `Some(sexp)` with the subset result
pub trait AltrepExtractSubset {
    /// Extract a subset of this ALTREP.
    ///
    /// `indices` contains the 1-based indices to extract.
    /// Return `None` to fall back to R's default subsetting.
    fn extract_subset(&self, indices: &[i32]) -> Option<SEXP>;
}

// =============================================================================
// InferBase trait - automatic base type inference from data traits
// =============================================================================

/// Trait for inferring the R base type from a data type's implemented traits.
///
/// This is automatically implemented via blanket impls for types that implement
/// one of the `Alt*Data` traits. It allows the `#[miniextendr]` macro to infer
/// the base type without requiring `base = "..."` or manual `AltrepBase` impl.
///
/// # Example
///
/// ```ignore
/// // ConstantIntData implements AltIntegerData, so InferBase is auto-implemented
/// impl AltIntegerData for ConstantIntData { ... }
///
/// // Now the macro can infer the base type:
/// #[miniextendr(class = "ConstantInt", pkg = "rpkg")]  // No base needed!
/// pub struct ConstantIntClass(ConstantIntData);
/// ```
pub trait InferBase {
    /// The inferred R base type.
    const BASE: crate::altrep::RBase;

    /// Create the ALTREP class handle.
    ///
    /// # Safety
    /// Must be called during R initialization.
    unsafe fn make_class(
        class_name: *const i8,
        pkg_name: *const i8,
    ) -> crate::ffi::altrep::R_altrep_class_t;

    /// Install ALTREP methods on the class.
    ///
    /// # Safety
    /// Must be called during R initialization with a valid class handle.
    unsafe fn install_methods(cls: crate::ffi::altrep::R_altrep_class_t);
}

/// Implement `InferBase` for an integer ALTREP data type.
///
/// This macro should be called after `impl_altinteger_from_data!` to enable
/// automatic base type inference in the `#[miniextendr]` macro.
#[macro_export]
macro_rules! impl_inferbase_integer {
    ($ty:ty) => {
        impl $crate::altrep_data::InferBase for $ty {
            const BASE: $crate::altrep::RBase = $crate::altrep::RBase::Int;

            unsafe fn make_class(
                class_name: *const i8,
                pkg_name: *const i8,
            ) -> $crate::ffi::altrep::R_altrep_class_t {
                unsafe {
                    $crate::ffi::altrep::R_make_altinteger_class(
                        class_name,
                        pkg_name,
                        core::ptr::null_mut(),
                    )
                }
            }

            unsafe fn install_methods(cls: $crate::ffi::altrep::R_altrep_class_t) {
                use $crate::altrep_traits::*;
                use $crate::ffi::altrep::*;
                use $crate::altrep_bridge as bridge;

                // ALTREP base methods
                unsafe { R_set_altrep_Length_method(cls, Some(bridge::t_length::<$ty>)) };

                if <$ty as Altrep>::HAS_SERIALIZED_STATE {
                    unsafe { R_set_altrep_Serialized_state_method(cls, Some(bridge::t_serialized_state::<$ty>)) };
                }

                // AltVec methods
                if <$ty as AltVec>::HAS_DATAPTR {
                    unsafe { R_set_altvec_Dataptr_method(cls, Some(bridge::t_dataptr::<$ty>)) };
                }
                if <$ty as AltVec>::HAS_DATAPTR_OR_NULL {
                    unsafe { R_set_altvec_Dataptr_or_null_method(cls, Some(bridge::t_dataptr_or_null::<$ty>)) };
                }
                if <$ty as AltVec>::HAS_EXTRACT_SUBSET {
                    unsafe { R_set_altvec_Extract_subset_method(cls, Some(bridge::t_extract_subset::<$ty>)) };
                }

                // AltInteger methods
                if <$ty as AltInteger>::HAS_ELT {
                    unsafe { R_set_altinteger_Elt_method(cls, Some(bridge::t_int_elt::<$ty>)) };
                }
                if <$ty as AltInteger>::HAS_GET_REGION {
                    unsafe { R_set_altinteger_Get_region_method(cls, Some(bridge::t_int_get_region::<$ty>)) };
                }
                if <$ty as AltInteger>::HAS_IS_SORTED {
                    unsafe { R_set_altinteger_Is_sorted_method(cls, Some(bridge::t_int_is_sorted::<$ty>)) };
                }
                if <$ty as AltInteger>::HAS_NO_NA {
                    unsafe { R_set_altinteger_No_NA_method(cls, Some(bridge::t_int_no_na::<$ty>)) };
                }
                if <$ty as AltInteger>::HAS_SUM {
                    unsafe { R_set_altinteger_Sum_method(cls, Some(bridge::t_int_sum::<$ty>)) };
                }
                if <$ty as AltInteger>::HAS_MIN {
                    unsafe { R_set_altinteger_Min_method(cls, Some(bridge::t_int_min::<$ty>)) };
                }
                if <$ty as AltInteger>::HAS_MAX {
                    unsafe { R_set_altinteger_Max_method(cls, Some(bridge::t_int_max::<$ty>)) };
                }
            }
        }
    };
}

/// Implement `InferBase` for a real ALTREP data type.
#[macro_export]
macro_rules! impl_inferbase_real {
    ($ty:ty) => {
        impl $crate::altrep_data::InferBase for $ty {
            const BASE: $crate::altrep::RBase = $crate::altrep::RBase::Real;

            unsafe fn make_class(
                class_name: *const i8,
                pkg_name: *const i8,
            ) -> $crate::ffi::altrep::R_altrep_class_t {
                unsafe {
                    $crate::ffi::altrep::R_make_altreal_class(
                        class_name,
                        pkg_name,
                        core::ptr::null_mut(),
                    )
                }
            }

            unsafe fn install_methods(cls: $crate::ffi::altrep::R_altrep_class_t) {
                use $crate::altrep_traits::*;
                use $crate::ffi::altrep::*;
                use $crate::altrep_bridge as bridge;

                unsafe { R_set_altrep_Length_method(cls, Some(bridge::t_length::<$ty>)) };

                if <$ty as Altrep>::HAS_SERIALIZED_STATE {
                    unsafe { R_set_altrep_Serialized_state_method(cls, Some(bridge::t_serialized_state::<$ty>)) };
                }

                if <$ty as AltVec>::HAS_DATAPTR {
                    unsafe { R_set_altvec_Dataptr_method(cls, Some(bridge::t_dataptr::<$ty>)) };
                }
                if <$ty as AltVec>::HAS_DATAPTR_OR_NULL {
                    unsafe { R_set_altvec_Dataptr_or_null_method(cls, Some(bridge::t_dataptr_or_null::<$ty>)) };
                }
                if <$ty as AltVec>::HAS_EXTRACT_SUBSET {
                    unsafe { R_set_altvec_Extract_subset_method(cls, Some(bridge::t_extract_subset::<$ty>)) };
                }

                if <$ty as AltReal>::HAS_ELT {
                    unsafe { R_set_altreal_Elt_method(cls, Some(bridge::t_real_elt::<$ty>)) };
                }
                if <$ty as AltReal>::HAS_GET_REGION {
                    unsafe { R_set_altreal_Get_region_method(cls, Some(bridge::t_real_get_region::<$ty>)) };
                }
                if <$ty as AltReal>::HAS_IS_SORTED {
                    unsafe { R_set_altreal_Is_sorted_method(cls, Some(bridge::t_real_is_sorted::<$ty>)) };
                }
                if <$ty as AltReal>::HAS_NO_NA {
                    unsafe { R_set_altreal_No_NA_method(cls, Some(bridge::t_real_no_na::<$ty>)) };
                }
                if <$ty as AltReal>::HAS_SUM {
                    unsafe { R_set_altreal_Sum_method(cls, Some(bridge::t_real_sum::<$ty>)) };
                }
                if <$ty as AltReal>::HAS_MIN {
                    unsafe { R_set_altreal_Min_method(cls, Some(bridge::t_real_min::<$ty>)) };
                }
                if <$ty as AltReal>::HAS_MAX {
                    unsafe { R_set_altreal_Max_method(cls, Some(bridge::t_real_max::<$ty>)) };
                }
            }
        }
    };
}

/// Implement `InferBase` for a logical ALTREP data type.
#[macro_export]
macro_rules! impl_inferbase_logical {
    ($ty:ty) => {
        impl $crate::altrep_data::InferBase for $ty {
            const BASE: $crate::altrep::RBase = $crate::altrep::RBase::Logical;

            unsafe fn make_class(
                class_name: *const i8,
                pkg_name: *const i8,
            ) -> $crate::ffi::altrep::R_altrep_class_t {
                unsafe {
                    $crate::ffi::altrep::R_make_altlogical_class(
                        class_name,
                        pkg_name,
                        core::ptr::null_mut(),
                    )
                }
            }

            unsafe fn install_methods(cls: $crate::ffi::altrep::R_altrep_class_t) {
                use $crate::altrep_traits::*;
                use $crate::ffi::altrep::*;
                use $crate::altrep_bridge as bridge;

                unsafe { R_set_altrep_Length_method(cls, Some(bridge::t_length::<$ty>)) };

                if <$ty as AltLogical>::HAS_ELT {
                    unsafe { R_set_altlogical_Elt_method(cls, Some(bridge::t_lgl_elt::<$ty>)) };
                }
                if <$ty as AltLogical>::HAS_GET_REGION {
                    unsafe { R_set_altlogical_Get_region_method(cls, Some(bridge::t_lgl_get_region::<$ty>)) };
                }
                if <$ty as AltLogical>::HAS_IS_SORTED {
                    unsafe { R_set_altlogical_Is_sorted_method(cls, Some(bridge::t_lgl_is_sorted::<$ty>)) };
                }
                if <$ty as AltLogical>::HAS_NO_NA {
                    unsafe { R_set_altlogical_No_NA_method(cls, Some(bridge::t_lgl_no_na::<$ty>)) };
                }
            }
        }
    };
}

/// Implement `InferBase` for a raw ALTREP data type.
#[macro_export]
macro_rules! impl_inferbase_raw {
    ($ty:ty) => {
        impl $crate::altrep_data::InferBase for $ty {
            const BASE: $crate::altrep::RBase = $crate::altrep::RBase::Raw;

            unsafe fn make_class(
                class_name: *const i8,
                pkg_name: *const i8,
            ) -> $crate::ffi::altrep::R_altrep_class_t {
                unsafe {
                    $crate::ffi::altrep::R_make_altraw_class(
                        class_name,
                        pkg_name,
                        core::ptr::null_mut(),
                    )
                }
            }

            unsafe fn install_methods(cls: $crate::ffi::altrep::R_altrep_class_t) {
                use $crate::altrep_traits::*;
                use $crate::ffi::altrep::*;
                use $crate::altrep_bridge as bridge;

                unsafe { R_set_altrep_Length_method(cls, Some(bridge::t_length::<$ty>)) };

                if <$ty as AltRaw>::HAS_ELT {
                    unsafe { R_set_altraw_Elt_method(cls, Some(bridge::t_raw_elt::<$ty>)) };
                }
                if <$ty as AltRaw>::HAS_GET_REGION {
                    unsafe { R_set_altraw_Get_region_method(cls, Some(bridge::t_raw_get_region::<$ty>)) };
                }
            }
        }
    };
}

/// Implement `InferBase` for a string ALTREP data type.
#[macro_export]
macro_rules! impl_inferbase_string {
    ($ty:ty) => {
        impl $crate::altrep_data::InferBase for $ty {
            const BASE: $crate::altrep::RBase = $crate::altrep::RBase::String;

            unsafe fn make_class(
                class_name: *const i8,
                pkg_name: *const i8,
            ) -> $crate::ffi::altrep::R_altrep_class_t {
                unsafe {
                    $crate::ffi::altrep::R_make_altstring_class(
                        class_name,
                        pkg_name,
                        core::ptr::null_mut(),
                    )
                }
            }

            unsafe fn install_methods(cls: $crate::ffi::altrep::R_altrep_class_t) {
                use $crate::altrep_traits::*;
                use $crate::ffi::altrep::*;
                use $crate::altrep_bridge as bridge;

                unsafe { R_set_altrep_Length_method(cls, Some(bridge::t_length::<$ty>)) };
                unsafe { R_set_altstring_Elt_method(cls, Some(bridge::t_str_elt::<$ty>)) };

                if <$ty as AltString>::HAS_IS_SORTED {
                    unsafe { R_set_altstring_Is_sorted_method(cls, Some(bridge::t_str_is_sorted::<$ty>)) };
                }
                if <$ty as AltString>::HAS_NO_NA {
                    unsafe { R_set_altstring_No_NA_method(cls, Some(bridge::t_str_no_na::<$ty>)) };
                }
                if <$ty as AltString>::HAS_SET_ELT {
                    unsafe { R_set_altstring_Set_elt_method(cls, Some(bridge::t_str_set_elt::<$ty>)) };
                }
            }
        }
    };
}

/// Implement `InferBase` for a complex ALTREP data type.
#[macro_export]
macro_rules! impl_inferbase_complex {
    ($ty:ty) => {
        impl $crate::altrep_data::InferBase for $ty {
            const BASE: $crate::altrep::RBase = $crate::altrep::RBase::Complex;

            unsafe fn make_class(
                class_name: *const i8,
                pkg_name: *const i8,
            ) -> $crate::ffi::altrep::R_altrep_class_t {
                unsafe {
                    $crate::ffi::altrep::R_make_altcomplex_class(
                        class_name,
                        pkg_name,
                        core::ptr::null_mut(),
                    )
                }
            }

            unsafe fn install_methods(cls: $crate::ffi::altrep::R_altrep_class_t) {
                use $crate::altrep_traits::*;
                use $crate::ffi::altrep::*;
                use $crate::altrep_bridge as bridge;

                unsafe { R_set_altrep_Length_method(cls, Some(bridge::t_length::<$ty>)) };

                if <$ty as AltComplex>::HAS_ELT {
                    unsafe { R_set_altcomplex_Elt_method(cls, Some(bridge::t_cplx_elt::<$ty>)) };
                }
                if <$ty as AltComplex>::HAS_GET_REGION {
                    unsafe { R_set_altcomplex_Get_region_method(cls, Some(bridge::t_cplx_get_region::<$ty>)) };
                }
            }
        }
    };
}

/// Implement `InferBase` for a list ALTREP data type.
#[macro_export]
macro_rules! impl_inferbase_list {
    ($ty:ty) => {
        impl $crate::altrep_data::InferBase for $ty {
            const BASE: $crate::altrep::RBase = $crate::altrep::RBase::List;

            unsafe fn make_class(
                class_name: *const i8,
                pkg_name: *const i8,
            ) -> $crate::ffi::altrep::R_altrep_class_t {
                unsafe {
                    $crate::ffi::altrep::R_make_altlist_class(
                        class_name,
                        pkg_name,
                        core::ptr::null_mut(),
                    )
                }
            }

            unsafe fn install_methods(cls: $crate::ffi::altrep::R_altrep_class_t) {
                use $crate::altrep_traits::*;
                use $crate::ffi::altrep::*;
                use $crate::altrep_bridge as bridge;

                unsafe { R_set_altrep_Length_method(cls, Some(bridge::t_length::<$ty>)) };
                unsafe { R_set_altlist_Elt_method(cls, Some(bridge::t_list_elt::<$ty>)) };

                if <$ty as AltList>::HAS_SET_ELT {
                    unsafe { R_set_altlist_Set_elt_method(cls, Some(bridge::t_list_set_elt::<$ty>)) };
                }
            }
        }
    };
}

// =============================================================================
// Built-in implementations for Vec<T>
// =============================================================================

impl AltrepLen for Vec<i32> {
    fn len(&self) -> usize {
        Vec::len(self)
    }
}

impl AltIntegerData for Vec<i32> {
    fn elt(&self, i: usize) -> i32 {
        self[i]
    }

    fn as_slice(&self) -> Option<&[i32]> {
        Some(self.as_slice())
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [i32]) -> usize {
        let end = (start + len).min(self.len());
        let actual_len = end.saturating_sub(start);
        if actual_len > 0 {
            buf[..actual_len].copy_from_slice(&self[start..end]);
        }
        actual_len
    }

    fn no_na(&self) -> Option<bool> {
        Some(!self.contains(&i32::MIN))
    }

    fn sum(&self, na_rm: bool) -> Option<i64> {
        let mut sum: i64 = 0;
        for &x in self.iter() {
            if x == i32::MIN {
                if !na_rm {
                    return None; // NA propagates
                }
            } else {
                sum += x as i64;
            }
        }
        Some(sum)
    }

    fn min(&self, na_rm: bool) -> Option<i32> {
        let mut min = i32::MAX;
        let mut found = false;
        for &x in self.iter() {
            if x == i32::MIN {
                if !na_rm {
                    return None;
                }
            } else {
                found = true;
                min = min.min(x);
            }
        }
        if found {
            Some(min)
        } else {
            None
        }
    }

    fn max(&self, na_rm: bool) -> Option<i32> {
        let mut max = i32::MIN + 1; // Avoid NA sentinel
        let mut found = false;
        for &x in self.iter() {
            if x == i32::MIN {
                if !na_rm {
                    return None;
                }
            } else {
                found = true;
                max = max.max(x);
            }
        }
        if found {
            Some(max)
        } else {
            None
        }
    }
}

impl AltrepDataptr<i32> for Vec<i32> {
    fn dataptr(&mut self, _writable: bool) -> Option<*mut i32> {
        Some(self.as_mut_ptr())
    }

    fn dataptr_or_null(&self) -> Option<*const i32> {
        Some(self.as_ptr())
    }
}

impl AltrepLen for Vec<f64> {
    fn len(&self) -> usize {
        Vec::len(self)
    }
}

impl AltRealData for Vec<f64> {
    fn elt(&self, i: usize) -> f64 {
        self[i]
    }

    fn as_slice(&self) -> Option<&[f64]> {
        Some(self.as_slice())
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [f64]) -> usize {
        let end = (start + len).min(self.len());
        let actual_len = end.saturating_sub(start);
        if actual_len > 0 {
            buf[..actual_len].copy_from_slice(&self[start..end]);
        }
        actual_len
    }

    fn no_na(&self) -> Option<bool> {
        Some(!self.iter().any(|x| x.is_nan()))
    }

    fn sum(&self, na_rm: bool) -> Option<f64> {
        let mut sum = 0.0;
        for &x in self.iter() {
            if x.is_nan() {
                if !na_rm {
                    return Some(f64::NAN);
                }
            } else {
                sum += x;
            }
        }
        Some(sum)
    }

    fn min(&self, na_rm: bool) -> Option<f64> {
        let mut min = f64::INFINITY;
        let mut found = false;
        for &x in self.iter() {
            if x.is_nan() {
                if !na_rm {
                    return Some(f64::NAN);
                }
            } else {
                found = true;
                min = min.min(x);
            }
        }
        if found {
            Some(min)
        } else {
            None
        }
    }

    fn max(&self, na_rm: bool) -> Option<f64> {
        let mut max = f64::NEG_INFINITY;
        let mut found = false;
        for &x in self.iter() {
            if x.is_nan() {
                if !na_rm {
                    return Some(f64::NAN);
                }
            } else {
                found = true;
                max = max.max(x);
            }
        }
        if found {
            Some(max)
        } else {
            None
        }
    }
}

impl AltrepDataptr<f64> for Vec<f64> {
    fn dataptr(&mut self, _writable: bool) -> Option<*mut f64> {
        Some(self.as_mut_ptr())
    }

    fn dataptr_or_null(&self) -> Option<*const f64> {
        Some(self.as_ptr())
    }
}

impl AltrepLen for Vec<u8> {
    fn len(&self) -> usize {
        Vec::len(self)
    }
}

impl AltRawData for Vec<u8> {
    fn elt(&self, i: usize) -> u8 {
        self[i]
    }

    fn as_slice(&self) -> Option<&[u8]> {
        Some(self.as_slice())
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [u8]) -> usize {
        let end = (start + len).min(self.len());
        let actual_len = end.saturating_sub(start);
        if actual_len > 0 {
            buf[..actual_len].copy_from_slice(&self[start..end]);
        }
        actual_len
    }
}

impl AltrepDataptr<u8> for Vec<u8> {
    fn dataptr(&mut self, _writable: bool) -> Option<*mut u8> {
        Some(self.as_mut_ptr())
    }

    fn dataptr_or_null(&self) -> Option<*const u8> {
        Some(self.as_ptr())
    }
}

impl AltrepLen for Vec<String> {
    fn len(&self) -> usize {
        Vec::len(self)
    }
}

impl AltStringData for Vec<String> {
    fn elt(&self, i: usize) -> Option<&str> {
        Some(self[i].as_str())
    }

    fn no_na(&self) -> Option<bool> {
        Some(true) // String vectors don't have NA
    }
}

impl AltrepLen for Vec<Option<String>> {
    fn len(&self) -> usize {
        Vec::len(self)
    }
}

impl AltStringData for Vec<Option<String>> {
    fn elt(&self, i: usize) -> Option<&str> {
        self[i].as_deref()
    }

    fn no_na(&self) -> Option<bool> {
        Some(!self.iter().any(|x| x.is_none()))
    }
}

impl AltrepLen for Vec<bool> {
    fn len(&self) -> usize {
        Vec::len(self)
    }
}

impl AltLogicalData for Vec<bool> {
    fn elt(&self, i: usize) -> Logical {
        if self[i] {
            Logical::True
        } else {
            Logical::False
        }
    }

    fn no_na(&self) -> Option<bool> {
        Some(true) // bool can't be NA
    }

    fn sum(&self, _na_rm: bool) -> Option<i64> {
        Some(self.iter().filter(|&&x| x).count() as i64)
    }
}

// =============================================================================
// Built-in implementations for Box<[T]> (owned slices)
// =============================================================================
// Box<[T]> is a fat pointer (Sized) that wraps a DST slice.
// Unlike Vec<T>, it has no capacity field - just ptr + len (2 words).
// This makes it more memory-efficient for fixed-size data.
//
// Box<[T]> CAN be used directly with ALTREP via the proc-macro:
// ```
// #[miniextendr(class = "BoxedInts", pkg = "mypkg")]
// pub struct BoxedIntsClass(Box<[i32]>);
// ```
//
// Or use these trait implementations in custom wrapper structs.

impl AltrepLen for Box<[i32]> {
    fn len(&self) -> usize {
        <[i32]>::len(self)
    }
}

impl AltIntegerData for Box<[i32]> {
    fn elt(&self, i: usize) -> i32 {
        self[i]
    }

    fn as_slice(&self) -> Option<&[i32]> {
        Some(self)
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [i32]) -> usize {
        let end = (start + len).min(<[i32]>::len(self));
        let actual_len = end.saturating_sub(start);
        if actual_len > 0 {
            buf[..actual_len].copy_from_slice(&self[start..end]);
        }
        actual_len
    }

    fn no_na(&self) -> Option<bool> {
        Some(!self.contains(&i32::MIN))
    }

    fn sum(&self, na_rm: bool) -> Option<i64> {
        let mut sum: i64 = 0;
        for &x in self.iter() {
            if x == i32::MIN {
                if !na_rm {
                    return None;
                }
            } else {
                sum += x as i64;
            }
        }
        Some(sum)
    }

    fn min(&self, na_rm: bool) -> Option<i32> {
        let mut min = i32::MAX;
        let mut found = false;
        for &x in self.iter() {
            if x == i32::MIN {
                if !na_rm {
                    return None;
                }
            } else {
                found = true;
                min = min.min(x);
            }
        }
        if found { Some(min) } else { None }
    }

    fn max(&self, na_rm: bool) -> Option<i32> {
        let mut max = i32::MIN + 1; // i32::MIN is NA
        let mut found = false;
        for &x in self.iter() {
            if x == i32::MIN {
                if !na_rm {
                    return None;
                }
            } else {
                found = true;
                max = max.max(x);
            }
        }
        if found { Some(max) } else { None }
    }
}

impl AltrepDataptr<i32> for Box<[i32]> {
    fn dataptr(&mut self, _writable: bool) -> Option<*mut i32> {
        Some(self.as_mut_ptr())
    }

    fn dataptr_or_null(&self) -> Option<*const i32> {
        Some(self.as_ptr())
    }
}

impl AltrepLen for Box<[f64]> {
    fn len(&self) -> usize {
        <[f64]>::len(self)
    }
}

impl AltRealData for Box<[f64]> {
    fn elt(&self, i: usize) -> f64 {
        self[i]
    }

    fn as_slice(&self) -> Option<&[f64]> {
        Some(self)
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [f64]) -> usize {
        let end = (start + len).min(<[f64]>::len(self));
        let actual_len = end.saturating_sub(start);
        if actual_len > 0 {
            buf[..actual_len].copy_from_slice(&self[start..end]);
        }
        actual_len
    }

    fn no_na(&self) -> Option<bool> {
        Some(!self.iter().any(|x| x.is_nan()))
    }

    fn sum(&self, na_rm: bool) -> Option<f64> {
        let mut sum = 0.0;
        for &x in self.iter() {
            if x.is_nan() {
                if !na_rm { return Some(f64::NAN); }
            } else {
                sum += x;
            }
        }
        Some(sum)
    }

    fn min(&self, na_rm: bool) -> Option<f64> {
        let mut min = f64::INFINITY;
        let mut found = false;
        for &x in self.iter() {
            if x.is_nan() {
                if !na_rm { return Some(f64::NAN); }
            } else {
                found = true;
                min = min.min(x);
            }
        }
        if found { Some(min) } else { None }
    }

    fn max(&self, na_rm: bool) -> Option<f64> {
        let mut max = f64::NEG_INFINITY;
        let mut found = false;
        for &x in self.iter() {
            if x.is_nan() {
                if !na_rm { return Some(f64::NAN); }
            } else {
                found = true;
                max = max.max(x);
            }
        }
        if found { Some(max) } else { None }
    }
}

impl AltrepDataptr<f64> for Box<[f64]> {
    fn dataptr(&mut self, _writable: bool) -> Option<*mut f64> {
        Some(self.as_mut_ptr())
    }

    fn dataptr_or_null(&self) -> Option<*const f64> {
        Some(self.as_ptr())
    }
}

impl AltrepLen for Box<[u8]> {
    fn len(&self) -> usize {
        <[u8]>::len(self)
    }
}

impl AltRawData for Box<[u8]> {
    fn elt(&self, i: usize) -> u8 {
        self[i]
    }

    fn as_slice(&self) -> Option<&[u8]> {
        Some(self)
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [u8]) -> usize {
        let end = (start + len).min(<[u8]>::len(self));
        let actual_len = end.saturating_sub(start);
        if actual_len > 0 {
            buf[..actual_len].copy_from_slice(&self[start..end]);
        }
        actual_len
    }
}

impl AltrepDataptr<u8> for Box<[u8]> {
    fn dataptr(&mut self, _writable: bool) -> Option<*mut u8> {
        Some(self.as_mut_ptr())
    }

    fn dataptr_or_null(&self) -> Option<*const u8> {
        Some(self.as_ptr())
    }
}

impl AltrepLen for Box<[bool]> {
    fn len(&self) -> usize {
        <[bool]>::len(self)
    }
}

impl AltLogicalData for Box<[bool]> {
    fn elt(&self, i: usize) -> Logical {
        if self[i] { Logical::True } else { Logical::False }
    }

    fn no_na(&self) -> Option<bool> {
        Some(true) // bool can't be NA
    }

    fn sum(&self, _na_rm: bool) -> Option<i64> {
        Some(self.iter().filter(|&&x| x).count() as i64)
    }
}

impl AltrepLen for Box<[String]> {
    fn len(&self) -> usize {
        <[String]>::len(self)
    }
}

impl AltStringData for Box<[String]> {
    fn elt(&self, i: usize) -> Option<&str> {
        Some(self[i].as_str())
    }

    fn no_na(&self) -> Option<bool> {
        Some(true) // String can't be NA
    }
}

// =============================================================================
// Built-in implementations for Range types
// =============================================================================

use std::ops::Range;

impl AltrepLen for Range<i32> {
    fn len(&self) -> usize {
        if self.end > self.start {
            (self.end - self.start) as usize
        } else {
            0
        }
    }
}

impl AltIntegerData for Range<i32> {
    fn elt(&self, i: usize) -> i32 {
        self.start + i as i32
    }

    fn is_sorted(&self) -> Option<Sortedness> {
        Some(Sortedness::StrictlyIncreasing)
    }

    fn no_na(&self) -> Option<bool> {
        Some(true)
    }

    fn sum(&self, _na_rm: bool) -> Option<i64> {
        let n = AltrepLen::len(self) as i64;
        if n == 0 {
            return Some(0);
        }
        // Sum of arithmetic sequence: n/2 * (first + last)
        let first = self.start as i64;
        let last = (self.end - 1) as i64;
        Some(n * (first + last) / 2)
    }

    fn min(&self, _na_rm: bool) -> Option<i32> {
        if AltrepLen::len(self) > 0 {
            Some(self.start)
        } else {
            None
        }
    }

    fn max(&self, _na_rm: bool) -> Option<i32> {
        if AltrepLen::len(self) > 0 {
            Some(self.end - 1)
        } else {
            None
        }
    }
}

impl AltrepLen for Range<i64> {
    fn len(&self) -> usize {
        if self.end > self.start {
            (self.end - self.start) as usize
        } else {
            0
        }
    }
}

impl AltIntegerData for Range<i64> {
    fn elt(&self, i: usize) -> i32 {
        (self.start + i as i64) as i32
    }

    fn is_sorted(&self) -> Option<Sortedness> {
        Some(Sortedness::StrictlyIncreasing)
    }

    fn no_na(&self) -> Option<bool> {
        Some(true)
    }

    fn sum(&self, _na_rm: bool) -> Option<i64> {
        let n = AltrepLen::len(self) as i64;
        if n == 0 {
            return Some(0);
        }
        let first = self.start;
        let last = self.end - 1;
        Some(n * (first + last) / 2)
    }

    fn min(&self, _na_rm: bool) -> Option<i32> {
        if AltrepLen::len(self) > 0 {
            Some(self.start as i32)
        } else {
            None
        }
    }

    fn max(&self, _na_rm: bool) -> Option<i32> {
        if AltrepLen::len(self) > 0 {
            Some((self.end - 1) as i32)
        } else {
            None
        }
    }
}

impl AltrepLen for Range<f64> {
    fn len(&self) -> usize {
        // For f64 ranges, assume step of 1.0
        if self.end > self.start {
            (self.end - self.start).ceil() as usize
        } else {
            0
        }
    }
}

impl AltRealData for Range<f64> {
    fn elt(&self, i: usize) -> f64 {
        self.start + i as f64
    }

    fn is_sorted(&self) -> Option<Sortedness> {
        Some(Sortedness::StrictlyIncreasing)
    }

    fn no_na(&self) -> Option<bool> {
        Some(true)
    }

    fn sum(&self, _na_rm: bool) -> Option<f64> {
        let n = AltrepLen::len(self) as f64;
        if n == 0.0 {
            return Some(0.0);
        }
        let first = self.start;
        let last = self.start + (n - 1.0);
        Some(n * (first + last) / 2.0)
    }

    fn min(&self, _na_rm: bool) -> Option<f64> {
        if AltrepLen::len(self) > 0 {
            Some(self.start)
        } else {
            None
        }
    }

    fn max(&self, _na_rm: bool) -> Option<f64> {
        if AltrepLen::len(self) > 0 {
            Some(self.start + (AltrepLen::len(self) - 1) as f64)
        } else {
            None
        }
    }
}

// =============================================================================
// Built-in implementations for slices (read-only)
// =============================================================================

impl AltrepLen for &[i32] {
    fn len(&self) -> usize {
        <[i32]>::len(self)
    }
}

impl AltIntegerData for &[i32] {
    fn elt(&self, i: usize) -> i32 {
        self[i]
    }

    fn as_slice(&self) -> Option<&[i32]> {
        Some(self)
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [i32]) -> usize {
        let end = (start + len).min(<[i32]>::len(self));
        let actual_len = end.saturating_sub(start);
        if actual_len > 0 {
            buf[..actual_len].copy_from_slice(&self[start..end]);
        }
        actual_len
    }

    fn no_na(&self) -> Option<bool> {
        // i32 slices have NA as i32::MIN
        Some(!self.contains(&i32::MIN))
    }

    fn sum(&self, _na_rm: bool) -> Option<i64> {
        // Check for NA (i32::MIN)
        if self.contains(&i32::MIN) {
            if _na_rm {
                Some(self.iter().filter(|&&x| x != i32::MIN).map(|&x| x as i64).sum())
            } else {
                None // Return NA
            }
        } else {
            Some(self.iter().map(|&x| x as i64).sum())
        }
    }

    fn min(&self, _na_rm: bool) -> Option<i32> {
        if self.is_empty() {
            return None;
        }
        if _na_rm {
            self.iter().filter(|&&x| x != i32::MIN).copied().min()
        } else if self.contains(&i32::MIN) {
            None // NA present
        } else {
            self.iter().copied().min()
        }
    }

    fn max(&self, _na_rm: bool) -> Option<i32> {
        if self.is_empty() {
            return None;
        }
        if _na_rm {
            self.iter().filter(|&&x| x != i32::MIN).copied().max()
        } else if self.contains(&i32::MIN) {
            None // NA present
        } else {
            self.iter().copied().max()
        }
    }
}

impl AltrepLen for &[f64] {
    fn len(&self) -> usize {
        <[f64]>::len(self)
    }
}

impl AltRealData for &[f64] {
    fn elt(&self, i: usize) -> f64 {
        self[i]
    }

    fn as_slice(&self) -> Option<&[f64]> {
        Some(self)
    }

    fn no_na(&self) -> Option<bool> {
        Some(!self.iter().any(|x| x.is_nan()))
    }

    fn sum(&self, na_rm: bool) -> Option<f64> {
        if na_rm {
            Some(self.iter().filter(|x| !x.is_nan()).sum())
        } else if self.iter().any(|x| x.is_nan()) {
            None // Return NA
        } else {
            Some(self.iter().sum())
        }
    }

    fn min(&self, na_rm: bool) -> Option<f64> {
        if self.is_empty() {
            return None;
        }
        if na_rm {
            self.iter()
                .filter(|x| !x.is_nan())
                .copied()
                .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        } else if self.iter().any(|x| x.is_nan()) {
            None
        } else {
            self.iter()
                .copied()
                .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        }
    }

    fn max(&self, na_rm: bool) -> Option<f64> {
        if self.is_empty() {
            return None;
        }
        if na_rm {
            self.iter()
                .filter(|x| !x.is_nan())
                .copied()
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        } else if self.iter().any(|x| x.is_nan()) {
            None
        } else {
            self.iter()
                .copied()
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        }
    }
}

impl AltrepLen for &[u8] {
    fn len(&self) -> usize {
        <[u8]>::len(self)
    }
}

impl AltRawData for &[u8] {
    fn elt(&self, i: usize) -> u8 {
        self[i]
    }

    fn as_slice(&self) -> Option<&[u8]> {
        Some(self)
    }
}

impl AltrepLen for &[bool] {
    fn len(&self) -> usize {
        <[bool]>::len(self)
    }
}

impl AltLogicalData for &[bool] {
    fn elt(&self, i: usize) -> Logical {
        Logical::from_bool(self[i])
    }

    fn no_na(&self) -> Option<bool> {
        Some(true) // bool can't be NA
    }

    fn sum(&self, _na_rm: bool) -> Option<i64> {
        Some(self.iter().filter(|&&x| x).count() as i64)
    }
}

impl AltrepLen for &[String] {
    fn len(&self) -> usize {
        <[String]>::len(self)
    }
}

impl AltStringData for &[String] {
    fn elt(&self, i: usize) -> Option<&str> {
        Some(self[i].as_str())
    }
}

impl AltrepLen for &[&str] {
    fn len(&self) -> usize {
        <[&str]>::len(self)
    }
}

impl AltStringData for &[&str] {
    fn elt(&self, i: usize) -> Option<&str> {
        Some(self[i])
    }
}

// =============================================================================
// NOTE on &'static [T] (static slices)
// =============================================================================
//
// `&'static [T]` is Sized (fat pointer: ptr + len) and satisfies 'static,
// so it can be used DIRECTLY with ALTREP via ExternalPtr.
//
// The data trait implementations above for `&[T]` already cover `&'static [T]`
// since `&'static [T]` is a subtype of `&[T]`. The ALTREP trait implementations
// (Altrep, AltVec, AltInteger, etc.) are provided separately in altrep_impl.rs.
//
// Use cases:
// - Const arrays: `static DATA: [i32; 5] = [1, 2, 3, 4, 5]; create_altrep(&DATA[..])`
// - Leaked data: `let s: &'static [i32] = Box::leak(vec.into_boxed_slice());`
// - Memory-mapped files with 'static lifetime

// =============================================================================
// Built-in implementations for arrays (owned, fixed-size)
// =============================================================================

impl<const N: usize> AltrepLen for [i32; N] {
    fn len(&self) -> usize {
        N
    }
}

impl<const N: usize> AltIntegerData for [i32; N] {
    fn elt(&self, i: usize) -> i32 {
        self[i]
    }

    fn as_slice(&self) -> Option<&[i32]> {
        Some(self.as_slice())
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [i32]) -> usize {
        let end = (start + len).min(N);
        let actual_len = end.saturating_sub(start);
        if actual_len > 0 {
            buf[..actual_len].copy_from_slice(&self[start..end]);
        }
        actual_len
    }

    fn no_na(&self) -> Option<bool> {
        Some(!self.contains(&i32::MIN))
    }
}

impl<const N: usize> AltrepLen for [f64; N] {
    fn len(&self) -> usize {
        N
    }
}

impl<const N: usize> AltRealData for [f64; N] {
    fn elt(&self, i: usize) -> f64 {
        self[i]
    }

    fn as_slice(&self) -> Option<&[f64]> {
        Some(self.as_slice())
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [f64]) -> usize {
        let end = (start + len).min(N);
        let actual_len = end.saturating_sub(start);
        if actual_len > 0 {
            buf[..actual_len].copy_from_slice(&self[start..end]);
        }
        actual_len
    }

    fn no_na(&self) -> Option<bool> {
        Some(!self.iter().any(|x| x.is_nan()))
    }
}

impl<const N: usize> AltrepLen for [bool; N] {
    fn len(&self) -> usize {
        N
    }
}

impl<const N: usize> AltLogicalData for [bool; N] {
    fn elt(&self, i: usize) -> Logical {
        Logical::from_bool(self[i])
    }

    fn no_na(&self) -> Option<bool> {
        Some(true) // bool arrays can't have NA
    }
}

impl<const N: usize> AltrepLen for [u8; N] {
    fn len(&self) -> usize {
        N
    }
}

impl<const N: usize> AltRawData for [u8; N] {
    fn elt(&self, i: usize) -> u8 {
        self[i]
    }

    fn as_slice(&self) -> Option<&[u8]> {
        Some(self.as_slice())
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [u8]) -> usize {
        let end = (start + len).min(N);
        let actual_len = end.saturating_sub(start);
        if actual_len > 0 {
            buf[..actual_len].copy_from_slice(&self[start..end]);
        }
        actual_len
    }
}

impl<const N: usize> AltrepLen for [String; N] {
    fn len(&self) -> usize {
        N
    }
}

impl<const N: usize> AltStringData for [String; N] {
    fn elt(&self, i: usize) -> Option<&str> {
        Some(self[i].as_str())
    }
}

// =============================================================================
// Unit tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Logical enum tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_logical_to_r_int() {
        assert_eq!(Logical::False.to_r_int(), 0);
        assert_eq!(Logical::True.to_r_int(), 1);
        assert_eq!(Logical::Na.to_r_int(), i32::MIN);
    }

    #[test]
    fn test_logical_from_r_int() {
        assert_eq!(Logical::from_r_int(0), Logical::False);
        assert_eq!(Logical::from_r_int(1), Logical::True);
        assert_eq!(Logical::from_r_int(42), Logical::True); // Non-zero is TRUE
        assert_eq!(Logical::from_r_int(-1), Logical::True);
        assert_eq!(Logical::from_r_int(i32::MIN), Logical::Na);
    }

    #[test]
    fn test_logical_from_bool() {
        assert_eq!(Logical::from_bool(false), Logical::False);
        assert_eq!(Logical::from_bool(true), Logical::True);
    }

    // -------------------------------------------------------------------------
    // Sortedness enum tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_sortedness_to_r_int() {
        assert_eq!(Sortedness::Unknown.to_r_int(), i32::MIN);
        assert_eq!(Sortedness::None.to_r_int(), 0);
        assert_eq!(Sortedness::Increasing.to_r_int(), 1);
        assert_eq!(Sortedness::Decreasing.to_r_int(), -1);
        assert_eq!(Sortedness::StrictlyIncreasing.to_r_int(), 2);
        assert_eq!(Sortedness::StrictlyDecreasing.to_r_int(), -2);
    }

    #[test]
    fn test_sortedness_from_r_int() {
        assert_eq!(Sortedness::from_r_int(i32::MIN), Sortedness::Unknown);
        assert_eq!(Sortedness::from_r_int(0), Sortedness::None);
        assert_eq!(Sortedness::from_r_int(1), Sortedness::Increasing);
        assert_eq!(Sortedness::from_r_int(-1), Sortedness::Decreasing);
        assert_eq!(Sortedness::from_r_int(2), Sortedness::StrictlyIncreasing);
        assert_eq!(Sortedness::from_r_int(-2), Sortedness::StrictlyDecreasing);
        // Invalid values map to Unknown
        assert_eq!(Sortedness::from_r_int(99), Sortedness::Unknown);
    }

    // -------------------------------------------------------------------------
    // Vec<i32> AltIntegerData tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_vec_i32_len() {
        let v: Vec<i32> = vec![1, 2, 3, 4, 5];
        assert_eq!(AltrepLen::len(&v), 5);
        assert!(!AltrepLen::is_empty(&v));

        let empty: Vec<i32> = vec![];
        assert_eq!(AltrepLen::len(&empty), 0);
        assert!(AltrepLen::is_empty(&empty));
    }

    #[test]
    fn test_vec_i32_elt() {
        let v = vec![10, 20, 30];
        assert_eq!(AltIntegerData::elt(&v, 0), 10);
        assert_eq!(AltIntegerData::elt(&v, 1), 20);
        assert_eq!(AltIntegerData::elt(&v, 2), 30);
    }

    #[test]
    fn test_vec_i32_as_slice() {
        let v = vec![1, 2, 3];
        assert_eq!(AltIntegerData::as_slice(&v), Some(&[1, 2, 3][..]));
    }

    #[test]
    fn test_vec_i32_get_region() {
        let v = vec![10, 20, 30, 40, 50];
        let mut buf = [0i32; 3];

        // Normal region
        let n = AltIntegerData::get_region(&v, 1, 3, &mut buf);
        assert_eq!(n, 3);
        assert_eq!(buf, [20, 30, 40]);

        // Region at end (partial)
        let n = AltIntegerData::get_region(&v, 3, 5, &mut buf);
        assert_eq!(n, 2);
        assert_eq!(buf[..2], [40, 50]);

        // Start beyond length
        let n = AltIntegerData::get_region(&v, 10, 3, &mut buf);
        assert_eq!(n, 0);
    }

    #[test]
    fn test_vec_i32_no_na() {
        let v = vec![1, 2, 3];
        assert_eq!(AltIntegerData::no_na(&v), Some(true));

        let v_with_na = vec![1, i32::MIN, 3]; // i32::MIN is NA
        assert_eq!(AltIntegerData::no_na(&v_with_na), Some(false));
    }

    #[test]
    fn test_vec_i32_sum() {
        let v = vec![1, 2, 3, 4, 5];
        assert_eq!(AltIntegerData::sum(&v, false), Some(15));
        assert_eq!(AltIntegerData::sum(&v, true), Some(15));

        // With NA
        let v_na = vec![1, 2, i32::MIN, 4, 5];
        assert_eq!(AltIntegerData::sum(&v_na, false), None); // NA propagates
        assert_eq!(AltIntegerData::sum(&v_na, true), Some(12)); // na.rm=TRUE
    }

    #[test]
    fn test_vec_i32_min_max() {
        let v = vec![5, 2, 8, 1, 9];
        assert_eq!(AltIntegerData::min(&v, false), Some(1));
        assert_eq!(AltIntegerData::max(&v, false), Some(9));

        // With NA
        let v_na = vec![5, 2, i32::MIN, 1, 9];
        assert_eq!(AltIntegerData::min(&v_na, false), None);
        assert_eq!(AltIntegerData::max(&v_na, false), None);
        assert_eq!(AltIntegerData::min(&v_na, true), Some(1));
        assert_eq!(AltIntegerData::max(&v_na, true), Some(9));
    }

    // -------------------------------------------------------------------------
    // Vec<f64> AltRealData tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_vec_f64_sum() {
        let v = vec![1.0, 2.0, 3.0];
        assert_eq!(AltRealData::sum(&v, false), Some(6.0));

        let v_nan = vec![1.0, f64::NAN, 3.0];
        assert!(AltRealData::sum(&v_nan, false).unwrap().is_nan());
        assert_eq!(AltRealData::sum(&v_nan, true), Some(4.0));
    }

    #[test]
    fn test_vec_f64_min_max() {
        let v = vec![3.0, 1.0, 4.0, 1.5];
        assert_eq!(AltRealData::min(&v, false), Some(1.0));
        assert_eq!(AltRealData::max(&v, false), Some(4.0));
    }

    // -------------------------------------------------------------------------
    // Box<[T]> tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_box_slice_i32() {
        let b: Box<[i32]> = vec![1, 2, 3, 4, 5].into_boxed_slice();
        assert_eq!(AltrepLen::len(&b), 5);
        assert_eq!(AltIntegerData::elt(&b, 2), 3);
        assert_eq!(AltIntegerData::sum(&b, false), Some(15));
        assert_eq!(AltIntegerData::min(&b, false), Some(1));
        assert_eq!(AltIntegerData::max(&b, false), Some(5));
    }

    #[test]
    fn test_box_slice_f64() {
        let b: Box<[f64]> = vec![1.0, 2.0, 3.0].into_boxed_slice();
        assert_eq!(AltrepLen::len(&b), 3);
        assert_eq!(AltRealData::elt(&b, 1), 2.0);
        assert_eq!(AltRealData::sum(&b, false), Some(6.0));
    }

    // -------------------------------------------------------------------------
    // Range<i32> tests
    // -------------------------------------------------------------------------

    #[test]
    #[allow(clippy::reversed_empty_ranges)] // Intentionally testing empty range handling
    fn test_range_i32_len() {
        let r = 1..10;
        assert_eq!(AltrepLen::len(&r), 9);

        let empty = 10..5;
        assert_eq!(AltrepLen::len(&empty), 0);
    }

    #[test]
    fn test_range_i32_elt() {
        let r = 5..10;
        assert_eq!(AltIntegerData::elt(&r, 0), 5);
        assert_eq!(AltIntegerData::elt(&r, 4), 9);
    }

    #[test]
    fn test_range_i32_sum() {
        // Sum of 1..11 (1 to 10) = 55
        let r = 1..11;
        assert_eq!(AltIntegerData::sum(&r, false), Some(55));

        // Sum of 1..101 (1 to 100) = 5050
        let r = 1..101;
        assert_eq!(AltIntegerData::sum(&r, false), Some(5050));
    }

    #[test]
    fn test_range_i32_min_max() {
        let r = 5..15;
        assert_eq!(AltIntegerData::min(&r, false), Some(5));
        assert_eq!(AltIntegerData::max(&r, false), Some(14)); // end is exclusive
    }

    #[test]
    fn test_range_i32_is_sorted() {
        let r = 1..10;
        assert_eq!(AltIntegerData::is_sorted(&r), Some(Sortedness::StrictlyIncreasing));
    }

    // -------------------------------------------------------------------------
    // Static slice tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_static_slice_i32() {
        static DATA: [i32; 5] = [10, 20, 30, 40, 50];
        let s: &[i32] = &DATA;

        assert_eq!(AltrepLen::len(&s), 5);
        assert_eq!(AltIntegerData::elt(&s, 0), 10);
        assert_eq!(AltIntegerData::elt(&s, 4), 50);
        assert_eq!(AltIntegerData::sum(&s, false), Some(150));
        assert_eq!(AltIntegerData::min(&s, false), Some(10));
        assert_eq!(AltIntegerData::max(&s, false), Some(50));
    }

    #[test]
    fn test_static_slice_with_na() {
        let s: &[i32] = &[1, 2, i32::MIN, 4];
        assert_eq!(AltIntegerData::no_na(&s), Some(false));
        assert_eq!(AltIntegerData::sum(&s, false), None); // NA propagates
        assert_eq!(AltIntegerData::sum(&s, true), Some(7)); // na.rm=TRUE
    }

    #[test]
    fn test_static_slice_f64() {
        static DATA: [f64; 4] = [1.5, 2.5, 3.5, 4.5];
        let s: &[f64] = &DATA;

        assert_eq!(AltrepLen::len(&s), 4);
        assert_eq!(AltRealData::sum(&s, false), Some(12.0));
        assert_eq!(AltRealData::min(&s, false), Some(1.5));
        assert_eq!(AltRealData::max(&s, false), Some(4.5));
    }

    // -------------------------------------------------------------------------
    // Array tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_array_i32() {
        let arr: [i32; 3] = [100, 200, 300];
        assert_eq!(AltrepLen::len(&arr), 3);
        assert_eq!(AltIntegerData::elt(&arr, 1), 200);
        assert_eq!(AltIntegerData::as_slice(&arr), Some(&[100, 200, 300][..]));
    }

    #[test]
    fn test_array_f64() {
        let arr: [f64; 2] = [1.1, 2.2];
        assert_eq!(AltrepLen::len(&arr), 2);
        assert_eq!(AltRealData::elt(&arr, 0), 1.1);
    }

    // -------------------------------------------------------------------------
    // Vec<bool> AltLogicalData tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_vec_bool_logical() {
        let v = vec![true, false, true, true];
        assert_eq!(AltrepLen::len(&v), 4);
        assert_eq!(AltLogicalData::elt(&v, 0), Logical::True);
        assert_eq!(AltLogicalData::elt(&v, 1), Logical::False);
        assert_eq!(AltLogicalData::no_na(&v), Some(true));
        assert_eq!(AltLogicalData::sum(&v, false), Some(3)); // Count of TRUE
    }

    // -------------------------------------------------------------------------
    // Vec<String> AltStringData tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_vec_string() {
        let v = vec!["hello".to_string(), "world".to_string()];
        assert_eq!(AltrepLen::len(&v), 2);
        assert_eq!(AltStringData::elt(&v, 0), Some("hello"));
        assert_eq!(AltStringData::elt(&v, 1), Some("world"));
        assert_eq!(AltStringData::no_na(&v), Some(true));
    }

    #[test]
    fn test_vec_option_string() {
        let v: Vec<Option<String>> = vec![
            Some("a".to_string()),
            None,
            Some("b".to_string()),
        ];
        assert_eq!(AltrepLen::len(&v), 3);
        assert_eq!(AltStringData::elt(&v, 0), Some("a"));
        assert_eq!(AltStringData::elt(&v, 1), None); // NA
        assert_eq!(AltStringData::elt(&v, 2), Some("b"));
        assert_eq!(AltStringData::no_na(&v), Some(false)); // Has NA
    }

    // -------------------------------------------------------------------------
    // Vec<u8> AltRawData tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_vec_u8() {
        let v: Vec<u8> = vec![0x01, 0x02, 0xFF];
        assert_eq!(AltrepLen::len(&v), 3);
        assert_eq!(AltRawData::elt(&v, 0), 0x01);
        assert_eq!(AltRawData::elt(&v, 2), 0xFF);
        assert_eq!(AltRawData::as_slice(&v), Some(&[0x01, 0x02, 0xFF][..]));
    }

    // -------------------------------------------------------------------------
    // Edge cases
    // -------------------------------------------------------------------------

    #[test]
    fn test_empty_vec() {
        let v: Vec<i32> = vec![];
        assert_eq!(AltrepLen::len(&v), 0);
        assert!(AltrepLen::is_empty(&v));
        assert_eq!(AltIntegerData::sum(&v, false), Some(0));
        assert_eq!(AltIntegerData::min(&v, false), None);
        assert_eq!(AltIntegerData::max(&v, false), None);
    }

    #[test]
    fn test_single_element() {
        let v = vec![42];
        assert_eq!(AltIntegerData::sum(&v, false), Some(42));
        assert_eq!(AltIntegerData::min(&v, false), Some(42));
        assert_eq!(AltIntegerData::max(&v, false), Some(42));
    }

    #[test]
    fn test_large_sum_overflow() {
        // Sum that exceeds i32 range but fits in i64
        let v: Vec<i32> = vec![i32::MAX, i32::MAX];
        let sum = AltIntegerData::sum(&v, false).unwrap();
        assert_eq!(sum, 2 * i32::MAX as i64);
    }
}
