//! Function signature parsing for `#[miniextendr]`.
//!
//! This module handles parsing and normalizing Rust function signatures for the
//! `#[miniextendr]` attribute macro. It provides:
//!
//! - [`MiniextendrFunctionParsed`]: Parsed function with normalization and codegen helpers
//! - [`MiniextendrFnAttrs`]: Parsed `#[miniextendr(...)]` attribute options
//! - [`CoercionMapping`]: Type coercion analysis for automatic R→Rust conversion

use crate::r_wrapper_const_ident_for;

// region: Coercion analysis

/// Conversion selected by `coerce`, shared by Rust conversion and R checks.
pub(crate) enum CoercionMapping {
    /// Numeric scalars and vectors already have checked, multi-source converters.
    Numeric,
    /// Keep logical scalars and additionally accept integer zero or one.
    Bool,
    /// Keep logical vectors and additionally accept integer zeros and ones.
    BoolVec,
    /// Native `i32`: keep `INTSXP`, additionally accept whole-valued doubles,
    /// logicals, and raws.
    NativeInt,
    /// Native `Vec<i32>`: element-wise form of [`Self::NativeInt`].
    NativeIntVec,
    /// Native `f64`: keep `REALSXP`, additionally accept integers, logicals, and raws.
    NativeReal,
    /// Native `Vec<f64>`: element-wise form of [`Self::NativeReal`].
    NativeRealVec,
}

impl CoercionMapping {
    /// Recognize the scalar and vector types supported by `coerce`.
    ///
    /// Numeric conversion preserves all sources accepted by `TryFromSexp`:
    /// integer, double, logical, and raw. Booleans extend their logical-only
    /// converter with integer zero/one input. The native `i32` / `f64` scalars
    /// and vectors, whose bare conversion accepts one `SEXPTYPE`, widen to the
    /// same four sources (whole-valued doubles only for `i32`). Borrowed slices
    /// and other types keep `TryFromSexp`.
    pub(crate) fn from_type(ty: &syn::Type) -> Option<Self> {
        let syn::Type::Path(type_path) = ty else {
            return None;
        };
        let seg = type_path.path.segments.last()?;
        let is_vec = seg.ident == "Vec";
        let ident = if is_vec {
            let syn::PathArguments::AngleBracketed(args) = &seg.arguments else {
                return None;
            };
            let syn::GenericArgument::Type(syn::Type::Path(inner)) = args.args.first()? else {
                return None;
            };
            &inner.path.segments.last()?.ident
        } else {
            &seg.ident
        };
        match ident.to_string().as_str() {
            "i8" | "i16" | "u16" | "u32" | "i64" | "u64" | "isize" | "usize" | "f32" => {
                Some(Self::Numeric)
            }
            "bool" if is_vec => Some(Self::BoolVec),
            "bool" => Some(Self::Bool),
            "i32" if is_vec => Some(Self::NativeIntVec),
            "i32" => Some(Self::NativeInt),
            "f64" if is_vec => Some(Self::NativeRealVec),
            "f64" => Some(Self::NativeReal),
            _ => None,
        }
    }
}

// endregion

// region: Type inspection helpers

/// Check if a type path ends with the given identifier (e.g., "Dots", "Missing").
///
/// Handles fully-qualified paths like `miniextendr_api::dots::Dots` as well as
/// bare `Dots`.
fn type_ends_with(ty: &syn::Type, name: &str) -> bool {
    match ty {
        syn::Type::Path(tp) => tp
            .path
            .segments
            .last()
            .map(|s| s.ident == name)
            .unwrap_or(false),
        syn::Type::Reference(r) => type_ends_with(&r.elem, name),
        _ => false,
    }
}

/// Check if a type is `Dots` or `&Dots` (the variadic `...` parameter type).
pub(crate) fn is_dots_type(ty: &syn::Type) -> bool {
    type_ends_with(ty, "Dots")
}

/// Replace Rust variadic syntax (`name: ...` / `_: ...`) with a trailing
/// `&miniextendr_api::dots::Dots` parameter, so downstream codegen never
/// emits a non-extern variadic Rust fn. Named dots keep the user's
/// identifier; `_: ...` binds `__miniextendr_dots`.
///
/// A bare `...` binds `__miniextendr_dots` too, but rustc rejects it before
/// the macro runs (the deny-by-default `varargs_without_pattern` lint,
/// rust-lang/rust#145544) unless the crate allows that lint, and its fix is
/// `_: ...` (#1743).
///
/// syn parses the variadic only in last position, so a formal after the dots
/// is spelled with an explicit `rest: &Dots` parameter instead. A signature
/// with both is an error: a function takes at most one `...`.
pub(crate) fn rewrite_variadic_dots(sig: &mut syn::Signature) -> syn::Result<()> {
    use syn::spanned::Spanned;

    let Some(variadic) = sig.variadic.take() else {
        return Ok(());
    };
    if let Some(syn::FnArg::Typed(existing)) = dots_index(&sig.inputs).map(|idx| &sig.inputs[idx]) {
        let existing = match existing.pat.as_ref() {
            syn::Pat::Ident(pat_ident) => format!("`{}: &Dots`", pat_ident.ident),
            _ => "a `&Dots` parameter".to_string(),
        };
        return Err(syn::Error::new(
            variadic.span(),
            format!("this function already takes `...` as {existing}; remove one of them"),
        ));
    }
    let ident = match variadic.pat.map(|(pat, _)| *pat) {
        Some(syn::Pat::Ident(pat_ident)) => pat_ident.ident,
        // `_: ...` (rustc's fix for a bare `...`) and a bare `...`.
        Some(syn::Pat::Wild(_)) | None => {
            // Cannot use `_` as a variable name, so unnamed dots need a
            // stable synthetic binding that does not collide with user args.
            for arg in &sig.inputs {
                let syn::FnArg::Typed(pat_type) = arg else {
                    continue;
                };
                if let syn::Pat::Ident(pat_ident) = pat_type.pat.as_ref()
                    && pat_ident.ident == "__miniextendr_dots"
                {
                    return Err(syn::Error::new(
                        pat_ident.ident.span(),
                        "parameter named `__miniextendr_dots` conflicts with the binding of \
                         unnamed dots (`_: ...`); name the dots instead, e.g. `args: ...`",
                    ));
                }
            }
            syn::Ident::new("__miniextendr_dots", proc_macro2::Span::call_site())
        }
        Some(pat) => {
            return Err(syn::Error::new(
                pat.span(),
                "variadic pattern must be a simple identifier (`args: ...`) or `_` (`_: ...`)",
            ));
        }
    };
    sig.inputs
        .push(syn::parse_quote!(#ident: &::miniextendr_api::dots::Dots));
    Ok(())
}

/// Position of the first `&Dots` parameter in `inputs` (receiver included, as
/// [`RArgumentBuilder`](crate::r_wrapper_builder::RArgumentBuilder) counts
/// them), without validation: for code that runs after [`find_dots_param`]
/// has accepted the signature.
pub(crate) fn dots_index(
    inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
) -> Option<usize> {
    inputs
        .iter()
        .position(|arg| matches!(arg, syn::FnArg::Typed(pt) if is_dots_type(pt.ty.as_ref())))
}

/// Find the dots parameter of a signature by type, at any position, and
/// return its Rust binding.
///
/// The parameter of type `&Dots` is R's `...` at its own position: the R
/// formals and the `.Call()` arguments follow the Rust signature order, with
/// `...` / `list(...)` where it sits, and every formal after it is matched by
/// exact name only, as R does for any formal after `...`.
///
/// A function takes at most one `...`, so a second `&Dots` parameter is an
/// error spanned on it. The parameter needs a plain name: the body reads the
/// dots through it.
pub(crate) fn find_dots_param(
    inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
) -> syn::Result<Option<syn::Ident>> {
    use syn::spanned::Spanned;

    let mut found: Option<syn::Ident> = None;
    for arg in inputs {
        let syn::FnArg::Typed(pat_type) = arg else {
            continue;
        };
        if !is_dots_type(pat_type.ty.as_ref()) {
            continue;
        }
        let syn::Pat::Ident(pat_ident) = pat_type.pat.as_ref() else {
            return Err(syn::Error::new(
                pat_type.pat.span(),
                "the `...` parameter needs a plain name, for example `rest: &Dots`",
            ));
        };
        let ident = &pat_ident.ident;
        if let Some(first) = &found {
            return Err(syn::Error::new(
                ident.span(),
                format!(
                    "a function takes at most one `...`: `{first}` and `{ident}` both have type \
                     `&Dots`; keep one"
                ),
            ));
        }
        found = Some(ident.clone());
    }
    Ok(found)
}

/// Check if a type is `Missing<T>`.
pub(crate) fn is_missing_type(ty: &syn::Type) -> bool {
    type_ends_with(ty, "Missing")
}

/// Check if a type is a vector-like type that `several_ok` can populate.
///
/// Accepts `Vec<T>`, `Box<[T]>`, `&[T]` / `&mut [T]`, and `[T; N]`. Rejects
/// scalar types (like `Mode`, `String`, `&str`) so `several_ok` — which
/// produces a multi-element R character vector via
/// `match.arg(..., several.ok = TRUE)` — fails at compile time instead of
/// deserialization time.
pub(crate) fn is_vector_like_type(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Path(tp) => {
            let Some(seg) = tp.path.segments.last() else {
                return false;
            };
            if seg.ident == "Vec" {
                return true;
            }
            if seg.ident == "Box" {
                let syn::PathArguments::AngleBracketed(args) = &seg.arguments else {
                    return false;
                };
                return matches!(
                    args.args.first(),
                    Some(syn::GenericArgument::Type(syn::Type::Slice(_)))
                );
            }
            false
        }
        syn::Type::Reference(r) => matches!(&*r.elem, syn::Type::Slice(_)),
        syn::Type::Slice(_) => true,
        syn::Type::Array(_) => true,
        _ => false,
    }
}

/// Extract the inner type `T` from `Missing<T>`, if the type is `Missing<T>`.
///
/// Returns `None` if the type is not `Missing<T>` or has no generic argument.
pub(crate) fn get_missing_inner_type(ty: &syn::Type) -> Option<&syn::Type> {
    let syn::Type::Path(tp) = ty else {
        return None;
    };
    let seg = tp.path.segments.last()?;
    if seg.ident != "Missing" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(args) = &seg.arguments else {
        return None;
    };
    if let Some(syn::GenericArgument::Type(inner)) = args.args.first() {
        Some(inner)
    } else {
        None
    }
}

/// Validate a parameter's type for `Missing` and `Dots` conflicts.
///
/// Returns `Err` if:
/// - `Missing<Missing<T>>` (nested Missing)
/// - `Missing<Dots>` or `Missing<&Dots>`
pub(crate) fn validate_param_type(ty: &syn::Type, span: proc_macro2::Span) -> syn::Result<()> {
    if crate::return_wrap::contains_marker(ty) {
        return Err(syn::Error::new_spanned(
            ty,
            "class return markers are return-position only",
        ));
    }
    if let Some(err) = crate::type_inspect::visibility_marker_error(ty, "argument") {
        return Err(err);
    }
    if let Some(inner) = get_missing_inner_type(ty) {
        if is_missing_type(inner) {
            return Err(syn::Error::new(
                span,
                "Missing<T> cannot be nested; use Missing<T> with the inner type directly",
            ));
        }
        if is_dots_type(inner) {
            return Err(syn::Error::new(
                span,
                "Missing<T> cannot wrap Dots; variadic parameters (...) are always present when called",
            ));
        }
    }
    Ok(())
}

/// Validate per-parameter attribute conflicts.
///
/// Returns `Err` if:
/// - `coerce` + `match_arg` on the same parameter
/// - `coerce` + `choices(...)` on the same parameter
/// - `choices(...)` + explicit `default` on the same parameter
/// - `match_arg` + `choices(...)` on the same parameter (two sources for one
///   choice list; the `match_arg` placeholder would shadow the literal list
///   and, with no `MatchArg` entry to resolve it, dangle in the R formals)
/// - `no_default` on a parameter that is neither `match_arg` nor `choices`,
///   or together with `default` (#1828)
/// - `default` on a `&Dots` parameter
///
/// The type-dependent checks of a choice parameter live in
/// [`classify_choice_param`], shared with the method paths.
pub(crate) fn validate_per_param_attr_conflicts(
    attr: &PerParamMiniextendrAttr,
    param_name: &str,
    is_dots: bool,
    ty: Option<&syn::Type>,
    span: proc_macro2::Span,
) -> syn::Result<()> {
    if attr.has_no_default
        && let Some(msg) = no_default_conflict(
            param_name,
            attr.has_match_arg || attr.choices.is_some(),
            attr.default_value.is_some(),
            false,
        )
    {
        return Err(syn::Error::new(span, msg));
    }
    if attr.has_coerce && attr.has_match_arg {
        return Err(syn::Error::new(
            span,
            format!(
                "cannot combine coerce and match_arg on parameter `{}`; \
                 coerce converts the R type while match_arg validates string values",
                param_name
            ),
        ));
    }
    if attr.has_coerce && attr.choices.is_some() {
        return Err(syn::Error::new(
            span,
            format!(
                "cannot combine coerce and choices on parameter `{}`; \
                 coerce converts the R type while choices validates string values",
                param_name
            ),
        ));
    }
    if attr.choices.is_some() && attr.default_value.is_some() {
        return Err(syn::Error::new(
            span,
            format!(
                "cannot combine choices() and default on parameter `{}`; \
                 choices auto-generates its default from the first choice value",
                param_name
            ),
        ));
    }
    if attr.has_match_arg && attr.choices.is_some() {
        return Err(syn::Error::new(
            span,
            format!(
                "cannot combine match_arg and choices() on parameter `{}`; \
                 match_arg takes the choice list from the parameter type's `MatchArg` impl, \
                 choices() supplies a literal list for a string parameter; use one of them",
                param_name
            ),
        ));
    }
    if attr.has_several_ok && attr.choices.is_none() && !attr.has_match_arg {
        return Err(syn::Error::new(
            span,
            format!(
                "several_ok requires choices() or match_arg on parameter `{}`; \
                 several_ok enables multi-value match.arg which needs a choice list",
                param_name
            ),
        ));
    }
    if is_dots && !attr.checks.is_empty() {
        return Err(syn::Error::new(
            span,
            format!(
                "`inherits` / `not_inherits` / `no_na` cannot apply to the variadic (...) \
                 parameter `{}`; use `dots = typed_list!(...)` to check the entries",
                param_name
            ),
        ));
    }
    for check in [ClassCheck::Inherits, ClassCheck::NotInherits] {
        if let Some(classes) = check.classes(&attr.checks)
            && (classes.is_empty() || classes.iter().any(String::is_empty))
        {
            let key = check.keyword();
            return Err(syn::Error::new(
                span,
                format!(
                    "`{key}` on parameter `{param_name}` needs one or more non-empty class \
                     names, e.g. `{key} = \"pkg_obj\"` or `{key}(\"pkg_a\", \"pkg_b\")`"
                ),
            ));
        }
    }
    if let Some(msg) = attr.checks.class_conflict(param_name) {
        return Err(syn::Error::new(span, msg));
    }
    if is_dots && attr.default_value.is_some() {
        return Err(syn::Error::new(
            span,
            format!(
                "variadic (...) parameter `{}` cannot have a default value",
                param_name
            ),
        ));
    }
    if let Some(ty) = ty
        && is_missing_type(ty)
        && attr.default_value.is_some()
    {
        return Err(syn::Error::new(
            span,
            format!(
                "`Missing<T>` parameter `{}` cannot have a default value. \
                 `Missing<T>` detects omitted arguments via `missing()` in R, \
                 which is incompatible with default values in the R function signature. \
                 Use `Option<T>` with `#[miniextendr(default = \"...\")]` instead.",
                param_name
            ),
        ));
    }
    Ok(())
}

/// The error of a `no_default` that cannot apply (#1828), or `None`: on a
/// parameter that is neither `match_arg` nor `choices` (only a choice
/// parameter gets a generated default to drop), or together with a default.
/// `method_list` picks the spellings named: the method-level
/// `no_default(p)` / `defaults(p = ...)`, else the parameter's own
/// `no_default` / `default = "..."`.
pub(crate) fn no_default_conflict(
    param: &str,
    is_choice: bool,
    has_default: bool,
    method_list: bool,
) -> Option<String> {
    let (no_default, default) = if method_list {
        (
            format!("`no_default({param})`"),
            format!("`defaults({param} = ...)`"),
        )
    } else {
        (
            "`no_default`".to_string(),
            "`default = \"...\"`".to_string(),
        )
    };
    if !is_choice {
        return Some(format!(
            "{no_default} on parameter `{param}`, which is neither `match_arg` nor `choices`; \
             only a choice parameter gets a generated default to drop, and any other \
             parameter without a default already has a bare formal"
        ));
    }
    if has_default {
        return Some(format!(
            "cannot combine {no_default} and {default} on parameter `{param}`; \
             `no_default` leaves the R formal without a default, so drop one of them"
        ));
    }
    None
}

// endregion

// region: Per-parameter attribute parsing

/// Parsed per-parameter `#[miniextendr(...)]` attribute content.
///
/// A single attribute can contain multiple items, e.g.
/// `#[miniextendr(match_arg, default = "Safe")]`; a parameter carrying
/// several attributes gets them merged ([`PerParamMiniextendrAttr::merge`]).
#[derive(Default)]
pub(crate) struct PerParamMiniextendrAttr {
    /// Whether `coerce` was present, enabling automatic type coercion for this parameter
    /// (e.g., `i32` to `u16`, `f64` to `f32`).
    pub has_coerce: bool,
    /// Whether `match_arg` was present, generating R `match.arg()` validation for
    /// string parameters against a set of allowed values.
    pub has_match_arg: bool,
    /// Default value from `default = "..."`, if present. The tuple contains the default
    /// value string and the attribute span (for error reporting).
    pub default_value: Option<(String, proc_macro2::Span)>,
    /// Choices for string parameters: `#[miniextendr(choices("a", "b", "c"))]`.
    pub choices: Option<Vec<String>>,
    /// Whether `several_ok` was present, enabling multi-value `match.arg(several.ok = TRUE)`.
    /// Only valid with `choices(...)` or `match_arg`.
    pub has_several_ok: bool,
    /// Whether `no_default` was present: the choice parameter's R formal has
    /// no default (#1828). Only valid with `choices(...)` or `match_arg`, and
    /// not together with `default`.
    pub has_no_default: bool,
    /// `inherits = "cls"` / `inherits("a", "b")`, `not_inherits` (the same
    /// spellings) and `no_na`, each with an optional `message = "..."`: R-side
    /// checks named by the author (see
    /// [`crate::r_preconditions::ExplicitChecks`]).
    pub checks: crate::r_preconditions::ExplicitChecks,
    /// `preconditions` / `no_preconditions` on the parameter (#1566):
    /// `Some(true)` keeps its type-derived R-side checks, `Some(false)` drops
    /// them. The last one written wins.
    pub preconditions: Option<bool>,
}

impl PerParamMiniextendrAttr {
    /// Merge the options of another attribute on the same parameter: flags
    /// add up, the first `default` / `choices(...)` wins, the last
    /// `preconditions` / `no_preconditions` wins, and the checks merge (one
    /// `message` per check; the error says which).
    pub(crate) fn merge(&mut self, other: PerParamMiniextendrAttr) -> Result<(), String> {
        if other.preconditions.is_some() {
            self.preconditions = other.preconditions;
        }
        self.has_coerce |= other.has_coerce;
        self.has_match_arg |= other.has_match_arg;
        self.has_several_ok |= other.has_several_ok;
        self.has_no_default |= other.has_no_default;
        if self.default_value.is_none() {
            self.default_value = other.default_value;
        }
        if self.choices.is_none() {
            self.choices = other.choices;
        }
        self.checks.merge(other.checks)
    }
}

/// Parse all per-parameter options from a `#[miniextendr(...)]` attribute.
///
/// Handles mixed content like `#[miniextendr(match_arg, default = "\"Safe\"")]`
/// and `#[miniextendr(choices("a", "b", "c"))]`.
///
/// Returns `Ok(None)` if `attr` is not a `#[miniextendr(...)]` attribute, if its
/// content is not a list of options, or if it contains only function-level
/// options (like `strict`) with no per-parameter options. A malformed
/// `inherits(...)` / `not_inherits(...)` / `no_na(...)` is an error, as is
/// `preconditions = bool` / `no_preconditions = bool`, a function-level form
/// the parameter spells bare.
///
/// # Arguments
///
/// * `attr` - A `syn::Attribute` to inspect. Only attributes with path `miniextendr`
///   are considered.
pub(crate) fn parse_per_param_attr(
    attr: &syn::Attribute,
) -> syn::Result<Option<PerParamMiniextendrAttr>> {
    use syn::spanned::Spanned;
    if !attr.path().is_ident("miniextendr") {
        return Ok(None);
    }

    let syn::Meta::List(meta_list) = &attr.meta else {
        return Ok(None);
    };

    let mut result = PerParamMiniextendrAttr::default();
    let mut is_per_param = false;

    let Ok(metas) = meta_list.parse_args_with(
        syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
    ) else {
        return Ok(None);
    };

    for meta in &metas {
        match meta {
            syn::Meta::Path(path) => {
                if path.is_ident("coerce") {
                    result.has_coerce = true;
                    is_per_param = true;
                } else if path.is_ident("match_arg") {
                    result.has_match_arg = true;
                    is_per_param = true;
                } else if path.is_ident("several_ok") {
                    result.has_several_ok = true;
                    is_per_param = true;
                } else if path.is_ident("no_default") {
                    result.has_no_default = true;
                    is_per_param = true;
                } else if path.is_ident("no_na") {
                    result.checks.no_na = true;
                    is_per_param = true;
                } else if path.is_ident("preconditions") {
                    result.preconditions = Some(true);
                    is_per_param = true;
                } else if path.is_ident("no_preconditions") {
                    result.preconditions = Some(false);
                    is_per_param = true;
                }
                // Other paths (like `strict`) are function-level, ignore here
            }
            syn::Meta::NameValue(nv) => {
                if nv.path.is_ident("default")
                    && let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(lit_str),
                        ..
                    }) = &nv.value
                {
                    result.default_value = Some((lit_str.value(), attr.span()));
                    is_per_param = true;
                } else if let Some(check) = ClassCheck::of(&nv.path)
                    && let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(lit_str),
                        ..
                    }) = &nv.value
                {
                    check
                        .classes_mut(&mut result.checks)
                        .get_or_insert_with(Vec::new)
                        .push(lit_str.value());
                    is_per_param = true;
                } else if let Some(key) = ["preconditions", "no_preconditions"]
                    .into_iter()
                    .find(|key| nv.path.is_ident(key))
                {
                    return Err(syn::Error::new_spanned(
                        nv,
                        format!(
                            "`{key} = ...` on a parameter: write the bare `preconditions` or \
                             `no_preconditions` (or the `Checked<T>` / `Unchecked<T>` marker); \
                             the `= true | false` form is the function attribute's"
                        ),
                    ));
                }
                // Other name-value pairs are function-level, ignore here
            }
            syn::Meta::List(list) => {
                if list.path.is_ident("choices") {
                    // Parse choices("a", "b", "c") — a comma-separated list of string literals
                    let choice_lits = match list.parse_args_with(
                        syn::punctuated::Punctuated::<syn::LitStr, syn::Token![,]>::parse_terminated,
                    ) {
                        Ok(lits) => lits,
                        Err(_) => continue,
                    };
                    let choices: Vec<String> = choice_lits.iter().map(|l| l.value()).collect();
                    result.choices = Some(choices);
                    is_per_param = true;
                } else if let Some(check) = ClassCheck::of(&list.path) {
                    // inherits("a", "b"[, message = "..."]) / inherits(class = "a", ...),
                    // and the same for not_inherits.
                    let checks = parse_class_check_list(list, check)?;
                    result
                        .checks
                        .merge(checks)
                        .map_err(|msg| syn::Error::new_spanned(list, msg))?;
                    is_per_param = true;
                } else if list.path.is_ident("no_na") {
                    // no_na(message = "...")
                    let checks = parse_no_na_list(list)?;
                    result
                        .checks
                        .merge(checks)
                        .map_err(|msg| syn::Error::new_spanned(list, msg))?;
                    is_per_param = true;
                }
                // Other list forms are function-level, ignore here
            }
        }
    }

    if !is_per_param {
        return Ok(None);
    }
    Ok(Some(result))
}

