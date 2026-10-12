//! Windowed iterator-backed ALTREP data adaptors.
//!
//! Provides `WindowedIterState<I, T>` which keeps a sliding window of elements
//! in memory, and data-adaptor types for the integer/real ALTREP families.
//!
//! See the iterator-adaptor section in the [`altrep_data`](crate::altrep_data)
//! module docs for how to expose
//! these adaptors to R (wrap in a `#[derive(Altrep*)]` + `#[altrep(manual)]`
//! struct).

use std::cell::RefCell;
use std::sync::OnceLock;

use crate::altrep_data::{AltIntegerData, AltRealData, AltrepLen, fill_region};

// region: WindowedIterState

/// Core state for windowed iterator-backed ALTREP vectors.
///
/// Like [`super::IterState`], but only keeps a sliding window of elements in memory.
/// Access within the window is O(1), and reading past it slides the window
/// forward. The iterator cannot be rewound, so reading an element that has left
/// the window is an error. Use [`super::IterState`] when R may read elements
/// again or out of order.
///
/// This is useful for large iterators where only a small region is accessed
/// at a time (e.g., streaming data processed in order).
///
/// # Type Parameters
///
/// - `I`: The iterator type
/// - `T`: The element type produced by the iterator
pub struct WindowedIterState<I, T> {
    len: usize,
    iter: RefCell<Option<I>>,
    consumed: RefCell<usize>,
    window: RefCell<Vec<T>>,
    window_start: RefCell<usize>,
    window_size: usize,
    materialized: OnceLock<Vec<T>>,
}

