//! Integration with the Apache Arrow columnar format.
//!
//! This module provides zero-copy (where possible) conversions between R
//! vectors/data.frames and Arrow arrays/RecordBatch.
//!
//! # Zero-Copy Conversions (R → Arrow)
//!
//! | R Type | Arrow Type | Method |
//! |--------|-----------|--------|
//! | `numeric` (REALSXP) | `Float64Array` | Zero-copy via shared buffer |
//! | `integer` (INTSXP) | `Int32Array` | Zero-copy via shared buffer |
//! | `raw` (RAWSXP) | `UInt8Array` | Zero-copy via shared buffer |
//!
//! # Copy Conversions
//!
//! | R Type | Arrow Type | Notes |
//! |--------|-----------|-------|
//! | `logical` (LGLSXP) | `BooleanArray` | R uses i32 per element, Arrow uses bit-packed |
//! | `character` (STRSXP) | `StringArray` | Different string representations |
//! | `data.frame` | `RecordBatch` | Column-wise conversion |
//!
//! # NA Handling
//!
//! R's NA values (`NA_integer_`, `NA_real_`, `NA_character_`) are converted to
//! Arrow's null bitmap. For zero-copy primitives, the NA sentinel values remain
//! in the data buffer — Arrow ignores them because the null bitmap marks those
//! positions as null.
//!
//! # Features
//!
//! Enable with `features = ["arrow"]`:
//!
//! ```toml
//! [dependencies]
//! miniextendr-api = { version = "0.1", features = ["arrow"] }
//! ```

use std::sync::Arc;

pub use arrow_array::{
    self,
    types::{Float64Type, Int32Type, UInt8Type},
    Array, ArrayRef, BooleanArray, Float64Array, Int32Array, RecordBatch, StringArray, UInt8Array,
};
pub use arrow_buffer;
pub use arrow_schema::{self, DataType, Field, Schema};

use crate::ffi::{
    self, RNativeType, R_NaString, R_xlen_t, Rboolean, SEXP, SEXPTYPE, SexpExt,
    R_NamesSymbol, R_ClassSymbol, R_RowNamesSymbol,
};
use crate::from_r::{SexpError, SexpTypeError, TryFromSexp};
use crate::into_r::IntoR;

// region: RPreservedSexp — GC guard for Arrow Allocation trait

/// GC guard that keeps an R SEXP alive for as long as an Arrow Buffer exists.
///
/// Uses `R_PreserveObject`/`R_ReleaseObject` (mutex-protected in R 4.0+)
/// instead of the thread-local preserve list, because Arrow buffers may be
/// dropped on non-main threads (Arrow is `Send + Sync`).
struct RPreservedSexp(SEXP);

// SAFETY: R_PreserveObject/R_ReleaseObject are mutex-protected in R 4.0+.
// The SEXP data is immutable once preserved (we only read via DATAPTR_RO).
unsafe impl Send for RPreservedSexp {}
unsafe impl Sync for RPreservedSexp {}
impl std::panic::RefUnwindSafe for RPreservedSexp {}

// Allocation is automatically implemented via blanket impl:
//   impl<T: Send + Sync + RefUnwindSafe> Allocation for T

impl Drop for RPreservedSexp {
    fn drop(&mut self) {
        // SAFETY: R_ReleaseObject is thread-safe (mutex-protected in R 4.0+).
        // We use _unchecked because this Drop may fire off the R main thread.
        unsafe { ffi::R_ReleaseObject_unchecked(self.0) }
    }
}

// endregion

// region: Zero-copy buffer helpers

/// Create an Arrow Buffer backed by R vector memory (zero-copy).
///
/// The returned Buffer holds a GC guard (via `R_PreserveObject`) that keeps
/// the R SEXP alive. When all references to the Buffer are dropped, the
/// guard releases the R object for GC.
///
/// # Safety
///
/// - `sexp` must be a valid R vector with contiguous data of type `T`
/// - Must be called on R's main thread (for `R_PreserveObject`)
unsafe fn sexp_to_arrow_buffer<T: RNativeType>(sexp: SEXP) -> arrow_buffer::Buffer {
    let len = sexp.len();
    if len == 0 {
        return arrow_buffer::Buffer::from(Vec::<u8>::new());
    }

    // Preserve the R object so it won't be GC'd while Arrow holds a reference
    unsafe { ffi::R_PreserveObject(sexp) };
    let guard = Arc::new(RPreservedSexp(sexp));

    let ptr = unsafe { ffi::DATAPTR_RO(sexp) } as *const u8;
    let byte_len = len * std::mem::size_of::<T>();

    // SAFETY: R vectors have contiguous memory. The guard keeps the SEXP alive.
    unsafe {
        arrow_buffer::Buffer::from_custom_allocation(
            std::ptr::NonNull::new_unchecked(ptr as *mut u8),
            byte_len,
            guard,
        )
    }
}

