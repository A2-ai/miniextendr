//! N-dimensional R arrays with const generic dimension count.
//!
//! This module provides [`RArray<T, NDIM>`], a wrapper around R arrays that
//! tracks the number of dimensions at compile time.
//!
//! # Type Aliases
//!
//! | Alias | Type | R Equivalent |
//! |-------|------|--------------|
//! | [`RVector<T>`] | `RArray<T, 1>` | `vector` (with dim) |
//! | [`RMatrix<T>`] | `RArray<T, 2>` | `matrix` |
//! | [`RArray3D<T>`] | `RArray<T, 3>` | `array(..., dim=c(a,b,c))` |
//!
//! # Memory Layout
//!
//! R arrays are stored in **column-major** (Fortran) order. For a 2×3 matrix:
//!
//! ```text
//! Logical layout:     Memory layout:
//! [0,0] [0,1] [0,2]   [0,0] [1,0] [0,1] [1,1] [0,2] [1,2]
//! [1,0] [1,1] [1,2]
//! ```
//!
//! The [`get`][RArray::get] method handles index translation automatically.
//!
//! # Thread Safety
//!
//! **`RArray` is `!Send` and `!Sync`** - it cannot be transferred to or accessed
//! from other threads. This is because the underlying R APIs (`DATAPTR_RO`, etc.)
//! must be called on the R main thread.
//!
//! For functions that use `RArray`/`RMatrix` parameters, you must use
//! `#[miniextendr(unsafe(main_thread))]` to ensure execution on the main thread.
//!
//! For worker-thread usability, use [`to_vec()`][RArray::to_vec] to copy data
//! on the main thread, then pass the owned `Vec` to worker threads.
//!
//! # Performance
//!
//! For best performance, prefer slice-based and column-based access over per-element
//! indexing:
//!
//! | Method | Speed | Use Case |
//! |--------|-------|----------|
//! | [`as_slice()`][RArray::as_slice] | Fastest | Full-buffer iteration, SIMD |
//! | [`column()`][RMatrix::column] | Fast | Per-column operations (matrices) |
//! | [`column_mut()`][RMatrix::column_mut] | Fast | Per-column mutation |
//! | [`get()`][RArray::get] / [`get_rc()`][RMatrix::get_rc] | Slower | Single-element access |
//!
//! **Why?** Per-element methods like `get()` perform index translation and bounds
//! checks on every call. For tight loops, this overhead dominates.
//!
//! ```ignore
//! // Slow: per-element access
//! for row in 0..nrow {
//!     for col in 0..ncol {
//!         let val = unsafe { matrix.get_rc(row, col) };
//!     }
//! }
//!
//! // Fast: slice-based iteration
//! for val in unsafe { matrix.as_slice() } {
//!     // ...
//! }
//!
//! // Fast: column-wise iteration (columns are contiguous in R)
//! for col in 0..ncol {
//!     for val in unsafe { matrix.column(col) } {
//!         // ...
//!     }
//! }
//! ```
//!
//! # Example
//!
//! ```ignore
//! use miniextendr_api::rarray::{RMatrix, RArray};
//!
//! // Must run on main thread due to RMatrix parameter
//! #[miniextendr(unsafe(main_thread))]
//! fn matrix_sum(m: RMatrix<f64>) -> f64 {
//!     unsafe { m.as_slice().iter().sum() }
//! }
//! ```

use crate::ffi::{self, RNativeType, SEXP, SEXPTYPE, SexpExt};
use crate::from_r::{SexpError, SexpLengthError, SexpTypeError, TryFromSexp};
use crate::into_r::IntoR;
use core::marker::PhantomData;

// =============================================================================
// Type aliases
// =============================================================================

/// A 1-dimensional R vector with explicit dim attribute.
pub type RVector<T> = RArray<T, 1>;

/// A 2-dimensional R matrix.
pub type RMatrix<T> = RArray<T, 2>;

/// A 3-dimensional R array.
pub type RArray3D<T> = RArray<T, 3>;

