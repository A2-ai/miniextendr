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

use miniextendr_api::IntoR;
use miniextendr_api::ffi::SEXP;
use miniextendr_api::from_r::TryFromSexp;
use miniextendr_api::{miniextendr, miniextendr_module};

// Test modules
mod class_system_matrix;
mod coerce_tests;
mod conversion_tests;
mod convert_pref_tests;
mod default_tests;
mod dots_tests;
mod externalptr_tests;
mod identical_tests;
mod interrupt_tests;
mod misc_tests;
mod panic_tests;
mod r6_default_tests;
mod r6_tests;
mod receiver_tests;
mod rng_tests;
mod s3_tests;
mod s4_tests;
mod s7_tests;
mod shared_trait_test;
mod thread_tests;
mod trait_abi_tests;
mod unwind_protect_tests;
mod visibility_tests;
mod worker_tests;

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
    ConstantIntClass(data).into_sexp()
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

use miniextendr_api::altrep_data::{
    AltListData, AltLogicalData, AltRawData, AltRealData, AltStringData, Logical,
};

// -----------------------------------------------------------------------------
// ConstantReal: All elements are PI
// -----------------------------------------------------------------------------

#[derive(miniextendr_api::ExternalPtr)]
pub struct ConstantRealData {
    value: f64,
    len: usize,
}

impl AltrepLen for ConstantRealData {
    fn len(&self) -> usize {
        self.len
    }
}

impl AltRealData for ConstantRealData {
    fn elt(&self, _i: usize) -> f64 {
        self.value
    }
    fn no_na(&self) -> Option<bool> {
        Some(!self.value.is_nan())
    }
}

miniextendr_api::impl_altreal_from_data!(ConstantRealData);

#[miniextendr(class = "ConstantReal", pkg = "rpkg")]
pub struct ConstantRealClass(pub ConstantRealData);

/// # Safety
/// Caller must ensure this is called from R's main thread.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn rpkg_constant_real() -> SEXP {
    let data = ConstantRealData {
        value: std::f64::consts::PI,
        len: 10,
    };
    ConstantRealClass(data).into_sexp()
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
    fn len(&self) -> usize {
        self.len
    }
}

impl AltRealData for ArithSeqData {
    fn elt(&self, i: usize) -> f64 {
        self.start + (i as f64) * self.step
    }
    fn no_na(&self) -> Option<bool> {
        Some(true)
    }
}

miniextendr_api::impl_altreal_from_data!(ArithSeqData);

#[miniextendr(class = "ArithSeq", pkg = "rpkg")]
pub struct ArithSeqClass(pub ArithSeqData);

#[miniextendr]
fn arith_seq(from: f64, to: f64, length_out: i32) -> SEXP {
    let len = length_out as usize;
    let step = if len > 1 {
        (to - from) / (len - 1) as f64
    } else {
        0.0
    };
    let data = ArithSeqData {
        start: from,
        step,
        len,
    };
    ArithSeqClass(data).into_sexp()
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
        self.start
            .saturating_add((i as i32).saturating_mul(self.step))
    }

    fn no_na(&self) -> Option<bool> {
        // Check if any element would be NA (i32::MIN)
        // This is a conservative check - we know the formula
        Some(true)
    }

    fn is_sorted(&self) -> Option<miniextendr_api::altrep_data::Sortedness> {
        use miniextendr_api::altrep_data::Sortedness;
        if self.step < 0 {
            Some(Sortedness::Decreasing)
        } else {
            // step == 0 (all same) or step > 0 are both non-decreasing
            Some(Sortedness::Increasing)
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
                .map(|i| {
                    self.start
                        .saturating_add((i as i32).saturating_mul(self.step))
                })
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
            use miniextendr_api::ffi::{Rf_allocVector, SET_INTEGER_ELT, SEXPTYPE};
            let state = Rf_allocVector(SEXPTYPE::INTSXP, 3);
            SET_INTEGER_ELT(state, 0, self.start);
            SET_INTEGER_ELT(state, 1, self.step);
            SET_INTEGER_ELT(state, 2, self.len as i32);
            state
        }
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
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

/// ALTREP wrapper for LazyIntSeqData - base type auto-inferred!
#[miniextendr(class = "LazyIntSeq", pkg = "rpkg")]
pub struct LazyIntSeqClass(pub LazyIntSeqData);

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
    LazyIntSeqClass(data).into_sexp()
}

