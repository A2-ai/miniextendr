//! Derive macros for bidirectional row ↔ dataframe conversions.
//!
//! Supports both structs (direct field mapping) and enums (field-name union
//! across variants with `Option<T>` fill for missing fields).

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};
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
/// Mutually exclusive combinations (`as_list` + `expand`, `as_list` + `width`,
/// `as_factor` + `as_list`, `as_factor` + `expand`, `as_factor` + `width`) are
/// rejected during parsing.
#[derive(Default)]
pub(super) struct FieldAttrs {
    /// `#[dataframe(skip)]` -- omit this field from the DataFrame entirely.
    pub(super) skip: bool,
    /// `#[dataframe(rename = "col")]` -- use a custom column name instead of the field name.
    pub(super) rename: Option<String>,
    /// `#[dataframe(as_list)]` -- keep a collection field as a single R list column
    /// (suppresses automatic expansion into suffixed columns).
    pub(super) as_list: bool,
    /// `#[dataframe(as_factor)]` -- treat a unit-only inner enum field as an R factor column.
    /// Only valid on bare-ident enum types (no generic parameters). The inner enum must be
    /// unit-only (`#[derive(DataFrameRow)]` emits `IntoR` and `IntoR for Vec<Option<Self>>`).
    pub(super) as_factor: bool,
    /// `#[dataframe(expand)]` or `#[dataframe(unnest)]` -- explicitly expand a
    /// collection field into multiple suffixed columns.
    expand: bool,
    /// `#[dataframe(width = N)]` -- pin the expansion width for `Vec<T>`, `Box<[T]>`,
    /// or `&[T]` fields. Rows shorter than `N` get `None` for missing positions.
    pub(super) width: Option<usize>,
}

/// Parse field-level `#[dataframe(...)]` attributes from a `syn::Field`.
///
/// Recognizes: `skip`, `rename`, `as_list`, `as_factor`, `expand` (alias `unnest`), and `width`.
/// Validates mutual exclusivity of conflicting options (`as_list` vs `expand`/`width`,
/// `as_factor` vs `as_list`/`expand`/`width`).
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
            } else if meta.path.is_ident("as_factor") {
                attrs.as_factor = true;
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
                    "unknown field attribute; expected `skip`, `rename`, `as_list`, `as_factor`, `expand`, `unnest`, or `width`",
                ))
            }
        })?;
    }

    let span = field.ident.as_ref().map_or(Span::call_site(), |i| i.span());

    // Validation: conflicting options
    if attrs.as_list && attrs.expand {
        return Err(syn::Error::new(
            span,
            "`as_list` and `expand`/`unnest` are mutually exclusive",
        ));
    }
    if attrs.as_list && attrs.width.is_some() {
        return Err(syn::Error::new(
            span,
            "`as_list` and `width` are mutually exclusive",
        ));
    }
    if attrs.as_factor && attrs.as_list {
        return Err(syn::Error::new(
            span,
            "`as_factor` and `as_list` are mutually exclusive",
        ));
    }
    if attrs.as_factor && attrs.expand {
        return Err(syn::Error::new(
            span,
            "`as_factor` and `expand`/`unnest` are mutually exclusive",
        ));
    }
    if attrs.as_factor && attrs.width.is_some() {
        return Err(syn::Error::new(
            span,
            "`as_factor` and `width` are mutually exclusive",
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
    /// `HashMap<K, V>` or `BTreeMap<K, V>`. The two derive paths treat maps
    /// differently:
    /// - *enum path*: expands to two parallel list-columns `<field>_keys` /
    ///   `<field>_values` (see `enum_expansion.rs`).
    /// - *struct path*: resolves to a `Single` opaque list-of-named-lists
    ///   column (`Vec<map>: IntoR`); reader-capable for `String` keys +
    ///   reader-scalar values (#764, see `SingleFieldData::map_reader`).
    ///
    /// Key order follows the map's own iteration order: `BTreeMap` yields
    /// sorted keys, `HashMap` yields non-deterministic order.
    Map {
        key_ty: &'a syn::Type,
        val_ty: &'a syn::Type,
    },
    /// `Option<scalar>` (`Option<f64>`, `Option<String>`, …): one NA-able
    /// column. The companion stores the full `Vec<Option<T>>`, whose `IntoR`
    /// writes `None` as the typed `NA` and whose `TryFromSexp` reads `NA` back
    /// as `None`; the inner type is never needed separately.
    ///
    /// Only the **struct** path accepts this shape. Enum variant fields are
    /// already wrapped in `Option` (absent for other variants), so an
    /// `Option<scalar>` field there would need `Vec<Option<Option<T>>>`, which
    /// has no conversion; `enum_expansion.rs` rejects it with a pointer to the
    /// bare scalar. Non-scalar `Option<…>` payloads (maps, structs, collections)
    /// are still rejected in `classify_field_type` (#484).
    OptionScalar,
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
/// - `HashMap<K, V>` / `BTreeMap<K, V>` -> `Map`
/// - `Option<scalar>`, or `Option` around a datetime crate path -> `OptionScalar`
/// - A datetime crate path (`jiff::Timestamp`, `time::Date`, see
///   [`DATETIME_PATHS`]) -> `Scalar`
/// - Any non-scalar bare path type (single- or multi-segment, e.g. `Point` or
///   `crate::geom::Point`) -> `Struct`
/// - Everything else (known scalars, generic types with args, `::abs::Paths`) -> `Scalar`
///
/// Returns `Err` for shapes the macro cannot classify and that would silently
/// become opaque list-columns: `Option<T>`, `Cow<T>`, `Rc<T>`, `Arc<T>`,
/// `RefCell<T>`, `Cell<T>`, `Mutex<T>`, `RwLock<T>`.  Use
/// `#[dataframe(as_list)]` to opt into list-column treatment explicitly.
pub(super) fn classify_field_type(ty: &syn::Type) -> syn::Result<FieldTypeKind<'_>> {
    // Check for [T; N]
    if let syn::Type::Array(arr) = ty
        && let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Int(lit_int),
            ..
        }) = &arr.len
        && let Ok(n) = lit_int.base10_parse::<usize>()
    {
        return Ok(FieldTypeKind::FixedArray(&arr.elem, n));
    }

    // Check for &[T] and &[T; N]
    if let syn::Type::Reference(ref_ty) = ty {
        // &[T] → BorrowedSlice
        if let syn::Type::Slice(slice) = &*ref_ty.elem {
            return Ok(FieldTypeKind::BorrowedSlice(&slice.elem));
        }
        // &[T; N] → FixedArray (same as owned)
        if let syn::Type::Array(arr) = &*ref_ty.elem
            && let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(lit_int),
                ..
            }) = &arr.len
            && let Ok(n) = lit_int.base10_parse::<usize>()
        {
            return Ok(FieldTypeKind::FixedArray(&arr.elem, n));
        }
    }

    if let syn::Type::Path(type_path) = ty
        && let Some(seg) = type_path.path.segments.last()
        && let syn::PathArguments::AngleBracketed(args) = &seg.arguments
    {
        // Reject wrapper types that would silently fall through to Scalar /
        // Struct and produce a confusing opaque list-column or a downstream
        // DataFrameRow assertion error.  These are the common smart-pointer
        // and interior-mutability types that wrap a meaningful inner type but
        // that DataFrameRow does not know how to expand.
        //
        // The macro has no way to resolve through the wrapper without type-
        // checking (which is unavailable in proc macros). The user must either
        // unwrap to the inner type, or annotate with `#[dataframe(as_list)]`
        // to opt into an explicit opaque list-column.
        //
        // IMPORTANT: The rejection fires on *path identity alone*, before we
        // inspect generic args.  `Cow<'a, T>` has a lifetime as its first
        // generic argument, not a type; inspecting `args.args.first()` as a
        // `GenericArgument::Type` would silently skip `Cow`.  Checking ident
        // before args makes the rejection robust to any generic shape.
        const REJECTED_WRAPPERS: &[&str] = &[
            "Option", "Cow", "Rc", "Arc", "RefCell", "Cell", "Mutex", "RwLock",
        ];
        let name = seg.ident.to_string();
        // `Option<scalar>` is the NA contract (`None` ⇄ `NA`), not a wrapper to
        // see through: `Vec<Option<T>>` has `IntoR` / `TryFromSexp` for every
        // name in `OPTION_SCALAR_NAMES`, so it is a plain single column (#1437).
        if name == "Option"
            && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
        {
            if is_known_scalar_ty(inner, &["u8"]) {
                return Err(syn::Error::new_spanned(
                    ty,
                    "DataFrameRow cannot represent `Option<u8>` as a column: R raw vectors \
                     have no NA value. Use `u8` for a non-missing raw column, or \
                     `Option<u16>` / `Option<i32>` for a nullable integer column.",
                ));
            }
            if is_option_scalar_inner(inner) || is_datetime_ty(inner) {
                return Ok(FieldTypeKind::OptionScalar);
            }
        }
        if REJECTED_WRAPPERS.contains(&name.as_str()) {
            let hint = if name == "Option" {
                "`Option<…>` is supported only around a scalar (`Option<f64>`, \
                 `Option<String>`, …) or a datetime type spelled by its crate path \
                 (`Option<jiff::Timestamp>`, `Option<time::Date>`, …), where it is the \
                 NA-able column contract. "
            } else {
                ""
            };
            return Err(syn::Error::new_spanned(
                ty,
                format!(
                    "DataFrameRow does not support `{name}<…>` directly as a field type. {hint}\
                     Use `#[dataframe(as_list)]` to opt into an explicit opaque list-column, \
                     or unwrap to the inner type (e.g. store the inner value directly, using \
                     a sentinel / empty collection for the absent case)."
                ),
            ));
        }

        // For the collection types below we need the first *type* argument.
        // Skip any leading lifetime or const arguments (e.g. `Cow<'a, B>`
        // has a lifetime first, but `Cow` is already rejected above so we
        // only reach here for other angle-bracketed types).
        let first_type_arg = args.args.iter().find_map(|arg| {
            if let syn::GenericArgument::Type(t) = arg {
                Some(t)
            } else {
                None
            }
        });

        if let Some(inner) = first_type_arg {
            // Check for Vec<T>
            if seg.ident == "Vec" {
                return Ok(FieldTypeKind::VariableVec(inner));
            }

            // Check for Box<[T]>
            if seg.ident == "Box"
                && let syn::Type::Slice(slice) = inner
            {
                return Ok(FieldTypeKind::BoxedSlice(&slice.elem));
            }

            // Check for HashMap<K, V> and BTreeMap<K, V>
            if (seg.ident == "HashMap" || seg.ident == "BTreeMap")
                && let Some(syn::GenericArgument::Type(val_ty)) = args.args.iter().nth(1)
            {
                return Ok(FieldTypeKind::Map {
                    key_ty: inner,
                    val_ty,
                });
            }
        }
    }

    // Any remaining path type whose LAST segment is a bare ident (no generic args)
    // that is NOT a known scalar is treated as a user-defined struct whose
    // `DataFrameRow` derive should be called.  The compile-time assertion
    // `_assert_inner_is_dataframe_row::<Inner>()` in the generated code surfaces a
    // clear error if the inner type doesn't have the derive.
    //
    // Known scalars (i32, f64, String, bool, …) are kept as `Scalar` so that existing
    // enum variants with primitive fields (e.g. `Click { id: i64, x: f64 }`) are not
    // misclassified as struct fields.
    //
    // Multi-segment paths (e.g. `crate::geom::Point`, `geom::Point`) are now correctly
    // classified here — the previous `segs.len() == 1` guard was overly restrictive.
    // Paths with a leading `::` (absolute paths like `::std::ffi::CString`) still fall
    // through to `Scalar`; use `#[dataframe(as_list)]` or an unqualified import if
    // you need a custom treatment.
    //
    // RISK: a user type whose last path segment is named after a known-scalar
    // (e.g. `mymod::String`) still correctly falls through to `Scalar` because of the
    // KNOWN_SCALARS check. A type named `mymod::Option` / `mymod::Vec` would shadow
    // the detection above — accepted per Rust naming convention (canonical names are
    // rarely shadowed). `#[dataframe(as_list)]` is the documented escape hatch.
    //
    // A datetime type spelled by its crate path (`jiff::Timestamp`, `time::Date`)
    // is one column, not a struct to flatten; see `DATETIME_PATHS` for the rule.
    if is_datetime_ty(ty) {
        return Ok(FieldTypeKind::Scalar);
    }
    if let syn::Type::Path(type_path) = ty {
        let segs = &type_path.path.segments;
        // No leading colon (rules out `::std::…` absolute paths) and no self-type.
        if type_path.qself.is_none() && type_path.path.leading_colon.is_none() {
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
                    return Ok(FieldTypeKind::Struct { inner_ty: ty });
                }
            }
        }
    }

    Ok(FieldTypeKind::Scalar)
}

/// Scalar element types whose `Vec<T>` (and `Vec<Option<T>>`) round-trips through
/// `TryFromSexp` — the supported field types for the parallel from-R reader
/// (`try_from_dataframe_par`). This is intentionally narrower than
/// `FieldTypeKind::Scalar`: set/opaque collection types (`HashSet<…>`,
/// `BTreeSet<…>`) also classify as `Scalar` but do NOT implement `Vec<_>:
/// TryFromSexp`, so they must be excluded from the reader path.
///
/// `pub(super)` so `enum_expansion.rs` (in the `dataframe_derive` module dir)
/// can reuse the same allow-list without duplication.
pub(super) const READER_SCALAR_NAMES: &[&str] = &[
    "bool", "f32", "f64", "i8", "i16", "i32", "i64", "u8", "u16", "u32", "String",
];

/// Scalar types accepted inside `Option<…>` as a struct-path field (#1437):
/// every `T` here has `IntoR for Vec<Option<T>>` (typed `NA` for `None`) in
/// `miniextendr-api::into_r`. The reader side is gated separately by
/// [`is_reader_scalar_ty`], so `Option<u64>` / `Option<usize>` fields are
/// write-only, like their bare counterparts.
pub(super) const OPTION_SCALAR_NAMES: &[&str] = &[
    "bool", "f32", "f64", "i8", "i16", "i32", "i64", "u16", "u32", "u64", "isize", "usize",
    "String",
];

/// Datetime types that `DataFrameRow` stores as one column (`T`) or one NA-able
/// column (`Option<T>`): miniextendr-api has `IntoR` and `TryFromSexp` for
/// `Vec<T>` and `Vec<Option<T>>` of each, under its `jiff` / `time` feature.
///
/// Unlike the scalar names, these are recognized by crate path only:
/// `jiff::Timestamp`, `::jiff::Timestamp` or `miniextendr_api::jiff::Timestamp`,
/// never a bare `Timestamp` or `Date`. A proc macro can't see imports, and a
/// bare name may be the user's own struct, which flattens as a nested
/// `DataFrameRow` as before. RISK: a local module named `jiff` or `time`
/// holding a struct of one of these names reads as the crate's type; spell
/// that path `self::time::Date` or `crate::time::Date` to flatten it.
const DATETIME_PATHS: &[&[&str]] = &[
    &["jiff", "Timestamp"],
    &["jiff", "Zoned"],
    &["jiff", "SignedDuration"],
    &["jiff", "civil", "Date"],
    &["time", "OffsetDateTime"],
    &["time", "Date"],
];

/// The [`DATETIME_PATHS`] entry `tp` spells, with or without a leading `::`
/// or the `miniextendr_api::` re-export of the crate.
fn datetime_path(tp: &syn::TypePath) -> Option<&'static [&'static str]> {
    if tp.qself.is_some()
        || tp
            .path
            .segments
            .iter()
            .any(|seg| !matches!(seg.arguments, syn::PathArguments::None))
    {
        return None;
    }
    let idents: Vec<String> = tp
        .path
        .segments
        .iter()
        .map(|seg| seg.ident.to_string())
        .collect();
    let crate_path = match idents.split_first() {
        Some((first, rest)) if first == "miniextendr_api" => rest,
        _ => &idents[..],
    };
    DATETIME_PATHS.iter().copied().find(|path| {
        path.iter()
            .copied()
            .eq(crate_path.iter().map(String::as_str))
    })
}

