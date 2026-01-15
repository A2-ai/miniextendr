use std::ops::Range;

use crate::ffi::Rcomplex;
use crate::ffi::SEXP;

use super::{
    AltComplexData, AltIntegerData, AltLogicalData, AltRawData, AltRealData, AltStringData,
    AltrepDataptr, AltrepLen, AltrepSerialize, Logical, Sortedness,
};

// =============================================================================
// Helper macros to reduce repetition
// =============================================================================

/// Implement AltrepLen for Vec<$elem>
macro_rules! impl_len_vec {
    ($elem:ty) => {
        impl AltrepLen for Vec<$elem> {
            fn len(&self) -> usize {
                Vec::len(self)
            }
        }
    };
}

/// Implement AltrepLen for Box<[$elem]>
macro_rules! impl_len_boxed {
    ($elem:ty) => {
        impl AltrepLen for Box<[$elem]> {
            fn len(&self) -> usize {
                <[$elem]>::len(self)
            }
        }
    };
}

/// Implement AltrepLen for [$elem; N]
macro_rules! impl_len_array {
    ($elem:ty) => {
        impl<const N: usize> AltrepLen for [$elem; N] {
            fn len(&self) -> usize {
                N
            }
        }
    };
}

/// Implement AltrepLen for &[$elem]
macro_rules! impl_len_slice {
    ($elem:ty) => {
        impl AltrepLen for &[$elem] {
            fn len(&self) -> usize {
                <[$elem]>::len(self)
            }
        }
    };
}

/// Implement AltrepDataptr for Vec<$elem> (types with direct memory access)
macro_rules! impl_dataptr_vec {
    ($elem:ty) => {
        impl AltrepDataptr<$elem> for Vec<$elem> {
            fn dataptr(&mut self, _writable: bool) -> Option<*mut $elem> {
                Some(self.as_mut_ptr())
            }

            fn dataptr_or_null(&self) -> Option<*const $elem> {
                Some(self.as_ptr())
            }
        }
    };
}

/// Implement AltrepDataptr for Box<[$elem]> (types with direct memory access)
macro_rules! impl_dataptr_boxed {
    ($elem:ty) => {
        impl AltrepDataptr<$elem> for Box<[$elem]> {
            fn dataptr(&mut self, _writable: bool) -> Option<*mut $elem> {
                Some(self.as_mut_ptr())
            }

            fn dataptr_or_null(&self) -> Option<*const $elem> {
                Some(self.as_ptr())
            }
        }
    };
}

/// Implement AltrepSerialize for types that can be cloned and converted to/from R.
///
/// This serializes by converting to a native R vector, ensuring the data survives
/// even if the Rust package isn't loaded when unserializing.
macro_rules! impl_serialize {
    ($ty:ty) => {
        impl AltrepSerialize for $ty {
            fn serialized_state(&self) -> SEXP {
                use crate::into_r::IntoR;
                self.clone().into_sexp()
            }

            fn unserialize(state: SEXP) -> Option<Self> {
                use crate::from_r::TryFromSexp;
                <$ty>::try_from_sexp(state).ok()
            }
        }
    };
}

// =============================================================================
// AltrepSerialize implementations for Vec<T>
// =============================================================================

impl_serialize!(Vec<i32>);
impl_serialize!(Vec<f64>);
impl_serialize!(Vec<u8>);
impl_serialize!(Vec<bool>);
impl_serialize!(Vec<String>);
impl_serialize!(Vec<Option<String>>);
impl_serialize!(Vec<Rcomplex>);

// =============================================================================
// AltrepSerialize implementations for Box<[T]>
// =============================================================================

// Box<[T]> types don't have direct TryFromSexp implementations, so we manually
// implement serialization by converting to Vec and back.

impl AltrepSerialize for Box<[i32]> {
    fn serialized_state(&self) -> SEXP {
        use crate::into_r::IntoR;
        self.to_vec().into_sexp()
    }

    fn unserialize(state: SEXP) -> Option<Self> {
        use crate::from_r::TryFromSexp;
        Vec::<i32>::try_from_sexp(state)
            .ok()
            .map(|v| v.into_boxed_slice())
    }
}

