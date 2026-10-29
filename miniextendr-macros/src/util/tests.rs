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
