//! Write-time placeholder & symbol-name formatting for the match_arg pipeline.
//!
//! Tradeoff signpost: this module is the **wiring** between the
//! `#[derive(MatchArg)]` proc macro (see `match_arg_derive`) and the
//! wrapper writer's substitution pass. The `match.arg` codegen path
//! deliberately splits responsibilities — the proc macro emits placeholders
//! into the R wrapper, and the wrapper writer resolves them to a concrete `c("a",
//! "b", ...)` literal (and the check's `aliases =` argument, if any) at
//! write time using `MX_MATCH_ARG_CHOICES`. That
//! split lets the variant list change without re-running `cargo expand` on
//! every consumer. Compared to hand-rolling `match.arg` in a wrapper body,
//! the user gets partial matching, R-visible defaults, and `@param` doc
//! lines for free, without ever stringifying the variant list in source.
//!
//! All three placeholders (`choices_placeholder`, `aliases_placeholder`,
//! `param_doc_placeholder`) share the same
//! `{c_ident_without_prefix}_{r_param}` stem so the wrapper writer's pass
//! can correlate them. Keep them together so the shape can't drift.
//!
//! The `c_ident_without_prefix` input has `C_` already stripped (or is a stem
//! already, e.g. `MyType__method`). Callers that have a full `c_ident` should
//! pass `c_ident.trim_start_matches("C_")`.
//!
//! All three helpers call `c_stem(...)` internally so callers may also pass a
//! full `c_ident` (e.g. `"C_my_fn"`) — the `C_` prefix is normalized away.

fn c_stem(c_ident: &str) -> &str {
    c_ident.trim_start_matches("C_")
}

/// Start of every [`choices_placeholder`]. The shadowing pass
/// (`r_shadowing`) reads a name with this prefix as a call to `c`, which is
/// what the placeholder becomes.
pub(crate) const CHOICES_PLACEHOLDER_PREFIX: &str = ".__MX_MATCH_ARG_CHOICES_";

/// R-side placeholder that the wrapper writer resolves to a `c("a", "b", ...)` literal
/// at write time. Substituted by `MX_MATCH_ARG_CHOICES` entries.
pub(crate) fn choices_placeholder(c_ident: &str, r_param: &str) -> String {
    format!(
        "{CHOICES_PLACEHOLDER_PREFIX}{}_{}__",
        c_stem(c_ident),
        r_param
    )
}

/// Start of every [`aliases_placeholder`].
pub(crate) const ALIASES_PLACEHOLDER_PREFIX: &str = ".__MX_MATCH_ARG_ALIASES_";

/// R-side placeholder for the trailing `aliases =` argument of a `match_arg`
/// parameter's check (#1843), written as `, <placeholder>` after the other
/// arguments. The wrapper writer replaces `, <placeholder>` with
/// `, aliases = base::c("grey" = "gray")`, or with nothing when the type has
/// no aliases (`registry::match_arg_aliases_arg`), from the same
/// `MX_MATCH_ARG_CHOICES` entry as [`choices_placeholder`]. The macro can't
/// tell which: `MatchArg::ALIASES` resolves at link time, as `CHOICES` does.
pub(crate) fn aliases_placeholder(c_ident: &str, r_param: &str) -> String {
    format!(
        "{ALIASES_PLACEHOLDER_PREFIX}{}_{}__",
        c_stem(c_ident),
        r_param
    )
}

/// R-side placeholder for the `@param` doc line, substituted by
/// `MX_MATCH_ARG_PARAM_DOCS` entries at write time. See #210.
pub(crate) fn param_doc_placeholder(c_ident: &str, r_param: &str) -> String {
    format!(
        ".__MX_MATCH_ARG_PARAM_DOC_{}_{}__",
        c_stem(c_ident),
        r_param
    )
}