impl<I, T> WindowedIterState<I, T>
where
    I: Iterator<Item = T>,
    T: Copy,
{
    /// Create a new windowed iterator state.
    pub fn new(iter: I, len: usize, window_size: usize) -> Self {
        let window_size = window_size.max(1);
        Self {
            len,
            iter: RefCell::new(Some(iter)),
            consumed: RefCell::new(0),
            window: RefCell::new(Vec::with_capacity(window_size)),
            window_start: RefCell::new(0),
            window_size,
            materialized: OnceLock::new(),
        }
    }

    /// Get element at index `i`.
    pub fn get_element(&self, i: usize) -> Option<T> {
        if i >= self.len {
            return None;
        }

        // Check materialized first
        if let Some(vec) = self.materialized.get() {
            return vec.get(i).copied();
        }

        let window_start = *self.window_start.borrow();
        let window = self.window.borrow();

        // Check if in current window
        if i >= window_start && i < window_start + window.len() {
            return Some(window[i - window_start]);
        }
        drop(window);

        // Check if we can advance to reach this index
        let consumed = *self.consumed.borrow();
        if i >= consumed {
            // Forward access — advance iterator to fill window containing i
            self.advance_to(i);
            let window = self.window.borrow();
            let window_start = *self.window_start.borrow();
            if i >= window_start && i < window_start + window.len() {
                return Some(window[i - window_start]);
            }
            return None; // iterator exhausted
        }

        // Backward access: the element was discarded when the window moved on.
        panic!(
            "windowed iterator ALTREP: element {} left the window, which now starts at element {}; \
             a windowed vector can only be read forward",
            i + 1,
            *self.window_start.borrow() + 1
        );
    }

    /// Advance the iterator to fill a window containing index `i`.
    fn advance_to(&self, i: usize) {
        let mut iter_opt = self.iter.borrow_mut();
        let iter = match iter_opt.as_mut() {
            Some(it) => it,
            None => return,
        };

        let mut consumed = self.consumed.borrow_mut();
        let mut window = self.window.borrow_mut();
        let mut window_start = self.window_start.borrow_mut();

        // Skip elements before the target window
        let target_window_start = if i >= self.window_size {
            i - self.window_size + 1
        } else {
            0
        };

        // Skip elements we need to discard
        while *consumed < target_window_start {
            if iter.next().is_some() {
                *consumed += 1;
            } else {
                // The skipped elements are gone too; keep the window adjacent to `consumed`.
                iter_opt.take();
                window.clear();
                *window_start = *consumed;
                return;
            }
        }

        // Fill the window
        window.clear();
        *window_start = *consumed;

        while window.len() < self.window_size && *consumed < self.len {
            if let Some(elem) = iter.next() {
                window.push(elem);
                *consumed += 1;
            } else {
                // Never poll an exhausted iterator again: it need not be fused.
                iter_opt.take();
                break;
            }
            // Stop once we've passed index i
            if *consumed > i + 1 && window.len() >= self.window_size {
                break;
            }
        }
    }

    /// Materialize all elements.
    ///
    /// # Panics
    ///
    /// If elements have already left the window, or if the iterator ends before
    /// the declared length.
    pub fn materialize_all(&self) -> &[T] {
        if let Some(vec) = self.materialized.get() {
            return vec;
        }

        let mut iter_opt = self.iter.borrow_mut();
        let mut window = self.window.borrow_mut();
        let mut consumed = self.consumed.borrow_mut();
        assert!(
            *consumed == window.len(),
            "windowed iterator ALTREP: the first {} elements left the window and cannot be read again",
            *consumed - window.len()
        );

        // Nothing was discarded, so the window holds every element read so far.
        // Grow it in place: on early exhaustion those elements stay readable.
        if let Some(iter) = iter_opt.take() {
            window.extend(iter.take(self.len - *consumed));
            *consumed = window.len();
        }
        assert!(
            window.len() == self.len,
            "ALTREP iterator ended before element {} of its declared length {}",
            window.len() + 1,
            self.len
        );

        let result = std::mem::take(&mut *window);
        drop(consumed);
        drop(window);
        drop(iter_opt);
        self.materialized.get_or_init(|| result)
    }

    /// Get materialized slice if available.
    pub fn as_materialized(&self) -> Option<&[T]> {
        self.materialized.get().map(|v| v.as_slice())
    }

    /// Get the length.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<I, T> WindowedIterState<I, T>
where
    I: ExactSizeIterator<Item = T>,
    T: Copy,
{
    /// Create from an ExactSizeIterator.
    pub fn from_exact_size(iter: I, window_size: usize) -> Self {
        let len = iter.len();
        Self::new(iter, len, window_size)
    }
}
// endregion

// region: Windowed Iterator data adaptors

/// Windowed iterator-backed integer vector data adaptor.
///
/// Like [`super::IterIntData`], but only keeps a sliding window of elements in memory.
/// Forward reads are O(1); reading an element that has left the window is an
/// error (see [`WindowedIterState`]).
pub struct WindowedIterIntData<I: Iterator<Item = i32>> {
    state: WindowedIterState<I, i32>,
}

impl<I: Iterator<Item = i32>> WindowedIterIntData<I> {
    /// Create from an iterator with explicit length and window size.
    pub fn from_iter(iter: I, len: usize, window_size: usize) -> Self {
        Self {
            state: WindowedIterState::new(iter, len, window_size),
        }
    }
}

impl<I: ExactSizeIterator<Item = i32>> WindowedIterIntData<I> {
    /// Create from an ExactSizeIterator with window size (length auto-detected).
    pub fn from_exact_iter(iter: I, window_size: usize) -> Self {
        Self {
            state: WindowedIterState::from_exact_size(iter, window_size),
        }
    }
}

impl<I: Iterator<Item = i32>> AltrepLen for WindowedIterIntData<I> {
    fn altrep_len(&self) -> usize {
        self.state.len()
    }
}

impl<I: Iterator<Item = i32>> AltIntegerData for WindowedIterIntData<I> {
    fn elt(&self, i: usize) -> i32 {
        self.state
            .get_element(i)
            .unwrap_or(crate::altrep_traits::NA_INTEGER)
    }

    fn as_slice(&self) -> Option<&[i32]> {
        self.state.as_materialized()
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [i32]) -> usize {
        fill_region(start, len, self.altrep_len(), buf, |idx| self.elt(idx))
    }
}

/// Windowed iterator-backed real (f64) vector data adaptor.
///
/// Like [`super::IterRealData`], but only keeps a sliding window of elements in memory.
/// Forward reads are O(1); reading an element that has left the window is an
/// error (see [`WindowedIterState`]).
pub struct WindowedIterRealData<I: Iterator<Item = f64>> {
    state: WindowedIterState<I, f64>,
}

impl<I: Iterator<Item = f64>> WindowedIterRealData<I> {
    /// Create from an iterator with explicit length and window size.
    pub fn from_iter(iter: I, len: usize, window_size: usize) -> Self {
        Self {
            state: WindowedIterState::new(iter, len, window_size),
        }
    }
}

impl<I: ExactSizeIterator<Item = f64>> WindowedIterRealData<I> {
    /// Create from an ExactSizeIterator with window size (length auto-detected).
    pub fn from_exact_iter(iter: I, window_size: usize) -> Self {
        Self {
            state: WindowedIterState::from_exact_size(iter, window_size),
        }
    }
}

impl<I: Iterator<Item = f64>> AltrepLen for WindowedIterRealData<I> {
    fn altrep_len(&self) -> usize {
        self.state.len()
    }
}

impl<I: Iterator<Item = f64>> AltRealData for WindowedIterRealData<I> {
    fn elt(&self, i: usize) -> f64 {
        self.state
            .get_element(i)
            .unwrap_or(crate::altrep_traits::NA_REAL)
    }

    fn as_slice(&self) -> Option<&[f64]> {
        self.state.as_materialized()
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [f64]) -> usize {
        fill_region(start, len, self.altrep_len(), buf, |idx| self.elt(idx))
    }
}

// endregion

#[cfg(test)]
mod tests {
    use super::WindowedIterState;
    use std::panic::{AssertUnwindSafe, catch_unwind};

    #[test]
    fn a_read_behind_the_window_is_an_error() {
        let state = WindowedIterState::new(0..10, 10, 3);
        assert_eq!(state.get_element(0), Some(0));
        assert_eq!(state.get_element(5), Some(5));
        assert!(catch_unwind(AssertUnwindSafe(|| state.get_element(1))).is_err());
        assert_eq!(state.get_element(4), Some(4));
        assert_eq!(state.get_element(6), Some(6));
    }

    #[test]
    fn materialization_before_the_window_moves_reads_exactly_len() {
        let source = || (0..4).chain(std::iter::once_with(|| panic!("read beyond the length")));
        let state = WindowedIterState::new(source(), 4, 2);
        assert_eq!(state.materialize_all(), [0, 1, 2, 3]);

        let state = WindowedIterState::new(source(), 4, 2);
        assert_eq!(state.get_element(1), Some(1));
        assert_eq!(state.materialize_all(), [0, 1, 2, 3]);
        assert_eq!(state.get_element(3), Some(3));
    }

    #[test]
    fn materialization_after_the_window_moves_is_an_error() {
        let state = WindowedIterState::new(0..6, 6, 2);
        assert_eq!(state.get_element(4), Some(4));
        assert!(catch_unwind(AssertUnwindSafe(|| state.materialize_all())).is_err());
        assert!(state.as_materialized().is_none());
        assert_eq!(state.get_element(4), Some(4));
        assert_eq!(state.get_element(5), Some(5));
    }

    #[test]
    fn early_exhaustion_never_publishes_a_partial_buffer() {
        let state = WindowedIterState::new(0..2, 3, 4);
        for _ in 0..2 {
            assert!(catch_unwind(AssertUnwindSafe(|| state.materialize_all())).is_err());
            assert!(state.as_materialized().is_none());
            assert_eq!(state.get_element(0), Some(0));
            assert_eq!(state.get_element(1), Some(1));
            assert_eq!(state.get_element(2), None);
        }
    }

    #[test]
    fn an_exhausted_source_is_never_polled_again() {
        // Yields 0, then None, then 1 if polled again (not fused).
        struct Flaky(u8);
        impl Iterator for Flaky {
            type Item = i32;
            fn next(&mut self) -> Option<i32> {
                self.0 += 1;
                match self.0 {
                    1 => Some(0),
                    3 => Some(1),
                    _ => None,
                }
            }
        }

        // Exhausted while filling the window.
        let state = WindowedIterState::new(Flaky(0), 3, 2);
        assert_eq!(state.get_element(0), Some(0));
        assert_eq!(state.get_element(1), None);
        assert_eq!(state.get_element(1), None);

        // Exhausted while skipping to the window.
        let state = WindowedIterState::new(Flaky(0), 5, 1);
        assert_eq!(state.get_element(2), None);
        assert_eq!(state.get_element(2), None);
        assert!(catch_unwind(AssertUnwindSafe(|| state.get_element(0))).is_err());
    }
}
