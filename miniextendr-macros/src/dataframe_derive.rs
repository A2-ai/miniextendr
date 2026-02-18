//! Derive macros for bidirectional row ↔ dataframe conversions.
//!
//! Supports both structs (direct field mapping) and enums (field-name union
//! across variants with `Option<T>` fill for missing fields).

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields};

// =============================================================================
// Attribute parsing
// =============================================================================

/// Parsed container-level `#[dataframe(...)]` attributes.
struct DataFrameAttrs {
    /// Custom companion type name (default: `{TypeName}DataFrame`).
    name: Option<syn::Ident>,
    /// Enum alignment mode — implicit for enums, accepted but not required.
    align: bool,
    /// Tag column name for variant discriminator (also supported on structs).
    tag: Option<String>,
    /// Emit rayon parallel fill path (only effective when `rayon` feature is enabled).
    parallel: bool,
    /// Conflict resolution mode for type collisions across enum variants.
    /// Currently only "string" is supported: convert conflicting fields via `ToString`.
    conflicts: Option<String>,
}

/// Parse container-level `#[dataframe(...)]` attributes.
///
/// Supported keys:
/// - `name = "CustomName"` — custom companion type name
/// - `align` — enum alignment mode (field-name union)
/// - `tag = "col_name"` — variant discriminator column
fn parse_dataframe_attrs(input: &DeriveInput) -> syn::Result<DataFrameAttrs> {
    let mut attrs = DataFrameAttrs {
        name: None,
        align: false,
        tag: None,
        parallel: false,
        conflicts: None,
    };

    for attr in &input.attrs {
        if !attr.path().is_ident("dataframe") {
            continue;
        }

        let nested = attr.parse_args_with(
            syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
        )?;

        for meta in &nested {
            match meta {
                syn::Meta::NameValue(nv) if nv.path.is_ident("name") => {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(lit_str),
                        ..
                    }) = &nv.value
                    {
                        attrs.name =
                            Some(format_ident!("{}", lit_str.value(), span = lit_str.span()));
                    } else {
                        return Err(syn::Error::new_spanned(
                            &nv.value,
                            "expected string literal for `name`",
                        ));
                    }
                }
                syn::Meta::NameValue(nv) if nv.path.is_ident("tag") => {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(lit_str),
                        ..
                    }) = &nv.value
                    {
                        attrs.tag = Some(lit_str.value());
                    } else {
                        return Err(syn::Error::new_spanned(
                            &nv.value,
                            "expected string literal for `tag`",
                        ));
                    }
                }
                syn::Meta::NameValue(nv) if nv.path.is_ident("conflicts") => {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(lit_str),
                        ..
                    }) = &nv.value
                    {
                        let value = lit_str.value();
                        if value != "string" {
                            return Err(syn::Error::new_spanned(
                                lit_str,
                                "unknown conflict resolution mode; only `\"string\"` is supported",
                            ));
                        }
                        attrs.conflicts = Some(value);
                    } else {
                        return Err(syn::Error::new_spanned(
                            &nv.value,
                            "expected string literal for `conflicts`",
                        ));
                    }
                }
                syn::Meta::Path(path) if path.is_ident("align") => {
                    attrs.align = true;
                }
                syn::Meta::Path(path) if path.is_ident("parallel") => {
                    attrs.parallel = true;
                }
                other => {
                    return Err(syn::Error::new_spanned(
                        other,
                        "unknown dataframe attribute; expected `name`, `align`, `tag`, `parallel`, or `conflicts`",
                    ));
                }
            }
        }
    }

    Ok(attrs)
}

// =============================================================================
// Top-level dispatch
// =============================================================================

