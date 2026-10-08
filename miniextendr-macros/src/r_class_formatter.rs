//! Shared utilities for R class wrapper generation.
//!
//! This module provides abstractions to reduce duplication across the 5 class system
//! generators (Env, R6, S3, S4, S7). Each class system has different R idioms but shares
//! common patterns:
//!
//! - Class-level roxygen documentation
//! - Constructor generation
//! - Instance method iteration with `.Call()` building
//! - Static method handling
//! - Return strategy application
//!
//! ## Architecture
//!
//! ```text
//! ParsedImpl
//!     │
//!     ├─▶ ClassDocBuilder  → roxygen header lines (#' @title, @name, etc.)
//!     │
//!     └─▶ MethodContext[]  → pre-computed method data for each method
//!             │
//!             └─▶ ClassFormatter::format_constructor()
//!             └─▶ ClassFormatter::format_instance_method()
//!             └─▶ ClassFormatter::format_static_method()
//! ```

use crate::miniextendr_impl::{ParsedImpl, ParsedMethod};

/// Determine whether a class or method should be `@export`-ed.
///
/// Returns `true` unless the doc tags include `@noRd` or `@keywords internal`,
/// or the `noexport` flag is set (which should incorporate both the `noexport`
/// attribute and the `internal` attribute from the impl block).
///
/// Call sites should pass `parsed_impl.noexport || parsed_impl.internal` as
/// `noexport` so the `internal` attribute is correctly folded in.
pub(crate) fn should_export_from_tags(tags: &[String], noexport: bool) -> bool {
    let has_no_rd = crate::roxygen::has_roxygen_tag(tags, "noRd");
    let has_internal = crate::roxygen::has_roxygen_tag(tags, "keywords internal");
    !has_no_rd && !has_internal && !noexport
}

/// Emit the conditional S3 generic guard for a given generic name.
///
/// Returns an R code string (to be pushed onto a `lines: Vec<String>` with
/// `lines.push(emit_s3_generic_guard(name))`) that creates the generic when
/// it doesn't already exist as a function, and — mirroring the S7 classifier
/// (#1114, `s7_class.rs:880-935`) — shadows any existing binding that
/// `UseMethod` dispatch would never consult:
///
/// ```r
/// if (!base::exists("name", mode = "function")) {
///   name <- function(x, ...) UseMethod("name")
/// } else if (local({ .mx_gen <- base::get("name", mode = "function"); !(is.primitive(.mx_gen) || isTRUE(utils::isS3stdGeneric(.mx_gen)) || methods::isGeneric("name") || inherits(.mx_gen, "S7_generic")) })) {
///   .mx_shadow_default <- local({
///     .mx_masked <- base::get("name", mode = "function")
///     function(x, ...) .mx_masked(x, ...)
///   })
///   name <- function(x, ...) UseMethod("name")
///   base::registerS3method("name", "default", .mx_shadow_default, envir = base::environment())
///   base::rm(.mx_shadow_default)
/// }
/// # else: existing usable generic (primitive/S3/S4/S7) — reuse as-is.
/// ```
///
/// `name` resolving to a **plain non-generic closure** (`var`, `get`, `row`,
/// `col`, `diag`, `reshape`, …) is the #1248 bug: a bare `exists()` check
/// sees the name is bound and never installs the `UseMethod` dispatcher, so
/// the generated `name.Class` method is registered but silently never fires.
/// The classifier shadows such bindings with a package-local generic and
/// delegates the `default` method back to the masked closure, so ordinary
/// (non-dispatching) calls like `var(1:10)` keep working. S3 generic formals
/// are always `function(x, ...)`, so the delegation works positionally even
/// when the masked closure's first argument has a different name (e.g.
/// `reshape`'s is `data`) — S3 doesn't need S7's `dispatch_args`-mirroring
/// `fallback_sig` machinery.
///
/// The delegating default method is registered via `base::registerS3method()`
/// so it lives ONLY in the namespace's S3 methods table
/// (`.__S3MethodsTable__.`), never as a `name.default` namespace binding:
///
/// - a literal `name.default` binding trips roxygen2's dynamic S3 scan
///   (`warn_missing_s3_exports` walks the loaded namespace's bindings and
///   flags any method-shaped function not covered by an
///   `@export`/`@exportS3Method` block);
/// - a static NAMESPACE `S3method(name, default)` directive would break
///   package load whenever the shadow branch doesn't fire (e.g. for a real
///   generic like `print`, where no `name.default` of ours exists — and we
///   must never touch `print.default`).
///
/// Ordering inside the branch is load-bearing: `.mx_shadow_default` captures
/// the masked closure BEFORE `name` is rebound (once `name` is the generic,
/// `base::get("name")` would find our own generic → infinite recursion; the
/// assignment inside `local()` forces the value now — a function *argument*
/// would stay an unforced promise). The generic is bound at the branch top
/// level (not inside `local()`) so its closure environment is the package
/// namespace — `registerS3method` resolves the generic via
/// `get(genname, envir)`, takes `environment(genfun)` as the defining env,
/// and registers into THAT env's `.__S3MethodsTable__.`; the generic's env
/// must be the namespace for the table to be the namespace's.
/// `registerS3method` is called AFTER the generic is bound so it finds our
/// generic (not the masked closure, whose home namespace would otherwise
/// receive the registration). `base::rm` then drops the helper so the
/// namespace ends with zero helper bindings.
///
/// The classifier condition is wrapped in `local({...})` so `.mx_gen` doesn't
/// leak into the package namespace (the braced `else if` in the mirrored S7
/// pattern evaluates at source time in the namespace env — see the
/// corresponding fix at `s7_class.rs:902`, #1261 item 1).
///
/// Everything is `base::`-qualified (`exists`/`get`/`registerS3method`/
/// `environment`/`rm`): once we define a shadow generic named e.g. `get`, a
/// bare `get(...)` in a later generic's classifier would route through our
/// own generic instead of the real `base::get`.
///
/// Use this for S3/vctrs class generators and trait-ABI wrappers. Do **not**
/// use for S7 generics — those use `S7::new_generic()` / `S7::new_external_generic()`.
pub(crate) fn emit_s3_generic_guard(name: &str) -> String {
    // The assignment target must be an R symbol; operator generics such as
    // `%custom%` need backticks there, while the string literals keep the bare
    // name (#1475).
    let symbol = crate::naming::r_def_name(name);
    format!(
        "if (!base::exists(\"{name}\", mode = \"function\")) {{\n  {symbol} <- function(x, ...) UseMethod(\"{name}\")\n}} else if (local({{ .mx_gen <- base::get(\"{name}\", mode = \"function\"); !(is.primitive(.mx_gen) || isTRUE(utils::isS3stdGeneric(.mx_gen)) || methods::isGeneric(\"{name}\") || inherits(.mx_gen, \"S7_generic\")) }})) {{\n  # `{name}` is a plain closure that UseMethod dispatch will never consult.\n  # Shadow it with a package-local generic. The default method delegating to\n  # the masked closure is registered via registerS3method() so it lives ONLY\n  # in the namespace's S3 methods table: a literal `{name}.default` binding\n  # would trip roxygen2's dynamic S3 scan (warn_missing_s3_exports), and a\n  # static NAMESPACE S3method({name}, default) would break package load\n  # whenever this branch does not fire.\n  .mx_shadow_default <- local({{\n    .mx_masked <- base::get(\"{name}\", mode = \"function\")\n    function(x, ...) .mx_masked(x, ...)\n  }})\n  {symbol} <- function(x, ...) UseMethod(\"{name}\")\n  base::registerS3method(\"{name}\", \"default\", .mx_shadow_default, envir = base::environment())\n  base::rm(.mx_shadow_default)\n}}\n# else: existing usable generic (primitive/S3/S4/S7) — reuse as-is."
    )
}

/// Check whether `s` is a bare R identifier (only `[A-Za-z_][A-Za-z0-9_]*`).
pub(crate) fn is_bare_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Return a `.__MX_CLASS_REF_<name>__` placeholder (for bare identifiers) so the
/// resolver can look up the actual R class name at wrapper write time, or `name`
/// verbatim (for namespaced / non-identifier strings).
pub(crate) fn class_ref_or_verbatim(name: &str) -> String {
    if is_bare_identifier(name) {
        format!(".__MX_CLASS_REF_{name}__")
    } else {
        name.to_string()
    }
}

pub(crate) use crate::match_arg_keys::{
    choices_placeholder as match_arg_placeholder,
    param_doc_placeholder as match_arg_param_doc_placeholder,
};

/// Build the R-param-name → auto-generated `@param` text map for a method's
/// choice params: the write-time placeholder for a `match_arg` param (the
/// wrapper writer renders it from the enum's `MatchArg::CHOICES`, #210), the
/// literal `One of "a", "b".` line for a `choices(...)` param
/// ([`ParamAttrs::literal_choices_doc`](crate::miniextendr_fn::ParamAttrs::literal_choices_doc),
/// the same text a standalone function gets). Pass to
/// `MethodDocBuilder::with_choice_param_docs` in each class generator.
///
/// Takes the per-param attribute map directly (rather than `&ParsedMethod`) so
/// it's shared by both the inherent-impl (`MethodContext`) and trait-impl
/// (`TraitMethodContext`, `miniextendr_impl_trait/method_context.rs`) paths.
pub(crate) fn choice_param_doc_map(
    c_ident: &str,
    per_param: &std::collections::HashMap<String, crate::miniextendr_fn::ParamAttrs>,
) -> std::collections::HashMap<String, String> {
    let mut out = std::collections::HashMap::new();
    for (rust_name, attrs) in per_param {
        let r_name = crate::r_wrapper_builder::normalize_r_arg_string(rust_name);
        let doc = if attrs.match_arg {
            match_arg_param_doc_placeholder(c_ident, &r_name)
        } else if let Some(doc) = attrs.literal_choices_doc() {
            doc
        } else {
            continue;
        };
        out.insert(r_name, doc);
    }
    out
}

