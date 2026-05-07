//! Tests for RSidecar and #[r_data] functionality.
//!
//! This module tests the R-side sidecar accessor generation with different class systems.

use miniextendr_api::externalptr::{ExternalPtr, RSidecar};
use miniextendr_api::ffi::SEXP;
use miniextendr_api::miniextendr;

// region: Env (default) - standalone functions: Type_get_field(), Type_set_field()

/// Demonstrates env class system (default).
/// Generates: SidecarEnv_get_count(), SidecarEnv_set_count(), etc.
#[derive(miniextendr_api::ExternalPtr, Debug)]
#[externalptr(env)]
pub struct SidecarEnv {
    /// Regular Rust field (not exposed to R) - intentionally unused to demonstrate
    /// that non-#[r_data] fields are private to Rust.
    _internal_value: i32,

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

/// Env class registration for SidecarEnv (enables R sidecar accessors).
#[miniextendr(env)]
impl SidecarEnv {}

/// Test creating a SidecarEnv with all sidecar field types.
/// @param count Integer sidecar field.
/// @param score Numeric sidecar field.
/// @param flag Logical sidecar field.
/// @param name Character sidecar field.
#[miniextendr]
pub fn rdata_sidecar_env_new(
    count: i32,
    score: f64,
    flag: bool,
    name: String,
) -> ExternalPtr<SidecarEnv> {
    use miniextendr_api::ffi::SEXP;

    ExternalPtr::new(SidecarEnv {
        _internal_value: 999,
        _r: RSidecar,
        count,
        score,
        flag,
        name,
        raw_slot: SEXP::nil(),
    })
}
// endregion

// region: R6 - active bindings: obj$field, obj$field <- value

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

/// R6 class registration for SidecarR6 with active binding sidecar accessors.
/// @field value Integer sidecar field (active binding).
/// @field label Character sidecar field (active binding).
#[miniextendr(r6(r_data_accessors))]
impl SidecarR6 {
    /// Create a new SidecarR6 with initial values.
    /// @param value Integer sidecar field.
    /// @param label Character sidecar field.
    pub fn new(value: i32, label: String) -> Self {
        SidecarR6 {
            _r: RSidecar,
            value,
            label,
        }
    }
}

/// Test creating a SidecarR6 with R6 active binding accessors.
/// @param value Integer sidecar field.
/// @param label Character sidecar field.
#[miniextendr]
pub fn rdata_sidecar_r6_new(value: i32, label: String) -> ExternalPtr<SidecarR6> {
    ExternalPtr::new(SidecarR6 {
        _r: RSidecar,
        value,
        label,
    })
}
// endregion

// region: S3 - $ method dispatch: obj$field, obj$field <- value

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

/// S3 class registration for SidecarS3 (enables $ method dispatch for sidecar fields).
/// @param x An object.
/// @param ... Additional arguments.
#[miniextendr(s3)]
impl SidecarS3 {}

/// Test creating a SidecarS3 with S3 $ method dispatch for sidecar fields.
/// @param data Numeric sidecar field.
#[miniextendr]
pub fn rdata_sidecar_s3_new(data: f64) -> ExternalPtr<SidecarS3> {
    ExternalPtr::new(SidecarS3 { _r: RSidecar, data })
}
// endregion

// region: S4 - slot accessors via setMethod

/// Demonstrates S4 class system.
/// Generates setMethod() calls for slot accessors.
#[derive(miniextendr_api::ExternalPtr, Debug)]
#[externalptr(s4)]
pub struct SidecarS4 {
    #[r_data]
    _r: RSidecar,

    #[r_data]
    pub slot_int: i32,

    #[r_data]
    pub slot_real: f64,

    #[r_data]
    pub slot_str: String,
}

/// S4 class registration for SidecarS4 (enables setMethod slot accessors).
#[miniextendr(s4)]
impl SidecarS4 {}

/// Test creating a SidecarS4 with S4 slot accessors.
/// @param slot_int Integer sidecar field.
/// @param slot_real Numeric sidecar field.
/// @param slot_str Character sidecar field.
#[miniextendr]
pub fn rdata_sidecar_s4_new(
    slot_int: i32,
    slot_real: f64,
    slot_str: String,
) -> ExternalPtr<SidecarS4> {
    ExternalPtr::new(SidecarS4 {
        _r: RSidecar,
        slot_int,
        slot_real,
        slot_str,
    })
}
// endregion

// region: S7 - properties via new_property()

/// Demonstrates S7 class system.
/// Generates standalone accessors that can be wrapped with S7::new_property().
#[derive(miniextendr_api::ExternalPtr, Debug)]
#[externalptr(s7)]
pub struct SidecarS7 {
    #[r_data]
    _r: RSidecar,

    #[r_data(prop_doc = "An integer sidecar property.")]
    pub prop_int: i32,

    #[r_data(prop_doc = "A logical sidecar property.")]
    pub prop_flag: bool,

