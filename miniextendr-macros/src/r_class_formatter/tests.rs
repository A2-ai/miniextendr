#[test]
fn env_parameter_details_end_examples_blocks() {
    let type_ident: syn::Ident = syn::parse_quote!(Example);
    for examples in [
        "@examples\nExample$new(1L)",
        "@examplesIf TRUE\nExample$new(1L)",
    ] {
        let tags = vec![
            "@param value Integer input.".to_owned(),
            examples.to_owned(),
        ];
        let docs = super::MethodDocBuilder::new("Example", "new", &type_ident, &tags)
            .with_params_as_details()
            .build()
            .join("\n");
        assert!(
            docs.contains("Example$new(1L)\n#' @details\n#' \\describe{"),
            "{docs}"
        );
    }
}

/// An env method block drops the author's bare `@export`: roxygen2 would
/// write `export("Example$new")`, which R refuses to load. The other tags,
/// an `@export <symbol>` among them, are forwarded.
#[test]
fn env_method_drops_bare_export() {
    let type_ident: syn::Ident = syn::parse_quote!(Example);
    let tags: Vec<String> = [
        "@description Make one.",
        "@export",
        "@export example_helper",
        "@param value Integer input.",
    ]
    .map(str::to_owned)
    .to_vec();
    let docs = super::MethodDocBuilder::new("Example", "new", &type_ident, &tags)
        .with_name_prefix("$")
        .with_params_as_details()
        .build()
        .join("\n");
    assert!(
        !docs.lines().any(|l| l.trim_end() == "#' @export"),
        "{docs}"
    );
    for line in [
        "#' @description Make one.",
        "#' @export example_helper",
        "#' @name Example$new",
    ] {
        assert_eq!(
            docs.lines().filter(|l| *l == line).count(),
            1,
            "`{line}`: {docs}"
        );
    }
}

/// A method whose own tags join or inherit a topic gets no generated
/// `@param` lines (#1590); its own `@param` stays. The class-page default
/// the builder appends is not an author tag, so a method on the class page
/// keeps the filler for each undocumented argument, also when the author
/// names that page with a redundant `@rdname Counter`.
#[test]
fn method_param_filler_only_on_the_class_page() {
    let type_ident: syn::Ident = syn::parse_quote!(Counter);
    let build = |tags: &[&str]| {
        let tags: Vec<String> = tags.iter().map(|t| t.to_string()).collect();
        super::MethodDocBuilder::new("Counter", "add", &type_ident, &tags)
            .with_r_params("by, times = 1L")
            .build()
            .join("\n")
    };

    // No author page tag, or one naming the class page itself.
    for tags in [
        &["@param by Step size."][..],
        &["@param by Step size.", "@rdname Counter"][..],
    ] {
        let class_page = build(tags);
        assert!(class_page.contains("#' @rdname Counter"), "{class_page}");
        assert!(
            class_page.contains("#' @param times (undocumented)"),
            "{class_page}"
        );
    }

    for tag in ["@rdname counter_ops", "@inheritParams counter_ops"] {
        let docs = build(&["@param by Step size.", tag]);
        assert_eq!(
            docs.matches("#' @param ").count(),
            1,
            "`{tag}`: only the author's @param, got:\n{docs}"
        );
        assert!(docs.contains("#' @param by Step size."), "{docs}");
    }
}

/// A method-level `@describeIn` block carries neither `@name` nor
/// `@rdname` (roxygen2: "@describeIn can not be used with @name") nor a
/// structural title, sorts after the destination's own block, and leaves
/// undocumented arguments to it. An `@rdname` naming another page sorts
/// last too; the class page keeps the defaults.
#[test]
fn method_describe_in_drops_name_and_rdname() {
    let type_ident: syn::Ident = syn::parse_quote!(Counter);
    let order = format!("#' {}", crate::roxygen::ORDER_AFTER_TOPIC_BLOCKS);
    let build = |tags: &[&str]| {
        let tags: Vec<String> = tags.iter().map(|t| t.to_string()).collect();
        super::MethodDocBuilder::new("Counter", "add", &type_ident, &tags)
            .with_r_params("by, times = 1L")
            .with_r_name("add.Counter".to_string())
            .build()
    };

    let described = build(&[
        "@describeIn counter_ops Add a step.",
        "@param by Step size.",
    ]);
    assert_eq!(
        described,
        [
            "#' @describeIn counter_ops Add a step.",
            "#' @param by Step size.",
            order.as_str(),
        ],
        "{described:#?}"
    );

    let split = build(&["@rdname counter_ops"]);
    assert!(
        split.contains(&"#' @name add.Counter".to_string()),
        "{split:#?}"
    );
    assert!(
        split.contains(&"#' @title add.Counter".to_string()),
        "{split:#?}"
    );
    assert!(split.contains(&order), "{split:#?}");

    let class_page = build(&[]);
    assert!(
        class_page.contains(&crate::miniextendr_impl::member_default(
            "#' @rdname Counter"
        )),
        "{class_page:#?}"
    );
    assert!(!class_page.contains(&order), "{class_page:#?}");
}

