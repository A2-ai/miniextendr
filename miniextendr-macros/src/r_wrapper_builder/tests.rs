use super::*;

fn parse_inputs(s: &str) -> syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma> {
    let signature: syn::Signature = syn::parse_str(&format!("fn test({})", s)).unwrap();
    signature.inputs
}

#[test]
fn test_normalize_arg_ident() {
    // Leading underscores are stripped
    let ident = syn::Ident::new("_x", proc_macro2::Span::call_site());
    assert_eq!(normalize_r_arg_ident(&ident).to_string(), "x");

    let ident = syn::Ident::new("__private", proc_macro2::Span::call_site());
    assert_eq!(normalize_r_arg_ident(&ident).to_string(), "private");

    let ident = syn::Ident::new("value", proc_macro2::Span::call_site());
    assert_eq!(normalize_r_arg_ident(&ident).to_string(), "value");
}

#[test]
fn test_basic_formals() {
    let inputs = parse_inputs("x: i32, y: f64");
    let builder = RArgumentBuilder::new(&inputs);
    assert_eq!(builder.build_formals(), "x, y");
}

#[test]
fn test_unit_type_default() {
    // `_unused` becomes `unused` (underscore stripped), unit type gets NULL default
    let inputs = parse_inputs("x: i32, _unused: ()");
    let builder = RArgumentBuilder::new(&inputs);
    assert_eq!(builder.build_formals(), "x, unused = NULL");
}

#[test]
fn test_dots() {
    // In R, `...` takes no name or default: the Rust binding never shows.
    for sig in [
        "x: i32, _dots: &Dots",
        "x: i32, dots: &Dots",
        "x: i32, args: &::miniextendr_api::dots::Dots",
    ] {
        let inputs = parse_inputs(sig);
        let builder = RArgumentBuilder::new(&inputs);
        assert_eq!(builder.build_formals(), "x, ...", "{sig}");
        assert_eq!(builder.build_call_args(), "x, list(...)", "{sig}");
    }
}

/// The `&Dots` parameter is `...` / `list(...)` at its own position; the
/// formals after it keep their defaults and `Missing<T>` forwarding.
#[test]
fn test_dots_at_any_position() {
    let build = |sig: &str, defaults: &[(&str, &str)]| {
        let inputs = parse_inputs(sig);
        let defaults = defaults
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        let builder = RArgumentBuilder::new(&inputs).with_defaults(defaults);
        (builder.build_formals(), builder.build_call_args())
    };
    assert_eq!(
        build(
            "x: i32, rest: &Dots, overwrite: bool",
            &[("overwrite", "FALSE")]
        ),
        (
            "x, ..., overwrite = FALSE".to_string(),
            "x, list(...), overwrite".to_string()
        )
    );
    assert_eq!(
        build("rest: &Dots, x: i32", &[]),
        ("..., x".to_string(), "list(...), x".to_string())
    );
    assert_eq!(
        build("x: i32, rest: &Dots, p: Missing<f64>", &[]),
        (
            "x, ..., p".to_string(),
            "x, list(...), if (missing(p)) quote(expr=) else p".to_string()
        )
    );
}

/// The dots index counts the receiver, as both build loops do: an
/// off-by-one would put `...` on the wrong formal.
#[test]
fn test_dots_position_after_receiver() {
    let inputs = parse_inputs("&self, n: i32, rest: &Dots, flag: bool");
    let builder = RArgumentBuilder::new(&inputs);
    assert_eq!(builder.build_formals(), "n, ..., flag");
    assert_eq!(builder.build_call_args(), "n, list(...), flag");
    let builder = RArgumentBuilder::new(&inputs).skip_first();
    assert_eq!(builder.build_formals(), "n, ..., flag");
    assert_eq!(builder.build_call_args(), "n, list(...), flag");

    let inputs = parse_inputs("x: i32, n: i32, rest: &Dots, flag: bool");
    let builder = RArgumentBuilder::new(&inputs).skip_first();
    assert_eq!(builder.build_formals(), "n, ..., flag");
    assert_eq!(builder.build_call_args(), "n, list(...), flag");
}

