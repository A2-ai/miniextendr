//! Condition macros and signal enum for the Rust→R condition pipeline.
//!
//! This module provides two things:
//!
//! 1. **[`enum@RCondition`] enum** — the internal panic payload used by `error!()`,
//!    `warning!()`, `message!()`, and `condition!()` macros. Caught by
//!    [`crate::unwind_protect::with_r_unwind_protect`] before the generic
//!    panic→error path, then forwarded to R as a structured condition with
//!    `rust_*` class layering via
//!    [`crate::error_value::make_rust_condition_value`].
//!
//! 2. **[`AsRError`] struct** — wraps any `E: std::error::Error` and
//!    preserves the full error chain (cause/source) when converting to an R
//!    error message. Use as the `Err` type in `Result` returns.
//!
//! # When to reach for what
//!
//! There are three Rust→R error-emission paths and they are not
//! interchangeable. The crate-level rationale (why tagged-SEXP at all, what
//! `error_in_r` defaults imply, and the `with_r_unwind_protect` leak) lives
//! on [`crate::error_value`]; the practical picking-one summary:
//!
//! - **`panic!`** — escape hatch. Becomes class `c("rust_error",
//!   "simpleError", "error", "condition")` with `kind = "panic"`. Use for
//!   genuine bugs or impossible states. Cheapest in source; coarsest in R
//!   (callers can only match `rust_error` / `error`, not a specific class).
//! - **`error!` / `warning!` / `message!` / `condition!`** (this module) —
//!   typed conditions. Same transport, but allow an optional `class =
//!   "name"` so R-side `tryCatch` can route by class. `warning!` /
//!   `message!` / `condition!` are the only way to emit non-error
//!   conditions; `panic!` is always an error.
//! - **`Result<T, E>` with [`AsRError<E>`]** — value-style propagation
//!   through Rust code. Converts at the boundary; `kind = "result_err"`.
//!   Best when the failure path is real-and-recoverable in Rust and the
//!   error chain (`std::error::Error::source`) is worth preserving.
//!
//! `Rf_error` is *not* on this list. Direct `Rf_error` skips Rust
//! destructors unconditionally and is forbidden by lint MXL300; see
//! [`crate::error_value`] for the full reasoning.
//!
//! # Macro-vs-module name collision
//!
//! `#[macro_export]` puts each macro at the *crate root*, where `error!` and
//! `condition!` collide with the same-named modules `pub mod error` and `pub
//! mod condition`. The practical implication: `use miniextendr_api::{error,
//! condition}` imports the *modules*, not the macros, and a subsequent
//! `error!(...)` call fails to resolve.
//!
//! Workarounds, in rough order of ergonomics:
//!
//! 1. Invoke via fully-qualified path: `miniextendr_api::error!("...")`.
//! 2. `use miniextendr_api as mx;` then `mx::error!("...")`.
//! 3. `warning!` and `message!` have no module conflict — `use
//!    miniextendr_api::{warning, message};` works directly.
//!
//! See the individual macro docs for the per-macro reminder.
//!
//! # Condition macros
//!
//! The four macros are the user-facing API for raising non-panic conditions from
//! Rust. They ride the tagged-condition transport that every `#[miniextendr]`
//! function uses:
//!
//! ```ignore
//! use miniextendr_api::{error, warning, message, condition};
//!
//! #[miniextendr]
//! fn demo_error() {
//!     error!("something went wrong: {}", 42);
//! }
//!
//! #[miniextendr]
//! fn demo_warning() {
//!     warning!("something looks suspicious");
//! }
//!
//! #[miniextendr]
//! fn demo_message() {
//!     message!("progress: {} of {}", 1, 10);
//! }
//!
//! #[miniextendr]
//! fn demo_condition() {
//!     condition!("a signallable condition");
//! }
//! ```
//!
//! Optional `class =` extension for programmatic catching:
//!
//! ```ignore
//! #[miniextendr]
//! fn typed_error(name: &str) {
//!     error!(class = "my_error", "missing field: {name}");
//! }
//! ```
//!
//! ```r
//! tryCatch(typed_error("x"), my_error = function(e) "caught!")
//! # [1] "caught!"
//! ```
//!
//! Optional `data =` extension attaches structured fields readable as
//! `e$<name>` in handlers (rlang-`abort()`-style):
//!
//! ```ignore
//! #[miniextendr]
//! fn validate(value: i32) {
//!     if !(0..=100).contains(&value) {
//!         miniextendr_api::error!(
//!             class = "validation_error",
//!             data = [("value", value), ("min", 0), ("max", 100)],
//!             "value {value} out of range"
//!         );
//!     }
//! }
//! ```
//!
//! ```r
//! tryCatch(validate(150L), validation_error = function(e) c(e$value, e$min, e$max))
//! # [1] 150   0 100
//! ```
//!
//! Supported `data` value types (anything with `RValue: From<_>`): scalars and
//! `Vec`s of `i32`, `f64`, `bool`, `String` / `&str`; their NA-aware `Option` /
//! `Vec<Option<_>>` forms (`None` → R `NA`); the wide-integer ladder (`i64` /
//! `u32`, narrowed to `integer(1)` when it fits, `double(1)` otherwise); and the
//! [`RValue::debug`](crate::RValue::debug) escape hatch, which stringifies any
//! `T: Debug`. For nested lists or complex/raw/NA-bearing values build an
//! [`RValue`](crate::RValue) directly. The payload is built as a Send-safe owned
//! value at the call site and materialised as R objects at the unwind
//! boundary.
//!
//! Three `data =` grammars are accepted (see [`crate::error!`]):
//! - single pair: `data = ("name", value)`
//! - bracketed list: `data = [("a", v1), ("b", v2)]`
//! - keyed builder sugar: `data = { value = 42, code = 7 }` (bare-ident keys)
//!
//! # `AsRError`
//!
//! ```ignore
//! use miniextendr_api::condition::AsRError;
//!
//! #[miniextendr]
//! fn parse_config(path: &str) -> Result<i32, AsRError<std::io::Error>> {
//!     let content = std::fs::read_to_string(path).map_err(AsRError)?;
//!     Ok(content.len() as i32)
//! }
//! ```

// region: ConditionData — Send-safe owned condition-data payload

/// Named condition-data payload: an ordered list of `(name, value)` pairs.
///
/// Produced by the macros' `data = ...` form and consumed by
/// [`crate::error_value::make_rust_condition_value`]. Each value is an
/// [`RValue`](crate::RValue) — an owned, `Send`, R-native value tree. Send-safe
/// by construction (no live `SEXP`), so the payload can travel through
/// `panic_any`, whose payload must be `Send`; the R objects are materialised
/// on the main thread at the unwind boundary.
///
/// The macros accept any value with `RValue: From<_>` (scalars and `Vec`s of
/// `i32` / `f64` / `bool` / `String` / `&str`; their NA-aware `Option` /
/// `Vec<Option<_>>` forms; the `i64` / `u32` wide-integer ladder); a scalar
/// `7i32` becomes `integer(1)` and a `Vec<i32>` becomes `integer(n)`. Any
/// `T: Debug` rides along via [`RValue::debug`](crate::RValue::debug). For
/// nested lists or complex/raw values build an [`RValue`](crate::RValue)
/// directly.
pub type ConditionData = Vec<(String, crate::RValue)>;

// endregion

// region: ConditionCall — whether a condition reports a call

/// Whether a condition reports a call (`conditionCall()`): the per-condition
/// equivalent of R's `warning(..., call. = FALSE)` / `stop(..., call. = FALSE)`.
///
/// Every condition a `#[miniextendr]` function raises carries the call its
/// transport captured: the wrapper's call as written (or the caller's, under
/// `call = caller`), `NULL` in ALTREP and connection callbacks. A condition
/// about the data rather than the call opts out with [`ConditionCall::None`];
/// its siblings from the same function keep their call.
///
/// Spellings: `call = none` first in [`crate::warning!`], [`crate::error!`],
/// [`crate::condition!`] and the `defer_*!` macros;
/// [`RError::without_call`]; `#[condition(call = none)]` on a
/// `#[derive(RCondition)]` type or variant; or an
/// [`RConditionData::call`] impl.
///
/// On the tagged condition value the choice is the `call` slot itself:
/// `FALSE` for [`ConditionCall::None`], read back by the R helper (and by the
/// trait-ABI re-panic, [`RCondition::from_tagged_sexp`]) as "no call".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConditionCall {
    /// The call the transport captured (the default).
    #[default]
    Inherit,
    /// No call: `conditionCall()` is `NULL` and the printed condition has no
    /// `In f(...) :` prefix.
    None,
}

impl ConditionCall {
    /// `call` when the condition keeps its call, `None` otherwise: the call to
    /// hand a transport that takes the R call object directly.
    #[inline]
    pub fn apply(self, call: Option<crate::SEXP>) -> Option<crate::SEXP> {
        match self {
            ConditionCall::Inherit => call,
            ConditionCall::None => Option::None,
        }
    }
}

// endregion

// region: RCondition enum — structured condition payload

/// Owned payload for structured R conditions.
///
/// `#[derive(RCondition)]` gives a struct or enum a conversion into this
/// payload. Its `#[condition(kind = warning)]` (or `error` / `message` /
/// `condition`) chooses the signal; an enum variant can override the kind.
/// Convert with `RCondition::from(value)`, then call [`defer`](Self::defer)
/// to accompany a return value, or [`raise`](Self::raise) to leave Rust.
///
/// Raised by the `error!()`, `warning!()`, `message!()`, and `condition!()`
/// macros via `std::panic::panic_any`. Caught by `with_r_unwind_protect`
/// before the generic panic→string path and forwarded to R as a tagged SEXP
/// with `rust_*` class layering. The `defer_*` family
/// ([`crate::deferred_condition`]) queues the same payloads instead of
/// panicking, so the call still returns its value.
///
/// Every variant carries the same four parts: the message, the user classes
/// (prepended to the `rust_*` layering, empty for none), the optional `data`
/// fields and the [`ConditionCall`] choice. All four are owned and `Send`, so
/// the payload can travel through `panic_any`. Each variant is one kind of the
/// tagged condition value ([`crate::error_value::kind`]).
#[derive(Debug, Clone)]
pub enum RCondition {
    /// Raised by `error!(...)` / `error!(class = "...", ...)`, and by the
    /// `Result<T, E>` Err arm once reconstructed across a package boundary.
    Error {
        /// Human-readable condition message.
        message: String,
        /// User classes, most specific first, before the framework's classes.
        class: Vec<String>,
        /// Named fields attached to the R condition.
        data: Option<ConditionData>,
        /// Call attribution policy; messages always omit the call.
        call: ConditionCall,
    },
    /// Raised by `warning!(...)` / `warning!(class = "...", ...)`.
    Warning {
        /// Human-readable condition message.
        message: String,
        /// User classes, most specific first, before the framework's classes.
        class: Vec<String>,
        /// Named fields attached to the R condition.
        data: Option<ConditionData>,
        /// Call attribution policy; messages always omit the call.
        call: ConditionCall,
    },
    /// Raised by `message!(...)`, and queued by [`crate::defer_message()`]. Only
    /// the typed-payload path fills `class` (the macro has no `class =` form);
    /// the R helper layers it in front of `rust_message` like the other kinds.
    /// A message never reports a call, whatever `call` says.
    Message {
        /// Human-readable condition message.
        message: String,
        /// User classes, most specific first, before the framework's classes.
        class: Vec<String>,
        /// Named fields attached to the R condition.
        data: Option<ConditionData>,
        /// Call attribution policy; messages always omit the call.
        call: ConditionCall,
    },
    /// Raised by `condition!(...)` / `condition!(class = "...", ...)`.
    Condition {
        /// Human-readable condition message.
        message: String,
        /// User classes, most specific first, before the framework's classes.
        class: Vec<String>,
        /// Named fields attached to the R condition.
        data: Option<ConditionData>,
        /// Call attribution policy; messages always omit the call.
        call: ConditionCall,
    },
    /// An argument error, `kind = "conversion"`: raised by
    /// [`crate::arg_error!`] / [`ArgError::raise`] (built by
    /// [`RCondition::arg_error`]), and a failed argument conversion once
    /// reconstructed across a package boundary.
    Conversion {
        /// Human-readable condition message.
        message: String,
        /// User classes, most specific first, before the framework's classes.
        class: Vec<String>,
        /// Named fields attached to the R condition.
        data: Option<ConditionData>,
        /// Call attribution policy; messages always omit the call.
        call: ConditionCall,
    },
}

impl RCondition {
    /// Leave the current Rust body with this condition.
    ///
    /// The enclosing miniextendr guard forwards it to R with its kind and
    /// metadata intact. Like `warning!`, raising a warning still exits Rust:
    /// muffling it in R does not resume this Rust body. Use [`defer`](Self::defer)
    /// to keep computing and return a value.
    pub fn raise(self) -> ! {
        std::panic::panic_any(self)
    }

    /// Queue this condition until the current guarded call returns.
    ///
    /// Preserves its declared kind, classes, fields and call policy, including
    /// for a mixed-kind enum. Errors queued here stop R at that boundary.
    ///
    /// Panics off R's main thread; see
    /// [Threads and boundaries](crate::deferred_condition#threads-and-boundaries).
    pub fn defer(self) {
        crate::deferred_condition::defer(self);
    }

    /// Split into the kind (one of [`crate::error_value::kind`]) and the
    /// [`ErrParts`] (message, class, data, call choice) that
    /// [`crate::error_value::condition_parts_value`] builds the tagged value
    /// from.
    #[doc(hidden)]
    pub fn into_parts(self) -> (&'static str, ErrParts) {
        use crate::error_value::kind;
        let (kind, message, class, data, call) = match self {
            RCondition::Error {
                message,
                class,
                data,
                call,
            } => (kind::ERROR, message, class, data, call),
            RCondition::Warning {
                message,
                class,
                data,
                call,
            } => (kind::WARNING, message, class, data, call),
            RCondition::Message {
                message,
                class,
                data,
                call,
            } => (kind::MESSAGE, message, class, data, call),
            RCondition::Condition {
                message,
                class,
                data,
                call,
            } => (kind::CONDITION, message, class, data, call),
            RCondition::Conversion {
                message,
                class,
                data,
                call,
            } => (kind::CONVERSION, message, class, data, call),
        };
        (
            kind,
            ErrParts {
                message,
                class,
                data,
                call,
            },
        )
    }

    /// The condition message, as `conditionMessage()` would read it in R.
    ///
    /// Use it on a condition [`catch_condition`] caught or
    /// [`capture`](crate::deferred_condition::capture) collected, for example
    /// to put a cause's text into a new message.
    pub fn message(&self) -> &str {
        match self {
            RCondition::Error { message, .. }
            | RCondition::Warning { message, .. }
            | RCondition::Message { message, .. }
            | RCondition::Condition { message, .. }
            | RCondition::Conversion { message, .. } => message,
        }
    }

    /// This condition with `parent` as its cause, as `rlang::abort(parent = )`
    /// chains conditions.
    ///
    /// R gets the cause as the field `e$parent`: a condition object with its
    /// own classes, message and fields (and its own parent, if it has one),
    /// built like the condition itself but not signalled. The cause's classes
    /// stay out of this condition's class vector, so a handler written for
    /// the cause's class does not catch this condition; code that wants the
    /// cause branches on it: `inherits(e$parent, "pkg_error_config")`.
    ///
    /// The cause reports the same call as this condition, or none when it was
    /// raised with `call = none`. A second `with_parent` replaces the first,
    /// and the cause replaces a `data` field this condition names `parent`.
    ///
    /// ```ignore
    /// match catch_condition(|| load_config(path)) {
    ///     Ok(value) => value,
    ///     Err(cause) => RCondition::Error {
    ///         message: format!("could not load the model: {}", cause.message()),
    ///         class: vec!["pkg_error_load".into()],
    ///         data: None,
    ///         call: ConditionCall::Inherit,
    ///     }
    ///     .with_parent(cause)
    ///     .raise(),
    /// }
    /// ```
    pub fn with_parent(mut self, parent: RCondition) -> Self {
        let encoded = parent.into_tagged_rvalue();
        let data = match &mut self {
            RCondition::Error { data, .. }
            | RCondition::Warning { data, .. }
            | RCondition::Message { data, .. }
            | RCondition::Condition { data, .. }
            | RCondition::Conversion { data, .. } => data,
        };
        let fields = data.get_or_insert_with(Vec::new);
        fields.retain(|(name, _)| name != PARENT_FIELD);
        fields.push((PARENT_FIELD.to_owned(), encoded));
        self
    }

    /// The condition as the tagged value the R raise helper reads (`error`,
    /// `kind`, `class`, `call`, `data`), in [`RValue`](crate::RValue) form so
    /// it can travel inside another condition's data ([`Self::with_parent`]).
    fn into_tagged_rvalue(self) -> crate::RValue {
        use crate::RValue;
        let (kind, parts) = self.into_parts();
        let text = |value: String| RValue::Character(vec![Some(value)]);
        let class = if parts.class.is_empty() {
            RValue::Null
        } else {
            RValue::Character(parts.class.into_iter().map(Some).collect())
        };
        let call = match parts.call {
            ConditionCall::Inherit => RValue::Null,
            ConditionCall::None => RValue::Logical(vec![Some(false)]),
        };
        let data = parts.data.map_or(RValue::Null, |fields| {
            RValue::List(
                fields
                    .into_iter()
                    .map(|(name, value)| (Some(name), value))
                    .collect(),
            )
        });
        RValue::List(vec![
            (Some("error".to_owned()), text(parts.message)),
            (Some("kind".to_owned()), text(kind.to_owned())),
            (Some("class".to_owned()), class),
            (Some("call".to_owned()), call),
            (Some("data".to_owned()), data),
        ])
    }