/// True if `ty` is a datetime type spelled by its crate path ([`DATETIME_PATHS`]).
pub(super) fn is_datetime_ty(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Path(tp) if datetime_path(tp).is_some())
}

/// The datetime type of a `T` or `Option<T>` field ([`DATETIME_PATHS`]).
pub(super) fn datetime_column_ty(ty: &syn::Type) -> Option<&syn::Type> {
    if is_datetime_ty(ty) {
        Some(ty)
    } else {
        option_payload(ty).filter(|inner| is_datetime_ty(inner))
    }
}

/// One compile-time check per datetime type in `tys`: without the matching
/// miniextendr-api feature, the field reports the feature to enable
/// (`markers::DataFrameDateTime`) instead of a bare missing `IntoR` impl.
pub(super) fn datetime_feature_assertions<'a>(
    tys: impl IntoIterator<Item = &'a syn::Type>,
) -> Vec<TokenStream> {
    tys.into_iter()
        .map(|ty| {
            quote! {
                const _: () = {
                    fn _assert_datetime_column<T: ::miniextendr_api::markers::DataFrameDateTime>() {}
                    fn _do_assert() {
                        _assert_datetime_column::<#ty>();
                    }
                };
            }
        })
        .collect()
}

/// Match the final scalar name, as field classification does. Absolute and
/// relative paths have the same conversion support; associated types and
/// generic payloads still need their own classification.
fn is_known_scalar_ty(ty: &syn::Type, names: &[&str]) -> bool {
    if let syn::Type::Path(tp) = ty
        && tp.qself.is_none()
        && let Some(seg) = tp.path.segments.last()
        && matches!(seg.arguments, syn::PathArguments::None)
    {
        return names.contains(&seg.ident.to_string().as_str());
    }
    false
}

/// True if `ty` is `u8`, bare or qualified. R stores it in a raw vector, which
/// has no NA, so a `u8` column cannot pad or omit cells (`Vec<Option<u8>>` has
/// no `IntoR`).
pub(super) fn is_raw_scalar_ty(ty: &syn::Type) -> bool {
    is_known_scalar_ty(ty, &["u8"])
}

/// The payload of `Option<T>` (possibly qualified), if `ty` is one.
pub(super) fn option_payload(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(tp) = ty
        && tp.qself.is_none()
        && let Some(seg) = tp.path.segments.last()
        && seg.ident == "Option"
        && let syn::PathArguments::AngleBracketed(args) = &seg.arguments
        && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
    {
        return Some(inner);
    }
    None
}

/// A `u8` in a column whose cells can be NA. `why` names the source of the NA
/// cells and `fix` the alternative, so the error points at the field instead of
/// the missing `IntoR` impl inside the generated code (#2025).
pub(super) fn raw_na_column_error(ty: &syn::Type, why: &str, fix: &str) -> syn::Error {
    syn::Error::new_spanned(
        ty,
        format!(
            "DataFrameRow cannot store `u8` here: {why}, and R raw vectors have no NA \
             value. {fix}"
        ),
    )
}

/// True if `ty` names a type from [`OPTION_SCALAR_NAMES`] (possibly qualified,
/// without generic args), a valid payload for a single NA-able column.
pub(super) fn is_option_scalar_inner(ty: &syn::Type) -> bool {
    is_known_scalar_ty(ty, OPTION_SCALAR_NAMES)
}

/// True if `ty` names a known scalar, qualified or bare (no `Option` or generics).
///
/// These are the element types whose `Vec<Option<ty>>` round-trips through
/// `TryFromSexp` — required for the *column-expansion* readers, which read each
/// expanded slot as `Vec<Option<elem>>` (the write side wraps every slot in
/// `Option`). Allowing `Option<scalar>` here would ask for `Vec<Option<Option<…>>>`,
/// which has no `TryFromSexp` impl.
///
/// `pub(super)` so `enum_expansion.rs` can reuse it.
pub(super) fn is_bare_reader_scalar_ty(ty: &syn::Type) -> bool {
    is_known_scalar_ty(ty, READER_SCALAR_NAMES)
}

/// True if `ty` is a known scalar path, or `Option<known-scalar>`.
///
/// These are exactly the field types for which the from-R reader can pull a
/// column out as `Vec<ty>` via `TryFromSexp` (scalar `Single` fields and
/// `[T; N]` fixed-array elements, neither of which adds an `Option` wrapper).
///
/// `pub(super)` so `enum_expansion.rs` can reuse it.
pub(super) fn is_reader_scalar_ty(ty: &syn::Type) -> bool {
    if is_bare_reader_scalar_ty(ty) {
        return true;
    }
    // Option<scalar>
    if let syn::Type::Path(tp) = ty
        && let Some(seg) = tp.path.segments.last()
        && seg.ident == "Option"
        && let syn::PathArguments::AngleBracketed(args) = &seg.arguments
        && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
    {
        return is_bare_reader_scalar_ty(inner);
    }
    false
}

/// True if `ty`'s last path segment is the bare ident `String` (path-identity
/// check, same convention as `classify_field_type`).
fn is_string_ty(ty: &syn::Type) -> bool {
    if let syn::Type::Path(tp) = ty
        && let Some(seg) = tp.path.segments.last()
        && matches!(seg.arguments, syn::PathArguments::None)
    {
        return seg.ident == "String";
    }
    false
}

/// Number of *type* arguments on `ty`'s last path segment (0 when not
/// angle-bracketed). Rejects `HashMap<K, V, S>` custom-hasher maps from the
/// reader path — the `Vec<HashMap<String, V>>: TryFromSexp` impl only covers
/// the default hasher.
fn generic_type_arg_count(ty: &syn::Type) -> usize {
    if let syn::Type::Path(tp) = ty
        && let Some(seg) = tp.path.segments.last()
        && let syn::PathArguments::AngleBracketed(args) = &seg.arguments
    {
        return args
            .args
            .iter()
            .filter(|a| matches!(a, syn::GenericArgument::Type(_)))
            .count();
    }
    0
}

/// True if a resolved struct field can be read back out of an R `data.frame`.
///
/// Determines whether the struct gets a generated `try_from_dataframe` reader.
/// Each shape's reader is the inverse of its column-expansion write rule:
/// - `Single` scalar: one column read as `Vec<ty>` (excludes `as_list` and
///   opaque set/collection columns — those classify as `Single` but lack
///   `Vec<_>: TryFromSexp`). A datetime column (`jiff::Timestamp`,
///   `Option<time::Date>`, …) reads back the same way.
/// - `Single` factor: `FactorVec<ty>` validates levels and reads owned variants.
/// - `Single` map column (`HashMap<String, V>` / `BTreeMap<String, V>`): one
///   VECSXP list-of-named-lists column read whole via `Vec<map>: TryFromSexp`
///   — the exact inverse of the `Vec<map>: IntoR` write shape (#764). Gated
///   to `String` keys + reader-scalar values at resolve time (`map_reader`).
/// - `Single` owned list-column: `Vec<scalar>` / `Box<[scalar]>` stored as a
///   VECSXP list-column; the reader deserialises each row's element via
///   `Vec<elem>: TryFromSexp` and `.into()`-converts to the field type (#809).
/// - `ExpandedFixed` (`[T; N]`): `N` columns regrouped into the array.
/// - `ExpandedVec` / `AutoExpandVec` (`Vec<T>`): suffixed `Option` columns
///   flattened back per row (bare-scalar elements only).
/// - `Struct` (nested `DataFrameRow`): always eligible — the reader routes the
///   un-prefixed sub-frame through the inner type's `DataFrameRowConvert`, which
///   degrades to a clear runtime error if the inner shape itself has no reader.
///
/// Borrowed expansion origins (`&[T]` / `&[T; N]`) are not readable (owned R data
/// can't produce a borrow) — flagged via `readable` at resolve time.
fn field_reader_capable(rf: &ResolvedField) -> bool {
    match rf {
        ResolvedField::Single(d) => {
            !d.needs_into_list
                && (d.is_factor
                    || is_reader_scalar_ty(&d.ty)
                    || datetime_column_ty(&d.ty).is_some()
                    || d.map_reader
                    || d.list_elem_ty
                        .as_ref()
                        .is_some_and(is_bare_reader_scalar_ty))
        }
        ResolvedField::ExpandedFixed(d) => d.readable && is_reader_scalar_ty(&d.elem_ty),
        ResolvedField::ExpandedVec(d) => d.readable && is_bare_reader_scalar_ty(&d.elem_ty),
        ResolvedField::AutoExpandVec(d) => d.readable && is_bare_reader_scalar_ty(&d.elem_ty),
        ResolvedField::Struct(_) => true,
        ResolvedField::Map(d) => {
            is_bare_reader_scalar_ty(&d.key_ty) && is_bare_reader_scalar_ty(&d.val_ty)
        }
    }
}

/// True if `ty` is a borrowed reference (`&[T]`, `&[T; N]`, `&str`, …). Such
/// expansion fields can't be reconstructed by value in the R→Rust reader.
pub(super) fn field_is_borrowed_ref(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Reference(_))
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
    /// Auto-expanded `Vec<T>`/`Box<[T]>`: column count determined at runtime from max row length.
    AutoExpandVec(Box<AutoExpandVecData>),
    /// Struct field whose inner type implements `DataFrameRow` (issue #485).
    /// Companion holds `Vec<Inner>`; `into_data_frame` calls `Inner::to_dataframe`
    /// and flattens columns under the `<base>_` prefix.
    Struct(Box<StructFieldData>),
    /// Non-String-keyed `HashMap<K,V>` / `BTreeMap<K,V>` field (#919).
    /// Expands to two parallel list-columns `<base>_keys` / `<base>_values`,
    /// each a `Vec<Vec<K>>` / `Vec<Vec<V>>` (VECSXP of typed vectors). The
    /// reader zips `keys[i]` with `values[i]` back into the map type.
    Map(Box<MapFieldData>),
}

/// Data for [`ResolvedField::Map`] (struct path, non-String-keyed maps, #919).
struct MapFieldData {
    /// Rust field name (for access on the row type).
    rust_name: syn::Ident,
    /// Column name base — keys col = `<base>_keys`, values col = `<base>_values`.
    base_name: String,
    /// Key type `K`.
    key_ty: syn::Type,
    /// Value type `V`.
    val_ty: syn::Type,
    /// Full map type `HashMap<K,V>` / `BTreeMap<K,V>` (used in reader zip).
    map_ty: syn::Type,
    /// Index in tuple struct (None for named).
    tuple_index: Option<syn::Index>,
}

/// Data for [`ResolvedField::Single`].
struct SingleFieldData {
    /// Rust field name (for access).
    rust_name: syn::Ident,
    /// Column name in the DataFrame.
    col_name: syn::Ident,
    /// Column name string.
    col_name_str: String,
    /// Field type stored in the companion `Vec<#ty>`: the field's Rust type, also
    /// for `#[dataframe(as_list)]` on a struct-typed field (see
    /// `needs_into_list`), so the companion never holds R objects.
    ty: syn::Type,
    /// Index in tuple struct (None for named).
    tuple_index: Option<syn::Index>,
    /// `#[dataframe(as_list)]` on a struct-typed field (#485 workaround).
    /// When `true`, the frame assembly builds the list-column with
    /// `into_list_column`, which converts each value via `IntoList::into_list()`
    /// straight into the rooted column (#1748).
    needs_into_list: bool,
    /// `Some(elem)` when this Single field is an un-annotated *owned* collection
    /// (`Vec<scalar>` / `Box<[scalar]>`) stored as an opaque list-column (#809).
    /// The reader deserialises the list-column back into the owned collection per
    /// row via `Vec<elem>: TryFromSexp` then `.into()` to the field container type.
    /// `None` for scalar Single, `as_list`, opaque `Map`/set columns, and borrowed
    /// `&[T]` (not readable).
    list_elem_ty: Option<syn::Type>,
    /// `true` when this Single field is a reader-capable map column (#764):
    /// `HashMap<String, V>` / `BTreeMap<String, V>` with a reader-scalar `V`
    /// and no custom hasher. The column is read whole as `Vec<#ty>` via the
    /// `Vec<map>: TryFromSexp` list-of-named-lists impl — the same `pull_col`
    /// path scalar Singles use, so it only widens the capability gate.
    map_reader: bool,
    /// A unit-only enum stored as one factor column via `UnitEnumFactor`.
    is_factor: bool,
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
    /// Whether the field can be reconstructed by value in the R→Rust reader.
    /// `false` for a borrowed origin (`&[T; N]`) — owned R data can't produce a
    /// borrow, so the struct gets no reader (see [`field_reader_capable`]).
    readable: bool,
    /// The field is `&[T; N]`: its elements are cloned out, not moved.
    borrowed: bool,
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
    /// Whether the field can be reconstructed by value in the R→Rust reader.
    /// `false` for a borrowed origin (`&[T]`); `Vec<T>` / `Box<[T]>` are readable
    /// (the reader collects a `Vec<T>` and `.into()`-converts to the field type).
    readable: bool,
    /// The field is `&[T]`: its elements are cloned out, not moved.
    borrowed: bool,
}

/// Data for [`ResolvedField::Struct`].
///
/// A struct field whose inner type implements `DataFrameRow`. The companion
/// struct holds `Vec<Inner>` (the same type users already pass into
/// `to_dataframe(vec![...])`). At `into_data_frame()` time the inner rows are
/// converted via `Inner::to_dataframe` → `into_named_columns()`, prefixed with
/// `<base_name>_`, and pushed into the parent data.frame.
struct StructFieldData {
    /// Rust field name (for access on the row type).
    rust_name: syn::Ident,
    /// Companion struct field name (ident).
    col_name: syn::Ident,
    /// Column name base used as the R-side prefix (`<base>_<inner_col>`).
    col_name_str: String,
    /// Inner struct type (used for `to_dataframe` dispatch + DataFrameRow assertion).
    inner_ty: syn::Type,
    /// Index in tuple struct (None for named).
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
    /// Container type for companion struct (`Vec<T>` or `Box<[T]>`).
    container_ty: syn::Type,
    /// Index in tuple struct.
    tuple_index: Option<syn::Index>,
    /// Whether the field can be reconstructed by value in the R→Rust reader.
    /// `false` for a borrowed origin (`&[T]`); `Vec<T>` / `Box<[T]>` are readable.
    readable: bool,
    /// The field is `&[T]`: its elements are cloned out, not moved.
    borrowed: bool,
}

/// The same `as_factor` shapes are accepted on struct and enum fields. The
/// generated conversion checks `UnitEnumFactor` on the selected type.
fn validate_factor_field_type(ty: &syn::Type) -> syn::Result<()> {
    match classify_field_type(ty)? {
        FieldTypeKind::Struct { .. } => Ok(()),
        _ => Err(syn::Error::new_spanned(
            ty,
            "`as_factor` is only valid on bare-ident enum/struct types; \
             use `as_list` for generic or complex types, or remove \
             `as_factor` for scalar fields",
        )),
    }
}