/// Derive `DataFrameRow`: generates a companion DataFrame type with collection fields.
///
/// # Requirements
///
/// For structs: the type must implement `IntoList`.
/// For enums: all variants must have named fields.
///
/// # Generated Items
///
/// For a struct `Measurement { time: f64, value: f64 }`:
/// - Struct `MeasurementDataFrame { time: Vec<f64>, value: Vec<f64> }`
/// - `impl IntoDataFrame for MeasurementDataFrame`
/// - `impl From<Vec<Measurement>> for MeasurementDataFrame`
/// - `impl IntoIterator for MeasurementDataFrame`
/// - Associated methods on `Measurement`:
///   - `to_dataframe(Vec<Self>) -> MeasurementDataFrame`
///   - `from_dataframe(MeasurementDataFrame) -> Vec<Self>`
///
/// For an enum:
/// - Companion struct with `Vec<Option<T>>` columns (field-name union)
/// - Optional tag column for variant discrimination
/// - `impl From<Vec<Enum>> for EnumDataFrame`
/// - `impl IntoDataFrame for EnumDataFrame`
/// - Associated `to_dataframe` method
///
/// # Attributes
///
/// - `#[dataframe(name = "CustomName")]` — Custom companion type name
/// - `#[dataframe(align)]` — Enum alignment mode (accepted but implicit)
/// - `#[dataframe(tag = "col")]` — Add variant discriminator column
/// - `#[dataframe(parallel)]` — Enable rayon parallel fill for large inputs
///   (requires `rayon` feature; uses threshold-based fallback to sequential)
pub fn derive_dataframe_row(input: DeriveInput) -> syn::Result<TokenStream> {
    let row_name = &input.ident;

    // Reject generic types — the generated companion type and impls cannot
    // propagate generics correctly, so fail early with a clear message.
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.generics,
            "DataFrameRow does not support generic types",
        ));
    }

    // Parse attributes
    let attrs = parse_dataframe_attrs(&input)?;

    let df_name = attrs
        .name
        .clone()
        .unwrap_or_else(|| format_ident!("{}DataFrame", row_name));

    match &input.data {
        Data::Struct(data) => {
            // `align` is a no-op on structs (only semantically meaningful for enums)
            derive_struct_dataframe(row_name, &input, data, &df_name, &attrs)
        }
        Data::Enum(data) => {
            // align is implicit for enums — accept but don't require
            derive_enum_dataframe(row_name, &input, data, &df_name, &attrs)
        }
        Data::Union(_) => Err(syn::Error::new_spanned(
            row_name,
            "DataFrameRow does not support unions",
        )),
    }
}

// =============================================================================
// Struct path (existing logic, extracted)
// =============================================================================

