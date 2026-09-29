use super::*;
use crate::r_wrapper_builder::{CallAttribution, DotCallBuilder, RArgumentBuilder};

fn formals(names: &[&str]) -> BTreeSet<String> {
    names.iter().map(|name| name.to_string()).collect()
}

fn qualify(text: &str, names: &[&str]) -> String {
    qualify_shadowed_calls(text, &formals(names)).into_owned()
}

fn parse_inputs(s: &str) -> syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma> {
    let signature: syn::Signature = syn::parse_str(&format!("fn f({s})")).unwrap();
    signature.inputs
}

/// The attributes of a `match_arg` parameter `name` of type `ty`, classified
/// the way the parser does it.
fn choice_attrs(name: &str, ty: &str) -> crate::miniextendr_fn::ParamAttrs {
    let mut attrs = crate::miniextendr_fn::ParamAttrs {
        match_arg: true,
        ..Default::default()
    };
    let ty: syn::Type = syn::parse_str(ty).unwrap();
    crate::miniextendr_fn::classify_choice_param(&mut attrs, name, &ty, false).unwrap();
    attrs
}

// region: the pass

#[test]
fn no_collision_returns_the_text_unchanged() {
    let text = "f <- function(x, n) {\n  if (!isTRUE(length(x) == 1L)) stop(\"x\")\n  c(x, n)\n}";
    let out = qualify_shadowed_calls(text, &formals(&["x", "n"]));
    assert!(matches!(out, Cow::Borrowed(_)));
    assert_eq!(out, text);
    // A colliding formal that the text never calls allocates nothing either.
    let out = qualify_shadowed_calls("f <- function(length) length", &formals(&["length"]));
    assert!(matches!(out, Cow::Borrowed(_)));
}

#[test]
fn colliding_calls_are_qualified() {
    assert_eq!(qualify("length(x)", &["length"]), "base::length(x)");
    assert_eq!(qualify("length (x)", &["length"]), "base::length (x)");
    assert_eq!(
        qualify("isTRUE(length(x) == 1L) && length(y)", &["length"]),
        "isTRUE(base::length(x) == 1L) && base::length(y)"
    );
    // Only the names that collide.
    assert_eq!(
        qualify("c(length(x), list(y))", &["length", "list", "x"]),
        "c(base::length(x), base::list(y))"
    );
}

#[test]
fn non_calls_and_other_names_are_left_alone() {
    for text in [
        "base::length(x)",
        "base:::length(x)",
        "self$length(x)",
        "self$ length(x)",
        "obj@length(x)",
        "length.MyClass(x)",
        "xlength(x)",
        "x_length(x)",
        "ålength(x)",
        "\"length(x)\"",
        "'length(x)'",
        "\"a \\\" length(x)\"",
        "r\"(length(x))\"",
        "R\"[length(x)]\"",
        "r\"-{length(x) }\" }-\"",
        "r'(length(x))'",
        "# length(x)",
        "#' @examples length(x)",
        "`length`(x)",
        "length <- 2",
        "f(length = 2)",
        "function(length) NULL",
        "x %length% y",
        "1length(x)",
    ] {
        assert_eq!(qualify(text, &["length"]), text, "{text}");
    }
    // Text after a skipped region is still lexed.
    assert_eq!(
        qualify("\"(\" # length(\nlength(x)", &["length"]),
        "\"(\" # length(\nbase::length(x)"
    );
    assert_eq!(
        qualify("r\"(x)\"; length(x)", &["length"]),
        "r\"(x)\"; base::length(x)"
    );
}

#[test]
fn replacement_calls_are_qualified() {
    assert_eq!(
        qualify("class(x) <- \"a\"", &["class"]),
        "base::class(x) <- \"a\""
    );
    assert_eq!(
        qualify("names(x) <- nm", &["names"]),
        "base::names(x) <- nm"
    );
}

#[test]
fn choices_placeholder_counts_as_a_call_to_c() {
    let placeholder = crate::match_arg_keys::choices_placeholder("C_f", "mode");
    let text = format!(
        "f <- function(mode = {placeholder}, c) {{\n  mode <- .miniextendr_match_arg(mode, {placeholder}, \"mode\")\n}}"
    );
    assert_eq!(
        qualify(&text, &["mode", "c"]),
        format!(
            "f <- function(mode = base::{placeholder}, c) {{\n  mode <- .miniextendr_match_arg(mode, base::{placeholder}, \"mode\")\n}}"
        )
    );
    assert_eq!(qualify(&text, &["mode"]), text);
}

// endregion

// region: one test per generated site