/// Resolve a struct field into a [`ResolvedField`], applying field attributes.
///
/// Combines the field's `#[dataframe(...)]` attributes with its type classification
/// to determine the codegen strategy:
/// - `skip` -> returns `None`
/// - `as_list` / `as_factor` -> `Single` (suppresses expansion)
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
        .unwrap_or_else(|| crate::naming::ident_name(&rust_name));
    let col_name = crate::naming::name_ident(&col_name_str);

    let tuple_index = if is_tuple {
        Some(syn::Index::from(index))
    } else {
        None
    };

    let ty = &field.ty;
    // Propagate classification errors (e.g. Option<T>, Arc<T>) when as_list is
    // not set.  The as_list branch below uses `.ok()` to suppress errors.
    let kind = classify_field_type(ty);

    if field_attrs.as_factor {
        validate_factor_field_type(ty)?;
        return Ok(Some(ResolvedField::Single(Box::new(SingleFieldData {
            rust_name,
            col_name,
            col_name_str,
            ty: ty.clone(),
            tuple_index,
            needs_into_list: false,
            list_elem_ty: None,
            map_reader: false,
            is_factor: true,
        }))));
    }

    // as_list suppresses expansion. For struct-typed fields (#485 opt-out), the
    // companion stores the struct values and the frame assembly converts each
    // one via `IntoList::into_list()` straight into the rooted list-column
    // (`into_list_column`). For non-struct as_list fields, the existing
    // behavior is preserved: the field type is serialized natively (this
    // requires `Vec<#ty>: IntoR`).
    if field_attrs.as_list {
        // Use `.ok()` here: `as_list` is an explicit opt-in, so wrapper types
        // like `Option<T>` / `Arc<T>` are allowed — they become opaque list-
        // columns. Any classification error is suppressed and treated as non-Struct.
        let needs_into_list = matches!(
            classify_field_type(ty).ok(),
            Some(FieldTypeKind::Struct { .. })
        );
        return Ok(Some(ResolvedField::Single(Box::new(SingleFieldData {
            rust_name,
            col_name,
            col_name_str,
            ty: ty.clone(),
            tuple_index,
            needs_into_list,
            list_elem_ty: None,
            map_reader: false,
            is_factor: false,
        }))));
    }

    match kind? {
        FieldTypeKind::FixedArray(elem_ty, len) => Ok(Some(ResolvedField::ExpandedFixed(
            Box::new(ExpandedFixedData {
                rust_name,
                base_name: col_name_str,
                elem_ty: elem_ty.clone(),
                len,
                tuple_index,
                readable: !field_is_borrowed_ref(ty),
                borrowed: field_is_borrowed_ref(ty),
            }),
        ))),
        FieldTypeKind::VariableVec(elem_ty)
        | FieldTypeKind::BoxedSlice(elem_ty)
        | FieldTypeKind::BorrowedSlice(elem_ty) => {
            if (field_attrs.width.is_some() || field_attrs.expand) && is_raw_scalar_ty(elem_ty) {
                let attr = if field_attrs.width.is_some() {
                    "width"
                } else {
                    "expand"
                };
                return Err(raw_na_column_error(
                    elem_ty,
                    &format!("`{attr}` pads shorter rows with NA"),
                    &format!(
                        "Use `u16` or `i32` elements for integer columns, or drop `{attr}` \
                         to keep each value as one raw-vector cell of a list-column."
                    ),
                ));
            }
            if let Some(width) = field_attrs.width {
                Ok(Some(ResolvedField::ExpandedVec(Box::new(
                    ExpandedVecData {
                        rust_name,
                        base_name: col_name_str,
                        elem_ty: elem_ty.clone(),
                        width,
                        tuple_index,
                        readable: !field_is_borrowed_ref(ty),
                        borrowed: field_is_borrowed_ref(ty),
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
                        readable: !field_is_borrowed_ref(ty),
                        borrowed: field_is_borrowed_ref(ty),
                    },
                ))))
            } else {
                // No expansion — keep as opaque single column (list-column on R side).
                // Readable owned collections (`Vec<scalar>` / `Box<[scalar]>`) record
                // the element type for the list-column reader (#809). Borrowed `&[T]`
                // is not readable (can't produce a borrow from owned R data).
                Ok(Some(ResolvedField::Single(Box::new(SingleFieldData {
                    rust_name,
                    col_name,
                    col_name_str,
                    ty: ty.clone(),
                    tuple_index,
                    needs_into_list: false,
                    list_elem_ty: if field_is_borrowed_ref(ty) {
                        None
                    } else {
                        Some((*elem_ty).clone())
                    },
                    map_reader: false,
                    is_factor: false,
                }))))
            }
        }
        // Struct-in-struct flattening (issue #485): inner type must implement
        // `DataFrameRow`. Flattening happens at `into_data_frame()` time; the
        // companion stores `Vec<Inner>`. `as_list` opts out (handled above).
        FieldTypeKind::Struct { inner_ty } => {
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
            Ok(Some(ResolvedField::Struct(Box::new(StructFieldData {
                rust_name,
                col_name,
                col_name_str,
                inner_ty: inner_ty.clone(),
                tuple_index,
            }))))
        }
        kind
        @ (FieldTypeKind::Scalar | FieldTypeKind::OptionScalar | FieldTypeKind::Map { .. }) => {
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
            // Struct-path map fields:
            //
            // - `String`-keyed + reader-scalar value + 2 type args (default hasher):
            //   write as one opaque list-of-named-lists column (`Vec<map>: IntoR`).
            //   Reader-capable via `Vec<map>: TryFromSexp` (#764). Falls through
            //   to `Single` with `map_reader: true` — unchanged.
            //
            // - Non-String bare-reader-scalar key + bare-reader-scalar value + 2 type args:
            //   expand to two parallel list-columns `<base>_keys` / `<base>_values` (#919).
            //   `Vec<Vec<K>>: IntoR` and `Vec<Vec<V>>: IntoR` work via the generic
            //   `Vec<Vec<T>>` impl (any `T` with `Vec<T>: IntoR`). Float keys (`f32`/`f64`)
            //   are also bare-reader-scalar but lack `Eq + Hash` / `Ord`, so reject them
            //   with a clear error.
            //
            // - Custom hasher (3+ type args): fall through to `Single` with no reader
            //   (keeps existing behaviour for `HashMap<K, V, S>`).
            //
            // - Non-scalar key or value: emit a clear error directing to `as_list`.
            if let FieldTypeKind::Map { key_ty, val_ty } = kind {
                let two_args = generic_type_arg_count(ty) == 2;
                if is_string_ty(key_ty) && is_reader_scalar_ty(val_ty) && two_args {
                    // String-keyed — existing path (#764).
                    return Ok(Some(ResolvedField::Single(Box::new(SingleFieldData {
                        rust_name,
                        col_name,
                        col_name_str,
                        ty: ty.clone(),
                        tuple_index,
                        needs_into_list: false,
                        list_elem_ty: None,
                        map_reader: true,
                        is_factor: false,
                    }))));
                }
                // Float key check: f32/f64 classify as bare_reader_scalar but are
                // neither Eq+Hash nor Ord, so they can't be a map key at all.
                let is_float_ty = |t: &syn::Type| -> bool {
                    if let syn::Type::Path(tp) = t
                        && let Some(seg) = tp.path.segments.last()
                        && matches!(seg.arguments, syn::PathArguments::None)
                    {
                        let n = seg.ident.to_string();
                        return n == "f32" || n == "f64";
                    }
                    false
                };
                if is_float_ty(key_ty) {
                    return Err(syn::Error::new_spanned(
                        ty,
                        "HashMap/BTreeMap with float keys is not supported \
                         (f32/f64 are not Eq+Hash/Ord); use a newtype wrapper \
                         or `#[dataframe(as_list)]`",
                    ));
                }
                if two_args && is_bare_reader_scalar_ty(key_ty) && is_bare_reader_scalar_ty(val_ty)
                {
                    // Non-String bare-scalar keyed — new parallel _keys/_values path (#919).
                    return Ok(Some(ResolvedField::Map(Box::new(MapFieldData {
                        rust_name,
                        base_name: col_name_str,
                        key_ty: key_ty.clone(),
                        val_ty: val_ty.clone(),
                        map_ty: ty.clone(),
                        tuple_index,
                    }))));
                }
                if two_args
                    && (!is_bare_reader_scalar_ty(key_ty) || !is_bare_reader_scalar_ty(val_ty))
                {
                    // Non-scalar key or value (and not String-keyed) — opaque, no reader.
                    // Fall through to Single below.
                }
                // 3+ type args (custom hasher) or non-scalar — Single with no reader.
            }
            let map_reader = false;
            Ok(Some(ResolvedField::Single(Box::new(SingleFieldData {
                rust_name,
                col_name,
                col_name_str,
                ty: ty.clone(),
                tuple_index,
                needs_into_list: false,
                list_elem_ty: None,
                map_reader,
                is_factor: false,
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
/// For enums: all variants must have named fields. A struct row needs no other
/// trait; nothing generated converts a row to a list.
///
/// # Generated Items
///
/// For a struct `Measurement { time: f64, value: f64 }`:
/// - Struct `MeasurementDataFrame { time: Vec<f64>, value: Vec<f64> }`
/// - `impl IntoDataFrame for MeasurementDataFrame` (rows → R `data.frame`)
/// - `impl ColumnarFrame for MeasurementDataFrame` (rows ↔ the pure-Rust companion)
/// - `impl From<Vec<Measurement>> for MeasurementDataFrame`
/// - `impl IntoIterator for MeasurementDataFrame`
///
/// For an enum:
/// - Companion struct with `Vec<Option<T>>` columns (field-name union)
/// - Optional tag column for variant discrimination
/// - `impl From<Vec<Enum>> for EnumDataFrame`
/// - `impl IntoDataFrame for EnumDataFrame`
/// - `impl ColumnarFrame for EnumDataFrame`
/// - `impl DataFrameRowSplit for Enum` (the split representation behind
///   `IntoDataFrameSplit`)
///
/// # Public surface (which verbs to call)
///
/// - Rows → R `data.frame`: `rows.into_dataframe()?` (`/_par`) or
///   `rows.wrap_data_frame()`, from `IntoDataFrame` / `AsDataFrameExt`.
/// - R `data.frame` → rows: `Vec::<Row>::from_dataframe(&df)?` (`/_par`) from
///   `FromDataFrame`, or the one-call `Row::try_from_dataframe(sexp)` reader.
/// - Rows ↔ the pure-Rust columnar companion: the `ColumnarFrame` trait
///   (`<Row>DataFrame::from_rows` / `from_rows_par` / `into_rows`), or the `std`
///   `Vec<Row>: Into<companion>` / companion `IntoIterator`.
/// - Enums: `rows.into_dataframe_split()` (one `data.frame` per variant) from
///   `IntoDataFrameSplit`, implemented via the hidden `DataFrameRowSplit` bridge
///   on the enum type.
///
/// All the traits above are re-exported from `miniextendr_api::prelude`. The row
/// type also carries two `#[doc(hidden)]` helpers: `to_dataframe(rows) -> companion`,
/// retained only because the struct-flatten / nested-enum write paths build a
/// nested companion via `Inner::to_dataframe(..)` without naming its type, and
/// `to_dataframe_split(rows)`, the body holder behind the `DataFrameRowSplit` bridge.
///
/// # Attributes
///
/// - `#[dataframe(name = "CustomName")]` — Custom companion type name
/// - `#[dataframe(align)]` — Enum alignment mode (accepted but implicit)
/// - `#[dataframe(tag = "col")]` — Add variant discriminator column
pub fn derive_dataframe_row(input: DeriveInput) -> syn::Result<TokenStream> {
    let row_name = &input.ident;

    // Allow lifetime parameters (needed for &[T] borrowed slice fields).
    // Allow type parameters on unit-only enums (all variants are unit) — the
    // companion struct has no field columns to type-parameterise, and the three
    // unit-enum impls (UnitEnumFactor, IntoR, IntoList) handle generics via the
    // split path in enum_expansion.rs.
    // Reject type and const parameters for everything else.
    let has_type_params = input.generics.type_params().next().is_some();
    let has_const_params = input.generics.const_params().next().is_some();
    if has_type_params || has_const_params {
        let is_unit_only_enum = matches!(&input.data, Data::Enum(e)
            if e.variants.iter().all(|v| matches!(v.fields, Fields::Unit)));
        if !is_unit_only_enum {
            return Err(syn::Error::new_spanned(
                &input.generics,
                "DataFrameRow does not support type or const generic parameters",
            ));
        }
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
            type Error = ::core::convert::Infallible;

            #[inline]
            fn try_into_sexp(self) -> ::core::result::Result<::miniextendr_api::SEXP, ::core::convert::Infallible> {
                ::core::result::Result::Ok(self.into_sexp())
            }

            #[inline]
            unsafe fn try_into_sexp_unchecked(self) -> ::core::result::Result<::miniextendr_api::SEXP, ::core::convert::Infallible> {
                self.try_into_sexp()
            }

            #[inline]
            fn into_sexp(self) -> ::miniextendr_api::SEXP {
                ::miniextendr_api::convert::ColumnSource::into_column_list(self).into_sexp()
            }

            #[inline]
            unsafe fn into_sexp_unchecked(self) -> ::miniextendr_api::SEXP {
                ::miniextendr_api::convert::ColumnSource::into_column_list(self).into_sexp()
            }
        }
    })
}
// endregion

// region: Field-type bounds (shared by the struct and enum paths)
//
// The derive asks of a field type only what turning that field into columns
// needs: a column's `IntoR`, `IntoList` for an `as_list` value, `DataFrameRow`
// for a flattened struct. Everything else it emits for a field type exists
// only where the type allows it: the companion's `Debug` / `Clone`, the rows
// iterator that rebuilds `skip` fields with `Default`, and the parallel fill
// behind `from_rows_par` carry conditional bounds
// (`crate::naming::conditional_bound_lifetime`) instead.

/// `where` clause: `where_clause`'s predicates, then `for<'__mx_cond> ty: bound`
/// for each `(ty, bound)` pair.
pub(super) fn conditional_where(
    where_clause: Option<&syn::WhereClause>,
    bounds: &[(TokenStream, TokenStream)],
) -> TokenStream {
    let binder = crate::naming::conditional_bound_lifetime();
    let mut preds: Vec<TokenStream> = where_clause
        .into_iter()
        .flat_map(|wc| wc.predicates.iter())
        .map(|p| quote! { #p })
        .collect();
    preds.extend(
        bounds
            .iter()
            .map(|(ty, bound)| quote! { for<#binder> #ty: #bound }),
    );
    if preds.is_empty() {
        TokenStream::new()
    } else {
        quote! { where #(#preds),* }
    }
}

/// Keep lifetimes erased by column expansion or skipped fields in the companion.
/// Columns that still borrow already carry their lifetimes; only missing ones
/// need a marker. This adds no data column or element-type trait requirements.
pub(super) fn companion_lifetime_marker(
    generics: &syn::Generics,
    fields: &mut Vec<(syn::Ident, TokenStream)>,
) -> (TokenStream, TokenStream) {
    use syn::visit_mut::VisitMut;

    struct MissingLifetimes(Vec<syn::Lifetime>);
    impl VisitMut for MissingLifetimes {
        fn visit_lifetime_mut(&mut self, lifetime: &mut syn::Lifetime) {
            self.0.retain(|candidate| candidate.ident != lifetime.ident);
        }
    }
    let mut missing = MissingLifetimes(
        generics
            .lifetimes()
            .map(|param| param.lifetime.clone())
            .collect(),
    );
    for (_, ty) in fields.iter() {
        missing.visit_type_mut(&mut syn::parse2::<syn::Type>(ty.clone()).unwrap());
    }
    if missing.0.is_empty() {
        return (TokenStream::new(), TokenStream::new());
    }
    let lifetimes = missing.0;
    let mut name = format_ident!("__mx_lifetimes");
    while fields.iter().any(|(field, _)| field == &name) {
        name = format_ident!("{}_", name);
    }
    let ty = quote! { ::core::marker::PhantomData<(#(&#lifetimes (),)*)> };
    fields.push((name.clone(), ty.clone()));
    (
        quote! { #name: #ty, },
        quote! { #name: ::core::marker::PhantomData, },
    )
}

/// `Debug` and `Clone` for a companion, each implemented only when every field
/// type has it. `#[derive(Debug, Clone)]` would demand both of every column's
/// element type (#2013). `fields` are the companion's fields with their types.
pub(super) fn companion_debug_clone(
    df_name: &syn::Ident,
    generics: &syn::Generics,
    fields: &[(syn::Ident, TokenStream)],
) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let bounds = |bound: TokenStream| -> Vec<(TokenStream, TokenStream)> {
        fields
            .iter()
            .map(|(_, ty)| (ty.clone(), bound.clone()))
            .collect()
    };
    let debug_where = conditional_where(where_clause, &bounds(quote! { ::core::fmt::Debug }));
    let clone_where = conditional_where(where_clause, &bounds(quote! { ::core::clone::Clone }));
    let df_name_str = df_name.to_string();
    let idents: Vec<&syn::Ident> = fields.iter().map(|(ident, _)| ident).collect();
    let names: Vec<String> = idents
        .iter()
        .map(|ident| crate::naming::ident_name(ident))
        .collect();
    quote! {
        #[automatically_derived]
        #[allow(unused_lifetimes)]
        impl #impl_generics ::core::fmt::Debug for #df_name #ty_generics #debug_where {
            fn fmt(&self, __f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                __f.debug_struct(#df_name_str)
                    #(.field(#names, &self.#idents))*
                    .finish()
            }
        }

        #[automatically_derived]
        #[allow(unused_lifetimes)]
        impl #impl_generics ::core::clone::Clone for #df_name #ty_generics #clone_where {
            fn clone(&self) -> Self {
                Self { #(#idents: ::core::clone::Clone::clone(&self.#idents)),* }
            }
        }
    }
}

/// The parallel column fill of a companion: `ParallelColumns` with `body` as
/// `from_rows_parallel(rows)`, implemented only where `bounds` hold (see
/// `miniextendr_api::dataframe::ParallelColumns`). Emitted only when
/// miniextendr-api has `rayon`; pair it with [`from_rows_par_override`].
pub(super) fn parallel_columns_impl(
    df_name: &syn::Ident,
    row_name: &syn::Ident,
    generics: &syn::Generics,
    bounds: &[(TokenStream, TokenStream)],
    body: TokenStream,
) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let where_tokens = conditional_where(where_clause, bounds);
    quote! {
        // The `#[cfg(feature = "rayon")]` gate lives on the api-side macro
        // (evaluated against miniextendr-api's own features), not here —
        // stamping the cfg into the consumer crate trips `unexpected_cfgs`
        // wherever the derive is used without a `rayon` feature (#1117).
        ::miniextendr_api::__dataframe_row_when_rayon! {
            #[allow(unused_lifetimes)]
            impl #impl_generics ::miniextendr_api::dataframe::ParallelColumns<#row_name #ty_generics>
                for #df_name #ty_generics #where_tokens
            {
                fn from_rows_parallel(rows: ::std::vec::Vec<#row_name #ty_generics>) -> Self {
                    #body
                }
            }
        }
    }
}

/// The `from_rows_par` override of a companion's `ColumnarFrame` impl: the
/// companion's `ParallelColumns` impl where it applies, `from_rows` otherwise
/// (`miniextendr_api::dataframe::FromRowsParProbe`).
pub(super) fn from_rows_par_override(row_ty: TokenStream) -> TokenStream {
    quote! {
        ::miniextendr_api::__dataframe_row_when_rayon! {
            // The explicit borrow is what lets lookup fall back to the
            // sequential arm; it is not needless.
            #[allow(clippy::needless_borrow)]
            fn from_rows_par(rows: ::std::vec::Vec<#row_ty>) -> Self {
                #[allow(unused_imports)]
                use ::miniextendr_api::dataframe::{
                    FromRowsParParallel as _, FromRowsParSequential as _,
                };
                (&::miniextendr_api::dataframe::FromRowsParProbe::<Self, #row_ty>::new())
                    .fill_columns(rows)
            }
        }
    }
}

/// `<ty as Clone>::clone`, spanned at `ty`: a type without `Clone` is then
/// reported at the user's field instead of at the derive.
pub(super) fn spanned_clone_fn(ty: &syn::Type) -> TokenStream {
    let span = syn::spanned::Spanned::span(ty);
    quote_spanned! {span=> <#ty as ::core::clone::Clone>::clone }
}

/// Bind each element of fixed-array field value `access` to its own variable
/// in `elems`: moved out of an owned array (indexing would need `Copy`), cloned
/// out of a borrowed one, which is the only way to own an element behind a
/// reference.
pub(super) fn fixed_array_elems(
    access: &TokenStream,
    borrowed: bool,
    elem_ty: &syn::Type,
    elems: &[syn::Ident],
) -> TokenStream {
    if borrowed {
        let clone = spanned_clone_fn(elem_ty);
        let lets: Vec<TokenStream> = elems
            .iter()
            .enumerate()
            .map(|(i, elem)| {
                let idx = syn::Index::from(i);
                quote! { let #elem = #clone(&#access[#idx]); }
            })
            .collect();
        quote! { #(#lets)* }
    } else {
        quote! { let [#(#elems),*] = #access; }
    }
}

/// The elements of an expanded `Vec<T>` / `Box<[T]>` / `&[T]` field value
/// `access` as an iterator of owned `T`: moved out of an owned container,
/// cloned out of a borrowed slice. `IntoIterator::into_iter` is called by path
/// so a `Box<[T]>` iterates by value in every edition.
pub(super) fn owned_elems_iter(
    access: &TokenStream,
    borrowed: bool,
    elem_ty: &syn::Type,
) -> TokenStream {
    if borrowed {
        // A closure, not the clone fn itself: the closure is `FnMut` whatever
        // `elem_ty` implements, so a missing `Clone` is reported once, at the
        // field, instead of again wherever the iterator is used.
        let clone = spanned_clone_fn(elem_ty);
        quote! {
            ::core::iter::Iterator::map(
                ::core::iter::IntoIterator::into_iter(#access),
                |__mx_elem: &#elem_ty| #clone(__mx_elem),
            )
        }
    } else {
        quote! { ::core::iter::IntoIterator::into_iter(#access) }
    }
}

/// Bind the first `elems.len()` elements of a `width = N` field value `access`
/// to `Option<T>` variables (`None` past its end; elements past `N` are
/// dropped), through iterator variable `iter`.
pub(super) fn width_elems(
    access: &TokenStream,
    borrowed: bool,
    elem_ty: &syn::Type,
    iter: &syn::Ident,
    elems: &[syn::Ident],
) -> TokenStream {
    let elems_iter = owned_elems_iter(access, borrowed, elem_ty);
    quote! {
        let mut #iter = #elems_iter;
        #(let #elems = ::core::iter::Iterator::next(&mut #iter);)*
    }
}

/// The variables [`fixed_array_elems`] / [`width_elems`] bind for the `n`
/// columns of the field `key` names. Leading underscores are dropped from `key`
/// so the variables stay snake case (`_0` gives `__mxe1_0`).
pub(super) fn expanded_elem_idents(key: &str, n: usize) -> Vec<syn::Ident> {
    let key = key.trim_start_matches('_');
    (1..=n)
        .map(|k| format_ident!("__mxe{}_{}", k, key))
        .collect()
}
// endregion

// region: Struct path (existing logic, extracted)

/// Convert a companion column without adding R objects to the companion:
/// `as_list` uses the rooted list builder, `as_factor` uses `UnitEnumFactor`,
/// and ordinary columns convert the whole `Vec` through `IntoR`.
fn column_sexp(
    name: &syn::Ident,
    ty: &syn::Type,
    needs_into_list: bool,
    is_factor: bool,
) -> TokenStream {
    if needs_into_list {
        let into_list_column = list_column_fn(ty, false);
        quote! { #into_list_column(self.#name) }
    } else if is_factor {
        let span = syn::spanned::Spanned::span(ty);
        quote_spanned! {span=>
            ::miniextendr_api::IntoR::into_sexp(
                ::miniextendr_api::factor::FactorVec::<#ty>::from(self.#name)
            )
        }
    } else {
        quote! { ::miniextendr_api::IntoR::into_sexp(self.#name) }
    }
}

/// `into_list_column::<#ty>` (or `into_option_list_column::<#ty>`, for a column
/// with absent rows), spanned at the field type: a type without `IntoList` is
/// then reported at the field instead of at the derive.
pub(super) fn list_column_fn(ty: &syn::Type, optional: bool) -> TokenStream {
    let span = syn::spanned::Spanned::span(ty);
    let name = if optional {
        "into_option_list_column"
    } else {
        "into_list_column"
    };
    let func = syn::Ident::new(name, span);
    quote_spanned! {span=> ::miniextendr_api::list::#func::<#ty> }
}

/// Generate `DataFrameRow` expansion for struct types.
///
/// Produces:
/// - A companion struct `{Name}DataFrame` with `Vec<T>` columns
/// - `impl IntoDataFrame for {Name}DataFrame`
/// - `impl From<Vec<{Name}>> for {Name}DataFrame`
/// - `impl IntoIterator` (for named structs without expansion)
/// - Associated methods: `to_dataframe`, `from_dataframe`, `from_rows`, `from_rows_par`
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
    // The companion and its row iterator take the row struct's visibility: their
    // `IntoIterator` / `Iterator` impls name the row type as `Item`, so a `pub`
    // companion over a private row is E0446 ("private type in public interface").
    let vis = &input.vis;

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
    // Track which Rust fields were skipped, with their types (for destructure
    // patterns and the rows iterator's `Default` bounds).
    let skipped_fields: Vec<(syn::Ident, syn::Type)> = match &data.fields {
        Fields::Named(fields) => fields
            .named
            .iter()
            .filter_map(|f| {
                let fa = parse_field_attrs(f).ok()?;
                if fa.skip {
                    Some((f.ident.as_ref().unwrap().clone(), f.ty.clone()))
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
        /// `#[dataframe(as_list)]` on a struct-typed field — companion stores
        /// the struct values, converted to a list-column at frame assembly.
        /// The `from_rows_par` pre-pass clones these sequentially instead of
        /// scatter-writing (the struct need not implement `Default`).
        needs_into_list: bool,
        /// Convert this column through `UnitEnumFactor`.
        is_factor: bool,
    }

    let mut flat_cols: Vec<FlatCol> = Vec::new();

    for rf in &resolved {
        match rf {
            ResolvedField::Single(data) => {
                flat_cols.push(FlatCol {
                    df_field: data.col_name.clone(),
                    col_name_str: data.col_name_str.clone(),
                    vec_elem_ty: data.ty.clone(),
                    needs_into_list: data.needs_into_list,
                    is_factor: data.is_factor,
                });
            }
            ResolvedField::ExpandedFixed(data) => {
                for i in 1..=data.len {
                    let name = format!("{}_{}", data.base_name, i);
                    flat_cols.push(FlatCol {
                        df_field: format_ident!("{}_{}", data.base_name, i),
                        col_name_str: name,
                        vec_elem_ty: data.elem_ty.clone(),
                        needs_into_list: false,
                        is_factor: false,
                    });
                }
            }
            ResolvedField::ExpandedVec(data) => {
                for i in 1..=data.width {
                    let name = format!("{}_{}", data.base_name, i);
                    let elem_ty = &data.elem_ty;
                    let opt_ty: syn::Type = syn::parse_quote!(::core::option::Option<#elem_ty>);
                    flat_cols.push(FlatCol {
                        df_field: format_ident!("{}_{}", data.base_name, i),
                        col_name_str: name,
                        vec_elem_ty: opt_ty,
                        needs_into_list: false,
                        is_factor: false,
                    });
                }
            }
            // AutoExpandVec / Struct do not produce FlatCols — handled separately.
            ResolvedField::AutoExpandVec(..) | ResolvedField::Struct(..) => {}
            // Map (#919): two parallel list-columns `<base>_keys` / `<base>_values`.
            // Each is a `Vec<Vec<K>>` / `Vec<Vec<V>>` (VECSXP of typed vectors).
            ResolvedField::Map(data) => {
                let keys_col_name = format!("{}_keys", data.base_name);
                let vals_col_name = format!("{}_values", data.base_name);
                let key_ty = &data.key_ty;
                let val_ty = &data.val_ty;
                let keys_vec_ty: syn::Type = syn::parse_quote!(::std::vec::Vec<#key_ty>);
                let vals_vec_ty: syn::Type = syn::parse_quote!(::std::vec::Vec<#val_ty>);
                flat_cols.push(FlatCol {
                    df_field: format_ident!("{}_keys", data.base_name),
                    col_name_str: keys_col_name,
                    vec_elem_ty: keys_vec_ty,
                    needs_into_list: false,
                    is_factor: false,
                });
                flat_cols.push(FlatCol {
                    df_field: format_ident!("{}_values", data.base_name),
                    col_name_str: vals_col_name,
                    vec_elem_ty: vals_vec_ty,
                    needs_into_list: false,
                    is_factor: false,
                });
            }
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
                    df_field: crate::naming::name_ident(&data.col_name_str),
                    container_ty: data.container_ty.clone(),
                })
            } else {
                None
            }
        })
        .collect();
    let has_auto_expand = !auto_expand_cols.is_empty();
    // endregion

    // region: Collect struct (DataFrameRow-flattened) fields (#485)
    //
    // Only the codegen-time bits are mirrored here — `rust_name` / `tuple_index`
    // are read directly off `ResolvedField::Struct` at the per-row pushes site.
    struct StructCol {
        df_field: syn::Ident,
        col_name_str: String,
        inner_ty: syn::Type,
    }

    let struct_cols: Vec<StructCol> = resolved
        .iter()
        .filter_map(|rf| {
            if let ResolvedField::Struct(data) = rf {
                Some(StructCol {
                    df_field: data.col_name.clone(),
                    col_name_str: data.col_name_str.clone(),
                    inner_ty: data.inner_ty.clone(),
                })
            } else {
                None
            }
        })
        .collect();
    let has_struct = !struct_cols.is_empty();

    // Any `#[dataframe(as_list)]` on a struct-typed field (#485 opt-out) keeps
    // the struct in the companion and becomes a list-column only at frame
    // assembly. The struct need not implement `Default`, so several codegen
    // branches special-case it: IntoIterator generation is skipped, and
    // `from_rows_par` fills the column in a sequential pre-pass.
    let has_into_list_struct = resolved
        .iter()
        .any(|rf| matches!(rf, ResolvedField::Single(d) if d.needs_into_list));
    // endregion

    // region: Companion struct
    let tag_field_decl = if has_tag {
        quote! { pub _tag: ::std::vec::Vec<::std::string::String>, }
    } else {
        TokenStream::new()
    };

    // The column fields with their types: declared below, and bounding the
    // companion's conditional `Debug` / `Clone`.
    let mut column_fields: Vec<(syn::Ident, TokenStream)> = flat_cols
        .iter()
        .map(|fc| {
            let ty = &fc.vec_elem_ty;
            (fc.df_field.clone(), quote! { ::std::vec::Vec<#ty> })
        })
        .collect();
    for ac in &auto_expand_cols {
        let cty = &ac.container_ty;
        column_fields.push((ac.df_field.clone(), quote! { ::std::vec::Vec<#cty> }));
    }
    for sc in &struct_cols {
        let ity = &sc.inner_ty;
        column_fields.push((sc.df_field.clone(), quote! { ::std::vec::Vec<#ity> }));
    }
    let df_fields_tokens: Vec<TokenStream> = column_fields
        .iter()
        .map(|(name, ty)| quote! { pub #name: #ty })
        .collect();

    let has_len_field =
        flat_cols.is_empty() && auto_expand_cols.is_empty() && struct_cols.is_empty() && !has_tag;
    let len_field_decl = if has_len_field {
        quote! { pub _len: usize, }
    } else {
        TokenStream::new()
    };

    let mut companion_fields: Vec<(syn::Ident, TokenStream)> = Vec::new();
    if has_tag {
        companion_fields.push((
            format_ident!("_tag"),
            quote! { ::std::vec::Vec<::std::string::String> },
        ));
    }
    if has_len_field {
        companion_fields.push((format_ident!("_len"), quote! { usize }));
    }
    companion_fields.extend(column_fields);
    let (phantom_field, phantom_init) =
        companion_lifetime_marker(&input.generics, &mut companion_fields);
    let companion_debug_clone = companion_debug_clone(df_name, &input.generics, &companion_fields);

    let dataframe_struct = quote! {
        #vis struct #df_name #impl_generics #where_clause {
            #tag_field_decl
            #len_field_decl
            #phantom_field
            #(#df_fields_tokens),*
        }

        #companion_debug_clone
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
    } else if !struct_cols.is_empty() {
        let first = &struct_cols[0].df_field;
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
            let column = column_sexp(name, &fc.vec_elem_ty, fc.needs_into_list, fc.is_factor);
            quote! { (#name_str, __scope.protect_raw(#column)) }
        })
        .collect();

    let mut length_checks: Vec<TokenStream> = flat_cols
        .iter()
        .map(|fc| {
            let name = &fc.df_field;
            let name_str = &fc.col_name_str;
            quote! {
                ::core::assert!(
                    self.#name.len() == _n_rows,
                    "column length mismatch in {}: column `{}` has length {} but expected {}",
                    ::core::stringify!(#df_name),
                    #name_str,
                    self.#name.len(),
                    _n_rows,
                );
            }
        })
        .collect();
    for sc in &struct_cols {
        let name = &sc.df_field;
        let name_str = &sc.col_name_str;
        length_checks.push(quote! {
            ::core::assert!(
                self.#name.len() == _n_rows,
                "column length mismatch in {}: struct column `{}` has length {} but expected {}",
                ::core::stringify!(#df_name),
                #name_str,
                self.#name.len(),
                _n_rows,
            );
        });
    }

    let into_dataframe_impl = if has_auto_expand || has_struct {
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
                    let column = column_sexp(col_name, &data.ty, data.needs_into_list, data.is_factor);
                    quote! {
                        __df_pairs.push((
                            #col_name_str.to_string(),
                            __scope.protect_raw(#column),
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
                    let elems_iter =
                        owned_elems_iter(&quote! { __row_vec }, data.borrowed, elem_ty);
                    quote! {
                        {
                            let __auto = self.#col_name;
                            let __max = __auto.iter().map(|v| v.len()).max().unwrap_or(0);
                            let mut __cols: ::std::vec::Vec<::std::vec::Vec<::core::option::Option<#elem_ty>>> = (0..__max)
                                .map(|_| ::std::vec::Vec::with_capacity(_n_rows))
                                .collect();
                            for __row_vec in __auto {
                                let mut __elems = #elems_iter;
                                for __col in __cols.iter_mut() {
                                    __col.push(::core::iter::Iterator::next(&mut __elems));
                                }
                            }
                            for (__i, __col) in __cols.into_iter().enumerate() {
                                __df_pairs.push((
                                    ::std::format!("{}_{}", #col_name_str, __i + 1),
                                    __scope.protect_raw(::miniextendr_api::IntoR::into_sexp(__col)),
                                ));
                            }
                        }
                    }
                }
                ResolvedField::Struct(data) => {
                    // Issue #485: convert `Vec<Inner>` via Inner::to_dataframe,
                    // extract its named columns, and push under `<base>_` prefix.
                    let col_name = &data.col_name;
                    let base_name_str = &data.col_name_str;
                    let inner_ty = &data.inner_ty;
                    quote! {
                        {
                            let __inner_df = <#inner_ty>::to_dataframe(self.#col_name);
                            // The inner column list is rooted in __scope, which keeps its
                            // columns alive across the later columns' allocations.
                            let __inner_cols = ::miniextendr_api::convert::ColumnSource::into_named_columns(
                                __inner_df,
                                &__scope,
                            );
                            for (__inner_col_name, __inner_col_sexp) in __inner_cols {
                                __df_pairs.push((
                                    ::std::format!("{}_{}", #base_name_str, __inner_col_name),
                                    __inner_col_sexp,
                                ));
                            }
                        }
                    }
                }
                // Map (#919): push two list-columns `<base>_keys` / `<base>_values`.
                ResolvedField::Map(data) => {
                    let keys_ident = format_ident!("{}_keys", data.base_name);
                    let vals_ident = format_ident!("{}_values", data.base_name);
                    let keys_name = format!("{}_keys", data.base_name);
                    let vals_name = format!("{}_values", data.base_name);
                    quote! {
                        __df_pairs.push((
                            #keys_name.to_string(),
                            __scope.protect_raw(::miniextendr_api::IntoR::into_sexp(self.#keys_ident)),
                        ));
                        __df_pairs.push((
                            #vals_name.to_string(),
                            __scope.protect_raw(::miniextendr_api::IntoR::into_sexp(self.#vals_ident)),
                        ));
                    }
                }
            })
            .collect();

        quote! {
            impl #impl_generics ::miniextendr_api::convert::ColumnSource for #df_name #ty_generics #where_clause {
                fn into_column_list(self) -> ::miniextendr_api::List {
                    let _n_rows = #length_ref;
                    #(#length_checks)*
                    // SAFETY: into_column_list only runs on the R main thread.
                    // ProtectScope keeps each column SEXP rooted across the
                    // next column's allocations; from_raw_pairs writes them
                    // into the parent VECSXP before we drop the scope.
                    unsafe {
                        let __scope = ::miniextendr_api::gc_protect::ProtectScope::new();
                        let mut __df_pairs: ::std::vec::Vec<(
                            ::std::string::String,
                            ::miniextendr_api::SEXP,
                        )> = ::std::vec::Vec::new();
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
            impl #impl_generics ::miniextendr_api::convert::ColumnSource for #df_name #ty_generics #where_clause {
                fn into_column_list(self) -> ::miniextendr_api::List {
                    let _n_rows = #length_ref;
                    #(#length_checks)*
                    // SAFETY: see auto-expand branch.
                    unsafe {
                        let __scope = ::miniextendr_api::gc_protect::ProtectScope::new();
                        ::miniextendr_api::list::List::from_raw_pairs(::std::vec![
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
            quote! { let mut #name: ::std::vec::Vec<#ty> = ::std::vec::Vec::with_capacity(len); }
        })
        .collect();
    for ac in &auto_expand_cols {
        let name = &ac.df_field;
        let cty = &ac.container_ty;
        col_vec_inits.push(
            quote! { let mut #name: ::std::vec::Vec<#cty> = ::std::vec::Vec::with_capacity(len); },
        );
    }
    for sc in &struct_cols {
        let name = &sc.df_field;
        let ity = &sc.inner_ty;
        col_vec_inits.push(
            quote! { let mut #name: ::std::vec::Vec<#ity> = ::std::vec::Vec::with_capacity(len); },
        );
    }

    let tag_init = if has_tag {
        quote! { let mut _tag: ::std::vec::Vec<::std::string::String> = ::std::vec::Vec::with_capacity(len); }
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
                let elems = expanded_elem_idents(
                    &crate::naming::unraw(&data.rust_name).to_string(),
                    data.len,
                );
                let bind = fixed_array_elems(&access, data.borrowed, &data.elem_ty, &elems);
                let pushes: Vec<TokenStream> = elems
                    .iter()
                    .enumerate()
                    .map(|(i, elem)| {
                        let col_ident = format_ident!("{}_{}", data.base_name, i + 1);
                        quote! { #col_ident.push(#elem); }
                    })
                    .collect();
                quote! {
                    #bind
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
                let iter = format_ident!(
                    "__mxv_{}",
                    crate::naming::unraw(&data.rust_name)
                        .to_string()
                        .trim_start_matches('_')
                );
                let elems = expanded_elem_idents(
                    &crate::naming::unraw(&data.rust_name).to_string(),
                    data.width,
                );
                let bind = width_elems(&access, data.borrowed, &data.elem_ty, &iter, &elems);
                let pushes: Vec<TokenStream> = elems
                    .iter()
                    .enumerate()
                    .map(|(i, elem)| {
                        let col_ident = format_ident!("{}_{}", data.base_name, i + 1);
                        quote! { #col_ident.push(#elem); }
                    })
                    .collect();
                quote! {
                    #bind
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
            ResolvedField::Struct(data) => {
                let access = if let Some(idx) = &data.tuple_index {
                    quote! { row.#idx }
                } else {
                    let rust_name = &data.rust_name;
                    quote! { row.#rust_name }
                };
                let col_name = &data.col_name;
                quote! { #col_name.push(#access); }
            }
            // Map (#919): unzip into parallel keys/values vecs.
            ResolvedField::Map(data) => {
                let access = if let Some(idx) = &data.tuple_index {
                    quote! { row.#idx }
                } else {
                    let rust_name = &data.rust_name;
                    quote! { row.#rust_name }
                };
                let keys_col = format_ident!("{}_keys", data.base_name);
                let vals_col = format_ident!("{}_values", data.base_name);
                quote! {
                    let (__mx_keys, __mx_vals) = #access
                        .into_iter()
                        .unzip::<_, _, ::std::vec::Vec<_>, ::std::vec::Vec<_>>();
                    #keys_col.push(__mx_keys);
                    #vals_col.push(__mx_vals);
                }
            }
        })
        .collect();

    let tag_struct_field = if has_tag {
        quote! { _tag, }
    } else {
        TokenStream::new()
    };

    let len_struct_field = if flat_cols.is_empty()
        && auto_expand_cols.is_empty()
        && struct_cols.is_empty()
        && !has_tag
    {
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
    for sc in &struct_cols {
        let name = &sc.df_field;
        col_struct_fields.push(quote! { #name });
    }

    // For skipped fields in destructure: bind to `_`
    let skip_bindings: Vec<TokenStream> = skipped_fields
        .iter()
        .map(|(name, _)| quote! { let _ = row.#name; })
        .collect();

    let from_vec_impl = quote! {
        impl #impl_generics ::core::convert::From<::std::vec::Vec<#row_name #ty_generics>> for #df_name #ty_generics #where_clause {
            fn from(rows: ::std::vec::Vec<#row_name #ty_generics>) -> Self {
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
                    #phantom_init
                    #(#col_struct_fields),*
                }
            }
        }
    };
    // endregion

    // region: Generate from_rows_par (parallel scatter-write via ColumnWriter)
    //
    // The companion's `ParallelColumns` impl moves each row to a rayon worker,
    // which writes the row's values into columns pre-filled with `Default`
    // placeholders. Two field kinds keep their values in the companion and have
    // no placeholder:
    //   - struct (DataFrameRow-flattened) fields (#485): companion stores `Vec<Inner>`.
    //   - `as_list`-on-struct fields (#485 opt-out): converted to a list-column
    //     only at frame assembly.
    // Both are cloned in a sequential pre-pass (`for __prerow in &rows { ... }`)
    // before `into_par_iter()` consumes the rows, and skipped in the parallel loop.
    // The pre-pass is O(n) extra per such field: a constant factor, not a change
    // in asymptotic complexity.
    //
    // What the fill needs of the field types (the rows `Send`, placeholder
    // columns `Default + Send`, pre-pass columns `Clone`) is the impl's
    // conditional `where` clause, so a row shape without it is not an error:
    // `from_rows_par` falls back to `from_rows` (`from_rows_par_override`).
    let has_par_impl = !flat_cols.is_empty()
        || !auto_expand_cols.is_empty()
        || has_tag
        || has_struct
        || has_into_list_struct;
    let (parallel_columns_item, from_rows_par_method) = if has_par_impl {
        let mut par_bounds: Vec<(TokenStream, TokenStream)> = vec![(
            quote! { #row_name #ty_generics },
            quote! { ::core::marker::Send },
        )];
        for fc in &flat_cols {
            let ty = &fc.vec_elem_ty;
            let bound = if fc.needs_into_list {
                quote! { ::core::clone::Clone }
            } else {
                quote! { ::core::default::Default + ::core::marker::Send }
            };
            par_bounds.push((quote! { #ty }, bound));
        }
        for ac in &auto_expand_cols {
            let cty = &ac.container_ty;
            par_bounds.push((
                quote! { #cty },
                quote! { ::core::default::Default + ::core::marker::Send },
            ));
        }
        for sc in &struct_cols {
            let ity = &sc.inner_ty;
            par_bounds.push((quote! { #ity }, quote! { ::core::clone::Clone }));
        }

        // Column declarations:
        //   - scalar / expand cols: `Default` placeholders (scatter-written in parallel)
        //   - struct / as_list-struct cols: filled in the pre-pass
        let mut par_col_decls = Vec::new();
        if has_tag {
            par_col_decls.push(quote! {
                let mut _tag: ::std::vec::Vec<::std::string::String> = ::std::vec![::std::string::String::new(); len];
            });
        }
        // Sequential pre-pass: struct fields.
        // Iterate resolved to pick up tuple_index for tuple-struct outers.
        for rf in &resolved {
            if let ResolvedField::Struct(data) = rf {
                let col_name = &data.col_name;
                let ity = &data.inner_ty;
                let access = if let Some(idx) = &data.tuple_index {
                    quote! { __prerow.#idx }
                } else {
                    let rust_name = &data.rust_name;
                    quote! { __prerow.#rust_name }
                };
                par_col_decls.push(quote! {
                    let mut #col_name: ::std::vec::Vec<#ity> = ::std::vec::Vec::with_capacity(len);
                    for __prerow in &rows {
                        #col_name.push(::core::clone::Clone::clone(&#access));
                    }
                });
            }
        }
        // Sequential pre-pass: as_list-on-struct fields. Pure Rust: the
        // list-column is built at frame assembly.
        for rf in &resolved {
            if let ResolvedField::Single(data) = rf
                && data.needs_into_list
            {
                let col_name = &data.col_name;
                let rust_name = &data.rust_name;
                let ty = &data.ty;
                let access = if let Some(idx) = &data.tuple_index {
                    quote! { __prerow.#idx }
                } else {
                    quote! { __prerow.#rust_name }
                };
                par_col_decls.push(quote! {
                    let mut #col_name: ::std::vec::Vec<#ty> = ::std::vec::Vec::with_capacity(len);
                    for __prerow in &rows {
                        #col_name.push(::core::clone::Clone::clone(&#access));
                    }
                });
            }
        }
        // Parallel scalar/expand columns.
        let placeholders = |name: &syn::Ident, ty: &syn::Type| {
            quote! {
                let mut #name: ::std::vec::Vec<#ty> = ::core::iter::Iterator::collect(
                    ::core::iter::Iterator::take(
                        ::core::iter::repeat_with(<#ty as ::core::default::Default>::default),
                        len,
                    ),
                );
            }
        };
        for fc in &flat_cols {
            if fc.needs_into_list {
                // Handled in the sequential pre-pass above.
                continue;
            }
            par_col_decls.push(placeholders(&fc.df_field, &fc.vec_elem_ty));
        }
        for ac in &auto_expand_cols {
            par_col_decls.push(placeholders(&ac.df_field, &ac.container_ty));
        }

        // Writer declarations (only for scatter-write cols — struct/as_list pre-pass
        // cols are already populated and need no ColumnWriter).
        let mut writer_decls = Vec::new();
        if has_tag {
            writer_decls.push(quote! {
                let __w_tag = unsafe {
                    ::miniextendr_api::rayon_bridge::ColumnWriter::new(&mut _tag)
                };
            });
        }
        for fc in &flat_cols {
            if fc.needs_into_list {
                continue;
            }
            let name = &fc.df_field;
            let w_name = format_ident!("__w_{}", crate::naming::unraw(name));
            writer_decls.push(quote! {
                let #w_name = unsafe {
                    ::miniextendr_api::rayon_bridge::ColumnWriter::new(&mut #name)
                };
            });
        }
        for ac in &auto_expand_cols {
            let name = &ac.df_field;
            let w_name = format_ident!("__w_{}", crate::naming::unraw(name));
            writer_decls.push(quote! {
                let #w_name = unsafe {
                    ::miniextendr_api::rayon_bridge::ColumnWriter::new(&mut #name)
                };
            });
        }

        // Write calls per resolved field (parallel scatter-write only).
        let tag_write = if has_tag {
            quote! { __w_tag.write(__i, #row_name_str.to_string()); }
        } else {
            TokenStream::new()
        };

        let par_write_calls: Vec<TokenStream> = resolved
            .iter()
            .map(|rf| match rf {
                ResolvedField::Single(data) => {
                    if data.needs_into_list {
                        // Handled in the sequential pre-pass; skip in par loop.
                        return TokenStream::new();
                    }
                    let access = if let Some(idx) = &data.tuple_index {
                        quote! { __row.#idx }
                    } else {
                        let rust_name = &data.rust_name;
                        quote! { __row.#rust_name }
                    };
                    let w_name = format_ident!("__w_{}", crate::naming::unraw(&data.col_name));
                    quote! { #w_name.write(__i, #access); }
                }
                ResolvedField::ExpandedFixed(data) => {
                    let access = if let Some(idx) = &data.tuple_index {
                        quote! { __row.#idx }
                    } else {
                        let rust_name = &data.rust_name;
                        quote! { __row.#rust_name }
                    };
                    let elems = expanded_elem_idents(
                        &crate::naming::unraw(&data.rust_name).to_string(),
                        data.len,
                    );
                    let bind = fixed_array_elems(&access, data.borrowed, &data.elem_ty, &elems);
                    let writes: Vec<TokenStream> = elems
                        .iter()
                        .enumerate()
                        .map(|(i, elem)| {
                            let w_name = format_ident!("__w_{}_{}", data.base_name, i + 1);
                            quote! { #w_name.write(__i, #elem); }
                        })
                        .collect();
                    quote! {
                        #bind
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
                    let iter = format_ident!(
                        "__mxv_{}",
                        crate::naming::unraw(&data.rust_name)
                            .to_string()
                            .trim_start_matches('_')
                    );
                    let elems = expanded_elem_idents(
                        &crate::naming::unraw(&data.rust_name).to_string(),
                        data.width,
                    );
                    let bind = width_elems(&access, data.borrowed, &data.elem_ty, &iter, &elems);
                    let writes: Vec<TokenStream> = elems
                        .iter()
                        .enumerate()
                        .map(|(i, elem)| {
                            let w_name = format_ident!("__w_{}_{}", data.base_name, i + 1);
                            quote! { #w_name.write(__i, #elem); }
                        })
                        .collect();
                    quote! {
                        #bind
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
                    let w_name = format_ident!("__w_{}", crate::naming::unraw(&data.col_name));
                    quote! { #w_name.write(__i, #access); }
                }
                // Struct fields (#485) are collected in the sequential pre-pass
                // above; nothing to write in the parallel loop.
                ResolvedField::Struct(_) => TokenStream::new(),
                // Map (#919): unzip into parallel keys/values vecs via scatter-write.
                ResolvedField::Map(data) => {
                    let access = if let Some(idx) = &data.tuple_index {
                        quote! { __row.#idx }
                    } else {
                        let rust_name = &data.rust_name;
                        quote! { __row.#rust_name }
                    };
                    let w_keys = format_ident!("__w_{}_keys", data.base_name);
                    let w_vals = format_ident!("__w_{}_values", data.base_name);
                    quote! {
                        let (__mx_keys, __mx_vals) = #access
                            .into_iter()
                            .unzip::<_, _, ::std::vec::Vec<_>, ::std::vec::Vec<_>>();
                        #w_keys.write(__i, __mx_keys);
                        #w_vals.write(__i, __mx_vals);
                    }
                }
            })
            .collect();

        let par_skip_bindings: Vec<TokenStream> = skipped_fields
            .iter()
            .map(|(name, _)| quote! { let _ = __row.#name; })
            .collect();

        // Return struct fields
        let par_tag_field = if has_tag {
            quote! { _tag, }
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
        for sc in &struct_cols {
            let name = &sc.df_field;
            par_struct_fields.push(quote! { #name });
        }

        // Only emit an into_par_iter call when there are scalar/expand/tag cols
        // to scatter-write; struct/as_list-only structs skip the parallel loop.
        let has_par_cols = !flat_cols.iter().all(|fc| fc.needs_into_list)
            || !auto_expand_cols.is_empty()
            || has_tag;
        let par_loop = if has_par_cols {
            quote! {
                {
                    use ::miniextendr_api::rayon_bridge::rayon::prelude::*;
                    ::miniextendr_api::optionals::parallel::ensure_pool();
                    #(#writer_decls)*
                    rows.into_par_iter().enumerate().for_each(|(__i, __row)| unsafe {
                        #tag_write
                        #(#par_write_calls)*
                        #(#par_skip_bindings)*
                    });
                }
            }
        } else {
            // All columns were collected in the pre-pass; rows already consumed.
            quote! { let _rows = rows; }
        };

        let parallel_columns_item = parallel_columns_impl(
            df_name,
            row_name,
            &input.generics,
            &par_bounds,
            quote! {
                let len = rows.len();
                #(#par_col_decls)*
                #par_loop
                #df_name { #par_tag_field #phantom_init #(#par_struct_fields),* }
            },
        );
        (
            parallel_columns_item,
            from_rows_par_override(quote! { #row_name #ty_generics }),
        )
    } else {
        (TokenStream::new(), TokenStream::new())
    };

    // region: IntoIterator (only for named non-empty structs without expansion)
    let can_iterate = !flat_cols.is_empty()
        && !is_tuple_struct
        && !is_unit_struct
        && !has_expansion
        && !has_into_list_struct;
    let into_iterator_impl = if can_iterate {
        let iterator_name = format_ident!("{}Iterator", df_name);

        let iter_field_decls: Vec<_> = flat_cols
            .iter()
            .map(|fc| {
                let name = &fc.df_field;
                let ty = &fc.vec_elem_ty;
                quote! { #name: ::std::vec::IntoIter<#ty> }
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
            iter_init_tokens.extend(quote! { #name: <::std::vec::Vec<#ty>>::into_iter(#name) });
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

        // Skipped fields are reconstructed via `Default::default()` each time
        // `next()` yields a row, so the companion iterates rows only where every
        // `#[dataframe(skip)]` field type implements `Default`.
        let skip_defaults: Vec<TokenStream> = skipped_fields
            .iter()
            .map(|(name, _)| quote! { , #name: ::core::default::Default::default() })
            .collect();
        let skip_bounds: Vec<(TokenStream, TokenStream)> = skipped_fields
            .iter()
            .map(|(_, ty)| (quote! { #ty }, quote! { ::core::default::Default }))
            .collect();
        let iter_where = conditional_where(where_clause, &skip_bounds);

        quote! {
            #vis struct #iterator_name #impl_generics #where_clause {
                #phantom_field
                #(#iter_field_decls),*
            }

            #[allow(unused_lifetimes)]
            impl #impl_generics ::core::iter::IntoIterator for #df_name #ty_generics #iter_where {
                type Item = #row_name #ty_generics;
                type IntoIter = #iterator_name #ty_generics;

                fn into_iter(self) -> Self::IntoIter {
                    let #df_name { #(#destruct_pattern,)* .. } = self;
                    #iterator_name {
                        #phantom_init
                        #iter_init_tokens
                    }
                }
            }

            #[allow(unused_lifetimes)]
            impl #impl_generics ::core::iter::Iterator for #iterator_name #ty_generics #iter_where {
                type Item = #row_name #ty_generics;

                fn next(&mut self) -> ::core::option::Option<Self::Item> {
                    ::core::option::Option::Some(#row_name {
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

    // The companion→rows path (`from_dataframe`) is not an inherent method: it is the
    // companion's `IntoIterator` impl, surfaced as the documented verb
    // `ColumnarFrame::into_rows` (available for row-iterable companions).

    // region: from-R readers (try_from_dataframe / try_from_dataframe_par, #738/#782)
    //
    // Read an R `data.frame` SEXP directly into `Vec<Self>` without first
    // materialising a companion `#df_name`. A reader is generated for every
    // *named* struct whose fields are all reader-capable (see
    // `field_reader_capable`): scalar `Single` fields, column-expansion fields
    // (`[T; N]` / `Vec<T>` + `width`/`expand`), and struct-flatten fields (nested
    // `DataFrameRow`). Each shape's reader is the exact inverse of its write rule
    // — regroup the suffixed expansion columns, un-prefix and recurse into the
    // nested reader. Struct-path `HashMap<String, V>`/`BTreeMap<String, V>` map
    // columns read whole via `Vec<map>: TryFromSexp` (#764) — they share the
    // scalar `pull_col` path, gated by `SingleFieldData::map_reader` (non-String
    // keys / non-scalar values / custom hashers stay reader-incapable). `skip` /
    // `as_list` / set columns / tuple / unit shapes are not reader-capable and
    // fall through to the trait default (a clear runtime `DataFrameError`).
    // Tagged-enum and enum-`Map` readers already landed (#807/#816) — see
    // `enum_expansion::build_enum_reader`.
    //
    // The parallel variant (`#[cfg(feature = "rayon")]`) splits cleanly along the
    // R-thread boundary: all SEXP access (column extraction, ALTREP
    // materialisation, sub-frame selection + recursive nested reads) happens up
    // front on R's main thread; only then does `(0..nrow).into_par_iter()`
    // assemble each `Self` from the pre-extracted, owned column data by index —
    // pure Rust, zero R API calls. Flattened and factor fields would need
    // `Clone` for by-index assembly, so their `_par` reader delegates to the
    // sequential one (which moves) rather than imposing `Clone`.
    let struct_reader = !is_tuple_struct
        && !is_unit_struct
        && !has_tag
        && skipped_fields.is_empty()
        && !resolved.is_empty()
        && resolved.iter().all(field_reader_capable);

    let has_autoexpand_field = resolved
        .iter()
        .any(|rf| matches!(rf, ResolvedField::AutoExpandVec(_)));
    let has_moved_reader_field = resolved.iter().any(|rf| match rf {
        ResolvedField::Struct(_) => true,
        ResolvedField::Single(data) => data.is_factor,
        _ => false,
    });

    let reader_methods = if struct_reader {
        // Per-field fragments:
        //   extracts   — prelude statements (R-thread): pull/convert columns, run
        //                length checks, materialise nested sub-frames.
        //   seq_decls  — draining-iterator decls for the sequential row loop.
        //   seq_builds — `field: expr` in the sequential `Self { … }` literal
        //                (drains moved columns; indexes only `AutoExpandVec`).
        //   par_builds — `field: expr` in the parallel `Self { … }` literal
        //                (by-index `.clone()`; scalars only — never reached when
        //                a struct-flatten field is present).
        let mut extracts: Vec<TokenStream> = Vec::new();
        let mut seq_decls: Vec<TokenStream> = Vec::new();
        let mut seq_builds: Vec<TokenStream> = Vec::new();
        let mut par_builds: Vec<TokenStream> = Vec::new();

        // Pull a named column out of R as an owned `Vec<#elem>` via `TryFromSexp`
        // (NA-aware, ALTREP-materialising), then length-check it against `__nrow`.
        // Bypasses `DataFrame::column` because its `Error = SexpError` bound is
        // tighter than the scalar element types' `TryFromSexp::Error`.
        let pull_col = |col_var: &syn::Ident,
                        col_name_str: &str,
                        elem_ty: &syn::Type,
                        is_factor: bool| {
            let convert = if is_factor {
                let span = syn::spanned::Spanned::span(elem_ty);
                quote_spanned! {span=>
                    <::miniextendr_api::factor::FactorVec<#elem_ty> as ::miniextendr_api::from_r::TryFromSexp>::try_from_sexp(__col_sexp)
                        .map(|values| values.into_inner())
                }
            } else {
                quote! {
                    <::std::vec::Vec<#elem_ty> as ::miniextendr_api::from_r::TryFromSexp>::try_from_sexp(__col_sexp)
                }
            };
            quote! {
                let #col_var: ::std::vec::Vec<#elem_ty> = {
                    let __col_sexp = __view.column_raw(#col_name_str).ok_or_else(|| {
                        ::std::format!("column `{}` is missing from the data.frame", #col_name_str)
                    })?;
                    #convert
                        .map_err(|e| ::std::format!(
                            "column `{}` could not be converted to the expected type: {}",
                            #col_name_str, e
                        ))?
                };
                if #col_var.len() != __nrow {
                    return ::core::result::Result::Err(::std::format!(
                        "column `{}` has length {} but data.frame has {} rows",
                        #col_name_str, #col_var.len(), __nrow
                    ));
                }
            }
        };

        for rf in &resolved {
            match rf {
                ResolvedField::Single(data) => {
                    let rust_name = &data.rust_name;
                    let col_var = format_ident!("__col_{}", crate::naming::unraw(rust_name));
                    let it_var = format_ident!("__it_{}", crate::naming::unraw(rust_name));
                    match &data.list_elem_ty {
                        // Un-annotated owned collection: opaque list-column (VECSXP). Read
                        // each row's element back via `Vec<elem>: TryFromSexp`, then
                        // `.into()` to the field container type (`Vec<elem>` identity /
                        // `Box<[elem]>`). A non-list column (e.g. an all-empty column
                        // materialised as logical-NA) reads back as `__nrow` empty
                        // collections. (#809)
                        ::core::option::Option::Some(elem_ty) => {
                            let field_ty = &data.ty;
                            let col_name_str = &data.col_name_str;
                            extracts.push(quote! {
                                let #col_var: ::std::vec::Vec<#field_ty> = {
                                    let __col_sexp = __view.column_raw(#col_name_str).ok_or_else(|| {
                                        ::std::format!("column `{}` is missing from the data.frame", #col_name_str)
                                    })?;
                                    // VECSXP check via `SexpExt::is_list` (UFCS — avoids the
                                    // `List::is_list()` bug that calls `is_pair_list()` instead).
                                    if <::miniextendr_api::SEXP as ::miniextendr_api::SexpExt>::is_list(&__col_sexp) {
                                        let __list = unsafe {
                                            ::miniextendr_api::list::List::from_raw(__col_sexp)
                                        };
                                        let __len = __list.len();
                                        let mut __v: ::std::vec::Vec<#field_ty> = ::std::vec::Vec::with_capacity(__len as usize);
                                        for __j in 0..__len {
                                            // in-bounds by construction (0..len)
                                            let __elt = __list.get(__j).unwrap();
                                            let __inner: ::std::vec::Vec<#elem_ty> =
                                                <::std::vec::Vec<#elem_ty> as ::miniextendr_api::from_r::TryFromSexp>::try_from_sexp(__elt)
                                                    .map_err(|e| ::std::format!(
                                                        "column `{}` element {} could not be converted to the expected type: {}",
                                                        #col_name_str, __j, e
                                                    ))?;
                                            __v.push(::core::convert::Into::into(__inner));
                                        }
                                        __v
                                    } else {
                                        // Non-list column → every row is an empty collection.
                                        (0..__nrow)
                                            .map(|_| ::core::convert::Into::into(::std::vec::Vec::<#elem_ty>::new()))
                                            .collect()
                                    }
                                };
                                if #col_var.len() != __nrow {
                                    return ::core::result::Result::Err(::std::format!(
                                        "column `{}` has length {} but data.frame has {} rows",
                                        #col_name_str, #col_var.len(), __nrow
                                    ));
                                }
                            });
                            seq_decls.push(quote! { let mut #it_var = #col_var.into_iter(); });
                            seq_builds.push(quote! { #rust_name: #it_var.next().unwrap() });
                            par_builds.push(
                                quote! { #rust_name: ::core::clone::Clone::clone(&#col_var[__i]) },
                            );
                        }
                        // Scalar Single — and reader-capable map Single (#764):
                        // `pull_col`'s `Vec<#ty>: TryFromSexp` covers both
                        // (maps via the list-of-named-lists impl).
                        ::core::option::Option::None => {
                            extracts.push(pull_col(
                                &col_var,
                                &data.col_name_str,
                                &data.ty,
                                data.is_factor,
                            ));
                            seq_decls.push(quote! { let mut #it_var = #col_var.into_iter(); });
                            seq_builds.push(quote! { #rust_name: #it_var.next().unwrap() });
                            par_builds.push(
                                quote! { #rust_name: ::core::clone::Clone::clone(&#col_var[__i]) },
                            );
                        }
                    }
                }
                // `[T; N]` → columns `base_1..base_N`, each a plain `Vec<elem>`.
                // Regroup into the fixed array per row.
                ResolvedField::ExpandedFixed(data) => {
                    let rust_name = &data.rust_name;
                    let elem_ty = &data.elem_ty;
                    let mut it_nexts: Vec<TokenStream> = Vec::new();
                    let mut idx_clones: Vec<TokenStream> = Vec::new();
                    for k in 1..=data.len {
                        let col_var =
                            format_ident!("__ef_{}_{}", crate::naming::unraw(rust_name), k);
                        let it_var =
                            format_ident!("__efit_{}_{}", crate::naming::unraw(rust_name), k);
                        let col_name_str = format!("{}_{}", data.base_name, k);
                        extracts.push(pull_col(&col_var, &col_name_str, elem_ty, false));
                        seq_decls.push(quote! { let mut #it_var = #col_var.into_iter(); });
                        it_nexts.push(quote! { #it_var.next().unwrap() });
                        idx_clones.push(quote! { ::core::clone::Clone::clone(&#col_var[__i]) });
                    }
                    seq_builds.push(quote! { #rust_name: [ #(#it_nexts),* ] });
                    par_builds.push(quote! { #rust_name: [ #(#idx_clones),* ] });
                }
                // `Vec<T>` + `width = N` → columns `base_1..base_N`, each
                // `Vec<Option<elem>>`. Flatten the N optionals per row back into a
                // `Vec<elem>` (trailing-NA padding from the write side drops out).
                ResolvedField::ExpandedVec(data) => {
                    let rust_name = &data.rust_name;
                    let elem_ty = &data.elem_ty;
                    let opt_ty: syn::Type = syn::parse_quote!(::core::option::Option<#elem_ty>);
                    let mut it_nexts: Vec<TokenStream> = Vec::new();
                    let mut idx_clones: Vec<TokenStream> = Vec::new();
                    for k in 1..=data.width {
                        let col_var =
                            format_ident!("__ev_{}_{}", crate::naming::unraw(rust_name), k);
                        let it_var =
                            format_ident!("__evit_{}_{}", crate::naming::unraw(rust_name), k);
                        let col_name_str = format!("{}_{}", data.base_name, k);
                        extracts.push(pull_col(&col_var, &col_name_str, &opt_ty, false));
                        seq_decls.push(quote! { let mut #it_var = #col_var.into_iter(); });
                        it_nexts.push(quote! { #it_var.next().unwrap() });
                        idx_clones.push(quote! { ::core::clone::Clone::clone(&#col_var[__i]) });
                    }
                    // `.into()` converts the collected `Vec<elem>` to the field's
                    // own container type (`Vec<T>` identity or `Box<[T]>`).
                    seq_builds.push(quote! {
                        #rust_name: [ #(#it_nexts),* ]
                            .into_iter().flatten().collect::<::std::vec::Vec<#elem_ty>>().into()
                    });
                    par_builds.push(quote! {
                        #rust_name: [ #(#idx_clones),* ]
                            .into_iter().flatten().collect::<::std::vec::Vec<#elem_ty>>().into()
                    });
                }
                // `Vec<T>`/`Box<[T]>` + `expand` → a runtime-determined number of
                // columns `name_1..name_k`, each `Vec<Option<elem>>`. Discover them
                // by walking `name_<i>` until the first gap, then flatten per row.
                ResolvedField::AutoExpandVec(data) => {
                    let rust_name = &data.rust_name;
                    let elem_ty = &data.elem_ty;
                    let cols_var = format_ident!("__aev_{}", crate::naming::unraw(rust_name));
                    let col_name_str = &data.col_name_str;
                    extracts.push(quote! {
                        let #cols_var: ::std::vec::Vec<::std::vec::Vec<::core::option::Option<#elem_ty>>> = {
                            let mut __cols: ::std::vec::Vec<::std::vec::Vec<::core::option::Option<#elem_ty>>> =
                                ::std::vec::Vec::new();
                            let mut __k: usize = 1;
                            loop {
                                let __cn = ::std::format!("{}_{}", #col_name_str, __k);
                                match __view.column_raw(&__cn) {
                                    ::core::option::Option::Some(__s) => {
                                        let __c: ::std::vec::Vec<::core::option::Option<#elem_ty>> =
                                            <::std::vec::Vec<::core::option::Option<#elem_ty>> as ::miniextendr_api::from_r::TryFromSexp>::try_from_sexp(__s)
                                                .map_err(|e| ::std::format!(
                                                    "column `{}` could not be converted to the expected type: {}",
                                                    __cn, e
                                                ))?;
                                        if __c.len() != __nrow {
                                            return ::core::result::Result::Err(::std::format!(
                                                "column `{}` has length {} but data.frame has {} rows",
                                                __cn, __c.len(), __nrow
                                            ));
                                        }
                                        __cols.push(__c);
                                        __k += 1;
                                    }
                                    ::core::option::Option::None => break,
                                }
                            }
                            __cols
                        };
                    });
                    // Both seq and par index by row (`__i`): the columns are a
                    // `Vec<Vec<…>>`, so there is nothing to drain field-wise.
                    let build = quote! {
                        #rust_name: #cols_var
                            .iter()
                            .filter_map(|__c| ::core::clone::Clone::clone(&__c[__i]))
                            .collect::<::std::vec::Vec<#elem_ty>>()
                            .into()
                    };
                    seq_builds.push(build.clone());
                    par_builds.push(build);
                }
                // Nested `DataFrameRow` (#485): the inner type's columns were
                // written under a `<base>_` prefix. Select those parent columns,
                // strip the prefix into a fresh sub-frame, and recurse through the
                // inner type's `DataFrameRowConvert` reader. Routing through the
                // trait (rather than `Inner::try_from_dataframe`) keeps this
                // compiling even when the inner shape has no reader — it degrades
                // to a clear runtime error instead.
                ResolvedField::Struct(data) => {
                    let rust_name = &data.rust_name;
                    let inner_ty = &data.inner_ty;
                    let vec_var = format_ident!("__sf_{}", crate::naming::unraw(rust_name));
                    let it_var = format_ident!("__sfit_{}", crate::naming::unraw(rust_name));
                    let base = &data.col_name_str;
                    let prefix_lit = format!("{}_", data.col_name_str);
                    extracts.push(quote! {
                        let #vec_var: ::std::vec::Vec<#inner_ty> = {
                            let __prefix: &str = #prefix_lit;
                            let __names = __view.names();
                            let __sel: ::std::vec::Vec<&str> = __names
                                .iter()
                                .filter(|__n| __n.starts_with(__prefix))
                                .map(|__n| __n.as_str())
                                .collect();
                            if __sel.is_empty() {
                                return ::core::result::Result::Err(::std::format!(
                                    "struct column `{}`: no columns with prefix `{}` found in the data.frame",
                                    #base, __prefix
                                ));
                            }
                            // `select` returns an owned, GC-rooted `BuiltDataFrame`
                            // (#1247); `strip_prefix` (inherent forward) edits the
                            // same frame in place and carries the handle through, so
                            // the sub-frame stays rooted across the CHARSXP
                            // allocations and the recursive read below.
                            let __sub = __view.select(&__sel).strip_prefix(__prefix);
                            let __out = match <#inner_ty as ::miniextendr_api::dataframe::DataFrameRowConvert>::rows_from_dataframe(&__sub) {
                                ::core::option::Option::Some(::core::result::Result::Ok(__v)) => __v,
                                ::core::option::Option::Some(::core::result::Result::Err(__e)) => {
                                    return ::core::result::Result::Err(::std::format!(
                                        "struct column `{}`: {}", #base, __e
                                    ));
                                }
                                ::core::option::Option::None => {
                                    return ::core::result::Result::Err(::std::format!(
                                        "struct column `{}`: nested type has no data.frame reader", #base
                                    ));
                                }
                            };
                            __out
                        };
                        if #vec_var.len() != __nrow {
                            return ::core::result::Result::Err(::std::format!(
                                "struct column `{}` produced {} rows but data.frame has {} rows",
                                #base, #vec_var.len(), __nrow
                            ));
                        }
                    });
                    seq_decls.push(quote! { let mut #it_var = #vec_var.into_iter(); });
                    seq_builds.push(quote! { #rust_name: #it_var.next().unwrap() });
                    par_builds
                        .push(quote! { #rust_name: ::core::clone::Clone::clone(&#vec_var[__i]) });
                }
                // Non-String-keyed map (#919): read two parallel list-columns
                // `<base>_keys` / `<base>_values` (VECSXP of typed vectors), then
                // zip keys[i] with values[i] back into the map type per row.
                // NULL VECSXP elements → empty Vec (not None — struct path uses owned cols).
                // All SEXP access happens in `extracts` on the R thread; the
                // parallel iterator touches only owned Rust `Vec<Vec<K>>` / `Vec<Vec<V>>`.
                ResolvedField::Map(data) => {
                    let rust_name = &data.rust_name;
                    let key_ty = &data.key_ty;
                    let val_ty = &data.val_ty;
                    let map_ty = &data.map_ty;
                    let base = &data.base_name;
                    let keys_col_name = format!("{}_keys", base);
                    let vals_col_name = format!("{}_values", base);
                    let keys_var = format_ident!("__mapcol_{}_keys", base.replace('-', "_"));
                    let vals_var = format_ident!("__mapcol_{}_values", base.replace('-', "_"));
                    let keys_it_var = format_ident!("__mapcol_it_{}_keys", base.replace('-', "_"));
                    let vals_it_var =
                        format_ident!("__mapcol_it_{}_values", base.replace('-', "_"));

                    // Extract helper: walk a VECSXP list-column; NULL/nil element → empty Vec.
                    let extract_col = |col_var: &syn::Ident,
                                       col_name: &str,
                                       elem_ty: &syn::Type| {
                        quote! {
                            let #col_var: ::std::vec::Vec<::std::vec::Vec<#elem_ty>> = {
                                let __col_sexp = __view.column_raw(#col_name).ok_or_else(|| {
                                    ::std::format!("column `{}` is missing from the data.frame", #col_name)
                                })?;
                                if <::miniextendr_api::SEXP as ::miniextendr_api::SexpExt>::is_list(&__col_sexp) {
                                    let __list = unsafe {
                                        ::miniextendr_api::list::List::from_raw(__col_sexp)
                                    };
                                    let __len = __list.len();
                                    let mut __v: ::std::vec::Vec<::std::vec::Vec<#elem_ty>> =
                                        ::std::vec::Vec::with_capacity(__len as usize);
                                    for __j in 0..__len {
                                        let __elt = __list.get(__j).unwrap();
                                        if __elt == ::miniextendr_api::SEXP::nil() {
                                            __v.push(::std::vec::Vec::new());
                                        } else {
                                            let __inner: ::std::vec::Vec<#elem_ty> =
                                                <::std::vec::Vec<#elem_ty> as ::miniextendr_api::from_r::TryFromSexp>::try_from_sexp(__elt)
                                                    .map_err(|e| ::std::format!(
                                                        "column `{}` element {} could not be converted: {}",
                                                        #col_name, __j, e
                                                    ))?;
                                            __v.push(__inner);
                                        }
                                    }
                                    __v
                                } else {
                                    // Non-list column → all rows have empty maps.
                                    (0..__nrow).map(|_| ::std::vec::Vec::new()).collect()
                                }
                            };
                            if #col_var.len() != __nrow {
                                return ::core::result::Result::Err(::std::format!(
                                    "column `{}` has length {} but data.frame has {} rows",
                                    #col_name, #col_var.len(), __nrow
                                ));
                            }
                        }
                    };
                    extracts.push(extract_col(&keys_var, &keys_col_name, key_ty));
                    extracts.push(extract_col(&vals_var, &vals_col_name, val_ty));

                    seq_decls.push(quote! { let mut #keys_it_var = #keys_var.into_iter(); });
                    seq_decls.push(quote! { let mut #vals_it_var = #vals_var.into_iter(); });
                    seq_builds.push(quote! {
                        #rust_name: {
                            let __k = #keys_it_var.next().unwrap();
                            let __v = #vals_it_var.next().unwrap();
                            __k.into_iter().zip(__v).collect::<#map_ty>()
                        }
                    });
                    par_builds.push(quote! {
                        #rust_name: ::core::clone::Clone::clone(&#keys_var[__i])
                            .into_iter()
                            .zip(::core::clone::Clone::clone(&#vals_var[__i]))
                            .collect::<#map_ty>()
                    });
                }
            }
        }

        // Only `AutoExpandVec` builds reference `__i` in the sequential loop; bind
        // the counter only then to avoid an `unused_variables` warning.
        let seq_counter = if has_autoexpand_field {
            quote! { __i }
        } else {
            quote! { _ }
        };

        // Flattened and factor values need not be `Clone`. Move them through
        // the sequential reader, as the enum reader does for the same shapes.
        let par_body = if has_moved_reader_field {
            quote! { Self::try_from_dataframe(sexp) }
        } else {
            quote! {
                use ::miniextendr_api::rayon_bridge::rayon::prelude::*;
                ::miniextendr_api::optionals::parallel::ensure_pool();
                let __view = ::miniextendr_api::dataframe::DataFrame::from_sexp(sexp)
                    .map_err(|e| e.to_string())?;
                let __nrow = __view.nrow();
                #(#extracts)*
                let __rows: ::std::vec::Vec<Self> = (0..__nrow)
                    .into_par_iter()
                    .map(|__i| Self { #(#par_builds),* })
                    .collect();
                ::core::result::Result::Ok(__rows)
            }
        };

        quote! {
            /// Read an R `data.frame` directly into a `Vec<Self>` (sequential).
            ///
            /// Each column is materialised out of R (NA-aware, ALTREP-materialising)
            /// and the rows are assembled by transposing column-major into row-major.
            /// Column-expansion fields are regrouped and nested-struct fields are
            /// read from their `<field>_`-prefixed sub-frame. Returns `Err` with a
            /// descriptive message if a column is missing, mis-typed, or ragged.
            ///
            /// This is the one-call `SEXP → Vec<Self>` reader; the same conversion is also
            /// reachable via the boundary-crossing [`FromDataFrame`] trait as
            /// `Vec::<Self>::from_dataframe(&df)`.
            ///
            /// [`FromDataFrame`]: ::miniextendr_api::dataframe::FromDataFrame
            pub fn try_from_dataframe(
                sexp: ::miniextendr_api::SEXP,
            ) -> ::core::result::Result<::std::vec::Vec<Self>, ::std::string::String> {
                let __view = ::miniextendr_api::dataframe::DataFrame::from_sexp(sexp)
                    .map_err(|e| e.to_string())?;
                let __nrow = __view.nrow();
                #(#extracts)*
                #(#seq_decls)*
                let mut __rows: ::std::vec::Vec<Self> = ::std::vec::Vec::with_capacity(__nrow);
                for #seq_counter in 0..__nrow {
                    __rows.push(Self { #(#seq_builds),* });
                }
                ::core::result::Result::Ok(__rows)
            }

            // api-side rayon gate — see the #1117 note on `from_rows_par` above.
            ::miniextendr_api::__dataframe_row_when_rayon! {
                /// Read an R `data.frame` directly into a `Vec<Self>` (parallel).
                ///
                /// Mirrors [`Self::try_from_dataframe`] but assembles rows off the R
                /// thread via rayon. Safety: all SEXP access (column extraction, ALTREP
                /// materialisation, nested sub-frame reads) happens up front on the
                /// R main thread; the `into_par_iter()` region touches only
                /// pre-extracted owned data and makes no R API calls.
                pub fn try_from_dataframe_par(
                    sexp: ::miniextendr_api::SEXP,
                ) -> ::core::result::Result<::std::vec::Vec<Self>, ::std::string::String> {
                    #par_body
                }
            }
        }
    } else {
        TokenStream::new()
    };
    // endregion

    // region: ColumnarFrame impl on the companion (from_rows / into_rows / from_rows_par)
    //
    // The pure-Rust columnar verbs live on the `ColumnarFrame` trait, not as inherent
    // methods: `from_rows` (= the `From<Vec<Row>>` impl), `into_rows` (= the companion's
    // `IntoIterator`, for row-iterable shapes) and the `from_rows_par` override, which
    // picks the parallel fill where the row shape allows one.
    let df_methods = quote! {
        impl #impl_generics ::miniextendr_api::dataframe::ColumnarFrame<#row_name #ty_generics>
            for #df_name #ty_generics #where_clause
        {
            fn from_rows(rows: ::std::vec::Vec<#row_name #ty_generics>) -> Self {
                rows.into()
            }

            #from_rows_par_method
        }
    };

    // The row type carries the one-call SEXP readers (when reader-capable) plus a hidden
    // `to_dataframe`. The documented rows↔companion verbs live on `ColumnarFrame` / `From` /
    // `IntoIterator`; `to_dataframe` is retained `#[doc(hidden)]` because the struct-flatten
    // write path builds a nested companion via `Inner::to_dataframe(..)` without naming the
    // inner companion type (its name may be customised with `#[dataframe(name = ..)]`).
    let row_methods = quote! {
        impl #impl_generics #row_name #ty_generics #where_clause {
            #[doc(hidden)]
            pub fn to_dataframe(rows: ::std::vec::Vec<Self>) -> #df_name #ty_generics {
                rows.into()
            }

            #reader_methods
        }
    };

    // Marker trait impl: struct type implements DataFrameRow via IntoDataFrame chain.
    let marker_impl = quote! {
        impl #impl_generics ::miniextendr_api::markers::DataFrameRow
            for #row_name #ty_generics #where_clause {}
    };

    // DataFramePayloadFields impl: exposes FIELDS (all resolved column names) and TAG
    // (the #[dataframe(tag = "...")] value, or "") for compile-time collision detection
    // by outer DataFrameRow enums that nest this type as a struct-flattened field.
    let payload_fields_impl = {
        // Collect all column names: flat_cols + struct_col base names.
        let mut field_names: Vec<String> =
            flat_cols.iter().map(|fc| fc.col_name_str.clone()).collect();
        for sc in &struct_cols {
            field_names.push(sc.col_name_str.clone());
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

    // Compile-time assertions for struct-flattened fields (#485): each inner
    // type must implement `DataFrameRow`, otherwise users get a confusing
    // error pointing at the `to_dataframe` call site instead of the field.
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
    let datetime_assertions =
        datetime_feature_assertions(resolved.iter().filter_map(|rf| match rf {
            ResolvedField::Single(d) => datetime_column_ty(&d.ty),
            _ => None,
        }));

    // region: DataFrameRowConvert on Row — orphan-rule bridge for the public verbs
    //
    // The derive cannot write `impl IntoDataFrame for Vec<Row>` directly: the orphan rule
    // forbids it (both `IntoDataFrame` and `Vec` are foreign in the user crate, and `Row` is
    // only *covered* inside `Vec<_>`). Instead it implements the local `DataFrameRowConvert`
    // marker on the local `Row`; miniextendr_api's blanket
    // `impl<T: DataFrameRowConvert> IntoDataFrame/FromDataFrame for Vec<T>` then gives users the
    // public verbs `rows.into_dataframe()?` / `Vec::<Row>::from_dataframe(&df)?`. The methods
    // delegate to the companion engine (`to_dataframe` → `ColumnSource::into_dataframe`), the
    // merged parallel builder (#777 `from_rows_par`) and reader (#765 `try_from_dataframe[_par]`),
    // converting the reader's bare `String` error into the unified `DataFrameError`.

    // The parallel build goes through `from_rows_par` when the derive overrides it for this
    // shape; otherwise it is the sequential transposition.
    let has_par_builder = !from_rows_par_method.is_empty();
    let rows_into_dataframe_par_body = if has_par_builder {
        quote! {
            ::miniextendr_api::convert::ColumnSource::into_dataframe(
                <#df_name #ty_generics as ::miniextendr_api::dataframe::ColumnarFrame<
                    #row_name #ty_generics,
                >>::from_rows_par(rows),
            )
        }
    } else {
        quote! { Self::rows_into_dataframe(rows) }
    };

    // Readers are overridden for every reader-capable struct shape (scalar,
    // column-expansion, struct-flatten — see `struct_reader` / `try_from_dataframe`).
    // Other shapes use the trait default (`None`), surfaced by the blanket as a
    // clear `DataFrameError`.
    let reader_overrides = if struct_reader {
        quote! {
            fn rows_from_dataframe(
                df: &::miniextendr_api::dataframe::DataFrame,
            ) -> ::core::option::Option<::core::result::Result<
                ::std::vec::Vec<Self>,
                ::miniextendr_api::dataframe::DataFrameError,
            >> {
                ::core::option::Option::Some(
                    <#row_name #ty_generics>::try_from_dataframe(df.as_sexp())
                        .map_err(::miniextendr_api::dataframe::DataFrameError::Conversion),
                )
            }

            ::miniextendr_api::__dataframe_row_when_rayon! {
                fn rows_from_dataframe_par(
                    df: &::miniextendr_api::dataframe::DataFrame,
                ) -> ::core::option::Option<::core::result::Result<
                    ::std::vec::Vec<Self>,
                    ::miniextendr_api::dataframe::DataFrameError,
                >> {
                    ::core::option::Option::Some(
                        <#row_name #ty_generics>::try_from_dataframe_par(df.as_sexp())
                            .map_err(::miniextendr_api::dataframe::DataFrameError::Conversion),
                    )
                }
            }
        }
    } else {
        TokenStream::new()
    };

    let datarow_convert_impl = quote! {
        impl #impl_generics ::miniextendr_api::dataframe::DataFrameRowConvert
            for #row_name #ty_generics #where_clause
        {
            fn rows_into_dataframe(
                rows: ::std::vec::Vec<Self>,
            ) -> ::core::result::Result<
                ::miniextendr_api::dataframe::DataFrame,
                ::miniextendr_api::dataframe::DataFrameError,
            > {
                ::miniextendr_api::convert::ColumnSource::into_dataframe(
                    <#df_name #ty_generics as ::core::convert::From<
                        ::std::vec::Vec<#row_name #ty_generics>,
                    >>::from(rows),
                )
            }

            ::miniextendr_api::__dataframe_row_when_rayon! {
                fn rows_into_dataframe_par(
                    rows: ::std::vec::Vec<Self>,
                ) -> ::core::result::Result<
                    ::miniextendr_api::dataframe::DataFrame,
                    ::miniextendr_api::dataframe::DataFrameError,
                > {
                    #rows_into_dataframe_par_body
                }
            }

            #reader_overrides
        }
    };
    // endregion

    Ok(quote! {
        #dataframe_struct
        #into_dataframe_impl
        #from_vec_impl
        #df_methods
        #parallel_columns_item
        #into_iterator_impl
        #row_methods
        #marker_impl
        #payload_fields_impl
        #datarow_convert_impl
        #(#struct_assertions)*
        #(#datetime_assertions)*
    })
    // endregion
}
// endregion

// region: Enum align path

/// Compare column types using canonical standard-library spellings, without
/// changing the type emitted for the first variant. Recurse through containers
/// and references, but never discard arbitrary user module paths or qself.
///
/// A std path is shortened only to a prelude name (`::core::primitive::i32` →
/// `i32`, `std::vec::Vec` → `Vec`); a non-prelude name such as `HashMap` names
/// whatever the user imported. A datetime crate path ([`DATETIME_PATHS`]) drops
/// its leading `::` and `miniextendr_api::` prefix, so `::jiff::Timestamp`
/// matches `jiff::Timestamp`; a bare `Timestamp` is a nested struct field, not
/// a datetime column, so it never reaches this comparison as one. Parentheses and the invisible group around a
/// `macro_rules!` `$t:ty` are dropped. Aliases and renamed imports stay as
/// written, so they remain conflicts. A false match (a user `struct String`
/// in scope) is still caught: every merged column has one Rust type, and the
/// generated code moves values into and out of it without converting them, so
/// rustc rejects the mismatch.
fn same_column_type(left: &syn::Type, right: &syn::Type) -> bool {
    use syn::visit_mut::VisitMut;

    struct StandardPaths;
    impl VisitMut for StandardPaths {
        fn visit_type_mut(&mut self, ty: &mut syn::Type) {
            loop {
                *ty = match ty {
                    syn::Type::Group(group) => (*group.elem).clone(),
                    syn::Type::Paren(paren) => (*paren.elem).clone(),
                    _ => break,
                };
            }
            syn::visit_mut::visit_type_mut(self, ty);
        }

        fn visit_type_path_mut(&mut self, ty: &mut syn::TypePath) {
            syn::visit_mut::visit_type_path_mut(self, ty);
            if ty.qself.is_some() {
                return;
            }
            if let Some(path) = datetime_path(ty) {
                ty.path.leading_colon = None;
                ty.path.segments = path
                    .iter()
                    .map(|segment| syn::PathSegment::from(format_ident!("{segment}")))
                    .collect();
                return;
            }
            let segments: Vec<_> = ty.path.segments.iter().collect();
            let [root, module, item] = segments.as_slice() else {
                return;
            };
            if !matches!(root.arguments, syn::PathArguments::None)
                || !matches!(module.arguments, syn::PathArguments::None)
            {
                return;
            }
            let standard = match (
                module.ident.to_string().as_str(),
                item.ident.to_string().as_str(),
            ) {
                ("primitive", _) | ("option", "Option") => {
                    root.ident == "core" || root.ident == "std"
                }
                ("string", "String") | ("vec", "Vec") | ("boxed", "Box") => {
                    root.ident == "alloc" || root.ident == "std"
                }
                _ => false,
            };
            if standard {
                let item = (*item).clone();
                ty.path.leading_colon = None;
                ty.path.segments = std::iter::once(item).collect();
            }
        }
    }

    if left == right {
        return true;
    }
    let (mut left, mut right) = (left.clone(), right.clone());
    StandardPaths.visit_type_mut(&mut left);
    StandardPaths.visit_type_mut(&mut right);
    left == right
}

/// The fix every cross-variant type-conflict error offers first: the derive
/// compares spellings ([`same_column_type`]), so one type needs one spelling.
pub(super) const SPELL_TYPES_ALIKE: &str = "If both name the same type, spell it the same way \
     in every variant (type aliases and renamed imports are compared as written)";

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
    /// The variant that registered the column first (whose spelling `ty` keeps),
    /// named in a type-conflict error.
    pub(super) first_variant: syn::Ident,
    /// Whether this column was coerced to `String` due to type conflicts.
    /// When true, values are converted via `ToString::to_string()` at push time.
    pub(super) string_coerced: bool,
    /// Whether this column should be emitted as an R factor (via `as_factor` attribute).
    /// When `true`, `into_data_frame` wraps the `Vec<Option<T>>` in `FactorOptionVec<T>`
    /// before calling `IntoR::into_sexp`, using the `UnitEnumFactor` blanket impl.
    pub(super) is_factor: bool,
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
            if !existing.string_coerced && !same_column_type(&existing.ty, col_ty) {
                if self.coerce_to_string {
                    self.columns[idx].ty = self.string_ty.clone();
                    self.columns[idx].string_coerced = true;
                } else {
                    return Err(syn::Error::new(
                        error_span,
                        format!(
                            "type conflict for field `{col_name}`: variant `{variant_name}` has \
                             type `{}`, variant `{}` has `{}`. {SPELL_TYPES_ALIKE}; \
                             otherwise use `#[dataframe(conflicts = \"string\")]` to store \
                             conflicting fields as String",
                            crate::type_inspect::type_display(col_ty),
                            existing.first_variant,
                            crate::type_inspect::type_display(&existing.ty),
                        ),
                    ));
                }
            }
            self.columns[idx].present_in.push(variant_idx);
        } else {
            let idx = self.columns.len();
            self.columns.push(ResolvedColumn {
                col_name: crate::naming::name_ident(col_name),
                ty: col_ty.clone(),
                present_in: vec![variant_idx],
                first_variant: variant_name.clone(),
                string_coerced: false,
                is_factor: false,
            });
            self.col_index.insert(col_name.to_string(), idx);
        }
        Ok(())
    }

    /// Like `register`, but marks the column as a factor column (`is_factor = true`).
    ///
    /// Used for fields annotated with `#[dataframe(as_factor)]`. The companion struct
    /// field type stays `Vec<Option<T>>`, but `into_data_frame` wraps it in
    /// `FactorOptionVec<T>` (using the `UnitEnumFactor` blanket `IntoR` impl).
    pub(super) fn register_factor(
        &mut self,
        col_name: &str,
        col_ty: &syn::Type,
        variant_idx: usize,
        variant_name: &syn::Ident,
        error_span: Span,
    ) -> syn::Result<()> {
        self.register(col_name, col_ty, variant_idx, variant_name, error_span)?;
        if let Some(&idx) = self.col_index.get(col_name) {
            self.columns[idx].is_factor = true;
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
    /// Expanded from `Vec<T>` with pinned width.
    ExpandedVec(Box<EnumExpandedVecData>),
    /// Auto-expanded `Vec<T>`/`Box<[T]>`: column count determined at runtime.
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
    /// Column type stored in the companion Vec: the field's Rust type, also
    /// when `needs_into_list` is set (the companion holds no R objects).
    pub(super) ty: syn::Type,
    /// Whether the column is built by converting each value via `.into_list()`
    /// when the frame is assembled (`into_option_list_column` /
    /// `into_list_column`), rather than through `IntoR` for the whole `Vec`.
    ///
    /// Set to `true` only for struct-typed fields (`FieldTypeKind::Struct`)
    /// that carry `#[dataframe(as_list)]`.
    pub(super) needs_into_list: bool,
    /// Whether the field should be emitted as an R factor column.
    ///
    /// Set to `true` for fields annotated with `#[dataframe(as_factor)]`.
    /// The companion struct field type is `Vec<Option<T>>` (unchanged), but
    /// `into_data_frame` wraps it in `FactorOptionVec<T>` to use the
    /// `UnitEnumFactor`-based blanket `IntoR` impl.
    pub(super) is_factor: bool,
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
    /// The field is `&[T; N]`: its elements are cloned out, not moved.
    pub(super) borrowed: bool,
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
    /// The field is `&[T]`: its elements are cloned out, not moved.
    pub(super) borrowed: bool,
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
    /// Container type for companion struct (`Vec<T>` or `Box<[T]>`).
    pub(super) container_ty: syn::Type,
    /// The field is `&[T]`: its elements are cloned out, not moved.
    pub(super) borrowed: bool,
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
    /// Full original field type (`HashMap<K, V>` / `BTreeMap<K, V>`). The reader
    /// regroups the `_keys`/`_values` list-columns and `collect()`s back into this
    /// exact map type — both `HashMap` and `BTreeMap` implement `FromIterator<(K, V)>`.
    pub(super) map_ty: syn::Type,
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

// region: tests
#[cfg(test)]
mod tests;
// endregion
