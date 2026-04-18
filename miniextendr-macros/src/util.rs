//! Small cross-cutting helpers used by multiple macro modules.

/// Extract `#[cfg(...)]` attributes from a list of attributes.
///
/// These should be propagated to generated items so they are conditionally
/// compiled along with the original function.
pub(crate) fn extract_cfg_attrs(attrs: &[syn::Attribute]) -> Vec<syn::Attribute> {
    attrs
        .iter()
        .filter(|attr| attr.path().is_ident("cfg"))
        .cloned()
        .collect()
}

/// Format a human-readable source location note from a syntax span.
///
/// Column is reported as 1-based for consistency with editor displays.
pub(crate) fn source_location_doc(span: proc_macro2::Span) -> String {
    let start = span.start();
    format!(
        "Generated from source location line {}, column {}.",
        start.line,
        start.column + 1
    )
}

/// Build a `TokenStream` containing a raw string literal from an R wrapper string.
pub(crate) fn r_wrapper_raw_literal(s: &str) -> proc_macro2::TokenStream {
    use std::str::FromStr;
    let raw = format!("r#\"\n{}\n\"#", s);
    proc_macro2::TokenStream::from_str(&raw).expect("valid raw string literal")
}
