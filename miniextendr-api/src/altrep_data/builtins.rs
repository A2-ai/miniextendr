use std::ops::Range;

use super::{
    AltIntegerData, AltLogicalData, AltRawData, AltRealData, AltStringData, AltrepDataptr,
    AltrepLen, Logical, Sortedness,
};

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
        let val = self.start.saturating_add(i as i64);
        // Bounds check: return NA_INTEGER for values outside i32 range
        if val > i32::MAX as i64 || val < i32::MIN as i64 {
            crate::altrep_traits::NA_INTEGER
        } else {
            val as i32
        }
    }

    fn is_sorted(&self) -> Option<Sortedness> {
        Some(Sortedness::Increasing)
    }

    fn no_na(&self) -> Option<bool> {
        // May contain NA if range exceeds i32 bounds
        let start_ok = self.start >= i32::MIN as i64 && self.start <= i32::MAX as i64;
        let end_ok = self.end >= i32::MIN as i64 && self.end <= i32::MAX as i64 + 1;
        Some(start_ok && end_ok)
    }

    fn sum(&self, _na_rm: bool) -> Option<i64> {
        let n = AltrepLen::len(self) as i64;
        if n == 0 {
            return Some(0);
        }
        let first = self.start;
        let last = self.end - 1;

        // Use checked arithmetic to detect overflow
        // Formula: n * (first + last) / 2
        let sum_endpoints = first.checked_add(last)?;
        let product = n.checked_mul(sum_endpoints)?;
        Some(product / 2)
    }

    fn min(&self, _na_rm: bool) -> Option<i32> {
        if AltrepLen::len(self) > 0 {
            let val = self.start;
            if val > i32::MAX as i64 || val < i32::MIN as i64 {
                None // Out of range, let R compute
            } else {
                Some(val as i32)
            }
        } else {
            None
        }
    }

    fn max(&self, _na_rm: bool) -> Option<i32> {
        if AltrepLen::len(self) > 0 {
            let val = self.end - 1;
            if val > i32::MAX as i64 || val < i32::MIN as i64 {
                None // Out of range, let R compute
            } else {
                Some(val as i32)
            }
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