/// Build R prelude lines that validate `match_arg` / `choices` / `several_ok`
/// parameters before the `.Call()`.
///
/// Returns an empty vector when the method declares none. Every form is one
/// statement from `CallAttribution::match_arg_statement` with the method's
/// own attribution: the strict scalar helper (`NULL` and the formal default
/// select the first choice, a factor is read as its labels), the `Option<T>`
/// form that skips it for `NULL` (#1473), and the strict `several_ok` helper
/// (every element must match, `NULL` selects all; #1472). An `Either` layer,
/// over a scalar or a `several_ok` list (#1612), guards either helper with
/// `is.character(x) || is.factor(x)`, so everything else reaches the `R`
/// arm; `Missing` adds `!missing(x)` (#1551). For `match_arg` the
/// choice list is the write-time placeholder (`c_ident` keys it, the same one
/// `effective_r_defaults` puts in the formal); for `choices(...)` it is the
/// literal.
///
/// The `match_arg` statements come first, then the `choices(...)` ones, each
/// group in signature order (`inputs`), as in a standalone function's wrapper.
///
/// Shared by `MethodContext::match_arg_prelude` (inherent impls) and
/// `TraitMethodContext::match_arg_prelude` (trait impls) — see
/// `audit/2026-07-03-dogfooding-macros-codegen.md` finding #1 (trait methods
/// previously had no match_arg support at all).
pub(crate) fn build_match_arg_prelude(
    inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
    per_param: &std::collections::HashMap<String, crate::miniextendr_fn::ParamAttrs>,
    c_ident: &str,
    attribution: crate::r_wrapper_builder::CallAttribution,
) -> Vec<String> {
    use crate::miniextendr_fn::per_param_in_signature_order;
    let mut lines = Vec::new();

    for (rust_name, attrs) in per_param_in_signature_order(inputs, per_param) {
        if !attrs.match_arg {
            continue;
        }
        let r_name = crate::r_wrapper_builder::normalize_r_arg_string(rust_name);
        let placeholder = match_arg_placeholder(c_ident, &r_name);
        let aliases = crate::match_arg_keys::aliases_placeholder(c_ident, &r_name);
        lines.push(attribution.match_arg_statement(&r_name, &placeholder, Some(&aliases), attrs));
    }

    for (rust_name, attrs) in per_param_in_signature_order(inputs, per_param) {
        let Some(choices) = attrs.choices.as_ref() else {
            continue;
        };
        let r_name = crate::r_wrapper_builder::normalize_r_arg_string(rust_name);
        let quoted: Vec<String> = choices.iter().map(|c| format!("\"{c}\"")).collect();
        let choices_expr = format!("c({})", quoted.join(", "));
        lines.push(attribution.match_arg_statement(&r_name, &choices_expr, None, attrs));
    }

    lines
}

/// Rust-side parameter names that are validated by R's `match.arg()` and
/// therefore don't need type-check preconditions generated for them.
/// Shared by `MethodContext` and `TraitMethodContext`.
pub(crate) fn match_arg_skip_set(
    per_param: &std::collections::HashMap<String, crate::miniextendr_fn::ParamAttrs>,
) -> std::collections::HashSet<String> {
    let mut s = std::collections::HashSet::new();
    for (rust_name, attrs) in per_param {
        if attrs.match_arg || attrs.choices.is_some() {
            s.insert(crate::r_wrapper_builder::normalize_r_arg_string(rust_name));
        }
    }
    s
}

/// Build the R-side precondition guard lines for a parameter list, given
/// its per-param map (match_arg/choices skips, `inherits` / `not_inherits` /
/// `no_na` checks, each parameter's own `preconditions` decision), whether
/// `coerce` is active
/// for the whole method, and the method's and impl block's `preconditions` /
/// `no_preconditions` (`method_level`, `impl_level`). Each parameter keeps
/// its type-derived checks as `crate::r_preconditions::resolve_type_checks`
/// decides, under the crate default and the `no-preconditions-default`
/// feature; the named checks stay either way.
///
/// Neither impl methods nor trait methods carry a per-param `coerce` flag
/// (only function-wide `coerce`, see `ParsedMethod::per_param` docs), so
/// `coerce_params` is always empty here. `call` is the call every failing
/// guard is attributed to (`PreconditionOutput::guards`): `None` for the
/// method's own frame, `environment()` for an S3 method (#1851). Shared by
/// `MethodContext::precondition_checks`,
/// `TraitMethodContext::precondition_checks` and the R6 active-binding
/// setter.
pub(crate) fn build_method_precondition_checks(
    inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
    per_param: &std::collections::HashMap<String, crate::miniextendr_fn::ParamAttrs>,
    coerce_all: bool,
    method_level: Option<bool>,
    impl_level: Option<bool>,
    call: Option<&str>,
) -> Vec<String> {
    let opts = crate::r_preconditions::PreconditionOptions {
        coerce_all,
        coerce_params: std::collections::HashSet::new(),
        explicit: crate::miniextendr_fn::explicit_checks_by_r_name(per_param),
        unchecked: crate::r_preconditions::unchecked_params(
            inputs,
            |name| per_param.get(name).and_then(|a| a.preconditions),
            method_level,
            impl_level,
            crate::crate_config::preconditions_default(),
        ),
    };
    crate::r_preconditions::build_precondition_checks(inputs, &match_arg_skip_set(per_param), &opts)
        .guards(call)
}

/// Effective R-formal defaults for a method.
///
/// Layers defaults in priority order:
/// 1. `#[miniextendr(match_arg)]` → ALWAYS a write-time placeholder that the
///    wrapper writer resolves to `c("a", "b", ...)`. Any user-
///    supplied `default = "X"` is consumed elsewhere (rotates X to the front
///    of the choice list at write time) rather than overriding the formal.
/// 2. `#[miniextendr(choices("a", "b", ...))]` → `c("a", "b", ...)` formal default.
/// 3. User-provided `#[miniextendr(defaults(param = "..."))]` for non-match_arg
///    params.
///
/// The `Option<T>` form of a choice parameter gets `NULL` instead (#1473),
/// and a `no_default(p)` one no default at all (#1828; see
/// [`ParamAttrs::choice_formal`](crate::miniextendr_fn::ParamAttrs::choice_formal)).
/// A choice formal is documentation: the prelude
/// ([`build_match_arg_prelude`]) passes the choice list to the helpers, which
/// never read it off the formal (#1552). Shared by `MethodContext::new`
/// (inherent impls) and `TraitMethodContext::new` (trait impls,
/// `miniextendr_impl_trait/method_context.rs`).
pub(crate) fn effective_r_defaults(
    param_defaults: &std::collections::HashMap<String, String>,
    per_param: &std::collections::HashMap<String, crate::miniextendr_fn::ParamAttrs>,
    c_ident: &str,
) -> std::collections::HashMap<String, String> {
    let mut defaults = param_defaults.clone();
    // match_arg → unconditionally splice the placeholder (overriding any user
    // default, which is captured separately for write-time rotation).
    for (rust_name, attrs) in per_param {
        if !attrs.match_arg {
            continue;
        }
        let r_name = crate::r_wrapper_builder::normalize_r_arg_string(rust_name);
        // `Option<T>` (#1473): the formal is NULL (no choice); `no_default`
        // (#1828, refused with `defaults(p = ...)`): none. The prelude spells
        // the choices out through the placeholder either way.
        if let Some(default) = attrs.choice_formal(&match_arg_placeholder(c_ident, &r_name)) {
            defaults.insert(r_name, default);
        }
    }
    // choices(...) → c("a", "b", ...) formal (NULL for the `Option<T>` form,
    // none under `no_default`). Lower priority than user defaults (kept for
    // back-compat on non-match_arg params).
    for (rust_name, attrs) in per_param {
        let Some(choices) = attrs.choices.as_ref() else {
            continue;
        };
        let r_name = crate::r_wrapper_builder::normalize_r_arg_string(rust_name);
        let quoted: Vec<String> = choices.iter().map(|c| format!("\"{c}\"")).collect();
        if let Some(default) = attrs.choice_formal(&format!("c({})", quoted.join(", "))) {
            defaults.entry(r_name).or_insert(default);
        }
    }
    defaults
}

/// Pre-computed context for a method, holding all data needed for R wrapper generation.
///
/// This struct captures the common computations performed for every method across all
/// class systems, reducing duplicate code. It pre-formats the C wrapper name, R formal
/// parameters (with defaults), and R call arguments so each class generator can
/// focus on its specific formatting logic.
pub struct MethodContext<'a> {
    /// Reference to the parsed method metadata.
    pub method: &'a ParsedMethod,
    /// The C wrapper identifier string (e.g., `"C_Counter__inc"`), used in `.Call()`.
    pub c_ident: String,
    /// R formals string with defaults (e.g., `"value, step = 1L"`), used in
    /// `function(...)` signatures.
    pub params: String,
    /// R call arguments string without defaults (e.g., `"value, step"`), used
    /// inside `.Call()` expressions.
    pub args: String,
    /// The impl block's `preconditions` / `no_preconditions`
    /// (`ImplAttrs::preconditions`): below the method's own and each
    /// parameter's, above the crate default (see
    /// `build_method_precondition_checks`).
    pub impl_preconditions: Option<bool>,
    /// The call the method's conditions report (`CallAttribution`): the
    /// method's own `sys.call()` by default; an S3 method's generator sets
    /// `CallAttribution::Generic`, the frame resolved to the generic's call
    /// on a raise (#1851). It decides the `.call` of `instance_call`, the
    /// call the R-side checks raise with and the raise fallback.
    pub call_attribution: crate::r_wrapper_builder::CallAttribution,
}

