//! rpkg: Example R package demonstrating miniextendr features.
//!
//! This crate is organized into focused modules for different test categories:
//! - `panic_tests`: Panic, drop, and R error handling tests
//! - `unwind_protect_tests`: `with_r_unwind_protect` mechanism tests
//! - `dots_tests`: R dots (`...`) handling tests
//! - `interrupt_tests`: R interrupt checking tests
//! - `conversion_tests`: Scalar and slice conversion tests
//! - `externalptr_tests`: ExternalPtr functionality tests
//! - `receiver_tests`: Receiver-style impl block tests
//! - `worker_tests`: Worker thread and `with_r_thread` tests
//! - `coerce_tests`: Coerce, TryCoerce, RNativeType trait tests
//! - `visibility_tests`: R return value visibility tests
//! - `thread_tests`: RThreadBuilder and thread safety tests
//! - `misc_tests`: Miscellaneous test functions
//! - `nonapi`: Feature-gated tests requiring nonapi feature

use miniextendr_api::ffi::SEXP;
use miniextendr_api::from_r::TryFromSexp;
use miniextendr_api::{miniextendr, miniextendr_module};

// Test modules
mod coerce_tests;
mod conversion_tests;
mod dots_tests;
mod externalptr_tests;
mod interrupt_tests;
mod misc_tests;
mod panic_tests;
mod r6_tests;
mod receiver_tests;
mod s3_tests;
mod s4_tests;
mod s7_tests;
mod thread_tests;
mod unwind_protect_tests;
mod visibility_tests;
mod worker_tests;

// Stub for ALTREP re-exports (actual ALTREP code is below)
mod altrep;

// region: proc-macro ALTREP test
// This tests the #[miniextendr] on struct path for custom ALTREP classes.
//
// The new approach requires:
// 1. A data type that implements high-level data traits (AltrepLen, AltIntegerData, etc.)
// 2. Low-level trait impls generated via impl_alt*_from_data! macro
// 3. A 1-field wrapper struct with #[miniextendr] macro

use miniextendr_api::altrep_data::{AltIntegerData, AltrepLen};

// -----------------------------------------------------------------------------
// ConstantInt: An ALTREP integer that always returns the same value
// -----------------------------------------------------------------------------

/// Data type that stores a constant value and length
#[derive(miniextendr_api::ExternalPtr)]
pub struct ConstantIntData {
    value: i32,
    len: usize,
}

// Implement high-level data traits
impl AltrepLen for ConstantIntData {
    fn len(&self) -> usize {
        self.len
    }
}

impl AltIntegerData for ConstantIntData {
    fn elt(&self, _i: usize) -> i32 {
        self.value
    }

    fn no_na(&self) -> Option<bool> {
        Some(self.value != i32::MIN) // NA is i32::MIN
    }
}

// Generate low-level traits from data traits
miniextendr_api::impl_altinteger_from_data!(ConstantIntData);

/// ALTREP wrapper for ConstantIntData
#[miniextendr(class = "ConstantInt", pkg = "rpkg", base = "Int")]
pub struct ConstantIntClass(ConstantIntData);

/// Create a ConstantInt ALTREP instance (all elements are 42, length 10).
///
/// # Safety
///
/// Must be called from R main thread with R properly initialized.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn rpkg_constant_int() -> SEXP {
    let data = ConstantIntData { value: 42, len: 10 };
    unsafe { ConstantIntClass::into_altrep(data) }
}

// endregion

// region: ExternalPtr tests

use miniextendr_api::externalptr::ErasedExternalPtr;
// Note: ExternalPtr type is accessed via full path to avoid conflict with derive macro
use miniextendr_api::ExternalPtr as DeriveExternalPtr;

/// A simple test struct for ExternalPtr
#[derive(DeriveExternalPtr, Debug)]
struct Counter {
    value: i32,
}

/// Another test struct to verify type safety
#[derive(DeriveExternalPtr, Debug)]
struct Point {
    x: f64,
    y: f64,
}

/// Create a new Counter wrapped in an ExternalPtr
#[miniextendr(unsafe(main_thread))]
fn extptr_counter_new(initial: i32) -> miniextendr_api::externalptr::ExternalPtr<Counter> {
    miniextendr_api::externalptr::ExternalPtr::new(Counter { value: initial })
}

/// Get the current value from a Counter ExternalPtr
///
/// # Safety
///
/// `ptr` must be a valid SEXP.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn C_extptr_counter_get(ptr: SEXP) -> SEXP {
    use miniextendr_api::externalptr::ExternalPtr;
    use miniextendr_api::ffi::Rf_ScalarInteger;
    unsafe {
        match ExternalPtr::<Counter>::try_from_sexp(ptr) {
            Some(ext) => Rf_ScalarInteger(ext.value),
            None => Rf_ScalarInteger(i32::MIN), // NA_INTEGER equivalent
        }
    }
}

