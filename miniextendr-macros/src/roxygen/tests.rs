use super::*;

// =============================================================================
// Doc-lint feature tests (implicit title/description extraction)
// =============================================================================

#[cfg(feature = "doc-lint")]
mod doc_lint_tests {
    use super::*;

    /// Helper to create doc attributes from lines (simulates `/// line1`, `/// line2`, etc.)
    fn make_doc_attrs(lines: &[&str]) -> Vec<syn::Attribute> {
        lines
            .iter()
            .map(|line| syn::parse_quote!(#[doc = #line]))
            .collect()
    }

    #[test]
    fn test_implicit_title_simple() {
        let attrs = make_doc_attrs(&["Simple title"]);
        assert_eq!(
            implicit_title_from_attrs(&attrs),
            Some("Simple title".to_string())
        );
    }

    #[test]
    fn test_implicit_title_with_period() {
        // Title ends at first period
        let attrs = make_doc_attrs(&["This is the title. This is description."]);
        assert_eq!(
            implicit_title_from_attrs(&attrs),
            Some("This is the title".to_string())
        );
    }

    #[test]
    fn test_implicit_title_trailing_period_stripped() {
        let attrs = make_doc_attrs(&["Title with trailing period."]);
        assert_eq!(
            implicit_title_from_attrs(&attrs),
            Some("Title with trailing period".to_string())
        );
    }

    #[test]
    fn test_implicit_title_multiline_before_blank() {
        // Title spans multiple lines until blank line
        let attrs = make_doc_attrs(&[
            "First part of title",
            "second part of title",
            "",
            "Description",
        ]);
        assert_eq!(
            implicit_title_from_attrs(&attrs),
            Some("First part of title second part of title".to_string())
        );
    }

    #[test]
    fn test_implicit_title_none_when_starts_with_tag() {
        let attrs = make_doc_attrs(&["@param x A parameter"]);
        assert_eq!(implicit_title_from_attrs(&attrs), None);
    }

    #[test]
    fn test_implicit_title_empty_docs() {
        let attrs: Vec<syn::Attribute> = vec![];
        assert_eq!(implicit_title_from_attrs(&attrs), None);
    }

    #[test]
    fn test_implicit_description_single_paragraph() {
        let attrs = make_doc_attrs(&["This is the description."]);
        assert_eq!(
            implicit_description_from_attrs(&attrs),
            Some("This is the description.".to_string())
        );
    }

    #[test]
    fn test_implicit_description_multiline_paragraph() {
        let attrs = make_doc_attrs(&["First line of description.", "Second line of description."]);
        assert_eq!(
            implicit_description_from_attrs(&attrs),
            Some("First line of description. Second line of description.".to_string())
        );
    }

    #[test]
    fn test_implicit_description_stops_at_blank_line() {
        let attrs = make_doc_attrs(&[
            "This is description.",
            "",
            "This is details (not description).",
        ]);
        assert_eq!(
            implicit_description_from_attrs(&attrs),
            Some("This is description.".to_string())
        );
    }

    #[test]
    fn test_implicit_description_none_when_starts_with_tag() {
        let attrs = make_doc_attrs(&["@title Explicit title", "@description Explicit desc"]);
        assert_eq!(implicit_description_from_attrs(&attrs), None);
    }
}

// =============================================================================
// Tag extraction tests
// =============================================================================

#[test]
fn test_format_single_line_tags() {
    let tags = vec![
        "@param x An input".to_string(),
        "@return Output".to_string(),
    ];
    let formatted = format_roxygen_tags(&tags);
    assert_eq!(formatted, "#' @param x An input\n#' @return Output\n");
}

#[test]
fn test_format_multiline_tag() {
    // Simulates: @description First line\nSecond line
    let tags = vec!["@description First line\nSecond line".to_string()];
    let formatted = format_roxygen_tags(&tags);
    assert_eq!(formatted, "#' @description First line\n#' Second line\n");
}

#[test]
fn test_push_multiline_tag() {
    let tags = vec!["@description Line one\nLine two\nLine three".to_string()];
    let mut lines = Vec::new();
    push_roxygen_tags(&mut lines, &tags);
    assert_eq!(
        lines,
        vec!["#' @description Line one", "#' Line two", "#' Line three"]
    );
}

#[test]
fn test_has_roxygen_tag_multiline() {
    // Tag name detection should work even with multiline content
    let tags = vec!["@description First\nSecond".to_string()];
    assert!(has_roxygen_tag(&tags, "description"));
    assert!(!has_roxygen_tag(&tags, "param"));
}

#[test]
fn test_find_tag_value() {
    let tags = vec![
        "@title My Title".to_string(),
        "@description A longer description".to_string(),
        "@param x An input".to_string(),
    ];
    assert_eq!(find_tag_value(&tags, "title"), Some("My Title"));
    assert_eq!(
        find_tag_value(&tags, "description"),
        Some("A longer description")
    );
    assert_eq!(find_tag_value(&tags, "param"), Some("x An input"));
    assert_eq!(find_tag_value(&tags, "return"), None);
}

#[test]
fn test_normalize_for_comparison() {
    // Basic normalization
    assert_eq!(normalize_for_comparison("Hello World"), "hello world");
    // Collapse whitespace
    assert_eq!(normalize_for_comparison("Hello    World"), "hello world");
    // Strip trailing punctuation
    assert_eq!(normalize_for_comparison("Hello World."), "hello world");
    assert_eq!(normalize_for_comparison("Hello World!"), "hello world");
    // Combined
    assert_eq!(
        normalize_for_comparison("  Hello    World.  "),
        "hello world"
    );
}