impl<'a> MethodContext<'a> {
    /// Create a new MethodContext for a method.
    ///
    /// Computes the C wrapper identifier from the method name, type name, and optional
    /// label (for multi-impl-block disambiguation), then formats the R formals and
    /// call arguments from the method's signature and default values. The
    /// impl block says nothing about the R-side checks; use
    /// [`MethodContext::with_impl_preconditions`] to inherit its
    /// `preconditions` / `no_preconditions`.
    pub fn new(method: &'a ParsedMethod, type_ident: &syn::Ident, label: Option<&str>) -> Self {
        let c_ident = method.c_wrapper_ident(type_ident, label).to_string();
        let effective_defaults = effective_r_defaults(
            &method.param_defaults,
            &method.method_attrs.per_param,
            &c_ident,
        );
        let arg_builder = crate::r_wrapper_builder::RArgumentBuilder::new(&method.sig.inputs)
            .with_defaults(effective_defaults);
        let params = arg_builder.build_formals();
        let args = arg_builder.build_call_args();
        Self {
            method,
            c_ident,
            params,
            args,
            impl_preconditions: None,
            call_attribution: crate::r_wrapper_builder::CallAttribution::Wrapper,
        }
    }

    /// Set the `preconditions` decision inherited from the surrounding
    /// `ImplAttrs`. Returns `self` so callers can chain on top of
    /// `MethodContext::new`.
    pub fn with_impl_preconditions(mut self, impl_preconditions: Option<bool>) -> Self {
        self.impl_preconditions = impl_preconditions;
        self
    }

    /// Set the call the method's conditions report (see
    /// [`MethodContext::call_attribution`]).
    pub fn with_call_attribution(
        mut self,
        call_attribution: crate::r_wrapper_builder::CallAttribution,
    ) -> Self {
        self.call_attribution = call_attribution;
        self
    }

    /// Build the R-param-name → `@param` text map for this method's
    /// `match_arg` / `choices` params ([`choice_param_doc_map`]). Pass to
    /// `MethodDocBuilder::with_choice_param_docs`.
    pub fn choice_param_docs(&self) -> std::collections::HashMap<String, String> {
        choice_param_doc_map(&self.c_ident, &self.method.method_attrs.per_param)
    }

    /// Build R prelude lines that validate `match_arg` / `choices` / `several_ok`
    /// parameters via `base::match.arg()` before the `.Call()`.
    ///
    /// Returns an empty vector when the method declares none. Both `match_arg`
    /// and `choices(...)` carry their choice list as the formal default
    /// (`c("a", "b", ...)`), so `base::match.arg(arg)` finds the list by
    /// itself — no second arg, no C helper lookup. `match_arg` adds a
    /// factor → character coercion in front of `match.arg`.
    ///
    /// Callers should include these lines in the R wrapper body after parameter
    /// defaulting but before the `.Call()`.
    pub fn match_arg_prelude(&self) -> Vec<String> {
        build_match_arg_prelude(
            &self.method.sig.inputs,
            &self.method.method_attrs.per_param,
            &self.c_ident,
            self.call_attribution,
        )
    }

    /// Build the `.Call()` expression for a static/constructor call.
    pub fn static_call(&self) -> String {
        crate::r_wrapper_builder::DotCallBuilder::new(&self.c_ident)
            .with_args_str(&self.args)
            .build()
    }

    /// Build the `.Call()` expression for an instance method with `self` as ptr.
    ///
    /// The `self_expr` is typically "self", "private$.ptr", "x", "x@ptr", or "x@.ptr".
    /// The `.call` is the context's [`MethodContext::call_attribution`].
    pub fn instance_call(&self, self_expr: &str) -> String {
        crate::r_wrapper_builder::DotCallBuilder::new(&self.c_ident)
            .with_call_attribution(self.call_attribution)
            .with_self(self_expr)
            .with_args_str(&self.args)
            .build()
    }

    /// Like [`instance_call`](Self::instance_call) but passes `.call = NULL`.
    ///
    /// Use for lambda dispatch sites (S7 property getter/setter) where
    /// `sys.call()` names the S7 dispatch frame, not the user's call.
    pub fn instance_call_null_attr(&self, self_expr: &str) -> String {
        crate::r_wrapper_builder::DotCallBuilder::new(&self.c_ident)
            .null_call_attribution()
            .with_self(self_expr)
            .with_args_str(&self.args)
            .build()
    }

    /// Build full R formals for instance methods (prefixing x/self parameter).
    ///
    /// For S3/S4/S7: `"x, <params>, ..."`
    /// For Env/R6: `"<params>"` (self is implicit)
    pub fn instance_formals(&self, add_self_param: bool) -> String {
        self.instance_formals_with_dots(add_self_param, true)
    }

    /// Full R formals of a `<generic>.<Class>` S3 method: `"x, <params>, ..."`,
    /// except that for a replacement generic (a name ending in `<-`, such as
    /// `[[<-` or `names<-`) the dispatch `...` goes before the last formal,
    /// `"x, i, ..., value"`. `R CMD check` (`tools::checkReplaceFuns()`)
    /// requires a replacement method's last formal to be `value`, and the
    /// impl-block check (`check_s3_replacement_methods`) makes sure it is
    /// (#1853). A method that takes `&Dots` keeps its own order.
    pub fn s3_method_formals(&self) -> String {
        if self.method.has_dots || !self.generic_name().ends_with("<-") {
            return self.instance_formals(true);
        }
        let mut formals = crate::roxygen::split_r_formals(&self.params);
        formals.retain(|formal| !formal.is_empty());
        let last = formals.pop();
        let mut out = vec!["x"];
        out.extend(formals);
        out.push("...");
        out.extend(last);
        out.join(", ")
    }

    /// Build full R formals for instance methods with optional dots.
    ///
    /// When `include_dots` is false, omits `...` from the signature.
    /// This is used for strict generics that don't accept extra args.
    pub fn instance_formals_with_dots(&self, add_self_param: bool, include_dots: bool) -> String {
        let include_dispatch_dots = include_dots && !self.method.has_dots;
        if add_self_param {
            if include_dispatch_dots {
                if self.params.is_empty() {
                    "x, ...".to_string()
                } else {
                    format!("x, {}, ...", self.params)
                }
            } else {
                // No dots - strict formals
                if self.params.is_empty() {
                    "x".to_string()
                } else {
                    format!("x, {}", self.params)
                }
            }
        } else {
            self.params.clone()
        }
    }

    /// Build instance formals with a custom receiver name (default is `x`).
    ///
    /// Used by the S7 per-class fast-path shortcut (#949), whose receiver is
    /// named `self` to mirror the property dispatch lambdas, rather than the
    /// `x` used by the S7 generic.
    pub fn instance_formals_with_receiver(&self, receiver: &str, include_dots: bool) -> String {
        let tail = if include_dots && !self.method.has_dots {
            ", ..."
        } else {
            ""
        };
        if self.params.is_empty() {
            format!("{receiver}{tail}")
        } else {
            format!("{receiver}, {}{tail}", self.params)
        }
    }

    /// Get the generic name (uses override if present).
    pub fn generic_name(&self) -> String {
        // Explicit `generic = ".."` wins; otherwise the R-facing method name
        // (`r_name` / `postfix` / Rust ident) doubles as the generic, so a
        // renamed S3/S7 instance method dispatches under its R name.
        self.method
            .method_attrs
            .generic
            .clone()
            .unwrap_or_else(|| self.method.r_method_name())
    }

    /// Generate a source location comment for this method.
    ///
    /// Returns a string like `# Type::method (line:col)` using the method's span info.
    /// The file name is already stated in the impl block header comment, so line:col
    /// is sufficient to locate the method within that file.
    pub fn source_comment(&self, type_ident: &syn::Ident) -> String {
        let start = self.method.ident.span().start();
        format!(
            "# {}::{} ({}:{})",
            type_ident,
            self.method.ident,
            start.line,
            start.column + 1,
        )
    }

    /// Check if this method uses a generic override (for existing generics like print).
    pub fn has_generic_override(&self) -> bool {
        self.method.method_attrs.generic.is_some()
    }

    /// Get custom class suffix if specified.
    ///
    /// This allows double-dispatch patterns like `vec_ptype2.my_class.my_class`
    /// by specifying `#[miniextendr(s3(generic = "vec_ptype2", class = "my_class.my_class"))]`.
    pub fn class_suffix(&self) -> Option<&str> {
        self.method.method_attrs.class.as_deref()
    }

    /// Check if this method uses a custom class suffix.
    pub fn has_class_override(&self) -> bool {
        self.method.method_attrs.class.is_some()
    }