/// The two class checks a parameter can name: `inherits` (the argument must
/// inherit from one of the classes) and `not_inherits` (from none of them,
/// #1815). Both take the same spellings at parameter and method level; this
/// says which one a spelling is, and words its errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ClassCheck {
    Inherits,
    NotInherits,
}

impl ClassCheck {
    /// The check `path` names, if it is `inherits` or `not_inherits`.
    pub(crate) fn of(path: &syn::Path) -> Option<Self> {
        if path.is_ident("inherits") {
            Some(Self::Inherits)
        } else if path.is_ident("not_inherits") {
            Some(Self::NotInherits)
        } else {
            None
        }
    }

    /// The attribute keyword, for error messages.
    fn keyword(self) -> &'static str {
        match self {
            Self::Inherits => "inherits",
            Self::NotInherits => "not_inherits",
        }
    }

    /// The `when(...)` hints in this check's list of options, for errors:
    /// only `inherits` takes them.
    fn hints_option(self) -> &'static str {
        match self {
            Self::Inherits => ", `when(...)` hints",
            Self::NotInherits => "",
        }
    }

    /// The error for an unknown option `key` in a parameter-level
    /// `inherits(...)` / `not_inherits(...)`.
    fn unknown_option(self, key: &syn::Ident) -> String {
        format!(
            "unknown `{}` option `{key}`; expected class names \
             (`\"cls\"` or `class = \"cls\"`){} and an optional `message = \"...\"`",
            self.keyword(),
            self.hints_option()
        )
    }

    /// The error for an unknown option in a method-level
    /// `inherits(p(...))` / `not_inherits(p(...))`.
    fn unknown_method_option(self) -> String {
        format!(
            "unknown `{}` option; expected `class = \"...\"`{} and an optional \
             `message = \"...\"`",
            self.keyword(),
            self.hints_option()
        )
    }

    /// This check's classes in `checks`, if it is requested.
    fn classes(self, checks: &crate::r_preconditions::ExplicitChecks) -> Option<&[String]> {
        match self {
            Self::Inherits => checks.inherits.as_deref(),
            Self::NotInherits => checks.not_inherits.as_deref(),
        }
    }

    /// This check's classes in `checks`, to add to.
    fn classes_mut(
        self,
        checks: &mut crate::r_preconditions::ExplicitChecks,
    ) -> &mut Option<Vec<String>> {
        match self {
            Self::Inherits => &mut checks.inherits,
            Self::NotInherits => &mut checks.not_inherits,
        }
    }

    /// The checks holding this check with `classes` and `message`.
    fn checks(
        self,
        classes: Vec<String>,
        message: Option<String>,
    ) -> crate::r_preconditions::ExplicitChecks {
        let mut checks = crate::r_preconditions::ExplicitChecks::default();
        *self.classes_mut(&mut checks) = Some(classes);
        match self {
            Self::Inherits => checks.inherits_message = message,
            Self::NotInherits => checks.not_inherits_message = message,
        }
        checks
    }
}

/// `inherits(...)` / `not_inherits(...)` on a parameter (`check` says
/// which): each string literal, and each `class = "..."`, names one class
/// (the argument must inherit from one of them, or from none of them);
/// `message = "..."` gives the condition message of a failure, for all of
/// them. No string is split on commas, as with `inherits = "cls"` (the
/// method-level `class = "a, b"` is split, see [`parse_method_class_check`]).
///
/// `inherits(...)` also takes `when(...)` hints (#1824), each with its
/// classes spelled the same way and a required `message = "..."`
/// ([`parse_when_list`]).
fn parse_class_check_list(
    list: &syn::MetaList,
    check: ClassCheck,
) -> syn::Result<crate::r_preconditions::ExplicitChecks> {
    use syn::parse::Parser as _;
    let key_word = check.keyword();
    let mut classes = Vec::new();
    let mut message = None;
    let mut hints = Vec::new();
    let parser = |input: syn::parse::ParseStream| -> syn::Result<()> {
        while !input.is_empty() {
            if input.peek(syn::LitStr) {
                let class: syn::LitStr = input.parse()?;
                classes.push(class.value());
            } else {
                let key: syn::Ident = input.parse()?;
                if key == "when" && input.peek(syn::token::Paren) {
                    if check != ClassCheck::Inherits {
                        return Err(syn::Error::new_spanned(&key, WHEN_ON_NOT_INHERITS));
                    }
                    let content;
                    syn::parenthesized!(content in input);
                    hints.push(parse_when_list(&key, &content)?);
                } else {
                    input.parse::<syn::Token![=]>()?;
                    let value: syn::LitStr = input.parse()?;
                    if key == "class" {
                        classes.push(value.value());
                    } else if key == "message" {
                        set_check_message(&mut message, &value, key_word)?;
                    } else {
                        return Err(syn::Error::new_spanned(&key, check.unknown_option(&key)));
                    }
                }
            }
            if input.is_empty() {
                break;
            }
            input.parse::<syn::Token![,]>()?;
        }
        Ok(())
    };
    parser.parse2(list.tokens.clone())?;
    if message.is_some() && classes.is_empty() {
        return Err(syn::Error::new_spanned(
            list,
            format!(
                "`message` in `{key_word}(...)` needs a class to check, \
                 e.g. `{key_word}(class = \"pkg_obj\", message = \"...\")`"
            ),
        ));
    }
    if !hints.is_empty() && classes.is_empty() {
        return Err(syn::Error::new_spanned(
            list,
            "`when(...)` in `inherits(...)` needs a class to check, e.g. \
             `inherits(class = \"pkg_obj\", when(class = \"data.frame\", message = \"...\"))`",
        ));
    }
    let mut checks = check.checks(classes, message);
    checks.inherits_hints = hints;
    Ok(checks)
}

/// The error for `when(...)` in `not_inherits(...)`.
const WHEN_ON_NOT_INHERITS: &str = "`when(...)` is an `inherits` option: it gives a value that \
     fails `inherits` a message for its class; `not_inherits` has one `message` for every \
     class it refuses";

/// The content of a parameter-level `when(...)` hint in `inherits(...)`
/// (`key` is the `when` keyword, for error spans): its classes, spelled as in
/// `inherits(...)` (each string literal and each `class = "..."` names one
/// class, none split on commas), and its required `message = "..."`.
fn parse_when_list(
    key: &syn::Ident,
    content: syn::parse::ParseStream,
) -> syn::Result<crate::r_preconditions::InheritsHint> {
    let mut classes = Vec::new();
    let mut message = None;
    while !content.is_empty() {
        if content.peek(syn::LitStr) {
            let class: syn::LitStr = content.parse()?;
            classes.push(hint_class(&class)?);
        } else {
            let opt: syn::Ident = content.parse()?;
            content.parse::<syn::Token![=]>()?;
            let value: syn::LitStr = content.parse()?;
            if opt == "class" {
                classes.push(hint_class(&value)?);
            } else if opt == "message" {
                set_hint_message(&mut message, &value)?;
            } else {
                return Err(syn::Error::new_spanned(
                    &opt,
                    format!(
                        "unknown `when` option `{opt}`; expected class names (`\"cls\"` or \
                         `class = \"cls\"`) and `message = \"...\"`"
                    ),
                ));
            }
        }
        if content.is_empty() {
            break;
        }
        content.parse::<syn::Token![,]>()?;
    }
    inherits_hint(classes, message).map_err(|msg| syn::Error::new_spanned(key, msg))
}

/// One class name of a parameter-level `when(...)` hint: not empty.
fn hint_class(value: &syn::LitStr) -> syn::Result<String> {
    let class = value.value();
    if class.is_empty() {
        return Err(syn::Error::new(
            value.span(),
            "a class name in `when(...)` must not be empty",
        ));
    }
    Ok(class)
}