impl AltStringData for LazyStringData {
    fn elt(&self, _i: usize) -> Option<&str> {
        // Note: For a real implementation you'd want to cache generated strings
        // Since we can't return a reference to a newly created String, return None
        // which triggers R's default behavior (NA)
        None
    }
    fn no_na(&self) -> Option<bool> {
        Some(false)
    } // We return None which is like NA
}

miniextendr_api::impl_altstring_from_data!(LazyStringData);

#[miniextendr(class = "LazyString", pkg = "rpkg")]
pub struct LazyStringClass(pub LazyStringData);

#[miniextendr]
fn lazy_string(prefix: &str, n: i32) -> SEXP {
    let data = LazyStringData {
        prefix: prefix.to_string(),
        len: n as usize,
    };
    LazyStringClass::into_altrep(data)
}

// -----------------------------------------------------------------------------
// RepeatingRaw: Repeating byte pattern
// -----------------------------------------------------------------------------

#[derive(miniextendr_api::ExternalPtr)]
pub struct RepeatingRawData {
    pattern: Vec<u8>,
    total_len: usize,
}

impl AltrepLen for RepeatingRawData {
    fn len(&self) -> usize {
        self.total_len
    }
}

impl AltRawData for RepeatingRawData {
    fn elt(&self, i: usize) -> u8 {
        if self.pattern.is_empty() {
            0
        } else {
            self.pattern[i % self.pattern.len()]
        }
    }
}

miniextendr_api::impl_altraw_from_data!(RepeatingRawData);

#[miniextendr(class = "RepeatingRaw", pkg = "rpkg")]
pub struct RepeatingRawClass(pub RepeatingRawData);

#[miniextendr]
fn repeating_raw(pattern: &[u8], n: i32) -> SEXP {
    let data = RepeatingRawData {
        pattern: pattern.to_vec(),
        total_len: n as usize,
    };
    RepeatingRawClass::into_altrep(data)
}

// -----------------------------------------------------------------------------
// UnitCircle: Complex numbers on the unit circle (e^(i*theta))
// This demonstrates ALTREP for complex vectors
// -----------------------------------------------------------------------------

use miniextendr_api::altrep_data::AltComplexData;
use miniextendr_api::ffi::Rcomplex;

#[derive(miniextendr_api::ExternalPtr)]
pub struct UnitCircleData {
    /// Number of points on the unit circle
    n: usize,
}

impl AltrepLen for UnitCircleData {
    fn len(&self) -> usize {
        self.n
    }
}

impl AltComplexData for UnitCircleData {
    fn elt(&self, i: usize) -> Rcomplex {
        // Generate e^(i * 2π * k/n) = cos(2πk/n) + i*sin(2πk/n)
        let theta = 2.0 * std::f64::consts::PI * (i as f64) / (self.n as f64);
        Rcomplex {
            r: theta.cos(),
            i: theta.sin(),
        }
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [Rcomplex]) -> usize {
        let end = (start + len).min(self.n);
        for (buf_i, i) in (start..end).enumerate() {
            buf[buf_i] = self.elt(i);
        }
        end - start
    }
}

miniextendr_api::impl_altcomplex_from_data!(UnitCircleData);

/// ALTREP wrapper for UnitCircleData - generates complex numbers on unit circle
#[miniextendr(class = "UnitCircle", pkg = "rpkg")]
pub struct UnitCircleClass(pub UnitCircleData);

/// Create complex numbers on the unit circle: e^(i * 2π * k/n) for k = 0, 1, ..., n-1
/// These are the n-th roots of unity, evenly spaced around the unit circle.
/// @title ALTREP Example Constructors
/// @name rpkg_altrep_examples
/// @description ALTREP example constructors
/// @return An ALTREP vector.
/// @examples
/// unit_circle(8L)
/// lazy <- lazy_int_seq(1L, 5L, 1L)
/// lazy[1:3]
/// boxed_ints(3L)
/// static_strings()
/// @aliases unit_circle lazy_int_seq boxed_ints static_ints leaked_ints static_strings
#[miniextendr]
pub fn unit_circle(n: i32) -> SEXP {
    let data = UnitCircleData { n: n as usize };
    UnitCircleClass::into_altrep(data)
}

// -----------------------------------------------------------------------------
// SimpleVecInt: Vec<i32> wrapper (simplest example)
// -----------------------------------------------------------------------------