    /// Which call the condition reports when R raises it: see
    /// [`ConditionCall`].
    pub fn call(&self) -> ConditionCall {
        match self {
            RCondition::Error { call, .. }
            | RCondition::Warning { call, .. }
            | RCondition::Message { call, .. }
            | RCondition::Condition { call, .. }
            | RCondition::Conversion { call, .. } => *call,
        }
    }

    /// The user classes and the fields, whatever the variant.
    fn class_and_data(&self) -> (&[String], &Option<ConditionData>) {
        match self {
            RCondition::Error { class, data, .. }
            | RCondition::Warning { class, data, .. }
            | RCondition::Message { class, data, .. }
            | RCondition::Condition { class, data, .. }
            | RCondition::Conversion { class, data, .. } => (class, data),
        }
    }

    /// The payload of [`crate::arg_error!`] and [`ArgError::raise`]: the
    /// argument error of parameter `param` (its R name) with `message`.
    ///
    /// `kind = "conversion"`, `e$param` as the only data field, and the class
    /// [`CONVERSION_ERROR_CLASS_MARKER`], which the generated R replaces with
    /// the crate's `conversion_error_class`: the condition the wrapper's own
    /// argument checks raise (`.miniextendr_arg_error`).
    #[doc(hidden)]
    pub fn arg_error(param: impl Into<String>, message: String, call: ConditionCall) -> Self {
        RCondition::Conversion {
            message,
            class: vec![CONVERSION_ERROR_CLASS_MARKER.to_string()],
            data: Some(vec![(
                CONVERSION_PARAM_FIELD.to_string(),
                crate::RValue::from(param.into()),
            )]),
            call,
        }
    }
}

/// The class that stands for the crate's `conversion_error_class` in a tagged
/// condition value (#1740).
///
/// A `macro_rules!` macro cannot read `[package.metadata.miniextendr]`, so
/// [`crate::arg_error!`] cannot name the crate's classes the way the
/// generated conversion arms do. It writes this class instead, and the
/// generated R wrapper's `.miniextendr_raise_condition` replaces it with
/// `.miniextendr_conversion_error_class`, the binding its own argument checks
/// read. The string is that binding's name. Where no generated wrapper raises
/// the condition (a raising guard: an ALTREP `RUnwind` callback,
/// [`crate::unwind_protect::with_r_unwind_protect_or_raise`]) Rust replaces it
/// with the classes `miniextendr_init!` registered for the package
/// ([`resolve_conversion_error_class_marker`]). So does a trait-ABI shim
/// ([`crate::unwind_protect::with_r_unwind_protect_shim`]): the condition is
/// raised by another package's wrapper, and takes the implementing package's
/// classes, as from that package's own wrapper.
#[doc(hidden)]
pub const CONVERSION_ERROR_CLASS_MARKER: &str = ".miniextendr_conversion_error_class";

/// `class` with [`CONVERSION_ERROR_CLASS_MARKER`] replaced by `crate_classes`,
/// in place, as the generated `.miniextendr_raise_condition` replaces it in R.
/// A raising guard has no generated R to do it, so it calls this with
/// [`crate_conversion_error_class`].
pub(crate) fn resolve_conversion_error_class_marker(
    class: Vec<String>,
    crate_classes: &[&str],
) -> Vec<String> {
    let mut resolved = Vec::with_capacity(class.len() + crate_classes.len());
    for c in class {
        if c == CONVERSION_ERROR_CLASS_MARKER {
            resolved.extend(crate_classes.iter().map(|&k| k.to_string()));
        } else {
            resolved.push(c);
        }
    }
    resolved
}

/// The package crate's `conversion_error_class`, from the entry
/// `miniextendr_init!` registers in
/// [`MX_CONVERSION_ERROR_CLASS`](crate::registry::MX_CONVERSION_ERROR_CLASS)
/// (each package links its own copy of this crate, so it is that package's),
/// or on wasm32 from the `wasm_registry.rs` snapshot (#1768). Empty when the
/// crate sets none.
pub(crate) fn crate_conversion_error_class() -> &'static [&'static str] {
    crate::registry::conversion_error_class()
}

/// Run `f`, returning a condition raised inside it as `Err` instead of
/// letting it reach R; any other unwind carries on.
///
/// For a body that turns another function's failure into its own result: a
/// refusal that names the reason, a fallback, a retry.
///
/// ```ignore
/// use miniextendr_api::condition::catch_condition;
///
/// match catch_condition(|| load_config(path)) {
///     Ok(object) => object,
///     Err(cause) => error!(
///         class = "pkg_error_load",
///         data = ("cause", cause.message().to_string()),
///         "cannot load the model: {}", cause.message()
///     ),
/// }
/// ```
///
/// **Caught:** every [`RCondition`](enum@RCondition) payload, with its kind, classes, data
/// and call choice: [`error!`](crate::error!) / [`rust_error!`](crate::rust_error!),
/// [`warning!`](crate::warning!), [`message!`](crate::message!),
/// [`condition!`](crate::condition!), [`RCondition::raise`], the argument and
/// conversion errors ([`RCondition::arg_error`]) and the `ExternalPtr` type
/// errors. An `Err` can be raised again with [`RCondition::raise`] or queued
/// with [`RCondition::defer`].
///
/// **Resumed** unchanged, with [`std::panic::resume_unwind`]:
/// - an R error, interrupt or other R exit crossing the Rust frames (R code
///   run through [`with_r_unwind_protect`](crate::unwind_protect::with_r_unwind_protect)
///   or [`eval_with_handlers`](crate::expression::eval_with_handlers)), so
///   R carries on to the context it was jumping to;
/// - a plain `panic!()` or any other payload, which still becomes an R error
///   at the boundary, with its source location.
///
/// Use [`try_eval_with_handlers`](crate::expression::try_eval_with_handlers)
/// to get an R error back as a value.
///
/// Deferred conditions queued inside `f` stay queued (see
/// [`deferred_condition::capture`](crate::deferred_condition::capture)). A
/// caught raise still runs the panic hook: with `MINIEXTENDR_BACKTRACE=true`
/// it prints the panic to stderr.
pub fn catch_condition<T>(f: impl FnOnce() -> T) -> Result<T, RCondition> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(value) => Ok(value),
        Err(payload) => match payload.downcast::<RCondition>() {
            Ok(condition) => Err(*condition),
            Err(payload) => std::panic::resume_unwind(payload),
        },
    }
}

// endregion

// region: Macros

/// Internal: one condition-data field name, validated against the reserved
/// slots. Not part of the public API.
///
/// - `lit "name"` / `ident name`: the name is a literal, so the reserved-name
///   check runs at compile time (`const` assertion) and the field costs one
///   `to_string()` at runtime.
/// - `expr <e>`: a computed name; checked at runtime by
///   [`crate::condition::check_condition_field_name`].
#[doc(hidden)]
#[macro_export]
macro_rules! __mx_condition_field_name {
    (lit $name:literal) => {{
        const _: () = ::core::assert!(
            !$crate::condition::is_reserved_condition_field($name),
            "condition data field name is reserved: `message`, `call` and `kind` are the condition's own slots, and `.mx_parent` carries its parent; rename the field"
        );
        ($name).to_string()
    }};
    (ident $name:ident) => {{
        const _: () = ::core::assert!(
            !$crate::condition::is_reserved_condition_field(::core::stringify!($name)),
            "condition data field name is reserved: `message`, `call` and `kind` are the condition's own slots, and `.mx_parent` carries its parent; rename the field"
        );
        ::core::stringify!($name).to_string()
    }};
    (expr $name:expr) => {
        $crate::condition::check_condition_field_name(($name).to_string())
    };
}

/// Internal: one `(name, value)` pair of a bracketed `data = [...]` list; the
/// name goes through the reserved-slot check (compile time for literals).
#[doc(hidden)]
#[macro_export]
macro_rules! __mx_condition_pair {
    (($name:literal, $value:expr $(,)?)) => {
        ($crate::__mx_condition_field_name!(lit $name), $crate::RValue::from($value))
    };
    (($name:expr, $value:expr $(,)?)) => {
        ($crate::__mx_condition_field_name!(expr $name), $crate::RValue::from($value))
    };
}

/// Internal: normalise a macro `data = ...` argument into
/// `Option<ConditionData>`. Not part of the public API.
///
/// Three forms are accepted:
/// - a single pair: `("name", value)`
/// - a bracketed list of pairs: `[("a", v1), ("b", v2)]`
/// - keyed builder sugar: `{ name = value, other = value }` — the field name is
///   a bare identifier (stringified by the macro), so `{ value = 42, code = 7 }`
///   is shorthand for `[("value", 42), ("code", 7)]`.
///
/// Each `value` is converted via `RValue::from`, so any type with an `RValue`
/// `From` impl (the scalar/vector/`Option`/wide-integer set, and tuples of
/// those as unnamed lists) works without ceremony.
///
/// Field names are checked against the condition's own slots (`message` /
/// `call` / `kind`): at compile time when the name is a literal or bare
/// identifier, at runtime otherwise.
#[doc(hidden)]
#[macro_export]
macro_rules! __mx_condition_data {
    (($name:literal, $value:expr $(,)?)) => {
        ::std::option::Option::Some(::std::vec![(
            $crate::__mx_condition_field_name!(lit $name),
            $crate::RValue::from($value),
        )])
    };
    (($name:expr, $value:expr $(,)?)) => {
        ::std::option::Option::Some(::std::vec![(
            $crate::__mx_condition_field_name!(expr $name),
            $crate::RValue::from($value),
        )])
    };
    ([ $($pair:tt),* $(,)? ]) => {
        ::std::option::Option::Some(::std::vec![
            $( $crate::__mx_condition_pair!($pair), )*
        ])
    };
    ({ $($name:ident = $value:expr),* $(,)? }) => {
        ::std::option::Option::Some(::std::vec![
            $(
                (
                    $crate::__mx_condition_field_name!(ident $name),
                    $crate::RValue::from($value),
                ),
            )*
        ])
    };
}

/// Internal: parse the shared option grid of `error!` / `warning!` /
/// `condition!` and the `defer_*!` macros into `(call, class, data, message)`.
/// Not part of the public API.
///
/// Accepted orders (every part optional except the message):
/// `call = none`, then `class = ..`, then `data = ..`, then the `format!`
/// arguments. `class` takes anything implementing
/// [`crate::condition::ConditionClass`] (one string or several). A `call`
/// other than `none`, or one after `class` / `data`, is a compile error: a
/// `format!` string must start with a literal, so `call` as the first token is
/// never a message.
///
/// ```compile_fail
/// # use miniextendr_api as mx;
/// # fn f() {
/// mx::warning!(call = wrapper, "no such call option");
/// # }
/// ```
///
/// ```compile_fail
/// # use miniextendr_api as mx;
/// # fn f() {
/// mx::warning!(class = "pkg_warning", call = none, "`call` goes first");
/// # }
/// ```
///
/// ```
/// # use miniextendr_api as mx;
/// # fn f() {
/// mx::warning!(call = none, class = "pkg_warning", data = { n = 1 }, "plain");
/// # }
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! __mx_condition_parts {
    // The option grid after the `call` choice: `(class, data, message)`.
    (@options class = $class:expr, call $($rest:tt)*) => {
        $crate::__mx_condition_parts!(@misplaced_call)
    };
    (@options class = $class:expr, data = $data:tt, call $($rest:tt)*) => {
        $crate::__mx_condition_parts!(@misplaced_call)
    };
    (@options data = $data:tt, call $($rest:tt)*) => {
        $crate::__mx_condition_parts!(@misplaced_call)
    };
    (@options class = $class:expr, data = $data:tt, $($arg:tt)*) => {
        (
            $crate::condition::ConditionClass::into_condition_class($class),
            $crate::__mx_condition_data!($data),
            ::std::format!($($arg)*),
        )
    };
    (@options data = $data:tt, $($arg:tt)*) => {
        (
            ::std::vec::Vec::<::std::string::String>::new(),
            $crate::__mx_condition_data!($data),
            ::std::format!($($arg)*),
        )
    };
    (@options class = $class:expr, $($arg:tt)*) => {
        (
            $crate::condition::ConditionClass::into_condition_class($class),
            ::std::option::Option::<$crate::condition::ConditionData>::None,
            ::std::format!($($arg)*),
        )
    };
    (@options $($arg:tt)*) => {
        (
            ::std::vec::Vec::<::std::string::String>::new(),
            ::std::option::Option::<$crate::condition::ConditionData>::None,
            ::std::format!($($arg)*),
        )
    };
    (@options call $($rest:tt)*) => {
        ::core::compile_error!("condition macros: `call = none` is given twice")
    };
    (@misplaced_call) => {
        ::core::compile_error!(
            "condition macros: `call = none` comes first, before `class = ...` and `data = ...`: \
             `warning!(call = none, class = \"...\", \"...\")`"
        )
    };
    // Entry: the optional `call = none`, then the option grid.
    (call = none, $($rest:tt)+) => {{
        let (__mx_class, __mx_data, __mx_message) =
            $crate::__mx_condition_parts!(@options $($rest)+);
        (
            $crate::condition::ConditionCall::None,
            __mx_class,
            __mx_data,
            __mx_message,
        )
    }};
    (call $($rest:tt)*) => {
        ::core::compile_error!(
            "condition macros: the only call option is `call = none` (no call, like R's \
             `call. = FALSE`), and it comes first, followed by a comma: \
             `warning!(call = none, \"...\")`"
        )
    };
    ($($rest:tt)*) => {{
        let (__mx_class, __mx_data, __mx_message) =
            $crate::__mx_condition_parts!(@options $($rest)*);
        (
            $crate::condition::ConditionCall::Inherit,
            __mx_class,
            __mx_data,
            __mx_message,
        )
    }};
}

/// Raise an R error from Rust with `rust_error` class layering.
///
/// Rides the tagged-condition transport that every `#[miniextendr]` function uses.
/// The raised condition has class `c("rust_error", "simpleError", "error", "condition")`.
///
/// An optional `class = "name"` form prepends a custom class for programmatic catching:
/// `c("name", "rust_error", "simpleError", "error", "condition")`. `class` also
/// takes a vector (`class = ["pkg_error_missing_field", "pkg_error"]`, or any
/// [`ConditionClass`] value) so handlers can catch the family or the member.
///
/// The framework layers (`rust_error`, `simpleError`, `error`, `condition`)
/// always follow the user classes; no option leaves them out. Handlers and
/// `inherits()` see the user classes as on an R `errorCondition()`; a test
/// that compares the whole class vector with an R implementation's compares
/// the leading classes instead.
///
/// # Structured `data = ...` payloads
///
/// An optional `data = ...` form (after `class`, before the message) attaches
/// named fields to the condition object, rlang-`abort()`-style. Handlers read
/// them as `e$<name>` instead of parsing the message string:
///
/// ```ignore
/// // Single field:
/// mx::error!(class = "range_error", data = ("value", value), "value {value} out of range");
///
/// // Multiple fields (bracketed list of pairs):
/// mx::error!(
///     class = "validation_error",
///     data = [("value", value), ("min", 0), ("max", 100)],
///     "value {value} out of range"
/// );
///
/// // Keyed builder sugar (bare-ident keys, stringified by the macro):
/// mx::error!(
///     class = "validation_error",
///     data = { value = value, min = 0, max = 100 },
///     "value {value} out of range"
/// );
/// ```
///
/// ```r
/// tryCatch(validate(150L), validation_error = function(e) c(e$value, e$min, e$max))
/// # [1] 150   0 100
/// ```
///
/// Argument order is fixed: `call = none` (optional), then `class = ...`
/// (optional), then `data = ...` (optional), then the format message.
///
/// Field names `message`, `call` and `kind` are the condition's own slots and
/// are rejected (at compile time for literal / bare-identifier names, at
/// runtime otherwise); give the field another name (`rule` instead of `kind`,
/// say). A downstream error type with such a field renames it where the
/// payload is built (`#[serde(rename)]`, a different key).
///
/// **Supported value types**: scalars and `Vec`s of `i32`, `f64`, `bool`, and
/// `String` (plus `&str` / `Vec<&str>`, converted to owned); their NA-aware
/// `Option` / `Vec<Option<_>>` forms (→ R `NA`); the wide-integer ladder (`i64`
/// / `u32`); and the [`RValue::debug`](crate::RValue::debug) escape hatch for
/// any `T: Debug`. The payload is owned — it travels through `panic_any`, so
/// live `SEXP`s cannot ride along; the R objects are materialised on the main
/// thread at the unwind boundary. For nested lists or complex/raw values build an
/// [`RValue`](crate::RValue) directly.
///
/// # Conditions without a call
///
/// A leading `call = none` raises the condition with no call, like R's
/// `stop(..., call. = FALSE)`: `conditionCall(e)` is `NULL` and the error
/// prints as `Error: <message>` instead of `Error in f(...) : <message>`. Use
/// it for a condition about the data rather than about the call. Classes and
/// `data` are unchanged:
///
/// ```ignore
/// mx::error!(call = none, class = "pkg_bad_row", data = { row = row }, "row {row} is malformed");
/// ```
///
/// The same marker exists as [`RError::without_call`] for `Result` errors and
/// as `#[condition(call = none)]` on a `#[derive(RCondition)]` type or
/// variant; see [`ConditionCall`].
///
/// # See also
///
/// - [`crate::warning!`] / [`crate::message!`] / [`crate::condition!`] — the
///   non-error sibling kinds (warning continues execution; message is muffled
///   by `suppressMessages`; condition is silent without a handler).
/// - [`std::panic!`] — escape hatch with the same `rust_error` class layering
///   but no custom-class slot. Use for true bugs / impossible states; reach for
///   `error!` when callers might want to route by class.
/// - [`AsRError`] — wraps `Result<_, E: std::error::Error>` for value-style
///   propagation through Rust code; converts at the boundary.
/// - [`crate::error_value`] — module-level rationale for the tagged-SEXP
///   transport and the `error_in_r` default.
///
/// **Name-collision note.** Because `pub mod error` exists at the crate root,
/// `use miniextendr_api::error` imports the module rather than this macro, and
/// glob imports (`use miniextendr_api::*;`) hit the same shadow. Prefer the
/// collision-free alias [`crate::rust_error!`], which has the identical
/// expansion; otherwise invoke via `miniextendr_api::error!(...)` (fully
/// qualified) or `mx::error!(...)` after `use miniextendr_api as mx;`.
///
/// # Examples
///
/// ```ignore
/// use miniextendr_api as mx;
///
/// #[miniextendr]
/// fn fail() {
///     mx::error!("something went wrong: {}", 42);
/// }
///
/// // With a custom class for tryCatch:
/// #[miniextendr]
/// fn typed_fail(name: &str) {
///     mx::error!(class = "my_error", "missing field: {name}");
/// }
/// ```
///
/// ```r
/// tryCatch(fail(), rust_error = function(e) conditionMessage(e))
/// # [1] "something went wrong: 42"
///
/// tryCatch(typed_fail("x"), my_error = function(e) "caught!")
/// # [1] "caught!"
/// ```
#[macro_export]
macro_rules! error {
    ($($t:tt)*) => {{
        let (__mx_call, __mx_class, __mx_data, __mx_message) =
            $crate::__mx_condition_parts!($($t)*);
        ::std::panic::panic_any($crate::condition::RCondition::Error {
            message: __mx_message,
            class: __mx_class,
            data: __mx_data,
            call: __mx_call,
        })
    }};
}

