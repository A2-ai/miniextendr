//! Integration with the `bytemuck` crate for POD type conversions.
//!
//! Provides explicit, safe conversions between Rust POD (Plain Old Data) types
//! and R raw vectors.
//!
//! # Features
//!
//! Enable this module with the `raw_conversions` feature:
//!
//! ```toml
//! [dependencies]
//! miniextendr-api = { version = "0.1", features = ["raw_conversions"] }
//! ```
//!
//! # Overview
//!
//! This module provides wrapper types for converting POD types to/from R raw vectors:
//!
//! - [`Raw<T>`] - Single POD value (headerless, native layout)
//! - [`RawSlice<T>`] - Sequence of POD values (headerless, native layout)
//! - [`RawTagged<T>`] - Single POD value with header metadata
//! - [`RawSliceTagged<T>`] - Sequence of POD values with header metadata
//!
//! # Safety Model
//!
//! - **Alignment checks** are mandatory. Misaligned data is copied to aligned buffers.
//! - **Length checks** are mandatory. Mismatched lengths return errors.
//! - **No endian conversion**: bytes are stored in native layout for speed.
//! - **Not portable**: raw bytes are architecture-specific.
//!
//! # Example
//!
//! ```ignore
//! use bytemuck::{Pod, Zeroable};
//! use miniextendr_api::raw_conversions::{Raw, RawSlice};
//!
//! #[derive(Copy, Clone, Pod, Zeroable)]
//! #[repr(C)]
//! struct Vec3 {
//!     x: f32,
//!     y: f32,
//!     z: f32,
//! }
//!
//! #[miniextendr]
//! fn encode_vec3(x: f64, y: f64, z: f64) -> Raw<Vec3> {
//!     Raw(Vec3 { x: x as f32, y: y as f32, z: z as f32 })
//! }
//!
//! #[miniextendr]
//! fn decode_vec3(raw: Raw<Vec3>) -> Vec<f64> {
//!     vec![raw.0.x as f64, raw.0.y as f64, raw.0.z as f64]
//! }
//! ```

pub use bytemuck::{Pod, Zeroable};

use std::fmt;
use std::mem;

use crate::ffi::{
    RAW, Rf_ScalarString, Rf_allocVector, Rf_install, Rf_mkCharLenCE, Rf_setAttrib, Rf_xlength,
    SEXP, SEXPTYPE, SexpExt, cetype_t,
};
use crate::from_r::{SexpError, SexpTypeError, TryFromSexp};
use crate::into_r::IntoR;

// =============================================================================
// Error type
// =============================================================================

/// Errors that can occur during raw conversion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawError {
    /// Length mismatch during conversion.
    LengthMismatch { expected: usize, actual: usize },
    /// Alignment mismatch (internal - we handle this by copying).
    AlignmentMismatch { align: usize },
    /// Invalid header in tagged format.
    InvalidHeader(String),
    /// Type name mismatch.
    TypeMismatch {
        expected: String,
        actual: Option<String>,
    },
}

impl fmt::Display for RawError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RawError::LengthMismatch { expected, actual } => {
                write!(
                    f,
                    "length mismatch: expected {expected} bytes, got {actual}"
                )
            }
            RawError::AlignmentMismatch { align } => {
                write!(f, "alignment mismatch: required {align}-byte alignment")
            }
            RawError::InvalidHeader(msg) => {
                write!(f, "invalid header: {msg}")
            }
            RawError::TypeMismatch { expected, actual } => match actual {
                Some(a) => write!(f, "type mismatch: expected {expected}, got {a}"),
                None => write!(f, "type mismatch: expected {expected}, got untagged data"),
            },
        }
    }
}

impl std::error::Error for RawError {}

// =============================================================================
// Header for tagged format
// =============================================================================

/// Header for tagged raw format.
///
/// Layout: magic (4 bytes) + version (4 bytes) + elem_size (4 bytes) + elem_count (4 bytes)
#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct RawHeader {
    /// Magic bytes: "MXRB"
    pub magic: [u8; 4],
    /// Format version (currently 1)
    pub version: u32,
    /// Size of each element in bytes
    pub elem_size: u32,
    /// Number of elements
    pub elem_count: u32,
}

impl RawHeader {
    /// Magic bytes for miniextendr raw format.
    pub const MAGIC: [u8; 4] = *b"MXRB";
    /// Current format version.
    pub const VERSION: u32 = 1;
    /// Header size in bytes.
    pub const SIZE: usize = mem::size_of::<RawHeader>();

