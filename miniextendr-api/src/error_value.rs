//! Tagged condition value transport.
//!
//! Rust-origin failures (panics, `Result::Err`, `Option::None`) and user-raised
//! conditions (`error!()`, `warning!()`, `message!()`, `condition!()`) are
//! converted to a tagged SEXP value instead of raising an R error immediately.
//! The generated R wrapper inspects this tagged value and escalates it to a
//! proper R condition past the Rust boundary, with `rust_*` class layering.
//!
//! # Why tagged SEXP instead of `Rf_error`
//!
//! The naive way to surface a Rust error in R is to call `Rf_error`, which
//! `longjmp`s out of the call frame. That works for C — C has no destructors —
//! but in Rust it skips every drop on the stack: open files, `Mutex` guards,
//! `Box::into_raw` round-trips, the worker-thread continuation token. Anything
//! holding a resource leaks or corrupts.
//!
//! The framework instead catches every Rust panic (and every `RCondition`
//! `panic_any` payload) at the boundary inside
//! [`crate::unwind_protect::with_r_unwind_protect`], encodes it as the
//! 4-element list described below, and *returns* that SEXP normally. The
//! generated R wrapper then re-raises with `stop(structure(..., class =
//! c("rust_*", ...)))`. Destructors run; `tryCatch` sees the right class.
//!
//! There is one accepted leak: on the R-longjmp branch inside
//! `with_r_unwind_protect` (when an R-origin error is propagated through via
//! `R_ContinueUnwind`), the `RErrorMarker` panic payload — about 8 bytes plus
//! `Box` header — escapes Rust drop ordering. This is the price we pay for
//! routing Rust failures through real R conditions instead of
//! `Rf_error`-via-longjmp, and is exactly why lint MXL300 forbids direct
//! `Rf_error` / `Rf_errorcall` in user code: every `Rf_error` skips Rust
//! destructors unconditionally, not just on the (rare) R-longjmp path.
//!
//! # The three error-emission entry points
//!
//! Authors of `#[miniextendr]` functions reach for one of:
//!
//! 1. **`panic!(msg)`** — escape hatch. Produces `kind = "panic"` and R class
//!    `c("rust_error", "simpleError", "error", "condition")`. Use for true
//!    bugs / impossible states; the caller has nothing to catch by class.
//! 2. **`miniextendr_api::error!("msg")`** — typed condition. Produces `kind
//!    = "error"` and the same `rust_error` class layering. The `class =
//!    "my_class"` form prepends a user class, giving R-side
//!    `c("rust_my_class", "rust_error", "simpleError", "error", "condition")`
//!    — exactly what a caller's `tryCatch(my_class = …)` matches on. The
//!    sibling [`crate::warning!`], [`crate::message!`], [`crate::condition!`]
//!    macros cover the non-error condition kinds.
//! 3. **`Result<_, E>` where `E: std::error::Error`**, often via
//!    [`crate::condition::AsRError`] — value-style propagation through Rust
//!    code. Converts at the boundary using `kind = "result_err"`.
//!    `Option::None` follows the same path with `kind = "none_err"`.
//!
//! # `error_in_r` is the default
//!
//! For every `#[miniextendr]` fn / method, the proc macro emits a wrapper that
//! routes through this tagged-SEXP transport — i.e. `error_in_r = true` is the
//! default. The opt-outs are documented on the macro:
//!
//! - `#[miniextendr(no_error_in_r)]` — bypass the tagged-SEXP path entirely.
//!   Useful for trait-ABI vtable shims and benchmarks; Rust panics become
//!   classic `Rf_error` longjmps. Drops the leak above at the cost of skipping
//!   Rust destructors universally.
//! - `#[miniextendr(unwrap_in_r)]` — `Result<T, E>` returns are unwrapped on
//!   the R side rather than encoded as `kind = "result_err"`. Orthogonal to
//!   the transport: still rides this SEXP path, just changes how `Err` is
//!   stringified.
//!
//! Older comments suggesting `Rf_error` is the user-facing path predate PR
//! #344 and are wrong. The wrapper preambles now consistently use this
//! transport.
//!
//! # Condition value structure (`make_rust_condition_value`)
//!
//! The tagged SEXP is a 5-element named list:
//! - `error`: error message (character scalar)
//! - `kind`: condition kind string — one of the constants in [`kind`]
//! - `class`: optional user-supplied custom class (character scalar or `NULL`)
//! - `call`: the R call SEXP (or `NULL` if not available)
//! - `data`: optional named-list condition-data payload (from the macros'
//!   `data = ...` form), or `NULL`. The R helper splices these named fields
//!   into the condition object so handlers can read `e$<name>`.
//! - class attribute: `"rust_condition_value"`
//! - `__rust_condition__` attribute: `TRUE`
//!
//! # PROTECT discipline (read before editing)
//!
//! [`make_rust_condition_value`] allocates SEXPs that must remain live
//! across subsequent allocations (`SET_VECTOR_ELT` / `SETATTRIB` both
//! trigger old-to-new GC barriers): the list itself, the message scalar
//! STRSXP, the kind scalar STRSXP, the optional class scalar STRSXP, the
//! `TRUE` marker LGLSXP, and — when a `data` payload is present — the data
//! VECSXP, its names STRSXP, and each materialised field value. Each is
//! `Rf_protect`ed before the next allocation; `prot` counts them;
//! `Rf_unprotect(prot)` balances at exit on every branch. Field values are
//! materialised one at a time and rooted into the protected data list
//! immediately (same shape as `List::from_pairs`) so an unrooted value SEXP
//! never survives across the next allocation.
//!
//! R-devel runs a more aggressive GC than R-release/oldrel and *will* fire
//! inside the window between two allocations. PR #344 commit `af6b4875`
//! tracked down a `recursive gc invocation` segfault that lit up only on
//! R-devel because the pre-existing 3-element version was lucky-not-safe;
//! adding the class slot crossed the threshold. **If you add another fresh
//! allocation, protect it.** A green R-release CI run is *not* proof of
//! safety here; run `gctorture(TRUE)` on R-devel before merging.

