//! R-side precondition generation for type checking.
//!
//! Generates argument checks in R wrapper functions that run BEFORE the `.Call()` boundary.
//! This gives users clear, idiomatic R error messages with proper stack traces instead of
//! Rust panic messages.
//!
//! Each check tests ONE thing and names the requirement it enforces:
//!
//! ```r
//! add <- function(a, b) {
//!   if (!isTRUE(is.integer(a))) .miniextendr_arg_error("a", "must be integer")
//!   if (!isTRUE(length(a) == 1L)) .miniextendr_arg_error("a", "must have length 1")
//!   if (!isTRUE(is.integer(b))) .miniextendr_arg_error("b", "must be integer")
//!   if (!isTRUE(length(b) == 1L)) .miniextendr_arg_error("b", "must have length 1")
//!   .Call(C_add, .call = sys.call(), a, b)
//! }
//! ```
//!
//! A failing check calls the preamble helper `.miniextendr_arg_error`
//! (`miniextendr-api/src/registry.rs`), which raises the same condition as a
//! failed Rust conversion: the crate's `conversion_error_class`, `rust_error`,
//! `kind = "conversion"` and `e$param`, with the message `'a' must be integer`
//! (#1591). A passing check costs one `isTRUE()` test; the condition is only
//! built on failure. `isTRUE()` keeps `stopifnot()`'s failure semantics (`NA`
//! fails too), and the guards are cheaper than the `stopifnot()` call they
//! replace. [`conversion_expectation`] gives the Rust conversion the same
//! vocabulary for its messages.

use std::collections::{HashMap, HashSet};

use crate::naming::r_string_escape;

/// A single R-side check on one parameter: a requirement and the R condition
/// enforcing it.
///
/// Formatted as a guard (see [`RAssertion::to_guard`]):
/// `if (!isTRUE(is.numeric(x))) .miniextendr_arg_error("x", "must be numeric")`.
struct RAssertion {
    /// R name of the checked parameter (`e$param` on failure).
    param: String,
    /// What the parameter must satisfy, without the parameter name
    /// (`must be numeric`). The condition message is `'<param>' <requirement>`.
    requirement: String,
    /// R expression that must evaluate to `TRUE` for the check to pass.
    condition: String,
    /// The author's own condition message (`message = "..."` on `inherits` /
    /// `not_inherits` / `no_na`; an `inherits` message also on the
    /// parameter's type checks),
    /// used verbatim instead of `'<param>' <requirement>`.
    message: Option<String>,
}

impl RAssertion {
    /// Create a new assertion on `param` with its requirement and R condition expression.
    fn new(
        param: impl Into<String>,
        requirement: impl Into<String>,
        condition: impl Into<String>,
    ) -> Self {
        Self {
            param: param.into(),
            requirement: requirement.into(),
            condition: condition.into(),
            message: None,
        }
    }

    /// Use `message` verbatim as the condition message of a failure.
    fn with_message(mut self, message: Option<&String>) -> Self {
        self.message = message.cloned();
        self
    }

    /// The condition message a failure raises: the author's message, else
    /// `'<param>' <requirement>`.
    #[cfg(test)]
    fn message(&self) -> String {
        match &self.message {
            Some(message) => message.clone(),
            None => format!("'{}' {}", self.param, self.requirement),
        }
    }

    /// Format as a guard that raises an argument error on failure:
    /// `if (!isTRUE(condition)) .miniextendr_arg_error("param", "requirement")`,
    /// or with the author's message
    /// `if (!isTRUE(condition)) .miniextendr_arg_error("param", message = "...")`.
    ///
    /// `call` is the call the error is attributed to. `None` leaves the
    /// helper's default, the wrapper's own call (what `stopifnot()` reported);
    /// a `call = caller` wrapper passes `.mx_call` (#1548). Next to a named
    /// `message` the call is named too: positionally it would bind to `what`.
    fn to_guard(&self, call: Option<&str>) -> String {
        match &self.message {
            None => {
                let call_arg = call.map(|c| format!(", {c}")).unwrap_or_default();
                format!(
                    "if (!isTRUE({})) .miniextendr_arg_error(\"{}\", \"{}\"{})",
                    self.condition, self.param, self.requirement, call_arg
                )
            }
            Some(message) => {
                let call_arg = call.map(|c| format!(", call = {c}")).unwrap_or_default();
                format!(
                    "if (!isTRUE({})) .miniextendr_arg_error(\"{}\", message = \"{}\"{})",
                    self.condition,
                    self.param,
                    r_string_escape(message),
                    call_arg
                )
            }
        }
    }

    /// Wrap for nullable: prepend `is.null(param) || ` to the condition,
    /// and adjust the requirement to mention NULL.
    fn nullable(self) -> Self {
        let requirement = if let Some(rest) = self.requirement.strip_prefix("must be ") {
            // "must be character" → "must be NULL or character"
            format!("must be NULL or {rest}")
        } else if let Some(rest) = self.requirement.strip_prefix("must have ") {
            // "must have length 1" → "must be NULL or have length 1"
            format!("must be NULL or have {rest}")
        } else {
            format!("{} (or NULL)", self.requirement)
        };
        Self {
            condition: format!("is.null({}) || {}", self.param, self.condition),
            param: self.param,
            requirement,
            message: self.message,
        }
    }
}

/// Per-function knobs that influence precondition codegen.
///
/// Coercion preserves numeric input types, extends `bool` / `Vec<bool>`
/// checks to accept integers as well as logicals, widens the native
/// `i32` / `Vec<i32>` gates to whole-number doubles and the native `f64` /
/// `Vec<f64>` gates to every numeric source, in the `Option<T>` and
/// `Vec<Option<T>>` forms too (see `coerce_widened`). Other types keep their checks.
/// Strict input conversion remains enforced in Rust, where range and precision
/// failures can carry contextual diagnostics.
#[derive(Clone, Default)]
pub struct PreconditionOptions {
    /// `coerce` knob is active for all parameters (`coerce_all`).
    pub coerce_all: bool,
    /// R-normalized names of parameters with a per-param `coerce` attribute.
    pub coerce_params: HashSet<String>,
    /// Checks the author named per parameter (`inherits`, `not_inherits`,
    /// `no_na`), keyed by R-normalized parameter name.
    pub explicit: HashMap<String, ExplicitChecks>,
    /// R-normalized names of the parameters whose type-derived checks are
    /// dropped (each one resolved by [`resolve_type_checks`]). Their
    /// [`explicit`](Self::explicit) checks are still emitted.
    pub unchecked: HashSet<String>,
    /// The R formal of each parameter (#1842): the name every check, every
    /// argument error and every map above uses. Without a declared formal,
    /// the Rust name without leading underscores.
    pub formals: crate::r_wrapper_builder::Formals,
}

impl PreconditionOptions {
    /// Returns `true` if the parameter `r_name` is coerced (function-wide or
    /// per-param).
    fn is_coerced(&self, r_name: &str) -> bool {
        self.coerce_all || self.coerce_params.contains(r_name)
    }
}

// region: which parameters keep their type-derived checks (#1566 §2)

/// Whether one parameter keeps its type-derived R-side checks, from every
/// spelling that can decide it, most specific first; the first that speaks
/// wins:
///
/// 1. `param`: a `Checked<T>` / `Unchecked<T>` marker, the per-parameter
///    `preconditions` / `no_preconditions`, or a method's
///    `preconditions(x)` / `no_preconditions(x)` list (merged at parse time,
///    where a marker and a keyword that disagree are an error);
/// 2. `item`: the function's or method's own `preconditions` /
///    `no_preconditions`;
/// 3. `impl_level`: the impl block's (`None` for a free function);
/// 4. `crate_default`: `[package.metadata.miniextendr] preconditions`;
/// 5. `no_preconditions_default`: the `no-preconditions-default` feature;
/// 6. otherwise the checks are kept.
///
/// `Some(true)` everywhere means "keep".
pub(crate) fn resolve_type_checks(
    param: Option<bool>,
    item: Option<bool>,
    impl_level: Option<bool>,
    crate_default: Option<bool>,
    no_preconditions_default: bool,
) -> bool {
    param.unwrap_or_else(|| {
        resolve_item_type_checks(item, impl_level, crate_default, no_preconditions_default)
    })
}