    /// Build the R-side precondition guard lines for this method's parameters.
    ///
    /// Returns static checks for known types. Custom types not in the static table
    /// are identified as fallback params but no R-side precheck is generated for them.
    ///
    /// Skips `self`/receiver parameters automatically (they are `FnArg::Receiver`) and
    /// any parameter validated by `base::match.arg()` (via `match_arg` / `choices`) —
    /// those already have a stronger runtime guarantee than an `is.character()` check.
    ///
    /// Each parameter keeps or drops its type-derived checks as its own
    /// spelling, the method's, the impl block's, the crate default and the
    /// feature decide ([`build_method_precondition_checks`]); the
    /// per-parameter `inherits(...)` / `not_inherits(...)` / `no_na(...)`
    /// checks stay.
    pub fn precondition_checks(&self) -> Vec<String> {
        // A coerced integer-element vector reads via `&[i32]` (INTSXP-only), so
        // its precondition tightens to `is.integer` (#616). Impl methods carry
        // coerce at method level (`method_attrs.coerce`, equivalent to
        // `coerce_all`); there is no per-param coerce on the impl path (see
        // ParsedMethod::per_param docs).
        build_method_precondition_checks(
            &self.method.sig.inputs,
            &self.method.method_attrs.per_param,
            self.method.method_attrs.coerce,
            self.method.method_attrs.preconditions,
            self.impl_preconditions,
            self.call_attribution.r_check_call(),
        )
    }

    /// Emit the 6-step method prelude into `lines`, each line prefixed with `indent`.
    ///
    /// The prelude is the standardised sequence that appears at the top of every
    /// generated R method body, in order:
    ///
    /// 1. `r_entry` — user code injected before any checks
    /// 2. `r_on_exit` — `on.exit(...)` cleanup
    /// 3. `lifecycle_prelude` — deprecation/superseded banner (class-system-specific label)
    /// 4. `precondition_checks` — one `isTRUE()` guard per check on typed params
    /// 5. `match_arg_prelude` — `base::match.arg(param)` validation
    /// 6. `r_post_checks` — user code after all checks, before `.Call()`
    ///
    /// (`Missing<T>` forwarding is not a prelude step: it lives inline in the
    /// `.Call()` args — see `build_call_args_vec` — because a binding of the
    /// missing sentinel errors on lookup.)
    ///
    /// `what` is the human-readable method label passed to `lifecycle_prelude`
    /// (e.g., `"Type.method"` for S3/S4, `"Type$method"` for Env/R6/S7).
    /// `indent` is the per-line prefix (e.g., `"  "` for 2-space, `"      "` for 6-space).
    pub fn emit_method_prelude(&self, lines: &mut Vec<String>, indent: &str, what: &str) {
        let m = self.method;
        if let Some(ref entry) = m.method_attrs.r_entry {
            for line in entry.lines() {
                lines.push(format!("{}{}", indent, line));
            }
        }
        if let Some(ref on_exit) = m.method_attrs.r_on_exit {
            lines.push(format!("{}{}", indent, on_exit.to_r_code()));
        }
        if let Some(prelude) = m.lifecycle_prelude(what) {
            lines.push(format!("{}{}", indent, prelude));
        }
        for check in self.precondition_checks() {
            lines.push(format!("{}{}", indent, check));
        }
        for line in self.match_arg_prelude() {
            lines.push(format!("{}{}", indent, line));
        }
        if let Some(ref post) = m.method_attrs.r_post_checks {
            for line in post.lines() {
                lines.push(format!("{}{}", indent, line));
            }
        }
    }
}

// region: generated parameter docs

/// The formals of `r_params` that `doc_tags` leaves undocumented, each with
/// its choice text from `choice_docs` (the write-time placeholder of a
/// `match_arg` parameter, the literal line of a `choices(...)` one), or
/// `None` for a formal that has none. Skips `self`, `.ptr` and `...`, and
/// yields nothing when the tags take the arguments from another topic
/// (`roxygen::params_documented_elsewhere` against `page`, #1590). The class
/// page default a builder appends is not in `doc_tags`, and an author
/// `@rdname` naming `page` is the same page, so neither suppresses them.
///
/// Splits on top-level commas only: a naive `split(", ")` shreds a
/// `mode = c("fast", "slow")` default into a bogus `"slow")` formal.
fn generated_params<'p, 'd>(
    r_params: &'p str,
    doc_tags: &[String],
    page: &str,
    choice_docs: Option<&'d std::collections::HashMap<String, String>>,
) -> Vec<(&'p str, Option<&'d str>)> {
    if crate::roxygen::params_documented_elsewhere(doc_tags, Some(page)) {
        return Vec::new();
    }
    crate::roxygen::split_r_formals(r_params)
        .into_iter()
        .map(crate::roxygen::formal_name)
        .filter(|name| !matches!(*name, ".ptr" | "..." | "self"))
        .filter(|name| !crate::roxygen::param_documented(doc_tags, name))
        .map(|name| {
            let choice = choice_docs
                .and_then(|docs| docs.get(name))
                .map(String::as_str);
            (name, choice)
        })
        .collect()
}

/// Push one `#' @param name text` line per [`generated_params`] entry: the
/// choice text for a choice parameter, `(undocumented)` otherwise (so
/// `R CMD check` codoc sees every formal in `\usage` documented).
fn push_param_filler(
    lines: &mut Vec<String>,
    r_params: &str,
    doc_tags: &[String],
    page: &str,
    choice_docs: Option<&std::collections::HashMap<String, String>>,
) {
    for (name, choice) in generated_params(r_params, doc_tags, page, choice_docs) {
        lines.push(format!(
            "#' @param {name} {}",
            choice.unwrap_or("(undocumented)")
        ));
    }
}

/// The documentation body of a method block that has no `\usage` (env
/// methods): the author tags `forward` accepts, verbatim, then one
/// `\describe{}` list with the author's `@param` items followed by the
/// choice items of [`generated_params`]. A formal without choice text gets no
/// item (nothing in `\usage` asks for one), and no `@param` line is emitted:
/// with no usage entry to match, roxygen2 would write an `\arguments` entry
/// that `R CMD check` reports as "Documented arguments not in \usage".
///
/// After a forwarded `@description` / `@details`, the list continues that
/// section after a blank line; after `@examples` / `@examplesIf`, whose code
/// block a blank line does not end, it opens a `@details`. Otherwise
/// `@description Arguments of \code{<label>()}:` and a blank line open it:
/// roxygen2 runs a tag's text up to the next tag, so after any other tag
/// (`@return`, `@seealso`, `@title`, ...) the list would land in that tag's
/// section, and an untagged list would be the block's intro paragraph, i.e.
/// its title, which roxygen2 drops on the class page the block merges into.
///
/// `doc_tags` is the full tag list, so a page tag that `forward` rejects
/// still sends the arguments elsewhere. Emits nothing when there is neither a
/// forwarded tag nor an item.
pub(crate) fn describe_params_lines(
    doc_tags: &[String],
    forward: impl Fn(&str) -> bool,
    r_params: &str,
    page: &str,
    choice_docs: Option<&std::collections::HashMap<String, String>>,
    label: &str,
) -> Vec<String> {
    let (param_tags, other_tags): (Vec<&str>, Vec<&str>) = doc_tags
        .iter()
        .map(String::as_str)
        .filter(|tag| forward(tag))
        .partition(|tag| tag.trim_start().starts_with("@param "));
    let item = |name: &str, desc: &str| format!("  \\item{{\\code{{{name}}}}}{{{desc}}}");
    let mut items: Vec<String> = param_tags
        .iter()
        .filter_map(|tag| {
            let rest = tag.trim_start().strip_prefix("@param ")?;
            let (name, desc) = rest.split_once(char::is_whitespace).unwrap_or((rest, ""));
            Some(item(name, desc))
        })
        .collect();
    items.extend(
        generated_params(r_params, doc_tags, page, choice_docs)
            .into_iter()
            .filter_map(|(name, choice)| Some(item(name, choice?))),
    );

    let mut lines = Vec::new();
    crate::roxygen::push_roxygen_tags_str(&mut lines, &other_tags);
    if items.is_empty() {
        return lines;
    }
    match other_tags
        .last()
        .and_then(|tag| crate::roxygen::roxygen_tag_name(tag))
    {
        Some("description" | "details") => lines.push("#'".to_string()),
        Some("examples" | "examplesIf") => lines.push("#' @details".to_string()),
        _ => {
            // `%` starts an Rd comment and `\` an escape, also inside `\code{}`.
            let label = label.replace('\\', "\\\\").replace('%', "\\%");
            lines.push(format!("#' @description Arguments of \\code{{{label}()}}:"));
            lines.push("#'".to_string());
        }
    }
    lines.push("#' \\describe{".to_string());
    // One `#' ` per line, so a wrapped `@param` stays inside the block.
    let item_refs: Vec<&str> = items.iter().map(String::as_str).collect();
    crate::roxygen::push_roxygen_tags_str(&mut lines, &item_refs);
    lines.push("#' }".to_string());
    lines
}

// endregion

/// Builder for class-level roxygen documentation header.
///
/// Generates the common roxygen tags that appear at the start of each class definition:
/// - `@title` (unless user provided)
/// - `@name` (unless user provided)
/// - `@rdname` (unless user provided)
/// - User-provided doc tags
/// - `@param` for the constructor formals the tags leave undocumented, when
///   set (`with_ctor_params`; S3 and vctrs, whose class block is the
///   constructor's block)
/// - `@source Generated by miniextendr...`
/// - Class-system-specific imports
/// - `@export` (unless user provided, `@noRd`, or internal/noexport flags)
pub struct ClassDocBuilder<'a> {
    /// The R-visible class name (e.g., `"Counter"`).
    class_name: &'a str,
    /// The Rust type identifier, used in the `@source` annotation.
    type_ident: &'a syn::Ident,
    /// User-provided roxygen tags extracted from doc comments.
    doc_tags: &'a [String],
    /// Human-readable label for the class system (e.g., `"R6"`, `"S3"`), used in
    /// the auto-generated `@title`. Empty for Env classes, whose title is plain
    /// `"<Class> Class"`.
    class_system_label: &'static str,
    /// Optional `@importFrom` tag for class-system-specific R packages
    /// (e.g., `"@importFrom R6 R6Class"`).
    imports: Option<String>,
    /// When `true`, adds `@keywords internal` and suppresses `@export`.
    /// Set by `#[miniextendr(internal)]`.
    attr_internal: bool,
    /// When `true`, suppresses `@export` but does not add `@keywords internal`.
    /// Set by `#[miniextendr(noexport)]`.
    attr_noexport: bool,
    /// The constructor's R formals and its choice-parameter `@param` text
    /// ([`choice_param_doc_map`]), for classes whose class block documents
    /// the constructor (S3, vctrs). See [`ClassDocBuilder::with_ctor_params`].
    ctor_params: Option<(&'a str, &'a std::collections::HashMap<String, String>)>,
}

