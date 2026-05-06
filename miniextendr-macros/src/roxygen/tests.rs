use super::*;

// region: Doc-lint feature tests (implicit title/description extraction)

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
    fn test_implicit_description_is_second_paragraph() {
        // First paragraph = title, second paragraph = description
        let attrs = make_doc_attrs(&["This is the title.", "", "This is the description."]);
        assert_eq!(
            implicit_description_from_attrs(&attrs),
            Some("This is the description.".to_string())
        );
    }

    #[test]
    fn test_implicit_description_multiline_second_paragraph() {
        let attrs = make_doc_attrs(&[
            "Title line.",
            "",
            "First line of description.",
            "Second line of description.",
        ]);
        assert_eq!(
            implicit_description_from_attrs(&attrs),
            Some("First line of description. Second line of description.".to_string())
        );
    }

    #[test]
    fn test_implicit_description_stops_at_third_paragraph() {
        let attrs = make_doc_attrs(&[
            "Title.",
            "",
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
    fn test_implicit_description_none_when_only_one_paragraph() {
        // Only a title, no second paragraph
        let attrs = make_doc_attrs(&["Just a title."]);
        assert_eq!(implicit_description_from_attrs(&attrs), None);
    }

    #[test]
    fn test_implicit_description_none_when_starts_with_tag() {
        let attrs = make_doc_attrs(&["@title Explicit title", "@description Explicit desc"]);
        assert_eq!(implicit_description_from_attrs(&attrs), None);
    }

    #[test]
    fn test_implicit_description_skips_multiple_blank_lines() {
        let attrs = make_doc_attrs(&["Title.", "", "", "Description after multiple blanks."]);
        assert_eq!(
            implicit_description_from_attrs(&attrs),
            Some("Description after multiple blanks.".to_string())
        );
    }

    // region: implicit_details_from_attrs unit tests

    #[test]
    fn test_implicit_details_none_when_one_paragraph() {
        let attrs = make_doc_attrs(&["Just a title."]);
        assert_eq!(implicit_details_from_attrs(&attrs), None);
    }

    #[test]
    fn test_implicit_details_none_when_two_paragraphs() {
        let attrs = make_doc_attrs(&["Title.", "", "Description only."]);
        assert_eq!(implicit_details_from_attrs(&attrs), None);
    }

    #[test]
    fn test_implicit_details_returns_third_paragraph() {
        let attrs = make_doc_attrs(&["Title.", "", "Description.", "", "Details paragraph."]);
        assert_eq!(
            implicit_details_from_attrs(&attrs),
            Some("Details paragraph.".to_string())
        );
    }

    #[test]
    fn test_implicit_details_joins_multiple_detail_paragraphs() {
        let attrs = make_doc_attrs(&[
            "Title.",
            "",
            "Description.",
            "",
            "First details para.",
            "",
            "Second details para.",
        ]);
        assert_eq!(
            implicit_details_from_attrs(&attrs),
            Some("First details para.\n\nSecond details para.".to_string())
        );
    }

    #[test]
    fn test_implicit_details_stops_at_tag() {
        let attrs = make_doc_attrs(&[
            "Title.",
            "",
            "Description.",
            "",
            "Details here.",
            "@param x Something",
        ]);
        assert_eq!(
            implicit_details_from_attrs(&attrs),
            Some("Details here.".to_string())
        );
    }

    #[test]
    fn test_implicit_details_multiline_detail_paragraph() {
        let attrs = make_doc_attrs(&[
            "Title.",
            "",
            "Description.",
            "",
            "First line of details.",
            "Second line of details.",
        ]);
        assert_eq!(
            implicit_details_from_attrs(&attrs),
            Some("First line of details. Second line of details.".to_string())
        );
    }

    // endregion
}
// endregion

// region: @details auto-injection integration tests (via roxygen_tags_from_attrs)

fn make_doc_attrs_plain(lines: &[&str]) -> Vec<syn::Attribute> {
    lines
        .iter()
        .map(|line| syn::parse_quote!(#[doc = #line]))
        .collect()
}

#[test]
fn auto_details_not_injected_for_one_paragraph() {
    // One paragraph + @param → no @details
    let attrs = make_doc_attrs_plain(&["Title.", "@param x A value"]);
    let tags = roxygen_tags_from_attrs(&attrs);
    assert!(!tags.iter().any(|t| t.starts_with("@details")));
}

#[test]
fn auto_details_not_injected_for_two_paragraphs() {
    // Two paragraphs + @param → @description injected but no @details
    let attrs = make_doc_attrs_plain(&["Title.", "", "Description.", "@param x A value"]);
    let tags = roxygen_tags_from_attrs(&attrs);
    assert!(tags.iter().any(|t| t.starts_with("@description")));
    assert!(!tags.iter().any(|t| t.starts_with("@details")));
}

#[test]
fn auto_details_injected_for_three_paragraphs() {
    // Three paragraphs + @param → both @description and @details injected
    let attrs = make_doc_attrs_plain(&[
        "Title.",
        "",
        "Description.",
        "",
        "Details paragraph.",
        "@param x A value",
    ]);
    let tags = roxygen_tags_from_attrs(&attrs);
    assert!(tags.iter().any(|t| t.starts_with("@description")));
    let details_tag = tags.iter().find(|t| t.starts_with("@details"));
    assert!(
        details_tag.is_some(),
        "expected @details tag in: {:?}",
        tags
    );
    assert!(details_tag.unwrap().contains("Details paragraph."));
}

#[test]
fn auto_details_joins_four_paragraphs() {
    // Four paragraphs → @details joins paragraphs 3 and 4 with \n\n
    let attrs = make_doc_attrs_plain(&[
        "Title.",
        "",
        "Description.",
        "",
        "First details.",
        "",
        "Second details.",
        "@param x A value",
    ]);
    let tags = roxygen_tags_from_attrs(&attrs);
    let details_tag = tags.iter().find(|t| t.starts_with("@details")).unwrap();
    assert!(
        details_tag.contains("First details."),
        "missing first: {:?}",
        tags
    );
    assert!(
        details_tag.contains("Second details."),
        "missing second: {:?}",
        tags
    );
}

#[test]
fn auto_details_idempotent_when_explicit_details_present() {
    // Explicit @details → no auto-injection
    let attrs = make_doc_attrs_plain(&[
        "Title.",
        "",
        "Description.",
        "",
        "This would become details.",
        "@details Explicit details.",
        "@param x A value",
    ]);
    let tags = roxygen_tags_from_attrs(&attrs);
    let count = tags.iter().filter(|t| t.starts_with("@details")).count();
    assert_eq!(count, 1, "expected exactly one @details tag: {:?}", tags);
    // It should be the explicit one
    let details_tag = tags.iter().find(|t| t.starts_with("@details")).unwrap();
    assert!(details_tag.contains("Explicit details."));
}

#[test]
fn auto_details_order_is_title_description_details_param() {
    // Verify order: @title before @description before @details before @param
    let attrs = make_doc_attrs_plain(&[
        "Title.",
        "",
        "Description.",
        "",
        "Details.",
        "@param x A value",
    ]);
    let tags = roxygen_tags_from_attrs(&attrs);
    let title_pos = tags.iter().position(|t| t.starts_with("@title"));
    let desc_pos = tags.iter().position(|t| t.starts_with("@description"));
    let details_pos = tags.iter().position(|t| t.starts_with("@details"));
    let param_pos = tags.iter().position(|t| t.starts_with("@param"));
    assert!(
        title_pos < desc_pos && desc_pos < details_pos && details_pos < param_pos,
        "expected title<desc<details<param order, got positions: title={:?} desc={:?} details={:?} param={:?}\ntags={:?}",
        title_pos,
        desc_pos,
        details_pos,
        param_pos,
        tags
    );
}

// endregion

// region: Tag extraction tests

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
// endregion

// region: has_roxygen_tag: single-word and multi-word matching

#[test]
fn test_has_roxygen_tag_single_word() {
    let tags = vec!["@export".to_string(), "@noRd".to_string()];
    assert!(has_roxygen_tag(&tags, "export"));
    assert!(has_roxygen_tag(&tags, "noRd"));
    assert!(!has_roxygen_tag(&tags, "param"));
}

#[test]
fn test_has_roxygen_tag_keywords_internal() {
    let tags = vec!["@keywords internal".to_string()];
    assert!(has_roxygen_tag(&tags, "keywords internal"));
    // Single-word "keywords" should also match
    assert!(has_roxygen_tag(&tags, "keywords"));
    assert!(!has_roxygen_tag(&tags, "internal"));
}

#[test]
fn test_has_roxygen_tag_keywords_internal_with_whitespace() {
    // Extra whitespace around the tag content
    let tags = vec!["  @keywords internal  ".to_string()];
    assert!(has_roxygen_tag(&tags, "keywords internal"));
}

#[test]
fn test_has_roxygen_tag_keywords_other() {
    // @keywords with a different value should not match "keywords internal"
    let tags = vec!["@keywords datasets".to_string()];
    assert!(has_roxygen_tag(&tags, "keywords"));
    assert!(!has_roxygen_tag(&tags, "keywords internal"));
}

#[test]
fn test_has_roxygen_tag_param_with_name() {
    // "param x" is a multi-word search that should match "@param x"
    let tags = vec!["@param x An input".to_string()];
    assert!(has_roxygen_tag(&tags, "param"));
    // Multi-word match: "param x An input" won't match "param x" because
    // the full content after @ is "param x An input", not "param x"
    assert!(!has_roxygen_tag(&tags, "param x"));
}
// endregion

// region: tag_names: extraction tests

#[test]
fn test_tag_names_extracts_first_word() {
    let tags = vec![
        "@param x Input".to_string(),
        "@return Output".to_string(),
        "@export".to_string(),
    ];
    let names = tag_names(&tags);
    assert!(names.contains("param"));
    assert!(names.contains("return"));
    assert!(names.contains("export"));
    assert!(!names.contains("x"));
}

#[test]
fn test_tag_names_ignores_non_tag_lines() {
    let tags = vec!["Just a comment".to_string(), "@title Real tag".to_string()];
    let names = tag_names(&tags);
    assert!(names.contains("title"));
    assert_eq!(names.len(), 1);
}

#[test]
fn test_tag_names_handles_leading_whitespace() {
    let tags = vec!["  @export".to_string()];
    let names = tag_names(&tags);
    assert!(names.contains("export"));
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

// region: strip_method_tags — impl-block roxygen tag filtering

#[test]
fn strip_method_tags_drops_param_return_examples_export() {
    let tags: Vec<String> = [
        "@param x an input",
        "@return something",
        "@returns another thing",
        "@examples f()",
        "@export",
        "@title Keep me",
        "@name keep_me",
        "@description Keep this too",
    ]
    .iter()
    .map(|s| (*s).to_string())
    .collect();

    let span = proc_macro2::Span::call_site();
    let (kept, _warnings) = strip_method_tags(&tags, "MyType", span);
    assert_eq!(
        kept,
        vec![
            "@title Keep me".to_string(),
            "@name keep_me".to_string(),
            "@description Keep this too".to_string(),
        ]
    );
}

#[test]
fn strip_method_tags_preserves_export_variants() {
    // @exportClass / @exportMethod / @exportPattern are valid on class-level
    // docs (S4). Only the bare @export tag should be stripped.
    let tags: Vec<String> = [
        "@export",
        "@exportClass MyS4Class",
        "@exportMethod show",
        "@exportPattern ^foo",
    ]
    .iter()
    .map(|s| (*s).to_string())
    .collect();

    let (kept, _warnings) = strip_method_tags(&tags, "MyType", proc_macro2::Span::call_site());
    assert_eq!(
        kept,
        vec![
            "@exportClass MyS4Class".to_string(),
            "@exportMethod show".to_string(),
            "@exportPattern ^foo".to_string(),
        ]
    );
}

#[test]
fn strip_method_tags_leaves_prose_untouched() {
    let tags: Vec<String> = ["This is prose.", "@title A title", "More prose"]
        .iter()
        .map(|s| (*s).to_string())
        .collect();

    let (kept, _warnings) = strip_method_tags(&tags, "MyType", proc_macro2::Span::call_site());
    assert_eq!(kept, tags);
}

// endregion

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
// endregion