    #[r_data(prop_doc = "A character sidecar property.")]
    pub prop_name: String,
}

/// S7 class registration for SidecarS7 with property-based sidecar accessors.
#[miniextendr(s7(r_data_accessors))]
impl SidecarS7 {
    /// Create a new SidecarS7 with initial values.
    /// @param prop_int Integer sidecar field.
    /// @param prop_flag Logical sidecar field.
    /// @param prop_name Character sidecar field.
    pub fn new(prop_int: i32, prop_flag: bool, prop_name: String) -> Self {
        SidecarS7 {
            _r: RSidecar,
            prop_int,
            prop_flag,
            prop_name,
        }
    }
}

/// Test creating a SidecarS7 with S7 property-based sidecar accessors.
/// @param prop_int Integer sidecar field.
/// @param prop_flag Logical sidecar field.
/// @param prop_name Character sidecar field.
#[miniextendr]
pub fn rdata_sidecar_s7_new(
    prop_int: i32,
    prop_flag: bool,
    prop_name: String,
) -> ExternalPtr<SidecarS7> {
    ExternalPtr::new(SidecarS7 {
        _r: RSidecar,
        prop_int,
        prop_flag,
        prop_name,
    })
}
// endregion

// region: Vctrs - S3-style $ dispatch for vctrs compatibility

/// Demonstrates vctrs class system.
/// Generates S3-style $.class and $<-.class methods like S3.
#[derive(miniextendr_api::ExternalPtr, Debug)]
#[externalptr(vctrs)]
pub struct SidecarVctrs {
    #[r_data]
    _r: RSidecar,

    #[r_data]
    pub vec_data: Vec<f64>,

    #[r_data]
    pub vec_label: String,
}

/// Vctrs class registration for SidecarVctrs (S3-style $ dispatch for vctrs compat).
#[miniextendr(vctrs)]
impl SidecarVctrs {}

/// Test creating a SidecarVctrs with vctrs-compatible sidecar fields.
/// @param vec_data Numeric vector sidecar field.
/// @param vec_label Character sidecar field.
#[miniextendr]
pub fn rdata_sidecar_vctrs_new(vec_data: Vec<f64>, vec_label: String) -> ExternalPtr<SidecarVctrs> {
    ExternalPtr::new(SidecarVctrs {
        _r: RSidecar,
        vec_data,
        vec_label,
    })
}
// endregion

// region: Raw SEXP slot comprehensive tests

/// Tests raw SEXP slot functionality with various R types.
#[derive(miniextendr_api::ExternalPtr, Debug)]
#[externalptr(env)]
pub struct SidecarRawSexp {
    #[r_data]
    _r: RSidecar,

    /// Can store any SEXP - integer vector
    #[r_data]
    pub int_vec: SEXP,

    /// Can store any SEXP - real vector
    #[r_data]
    pub real_vec: SEXP,

    /// Can store any SEXP - character vector
    #[r_data]
    pub char_vec: SEXP,

    /// Can store any SEXP - list
    #[r_data]
    pub list_val: SEXP,

    /// Can store any SEXP - function/closure
    #[r_data]
    pub func_val: SEXP,

    /// Can store any SEXP - environment
    #[r_data]
    pub env_val: SEXP,
}

/// Env class registration for SidecarRawSexp (raw SEXP slot testing).
#[miniextendr(env)]
impl SidecarRawSexp {}

/// Test creating a SidecarRawSexp with all SEXP slots initialized to NULL.
#[miniextendr]
pub fn rdata_sidecar_rawsexp_new() -> ExternalPtr<SidecarRawSexp> {
    use miniextendr_api::ffi::SEXP;

    ExternalPtr::new(SidecarRawSexp {
        _r: RSidecar,
        int_vec: SEXP::nil(),
        real_vec: SEXP::nil(),
        char_vec: SEXP::nil(),
        list_val: SEXP::nil(),
        func_val: SEXP::nil(),
        env_val: SEXP::nil(),
    })
}
// endregion

// region: u8 (raw) scalar test

/// Tests u8 scalar field (maps to R raw).
#[derive(miniextendr_api::ExternalPtr, Debug)]
#[externalptr(env)]
pub struct SidecarRaw {
    #[r_data]
    _r: RSidecar,

    #[r_data]
    pub byte_val: u8,
}

/// Env class registration for SidecarRaw (u8 scalar sidecar testing).
#[miniextendr(env)]
impl SidecarRaw {}

/// Test creating a SidecarRaw with a u8 scalar sidecar field.
/// @param byte_val Raw byte value for the sidecar field.
#[miniextendr]
pub fn rdata_sidecar_raw_new(byte_val: u8) -> ExternalPtr<SidecarRaw> {
    ExternalPtr::new(SidecarRaw {
        _r: RSidecar,
        byte_val,
    })
}
// endregion

// region: Module registration
// endregion