/// [`resolve_type_checks`] without a per-parameter spelling: whether an item
/// (function or method) keeps its type-derived checks by default, ranks 2 to
/// 6. The condition-raise codegen keys on this item-level decision.
pub(crate) fn resolve_item_type_checks(
    item: Option<bool>,
    impl_level: Option<bool>,
    crate_default: Option<bool>,
    no_preconditions_default: bool,
) -> bool {
    item.or(impl_level)
        .or(crate_default)
        .unwrap_or(!no_preconditions_default)
}

/// The R formals (`formals`) of the parameters of `inputs` whose
/// type-derived checks are dropped, given each one's rank-1 decision
/// (`param`, keyed by Rust name) and the item, impl and crate defaults
/// ([`resolve_type_checks`]; the feature is this macro crate's
/// `no-preconditions-default`).
pub(crate) fn unchecked_params(
    inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
    formals: &crate::r_wrapper_builder::Formals,
    param: impl Fn(&str) -> Option<bool>,
    item: Option<bool>,
    impl_level: Option<bool>,
    crate_default: Option<bool>,
) -> HashSet<String> {
    inputs
        .iter()
        .filter_map(|arg| {
            let syn::FnArg::Typed(pt) = arg else {
                return None;
            };
            let syn::Pat::Ident(pat_ident) = pt.pat.as_ref() else {
                return None;
            };
            let name = crate::naming::ident_name(&pat_ident.ident);
            let keep = resolve_type_checks(
                param(&name),
                item,
                impl_level,
                crate_default,
                cfg!(feature = "no-preconditions-default"),
            );
            (!keep).then(|| formals.of(&name))
        })
        .collect()
}

/// Whether a parameter type has type-derived R-side checks at all, i.e.
/// whether `Checked` / `Unchecked` and the per-parameter keywords have
/// anything to keep or drop (`SEXP`, `Missing<T>`, `ExternalPtr<T>`,
/// `&Dots` and unknown types have none).
pub(crate) fn has_type_check(ty: &syn::Type) -> bool {
    r_check_for_type(ty).is_some()
}

// endregion

/// R-side checks the author asked for by name on one parameter, rather than
/// ones derived from its Rust type.
///
/// Spelled `#[miniextendr(inherits = "cls", not_inherits = "bad", no_na)]` on
/// a standalone fn parameter, or `inherits(x = "cls")` /
/// `not_inherits(x = "bad")` / `no_na(x)` on an impl or trait method. They
/// are the same kind of guards as the type checks, raising the same argument
/// error. The NA check runs first, so an argument holding `NA` gets its
/// message (`'x' must not be NA`) whatever its class, type or range, the
/// checks that would otherwise refuse a typed `NA` (an `NA` is logical, and
/// fails a range check) before it. The class checks run next, before the
/// parameter's type checks, so a value of the wrong class gets the class
/// message whatever its type: `inherits` first, then `not_inherits` (#1815). That order matters for the classes a
/// `not_inherits` typically refuses: `is.numeric()` is `FALSE` for a
/// `difftime`, `Date` or `POSIXt`, so after an `AsNumeric` type check the
/// refusal would never be reached. An `inherits` message also replaces the
/// messages of the parameter's type checks; a `not_inherits` message does
/// not, since a value that passes it is of none of the refused classes and is
/// judged by its type. All survive `no_preconditions`: the Rust conversion does not
/// repeat them, so dropping them would change what the function accepts.
///
/// `no_na` also has a Rust half. A type can read more inputs as missing than
/// `anyNA()` sees (the reading markers `AsNumeric*` / `AsCharacter*` read the
/// text `"NA"`, blank strings, a factor `NA` level as `NA`), so the C wrapper
/// checks the converted value too (`TryFromSexp::__mx_has_na`, see
/// `RustConversionBuilder::with_no_na`) and raises the same condition with
/// the same message ([`no_na_message`]). An `Either` parameter has only the
/// Rust half: the arm the value converted to checks the input
/// ([`no_na_checked_after_conversion`]).
///
/// Each check can carry the author's own condition message
/// (`inherits(class = "cls", message = "...")`,
/// `not_inherits(class = "bad", message = "...")`, `no_na(message = "...")`;
/// method level `inherits(x(class = "cls", message = "..."))`,
/// `not_inherits(x(class = "bad", message = "..."))`,
/// `no_na(x(message = "..."))`), used verbatim in place of the generated
/// `'x' must inherit from 'cls'`. The condition is otherwise the same.
///
/// An `inherits` check can also give a value that fails it a message chosen
/// by the value's class (#1824): `inherits(class = "cls",
/// when(class = "data.frame", message = "..."))`, method level
/// `inherits(x(class = "cls", when(class = "data.frame", message = "...")))`
/// ([`InheritsHint`]).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ExplicitChecks {
    /// `inherits = "cls"` / `inherits("a", "b")`: the argument must inherit
    /// from at least one of these classes (`inherits(x, c(...))`).
    pub inherits: Option<Vec<String>>,
    /// `message = "..."` in `inherits(...)`: the message of a failed class
    /// check, for all of its classes, and of that parameter's R type checks.
    /// Only set together with `inherits`.
    pub inherits_message: Option<String>,
    /// `when(class = "...", message = "...")` in `inherits(...)`: the
    /// messages a value that fails the `inherits` check gets for its class,
    /// in the order written. Only set together with `inherits`.
    pub inherits_hints: Vec<InheritsHint>,
    /// `not_inherits = "cls"` / `not_inherits("a", "b")`: the argument must
    /// inherit from none of these classes (`!inherits(x, c(...))`, #1815).
    pub not_inherits: Option<Vec<String>>,
    /// `message = "..."` in `not_inherits(...)`: the message of a failed
    /// `not_inherits` check, for all of its classes. Unlike the `inherits`
    /// message it leaves the type checks' messages alone. Only set together
    /// with `not_inherits`.
    pub not_inherits_message: Option<String>,
    /// `no_na`: the argument must not contain `NA` (`!anyNA(x)`, so `NaN`
    /// is refused too, as `is.na()` does), nor a value its type reads as `NA`
    /// (checked in Rust after the conversion). For an `Either`, both checks
    /// run in Rust, by the arm the value converted to.
    pub no_na: bool,
    /// `no_na(message = "...")`: the message of a failed NA check. Only set
    /// together with `no_na`.
    pub no_na_message: Option<String>,
}

impl ExplicitChecks {
    /// Whether any check is requested.
    pub fn is_empty(&self) -> bool {
        self.inherits.is_none() && self.not_inherits.is_none() && !self.no_na
    }

    /// Merge `other` into `self` (a parameter may carry several attributes).
    /// The `inherits` hints of `other` follow those of `self`, so the hints
    /// keep the order they are written in.
    ///
    /// Fails when both give a message for the same check; the error names
    /// the check.
    pub fn merge(&mut self, other: ExplicitChecks) -> Result<(), String> {
        merge_message(
            &mut self.inherits_message,
            other.inherits_message,
            "inherits",
        )?;
        merge_message(
            &mut self.not_inherits_message,
            other.not_inherits_message,
            "not_inherits",
        )?;
        merge_message(&mut self.no_na_message, other.no_na_message, "no_na")?;
        if let Some(classes) = other.inherits {
            self.inherits.get_or_insert_with(Vec::new).extend(classes);
        }
        self.inherits_hints.extend(other.inherits_hints);
        if let Some(classes) = other.not_inherits {
            self.not_inherits
                .get_or_insert_with(Vec::new)
                .extend(classes);
        }
        self.no_na |= other.no_na;
        Ok(())
    }