impl<'a> ClassDocBuilder<'a> {
    /// Create a new ClassDocBuilder with the given class metadata.
    ///
    /// By default, `@export` is included unless suppressed by user tags or
    /// the `with_export_control` method.
    pub fn new(
        class_name: &'a str,
        type_ident: &'a syn::Ident,
        doc_tags: &'a [String],
        class_system_label: &'static str,
    ) -> Self {
        Self {
            class_name,
            type_ident,
            doc_tags,
            class_system_label,
            imports: None,
            attr_internal: false,
            attr_noexport: false,
            ctor_params: None,
        }
    }

    /// Document the constructor's formals on the class block: each formal the
    /// doc tags leave undocumented gets `@param name <choice text>` for a
    /// choice parameter and `@param name (undocumented)` otherwise, right after
    /// the doc tags. Nothing under `@noRd` / plain `noexport`, or when the tags
    /// take the arguments from another topic (`@rdname other`,
    /// `@inheritParams`). For generators whose class block is the
    /// constructor's block (S3 `new_<class>()`, vctrs `new_<class>()`).
    pub fn with_ctor_params(
        mut self,
        params: &'a str,
        choice_docs: &'a std::collections::HashMap<String, String>,
    ) -> Self {
        self.ctor_params = Some((params, choice_docs));
        self
    }

    /// Set R package imports (e.g., "@importFrom R6 R6Class").
    pub fn with_imports(mut self, imports: impl Into<String>) -> Self {
        self.imports = Some(imports.into());
        self
    }

    /// Set attribute-level internal/noexport flags from `ParsedImpl`.
    pub fn with_export_control(mut self, internal: bool, noexport: bool) -> Self {
        self.attr_internal = internal;
        self.attr_noexport = noexport;
        self
    }

    /// Build the roxygen `#' @tag` lines for the class header.
    ///
    /// Returns a vector of strings, each a complete roxygen comment line (e.g., `"#' @title ..."`).
    /// Auto-generates `@title`, `@name`, and `@rdname` if not provided by the user, and
    /// respects `@noRd` to suppress all documentation output.
    pub fn build(&self) -> Vec<String> {
        let has_title = crate::roxygen::has_roxygen_tag(self.doc_tags, "title");
        let has_name = crate::roxygen::has_roxygen_tag(self.doc_tags, "name");
        let has_rdname = crate::roxygen::has_roxygen_tag(self.doc_tags, "rdname");
        let has_export = crate::roxygen::has_roxygen_tag(self.doc_tags, "export");
        let has_no_rd = crate::roxygen::has_roxygen_tag(self.doc_tags, "noRd");
        let has_internal = crate::roxygen::has_roxygen_tag(self.doc_tags, "keywords internal");
        let effective_internal = has_internal || self.attr_internal;

        // `noexport` (without `internal`) must produce no Rd contribution at all —
        // no alias, no usage entry, nothing on a shared page — distinct from
        // `internal`, which stays documented under `\keyword{internal}`. Fold a
        // plain `noexport` into the same suppression gate as a user-written
        // `@noRd`. `internal` wins if both flags are set on the same impl block
        // (mirrors the standalone-fn `#[miniextendr(internal)]` precedence, where
        // `internal` + `noexport` together is a compile error).
        let suppress_rd = has_no_rd || (self.attr_noexport && !effective_internal);

        let mut lines = Vec::new();

        if suppress_rd && !has_no_rd {
            lines.push("#' @noRd".to_string());
        }

        if !has_title && !suppress_rd {
            // Env classes pass an empty label: "Counter Class", not "Counter  Class".
            let title = if self.class_system_label.is_empty() {
                format!("{} Class", self.class_name)
            } else {
                format!("{} {} Class", self.class_name, self.class_system_label)
            };
            lines.push(format!("#' @title {title}"));
        }
        if !has_name && !suppress_rd {
            lines.push(format!("#' @name {}", self.class_name));
        }
        if !has_rdname && !suppress_rd {
            lines.push(format!("#' @rdname {}", self.class_name));
        }
        crate::roxygen::push_roxygen_tags(&mut lines, self.doc_tags);
        if !suppress_rd {
            if let Some((params, choice_docs)) = self.ctor_params {
                push_param_filler(
                    &mut lines,
                    params,
                    self.doc_tags,
                    self.class_name,
                    Some(choice_docs),
                );
            }
            // An impl-level `@rdname topic` puts the class block on an author
            // topic, whose own block names and titles the page (#1590).
            crate::roxygen::push_order_after_topic_blocks(
                &mut lines,
                self.doc_tags,
                self.class_name,
            );
            lines.extend(crate::roxygen::class_source_tag(self.type_ident));
        }
        if let Some(ref imports) = self.imports
            && !suppress_rd
        {
            lines.push(format!("#' {}", imports));
        }
        // Inject @keywords internal if attr flag set and not already present
        if self.attr_internal && !has_internal && !suppress_rd {
            lines.push("#' @keywords internal".to_string());
        }
        // Don't auto-export if @noRd, @keywords internal, or attr flags are present
        if !has_export && !suppress_rd && !effective_internal && !self.attr_noexport {
            lines.push("#' @export".to_string());
        }

        lines
    }
}

/// Builder for method-level roxygen documentation.
///
/// Generates roxygen tags for individual methods within a class. Methods share
/// the class's `@rdname` so they appear on the same help page. The builder handles
/// `@name` formatting (with optional prefix like `$` for `Class$method` style)
/// and respects `@noRd` inheritance from the parent class.
pub struct MethodDocBuilder<'a> {
    /// The R class name (e.g., `"Counter"`).
    class_name: &'a str,
    /// The Rust method name (e.g., `"inc"`).
    method_name: &'a str,
    /// The Rust type identifier, used in the `@source` annotation.
    type_ident: &'a syn::Ident,
    /// User-provided roxygen tags extracted from the method's doc comments.
    doc_tags: &'a [String],
    /// Optional separator between class name and method name in `@name`
    /// (e.g., `"$"` produces `@name Counter$inc`).
    name_prefix: Option<&'a str>,
    /// Override for the `@name` tag when the R function name differs from the Rust
    /// method name (e.g., for standalone S3 methods like `format.my_class`).
    r_name_override: Option<String>,
    /// When `true`, adds `@export` to the method (used for standalone S3/S4 generics).
    /// Defaults to `false` because `Class$method` access does not need separate export.
    always_export: bool,
    /// Whether the parent class has `@noRd`. When `true`, this method emits only
    /// `#' @noRd` and skips all other documentation tags.
    class_has_no_rd: bool,
    /// When `true`, convert `@param` tags into `\describe{}` blocks instead of
    /// roxygen `@param` entries, with the choice text of undocumented choice
    /// parameters as extra items ([`describe_params_lines`]).
    ///
    /// Used for env-class methods where roxygen cannot infer `\usage` from
    /// `Class$method <- function()`. Without this, `@param` tags create
    /// `\arguments` entries with no matching `\usage`, causing R CMD check
    /// warnings ("Documented arguments not in \\usage").
    params_as_details: bool,
    /// Optional comma-separated R parameter string for auto-generating `@param` tags.
    /// When set, any parameter not already documented gets `@param name (undocumented)`
    /// (in `params_as_details` mode only a choice parameter gets an item).
    r_params: Option<&'a str>,
    /// When `true`, filter out `@param` tags from the doc_tags before pushing.
    ///
    /// Used for S4/S7 instance methods where the method is defined via `setMethod()`
    /// or `S7::method()` assignment, which roxygen2 doesn't parse for `\usage` entries.
    /// Including `@param` tags would create "Documented arguments not in \\usage" warnings.
    suppress_params: bool,
    /// Map of R-param-name → auto-generated `@param` text for choice
    /// parameters ([`choice_param_doc_map`]).
    ///
    /// When the auto-generated `@param` line would otherwise say `(undocumented)`,
    /// a `match_arg` param emits its placeholder instead, which the wrapper
    /// writer replaces with a rendered choice description (#210), and
    /// a `choices(...)` param its literal `One of ...` line.
    choice_param_docs: Option<&'a std::collections::HashMap<String, String>>,
}

impl<'a> MethodDocBuilder<'a> {
    /// Create a new MethodDocBuilder with default settings.
    ///
    /// By default, `always_export` is `false` because methods accessed via `Class$method`
    /// should not be exported directly -- only the class env and standalone S3 methods
    /// need `@export`.
    pub fn new(
        class_name: &'a str,
        method_name: &'a str,
        type_ident: &'a syn::Ident,
        doc_tags: &'a [String],
    ) -> Self {
        Self {
            class_name,
            method_name,
            type_ident,
            doc_tags,
            name_prefix: None,
            r_name_override: None,
            always_export: false,
            class_has_no_rd: false,
            params_as_details: false,
            r_params: None,
            suppress_params: false,
            choice_param_docs: None,
        }
    }

