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
pub(super) struct DataFrameAttrs {
    /// Custom companion type name (default: `{TypeName}DataFrame`).
    pub(super) name: Option<syn::Ident>,
    /// Enum alignment mode — implicit for enums, accepted but not required.
    pub(super) align: bool,
    /// Tag column name for variant discriminator (also supported on structs).
    pub(super) tag: Option<String>,
    /// Emit rayon parallel fill path (only effective when `rayon` feature is enabled).
    pub(super) parallel: bool,
    /// Conflict resolution mode for type collisions across enum variants.
    /// Currently only "string" is supported: convert conflicting fields via `ToString`.
    pub(super) conflicts: Option<String>,
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
pub(super) struct FieldAttrs {
    /// Omit this field from the DataFrame.
    pub(super) skip: bool,
    /// Custom column name.
    pub(super) rename: Option<String>,
    /// Keep collection as single list column (suppress expansion).
    as_list: bool,
    /// Explicitly expand to suffixed columns.
    expand: bool,
    /// Pin expansion width for variable-length collections.
    pub(super) width: Option<usize>,
}

/// Parse field-level `#[dataframe(...)]` attributes from a `syn::Field`.
pub(super) fn parse_field_attrs(field: &syn::Field) -> syn::Result<FieldAttrs> {
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
            } else if meta.path.is_ident("expand") || meta.path.is_ident("unnest") {
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
                    "unknown field attribute; expected `skip`, `rename`, `as_list`, `expand`, `unnest`, or `width`",
                ))
            }
        })?;
    }

    // Validation: conflicting options
    if attrs.as_list && attrs.expand {
        return Err(syn::Error::new(
            field.ident.as_ref().map_or(Span::call_site(), |i| i.span()),
            "`as_list` and `expand`/`unnest` are mutually exclusive",
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
pub(super) enum FieldTypeKind<'a> {
    /// Single column (most types).
    Scalar,
    /// `[T; N]` — expands to N columns at compile time.
    FixedArray(&'a syn::Type, usize),
    /// `Vec<T>` — variable length, needs `width` for expansion.
    VariableVec(&'a syn::Type),
}

/// Classify a field type for DataFrame expansion.
pub(super) fn classify_field_type(ty: &syn::Type) -> FieldTypeKind<'_> {
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
    /// Auto-expanded Vec<T>: column count determined at runtime from max row length.
    AutoExpandVec {
        /// Rust field name (for row access).
        rust_name: syn::Ident,
        /// Companion struct field name (ident).
        col_name: syn::Ident,
        /// Column name base string (for suffixed column names).
        col_name_str: String,
        /// Element type T.
        elem_ty: syn::Type,
        /// Index in tuple struct.
        tuple_index: Option<syn::Index>,
    },
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
                Ok(Some(ResolvedField::AutoExpandVec {
                    rust_name,
                    col_name,
                    col_name_str,
                    elem_ty: elem_ty.clone(),
                    tuple_index,
                }))
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
                    "`expand`/`unnest` is only valid on `[T; N]` or `Vec<T>` fields",
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
            // AutoExpandVec does not produce FlatCols — handled separately.
            ResolvedField::AutoExpandVec { .. } => {}
        }
    }

    // ── Collect auto-expand fields ──────────────────────────────────────
    struct AutoExpandCol {
        /// Companion struct field name.
        df_field: syn::Ident,
        /// Element type T.
        elem_ty: syn::Type,
    }

    let auto_expand_cols: Vec<AutoExpandCol> = resolved
        .iter()
        .filter_map(|rf| {
            if let ResolvedField::AutoExpandVec {
                col_name_str,
                elem_ty,
                ..
            } = rf
            {
                Some(AutoExpandCol {
                    df_field: format_ident!("{}", col_name_str),
                    elem_ty: elem_ty.clone(),
                })
            } else {
                None
            }
        })
        .collect();
    let has_auto_expand = !auto_expand_cols.is_empty();

    // ── Companion struct ────────────────────────────────────────────────
    let tag_field_decl = if has_tag {
        quote! { pub _tag: Vec<String>, }
    } else {
        TokenStream::new()
    };

    let mut df_fields_tokens: Vec<TokenStream> = flat_cols
        .iter()
        .map(|fc| {
            let name = &fc.df_field;
            let ty = &fc.vec_elem_ty;
            quote! { pub #name: Vec<#ty> }
        })
        .collect();
    for ac in &auto_expand_cols {
        let name = &ac.df_field;
        let ty = &ac.elem_ty;
        df_fields_tokens.push(quote! { pub #name: Vec<Vec<#ty>> });
    }

    let len_field_decl = if flat_cols.is_empty() && auto_expand_cols.is_empty() && !has_tag {
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
    } else if !flat_cols.is_empty() {
        let first = &flat_cols[0].df_field;
        quote! { self.#first.len() }
    } else if !auto_expand_cols.is_empty() {
        let first = &auto_expand_cols[0].df_field;
        quote! { self.#first.len() }
    } else {
        quote! { self._len }
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

    let into_dataframe_impl = if has_auto_expand {
        // Dynamic pair building: iterate resolved fields in order,
        // emitting static pairs for flat columns and runtime-expanded
        // pairs for auto-expand fields.
        let tag_push_pair = if let Some(ref tag_name) = attrs.tag {
            quote! {
                __df_pairs.push((#tag_name.to_string(), ::miniextendr_api::IntoR::into_sexp(self._tag)));
            }
        } else {
            TokenStream::new()
        };

        let pair_pushes: Vec<TokenStream> = resolved
            .iter()
            .map(|rf| match rf {
                ResolvedField::Single {
                    col_name,
                    col_name_str,
                    ..
                } => {
                    quote! {
                        __df_pairs.push((
                            #col_name_str.to_string(),
                            ::miniextendr_api::IntoR::into_sexp(self.#col_name),
                        ));
                    }
                }
                ResolvedField::ExpandedFixed {
                    base_name, len, ..
                } => {
                    let pushes: Vec<TokenStream> = (1..=*len)
                        .map(|i| {
                            let name = format!("{}_{}", base_name, i);
                            let ident = format_ident!("{}_{}", base_name, i);
                            quote! {
                                __df_pairs.push((
                                    #name.to_string(),
                                    ::miniextendr_api::IntoR::into_sexp(self.#ident),
                                ));
                            }
                        })
                        .collect();
                    quote! { #(#pushes)* }
                }
                ResolvedField::ExpandedVec {
                    base_name, width, ..
                } => {
                    let pushes: Vec<TokenStream> = (1..=*width)
                        .map(|i| {
                            let name = format!("{}_{}", base_name, i);
                            let ident = format_ident!("{}_{}", base_name, i);
                            quote! {
                                __df_pairs.push((
                                    #name.to_string(),
                                    ::miniextendr_api::IntoR::into_sexp(self.#ident),
                                ));
                            }
                        })
                        .collect();
                    quote! { #(#pushes)* }
                }
                ResolvedField::AutoExpandVec {
                    col_name,
                    col_name_str,
                    elem_ty,
                    ..
                } => {
                    quote! {
                        {
                            let __auto = self.#col_name;
                            let __max = __auto.iter().map(|v| v.len()).max().unwrap_or(0);
                            let mut __cols: Vec<Vec<Option<#elem_ty>>> = (0..__max)
                                .map(|_| Vec::with_capacity(_n_rows))
                                .collect();
                            for __row_vec in &__auto {
                                for (__i, __col) in __cols.iter_mut().enumerate() {
                                    __col.push(__row_vec.get(__i).cloned());
                                }
                            }
                            for (__i, __col) in __cols.into_iter().enumerate() {
                                __df_pairs.push((
                                    format!("{}_{}", #col_name_str, __i + 1),
                                    ::miniextendr_api::IntoR::into_sexp(__col),
                                ));
                            }
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
                    #(#pair_pushes)*
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
                        #(#df_pairs),*
                    ])
                    .set_class_str(&["data.frame"])
                    .set_row_names_int(_n_rows)
                }
            }
        }
    };

    // ── From<Vec<RowType>> ──────────────────────────────────────────────
    let mut col_vec_inits: Vec<TokenStream> = flat_cols
        .iter()
        .map(|fc| {
            let name = &fc.df_field;
            let ty = &fc.vec_elem_ty;
            quote! { let mut #name: Vec<#ty> = Vec::with_capacity(len); }
        })
        .collect();
    for ac in &auto_expand_cols {
        let name = &ac.df_field;
        let ty = &ac.elem_ty;
        col_vec_inits.push(
            quote! { let mut #name: Vec<Vec<#ty>> = Vec::with_capacity(len); },
        );
    }

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
            ResolvedField::AutoExpandVec {
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
        })
        .collect();

    let tag_struct_field = if has_tag {
        quote! { _tag, }
    } else {
        TokenStream::new()
    };

    let len_struct_field =
        if flat_cols.is_empty() && auto_expand_cols.is_empty() && !has_tag {
            quote! { _len: len, }
        } else {
            TokenStream::new()
        };

    let mut col_struct_fields: Vec<TokenStream> = flat_cols
        .iter()
        .map(|fc| {
            let name = &fc.df_field;
            quote! { #name }
        })
        .collect();
    for ac in &auto_expand_cols {
        let name = &ac.df_field;
        col_struct_fields.push(quote! { #name });
    }

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
pub(super) struct ResolvedColumn {
    /// Column name in the companion struct / data frame.
    pub(super) col_name: syn::Ident,
    /// Element type (used as `Vec<Option<#ty>>`).
    /// When `string_coerced` is true, this is always `String`.
    pub(super) ty: syn::Type,
    /// Indices of variants that contain this field.
    pub(super) present_in: Vec<usize>,
    /// Whether this column was coerced to `String` due to type conflicts.
    /// When true, values are converted via `ToString::to_string()` at push time.
    pub(super) string_coerced: bool,
}

/// Accumulates unique columns for an enum-to-dataframe schema.
pub(super) struct ColumnRegistry<'a> {
    pub(super) columns: Vec<ResolvedColumn>,
    pub(super) col_index: std::collections::HashMap<String, usize>,
    pub(super) coerce_to_string: bool,
    pub(super) string_ty: &'a syn::Type,
}

impl<'a> ColumnRegistry<'a> {
    fn new(coerce_to_string: bool, string_ty: &'a syn::Type) -> Self {
        Self {
            columns: Vec::new(),
            col_index: std::collections::HashMap::new(),
            coerce_to_string,
            string_ty,
        }
    }

    /// Register a single column in the schema.
    fn register(
        &mut self,
        col_name: &str,
        col_ty: &syn::Type,
        variant_idx: usize,
        variant_name: &syn::Ident,
        error_span: Span,
    ) -> syn::Result<()> {
        if let Some(&idx) = self.col_index.get(col_name) {
            let existing = &self.columns[idx];
            if !existing.string_coerced && existing.ty != *col_ty {
                if self.coerce_to_string {
                    self.columns[idx].ty = self.string_ty.clone();
                    self.columns[idx].string_coerced = true;
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
            self.columns[idx].present_in.push(variant_idx);
        } else {
            let idx = self.columns.len();
            self.columns.push(ResolvedColumn {
                col_name: format_ident!("{}", col_name),
                ty: col_ty.clone(),
                present_in: vec![variant_idx],
                string_coerced: false,
            });
            self.col_index.insert(col_name.to_string(), idx);
        }
        Ok(())
    }
}

/// Describes the shape of an enum variant's fields.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum VariantShape {
    /// `Variant { field: Type, ... }`
    Named,
    /// `Variant(Type, ...)`
    Tuple,
    /// `Variant` (no fields)
    Unit,
}

/// A resolved enum field — either a single column or expanded from an array/Vec.
pub(super) enum EnumResolvedField {
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
    /// Auto-expanded Vec<T>: column count determined at runtime.
    AutoExpandVec {
        /// Base column name.
        base_name: String,
        /// Binding name.
        binding: syn::Ident,
        /// Original Rust field name.
        rust_name: syn::Ident,
        /// Element type.
        elem_ty: syn::Type,
    },
}

/// Parsed information about an enum variant.
pub(super) struct VariantInfo {
    /// Variant name.
    pub(super) name: syn::Ident,
    /// Shape of this variant.
    pub(super) shape: VariantShape,
    /// Resolved fields (after applying field attrs + type classification).
    pub(super) fields: Vec<EnumResolvedField>,
    /// Original Rust field names (for named variants) — needed for skipped fields in destructure.
    pub(super) skipped_fields: Vec<syn::Ident>,
}

// =============================================================================
// Enum-specific expansion (in sub-module)
// =============================================================================

mod enum_expansion;
use enum_expansion::{derive_enum_dataframe, gen_parallel_struct_from};