#[miniextendr(class = "SimpleVecInt", pkg = "rpkg")]
pub struct SimpleVecIntClass(pub Vec<i32>);

/// # Safety
/// Caller must ensure `x` is a valid integer SEXP and this is called from R's main thread.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn C_extptr_is_point(ptr: SEXP) -> SEXP {
    use miniextendr_api::ffi::Rf_ScalarInteger;
    unsafe {
        let erased = ErasedExternalPtr::from_sexp(ptr);
        if erased.is::<Point>() {
            Rf_ScalarInteger(1)
        } else {
            Rf_ScalarInteger(0)
        }
    }
}

// endregion

// region: Additional ALTREP examples - using new 1-field struct pattern
//
// The new ALTREP API requires:
// 1. A data type that implements high-level data traits (AltrepLen, Alt*Data)
// 2. Low-level trait impls generated via impl_alt*_from_data! macro
// 3. A 1-field wrapper struct with #[miniextendr] macro
//
// For custom behavior that can't be expressed through the data traits,
// manually implement the low-level traits on the data type.

use miniextendr_api::altrep_data::{AltRealData, AltLogicalData, AltRawData, AltStringData, Logical};

// -----------------------------------------------------------------------------
// ConstantReal: All elements are PI
// -----------------------------------------------------------------------------

#[derive(miniextendr_api::ExternalPtr)]
pub struct ConstantRealData {
    value: f64,
    len: usize,
}

impl AltrepLen for ConstantRealData {
    fn len(&self) -> usize { self.len }
}

impl AltRealData for ConstantRealData {
    fn elt(&self, _i: usize) -> f64 { self.value }
    fn no_na(&self) -> Option<bool> { Some(!self.value.is_nan()) }
}

miniextendr_api::impl_altreal_from_data!(ConstantRealData);

#[miniextendr(class = "ConstantReal", pkg = "rpkg", base = "Real")]
pub struct ConstantRealClass(ConstantRealData);

#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn rpkg_constant_real() -> SEXP {
    let data = ConstantRealData { value: std::f64::consts::PI, len: 10 };
    unsafe { ConstantRealClass::into_altrep(data) }
}

// -----------------------------------------------------------------------------
// ArithSeq: Arithmetic sequence (like R's seq())
// -----------------------------------------------------------------------------

#[derive(miniextendr_api::ExternalPtr)]
pub struct ArithSeqData {
    start: f64,
    step: f64,
    len: usize,
}

impl AltrepLen for ArithSeqData {
    fn len(&self) -> usize { self.len }
}

impl AltRealData for ArithSeqData {
    fn elt(&self, i: usize) -> f64 {
        self.start + (i as f64) * self.step
    }
    fn no_na(&self) -> Option<bool> { Some(true) }
}

miniextendr_api::impl_altreal_from_data!(ArithSeqData);

#[miniextendr(class = "ArithSeq", pkg = "rpkg", base = "Real")]
pub struct ArithSeqClass(ArithSeqData);

#[miniextendr]
fn arith_seq(from: f64, to: f64, length_out: i32) -> SEXP {
    let len = length_out as usize;
    let step = if len > 1 { (to - from) / (len - 1) as f64 } else { 0.0 };
    let data = ArithSeqData { start: from, step, len };
    unsafe { ArithSeqClass::into_altrep(data) }
}

// -----------------------------------------------------------------------------
// LazyIntSeq: Integer arithmetic sequence with lazy materialization
// This demonstrates the Dataptr lazy materialization pattern:
// - Elements are computed on-demand via Elt/Get_region
// - Full buffer is only allocated when Dataptr is called
// - Dataptr_or_null returns NULL until materialized
// -----------------------------------------------------------------------------

/// Data type for lazy integer sequence with materialization support
#[derive(miniextendr_api::ExternalPtr)]
pub struct LazyIntSeqData {
    start: i32,
    step: i32,
    len: usize,
    /// Lazily-allocated buffer for materialization
    materialized: Option<Vec<i32>>,
}

impl AltrepLen for LazyIntSeqData {
    fn len(&self) -> usize {
        self.len
    }
}

impl AltIntegerData for LazyIntSeqData {
    fn elt(&self, i: usize) -> i32 {
        // Compute element on-the-fly (no materialization needed)
        self.start.saturating_add((i as i32).saturating_mul(self.step))
    }

    fn no_na(&self) -> Option<bool> {
        // Check if any element would be NA (i32::MIN)
        // This is a conservative check - we know the formula
        Some(true)
    }

    fn is_sorted(&self) -> Option<miniextendr_api::altrep_data::Sortedness> {
        use miniextendr_api::altrep_data::Sortedness;
        if self.step == 0 {
            Some(Sortedness::Increasing) // All same value
        } else if self.step > 0 {
            Some(Sortedness::StrictlyIncreasing)
        } else {
            Some(Sortedness::StrictlyDecreasing)
        }
    }