    /// Create a new header for a single element.
    pub fn new_single<T: Pod>() -> Self {
        Self {
            magic: Self::MAGIC,
            version: Self::VERSION,
            elem_size: mem::size_of::<T>() as u32,
            elem_count: 1,
        }
    }

    /// Create a new header for a slice.
    pub fn new_slice<T: Pod>(count: usize) -> Self {
        Self {
            magic: Self::MAGIC,
            version: Self::VERSION,
            elem_size: mem::size_of::<T>() as u32,
            elem_count: count as u32,
        }
    }

    /// Validate header.
    pub fn validate<T: Pod>(&self, expected_count: Option<usize>) -> Result<(), RawError> {
        if self.magic != Self::MAGIC {
            return Err(RawError::InvalidHeader(format!(
                "invalid magic: expected {:?}, got {:?}",
                Self::MAGIC,
                self.magic
            )));
        }
        if self.version != Self::VERSION {
            return Err(RawError::InvalidHeader(format!(
                "unsupported version: expected {}, got {}",
                Self::VERSION,
                self.version
            )));
        }
        let expected_size = mem::size_of::<T>() as u32;
        if self.elem_size != expected_size {
            return Err(RawError::LengthMismatch {
                expected: expected_size as usize,
                actual: self.elem_size as usize,
            });
        }
        if let Some(count) = expected_count {
            if self.elem_count as usize != count {
                return Err(RawError::LengthMismatch {
                    expected: count,
                    actual: self.elem_count as usize,
                });
            }
        }
        Ok(())
    }
}

// =============================================================================
// Wrapper types
// =============================================================================

/// Wrapper for a single POD value (headerless, native layout).
///
/// Use this for fast serialization when portability is not needed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Raw<T: Pod>(pub T);

impl<T: Pod> Raw<T> {
    /// Unwrap the inner value.
    pub fn into_inner(self) -> T {
        self.0
    }

    /// Get a reference to the inner value.
    pub fn inner(&self) -> &T {
        &self.0
    }
}

/// Wrapper for a slice of POD values (headerless, native layout).
///
/// Use this for fast serialization when portability is not needed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawSlice<T: Pod>(pub Vec<T>);

impl<T: Pod> RawSlice<T> {
    /// Unwrap the inner vector.
    pub fn into_inner(self) -> Vec<T> {
        self.0
    }

    /// Get a reference to the inner vector.
    pub fn inner(&self) -> &[T] {
        &self.0
    }

    /// Get the number of elements.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Wrapper for a single POD value with header metadata.
///
/// The tagged format includes a header with magic bytes, version, and size info
/// for safer decoding across sessions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct RawTagged<T: Pod>(pub T);

impl<T: Pod> RawTagged<T> {
    /// Unwrap the inner value.
    pub fn into_inner(self) -> T {
        self.0
    }

    /// Get a reference to the inner value.
    pub fn inner(&self) -> &T {
        &self.0
    }
}

/// Wrapper for a slice of POD values with header metadata.
///
/// The tagged format includes a header with magic bytes, version, and size info
/// for safer decoding across sessions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawSliceTagged<T: Pod>(pub Vec<T>);

impl<T: Pod> RawSliceTagged<T> {
    /// Unwrap the inner vector.
    pub fn into_inner(self) -> Vec<T> {
        self.0
    }

    /// Get a reference to the inner vector.
    pub fn inner(&self) -> &[T] {
        &self.0
    }