use super::ClassDocBuilder;

#[test]
fn test_method_context_static_call_no_args() {
    // This is a unit test for the static_call method
    // We'd need a mock ParsedMethod to test fully, but we can test the logic
    let call = ".Call(C_Test, .call = sys.call())";
    assert!(call.contains(".Call"));
}

/// Audit A10: a class-level `#[miniextendr(noexport)]` (without `internal`)
/// must produce no Rd contribution at all — no `@title`/`@name`/`@rdname`/
/// `@export` — same as a user-written `@noRd`. Before the fix, `noexport`
/// only suppressed `@export`, leaving the class fully documented (with an
/// alias) minus the export line.
#[test]
fn test_class_noexport_suppresses_all_roxygen() {
    let type_ident: syn::Ident = syn::parse_str("Foo").unwrap();
    let doc_tags: Vec<String> = vec![];
    let lines = ClassDocBuilder::new("Foo", &type_ident, &doc_tags, "R6")
        .with_export_control(false, true)
        .render();
    let joined = lines.join("\n");

    assert!(
        lines.iter().any(|l| l == "#' @noRd"),
        "noexport should emit @noRd, got:\n{}",
        joined
    );
    assert!(
        !joined.contains("@title") && !joined.contains("@name") && !joined.contains("@rdname"),
        "noexport should suppress @title/@name/@rdname entirely, got:\n{}",
        joined
    );
    assert!(
        !joined.contains("@export"),
        "noexport should suppress @export, got:\n{}",
        joined
    );
}

/// Companion: `#[miniextendr(internal)]` keeps the class documented (under
/// `@keywords internal`) — it still contributes `@title`/`@name`/`@rdname`
/// so it lands on a real help page, just unexported.
#[test]
fn test_class_internal_still_documented() {
    let type_ident: syn::Ident = syn::parse_str("Foo").unwrap();
    let doc_tags: Vec<String> = vec![];
    let lines = ClassDocBuilder::new("Foo", &type_ident, &doc_tags, "R6")
        .with_export_control(true, false)
        .render();
    let joined = lines.join("\n");

    assert!(
        !lines.iter().any(|l| l == "#' @noRd"),
        "internal should NOT emit @noRd (stays documented), got:\n{}",
        joined
    );
    assert!(
        joined.contains("@keywords internal"),
        "internal should add @keywords internal, got:\n{}",
        joined
    );
    assert!(
        joined.contains("@title") && joined.contains("@name") && joined.contains("@rdname"),
        "internal should still emit @title/@name/@rdname, got:\n{}",
        joined
    );
    assert!(
        !joined.contains("#' @export"),
        "internal should suppress @export, got:\n{}",
        joined
    );
}

/// Neither flag set: normal fully-documented, exported class.
#[test]
fn test_class_no_flags_fully_documented_and_exported() {
    let type_ident: syn::Ident = syn::parse_str("Foo").unwrap();
    let doc_tags: Vec<String> = vec![];
    let lines = ClassDocBuilder::new("Foo", &type_ident, &doc_tags, "R6")
        .with_export_control(false, false)
        .render();
    let joined = lines.join("\n");

    assert!(!joined.contains("@noRd"));
    assert!(!joined.contains("@keywords internal"));
    assert!(joined.contains("@title") && joined.contains("@name") && joined.contains("@rdname"));
    assert!(joined.contains("#' @export"));
}

