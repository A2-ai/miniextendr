//! Cross class-system condition fixtures.
//!
//! Each class system (R6, S3, S4, S7, Env) gets a small struct whose methods
//! raise every `RCondition` variant — bare and with a custom class. The matrix
//! exercises the per-class-system R wrapper shape (active bindings, setMethod,
//! `self`-default arg, S7 `tryCatch(x@.ptr)` lambda, ...) which all dispatch
//! through `.miniextendr_raise_condition` after PR #382.
//!
//! Tests live in `rpkg/tests/testthat/test-conditions-comprehensive.R`.

use miniextendr_api::miniextendr;

type RCondition = miniextendr_api::condition::RCondition;

// region: shared helpers

/// Variable-class `error!` — the `error!` macro requires a literal class string.
fn raise_error_with_class(class: &str, msg: &str) -> ! {
    std::panic::panic_any(RCondition::Error {
        message: msg.to_string(),
        class: Some(class.to_string()),
    });
}

fn raise_warning_with_class(class: &str, msg: &str) -> ! {
    std::panic::panic_any(RCondition::Warning {
        message: msg.to_string(),
        class: Some(class.to_string()),
    });
}

fn raise_condition_with_class(class: &str, msg: &str) -> ! {
    std::panic::panic_any(RCondition::Condition {
        message: msg.to_string(),
        class: Some(class.to_string()),
    });
}

// endregion

// region: R6 — active bindings + $-dispatch

/// R6 fixture for raising every condition kind from instance methods.
#[derive(miniextendr_api::ExternalPtr)]
pub struct R6Raiser {
    id: i32,
}

#[miniextendr(r6)]
impl R6Raiser {
    pub fn new(id: i32) -> Self {
        R6Raiser { id }
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn raise_error(&self, msg: &str) {
        miniextendr_api::error!("{msg}");
    }

    pub fn raise_error_classed(&self, class: &str, msg: &str) {
        raise_error_with_class(class, msg);
    }

    pub fn raise_warning(&self, msg: &str) {
        miniextendr_api::warning!("{msg}");
    }

    pub fn raise_warning_classed(&self, class: &str, msg: &str) {
        raise_warning_with_class(class, msg);
    }

    pub fn raise_message(&self, msg: &str) {
        miniextendr_api::message!("{msg}");
    }

    pub fn raise_condition(&self, msg: &str) {
        miniextendr_api::condition!("{msg}");
    }

    pub fn raise_condition_classed(&self, class: &str, msg: &str) {
        raise_condition_with_class(class, msg);
    }
}

// endregion

// region: S3 — setMethod-style

#[derive(miniextendr_api::ExternalPtr)]
pub struct S3Raiser {
    id: i32,
}

#[miniextendr(s3)]
impl S3Raiser {
    /// @param id Numeric identifier for this raiser instance.
    pub fn new(id: i32) -> Self {
        S3Raiser { id }
    }

    pub fn s3_id(&self) -> i32 {
        self.id
    }

    pub fn s3_raise_error(&self, msg: &str) {
        miniextendr_api::error!("{msg}");
    }

    pub fn s3_raise_error_classed(&self, class: &str, msg: &str) {
        raise_error_with_class(class, msg);
    }

    pub fn s3_raise_warning(&self, msg: &str) {
        miniextendr_api::warning!("{msg}");
    }

    pub fn s3_raise_warning_classed(&self, class: &str, msg: &str) {
        raise_warning_with_class(class, msg);
    }

    pub fn s3_raise_message(&self, msg: &str) {
        miniextendr_api::message!("{msg}");
    }

    pub fn s3_raise_condition(&self, msg: &str) {
        miniextendr_api::condition!("{msg}");
    }