    /// Get the number of elements.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

// =============================================================================
// Helper: get raw bytes from SEXP
// =============================================================================

/// Get raw bytes from a RAWSXP.
fn get_raw_bytes(sexp: SEXP) -> Result<&'static [u8], SexpError> {
    if sexp.type_of() != SEXPTYPE::RAWSXP {
        return Err(SexpError::Type(SexpTypeError {
            expected: SEXPTYPE::RAWSXP,
            actual: sexp.type_of(),
        }));
    }
    let len = unsafe { Rf_xlength(sexp) } as usize;
    let ptr = unsafe { RAW(sexp) };
    Ok(unsafe { std::slice::from_raw_parts(ptr, len) })
}

/// Align bytes to type T, copying if necessary.
fn align_bytes<T: Pod>(bytes: &[u8]) -> Result<T, RawError> {
    let expected = mem::size_of::<T>();
    if bytes.len() != expected {
        return Err(RawError::LengthMismatch {
            expected,
            actual: bytes.len(),
        });
    }

    // Try zero-copy first
    if let Ok(val) = bytemuck::try_from_bytes::<T>(bytes) {
        return Ok(*val);
    }

    // Fall back to copy if alignment is wrong
    let mut aligned = T::zeroed();
    let aligned_bytes = bytemuck::bytes_of_mut(&mut aligned);
    aligned_bytes.copy_from_slice(bytes);
    Ok(aligned)
}

/// Align bytes to slice of type T, copying if necessary.
fn align_slice<T: Pod>(bytes: &[u8]) -> Result<Vec<T>, RawError> {
    let elem_size = mem::size_of::<T>();
    if elem_size == 0 {
        // Zero-sized types
        return Ok(Vec::new());
    }
    if bytes.len() % elem_size != 0 {
        return Err(RawError::LengthMismatch {
            expected: bytes.len() - (bytes.len() % elem_size) + elem_size,
            actual: bytes.len(),
        });
    }

    let count = bytes.len() / elem_size;

    // Try zero-copy first
    if let Ok(slice) = bytemuck::try_cast_slice::<u8, T>(bytes) {
        return Ok(slice.to_vec());
    }

    // Fall back to element-by-element copy if alignment is wrong
    let mut result = Vec::with_capacity(count);
    for i in 0..count {
        let start = i * elem_size;
        let end = start + elem_size;
        let elem = align_bytes::<T>(&bytes[start..end])?;
        result.push(elem);
    }
    Ok(result)
}

// =============================================================================
// IntoR implementations
// =============================================================================

impl<T: Pod> IntoR for Raw<T> {
    fn into_sexp(self) -> SEXP {
        let bytes = bytemuck::bytes_of(&self.0);
        unsafe {
            let sexp = Rf_allocVector(SEXPTYPE::RAWSXP, bytes.len() as isize);
            let ptr = RAW(sexp);
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, bytes.len());
            sexp
        }
    }
}

impl<T: Pod> IntoR for RawSlice<T> {
    fn into_sexp(self) -> SEXP {
        let bytes = bytemuck::cast_slice::<T, u8>(&self.0);
        unsafe {
            let sexp = Rf_allocVector(SEXPTYPE::RAWSXP, bytes.len() as isize);
            let ptr = RAW(sexp);
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, bytes.len());
            sexp
        }
    }
}

impl<T: Pod> IntoR for RawTagged<T> {
    fn into_sexp(self) -> SEXP {
        let header = RawHeader::new_single::<T>();
        let header_bytes = bytemuck::bytes_of(&header);
        let value_bytes = bytemuck::bytes_of(&self.0);
        let total_len = header_bytes.len() + value_bytes.len();

        unsafe {
            let sexp = Rf_allocVector(SEXPTYPE::RAWSXP, total_len as isize);
            let ptr = RAW(sexp);
            std::ptr::copy_nonoverlapping(header_bytes.as_ptr(), ptr, header_bytes.len());
            std::ptr::copy_nonoverlapping(
                value_bytes.as_ptr(),
                ptr.add(header_bytes.len()),
                value_bytes.len(),
            );

            // Set type attribute
            let type_name = std::any::type_name::<T>();
            let attr_sym = Rf_install(c"mx_raw_type".as_ptr());
            let charsxp = Rf_mkCharLenCE(
                type_name.as_ptr().cast(),
                type_name.len() as i32,
                cetype_t::CE_UTF8,
            );
            Rf_setAttrib(sexp, attr_sym, Rf_ScalarString(charsxp));

            sexp
        }
    }
}

impl<T: Pod> IntoR for RawSliceTagged<T> {
    fn into_sexp(self) -> SEXP {
        let header = RawHeader::new_slice::<T>(self.0.len());
        let header_bytes = bytemuck::bytes_of(&header);
        let value_bytes = bytemuck::cast_slice::<T, u8>(&self.0);
        let total_len = header_bytes.len() + value_bytes.len();

        unsafe {
            let sexp = Rf_allocVector(SEXPTYPE::RAWSXP, total_len as isize);
            let ptr = RAW(sexp);
            std::ptr::copy_nonoverlapping(header_bytes.as_ptr(), ptr, header_bytes.len());
            std::ptr::copy_nonoverlapping(
                value_bytes.as_ptr(),
                ptr.add(header_bytes.len()),
                value_bytes.len(),
            );

            // Set type attribute
            let type_name = std::any::type_name::<T>();
            let attr_sym = Rf_install(c"mx_raw_type".as_ptr());
            let charsxp = Rf_mkCharLenCE(
                type_name.as_ptr().cast(),
                type_name.len() as i32,
                cetype_t::CE_UTF8,
            );
            Rf_setAttrib(sexp, attr_sym, Rf_ScalarString(charsxp));

            sexp
        }
    }
}