/// Check if a LazyIntSeq has been materialized
///
/// # Safety
/// Caller must ensure `x` is a valid SEXP and this is called from R's main thread.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn rpkg_lazy_int_seq_is_materialized(x: SEXP) -> SEXP {
    use miniextendr_api::altrep_data1_as;
    use miniextendr_api::ffi::{ALTREP, Rf_ScalarLogical};

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

/// Create a compact integer sequence with explicit length.
///
/// This is the entrypoint used by R/altrep.R for integer ALTREP tests.
///
/// # Safety
/// Caller must ensure this is called from R's main thread.
/// @title ALTREP Unsafe Entry Points
/// @name rpkg_altrep_unsafe
/// @keywords internal
/// @description ALTREP low-level entry points (unsafe)
/// @examples \dontrun{
/// x <- unsafe_rpkg_altrep_from_doubles(c(1, 2, 3))
/// unsafe_rpkg_lazy_int_seq_is_materialized(x)
/// }
/// @aliases unsafe_rpkg_altrep_compact_int unsafe_rpkg_altrep_from_doubles
///   unsafe_rpkg_altrep_from_strings unsafe_rpkg_altrep_from_logicals
///   unsafe_rpkg_altrep_from_raw unsafe_rpkg_altrep_from_list
///   unsafe_rpkg_constant_int unsafe_rpkg_constant_real
///   unsafe_rpkg_simple_vec_int unsafe_rpkg_inferred_vec_real
///   unsafe_rpkg_lazy_int_seq_is_materialized
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn rpkg_altrep_compact_int(n: SEXP, start: SEXP, step: SEXP) -> SEXP {
    let n: i32 = TryFromSexp::try_from_sexp(n)
        .unwrap_or_else(|err| miniextendr_api::r_error!("altrep_compact_int: n: {err}"));
    let start: i32 = TryFromSexp::try_from_sexp(start)
        .unwrap_or_else(|err| miniextendr_api::r_error!("altrep_compact_int: start: {err}"));
    let step: i32 = TryFromSexp::try_from_sexp(step)
        .unwrap_or_else(|err| miniextendr_api::r_error!("altrep_compact_int: step: {err}"));

    if n == i32::MIN || start == i32::MIN || step == i32::MIN {
        miniextendr_api::r_error!("altrep_compact_int: n/start/step cannot be NA");
    }
    if n < 0 {
        miniextendr_api::r_error!("altrep_compact_int: n must be >= 0");
    }

    let len = if n == 0 { 0 } else { n as usize };
    let data = LazyIntSeqData {
        start,
        step,
        len,
        materialized: None,
    };
    LazyIntSeqClass(data).into_sexp()
}

// -----------------------------------------------------------------------------
// ALTREP helpers (internal R-facing wrappers)
// -----------------------------------------------------------------------------

/// @title ALTREP Helpers
/// @name rpkg_altrep_helpers
/// @keywords internal
/// @description ALTREP convenience wrappers (internal)
/// @examples
/// \dontrun{
/// x <- altrep_compact_int(5L, 1L, 2L)
/// y <- altrep_from_doubles(c(1, 2, 3))
/// z <- altrep_from_strings(c("a", "b"))
/// altrep_lazy_int_seq_is_materialized(lazy_int_seq(1L, 5L, 1L))
/// }
#[miniextendr(unsafe(main_thread))]
fn altrep_compact_int(n: SEXP, start: SEXP, step: SEXP) -> SEXP {
    unsafe { rpkg_altrep_compact_int(n, start, step) }
}

/// @rdname rpkg_altrep_helpers
#[miniextendr(unsafe(main_thread))]
fn altrep_from_doubles(x: SEXP) -> SEXP {
    unsafe { rpkg_altrep_from_doubles(x) }
}

/// @rdname rpkg_altrep_helpers
#[miniextendr(unsafe(main_thread))]
fn altrep_from_strings(x: SEXP) -> SEXP {
    unsafe { rpkg_altrep_from_strings(x) }
}

/// @rdname rpkg_altrep_helpers
#[miniextendr(unsafe(main_thread))]
fn altrep_from_logicals(x: SEXP) -> SEXP {
    unsafe { rpkg_altrep_from_logicals(x) }
}