    /// The error for a parameter whose checks contradict each other: a class
    /// both in `inherits` and in `not_inherits`, which no argument could
    /// satisfy through that class, or an `inherits` class in a `when(...)`
    /// hint, whose message could never be shown (a value of that class
    /// passes `inherits`). `None` when they agree.
    ///
    /// A hint class may also be a `not_inherits` class: a value of that
    /// class alone gets the hint, and one that also inherits from a required
    /// class passes the hint and `inherits` and is refused by `not_inherits`.
    ///
    /// Run once every attribute of the parameter is merged: the
    /// `inherits = "cls"` spelling adds its class without a merge, and a
    /// hint can name a class that another attribute requires.
    pub fn class_conflict(&self, param: &str) -> Option<String> {
        let required = self.inherits.as_deref().unwrap_or_default();
        if let Some(class) = self
            .not_inherits
            .iter()
            .flatten()
            .find(|c| required.contains(c))
        {
            return Some(format!(
                "class `{class}` is in both `inherits` and `not_inherits` on parameter `{param}`; \
                 name it in only one of them"
            ));
        }
        let class = self
            .inherits_hints
            .iter()
            .flat_map(|hint| &hint.classes)
            .find(|c| required.contains(c))?;
        Some(format!(
            "class `{class}` is in both `inherits` and a `when(...)` hint on parameter \
             `{param}`; a value of that class passes `inherits`, so the hint would never be \
             shown: drop it from `when(...)`"
        ))
    }

    /// The class checks (`inherits`, then `not_inherits`) for parameter
    /// `param` of type `ty`, which run before the parameter's type checks.
    /// Empty without either.
    ///
    /// The `inherits` hints come first, one check each in the order written,
    /// so a refused value gets the first hint naming one of its classes:
    /// `inherits(p, <required>) || !inherits(p, <hint classes>)`, failing
    /// with the hint's message. A value that passes `inherits` passes every
    /// hint, so a hint changes the message of a refusal, never what is
    /// accepted.
    ///
    /// An `Option<T>` / `Missing<T>` parameter passes `NULL` / an omitted
    /// argument (see [`guarded`]).
    fn class_assertions(&self, param: &str, ty: &syn::Type) -> Vec<RAssertion> {
        let mut out = Vec::new();
        if let Some(classes) = &self.inherits {
            let (what, class_arg) = class_list_r(classes);
            for hint in &self.inherits_hints {
                let (_, hint_arg) = class_list_r(&hint.classes);
                out.push(
                    RAssertion::new(
                        param,
                        format!("must inherit from {what}"),
                        format!("inherits({param}, {class_arg}) || !inherits({param}, {hint_arg})"),
                    )
                    .with_message(Some(&hint.message)),
                );
            }
            out.push(
                RAssertion::new(
                    param,
                    format!("must inherit from {what}"),
                    format!("inherits({param}, {class_arg})"),
                )
                .with_message(self.inherits_message.as_ref()),
            );
        }
        if let Some(classes) = &self.not_inherits {
            let (what, class_arg) = class_list_r(classes);
            out.push(
                RAssertion::new(
                    param,
                    format!("must not inherit from {what}"),
                    format!("!inherits({param}, {class_arg})"),
                )
                .with_message(self.not_inherits_message.as_ref()),
            );
        }
        guarded(param, ty, out)
    }

    /// The NA check (`no_na`) for parameter `param` of type `ty`, which runs
    /// before the parameter's other checks, with its own message. Empty without
    /// `no_na`, and for an `Either` parameter that is not a choice parameter
    /// (`choice`: `match_arg` / `choices`), whose NA check runs in Rust after
    /// the conversion ([`no_na_checked_after_conversion`]).
    ///
    /// An `Option<T>` / `Missing<T>` parameter passes `NULL` / an omitted
    /// argument (see [`guarded`]).
    fn value_assertions(&self, param: &str, ty: &syn::Type, choice: bool) -> Vec<RAssertion> {
        let mut out = Vec::new();
        if self.no_na && (choice || !no_na_checked_after_conversion(ty)) {
            out.push(
                RAssertion::new(param, no_na_requirement(ty), format!("!anyNA({param})"))
                    .with_message(self.no_na_message.as_ref()),
            );
        }
        guarded(param, ty, out)
    }

    /// The message of the parameter's R type checks: the `inherits` message,
    /// when there is one. Without it the type checks keep their generated
    /// messages. A `not_inherits` message is not one: a value that passed
    /// `not_inherits` is of none of the refused classes, so its type check
    /// keeps its own wording.
    fn type_check_message(&self) -> Option<&String> {
        self.inherits_message.as_ref()
    }
}

/// One `when(class = "...", message = "...")` hint of an `inherits` check
/// (#1824): the message a value gets when it fails the check and inherits
/// from one of `classes`, in place of the check's own. A `data.frame` passed
/// where a model object is expected can be told which function takes a data
/// frame, while `NULL` or a number gets the plain refusal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InheritsHint {
    /// The classes the hint is for (any one of them).
    pub classes: Vec<String>,
    /// The condition message, used verbatim.
    pub message: String,
}

/// A class check's classes in R: the quoted list for its message
/// (`'a', 'b' or 'c'`) and the argument of `inherits()` (`"a"` or
/// `c("a", "b", "c")`), each class escaped as an R string.
fn class_list_r(classes: &[String]) -> (String, String) {
    let quoted: Vec<String> = classes
        .iter()
        .map(|c| format!("'{}'", r_string_escape(c)))
        .collect();
    let literals: Vec<String> = classes
        .iter()
        .map(|c| format!("\"{}\"", r_string_escape(c)))
        .collect();
    let what = match quoted.as_slice() {
        [one] => one.clone(),
        [init @ .., last] => format!("{} or {last}", init.join(", ")),
        [] => unreachable!("a class check is parsed as a non-empty class list"),
    };
    let class_arg = match literals.as_slice() {
        [one] => one.clone(),
        _ => format!("c({})", literals.join(", ")),
    };
    (what, class_arg)
}

/// Let an `Option<T>` parameter pass `NULL`, and a `Missing<T>` parameter an
/// omitted argument, in the named checks `out` on parameter `param` of type
/// `ty`: each condition gains an `is.null(param) || ` / `missing(param) || `
/// prefix. The failure message names only the failed requirement: it is shown
/// only for a present, non-`NULL` value.
fn guarded(param: &str, ty: &syn::Type, mut out: Vec<RAssertion>) -> Vec<RAssertion> {
    let guard = if crate::miniextendr_fn::is_missing_type(ty) {
        Some(format!("missing({param})"))
    } else if crate::type_inspect::is_option_type(ty) {
        Some(format!("is.null({param})"))
    } else {
        None
    };
    if let Some(guard) = guard {
        for a in &mut out {
            a.condition = format!("{guard} || {}", a.condition);
        }
    }
    out
}

