//! Deferred conditions: signal an R warning, message or condition **and**
//! still return a value.
//!
//! [`crate::warning!`], [`crate::message!`] and [`crate::condition!`] unwind
//! the Rust function: R sees the signal, the call returns `invisible(NULL)`.
//! That is the right shape for "stop here, but softly", not for "here is the
//! result, and by the way …". The `defer_*` family queues the condition
//! instead; the generated C wrapper signals everything queued during the call
//! once the Rust body has returned, in queue order, and then hands the value
//! to R.
//!
//! ```ignore
//! use miniextendr_api::{defer_warning, miniextendr};
//! use miniextendr_api::condition::RConditionError;
//!
//! #[derive(Debug, RConditionError)]
//! #[condition(class = "pkg_warning")]
//! enum PkgWarning {
//!     #[condition(message = "dropped {dropped} of {total} rows")]
//!     Truncated { dropped: i32, total: i32 },
//! }
//!
//! #[miniextendr]
//! fn trim(n: i32) -> i32 {
//!     defer_warning(PkgWarning::Truncated { dropped: 2, total: n });
//!     n - 2
//! }
//! ```
//!
//! ```r
//! trim(5L)
//! # [1] 3
//! # Warning message:
//! # In trim(5L) : dropped 2 of 5 rows
//! w <- tryCatch(trim(5L), pkg_warning_truncated = function(w) w)
//! w$dropped               # 2
//! suppressWarnings(trim(5L))   # 3
//! ```
//!
//! # What R sees
//!
//! Exactly what `warning()` / `message()` / `signalCondition()` called from
//! an R function would produce, because that is what runs: the same R helper
//! the generated wrappers use for the panicking macros re-raises each queued
//! condition. Class layering (`c(<classes…>, "rust_warning", "simpleWarning",
//! "warning", "condition")`), `data` fields as `w$<name>` and
//! `conditionCall()` (the wrapper's call) are identical to the immediate
//! forms. `tryCatch(warning = )` exits the call before the value is returned;
//! `withCallingHandlers` + `invokeRestart("muffleWarning")` keeps it;
//! `suppressWarnings()` / `suppressMessages()` work as usual.
//!
//! # Ordering
//!
//! Queued conditions are signalled after the Rust body finished and before R
//! sees its outcome, so a deferred warning followed by `rust_error!` (or an
//! `Err` return) is signalled first and the error second, and two deferred
//! conditions keep their queue order. Nested calls (R code evaluated from
//! Rust that calls back into another `#[miniextendr]` function) flush only
//! what they queued themselves.
//!
//! # Warning placement
//!
//! Conditions queued before a failure still signal. If a body must not warn on
//! failure, collect warning payloads locally and queue them only after its last
//! fallible step (including any `?`). Moving a warning before a later `?` changes
//! that behavior: returning `Err` does not cancel the queued warning.
//!
//! ```no_run
//! use miniextendr_api::{defer_warning, miniextendr};
//!
//! #[miniextendr]
//! pub fn parse_trimmed(text: String) -> Result<i32, std::num::ParseIntError> {
//!     let trimmed = text.trim();
//!     let value = trimmed.parse::<i32>()?;
//!     if trimmed != text {
//!         defer_warning!(class = "pkg_trimmed", "removed surrounding whitespace");
//!     }
//!     Ok(value)
//! }
//! ```
//!
//! Here, parsing `" bad "` returns an error without a warning; parsing `" 42 "`
//! queues the whitespace warning and returns `42`.
//!
//! # Payloads
//!
//! [`defer_warning()`], [`defer_message()`] and [`defer_condition()`] take any
//! [`RConditionError`] value: a `#[derive(RConditionError)]` enum or struct
//! (class from the type and variant names, message from a `format!` string or
//! `Display`, fields as `data`), or a hand-built [`crate::condition::RError`].
//! The [`defer_warning!`](crate::defer_warning!) /
//! [`defer_message!`](crate::defer_message!) /
//! [`defer_condition!`](crate::defer_condition!) macros take the
//! [`crate::warning!`] grammar (`class = …`, `data = …`, format message) for
//! one-off conditions.
//!
//! # Threads and boundaries
//!
//! The queue is a process-wide `Mutex`, so `#[miniextendr(worker)]` bodies
//! push from the worker thread; the signalling always happens on R's main
//! thread after the worker result is back. Every generated `.Call` entry
//! point and every trait-ABI vtable shim flushes what was queued inside it.
//! Code that runs under no such boundary (ALTREP callbacks, finalizers) has
//! its conditions signalled by the next boundary that completes in the same
//! package; use the panicking macros or `error!` there (flushing at those
//! guard sites is issue #1518).

