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
    self, Array, ArrayRef, BooleanArray, Date32Array, DictionaryArray, Float64Array, Int32Array,
    RecordBatch, StringArray, TimestampSecondArray, UInt8Array,
    types::{Date32Type, Float64Type, Int32Type, TimestampSecondType, UInt8Type},
};
pub use arrow_buffer;
pub use arrow_schema::{self, DataType, Field, Schema};

use crate::ffi::{
    self, R_ClassSymbol, R_NaString, R_NamesSymbol, R_RowNamesSymbol, R_xlen_t, RNativeType,
    Rboolean, SEXP, SEXPTYPE, SexpExt,
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

// region: TryFromSexp — Factor, Date, POSIXct (R class-aware conversions)

/// Type alias for dictionary-encoded string arrays (Arrow equivalent of R factors).
pub type StringDictionaryArray = DictionaryArray<Int32Type>;

/// Check if an R SEXP has a specific class (checks "class" attribute).
///
/// `class` must be a NUL-terminated string (e.g., `"factor\0"`).
unsafe fn r_inherits(sexp: SEXP, class: &str) -> bool {
    debug_assert!(class.ends_with('\0'), "class must be NUL-terminated");
    unsafe { ffi::Rf_inherits(sexp, class.as_ptr().cast()) != Rboolean::FALSE }
}

/// Check if an R SEXP is a factor (INTSXP with "levels" attribute).
unsafe fn is_factor(sexp: SEXP) -> bool {
    sexp.type_of() == SEXPTYPE::INTSXP && unsafe { r_inherits(sexp, "factor\0") }
}

/// Check if an R SEXP is a Date (REALSXP with class "Date").
unsafe fn is_date(sexp: SEXP) -> bool {
    sexp.type_of() == SEXPTYPE::REALSXP && unsafe { r_inherits(sexp, "Date\0") }
}

/// Check if an R SEXP is POSIXct (REALSXP with class "POSIXct").
unsafe fn is_posixct(sexp: SEXP) -> bool {
    sexp.type_of() == SEXPTYPE::REALSXP && unsafe { r_inherits(sexp, "POSIXct\0") }
}

/// Convert R factor to Arrow DictionaryArray<Int32Type> with string values.
///
/// R factors are INTSXP with 1-based indices into a "levels" character vector.
/// Arrow DictionaryArray uses 0-based indices, so we subtract 1.
/// NA in factor (NA_integer_) → null in the dictionary keys.
impl TryFromSexp for StringDictionaryArray {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if !unsafe { is_factor(sexp) } {
            return Err(SexpError::InvalidValue(
                "expected R factor (integer with levels attribute)".into(),
            ));
        }

        let n = sexp.len();
        let levels_sexp = sexp.get_levels();
        if levels_sexp.type_of() != SEXPTYPE::STRSXP {
            return Err(SexpError::InvalidValue(
                "factor missing levels attribute".into(),
            ));
        }

        // Build the dictionary (levels)
        let n_levels = levels_sexp.len();
        let mut dict_builder =
            arrow_array::builder::StringBuilder::with_capacity(n_levels, n_levels * 8);
        for i in 0..n_levels {
            let charsxp = unsafe { ffi::STRING_ELT(levels_sexp, i as R_xlen_t) };
            let s = unsafe { crate::from_r::charsxp_to_str(charsxp) };
            dict_builder.append_value(s);
        }
        let dictionary = dict_builder.finish();

        // Build the keys (1-based → 0-based, NA → null)
        let slice: &[i32] = unsafe { sexp.as_slice() };
        let mut keys_builder = arrow_array::builder::Int32Builder::with_capacity(n);
        for &v in slice {
            if v == NA_INTEGER {
                keys_builder.append_null();
            } else {
                keys_builder.append_value(v - 1); // R is 1-based, Arrow is 0-based
            }
        }
        let keys = keys_builder.finish();

        DictionaryArray::try_new(keys, Arc::new(dictionary))
            .map_err(|e| SexpError::InvalidValue(e.to_string()))
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Self::try_from_sexp(sexp)
    }
}

