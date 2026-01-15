//! # `#[derive(Vctrs)]` - Rust Structs ↔ vctrs S3 Classes
//!
//! This module implements the `#[derive(Vctrs)]` macro which generates
//! vctrs-compatible S3 classes from Rust structs.
//!
//! ## Usage
//!
//! ```ignore
//! #[derive(Vctrs)]
//! #[vctrs(class = "percent", base = "double")]
//! pub struct Percent {
//!     data: Vec<f64>,
//! }
//! ```
//!
//! ## Attributes
//!
//! ### Container-level
//!
//! - `#[vctrs(class = "name")]` - R class name (required)
//! - `#[vctrs(base = "double" | "integer" | "list" | "record")]` - Base vector type
//! - `#[vctrs(abbr = "pct")]` - Abbreviation for `vec_ptype_abbr`
//! - `#[vctrs(inherit_base = true | false)]` - Whether to include base type in class vector
//!
//! ### Method-level (on impl methods)
//!
//! - `#[vctrs(format)]` - Custom format method
//! - `#[vctrs(proxy)]` - Custom vec_proxy
//! - `#[vctrs(restore)]` - Custom vec_restore
//! - `#[vctrs(ptype2)]` - Custom vec_ptype2
//! - `#[vctrs(cast)]` - Custom vec_cast
//! - `#[vctrs(proxy_equal)]` - Custom vec_proxy_equal
//! - `#[vctrs(proxy_compare)]` - Custom vec_proxy_compare
//! - `#[vctrs(proxy_order)]` - Custom vec_proxy_order

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields};

/// Parsed vctrs attributes from struct definition.
#[derive(Default)]
struct VctrsAttrs {
    /// R class name (e.g., "vctrs_percent")
    class: Option<String>,
    /// Base vector type: "double", "integer", "character", "list", "record"
    base: Option<String>,
    /// Abbreviation for vec_ptype_abbr
    abbr: Option<String>,
    /// Whether to inherit base type in class vector
    inherit_base: Option<bool>,
}

/// Parse vctrs attributes from a struct.
fn parse_vctrs_attrs(attrs: &[syn::Attribute]) -> syn::Result<VctrsAttrs> {
    let mut result = VctrsAttrs::default();

    for attr in attrs {
        if attr.path().is_ident("vctrs") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("class") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    result.class = Some(value.value());
                } else if meta.path.is_ident("base") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    result.base = Some(value.value());
                } else if meta.path.is_ident("abbr") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    result.abbr = Some(value.value());
                } else if meta.path.is_ident("inherit_base") {
                    let value: syn::LitBool = meta.value()?.parse()?;
                    result.inherit_base = Some(value.value());
                } else {
                    return Err(meta.error(
                        "unknown vctrs attribute; expected one of: class, base, abbr, inherit_base",
                    ));
                }
                Ok(())
            })?;
        }
    }

    Ok(result)
}

/// Map base type string to SEXPTYPE.
fn base_to_sexptype(base: &str) -> Option<TokenStream> {
    match base {
        "double" | "numeric" => Some(quote! { ::miniextendr_api::ffi::SEXPTYPE::REALSXP }),
        "integer" => Some(quote! { ::miniextendr_api::ffi::SEXPTYPE::INTSXP }),
        "logical" => Some(quote! { ::miniextendr_api::ffi::SEXPTYPE::LGLSXP }),
        "character" => Some(quote! { ::miniextendr_api::ffi::SEXPTYPE::STRSXP }),
        "raw" => Some(quote! { ::miniextendr_api::ffi::SEXPTYPE::RAWSXP }),
        "list" => Some(quote! { ::miniextendr_api::ffi::SEXPTYPE::VECSXP }),
        "record" => Some(quote! { ::miniextendr_api::ffi::SEXPTYPE::VECSXP }),
        _ => None,
    }
}

/// Map base type string to VctrsKind.
fn base_to_kind(base: &str) -> TokenStream {
    match base {
        "record" => quote! { ::miniextendr_api::vctrs::VctrsKind::Rcrd },
        "list" => quote! { ::miniextendr_api::vctrs::VctrsKind::ListOf },
        _ => quote! { ::miniextendr_api::vctrs::VctrsKind::Vctr },
    }
}

