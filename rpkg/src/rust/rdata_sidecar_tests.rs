//! Tests for RSidecar and `#[r_data]` functionality.
//!
//! This module tests the R-side sidecar accessor generation with different class systems.

use miniextendr_api::externalptr::{ExternalPtr, RSidecar, RSlot};
use miniextendr_api::into_r::IntoR;
use miniextendr_api::miniextendr;
use miniextendr_api::prelude::{SEXP, SexpExt};

// region: Env (default) - standalone functions: Type_get_field(), Type_set_field()

/// Demonstrates env class system (default).
/// Generates: SidecarEnv_get_count(), SidecarEnv_set_count(), etc.
#[derive(miniextendr_api::ExternalPtr, Debug)]
#[externalptr(env)]
pub struct SidecarEnv {
    /// Regular Rust field (not exposed to R) - intentionally unused to demonstrate
    /// that non-`#[r_data]` fields are private to Rust.
    _internal_value: i32,

    /// Selector - enables R accessors
    #[r_data]
    _r: RSidecar,

    /// Zero-overhead scalar: i32
    #[r_data]
    pub count: i32,

    /// Zero-overhead scalar: f64
    #[r_data(setter = "visible")]
    pub score: f64,

    /// Zero-overhead scalar: bool
    #[r_data(setter = "invisible")]
    pub flag: bool,

    /// Conversion type: String
    #[r_data]
    pub name: String,