/// Convert R Date to Arrow Date32Array.
///
/// R Date values are doubles (days since 1970-01-01). Arrow Date32 is i32 (same epoch).
impl TryFromSexp for Date32Array {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if !unsafe { is_date(sexp) } {
            return Err(SexpError::InvalidValue(
                "expected R Date object (numeric with class 'Date')".into(),
            ));
        }

        let n = sexp.len();
        let slice: &[f64] = unsafe { sexp.as_slice() };
        let mut builder = arrow_array::builder::Date32Builder::with_capacity(n);

        for &v in slice {
            if is_na_real(v) {
                builder.append_null();
            } else {
                builder.append_value(v as i32); // f64 days → i32 days
            }
        }

        Ok(builder.finish())
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Self::try_from_sexp(sexp)
    }
}

/// Convert R POSIXct to Arrow TimestampSecondArray.
///
/// R POSIXct values are doubles (seconds since Unix epoch, possibly fractional).
/// Arrow TimestampSecondArray uses i64 seconds. Fractional seconds are truncated.
/// Timezone from R's "tzone" attribute is preserved if present.
pub fn posixct_to_timestamp(sexp: SEXP) -> Result<TimestampSecondArray, SexpError> {
    if !unsafe { is_posixct(sexp) } {
        return Err(SexpError::InvalidValue(
            "expected R POSIXct object (numeric with class 'POSIXct')".into(),
        ));
    }

    let n = sexp.len();
    let slice: &[f64] = unsafe { sexp.as_slice() };

    // Extract timezone if present
    let tzone_sexp = sexp.get_attr(unsafe { ffi::Rf_install(c"tzone".as_ptr()) });
    let tz: Option<Arc<str>> = if tzone_sexp.type_of() == SEXPTYPE::STRSXP && tzone_sexp.len() > 0 {
        let charsxp = unsafe { ffi::STRING_ELT(tzone_sexp, 0) };
        let s = unsafe { crate::from_r::charsxp_to_str(charsxp) };
        if s.is_empty() {
            None
        } else {
            Some(Arc::from(s))
        }
    } else {
        None
    };

    let mut builder = arrow_array::builder::TimestampSecondBuilder::with_capacity(n);
    for &v in slice {
        if is_na_real(v) {
            builder.append_null();
        } else {
            builder.append_value(v as i64); // f64 seconds → i64 seconds
        }
    }

    let mut arr = builder.finish();
    if let Some(tz) = tz {
        arr = arr.with_timezone(tz);
    }
    Ok(arr)
}

// endregion

// region: IntoR — Factor, Date, POSIXct

/// Convert Arrow DictionaryArray<Int32Type> to R factor.
impl IntoR for StringDictionaryArray {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }

    fn into_sexp(self) -> SEXP {
        use arrow_array::cast::AsArray;

        let n = self.len();
        let keys = self.keys();
        let values = self.values().as_string::<i32>();

        unsafe {
            let scope = crate::gc_protect::ProtectScope::new();

            // Create integer vector for factor codes (0-based → 1-based)
            let (codes, codes_dst) = crate::into_r::alloc_r_vector::<i32>(n);
            scope.protect_raw(codes);
            for (i, slot) in codes_dst.iter_mut().enumerate() {
                *slot = if self.is_null(i) {
                    NA_INTEGER
                } else {
                    keys.value(i) + 1 // Arrow 0-based → R 1-based
                };
            }

            // Create levels character vector
            let n_levels = Array::len(&*values);
            let levels = scope.alloc_character(n_levels).into_raw();
            for i in 0..n_levels {
                let s = values.value(i);
                let charsxp =
                    SEXP::charsxp(s);
                ffi::SET_STRING_ELT(levels, i as R_xlen_t, charsxp);
            }

            // Set levels and class attributes
            codes.set_levels(levels);
            let class_str = scope.alloc_character(1).into_raw();
            ffi::SET_STRING_ELT(
                class_str,
                0,
                SEXP::charsxp("factor"),
            );
            codes.set_class(class_str);

            codes
        }
    }
}