// =============================================================================
// RArray
// =============================================================================

/// An N-dimensional R array.
///
/// This type wraps an R array SEXP. The dimension count `NDIM` is tracked
/// at compile time, but dimension sizes are read from the R object.
///
/// # Type Parameters
///
/// - `T`: The element type, must implement [`RNativeType`]
/// - `NDIM`: The number of dimensions (compile-time constant)
///
/// # Thread Safety
///
/// This type is `!Send` and `!Sync` because its methods require access to
/// R APIs that must run on the R main thread.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct RArray<T, const NDIM: usize> {
    sexp: SEXP,
    // PhantomData<*const T> keeps T in the type AND makes this !Send + !Sync
    _marker: PhantomData<*const T>,
}

// =============================================================================
// Basic methods (no T bounds - available for all RArray types)
// =============================================================================

impl<T, const NDIM: usize> RArray<T, NDIM> {
    /// Create an RArray from a SEXP without validation.
    ///
    /// # Safety
    ///
    /// - The SEXP must be protected from GC
    /// - The SEXP must have the correct type for `T`
    /// - The SEXP must have exactly `NDIM` dimensions
    #[inline]
    pub const unsafe fn from_sexp_unchecked(sexp: SEXP) -> Self {
        Self {
            sexp,
            _marker: PhantomData,
        }
    }

    /// Get the underlying SEXP.
    #[inline]
    pub const fn as_sexp(&self) -> SEXP {
        self.sexp
    }

    /// Consume and return the underlying SEXP.
    #[inline]
    pub fn into_inner(self) -> SEXP {
        self.sexp
    }

    /// Get the dimensions as an array.
    ///
    /// # Safety
    ///
    /// The SEXP must be valid.
    #[inline]
    pub unsafe fn dims(&self) -> [usize; NDIM] {
        unsafe { get_dims::<NDIM>(self.sexp) }
    }

    /// Get a specific dimension size.
    ///
    /// # Safety
    ///
    /// The SEXP must be valid.
    ///
    /// # Panics
    ///
    /// Panics if `dim >= NDIM`.
    #[inline]
    pub unsafe fn dim(&self, dim: usize) -> usize {
        assert!(dim < NDIM, "dimension index out of bounds");
        unsafe { self.dims()[dim] }
    }

    /// Get the total number of elements.
    #[inline]
    pub fn len(&self) -> usize {
        self.sexp.len()
    }

    /// Check if the array is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Convert N-dimensional indices to linear index (column-major).
    ///
    /// # Safety
    ///
    /// The SEXP must be valid (needed to read dims).
    ///
    /// # Panics
    ///
    /// Panics if any index is out of bounds.
    #[inline]
    pub unsafe fn linear_index(&self, indices: [usize; NDIM]) -> usize {
        let dims = unsafe { self.dims() };
        let mut linear = 0;
        let mut stride = 1;
        for i in 0..NDIM {
            assert!(
                indices[i] < dims[i],
                "index {} out of bounds for dimension {} (size {})",
                indices[i],
                i,
                dims[i]
            );
            linear += indices[i] * stride;
            stride *= dims[i];
        }
        linear
    }
}

// =============================================================================
// Native type methods (T: RNativeType - slice access, mutation, etc.)
// =============================================================================

impl<T: RNativeType, const NDIM: usize> RArray<T, NDIM> {
    /// Create an RArray from a SEXP, validating type and dimensions.
    ///
    /// # Safety
    ///
    /// The SEXP must be protected from GC for the lifetime of the returned RArray.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The SEXP type doesn't match `T::SEXP_TYPE`
    /// - The dim attribute has wrong number of dimensions
    #[inline]
    pub unsafe fn from_sexp(sexp: SEXP) -> Result<Self, SexpError> {
        // Type check
        let actual = sexp.type_of();
        if actual != T::SEXP_TYPE {
            return Err(SexpTypeError {
                expected: T::SEXP_TYPE,
                actual,
            }
            .into());
        }

        // Validate dimensions count
        let ndim = get_ndim(sexp);
        if ndim != NDIM {
            return Err(SexpLengthError {
                expected: NDIM,
                actual: ndim,
            }
            .into());
        }

        Ok(Self {
            sexp,
            _marker: PhantomData,
        })
    }