/// Whether a `no_na` parameter of Rust type `ty` is checked for `NA` only
/// in Rust, after the conversion, with no R guard: an `Either<L, R>`, also
/// under `Missing` and `Option`. The layers are peeled as for a choice
/// parameter ([`crate::type_inspect::choice_layers`]). A plain
/// `Option<Either<..>>` converts through `TryFromSexp`, which reads `NULL` as
/// `None` (no `NA`) and asks the `Either` otherwise; a choice
/// `Option<Either<Mode, X>>` keeps the guard (see below). The type is read
/// as written: an alias, a newtype or a `Result<Either<..>, ()>` keeps the
/// guard.
///
/// One `!anyNA(x)` on the whole argument is wrong for an arm with its own
/// reading of `NA` (a data frame arm whose cells may be missing), and it
/// scans the input a second time. So the C wrapper asks the arm the value
/// converted to (`TryFromSexp::__mx_input_has_na`): by default the arm runs
/// `anyNA()`'s check on the input in Rust, and a `DataFrame` arm refuses
/// nothing. A choice parameter (`match_arg` / `choices`) keeps the R guard:
/// its conversion does not go through `TryFromSexp`, so no Rust check follows.
pub(crate) fn no_na_checked_after_conversion(ty: &syn::Type) -> bool {
    crate::type_inspect::choice_layers(ty)
        .either_right
        .is_some()
}

/// What a `no_na` parameter of Rust type `ty` must satisfy, without the
/// parameter name: `must not contain NA` for a value holding several values
/// ([`no_na_holds_several`]), else `must not be NA`.
///
/// Shared by the R guard and the Rust check that the C wrapper runs after the
/// conversion (see [`no_na_message`]), so both word a failure alike.
pub(crate) fn no_na_requirement(ty: &syn::Type) -> String {
    let verb = if no_na_holds_several(ty) {
        "contain"
    } else {
        "be"
    };
    format!("must not {verb} NA")
}

/// Whether a `no_na` value of Rust type `ty` holds several values, looking
/// through the layers `no_na` checks through: `Missing`, `Option`,
/// `Result<T, _>` (its `T`) and `Either<L, R>`, which holds several when
/// either arm does. The text is fixed at compile time, before the conversion
/// picks an arm, and `contain` also reads right for one value.
fn no_na_holds_several(ty: &syn::Type) -> bool {
    let ty = crate::miniextendr_fn::get_missing_inner_type(ty).unwrap_or(ty);
    let ty = crate::type_inspect::option_inner_type(ty).unwrap_or(ty);
    if let Some((left, right)) = crate::type_inspect::either_arms(ty) {
        return no_na_holds_several(left) || no_na_holds_several(right);
    }
    if let syn::Type::Path(tp) = ty
        && let Some(seg) = tp.path.segments.last()
        && seg.ident == "Result"
        && let Some(ok) = crate::type_inspect::first_type_argument(seg)
    {
        return no_na_holds_several(ok);
    }
    is_vector_valued(ty)
}

/// The condition message of a failed `no_na` check on parameter `param` of
/// type `ty`: the author's `no_na(message = "...")` verbatim (`custom`), else
/// `'<param>' must not be NA` / `'<param>' must not contain NA`
/// ([`no_na_requirement`]). The R guard builds the same text in R
/// (`.miniextendr_arg_error`); the C wrapper's post-conversion check passes
/// this one to `arg_check_condition_value`.
pub(crate) fn no_na_message(param: &str, ty: &syn::Type, custom: Option<&str>) -> String {
    match custom {
        Some(message) => message.to_string(),
        None => format!("'{param}' {}", no_na_requirement(ty)),
    }
}

/// Whether an argument of Rust type `ty` holds several values, so a `no_na`
/// failure reads `must not contain NA` rather than `must not be NA`.
///
/// The type table decides when it knows the type ([`RTypeCheck::is_vector`]):
/// `Vec<T>` and slices, the vector markers (`AsNumericVec`, `AsCharacterVec`,
/// any marker whose table entry is a vector check), maps and lists. For a type the
/// table does not know, the shape decides: `Vec<T>`, `Box<[T]>`, slices and
/// arrays, and a type named `...Vec` (the vector half of a marker family such
/// as `AsFromStrVec` / `AsDisplayVec`, or a vector type like `BitVec`).
fn is_vector_valued(ty: &syn::Type) -> bool {
    if let Some(check) = r_check_for_type(ty) {
        return check.is_vector();
    }
    crate::miniextendr_fn::is_vector_like_type(ty)
        || matches!(
            ty,
            syn::Type::Path(tp)
                if tp.path.segments.last().is_some_and(|s| s.ident.to_string().ends_with("Vec"))
        )
}

/// Record `from` as the message of `check`, which may have only one.
fn merge_message(
    into: &mut Option<String>,
    from: Option<String>,
    check: &str,
) -> Result<(), String> {
    match (into.is_some(), from) {
        (true, Some(_)) => Err(format!(
            "`{check}` is given more than one `message` on this parameter; give it once"
        )),
        (false, Some(message)) => {
            *into = Some(message);
            Ok(())
        }
        (_, None) => Ok(()),
    }
}

