//! Conditions with a given call (#1852): `call = <expr>` on the condition
//! macros and `arg_error!`, `RError::with_call()` and
//! `#[condition(call = <expr>)]`, so that a condition reads like the code the
//! user wrote (`x$table <- value` from a replacement method).
//!
//! Each fixture takes the call to report from R (an `RCallValue` argument),
//! which also covers its conversion. Tests live in
//! `rpkg/tests/testthat/test-given-call-conditions.R`.

use miniextendr_api::condition::{RCondition, RError};
use miniextendr_api::{
    RCallValue, arg_error, defer_message, defer_warning, miniextendr, rust_condition, rust_error,
    warning,
};

// region: Typed payload

/// Table errors: `Replaced` reports the call it carries, `Missing` keeps the
/// wrapper's.
#[derive(Debug, RCondition)]
#[condition(class = "pkg_table_error")]
pub enum TableError {
    #[condition(call = site.clone(), message = "the new table has no `{column}` column")]
    Replaced {
        column: String,
        #[condition(skip)]
        site: RCallValue,
    },
    #[condition(message = "no table")]
    Missing,
}

/// A warning whose call comes from the whole type's `call`.
#[derive(Debug, RCondition)]
#[condition(class = "pkg_table_warning", call = site.clone(), message = "{rows} rows dropped")]
pub struct DroppedRows {
    rows: i32,
    #[condition(skip)]
    site: RCallValue,
}

// endregion

// region: Immediate conditions

/// An error that reports `call`.
#[miniextendr(noexport)]
pub fn given_call_error(call: RCallValue) {
    rust_error!(
        call = call,
        class = "pkg_bad_table",
        data = { column = "id" },
        "the table has no `id` column"
    );
}

/// A warning that reports `call`.
#[miniextendr(noexport)]
pub fn given_call_warning(call: RCallValue) {
    warning!(
        call = call,
        class = "pkg_table_warning",
        "the table is empty"
    );
}

/// A plain condition that reports `call`.
#[miniextendr(noexport)]
pub fn given_call_condition(call: RCallValue) {
    rust_condition!(call = call, class = "pkg_progress", "halfway");
}

/// An argument error that reports `call`.
#[miniextendr(noexport)]
pub fn given_call_arg_error(call: RCallValue, value: i32) {
    arg_error!(
        call = call,
        param = "value",
        "'value' must be positive, not {value}"
    );
}

/// An error whose call R can't build (a name over R's 10000 bytes): it keeps
/// the wrapper's call.
#[miniextendr(noexport)]
pub fn given_call_unbuildable() {
    rust_error!(
        call = RCallValue::new("x".repeat(10_001)),
        class = "pkg_bad_table",
        "the call is too long"
    );
}

// endregion

// region: Deferred conditions and results

/// Four deferred conditions that report `call` (a macro warning, an
/// `RError`, a derived struct and a message, which has no call anyway) and
/// one that keeps the wrapper's. Returns 1.
#[miniextendr(noexport)]
pub fn given_call_deferred(call: RCallValue) -> i32 {
    defer_warning!(call = call.clone(), class = "pkg_table_warning", "macro");
    defer_warning(
        RError::new("typed")
            .class("pkg_table_warning")
            .with_call(call.clone()),
    );
    defer_warning(DroppedRows {
        rows: 3,
        site: call.clone(),
    });
    defer_message!(call = call, class = "pkg_note", "noted");
    defer_warning!(class = "pkg_table_warning", "sibling keeps its call");
    1
}

/// A warning deferred from another thread with `call`: `RCallValue` is
/// `Send`, and the main thread builds the call. Returns 1.
#[miniextendr(noexport)]
pub fn given_call_off_thread(call: RCallValue) -> i32 {
    std::thread::scope(|scope| {
        scope.spawn(move || {
            defer_warning!(call = call, class = "pkg_thread_warning", "from a thread");
        });
    });
    1
}

/// A `Result` error that reports `call`.
#[miniextendr(noexport)]
pub fn given_call_result(call: RCallValue) -> Result<i32, RError> {
    Err(RError::new("the table is locked")
        .class("pkg_locked")
        .with_call(call))
}

/// A derived error: `Replaced` reports `call`, `Missing` the wrapper's call.
/// @param replaced Which variant to return.
#[miniextendr(noexport)]
pub fn given_call_derived(call: RCallValue, replaced: bool) -> Result<(), TableError> {
    Err(if replaced {
        TableError::Replaced {
            column: "id".into(),
            site: call,
        }
    } else {
        TableError::Missing
    })
}

// endregion

// region: The call type itself

/// `call`, read into an `RCallValue` and built again.
#[miniextendr(noexport)]
pub fn given_call_roundtrip(call: RCallValue) -> RCallValue {
    call
}

// endregion