/// Collision-free alias for [`crate::error!`].
///
/// Identical expansion and grammar (`call = none`, `class = …`, `data = …`,
/// and the plain `format!` forms) — it exists so that `use miniextendr_api::*;` or
/// `use miniextendr_api::rust_error;` gives you a usable macro name. The bare
/// `error!` name is shadowed at the crate root by `pub mod error` (see the
/// name-collision note on [`crate::error!`]), so a glob or direct import
/// resolves to the module, not the macro. `rust_error!` has no such clash.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::rust_error;
///
/// #[miniextendr]
/// fn typed_fail(name: &str) {
///     rust_error!(class = "my_error", "missing field: {name}");
/// }
/// ```
#[macro_export]
macro_rules! rust_error {
    ($($t:tt)*) => { $crate::error!($($t)*) };
}

/// Raise an R warning from Rust with `rust_warning` class layering.
///
/// Rides the tagged-condition transport that every `#[miniextendr]` function uses.
/// Unlike `panic!`, execution continues after `warning!` is caught by a handler.
/// The raised condition has class `c("rust_warning", "simpleWarning", "warning", "condition")`.
///
/// An optional `class = "name"` form prepends a custom class. An optional
/// `data = ...` form (after `class`, before the message) attaches named fields
/// readable as `w$<name>` in handlers — same grammar and supported value types
/// as [`crate::error!`] (see there for details):
///
/// ```ignore
/// warning!(class = "truncation", data = ("dropped", n), "dropped {n} rows");
/// ```
///
/// A leading `call = none` raises the warning without a call, like R's
/// `warning(..., call. = FALSE)`: `conditionCall(w)` is `NULL` and the
/// printed warning has no `In f(...) :` prefix. Use it for a warning about the
/// data rather than about the call (see
/// [conditions without a call](crate::error!#conditions-without-a-call)):
///
/// ```ignore
/// warning!(call = none, class = "pkg_override", data = { row = row }, "row {row} overrides an earlier row");
/// ```
///
/// # See also
///
/// - [`crate::error!`] — fatal sibling; aborts the call instead of continuing.
/// - [`crate::defer_warning!`] / [`crate::defer_warning()`] — queue the warning
///   and still return the value (signalled when the call returns).
/// - [`crate::message!`] / [`crate::condition!`] — softer signal kinds (muffled
///   by `suppressMessages` / silent without handler, respectively).
/// - [`std::panic!`] — escape hatch when "continue after this" is not a sensible
///   semantic.
/// - [`crate::error_value`] — tagged-SEXP transport rationale.
///
/// No name-collision caveat: there is no `pub mod warning`, so
/// `use miniextendr_api::warning;` then `warning!(...)` works directly.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::warning;
///
/// #[miniextendr]
/// fn maybe_warn(x: i32) -> i32 {
///     if x > 100 {
///         warning!("x is large: {x}");
///     }
///     x * 2
/// }
/// ```
///
/// ```r
/// withCallingHandlers(
///   maybe_warn(200L),
///   warning = function(w) { cat("saw:", conditionMessage(w)); invokeRestart("muffleWarning") }
/// )
/// # saw: x is large: 200
/// # [1] 400
/// ```
#[macro_export]
macro_rules! warning {
    ($($t:tt)*) => {{
        let (__mx_call, __mx_class, __mx_data, __mx_message) =
            $crate::__mx_condition_parts!($($t)*);
        ::std::panic::panic_any($crate::condition::RCondition::Warning {
            message: __mx_message,
            class: __mx_class,
            data: __mx_data,
            call: __mx_call,
        })
    }};
}

/// Emit an R message from Rust with `rust_message` class layering.
///
/// Rides the tagged-condition transport that every `#[miniextendr]` function uses.
/// The raised condition has class `c("rust_message", "simpleMessage", "message", "condition")`.
/// Muffled by `suppressMessages()` automatically (standard R restart mechanism).
///
/// An optional `data = ...` form (before the message) attaches named fields
/// readable as `m$<name>` in `withCallingHandlers` — same grammar and
/// supported value types as [`crate::error!`] (see there for details). There
/// is no `class =` form for `message!`, and no `call =` form: an R message
/// carries no call (`conditionCall(m)` is `NULL`), so there is nothing to drop.
///
/// # See also
///
/// - [`crate::warning!`] / [`crate::condition!`] — louder/quieter sibling kinds.
/// - [`crate::error!`] — for fatal failures.
/// - [`std::panic!`] — escape hatch.
/// - [`crate::error_value`] — tagged-SEXP transport rationale.
///
/// No name-collision caveat: there is no `pub mod message`, so
/// `use miniextendr_api::message;` then `message!(...)` works directly.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::message;
///
/// #[miniextendr]
/// fn log_step(step: i32) {
///     message!("step {} complete", step);
/// }
/// ```
///
/// ```r
/// log_step(3L)
/// # step 3 complete
///
/// suppressMessages(log_step(3L))  # no output
/// ```
#[macro_export]
macro_rules! message {
    (call $($rest:tt)*) => {
        ::core::compile_error!(
            "message!: there is no `call` option, an R message carries no call"
        )
    };
    (data = $data:tt, $($arg:tt)*) => {
        ::std::panic::panic_any($crate::condition::RCondition::Message {
            message: ::std::format!($($arg)*),
            class: ::std::vec::Vec::new(),
            data: $crate::__mx_condition_data!($data),
            call: $crate::condition::ConditionCall::Inherit,
        })
    };
    ($($arg:tt)*) => {
        ::std::panic::panic_any($crate::condition::RCondition::Message {
            message: ::std::format!($($arg)*),
            class: ::std::vec::Vec::new(),
            data: ::std::option::Option::None,
            call: $crate::condition::ConditionCall::Inherit,
        })
    };
}

/// Signal a generic R condition from Rust with `rust_condition` class layering.
///
/// Rides the tagged-condition transport that every `#[miniextendr]` function uses.
/// Unlike `error!`, a bare condition is a silent no-op if there is no handler.
/// The raised condition has class `c("rust_condition", "simpleCondition", "condition")`.
///
/// An optional `class = "name"` form prepends a custom class. An optional
/// `data = ...` form (after `class`, before the message) attaches named fields
/// readable as `c$<name>` in handlers — same grammar and supported value types
/// as [`crate::error!`] (see there for details). A leading `call = none`
/// signals the condition without a call (`conditionCall(c)` is `NULL`), like
/// [`crate::error!`]'s.
///
/// # See also
///
/// - [`crate::error!`] / [`crate::warning!`] / [`crate::message!`] — louder
///   condition kinds. Pick `condition!` when "no handler = silent" is the
///   right default (progress events, structured logging hooks).
/// - [`std::panic!`] — escape hatch when the failure cannot be ignored.
/// - [`crate::error_value`] — tagged-SEXP transport rationale.
///
/// **Name-collision note.** Because `pub mod condition` exists at the crate
/// root, `use miniextendr_api::condition` imports the module rather than this
/// macro, and glob imports (`use miniextendr_api::*;`) hit the same shadow.
/// Prefer the collision-free alias [`crate::rust_condition!`], which has the
/// identical expansion; otherwise invoke via `miniextendr_api::condition!(...)`
/// (fully qualified) or `mx::condition!(...)` after `use miniextendr_api as mx;`.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::condition;
///
/// #[miniextendr]
/// fn signal_progress(n: i32) {
///     condition!(class = "my_progress", "processed {n} items");
/// }
/// ```
///
/// ```r
/// withCallingHandlers(
///   signal_progress(42L),
///   my_progress = function(c) cat("progress:", conditionMessage(c), "\n")
/// )
/// # progress: processed 42 items
/// ```
#[macro_export]
macro_rules! condition {
    ($($t:tt)*) => {{
        let (__mx_call, __mx_class, __mx_data, __mx_message) =
            $crate::__mx_condition_parts!($($t)*);
        ::std::panic::panic_any($crate::condition::RCondition::Condition {
            message: __mx_message,
            class: __mx_class,
            data: __mx_data,
            call: __mx_call,
        })
    }};
}

/// Collision-free alias for [`crate::condition!`].
///
/// Identical expansion and grammar (`call = none`, `class = …`, `data = …`,
/// and the plain `format!` forms) — it exists so that `use miniextendr_api::*;` or
/// `use miniextendr_api::rust_condition;` gives you a usable macro name. The
/// bare `condition!` name is shadowed at the crate root by `pub mod condition`
/// (see the name-collision note on [`crate::condition!`]), so a glob or direct
/// import resolves to the module, not the macro. `rust_condition!` has no such
/// clash.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::rust_condition;
///
/// #[miniextendr]
/// fn signal_progress(n: i32) {
///     rust_condition!(class = "my_progress", "processed {n} items");
/// }
/// ```
#[macro_export]
macro_rules! rust_condition {
    ($($t:tt)*) => { $crate::condition!($($t)*) };
}

/// Raise the argument error the generated R wrapper raises for a bad
/// argument, from a `#[miniextendr]` function body.
///
/// `arg_error!(param = "mode", "'mode' should be one of {}", choices)` raises
/// the condition of the wrapper's own argument checks (`match_arg`,
/// `inherits`, `no_na`, type preconditions) and of a failed argument
/// conversion: `kind = "conversion"`, `e$param` set to `param` (the
/// argument's R name), the message as formatted, and the crate's
/// `conversion_error_class` (`[package.metadata.miniextendr]`) before the
/// `rust_error` layering:
///
/// ```r
/// class(e)
/// # [1] "pkg_error_argument" "pkg_error" "rust_error" "simpleError" "error" "condition"
/// ```
///
/// Use it for a check that applies on some paths only, where a
/// `#[miniextendr]` attribute on the parameter would check every call: an
/// argument passed on to another function's body, say, which skips that
/// function's R wrapper and its checks. [`crate::match_arg_param`] matches a
/// raw choice argument with the wrapper's semantics and message and raises
/// through this macro.
///
/// It diverges like [`crate::error!`] (the same `panic_any` transport), and
/// conditions queued with [`crate::defer_warning!`] & co. are signalled first.
/// The call is the one the wrapper's checks report: the wrapper's call as
/// written, or under `#[miniextendr(call = caller)]` its caller's. A leading
/// `call = none` raises it without a call, as for the other condition macros.
/// There is no `class =` or `data =`: the classes are the crate's and the only
/// field is `param`, so the condition is the wrapper's.
///
/// ```ignore
/// use miniextendr_api::{arg_error, miniextendr};
///
/// #[miniextendr]
/// pub fn in_metres(x: f64, unit: &str) -> f64 {
///     match unit {
///         "m" => x,
///         "cm" => x / 100.0,
///         _ => arg_error!(param = "unit", "'unit' should be one of \"m\", \"cm\""),
///     }
/// }
/// ```
///
/// # How the crate's classes get there
///
/// A `macro_rules!` macro cannot read `[package.metadata.miniextendr]`, so the
/// tagged condition value carries the marker class
/// [`CONVERSION_ERROR_CLASS_MARKER`]
/// and the generated wrapper's `.miniextendr_raise_condition` replaces it with
/// `.miniextendr_conversion_error_class`, the binding its own checks use. Across
/// a trait-ABI call the consumer package's wrapper raises the condition, so
/// its crate's classes apply. A raising guard (an ALTREP `RUnwind` callback,
/// `with_r_unwind_protect_or_raise`) has no generated R: it resolves the
/// marker in Rust against the classes `miniextendr_init!` registered (on
/// wasm32, the copy in the `wasm_registry.rs` snapshot), then raises the same
/// condition, `kind = "conversion"` included (#1768). A connection callback
/// returns its fallback, as for any panic.
///
/// The generated `no_na` check after a conversion builds the same condition
/// with [`crate::error_value::arg_check_condition_value`], which takes the
/// classes the macro read at expansion time.
///
/// # Grammar
///
/// `arg_error!([call = none,] param = <expr>, <format string>, <args>...)`.
/// `param` is anything `Into<String>`. A `call` anywhere but first, a repeated
/// one, a missing `param`, and `class =` / `data =` are compile errors:
///
/// ```compile_fail
/// # use miniextendr_api as mx;
/// # fn f() {
/// mx::arg_error!(param = "mode", call = none, "bad");
/// # }
/// ```
///
/// ```compile_fail
/// # use miniextendr_api as mx;
/// # fn f() {
/// mx::arg_error!("'mode' is bad");
/// # }
/// ```
///
/// ```compile_fail
/// # use miniextendr_api as mx;
/// # fn f() {
/// mx::arg_error!(param = "mode", class = "pkg_error", "bad");
/// # }
/// ```
///
/// ```
/// # use miniextendr_api as mx;
/// # fn f(mode: &str) -> ! {
/// mx::arg_error!(call = none, param = "mode", "'mode' is {mode:?}")
/// # }
/// ```
#[macro_export]
macro_rules! arg_error {
    // Internal: after the optional `call = none`.
    (@param $call:expr; call $($rest:tt)*) => {
        ::core::compile_error!("arg_error!: `call = none` is given twice")
    };
    (@param $call:expr; param = $param:expr, call $($rest:tt)*) => {
        ::core::compile_error!(
            "arg_error!: `call = none` comes first, before `param`: \
             `arg_error!(call = none, param = \"mode\", \"...\")`"
        )
    };
    (@param $call:expr; param = $param:expr, class $($rest:tt)*) => {
        $crate::arg_error!(@no_class_or_data)
    };
    (@param $call:expr; param = $param:expr, data $($rest:tt)*) => {
        $crate::arg_error!(@no_class_or_data)
    };
    (@param $call:expr; param = $param:expr, $($arg:tt)+) => {
        ::std::panic::panic_any($crate::condition::RCondition::arg_error(
            $param,
            ::std::format!($($arg)+),
            $call,
        ))
    };
    (@param $call:expr; $($rest:tt)*) => {
        ::core::compile_error!(
            "arg_error!: name the argument, then give the message: \
             `arg_error!(param = \"mode\", \"'mode' should be one of ...\")`"
        )
    };
    (@no_class_or_data) => {
        ::core::compile_error!(
            "arg_error!: takes no `class` or `data`: its classes are the crate's \
             `conversion_error_class` and its only field is `param`, as for the wrapper's \
             own argument errors; use `error!` for a classed error"
        )
    };
    // Entry: the optional `call = none`, then `param = ...` and the message.
    (call = none, $($rest:tt)*) => {
        $crate::arg_error!(@param $crate::condition::ConditionCall::None; $($rest)*)
    };
    (call $($rest:tt)*) => {
        ::core::compile_error!(
            "arg_error!: the only call option is `call = none` (no call, like R's \
             `call. = FALSE`), and it comes first, followed by a comma: \
             `arg_error!(call = none, param = \"mode\", \"...\")`"
        )
    };
    ($($rest:tt)*) => {
        $crate::arg_error!(@param $crate::condition::ConditionCall::Inherit; $($rest)*)
    };
}

// endregion

// region: Class vectors and reserved field names

