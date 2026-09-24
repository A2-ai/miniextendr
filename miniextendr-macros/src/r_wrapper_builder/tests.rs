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
    let inputs = parse_inputs("x: i32, _dots: &Dots");
    let builder = RArgumentBuilder::new(&inputs).with_dots(None);
    assert_eq!(builder.build_formals(), "x, ...");
    assert_eq!(builder.build_call_args(), "x, list(...)");
}

#[test]
fn test_trailing_dots_auto_detected() {
    let inputs = parse_inputs("x: i32, dots: &Dots");
    let builder = RArgumentBuilder::new(&inputs);
    assert_eq!(builder.build_formals(), "x, ...");
    assert_eq!(builder.build_call_args(), "x, list(...)");
}

#[test]
fn test_named_dots() {
    // Note: In R, `...` cannot have a name/default in formals.
    // The named_dots is only used on Rust side. R always uses plain `...`
    let inputs = parse_inputs("x: i32, _dots: &Dots");
    let builder = RArgumentBuilder::new(&inputs).with_dots(Some("args".to_string()));
    assert_eq!(builder.build_formals(), "x, ...");
    assert_eq!(builder.build_call_args(), "x, list(...)");
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
    assert_eq!(call, ".Call(C_Counter__new, .call = match.call())");
}

#[test]
fn test_dot_call_with_self() {
    let call = DotCallBuilder::new("C_Counter__value")
        .with_self("self")
        .build();
    assert_eq!(call, ".Call(C_Counter__value, .call = match.call(), self)");
}

#[test]
fn test_dot_call_with_self_and_args() {
    let call = DotCallBuilder::new("C_Counter__add")
        .with_self("x")
        .with_args(&["n"])
        .build();
    assert_eq!(call, ".Call(C_Counter__add, .call = match.call(), x, n)");
}

#[test]
fn test_dot_call_static_with_args() {
    let call = DotCallBuilder::new("C_Counter__from_parts")
        .with_args(&["a", "b", "c"])
        .build();
    assert_eq!(
        call,
        ".Call(C_Counter__from_parts, .call = match.call(), a, b, c)"
    );
}

#[test]
fn test_dot_call_with_args_str_empty_skips_args() {
    let call = DotCallBuilder::new("C_Counter__new")
        .with_args_str("")
        .build();
    assert_eq!(call, ".Call(C_Counter__new, .call = match.call())");
}

#[test]
fn test_dot_call_with_args_str_passes_through() {
    let call = DotCallBuilder::new("C_Counter__update")
        .with_self("self")
        .with_args_str("step, verbose")
        .build();
    assert_eq!(
        call,
        ".Call(C_Counter__update, .call = match.call(), self, step, verbose)"
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
    let builder = RArgumentBuilder::new(&inputs).with_dots(None);
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
        ".call = match.call()"
    );
    assert_eq!(CallAttribution::None.dot_call_arg(), ".call = NULL");
    assert_eq!(CallAttribution::Caller.dot_call_arg(), ".call = .mx_call");
    assert_eq!(CallAttribution::Wrapper.raise_default(), "sys.call()");
    assert_eq!(CallAttribution::None.raise_default(), "sys.call()");
    assert_eq!(CallAttribution::Caller.raise_default(), ".mx_call");
    assert_eq!(CallAttribution::Wrapper.prelude("  "), "");
    assert_eq!(CallAttribution::None.prelude("  "), "");
    // One line since #1552: the frame arithmetic lives in the preamble helper.
    assert_eq!(
        CallAttribution::Caller.prelude("  "),
        ".mx_call <- .miniextendr_caller_call()"
    );
    assert_eq!(CallAttribution::Wrapper.r_check_call(), None);
    assert_eq!(CallAttribution::None.r_check_call(), None);
    assert_eq!(CallAttribution::Caller.r_check_call(), Some(".mx_call"));
}

#[test]
fn call_attribution_names_and_markers_round_trip() {
    for attribution in [
        CallAttribution::None,
        CallAttribution::Wrapper,
        CallAttribution::Caller,
    ] {
        assert_eq!(
            CallAttribution::parse_name(attribution.name()),
            Some(attribution)
        );
    }
    assert_eq!(CallAttribution::parse_name("parent"), None);
    assert_eq!(CallAttribution::Wrapper.marker_name(), Some("Call"));
    assert_eq!(CallAttribution::Caller.marker_name(), Some("CallerCall"));
    assert_eq!(CallAttribution::None.marker_name(), None);
}

#[test]
fn call_attribution_resolve_precedence() {
    use CallAttribution::{Caller, None as NoCall, Wrapper};
    // Nothing said: framework default, or `none` under `fast-default`.
    assert_eq!(
        CallAttribution::resolve(None, None, None, false, false),
        Wrapper
    );
    assert_eq!(
        CallAttribution::resolve(None, None, None, false, true),
        NoCall
    );
    // Crate default beats the feature; `caller` applies to internal entries only.
    assert_eq!(
        CallAttribution::resolve(None, None, Some(NoCall), false, false),
        NoCall
    );
    assert_eq!(
        CallAttribution::resolve(None, None, Some(Wrapper), false, true),
        Wrapper
    );
    assert_eq!(
        CallAttribution::resolve(None, None, Some(Caller), true, false),
        Caller
    );
    assert_eq!(
        CallAttribution::resolve(None, None, Some(Caller), false, false),
        Wrapper
    );
    // Attribute beats the crate default; marker beats the attribute (they are
    // validated to agree before this runs, so the order only matters for the
    // fallbacks).
    assert_eq!(
        CallAttribution::resolve(None, Some(Wrapper), Some(NoCall), false, true),
        Wrapper
    );
    assert_eq!(
        CallAttribution::resolve(Some(Caller), None, Some(NoCall), true, true),
        Caller
    );
    assert_eq!(
        CallAttribution::resolve(Some(Wrapper), Some(Wrapper), Some(Caller), true, false),
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
    // Wrapper attribution (and `no_call_attribution`): the preamble helpers
    // with their default call, i.e. the wrapper's own frame (#1552 folded the
    // factor coercion and `base::match.arg()` into them).
    for attribution in [CallAttribution::Wrapper, CallAttribution::None] {
        assert_eq!(
            attribution.match_arg_statement("mode", "c(\"a\", \"b\")", &scalar),
            "mode <- .miniextendr_match_arg(mode, c(\"a\", \"b\"), \"mode\")"
        );
        assert_eq!(
            attribution.match_arg_statement("mode", "c(\"a\", \"b\")", &optional),
            "if (!is.null(mode)) mode <- .miniextendr_match_arg(mode, c(\"a\", \"b\"), \"mode\")"
        );
        assert_eq!(
            attribution.match_arg_statement("modes", ".__MX_CHOICES__", &several),
            "modes <- .miniextendr_match_arg_several(modes, .__MX_CHOICES__, \"modes\")"
        );
    }
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
    assert!(err("Either<Vec<Mode>, List>", true, false).contains("cannot be an `Either<..>`"));
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
    assert_eq!(
        literal("Either<String, f64>"),
        Some(crate::rust_conversion_builder::ChoiceLeaf::Literal)
    );
    assert_eq!(literal("Missing<Option<String>>"), None);
}
