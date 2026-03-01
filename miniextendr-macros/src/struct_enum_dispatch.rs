//! Dispatch `#[miniextendr]` on structs and enums to the appropriate derive helpers.
//!
//! This module handles `#[miniextendr]` when applied to structs or enums (not functions,
//! impl blocks, or traits). It dispatches to the correct derive helper based on:
//!
//! - Field count (1-field → ALTREP by default, multi-field → ExternalPtr by default)
//! - Explicit mode attributes (`list`, `dataframe`, `externalptr`, `match_arg`, `factor`)
//! - Preference markers (`prefer = "..."`)
//!
//! # Disambiguation table
//!
//! | Syntax | Result |
//! |---|---|
//! | `#[miniextendr]` on 1-field struct | ALTREP (backwards compat) |
//! | `#[miniextendr(externalptr)]` on 1-field struct | ExternalPtr |
//! | `#[miniextendr(list)]` on 1-field struct | List conversion |
//! | `#[miniextendr(class = "...", base = "...")]` on 1-field struct | ALTREP (explicit) |
//! | `#[miniextendr]` on multi-field struct | ExternalPtr |
//! | `#[miniextendr(list)]` on multi-field struct | IntoList + TryFromList + PreferList |
//! | `#[miniextendr(dataframe)]` on multi-field struct | DataFrameRow + PreferDataFrame |
//! | `#[miniextendr(prefer = "...")]` on struct | Prefer* marker |
//! | `#[miniextendr]` on fieldless enum | RFactor |
//! | `#[miniextendr(match_arg)]` on fieldless enum | MatchArg |

/// Parsed attributes for `#[miniextendr]` on structs/enums.
///
/// These attributes control which derive path is taken when `#[miniextendr]`
/// is applied to a struct or enum.
struct StructEnumAttrs {
    /// ALTREP class name override (forwarded to the ALTREP derive path).
    class: Option<String>,
    /// ALTREP base type override (forwarded to the ALTREP derive path).
    base: Option<String>,
    /// Derive `IntoList` + `TryFromList` + `PreferList` for struct-to-list round-tripping.
    list: bool,
    /// Derive `DataFrameRow` for struct-to-data-frame conversion.
    dataframe: bool,
    /// Force `ExternalPtr` derive even on single-field structs (which default to ALTREP).
    externalptr: bool,
    /// Derive `MatchArg` for enum (single-selection from R character scalar).
    match_arg: bool,
    /// Derive `RFactor` for enum (R factor representation).
    factor: bool,
    /// Preference marker string: `"externalptr"`, `"list"`, `"dataframe"`, or `"native"`.
    /// Acts as a soft mode selector when no explicit mode attribute is set.
    prefer: Option<String>,
}