    /// Get the data as a slice (column-major order).
    ///
    /// # Safety
    ///
    /// The SEXP must be protected and valid.
    #[inline]
    pub unsafe fn as_slice(&self) -> &[T] {
        unsafe { self.sexp.as_slice() }
    }

    /// Get the data as a mutable slice (column-major order).
    ///
    /// # Safety
    ///
    /// - The SEXP must be protected and valid
    /// - No other references to the data may exist
    #[inline]
    pub unsafe fn as_slice_mut(&mut self) -> &mut [T] {
        unsafe {
            let ptr = T::dataptr_mut(self.sexp);
            std::slice::from_raw_parts_mut(ptr, self.len())
        }
    }

    /// Copy array data to an owned `Vec<T>`.
    ///
    /// This method copies the data, making it safe to use in worker threads
    /// or pass to parallel computation. The copy is performed on the current
    /// thread (which must be the R main thread).
    ///
    /// # Safety
    ///
    /// The SEXP must be protected and valid.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use miniextendr_api::rarray::RMatrix;
    ///
    /// #[miniextendr(unsafe(main_thread))]
    /// fn process_matrix(m: RMatrix<f64>) -> f64 {
    ///     // Copy data - Vec<f64> is Send and can be used in worker threads
    ///     let data: Vec<f64> = unsafe { m.to_vec() };
    ///     // Now data can be passed to parallel computation
    ///     data.iter().sum()
    /// }
    /// ```
    #[inline]
    pub unsafe fn to_vec(&self) -> Vec<T>
    where
        T: Copy,
    {
        unsafe { self.as_slice().to_vec() }
    }

    /// Get an element by N-dimensional indices.
    ///
    /// # Safety
    ///
    /// The SEXP must be protected and valid.
    ///
    /// # Panics
    ///
    /// Panics if any index is out of bounds.
    #[inline]
    pub unsafe fn get(&self, indices: [usize; NDIM]) -> T
    where
        T: Copy,
    {
        let idx = unsafe { self.linear_index(indices) };
        unsafe { *self.as_slice().get_unchecked(idx) }
    }

    /// Set an element by N-dimensional indices.
    ///
    /// # Safety
    ///
    /// - The SEXP must be protected and valid
    /// - No other references to the data may exist
    ///
    /// # Panics
    ///
    /// Panics if any index is out of bounds.
    #[inline]
    pub unsafe fn set(&mut self, indices: [usize; NDIM], value: T)
    where
        T: Copy,
    {
        let idx = unsafe { self.linear_index(indices) };
        unsafe {
            *self.as_slice_mut().get_unchecked_mut(idx) = value;
        }
    }
}

// =============================================================================
// Matrix-specific methods (NDIM = 2)
// =============================================================================

impl<T: RNativeType> RMatrix<T> {
    /// Get the number of rows.
    ///
    /// # Safety
    ///
    /// The SEXP must be valid.
    #[inline]
    pub unsafe fn nrow(&self) -> usize {
        unsafe { self.dim(0) }
    }

    /// Get the number of columns.
    ///
    /// # Safety
    ///
    /// The SEXP must be valid.
    #[inline]
    pub unsafe fn ncol(&self) -> usize {
        unsafe { self.dim(1) }
    }

    /// Get an element by row and column.
    ///
    /// # Safety
    ///
    /// The SEXP must be protected and valid.
    #[inline]
    pub unsafe fn get_rc(&self, row: usize, col: usize) -> T
    where
        T: Copy,
    {
        unsafe { self.get([row, col]) }
    }