/// Generate the Vctrs derive implementation.
pub fn derive_vctrs(input: DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Parse struct-level attributes
    let attrs = parse_vctrs_attrs(&input.attrs)?;

    // Validate: must be a struct
    let _fields = match &input.data {
        Data::Struct(data) => &data.fields,
        Data::Enum(_) => {
            return Err(syn::Error::new_spanned(
                &input,
                "#[derive(Vctrs)] can only be applied to structs (use #[derive(RFactor)] for enums)",
            ));
        }
        Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                &input,
                "#[derive(Vctrs)] can only be applied to structs",
            ));
        }
    };

    // Require class attribute
    let class_name = attrs.class.ok_or_else(|| {
        syn::Error::new_spanned(
            &input,
            "#[derive(Vctrs)] requires #[vctrs(class = \"name\")] attribute",
        )
    })?;

    // Get base type (default to "double")
    let base = attrs.base.as_deref().unwrap_or("double");

    // Validate base type
    let sexptype = base_to_sexptype(base).ok_or_else(|| {
        syn::Error::new_spanned(
            &input,
            format!(
                "unknown base type '{}'; expected one of: double, integer, logical, character, raw, list, record",
                base
            ),
        )
    })?;

    // Get VctrsKind
    let kind = base_to_kind(base);

    // Determine inherit_base_type
    // Default: true for list/record, false for others
    let inherit_base = attrs
        .inherit_base
        .unwrap_or(matches!(base, "list" | "record"));

    // Get abbreviation
    let abbr = match &attrs.abbr {
        Some(a) => quote! { Some(#a) },
        None => quote! { None },
    };

    // Generate VctrsClass implementation
    let vctrs_class_impl = quote! {
        impl #impl_generics ::miniextendr_api::vctrs::VctrsClass for #name #ty_generics #where_clause {
            const CLASS_NAME: &'static str = #class_name;
            const KIND: ::miniextendr_api::vctrs::VctrsKind = #kind;
            const BASE_TYPE: Option<::miniextendr_api::ffi::SEXPTYPE> = Some(#sexptype);
            const INHERIT_BASE_TYPE: bool = #inherit_base;
            const ABBR: Option<&'static str> = #abbr;
        }
    };

    // Generate VctrsRecord implementation if base is "record"
    let record_impl = if base == "record" {
        let field_names = extract_field_names(&input)?;
        let field_name_strs: Vec<&str> = field_names.iter().map(|s| s.as_str()).collect();

        quote! {
            impl #impl_generics ::miniextendr_api::vctrs::VctrsRecord for #name #ty_generics #where_clause {
                fn field_names() -> &'static [&'static str] {
                    &[#(#field_name_strs),*]
                }
            }
        }
    } else {
        TokenStream::new()
    };

    Ok(quote! {
        #vctrs_class_impl
        #record_impl
    })
}

/// Extract field names from a struct for record types.
fn extract_field_names(input: &DeriveInput) -> syn::Result<Vec<String>> {
    let fields = match &input.data {
        Data::Struct(data) => &data.fields,
        _ => return Ok(Vec::new()),
    };

    match fields {
        Fields::Named(named) => Ok(named
            .named
            .iter()
            .filter_map(|f| f.ident.as_ref().map(|i| i.to_string()))
            .collect()),
        Fields::Unnamed(_) => Err(syn::Error::new_spanned(
            fields,
            "vctrs record types require named fields",
        )),
        Fields::Unit => Ok(Vec::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_vctr() {
        let input: DeriveInput = syn::parse_quote! {
            #[vctrs(class = "percent", base = "double")]
            struct Percent {
                data: Vec<f64>,
            }
        };

        let result = derive_vctrs(input).unwrap();
        let code = result.to_string();

        assert!(code.contains("VctrsClass"));
        assert!(code.contains("CLASS_NAME"));
        assert!(code.contains("percent"));
        assert!(code.contains("REALSXP"));
    }

    #[test]
    fn test_record_vctr() {
        let input: DeriveInput = syn::parse_quote! {
            #[vctrs(class = "rational", base = "record")]
            struct Rational {
                n: Vec<i32>,
                d: Vec<i32>,
            }
        };

        let result = derive_vctrs(input).unwrap();
        let code = result.to_string();

        assert!(code.contains("VctrsClass"));
        assert!(code.contains("VctrsRecord"));
        assert!(code.contains("Rcrd"));
        assert!(code.contains("field_names"));
    }

    #[test]
    fn test_with_abbr() {
        let input: DeriveInput = syn::parse_quote! {
            #[vctrs(class = "percent", base = "double", abbr = "pct")]
            struct Percent {
                data: Vec<f64>,
            }
        };

        let result = derive_vctrs(input).unwrap();
        let code = result.to_string();

        assert!(code.contains("pct"));
    }

    #[test]
    fn test_missing_class_error() {
        let input: DeriveInput = syn::parse_quote! {
            #[vctrs(base = "double")]
            struct Percent {
                data: Vec<f64>,
            }
        };

        let result = derive_vctrs(input);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("requires #[vctrs(class"));
    }

    #[test]
    fn test_enum_error() {
        let input: DeriveInput = syn::parse_quote! {
            #[vctrs(class = "color")]
            enum Color {
                Red,
                Green,
                Blue,
            }
        };

        let result = derive_vctrs(input);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("can only be applied to structs"));
    }
}
