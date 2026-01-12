//! Tests for RSidecar and #[r_data] functionality.
//!
//! This module tests the R-side sidecar accessor generation with different class systems.

use miniextendr_api::externalptr::{ExternalPtr, RSidecar};
use miniextendr_api::ffi::SEXP;
use miniextendr_api::{miniextendr, miniextendr_module};

// =============================================================================
// Env (default) - standalone functions: Type_get_field(), Type_set_field()
// =============================================================================

/// Demonstrates env class system (default).
/// Generates: SidecarEnv_get_count(), SidecarEnv_set_count(), etc.
#[derive(miniextendr_api::ExternalPtr, Debug)]
#[externalptr(env)]
pub struct SidecarEnv {
    /// Regular Rust field (not exposed to R)
    internal_value: i32,

    /// Selector - enables R accessors
    #[r_data]
    _r: RSidecar,

    /// Zero-overhead scalar: i32
    #[r_data]
    pub count: i32,

    /// Zero-overhead scalar: f64
    #[r_data]
    pub score: f64,

    /// Zero-overhead scalar: bool
    #[r_data]
    pub flag: bool,

    /// Conversion type: String
    #[r_data]
    pub name: String,

    /// Raw SEXP slot
    #[r_data]
    pub raw_slot: SEXP,
}

/// Empty impl block to trigger r_data call def collection
#[miniextendr(env)]
impl SidecarEnv {}

/// Create a new SidecarEnv wrapped in ExternalPtr.
///
/// @name rpkg_rdata_sidecar_env
/// @examples
/// ptr <- rdata_sidecar_env_new(42L, 3.14, TRUE, "hello")
/// SidecarEnv_get_count(ptr)
/// SidecarEnv_set_count(ptr, 100L)
/// SidecarEnv_get_count(ptr)
/// @aliases rdata_sidecar_env_new SidecarEnv_get_count SidecarEnv_set_count
///   SidecarEnv_get_score SidecarEnv_set_score SidecarEnv_get_flag
///   SidecarEnv_set_flag SidecarEnv_get_name SidecarEnv_set_name
///   SidecarEnv_get_raw_slot SidecarEnv_set_raw_slot
/// @param count Initial integer count.
/// @param score Initial double score.
/// @param flag Initial boolean flag.
/// @param name Initial string name.
#[miniextendr]
pub fn rdata_sidecar_env_new(
    count: i32,
    score: f64,
    flag: bool,
    name: String,
) -> ExternalPtr<SidecarEnv> {
    use miniextendr_api::ffi::R_NilValue;

    ExternalPtr::new(SidecarEnv {
        internal_value: 999,
        _r: RSidecar,
        count,
        score,
        flag,
        name,
        raw_slot: unsafe { R_NilValue },
    })
}

// =============================================================================
// R6 - active bindings: obj$field, obj$field <- value
// =============================================================================

/// Demonstrates R6 class system.
/// Generates active bindings that integrate with R6Class.
#[derive(miniextendr_api::ExternalPtr, Debug)]
#[externalptr(r6)]
pub struct SidecarR6 {
    #[r_data]
    _r: RSidecar,

    #[r_data]
    pub value: i32,

    #[r_data]
    pub label: String,
}

/// Empty impl block to trigger r_data call def collection
#[miniextendr(r6)]
impl SidecarR6 {}

/// Create a new SidecarR6 (the R6Class will wrap this).
#[miniextendr]
pub fn rdata_sidecar_r6_new(value: i32, label: String) -> ExternalPtr<SidecarR6> {
    ExternalPtr::new(SidecarR6 {
        _r: RSidecar,
        value,
        label,
    })
}

// =============================================================================
// S3 - $ method dispatch: obj$field, obj$field <- value
// =============================================================================

/// Demonstrates S3 class system.
/// Generates $.class and $<-.class methods.
#[derive(miniextendr_api::ExternalPtr, Debug)]
#[externalptr(s3)]
pub struct SidecarS3 {
    #[r_data]
    _r: RSidecar,

    #[r_data]
    pub data: f64,
}

/// Empty impl block to trigger r_data call def collection
#[miniextendr(s3)]
impl SidecarS3 {}

/// Create a new SidecarS3.
#[miniextendr]
pub fn rdata_sidecar_s3_new(data: f64) -> ExternalPtr<SidecarS3> {
    ExternalPtr::new(SidecarS3 {
        _r: RSidecar,
        data,
    })
}

// =============================================================================
// Module registration
// =============================================================================

miniextendr_module! {
    mod rdata_sidecar_tests;

    // Impl blocks trigger r_data call def collection
    impl SidecarEnv;
    impl SidecarR6;
    impl SidecarS3;

    fn rdata_sidecar_env_new;
    fn rdata_sidecar_r6_new;
    fn rdata_sidecar_s3_new;
}