impl AltrepSerialize for Box<[f64]> {
    fn serialized_state(&self) -> SEXP {
        use crate::into_r::IntoR;
        self.to_vec().into_sexp()
    }

    fn unserialize(state: SEXP) -> Option<Self> {
        use crate::from_r::TryFromSexp;
        Vec::<f64>::try_from_sexp(state)
            .ok()
            .map(|v| v.into_boxed_slice())
    }
}

impl AltrepSerialize for Box<[u8]> {
    fn serialized_state(&self) -> SEXP {
        use crate::into_r::IntoR;
        self.to_vec().into_sexp()
    }

    fn unserialize(state: SEXP) -> Option<Self> {
        use crate::from_r::TryFromSexp;
        Vec::<u8>::try_from_sexp(state)
            .ok()
            .map(|v| v.into_boxed_slice())
    }
}

impl AltrepSerialize for Box<[bool]> {
    fn serialized_state(&self) -> SEXP {
        use crate::into_r::IntoR;
        self.to_vec().into_sexp()
    }

    fn unserialize(state: SEXP) -> Option<Self> {
        use crate::from_r::TryFromSexp;
        Vec::<bool>::try_from_sexp(state)
            .ok()
            .map(|v| v.into_boxed_slice())
    }
}

impl AltrepSerialize for Box<[String]> {
    fn serialized_state(&self) -> SEXP {
        use crate::into_r::IntoR;
        self.to_vec().into_sexp()
    }

    fn unserialize(state: SEXP) -> Option<Self> {
        use crate::from_r::TryFromSexp;
        Vec::<String>::try_from_sexp(state)
            .ok()
            .map(|v| v.into_boxed_slice())
    }
}

impl AltrepSerialize for Box<[Rcomplex]> {
    fn serialized_state(&self) -> SEXP {
        use crate::into_r::IntoR;
        self.to_vec().into_sexp()
    }

    fn unserialize(state: SEXP) -> Option<Self> {
        use crate::from_r::TryFromSexp;
        Vec::<Rcomplex>::try_from_sexp(state)
            .ok()
            .map(|v| v.into_boxed_slice())
    }
}

// =============================================================================
// Built-in implementations for Vec<T>
// =============================================================================

impl_len_vec!(i32);

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
        if found { Some(min) } else { None }
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
        if found { Some(max) } else { None }
    }
}

impl_dataptr_vec!(i32);

impl_len_vec!(f64);

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
        if found { Some(min) } else { None }
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
        if found { Some(max) } else { None }
    }
}

impl_dataptr_vec!(f64);

impl_len_vec!(u8);

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

impl_dataptr_vec!(u8);

impl_len_vec!(String);

impl AltStringData for Vec<String> {
    fn elt(&self, i: usize) -> Option<&str> {
        Some(self[i].as_str())
    }

    fn no_na(&self) -> Option<bool> {
        Some(true) // String vectors don't have NA
    }
}

impl_len_vec!(Option<String>);

impl AltStringData for Vec<Option<String>> {
    fn elt(&self, i: usize) -> Option<&str> {
        self[i].as_deref()
    }

    fn no_na(&self) -> Option<bool> {
        Some(!self.iter().any(|x| x.is_none()))
    }
}

impl_len_vec!(bool);

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

impl_len_boxed!(i32);

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

impl_dataptr_boxed!(i32);

impl_len_boxed!(f64);

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
        if found { Some(min) } else { None }
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
        if found { Some(max) } else { None }
    }
}

impl_dataptr_boxed!(f64);

impl_len_boxed!(u8);

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

impl_dataptr_boxed!(u8);

impl_len_boxed!(bool);

impl AltLogicalData for Box<[bool]> {
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

impl_len_boxed!(String);

impl AltStringData for Box<[String]> {
    fn elt(&self, i: usize) -> Option<&str> {
        Some(self[i].as_str())
    }

