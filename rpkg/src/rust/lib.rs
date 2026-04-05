#![allow(rustdoc::private_intra_doc_links)]
//! rpkg: Example R package demonstrating miniextendr features.
//!
//! This crate is organized into focused modules for different test categories.
//!
//! # Core Functionality
//!
//! - [`panic_tests`]: Panic, drop, and R error handling tests
//! - [`unwind_protect_tests`]: `with_r_unwind_protect` mechanism tests
//! - [`worker_tests`]: Worker thread and `with_r_thread` tests
//! - [`thread_tests`]: RThreadBuilder and thread safety tests
//! - [`interrupt_tests`]: R interrupt checking tests
//!
//! # Type Conversions
//!
//! - [`conversion_tests`]: Scalar and slice conversion tests
//! - [`conversions`]: Additional conversion utilities
//! - [`coerce_tests`]: Coerce, TryCoerce, RNativeType trait tests
//! - [`convert_pref_tests`]: Conversion preference tests
//! - [`adapter_traits_tests`]: Adapter trait implementations
//!
//! # Class Systems
//!
//! - [`r6_tests`]: R6 class system tests (including active bindings)
//! - [`r6_default_tests`]: R6 default parameter tests
//! - [`s3_tests`]: S3 class system tests
//! - [`s4_tests`]: S4 class system tests
//! - [`s7_tests`]: S7 class system tests
//! - [`class_system_matrix`]: Cross-class-system compatibility matrix
//! - [`receiver_tests`]: Receiver-style impl block tests
//!
//! # R Interface
//!
//! - [`dots_tests`]: R dots (`...`) handling tests
//! - [`default_tests`]: Default parameter value tests
//! - [`externalptr_tests`]: ExternalPtr functionality tests
//! - [`visibility_tests`]: R return value visibility tests
//! - [`identical_tests`]: R identical() comparison tests
//! - [`factor_tests`]: R factor handling tests
//! - [`rng_tests`]: R random number generator tests
//!
//! # Trait ABI
//!
//! - [`trait_abi_tests`]: Cross-package trait dispatch tests
//! - [`shared_trait_test`]: Shared trait implementation tests
//!
//! # Feature-Gated Modules
//!
//! These modules require specific Cargo features to be enabled:
//!
//! - [`rayon_tests`]: Parallel iteration tests (feature: `rayon`)
//! - [`serde_r_tests`]: Serde R serialization tests (feature: `serde`)
//! - [`ndarray_tests`]: N-dimensional array tests (feature: `ndarray`)
//! - [`vctrs_tests`]: vctrs compatibility tests (feature: `vctrs`)
//! - [`vctrs_class_example`]: vctrs class implementation example (feature: `vctrs`)
//! - [`nonapi`]: Non-API R internals tests (feature: `nonapi`)
//! - [`connection_tests`]: R connection handling tests (feature: `connections`)
//!
//! # Adapter Tests (Feature-Gated)
//!
//! Each adapter has its own feature flag:
//!
//! - [`uuid_adapter_tests`]: UUID type adapter (feature: `uuid`)
//! - [`regex_adapter_tests`]: Regex type adapter (feature: `regex`)
//! - [`time_adapter_tests`]: Time/date type adapter (feature: `time`)
//! - [`ordered_float_adapter_tests`]: OrderedFloat adapter (feature: `ordered-float`)
//! - [`bigint_adapter_tests`]: BigInt type adapter (feature: `num-bigint`)
//! - [`decimal_adapter_tests`]: Decimal type adapter (feature: `rust_decimal`)
//! - [`indexmap_adapter_tests`]: IndexMap type adapter (feature: `indexmap`)
//! - [`bytes_adapter_tests`]: Bytes/BytesMut adapter (feature: `bytes`)
//! - [`bitflags_adapter_tests`]: Bitflags adapter (feature: `bitflags`)
//! - [`bitvec_adapter_tests`]: BitVec adapter (feature: `bitvec`)
//! - [`tinyvec_adapter_tests`]: TinyVec/ArrayVec adapter (feature: `tinyvec`)
//! - [`sha2_adapter_tests`]: SHA-2 hashing adapter (feature: `sha2`)
//! - [`url_adapter_tests`]: URL parsing adapter (feature: `url`)
//! - [`aho_corasick_adapter_tests`]: Aho-Corasick string search adapter (feature: `aho-corasick`)
//! - [`toml_adapter_tests`]: TOML parsing adapter (feature: `toml`)
//! - [`tabled_adapter_tests`]: Table formatting adapter (feature: `tabled`)
//! - [`nalgebra_adapter_tests`]: Linear algebra adapter (feature: `nalgebra`)
//! - [`either_adapter_tests`]: Either type adapter (feature: `either`)
//! - [`serde_json_adapter_tests`]: JSON serialization adapter (feature: `serde_json`)
//!
//! # Miscellaneous
//!
//! - [`misc_tests`]: Miscellaneous test functions

use miniextendr_api::Altrep;
use miniextendr_api::IntoR;
use miniextendr_api::ffi::SEXP;
use miniextendr_api::miniextendr;

// Package initialization — generates R_init_miniextendr() entry point.
// Replaces the previous entrypoint.c with a pure-Rust implementation.
miniextendr_api::miniextendr_init!();

// Re-export the serde crate from miniextendr-api so test modules can derive
// Serialize/Deserialize without a direct serde dependency.
// Use `#[serde(crate = "crate::serde")]` on derived types.
#[cfg(feature = "serde")]
pub use miniextendr_api::serde_crate as serde;

mod raw_ffi;