    fn sum(&self, _na_rm: bool) -> Option<i64> {
        // Arithmetic sequence sum: n * (first + last) / 2
        let n = self.len as i64;
        let first = self.start as i64;
        let last = first + (self.len.saturating_sub(1) as i64) * (self.step as i64);
        Some(n * (first + last) / 2)
    }

    fn min(&self, _na_rm: bool) -> Option<i32> {
        if self.len == 0 {
            None
        } else if self.step >= 0 {
            Some(self.start)
        } else {
            Some(self.elt(self.len - 1))
        }
    }

    fn max(&self, _na_rm: bool) -> Option<i32> {
        if self.len == 0 {
            None
        } else if self.step >= 0 {
            Some(self.elt(self.len - 1))
        } else {
            Some(self.start)
        }
    }
}

/// Implement AltrepDataptr for lazy materialization
impl miniextendr_api::altrep_data::AltrepDataptr<i32> for LazyIntSeqData {
    fn dataptr(&mut self, _writable: bool) -> Option<*mut i32> {
        // Materialize on first access
        if self.materialized.is_none() {
            eprintln!("[Rust] LazyIntSeq: Materializing {} elements...", self.len);
            let data: Vec<i32> = (0..self.len)
                .map(|i| self.start.saturating_add((i as i32).saturating_mul(self.step)))
                .collect();
            self.materialized = Some(data);
            eprintln!("[Rust] LazyIntSeq: Materialization complete!");
        }
        self.materialized.as_mut().map(|v| v.as_mut_ptr())
    }

    fn dataptr_or_null(&self) -> Option<*const i32> {
        // Only return pointer if already materialized
        // This allows R to use Elt/Get_region for unmaterialized data
        self.materialized.as_ref().map(|v| v.as_ptr())
    }
}

// Implement serialization support
impl miniextendr_api::altrep_data::AltrepSerialize for LazyIntSeqData {
    fn serialized_state(&self) -> SEXP {
        // Store start, step, len in an integer vector
        // Note: We don't serialize the materialized buffer - it will be recomputed on demand
        unsafe {
            use miniextendr_api::ffi::{Rf_allocVector, SET_INTEGER_ELT, INTSXP};
            let state = Rf_allocVector(INTSXP, 3);
            SET_INTEGER_ELT(state, 0, self.start);
            SET_INTEGER_ELT(state, 1, self.step);
            SET_INTEGER_ELT(state, 2, self.len as i32);
            state
        }
    }

    fn unserialize(state: SEXP) -> Option<Self> {
        unsafe {
            use miniextendr_api::ffi::INTEGER_ELT;
            let start = INTEGER_ELT(state, 0);
            let step = INTEGER_ELT(state, 1);
            let len = INTEGER_ELT(state, 2) as usize;
            Some(LazyIntSeqData {
                start,
                step,
                len,
                materialized: None, // Fresh start - not materialized
            })
        }
    }
}

// Use the dataptr + serialize variant to enable both Dataptr and serialization methods
miniextendr_api::impl_altinteger_from_data!(LazyIntSeqData, dataptr, serialize);

// Implement AltrepBase and AltrepInstaller to enable auto-inference of base type
impl miniextendr_api::altrep::AltrepBase for LazyIntSeqData {
    const BASE: miniextendr_api::altrep::RBase = miniextendr_api::altrep::RBase::Int;
}

impl miniextendr_api::altrep_registration::AltrepInstaller for LazyIntSeqData {
    unsafe fn make_class(
        class_name: *const i8,
        pkg_name: *const i8,
    ) -> miniextendr_api::ffi::altrep::R_altrep_class_t {
        unsafe {
            miniextendr_api::ffi::altrep::R_make_altinteger_class(
                class_name,
                pkg_name,
                core::ptr::null_mut(),
            )
        }
    }