/// A parsed `when(...)` hint, which needs both its classes and its message.
fn inherits_hint(
    classes: Vec<String>,
    message: Option<String>,
) -> Result<crate::r_preconditions::InheritsHint, &'static str> {
    if classes.is_empty() {
        return Err("`when(...)` needs the classes it is for, e.g. \
             `when(class = \"data.frame\", message = \"...\")`");
    }
    let Some(message) = message else {
        return Err(
            "`when(...)` needs a `message = \"...\"`: the hint a value of its classes gets, \
             e.g. `when(class = \"data.frame\", message = \"...\")`",
        );
    };
    Ok(crate::r_preconditions::InheritsHint { classes, message })
}

/// `no_na(message = "...")` on a parameter: `no_na` with the condition
/// message of a failure.
fn parse_no_na_list(list: &syn::MetaList) -> syn::Result<crate::r_preconditions::ExplicitChecks> {
    let mut message = None;
    list.parse_nested_meta(|opt| parse_no_na_option(&opt, &mut message))?;
    Ok(crate::r_preconditions::ExplicitChecks {
        no_na: true,
        no_na_message: message,
        ..Default::default()
    })
}

/// One option inside `no_na(...)` (parameter level) or `no_na(p(...))`
/// (method level): only `message = "..."`.
fn parse_no_na_option(
    opt: &syn::meta::ParseNestedMeta,
    message: &mut Option<String>,
) -> syn::Result<()> {
    if opt.path.is_ident("message") {
        let value: syn::LitStr = opt.value()?.parse()?;
        set_check_message(message, &value, "no_na")
    } else {
        Err(opt.error("unknown `no_na` option; expected `message = \"...\"`"))
    }
}

/// Record the `message = "..."` of an `inherits` / `not_inherits` / `no_na`
/// check: the condition message of a failure, used verbatim. It is given
/// once, is not empty, and holds no NUL (an R string cannot).
fn set_check_message(
    slot: &mut Option<String>,
    value: &syn::LitStr,
    check: &str,
) -> syn::Result<()> {
    set_message(
        slot,
        value,
        check,
        "leave it out to get the generated message",
    )
}

/// Record the `message = "..."` of a `when(...)` hint in `inherits(...)`,
/// under the rules of [`set_check_message`]; unlike a check's message, the
/// hint's is required.
fn set_hint_message(slot: &mut Option<String>, value: &syn::LitStr) -> syn::Result<()> {
    set_message(
        slot,
        value,
        "when",
        "it is the message a value of those classes gets",
    )
}

/// Record `value` as the message in `slot` of `check(...)`; `if_empty`
/// ends the error for an empty message.
fn set_message(
    slot: &mut Option<String>,
    value: &syn::LitStr,
    check: &str,
    if_empty: &str,
) -> syn::Result<()> {
    let text = value.value();
    if slot.is_some() {
        return Err(syn::Error::new(
            value.span(),
            format!("`message` is given more than once in `{check}(...)`"),
        ));
    }
    if text.trim().is_empty() {
        return Err(syn::Error::new(
            value.span(),
            format!("`message` in `{check}(...)` must not be empty; {if_empty}"),
        ));
    }
    if text.contains('\0') {
        return Err(syn::Error::new(
            value.span(),
            "`message` must not contain a NUL character: an R string cannot hold one",
        ));
    }
    *slot = Some(text);
    Ok(())
}

/// The parameter an entry of a method-level `match_arg(...)` / `choices(...)`
/// / `no_na(...)` / `inherits(...)` / `not_inherits(...)` names, spelled as
/// the signature's [`crate::naming::ident_name`] (`r#type` names `type`), so
/// [`finalize_method_param_attrs`] finds it.
pub(crate) fn method_check_param(entry: &syn::meta::ParseNestedMeta) -> syn::Result<String> {
    Ok(crate::naming::ident_name(
        entry
            .path
            .get_ident()
            .ok_or_else(|| entry.error("expected parameter name"))?,
    ))
}

/// Method-level `no_na(p, q(message = "..."))` on an impl or trait method,
/// whose parameters cannot carry attributes: each entry names a parameter,
/// optionally with the condition message of its NA check. Shared by the
/// inherent-impl and trait-impl method parsers.
pub(crate) fn parse_method_no_na(
    meta: &syn::meta::ParseNestedMeta,
    per_param: &mut std::collections::HashMap<String, ParamAttrs>,
) -> syn::Result<()> {
    meta.parse_nested_meta(|entry| {
        let name = method_check_param(&entry)?;
        let mut message = None;
        if entry.input.peek(syn::token::Paren) {
            entry.parse_nested_meta(|opt| parse_no_na_option(&opt, &mut message))?;
        }
        let checks = crate::r_preconditions::ExplicitChecks {
            no_na: true,
            no_na_message: message,
            ..Default::default()
        };
        per_param
            .entry(name)
            .or_default()
            .checks
            .merge(checks)
            .map_err(|msg| entry.error(msg))
    })
}

/// Method-level `inherits(p = "a, b", q(class = "a, b", message = "..."))` /
/// `not_inherits(...)` (the same shape, `check` says which) on an impl or
/// trait method: each entry names a parameter and the classes it must
/// inherit from (one of) or must not inherit from (any of), optionally with
/// the condition message of the class check. A nested value cannot be a bare
/// list of literals, so the classes are one comma-separated string, as in
/// `choices(p = "a, b")`. `inherits(q(class = "a", when(class = "b, c",
/// message = "...")))` adds a hint (#1824; [`parse_method_when`]). Shared by
/// the inherent-impl and trait-impl method parsers.
pub(crate) fn parse_method_class_check(
    meta: &syn::meta::ParseNestedMeta,
    per_param: &mut std::collections::HashMap<String, ParamAttrs>,
    check: ClassCheck,
) -> syn::Result<()> {
    let key_word = check.keyword();
    meta.parse_nested_meta(|entry| {
        let name = method_check_param(&entry)?;
        let mut classes = Vec::new();
        let mut message = None;
        let mut hints = Vec::new();
        if entry.input.peek(syn::Token![=]) {
            let value: syn::LitStr = entry.value()?.parse()?;
            classes = method_class_list(&value, check)?;
        } else if entry.input.peek(syn::token::Paren) {
            entry.parse_nested_meta(|opt| {
                if opt.path.is_ident("class") {
                    let value: syn::LitStr = opt.value()?.parse()?;
                    classes.extend(method_class_list(&value, check)?);
                    Ok(())
                } else if opt.path.is_ident("message") {
                    let value: syn::LitStr = opt.value()?.parse()?;
                    set_check_message(&mut message, &value, key_word)
                } else if opt.path.is_ident("when") {
                    if check != ClassCheck::Inherits {
                        return Err(opt.error(WHEN_ON_NOT_INHERITS));
                    }
                    hints.push(parse_method_when(&opt)?);
                    Ok(())
                } else {
                    Err(opt.error(check.unknown_method_option()))
                }
            })?;
            if classes.is_empty() {
                return Err(entry.error(format!(
                    "`{key_word}({name}(...))` needs `class = \"...\"`, \
                     e.g. `{key_word}({name}(class = \"pkg_obj\", message = \"...\"))`"
                )));
            }
        } else {
            return Err(entry.error(format!(
                "expected `{name} = \"cls\"` or `{name}(class = \"cls\", message = \"...\")`"
            )));
        }
        let mut checks = check.checks(classes, message);
        checks.inherits_hints = hints;
        per_param
            .entry(name)
            .or_default()
            .checks
            .merge(checks)
            .map_err(|msg| entry.error(msg))
    })
}

/// A method-level `when(class = "a, b", message = "...")` hint in
/// `inherits(p(...))`: its classes are one comma-separated string, as the
/// entry's own `class`, and its message is required.
fn parse_method_when(
    opt: &syn::meta::ParseNestedMeta,
) -> syn::Result<crate::r_preconditions::InheritsHint> {
    let mut classes = Vec::new();
    let mut message = None;
    opt.parse_nested_meta(|hint| {
        if hint.path.is_ident("class") {
            let value: syn::LitStr = hint.value()?.parse()?;
            let names = crate::r_wrapper_builder::split_choice_list(&value.value());
            if names.is_empty() {
                return Err(syn::Error::new(
                    value.span(),
                    "`when(class = \"...\")` needs one or more class names",
                ));
            }
            classes.extend(names);
            Ok(())
        } else if hint.path.is_ident("message") {
            let value: syn::LitStr = hint.value()?.parse()?;
            set_hint_message(&mut message, &value)
        } else {
            Err(hint
                .error("unknown `when` option; expected `class = \"...\"` and `message = \"...\"`"))
        }
    })?;
    inherits_hint(classes, message).map_err(|msg| opt.error(msg))
}

/// Method-level `no_default(p, q)` on an impl or trait method, whose
/// parameters cannot carry attributes: each entry names a `match_arg` /
/// `choices` parameter whose R formal has no default (#1828). Shared by the
/// inherent-impl and trait-impl method parsers; [`finalize_method_param_attrs`]
/// refuses a name that is no parameter, no choice or has a default.
pub(crate) fn parse_method_no_default(
    meta: &syn::meta::ParseNestedMeta,
    per_param: &mut std::collections::HashMap<String, ParamAttrs>,
) -> syn::Result<()> {
    meta.parse_nested_meta(|entry| {
        let name = method_check_param(&entry)?;
        per_param.entry(name).or_default().no_default = true;
        Ok(())
    })
}

/// The classes of a method-level `inherits` / `not_inherits` entry: a
/// comma-separated, non-empty list.
fn method_class_list(value: &syn::LitStr, check: ClassCheck) -> syn::Result<Vec<String>> {
    let classes = crate::r_wrapper_builder::split_choice_list(&value.value());
    if classes.is_empty() {
        return Err(syn::Error::new(
            value.span(),
            format!(
                "`{}(param = \"...\")` needs one or more class names",
                check.keyword()
            ),
        ));
    }
    Ok(classes)
}
// endregion

// region: per-parameter preconditions (#1566 §2)

/// Method-level `preconditions(p, q)` / `no_preconditions(p)` on an impl or
/// trait method, whose parameters cannot carry attributes: each entry names a
/// parameter that keeps (`keep`) or drops its type-derived R-side checks.
/// Naming one parameter twice follows the pair rule: the last one written
/// wins. Shared by the inherent-impl and trait-impl method parsers; the bare
/// `preconditions` / `no_preconditions` (the whole method) is theirs to read.
pub(crate) fn parse_method_preconditions(
    meta: &syn::meta::ParseNestedMeta,
    per_param: &mut std::collections::HashMap<String, ParamAttrs>,
    keep: bool,
) -> syn::Result<()> {
    meta.parse_nested_meta(|entry| {
        let name = method_check_param(&entry)?;
        per_param.entry(name).or_default().preconditions = Some(keep);
        Ok(())
    })
}

/// The keyword spelling of a rank-1 precondition decision, for messages:
/// the per-parameter attribute on a function parameter, the method-level
/// list on a method.
fn preconditions_keyword(keep: bool, param: &str, method_list: bool) -> String {
    let key = if keep {
        "preconditions"
    } else {
        "no_preconditions"
    };
    if method_list {
        format!("`{key}({param})`")
    } else {
        format!("`#[miniextendr({key})]`")
    }
}

/// Combine a parameter's `Checked<T>` / `Unchecked<T>` marker with its
/// keyword decision (`keyword`: the per-parameter `preconditions` /
/// `no_preconditions`, or the method's `preconditions(p)` /
/// `no_preconditions(p)`). A marker is a type, not one more flag of the pair,
/// so a marker and a keyword that disagree are an error, as a `Call` marker
/// and `call = ...` are; agreeing ones are one decision. The error points at
/// `ty`, the parameter's type as written.
pub(crate) fn merge_param_policy(
    markers: &[crate::type_inspect::ParamMarker],
    keyword: Option<bool>,
    param: &str,
    method_list: bool,
    ty: &syn::Type,
) -> syn::Result<Option<bool>> {
    let marker = crate::type_inspect::marker_in_family(
        markers,
        crate::type_inspect::ParamMarkerFamily::Preconditions,
    );
    let Some(marker) = marker else {
        return Ok(keyword);
    };
    let from_marker = marker.preconditions();
    if let Some(keep) = keyword
        && Some(keep) != from_marker
    {
        let (marker_does, keyword_does) = if keep {
            ("drops", "keeps")
        } else {
            ("keeps", "drops")
        };
        return Err(syn::Error::new_spanned(
            ty,
            format!(
                "the `{}` parameter `{param}` {marker_does} the R-side type checks but {} \
                 {keyword_does} them; keep one of them (or make them agree)",
                marker.name(),
                preconditions_keyword(keep, param, method_list),
            ),
        ));
    }
    Ok(from_marker)
}

/// Reject a parameter's rank-1 precondition decision where it has nothing to
/// act on: a `match_arg` / `choices` parameter (validated by `match.arg()`,
/// never by type checks) and a type without type-derived checks (`SEXP`,
/// `Missing<T>`, `ExternalPtr<T>`, `&Dots`, a type the check table does not
/// know). `ty` is the parameter's type with the markers peeled, which the
/// errors point at; `markers` are the ones it carried. A no-op when the
/// parameter decides nothing itself.
pub(crate) fn check_param_preconditions(
    attrs: &ParamAttrs,
    markers: &[crate::type_inspect::ParamMarker],
    param: &str,
    ty: &syn::Type,
    method_list: bool,
) -> syn::Result<()> {
    let Some(keep) = attrs.preconditions else {
        return Ok(());
    };
    let marker = crate::type_inspect::marker_in_family(
        markers,
        crate::type_inspect::ParamMarkerFamily::Preconditions,
    );
    let spelling = match marker {
        Some(m) => format!("`{}<{}>`", m.name(), crate::type_inspect::type_display(ty)),
        None => preconditions_keyword(keep, param, method_list),
    };
    if attrs.is_choice() {
        return Err(syn::Error::new_spanned(
            ty,
            format!(
                "{spelling} on parameter `{param}`: a match_arg/choices parameter is validated by \
                 `match.arg()`, not by type checks; `Checked` / `Unchecked` and \
                 `preconditions` / `no_preconditions` do not apply to it"
            ),
        ));
    }
    if !crate::r_preconditions::has_type_check(ty) {
        return Err(syn::Error::new_spanned(
            ty,
            format!(
                "{spelling} on parameter `{param}`: `{}` has no R-side type check to keep or drop",
                crate::type_inspect::type_display(ty)
            ),
        ));
    }
    Ok(())
}

/// `inputs` with each marked parameter's type replaced by the type under its
/// markers (`markers`, keyed by Rust name): what every R-side consumer and
/// the conversion see. The item re-emitted for the user keeps the markers.
pub(crate) fn peeled_inputs(
    inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
    markers: &[(String, Vec<crate::type_inspect::ParamMarker>)],
) -> syn::Result<syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>> {
    let mut peeled = inputs.clone();
    for arg in peeled.iter_mut() {
        let syn::FnArg::Typed(pt) = arg else {
            continue;
        };
        let syn::Pat::Ident(pat_ident) = pt.pat.as_ref() else {
            continue;
        };
        let name = crate::naming::ident_name(&pat_ident.ident);
        if markers.iter().any(|(n, _)| *n == name) {
            let (_, inner) = crate::type_inspect::peel_param_markers(pt.ty.as_ref())?;
            *pt.ty = inner.clone();
        }
    }
    Ok(peeled)
}

/// The markers of every parameter of `inputs` that carries one, keyed by
/// Rust name, in signature order. Nested or stacked markers are an error
/// (see [`crate::type_inspect::peel_param_markers`]).
pub(crate) fn collect_param_markers(
    inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
) -> syn::Result<Vec<(String, Vec<crate::type_inspect::ParamMarker>)>> {
    let mut out = Vec::new();
    for arg in inputs {
        let syn::FnArg::Typed(pt) = arg else {
            continue;
        };
        let (markers, _) = crate::type_inspect::peel_param_markers(pt.ty.as_ref())?;
        if markers.is_empty() {
            continue;
        }
        let syn::Pat::Ident(pat_ident) = pt.pat.as_ref() else {
            continue;
        };
        out.push((crate::naming::ident_name(&pat_ident.ident), markers));
    }
    Ok(out)
}
// endregion

// region: Function parsing