// =============================================================================
// TryFromSexp implementations
// =============================================================================

impl<T: Pod> TryFromSexp for Raw<T> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, SexpError> {
        let bytes = get_raw_bytes(sexp)?;
        let value = align_bytes::<T>(bytes)
            .map_err(|e| SexpError::InvalidValue(format!("Raw<T>: {}", e)))?;
        Ok(Raw(value))
    }
}

impl<T: Pod> TryFromSexp for RawSlice<T> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, SexpError> {
        let bytes = get_raw_bytes(sexp)?;
        let values = align_slice::<T>(bytes)
            .map_err(|e| SexpError::InvalidValue(format!("RawSlice<T>: {}", e)))?;
        Ok(RawSlice(values))
    }
}

impl<T: Pod> TryFromSexp for RawTagged<T> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, SexpError> {
        let bytes = get_raw_bytes(sexp)?;

        // Check minimum length
        if bytes.len() < RawHeader::SIZE {
            return Err(SexpError::InvalidValue(format!(
                "RawTagged<T>: expected at least {} bytes for header, got {}",
                RawHeader::SIZE,
                bytes.len()
            )));
        }

        // Parse header
        let header = align_bytes::<RawHeader>(&bytes[..RawHeader::SIZE])
            .map_err(|e| SexpError::InvalidValue(format!("RawTagged<T> header: {}", e)))?;

        // Validate header
        header
            .validate::<T>(Some(1))
            .map_err(|e| SexpError::InvalidValue(format!("RawTagged<T>: {}", e)))?;

        // Parse value
        let value_bytes = &bytes[RawHeader::SIZE..];
        let value = align_bytes::<T>(value_bytes)
            .map_err(|e| SexpError::InvalidValue(format!("RawTagged<T> value: {}", e)))?;

        Ok(RawTagged(value))
    }
}

impl<T: Pod> TryFromSexp for RawSliceTagged<T> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, SexpError> {
        let bytes = get_raw_bytes(sexp)?;

        // Check minimum length
        if bytes.len() < RawHeader::SIZE {
            return Err(SexpError::InvalidValue(format!(
                "RawSliceTagged<T>: expected at least {} bytes for header, got {}",
                RawHeader::SIZE,
                bytes.len()
            )));
        }

        // Parse header
        let header = align_bytes::<RawHeader>(&bytes[..RawHeader::SIZE])
            .map_err(|e| SexpError::InvalidValue(format!("RawSliceTagged<T> header: {}", e)))?;

        // Validate header (don't enforce count, derive from remaining bytes)
        header
            .validate::<T>(None)
            .map_err(|e| SexpError::InvalidValue(format!("RawSliceTagged<T>: {}", e)))?;

        // Parse values
        let value_bytes = &bytes[RawHeader::SIZE..];
        let values = align_slice::<T>(value_bytes)
            .map_err(|e| SexpError::InvalidValue(format!("RawSliceTagged<T> values: {}", e)))?;

        // Verify count matches header
        if values.len() != header.elem_count as usize {
            return Err(SexpError::InvalidValue(format!(
                "RawSliceTagged<T>: element count mismatch: header says {}, got {}",
                header.elem_count,
                values.len()
            )));
        }

        Ok(RawSliceTagged(values))
    }
}

// =============================================================================
// Helper functions
// =============================================================================

/// Encode a POD value to raw bytes.
pub fn raw_to_bytes<T: Pod>(value: &T) -> Vec<u8> {
    bytemuck::bytes_of(value).to_vec()
}

/// Decode a POD value from raw bytes.
pub fn raw_from_bytes<T: Pod>(bytes: &[u8]) -> Result<T, RawError> {
    align_bytes(bytes)
}

/// Encode a slice of POD values to raw bytes.
pub fn raw_slice_to_bytes<T: Pod>(values: &[T]) -> Vec<u8> {
    bytemuck::cast_slice::<T, u8>(values).to_vec()
}

