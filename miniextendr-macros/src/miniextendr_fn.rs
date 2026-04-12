//! Function signature parsing for `#[miniextendr]`.
//!
//! This module handles parsing and normalizing Rust function signatures for the
//! `#[miniextendr]` attribute macro. It provides:
//!
//! - [`MiniextendrFunctionParsed`]: Parsed function with normalization and codegen helpers
//! - [`MiniextendrFnAttrs`]: Parsed `#[miniextendr(...)]` attribute options
//! - [`CoercionMapping`]: Type coercion analysis for automatic R→Rust conversion

use crate::{call_method_def_ident_for, r_wrapper_const_ident_for};

// region: Coercion analysis

/// Result of coercion analysis for a type.
/// Contains the R native type to extract from SEXP and the target type to coerce to.
pub(crate) enum CoercionMapping {
    /// Scalar coercion: extract R native type, coerce to target.
    Scalar {
        /// The R-native scalar type to extract from the SEXP (e.g., `i32` for R integers,
        /// `f64` for R reals). This is the type that R stores internally.
        r_native: proc_macro2::TokenStream,
        /// The Rust target type to coerce into (e.g., `u16`, `bool`, `f32`).
        target: proc_macro2::TokenStream,
    },
    /// Vec coercion: extract R native slice, coerce element-wise to `Vec<target>`.
    Vec {
        /// The R-native element type of the source slice (e.g., `i32` for integer vectors,
        /// `f64` for real vectors).
        r_native_elem: proc_macro2::TokenStream,
        /// The Rust target element type for the resulting `Vec` (e.g., `u16`, `bool`, `f32`).
        target_elem: proc_macro2::TokenStream,
    },
}

