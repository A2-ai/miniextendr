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
//! use miniextendr_api::RCondition;
//!
//! #[derive(Debug, RCondition)]
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
//! `conditionCall()` (the wrapper's call, or `NULL` for a condition queued
//! with `call = none`) are identical to the immediate forms. `tryCatch(warning = )` exits the call before the value is returned;
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
//! # Capturing a block's conditions
//!
//! [`capture`] runs a closure and returns what it queued instead of
//! signalling it, for code whose diagnostics the caller already has (a
//! replay of the steps that built a saved object). Conditions queued outside
//! the closure are untouched.
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
//! # miniextendr_api::miniextendr_init!(pkg);
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
//! # fn main() {}
//! ```
//!
//! Here, parsing `" bad "` returns an error without a warning; parsing `" 42 "`
//! queues the whitespace warning and returns `42`.
//!
//! # Payloads
//!
//! [`defer_warning()`], [`defer_message()`] and [`defer_condition()`] take any
//! [`RConditionData`] value: a `#[derive(RCondition)]` enum or struct
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
//! The queue lives on R's main thread, where every generated `.Call` entry
//! point and every trait-ABI vtable shim flushes what was queued inside it.
//! A condition queued on another thread (a Rayon task, a `std::thread`) goes
//! to the main thread over a bounded channel and joins the queue the next
//! time the main thread reads it: when a call signals its conditions, or
//! discards them on an R error. Rayon's `par_iter`, `join` and `scope` block
//! the calling thread until every task has finished, so every condition a
//! task queued is waiting when the call that ran the parallel section
//! returns, and that call signals it (or discards it, if it fails with an R
//! error). Only a thread nobody joins (a `std::thread::spawn` left running, a
//! `rayon::spawn`) can deliver its conditions to a later, unrelated call.
//!
//! The channel holds 1024 conditions. One queued while it is full is
//! dropped, and the main thread queues a warning that says how many were,
//! after the ones it received. [`capture`] needs the queue itself, so it
//! panics off the main thread; capture around the parallel section instead.
//!
//! ALTREP `RUnwind` / `RustUnwind` callbacks and connection I/O callbacks also
//! signal locally, with `conditionCall() = NULL`; raising guards use their
//! explicit `call` argument. An R-origin error discards the abandoned queue.
//!
//! External-pointer finalizers and connection close/destroy/Drop callbacks
//! suppress deferred conditions: R finalization removes the caller's handlers.
//! No R signalling or delayed delivery is attempted from destructors guarded
//! by `drop_catching_panic`. Unguarded raw FFI / `AltrepGuard::Unsafe` callbacks
//! must not queue conditions; choose a guarded callback instead.

use std::cell::{Cell, OnceCell, RefCell};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};

use crate::SEXP;
use crate::condition::{RCondition, RConditionData};

// region: Queue

std::thread_local! {
    /// Conditions queued during the current guarded callback (and enclosing
    /// calls), on R's main thread; other threads send theirs over the channel
    /// ([`defer`]). Every borrow ends before any R signalling, so handlers can
    /// re-enter Rust and queue again.
    static PENDING: RefCell<Vec<RCondition>> = const { RefCell::new(Vec::new()) };
    /// Open finalizer suppressions ([`Suppress`]).
    static SUPPRESSED: Cell<usize> = const { Cell::new(0) };
    /// Receiving end of the channel from other threads: set on the thread
    /// that opened it (R's main thread), empty on every other thread.
    static RECEIVER: OnceCell<Receiver<RCondition>> = const { OnceCell::new() };
}

/// Capacity of the channel carrying conditions from other threads.
const CHANNEL_CAPACITY: usize = 1024;

/// Sending end of the channel from other threads. Any thread sends on it.
static SENDER: OnceLock<SyncSender<RCondition>> = OnceLock::new();

/// Conditions dropped because the channel was full (or not open yet), since
/// the main thread last received.
static DROPPED: AtomicUsize = AtomicUsize::new(0);

/// Why [`capture`] refuses to run off R's main thread.
const CAPTURE_OFF_MAIN_THREAD: &str = "deferred_condition::capture() was called off R's main \
     thread.\n\
     \n\
     Conditions deferred on another thread go to R's main thread, so there is \
     nothing here to capture. Call capture() on the main thread, around the \
     parallel section (Rayon, std::thread) that queues them.";