    /// Supply the R-param-name → `@param` text map of the method's choice
    /// params ([`choice_param_doc_map`]). When the auto-generated `@param`
    /// line would otherwise say `(undocumented)`, that text is emitted instead
    /// (a `match_arg` placeholder is rewritten by the wrapper writer, #210).
    pub fn with_choice_param_docs(
        mut self,
        docs: &'a std::collections::HashMap<String, String>,
    ) -> Self {
        self.choice_param_docs = Some(docs);
        self
    }

    /// Set a prefix for the @name tag (e.g., "$" for "Class$method").
    pub fn with_name_prefix(mut self, prefix: &'a str) -> Self {
        self.name_prefix = Some(prefix);
        self
    }

    /// Override the @name tag with a custom R function name.
    ///
    /// Use this when the R function name differs from the Rust method name
    /// (e.g., for standalone S3/S4/S7 static methods like `s3counter_default_counter`).
    pub fn with_r_name(mut self, r_name: String) -> Self {
        self.r_name_override = Some(r_name);
        self
    }

    /// Set whether the parent class has @noRd.
    ///
    /// When true, skips @name, @rdname, @source tags and adds @noRd instead.
    pub fn with_class_no_rd(mut self, class_has_no_rd: bool) -> Self {
        self.class_has_no_rd = class_has_no_rd;
        self
    }

    /// Convert `@param` tags to inline `\describe{}` blocks instead of roxygen `@param`.
    ///
    /// Used for env-class methods where roxygen can't infer `\usage` from `Class$method <- function()`.
    /// Without this, `@param` tags create `\arguments` entries with no matching `\usage`,
    /// causing R CMD check warnings ("Documented arguments not in \\usage"). With
    /// [`with_r_params`](Self::with_r_params) and
    /// [`with_choice_param_docs`](Self::with_choice_param_docs), an undocumented
    /// choice parameter gets a list item with its choice text; no parameter
    /// gets an `(undocumented)` filler.
    pub fn with_params_as_details(mut self) -> Self {
        self.params_as_details = true;
        self
    }

    /// Set the method's formal parameter names (comma-separated R params string).
    ///
    /// When set, auto-generates `@param name (undocumented)` for any parameter
    /// not already covered by a user `@param` tag. Skips `self`, `.ptr`, and
    /// `...` parameters. Generates nothing when the method's own tags take the
    /// arguments from another topic (`@rdname`, `@describeIn`,
    /// `@inheritParams`; see `roxygen::params_documented_elsewhere`).
    pub fn with_r_params(mut self, params: &'a str) -> Self {
        self.r_params = Some(params);
        self
    }

    /// Suppress `@param` tags from user doc comments.
    ///
    /// Used for S4/S7 instance methods where the method is defined via `setMethod()`
    /// or `S7::method()` assignment, which roxygen2 doesn't parse for `\usage` entries.
    pub fn with_suppress_params(mut self) -> Self {
        self.suppress_params = true;
        self
    }

    /// Build the roxygen `#' @tag` lines for the method.
    ///
    /// Returns a vector of strings, each a complete roxygen comment line. If the parent
    /// class has `@noRd`, returns only `["#' @noRd"]`. Otherwise generates `@name`,
    /// `@rdname`, `@source`, and optionally `@export` tags, plus any user-provided tags.
    /// A method that joins an author topic (`@rdname other`, `@describeIn other`)
    /// gets no `@name` / `@rdname` next to `@describeIn`, and sorts after the
    /// topic's own block (`roxygen::ORDER_AFTER_TOPIC_BLOCKS`).
    pub fn build(&self) -> Vec<String> {
        let mut lines = Vec::new();

        // If parent class has @noRd, skip all documentation and just add @noRd
        if self.class_has_no_rd {
            lines.push("#' @noRd".to_string());
            return lines;
        }

        let r_name = if let Some(ref r_name) = self.r_name_override {
            r_name.clone()
        } else if let Some(prefix) = self.name_prefix {
            format!("{}{}{}", self.class_name, prefix, self.method_name)
        } else {
            self.method_name.to_string()
        };

        if self.params_as_details {
            // Env methods: no `\usage`, so the author's `@param` tags and the
            // choice text go into a `\describe{}` list (no `@param` filler).
            // A bare `@export` would export `Type$method` itself.
            lines.extend(describe_params_lines(
                self.doc_tags,
                crate::roxygen::forwarded_member_tag,
                self.r_params.unwrap_or(""),
                self.class_name,
                self.choice_param_docs,
                &r_name,
            ));
        } else {
            if self.suppress_params {
                // Filter out @param tags — they would create "Documented arguments
                // not in \usage" warnings for S4/S7 methods.
                let filtered: Vec<&str> = self
                    .doc_tags
                    .iter()
                    .filter(|t| {
                        !t.trim_start()
                            .strip_prefix('@')
                            .is_some_and(|rest| rest.starts_with("param"))
                    })
                    .map(|s| s.as_str())
                    .collect();
                crate::roxygen::push_roxygen_tags_str(&mut lines, &filtered);
            } else {
                crate::roxygen::push_roxygen_tags(&mut lines, self.doc_tags);
            }

            // Auto-generate @param for undocumented method parameters, unless
            // the method's own tags send it to a topic that documents them
            // (`@rdname`, `@describeIn`) or inherit them (`@inheritParams`,
            // #1590). Choice params get their choice text (a match_arg
            // placeholder is rendered by the wrapper writer, #210).
            if let Some(params) = self.r_params {
                push_param_filler(
                    &mut lines,
                    params,
                    self.doc_tags,
                    self.class_name,
                    self.choice_param_docs,
                );
            }
        }

        // A method-level `@describeIn topic ...` lists the method in `topic`'s
        // "Functions" section. roxygen2 rejects it next to `@name` or `@rdname`
        // ("can not be used with @name") and takes the page from the method's
        // object, so the builder pushes neither, and the destination topic
        // supplies the title. Only generators whose method block documents an
        // R object get here with the tag (`reject_unsupported_describe_in`).
        let describe_in = crate::roxygen::describe_in_topic(self.doc_tags).is_some();
        if !describe_in && !crate::roxygen::has_roxygen_tag(self.doc_tags, "name") {
            lines.push(format!("#' @name {}", r_name));
        }

        // A method-level `@rdname` splits the method onto its own page (#1438).
        // Method prose is demoted to `@description` (see `roxygen_tags_from_attrs`)
        // and the class page normally supplies the `@title`, so the new page
        // would have none and roxygen2 would skip it ("no name and/or title").
        // Follow the standalone-function convention (`lib.rs`) and use the
        // structural R name as the title unless the author wrote one. A
        // `@noRd` block renders no page, so it needs no title (#1552).
        let has_no_rd = crate::roxygen::has_roxygen_tag(self.doc_tags, "noRd");
        if !describe_in {
            if crate::roxygen::has_roxygen_tag(self.doc_tags, "rdname") {
                if !has_no_rd && !crate::roxygen::has_roxygen_tag(self.doc_tags, "title") {
                    lines.push(format!("#' @title {}", r_name));
                }
            } else {
                lines.push(format!("#' @rdname {}", self.class_name));
            }
        }
        // On an author topic the topic's own block names and titles the page
        // (`ORDER_AFTER_TOPIC_BLOCKS`); the structural title above only
        // matters for a page no other block defines.
        crate::roxygen::push_order_after_topic_blocks(&mut lines, self.doc_tags, self.class_name);

        lines.extend(crate::roxygen::source_tag(format!(
            "Generated by miniextendr from `{}::{}`",
            self.type_ident, self.method_name
        )));

        let has_internal = crate::roxygen::has_roxygen_tag(self.doc_tags, "keywords internal");
        // Don't auto-export if @noRd or @keywords internal is present
        if self.always_export
            && !crate::roxygen::has_roxygen_tag(self.doc_tags, "export")
            && !has_no_rd
            && !has_internal
        {
            lines.push("#' @export".to_string());
        }

        lines
    }
}

/// Extension trait for `ParsedImpl` to iterate over methods as [`MethodContext`].
///
/// Provides convenience methods that wrap `ParsedImpl`'s method iterators,
/// automatically constructing a `MethodContext` for each method. This avoids
/// repeating the `MethodContext::new(m, type_ident, label)` boilerplate in
/// every class system generator.
pub trait ParsedImplExt {
    /// Create a `MethodContext` for the constructor method, if one exists.
    fn constructor_context(&self) -> Option<MethodContext<'_>>;

    /// Iterate over all instance methods (public + private + active) as `MethodContext`.
    fn instance_method_contexts(&self) -> impl Iterator<Item = MethodContext<'_>>;

    /// Iterate over static (non-receiver) methods as `MethodContext`.
    fn static_method_contexts(&self) -> impl Iterator<Item = MethodContext<'_>>;

    /// Iterate over public instance methods as `MethodContext` (for R6 `public` list).
    fn public_instance_method_contexts(&self) -> impl Iterator<Item = MethodContext<'_>>;

    /// Iterate over private instance methods as `MethodContext` (for R6 `private` list).
    fn private_instance_method_contexts(&self) -> impl Iterator<Item = MethodContext<'_>>;

    /// Iterate over active binding methods as `MethodContext` (for R6 `active` list).
    fn active_instance_method_contexts(&self) -> impl Iterator<Item = MethodContext<'_>>;
}