// Test modules
mod adapter_traits_tests;
#[cfg(feature = "aho-corasick")]
mod aho_corasick_adapter_tests;
mod altrep_sexp_tests;
#[cfg(feature = "arrow")]
mod arrow_adapter_tests;
mod as_coerce_tests;
mod backtrace_tests;
#[cfg(feature = "num-bigint")]
mod bigint_adapter_tests;
#[cfg(feature = "bitflags")]
mod bitflags_adapter_tests;
#[cfg(feature = "bitvec")]
mod bitvec_adapter_tests;
#[cfg(feature = "borsh")]
mod borsh_adapter_tests;
mod box_slice_tests;
#[cfg(feature = "bytes")]
mod bytes_adapter_tests;
mod class_system_matrix;
mod coerce_tests;
mod collect_tests;
#[cfg(feature = "serde")]
mod columnar_flatten_tests;
mod condition_tests;
#[cfg(feature = "connections")]
mod connection_tests;
mod conversion_tests;
mod conversions;
mod convert_pref_tests;
mod dataframe_examples;
#[cfg(feature = "rayon")]
mod dataframe_rayon_tests;
#[cfg(feature = "datafusion")]
mod datafusion_tests;
#[cfg(feature = "rust_decimal")]
mod decimal_adapter_tests;
mod default_tests;
mod display_fromstr_tests;
mod doc_attr_tests;
mod dots_tests;
#[cfg(feature = "either")]
mod either_adapter_tests;
mod encoding_tests;
mod error_in_r_tests;
mod export_control_tests;
mod externalptr_any_tests;
mod externalptr_tests;
mod externalslice_tests;
mod factor_tests;
mod ffi_guard_tests;
mod gc_protect_tests;
mod gc_stress_fixtures;
#[cfg(feature = "growth-debug")]
mod growth_debug_tests;
mod identical_tests;
mod impl_trait_tests;
#[cfg(feature = "indexmap")]
mod indexmap_adapter_tests;
#[cfg(feature = "indicatif")]
mod indicatif_adapter_tests;
mod interrupt_tests;
mod into_r_as_tests;
mod into_r_error_tests;
#[cfg(feature = "serde")]
mod json_string_tests;
mod lazy_tests;
#[allow(deprecated)] // Intentional: tests #[deprecated] integration
mod lifecycle_tests;
#[cfg(feature = "log")]
mod log_tests;
mod macro_equivalence;
mod match_arg_tests;
mod misc_tests;
mod missing_tests;
#[cfg(feature = "nalgebra")]
mod nalgebra_adapter_tests;
#[cfg(feature = "ndarray")]
mod ndarray_tests;
#[cfg(feature = "num-complex")]
mod num_complex_adapter_tests;
#[cfg(feature = "num-traits")]
mod num_traits_adapter_tests;
#[cfg(feature = "ordered-float")]
mod ordered_float_adapter_tests;
mod panic_telemetry_tests;
mod panic_tests;
mod protect_pool_tests;
mod r6_default_tests;
mod r6_tests;
#[cfg(all(feature = "nalgebra", feature = "ndarray"))]
mod r_backed_tests;
mod r_wrapper_attrs;
mod rarray_tests;
#[cfg(feature = "rayon")]
mod rayon_tests;
mod rdata_sidecar_tests;
mod receiver_tests;
mod refcount_protect_tests;
#[cfg(feature = "regex")]
mod regex_adapter_tests;
mod rng_tests;
mod s3_tests;
mod s4_helpers_tests;
mod s4_tests;
mod s7_tests;
#[cfg(feature = "serde_json")]
mod serde_json_adapter_tests;
#[cfg(feature = "serde")]
mod serde_r_tests;
#[cfg(feature = "sha2")]
mod sha2_adapter_tests;
mod shared_trait_test;
mod streaming_altrep_tests;
#[cfg(feature = "tabled")]
mod tabled_adapter_tests;
mod thread_tests;
#[cfg(feature = "time")]
mod time_adapter_tests;
#[cfg(feature = "tinyvec")]
mod tinyvec_adapter_tests;
#[cfg(feature = "toml")]
mod toml_adapter_tests;
mod trait_abi_tests;
mod unwind_protect_tests;
#[cfg(feature = "url")]
mod url_adapter_tests;
#[cfg(feature = "uuid")]
mod uuid_adapter_tests;
mod visibility_tests;
#[cfg(feature = "worker-thread")]
mod worker_tests;

// region: proc-macro ALTREP test
// This tests the #[miniextendr] on struct path for custom ALTREP classes.
//
// The new approach requires:
// 1. A data type that implements high-level data traits (AltrepLen, AltIntegerData, etc.)
// 2. Low-level trait impls generated via impl_alt*_from_data! macro
// 3. A 1-field wrapper struct with #[miniextendr] macro

use miniextendr_api::altrep_data::{AltIntegerData, AltrepLen};
// endregion

// region: ConstantInt: An ALTREP integer that always returns the same value

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

/// @noRd
#[miniextendr(class = "ConstantInt")]
pub struct ConstantIntClass(pub ConstantIntData);

