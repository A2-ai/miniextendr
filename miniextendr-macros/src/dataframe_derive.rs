//! Derive macros for bidirectional row ↔ dataframe conversions.
//!
//! Supports both structs (direct field mapping) and enums (field-name union
//! across variants with `Option<T>` fill for missing fields).

use proc_macro2::{Span, TokenStream};
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
// Field-level attribute parsing
// =============================================================================

/// Parsed field-level `#[dataframe(...)]` attributes.
#[derive(Default)]
struct FieldAttrs {
    /// Omit this field from the DataFrame.
    skip: bool,
    /// Custom column name.
    rename: Option<String>,
    /// Keep collection as single list column (suppress expansion).
    as_list: bool,
    /// Explicitly expand to suffixed columns.
    expand: bool,
    /// Pin expansion width for variable-length collections.
    width: Option<usize>,
}

/// Parse field-level `#[dataframe(...)]` attributes from a `syn::Field`.
fn parse_field_attrs(field: &syn::Field) -> syn::Result<FieldAttrs> {
    let mut attrs = FieldAttrs::default();

    for attr in &field.attrs {
        if !attr.path().is_ident("dataframe") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("skip") {
                attrs.skip = true;
                Ok(())
            } else if meta.path.is_ident("rename") {
                let value = meta.value()?;
                let lit: syn::LitStr = value.parse()?;
                attrs.rename = Some(lit.value());
                Ok(())
            } else if meta.path.is_ident("as_list") {
                attrs.as_list = true;
                Ok(())
            } else if meta.path.is_ident("expand") {
                attrs.expand = true;
                Ok(())
            } else if meta.path.is_ident("width") {
                let value = meta.value()?;
                let lit: syn::LitInt = value.parse()?;
                let n: usize = lit.base10_parse()?;
                if n == 0 {
                    return Err(syn::Error::new(lit.span(), "`width` must be >= 1"));
                }
                attrs.width = Some(n);
                Ok(())
            } else {
                Err(meta.error(
                    "unknown field attribute; expected `skip`, `rename`, `as_list`, `expand`, or `width`",
                ))
            }
        })?;
    }

    // Validation: conflicting options
    if attrs.as_list && attrs.expand {
        return Err(syn::Error::new(
            field.ident.as_ref().map_or(Span::call_site(), |i| i.span()),
            "`as_list` and `expand` are mutually exclusive",
        ));
    }
    if attrs.as_list && attrs.width.is_some() {
        return Err(syn::Error::new(
            field.ident.as_ref().map_or(Span::call_site(), |i| i.span()),
            "`as_list` and `width` are mutually exclusive",
        ));
    }

    Ok(attrs)
}

// =============================================================================
// Type classification
// =============================================================================

/// Classification of a field type for expansion purposes.
enum FieldTypeKind<'a> {
    /// Single column (most types).
    Scalar,
    /// `[T; N]` — expands to N columns at compile time.
    FixedArray(&'a syn::Type, usize),
    /// `Vec<T>` — variable length, needs `width` for expansion.
    VariableVec(&'a syn::Type),
}

/// Classify a field type for DataFrame expansion.
fn classify_field_type(ty: &syn::Type) -> FieldTypeKind<'_> {
    // Check for [T; N]
    if let syn::Type::Array(arr) = ty
        && let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Int(lit_int),
            ..
        }) = &arr.len
        && let Ok(n) = lit_int.base10_parse::<usize>()
    {
        return FieldTypeKind::FixedArray(&arr.elem, n);
    }

    // Check for Vec<T>
    if let syn::Type::Path(type_path) = ty
        && let Some(seg) = type_path.path.segments.last()
        && seg.ident == "Vec"
        && let syn::PathArguments::AngleBracketed(args) = &seg.arguments
        && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
    {
        return FieldTypeKind::VariableVec(inner);
    }

    FieldTypeKind::Scalar
}

// =============================================================================
// Resolved field model (struct path)
// =============================================================================