/// `check_r_formals` skips the dots wherever they sit (their formal is
/// `...`) and still checks the formals after them.
#[test]
fn check_r_formals_skips_dots_at_any_position() {
    check_r_formals(&parse_inputs("x: i32, r#in: &Dots, flag: bool"), &[]).expect("accepted");
    check_r_formals(&parse_inputs("r#if: &Dots, x: i32"), &[]).expect("accepted");
    let err = check_r_formals(&parse_inputs("x: i32, rest: &Dots, r#in: i32"), &[])
        .expect_err("`in` after the dots is still refused")
        .to_string();
    assert!(
        err.contains("becomes the R argument `in`, which is an R reserved word"),
        "{err}"
    );
}

#[test]
fn test_skip_first() {
    let inputs = parse_inputs("&self, x: i32, y: f64");
    let builder = RArgumentBuilder::new(&inputs).skip_first();
    assert_eq!(builder.build_formals(), "x, y");
    assert_eq!(builder.build_call_args(), "x, y");
}

#[test]
fn test_underscore_normalization() {
    // Leading underscores are stripped in R (Rust convention for unused params)
    let inputs = parse_inputs("_x: i32, __private: String");
    let builder = RArgumentBuilder::new(&inputs);
    assert_eq!(builder.build_formals(), "x, private");
}

// DotCallBuilder tests
#[test]
fn test_dot_call_no_args() {
    let call = DotCallBuilder::new("C_Counter__new").build();
    assert_eq!(call, ".Call(C_Counter__new, .call = sys.call())");
}

#[test]
fn test_dot_call_with_self() {
    let call = DotCallBuilder::new("C_Counter__value")
        .with_self("self")
        .build();
    assert_eq!(call, ".Call(C_Counter__value, .call = sys.call(), self)");
}

#[test]
fn test_dot_call_with_self_and_args() {
    let call = DotCallBuilder::new("C_Counter__add")
        .with_self("x")
        .with_args(&["n"])
        .build();
    assert_eq!(call, ".Call(C_Counter__add, .call = sys.call(), x, n)");
}

#[test]
fn test_dot_call_static_with_args() {
    let call = DotCallBuilder::new("C_Counter__from_parts")
        .with_args(&["a", "b", "c"])
        .build();
    assert_eq!(
        call,
        ".Call(C_Counter__from_parts, .call = sys.call(), a, b, c)"
    );
}

#[test]
fn test_dot_call_with_args_str_empty_skips_args() {
    let call = DotCallBuilder::new("C_Counter__new")
        .with_args_str("")
        .build();
    assert_eq!(call, ".Call(C_Counter__new, .call = sys.call())");
}

#[test]
fn test_dot_call_with_args_str_passes_through() {
    let call = DotCallBuilder::new("C_Counter__update")
        .with_self("self")
        .with_args_str("step, verbose")
        .build();
    assert_eq!(
        call,
        ".Call(C_Counter__update, .call = sys.call(), self, step, verbose)"
    );
}

// null_call_attribution tests
#[test]
fn test_dot_call_null_call_no_args() {
    let call = DotCallBuilder::new("C_Type__finalize")
        .null_call_attribution()
        .with_self("private$.ptr")
        .build();
    assert_eq!(call, ".Call(C_Type__finalize, .call = NULL, private$.ptr)");
}

#[test]
fn test_dot_call_null_call_with_args() {
    let call = DotCallBuilder::new("C_Type__deep_clone")
        .null_call_attribution()
        .with_self("private$.ptr")
        .with_args(&["name", "value"])
        .build();
    assert_eq!(
        call,
        ".Call(C_Type__deep_clone, .call = NULL, private$.ptr, name, value)"
    );
}

#[test]
fn test_dot_call_null_call_validator() {
    let call = DotCallBuilder::new("C_Type__validate_prop")
        .null_call_attribution()
        .with_args(&["value"])
        .build();
    assert_eq!(call, ".Call(C_Type__validate_prop, .call = NULL, value)");
}

// RoxygenBuilder tests
#[test]
fn test_roxygen_basic() {
    let tags = RoxygenBuilder::new()
        .name("Counter$increment")
        .rdname("Counter")
        .export()
        .build();
    assert_eq!(
        tags,
        vec![
            "#' @name Counter$increment",
            "#' @rdname Counter",
            "#' @export"
        ]
    );
}

