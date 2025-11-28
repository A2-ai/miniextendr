//! Standard ALTREP backends (Vec/Arc/Slice/Mmap/Owned) split out for clarity.

use crate::altrep::{IntBackend, LogicalBackend, RawBackend, RealBackend, StringBackend};
use crate::ffi::{DATAPTR_RO, R_NaString, R_xlen_t, Rbyte, Rf_translateCharUTF8, SEXP, VECTOR_ELT};
use core::slice;
use std::sync::{Arc, OnceLock};

// Compact integer sequence backend
pub struct CompactIntSeq {
    pub len: R_xlen_t,
    pub start: i32,
    pub step: i32,
}
impl CompactIntSeq {
    pub fn new(len: R_xlen_t, start: i32, step: i32) -> Self {
        Self { len, start, step }
    }
}
impl IntBackend for CompactIntSeq {
    fn len(&self) -> R_xlen_t {
        self.len
    }
    fn elt(&self, i: R_xlen_t) -> i32 {
        self.start.wrapping_add(self.step.wrapping_mul(i as i32))
    }
    fn is_sorted(&self) -> i32 {
        if self.step < 0 { -1 } else { 1 }
    }
    fn no_na(&self) -> i32 {
        1
    }
}

// Owned/Arc/Slice/Map for Real
pub struct OwnedReal {
    pub data: Box<[f64]>,
}
impl OwnedReal {
    /// # Safety
    /// `x` must be a REALSXP vector; its data must remain valid for the duration
    /// of the copy. Must be called on the R thread with R initialized.
    pub unsafe fn from_reals_sexp(x: SEXP) -> Self {
        let n = unsafe { crate::ffi::Rf_xlength(x) } as usize;
        let ptr = unsafe { DATAPTR_RO(x) as *const f64 };
        let slice = unsafe { slice::from_raw_parts(ptr, n) };
        Self {
            data: slice.to_vec().into_boxed_slice(),
        }
    }
}
impl RealBackend for OwnedReal {
    fn len(&self) -> R_xlen_t {
        self.data.len() as R_xlen_t
    }
    fn elt(&self, i: R_xlen_t) -> f64 {
        self.data[i as usize]
    }
    fn get_region(&self, i: R_xlen_t, n: R_xlen_t, out: &mut [f64]) -> R_xlen_t {
        let end = (i + n).min(self.len()) as usize;
        let src = &self.data[i as usize..end];
        out[..src.len()].copy_from_slice(src);
        src.len() as R_xlen_t
    }
    fn dataptr(&self) -> Option<&[f64]> {
        Some(&self.data)
    }
    fn no_na(&self) -> i32 {
        1
    }
}

pub struct RealVec {
    pub data: Box<[f64]>,
}
impl From<Vec<f64>> for RealVec {
    fn from(v: Vec<f64>) -> Self {
        Self {
            data: v.into_boxed_slice(),
        }
    }
}
impl RealBackend for RealVec {
    fn len(&self) -> R_xlen_t {
        self.data.len() as R_xlen_t
    }
    fn elt(&self, i: R_xlen_t) -> f64 {
        self.data[i as usize]
    }
    fn get_region(&self, i: R_xlen_t, n: R_xlen_t, out: &mut [f64]) -> R_xlen_t {
        let end = (i + n).min(self.len()) as usize;
        let src = &self.data[i as usize..end];
        out[..src.len()].copy_from_slice(src);
        src.len() as R_xlen_t
    }
    fn dataptr(&self) -> Option<&[f64]> {
        Some(&self.data)
    }
}

pub struct RealArc {
    pub data: Arc<[f64]>,
}
impl From<Arc<[f64]>> for RealArc {
    fn from(data: Arc<[f64]>) -> Self {
        Self { data }
    }
}
impl RealBackend for RealArc {
    fn len(&self) -> R_xlen_t {
        self.data.len() as R_xlen_t
    }
    fn elt(&self, i: R_xlen_t) -> f64 {
        self.data[i as usize]
    }
    fn get_region(&self, i: R_xlen_t, n: R_xlen_t, out: &mut [f64]) -> R_xlen_t {
        let end = (i + n).min(self.len()) as usize;
        let src = &self.data[i as usize..end];
        out[..src.len()].copy_from_slice(src);
        src.len() as R_xlen_t
    }
    fn dataptr(&self) -> Option<&[f64]> {
        Some(&self.data)
    }
}

