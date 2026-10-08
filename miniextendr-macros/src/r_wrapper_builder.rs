//! Shared utilities for building R wrapper code.
//!
//! This module provides builders for constructing R function signatures and call arguments
//! consistently across both standalone functions and impl methods.
//!
//! ## Key Components
//!
//! - [`RArgumentBuilder`]: Builds R formals and `.Call()` arguments from Rust signatures
//! - [`DotCallBuilder`]: Formats `.Call()` invocations with proper argument handling
//! - [`RoxygenBuilder`]: Generates roxygen2 documentation tags
//!
//! ## Usage
//!
//! ```ignore
//! // Build R function signature
//! let formals = build_r_formals_from_sig(&method.sig, &defaults);
//! let call_args = build_r_call_args_from_sig(&method.sig);
//!
//! // Build .Call() invocation
//! let call = DotCallBuilder::new("C_MyType__method")
//!     .with_self("self")
//!     .with_args(&["x", "y"])
//!     .build();
//!
//! // Build roxygen tags
//! let tags = RoxygenBuilder::new("MyType")
//!     .name("method")
//!     .rdname("MyType")
//!     .export()
//!     .build();
//! ```

/// Normalizes Rust argument identifiers for R.
///
/// - Leading `_` → stripped (Rust convention for unused params)
/// - Leading `__` → stripped
/// - Otherwise → unchanged
///
/// # Examples
/// - `_x` → `x`
/// - `_to` → `to`
/// - `__field` → `field`
/// - `value` → `value`
///
/// Note: We strip underscores rather than prefixing "unused" because R callers
/// (like vctrs) may use named arguments that must match the original name.
pub fn normalize_r_arg_ident(rust_ident: &syn::Ident) -> syn::Ident {
    syn::Ident::new(
        &normalize_r_arg_string(&crate::naming::ident_name(rust_ident)),
        rust_ident.span(),
    )
}

/// String form of [`normalize_r_arg_ident`] that skips the `syn::Ident` round-trip.
///
/// Most callers feed the result into `format!`/`HashMap` keys and immediately
/// `.to_string()` the returned ident — this avoids that allocation pair.
pub fn normalize_r_arg_string(name: &str) -> String {
    let normalized = name.trim_start_matches('_');
    if normalized.is_empty() {
        "arg".to_string()
    } else {
        normalized.to_string()
    }
}

/// Check the R formals a signature produces, before any wrapper is generated.
///
/// Each parameter becomes the formal [`normalize_r_arg_string`] gives it; the
/// `&Dots` parameter becomes `...`, wherever it sits, and is skipped. A formal must be an R
/// name (not a reserved word such as `if` or `in`, not starting with a digit),
/// distinct from the others (`x` and `_x` both become `x`), and not one of the
/// `reserved` names the generated wrapper binds itself (a method's receiver,
/// R6's `self` and `private`), each paired with what it is. Any of these would
/// break the wrapper: a parse error in the wrappers file, or a parameter that
/// hides the object the method works on.
///
/// Every case is a compile error rather than a rename: callers pass arguments
/// by name, so the R formal has to stay the parameter's own name.
pub(crate) fn check_r_formals(
    inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
    reserved: &[(String, String)],
) -> syn::Result<()> {
    let mut seen: std::collections::HashMap<String, &syn::Ident> = std::collections::HashMap::new();
    for (formal, ident) in r_formal_names(inputs) {
        let problem = if crate::naming::is_r_reserved_word(&formal) {
            Some(format!(
                "is an R reserved word, so the generated R wrapper would not parse. \
                 Rename the parameter (for example `{formal}_`)"
            ))
        } else if formal.starts_with(|c: char| c.is_ascii_digit()) {
            Some(
                "starts with a digit, so the generated R wrapper would not parse. \
                 Rename the parameter so that, without its leading underscores, it \
                 starts with a letter"
                    .to_string(),
            )
        } else {
            reserved
                .iter()
                .find(|(name, _)| *name == formal)
                .map(|(_, role)| format!("{role}. Rename the parameter"))
        };
        if let Some(problem) = problem {
            return Err(syn::Error::new(
                ident.span(),
                format!("parameter `{ident}` becomes the R argument `{formal}`, which {problem}."),
            ));
        }
        if let Some(first) = seen.insert(formal.clone(), ident) {
            return Err(syn::Error::new(
                ident.span(),
                format!(
                    "parameters `{first}` and `{ident}` both become the R argument `{formal}` \
                     (the R name drops leading underscores), and R rejects a repeated argument \
                     name. Rename one of them."
                ),
            ));
        }
    }
    Ok(())
}

/// The R formal each parameter of a signature becomes, with the parameter's
/// ident: [`normalize_r_arg_string`] of its name. The `&Dots` parameter
/// becomes `...` wherever it sits and is skipped, as are receivers,
/// non-ident patterns and an `NArgs` parameter, which is no formal (#1860).
///
/// [`check_r_formals`] checks these names, and the shadowing pass
/// ([`formal_names`](crate::r_shadowing::formal_names)) qualifies the calls
/// they would shadow, so the two agree on what each formal is named.
pub(crate) fn r_formal_names(
    inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
) -> impl Iterator<Item = (String, &syn::Ident)> {
    let dots_index = crate::miniextendr_fn::dots_index(inputs);
    inputs.iter().enumerate().filter_map(move |(idx, input)| {
        let syn::FnArg::Typed(pat_type) = input else {
            return None;
        };
        if Some(idx) == dots_index || crate::type_inspect::is_nargs_marker(&pat_type.ty) {
            return None;
        }
        let syn::Pat::Ident(pat_ident) = pat_type.pat.as_ref() else {
            return None;
        };
        let ident = &pat_ident.ident;
        Some((
            normalize_r_arg_string(&crate::naming::ident_name(ident)),
            ident,
        ))
    })
}