/// What the condition macros' `class = …` (and [`RError::class`]) accept: one
/// class or several, as `&str` / `String` / arrays / `Vec` / slices.
///
/// The classes are prepended in order to the `rust_*` layering, so put the most
/// specific first: `class = ["pkg_error_missing_field", "pkg_error"]` renders
/// as `c("pkg_error_missing_field", "pkg_error", "rust_error", …)` and both
/// `tryCatch(pkg_error_missing_field = …)` and `tryCatch(pkg_error = …)` match.
pub trait ConditionClass {
    /// The class vector, most specific first.
    fn into_condition_class(self) -> Vec<String>;
}

impl ConditionClass for &str {
    fn into_condition_class(self) -> Vec<String> {
        vec![self.to_string()]
    }
}
impl ConditionClass for String {
    fn into_condition_class(self) -> Vec<String> {
        vec![self]
    }
}
impl ConditionClass for &String {
    fn into_condition_class(self) -> Vec<String> {
        vec![self.clone()]
    }
}
impl<const N: usize> ConditionClass for [&str; N] {
    fn into_condition_class(self) -> Vec<String> {
        self.iter().map(|s| s.to_string()).collect()
    }
}
impl<const N: usize> ConditionClass for [String; N] {
    fn into_condition_class(self) -> Vec<String> {
        self.into_iter().collect()
    }
}
impl ConditionClass for Vec<&str> {
    fn into_condition_class(self) -> Vec<String> {
        self.into_iter().map(str::to_string).collect()
    }
}
impl ConditionClass for Vec<String> {
    fn into_condition_class(self) -> Vec<String> {
        self
    }
}
impl ConditionClass for &[&str] {
    fn into_condition_class(self) -> Vec<String> {
        self.iter().map(|s| s.to_string()).collect()
    }
}
impl ConditionClass for &[String] {
    fn into_condition_class(self) -> Vec<String> {
        self.to_vec()
    }
}

/// Condition-data field names that would overwrite the condition object's own
/// slots when the R helper splices `data` in (`utils::modifyList` over
/// `list(message, call, kind)`).
pub const RESERVED_CONDITION_FIELDS: &[&str] = &["message", "call", "kind"];

/// `true` for [`RESERVED_CONDITION_FIELDS`]. `const` so the macros can reject a
/// literal field name at compile time:
///
/// ```compile_fail
/// # use miniextendr_api as mx;
/// # fn f() {
/// mx::rust_error!(data = ("kind", 1), "boom");
/// # }
/// ```
///
/// Any other name is fine, so the fix for a clash is a rename at the source:
///
/// ```
/// # use miniextendr_api as mx;
/// # fn f() {
/// mx::rust_error!(data = ("rule", 1), "boom");
/// # }
/// ```
pub const fn is_reserved_condition_field(name: &str) -> bool {
    str_eq(name, "message")
        || str_eq(name, "call")
        || str_eq(name, "kind")
        || str_eq(name, PARENT_FIELD)
}

/// `a == b`, usable in `const fn`.
const fn str_eq(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}

/// The `data` field under which [`RCondition::with_parent`] sends a cause to
/// R; the raise helper turns it into the condition object `e$parent`.
const PARENT_FIELD: &str = ".mx_parent";

/// Runtime half of the reserved-name check, for computed field names (the
/// macros' `expr` path and [`RConditionData::data`] on user types). Panics
/// when `name` is reserved; the panic becomes a plain `rust_error` in R,
/// replacing the former silent overwrite.
#[track_caller]
pub fn check_condition_field_name(name: String) -> String {
    if is_reserved_condition_field(&name) {
        panic!(
            "condition data field `{name}` is reserved: `message`, `call` and `kind` are the \
             condition's own slots, and `.mx_parent` carries its parent; rename the field."
        );
    }
    name
}

/// Apply [`check_condition_field_name`] to every field of a payload.
#[track_caller]
pub fn check_condition_data(data: Option<ConditionData>) -> Option<ConditionData> {
    data.map(|fields| {
        fields
            .into_iter()
            .map(|(name, value)| (check_condition_field_name(name), value))
            .collect()
    })
}

// endregion

// region: Classed `Result` errors — RConditionData + RError

/// Give a `Result<T, E>` error type an R class vector and structured fields.
///
/// Every `#[miniextendr]` function or method returning `Result<T, E>` raises
/// `Err(e)` as an R error. Implement this trait for `E` (or return
/// [`RError`], which implements it) and the `Err` arm raises
/// `c(<class()…>, "rust_error", "simpleError", "error", "condition")` with
/// every `data()` field readable as `e$<name>`, so a thiserror-style error enum
/// can keep `?` composition *and* give R handlers something to dispatch on.
///
/// Without an impl, the `Err` arm falls back in two steps. Under the `serde`
/// feature an `E: serde::Serialize + Display` is classed from its serde shape
/// (see `serde_err_parts`: variant → member class, fields → data, `Display`
/// → message). Anything else is a bare `rust_error` whose message is
/// `format!("{e:?}")`.
///
/// ```ignore
/// use miniextendr_api::condition::{ConditionData, RConditionData};
///
/// #[derive(Debug, thiserror::Error)]
/// pub enum PkgError {
///     #[error("column `{column}` is missing")]
///     MissingColumn { column: String },
///     #[error("{value} exceeds {max}")]
///     TooLarge { value: f64, max: f64 },
/// }
///
/// impl RConditionData for PkgError {
///     fn message(&self) -> String { self.to_string() }
///     fn class(&self) -> Vec<String> {
///         let member = match self {
///             PkgError::MissingColumn { .. } => "pkg_error_missing_column",
///             PkgError::TooLarge { .. } => "pkg_error_too_large",
///         };
///         vec![member.into(), "pkg_error".into()]
///     }
///     fn data(&self) -> Option<ConditionData> {
///         Some(match self {
///             PkgError::MissingColumn { column } => vec![("column".into(), column.clone().into())],
///             PkgError::TooLarge { value, max } => vec![("value".into(), (*value).into()), ("max".into(), (*max).into())],
///         })
///     }
/// }
///
/// #[miniextendr]
/// pub fn check(value: f64) -> Result<f64, PkgError> { /* uses `?` freely */ }
/// ```
///
/// ```r
/// tryCatch(check(1e9), pkg_error_too_large = function(e) e$max)   # member
/// tryCatch(check(1e9), pkg_error = function(e) conditionMessage(e)) # family
/// ```
///
/// Detection is by trait, not by attribute: the generated `Err` arm probes
/// `E: RConditionData` first and falls back to the `Debug` rendering
/// otherwise, so existing `Result<T, String>` / `Result<T, MyDebugError>`
/// functions are unchanged. The condition's `kind` stays `"result_err"`.
///
/// The same impl classes argument-conversion failures: when a parameter
/// type's `TryFromSexp::Error` implements this trait, a failed conversion
/// raises its classes and fields with `kind = "conversion"`, the message
/// `'<p>' must be <expected>: <message()>` (or `invalid '<p>' argument:
/// <message()>` when the argument's type has no R-facing expectation), the
/// parameter's R name as `e$param` and the Rust type as `e$rust_type`
/// (unless `data()` has a field of the same name, which then wins). An error
/// type without the impl renders with `Display` and keeps the plain
/// `rust_error` class vector.
///
/// `data()` field names must not be `message`, `call` or `kind` (see
/// [`RESERVED_CONDITION_FIELDS`]); a reserved name raises a plain
/// `rust_error` explaining the clash, so rename such a field at the source.
///
/// [`call()`](RConditionData::call) returning [`ConditionCall::None`] drops
/// the call (`conditionCall(e)` is `NULL`), for errors and warnings about the
/// data rather than the call.
pub trait RConditionData {
    /// The condition message (`conditionMessage(e)`).
    fn message(&self) -> String;
    /// User classes, most specific first; empty for none.
    fn class(&self) -> Vec<String> {
        Vec::new()
    }
    /// Structured fields spliced into the condition object (`e$<name>`).
    fn data(&self) -> Option<ConditionData> {
        None
    }
    /// Whether the condition reports a call. [`ConditionCall::None`] makes
    /// `conditionCall()` `NULL`, like R's `call. = FALSE`, for a condition
    /// about the data rather than the call; the default keeps the call the
    /// transport captured. Honoured by `defer_*` and by `Result` returns;
    /// an argument-conversion error always keeps the call (see
    /// [`conversion_err_parts`]). `#[condition(call = none)]` on a derived
    /// type or variant overrides this.
    fn call(&self) -> ConditionCall {
        ConditionCall::Inherit
    }
    /// The whole message when this error refuses an argument, or `None`.
    ///
    /// When the error comes out of a parameter's conversion (the type's
    /// `TryFromSexp`, or its [`check_sexp`](crate::TryFromSexpElement::check_sexp)
    /// under `Option<T>` / `Vec<T>` / `Vec<Option<T>>`), a `Some` is the
    /// condition's message as given, in place of
    /// `'<p>' must be <expected>: <message()>`. The classes, `kind`,
    /// `e$param`, `e$rust_type` and the call are unchanged. This is how a type
    /// words a refusal once for every parameter of that type, as
    /// `#[miniextendr(inherits(..., message = ...))]` does for one parameter:
    /// both use the author's message as given and name the argument in
    /// `e$param`. Ignored everywhere else (`Result` returns, `defer_*`), and
    /// inside an `Either`, whose message names both arms. For a refused
    /// element of a list of objects (`Vec<T>` of a newtype over `List`), it
    /// is that element's reason, before the element's position, in the
    /// prefixed batched message: an element is not the whole argument.
    fn argument_message(&self) -> Option<String> {
        None
    }
    /// Internal: whether a `.mx_parent` field among
    /// [`data()`](RConditionData::data) is a cause
    /// [`RCondition::with_parent`] attached, which the generated `Err` arms
    /// pass on as `e$parent`. Every other reserved name is refused
    /// ([`check_condition_data`]). True for an [`RCondition`](enum@RCondition)
    /// and for an [`RError`] converted from one.
    #[doc(hidden)]
    fn __mx_keeps_parent(&self) -> bool {
        false
    }
}

/// [`check_condition_data`] for an `Err` arm's fields, passing a
/// `.mx_parent` field when `keeps_parent`
/// ([`RConditionData::__mx_keeps_parent`]).
#[track_caller]
fn check_err_fields(data: Option<ConditionData>, keeps_parent: bool) -> Option<ConditionData> {
    data.map(|fields| {
        fields
            .into_iter()
            .map(|(name, value)| (check_err_field(name, keeps_parent), value))
            .collect()
    })
}

/// [`check_condition_field_name`], passing `.mx_parent` when `keeps_parent`.
#[track_caller]
fn check_err_field(name: String, keeps_parent: bool) -> String {
    if keeps_parent && name == PARENT_FIELD {
        name
    } else {
        check_condition_field_name(name)
    }
}

/// `#[derive(RCondition)]`: [`RConditionData`] generated from a type's shape
/// (`#[condition(class = "…")]`, `#[condition(message = "…")]`, the
/// `rename` / `skip` / `debug` field options) and `From<T>` for
/// [`RCondition`](enum@RCondition) with the declared kind
/// (`#[condition(kind = …)]`, default `error`).
#[doc(inline)]
pub use miniextendr_macros::RCondition;

/// A ready-made classed error value for `Result<T, RError>` returns.
///
/// Carries a message, an optional class vector, structured fields and the
/// call choice, and implements [`RConditionData`]. Any
/// `std::error::Error` converts into it with `?` / [`From`] (the message keeps
/// the `caused by:` chain, like [`AsRError`]), after which the builder methods
/// add the R-facing parts ([`RError::class`], [`RError::data`],
/// [`RError::without_call`]):
///
/// ```ignore
/// use miniextendr_api::condition::RError;
///
/// #[miniextendr]
/// pub fn parse_port(s: &str) -> Result<i32, RError> {
///     let port: i32 = s
///         .parse()
///         .map_err(|e| RError::from(e).class(["pkg_bad_port", "pkg_error"]).data("input", s))?;
///     Ok(port)
/// }
/// ```
///
/// ```r
/// e <- tryCatch(parse_port("x"), pkg_bad_port = function(e) e)
/// e$input          # "x"
/// class(e)[1:2]    # "pkg_bad_port" "pkg_error"
/// ```
///
/// `RError` deliberately does **not** implement `std::error::Error`: that
/// keeps the blanket `From<E: Error>` coherent. It does implement `Display`
/// (the message), so it also works with `#[miniextendr(unwrap_in_r)]`.
#[derive(Debug, Clone)]
pub struct RError {
    message: String,
    class: Vec<String>,
    data: ConditionData,
    call: ConditionCall,
    argument_message: Option<String>,
    /// Converted from an [`RCondition`](enum@RCondition): a `.mx_parent`
    /// field is the cause it carried ([`RConditionData::__mx_keeps_parent`]).
    keeps_parent: bool,
}

impl RError {
    /// A classless error with `message`.
    pub fn new(message: impl Into<String>) -> Self {
        RError {
            message: message.into(),
            class: Vec::new(),
            data: Vec::new(),
            call: ConditionCall::Inherit,
            argument_message: None,
            keeps_parent: false,
        }
    }

    /// Raise the condition without a call: `conditionCall(e)` is `NULL` and
    /// the printed condition has no `In f(...) :` prefix, like R's
    /// `stop(..., call. = FALSE)`. For an error or warning about the data, not
    /// the call. Applies to `Result` returns and the `defer_*` functions; see
    /// [`ConditionCall`].
    pub fn without_call(mut self) -> Self {
        self.call = ConditionCall::None;
        self
    }

    /// Append user classes (most specific first). Accepts one string or a
    /// vector, see [`ConditionClass`].
    pub fn class(mut self, class: impl ConditionClass) -> Self {
        self.class.extend(class.into_condition_class());
        self
    }

    /// Attach a structured field readable as `e$<name>`. Any value with an
    /// [`RValue`](crate::RValue) `From` impl works. The name must not be
    /// `message`, `call` or `kind`; see [`RESERVED_CONDITION_FIELDS`].
    pub fn data(mut self, name: impl Into<String>, value: impl Into<crate::RValue>) -> Self {
        self.data.push((name.into(), value.into()));
        self
    }

    /// The whole message of the argument error when this error refuses an
    /// argument, used as given instead of `'<p>' must be <expected>:
    /// <message>`; see [`RConditionData::argument_message`]. `message` stays
    /// the message everywhere else.
    ///
    /// ```ignore
    /// fn model_object(x: SEXP) -> Result<(), RError> {
    ///     if x.inherits_class(c"data.frame") {
    ///         return Err(RError::new("got a data frame")
    ///             .argument_message("use model_from_df() for a data frame"));
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub fn argument_message(mut self, message: impl Into<String>) -> Self {
        self.argument_message = Some(message.into());
        self
    }

    /// The message.
    pub fn message_str(&self) -> &str {
        &self.message
    }

    /// The whole argument-error message, if one was given
    /// ([`RError::argument_message`]).
    pub fn argument_message_str(&self) -> Option<&str> {
        self.argument_message.as_deref()
    }

    /// The user classes, most specific first.
    pub fn classes(&self) -> &[String] {
        &self.class
    }

    /// The structured fields as attached (unprefixed).
    pub fn fields(&self) -> &ConditionData {
        &self.data
    }
}

impl<E: std::error::Error> From<E> for RError {
    /// Message = the error's `Display` plus its `source()` chain, one
    /// `caused by:` line per link (same rendering as [`AsRError`]).
    fn from(err: E) -> Self {
        let mut message = err.to_string();
        let mut current: &dyn std::error::Error = &err;
        while let Some(source) = current.source() {
            message.push_str("\n  caused by: ");
            message.push_str(&source.to_string());
            current = source;
        }
        RError::new(message)
    }
}

impl std::fmt::Display for RError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl RConditionData for RError {
    fn message(&self) -> String {
        self.message.clone()
    }
    fn class(&self) -> Vec<String> {
        self.class.clone()
    }
    fn data(&self) -> Option<ConditionData> {
        (!self.data.is_empty()).then(|| self.data.clone())
    }
    fn call(&self) -> ConditionCall {
        self.call
    }
    fn argument_message(&self) -> Option<String> {
        self.argument_message.clone()
    }
    fn __mx_keeps_parent(&self) -> bool {
        self.keeps_parent
    }
}

/// An [`RCondition`](enum@RCondition) returned as an `Err` (#2217): from a
/// `#[miniextendr]` function or method returning `Result<T, RCondition>`.
/// Its message, classes, fields and call choice reach R as given. Like any `Err`, it is an
/// R error whatever its variant: the kind is read only where the condition is
/// raised or deferred ([`RCondition::raise`], [`RCondition::defer`]). A cause
/// attached with [`RCondition::with_parent`], as a caught refusal of a
/// restored object carries one, stays its `e$parent`, also through the
/// [`SexpError`](crate::from_r::SexpError) it converts into.
///
/// It is how a body hands on a condition [`catch_condition`] caught, or one
/// built by [`serde_condition!`](crate::serde_condition):
///
/// ```ignore
/// #[miniextendr]
/// pub fn load(path: &str) -> Result<Model, RCondition> {
///     let config = catch_condition(|| read_config(path))?;
///     Ok(Model::new(config))
/// }
/// ```
impl RConditionData for RCondition {
    fn message(&self) -> String {
        RCondition::message(self).to_owned()
    }
    fn class(&self) -> Vec<String> {
        self.class_and_data().0.to_vec()
    }
    fn data(&self) -> Option<ConditionData> {
        self.class_and_data().1.clone()
    }
    fn call(&self) -> ConditionCall {
        RCondition::call(self)
    }
    /// A `.mx_parent` field is the cause [`RCondition::with_parent`]
    /// attached (a refusal [`catch_condition`] caught carries one).
    fn __mx_keeps_parent(&self) -> bool {
        true
    }
}