/// Parsed + normalized Rust function item for `#[miniextendr]`.
///
/// This performs signature normalization that the wrapper generator depends on:
/// - `...` → a final `&miniextendr_api::dots::Dots` argument
/// - `_` wildcard patterns → synthetic identifiers (`__unused0`, `__unused1`, ...)
/// - Destructuring patterns (tuple, struct) → synthetic identifiers with let-binding in body
/// - consumes `#[miniextendr(coerce)]` parameter attributes and records which params had it
pub(crate) struct MiniextendrFunctionParsed {
    /// The normalized function item (with dots transformed, wildcards renamed).
    /// Re-emitted for the user, so it keeps the parameter markers.
    item: syn::ItemFn,
    /// `item`'s parameters with the parameter markers (`Checked<T>` /
    /// `Unchecked<T>`, #1566) peeled: what the R wrapper, the preconditions
    /// and the C wrapper's conversions see.
    inputs: syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
    /// The markers peeled off each marked parameter (Rust name), in
    /// signature order: the C wrapper wraps the converted value in them.
    param_markers: Vec<(String, Vec<crate::type_inspect::ParamMarker>)>,
    /// Rust binding of the `&Dots` parameter (R's `...`), from Rust `...` or
    /// an explicit `&Dots` parameter at any position (see
    /// [`find_dots_param`]).
    dots: Option<syn::Ident>,
    /// All per-parameter `#[miniextendr(...)]` options (coerce, match_arg,
    /// default, choices, several_ok), keyed by the (possibly synthesized) Rust
    /// parameter name. Replaces five parallel `HashSet` / `HashMap` fields.
    per_param: std::collections::HashMap<String, ParamAttrs>,
}

/// Collapsed per-parameter attribute state for a single function parameter.
///
/// Built during parsing from `#[miniextendr(coerce | match_arg | several_ok |
/// no_default | default = "…" | choices("…"))]` on the argument. Accessors on
/// [`MiniextendrFunctionParsed`] query this struct rather than looking
/// through multiple side-tables.
#[derive(Default, Debug, Clone)]
pub(crate) struct ParamAttrs {
    pub coerce: bool,
    pub match_arg: bool,
    pub several_ok: bool,
    pub choices: Option<Vec<String>>,
    pub default: Option<String>,
    /// `Option<..>` layer of a `match_arg` / `choices` parameter (#1473):
    /// `NULL` means no choice, so the prelude skips the check for it and Rust
    /// sees `None`. Unless [`Self::omittable`] is set too, the R formal is
    /// `NULL` instead of the choice vector. Set from the parameter type by
    /// [`classify_choice_param`]; together with `several_ok` only under an
    /// `Either` layer (`Option<Either<Vec<T>, R>>`), since a plain
    /// `several_ok` list already reads `NULL` as every choice.
    pub optional: bool,
    /// `Missing<..>` layer of a `match_arg` / `choices` parameter (#1551): the
    /// R formal keeps the choice vector, the prelude skips the check for an
    /// omitted argument, and Rust sees `Missing::Absent`. Set by
    /// [`classify_choice_param`].
    pub omittable: bool,
    /// `no_default` on a `match_arg` / `choices` parameter (#1828): the R
    /// formal is the bare name instead of the choice vector (or `NULL`), so
    /// an omitted argument is `Missing::Absent` under `Missing<..>` and R's
    /// missing-argument error otherwise. The prelude and the choice list do
    /// not change. Set from the parameter's `no_default` or the method's
    /// `no_default(p)`.
    pub no_default: bool,
    /// `Either<.., R>` layer of a `match_arg` / `choices` parameter, over a
    /// scalar choice (`Either<T, R>`) or a `several_ok` list
    /// (`Either<Vec<T>, R>` / `Either<Box<[T]>, R>`): the R-facing name of the
    /// `R` arm (`"a data frame"`, see [`crate::type_inspect::r_value_noun`])
    /// for the `@param` line. The prelude checks only character or factor
    /// input; anything else, `NULL` included, reaches Rust unchanged and
    /// decodes as `R`. Set by [`classify_choice_param`].
    pub either_noun: Option<String>,
    /// R-side checks named by the author: `inherits` / `not_inherits` /
    /// `no_na`.
    pub checks: crate::r_preconditions::ExplicitChecks,
    /// This parameter's own decision on its type-derived R-side checks
    /// (#1566), rank 1 of [`crate::r_preconditions::resolve_type_checks`]:
    /// a `Checked<T>` / `Unchecked<T>` marker, the per-parameter
    /// `preconditions` / `no_preconditions`, or a method's
    /// `preconditions(x)` / `no_preconditions(x)`. `Some(true)` keeps them;
    /// `None` defers to the function, impl and crate defaults.
    pub preconditions: Option<bool>,
}

impl ParamAttrs {
    /// Whether this is a `match_arg` / `choices` parameter.
    pub(crate) fn is_choice(&self) -> bool {
        self.match_arg || self.choices.is_some()
    }

    /// The R formal default of a choice parameter, given its choice list
    /// (`c("a", "b")` or the write-time placeholder): `NULL` for the
    /// `Option<T>` form (#1473), the choice list otherwise, including
    /// `Missing<Option<T>>`, whose omission is reported by `Missing` (#1551).
    /// `None` under `no_default` (#1828): the formal is the bare name. The
    /// prelude spells the choice list out either way, so a `match_arg`
    /// placeholder then lives in the prelude only.
    pub(crate) fn choice_formal(&self, choices: &str) -> Option<String> {
        if self.no_default {
            None
        } else if self.optional && !self.omittable {
            Some("NULL".to_string())
        } else {
            Some(choices.to_string())
        }
    }

    /// The text after the quoted choice list in the auto-generated `@param`
    /// line (`One of "a", "b"<suffix>.`): the other accepted values, then an
    /// omission note. `, or NULL for no choice` for `Option<T>`,
    /// `, or a data frame` for `Either<T, DataFrame>`,
    /// `, a data frame, or NULL for no choice` for both,
    /// `, or NULL; omitting the argument means no choice` for
    /// `Missing<Option<T>>`, `; omitting the argument means no choice` for
    /// `Missing<T>`, and nothing for a plain choice. Under `no_default`
    /// (#1828) the omission note goes (the author documents what omission
    /// means) and the rest stays.
    pub(crate) fn choice_doc_suffix(&self) -> String {
        let null_word = if self.omittable {
            "NULL"
        } else {
            "NULL for no choice"
        };
        let mut suffix =
            choice_alternatives_suffix(self.either_noun.as_deref(), self.optional, null_word);
        if self.omittable && !self.no_default {
            suffix.push_str("; omitting the argument means no choice");
        }
        suffix
    }

    /// The auto-generated `@param` text of a `choices(...)` parameter:
    /// `One of "a", "b"<suffix>.` (`One or more of` for `several_ok`), with
    /// [`Self::choice_doc_suffix`]. `None` without a literal choice list; a
    /// `match_arg` parameter's text is only known at write time (its
    /// placeholder, #210).
    pub(crate) fn literal_choices_doc(&self) -> Option<String> {
        let choices = self.choices.as_ref()?;
        let quoted: Vec<String> = choices.iter().map(|c| format!("\"{c}\"")).collect();
        let prefix = if self.several_ok {
            "One or more of"
        } else {
            "One of"
        };
        Some(format!(
            "{prefix} {}{}.",
            quoted.join(", "),
            self.choice_doc_suffix()
        ))
    }

    /// How the C wrapper decodes this parameter when a plain `TryFromSexp`
    /// cannot: a `match_arg` parameter with a `Missing` / `Option` / `Either`
    /// layer (`MatchArgSeveral` for a `several_ok` list, `Either<Vec<T>, R>`
    /// included), or a `choices` parameter with an `Either` layer (its string
    /// type or `several_ok` string list on the left). `None` for every other
    /// parameter (a `choices` string type converts through `TryFromSexp` with
    /// its `Missing` / `Option` layers; a plain `several_ok` container has its
    /// own path).
    pub(crate) fn layered_leaf(&self) -> Option<crate::rust_conversion_builder::ChoiceLeaf> {
        use crate::rust_conversion_builder::ChoiceLeaf;
        let either = self.either_noun.is_some();
        if self.match_arg && (self.optional || self.omittable || either) {
            return Some(if self.several_ok {
                ChoiceLeaf::MatchArgSeveral
            } else {
                ChoiceLeaf::MatchArg
            });
        }
        let choices = self.choices.as_ref().filter(|_| either)?;
        Some(ChoiceLeaf::Literal {
            choices: choices.clone(),
            several: self.several_ok,
        })
    }
}

/// The other values a choice parameter accepts, after its quoted choices:
/// `, or a data frame` for an `Either<.., R>` layer (`noun`, the `R` arm),
/// `, or <null_word>` for an `Option<..>` layer (`optional`), `, a data frame,
/// or <null_word>` for both, nothing for neither. Shared by the parameter's
/// `@param` line ([`ParamAttrs::choice_doc_suffix`], which passes
/// `NULL for no choice`) and the argument error of an `Either` choice
/// parameter (which passes `NULL`), so the two word the alternatives alike.
pub(crate) fn choice_alternatives_suffix(
    noun: Option<&str>,
    optional: bool,
    null_word: &str,
) -> String {
    match (noun, optional) {
        (None, false) => String::new(),
        (Some(only), false) => format!(", or {only}"),
        (None, true) => format!(", or {null_word}"),
        (Some(noun), true) => format!(", {noun}, or {null_word}"),
    }
}

/// Record the layers of a `match_arg` / `choices` parameter's type on its
/// [`ParamAttrs`] (`optional`, `omittable`, `either_noun`) and reject the
/// shapes the codegen cannot serve, now that the type is known. Shared by the
/// standalone-fn parser and [`finalize_method_param_attrs`] so the two cannot
/// classify a type differently. A no-op for other parameters.
///
/// Accepted, outermost layer first: an optional `Missing<..>` (#1551), an
/// optional `Option<..>` (#1473), an optional `Either<.., R>`, then the
/// scalar choice type (so `T`, `Option<T>`, `Missing<Option<T>>`,
/// `Either<T, R>`, `Option<Either<T, R>>`, ...). A `several_ok` parameter
/// takes any container (`Vec<T>`, `Box<[T]>`, `&[T]`, `[T; N]`) bare, and
/// only the owned `Vec<T>` / `Box<[T]>` under a layer: `Missing<Vec<T>>`,
/// `Either<Vec<T>, R>` (#1612) and the `Missing` / `Option` stacks over that
/// `Either`, which follow the scalar `Either` rule. A bare
/// `Option<Vec<T>>` stays rejected: its `NULL` already means every choice,
/// while under `Either` it goes to `R`. `has_default` says whether the
/// parameter carries a `default` (a choice with a `NULL` formal, `Option<..>`
/// without `Missing`, cannot; the `Missing<T>` + default conflict is reported
/// by the callers' own check).
pub(crate) fn classify_choice_param(
    attrs: &mut ParamAttrs,
    param_name: &str,
    ty: &syn::Type,
    has_default: bool,
) -> syn::Result<()> {
    use syn::spanned::Spanned;
    if !attrs.is_choice() {
        return Ok(());
    }
    let layers = crate::type_inspect::choice_layers(ty);
    if let Some(layer) = crate::type_inspect::choice_layer_name(layers.value) {
        return Err(syn::Error::new(
            ty.span(),
            format!(
                "match_arg/choices parameter `{param_name}` has `{layer}<..>` in an unsupported \
                 position; the wrappers go outermost first as `Missing<Option<Either<T, R>>>` \
                 (each one optional, the choice type `T` on the left of `Either`)"
            ),
        ));
    }
    if attrs.several_ok {
        let either = layers.either_right.is_some();
        // Under `Either`, `NULL` goes to the `R` arm, so `Option<Either<Vec<T>, R>>`
        // has a `None` of its own; a bare `Option<Vec<T>>` does not.
        if layers.nullable && !either {
            return Err(syn::Error::new(
                ty.span(),
                format!(
                    "several_ok parameter `{param_name}` cannot be `Option<..>`; an omitted \
                     argument and NULL both select every choice. Use `Missing<Vec<T>>` to \
                     tell an omitted argument apart"
                ),
            ));
        }
        if !is_vector_like_type(layers.value) {
            // Under `Missing` / `Either` only the owned containers decode.
            let containers = if layers.missing || either {
                "`Vec<T>` or `Box<[T]>`"
            } else {
                "`Vec<T>`, `Box<[T]>`, `&[T]`, or `[T; N]`"
            };
            return Err(syn::Error::new(
                ty.span(),
                format!(
                    "several_ok requires a vector type on parameter `{param_name}`; \
                     several_ok enables multi-value match.arg which returns a character vector. \
                     Use {containers} instead of a scalar type"
                ),
            ));
        }
        let owned = matches!(
            crate::classify_several_ok_container(layers.value),
            Some((
                crate::SeveralOkContainer::Vec | crate::SeveralOkContainer::BoxedSlice,
                _
            ))
        );
        if either && !owned {
            return Err(syn::Error::new(
                ty.span(),
                format!(
                    "several_ok parameter `{param_name}` can take another kind of value only \
                     as `Either<Vec<T>, R>` or `Either<Box<[T]>, R>`"
                ),
            ));
        }
        if layers.missing && !owned {
            return Err(syn::Error::new(
                ty.span(),
                format!(
                    "several_ok parameter `{param_name}` can be omittable only as \
                     `Missing<Vec<T>>` or `Missing<Box<[T]>>`"
                ),
            ));
        }
    }
    // A `NULL` formal (`Option<..>` without `Missing`) makes a `default`
    // meaningless, for a scalar choice and for `Option<Either<Vec<T>, R>>`.
    if layers.nullable && !layers.missing && has_default {
        return Err(syn::Error::new(
            ty.span(),
            optional_choice_default_msg(param_name),
        ));
    }
    attrs.optional = layers.nullable;
    attrs.omittable = layers.missing;
    attrs.either_noun = layers.either_right.map(crate::type_inspect::r_value_noun);
    Ok(())
}

/// The entries of `per_param` in the order their parameters appear in
/// `inputs`. Code that emits one line or item per parameter walks this, never
/// the map itself: a `HashMap`'s order differs between maps and between
/// compiler runs, so the generated R wrapper (and the Rust items) would too.
pub(crate) fn per_param_in_signature_order<'a>(
    inputs: &'a syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
    per_param: &'a std::collections::HashMap<String, ParamAttrs>,
) -> impl Iterator<Item = (&'a String, &'a ParamAttrs)> {
    inputs.iter().filter_map(|arg| {
        let syn::FnArg::Typed(pt) = arg else {
            return None;
        };
        let syn::Pat::Ident(pat_ident) = pt.pat.as_ref() else {
            return None;
        };
        per_param.get_key_value(&crate::naming::ident_name(&pat_ident.ident))
    })
}

/// The non-empty [`ParamAttrs::checks`] of `per_param` (keyed by Rust name),
/// re-keyed by R-normalized parameter name for
/// [`crate::r_preconditions::PreconditionOptions::explicit`].
pub(crate) fn explicit_checks_by_r_name(
    per_param: &std::collections::HashMap<String, ParamAttrs>,
) -> std::collections::HashMap<String, crate::r_preconditions::ExplicitChecks> {
    per_param
        .iter()
        .filter(|(_, a)| !a.checks.is_empty())
        .map(|(name, a)| {
            (
                crate::r_wrapper_builder::normalize_r_arg_string(name),
                a.checks.clone(),
            )
        })
        .collect()
}