/// Split a comma-separated choices list (as given to `choices(param = "a, b, c")`)
/// into individual trimmed entries. Surrounding double-quotes are tolerated so
/// users can spell the list either way: `"a, b"` or `"\"a\", \"b\""`.
///
/// Shared by the inherent-impl (`miniextendr_impl.rs`) and trait-impl
/// (`miniextendr_impl_trait/vtable.rs`) `choices(...)` attribute parsers so the
/// two independently-maintained parsers can't drift on quoting/whitespace rules.
pub(crate) fn split_choice_list(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|s| s.trim().trim_matches('"').to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Builder for R function formal parameters and call arguments.
///
/// Handles:
/// - Underscore normalization (`_x` → `x`)
/// - Unit type defaults (`()` → `= NULL`)
/// - Dots: the `&Dots` parameter is `...` in the formals and `list(...)` in
///   the call arguments, at its own position in the signature
/// - The argument count: an `NArgs` parameter is no formal and `nargs()` in
///   the call arguments, at its own position (#1860)
/// - Consistent formatting across function and method wrappers
pub struct RArgumentBuilder<'a> {
    /// The function's input parameters from the parsed Rust signature.
    inputs: &'a syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
    /// Position of the `&Dots` parameter in `inputs` (receiver included, the
    /// way both build loops count). Its Rust name never reaches R: the formal
    /// is always plain `...`.
    dots_index: Option<usize>,
    /// If true, skip the first parameter (used for `self`/`&self` in method wrappers,
    /// since the self argument is handled separately by [`DotCallBuilder::with_self`]).
    skip_first: bool,
    /// Parameter default values from `#[miniextendr(default = "...")]` attributes.
    /// Keys are normalized R parameter names, values are R expressions emitted verbatim
    /// (e.g., `"1L"`, `"c(1, 2, 3)"`, `"NULL"`).
    defaults: std::collections::HashMap<String, String>,
}

impl<'a> RArgumentBuilder<'a> {
    /// Create a new builder for the given function inputs.
    pub fn new(inputs: &'a syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>) -> Self {
        Self {
            inputs,
            dots_index: crate::miniextendr_fn::dots_index(inputs),
            skip_first: false,
            defaults: std::collections::HashMap::new(),
        }
    }

    /// Add parameter defaults from `#[miniextendr(default = "...")]` attributes.
    ///
    /// Keys are normalized R parameter names (after underscore stripping),
    /// values are R expression strings emitted verbatim into formals.
    pub fn with_defaults(mut self, defaults: std::collections::HashMap<String, String>) -> Self {
        self.defaults = defaults;
        self
    }

    /// Skip the first parameter (for instance methods with `self`).
    pub fn skip_first(mut self) -> Self {
        self.skip_first = true;
        self
    }

    /// Build R formal parameters string (for function signature).
    ///
    /// # Returns
    /// Comma-separated parameter list, e.g., `"x, y = NULL, ..."`
    ///
    /// This method handles R-style defaults (like `1L`, `c(1,2,3)`) that aren't
    /// valid Rust syntax by outputting them directly as strings.
    pub fn build_formals(&self) -> String {
        let mut formals = Vec::new();

        for (idx, input) in self.inputs.iter().enumerate() {
            // Skip first if requested (for self in methods)
            if self.skip_first && idx == 0 {
                continue;
            }

            let pat_type = match input {
                syn::FnArg::Typed(pt) => pt,
                syn::FnArg::Receiver(_) => continue, // Skip self receivers
            };

            // The dots, at their own position. In R, `...` takes no name or
            // default, so the Rust binding never shows: the formal is `...`.
            if Some(idx) == self.dots_index {
                formals.push("...".to_string());
                continue;
            }

            // The argument count (#1860) is no formal: the call arguments pass
            // `nargs()` in its place.
            if crate::type_inspect::is_nargs_marker(&pat_type.ty) {
                continue;
            }

            // Extract and normalize argument name
            let arg_ident = match pat_type.pat.as_ref() {
                syn::Pat::Ident(pat_ident) => normalize_r_arg_ident(&pat_ident.ident),
                _ => continue,
            };

            // Check for user-specified default value
            if let Some(default_val) = self.defaults.get(&arg_ident.to_string()) {
                // User provided default via #[miniextendr(default = "...")]
                // Output directly as string - supports R-style defaults like "1L", "c(1,2,3)"
                formals.push(format!("{} = {}", arg_ident, default_val));
                continue;
            }

            // Add default for unit types
            match pat_type.ty.as_ref() {
                syn::Type::Tuple(t) if t.elems.is_empty() => {
                    formals.push(format!("{} = NULL", arg_ident));
                }
                _ => {
                    formals.push(arg_ident.to_string());
                }
            }
        }

        formals.join(", ")
    }

