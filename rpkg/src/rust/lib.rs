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

    fn sum(&self, _na_rm: bool) -> Option<i64> {
        if self.value == i32::MIN {
            // All elements are NA
            if _na_rm {
                Some(0) // sum of empty set after removing NAs
            } else {
                None // NA propagates
            }
        } else {
            Some(self.value as i64 * self.len as i64)
        }
    }
}

// Generate low-level traits from data traits (also enables base type inference)
miniextendr_api::impl_altinteger_from_data!(ConstantIntData);

/// ALTREP wrapper for ConstantIntData
#[miniextendr(class = "ConstantInt", pkg = "rpkg")]
pub struct ConstantIntClass(pub ConstantIntData);

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
    ConstantIntClass::into_altrep(data)
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

// region: Additional ALTREP examples

// =============================================================================
// Example 1: Real ALTREP - Constant value (all elements are PI)
// =============================================================================

use miniextendr_api::altrep_traits::AltReal;

/// A custom ALTREP real class: always returns PI.
#[miniextendr(class = "ConstantReal", pkg = "rpkg", base = "Real")]
pub struct ConstantRealClass;

impl Altrep for ConstantRealClass {
    const HAS_LENGTH: bool = true;
    fn length(_x: SEXP) -> R_xlen_t {
        10
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl AltVec for ConstantRealClass {
    const HAS_DATAPTR: bool = true;
    const HAS_DATAPTR_OR_NULL: bool = true;

    fn dataptr(x: SEXP, _writable: bool) -> *mut core::ffi::c_void {
        use miniextendr_api::ffi::{
            R_NilValue, R_altrep_data2, R_set_altrep_data2, REAL, Rf_allocVector, Rf_protect,
            Rf_unprotect, SEXPTYPE,
        };
        unsafe {
            let expanded = R_altrep_data2(x);
            if expanded == R_NilValue {
                let n = Self::length(x);
                let val = Rf_allocVector(SEXPTYPE::REALSXP, n);
                Rf_protect(val);
                let buf = REAL(val);
                for i in 0..n {
                    *buf.offset(i) = Self::elt(x, i);
                }
                R_set_altrep_data2(x, val);
                Rf_unprotect(1);
                buf.cast()
            } else {
                REAL(expanded).cast()
            }
        }
    }

    fn dataptr_or_null(x: SEXP) -> *const core::ffi::c_void {
        use miniextendr_api::ffi::{R_NilValue, R_altrep_data2, REAL};
        unsafe {
            let expanded = R_altrep_data2(x);
            if expanded == R_NilValue {
                core::ptr::null()
            } else {
                REAL(expanded).cast()
            }
        }
    }
}

impl AltReal for ConstantRealClass {
    const HAS_ELT: bool = true;
    fn elt(_x: SEXP, _i: R_xlen_t) -> f64 {
        std::f64::consts::PI
    }

    // Optimized sum: n * PI
    const HAS_SUM: bool = true;
    fn sum(x: SEXP, _narm: bool) -> SEXP {
        let n = Self::length(x) as f64;
        unsafe { miniextendr_api::ffi::Rf_ScalarReal(n * std::f64::consts::PI) }
    }
}

/// Create a ConstantReal ALTREP instance (all elements are PI, length 10).
///
/// # Safety
/// Must be called from R main thread.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn rpkg_constant_real() -> SEXP {
    use miniextendr_api::altrep_registration::RegisterAltrep;
    use miniextendr_api::ffi::altrep::R_new_altrep;
    let cls = ConstantRealClass::get_or_init_class();
    unsafe { R_new_altrep(cls, R_NilValue, R_NilValue) }
}

// =============================================================================
// Example 2: Real ALTREP - Arithmetic sequence (like R's seq())
// =============================================================================

/// Stores (start, step) for arithmetic sequence
#[derive(DeriveExternalPtr)]
struct ArithSeqData {
    start: f64,
    step: f64,
    len: i64,
}

/// ALTREP class for arithmetic sequences: start, start+step, start+2*step, ...
#[miniextendr(class = "ArithSeq", pkg = "rpkg", base = "Real")]
pub struct ArithSeqClass;

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl Altrep for ArithSeqClass {
    const HAS_LENGTH: bool = true;
    fn length(x: SEXP) -> R_xlen_t {
        use miniextendr_api::altrep_data1_as;
        match unsafe { altrep_data1_as::<ArithSeqData>(x) } {
            Some(data) => data.len as R_xlen_t,
            None => 0,
        }
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl AltVec for ArithSeqClass {}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl AltReal for ArithSeqClass {
    const HAS_ELT: bool = true;
    fn elt(x: SEXP, i: R_xlen_t) -> f64 {
        use miniextendr_api::altrep_data1_as;
        match unsafe { altrep_data1_as::<ArithSeqData>(x) } {
            Some(data) => data.start + (i as f64) * data.step,
            None => f64::NAN,
        }
    }

    // Optimized sum using arithmetic series formula: n/2 * (first + last)
    const HAS_SUM: bool = true;
    fn sum(x: SEXP, _narm: bool) -> SEXP {
        use miniextendr_api::altrep_data1_as;
        match unsafe { altrep_data1_as::<ArithSeqData>(x) } {
            Some(data) => {
                let n = data.len as f64;
                let first = data.start;
                let last = data.start + (n - 1.0) * data.step;
                let sum = n / 2.0 * (first + last);
                unsafe { miniextendr_api::ffi::Rf_ScalarReal(sum) }
            }
            None => unsafe { miniextendr_api::ffi::Rf_ScalarReal(f64::NAN) },
        }
    }

    // Optimized min/max for monotonic sequences
    const HAS_MIN: bool = true;
    fn min(x: SEXP, _narm: bool) -> SEXP {
        use miniextendr_api::altrep_data1_as;
        match unsafe { altrep_data1_as::<ArithSeqData>(x) } {
            Some(data) => {
                let min = if data.step >= 0.0 {
                    data.start
                } else {
                    data.start + ((data.len - 1) as f64) * data.step
                };
                unsafe { miniextendr_api::ffi::Rf_ScalarReal(min) }
            }
            None => unsafe { miniextendr_api::ffi::Rf_ScalarReal(f64::NAN) },
        }
    }

    const HAS_MAX: bool = true;
    fn max(x: SEXP, _narm: bool) -> SEXP {
        use miniextendr_api::altrep_data1_as;
        match unsafe { altrep_data1_as::<ArithSeqData>(x) } {
            Some(data) => {
                let max = if data.step >= 0.0 {
                    data.start + ((data.len - 1) as f64) * data.step
                } else {
                    data.start
                };
                unsafe { miniextendr_api::ffi::Rf_ScalarReal(max) }
            }
            None => unsafe { miniextendr_api::ffi::Rf_ScalarReal(f64::NAN) },
        }
    }

    const HAS_IS_SORTED: bool = true;
    fn is_sorted(x: SEXP) -> i32 {
        use miniextendr_api::altrep_data1_as;
        match unsafe { altrep_data1_as::<ArithSeqData>(x) } {
            Some(data) => {
                if data.step > 0.0 {
                    1 // SORTED_INCR
                } else if data.step < 0.0 {
                    -1 // SORTED_DECR
                } else {
                    1 // All same value = sorted
                }
            }
            None => 0, // UNKNOWN
        }
    }

    const HAS_NO_NA: bool = true;
    fn no_na(_x: SEXP) -> i32 {
        1 // Arithmetic sequences never have NA
    }
}

/// Create an ArithSeq ALTREP instance: seq(from, to, length.out)
#[miniextendr]
fn arith_seq(from: f64, to: f64, length_out: i32) -> SEXP {
    use miniextendr_api::altrep_registration::RegisterAltrep;
    use miniextendr_api::externalptr::ExternalPtr;
    use miniextendr_api::ffi::altrep::R_new_altrep;

    let len = length_out as i64;
    let step = if len > 1 {
        (to - from) / (len - 1) as f64
    } else {
        0.0
    };

    let ext_ptr = ExternalPtr::new(ArithSeqData {
        start: from,
        step,
        len,
    });

    let cls = ArithSeqClass::get_or_init_class();
    unsafe { R_new_altrep(cls, ext_ptr.as_sexp(), R_NilValue) }
}

// =============================================================================
// Example 3: Logical ALTREP - All TRUE or all FALSE
// =============================================================================

use miniextendr_api::altrep_traits::AltLogical;

/// Stores the constant value and length
#[derive(DeriveExternalPtr)]
struct ConstantLogicalData {
    value: i32, // TRUE=1, FALSE=0, NA=i32::MIN
    len: i64,
}

/// ALTREP class for constant logical vectors
#[miniextendr(class = "ConstantLogical", pkg = "rpkg", base = "Logical")]
pub struct ConstantLogicalClass;

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl Altrep for ConstantLogicalClass {
    const HAS_LENGTH: bool = true;
    fn length(x: SEXP) -> R_xlen_t {
        use miniextendr_api::altrep_data1_as;
        match unsafe { altrep_data1_as::<ConstantLogicalData>(x) } {
            Some(data) => data.len as R_xlen_t,
            None => 0,
        }
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl AltVec for ConstantLogicalClass {}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl AltLogical for ConstantLogicalClass {
    const HAS_ELT: bool = true;
    fn elt(x: SEXP, _i: R_xlen_t) -> i32 {
        use miniextendr_api::altrep_data1_as;
        match unsafe { altrep_data1_as::<ConstantLogicalData>(x) } {
            Some(data) => data.value,
            None => i32::MIN, // NA_LOGICAL
        }
    }

    // Optimized sum: n * value (for sum(TRUE_vec) = n, sum(FALSE_vec) = 0)
    const HAS_SUM: bool = true;
    fn sum(x: SEXP, narm: bool) -> SEXP {
        use miniextendr_api::altrep_data1_as;
        match unsafe { altrep_data1_as::<ConstantLogicalData>(x) } {
            Some(data) => {
                if data.value == i32::MIN {
                    // NA
                    if narm {
                        unsafe { miniextendr_api::ffi::Rf_ScalarInteger(0) }
                    } else {
                        unsafe { miniextendr_api::ffi::Rf_ScalarInteger(i32::MIN) }
                    }
                } else {
                    let sum = data.len as i32 * data.value;
                    unsafe { miniextendr_api::ffi::Rf_ScalarInteger(sum) }
                }
            }
            None => unsafe { miniextendr_api::ffi::Rf_ScalarInteger(i32::MIN) },
        }
    }

    const HAS_NO_NA: bool = true;
    fn no_na(x: SEXP) -> i32 {
        use miniextendr_api::altrep_data1_as;
        match unsafe { altrep_data1_as::<ConstantLogicalData>(x) } {
            Some(data) => {
                if data.value == i32::MIN {
                    0
                } else {
                    1
                }
            }
            None => 0,
        }
    }
}

/// Create a constant logical ALTREP: rep(value, n)
#[miniextendr]
fn constant_logical(value: i32, n: i32) -> SEXP {
    use miniextendr_api::altrep_registration::RegisterAltrep;
    use miniextendr_api::externalptr::ExternalPtr;
    use miniextendr_api::ffi::altrep::R_new_altrep;

    let ext_ptr = ExternalPtr::new(ConstantLogicalData {
        value,
        len: n as i64,
    });

    let cls = ConstantLogicalClass::get_or_init_class();
    unsafe { R_new_altrep(cls, ext_ptr.as_sexp(), R_NilValue) }
}

// =============================================================================
// Example 4: String ALTREP - Lazy-generated strings
// =============================================================================

use miniextendr_api::altrep_traits::AltString;

/// Generates strings like "item_0", "item_1", etc. on demand
#[derive(DeriveExternalPtr)]
struct LazyStringData {
    prefix: String,
    len: i64,
}

/// ALTREP class for lazily-generated strings
#[miniextendr(class = "LazyString", pkg = "rpkg", base = "String")]
pub struct LazyStringClass;

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl Altrep for LazyStringClass {
    const HAS_LENGTH: bool = true;
    fn length(x: SEXP) -> R_xlen_t {
        use miniextendr_api::altrep_data1_as;
        match unsafe { altrep_data1_as::<LazyStringData>(x) } {
            Some(data) => data.len as R_xlen_t,
            None => 0,
        }
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl AltVec for LazyStringClass {}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl AltString for LazyStringClass {
    const HAS_ELT: bool = true;
    fn elt(x: SEXP, i: R_xlen_t) -> SEXP {
        use miniextendr_api::altrep_data1_as;
        use miniextendr_api::ffi::{Rf_mkCharLenCE, cetype_t};
        match unsafe { altrep_data1_as::<LazyStringData>(x) } {
            Some(data) => {
                let s = format!("{}_{}", data.prefix, i);
                unsafe { Rf_mkCharLenCE(s.as_ptr().cast(), s.len() as i32, cetype_t::CE_UTF8) }
            }
            None => unsafe { miniextendr_api::ffi::R_NaString },
        }
    }

    const HAS_NO_NA: bool = true;
    fn no_na(_x: SEXP) -> i32 {
        1 // Generated strings are never NA
    }
}

/// Create a LazyString ALTREP: generates "prefix_0", "prefix_1", ... on demand
#[miniextendr]
fn lazy_string(prefix: &str, n: i32) -> SEXP {
    use miniextendr_api::altrep_registration::RegisterAltrep;
    use miniextendr_api::externalptr::ExternalPtr;
    use miniextendr_api::ffi::altrep::R_new_altrep;

    let ext_ptr = ExternalPtr::new(LazyStringData {
        prefix: prefix.to_string(),
        len: n as i64,
    });

    let cls = LazyStringClass::get_or_init_class();
    unsafe { R_new_altrep(cls, ext_ptr.as_sexp(), R_NilValue) }
}

// =============================================================================
// Example 5: Raw ALTREP - Repeating byte pattern
// =============================================================================

use miniextendr_api::altrep_traits::AltRaw;

/// Repeating pattern of bytes
#[derive(DeriveExternalPtr)]
struct RepeatingRawData {
    pattern: Vec<u8>,
    total_len: i64,
}

/// ALTREP class for repeating raw byte patterns
#[miniextendr(class = "RepeatingRaw", pkg = "rpkg", base = "Raw")]
pub struct RepeatingRawClass;

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl Altrep for RepeatingRawClass {
    const HAS_LENGTH: bool = true;
    fn length(x: SEXP) -> R_xlen_t {
        use miniextendr_api::altrep_data1_as;
        match unsafe { altrep_data1_as::<RepeatingRawData>(x) } {
            Some(data) => data.total_len as R_xlen_t,
            None => 0,
        }
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl AltVec for RepeatingRawClass {}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl AltRaw for RepeatingRawClass {
    const HAS_ELT: bool = true;
    fn elt(x: SEXP, i: R_xlen_t) -> u8 {
        use miniextendr_api::altrep_data1_as;
        match unsafe { altrep_data1_as::<RepeatingRawData>(x) } {
            Some(data) => {
                if data.pattern.is_empty() {
                    0
                } else {
                    data.pattern[i as usize % data.pattern.len()]
                }
            }
            None => 0,
        }
    }
}

/// Create a RepeatingRaw ALTREP: repeats pattern to fill n bytes
#[miniextendr]
fn repeating_raw(pattern: &[u8], n: i32) -> SEXP {
    use miniextendr_api::altrep_registration::RegisterAltrep;
    use miniextendr_api::externalptr::ExternalPtr;
    use miniextendr_api::ffi::altrep::R_new_altrep;

    let ext_ptr = ExternalPtr::new(RepeatingRawData {
        pattern: pattern.to_vec(),
        total_len: n as i64,
    });

    let cls = RepeatingRawClass::get_or_init_class();
    unsafe { R_new_altrep(cls, ext_ptr.as_sexp(), R_NilValue) }
}

// =============================================================================
// Example 6: List ALTREP - Lazy list of numbered lists
// =============================================================================

use miniextendr_api::altrep_traits::AltList;

/// Generates list elements on demand
#[derive(DeriveExternalPtr)]
struct LazyListData {
    len: i64,
}

/// ALTREP class for lazily-generated lists
#[miniextendr(class = "LazyList", pkg = "rpkg", base = "List")]
pub struct LazyListClass;

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl Altrep for LazyListClass {
    const HAS_LENGTH: bool = true;
    fn length(x: SEXP) -> R_xlen_t {
        use miniextendr_api::altrep_data1_as;
        match unsafe { altrep_data1_as::<LazyListData>(x) } {
            Some(data) => data.len as R_xlen_t,
            None => 0,
        }
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl AltVec for LazyListClass {}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl AltList for LazyListClass {
    const HAS_ELT: bool = true;
    fn elt(_x: SEXP, i: R_xlen_t) -> SEXP {
        // Return a list with index info: list(index = i, squared = i*i)
        use miniextendr_api::ffi::{
            R_NamesSymbol, Rf_allocVector, Rf_protect, Rf_setAttrib, Rf_unprotect, SET_VECTOR_ELT,
            SEXPTYPE,
        };
        unsafe {
            let result = Rf_allocVector(SEXPTYPE::VECSXP, 2);
            Rf_protect(result);

            SET_VECTOR_ELT(result, 0, miniextendr_api::ffi::Rf_ScalarInteger(i as i32));
            SET_VECTOR_ELT(
                result,
                1,
                miniextendr_api::ffi::Rf_ScalarInteger((i * i) as i32),
            );

            // Set names
            let names = Rf_allocVector(SEXPTYPE::STRSXP, 2);
            Rf_protect(names);
            miniextendr_api::ffi::SET_STRING_ELT(
                names,
                0,
                miniextendr_api::ffi::Rf_mkChar(c"index".as_ptr()),
            );
            miniextendr_api::ffi::SET_STRING_ELT(
                names,
                1,
                miniextendr_api::ffi::Rf_mkChar(c"squared".as_ptr()),
            );
            Rf_setAttrib(result, R_NamesSymbol, names);

            Rf_unprotect(2);
            result
        }
    }
}

/// Create a LazyList ALTREP: each element is list(index=i, squared=i*i)
#[miniextendr]
fn lazy_list(n: i32) -> SEXP {
    use miniextendr_api::altrep_registration::RegisterAltrep;
    use miniextendr_api::externalptr::ExternalPtr;
    use miniextendr_api::ffi::altrep::R_new_altrep;

    let ext_ptr = ExternalPtr::new(LazyListData { len: n as i64 });

    let cls = LazyListClass::get_or_init_class();
    unsafe { R_new_altrep(cls, ext_ptr.as_sexp(), R_NilValue) }
}

// =============================================================================
// Example 7: Integer ALTREP - Fibonacci sequence with memoization
// =============================================================================

use std::cell::RefCell;

/// Fibonacci data with memoization cache
#[derive(DeriveExternalPtr)]
struct FibonacciData {
    len: i64,
    cache: RefCell<Vec<Option<i32>>>,
}

/// ALTREP class for Fibonacci sequence with memoization
#[miniextendr(class = "Fibonacci", pkg = "rpkg", base = "Int")]
pub struct FibonacciClass;

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl Altrep for FibonacciClass {
    const HAS_LENGTH: bool = true;
    fn length(x: SEXP) -> R_xlen_t {
        use miniextendr_api::altrep_data1_as;
        match unsafe { altrep_data1_as::<FibonacciData>(x) } {
            Some(data) => data.len as R_xlen_t,
            None => 0,
        }
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl AltVec for FibonacciClass {}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl AltInteger for FibonacciClass {
    const HAS_ELT: bool = true;
    fn elt(x: SEXP, i: R_xlen_t) -> i32 {
        use miniextendr_api::altrep_data1_as;

        fn fib(n: usize, cache: &RefCell<Vec<Option<i32>>>) -> i32 {
            if n <= 1 {
                return n as i32;
            }

            // Check cache
            {
                let c = cache.borrow();
                if let Some(&Some(v)) = c.get(n) {
                    return v;
                }
            }

            // Compute iteratively to avoid stack overflow
            let mut c = cache.borrow_mut();
            while c.len() <= n {
                c.push(None);
            }

            if c[0].is_none() {
                c[0] = Some(0);
            }
            if n >= 1 && c[1].is_none() {
                c[1] = Some(1);
            }

            for idx in 2..=n {
                if c[idx].is_none() {
                    let a = c[idx - 1].unwrap_or(0);
                    let b = c[idx - 2].unwrap_or(0);
                    c[idx] = Some(a.saturating_add(b));
                }
            }

            c[n].unwrap_or(0)
        }

        match unsafe { altrep_data1_as::<FibonacciData>(x) } {
            Some(data) => fib(i as usize, &data.cache),
            None => i32::MIN,
        }
    }

    const HAS_NO_NA: bool = true;
    fn no_na(_x: SEXP) -> i32 {
        1 // Fibonacci values are never NA
    }
}

/// Create a Fibonacci ALTREP: fib(0), fib(1), ..., fib(n-1)
#[miniextendr]
fn fibonacci(n: i32) -> SEXP {
    use miniextendr_api::altrep_registration::RegisterAltrep;
    use miniextendr_api::externalptr::ExternalPtr;
    use miniextendr_api::ffi::altrep::R_new_altrep;

    let ext_ptr = ExternalPtr::new(FibonacciData {
        len: n as i64,
        cache: RefCell::new(Vec::new()),
    });

    let cls = FibonacciClass::get_or_init_class();
    unsafe { R_new_altrep(cls, ext_ptr.as_sexp(), R_NilValue) }
}

// =============================================================================
// Example 8: Integer ALTREP - Powers of 2
// =============================================================================

/// ALTREP class for powers of 2: 1, 2, 4, 8, 16, ...
#[miniextendr(class = "PowersOf2", pkg = "rpkg", base = "Int")]
pub struct PowersOf2Class;

impl Altrep for PowersOf2Class {
    const HAS_LENGTH: bool = true;
    fn length(_x: SEXP) -> R_xlen_t {
        31 // 2^0 to 2^30 fit in i32
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl AltVec for PowersOf2Class {}

impl AltInteger for PowersOf2Class {
    const HAS_ELT: bool = true;
    fn elt(_x: SEXP, i: R_xlen_t) -> i32 {
        if i >= 31 {
            i32::MIN // NA for overflow
        } else {
            1 << i
        }
    }

    // Optimized sum: 2^n - 1 (sum of geometric series)
    const HAS_SUM: bool = true;
    fn sum(_x: SEXP, _narm: bool) -> SEXP {
        // Sum of 2^0 + 2^1 + ... + 2^30 = 2^31 - 1
        let sum = (1i64 << 31) - 1;
        unsafe { miniextendr_api::ffi::Rf_ScalarReal(sum as f64) }
    }

    const HAS_IS_SORTED: bool = true;
    fn is_sorted(_x: SEXP) -> i32 {
        1 // Always sorted ascending
    }

    const HAS_NO_NA: bool = true;
    fn no_na(_x: SEXP) -> i32 {
        1
    }
}

/// Create a PowersOf2 ALTREP: 1, 2, 4, 8, ..., 2^30
///
/// # Safety
/// Must be called from R main thread.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn rpkg_powers_of_2() -> SEXP {
    use miniextendr_api::altrep_registration::RegisterAltrep;
    use miniextendr_api::ffi::altrep::R_new_altrep;
    let cls = PowersOf2Class::get_or_init_class();
    unsafe { R_new_altrep(cls, R_NilValue, R_NilValue) }
}

// endregion

// region: ALTREP with ExternalPtr backend

/// An ALTREP integer class that stores its data in an ExternalPtr
#[derive(DeriveExternalPtr)]
struct VecIntData {
    data: Vec<i32>,
}

/// ALTREP class using ExternalPtr for storage
#[miniextendr(class = "VecIntAltrep", pkg = "rpkg", base = "Int")]
pub struct VecIntAltrepClass;

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl Altrep for VecIntAltrepClass {
    const HAS_LENGTH: bool = true;
    fn length(x: SEXP) -> R_xlen_t {
        use miniextendr_api::altrep_data1_as;
        match unsafe { altrep_data1_as::<VecIntData>(x) } {
            Some(ext) => ext.data.len() as R_xlen_t,
            None => 0,
        }
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl AltVec for VecIntAltrepClass {
    const HAS_DATAPTR: bool = true;
    const HAS_DATAPTR_OR_NULL: bool = true;

    fn dataptr(x: SEXP, _writable: bool) -> *mut core::ffi::c_void {
        use miniextendr_api::altrep_data1_mut;
        match unsafe { altrep_data1_mut::<VecIntData>(x) } {
            Some(vec_data) => vec_data.data.as_mut_ptr().cast(),
            None => core::ptr::null_mut(),
        }
    }

    fn dataptr_or_null(x: SEXP) -> *const core::ffi::c_void {
        use miniextendr_api::altrep_data1_as;
        match unsafe { altrep_data1_as::<VecIntData>(x) } {
            Some(ext) => ext.data.as_ptr().cast(),
            None => core::ptr::null(),
        }
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl AltInteger for VecIntAltrepClass {
    const HAS_ELT: bool = true;
    fn elt(x: SEXP, i: R_xlen_t) -> i32 {
        use miniextendr_api::altrep_data1_as;
        match unsafe { altrep_data1_as::<VecIntData>(x) } {
            Some(ext) => ext.data.get(i as usize).copied().unwrap_or(i32::MIN),
            None => i32::MIN,
        }
    }
}

/// Create a VecIntAltrep instance from an integer vector
///
/// # Safety
///
/// Must be called from R main thread with valid SEXP.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn rpkg_vec_int_altrep(x: SEXP) -> SEXP {
    use miniextendr_api::altrep_registration::RegisterAltrep;
    use miniextendr_api::externalptr::ExternalPtr;
    use miniextendr_api::ffi::altrep::R_new_altrep;
    use miniextendr_api::ffi::{INTEGER, Rf_xlength};
    let n = unsafe { Rf_xlength(x) } as usize;
    let src = unsafe { INTEGER(x) };
    let mut data = Vec::with_capacity(n);
    for i in 0..n {
        data.push(unsafe { *src.add(i) });
    }
    SimpleVecIntClass::into_altrep(data)
}

// -----------------------------------------------------------------------------
// SimpleVecString: Vec<Option<String>> wrapper (preserves NA)
// -----------------------------------------------------------------------------

#[derive(miniextendr_api::ExternalPtr)]
pub struct StringVecData {
    data: Vec<Option<String>>,
}

impl AltrepLen for StringVecData {
    fn len(&self) -> usize {
        self.data.len()
    }
}

impl AltStringData for StringVecData {
    fn elt(&self, i: usize) -> Option<&str> {
        self.data[i].as_deref()
    }

    fn no_na(&self) -> Option<bool> {
        Some(!self.data.iter().any(|v| v.is_none()))
    }
}

miniextendr_api::impl_altstring_from_data!(StringVecData);

#[miniextendr(class = "SimpleVecString", pkg = "rpkg")]
pub struct SimpleVecStringClass(pub StringVecData);

/// # Safety
/// Caller must ensure `x` is a valid character SEXP and this is called from R's main thread.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn rpkg_altrep_from_strings(x: SEXP) -> SEXP {
    let data: Vec<Option<String>> = TryFromSexp::try_from_sexp(x)
        .unwrap_or_else(|err| miniextendr_api::r_error!("altrep_from_strings: {err}"));
    SimpleVecStringClass::into_altrep(StringVecData { data })
}

// -----------------------------------------------------------------------------
// SimpleVecRaw: Vec<u8> wrapper
// -----------------------------------------------------------------------------

#[miniextendr(class = "SimpleVecRaw", pkg = "rpkg")]
pub struct SimpleVecRawClass(pub Vec<u8>);

/// # Safety
/// Caller must ensure `x` is a valid raw SEXP and this is called from R's main thread.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn rpkg_altrep_from_raw(x: SEXP) -> SEXP {
    use miniextendr_api::ffi::{RAW, Rf_xlength};

    let n = unsafe { Rf_xlength(x) } as usize;
    let src = unsafe { RAW(x) };
    let mut data = Vec::with_capacity(n);
    for i in 0..n {
        data.push(unsafe { *src.add(i) });
    }
    SimpleVecRawClass::into_altrep(data)
}

// -----------------------------------------------------------------------------
// InferredVecReal: Vec<f64> wrapper with base type inferred from inner type
// -----------------------------------------------------------------------------

/// Test case for auto-inferred base type (no explicit `base = "..."` attribute)
#[miniextendr(class = "InferredVecReal", pkg = "rpkg")]
pub struct InferredVecRealClass(pub Vec<f64>);

/// .
///
/// # Safety
///
/// .
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
    InferredVecRealClass::into_altrep(data)
}

/// # Safety
/// Caller must ensure `x` is a valid real SEXP and this is called from R's main thread.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn rpkg_altrep_from_doubles(x: SEXP) -> SEXP {
    // Reuse the existing Vec<f64> ALTREP constructor.
    unsafe { rpkg_inferred_vec_real(x) }
}

// -----------------------------------------------------------------------------
// BoxedInts: Box<[i32]> wrapper (owned slice example)
// -----------------------------------------------------------------------------

/// ALTREP class wrapping a Box<[i32]> - fixed-size heap allocation
#[miniextendr(class = "BoxedInts", pkg = "rpkg")]
pub struct BoxedIntsClass(pub Box<[i32]>);

/// Create an ALTREP backed by a boxed slice.
/// More memory-efficient than Vec when size is known upfront.
#[miniextendr]
pub fn boxed_ints(n: i32) -> SEXP {
    let data: Box<[i32]> = (1..=n).collect::<Vec<_>>().into_boxed_slice();
    BoxedIntsClass::into_altrep(data)
}

// region: StaticInts: &'static [i32] wrapper (static slice example)

/// Static data that lives for the entire program lifetime
/// 
/// Data to showcase functionality
static STATIC_INTS: [i32; 5] = [10, 20, 30, 40, 50];

/// ALTREP class wrapping a static slice - demonstrates `&'static [T]` support
#[miniextendr(class = "StaticInts", pkg = "rpkg")]
pub struct StaticIntsClass(pub &'static [i32]);

/// Create an ALTREP backed by static data.
/// This data lives in the binary and never needs to be freed.
#[miniextendr]
pub fn static_ints() -> SEXP {
    StaticIntsClass::into_altrep(&STATIC_INTS[..])
}

/// Create an ALTREP backed by leaked heap data (intentional memory leak).
/// Useful when you need dynamic data with 'static lifetime.
#[miniextendr]
pub fn leaked_ints(n: i32) -> SEXP {
    // Create data and leak it to get 'static lifetime
    let data: Vec<i32> = (1..=n).collect();
    let leaked: &'static [i32] = Box::leak(data.into_boxed_slice());
    StaticIntsClass::into_altrep(leaked)
}

// endregion

// -----------------------------------------------------------------------------
// StaticStrings: &'static [&'static str] wrapper
// -----------------------------------------------------------------------------

/// Static string data
/// 
/// Data to showcase functionality
static STATIC_STRINGS: [&str; 4] = ["alpha", "beta", "gamma", "delta"];

/// ALTREP class wrapping static string slices
#[miniextendr(class = "StaticStrings", pkg = "rpkg")]
pub struct StaticStringsClass(pub &'static [&'static str]);

/// Create a string ALTREP backed by static data.
#[miniextendr]
pub fn static_strings() -> SEXP {
    StaticStringsClass::into_altrep(&STATIC_STRINGS[..])
}

// endregion

// -----------------------------------------------------------------------------
// ListData: list-backed ALTREP (stores original list SEXP)
// -----------------------------------------------------------------------------

#[derive(miniextendr_api::ExternalPtr)]
pub struct ListData {
    list: SEXP,
    len: usize,
}

impl Drop for ListData {
    fn drop(&mut self) {
        unsafe {
            if self.list != miniextendr_api::ffi::R_NilValue {
                miniextendr_api::ffi::R_ReleaseObject(self.list);
            }
        }
    }
}

impl AltrepLen for ListData {
    fn len(&self) -> usize {
        self.len
    }
}

impl AltListData for ListData {
    fn elt(&self, i: usize) -> SEXP {
        unsafe { miniextendr_api::ffi::VECTOR_ELT(self.list, i as miniextendr_api::ffi::R_xlen_t) }
    }
}

miniextendr_api::impl_altlist_from_data!(ListData);

#[miniextendr(class = "ListData", pkg = "rpkg")]
pub struct ListDataClass(pub ListData);

/// # Safety
/// Caller must ensure `x` is a valid list SEXP and this is called from R's main thread.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn rpkg_altrep_from_list(x: SEXP) -> SEXP {
    use miniextendr_api::ffi::{Rf_xlength, R_NilValue, R_PreserveObject, SEXPTYPE, TYPEOF};

    if unsafe { TYPEOF(x) } != SEXPTYPE::VECSXP {
        miniextendr_api::r_error!("altrep_from_list: expected a list (VECSXP)");
    }

    if x != unsafe { R_NilValue } {
        unsafe { R_PreserveObject(x) };
    }

    let len = unsafe { Rf_xlength(x) } as usize;
    let data = ListData { list: x, len };
    ListDataClass::into_altrep(data)
}

// region: Nonapi module for lean-stack thread tests

#[cfg(feature = "nonapi")]
mod nonapi {
    use miniextendr_api::ffi::SEXP;
    use miniextendr_api::thread::{StackCheckGuard, spawn_with_r};
    use miniextendr_api::{miniextendr, miniextendr_module};

    /// Test spawn_with_r with lean stack (8 MiB) enabled by StackCheckGuard.
    #[miniextendr]
    #[unsafe(no_mangle)]
    #[allow(non_snake_case)]
    pub unsafe extern "C-unwind" fn C_test_spawn_with_r_lean_stack() -> SEXP {
        let handle = spawn_with_r(|| {
            let sexp = unsafe { miniextendr_api::ffi::Rf_ScalarInteger_unchecked(999) };
            unsafe { *miniextendr_api::ffi::INTEGER(sexp) }
        })
        .expect("failed to spawn");

        let result = handle.join().expect("thread panicked");
        unsafe { miniextendr_api::ffi::Rf_ScalarInteger(result) }
    }

    /// Test StackCheckGuard with Rust's default 2 MiB stack.
    #[miniextendr]
    #[unsafe(no_mangle)]
    #[allow(non_snake_case)]
    pub unsafe extern "C-unwind" fn C_test_stack_check_guard_lean() -> SEXP {
        let handle = std::thread::spawn(|| {
            let _guard = StackCheckGuard::disable();
            let sexp = unsafe { miniextendr_api::ffi::Rf_ScalarInteger_unchecked(777) };
            unsafe { *miniextendr_api::ffi::INTEGER(sexp) }
        });

        let result = handle.join().expect("thread panicked");
        unsafe { miniextendr_api::ffi::Rf_ScalarInteger(result) }
    }

    miniextendr_module! {
        mod nonapi;
        extern "C-unwind" fn C_test_spawn_with_r_lean_stack;
        extern "C-unwind" fn C_test_stack_check_guard_lean;
    }
}

#[cfg(not(feature = "nonapi"))]
mod nonapi {
    use miniextendr_api::miniextendr_module;

    miniextendr_module! {
        mod nonapi;
    }
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

    // Logical ALTREP
    struct ConstantLogicalClass;
    fn constant_logical;

    // String ALTREP
    struct LazyStringClass;
    fn lazy_string;

    // Raw ALTREP
    struct RepeatingRawClass;
    fn repeating_raw;

    // List ALTREP
    struct LazyListClass;
    fn lazy_list;

    // More Integer ALTREP examples
    struct FibonacciClass;
    fn fibonacci;
    struct PowersOf2Class;
    extern "C-unwind" fn rpkg_powers_of_2;

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

    // Lazy materialization ALTREP example
    struct LazyIntSeqClass;
    fn lazy_int_seq;
    extern "C-unwind" fn rpkg_lazy_int_seq_is_materialized;

    // Logical ALTREP
    struct ConstantLogicalClass;
    fn constant_logical;
    struct LogicalVecClass;

    // String ALTREP
    struct LazyStringClass;
    fn lazy_string;
    struct SimpleVecStringClass;

    // Raw ALTREP
    struct RepeatingRawClass;
    fn repeating_raw;
    struct SimpleVecRawClass;

    // Complex ALTREP - unit circle (roots of unity)
    struct UnitCircleClass;
    fn unit_circle;

    // ALTREP with Vec<i32> backend - simplified API
    struct SimpleVecIntClass;
    extern "C-unwind" fn rpkg_simple_vec_int;

    // ALTREP with Vec<f64> backend - base type auto-inferred
    struct InferredVecRealClass;
    extern "C-unwind" fn rpkg_inferred_vec_real;

    // List ALTREP
    struct ListDataClass;

    // Box<[T]> ALTREP example
    struct BoxedIntsClass;
    fn boxed_ints;

    // Static slice ALTREP examples
    struct StaticIntsClass;
    fn static_ints;
    fn leaked_ints;
    struct StaticStringsClass;
    fn static_strings;
}