pub struct RealSliceMat {
    pub src: &'static [f64],
    pub materialized: OnceLock<Box<[f64]>>,
}
impl RealSliceMat {
    pub fn new(src: &'static [f64]) -> Self {
        Self {
            src,
            materialized: OnceLock::new(),
        }
    }
}
impl RealBackend for RealSliceMat {
    fn len(&self) -> R_xlen_t {
        self.src.len() as R_xlen_t
    }
    fn elt(&self, i: R_xlen_t) -> f64 {
        self.src[i as usize]
    }
    fn get_region(&self, i: R_xlen_t, n: R_xlen_t, out: &mut [f64]) -> R_xlen_t {
        let end = (i + n).min(self.len()) as usize;
        let src = &self.src[i as usize..end];
        out[..src.len()].copy_from_slice(src);
        src.len() as R_xlen_t
    }
    fn dataptr(&self) -> Option<&[f64]> {
        let bx = self
            .materialized
            .get_or_init(|| self.src.to_vec().into_boxed_slice());
        Some(&**bx)
    }
}

pub struct RealMmap {
    pub ptr: *const f64,
    pub len: usize,
    pub cleanup: Option<unsafe extern "C-unwind" fn(*const f64, usize)>,
}
unsafe impl Send for RealMmap {}
unsafe impl Sync for RealMmap {}
impl RealMmap {
    /// # Safety
    /// Caller must guarantee `ptr` points to a readable buffer of `len` f64s
    /// valid for the lifetime of this object, or until `cleanup` is invoked.
    pub unsafe fn new(
        ptr: *const f64,
        len: usize,
        cleanup: Option<unsafe extern "C-unwind" fn(*const f64, usize)>,
    ) -> Self {
        Self { ptr, len, cleanup }
    }
}
impl Drop for RealMmap {
    fn drop(&mut self) {
        if let Some(f) = self.cleanup {
            unsafe { f(self.ptr, self.len) }
        }
    }
}
impl RealBackend for RealMmap {
    fn len(&self) -> R_xlen_t {
        self.len as R_xlen_t
    }
    fn elt(&self, i: R_xlen_t) -> f64 {
        unsafe { *self.ptr.add(i as usize) }
    }
    fn get_region(&self, i: R_xlen_t, n: R_xlen_t, out: &mut [f64]) -> R_xlen_t {
        let start = i as usize;
        let end = ((i + n).min(self.len())) as usize;
        let src = unsafe { slice::from_raw_parts(self.ptr.add(start), end - start) };
        out[..src.len()].copy_from_slice(src);
        src.len() as R_xlen_t
    }
    fn dataptr(&self) -> Option<&[f64]> {
        Some(unsafe { slice::from_raw_parts(self.ptr, self.len) })
    }
}

// String backends
pub struct Utf8Vec {
    pub data: Vec<String>,
    pub na: Vec<bool>,
}
impl Utf8Vec {
    /// # Safety
    /// `x` must be a STRSXP vector and used on the R thread. Elements are copied
    /// to owned Strings; NA elements are tracked separately.
    pub unsafe fn from_strs_sexp(x: SEXP) -> Self {
        let n: R_xlen_t = unsafe { crate::ffi::Rf_xlength(x) };
        let mut data = Vec::with_capacity(n as usize);
        let mut na = Vec::with_capacity(n as usize);
        for i in 0..n {
            let ch = unsafe { VECTOR_ELT(x, i) };
            if ch == unsafe { R_NaString } {
                data.push(String::new());
                na.push(true);
            } else {
                let c = unsafe { Rf_translateCharUTF8(ch) };
                let s = unsafe { std::ffi::CStr::from_ptr(c) }
                    .to_string_lossy()
                    .into_owned();
                data.push(s);
                na.push(false);
            }
        }
        Self { data, na }
    }
}
impl StringBackend for Utf8Vec {
    fn len(&self) -> R_xlen_t {
        self.data.len() as R_xlen_t
    }
    fn utf8_at(&self, i: R_xlen_t) -> Option<&str> {
        let i = i as usize;
        if self.na[i] {
            None
        } else {
            Some(self.data[i].as_str())
        }
    }
}

