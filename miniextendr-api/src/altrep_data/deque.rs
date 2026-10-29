//! ALTREP storage for native-element ring buffers.
//!
//! Element and region reads preserve the split buffer. A contiguous pointer is
//! exposed only after `VecDeque::make_contiguous`, without converting the owner
//! to a Vec or copying into an R vector.

use std::collections::VecDeque;

use super::{
    AltComplexData, AltIntegerData, AltRawData, AltRealData, AltrepDataptr, AltrepLen,
    AltrepSerialize,
};
use crate::{IntoR, Rcomplex, SEXP, TryFromSexp};

impl<T> AltrepLen for VecDeque<T> {
    fn altrep_len(&self) -> usize {
        VecDeque::len(self)
    }
}

/// Borrow the complete logical sequence only when it occupies one segment.
fn contiguous_slice<T>(values: &VecDeque<T>) -> Option<&[T]> {
    let (front, back) = values.as_slices();
    if back.is_empty() {
        Some(front)
    } else if front.is_empty() {
        Some(back)
    } else {
        None
    }
}

/// Copy at most two contiguous regions, with no allocation or rearrangement.
fn copy_region<T: Copy>(values: &VecDeque<T>, start: usize, len: usize, out: &mut [T]) -> usize {
    let n = len.min(out.len()).min(values.len().saturating_sub(start));
    if n == 0 {
        return 0;
    }
    let (front, back) = values.as_slices();
    let first = n.min(front.len().saturating_sub(start));
    if first != 0 {
        out[..first].copy_from_slice(&front[start..start + first]);
    }
    if first != n {
        let offset = start.saturating_sub(front.len());
        out[first..n].copy_from_slice(&back[offset..offset + n - first]);
    }
    n
}

macro_rules! impl_native_deque {
    ($elem:ty, $family:ident) => {
        impl $family for VecDeque<$elem> {
            fn elt(&self, i: usize) -> $elem {
                self[i]
            }

            fn as_slice(&self) -> Option<&[$elem]> {
                contiguous_slice(self)
            }

            fn get_region(&self, start: usize, len: usize, out: &mut [$elem]) -> usize {
                copy_region(self, start, len, out)
            }
        }

        impl AltrepDataptr<$elem> for VecDeque<$elem> {
            fn dataptr(&mut self, _writable: bool) -> Option<*mut $elem> {
                Some(self.make_contiguous().as_mut_ptr())
            }

            fn dataptr_or_null(&self) -> Option<*const $elem> {
                contiguous_slice(self).map(<[$elem]>::as_ptr)
            }
        }

        impl AltrepSerialize for VecDeque<$elem> {
            fn serialized_state(&self) -> SEXP {
                self.clone().into_sexp()
            }

            fn unserialize(state: SEXP) -> Option<Self> {
                Self::try_from_sexp(state).ok()
            }
        }
    };
}

impl_native_deque!(i32, AltIntegerData);
impl_native_deque!(f64, AltRealData);
impl_native_deque!(u8, AltRawData);
impl_native_deque!(Rcomplex, AltComplexData);

#[cfg(test)]
mod tests;