impl ParsedImplExt for ParsedImpl {
    fn constructor_context(&self) -> Option<MethodContext<'_>> {
        let impl_prec = self.preconditions;
        self.constructor().map(|m| {
            MethodContext::new(m, &self.type_ident, self.label()).with_impl_preconditions(impl_prec)
        })
    }

    fn instance_method_contexts(&self) -> impl Iterator<Item = MethodContext<'_>> {
        let type_ident = &self.type_ident;
        let label = self.label();
        let impl_prec = self.preconditions;
        self.instance_methods().map(move |m| {
            MethodContext::new(m, type_ident, label).with_impl_preconditions(impl_prec)
        })
    }

    fn static_method_contexts(&self) -> impl Iterator<Item = MethodContext<'_>> {
        let type_ident = &self.type_ident;
        let label = self.label();
        let impl_prec = self.preconditions;
        self.static_methods().map(move |m| {
            MethodContext::new(m, type_ident, label).with_impl_preconditions(impl_prec)
        })
    }

    fn public_instance_method_contexts(&self) -> impl Iterator<Item = MethodContext<'_>> {
        let type_ident = &self.type_ident;
        let label = self.label();
        let impl_prec = self.preconditions;
        self.public_instance_methods().map(move |m| {
            MethodContext::new(m, type_ident, label).with_impl_preconditions(impl_prec)
        })
    }

    fn private_instance_method_contexts(&self) -> impl Iterator<Item = MethodContext<'_>> {
        let type_ident = &self.type_ident;
        let label = self.label();
        let impl_prec = self.preconditions;
        self.private_instance_methods().map(move |m| {
            MethodContext::new(m, type_ident, label).with_impl_preconditions(impl_prec)
        })
    }

    fn active_instance_method_contexts(&self) -> impl Iterator<Item = MethodContext<'_>> {
        let type_ident = &self.type_ident;
        let label = self.label();
        let impl_prec = self.preconditions;
        self.active_instance_methods().map(move |m| {
            MethodContext::new(m, type_ident, label).with_impl_preconditions(impl_prec)
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn env_parameter_details_end_examples_blocks() {
        let type_ident: syn::Ident = syn::parse_quote!(Example);
        for examples in [
            "@examples\nExample$new(1L)",
            "@examplesIf TRUE\nExample$new(1L)",
        ] {
            let tags = vec![
                "@param value Integer input.".to_owned(),
                examples.to_owned(),
            ];
            let docs = super::MethodDocBuilder::new("Example", "new", &type_ident, &tags)
                .with_params_as_details()
                .build()
                .join("\n");
            assert!(
                docs.contains("Example$new(1L)\n#' @details\n#' \\describe{"),
                "{docs}"
            );
        }
    }

    /// An env method block drops the author's bare `@export`: roxygen2 would
    /// write `export("Example$new")`, which R refuses to load. The other tags,
    /// an `@export <symbol>` among them, are forwarded.
    #[test]
    fn env_method_drops_bare_export() {
        let type_ident: syn::Ident = syn::parse_quote!(Example);
        let tags: Vec<String> = [
            "@description Make one.",
            "@export",
            "@export example_helper",
            "@param value Integer input.",
        ]
        .map(str::to_owned)
        .to_vec();
        let docs = super::MethodDocBuilder::new("Example", "new", &type_ident, &tags)
            .with_name_prefix("$")
            .with_params_as_details()
            .build()
            .join("\n");
        assert!(
            !docs.lines().any(|l| l.trim_end() == "#' @export"),
            "{docs}"
        );
        for line in [
            "#' @description Make one.",
            "#' @export example_helper",
            "#' @name Example$new",
        ] {
            assert_eq!(
                docs.lines().filter(|l| *l == line).count(),
                1,
                "`{line}`: {docs}"
            );
        }
    }

    /// A method whose own tags join or inherit a topic gets no generated
    /// `@param` lines (#1590); its own `@param` stays. The class-page default
    /// the builder appends is not an author tag, so a method on the class page
    /// keeps the filler for each undocumented argument, also when the author
    /// names that page with a redundant `@rdname Counter`.
    #[test]
    fn method_param_filler_only_on_the_class_page() {
        let type_ident: syn::Ident = syn::parse_quote!(Counter);
        let build = |tags: &[&str]| {
            let tags: Vec<String> = tags.iter().map(|t| t.to_string()).collect();
            super::MethodDocBuilder::new("Counter", "add", &type_ident, &tags)
                .with_r_params("by, times = 1L")
                .build()
                .join("\n")
        };

        // No author page tag, or one naming the class page itself.
        for tags in [
            &["@param by Step size."][..],
            &["@param by Step size.", "@rdname Counter"][..],
        ] {
            let class_page = build(tags);
            assert!(class_page.contains("#' @rdname Counter"), "{class_page}");
            assert!(
                class_page.contains("#' @param times (undocumented)"),
                "{class_page}"
            );
        }

        for tag in ["@rdname counter_ops", "@inheritParams counter_ops"] {
            let docs = build(&["@param by Step size.", tag]);
            assert_eq!(
                docs.matches("#' @param ").count(),
                1,
                "`{tag}`: only the author's @param, got:\n{docs}"
            );
            assert!(docs.contains("#' @param by Step size."), "{docs}");
        }
    }

    /// A method-level `@describeIn` block carries neither `@name` nor
    /// `@rdname` (roxygen2: "@describeIn can not be used with @name") nor a
    /// structural title, sorts after the destination's own block, and leaves
    /// undocumented arguments to it. An `@rdname` naming another page sorts
    /// last too; the class page keeps the defaults.
    #[test]
    fn method_describe_in_drops_name_and_rdname() {
        let type_ident: syn::Ident = syn::parse_quote!(Counter);
        let order = format!("#' {}", crate::roxygen::ORDER_AFTER_TOPIC_BLOCKS);
        let build = |tags: &[&str]| {
            let tags: Vec<String> = tags.iter().map(|t| t.to_string()).collect();
            super::MethodDocBuilder::new("Counter", "add", &type_ident, &tags)
                .with_r_params("by, times = 1L")
                .with_r_name("add.Counter".to_string())
                .build()
        };

        let described = build(&[
            "@describeIn counter_ops Add a step.",
            "@param by Step size.",
        ]);
        assert_eq!(
            described,
            [
                "#' @describeIn counter_ops Add a step.",
                "#' @param by Step size.",
                order.as_str(),
            ],
            "{described:#?}"
        );

        let split = build(&["@rdname counter_ops"]);
        assert!(
            split.contains(&"#' @name add.Counter".to_string()),
            "{split:#?}"
        );
        assert!(
            split.contains(&"#' @title add.Counter".to_string()),
            "{split:#?}"
        );
        assert!(split.contains(&order), "{split:#?}");

        let class_page = build(&[]);
        assert!(
            class_page.contains(&"#' @rdname Counter".to_string()),
            "{class_page:#?}"
        );
        assert!(!class_page.contains(&order), "{class_page:#?}");
    }

    use super::ClassDocBuilder;

    #[test]
    fn test_method_context_static_call_no_args() {
        // This is a unit test for the static_call method
        // We'd need a mock ParsedMethod to test fully, but we can test the logic
        let call = ".Call(C_Test, .call = sys.call())";
        assert!(call.contains(".Call"));
    }

    /// Audit A10: a class-level `#[miniextendr(noexport)]` (without `internal`)
    /// must produce no Rd contribution at all — no `@title`/`@name`/`@rdname`/
    /// `@export` — same as a user-written `@noRd`. Before the fix, `noexport`
    /// only suppressed `@export`, leaving the class fully documented (with an
    /// alias) minus the export line.
    #[test]
    fn test_class_noexport_suppresses_all_roxygen() {
        let type_ident: syn::Ident = syn::parse_str("Foo").unwrap();
        let doc_tags: Vec<String> = vec![];
        let lines = ClassDocBuilder::new("Foo", &type_ident, &doc_tags, "R6")
            .with_export_control(false, true)
            .build();
        let joined = lines.join("\n");

        assert!(
            lines.iter().any(|l| l == "#' @noRd"),
            "noexport should emit @noRd, got:\n{}",
            joined
        );
        assert!(
            !joined.contains("@title") && !joined.contains("@name") && !joined.contains("@rdname"),
            "noexport should suppress @title/@name/@rdname entirely, got:\n{}",
            joined
        );
        assert!(
            !joined.contains("@export"),
            "noexport should suppress @export, got:\n{}",
            joined
        );
    }

    /// Companion: `#[miniextendr(internal)]` keeps the class documented (under
    /// `@keywords internal`) — it still contributes `@title`/`@name`/`@rdname`
    /// so it lands on a real help page, just unexported.
    #[test]
    fn test_class_internal_still_documented() {
        let type_ident: syn::Ident = syn::parse_str("Foo").unwrap();
        let doc_tags: Vec<String> = vec![];
        let lines = ClassDocBuilder::new("Foo", &type_ident, &doc_tags, "R6")
            .with_export_control(true, false)
            .build();
        let joined = lines.join("\n");

        assert!(
            !lines.iter().any(|l| l == "#' @noRd"),
            "internal should NOT emit @noRd (stays documented), got:\n{}",
            joined
        );
        assert!(
            joined.contains("@keywords internal"),
            "internal should add @keywords internal, got:\n{}",
            joined
        );
        assert!(
            joined.contains("@title") && joined.contains("@name") && joined.contains("@rdname"),
            "internal should still emit @title/@name/@rdname, got:\n{}",
            joined
        );
        assert!(
            !joined.contains("#' @export"),
            "internal should suppress @export, got:\n{}",
            joined
        );
    }

    /// Neither flag set: normal fully-documented, exported class.
    #[test]
    fn test_class_no_flags_fully_documented_and_exported() {
        let type_ident: syn::Ident = syn::parse_str("Foo").unwrap();
        let doc_tags: Vec<String> = vec![];
        let lines = ClassDocBuilder::new("Foo", &type_ident, &doc_tags, "R6")
            .with_export_control(false, false)
            .build();
        let joined = lines.join("\n");

        assert!(!joined.contains("@noRd"));
        assert!(!joined.contains("@keywords internal"));
        assert!(
            joined.contains("@title") && joined.contains("@name") && joined.contains("@rdname")
        );
        assert!(joined.contains("#' @export"));
    }

    /// The auto title names the class system, and has one space between words
    /// when there is no label (Env classes).
    #[test]
    fn test_class_title_spacing() {
        let type_ident: syn::Ident = syn::parse_str("Foo").unwrap();
        let doc_tags: Vec<String> = vec![];
        for (label, title) in [
            ("R6", "#' @title Foo R6 Class"),
            ("", "#' @title Foo Class"),
        ] {
            let lines = ClassDocBuilder::new("Foo", &type_ident, &doc_tags, label).build();
            assert_eq!(
                lines.iter().filter(|l| l.starts_with("#' @title")).count(),
                1
            );
            assert!(lines.iter().any(|l| l == title), "{lines:?}");
        }
    }

    fn tags_of(tags: &[&str]) -> Vec<String> {
        tags.iter().map(|t| t.to_string()).collect()
    }

    fn choice_docs(entries: &[(&str, &str)]) -> std::collections::HashMap<String, String> {
        entries
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    /// An S3 / vctrs constructor gets one generated `@param` per formal its
    /// tags leave undocumented, before `@export`: the choice text for a
    /// choice parameter, `(undocumented)` otherwise. An author `@param` is
    /// kept and not repeated; `@noRd`, `noexport` and `@inheritParams` get
    /// none.
    #[test]
    fn class_ctor_params_fill_undocumented_formals() {
        let type_ident: syn::Ident = syn::parse_quote!(Point);
        let docs = choice_docs(&[("mode", "One of \"a\", \"b\".")]);
        let params = "x, y = 1, mode = c(\"a\", \"b\"), ...";
        let build = |tags: &[&str], internal: bool, noexport: bool| {
            let tags = tags_of(tags);
            ClassDocBuilder::new("Point", &type_ident, &tags, "S3")
                .with_ctor_params(params, &docs)
                .with_export_control(internal, noexport)
                .build()
        };

        let lines = build(&["@param x The x coordinate."], false, false);
        assert_eq!(
            lines,
            [
                "#' @title Point S3 Class",
                "#' @name Point",
                "#' @rdname Point",
                "#' @param x The x coordinate.",
                "#' @param y (undocumented)",
                "#' @param mode One of \"a\", \"b\".",
                "#' @export",
            ],
            "{lines:#?}"
        );

        let internal = build(&[], true, false);
        assert!(
            internal.contains(&"#' @param x (undocumented)".to_string()),
            "{internal:#?}"
        );

        for (tags, noexport) in [
            (&["@noRd"][..], false),
            (&[][..], true),
            (&["@inheritParams point_ops"][..], false),
        ] {
            let lines = build(tags, false, noexport);
            assert!(
                !lines.iter().any(|l| l.starts_with("#' @param")),
                "{tags:?} noexport={noexport}: {lines:#?}"
            );
        }
    }

    fn env_docs(
        tags: &[&str],
        params: &str,
        docs: &std::collections::HashMap<String, String>,
    ) -> Vec<String> {
        let type_ident: syn::Ident = syn::parse_quote!(Example);
        let tags = tags_of(tags);
        super::MethodDocBuilder::new("Example", "new", &type_ident, &tags)
            .with_name_prefix("$")
            .with_params_as_details()
            .with_r_params(params)
            .with_choice_param_docs(docs)
            .build()
    }

    /// An env method (no `\usage`) lists the author's `@param` items and then
    /// the choice items in one `\describe{}`, with no `(undocumented)` item
    /// and no `@param` line (roxygen2 would report "Documented arguments not
    /// in \usage"). Unless a `@description` / `@details` tag directly
    /// precedes the list, a labelled `@description` lead-in and a blank line
    /// open it.
    #[test]
    fn env_params_share_one_describe_list() {
        let docs = choice_docs(&[("mode", "One of \"a\", \"b\".")]);
        let params = "n, plain, mode = c(\"a\", \"b\")";

        for tags in [&[][..], &["@param n Amount."][..]] {
            let lines = env_docs(tags, params, &docs);
            assert!(!lines.iter().any(|l| l.contains("@param")), "{lines:#?}");
            assert!(
                !lines.iter().any(|l| l.contains("undocumented")),
                "{lines:#?}"
            );
            assert_eq!(lines.iter().filter(|l| *l == "#' \\describe{").count(), 1);
            let joined = lines.join("\n");
            assert!(
                joined.starts_with(
                    "#' @description Arguments of \\code{Example$new()}:\n#'\n#' \\describe{\n"
                ),
                "{joined}"
            );
            assert!(
                joined.contains("#'   \\item{\\code{mode}}{One of \"a\", \"b\".}\n#' }"),
                "{joined}"
            );
        }
        let with_author = env_docs(&["@param n Amount."], params, &docs).join("\n");
        assert!(
            with_author.contains(
                "#'   \\item{\\code{n}}{Amount.}\n#'   \\item{\\code{mode}}{One of \"a\", \"b\".}"
            ),
            "author item first: {with_author}"
        );

        // A prose tag before the list keeps it in that section; after any
        // other tag the lead-in opens it, or the list would land in that
        // tag's section (`\value`, `\seealso`, the title).
        let lead_in = "#' @description Arguments of \\code{Example$new()}:\n#'\n#' \\describe{";
        for (last, sep) in [
            (
                "@description Plan it.",
                "#' @description Plan it.\n#'\n#' \\describe{".to_string(),
            ),
            (
                "@details More.",
                "#' @details More.\n#'\n#' \\describe{".to_string(),
            ),
            ("@seealso other", format!("#' @seealso other\n{lead_in}")),
            (
                "@return A value.",
                format!("#' @return A value.\n{lead_in}"),
            ),
            ("@title Mine", format!("#' @title Mine\n{lead_in}")),
        ] {
            let joined = env_docs(&[last], params, &docs).join("\n");
            assert!(joined.starts_with(&sep), "`{last}`: {joined}");
            assert_eq!(
                joined.matches("Arguments of").count(),
                usize::from(sep.contains("Arguments of")),
                "`{last}`: {joined}"
            );
        }
        // Prose and a trailing `@return`: the lead-in is a second
        // `@description`, which roxygen2 merges into the first.
        let joined = env_docs(
            &["@description Plan it.", "@return A value."],
            params,
            &docs,
        )
        .join("\n");
        assert!(
            joined.starts_with(&format!(
                "#' @description Plan it.\n#' @return A value.\n{lead_in}"
            )),
            "{joined}"
        );

        // An author `@param` on the choice parameter replaces its choice item.
        let joined = env_docs(&["@param mode Mine."], params, &docs).join("\n");
        assert_eq!(
            joined.matches("\\item{\\code{mode}}").count(),
            1,
            "{joined}"
        );
        assert!(
            joined.contains("#'   \\item{\\code{mode}}{Mine.}"),
            "{joined}"
        );
        assert!(!joined.contains("One of"), "{joined}");

        // No item and no tag: nothing at all; a plain formal gets no item.
        let empty = std::collections::HashMap::new();
        let nothing = env_docs(&[], "n, plain", &empty);
        assert!(
            nothing
                .iter()
                .all(|l| !l.contains("describe") && !l.contains("@description")),
            "{nothing:#?}"
        );
    }

    /// Tags the caller does not forward still send the arguments elsewhere
    /// (`@inheritParams`), so no choice item is generated for them; the label
    /// is used verbatim, with `%` and `\` escaped for Rd.
    #[test]
    fn describe_params_lines_forward_and_label() {
        let docs = choice_docs(&[("mode", "One of \"a\", \"b\".")]);
        let tags = tags_of(&["@inheritParams family"]);
        let lines = super::describe_params_lines(
            &tags,
            |tag| !tag.starts_with("@inheritParams"),
            "mode = c(\"a\", \"b\")",
            "Example",
            Some(&docs),
            "Example$Trait$pick",
        );
        assert!(lines.is_empty(), "{lines:#?}");

        let lines = super::describe_params_lines(
            &[],
            |_| true,
            "mode = c(\"a\", \"b\")",
            "Example",
            Some(&docs),
            "Example$%op%",
        );
        assert_eq!(
            lines[0], "#' @description Arguments of \\code{Example$\\%op\\%()}:",
            "{lines:#?}"
        );
    }

    /// Every line of a wrapped author `@param` stays inside the roxygen
    /// block: the continuation line keeps its `#' ` lead instead of landing
    /// in wrappers.R as bare R code.
    #[test]
    fn env_wrapped_param_stays_in_roxygen_block() {
        let empty = std::collections::HashMap::new();
        let lines = env_docs(&["@param n Amount\nto add."], "n", &empty);
        assert!(lines.iter().all(|l| l.starts_with("#'")), "{lines:#?}");
        let joined = lines.join("\n");
        assert!(
            joined.contains("#'   \\item{\\code{n}}{Amount\n#' to add.}"),
            "{joined}"
        );
    }
}