    /// Build R call arguments string (for `.Call()` invocation).
    ///
    /// # Returns
    /// Comma-separated argument list, e.g., `"x, y, list(...)"`
    pub fn build_call_args(&self) -> String {
        self.build_call_args_vec().join(", ")
    }

    /// Build R call arguments as a `Vec<String>`.
    ///
    /// Each element is a single argument expression. Dots parameters become
    /// `"list(...)"` to capture variadic args as an R list for the `.Call()` interface.
    pub fn build_call_args_vec(&self) -> Vec<String> {
        let mut call_args = Vec::new();

        for (idx, input) in self.inputs.iter().enumerate() {
            // Skip first if requested (for self in methods)
            if self.skip_first && idx == 0 {
                continue;
            }

            let syn::FnArg::Typed(pat_type) = input else {
                continue;
            };

            // The dots, at their own position: the formal is plain `...`, so
            // the argument is always `list(...)`.
            if Some(idx) == self.dots_index {
                call_args.push("list(...)".to_string());
                continue;
            }

            // The argument count (#1860): the wrapper's own `nargs()`, which
            // the C wrapper converts like any argument. `.Call()` evaluates
            // its arguments in the wrapper's frame, so `nargs()` counts the
            // wrapper's call as typed.
            if crate::type_inspect::is_nargs_marker(&pat_type.ty) {
                call_args.push("nargs()".to_string());
                continue;
            }

            // Extract and normalize argument name
            let arg_ident = match pat_type.pat.as_ref() {
                syn::Pat::Ident(pat_ident) => normalize_r_arg_ident(&pat_ident.ident),
                _ => continue,
            };

            // `Quoted` / `Quosure` (also under `Missing<..>`, #1835): the
            // argument unevaluated, behind the same missing-argument sentinel
            // as `Missing<T>`. Neither branch forces it. A `default` on
            // `Missing<..>` only writes the formal: `missing()` is `TRUE` for
            // an omitted argument with a default, so the default is never
            // evaluated.
            if let Some(param) = crate::type_inspect::unevaluated_param(pat_type.ty.as_ref()) {
                call_args.push(param.kind.r_call_arg(&arg_ident.to_string()));
                continue;
            }

            // `Missing<T>`: forward true missingness as the `R_MissingArg`
            // sentinel, produced *at the argument position*. A binding holding
            // the sentinel errors on symbol lookup ("argument is missing, with
            // no default"), so the former `if (missing(x)) x <- quote(expr=)`
            // prelude broke every truly-missing call. (`Missing<T>` + user
            // default is rejected at macro parse time, so no default-shadowing
            // concern here.)
            if is_missing_type(pat_type.ty.as_ref()) {
                call_args.push(format!(
                    "if (missing({p})) quote(expr=) else {p}",
                    p = arg_ident
                ));
                continue;
            }

            call_args.push(arg_ident.to_string());
        }

        call_args
    }
}

/// Build R formal parameters from a Rust function signature, with optional defaults.
///
/// Automatically skips `self`/`&self` receivers. `Missing<T>` parameters without
/// user-provided defaults appear as bare formals (no default value); the
/// `R_MissingArg` sentinel forwarding is emitted inline in the `.Call()` args
/// (see [`RArgumentBuilder::build_call_args_vec`]).
///
/// Returns a comma-separated string of R formals, e.g., `"x, y = NULL, ..."`.
pub(crate) fn build_r_formals_from_sig(
    sig: &syn::Signature,
    defaults: &std::collections::HashMap<String, String>,
) -> String {
    let mut builder = RArgumentBuilder::new(&sig.inputs);
    if matches!(sig.inputs.first(), Some(syn::FnArg::Receiver(_))) {
        builder = builder.skip_first();
    }
    builder = builder.with_defaults(defaults.clone());
    builder.build_formals()
}

/// Build R `.Call()` arguments from a Rust function signature.
///
/// Automatically skips `self`/`&self` receivers (those are passed separately
/// via [`DotCallBuilder::with_self`]). Dots become `list(...)`.
///
/// Returns a comma-separated string of R call arguments, e.g., `"x, y, list(...)"`.
pub(crate) fn build_r_call_args_from_sig(sig: &syn::Signature) -> String {
    let mut builder = RArgumentBuilder::new(&sig.inputs);
    if matches!(sig.inputs.first(), Some(syn::FnArg::Receiver(_))) {
        builder = builder.skip_first();
    }
    builder.build_call_args()
}

// region: Missing<T> detection for automatic defaults

/// Check if a type is `Missing<T>` by examining the last path segment.
///
/// `Missing<T>` is the miniextendr wrapper for R's "missing argument" concept,
/// allowing Rust functions to accept optional arguments that R callers can omit.
pub(crate) fn is_missing_type(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Path(tp) => tp
            .path
            .segments
            .last()
            .map(|s| s.ident == "Missing")
            .unwrap_or(false),
        _ => false,
    }
}

// endregion

// region: DotCallBuilder - .Call() invocation formatting

