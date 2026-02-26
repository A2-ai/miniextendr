//! # `#[derive(MatchArg)]` - Enum ↔ R String with `match.arg` Support
//!
//! This module implements the `#[derive(MatchArg)]` macro which generates
//! the `MatchArg` trait implementation for C-style enums, enabling automatic
//! conversion between Rust enums and R character strings with partial matching.
//!
//! ## Usage
//!
//! ```ignore
//! #[derive(Copy, Clone, MatchArg)]
//! enum Mode {
//!     Fast,
//!     Safe,
//!     Debug,
//! }
//!
//! // Generates impl MatchArg for Mode, TryFromSexp for Mode, IntoR for Mode.
//! ```
//!
//! ## Attributes
//!
//! - `#[match_arg(rename = "name")]` - Rename a variant's choice string
//! - `#[match_arg(rename_all = "snake_case")]` - Rename all variants (snake_case, kebab-case, lower, upper)

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields};

/// Parsed match_arg attributes.
#[derive(Default)]
struct MatchArgAttrs {
    rename: Option<String>,
    rename_all: Option<String>,
}

/// Parse match_arg attributes from an enum or variant.
fn parse_match_arg_attrs(attrs: &[syn::Attribute]) -> syn::Result<MatchArgAttrs> {
    let mut result = MatchArgAttrs::default();

    for attr in attrs {
        if attr.path().is_ident("match_arg") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("rename") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    result.rename = Some(value.value());
                } else if meta.path.is_ident("rename_all") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    let val = value.value();
                    match val.as_str() {
                        "snake_case" | "kebab-case" | "lower" | "upper" => {}
                        _ => {
                            return Err(meta.error(
                                "unsupported rename_all value; expected one of: \
                                 snake_case, kebab-case, lower, upper",
                            ));
                        }
                    }
                    result.rename_all = Some(val);
                } else {
                    return Err(meta
                        .error("unknown match_arg attribute; expected `rename` or `rename_all`"));
                }
                Ok(())
            })?;
        }
    }

    Ok(result)
}

// Reuse the case conversion from factor_derive (same logic).
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

fn to_kebab_case(s: &str) -> String {
    to_snake_case(s).replace('_', "-")
}

fn apply_rename_all(name: &str, rename_all: Option<&str>) -> String {
    match rename_all {
        Some("snake_case") => to_snake_case(name),
        Some("kebab-case") => to_kebab_case(name),
        Some("lower") => name.to_lowercase(),
        Some("upper") => name.to_uppercase(),
        _ => name.to_string(),
    }
}