    fn no_na(&self) -> Option<bool> {
        Some(true) // String can't be NA
    }
}

// =============================================================================
// AltrepSerialize implementations for Range types
// =============================================================================
// Ranges serialize to a 2-element integer/real vector [start, end].

impl AltrepSerialize for Range<i32> {
    fn serialized_state(&self) -> SEXP {
        use crate::into_r::IntoR;
        vec![self.start, self.end].into_sexp()
    }

    fn unserialize(state: SEXP) -> Option<Self> {
        use crate::from_r::TryFromSexp;
        let v = Vec::<i32>::try_from_sexp(state).ok()?;
        if v.len() == 2 { Some(v[0]..v[1]) } else { None }
    }
}

impl AltrepSerialize for Range<i64> {
    fn serialized_state(&self) -> SEXP {
        use crate::into_r::IntoR;
        // Serialize as f64 to preserve i64 range (R's doubles have 53-bit mantissa)
        vec![self.start as f64, self.end as f64].into_sexp()
    }

    fn unserialize(state: SEXP) -> Option<Self> {
        use crate::from_r::TryFromSexp;
        let v = Vec::<f64>::try_from_sexp(state).ok()?;
        if v.len() == 2 {
            Some((v[0] as i64)..(v[1] as i64))
        } else {
            None
        }
    }
}

impl AltrepSerialize for Range<f64> {
    fn serialized_state(&self) -> SEXP {
        use crate::into_r::IntoR;
        vec![self.start, self.end].into_sexp()
    }