/// Check an impl or trait method's per-parameter attributes against its
/// signature, now that it is known. Every parameter a method-level
/// `match_arg(...)` / `choices(...)` / `no_default(...)` / `inherits(...)` /
/// `not_inherits(...)` / `no_na(...)` / `preconditions(...)` /
/// `no_preconditions(...)` names must exist (a typo would otherwise drop the
/// check without a word); then a `no_default(p)` is refused on a parameter
/// that is no choice or has a default ([`no_default_conflict`]), the
/// choice parameters are classified (see [`classify_choice_param`]), a class
/// in both `inherits` and `not_inherits` is refused
/// ([`crate::r_preconditions::ExplicitChecks::class_conflict`]), and each
/// parameter's own precondition decision is checked
/// ([`check_param_preconditions`]). The standalone-fn path does the same
/// while parsing; this is the twin for method-level attributes, whose
/// parameter names arrive before the types. The inherent-impl and trait-impl
/// parsers both call it, so they reject an unknown name the same way.
/// `inputs` has the parameter markers peeled; `markers` are the ones peeled
/// (empty on a trait impl, whose signature is the trait's). `span` is where
/// the errors point: the first such attribute, else the method name.
pub(crate) fn finalize_method_param_attrs(
    per_param: &mut std::collections::HashMap<String, ParamAttrs>,
    inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
    markers: &[(String, Vec<crate::type_inspect::ParamMarker>)],
    defaults: &std::collections::HashMap<String, String>,
    span: proc_macro2::Span,
) -> syn::Result<()> {
    let sig_names: std::collections::HashSet<String> = inputs
        .iter()
        .filter_map(|arg| match arg {
            syn::FnArg::Typed(pt) => match pt.pat.as_ref() {
                syn::Pat::Ident(pat_ident) => Some(crate::naming::ident_name(&pat_ident.ident)),
                _ => None,
            },
            syn::FnArg::Receiver(_) => None,
        })
        .collect();
    let mut unknown: Vec<&String> = per_param
        .iter()
        .filter(|(name, a)| {
            (a.match_arg
                || a.choices.is_some()
                || a.no_default
                || !a.checks.is_empty()
                || a.preconditions.is_some())
                && !sig_names.contains(name.as_str())
        })
        .map(|(name, _)| name)
        .collect();
    unknown.sort();
    if let Some(first) = unknown.first() {
        return Err(syn::Error::new(
            span,
            format!(
                "match_arg/choices/no_default/inherits/not_inherits/no_na/(no_)preconditions \
                 references non-existent parameter `{first}`"
            ),
        ));
    }
    for arg in inputs {
        let syn::FnArg::Typed(pt) = arg else {
            continue;
        };
        let syn::Pat::Ident(pat_ident) = pt.pat.as_ref() else {
            continue;
        };
        let name = crate::naming::ident_name(&pat_ident.ident);
        let Some(attrs) = per_param.get_mut(&name) else {
            continue;
        };
        let has_default = attrs.default.is_some() || defaults.contains_key(&name);
        if attrs.no_default
            && let Some(msg) = no_default_conflict(&name, attrs.is_choice(), has_default, true)
        {
            return Err(syn::Error::new(span, msg));
        }
        classify_choice_param(attrs, &name, pt.ty.as_ref(), has_default)?;
        if let Some(msg) = attrs.checks.class_conflict(&name) {
            return Err(syn::Error::new(span, msg));
        }
        let param_markers = markers
            .iter()
            .find(|(n, _)| *n == name)
            .map_or(&[][..], |(_, m)| m.as_slice());
        check_param_preconditions(attrs, param_markers, &name, pt.ty.as_ref(), true)?;
    }
    Ok(())
}

fn optional_choice_default_msg(param_name: &str) -> String {
    format!(
        "`Option<T>` parameter `{param_name}` with match_arg/choices cannot have a default; \
         its R formal defaults to NULL, which means no choice. Drop the `Option` to make a \
         choice the default, or drop the default"
    )
}

/// Parses a Rust `fn` item from a token stream, performing all normalizations
/// required by the `#[miniextendr]` codegen pipeline.
///
/// # Normalizations performed
///
/// 1. **Variadic (`...`) rewriting**: Replaces Rust variadic syntax with a typed
///    `&miniextendr_api::dots::Dots` parameter. Named dots (`my_dots: ...`) preserve
///    the user's identifier; `_: ...` becomes `__miniextendr_dots`.
/// 2. **Wildcard pattern renaming**: `_` parameter patterns become `__unused0`,
///    `__unused1`, etc., so they can be passed by name to the C wrapper.
/// 3. **Destructuring expansion**: Tuple/struct destructuring patterns are replaced
///    with synthetic identifiers (`__param_0`, ...) and a `let` binding is prepended
///    to the function body.
/// 4. **Per-parameter attribute consumption**: `#[miniextendr(coerce)]`,
///    `#[miniextendr(match_arg)]`, `#[miniextendr(default = "...")]`, and
///    `#[miniextendr(choices(...))]` are consumed from parameters and recorded in
///    the corresponding `per_param_*` fields.
/// 5. **Validation**: Rejects `#[export_name]` on non-extern functions, rejects
///    unsupported parameter patterns, and validates that defaults reference existing
///    parameter names.
impl syn::parse::Parse for MiniextendrFunctionParsed {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        use syn::spanned::Spanned;

        let mut item: syn::ItemFn = input.parse()?;

        // dots support: replace `...` with `&Dots` (the dots parameter itself
        // is found by type once wildcard patterns have their names).
        rewrite_variadic_dots(&mut item.sig)?;

        // Reject #[export_name] for regular functions (not extern "C-unwind").
        // For extern functions, #[export_name] can be used as an alternative to #[no_mangle].
        let is_extern = item.sig.abi.is_some();
        if !is_extern {
            for attr in &item.attrs {
                if attr.path().is_ident("export_name") {
                    return Err(syn::Error::new_spanned(
                        attr,
                        "#[export_name] is not supported with #[miniextendr] on regular functions; \
                         use `#[miniextendr(c_symbol = \"...\")]` to customize the C symbol name. \
                         For extern \"C-unwind\" functions, #[export_name] is allowed.",
                    ));
                }
            }
        }

        // Transform `_` wildcard patterns to synthetic identifiers, and consume
        // per-parameter `#[miniextendr(coerce)]`, `#[miniextendr(default = "...")]`,
        // and `#[miniextendr(choices(...))]` attributes.
        let mut per_param: std::collections::HashMap<String, ParamAttrs> =
            std::collections::HashMap::new();
        let mut per_param_default_spans: std::collections::HashMap<String, proc_macro2::Span> =
            std::collections::HashMap::new();
        let mut unused_counter = 0usize;
        let mut pattern_destructures: Vec<(Box<syn::Pat>, syn::Ident)> = Vec::new();
        let mut param_markers: Vec<(String, Vec<crate::type_inspect::ParamMarker>)> = Vec::new();
        for arg in &mut item.sig.inputs {
            let syn::FnArg::Typed(pat_type) = arg else {
                // Self parameters are not allowed in standalone functions.
                // Users should use #[miniextendr(env|r6|s3|s4|s7)] on impl blocks instead.
                // The error is raised in lib.rs c_wrapper_inputs generation.
                continue;
            };

            // Consume the per-parameter miniextendr attributes (coerce,
            // match_arg, choices, several_ok, default, inherits, not_inherits,
            // no_na), merging
            // them when a parameter carries several; keep every other attribute.
            let mut param_attr = PerParamMiniextendrAttr::default();
            let mut kept_attrs = Vec::with_capacity(pat_type.attrs.len());
            for attr in std::mem::take(&mut pat_type.attrs) {
                match parse_per_param_attr(&attr)? {
                    Some(parsed) => param_attr
                        .merge(parsed)
                        .map_err(|msg| syn::Error::new_spanned(&attr, msg))?,
                    None => kept_attrs.push(attr),
                }
            }
            pat_type.attrs = kept_attrs;
            let PerParamMiniextendrAttr {
                has_coerce: had_coerce_attr,
                has_match_arg: had_match_arg_attr,
                default_value: default_with_span,
                choices: had_choices,
                has_several_ok: had_several_ok,
                has_no_default: had_no_default,
                checks: had_checks,
                preconditions: had_preconditions,
            } = param_attr;

            // Parameter markers (`Checked<T>` / `Unchecked<T>`, #1566): the
            // parameter converts as the inner type, which every check below
            // and every R-side consumer sees; the item keeps the marker, and
            // the C wrapper wraps the converted value before the call.
            let (markers, inner_ty) =
                crate::type_inspect::peel_param_markers(pat_type.ty.as_ref())?;
            let inner_ty = inner_ty.clone();
            if is_extern && let Some(marker) = markers.first() {
                return Err(syn::Error::new_spanned(
                    &pat_type.ty,
                    format!(
                        "`{}<T>` on an `extern \"C-unwind\"` function: it takes the R values \
                         as they are, with no generated conversion to unwrap the marker; take \
                         the inner type",
                        marker.name()
                    ),
                ));
            }

            // Validate type-based constraints (Missing nesting, Missing<Dots>)
            validate_param_type(&inner_ty, pat_type.ty.span())?;

            // Resolve the Rust parameter name — either the user's identifier,
            // or a synthesized one for wildcard / destructuring patterns.
            let param_name: String = match pat_type.pat.as_ref() {
                syn::Pat::Ident(pat_ident) => crate::naming::ident_name(&pat_ident.ident),
                syn::Pat::Wild(_) => {
                    let synthetic_name = format!("__unused{}", unused_counter);
                    unused_counter += 1;
                    let synthetic_ident = syn::Ident::new(&synthetic_name, pat_type.pat.span());
                    *pat_type.pat = syn::Pat::Ident(syn::PatIdent {
                        attrs: vec![],
                        by_ref: None,
                        mutability: None,
                        ident: synthetic_ident,
                        subpat: None,
                    });
                    synthetic_name
                }
                syn::Pat::Tuple(_) | syn::Pat::TupleStruct(_) | syn::Pat::Struct(_) => {
                    let synthetic_name = format!("__param_{}", unused_counter);
                    unused_counter += 1;
                    let synthetic_ident = syn::Ident::new(&synthetic_name, pat_type.pat.span());
                    let original_pat = pat_type.pat.clone();
                    *pat_type.pat = syn::Pat::Ident(syn::PatIdent {
                        attrs: vec![],
                        by_ref: None,
                        mutability: None,
                        ident: synthetic_ident.clone(),
                        subpat: None,
                    });
                    pattern_destructures.push((original_pat, synthetic_ident));
                    synthetic_name
                }
                _ => {
                    return Err(syn::Error::new(
                        pat_type.pat.span(),
                        "miniextendr parameters must be identifiers or destructuring patterns (tuple, struct)",
                    ));
                }
            };
            // The marker and the per-parameter keyword are one decision.
            let preconditions = merge_param_policy(
                &markers,
                had_preconditions,
                &param_name,
                false,
                &pat_type.ty,
            )?;

            // Validate per-parameter attribute conflicts (coerce+match_arg, coerce+choices, etc.)
            let per_param_combined = PerParamMiniextendrAttr {
                has_coerce: had_coerce_attr,
                has_match_arg: had_match_arg_attr,
                default_value: default_with_span.clone(),
                choices: had_choices.clone(),
                has_several_ok: had_several_ok,
                has_no_default: had_no_default,
                checks: had_checks.clone(),
                preconditions,
            };
            validate_per_param_attr_conflicts(
                &per_param_combined,
                &param_name,
                is_dots_type(&inner_ty),
                Some(&inner_ty),
                pat_type.ty.span(),
            )?;

            // Record per-parameter attrs in one entry instead of five side-tables.
            if had_coerce_attr
                || had_match_arg_attr
                || had_several_ok
                || had_choices.is_some()
                || default_with_span.is_some()
                || !had_checks.is_empty()
                || preconditions.is_some()
            {
                let entry = per_param.entry(param_name.clone()).or_default();
                entry.checks = had_checks;
                entry.coerce = had_coerce_attr;
                entry.match_arg = had_match_arg_attr;
                entry.several_ok = had_several_ok;
                entry.no_default = had_no_default;
                entry.choices = had_choices;
                entry.preconditions = preconditions;
                if let Some((default, span)) = default_with_span {
                    entry.default = Some(default);
                    per_param_default_spans.insert(param_name.clone(), span);
                }
                // The `Option` / `Missing` layers of a choice parameter (#1473, #1551).
                let has_default = entry.default.is_some();
                classify_choice_param(entry, &param_name, &inner_ty, has_default)?;
                check_param_preconditions(entry, &markers, &param_name, &inner_ty, false)?;
            }
            if !markers.is_empty() {
                param_markers.push((param_name, markers));
            }
        }

        // Insert destructuring let-bindings for pattern parameters at the start of the function body
        for (pat, ident) in pattern_destructures.iter().rev() {
            item.block.stmts.insert(
                0,
                syn::parse_quote! {
                    let #pat = #ident;
                },
            );
        }

        // Validate: all defaults reference existing parameters
        let param_names: std::collections::HashSet<String> = item
            .sig
            .inputs
            .iter()
            .filter_map(|input| {
                if let syn::FnArg::Typed(pat_type) = input
                    && let syn::Pat::Ident(pat_ident) = pat_type.pat.as_ref()
                {
                    Some(crate::naming::ident_name(&pat_ident.ident))
                } else {
                    None
                }
            })
            .collect();

        let mut invalid_params: Vec<String> = per_param
            .iter()
            .filter_map(|(name, attrs)| {
                if attrs.default.is_some() && !param_names.contains(name) {
                    Some(name.clone())
                } else {
                    None
                }
            })
            .collect();
        invalid_params.sort();

        if !invalid_params.is_empty() {
            // Use the span of the first invalid param's attribute for the error
            let error_span = invalid_params
                .first()
                .and_then(|p| per_param_default_spans.get(p).copied())
                .unwrap_or_else(|| item.sig.ident.span());
            return Err(syn::Error::new(
                error_span,
                format!(
                    "default attribute(s) reference non-existent parameter(s): {}",
                    invalid_params.join(", ")
                ),
            ));
        }

        let dots = find_dots_param(&item.sig.inputs)?;
        crate::r_wrapper_builder::check_r_formals(&item.sig.inputs, &[])?;
        let inputs = peeled_inputs(&item.sig.inputs, &param_markers)?;

        Ok(Self {
            item,
            inputs,
            param_markers,
            dots,
            per_param,
        })
    }
}

/// Accessors and codegen helpers for [`MiniextendrFunctionParsed`].
///
/// Accessors are split into two groups:
/// - **Parsed metadata**: dots, coerce, match_arg, choices, and defaults from
///   per-parameter `#[miniextendr(...)]` attributes.
/// - **Signature components**: attrs, vis, abi, ident, generics, inputs, output
///   from the normalized `syn::ItemFn`.
///
/// Codegen helpers produce identifiers and perform mutations needed by the
/// `#[miniextendr]` expansion pipeline.
impl MiniextendrFunctionParsed {
    // region: Accessors for parsed metadata

    /// Whether the function takes `...`: Rust `...` or an explicit `&Dots`
    /// parameter, at any position.
    pub(crate) fn has_dots(&self) -> bool {
        self.dots.is_some()
    }

    /// The Rust binding of the dots: the user's name (`args: ...`,
    /// `rest: &Dots`), or `__miniextendr_dots` for `_: ...`.
    pub(crate) fn dots_ident(&self) -> Option<&syn::Ident> {
        self.dots.as_ref()
    }

    /// Check if a parameter is the dots (`...`) param, whose R formal is `...`.
    pub(crate) fn is_dots_param(&self, ident: &syn::Ident) -> bool {
        self.dots_ident() == Some(ident)
    }

    /// Whether a parameter carried any per-parameter `#[miniextendr(...)]`
    /// option. A `Call` / `CallerCall` marker (#1566) is bound from the call
    /// slot, not from an R argument, so none of them apply to it.
    pub(crate) fn has_param_attrs(&self, param_name: &str) -> bool {
        self.per_param.contains_key(param_name)
    }

    /// Check if a parameter name had `#[miniextendr(coerce)]` attribute.
    pub(crate) fn has_coerce_attr(&self, param_name: &str) -> bool {
        self.per_param.get(param_name).is_some_and(|a| a.coerce)
    }

    /// Check if a parameter name had `#[miniextendr(match_arg)]` attribute.
    pub(crate) fn has_match_arg_attr(&self, param_name: &str) -> bool {
        self.per_param.get(param_name).is_some_and(|a| a.match_arg)
    }

    /// Iterator over parameter names annotated with `#[miniextendr(match_arg)]`,
    /// in signature order.
    pub(crate) fn match_arg_params(&self) -> impl Iterator<Item = &String> {
        per_param_in_signature_order(&self.item.sig.inputs, &self.per_param)
            .filter_map(|(name, a)| if a.match_arg { Some(name) } else { None })
    }