// endregion

// region: NA bitmap construction

use crate::altrep_traits::{NA_INTEGER, NA_REAL};

/// Check if an f64 value is R's NA_real_ (specific NaN bit pattern).
#[inline]
fn is_na_real(value: f64) -> bool {
    value.to_bits() == NA_REAL.to_bits()
}

/// Scan an R integer vector for `NA_integer_` and build an Arrow NullBuffer.
///
/// Returns `None` if no NAs found (all values valid).
fn build_i32_null_buffer(slice: &[i32]) -> Option<arrow_buffer::NullBuffer> {
    if !slice.contains(&NA_INTEGER) {
        return None;
    }
    let mut builder = arrow_buffer::BooleanBufferBuilder::new(slice.len());
    for &v in slice {
        builder.append(v != NA_INTEGER);
    }
    Some(arrow_buffer::NullBuffer::new(builder.finish()))
}

/// Scan an R real vector for `NA_real_` and build an Arrow NullBuffer.
///
/// Returns `None` if no NAs found (all values valid).
fn build_f64_null_buffer(slice: &[f64]) -> Option<arrow_buffer::NullBuffer> {
    let has_any_na = slice.iter().any(|&v| is_na_real(v));
    if !has_any_na {
        return None;
    }
    let mut builder = arrow_buffer::BooleanBufferBuilder::new(slice.len());
    for &v in slice {
        builder.append(!is_na_real(v));
    }
    Some(arrow_buffer::NullBuffer::new(builder.finish()))
}

// endregion

// region: TryFromSexp — zero-copy primitives (R → Arrow)

impl TryFromSexp for Float64Array {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::REALSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::REALSXP,
                actual,
            }
            .into());
        }
        let len = sexp.len();
        if len == 0 {
            return Ok(Float64Array::from(Vec::<f64>::new()));
        }

        // Zero-copy: wrap R's data buffer
        let buffer = unsafe { sexp_to_arrow_buffer::<f64>(sexp) };
        let scalar_buffer = arrow_buffer::ScalarBuffer::<f64>::from(buffer);

        // Scan for NAs to build null bitmap (data stays untouched)
        let slice: &[f64] = unsafe { sexp.as_slice() };
        let null_buffer = build_f64_null_buffer(slice);

        Ok(Float64Array::new(scalar_buffer, null_buffer))
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Self::try_from_sexp(sexp)
    }
}

impl TryFromSexp for Int32Array {
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
        let len = sexp.len();
        if len == 0 {
            return Ok(Int32Array::from(Vec::<i32>::new()));
        }

        let buffer = unsafe { sexp_to_arrow_buffer::<i32>(sexp) };
        let scalar_buffer = arrow_buffer::ScalarBuffer::<i32>::from(buffer);

        let slice: &[i32] = unsafe { sexp.as_slice() };
        let null_buffer = build_i32_null_buffer(slice);

        Ok(Int32Array::new(scalar_buffer, null_buffer))
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Self::try_from_sexp(sexp)
    }
}

impl TryFromSexp for UInt8Array {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::RAWSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::RAWSXP,
                actual,
            }
            .into());
        }
        let len = sexp.len();
        if len == 0 {
            return Ok(UInt8Array::from(Vec::<u8>::new()));
        }

        // Zero-copy, no NA concept for raw vectors
        let buffer = unsafe { sexp_to_arrow_buffer::<u8>(sexp) };
        let scalar_buffer = arrow_buffer::ScalarBuffer::<u8>::from(buffer);

        Ok(UInt8Array::new(scalar_buffer, None))
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Self::try_from_sexp(sexp)
    }
}