/// Parses the attribute arguments of `#[miniextendr(...)]` when applied to a struct or enum.
///
/// Supports path-style flags (`list`, `dataframe`, `externalptr`, `match_arg`, `factor`)
/// and key-value pairs (`class = "..."`, `base = "..."`, `prefer = "..."`).
///
/// Returns a [`StructEnumAttrs`] on success, or a compile error if an unknown attribute is found.
fn parse_attrs(attr: proc_macro::TokenStream) -> syn::Result<StructEnumAttrs> {
    use syn::parse::Parser;

    let mut attrs = StructEnumAttrs {
        class: None,
        base: None,
        list: false,
        dataframe: false,
        externalptr: false,
        match_arg: false,
        factor: false,
        prefer: None,
    };

    // Parse as comma-separated meta items
    let parser = syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated;
    let metas = parser.parse(attr)?;

    for meta in &metas {
        match meta {
            syn::Meta::Path(path) => {
                let ident = path
                    .get_ident()
                    .ok_or_else(|| syn::Error::new_spanned(path, "expected identifier"))?;
                match ident.to_string().as_str() {
                    "list" => attrs.list = true,
                    "dataframe" => attrs.dataframe = true,
                    "externalptr" => attrs.externalptr = true,
                    "match_arg" => attrs.match_arg = true,
                    "factor" => attrs.factor = true,
                    _ => {
                        return Err(syn::Error::new_spanned(
                            ident,
                            format!(
                                "unknown #[miniextendr] attribute `{}`; expected one of: \
                                 list, dataframe, externalptr, match_arg, factor, prefer, class, base",
                                ident
                            ),
                        ));
                    }
                }
            }
            syn::Meta::NameValue(nv) => {
                let key = nv
                    .path
                    .get_ident()
                    .map(|i| i.to_string())
                    .unwrap_or_default();
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                }) = &nv.value
                {
                    match key.as_str() {
                        "class" => attrs.class = Some(s.value()),
                        "base" => attrs.base = Some(s.value()),
                        "prefer" => attrs.prefer = Some(s.value()),
                        // Silently ignore "pkg" for backwards compatibility
                        "pkg" => {}
                        _ => {
                            return Err(syn::Error::new_spanned(
                                &nv.path,
                                format!(
                                    "unknown #[miniextendr] attribute `{}`; expected one of: \
                                     class, base, prefer",
                                    key
                                ),
                            ));
                        }
                    }
                }
            }
            syn::Meta::List(_) => {
                // Not expected at this level
            }
        }
    }

    Ok(attrs)
}

/// Returns the number of fields on a struct (named, unnamed, or unit).
fn field_count(item: &syn::ItemStruct) -> usize {
    match &item.fields {
        syn::Fields::Named(f) => f.named.len(),
        syn::Fields::Unnamed(f) => f.unnamed.len(),
        syn::Fields::Unit => 0,
    }
}

/// Returns `true` if every variant of the enum is a unit variant (no fields).
fn is_fieldless_enum(item: &syn::ItemEnum) -> bool {
    item.variants
        .iter()
        .all(|v| matches!(v.fields, syn::Fields::Unit))
}

/// Main dispatch entry point for `#[miniextendr]` on a struct or enum.
///
/// Attempts to parse the item as a struct first, then as an enum.
/// Dispatches to the appropriate derive path based on the parsed attributes
/// and item shape (field count, variant structure).
///
/// Returns the original item plus any generated trait implementations as a combined token stream.
pub fn expand_struct_or_enum(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // Try parsing as struct first, then enum
    if let Ok(item_struct) = syn::parse::<syn::ItemStruct>(item.clone()) {
        return expand_struct(attr, item, &item_struct);
    }

    if let Ok(item_enum) = syn::parse::<syn::ItemEnum>(item.clone()) {
        return expand_enum(attr, item, &item_enum);
    }

    // If neither, give a helpful error
    syn::Error::new(
        proc_macro2::Span::call_site(),
        "#[miniextendr] on non-function items requires a struct or enum",
    )
    .into_compile_error()
    .into()
}