/// A resolved field ready for codegen — either a single column or expanded.
enum ResolvedField {
    /// Single column: `name → Vec<ty>`.
    Single {
        /// Rust field name (for access).
        rust_name: syn::Ident,
        /// Column name in the DataFrame.
        col_name: syn::Ident,
        /// Column name string.
        col_name_str: String,
        /// Field type.
        ty: syn::Type,
        /// Index in tuple struct (None for named).
        tuple_index: Option<syn::Index>,
    },
    /// Expanded fixed array: `name: [T; N]` → `name_1..name_N`.
    ExpandedFixed {
        /// Rust field name.
        rust_name: syn::Ident,
        /// Base column name (before suffix).
        base_name: String,
        /// Element type T.
        elem_ty: syn::Type,
        /// Array length N.
        len: usize,
        /// Index in tuple struct.
        tuple_index: Option<syn::Index>,
    },
    /// Expanded variable vec with pinned width: `name: Vec<T>` + `width = N`.
    ExpandedVec {
        /// Rust field name.
        rust_name: syn::Ident,
        /// Base column name.
        base_name: String,
        /// Element type T.
        elem_ty: syn::Type,
        /// Pinned width.
        width: usize,
        /// Index in tuple struct.
        tuple_index: Option<syn::Index>,
    },
}

impl ResolvedField {
    /// Number of DataFrame columns this field expands to.
    #[allow(dead_code)]
    fn column_count(&self) -> usize {
        match self {
            ResolvedField::Single { .. } => 1,
            ResolvedField::ExpandedFixed { len, .. } => *len,
            ResolvedField::ExpandedVec { width, .. } => *width,
        }
    }
}