use crate::cached_class::{
    condition_names_sexp, rust_condition_attr_symbol, rust_condition_class_sexp,
};
use crate::sexp_types::CE_UTF8;
use crate::sys::{self};
use crate::{SEXP, SEXPTYPE, SexpExt};

/// Canonical kind strings for tagged condition values.
///
/// These constants are emitted into the `kind` slot of
/// [`make_rust_condition_value`] and consumed by the R-side
/// `.miniextendr_raise_condition` switch (see
/// `registry::write_r_wrappers_to_file`). Reference these constants
/// from codegen and runtime sites instead of bare string literals so a
/// typo cannot silently change which switch arm fires.
///
/// The constants are kept in lockstep with the generated R helper; if a new
/// kind is added, both the emission site and the R helper need to learn it.
pub mod kind {
    /// Default kind for Rust panics that surface to R via the generic panic
    /// path (no `RCondition` payload). Layered as `rust_error`.
    pub const PANIC: &str = "panic";
    /// `Result<_, E>::Err(...)` formatted via `Debug` (raised when the user
    /// returns an `Err` from a `#[miniextendr]` fn/method).
    pub const RESULT_ERR: &str = "result_err";
    /// `Option<T>::None` reached where a value was required (raised by the
    /// `NoneOnErr` / required-Option return paths).
    pub const NONE_ERR: &str = "none_err";
    /// `TryFromSexp` / coerce / strict-mode conversion failed at argument
    /// unmarshalling.
    pub const CONVERSION: &str = "conversion";
    /// User-raised `error!(...)` condition.
    pub const ERROR: &str = "error";
    /// User-raised `warning!(...)` condition.
    pub const WARNING: &str = "warning";
    /// User-raised `message!(...)` condition.
    pub const MESSAGE: &str = "message";
    /// User-raised `condition!(...)` condition.
    pub const CONDITION: &str = "condition";
    /// Fallback kind written by [`super::make_rust_condition_value`] when the
    /// caller's `kind` argument contained an interior NUL and could not be
    /// converted to a `CString`. Should not appear in normal flow; the match
    /// arm in [`crate::condition::RCondition::from_tagged_sexp`] handles it
    /// defensively by degrading to `RCondition::Error`.
    pub const OTHER_RUST_ERROR: &str = "other_rust_error";
}