/// Dispatches `#[miniextendr]` on a struct to the correct derive path.
///
/// Decision logic:
/// - 1-field struct with no explicit mode: ALTREP (backwards compatibility)
/// - Explicit `list` mode: `IntoList` + `TryFromList` + `PreferList`
/// - Explicit `dataframe` mode: `IntoList` + `DataFrameRow` + companion `IntoR`
/// - Default multi-field or explicit `externalptr`: `ExternalPtr`
/// - `prefer = "native"`: `ExternalPtr` + `PreferRNative` marker
fn expand_struct(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
    item_struct: &syn::ItemStruct,
) -> proc_macro::TokenStream {
    let attrs = match parse_attrs(attr.clone()) {
        Ok(a) => a,
        Err(e) => return e.into_compile_error().into(),
    };

    let n_fields = field_count(item_struct);
    let has_altrep_attrs = attrs.class.is_some() || attrs.base.is_some();
    let has_mode_attr = attrs.list || attrs.dataframe || attrs.externalptr;

    // Resolve prefer into a mode if no explicit mode attr is set
    let effective_list = attrs.list || (!has_mode_attr && attrs.prefer.as_deref() == Some("list"));
    let effective_dataframe =
        attrs.dataframe || (!has_mode_attr && attrs.prefer.as_deref() == Some("dataframe"));
    let effective_externalptr =
        attrs.externalptr || (!has_mode_attr && attrs.prefer.as_deref() == Some("externalptr"));
    let effective_mode = effective_list || effective_dataframe || effective_externalptr;

    // 1-field struct: default to ALTREP unless explicitly overridden
    if n_fields == 1 && !effective_mode && attrs.prefer.is_none() {
        // ALTREP path (backwards compat or explicit class/base)
        return crate::altrep::expand_altrep_struct(attr, item);
    }

    // Reject ALTREP attrs on non-ALTREP paths
    if has_altrep_attrs && effective_mode {
        return syn::Error::new(
            item_struct.ident.span(),
            "cannot combine ALTREP attributes (class, base) with mode attributes (list, dataframe, externalptr)",
        )
        .into_compile_error()
        .into();
    }

    // Check for conflicting mode attrs
    let mode_count = [effective_list, effective_dataframe, effective_externalptr]
        .iter()
        .filter(|&&b| b)
        .count();
    if mode_count > 1 {
        return syn::Error::new(
            item_struct.ident.span(),
            "only one of `list`, `dataframe`, `externalptr` can be specified",
        )
        .into_compile_error()
        .into();
    }

    // Validate prefer value
    if let Some(ref prefer) = attrs.prefer
        && !matches!(
            prefer.as_str(),
            "externalptr" | "list" | "dataframe" | "native"
        )
    {
        return syn::Error::new(
            item_struct.ident.span(),
            format!(
                "unknown prefer value `{}`; expected one of: externalptr, list, dataframe, native",
                prefer
            ),
        )
        .into_compile_error()
        .into();
    }

    // Convert to DeriveInput for the derive helpers
    let derive_input: syn::DeriveInput = match syn::parse(item.clone()) {
        Ok(d) => d,
        Err(e) => return e.into_compile_error().into(),
    };
    // Strip #[miniextendr(...)] from the DeriveInput attrs — derive helpers don't expect them
    let derive_input = strip_miniextendr_attrs(derive_input);

    let item_ts: proc_macro2::TokenStream = item.into();

    if effective_list {
        // List mode: IntoList + TryFromList + PreferList
        let result = (|| -> syn::Result<proc_macro2::TokenStream> {
            let into_list = crate::list_derive::derive_into_list(derive_input.clone())?;
            let try_from_list = crate::list_derive::derive_try_from_list(derive_input.clone())?;
            let prefer_list = crate::list_derive::derive_prefer_list(derive_input)?;
            Ok(quote::quote! {
                #item_ts
                #into_list
                #try_from_list
                #prefer_list
            })
        })();
        return result.unwrap_or_else(|e| e.into_compile_error()).into();
    }

    if effective_dataframe {
        // DataFrame mode: IntoList + DataFrameRow + IntoR on companion type
        // IntoList is required by DataFrameRow's trait assertion.
        // The companion type ({Name}DataFrame) gets IntoR so it can be returned
        // from #[miniextendr] functions directly.
        let ident = &item_struct.ident;
        let df_ident = quote::format_ident!("{}DataFrame", ident);
        let result = (|| -> syn::Result<proc_macro2::TokenStream> {
            let into_list = crate::list_derive::derive_into_list(derive_input.clone())?;
            let dataframe_row = crate::dataframe_derive::derive_dataframe_row(derive_input)?;
            Ok(quote::quote! {
                #item_ts
                #into_list
                #dataframe_row

                impl ::miniextendr_api::into_r::IntoR for #df_ident {
                    type Error = std::convert::Infallible;

                    #[inline]
                    fn try_into_sexp(self) -> Result<::miniextendr_api::ffi::SEXP, Self::Error> {
                        Ok(self.into_sexp())
                    }

                    #[inline]
                    unsafe fn try_into_sexp_unchecked(self) -> Result<::miniextendr_api::ffi::SEXP, Self::Error> {
                        self.try_into_sexp()
                    }

                    #[inline]
                    fn into_sexp(self) -> ::miniextendr_api::ffi::SEXP {
                        ::miniextendr_api::convert::IntoDataFrame::into_data_frame(self).into_sexp()
                    }

                    #[inline]
                    unsafe fn into_sexp_unchecked(self) -> ::miniextendr_api::ffi::SEXP {
                        ::miniextendr_api::convert::IntoDataFrame::into_data_frame(self).into_sexp()
                    }
                }
            })
        })();
        return result.unwrap_or_else(|e| e.into_compile_error()).into();
    }

    // Default for multi-field or explicit externalptr: ExternalPtr
    // Also handles prefer = "native" (ExternalPtr + native preference marker)
    let result = (|| -> syn::Result<proc_macro2::TokenStream> {
        let external_ptr = crate::externalptr_derive::derive_external_ptr(derive_input.clone())?;

        // Apply native preference marker if specified
        let prefer = if attrs.prefer.as_deref() == Some("native") {
            crate::list_derive::derive_prefer_rnative(derive_input)?
        } else {
            proc_macro2::TokenStream::new()
        };

        Ok(quote::quote! {
            #item_ts
            #external_ptr
            #prefer
        })
    })();
    result.unwrap_or_else(|e| e.into_compile_error()).into()
}

