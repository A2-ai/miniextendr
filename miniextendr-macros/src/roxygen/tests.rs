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

    // region: #1172 — lint must only fire on author-written @description

    /// Multi-paragraph prose with no explicit tags must not warn. Before the
    /// fix, `doc_conflict_warnings` saw the `@description` synthesized from
    /// leading prose by `roxygen_tags_from_attrs` and compared it against the
    /// second paragraph — a guaranteed mismatch for every multi-paragraph doc.
    #[test]
    fn test_no_desc_warning_for_prose_only_docs() {
        let attrs = make_doc_attrs(&[
            "Create a lazy integer sequence ALTREP.",
            "",
            "Elements are computed on demand.",
            "@param n Length.",
        ]);
        let warnings = doc_conflict_warnings(&attrs, proc_macro2::Span::call_site());
        assert!(
            warnings.is_empty(),
            "prose-only docs must not trigger the @description lint: {warnings}"
        );
    }

    #[test]
    fn test_desc_warning_fires_on_drifted_explicit_description() {
        let attrs = make_doc_attrs(&[
            "Title.",
            "",
            "Second paragraph.",
            "@description Something entirely different.",
        ]);
        let warnings = doc_conflict_warnings(&attrs, proc_macro2::Span::call_site());
        assert!(
            warnings.to_string().contains("MINIEXTENDR_DOC_LINT_DESC"),
            "drifted explicit @description must still warn: {warnings}"
        );
    }

    #[test]
    fn test_no_desc_warning_when_explicit_matches_second_paragraph() {
        let attrs = make_doc_attrs(&[
            "Title.",
            "",
            "Second paragraph.",
            "@description Second paragraph.",
        ]);
        let warnings = doc_conflict_warnings(&attrs, proc_macro2::Span::call_site());
        assert!(
            warnings.is_empty(),
            "matching explicit @description must not warn: {warnings}"
        );
    }

    // endregion
}
// endregion

// region: #613 — doc attrs partitioned before non-doc attrs (Option B from #586)
//
// With partition-based normalization, all doc attrs are collected and processed
// contiguously first (stable order within the doc group). Non-doc attrs like
// #[cfg(...)] are skipped entirely for roxygen processing.  This means doc
// content that was separated by a #[cfg(...)] in source order now correctly
// continues multiline tags.