pub struct Utf8Arc {
    pub data: Arc<[String]>,
}
impl From<Arc<[String]>> for Utf8Arc {
    fn from(data: Arc<[String]>) -> Self {
        Self { data }
    }
}
impl StringBackend for Utf8Arc {
    fn len(&self) -> R_xlen_t {
        self.data.len() as R_xlen_t
    }
    fn utf8_at(&self, i: R_xlen_t) -> Option<&str> {
        Some(self.data[i as usize].as_str())
    }
}

pub struct Utf8Slice {
    pub data: &'static [&'static str],
}
impl Utf8Slice {
    pub fn new(data: &'static [&'static str]) -> Self {
        Self { data }
    }
}
impl StringBackend for Utf8Slice {
    fn len(&self) -> R_xlen_t {
        self.data.len() as R_xlen_t
    }
    fn utf8_at(&self, i: R_xlen_t) -> Option<&str> {
        Some(self.data[i as usize])
    }
}

// Logical
pub struct LogicalVec {
    pub data: Box<[i32]>,
}
impl From<Vec<i32>> for LogicalVec {
    fn from(v: Vec<i32>) -> Self {
        Self {
            data: v.into_boxed_slice(),
        }
    }
}
impl LogicalBackend for LogicalVec {
    fn len(&self) -> R_xlen_t {
        self.data.len() as R_xlen_t
    }
    fn elt(&self, i: R_xlen_t) -> i32 {
        self.data[i as usize]
    }
    fn get_region(&self, i: R_xlen_t, n: R_xlen_t, out: &mut [i32]) -> R_xlen_t {
        let end = (i + n).min(self.len()) as usize;
        let src = &self.data[i as usize..end];
        out[..src.len()].copy_from_slice(src);
        src.len() as R_xlen_t
    }
    fn dataptr(&self) -> Option<&[i32]> {
        Some(&self.data)
    }
}

pub struct LogicalArc {
    pub data: Arc<[i32]>,
}
impl From<Arc<[i32]>> for LogicalArc {
    fn from(data: Arc<[i32]>) -> Self {
        Self { data }
    }
}
impl LogicalBackend for LogicalArc {
    fn len(&self) -> R_xlen_t {
        self.data.len() as R_xlen_t
    }
    fn elt(&self, i: R_xlen_t) -> i32 {
        self.data[i as usize]
    }
    fn get_region(&self, i: R_xlen_t, n: R_xlen_t, out: &mut [i32]) -> R_xlen_t {
        let end = (i + n).min(self.len()) as usize;
        let src = &self.data[i as usize..end];
        out[..src.len()].copy_from_slice(src);
        src.len() as R_xlen_t
    }
    fn dataptr(&self) -> Option<&[i32]> {
        Some(&self.data)
    }
}

pub struct LogicalSliceMat {
    pub src: &'static [i32],
    pub materialized: OnceLock<Box<[i32]>>,
}
impl LogicalSliceMat {
    pub fn new(src: &'static [i32]) -> Self {
        Self {
            src,
            materialized: OnceLock::new(),
        }
    }
}
impl LogicalBackend for LogicalSliceMat {
    fn len(&self) -> R_xlen_t {
        self.src.len() as R_xlen_t
    }
    fn elt(&self, i: R_xlen_t) -> i32 {
        self.src[i as usize]
    }
    fn get_region(&self, i: R_xlen_t, n: R_xlen_t, out: &mut [i32]) -> R_xlen_t {
        let end = (i + n).min(self.len()) as usize;
        let src = &self.src[i as usize..end];
        out[..src.len()].copy_from_slice(src);
        src.len() as R_xlen_t
    }
    fn dataptr(&self) -> Option<&[i32]> {
        let bx = self
            .materialized
            .get_or_init(|| self.src.to_vec().into_boxed_slice());
        Some(&**bx)
    }
}

