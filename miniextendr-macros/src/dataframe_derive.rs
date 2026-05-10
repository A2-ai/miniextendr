//! Derive macros for bidirectional row ↔ dataframe conversions.
//!
//! Supports both structs (direct field mapping) and enums (field-name union
//! across variants with `Option<T>` fill for missing fields).

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields};

// region: Attribute parsing

/// Parsed container-level `#[dataframe(...)]` attributes.
pub(super) struct DataFrameAttrs {
    /// Custom companion type name (default: `{TypeName}DataFrame`).
    pub(super) name: Option<syn::Ident>,
    /// Enum alignment mode — implicit for enums, accepted but not required.
    pub(super) align: bool,
    /// Tag column name for variant discriminator (also supported on structs).
    pub(super) tag: Option<String>,
    /// Conflict resolution mode for type collisions across enum variants.
    /// Currently only "string" is supported: convert conflicting fields via `ToString`.
    pub(super) conflicts: Option<String>,
}

/// Parse container-level `#[dataframe(...)]` attributes from the derive input.
///
/// Supported keys:
/// - `name = "CustomName"` -- custom companion type name (default: `{TypeName}DataFrame`)
/// - `align` -- enum alignment mode (field-name union across variants)
/// - `tag = "col_name"` -- add a variant discriminator column (works on both structs and enums)
/// - `conflicts = "string"` -- coerce type-conflicting columns to `String` via `ToString`
///
/// Returns `Err` for unknown keys or non-string-literal values.
fn parse_dataframe_attrs(input: &DeriveInput) -> syn::Result<DataFrameAttrs> {
    let mut attrs = DataFrameAttrs {
        name: None,
        align: false,
        tag: None,
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
                other => {
                    return Err(syn::Error::new_spanned(
                        other,
                        "unknown dataframe attribute; expected `name`, `align`, `tag`, or `conflicts`",
                    ));
                }
            }
        }
    }

    Ok(attrs)
}
// endregion

// region: Field-level attribute parsing

/// Parsed field-level `#[dataframe(...)]` attributes.
///
/// These attributes control how individual struct/enum fields map to DataFrame columns.
/// Mutually exclusive combinations (`as_list` + `expand`, `as_list` + `width`) are
/// rejected during parsing.
#[derive(Default)]
pub(super) struct FieldAttrs {
    /// `#[dataframe(skip)]` -- omit this field from the DataFrame entirely.
    pub(super) skip: bool,
    /// `#[dataframe(rename = "col")]` -- use a custom column name instead of the field name.
    pub(super) rename: Option<String>,
    /// `#[dataframe(as_list)]` -- keep a collection field as a single R list column
    /// (suppresses automatic expansion into suffixed columns).
    as_list: bool,
    /// `#[dataframe(expand)]` or `#[dataframe(unnest)]` -- explicitly expand a
    /// collection field into multiple suffixed columns.
    expand: bool,
    /// `#[dataframe(width = N)]` -- pin the expansion width for `Vec<T>`, `Box<[T]>`,
    /// or `&[T]` fields. Rows shorter than `N` get `None` for missing positions.
    pub(super) width: Option<usize>,
}

/// Parse field-level `#[dataframe(...)]` attributes from a `syn::Field`.
///
/// Recognizes: `skip`, `rename`, `as_list`, `expand` (alias `unnest`), and `width`.
/// Validates mutual exclusivity of conflicting options (`as_list` vs `expand`/`width`).
/// Returns `Err` for unknown keys, invalid width values, or conflicting options.
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
// endregion

// region: Type classification