impl CoercionMapping {
    /// Determines the coercion mapping for a Rust type, if it needs coercion from
    /// an R-native type.
    ///
    /// Returns `None` if the type is already R-native (`i32`, `f64`, `String`, etc.)
    /// or is not a recognized coercible type.
    ///
    /// # Recognized coercions
    ///
    /// - **Scalar integer-like** (`u16`, `i16`, `i8`, `u32`, `u64`, `i64`, `isize`, `usize`):
    ///   coerced from `i32` (R's native integer type).
    /// - **Scalar `bool`**: coerced from `i32` (R's logical vectors use `i32` internally).
    /// - **Scalar `f32`**: coerced from `f64` (R's native real type).
    /// - **`Vec<T>`** variants: element-wise coercion from the corresponding R-native slice type.
    pub(crate) fn from_type(ty: &syn::Type) -> Option<Self> {
        match ty {
            syn::Type::Path(type_path) => {
                let seg = type_path.path.segments.last()?;
                let type_name = seg.ident.to_string();

                // Check for Vec<T> types
                if type_name == "Vec" {
                    if let syn::PathArguments::AngleBracketed(args) = &seg.arguments
                        && let Some(syn::GenericArgument::Type(syn::Type::Path(inner_path))) =
                            args.args.first()
                    {
                        let inner_name = inner_path.path.segments.last()?.ident.to_string();
                        return match inner_name.as_str() {
                            // Vec<integer-like> from &[i32]
                            "u16" | "i16" | "i8" | "u32" | "u64" | "i64" | "isize" | "usize" => {
                                let target_elem: proc_macro2::TokenStream =
                                    inner_name.parse().ok()?;
                                Some(Self::Vec {
                                    r_native_elem: quote::quote!(i32),
                                    target_elem,
                                })
                            }
                            // Vec<bool> from &[i32] (R logical vectors use i32)
                            "bool" => Some(Self::Vec {
                                r_native_elem: quote::quote!(i32),
                                target_elem: quote::quote!(bool),
                            }),
                            // Vec<f32> from &[f64]
                            "f32" => Some(Self::Vec {
                                r_native_elem: quote::quote!(f64),
                                target_elem: quote::quote!(f32),
                            }),
                            _ => None,
                        };
                    }
                    return None;
                }

                // Check for scalar types
                match type_name.as_str() {
                    // Integer-like types from i32
                    "u16" | "i16" | "i8" | "u32" | "u64" | "i64" | "isize" | "usize" => {
                        let target: proc_macro2::TokenStream = type_name.parse().ok()?;
                        Some(Self::Scalar {
                            r_native: quote::quote!(i32),
                            target,
                        })
                    }
                    // bool from i32 (R logical vectors use i32 internally)
                    "bool" => Some(Self::Scalar {
                        r_native: quote::quote!(i32),
                        target: quote::quote!(bool),
                    }),
                    // f32 from f64
                    "f32" => Some(Self::Scalar {
                        r_native: quote::quote!(f64),
                        target: quote::quote!(f32),
                    }),
                    // R-native types or unknown - no coercion
                    _ => None,
                }
            }
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

/// Check if a type is `Missing<T>`.
pub(crate) fn is_missing_type(ty: &syn::Type) -> bool {
    type_ends_with(ty, "Missing")
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
/// - `default` on a `&Dots` parameter
pub(crate) fn validate_per_param_attr_conflicts(
    attr: &PerParamMiniextendrAttr,
    param_name: &str,
    is_dots: bool,
    ty: Option<&syn::Type>,
    span: proc_macro2::Span,
) -> syn::Result<()> {
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

// endregion

// region: Per-parameter attribute parsing

/// Parsed per-parameter `#[miniextendr(...)]` attribute content.
///
/// A single attribute can contain multiple items, e.g.
/// `#[miniextendr(match_arg, default = "Safe")]`.
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
}

/// Parse all per-parameter options from a `#[miniextendr(...)]` attribute.
///
/// Handles mixed content like `#[miniextendr(match_arg, default = "\"Safe\"")]`
/// and `#[miniextendr(choices("a", "b", "c"))]`.
///
/// Returns `None` if `attr` is not a `#[miniextendr(...)]` attribute, if it cannot
/// be parsed, or if it contains only function-level options (like `strict`) with
/// no per-parameter options.
///
/// # Arguments
///
/// * `attr` - A `syn::Attribute` to inspect. Only attributes with path `miniextendr`
///   are considered.
pub(crate) fn parse_per_param_attr(attr: &syn::Attribute) -> Option<PerParamMiniextendrAttr> {
    use syn::spanned::Spanned;
    if !attr.path().is_ident("miniextendr") {
        return None;
    }

    let syn::Meta::List(meta_list) = &attr.meta else {
        return None;
    };

    let mut result = PerParamMiniextendrAttr::default();
    let mut is_per_param = false;

    let metas = match meta_list
        .parse_args_with(syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated)
    {
        Ok(m) => m,
        Err(_) => return None,
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
                }
                // Other list forms are function-level, ignore here
            }
        }
    }

    if !is_per_param {
        return None;
    }
    Some(result)
}

/// Returns `true` if `attr` is a `#[miniextendr(...)]` attribute containing `coerce`.
///
/// The `coerce` flag may be combined with other per-parameter options (e.g.,
/// `#[miniextendr(coerce, default = "0")]`).
pub(crate) fn is_miniextendr_coerce_attr(attr: &syn::Attribute) -> bool {
    parse_per_param_attr(attr).is_some_and(|a| a.has_coerce)
}

/// Returns `true` if `attr` is a `#[miniextendr(...)]` attribute containing `match_arg`.
///
/// The `match_arg` flag may be combined with other per-parameter options (e.g.,
/// `#[miniextendr(match_arg, choices("a", "b"))]`).
pub(crate) fn is_miniextendr_match_arg_attr(attr: &syn::Attribute) -> bool {
    parse_per_param_attr(attr).is_some_and(|a| a.has_match_arg)
}

/// Returns `true` if `attr` is a `#[miniextendr(...)]` attribute containing `choices(...)`.
///
/// The `choices(...)` option may be combined with other per-parameter options (e.g.,
/// `#[miniextendr(match_arg, choices("a", "b"))]`).
pub(crate) fn is_miniextendr_choices_attr(attr: &syn::Attribute) -> bool {
    parse_per_param_attr(attr).is_some_and(|a| a.choices.is_some())
}

/// Extracts the list of choice strings from a `#[miniextendr(choices("a", "b", "c"))]` attribute.
///
/// Returns `None` if the attribute does not contain `choices(...)` or is not a
/// `#[miniextendr(...)]` attribute.
pub(crate) fn parse_choices_attr(attr: &syn::Attribute) -> Option<Vec<String>> {
    parse_per_param_attr(attr).and_then(|a| a.choices)
}

/// Extracts the default value from a `#[miniextendr(default = "...")]` attribute.
///
/// Returns `Some((default_value, attr_span))` if the attribute contains a `default` option.
/// The span is used for error reporting when the default references a non-existent parameter.
pub(crate) fn parse_default_attr(attr: &syn::Attribute) -> Option<(String, proc_macro2::Span)> {
    parse_per_param_attr(attr).and_then(|a| a.default_value)
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
    item: syn::ItemFn,
    /// Whether the original function had `...` (variadic).
    has_dots: bool,
    /// If dots were named (e.g., `my_dots: ...`), the identifier.
    named_dots: Option<syn::Ident>,
    /// Parameter names that had `#[miniextendr(coerce)]` attribute.
    per_param_coerce: std::collections::HashSet<String>,
    /// Parameter names that had `#[miniextendr(match_arg)]` attribute.
    per_param_match_arg: std::collections::HashSet<String>,
    /// Parameter names with `#[miniextendr(default = "...")]` and their default values.
    per_param_defaults: std::collections::HashMap<String, String>,
    /// Parameter names with `#[miniextendr(choices("a", "b", "c"))]` and their choices.
    per_param_choices: std::collections::HashMap<String, Vec<String>>,
}

/// Parses a Rust `fn` item from a token stream, performing all normalizations
/// required by the `#[miniextendr]` codegen pipeline.
///
/// # Normalizations performed
///
/// 1. **Variadic (`...`) rewriting**: Replaces Rust variadic syntax with a typed
///    `&miniextendr_api::dots::Dots` parameter. Named dots (`my_dots: ...`) preserve
///    the user's identifier; unnamed `...` becomes `__miniextendr_dots`.
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