use std::sync::{Mutex, MutexGuard, OnceLock, PoisonError};

use crate::condition::{RCondition, RConditionError};
use crate::{SEXP, SexpExt};

// region: Queue

/// Conditions queued during the current `.Call` (and any enclosing ones).
///
/// A plain static rather than a thread-local so worker-thread pushes land in
/// the same queue the main thread drains. R is single-threaded and the main
/// thread blocks while a worker body runs, so the queue is never contended.
static PENDING: Mutex<Vec<RCondition>> = Mutex::new(Vec::new());

fn pending() -> MutexGuard<'static, Vec<RCondition>> {
    PENDING.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Queue a warning to be signalled when the current `#[miniextendr]` call
/// returns; the call still returns its value.
///
/// `payload` supplies the message, the user classes (prepended to
/// `rust_warning`) and the `data` fields (`w$<name>`). See the
/// [module docs](self) for the R-side behaviour and [`defer_warning!`](crate::defer_warning!)
/// for the macro form.
///
/// A later `Err` does not cancel this warning. To avoid warnings on failure,
/// queue after the last fallible step; see [warning placement](self#warning-placement).
pub fn defer_warning(payload: impl RConditionError) {
    defer(RCondition::Warning {
        message: payload.message(),
        class: payload.class(),
        data: payload.data(),
    });
}

/// Queue a message (`message()`, muffled by `suppressMessages()`) to be
/// emitted when the current `#[miniextendr]` call returns.
///
/// Unlike [`crate::message!`], the payload's classes are honoured: they are
/// layered in front of `rust_message` so handlers can dispatch on them.
pub fn defer_message(payload: impl RConditionError) {
    defer(RCondition::Message {
        message: payload.message(),
        class: payload.class(),
        data: payload.data(),
    });
}

/// Queue a plain condition (`signalCondition()`: silent without a handler)
/// to be signalled when the current `#[miniextendr]` call returns.
pub fn defer_condition(payload: impl RConditionError) {
    defer(RCondition::Condition {
        message: payload.message(),
        class: payload.class(),
        data: payload.data(),
    });
}

/// Queue an already-built condition payload. Used by the `defer_*!` macros.
#[doc(hidden)]
pub fn defer(condition: RCondition) {
    pending().push(condition);
}

/// The queue position at the start of a call. Pair with [`finish`].
#[doc(hidden)]
#[inline]
pub fn mark() -> usize {
    pending().len()
}

/// Remove and return everything queued since `mark`, oldest first. Entries
/// below `mark` belong to enclosing calls and stay.
#[doc(hidden)]
pub fn take_pending(mark: usize) -> Vec<RCondition> {
    let mut queue = pending();
    if queue.len() <= mark {
        return Vec::new();
    }
    queue.split_off(mark)
}

// endregion

// region: Flush

/// Signal everything queued since `mark`, then return `value`.
///
/// Called by the generated C wrappers with the wrapper's outcome (the value
/// SEXP, or a tagged condition value when the body failed) and by
/// [`crate::unwind_protect::with_r_unwind_protect_shim`]. The queued
/// conditions are materialised as tagged condition values and handed to the
/// R helper (`stop()` / `warning()` / `message()` / `signalCondition()`), so
/// their R-side shape is identical to the panicking macros'. The signalling
/// runs under its own `R_UnwindProtect`: an exiting handler unwinds through
/// it like any R error, and a Rust panic inside (a framework bug) comes back
/// as a tagged error value, which replaces `value`.
///
/// `call` is the wrapper's call object (`conditionCall()` of every signalled
/// condition); `None` gives `NULL`.
///
/// # Safety
///
/// Must run on R's main thread. `value` need not be protected: it is rooted
/// for the duration of the signalling.
#[doc(hidden)]
pub unsafe fn finish(mark: usize, value: SEXP, call: Option<SEXP>) -> SEXP {
    let queued = take_pending(mark);
    if queued.is_empty() {
        return value;
    }
    let outcome = crate::unwind_protect::with_r_unwind_protect(
        || {
            // SAFETY: main thread (caller contract), inside R_UnwindProtect.
            unsafe {
                let _value_root = crate::OwnedProtect::new(value);
                signal_now(queued, call);
            }
            SEXP::nil()
        },
        call,
    );
    if outcome.is_nil() { value } else { outcome }
}