/// Classification of a field type for DataFrame column expansion.
///
/// Used to decide whether a field maps to a single column or should be
/// expanded into multiple suffixed columns (e.g., `coords_1`, `coords_2`).
pub(super) enum FieldTypeKind<'a> {
    /// Single column (most types). No expansion.
    Scalar,
    /// `[T; N]` -- fixed-size array, expands to `N` columns at compile time.
    /// Contains the element type and array length.
    FixedArray(&'a syn::Type, usize),
    /// `Vec<T>` -- variable length, needs `width` attribute or `expand` for expansion.
    /// Contains the element type.
    VariableVec(&'a syn::Type),
    /// `Box<[T]>` -- owned slice, treated like `Vec<T>` for expansion purposes.
    /// Contains the element type.
    BoxedSlice(&'a syn::Type),
    /// `&[T]` -- borrowed slice, treated like `Vec<T>` for expansion purposes.
    /// Contains the element type.
    BorrowedSlice(&'a syn::Type),
    /// `HashMap<K, V>` or `BTreeMap<K, V>` -- expands to two parallel list-columns:
    /// `<field>_keys` and `<field>_values`. Key order follows the map's own iteration
    /// order: `BTreeMap` yields sorted keys, `HashMap` yields non-deterministic order.
    Map {
        key_ty: &'a syn::Type,
        val_ty: &'a syn::Type,
    },
    /// A struct-typed field whose inner type implements `DataFrameRow`.
    ///
    /// Flattened into `<field>_<inner_col>` prefixed columns by default.
    /// A compile-time assertion against `::miniextendr_api::markers::DataFrameRow`
    /// is emitted so rustc gives a clear error when the inner type is missing the
    /// derive.
    ///
    /// Suppressed by `#[dataframe(as_list)]` — with as_list the field becomes
    /// a `Scalar` and uses the ordinary single-column codegen path.
    Struct {
        /// The full field type (used for the compile-time DataFrameRow assertion).
        inner_ty: &'a syn::Type,
    },
}

/// Classify a field type for DataFrame column expansion.
///
/// Inspects the type AST to detect:
/// - `[T; N]` or `&[T; N]` -> `FixedArray`
/// - `&[T]` -> `BorrowedSlice`
/// - `Vec<T>` -> `VariableVec`
/// - `Box<[T]>` -> `BoxedSlice`
/// - Everything else -> `Scalar`
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

    // Check for &[T] and &[T; N]
    if let syn::Type::Reference(ref_ty) = ty {
        // &[T] → BorrowedSlice
        if let syn::Type::Slice(slice) = &*ref_ty.elem {
            return FieldTypeKind::BorrowedSlice(&slice.elem);
        }
        // &[T; N] → FixedArray (same as owned)
        if let syn::Type::Array(arr) = &*ref_ty.elem
            && let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(lit_int),
                ..
            }) = &arr.len
            && let Ok(n) = lit_int.base10_parse::<usize>()
        {
            return FieldTypeKind::FixedArray(&arr.elem, n);
        }
    }

    if let syn::Type::Path(type_path) = ty
        && let Some(seg) = type_path.path.segments.last()
        && let syn::PathArguments::AngleBracketed(args) = &seg.arguments
        && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
    {
        // Check for Vec<T>
        if seg.ident == "Vec" {
            return FieldTypeKind::VariableVec(inner);
        }

        // Check for Box<[T]>
        if seg.ident == "Box"
            && let syn::Type::Slice(slice) = inner
        {
            return FieldTypeKind::BoxedSlice(&slice.elem);
        }

        // Check for HashMap<K, V> and BTreeMap<K, V>
        if (seg.ident == "HashMap" || seg.ident == "BTreeMap")
            && let Some(syn::GenericArgument::Type(val_ty)) = args.args.iter().nth(1)
        {
            return FieldTypeKind::Map {
                key_ty: inner,
                val_ty,
            };
        }
    }

    // Any remaining single-segment bare path type that is NOT a known scalar is
    // treated as a user-defined struct whose `DataFrameRow` derive should be called.
    // The compile-time assertion `_assert_inner_is_dataframe_row::<Inner>()` in the
    // generated code surfaces a clear error if the inner type doesn't have the derive.
    //
    // Known scalars (i32, f64, String, bool, …) are kept as `Scalar` so that existing
    // enum variants with primitive fields (e.g. `Click { id: i64, x: f64 }`) are not
    // misclassified as struct fields.  Multi-segment paths (std::ffi::CString) and
    // path types with a qualifying `self::`/`super::` prefix fall through to `Scalar`
    // as well — the user can opt into list-column treatment with `#[dataframe(as_list)]`.
    if let syn::Type::Path(type_path) = ty {
        let segs = &type_path.path.segments;
        // Single-segment, no leading colon (i.e. a bare ident like `Point`)
        if segs.len() == 1 && type_path.qself.is_none() && type_path.path.leading_colon.is_none() {
            let seg = segs.last().unwrap();
            if matches!(seg.arguments, syn::PathArguments::None) {
                let name = seg.ident.to_string();
                // Known scalar type names — keep as Scalar so they do not trigger the
                // struct-flatten path and the DataFrameRow compile-time assertion.
                const KNOWN_SCALARS: &[&str] = &[
                    "bool", "char", "str", "f32", "f64", "i8", "i16", "i32", "i64", "i128",
                    "isize", "u8", "u16", "u32", "u64", "u128", "usize", "String",
                ];
                if !KNOWN_SCALARS.contains(&name.as_str()) {
                    return FieldTypeKind::Struct { inner_ty: ty };
                }
            }
        }
    }

    FieldTypeKind::Scalar
}
// endregion

// region: Resolved field model (struct path)

/// A resolved struct field ready for codegen -- determines how this field maps
/// to DataFrame companion struct columns.
///
/// Each variant represents a different expansion strategy:
/// - `Single`: one field -> one `Vec<T>` column
/// - `ExpandedFixed`: `[T; N]` -> N columns (`name_1..name_N`) at compile time
/// - `ExpandedVec`: `Vec<T>` + `width = N` -> N `Vec<Option<T>>` columns
/// - `AutoExpandVec`: `Vec<T>` + `expand` -> dynamic column count at runtime
enum ResolvedField {
    /// Single column: `name → Vec<ty>`.
    Single(Box<SingleFieldData>),
    /// Expanded fixed array: `name: [T; N]` → `name_1..name_N`.
    ExpandedFixed(Box<ExpandedFixedData>),
    /// Expanded variable vec with pinned width: `name: Vec<T>` + `width = N`.
    ExpandedVec(Box<ExpandedVecData>),
    /// Auto-expanded Vec<T>/Box<[T]>: column count determined at runtime from max row length.
    AutoExpandVec(Box<AutoExpandVecData>),
}