/// Which frame a generated wrapper hands to `.Call(.., .call = ..)` and uses as
/// the raise fallback (`.miniextendr_raise_condition(.val, <default>)`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CallAttribution {
    /// `.call = sys.call()`: the wrapper's own call, as written (default).
    #[default]
    Wrapper,
    /// `.call = .mx_call`, where the wrapper body first binds
    /// `.mx_call <- .miniextendr_caller_call()`: the caller's call as written,
    /// resolved by the preamble helper in `miniextendr-api/src/registry.rs`
    /// (one line per wrapper since #1552, four before). The helper looks two
    /// frames up the calling chain for the wrapper's caller. It falls back to
    /// the wrapper's own call when there is no parent frame (top level) or the
    /// parent frame is not a closure: `eval()`'d code such as a testthat block
    /// or `source()` has the `eval` primitive as its frame function, whose call
    /// names `eval`. For a `noexport` entry point behind a hand-written R
    /// function (`#[miniextendr(noexport, call = caller)]`).
    ///
    /// A standalone wrapper with this attribution also takes a trailing
    /// `.call = NULL` formal ([`CallAttribution::formal`], #1613), which the
    /// prelude hands to the helper: a hand-written helper sitting between the
    /// public function and the entry point passes its own caller's frame
    /// (`call = parent.frame()` in the helper's formals, `.call = call` in its
    /// body), and the condition names the public function. `.call` also takes a
    /// call object, used as is. S3 methods keep the zero-argument prelude: a
    /// trailing formal would break generic/method consistency.
    Caller,
    /// `.call = .mx_call`, where the wrapper body first binds
    /// `.mx_call <- if (is.null(.call)) sys.call() else
    /// .miniextendr_caller_call(.call, own = TRUE)`: the wrapper's own call,
    /// as for [`CallAttribution::Wrapper`], unless the caller passes another
    /// as the trailing `.call = NULL` formal (#1834). Selected by
    /// `#[miniextendr(call_arg)]` on a standalone function, exported or not:
    /// an R function composing exported functions (applying them through
    /// `do.call()`, say) passes `.call = environment()` and every condition
    /// names its own call. The helper resolves a frame or a call as under
    /// [`CallAttribution::Caller`]; a frame no closure owns gives the
    /// wrapper's own call. The success path without `.call` costs one
    /// `is.null()`. Never parsed from `call = ...`: it is `wrapper`
    /// attribution plus the formal, so a `Call` marker reads the resolved
    /// call.
    Argument,
    /// `.call = FALSE`: no call at all (#1851). Selected by
    /// `#[miniextendr(call = none)]` on a standalone function (an S3 method
    /// included) or by the crate default `call_attribution = "none"`, which
    /// applies to every standalone function. Every condition the wrapper
    /// raises then has a `NULL` `conditionCall()`, like R's `call. = FALSE`:
    /// the R-side checks and the `match_arg` helpers are passed `NULL`, the
    /// slot carries `FALSE`, the marker the raise helper already reads as "no
    /// call" (what `call = none` on a condition macro writes, see
    /// `ConditionCall::None`), and the raise fallback is `NULL`. There is no
    /// marker spelling: a `Call` / `CallerCall` parameter receives the call,
    /// and this attribution has none, so either one is a compile error with
    /// it, as is `call_arg` (whose `.call` would have nothing to replace).
    NoCall,
    /// `.call = environment()`: the wrapper's own frame, turned into a call
    /// only when a condition is raised (#1851). The attribution of every
    /// generated S3 method that would otherwise be `wrapper`: a standalone
    /// `s3(...)` function, an impl-block or trait `s3` / `vctrs` method
    /// ([`CallAttribution::for_s3_method`]). The raise helper and
    /// `.miniextendr_arg_error` resolve the frame with
    /// `.miniextendr_frame_call` (`miniextendr-api/src/registry.rs`), which
    /// names the generic the user called when R dispatched the method
    /// (`.Generic` is bound in the frame: `summary(x, a = 1)` for
    /// `summary.cls(x, a = 1)`, `x[i]` for `` `[.cls`(x, i) ``) and writes a
    /// replacement generic's call as the assignment, `x$name <- value`,
    /// without the value R passed. A direct call of the method keeps the
    /// method's name. The success path evaluates `environment()` instead of
    /// `sys.call()`, the same cost; the rewrite runs only on a raise.
    Generic,
    /// `.call = .mx_call`, where the wrapper body first binds
    /// `.mx_call <- .miniextendr_frame_call(environment())`: the
    /// [`CallAttribution::Generic`] call resolved before the `.Call()`, for an
    /// S3 method with a `Call` parameter, whose C wrapper binds it from the
    /// slot and so needs a call object there (#1851). Costs the frame scan on
    /// every call, so only the marker selects it.
    GenericEager,
}

impl CallAttribution {
    /// The spelling shared by the attribute (`call = wrapper | caller | none`)
    /// and the crate default (`call_attribution = "wrapper" | "caller" |
    /// "none"`).
    pub fn parse_name(name: &str) -> Option<Self> {
        match name {
            "wrapper" => Some(CallAttribution::Wrapper),
            "caller" => Some(CallAttribution::Caller),
            "none" => Some(CallAttribution::NoCall),
            _ => None,
        }
    }