/// Materialise each queued condition and hand it to the R helper.
///
/// # Safety
///
/// Main thread, inside `R_UnwindProtect` (handlers may longjmp).
unsafe fn signal_now(queued: Vec<RCondition>, call: Option<SEXP>) {
    let helper = raise_condition_helper();
    let call_sexp = call.unwrap_or(SEXP::nil());
    for condition in queued {
        let (kind, message, class, data) = condition.into_parts();
        // SAFETY: main thread inside R_UnwindProtect (caller contract). The
        // tagged value and the call object are rooted across `Rf_eval`.
        unsafe {
            let tagged =
                crate::OwnedProtect::new(crate::error_value::make_rust_condition_value_with_data(
                    &message, kind, &class, call, data,
                ));
            let expr =
                crate::OwnedProtect::new(crate::sys::Rf_lang3(helper, tagged.get(), call_sexp));
            crate::sys::Rf_eval(expr.get(), crate::sys::R_BaseEnv);
        }
    }
}

/// The `.miniextendr_raise_condition` closure, evaluated once from the shared
/// source ([`crate::registry::RAISE_CONDITION_HELPER_FN`]) in the base
/// namespace and preserved for the life of the process.
///
/// Evaluating the source here, rather than looking the helper up in a package
/// namespace, keeps this path independent of which package (or embedded
/// engine) is running: the helper only needs base and `utils::modifyList`.
fn raise_condition_helper() -> SEXP {
    static HELPER: OnceLock<SEXP> = OnceLock::new();
    *HELPER.get_or_init(|| {
        // SAFETY: only reached from `signal_now`, on the main thread.
        unsafe {
            let closure = crate::expression::r_eval_str(
                crate::registry::RAISE_CONDITION_HELPER_FN,
                crate::sys::R_BaseNamespace,
            )
            .unwrap_or_else(|err| {
                panic!("miniextendr: could not define the deferred-condition helper: {err}")
            });
            crate::sys::R_PreserveObject(closure);
            closure
        }
    })
}

// endregion

// region: Macros

/// Queue an R warning with `rust_warning` class layering and return normally;
/// the surrounding `#[miniextendr]` call signals it after its value is
/// computed.
///
/// Same grammar as [`crate::warning!`]: optional `class = …` (one class or a
/// vector, most specific first), optional `data = …` (a pair, a bracketed
/// list of pairs, or `{ name = value }` sugar), then the `format!` message.
/// For a typed payload use [`crate::defer_warning()`] with a
/// `#[derive(RConditionError)]` type.
///
/// A later `Err` does not cancel this warning. To avoid warnings on failure,
/// queue after the last fallible step; see [warning placement](crate::deferred_condition#warning-placement).
///
/// ```ignore
/// use miniextendr_api::defer_warning;
///
/// #[miniextendr]
/// fn parse_all(xs: Vec<String>) -> Vec<i32> {
///     let (ok, bad): (Vec<_>, Vec<_>) = xs.iter().map(|s| s.parse::<i32>()).partition(Result::is_ok);
///     if !bad.is_empty() {
///         defer_warning!(class = "pkg_unparsed", data = ("n", bad.len() as i32), "{} values could not be parsed", bad.len());
///     }
///     ok.into_iter().map(Result::unwrap).collect()
/// }
/// ```
#[macro_export]
macro_rules! defer_warning {
    ($($t:tt)*) => {{
        let (__mx_class, __mx_data, __mx_message) = $crate::__mx_condition_parts!($($t)*);
        $crate::deferred_condition::defer($crate::condition::RCondition::Warning {
            message: __mx_message,
            class: __mx_class,
            data: __mx_data,
        });
    }};
}

/// Queue an R message (`rust_message` layering, muffled by
/// `suppressMessages()`) and return normally; emitted when the surrounding
/// `#[miniextendr]` call returns.
///
/// Same grammar as [`crate::defer_warning!`]. Unlike [`crate::message!`] a
/// `class = …` part is accepted and layered in front of `rust_message`.
#[macro_export]
macro_rules! defer_message {
    ($($t:tt)*) => {{
        let (__mx_class, __mx_data, __mx_message) = $crate::__mx_condition_parts!($($t)*);
        $crate::deferred_condition::defer($crate::condition::RCondition::Message {
            message: __mx_message,
            class: __mx_class,
            data: __mx_data,
        });
    }};
}

