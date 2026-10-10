//! Registered native iterator containers built on the shared prefix cache.

use super::IterState;
use crate::altrep_data::{
    AltComplexData, AltIntegerData, AltRawData, AltRealData, AltrepDataptr, AltrepLen,
    AltrepSerialize,
};
use crate::{IntoR, Rcomplex, SEXP, TryFromSexp, TypedExternal};

/// An owned iterator with repeatable random access and built-in ALTREP registration.
///
/// Supported element types are `i32`, `f64`, `u8` and `Rcomplex`. Return it
/// with `Altrep::from_exact_iter(iter)` or
/// `Altrep::from_iter_with_len(iter, len)`; no custom derive or class is needed.
///
/// The iterator is consumed only through the largest requested index. Its
/// prefix is cached, so revisiting an earlier element never reruns the iterator.
/// A contiguous-pointer request finishes the declared prefix and exposes its
/// Rust-owned buffer. Writes through that buffer affect reads and serialization.
///
/// The declared length is a limit: extra elements are never requested. Ending
/// before that length is an error, including for an incorrect `ExactSizeIterator`.
/// Serialization evaluates the remaining prefix and restores owned values.
/// Duplication follows R's default policy, as with the other built-in containers.
///
/// The iterator must own its captures (`'static`). It need not be `Send`;
/// construct and convert this container on the R thread.
pub struct AltrepIterator<T> {
    state: IterState<Box<dyn Iterator<Item = T>>, T>,
}

impl<T: 'static> AltrepIterator<T> {
    /// Wrap a finite prefix without advancing the iterator.
    ///
    /// No element beyond `len` is requested, even when the source is infinite.
    /// An early end raises an error when the missing element is requested.
    pub fn from_iter(iter: impl Iterator<Item = T> + 'static, len: usize) -> Self {
        let iter: Box<dyn Iterator<Item = T>> = Box::new(iter.fuse());
        Self {
            state: if len == 0 {
                // Do not probe a zero-length prefix or retain unused captures.
                IterState::from_materialized(Vec::new())
            } else {
                IterState::new(iter, len)
            },
        }
    }

    /// Infer the declared length without advancing an exact-size iterator.
    pub fn from_exact_iter(iter: impl ExactSizeIterator<Item = T> + 'static) -> Self {
        let len = iter.len();
        Self::from_iter(iter, len)
    }
}

impl<T: Copy> AltrepIterator<T> {
    fn element(&self, i: usize) -> T {
        self.state.get_element(i).unwrap_or_else(|| {
            panic!(
                "ALTREP iterator ended before element {} of its declared length {}",
                i + 1,
                self.state.len()
            )
        })
    }

    fn materialized(&self) -> &[T] {
        if !self.state.is_empty() {
            self.element(self.state.len() - 1);
        }
        self.state.as_materialized().unwrap()
    }
}

impl<T> AltrepLen for AltrepIterator<T> {
    fn len(&self) -> usize {
        self.state.len()
    }
}

impl<T: 'static> TypedExternal for AltrepIterator<T> {
    const TYPE_NAME: &'static str = "AltrepIterator";
    const TYPE_NAME_CSTR: &'static [u8] = b"AltrepIterator\0";
    const TYPE_ID_CSTR: &'static [u8] = b"miniextendr::AltrepIterator\0";
}

macro_rules! impl_native_iterator {
    ($elem:ty, $family:ident) => {
        impl $family for AltrepIterator<$elem> {
            fn elt(&self, i: usize) -> $elem {
                self.element(i)
            }

            fn as_slice(&self) -> Option<&[$elem]> {
                self.state.as_materialized()
            }
        }

        impl AltrepDataptr<$elem> for AltrepIterator<$elem> {
            fn dataptr(&mut self, _writable: bool) -> Option<*mut $elem> {
                self.materialized();
                self.state.as_materialized_mut().map(<[$elem]>::as_mut_ptr)
            }

            fn dataptr_or_null(&self) -> Option<*const $elem> {
                self.state.as_materialized().map(<[$elem]>::as_ptr)
            }
        }

        impl AltrepSerialize for AltrepIterator<$elem> {
            fn serialized_state(&self) -> SEXP {
                self.materialized().into_sexp()
            }

            fn unserialize(state: SEXP) -> Option<Self> {
                Vec::<$elem>::try_from_sexp(state).ok().map(|values| Self {
                    state: IterState::from_materialized(values),
                })
            }
        }
    };
}

impl_native_iterator!(i32, AltIntegerData);
impl_native_iterator!(f64, AltRealData);
impl_native_iterator!(u8, AltRawData);
impl_native_iterator!(Rcomplex, AltComplexData);

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic::{AssertUnwindSafe, catch_unwind};

    #[test]
    fn early_exhaustion_keeps_cached_elements_and_never_publishes_a_short_buffer() {
        let data = AltrepIterator::from_iter(0..2, 3);
        assert_eq!(data.element(1), 1);
        for _ in 0..2 {
            assert!(catch_unwind(AssertUnwindSafe(|| data.materialized())).is_err());
            assert!(data.state.as_materialized().is_none());
            assert_eq!(data.element(0), 0);
        }
    }

    #[test]
    fn exhaustion_is_final_even_for_a_non_fused_source() {
        use std::cell::Cell;
        use std::rc::Rc;

        let calls = Rc::new(Cell::new(0));
        let count = calls.clone();
        let iter = std::iter::from_fn(move || {
            let next = count.get();
            count.set(next + 1);
            match next {
                0 => Some(10),
                1 => None,
                _ => Some(20),
            }
        });
        let data = AltrepIterator::from_iter(iter, 2);
        assert_eq!(data.element(0), 10);
        for _ in 0..2 {
            assert!(catch_unwind(AssertUnwindSafe(|| data.element(1))).is_err());
            assert_eq!(calls.get(), 2);
        }
    }
}