/// Extract the unquoted form of a user-supplied `default = "..."` literal
/// for a `match_arg` parameter.
///
/// The user writes the value as it appears in R source, so a string default
/// is `default = "\"zstd\""` — the `String` we receive is `"zstd"` (with the
/// quote chars). Strip the outer quotes if present; otherwise pass the raw
/// value through unchanged. The write-time pass validates the result against
/// the enum's `CHOICES` and panics on miss, so a malformed literal (e.g.
/// `default = "1L"`) still surfaces as a clear error at wrapper generation.
pub(crate) fn extract_match_arg_default(raw: &str) -> String {
    raw.strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .unwrap_or(raw)
        .to_string()
}

/// Derive a safe Rust ident from a write-time placeholder string.
///
/// Escape underscores and dots distinctly so dotted formals cannot collide
/// with Rust-style spellings. The fixed leading dot is not part of the ident.
pub(crate) fn placeholder_ident_suffix(placeholder: &str) -> String {
    placeholder
        .trim_start_matches('.')
        .chars()
        .map(|c| match c {
            '_' => "__".to_owned(),
            '.' => "_dot_".to_owned(),
            _ => c.to_string(),
        })
        .collect()
}

/// The choice registration referenced by its parameter documentation too.
pub(crate) fn choices_entry_ident(placeholder: &str) -> syn::Ident {
    syn::Ident::new(
        &format!(
            "match_arg_choices_entry_{}",
            placeholder_ident_suffix(placeholder)
        ),
        proc_macro2::Span::call_site(),
    )
}

/// Emit the `MX_MATCH_ARG_CHOICES` static + its linkme registration.
///
/// Factored so lib.rs (standalone fns) and miniextendr_impl.rs (impl methods)
/// can't drift apart — both previously open-coded the same quote! block.
///
/// `preferred_default` is the unquoted form of the user's `default = "..."`
/// (e.g. `"zstd"`). Pass `""` when the user supplied no default — the write
/// pass then keeps the natural enum order.
///
/// `values` is a const expression yielding `(choices, aliases)`.
/// `aliases_placeholder` is the parameter's [`aliases_placeholder`], which
/// its check carries; the entry resolves it from the type's
/// `MatchArg::ALIASES`.
pub(crate) fn choices_entry_tokens(
    cfg_attrs: &[syn::Attribute],
    entry_ident: &syn::Ident,
    placeholder: &str,
    aliases_placeholder: &str,
    values: &proc_macro2::TokenStream,
    preferred_default: &str,
) -> proc_macro2::TokenStream {
    quote::quote! {
        #(#cfg_attrs)*
        #[cfg_attr(not(target_arch = "wasm32"), ::miniextendr_api::linkme::distributed_slice(::miniextendr_api::registry::MX_MATCH_ARG_CHOICES), linkme(crate = ::miniextendr_api::linkme))]
        #[allow(non_upper_case_globals)]
        #[allow(non_snake_case)]
        static #entry_ident: ::miniextendr_api::registry::MatchArgChoicesEntry = {
            let (__mx_choices, __mx_aliases) = #values;
            ::miniextendr_api::registry::MatchArgChoicesEntry {
                placeholder: #placeholder,
                choices: __mx_choices,
                preferred_default: #preferred_default,
                aliases_placeholder: #aliases_placeholder,
                aliases: __mx_aliases,
            }
        };
    }
}

/// Emit the `MX_MATCH_ARG_PARAM_DOCS` static + its linkme registration.
///
/// `suffix` is the text the write pass puts after the rendered choice list,
/// from `ParamAttrs::choice_doc_suffix` (", or NULL for no choice" for an
/// `Option<T>` param, #1473; the omission note for `Missing<T>`, #1551).
pub(crate) fn param_doc_entry_tokens(
    cfg_attrs: &[syn::Attribute],
    entry_ident: &syn::Ident,
    placeholder: &str,
    several_ok: bool,
    suffix: &str,
    choices_entry: &syn::Ident,
) -> proc_macro2::TokenStream {
    quote::quote! {
        #(#cfg_attrs)*
        #[cfg_attr(not(target_arch = "wasm32"), ::miniextendr_api::linkme::distributed_slice(::miniextendr_api::registry::MX_MATCH_ARG_PARAM_DOCS), linkme(crate = ::miniextendr_api::linkme))]
        #[allow(non_upper_case_globals)]
        #[allow(non_snake_case)]
        static #entry_ident: ::miniextendr_api::registry::MatchArgParamDocEntry =
            ::miniextendr_api::registry::MatchArgParamDocEntry {
                placeholder: #placeholder,
                several_ok: #several_ok,
                suffix: #suffix,
                choices: &#choices_entry,
            };
    }
}