        // dots support: parse variadic name (if any) and replace `...` with `&Dots`.
        let has_dots = item.sig.variadic.is_some();
        let named_dots = if has_dots {
            let dots = item.sig.variadic.as_ref().unwrap();
            if let Some(named_dots) = dots.pat.as_ref() {
                if let syn::Pat::Ident(named_dots_ident) = named_dots.0.as_ref() {
                    Some(named_dots_ident.ident.clone())
                } else {
                    return Err(syn::Error::new(
                        named_dots.0.span(),
                        "variadic pattern must be a simple identifier (e.g. `dots: ...`) or unnamed `...`",
                    ));
                }
            } else {
                None
            }
        } else {
            None
        };

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
        let mut per_param_coerce: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        let mut per_param_match_arg: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        let mut per_param_defaults: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        let mut per_param_default_spans: std::collections::HashMap<String, proc_macro2::Span> =
            std::collections::HashMap::new();
        let mut per_param_choices: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();
        let mut unused_counter = 0usize;
        let mut pattern_destructures: Vec<(Box<syn::Pat>, syn::Ident)> = Vec::new();
        for arg in &mut item.sig.inputs {
            let syn::FnArg::Typed(pat_type) = arg else {
                // Self parameters are not allowed in standalone functions.
                // Users should use #[miniextendr(env|r6|s3|s4|s7)] on impl blocks instead.
                // The error is raised in lib.rs c_wrapper_inputs generation.
                continue;
            };

            let had_coerce_attr = pat_type.attrs.iter().any(is_miniextendr_coerce_attr);
            let had_match_arg_attr = pat_type.attrs.iter().any(is_miniextendr_match_arg_attr);
            let default_with_span = pat_type.attrs.iter().find_map(parse_default_attr);
            let had_choices = pat_type.attrs.iter().find_map(parse_choices_attr);

            // Remove miniextendr attributes from parameters (coerce, match_arg, choices, default)
            pat_type.attrs.retain(|attr| {
                !is_miniextendr_coerce_attr(attr)
                    && !is_miniextendr_match_arg_attr(attr)
                    && !is_miniextendr_choices_attr(attr)
                    && parse_default_attr(attr).is_none()
            });

            // Validate type-based constraints (Missing nesting, Missing<Dots>)
            validate_param_type(pat_type.ty.as_ref(), pat_type.ty.span())?;

            let param_name_for_validation: String;
            match pat_type.pat.as_ref() {
                syn::Pat::Ident(pat_ident) => {
                    let param_name = pat_ident.ident.to_string();
                    param_name_for_validation = param_name.clone();
                    if had_coerce_attr {
                        per_param_coerce.insert(param_name.clone());
                    }
                    if had_match_arg_attr {
                        per_param_match_arg.insert(param_name.clone());
                    }
                    if let Some(choices) = had_choices.clone() {
                        per_param_choices.insert(param_name.clone(), choices);
                    }
                    if let Some((default, span)) = default_with_span.clone() {
                        per_param_defaults.insert(param_name.clone(), default);
                        per_param_default_spans.insert(param_name, span);
                    }
                }
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
                    param_name_for_validation = synthetic_name.clone();
                    if had_coerce_attr {
                        per_param_coerce.insert(synthetic_name.clone());
                    }
                    if had_match_arg_attr {
                        per_param_match_arg.insert(synthetic_name.clone());
                    }
                    if let Some(choices) = had_choices.clone() {
                        per_param_choices.insert(synthetic_name.clone(), choices);
                    }
                    if let Some((default, span)) = default_with_span.clone() {
                        per_param_defaults.insert(synthetic_name.clone(), default);
                        per_param_default_spans.insert(synthetic_name, span);
                    }
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
                    pattern_destructures.push((original_pat, synthetic_ident.clone()));
                    param_name_for_validation = synthetic_name.clone();
                    if had_coerce_attr {
                        per_param_coerce.insert(synthetic_name.clone());
                    }
                    if had_match_arg_attr {
                        per_param_match_arg.insert(synthetic_name.clone());
                    }
                    if let Some(choices) = had_choices.clone() {
                        per_param_choices.insert(synthetic_name.clone(), choices);
                    }
                    if let Some((default, span)) = default_with_span.clone() {
                        per_param_defaults.insert(synthetic_name.clone(), default);
                        per_param_default_spans.insert(synthetic_name, span);
                    }
                }
                _ => {
                    return Err(syn::Error::new(
                        pat_type.pat.span(),
                        "miniextendr parameters must be identifiers or destructuring patterns (tuple, struct)",
                    ));
                }
            }

            // Validate per-parameter attribute conflicts (coerce+match_arg, coerce+choices, etc.)
            let per_param_combined = PerParamMiniextendrAttr {
                has_coerce: had_coerce_attr,
                has_match_arg: had_match_arg_attr,
                default_value: default_with_span,
                choices: had_choices,
            };
            validate_per_param_attr_conflicts(
                &per_param_combined,
                &param_name_for_validation,
                is_dots_type(pat_type.ty.as_ref()),
                Some(pat_type.ty.as_ref()),
                pat_type.ty.span(),
            )?;
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