    /// Set an element by row and column.
    ///
    /// # Safety
    ///
    /// - The SEXP must be protected and valid
    /// - No other references to the data may exist
    #[inline]
    pub unsafe fn set_rc(&mut self, row: usize, col: usize, value: T)
    where
        T: Copy,
    {
        unsafe { self.set([row, col], value) }
    }

    /// Get a column as a slice.
    ///
    /// # Safety
    ///
    /// The SEXP must be protected and valid.
    #[inline]
    pub unsafe fn column(&self, col: usize) -> &[T] {
        let nrow = unsafe { self.nrow() };
        let ncol = unsafe { self.ncol() };
        assert!(col < ncol, "column index out of bounds");
        let start = col * nrow;
        unsafe { &self.as_slice()[start..start + nrow] }
    }

    /// Get a mutable column as a slice.
    ///
    /// Columns are contiguous in R's column-major layout, so this returns
    /// a proper `&mut [T]` without any striding.
    ///
    /// # Safety
    ///
    /// The SEXP must be protected and valid.
    ///
    /// # Panics
    ///
    /// Panics if `col >= ncol`.
    #[inline]
    pub unsafe fn column_mut(&mut self, col: usize) -> &mut [T] {
        let nrow = unsafe { self.nrow() };
        let ncol = unsafe { self.ncol() };
        assert!(col < ncol, "column index out of bounds");
        let start = col * nrow;
        unsafe { &mut self.as_slice_mut()[start..start + nrow] }
    }
}

// =============================================================================
// Attribute access (equivalent to R's GET_*/SET_* macros)
// =============================================================================

impl<T: RNativeType, const NDIM: usize> RArray<T, NDIM> {
    // -------------------------------------------------------------------------
    // Attribute getters
    // -------------------------------------------------------------------------

    /// Get an arbitrary attribute by symbol (unchecked internal helper).
    ///
    /// # Safety
    ///
    /// - The SEXP must be valid.
    /// - `what` must be a valid symbol SEXP.
    #[inline]
    unsafe fn get_attr_impl_unchecked(&self, what: SEXP) -> Option<SEXP> {
        unsafe {
            let attr = ffi::Rf_getAttrib(self.sexp, what);
            if attr == ffi::R_NilValue {
                None
            } else {
                Some(attr)
            }
        }
    }

    /// Get the `names` attribute if present.
    ///
    /// Equivalent to R's `GET_NAMES(x)`.
    ///
    /// # Safety
    ///
    /// The SEXP must be valid.
    #[inline]
    pub unsafe fn get_names(&self) -> Option<SEXP> {
        // Safety: R_NamesSymbol is a known symbol
        unsafe { self.get_attr_impl_unchecked(ffi::R_NamesSymbol) }
    }

    /// Get the `class` attribute if present.
    ///
    /// Equivalent to R's `GET_CLASS(x)`.
    ///
    /// # Safety
    ///
    /// The SEXP must be valid.
    #[inline]
    pub unsafe fn get_class(&self) -> Option<SEXP> {
        // Safety: R_ClassSymbol is a known symbol
        unsafe { self.get_attr_impl_unchecked(ffi::R_ClassSymbol) }
    }

    /// Get the `dimnames` attribute if present.
    ///
    /// Equivalent to R's `GET_DIMNAMES(x)`.
    ///
    /// # Safety
    ///
    /// The SEXP must be valid.
    #[inline]
    pub unsafe fn get_dimnames(&self) -> Option<SEXP> {
        // Safety: R_DimNamesSymbol is a known symbol
        unsafe { self.get_attr_impl_unchecked(ffi::R_DimNamesSymbol) }
    }

    /// Get row names from the `dimnames` attribute.
    ///
    /// Equivalent to R's `GET_ROWNAMES(x)` / `Rf_GetRowNames(x)`.
    ///
    /// # Safety
    ///
    /// The SEXP must be valid.
    #[inline]
    pub unsafe fn get_rownames(&self) -> Option<SEXP> {
        unsafe {
            let rownames = ffi::Rf_GetRowNames(self.sexp);
            if rownames == ffi::R_NilValue {
                None
            } else {
                Some(rownames)
            }
        }
    }