    fn unserialize(state: SEXP) -> Option<Self> {
        use crate::from_r::TryFromSexp;
        let v = Vec::<f64>::try_from_sexp(state).ok()?;
        if v.len() == 2 { Some(v[0]..v[1]) } else { None }
    }
}

// =============================================================================
// Built-in implementations for Range types
// =============================================================================

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
        Some(Sortedness::Increasing)
    }

    fn no_na(&self) -> Option<bool> {
        // i32::MIN is NA_INTEGER in R. Check if the range contains it.
        // Range is [start, end), so i32::MIN is included iff start <= i32::MIN < end
        // Since start is the smallest value, we just check if start == i32::MIN
        let contains_na = self.start == i32::MIN && self.end > i32::MIN;
        Some(!contains_na)
    }

    fn sum(&self, na_rm: bool) -> Option<i64> {
        let n = AltrepLen::len(self) as i64;
        if n == 0 {
            return Some(0);
        }

        // Check if range contains NA (i32::MIN)
        let contains_na = self.start == i32::MIN && self.end > i32::MIN;
        if contains_na && !na_rm {
            return None; // NA propagates
        }

        // Sum of arithmetic sequence: n/2 * (first + last)
        // If na_rm and contains NA, exclude the first element (i32::MIN)
        if contains_na {
            // Exclude first element (NA), sum from start+1 to end-1
            let n_valid = n - 1;
            if n_valid == 0 {
                return Some(0);
            }
            let first = (self.start + 1) as i64;
            let last = (self.end - 1) as i64;
            Some(n_valid * (first + last) / 2)
        } else {
            let first = self.start as i64;
            let last = (self.end - 1) as i64;
            Some(n * (first + last) / 2)
        }
    }

    fn min(&self, na_rm: bool) -> Option<i32> {
        if AltrepLen::len(self) == 0 {
            return None;
        }

        // Check if first element is NA
        if self.start == i32::MIN {
            if na_rm {
                // Skip NA, return second element if it exists
                if self.end > self.start + 1 {
                    Some(self.start + 1)
                } else {
                    None // Only element was NA
                }
            } else {
                None // NA propagates
            }
        } else {
            Some(self.start)
        }
    }

    fn max(&self, na_rm: bool) -> Option<i32> {
        if AltrepLen::len(self) == 0 {
            return None;
        }

        // For increasing range, max is end-1 (last element)
        // Check if range contains NA (first element)
        let contains_na = self.start == i32::MIN && self.end > i32::MIN;
        if contains_na && !na_rm {
            return None; // NA propagates
        }

        // Max is always end-1 (last element), which is not NA
        // (NA would only be first element if start == i32::MIN)
        Some(self.end - 1)
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
        let val = self.start.saturating_add(i as i64);
        // Bounds check: return NA_INTEGER for values outside i32 range
        // Also, i32::MIN is the NA sentinel, so values equal to it are NA
        if val > i32::MAX as i64 || val <= i32::MIN as i64 {
            crate::altrep_traits::NA_INTEGER
        } else {
            val as i32
        }
    }

    fn is_sorted(&self) -> Option<Sortedness> {
        Some(Sortedness::Increasing)
    }

    fn no_na(&self) -> Option<bool> {
        // An element is NA if:
        // 1. It's outside valid i32 range (< i32::MIN or > i32::MAX)
        // 2. It equals i32::MIN (NA sentinel)
        //
        // For increasing range [start, end), elements range from start to end-1.
        // Range contains NA if:
        // - start <= i32::MIN as i64 (NA sentinel could be in range)
        // - OR end > i32::MAX as i64 + 1 (values exceed i32::MAX)
        // - OR start < i32::MIN as i64 (values below i32 range)
        let na_sentinel = i32::MIN as i64;
        let i32_max = i32::MAX as i64;

        // Check if NA sentinel is in [start, end)
        let contains_na_sentinel = self.start <= na_sentinel && self.end > na_sentinel;

        // Check if any values are outside valid i32 range
        // Valid range for ALTREP integers: (i32::MIN, i32::MAX] (excluding NA sentinel)
        let has_underflow = self.start < na_sentinel;
        let has_overflow = (self.end - 1) > i32_max;

        Some(!contains_na_sentinel && !has_underflow && !has_overflow)
    }

    fn sum(&self, na_rm: bool) -> Option<i64> {
        let n = AltrepLen::len(self) as i64;
        if n == 0 {
            return Some(0);
        }

        // Check if range contains any NA values
        let na_sentinel = i32::MIN as i64;
        let i32_max = i32::MAX as i64;
        let contains_na_sentinel = self.start <= na_sentinel && self.end > na_sentinel;
        let has_underflow = self.start < na_sentinel;
        let has_overflow = (self.end - 1) > i32_max;
        let has_na = contains_na_sentinel || has_underflow || has_overflow;

        if has_na && !na_rm {
            return None; // NA propagates
        }

        if has_na {
            // When na_rm=true, we need to exclude NA values
            // This is complex for ranges with out-of-bounds values, so let R compute
            return None;
        }

        let first = self.start;
        let last = self.end - 1;

        // Use checked arithmetic to detect overflow
        // Formula: n * (first + last) / 2
        let sum_endpoints = first.checked_add(last)?;
        let product = n.checked_mul(sum_endpoints)?;
        Some(product / 2)
    }

    fn min(&self, na_rm: bool) -> Option<i32> {
        if AltrepLen::len(self) == 0 {
            return None;
        }

        let na_sentinel = i32::MIN as i64;
        let i32_max = i32::MAX as i64;

        // Check for NA conditions
        let contains_na_sentinel = self.start <= na_sentinel && self.end > na_sentinel;
        let has_underflow = self.start < na_sentinel;
        let has_overflow = (self.end - 1) > i32_max;
        let has_na = contains_na_sentinel || has_underflow || has_overflow;

        if has_na && !na_rm {
            return None; // NA propagates
        }

        if has_na {
            // Complex case: need to find first non-NA value
            // Let R compute this
            return None;
        }

        // No NA, return start (which is within valid i32 range)
        Some(self.start as i32)
    }

    fn max(&self, na_rm: bool) -> Option<i32> {
        if AltrepLen::len(self) == 0 {
            return None;
        }

        let na_sentinel = i32::MIN as i64;
        let i32_max = i32::MAX as i64;

        // Check for NA conditions
        let contains_na_sentinel = self.start <= na_sentinel && self.end > na_sentinel;
        let has_underflow = self.start < na_sentinel;
        let has_overflow = (self.end - 1) > i32_max;
        let has_na = contains_na_sentinel || has_underflow || has_overflow;

        if has_na && !na_rm {
            return None; // NA propagates
        }

        if has_na {
            // Complex case: need to find last non-NA value
            // Let R compute this
            return None;
        }

        // No NA, return end-1 (which is within valid i32 range)
        Some((self.end - 1) as i32)
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
        Some(Sortedness::Increasing)
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

impl_len_slice!(i32);

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
                Some(
                    self.iter()
                        .filter(|&&x| x != i32::MIN)
                        .map(|&x| x as i64)
                        .sum(),
                )
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

impl_len_slice!(f64);

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

impl_len_slice!(u8);

impl AltRawData for &[u8] {
    fn elt(&self, i: usize) -> u8 {
        self[i]
    }

    fn as_slice(&self) -> Option<&[u8]> {
        Some(self)
    }
}

impl_len_slice!(bool);

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

impl_len_slice!(String);

impl AltStringData for &[String] {
    fn elt(&self, i: usize) -> Option<&str> {
        Some(self[i].as_str())
    }
}

impl_len_slice!(&str);

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

impl_len_array!(i32);

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

impl_len_array!(f64);

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

impl_len_array!(bool);

impl<const N: usize> AltLogicalData for [bool; N] {
    fn elt(&self, i: usize) -> Logical {
        Logical::from_bool(self[i])
    }

    fn no_na(&self) -> Option<bool> {
        Some(true) // bool arrays can't have NA
    }
}

impl_len_array!(u8);

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

impl_len_array!(String);

impl<const N: usize> AltStringData for [String; N] {
    fn elt(&self, i: usize) -> Option<&str> {
        Some(self[i].as_str())
    }
}

// =============================================================================
// Built-in implementations for Vec<Rcomplex> (complex numbers)
// =============================================================================

impl_len_vec!(Rcomplex);

impl AltComplexData for Vec<Rcomplex> {
    fn elt(&self, i: usize) -> Rcomplex {
        self[i]
    }

    fn as_slice(&self) -> Option<&[Rcomplex]> {
        Some(self.as_slice())
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [Rcomplex]) -> usize {
        let end = (start + len).min(self.len());
        let actual_len = end.saturating_sub(start);
        if actual_len > 0 {
            buf[..actual_len].copy_from_slice(&self[start..end]);
        }
        actual_len
    }
}

impl_dataptr_vec!(Rcomplex);

// =============================================================================
// Built-in implementations for Box<[Rcomplex]>
// =============================================================================

impl_len_boxed!(Rcomplex);

impl AltComplexData for Box<[Rcomplex]> {
    fn elt(&self, i: usize) -> Rcomplex {
        self[i]
    }

    fn as_slice(&self) -> Option<&[Rcomplex]> {
        Some(self.as_ref())
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [Rcomplex]) -> usize {
        let end = (start + len).min(self.len());
        let actual_len = end.saturating_sub(start);
        if actual_len > 0 {
            buf[..actual_len].copy_from_slice(&self[start..end]);
        }
        actual_len
    }
}

impl_dataptr_boxed!(Rcomplex);

// =============================================================================
// Built-in implementations for [Rcomplex; N] (complex arrays)
// =============================================================================

impl_len_array!(Rcomplex);

impl<const N: usize> AltComplexData for [Rcomplex; N] {
    fn elt(&self, i: usize) -> Rcomplex {
        self[i]
    }

    fn as_slice(&self) -> Option<&[Rcomplex]> {
        Some(self.as_slice())
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [Rcomplex]) -> usize {
        let end = (start + len).min(N);
        let actual_len = end.saturating_sub(start);
        if actual_len > 0 {
            buf[..actual_len].copy_from_slice(&self[start..end]);
        }
        actual_len
    }
}

// =============================================================================
// Low-level ALTREP trait implementations
// =============================================================================
//
// The low-level trait impls (Altrep, AltVec, Alt*, InferBase) for builtin types
// are located in altrep_impl.rs. This is because the impl_alt*_from_data! macros
// are defined there and need to be in the same module.
//
// See altrep_impl.rs for:
// - Vec<i32>, Vec<f64>, Vec<bool>, Vec<u8>, Vec<String>, Vec<Rcomplex>
// - Box<[i32]>, Box<[f64]>, Box<[bool]>, Box<[u8]>, Box<[String]>, Box<[Rcomplex]>
// - Range<i32>, Range<i64>, Range<f64>