pub struct LogicalMmap {
    pub ptr: *const i32,
    pub len: usize,
    pub cleanup: Option<unsafe extern "C-unwind" fn(*const i32, usize)>,
}
unsafe impl Send for LogicalMmap {}
unsafe impl Sync for LogicalMmap {}
impl LogicalMmap {
    /// # Safety
    /// Caller must guarantee `ptr` points to a readable buffer of `len` i32s
    /// valid for the lifetime of this object, or until `cleanup` is invoked.
    pub unsafe fn new(
        ptr: *const i32,
        len: usize,
        cleanup: Option<unsafe extern "C-unwind" fn(*const i32, usize)>,
    ) -> Self {
        Self { ptr, len, cleanup }
    }
}
impl Drop for LogicalMmap {
    fn drop(&mut self) {
        if let Some(f) = self.cleanup {
            unsafe { f(self.ptr, self.len) }
        }
    }
}
impl LogicalBackend for LogicalMmap {
    fn len(&self) -> R_xlen_t {
        self.len as R_xlen_t
    }
    fn elt(&self, i: R_xlen_t) -> i32 {
        unsafe { *self.ptr.add(i as usize) }
    }
    fn get_region(&self, i: R_xlen_t, n: R_xlen_t, out: &mut [i32]) -> R_xlen_t {
        let start = i as usize;
        let end = ((i + n).min(self.len())) as usize;
        let src = unsafe { slice::from_raw_parts(self.ptr.add(start), end - start) };
        out[..src.len()].copy_from_slice(src);
        src.len() as R_xlen_t
    }
    fn dataptr(&self) -> Option<&[i32]> {
        Some(unsafe { slice::from_raw_parts(self.ptr, self.len) })
    }
}

// Raw
pub struct RawVec {
    pub data: Box<[Rbyte]>,
}
impl From<Vec<Rbyte>> for RawVec {
    fn from(v: Vec<Rbyte>) -> Self {
        Self {
            data: v.into_boxed_slice(),
        }
    }
}
impl RawBackend for RawVec {
    fn len(&self) -> R_xlen_t {
        self.data.len() as R_xlen_t
    }
    fn elt(&self, i: R_xlen_t) -> Rbyte {
        self.data[i as usize]
    }
    fn get_region(&self, i: R_xlen_t, n: R_xlen_t, out: &mut [Rbyte]) -> R_xlen_t {
        let end = (i + n).min(self.len()) as usize;
        let src = &self.data[i as usize..end];
        out[..src.len()].copy_from_slice(src);
        src.len() as R_xlen_t
    }
    fn dataptr(&self) -> Option<&[Rbyte]> {
        Some(&self.data)
    }
}

pub struct RawArc {
    pub data: Arc<[Rbyte]>,
}
impl From<Arc<[Rbyte]>> for RawArc {
    fn from(data: Arc<[Rbyte]>) -> Self {
        Self { data }
    }
}
impl RawBackend for RawArc {
    fn len(&self) -> R_xlen_t {
        self.data.len() as R_xlen_t
    }
    fn elt(&self, i: R_xlen_t) -> Rbyte {
        self.data[i as usize]
    }
    fn get_region(&self, i: R_xlen_t, n: R_xlen_t, out: &mut [Rbyte]) -> R_xlen_t {
        let end = (i + n).min(self.len()) as usize;
        let src = &self.data[i as usize..end];
        out[..src.len()].copy_from_slice(src);
        src.len() as R_xlen_t
    }
    fn dataptr(&self) -> Option<&[Rbyte]> {
        Some(&self.data)
    }
}