    /// Get column names from the `dimnames` attribute.
    ///
    /// Equivalent to R's `GET_COLNAMES(x)` / `Rf_GetColNames(x)`.
    ///
    /// # Safety
    ///
    /// The SEXP must be valid.
    #[inline]
    pub unsafe fn get_colnames(&self) -> Option<SEXP> {
        unsafe {
            let dimnames = ffi::Rf_getAttrib(self.sexp, ffi::R_DimNamesSymbol);
            if dimnames == ffi::R_NilValue {
                return None;
            }
            let colnames = ffi::Rf_GetColNames(dimnames);
            if colnames == ffi::R_NilValue {
                None
            } else {
                Some(colnames)
            }
        }
    }

    // -------------------------------------------------------------------------
    // Attribute setters
    // -------------------------------------------------------------------------

    /// Set an arbitrary attribute by symbol (unchecked internal helper).
    ///
    /// # Safety
    ///
    /// - The SEXP must be valid and not shared.
    /// - `what` must be a valid symbol SEXP.
    #[inline]
    #[allow(dead_code)]
    unsafe fn set_attr_impl_unchecked(&mut self, what: SEXP, value: SEXP) {
        unsafe { ffi::Rf_setAttrib(self.sexp, what, value) };
    }

    /// Set the `names` attribute.
    ///
    /// Equivalent to R's `SET_NAMES(x, n)`.
    ///
    /// # Safety
    ///
    /// The SEXP must be valid and not shared.
    #[inline]
    pub unsafe fn set_names(&mut self, names: SEXP) {
        unsafe { ffi::Rf_namesgets(self.sexp, names) };
    }

    /// Set the `class` attribute.
    ///
    /// Equivalent to R's `SET_CLASS(x, n)`.
    ///
    /// # Safety
    ///
    /// The SEXP must be valid and not shared.
    #[inline]
    pub unsafe fn set_class(&mut self, class: SEXP) {
        unsafe { ffi::Rf_setAttrib(self.sexp, ffi::R_ClassSymbol, class) };
    }

    /// Set the `dimnames` attribute.
    ///
    /// Equivalent to R's `SET_DIMNAMES(x, n)`.
    ///
    /// # Safety
    ///
    /// The SEXP must be valid and not shared.
    #[inline]
    pub unsafe fn set_dimnames(&mut self, dimnames: SEXP) {
        unsafe { ffi::Rf_setAttrib(self.sexp, ffi::R_DimNamesSymbol, dimnames) };
    }
}

// =============================================================================
// Construction helpers
// =============================================================================

impl<T: RNativeType, const NDIM: usize> RArray<T, NDIM> {
    /// Allocate a new R array with the given dimensions.
    ///
    /// The array is allocated. The closure receives a mutable slice to
    /// initialize the data.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread (or via routed FFI).
    /// The returned RArray holds an unprotected SEXP - caller must protect.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let matrix = unsafe {
    ///     RMatrix::<f64>::new([3, 4], |slice| {
    ///         for (i, v) in slice.iter_mut().enumerate() {
    ///             *v = i as f64;
    ///         }
    ///     })
    /// };
    /// ```
    pub unsafe fn new<F>(dims: [usize; NDIM], init: F) -> Self
    where
        F: FnOnce(&mut [T]),
    {
        let total_len: usize = dims.iter().product();

        // Allocate the vector
        let sexp = unsafe { ffi::Rf_allocVector(T::SEXP_TYPE, total_len as ffi::R_xlen_t) };

        // Set dimensions
        unsafe { set_dims::<NDIM>(sexp, &dims) };

        // Initialize data
        let ptr = unsafe { T::dataptr_mut(sexp) };
        let slice = unsafe { std::slice::from_raw_parts_mut(ptr, total_len) };
        init(slice);

        Self {
            sexp,
            _marker: PhantomData,
        }
    }

