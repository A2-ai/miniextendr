//! Streaming ALTREP data backed by chunk-cached reader closures.
//!
//! These types provide ALTREP vectors where elements are loaded on-demand
//! from a reader function in fixed-size chunks. Chunks are cached for
//! repeated access within the same region.

use std::cell::RefCell;
use std::collections::BTreeMap;

use super::{AltIntegerData, AltRealData, AltrepLen, InferBase};

// region: StreamingRealData

/// Streaming ALTREP for real (f64) vectors.
///
/// Elements are loaded on-demand via a reader closure in fixed-size chunks.
/// Chunks are cached in a `BTreeMap` for repeated access.
///
/// # Reader Contract
///
/// The reader `F(start, buf) -> count` fills `buf` with elements starting
/// at index `start` and returns the number of elements actually written.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::altrep_data::StreamingRealData;
///
/// let data = StreamingRealData::new(1000, 64, |start, buf| {
///     let count = buf.len().min(1000 - start);
///     for (i, slot) in buf[..count].iter_mut().enumerate() {
///         *slot = (start + i) as f64 * 0.1;
///     }
///     count
/// });
/// ```
pub struct StreamingRealData<F: Fn(usize, &mut [f64]) -> usize> {
    len: usize,
    reader: F,
    cache: RefCell<BTreeMap<usize, Vec<f64>>>,
    chunk_size: usize,
}

impl<F: Fn(usize, &mut [f64]) -> usize> StreamingRealData<F> {
    /// Create a new streaming real data source.
    ///
    /// - `len`: total number of elements
    /// - `chunk_size`: number of elements per cache chunk
    /// - `reader`: closure that fills a buffer starting at a given index
    pub fn new(len: usize, chunk_size: usize, reader: F) -> Self {
        Self {
            len,
            reader,
            cache: RefCell::new(BTreeMap::new()),
            chunk_size: chunk_size.max(1),
        }
    }

    /// Load a chunk into the cache if not already present.
    fn ensure_chunk(&self, chunk_idx: usize) {
        let mut cache = self.cache.borrow_mut();
        if cache.contains_key(&chunk_idx) {
            return;
        }
        let start = chunk_idx * self.chunk_size;
        let count = self.chunk_size.min(self.len.saturating_sub(start));
        if count == 0 {
            return;
        }
        let mut buf = vec![0.0f64; count];
        let written = (self.reader)(start, &mut buf);
        buf.truncate(written);
        cache.insert(chunk_idx, buf);
    }
}

impl<F: Fn(usize, &mut [f64]) -> usize> AltrepLen for StreamingRealData<F> {
    fn len(&self) -> usize {
        self.len
    }
}

impl<F: Fn(usize, &mut [f64]) -> usize> AltRealData for StreamingRealData<F> {
    fn elt(&self, i: usize) -> f64 {
        if i >= self.len {
            return f64::NAN;
        }
        let chunk_idx = i / self.chunk_size;
        self.ensure_chunk(chunk_idx);
        let cache = self.cache.borrow();
        let offset = i % self.chunk_size;
        cache
            .get(&chunk_idx)
            .and_then(|chunk| chunk.get(offset).copied())
            .unwrap_or(f64::NAN)
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [f64]) -> usize {
        let count = len.min(self.len.saturating_sub(start)).min(buf.len());
        if count == 0 {
            return 0;
        }
        (self.reader)(start, &mut buf[..count])
    }
}

impl<F: Fn(usize, &mut [f64]) -> usize + 'static> crate::externalptr::TypedExternal
    for StreamingRealData<F>
{
    const TYPE_NAME: &'static str = "StreamingRealData";
    const TYPE_NAME_CSTR: &'static [u8] = b"StreamingRealData\0";
    const TYPE_ID_CSTR: &'static [u8] = b"miniextendr_api::altrep::StreamingRealData\0";
}

impl<F: Fn(usize, &mut [f64]) -> usize + 'static> InferBase for StreamingRealData<F> {
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

impl<F: Fn(usize, &mut [f64]) -> usize + 'static> crate::altrep_traits::Altrep
    for StreamingRealData<F>
{
    fn length(x: crate::ffi::SEXP) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_ext::AltrepSexpExt::altrep_data1::<Self>(&x) }
            .map(|d| d.len() as crate::ffi::R_xlen_t)
            .unwrap_or(0)
    }
}

impl<F: Fn(usize, &mut [f64]) -> usize + 'static> crate::altrep_traits::AltVec
    for StreamingRealData<F>
{
}

impl<F: Fn(usize, &mut [f64]) -> usize + 'static> crate::altrep_traits::AltReal
    for StreamingRealData<F>
{
    const HAS_ELT: bool = true;

    fn elt(x: crate::ffi::SEXP, i: crate::ffi::R_xlen_t) -> f64 {
        unsafe { crate::altrep_ext::AltrepSexpExt::altrep_data1::<Self>(&x) }
            .map(|d| AltRealData::elt(&*d, i as usize))
            .unwrap_or(f64::NAN)
    }

    const HAS_GET_REGION: bool = true;

    fn get_region(
        x: crate::ffi::SEXP,
        start: crate::ffi::R_xlen_t,
        len: crate::ffi::R_xlen_t,
        buf: &mut [f64],
    ) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_ext::AltrepSexpExt::altrep_data1::<Self>(&x) }
            .map(|d| {
                AltRealData::get_region(&*d, start as usize, len as usize, buf)
                    as crate::ffi::R_xlen_t
            })
            .unwrap_or(0)
    }
}
// endregion