    /// Iterator over parameter names annotated with `#[miniextendr(choices(…))]`,
    /// together with their choice lists, in signature order.
    pub(crate) fn choices_params(&self) -> impl Iterator<Item = (&String, &Vec<String>)> {
        per_param_in_signature_order(&self.item.sig.inputs, &self.per_param)
            .filter_map(|(name, a)| a.choices.as_ref().map(|c| (name, c)))
    }

    /// Check if a parameter has `several_ok` (multi-value match.arg).
    pub(crate) fn has_several_ok(&self, param_name: &str) -> bool {
        self.per_param.get(param_name).is_some_and(|a| a.several_ok)
    }

    /// The per-parameter attribute state of `param_name`, if it carried any
    /// `#[miniextendr(...)]` option. Choice parameters read their layers
    /// (`Option` / `Missing`, #1473 / #1551) from here.
    pub(crate) fn param_attrs(&self, param_name: &str) -> Option<&ParamAttrs> {
        self.per_param.get(param_name)
    }

    /// The `inherits` / `not_inherits` / `no_na` checks of every parameter,
    /// keyed by R name (see [`explicit_checks_by_r_name`]).
    pub(crate) fn explicit_checks(
        &self,
    ) -> std::collections::HashMap<String, crate::r_preconditions::ExplicitChecks> {
        explicit_checks_by_r_name(&self.per_param)
    }

    /// R names of the `match_arg` and `choices(...)` parameters, whose
    /// type-derived preconditions are skipped because `match.arg()` already
    /// validates them. The method paths build the same set.
    pub(crate) fn precondition_skip_params(&self) -> std::collections::HashSet<String> {
        crate::r_class_formatter::match_arg_skip_set(&self.per_param)
    }

    /// Returns all parameter defaults as an owned map from parameter name to
    /// default value string (the raw R expression used in the wrapper formals,
    /// e.g. `"NULL"`, `"TRUE"`, `"\"Safe\""`).
    pub(crate) fn param_defaults(&self) -> std::collections::HashMap<String, String> {
        self.per_param
            .iter()
            .filter_map(|(name, a)| a.default.as_ref().map(|d| (name.clone(), d.clone())))
            .collect()
    }
    // endregion

    // region: Accessors for signature components

    /// Original attributes on the function item (doc comments, cfgs, etc.).
    pub(crate) fn attrs(&self) -> &[syn::Attribute] {
        &self.item.attrs
    }

    /// Visibility of the function (`pub`, `pub(crate)`, or private).
    pub(crate) fn vis(&self) -> &syn::Visibility {
        &self.item.vis
    }

    /// Explicit ABI, if the function was declared `extern "C-unwind"`.
    pub(crate) fn abi(&self) -> Option<&syn::Abi> {
        self.item.sig.abi.as_ref()
    }

    /// Function identifier after normalization.
    pub(crate) fn ident(&self) -> &syn::Ident {
        &self.item.sig.ident
    }

    /// Generic parameters on the function signature.
    pub(crate) fn generics(&self) -> &syn::Generics {
        &self.item.sig.generics
    }

    /// Function inputs after normalization (dots rewritten, wildcards
    /// renamed), with the parameter markers peeled (see [`Self::param_markers`]).
    pub(crate) fn inputs(&self) -> &syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]> {
        &self.inputs
    }

    /// The `Checked<T>` / `Unchecked<T>` markers peeled off the parameters
    /// (Rust name → markers, outermost first), in signature order.
    pub(crate) fn param_markers(&self) -> &[(String, Vec<crate::type_inspect::ParamMarker>)] {
        &self.param_markers
    }

    /// Function return type.
    pub(crate) fn output(&self) -> &syn::ReturnType {
        &self.item.sig.output
    }

    /// The normalized function item (with original doc comments).
    pub(crate) fn item(&self) -> &syn::ItemFn {
        &self.item
    }

    /// The normalized function item with roxygen tags stripped from doc comments.
    ///
    /// This is used for emitting the Rust function without R-specific documentation
    /// tags (e.g., `@param`, `@examples`) that don't belong in rustdoc.
    pub(crate) fn item_without_roxygen(&self) -> syn::ItemFn {
        let mut item = self.item.clone();
        item.attrs = crate::roxygen::strip_roxygen_from_attrs(&item.attrs);
        item
    }
    // endregion

    // region: Codegen helpers

    /// Returns `true` if this function needs an internal C wrapper (`C_<crate>_<name>` function).
    ///
    /// Rust-ABI functions (no explicit `extern`) need a generated `extern "C-unwind"` wrapper
    /// that handles SEXP conversion and error propagation. Functions already declared as
    /// `extern "C-unwind"` are passed through directly without wrapping.
    pub(crate) fn uses_internal_c_wrapper(&self) -> bool {
        self.abi().is_none()
    }

    /// Returns the identifier for the generated `const &str` holding the R wrapper code.
    ///
    /// The R wrapper is a string constant containing the R function definition that
    /// calls `.Call(C_<crate>_<name>, ...)`. It is collected via linkme distributed slices to
    /// produce the `R/miniextendr-wrappers.R` file.
    pub(crate) fn r_wrapper_const_ident(&self) -> syn::Ident {
        r_wrapper_const_ident_for(self.ident())
    }

    /// Returns the identifier for the C-callable entry point.
    ///
    /// - **Rust ABI functions**: Returns `C_<crate>_<name>` (the generated wrapper
    ///   function, crate-prefixed for webR cross-package symbol uniqueness — #1273).
    /// - **`extern "C-unwind"` functions**: Returns the function's own name, or the
    ///   value from `#[export_name = "..."]` if present. The user owns these symbols,
    ///   including their cross-package uniqueness under webR.
    pub(crate) fn c_wrapper_ident(&self) -> syn::Ident {
        if self.uses_internal_c_wrapper() {
            crate::naming::bare_fn_c_wrapper_ident(self.ident())
        } else {
            // For extern functions, check for #[export_name = "..."]
            self.export_name_ident()
                .unwrap_or_else(|| self.ident().clone())
        }
    }

    /// Extracts the custom symbol name from `#[export_name = "..."]`, if present.
    ///
    /// Only meaningful for `extern "C-unwind"` functions, where `#[export_name]` is
    /// allowed as an alternative to `#[no_mangle]`. Returns `None` if no such attribute exists.
    pub(crate) fn export_name_ident(&self) -> Option<syn::Ident> {
        for attr in &self.item.attrs {
            if attr.path().is_ident("export_name")
                && let syn::Meta::NameValue(meta) = &attr.meta
                && let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit_str),
                    ..
                }) = &meta.value
            {
                return Some(syn::Ident::new(&lit_str.value(), lit_str.span()));
            }
        }
        None
    }

    /// Add `#[inline(never)]` if no `#[inline(...)]` attribute is present.
    /// Only for Rust ABI functions - extern "C-unwind" functions are passed through as-is.
    ///
    /// Preventing inlining ensures:
    /// - Worker-dispatched functions retain a distinct call frame
    /// - Panic handling and unwinding retain the intended boundary
    /// - Stack traces show the actual function name
    pub(crate) fn add_inline_never_if_needed(&mut self) {
        let has_explicit_abi = self.item.sig.abi.is_some();
        let has_inline = self
            .item
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("inline"));
        if !has_inline && !has_explicit_abi {
            self.item.attrs.push(syn::parse_quote!(#[inline(never)]));
        }
    }
    // endregion
}
// endregion

// region: Attribute parsing

/// Parse the value of a `name = "..."` meta item as a string literal.
///
/// Returns a compile error spanning the offending token when the RHS is not a
/// `&str` literal. `field` is used in the diagnostic (e.g. `"c_symbol"`).
/// `postfix = "..."` must be a non-empty R identifier fragment: it is appended
/// verbatim to the Rust name, so anything outside letters, digits, `_` and `.`
/// would produce an R name that needs backticks.
pub(crate) fn validate_postfix(val: &str, span: &dyn quote::ToTokens) -> syn::Result<()> {
    if val.is_empty() {
        return Err(syn::Error::new_spanned(span, "postfix must not be empty"));
    }
    if !val
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
    {
        return Err(syn::Error::new_spanned(
            span,
            "postfix must be a valid R identifier fragment (letters, digits, `_`, `.`)",
        ));
    }
    Ok(())
}

fn parse_lit_str(nv: &syn::MetaNameValue, field: &str) -> syn::Result<String> {
    match &nv.value {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(lit),
            ..
        }) => Ok(lit.value()),
        syn::Expr::Lit(expr_lit) => Err(syn::Error::new_spanned(
            &expr_lit.lit,
            format!("{field} expects a string literal"),
        )),
        other => Err(syn::Error::new_spanned(
            other,
            format!("{field} expects a string literal"),
        )),
    }
}

/// Comma-separated list of all fn-level boolean flags, for error messages.
///
/// Kept as a single constant so the three "unknown option" error paths (Path,
/// NameValue bool, parenthesized bool) all read from the same list and can't
/// drift.
const FN_BOOL_FLAGS_HELP: &str = "invisible, visible, check_interrupt, worker, no_worker, coerce, no_coerce, \
     rng, unwrap_in_r, serialize, serde_error, strict, no_strict, \
     preconditions, no_preconditions, internal, noexport, export";

/// Comma-separated list of fn-level nested options, for error messages.
const FN_NESTED_OPTIONS_HELP: &str =
    "`s3(...)`, `lifecycle(...)`, `defaults(...)`, `r_on_exit(...)`, `serde_error(...)`";

/// Parsed arguments for the `#[miniextendr(...)]` attribute on functions.
///
/// This is intentionally a small, "data-only" struct that:
/// - Owns the parsing rules for the attribute
/// - Produces a normalized, easy-to-consume representation for codegen
///
/// # Accepted flags
///
/// - `invisible` / `visible`: control whether the generated R wrapper returns invisibly
/// - `check_interrupt`: insert `R_CheckUserInterrupt()` before calling Rust
/// - `worker`: opt into worker-thread execution (default is main thread)
/// - `coerce`: enable automatic coercion for supported parameter types
/// - `rng`: enable RNG state management (GetRNGstate/PutRNGstate)
/// - `unwrap_in_r`: return `Result<T, E>` to R without unwrapping
/// - `prefer = "auto" | "list" | "externalptr" | "vector"`: prefer a specific `IntoR` path
/// - `no_preconditions` / `preconditions` (bare or `= true/false`): drop or
///   keep the R-side type checks. Without them `TryFromSexp` still raises on
///   bad input, with the same argument-error condition (#1591); the message
///   comes from the conversion. Saves one `isTRUE()` guard per check.
///   Hot-path opt-in. The last one written wins. A parameter decides for
///   itself first (`Checked<T>` / `Unchecked<T>`, or the per-parameter
///   `#[miniextendr(preconditions)]` / `#[miniextendr(no_preconditions)]`);
///   below the function come the crate's `[package.metadata.miniextendr]
///   preconditions = true | false` and then the `no-preconditions-default`
///   feature (`crate::r_preconditions::resolve_type_checks`).
/// - `call = wrapper | caller`: which call the wrapper attributes conditions
///   to (#1566), always as written. `wrapper` (the framework default) passes
///   `.call = sys.call()`; `caller` binds the caller's call first and passes
///   that (internal entry points behind a hand-written R function; needs
///   `noexport` / `internal`). A `Call` / `CallerCall` parameter is the
///   marker spelling of `wrapper` / `caller`, and
///   `[package.metadata.miniextendr] call_attribution` the crate default; see
///   `CallAttribution::resolve`.
///
/// # Note
///
/// Unknown flags are rejected with a compile error to avoid silently ignoring typos.
#[derive(Default)]
pub(crate) struct MiniextendrFnAttrs {
    /// Force execution on worker thread (set by `worker`).
    pub(crate) force_worker: bool,
    /// Override visibility; `Some(true)` makes the wrapper return invisibly, `Some(false)` forces visibility.
    pub(crate) force_invisible: Option<bool>,
    /// Insert `R_CheckUserInterrupt()` before calling the Rust function.
    pub(crate) check_interrupt: bool,
    /// Enable automatic coercion for all parameters that support it.
    pub(crate) coerce_all: bool,
    /// Enable RNG state management (GetRNGstate/PutRNGstate).
    pub(crate) rng: bool,
    /// Return `Result<T, E>` to R without unwrapping.
    pub(crate) unwrap_in_r: bool,
    /// Serialize the complete return value through `AsSerialize<T>`.
    pub(crate) serialize: bool,
    pub(crate) wrap: Option<crate::miniextendr_impl::ClassSystem>,
    /// Build the `Err` arm's condition from the error's serde output
    /// (`#[miniextendr(serde_error)]`, optionally `serde_error(tag = .., prefix = ..)`).
    pub(crate) serde_error: Option<SerdeErrorSpec>,
    /// Keep (`Some(true)`) or drop (`Some(false)`) the R-side type-check
    /// guards of the parameters that do not decide for themselves.
    ///
    /// `TryFromSexp` already raises the same argument-error condition on
    /// mismatched input (#1591), so the information isn't lost: it is
    /// worded by the conversion rather than by the R check. Useful for hot
    /// paths where the per-call precondition cost (one `isTRUE()` guard per
    /// check) dominates over actual work.
    ///
    /// `Some(true)` from `#[miniextendr(preconditions)]` (or
    /// `preconditions = true`, `no_preconditions = false`), `Some(false)` from
    /// `no_preconditions` (or `no_preconditions = true`, `preconditions =
    /// false`); the last one written wins. `None`: the crate default, then the
    /// `no-preconditions-default` feature decide, at codegen
    /// (`crate::r_preconditions::resolve_type_checks`).
    pub(crate) preconditions: Option<bool>,
    /// The attribution the attribute asked for, if any (#1566):
    /// `call = wrapper | caller`. `None` here means the attribute said
    /// nothing; the codegen then falls back to a `Call` / `CallerCall`
    /// parameter marker, the crate default and finally `wrapper`
    /// (`crate::r_wrapper_builder::CallAttribution::resolve`).
    pub(crate) call_attribution: Option<crate::r_wrapper_builder::CallAttribution>,
    /// Preferred return conversion: forces `AsList`/`AsExternalPtr`/`AsRNative` wrapping
    /// of the return value before `IntoR::into_sexp` is called.
    pub(crate) return_pref: ReturnPref,
    /// Span of the `prefer = ...` attribute, for error reporting when the return type
    /// falls into a codegen category (`Option<T>`, `Result<T, E>`, `()`, `Self`, raw
    /// `SEXP`, ...) that can't honor it.
    pub(crate) return_pref_span: Option<proc_macro2::Span>,
    /// S3 generic name (if this function is an S3 method).
    ///
    /// Use `#[miniextendr(s3(generic = "vec_proxy", class = "my_vctr"))]` to mark a function
    /// as an S3 method for an existing generic.
    pub(crate) s3_generic: Option<String>,
    /// S3 class suffix for the method (e.g., "my_vctr" or "my_vctr.my_vctr" for double-dispatch).
    pub(crate) s3_class: Option<String>,
    /// Typed list validation spec for dots parameter.
    ///
    /// Use `#[miniextendr(dots = typed_list!(...))]` to automatically validate dots
    /// at the start of the function and bind the result to `dots_typed`.
    pub(crate) dots_spec: Option<proc_macro2::TokenStream>,
    /// Span of the `dots = ...` attribute for error reporting.
    pub(crate) dots_span: Option<proc_macro2::Span>,
    /// Lifecycle specification for deprecation/experimental status.
    pub(crate) lifecycle: Option<crate::lifecycle::LifecycleSpec>,
    /// Strict output conversion: panic instead of lossy widening for i64/u64/isize/usize.
    pub(crate) strict: bool,
    /// Mark as internal: adds `@keywords internal`, suppresses `@export`.
    pub(crate) internal: bool,
    /// Suppress `@export` without adding `@keywords internal`.
    pub(crate) noexport: bool,
    /// Force `@export` even on non-pub functions. Antidote to `noexport`.
    pub(crate) export: bool,
    /// Custom roxygen documentation override.
    ///
    /// When set, replaces auto-extracted roxygen from Rust doc comments.
    /// Each `\n` in the string becomes a separate `#'` line.
    pub(crate) doc: Option<String>,
    /// Custom C symbol name for the generated wrapper.
    ///
    /// Overrides the default `C_<crate>_<fn_name>` naming convention. The value is used
    /// verbatim (no crate prefix) — the author owns cross-package uniqueness on webR (#1273).
    /// Must be a valid C identifier (alphanumeric + underscore, starting with letter or underscore).
    pub(crate) c_symbol: Option<String>,
    /// Override R wrapper function name.
    ///
    /// Use `#[miniextendr(r_name = "is.my_type")]` to give the R wrapper a different name
    /// than the Rust function. The C symbol is still derived from the Rust name.
    /// Cannot be combined with `s3(generic/class)` — use `generic`/`class` for S3 naming.
    pub(crate) r_name: Option<String>,
    /// Append a fixed suffix to the Rust name for the R wrapper
    /// (`#[miniextendr(noexport, postfix = "_impl")]` on `fn f` yields `f_impl`).
    /// States the "hand-written `f()` delegates to generated `f_impl()`"
    /// convention without repeating the name in `r_name`. Exclusive with
    /// `r_name` and `s3(...)`; the C symbol is unchanged.
    pub(crate) postfix: Option<String>,
    /// R code to inject at the very top of the wrapper body (before all built-in checks).
    ///
    /// Use `#[miniextendr(r_entry = "x <- as.integer(x)")]` to run R code before
    /// missing-default handling, lifecycle checks, preconditions, and match.arg.
    /// Multi-line via `\n`. No validation of R syntax.
    pub(crate) r_entry: Option<String>,
    /// R code to inject after all built-in checks, immediately before `.Call()`.
    ///
    /// Use `#[miniextendr(r_post_checks = "message('calling rust')")]` to run R code
    /// after all precondition checks but before the Rust function is invoked.
    /// Multi-line via `\n`. No validation of R syntax.
    pub(crate) r_post_checks: Option<String>,
    /// Register `on.exit()` cleanup code in the R wrapper.
    ///
    /// Short form: `#[miniextendr(r_on_exit = "close(con)")]` → `on.exit(close(con), add = TRUE)`
    ///
    /// Long form: `#[miniextendr(r_on_exit(expr = "close(con)", add = false))]`
    ///
    /// Defaults: `add = TRUE`, `after = TRUE`. Injected after `r_entry`, before other checks.
    pub(crate) r_on_exit: Option<ROnExit>,
}