// endregion

// region: TryFromSexp — copy conversions (BooleanArray, StringArray)

impl TryFromSexp for BooleanArray {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::LGLSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::LGLSXP,
                actual,
            }
            .into());
        }
        let len = sexp.len();
        let mut builder = arrow_array::builder::BooleanBuilder::with_capacity(len);

        for i in 0..len {
            let val = unsafe { ffi::LOGICAL_ELT(sexp, i as R_xlen_t) };
            if val == crate::altrep_traits::NA_LOGICAL {
                builder.append_null();
            } else {
                builder.append_value(val != 0);
            }
        }

        Ok(builder.finish())
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Self::try_from_sexp(sexp)
    }
}

impl TryFromSexp for StringArray {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::STRSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::STRSXP,
                actual,
            }
            .into());
        }
        let len = sexp.len();
        let mut builder = arrow_array::builder::StringBuilder::with_capacity(len, len * 8);

        for i in 0..len {
            let charsxp = unsafe { ffi::STRING_ELT(sexp, i as R_xlen_t) };
            if std::ptr::addr_eq(charsxp.0, unsafe { R_NaString.0 }) {
                builder.append_null();
            } else {
                let s = unsafe { crate::from_r::charsxp_to_str(charsxp) };
                builder.append_value(s);
            }
        }

        Ok(builder.finish())
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Self::try_from_sexp(sexp)
    }
}

// endregion

// region: TryFromSexp — RecordBatch from data.frame

/// Convert a single R column SEXP to an Arrow ArrayRef.
///
/// Dispatches on `TYPEOF(sexp)` to choose the appropriate array type.
fn sexp_column_to_arrow(col_sexp: SEXP, col_name: &str) -> Result<(Field, ArrayRef), SexpError> {
    let (field, array): (Field, ArrayRef) = match col_sexp.type_of() {
        SEXPTYPE::REALSXP => {
            let arr = Float64Array::try_from_sexp(col_sexp)?;
            let nullable = arr.null_count() > 0;
            (
                Field::new(col_name, DataType::Float64, nullable),
                Arc::new(arr),
            )
        }
        SEXPTYPE::INTSXP => {
            let arr = Int32Array::try_from_sexp(col_sexp)?;
            let nullable = arr.null_count() > 0;
            (
                Field::new(col_name, DataType::Int32, nullable),
                Arc::new(arr),
            )
        }
        SEXPTYPE::LGLSXP => {
            let arr = BooleanArray::try_from_sexp(col_sexp)?;
            let nullable = arr.null_count() > 0;
            (
                Field::new(col_name, DataType::Boolean, nullable),
                Arc::new(arr),
            )
        }
        SEXPTYPE::STRSXP => {
            let arr = StringArray::try_from_sexp(col_sexp)?;
            let nullable = arr.null_count() > 0;
            (
                Field::new(col_name, DataType::Utf8, nullable),
                Arc::new(arr),
            )
        }
        SEXPTYPE::RAWSXP => {
            let arr = UInt8Array::try_from_sexp(col_sexp)?;
            (
                Field::new(col_name, DataType::UInt8, false),
                Arc::new(arr),
            )
        }
        other => {
            return Err(SexpError::InvalidValue(format!(
                "unsupported column type for Arrow conversion: {other:?}"
            )));
        }
    };
    Ok((field, array))
}

impl TryFromSexp for RecordBatch {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::VECSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::VECSXP,
                actual,
            }
            .into());
        }

        let ncol = sexp.len();

        // Get column names
        let names_sexp = unsafe { ffi::Rf_getAttrib(sexp, R_NamesSymbol) };
        let names: Vec<String> = if names_sexp.type_of() == SEXPTYPE::STRSXP {
            (0..ncol)
                .map(|i| {
                    let charsxp = unsafe { ffi::STRING_ELT(names_sexp, i as R_xlen_t) };
                    unsafe { crate::from_r::charsxp_to_str(charsxp) }.to_string()
                })
                .collect()
        } else {
            (0..ncol).map(|i| format!("V{}", i + 1)).collect()
        };

        let mut fields = Vec::with_capacity(ncol);
        let mut columns: Vec<ArrayRef> = Vec::with_capacity(ncol);

        for (i, name) in names.iter().enumerate() {
            let col_sexp = unsafe { ffi::VECTOR_ELT(sexp, i as R_xlen_t) };
            let (field, array) = sexp_column_to_arrow(col_sexp, name)?;
            fields.push(field);
            columns.push(array);
        }

        let schema = Arc::new(Schema::new(fields));
        RecordBatch::try_new(schema, columns)
            .map_err(|e| SexpError::InvalidValue(e.to_string()))
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Self::try_from_sexp(sexp)
    }
}