/// Generate the MatchArg derive implementation.
pub fn derive_match_arg(input: DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Reject generics for v1
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.generics,
            "#[derive(MatchArg)] does not support generic enums",
        ));
    }

    // Parse enum-level attributes
    let attrs = parse_match_arg_attrs(&input.attrs)?;

    // Get enum variants
    let variants = match &input.data {
        Data::Enum(data) => &data.variants,
        Data::Struct(_) => {
            return Err(syn::Error::new_spanned(
                &input,
                "#[derive(MatchArg)] can only be applied to enums",
            ));
        }
        Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                &input,
                "#[derive(MatchArg)] can only be applied to enums",
            ));
        }
    };

    if variants.is_empty() {
        return Err(syn::Error::new_spanned(
            &input,
            "#[derive(MatchArg)] requires at least one variant",
        ));
    }

    let mut choice_names = Vec::new();
    let mut variant_idents = Vec::new();

    for variant in variants {
        // Only allow unit variants (fieldless)
        if !matches!(variant.fields, Fields::Unit) {
            return Err(syn::Error::new_spanned(
                variant,
                "#[derive(MatchArg)] only supports fieldless (C-style) enum variants",
            ));
        }

        // Parse variant-level attributes
        let var_attrs = parse_match_arg_attrs(&variant.attrs)?;

        // Determine choice name
        let choice_name = if let Some(r) = var_attrs.rename {
            r
        } else {
            apply_rename_all(&variant.ident.to_string(), attrs.rename_all.as_deref())
        };

        choice_names.push(choice_name);
        variant_idents.push(&variant.ident);
    }

    // Check for duplicate choice names
    {
        let mut seen = std::collections::HashSet::new();
        for (i, name) in choice_names.iter().enumerate() {
            if !seen.insert(name.as_str()) {
                return Err(syn::Error::new_spanned(
                    &variants.iter().nth(i).unwrap().ident,
                    format!("duplicate choice name {:?} in #[derive(MatchArg)]", name),
                ));
            }
        }
    }

    let choice_strs: Vec<&str> = choice_names.iter().map(|s| s.as_str()).collect();

    Ok(quote! {
        impl #impl_generics ::miniextendr_api::match_arg::MatchArg for #name #ty_generics #where_clause {
            const CHOICES: &'static [&'static str] = &[#(#choice_strs),*];

            fn from_choice(choice: &str) -> Option<Self> {
                match choice {
                    #(#choice_strs => Some(Self::#variant_idents),)*
                    _ => None,
                }
            }

            fn to_choice(self) -> &'static str {
                match self {
                    #(Self::#variant_idents => #choice_strs,)*
                }
            }
        }

        impl #impl_generics ::miniextendr_api::TryFromSexp for #name #ty_generics #where_clause {
            type Error = ::miniextendr_api::SexpError;

            fn try_from_sexp(sexp: ::miniextendr_api::ffi::SEXP) -> Result<Self, Self::Error> {
                ::miniextendr_api::match_arg_from_sexp(sexp).map_err(Into::into)
            }
        }

        impl #impl_generics ::miniextendr_api::IntoR for #name #ty_generics #where_clause {
            fn into_sexp(self) -> ::miniextendr_api::ffi::SEXP {
                use ::miniextendr_api::match_arg::MatchArg;
                self.to_choice().into_sexp()
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_derive() {
        let input: DeriveInput = syn::parse_quote! {
            enum Mode {
                Fast,
                Safe,
                Debug,
            }
        };

        let result = derive_match_arg(input).unwrap();
        let code = result.to_string();
        assert!(code.contains("Fast"));
        assert!(code.contains("Safe"));
        assert!(code.contains("Debug"));
        assert!(code.contains("CHOICES"));
        assert!(code.contains("from_choice"));
        assert!(code.contains("to_choice"));
    }

    #[test]
    fn test_rename_all() {
        let input: DeriveInput = syn::parse_quote! {
            #[match_arg(rename_all = "snake_case")]
            enum Mode {
                FastMode,
                SafeMode,
            }
        };

        let result = derive_match_arg(input).unwrap();
        let code = result.to_string();
        assert!(code.contains("fast_mode"));
        assert!(code.contains("safe_mode"));
    }

    #[test]
    fn test_rename_variant() {
        let input: DeriveInput = syn::parse_quote! {
            enum Priority {
                #[match_arg(rename = "lo")]
                Low,
                #[match_arg(rename = "hi")]
                High,
            }
        };

        let result = derive_match_arg(input).unwrap();
        let code = result.to_string();
        assert!(code.contains("\"lo\""));
        assert!(code.contains("\"hi\""));
    }

    #[test]
    fn test_reject_fields() {
        let input: DeriveInput = syn::parse_quote! {
            enum Bad {
                A(i32),
            }
        };

        let result = derive_match_arg(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_reject_struct() {
        let input: DeriveInput = syn::parse_quote! {
            struct Bad;
        };

        let result = derive_match_arg(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_reject_empty() {
        let input: DeriveInput = syn::parse_quote! {
            enum Empty {}
        };

        let result = derive_match_arg(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_duplicate_choice_names() {
        let input: DeriveInput = syn::parse_quote! {
            enum Dup {
                #[match_arg(rename = "same")]
                A,
                #[match_arg(rename = "same")]
                B,
            }
        };

        let result = derive_match_arg(input);
        assert!(result.is_err());
    }
}