/// Any classed error (an [`RError`], a `#[derive(RCondition)]` type)
/// converts into a [`SexpError::Condition`](crate::from_r::SexpError::Condition)
/// with its message, classes, fields and call choice, so a newtype's
/// [`check_sexp`](crate::TryFromSexpElement::check_sexp) can refuse with it
/// and a conversion whose error converts into `SexpError` keeps its classes.
impl<E: RConditionData> From<E> for crate::from_r::SexpError {
    fn from(e: E) -> Self {
        crate::from_r::SexpError::Condition(RError {
            message: e.message(),
            class: e.class(),
            data: e.data().unwrap_or_default(),
            call: e.call(),
            argument_message: e.argument_message(),
            keeps_parent: e.__mx_keeps_parent(),
        })
    }
}

/// An argument error as a value: the parameter's R name and the message.
/// [`ArgError::raise`] raises it as the generated wrapper's own argument
/// error, through [`crate::arg_error!`].
///
/// [`crate::match_arg_param`] returns it for a choice argument the body
/// matches itself. Raise it where the argument is bad, or handle it:
///
/// ```ignore
/// use miniextendr_api::{SEXP, match_arg_param, miniextendr};
///
/// #[miniextendr]
/// pub fn prepare(data: SEXP, mode: SEXP) -> String {
///     if is_data_frame(data) {
///         let mode: Mode = match_arg_param(mode, "mode").unwrap_or_else(|e| e.raise());
///         // ...
///     }
///     // ...
/// }
/// ```
///
/// It does not implement [`RConditionData`]: returned as a body's `Err` it
/// would take the `Result` path (`kind = "result_err"`), not the argument
/// error's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgError {
    param: String,
    message: String,
}

impl ArgError {
    /// The argument error of parameter `param` (its R name) with `message`,
    /// used as given.
    pub fn new(param: impl Into<String>, message: impl Into<String>) -> Self {
        ArgError {
            param: param.into(),
            message: message.into(),
        }
    }

    /// The parameter's R name (`e$param`).
    pub fn param(&self) -> &str {
        &self.param
    }

    /// The message (`conditionMessage(e)`).
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Raise it: `arg_error!(param = <param>, "{}", <message>)`. The
    /// condition is the one the generated wrapper raises for that parameter;
    /// see [`crate::arg_error!`].
    pub fn raise(self) -> ! {
        std::panic::panic_any(RCondition::arg_error(
            self.param,
            self.message,
            ConditionCall::Inherit,
        ))
    }
}

impl std::fmt::Display for ArgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ArgError {}

/// The parts of a condition the generated `Err` arms hand to
/// [`crate::error_value::result_err_condition_value`] /
/// [`crate::error_value::conversion_condition_value`], and that
/// [`RCondition::into_parts`] splits a payload into: message, user classes,
/// `data` fields and the [`ConditionCall`] choice.
#[doc(hidden)]
#[derive(Debug)]
pub struct ErrParts {
    pub message: String,
    pub class: Vec<String>,
    pub data: Option<ConditionData>,
    pub call: ConditionCall,
}

/// Autoref-specialisation probe, first stage, preferred arm: `E: RConditionData`.
///
/// Under the `serde` feature a declared-shape stage runs before this one: a
/// shape that `serde_error_shape!` declared for `E` wins over the impl (see
/// `ErrShapeDeclared`).
///
/// The generated code calls `(&e).__mx_classed_parts()` with both first-stage
/// traits in scope. Method resolution tries the receiver type `&E` by value
/// first, which only this impl (on `E`, taking `&self`) satisfies; the
/// [`ErrPartsUnclassed`] arm on `&E` needs one more auto-ref and is only
/// reached when `E` does not implement [`RConditionData`]. It yields `None`,
/// and the second stage ([`ErrPartsSerde`] / [`ErrPartsDebug`]) decides.
///
/// Two two-way probes rather than one three-way probe: the `Serialize` and
/// `Debug` bounds both hold for `&E` whenever they hold for `E`, so a single
/// method name with three impls at increasing reference depth is ambiguous at
/// the first step. Each stage has exactly one impl on `E` and one on `&E`.
#[doc(hidden)]
pub trait ErrPartsClassed {
    fn __mx_classed_parts(&self) -> Option<ErrParts>;
}

impl<E: RConditionData> ErrPartsClassed for E {
    #[track_caller]
    fn __mx_classed_parts(&self) -> Option<ErrParts> {
        Some(ErrParts {
            message: self.message(),
            class: self.class(),
            data: check_err_fields(self.data(), self.__mx_keeps_parent()),
            call: self.call(),
        })
    }
}

/// First stage, fallback arm: no [`RConditionData`] impl.
#[doc(hidden)]
pub trait ErrPartsUnclassed {
    fn __mx_classed_parts(&self) -> Option<ErrParts>;
}

impl<E> ErrPartsUnclassed for &E {
    fn __mx_classed_parts(&self) -> Option<ErrParts> {
        None
    }
}

/// Second stage, preferred arm: `E: Serialize + Display` under the `serde`
/// feature, classed via [`serde_err_parts`] with the defaults (`kind` tag,
/// the `<crate>_error` prefix the macro passes in, no shapes).
/// `#[miniextendr(serde_error(..))]` bypasses the probe with explicit options.
///
/// The trait exists without the feature so the generated `use` resolves; it
/// then has no impl and every type takes the [`ErrPartsDebug`] arm.
#[doc(hidden)]
pub trait ErrPartsSerde {
    fn __mx_fallback_parts(&self, prefix: &str) -> ErrParts;
}

#[cfg(feature = "serde")]
impl<E: ::serde::Serialize + std::fmt::Display> ErrPartsSerde for E {
    #[track_caller]
    fn __mx_fallback_parts(&self, prefix: &str) -> ErrParts {
        serde_err_parts(self, prefix, &[])
    }
}

/// Second stage, fallback arm: any `E: Debug` (the historical
/// `format!("{e:?}")` rendering, no class, no data).
#[doc(hidden)]
pub trait ErrPartsDebug {
    fn __mx_fallback_parts(&self, prefix: &str) -> ErrParts;
}

impl<E: std::fmt::Debug> ErrPartsDebug for &E {
    fn __mx_fallback_parts(&self, _prefix: &str) -> ErrParts {
        ErrParts {
            message: format!("{self:?}"),
            class: Vec::new(),
            data: None,
            call: ConditionCall::Inherit,
        }
    }
}

/// Internal: the `Err(e)` probe used by generated wrappers. Not public API.
///
/// `$prefix` is the family class for the serde arms (`<crate>_error`, computed
/// by the macro from `CARGO_CRATE_NAME` at expansion time). `$marker` is the
/// calling crate's marker type (`crate::__MiniextendrInitMarker`), under which
/// [`serde_error_shape!`](crate::serde_error_shape) declarations are looked up
/// first, under the `serde` feature: a declared shape wins over the stages
/// below (the `ErrShapeDeclared` probe).
#[doc(hidden)]
#[macro_export]
macro_rules! __mx_result_err_parts {
    ($e:expr, $prefix:expr, $marker:ty) => {{
        #[allow(unused_imports)]
        use $crate::condition::{
            ErrPartsClassed as _, ErrPartsDebug as _, ErrPartsSerde as _, ErrPartsUnclassed as _,
        };
        match &$e {
            __mx_e => match $crate::__mx_declared_err_parts!(__mx_e, $prefix, $marker) {
                ::core::option::Option::Some(parts) => parts,
                ::core::option::Option::None => match __mx_e.__mx_classed_parts() {
                    ::core::option::Option::Some(parts) => parts,
                    ::core::option::Option::None => __mx_e.__mx_fallback_parts($prefix),
                },
            },
        }
    }};
}

// endregion

// region: Classed argument-conversion errors

/// Conversion probe, preferred arm: `E: RConditionData`.
///
/// When an argument fails its `TryFromSexp` conversion, the generated wrapper
/// calls `(&e).__mx_conversion_parts(expected_known)` with the three
/// conversion-probe traits in scope (see [`crate::__mx_conversion_err_parts!`]).
/// This impl, on `E` taking `&self`, matches the receiver `&E` by value, so a
/// `TryFromSexp::Error` that implements [`RConditionData`] contributes its
/// class vector, message and data. [`ConversionErrBuiltin`] matches at the
/// same step for the built-in conversion errors, which never implement
/// [`RConditionData`], so the two never compete. [`ConversionErrDisplay`]
/// needs one more auto-ref and is only reached otherwise.
///
/// Unlike the `Result` probe ([`crate::__mx_result_err_parts!`]) there is no
/// serde stage and the fallback is `Display`, not `Debug`: the generated code
/// has always rendered conversion errors with `Display`, which every built-in
/// error type (`SexpError`, `MatchArgError`, ...) implements.
#[doc(hidden)]
pub trait ConversionErrClassed {
    fn __mx_conversion_parts(&self, expected_known: bool) -> ErrParts;
}

impl<E: RConditionData> ConversionErrClassed for E {
    #[track_caller]
    fn __mx_conversion_parts(&self, _expected_known: bool) -> ErrParts {
        ErrParts {
            message: self.message(),
            class: self.class(),
            data: check_err_fields(self.data(), self.__mx_keeps_parent()),
            call: self.call(),
        }
    }
}

/// Conversion probe, built-in arm: the framework's own conversion errors
/// ([`SexpError`](crate::from_r::SexpError) and the
/// [`SexpTypeError`](crate::from_r::SexpTypeError) /
/// [`SexpClassError`](crate::from_r::SexpClassError) /
/// [`SexpLengthError`](crate::from_r::SexpLengthError) /
/// [`SexpNaError`](crate::from_r::SexpNaError) it wraps, and
/// [`MatchArgError`](crate::match_arg::MatchArgError)), reworded for R
/// users (#1591).
///
/// Their `Display` text is written for the package author: SEXPTYPE names
/// (`expected INTSXP, got STRSXP`) and a variant prefix (`invalid value: ...`).
/// The condition message says instead what the argument should have been,
/// with the reason from `SexpError::r_reason` (`got character`,
/// `got length 2`, `NA is not allowed`, the invalid value's own text, ...)
/// or `MatchArgError::r_reason` (`got "zzz"`).
///
/// `expected_known` is set by the macro when it knows the argument's R-facing
/// expectation and puts it in the message prefix (`'x' must be a single
/// integer`); the reason then does not repeat it. The type names are those of
/// [`sexptype_name`](crate::typed_list::sexptype_name) (`numeric` for a
/// double, `list`, `NULL`, ...).
#[doc(hidden)]
pub trait ConversionErrBuiltin {
    fn __mx_conversion_parts(&self, expected_known: bool) -> ErrParts;
}

/// The R-facing reason for a built-in conversion error, see
/// [`ConversionErrBuiltin`].
fn builtin_reason_parts(message: String) -> ErrParts {
    ErrParts {
        message,
        class: Vec::new(),
        data: None,
        call: ConditionCall::Inherit,
    }
}

impl ConversionErrBuiltin for crate::from_r::SexpTypeError {
    fn __mx_conversion_parts(&self, expected_known: bool) -> ErrParts {
        builtin_reason_parts(self.r_reason(expected_known))
    }
}

impl ConversionErrBuiltin for crate::from_r::SexpClassError {
    fn __mx_conversion_parts(&self, expected_known: bool) -> ErrParts {
        builtin_reason_parts(self.r_reason(expected_known))
    }
}

impl ConversionErrBuiltin for crate::from_r::SexpLengthError {
    fn __mx_conversion_parts(&self, expected_known: bool) -> ErrParts {
        builtin_reason_parts(self.r_reason(expected_known))
    }
}

impl ConversionErrBuiltin for crate::from_r::SexpNaError {
    fn __mx_conversion_parts(&self, _expected_known: bool) -> ErrParts {
        builtin_reason_parts("NA is not allowed".to_string())
    }
}

/// A `SexpError` also carries the classes and fields of the check refusals
/// ([`SexpError::Condition`](crate::from_r::SexpError::Condition)) whose
/// reasons its message reports, directly or from an `Either` arm.
impl ConversionErrBuiltin for crate::from_r::SexpError {
    fn __mx_conversion_parts(&self, expected_known: bool) -> ErrParts {
        let mut parts = builtin_reason_parts(self.r_reason(expected_known));
        let mut data = ConditionData::new();
        for refusal in self.reported_conditions() {
            for class in refusal.classes() {
                if !parts.class.contains(class) {
                    parts.class.push(class.clone());
                }
            }
            for (name, value) in refusal.fields() {
                if !data.iter().any(|(seen, _)| seen == name) {
                    data.push((
                        check_err_field(name.clone(), refusal.keeps_parent),
                        value.clone(),
                    ));
                }
            }
        }
        parts.data = (!data.is_empty()).then_some(data);
        parts
    }
}

impl ConversionErrBuiltin for crate::match_arg::MatchArgError {
    fn __mx_conversion_parts(&self, expected_known: bool) -> ErrParts {
        builtin_reason_parts(self.r_reason(expected_known))
    }
}

/// Conversion probe, fallback arm: any `E: Display`. The message is the
/// error's `Display` text, with no class and no data, so the condition keeps
/// the plain `rust_error` class vector.
#[doc(hidden)]
pub trait ConversionErrDisplay {
    fn __mx_conversion_parts(&self, expected_known: bool) -> ErrParts;
}

impl<E: std::fmt::Display> ConversionErrDisplay for &E {
    fn __mx_conversion_parts(&self, _expected_known: bool) -> ErrParts {
        ErrParts {
            message: self.to_string(),
            class: Vec::new(),
            data: None,
            call: ConditionCall::Inherit,
        }
    }
}

/// Internal: the argument-conversion `Err(e)` probe used by generated
/// wrappers. Not public API. Yields the error's own [`ErrParts`];
/// [`crate::error_value::conversion_condition_value`] adds the parameter
/// context.
///
/// The second argument is `expected_known` (see [`ConversionErrBuiltin`]):
/// `true` when the message prefix already states what the argument should
/// be. It defaults to `false`.
#[doc(hidden)]
#[macro_export]
macro_rules! __mx_conversion_err_parts {
    ($e:expr) => {
        $crate::__mx_conversion_err_parts!($e, false)
    };
    ($e:expr, $expected_known:expr) => {{
        #[allow(unused_imports)]
        use $crate::condition::{
            ConversionErrBuiltin as _, ConversionErrClassed as _, ConversionErrDisplay as _,
        };
        match &$e {
            __mx_e => __mx_e.__mx_conversion_parts($expected_known),
        }
    }};
}

/// Expectation probe, built-in arm: an error that knows what the value should
/// have been. A `match_arg` choice error (`MatchArgError`, or a `SexpError`
/// carrying one, as `#[derive(MatchArg)]`'s `TryFromSexp` returns) names its
/// choices: `one of "fast", "slow"`; a class error names the class
/// (`a data frame`); an `Either` whose arms both refused the kind of value
/// names both (`one of "fast", "slow", or numeric`).
#[doc(hidden)]
pub trait ConversionExpectBuiltin {
    fn __mx_conversion_expectation(&self) -> Option<String>;
}

impl ConversionExpectBuiltin for crate::from_r::SexpError {
    fn __mx_conversion_expectation(&self) -> Option<String> {
        self.r_expectation()
    }
}

impl ConversionExpectBuiltin for crate::match_arg::MatchArgError {
    fn __mx_conversion_expectation(&self) -> Option<String> {
        Some(self.expectation())
    }
}

/// Expectation probe, fallback arm: any other error type says nothing about
/// what the value should have been.
#[doc(hidden)]
pub trait ConversionExpectNone {
    fn __mx_conversion_expectation(&self) -> Option<String>;
}

impl<E> ConversionExpectNone for &E {
    fn __mx_conversion_expectation(&self) -> Option<String> {
        None
    }
}

/// Internal: what a failed argument conversion's error says the value should
/// have been, for a parameter whose Rust type gives the macro no R-facing
/// expectation (#1594). `Some("one of \"fast\", \"slow\"")` for a `match_arg`
/// choice error, `Some("a data frame")` for a class error, the joined
/// expectation of an `Either` whose arms both refused the kind of value,
/// `None` otherwise. Not public API.
#[doc(hidden)]
#[macro_export]
macro_rules! __mx_conversion_expectation {
    ($e:expr) => {{
        #[allow(unused_imports)]
        use $crate::condition::{ConversionExpectBuiltin as _, ConversionExpectNone as _};
        match &$e {
            __mx_e => __mx_e.__mx_conversion_expectation(),
        }
    }};
}

/// Whole-message probe, built-in arm: a `match_arg` choice error, as a
/// [`MatchArgError`](crate::match_arg::MatchArgError) or at the top of a
/// `SexpError` (`#[derive(MatchArg)]`'s `TryFromSexp` returns one), words the
/// whole argument error the way the wrapper's R-side `match_arg` check
/// (`.miniextendr_match_arg`) does, from the same
/// [`MatchArgError::arg_error`](crate::match_arg::MatchArgError::arg_error)
/// (#1767): `'mode' should be one of "fast", "slow"`, `'mode' must be of
/// length 1`, `'mode' must be NULL or a character vector`. So a `match_arg`
/// type converted without `#[miniextendr(match_arg)]` and one checked by the
/// wrapper raise the same message.
///
/// A choice error inside an `Either` is not one: the `Either` names both of
/// its arms (`'x' must be one of "oral", "bolus", or a single double: got
/// logical`).
#[doc(hidden)]
pub trait ConversionArgMessageBuiltin {
    fn __mx_conversion_arg_message(&self, name: &str) -> Option<String>;
}