/// Classification of an R-side type check for a function parameter.
///
/// Each variant maps to a specific set of R-side checks. Numeric checks
/// use a broad predicate (`is.numeric || is.logical || is.raw`) because R coerces
/// logical to numeric freely and raw to integer is valid for byte-sized types.
/// Borderline cases (e.g., raw to i64 in strict mode) pass the precondition and
/// reach Rust's strict checker, which produces better contextual error messages.
enum RTypeCheck {
    /// Numeric scalar: type check + length-1 check (2 assertions).
    /// Used for the multi-source scalars `f32`, `i8`, `i16`, `i64`, `isize`, and
    /// for a coerced native `f64`. Native `i32` / `f64` without coerce use
    /// [`RTypeCheck::Scalar`] with `"integer"` / `"double"`.
    ScalarNumeric,
    /// Coerced bool: logical or integer, with length one.
    ScalarLogicalOrInteger,
    /// Coerced bool vector: logical or integer.
    VectorLogicalOrInteger,
    /// Coerced native `i32` scalar: integer, logical, raw, or a whole-number
    /// double, with length one. The vector form is [`RTypeCheck::VectorIntegerWide`].
    ScalarIntegerWide,
    /// Non-negative numeric scalar: type + length-1 + `>= 0` (3 assertions).
    /// Used for `u16`, `u32`, `u64`, `usize`.
    ScalarNonNeg,
    /// Non-numeric scalar: `is.<type>(x)` + length-1 check (2 assertions).
    /// The string is the R type predicate name (e.g., `"logical"`, `"character"`).
    Scalar(&'static str),
    /// Floating-point numeric vector: loose `is.numeric || is.logical || is.raw`
    /// (1 assertion). Used for `Vec<f32>` (multi-source converter) and for a
    /// coerced native `Vec<f64>`; `Vec<f64>` / `&[f64]` without coerce use
    /// [`RTypeCheck::Vector`] with `"double"` because the native path reads
    /// REALSXP only.
    VectorNumeric,
    /// **INTSXP-only** integer vector: `is.integer(x)` (1 assertion). Used *only*
    /// for `Vec<i32>` / `&[i32]` (issue #616). These use the native `RNativeType`
    /// inbound path (`impl_vec_try_from_sexp_native!(i32)`), which requires
    /// `INTSXP` and rejects `REALSXP` outright. The loose `is.numeric` predicate
    /// previously let a `double` like `c(1, 2)` pass the R gate only to fail with
    /// a cryptic "expected INTSXP, got REALSXP". `is.integer(x)` rejects every
    /// `double` (whole or fractional) at the boundary with a clean message,
    /// matching the actual Rust behaviour and closing the silent-truncation gap.
    VectorIntegerStrict,
    /// **Wide** integer vector accepting `REALSXP` whole-number values: the
    /// lossless whole-number predicate (1 assertion). Used for every non-`i32`
    /// integer element type — `Vec<i8>` / `Vec<i16>` / `Vec<u16>` / `Vec<u32>`
    /// and the 64-bit family `Vec<i64>` / `Vec<u64>` / `Vec<isize>` / `Vec<usize>`
    /// (issue #616). These use the coercing inbound path
    /// (`impl_vec_try_from_sexp_numeric!` → `from_numeric_vec_with`) which accepts
    /// INTSXP/REALSXP/RAWSXP/LGLSXP and rejects fractional doubles element-wise
    /// (`f64: TryCoerce<T>` checks `self.fract() != 0`). We accept
    /// integer/logical/raw **and** whole-number doubles, and reject genuinely
    /// lossy fractional doubles (`1.5`) at the boundary.
    VectorIntegerWide,
    /// Non-numeric vector: `is.<type>(x)` only (1 assertion).
    /// The string is the R type predicate name.
    Vector(&'static str),
    /// `AsNumeric` scalar: numeric, logical, character, or factor, with length
    /// one (2 assertions). The marker parses character and factor labels the way
    /// `as.numeric()` does, so the gate admits exactly the types it reads.
    ScalarNumericOrText,
    /// `AsNumericVec`: numeric, logical, character, or factor (1 assertion).
    VectorNumericOrText,
    /// Nullable wrapper around an inner check: prepends `is.null(x) ||` to each assertion
    /// and adjusts messages to mention NULL.
    Nullable(Box<RTypeCheck>),
    /// List check: `is.list(x)` (1 assertion).
    /// Used for `HashMap`, `BTreeMap`, `NamedList`, `List`, `ListMut`.
    List,
}

/// Build the R expression for the numeric type predicate.
///
/// Returns `"is.numeric(p) || is.logical(p) || is.raw(p)"` for a given parameter `p`.
/// This broad predicate matches R's coercion rules: logical coerces to numeric freely,
/// and raw is accepted because it represents byte-level data.
fn numeric_type_check(param: &str) -> String {
    format!(
        "is.numeric({p}) || is.logical({p}) || is.raw({p})",
        p = param
    )
}

/// Build the lossless whole-number predicate for a `REALSXP`-accepting integer
/// vector (the `i64` / `u64` / `isize` / `usize` family).
///
/// Accepts integer/logical/raw and whole-number doubles, rejects fractional
/// doubles. The whole-number test is NA-safe (`is.na(p) | p == trunc(p)`):
/// `NA_real_` passes (maps to `NA_integer_`), `1.5` fails. `Inf`/`NaN` satisfy
/// `x == trunc(x)` in R but are caught by the Rust-side range/NaN conversion
/// check instead.
fn integer_vector_wide_check(param: &str) -> String {
    format!(
        "is.integer({p}) || is.logical({p}) || is.raw({p}) || \
         (is.numeric({p}) && all(is.na({p}) | {p} == trunc({p})))",
        p = param
    )
}

/// Build the predicate for the `AsNumeric` / `AsNumericVec` markers.
///
/// `is.numeric()` is `FALSE` for a factor, so `is.factor()` is listed on its
/// own: the marker reads a factor by its labels. Raw and complex stay out, as
/// the marker refuses them.
fn numeric_or_text_check(param: &str) -> String {
    format!(
        "is.numeric({p}) || is.logical({p}) || is.character({p}) || is.factor({p})",
        p = param
    )
}

impl RTypeCheck {
    /// Produce the individual checks for this type check.
    ///
    /// Returns one or more `RAssertion` values, each one requirement on
    /// `param` (the R parameter name used in the conditions and messages).
    fn assertions(&self, param: &str) -> Vec<RAssertion> {
        let length_one = || {
            RAssertion::new(
                param,
                "must have length 1",
                format!("length({param}) == 1L"),
            )
        };
        match self {
            RTypeCheck::ScalarNumeric => vec![
                RAssertion::new(
                    param,
                    "must be numeric, logical, or raw",
                    numeric_type_check(param),
                ),
                length_one(),
            ],
            RTypeCheck::ScalarLogicalOrInteger => vec![
                RAssertion::new(
                    param,
                    "must be logical or integer",
                    format!("is.logical({p}) || is.integer({p})", p = param),
                ),
                length_one(),
            ],
            RTypeCheck::VectorLogicalOrInteger => vec![RAssertion::new(
                param,
                "must be logical or integer",
                format!("is.logical({p}) || is.integer({p})", p = param),
            )],
            RTypeCheck::ScalarIntegerWide => vec![
                RAssertion::new(
                    param,
                    "must be integer or whole-number numeric",
                    integer_vector_wide_check(param),
                ),
                length_one(),
            ],
            RTypeCheck::ScalarNonNeg => vec![
                RAssertion::new(
                    param,
                    "must be numeric, logical, or raw",
                    numeric_type_check(param),
                ),
                length_one(),
                RAssertion::new(
                    param,
                    "must be non-negative",
                    // raw is always non-negative; guard with is.raw() to avoid
                    // "comparison not implemented" error for raw values
                    format!("is.raw({p}) || {p} >= 0", p = param),
                ),
            ],
            RTypeCheck::Scalar(r_type) => vec![
                RAssertion::new(
                    param,
                    format!("must be {r_type}"),
                    format!("is.{r_type}({param})"),
                ),
                length_one(),
            ],
            RTypeCheck::VectorNumeric => vec![RAssertion::new(
                param,
                "must be numeric, logical, or raw",
                numeric_type_check(param),
            )],
            RTypeCheck::VectorIntegerStrict => vec![RAssertion::new(
                param,
                "must be an integer vector",
                format!("is.integer({param})"),
            )],
            RTypeCheck::VectorIntegerWide => vec![RAssertion::new(
                param,
                "must be integer or whole-number numeric",
                integer_vector_wide_check(param),
            )],
            RTypeCheck::Vector(r_type) => vec![RAssertion::new(
                param,
                format!("must be {r_type}"),
                format!("is.{r_type}({param})"),
            )],
            RTypeCheck::ScalarNumericOrText => vec![
                RAssertion::new(
                    param,
                    "must be numeric, logical, character, or factor",
                    numeric_or_text_check(param),
                ),
                length_one(),
            ],
            RTypeCheck::VectorNumericOrText => vec![RAssertion::new(
                param,
                "must be numeric, logical, character, or factor",
                numeric_or_text_check(param),
            )],
            RTypeCheck::Nullable(inner) => inner
                .assertions(param)
                .into_iter()
                .map(RAssertion::nullable)
                .collect(),
            RTypeCheck::List => vec![RAssertion::new(
                param,
                "must be a list",
                format!("is.list({param})"),
            )],
        }
    }

    /// Whether the check admits several values: the vector checks and
    /// [`RTypeCheck::List`], through [`RTypeCheck::Nullable`]. Matched
    /// exhaustively, so a new check has to say which it is.
    fn is_vector(&self) -> bool {
        match self {
            RTypeCheck::VectorLogicalOrInteger
            | RTypeCheck::VectorNumeric
            | RTypeCheck::VectorIntegerStrict
            | RTypeCheck::VectorIntegerWide
            | RTypeCheck::Vector(_)
            | RTypeCheck::VectorNumericOrText
            | RTypeCheck::List => true,
            RTypeCheck::ScalarNumeric
            | RTypeCheck::ScalarLogicalOrInteger
            | RTypeCheck::ScalarIntegerWide
            | RTypeCheck::ScalarNonNeg
            | RTypeCheck::Scalar(_)
            | RTypeCheck::ScalarNumericOrText => false,
            RTypeCheck::Nullable(inner) => inner.is_vector(),
        }
    }

    /// What an argument of this type must be, in R terms: the `<expected>` of
    /// a Rust conversion failure's message, `'<p>' must be <expected>: <reason>`
    /// (#1591).
    ///
    /// The R-side checks test one requirement at a time (type, then length);
    /// a conversion failure names the whole expectation once. Scalars read
    /// `a single <noun>` (`a single integer`, `a single string`, `TRUE or
    /// FALSE`), vectors name the R type (`numeric`, `character`, `a list`).
    /// `AsNumeric` / `AsNumericVec` say `a single number` / `numeric`, the
    /// value they produce, rather than the inputs they read. `AsCharacter` /
    /// `AsCharacterVec` likewise say `coercible to a single string` /
    /// `coercible to character`: their R-side check has already required an
    /// atomic argument, so what is left to fail is the `as.character()` step
    /// (a failing method, a non-character or wrong-length result).
    fn expectation(&self) -> String {
        match self {
            RTypeCheck::ScalarNumeric | RTypeCheck::ScalarNumericOrText => "a single number".into(),
            RTypeCheck::ScalarLogicalOrInteger => "a single logical or integer".into(),
            RTypeCheck::VectorLogicalOrInteger => "logical or integer".into(),
            RTypeCheck::ScalarIntegerWide => "a single whole number".into(),
            RTypeCheck::ScalarNonNeg => "a single non-negative number".into(),
            RTypeCheck::Scalar(r_type) => match *r_type {
                "logical" => "TRUE or FALSE".into(),
                "character" => "a single string".into(),
                "integer" | "double" => format!("a single {r_type}"),
                "atomic" => "coercible to a single string".into(),
                // `raw`, `complex`.
                other => format!("a single {other} value"),
            },
            RTypeCheck::VectorNumeric | RTypeCheck::VectorNumericOrText => "numeric".into(),
            RTypeCheck::VectorIntegerStrict => "integer".into(),
            RTypeCheck::VectorIntegerWide => "integer or whole-number numeric".into(),
            RTypeCheck::Vector("atomic") => "coercible to character".into(),
            RTypeCheck::Vector(r_type) => (*r_type).into(),
            RTypeCheck::Nullable(inner) => match inner.expectation().as_str() {
                "TRUE or FALSE" => "NULL, TRUE or FALSE".into(),
                inner => format!("NULL or {inner}"),
            },
            RTypeCheck::List => "a list".into(),
        }
    }
}

/// The R-facing expectation for a parameter of Rust type `ty`, for the
/// message of its Rust conversion failure: `'<p>' must be <expected>: ...`
/// (see [`RTypeCheck::expectation`]).
///
/// Derived from the same type classification as the R-side checks, including
/// the `coerce` widening (`coerced`) and the `Missing<T>` / `Option<T>`
/// wrappers, so the two paths describe an argument the same way. A few types
/// without an R-side check still have an R-facing expectation
/// ([`unchecked_expectation`]): `Either<L, R>`, the `AsFromStr` markers and
/// tuples. `None` only for an opaque type (a custom `TryFromSexp` type, or an
/// `Either` with such an arm): the wrapper then asks the type at run time
/// (`rust_conversion_builder::declared_expectation`: a derived newtype's
/// declared expectation, an `Either` side by side), else reports
/// `invalid '<p>' argument: <reason>`.
pub(crate) fn conversion_expectation(ty: &syn::Type, coerced: bool) -> Option<String> {
    let ty = crate::miniextendr_fn::get_missing_inner_type(ty).unwrap_or(ty);
    let Some(check) = r_check_for_type(ty) else {
        return unchecked_expectation(ty);
    };
    let check = if coerced {
        coerce_widened(check, ty)
    } else {
        check
    };
    Some(check.expectation())
}

/// The R-facing expectation of a type that has no R-side check, so its
/// conversion alone says what it accepts (#1594):
///
/// | type | `<expected>` |
/// |------|--------------|
/// | `AsFromStr<T>` / `AsFromStrVec<T>` | `a single string` / `character` (the input `T` is parsed from) |
/// | `Either<L, R>` | `<L> or <R>`, when both sides have one (`a single integer or a single string`) |
/// | `(A, B, ...)` | `a list of length N` |
/// | `DataFrame` | `a data frame` |
/// | `Option<T>` of the above | `NULL or <T>` |
fn unchecked_expectation(ty: &syn::Type) -> Option<String> {
    match ty {
        syn::Type::Tuple(tuple) if !tuple.elems.is_empty() => {
            Some(format!("a list of length {}", tuple.elems.len()))
        }
        syn::Type::Path(type_path) => {
            let segment = type_path.path.segments.last()?;
            match segment.ident.to_string().as_str() {
                "AsFromStr" => Some("a single string".into()),
                "AsFromStrVec" => Some("character".into()),
                "DataFrame" => Some("a data frame".into()),
                "Either" => {
                    let left = crate::type_inspect::first_type_argument(segment)?;
                    let right = crate::type_inspect::second_type_argument(segment)?;
                    Some(format!(
                        "{} or {}",
                        conversion_expectation(left, false)?,
                        conversion_expectation(right, false)?
                    ))
                }
                "Option" => {
                    let inner = extract_single_generic_arg(segment)?;
                    Some(format!("NULL or {}", unchecked_expectation(inner)?))
                }
                _ => None,
            }
        }
        _ => None,
    }
}

/// The R-facing expectation of a `strict` lossy-integer parameter (`i64`,
/// `u64`, `isize`, `usize` and their `Vec` / `Vec<Option<_>>` forms), whose
/// Rust conversion accepts only integer and whole-number double input
/// (#1594): `a single whole number` / `a single non-negative whole number`
/// for a scalar, the vector's usual `integer or whole-number numeric`
/// otherwise.
pub(crate) fn strict_conversion_expectation(ty: &syn::Type) -> Option<String> {
    let scalar = match ty {
        syn::Type::Path(tp) => tp.path.segments.last().map(|s| s.ident.to_string()),
        _ => None,
    };
    match scalar.as_deref() {
        Some("i64" | "isize") => Some("a single whole number".into()),
        Some("u64" | "usize") => Some("a single non-negative whole number".into()),
        _ => conversion_expectation(ty, false),
    }
}

/// Keep the R gate in sync with the conversion actually selected for this type.
///
/// Native `i32` widens to whole-number doubles (plus logical/raw), so its gate
/// becomes the lossless whole-number predicate; `Vec<i32>` moves from the
/// INTSXP-only gate (#616) to the same wide predicate the other integer vectors
/// use. Native `f64` / `Vec<f64>` move from the `is.double` gate to the loose
/// numeric predicate. `Option<T>` of these types takes its scalar's widened
/// check under [`RTypeCheck::Nullable`], and `Vec<Option<T>>` its vector's
/// (#2274): the vector checks already let `NA` elements through, and a
/// `Vec<Option<T>>` reads `NULL` no more than a `Vec<T>` does.
fn coerce_widened(check: RTypeCheck, ty: &syn::Type) -> RTypeCheck {
    use crate::miniextendr_fn::CoercionMapping;
    let nullable = |inner| RTypeCheck::Nullable(Box::new(inner));
    match CoercionMapping::from_type(ty) {
        Some(CoercionMapping::Bool) => RTypeCheck::ScalarLogicalOrInteger,
        Some(CoercionMapping::OptionBool) => nullable(RTypeCheck::ScalarLogicalOrInteger),
        Some(CoercionMapping::BoolVec | CoercionMapping::OptionBoolVec) => {
            RTypeCheck::VectorLogicalOrInteger
        }
        Some(CoercionMapping::NativeInt) => RTypeCheck::ScalarIntegerWide,
        Some(CoercionMapping::OptionNativeInt) => nullable(RTypeCheck::ScalarIntegerWide),
        Some(CoercionMapping::NativeIntVec | CoercionMapping::OptionNativeIntVec) => {
            RTypeCheck::VectorIntegerWide
        }
        Some(CoercionMapping::NativeReal) => RTypeCheck::ScalarNumeric,
        Some(CoercionMapping::OptionNativeReal) => nullable(RTypeCheck::ScalarNumeric),
        Some(CoercionMapping::NativeRealVec | CoercionMapping::OptionNativeRealVec) => {
            RTypeCheck::VectorNumeric
        }
        Some(CoercionMapping::Numeric) | None => check,
    }
}

/// Map a Rust type to its R-side type check, if applicable.
///
/// Returns `None` for types that should skip precondition checks (SEXP, Dots, ExternalPtr, etc.).
fn r_check_for_type(ty: &syn::Type) -> Option<RTypeCheck> {
    match ty {
        syn::Type::Path(type_path) => r_check_for_type_path(type_path),
        syn::Type::Reference(type_ref) => r_check_for_reference(type_ref),
        _ => None,
    }
}

/// Map a `syn::TypePath` to its R-side type check.
///
/// Handles the most common case: simple types (`i32`, `String`, `bool`),
/// generic wrappers (`Vec<T>`, `Option<T>`), map types, and skip types.
/// Returns `None` for types that cannot be prechecked from R.
fn r_check_for_type_path(type_path: &syn::TypePath) -> Option<RTypeCheck> {
    let segment = type_path.path.segments.last()?;
    let ident = segment.ident.to_string();

    match ident.as_str() {
        // Native scalars convert from exactly one SEXPTYPE (`i32::try_from_sexp`
        // is INTSXP-only, `f64` REALSXP-only), so the R gate names that storage
        // instead of letting a double reach Rust and fail with "expected INTSXP,
        // got REALSXP" (#1112). Under `coerce` the gate widens with the conversion
        // (see `coerce_widened`).
        "i32" => Some(RTypeCheck::Scalar("integer")),
        "f64" => Some(RTypeCheck::Scalar("double")),

        // Non-native numeric scalars use the multi-source converters (integer,
        // double, logical, raw), so the loose predicate matches what Rust accepts.
        "f32" | "i8" | "i16" | "i64" | "isize" => Some(RTypeCheck::ScalarNumeric),

        // Unsigned numeric scalars (non-negative constraint)
        "u16" | "u32" | "u64" | "usize" => Some(RTypeCheck::ScalarNonNeg),

        // Logical scalar
        "bool" | "Rbool" | "Rboolean" => Some(RTypeCheck::Scalar("logical")),

        // Character scalar
        "String" | "char" | "PathBuf" => Some(RTypeCheck::Scalar("character")),

        // Raw scalar
        "u8" => Some(RTypeCheck::Scalar("raw")),

        // Complex scalar
        "Rcomplex" => Some(RTypeCheck::Scalar("complex")),

        // `as.numeric()`-style markers: numbers, text, and factor labels.
        "AsNumeric" => Some(RTypeCheck::ScalarNumericOrText),
        "AsNumericVec" => Some(RTypeCheck::VectorNumericOrText),

        // `as.character()`-style markers: any atomic vector (a list or data
        // frame is refused, as `as.character()` would flatten it), with length
        // one for the scalar. "'x' must be atomic" is base R's own wording.
        "AsCharacter" => Some(RTypeCheck::Scalar("atomic")),
        "AsCharacterVec" => Some(RTypeCheck::Vector("atomic")),

        // Option<T> → Nullable
        "Option" => {
            let inner_ty = extract_single_generic_arg(segment)?;
            r_check_for_type(inner_ty).map(|inner| RTypeCheck::Nullable(Box::new(inner)))
        }

        // `Cow<str>` converts as `&str` does, `Cow<[T]>` as `&[T]`. The first
        // generic argument is the lifetime, so look past it.
        "Cow" => match crate::type_inspect::first_type_argument(segment)? {
            syn::Type::Slice(slice) => r_check_for_vec_element(&slice.elem),
            inner if is_str(inner) => Some(RTypeCheck::Scalar("character")),
            _ => None,
        },

        // Vec<T> → Vector (depends on element type)
        "Vec" => {
            let inner_ty = extract_single_generic_arg(segment)?;
            r_check_for_vec_element(inner_ty)
        }

        // Map types and named list → List
        "HashMap" | "BTreeMap" | "NamedList" => Some(RTypeCheck::List),

        // List (bare) → List
        "List" | "ListMut" => Some(RTypeCheck::List),

        // Skip types: SEXP, Dots, Missing, ExternalPtr, RLogical, etc.
        // `LazyDots` is `...`, passed as the wrapper's frame (#1892).
        "SEXP" | "Dots" | "LazyDots" | "Missing" | "ExternalPtr" | "OwnedProtect" => None,

        // Unknown type → skip (let Rust side validate)
        _ => None,
    }
}

/// Map a reference type to its R-side type check.
///
/// Handles `&str` and `&Path` (character scalar), `&[T]` (vector based on element type),
/// and `&Dots` (skipped). Returns `None` for unrecognized reference types.
fn r_check_for_reference(type_ref: &syn::TypeReference) -> Option<RTypeCheck> {
    match type_ref.elem.as_ref() {
        // &str → character scalar
        syn::Type::Path(tp) => {
            let seg = tp.path.segments.last()?;
            match seg.ident.to_string().as_str() {
                "str" => Some(RTypeCheck::Scalar("character")),
                "Path" => Some(RTypeCheck::Scalar("character")),
                "Dots" => None,
                _ => None,
            }
        }
        // &[T] → vector check based on element type
        syn::Type::Slice(slice) => r_check_for_vec_element(&slice.elem),
        _ => None,
    }
}

/// Map a `Vec<T>` or `&[T]` element type to the appropriate vector type check.
///
/// Numeric elements produce `VectorNumeric`, `bool` produces `Vector("logical")`,
/// `String` produces `Vector("character")`, etc. Handles nested `Option<T>` for
/// nullable element types (e.g., `Vec<Option<String>>` becomes character vector).
fn r_check_for_vec_element(elem_ty: &syn::Type) -> Option<RTypeCheck> {
    let tp = match elem_ty {
        syn::Type::Path(tp) => tp,
        // `Vec<&str>` reads a character vector, as `Vec<String>` does.
        syn::Type::Reference(type_ref) if is_str(&type_ref.elem) => {
            return Some(RTypeCheck::Vector("character"));
        }
        _ => return None,
    };
    let seg = tp.path.segments.last()?;
    let ident = seg.ident.to_string();

    match ident.as_str() {
        // `Vec<f64>` / `&[f64]` read the REALSXP payload directly (native path),
        // so the gate is `is.double`; `Vec<f32>` coerces from every numeric source.
        "f64" => Some(RTypeCheck::Vector("double")),
        "f32" => Some(RTypeCheck::VectorNumeric),

        // `Vec<i32>` / `&[i32]` is the *only* INTSXP-only integer vector (issue
        // #616): its inbound conversion uses the native `RNativeType` path
        // (`impl_vec_try_from_sexp_native!(i32)` in `from_r/collections.rs`),
        // which rejects REALSXP outright. So the R gate is `is.integer` — a
        // `double` like `c(1, 2)` fails cleanly at the boundary instead of
        // producing a cryptic "expected INTSXP, got REALSXP". `u8` is handled
        // below as a raw vector.
        "i32" => Some(RTypeCheck::VectorIntegerStrict),

        // Every other integer-element vector (`i8`/`i16`/`u16`/`u32` and the
        // 64-bit family `i64`/`u64`/`isize`/`usize`) uses the coercing inbound
        // path (`impl_vec_try_from_sexp_numeric!` → `from_numeric_vec_with`),
        // which accepts INTSXP/REALSXP/RAWSXP/LGLSXP and rejects fractional
        // doubles element-wise via `f64: TryCoerce<T>` (the `self.fract() != 0`
        // check in `coerce.rs`). The R gate mirrors that: accept integer/logical/
        // raw + whole-number doubles, reject fractional. (64-bit ints also arrive
        // as REALSXP since R has no native 64-bit integer type.)
        "i8" | "i16" | "u16" | "u32" | "i64" | "u64" | "isize" | "usize" => {
            Some(RTypeCheck::VectorIntegerWide)
        }

        // Logical vector
        "bool" => Some(RTypeCheck::Vector("logical")),

        // Character vector; `Vec<Cow<str>>` reads one as `Vec<String>` does.
        "String" => Some(RTypeCheck::Vector("character")),
        "Cow" if crate::type_inspect::first_type_argument(seg).is_some_and(is_str) => {
            Some(RTypeCheck::Vector("character"))
        }

        // Raw vector
        "u8" => Some(RTypeCheck::Vector("raw")),

        // Complex vector
        "Rcomplex" => Some(RTypeCheck::Vector("complex")),

        // Vec<Option<T>> — e.g., Vec<Option<String>> for nullable strings
        "Option" => {
            let inner = extract_single_generic_arg(seg)?;
            // Vec<Option<String>> → character, Vec<Option<i32>> → numeric, etc.
            r_check_for_vec_element(inner)
        }

        _ => None,
    }
}

/// Whether `ty` is `str`.
fn is_str(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Path(tp) if tp.qself.is_none() && tp.path.is_ident("str"))
}

/// Extract the single generic type argument from a path segment.
///
/// e.g., `Option<String>` → `String`, `Vec<i32>` → `i32`
fn extract_single_generic_arg(segment: &syn::PathSegment) -> Option<&syn::Type> {
    if let syn::PathArguments::AngleBracketed(ref args) = segment.arguments
        && let Some(syn::GenericArgument::Type(ty)) = args.args.first()
    {
        return Some(ty);
    }
    None
}

/// A parameter whose Rust type is not in the static type table.
///
/// Currently, fallback params are recorded but no R-side validation is generated
/// for them -- the Rust-side conversion handles type errors with its own messages.
#[allow(dead_code)] // Read in tests
pub struct FallbackParam {
    /// R-normalized parameter name (e.g., `_dots` becomes `dots`).
    pub r_name: String,
}

/// Output of precondition analysis for a function's parameters.
///
/// Holds the R-side checks for known types and a list of parameters with
/// unknown types that were not statically prechecked.
pub struct PreconditionOutput {
    /// The checks, in parameter order: each parameter's NA check, its class
    /// checks, then its type checks ([`ExplicitChecks`]). Rendered by
    /// [`PreconditionOutput::guards`].
    assertions: Vec<RAssertion>,
    /// Parameters with unknown custom types that were not prechecked.
    #[allow(dead_code)] // Read in tests
    pub fallback_params: Vec<FallbackParam>,
}

impl PreconditionOutput {
    /// One guard line per check, raising an argument error through
    /// `.miniextendr_arg_error` when it fails (#1591); empty when there is
    /// nothing to check.
    ///
    /// `call` is the call every failure is attributed to: `None` for the
    /// wrapper's own call (the helper's default, the call `stopifnot()`
    /// reported), or `.mx_call` for a `call = caller` wrapper, which binds the
    /// caller's matched call before the checks run (#1548).
    pub fn guards(&self, call: Option<&str>) -> Vec<String> {
        self.assertions.iter().map(|a| a.to_guard(call)).collect()
    }
}

/// Returns `true` for types that should never get a fallback precheck.
///
/// These types are either handled specially by the FFI layer (`SEXP`),
/// consumed by the macro infrastructure (`Dots`, `LazyDots`, `Missing`), or
/// managed internally (`ExternalPtr`, `OwnedProtect`).
fn is_skip_type(ident: &str) -> bool {
    matches!(
        ident,
        "SEXP" | "Dots" | "LazyDots" | "Missing" | "ExternalPtr" | "OwnedProtect"
    )
}

/// Returns `true` if a type is unknown to the static type table and should
/// be recorded as a fallback parameter.
///
/// Returns `false` for skip types (SEXP, Dots, etc.) and reference types
/// (which are handled by the static table or skipped).
fn needs_fallback(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Path(tp) => {
            let Some(seg) = tp.path.segments.last() else {
                return false;
            };
            !is_skip_type(&seg.ident.to_string())
        }
        // References (&str, &[T], &Dots) are handled by static table or skipped
        syn::Type::Reference(_) => false,
        _ => false,
    }
}