/// Decode a slice of POD values from raw bytes.
pub fn raw_slice_from_bytes<T: Pod>(bytes: &[u8]) -> Result<Vec<T>, RawError> {
    align_slice(bytes)
}

// =============================================================================
// Unit tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Copy, Clone, Pod, Zeroable, Debug, PartialEq)]
    #[repr(C)]
    struct TestVec3 {
        x: f32,
        y: f32,
        z: f32,
    }

    #[derive(Copy, Clone, Pod, Zeroable, Debug, PartialEq)]
    #[repr(C)]
    struct TestPoint {
        x: i32,
        y: i32,
    }

    #[test]
    fn test_raw_bytes_roundtrip() {
        let value = TestVec3 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };
        let bytes = raw_to_bytes(&value);
        let decoded: TestVec3 = raw_from_bytes(&bytes).unwrap();
        assert_eq!(value, decoded);
    }

    #[test]
    fn test_raw_slice_bytes_roundtrip() {
        let values = vec![
            TestPoint { x: 1, y: 2 },
            TestPoint { x: 3, y: 4 },
            TestPoint { x: 5, y: 6 },
        ];
        let bytes = raw_slice_to_bytes(&values);
        let decoded: Vec<TestPoint> = raw_slice_from_bytes(&bytes).unwrap();
        assert_eq!(values, decoded);
    }

    #[test]
    fn test_raw_length_mismatch() {
        let bytes = vec![1u8, 2, 3]; // Not enough for TestVec3 (12 bytes)
        let result = raw_from_bytes::<TestVec3>(&bytes);
        assert!(matches!(result, Err(RawError::LengthMismatch { .. })));
    }

    #[test]
    fn test_raw_slice_length_mismatch() {
        let bytes = vec![1u8, 2, 3, 4, 5]; // Not a multiple of 8 (TestPoint size)
        let result = raw_slice_from_bytes::<TestPoint>(&bytes);
        assert!(matches!(result, Err(RawError::LengthMismatch { .. })));
    }

    #[test]
    fn test_header_validation() {
        let header = RawHeader::new_single::<TestVec3>();
        assert!(header.validate::<TestVec3>(Some(1)).is_ok());
        assert!(header.validate::<TestVec3>(Some(2)).is_err()); // Wrong count
        assert!(header.validate::<TestPoint>(Some(1)).is_err()); // Wrong size
    }

    #[test]
    fn test_header_invalid_magic() {
        let mut header = RawHeader::new_single::<TestVec3>();
        header.magic = *b"XXXX";
        assert!(matches!(
            header.validate::<TestVec3>(Some(1)),
            Err(RawError::InvalidHeader(_))
        ));
    }

    #[test]
    fn test_header_invalid_version() {
        let mut header = RawHeader::new_single::<TestVec3>();
        header.version = 99;
        assert!(matches!(
            header.validate::<TestVec3>(Some(1)),
            Err(RawError::InvalidHeader(_))
        ));
    }

    #[test]
    fn test_raw_wrapper() {
        let raw = Raw(TestVec3 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        });
        assert_eq!(raw.inner().x, 1.0);
        let inner = raw.into_inner();
        assert_eq!(inner.y, 2.0);
    }

    #[test]
    fn test_raw_slice_wrapper() {
        let raw = RawSlice(vec![TestPoint { x: 1, y: 2 }, TestPoint { x: 3, y: 4 }]);
        assert_eq!(raw.len(), 2);
        assert!(!raw.is_empty());
        assert_eq!(raw.inner()[0].x, 1);
        let inner = raw.into_inner();
        assert_eq!(inner[1].y, 4);
    }

    #[test]
    fn test_raw_tagged_wrapper() {
        let raw = RawTagged(TestVec3 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        });
        assert_eq!(raw.inner().z, 3.0);
        let inner = raw.into_inner();
        assert_eq!(inner.x, 1.0);
    }

    #[test]
    fn test_raw_slice_tagged_wrapper() {
        let raw = RawSliceTagged(vec![TestPoint { x: 1, y: 2 }]);
        assert_eq!(raw.len(), 1);
        assert!(!raw.is_empty());
        let inner = raw.into_inner();
        assert_eq!(inner[0], TestPoint { x: 1, y: 2 });
    }

    #[test]
    fn test_empty_slice() {
        let raw: RawSlice<TestPoint> = RawSlice(vec![]);
        assert!(raw.is_empty());
        assert_eq!(raw.len(), 0);
    }
}