// `.source(...)` only renders when the crate opts in with
// `[package.metadata.miniextendr] source_tags = true` (#1552); the test
// crate has no such table, so the builder drops it.
#[test]
fn test_roxygen_s3_method() {
    let tags = RoxygenBuilder::new()
        .name("value")
        .source("Generated by miniextendr from `impl Counter for MyType`")
        .method("value", "MyType")
        .export()
        .build();
    assert_eq!(
        tags,
        vec!["#' @name value", "#' @method value MyType", "#' @export"]
    );
}

#[test]
fn test_roxygen_s4_method() {
    let tags = RoxygenBuilder::new()
        .name("s4_trait_Counter_value")
        .source("Generated by miniextendr")
        .export_method("s4_trait_Counter_value")
        .build();
    assert_eq!(
        tags,
        vec![
            "#' @name s4_trait_Counter_value",
            "#' @exportMethod s4_trait_Counter_value"
        ]
    );
}

// Missing<T> tests
#[test]
fn test_is_missing_type() {
    let inputs = parse_inputs("x: Missing<i32>");
    let arg = inputs.first().unwrap();
    if let syn::FnArg::Typed(pat_type) = arg {
        assert!(is_missing_type(&pat_type.ty));
    } else {
        panic!("Expected typed argument");
    }
}

#[test]
fn test_is_not_missing_type() {
    let inputs = parse_inputs("x: Option<i32>");
    let arg = inputs.first().unwrap();
    if let syn::FnArg::Typed(pat_type) = arg {
        assert!(!is_missing_type(&pat_type.ty));
    } else {
        panic!("Expected typed argument");
    }
}

#[test]
fn test_missing_type_call_args_inline_sentinel() {
    // The R_MissingArg sentinel must be produced at the argument position —
    // a prelude binding of the sentinel errors on symbol lookup.
    let inputs = parse_inputs("x: i32, y: Missing<f64>, z: String");
    let builder = RArgumentBuilder::new(&inputs);
    assert_eq!(
        builder.build_call_args(),
        "x, if (missing(y)) quote(expr=) else y, z"
    );
}

#[test]
fn test_multiple_missing_type_call_args() {
    let inputs = parse_inputs("a: Missing<i32>, b: f64, c: Missing<String>");
    let builder = RArgumentBuilder::new(&inputs);
    assert_eq!(
        builder.build_call_args(),
        "if (missing(a)) quote(expr=) else a, b, if (missing(c)) quote(expr=) else c"
    );
}

#[test]
fn test_missing_type_formals_clean_signature() {
    // Missing<T> params without user defaults appear as bare formals
    let inputs = parse_inputs("x: i32, y: Missing<f64>");
    let builder = RArgumentBuilder::new(&inputs);
    assert_eq!(builder.build_formals(), "x, y");
}

// region: Insta snapshot tests for builder output stability