/// Dispatches `#[miniextendr]` on a fieldless enum to the correct derive path.
///
/// Only C-style (fieldless) enums are supported. Dispatches to:
/// - `match_arg` mode: `MatchArg` derive (single-selection from R character scalar)
/// - `factor` mode or default: `RFactor` derive (R factor representation)
fn expand_enum(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
    item_enum: &syn::ItemEnum,
) -> proc_macro::TokenStream {
    let attrs = match parse_attrs(attr) {
        Ok(a) => a,
        Err(e) => return e.into_compile_error().into(),
    };

    if !is_fieldless_enum(item_enum) {
        return syn::Error::new(
            item_enum.ident.span(),
            "#[miniextendr] on enums requires all variants to be fieldless (C-style)",
        )
        .into_compile_error()
        .into();
    }

    let derive_input: syn::DeriveInput = match syn::parse(item.clone()) {
        Ok(d) => d,
        Err(e) => return e.into_compile_error().into(),
    };
    let derive_input = strip_miniextendr_attrs(derive_input);

    let item_ts: proc_macro2::TokenStream = item.into();

    if attrs.match_arg {
        // MatchArg mode
        let result = crate::match_arg_derive::derive_match_arg(derive_input);
        return match result {
            Ok(ts) => quote::quote! { #item_ts #ts }.into(),
            Err(e) => e.into_compile_error().into(),
        };
    }

    if attrs.factor {
        // Explicit factor mode
        let result = crate::factor_derive::derive_r_factor(derive_input);
        return match result {
            Ok(ts) => quote::quote! { #item_ts #ts }.into(),
            Err(e) => e.into_compile_error().into(),
        };
    }

    // Default: RFactor
    let result = crate::factor_derive::derive_r_factor(derive_input);
    match result {
        Ok(ts) => quote::quote! { #item_ts #ts }.into(),
        Err(e) => e.into_compile_error().into(),
    }
}

/// Strip `#[miniextendr(...)]` attributes from a DeriveInput.
///
/// Derive helpers don't expect `#[miniextendr]` attributes and may fail or
/// misinterpret them. We strip them before forwarding.
fn strip_miniextendr_attrs(mut input: syn::DeriveInput) -> syn::DeriveInput {
    input
        .attrs
        .retain(|attr| !attr.path().is_ident("miniextendr"));
    input
}