/// Open the channel from other threads and keep its receiving end on the
/// current thread. [`crate::miniextendr_runtime_init`] calls it on R's main
/// thread; only the first call opens it.
pub(crate) fn open_channel() {
    SENDER.get_or_init(|| {
        let (tx, rx) = sync_channel(CHANNEL_CAPACITY);
        RECEIVER.with(|cell| {
            cell.get_or_init(|| rx);
        });
        tx
    });
}

/// Queue a warning to be signalled when the current guarded call or callback
/// returns; the call still returns its value.
///
/// `payload` supplies the message, the user classes (prepended to
/// `rust_warning`) and the `data` fields (`w$<name>`). See the
/// [module docs](self) for the R-side behaviour and [`defer_warning!`](crate::defer_warning!)
/// for the macro form.
///
/// A later `Err` does not cancel this warning. To avoid warnings on failure,
/// queue after the last fallible step; see [warning placement](self#warning-placement).
///
/// From another thread it goes to R's main thread over a channel; see
/// [Threads and boundaries](self#threads-and-boundaries).
pub fn defer_warning(payload: impl RConditionData) {
    defer(RCondition::Warning {
        message: payload.message(),
        class: payload.class(),
        data: payload.data(),
        call: payload.call(),
    });
}

/// Queue a message (`message()`, muffled by `suppressMessages()`) to be
/// emitted when the current guarded call or callback returns.
///
/// Unlike [`crate::message!`], the payload's classes are honoured: they are
/// layered in front of `rust_message` so handlers can dispatch on them.
///
/// From another thread it goes to R's main thread over a channel; see
/// [Threads and boundaries](self#threads-and-boundaries).
pub fn defer_message(payload: impl RConditionData) {
    defer(RCondition::Message {
        message: payload.message(),
        class: payload.class(),
        data: payload.data(),
        call: payload.call(),
    });
}

/// Queue a plain condition (`signalCondition()`: silent without a handler)
/// to be signalled when the current guarded call or callback returns.
///
/// From another thread it goes to R's main thread over a channel; see
/// [Threads and boundaries](self#threads-and-boundaries).
pub fn defer_condition(payload: impl RConditionData) {
    defer(RCondition::Condition {
        message: payload.message(),
        class: payload.class(),
        data: payload.data(),
        call: payload.call(),
    });
}

/// Run `f` and return the conditions it queued, oldest first, instead of
/// signalling them when the guarded call returns.
///
/// For code that queues diagnostics a caller already has: replaying the
/// steps that built a saved object repeats warnings the object recorded.
/// `capture(f).0` discards them; each [`RCondition`](enum@RCondition) can also be inspected,
/// queued again with [`RCondition::defer`] or raised with
/// [`RCondition::raise`].
///
/// ```ignore
/// use miniextendr_api::deferred_condition::capture;
///
/// let (rebuilt, warnings) = capture(|| rebuild(&snapshot));
/// debug_assert_eq!(warnings.len(), snapshot.warnings.len());
/// ```
///
/// - Conditions queued before the block, or by enclosing calls, stay queued.
/// - Conditions a nested `#[miniextendr]` call queued were signalled when
///   that call returned, as always.
/// - If `f` unwinds (`error!`, `Err` through a nested raise, an R error), the
///   block's conditions stay queued and follow the usual rules: a Rust
///   failure signals them, an R-origin error discards them. To keep them on
///   failure too, catch the failure inside the block:
///   `capture(|| catch_condition(|| rebuild(&snapshot)))` (see
///   [`catch_condition`](crate::condition::catch_condition)).
/// - Where deferred conditions are suppressed (finalizers, see the
///   [module docs](self#threads-and-boundaries)), nothing is queued and the
///   vector is empty.
/// - Conditions that tasks of a parallel section inside the block queued are
///   captured too: they have arrived when the section returns.
///
/// # Panics
///
/// Off R's main thread, where the queue isn't: capture around the parallel
/// section instead (see [Threads and boundaries](self#threads-and-boundaries)).
pub fn capture<T>(f: impl FnOnce() -> T) -> (T, Vec<RCondition>) {
    if !crate::is_r_main_thread() {
        panic!("{CAPTURE_OFF_MAIN_THREAD}");
    }
    let start = mark();
    let value = f();
    (value, take_pending(start))
}

