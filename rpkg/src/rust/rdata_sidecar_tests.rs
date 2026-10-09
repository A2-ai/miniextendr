//! Tests for RSidecar and `#[r_data]` functionality.
//!
//! This module tests the R-side sidecar accessor generation with different
//! class systems, and the typed `Sidecar<T>` fields whose values live in the
//! external pointer's protection list (#1846, #1855), read and written
//! through the `ExternalPtr` handle (#1856).

use miniextendr_api::externalptr::{ErasedExternalPtr, ExternalPtr, RSidecar, Sidecar};
use miniextendr_api::into_r::IntoR;
use miniextendr_api::miniextendr;
use miniextendr_api::prelude::SEXP;
use miniextendr_api::{List, SEXPTYPE, SexpExt, TryFromSexp};

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

    /// Any R value, kept unconverted in the external pointer's protection list
    #[r_data]
    pub raw_slot: Sidecar<SEXP>,
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
        raw_slot: Sidecar::new(SEXP::nil()),
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
/// Generates the standalone `SidecarS3_get_data()` / `SidecarS3_set_data()`,
/// and S3 methods that take the handle to reach the `tags` sidecar value.
#[derive(miniextendr_api::ExternalPtr, Debug)]
#[externalptr(s3)]
pub struct SidecarS3 {
    #[r_data]
    _r: RSidecar,

    #[r_data]
    pub data: f64,

    /// Tags kept in the pointer's protection list.
    #[r_data]
    pub tags: Sidecar<Vec<String>>,
}

/// S3 class registration for SidecarS3: the methods take the handle, which
/// is how a method reads a `Sidecar` value.
#[miniextendr(s3)]
impl SidecarS3 {
    /// Create a SidecarS3 with no tags.
    /// @param data Numeric sidecar field.
    pub fn new(data: f64) -> Self {
        SidecarS3 {
            _r: RSidecar,
            data,
            tags: Sidecar::new(vec![]),
        }
    }

    /// Number of `tags`, read through the handle.
    pub fn tag_count(self: &ExternalPtr<Self>) -> i32 {
        i32::try_from(Self::tags(self).len()).expect("tag count exceeds i32")
    }

    /// Appends `tag` to `tags` through the handle.
    /// @param tag The tag to append.
    pub fn add_tag(self: &mut ExternalPtr<Self>, tag: String) {
        let mut tags = Self::tags(self);
        tags.push(tag);
        Self::set_tags(self, tags);
    }
}

/// Test creating a SidecarS3 with a sidecar field.
/// @param data Numeric sidecar field.
#[miniextendr]
pub fn rdata_sidecar_s3_new(data: f64) -> ExternalPtr<SidecarS3> {
    ExternalPtr::new(SidecarS3::new(data))
}
// endregion

// region: S4 - standalone accessors: SidecarS4_get_slot_int(x), ...

/// Demonstrates S4 class system.
/// Generates the standalone `SidecarS4_get_*()` / `SidecarS4_set_*()`, and
/// S4 methods that take the handle to reach the `history` sidecar value.
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

    /// Values `record()` kept, in the pointer's protection list.
    #[r_data]
    pub history: Sidecar<Vec<f64>>,
}

/// S4 class registration for SidecarS4: the methods take the handle, which
/// is how a method reads a `Sidecar` value.
#[miniextendr(s4)]
impl SidecarS4 {
    /// Create a SidecarS4 with an empty history.
    /// @param slot_int Integer sidecar field.
    /// @param slot_real Numeric sidecar field.
    /// @param slot_str Character sidecar field.
    pub fn new(slot_int: i32, slot_real: f64, slot_str: String) -> Self {
        SidecarS4 {
            _r: RSidecar,
            slot_int,
            slot_real,
            slot_str,
            history: Sidecar::new(vec![]),
        }
    }

    /// Number of values `record()` kept, read through the handle.
    pub fn history_len(self: &ExternalPtr<Self>) -> i32 {
        i32::try_from(Self::history(self).len()).expect("history length exceeds i32")
    }