#[test]
fn snapshot_dot_call_variations() {
    let mut output = String::new();

    output.push_str("# No args\n");
    output.push_str(&DotCallBuilder::new("C_my_fn").build());
    output.push_str("\n\n# Self only\n");
    output.push_str(
        &DotCallBuilder::new("C_Counter__get")
            .with_self("self")
            .build(),
    );
    output.push_str("\n\n# Self + args\n");
    output.push_str(
        &DotCallBuilder::new("C_Counter__add")
            .with_self("x")
            .with_args(&["n", "verbose"])
            .build(),
    );
    output.push_str("\n\n# Static with args\n");
    output.push_str(
        &DotCallBuilder::new("C_Counter__from_parts")
            .with_args(&["a", "b", "c"])
            .build(),
    );

    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_roxygen_builder_variations() {
    let mut output = String::new();

    output.push_str("# Basic export\n");
    let tags = RoxygenBuilder::new()
        .name("Counter$increment")
        .rdname("Counter")
        .export()
        .build();
    output.push_str(&tags.join("\n"));

    output.push_str("\n\n# S3 method\n");
    let tags = RoxygenBuilder::new()
        .name("get.Counter")
        .source("Generated by miniextendr from `impl Counter`")
        .method("get", "Counter")
        .export()
        .build();
    output.push_str(&tags.join("\n"));

    output.push_str("\n\n# S4 method\n");
    let tags = RoxygenBuilder::new()
        .name("s4_trait_Counter_value")
        .source("Generated by miniextendr")
        .export_method("s4_trait_Counter_value")
        .build();
    output.push_str(&tags.join("\n"));

    output.push_str("\n\n# Title + description\n");
    let tags = RoxygenBuilder::new()
        .title("Widget constructor")
        .description("Creates a new Widget with default settings.")
        .name("Widget")
        .export()
        .build();
    output.push_str(&tags.join("\n"));

    output.push_str("\n\n# Custom tags\n");
    let tags = RoxygenBuilder::new()
        .name("my_fn")
        .custom("@param x A numeric value")
        .custom("@return The squared value")
        .export()
        .build();
    output.push_str(&tags.join("\n"));

    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_formals_and_call_args() {
    let mut output = String::new();

    // Basic scalar args
    output.push_str("# Basic scalars\n");
    let inputs = parse_inputs("x: i32, y: f64, name: String");
    let builder = RArgumentBuilder::new(&inputs);
    output.push_str(&format!("formals: {}\n", builder.build_formals()));
    output.push_str(&format!("call_args: {}\n", builder.build_call_args()));

    // With unit type default
    output.push_str("\n# Unit type default\n");
    let inputs = parse_inputs("x: i32, _unused: ()");
    let builder = RArgumentBuilder::new(&inputs);
    output.push_str(&format!("formals: {}\n", builder.build_formals()));
    output.push_str(&format!("call_args: {}\n", builder.build_call_args()));

    // With dots
    output.push_str("\n# With dots\n");
    let inputs = parse_inputs("x: i32, _dots: &Dots");
    let builder = RArgumentBuilder::new(&inputs);
    output.push_str(&format!("formals: {}\n", builder.build_formals()));
    output.push_str(&format!("call_args: {}\n", builder.build_call_args()));

    // With skip_first (method receiver)
    output.push_str("\n# Skip first (method)\n");
    let inputs = parse_inputs("&self, x: i32, y: f64");
    let builder = RArgumentBuilder::new(&inputs).skip_first();
    output.push_str(&format!("formals: {}\n", builder.build_formals()));
    output.push_str(&format!("call_args: {}\n", builder.build_call_args()));

    // With user defaults
    output.push_str("\n# User defaults\n");
    let inputs = parse_inputs("x: i32, step: i32, verbose: bool");
    let mut defaults = std::collections::HashMap::new();
    defaults.insert("step".to_string(), "1L".to_string());
    defaults.insert("verbose".to_string(), "FALSE".to_string());
    let builder = RArgumentBuilder::new(&inputs).with_defaults(defaults);
    output.push_str(&format!("formals: {}\n", builder.build_formals()));
    output.push_str(&format!("call_args: {}\n", builder.build_call_args()));

    // Missing<T> - clean formals (no quote(expr=) in signature); the sentinel
    // forwarding is inline in call_args
    output.push_str("\n# Missing<T> clean formals\n");
    let inputs = parse_inputs("x: i32, y: Missing<f64>, z: Missing<String>");
    let builder = RArgumentBuilder::new(&inputs);
    output.push_str(&format!("formals: {}\n", builder.build_formals()));
    output.push_str(&format!("call_args: {}\n", builder.build_call_args()));

    insta::assert_snapshot!(output);
}
// endregion

#[test]
fn call_attribution_strings() {
    assert_eq!(CallAttribution::default(), CallAttribution::Wrapper);
    assert_eq!(
        CallAttribution::Wrapper.dot_call_arg(),
        ".call = sys.call()"
    );
    assert_eq!(CallAttribution::Caller.dot_call_arg(), ".call = .mx_call");
    assert_eq!(CallAttribution::Wrapper.raise_default(), "sys.call()");
    assert_eq!(CallAttribution::Caller.raise_default(), ".mx_call");
    for call_formal in [false, true] {
        assert_eq!(CallAttribution::Wrapper.prelude(call_formal), "");
    }
    // One line since #1552: the frame arithmetic lives in the preamble helper.
    // With the `.call` formal (#1613) the helper resolves what was passed
    // there; an S3 method has no formal and keeps the zero-argument call.
    assert_eq!(
        CallAttribution::Caller.prelude(true),
        ".mx_call <- .miniextendr_caller_call(.call)"
    );
    assert_eq!(
        CallAttribution::Caller.prelude(false),
        ".mx_call <- .miniextendr_caller_call()"
    );
    assert_eq!(CallAttribution::Wrapper.r_check_call(), None);
    assert_eq!(CallAttribution::Caller.r_check_call(), Some(".mx_call"));
}

#[test]
fn call_attribution_formal_is_caller_only() {
    assert_eq!(CallAttribution::Caller.formal(), Some(".call = NULL"));
    assert_eq!(CallAttribution::Wrapper.formal(), None);
    let doc = CallAttribution::Caller
        .param_doc()
        .expect("caller documents .call");
    assert!(doc.contains("parent.frame()"), "{doc}");
    assert!(doc.contains("Pass it by name"), "{doc}");
    assert_eq!(CallAttribution::Wrapper.param_doc(), None);
}

/// The `.call` formal goes last (#1613): after `...`, so positional extras
/// land in the dots and `.call` is name-only; alone on a wrapper without other
/// formals.
#[test]
fn call_formal_is_appended_after_the_dots() {
    let formal = CallAttribution::Caller.formal();
    // `RArgumentBuilder::new` reads the `&Dots` parameter as `...`, wherever
    // it sits; `.call` still goes last, after any formal that follows it.
    let formals = |sig: &str| RArgumentBuilder::new(&parse_inputs(sig)).build_formals();
    assert_eq!(
        with_call_formal(&formals("x: i32"), formal),
        "x, .call = NULL"
    );
    assert_eq!(
        with_call_formal(&formals("x: i32, _dots: &Dots"), formal),
        "x, ..., .call = NULL"
    );
    assert_eq!(
        with_call_formal(&formals("x: i32, rest: &Dots, flag: bool"), formal),
        "x, ..., flag, .call = NULL"
    );
    assert_eq!(with_call_formal(&formals(""), formal), ".call = NULL");
    assert_eq!(with_call_formal("x, y = 1L", None), "x, y = 1L");
    assert_eq!(with_call_formal("", None), "");
}

#[test]
fn call_attribution_names_and_markers_round_trip() {
    for attribution in [CallAttribution::Wrapper, CallAttribution::Caller] {
        assert_eq!(
            CallAttribution::parse_name(attribution.name()),
            Some(attribution)
        );
    }
    assert_eq!(CallAttribution::parse_name("parent"), None);
    assert_eq!(CallAttribution::parse_name("none"), None);
    assert_eq!(CallAttribution::Wrapper.marker_name(), "Call");
    assert_eq!(CallAttribution::Caller.marker_name(), "CallerCall");
}

#[test]
fn call_attribution_resolve_precedence() {
    use CallAttribution::{Caller, Wrapper};
    // Nothing said: the framework default.
    assert_eq!(CallAttribution::resolve(None, None, None, false), Wrapper);
    assert_eq!(CallAttribution::resolve(None, None, None, true), Wrapper);
    // The crate default; `caller` applies to internal entries only.
    assert_eq!(
        CallAttribution::resolve(None, None, Some(Wrapper), false),
        Wrapper
    );
    assert_eq!(
        CallAttribution::resolve(None, None, Some(Caller), true),
        Caller
    );
    assert_eq!(
        CallAttribution::resolve(None, None, Some(Caller), false),
        Wrapper
    );
    // Attribute beats the crate default; marker beats the attribute (they are
    // validated to agree before this runs, so the order only matters for the
    // fallbacks).
    assert_eq!(
        CallAttribution::resolve(None, Some(Wrapper), Some(Caller), true),
        Wrapper
    );
    assert_eq!(
        CallAttribution::resolve(Some(Caller), None, Some(Wrapper), true),
        Caller
    );
    assert_eq!(
        CallAttribution::resolve(Some(Wrapper), Some(Wrapper), Some(Caller), true),
        Wrapper
    );
}

/// The attributes of a `match_arg` parameter of type `ty`, classified the way
/// the parser does it.
fn choice_attrs(ty: &str, several_ok: bool) -> crate::miniextendr_fn::ParamAttrs {
    let mut attrs = crate::miniextendr_fn::ParamAttrs {
        match_arg: true,
        several_ok,
        ..Default::default()
    };
    let ty: syn::Type = syn::parse_str(ty).unwrap();
    crate::miniextendr_fn::classify_choice_param(&mut attrs, "mode", &ty, false).unwrap();
    attrs
}

#[test]
fn match_arg_statement_per_attribution() {
    let scalar = choice_attrs("Mode", false);
    let optional = choice_attrs("Option<Mode>", false);
    let several = choice_attrs("Vec<Mode>", true);
    // Wrapper attribution: the preamble helpers with their default call, i.e.
    // the wrapper's own frame (#1552 folded the factor coercion and
    // `base::match.arg()` into them).
    let wrapper = CallAttribution::Wrapper;
    assert_eq!(
        wrapper.match_arg_statement("mode", "c(\"a\", \"b\")", &scalar),
        "mode <- .miniextendr_match_arg(mode, c(\"a\", \"b\"), \"mode\")"
    );
    assert_eq!(
        wrapper.match_arg_statement("mode", "c(\"a\", \"b\")", &optional),
        "if (!is.null(mode)) mode <- .miniextendr_match_arg(mode, c(\"a\", \"b\"), \"mode\")"
    );
    assert_eq!(
        wrapper.match_arg_statement("modes", ".__MX_CHOICES__", &several),
        "modes <- .miniextendr_match_arg_several(modes, .__MX_CHOICES__, \"modes\")"
    );
    // `call = caller` (#1548): every form raises with `.mx_call` and names the
    // argument; the scalar helper gets the choice list explicitly.
    let caller = CallAttribution::Caller;
    assert_eq!(
        caller.match_arg_statement("mode", "c(\"a\", \"b\")", &scalar),
        "mode <- .miniextendr_match_arg(mode, c(\"a\", \"b\"), \"mode\", .mx_call)"
    );
    assert_eq!(
        caller.match_arg_statement("mode", "c(\"a\", \"b\")", &optional),
        "if (!is.null(mode)) mode <- .miniextendr_match_arg(mode, c(\"a\", \"b\"), \"mode\", .mx_call)"
    );
    assert_eq!(
        caller.match_arg_statement("modes", ".__MX_CHOICES__", &several),
        "modes <- .miniextendr_match_arg_several(modes, .__MX_CHOICES__, \"modes\", .mx_call)"
    );
}

/// Every accepted choice-parameter shape: the R formal, the prelude statement
/// under the default and the `call = caller` attribution, and the `@param`
/// line (#1473, #1551).
#[test]
fn snapshot_choice_param_forms() {
    let forms = [
        ("Mode", false),
        ("Option<Mode>", false),
        ("Missing<Mode>", false),
        ("Missing<Option<Mode>>", false),
        ("Vec<Mode>", true),
        ("Missing<Vec<Mode>>", true),
        ("Missing<Box<[Mode]>>", true),
        ("Either<Mode, DataFrame>", false),
        ("Option<Either<Mode, DataFrame>>", false),
        ("Missing<Either<Mode, f64>>", false),
        ("Missing<Option<Either<Mode, List>>>", false),
        ("Either<Vec<Mode>, DataFrame>", true),
        ("Either<Box<[Mode]>, List>", true),
        ("Missing<Either<Vec<Mode>, f64>>", true),
        ("Option<Either<Vec<Mode>, DataFrame>>", true),
        ("Missing<Option<Either<Vec<Mode>, List>>>", true),
    ];
    let choices = "c(\"fast\", \"safe\")";
    let mut output = String::new();
    for (ty, several_ok) in forms {
        let attrs = choice_attrs(ty, several_ok);
        let prefix = if several_ok {
            "One or more of"
        } else {
            "One of"
        };
        output.push_str(&format!(
            "{ty}{}\n  formal:  mode = {}\n  wrapper: {}\n  caller:  {}\n  @param:  mode {prefix} \"fast\", \"safe\"{}.\n",
            if several_ok { " (several_ok)" } else { "" },
            attrs.choice_formal(choices),
            CallAttribution::Wrapper.match_arg_statement("mode", choices, &attrs),
            CallAttribution::Caller.match_arg_statement("mode", choices, &attrs),
            attrs.choice_doc_suffix(),
        ));
    }
    insta::assert_snapshot!(output);
}

#[test]
fn classify_choice_param_rejects_unsupported_layers() {
    let err = |ty: &str, several_ok: bool, has_default: bool| {
        let mut attrs = crate::miniextendr_fn::ParamAttrs {
            match_arg: true,
            several_ok,
            ..Default::default()
        };
        let ty: syn::Type = syn::parse_str(ty).unwrap();
        crate::miniextendr_fn::classify_choice_param(&mut attrs, "mode", &ty, has_default)
            .unwrap_err()
            .to_string()
    };
    assert!(err("Option<Missing<Mode>>", false, false).contains("outermost"));
    assert!(err("Missing<Missing<Mode>>", false, false).contains("outermost"));
    assert!(err("Option<Vec<Mode>>", true, false).contains("cannot be `Option<..>`"));
    assert!(err("Missing<[Mode; 2]>", true, false).contains("`Missing<Vec<T>>`"));
    assert!(err("Missing<&[Mode]>", true, false).contains("`Missing<Vec<T>>`"));
    assert!(err("Missing<Mode>", true, false).contains("requires a vector type"));
    assert!(err("Option<Mode>", false, true).contains("cannot have a default"));
    assert!(err("Option<Either<Mode, List>>", false, true).contains("cannot have a default"));
    assert!(err("Either<Option<Mode>, List>", false, false).contains("outermost"));
    assert!(err("Either<Either<Mode, f64>, List>", false, false).contains("outermost"));
    // `several_ok` under `Either` (#1612): only the owned containers decode,
    // bare or under `Missing`; a scalar gets the narrower container hint.
    for ty in [
        "Either<[Mode; 2], List>",
        "Either<&[Mode], List>",
        "Missing<Either<&[Mode], List>>",
    ] {
        assert!(err(ty, true, false).contains("`Either<Vec<T>, R>`"), "{ty}");
    }
    let scalar = err("Either<Mode, List>", true, false);
    assert!(scalar.contains("requires a vector type"), "{scalar}");
    assert!(scalar.contains("`Vec<T>` or `Box<[T]>`"), "{scalar}");
    assert!(!scalar.contains("&[T]"), "{scalar}");
    // A `NULL` formal makes a default meaningless, `several_ok` or not.
    assert!(err("Option<Either<Vec<Mode>, List>>", true, true).contains("cannot have a default"));
}

#[test]
fn either_choice_layers_record_the_other_arm() {
    let attrs = choice_attrs("Either<Mode, DataFrame>", false);
    assert_eq!(attrs.either_noun.as_deref(), Some("a data frame"));
    assert!(!attrs.optional && !attrs.omittable);
    assert_eq!(
        attrs.layered_leaf(),
        Some(crate::rust_conversion_builder::ChoiceLeaf::MatchArg)
    );
    // `choices(...)` on `Either<String, R>` needs the split decoder too; its
    // `Missing` / `Option` layers alone convert through `TryFromSexp`.
    let literal = |ty: &str| {
        let mut attrs = crate::miniextendr_fn::ParamAttrs {
            choices: Some(vec!["a".into(), "b".into()]),
            ..Default::default()
        };
        let ty: syn::Type = syn::parse_str(ty).unwrap();
        crate::miniextendr_fn::classify_choice_param(&mut attrs, "level", &ty, false).unwrap();
        attrs.layered_leaf()
    };
    // The leaf carries the list, which the argument error names.
    assert_eq!(
        literal("Either<String, f64>"),
        Some(crate::rust_conversion_builder::ChoiceLeaf::Literal {
            choices: vec!["a".into(), "b".into()],
            several: false,
        })
    );
    assert_eq!(literal("Missing<Option<String>>"), None);

    // A `several_ok` list with another kind of value (#1612).
    let several = choice_attrs("Either<Vec<Mode>, DataFrame>", true);
    assert_eq!(several.either_noun.as_deref(), Some("a data frame"));
    assert!(!several.optional && !several.omittable);
    assert_eq!(
        several.layered_leaf(),
        Some(crate::rust_conversion_builder::ChoiceLeaf::MatchArgSeveral)
    );
    let optional = choice_attrs("Option<Either<Vec<Mode>, DataFrame>>", true);
    assert!(optional.optional && !optional.omittable);
    assert_eq!(optional.choice_formal("c(\"a\")"), "NULL");
    let mut literal_several = crate::miniextendr_fn::ParamAttrs {
        choices: Some(vec!["a".into(), "b".into()]),
        several_ok: true,
        ..Default::default()
    };
    let ty: syn::Type = syn::parse_str("Either<Vec<String>, f64>").unwrap();
    crate::miniextendr_fn::classify_choice_param(&mut literal_several, "tiers", &ty, false)
        .unwrap();
    assert_eq!(
        literal_several.layered_leaf(),
        Some(crate::rust_conversion_builder::ChoiceLeaf::Literal {
            choices: vec!["a".into(), "b".into()],
            several: true,
        })
    );
    assert_eq!(
        literal_several.literal_choices_doc().as_deref(),
        Some("One or more of \"a\", \"b\", or a number.")
    );
}

/// The other accepted values of a choice parameter are worded by one
/// function for the `@param` line and for the argument error of an `Either`
/// choice, which differ only in how they name `NULL`.
#[test]
fn choice_alternatives_suffix_serves_the_param_line_and_the_error() {
    use crate::miniextendr_fn::choice_alternatives_suffix;
    let noun = Some("a data frame");
    assert_eq!(choice_alternatives_suffix(None, false, "NULL"), "");
    assert_eq!(
        choice_alternatives_suffix(noun, false, "NULL"),
        ", or a data frame"
    );
    assert_eq!(
        choice_alternatives_suffix(None, true, "NULL for no choice"),
        ", or NULL for no choice"
    );
    assert_eq!(
        choice_alternatives_suffix(noun, true, "NULL"),
        ", a data frame, or NULL"
    );
    // The `@param` line adds the omission note after them.
    for (ty, want) in [
        ("Mode", ""),
        ("Option<Mode>", ", or NULL for no choice"),
        ("Missing<Mode>", "; omitting the argument means no choice"),
        (
            "Missing<Option<Mode>>",
            ", or NULL; omitting the argument means no choice",
        ),
        ("Either<Mode, DataFrame>", ", or a data frame"),
        (
            "Option<Either<Mode, DataFrame>>",
            ", a data frame, or NULL for no choice",
        ),
        (
            "Missing<Option<Either<Mode, DataFrame>>>",
            ", a data frame, or NULL; omitting the argument means no choice",
        ),
    ] {
        assert_eq!(choice_attrs(ty, false).choice_doc_suffix(), want, "{ty}");
    }
}

/// `several_ok` under `Either` and `Missing` with `call = caller` (#1612): the
/// several-choice helper carries `.mx_call` and sits behind both guards.
#[test]
fn match_arg_statement_several_either_caller() {
    let either = choice_attrs("Either<Vec<Mode>, DataFrame>", true);
    let omitted = choice_attrs("Missing<Either<Vec<Mode>, DataFrame>>", true);
    let caller = CallAttribution::Caller;
    assert_eq!(
        caller.match_arg_statement("modes", ".__MX_CHOICES__", &either),
        "if (is.character(modes) || is.factor(modes)) modes <- \
         .miniextendr_match_arg_several(modes, .__MX_CHOICES__, \"modes\", .mx_call)"
    );
    assert_eq!(
        caller.match_arg_statement("modes", ".__MX_CHOICES__", &omitted),
        "if (!missing(modes) && (is.character(modes) || is.factor(modes))) modes <- \
         .miniextendr_match_arg_several(modes, .__MX_CHOICES__, \"modes\", .mx_call)"
    );
}
