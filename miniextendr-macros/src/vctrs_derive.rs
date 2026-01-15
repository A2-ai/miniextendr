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
//!     #[vctrs(data)]
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
//! ### Field-level
//!
//! - `#[vctrs(data)]` - Mark field as the underlying data (required for `IntoVctrs`)
//! - `#[vctrs(skip)]` - Skip field when generating record fields
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
//!
//! ## Module Registration
//!
//! Types with `#[derive(Vctrs)]` must be registered in `miniextendr_module!`:
//!
//! ```ignore
//! miniextendr_module! {
//!     mod mypackage;
//!     vctrs Percent;  // Register vctrs type
//! }
//! ```

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Data, DeriveInput, Fields, Ident};

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

/// Parsed vctrs attributes from a field.
#[derive(Default)]
struct VctrsFieldAttrs {
    /// Mark as the data field for IntoVctrs
    is_data: bool,
    /// Skip this field in record generation
    skip: bool,
}

/// Information about a struct field.
struct FieldInfo {
    ident: syn::Ident,
    attrs: VctrsFieldAttrs,
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

/// Parse vctrs attributes from a field.
fn parse_vctrs_field_attrs(attrs: &[syn::Attribute]) -> syn::Result<VctrsFieldAttrs> {
    let mut result = VctrsFieldAttrs::default();

    for attr in attrs {
        if attr.path().is_ident("vctrs") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("data") {
                    result.is_data = true;
                } else if meta.path.is_ident("skip") {
                    result.skip = true;
                } else {
                    return Err(meta.error("unknown vctrs field attribute; expected: data, skip"));
                }
                Ok(())
            })?;
        }
    }

    Ok(result)
}