#[test]
fn type_guards_call_the_base_length() {
    let inputs = parse_inputs("overwrite: bool, length: f64");
    let guards = crate::r_preconditions::build_precondition_checks(
        &inputs,
        &Default::default(),
        &Default::default(),
    )
    .guards(None)
    .join("\n");
    let out = qualify(&guards, &["overwrite", "length"]);
    assert_eq!(
        out,
        "if (!isTRUE(is.logical(overwrite))) .miniextendr_arg_error(\"overwrite\", \"must be logical\")\n\
         if (!isTRUE(base::length(overwrite) == 1L)) .miniextendr_arg_error(\"overwrite\", \"must have length 1\")\n\
         if (!isTRUE(is.double(length))) .miniextendr_arg_error(\"length\", \"must be double\")\n\
         if (!isTRUE(base::length(length) == 1L)) .miniextendr_arg_error(\"length\", \"must have length 1\")"
    );
}

#[test]
fn choice_formal_default_calls_the_base_c() {
    let attrs = choice_attrs("mode", "Mode");
    let choices = "c(\"fast\", \"slow\")";
    let text = format!(
        "function(mode = {}, c) {{\n  {}\n}}",
        attrs.choice_formal(choices),
        CallAttribution::Wrapper.match_arg_statement("mode", choices, &attrs)
    );
    assert_eq!(
        qualify(&text, &["mode", "c"]),
        "function(mode = base::c(\"fast\", \"slow\"), c) {\n  \
         mode <- .miniextendr_match_arg(mode, base::c(\"fast\", \"slow\"), \"mode\")\n}"
    );
}

#[test]
fn missing_forwarding_calls_the_base_missing_and_quote() {
    let inputs = parse_inputs("x: Missing<f64>, missing: Missing<SEXP>, quote: Missing<SEXP>");
    let call = DotCallBuilder::new("C_f")
        .with_args(&RArgumentBuilder::new(&inputs).build_call_args_vec())
        .build();
    assert_eq!(
        qualify(&call, &["x", "missing", "quote"]),
        ".Call(C_f, .call = sys.call(), \
         if (base::missing(x)) base::quote(expr=) else x, \
         if (base::missing(missing)) base::quote(expr=) else missing, \
         if (base::missing(quote)) base::quote(expr=) else quote)"
    );
}

#[test]
fn condition_check_after_call_uses_the_base_functions() {
    let check = crate::method_return_builder::condition_check_lines("").join("\n");
    assert_eq!(
        qualify(&check, &["attr", "inherits", "isTRUE", "return"]),
        "if (base::inherits(.val, \"rust_condition_value\") && \
         base::isTRUE(base::attr(.val, \"__rust_condition__\"))) \
         base::return(.miniextendr_raise_condition(.val, sys.call()))"
    );
}

#[test]
fn dots_forwarding_calls_the_base_list() {
    let inputs = parse_inputs("list: Missing<SEXP>, _dots: &Dots");
    let call = DotCallBuilder::new("C_f")
        .with_args(&RArgumentBuilder::new(&inputs).build_call_args_vec())
        .build();
    assert_eq!(
        qualify(&call, &["list"]),
        ".Call(C_f, .call = sys.call(), \
         if (missing(list)) quote(expr=) else list, base::list(...))"
    );
}

/// A standalone wrapper with `length` and `c` formals, assembled from the
/// builders the generator uses, after the pass.
#[test]
fn snapshot_shadowed_formals() {
    let inputs = parse_inputs("overwrite: bool, length: f64, mode: Mode, c: Missing<SEXP>");
    let placeholder = crate::match_arg_keys::choices_placeholder("C_shadowed", "mode");
    let r_formals = RArgumentBuilder::new(&inputs)
        .with_defaults(
            [
                ("overwrite".to_string(), "FALSE".to_string()),
                ("mode".to_string(), placeholder.clone()),
            ]
            .into(),
        )
        .build_formals();
    let mut body = crate::r_preconditions::build_precondition_checks(
        &inputs,
        &["mode".to_string()].into(),
        &Default::default(),
    )
    .guards(None);
    body.push(CallAttribution::Wrapper.match_arg_statement(
        "mode",
        &placeholder,
        &choice_attrs("mode", "Mode"),
    ));
    body.push(format!(
        ".val <- {}",
        DotCallBuilder::new("C_shadowed")
            .with_args(&RArgumentBuilder::new(&inputs).build_call_args_vec())
            .build()
    ));
    body.extend(crate::method_return_builder::condition_check_lines(""));
    body.push(".val".to_string());
    let wrapper = format!(
        "#' @param length Length, `length(x)` in the docs is left alone.\n\
         shadowed <- function({r_formals}) {{\n  {}\n}}",
        body.join("\n  ")
    );
    let names = formal_names([&inputs]);
    insta::assert_snapshot!(qualify_shadowed_calls(&wrapper, &names));
}

// endregion

// region: formals

#[test]
fn formal_names_pool_signatures_and_skip_receivers_and_dots() {
    let method = parse_inputs("&self, _length: f64, r#c: i32, dots: &Dots");
    let other = parse_inputs("list: i32");
    assert_eq!(
        formal_names([&method, &other]),
        formals(&["c", "length", "list"])
    );
    // The dots are skipped wherever they sit; the formal after them counts.
    let mid_dots = parse_inputs("&self, n: i32, rest: &Dots, r#c: bool");
    assert_eq!(formal_names([&mid_dots]), formals(&["c", "n"]));
}