        if has_dots {
            item.sig.variadic = None;
            item.sig
                .inputs
                .push(if let Some(named_dots) = named_dots.as_ref() {
                    syn::parse_quote!(#named_dots: &::miniextendr_api::dots::Dots)
                } else {
                    // cannot use `_` as variable name, thus cannot use it as a placeholder for `...`
                    // Check that no existing parameter is named `__miniextendr_dots`
                    for arg in &item.sig.inputs {
                        let syn::FnArg::Typed(pat_type) = arg else {
                            continue;
                        };
                        if let syn::Pat::Ident(pat_ident) = pat_type.pat.as_ref()
                            && pat_ident.ident == "__miniextendr_dots" {
                                return Err(syn::Error::new(
                                    pat_ident.ident.span(),
                                    "parameter named `__miniextendr_dots` conflicts with implicit dots parameter; use named dots like `my_dots: ...` instead",
                                ));
                            }
                    }
                    syn::parse_quote!(__miniextendr_dots: &::miniextendr_api::dots::Dots)
                });
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
                    Some(pat_ident.ident.to_string())
                } else {
                    None
                }
            })
            .collect();

        let mut invalid_params: Vec<String> = per_param_defaults
            .keys()
            .filter(|key| !param_names.contains(*key))
            .cloned()
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

        Ok(Self {
            item,
            has_dots,
            named_dots,
            per_param_coerce,
            per_param_match_arg,
            per_param_defaults,
            per_param_choices,
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

    /// Whether the original function had `...` (variadic).
    pub(crate) fn has_dots(&self) -> bool {
        self.has_dots
    }

    /// If dots were named (e.g., `my_dots: ...`), returns the identifier.
    pub(crate) fn named_dots(&self) -> Option<&syn::Ident> {
        self.named_dots.as_ref()
    }

    /// Check if a parameter is the dots (`...`) param.
    /// After parsing, dots are rewritten to `&Dots` — this checks the original name.
    pub(crate) fn is_dots_param(&self, ident: &syn::Ident) -> bool {
        if !self.has_dots {
            return false;
        }
        // Named dots: check if ident matches the original name (e.g., `dots`, `my_dots`)
        if let Some(ref named) = self.named_dots {
            return ident == named;
        }
        // Unnamed dots: the variadic was replaced with `_dots` as the param name
        ident == "_dots"
    }

    /// Check if a parameter name had `#[miniextendr(coerce)]` attribute.
    pub(crate) fn has_coerce_attr(&self, param_name: &str) -> bool {
        self.per_param_coerce.contains(param_name)
    }

    /// Check if a parameter name had `#[miniextendr(match_arg)]` attribute.
    pub(crate) fn has_match_arg_attr(&self, param_name: &str) -> bool {
        self.per_param_match_arg.contains(param_name)
    }

    /// Returns the set of parameter names annotated with `#[miniextendr(match_arg)]`.
    ///
    /// Used by the R wrapper generator to emit `match.arg()` calls for these parameters.
    pub(crate) fn match_arg_params(&self) -> &std::collections::HashSet<String> {
        &self.per_param_match_arg
    }

    /// Get the choices for a parameter, if any.
    pub(crate) fn choices_for_param(&self, param_name: &str) -> Option<&[String]> {
        self.per_param_choices.get(param_name).map(|v| v.as_slice())
    }

    /// Returns the full choices map (parameter name to list of allowed string values).
    ///
    /// Used by the R wrapper generator to emit `match.arg()` with an explicit choices vector.
    pub(crate) fn choices_params(&self) -> &std::collections::HashMap<String, Vec<String>> {
        &self.per_param_choices
    }

    /// Returns all parameter defaults as a map from parameter name to default value string.
    ///
    /// The default value string is the raw R expression that will be placed in the
    /// R wrapper's formals (e.g., `"NULL"`, `"TRUE"`, `"\"Safe\""`).
    pub(crate) fn param_defaults(&self) -> &std::collections::HashMap<String, String> {
        &self.per_param_defaults
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

    /// Function inputs after normalization (dots rewritten, wildcards renamed).
    pub(crate) fn inputs(&self) -> &syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]> {
        &self.item.sig.inputs
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

    /// Returns `true` if this function needs an internal C wrapper (`C_<name>` function).
    ///
    /// Rust-ABI functions (no explicit `extern`) need a generated `extern "C-unwind"` wrapper
    /// that handles SEXP conversion and error propagation. Functions already declared as
    /// `extern "C-unwind"` are passed through directly without wrapping.
    pub(crate) fn uses_internal_c_wrapper(&self) -> bool {
        self.abi().is_none()
    }

    /// Returns the identifier for the generated `const R_CallMethodDef` value.
    ///
    /// This constant is automatically registered with R's `.Call` interface
    /// via linkme distributed slices.
    pub(crate) fn call_method_def_ident(&self) -> syn::Ident {
        call_method_def_ident_for(self.ident())
    }

    /// Returns the identifier for the generated `const &str` holding the R wrapper code.
    ///
    /// The R wrapper is a string constant containing the R function definition that
    /// calls `.Call(C_<name>, ...)`. It is collected via linkme distributed slices to
    /// produce the `R/miniextendr_wrappers.R` file.
    pub(crate) fn r_wrapper_const_ident(&self) -> syn::Ident {
        r_wrapper_const_ident_for(self.ident())
    }

    /// Returns the identifier for the C-callable entry point.
    ///
    /// - **Rust ABI functions**: Returns `C_<name>` (the generated wrapper function).
    /// - **`extern "C-unwind"` functions**: Returns the function's own name, or the
    ///   value from `#[export_name = "..."]` if present.
    pub(crate) fn c_wrapper_ident(&self) -> syn::Ident {
        if self.uses_internal_c_wrapper() {
            quote::format_ident!("C_{}", self.ident())
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

    /// Add `#[track_caller]` if not already present (for better panic locations).
    /// Only for Rust ABI functions - extern "C-unwind" doesn't support track_caller.
    pub(crate) fn add_track_caller_if_needed(&mut self) {
        let has_explicit_abi = self.item.sig.abi.is_some();
        let has_track_caller = self
            .item
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("track_caller"));
        if !has_track_caller && !has_explicit_abi {
            self.item.attrs.push(syn::parse_quote!(#[track_caller]));
        }
    }

    /// Add `#[inline(never)]` if no `#[inline(...)]` attribute is present.
    /// Only for Rust ABI functions - extern "C-unwind" functions are passed through as-is.
    ///
    /// Preventing inlining ensures:
    /// - The worker thread pattern works correctly (function runs in separate context)
    /// - Panic handling and unwinding work as expected
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
/// - `unsafe(main_thread)`: force execution on R's main thread (unsafe: panics will leak resources)
/// - `worker`: explicitly request worker thread execution (default for most functions)
/// - `coerce`: enable automatic coercion for supported parameter types
/// - `rng`: enable RNG state management (GetRNGstate/PutRNGstate)
/// - `unwrap_in_r`: return `Result<T, E>` to R without unwrapping
/// - `prefer = "auto" | "list" | "externalptr" | "vector"`: prefer a specific `IntoR` path
///
/// # Note
///
/// Unknown flags are rejected with a compile error to avoid silently ignoring typos.
#[derive(Default)]
pub(crate) struct MiniextendrFnAttrs {
    /// Force execution on R's main thread (set by `unsafe(main_thread)`).
    pub(crate) force_main_thread: bool,
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
    /// Preferred return conversion.
    pub(crate) return_pref: ReturnPref,
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
    /// Transport Rust-origin errors as tagged values; R wrapper raises condition.
    pub(crate) error_in_r: bool,
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
    /// Run in background thread. Returns `MxAsyncHandle` immediately to R.
    /// Generates `$is_resolved()` and `$value()` methods on the R side.
    /// Mutually exclusive with `worker` and `unsafe(main_thread)`.
    pub(crate) background: bool,
    /// Custom C symbol name for the generated wrapper.
    ///
    /// Overrides the default `C_<fn_name>` naming convention.
    /// Must be a valid C identifier (alphanumeric + underscore, starting with letter or underscore).
    pub(crate) c_symbol: Option<String>,
    /// Override R wrapper function name.
    ///
    /// Use `#[miniextendr(r_name = "is.my_type")]` to give the R wrapper a different name
    /// than the Rust function. The C symbol is still derived from the Rust name.
    /// Cannot be combined with `s3(generic/class)` — use `generic`/`class` for S3 naming.
    pub(crate) r_name: Option<String>,
    /// R code to inject at the very top of the wrapper body (before all built-in checks).
    ///
    /// Use `#[miniextendr(r_entry = "x <- as.integer(x)")]` to run R code before
    /// missing-default handling, lifecycle checks, stopifnot, and match.arg.
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
/// - **Nested list**: `#[miniextendr(unsafe(main_thread))]`, `#[miniextendr(s3(generic = "...", class = "..."))]`
///
/// Options with negated forms (`no_worker`, `no_coerce`, `no_strict`, `no_error_in_r`)
/// explicitly disable the corresponding flag, which is useful for overriding
/// feature-based defaults.
///
/// An empty input (plain `#[miniextendr]`) resolves all options to their feature-based
/// defaults (e.g., `default-worker`, `default-coerce`, `default-strict`).
///
/// # Errors
///
/// Returns a compile error for:
/// - Unknown option names (prevents silent typos)
/// - Mutually exclusive options (`error_in_r` + `unwrap_in_r`, `internal` + `noexport`)
/// - Invalid values for key-value options (e.g., bad `prefer` or `c_symbol`)
/// - Missing required sub-options (e.g., `s3(...)` without `class`)
impl syn::parse::Parse for MiniextendrFnAttrs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        use syn::spanned::Spanned;
        // Use Option<bool> for fields that support feature defaults.
        // None = not explicitly set → resolve from cfg!(feature = "...") at end.
        let mut force_main_thread: Option<bool> = None;
        let mut force_worker: Option<bool> = None;
        let mut force_invisible: Option<bool> = None;
        let mut check_interrupt = false;
        let mut coerce_all: Option<bool> = None;
        let mut rng = false;
        let mut unwrap_in_r = false;
        let mut return_pref = ReturnPref::Auto;
        let mut s3_generic = None;
        let mut s3_class = None;
        let mut dots_spec = None;
        let mut dots_span = None;
        let mut lifecycle = None;
        let mut strict: Option<bool> = None;
        let mut error_in_r: Option<bool> = None;
        let mut internal = false;
        let mut noexport = false;
        let mut export = false;
        let mut doc = None;
        let mut background = false;
        let mut c_symbol = None;
        let mut r_name = None;
        let mut r_entry = None;
        let mut r_post_checks = None;
        let mut r_on_exit = None;

        if input.is_empty() {
            return Ok(Self {
                force_main_thread: force_main_thread.unwrap_or(true),
                force_worker: force_worker.unwrap_or(cfg!(feature = "default-worker")),
                force_invisible,
                check_interrupt,
                coerce_all: coerce_all.unwrap_or(cfg!(feature = "default-coerce")),
                rng,
                unwrap_in_r,
                return_pref,
                s3_generic,
                s3_class,
                dots_spec,
                dots_span,
                lifecycle,
                strict: strict.unwrap_or(cfg!(feature = "default-strict")),
                error_in_r: error_in_r.unwrap_or(true),
                internal,
                noexport,
                export,
                doc,
                background,
                c_symbol,
                r_name,
                r_entry,
                r_post_checks,
                r_on_exit,
            });
        }

        let metas =
            syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated(input)?;

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
                            if error_in_r == Some(true) {
                                return Err(syn::Error::new_spanned(
                                    ident,
                                    "`error_in_r` and `unwrap_in_r` are mutually exclusive",
                                ));
                            }
                            unwrap_in_r = true;
                        } else if ident == "worker" {
                            force_worker = Some(true);
                        } else if ident == "no_worker" {
                            force_worker = Some(false);
                        } else if ident == "strict" {
                            strict = Some(true);
                        } else if ident == "no_strict" {
                            strict = Some(false);
                        } else if ident == "error_in_r" {
                            if unwrap_in_r {
                                return Err(syn::Error::new_spanned(
                                    ident,
                                    "`error_in_r` and `unwrap_in_r` are mutually exclusive",
                                ));
                            }
                            error_in_r = Some(true);
                        } else if ident == "no_error_in_r" {
                            error_in_r = Some(false);
                        } else if ident == "internal" {
                            internal = true;
                        } else if ident == "noexport" {
                            noexport = true;
                        } else if ident == "export" {
                            export = true;
                        } else if ident == "background" {
                            if force_worker == Some(true) {
                                return Err(syn::Error::new_spanned(
                                    ident,
                                    "`background` and `worker` are mutually exclusive",
                                ));
                            }
                            background = true;
                        } else {
                            return Err(syn::Error::new_spanned(
                                ident,
                                "unknown `#[miniextendr]` option; expected one of: invisible, visible, check_interrupt, unsafe(main_thread), worker, no_worker, coerce, no_coerce, rng, unwrap_in_r, error_in_r, no_error_in_r, strict, no_strict, internal, noexport, export, background",
                            ));
                        }
                    }
                }
                syn::Meta::NameValue(nv) => {
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
                                if val && error_in_r == Some(true) {
                                    return Err(syn::Error::new_spanned(
                                        ident,
                                        "`error_in_r` and `unwrap_in_r` are mutually exclusive",
                                    ));
                                }
                                unwrap_in_r = val;
                            } else if ident == "strict" {
                                strict = Some(val);
                            } else if ident == "no_strict" {
                                strict = Some(!val);
                            } else if ident == "error_in_r" {
                                if val && unwrap_in_r {
                                    return Err(syn::Error::new_spanned(
                                        ident,
                                        "`error_in_r` and `unwrap_in_r` are mutually exclusive",
                                    ));
                                }
                                error_in_r = Some(val);
                            } else if ident == "no_error_in_r" {
                                error_in_r = Some(!val);
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
                                        "unknown `#[miniextendr]` option `{}`; expected one of: \
                                         invisible, visible, check_interrupt, unsafe(main_thread), \
                                         worker, no_worker, coerce, no_coerce, rng, unwrap_in_r, \
                                         error_in_r, no_error_in_r, strict, no_strict, internal, noexport, export",
                                        ident,
                                    ),
                                ));
                            }
                            continue;
                        }
                    }

                    if nv.path.is_ident("prefer") {
                        match &nv.value {
                            syn::Expr::Lit(expr_lit) => {
                                if let syn::Lit::Str(lit) = &expr_lit.lit {
                                    let v = lit.value();
                                    return_pref = match v.as_str() {
                                        "list" => ReturnPref::List,
                                        "externalptr" => ReturnPref::ExternalPtr,
                                        "vector" | "native" => ReturnPref::Native,
                                        "auto" => ReturnPref::Auto,
                                        _ => {
                                            return Err(syn::Error::new_spanned(
                                                lit,
                                                "prefer must be one of: auto, list, externalptr, vector/native",
                                            ));
                                        }
                                    };
                                } else {
                                    return Err(syn::Error::new_spanned(
                                        &expr_lit.lit,
                                        "prefer expects a string literal",
                                    ));
                                }
                            }
                            other => {
                                return Err(syn::Error::new_spanned(
                                    other,
                                    "prefer expects a string literal",
                                ));
                            }
                        }
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
                        // doc = "custom roxygen documentation"
                        match &nv.value {
                            syn::Expr::Lit(expr_lit) => {
                                if let syn::Lit::Str(lit) = &expr_lit.lit {
                                    doc = Some(lit.value());
                                } else {
                                    return Err(syn::Error::new_spanned(
                                        &expr_lit.lit,
                                        "doc expects a string literal",
                                    ));
                                }
                            }
                            other => {
                                return Err(syn::Error::new_spanned(
                                    other,
                                    "doc expects a string literal",
                                ));
                            }
                        }
                    } else if nv.path.is_ident("c_symbol") {
                        // c_symbol = "custom_C_name"
                        match &nv.value {
                            syn::Expr::Lit(expr_lit) => {
                                if let syn::Lit::Str(lit) = &expr_lit.lit {
                                    let val = lit.value();
                                    if val.is_empty()
                                        || (!val.starts_with(|c: char| c.is_ascii_alphabetic())
                                            && !val.starts_with('_'))
                                    {
                                        return Err(syn::Error::new_spanned(
                                            lit,
                                            "c_symbol must be a valid C identifier",
                                        ));
                                    }
                                    if !val.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                                        return Err(syn::Error::new_spanned(
                                            lit,
                                            "c_symbol must be a valid C identifier (alphanumeric and underscore only)",
                                        ));
                                    }
                                    c_symbol = Some(val);
                                } else {
                                    return Err(syn::Error::new_spanned(
                                        &expr_lit.lit,
                                        "c_symbol expects a string literal",
                                    ));
                                }
                            }
                            other => {
                                return Err(syn::Error::new_spanned(
                                    other,
                                    "c_symbol expects a string literal",
                                ));
                            }
                        }
                    } else if nv.path.is_ident("r_name") {
                        // r_name = "custom.r.name"
                        match &nv.value {
                            syn::Expr::Lit(expr_lit) => {
                                if let syn::Lit::Str(lit) = &expr_lit.lit {
                                    let val = lit.value();
                                    if val.is_empty() {
                                        return Err(syn::Error::new_spanned(
                                            lit,
                                            "r_name must not be empty",
                                        ));
                                    }
                                    r_name = Some(val);
                                } else {
                                    return Err(syn::Error::new_spanned(
                                        &expr_lit.lit,
                                        "r_name expects a string literal",
                                    ));
                                }
                            }
                            other => {
                                return Err(syn::Error::new_spanned(
                                    other,
                                    "r_name expects a string literal",
                                ));
                            }
                        }
                    } else if nv.path.is_ident("r_entry") {
                        // r_entry = "x <- as.integer(x)"
                        match &nv.value {
                            syn::Expr::Lit(expr_lit) => {
                                if let syn::Lit::Str(lit) = &expr_lit.lit {
                                    r_entry = Some(lit.value());
                                } else {
                                    return Err(syn::Error::new_spanned(
                                        &expr_lit.lit,
                                        "r_entry expects a string literal",
                                    ));
                                }
                            }
                            other => {
                                return Err(syn::Error::new_spanned(
                                    other,
                                    "r_entry expects a string literal",
                                ));
                            }
                        }
                    } else if nv.path.is_ident("r_post_checks") {
                        // r_post_checks = "message('validated')"
                        match &nv.value {
                            syn::Expr::Lit(expr_lit) => {
                                if let syn::Lit::Str(lit) = &expr_lit.lit {
                                    r_post_checks = Some(lit.value());
                                } else {
                                    return Err(syn::Error::new_spanned(
                                        &expr_lit.lit,
                                        "r_post_checks expects a string literal",
                                    ));
                                }
                            }
                            other => {
                                return Err(syn::Error::new_spanned(
                                    other,
                                    "r_post_checks expects a string literal",
                                ));
                            }
                        }
                    } else if nv.path.is_ident("r_on_exit") {
                        // Short form: r_on_exit = "expr" → on.exit(expr, add = TRUE)
                        match &nv.value {
                            syn::Expr::Lit(expr_lit) => {
                                if let syn::Lit::Str(lit) = &expr_lit.lit {
                                    r_on_exit = Some(ROnExit {
                                        expr: lit.value(),
                                        add: true,
                                        after: true,
                                    });
                                } else {
                                    return Err(syn::Error::new_spanned(
                                        &expr_lit.lit,
                                        "r_on_exit expects a string literal",
                                    ));
                                }
                            }
                            other => {
                                return Err(syn::Error::new_spanned(
                                    other,
                                    "r_on_exit expects a string literal",
                                ));
                            }
                        }
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
                                 `r_name = \"...\"`, `r_entry = \"...\"`, `r_post_checks = \"...\"`, \
                                 `r_on_exit = \"...\"`",
                                key_name,
                            ),
                        ));
                    }
                }
                // Nested: unsafe(main_thread)
                syn::Meta::List(list) => {
                    if list.path.is_ident("unsafe") {
                        let nested = list.parse_args_with(
                            syn::punctuated::Punctuated::<syn::Ident, syn::Token![,]>::parse_terminated,
                        )?;
                        if nested.is_empty() {
                            return Err(syn::Error::new_spanned(
                                list,
                                "`unsafe(...)` must specify an option: currently only `unsafe(main_thread)` is supported",
                            ));
                        }
                        for ident in nested {
                            if ident == "main_thread" {
                                force_main_thread = Some(true);
                            } else {
                                return Err(syn::Error::new_spanned(
                                    ident,
                                    "unknown `unsafe(...)` option; only `main_thread` is supported",
                                ));
                            }
                        }
                    } else if list.path.is_ident("defaults") {
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
                    } else if let Some(ident) = list.path.get_ident() {
                        // Try parsing as boolean: option(true) / option(false)
                        if let Ok(lit_bool) = list.parse_args::<syn::LitBool>() {
                            let val = lit_bool.value;
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
                                if val && error_in_r == Some(true) {
                                    return Err(syn::Error::new_spanned(
                                        ident,
                                        "`error_in_r` and `unwrap_in_r` are mutually exclusive",
                                    ));
                                }
                                unwrap_in_r = val;
                            } else if ident == "strict" {
                                strict = Some(val);
                            } else if ident == "no_strict" {
                                strict = Some(!val);
                            } else if ident == "error_in_r" {
                                if val && unwrap_in_r {
                                    return Err(syn::Error::new_spanned(
                                        ident,
                                        "`error_in_r` and `unwrap_in_r` are mutually exclusive",
                                    ));
                                }
                                error_in_r = Some(val);
                            } else if ident == "no_error_in_r" {
                                error_in_r = Some(!val);
                            } else if ident == "internal" {
                                internal = val;
                            } else if ident == "noexport" {
                                noexport = val;
                            } else if ident == "export" {
                                export = val;
                            } else {
                                let opt_name = ident.to_string();
                                return Err(syn::Error::new_spanned(
                                    &list,
                                    format!(
                                        "unknown `#[miniextendr]` option `{opt_name}`. Boolean flags should be \
                                         written as `option` (alone) or `option = true/false`. \
                                         Nested options: `unsafe(main_thread)`, `s3(...)`, `lifecycle(...)`, `defaults(...)`",
                                    ),
                                ));
                            }
                        } else {
                            let opt_name = list
                                .path
                                .get_ident()
                                .map(|i| i.to_string())
                                .unwrap_or_default();
                            return Err(syn::Error::new_spanned(
                                &list,
                                format!(
                                    "`{opt_name}` does not accept parenthesized arguments. \
                                     Use `{opt_name}` alone or `{opt_name} = true/false`.",
                                ),
                            ));
                        }
                    } else {
                        // path(something) where path is not a single ident
                        return Err(syn::Error::new_spanned(
                            list,
                            "unrecognized nested option. \
                             Nested options are: `unsafe(main_thread)`, `s3(...)`, `lifecycle(...)`, `defaults(...)`",
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
        if r_name.is_some() && (s3_generic.is_some() || s3_class.is_some()) {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "`r_name` cannot be used with `s3(generic = ..., class = ...)`. \
                 S3 method names are always `generic.class`. Use `generic` and `class` instead.",
            ));
        }

        // Resolve feature defaults for fields not explicitly set
        let resolved_error_in_r = error_in_r.unwrap_or(true);

        // Validate: rng requires error_in_r
        if rng && !resolved_error_in_r {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "`rng` requires `error_in_r` (PutRNGstate must run after .Call returns; \
                 non-error_in_r diverges via longjmp, skipping PutRNGstate)",
            ));
        }

        if resolved_error_in_r && unwrap_in_r {
            // This can happen when error_in_r is the default and unwrap_in_r is explicit
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "`error_in_r` (default) and `unwrap_in_r` are mutually exclusive; use `no_error_in_r` to opt out",
            ));
        }

        Ok(Self {
            force_main_thread: force_main_thread.unwrap_or(true),
            force_worker: force_worker.unwrap_or(cfg!(feature = "default-worker")),
            force_invisible,
            check_interrupt,
            coerce_all: coerce_all.unwrap_or(cfg!(feature = "default-coerce")),
            rng,
            unwrap_in_r,
            return_pref,
            s3_generic,
            s3_class,
            dots_spec,
            dots_span,
            lifecycle,
            strict: strict.unwrap_or(cfg!(feature = "default-strict")),
            error_in_r: resolved_error_in_r,
            internal,
            noexport,
            export,
            doc,
            background,
            c_symbol,
            r_name,
            r_entry,
            r_post_checks,
            r_on_exit,
        })
    }
}
// endregion
