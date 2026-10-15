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
///
/// Every generated wrapper is finalized here, so this is also where calls a
/// formal would shadow are qualified: `formals` holds every R formal of the
/// wrappers in `s`, and each call to a base function that shares a name with
/// one of them is written `base::name(...)`
/// ([`qualify_shadowed_calls`](crate::r_shadowing::qualify_shadowed_calls)).
///
/// The delimiter has one more `#` than the longest run of `#` after a `"` in
/// the wrapper, so a doc comment quoting `"# a footer"` or a default of `"#"`
/// can't end the literal early (#2111).
pub(crate) fn r_wrapper_raw_literal(
    s: &str,
    formals: &std::collections::BTreeSet<String>,
) -> proc_macro2::TokenStream {
    use std::str::FromStr;
    let s = crate::r_shadowing::qualify_shadowed_calls(s, formals);
    let hashes = "#".repeat(raw_string_hashes(&s));
    let raw = format!("r{hashes}\"\n{s}\n\"{hashes}");
    proc_macro2::TokenStream::from_str(&raw).expect("valid raw string literal")
}

/// How many `#` a raw string literal holding `s` needs: one more than the
/// longest run of `#` that follows a `"` in `s`.
fn raw_string_hashes(s: &str) -> usize {
    let longest = s
        .match_indices('"')
        .map(|(i, _)| s[i + 1..].bytes().take_while(|&b| b == b'#').count())
        .max()
        .unwrap_or(0);
    longest + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The literal parses back to the wrapper text, newline-framed, however
    /// many `#` follow a quote in it (#2111).
    #[test]
    fn raw_literal_holds_quote_hash_runs() {
        let formals = std::collections::BTreeSet::new();
        for text in [
            "plain",
            "#' the last line reads \"# Not shown: sd\".",
            "f <- function(sep = \"#\") sep",
            "\"##\" and \"###x\"",
            "ends with a quote\"",
        ] {
            let literal: syn::LitStr = syn::parse2(r_wrapper_raw_literal(text, &formals))
                .unwrap_or_else(|e| panic!("{text:?}: {e}"));
            assert_eq!(literal.value(), format!("\n{text}\n"));
        }
    }

    #[test]
    fn raw_string_hashes_counts_runs_after_quotes() {
        assert_eq!(raw_string_hashes("no quotes ##"), 1);
        assert_eq!(raw_string_hashes("\"x\""), 1);
        assert_eq!(raw_string_hashes("\"#"), 2);
        assert_eq!(raw_string_hashes("\"## then \"#"), 3);
    }
}
