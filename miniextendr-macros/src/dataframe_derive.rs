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
    /// Tag column name for variant discriminator (enums only).
    tag: Option<String>,
    /// Emit rayon parallel fill path (only effective when `rayon` feature is enabled).
    parallel: bool,
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
                syn::Meta::Path(path) if path.is_ident("align") => {
                    attrs.align = true;
                }
                syn::Meta::Path(path) if path.is_ident("parallel") => {
                    attrs.parallel = true;
                }
                other => {
                    return Err(syn::Error::new_spanned(
                        other,
                        "unknown dataframe attribute; expected `name`, `align`, `tag`, or `parallel`",
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
            if attrs.align {
                return Err(syn::Error::new_spanned(
                    row_name,
                    "`align` is only supported on enums, not structs",
                ));
            }
            if attrs.tag.is_some() {
                return Err(syn::Error::new_spanned(
                    row_name,
                    "`tag` is only supported on enums",
                ));
            }
            derive_struct_dataframe(row_name, &input, data, &df_name, attrs.parallel)
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
    parallel: bool,
) -> syn::Result<TokenStream> {
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let fields = match &data.fields {
        Fields::Named(fields) => &fields.named,
        _ => {
            return Err(syn::Error::new_spanned(
                row_name,
                "DataFrameRow only supports structs with named fields",
            ));
        }
    };

    // Reject zero-field structs — a DataFrame with no columns is meaningless.
    if fields.is_empty() {
        return Err(syn::Error::new_spanned(
            row_name,
            "DataFrameRow requires at least one named field",
        ));
    }

    // Collect field info
    let field_info: Vec<_> = fields
        .iter()
        .map(|f| {
            let name = f.ident.as_ref().unwrap();
            let ty = &f.ty;
            (name, ty)
        })
        .collect();

    // Build DataFrame struct with one field at a time
    let df_fields_tokens: Vec<TokenStream> = field_info
        .iter()
        .map(|(name, ty)| {
            quote! { pub #name: Vec<#ty> }
        })
        .collect();

    // Generate the companion DataFrame struct
    let dataframe_struct = quote! {
        #[derive(Debug, Clone)]
        pub struct #df_name {
            #(#df_fields_tokens),*
        }
    };

    // Extract field names
    let field_names: Vec<_> = field_info.iter().map(|(name, _)| name).collect();

    // Generate IntoDataFrame for DataFrame type
    let first_field = field_names.first().unwrap();
    let df_pairs = field_names.iter().map(|name| {
        let name_str = name.to_string();
        quote! { (#name_str, ::miniextendr_api::IntoR::into_sexp(self.#name)) }
    });

    // Generate column length checks for all fields after the first
    let length_checks: Vec<TokenStream> = field_names
        .iter()
        .skip(1)
        .map(|name| {
            let name_str = name.to_string();
            let first_str = first_field.to_string();
            quote! {
                assert!(
                    self.#name.len() == n_rows,
                    "column length mismatch in {}: column `{}` has length {} but column `{}` has length {}",
                    stringify!(#df_name),
                    #name_str,
                    self.#name.len(),
                    #first_str,
                    n_rows,
                );
            }
        })
        .collect();

    let into_dataframe_impl = quote! {
        impl ::miniextendr_api::convert::IntoDataFrame for #df_name {
            fn into_data_frame(self) -> ::miniextendr_api::List {
                let n_rows = self.#first_field.len();
                #(#length_checks)*
                ::miniextendr_api::list::List::from_raw_pairs(vec![
                    #(#df_pairs),*
                ])
                .set_class_str(&["data.frame"])
                .set_row_names_int(n_rows)
            }
        }
    };

    // Build From impl: consume rows by value to avoid Clone requirement.
    // Single pass: destructure each row and push fields into column Vecs.
    let col_vec_inits: Vec<TokenStream> = field_info
        .iter()
        .map(|(name, ty)| {
            quote! { let mut #name: Vec<#ty> = Vec::with_capacity(len); }
        })
        .collect();

    let col_pushes: Vec<TokenStream> = field_info
        .iter()
        .map(|(name, _ty)| {
            quote! { #name.push(row.#name); }
        })
        .collect();

    let col_struct_fields: Vec<TokenStream> = field_info
        .iter()
        .map(|(name, _ty)| {
            quote! { #name }
        })
        .collect();

    let parallel_block = if parallel {
        gen_parallel_struct_from(row_name, df_name, &field_info)
    } else {
        TokenStream::new()
    };

    let from_vec_impl = quote! {
        impl From<Vec<#row_name>> for #df_name {
            fn from(rows: Vec<#row_name>) -> Self {
                let len = rows.len();
                #parallel_block
                #(#col_vec_inits)*
                for row in rows {
                    #(#col_pushes)*
                }
                #df_name {
                    #(#col_struct_fields),*
                }
            }
        }
    };

    // Generate IntoIterator for DataFrame - use concrete types for simplicity
    let iterator_name = format_ident!("{}Iterator", df_name);

    // Generate iterator fields paired with their types to ensure correct correspondence
    let iter_field_decls: Vec<_> = field_info
        .iter()
        .map(|(name, ty)| {
            quote! { #name: std::vec::IntoIter<#ty> }
        })
        .collect();

    // Generate destructuring pattern
    let destruct_pattern = field_info
        .iter()
        .map(|(name, _ty)| quote! { #name })
        .collect::<Vec<_>>();

    // Build struct initialization tokens explicitly with explicit types
    let mut iter_init_tokens = TokenStream::new();
    for (i, (name, ty)) in field_info.iter().enumerate() {
        if i > 0 {
            iter_init_tokens.extend(quote! { , });
        }
        iter_init_tokens.extend(quote! { #name: <Vec<#ty>>::into_iter(#name) });
    }

    // Build Iterator::next() return struct tokens explicitly
    let mut next_struct_tokens = TokenStream::new();
    for (i, (name, _ty)) in field_info.iter().enumerate() {
        if i > 0 {
            next_struct_tokens.extend(quote! { , });
        }
        next_struct_tokens.extend(quote! { #name: self.#name.next()? });
    }

    let into_iterator_impl = quote! {
        pub struct #iterator_name {
            #(#iter_field_decls),*
        }

        impl IntoIterator for #df_name {
            type Item = #row_name;
            type IntoIter = #iterator_name;

            fn into_iter(self) -> Self::IntoIter {
                let #df_name { #(#destruct_pattern),* } = self;
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
    };

    // Generate associated methods on row type for working with DataFrame
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

            /// Convert a DataFrame back into a vector of rows.
            ///
            /// This transposes column-oriented data back into row-oriented format.
            pub fn from_dataframe(df: #df_name) -> Vec<Self> {
                df.into_iter().collect()
            }
        }
    };

    // Compile-time assertion: row type must implement IntoList
    let trait_check = quote! {
        const _: () = {
            fn _assert_into_list #impl_generics () #where_clause {
                fn _check<T: ::miniextendr_api::list::IntoList>() {}
                _check::<#row_name #ty_generics>();
            }
        };
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
    ty: syn::Type,
    /// Indices of variants that contain this field.
    present_in: Vec<usize>,
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

    let mut variant_infos: Vec<(&syn::Ident, Vec<(&syn::Ident, &syn::Type)>)> = Vec::new();

    for variant in &data.variants {
        match &variant.fields {
            Fields::Named(fields) => {
                let field_info: Vec<_> = fields
                    .named
                    .iter()
                    .map(|f| (f.ident.as_ref().unwrap(), &f.ty))
                    .collect();
                if field_info.is_empty() {
                    return Err(syn::Error::new_spanned(
                        &variant.ident,
                        "DataFrameRow align: variants must have at least one named field",
                    ));
                }
                variant_infos.push((&variant.ident, field_info));
            }
            Fields::Unit => {
                return Err(syn::Error::new_spanned(
                    &variant.ident,
                    "DataFrameRow align does not support unit variants",
                ));
            }
            Fields::Unnamed(_) => {
                return Err(syn::Error::new_spanned(
                    &variant.ident,
                    "DataFrameRow align does not support tuple variants",
                ));
            }
        }
    }

    // ── Resolve unified schema ───────────────────────────────────────────
    // Collect all unique field names, check type consistency.
    let mut columns: Vec<ResolvedColumn> = Vec::new();
    // Map field name → index in `columns` for fast lookup.
    let mut col_index: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for (variant_idx, (_variant_name, fields)) in variant_infos.iter().enumerate() {
        for (field_name, field_ty) in fields {
            let key = field_name.to_string();
            if let Some(&idx) = col_index.get(&key) {
                // Field already seen — check type matches exactly.
                let existing = &columns[idx];
                if existing.ty != **field_ty {
                    return Err(syn::Error::new_spanned(
                        field_ty,
                        format!(
                            "type conflict for field `{}`: variant `{}` has a different type \
                             than a previous variant",
                            key, _variant_name
                        ),
                    ));
                }
                columns[idx].present_in.push(variant_idx);
            } else {
                let idx = columns.len();
                columns.push(ResolvedColumn {
                    col_name: format_ident!("{}", field_name),
                    ty: (*field_ty).clone(),
                    present_in: vec![variant_idx],
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
    } else {
        let first_col = &columns[0].col_name;
        quote! { self.#first_col.len() }
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

    // Length checks for all columns after the first
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
        .map(|(variant_idx, (variant_name, fields))| {
            let variant_name_str = variant_name.to_string();

            // Destructure pattern: variant fields get prefixed to avoid shadowing
            let field_bindings: Vec<TokenStream> = fields
                .iter()
                .map(|(fname, _)| {
                    let binding = format_ident!("__v_{}", fname);
                    quote! { #fname: #binding }
                })
                .collect();

            let tag_push = if has_tag {
                quote! { _tag.push(#variant_name_str.to_string()); }
            } else {
                TokenStream::new()
            };

            // Push Some for present fields, None for absent
            let col_pushes: Vec<TokenStream> = columns
                .iter()
                .map(|col| {
                    let col_name = &col.col_name;
                    if col.present_in.contains(&variant_idx) {
                        let binding = format_ident!("__v_{}", col_name);
                        quote! { #col_name.push(Some(#binding)); }
                    } else {
                        quote! { #col_name.push(None); }
                    }
                })
                .collect();

            quote! {
                #row_name::#variant_name { #(#field_bindings),* } => {
                    #tag_push
                    #(#col_pushes)*
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

    let parallel_block = if attrs.parallel {
        gen_parallel_enum_from(row_name, df_name, &columns, &variant_infos, has_tag)
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