    /// The attribute spelling of this attribution. [`CallAttribution::Argument`]
    /// is `wrapper` attribution with the `call_arg` option, and the two S3
    /// forms are `wrapper` attribution on a dispatched method, so all three
    /// share `wrapper`'s spelling.
    pub fn name(self) -> &'static str {
        match self {
            CallAttribution::Wrapper
            | CallAttribution::Argument
            | CallAttribution::Generic
            | CallAttribution::GenericEager => "wrapper",
            CallAttribution::Caller => "caller",
            CallAttribution::NoCall => "none",
        }
    }

    /// The parameter marker type that selects this attribution (#1566): `Call`
    /// for `wrapper`, `CallerCall` for `caller`. `none` has no marker: a
    /// marker receives the call, and `none` passes none.
    pub fn marker_name(self) -> Option<&'static str> {
        match self {
            CallAttribution::Wrapper
            | CallAttribution::Argument
            | CallAttribution::Generic
            | CallAttribution::GenericEager => Some("Call"),
            CallAttribution::Caller => Some("CallerCall"),
            CallAttribution::NoCall => None,
        }
    }

    /// The attribution of a generated S3 method (#1851): `wrapper` becomes
    /// [`CallAttribution::Generic`], or [`CallAttribution::GenericEager`] when
    /// the method takes a `Call` parameter (`has_call_marker`), which needs
    /// the call resolved before the `.Call()`. `caller` and `none` are
    /// explicit choices and stay; `Argument` never reaches an S3 method
    /// (`call_arg` is refused on them).
    pub fn for_s3_method(self, has_call_marker: bool) -> Self {
        match self {
            CallAttribution::Wrapper if has_call_marker => CallAttribution::GenericEager,
            CallAttribution::Wrapper => CallAttribution::Generic,
            other => other,
        }
    }

    /// Resolve a standalone function's attribution from its three spellings
    /// (#1566), most specific first: the `Call` / `CallerCall` parameter
    /// marker, the `call = wrapper | caller | none` attribute, the crate's
    /// `[package.metadata.miniextendr] call_attribution` default and finally
    /// the framework default, `wrapper`. A crate default of `caller` applies
    /// to internal entry points (`noexport` / `internal`) only; an exported
    /// function's caller is arbitrary user code, so it keeps `wrapper`. A
    /// crate default of `none` applies to every standalone function, like
    /// `wrapper` (#1851). The explicit spellings are validated before this
    /// runs (a `caller` marker or attribute on an exported function is a
    /// compile error, not a fallback). Class and trait methods always use
    /// `wrapper`, except the R6 / S7 lambda frames
    /// ([`DotCallBuilder::null_call_attribution`]) and the S3 methods
    /// ([`CallAttribution::for_s3_method`]).
    pub fn resolve(
        marker: Option<Self>,
        attribute: Option<Self>,
        crate_default: Option<Self>,
        internal_entry: bool,
    ) -> Self {
        marker.or(attribute).unwrap_or(match crate_default {
            Some(CallAttribution::Caller) if !internal_entry => CallAttribution::Wrapper,
            Some(default) => default,
            None => CallAttribution::Wrapper,
        })
    }

    /// Apply the `call_arg` option (#1834) to a resolved attribution:
    /// `wrapper` becomes [`CallAttribution::Argument`]. The option is
    /// validated before this runs: it is a compile error with `call = caller`
    /// or a `CallerCall` parameter (whose wrapper already takes `.call`), on
    /// an S3 method and on an `extern "C-unwind"` function, and the caller
    /// leaves the crate default out of [`CallAttribution::resolve`] when it
    /// is set, so `caller` never reaches here with it.
    pub fn with_call_arg(self, call_arg: bool) -> Self {
        match self {
            CallAttribution::Wrapper if call_arg => CallAttribution::Argument,
            other => other,
        }
    }

    /// The expression the `.Call()` line passes as `.call`: the call as
    /// written, the frame it is read from on a raise, or `FALSE` for none.
    pub fn dot_call_expr(self) -> &'static str {
        match self {
            CallAttribution::Wrapper => "sys.call()",
            CallAttribution::Caller | CallAttribution::Argument | CallAttribution::GenericEager => {
                ".mx_call"
            }
            CallAttribution::NoCall => "FALSE",
            CallAttribution::Generic => "environment()",
        }
    }

    /// The `.call = ...` argument for the `.Call()` line.
    pub fn dot_call_arg(self) -> &'static str {
        match self {
            CallAttribution::Wrapper => ".call = sys.call()",
            CallAttribution::Caller | CallAttribution::Argument | CallAttribution::GenericEager => {
                ".call = .mx_call"
            }
            CallAttribution::NoCall => ".call = FALSE",
            CallAttribution::Generic => ".call = environment()",
        }
    }

    /// The fallback call handed to `.miniextendr_raise_condition`: the same
    /// value as the slot, except that `none` passes `NULL` (the helper reads
    /// `FALSE` in the slot as "no call" and would read it here too; `NULL` is
    /// the plainer spelling).
    pub fn raise_default(self) -> &'static str {
        match self {
            CallAttribution::Caller | CallAttribution::Argument | CallAttribution::GenericEager => {
                ".mx_call"
            }
            CallAttribution::Wrapper => "sys.call()",
            CallAttribution::NoCall => "NULL",
            CallAttribution::Generic => "environment()",
        }
    }

    /// The statement the wrapper body needs before anything else: empty for
    /// [`CallAttribution::Wrapper`], [`CallAttribution::NoCall`] and
    /// [`CallAttribution::Generic`]; the others bind `.mx_call`. It is the
    /// first part of the wrapper prelude, ahead of the R-side checks
    /// (preconditions, `match.arg`), so that those checks can attribute their
    /// failures to the caller too (#1548). `call_formal` says whether the
    /// wrapper has the `.call` formal ([`CallAttribution::formal`]); the
    /// helper then resolves whatever the caller passed there (#1613). Under
    /// [`CallAttribution::Argument`] a `NULL` `.call` is the wrapper's own
    /// call, read inline so that a call without `.call` costs one `is.null()`
    /// (#1834); the wrapper always has the formal there, since the option is
    /// refused on S3 methods. [`CallAttribution::GenericEager`] resolves the
    /// method frame's call up front (#1851).
    pub fn prelude(self, call_formal: bool) -> String {
        match self {
            CallAttribution::Caller if call_formal => {
                ".mx_call <- .miniextendr_caller_call(.call)".to_string()
            }
            CallAttribution::Caller => ".mx_call <- .miniextendr_caller_call()".to_string(),
            CallAttribution::Argument => ".mx_call <- if (is.null(.call)) sys.call() else \
                 .miniextendr_caller_call(.call, own = TRUE)"
                .to_string(),
            CallAttribution::GenericEager => {
                ".mx_call <- .miniextendr_frame_call(environment())".to_string()
            }
            CallAttribution::Wrapper | CallAttribution::NoCall | CallAttribution::Generic => {
                String::new()
            }
        }
    }

    /// The trailing formal a standalone wrapper with this attribution takes:
    /// `.call = NULL` for [`CallAttribution::Caller`] (#1613), so a
    /// hand-written helper between the public function and the entry point
    /// can pass on the call to report, and for [`CallAttribution::Argument`]
    /// (#1834), so an R function composing exported functions can. `NULL`
    /// keeps the wrapper's default (the caller's call, or its own); an
    /// environment names the closure owning that frame (`parent.frame()` from
    /// a helper, `environment()` from the composing function); a call object
    /// is used as is. Rust identifiers cannot start with `.`, so the name
    /// never collides with a parameter. The caller leaves S3 methods out.
    pub fn formal(self) -> Option<&'static str> {
        match self {
            CallAttribution::Caller | CallAttribution::Argument => Some(".call = NULL"),
            CallAttribution::Wrapper
            | CallAttribution::NoCall
            | CallAttribution::Generic
            | CallAttribution::GenericEager => None,
        }
    }

    /// The generated `@param` text for the [`CallAttribution::formal`]. It is
    /// written as the author's own line would be, not as a filler, so a block
    /// on a page or an inheritance source no author block documents `.call`
    /// for still documents it (see `crate::roxygen::push_fn_param_tags`).
    pub fn param_doc(self) -> Option<&'static str> {
        match self {
            CallAttribution::Caller => Some(
                "The call conditions from this function report: NULL (the default) for \
                 the calling function's call, a frame such as parent.frame() for the call \
                 of the function owning that frame, or a call object. Pass it by name.",
            ),
            CallAttribution::Argument => Some(
                "The call conditions from this function report: NULL (the default) for \
                 this call, a frame such as environment() for the call of the function \
                 owning that frame, or a call object. Pass it by name.",
            ),
            CallAttribution::Wrapper
            | CallAttribution::NoCall
            | CallAttribution::Generic
            | CallAttribution::GenericEager => None,
        }
    }

    /// The call an R-side check raised in the wrapper body should carry:
    /// `.mx_call` when the prelude binds it, `NULL` for `none`, the frame
    /// (`environment()`, which `.miniextendr_arg_error` resolves) for an S3
    /// method, otherwise `None` (the check keeps its own attribution, which
    /// is the wrapper's frame).
    pub fn r_check_call(self) -> Option<&'static str> {
        match self {
            CallAttribution::Caller | CallAttribution::Argument | CallAttribution::GenericEager => {
                Some(".mx_call")
            }
            CallAttribution::NoCall => Some("NULL"),
            CallAttribution::Generic => Some("environment()"),
            CallAttribution::Wrapper => None,
        }
    }

    /// The R statement validating a choice parameter (`match_arg` / `choices`).
    /// `choices` is the R expression for the choice list: a literal
    /// `c("a", "b")`, or the write-time placeholder for an enum. `attrs` are
    /// the parameter's own attributes, which carry `several_ok` and the
    /// type's layers.
    ///
    /// Every form is one call to a preamble helper (`.miniextendr_match_arg`
    /// for a scalar, `.miniextendr_match_arg_several` for `several_ok`,
    /// #1472), guarded by the layers of the parameter type: `!missing(..)`
    /// for `Missing<..>` (#1551), then `!is.null(..)` for `Option<..>`
    /// (#1473), or `is.character(..) || is.factor(..)` for `Either<T, R>` and
    /// `Either<Vec<T>, R>` (#1612), which leaves `NULL` out too: Rust reads it
    /// as `None` under `Option`, and otherwise converts it to the `R` arm, even
    /// for `several_ok`, whose plain form reads `NULL` as every choice. The
    /// helpers name the argument in their messages, read a factor as its
    /// labels, and attribute the error to the wrapper's own call by default;
    /// under [`CallAttribution::Caller`] the statement passes `.mx_call` so the
    /// caller is named instead (#1548), and under
    /// [`CallAttribution::Argument`] so that the call passed as `.call` is.
    /// The list is spelled out because the helpers, unlike
    /// `base::match.arg(param)`, do not read it off the formal.
    ///
    /// `aliases` is the `match_arg` parameter's write-time
    /// `match_arg_keys::aliases_placeholder`, written last as `, <placeholder>`:
    /// the wrapper writer turns it into the type's `aliases =` argument, or
    /// removes it for a type without aliases (#1843). `None` for a literal
    /// `choices(...)` list, which has no aliases.
    pub fn match_arg_statement(
        self,
        param: &str,
        choices: &str,
        aliases: Option<&str>,
        attrs: &crate::miniextendr_fn::ParamAttrs,
    ) -> String {
        let helper = if attrs.several_ok {
            ".miniextendr_match_arg_several"
        } else {
            ".miniextendr_match_arg"
        };
        let call = match self.r_check_call() {
            Some(call) => format!(", {call}"),
            None => String::new(),
        };
        let aliases = match aliases {
            Some(placeholder) => format!(", {placeholder}"),
            None => String::new(),
        };
        let statement =
            format!("{param} <- {helper}({param}, {choices}, \"{param}\"{call}{aliases})");
        let mut guards = Vec::new();
        if attrs.omittable {
            guards.push(format!("!missing({param})"));
        }
        if attrs.either_noun.is_some() {
            // Only the forms `match.arg()` reads are choices; everything else
            // (NULL included) goes to the `Either`'s `R` arm unchanged.
            let reads = format!("is.character({param}) || is.factor({param})");
            guards.push(if guards.is_empty() {
                reads
            } else {
                format!("({reads})")
            });
        } else if attrs.optional {
            guards.push(format!("!is.null({param})"));
        }
        if guards.is_empty() {
            statement
        } else {
            format!("if ({}) {statement}", guards.join(" && "))
        }
    }
}

