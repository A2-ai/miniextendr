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

/// @noRd
#[miniextendr(env)]
impl SidecarEnv {}

/// @noRd
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
        raw_slot: SEXP::null(),
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

/// @noRd
#[miniextendr(r6(r_data_accessors))]
impl SidecarR6 {
    /// Create a new SidecarR6 with initial values.
    pub fn new(value: i32, label: String) -> Self {
        SidecarR6 {
            _r: RSidecar,
            value,
            label,
        }
    }
}

/// @noRd
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

/// @noRd
#[miniextendr(s3)]
impl SidecarS3 {}

/// @noRd
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

/// @noRd
#[miniextendr(s4)]
impl SidecarS4 {}

/// @noRd
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

    #[r_data]
    pub prop_int: i32,

    #[r_data]
    pub prop_flag: bool,

    #[r_data]
    pub prop_name: String,
}

/// @noRd
#[miniextendr(s7(r_data_accessors))]
impl SidecarS7 {
    /// Create a new SidecarS7 with initial values.
    pub fn new(prop_int: i32, prop_flag: bool, prop_name: String) -> Self {
        SidecarS7 {
            _r: RSidecar,
            prop_int,
            prop_flag,
            prop_name,
        }
    }
}

/// @noRd
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

/// @noRd
#[miniextendr(vctrs)]
impl SidecarVctrs {}

/// @noRd
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

/// @noRd
#[miniextendr(env)]
impl SidecarRawSexp {}

/// @noRd
#[miniextendr]
pub fn rdata_sidecar_rawsexp_new() -> ExternalPtr<SidecarRawSexp> {
    use miniextendr_api::ffi::SEXP;

    ExternalPtr::new(SidecarRawSexp {
        _r: RSidecar,
        int_vec: SEXP::null(),
        real_vec: SEXP::null(),
        char_vec: SEXP::null(),
        list_val: SEXP::null(),
        func_val: SEXP::null(),
        env_val: SEXP::null(),
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

/// @noRd
#[miniextendr(env)]
impl SidecarRaw {}

/// @noRd
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