// endregion

// region: TryFromSexp — ArrayRef dynamic dispatch

impl TryFromSexp for ArrayRef {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        match sexp.type_of() {
            SEXPTYPE::REALSXP => Ok(Arc::new(Float64Array::try_from_sexp(sexp)?)),
            SEXPTYPE::INTSXP => Ok(Arc::new(Int32Array::try_from_sexp(sexp)?)),
            SEXPTYPE::LGLSXP => Ok(Arc::new(BooleanArray::try_from_sexp(sexp)?)),
            SEXPTYPE::STRSXP => Ok(Arc::new(StringArray::try_from_sexp(sexp)?)),
            SEXPTYPE::RAWSXP => Ok(Arc::new(UInt8Array::try_from_sexp(sexp)?)),
            other => Err(SexpError::InvalidValue(format!(
                "unsupported R type for Arrow conversion: {other:?}"
            ))),
        }
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Self::try_from_sexp(sexp)
    }
}

// endregion

// region: IntoR — Arrow arrays to R vectors

impl IntoR for Float64Array {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }

    fn into_sexp(self) -> SEXP {
        let n = self.len();
        unsafe {
            let sexp = ffi::Rf_allocVector(SEXPTYPE::REALSXP, n as R_xlen_t);
            let dst = ffi::REAL(sexp);
            if self.null_count() == 0 {
                std::ptr::copy_nonoverlapping(self.values().as_ptr(), dst, n);
            } else {
                for i in 0..n {
                    *dst.add(i) = if self.is_null(i) {
                        NA_REAL
                    } else {
                        self.value(i)
                    };
                }
            }
            sexp
        }
    }
}

impl IntoR for Int32Array {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }

    fn into_sexp(self) -> SEXP {
        let n = self.len();
        unsafe {
            let sexp = ffi::Rf_allocVector(SEXPTYPE::INTSXP, n as R_xlen_t);
            let dst = ffi::INTEGER(sexp);
            if self.null_count() == 0 {
                std::ptr::copy_nonoverlapping(self.values().as_ptr(), dst, n);
            } else {
                for i in 0..n {
                    *dst.add(i) = if self.is_null(i) {
                        NA_INTEGER
                    } else {
                        self.value(i)
                    };
                }
            }
            sexp
        }
    }
}

impl IntoR for UInt8Array {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }

    fn into_sexp(self) -> SEXP {
        let n = self.len();
        unsafe {
            let sexp = ffi::Rf_allocVector(SEXPTYPE::RAWSXP, n as R_xlen_t);
            let dst = ffi::RAW(sexp);
            if self.null_count() == 0 {
                std::ptr::copy_nonoverlapping(self.values().as_ptr(), dst, n);
            } else {
                for i in 0..n {
                    *dst.add(i) = if self.is_null(i) { 0 } else { self.value(i) };
                }
            }
            sexp
        }
    }
}

impl IntoR for BooleanArray {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }

    fn into_sexp(self) -> SEXP {
        let n = self.len();
        unsafe {
            let sexp = ffi::Rf_allocVector(SEXPTYPE::LGLSXP, n as R_xlen_t);
            let dst = ffi::LOGICAL(sexp);
            for i in 0..n {
                *dst.add(i) = if self.is_null(i) {
                    crate::altrep_traits::NA_LOGICAL
                } else if self.value(i) {
                    Rboolean::TRUE as i32
                } else {
                    Rboolean::FALSE as i32
                };
            }
            sexp
        }
    }
}

