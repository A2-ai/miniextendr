//! Enum-specific DataFrame derive expansion.
//!
//! Generates a companion struct where every column is `Vec<Option<T>>`, with
//! `None` fill for fields absent in a given variant.

use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{DeriveInput, Fields};

use super::{
    ColumnRegistry, DataFrameAttrs, EnumAutoExpandVecData, EnumExpandedFixedData,
    EnumExpandedVecData, EnumMapFieldData, EnumResolvedField, EnumSingleFieldData,
    EnumStructFieldData, FieldTypeKind, VariantInfo, VariantShape, classify_field_type,
    parse_field_attrs,
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
                        // Struct-typed fields with `as_list` must be converted via `into_list()`
                        // at `into_data_frame` time. We keep the original Rust type in the
                        // companion struct (so no R API is called during row accumulation) and
                        // flag `needs_into_list = true` to trigger per-element conversion in the
                        // dynamic `into_data_frame` path.
                        let needs_into_list =
                            matches!(classify_field_type(&f.ty), FieldTypeKind::Struct { .. });
                        resolved.push(EnumResolvedField::Single(Box::new(EnumSingleFieldData {
                            col_name: format_ident!("{}", col_name_str),
                            binding: binding.clone(),
                            rust_name: rust_name.clone(),
                            ty: f.ty.clone(),
                            needs_into_list,
                            is_factor: false,
                        })));
                    } else if fa.as_factor {
                        // `as_factor` is only valid on bare-ident enum types (Struct kind).
                        // The inner enum must be unit-only and derive DataFrameRow, which
                        // auto-emits UnitEnumFactor so FactorOptionVec<T> implements IntoR.
                        match classify_field_type(&f.ty) {
                            FieldTypeKind::Struct { .. } => {
                                resolved.push(EnumResolvedField::Single(Box::new(
                                    EnumSingleFieldData {
                                        col_name: format_ident!("{}", col_name_str),
                                        binding: binding.clone(),
                                        rust_name: rust_name.clone(),
                                        ty: f.ty.clone(),
                                        needs_into_list: false,
                                        is_factor: true,
                                    },
                                )));
                            }
                            _ => {
                                return Err(syn::Error::new_spanned(
                                    &f.ty,
                                    "`as_factor` is only valid on bare-ident enum/struct types; \
                                     use `as_list` for generic or complex types, or remove \
                                     `as_factor` for scalar fields",
                                ));
                            }
                        }
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
                                            needs_into_list: false,
                                            is_factor: false,
                                        },
                                    )));
                                }
                            }
                            FieldTypeKind::Map { key_ty, val_ty } => {
                                if fa.width.is_some() {
                                    return Err(syn::Error::new_spanned(
                                        &f.ty,
                                        "`width` is not valid on HashMap/BTreeMap fields",
                                    ));
                                }
                                if fa.expand {
                                    return Err(syn::Error::new_spanned(
                                        &f.ty,
                                        "`expand`/`unnest` is not valid on HashMap/BTreeMap fields",
                                    ));
                                }
                                resolved.push(EnumResolvedField::Map(Box::new(EnumMapFieldData {
                                    base_name: col_name_str,
                                    binding: binding.clone(),
                                    rust_name: rust_name.clone(),
                                    key_ty: key_ty.clone(),
                                    val_ty: val_ty.clone(),
                                })));
                            }
                            FieldTypeKind::Struct { inner_ty } => {
                                if fa.width.is_some() {
                                    return Err(syn::Error::new_spanned(
                                        &f.ty,
                                        "`width` is not valid on struct fields; use \
                                         `#[dataframe(as_list)]` to keep as an opaque list-column",
                                    ));
                                }
                                if fa.expand {
                                    return Err(syn::Error::new_spanned(
                                        &f.ty,
                                        "`expand`/`unnest` is not valid on struct fields; struct \
                                         fields flatten by default via their `DataFrameRow` impl",
                                    ));
                                }
                                resolved.push(EnumResolvedField::Struct(Box::new(
                                    EnumStructFieldData {
                                        base_name: col_name_str,
                                        binding: binding.clone(),
                                        rust_name: rust_name.clone(),
                                        inner_ty: inner_ty.clone(),
                                    },
                                )));
                            }
                            FieldTypeKind::Scalar => {
                                resolved.push(EnumResolvedField::Single(Box::new(
                                    EnumSingleFieldData {
                                        col_name: format_ident!("{}", col_name_str),
                                        binding: binding.clone(),
                                        rust_name: rust_name.clone(),
                                        ty: f.ty.clone(),
                                        needs_into_list: false,
                                        is_factor: false,
                                    },
                                )));
                            }
                        }
                    }
                }
                // B1: Check for `<base>_<inner_tag>` discriminant column collision.
                //
                // When a Struct field `kind: Inner` is flattened, the inner enum's
                // discriminant column (tag) is emitted under `<base>_<inner_tag>`.
                // The inner tag is retrieved at runtime from
                // `<Inner as DataFramePayloadFields>::TAG`; the B1 check here uses the
                // hardcoded default `"variant"` for compile-time sibling detection because
                // we cannot inspect inner enum attributes from the outer macro parse phase.
                // The per-inner-field payload collision is caught separately via the
                // `const _:` assertions emitted below (using `DataFramePayloadFields`).
                //
                // We detect the following cases at compile time (both using "variant"):
                //   1. Struct field `kind: Inner` + Single/Scalar sibling named `kind_variant`
                //   2. Struct field `kind: Inner` + another Struct sibling field renamed to
                //      produce `kind_variant`
                //
                // Inner-enum-internal collision (Inner has both `tag = "X"` AND payload
                // field `X`) is caught by the `assert_no_payload_field_collision` const
                // assertion emitted below — no carve-out needed.
                {
                    // Collect every flat column name produced by non-Struct resolved fields.
                    let flat_col_names: Vec<String> = resolved
                        .iter()
                        .filter_map(|r| match r {
                            EnumResolvedField::Single(d) => Some(d.col_name.to_string()),
                            EnumResolvedField::Map(d) => {
                                // Map fields produce <base>_keys and <base>_values.
                                // Neither collides with <struct>_variant unless someone
                                // explicitly renamed to match — covered by the Struct check
                                // via base_name.
                                let _ = d;
                                None
                            }
                            _ => None,
                        })
                        .collect();

                    for r in &resolved {
                        if let EnumResolvedField::Struct(struct_data) = r {
                            // Use hardcoded "variant" for the sibling check — this is the
                            // default inner tag. The inner-payload collision for non-default
                            // tags is caught by assert_no_payload_field_collision below.
                            let discriminant_col = format!("{}_variant", struct_data.base_name);
                            if flat_col_names.contains(&discriminant_col) {
                                // Find the colliding field for a better span.
                                let colliding_span = resolved
                                    .iter()
                                    .find_map(|r2| match r2 {
                                        EnumResolvedField::Single(d)
                                            if d.col_name == discriminant_col.as_str() =>
                                        {
                                            Some(d.col_name.span())
                                        }
                                        _ => None,
                                    })
                                    .unwrap_or_else(proc_macro2::Span::call_site);
                                return Err(syn::Error::new(
                                    colliding_span,
                                    format!(
                                        "column name collision: the flatten field `{base}` \
                                         (a nested `DataFrameRow` enum) will emit a \
                                         discriminant column named `{disc}`, but a sibling \
                                         field already produces a column with the same name. \
                                         Rename the sibling field or use \
                                         `#[dataframe(tag = \"...\")]` on the inner enum to \
                                         choose a different discriminant column name \
                                         (e.g. `#[dataframe(tag = \"type\")]` → `{base}_type`)",
                                        base = struct_data.base_name,
                                        disc = discriminant_col,
                                    ),
                                ));
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
                        let needs_into_list =
                            matches!(classify_field_type(&f.ty), FieldTypeKind::Struct { .. });
                        resolved.push(EnumResolvedField::Single(Box::new(EnumSingleFieldData {
                            col_name: format_ident!("{}", col_name_str),
                            binding,
                            rust_name,
                            ty: f.ty.clone(),
                            needs_into_list,
                            is_factor: false,
                        })));
                    } else if fa.as_factor {
                        match classify_field_type(&f.ty) {
                            FieldTypeKind::Struct { .. } => {
                                resolved.push(EnumResolvedField::Single(Box::new(
                                    EnumSingleFieldData {
                                        col_name: format_ident!("{}", col_name_str),
                                        binding,
                                        rust_name,
                                        ty: f.ty.clone(),
                                        needs_into_list: false,
                                        is_factor: true,
                                    },
                                )));
                            }
                            _ => {
                                return Err(syn::Error::new_spanned(
                                    &f.ty,
                                    "`as_factor` is only valid on bare-ident enum/struct types; \
                                     use `as_list` for generic or complex types, or remove \
                                     `as_factor` for scalar fields",
                                ));
                            }
                        }
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
                                            needs_into_list: false,
                                            is_factor: false,
                                        },
                                    )));
                                }
                            }
                            FieldTypeKind::Map { key_ty, val_ty } => {
                                if fa.width.is_some() {
                                    return Err(syn::Error::new_spanned(
                                        &f.ty,
                                        "`width` is not valid on HashMap/BTreeMap fields",
                                    ));
                                }
                                if fa.expand {
                                    return Err(syn::Error::new_spanned(
                                        &f.ty,
                                        "`expand`/`unnest` is not valid on HashMap/BTreeMap fields",
                                    ));
                                }
                                resolved.push(EnumResolvedField::Map(Box::new(EnumMapFieldData {
                                    base_name: col_name_str,
                                    binding,
                                    rust_name,
                                    key_ty: key_ty.clone(),
                                    val_ty: val_ty.clone(),
                                })));
                            }
                            FieldTypeKind::Struct { inner_ty } => {
                                if fa.width.is_some() {
                                    return Err(syn::Error::new_spanned(
                                        &f.ty,
                                        "`width` is not valid on struct fields; use `#[dataframe(as_list)]` \
                                         to keep as an opaque list-column",
                                    ));
                                }
                                if fa.expand {
                                    return Err(syn::Error::new_spanned(
                                        &f.ty,
                                        "`expand`/`unnest` is not valid on struct fields; struct fields \
                                         flatten by default via their DataFrameRow impl",
                                    ));
                                }
                                resolved.push(EnumResolvedField::Struct(Box::new(
                                    EnumStructFieldData {
                                        base_name: col_name_str,
                                        binding,
                                        rust_name,
                                        inner_ty: inner_ty.clone(),
                                    },
                                )));
                            }
                            FieldTypeKind::Scalar => {
                                resolved.push(EnumResolvedField::Single(Box::new(
                                    EnumSingleFieldData {
                                        col_name: format_ident!("{}", col_name_str),
                                        binding,
                                        rust_name,
                                        ty: f.ty.clone(),
                                        needs_into_list: false,
                                        is_factor: false,
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
                    if data.is_factor {
                        registry.register_factor(
                            &data.col_name.to_string(),
                            &data.ty,
                            variant_idx,
                            &vi.name,
                            err_span,
                        )?;
                    } else {
                        registry.register(
                            &data.col_name.to_string(),
                            &data.ty,
                            variant_idx,
                            &vi.name,
                            err_span,
                        )?;
                    }
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
                EnumResolvedField::Map(data) => {
                    let key_ty = &data.key_ty;
                    let val_ty = &data.val_ty;
                    let keys_name = format!("{}_keys", data.base_name);
                    let vals_name = format!("{}_values", data.base_name);
                    // Column types are Vec<K> and Vec<V> respectively (used as Vec<Option<Vec<K>>>
                    // / Vec<Option<Vec<V>>> in companion struct via ColumnRegistry wrapping).
                    let key_vec_ty: syn::Type = syn::parse_quote!(Vec<#key_ty>);
                    let val_vec_ty: syn::Type = syn::parse_quote!(Vec<#val_ty>);
                    registry.register(&keys_name, &key_vec_ty, variant_idx, &vi.name, err_span)?;
                    registry.register(&vals_name, &val_vec_ty, variant_idx, &vi.name, err_span)?;
                }
                // Struct: registers one Vec<Option<Inner>> column under base_name.
                // Flattening into prefixed columns happens at into_data_frame() time, not here.
                EnumResolvedField::Struct(data) => {
                    let inner_ty = &data.inner_ty;
                    // Register as Option<Inner>; the column in the companion struct is Vec<Option<Inner>>.
                    registry.register(
                        &data.base_name,
                        inner_ty,
                        variant_idx,
                        &vi.name,
                        err_span,
                    )?;
                }
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

    // region: Collect struct fields (for bespoke into_data_frame flatten)
    struct EnumStructCol {
        /// Companion struct field name (matches base_name in registry).
        df_field: syn::Ident,
        /// Column prefix (same as df_field, used to prefix inner col names).
        base_name: String,
        /// Inner type.
        inner_ty: syn::Type,
    }

    let mut struct_cols: Vec<EnumStructCol> = Vec::new();
    let mut struct_col_index: HashMap<String, bool> = HashMap::new();

    for vi in &variant_infos {
        for erf in &vi.fields {
            if let EnumResolvedField::Struct(data) = erf
                && !struct_col_index.contains_key(&data.base_name)
            {
                struct_col_index.insert(data.base_name.clone(), true);
                struct_cols.push(EnumStructCol {
                    df_field: format_ident!("{}", data.base_name),
                    base_name: data.base_name.clone(),
                    inner_ty: data.inner_ty.clone(),
                });
            }
        }
    }
    let has_struct_cols = !struct_cols.is_empty();
    // endregion

    // region: Collect as_list struct fields (Single fields that need per-element into_list())
    //
    // These are Single fields with `needs_into_list = true`: struct-typed fields that carry
    // `#[dataframe(as_list)]`. The companion struct stores `Vec<Option<T>>` (raw Rust struct),
    // but `into_data_frame` must convert each element via `.into_list()` before building the SEXP.
    // We collect them so we can:
    //   a) Force the dynamic `into_data_frame` path (they need per-element conversion, not IntoR).
    //   b) Emit the per-element conversion in the dynamic path.
    let mut as_list_struct_col_names: std::collections::HashSet<String> =
        std::collections::HashSet::new();
    for vi in &variant_infos {
        for erf in &vi.fields {
            if let EnumResolvedField::Single(data) = erf
                && data.needs_into_list
            {
                as_list_struct_col_names.insert(data.col_name.to_string());
            }
        }
    }
    let has_as_list_struct_cols = !as_list_struct_col_names.is_empty();

    // Collect factor column names (Single fields with `is_factor = true`).
    // These are emitted via `FactorOptionVec<T>` wrapping in `into_data_frame`.
    let mut factor_col_names: std::collections::HashSet<String> = std::collections::HashSet::new();
    for vi in &variant_infos {
        for erf in &vi.fields {
            if let EnumResolvedField::Single(data) = erf
                && data.is_factor
            {
                factor_col_names.insert(data.col_name.to_string());
            }
        }
    }
    let has_factor_cols = !factor_col_names.is_empty();
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

    // When the companion struct would otherwise have no fields (unit-only enum,
    // no tag) but has generic type parameters, emit a PhantomData field to keep
    // the type parameter in scope — without it the struct is E0392 (unused param).
    let has_any_field = has_tag || !df_fields.is_empty();
    let phantom_field = if !has_any_field && !impl_generics.to_token_stream().is_empty() {
        let type_params: Vec<_> = input.generics.type_params().map(|tp| &tp.ident).collect();
        if !type_params.is_empty() {
            quote! {
                #[allow(dead_code)]
                _phantom: ::std::marker::PhantomData<(#(#type_params,)*)>,
            }
        } else {
            TokenStream::new()
        }
    } else {
        TokenStream::new()
    };

    let dataframe_struct = quote! {
        #[derive(Debug, Clone)]
        pub struct #df_name #impl_generics #where_clause {
            #tag_field
            #(#df_fields),*
            #phantom_field
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

    // Each pair protects its SEXP via `__scope.protect_raw` so previously-built
    // column SEXPs survive subsequent column allocations. Pre-fix the raw
    // `vec![(name, into_sexp(...)), ...]` left every SEXP unrooted across the
    // next column's allocations — UAF under gctorture
    // (reviews/2026-05-07-gctorture-audit.md).
    let tag_pair = if let Some(ref tag_name) = attrs.tag {
        quote! { (#tag_name, __scope.protect_raw(::miniextendr_api::IntoR::into_sexp(self._tag))), }
    } else {
        TokenStream::new()
    };

    let col_pairs: Vec<TokenStream> = columns
        .iter()
        .map(|col| {
            let name = &col.col_name;
            let name_str = name.to_string();
            quote! { (#name_str, __scope.protect_raw(::miniextendr_api::IntoR::into_sexp(self.#name))) }
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

    // Build the set of column names that are struct-col placeholders (to skip in normal push).
    let struct_col_names: std::collections::HashSet<String> =
        struct_cols.iter().map(|sc| sc.base_name.clone()).collect();

    let into_dataframe_impl = if has_enum_auto_expand
        || has_struct_cols
        || has_as_list_struct_cols
        || has_factor_cols
    {
        // Dynamic pair building for auto-expand, struct fields, as_list struct fields,
        // and/or as_factor fields.
        let tag_push_pair = if let Some(ref tag_name) = attrs.tag {
            quote! {
                __df_pairs.push((
                    #tag_name.to_string(),
                    __scope.protect_raw(::miniextendr_api::IntoR::into_sexp(self._tag)),
                ));
            }
        } else {
            TokenStream::new()
        };

        // Static columns — skip struct-col placeholders (handled in flatten block below),
        // as-list struct fields (handled in the per-element conversion block below),
        // and factor columns (handled in the FactorOptionVec wrapping block below).
        let static_pair_pushes: Vec<TokenStream> = columns
            .iter()
            .filter(|col| {
                let name_str = col.col_name.to_string();
                !struct_col_names.contains(&name_str)
                    && !as_list_struct_col_names.contains(&name_str)
                    && !factor_col_names.contains(&name_str)
            })
            .map(|col| {
                let name = &col.col_name;
                let name_str = name.to_string();
                quote! {
                    __df_pairs.push((
                        #name_str.to_string(),
                        __scope.protect_raw(::miniextendr_api::IntoR::into_sexp(self.#name)),
                    ));
                }
            })
            .collect();

        // as_list struct fields: convert each element via into_list() at conversion time
        // (not during row accumulation), producing a VECSXP list-column with NULL for absent rows.
        let as_list_struct_pushes: Vec<TokenStream> = columns
            .iter()
            .filter(|col| as_list_struct_col_names.contains(&col.col_name.to_string()))
            .map(|col| {
                let name = &col.col_name;
                let name_str = name.to_string();
                let ty = &col.ty;
                quote! {
                    {
                        // Map Vec<Option<T>> → Vec<Option<List>> then convert to SEXP.
                        // This is the only R-touching operation for as_list struct fields.
                        let __as_list_col: Vec<Option<::miniextendr_api::list::List>> =
                            self.#name
                                .into_iter()
                                .map(|__opt: Option<#ty>| {
                                    __opt.map(|v| ::miniextendr_api::list::IntoList::into_list(v))
                                })
                                .collect();
                        __df_pairs.push((
                            #name_str.to_string(),
                            __scope.protect_raw(::miniextendr_api::IntoR::into_sexp(__as_list_col)),
                        ));
                    }
                }
            })
            .collect();

        // as_factor columns: wrap Vec<Option<T>> in FactorOptionVec<T> before calling into_sexp.
        // Uses the UnitEnumFactor blanket impl: impl<T: UnitEnumFactor> IntoR for FactorOptionVec<T>.
        let factor_pair_pushes: Vec<TokenStream> = columns
            .iter()
            .filter(|col| factor_col_names.contains(&col.col_name.to_string()))
            .map(|col| {
                let name = &col.col_name;
                let name_str = name.to_string();
                let ty = &col.ty;
                quote! {
                    __df_pairs.push((
                        #name_str.to_string(),
                        __scope.protect_raw(::miniextendr_api::IntoR::into_sexp(
                            ::miniextendr_api::factor::FactorOptionVec::<#ty>::from(self.#name)
                        )),
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
                                __scope.protect_raw(::miniextendr_api::IntoR::into_sexp(__col)),
                            ));
                        }
                    }
                }
            })
            .collect();

        // Struct field flatten blocks: for each Vec<Option<Inner>> column, collect present
        // rows into a dense Vec<Inner>, track presence indices, call Inner::to_dataframe,
        // extract named columns via into_named_columns(), scatter them to full row count
        // with None-fill, and push with prefixed names.
        let struct_flatten_pushes: Vec<TokenStream> = struct_cols
            .iter()
            .map(|sc| {
                let df_field = &sc.df_field;
                let base_name_str = &sc.base_name;
                let inner_ty = &sc.inner_ty;
                quote! {
                    {
                        // Separate the Some/None rows — collect present rows densely
                        // (no Clone needed: we consume the Vec<Option<Inner>>).
                        let mut __present_idx: Vec<usize> = Vec::new();
                        let mut __inner_rows: Vec<#inner_ty> = Vec::new();
                        for (__row_i, __opt) in self.#df_field.into_iter().enumerate() {
                            if let Some(__inner) = __opt {
                                __present_idx.push(__row_i);
                                __inner_rows.push(__inner);
                            }
                        }
                        // Call Inner::to_dataframe and extract named column SEXPs.
                        let __inner_df = <#inner_ty>::to_dataframe(__inner_rows);
                        // into_named_columns consumes __inner_df and returns (name, SEXP) pairs.
                        let __inner_cols = ::miniextendr_api::convert::IntoDataFrame::into_named_columns(__inner_df);
                        // Scatter each column back to full _n_rows with NA/NULL-fill,
                        // preserving the source column's SEXPTYPE.
                        for (__inner_col_name, __inner_col_sexp) in __inner_cols {
                            // Protect the source column across the scatter allocation.
                            let __src = __scope.protect_raw(__inner_col_sexp);
                            let __prefixed = format!("{}_{}", #base_name_str, __inner_col_name);
                            let __scattered = unsafe {
                                let __out = ::miniextendr_api::convert::scatter_column(
                                    __src,
                                    &__present_idx,
                                    _n_rows,
                                );
                                __scope.protect_raw(__out)
                            };
                            __df_pairs.push((__prefixed, __scattered));
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
                    // SAFETY: into_data_frame only runs on the R main thread.
                    // ProtectScope keeps each column SEXP rooted across the
                    // next column's allocations; from_raw_pairs writes them
                    // into the parent VECSXP before we drop the scope.
                    unsafe {
                        let __scope = ::miniextendr_api::gc_protect::ProtectScope::new();
                        let mut __df_pairs: Vec<(
                            String,
                            ::miniextendr_api::ffi::SEXP,
                        )> = Vec::new();
                        #tag_push_pair
                        #(#static_pair_pushes)*
                        #(#factor_pair_pushes)*
                        #(#auto_expand_pair_pushes)*
                        #(#struct_flatten_pushes)*
                        #(#as_list_struct_pushes)*
                        ::miniextendr_api::list::List::from_raw_pairs(__df_pairs)
                            .set_class_str(&["data.frame"])
                            .set_row_names_int(_n_rows)
                    }
                }
            }
        }
    } else {
        quote! {
            impl #impl_generics ::miniextendr_api::convert::IntoDataFrame for #df_name #ty_generics #where_clause {
                fn into_data_frame(self) -> ::miniextendr_api::List {
                    let _n_rows = #length_ref;
                    #(#length_checks)*
                    // SAFETY: see auto-expand branch.
                    unsafe {
                        let __scope = ::miniextendr_api::gc_protect::ProtectScope::new();
                        // Explicit type annotation so the vec![] case (unit-only enum
                        // with no columns and no tag) doesn't hit E0282 inference failure.
                        let __pairs: Vec<(&str, ::miniextendr_api::ffi::SEXP)> = vec![
                            #tag_pair
                            #(#col_pairs),*
                        ];
                        ::miniextendr_api::list::List::from_raw_pairs(__pairs)
                        .set_class_str(&["data.frame"])
                        .set_row_names_int(_n_rows)
                    }
                }
            }
        }
    };

    // Compile-time assertions: one per struct field, asserting the inner type
    // implements DataFrameRow.
    let struct_assertions: Vec<TokenStream> = struct_cols
        .iter()
        .map(|sc| {
            let inner_ty = &sc.inner_ty;
            quote! {
                const _: () = {
                    fn _assert_inner_is_dataframe_row<T: ::miniextendr_api::markers::DataFrameRow>() {}
                    fn _do_assert #impl_generics () #where_clause {
                        _assert_inner_is_dataframe_row::<#inner_ty>();
                    }
                };
            }
        })
        .collect();

    // Payload collision assertions (#486): one per nested-enum struct field.
    // For each `kind: Inner` field, emit:
    //   const _: () = ::miniextendr_api::markers::assert_no_payload_field_collision(
    //       <Inner as DataFramePayloadFields>::FIELDS,
    //       <Inner as DataFramePayloadFields>::TAG,
    //   );
    // This fires a compile-time panic if any inner payload field name equals the
    // inner enum's own tag suffix, which would (after outer prefix expansion) produce
    // a column name identical to the outer discriminant column.
    let payload_collision_assertions: Vec<TokenStream> = struct_cols
        .iter()
        .map(|sc| {
            let inner_ty = &sc.inner_ty;
            quote! {
                const _: () = ::miniextendr_api::markers::assert_no_payload_field_collision(
                    <#inner_ty as ::miniextendr_api::markers::DataFramePayloadFields>::FIELDS,
                    <#inner_ty as ::miniextendr_api::markers::DataFramePayloadFields>::TAG,
                );
            }
        })
        .collect();
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
                                EnumResolvedField::Map(data) => {
                                    let keys_name = format!("{}_keys", data.base_name);
                                    let vals_name = format!("{}_values", data.base_name);
                                    let binding = &data.binding;
                                    // Use unzip() to guarantee pairwise alignment of keys and values.
                                    // Both columns are emitted together when the _keys column is
                                    // processed; the _values column is skipped (already handled).
                                    if col_name_str == keys_name {
                                        let vals_col = format_ident!("{}", vals_name);
                                        return quote! {
                                            let (__mx_keys, __mx_vals) = #binding.into_iter().unzip::<_, _, Vec<_>, Vec<_>>();
                                            #col_name.push(Some(__mx_keys));
                                            #vals_col.push(Some(__mx_vals));
                                        };
                                    }
                                    if col_name_str == vals_name {
                                        // Already handled when keys col was processed; emit no-op.
                                        return quote! {};
                                    }
                                }
                                // Struct field: push Some(binding) to the Vec<Option<Inner>> column.
                                EnumResolvedField::Struct(data)
                                    if data.base_name == col_name_str =>
                                {
                                    let binding = &data.binding;
                                    return quote! { #col_name.push(Some(#binding)); };
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

    // Struct literal initializer for the PhantomData field, when emitted.
    //
    // `phantom_field` is:
    //   - Empty when the companion struct has at least one real field (tag or
    //     column), or when there are no generic type parameters (const-param
    //     enums don't need PhantomData — Rust allows unused const params).
    //   - Non-empty only when the struct would otherwise have *zero* fields AND
    //     the enum carries at least one type parameter `T`, where the generated
    //     `PhantomData<T>` field prevents E0392 ("unused type parameter") on the
    //     companion struct.  In practice this path is only reachable if the user
    //     somehow has a type-generic unit-only enum; Rust's own E0392 rule blocks
    //     such enums at the user-definition level, so this branch is a defensive
    //     guard for hypothetical macro-generated enum inputs.
    let phantom_struct_field_init = if phantom_field.is_empty() {
        TokenStream::new()
    } else {
        quote! { _phantom: ::std::marker::PhantomData, }
    };

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
                    #phantom_struct_field_init
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
                                    EnumResolvedField::Map(data) => {
                                        let keys_name = format!("{}_keys", data.base_name);
                                        let vals_name = format!("{}_values", data.base_name);
                                        let binding = &data.binding;
                                        // Combined unzip: emit both key and value writes when the
                                        // keys column is processed; skip the values column (handled here).
                                        if col_name_str == keys_name {
                                            let vals_col = format_ident!("{}", vals_name);
                                            let w_vals = format_ident!("__w_{}", vals_col);
                                            return quote! {
                                                let (__mx_keys, __mx_vals) = #binding.into_iter().unzip::<_, _, Vec<_>, Vec<_>>();
                                                #w_name.write(__i, Some(__mx_keys));
                                                #w_vals.write(__i, Some(__mx_vals));
                                            };
                                        }
                                        if col_name_str == vals_name {
                                            // Already handled when keys col was processed.
                                            return quote! {};
                                        }
                                    }
                                    // Struct field: write Some(binding) to Vec<Option<Inner>>.
                                    EnumResolvedField::Struct(data)
                                        if data.base_name == col_name_str =>
                                    {
                                        let binding = &data.binding;
                                        return quote! { #w_name.write(__i, Some(#binding)); };
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

    // Marker trait impl: row type implements DataFrameRow via IntoDataFrame chain.
    // This is the impl the compile-time assertion checks for struct-typed variant fields.
    let marker_impl = quote! {
        impl #impl_generics ::miniextendr_api::markers::DataFrameRow
            for #row_name #ty_generics #where_clause {}
    };

    // DataFramePayloadFields impl (#486): exposes FIELDS (all resolved column names,
    // deduplicated) and TAG for compile-time collision detection by outer enums.
    // FIELDS lists every single-column payload field name across all variants.
    // TAG is the inner enum's #[dataframe(tag = "...")] value (or "" if absent).
    let payload_fields_impl = {
        // Collect unique field names from all variant payload fields (single columns only).
        // We skip expanded (fixed/vec) and struct fields — only direct column contributions.
        let mut field_names: Vec<String> = Vec::new();
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        for vi in &variant_infos {
            for erf in &vi.fields {
                if let EnumResolvedField::Single(data) = erf {
                    let name = data.col_name.to_string();
                    if seen.insert(name.clone()) {
                        field_names.push(name);
                    }
                }
            }
        }
        let tag_str = attrs.tag.as_deref().unwrap_or("");
        quote! {
            impl #impl_generics ::miniextendr_api::markers::DataFramePayloadFields
                for #row_name #ty_generics #where_clause
            {
                const FIELDS: &'static [&'static str] = &[#(#field_names),*];
                const TAG: &'static str = #tag_str;
            }
        }
    };

    // region: unit-only enum factor impls
    // For a unit-only enum (all variants are unit), auto-emit:
    //   1. `impl UnitEnumFactor for Self`  — provides FACTOR_LEVELS and to_factor_index()
    //   2. `impl IntoR for Self`  — produces a single-element factor SEXP (cached levels)
    //   3. `impl IntoList for Self`  — delegates to vec![self].into_list()
    //
    // The `UnitEnumFactor` impl is consumed by the blanket
    // `impl<T: UnitEnumFactor> IntoR for FactorOptionVec<T>` in miniextendr-api,
    // which is what `into_data_frame` calls for `as_factor` companion struct columns.
    //
    // NOTE: `impl IntoR for Vec<Option<Self>>` violates orphan rules (Vec is foreign),
    // so we use the `FactorOptionVec<T>` wrapper type (local to miniextendr-api) instead.
    //
    // These impls allow `as_factor` and `as_list` to work on the inner type when it
    // appears as a field of an outer enum or struct DataFrameRow.
    let unit_only_factor_impls = {
        let all_unit = variant_infos
            .iter()
            .all(|vi| vi.shape == VariantShape::Unit);
        // For unit-only enums, auto-emit three impls:
        //   1. `impl UnitEnumFactor for Self`  — provides FACTOR_LEVELS and to_factor_index()
        //   2. `impl IntoR for Self`  — produces a single-element factor SEXP
        //   3. `impl IntoList for Self`  — delegates to vec![self].into_list()
        //
        // Non-generic enums: `IntoR` caches the levels SEXP via `OnceLock<SEXP>` (one-time
        // `R_PreserveObject`).
        //
        // Generic enums: Rust does not allow generic statics, so `IntoR` builds the levels
        // SEXP on each call using `build_levels_sexp` + manual `Rf_protect`/`Rf_unprotect`.
        // This is the same pattern used by `impl<T: UnitEnumFactor> IntoR for FactorOptionVec<T>`
        // in `miniextendr-api/src/factor.rs`.
        if all_unit {
            // Collect variant names and assign 1-based R factor indices (used by both branches).
            let variant_idents: Vec<&syn::Ident> =
                variant_infos.iter().map(|vi| &vi.name).collect();
            let variant_strs: Vec<String> =
                variant_infos.iter().map(|vi| vi.name.to_string()).collect();
            let variant_strs_lit: Vec<&str> = variant_strs.iter().map(|s| s.as_str()).collect();
            let indices: Vec<i32> = (1i32..=(variant_idents.len() as i32)).collect();

            if impl_generics.to_token_stream().is_empty() {
                // Non-generic: cache levels SEXP permanently via OnceLock (one R_PreserveObject).
                quote! {
                    // impl UnitEnumFactor for Self: provides FACTOR_LEVELS + to_factor_index().
                    // Used by `impl<T: UnitEnumFactor> IntoR for FactorOptionVec<T>` in miniextendr-api
                    // to build factor SEXPs from `Vec<Option<Self>>` companion columns.
                    impl ::miniextendr_api::factor::UnitEnumFactor for #row_name {
                        const FACTOR_LEVELS: &'static [&'static str] = &[#(#variant_strs_lit),*];
                        fn to_factor_index(self) -> i32 {
                            match self {
                                #(#row_name::#variant_idents => #indices,)*
                            }
                        }
                    }

                    // impl IntoR for Self: single-element factor SEXP (cached levels via OnceLock).
                    // Used when a unit-only enum value is returned directly from a #[miniextendr] fn.
                    impl ::miniextendr_api::IntoR for #row_name {
                        type Error = ::std::convert::Infallible;
                        fn try_into_sexp(self) -> ::std::result::Result<::miniextendr_api::ffi::SEXP, Self::Error> {
                            use ::std::sync::OnceLock;
                            const LEVELS: &[&str] = &[#(#variant_strs_lit),*];
                            static LEVELS_CACHE: OnceLock<::miniextendr_api::ffi::SEXP> =
                                OnceLock::new();
                            let levels = *LEVELS_CACHE.get_or_init(|| {
                                ::miniextendr_api::factor::build_levels_sexp_cached(LEVELS)
                            });
                            let idx: i32 = match self {
                                #(#row_name::#variant_idents => #indices,)*
                            };
                            ::std::result::Result::Ok(
                                ::miniextendr_api::factor::build_factor(&[idx], levels)
                            )
                        }
                    }

                    // impl IntoList for Self: for as_list path in outer DataFrameRow.
                    // Delegates to Vec<Self>: IntoList (blanket impl via IntoR for Self).
                    impl ::miniextendr_api::list::IntoList for #row_name {
                        fn into_list(self) -> ::miniextendr_api::list::List {
                            ::miniextendr_api::list::IntoList::into_list(::std::vec![self])
                        }
                    }
                }
            } else {
                // Generic: cannot use generic statics (Rust restriction).
                // Build the levels SEXP on each call and protect it across the build_factor
                // allocation — same pattern as `FactorOptionVec<T>: IntoR` in
                // `miniextendr-api/src/factor.rs`.
                quote! {
                    // impl UnitEnumFactor: associated const is allowed in generic impls.
                    impl #impl_generics ::miniextendr_api::factor::UnitEnumFactor
                        for #row_name #ty_generics #where_clause
                    {
                        const FACTOR_LEVELS: &'static [&'static str] = &[#(#variant_strs_lit),*];
                        fn to_factor_index(self) -> i32 {
                            match self {
                                #(#row_name::#variant_idents => #indices,)*
                            }
                        }
                    }

                    // impl IntoR: build levels SEXP on each call (no generic static allowed).
                    // Protect the levels STRSXP before build_factor allocates so GC cannot
                    // collect it mid-build (see CLAUDE.md "PROTECT discipline against R-devel GC").
                    impl #impl_generics ::miniextendr_api::IntoR
                        for #row_name #ty_generics #where_clause
                    {
                        type Error = ::std::convert::Infallible;
                        fn try_into_sexp(self) -> ::std::result::Result<::miniextendr_api::ffi::SEXP, Self::Error> {
                            const LEVELS: &[&str] = &[#(#variant_strs_lit),*];
                            let idx: i32 = match self {
                                #(#row_name::#variant_idents => #indices,)*
                            };
                            unsafe {
                                let levels = ::miniextendr_api::ffi::Rf_protect(
                                    ::miniextendr_api::factor::build_levels_sexp(LEVELS)
                                );
                                let result = ::miniextendr_api::factor::build_factor(&[idx], levels);
                                ::miniextendr_api::ffi::Rf_unprotect(1);
                                ::std::result::Result::Ok(result)
                            }
                        }
                    }

                    // impl IntoList: for as_list path in outer DataFrameRow.
                    impl #impl_generics ::miniextendr_api::list::IntoList
                        for #row_name #ty_generics #where_clause
                    {
                        fn into_list(self) -> ::miniextendr_api::list::List {
                            ::miniextendr_api::list::IntoList::into_list(::std::vec![self])
                        }
                    }
                }
            }
        } else {
            TokenStream::new()
        }
    };
    // endregion

    Ok(quote! {
        #dataframe_struct
        #into_dataframe_impl
        #from_vec_impl
        #df_methods
        #row_methods
        #split_method
        #marker_impl
        #payload_fields_impl
        #(#struct_assertions)*
        #(#payload_collision_assertions)*
        #unit_only_factor_impls
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

        // Determine if any field is AutoExpandVec or Struct (both require the dynamic pairs path
        // because column names are only known at runtime).
        let has_auto = vi.fields.iter().any(|f| {
            matches!(
                f,
                EnumResolvedField::AutoExpandVec(_) | EnumResolvedField::Struct(_)
            )
        });

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
                            // For needs_into_list fields, ty is already List (the stored type).
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
                        EnumResolvedField::Map(data) => {
                            let keys_buf = format_ident!("__s_{}_{}_keys", snake, data.base_name);
                            let vals_buf = format_ident!("__s_{}_{}_values", snake, data.base_name);
                            let key_ty = &data.key_ty;
                            let val_ty = &data.val_ty;
                            buf_decls.push(quote! {
                                let mut #keys_buf: Vec<Vec<#key_ty>> = Vec::new();
                                let mut #vals_buf: Vec<Vec<#val_ty>> = Vec::new();
                            });
                        }
                        // Struct field: buffer holds Vec<Inner> (no Option — split only sees
                        // rows of this variant, so every row has the field present).
                        EnumResolvedField::Struct(data) => {
                            let buf = format_ident!("__s_{}_{}", snake, data.base_name);
                            let inner_ty = &data.inner_ty;
                            buf_decls.push(quote! {
                                let mut #buf: Vec<#inner_ty> = Vec::new();
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
                            EnumResolvedField::Map(data) => {
                                let keys_buf =
                                    format_ident!("__s_{}_{}_keys", snake, data.base_name);
                                let vals_buf =
                                    format_ident!("__s_{}_{}_values", snake, data.base_name);
                                // unzip() guarantees pairwise alignment of keys and values.
                                vec![quote! {
                                    let (__mx_keys, __mx_vals) = #binding.into_iter().unzip::<_, _, Vec<_>, Vec<_>>();
                                    #keys_buf.push(__mx_keys);
                                    #vals_buf.push(__mx_vals);
                                }]
                            }
                            // Struct field: push binding directly (split only sees this variant's rows,
                            // so every row has the field — no Option needed).
                            EnumResolvedField::Struct(data) => {
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

                    // Find the first non-dynamic field for the length expression, or first dynamic.
                    // "Dynamic" = AutoExpandVec or Struct (both use dynamic pairs path).
                    let len_expr: TokenStream = {
                        let first_non_dynamic = vi.fields.iter().find(|f| {
                            !matches!(
                                f,
                                EnumResolvedField::AutoExpandVec(_) | EnumResolvedField::Struct(_)
                            )
                        });
                        if let Some(f) = first_non_dynamic {
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
                                EnumResolvedField::AutoExpandVec(_)
                                | EnumResolvedField::Struct(_) => unreachable!(),
                                EnumResolvedField::Map(data) => {
                                    let keys_buf =
                                        format_ident!("__s_{}_{}_keys", snake, data.base_name);
                                    quote! { #keys_buf.len() }
                                }
                            }
                        } else {
                            // All fields are dynamic — use the first dynamic buf length
                            if let Some(first) = vi.fields.first() {
                                match first {
                                    EnumResolvedField::AutoExpandVec(data) => {
                                        let buf = format_ident!("__s_{}_{}", snake, data.base_name);
                                        quote! { #buf.len() }
                                    }
                                    EnumResolvedField::Struct(data) => {
                                        let buf = format_ident!("__s_{}_{}", snake, data.base_name);
                                        quote! { #buf.len() }
                                    }
                                    _ => quote! { 0usize },
                                }
                            } else {
                                quote! { 0usize }
                            }
                        }
                    };

                    // Static pair pushes — wrap each `into_sexp()` in
                    // `__scope.protect_raw` to keep prior column SEXPs rooted
                    // across subsequent allocations
                    // (reviews/2026-05-07-gctorture-audit.md).
                    let static_pushes: Vec<TokenStream> = vi
                        .fields
                        .iter()
                        .flat_map(|erf| match erf {
                            EnumResolvedField::Single(data) => {
                                let buf = format_ident!("__s_{}_{}", snake, data.col_name);
                                let col_str = data.col_name.to_string();
                                let ty = &data.ty;
                                if data.needs_into_list {
                                    vec![quote! {
                                        {
                                            let __as_list_col: Vec<::miniextendr_api::list::List> =
                                                #buf.into_iter()
                                                    .map(|v: #ty| ::miniextendr_api::list::IntoList::into_list(v))
                                                    .collect();
                                            #pairs_var.push((
                                                #col_str.to_string(),
                                                __scope.protect_raw(::miniextendr_api::IntoR::into_sexp(__as_list_col)),
                                            ));
                                        }
                                    }]
                                } else if data.is_factor {
                                    // Factor column: convert Vec<T> → FactorOptionVec<T> (all present).
                                    vec![quote! {
                                        #pairs_var.push((
                                            #col_str.to_string(),
                                            __scope.protect_raw(::miniextendr_api::IntoR::into_sexp(
                                                ::miniextendr_api::factor::FactorOptionVec::<#ty>::from(
                                                    #buf.into_iter().map(|v| ::std::option::Option::Some(v)).collect::<::std::vec::Vec<_>>()
                                                )
                                            )),
                                        ));
                                    }]
                                } else {
                                    vec![quote! {
                                        #pairs_var.push((
                                            #col_str.to_string(),
                                            __scope.protect_raw(::miniextendr_api::IntoR::into_sexp(#buf)),
                                        ));
                                    }]
                                }
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
                                            __scope.protect_raw(::miniextendr_api::IntoR::into_sexp(#buf)),
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
                                            __scope.protect_raw(::miniextendr_api::IntoR::into_sexp(#buf)),
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
                                                __scope.protect_raw(::miniextendr_api::IntoR::into_sexp(__acol)),
                                            ));
                                        }
                                    }
                                }]
                            }
                            EnumResolvedField::Map(data) => {
                                let keys_buf =
                                    format_ident!("__s_{}_{}_keys", snake, data.base_name);
                                let vals_buf =
                                    format_ident!("__s_{}_{}_values", snake, data.base_name);
                                let keys_str = format!("{}_keys", data.base_name);
                                let vals_str = format!("{}_values", data.base_name);
                                vec![
                                    quote! {
                                        #pairs_var.push((
                                            #keys_str.to_string(),
                                            __scope.protect_raw(::miniextendr_api::IntoR::into_sexp(#keys_buf)),
                                        ));
                                    },
                                    quote! {
                                        #pairs_var.push((
                                            #vals_str.to_string(),
                                            __scope.protect_raw(::miniextendr_api::IntoR::into_sexp(#vals_buf)),
                                        ));
                                    },
                                ]
                            }
                            // Struct field: call Inner::to_dataframe(buf), extract columns,
                            // push with prefixed names. In the split path, all rows belong to
                            // this variant so no scatter is needed.
                            EnumResolvedField::Struct(data) => {
                                let buf = format_ident!("__s_{}_{}", snake, data.base_name);
                                let base_str = &data.base_name;
                                let inner_ty = &data.inner_ty;
                                vec![quote! {
                                    {
                                        let __inner_df = <#inner_ty>::to_dataframe(#buf);
                                        let __inner_cols = ::miniextendr_api::convert::IntoDataFrame::into_named_columns(__inner_df);
                                        for (__inner_col_name, __inner_col_sexp) in __inner_cols {
                                            let __prefixed = format!("{}_{}", #base_str, __inner_col_name);
                                            #pairs_var.push((
                                                __prefixed,
                                                __scope.protect_raw(__inner_col_sexp),
                                            ));
                                        }
                                    }
                                }]
                            }
                        })
                        .collect();

                    df_constructions.push(quote! {
                        let #n_var = #len_expr;
                        // SAFETY: split-method runs on the R main thread; scope
                        // unprotects after each variant data.frame is built.
                        let #df_var = unsafe {
                            let __scope = ::miniextendr_api::gc_protect::ProtectScope::new();
                            let mut #pairs_var: Vec<(String, ::miniextendr_api::ffi::SEXP)> = Vec::new();
                            #(#static_pushes)*
                            ::miniextendr_api::list::List::from_raw_pairs(#pairs_var)
                                .set_class_str(&["data.frame"])
                                .set_row_names_int(#n_var)
                        };
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
                            // AutoExpandVec and Struct both trigger has_auto = true, so these
                            // branches are unreachable in the non-auto static path.
                            EnumResolvedField::AutoExpandVec(_) | EnumResolvedField::Struct(_) => {
                                unreachable!()
                            }
                            EnumResolvedField::Map(data) => {
                                let keys_buf =
                                    format_ident!("__s_{}_{}_keys", snake, data.base_name);
                                quote! { #keys_buf.len() }
                            }
                        }
                    } else {
                        // No fields (unexpected for Named/Tuple, but handle it)
                        quote! { 0usize }
                    };

                    // Collect pairs — each `into_sexp()` is rooted via
                    // `__scope.protect_raw` so prior columns survive the
                    // next column's allocation
                    // (reviews/2026-05-07-gctorture-audit.md).
                    let pairs: Vec<TokenStream> = vi
                        .fields
                        .iter()
                        .flat_map(|erf| match erf {
                            EnumResolvedField::Single(data) => {
                                let buf = format_ident!("__s_{}_{}", snake, data.col_name);
                                let col_str = data.col_name.to_string();
                                let ty = &data.ty;
                                if data.needs_into_list {
                                    // Convert Vec<T> → Vec<List> → SEXP at split time.
                                    vec![quote! {
                                        (#col_str, {
                                            let __as_list_col: Vec<::miniextendr_api::list::List> =
                                                #buf.into_iter()
                                                    .map(|v: #ty| ::miniextendr_api::list::IntoList::into_list(v))
                                                    .collect();
                                            __scope.protect_raw(::miniextendr_api::IntoR::into_sexp(__as_list_col))
                                        })
                                    }]
                                } else if data.is_factor {
                                    // Factor: convert Vec<T> → FactorOptionVec<T> (all present).
                                    vec![quote! {
                                        (#col_str, __scope.protect_raw(::miniextendr_api::IntoR::into_sexp(
                                            ::miniextendr_api::factor::FactorOptionVec::<#ty>::from(
                                                #buf.into_iter().map(|v| ::std::option::Option::Some(v)).collect::<::std::vec::Vec<_>>()
                                            )
                                        )))
                                    }]
                                } else {
                                    vec![quote! {
                                        (#col_str, __scope.protect_raw(::miniextendr_api::IntoR::into_sexp(#buf)))
                                    }]
                                }
                            }
                            EnumResolvedField::ExpandedFixed(data) => (1..=data.len)
                                .map(|i| {
                                    let buf =
                                        format_ident!("__s_{}_{}_{}", snake, data.base_name, i);
                                    let col_str = format!("{}_{}", data.base_name, i);
                                    quote! {
                                        (#col_str, __scope.protect_raw(::miniextendr_api::IntoR::into_sexp(#buf)))
                                    }
                                })
                                .collect(),
                            EnumResolvedField::ExpandedVec(data) => (1..=data.width)
                                .map(|i| {
                                    let buf =
                                        format_ident!("__s_{}_{}_{}", snake, data.base_name, i);
                                    let col_str = format!("{}_{}", data.base_name, i);
                                    quote! {
                                        (#col_str, __scope.protect_raw(::miniextendr_api::IntoR::into_sexp(#buf)))
                                    }
                                })
                                .collect(),
                            // AutoExpandVec and Struct both trigger has_auto = true.
                            EnumResolvedField::AutoExpandVec(_) | EnumResolvedField::Struct(_) => unreachable!(),
                            EnumResolvedField::Map(data) => {
                                let keys_buf =
                                    format_ident!("__s_{}_{}_keys", snake, data.base_name);
                                let vals_buf =
                                    format_ident!("__s_{}_{}_values", snake, data.base_name);
                                let keys_str = format!("{}_keys", data.base_name);
                                let vals_str = format!("{}_values", data.base_name);
                                vec![
                                    quote! {
                                        (#keys_str, __scope.protect_raw(::miniextendr_api::IntoR::into_sexp(#keys_buf)))
                                    },
                                    quote! {
                                        (#vals_str, __scope.protect_raw(::miniextendr_api::IntoR::into_sexp(#vals_buf)))
                                    },
                                ]
                            }
                        })
                        .collect();

                    df_constructions.push(quote! {
                        let #n_var = #len_expr;
                        // SAFETY: see has_auto branch.
                        let #df_var = unsafe {
                            let __scope = ::miniextendr_api::gc_protect::ProtectScope::new();
                            ::miniextendr_api::list::List::from_raw_pairs(vec![
                                #(#pairs),*
                            ])
                            .set_class_str(&["data.frame"])
                            .set_row_names_int(#n_var)
                        };
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
        // Multiple variants: return named list of data.frames.
        // Each per-variant data.frame's `into_sexp()` is rooted via
        // `__outer_scope.protect_raw` so prior variant data.frames survive
        // the next variant's allocation
        // (reviews/2026-05-07-gctorture-audit.md).
        let outer_pairs: Vec<TokenStream> = snake_names
            .iter()
            .zip(df_var_names.iter())
            .map(|(name, var)| {
                quote! { (#name, __outer_scope.protect_raw(::miniextendr_api::IntoR::into_sexp(#var))) }
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
            // SAFETY: split-method runs on the R main thread.
            unsafe {
                let __outer_scope = ::miniextendr_api::gc_protect::ProtectScope::new();
                ::miniextendr_api::list::List::from_raw_pairs(vec![
                    #(#outer_pairs),*
                ])
            }
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