    /// Allocate a new R array filled with zeros.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread (or via routed FFI).
    /// The returned RArray holds an unprotected SEXP - caller must protect.
    pub unsafe fn zeros(dims: [usize; NDIM]) -> Self
    where
        T: Default + Copy,
    {
        unsafe {
            Self::new(dims, |slice| {
                slice.fill(T::default());
            })
        }
    }
}

// =============================================================================
// TryFromSexp implementation
// =============================================================================

impl<T: RNativeType, const NDIM: usize> TryFromSexp for RArray<T, NDIM> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        unsafe { Self::from_sexp(sexp) }
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        unsafe { Self::from_sexp(sexp) }
    }
}

// =============================================================================
// Direct coercion TryFromSexp implementations
// =============================================================================
//
// These implement TryFromSexp for RArray<T, NDIM> where T is not an R native type
// but can be coerced from one. The RArray wraps the source SEXP directly (zero-copy).
// Note: as_slice() is not available for coerced types - use to_vec_coerced() instead.

use crate::coerce::TryCoerce;
use crate::ffi::RLogical;

/// Helper to validate all elements can be coerced.
fn validate_coercion<S, T>(slice: &[S]) -> Result<(), SexpError>
where
    S: Copy + TryCoerce<T>,
    <S as TryCoerce<T>>::Error: std::fmt::Debug,
{
    for &val in slice {
        val.try_coerce()
            .map_err(|e| SexpError::InvalidValue(format!("{e:?}")))?;
    }
    Ok(())
}

/// Implement `TryFromSexp for RArray<$target, NDIM>` by reading R's native `$source` type.
///
/// The RArray wraps the source SEXP directly. Use `to_vec_coerced()` to get coerced data.
macro_rules! impl_rarray_try_from_sexp_coerce {
    ($source:ty => $target:ty) => {
        impl<const NDIM: usize> TryFromSexp for RArray<$target, NDIM> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                // Check source type
                let actual = sexp.type_of();
                if actual != <$source as RNativeType>::SEXP_TYPE {
                    return Err(SexpTypeError {
                        expected: <$source as RNativeType>::SEXP_TYPE,
                        actual,
                    }
                    .into());
                }

                // Validate dimensions count
                let ndim = get_ndim(sexp);
                if ndim != NDIM {
                    return Err(SexpLengthError {
                        expected: NDIM,
                        actual: ndim,
                    }
                    .into());
                }

                // Validate all elements can be coerced
                let slice: &[$source] = unsafe { sexp.as_slice() };
                validate_coercion::<$source, $target>(slice)?;

                Ok(Self {
                    sexp,
                    _marker: PhantomData,
                })
            }

            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                Self::try_from_sexp(sexp)
            }
        }

        impl<const NDIM: usize> RArray<$target, NDIM> {
            /// Copy array data to an owned `Vec`, coercing from the R native type.
            ///
            /// # Safety
            ///
            /// The SEXP must be protected and valid.
            ///
            /// # Panics
            ///
            /// Panics if any element fails to coerce (shouldn't happen if constructed via TryFromSexp).
            #[inline]
            pub unsafe fn to_vec_coerced(&self) -> Vec<$target> {
                let slice: &[$source] = unsafe { self.sexp.as_slice() };
                slice
                    .iter()
                    .copied()
                    .map(|v| {
                        <$source as TryCoerce<$target>>::try_coerce(v)
                            .expect("coercion should succeed")
                    })
                    .collect()
            }
        }
    };
}

// Integer coercions: R integer (i32) -> various Rust integer types
impl_rarray_try_from_sexp_coerce!(i32 => i8);
impl_rarray_try_from_sexp_coerce!(i32 => i16);
impl_rarray_try_from_sexp_coerce!(i32 => i64);
impl_rarray_try_from_sexp_coerce!(i32 => isize);
impl_rarray_try_from_sexp_coerce!(i32 => u16);
impl_rarray_try_from_sexp_coerce!(i32 => u32);
impl_rarray_try_from_sexp_coerce!(i32 => u64);
impl_rarray_try_from_sexp_coerce!(i32 => usize);