/// Append a standalone wrapper's [`CallAttribution::formal`] to its joined
/// `formals` (#1613). The formal goes last: after `...` and after any formal
/// that follows the dots, so positional extras land in the dots and `.call`
/// is matched by name only; a wrapper without other formals takes it alone.
pub(crate) fn with_call_formal(formals: &str, call_formal: Option<&str>) -> String {
    match call_formal {
        Some(formal) if formals.is_empty() => formal.to_string(),
        Some(formal) => format!("{formals}, {formal}"),
        None => formals.to_string(),
    }
}

/// Builder for formatting `.Call()` invocations in R wrapper code.
///
/// Handles the common pattern of `.Call(C_ident, .call = sys.call(), args...)`.
///
/// # Example
///
/// ```ignore
/// let call = DotCallBuilder::new("C_Counter__increment")
///     .with_self("self")
///     .build();
/// // => ".Call(C_Counter__increment, .call = sys.call(), self)"
///
/// let call = DotCallBuilder::new("C_Counter__add")
///     .with_self("x")
///     .with_args(&["n"])
///     .build();
/// // => ".Call(C_Counter__add, .call = sys.call(), x, n)"
/// ```
pub struct DotCallBuilder {
    /// The C entry point symbol name (e.g., `"C_Counter__increment"`).
    /// This is the first argument to `.Call()`.
    c_ident: String,
    /// Optional self/receiver variable name (e.g., `"self"`, `"x"`).
    /// When present, prepended before other arguments in the `.Call()` invocation.
    self_var: Option<String>,
    /// Additional argument names passed after self (if any) in the `.Call()` invocation.
    args: Vec<String>,
    /// Expression for the `.call` named argument. `None` means `sys.call()` (the default).
    /// Set via [`DotCallBuilder::null_call_attribution`] to emit `.call = NULL` instead.
    call_expr: Option<String>,
}