/// Queue an already-built condition payload. Used by the `defer_*!` macros.
///
/// Off R's main thread the condition goes over the channel, with `try_send`:
/// the main thread may be joining this one, so a send must never wait. A full
/// channel (or one not open yet) drops it and counts it in `DROPPED`.
#[doc(hidden)]
pub fn defer(condition: RCondition) {
    if crate::is_r_main_thread() {
        push(condition);
    } else {
        let sent = SENDER
            .get()
            .is_some_and(|tx| tx.try_send(condition).is_ok());
        if !sent {
            DROPPED.fetch_add(1, Ordering::Relaxed);
        }
    }
}

/// Queue on this thread; [`defer`] checks the thread first.
fn push(condition: RCondition) {
    if SUPPRESSED.get() == 0 {
        PENDING.with_borrow_mut(|queue| queue.push(condition));
    }
}

/// Append what other threads sent to the queue, then the warning for any
/// that were dropped. A no-op off the thread that opened the channel.
///
/// Called before the queue is read or cut back ([`take_pending`],
/// [`discard`]), not by [`mark`]: what arrived belongs to the call that reads
/// next, not to the queue below a new mark, which no call would take.
fn receive() {
    RECEIVER.with(|cell| {
        let Some(rx) = cell.get() else { return };
        PENDING.with_borrow_mut(|queue| {
            // Bounded: other threads keep sending while this runs.
            queue.extend(rx.try_iter().take(CHANNEL_CAPACITY));
            // After the conditions kept: a full channel drops the newest.
            let dropped = DROPPED.swap(0, Ordering::Relaxed);
            if dropped > 0 {
                queue.push(RCondition::Warning {
                    message: format!(
                        "{dropped} deferred conditions from other threads dropped due to \
                         queue overflow"
                    ),
                    class: Vec::new(),
                    data: None,
                    call: crate::condition::ConditionCall::Inherit,
                });
            }
        });
    });
}

/// The queue position at the start of a call. Pair with [`finish`].
#[doc(hidden)]
#[inline]
pub fn mark() -> usize {
    PENDING.with_borrow(Vec::len)
}

/// Remove and return everything queued since `mark`, oldest first. Entries
/// below `mark` belong to enclosing calls and stay.
#[doc(hidden)]
pub fn take_pending(mark: usize) -> Vec<RCondition> {
    // A nested guard inside a finalizer must not drain the enclosing call's
    // entries: they belong to its signal-capable boundary.
    if SUPPRESSED.get() != 0 {
        return Vec::new();
    }
    receive();
    PENDING.with_borrow_mut(|queue| {
        if queue.len() <= mark {
            Vec::new()
        } else {
            queue.split_off(mark)
        }
    })
}

/// Discard conditions whose Rust/R call was abandoned by an R longjmp.
pub(crate) fn discard(mark: usize) {
    if SUPPRESSED.get() == 0 {
        receive();
        PENDING.with_borrow_mut(|queue| queue.truncate(mark));
    }
}

/// Finalization has no caller condition handlers. Suppress queues throughout
/// the destructor, including any nested framework boundaries, without R calls.
pub(crate) struct Suppress(std::marker::PhantomData<std::rc::Rc<()>>);

impl Suppress {
    pub(crate) fn new() -> Self {
        SUPPRESSED.set(SUPPRESSED.get() + 1);
        Self(std::marker::PhantomData)
    }
}