/// Parsed `r_on_exit` attribute for `on.exit()` cleanup code in R wrappers.
///
/// Two forms:
/// - Short: `r_on_exit = "expr"` → `ROnExit { expr, add: true, after: true }`
/// - Long: `r_on_exit(expr = "...", add = false, after = false)`
///
/// Defaults match R conventions for composable code: `add = TRUE`, `after = TRUE`.
#[derive(Debug, Clone)]
pub(crate) struct ROnExit {
    pub expr: String,
    pub add: bool,
    pub after: bool,
}

impl ROnExit {
    /// Generate the R `on.exit(...)` call string.
    ///
    /// - `add = FALSE` (R default): `on.exit(expr)`
    /// - `add = TRUE, after = TRUE`: `on.exit(expr, add = TRUE)`
    /// - `add = TRUE, after = FALSE`: `on.exit(expr, add = TRUE, after = FALSE)`
    pub fn to_r_code(&self) -> String {
        if !self.add {
            format!("on.exit({})", self.expr)
        } else if !self.after {
            format!("on.exit({}, add = TRUE, after = FALSE)", self.expr)
        } else {
            format!("on.exit({}, add = TRUE)", self.expr)
        }
    }
}

/// `#[miniextendr(serde_error)]`: derive the `Err` arm's condition class and
/// data from the error type's `serde::Serialize` output instead of the
/// `RConditionError`/`Debug` probe.
///
/// The enum variant (external tagging, or the `tag` field of an internally
/// tagged enum) becomes the member class `<prefix>_<variant>`; the payload
/// fields become the condition's data. Defaults: `tag = "kind"`,
/// `prefix = "<crate>_error"` (from `CARGO_CRATE_NAME` at expansion time).
///
/// Field control (#1457): `skip("a", "b")` drops payload fields by name,
/// `rename(a = "b")` splices field `a` as `b`. Both name the field as it
/// serializes; a variant that lacks the field is unaffected. The macro cannot
/// see the error type's fields, so only the option grammar is checked here;
/// a `rename` target may not be one of the reserved condition slots.
///
/// The serde path itself needs no attribute: under the `serde` feature every
/// `Result<T, E>` with `E: Serialize + Display` takes it through the runtime
/// probe. A spec only exists to carry options, so the bare flag and the
/// boolean forms are rejected with [`SERDE_ERROR_BARE_HELP`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct SerdeErrorSpec {
    /// Internally-tagged discriminator field name (`#[serde(tag = "...")]`).
    pub tag: Option<String>,
    /// Condition class prefix (family class).
    pub prefix: Option<String>,
    /// Payload fields dropped from the condition data.
    pub skip: Vec<String>,
    /// Payload fields spliced under another name: `(from, to)`.
    pub rename: Vec<(String, String)>,
}

/// The condition's own slots, mirrored from
/// `miniextendr_api::condition::RESERVED_CONDITION_FIELDS` (the macros crate
/// cannot depend on the API crate). The runtime check remains the backstop.
const RESERVED_CONDITION_FIELDS: &[&str] = &["message", "call", "kind"];

const SERDE_ERROR_OPTIONS_HELP: &str =
    "unknown serde_error option; expected `tag`, `prefix`, `skip(...)` or `rename(...)`";

/// The bare flag / boolean forms: the serde path is not something to switch
/// on per function.
pub(crate) const SERDE_ERROR_BARE_HELP: &str = "`serde_error` is not a switch: with the `serde` \
    feature every `Result<T, E>` whose `E: Serialize + Display` is already classed from its \
    serde shape. The attribute only carries options; write \
    `serde_error(tag = \"..\", prefix = \"..\", skip(..), rename(a = \"..\"))` or drop it";

impl SerdeErrorSpec {
    /// The discriminator field consumed as the variant name.
    pub fn tag(&self) -> &str {
        self.tag.as_deref().unwrap_or("kind")
    }

    /// The family class; `<crate>_error` unless overridden.
    pub fn prefix(&self) -> String {
        self.prefix
            .clone()
            .unwrap_or_else(default_serde_error_prefix)
    }

    /// Parse `serde_error(...)` contents: a comma-separated list of options.
    fn from_metas<'a>(
        metas: impl IntoIterator<Item = &'a syn::Meta>,
        span: proc_macro2::Span,
    ) -> syn::Result<Self> {
        let mut spec = SerdeErrorSpec::default();
        let mut any = false;
        for meta in metas {
            any = true;
            spec.apply(meta)?;
        }
        if !any {
            return Err(syn::Error::new(span, SERDE_ERROR_BARE_HELP));
        }
        spec.finish()?;
        Ok(spec)
    }

    fn apply(&mut self, meta: &syn::Meta) -> syn::Result<()> {
        match meta {
            syn::Meta::NameValue(nv) if nv.path.is_ident("tag") => {
                self.tag = Some(non_empty_lit_str(nv, "tag")?);
            }
            syn::Meta::NameValue(nv) if nv.path.is_ident("prefix") => {
                self.prefix = Some(non_empty_lit_str(nv, "prefix")?);
            }
            syn::Meta::List(list) if list.path.is_ident("skip") => {
                let names = list.parse_args_with(
                    syn::punctuated::Punctuated::<syn::LitStr, syn::Token![,]>::parse_terminated,
                )?;
                if names.is_empty() {
                    return Err(syn::Error::new_spanned(
                        list,
                        "serde_error skip needs at least one field name: `skip(\"message\")`",
                    ));
                }
                for lit in &names {
                    let name = lit.value();
                    if name.is_empty() {
                        return Err(syn::Error::new_spanned(
                            lit,
                            "serde_error skip field name must not be empty",
                        ));
                    }
                    if self.skip.contains(&name) {
                        return Err(syn::Error::new_spanned(
                            lit,
                            format!("serde_error skip names `{name}` twice"),
                        ));
                    }
                    self.skip.push(name);
                }
            }
            syn::Meta::List(list) if list.path.is_ident("rename") => {
                let pairs = list.parse_args_with(
                    syn::punctuated::Punctuated::<RenamePair, syn::Token![,]>::parse_terminated,
                )?;
                if pairs.is_empty() {
                    return Err(syn::Error::new_spanned(
                        list,
                        "serde_error rename needs at least one pair: `rename(message = \"detail\")`",
                    ));
                }
                for pair in pairs {
                    let RenamePair { from, to, span } = pair;
                    if from.is_empty() || to.is_empty() {
                        return Err(syn::Error::new(
                            span,
                            "serde_error rename names must not be empty",
                        ));
                    }
                    if RESERVED_CONDITION_FIELDS.contains(&to.as_str()) {
                        return Err(syn::Error::new(
                            span,
                            format!(
                                "serde_error rename target `{to}` is reserved: `message`, `call` \
                                 and `kind` are the condition's own slots"
                            ),
                        ));
                    }
                    if self.rename.iter().any(|(f, _)| *f == from) {
                        return Err(syn::Error::new(
                            span,
                            format!("serde_error rename names `{from}` twice"),
                        ));
                    }
                    if self.rename.iter().any(|(_, t)| *t == to) {
                        return Err(syn::Error::new(
                            span,
                            format!(
                                "serde_error rename targets `{to}` twice; the condition would \
                                 carry two `{to}` fields and R would read only the first"
                            ),
                        ));
                    }
                    self.rename.push((from, to));
                }
            }
            syn::Meta::NameValue(nv) if nv.path.is_ident("skip") => {
                return Err(syn::Error::new_spanned(
                    nv,
                    "serde_error skip takes a list of field names: `skip(\"message\")`",
                ));
            }
            syn::Meta::NameValue(nv) if nv.path.is_ident("rename") => {
                return Err(syn::Error::new_spanned(
                    nv,
                    "serde_error rename takes `from = \"to\"` pairs: `rename(message = \"detail\")`",
                ));
            }
            other => {
                return Err(syn::Error::new_spanned(
                    other.path(),
                    SERDE_ERROR_OPTIONS_HELP,
                ));
            }
        }
        Ok(())
    }

    /// Cross-option validation once every option is in.
    fn finish(&self) -> syn::Result<()> {
        if let Some((from, _)) = self
            .rename
            .iter()
            .find(|(from, _)| self.skip.contains(from))
        {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                format!(
                    "serde_error names `{from}` in both skip and rename; a skipped field has no \
                     name to rename"
                ),
            ));
        }
        Ok(())
    }
}

/// A `tag = "..."` / `prefix = "..."` value: a non-empty string literal.
fn non_empty_lit_str(nv: &syn::MetaNameValue, key: &str) -> syn::Result<String> {
    let val = parse_lit_str(nv, key)?;
    if val.is_empty() {
        return Err(syn::Error::new_spanned(
            &nv.value,
            format!("serde_error {key} must not be empty"),
        ));
    }
    Ok(val)
}

/// One `from = "to"` entry of `rename(...)`. `from` is an identifier or, for a
/// serde-renamed field whose name is not one, a string literal.
struct RenamePair {
    from: String,
    to: String,
    span: proc_macro2::Span,
}

impl syn::parse::Parse for RenamePair {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        use syn::ext::IdentExt;
        let lookahead = input.lookahead1();
        let (from, span) = if lookahead.peek(syn::LitStr) {
            let lit: syn::LitStr = input.parse()?;
            (lit.value(), lit.span())
        } else if lookahead.peek(syn::Ident::peek_any) {
            let ident = input.call(syn::Ident::parse_any)?;
            (ident.to_string(), ident.span())
        } else {
            return Err(lookahead.error());
        };
        let _: syn::Token![=] = input.parse()?;
        let to: syn::LitStr = input.parse()?;
        Ok(RenamePair {
            from,
            to: to.value(),
            span,
        })
    }
}

/// Default family class for `serde_error`: `<crate>_error`, from the crate being
/// compiled (cargo sets `CARGO_CRATE_NAME` for the rustc invocation that runs
/// this proc macro).
pub(crate) fn default_serde_error_prefix() -> String {
    let krate = std::env::var("CARGO_CRATE_NAME").unwrap_or_else(|_| "rust".to_string());
    format!("{krate}_error")
}

/// Parse `serde_error(tag = "...", prefix = "...", skip(...), rename(...))`
/// given as a `Meta::List`.
pub(crate) fn parse_serde_error_list(list: &syn::MetaList) -> syn::Result<SerdeErrorSpec> {
    use syn::spanned::Spanned;
    let metas = list.parse_args_with(
        syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
    )?;
    SerdeErrorSpec::from_metas(&metas, list.span())
}

/// Parse the tail of a `serde_error` option inside `parse_nested_meta`. Only
/// the option list `(tag = "...", prefix = "...", skip(...), rename(...))` is
/// accepted; the bare flag and `= true/false` are rejected with
/// [`SERDE_ERROR_BARE_HELP`], since the serde path is on for every eligible
/// error type under the `serde` feature.
pub(crate) fn parse_serde_error_nested(
    meta: &syn::meta::ParseNestedMeta,
) -> syn::Result<SerdeErrorSpec> {
    use syn::spanned::Spanned;
    let input = meta.input;
    if input.peek(syn::token::Paren) {
        let content;
        syn::parenthesized!(content in input);
        let metas =
            content.parse_terminated(<syn::Meta as syn::parse::Parse>::parse, syn::Token![,])?;
        return SerdeErrorSpec::from_metas(&metas, meta.path.span());
    }
    if input.peek(syn::Token![=]) {
        let _: syn::Token![=] = input.parse()?;
        let _: syn::LitBool = input.parse()?;
    }
    Err(meta.error(SERDE_ERROR_BARE_HELP))
}

#[derive(Clone, Copy, Default)]
/// Preferred return-conversion path for `IntoR`.
pub(crate) enum ReturnPref {
    /// Use the default `IntoR` implementation for the type.
    #[default]
    Auto,
    /// Force list conversion via the `AsList` wrapper.
    List,
    /// Force external pointer conversion via the `AsExternalPtr` wrapper.
    ExternalPtr,
    /// Force native vector/scalar conversion via the `AsRNative` wrapper.
    Native,
}