    unsafe fn install_methods(cls: miniextendr_api::ffi::altrep::R_altrep_class_t) {
        use miniextendr_api::altrep_traits::*;
        use miniextendr_api::ffi::altrep::*;
        use miniextendr_api::altrep_bridge as bridge;

        // Install integer methods
        if <Self as AltInteger>::HAS_ELT {
            unsafe { R_set_altinteger_Elt_method(cls, Some(bridge::t_int_elt::<Self>)) };
        }
        if <Self as AltInteger>::HAS_GET_REGION {
            unsafe { R_set_altinteger_Get_region_method(cls, Some(bridge::t_int_get_region::<Self>)) };
        }
        if <Self as AltInteger>::HAS_IS_SORTED {
            unsafe { R_set_altinteger_Is_sorted_method(cls, Some(bridge::t_int_is_sorted::<Self>)) };
        }
        if <Self as AltInteger>::HAS_NO_NA {
            unsafe { R_set_altinteger_No_NA_method(cls, Some(bridge::t_int_no_na::<Self>)) };
        }
        if <Self as AltInteger>::HAS_SUM {
            unsafe { R_set_altinteger_Sum_method(cls, Some(bridge::t_int_sum::<Self>)) };
        }
        if <Self as AltInteger>::HAS_MIN {
            unsafe { R_set_altinteger_Min_method(cls, Some(bridge::t_int_min::<Self>)) };
        }
        if <Self as AltInteger>::HAS_MAX {
            unsafe { R_set_altinteger_Max_method(cls, Some(bridge::t_int_max::<Self>)) };
        }

        // Install ALTVEC methods (dataptr)
        if <Self as AltVec>::HAS_DATAPTR {
            unsafe { R_set_altvec_Dataptr_method(cls, Some(bridge::t_dataptr::<Self>)) };
        }
        if <Self as AltVec>::HAS_DATAPTR_OR_NULL {
            unsafe { R_set_altvec_Dataptr_or_null_method(cls, Some(bridge::t_dataptr_or_null::<Self>)) };
        }

        // Install serialization methods
        if <Self as Altrep>::HAS_SERIALIZED_STATE {
            unsafe { R_set_altrep_Serialized_state_method(cls, Some(bridge::t_serialized_state::<Self>)) };
        }
        // Note: Unserialize is handled by the wrapper class (LazyIntSeqClass) since
        // it needs to call LazyIntSeqClass::into_altrep() which we can't do from here.
    }
}

/// ALTREP wrapper for LazyIntSeqData - base type auto-inferred!
#[miniextendr(class = "LazyIntSeq", pkg = "rpkg")]
pub struct LazyIntSeqClass(LazyIntSeqData);

/// Create a lazy integer sequence (similar to R's seq())
/// Elements are computed on-demand; full buffer only allocated on DATAPTR access.
#[miniextendr]
pub fn lazy_int_seq(from: i32, to: i32, by: i32) -> SEXP {
    let len = if by == 0 {
        1
    } else {
        ((to - from) / by + 1).max(0) as usize
    };
    let data = LazyIntSeqData {
        start: from,
        step: by,
        len,
        materialized: None,
    };
    unsafe { LazyIntSeqClass::into_altrep(data) }
}

/// Check if a LazyIntSeq has been materialized
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn rpkg_lazy_int_seq_is_materialized(x: SEXP) -> SEXP {
    use miniextendr_api::ffi::{ALTREP, Rf_ScalarLogical};
    use miniextendr_api::altrep_data1_as;

    // Check if it's an ALTREP object
    if unsafe { ALTREP(x) } == 0 {
        return unsafe { Rf_ScalarLogical(0) }; // Not ALTREP
    }

    // Try to extract the data
    match unsafe { altrep_data1_as::<LazyIntSeqData>(x) } {
        Some(data) => {
            let is_mat = data.materialized.is_some();
            unsafe { Rf_ScalarLogical(if is_mat { 1 } else { 0 }) }
        }
        None => unsafe { Rf_ScalarLogical(0) },
    }
}

// -----------------------------------------------------------------------------
// ConstantLogical: All TRUE or all FALSE
// -----------------------------------------------------------------------------

#[derive(miniextendr_api::ExternalPtr)]
pub struct ConstantLogicalData {
    value: Logical,
    len: usize,
}

impl AltrepLen for ConstantLogicalData {
    fn len(&self) -> usize { self.len }
}

impl AltLogicalData for ConstantLogicalData {
    fn elt(&self, _i: usize) -> Logical { self.value }
    fn no_na(&self) -> Option<bool> {
        Some(!matches!(self.value, Logical::Na))
    }
}

miniextendr_api::impl_altlogical_from_data!(ConstantLogicalData);

#[miniextendr(class = "ConstantLogical", pkg = "rpkg", base = "Logical")]
pub struct ConstantLogicalClass(ConstantLogicalData);

#[miniextendr]
fn constant_logical(value: i32, n: i32) -> SEXP {
    let logical_value = match value {
        0 => Logical::False,
        i if i == i32::MIN => Logical::Na,
        _ => Logical::True,
    };
    let data = ConstantLogicalData { value: logical_value, len: n as usize };
    unsafe { ConstantLogicalClass::into_altrep(data) }
}

// -----------------------------------------------------------------------------
// LazyString: Lazily-generated strings
// -----------------------------------------------------------------------------

