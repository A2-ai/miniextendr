//! Attribute/type parity for serde return transport (#1521).

use crate::serde::{Serialize, Serializer};
use miniextendr_api::serde::AsSerialize;
use miniextendr_api::{ExternalPtr, Invisible, miniextendr};

/// A serde-only payload with no IntoR implementation.
#[derive(Serialize)]
#[serde(crate = "crate::serde")]
pub struct SerializedRecord {
    value: i32,
    label: String,
}

fn record(value: i32) -> SerializedRecord {
    SerializedRecord {
        value,
        label: format!("value-{value}"),
    }
}

/// Return a serde-only record through the attribute.
/// @param value Integer stored in the record.
#[miniextendr(serialize)]
pub fn serialized_record_attr(value: i32) -> SerializedRecord {
    record(value)
}

/// The equivalent return-type spelling.
#[miniextendr]
pub fn serialized_record_type(value: i32) -> AsSerialize<SerializedRecord> {
    AsSerialize(record(value))
}

/// Serialize after the worker returns its owned Rust payload.
#[cfg(feature = "worker-thread")]
#[miniextendr(worker, serialize)]
pub fn serialized_record_worker(value: i32) -> SerializedRecord {
    record(value)
}

/// Early returns remain ordinary Rust returns.
#[miniextendr(serialize)]
pub fn serialized_record_early(value: i32) -> SerializedRecord {
    if value < 0 {
        return record(-value);
    }
    record(value)
}

/// Serialize a complete Result, including its variant.
/// @param fail Whether to return the Err variant as serde data.
#[miniextendr(serialize)]
pub fn serialized_result_attr(fail: bool) -> Result<SerializedRecord, String> {
    if fail {
        Err("example failure".into())
    } else {
        Ok(record(7))
    }
}

/// The equivalent complete-Result type spelling.
#[miniextendr]
pub fn serialized_result_type(fail: bool) -> AsSerialize<Result<SerializedRecord, String>> {
    AsSerialize(serialized_result_attr(fail))
}

/// Keep normal boundary errors by wrapping only the successful payload.
#[miniextendr]
pub fn serialized_result_payload(fail: bool) -> Result<AsSerialize<SerializedRecord>, String> {
    serialized_result_attr(fail).map(AsSerialize)
}

/// Serialize a complete Option: None is serde NULL.
/// @param present Whether to include a record.
#[miniextendr(serialize)]
pub fn serialized_option_attr(present: bool) -> Option<SerializedRecord> {
    present.then(|| record(8))
}

/// The equivalent complete-Option type spelling.
#[miniextendr]
pub fn serialized_option_type(present: bool) -> AsSerialize<Option<SerializedRecord>> {
    AsSerialize(serialized_option_attr(present))
}

/// Combine serialization with an outer visibility marker.
#[miniextendr(serialize)]
pub fn serialized_invisible_attr() -> Invisible<SerializedRecord> {
    Invisible(record(9))
}

/// The equivalent nested marker spelling.
#[miniextendr]
pub fn serialized_invisible_type() -> Invisible<AsSerialize<SerializedRecord>> {
    Invisible(AsSerialize(record(9)))
}

/// A unit return is serde NULL, with the same visibility as AsSerialize<()>.
#[miniextendr(serialize)]
pub fn serialized_unit_attr() {}

/// The equivalent unit-return type spelling.
#[miniextendr]
pub fn serialized_unit_type() -> AsSerialize<()> {
    AsSerialize(())
}

pub struct SerializationFailure;
impl Serialize for SerializationFailure {
    fn serialize<S: Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
        Err(crate::serde::ser::Error::custom(
            "intentional serialization failure",
        ))
    }
}

/// Serializer failures travel through the normal Rust error boundary.
#[miniextendr(serialize)]
pub fn serialized_failure_attr() -> SerializationFailure {
    SerializationFailure
}

/// A serializable class also used to test bypassing class wrapping.
#[derive(ExternalPtr, Serialize)]
#[serde(crate = "crate::serde")]
pub struct SerializeHost {
    value: i32,
}

#[miniextendr(env)]
impl SerializeHost {
    pub fn new(value: i32) -> Self {
        Self { value }
    }

    #[miniextendr(serialize)]
    pub fn snapshot(&self) -> SerializedRecord {
        record(self.value)
    }

    pub fn snapshot_type(&self) -> AsSerialize<SerializedRecord> {
        AsSerialize(record(self.value))
    }

    #[miniextendr(env(serialize))]
    pub fn self_data(&self) -> Self {
        Self { value: self.value }
    }

    #[miniextendr(serialize)]
    pub fn consume(self) -> Self {
        self
    }
}

/// Trait-return serde transport; the vector remains compatible with its View.
#[miniextendr]
pub trait SerializeValues {
    fn values(&self) -> Vec<i32>;
}

#[miniextendr(env)]
impl SerializeValues for SerializeHost {
    #[miniextendr(serialize)]
    fn values(&self) -> Vec<i32> {
        vec![self.value, self.value + 1]
    }
}

/// Exercise the concrete trait vtable and its Rust View conversion.
/// @param obj A SerializeHost instance.
#[miniextendr(no_worker)]
pub fn serialized_trait_view(obj: miniextendr_api::SEXP) -> Vec<i32> {
    unsafe {
        let sexp = miniextendr_api::externalptr::resolve_receiver::<SerializeHost>(obj);
        SerializeValuesView::from_sexp(sexp).values()
    }
}