impl ConversionArgMessageBuiltin for crate::match_arg::MatchArgError {
    fn __mx_conversion_arg_message(&self, name: &str) -> Option<String> {
        Some(self.arg_error(name).message().to_string())
    }
}

impl ConversionArgMessageBuiltin for crate::from_r::SexpError {
    fn __mx_conversion_arg_message(&self, name: &str) -> Option<String> {
        match self {
            crate::from_r::SexpError::MatchArg(e) => Some(e.arg_error(name).message().to_string()),
            // A type's check refusal (`check_sexp` under `Option<T>` /
            // `Vec<T>`, or a conversion whose error converted into
            // `SexpError`) with its own whole message (#1833).
            crate::from_r::SexpError::Condition(e) => e.argument_message_str().map(str::to_string),
            _ => None,
        }
    }
}

/// Whole-message probe, classed arm: any [`RConditionData`] answers with its
/// [`argument_message`](RConditionData::argument_message), so a type's own
/// conversion error can word the whole argument error (#1833). Same autoref
/// level as [`ConversionArgMessageBuiltin`], like [`ConversionErrClassed`]:
/// no built-in error type implements [`RConditionData`].
#[doc(hidden)]
pub trait ConversionArgMessageClassed {
    fn __mx_conversion_arg_message(&self, name: &str) -> Option<String>;
}

impl<E: RConditionData> ConversionArgMessageClassed for E {
    fn __mx_conversion_arg_message(&self, _name: &str) -> Option<String> {
        self.argument_message()
    }
}

/// Whole-message probe, fallback arm: any other error leaves the message to
/// the prefix and the error's reason.
#[doc(hidden)]
pub trait ConversionArgMessageNone {
    fn __mx_conversion_arg_message(&self, name: &str) -> Option<String>;
}

impl<E> ConversionArgMessageNone for &E {
    fn __mx_conversion_arg_message(&self, _name: &str) -> Option<String> {
        None
    }
}

/// Internal: the whole message of a failed argument conversion whose error
/// words it itself (a `match_arg` choice error, see
/// [`ConversionArgMessageBuiltin`], or an error with an
/// [`argument_message`](RConditionData::argument_message), see
/// [`ConversionArgMessageClassed`]), for the argument `$name`; `None`
/// otherwise. [`crate::error_value::conversion_condition_value`] then uses it
/// in place of `<prefix>: <reason>`. Not public API.
#[doc(hidden)]
#[macro_export]
macro_rules! __mx_conversion_arg_message {
    ($e:expr, $name:expr) => {{
        #[allow(unused_imports)]
        use $crate::condition::{
            ConversionArgMessageBuiltin as _, ConversionArgMessageClassed as _,
            ConversionArgMessageNone as _,
        };
        match &$e {
            __mx_e => __mx_e.__mx_conversion_arg_message($name),
        }
    }};
}

/// The message prefix of a conversion failure whose expectation comes from
/// the error at run time (see [`crate::__mx_conversion_expectation!`]):
/// `'<p>' must be <expected>` (`'<p>' must be NULL or <expected>` for an
/// `Option<_>` parameter, `nullable`), else `invalid '<p>' argument`.
#[doc(hidden)]
pub fn conversion_prefix(param: &str, nullable: bool, expected: Option<&str>) -> String {
    match expected {
        Some(expected) if nullable => format!("'{param}' must be NULL or {expected}"),
        Some(expected) => format!("'{param}' must be {expected}"),
        None => format!("invalid '{param}' argument"),
    }
}

/// The name of the structured field that carries the failing parameter's R
/// name on every argument-conversion condition (`e$param`).
const CONVERSION_PARAM_FIELD: &str = "param";

/// The name of the structured field that carries the Rust type an argument
/// failed to convert to (`e$rust_type`, #1591).
const CONVERSION_RUST_TYPE_FIELD: &str = "rust_type";

/// Combine an argument-conversion error's own parts with the wrapper's
/// parameter context.
///
/// - The message becomes `<prefix>: <error message>`. For an argument the
///   macro writes the prefix in R terms (#1591): `'<p>' must be <expected>`
///   when it knows what the argument should be (`'x' must be a single
///   integer`, `'dv' must be numeric`), `invalid '<p>' argument` otherwise.
///   A sidecar setter passes its own prefix. An error that words the whole
///   argument error itself, a `match_arg` choice error (#1767) or one with an
///   [`argument_message`](RConditionData::argument_message) (#1833), gives
///   `arg_message` ([`crate::__mx_conversion_arg_message!`]), which is the
///   message instead.
/// - The class vector is the error's own (empty for the built-in and
///   `Display` arms), followed by `crate_class`, the crate's
///   `conversion_error_class` from `[package.metadata.miniextendr]`, minus
///   any class the error already names. The R helper appends the
///   `rust_error` layering after both.
/// - `param` (the parameter's R name) is added as the first data field,
///   `e$param`, and `rust_type` (the Rust type as written in the signature,
///   when given) after it, `e$rust_type`. Neither is added when the error's
///   own data already has a field of that name: the type author's value is
///   kept, since the framework never overwrites a field the type author
///   chose, and R's `e$name` reads only the first of two same-named fields.
/// - The call is always the wrapper's ([`ConditionCall::Inherit`]), whatever
///   the error type's [`RConditionData::call`] says: the R-side checks
///   (`.miniextendr_arg_error`) raise the same condition with the call
///   (#1591), and an argument error must not differ by which side caught it.
#[doc(hidden)]
pub fn conversion_err_parts(
    prefix: &str,
    param: &str,
    rust_type: Option<&str>,
    crate_class: &[&str],
    parts: ErrParts,
    arg_message: Option<String>,
) -> ErrParts {
    let ErrParts {
        message,
        mut class,
        data,
        call: _,
    } = parts;
    let message = arg_message.unwrap_or_else(|| format!("{prefix}: {message}"));
    for extra in crate_class {
        if !class.iter().any(|c| c == extra) {
            class.push((*extra).to_string());
        }
    }
    let mut fields = data.unwrap_or_default();
    let has_field =
        |fields: &ConditionData, field: &str| fields.iter().any(|(name, _)| name == field);
    let mut at = 0;
    if !has_field(&fields, CONVERSION_PARAM_FIELD) {
        fields.insert(0, (CONVERSION_PARAM_FIELD.to_string(), param.into()));
        at = 1;
    }
    if let Some(rust_type) = rust_type
        && !has_field(&fields, CONVERSION_RUST_TYPE_FIELD)
    {
        fields.insert(
            at,
            (CONVERSION_RUST_TYPE_FIELD.to_string(), rust_type.into()),
        );
    }
    ErrParts {
        message,
        class,
        data: Some(fields),
        call: ConditionCall::Inherit,
    }
}

/// The parts of an argument check that failed in Rust after the conversion:
/// `#[miniextendr(no_na)]` on a reading marker (`AsNumeric*`,
/// `AsCharacter*`) whose converted value holds an `NA` that R's `anyNA()`
/// did not see (the hidden `TryFromSexp::__mx_has_na` says so), or on an
/// `Either`, which has no R guard, an `NA` the arm taken refuses
/// (`TryFromSexp::__mx_input_has_na`).
///
/// The shape is the R guard's (`.miniextendr_arg_error`): `message` as given
/// (the macro writes `'<p>' must not be NA` or the author's own message),
/// the classes `crate_class` (the crate's `conversion_error_class`), and
/// `param` as the only data field, `e$param`. No `rust_type`: the R guard
/// has none, and the two paths raise one condition.
#[doc(hidden)]
pub fn arg_check_parts(message: &str, param: &str, crate_class: &[&str]) -> ErrParts {
    ErrParts {
        message: message.to_string(),
        class: crate_class.iter().map(|c| (*c).to_string()).collect(),
        data: Some(vec![(CONVERSION_PARAM_FIELD.to_string(), param.into())]),
        call: ConditionCall::Inherit,
    }
}

// endregion

// region: SerdeErrorShape — `serde_error` options as a shared const (#2207)

/// The options of `#[miniextendr(serde_error(..))]` as a value: declared once,
/// shared by every function that returns one error type.
///
/// ```
/// use miniextendr_api::condition::SerdeErrorShape;
///
/// pub const ENGINE: SerdeErrorShape = SerdeErrorShape::new()
///     .prefix("engine")
///     .skip(&["message"])
///     .as_character(&["rule", "column"])
///     .as_integer(&["n"]);
/// ```
///
/// Declare it for the error type with
/// [`serde_error_shape!`](crate::serde_error_shape), and every
/// `#[miniextendr]` function or method of the crate that returns that type
/// uses it, without naming it:
///
/// ```ignore
/// miniextendr_api::serde_error_shape!(EngineError = ENGINE);
///
/// #[miniextendr]
/// pub fn check(x: f64) -> Result<f64, EngineError> { /* ... */ }
/// ```
///
/// A function can override it for itself: with options of its own, or with
/// another shape named by path, which applies after the declared one:
///
/// ```ignore
/// #[miniextendr(serde_error(as_double("n")))]
/// pub fn rate(x: f64) -> Result<f64, EngineError> { /* ... */ }
///
/// #[miniextendr(serde_error(shape = STRICT, as_double("n")))]
/// pub fn strict_rate(x: f64) -> Result<f64, EngineError> { /* ... */ }
/// ```
///
/// The path resolves where the function is defined (`STRICT`,
/// `crate::errors::STRICT`, or `Self::SHAPE` on a method for an associated
/// const of its type), so a misspelt one is a compile error. The options act
/// on the serde-classed `Err` arm, which needs the `serde` feature.
///
/// # Options
///
/// One method per `serde_error` option. Fields are named as the error
/// serializes them, before any `rename`:
///
/// - [`tag`](Self::tag): the internally tagged discriminator field consumed
///   as the variant; `"kind"` when unset.
/// - [`prefix`](Self::prefix): the family class; `<crate>_error`, from the
///   function's crate, when unset.
/// - [`skip`](Self::skip): fields to drop.
/// - [`rename`](Self::rename): `(from, to)` pairs; field `from` reaches R as
///   `e$to`.
/// - [`as_character`](Self::as_character), [`as_integer`](Self::as_integer),
///   [`as_double`](Self::as_double), [`as_logical`](Self::as_logical): the
///   field's usual serialization as one vector of that R type, as the
///   `miniextendr_api::serde::as_character` & co. field helpers give it from
///   the error type's own `#[serde(serialize_with = ..)]`: `character(0)` for
///   an empty sequence, `NA` for a `None` element. A value that doesn't fit
///   (a string under `as_integer`) raises an R error naming the field and the
///   option.
///
/// A field that a variant doesn't carry is a no-op for that variant, so one
/// shape covers an enum whose variants differ.
///
/// # Precedence
///
/// Shapes apply in order: the one declared for the error type, then
/// `serde_error(shape = ..)`, then the function's own options; for a field
/// set by more than one, the later wins. A later `skip` drops a field an
/// earlier shape types or renames, and a later `as_*` or `rename` keeps a
/// field an earlier shape skips. A type and a new name compose: the
/// function's `rename(n = "count")` keeps the shape's `as_integer(&["n"])`.
/// A later `tag` or `prefix` replaces an earlier one.
///
/// # Contradictions don't compile
///
/// Each method checks its fields against the options already set and panics
/// during const evaluation, so a `const` holding a contradictory shape is a
/// compile error (E0080) naming the field:
///
/// - a field in two `as_*` lists;
/// - a field skipped and also typed or renamed;
/// - a field listed twice, renamed twice, or two renames onto one name;
/// - a `rename` target that is one of the condition's own slots (`message`,
///   `call`, `kind`);
/// - an empty list or name, and an option set twice.
///
/// ```compile_fail,E0080
/// use miniextendr_api::condition::SerdeErrorShape;
///
/// const ENGINE: SerdeErrorShape = SerdeErrorShape::new()
///     .as_integer(&["n"])
///     .as_double(&["n"]);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SerdeErrorShape {
    tag: Option<&'static str>,
    prefix: Option<&'static str>,
    skip: &'static [&'static str],
    rename: &'static [(&'static str, &'static str)],
    /// The `as_*` lists, in [`SHAPE_TYPE_OPTIONS`] order.
    types: [&'static [&'static str]; 4],
}

/// The `as_*` options, in the order of [`SerdeErrorShape`]'s type lists.
const SHAPE_TYPE_OPTIONS: [&str; 4] = ["as_logical", "as_integer", "as_double", "as_character"];

impl SerdeErrorShape {
    /// The discriminator field when no shape names one.
    #[cfg(feature = "serde")]
    const DEFAULT_TAG: &'static str = "kind";

    /// A shape that sets nothing.
    pub const fn new() -> Self {
        SerdeErrorShape {
            tag: None,
            prefix: None,
            skip: &[],
            rename: &[],
            types: [&[]; 4],
        }
    }

    /// The internally tagged discriminator field (`#[serde(tag = "...")]`),
    /// consumed as the variant.
    #[track_caller]
    pub const fn tag(mut self, tag: &'static str) -> Self {
        self.tag = Some(scalar_option("tag", self.tag, tag));
        self
    }

    /// The family class: conditions get `c("<prefix>_<variant>", "<prefix>")`.
    #[track_caller]
    pub const fn prefix(mut self, prefix: &'static str) -> Self {
        self.prefix = Some(scalar_option("prefix", self.prefix, prefix));
        self
    }

    /// Drop these payload fields.
    #[track_caller]
    pub const fn skip(mut self, fields: &'static [&'static str]) -> Self {
        check_field_list("skip", self.skip.is_empty(), fields);
        let mut i = 0;
        while i < fields.len() {
            let name = fields[i];
            if let Some(option) = self.type_option(name) {
                conflict(name, option, "skip", "a skipped field has no R type");
            }
            if self.renames(name) {
                conflict(name, "rename", "skip", "a skipped field has no name");
            }
            i += 1;
        }
        self.skip = fields;
        self
    }

    /// Splice field `from` as `to`, for each `(from, to)` pair.
    #[track_caller]
    pub const fn rename(mut self, pairs: &'static [(&'static str, &'static str)]) -> Self {
        if !self.rename.is_empty() {
            set_twice("rename");
        }
        if pairs.is_empty() {
            fail(ShapeMessage::new().push("`rename` needs at least one pair"));
        }
        let mut i = 0;
        while i < pairs.len() {
            let (from, to) = pairs[i];
            if from.is_empty() || to.is_empty() {
                fail(ShapeMessage::new().push("`rename` has an empty name"));
            }
            if is_reserved_condition_field(to) {
                fail(ShapeMessage::new().push("`rename` target `").push(to).push(
                    "` is reserved: `message`, `call` and `kind` are the condition's own slots",
                ));
            }
            let mut j = 0;
            while j < i {
                if str_eq(pairs[j].0, from) {
                    fail(
                        ShapeMessage::new()
                            .push("field `")
                            .push(from)
                            .push("` is renamed twice"),
                    );
                }
                if str_eq(pairs[j].1, to) {
                    fail(
                        ShapeMessage::new()
                            .push("two `rename` pairs target `")
                            .push(to)
                            .push("`; R would read only the first"),
                    );
                }
                j += 1;
            }
            if const_contains(self.skip, from) {
                conflict(from, "skip", "rename", "a skipped field has no name");
            }
            i += 1;
        }
        self.rename = pairs;
        self
    }

    /// Give these fields an R `logical` vector.
    #[track_caller]
    pub const fn as_logical(self, fields: &'static [&'static str]) -> Self {
        self.with_type(0, fields)
    }

    /// Give these fields an R `integer` vector; a value outside R's integer
    /// range is an error, not a double.
    #[track_caller]
    pub const fn as_integer(self, fields: &'static [&'static str]) -> Self {
        self.with_type(1, fields)
    }

    /// Give these fields an R `double` vector; integers widen.
    #[track_caller]
    pub const fn as_double(self, fields: &'static [&'static str]) -> Self {
        self.with_type(2, fields)
    }

    /// Give these fields an R `character` vector.
    #[track_caller]
    pub const fn as_character(self, fields: &'static [&'static str]) -> Self {
        self.with_type(3, fields)
    }

    #[track_caller]
    const fn with_type(mut self, index: usize, fields: &'static [&'static str]) -> Self {
        let option = SHAPE_TYPE_OPTIONS[index];
        check_field_list(option, self.types[index].is_empty(), fields);
        let mut i = 0;
        while i < fields.len() {
            let name = fields[i];
            if const_contains(self.skip, name) {
                conflict(name, "skip", option, "a skipped field has no R type");
            }
            if let Some(other) = self.type_option(name) {
                conflict(name, other, option, "a field has one R type");
            }
            i += 1;
        }
        self.types[index] = fields;
        self
    }

    /// The `as_*` option that lists `name`.
    const fn type_option(&self, name: &str) -> Option<&'static str> {
        let mut i = 0;
        while i < self.types.len() {
            if const_contains(self.types[i], name) {
                return Some(SHAPE_TYPE_OPTIONS[i]);
            }
            i += 1;
        }
        None
    }

    /// Whether a `rename` pair renames `name`.
    const fn renames(&self, name: &str) -> bool {
        let mut i = 0;
        while i < self.rename.len() {
            if str_eq(self.rename[i].0, name) {
                return true;
            }
            i += 1;
        }
        false
    }
}