/// Resolve a struct field into a `ResolvedField`, applying field attrs.
fn resolve_struct_field(
    field: &syn::Field,
    index: usize,
    is_tuple: bool,
) -> syn::Result<Option<ResolvedField>> {
    let field_attrs = parse_field_attrs(field)?;

    if field_attrs.skip {
        return Ok(None);
    }

    let rust_name = if is_tuple {
        format_ident!("_{}", index)
    } else {
        field.ident.as_ref().unwrap().clone()
    };

    let col_name_str = field_attrs
        .rename
        .clone()
        .unwrap_or_else(|| rust_name.to_string());
    let col_name = format_ident!("{}", col_name_str);

    let tuple_index = if is_tuple {
        Some(syn::Index::from(index))
    } else {
        None
    };

    let ty = &field.ty;
    let kind = classify_field_type(ty);

    // as_list suppresses expansion
    if field_attrs.as_list {
        return Ok(Some(ResolvedField::Single {
            rust_name,
            col_name,
            col_name_str,
            ty: ty.clone(),
            tuple_index,
        }));
    }

    match kind {
        FieldTypeKind::FixedArray(elem_ty, len) => Ok(Some(ResolvedField::ExpandedFixed {
            rust_name,
            base_name: col_name_str,
            elem_ty: elem_ty.clone(),
            len,
            tuple_index,
        })),
        FieldTypeKind::VariableVec(elem_ty) => {
            if let Some(width) = field_attrs.width {
                Ok(Some(ResolvedField::ExpandedVec {
                    rust_name,
                    base_name: col_name_str,
                    elem_ty: elem_ty.clone(),
                    width,
                    tuple_index,
                }))
            } else if field_attrs.expand {
                Err(syn::Error::new_spanned(
                    ty,
                    "`expand` on Vec<T> requires `width = N`; \
                     use `#[dataframe(expand, width = N)]` or `#[dataframe(width = N)]`",
                ))
            } else {
                // No expansion — keep as opaque single column
                Ok(Some(ResolvedField::Single {
                    rust_name,
                    col_name,
                    col_name_str,
                    ty: ty.clone(),
                    tuple_index,
                }))
            }
        }
        FieldTypeKind::Scalar => {
            if field_attrs.width.is_some() {
                return Err(syn::Error::new_spanned(
                    ty,
                    "`width` is only valid on `Vec<T>` fields",
                ));
            }
            if field_attrs.expand {
                return Err(syn::Error::new_spanned(
                    ty,
                    "`expand` is only valid on `[T; N]` or `Vec<T>` fields",
                ));
            }
            Ok(Some(ResolvedField::Single {
                rust_name,
                col_name,
                col_name_str,
                ty: ty.clone(),
                tuple_index,
            }))
        }
    }
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

    let is_tuple_struct = matches!(&data.fields, Fields::Unnamed(_));
    let is_unit_struct = matches!(&data.fields, Fields::Unit);

    // Resolve fields through the new FieldAttrs + type classification system.
    let resolved: Vec<ResolvedField> = match &data.fields {
        Fields::Named(fields) => {
            let mut out = Vec::new();
            for (i, f) in fields.named.iter().enumerate() {
                if let Some(rf) = resolve_struct_field(f, i, false)? {
                    out.push(rf);
                }
            }
            out
        }
        Fields::Unnamed(fields) => {
            let mut out = Vec::new();
            for (i, f) in fields.unnamed.iter().enumerate() {
                if let Some(rf) = resolve_struct_field(f, i, true)? {
                    out.push(rf);
                }
            }
            out
        }
        Fields::Unit => vec![],
    };

    // Check whether any field uses expansion — affects whether we can generate
    // IntoIterator (expanded fields change the companion struct shape).
    let has_expansion = resolved
        .iter()
        .any(|rf| !matches!(rf, ResolvedField::Single { .. }));
    // Track which Rust fields were skipped (for destructure patterns).
    let skipped_fields: Vec<syn::Ident> = match &data.fields {
        Fields::Named(fields) => fields
            .named
            .iter()
            .filter_map(|f| {
                let fa = parse_field_attrs(f).ok()?;
                if fa.skip {
                    Some(f.ident.as_ref().unwrap().clone())
                } else {
                    None
                }
            })
            .collect(),
        _ => vec![],
    };

    let has_tag = attrs.tag.is_some();
    let row_name_str = row_name.to_string();

    // ── Build flat column lists from resolved fields ─────────────────────
    // Each resolved field may produce 1..N columns.
    struct FlatCol {
        /// Companion struct field name.
        df_field: syn::Ident,
        /// Column name string in the R data frame.
        col_name_str: String,
        /// Type of the companion Vec<T>.
        vec_elem_ty: syn::Type,
    }

    let mut flat_cols: Vec<FlatCol> = Vec::new();

    for rf in &resolved {
        match rf {
            ResolvedField::Single {
                col_name,
                col_name_str,
                ty,
                ..
            } => {
                flat_cols.push(FlatCol {
                    df_field: col_name.clone(),
                    col_name_str: col_name_str.clone(),
                    vec_elem_ty: ty.clone(),
                });
            }
            ResolvedField::ExpandedFixed {
                base_name,
                elem_ty,
                len,
                ..
            } => {
                for i in 1..=*len {
                    let name = format!("{}_{}", base_name, i);
                    flat_cols.push(FlatCol {
                        df_field: format_ident!("{}_{}", base_name, i),
                        col_name_str: name,
                        vec_elem_ty: elem_ty.clone(),
                    });
                }
            }
            ResolvedField::ExpandedVec {
                base_name,
                elem_ty,
                width,
                ..
            } => {
                for i in 1..=*width {
                    let name = format!("{}_{}", base_name, i);
                    let opt_ty: syn::Type = syn::parse_quote!(Option<#elem_ty>);
                    flat_cols.push(FlatCol {
                        df_field: format_ident!("{}_{}", base_name, i),
                        col_name_str: name,
                        vec_elem_ty: opt_ty,
                    });
                }
            }
        }
    }

    // ── Companion struct ────────────────────────────────────────────────
    let tag_field_decl = if has_tag {
        quote! { pub _tag: Vec<String>, }
    } else {
        TokenStream::new()
    };

    let df_fields_tokens: Vec<TokenStream> = flat_cols
        .iter()
        .map(|fc| {
            let name = &fc.df_field;
            let ty = &fc.vec_elem_ty;
            quote! { pub #name: Vec<#ty> }
        })
        .collect();

    let len_field_decl = if flat_cols.is_empty() && !has_tag {
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
    let length_ref = if has_tag {
        quote! { self._tag.len() }
    } else if flat_cols.is_empty() {
        quote! { self._len }
    } else {
        let first = &flat_cols[0].df_field;
        quote! { self.#first.len() }
    };

    let tag_pair = if let Some(ref tag_name) = attrs.tag {
        quote! { (#tag_name, ::miniextendr_api::IntoR::into_sexp(self._tag)), }
    } else {
        TokenStream::new()
    };

    let df_pairs: Vec<TokenStream> = flat_cols
        .iter()
        .map(|fc| {
            let name = &fc.df_field;
            let name_str = &fc.col_name_str;
            quote! { (#name_str, ::miniextendr_api::IntoR::into_sexp(self.#name)) }
        })
        .collect();

    let length_checks: Vec<TokenStream> = flat_cols
        .iter()
        .map(|fc| {
            let name = &fc.df_field;
            let name_str = &fc.col_name_str;
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
    let col_vec_inits: Vec<TokenStream> = flat_cols
        .iter()
        .map(|fc| {
            let name = &fc.df_field;
            let ty = &fc.vec_elem_ty;
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

    // Generate push statements for each resolved field
    let col_pushes: Vec<TokenStream> = resolved
        .iter()
        .map(|rf| match rf {
            ResolvedField::Single {
                rust_name,
                col_name,
                tuple_index,
                ..
            } => {
                let access = if let Some(idx) = tuple_index {
                    quote! { row.#idx }
                } else {
                    quote! { row.#rust_name }
                };
                quote! { #col_name.push(#access); }
            }
            ResolvedField::ExpandedFixed {
                rust_name,
                base_name,
                len,
                tuple_index,
                ..
            } => {
                let access = if let Some(idx) = tuple_index {
                    quote! { row.#idx }
                } else {
                    quote! { row.#rust_name }
                };
                let bind = format_ident!("__arr_{}", rust_name);
                let pushes: Vec<TokenStream> = (0..*len)
                    .map(|i| {
                        let col_ident = format_ident!("{}_{}", base_name, i + 1);
                        let idx = syn::Index::from(i);
                        quote! { #col_ident.push(#bind[#idx]); }
                    })
                    .collect();
                quote! {
                    let #bind = #access;
                    #(#pushes)*
                }
            }
            ResolvedField::ExpandedVec {
                rust_name,
                base_name,
                width,
                tuple_index,
                ..
            } => {
                let access = if let Some(idx) = tuple_index {
                    quote! { row.#idx }
                } else {
                    quote! { row.#rust_name }
                };
                let bind = format_ident!("__vec_{}", rust_name);
                let pushes: Vec<TokenStream> = (0..*width)
                    .map(|i| {
                        let col_ident = format_ident!("{}_{}", base_name, i + 1);
                        quote! { #col_ident.push(#bind.get(#i).cloned()); }
                    })
                    .collect();
                quote! {
                    let #bind = #access;
                    #(#pushes)*
                }
            }
        })
        .collect();

    let tag_struct_field = if has_tag {
        quote! { _tag, }
    } else {
        TokenStream::new()
    };

    let len_struct_field = if flat_cols.is_empty() && !has_tag {
        quote! { _len: len, }
    } else {
        TokenStream::new()
    };

    let col_struct_fields: Vec<TokenStream> = flat_cols
        .iter()
        .map(|fc| {
            let name = &fc.df_field;
            quote! { #name }
        })
        .collect();

    // Skip parallel path when expansion is used (would need rewrite of ColumnWriter logic)
    let field_info_for_parallel: Vec<(&syn::Ident, &syn::Type)> = if !has_expansion {
        flat_cols
            .iter()
            .map(|fc| (&fc.df_field, &fc.vec_elem_ty))
            .collect()
    } else {
        vec![]
    };
    let parallel_block = if parallel && !field_info_for_parallel.is_empty() {
        gen_parallel_struct_from(row_name, df_name, &field_info_for_parallel)
    } else {
        TokenStream::new()
    };

    // For skipped fields in destructure: bind to `_`
    let skip_bindings: Vec<TokenStream> = skipped_fields
        .iter()
        .map(|name| quote! { let _ = row.#name; })
        .collect();

    let from_vec_impl = quote! {
        impl From<Vec<#row_name>> for #df_name {
            fn from(rows: Vec<#row_name>) -> Self {
                let len = rows.len();
                #parallel_block
                #tag_init
                #(#col_vec_inits)*
                for row in rows {
                    #tag_push
                    #(#skip_bindings)*
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

    // ── IntoIterator (only for named non-empty structs without expansion) ─
    let can_iterate =
        !flat_cols.is_empty() && !is_tuple_struct && !is_unit_struct && !has_expansion;
    let into_iterator_impl = if can_iterate {
        let iterator_name = format_ident!("{}Iterator", df_name);

        let iter_field_decls: Vec<_> = flat_cols
            .iter()
            .map(|fc| {
                let name = &fc.df_field;
                let ty = &fc.vec_elem_ty;
                quote! { #name: std::vec::IntoIter<#ty> }
            })
            .collect();

        let destruct_pattern: Vec<_> = flat_cols
            .iter()
            .map(|fc| {
                let name = &fc.df_field;
                quote! { #name }
            })
            .collect();

        let mut iter_init_tokens = TokenStream::new();
        for (i, fc) in flat_cols.iter().enumerate() {
            let name = &fc.df_field;
            let ty = &fc.vec_elem_ty;
            if i > 0 {
                iter_init_tokens.extend(quote! { , });
            }
            iter_init_tokens.extend(quote! { #name: <Vec<#ty>>::into_iter(#name) });
        }

        // For next(): reconstruct original field names (col_name == rust_name for Single)
        let mut next_struct_tokens = TokenStream::new();
        for (i, rf) in resolved.iter().enumerate() {
            if let ResolvedField::Single {
                rust_name,
                col_name,
                ..
            } = rf
            {
                if i > 0 {
                    next_struct_tokens.extend(quote! { , });
                }
                next_struct_tokens.extend(quote! { #rust_name: self.#col_name.next()? });
            }
        }

        let ignore_tag = if has_tag {
            quote! { _tag: _, }
        } else {
            TokenStream::new()
        };

        // Add default values for skipped fields in the reconstructed struct
        let skip_defaults: Vec<TokenStream> = skipped_fields
            .iter()
            .map(|name| quote! { , #name: Default::default() })
            .collect();

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
                        #(#skip_defaults)*
                    })
                }
            }
        }
    } else {
        TokenStream::new()
    };

    // ── Associated methods ──────────────────────────────────────────────
    let from_dataframe_method = if can_iterate {
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
    // Skip for unit/empty structs, tuple structs, and structs with expansion
    let trait_check =
        if !flat_cols.is_empty() && !is_tuple_struct && !is_unit_struct && !has_expansion {
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

/// A resolved enum field — either a single column or expanded from an array/Vec.
enum EnumResolvedField {
    /// Single column contribution.
    Single {
        /// Column name in the schema.
        col_name: syn::Ident,
        /// Binding name used in destructure pattern.
        binding: syn::Ident,
        /// Original Rust field name (for named variants).
        rust_name: syn::Ident,
        /// Column type.
        ty: syn::Type,
    },
    /// Expanded from [T; N].
    ExpandedFixed {
        /// Base column name.
        base_name: String,
        /// Binding name.
        binding: syn::Ident,
        /// Original Rust field name.
        rust_name: syn::Ident,
        /// Element type.
        elem_ty: syn::Type,
        /// Array length.
        len: usize,
    },
    /// Expanded from Vec<T> with pinned width.
    ExpandedVec {
        /// Base column name.
        base_name: String,
        /// Binding name.
        binding: syn::Ident,
        /// Original Rust field name.
        rust_name: syn::Ident,
        /// Element type.
        elem_ty: syn::Type,
        /// Pinned width.
        width: usize,
    },
}

/// Parsed information about an enum variant.
struct VariantInfo {
    /// Variant name.
    name: syn::Ident,
    /// Shape of this variant.
    shape: VariantShape,
    /// Resolved fields (after applying field attrs + type classification).
    fields: Vec<EnumResolvedField>,
    /// Original Rust field names (for named variants) — needed for skipped fields in destructure.
    skipped_fields: Vec<syn::Ident>,
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
                                    return Err(syn::Error::new_spanned(
                                        &f.ty,
                                        "`expand` on Vec<T> requires `width = N`",
                                    ));
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
                                        "`expand` is only valid on `[T; N]` or `Vec<T>` fields",
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

    let mut columns: Vec<ResolvedColumn> = Vec::new();
    let mut col_index: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    /// Helper: register a single column in the schema.
    #[allow(clippy::too_many_arguments)]
    fn register_column(
        columns: &mut Vec<ResolvedColumn>,
        col_index: &mut std::collections::HashMap<String, usize>,
        col_name: &str,
        col_ty: &syn::Type,
        variant_idx: usize,
        variant_name: &syn::Ident,
        coerce_to_string: bool,
        string_ty: &syn::Type,
        error_span: Span,
    ) -> syn::Result<()> {
        if let Some(&idx) = col_index.get(col_name) {
            let existing = &columns[idx];
            if !existing.string_coerced && existing.ty != *col_ty {
                if coerce_to_string {
                    columns[idx].ty = string_ty.clone();
                    columns[idx].string_coerced = true;
                } else {
                    return Err(syn::Error::new(
                        error_span,
                        format!(
                            "type conflict for field `{}`: variant `{}` has a different type \
                             than a previous variant; \
                             use `#[dataframe(conflicts = \"string\")]` to coerce all conflicting fields to String",
                            col_name, variant_name
                        ),
                    ));
                }
            }
            columns[idx].present_in.push(variant_idx);
        } else {
            let idx = columns.len();
            columns.push(ResolvedColumn {
                col_name: format_ident!("{}", col_name),
                ty: col_ty.clone(),
                present_in: vec![variant_idx],
                string_coerced: false,
            });
            col_index.insert(col_name.to_string(), idx);
        }
        Ok(())
    }

    for (variant_idx, vi) in variant_infos.iter().enumerate() {
        for erf in &vi.fields {
            // Use the rust_name span for error reporting
            let err_span = match erf {
                EnumResolvedField::Single { rust_name, .. }
                | EnumResolvedField::ExpandedFixed { rust_name, .. }
                | EnumResolvedField::ExpandedVec { rust_name, .. } => rust_name.span(),
            };
            match erf {
                EnumResolvedField::Single { col_name, ty, .. } => {
                    register_column(
                        &mut columns,
                        &mut col_index,
                        &col_name.to_string(),
                        ty,
                        variant_idx,
                        &vi.name,
                        coerce_to_string,
                        &string_ty,
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
                        register_column(
                            &mut columns,
                            &mut col_index,
                            &name,
                            elem_ty,
                            variant_idx,
                            &vi.name,
                            coerce_to_string,
                            &string_ty,
                            err_span,
                        )?;
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
                        register_column(
                            &mut columns,
                            &mut col_index,
                            &name,
                            elem_ty,
                            variant_idx,
                            &vi.name,
                            coerce_to_string,
                            &string_ty,
                            err_span,
                        )?;
                    }
                }
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

            // Build push statements for each schema column.
            // For present columns: push Some(value), for absent: push None.
            // Expanded fields contribute multiple columns from one binding.

            // First, build a map of which schema columns this variant contributes to.
            let col_pushes: Vec<TokenStream> = columns
                .iter()
                .map(|col| {
                    let col_name = &col.col_name;
                    if col.present_in.contains(&variant_idx) {
                        // Find the binding for this column.
                        // For single fields: binding matches col_name directly.
                        // For expanded fields: binding is the array/vec, index by suffix.
                        let col_name_str = col_name.to_string();

                        // Search resolved fields for the source of this column
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
                                _ => {}
                            }
                        }
                        // Shouldn't reach here if schema is correct
                        quote! { #col_name.push(None); }
                    } else {
                        quote! { #col_name.push(None); }
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
                            | EnumResolvedField::ExpandedVec { rust_name, binding, .. } => {
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
                        }
                    }
                }
                VariantShape::Tuple => {
                    let field_bindings: Vec<TokenStream> = vi.fields.iter().map(|erf| {
                        let binding = match erf {
                            EnumResolvedField::Single { binding, .. }
                            | EnumResolvedField::ExpandedFixed { binding, .. }
                            | EnumResolvedField::ExpandedVec { binding, .. } => binding,
                        };
                        quote! { #binding }
                    }).collect();
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