#[derive(miniextendr_api::ExternalPtr)]
pub struct LazyStringData {
    prefix: String,
    len: usize,
}

impl AltrepLen for LazyStringData {
    fn len(&self) -> usize { self.len }
}

impl AltStringData for LazyStringData {
    fn elt(&self, _i: usize) -> Option<&str> {
        // Note: For a real implementation you'd want to cache generated strings
        // Since we can't return a reference to a newly created String, return None
        // which triggers R's default behavior (NA)
        None
    }
    fn no_na(&self) -> Option<bool> { Some(false) } // We return None which is like NA
}

miniextendr_api::impl_altstring_from_data!(LazyStringData);

#[miniextendr(class = "LazyString", pkg = "rpkg", base = "String")]
pub struct LazyStringClass(LazyStringData);

#[miniextendr]
fn lazy_string(prefix: &str, n: i32) -> SEXP {
    let data = LazyStringData { prefix: prefix.to_string(), len: n as usize };
    unsafe { LazyStringClass::into_altrep(data) }
}

// -----------------------------------------------------------------------------
// RepeatingRaw: Repeating byte pattern
// -----------------------------------------------------------------------------

#[derive(miniextendr_api::ExternalPtr)]
pub struct RepeatingRawData {
    pattern: Vec<u8>,
    total_len: usize,
}

impl AltrepLen for RepeatingRawData {
    fn len(&self) -> usize { self.total_len }
}

impl AltRawData for RepeatingRawData {
    fn elt(&self, i: usize) -> u8 {
        if self.pattern.is_empty() { 0 }
        else { self.pattern[i % self.pattern.len()] }
    }
}

miniextendr_api::impl_altraw_from_data!(RepeatingRawData);

#[miniextendr(class = "RepeatingRaw", pkg = "rpkg", base = "Raw")]
pub struct RepeatingRawClass(RepeatingRawData);

#[miniextendr]
fn repeating_raw(pattern: &[u8], n: i32) -> SEXP {
    let data = RepeatingRawData { pattern: pattern.to_vec(), total_len: n as usize };
    unsafe { RepeatingRawClass::into_altrep(data) }
}

// -----------------------------------------------------------------------------
// UnitCircle: Complex numbers on the unit circle (e^(i*theta))
// This demonstrates ALTREP for complex vectors
// -----------------------------------------------------------------------------

use miniextendr_api::altrep_data::AltComplexData;
use miniextendr_api::ffi::Rcomplex;

#[derive(miniextendr_api::ExternalPtr)]
pub struct UnitCircleData {
    /// Number of points on the unit circle
    n: usize,
}

impl AltrepLen for UnitCircleData {
    fn len(&self) -> usize {
        self.n
    }
}

impl AltComplexData for UnitCircleData {
    fn elt(&self, i: usize) -> Rcomplex {
        // Generate e^(i * 2π * k/n) = cos(2πk/n) + i*sin(2πk/n)
        let theta = 2.0 * std::f64::consts::PI * (i as f64) / (self.n as f64);
        Rcomplex {
            r: theta.cos(),
            i: theta.sin(),
        }
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [Rcomplex]) -> usize {
        let end = (start + len).min(self.n);
        for (buf_i, i) in (start..end).enumerate() {
            buf[buf_i] = self.elt(i);
        }
        end - start
    }
}

miniextendr_api::impl_altcomplex_from_data!(UnitCircleData);

// Implement AltrepBase for auto-inference
impl miniextendr_api::altrep::AltrepBase for UnitCircleData {
    const BASE: miniextendr_api::altrep::RBase = miniextendr_api::altrep::RBase::Complex;
}

impl miniextendr_api::altrep_registration::AltrepInstaller for UnitCircleData {
    unsafe fn make_class(
        class_name: *const i8,
        pkg_name: *const i8,
    ) -> miniextendr_api::ffi::altrep::R_altrep_class_t {
        unsafe {
            miniextendr_api::ffi::altrep::R_make_altcomplex_class(
                class_name,
                pkg_name,
                core::ptr::null_mut(),
            )
        }
    }

    unsafe fn install_methods(cls: miniextendr_api::ffi::altrep::R_altrep_class_t) {
        use miniextendr_api::altrep_traits::AltComplex;
        use miniextendr_api::ffi::altrep::*;
        use miniextendr_api::altrep_bridge as bridge;

        if <Self as AltComplex>::HAS_ELT {
            unsafe { R_set_altcomplex_Elt_method(cls, Some(bridge::t_cplx_elt::<Self>)) };
        }
        if <Self as AltComplex>::HAS_GET_REGION {
            unsafe { R_set_altcomplex_Get_region_method(cls, Some(bridge::t_cplx_get_region::<Self>)) };
        }
    }
}