// region: StreamingIntData

/// Streaming ALTREP for integer (i32) vectors.
///
/// Elements are loaded on-demand via a reader closure in fixed-size chunks.
/// Chunks are cached in a `BTreeMap` for repeated access.
///
/// # Reader Contract
///
/// The reader `F(start, buf) -> count` fills `buf` with elements starting
/// at index `start` and returns the number of elements actually written.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::altrep_data::StreamingIntData;
///
/// let data = StreamingIntData::new(1000, 64, |start, buf| {
///     let count = buf.len().min(1000 - start);
///     for (i, slot) in buf[..count].iter_mut().enumerate() {
///         *slot = (start + i) as i32;
///     }
///     count
/// });
/// ```
pub struct StreamingIntData<F: Fn(usize, &mut [i32]) -> usize> {
    len: usize,
    reader: F,
    cache: RefCell<BTreeMap<usize, Vec<i32>>>,
    chunk_size: usize,
}

impl<F: Fn(usize, &mut [i32]) -> usize> StreamingIntData<F> {
    /// Create a new streaming integer data source.
    ///
    /// - `len`: total number of elements
    /// - `chunk_size`: number of elements per cache chunk
    /// - `reader`: closure that fills a buffer starting at a given index
    pub fn new(len: usize, chunk_size: usize, reader: F) -> Self {
        Self {
            len,
            reader,
            cache: RefCell::new(BTreeMap::new()),
            chunk_size: chunk_size.max(1),
        }
    }

    /// Load a chunk into the cache if not already present.
    fn ensure_chunk(&self, chunk_idx: usize) {
        let mut cache = self.cache.borrow_mut();
        if cache.contains_key(&chunk_idx) {
            return;
        }
        let start = chunk_idx * self.chunk_size;
        let count = self.chunk_size.min(self.len.saturating_sub(start));
        if count == 0 {
            return;
        }
        let mut buf = vec![0i32; count];
        let written = (self.reader)(start, &mut buf);
        buf.truncate(written);
        cache.insert(chunk_idx, buf);
    }
}

impl<F: Fn(usize, &mut [i32]) -> usize> AltrepLen for StreamingIntData<F> {
    fn len(&self) -> usize {
        self.len
    }
}

impl<F: Fn(usize, &mut [i32]) -> usize> AltIntegerData for StreamingIntData<F> {
    fn elt(&self, i: usize) -> i32 {
        if i >= self.len {
            return crate::altrep_traits::NA_INTEGER;
        }
        let chunk_idx = i / self.chunk_size;
        self.ensure_chunk(chunk_idx);
        let cache = self.cache.borrow();
        let offset = i % self.chunk_size;
        cache
            .get(&chunk_idx)
            .and_then(|chunk| chunk.get(offset).copied())
            .unwrap_or(crate::altrep_traits::NA_INTEGER)
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [i32]) -> usize {
        let count = len.min(self.len.saturating_sub(start)).min(buf.len());
        if count == 0 {
            return 0;
        }
        (self.reader)(start, &mut buf[..count])
    }
}

impl<F: Fn(usize, &mut [i32]) -> usize + 'static> crate::externalptr::TypedExternal
    for StreamingIntData<F>
{
    const TYPE_NAME: &'static str = "StreamingIntData";
    const TYPE_NAME_CSTR: &'static [u8] = b"StreamingIntData\0";
    const TYPE_ID_CSTR: &'static [u8] = b"miniextendr_api::altrep::StreamingIntData\0";
}

impl<F: Fn(usize, &mut [i32]) -> usize + 'static> InferBase for StreamingIntData<F> {
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

impl<F: Fn(usize, &mut [i32]) -> usize + 'static> crate::altrep_traits::Altrep
    for StreamingIntData<F>
{
    fn length(x: crate::ffi::SEXP) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_ext::AltrepSexpExt::altrep_data1::<Self>(&x) }
            .map(|d| d.len() as crate::ffi::R_xlen_t)
            .unwrap_or(0)
    }
}

impl<F: Fn(usize, &mut [i32]) -> usize + 'static> crate::altrep_traits::AltVec
    for StreamingIntData<F>
{
}

impl<F: Fn(usize, &mut [i32]) -> usize + 'static> crate::altrep_traits::AltInteger
    for StreamingIntData<F>
{
    const HAS_ELT: bool = true;

    fn elt(x: crate::ffi::SEXP, i: crate::ffi::R_xlen_t) -> i32 {
        unsafe { crate::altrep_ext::AltrepSexpExt::altrep_data1::<Self>(&x) }
            .map(|d| AltIntegerData::elt(&*d, i as usize))
            .unwrap_or(i32::MIN)
    }

    const HAS_GET_REGION: bool = true;

    fn get_region(
        x: crate::ffi::SEXP,
        start: crate::ffi::R_xlen_t,
        len: crate::ffi::R_xlen_t,
        buf: &mut [i32],
    ) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_ext::AltrepSexpExt::altrep_data1::<Self>(&x) }
            .map(|d| {
                AltIntegerData::get_region(&*d, start as usize, len as usize, buf)
                    as crate::ffi::R_xlen_t
            })
            .unwrap_or(0)
    }
}
// endregion