impl Default for SerdeErrorShape {
    fn default() -> Self {
        Self::new()
    }
}

/// `tag` / `prefix`: set once, never empty.
#[track_caller]
const fn scalar_option(
    option: &str,
    current: Option<&'static str>,
    value: &'static str,
) -> &'static str {
    if current.is_some() {
        set_twice(option);
    }
    if value.is_empty() {
        fail(
            ShapeMessage::new()
                .push("`")
                .push(option)
                .push("` must not be empty"),
        );
    }
    value
}

/// A field list: set once, not empty, each name non-empty and listed once.
#[track_caller]
const fn check_field_list(option: &str, unset: bool, fields: &[&str]) {
    if !unset {
        set_twice(option);
    }
    if fields.is_empty() {
        fail(
            ShapeMessage::new()
                .push("`")
                .push(option)
                .push("` needs at least one field"),
        );
    }
    let mut i = 0;
    while i < fields.len() {
        if fields[i].is_empty() {
            fail(
                ShapeMessage::new()
                    .push("`")
                    .push(option)
                    .push("` has an empty field name"),
            );
        }
        let mut j = 0;
        while j < i {
            if str_eq(fields[j], fields[i]) {
                fail(
                    ShapeMessage::new()
                        .push("field `")
                        .push(fields[i])
                        .push("` is listed twice in `")
                        .push(option)
                        .push("`"),
                );
            }
            j += 1;
        }
        i += 1;
    }
}

#[track_caller]
const fn set_twice(option: &str) -> ! {
    fail(
        ShapeMessage::new()
            .push("`")
            .push(option)
            .push("` is set twice; set each option once"),
    )
}

#[track_caller]
const fn conflict(name: &str, first: &str, second: &str, why: &str) -> ! {
    fail(
        ShapeMessage::new()
            .push("field `")
            .push(name)
            .push("` is in both `")
            .push(first)
            .push("` and `")
            .push(second)
            .push("`; ")
            .push(why),
    )
}

const fn const_contains(list: &[&str], name: &str) -> bool {
    let mut i = 0;
    while i < list.len() {
        if str_eq(list[i], name) {
            return true;
        }
        i += 1;
    }
    false
}

/// A const-evaluation panic message. `const fn` can't `format!`, so the parts
/// are copied into a fixed buffer.
struct ShapeMessage {
    buf: [u8; 256],
    len: usize,
}

impl ShapeMessage {
    const fn new() -> Self {
        ShapeMessage {
            buf: [0; 256],
            len: 0,
        }
        .push("SerdeErrorShape: ")
    }

    const fn push(mut self, text: &str) -> Self {
        let bytes = text.as_bytes();
        let mut i = 0;
        while i < bytes.len() && self.len < self.buf.len() {
            self.buf[self.len] = bytes[i];
            self.len += 1;
            i += 1;
        }
        self
    }
}

/// Panic with `message`. `#[track_caller]` here and on every caller puts the
/// compile error at the user's `const`, not in this crate.
#[track_caller]
const fn fail(message: ShapeMessage) -> ! {
    let (text, _) = message.buf.split_at(message.len);
    match core::str::from_utf8(text) {
        Ok(text) => panic!("{}", text),
        // The buffer ended inside a long field name's character.
        Err(_) => {
            panic!("SerdeErrorShape: contradictory options; a field name is too long to quote")
        }
    }
}

// endregion

// region: Serde-tagged Result errors (#[miniextendr(serde_error)])

/// Build the `Err`-arm parts from the error's serde shape.
///
/// Reached three ways: for a `Result<T, E>` whose `E` has a shape declared
/// by [`serde_error_shape!`](crate::serde_error_shape) (the
/// [`ErrShapeDeclared`] probe arm, that one shape); automatically, for every
/// other `E` that is `Serialize + Display` without an [`RConditionData`] impl
/// (the [`ErrPartsSerde`] probe arm, no shapes); or explicitly through
/// `#[miniextendr(serde_error(..))]`, whose options arrive as `shapes`: the
/// declared shape first (one that sets nothing when there is none), then the
/// named `shape = ..`, then the function's own options as an inline const,
/// each applied in order (see [`SerdeErrorShape`], "Precedence").
/// `prefix` is the macro's `<crate>_error`, used when no shape sets one.
///
/// Message from `Display`; classes `[<prefix>_<variant>, <prefix>]` when the
/// serialized value carries a variant, `[<prefix>]` otherwise; data from the
/// serialized fields (checked against the reserved names). `tag` names the
/// internally-tagged discriminator field (`#[serde(tag = "kind")]`), which is
/// consumed as the member class rather than spliced as data; externally tagged
/// enums (serde's default) yield the variant name directly. See
/// `docs/CONDITIONS.md`, "Classed `Result` errors".
///
/// Field control, applied to the payload fields before the reserved-name
/// check (`serde_error(skip(..), rename(..), as_*(..))`, #1457, #2207):
///
/// - `skip`: fields with these names are dropped.
/// - `as_character` / `as_integer` / `as_double` / `as_logical`: the field's
///   value becomes one vector of that type, through the same conversion as
///   the `serde::as_*` field helpers; a value that doesn't fit panics, naming
///   the field and the option.
/// - `rename`: `(from, to)` pairs; a field named `from` is spliced as `to`.
/// - A remaining field named `message` whose value is the single string equal
///   to the `Display` output is dropped: it duplicates the condition's own
///   `message` slot, so a wrapped parser error of the shape
///   `Variant { message: String }` reaches R without any option.
///
/// Names that a given variant does not carry are a no-op for that variant.
/// Any other collision with `message`, `call` or `kind` still panics via
/// [`check_condition_data`].
///
/// After `skip` / `rename` the field names must be distinct: a `rename` whose
/// target is a name the variant already carries (or two payload fields that
/// serialize under one name) would give the condition two entries with the
/// same name, of which R's `e$name` reads only the first. That panics too,
/// naming the field and the rename that produced it (#1459).
///
/// Serialization failing (a map with non-string keys, a custom `Serialize`
/// erroring) panics with both failures in the message: the original error
/// text must not be lost behind the reporting problem.
#[cfg(feature = "serde")]
#[doc(hidden)]
#[track_caller]
pub fn serde_err_parts<E: ?Sized + ::serde::Serialize + std::fmt::Display>(
    e: &E,
    prefix: &str,
    shapes: &[&SerdeErrorShape],
) -> ErrParts {
    let message = e.to_string();
    let tag = shapes
        .iter()
        .rev()
        .find_map(|s| s.tag)
        .unwrap_or(SerdeErrorShape::DEFAULT_TAG);
    let prefix = shapes.iter().rev().find_map(|s| s.prefix).unwrap_or(prefix);
    let (member, fields) = match crate::serde::rvalue_ser::tagged_parts(e, tag) {
        Ok(parts) => parts,
        Err(err) => panic!(
            "#[miniextendr(serde_error)]: serializing `{}` for its condition data failed ({err}); \
             the original error was: {message}",
            std::any::type_name::<E>()
        ),
    };
    let mut class = Vec::with_capacity(2);
    if let Some(member) = member {
        class.push(format!("{prefix}_{member}"));
    }
    class.push(prefix.to_string());
    let mut renames = Vec::new();
    let mut data: ConditionData = Vec::with_capacity(fields.len());
    for (name, value) in fields {
        let Some(field) = FieldShape::resolve(shapes, &name) else {
            continue;
        };
        let value = match field.atomic {
            None => value,
            Some(atomic) => match atomic.try_coerce(value) {
                Ok(value) => value,
                Err(element) => panic!(
                    "#[miniextendr(serde_error)]: `{}` on condition data field `{name}` failed: \
                     {}; the original error was: {message}",
                    atomic.helper(),
                    atomic.mismatch(element)
                ),
            },
        };
        if let Some((from, to)) = field.rename {
            renames.push((from, to));
            data.push((to.to_string(), value));
            continue;
        }
        let echoes_display = name == "message"
            && matches!(&value, crate::RValue::Character(v)
                if v.len() == 1 && v[0].as_deref() == Some(message.as_str()));
        if !echoes_display {
            data.push((name, value));
        }
    }
    check_distinct_field_names(&data, &renames);
    ErrParts {
        message,
        class,
        data: check_condition_data((!data.is_empty()).then_some(data)),
        call: ConditionCall::Inherit,
    }
}

/// What the shapes, applied in order, give one payload field.
#[cfg(feature = "serde")]
#[derive(Default)]
struct FieldShape {
    atomic: Option<crate::serde::rvalue_ser::Atomic>,
    rename: Option<(&'static str, &'static str)>,
}

#[cfg(feature = "serde")]
impl FieldShape {
    /// `None` when the last shape that names the field skips it. A later
    /// `skip` drops what earlier shapes set; a later type or rename replaces
    /// that setting and keeps the field.
    fn resolve(shapes: &[&SerdeErrorShape], name: &str) -> Option<Self> {
        let mut skipped = false;
        let mut field = FieldShape::default();
        for shape in shapes {
            if shape.skip.contains(&name) {
                skipped = true;
                field = FieldShape::default();
            }
            if let Some(i) = shape.types.iter().position(|list| list.contains(&name)) {
                skipped = false;
                field.atomic = Some(crate::serde::rvalue_ser::Atomic::ALL[i]);
            }
            if let Some(&pair) = shape.rename.iter().find(|(from, _)| *from == name) {
                skipped = false;
                field.rename = Some(pair);
            }
        }
        (!skipped).then_some(field)
    }
}

/// Panic when two payload fields share a name after `skip` / `rename`.
///
/// R's `e$<name>` returns the first entry with that name and the second is
/// unreachable, which is the silent-overwrite failure the reserved-name rule
/// exists to remove (#1440). Neither first-wins nor last-wins would report it.
#[cfg(feature = "serde")]
#[track_caller]
fn check_distinct_field_names(fields: &[(String, crate::RValue)], rename: &[(&str, &str)]) {
    let mut seen = std::collections::HashSet::with_capacity(fields.len());
    for (name, _) in fields {
        if seen.insert(name.as_str()) {
            continue;
        }
        let renames: Vec<String> = rename
            .iter()
            .filter(|(_, to)| *to == name)
            .map(|(from, to)| format!("`rename({from} = \"{to}\")`"))
            .collect();
        if renames.is_empty() {
            panic!(
                "#[miniextendr(serde_error)]: condition data field `{name}` appears twice in the \
                 serialized payload; R's `e${name}` would read only the first. Give the fields \
                 distinct names or `skip` one of them."
            );
        }
        panic!(
            "#[miniextendr(serde_error)]: condition data field `{name}` appears twice: {} targets \
             a name this variant already carries; R's `e${name}` would read only the first. Pick \
             another target or `skip` the existing field.",
            renames.join(" and ")
        );
    }
}

// endregion

// region: Declared shapes — one SerdeErrorShape per error type (#2207)

/// The [`SerdeErrorShape`] declared for the error type `Self` by the crate
/// whose marker type is `M`. Implemented by
/// [`serde_error_shape!`](crate::serde_error_shape), not by hand.
///
/// `M` is a type local to the declaring crate (`miniextendr_init!` defines it
/// at the crate root). That is what lets an R-facing crate declare a shape
/// for an error type from a crate that doesn't depend on miniextendr: the
/// orphan rule accepts `impl ForeignTrait<LocalType> for ForeignType`. It also
/// scopes the declaration to the crate: another crate returning the same type
/// looks up its own declaration.
///
/// The supertraits make a declaration for a type that can't be serialized, or
/// has no `Display` for the condition message, an error at the declaration.
#[cfg(feature = "serde")]
pub trait SerdeErrorShapeFor<M>: ::serde::Serialize + std::fmt::Display {
    /// The declared shape.
    const SHAPE: &'static SerdeErrorShape;
}

/// Declare the [`SerdeErrorShape`] of an error type once, for every
/// `#[miniextendr]` function and method of this crate that returns it.
///
/// ```ignore
/// use miniextendr_api::condition::SerdeErrorShape;
///
/// pub const ENGINE: SerdeErrorShape = SerdeErrorShape::new()
///     .prefix("engine")
///     .as_character(&["rule"])
///     .as_integer(&["n"]);
///
/// miniextendr_api::serde_error_shape!(engine::Error = ENGINE);
///
/// // Both raise with ENGINE's field types; neither names it.
/// #[miniextendr]
/// pub fn check(x: f64) -> Result<f64, engine::Error> { /* ... */ }
///
/// #[miniextendr]
/// pub fn rate(x: f64) -> Result<f64, engine::Error> { /* ... */ }
/// ```
///
/// The right-hand side is any constant expression of type
/// `SerdeErrorShape`: a path, as above, or the builder chain itself. The type
/// must be `Serialize + Display` and may come from any crate, including one
/// that can't depend on miniextendr. The declaration goes anywhere in the
/// crate that calls `miniextendr_init!`, which defines the marker type it is
/// keyed by (see [`SerdeErrorShapeFor`]); it applies to that crate's
/// functions only.
///
/// # Precedence
///
/// Every `Err` of the type, from a function with or without
/// `#[miniextendr(serde_error(..))]`, is classed through serde with the
/// declared shape applied first. A function's `serde_error(shape = X)` applies
/// next and its own options last; for a field set more than once, the later
/// setting wins, by the rules under [`SerdeErrorShape`], "Precedence".
///
/// The declaration also wins over an [`RConditionData`] impl of the type: it is
/// the R-facing crate's choice for its own functions, where the impl is the
/// error crate's general one.
///
/// # One declaration per type
///
/// A second declaration for a type in the same crate is the compiler's
/// conflicting-implementations error:
///
/// ```compile_fail,E0119
/// use miniextendr_api::condition::SerdeErrorShape;
///
/// miniextendr_api::miniextendr_init!(pkg);
///
/// const A: SerdeErrorShape = SerdeErrorShape::new().prefix("a");
/// const B: SerdeErrorShape = SerdeErrorShape::new().prefix("b");
///
/// miniextendr_api::serde_error_shape!(String = A);
/// miniextendr_api::serde_error_shape!(String = B);
///
/// fn main() {}
/// ```
#[cfg(feature = "serde")]
#[macro_export]
// `crate::` is meant: the marker is the calling crate's.
#[allow(clippy::crate_in_macro_def)]
macro_rules! serde_error_shape {
    ($error:ty = $shape:expr $(,)?) => {
        impl $crate::condition::SerdeErrorShapeFor<crate::__MiniextendrInitMarker> for $error {
            const SHAPE: &'static $crate::condition::SerdeErrorShape = &$shape;
        }
    };
}

/// Declare the `SerdeErrorShape` of an error type once, for every
/// `#[miniextendr]` function and method of this crate that returns it. Needs
/// the `serde` feature, without which a declaration is a compile error.
#[cfg(not(feature = "serde"))]
#[macro_export]
macro_rules! serde_error_shape {
    ($($input:tt)*) => {
        ::core::compile_error!(
            "`serde_error_shape!` needs the `serde` feature of miniextendr-api; \
             enable it to class errors through serde"
        );
    };
}

/// The [`RCondition`](enum@RCondition) an error type that is
/// `Serialize + Display` gives as a `#[miniextendr]` function's `Err`, for
/// code that is not an entry point (#2217): a helper returning
/// `Result<T, RCondition>`, a body mixing it with conditions
/// [`catch_condition`] caught.
///
/// `serde_condition!(&err)` takes a reference to the error and runs the probe
/// a `Result` return's `Err` arm runs, with the calling crate's marker and
/// family class: the shape [`serde_error_shape!`](crate::serde_error_shape)
/// declared for the type in this crate, else the type's [`RConditionData`]
/// impl, else its serde shape with the `<crate>_error` family class. The
/// condition is an error ([`RCondition::Error`]) with the classes, message,
/// fields and call choice an entry point returning `Err(err)` would raise.
///
/// ```ignore
/// use miniextendr_api::condition::{RCondition, catch_condition};
///
/// miniextendr_api::serde_error_shape!(engine::Error = ENGINE);
///
/// impl Model {
///     fn load(path: &str) -> Result<Self, RCondition> {
///         let spec = catch_condition(|| Self::spec(path))?;
///         let engine = engine::Engine::load(&spec)
///             .map_err(|e| miniextendr_api::serde_condition!(&e))?;
///         Ok(Model::new(spec, engine))
///     }
/// }
/// ```
///
/// The type must be `Serialize + Display`, which the macro checks. The marker
/// is the one `miniextendr_init!` defines at the crate root, so the crate must
/// call it, as every R package does. Without the `serde` feature the macro
/// is a compile error.
///
/// Pass one reference. An error already held as `&E` (`if let Err(e) =
/// &result`) goes in as it is, `serde_condition!(e)`: `&e` is a `&&E`, which
/// compiles, since `&E` is `Serialize + Display` too, but skips the declared
/// shape and the `RConditionData` impl, as an entry point returning `&E`
/// would.
#[cfg(feature = "serde")]
#[macro_export]
// `crate::` is meant: the marker is the calling crate's.
#[allow(clippy::crate_in_macro_def)]
macro_rules! serde_condition {
    // A `match`, not a `let`: a temporary in `$err` (`&Error::new(..)`) lives
    // to the end of the scrutinee's match.
    ($err:expr $(,)?) => {
        match $crate::condition::__mx_serde_condition_arg($err) {
            __mx_err => {
                let __mx_parts = $crate::__mx_result_err_parts!(
                    *__mx_err,
                    ::core::concat!(::core::env!("CARGO_CRATE_NAME"), "_error"),
                    crate::__MiniextendrInitMarker
                );
                $crate::condition::RCondition::Error {
                    message: __mx_parts.message,
                    class: __mx_parts.class,
                    data: __mx_parts.data,
                    call: __mx_parts.call,
                }
            }
        }
    };
}