impl DotCallBuilder {
    /// Create a new builder with the C function identifier.
    pub fn new(c_ident: impl Into<String>) -> Self {
        Self {
            c_ident: c_ident.into(),
            self_var: None,
            args: Vec::new(),
            call_expr: None,
        }
    }

    /// Add a self/x parameter (prepended to args).
    pub fn with_self(mut self, var: impl Into<String>) -> Self {
        self.self_var = Some(var.into());
        self
    }

    /// Add arguments after self (if any).
    pub fn with_args(mut self, args: &[impl AsRef<str>]) -> Self {
        self.args = args.iter().map(|s| s.as_ref().to_string()).collect();
        self
    }

    /// Add a pre-joined argument string (e.g., `"x, y"`) as a single emit unit.
    ///
    /// Empty strings are ignored, so callers can pass the result of
    /// `build_r_call_args_from_sig` directly without a length check.
    pub fn with_args_str(mut self, args: &str) -> Self {
        if !args.is_empty() {
            self.args.push(args.to_string());
        }
        self
    }

    /// Pass `.call = NULL` instead of `.call = sys.call()`: for the R6 / S7
    /// lambda frames, which R6 / S7 dispatch calls (R6 finalizer / `deep_clone`, S7
    /// property getter / setter / validator), where `sys.call()` names an
    /// internal dispatch frame instead of the user's call. Not reachable from
    /// any attribute. With `NULL`, the raise helper's fallback (a `NULL`
    /// `.val$call` takes `.call_default`, the `sys.call()` in
    /// `condition_check_lines`; only a call-less condition's `FALSE` marker
    /// gives `NULL`) surfaces the nearest meaningful frame instead.
    pub fn null_call_attribution(mut self) -> Self {
        self.call_expr = Some("NULL".to_string());
        self
    }

    /// Pass the `.call` of `attribution` ([`CallAttribution::dot_call_expr`]):
    /// `environment()` for a generated S3 method
    /// ([`CallAttribution::Generic`], #1851), which the raise helper turns into
    /// the generic's call when a condition is raised.
    pub fn with_call_attribution(mut self, attribution: CallAttribution) -> Self {
        self.call_expr = Some(attribution.dot_call_expr().to_string());
        self
    }

