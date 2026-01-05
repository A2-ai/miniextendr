//! # `#[derive(RFactor)]` - Enum ↔ R Factor Support
//!
//! This module implements the `#[derive(RFactor)]` macro which generates
//! the `RFactor` trait implementation for C-style enums, enabling automatic
//! conversion between Rust enums and R factors.
//!
//! ## Usage
//!
//! ```ignore
//! #[derive(Copy, Clone, RFactor)]
//! enum Color {
//!     Red,
//!     Green,
//!     Blue,
//! }
//!
//! // Generates impl RFactor for Color, IntoR for Color, TryFromSexp for Color,
//! // and Vec/Option variants.
//! ```
//!
//! ## Attributes
//!
//! - `#[r_factor(rename = "name")]` - Rename a variant's level string
//! - `#[r_factor(rename_all = "snake_case")]` - Rename all variants (snake_case, kebab-case, lower, upper)
//!
//! ```ignore
//! #[derive(Copy, Clone, RFactor)]
//! #[r_factor(rename_all = "snake_case")]
//! enum Status {
//!     InProgress,  // level: "in_progress"
//!     #[r_factor(rename = "done")]
//!     Completed,   // level: "done"
//! }
//! ```

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields};

/// Convert PascalCase to snake_case.
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

/// Convert PascalCase to kebab-case.
fn to_kebab_case(s: &str) -> String {
    to_snake_case(s).replace('_', "-")
}

/// Apply rename_all transformation.
fn apply_rename_all(name: &str, rename_all: Option<&str>) -> String {
    match rename_all {
        Some("snake_case") => to_snake_case(name),
        Some("kebab-case") => to_kebab_case(name),
        Some("lower") => name.to_lowercase(),
        Some("upper") => name.to_uppercase(),
        _ => name.to_string(),
    }
}

/// Parse r_factor attributes from an enum or variant.
fn parse_r_factor_attrs(attrs: &[syn::Attribute]) -> syn::Result<(Option<String>, Option<String>)> {
    let mut rename = None;
    let mut rename_all = None;

    for attr in attrs {
        if attr.path().is_ident("r_factor") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("rename") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    rename = Some(value.value());
                } else if meta.path.is_ident("rename_all") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    rename_all = Some(value.value());
                } else {
                    return Err(meta.error("unknown r_factor attribute"));
                }
                Ok(())
            })?;
        }
    }

    Ok((rename, rename_all))
}

/// Generate the RFactor derive implementation.
pub fn derive_r_factor(input: DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Parse enum-level attributes
    let (_, rename_all) = parse_r_factor_attrs(&input.attrs)?;

    // Get enum variants
    let variants = match &input.data {
        Data::Enum(data) => &data.variants,
        Data::Struct(_) => {
            return Err(syn::Error::new_spanned(
                &input,
                "#[derive(RFactor)] can only be applied to enums",
            ));
        }
        Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                &input,
                "#[derive(RFactor)] can only be applied to enums",
            ));
        }
    };

    // Validate variants are fieldless and collect level names
    let mut level_names = Vec::new();
    let mut variant_idents = Vec::new();

    for variant in variants {
        // Check for fields (only allow unit variants)
        if !matches!(variant.fields, Fields::Unit) {
            return Err(syn::Error::new_spanned(
                variant,
                "#[derive(RFactor)] only supports fieldless (C-style) enum variants",
            ));
        }

        // Parse variant-level attributes
        let (rename, _) = parse_r_factor_attrs(&variant.attrs)?;

        // Determine level name
        let level_name = if let Some(r) = rename {
            r
        } else {
            apply_rename_all(&variant.ident.to_string(), rename_all.as_deref())
        };

        level_names.push(level_name);
        variant_idents.push(&variant.ident);
    }

    // Generate indices (1-based for R)
    let indices: Vec<i32> = (1..=variant_idents.len() as i32).collect();

    // Generate the implementation
    let level_name_strs: Vec<&str> = level_names.iter().map(|s| s.as_str()).collect();

    // Generate impl for the enum itself. Vec/Option wrappers cannot be generated
    // due to Rust's orphan rules - neither IntoR nor Vec is local to the user's crate.
    // For Vec conversions, users must use the helper functions:
    // - factor_vec_to_sexp(&[T])
    // - factor_vec_from_sexp::<T>(SEXP)
    // - factor_option_vec_to_sexp(&[Option<T>])
    // - factor_option_vec_from_sexp::<T>(SEXP)

    Ok(quote! {
        impl #impl_generics ::miniextendr_api::RFactor for #name #ty_generics #where_clause {
            const LEVELS: &'static [&'static str] = &[#(#level_name_strs),*];

            fn to_level_index(self) -> i32 {
                match self {
                    #(Self::#variant_idents => #indices,)*
                }
            }

            fn from_level_index(idx: i32) -> Option<Self> {
                match idx {
                    #(#indices => Some(Self::#variant_idents),)*
                    _ => None,
                }
            }
        }

        impl #impl_generics ::miniextendr_api::IntoR for #name #ty_generics #where_clause {
            fn into_sexp(self) -> ::miniextendr_api::ffi::SEXP {
                ::miniextendr_api::factor_to_sexp(self)
            }
        }

        impl #impl_generics ::miniextendr_api::TryFromSexp for #name #ty_generics #where_clause {
            type Error = ::miniextendr_api::SexpError;

            fn try_from_sexp(sexp: ::miniextendr_api::ffi::SEXP) -> Result<Self, Self::Error> {
                ::miniextendr_api::factor_from_sexp(sexp)
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snake_case() {
        assert_eq!(to_snake_case("HelloWorld"), "hello_world");
        assert_eq!(to_snake_case("InProgress"), "in_progress");
        assert_eq!(to_snake_case("ABC"), "a_b_c");
        assert_eq!(to_snake_case("Red"), "red");
    }

    #[test]
    fn test_kebab_case() {
        assert_eq!(to_kebab_case("HelloWorld"), "hello-world");
        assert_eq!(to_kebab_case("InProgress"), "in-progress");
    }

    #[test]
    fn test_apply_rename_all() {
        assert_eq!(
            apply_rename_all("HelloWorld", Some("snake_case")),
            "hello_world"
        );
        assert_eq!(
            apply_rename_all("HelloWorld", Some("kebab-case")),
            "hello-world"
        );
        assert_eq!(apply_rename_all("HelloWorld", Some("lower")), "helloworld");
        assert_eq!(apply_rename_all("HelloWorld", Some("upper")), "HELLOWORLD");
        assert_eq!(apply_rename_all("HelloWorld", None), "HelloWorld");
    }
}