impl IntoR for StringArray {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }

    fn into_sexp(self) -> SEXP {
        let n = self.len();
        unsafe {
            let sexp = ffi::Rf_allocVector(SEXPTYPE::STRSXP, n as R_xlen_t);
            let guard = crate::gc_protect::OwnedProtect::new(sexp);
            for i in 0..n {
                if self.is_null(i) {
                    ffi::SET_STRING_ELT(guard.get(), i as R_xlen_t, R_NaString);
                } else {
                    let s = self.value(i);
                    let charsxp = ffi::Rf_mkCharLenCE(
                        s.as_ptr().cast(),
                        s.len() as i32,
                        ffi::cetype_t::CE_UTF8,
                    );
                    ffi::SET_STRING_ELT(guard.get(), i as R_xlen_t, charsxp);
                }
            }
            guard.get()
        }
    }
}

// endregion

// region: IntoR — RecordBatch to data.frame

/// Convert an Arrow ArrayRef to an R SEXP, dispatching on DataType.
fn arrow_array_to_sexp(array: &ArrayRef) -> SEXP {
    use arrow_array::cast::AsArray;

    match array.data_type() {
        DataType::Float64 => array.as_primitive::<Float64Type>().clone().into_sexp(),
        DataType::Int32 => array.as_primitive::<Int32Type>().clone().into_sexp(),
        DataType::UInt8 => array.as_primitive::<UInt8Type>().clone().into_sexp(),
        DataType::Boolean => array.as_boolean().clone().into_sexp(),
        DataType::Utf8 => array.as_string::<i32>().clone().into_sexp(),
        // Fallback for unsupported types: return character vector of NA
        _ => {
            let n = array.len();
            unsafe {
                let sexp = ffi::Rf_allocVector(SEXPTYPE::STRSXP, n as R_xlen_t);
                let guard = crate::gc_protect::OwnedProtect::new(sexp);
                for i in 0..n {
                    ffi::SET_STRING_ELT(guard.get(), i as R_xlen_t, R_NaString);
                }
                guard.get()
            }
        }
    }
}

impl IntoR for RecordBatch {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }

    fn into_sexp(self) -> SEXP {
        let ncol = self.num_columns();
        let nrow = self.num_rows();
        let schema = self.schema();

        unsafe {
            let scope = crate::gc_protect::ProtectScope::new();

            // Create list for columns
            let list = scope.protect_raw(ffi::Rf_allocVector(
                SEXPTYPE::VECSXP,
                ncol as R_xlen_t,
            ));

            // Create names vector
            let names = scope.protect_raw(ffi::Rf_allocVector(
                SEXPTYPE::STRSXP,
                ncol as R_xlen_t,
            ));

            for (i, (col, field)) in
                self.columns().iter().zip(schema.fields()).enumerate()
            {
                let col_sexp = arrow_array_to_sexp(col);
                ffi::SET_VECTOR_ELT(list, i as R_xlen_t, col_sexp);

                let name = field.name();
                let charsxp = ffi::Rf_mkCharLenCE(
                    name.as_ptr().cast(),
                    name.len() as i32,
                    ffi::cetype_t::CE_UTF8,
                );
                ffi::SET_STRING_ELT(names, i as R_xlen_t, charsxp);
            }

            // Set names attribute
            ffi::Rf_setAttrib(list, R_NamesSymbol, names);

            // Set class = "data.frame"
            let class_str = scope.protect_raw(ffi::Rf_allocVector(SEXPTYPE::STRSXP, 1));
            ffi::SET_STRING_ELT(
                class_str,
                0,
                ffi::Rf_mkCharLenCE(
                    c"data.frame".as_ptr(),
                    10,
                    ffi::cetype_t::CE_UTF8,
                ),
            );
            ffi::Rf_setAttrib(list, R_ClassSymbol, class_str);

            // Set compact row.names: c(NA_integer_, -nrow)
            let rownames = scope.protect_raw(ffi::Rf_allocVector(SEXPTYPE::INTSXP, 2));
            let rn_ptr = ffi::INTEGER(rownames);
            *rn_ptr = NA_INTEGER;
            *rn_ptr.add(1) = -(nrow as i32);
            ffi::Rf_setAttrib(list, R_RowNamesSymbol, rownames);

            list
        }
    }
}

// endregion

// region: IntoR — ArrayRef dynamic dispatch

impl IntoR for ArrayRef {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }

    fn into_sexp(self) -> SEXP {
        arrow_array_to_sexp(&self)
    }
}

// endregion
