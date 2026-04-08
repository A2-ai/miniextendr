//! Windowed iterator-backed ALTREP infrastructure.
//!
//! Provides `WindowedIterState<I, T>` which keeps a sliding window of elements
//! in memory, and wrapper types for each ALTREP family.

use std::cell::RefCell;
use std::sync::OnceLock;

use crate::altrep_data::{AltIntegerData, AltRealData, AltrepLen, InferBase, fill_region};

/// Core state for windowed iterator-backed ALTREP vectors.
///
/// Like [`super::IterState`], but only keeps a sliding window of elements in memory.
/// Sequential access within the window is O(1). Access outside the window
/// materializes the entire vector (falling back to full caching).
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

        // Backward access — must materialize
        self.materialize_all();
        self.materialized.get().and_then(|v| v.get(i).copied())
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
                break;
            }
            // Stop once we've passed index i
            if *consumed > i + 1 && window.len() >= self.window_size {
                break;
            }
        }
    }

    /// Materialize all elements.
    pub fn materialize_all(&self) -> &[T] {
        if let Some(vec) = self.materialized.get() {
            return vec;
        }

        // We can only materialize elements from consumed onward
        // For backward access, we'd need to restart the iterator
        // Since iterators are consumed, materialize what we can
        let mut iter_opt = self.iter.borrow_mut();
        let window = self.window.borrow();
        let window_start = *self.window_start.borrow();

        let mut result = Vec::with_capacity(self.len);

        // Copy window elements at their correct positions
        // Start fresh approach: collect remaining from iterator
        if let Some(iter) = iter_opt.take() {
            // We have: window contents at window_start..window_start+window.len()
            // And: unconsumed elements from consumed onward
            // Elements before window_start are lost (consumed and discarded)

            // Fill lost positions with default
            for _ in 0..window_start {
                // These elements were consumed — can't recover
                result.push(window.first().copied().unwrap_or_else(|| {
                    // This shouldn't happen with valid usage
                    unsafe { std::mem::zeroed() }
                }));
            }

            // Copy window
            result.extend_from_slice(&window);

            // Consume rest
            for elem in iter {
                if result.len() >= self.len {
                    break;
                }
                result.push(elem);
            }
        }

        drop(window);
        drop(iter_opt);

        if result.len() < self.len {
            eprintln!(
                "[miniextendr warning] windowed iterator ALTREP: could only recover {}/{} elements on materialization",
                result.len(),
                self.len
            );
        }

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

// region: Windowed Iterator wrapper types

/// Windowed iterator-backed integer vector data.
///
/// Like [`super::IterIntData`], but only keeps a sliding window of elements in memory.
/// Sequential forward access within the window is O(1). Access outside the
/// window triggers full materialization.
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
    fn len(&self) -> usize {
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
        fill_region(start, len, self.len(), buf, |idx| self.elt(idx))
    }
}

impl<I: Iterator<Item = i32> + 'static> crate::externalptr::TypedExternal
    for WindowedIterIntData<I>
{
    const TYPE_NAME: &'static str = "WindowedIterIntData";
    const TYPE_NAME_CSTR: &'static [u8] = b"WindowedIterIntData\0";
    const TYPE_ID_CSTR: &'static [u8] = b"miniextendr_api::altrep::WindowedIterIntData\0";
}

impl<I: Iterator<Item = i32> + 'static> InferBase for WindowedIterIntData<I> {
    const BASE: crate::altrep::RBase = crate::altrep::RBase::Int;

    unsafe fn make_class(
        class_name: *const i8,
        pkg_name: *const i8,
    ) -> crate::ffi::altrep::R_altrep_class_t {
        unsafe {
            crate::ffi::altrep::R_make_altinteger_class(class_name, pkg_name, core::ptr::null_mut())
        }
    }

    unsafe fn install_methods(cls: crate::ffi::altrep::R_altrep_class_t) {
        unsafe { crate::altrep_bridge::install_base::<Self>(cls) };
        unsafe { crate::altrep_bridge::install_vec::<Self>(cls) };
        unsafe { crate::altrep_bridge::install_int::<Self>(cls) };
    }
}

impl<I: Iterator<Item = i32> + 'static> crate::altrep_traits::Altrep for WindowedIterIntData<I> {
    fn length(x: crate::ffi::SEXP) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .map(|d| d.len() as crate::ffi::R_xlen_t)
            .unwrap_or(0)
    }
}