/// Register the same choice/default/doc placeholders for every method emitter.
pub(crate) fn method_entries(
    sig: &syn::Signature,
    per_param: &std::collections::HashMap<String, crate::miniextendr_fn::ParamAttrs>,
    c_ident: &str,
    cfg_attrs: &[syn::Attribute],
    inherited_callable: Option<&proc_macro2::TokenStream>,
) -> proc_macro2::TokenStream {
    let mut out = proc_macro2::TokenStream::new();
    let names = crate::r_wrapper_builder::FormalNames::from_params(per_param);

    // Signature order, so the emitted items don't follow the map's order.
    for (rust_name, attrs) in
        crate::miniextendr_fn::per_param_in_signature_order(&sig.inputs, per_param)
    {
        if !attrs.match_arg {
            continue;
        }
        // Find the parameter type from the (already-normalized) signature.
        let Some(param_ty) = find_param_type(&sig.inputs, rust_name) else {
            continue;
        };
        // several_ok containers (Vec<Mode>, Box<[Mode]>, [Mode; N], &[Mode]) and the
        // `Option<Mode>` scalar form both wrap the enum; register the enum's
        // CHOICES, not the wrapper's.
        let several_ok = attrs.several_ok;
        let choices_ty = crate::match_arg_choices_ty(param_ty, several_ok).clone();
        let values = if let Some(callable) = inherited_callable {
            crate::trait_signature::match_arg_values(sig, rust_name, &choices_ty, callable)
        } else {
            quote::quote!((
                <#choices_ty as ::miniextendr_api::match_arg::MatchArg>::CHOICES,
                <#choices_ty as ::miniextendr_api::match_arg::MatchArg>::ALIASES,
            ))
        };

        let r_name = names.resolve(rust_name);

        // Placeholder that the write-time substitution pass replaces with the
        // literal `c("a", "b", ...)` default. Shape matches the standalone-fn convention.
        let placeholder = crate::r_class_formatter::match_arg_placeholder(c_ident, &r_name);
        let entry_ident = choices_entry_ident(&placeholder);
        let doc_placeholder =
            crate::r_class_formatter::match_arg_param_doc_placeholder(c_ident, &r_name);
        let doc_entry_ident = syn::Ident::new(
            &format!(
                "match_arg_param_doc_entry_{}",
                crate::match_arg_keys::placeholder_ident_suffix(&doc_placeholder)
            ),
            proc_macro2::Span::call_site(),
        );

        let preferred_default = attrs
            .default
            .as_deref()
            .map(crate::match_arg_keys::extract_match_arg_default)
            .unwrap_or_default();
        let aliases_placeholder = crate::match_arg_keys::aliases_placeholder(c_ident, &r_name);
        let choices_entry_tokens = choices_entry_tokens(
            cfg_attrs,
            &entry_ident,
            &placeholder,
            &aliases_placeholder,
            &values,
            &preferred_default,
        );
        let param_doc_entry_tokens = param_doc_entry_tokens(
            cfg_attrs,
            &doc_entry_ident,
            &doc_placeholder,
            several_ok,
            &attrs.choice_doc_suffix(),
            &entry_ident,
        );

        out.extend(quote::quote! {
            #choices_entry_tokens
            #param_doc_entry_tokens
        });
    }

    out
}

/// Find a parameter's Rust type from a stripped signature by identifier name.
fn find_param_type<'a>(
    inputs: &'a syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
    name: &str,
) -> Option<&'a syn::Type> {
    for arg in inputs {
        if let syn::FnArg::Typed(pt) = arg
            && let syn::Pat::Ident(pat_ident) = pt.pat.as_ref()
            && crate::naming::unraw(&pat_ident.ident) == name
        {
            return Some(pt.ty.as_ref());
        }
    }
    None
}