/// Queue a plain R condition (`rust_condition` layering, silent without a
/// handler) and return normally; signalled when the surrounding
/// `#[miniextendr]` call returns.
///
/// Same grammar as [`crate::defer_warning!`]. Useful for progress or audit
/// events a caller may opt into with `withCallingHandlers()`.
#[macro_export]
macro_rules! defer_condition {
    ($($t:tt)*) => {{
        let (__mx_class, __mx_data, __mx_message) = $crate::__mx_condition_parts!($($t)*);
        $crate::deferred_condition::defer($crate::condition::RCondition::Condition {
            message: __mx_message,
            class: __mx_class,
            data: __mx_data,
        });
    }};
}

// endregion

// region: Tests — queue semantics (no R runtime needed)

#[cfg(test)]
mod tests {
    use super::*;
    use crate::condition::{ConditionData, RError};
    use crate::error_value::kind;

    /// `(name, i32)` view of a data payload (`RValue` has no `PartialEq`).
    fn ints(data: &Option<ConditionData>) -> Vec<(&str, Option<i32>)> {
        data.as_ref()
            .map(|fields| {
                fields
                    .iter()
                    .map(|(name, value)| (name.as_str(), value.as_i32()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The queue is process-wide; serialise the tests that touch it.
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn with_empty_queue<T>(f: impl FnOnce() -> T) -> T {
        let _lock = TEST_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
        take_pending(0);
        f()
    }

    #[test]
    fn typed_payloads_keep_kind_class_and_data() {
        with_empty_queue(|| {
            let m = mark();
            defer_warning(RError::new("w").class(["a", "b"]).data("n", 1));
            defer_message(RError::new("m").class("c"));
            defer_condition(RError::new("c"));
            let parts: Vec<_> = take_pending(m)
                .into_iter()
                .map(RCondition::into_parts)
                .collect();
            assert_eq!(parts.len(), 3);
            assert_eq!(parts[0].0, kind::WARNING);
            assert_eq!(parts[0].1, "w");
            assert_eq!(parts[0].2, vec!["a", "b"]);
            assert_eq!(ints(&parts[0].3), vec![("n", Some(1))]);
            assert_eq!(parts[1].0, kind::MESSAGE);
            assert_eq!(parts[1].2, vec!["c"]);
            assert!(parts[1].3.is_none());
            assert_eq!(parts[2].0, kind::CONDITION);
            assert!(parts[2].2.is_empty());
            assert!(take_pending(m).is_empty());
        });
    }

    #[test]
    fn nested_marks_take_only_their_own_entries() {
        with_empty_queue(|| {
            let outer = mark();
            crate::defer_warning!("outer");
            let inner = mark();
            crate::defer_warning!(class = "t", data = ("n", 2), "inner {}", 2);
            let inner_taken = take_pending(inner);
            assert_eq!(inner_taken.len(), 1);
            let (kind, message, class, data) = inner_taken.into_iter().next().unwrap().into_parts();
            assert_eq!((kind, message.as_str()), (kind::WARNING, "inner 2"));
            assert_eq!(class, vec!["t"]);
            assert_eq!(ints(&data), vec![("n", Some(2))]);
            let outer_taken = take_pending(outer);
            assert_eq!(outer_taken.len(), 1);
            assert_eq!(
                outer_taken.into_iter().next().unwrap().into_parts().1,
                "outer"
            );
        });
    }

    #[test]
    fn macro_forms_cover_message_and_condition() {
        with_empty_queue(|| {
            let m = mark();
            crate::defer_message!(data = { step = 1 }, "step {}", 1);
            crate::defer_condition!(class = ["member", "family"], "tick");
            let parts: Vec<_> = take_pending(m)
                .into_iter()
                .map(RCondition::into_parts)
                .collect();
            assert_eq!(parts[0].0, kind::MESSAGE);
            assert_eq!(parts[0].1, "step 1");
            assert_eq!(parts[1].0, kind::CONDITION);
            assert_eq!(parts[1].2, vec!["member", "family"]);
        });
    }

    #[test]
    fn take_pending_past_the_end_is_empty() {
        with_empty_queue(|| {
            assert!(take_pending(usize::MAX).is_empty());
            assert!(take_pending(mark() + 1).is_empty());
        });
    }
}

// endregion
