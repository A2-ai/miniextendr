//! Shared utility functions for lint rule implementations.

use std::path::Path;

use syn::Attribute;

/// Returns true when the attribute list contains `#[miniextendr]`.
pub fn has_miniextendr_attr(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path()
            .segments
            .last()
            .is_some_and(|seg| seg.ident == "miniextendr")
    })
}

/// Extracts a displayable type name from an impl self type.
pub fn impl_type_name(ty: &syn::Type) -> Option<String> {
    match ty {
        syn::Type::Path(type_path) => type_path
            .path
            .segments
            .last()
            .map(|seg| seg.ident.to_string()),
        syn::Type::Reference(type_ref) => impl_type_name(&type_ref.elem),
        _ => None,
    }
}

/// Returns true if the attribute list contains `#[derive(ExternalPtr)]`
/// or `#[derive(miniextendr_api::ExternalPtr)]`.
pub fn has_external_ptr_derive(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if !attr.path().is_ident("derive") {
            return false;
        }
        let syn::Meta::List(meta_list) = &attr.meta else {
            return false;
        };
        let Ok(paths) = meta_list.parse_args_with(
            syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
        ) else {
            return false;
        };
        paths.iter().any(|p| {
            p.segments
                .last()
                .is_some_and(|seg| seg.ident == "ExternalPtr")
        })
    })
}

/// Returns true if the attribute list contains `#[derive(Altrep)]`
/// or `#[derive(miniextendr_api::Altrep)]`.
pub fn has_altrep_derive(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if !attr.path().is_ident("derive") {
            return false;
        }
        let syn::Meta::List(meta_list) = &attr.meta else {
            return false;
        };
        let Ok(paths) = meta_list.parse_args_with(
            syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
        ) else {
            return false;
        };
        paths
            .iter()
            .any(|p| p.segments.last().is_some_and(|seg| seg.ident == "Altrep"))
    })
}

/// Returns true if the attribute list contains `#[derive(Vctrs)]`
/// or `#[derive(miniextendr_api::Vctrs)]`.
pub fn has_vctrs_derive(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if !attr.path().is_ident("derive") {
            return false;
        }
        let syn::Meta::List(meta_list) = &attr.meta else {
            return false;
        };
        let Ok(paths) = meta_list.parse_args_with(
            syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
        ) else {
            return false;
        };
        paths
            .iter()
            .any(|p| p.segments.last().is_some_and(|seg| seg.ident == "Vctrs"))
    })
}

/// Returns whether a directory should be skipped during lint tree traversal.
pub fn should_skip_dir(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    matches!(name, "target" | "ra_target" | ".cargo" | ".git" | "vendor")
}

/// Parsed miniextendr attribute information for an impl block.
#[derive(Debug, Default)]
pub struct MiniextendrImplAttrs {
    /// Class system (e.g., "r6", "s3", "s4", "s7", or empty for env)
    pub class_system: Option<String>,
    /// Optional label for distinguishing multiple impl blocks of the same type
    pub label: Option<String>,
    /// Has `internal` flag
    pub internal: bool,
    /// Has `noexport` flag
    pub noexport: bool,
    /// Has `strict` flag
    pub strict: bool,
}

/// Parse the #[miniextendr(...)] attribute to extract class system, label, and flags.
pub fn parse_miniextendr_impl_attrs(attrs: &[Attribute]) -> MiniextendrImplAttrs {
    let mut result = MiniextendrImplAttrs::default();

    for attr in attrs {
        if attr
            .path()
            .segments
            .last()
            .is_none_or(|seg| seg.ident != "miniextendr")
        {
            continue;
        }

        if let syn::Meta::List(meta_list) = &attr.meta {
            let tokens = meta_list.tokens.to_string();
            let tokens = tokens.trim();
            if tokens.is_empty() {
                continue;
            }

            for part in tokens.split(',') {
                let part = part.trim();
                if part.is_empty() {
                    continue;
                }

                if part.starts_with("label") {
                    if let Some(eq_pos) = part.find('=') {
                        let value = part[eq_pos + 1..].trim();
                        let value = value.trim_matches('"').trim_matches('\'');
                        result.label = Some(value.to_string());
                    }
                } else if part == "internal" {
                    result.internal = true;
                } else if part == "noexport" {
                    result.noexport = true;
                } else if part == "strict" {
                    result.strict = true;
                } else if !part.contains('=') {
                    // Class system identifier (env, r6, s3, s4, s7)
                    result.class_system = Some(part.to_string());
                }
            }
        }
    }

    result
}