/// Convert Arrow Date32Array to R Date.
impl IntoR for Date32Array {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }

    fn into_sexp(self) -> SEXP {
        let n = self.len();
        unsafe {
            let scope = crate::gc_protect::ProtectScope::new();

            let (sexp, dst) = crate::into_r::alloc_r_vector::<f64>(n);
            scope.protect_raw(sexp);
            for (i, slot) in dst.iter_mut().enumerate() {
                *slot = if self.is_null(i) {
                    NA_REAL
                } else {
                    self.value(i) as f64
                };
            }

            // Set class = "Date"
            let class_str = scope.alloc_character(1).into_raw();
            ffi::SET_STRING_ELT(
                class_str,
                0,
                SEXP::charsxp("Date"),
            );
            sexp.set_class(class_str);

            sexp
        }
    }
}

/// Convert Arrow TimestampSecondArray to R POSIXct.
impl IntoR for TimestampSecondArray {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }

    fn into_sexp(self) -> SEXP {
        let n = self.len();
        let tz = match self.data_type() {
            DataType::Timestamp(_, Some(tz)) => Some(tz.clone()),
            _ => None,
        };

        unsafe {
            let scope = crate::gc_protect::ProtectScope::new();

            let (sexp, dst) = crate::into_r::alloc_r_vector::<f64>(n);
            scope.protect_raw(sexp);
            for (i, slot) in dst.iter_mut().enumerate() {
                *slot = if self.is_null(i) {
                    NA_REAL
                } else {
                    self.value(i) as f64
                };
            }

            // Set class = c("POSIXct", "POSIXt")
            let class_str = scope.alloc_character(2).into_raw();
            ffi::SET_STRING_ELT(
                class_str,
                0,
                SEXP::charsxp("POSIXct"),
            );
            ffi::SET_STRING_ELT(
                class_str,
                1,
                SEXP::charsxp("POSIXt"),
            );
            sexp.set_class(class_str);

            // Set tzone attribute if present
            if let Some(tz) = tz {
                let tz_str = scope.alloc_character(1).into_raw();
                ffi::SET_STRING_ELT(
                    tz_str,
                    0,
                    ffi::Rf_mkCharLenCE(
                        tz.as_ptr().cast(),
                        tz.len() as i32,
                        ffi::cetype_t::CE_UTF8,
                    ),
                );
                sexp.set_attr(ffi::Rf_install(c"tzone".as_ptr()), tz_str);
            }

            sexp
        }
    }
}

// endregion

// region: TryFromSexp — RecordBatch from data.frame (class-aware dispatch)