/// Data for [`ResolvedField::Single`].
struct SingleFieldData {
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
}

/// Data for [`ResolvedField::ExpandedFixed`].
struct ExpandedFixedData {
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
}

/// Data for [`ResolvedField::ExpandedVec`].
struct ExpandedVecData {
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
}

/// Data for [`ResolvedField::AutoExpandVec`].
struct AutoExpandVecData {
    /// Rust field name (for row access).
    rust_name: syn::Ident,
    /// Companion struct field name (ident).
    col_name: syn::Ident,
    /// Column name base string (for suffixed column names).
    col_name_str: String,
    /// Element type T.
    elem_ty: syn::Type,
    /// Container type for companion struct (Vec<T> or Box<[T]>).
    container_ty: syn::Type,
    /// Index in tuple struct.
    tuple_index: Option<syn::Index>,
}

/// Resolve a struct field into a [`ResolvedField`], applying field attributes.
///
/// Combines the field's `#[dataframe(...)]` attributes with its type classification
/// to determine the codegen strategy:
/// - `skip` -> returns `None`
/// - `as_list` -> `Single` (suppresses expansion)
/// - `FixedArray` -> `ExpandedFixed` (compile-time expansion to N columns)
/// - `VariableVec`/`BoxedSlice`/`BorrowedSlice` + `width` -> `ExpandedVec`
/// - `VariableVec`/`BoxedSlice`/`BorrowedSlice` + `expand` -> `AutoExpandVec`
/// - Everything else -> `Single`
///
/// Returns `Err` if `width` or `expand` is used on an incompatible type.
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
        return Ok(Some(ResolvedField::Single(Box::new(SingleFieldData {
            rust_name,
            col_name,
            col_name_str,
            ty: ty.clone(),
            tuple_index,
        }))));
    }

    match kind {
        FieldTypeKind::FixedArray(elem_ty, len) => Ok(Some(ResolvedField::ExpandedFixed(
            Box::new(ExpandedFixedData {
                rust_name,
                base_name: col_name_str,
                elem_ty: elem_ty.clone(),
                len,
                tuple_index,
            }),
        ))),
        FieldTypeKind::VariableVec(elem_ty)
        | FieldTypeKind::BoxedSlice(elem_ty)
        | FieldTypeKind::BorrowedSlice(elem_ty) => {
            if let Some(width) = field_attrs.width {
                Ok(Some(ResolvedField::ExpandedVec(Box::new(
                    ExpandedVecData {
                        rust_name,
                        base_name: col_name_str,
                        elem_ty: elem_ty.clone(),
                        width,
                        tuple_index,
                    },
                ))))
            } else if field_attrs.expand {
                Ok(Some(ResolvedField::AutoExpandVec(Box::new(
                    AutoExpandVecData {
                        rust_name,
                        col_name,
                        col_name_str,
                        elem_ty: elem_ty.clone(),
                        container_ty: ty.clone(),
                        tuple_index,
                    },
                ))))
            } else {
                // No expansion — keep as opaque single column
                Ok(Some(ResolvedField::Single(Box::new(SingleFieldData {
                    rust_name,
                    col_name,
                    col_name_str,
                    ty: ty.clone(),
                    tuple_index,
                }))))
            }
        }
        // Struct-in-struct flattening is out of scope — treat as a single opaque column.
        // See issue #459 follow-up note (Q4).
        FieldTypeKind::Scalar | FieldTypeKind::Map { .. } | FieldTypeKind::Struct { .. } => {
            if field_attrs.width.is_some() {
                return Err(syn::Error::new_spanned(
                    ty,
                    "`width` is only valid on `Vec<T>`, `Box<[T]>`, or `&[T]` fields",
                ));
            }
            if field_attrs.expand {
                return Err(syn::Error::new_spanned(
                    ty,
                    "`expand`/`unnest` is only valid on `[T; N]`, `Vec<T>`, `Box<[T]>`, or `&[T]` fields",
                ));
            }
            Ok(Some(ResolvedField::Single(Box::new(SingleFieldData {
                rust_name,
                col_name,
                col_name_str,
                ty: ty.clone(),
                tuple_index,
            }))))
        }
    }
}
// endregion