    /// Build the `.Call()` string.
    pub fn build(&self) -> String {
        let call_arg = self.call_expr.as_deref().unwrap_or("sys.call()");

        let mut all_args = Vec::new();

        if let Some(ref self_var) = self.self_var {
            all_args.push(self_var.clone());
        }
        all_args.extend(self.args.clone());

        if all_args.is_empty() {
            format!(".Call({}, .call = {})", self.c_ident, call_arg)
        } else {
            format!(
                ".Call({}, .call = {}, {})",
                self.c_ident,
                call_arg,
                all_args.join(", ")
            )
        }
    }
}
// endregion

// region: RoxygenBuilder - roxygen2 documentation tag generation

/// Builder for generating roxygen2 documentation tags.
///
/// Provides a fluent API for building common roxygen tag patterns used
/// across all class systems.
///
/// # Example
///
/// ```ignore
/// let tags = RoxygenBuilder::new()
///     .name("Counter$increment")
///     .rdname("Counter")
///     .export()
///     .build();
/// // => vec!["#' @name Counter$increment", "#' @rdname Counter", "#' @export"]
/// ```
pub struct RoxygenBuilder {
    /// Value for `@name` tag. Identifies the documented topic (e.g., `"Counter$increment"`).
    name: Option<String>,
    /// Value for `@rdname` tag. Groups multiple entries onto a single help page
    /// (e.g., all methods of `"Counter"` share one Rd file).
    rdname: Option<String>,
    /// Value for `@title` tag. The one-line title shown in help page headers.
    title: Option<String>,
    /// Value for `@description` tag. Longer description text below the title.
    description: Option<String>,
    /// Value for `@source` tag. Typically `"Generated by miniextendr"` provenance info.
    source: Option<String>,
    /// Whether to emit `@export`. When true, the item is exported from the package NAMESPACE.
    export: bool,
    /// Value for `@exportMethod` tag. Used for S4 method exports (e.g., `"show"`).
    export_method: Option<String>,
    /// Values for `@method` tag as `(generic, class)`. Used for S3 method dispatch
    /// (e.g., `("print", "Counter")` emits `@method print Counter`).
    method: Option<(String, String)>,
    /// Additional custom tag lines emitted verbatim (without the `#' ` prefix,
    /// which is added during [`build`](Self::build)). Used for tags like
    /// `@keywords internal` or `@param` entries.
    custom_tags: Vec<String>,
}

impl RoxygenBuilder {
    /// Create a new empty builder.
    pub fn new() -> Self {
        Self {
            name: None,
            rdname: None,
            title: None,
            description: None,
            source: None,
            export: false,
            export_method: None,
            method: None,
            custom_tags: Vec::new(),
        }
    }

    /// Set the `@name` tag.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the `@rdname` tag (groups docs into one page).
    pub fn rdname(mut self, rdname: impl Into<String>) -> Self {
        self.rdname = Some(rdname.into());
        self
    }

    /// Set the `@title` tag.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the `@description` tag.
    #[allow(dead_code)] // Exercised by tests
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set the `@source` tag (typically "Generated by miniextendr...").
    pub fn source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Add `@export` tag.
    pub fn export(mut self) -> Self {
        self.export = true;
        self
    }

    /// Add `@exportMethod` tag (for S4).
    #[allow(dead_code)] // Exercised by tests
    pub fn export_method(mut self, method: impl Into<String>) -> Self {
        self.export_method = Some(method.into());
        self
    }

    /// Add `@method` tag (for S3).
    pub fn method(mut self, generic: impl Into<String>, class: impl Into<String>) -> Self {
        self.method = Some((generic.into(), class.into()));
        self
    }

    /// Add a custom tag line (without the `#' ` prefix).
    pub fn custom(mut self, tag: impl Into<String>) -> Self {
        self.custom_tags.push(tag.into());
        self
    }

    /// Build the roxygen tag lines (each prefixed with `#' `).
    pub fn build(&self) -> Vec<String> {
        let mut lines = Vec::new();

        if let Some(ref title) = self.title {
            lines.push(format!("#' @title {}", title));
        }
        if let Some(ref desc) = self.description {
            lines.push(format!("#' @description {}", desc));
        }
        if let Some(ref name) = self.name {
            lines.push(format!("#' @name {}", name));
        }
        if let Some(ref rdname) = self.rdname {
            lines.push(format!("#' @rdname {}", rdname));
        }
        if let Some(ref source) = self.source {
            lines.extend(crate::roxygen::source_tag(source));
        }
        if let Some((ref generic, ref class)) = self.method {
            lines.push(format!("#' @method {} {}", generic, class));
        }
        // A tag the author wrapped (`@param`, `@describeIn`, ...) keeps its
        // continuation lines, each with its own `#' ` lead.
        crate::roxygen::push_roxygen_tags(&mut lines, &self.custom_tags);
        if self.export {
            lines.push("#' @export".to_string());
        }
        if let Some(ref method) = self.export_method {
            lines.push(format!("#' @exportMethod {}", method));
        }

        lines
    }
}

/// Creates an empty builder with no tags set.
impl Default for RoxygenBuilder {
    fn default() -> Self {
        Self::new()
    }
}
// endregion

// region: Tests

#[cfg(test)]
mod tests;
// endregion