fn derive_struct_dataframe(
    row_name: &syn::Ident,
    input: &DeriveInput,
    data: &syn::DataStruct,
    df_name: &syn::Ident,
    attrs: &DataFrameAttrs,
) -> syn::Result<TokenStream> {
    let parallel = attrs.parallel;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Collect field info — supports both named and unnamed (tuple) fields.
    // For unnamed fields, synthesize column names: _0, _1, _2, ...
    let owned_field_info: Vec<(syn::Ident, &syn::Type)> = match &data.fields {
        Fields::Named(fields) => fields
            .named
            .iter()
            .map(|f| (f.ident.as_ref().unwrap().clone(), &f.ty))
            .collect(),
        Fields::Unnamed(fields) => fields
            .unnamed
            .iter()
            .enumerate()
            .map(|(i, f)| (format_ident!("_{}", i), &f.ty))
            .collect(),
        Fields::Unit => vec![],
    };

    // Build field_info as references for compatibility with existing code
    let field_info: Vec<(&syn::Ident, &syn::Type)> = owned_field_info
        .iter()
        .map(|(name, ty)| (name, *ty))
        .collect();

    let is_tuple_struct = matches!(&data.fields, Fields::Unnamed(_));
    let is_unit_struct = matches!(&data.fields, Fields::Unit);

    let has_tag = attrs.tag.is_some();
    let row_name_str = row_name.to_string();

    // ── Companion struct ────────────────────────────────────────────────
    let tag_field_decl = if has_tag {
        quote! { pub _tag: Vec<String>, }
    } else {
        TokenStream::new()
    };

    // Build DataFrame struct fields (one Vec per source field)
    let df_fields_tokens: Vec<TokenStream> = field_info
        .iter()
        .map(|(name, ty)| {
            quote! { pub #name: Vec<#ty> }
        })
        .collect();

    // For empty structs, add a _len tracking field
    let len_field_decl = if field_info.is_empty() && !has_tag {
        quote! { pub _len: usize, }
    } else {
        TokenStream::new()
    };

    let dataframe_struct = quote! {
        #[derive(Debug, Clone)]
        pub struct #df_name {
            #tag_field_decl
            #len_field_decl
            #(#df_fields_tokens),*
        }
    };

    // ── IntoDataFrame ───────────────────────────────────────────────────
    // Determine the row-count reference
    let length_ref = if has_tag {
        quote! { self._tag.len() }
    } else if field_info.is_empty() {
        quote! { self._len }
    } else {
        let first = &field_info[0].0;
        quote! { self.#first.len() }
    };

    let tag_pair = if let Some(ref tag_name) = attrs.tag {
        quote! { (#tag_name, ::miniextendr_api::IntoR::into_sexp(self._tag)), }
    } else {
        TokenStream::new()
    };

    let df_pairs: Vec<TokenStream> = field_info
        .iter()
        .map(|(name, _)| {
            let name_str = name.to_string();
            quote! { (#name_str, ::miniextendr_api::IntoR::into_sexp(self.#name)) }
        })
        .collect();

    // Length checks for all columns
    let length_checks: Vec<TokenStream> = field_info
        .iter()
        .map(|(name, _)| {
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

    let into_dataframe_impl = quote! {
        impl ::miniextendr_api::convert::IntoDataFrame for #df_name {
            fn into_data_frame(self) -> ::miniextendr_api::List {
                let _n_rows = #length_ref;
                #(#length_checks)*
                ::miniextendr_api::list::List::from_raw_pairs(vec![
                    #tag_pair
                    #(#df_pairs),*
                ])
                .set_class_str(&["data.frame"])
                .set_row_names_int(_n_rows)
            }
        }
    };

    // ── From<Vec<RowType>> ──────────────────────────────────────────────
    let col_vec_inits: Vec<TokenStream> = field_info
        .iter()
        .map(|(name, ty)| {
            quote! { let mut #name: Vec<#ty> = Vec::with_capacity(len); }
        })
        .collect();

    let tag_init = if has_tag {
        quote! { let mut _tag: Vec<String> = Vec::with_capacity(len); }
    } else {
        TokenStream::new()
    };

    let tag_push = if has_tag {
        quote! { _tag.push(#row_name_str.to_string()); }
    } else {
        TokenStream::new()
    };

    // Access pattern for row fields depends on named vs tuple struct
    let col_pushes: Vec<TokenStream> = if is_tuple_struct {
        field_info
            .iter()
            .enumerate()
            .map(|(i, (name, _))| {
                let idx = syn::Index::from(i);
                quote! { #name.push(row.#idx); }
            })
            .collect()
    } else {
        field_info
            .iter()
            .map(|(name, _)| {
                quote! { #name.push(row.#name); }
            })
            .collect()
    };

    let tag_struct_field = if has_tag {
        quote! { _tag, }
    } else {
        TokenStream::new()
    };

    let len_struct_field = if field_info.is_empty() && !has_tag {
        quote! { _len: len, }
    } else {
        TokenStream::new()
    };

    let col_struct_fields: Vec<TokenStream> = field_info
        .iter()
        .map(|(name, _)| quote! { #name })
        .collect();

    let parallel_block = if parallel && !field_info.is_empty() {
        gen_parallel_struct_from(row_name, df_name, &field_info)
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
                    #tag_push
                    #(#col_pushes)*
                }
                #df_name {
                    #tag_struct_field
                    #len_struct_field
                    #(#col_struct_fields),*
                }
            }
        }
    };

    // ── IntoIterator (only when there are fields — empty/unit structs skip) ─
    let into_iterator_impl = if !field_info.is_empty() && !is_tuple_struct && !is_unit_struct {
        let iterator_name = format_ident!("{}Iterator", df_name);

        let iter_field_decls: Vec<_> = field_info
            .iter()
            .map(|(name, ty)| quote! { #name: std::vec::IntoIter<#ty> })
            .collect();

        let destruct_pattern: Vec<_> = field_info
            .iter()
            .map(|(name, _)| quote! { #name })
            .collect();

        let mut iter_init_tokens = TokenStream::new();
        for (i, (name, ty)) in field_info.iter().enumerate() {
            if i > 0 {
                iter_init_tokens.extend(quote! { , });
            }
            iter_init_tokens.extend(quote! { #name: <Vec<#ty>>::into_iter(#name) });
        }

        let mut next_struct_tokens = TokenStream::new();
        for (i, (name, _)) in field_info.iter().enumerate() {
            if i > 0 {
                next_struct_tokens.extend(quote! { , });
            }
            next_struct_tokens.extend(quote! { #name: self.#name.next()? });
        }

        // Destructure the df, ignoring tag/_len fields
        let ignore_tag = if has_tag {
            quote! { _tag: _, }
        } else {
            TokenStream::new()
        };

        quote! {
            pub struct #iterator_name {
                #(#iter_field_decls),*
            }

            impl IntoIterator for #df_name {
                type Item = #row_name;
                type IntoIter = #iterator_name;

                fn into_iter(self) -> Self::IntoIter {
                    let #df_name { #ignore_tag #(#destruct_pattern),* } = self;
                    #iterator_name {
                        #iter_init_tokens
                    }
                }
            }

            impl Iterator for #iterator_name {
                type Item = #row_name;

                fn next(&mut self) -> Option<Self::Item> {
                    Some(#row_name {
                        #next_struct_tokens
                    })
                }
            }
        }
    } else {
        // For tuple structs, unit structs, and empty structs:
        // skip IntoIterator generation (no round-trip back to original type)
        TokenStream::new()
    };

    // ── Associated methods ──────────────────────────────────────────────
    let from_dataframe_method = if !field_info.is_empty() && !is_tuple_struct && !is_unit_struct {
        quote! {
            /// Convert a DataFrame back into a vector of rows.
            ///
            /// This transposes column-oriented data back into row-oriented format.
            pub fn from_dataframe(df: #df_name) -> Vec<Self> {
                df.into_iter().collect()
            }
        }
    } else {
        TokenStream::new()
    };

    let row_methods = quote! {
        impl #impl_generics #row_name #ty_generics #where_clause {
            /// Name of the generated DataFrame companion type.
            pub const DATAFRAME_TYPE_NAME: &'static str = stringify!(#df_name);

            /// Convert a vector of rows into the companion DataFrame type.
            ///
            /// This transposes row-oriented data into column-oriented format.
            pub fn to_dataframe(rows: Vec<Self>) -> #df_name {
                rows.into()
            }

            #from_dataframe_method
        }
    };

    // Compile-time assertion: row type must implement IntoList
    // Skip for unit/empty structs and tuple structs (they may not implement IntoList)
    let trait_check = if !field_info.is_empty() && !is_tuple_struct && !is_unit_struct {
        quote! {
            const _: () = {
                fn _assert_into_list #impl_generics () #where_clause {
                    fn _check<T: ::miniextendr_api::list::IntoList>() {}
                    _check::<#row_name #ty_generics>();
                }
            };
        }
    } else {
        TokenStream::new()
    };

    Ok(quote! {
        #dataframe_struct
        #into_dataframe_impl
        #from_vec_impl
        #into_iterator_impl
        #row_methods
        #trait_check
    })
}

// =============================================================================
// Enum align path
// =============================================================================

/// A resolved column in the unified schema across all enum variants.
struct ResolvedColumn {
    /// Column name in the companion struct / data frame.
    col_name: syn::Ident,
    /// Element type (used as `Vec<Option<#ty>>`).
    /// When `string_coerced` is true, this is always `String`.
    ty: syn::Type,
    /// Indices of variants that contain this field.
    present_in: Vec<usize>,
    /// Whether this column was coerced to `String` due to type conflicts.
    /// When true, values are converted via `ToString::to_string()` at push time.
    string_coerced: bool,
}

/// Describes the shape of an enum variant's fields.
#[derive(Clone, Copy, PartialEq, Eq)]
enum VariantShape {
    /// `Variant { field: Type, ... }`
    Named,
    /// `Variant(Type, ...)`
    Tuple,
    /// `Variant` (no fields)
    Unit,
}

/// Parsed information about an enum variant.
struct VariantInfo {
    /// Variant name.
    name: syn::Ident,
    /// Shape of this variant.
    shape: VariantShape,
    /// Fields: (column_name, type). For tuple variants, column names are _0, _1, etc.
    /// For unit variants, this is empty.
    fields: Vec<(syn::Ident, syn::Type)>,
}

/// Derive `DataFrameRow` for an enum with `#[dataframe(align)]`.
///
/// Generates a companion struct where every column is `Vec<Option<T>>`, with
/// `None` fill for fields absent in a given variant.
fn derive_enum_dataframe(
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
                let field_info: Vec<_> = fields
                    .named
                    .iter()
                    .map(|f| (f.ident.as_ref().unwrap().clone(), f.ty.clone()))
                    .collect();
                // Empty named variants are allowed — they only contribute to tag column
                variant_infos.push(VariantInfo {
                    name: variant.ident.clone(),
                    shape: VariantShape::Named,
                    fields: field_info,
                });
            }
            Fields::Unnamed(fields) => {
                let field_info: Vec<_> = fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(i, f)| (format_ident!("_{}", i), f.ty.clone()))
                    .collect();
                variant_infos.push(VariantInfo {
                    name: variant.ident.clone(),
                    shape: VariantShape::Tuple,
                    fields: field_info,
                });
            }
            Fields::Unit => {
                // Unit variants contribute only to tag column; all data columns get None
                variant_infos.push(VariantInfo {
                    name: variant.ident.clone(),
                    shape: VariantShape::Unit,
                    fields: vec![],
                });
            }
        }
    }

    // ── Resolve unified schema ───────────────────────────────────────────
    // Collect all unique field names, check type consistency.
    let coerce_to_string = attrs.conflicts.as_deref() == Some("string");
    let string_ty: syn::Type = syn::parse_quote!(String);

    let mut columns: Vec<ResolvedColumn> = Vec::new();
    // Map field name → index in `columns` for fast lookup.
    let mut col_index: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for (variant_idx, vi) in variant_infos.iter().enumerate() {
        for (field_name, field_ty) in &vi.fields {
            let key = field_name.to_string();
            if let Some(&idx) = col_index.get(&key) {
                // Field already seen — check type matches.
                let existing = &columns[idx];
                if !existing.string_coerced && existing.ty != *field_ty {
                    if coerce_to_string {
                        // Coerce this column to String
                        columns[idx].ty = string_ty.clone();
                        columns[idx].string_coerced = true;
                    } else {
                        return Err(syn::Error::new_spanned(
                            field_ty,
                            format!(
                                "type conflict for field `{}`: variant `{}` has a different type \
                                 than a previous variant; \
                                 use `#[dataframe(conflicts = \"string\")]` to coerce all conflicting fields to String",
                                key, vi.name
                            ),
                        ));
                    }
                }
                columns[idx].present_in.push(variant_idx);
            } else {
                let idx = columns.len();
                columns.push(ResolvedColumn {
                    col_name: format_ident!("{}", field_name),
                    ty: field_ty.clone(),
                    present_in: vec![variant_idx],
                    string_coerced: false,
                });
                col_index.insert(key, idx);
            }
        }
    }

    // ── Generate companion struct ────────────────────────────────────────
    let has_tag = attrs.tag.is_some();

    let tag_field = if has_tag {
        quote! { pub _tag: Vec<String>, }
    } else {
        TokenStream::new()
    };

    let df_fields: Vec<TokenStream> = columns
        .iter()
        .map(|col| {
            let name = &col.col_name;
            let ty = &col.ty;
            quote! { pub #name: Vec<Option<#ty>> }
        })
        .collect();

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

    let into_dataframe_impl = quote! {
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
    };

    // ── Generate From<Vec<Enum>> ─────────────────────────────────────────
    let col_vec_inits: Vec<TokenStream> = columns
        .iter()
        .map(|col| {
            let name = &col.col_name;
            let ty = &col.ty;
            quote! { let mut #name: Vec<Option<#ty>> = Vec::with_capacity(len); }
        })
        .collect();

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

            // Push Some for present fields, None for absent.
            // String-coerced columns use `ToString::to_string()` on the value.
            let col_pushes: Vec<TokenStream> = columns
                .iter()
                .map(|col| {
                    let col_name = &col.col_name;
                    if col.present_in.contains(&variant_idx) {
                        let binding = format_ident!("__v_{}", col_name);
                        if col.string_coerced {
                            quote! { #col_name.push(Some(ToString::to_string(&#binding))); }
                        } else {
                            quote! { #col_name.push(Some(#binding)); }
                        }
                    } else {
                        quote! { #col_name.push(None); }
                    }
                })
                .collect();

            // Generate destructure pattern based on variant shape
            match vi.shape {
                VariantShape::Named => {
                    let field_bindings: Vec<TokenStream> = vi
                        .fields
                        .iter()
                        .map(|(fname, _)| {
                            let binding = format_ident!("__v_{}", fname);
                            quote! { #fname: #binding }
                        })
                        .collect();
                    quote! {
                        #row_name::#variant_name { #(#field_bindings),* } => {
                            #tag_push
                            #(#col_pushes)*
                        }
                    }
                }
                VariantShape::Tuple => {
                    let field_bindings: Vec<TokenStream> = vi
                        .fields
                        .iter()
                        .map(|(fname, _)| format_ident!("__v_{}", fname))
                        .map(|binding| quote! { #binding })
                        .collect();
                    quote! {
                        #row_name::#variant_name(#(#field_bindings),*) => {
                            #tag_push
                            #(#col_pushes)*
                        }
                    }
                }
                VariantShape::Unit => {
                    quote! {
                        #row_name::#variant_name => {
                            #tag_push
                            #(#col_pushes)*
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

    let col_struct_fields: Vec<TokenStream> = columns
        .iter()
        .map(|col| {
            let name = &col.col_name;
            quote! { #name }
        })
        .collect();

    // Skip parallel path for simplicity when non-named variants are present
    let has_non_named = variant_infos
        .iter()
        .any(|vi| vi.shape != VariantShape::Named);
    let parallel_block = if attrs.parallel && !has_non_named {
        // Convert to old-style variant_infos for parallel gen (only named variants)
        let old_variant_infos: Vec<(&syn::Ident, Vec<(&syn::Ident, &syn::Type)>)> = variant_infos
            .iter()
            .map(|vi| {
                let fields: Vec<_> = vi.fields.iter().map(|(n, t)| (n, t)).collect();
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
fn gen_parallel_struct_from(
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