/// Convert a single R column SEXP to an Arrow ArrayRef.
///
/// Dispatches on R class attributes first (factor, Date, POSIXct), then
/// falls back to TYPEOF for plain vectors.
fn sexp_column_to_arrow(col_sexp: SEXP, col_name: &str) -> Result<(Field, ArrayRef), SexpError> {
    // Check class-based types first (before plain TYPEOF dispatch)
    unsafe {
        if is_factor(col_sexp) {
            let arr = StringDictionaryArray::try_from_sexp(col_sexp)?;
            let nullable = arr.logical_null_count() > 0;
            return Ok((
                Field::new(
                    col_name,
                    DataType::Dictionary(Box::new(DataType::Int32), Box::new(DataType::Utf8)),
                    nullable,
                ),
                Arc::new(arr),
            ));
        }
        if is_date(col_sexp) {
            let arr = Date32Array::try_from_sexp(col_sexp)?;
            let nullable = arr.null_count() > 0;
            return Ok((
                Field::new(col_name, DataType::Date32, nullable),
                Arc::new(arr),
            ));
        }
        if is_posixct(col_sexp) {
            let arr = posixct_to_timestamp(col_sexp)?;
            let nullable = arr.null_count() > 0;
            return Ok((
                Field::new(col_name, arr.data_type().clone(), nullable),
                Arc::new(arr),
            ));
        }
    }

    // Plain TYPEOF dispatch
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
            (Field::new(col_name, DataType::UInt8, false), Arc::new(arr))
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
        let names_sexp = sexp.get_names();
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
            let col_sexp = sexp.vector_elt(i as R_xlen_t);
            let (field, array) = sexp_column_to_arrow(col_sexp, name)?;
            fields.push(field);
            columns.push(array);
        }

        let schema = Arc::new(Schema::new(fields));
        RecordBatch::try_new(schema, columns).map_err(|e| SexpError::InvalidValue(e.to_string()))
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
        unsafe {
            let (sexp, dst) = crate::into_r::alloc_r_vector::<f64>(self.len());
            if self.null_count() == 0 {
                dst.copy_from_slice(self.values().as_ref());
            } else {
                for (i, slot) in dst.iter_mut().enumerate() {
                    *slot = if self.is_null(i) {
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
        unsafe {
            let (sexp, dst) = crate::into_r::alloc_r_vector::<i32>(self.len());
            if self.null_count() == 0 {
                dst.copy_from_slice(self.values().as_ref());
            } else {
                for (i, slot) in dst.iter_mut().enumerate() {
                    *slot = if self.is_null(i) {
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
        unsafe {
            let (sexp, dst) = crate::into_r::alloc_r_vector::<u8>(self.len());
            if self.null_count() == 0 {
                dst.copy_from_slice(self.values().as_ref());
            } else {
                for (i, slot) in dst.iter_mut().enumerate() {
                    *slot = if self.is_null(i) { 0 } else { self.value(i) };
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
        unsafe {
            let (sexp, dst) = crate::into_r::alloc_r_vector::<crate::ffi::RLogical>(self.len());
            let dst_i32: &mut [i32] =
                std::slice::from_raw_parts_mut(dst.as_mut_ptr().cast::<i32>(), self.len());
            for (i, slot) in dst_i32.iter_mut().enumerate() {
                *slot = if self.is_null(i) {
                    crate::altrep_traits::NA_LOGICAL
                } else if self.value(i) {
                    1
                } else {
                    0
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
        let n = Array::len(&self);
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
        DataType::Float32 => {
            // Widen f32 → f64 for R
            let arr = array.as_primitive::<arrow_array::types::Float32Type>();
            let widened: Float64Array = arr.iter().map(|v| v.map(|x| x as f64)).collect();
            widened.into_sexp()
        }
        DataType::Int32 => array.as_primitive::<Int32Type>().clone().into_sexp(),
        DataType::Int64 => {
            // R has no i64 — convert to f64 (may lose precision for values > 2^53)
            let arr = array.as_primitive::<arrow_array::types::Int64Type>();
            let converted: Float64Array = arr.iter().map(|v| v.map(|x| x as f64)).collect();
            converted.into_sexp()
        }
        DataType::Int16 => {
            let arr = array.as_primitive::<arrow_array::types::Int16Type>();
            let widened: Int32Array = arr.iter().map(|v| v.map(|x| x as i32)).collect();
            widened.into_sexp()
        }
        DataType::Int8 => {
            let arr = array.as_primitive::<arrow_array::types::Int8Type>();
            let widened: Int32Array = arr.iter().map(|v| v.map(|x| x as i32)).collect();
            widened.into_sexp()
        }
        DataType::UInt8 => array.as_primitive::<UInt8Type>().clone().into_sexp(),
        DataType::Boolean => array.as_boolean().clone().into_sexp(),
        DataType::Utf8 => array.as_string::<i32>().clone().into_sexp(),
        DataType::Date32 => array.as_primitive::<Date32Type>().clone().into_sexp(),
        DataType::Timestamp(arrow_schema::TimeUnit::Second, _) => array
            .as_primitive::<TimestampSecondType>()
            .clone()
            .into_sexp(),
        DataType::Dictionary(key_type, _) if key_type.as_ref() == &DataType::Int32 => {
            array
                .as_any()
                .downcast_ref::<StringDictionaryArray>()
                .cloned()
                .map(|a| a.into_sexp())
                .unwrap_or_else(|| {
                    // Not a string dictionary, fall through to default
                    let n = array.len();
                    unsafe {
                        let sexp = ffi::Rf_allocVector(SEXPTYPE::STRSXP, n as R_xlen_t);
                        let guard = crate::gc_protect::OwnedProtect::new(sexp);
                        for i in 0..n {
                            ffi::SET_STRING_ELT(guard.get(), i as R_xlen_t, R_NaString);
                        }
                        guard.get()
                    }
                })
        }
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
            let list = scope.alloc_vecsxp(ncol).into_raw();

            // Create names vector
            let names = scope.alloc_character(ncol).into_raw();

            for (i, (col, field)) in self.columns().iter().zip(schema.fields()).enumerate() {
                let col_sexp = arrow_array_to_sexp(col);
                list.set_vector_elt(i as R_xlen_t, col_sexp);

                let name = field.name();
                let charsxp = ffi::Rf_mkCharLenCE(
                    name.as_ptr().cast(),
                    name.len() as i32,
                    ffi::cetype_t::CE_UTF8,
                );
                ffi::SET_STRING_ELT(names, i as R_xlen_t, charsxp);
            }

            // Set names attribute
            list.set_names(names);

            // Set class = "data.frame"
            let class_str = scope.alloc_character(1).into_raw();
            ffi::SET_STRING_ELT(
                class_str,
                0,
                SEXP::charsxp("data.frame"),
            );
            list.set_class(class_str);

            // Set compact row.names: c(NA_integer_, -nrow)
            let (rownames, rn) = crate::into_r::alloc_r_vector::<i32>(2);
            scope.protect_raw(rownames);
            rn[0] = NA_INTEGER;
            rn[1] = -(nrow as i32);
            list.set_row_names(rownames);

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

// region: ALTREP support for Arrow arrays (Lazy<T>)
//
// These impls allow `Lazy<Float64Array>`, `Lazy<Int32Array>`, etc. to return
// Arrow data as ALTREP vectors. R reads elements on demand; for null-free
// arrays the Dataptr callback returns the Arrow buffer pointer directly
// (true zero-copy, O(1)).

use crate::altrep_data::{
    AltIntegerData, AltLogicalData, AltRawData, AltRealData, AltrepDataptr, AltrepLen, Logical,
};
use crate::externalptr::TypedExternal;

// region: TypedExternal impls (required for ExternalPtr wrapping in ALTREP)

impl TypedExternal for Float64Array {
    const TYPE_NAME: &'static str = "arrow::Float64Array";
    const TYPE_NAME_CSTR: &'static [u8] = b"arrow::Float64Array\0";
    const TYPE_ID_CSTR: &'static [u8] = b"arrow::Float64Array\0";
}

impl TypedExternal for Int32Array {
    const TYPE_NAME: &'static str = "arrow::Int32Array";
    const TYPE_NAME_CSTR: &'static [u8] = b"arrow::Int32Array\0";
    const TYPE_ID_CSTR: &'static [u8] = b"arrow::Int32Array\0";
}

impl TypedExternal for UInt8Array {
    const TYPE_NAME: &'static str = "arrow::UInt8Array";
    const TYPE_NAME_CSTR: &'static [u8] = b"arrow::UInt8Array\0";
    const TYPE_ID_CSTR: &'static [u8] = b"arrow::UInt8Array\0";
}

impl TypedExternal for BooleanArray {
    const TYPE_NAME: &'static str = "arrow::BooleanArray";
    const TYPE_NAME_CSTR: &'static [u8] = b"arrow::BooleanArray\0";
    const TYPE_ID_CSTR: &'static [u8] = b"arrow::BooleanArray\0";
}

impl TypedExternal for StringArray {
    const TYPE_NAME: &'static str = "arrow::StringArray";
    const TYPE_NAME_CSTR: &'static [u8] = b"arrow::StringArray\0";
    const TYPE_ID_CSTR: &'static [u8] = b"arrow::StringArray\0";
}

// endregion

// region: AltrepLen impls

impl AltrepLen for Float64Array {
    fn len(&self) -> usize {
        Array::len(self)
    }
}

impl AltrepLen for Int32Array {
    fn len(&self) -> usize {
        Array::len(self)
    }
}

impl AltrepLen for UInt8Array {
    fn len(&self) -> usize {
        Array::len(self)
    }
}

impl AltrepLen for BooleanArray {
    fn len(&self) -> usize {
        Array::len(self)
    }
}

// endregion

// region: ALTREP data trait impls

impl AltRealData for Float64Array {
    fn elt(&self, i: usize) -> f64 {
        if self.is_null(i) {
            NA_REAL
        } else {
            self.value(i)
        }
    }

    fn as_slice(&self) -> Option<&[f64]> {
        // Zero-copy: return Arrow buffer pointer directly if no nulls.
        // Arrow's null bitmap marks nulls but data buffer has garbage there,
        // so we can only expose the raw buffer when null_count == 0.
        if self.null_count() == 0 {
            Some(self.values().as_ref())
        } else {
            None
        }
    }

    fn no_na(&self) -> Option<bool> {
        Some(self.null_count() == 0)
    }
}

impl AltIntegerData for Int32Array {
    fn elt(&self, i: usize) -> i32 {
        if self.is_null(i) {
            crate::altrep_traits::NA_INTEGER
        } else {
            self.value(i)
        }
    }

    fn as_slice(&self) -> Option<&[i32]> {
        if self.null_count() == 0 {
            Some(self.values().as_ref())
        } else {
            None
        }
    }

    fn no_na(&self) -> Option<bool> {
        Some(self.null_count() == 0)
    }
}

impl AltRawData for UInt8Array {
    fn elt(&self, i: usize) -> u8 {
        if self.is_null(i) { 0 } else { self.value(i) }
    }

    fn as_slice(&self) -> Option<&[u8]> {
        if self.null_count() == 0 {
            Some(self.values().as_ref())
        } else {
            None
        }
    }
}

impl AltLogicalData for BooleanArray {
    fn elt(&self, i: usize) -> Logical {
        if self.is_null(i) {
            Logical::Na
        } else if self.value(i) {
            Logical::True
        } else {
            Logical::False
        }
    }

    fn no_na(&self) -> Option<bool> {
        Some(self.null_count() == 0)
    }
}

// endregion

// region: AltrepDataptr impls (zero-copy when no nulls)

impl AltrepDataptr<f64> for Float64Array {
    fn dataptr(&mut self, _writable: bool) -> Option<*mut f64> {
        // Arrow buffers are immutable — can't provide writable pointer.
        // Return read-only pointer cast to mut (R handles const-correctness).
        if self.null_count() == 0 {
            Some(self.values().as_ptr() as *mut f64)
        } else {
            None
        }
    }

    fn dataptr_or_null(&self) -> Option<*const f64> {
        if self.null_count() == 0 {
            Some(self.values().as_ptr())
        } else {
            None
        }
    }
}

impl AltrepDataptr<i32> for Int32Array {
    fn dataptr(&mut self, _writable: bool) -> Option<*mut i32> {
        if self.null_count() == 0 {
            Some(self.values().as_ptr() as *mut i32)
        } else {
            None
        }
    }

    fn dataptr_or_null(&self) -> Option<*const i32> {
        if self.null_count() == 0 {
            Some(self.values().as_ptr())
        } else {
            None
        }
    }
}

impl AltrepDataptr<u8> for UInt8Array {
    fn dataptr(&mut self, _writable: bool) -> Option<*mut u8> {
        if self.null_count() == 0 {
            Some(self.values().as_ptr() as *mut u8)
        } else {
            None
        }
    }

    fn dataptr_or_null(&self) -> Option<*const u8> {
        if self.null_count() == 0 {
            Some(self.values().as_ptr())
        } else {
            None
        }
    }
}

// endregion

// region: ALTREP bridge traits + InferBase + RegisterAltrep
//
// impl_alt*_from_data! generates the bridge traits (Altrep, AltVec, AltReal/etc.)
// AND calls impl_inferbase_*! to register the class creation.

crate::impl_altreal_from_data!(Float64Array, dataptr);
crate::impl_altinteger_from_data!(Int32Array, dataptr);
crate::impl_altraw_from_data!(UInt8Array, dataptr);
crate::impl_altlogical_from_data!(BooleanArray);

use crate::altrep::RegisterAltrep;

impl RegisterAltrep for Float64Array {
    fn get_or_init_class() -> crate::ffi::altrep::R_altrep_class_t {
        use std::sync::OnceLock;
        static CLASS: OnceLock<crate::ffi::altrep::R_altrep_class_t> = OnceLock::new();
        *CLASS.get_or_init(|| {
            let cls = unsafe {
                <Float64Array as crate::altrep_data::InferBase>::make_class(
                    b"arrow_Float64Array\0".as_ptr().cast(),
                    crate::AltrepPkgName::as_ptr(),
                )
            };
            unsafe {
                <Float64Array as crate::altrep_data::InferBase>::install_methods(cls);
            }
            cls
        })
    }
}

impl RegisterAltrep for Int32Array {
    fn get_or_init_class() -> crate::ffi::altrep::R_altrep_class_t {
        use std::sync::OnceLock;
        static CLASS: OnceLock<crate::ffi::altrep::R_altrep_class_t> = OnceLock::new();
        *CLASS.get_or_init(|| {
            let cls = unsafe {
                <Int32Array as crate::altrep_data::InferBase>::make_class(
                    b"arrow_Int32Array\0".as_ptr().cast(),
                    crate::AltrepPkgName::as_ptr(),
                )
            };
            unsafe {
                <Int32Array as crate::altrep_data::InferBase>::install_methods(cls);
            }
            cls
        })
    }
}

impl RegisterAltrep for UInt8Array {
    fn get_or_init_class() -> crate::ffi::altrep::R_altrep_class_t {
        use std::sync::OnceLock;
        static CLASS: OnceLock<crate::ffi::altrep::R_altrep_class_t> = OnceLock::new();
        *CLASS.get_or_init(|| {
            let cls = unsafe {
                <UInt8Array as crate::altrep_data::InferBase>::make_class(
                    b"arrow_UInt8Array\0".as_ptr().cast(),
                    crate::AltrepPkgName::as_ptr(),
                )
            };
            unsafe {
                <UInt8Array as crate::altrep_data::InferBase>::install_methods(cls);
            }
            cls
        })
    }
}

impl RegisterAltrep for BooleanArray {
    fn get_or_init_class() -> crate::ffi::altrep::R_altrep_class_t {
        use std::sync::OnceLock;
        static CLASS: OnceLock<crate::ffi::altrep::R_altrep_class_t> = OnceLock::new();
        *CLASS.get_or_init(|| {
            let cls = unsafe {
                <BooleanArray as crate::altrep_data::InferBase>::make_class(
                    b"arrow_BooleanArray\0".as_ptr().cast(),
                    crate::AltrepPkgName::as_ptr(),
                )
            };
            unsafe {
                <BooleanArray as crate::altrep_data::InferBase>::install_methods(cls);
            }
            cls
        })
    }
}

// StringArray ALTREP — Elt creates CHARSXP on demand, no Dataptr (not contiguous).

impl AltrepLen for StringArray {
    fn len(&self) -> usize {
        Array::len(self)
    }
}

impl crate::altrep_data::AltStringData for StringArray {
    fn elt(&self, i: usize) -> Option<&str> {
        if self.is_null(i) {
            None
        } else {
            Some(self.value(i))
        }
    }

    fn no_na(&self) -> Option<bool> {
        Some(self.null_count() == 0)
    }
}

crate::impl_altstring_from_data!(StringArray);

impl RegisterAltrep for StringArray {
    fn get_or_init_class() -> crate::ffi::altrep::R_altrep_class_t {
        use std::sync::OnceLock;
        static CLASS: OnceLock<crate::ffi::altrep::R_altrep_class_t> = OnceLock::new();
        *CLASS.get_or_init(|| {
            let cls = unsafe {
                <StringArray as crate::altrep_data::InferBase>::make_class(
                    b"arrow_StringArray\0".as_ptr().cast(),
                    crate::AltrepPkgName::as_ptr(),
                )
            };
            unsafe {
                <StringArray as crate::altrep_data::InferBase>::install_methods(cls);
            }
            cls
        })
    }
}

// endregion
