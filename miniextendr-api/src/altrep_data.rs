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
        for i in 0..actual_len {
            buf[i] = self.elt(start + i);
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
        for i in 0..actual_len {
            buf[i] = self.elt(start + i);
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
        for i in 0..actual_len {
            buf[i] = self.elt(start + i).to_r_int();
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
        for i in 0..actual_len {
            buf[i] = self.elt(start + i);
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
        for i in 0..actual_len {
            buf[i] = self.elt(start + i);
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

/// Marker trait for types that can provide a mutable data pointer.
///
/// This is separate from element access because some ALTREP types
/// compute elements on-the-fly but can materialize to a buffer.
pub trait AltrepDataptr<T> {
    /// Get a mutable pointer to the underlying data.
    ///
    /// If `writable` is true, R may modify the data.
    /// Return `None` if data cannot be accessed as a contiguous buffer.
    fn dataptr(&mut self, writable: bool) -> Option<*mut T>;

    /// Get a read-only pointer without forcing materialization.
    ///
    /// Return `None` if data is not already materialized.
    fn dataptr_or_null(&self) -> Option<*const T> {
        None
    }
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

impl<'a> AltrepLen for &[&'a str] {
    fn len(&self) -> usize {
        <[&str]>::len(self)
    }
}

impl<'a> AltStringData for &[&'a str] {
    fn elt(&self, i: usize) -> Option<&str> {
        Some(self[i])
    }
}