/// Parses the comma-separated option list inside `#[miniextendr(...)]`.
///
/// Supports three syntactic forms for each option:
/// - **Bare identifier**: `#[miniextendr(invisible)]`
/// - **Name-value**: `#[miniextendr(prefer = "list")]` or `#[miniextendr(invisible = true)]`
/// - **Nested list**: `#[miniextendr(s3(generic = "...", class = "..."))]`
///
/// Options with negated forms (`no_worker`, `no_coerce`, `no_strict`) explicitly
/// disable the corresponding flag, which is useful for overriding feature-based
/// defaults.
///
/// An empty input (plain `#[miniextendr]`) resolves all options to their feature-based
/// defaults (e.g., `worker-default`, `coerce-default`, `strict-default`).
///
/// # Errors
///
/// Returns a compile error for:
/// - Unknown option names (prevents silent typos)
/// - Mutually exclusive options (`internal` + `noexport`)
/// - Invalid values for key-value options (e.g., bad `prefer` or `c_symbol`)
/// - Missing required sub-options (e.g., `s3(...)` without `class`)
impl syn::parse::Parse for MiniextendrFnAttrs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        use syn::spanned::Spanned;
        // Use Option<bool> for fields that support feature defaults.
        // None = not explicitly set → resolve from cfg!(feature = "...") at end.
        let mut force_worker: Option<bool> = None;
        let mut force_invisible: Option<bool> = None;
        let mut check_interrupt = false;
        let mut coerce_all: Option<bool> = None;
        let mut rng = false;
        let mut unwrap_in_r = false;
        let mut serialize = false;
        let mut wrap = None;
        let mut serde_error: Option<SerdeErrorSpec> = None;
        let mut preconditions: Option<bool> = None;
        let mut return_pref = ReturnPref::Auto;
        let mut return_pref_span: Option<proc_macro2::Span> = None;
        let mut s3_generic = None;
        let mut s3_class = None;
        let mut dots_spec = None;
        let mut dots_span = None;
        let mut lifecycle = None;
        let mut strict: Option<bool> = None;
        let mut internal = false;
        let mut noexport = false;
        let mut export = false;
        let mut doc = None;
        let mut c_symbol = None;
        let mut r_name = None;
        let mut postfix = None;
        let mut call_attr: Option<crate::r_wrapper_builder::CallAttribution> = None;
        let mut r_entry = None;
        let mut r_post_checks = None;
        let mut r_on_exit = None;

        // Empty input (`#[miniextendr]`) → skip the parse loop and fall through
        // to the single Ok(Self {...}) at the bottom; every local is already
        // seeded with its default value above.
        let metas = if input.is_empty() {
            syn::punctuated::Punctuated::new()
        } else {
            syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated(input)?
        };

        for meta in metas {
            match meta {
                // Simple identifiers: invisible, visible, check_interrupt, coerce, worker, rng
                syn::Meta::Path(path) => {
                    if let Some(ident) = path.get_ident() {
                        if ident == "invisible" {
                            force_invisible = Some(true);
                        } else if ident == "visible" {
                            force_invisible = Some(false);
                        } else if ident == "check_interrupt" {
                            check_interrupt = true;
                        } else if ident == "coerce" {
                            coerce_all = Some(true);
                        } else if ident == "no_coerce" {
                            coerce_all = Some(false);
                        } else if ident == "rng" {
                            rng = true;
                        } else if ident == "unwrap_in_r" {
                            unwrap_in_r = true;
                        } else if ident == "serialize" {
                            serialize = true;
                        } else if ident == "serde_error" {
                            return Err(syn::Error::new_spanned(&path, SERDE_ERROR_BARE_HELP));
                        } else if ident == "worker" {
                            force_worker = Some(true);
                        } else if ident == "no_worker" {
                            force_worker = Some(false);
                        } else if ident == "strict" {
                            strict = Some(true);
                        } else if ident == "no_strict" {
                            strict = Some(false);
                        } else if ident == "preconditions" {
                            preconditions = Some(true);
                        } else if ident == "no_preconditions" {
                            preconditions = Some(false);
                        } else if ident == "internal" {
                            internal = true;
                        } else if ident == "noexport" {
                            noexport = true;
                        } else if ident == "export" {
                            export = true;
                        } else {
                            return Err(syn::Error::new_spanned(
                                ident,
                                format!(
                                    "unknown `#[miniextendr]` option; expected one of: {FN_BOOL_FLAGS_HELP}"
                                ),
                            ));
                        }
                    }
                }
                syn::Meta::NameValue(nv) => {
                    if nv.path.is_ident("wrap") {
                        let value = parse_lit_str(&nv, "wrap")?;
                        wrap = Some(crate::return_wrap::parse_system(&syn::LitStr::new(
                            &value,
                            nv.path.span(),
                        ))?);
                        continue;
                    }
                    // Check for boolean flag options: option = true / option = false
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Bool(lit_bool),
                        ..
                    }) = &nv.value
                    {
                        let val = lit_bool.value;
                        if let Some(ident) = nv.path.get_ident() {
                            if ident == "invisible" {
                                force_invisible = Some(val);
                            } else if ident == "visible" {
                                force_invisible = Some(!val);
                            } else if ident == "check_interrupt" {
                                check_interrupt = val;
                            } else if ident == "worker" {
                                force_worker = Some(val);
                            } else if ident == "no_worker" {
                                force_worker = Some(!val);
                            } else if ident == "coerce" {
                                coerce_all = Some(val);
                            } else if ident == "no_coerce" {
                                coerce_all = Some(!val);
                            } else if ident == "rng" {
                                rng = val;
                            } else if ident == "unwrap_in_r" {
                                unwrap_in_r = val;
                            } else if ident == "serialize" {
                                serialize = val;
                            } else if ident == "serde_error" {
                                return Err(syn::Error::new_spanned(&nv, SERDE_ERROR_BARE_HELP));
                            } else if ident == "strict" {
                                strict = Some(val);
                            } else if ident == "no_strict" {
                                strict = Some(!val);
                            } else if ident == "preconditions" {
                                preconditions = Some(val);
                            } else if ident == "no_preconditions" {
                                preconditions = Some(!val);
                            } else if ident == "internal" {
                                internal = val;
                            } else if ident == "noexport" {
                                noexport = val;
                            } else if ident == "export" {
                                export = val;
                            } else {
                                return Err(syn::Error::new_spanned(
                                    ident,
                                    format!(
                                        "unknown `#[miniextendr]` option `{ident}`; expected one of: \
                                         {FN_BOOL_FLAGS_HELP}"
                                    ),
                                ));
                            }
                            continue;
                        }
                    }

                    if nv.path.is_ident("prefer") {
                        let v = parse_lit_str(&nv, "prefer")?;
                        return_pref_span = Some(nv.span());
                        return_pref = match v.as_str() {
                            "list" => ReturnPref::List,
                            "externalptr" => ReturnPref::ExternalPtr,
                            "vector" | "native" => ReturnPref::Native,
                            "auto" => ReturnPref::Auto,
                            _ => {
                                return Err(syn::Error::new_spanned(
                                    &nv.value,
                                    "prefer must be one of: auto, list, externalptr, vector/native",
                                ));
                            }
                        };
                    } else if nv.path.is_ident("dots") {
                        // dots = typed_list!(...) - capture the macro invocation
                        // Store span for error reporting
                        dots_span = Some(nv.path.span());
                        if let syn::Expr::Macro(expr_macro) = &nv.value {
                            if expr_macro.mac.path.is_ident("typed_list") {
                                // Capture the entire macro invocation as TokenStream
                                dots_spec = Some(quote::quote!(#expr_macro));
                            } else {
                                return Err(syn::Error::new_spanned(
                                    &expr_macro.mac.path,
                                    "dots expects `typed_list!(...)` macro",
                                ));
                            }
                        } else {
                            return Err(syn::Error::new_spanned(
                                &nv.value,
                                "dots expects `typed_list!(...)` macro",
                            ));
                        }
                    } else if nv.path.is_ident("lifecycle") {
                        // lifecycle = "stage"
                        if let Some(spec) = crate::lifecycle::parse_lifecycle_attr(
                            &syn::Meta::NameValue(nv.clone()),
                        )? {
                            lifecycle = Some(spec);
                        }
                    } else if nv.path.is_ident("doc") {
                        doc = Some(parse_lit_str(&nv, "doc")?);
                    } else if nv.path.is_ident("c_symbol") {
                        let val = parse_lit_str(&nv, "c_symbol")?;
                        if val.is_empty()
                            || (!val.starts_with(|c: char| c.is_ascii_alphabetic())
                                && !val.starts_with('_'))
                        {
                            return Err(syn::Error::new_spanned(
                                &nv.value,
                                "c_symbol must be a valid C identifier",
                            ));
                        }
                        if !val.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                            return Err(syn::Error::new_spanned(
                                &nv.value,
                                "c_symbol must be a valid C identifier (alphanumeric and underscore only)",
                            ));
                        }
                        c_symbol = Some(val);
                    } else if nv.path.is_ident("r_name") {
                        let val = parse_lit_str(&nv, "r_name")?;
                        if val.is_empty() {
                            return Err(syn::Error::new_spanned(
                                &nv.value,
                                "r_name must not be empty",
                            ));
                        }
                        r_name = Some(val);
                    } else if nv.path.is_ident("postfix") {
                        let val = parse_lit_str(&nv, "postfix")?;
                        validate_postfix(&val, &nv.value)?;
                        postfix = Some(val);
                    } else if nv.path.is_ident("call") {
                        let name = match &nv.value {
                            syn::Expr::Path(p) => p.path.get_ident().map(|i| i.to_string()),
                            syn::Expr::Lit(syn::ExprLit {
                                lit: syn::Lit::Str(s),
                                ..
                            }) => Some(s.value()),
                            _ => None,
                        };
                        let Some(attribution) = name
                            .as_deref()
                            .and_then(crate::r_wrapper_builder::CallAttribution::parse_name)
                        else {
                            return Err(syn::Error::new_spanned(
                                &nv.value,
                                "`call = ...` accepts `wrapper` (the call as written, the \
                                 default) or `caller` (attribute conditions to the wrapper's \
                                 caller)",
                            ));
                        };
                        if call_attr.is_some() {
                            return Err(syn::Error::new_spanned(
                                &nv,
                                "`call = ...` is set more than once",
                            ));
                        }
                        call_attr = Some(attribution);
                    } else if nv.path.is_ident("r_entry") {
                        r_entry = Some(parse_lit_str(&nv, "r_entry")?);
                    } else if nv.path.is_ident("r_post_checks") {
                        r_post_checks = Some(parse_lit_str(&nv, "r_post_checks")?);
                    } else if nv.path.is_ident("r_on_exit") {
                        // Short form: r_on_exit = "expr" → on.exit(expr, add = TRUE)
                        r_on_exit = Some(ROnExit {
                            expr: parse_lit_str(&nv, "r_on_exit")?,
                            add: true,
                            after: true,
                        });
                    } else {
                        let key_name = nv
                            .path
                            .get_ident()
                            .map(|i| i.to_string())
                            .unwrap_or_default();
                        return Err(syn::Error::new_spanned(
                            nv,
                            format!(
                                "unknown `#[miniextendr]` key-value option `{}`. \
                                 Key-value options are: `prefer = \"...\"`, `dots = typed_list!(...)`, \
                                 `lifecycle = \"...\"`, `doc = \"...\"`, `c_symbol = \"...\"`, \
                                 `r_name = \"...\"`, `postfix = \"...\"`, `call = wrapper | caller`, `r_entry = \"...\"`, \
                                 `r_post_checks = \"...\"`, \
                                 `r_on_exit = \"...\"`",
                                key_name,
                            ),
                        ));
                    }
                }
                syn::Meta::List(list) => {
                    if list.path.is_ident("defaults") {
                        // Ignore defaults(...) - it's handled by impl method parsing
                        // This allows #[miniextendr(defaults(...))] on impl methods
                    } else if list.path.is_ident("lifecycle") {
                        // lifecycle(stage = "deprecated", when = "0.4.0", ...)
                        if let Some(spec) =
                            crate::lifecycle::parse_lifecycle_attr(&syn::Meta::List(list.clone()))?
                        {
                            lifecycle = Some(spec);
                        }
                    } else if list.path.is_ident("s3") {
                        // Parse s3(generic = "...", class = "...")
                        list.parse_nested_meta(|meta| {
                            if meta.path.is_ident("generic") {
                                let _: syn::Token![=] = meta.input.parse()?;
                                let value: syn::LitStr = meta.input.parse()?;
                                s3_generic = Some(value.value());
                            } else if meta.path.is_ident("class") {
                                let _: syn::Token![=] = meta.input.parse()?;
                                let value: syn::LitStr = meta.input.parse()?;
                                s3_class = Some(value.value());
                            } else {
                                return Err(
                                    meta.error("unknown s3 option; expected `generic` or `class`")
                                );
                            }
                            Ok(())
                        })?;
                        // Validate: s3 requires class (generic can default to function name)
                        if s3_class.is_none() {
                            return Err(syn::Error::new_spanned(
                                &list,
                                "s3(...) requires `class = \"...\"` to specify the S3 class suffix; \
                                 `generic` is optional and defaults to the function name",
                            ));
                        }
                    } else if list.path.is_ident("r_on_exit") {
                        // Long form: r_on_exit(expr = "...", add = false, after = false)
                        let mut expr = None;
                        let mut add = true;
                        let mut after = true;
                        list.parse_nested_meta(|meta| {
                            if meta.path.is_ident("expr") {
                                let _: syn::Token![=] = meta.input.parse()?;
                                let value: syn::LitStr = meta.input.parse()?;
                                expr = Some(value.value());
                            } else if meta.path.is_ident("add") {
                                let _: syn::Token![=] = meta.input.parse()?;
                                let value: syn::LitBool = meta.input.parse()?;
                                add = value.value;
                            } else if meta.path.is_ident("after") {
                                let _: syn::Token![=] = meta.input.parse()?;
                                let value: syn::LitBool = meta.input.parse()?;
                                after = value.value;
                            } else {
                                return Err(meta.error(
                                    "unknown r_on_exit option; expected `expr`, `add`, or `after`",
                                ));
                            }
                            Ok(())
                        })?;
                        let expr = expr.ok_or_else(|| {
                            syn::Error::new_spanned(
                                &list,
                                "r_on_exit(...) requires `expr = \"...\"` specifying the R expression",
                            )
                        })?;
                        r_on_exit = Some(ROnExit { expr, add, after });
                    } else if list.path.is_ident("serde_error") {
                        serde_error = Some(parse_serde_error_list(&list)?);
                    } else if let Some(ident) = list.path.get_ident() {
                        // Bool-flag parenthesized form (e.g. `strict(true)`) is not
                        // supported — write `strict` alone or `strict = true` instead.
                        let opt_name = ident.to_string();
                        return Err(syn::Error::new_spanned(
                            &list,
                            format!(
                                "`{opt_name}` does not accept parenthesized arguments. \
                                 Use `{opt_name}` alone or `{opt_name} = true/false`.",
                            ),
                        ));
                    } else {
                        // path(something) where path is not a single ident
                        return Err(syn::Error::new_spanned(
                            list,
                            format!(
                                "unrecognized nested option. Nested options are: {FN_NESTED_OPTIONS_HELP}"
                            ),
                        ));
                    }
                }
            }
        }

        // Validate: `internal` and `noexport` are redundant together
        if internal && noexport {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "`internal` and `noexport` cannot be used together. \
                 `internal` already suppresses @export and also adds @keywords internal. \
                 Use `internal` alone to mark as internal, or `noexport` alone to only suppress export.",
            ));
        }

        // Validate: `export` conflicts with `noexport` and `internal`
        if export && noexport {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "`export` and `noexport` are contradictory.",
            ));
        }
        if export && internal {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "`export` and `internal` are contradictory.",
            ));
        }

        // Validate: `r_name` is incompatible with S3 naming (`s3(generic/class)`)
        // Validate: `postfix` derives the wrapper name from the Rust name; it
        // cannot combine with another naming source.
        if postfix.is_some() && r_name.is_some() {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "`postfix` and `r_name` both set the R wrapper name; use one of them.",
            ));
        }
        if postfix.is_some() && (s3_generic.is_some() || s3_class.is_some()) {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "`postfix` cannot be used with `s3(generic = ..., class = ...)`. \
                 S3 method names are always `generic.class`.",
            ));
        }

        // Validate: `call = caller` is for internal entry points only.
        use crate::r_wrapper_builder::CallAttribution;
        if call_attr == Some(CallAttribution::Caller) && !(noexport || internal) {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "`call = caller` attributes conditions to the wrapper's caller, which is only \
                 meaningful for a package-internal entry point; add `noexport` or `internal`.",
            ));
        }
        if r_name.is_some() && (s3_generic.is_some() || s3_class.is_some()) {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "`r_name` cannot be used with `s3(generic = ..., class = ...)`. \
                 S3 method names are always `generic.class`. Use `generic` and `class` instead.",
            ));
        }

        if serialize && (unwrap_in_r || serde_error.is_some()) {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "`serialize` cannot be combined with `unwrap_in_r` or `serde_error`: it serializes the complete return value, including any Result variant",
            ));
        }

        // Validate: `serde_error` classes the raised condition; `unwrap_in_r`
        // never raises (the Result is returned as a value), so combining them
        // is a contradiction.
        if serde_error.is_some() && unwrap_in_r {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "`serde_error` cannot be used with `unwrap_in_r`: `unwrap_in_r` returns the \
                 `Result` to R as a value, so there is no raised condition to class.",
            ));
        }

        Ok(Self {
            force_worker: force_worker.unwrap_or(cfg!(feature = "worker-default")),
            force_invisible,
            check_interrupt,
            coerce_all: coerce_all.unwrap_or(cfg!(feature = "coerce-default")),
            rng,
            unwrap_in_r,
            serialize,
            wrap,
            serde_error,
            preconditions,
            call_attribution: call_attr,
            return_pref,
            return_pref_span,
            s3_generic,
            s3_class,
            dots_spec,
            dots_span,
            lifecycle,
            strict: strict.unwrap_or(cfg!(feature = "strict-default")),
            internal,
            noexport,
            export,
            doc,
            c_symbol,
            r_name,
            postfix,
            r_entry,
            r_post_checks,
            r_on_exit,
        })
    }
}
// endregion