#[test]
fn attr_interrupt_cfg_between_examples_lines_doc_continues() {
    // Simulates: /// @examples\n/// ex()\n  #[cfg(feature="x")]\n/// more_ex()
    // With partition, all doc attrs are processed contiguously, so more_ex()
    // DOES continue the @examples block (correct behaviour).
    let attrs: Vec<syn::Attribute> = vec![
        syn::parse_quote!(#[doc = "@examples"]),
        syn::parse_quote!(#[doc = "ex()"]),
        syn::parse_quote!(#[cfg(feature = "x")]),
        syn::parse_quote!(#[doc = "more_ex()"]),
    ];
    let tags = roxygen_tags_from_attrs(&attrs);
    let examples_tag = tags.iter().find(|t| t.starts_with("@examples")).unwrap();
    assert!(
        examples_tag.contains("more_ex()"),
        "doc after cfg must continue @examples with partition: {:?}",
        tags
    );
}

#[test]
fn attr_interrupt_before_any_doc_does_not_affect_result() {
    // Non-doc attr BEFORE any doc content is harmless
    let attrs: Vec<syn::Attribute> = vec![
        syn::parse_quote!(#[cfg(feature = "x")]),
        syn::parse_quote!(#[doc = "@param x A value"]),
    ];
    let tags = roxygen_tags_from_attrs(&attrs);
    assert!(tags.iter().any(|t| t.starts_with("@param")));
}

#[test]
fn attr_interrupt_cfg_between_return_lines_doc_continues() {
    // Split: @return block, then cfg, then bare prose.
    // With partition, prose DOES continue the @return tag (correct behaviour).
    let attrs: Vec<syn::Attribute> = vec![
        syn::parse_quote!(#[doc = "@return The output value."]),
        syn::parse_quote!(#[cfg(feature = "x")]),
        syn::parse_quote!(#[doc = "Extra continuation line."]),
    ];
    let tags = roxygen_tags_from_attrs(&attrs);
    let return_tag = tags.iter().find(|t| t.starts_with("@return")).unwrap();
    assert!(
        return_tag.contains("Extra continuation line."),
        "@return must include post-cfg doc continuation with partition: {:?}",
        tags
    );
}

#[test]
fn attr_interrupt_multiple_cfg_between_doc_all_continue() {
    // Multiple non-doc attrs between doc lines — all doc still processes contiguously
    let attrs: Vec<syn::Attribute> = vec![
        syn::parse_quote!(#[doc = "@examples"]),
        syn::parse_quote!(#[cfg(feature = "a")]),
        syn::parse_quote!(#[doc = "line_a()"]),
        syn::parse_quote!(#[cfg(feature = "b")]),
        syn::parse_quote!(#[doc = "line_b()"]),
    ];
    let tags = roxygen_tags_from_attrs(&attrs);
    let examples_tag = tags.iter().find(|t| t.starts_with("@examples")).unwrap();
    assert!(
        examples_tag.contains("line_a()") && examples_tag.contains("line_b()"),
        "all doc lines must continue @examples across multiple cfg: {:?}",
        tags
    );
}

// endregion

// region: method doc prose is promoted to @description, never @title
//
// The page @title is the structural class/method name (emitted by ClassDocBuilder /
// lib.rs), so the core never derives a @title from prose. Leading prose folds into a
// single multi-paragraph @description.

fn make_r6_method_doc_attrs(lines: &[&str]) -> Vec<syn::Attribute> {
    lines
        .iter()
        .map(|line| syn::parse_quote!(#[doc = #line]))
        .collect()
}

#[test]
fn method_prose_emits_description_not_title() {
    let attrs = make_r6_method_doc_attrs(&["Compute the sum of all elements."]);
    let tags = roxygen_tags_from_attrs_for_r6_method(&attrs);
    assert!(
        !tags.iter().any(|t| t.starts_with("@title")),
        "prose must not become @title: {:?}",
        tags
    );
    let desc = tags.iter().find(|t| t.starts_with("@description"));
    assert!(desc.is_some(), "expected @description: {:?}", tags);
    assert!(
        desc.unwrap().contains("Compute the sum of all elements."),
        "wrong @description content: {:?}",
        tags
    );
}

#[test]
fn method_prose_paragraphs_fold_into_one_description() {
    let attrs = make_r6_method_doc_attrs(&[
        "Compute the sum of all elements.",
        "",
        "This is the second paragraph.",
    ]);
    let tags = roxygen_tags_from_attrs_for_r6_method(&attrs);
    assert!(
        !tags
            .iter()
            .any(|t| t.starts_with("@title") || t.starts_with("@details")),
        "no @title or @details from prose: {:?}",
        tags
    );
    let desc = tags.iter().find(|t| t.starts_with("@description")).unwrap();
    // Both paragraphs land in the description, separated by a blank line.
    assert!(
        desc.contains("Compute the sum of all elements.")
            && desc.contains("This is the second paragraph."),
        "both paragraphs must be in @description: {:?}",
        tags
    );
}

#[test]
fn method_no_doc_emits_nothing() {
    let attrs: Vec<syn::Attribute> = vec![];
    let tags = roxygen_tags_from_attrs_for_r6_method(&attrs);
    assert!(
        tags.is_empty(),
        "expected empty tags for no-doc: {:?}",
        tags
    );
}

// endregion

// region: prose → @description promotion (via roxygen_tags_from_attrs)

fn make_doc_attrs_plain(lines: &[&str]) -> Vec<syn::Attribute> {
    lines
        .iter()
        .map(|line| syn::parse_quote!(#[doc = #line]))
        .collect()
}

#[test]
fn prose_promoted_to_description_never_title_or_details() {
    // The core never derives @title (structural name, set by the caller) nor @details
    // (folded into @description) from prose.
    let attrs = make_doc_attrs_plain(&[
        "Title.",
        "",
        "Description.",
        "",
        "What used to be details.",
        "@param x A value",
    ]);
    let tags = roxygen_tags_from_attrs(&attrs);
    assert!(
        !tags.iter().any(|t| t.starts_with("@title")),
        "no @title from prose: {:?}",
        tags
    );
    assert!(
        !tags.iter().any(|t| t.starts_with("@details")),
        "no @details from prose: {:?}",
        tags
    );
    let desc = tags.iter().find(|t| t.starts_with("@description")).unwrap();
    // All three leading paragraphs fold into the single @description.
    assert!(
        desc.contains("Title.")
            && desc.contains("Description.")
            && desc.contains("What used to be details."),
        "all leading paragraphs must fold into @description: {:?}",
        tags
    );
    // @description precedes @param.
    assert!(
        tags.iter().position(|t| t.starts_with("@description"))
            < tags.iter().position(|t| t.starts_with("@param")),
        "@description must precede @param: {:?}",
        tags
    );
}

#[test]
fn explicit_description_not_clobbered_by_prose() {
    // An author-written @description suppresses prose promotion entirely.
    let attrs = make_doc_attrs_plain(&[
        "Leading prose that would otherwise become the description.",
        "@description Explicit description.",
        "@param x A value",
    ]);
    let tags = roxygen_tags_from_attrs(&attrs);
    let count = tags
        .iter()
        .filter(|t| t.starts_with("@description"))
        .count();
    assert_eq!(count, 1, "expected exactly one @description: {:?}", tags);
    let desc = tags.iter().find(|t| t.starts_with("@description")).unwrap();
    assert!(
        desc.contains("Explicit description.") && !desc.contains("Leading prose"),
        "explicit @description must win: {:?}",
        tags
    );
}

#[test]
fn tag_led_block_gets_no_description_or_title() {
    // An @inherit-led block (no leading prose) must NOT gain a spurious description
    // or title; the tag is preserved.
    let attrs = make_doc_attrs_plain(&["@inherit foo"]);
    let tags = roxygen_tags_from_attrs(&attrs);
    assert!(
        !tags
            .iter()
            .any(|t| t.starts_with("@title") || t.starts_with("@description")),
        "tag-led block must not gain @title/@description: {:?}",
        tags
    );
    assert!(
        tags.iter().any(|t| t.starts_with("@inherit")),
        "the @inherit tag must be preserved: {:?}",
        tags
    );
}

// endregion

// region: rustdoc intra-doc links neutralized for roxygen2 (#1054 follow-up)
//
// rpkg's DESCRIPTION enables `Config/roxygen2/markdown: TRUE`, so roxygen2 reads
// `[Foo]` as a `\link{}` to an R topic. rustdoc summaries use the same syntax for
// *Rust* items, which roxygen2 can't resolve. `sanitize_roxygen_links` strips the
// link brackets (keeping `` `code` `` spans) before prose becomes `@description`.

/// The scanner as the default `roxygen_prose_links = "strip"` runs it.
fn strip_links(s: &str) -> String {
    sanitize_roxygen_links(s, Neutralize::AllLinks)
}

#[test]
fn sanitize_strips_shortcut_and_reference_links() {
    // Shortcut `[`Foo`]` → keep the code span; reference `[x][crate::y]` → keep `x`.
    assert_eq!(
        strip_links("same column shape as [`REMapB`]"),
        "same column shape as `REMapB`"
    );
    assert_eq!(
        strip_links("see [`AsSerialize`][serde::AsSerialize] wrapper"),
        "see `AsSerialize` wrapper"
    );
    assert_eq!(strip_links("a plain [Topic] link"), "a plain Topic link");
}

#[test]
fn sanitize_keeps_real_markdown_links() {
    // `[text](url)` is valid roxygen2 markdown — leave it alone.
    let s = "see [the docs](https://example.com) for more";
    assert_eq!(strip_links(s), s);
}

#[test]
fn sanitize_leaves_brackets_inside_code_spans() {
    // Markdown (and roxygen2) never parse `[...]` inside backticks as a
    // link — stripping there would corrupt code like `x[i]`.
    let s = "index with `x[i]` or `m[, 1]` as usual";
    assert_eq!(strip_links(s), s);
    // A rustdoc link *around* a code span is still stripped.
    assert_eq!(
        strip_links("see [`Vec<T>`] and `v[0]`"),
        "see `Vec<T>` and `v[0]`"
    );
}

#[test]
fn sanitize_is_utf8_safe() {
    // Multi-byte chars around a link must not panic or corrupt.
    assert_eq!(strip_links("café [`Foo`] — déjà"), "café `Foo` — déjà");
}

#[test]
fn sanitize_preserves_escaped_brackets() {
    // `\[u8\]` is idiomatic rustdoc for a literal bracket (suppresses
    // intra-doc link resolution). Eating the brackets but not the
    // backslashes produced invalid Rd macros like `Box<\u8\>` in
    // altrep_vec.Rd. Pass the escape through; roxygen2 markdown unescapes.
    let s = r"Create a Box<\[u8\]> ALTREP raw vector.";
    assert_eq!(strip_links(s), s);
    // An unescaped shortcut link in the same string is still stripped.
    assert_eq!(
        strip_links(r"see [Foo] and Box<\[u8\]>"),
        r"see Foo and Box<\[u8\]>"
    );
}

#[test]
fn leading_prose_promotes_and_sanitizes() {
    let attrs = make_doc_attrs_plain(&[
        "`HashMap` field — same column shape as [`REMapB`].",
        "",
        "Second paragraph references [`S7PropOuter`].",
    ]);
    let desc = leading_prose_from_attrs(&attrs, ProseLinks::Strip).expect("expected leading prose");
    assert!(
        desc.contains("`REMapB`") && !desc.contains("[`REMapB`]"),
        "links must be neutralized: {desc:?}"
    );
    assert!(
        desc.contains("`S7PropOuter`") && !desc.contains("[`S7PropOuter`]"),
        "second-paragraph links must be neutralized: {desc:?}"
    );
    // Paragraph boundary preserved as a blank-line separator.
    assert!(
        desc.contains("\n\n"),
        "paragraph break must be preserved: {desc:?}"
    );
}

#[test]
fn leading_prose_none_for_tag_led_block() {
    let attrs = make_doc_attrs_plain(&["@param x A value"]);
    assert_eq!(leading_prose_from_attrs(&attrs, ProseLinks::Strip), None);
}

/// Leading prose as rustdoc writes it (`/// text`), up to a closing tag.
fn prose_of(lines: &[&str]) -> Option<String> {
    let attrs: Vec<syn::Attribute> = lines
        .iter()
        .chain(&["@export"])
        .map(|line| {
            let value = if line.is_empty() {
                String::new()
            } else {
                format!(" {line}")
            };
            syn::parse_quote!(#[doc = #value])
        })
        .collect();
    leading_prose_from_attrs(&attrs, ProseLinks::Strip)
}

#[test]
fn leading_prose_keeps_its_line_breaks() {
    assert_eq!(
        prose_of(&["", "One paragraph", "wrapped over two lines.", "", ""]).as_deref(),
        Some("One paragraph\nwrapped over two lines.")
    );
}

#[test]
fn leading_prose_keeps_a_list_and_its_nested_indent() {
    assert_eq!(
        prose_of(&[
            "Modes:",
            "- a: first",
            "  continued",
            "  - nested",
            "- b: second"
        ])
        .as_deref(),
        Some("Modes:\n- a: first\n  continued\n  - nested\n- b: second")
    );
}

#[test]
fn leading_prose_neutralizes_a_link_across_two_lines() {
    assert_eq!(
        prose_of(&["see [the `Foo`", "type] here"]).as_deref(),
        Some("see the `Foo`\ntype here")
    );
}

#[test]
fn leading_prose_leaves_fenced_code_untouched() {
    assert_eq!(
        prose_of(&[
            "Example:",
            "```r",
            "a[i]",
            "",
            "b[j]",
            "```",
            "after [`Foo`]"
        ])
        .as_deref(),
        Some("Example:\n```r\na[i]\n\nb[j]\n```\nafter `Foo`")
    );
}

#[test]
fn leading_prose_confines_a_stray_backtick_to_its_paragraph() {
    assert_eq!(
        prose_of(&["a stray ` backtick [kept]", "", "then [Foo]"]).as_deref(),
        Some("a stray ` backtick [kept]\n\nthen Foo")
    );
}

// endregion

// region: roxygen_prose_links = "keep" — leading prose written for roxygen2
//
// rustdoc reads `[set_threshold()]` as an intra-doc link to a Rust fn, roxygen2
// as `\link{}` to an R topic; the text cannot say which. The crate key picks
// the reader, and the default stays `strip`.

/// The downstream report's block, as `///` lines: a roxygen2 link in the
/// leading prose and in an explicit tag.
fn r_first_block() -> Vec<syn::Attribute> {
    make_doc_attrs_plain(&[
        "`summary()`: ... 50 unless [set_threshold()] set another.",
        "See [stats::median()] and [`Thing`][thing_class].",
        "@param x An object, see [set_threshold()].",
    ])
}

#[test]
fn strip_drops_roxygen_links_from_prose_but_not_from_tags() {
    let tags = roxygen_tags_with(&r_first_block(), ProseLinks::Strip);
    assert_eq!(
        tags,
        vec![
            "@description `summary()`: ... 50 unless set_threshold() set another.\n\
             See stats::median() and `Thing`."
                .to_string(),
            "@param x An object, see [set_threshold()].".to_string(),
        ]
    );
}

#[test]
fn keep_passes_prose_through_like_an_explicit_tag() {
    let tags = roxygen_tags_with(&r_first_block(), ProseLinks::Keep);
    assert_eq!(
        tags,
        vec![
            "@description `summary()`: ... 50 unless [set_threshold()] set another.\n\
             See [stats::median()] and [`Thing`][thing_class]."
                .to_string(),
            "@param x An object, see [set_threshold()].".to_string(),
        ]
    );
}

#[test]
fn keep_preserves_layout_and_drops_only_outer_blank_lines() {
    // As rustdoc writes `/// text`: one leading space, which `doc_lines` drops.
    let attrs = make_doc_attrs_plain(&[
        "",
        " Paragraph one with [other_fn()]",
        "   and an indented continuation.",
        "",
        " ```r",
        " x[i]",
        " ```",
        "",
        " @export",
    ]);
    assert_eq!(
        leading_prose_from_attrs(&attrs, ProseLinks::Keep).as_deref(),
        Some("Paragraph one with [other_fn()]\n  and an indented continuation.\n\n```r\nx[i]\n```")
    );
    // Without links, both settings agree.
    let plain = make_doc_attrs_plain(&["One line.", "", "Two `x[i]`."]);
    assert_eq!(
        leading_prose_from_attrs(&plain, ProseLinks::Keep),
        leading_prose_from_attrs(&plain, ProseLinks::Strip)
    );
}

#[test]
fn keep_still_yields_no_description_for_a_tag_led_block() {
    let attrs = make_doc_attrs_plain(&["@param x A value, see [other_fn()]"]);
    assert_eq!(leading_prose_from_attrs(&attrs, ProseLinks::Keep), None);
    assert_eq!(
        roxygen_tags_with(&attrs, ProseLinks::Keep),
        vec!["@param x A value, see [other_fn()]".to_string()]
    );
}

#[test]
fn explicit_description_wins_under_either_setting() {
    let attrs = make_doc_attrs_plain(&[
        "Rustdoc summary with [`RustType`].",
        "@description See [other_fn()].",
    ]);
    for links in [ProseLinks::Strip, ProseLinks::Keep] {
        assert_eq!(
            roxygen_tags_with(&attrs, links),
            vec!["@description See [other_fn()].".to_string()]
        );
    }
}

#[test]
fn prose_links_default_is_strip() {
    assert_eq!(ProseLinks::default(), ProseLinks::Strip);
    assert_eq!(ProseLinks::parse_name("keep"), Some(ProseLinks::Keep));
    assert_eq!(ProseLinks::parse_name("strip"), Some(ProseLinks::Strip));
    assert_eq!(ProseLinks::parse_name("Keep"), None);
}

// endregion

// region: rustdoc-only links never reach roxygen2 (#1739)
//
// roxygen2 reads `[pkg::topic]` (and `` [`pkg::topic`] ``, `[text][pkg::topic]`)
// as a link into the R package `pkg`, everything before the last `::`. When
// that cannot be a package, the link is rustdoc's: it loses its brackets under
// `"keep"` and in explicit tag text too, not only under the default `"strip"`.

/// The scanner as `"keep"` prose and explicit tag text run it.
fn rustdoc_only(s: &str) -> String {
    sanitize_roxygen_links(s, Neutralize::RustdocOnly)
}

#[test]
fn rustdoc_only_targets_are_those_no_r_package_can_resolve() {
    for target in [
        "crate::x",
        "`crate::x`",
        "self::x",
        "Self::new",
        "Self::new()",
        "crate::a::B",
        "a::b::c",
        "`a::b::c`",
        "my_mod::f()",
        "std::vec::Vec",
        "x::f",
        "pkg.::f",
        "1pkg::f",
        "fn@crate::x",
    ] {
        assert!(is_rustdoc_only_target(target), "{target:?} is rustdoc-only");
    }
    // roxygen2's own forms, and paths whose root can be an R package.
    for target in [
        "Foo",
        "`Foo`",
        "fn()",
        "`fn()`",
        "pkg::fn()",
        "stats::median()",
        "data.table::fread",
        "R6::R6Class",
        "Type::method",
        "`Sources::prepare`",
        "super::glue()",
        "S7::new_class",
    ] {
        assert!(
            !is_rustdoc_only_target(target),
            "{target:?} may be an R link"
        );
    }
}

#[test]
fn rustdoc_only_scope_rewrites_each_rustdoc_form() {
    assert_eq!(rustdoc_only("see [`crate::Foo`]."), "see `crate::Foo`.");
    assert_eq!(rustdoc_only("see [Self::new()]."), "see Self::new().");
    assert_eq!(rustdoc_only("see [self::helper]."), "see self::helper.");
    assert_eq!(
        rustdoc_only("the [registry][crate::registry]"),
        "the registry"
    );
    assert_eq!(
        rustdoc_only("[`Sources::prepare`][crate::Sources::prepare]"),
        "`Sources::prepare`"
    );
    assert_eq!(
        rustdoc_only("[`a::b::c`] and [my_mod::f()]"),
        "`a::b::c` and my_mod::f()"
    );
}

#[test]
fn rustdoc_only_scope_leaves_r_links_and_code_alone() {
    for s in [
        "[the docs](https://example.com)",
        "[other_fn()]",
        "[stats::median()]",
        "[topic]",
        "[`topic`]",
        "[Type::method]",
        "[`Sources::prepare`]",
        "[super::glue()]",
        "[text][other_fn()]",
        "[`text`][pkg::topic]",
        "code `x[crate::y]` stays",
        r"escaped \[crate::x\] stays",
    ] {
        assert_eq!(rustdoc_only(s), s, "{s:?} must stay as written");
    }
}

#[test]
fn keep_prose_loses_only_its_rustdoc_links() {
    let attrs = make_doc_attrs_plain(&[
        "See [`crate::Sources::prepare`], [Self::new] and [other_fn()].",
        "Also [`Sources::prepare`][crate::Sources::prepare] and [pkg::fn()].",
        "@export",
    ]);
    assert_eq!(
        roxygen_tags_with(&attrs, ProseLinks::Keep),
        vec![
            "@description See `crate::Sources::prepare`, Self::new and [other_fn()].\n\
             Also `Sources::prepare` and [pkg::fn()]."
                .to_string(),
            "@export".to_string(),
        ]
    );
}

#[test]
fn tag_text_loses_its_rustdoc_links_under_either_setting() {
    let attrs = make_doc_attrs_plain(&[
        "@param x A [crate::Thing], see [pkg::fn()].",
        "@details See [`Sources::prepare`][crate::Sources::prepare]",
        "and [`Self::x`].",
        "",
        "```r",
        "a[crate::b]",
        "```",
        "Then [other_fn()].",
    ]);
    for links in [ProseLinks::Strip, ProseLinks::Keep] {
        assert_eq!(
            roxygen_tags_with(&attrs, links),
            vec![
                "@param x A crate::Thing, see [pkg::fn()].".to_string(),
                "@details See `Sources::prepare`\nand `Self::x`.\n\n```r\na[crate::b]\n```\n\
                 Then [other_fn()]."
                    .to_string(),
            ]
        );
    }
}

#[test]
fn code_tags_keep_their_text() {
    let attrs = make_doc_attrs_plain(&[
        "@examples",
        "x <- list(a = 1)",
        "x[crate::a] # [Self::x]",
        "@usage f(x[crate::a])",
        "@rawRd \\note{[crate::x]}",
    ]);
    assert_eq!(
        roxygen_tags_with(&attrs, ProseLinks::Keep),
        vec![
            "@examples\nx <- list(a = 1)\nx[crate::a] # [Self::x]".to_string(),
            "@usage f(x[crate::a])".to_string(),
            "@rawRd \\note{[crate::x]}".to_string(),
        ]
    );
}

#[test]
fn ambiguous_type_paths_stay_r_links() {
    // `[`Type::method`]` is roxygen2's `[`pkg::topic`]`: it stays a link in tag
    // text and under "keep" (leading prose under "strip" loses it). Telling
    // the two apart by the package's dependencies is #1742.
    let attrs = make_doc_attrs_plain(&[
        "See [`Sources::prepare`].",
        "@details See [`Sources::prepare`].",
    ]);
    assert_eq!(
        roxygen_tags_with(&attrs, ProseLinks::Keep),
        vec![
            "@description See [`Sources::prepare`].".to_string(),
            "@details See [`Sources::prepare`].".to_string(),
        ]
    );
    assert_eq!(
        roxygen_tags_with(&attrs, ProseLinks::Strip),
        vec![
            "@description See `Sources::prepare`.".to_string(),
            "@details See [`Sources::prepare`].".to_string(),
        ]
    );
}

#[test]
fn unqualified_underscore_topics_stay_r_links() {
    // Only the package part before `::` is checked, so an R topic with an
    // underscore and no package is never taken for a Rust path.
    let links = "[run_model()], [`run_model()`], [pkg_results_methods], \
                 [`pkg_results_methods`], [text][other_topic], [text][other_fn()], \
                 [dplyr::bind_rows()], [readRDS()]";
    let attrs = make_doc_attrs_plain(&[&format!("See {links}."), &format!("@seealso {links}")]);
    assert_eq!(
        roxygen_tags_with(&attrs, ProseLinks::Keep),
        vec![
            format!("@description See {links}."),
            format!("@seealso {links}"),
        ]
    );
    // Under "strip" leading prose loses every link by design; tag text keeps them.
    let tags = roxygen_tags_with(&attrs, ProseLinks::Strip);
    assert_eq!(tags[1], format!("@seealso {links}"));
}

#[test]
fn a_line_after_a_single_line_tag_keeps_any_link_out_of_r() {
    // The docs' recipe for a rustdoc-only line (MINIEXTENDR_ATTRIBUTE.md).
    let attrs = make_doc_attrs_plain(&[
        "Prepares the sources.",
        "",
        "@export",
        "",
        "Rust callers: see [`Sources::prepare`] and [`Prepared`].",
    ]);
    assert_eq!(
        roxygen_tags_with(&attrs, ProseLinks::Keep),
        vec![
            "@description Prepares the sources.".to_string(),
            "@export".to_string(),
        ]
    );
    assert_eq!(
        kept_docs(&attrs),
        [
            "Prepares the sources.",
            "",
            "",
            "Rust callers: see [`Sources::prepare`] and [`Prepared`].",
        ]
    );
}

// endregion

// region: inline rustdoc links never reach roxygen2 as \href (#1744)
//
// rustdoc also reads `[text](crate::x)` as an intra-doc link. roxygen2 reads
// it as a web link and renders `\href{crate::x}{text}`, a dead link, with no
// warning. When the destination is a Rust path that is rustdoc-only, the link
// keeps its text alone under every scope; any URL, and a path roxygen2 or a
// relative URL may mean, stays as written.

/// Both scopes the scanner runs with.
fn under_each_scope(s: &str) -> [String; 2] {
    [strip_links(s), rustdoc_only(s)]
}

#[test]
fn inline_rustdoc_links_keep_their_text_under_each_scope() {
    for (written, text) in [
        ("see [text](crate::x).", "see text."),
        ("see [`Foo`](crate::Foo).", "see `Foo`."),
        ("[text](self::a::b())", "text"),
        ("[m](Self::new)", "m"),
        ("[t](my_crate::x)", "t"),
        ("[mac](crate::mac!)", "mac"),
        ("[c](a::b::c)", "c"),
        ("café [`Foo`](crate::Foo) — déjà", "café `Foo` — déjà"),
        ("[a](crate::x) and [b](Self::y())", "a and b"),
    ] {
        assert_eq!(under_each_scope(written), [text, text], "{written:?}");
    }
}

#[test]
fn inline_links_to_urls_or_r_paths_stay() {
    for s in [
        "[text](https://a.b/c::d)",
        "[text](https://example.com)",
        "[text](./x.html)",
        "[text](x.html#crate::y)",
        "[text](crate::x?y)",
        "[text](crate:x)",
        "[text](crate:::x)",
        "[text](Foo)",
        "[text](Type::method)",
        "[text](dplyr::bind_rows)",
        "[text](super::glue())",
        "[text](crate::f(x))",
        "[text](crate::x \"title\")",
        "[text](<crate::x>)",
        "[text]()",
        "[text](::x)",
        "[text](crate::)",
    ] {
        assert_eq!(under_each_scope(s), [s, s], "{s:?} must stay as written");
    }
}

#[test]
fn inline_rustdoc_links_leave_the_scanner_on_track() {
    // An unclosed destination is no link: the brackets stay.
    assert_eq!(
        under_each_scope("[t](crate::x"),
        ["[t](crate::x", "[t](crate::x"]
    );
    // An unmatched `)` after the link is text.
    assert_eq!(
        under_each_scope("[a](crate::f()) more)"),
        ["a more)", "a more)"]
    );
    // CommonMark gives the inner link priority, so the outer `[` stays text.
    assert_eq!(under_each_scope("[a [b](crate::x) c"), ["[a b c", "[a b c"]);
    // A link after one that stays is still found.
    assert_eq!(
        under_each_scope("[u](https://x.y) then [r](crate::r)"),
        ["[u](https://x.y) then r", "[u](https://x.y) then r"]
    );
    // Neither a code span nor an escaped bracket is a link.
    for s in ["`[t](crate::x)` stays", r"escaped \[t](crate::x) stays"] {
        assert_eq!(under_each_scope(s), [s, s], "{s:?} must stay as written");
    }
}

#[test]
fn inline_rustdoc_links_lose_their_link_in_prose_and_tags_under_either_setting() {
    let attrs = make_doc_attrs_plain(&[
        "See [`Foo`](crate::Foo) and [the docs](https://example.com).",
        "@description Built by [new](Self::new), see [stats](stats::median).",
        "@seealso [t](my_crate::x), [`bind_rows`](dplyr::bind_rows)",
        "@examples",
        "x[1](crate::x)",
    ]);
    for links in [ProseLinks::Strip, ProseLinks::Keep] {
        assert_eq!(
            roxygen_tags_with(&attrs, links),
            vec![
                "@description Built by new, see [stats](stats::median).".to_string(),
                "@seealso t, [`bind_rows`](dplyr::bind_rows)".to_string(),
                "@examples\nx[1](crate::x)".to_string(),
            ]
        );
    }
    // Leading prose (no explicit `@description`) loses it too.
    let attrs = make_doc_attrs_plain(&[
        "See [`Foo`](crate::Foo) and [the docs](https://example.com).",
        "@export",
    ]);
    for links in [ProseLinks::Strip, ProseLinks::Keep] {
        assert_eq!(
            roxygen_tags_with(&attrs, links),
            vec![
                "@description See `Foo` and [the docs](https://example.com).".to_string(),
                "@export".to_string(),
            ]
        );
    }
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
fn test_describein_keeps_continuation_lines() {
    // `@describeIn topic description` may wrap; the wrapped part is the rest
    // of the description and must not be dropped (#1476). The continuation
    // keeps its indent, less the one rustdoc space.
    let attrs: Vec<syn::Attribute> = vec![
        syn::parse_quote!(#[doc = " @describeIn new_bag Number of values in the bag,"]),
        syn::parse_quote!(#[doc = "   as an integer scalar."]),
        syn::parse_quote!(#[doc = " @export"]),
    ];
    let tags = roxygen_tags_from_attrs(&attrs);
    assert_eq!(
        tags,
        vec![
            "@describeIn new_bag Number of values in the bag,\n  as an integer scalar.",
            "@export"
        ]
    );
}

#[test]
fn test_wrapped_single_word_tags_keep_continuation() {
    for tag in ["@family", "@inherit", "@inheritParams", "@inheritSection"] {
        let first = format!("{tag} alpha");
        let attrs: Vec<syn::Attribute> = vec![
            syn::parse_quote!(#[doc = #first]),
            syn::parse_quote!(#[doc = "   beta"]),
        ];
        let tags = roxygen_tags_from_attrs(&attrs);
        assert_eq!(tags, vec![format!("{tag} alpha\n  beta")], "tag {tag}");
    }
}

#[test]
fn test_title_continuation_joined_onto_one_line() {
    // A wrapped @title stays a single roxygen line (roxygen2 wants one line),
    // instead of losing the wrapped words.
    let attrs: Vec<syn::Attribute> = vec![
        syn::parse_quote!(#[doc = "@title A rather long title that the author"]),
        syn::parse_quote!(#[doc = "  wrapped onto a second line"]),
        syn::parse_quote!(#[doc = "@param x A value."]),
    ];
    let tags = roxygen_tags_from_attrs(&attrs);
    assert_eq!(
        tags,
        vec![
            "@title A rather long title that the author wrapped onto a second line",
            "@param x A value."
        ]
    );
    // Rendering keeps it on one `#'` line.
    assert!(!format_roxygen_tags(&tags[..1]).trim_end().contains('\n'));
}

#[test]
fn test_keywords_and_concept_join_continuation_for_roxygen() {
    for tag in ["@keywords", "@concept"] {
        let first = format!("{tag} alpha");
        let attrs: Vec<syn::Attribute> = vec![
            syn::parse_quote!(#[doc = #first]),
            syn::parse_quote!(#[doc = "  beta"]),
            syn::parse_quote!(#[doc = "@param x A value."]),
        ];
        let tags = roxygen_tags_from_attrs(&attrs);
        assert_eq!(
            tags,
            vec![format!("{tag} alpha beta"), "@param x A value.".into()]
        );
        assert!(!format_roxygen_tags(&tags[..1]).trim_end().contains('\n'));
    }
}

#[test]
fn test_bare_name_takes_next_line_as_topic() {
    // roxygen2 accepts the topic on the line after a bare `@name` / `@rdname`;
    // the registry routes pages from the joined `#' @name <topic>` line, so
    // the topic must not be dropped.
    for tag in ["@name", "@rdname"] {
        let attrs: Vec<syn::Attribute> = vec![
            syn::parse_quote!(#[doc = #tag]),
            syn::parse_quote!(#[doc = "  shared_topic"]),
            syn::parse_quote!(#[doc = "not part of the topic"]),
            syn::parse_quote!(#[doc = "@param x A value."]),
        ];
        let tags = roxygen_tags_from_attrs(&attrs);
        assert_eq!(
            tags,
            vec![
                format!("{tag} shared_topic"),
                "@param x A value.".to_string()
            ],
            "{tag}"
        );
    }
}

#[test]
fn test_rdname_stays_single_line() {
    // `@rdname` takes a bare topic name; a following prose line is not part of it.
    let attrs: Vec<syn::Attribute> = vec![
        syn::parse_quote!(#[doc = "@rdname topic"]),
        syn::parse_quote!(#[doc = "stray prose"]),
    ];
    let tags = roxygen_tags_from_attrs(&attrs);
    assert_eq!(tags, vec!["@rdname topic"]);
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

#[test]
fn strip_method_tags_wraps_each_warning_in_its_own_anonymous_const_scope() {
    // #1118: two impl blocks on the same type each strip a method-only tag.
    // Previously this needed a per-block `block_id` disambiguator baked into
    // the const names, or the two emissions collided with E0428 once spliced
    // side by side. Wrapping each warning in its own `const _: () = { .. };`
    // block gives every emission a fresh, anonymous item scope instead — so
    // two calls produce textually *identical* tokens and still never collide.
    let tags: Vec<String> = vec!["@param x an input".to_string()];
    let span = proc_macro2::Span::call_site();

    let (_, warnings_a) = strip_method_tags(&tags, "MyType", span);
    let (_, warnings_b) = strip_method_tags(&tags, "MyType", span);

    let sa = warnings_a.to_string();
    let sb = warnings_b.to_string();

    assert_eq!(
        sa, sb,
        "no per-block disambiguator needed — anonymous scoping does the work"
    );
    assert!(
        sa.contains("const _"),
        "warning must be wrapped in an anonymous const scope: {sa}"
    );
    assert!(
        sa.contains("_MINIEXTENDR_IMPL_METHOD_TAG_WARN"),
        "WARN const missing: {sa}"
    );
}

#[test]
fn strip_method_tags_activates_use_const_referencing_warn_ident() {
    // #1206: an unused `#[deprecated]` const warns nowhere — it's dead code
    // implying a feature. A sibling USE const whose initializer reads the
    // WARN const's value makes rustc's `deprecated` lint actually fire at the
    // impl-block span, turning the silent no-op into a real compile warning
    // that points the user at the misplaced tag.
    let tags: Vec<String> = vec!["@param x an input".to_string()];
    let span = proc_macro2::Span::call_site();

    let (_, warnings) = strip_method_tags(&tags, "MyType", span);
    let s = warnings.to_string();

    assert!(
        s.contains("_MINIEXTENDR_IMPL_METHOD_TAG_WARN"),
        "WARN const missing: {s}"
    );
    assert!(
        s.contains("_MINIEXTENDR_IMPL_METHOD_TAG_USE"),
        "USE const missing: {s}"
    );
    // The USE const's initializer must read the WARN const by name, so
    // referencing it trips rustc's `deprecated` lint on the WARN const.
    assert!(
        s.contains(
            "const _MINIEXTENDR_IMPL_METHOD_TAG_USE : () = _MINIEXTENDR_IMPL_METHOD_TAG_WARN"
        ),
        "USE const must initialize from the WARN const's value: {s}"
    );
}

#[test]
fn strip_method_tags_r6_activates_use_const_for_stripped_tags() {
    // Same activation for the R6 variant, exercised on a tag R6 still strips
    // (`@return` — unlike `@param`, which R6 intentionally keeps).
    let tags: Vec<String> = vec!["@return something".to_string()];
    let span = proc_macro2::Span::call_site();

    let (kept, warnings) = strip_method_tags_r6(&tags, "MyR6Type", span);
    assert!(kept.is_empty(), "@return must still be stripped for R6");

    let s = warnings.to_string();
    assert!(
        s.contains("_MINIEXTENDR_IMPL_METHOD_TAG_WARN"),
        "WARN const missing: {s}"
    );
    assert!(
        s.contains("_MINIEXTENDR_IMPL_METHOD_TAG_USE"),
        "USE const missing: {s}"
    );
    assert!(
        s.contains(
            "const _MINIEXTENDR_IMPL_METHOD_TAG_USE : () = _MINIEXTENDR_IMPL_METHOD_TAG_WARN"
        ),
        "USE const must initialize from the WARN const's value: {s}"
    );
}

// endregion

#[test]
fn split_r_formals_ignores_nested_commas() {
    // Plain formals split as before.
    assert_eq!(split_r_formals("x, y, z"), vec!["x", "y", "z"]);
    // A match_arg default `c("fast", "slow")` must stay one formal, not be
    // shredded into `mode = c("fast"` + `"slow")` (the ScalerS7/ScalerR6 bug).
    assert_eq!(
        split_r_formals(r#"x, mode = c("fast", "slow"), ..."#),
        vec!["x", r#"mode = c("fast", "slow")"#, "..."]
    );
    // Nested calls / brackets are respected too.
    assert_eq!(
        split_r_formals("self, opts = list(a = 1, b = 2), ..."),
        vec!["self", "opts = list(a = 1, b = 2)", "..."]
    );
    assert!(split_r_formals("").is_empty());
}

#[test]
fn formal_name_strips_default() {
    assert_eq!(formal_name("x"), "x");
    assert_eq!(formal_name(r#"mode = c("fast", "slow")"#), "mode");
    assert_eq!(formal_name("..."), "...");
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
// endregion

// region: comma-list @param dedup (duplicated \argument Rd entries, #1261 item 2)

#[test]
fn extract_param_names_splits_comma_list() {
    // A single `@param a,b,c desc` tag documents three names, not one.
    let tags = vec!["@param a,b,c Numeric scalars.".to_string()];
    let names = extract_param_names(&tags);
    assert_eq!(names.len(), 3);
    assert!(names.contains("a"));
    assert!(names.contains("b"));
    assert!(names.contains("c"));
}

#[test]
fn extract_param_names_single_name_unaffected() {
    let tags = vec!["@param x Input value.".to_string()];
    let names = extract_param_names(&tags);
    assert_eq!(names.len(), 1);
    assert!(names.contains("x"));
}

#[test]
fn param_documented_true_for_every_name_in_comma_list() {
    let tags = vec!["@param a,b,c Numeric scalars.".to_string()];
    assert!(param_documented(&tags, "a"));
    assert!(param_documented(&tags, "b"));
    assert!(param_documented(&tags, "c"));
}

#[test]
fn param_documented_false_for_undocumented_name() {
    let tags = vec!["@param a,b,c Numeric scalars.".to_string()];
    assert!(!param_documented(&tags, "d"));
}

#[test]
fn param_documented_no_prefix_false_positive() {
    // The old `starts_with(&format!("@param {name}"))` check would wrongly
    // treat "@param x2 desc" as documenting "x" too, since "@param x2 desc"
    // starts with "@param x". Exact comma-split membership must not repeat
    // that false positive.
    let tags = vec!["@param x2 Second input.".to_string()];
    assert!(!param_documented(&tags, "x"));
    assert!(param_documented(&tags, "x2"));
}

#[test]
fn find_param_tag_returns_the_comma_list_tag_for_any_covered_name() {
    let tags = vec!["@param a,b,c Numeric scalars.".to_string()];
    assert_eq!(
        find_param_tag(&tags, "b"),
        Some(&"@param a,b,c Numeric scalars.".to_string())
    );
    assert_eq!(find_param_tag(&tags, "d"), None);
}
// endregion

// region: @source opt-in (#1552)

#[test]
fn source_tag_renders_only_when_enabled() {
    assert_eq!(
        source_tag_with(true, "Generated by miniextendr from `T::m`"),
        Some("#' @source Generated by miniextendr from `T::m`".to_string())
    );
    assert_eq!(
        source_tag_with(false, "Generated by miniextendr from `T::m`"),
        None
    );
    // The test crate's manifest has no `[package.metadata.miniextendr]`
    // table, so the manifest-backed entry points read as disabled.
    assert_eq!(source_tag("x"), None);
    let ty: syn::Ident = syn::parse_quote!(Counter);
    let method: syn::Ident = syn::parse_quote!(get);
    assert_eq!(method_source_tag(&ty, &method), None);
    assert_eq!(class_source_tag(&ty), None);
}

// endregion

// region: generated @param lines vs topics that document the arguments (#1590)

fn tag_list(lines: &[&str]) -> Vec<String> {
    lines.iter().map(|line| line.to_string()).collect()
}

#[test]
fn params_documented_elsewhere_for_topic_and_inheritance_tags() {
    for tag in [
        "@rdname observed",
        "@describeIn observed Largest value.",
        "@inheritParams observed",
        "@inherit observed",
        "@inherit observed params return",
    ] {
        let block = tag_list(&["@param x Values.", tag]);
        assert!(
            params_documented_elsewhere(&block, None),
            "`{tag}` takes the arguments from another block"
        );
        assert!(
            params_documented_elsewhere(&block, Some("Counter")),
            "`{tag}` leaves the class page for another block"
        );
    }
}

/// An author `@rdname` naming the page the block lands on anyway (a method's
/// class page) changes nothing, so the block keeps its generated lines.
#[test]
fn rdname_naming_the_default_page_keeps_the_filler() {
    let block = tag_list(&["@rdname Counter"]);
    assert!(!params_documented_elsewhere(&block, Some("Counter")));
    assert!(params_documented_elsewhere(&block, Some("Gauge")));
    assert!(params_documented_elsewhere(&block, None));
}

#[test]
fn params_not_documented_elsewhere_for_own_page_tags() {
    for block in [
        tag_list(&[]),
        tag_list(&["@param x Values."]),
        // `@name` documents the block's own page (#1476); nothing fills it.
        tag_list(&["@name observed", "@title Observed values"]),
        // A field list without `params`: the arguments are not inherited.
        tag_list(&["@inherit observed return description"]),
        // Only `...` is inherited, and `...` never gets a generated line.
        tag_list(&["@inheritDotParams observed"]),
        // Tag names match exactly, not by prefix.
        tag_list(&["@rdnamex observed", "@inheritParamsx observed"]),
    ] {
        assert!(
            !params_documented_elsewhere(&block, None),
            "{block:?} has no other block documenting its arguments"
        );
    }
}

/// Run the standalone-function `@param` generation over a parsed fn item.
fn generated_fn_param_tags(item: proc_macro2::TokenStream) -> (Vec<String>, Vec<(String, String)>) {
    generated_fn_param_tags_with_call(item, None)
}

/// The same for a wrapper whose `.call` formal gets `call_param_doc` (#1613).
fn generated_fn_param_tags_with_call(
    item: proc_macro2::TokenStream,
    call_param_doc: Option<&str>,
) -> (Vec<String>, Vec<(String, String)>) {
    let parsed: crate::miniextendr_fn::MiniextendrFunctionParsed =
        syn::parse2(item).expect("fixture fn parses");
    let mut tags = roxygen_tags_from_attrs(parsed.attrs());
    let placeholders = push_fn_param_tags(
        &mut tags,
        parsed.inputs(),
        &parsed,
        "C_pkg_summary",
        call_param_doc,
    );
    (tags, placeholders)
}

/// The same signature under each doc block: one argument the author
/// documents, one plain, one `match_arg`, one `choices`, plus unnamed dots
/// (never given a generated line).
fn summary_fn(doc_tag: Option<&str>) -> proc_macro2::TokenStream {
    let doc_tag = doc_tag.map(|tag| quote::quote!(#[doc = #tag]));
    quote::quote! {
        /// @param values Numbers to summarise.
        #doc_tag
        fn summary(
            values: Vec<f64>,
            weights: Vec<f64>,
            #[miniextendr(match_arg)] mode: Mode,
            #[miniextendr(choices("low", "high"))] side: &str,
            ...
        ) -> f64 {
            0.0
        }
    }
}

/// The `@param` lines of a tag list; a generated one keeps its
/// [`PARAM_FILLER_MARKER`] prefix.
fn param_lines(tags: &[String]) -> Vec<&str> {
    tags.iter()
        .map(String::as_str)
        .filter(|tag| tag.starts_with("@param") || tag.starts_with(PARAM_FILLER_MARKER))
        .collect()
}

/// The generated lines a standalone function gets for `summary_fn`'s
/// undocumented arguments, each marked for the wrapper registry.
fn summary_filler_lines(mode_placeholder: &str) -> Vec<String> {
    [
        "@param weights (no documentation available)".to_string(),
        format!("@param mode {mode_placeholder}"),
        "@param side One of \"low\", \"high\".".to_string(),
    ]
    .into_iter()
    .map(|line| format!("{PARAM_FILLER_MARKER}{line}"))
    .collect()
}

#[test]
fn fn_param_tags_fill_every_undocumented_argument_on_its_own_page() {
    let (tags, placeholders) = generated_fn_param_tags(summary_fn(None));
    let mode_placeholder = crate::match_arg_keys::param_doc_placeholder("C_pkg_summary", "mode");
    let mut expected = vec!["@param values Numbers to summarise.".to_string()];
    expected.extend(summary_filler_lines(&mode_placeholder));
    assert_eq!(param_lines(&tags), expected, "got {tags:?}");
    assert_eq!(placeholders, [(mode_placeholder, "mode".to_string())]);
}

/// An explicit `&Dots` parameter is `...` in `\usage` wherever it sits, so it
/// gets no `@param rest` filler (that line would document an argument the
/// usage does not have). The formal after it still gets its own.
#[test]
fn fn_param_tags_skip_an_explicit_dots_parameter() {
    for item in [
        quote::quote! {
            /// @param x A number.
            /// @param ... More values.
            fn f(x: i32, rest: &Dots) {}
        },
        quote::quote! {
            /// @param x A number.
            /// @param ... More values.
            fn f(x: i32, rest: &Dots, flag: bool) {}
        },
    ] {
        let (tags, _) = generated_fn_param_tags(item.clone());
        let lines = param_lines(&tags);
        assert!(
            !lines.iter().any(|line| line.contains("@param rest")),
            "{item}: got {tags:?}"
        );
        let has_flag = item.to_string().contains("flag");
        assert_eq!(
            lines
                .iter()
                .any(|line| line.contains("@param flag (no documentation available)")),
            has_flag,
            "{item}: got {tags:?}"
        );
    }
}

/// An `@rdname topic` may name the function's own file-stem page, which only
/// the wrapper registry knows, so the fillers stay, marked, for it to decide.
#[test]
fn fn_param_tags_leave_an_rdname_block_to_the_registry() {
    let (tags, placeholders) = generated_fn_param_tags(summary_fn(Some("@rdname summaries")));
    let mode_placeholder = crate::match_arg_keys::param_doc_placeholder("C_pkg_summary", "mode");
    let mut expected = vec!["@param values Numbers to summarise.".to_string()];
    expected.extend(summary_filler_lines(&mode_placeholder));
    assert_eq!(param_lines(&tags), expected, "got {tags:?}");
    assert_eq!(placeholders, [(mode_placeholder, "mode".to_string())]);
}

#[test]
fn fn_param_tags_leave_arguments_to_the_topic_or_inheritance_source() {
    for doc_tag in [
        "@describeIn summaries Weighted summary.",
        "@inheritParams summaries",
        "@inherit summaries params",
    ] {
        let (tags, placeholders) = generated_fn_param_tags(summary_fn(Some(doc_tag)));
        // The author's own `@param` stays, exactly once; nothing is generated.
        assert_eq!(
            param_lines(&tags),
            ["@param values Numbers to summarise."],
            "`{doc_tag}`: got {tags:?}"
        );
        assert!(
            placeholders.is_empty(),
            "`{doc_tag}`: a match_arg doc placeholder needs its @param line"
        );
    }
}

/// A `call = caller` wrapper's `.call` formal gets its filler after the
/// parameters' lines, under the same rules (#1613): none under `@describeIn`
/// or inherited params, none when the author documented `.call`.
#[test]
fn fn_param_tags_fill_the_call_formal_last() {
    let call_doc = crate::r_wrapper_builder::CallAttribution::Caller
        .param_doc()
        .expect("caller documents its formal");
    let (tags, _) = generated_fn_param_tags_with_call(summary_fn(None), Some(call_doc));
    let mode_placeholder = crate::match_arg_keys::param_doc_placeholder("C_pkg_summary", "mode");
    let mut expected = vec!["@param values Numbers to summarise.".to_string()];
    expected.extend(summary_filler_lines(&mode_placeholder));
    expected.push(format!("{PARAM_FILLER_MARKER}@param .call {call_doc}"));
    assert_eq!(param_lines(&tags), expected, "got {tags:?}");

    for doc_tag in [
        "@describeIn summaries Weighted summary.",
        "@inheritParams summaries",
    ] {
        let (tags, _) =
            generated_fn_param_tags_with_call(summary_fn(Some(doc_tag)), Some(call_doc));
        assert_eq!(
            param_lines(&tags),
            ["@param values Numbers to summarise."],
            "`{doc_tag}`: got {tags:?}"
        );
    }

    let (tags, _) = generated_fn_param_tags_with_call(
        summary_fn(Some("@param .call Where the error points.")),
        Some(call_doc),
    );
    let lines = param_lines(&tags);
    assert!(
        lines.contains(&"@param .call Where the error points."),
        "got {tags:?}"
    );
    assert!(
        !lines
            .iter()
            .any(|line| line.starts_with(PARAM_FILLER_MARKER) && line.contains(".call")),
        "the author's `.call` line keeps its text: got {tags:?}"
    );
}

/// A layered `choices` parameter's generated line names what else the
/// argument takes (#1551, #1599). It is a filler like the others: marked for
/// the registry on the function's own page, dropped under `@describeIn`.
#[test]
fn fn_param_tags_describe_layered_choices() {
    let route_fn = |doc_tag: Option<&str>| {
        let doc_tag = doc_tag.map(|tag| quote::quote!(#[doc = #tag]));
        quote::quote! {
            /// @param dose Amount per administration.
            #doc_tag
            fn route(
                dose: f64,
                #[miniextendr(choices("oral", "bolus"))] route: Either<String, DataFrame>,
                #[miniextendr(choices("low", "high"))] side: Missing<Option<String>>,
            ) {}
        }
    };
    let (tags, _) = generated_fn_param_tags(route_fn(None));
    assert_eq!(
        param_lines(&tags),
        vec![
            "@param dose Amount per administration.".to_string(),
            format!(
                "{PARAM_FILLER_MARKER}@param route One of \"oral\", \"bolus\", or a data frame."
            ),
            format!(
                "{PARAM_FILLER_MARKER}@param side One of \"low\", \"high\", or NULL; omitting the \
                 argument means no choice."
            ),
        ],
        "got {tags:?}"
    );
    let (tags, _) = generated_fn_param_tags(route_fn(Some("@describeIn routes By route.")));
    assert_eq!(
        param_lines(&tags),
        ["@param dose Amount per administration."],
        "got {tags:?}"
    );
}

#[test]
fn describe_in_topic_is_the_first_word() {
    assert_eq!(
        describe_in_topic(&tag_list(&["@describeIn observed Largest\nvalue."])),
        Some("observed")
    );
    assert_eq!(
        describe_in_topic(&tag_list(&["@describeInx observed"])),
        None
    );
    assert_eq!(describe_in_topic(&tag_list(&["@rdname observed"])), None);
}

/// A block joins an author topic through `@describeIn`, or through an
/// `@rdname` naming another page than the framework's default; the page its
/// companion blocks follow is that topic.
#[test]
fn joins_author_topic_and_method_page() {
    let describe = tag_list(&["@describeIn counter_ops Add a step."]);
    let rdname_other = tag_list(&["@rdname counter_ops"]);
    let rdname_class = tag_list(&["@rdname Counter"]);
    let inherit_only = tag_list(&["@inheritParams counter_ops"]);

    for tags in [&describe, &rdname_other] {
        assert!(joins_author_topic(tags, Some("Counter")), "{tags:?}");
        assert!(joins_author_topic(tags, None), "{tags:?}");
        assert_eq!(method_page(tags, "Counter"), "counter_ops");
    }
    assert!(!joins_author_topic(&rdname_class, Some("Counter")));
    assert!(joins_author_topic(&rdname_class, None));
    assert_eq!(method_page(&rdname_class, "Counter"), "Counter");
    // Inheritance fills the arguments but keeps the block on its own page.
    assert!(!joins_author_topic(&inherit_only, Some("Counter")));
    assert!(params_documented_elsewhere(&inherit_only, Some("Counter")));
    assert_eq!(method_page(&inherit_only, "Counter"), "Counter");
}

#[test]
fn order_after_topic_blocks_only_on_an_author_topic() {
    let order_line = format!("#' {ORDER_AFTER_TOPIC_BLOCKS}");
    let pushed = |tags: &[&str]| {
        let mut lines = Vec::new();
        push_order_after_topic_blocks(&mut lines, &tag_list(tags), "Counter");
        lines
    };
    for tags in [
        &["@rdname counter_ops"][..],
        &["@describeIn counter_ops Add a step."][..],
    ] {
        assert_eq!(pushed(tags), std::slice::from_ref(&order_line), "{tags:?}");
    }
    for tags in [
        &[][..],
        &["@rdname Counter"][..],
        &["@inheritParams counter_ops"][..],
        // The author's own order wins; a block without a page needs none.
        &["@rdname counter_ops", "@order 2"][..],
        &["@rdname counter_ops", "@noRd"][..],
    ] {
        assert!(pushed(tags).is_empty(), "{tags:?}");
    }
}

// endregion

// region: multi-line tags keep blank lines and indentation

/// Doc attributes as rustdoc writes `/// text` (one leading space), with an
/// empty `///` for `""`.
fn rustdoc_attrs(lines: &[&str]) -> Vec<syn::Attribute> {
    lines
        .iter()
        .map(|line| {
            let value = if line.is_empty() {
                String::new()
            } else {
                format!(" {line}")
            };
            syn::parse_quote!(#[doc = #value])
        })
        .collect()
}

fn explicit_tags(lines: &[&str]) -> Vec<String> {
    explicit_roxygen_tags_from_attrs(&rustdoc_attrs(lines))
}

#[test]
fn multiline_tags_keep_paragraphs_and_example_indent() {
    let tags = explicit_tags(&[
        "@title Demo",
        "@description First paragraph.",
        "",
        "Second paragraph.",
        "@return A data frame.",
        "",
        "  Its columns are `a` and `b`.",
        "@examples",
        "x <- c(1, 2) |>",
        "  sum()",
    ]);
    assert_eq!(
        tags,
        [
            "@title Demo",
            "@description First paragraph.\n\nSecond paragraph.",
            "@return A data frame.\n\n  Its columns are `a` and `b`.",
            "@examples\nx <- c(1, 2) |>\n  sum()",
        ]
    );
    let mut lines = Vec::new();
    push_roxygen_tags(&mut lines, &tags);
    assert_eq!(
        lines,
        [
            "#' @title Demo",
            "#' @description First paragraph.",
            "#'",
            "#' Second paragraph.",
            "#' @return A data frame.",
            "#'",
            "#'   Its columns are `a` and `b`.",
            "#' @examples",
            "#' x <- c(1, 2) |>",
            "#'   sum()",
        ]
    );
}

#[test]
fn examples_if_keeps_its_code_lines() {
    let tags = explicit_tags(&["@examplesIf interactive()", "f(1) |>", "  g()", "@export"]);
    assert_eq!(
        tags,
        ["@examplesIf interactive()\nf(1) |>\n  g()", "@export"]
    );
}

#[test]
fn wrapped_aliases_join_onto_one_line() {
    let tags = explicit_tags(&["@aliases alpha beta", "  gamma", "", "delta"]);
    assert_eq!(tags, ["@aliases alpha beta gamma delta"]);
}

#[test]
fn trailing_blank_lines_of_a_tag_are_dropped() {
    let tags = explicit_tags(&["@return X.", "", "", "@examples", "f()", "", ""]);
    assert_eq!(tags, ["@return X.", "@examples\nf()"]);
}

#[test]
fn interior_blank_lines_are_kept_inside_examples() {
    let tags = explicit_tags(&["@examples", "a()", "", "", "b()"]);
    assert_eq!(tags, ["@examples\na()\n\n\nb()"]);
}

#[test]
fn param_continuations_keep_their_indent() {
    let tags = explicit_tags(&[
        "@param x First",
        "  second.",
        "@param y One.",
        "",
        "    code(2)",
    ]);
    assert_eq!(
        tags,
        ["@param x First\n  second.", "@param y One.\n\n    code(2)"]
    );
}

#[test]
fn prose_after_a_single_line_tag_is_not_roxygen() {
    let tags = explicit_tags(&["@export", "Rust-only note.", "", "More Rust notes."]);
    assert_eq!(tags, ["@export"]);
}

#[test]
fn title_still_joins_prose_after_a_blank_line() {
    let tags = explicit_tags(&["@title Demo", "", "More."]);
    assert_eq!(tags, ["@title Demo More."]);
}

#[test]
fn bare_rdname_takes_its_topic_then_ends() {
    let tags = explicit_tags(&["@rdname", "", "  shared_topic", "Rust-only prose."]);
    assert_eq!(tags, ["@rdname shared_topic"]);
}

#[test]
fn a_literal_without_the_rustdoc_space_still_starts_a_tag() {
    let attrs: Vec<syn::Attribute> = vec![
        syn::parse_quote!(#[doc = "@title X"]),
        syn::parse_quote!(#[doc = "@examples"]),
        syn::parse_quote!(#[doc = "f()"]),
        syn::parse_quote!(#[doc = "  g()"]),
    ];
    assert_eq!(
        explicit_roxygen_tags_from_attrs(&attrs),
        ["@title X", "@examples\nf()\n g()"]
    );
}

#[test]
fn a_multiline_literal_loses_only_its_common_indent() {
    let attrs: Vec<syn::Attribute> = vec![syn::parse_quote!(
        #[doc = "\n    @examples\n    f() |>\n      g()\n"]
    )];
    assert_eq!(
        explicit_roxygen_tags_from_attrs(&attrs),
        ["@examples\nf() |>\n  g()"]
    );
}

#[test]
fn cfg_between_a_blank_and_the_next_return_line_still_continues_it() {
    // #613: only doc attributes are read, in order.
    let attrs: Vec<syn::Attribute> = vec![
        syn::parse_quote!(#[doc = " @return The value."]),
        syn::parse_quote!(#[doc = ""]),
        syn::parse_quote!(#[cfg(feature = "x")]),
        syn::parse_quote!(#[doc = " More about it."]),
    ];
    assert_eq!(
        explicit_roxygen_tags_from_attrs(&attrs),
        ["@return The value.\n\nMore about it."]
    );
}

#[test]
fn doc_lines_strip_the_rustdoc_lead_per_attribute() {
    let attrs: Vec<syn::Attribute> = vec![
        syn::parse_quote!(#[doc = " one"]),
        syn::parse_quote!(#[doc = "   two"]),
        syn::parse_quote!(#[doc = ""]),
        syn::parse_quote!(#[doc = "   "]),
        syn::parse_quote!(#[cfg(feature = "x")]),
        syn::parse_quote!(#[doc = "\n    a\n      b\n\n    c"]),
        syn::parse_quote!(#[doc(hidden)]),
    ];
    let lines = doc_lines(&attrs);
    let got: Vec<(usize, &str)> = lines.iter().map(|l| (l.attr, l.text.as_str())).collect();
    assert_eq!(
        got,
        [
            (0, "one"),
            (1, "  two"),
            (2, ""),
            (3, ""),
            (5, ""),
            (5, "a"),
            (5, "  b"),
            (5, ""),
            (5, "c"),
        ]
    );
}

#[test]
fn classify_gives_each_line_one_role() {
    use LineRole::*;
    let attrs = rustdoc_attrs(&[
        "Summary.",    // Prose
        "",            // Prose
        "@param x A.", // TagStart
        "",            // TagBody
        "Note: more.", // TagBody
        "@title T",    // TagStart
        "  wrapped",   // TagBody
        "@export",     // TagStart
        "",            // Rustdoc (a rustdoc line follows)
        "Rust only.",  // Rustdoc
        "",            // TagBody (the next tag follows)
        "@rdname",     // TagStart
        "topic",       // TagBody
        "Rust again.", // Rustdoc
        "",            // TagBody (end)
    ]);
    assert_eq!(
        classify(&doc_lines(&attrs)),
        [
            Prose, Prose, TagStart, TagBody, TagBody, TagStart, TagBody, TagStart, Rustdoc,
            Rustdoc, TagBody, TagStart, TagBody, Rustdoc, TagBody,
        ]
    );
}

/// The text of the doc attributes `strip_roxygen_from_attrs` keeps.
fn kept_docs(attrs: &[syn::Attribute]) -> Vec<String> {
    strip_roxygen_from_attrs(attrs)
        .iter()
        .map(|attr| match &attr.meta {
            syn::Meta::NameValue(nv) => match &nv.value {
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit),
                    ..
                }) => lit.value(),
                _ => "<non-literal>".to_string(),
            },
            _ => "<non-doc>".to_string(),
        })
        .collect()
}

#[test]
fn strip_keeps_leading_prose_and_rustdoc_only_lines() {
    let attrs = rustdoc_attrs(&[
        "Summary.",
        "",
        "@return A value.",
        "",
        "Second paragraph of the return value.",
        "@export",
        "",
        "Rust-only note.",
    ]);
    assert_eq!(kept_docs(&attrs), [" Summary.", "", "", " Rust-only note."]);
}

#[test]
fn strip_drops_a_paragraph_joined_onto_the_title() {
    let attrs = rustdoc_attrs(&["@title Demo", "", "More.", "@param x A."]);
    assert!(kept_docs(&attrs).is_empty(), "{:?}", kept_docs(&attrs));
}

#[test]
fn strip_passes_non_doc_attributes_through() {
    let mut attrs = rustdoc_attrs(&["Summary.", "@export"]);
    attrs.insert(1, syn::parse_quote!(#[allow(dead_code)]));
    assert_eq!(kept_docs(&attrs), [" Summary.", "<non-doc>"]);
}

#[test]
fn blank_roxygen_lines_have_no_trailing_space() {
    assert_eq!(roxygen_line(""), "#'");
    assert_eq!(roxygen_line("  "), "#'");
    assert_eq!(roxygen_line("  sum()"), "#'   sum()");
    let tags = vec!["@description A\n\nB".to_string()];
    assert_eq!(format_roxygen_tags(&tags), "#' @description A\n#'\n#' B\n");
    let mut lines = Vec::new();
    push_roxygen_tags(&mut lines, &tags);
    assert_eq!(lines, ["#' @description A", "#'", "#' B"]);
    let mut lines = Vec::new();
    push_roxygen_tags_str(&mut lines, &["@return X\n\nY"]);
    assert_eq!(lines, ["#' @return X", "#'", "#' Y"]);
}

// endregion
