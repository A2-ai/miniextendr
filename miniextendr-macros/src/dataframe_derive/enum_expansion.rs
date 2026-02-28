//! Enum-specific DataFrame derive expansion.
//!
//! Generates a companion struct where every column is `Vec<Option<T>>`, with
//! `None` fill for fields absent in a given variant.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, Fields};

use super::{
    ColumnRegistry, DataFrameAttrs, EnumResolvedField, FieldTypeKind, ResolvedColumn, VariantInfo,
    VariantShape, classify_field_type, parse_field_attrs,
};
use std::collections::HashMap;

/// Derive `DataFrameRow` for an enum with `#[dataframe(align)]`.
///
/// Generates a companion struct where every column is `Vec<Option<T>>`, with
/// `None` fill for fields absent in a given variant.
pub(super) fn derive_enum_dataframe(
    row_name: &syn::Ident,
    _input: &DeriveInput,
    data: &syn::DataEnum,
    df_name: &syn::Ident,
    attrs: &DataFrameAttrs,
) -> syn::Result<TokenStream> {
    // ── Validate variants ────────────────────────────────────────────────
    if data.variants.is_empty() {
        return Err(syn::Error::new_spanned(
            row_name,
            "DataFrameRow requires at least one variant",
        ));
    }

    let mut variant_infos: Vec<VariantInfo> = Vec::new();

    for variant in &data.variants {
        match &variant.fields {
            Fields::Named(fields) => {
                let mut resolved = Vec::new();
                let mut skipped = Vec::new();
                for f in &fields.named {
                    let fa = parse_field_attrs(f)?;
                    let rust_name = f.ident.as_ref().unwrap().clone();
                    if fa.skip {
                        skipped.push(rust_name);
                        continue;
                    }
                    let col_name_str = fa.rename.unwrap_or_else(|| rust_name.to_string());
                    let binding = format_ident!("__v_{}", rust_name);

                    if fa.as_list {
                        resolved.push(EnumResolvedField::Single {
                            col_name: format_ident!("{}", col_name_str),
                            binding: binding.clone(),
                            rust_name: rust_name.clone(),
                            ty: f.ty.clone(),
                        });
                    } else {
                        match classify_field_type(&f.ty) {
                            FieldTypeKind::FixedArray(elem_ty, len) => {
                                resolved.push(EnumResolvedField::ExpandedFixed {
                                    base_name: col_name_str,
                                    binding: binding.clone(),
                                    rust_name: rust_name.clone(),
                                    elem_ty: elem_ty.clone(),
                                    len,
                                });
                            }
                            FieldTypeKind::VariableVec(elem_ty) => {
                                if let Some(width) = fa.width {
                                    resolved.push(EnumResolvedField::ExpandedVec {
                                        base_name: col_name_str,
                                        binding: binding.clone(),
                                        rust_name: rust_name.clone(),
                                        elem_ty: elem_ty.clone(),
                                        width,
                                    });
                                } else if fa.expand {
                                    resolved.push(EnumResolvedField::AutoExpandVec {
                                        base_name: col_name_str,
                                        binding: binding.clone(),
                                        rust_name: rust_name.clone(),
                                        elem_ty: elem_ty.clone(),
                                    });
                                } else {
                                    resolved.push(EnumResolvedField::Single {
                                        col_name: format_ident!("{}", col_name_str),
                                        binding: binding.clone(),
                                        rust_name: rust_name.clone(),
                                        ty: f.ty.clone(),
                                    });
                                }
                            }
                            FieldTypeKind::Scalar => {
                                if fa.width.is_some() {
                                    return Err(syn::Error::new_spanned(
                                        &f.ty,
                                        "`width` is only valid on `Vec<T>` fields",
                                    ));
                                }
                                if fa.expand {
                                    return Err(syn::Error::new_spanned(
                                        &f.ty,
                                        "`expand`/`unnest` is only valid on `[T; N]` or `Vec<T>` fields",
                                    ));
                                }
                                resolved.push(EnumResolvedField::Single {
                                    col_name: format_ident!("{}", col_name_str),
                                    binding: binding.clone(),
                                    rust_name: rust_name.clone(),
                                    ty: f.ty.clone(),
                                });
                            }
                        }
                    }
                }
                variant_infos.push(VariantInfo {
                    name: variant.ident.clone(),
                    shape: VariantShape::Named,
                    fields: resolved,
                    skipped_fields: skipped,
                });
            }
            Fields::Unnamed(fields) => {
                let mut resolved = Vec::new();
                for (i, f) in fields.unnamed.iter().enumerate() {
                    let fa = parse_field_attrs(f)?;
                    let rust_name = format_ident!("_{}", i);
                    if fa.skip {
                        continue;
                    }
                    let col_name_str = fa.rename.unwrap_or_else(|| rust_name.to_string());
                    let binding = format_ident!("__v_{}", rust_name);

                    // Tuple enum fields: same expansion logic
                    if fa.as_list {
                        resolved.push(EnumResolvedField::Single {
                            col_name: format_ident!("{}", col_name_str),
                            binding,
                            rust_name,
                            ty: f.ty.clone(),
                        });
                    } else {
                        match classify_field_type(&f.ty) {
                            FieldTypeKind::FixedArray(elem_ty, len) => {
                                resolved.push(EnumResolvedField::ExpandedFixed {
                                    base_name: col_name_str,
                                    binding,
                                    rust_name,
                                    elem_ty: elem_ty.clone(),
                                    len,
                                });
                            }
                            FieldTypeKind::VariableVec(elem_ty) => {
                                if let Some(width) = fa.width {
                                    resolved.push(EnumResolvedField::ExpandedVec {
                                        base_name: col_name_str,
                                        binding,
                                        rust_name,
                                        elem_ty: elem_ty.clone(),
                                        width,
                                    });
                                } else if fa.expand {
                                    resolved.push(EnumResolvedField::AutoExpandVec {
                                        base_name: col_name_str,
                                        binding,
                                        rust_name,
                                        elem_ty: elem_ty.clone(),
                                    });
                                } else {
                                    resolved.push(EnumResolvedField::Single {
                                        col_name: format_ident!("{}", col_name_str),
                                        binding,
                                        rust_name,
                                        ty: f.ty.clone(),
                                    });
                                }
                            }
                            FieldTypeKind::Scalar => {
                                resolved.push(EnumResolvedField::Single {
                                    col_name: format_ident!("{}", col_name_str),
                                    binding,
                                    rust_name,
                                    ty: f.ty.clone(),
                                });
                            }
                        }
                    }
                }
                variant_infos.push(VariantInfo {
                    name: variant.ident.clone(),
                    shape: VariantShape::Tuple,
                    fields: resolved,
                    skipped_fields: vec![],
                });
            }
            Fields::Unit => {
                variant_infos.push(VariantInfo {
                    name: variant.ident.clone(),
                    shape: VariantShape::Unit,
                    fields: vec![],
                    skipped_fields: vec![],
                });
            }
        }
    }

    // ── Resolve unified schema ───────────────────────────────────────────
    // Collect all unique column names, check type consistency.
    // Expanded fields contribute multiple columns to the schema.
    let coerce_to_string = attrs.conflicts.as_deref() == Some("string");
    let string_ty: syn::Type = syn::parse_quote!(String);
    let mut registry = ColumnRegistry::new(coerce_to_string, &string_ty);

    for (variant_idx, vi) in variant_infos.iter().enumerate() {
        for erf in &vi.fields {
            // Use the rust_name span for error reporting
            let err_span = match erf {
                EnumResolvedField::Single { rust_name, .. }
                | EnumResolvedField::ExpandedFixed { rust_name, .. }
                | EnumResolvedField::ExpandedVec { rust_name, .. }
                | EnumResolvedField::AutoExpandVec { rust_name, .. } => rust_name.span(),
            };
            match erf {
                EnumResolvedField::Single { col_name, ty, .. } => {
                    registry.register(
                        &col_name.to_string(),
                        ty,
                        variant_idx,
                        &vi.name,
                        err_span,
                    )?;
                }
                EnumResolvedField::ExpandedFixed {
                    base_name,
                    elem_ty,
                    len,
                    ..
                } => {
                    for i in 1..=*len {
                        let name = format!("{}_{}", base_name, i);
                        registry.register(&name, elem_ty, variant_idx, &vi.name, err_span)?;
                    }
                }
                EnumResolvedField::ExpandedVec {
                    base_name,
                    elem_ty,
                    width,
                    ..
                } => {
                    for i in 1..=*width {
                        let name = format!("{}_{}", base_name, i);
                        registry.register(&name, elem_ty, variant_idx, &vi.name, err_span)?;
                    }
                }
                // AutoExpandVec: not registered in ColumnRegistry (width is dynamic).
                // Collected separately below.
                EnumResolvedField::AutoExpandVec { .. } => {}
            }
        }
    }
    let columns = registry.columns;

    // ── Collect auto-expand fields ──────────────────────────────────────
    struct EnumAutoExpandCol {
        df_field: syn::Ident,
        base_name: String,
        elem_ty: syn::Type,
        present_in: Vec<usize>,
    }

    let mut auto_expand_cols: Vec<EnumAutoExpandCol> = Vec::new();
    let mut auto_expand_index: HashMap<String, usize> = HashMap::new();

    for (variant_idx, vi) in variant_infos.iter().enumerate() {
        for erf in &vi.fields {
            if let EnumResolvedField::AutoExpandVec {
                base_name,
                elem_ty,
                rust_name,
                ..
            } = erf
            {
                if let Some(&idx) = auto_expand_index.get(base_name) {
                    let existing = &auto_expand_cols[idx];
                    if existing.elem_ty != *elem_ty {
                        if coerce_to_string {
                            auto_expand_cols[idx].elem_ty = string_ty.clone();
                        } else {
                            return Err(syn::Error::new(
                                rust_name.span(),
                                format!(
                                    "type conflict for auto-expand field `{}`: different element type \
                                     than a previous variant; \
                                     use `#[dataframe(conflicts = \"string\")]` to coerce",
                                    base_name,
                                ),
                            ));
                        }
                    }
                    auto_expand_cols[idx].present_in.push(variant_idx);
                } else {
                    let idx = auto_expand_cols.len();
                    auto_expand_cols.push(EnumAutoExpandCol {
                        df_field: format_ident!("{}", base_name),
                        base_name: base_name.clone(),
                        elem_ty: elem_ty.clone(),
                        present_in: vec![variant_idx],
                    });
                    auto_expand_index.insert(base_name.clone(), idx);
                }
            }
        }
    }
    let has_enum_auto_expand = !auto_expand_cols.is_empty();

    // ── Generate companion struct ────────────────────────────────────────
    let has_tag = attrs.tag.is_some();

    let tag_field = if has_tag {
        quote! { pub _tag: Vec<String>, }
    } else {
        TokenStream::new()
    };

    let mut df_fields: Vec<TokenStream> = columns
        .iter()
        .map(|col| {
            let name = &col.col_name;
            let ty = &col.ty;
            quote! { pub #name: Vec<Option<#ty>> }
        })
        .collect();
    // Auto-expand fields: Vec<Option<Vec<T>>>
    for ac in &auto_expand_cols {
        let name = &ac.df_field;
        let ty = &ac.elem_ty;
        df_fields.push(quote! { pub #name: Vec<Option<Vec<#ty>>> });
    }

    let dataframe_struct = quote! {
        #[derive(Debug, Clone)]
        pub struct #df_name {
            #tag_field
            #(#df_fields),*
        }
    };

    // ── Generate IntoDataFrame ───────────────────────────────────────────
    // The first "real" column for length reference. If tag exists, use _tag.
    let length_ref = if has_tag {
        quote! { self._tag.len() }
    } else if let Some(first_col) = columns.first() {
        let first = &first_col.col_name;
        quote! { self.#first.len() }
    } else if let Some(first_ac) = auto_expand_cols.first() {
        let first = &first_ac.df_field;
        quote! { self.#first.len() }
    } else {
        // No columns and no tag — degenerate case, length is 0
        quote! { 0usize }
    };

    let tag_pair = if let Some(ref tag_name) = attrs.tag {
        quote! { (#tag_name, ::miniextendr_api::IntoR::into_sexp(self._tag)), }
    } else {
        TokenStream::new()
    };

    let col_pairs: Vec<TokenStream> = columns
        .iter()
        .map(|col| {
            let name = &col.col_name;
            let name_str = name.to_string();
            quote! { (#name_str, ::miniextendr_api::IntoR::into_sexp(self.#name)) }
        })
        .collect();

    // Length checks for all columns
    let length_checks: Vec<TokenStream> = columns
        .iter()
        .map(|col| {
            let name = &col.col_name;
            let name_str = name.to_string();
            quote! {
                assert!(
                    self.#name.len() == _n_rows,
                    "column length mismatch in {}: column `{}` has length {} but expected {}",
                    stringify!(#df_name),
                    #name_str,
                    self.#name.len(),
                    _n_rows,
                );
            }
        })
        .collect();

    let into_dataframe_impl = if has_enum_auto_expand {
        // Dynamic pair building for auto-expand fields.
        let tag_push_pair = if let Some(ref tag_name) = attrs.tag {
            quote! {
                __df_pairs.push((
                    #tag_name.to_string(),
                    ::miniextendr_api::IntoR::into_sexp(self._tag),
                ));
            }
        } else {
            TokenStream::new()
        };

        let static_pair_pushes: Vec<TokenStream> = columns
            .iter()
            .map(|col| {
                let name = &col.col_name;
                let name_str = name.to_string();
                quote! {
                    __df_pairs.push((
                        #name_str.to_string(),
                        ::miniextendr_api::IntoR::into_sexp(self.#name),
                    ));
                }
            })
            .collect();

        let auto_expand_pair_pushes: Vec<TokenStream> = auto_expand_cols
            .iter()
            .map(|ac| {
                let df_field = &ac.df_field;
                let base_name_str = &ac.base_name;
                let elem_ty = &ac.elem_ty;
                quote! {
                    {
                        let __auto = self.#df_field;
                        let __max = __auto.iter()
                            .filter_map(|v| v.as_ref())
                            .map(|v| v.len())
                            .max()
                            .unwrap_or(0);
                        let mut __cols: Vec<Vec<Option<#elem_ty>>> = (0..__max)
                            .map(|_| Vec::with_capacity(_n_rows))
                            .collect();
                        for __opt_vec in &__auto {
                            for (__i, __col) in __cols.iter_mut().enumerate() {
                                __col.push(
                                    __opt_vec.as_ref().and_then(|v| v.get(__i).cloned()),
                                );
                            }
                        }
                        for (__i, __col) in __cols.into_iter().enumerate() {
                            __df_pairs.push((
                                format!("{}_{}", #base_name_str, __i + 1),
                                ::miniextendr_api::IntoR::into_sexp(__col),
                            ));
                        }
                    }
                }
            })
            .collect();

        quote! {
            impl ::miniextendr_api::convert::IntoDataFrame for #df_name {
                fn into_data_frame(self) -> ::miniextendr_api::List {
                    let _n_rows = #length_ref;
                    #(#length_checks)*
                    let mut __df_pairs: Vec<(
                        String,
                        ::miniextendr_api::ffi::SEXP,
                    )> = Vec::new();
                    #tag_push_pair
                    #(#static_pair_pushes)*
                    #(#auto_expand_pair_pushes)*
                    ::miniextendr_api::list::List::from_raw_pairs(__df_pairs)
                        .set_class_str(&["data.frame"])
                        .set_row_names_int(_n_rows)
                }
            }
        }
    } else {
        quote! {
            impl ::miniextendr_api::convert::IntoDataFrame for #df_name {
                fn into_data_frame(self) -> ::miniextendr_api::List {
                    let _n_rows = #length_ref;
                    #(#length_checks)*
                    ::miniextendr_api::list::List::from_raw_pairs(vec![
                        #tag_pair
                        #(#col_pairs),*
                    ])
                    .set_class_str(&["data.frame"])
                    .set_row_names_int(_n_rows)
                }
            }
        }
    };

    // ── Generate From<Vec<Enum>> ─────────────────────────────────────────
    let mut col_vec_inits: Vec<TokenStream> = columns
        .iter()
        .map(|col| {
            let name = &col.col_name;
            let ty = &col.ty;
            quote! { let mut #name: Vec<Option<#ty>> = Vec::with_capacity(len); }
        })
        .collect();
    for ac in &auto_expand_cols {
        let name = &ac.df_field;
        let ty = &ac.elem_ty;
        col_vec_inits.push(
            quote! { let mut #name: Vec<Option<Vec<#ty>>> = Vec::with_capacity(len); },
        );
    }

    let tag_init = if has_tag {
        quote! { let mut _tag: Vec<String> = Vec::with_capacity(len); }
    } else {
        TokenStream::new()
    };

    // Build match arms for each variant
    let match_arms: Vec<TokenStream> = variant_infos
        .iter()
        .enumerate()
        .map(|(variant_idx, vi)| {
            let variant_name = &vi.name;
            let variant_name_str = variant_name.to_string();

            let tag_push = if has_tag {
                quote! { _tag.push(#variant_name_str.to_string()); }
            } else {
                TokenStream::new()
            };

            // Build push statements for each schema column.
            // For present columns: push Some(value), for absent: push None.
            // Expanded fields contribute multiple columns from one binding.

            // First, build a map of which schema columns this variant contributes to.
            let col_pushes: Vec<TokenStream> = columns
                .iter()
                .map(|col| {
                    let col_name = &col.col_name;
                    if col.present_in.contains(&variant_idx) {
                        let col_name_str = col_name.to_string();

                        for erf in &vi.fields {
                            match erf {
                                EnumResolvedField::Single { col_name: fc, binding, .. }
                                    if fc == col_name =>
                                {
                                    if col.string_coerced {
                                        return quote! { #col_name.push(Some(ToString::to_string(&#binding))); };
                                    } else {
                                        return quote! { #col_name.push(Some(#binding)); };
                                    }
                                }
                                EnumResolvedField::ExpandedFixed { base_name, binding, len, .. } => {
                                    for i in 1..=*len {
                                        let expanded_name = format!("{}_{}", base_name, i);
                                        if expanded_name == col_name_str {
                                            let idx = syn::Index::from(i - 1);
                                            return quote! { #col_name.push(Some(#binding[#idx])); };
                                        }
                                    }
                                }
                                EnumResolvedField::ExpandedVec { base_name, binding, width, .. } => {
                                    for i in 1..=*width {
                                        let expanded_name = format!("{}_{}", base_name, i);
                                        if expanded_name == col_name_str {
                                            let get_idx = i - 1;
                                            return quote! { #col_name.push(#binding.get(#get_idx).cloned()); };
                                        }
                                    }
                                }
                                // AutoExpandVec doesn't contribute to static columns
                                _ => {}
                            }
                        }
                        quote! { #col_name.push(None); }
                    } else {
                        quote! { #col_name.push(None); }
                    }
                })
                .collect();

            // Auto-expand push statements
            let auto_expand_pushes: Vec<TokenStream> = auto_expand_cols
                .iter()
                .map(|ac| {
                    let ac_field = &ac.df_field;
                    if ac.present_in.contains(&variant_idx) {
                        // Find the binding for this auto-expand field
                        for erf in &vi.fields {
                            if let EnumResolvedField::AutoExpandVec {
                                base_name, binding, ..
                            } = erf
                                && base_name == &ac.base_name
                            {
                                return quote! { #ac_field.push(Some(#binding)); };
                            }
                        }
                        // shouldn't reach here
                        quote! { #ac_field.push(None); }
                    } else {
                        quote! { #ac_field.push(None); }
                    }
                })
                .collect();

            // Generate destructure pattern based on variant shape
            match vi.shape {
                VariantShape::Named => {
                    let mut field_bindings: Vec<TokenStream> = vi.fields.iter().map(|erf| {
                        let (rust_name, binding) = match erf {
                            EnumResolvedField::Single { rust_name, binding, .. }
                            | EnumResolvedField::ExpandedFixed { rust_name, binding, .. }
                            | EnumResolvedField::ExpandedVec { rust_name, binding, .. }
                            | EnumResolvedField::AutoExpandVec { rust_name, binding, .. } => {
                                (rust_name, binding)
                            }
                        };
                        quote! { #rust_name: #binding }
                    }).collect();
                    // Add skipped fields as wildcard bindings
                    for skipped in &vi.skipped_fields {
                        field_bindings.push(quote! { #skipped: _ });
                    }
                    quote! {
                        #row_name::#variant_name { #(#field_bindings),* } => {
                            #tag_push
                            #(#col_pushes)*
                            #(#auto_expand_pushes)*
                        }
                    }
                }
                VariantShape::Tuple => {
                    let field_bindings: Vec<TokenStream> = vi.fields.iter().map(|erf| {
                        let binding = match erf {
                            EnumResolvedField::Single { binding, .. }
                            | EnumResolvedField::ExpandedFixed { binding, .. }
                            | EnumResolvedField::ExpandedVec { binding, .. }
                            | EnumResolvedField::AutoExpandVec { binding, .. } => binding,
                        };
                        quote! { #binding }
                    }).collect();
                    quote! {
                        #row_name::#variant_name(#(#field_bindings),*) => {
                            #tag_push
                            #(#col_pushes)*
                            #(#auto_expand_pushes)*
                        }
                    }
                }
                VariantShape::Unit => {
                    quote! {
                        #row_name::#variant_name => {
                            #tag_push
                            #(#col_pushes)*
                            #(#auto_expand_pushes)*
                        }
                    }
                }
            }
        })
        .collect();

    let tag_struct_field = if has_tag {
        quote! { _tag, }
    } else {
        TokenStream::new()
    };

    let mut col_struct_fields: Vec<TokenStream> = columns
        .iter()
        .map(|col| {
            let name = &col.col_name;
            quote! { #name }
        })
        .collect();
    for ac in &auto_expand_cols {
        let name = &ac.df_field;
        col_struct_fields.push(quote! { #name });
    }

    // Skip parallel path when non-named variants or expansion is present
    let has_non_named = variant_infos
        .iter()
        .any(|vi| vi.shape != VariantShape::Named);
    let has_enum_expansion = variant_infos.iter().any(|vi| {
        vi.fields
            .iter()
            .any(|erf| !matches!(erf, EnumResolvedField::Single { .. }))
    });
    let parallel_block = if attrs.parallel && !has_non_named && !has_enum_expansion {
        // Convert to old-style variant_infos for parallel gen (only named, non-expanded variants)
        let old_variant_infos: Vec<(&syn::Ident, Vec<(&syn::Ident, &syn::Type)>)> = variant_infos
            .iter()
            .map(|vi| {
                let fields: Vec<_> = vi
                    .fields
                    .iter()
                    .filter_map(|erf| {
                        if let EnumResolvedField::Single { rust_name, ty, .. } = erf {
                            Some((rust_name, ty))
                        } else {
                            None
                        }
                    })
                    .collect();
                (&vi.name, fields)
            })
            .collect();
        gen_parallel_enum_from(row_name, df_name, &columns, &old_variant_infos, has_tag)
    } else {
        TokenStream::new()
    };

    let from_vec_impl = quote! {
        impl From<Vec<#row_name>> for #df_name {
            fn from(rows: Vec<#row_name>) -> Self {
                let len = rows.len();
                #parallel_block
                #tag_init
                #(#col_vec_inits)*
                for row in rows {
                    match row {
                        #(#match_arms)*
                    }
                }
                #df_name {
                    #tag_struct_field
                    #(#col_struct_fields),*
                }
            }
        }
    };

    // ── Generate associated methods ──────────────────────────────────────
    let row_methods = quote! {
        impl #row_name {
            /// Name of the generated DataFrame companion type.
            pub const DATAFRAME_TYPE_NAME: &'static str = stringify!(#df_name);

            /// Convert a vector of enum rows into the companion DataFrame type.
            ///
            /// Fields present in a variant get `Some(value)`, absent fields get `None` (→ NA in R).
            pub fn to_dataframe(rows: Vec<Self>) -> #df_name {
                rows.into()
            }
        }
    };

    // No IntoList assertion for align enums — they go through the companion struct path,
    // not the `DataFrame<T>` path, so IntoList is not required.

    Ok(quote! {
        #dataframe_struct
        #into_dataframe_impl
        #from_vec_impl
        #row_methods
    })
}

// =============================================================================
// Parallel fill codegen (rayon feature-gated)
// =============================================================================

/// Generate a `#[cfg(feature = "rayon")]` block that does parallel scatter-write
/// for struct DataFrameRow, returning early if above threshold.
pub(super) fn gen_parallel_struct_from(
    _row_name: &syn::Ident,
    df_name: &syn::Ident,
    field_info: &[(&syn::Ident, &syn::Type)],
) -> TokenStream {
    // Column declarations: `let mut field: Vec<Type> = Vec::with_capacity(len);`
    let par_col_decls: Vec<TokenStream> = field_info
        .iter()
        .map(|(name, ty)| {
            quote! {
                let mut #name: Vec<#ty> = Vec::with_capacity(len);
                unsafe { #name.set_len(len); }
            }
        })
        .collect();

    // Writer declarations: `let w_field = unsafe { ColumnWriter::new(&mut field) };`
    let writer_decls: Vec<TokenStream> = field_info
        .iter()
        .map(|(name, _ty)| {
            let w_name = format_ident!("__w_{}", name);
            quote! {
                let #w_name = unsafe {
                    ::miniextendr_api::rayon_bridge::ColumnWriter::new(&mut #name)
                };
            }
        })
        .collect();

    // Write calls inside for_each: `w_field.write(i, row.field);`
    let write_calls: Vec<TokenStream> = field_info
        .iter()
        .map(|(name, _ty)| {
            let w_name = format_ident!("__w_{}", name);
            quote! { #w_name.write(__i, __row.#name); }
        })
        .collect();

    // Struct fields for return
    let struct_fields: Vec<TokenStream> = field_info
        .iter()
        .map(|(name, _ty)| quote! { #name })
        .collect();

    quote! {
        #[cfg(feature = "rayon")]
        {
            #[allow(clippy::uninit_vec)]
            if len >= ::miniextendr_api::rayon_bridge::PARALLEL_FILL_THRESHOLD {
                use ::miniextendr_api::rayon_bridge::rayon::prelude::*;
                #(#par_col_decls)*
                {
                    #(#writer_decls)*
                    rows.into_par_iter().enumerate().for_each(|(__i, __row)| unsafe {
                        #(#write_calls)*
                    });
                }
                return #df_name { #(#struct_fields),* };
            }
        }
    }
}

/// Generate a `#[cfg(feature = "rayon")]` block that does parallel scatter-write
/// for enum DataFrameRow, returning early if above threshold.
fn gen_parallel_enum_from(
    row_name: &syn::Ident,
    df_name: &syn::Ident,
    columns: &[ResolvedColumn],
    variant_infos: &[(&syn::Ident, Vec<(&syn::Ident, &syn::Type)>)],
    has_tag: bool,
) -> TokenStream {
    // Tag column declaration
    let tag_decl = if has_tag {
        quote! {
            let mut _tag: Vec<String> = Vec::with_capacity(len);
            unsafe { _tag.set_len(len); }
        }
    } else {
        TokenStream::new()
    };

    // Column declarations with Option<T>
    let par_col_decls: Vec<TokenStream> = columns
        .iter()
        .map(|col| {
            let name = &col.col_name;
            let ty = &col.ty;
            quote! {
                let mut #name: Vec<Option<#ty>> = Vec::with_capacity(len);
                unsafe { #name.set_len(len); }
            }
        })
        .collect();

    // Tag writer
    let tag_writer_decl = if has_tag {
        quote! {
            let __w_tag = unsafe {
                ::miniextendr_api::rayon_bridge::ColumnWriter::new(&mut _tag)
            };
        }
    } else {
        TokenStream::new()
    };

    // Column writers
    let writer_decls: Vec<TokenStream> = columns
        .iter()
        .map(|col| {
            let name = &col.col_name;
            let w_name = format_ident!("__w_{}", name);
            quote! {
                let #w_name = unsafe {
                    ::miniextendr_api::rayon_bridge::ColumnWriter::new(&mut #name)
                };
            }
        })
        .collect();

    // Match arms for parallel path
    let par_match_arms: Vec<TokenStream> = variant_infos
        .iter()
        .enumerate()
        .map(|(variant_idx, (variant_name, fields))| {
            let variant_name_str = variant_name.to_string();

            let field_bindings: Vec<TokenStream> = fields
                .iter()
                .map(|(fname, _)| {
                    let binding = format_ident!("__v_{}", fname);
                    quote! { #fname: #binding }
                })
                .collect();

            let tag_write = if has_tag {
                quote! { __w_tag.write(__i, #variant_name_str.to_string()); }
            } else {
                TokenStream::new()
            };

            let col_writes: Vec<TokenStream> = columns
                .iter()
                .map(|col| {
                    let w_name = format_ident!("__w_{}", col.col_name);
                    if col.present_in.contains(&variant_idx) {
                        let binding = format_ident!("__v_{}", col.col_name);
                        quote! { #w_name.write(__i, Some(#binding)); }
                    } else {
                        quote! { #w_name.write(__i, None); }
                    }
                })
                .collect();

            quote! {
                #row_name::#variant_name { #(#field_bindings),* } => {
                    #tag_write
                    #(#col_writes)*
                }
            }
        })
        .collect();

    // Return struct fields
    let tag_field = if has_tag {
        quote! { _tag, }
    } else {
        TokenStream::new()
    };

    let struct_fields: Vec<TokenStream> = columns
        .iter()
        .map(|col| {
            let name = &col.col_name;
            quote! { #name }
        })
        .collect();

    quote! {
        #[cfg(feature = "rayon")]
        {
            #[allow(clippy::uninit_vec)]
            if len >= ::miniextendr_api::rayon_bridge::PARALLEL_FILL_THRESHOLD {
                use ::miniextendr_api::rayon_bridge::rayon::prelude::*;
                #tag_decl
                #(#par_col_decls)*
                {
                    #tag_writer_decl
                    #(#writer_decls)*
                    rows.into_par_iter().enumerate().for_each(|(__i, __row)| unsafe {
                        match __row {
                            #(#par_match_arms)*
                        }
                    });
                }
                return #df_name { #tag_field #(#struct_fields),* };
            }
        }
    }
}