    pub fn s3_raise_condition_classed(&self, class: &str, msg: &str) {
        raise_condition_with_class(class, msg);
    }
}

// endregion

// region: S4 — setMethod with formal class

#[derive(miniextendr_api::ExternalPtr)]
pub struct S4Raiser {
    id: i32,
}

/// @aliases s4_raise_error,S4Raiser-method s4_raise_error_classed,S4Raiser-method s4_raise_warning,S4Raiser-method s4_raise_warning_classed,S4Raiser-method s4_raise_message,S4Raiser-method s4_raise_condition,S4Raiser-method s4_raise_condition_classed,S4Raiser-method s4_id,S4Raiser-method
//
// NB: s4 codegen auto-prepends `s4_` to each method name when generating the R
// generic; keep the Rust method names unprefixed (`raise_error`, not `s4_raise_error`)
// or you end up with `s4_s4_raise_error`.
#[miniextendr(s4, internal)]
impl S4Raiser {
    pub fn new(id: i32) -> Self {
        S4Raiser { id }
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn raise_error(&self, msg: &str) {
        miniextendr_api::error!("{msg}");
    }

    pub fn raise_error_classed(&self, class: &str, msg: &str) {
        raise_error_with_class(class, msg);
    }

    pub fn raise_warning(&self, msg: &str) {
        miniextendr_api::warning!("{msg}");
    }

    pub fn raise_warning_classed(&self, class: &str, msg: &str) {
        raise_warning_with_class(class, msg);
    }

    pub fn raise_message(&self, msg: &str) {
        miniextendr_api::message!("{msg}");
    }

    pub fn raise_condition(&self, msg: &str) {
        miniextendr_api::condition!("{msg}");
    }

    pub fn raise_condition_classed(&self, class: &str, msg: &str) {
        raise_condition_with_class(class, msg);
    }
}

// endregion

// region: S7 — new_generic dispatch + tryCatch(x@.ptr) lambda

#[derive(miniextendr_api::ExternalPtr)]
pub struct S7Raiser {
    id: i32,
}

#[miniextendr(s7, internal)]
impl S7Raiser {
    pub fn new(id: i32) -> Self {
        S7Raiser { id }
    }

    pub fn s7_id(&self) -> i32 {
        self.id
    }

    pub fn s7_raise_error(&self, msg: &str) {
        miniextendr_api::error!("{msg}");
    }

    pub fn s7_raise_error_classed(&self, class: &str, msg: &str) {
        raise_error_with_class(class, msg);
    }

    pub fn s7_raise_warning(&self, msg: &str) {
        miniextendr_api::warning!("{msg}");
    }

    pub fn s7_raise_warning_classed(&self, class: &str, msg: &str) {
        raise_warning_with_class(class, msg);
    }

    pub fn s7_raise_message(&self, msg: &str) {
        miniextendr_api::message!("{msg}");
    }

    pub fn s7_raise_condition(&self, msg: &str) {
        miniextendr_api::condition!("{msg}");
    }

    pub fn s7_raise_condition_classed(&self, class: &str, msg: &str) {
        raise_condition_with_class(class, msg);
    }
}

// endregion

// region: Env — `self`-default-arg standalone wrapper

#[derive(miniextendr_api::ExternalPtr)]
pub struct EnvRaiser {
    id: i32,
}

#[miniextendr(env)]
impl EnvRaiser {
    pub fn new(id: i32) -> Self {
        EnvRaiser { id }
    }

    pub fn env_id(&self) -> i32 {
        self.id
    }

    pub fn env_raise_error(&self, msg: &str) {
        miniextendr_api::error!("{msg}");
    }

    pub fn env_raise_error_classed(&self, class: &str, msg: &str) {
        raise_error_with_class(class, msg);
    }

    pub fn env_raise_warning(&self, msg: &str) {
        miniextendr_api::warning!("{msg}");
    }

    pub fn env_raise_warning_classed(&self, class: &str, msg: &str) {
        raise_warning_with_class(class, msg);
    }

    pub fn env_raise_message(&self, msg: &str) {
        miniextendr_api::message!("{msg}");
    }

    pub fn env_raise_condition(&self, msg: &str) {
        miniextendr_api::condition!("{msg}");
    }

    pub fn env_raise_condition_classed(&self, class: &str, msg: &str) {
        raise_condition_with_class(class, msg);
    }
}

// endregion

// region: edge cases — non-RCondition panic, long/unicode messages

/// Panic with a non-`String`/non-`RCondition` payload — falls through to the
/// generic panic→string fallback.
#[miniextendr]
pub fn condition_panic_with_int_payload() {
    std::panic::panic_any(42_i32);
}

/// Long message exercises the `CString` + STRSXP encoding path.
#[miniextendr]
pub fn condition_error_long_message(n_chunks: i32) {
    let chunk = "abcdefghij_";
    let n = n_chunks.max(0) as usize;
    let msg: String = chunk.repeat(n);
    miniextendr_api::error!("{msg}");
}

/// Unicode + multibyte + embedded newline — tests UTF-8 round-trip.
#[miniextendr]
pub fn condition_error_unicode() {
    miniextendr_api::error!("rust ⚙️ panic\n日本語\nемоджи 🦀");
}

/// Empty error message — degenerate but should still produce a valid condition.
#[miniextendr]
pub fn condition_error_empty() {
    miniextendr_api::error!("");
}

// endregion
