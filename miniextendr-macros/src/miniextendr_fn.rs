//! Function signature parsing for `#[miniextendr]`.
//!
//! This module handles parsing and normalizing Rust function signatures for the
//! `#[miniextendr]` attribute macro. It provides:
//!
//! - [`MiniextendrFunctionParsed`]: Parsed function with normalization and codegen helpers
//! - [`MiniextendrFnAttrs`]: Parsed `#[miniextendr(...)]` attribute options
//! - [`CoercionMapping`]: Type coercion analysis for automatic R→Rust conversion

use crate::{call_method_def_ident_for, r_wrapper_const_ident_for};

// =============================================================================
// Coercion analysis
// =============================================================================

/// Result of coercion analysis for a type.
/// Contains the R native type to extract from SEXP and the target type to coerce to.
pub(crate) enum CoercionMapping {
    /// Scalar coercion: extract R native type, coerce to target
    Scalar {
        r_native: proc_macro2::TokenStream,
        target: proc_macro2::TokenStream,
    },
    /// Vec coercion: extract R native slice, coerce element-wise to `Vec<target>`
    Vec {
        r_native_elem: proc_macro2::TokenStream,
        target_elem: proc_macro2::TokenStream,
    },
}

impl CoercionMapping {
    /// Get the coercion mapping for a type, if it needs coercion.
    /// Returns None if the type is R-native (no coercion needed) or unknown.
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

/// Check if an attribute is `#[miniextendr(coerce)]`.
pub(crate) fn is_miniextendr_coerce_attr(attr: &syn::Attribute) -> bool {
    attr.path().is_ident("miniextendr")
        && matches!(&attr.meta, syn::Meta::List(list) if list.parse_args::<syn::Ident>().is_ok_and(|id| id == "coerce"))
}

/// Parse default value from `#[miniextendr(default = "...")]`.
///
/// Returns Some((default_value, attr_span)) if the attribute is present, None otherwise.
pub(crate) fn parse_default_attr(attr: &syn::Attribute) -> Option<(String, proc_macro2::Span)> {
    use syn::spanned::Spanned;
    if !attr.path().is_ident("miniextendr") {
        return None;
    }
    let syn::Meta::List(list) = &attr.meta else {
        return None;
    };

    // Parse as `default = "value"`
    let Ok(nv) = list.parse_args::<syn::MetaNameValue>() else {
        return None;
    };

    if !nv.path.is_ident("default") {
        return None;
    }

    // Extract string literal value
    let syn::Expr::Lit(expr_lit) = &nv.value else {
        return None;
    };
    let syn::Lit::Str(lit_str) = &expr_lit.lit else {
        return None;
    };

    Some((lit_str.value(), attr.span()))
}

// =============================================================================
// Function parsing
// =============================================================================

/// Parsed + normalized Rust function item for `#[miniextendr]`.
///
/// This performs signature normalization that the wrapper generator depends on:
/// - `...` → a final `&miniextendr_api::dots::Dots` argument
/// - `_` wildcard patterns → synthetic identifiers (`__unused0`, `__unused1`, ...)
/// - consumes `#[miniextendr(coerce)]` parameter attributes and records which params had it
///
/// Any non-identifier parameter patterns (e.g. `(a, b): (i32, i32)`) are rejected, because the
/// wrapper generator needs a stable parameter name for both:
/// - the generated C wrapper signature
/// - the generated R wrapper argument names
pub(crate) struct MiniextendrFunctionParsed {
    /// The normalized function item (with dots transformed, wildcards renamed).
    item: syn::ItemFn,
    /// Whether the original function had `...` (variadic).
    has_dots: bool,
    /// If dots were named (e.g., `my_dots: ...`), the identifier.
    named_dots: Option<syn::Ident>,
    /// Parameter names that had `#[miniextendr(coerce)]` attribute.
    per_param_coerce: std::collections::HashSet<String>,
    /// Parameter names with `#[miniextendr(default = "...")]` and their default values.
    per_param_defaults: std::collections::HashMap<String, String>,
}

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
                         the macro generates its own C symbol names. \
                         For extern \"C-unwind\" functions, #[export_name] is allowed.",
                    ));
                }
            }
        }

        // Transform `_` wildcard patterns to synthetic identifiers, and consume
        // per-parameter `#[miniextendr(coerce)]` and `#[miniextendr(default = "...")]` attributes.
        let mut per_param_coerce: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        let mut per_param_defaults: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        let mut per_param_default_spans: std::collections::HashMap<String, proc_macro2::Span> =
            std::collections::HashMap::new();
        let mut unused_counter = 0usize;
        for arg in &mut item.sig.inputs {
            let syn::FnArg::Typed(pat_type) = arg else {
                // Self parameters are not allowed in standalone functions.
                // Users should use #[miniextendr(env|r6|s3|s4|s7)] on impl blocks instead.
                // The error is raised in lib.rs c_wrapper_inputs generation.
                continue;
            };

            let had_coerce_attr = pat_type.attrs.iter().any(is_miniextendr_coerce_attr);
            let default_with_span = pat_type.attrs.iter().find_map(parse_default_attr);

            // Remove miniextendr attributes from parameters (coerce and default)
            pat_type.attrs.retain(|attr| {
                !is_miniextendr_coerce_attr(attr) && parse_default_attr(attr).is_none()
            });

            match pat_type.pat.as_ref() {
                syn::Pat::Ident(pat_ident) => {
                    let param_name = pat_ident.ident.to_string();
                    if had_coerce_attr {
                        per_param_coerce.insert(param_name.clone());
                    }
                    if let Some((default, span)) = default_with_span {
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
                    if had_coerce_attr {
                        per_param_coerce.insert(synthetic_name.clone());
                    }
                    if let Some((default, span)) = default_with_span {
                        per_param_defaults.insert(synthetic_name.clone(), default);
                        per_param_default_spans.insert(synthetic_name, span);
                    }
                }
                _ => {
                    return Err(syn::Error::new(
                        pat_type.pat.span(),
                        "miniextendr parameters must be simple identifiers (patterns are not supported)",
                    ));
                }
            }
        }

        if has_dots {
            item.sig.variadic = None;
            item.sig
                .inputs
                .push(if let Some(named_dots) = named_dots.as_ref() {
                    syn::parse_quote!(#named_dots: &::miniextendr_api::dots::Dots)
                } else {
                    // cannot use `_` as variable name, thus cannot use it as a placeholder for `...`
                    // Check that no existing parameter is named `_dots`
                    for arg in &item.sig.inputs {
                        let syn::FnArg::Typed(pat_type) = arg else {
                            continue;
                        };
                        if let syn::Pat::Ident(pat_ident) = pat_type.pat.as_ref()
                            && pat_ident.ident == "_dots" {
                                return Err(syn::Error::new(
                                    pat_ident.ident.span(),
                                    "parameter named `_dots` conflicts with implicit dots parameter; use named dots like `my_dots: ...` instead",
                                ));
                            }
                    }
                    syn::parse_quote!(_dots: &::miniextendr_api::dots::Dots)
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
            per_param_defaults,
        })
    }
}

impl MiniextendrFunctionParsed {
    // -------------------------------------------------------------------------
    // Accessors for parsed metadata
    // -------------------------------------------------------------------------

    /// Whether the original function had `...` (variadic).
    pub(crate) fn has_dots(&self) -> bool {
        self.has_dots
    }

    /// If dots were named (e.g., `my_dots: ...`), returns the identifier.
    pub(crate) fn named_dots(&self) -> Option<&syn::Ident> {
        self.named_dots.as_ref()
    }

    /// Check if a parameter name had `#[miniextendr(coerce)]` attribute.
    pub(crate) fn has_coerce_attr(&self, param_name: &str) -> bool {
        self.per_param_coerce.contains(param_name)
    }

    /// Get all parameter defaults.
    pub(crate) fn param_defaults(&self) -> &std::collections::HashMap<String, String> {
        &self.per_param_defaults
    }

    // -------------------------------------------------------------------------
    // Accessors for signature components
    // -------------------------------------------------------------------------

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
    #[allow(dead_code)] // Used in tests
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

    // -------------------------------------------------------------------------
    // Codegen helpers
    // -------------------------------------------------------------------------

    /// Whether this function needs an internal C wrapper (true for Rust ABI functions).
    /// Extern "C-unwind" functions are used directly without wrapping.
    pub(crate) fn uses_internal_c_wrapper(&self) -> bool {
        self.abi().is_none()
    }

    /// Identifier for the generated `const` `R_CallMethodDef` value.
    pub(crate) fn call_method_def_ident(&self) -> syn::Ident {
        call_method_def_ident_for(self.ident())
    }

    /// Identifier for the generated `const &str` holding the R wrapper code.
    pub(crate) fn r_wrapper_const_ident(&self) -> syn::Ident {
        r_wrapper_const_ident_for(self.ident())
    }

    /// Identifier for the C wrapper function.
    /// - Rust ABI: `C_<name>`
    /// - Extern "C-unwind": same as the function name (or export_name if specified)
    pub(crate) fn c_wrapper_ident(&self) -> syn::Ident {
        if self.uses_internal_c_wrapper() {
            quote::format_ident!("C_{}", self.ident())
        } else {
            // For extern functions, check for #[export_name = "..."]
            self.export_name_ident()
                .unwrap_or_else(|| self.ident().clone())
        }
    }

    /// Extract the export name from `#[export_name = "..."]` attribute, if present.
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
}

// =============================================================================
// Attribute parsing
// =============================================================================

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
/// - `return = "auto" | "list" | "externalptr" | "vector"`: prefer a specific `IntoR` path
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

impl syn::parse::Parse for MiniextendrFnAttrs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        use syn::spanned::Spanned;
        let mut out = Self::default();
        if input.is_empty() {
            return Ok(out);
        }

        let metas =
            syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated(input)?;

        for meta in metas {
            match meta {
                // Simple identifiers: invisible, visible, check_interrupt, coerce, worker, rng
                syn::Meta::Path(path) => {
                    if let Some(ident) = path.get_ident() {
                        if ident == "invisible" {
                            out.force_invisible = Some(true);
                        } else if ident == "visible" {
                            out.force_invisible = Some(false);
                        } else if ident == "check_interrupt" {
                            out.check_interrupt = true;
                        } else if ident == "coerce" {
                            out.coerce_all = true;
                        } else if ident == "rng" {
                            out.rng = true;
                        } else if ident == "unwrap_in_r" {
                            out.unwrap_in_r = true;
                        } else if ident == "worker" {
                            out.force_worker = true;
                        } else {
                            return Err(syn::Error::new_spanned(
                                ident,
                                "unknown `#[miniextendr]` option; expected one of: invisible, visible, check_interrupt, unsafe(main_thread), worker, coerce, rng, unwrap_in_r",
                            ));
                        }
                    }
                }
                syn::Meta::NameValue(nv) => {
                    if nv.path.is_ident("return") {
                        match &nv.value {
                            syn::Expr::Lit(expr_lit) => {
                                if let syn::Lit::Str(lit) = &expr_lit.lit {
                                    let v = lit.value();
                                    out.return_pref = match v.as_str() {
                                        "list" => ReturnPref::List,
                                        "externalptr" => ReturnPref::ExternalPtr,
                                        "vector" | "native" => ReturnPref::Native,
                                        "auto" => ReturnPref::Auto,
                                        _ => {
                                            return Err(syn::Error::new_spanned(
                                                lit,
                                                "return must be one of: auto, list, externalptr, vector/native",
                                            ));
                                        }
                                    };
                                } else {
                                    return Err(syn::Error::new_spanned(
                                        &expr_lit.lit,
                                        "return expects a string literal",
                                    ));
                                }
                            }
                            other => {
                                return Err(syn::Error::new_spanned(
                                    other,
                                    "return expects a string literal",
                                ));
                            }
                        }
                    } else if nv.path.is_ident("dots") {
                        // dots = typed_list!(...) - capture the macro invocation
                        // Store span for error reporting
                        out.dots_span = Some(nv.path.span());
                        if let syn::Expr::Macro(expr_macro) = &nv.value {
                            if expr_macro.mac.path.is_ident("typed_list") {
                                // Capture the entire macro invocation as TokenStream
                                out.dots_spec = Some(quote::quote!(#expr_macro));
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
                    } else {
                        return Err(syn::Error::new_spanned(
                            nv,
                            "unknown option; expected `return` or `dots`",
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
                                out.force_main_thread = true;
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
                    } else if list.path.is_ident("s3") {
                        // Parse s3(generic = "...", class = "...")
                        list.parse_nested_meta(|meta| {
                            if meta.path.is_ident("generic") {
                                let _: syn::Token![=] = meta.input.parse()?;
                                let value: syn::LitStr = meta.input.parse()?;
                                out.s3_generic = Some(value.value());
                            } else if meta.path.is_ident("class") {
                                let _: syn::Token![=] = meta.input.parse()?;
                                let value: syn::LitStr = meta.input.parse()?;
                                out.s3_class = Some(value.value());
                            } else {
                                return Err(
                                    meta.error("unknown s3 option; expected `generic` or `class`")
                                );
                            }
                            Ok(())
                        })?;
                        // Validate: s3 requires class (generic can default to function name)
                        if out.s3_class.is_none() {
                            return Err(syn::Error::new_spanned(
                                &list,
                                "s3(...) requires `class = \"...\"` to specify the S3 class suffix; \
                                 `generic` is optional and defaults to the function name",
                            ));
                        }
                    } else {
                        // invisible(something) etc
                        return Err(syn::Error::new_spanned(
                            list,
                            "this option does not take any arguments",
                        ));
                    }
                }
            }
        }

        Ok(out)
    }
}