/// @rdname rpkg_altrep_helpers
#[miniextendr(unsafe(main_thread))]
fn altrep_from_raw(x: SEXP) -> SEXP {
    unsafe { rpkg_altrep_from_raw(x) }
}

/// @rdname rpkg_altrep_helpers
#[miniextendr(unsafe(main_thread))]
fn altrep_from_list(x: SEXP) -> SEXP {
    unsafe { rpkg_altrep_from_list(x) }
}

/// @rdname rpkg_altrep_helpers
#[miniextendr(unsafe(main_thread))]
fn altrep_constant_int() -> SEXP {
    unsafe { rpkg_constant_int() }
}

/// @rdname rpkg_altrep_helpers
#[miniextendr(unsafe(main_thread))]
fn altrep_lazy_int_seq_is_materialized(x: SEXP) -> SEXP {
    unsafe { rpkg_lazy_int_seq_is_materialized(x) }
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
    fn len(&self) -> usize {
        self.len
    }
}

impl AltLogicalData for ConstantLogicalData {
    fn elt(&self, _i: usize) -> Logical {
        self.value
    }
    fn no_na(&self) -> Option<bool> {
        Some(!matches!(self.value, Logical::Na))
    }
}

miniextendr_api::impl_altlogical_from_data!(ConstantLogicalData);

#[miniextendr(class = "ConstantLogical", pkg = "rpkg")]
pub struct ConstantLogicalClass(pub ConstantLogicalData);

#[miniextendr]
fn constant_logical(value: i32, n: i32) -> SEXP {
    let logical_value = match value {
        0 => Logical::False,
        i if i == i32::MIN => Logical::Na,
        _ => Logical::True,
    };
    let data = ConstantLogicalData {
        value: logical_value,
        len: n as usize,
    };
    ConstantLogicalClass(data).into_sexp()
}

// -----------------------------------------------------------------------------
// LogicalVec: Vec<Logical> wrapper (preserves NA)
// -----------------------------------------------------------------------------

#[derive(miniextendr_api::ExternalPtr)]
pub struct LogicalVecData {
    data: Vec<Logical>,
}

impl AltrepLen for LogicalVecData {
    fn len(&self) -> usize {
        self.data.len()
    }
}

impl AltLogicalData for LogicalVecData {
    fn elt(&self, i: usize) -> Logical {
        self.data[i]
    }

    fn no_na(&self) -> Option<bool> {
        Some(!self.data.iter().any(|v| matches!(v, Logical::Na)))
    }

    fn sum(&self, na_rm: bool) -> Option<i64> {
        let mut total = 0i64;
        for v in &self.data {
            match v {
                Logical::True => total += 1,
                Logical::False => {}
                Logical::Na => {
                    if !na_rm {
                        return None;
                    }
                }
            }
        }
        Some(total)
    }
}

miniextendr_api::impl_altlogical_from_data!(LogicalVecData);

#[miniextendr(class = "LogicalVec", pkg = "rpkg")]
pub struct LogicalVecClass(pub LogicalVecData);

/// # Safety
/// Caller must ensure `x` is a valid logical SEXP and this is called from R's main thread.
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C-unwind" fn rpkg_altrep_from_logicals(x: SEXP) -> SEXP {
    use miniextendr_api::ffi::{LOGICAL, Rf_xlength};

    let n = unsafe { Rf_xlength(x) } as usize;
    let src = unsafe { LOGICAL(x) };
    let mut data = Vec::with_capacity(n);
    for i in 0..n {
        data.push(Logical::from_r_int(unsafe { *src.add(i) }));
    }

    LogicalVecClass(LogicalVecData { data }).into_sexp()
}

// -----------------------------------------------------------------------------
// LazyString: Lazily-generated strings
// -----------------------------------------------------------------------------

#[derive(miniextendr_api::ExternalPtr)]
pub struct LazyStringData {
    pub prefix: String,
    pub len: usize,
}