// region: Top-level dispatch

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
///
/// Both struct and enum companion types get `from_rows()` (sequential) and
/// `from_rows_par()` (parallel, `#[cfg(feature = "rayon")]`) methods automatically.
pub fn derive_dataframe_row(input: DeriveInput) -> syn::Result<TokenStream> {
    let row_name = &input.ident;

    // Allow lifetime parameters (needed for &[T] borrowed slice fields).
    // Reject type and const parameters — these can't be propagated correctly.
    let has_type_params = input.generics.type_params().next().is_some();
    let has_const_params = input.generics.const_params().next().is_some();
    if has_type_params || has_const_params {
        return Err(syn::Error::new_spanned(
            &input.generics,
            "DataFrameRow does not support type or const generic parameters",
        ));
    }

    // Parse attributes
    let attrs = parse_dataframe_attrs(&input)?;

    let df_name = attrs
        .name
        .clone()
        .unwrap_or_else(|| format_ident!("{}DataFrame", row_name));

    let base = match &input.data {
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
    }?;

    // Generate IntoR for the companion DataFrame type so it can be returned
    // directly from #[miniextendr] functions. This ensures both the standalone
    // #[derive(DataFrameRow)] path and the #[miniextendr(dataframe)] path
    // produce identical output.
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    Ok(quote::quote! {
        #base

        impl #impl_generics ::miniextendr_api::into_r::IntoR for #df_name #ty_generics #where_clause {
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
}
// endregion

// region: Struct path (existing logic, extracted)

/// Generate `DataFrameRow` expansion for struct types.
///
/// Produces:
/// - A companion struct `{Name}DataFrame` with `Vec<T>` columns
/// - `impl IntoDataFrame for {Name}DataFrame`
/// - `impl From<Vec<{Name}>> for {Name}DataFrame`
/// - `impl IntoIterator` (for named structs without expansion)
/// - Associated methods: `to_dataframe`, `from_dataframe`, `from_rows`, `from_rows_par`
/// - A compile-time `IntoList` assertion (for non-expanded named structs)
///
/// Handles fixed-array expansion (`[T; N]`), pinned-width Vec expansion
/// (`Vec<T>` + `width`), and auto-expand Vec (`Vec<T>` + `expand`).
fn derive_struct_dataframe(
    row_name: &syn::Ident,
    input: &DeriveInput,
    data: &syn::DataStruct,
    df_name: &syn::Ident,
    attrs: &DataFrameAttrs,
) -> syn::Result<TokenStream> {
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
        .any(|rf| !matches!(rf, ResolvedField::Single(..)));
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

    // region: Build flat column lists from resolved fields
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
            ResolvedField::Single(data) => {
                flat_cols.push(FlatCol {
                    df_field: data.col_name.clone(),
                    col_name_str: data.col_name_str.clone(),
                    vec_elem_ty: data.ty.clone(),
                });
            }
            ResolvedField::ExpandedFixed(data) => {
                for i in 1..=data.len {
                    let name = format!("{}_{}", data.base_name, i);
                    flat_cols.push(FlatCol {
                        df_field: format_ident!("{}_{}", data.base_name, i),
                        col_name_str: name,
                        vec_elem_ty: data.elem_ty.clone(),
                    });
                }
            }
            ResolvedField::ExpandedVec(data) => {
                for i in 1..=data.width {
                    let name = format!("{}_{}", data.base_name, i);
                    let elem_ty = &data.elem_ty;
                    let opt_ty: syn::Type = syn::parse_quote!(Option<#elem_ty>);
                    flat_cols.push(FlatCol {
                        df_field: format_ident!("{}_{}", data.base_name, i),
                        col_name_str: name,
                        vec_elem_ty: opt_ty,
                    });
                }
            }
            // AutoExpandVec does not produce FlatCols — handled separately.
            ResolvedField::AutoExpandVec(..) => {}
        }
    }
    // endregion

    // region: Collect auto-expand fields
    struct AutoExpandCol {
        /// Companion struct field name.
        df_field: syn::Ident,
        /// Container type (Vec<T> or Box<[T]>).
        container_ty: syn::Type,
    }

    let auto_expand_cols: Vec<AutoExpandCol> = resolved
        .iter()
        .filter_map(|rf| {
            if let ResolvedField::AutoExpandVec(data) = rf {
                Some(AutoExpandCol {
                    df_field: format_ident!("{}", data.col_name_str),
                    container_ty: data.container_ty.clone(),
                })
            } else {
                None
            }
        })
        .collect();
    let has_auto_expand = !auto_expand_cols.is_empty();
    // endregion

    // region: Companion struct
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
        let cty = &ac.container_ty;
        df_fields_tokens.push(quote! { pub #name: Vec<#cty> });
    }

    let len_field_decl = if flat_cols.is_empty() && auto_expand_cols.is_empty() && !has_tag {
        quote! { pub _len: usize, }
    } else {
        TokenStream::new()
    };

    let dataframe_struct = quote! {
        #[derive(Debug, Clone)]
        pub struct #df_name #impl_generics #where_clause {
            #tag_field_decl
            #len_field_decl
            #(#df_fields_tokens),*
        }
    };
    // endregion

    // region: IntoDataFrame
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

    let df_pairs: Vec<TokenStream> = flat_cols
        .iter()
        .map(|fc| {
            let name = &fc.df_field;
            let name_str = &fc.col_name_str;
            quote! { (#name_str, __scope.protect_raw(::miniextendr_api::IntoR::into_sexp(self.#name))) }
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
                __df_pairs.push((#tag_name.to_string(), __scope.protect_raw(::miniextendr_api::IntoR::into_sexp(self._tag))));
            }
        } else {
            TokenStream::new()
        };

        let pair_pushes: Vec<TokenStream> = resolved
            .iter()
            .map(|rf| match rf {
                ResolvedField::Single(data) => {
                    let col_name = &data.col_name;
                    let col_name_str = &data.col_name_str;
                    quote! {
                        __df_pairs.push((
                            #col_name_str.to_string(),
                            __scope.protect_raw(::miniextendr_api::IntoR::into_sexp(self.#col_name)),
                        ));
                    }
                }
                ResolvedField::ExpandedFixed(data) => {
                    let pushes: Vec<TokenStream> = (1..=data.len)
                        .map(|i| {
                            let name = format!("{}_{}", data.base_name, i);
                            let ident = format_ident!("{}_{}", data.base_name, i);
                            quote! {
                                __df_pairs.push((
                                    #name.to_string(),
                                    __scope.protect_raw(::miniextendr_api::IntoR::into_sexp(self.#ident)),
                                ));
                            }
                        })
                        .collect();
                    quote! { #(#pushes)* }
                }
                ResolvedField::ExpandedVec(data) => {
                    let pushes: Vec<TokenStream> = (1..=data.width)
                        .map(|i| {
                            let name = format!("{}_{}", data.base_name, i);
                            let ident = format_ident!("{}_{}", data.base_name, i);
                            quote! {
                                __df_pairs.push((
                                    #name.to_string(),
                                    __scope.protect_raw(::miniextendr_api::IntoR::into_sexp(self.#ident)),
                                ));
                            }
                        })
                        .collect();
                    quote! { #(#pushes)* }
                }
                ResolvedField::AutoExpandVec(data) => {
                    let col_name = &data.col_name;
                    let col_name_str = &data.col_name_str;
                    let elem_ty = &data.elem_ty;
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
                                    __scope.protect_raw(::miniextendr_api::IntoR::into_sexp(__col)),
                                ));
                            }
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
                        #(#pair_pushes)*
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
                        ::miniextendr_api::list::List::from_raw_pairs(vec![
                            #tag_pair
                            #(#df_pairs),*
                        ])
                        .set_class_str(&["data.frame"])
                        .set_row_names_int(_n_rows)
                    }
                }
            }
        }
    };
    // endregion

    // region: From<Vec<RowType>>
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
        let cty = &ac.container_ty;
        col_vec_inits.push(quote! { let mut #name: Vec<#cty> = Vec::with_capacity(len); });
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
            ResolvedField::Single(data) => {
                let access = if let Some(idx) = &data.tuple_index {
                    quote! { row.#idx }
                } else {
                    let rust_name = &data.rust_name;
                    quote! { row.#rust_name }
                };
                let col_name = &data.col_name;
                quote! { #col_name.push(#access); }
            }
            ResolvedField::ExpandedFixed(data) => {
                let access = if let Some(idx) = &data.tuple_index {
                    quote! { row.#idx }
                } else {
                    let rust_name = &data.rust_name;
                    quote! { row.#rust_name }
                };
                let bind = format_ident!("__arr_{}", data.rust_name);
                let pushes: Vec<TokenStream> = (0..data.len)
                    .map(|i| {
                        let col_ident = format_ident!("{}_{}", data.base_name, i + 1);
                        let idx = syn::Index::from(i);
                        quote! { #col_ident.push(#bind[#idx]); }
                    })
                    .collect();
                quote! {
                    let #bind = #access;
                    #(#pushes)*
                }
            }
            ResolvedField::ExpandedVec(data) => {
                let access = if let Some(idx) = &data.tuple_index {
                    quote! { row.#idx }
                } else {
                    let rust_name = &data.rust_name;
                    quote! { row.#rust_name }
                };
                let bind = format_ident!("__vec_{}", data.rust_name);
                let pushes: Vec<TokenStream> = (0..data.width)
                    .map(|i| {
                        let col_ident = format_ident!("{}_{}", data.base_name, i + 1);
                        quote! { #col_ident.push(#bind.get(#i).cloned()); }
                    })
                    .collect();
                quote! {
                    let #bind = #access;
                    #(#pushes)*
                }
            }
            ResolvedField::AutoExpandVec(data) => {
                let access = if let Some(idx) = &data.tuple_index {
                    quote! { row.#idx }
                } else {
                    let rust_name = &data.rust_name;
                    quote! { row.#rust_name }
                };
                let col_name = &data.col_name;
                quote! { #col_name.push(#access); }
            }
        })
        .collect();

    let tag_struct_field = if has_tag {
        quote! { _tag, }
    } else {
        TokenStream::new()
    };

    let len_struct_field = if flat_cols.is_empty() && auto_expand_cols.is_empty() && !has_tag {
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

    // For skipped fields in destructure: bind to `_`
    let skip_bindings: Vec<TokenStream> = skipped_fields
        .iter()
        .map(|name| quote! { let _ = row.#name; })
        .collect();

    let from_vec_impl = quote! {
        impl #impl_generics From<Vec<#row_name #ty_generics>> for #df_name #ty_generics #where_clause {
            fn from(rows: Vec<#row_name #ty_generics>) -> Self {
                let len = rows.len();
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
    // endregion

    // region: Generate from_rows_par (parallel scatter-write via ColumnWriter)
    let from_rows_par_method = if !flat_cols.is_empty() || !auto_expand_cols.is_empty() || has_tag {
        // Column declarations: vec![default; len]
        let mut par_col_decls = Vec::new();
        if has_tag {
            par_col_decls.push(quote! {
                let mut _tag: Vec<String> = vec![String::new(); len];
            });
        }
        for fc in &flat_cols {
            let name = &fc.df_field;
            let ty = &fc.vec_elem_ty;
            par_col_decls.push(quote! {
                let mut #name: Vec<#ty> = vec![<#ty as ::core::default::Default>::default(); len];
            });
        }
        for ac in &auto_expand_cols {
            let name = &ac.df_field;
            let cty = &ac.container_ty;
            par_col_decls.push(quote! {
                let mut #name: Vec<#cty> = vec![<#cty as ::core::default::Default>::default(); len];
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
        for fc in &flat_cols {
            let name = &fc.df_field;
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

        // Write calls per resolved field
        let tag_write = if has_tag {
            quote! { __w_tag.write(__i, #row_name_str.to_string()); }
        } else {
            TokenStream::new()
        };

        let par_write_calls: Vec<TokenStream> = resolved
            .iter()
            .map(|rf| match rf {
                ResolvedField::Single(data) => {
                    let access = if let Some(idx) = &data.tuple_index {
                        quote! { __row.#idx }
                    } else {
                        let rust_name = &data.rust_name;
                        quote! { __row.#rust_name }
                    };
                    let w_name = format_ident!("__w_{}", data.col_name);
                    quote! { #w_name.write(__i, #access); }
                }
                ResolvedField::ExpandedFixed(data) => {
                    let access = if let Some(idx) = &data.tuple_index {
                        quote! { __row.#idx }
                    } else {
                        let rust_name = &data.rust_name;
                        quote! { __row.#rust_name }
                    };
                    let bind = format_ident!("__arr_{}", data.rust_name);
                    let writes: Vec<TokenStream> = (0..data.len)
                        .map(|i| {
                            let w_name = format_ident!("__w_{}_{}", data.base_name, i + 1);
                            let idx = syn::Index::from(i);
                            quote! { #w_name.write(__i, #bind[#idx]); }
                        })
                        .collect();
                    quote! {
                        let #bind = #access;
                        #(#writes)*
                    }
                }
                ResolvedField::ExpandedVec(data) => {
                    let access = if let Some(idx) = &data.tuple_index {
                        quote! { __row.#idx }
                    } else {
                        let rust_name = &data.rust_name;
                        quote! { __row.#rust_name }
                    };
                    let bind = format_ident!("__vec_{}", data.rust_name);
                    let writes: Vec<TokenStream> = (0..data.width)
                        .map(|i| {
                            let w_name = format_ident!("__w_{}_{}", data.base_name, i + 1);
                            quote! { #w_name.write(__i, #bind.get(#i).cloned()); }
                        })
                        .collect();
                    quote! {
                        let #bind = #access;
                        #(#writes)*
                    }
                }
                ResolvedField::AutoExpandVec(data) => {
                    let access = if let Some(idx) = &data.tuple_index {
                        quote! { __row.#idx }
                    } else {
                        let rust_name = &data.rust_name;
                        quote! { __row.#rust_name }
                    };
                    let w_name = format_ident!("__w_{}", data.col_name);
                    quote! { #w_name.write(__i, #access); }
                }
            })
            .collect();

        let par_skip_bindings: Vec<TokenStream> = skipped_fields
            .iter()
            .map(|name| quote! { let _ = __row.#name; })
            .collect();

        // Return struct fields
        let par_tag_field = if has_tag {
            quote! { _tag, }
        } else {
            TokenStream::new()
        };
        let par_len_field = if flat_cols.is_empty() && auto_expand_cols.is_empty() && !has_tag {
            quote! { _len: len, }
        } else {
            TokenStream::new()
        };
        let mut par_struct_fields: Vec<TokenStream> = flat_cols
            .iter()
            .map(|fc| {
                let name = &fc.df_field;
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
                        #tag_write
                        #(#par_write_calls)*
                        #(#par_skip_bindings)*
                    });
                }
                #df_name { #par_tag_field #par_len_field #(#par_struct_fields),* }
            }
        }
    } else {
        TokenStream::new()
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
            if let ResolvedField::Single(data) = rf {
                if i > 0 {
                    next_struct_tokens.extend(quote! { , });
                }
                let rust_name = &data.rust_name;
                let col_name = &data.col_name;
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
            pub struct #iterator_name #impl_generics #where_clause {
                #(#iter_field_decls),*
            }

            impl #impl_generics IntoIterator for #df_name #ty_generics #where_clause {
                type Item = #row_name #ty_generics;
                type IntoIter = #iterator_name #ty_generics;

                fn into_iter(self) -> Self::IntoIter {
                    let #df_name { #ignore_tag #(#destruct_pattern),* } = self;
                    #iterator_name {
                        #iter_init_tokens
                    }
                }
            }

            impl #impl_generics Iterator for #iterator_name #ty_generics #where_clause {
                type Item = #row_name #ty_generics;

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
    // endregion

    // region: Associated methods
    let from_dataframe_method = if can_iterate {
        quote! {
            /// Convert a DataFrame back into a vector of rows.
            ///
            /// This transposes column-oriented data back into row-oriented format.
            pub fn from_dataframe(df: #df_name #ty_generics) -> Vec<Self> {
                df.into_iter().collect()
            }
        }
    } else {
        TokenStream::new()
    };
    // endregion

    // region: DataFrame type methods (from_rows, from_rows_par)
    let df_methods = quote! {
        impl #impl_generics #df_name #ty_generics #where_clause {
            /// Sequential row→column transposition.
            pub fn from_rows(rows: Vec<#row_name #ty_generics>) -> Self {
                rows.into()
            }

            #from_rows_par_method
        }
    };

    let row_methods = quote! {
        impl #impl_generics #row_name #ty_generics #where_clause {
            /// Name of the generated DataFrame companion type.
            pub const DATAFRAME_TYPE_NAME: &'static str = stringify!(#df_name);

            /// Convert a vector of rows into the companion DataFrame type.
            ///
            /// This transposes row-oriented data into column-oriented format.
            pub fn to_dataframe(rows: Vec<Self>) -> #df_name #ty_generics {
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

    // Marker trait impl: struct type implements DataFrameRow via IntoDataFrame chain.
    let marker_impl = quote! {
        impl #impl_generics ::miniextendr_api::markers::DataFrameRow
            for #row_name #ty_generics #where_clause {}
    };

    Ok(quote! {
        #dataframe_struct
        #into_dataframe_impl
        #from_vec_impl
        #df_methods
        #into_iterator_impl
        #row_methods
        #trait_check
        #marker_impl
    })
    // endregion
}
// endregion

// region: Enum align path

/// A resolved column in the unified schema across all enum variants.
///
/// Tracks the column name, element type, which variants contribute to this column,
/// and whether the type was coerced to `String` due to cross-variant type conflicts
/// (when `#[dataframe(conflicts = "string")]` is active).
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

/// Accumulates unique columns for an enum-to-dataframe unified schema.
///
/// As columns are registered from each variant's fields, the registry detects
/// duplicates and validates type consistency. When `coerce_to_string` is enabled,
/// type conflicts are resolved by coercing to `String`; otherwise they produce errors.
pub(super) struct ColumnRegistry<'a> {
    /// The ordered list of resolved columns in the schema.
    pub(super) columns: Vec<ResolvedColumn>,
    /// Maps column name strings to their index in `columns` for O(1) dedup lookup.
    pub(super) col_index: std::collections::HashMap<String, usize>,
    /// Whether to coerce type-conflicting columns to `String` instead of erroring.
    pub(super) coerce_to_string: bool,
    /// Cached `String` type AST node, used as the coercion target type.
    pub(super) string_ty: &'a syn::Type,
}

impl<'a> ColumnRegistry<'a> {
    /// Create a new empty column registry.
    fn new(coerce_to_string: bool, string_ty: &'a syn::Type) -> Self {
        Self {
            columns: Vec::new(),
            col_index: std::collections::HashMap::new(),
            coerce_to_string,
            string_ty,
        }
    }

    /// Register a single column in the schema, or merge with an existing column.
    ///
    /// If a column with the same name already exists, validates that the types match.
    /// On type conflict: coerces to `String` (if `coerce_to_string` is true) or
    /// returns `Err`. The `variant_idx` is appended to the column's `present_in` list.
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

/// A resolved enum field ready for codegen -- either a single column or expanded
/// from an array/Vec into multiple suffixed columns.
///
/// This is the enum-path counterpart of [`ResolvedField`] (used for structs).
/// Each variant carries both the binding name (for destructure patterns) and the
/// original Rust field name (for error reporting and named-variant patterns).
pub(super) enum EnumResolvedField {
    /// Single column contribution.
    Single(Box<EnumSingleFieldData>),
    /// Expanded from [T; N].
    ExpandedFixed(Box<EnumExpandedFixedData>),
    /// Expanded from Vec<T> with pinned width.
    ExpandedVec(Box<EnumExpandedVecData>),
    /// Auto-expanded Vec<T>/Box<[T]>: column count determined at runtime.
    AutoExpandVec(Box<EnumAutoExpandVecData>),
    /// `HashMap<K,V>` or `BTreeMap<K,V>` → two parallel list-columns: `<field>_keys`, `<field>_values`.
    Map(Box<EnumMapFieldData>),
    /// Struct field whose inner type implements `DataFrameRow` → flattened `<base>_<inner_col>` columns.
    Struct(Box<EnumStructFieldData>),
}

impl EnumResolvedField {
    /// Binding name used in destructure patterns.
    pub(super) fn binding(&self) -> &syn::Ident {
        match self {
            Self::Single(data) => &data.binding,
            Self::ExpandedFixed(data) => &data.binding,
            Self::ExpandedVec(data) => &data.binding,
            Self::AutoExpandVec(data) => &data.binding,
            Self::Map(data) => &data.binding,
            Self::Struct(data) => &data.binding,
        }
    }

    /// Original Rust field name.
    pub(super) fn rust_name(&self) -> &syn::Ident {
        match self {
            Self::Single(data) => &data.rust_name,
            Self::ExpandedFixed(data) => &data.rust_name,
            Self::ExpandedVec(data) => &data.rust_name,
            Self::AutoExpandVec(data) => &data.rust_name,
            Self::Map(data) => &data.rust_name,
            Self::Struct(data) => &data.rust_name,
        }
    }
}

/// Data for [`EnumResolvedField::Single`].
pub(super) struct EnumSingleFieldData {
    /// Column name in the schema.
    pub(super) col_name: syn::Ident,
    /// Binding name used in destructure pattern.
    pub(super) binding: syn::Ident,
    /// Original Rust field name (for named variants).
    pub(super) rust_name: syn::Ident,
    /// Column type stored in the companion Vec.
    ///
    /// For most fields this is the raw Rust type. When `needs_into_list` is
    /// `true` (struct-typed fields with `#[dataframe(as_list)]`), this is
    /// `::miniextendr_api::list::List` — the actual inner type is erased at
    /// the storage level and each row value is converted via `.into_list()`.
    pub(super) ty: syn::Type,
    /// Whether the field's value must be converted via `.into_list()` before
    /// being pushed into the companion `Vec<Option<List>>`.
    ///
    /// Set to `true` only for struct-typed fields (`FieldTypeKind::Struct`)
    /// that carry `#[dataframe(as_list)]`. The companion struct field type is
    /// `Vec<Option<::miniextendr_api::list::List>>` in this case.
    pub(super) needs_into_list: bool,
}

/// Data for [`EnumResolvedField::ExpandedFixed`].
pub(super) struct EnumExpandedFixedData {
    /// Base column name.
    pub(super) base_name: String,
    /// Binding name.
    pub(super) binding: syn::Ident,
    /// Original Rust field name.
    pub(super) rust_name: syn::Ident,
    /// Element type.
    pub(super) elem_ty: syn::Type,
    /// Array length.
    pub(super) len: usize,
}

/// Data for [`EnumResolvedField::ExpandedVec`].
pub(super) struct EnumExpandedVecData {
    /// Base column name.
    pub(super) base_name: String,
    /// Binding name.
    pub(super) binding: syn::Ident,
    /// Original Rust field name.
    pub(super) rust_name: syn::Ident,
    /// Element type.
    pub(super) elem_ty: syn::Type,
    /// Pinned width.
    pub(super) width: usize,
}

/// Data for [`EnumResolvedField::AutoExpandVec`].
pub(super) struct EnumAutoExpandVecData {
    /// Base column name.
    pub(super) base_name: String,
    /// Binding name.
    pub(super) binding: syn::Ident,
    /// Original Rust field name.
    pub(super) rust_name: syn::Ident,
    /// Element type.
    pub(super) elem_ty: syn::Type,
    /// Container type for companion struct (Vec<T> or Box<[T]>).
    pub(super) container_ty: syn::Type,
}

/// Data for [`EnumResolvedField::Map`].
///
/// A `HashMap<K,V>` or `BTreeMap<K,V>` field expands to two parallel list-columns:
/// `<base_name>_keys: Vec<Option<Vec<K>>>` and `<base_name>_values: Vec<Option<Vec<V>>>`.
/// Absent-variant rows get `None` in both columns. Key order follows the map's own
/// iteration order: `BTreeMap` yields sorted keys, `HashMap` yields non-deterministic order.
/// Both are produced via `into_iter().unzip()` which guarantees pairwise alignment.
pub(super) struct EnumMapFieldData {
    /// Base column name (field name or `rename` override).
    pub(super) base_name: String,
    /// Binding name used in destructure pattern.
    pub(super) binding: syn::Ident,
    /// Original Rust field name.
    pub(super) rust_name: syn::Ident,
    /// Key type K.
    pub(super) key_ty: syn::Type,
    /// Value type V.
    pub(super) val_ty: syn::Type,
}

/// Data for [`EnumResolvedField::Struct`].
///
/// A field whose inner type implements `DataFrameRow` expands to `<base_name>_<inner_col>`
/// prefixed columns — one output column per column emitted by the inner type's companion
/// DataFrame. Absent-variant rows produce `None` in every prefixed column.
///
/// The companion struct holds `Vec<Option<Inner>>` (not `Vec<Inner>`). The `into_data_frame`
/// method collects present rows into a dense `Vec<Inner>` (tracking presence indices),
/// calls `Inner::to_dataframe(present_rows)`, extracts named column SEXPs, and scatters
/// them back to the full row count with `None`-fill for absent rows.
pub(super) struct EnumStructFieldData {
    /// Base name for column prefixing (field name or `rename` override).
    pub(super) base_name: String,
    /// Binding name used in destructure pattern.
    pub(super) binding: syn::Ident,
    /// Original Rust field name.
    pub(super) rust_name: syn::Ident,
    /// Inner struct type (used for the compile-time DataFrameRow assertion and codegen).
    pub(super) inner_ty: syn::Type,
}

/// Parsed and resolved information about a single enum variant for DataFrame codegen.
///
/// Contains the variant's name, shape (named/tuple/unit), resolved fields (after
/// applying `#[dataframe(...)]` attributes and type classification), and any
/// skipped field names (needed for complete destructure patterns in named variants).
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
// endregion

// region: Enum-specific expansion (in sub-module)

mod enum_expansion;
use enum_expansion::derive_enum_dataframe;
// endregion