// Float coercions: R numeric (f64) -> f32
impl_rarray_try_from_sexp_coerce!(f64 => f32);

// Logical coercions: R logical (RLogical) -> bool
impl_rarray_try_from_sexp_coerce!(RLogical => bool);

// =============================================================================
// IntoR implementation
// =============================================================================

impl<T: RNativeType, const NDIM: usize> IntoR for RArray<T, NDIM> {
    fn into_sexp(self) -> SEXP {
        self.sexp
    }

    unsafe fn into_sexp_unchecked(self) -> SEXP {
        self.sexp
    }
}

// =============================================================================
// Helper functions
// =============================================================================

/// Get number of dimensions from SEXP.
fn get_ndim(sexp: SEXP) -> usize {
    unsafe {
        let dim_sexp = ffi::Rf_getAttrib(sexp, ffi::R_DimSymbol);
        if dim_sexp.type_of() != SEXPTYPE::INTSXP {
            // No dim attribute - treat as 1D
            1
        } else {
            dim_sexp.len()
        }
    }
}

/// Get dimensions from SEXP as array.
///
/// # Safety
///
/// Caller must ensure SEXP has NDIM dimensions.
unsafe fn get_dims<const NDIM: usize>(sexp: SEXP) -> [usize; NDIM] {
    let mut dims = [0usize; NDIM];

    unsafe {
        let dim_sexp = ffi::Rf_getAttrib(sexp, ffi::R_DimSymbol);

        if dim_sexp.type_of() != SEXPTYPE::INTSXP {
            // No dim attribute - treat as 1D with length
            if NDIM == 1 {
                dims[0] = sexp.len();
            }
        } else {
            let dim_slice: &[i32] = dim_sexp.as_slice();
            for (i, &d) in dim_slice.iter().take(NDIM).enumerate() {
                dims[i] = d as usize;
            }
        }
    }

    dims
}

/// Set dimensions on a SEXP.
///
/// # Safety
///
/// Must be called from R main thread.
unsafe fn set_dims<const NDIM: usize>(sexp: SEXP, dims: &[usize; NDIM]) {
    unsafe {
        let dim_sexp = ffi::Rf_allocVector(SEXPTYPE::INTSXP, NDIM as ffi::R_xlen_t);
        ffi::Rf_protect(dim_sexp);

        let dim_ptr = ffi::INTEGER(dim_sexp);
        for (i, &d) in dims.iter().enumerate() {
            *dim_ptr.add(i) = d as i32;
        }

        ffi::Rf_setAttrib(sexp, ffi::R_DimSymbol, dim_sexp);
        ffi::Rf_unprotect(1);
    }
}

// =============================================================================
// Debug implementation
// =============================================================================

impl<T: RNativeType, const NDIM: usize> std::fmt::Debug for RArray<T, NDIM> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RArray")
            .field("ndim", &NDIM)
            .field("len", &self.len())
            .field("sexp", &self.sexp)
            .finish()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matrix_is_array2() {
        fn assert_matrix<T: RNativeType>(_: RMatrix<T>) {}
        fn assert_array2<T: RNativeType>(_: RArray<T, 2>) {}

        // These should compile - RMatrix<T> == RArray<T, 2>
        let m: RMatrix<f64> = unsafe { RArray::from_sexp_unchecked(SEXP(std::ptr::null_mut())) };
        assert_matrix(m);
        assert_array2(m);
    }

    #[test]
    fn size_equals_sexp() {
        // RArray should be same size as SEXP (PhantomData is zero-sized)
        assert_eq!(
            std::mem::size_of::<RArray<f64, 2>>(),
            std::mem::size_of::<SEXP>()
        );
    }

    // Note: RArray is !Send and !Sync due to PhantomData<*const ()>.
    // This is verified by the compiler - attempting to send RArray across
    // threads will fail to compile.
}