    /// R value, kept in the external pointer's protection list
    #[r_data]
    pub raw_slot: RSlot,
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
    ExternalPtr::new(SidecarEnv {
        _internal_value: 999,
        _r: RSidecar,
        count,
        score,
        flag,
        name,
        raw_slot: RSlot,
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

    #[r_data(setter = "visible")]
    pub label: String,
}

/// R6 class registration for SidecarR6 with active binding sidecar accessors.
/// @field value Integer sidecar field (active binding).
/// @field label Character sidecar field (active binding); direct setter calls return visibly.
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
/// @examples
/// obj <- SidecarR6$new(1L, "before")
/// set_label <- activeBindingFunction("label", obj)
/// result <- withVisible(set_label("after"))
/// stopifnot(result$visible, identical(result$value, obj), obj$label == "after")
/// stopifnot(!withVisible(obj$label <- "assigned")$visible)
#[miniextendr]
pub fn rdata_sidecar_r6_new(value: i32, label: String) -> ExternalPtr<SidecarR6> {
    ExternalPtr::new(SidecarR6 {
        _r: RSidecar,
        value,
        label,
    })
}
// endregion

// region: S3 - standalone accessors: SidecarS3_get_data(x), SidecarS3_set_data(x, value)

/// Demonstrates S3 class system.
/// Generates the standalone `SidecarS3_get_data()` / `SidecarS3_set_data()`.
#[derive(miniextendr_api::ExternalPtr, Debug)]
#[externalptr(s3)]
pub struct SidecarS3 {
    #[r_data]
    _r: RSidecar,

    #[r_data]
    pub data: f64,
}

/// S3 class registration for SidecarS3.
#[miniextendr(s3)]
impl SidecarS3 {}

/// Test creating a SidecarS3 with a sidecar field.
/// @param data Numeric sidecar field.
#[miniextendr]
pub fn rdata_sidecar_s3_new(data: f64) -> ExternalPtr<SidecarS3> {
    ExternalPtr::new(SidecarS3 { _r: RSidecar, data })
}
// endregion

// region: S4 - standalone accessors: SidecarS4_get_slot_int(x), ...

/// Demonstrates S4 class system.
/// Generates the standalone `SidecarS4_get_*()` / `SidecarS4_set_*()`.
#[derive(miniextendr_api::ExternalPtr, Debug)]
#[externalptr(s4)]
pub struct SidecarS4 {
    #[r_data]
    _r: RSidecar,

    #[r_data]
    pub slot_int: i32,

    #[r_data(setter = "visible")]
    pub slot_real: f64,

    #[r_data]
    pub slot_str: String,
}

/// S4 class registration for SidecarS4.
#[miniextendr(s4)]
impl SidecarS4 {}

/// Test creating a SidecarS4 with sidecar fields.
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
/// With `s7(r_data_accessors)`, the fields are S7 properties (`obj@prop_int`).
#[derive(miniextendr_api::ExternalPtr, Debug)]
#[externalptr(s7)]
pub struct SidecarS7 {
    #[r_data]
    _r: RSidecar,

    #[r_data(prop_doc = "An integer sidecar property.", setter = "visible")]
    pub prop_int: i32,

    #[r_data(prop_doc = "A logical sidecar property.")]
    pub prop_flag: bool,

    #[r_data(prop_doc = "A character sidecar property.", setter = "invisible")]
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

// region: Vctrs - standalone accessors

/// Demonstrates vctrs class system.
/// Generates the standalone `SidecarVctrs_get_*()` / `SidecarVctrs_set_*()`.
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

/// Vctrs class registration for SidecarVctrs.
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

// region: RSlot fields holding various R types

/// Tests `RSlot` fields with various R types.
#[derive(miniextendr_api::ExternalPtr, Debug)]
#[externalptr(env)]
pub struct SidecarRawSexp {
    #[r_data]
    _r: RSidecar,

    /// Any R value - integer vector
    #[r_data]
    pub int_vec: RSlot,

    /// Any R value - real vector
    #[r_data]
    pub real_vec: RSlot,

    /// Any R value - character vector
    #[r_data]
    pub char_vec: RSlot,

    /// Any R value - list
    #[r_data]
    pub list_val: RSlot,

    /// Any R value - function/closure
    #[r_data]
    pub func_val: RSlot,

    /// Any R value - environment
    #[r_data]
    pub env_val: RSlot,
}

/// Env class registration for SidecarRawSexp (raw SEXP slot testing).
#[miniextendr(env)]
impl SidecarRawSexp {}

/// Test creating a SidecarRawSexp with all RSlot fields unset (`NULL`).
#[miniextendr]
pub fn rdata_sidecar_rawsexp_new() -> ExternalPtr<SidecarRawSexp> {
    ExternalPtr::new(SidecarRawSexp {
        _r: RSidecar,
        int_vec: RSlot,
        real_vec: RSlot,
        char_vec: RSlot,
        list_val: RSlot,
        func_val: RSlot,
        env_val: RSlot,
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

// region: Panic / error-transport fixtures

/// Field type whose `IntoR` conversion always panics.
///
/// Used to prove that a panic inside a sidecar getter body is caught by the
/// accessor's `with_r_unwind_protect` guard and re-raised as a structured R
/// condition instead of unwinding across the `extern "C-unwind"` FFI frame.
#[derive(Debug, Clone, Default)]
pub struct PanicOnGet;

impl miniextendr_api::into_r::IntoR for PanicOnGet {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        panic!("intentional panic in sidecar getter (PanicOnGet)")
    }
}

impl miniextendr_api::TryFromSexp for PanicOnGet {
    type Error = &'static str;

    fn try_from_sexp(_sexp: SEXP) -> Result<Self, Self::Error> {
        Err("PanicOnGet cannot be constructed from an R value")
    }
}

/// Sidecar struct whose only slot panics on read (and fails conversion on
/// write) — exercises the accessor guard + condition transport end to end.
#[derive(miniextendr_api::ExternalPtr, Debug)]
#[externalptr(env)]
pub struct SidecarPanicky {
    #[r_data]
    _r: RSidecar,

    /// Getter panics; setter always fails conversion.
    #[r_data]
    pub boom: PanicOnGet,
}

/// Env class registration for SidecarPanicky (panic-transport testing).
#[miniextendr(env)]
impl SidecarPanicky {}

/// Test creating a SidecarPanicky whose `boom` getter panics on access.
#[miniextendr]
pub fn rdata_sidecar_panicky_new() -> ExternalPtr<SidecarPanicky> {
    ExternalPtr::new(SidecarPanicky {
        _r: RSidecar,
        boom: PanicOnGet,
    })
}
// endregion

// region: RSlot fields on an R6 class, filled and read from Rust

/// R6 class whose `keys` field is an R value the constructor builds in Rust.
///
/// `cache` is a private `RSlot`: no R accessor, only Rust reaches it.
#[derive(miniextendr_api::ExternalPtr, Debug, Clone)]
#[externalptr(r6)]
pub struct SidecarSlotR6 {
    #[r_data]
    _r: RSidecar,

    #[r_data]
    pub keys: RSlot,

    #[r_data]
    pub n: i32,

    #[r_data]
    cache: RSlot,
}

/// R6 class with `RSlot` sidecar fields.
/// @field keys Integer keys `1:n`, built by the constructor (active binding).
/// @field n Number of keys the constructor built (active binding).
#[miniextendr(r6(r_data_accessors))]
impl SidecarSlotR6 {
    /// Create a SidecarSlotR6 whose `keys` are `1:n`, set from Rust.
    /// @param n Number of keys.
    #[miniextendr(r6(constructor))]
    pub fn new(n: i32) -> ExternalPtr<Self> {
        let ptr = ExternalPtr::new(SidecarSlotR6 {
            _r: RSidecar,
            keys: RSlot,
            n,
            cache: RSlot,
        });
        // `set_r_slot` allocates nothing, so the fresh vector needs no
        // protection before the slot roots it.
        ptr.set_r_slot(Self::KEYS_SLOT, (1..=n).collect::<Vec<i32>>().into_sexp());
        ptr
    }

    /// Length of the `keys` value, read from Rust.
    pub fn key_count(self: &ExternalPtr<Self>) -> i32 {
        i32::try_from(self.r_slot(Self::KEYS_SLOT).len()).expect("keys length exceeds i32")
    }

    /// Keep `value` in the private `cache` slot.
    /// @param value Any R value.
    pub fn remember(self: &ExternalPtr<Self>, value: SEXP) {
        self.set_r_slot(Self::CACHE_SLOT, value);
    }

    /// The value `remember()` kept; `NULL` before.
    pub fn recall(self: &ExternalPtr<Self>) -> SEXP {
        self.r_slot(Self::CACHE_SLOT)
    }

    /// A clone: a new Rust value, with the same R values in its slots.
    pub fn duplicate(self: &ExternalPtr<Self>) -> ExternalPtr<Self> {
        ExternalPtr::clone(self)
    }
}

/// Store fresh R values in `RSlot` fields, churn the GC, and read them back.
///
/// Regression fixture for #1846: a sidecar field used to hold a bare `SEXP`
/// that nothing rooted, so a value R no longer referenced was freed under the
/// pointer. Each value here is referenced only by its slot. The clone at the
/// end must share them.
///
/// No arguments — picked up by the fast `gctorture(TRUE)` no-arg sweep (#430).
#[miniextendr(noexport)]
pub fn gc_stress_sidecar_r_slot() {
    let handles: Vec<ExternalPtr<SidecarSlotR6>> = (1..=8).map(SidecarSlotR6::new).collect();
    for (i, handle) in (0i32..).zip(&handles) {
        handle.set_r_slot(SidecarSlotR6::CACHE_SLOT, vec![i; 16].into_sexp());
    }

    for i in 0..64 {
        let _garbage = vec![f64::from(i); 64].into_sexp();
    }

    for (i, handle) in (0i32..).zip(&handles) {
        let keys: Vec<i32> =
            miniextendr_api::TryFromSexp::try_from_sexp(handle.r_slot(SidecarSlotR6::KEYS_SLOT))
                .expect("keys slot holds an integer vector");
        assert_eq!(keys, (1..=i + 1).collect::<Vec<i32>>());
        let cache: Vec<i32> =
            miniextendr_api::TryFromSexp::try_from_sexp(handle.r_slot(SidecarSlotR6::CACHE_SLOT))
                .expect("cache slot holds an integer vector");
        assert_eq!(cache, vec![i; 16]);
    }

    let copy = ExternalPtr::clone(&handles[3]);
    assert_eq!(
        copy.r_slot(SidecarSlotR6::KEYS_SLOT),
        handles[3].r_slot(SidecarSlotR6::KEYS_SLOT)
    );
    assert_eq!(
        copy.r_slot(SidecarSlotR6::CACHE_SLOT),
        handles[3].r_slot(SidecarSlotR6::CACHE_SLOT)
    );
}
// endregion

// region: Module registration
// endregion