/// ALTREP wrapper for UnitCircleData - generates complex numbers on unit circle
#[miniextendr(class = "UnitCircle", pkg = "rpkg")]
pub struct UnitCircleClass(UnitCircleData);

/// Create complex numbers on the unit circle: e^(i * 2π * k/n) for k = 0, 1, ..., n-1
/// These are the n-th roots of unity, evenly spaced around the unit circle.
#[miniextendr]
pub fn unit_circle(n: i32) -> SEXP {
    let data = UnitCircleData { n: n as usize };
    unsafe { UnitCircleClass::into_altrep(data) }
}

// -----------------------------------------------------------------------------
// SimpleVecInt: Vec<i32> wrapper (simplest example)
// -----------------------------------------------------------------------------

#[miniextendr(class = "SimpleVecInt", pkg = "rpkg", base = "Int")]
pub struct SimpleVecIntClass(Vec<i32>);

#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn rpkg_simple_vec_int(x: SEXP) -> SEXP {
    use miniextendr_api::ffi::{INTEGER, Rf_xlength};
    let n = unsafe { Rf_xlength(x) } as usize;
    let src = unsafe { INTEGER(x) };
    let mut data = Vec::with_capacity(n);
    for i in 0..n {
        data.push(unsafe { *src.add(i) });
    }
    unsafe { SimpleVecIntClass::into_altrep(data) }
}

// -----------------------------------------------------------------------------
// InferredVecReal: Vec<f64> wrapper with base type inferred from inner type
// -----------------------------------------------------------------------------

/// Test case for auto-inferred base type (no explicit `base = "..."` attribute)
#[miniextendr(class = "InferredVecReal", pkg = "rpkg")]
pub struct InferredVecRealClass(Vec<f64>);

#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn rpkg_inferred_vec_real(x: SEXP) -> SEXP {
    use miniextendr_api::ffi::{REAL, Rf_xlength};
    let n = unsafe { Rf_xlength(x) } as usize;
    let src = unsafe { REAL(x) };
    let mut data = Vec::with_capacity(n);
    for i in 0..n {
        data.push(unsafe { *src.add(i) });
    }
    unsafe { InferredVecRealClass::into_altrep(data) }
}

// endregion