/// @noRd
#[miniextendr]
pub fn constant_int() -> ConstantIntClass {
    ConstantIntClass(ConstantIntData { value: 42, len: 10 })
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
// endregion

// region: ConstantReal: All elements are PI

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

#[miniextendr(class = "ConstantReal")]
pub struct ConstantRealClass(pub ConstantRealData);

/// @noRd
#[miniextendr]
pub fn constant_real() -> ConstantRealClass {
    ConstantRealClass(ConstantRealData {
        value: std::f64::consts::PI,
        len: 10,
    })
}
// endregion

// region: ArithSeq: Arithmetic sequence (like R's seq())

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

#[miniextendr(class = "ArithSeq")]
pub struct ArithSeqClass(pub ArithSeqData);

#[miniextendr]
fn arith_seq(from: f64, step: f64, length_out: i32) -> SEXP {
    let len = length_out as usize;
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
        // i32::MIN is NA_INTEGER in R. Check if any element equals it.
        // Elements are: start + i * step for i in 0..len (using saturating arithmetic)
        //
        // NA can occur if:
        // 1. start == i32::MIN (first element is NA)
        // 2. Saturating underflow produces i32::MIN
        //
        // Check first element
        if self.start == i32::MIN {
            return Some(false);
        }

        if self.len == 0 {
            return Some(true); // Empty sequence has no NA
        }

        // Check last element (computed via elt to catch saturation)
        let last = self.elt(self.len - 1);
        if last == i32::MIN {
            return Some(false);
        }

        // For sequences that don't saturate, check if i32::MIN is in range:
        // Compute actual bounds without saturation to detect if sequence contains i32::MIN
        let first = self.start as i64;
        let step = self.step as i64;
        let last_idx = (self.len - 1) as i64;
        let last_exact = first + last_idx * step;

        // Check if NA sentinel is in the range [min_val, max_val]
        let na_sentinel = i32::MIN as i64;
        let (min_val, max_val) = if step >= 0 {
            (first, last_exact)
        } else {
            (last_exact, first)
        };

        if na_sentinel >= min_val && na_sentinel <= max_val {
            return Some(false); // NA is in range
        }

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

    fn sum(&self, na_rm: bool) -> Option<i64> {
        if self.len == 0 {
            return Some(0);
        }

        // Check for NA values before computing sum
        if self.no_na() == Some(false) {
            if !na_rm {
                return None; // NA propagates
            }
            // When na_rm=true and there are NAs, let R compute
            return None;
        }

        // Arithmetic sequence sum: n * (first + last) / 2
        let n = self.len as i64;
        let first = self.start as i64;
        let last = first + (self.len.saturating_sub(1) as i64) * (self.step as i64);

        // Use checked arithmetic to detect overflow
        let sum_endpoints = first.checked_add(last)?;
        let product = n.checked_mul(sum_endpoints)?;
        Some(product / 2)
    }

    fn min(&self, na_rm: bool) -> Option<i32> {
        if self.len == 0 {
            return None;
        }

        // Check for NA values
        if self.no_na() == Some(false) {
            if !na_rm {
                return None; // NA propagates
            }
            // When na_rm=true and there are NAs, let R compute
            return None;
        }

        if self.step >= 0 {
            Some(self.start)
        } else {
            Some(self.elt(self.len - 1))
        }
    }

    fn max(&self, na_rm: bool) -> Option<i32> {
        if self.len == 0 {
            return None;
        }

        // Check for NA values
        if self.no_na() == Some(false) {
            if !na_rm {
                return None; // NA propagates
            }
            // When na_rm=true and there are NAs, let R compute
            return None;
        }

        if self.step >= 0 {
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
            use miniextendr_api::ffi::{Rf_allocVector, SEXPTYPE, SexpExt};
            let state = Rf_allocVector(SEXPTYPE::INTSXP, 3);
            state.set_integer_elt(0, self.start);
            state.set_integer_elt(1, self.step);
            state.set_integer_elt(2, self.len as i32);
            state
        }
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn unserialize(state: SEXP) -> Option<Self> {
        {
            use miniextendr_api::ffi::SexpExt;
            let start = state.integer_elt(0);
            let step = state.integer_elt(1);
            let len = state.integer_elt(2) as usize;
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

/// @noRd
#[miniextendr(class = "LazyIntSeq")]
pub struct LazyIntSeqClass(pub LazyIntSeqData);

/// @noRd
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

/// Check if a lazy int seq ALTREP has been materialized.
///
/// Takes raw SEXP (extern "C-unwind") because auto-materialization in
/// TryFromSexp for SEXP would trigger materialization before we can inspect it.
/// @noRd
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_lazy_int_seq_is_materialized(x: SEXP) -> SEXP {
    use miniextendr_api::altrep_data1_as;
    use miniextendr_api::ffi::ALTREP;

    let result = if unsafe { ALTREP(x) } == 0 {
        false
    } else {
        match unsafe { altrep_data1_as::<LazyIntSeqData>(x) } {
            Some(data) => data.materialized.is_some(),
            None => false,
        }
    };
    result.into_sexp()
}
// endregion

// region: ALTREP helper functions

/// @title ALTREP Helpers
/// @name rpkg_altrep_helpers
/// @noRd
/// @description ALTREP convenience functions for testing and examples.
/// @examples
/// \dontrun{
/// x <- altrep_compact_int(5L, 1L, 2L)
/// y <- altrep_from_doubles(c(1, 2, 3))
/// z <- altrep_from_strings(c("a", "b"))
/// lazy_int_seq_is_materialized(lazy_int_seq(1L, 5L, 1L))
/// }

/// @noRd
#[miniextendr]
fn altrep_compact_int(n: i32, start: i32, step: i32) -> LazyIntSeqClass {
    if n < 0 {
        panic!("altrep_compact_int: n must be >= 0");
    }
    let len = if n == 0 { 0 } else { n as usize };
    LazyIntSeqClass(LazyIntSeqData {
        start,
        step,
        len,
        materialized: None,
    })
}

/// @noRd
#[miniextendr]
pub fn altrep_from_doubles(x: Vec<f64>) -> InferredVecRealClass {
    InferredVecRealClass(x)
}

/// @noRd
#[miniextendr]
pub fn altrep_from_strings(x: Vec<Option<String>>) -> SimpleVecStringClass {
    SimpleVecStringClass(StringVecData { data: x })
}

/// @noRd
#[miniextendr]
pub fn altrep_from_logicals(x: Vec<Logical>) -> LogicalVecClass {
    LogicalVecClass(LogicalVecData { data: x })
}

/// @noRd
#[miniextendr]
pub fn altrep_from_raw(x: &[u8]) -> SimpleVecRawClass {
    SimpleVecRawClass(x.to_vec())
}

/// @noRd
#[miniextendr]
pub fn altrep_from_integers(x: Vec<i32>) -> SimpleVecIntClass {
    SimpleVecIntClass(x)
}

/// @noRd
#[miniextendr]
pub fn altrep_from_list(x: SEXP) -> ListDataClass {
    use miniextendr_api::ffi::{R_PreserveObject, SexpExt};

    if !x.is_list() {
        panic!("altrep_from_list: expected a list (VECSXP)");
    }

    if !x.is_nil() {
        unsafe { R_PreserveObject(x) };
    }

    let len = x.len();
    ListDataClass(ListData { list: x, len })
}
// endregion

// region: ALTREP Convenience Helpers Examples

/// Example: Small data - regular copy is fine
///
/// @export
#[miniextendr]
pub fn small_vec_copy() -> Vec<i32> {
    vec![1, 2, 3, 4, 5] // Uses IntoR, copies to R
}

/// Example: Large data - ALTREP avoids copy
///
/// @export
#[miniextendr]
pub fn large_vec_altrep() -> SEXP {
    use miniextendr_api::IntoRAltrep;
    let data = vec![0; 100_000];
    data.into_sexp_altrep() // Zero-copy via IntoRAltrep
}

/// Example: Lazy computation - compute on demand
///
/// @param n Length of the sequence.
/// @export
#[miniextendr]
pub fn lazy_squares(n: i32) -> SEXP {
    use miniextendr_api::IntoRAltrep;
    if n < 0 {
        panic!("lazy_squares: n must be >= 0");
    }
    (0..n)
        .map(|i| i * i)
        .collect::<Vec<i32>>()
        .into_sexp_altrep()
}

/// Example: Using into_altrep() to store wrapper
///
/// @param n Length of the vector.
/// @export
#[miniextendr]
pub fn boxed_data_altrep(n: i32) -> SEXP {
    use miniextendr_api::IntoRAltrep;
    if n < 0 {
        panic!("boxed_data_altrep: n must be >= 0");
    }
    let data = (0..n).collect::<Vec<i32>>().into_boxed_slice();
    data.into_altrep().into_sexp()
}
// endregion

// region: Benchmark Functions - Direct Comparison

/// Create a vector of given size using regular copy (IntoR)
///
/// @param n Length of the vector.
/// @export
#[miniextendr]
pub fn bench_vec_copy(n: i32) -> Vec<i32> {
    if n < 0 {
        panic!("n must be >= 0");
    }
    vec![0; n as usize] // Uses IntoR - copies to R
}

/// Create a vector of given size using ALTREP zero-copy
///
/// @param n Length of the vector.
/// @export
#[miniextendr]
pub fn bench_vec_altrep(n: i32) -> SEXP {
    use miniextendr_api::IntoRAltrep;
    if n < 0 {
        panic!("n must be >= 0");
    }
    vec![0; n as usize].into_sexp_altrep() // Zero-copy
}
// endregion

// region: ConstantLogical: All TRUE or all FALSE

#[derive(miniextendr_api::ExternalPtr, miniextendr_api::AltrepLogical)]
#[altrep(len = "len", elt = "value", dataptr)]
pub struct ConstantLogicalData {
    value: Logical,
    len: usize,
    materialized: Option<Vec<i32>>,
}

impl miniextendr_api::altrep_data::AltrepDataptr<i32> for ConstantLogicalData {
    fn dataptr(&mut self, _writable: bool) -> Option<*mut i32> {
        if self.materialized.is_none() {
            let value = self.value.to_r_int();
            let data = vec![value; self.len];
            self.materialized = Some(data);
        }
        self.materialized.as_mut().map(|v| v.as_mut_ptr())
    }

    fn dataptr_or_null(&self) -> Option<*const i32> {
        self.materialized.as_ref().map(|v| v.as_ptr())
    }
}

#[miniextendr(class = "ConstantLogical")]
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
        materialized: None,
    };
    ConstantLogicalClass(data).into_sexp()
}
// endregion

// region: LogicalVec: Vec<Logical> wrapper (preserves NA)

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

// Implement serialization support for LogicalVecData
impl miniextendr_api::altrep_data::AltrepSerialize for LogicalVecData {
    fn serialized_state(&self) -> SEXP {
        // Serialize as a regular logical vector
        // NA_LOGICAL in R is the same as NA_INTEGER = i32::MIN
        const NA_LOGICAL: i32 = i32::MIN;
        unsafe {
            use miniextendr_api::ffi::{Rf_allocVector, SEXPTYPE, SexpExt};
            let n = self.data.len();
            let state = Rf_allocVector(SEXPTYPE::LGLSXP, n as isize);
            for (i, v) in self.data.iter().enumerate() {
                let raw = match v {
                    Logical::True => 1,
                    Logical::False => 0,
                    Logical::Na => NA_LOGICAL,
                };
                state.set_logical_elt(i as isize, raw);
            }
            state
        }
    }

    fn unserialize(state: SEXP) -> Option<Self> {
        const NA_LOGICAL: i32 = i32::MIN;
        {
            use miniextendr_api::ffi::SexpExt;
            let n = state.len();
            let mut data = Vec::with_capacity(n);
            for i in 0..n {
                let raw = state.logical_elt(i as isize);
                let v = if raw == NA_LOGICAL {
                    Logical::Na
                } else if raw != 0 {
                    Logical::True
                } else {
                    Logical::False
                };
                data.push(v);
            }
            Some(LogicalVecData { data })
        }
    }
}

miniextendr_api::impl_altlogical_from_data!(LogicalVecData, serialize);

#[miniextendr(class = "LogicalVec")]
pub struct LogicalVecClass(pub LogicalVecData);
// endregion

// region: LazyString: Lazily-generated strings

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

#[miniextendr(class = "LazyString")]
pub struct LazyStringClass(pub LazyStringData);

#[miniextendr]
fn lazy_string(prefix: &str, n: i32) -> SEXP {
    let data = LazyStringData {
        prefix: prefix.to_string(),
        len: n as usize,
    };
    LazyStringClass(data).into_sexp()
}
// endregion

// region: RepeatingRaw: Repeating byte pattern

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

#[miniextendr(class = "RepeatingRaw")]
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

/// @noRd
#[miniextendr(class = "UnitCircle")]
pub struct UnitCircleClass(pub UnitCircleData);

/// @noRd
#[miniextendr]
pub fn unit_circle(n: i32) -> SEXP {
    let data = UnitCircleData { n: n as usize };
    UnitCircleClass(data).into_sexp()
}

// -----------------------------------------------------------------------------
// IntegerSequenceList: List where each element is an integer vector 1:i
// This demonstrates ALTREP for list vectors (VECSXP)
// -----------------------------------------------------------------------------

#[derive(miniextendr_api::ExternalPtr)]
pub struct IntegerSequenceListData {
    /// Number of elements in the list
    n: usize,
}

impl AltrepLen for IntegerSequenceListData {
    fn len(&self) -> usize {
        self.n
    }
}

impl AltListData for IntegerSequenceListData {
    fn elt(&self, i: usize) -> SEXP {
        // Each element is an integer vector from 1 to (i+1)
        // Element 1: c(1L)
        // Element 2: c(1L, 2L)
        // Element 3: c(1L, 2L, 3L)
        // etc.
        let seq: Vec<i32> = (1..=((i + 1) as i32)).collect();
        seq.into_sexp()
    }
}

miniextendr_api::impl_altlist_from_data!(IntegerSequenceListData);

/// @noRd
#[miniextendr(class = "IntegerSequenceList")]
pub struct IntegerSequenceListClass(pub IntegerSequenceListData);

/// Create a list ALTREP where each element is an integer sequence.
///
/// @param n Number of elements in the list.
/// @return A list where element i contains the vector 1:i.
/// @examples
/// lst <- integer_sequence_list(3L)
/// lst[[1]]  # c(1L)
/// lst[[2]]  # c(1L, 2L)
/// lst[[3]]  # c(1L, 2L, 3L)
/// @export
#[miniextendr]
pub fn integer_sequence_list(n: i32) -> SEXP {
    let data = IntegerSequenceListData { n: n as usize };
    IntegerSequenceListClass(data).into_sexp()
}
// endregion

// region: SimpleVecInt: Vec<i32> wrapper (simplest example)

#[miniextendr(class = "SimpleVecInt")]
pub struct SimpleVecIntClass(pub Vec<i32>);
// endregion

// region: SimpleVecString: Vec<Option<String>> wrapper (preserves NA)

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

miniextendr_api::impl_altstring_from_data!(StringVecData, dataptr);

#[miniextendr(class = "SimpleVecString")]
pub struct SimpleVecStringClass(pub StringVecData);
// endregion

// region: SimpleVecRaw: Vec<u8> wrapper

#[miniextendr(class = "SimpleVecRaw")]
pub struct SimpleVecRawClass(pub Vec<u8>);
// endregion

// region: InferredVecReal: Vec<f64> wrapper with base type inferred from inner type

/// @noRd
#[miniextendr(class = "InferredVecReal")]
pub struct InferredVecRealClass(pub Vec<f64>);
// endregion

// region: BoxedInts: Box<[i32]> wrapper (owned slice example)

/// @noRd
#[miniextendr(class = "BoxedInts")]
pub struct BoxedIntsClass(pub Box<[i32]>);

/// @noRd
#[miniextendr]
pub fn boxed_ints(n: i32) -> SEXP {
    let data: Box<[i32]> = (1..=n).collect::<Vec<_>>().into_boxed_slice();
    BoxedIntsClass(data).into_sexp()
}
// endregion

// region: StaticInts: &'static [i32] wrapper (static slice example)

/// Static data that lives for the entire program lifetime
///
/// Data to showcase functionality
static STATIC_INTS: [i32; 5] = [10, 20, 30, 40, 50];

/// @noRd
#[miniextendr(class = "StaticInts")]
pub struct StaticIntsClass(pub &'static [i32]);

/// @noRd
#[miniextendr]
pub fn static_ints() -> SEXP {
    StaticIntsClass(&STATIC_INTS[..]).into_sexp()
}

/// @noRd
#[miniextendr]
pub fn leaked_ints(n: i32) -> SEXP {
    // Create data and leak it to get 'static lifetime
    let data: Vec<i32> = (1..=n).collect();
    let leaked: &'static [i32] = Box::leak(data.into_boxed_slice());
    StaticIntsClass(leaked).into_sexp()
}

// endregion

// region: StaticStrings: &'static [&'static str] wrapper

/// Static string data
///
/// Data to showcase functionality
static STATIC_STRINGS: [&str; 4] = ["alpha", "beta", "gamma", "delta"];

/// @noRd
#[miniextendr(class = "StaticStrings")]
pub struct StaticStringsClass(pub &'static [&'static str]);

/// @noRd
#[miniextendr]
pub fn static_strings() -> SEXP {
    StaticStringsClass(&STATIC_STRINGS[..]).into_sexp()
}

// endregion

// region: ListData: list-backed ALTREP (stores original list SEXP)

#[derive(miniextendr_api::ExternalPtr)]
pub struct ListData {
    list: SEXP,
    len: usize,
}

impl Drop for ListData {
    fn drop(&mut self) {
        unsafe {
            if self.list != miniextendr_api::ffi::SEXP::null() {
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
        use miniextendr_api::ffi::SexpExt;
        self.list.vector_elt(i as miniextendr_api::ffi::R_xlen_t)
    }
}

miniextendr_api::impl_altlist_from_data!(ListData);

#[miniextendr(class = "ListData")]
pub struct ListDataClass(pub ListData);
// endregion

// region: Builtin ALTREP test fixtures
//
// These demonstrate ALTREP support using the `Altrep<T>` marker type.
// The marker type opts into ALTREP representation for standard types
// that would otherwise be eagerly copied to R.
//
// Without `Altrep<T>`:
//   fn foo() -> Vec<i32>  // Copies all data to R immediately
//
// With `Altrep<T>`:
//   fn foo() -> Altrep<Vec<i32>>  // Data stays in Rust, accessed on-demand

/// @noRd
#[miniextendr]
pub fn iter_int_range(from: i32, to: i32) -> Altrep<Vec<i32>> {
    Altrep((from..to).collect())
}

/// @noRd
#[miniextendr]
pub fn iter_real_squares(n: i32) -> Altrep<Vec<f64>> {
    let len = n.max(0) as usize;
    Altrep((0..len).map(|i| (i * i) as f64).collect())
}

/// @noRd
#[miniextendr]
pub fn iter_logical_alternating(n: i32) -> Altrep<Vec<bool>> {
    let len = n.max(0) as usize;
    Altrep((0..len).map(|i| i % 2 == 0).collect())
}

/// @noRd
#[miniextendr]
pub fn iter_raw_bytes(n: i32) -> Altrep<Vec<u8>> {
    let len = n.max(0) as usize;
    Altrep((0..len).map(|i| (i % 256) as u8).collect())
}

/// @noRd
#[miniextendr]
pub fn iter_string_items(n: i32) -> Altrep<Vec<String>> {
    let len = n.max(0) as usize;
    Altrep((0..len).map(|i| format!("item_{}", i)).collect())
}

// Note: iter_complex_spiral removed - Vec<Rcomplex> doesn't have builtin ALTREP support
// Use unit_circle() for complex ALTREP testing instead

/// @noRd
#[miniextendr]
pub fn iter_int_from_u16(n: i32) -> Altrep<Vec<i32>> {
    let len = n.max(0) as usize;
    Altrep((0..len).map(|i| (i * 100) as i32).collect())
}

/// @noRd
#[miniextendr]
pub fn iter_real_from_f32(n: i32) -> Altrep<Vec<f64>> {
    let len = n.max(0) as usize;
    Altrep((0..len).map(|i| i as f64 * 1.5).collect())
}

/// @noRd
#[miniextendr]
pub fn vec_int_altrep(n: i32) -> Altrep<Vec<i32>> {
    let len = n.max(0) as usize;
    Altrep((1..=len as i32).collect())
}

/// @noRd
#[miniextendr]
pub fn vec_real_altrep(n: i32) -> Altrep<Vec<f64>> {
    let len = n.max(0) as usize;
    Altrep((1..=len).map(|i| i as f64 * 0.5).collect())
}

/// @noRd
#[miniextendr]
pub fn vec_complex_altrep(n: i32) -> Altrep<Vec<Rcomplex>> {
    let len = n.max(0) as usize;
    Altrep(
        (0..len)
            .map(|i| Rcomplex {
                r: i as f64,
                i: -(i as f64),
            })
            .collect(),
    )
}

/// @noRd
#[miniextendr]
pub fn boxed_reals(n: i32) -> Altrep<Box<[f64]>> {
    let len = n.max(0) as usize;
    let data: Box<[f64]> = (1..=len)
        .map(|i| i as f64 * 1.5)
        .collect::<Vec<_>>()
        .into_boxed_slice();
    Altrep(data)
}

/// @noRd
#[miniextendr]
pub fn boxed_logicals(n: i32) -> Altrep<Box<[bool]>> {
    let len = n.max(0) as usize;
    let data: Box<[bool]> = (0..len)
        .map(|i| i % 2 == 0)
        .collect::<Vec<_>>()
        .into_boxed_slice();
    Altrep(data)
}

/// @noRd
#[miniextendr]
pub fn boxed_raw(n: i32) -> Altrep<Box<[u8]>> {
    let len = n.max(0) as usize;
    let data: Box<[u8]> = (0..len)
        .map(|i| (i % 256) as u8)
        .collect::<Vec<_>>()
        .into_boxed_slice();
    Altrep(data)
}

/// @noRd
#[miniextendr]
pub fn boxed_strings(n: i32) -> Altrep<Box<[String]>> {
    let len = n.max(0) as usize;
    let data: Box<[String]> = (0..len)
        .map(|i| format!("boxed_{}", i))
        .collect::<Vec<_>>()
        .into_boxed_slice();
    Altrep(data)
}

/// @noRd
#[miniextendr]
pub fn boxed_complex(n: i32) -> Altrep<Box<[Rcomplex]>> {
    let len = n.max(0) as usize;
    let data: Box<[Rcomplex]> = (0..len)
        .map(|i| Rcomplex {
            r: i as f64 + 0.25,
            i: i as f64 + 0.75,
        })
        .collect::<Vec<_>>()
        .into_boxed_slice();
    Altrep(data)
}

/// @noRd
#[miniextendr]
pub fn range_int_altrep(from: i32, to: i32) -> Altrep<std::ops::Range<i32>> {
    Altrep(from..to)
}

/// @noRd
#[miniextendr]
pub fn range_i64_altrep(from: i64, to: i64) -> Altrep<std::ops::Range<i64>> {
    Altrep(from..to)
}

/// @noRd
#[miniextendr]
pub fn range_real_altrep(from: f64, to: f64) -> Altrep<std::ops::Range<f64>> {
    Altrep(from..to)
}

// endregion

// region: Sparse iterator ALTREP test fixtures
//
// These demonstrate the sparse iterator ALTREP types that use Iterator::nth()
// to skip elements efficiently. Unlike the prefix-caching variants, sparse
// iterators only cache accessed elements and skip intermediate ones.

use miniextendr_api::altrep_data::{
    SparseIterIntData, SparseIterLogicalData, SparseIterRawData, SparseIterRealData,
};

/// Type alias for boxed iterator producing i32
type BoxedIntIter = Box<dyn Iterator<Item = i32>>;

/// Wrapper for sparse integer iterator ALTREP
#[derive(miniextendr_api::ExternalPtr)]
pub struct SparseIntIterData {
    inner: SparseIterIntData<BoxedIntIter>,
}

impl AltrepLen for SparseIntIterData {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl miniextendr_api::altrep_data::AltIntegerData for SparseIntIterData {
    fn elt(&self, i: usize) -> i32 {
        self.inner.elt(i)
    }

    fn as_slice(&self) -> Option<&[i32]> {
        None // Sparse storage cannot provide contiguous slice
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [i32]) -> usize {
        self.inner.get_region(start, len, buf)
    }
}

miniextendr_api::impl_altinteger_from_data!(SparseIntIterData);

/// @noRd
#[miniextendr(class = "SparseIntIter")]
pub struct SparseIntIterClass(pub SparseIntIterData);

/// Create a sparse integer iterator ALTREP that skips elements.
///
/// Elements are computed on-demand using Iterator::nth(). Once an element
/// is skipped (a higher index is accessed first), it cannot be retrieved
/// and will return NA.
///
/// @param from Start value (inclusive)
/// @param to End value (exclusive)
/// @noRd
#[miniextendr]
pub fn sparse_iter_int(from: i32, to: i32) -> SEXP {
    let len = (to - from).max(0) as usize;
    let start = from;
    let iter: BoxedIntIter = Box::new((0..len as i32).map(move |i| start + i));
    let data = SparseIntIterData {
        inner: SparseIterIntData::from_iter(iter, len),
    };
    SparseIntIterClass(data).into_sexp()
}

/// Create a sparse integer iterator that generates squares.
/// @noRd
#[miniextendr]
pub fn sparse_iter_int_squares(n: i32) -> SEXP {
    let len = n.max(0) as usize;
    let iter: BoxedIntIter = Box::new((0..len as i32).map(|i| i * i));
    let data = SparseIntIterData {
        inner: SparseIterIntData::from_iter(iter, len),
    };
    SparseIntIterClass(data).into_sexp()
}

/// Type alias for boxed iterator producing f64
type BoxedRealIter = Box<dyn Iterator<Item = f64>>;

/// Wrapper for sparse real iterator ALTREP
#[derive(miniextendr_api::ExternalPtr)]
pub struct SparseRealIterData {
    inner: SparseIterRealData<BoxedRealIter>,
}

impl AltrepLen for SparseRealIterData {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl miniextendr_api::altrep_data::AltRealData for SparseRealIterData {
    fn elt(&self, i: usize) -> f64 {
        self.inner.elt(i)
    }

    fn as_slice(&self) -> Option<&[f64]> {
        None
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [f64]) -> usize {
        self.inner.get_region(start, len, buf)
    }
}

miniextendr_api::impl_altreal_from_data!(SparseRealIterData);

/// @noRd
#[miniextendr(class = "SparseRealIter")]
pub struct SparseRealIterClass(pub SparseRealIterData);

/// Create a sparse real iterator ALTREP.
/// @noRd
#[miniextendr]
pub fn sparse_iter_real(from: f64, step: f64, n: i32) -> SEXP {
    let len = n.max(0) as usize;
    let iter: BoxedRealIter = Box::new((0..len).map(move |i| from + (i as f64) * step));
    let data = SparseRealIterData {
        inner: SparseIterRealData::from_iter(iter, len),
    };
    SparseRealIterClass(data).into_sexp()
}

/// Type alias for boxed iterator producing bool
type BoxedLogicalIter = Box<dyn Iterator<Item = bool>>;

/// Wrapper for sparse logical iterator ALTREP
#[derive(miniextendr_api::ExternalPtr)]
pub struct SparseLogicalIterData {
    inner: SparseIterLogicalData<BoxedLogicalIter>,
}

impl AltrepLen for SparseLogicalIterData {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl miniextendr_api::altrep_data::AltLogicalData for SparseLogicalIterData {
    fn elt(&self, i: usize) -> miniextendr_api::altrep_data::Logical {
        self.inner.elt(i)
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [i32]) -> usize {
        self.inner.get_region(start, len, buf)
    }
}

miniextendr_api::impl_altlogical_from_data!(SparseLogicalIterData);

/// @noRd
#[miniextendr(class = "SparseLogicalIter")]
pub struct SparseLogicalIterClass(pub SparseLogicalIterData);

/// Create a sparse logical iterator ALTREP (alternating TRUE/FALSE).
/// @noRd
#[miniextendr]
pub fn sparse_iter_logical(n: i32) -> SEXP {
    let len = n.max(0) as usize;
    let iter: BoxedLogicalIter = Box::new((0..len).map(|i| i % 2 == 0));
    let data = SparseLogicalIterData {
        inner: SparseIterLogicalData::from_iter(iter, len),
    };
    SparseLogicalIterClass(data).into_sexp()
}

/// Type alias for boxed iterator producing u8
type BoxedRawIter = Box<dyn Iterator<Item = u8>>;

/// Wrapper for sparse raw iterator ALTREP
#[derive(miniextendr_api::ExternalPtr)]
pub struct SparseRawIterData {
    inner: SparseIterRawData<BoxedRawIter>,
}

impl AltrepLen for SparseRawIterData {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl miniextendr_api::altrep_data::AltRawData for SparseRawIterData {
    fn elt(&self, i: usize) -> u8 {
        self.inner.elt(i)
    }

    fn as_slice(&self) -> Option<&[u8]> {
        None
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [u8]) -> usize {
        self.inner.get_region(start, len, buf)
    }
}

miniextendr_api::impl_altraw_from_data!(SparseRawIterData);

/// @noRd
#[miniextendr(class = "SparseRawIter")]
pub struct SparseRawIterClass(pub SparseRawIterData);

/// Create a sparse raw iterator ALTREP.
/// @noRd
#[miniextendr]
pub fn sparse_iter_raw(n: i32) -> SEXP {
    let len = n.max(0) as usize;
    let iter: BoxedRawIter = Box::new((0..len).map(|i| (i % 256) as u8));
    let data = SparseRawIterData {
        inner: SparseIterRawData::from_iter(iter, len),
    };
    SparseRawIterClass(data).into_sexp()
}

// endregion

// region: Nonapi module for lean-stack thread tests

#[cfg(feature = "nonapi")]
mod nonapi;

// endregion

// region: vctrs module (optional vctrs C API support)

#[cfg(feature = "vctrs")]
mod vctrs_class_example;
#[cfg(feature = "vctrs")]
mod vctrs_derive_example;
#[cfg(feature = "vctrs")]
mod vctrs_tests;

// endregion

// region: Feature detection

/// Returns a vector of enabled feature names for this build.
///
/// This function is useful for R tests to skip tests when features are not enabled.
///
/// @name rpkg_enabled_features
/// @return A character vector of enabled feature names.
/// @examples
/// rpkg_enabled_features()
/// @export
#[miniextendr]
pub fn rpkg_enabled_features() -> Vec<&'static str> {
    let mut features = Vec::new();

    // Core features
    if cfg!(feature = "nonapi") {
        features.push("nonapi");
    }

    // Optional crate features
    if cfg!(feature = "uuid") {
        features.push("uuid");
    }
    if cfg!(feature = "time") {
        features.push("time");
    }
    if cfg!(feature = "regex") {
        features.push("regex");
    }
    if cfg!(feature = "indexmap") {
        features.push("indexmap");
    }
    if cfg!(feature = "serde") {
        features.push("serde");
    }
    if cfg!(feature = "serde_json") {
        features.push("serde_json");
    }
    if cfg!(feature = "num-bigint") {
        features.push("num-bigint");
    }
    if cfg!(feature = "rust_decimal") {
        features.push("rust_decimal");
    }
    if cfg!(feature = "ordered-float") {
        features.push("ordered-float");
    }
    if cfg!(feature = "num-traits") {
        features.push("num-traits");
    }
    if cfg!(feature = "rand") {
        features.push("rand");
    }
    if cfg!(feature = "rand_distr") {
        features.push("rand_distr");
    }
    if cfg!(feature = "rayon") {
        features.push("rayon");
    }
    if cfg!(feature = "ndarray") {
        features.push("ndarray");
    }
    if cfg!(feature = "nalgebra") {
        features.push("nalgebra");
    }
    if cfg!(feature = "either") {
        features.push("either");
    }
    if cfg!(feature = "bytes") {
        features.push("bytes");
    }
    if cfg!(feature = "bitvec") {
        features.push("bitvec");
    }
    if cfg!(feature = "bitflags") {
        features.push("bitflags");
    }
    if cfg!(feature = "num-complex") {
        features.push("num-complex");
    }
    if cfg!(feature = "sha2") {
        features.push("sha2");
    }
    if cfg!(feature = "tabled") {
        features.push("tabled");
    }
    if cfg!(feature = "toml") {
        features.push("toml");
    }
    if cfg!(feature = "url") {
        features.push("url");
    }
    if cfg!(feature = "aho-corasick") {
        features.push("aho-corasick");
    }
    if cfg!(feature = "tinyvec") {
        features.push("tinyvec");
    }
    if cfg!(feature = "raw_conversions") {
        features.push("raw_conversions");
    }
    if cfg!(feature = "vctrs") {
        features.push("vctrs");
    }
    if cfg!(feature = "borsh") {
        features.push("borsh");
    }
    if cfg!(feature = "indicatif") {
        features.push("indicatif");
    }

    // Class systems (always available, not feature-gated)
    features.push("s7");

    features
}

// endregion
mod dataframe_collections_test;