/// The `RCondition` a `Serialize + Display` error gives as an entry point's
/// `Err`. Needs the `serde` feature, without which it is a compile error.
#[cfg(not(feature = "serde"))]
#[macro_export]
macro_rules! serde_condition {
    ($($input:tt)*) => {
        ::core::compile_error!(
            "`serde_condition!` needs the `serde` feature of miniextendr-api; \
             enable it to class errors through serde"
        )
    };
}

/// Internal: [`serde_condition!`](crate::serde_condition)'s argument, a
/// reference to a `Serialize + Display` error, unchanged. Taking `&E` fixes
/// the reference depth the probe's stages see: one reference more would
/// skip the declared shape and the [`RConditionData`] stage.
#[cfg(feature = "serde")]
#[doc(hidden)]
#[inline]
pub fn __mx_serde_condition_arg<E: ::serde::Serialize + std::fmt::Display>(err: &E) -> &E {
    err
}

/// The receiver of the declared-shape probe: the error, and the marker `M` of
/// the crate whose declaration to look up.
#[cfg(feature = "serde")]
#[doc(hidden)]
pub struct DeclaredShapeProbe<'a, M, E>(pub &'a E, pub std::marker::PhantomData<fn() -> M>);

/// Declared-shape probe, preferred arm: `E` has a shape declared under `M`.
///
/// The generated code calls `(&probe).__mx_declared_parts(..)` with both
/// traits in scope. This impl, on the probe taking `&self`, matches the
/// receiver `&DeclaredShapeProbe` by value; [`ErrShapeUndeclared`] on
/// `&DeclaredShapeProbe` needs one more auto-ref and is only reached when
/// nothing is declared.
#[cfg(feature = "serde")]
#[doc(hidden)]
pub trait ErrShapeDeclared {
    /// The parts under the declared shape, for an `Err` arm without
    /// `serde_error(..)`. The call to [`serde_err_parts`] lives here, where
    /// `E: Serialize + Display` holds, so an undeclared `E` needn't be.
    fn __mx_declared_parts(&self, prefix: &str) -> Option<ErrParts>;
    /// The declared shape, first in `serde_error(..)`'s shape list.
    fn __mx_declared_shape(&self) -> &'static SerdeErrorShape;
}

#[cfg(feature = "serde")]
impl<M, E: SerdeErrorShapeFor<M>> ErrShapeDeclared for DeclaredShapeProbe<'_, M, E> {
    #[track_caller]
    fn __mx_declared_parts(&self, prefix: &str) -> Option<ErrParts> {
        Some(serde_err_parts(self.0, prefix, &[E::SHAPE]))
    }

    fn __mx_declared_shape(&self) -> &'static SerdeErrorShape {
        E::SHAPE
    }
}

/// Declared-shape probe, fallback arm: no declaration. No parts, and a shape
/// that sets nothing.
#[cfg(feature = "serde")]
#[doc(hidden)]
pub trait ErrShapeUndeclared {
    fn __mx_declared_parts(&self, prefix: &str) -> Option<ErrParts>;
    fn __mx_declared_shape(&self) -> &'static SerdeErrorShape;
}

#[cfg(feature = "serde")]
impl<M, E> ErrShapeUndeclared for &DeclaredShapeProbe<'_, M, E> {
    fn __mx_declared_parts(&self, _prefix: &str) -> Option<ErrParts> {
        None
    }

    fn __mx_declared_shape(&self) -> &'static SerdeErrorShape {
        const NONE: &SerdeErrorShape = &SerdeErrorShape::new();
        NONE
    }
}

/// Internal: the declared-shape stage of [`crate::__mx_result_err_parts!`].
/// Without the `serde` feature it is `None` and `$marker` is never resolved,
/// so a crate without `miniextendr_init!` (a test crate) still compiles.
#[cfg(feature = "serde")]
#[doc(hidden)]
#[macro_export]
macro_rules! __mx_declared_err_parts {
    ($e:ident, $prefix:expr, $marker:ty) => {{
        #[allow(unused_imports)]
        use $crate::condition::{ErrShapeDeclared as _, ErrShapeUndeclared as _};
        (&$crate::condition::DeclaredShapeProbe::<$marker, _>($e, ::core::marker::PhantomData))
            .__mx_declared_parts($prefix)
    }};
}

/// Internal: the declared-shape stage of [`crate::__mx_result_err_parts!`];
/// nothing can be declared without the `serde` feature.
#[cfg(not(feature = "serde"))]
#[doc(hidden)]
#[macro_export]
macro_rules! __mx_declared_err_parts {
    ($e:ident, $prefix:expr, $marker:ty) => {
        ::core::option::Option::None::<$crate::condition::ErrParts>
    };
}

/// Internal: the shape `serde_error_shape!` declared for the type of `$e`
/// under `$marker`, or one that sets nothing. It leads the shape list of a
/// `#[miniextendr(serde_error(..))]` `Err` arm.
#[cfg(feature = "serde")]
#[doc(hidden)]
#[macro_export]
macro_rules! __mx_declared_serde_shape {
    ($e:expr, $marker:ty) => {{
        #[allow(unused_imports)]
        use $crate::condition::{ErrShapeDeclared as _, ErrShapeUndeclared as _};
        (&$crate::condition::DeclaredShapeProbe::<$marker, _>(&$e, ::core::marker::PhantomData))
            .__mx_declared_shape()
    }};
}

/// Internal: without the `serde` feature, a shape that sets nothing.
/// `serde_error(..)` needs the feature anyway: its `serde_err_parts` call
/// reports that, and `$marker` is never resolved.
#[cfg(not(feature = "serde"))]
#[doc(hidden)]
#[macro_export]
macro_rules! __mx_declared_serde_shape {
    ($e:expr, $marker:ty) => {{
        const NONE: &$crate::condition::SerdeErrorShape =
            &$crate::condition::SerdeErrorShape::new();
        NONE
    }};
}

// endregion

// region: from_tagged_sexp + repanic_if_rust_error — shim re-panic helpers

impl RCondition {
    /// Reconstruct an [`RCondition::Error`] from a tagged SEXP produced by
    /// [`crate::error_value::make_rust_condition_value`].
    ///
    /// Returns `Some(RCondition)` when `sexp` has class `"rust_condition_value"` AND
    /// the `"__rust_condition__"` attribute is `TRUE`. Returns `None` for all other
    /// SEXPs (normal return values, `R_NilValue`, etc.).
    ///
    /// Reconstructs the matching variant for each kind: `"error"`/`"panic"`/
    /// `"result_err"`/`"none_err"`/`"other_rust_error"` →
    /// [`RCondition::Error`] (class, message, data and a [`ConditionCall::None`]
    /// choice kept; the kind itself is not carried by the variant);
    /// `"conversion"` → [`RCondition::Conversion`], so an argument error keeps
    /// `kind = "conversion"` across a package boundary (and an
    /// [`crate::arg_error!`] its class marker, which the consumer's wrapper
    /// resolves); `"warning"` → [`RCondition::Warning`]; `"message"` →
    /// [`RCondition::Message`]; `"condition"` → [`RCondition::Condition`].
    /// Unknown kinds degrade to [`RCondition::Error`] with the kind string
    /// prefixed to the message.
    ///
    /// # Safety
    ///
    /// Must be called from R's main thread.
    pub unsafe fn from_tagged_sexp(sexp: crate::SEXP) -> Option<Self> {
        use crate::SexpExt;
        use crate::from_r::TryFromSexp;

        // Use SexpExt::inherits_class — wraps Rf_inherits, already main-thread.
        if !sexp.inherits_class(c"rust_condition_value") {
            return None;
        }

        // Belt-and-suspenders PROTECT across the full inspection window. The reads
        // below are nominally non-allocating, but R-devel's GC is aggressive enough
        // (see MEMORY.md "Common gotchas") that a defensive guard is cheap and
        // closes the door on subtle regressions if the read path ever changes.
        let _guard = unsafe { crate::gc_protect::OwnedProtect::new(sexp) };

        // Verify the __rust_condition__ marker attribute is TRUE (a length-1 LGLSXP
        // with value 1). This guards against coincidental class attribute collisions.
        let attr_sym = crate::cached_class::rust_condition_attr_symbol();
        let marker = sexp.get_attr(attr_sym);
        // marker should be a scalar logical TRUE: is_logical() and logical_elt(0) == 1
        if !marker.is_logical() || marker.logical_elt(0) != 1 {
            return None;
        }

        // It's a tagged SEXP. Read the elements.
        // Both 3-element (legacy) and 4-element (condition) forms have:
        //   [0] = error message (STRSXP)
        //   [1] = kind string (STRSXP)
        //   [2] = class name or NULL (only in 4-element form; absent in legacy)

        let len = sexp.len();

        // Defense-in-depth: a tagged SEXP must have at least the message and kind
        // slots. inherits_class + __rust_condition__ marker should already imply this,
        // but a corrupted/spoofed SEXP that satisfies both checks shouldn't OOB
        // the vector_elt reads below.
        if len < 2 {
            return None;
        }

        let msg_sexp = sexp.vector_elt(0);
        let msg: String = msg_sexp
            .string_elt_str(0)
            .unwrap_or("<invalid error message>")
            .to_string();

        let kind_sexp = sexp.vector_elt(1);
        let kind: &str = kind_sexp
            .string_elt_str(0)
            .unwrap_or(crate::error_value::kind::PANIC);

        // Class slot is element [2] in the 4-element form (NULL in legacy form):
        // a STRSXP of any length (the user class vector, family last).
        let class: Vec<String> = if len >= 4 {
            let class_sexp = sexp.vector_elt(2);
            if class_sexp.is_nil() || !class_sexp.is_character() {
                Vec::new()
            } else {
                (0..class_sexp.len() as isize)
                    .filter_map(|i| class_sexp.string_elt_str(i).map(str::to_string))
                    .collect()
            }
        } else {
            Vec::new()
        };

        use crate::error_value::kind as kind_const;

        // Slot [4] is the optional named-list condition data, present when `len >= 5`.
        //
        // Each field value is decoded through the single SEXP→owned-tree walker,
        // [`RValue::try_from_sexp`], so structured fields survive the cross-package
        // trait-ABI re-panic path (`repanic_if_rust_error`): the consumer's outer
        // `with_r_unwind_protect` guard rebuilds the tagged SEXP from the
        // reconstructed `RCondition`, which now carries the data — so `e$field_name`
        // is accessible in R handlers even when the error crossed a package boundary.
        //
        // `RValue` is NA-aware (logical/integer/character carry `None`; double
        // carries the `NA_REAL` bit), so NA-bearing fields now round-trip faithfully
        // rather than being dropped. Fields whose name is missing/empty, or whose
        // value is not R data (closures, environments, …, which `try_from_sexp`
        // rejects) are dropped — safe degradation that preserves message/class/kind.
        //
        // All reads here are non-allocating copies into owned Rust values, so no new
        // SEXPs are created and the existing `_guard` OwnedProtect suffices.
        let data: Option<ConditionData> = if len >= 5 {
            let data_sexp = sexp.vector_elt(4);
            if data_sexp.is_nil() || !data_sexp.is_list() {
                None
            } else {
                let data_len = data_sexp.len();
                let names_sexp = data_sexp.get_names();
                let mut fields: ConditionData = Vec::with_capacity(data_len);
                for i in 0..data_len as isize {
                    // Read the field name from the names attribute. If missing/empty, skip.
                    let name: String = if names_sexp.is_nil() || !names_sexp.is_character() {
                        continue;
                    } else {
                        match names_sexp.string_elt_str(i) {
                            Some(s) if !s.is_empty() => s.to_string(),
                            _ => continue,
                        }
                    };
                    if let Ok(value) = crate::RValue::try_from_sexp(data_sexp.vector_elt(i)) {
                        fields.push((name, value));
                    }
                }
                if fields.is_empty() {
                    None
                } else {
                    Some(fields)
                }
            }
        } else {
            None
        };

        // Slot [3] is the call. Only the "no call" marker (`FALSE`, see
        // `make_rust_condition_value_with_data`) survives the re-panic: a call
        // object captured by the producer names the vtable shim's frame, and the
        // consumer's outer guard attributes the condition to its own call.
        let call = if len >= 4 && is_no_call_marker(sexp.vector_elt(3)) {
            ConditionCall::None
        } else {
            ConditionCall::Inherit
        };

        let cond = match kind {
            kind_const::ERROR
            | kind_const::PANIC
            | kind_const::RESULT_ERR
            | kind_const::NONE_ERR
            | kind_const::OTHER_RUST_ERROR => RCondition::Error {
                message: msg,
                class,
                data,
                call,
            },
            kind_const::CONVERSION => RCondition::Conversion {
                message: msg,
                class,
                data,
                call,
            },
            kind_const::WARNING => RCondition::Warning {
                message: msg,
                class,
                data,
                call,
            },
            kind_const::MESSAGE => RCondition::Message {
                message: msg,
                class,
                data,
                call,
            },
            kind_const::CONDITION => RCondition::Condition {
                message: msg,
                class,
                data,
                call,
            },
            other => {
                // Unknown kind — degrade to error
                RCondition::Error {
                    message: format!("[{other}] {msg}"),
                    class,
                    data,
                    call,
                }
            }
        };
        Some(cond)
    }
}

/// `true` for the tagged value's "no call" marker in the `call` slot: a
/// length-1 logical `FALSE` ([`ConditionCall::None`]). A call object or `NULL`
/// is not the marker.
fn is_no_call_marker(call: crate::SEXP) -> bool {
    use crate::SexpExt;
    call.is_logical() && call.len() == 1 && call.logical_elt(0) == 0
}

/// Inspect a SEXP returned by a trait-ABI vtable shim and, if it is a tagged
/// error value, re-panic with the reconstructed [`enum@RCondition`].
///
/// This is the "re-panic at the View boundary" step of Approach 1 from the
/// issue-345 plan. The caller (a generated View method wrapper) does:
///
/// ```ignore
/// let result = { vtable_call };
/// ::miniextendr_api::trait_abi::repanic_if_rust_error(result);
/// // ... convert result normally if we reach here
/// ```
///
/// When `sexp` is a tagged error value:
/// - `RCondition::Error` / `RCondition::Warning` / etc. → `panic_any!(cond)`.
///   The outer `with_r_unwind_protect` in the consumer's C entry point will
///   catch this and produce a tagged SEXP for the consumer's R wrapper.
///
/// When `sexp` is a normal value: this is a no-op.
///
/// # Safety
///
/// Must be called from R's main thread. `sexp` must be a valid (possibly
/// tagged) SEXP.
pub unsafe fn repanic_if_rust_error(sexp: crate::SEXP) {
    if let Some(cond) = unsafe { RCondition::from_tagged_sexp(sexp) } {
        std::panic::panic_any(cond);
    }
}

// endregion

// region: AsRError struct — wraps std::error::Error for Result returns

/// Structured error wrapper that preserves the `std::error::Error` cause chain.
///
/// When displayed, formats the error message with its full source chain:
/// ```text
/// top-level message
///   caused by: middle error
///   caused by: root cause
/// ```
///
/// Implements `From<E>` so it works with `?` and `.map_err(AsRError)`.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::condition::AsRError;
/// use std::num::ParseIntError;
///
/// #[miniextendr]
/// fn parse_number(s: &str) -> Result<i32, AsRError<ParseIntError>> {
///     s.parse::<i32>().map_err(AsRError)
/// }
/// ```
pub struct AsRError<E: std::error::Error>(pub E);

impl<E: std::error::Error> From<E> for AsRError<E> {
    #[inline]
    fn from(err: E) -> Self {
        AsRError(err)
    }
}

impl<E: std::error::Error> std::fmt::Display for AsRError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Write the top-level message
        write!(f, "{}", self.0)?;

        // Walk the cause chain
        let mut current: &dyn std::error::Error = &self.0;
        while let Some(source) = current.source() {
            write!(f, "\n  caused by: {source}")?;
            current = source;
        }

        Ok(())
    }
}

impl<E: std::error::Error> std::fmt::Debug for AsRError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AsRError<{}>({})", std::any::type_name::<E>(), self)
    }
}

impl<E: std::error::Error> AsRError<E> {
    /// Get the inner error.
    #[inline]
    pub fn into_inner(self) -> E {
        self.0
    }

    /// Get the Rust type name of the wrapped error (for programmatic matching).
    #[inline]
    pub fn rust_type_name(&self) -> &'static str {
        std::any::type_name::<E>()
    }

    /// Collect the full cause chain as a `Vec<String>`.
    pub fn cause_chain(&self) -> Vec<String> {
        let mut chain = vec![self.0.to_string()];
        let mut current: &dyn std::error::Error = &self.0;
        while let Some(source) = current.source() {
            chain.push(source.to_string());
            current = source;
        }
        chain
    }
}

// endregion

// region: Tests — macro grammar + payload contents (no R runtime needed)

#[cfg(test)]
mod tests;

// endregion