    /// Appends `value` to `history` through the handle.
    /// @param value The value to keep.
    pub fn record(self: &mut ExternalPtr<Self>, value: f64) {
        let mut history = Self::history(self);
        history.push(value);
        Self::set_history(self, history);
    }
}

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
    ExternalPtr::new(SidecarS4::new(slot_int, slot_real, slot_str))
}
// endregion

// region: S7 - properties via new_property()

/// Demonstrates S7 class system.
/// With `s7(r_data_accessors)`, the fields are S7 properties (`obj@prop_int`),
/// and the methods take the handle to reach the `scores` sidecar value.
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

    /// Scores kept in the pointer's protection list.
    #[r_data(prop_doc = "Scores kept in the pointer's protection list.")]
    pub scores: Sidecar<Vec<f64>>,
}

/// S7 class registration for SidecarS7 with property-based sidecar accessors.
#[miniextendr(s7(r_data_accessors))]
impl SidecarS7 {
    /// Create a new SidecarS7 with initial values and no scores.
    /// @param prop_int Integer sidecar field.
    /// @param prop_flag Logical sidecar field.
    /// @param prop_name Character sidecar field.
    pub fn new(prop_int: i32, prop_flag: bool, prop_name: String) -> Self {
        SidecarS7 {
            _r: RSidecar,
            prop_int,
            prop_flag,
            prop_name,
            scores: Sidecar::new(vec![]),
        }
    }

    /// Sum of `scores`, read through the handle.
    pub fn score_total(self: &ExternalPtr<Self>) -> f64 {
        Self::scores(self).iter().sum()
    }