/// The auto title names the class system, and has one space between words
/// when there is no label (Env classes).
#[test]
fn test_class_title_spacing() {
    let type_ident: syn::Ident = syn::parse_str("Foo").unwrap();
    let doc_tags: Vec<String> = vec![];
    for (label, title) in [
        ("R6", "#' @title Foo R6 Class"),
        ("", "#' @title Foo Class"),
    ] {
        let lines = ClassDocBuilder::new("Foo", &type_ident, &doc_tags, label).render();
        assert_eq!(
            lines.iter().filter(|l| l.starts_with("#' @title")).count(),
            1
        );
        assert!(lines.iter().any(|l| l == title), "{lines:?}");
    }
}

fn tags_of(tags: &[&str]) -> Vec<String> {
    tags.iter().map(|t| t.to_string()).collect()
}

fn choice_docs(entries: &[(&str, &str)]) -> std::collections::HashMap<String, String> {
    entries
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

/// An S3 / vctrs constructor gets one generated `@param` per formal its
/// tags leave undocumented, before `@export`: the choice text for a
/// choice parameter, `(undocumented)` otherwise. An author `@param` is
/// kept and not repeated; `@noRd`, `noexport` and `@inheritParams` get
/// none.
#[test]
fn class_ctor_params_fill_undocumented_formals() {
    let type_ident: syn::Ident = syn::parse_quote!(Point);
    let docs = choice_docs(&[("mode", "One of \"a\", \"b\".")]);
    let params = "x, y = 1, mode = c(\"a\", \"b\"), ...";
    let build = |tags: &[&str], internal: bool, noexport: bool| {
        let tags = tags_of(tags);
        ClassDocBuilder::new("Point", &type_ident, &tags, "S3")
            .with_ctor_params(params, &docs)
            .with_export_control(internal, noexport)
            .render()
    };

    let lines = build(&["@param x The x coordinate."], false, false);
    assert_eq!(
        lines,
        [
            "#' @title Point S3 Class",
            "#' @name Point",
            "#' @rdname Point",
            "#' @param x The x coordinate.",
            "#' @param y (undocumented)",
            "#' @param mode One of \"a\", \"b\".",
            "#' @export",
        ],
        "{lines:#?}"
    );

    let internal = build(&[], true, false);
    assert!(
        internal.contains(&"#' @param x (undocumented)".to_string()),
        "{internal:#?}"
    );

    for (tags, noexport) in [
        (&["@noRd"][..], false),
        (&[][..], true),
        (&["@inheritParams point_ops"][..], false),
    ] {
        let lines = build(tags, false, noexport);
        assert!(
            !lines.iter().any(|l| l.starts_with("#' @param")),
            "{tags:?} noexport={noexport}: {lines:#?}"
        );
    }
}

fn env_docs(
    tags: &[&str],
    params: &str,
    docs: &std::collections::HashMap<String, String>,
) -> Vec<String> {
    let type_ident: syn::Ident = syn::parse_quote!(Example);
    let tags = tags_of(tags);
    super::MethodDocBuilder::new("Example", "new", &type_ident, &tags)
        .with_name_prefix("$")
        .with_params_as_details()
        .with_r_params(params)
        .with_choice_param_docs(docs)
        .build()
}

/// An env method (no `\usage`) lists the author's `@param` items and then
/// the choice items in one `\describe{}`, with no `(undocumented)` item
/// and no `@param` line (roxygen2 would report "Documented arguments not
/// in \usage"). Unless a `@description` / `@details` tag directly
/// precedes the list, a labelled `@description` lead-in and a blank line
/// open it.
#[test]
fn env_params_share_one_describe_list() {
    let docs = choice_docs(&[("mode", "One of \"a\", \"b\".")]);
    let params = "n, plain, mode = c(\"a\", \"b\")";

    for tags in [&[][..], &["@param n Amount."][..]] {
        let lines = env_docs(tags, params, &docs);
        assert!(!lines.iter().any(|l| l.contains("@param")), "{lines:#?}");
        assert!(
            !lines.iter().any(|l| l.contains("undocumented")),
            "{lines:#?}"
        );
        assert_eq!(lines.iter().filter(|l| *l == "#' \\describe{").count(), 1);
        let joined = lines.join("\n");
        assert!(
            joined.starts_with(
                "#' @description Arguments of \\code{Example$new()}:\n#'\n#' \\describe{\n"
            ),
            "{joined}"
        );
        assert!(
            joined.contains("#'   \\item{\\code{mode}}{One of \"a\", \"b\".}\n#' }"),
            "{joined}"
        );
    }
    let with_author = env_docs(&["@param n Amount."], params, &docs).join("\n");
    assert!(
        with_author.contains(
            "#'   \\item{\\code{n}}{Amount.}\n#'   \\item{\\code{mode}}{One of \"a\", \"b\".}"
        ),
        "author item first: {with_author}"
    );

    // A prose tag before the list keeps it in that section; after any
    // other tag the lead-in opens it, or the list would land in that
    // tag's section (`\value`, `\seealso`, the title).
    let lead_in = "#' @description Arguments of \\code{Example$new()}:\n#'\n#' \\describe{";
    for (last, sep) in [
        (
            "@description Plan it.",
            "#' @description Plan it.\n#'\n#' \\describe{".to_string(),
        ),
        (
            "@details More.",
            "#' @details More.\n#'\n#' \\describe{".to_string(),
        ),
        ("@seealso other", format!("#' @seealso other\n{lead_in}")),
        (
            "@return A value.",
            format!("#' @return A value.\n{lead_in}"),
        ),
        ("@title Mine", format!("#' @title Mine\n{lead_in}")),
    ] {
        let joined = env_docs(&[last], params, &docs).join("\n");
        assert!(joined.starts_with(&sep), "`{last}`: {joined}");
        assert_eq!(
            joined.matches("Arguments of").count(),
            usize::from(sep.contains("Arguments of")),
            "`{last}`: {joined}"
        );
    }
    // Prose and a trailing `@return`: the lead-in is a second
    // `@description`, which roxygen2 merges into the first.
    let joined = env_docs(
        &["@description Plan it.", "@return A value."],
        params,
        &docs,
    )
    .join("\n");
    assert!(
        joined.starts_with(&format!(
            "#' @description Plan it.\n#' @return A value.\n{lead_in}"
        )),
        "{joined}"
    );

    // An author `@param` on the choice parameter replaces its choice item.
    let joined = env_docs(&["@param mode Mine."], params, &docs).join("\n");
    assert_eq!(
        joined.matches("\\item{\\code{mode}}").count(),
        1,
        "{joined}"
    );
    assert!(
        joined.contains("#'   \\item{\\code{mode}}{Mine.}"),
        "{joined}"
    );
    assert!(!joined.contains("One of"), "{joined}");

    // No item and no tag: nothing at all; a plain formal gets no item.
    let empty = std::collections::HashMap::new();
    let nothing = env_docs(&[], "n, plain", &empty);
    assert!(
        nothing
            .iter()
            .all(|l| !l.contains("describe") && !l.contains("@description")),
        "{nothing:#?}"
    );
}

/// Tags the caller does not forward still send the arguments elsewhere
/// (`@inheritParams`), so no choice item is generated for them; the label
/// is used verbatim, with `%` and `\` escaped for Rd.
#[test]
fn describe_params_lines_forward_and_label() {
    let docs = choice_docs(&[("mode", "One of \"a\", \"b\".")]);
    let tags = tags_of(&["@inheritParams family"]);
    let lines = super::describe_params_lines(
        &tags,
        |tag| !tag.starts_with("@inheritParams"),
        "mode = c(\"a\", \"b\")",
        "Example",
        Some(&docs),
        "Example$Trait$pick",
    );
    assert!(lines.is_empty(), "{lines:#?}");

    let lines = super::describe_params_lines(
        &[],
        |_| true,
        "mode = c(\"a\", \"b\")",
        "Example",
        Some(&docs),
        "Example$%op%",
    );
    assert_eq!(
        lines[0], "#' @description Arguments of \\code{Example$\\%op\\%()}:",
        "{lines:#?}"
    );
}

/// Every line of a wrapped author `@param` stays inside the roxygen
/// block: the continuation line keeps its `#' ` lead instead of landing
/// in wrappers.R as bare R code.
#[test]
fn env_wrapped_param_stays_in_roxygen_block() {
    let empty = std::collections::HashMap::new();
    let lines = env_docs(&["@param n Amount\nto add."], "n", &empty);
    assert!(
        lines.iter().all(|l| l
            .strip_prefix(crate::miniextendr_impl::CLASS_MEMBER_DEFAULT_PREFIX)
            .unwrap_or(l)
            .starts_with("#'")),
        "{lines:#?}"
    );
    let joined = lines.join("\n");
    assert!(
        joined.contains("#'   \\item{\\code{n}}{Amount\n#' to add.}"),
        "{joined}"
    );
}
