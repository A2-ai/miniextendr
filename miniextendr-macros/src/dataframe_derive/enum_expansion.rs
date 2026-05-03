//! Enum-specific DataFrame derive expansion.
//!
//! Generates a companion struct where every column is `Vec<Option<T>>`, with
//! `None` fill for fields absent in a given variant.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, Fields};

use super::{
    ColumnRegistry, DataFrameAttrs, EnumAutoExpandVecData, EnumExpandedFixedData,
    EnumExpandedVecData, EnumResolvedField, EnumSingleFieldData, FieldTypeKind, VariantInfo,
    VariantShape, classify_field_type, parse_field_attrs,
};
use crate::naming;
use std::collections::HashMap;

/// Derive `DataFrameRow` for an enum with `#[dataframe(align)]`.
///
/// Generates a companion struct where every column is `Vec<Option<T>>`, with
/// `None` fill for fields absent in a given variant. This is the enum counterpart
/// of [`super::derive_struct_dataframe`].
///
/// # Generated items
///
/// - Companion struct `{Name}DataFrame` with `Vec<Option<T>>` columns (field-name union)
/// - Optional `_tag: Vec<String>` column for variant discrimination
/// - `impl IntoDataFrame` (converts companion struct to R data.frame)
/// - `impl From<Vec<Enum>>` (sequential row->column transposition)
/// - `from_rows()` / `from_rows_par()` methods on the companion struct
/// - `to_dataframe()` / `DATAFRAME_TYPE_NAME` associated items on the enum
///
/// # Variant support
///
/// - Named variants (`{ field: T }`): fields contribute by name to the unified schema
/// - Tuple variants (`(T, U)`): fields are named `_0`, `_1`, etc.
/// - Unit variants: contribute no columns (only tag if present)
///
/// # Auto-expand fields
///
/// Fields with `#[dataframe(expand)]` on `Vec<T>` types get dynamic column counts
/// determined at runtime from the maximum row length across all rows. These are
/// tracked separately from the static [`ColumnRegistry`](super::ColumnRegistry).
///
/// Returns `Err` if the enum has no variants or if type conflicts arise without
/// `#[dataframe(conflicts = "string")]`.
pub(super) fn derive_enum_dataframe(
    row_name: &syn::Ident,
    input: &DeriveInput,
    data: &syn::DataEnum,
    df_name: &syn::Ident,
    attrs: &DataFrameAttrs,
) -> syn::Result<TokenStream> {
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // region: Validate variants
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
                        resolved.push(EnumResolvedField::Single(Box::new(EnumSingleFieldData {
                            col_name: format_ident!("{}", col_name_str),
                            binding: binding.clone(),
                            rust_name: rust_name.clone(),
                            ty: f.ty.clone(),
                        })));
                    } else {
                        match classify_field_type(&f.ty) {
                            FieldTypeKind::FixedArray(elem_ty, len) => {
                                resolved.push(EnumResolvedField::ExpandedFixed(Box::new(
                                    EnumExpandedFixedData {
                                        base_name: col_name_str,
                                        binding: binding.clone(),
                                        rust_name: rust_name.clone(),
                                        elem_ty: elem_ty.clone(),
                                        len,
                                    },
                                )));
                            }
                            FieldTypeKind::VariableVec(elem_ty)
                            | FieldTypeKind::BoxedSlice(elem_ty)
                            | FieldTypeKind::BorrowedSlice(elem_ty) => {
                                if let Some(width) = fa.width {
                                    resolved.push(EnumResolvedField::ExpandedVec(Box::new(
                                        EnumExpandedVecData {
                                            base_name: col_name_str,
                                            binding: binding.clone(),
                                            rust_name: rust_name.clone(),
                                            elem_ty: elem_ty.clone(),
                                            width,
                                        },
                                    )));
                                } else if fa.expand {
                                    resolved.push(EnumResolvedField::AutoExpandVec(Box::new(
                                        EnumAutoExpandVecData {
                                            base_name: col_name_str,
                                            binding: binding.clone(),
                                            rust_name: rust_name.clone(),
                                            elem_ty: elem_ty.clone(),
                                            container_ty: f.ty.clone(),
                                        },
                                    )));
                                } else {
                                    resolved.push(EnumResolvedField::Single(Box::new(
                                        EnumSingleFieldData {
                                            col_name: format_ident!("{}", col_name_str),
                                            binding: binding.clone(),
                                            rust_name: rust_name.clone(),
                                            ty: f.ty.clone(),
                                        },
                                    )));
                                }
                            }
                            FieldTypeKind::Scalar => {
                                if fa.width.is_some() {
                                    return Err(syn::Error::new_spanned(
                                        &f.ty,
                                        "`width` is only valid on `Vec<T>`, `Box<[T]>`, or `&[T]` fields",
                                    ));
                                }
                                if fa.expand {
                                    return Err(syn::Error::new_spanned(
                                        &f.ty,
                                        "`expand`/`unnest` is only valid on `[T; N]`, `Vec<T>`, `Box<[T]>`, or `&[T]` fields",
                                    ));
                                }
                                resolved.push(EnumResolvedField::Single(Box::new(
                                    EnumSingleFieldData {
                                        col_name: format_ident!("{}", col_name_str),
                                        binding: binding.clone(),
                                        rust_name: rust_name.clone(),
                                        ty: f.ty.clone(),
                                    },
                                )));
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
                        resolved.push(EnumResolvedField::Single(Box::new(EnumSingleFieldData {
                            col_name: format_ident!("{}", col_name_str),
                            binding,
                            rust_name,
                            ty: f.ty.clone(),
                        })));
                    } else {
                        match classify_field_type(&f.ty) {
                            FieldTypeKind::FixedArray(elem_ty, len) => {
                                resolved.push(EnumResolvedField::ExpandedFixed(Box::new(
                                    EnumExpandedFixedData {
                                        base_name: col_name_str,
                                        binding,
                                        rust_name,
                                        elem_ty: elem_ty.clone(),
                                        len,
                                    },
                                )));
                            }
                            FieldTypeKind::VariableVec(elem_ty)
                            | FieldTypeKind::BoxedSlice(elem_ty)
                            | FieldTypeKind::BorrowedSlice(elem_ty) => {
                                if let Some(width) = fa.width {
                                    resolved.push(EnumResolvedField::ExpandedVec(Box::new(
                                        EnumExpandedVecData {
                                            base_name: col_name_str,
                                            binding,
                                            rust_name,
                                            elem_ty: elem_ty.clone(),
                                            width,
                                        },
                                    )));
                                } else if fa.expand {
                                    resolved.push(EnumResolvedField::AutoExpandVec(Box::new(
                                        EnumAutoExpandVecData {
                                            base_name: col_name_str,
                                            binding,
                                            rust_name,
                                            elem_ty: elem_ty.clone(),
                                            container_ty: f.ty.clone(),
                                        },
                                    )));
                                } else {
                                    resolved.push(EnumResolvedField::Single(Box::new(
                                        EnumSingleFieldData {
                                            col_name: format_ident!("{}", col_name_str),
                                            binding,
                                            rust_name,
                                            ty: f.ty.clone(),
                                        },
                                    )));
                                }
                            }
                            FieldTypeKind::Scalar => {
                                resolved.push(EnumResolvedField::Single(Box::new(
                                    EnumSingleFieldData {
                                        col_name: format_ident!("{}", col_name_str),
                                        binding,
                                        rust_name,
                                        ty: f.ty.clone(),
                                    },
                                )));
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
    // endregion

    // region: Resolve unified schema
    // Collect all unique column names, check type consistency.
    // Expanded fields contribute multiple columns to the schema.
    let coerce_to_string = attrs.conflicts.as_deref() == Some("string");
    let string_ty: syn::Type = syn::parse_quote!(String);
    let mut registry = ColumnRegistry::new(coerce_to_string, &string_ty);

    for (variant_idx, vi) in variant_infos.iter().enumerate() {
        for erf in &vi.fields {
            // Use the rust_name span for error reporting
            let err_span = erf.rust_name().span();
            match erf {
                EnumResolvedField::Single(data) => {
                    registry.register(
                        &data.col_name.to_string(),
                        &data.ty,
                        variant_idx,
                        &vi.name,
                        err_span,
                    )?;
                }
                EnumResolvedField::ExpandedFixed(data) => {
                    for i in 1..=data.len {
                        let name = format!("{}_{}", data.base_name, i);
                        registry.register(&name, &data.elem_ty, variant_idx, &vi.name, err_span)?;
                    }
                }
                EnumResolvedField::ExpandedVec(data) => {
                    for i in 1..=data.width {
                        let name = format!("{}_{}", data.base_name, i);
                        registry.register(&name, &data.elem_ty, variant_idx, &vi.name, err_span)?;
                    }
                }
                // AutoExpandVec: not registered in ColumnRegistry (width is dynamic).
                // Collected separately below.
                EnumResolvedField::AutoExpandVec(..) => {}
            }
        }
    }
    let columns = registry.columns;
    // endregion

    // region: Collect auto-expand fields (per-variant, for split method)
    struct EnumAutoExpandCol {
        df_field: syn::Ident,
        base_name: String,
        elem_ty: syn::Type,
        container_ty: syn::Type,
        present_in: Vec<usize>,
    }

    let mut auto_expand_cols: Vec<EnumAutoExpandCol> = Vec::new();
    let mut auto_expand_index: HashMap<String, usize> = HashMap::new();

    for (variant_idx, vi) in variant_infos.iter().enumerate() {
        for erf in &vi.fields {
            if let EnumResolvedField::AutoExpandVec(auto_data) = erf {
                if let Some(&idx) = auto_expand_index.get(&auto_data.base_name) {
                    let elem_match = auto_expand_cols[idx].elem_ty == auto_data.elem_ty;
                    let container_match =
                        auto_expand_cols[idx].container_ty == auto_data.container_ty;
                    if !elem_match {
                        if coerce_to_string {
                            auto_expand_cols[idx].elem_ty = string_ty.clone();
                        } else {
                            return Err(syn::Error::new(
                                auto_data.rust_name.span(),
                                format!(
                                    "type conflict for auto-expand field `{}`: different element type \
                                     than a previous variant; \
                                     use `#[dataframe(conflicts = \"string\")]` to coerce",
                                    auto_data.base_name,
                                ),
                            ));
                        }
                    }
                    if !container_match {
                        return Err(syn::Error::new(
                            auto_data.rust_name.span(),
                            format!(
                                "container type mismatch for auto-expand field `{}`: \
                                 all variants must use the same container type \
                                 (`Vec<T>`, `Box<[T]>`, or `&[T]`)",
                                auto_data.base_name,
                            ),
                        ));
                    }
                    auto_expand_cols[idx].present_in.push(variant_idx);
                } else {
                    let idx = auto_expand_cols.len();
                    auto_expand_cols.push(EnumAutoExpandCol {
                        df_field: format_ident!("{}", auto_data.base_name),
                        base_name: auto_data.base_name.clone(),
                        elem_ty: auto_data.elem_ty.clone(),
                        container_ty: auto_data.container_ty.clone(),
                        present_in: vec![variant_idx],
                    });
                    auto_expand_index.insert(auto_data.base_name.clone(), idx);
                }
            }
        }
    }
    let has_enum_auto_expand = !auto_expand_cols.is_empty();
    // endregion

    // region: Generate companion struct
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
    // Auto-expand fields: Vec<Option<ContainerType>>
    for ac in &auto_expand_cols {
        let name = &ac.df_field;
        let cty = &ac.container_ty;
        df_fields.push(quote! { pub #name: Vec<Option<#cty>> });
    }

    let dataframe_struct = quote! {
        #[derive(Debug, Clone)]
        pub struct #df_name #impl_generics #where_clause {
            #tag_field
            #(#df_fields),*
        }
    };
    // endregion

    // region: Generate IntoDataFrame
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
            impl #impl_generics ::miniextendr_api::convert::IntoDataFrame for #df_name #ty_generics #where_clause {
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
            impl #impl_generics ::miniextendr_api::convert::IntoDataFrame for #df_name #ty_generics #where_clause {
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
    // endregion

    // region: Generate From<Vec<Enum>>
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
        let cty = &ac.container_ty;
        col_vec_inits.push(quote! { let mut #name: Vec<Option<#cty>> = Vec::with_capacity(len); });
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
                                EnumResolvedField::Single(data)
                                    if data.col_name == *col_name =>
                                {
                                    let binding = &data.binding;
                                    if col.string_coerced {
                                        return quote! { #col_name.push(Some(ToString::to_string(&#binding))); };
                                    } else {
                                        return quote! { #col_name.push(Some(#binding)); };
                                    }
                                }
                                EnumResolvedField::ExpandedFixed(data) => {
                                    for i in 1..=data.len {
                                        let expanded_name = format!("{}_{}", data.base_name, i);
                                        if expanded_name == col_name_str {
                                            let binding = &data.binding;
                                            let idx = syn::Index::from(i - 1);
                                            return quote! { #col_name.push(Some(#binding[#idx])); };
                                        }
                                    }
                                }
                                EnumResolvedField::ExpandedVec(data) => {
                                    for i in 1..=data.width {
                                        let expanded_name = format!("{}_{}", data.base_name, i);
                                        if expanded_name == col_name_str {
                                            let binding = &data.binding;
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
                            if let EnumResolvedField::AutoExpandVec(data) = erf
                                && data.base_name == ac.base_name
                            {
                                let binding = &data.binding;
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
                        let rust_name = erf.rust_name();
                        let binding = erf.binding();
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
                        let binding = erf.binding();
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

    let from_vec_impl = quote! {
        impl #impl_generics From<Vec<#row_name #ty_generics>> for #df_name #ty_generics #where_clause {
            fn from(rows: Vec<#row_name #ty_generics>) -> Self {
                let len = rows.len();
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
    // endregion

    // region: Generate from_rows_par (parallel scatter-write via ColumnWriter)
    let from_rows_par_method = if !columns.is_empty() || !auto_expand_cols.is_empty() || has_tag {
        // Column declarations
        let mut par_col_decls = Vec::new();
        if has_tag {
            par_col_decls.push(quote! {
                let mut _tag: Vec<String> = vec![String::new(); len];
            });
        }
        for col in &columns {
            let name = &col.col_name;
            let ty = &col.ty;
            par_col_decls.push(quote! {
                let mut #name: Vec<Option<#ty>> = vec![None; len];
            });
        }
        for ac in &auto_expand_cols {
            let name = &ac.df_field;
            let cty = &ac.container_ty;
            par_col_decls.push(quote! {
                let mut #name: Vec<Option<#cty>> = vec![None; len];
            });
        }

        // Writer declarations
        let mut writer_decls = Vec::new();
        if has_tag {
            writer_decls.push(quote! {
                let __w_tag = unsafe {
                    ::miniextendr_api::rayon_bridge::ColumnWriter::new(&mut _tag)
                };
            });
        }
        for col in &columns {
            let name = &col.col_name;
            let w_name = format_ident!("__w_{}", name);
            writer_decls.push(quote! {
                let #w_name = unsafe {
                    ::miniextendr_api::rayon_bridge::ColumnWriter::new(&mut #name)
                };
            });
        }
        for ac in &auto_expand_cols {
            let name = &ac.df_field;
            let w_name = format_ident!("__w_{}", name);
            writer_decls.push(quote! {
                let #w_name = unsafe {
                    ::miniextendr_api::rayon_bridge::ColumnWriter::new(&mut #name)
                };
            });
        }

        // Match arms for parallel path
        let par_match_arms: Vec<TokenStream> = variant_infos
            .iter()
            .enumerate()
            .map(|(variant_idx, vi)| {
                let variant_name = &vi.name;
                let variant_name_str = variant_name.to_string();

                let tag_write = if has_tag {
                    quote! { __w_tag.write(__i, #variant_name_str.to_string()); }
                } else {
                    TokenStream::new()
                };

                // Write calls for schema columns
                let col_writes: Vec<TokenStream> = columns
                    .iter()
                    .map(|col| {
                        let col_name = &col.col_name;
                        let w_name = format_ident!("__w_{}", col_name);
                        if col.present_in.contains(&variant_idx) {
                            let col_name_str = col_name.to_string();
                            for erf in &vi.fields {
                                match erf {
                                    EnumResolvedField::Single(data)
                                        if data.col_name == *col_name =>
                                    {
                                        let binding = &data.binding;
                                        if col.string_coerced {
                                            return quote! { #w_name.write(__i, Some(ToString::to_string(&#binding))); };
                                        } else {
                                            return quote! { #w_name.write(__i, Some(#binding)); };
                                        }
                                    }
                                    EnumResolvedField::ExpandedFixed(data) => {
                                        for i in 1..=data.len {
                                            let expanded_name = format!("{}_{}", data.base_name, i);
                                            if expanded_name == col_name_str {
                                                let binding = &data.binding;
                                                let idx = syn::Index::from(i - 1);
                                                return quote! { #w_name.write(__i, Some(#binding[#idx])); };
                                            }
                                        }
                                    }
                                    EnumResolvedField::ExpandedVec(data) => {
                                        for i in 1..=data.width {
                                            let expanded_name = format!("{}_{}", data.base_name, i);
                                            if expanded_name == col_name_str {
                                                let binding = &data.binding;
                                                let get_idx = i - 1;
                                                return quote! { #w_name.write(__i, #binding.get(#get_idx).cloned()); };
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            quote! { #w_name.write(__i, None); }
                        } else {
                            quote! { #w_name.write(__i, None); }
                        }
                    })
                    .collect();

                // Auto-expand write calls
                let auto_expand_writes: Vec<TokenStream> = auto_expand_cols
                    .iter()
                    .map(|ac| {
                        let w_name = format_ident!("__w_{}", ac.df_field);
                        if ac.present_in.contains(&variant_idx) {
                            for erf in &vi.fields {
                                if let EnumResolvedField::AutoExpandVec(data) = erf
                                    && data.base_name == ac.base_name
                                {
                                    let binding = &data.binding;
                                    return quote! { #w_name.write(__i, Some(#binding)); };
                                }
                            }
                            quote! { #w_name.write(__i, None); }
                        } else {
                            quote! { #w_name.write(__i, None); }
                        }
                    })
                    .collect();

                // Generate destructure pattern based on variant shape
                match vi.shape {
                    VariantShape::Named => {
                        let mut field_bindings: Vec<TokenStream> = vi.fields.iter().map(|erf| {
                            let rust_name = erf.rust_name();
                            let binding = erf.binding();
                            quote! { #rust_name: #binding }
                        }).collect();
                        for skipped in &vi.skipped_fields {
                            field_bindings.push(quote! { #skipped: _ });
                        }
                        quote! {
                            #row_name::#variant_name { #(#field_bindings),* } => {
                                #tag_write
                                #(#col_writes)*
                                #(#auto_expand_writes)*
                            }
                        }
                    }
                    VariantShape::Tuple => {
                        let field_bindings: Vec<TokenStream> = vi.fields.iter().map(|erf| {
                            let binding = erf.binding();
                            quote! { #binding }
                        }).collect();
                        quote! {
                            #row_name::#variant_name(#(#field_bindings),*) => {
                                #tag_write
                                #(#col_writes)*
                                #(#auto_expand_writes)*
                            }
                        }
                    }
                    VariantShape::Unit => {
                        quote! {
                            #row_name::#variant_name => {
                                #tag_write
                                #(#col_writes)*
                                #(#auto_expand_writes)*
                            }
                        }
                    }
                }
            })
            .collect();

        // Return struct fields
        let par_tag_field = if has_tag {
            quote! { _tag, }
        } else {
            TokenStream::new()
        };
        let mut par_struct_fields: Vec<TokenStream> = columns
            .iter()
            .map(|col| {
                let name = &col.col_name;
                quote! { #name }
            })
            .collect();
        for ac in &auto_expand_cols {
            let name = &ac.df_field;
            par_struct_fields.push(quote! { #name });
        }

        quote! {
            /// Parallel row→column transposition using rayon scatter-write.
            ///
            /// Always uses rayon — no threshold check. Use `from_rows` for the
            /// sequential path.
            #[cfg(feature = "rayon")]
            #[allow(clippy::uninit_vec)]
            pub fn from_rows_par(rows: Vec<#row_name #ty_generics>) -> Self {
                use ::miniextendr_api::rayon_bridge::rayon::prelude::*;
                let len = rows.len();
                #(#par_col_decls)*
                {
                    #(#writer_decls)*
                    rows.into_par_iter().enumerate().for_each(|(__i, __row)| unsafe {
                        match __row {
                            #(#par_match_arms)*
                        }
                    });
                }
                #df_name { #par_tag_field #(#par_struct_fields),* }
            }
        }
    } else {
        TokenStream::new()
    };
    // endregion

    // region: Generate DataFrame type methods (from_rows, from_rows_par)
    let df_methods = quote! {
        impl #impl_generics #df_name #ty_generics #where_clause {
            /// Sequential row→column transposition.
            pub fn from_rows(rows: Vec<#row_name #ty_generics>) -> Self {
                rows.into()
            }

            #from_rows_par_method
        }
    };
    // endregion

    // region: Generate associated methods
    let row_methods = quote! {
        impl #impl_generics #row_name #ty_generics #where_clause {
            /// Name of the generated DataFrame companion type.
            pub const DATAFRAME_TYPE_NAME: &'static str = stringify!(#df_name);

            /// Convert a vector of enum rows into the companion DataFrame type.
            ///
            /// Fields present in a variant get `Some(value)`, absent fields get `None` (→ NA in R).
            pub fn to_dataframe(rows: Vec<Self>) -> #df_name #ty_generics {
                rows.into()
            }
        }
    };

    // No IntoList assertion for align enums — they go through the companion struct path,
    // not the `DataFrame<T>` path, so IntoList is not required.

    // region: Generate to_dataframe_split
    let split_method = generate_split_method(
        row_name,
        &variant_infos,
        &impl_generics,
        &ty_generics,
        where_clause,
    );
    // endregion

    Ok(quote! {
        #dataframe_struct
        #into_dataframe_impl
        #from_vec_impl
        #df_methods
        #row_methods
        #split_method
    })
    // endregion
}

// region: generate_split_method

/// Generate the `to_dataframe_split` associated method for an enum `DataFrameRow`.
///
/// For a single-variant enum, returns the data.frame directly.
/// For multi-variant enums, returns a named R list of data.frames (one per variant,
/// named with snake_case variant names). Each partition data.frame has only that
/// variant's columns (non-optional types — no NA fill from other variants).
fn generate_split_method(
    row_name: &syn::Ident,
    variant_infos: &[VariantInfo],
    impl_generics: &syn::ImplGenerics<'_>,
    ty_generics: &syn::TypeGenerics<'_>,
    where_clause: Option<&syn::WhereClause>,
) -> TokenStream {
    // Per-variant buffer declarations
    let mut buf_decls: Vec<TokenStream> = Vec::new();
    // Per-variant match arms (push to buffers)
    let mut match_arms: Vec<TokenStream> = Vec::new();
    // Per-variant data.frame construction
    let mut df_constructions: Vec<TokenStream> = Vec::new();
    // Names of the constructed data.frame variables (for the outer list)
    let mut df_var_names: Vec<syn::Ident> = Vec::new();
    // Snake-case string names (for the outer list pairs)
    let mut snake_names: Vec<String> = Vec::new();

    for vi in variant_infos {
        let variant_name = &vi.name;
        let snake = naming::to_snake_case(&variant_name.to_string());
        snake_names.push(snake.clone());

        let df_var = format_ident!("__{}_df", snake);
        df_var_names.push(df_var.clone());

        // Determine if any field is AutoExpandVec
        let has_auto = vi
            .fields
            .iter()
            .any(|f| matches!(f, EnumResolvedField::AutoExpandVec(_)));

        match vi.shape {
            // region: Unit variant
            VariantShape::Unit => {
                let count_var = format_ident!("__s_{}_count", snake);
                buf_decls.push(quote! {
                    let mut #count_var: usize = 0usize;
                });

                match_arms.push(quote! {
                    #row_name::#variant_name => {
                        #count_var += 1;
                    }
                });

                df_constructions.push(quote! {
                    let #df_var = ::miniextendr_api::list::List::from_raw_pairs(
                        Vec::<(&str, ::miniextendr_api::ffi::SEXP)>::new()
                    )
                    .set_class_str(&["data.frame"])
                    .set_row_names_int(#count_var);
                });
            }
            // endregion

            // region: Named or Tuple variants
            VariantShape::Named | VariantShape::Tuple => {
                // Declare per-field buffers
                for erf in &vi.fields {
                    match erf {
                        EnumResolvedField::Single(data) => {
                            let buf = format_ident!("__s_{}_{}", snake, data.col_name);
                            let ty = &data.ty;
                            buf_decls.push(quote! {
                                let mut #buf: Vec<#ty> = Vec::new();
                            });
                        }
                        EnumResolvedField::ExpandedFixed(data) => {
                            for i in 1..=data.len {
                                let buf = format_ident!("__s_{}_{}_{}", snake, data.base_name, i);
                                let elem_ty = &data.elem_ty;
                                buf_decls.push(quote! {
                                    let mut #buf: Vec<#elem_ty> = Vec::new();
                                });
                            }
                        }
                        EnumResolvedField::ExpandedVec(data) => {
                            for i in 1..=data.width {
                                let buf = format_ident!("__s_{}_{}_{}", snake, data.base_name, i);
                                let elem_ty = &data.elem_ty;
                                buf_decls.push(quote! {
                                    let mut #buf: Vec<Option<#elem_ty>> = Vec::new();
                                });
                            }
                        }
                        EnumResolvedField::AutoExpandVec(data) => {
                            let buf = format_ident!("__s_{}_{}", snake, data.base_name);
                            let container_ty = &data.container_ty;
                            buf_decls.push(quote! {
                                let mut #buf: Vec<#container_ty> = Vec::new();
                            });
                        }
                    }
                }

                // Build destructure pattern and push statements
                let push_stmts: Vec<TokenStream> = vi
                    .fields
                    .iter()
                    .flat_map(|erf| {
                        let binding = erf.binding();
                        match erf {
                            EnumResolvedField::Single(data) => {
                                let buf = format_ident!("__s_{}_{}", snake, data.col_name);
                                vec![quote! { #buf.push(#binding); }]
                            }
                            EnumResolvedField::ExpandedFixed(data) => (0..data.len)
                                .map(|i| {
                                    let buf =
                                        format_ident!("__s_{}_{}_{}", snake, data.base_name, i + 1);
                                    let idx = syn::Index::from(i);
                                    quote! { #buf.push(#binding[#idx]); }
                                })
                                .collect(),
                            EnumResolvedField::ExpandedVec(data) => (0..data.width)
                                .map(|i| {
                                    let buf =
                                        format_ident!("__s_{}_{}_{}", snake, data.base_name, i + 1);
                                    quote! { #buf.push(#binding.get(#i).cloned()); }
                                })
                                .collect(),
                            EnumResolvedField::AutoExpandVec(data) => {
                                let buf = format_ident!("__s_{}_{}", snake, data.base_name);
                                vec![quote! { #buf.push(#binding); }]
                            }
                        }
                    })
                    .collect();

                let arm = match vi.shape {
                    VariantShape::Named => {
                        let mut field_bindings: Vec<TokenStream> = vi
                            .fields
                            .iter()
                            .map(|erf| {
                                let rust_name = erf.rust_name();
                                let binding = erf.binding();
                                quote! { #rust_name: #binding }
                            })
                            .collect();
                        for skipped in &vi.skipped_fields {
                            field_bindings.push(quote! { #skipped: _ });
                        }
                        quote! {
                            #row_name::#variant_name { #(#field_bindings),* } => {
                                #(#push_stmts)*
                            }
                        }
                    }
                    VariantShape::Tuple => {
                        let bindings: Vec<TokenStream> = vi
                            .fields
                            .iter()
                            .map(|erf| {
                                let binding = erf.binding();
                                quote! { #binding }
                            })
                            .collect();
                        quote! {
                            #row_name::#variant_name(#(#bindings),*) => {
                                #(#push_stmts)*
                            }
                        }
                    }
                    VariantShape::Unit => unreachable!("handled above"),
                };
                match_arms.push(arm);

                // Construct the data.frame for this variant
                if has_auto {
                    // Dynamic path: build Vec<(String, SEXP)>
                    let pairs_var = format_ident!("__pairs_{}", snake);
                    let n_var = format_ident!("__n_{}", snake);

                    // Find the first non-auto field for the length expression, or first auto
                    let len_expr: TokenStream = {
                        let first_non_auto = vi
                            .fields
                            .iter()
                            .find(|f| !matches!(f, EnumResolvedField::AutoExpandVec(_)));
                        if let Some(f) = first_non_auto {
                            match f {
                                EnumResolvedField::Single(data) => {
                                    let buf = format_ident!("__s_{}_{}", snake, data.col_name);
                                    quote! { #buf.len() }
                                }
                                EnumResolvedField::ExpandedFixed(data) => {
                                    let buf = format_ident!(
                                        "__s_{}_{}_{}",
                                        snake,
                                        data.base_name,
                                        1usize
                                    );
                                    quote! { #buf.len() }
                                }
                                EnumResolvedField::ExpandedVec(data) => {
                                    let buf = format_ident!(
                                        "__s_{}_{}_{}",
                                        snake,
                                        data.base_name,
                                        1usize
                                    );
                                    quote! { #buf.len() }
                                }
                                EnumResolvedField::AutoExpandVec(_) => unreachable!(),
                            }
                        } else {
                            // All fields are AutoExpandVec — use the first auto buf length
                            if let Some(EnumResolvedField::AutoExpandVec(data)) = vi.fields.first()
                            {
                                let buf = format_ident!("__s_{}_{}", snake, data.base_name);
                                quote! { #buf.len() }
                            } else {
                                quote! { 0usize }
                            }
                        }
                    };

                    // Static pair pushes
                    let static_pushes: Vec<TokenStream> = vi
                        .fields
                        .iter()
                        .flat_map(|erf| match erf {
                            EnumResolvedField::Single(data) => {
                                let buf = format_ident!("__s_{}_{}", snake, data.col_name);
                                let col_str = data.col_name.to_string();
                                vec![quote! {
                                    #pairs_var.push((
                                        #col_str.to_string(),
                                        ::miniextendr_api::IntoR::into_sexp(#buf),
                                    ));
                                }]
                            }
                            EnumResolvedField::ExpandedFixed(data) => (1..=data.len)
                                .map(|i| {
                                    let buf = format_ident!(
                                        "__s_{}_{}_{}", snake, data.base_name, i
                                    );
                                    let col_str = format!("{}_{}", data.base_name, i);
                                    quote! {
                                        #pairs_var.push((
                                            #col_str.to_string(),
                                            ::miniextendr_api::IntoR::into_sexp(#buf),
                                        ));
                                    }
                                })
                                .collect(),
                            EnumResolvedField::ExpandedVec(data) => (1..=data.width)
                                .map(|i| {
                                    let buf = format_ident!(
                                        "__s_{}_{}_{}", snake, data.base_name, i
                                    );
                                    let col_str = format!("{}_{}", data.base_name, i);
                                    quote! {
                                        #pairs_var.push((
                                            #col_str.to_string(),
                                            ::miniextendr_api::IntoR::into_sexp(#buf),
                                        ));
                                    }
                                })
                                .collect(),
                            EnumResolvedField::AutoExpandVec(data) => {
                                let buf = format_ident!("__s_{}_{}", snake, data.base_name);
                                let base_str = &data.base_name;
                                let elem_ty = &data.elem_ty;
                                vec![quote! {
                                    {
                                        let __auto = #buf;
                                        let __max = __auto.iter().map(|v| v.len()).max().unwrap_or(0);
                                        let mut __auto_cols: Vec<Vec<Option<#elem_ty>>> = (0..__max)
                                            .map(|_| Vec::with_capacity(#n_var))
                                            .collect();
                                        for __row_vec in &__auto {
                                            for (__ai, __acol) in __auto_cols.iter_mut().enumerate() {
                                                __acol.push(__row_vec.get(__ai).cloned());
                                            }
                                        }
                                        for (__ai, __acol) in __auto_cols.into_iter().enumerate() {
                                            #pairs_var.push((
                                                format!("{}_{}", #base_str, __ai + 1),
                                                ::miniextendr_api::IntoR::into_sexp(__acol),
                                            ));
                                        }
                                    }
                                }]
                            }
                        })
                        .collect();

                    df_constructions.push(quote! {
                        let #n_var = #len_expr;
                        let mut #pairs_var: Vec<(String, ::miniextendr_api::ffi::SEXP)> = Vec::new();
                        #(#static_pushes)*
                        let #df_var = ::miniextendr_api::list::List::from_raw_pairs(#pairs_var)
                            .set_class_str(&["data.frame"])
                            .set_row_names_int(#n_var);
                    });
                } else {
                    // Static path: vec![...] of (&str, SEXP) pairs
                    let n_var = format_ident!("__n_{}", snake);

                    // Length expression: first field's buffer length
                    let len_expr: TokenStream = if let Some(erf) = vi.fields.first() {
                        match erf {
                            EnumResolvedField::Single(data) => {
                                let buf = format_ident!("__s_{}_{}", snake, data.col_name);
                                quote! { #buf.len() }
                            }
                            EnumResolvedField::ExpandedFixed(data) => {
                                let buf =
                                    format_ident!("__s_{}_{}_{}", snake, data.base_name, 1usize);
                                quote! { #buf.len() }
                            }
                            EnumResolvedField::ExpandedVec(data) => {
                                let buf =
                                    format_ident!("__s_{}_{}_{}", snake, data.base_name, 1usize);
                                quote! { #buf.len() }
                            }
                            EnumResolvedField::AutoExpandVec(_) => unreachable!(),
                        }
                    } else {
                        // No fields (unexpected for Named/Tuple, but handle it)
                        quote! { 0usize }
                    };

                    // Collect pairs
                    let pairs: Vec<TokenStream> = vi
                        .fields
                        .iter()
                        .flat_map(|erf| match erf {
                            EnumResolvedField::Single(data) => {
                                let buf = format_ident!("__s_{}_{}", snake, data.col_name);
                                let col_str = data.col_name.to_string();
                                vec![quote! {
                                    (#col_str, ::miniextendr_api::IntoR::into_sexp(#buf))
                                }]
                            }
                            EnumResolvedField::ExpandedFixed(data) => (1..=data.len)
                                .map(|i| {
                                    let buf =
                                        format_ident!("__s_{}_{}_{}", snake, data.base_name, i);
                                    let col_str = format!("{}_{}", data.base_name, i);
                                    quote! {
                                        (#col_str, ::miniextendr_api::IntoR::into_sexp(#buf))
                                    }
                                })
                                .collect(),
                            EnumResolvedField::ExpandedVec(data) => (1..=data.width)
                                .map(|i| {
                                    let buf =
                                        format_ident!("__s_{}_{}_{}", snake, data.base_name, i);
                                    let col_str = format!("{}_{}", data.base_name, i);
                                    quote! {
                                        (#col_str, ::miniextendr_api::IntoR::into_sexp(#buf))
                                    }
                                })
                                .collect(),
                            EnumResolvedField::AutoExpandVec(_) => unreachable!(),
                        })
                        .collect();

                    df_constructions.push(quote! {
                        let #n_var = #len_expr;
                        let #df_var = ::miniextendr_api::list::List::from_raw_pairs(vec![
                            #(#pairs),*
                        ])
                        .set_class_str(&["data.frame"])
                        .set_row_names_int(#n_var);
                    });
                }
            } // endregion
        }
    }

    // Build the method body
    let body = if variant_infos.len() == 1 {
        // Single variant: return the data.frame directly
        let df_var = &df_var_names[0];
        quote! {
            #(#buf_decls)*
            for __row in rows {
                match __row {
                    #(#match_arms)*
                }
            }
            #(#df_constructions)*
            #df_var
        }
    } else {
        // Multiple variants: return named list of data.frames
        let outer_pairs: Vec<TokenStream> = snake_names
            .iter()
            .zip(df_var_names.iter())
            .map(|(name, var)| {
                quote! { (#name, ::miniextendr_api::IntoR::into_sexp(#var)) }
            })
            .collect();

        quote! {
            #(#buf_decls)*
            for __row in rows {
                match __row {
                    #(#match_arms)*
                }
            }
            #(#df_constructions)*
            ::miniextendr_api::list::List::from_raw_pairs(vec![
                #(#outer_pairs),*
            ])
        }
    };

    quote! {
        impl #impl_generics #row_name #ty_generics #where_clause {
            /// Partition rows by variant and return one data.frame per variant.
            ///
            /// For single-variant enums, returns the data.frame directly.
            /// For multi-variant enums, returns a named R list of data.frames where
            /// each name is the variant name in snake_case. Each data.frame has only
            /// that variant's columns (non-optional types — no NA fill).
            pub fn to_dataframe_split(rows: Vec<Self>) -> ::miniextendr_api::list::List {
                #body
            }
        }
    }
}
// endregion