impl AltrepLen for LazyStringData {
    fn len(&self) -> usize {
        self.len
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
    LazyStringClass(data).into_sexp()
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
    RepeatingRawClass(data).into_sexp()
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
/// @param n Number of roots of unity (complex numbers on the unit circle).
#[miniextendr]
pub fn unit_circle(n: i32) -> SEXP {
    let data = UnitCircleData { n: n as usize };
    UnitCircleClass(data).into_sexp()
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
pub unsafe extern "C-unwind" fn rpkg_simple_vec_int(x: SEXP) -> SEXP {
    use miniextendr_api::ffi::{INTEGER, Rf_xlength};
    let n = unsafe { Rf_xlength(x) } as usize;
    let src = unsafe { INTEGER(x) };
    let mut data = Vec::with_capacity(n);
    for i in 0..n {
        data.push(unsafe { *src.add(i) });
    }
    SimpleVecIntClass(data).into_sexp()
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
    SimpleVecStringClass(StringVecData { data }).into_sexp()
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
    SimpleVecRawClass(data).into_sexp()
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
    InferredVecRealClass(data).into_sexp()
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
    BoxedIntsClass(data).into_sexp()
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
    StaticIntsClass(&STATIC_INTS[..]).into_sexp()
}

/// Create an ALTREP backed by leaked heap data (intentional memory leak).
/// Useful when you need dynamic data with 'static lifetime.
#[miniextendr]
pub fn leaked_ints(n: i32) -> SEXP {
    // Create data and leak it to get 'static lifetime
    let data: Vec<i32> = (1..=n).collect();
    let leaked: &'static [i32] = Box::leak(data.into_boxed_slice());
    StaticIntsClass(leaked).into_sexp()
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
    StaticStringsClass(&STATIC_STRINGS[..]).into_sexp()
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
    use miniextendr_api::ffi::{R_NilValue, R_PreserveObject, Rf_xlength, SEXPTYPE, TYPEOF};

    if unsafe { TYPEOF(x) } != SEXPTYPE::VECSXP {
        miniextendr_api::r_error!("altrep_from_list: expected a list (VECSXP)");
    }

    if x != unsafe { R_NilValue } {
        unsafe { R_PreserveObject(x) };
    }

    let len = unsafe { Rf_xlength(x) } as usize;
    let data = ListData { list: x, len };
    ListDataClass(data).into_sexp()
}

// region: Nonapi module for lean-stack thread tests

#[cfg(feature = "nonapi")]
mod nonapi {
    use miniextendr_api::ffi::SEXP;
    use miniextendr_api::thread::{StackCheckGuard, spawn_with_r};
    use miniextendr_api::{miniextendr, miniextendr_module};

    /// Test spawn_with_r with lean stack (8 MiB) enabled by StackCheckGuard.
    #[miniextendr]
    /// @title Non-API Thread Tests
    /// @name rpkg_nonapi
    /// @keywords internal
    /// @description Non-API thread tests (requires nonapi feature).
    /// @examples
    /// \dontrun{
    /// unsafe_C_test_spawn_with_r_lean_stack()
    /// unsafe_C_test_stack_check_guard_lean()
    /// }
    /// @aliases unsafe_C_test_spawn_with_r_lean_stack unsafe_C_test_stack_check_guard_lean
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
    mod miniextendr;

    // Aggregate all test modules
    use panic_tests;
    use unwind_protect_tests;
    use dots_tests;
    use interrupt_tests;
    use conversion_tests;
    use default_tests;
    use externalptr_tests;
    use identical_tests;
    use receiver_tests;
    use r6_tests;
    use r6_default_tests;
    use s3_tests;
    use s7_tests;
    use s4_tests;
    use worker_tests;
    use coerce_tests;
    use visibility_tests;
    use thread_tests;
    use misc_tests;
    use trait_abi_tests;
    use class_system_matrix;
    use shared_trait_test;
    use convert_pref_tests;
    use rng_tests;
    use nonapi;

    // ALTREP entrypoints are called directly from R via R/altrep.R
    extern "C-unwind" fn rpkg_altrep_compact_int;
    extern "C-unwind" fn rpkg_altrep_from_doubles;
    extern "C-unwind" fn rpkg_altrep_from_strings;
    extern "C-unwind" fn rpkg_altrep_from_logicals;
    extern "C-unwind" fn rpkg_altrep_from_raw;
    extern "C-unwind" fn rpkg_altrep_from_list;

    // ALTREP helper wrappers (internal)
    fn altrep_compact_int;
    fn altrep_from_doubles;
    fn altrep_from_strings;
    fn altrep_from_logicals;
    fn altrep_from_raw;
    fn altrep_from_list;
    fn altrep_constant_int;
    fn altrep_lazy_int_seq_is_materialized;

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