/// Formals come from Rust identifiers through `normalize_r_arg_string`, which
/// never produces a `.`. That is why dotted callees need no qualification.
#[test]
fn normalized_formals_never_contain_a_dot() {
    for rust in ["_x", "__x", "_", "x_y", "__"] {
        let formal = crate::r_wrapper_builder::normalize_r_arg_string(rust);
        assert!(!formal.contains('.'), "{rust} -> {formal}");
    }
    for rust in ["r#type", "r#in", "_r"] {
        let ident: syn::Ident = syn::parse_str(rust).unwrap();
        let formal = crate::r_wrapper_builder::normalize_r_arg_ident(&ident).to_string();
        assert!(!formal.contains('.'), "{rust} -> {formal}");
    }
}

// endregion

// region: drift

/// Keywords that look like calls.
const KEYWORDS: &[&str] = &["function", "if", "for", "while", "repeat"];

/// The text that goes through the pass: every macro snapshot of generated
/// wrapper text (minus the insta front matter), and the segments of the
/// tracked cross-package wrapper files that the `#[miniextendr]` fn, impl and
/// trait impl generators wrote. The registry preamble and the derive segments
/// are not processed, so they are left out.
fn processed_text() -> Vec<(String, String)> {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let mut out = Vec::new();
    for dir in [
        "src/miniextendr_impl/snapshots",
        "src/r_wrapper_builder/snapshots",
    ] {
        let dir = std::path::Path::new(manifest).join(dir);
        let mut paths: Vec<_> = std::fs::read_dir(&dir)
            .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "snap"))
            .collect();
        paths.sort();
        for path in paths {
            let snap = std::fs::read_to_string(&path).unwrap();
            let body = snap
                .match_indices("---\n")
                .nth(1)
                .map_or(snap.as_str(), |(n, m)| &snap[n + m.len()..]);
            out.push((path.display().to_string(), body.to_string()));
        }
    }
    for file in [
        "../tests/cross-package/producer.pkg/R/producer.pkg-wrappers.R",
        "../tests/cross-package/consumer.pkg/R/consumer.pkg-wrappers.R",
    ] {
        let path = std::path::Path::new(manifest).join(file);
        let text =
            std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let mut segment: Option<String> = None;
        for line in text.lines() {
            let starts = line.starts_with("# Generated from Rust fn ")
                || line.starts_with("# Generated from Rust impl ")
                || line.starts_with("# Generated by #[miniextendr] impl ");
            let ends = starts || line.starts_with("# Generated ") || line == "# nocov end";
            if ends && let Some(done) = segment.take() {
                out.push((path.display().to_string(), done));
            }
            if starts {
                segment = Some(String::new());
            }
            if let Some(segment) = segment.as_mut() {
                segment.push_str(line);
                segment.push('\n');
            }
        }
        assert!(segment.is_none(), "{}: no `# nocov end`", path.display());
    }
    out
}

/// Every undotted name that processed text calls without a namespace is a
/// [`BASE_CALLEES`] entry, a keyword, or a name the same segment binds with
/// `<-`. A new unqualified callee in generated code fails here until it is
/// added to the table, so a formal of that name cannot shadow it unnoticed.
#[test]
fn generated_callees_are_listed() {
    let mut unlisted = Vec::new();
    let mut listed_seen = BTreeSet::new();
    for (origin, text) in processed_text() {
        let idents = identifiers(&text);
        let bound: BTreeSet<&str> = idents
            .iter()
            .filter(|ident| {
                let rest = text[ident.end..].trim_start_matches([' ', '\t']);
                rest.starts_with("<-") || rest.starts_with("<<-")
            })
            .map(|ident| &text[ident.clone()])
            .collect();
        for ident in &idents {
            let name = &text[ident.clone()];
            if name.contains('.') || !is_call(&text, ident.end) || is_accessed(&text, ident.start) {
                continue;
            }
            if BASE_CALLEES.contains(&name) {
                listed_seen.insert(name.to_string());
                continue;
            }
            if KEYWORDS.contains(&name) || bound.contains(name) {
                continue;
            }
            unlisted.push(format!("{origin}: `{name}(`"));
        }
    }
    // The inputs hold real wrapper text: the guards, the check after
    // `.Call()`, forwarding and class construction all show up.
    for core in [
        "isTRUE", "length", "inherits", "attr", "return", "missing", "list", "c",
    ] {
        assert!(
            listed_seen.contains(core),
            "no `{core}(` in the processed text"
        );
    }
    unlisted.sort();
    unlisted.dedup();
    assert!(
        unlisted.is_empty(),
        "generated wrapper text calls names missing from BASE_CALLEES:\n{}",
        unlisted.join("\n")
    );
}

#[test]
fn base_callees_are_sorted_undotted_and_unique() {
    let mut sorted = BASE_CALLEES.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted, BASE_CALLEES);
    assert!(BASE_CALLEES.iter().all(|name| !name.contains('.')));
}

// endregion