/// Build precondition checks for a function's parameters.
///
/// Returns the checks for known types (render them with
/// [`PreconditionOutput::guards`]) and the parameters with unknown custom
/// types (`fallback_params`). The guards read:
/// ```r
/// if (!isTRUE(is.integer(a))) .miniextendr_arg_error("a", "must be integer")
/// if (!isTRUE(length(a) == 1L)) .miniextendr_arg_error("a", "must have length 1")
/// ```
///
/// Skips:
/// - `self`/`&self`/`&mut self` (receiver args)
/// - Parameters in `skip_params` (e.g., match_arg params already validated)
/// - Skip types (SEXP, Dots, ExternalPtr, etc.)
/// - The type-derived checks of the parameters in `opts.unchecked`
///
/// A parameter's [`ExplicitChecks`] are never skipped: its NA check comes
/// first, then its class checks (`inherits`, then `not_inherits`), then its
/// type checks, which take the `inherits` message when there is one.
pub fn build_precondition_checks(
    inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
    skip_params: &HashSet<String>,
    opts: &PreconditionOptions,
) -> PreconditionOutput {
    let mut assertions = Vec::new();
    let mut fallback_params = Vec::new();

    for arg in inputs {
        // Skip receiver (self/&self/&mut self)
        let syn::FnArg::Typed(pt) = arg else {
            continue;
        };

        // `NArgs` is no R formal: the wrapper passes `nargs()`, which no
        // check needs to look at (#1860). Nor is a `LazyDots` parameter's name:
        // its formal is `...`, passed unforced as the wrapper's frame (#1892).
        if crate::type_inspect::is_nargs_marker(&pt.ty)
            || crate::type_inspect::is_lazy_dots_marker(&pt.ty)
        {
            continue;
        }

        // Extract parameter name
        let syn::Pat::Ident(pat_ident) = pt.pat.as_ref() else {
            continue;
        };

        // The parameter's R formal (#1842): what the checks read and name
        let r_name = opts.formals.of_ident(&pat_ident.ident);
        let explicit = opts.explicit.get(&r_name);

        // The NA check first, so an `NA` gets its message whatever its class
        // or type, then the class checks, so a value of the wrong class gets
        // their message whatever its type.
        if let Some(checks) = explicit {
            assertions.extend(checks.value_assertions(
                &r_name,
                pt.ty.as_ref(),
                skip_params.contains(&r_name),
            ));
            assertions.extend(checks.class_assertions(&r_name, pt.ty.as_ref()));
        }

        // Type-derived checks: skipped for match_arg params (already validated
        // by match.arg()) and for the parameters that drop them.
        if !opts.unchecked.contains(&r_name) && !skip_params.contains(&r_name) {
            // Preserve the ordinary input domain and include any coercion extensions.
            if let Some(mut check) = r_check_for_type(pt.ty.as_ref()) {
                if opts.is_coerced(&r_name) {
                    check = coerce_widened(check, pt.ty.as_ref());
                }
                let mut type_assertions = check.assertions(&r_name);
                // An `inherits` message covers the type checks too.
                if let Some(message) = explicit.and_then(ExplicitChecks::type_check_message) {
                    for a in &mut type_assertions {
                        a.message = Some(message.clone());
                    }
                }
                assertions.extend(type_assertions);
            } else if needs_fallback(pt.ty.as_ref()) {
                // Unknown type → record for potential future validation
                fallback_params.push(FallbackParam {
                    r_name: r_name.clone(),
                });
            }
        }
    }

    PreconditionOutput {
        assertions,
        fallback_params,
    }
}

#[cfg(test)]
mod tests;