pub struct RawSliceMat {
    pub src: &'static [Rbyte],
    pub materialized: OnceLock<Box<[Rbyte]>>,
}
impl RawSliceMat {
    pub fn new(src: &'static [Rbyte]) -> Self {
        Self {
            src,
            materialized: OnceLock::new(),
        }
    }
}
impl RawBackend for RawSliceMat {
    fn len(&self) -> R_xlen_t {
        self.src.len() as R_xlen_t
    }
    fn elt(&self, i: R_xlen_t) -> Rbyte {
        self.src[i as usize]
    }
    fn get_region(&self, i: R_xlen_t, n: R_xlen_t, out: &mut [Rbyte]) -> R_xlen_t {
        let end = (i + n).min(self.len()) as usize;
        let src = &self.src[i as usize..end];
        out[..src.len()].copy_from_slice(src);
        src.len() as R_xlen_t
    }
    fn dataptr(&self) -> Option<&[Rbyte]> {
        let bx = self
            .materialized
            .get_or_init(|| self.src.to_vec().into_boxed_slice());
        Some(&**bx)
    }
}

pub struct RawMmap {
    pub ptr: *const Rbyte,
    pub len: usize,
    pub cleanup: Option<unsafe extern "C-unwind" fn(*const Rbyte, usize)>,
}
unsafe impl Send for RawMmap {}
unsafe impl Sync for RawMmap {}
impl RawMmap {
    /// # Safety
    /// Caller must guarantee `ptr` points to a readable buffer of `len` bytes
    /// valid for the lifetime of this object, or until `cleanup` is invoked.
    pub unsafe fn new(
        ptr: *const Rbyte,
        len: usize,
        cleanup: Option<unsafe extern "C-unwind" fn(*const Rbyte, usize)>,
    ) -> Self {
        Self { ptr, len, cleanup }
    }
}
impl Drop for RawMmap {
    fn drop(&mut self) {
        if let Some(f) = self.cleanup {
            unsafe { f(self.ptr, self.len) }
        }
    }
}
impl RawBackend for RawMmap {
    fn len(&self) -> R_xlen_t {
        self.len as R_xlen_t
    }
    fn elt(&self, i: R_xlen_t) -> Rbyte {
        unsafe { *self.ptr.add(i as usize) }
    }
    fn get_region(&self, i: R_xlen_t, n: R_xlen_t, out: &mut [Rbyte]) -> R_xlen_t {
        let start = i as usize;
        let end = ((i + n).min(self.len())) as usize;
        let src = unsafe { slice::from_raw_parts(self.ptr.add(start), end - start) };
        out[..src.len()].copy_from_slice(src);
        src.len() as R_xlen_t
    }
    fn dataptr(&self) -> Option<&[Rbyte]> {
        Some(unsafe { slice::from_raw_parts(self.ptr, self.len) })
    }
}

// Owned VECSXP helper
pub struct OwnedList {
    pub data: Vec<SEXP>,
}
impl OwnedList {
    pub fn from_sexps(v: Vec<SEXP>) -> Self {
        Self { data: v }
    }
    /// # Safety
    /// `x` must be a VECSXP list. Elements are shallow-copied SEXP handles.
    pub unsafe fn from_list_sexp(x: SEXP) -> Self {
        let n: R_xlen_t = unsafe { crate::ffi::Rf_xlength(x) };
        let mut v = Vec::with_capacity(n as usize);
        for i in 0..n {
            let elt = unsafe { VECTOR_ELT(x, i) };
            v.push(elt);
        }
        Self { data: v }
    }
}

// Int slice materializer
pub struct IntSliceMat {
    pub src: &'static [i32],
    pub materialized: OnceLock<Box<[i32]>>,
}
impl IntSliceMat {
    pub fn new(src: &'static [i32]) -> Self {
        Self {
            src,
            materialized: OnceLock::new(),
        }
    }
}
impl IntBackend for IntSliceMat {
    fn len(&self) -> R_xlen_t {
        self.src.len() as R_xlen_t
    }
    fn elt(&self, i: R_xlen_t) -> i32 {
        self.src[i as usize]
    }
    fn get_region(&self, i: R_xlen_t, n: R_xlen_t, out: &mut [i32]) -> R_xlen_t {
        let end = (i + n).min(self.len()) as usize;
        let src = &self.src[i as usize..end];
        out[..src.len()].copy_from_slice(src);
        src.len() as R_xlen_t
    }
    fn dataptr(&self) -> Option<&[i32]> {
        let bx = self
            .materialized
            .get_or_init(|| self.src.to_vec().into_boxed_slice());
        Some(&**bx)
    }
}