    /// Appends `score` to `scores` through the handle.
    /// @param score The score to append.
    pub fn add_score(self: &mut ExternalPtr<Self>, score: f64) {
        let mut scores = Self::scores(self);
        scores.push(score);
        Self::set_scores(self, scores);
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
    ExternalPtr::new(SidecarS7::new(prop_int, prop_flag, prop_name))
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

// region: Sidecar<SEXP> fields holding various R types

/// Tests `Sidecar<SEXP>` fields with various R types.
#[derive(miniextendr_api::ExternalPtr, Debug)]
#[externalptr(env)]
pub struct SidecarRawSexp {
    #[r_data]
    _r: RSidecar,

    /// Any R value - integer vector
    #[r_data]
    pub int_vec: Sidecar<SEXP>,

    /// Any R value - real vector
    #[r_data]
    pub real_vec: Sidecar<SEXP>,

    /// Any R value - character vector
    #[r_data]
    pub char_vec: Sidecar<SEXP>,

    /// Any R value - list
    #[r_data]
    pub list_val: Sidecar<SEXP>,

    /// Any R value - function/closure
    #[r_data]
    pub func_val: Sidecar<SEXP>,

    /// Any R value - environment
    #[r_data]
    pub env_val: Sidecar<SEXP>,
}

/// Env class registration for SidecarRawSexp (raw SEXP slot testing).
#[miniextendr(env)]
impl SidecarRawSexp {}

/// Test creating a SidecarRawSexp with every `Sidecar<SEXP>` field `NULL`.
#[miniextendr]
pub fn rdata_sidecar_rawsexp_new() -> ExternalPtr<SidecarRawSexp> {
    ExternalPtr::new(SidecarRawSexp {
        _r: RSidecar,
        int_vec: Sidecar::new(SEXP::nil()),
        real_vec: Sidecar::new(SEXP::nil()),
        char_vec: Sidecar::new(SEXP::nil()),
        list_val: Sidecar::new(SEXP::nil()),
        func_val: Sidecar::new(SEXP::nil()),
        env_val: Sidecar::new(SEXP::nil()),
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

// region: Typed Sidecar<T> fields on an R6 class, filled and read from Rust

/// R6 class whose `keys` live in the external pointer's protection list,
/// filled by a plain constructor and read back from Rust through the handle.
///
/// `keys` gets both Rust accessors, `label` only the getter, `note` only
/// the setter; `cache` is private, so only Rust reaches it. R gets its
/// accessors for every `pub` field whatever the Rust option. A method that
/// needs a sidecar value takes the handle (`self: &ExternalPtr<Self>` /
/// `self: &mut ExternalPtr<Self>`), since the struct holds nothing that
/// points back at its pointer (#1856).
#[derive(miniextendr_api::ExternalPtr, Debug, Clone)]
#[externalptr(r6)]
pub struct SidecarSlotR6 {
    #[r_data]
    _r: RSidecar,

    /// Integer keys, `1:n` from the constructor.
    #[r_data(ref, mut)]
    pub keys: Sidecar<Vec<i32>>,

    /// Number of keys the constructor built (a struct field).
    #[r_data]
    pub n: i32,

    /// A label Rust only reads; R reads and writes it.
    #[r_data(ref)]
    pub label: Sidecar<String>,

    /// A note Rust only writes; R reads and writes it.
    #[r_data(mut)]
    pub note: Sidecar<String>,

    /// A private cache: no R accessor.
    #[r_data]
    cache: Sidecar<Option<List>>,
}

/// A trait impl receives the struct, not the handle, so it can show only the
/// struct fields; the sidecar values are out of its reach.
impl std::fmt::Display for SidecarSlotR6 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "n={}", self.n)
    }
}

/// R6 class with typed `Sidecar<T>` fields.
/// @field keys Integer keys `1:n`, built by the constructor (active binding).
/// @field n Number of keys the constructor built (active binding).
/// @field label A label the constructor sets to `"fresh"` (active binding).
/// @field note A note Rust writes with `write_note()` (active binding).
#[miniextendr(r6(r_data_accessors))]
impl SidecarSlotR6 {
    /// Create a SidecarSlotR6 whose `keys` are `1:n`, set from Rust.
    /// @param n Number of keys.
    pub fn new(n: i32) -> Self {
        SidecarSlotR6 {
            _r: RSidecar,
            keys: Sidecar::new((1..=n).collect()),
            n,
            label: Sidecar::new(String::from("fresh")),
            note: Sidecar::new(String::new()),
            cache: Sidecar::default(),
        }
    }

    /// Length of `keys`, read from Rust through the handle.
    pub fn key_count(self: &ExternalPtr<Self>) -> i32 {
        i32::try_from(Self::keys(self).len()).expect("keys length exceeds i32")
    }

    /// Length of `label`, read from Rust (a `ref`-only field).
    pub fn label_len(self: &ExternalPtr<Self>) -> i32 {
        i32::try_from(Self::label(self).len()).expect("label length exceeds i32")
    }

    /// Writes `note` from Rust (a `mut`-only field).
    /// @param note The new note.
    pub fn write_note(self: &mut ExternalPtr<Self>, note: String) {
        Self::set_note(self, note);
    }

    /// Appends `key` to `keys` from Rust, through the mutable handle.
    /// @param key The key to append.
    pub fn push_key(self: &mut ExternalPtr<Self>, key: i32) {
        let mut keys = Self::keys(self);
        keys.push(key);
        Self::set_keys(self, keys);
    }

    /// Keep `value` in the private `cache`.
    /// @param value Any R list.
    pub fn remember(self: &mut ExternalPtr<Self>, value: List) {
        Self::set_cache(self, Some(value));
    }

    /// The list `remember()` kept; `NULL` before.
    pub fn recall(self: &ExternalPtr<Self>) -> SEXP {
        Self::cache(self).into_sexp()
    }

    /// Reads `keys` from another thread: an error, since sidecar accessors
    /// run on R's main thread only. Returns the panic message.
    pub fn keys_off_thread(self: &ExternalPtr<Self>) -> String {
        struct SendPtr(*const ExternalPtr<SidecarSlotR6>);
        // SAFETY: the pointee outlives the joined thread, and the accessor
        // panics on the thread check before it touches R.
        unsafe impl Send for SendPtr {}
        let ptr = SendPtr(std::ptr::from_ref(self));
        let outcome = std::thread::spawn(move || {
            let ptr = ptr;
            let _ = SidecarSlotR6::keys(unsafe { &*ptr.0 });
        })
        .join();
        match outcome {
            Ok(()) => String::from("no error"),
            Err(payload) => {
                miniextendr_api::unwind_protect::panic_payload_to_string(&*payload).into_owned()
            }
        }
    }

    /// A copy of this object in a new external pointer: `ExternalPtr::clone`
    /// copies the struct and shares the sidecar values as R objects, so a
    /// write to a field on either side replaces that side's slot only.
    pub fn duplicate(self: &ExternalPtr<Self>) -> ExternalPtr<Self> {
        self.clone()
    }

    /// Replaces the whole value through `DerefMut`: `n` keys again, no cache.
    /// Both sides see the new values at their next access.
    /// @param n Number of keys.
    pub fn reset(self: &mut ExternalPtr<Self>, n: i32) {
        **self = Self::new(n);
    }

    /// Rebuilds the value by value (`self -> Self`): `n` grows by `extra` and
    /// the private `cache` is set to `list(extra)`; `keys`, `label` and
    /// `note` stay in the pointer across the call.
    /// @param extra Added to `n` and kept in the cache.
    pub fn rebuilt(self, extra: i32) -> Self {
        Self {
            n: self.n + extra,
            cache: Sidecar::new(Some(List::from_values(vec![extra]))),
            ..self
        }
    }
}

/// RDisplay trait ABI registration for SidecarSlotR6: `as_r_string()` goes
/// through the trait-impl receiver path, which sees the struct only.
#[miniextendr(r6)]
impl miniextendr_api::adapter_traits::RDisplay for SidecarSlotR6 {}

/// Swaps the Rust values of two pointers. The sidecar values stay with each
/// pointer: they live in its protection list.
/// @param a,b Two `SidecarSlotR6` pointers.
#[miniextendr]
pub fn sidecar_swap(mut a: ExternalPtr<SidecarSlotR6>, mut b: ExternalPtr<SidecarSlotR6>) {
    std::mem::swap(&mut *a, &mut *b);
}

/// Moves the value out of `ptr` (whose R object is cleared) and wraps it
/// again: the sidecar values of `ptr` travel with it.
/// @param ptr A `SidecarSlotR6` pointer.
#[miniextendr]
pub fn sidecar_rewrap(ptr: ExternalPtr<SidecarSlotR6>) -> ExternalPtr<SidecarSlotR6> {
    ExternalPtr::new(ExternalPtr::into_inner(ptr))
}

/// Clones the handle: a new pointer sharing the sidecar values as R objects.
/// @param ptr A `SidecarSlotR6` pointer.
#[miniextendr]
pub fn sidecar_clone_handle(ptr: ExternalPtr<SidecarSlotR6>) -> ExternalPtr<SidecarSlotR6> {
    ptr.clone()
}

/// Clones `source`'s value into `target` in place (`ExternalPtr::clone_from`):
/// `target` reads `source`'s struct fields and sidecar values afterwards.
/// @param target,source Two `SidecarSlotR6` pointers.
#[miniextendr]
pub fn sidecar_clone_from(
    mut target: ExternalPtr<SidecarSlotR6>,
    source: ExternalPtr<SidecarSlotR6>,
) {
    target.clone_from(&source);
}

/// The `keys` of a pointer, read through the handle accessor
/// `SidecarSlotR6::keys(&ptr)`.
/// @param ptr A `SidecarSlotR6` pointer.
#[miniextendr]
pub fn sidecar_keys_via_handle(ptr: ExternalPtr<SidecarSlotR6>) -> Vec<i32> {
    SidecarSlotR6::keys(&ptr)
}

/// A wrapped struct holding another sidecar object by its handle: the inner
/// handle keeps the inner pointer alive, and reads its own pointer's values.
/// That is the pattern for nesting a struct with `Sidecar` fields.
#[derive(miniextendr_api::ExternalPtr, Debug)]
#[externalptr(r6)]
pub struct SidecarNest {
    #[r_data]
    _r: RSidecar,

    /// The inner object's handle, a plain Rust field.
    inner: ExternalPtr<SidecarSlotR6>,

    /// The outer struct's own sidecar field.
    #[r_data]
    pub tag: Sidecar<String>,
}

/// R6 class nesting a `SidecarSlotR6` handle as a plain field.
/// @field tag The outer struct's own sidecar value (active binding).
#[miniextendr(r6(r_data_accessors))]
impl SidecarNest {
    /// Create a SidecarNest whose inner object has `n` keys.
    /// @param n Number of inner keys.
    pub fn new(n: i32) -> Self {
        SidecarNest {
            _r: RSidecar,
            inner: ExternalPtr::new(SidecarSlotR6::new(n)),
            tag: Sidecar::new(String::from("outer")),
        }
    }

    /// The inner object's `keys`, through its handle.
    pub fn inner_keys(&self) -> Vec<i32> {
        SidecarSlotR6::keys(&self.inner)
    }

    /// Writes the inner object's `keys` through its handle.
    /// @param keys The new inner keys.
    pub fn set_inner_keys(&mut self, keys: Vec<i32>) {
        SidecarSlotR6::set_keys(&mut self.inner, keys);
    }
}

/// Rewrites the version in the type ID a restored pointer keeps in
/// `prot[0]` (`crate@version::module::Type`), so the pointer reads as a
/// save by `version` of this package. `x` must be an external pointer
/// without an address (after `readRDS()` / `unserialize()`), as a saved
/// pointer comes back; the type ID is left as it is for a live one.
/// @param x A restored external pointer.
/// @param version The version to write, e.g. `"0.0.1"`.
#[miniextendr(noexport)]
pub fn sidecar_rewrite_saved_version(x: SEXP, version: String) {
    assert_eq!(
        x.type_of(),
        SEXPTYPE::EXTPTRSXP,
        "`x` is an external pointer"
    );
    // SAFETY: on R's main thread (a `#[miniextendr]` fn); `x` is an EXTPTRSXP.
    unsafe { miniextendr_api::externalptr::rewrite_stored_version_for_tests(x, &version) };
}

/// Allocate garbage so a GC runs between the writes and the reads.
fn churn() {
    for i in 0..64 {
        let _garbage = vec![f64::from(i); 64].into_sexp();
    }
}

/// Flush, read, write, clone, swap and load `Sidecar<T>` fields through the
/// handle, churn the GC between the steps, and read every value back.
///
/// Regression fixture for #1846: a sidecar field used to hold a bare `SEXP`
/// that nothing rooted. Here every value is referenced only by its slot. No
/// arguments — picked up by the fast `gctorture(TRUE)` no-arg sweep (#430).
#[miniextendr(noexport)]
pub fn gc_stress_sidecar_fields() {
    // Flush at creation; writes through the handle.
    let mut handles: Vec<ExternalPtr<SidecarSlotR6>> = (1..=8)
        .map(|n| ExternalPtr::new(SidecarSlotR6::new(n)))
        .collect();
    for (i, handle) in (0i32..).zip(handles.iter_mut()) {
        SidecarSlotR6::set_cache(handle, Some(List::from_values(vec![i; 16])));
        SidecarSlotR6::push_key(handle, 100 + i);
    }
    churn();
    for (i, handle) in (0i32..).zip(&handles) {
        let mut expected: Vec<i32> = (1..=i + 1).collect();
        expected.push(100 + i);
        assert_eq!(SidecarSlotR6::keys(handle), expected);
        let cache = SidecarSlotR6::cache(handle).expect("cache was set");
        assert_eq!(cache.len(), 16);
        assert_eq!(cache.get_index::<i32>(0), Some(i));
    }

    // Clone: the slot values are shared; a write replaces one side's slot.
    let mut copy = ExternalPtr::clone(&handles[3]);
    churn();
    assert_eq!(SidecarSlotR6::keys(&copy), SidecarSlotR6::keys(&handles[3]));
    SidecarSlotR6::set_keys(&mut copy, vec![-1]);
    assert_eq!(SidecarSlotR6::keys(&copy), vec![-1]);
    assert_eq!(SidecarSlotR6::keys(&handles[3]), vec![1, 2, 3, 4, 103]);

    // `clone_from`: the target reads the source's values afterwards.
    let mut target = ExternalPtr::new(SidecarSlotR6::new(1));
    target.clone_from(&handles[4]);
    churn();
    assert_eq!(target.n, 5);
    assert_eq!(SidecarSlotR6::keys(&target), vec![1, 2, 3, 4, 5, 104]);
    SidecarSlotR6::set_keys(&mut target, vec![0]);
    assert_eq!(SidecarSlotR6::keys(&handles[4]), vec![1, 2, 3, 4, 5, 104]);

    // Swap the Rust values of two pointers: the sidecar values stay put.
    let (first, rest) = handles.split_at_mut(1);
    std::mem::swap(&mut *first[0], &mut *rest[0]);
    churn();
    assert_eq!(first[0].n, 2);
    assert_eq!(SidecarSlotR6::keys(&first[0]), vec![1, 100]);
    assert_eq!(rest[0].n, 1);
    assert_eq!(SidecarSlotR6::keys(&rest[0]), vec![1, 2, 101]);

    // Swap, then `into_inner` with nothing in between, then wrap again: the
    // values of the pointer being consumed travel with the struct it held
    // (the swap bug of #1856).
    let (front, back) = handles.split_at_mut(7);
    std::mem::swap(&mut *front[6], &mut *back[0]);
    let value = ExternalPtr::into_inner(handles.pop().expect("eight handles"));
    assert_eq!(value.n, 7);
    let back = ExternalPtr::new(value);
    churn();
    assert_eq!(back.n, 7);
    assert_eq!(
        SidecarSlotR6::keys(&back),
        vec![1, 2, 3, 4, 5, 6, 7, 8, 107]
    );
    assert_eq!(
        SidecarSlotR6::cache(&back)
            .expect("cache travels")
            .get_index::<i32>(0),
        Some(7)
    );

    // The by-value receiver path: take, rebuild, write back. The slots stay
    // in the pointer across the call; the write-back flushes what the method
    // set with `Sidecar::new`.
    let keep = ExternalPtr::new(SidecarSlotR6::new(3));
    let sexp = keep.as_sexp();
    let mut erased = unsafe { ErasedExternalPtr::from_sexp(sexp) };
    let taken = erased
        .take_for_consuming::<SidecarSlotR6>()
        .expect("the slot holds a SidecarSlotR6");
    assert_eq!(taken.n, 3);
    erased.restore_after_consuming::<SidecarSlotR6>(taken.rebuilt(9));
    churn();
    // `keep`'s cached pointer predates the write-back, so read through a view.
    let view = unsafe { ExternalPtr::<SidecarSlotR6>::wrap_sexp(sexp) }.expect("live pointer");
    assert_eq!(view.n, 12);
    assert_eq!(SidecarSlotR6::keys(&view), vec![1, 2, 3]);
    assert_eq!(SidecarSlotR6::label(&view), "fresh");
    assert_eq!(
        SidecarSlotR6::cache(&view)
            .expect("the write-back flushed the cache")
            .get_index::<i32>(0),
        Some(9)
    );
    drop(view);
    drop(keep);

    // A `Sidecar<SEXP>` read back from the R side of the fence.
    let mut env = rdata_sidecar_env_new(1, 1.0, true, String::from("x"));
    let payload: Vec<i32> = (0..32).collect();
    SidecarEnv::set_raw_slot(&mut env, payload.into_sexp());
    churn();
    let raw: Vec<i32> =
        TryFromSexp::try_from_sexp(SidecarEnv::raw_slot(&env)).expect("an integer vector");
    assert_eq!(raw, (0..32).collect::<Vec<i32>>());
}
// endregion

// region: Module registration
// endregion