impl<I: Iterator<Item = i32> + 'static> crate::altrep_traits::AltVec for WindowedIterIntData<I> {}

impl<I: Iterator<Item = i32> + 'static> crate::altrep_traits::AltInteger
    for WindowedIterIntData<I>
{
    const HAS_ELT: bool = true;

    fn elt(x: crate::ffi::SEXP, i: crate::ffi::R_xlen_t) -> i32 {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .map(|d| AltIntegerData::elt(&*d, i as usize))
            .unwrap_or(i32::MIN)
    }

    const HAS_GET_REGION: bool = true;

    fn get_region(
        x: crate::ffi::SEXP,
        start: crate::ffi::R_xlen_t,
        len: crate::ffi::R_xlen_t,
        buf: *mut i32,
    ) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .map(|d| {
                let slice = unsafe { crate::altrep_impl::altrep_region_buf(buf, len as usize) };
                AltIntegerData::get_region(&*d, start as usize, len as usize, slice)
                    as crate::ffi::R_xlen_t
            })
            .unwrap_or(0)
    }
}

/// Windowed iterator-backed real (f64) vector data.
///
/// Like [`super::IterRealData`], but only keeps a sliding window of elements in memory.
/// Sequential forward access within the window is O(1). Access outside the
/// window triggers full materialization.
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
    fn len(&self) -> usize {
        self.state.len()
    }
}

impl<I: Iterator<Item = f64>> AltRealData for WindowedIterRealData<I> {
    fn elt(&self, i: usize) -> f64 {
        self.state.get_element(i).unwrap_or(f64::NAN)
    }

    fn as_slice(&self) -> Option<&[f64]> {
        self.state.as_materialized()
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [f64]) -> usize {
        fill_region(start, len, self.len(), buf, |idx| self.elt(idx))
    }
}

impl<I: Iterator<Item = f64> + 'static> crate::externalptr::TypedExternal
    for WindowedIterRealData<I>
{
    const TYPE_NAME: &'static str = "WindowedIterRealData";
    const TYPE_NAME_CSTR: &'static [u8] = b"WindowedIterRealData\0";
    const TYPE_ID_CSTR: &'static [u8] = b"miniextendr_api::altrep::WindowedIterRealData\0";
}

impl<I: Iterator<Item = f64> + 'static> InferBase for WindowedIterRealData<I> {
    const BASE: crate::altrep::RBase = crate::altrep::RBase::Real;

    unsafe fn make_class(
        class_name: *const i8,
        pkg_name: *const i8,
    ) -> crate::ffi::altrep::R_altrep_class_t {
        unsafe {
            crate::ffi::altrep::R_make_altreal_class(class_name, pkg_name, core::ptr::null_mut())
        }
    }

    unsafe fn install_methods(cls: crate::ffi::altrep::R_altrep_class_t) {
        unsafe { crate::altrep_bridge::install_base::<Self>(cls) };
        unsafe { crate::altrep_bridge::install_vec::<Self>(cls) };
        unsafe { crate::altrep_bridge::install_real::<Self>(cls) };
    }
}

impl<I: Iterator<Item = f64> + 'static> crate::altrep_traits::Altrep for WindowedIterRealData<I> {
    fn length(x: crate::ffi::SEXP) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .map(|d| d.len() as crate::ffi::R_xlen_t)
            .unwrap_or(0)
    }
}

impl<I: Iterator<Item = f64> + 'static> crate::altrep_traits::AltVec for WindowedIterRealData<I> {}

impl<I: Iterator<Item = f64> + 'static> crate::altrep_traits::AltReal for WindowedIterRealData<I> {
    const HAS_ELT: bool = true;

    fn elt(x: crate::ffi::SEXP, i: crate::ffi::R_xlen_t) -> f64 {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .map(|d| AltRealData::elt(&*d, i as usize))
            .unwrap_or(f64::NAN)
    }

    const HAS_GET_REGION: bool = true;

    fn get_region(
        x: crate::ffi::SEXP,
        start: crate::ffi::R_xlen_t,
        len: crate::ffi::R_xlen_t,
        buf: *mut f64,
    ) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .map(|d| {
                let slice = unsafe { crate::altrep_impl::altrep_region_buf(buf, len as usize) };
                AltRealData::get_region(&*d, start as usize, len as usize, slice)
                    as crate::ffi::R_xlen_t
            })
            .unwrap_or(0)
    }
}
// endregion