/// Extract `#[path = "..."]` attribute value from a module declaration.
pub fn extract_path_attr(attrs: &[Attribute]) -> Option<String> {
    attrs.iter().find_map(|attr| {
        if !attr.path().is_ident("path") {
            return None;
        }
        if let syn::Meta::NameValue(nv) = &attr.meta
            && let syn::Expr::Lit(expr_lit) = &nv.value
            && let syn::Lit::Str(lit_str) = &expr_lit.lit
        {
            return Some(lit_str.value());
        }
        None
    })
}

/// Extract `#[cfg(...)]` attributes as normalized token strings.
pub fn extract_cfg_attrs(attrs: &[Attribute]) -> Vec<String> {
    attrs
        .iter()
        .filter(|attr| attr.path().is_ident("cfg"))
        .map(|attr| attr.meta.to_token_stream_string())
        .collect()
}

/// Extract roxygen tags from doc-comment attributes.
///
/// Looks through `/// ...` comments for patterns like `@export`, `@noRd`,
/// `@keywords internal`, etc. Returns the tag names found.
pub fn extract_roxygen_tags(attrs: &[Attribute]) -> Vec<String> {
    let mut tags = Vec::new();
    for attr in attrs {
        if !attr.path().is_ident("doc") {
            continue;
        }
        if let syn::Meta::NameValue(nv) = &attr.meta
            && let syn::Expr::Lit(expr_lit) = &nv.value
            && let syn::Lit::Str(lit_str) = &expr_lit.lit
        {
            let text = lit_str.value();
            for word in text.split_whitespace() {
                if let Some(tag) = word.strip_prefix('@')
                    && !tag.is_empty()
                {
                    tags.push(tag.to_string());
                }
            }
        }
    }
    tags
}

/// Check if a struct with `#[miniextendr]` should be treated as ALTREP (needing `struct Name;` in module).
///
/// Returns true only for 1-field structs without explicit mode attrs (list, dataframe, externalptr).
/// Multi-field structs, structs with explicit mode attrs, and enums don't need module entries.
pub fn is_altrep_struct(item: &syn::ItemStruct) -> bool {
    let field_count = match &item.fields {
        syn::Fields::Named(f) => f.named.len(),
        syn::Fields::Unnamed(f) => f.unnamed.len(),
        syn::Fields::Unit => 0,
    };

    // Only 1-field structs are ALTREP candidates
    if field_count != 1 {
        return false;
    }

    // Check if #[miniextendr(...)] has mode attrs that override ALTREP
    for attr in &item.attrs {
        if attr
            .path()
            .segments
            .last()
            .is_none_or(|seg| seg.ident != "miniextendr")
        {
            continue;
        }

        if let syn::Meta::List(meta_list) = &attr.meta {
            let tokens = meta_list.tokens.to_string();
            for part in tokens.split(',') {
                let part = part.trim();
                // These mode attrs mean "not ALTREP"
                if matches!(part, "list" | "dataframe" | "externalptr") {
                    return false;
                }
            }
        }
    }

    true
}

/// Helper trait for converting Meta to a normalized string.
trait MetaToString {
    fn to_token_stream_string(&self) -> String;
}

impl MetaToString for syn::Meta {
    fn to_token_stream_string(&self) -> String {
        use quote::ToTokens;
        self.to_token_stream().to_string()
    }
}