/// Convert a `&str` to a `CString`, falling back to `fallback` on interior NUL bytes.
///
/// Used internally by [`make_rust_condition_value`] to avoid duplicating the
/// `CString::new(s).unwrap_or_else(…)` pattern across every slot.
fn to_cstring_lossy(s: &str, fallback: &str) -> std::ffi::CString {
    std::ffi::CString::new(s).unwrap_or_else(|_| std::ffi::CString::new(fallback).unwrap())
}

/// Build a tagged condition value with no structured `data` payload.
///
/// Thin wrapper over [`make_rust_condition_value_with_data`] with `data =
/// None`. This is the entry point used by all proc-macro-generated codegen
/// (argument-conversion failures, `Option::None`, `Result::Err`), none of
/// which carries a `data` payload. Only the user-facing `error!()` /
/// `warning!()` / `message!()` / `condition!()` macros (routed through
/// [`crate::unwind_protect`]) attach `data`.
///
/// # Safety
///
/// See [`make_rust_condition_value_with_data`].
#[inline]
pub fn make_rust_condition_value(
    message: &str,
    kind: &str,
    class: Option<&str>,
    call: Option<SEXP>,
) -> SEXP {
    make_rust_condition_value_with_data(message, kind, class, call, None)
}

/// Build a tagged condition-value SEXP for transport across the Rust→R boundary.
///
/// Used for all Rust-origin failures and user-facing conditions. The R-side
/// switch in `condition_check_lines` reads `.val$kind` to select the condition
/// type and `.val$class` to prepend optional user classes before the standard
/// `rust_*` layering.
///
/// # Safety
///
/// Must be called from R's main thread (standard R API constraint).
/// The returned SEXP is unprotected — caller must protect if needed.
///
/// # PROTECT discipline
///
/// Every fresh allocation (msg, kind, optional class, true-marker, and — when
/// present — the `data` VECSXP, its names, and each field value) is protected
/// before the next allocation that might trigger a GC barrier. The `prot`
/// counter is incremented on each `Rf_protect` and balanced by
/// `Rf_unprotect(prot)` at exit on all branches. This pattern was established
/// by PR #344 commit `af6b4875` to fix a `recursive gc invocation` segfault on
/// R-devel.
///
/// # Arguments
///
/// * `message` - Human-readable condition message
/// * `kind` - Condition kind — one of the constants in [`kind`].
/// * `class` - Optional user-supplied class name to prepend to the layered vector
/// * `call` - Optional R call SEXP for error context. When `None`, uses `R_NilValue`.
/// * `data` - Optional named condition-data payload (from the macros' `data =
///   ...` form). When `Some`, each `(name, value)` becomes a named element of a
///   list stored in slot `[4]`; the R helper splices these into the condition
///   object so handlers can read `e$<name>`. When `None`, slot `[4]` is `NULL`.
pub fn make_rust_condition_value_with_data(
    message: &str,
    kind: &str,
    class: Option<&str>,
    call: Option<SEXP>,
    data: Option<crate::condition::ConditionData>,
) -> SEXP {
    unsafe {
        // PROTECT discipline: every fresh allocation that's live across another
        // allocation must be protected. SET_VECTOR_ELT and SETATTRIB can both
        // trigger old-to-new GC barriers; R-devel's GC fires more aggressively
        // here than R-release/oldrel, so unprotected intermediates corrupt the
        // heap on R-devel even when R 4.5/4.4 happen to survive (PR #344 fix).
        let list = sys::Rf_allocVector(SEXPTYPE::VECSXP, 5);
        sys::Rf_protect(list);
        let mut prot = 1;

        // Element 0: error message
        let msg_cstr = to_cstring_lossy(message, "<invalid error message>");
        let msg_charsxp = sys::Rf_mkCharCE(msg_cstr.as_ptr(), CE_UTF8);
        let msg_sexp = SEXP::scalar_string(msg_charsxp);
        sys::Rf_protect(msg_sexp);
        prot += 1;
        list.set_vector_elt(0, msg_sexp);

        // Element 1: kind string
        let kind_cstr = to_cstring_lossy(kind, kind::OTHER_RUST_ERROR);
        let kind_charsxp = sys::Rf_mkCharCE(kind_cstr.as_ptr(), CE_UTF8);
        let kind_sexp = SEXP::scalar_string(kind_charsxp);
        sys::Rf_protect(kind_sexp);
        prot += 1;
        list.set_vector_elt(1, kind_sexp);

        // Element 2: optional custom class (NULL when not provided).
        // Only the Some-branch allocates; nil is constant.
        let class_sexp = if let Some(class_name) = class {
            let class_cstr = to_cstring_lossy(class_name, "rust_condition");
            let class_charsxp = sys::Rf_mkCharCE(class_cstr.as_ptr(), CE_UTF8);
            let s = SEXP::scalar_string(class_charsxp);
            sys::Rf_protect(s);
            prot += 1;
            s
        } else {
            SEXP::nil()
        };
        list.set_vector_elt(2, class_sexp);

        // Element 3: caller-owned SEXP — already protected (or R_NilValue)
        list.set_vector_elt(3, call.unwrap_or(SEXP::nil()));

        // Element 4: optional named-list condition data (NULL when absent).
        //
        // PROTECT discipline: we build a fresh VECSXP `data_list` plus a STRSXP
        // `data_names`, and each field's materialised SEXP. Every one of these
        // is live across subsequent allocations, so each is protected before
        // the next alloc:
        //   - data_list protected before any field materialisation,
        //   - data_names protected before per-field CHARSXP allocations,
        //   - each field value materialised then immediately stored into the
        //     protected data_list (so it is rooted by the list before the next
        //     field allocates — same shape as `List::from_pairs`).
        let data_sexp = if let Some(fields) = data {
            let n: isize = fields
                .len()
                .try_into()
                .expect("condition data length exceeds isize::MAX");
            let data_list = sys::Rf_allocVector(SEXPTYPE::VECSXP, n);
            sys::Rf_protect(data_list);
            prot += 1;
            let data_names = sys::Rf_allocVector(SEXPTYPE::STRSXP, n);
            sys::Rf_protect(data_names);
            prot += 1;
            for (i, (name, value)) in fields.into_iter().enumerate() {
                let idx: isize = i.try_into().expect("index exceeds isize::MAX");
                // Materialise the value and immediately root it in data_list
                // (protected) before the name CHARSXP allocation below.
                let value_sexp = value.into_sexp();
                data_list.set_vector_elt(idx, value_sexp);
                let name_cstr = to_cstring_lossy(&name, "<invalid name>");
                let name_charsxp = sys::Rf_mkCharCE(name_cstr.as_ptr(), CE_UTF8);
                data_names.set_string_elt(idx, name_charsxp);
            }
            data_list.set_names(data_names);
            data_list
        } else {
            SEXP::nil()
        };
        list.set_vector_elt(4, data_sexp);

        // Names / class symbols are cached. The TRUE marker on set_attr is a
        // fresh LGLSXP — protect across the SETATTRIB call.
        list.set_names(condition_names_sexp());
        list.set_class(rust_condition_class_sexp());
        let true_marker = SEXP::scalar_logical(true);
        sys::Rf_protect(true_marker);
        prot += 1;
        list.set_attr(rust_condition_attr_symbol(), true_marker);

        sys::Rf_unprotect(prot);
        list
    }
}