miniextendr_module! {
    mod rpkg;

    // Aggregate all test modules
    use panic_tests;
    use unwind_protect_tests;
    use dots_tests;
    use interrupt_tests;
    use conversion_tests;
    use externalptr_tests;
    use receiver_tests;
    use r6_tests;
    use s3_tests;
    use s7_tests;
    use s4_tests;
    use worker_tests;
    use coerce_tests;
    use visibility_tests;
    use thread_tests;
    use misc_tests;
    use nonapi;

    // ALTREP entrypoints are called directly from R via R/altrep.R

    fn add;
    fn add2;
    fn add3;
    fn add4;
    fn add_panic;
    fn add_r_error;

    fn add_panic_heap;
    fn add_r_error_heap;

    extern "C-unwind" fn C_unwind_protect_normal;
    extern "C-unwind" fn C_unwind_protect_r_error;
    extern "C-unwind" fn C_unwind_protect_lowlevel_test;

    fn add_left_mut;
    fn add_right_mut;
    fn add_left_right_mut;

    fn take_and_return_nothing;

    extern "C-unwind" fn C_just_panic;
    extern "C-unwind" fn C_panic_and_catch;

    fn drop_message_on_success;
    fn drop_on_panic;
    fn drop_on_panic_with_move;

    fn greetings_with_named_dots;
    fn greetings_with_named_and_unused_dots;
    fn greetings_with_nameless_dots;
    fn greetings_last_as_named_dots;
    fn greetings_last_as_named_and_unused_dots;
    fn greetings_last_as_nameless_dots;

    fn invisibly_return_no_arrow;
    fn invisibly_return_arrow;
    fn invisibly_option_return_none;
    fn invisibly_option_return_some;
    fn invisibly_result_return_ok;
    fn force_invisible_i32;
    fn force_visible_unit;
    fn with_interrupt_check;

    extern fn C_r_error;
    extern fn C_r_error_in_catch;
    extern fn C_r_error_in_thread;
    extern fn C_r_print_in_thread;

    extern fn C_check_interupt_after;
    extern fn C_check_interupt_unwind;

    // Worker thread tests (basic)
    extern "C-unwind" fn C_worker_drop_on_success;
    extern "C-unwind" fn C_worker_drop_on_panic;

    // Comprehensive worker/with_r_thread tests
    extern "C-unwind" fn C_test_worker_simple;
    extern "C-unwind" fn C_test_worker_with_r_thread;
    extern "C-unwind" fn C_test_worker_multiple_r_calls;
    extern "C-unwind" fn C_test_worker_panic_simple;
    extern "C-unwind" fn C_test_worker_panic_with_drops;
    extern "C-unwind" fn C_test_worker_panic_in_r_thread;
    extern "C-unwind" fn C_test_worker_panic_in_r_thread_with_drops;
    extern "C-unwind" fn C_test_worker_r_error_in_r_thread;
    extern "C-unwind" fn C_test_worker_r_error_with_drops;
    extern "C-unwind" fn C_test_worker_r_calls_then_error;
    extern "C-unwind" fn C_test_worker_r_calls_then_panic;
    fn test_worker_return_i32;
    fn test_worker_return_string;
    fn test_worker_return_f64;
    extern "C-unwind" fn C_test_extptr_from_worker;
    extern "C-unwind" fn C_test_multiple_extptrs_from_worker;
    fn test_main_thread_r_api;
    fn test_main_thread_r_error;
    fn test_main_thread_r_error_with_drops;
    extern "C-unwind" fn C_test_wrong_thread_r_api;

    // Nested wrapper tests
    extern "C-unwind" fn C_test_nested_helper_from_worker;
    extern "C-unwind" fn C_test_nested_multiple_helpers;
    extern "C-unwind" fn C_test_nested_with_r_thread;
    extern "C-unwind" fn C_test_call_worker_fn_from_main;
    extern "C-unwind" fn C_test_nested_worker_calls;
    extern "C-unwind" fn C_test_nested_with_error;
    extern "C-unwind" fn C_test_nested_with_panic;
    extern "C-unwind" fn C_test_deep_with_r_thread_sequence;

    // Scalar conversion tests
    fn test_i32_identity;
    fn test_i32_add_one;
    fn test_i32_sum;
    fn test_f64_identity;
    fn test_f64_add_one;
    fn test_f64_multiply;
    fn test_u8_identity;
    fn test_u8_add_one;
    fn test_logical_identity;
    fn test_logical_not;
    fn test_logical_and;
    fn test_i32_to_f64;
    fn test_f64_to_i32;

    // Slice conversion tests
    fn test_i32_slice_len;
    fn test_i32_slice_sum;
    fn test_i32_slice_first;
    fn test_i32_slice_last;
    fn test_f64_slice_len;
    fn test_f64_slice_sum;
    fn test_f64_slice_mean;
    fn test_u8_slice_len;
    fn test_u8_slice_sum;
    fn test_logical_slice_len;
    fn test_logical_slice_any_true;
    fn test_logical_slice_all_true;

    // Wildcard parameter test
    fn underscore_it_all;

    // Proc-macro ALTREP test: struct registers the class, fn creates instances
    struct ConstantIntClass;
    extern "C-unwind" fn rpkg_constant_int;

    // Additional ALTREP examples
    // Real ALTREP
    struct ConstantRealClass;
    extern "C-unwind" fn rpkg_constant_real;
    struct ArithSeqClass;
    fn arith_seq;

    // Lazy materialization ALTREP example
    struct LazyIntSeqClass;
    fn lazy_int_seq;
    extern "C-unwind" fn rpkg_lazy_int_seq_is_materialized;

    // Logical ALTREP
    struct ConstantLogicalClass;
    fn constant_logical;

    // String ALTREP
    struct LazyStringClass;
    fn lazy_string;

    // Raw ALTREP
    struct RepeatingRawClass;
    fn repeating_raw;

    // Complex ALTREP - unit circle (roots of unity)
    struct UnitCircleClass;
    fn unit_circle;

    // ExternalPtr tests
    fn extptr_counter_new;
    extern "C-unwind" fn C_extptr_counter_get;
    extern "C-unwind" fn C_extptr_counter_increment;
    fn extptr_point_new;
    extern "C-unwind" fn C_extptr_point_get_x;
    extern "C-unwind" fn C_extptr_point_get_y;
    extern "C-unwind" fn C_extptr_type_mismatch_test;
    extern "C-unwind" fn C_extptr_null_test;
    extern "C-unwind" fn C_extptr_is_counter;
    extern "C-unwind" fn C_extptr_is_point;

    // ALTREP with Vec<i32> backend - simplified API
    struct SimpleVecIntClass;
    extern "C-unwind" fn rpkg_simple_vec_int;

    // ALTREP with Vec<f64> backend - base type auto-inferred
    struct InferredVecRealClass;
    extern "C-unwind" fn rpkg_inferred_vec_real;
}