impl Drop for Suppress {
    fn drop(&mut self) {
        SUPPRESSED.set(SUPPRESSED.get() - 1);
    }
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

/// Flush a non-raising callback's conditions after its Rust guard has finished.
/// R objects in `value` must already be rooted (the same contract as `or_raise`).
pub(crate) fn finish_guarded<R>(mark: usize, value: R) -> R {
    let queued = take_pending(mark);
    if queued.is_empty() {
        return value;
    }
    crate::unwind_protect::with_r_unwind_protect_or_raise(
        move || {
            // SAFETY: callback boundary is on R's main thread; this closure is
            // inside R_UnwindProtect, including the owned result's destructor.
            unsafe { signal_now(queued, None) };
            value
        },
        None,
    )
}

/// Materialise each queued condition and hand it to the R helper.
///
/// # Safety
///
/// Main thread, inside `R_UnwindProtect` (handlers may longjmp).
pub(crate) unsafe fn signal_now(queued: Vec<RCondition>, call: Option<SEXP>) {
    let helper = raise_condition_helper();
    let call_sexp = call.unwrap_or(SEXP::nil());
    for condition in queued {
        // SAFETY: main thread inside R_UnwindProtect (caller contract). The
        // tagged value and the call object are rooted across `Rf_eval`. A
        // call-less condition carries the "no call" marker, which the helper
        // maps to `call = NULL` instead of falling back to `call_sexp`.
        unsafe {
            let tagged =
                crate::OwnedProtect::new(crate::error_value::rust_condition_value(condition, call));
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
/// The raising guards raise their conditions with it too
/// (`unwind_protect::raise_rust_condition_via_stop`).
pub(crate) fn raise_condition_helper() -> SEXP {
    static HELPER: OnceLock<SEXP> = OnceLock::new();
    *HELPER.get_or_init(|| {
        // SAFETY: only reached from `signal_now` and
        // `raise_rust_condition_via_stop`, on the main thread.
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
/// the surrounding guarded call or callback signals it after its value is
/// computed.
///
/// Same grammar as [`crate::warning!`]: optional `call = none` (no call, like
/// R's `warning(..., call. = FALSE)`), optional `class = …` (one class or a
/// vector, most specific first), optional `data = …` (a pair, a bracketed
/// list of pairs, or `{ name = value }` sugar), then the `format!` message.
/// For a typed payload use [`crate::defer_warning()`] with a
/// `#[derive(RCondition)]` type (`#[condition(call = none)]` on the type
/// or a variant drops the call there).
///
/// A later `Err` does not cancel this warning. To avoid warnings on failure,
/// queue after the last fallible step; see [warning placement](crate::deferred_condition#warning-placement).
///
/// From another thread it goes to R's main thread over a channel; see
/// [Threads and boundaries](crate::deferred_condition#threads-and-boundaries).
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
        let (__mx_call, __mx_class, __mx_data, __mx_message) =
            $crate::__mx_condition_parts!($($t)*);
        $crate::deferred_condition::defer($crate::condition::RCondition::Warning {
            message: __mx_message,
            class: __mx_class,
            data: __mx_data,
            call: __mx_call,
        });
    }};
}

/// Queue an R message (`rust_message` layering, muffled by
/// `suppressMessages()`) and return normally; emitted when the surrounding
/// guarded call or callback returns.
///
/// Same grammar as [`crate::defer_warning!`]. Unlike [`crate::message!`] a
/// `class = …` part is accepted and layered in front of `rust_message`.
/// `call = none` is accepted for the shared grammar and changes nothing: an R
/// message carries no call, so `conditionCall(m)` is `NULL` either way.
///
/// From another thread it goes to R's main thread over a channel; see
/// [Threads and boundaries](crate::deferred_condition#threads-and-boundaries).
#[macro_export]
macro_rules! defer_message {
    ($($t:tt)*) => {{
        let (__mx_call, __mx_class, __mx_data, __mx_message) =
            $crate::__mx_condition_parts!($($t)*);
        $crate::deferred_condition::defer($crate::condition::RCondition::Message {
            message: __mx_message,
            class: __mx_class,
            data: __mx_data,
            call: __mx_call,
        });
    }};
}

/// Queue a plain R condition (`rust_condition` layering, silent without a
/// handler) and return normally; signalled when the surrounding
/// guarded call or callback returns.
///
/// Same grammar as [`crate::defer_warning!`]. Useful for progress or audit
/// events a caller may opt into with `withCallingHandlers()`.
///
/// From another thread it goes to R's main thread over a channel; see
/// [Threads and boundaries](crate::deferred_condition#threads-and-boundaries).
#[macro_export]
macro_rules! defer_condition {
    ($($t:tt)*) => {{
        let (__mx_call, __mx_class, __mx_data, __mx_message) =
            $crate::__mx_condition_parts!($($t)*);
        $crate::deferred_condition::defer($crate::condition::RCondition::Condition {
            message: __mx_message,
            class: __mx_class,
            data: __mx_data,
            call: __mx_call,
        });
    }};
}

// endregion

// region: Tests — queue semantics (no R runtime needed)

#[cfg(test)]
mod tests;

// endregion