/// Extract field information from a struct.
fn extract_fields(input: &DeriveInput) -> syn::Result<Vec<FieldInfo>> {
    let fields = match &input.data {
        Data::Struct(data) => &data.fields,
        _ => return Ok(Vec::new()),
    };

    match fields {
        Fields::Named(named) => {
            let mut result = Vec::new();
            for field in &named.named {
                if let Some(ident) = &field.ident {
                    let attrs = parse_vctrs_field_attrs(&field.attrs)?;
                    result.push(FieldInfo {
                        ident: ident.clone(),
                        attrs,
                    });
                }
            }
            Ok(result)
        }
        Fields::Unnamed(_) => Err(syn::Error::new_spanned(
            fields,
            "vctrs types require named fields",
        )),
        Fields::Unit => Ok(Vec::new()),
    }
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

/// Generate R wrapper code for vctrs S3 methods.
///
/// This generates the following S3 methods for the vctrs class:
/// - `format.<class>()` - Format for printing
/// - `vec_ptype_abbr.<class>()` - Abbreviation (if provided)
/// - `vec_ptype_full.<class>()` - Full type name
///
/// For record types, it additionally generates:
/// - Field accessor `$` methods via vctrs infrastructure
fn generate_r_wrappers(
    class: &str,
    base: &str,
    abbr: Option<&str>,
    record_fields: &[String],
) -> String {
    let mut r_code = String::new();

    // Generate format method
    if base == "record" {
        // Record format: paste fields together with separator
        let field_formats: Vec<String> = record_fields
            .iter()
            .map(|f| format!("vctrs::field(x, \"{f}\")"))
            .collect();
        let fields_str = field_formats.join(", \"/\", ");
        r_code.push_str(&format!(
            r#"
#' @export
format.{class} <- function(x, ...) {{
  paste0({fields_str})
}}
"#
        ));
    } else {
        // Simple vctr format: use underlying data representation
        r_code.push_str(&format!(
            r#"
#' @export
format.{class} <- function(x, ...) {{
  format(vctrs::vec_data(x), ...)
}}
"#
        ));
    }

    // Generate vec_ptype_abbr if abbreviation provided
    if let Some(abbr) = abbr {
        r_code.push_str(&format!(
            r#"
#' @export
vec_ptype_abbr.{class} <- function(x, ...) {{
  "{abbr}"
}}
"#
        ));
    }

    // Generate vec_ptype_full
    r_code.push_str(&format!(
        r#"
#' @export
vec_ptype_full.{class} <- function(x, ...) {{
  "{class}"
}}
"#
    ));

    r_code
}

/// Generate the Vctrs derive implementation.
pub fn derive_vctrs(input: DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Parse struct-level attributes
    let attrs = parse_vctrs_attrs(&input.attrs)?;

    // Validate: must be a struct
    match &input.data {
        Data::Struct(_) => {}
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

    // Extract field information
    let fields = extract_fields(&input)?;

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

    // Find data field for IntoVctrs
    let data_field = fields.iter().find(|f| f.attrs.is_data);

    // Generate IntoVctrs implementation if data field is marked
    let into_vctrs_impl = if let Some(data_field) = data_field {
        let data_ident = &data_field.ident;

        match base {
            "record" => {
                // For records, we need to build a List from all non-skipped fields
                let record_fields: Vec<_> = fields.iter().filter(|f| !f.attrs.skip).collect();
                let field_names: Vec<String> =
                    record_fields.iter().map(|f| f.ident.to_string()).collect();
                let field_idents: Vec<_> = record_fields.iter().map(|f| &f.ident).collect();

                quote! {
                    impl #impl_generics ::miniextendr_api::vctrs::IntoVctrs for #name #ty_generics #where_clause {
                        fn into_vctrs(self) -> Result<::miniextendr_api::ffi::SEXP, ::miniextendr_api::vctrs::VctrsBuildError> {
                            use ::miniextendr_api::IntoR;

                            // Get attrs before moving fields out of self
                            let attrs = self.attrs();

                            // Build the fields list from pairs
                            let pairs: Vec<(&str, ::miniextendr_api::ffi::SEXP)> = vec![
                                #( (#field_names, self.#field_idents.into_sexp()), )*
                            ];
                            let fields = ::miniextendr_api::list::List::from_raw_pairs(pairs);

                            ::miniextendr_api::vctrs::new_rcrd(
                                fields,
                                &[Self::CLASS_NAME],
                                &attrs,
                            )
                        }
                    }
                }
            }
            _ => {
                // For simple vctrs (double, integer, etc.)
                quote! {
                    impl #impl_generics ::miniextendr_api::vctrs::IntoVctrs for #name #ty_generics #where_clause {
                        fn into_vctrs(self) -> Result<::miniextendr_api::ffi::SEXP, ::miniextendr_api::vctrs::VctrsBuildError> {
                            use ::miniextendr_api::IntoR;

                            // Get attrs before moving data out of self
                            let attrs = self.attrs();
                            let data = self.#data_ident.into_sexp();
                            ::miniextendr_api::vctrs::new_vctr(
                                data,
                                &[Self::CLASS_NAME],
                                &attrs,
                                Some(Self::INHERIT_BASE_TYPE),
                            )
                        }
                    }
                }
            }
        }
    } else {
        TokenStream::new()
    };

    // Generate VctrsRecord implementation if base is "record"
    let record_impl = if base == "record" {
        let field_names: Vec<String> = fields
            .iter()
            .filter(|f| !f.attrs.skip)
            .map(|f| f.ident.to_string())
            .collect();
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

    // Generate R wrapper code for vctrs S3 methods
    let record_field_names: Vec<String> = if base == "record" {
        fields
            .iter()
            .filter(|f| !f.attrs.skip)
            .map(|f| f.ident.to_string())
            .collect()
    } else {
        Vec::new()
    };
    let r_wrappers = generate_r_wrappers(
        &class_name,
        base,
        attrs.abbr.as_deref(),
        &record_field_names,
    );

    // Generate the R_WRAPPERS_VCTRS_{TYPE} const
    let name_upper = name.to_string().to_uppercase();
    let r_wrappers_const_ident = Ident::new(
        &format!("R_WRAPPERS_VCTRS_{}", name_upper),
        Span::call_site(),
    );

    Ok(quote! {
        #vctrs_class_impl
        #record_impl
        #into_vctrs_impl

        /// Generated R wrapper code for vctrs S3 methods.
        #[doc(hidden)]
        pub const #r_wrappers_const_ident: &str = #r_wrappers;
    })
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
    fn test_simple_vctr_with_data_field() {
        let input: DeriveInput = syn::parse_quote! {
            #[vctrs(class = "percent", base = "double")]
            struct Percent {
                #[vctrs(data)]
                data: Vec<f64>,
            }
        };

        let result = derive_vctrs(input).unwrap();
        let code = result.to_string();

        // Should generate VctrsClass
        assert!(code.contains("VctrsClass"));
        // Should generate IntoVctrs with data field
        assert!(code.contains("IntoVctrs"));
        assert!(code.contains("self . data"));
        assert!(code.contains("new_vctr"));
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
    fn test_record_vctr_with_data() {
        let input: DeriveInput = syn::parse_quote! {
            #[vctrs(class = "rational", base = "record")]
            struct Rational {
                #[vctrs(data)]
                n: Vec<i32>,
                d: Vec<i32>,
            }
        };

        let result = derive_vctrs(input).unwrap();
        let code = result.to_string();

        // Should generate all three traits
        assert!(code.contains("VctrsClass"));
        assert!(code.contains("VctrsRecord"));
        assert!(code.contains("IntoVctrs"));
        // Record uses new_rcrd
        assert!(code.contains("new_rcrd"));
        // Field names should be included
        assert!(code.contains("\"n\""));
        assert!(code.contains("\"d\""));
    }

    #[test]
    fn test_skip_field() {
        let input: DeriveInput = syn::parse_quote! {
            #[vctrs(class = "rational", base = "record")]
            struct Rational {
                #[vctrs(data)]
                n: Vec<i32>,
                d: Vec<i32>,
                #[vctrs(skip)]
                cached: Option<f64>,
            }
        };

        let result = derive_vctrs(input).unwrap();
        let code = result.to_string();

        // Field names should NOT include cached
        assert!(code.contains("\"n\""));
        assert!(code.contains("\"d\""));
        assert!(!code.contains("\"cached\""));
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
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("requires #[vctrs(class")
        );
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
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("can only be applied to structs")
        );
    }

    #[test]
    fn test_r_wrappers_const_generated() {
        let input: DeriveInput = syn::parse_quote! {
            #[vctrs(class = "percent", base = "double")]
            struct Percent {
                data: Vec<f64>,
            }
        };

        let result = derive_vctrs(input).unwrap();
        let code = result.to_string();

        // Should generate R_WRAPPERS_VCTRS_PERCENT const
        assert!(code.contains("R_WRAPPERS_VCTRS_PERCENT"));
        assert!(code.contains("pub const"));
    }

    #[test]
    fn test_r_wrappers_content_simple() {
        let r_code = generate_r_wrappers("percent", "double", Some("pct"), &[]);

        // Should have format method
        assert!(r_code.contains("format.percent"));
        assert!(r_code.contains("vctrs::vec_data"));

        // Should have vec_ptype_abbr since abbr provided
        assert!(r_code.contains("vec_ptype_abbr.percent"));
        assert!(r_code.contains("\"pct\""));

        // Should have vec_ptype_full
        assert!(r_code.contains("vec_ptype_full.percent"));
    }

    #[test]
    fn test_r_wrappers_content_record() {
        let r_code = generate_r_wrappers(
            "rational",
            "record",
            None,
            &["n".to_string(), "d".to_string()],
        );

        // Should have format method with vctrs::field accessors
        assert!(r_code.contains("format.rational"));
        assert!(r_code.contains("vctrs::field(x, \"n\")"));
        assert!(r_code.contains("vctrs::field(x, \"d\")"));

        // Should NOT have vec_ptype_abbr since no abbr
        assert!(!r_code.contains("vec_ptype_abbr.rational"));

        // Should have vec_ptype_full
        assert!(r_code.contains("vec_ptype_full.rational"));
    }

    #[test]
    fn test_r_wrappers_no_abbr() {
        let r_code = generate_r_wrappers("mytype", "integer", None, &[]);

        // Should NOT have vec_ptype_abbr
        assert!(!r_code.contains("vec_ptype_abbr.mytype"));

        // Should still have format and vec_ptype_full
        assert!(r_code.contains("format.mytype"));
        assert!(r_code.contains("vec_ptype_full.mytype"));
    }
}
