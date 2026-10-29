use super::{
    ParamMarker, UnevaluatedKind, UnevaluatedParam, call_marker, choice_layer_name, choice_layers,
    erase_lifetimes, is_lazy_dots_marker, is_main_thread_bound_input, is_main_thread_bound_return,
    is_nargs_marker, is_rng_guard_param, match_arg_choices_ty, mentions_lazy_dots,
    mentions_rng_guard, mentions_unevaluated_marker, peel_param_markers, r_value_noun,
    type_display, unevaluated_param, visibility_marker_error,
};
use crate::r_wrapper_builder::CallAttribution;

fn ty(s: &str) -> syn::Type {
    syn::parse_str(s).unwrap()
}

#[test]
fn main_thread_bound_return_detects_nested_positions() {
    // Bare and fully-qualified paths
    assert!(is_main_thread_bound_return(&ty("BuiltDataFrame")));
    assert!(is_main_thread_bound_return(&ty(
        "miniextendr_api::dataframe::BuiltDataFrame"
    )));
    assert!(is_main_thread_bound_return(&ty("DataFrameShape")));
    assert!(is_main_thread_bound_input(&ty("NamedListBuilder")));
    assert!(is_main_thread_bound_return(&ty("NamedListBuilder")));
    assert!(is_main_thread_bound_return(&ty(
        "Result<miniextendr_api::NamedListBuilder, String>"
    )));
    assert!(is_main_thread_bound_return(&ty("SEXP")));
    // Nested inside Result / Option / containers / tuples
    assert!(is_main_thread_bound_return(&ty(
        "Result<BuiltDataFrame, String>"
    )));
    assert!(is_main_thread_bound_return(&ty(
        "Result<DataFrameShape, std::string::String>"
    )));
    assert!(is_main_thread_bound_return(&ty("Option<BuiltDataFrame>")));
    assert!(is_main_thread_bound_return(&ty("Vec<BuiltDataFrame>")));
    assert!(is_main_thread_bound_return(&ty("(i32, BuiltDataFrame)")));
    // Precious-list roots, in the error position too
    assert!(is_main_thread_bound_input(&ty("Preserved")));
    assert!(is_main_thread_bound_return(&ty("Result<bool, REvalError>")));
    assert!(is_main_thread_bound_input(&ty(
        "miniextendr_api::txt_progress_bar::RTxtProgressBar"
    )));
    // Send-safe returns stay worker-eligible
    assert!(!is_main_thread_bound_return(&ty("i32")));
    assert!(!is_main_thread_bound_return(&ty(
        "Result<Vec<f64>, String>"
    )));
    assert!(!is_main_thread_bound_return(&ty("ExternalPtr<MyType>")));
    assert!(!is_main_thread_bound_return(&ty("DataFrame")));
}

#[test]
fn unevaluated_param_matches_bare_and_missing_markers_only() {
    let required = |kind| {
        Some(UnevaluatedParam {
            kind,
            optional: false,
        })
    };
    let optional = |kind| {
        Some(UnevaluatedParam {
            kind,
            optional: true,
        })
    };
    assert_eq!(
        unevaluated_param(&ty("Quoted")),
        required(UnevaluatedKind::Quoted)
    );
    assert_eq!(
        unevaluated_param(&ty("Quoted<'_>")),
        required(UnevaluatedKind::Quoted)
    );
    assert_eq!(
        unevaluated_param(&ty("::miniextendr_api::Quosure<'a>")),
        required(UnevaluatedKind::Quosure)
    );
    assert_eq!(
        unevaluated_param(&ty("Missing<Quoted>")),
        optional(UnevaluatedKind::Quoted)
    );
    assert_eq!(
        unevaluated_param(&ty("miniextendr_api::Missing<Quosure<'_>>")),
        optional(UnevaluatedKind::Quosure)
    );
    for other in [
        "Option<Quoted>",
        "&Quoted",
        "Vec<Quosure>",
        "Quoted<i32>",
        "Missing<Option<Quoted>>",
        "SEXP",
    ] {
        assert_eq!(unevaluated_param(&ty(other)), None, "{other}");
    }
    // The misplaced shapes are still recognised as mentioning a marker, so
    // the macro can refuse them by name.
    for misplaced in [
        "Option<Quoted>",
        "&Quoted",
        "Vec<Quosure<'_>>",
        "(i32, Quoted)",
    ] {
        assert!(mentions_unevaluated_marker(&ty(misplaced)), "{misplaced}");
    }
    assert!(!mentions_unevaluated_marker(&ty("Vec<i32>")));
    // Bare markers are main-thread-bound inputs; `Missing<..>` is checked
    // by the caller.
    assert!(is_main_thread_bound_input(&ty("Quoted")));
    assert!(is_main_thread_bound_input(&ty("Quosure<'_>")));
}

#[test]
fn unevaluated_kind_r_call_args_never_force_the_argument() {
    assert_eq!(
        UnevaluatedKind::Quoted.r_call_arg("cond"),
        "if (missing(cond)) quote(expr=) else list(substitute(cond), parent.frame())"
    );
    assert_eq!(
        UnevaluatedKind::Quosure.r_call_arg("cols"),
        "if (missing(cols)) quote(expr=) else rlang::enquo(cols)"
    );
}

#[test]
fn call_marker_matches_bare_marker_types_only() {
    assert_eq!(call_marker(&ty("Call")), Some(CallAttribution::Wrapper));
    assert_eq!(
        call_marker(&ty("miniextendr_api::CallerCall")),
        Some(CallAttribution::Caller)
    );
    assert_eq!(
        call_marker(&ty("::miniextendr_api::call_marker::Call")),
        Some(CallAttribution::Wrapper)
    );
    // Not markers: generics, references, wrappers, other types.
    assert_eq!(call_marker(&ty("Call<i32>")), None);
    assert_eq!(call_marker(&ty("&Call")), None);
    assert_eq!(call_marker(&ty("Option<Call>")), None);
    assert_eq!(call_marker(&ty("SEXP")), None);
    assert_eq!(call_marker(&ty("Caller")), None);
    // A marker holds the `.call` slot's SEXP, so it pins the main thread.
    assert!(is_main_thread_bound_input(&ty("Call")));
    assert!(is_main_thread_bound_input(&ty(
        "miniextendr_api::CallerCall"
    )));
}

#[test]
fn nargs_marker_matches_the_bare_type_only() {
    assert!(is_nargs_marker(&ty("NArgs")));
    assert!(is_nargs_marker(&ty("miniextendr_api::NArgs")));
    assert!(is_nargs_marker(&ty("::miniextendr_api::nargs::NArgs")));
    // Not the marker: generics, references, wrappers, other types.
    assert!(!is_nargs_marker(&ty("NArgs<i32>")));
    assert!(!is_nargs_marker(&ty("&NArgs")));
    assert!(!is_nargs_marker(&ty("Option<NArgs>")));
    assert!(!is_nargs_marker(&ty("Missing<NArgs>")));
    assert!(!is_nargs_marker(&ty("Nargs")));
    // A plain count: it does not pin the main thread.
    assert!(!is_main_thread_bound_input(&ty("NArgs")));
}

#[test]
fn rng_guard_param_is_a_shared_borrow_of_the_bare_type() {
    for guard in [
        "&RngGuard",
        "&'a RngGuard",
        "&miniextendr_api::RngGuard",
        "&::miniextendr_api::rng::RngGuard",
    ] {
        assert!(is_rng_guard_param(&ty(guard)), "{guard}");
        assert!(mentions_rng_guard(&ty(guard)), "{guard}");
    }
    // Shapes the wrapper cannot fill: named, so the macro refuses them.
    for misplaced in [
        "&mut RngGuard",
        "RngGuard",
        "Option<&RngGuard>",
        "(i32, &RngGuard)",
    ] {
        assert!(!is_rng_guard_param(&ty(misplaced)), "{misplaced}");
        assert!(mentions_rng_guard(&ty(misplaced)), "{misplaced}");
    }
    for other in ["&RngGuard<i32>", "&Guard", "&str", "SEXP"] {
        assert!(!is_rng_guard_param(&ty(other)), "{other}");
    }
    assert!(!mentions_rng_guard(&ty("Vec<f64>")));
}

#[test]
fn lazy_dots_marker_matches_the_whole_type_by_value_only() {
    assert!(is_lazy_dots_marker(&ty("LazyDots")));
    assert!(is_lazy_dots_marker(&ty("LazyDots<'_>")));
    assert!(is_lazy_dots_marker(&ty("miniextendr_api::LazyDots<'a>")));
    assert!(is_lazy_dots_marker(&ty(
        "::miniextendr_api::lazy_dots::LazyDots"
    )));
    // Not the parameter: generics, references, wrappers, other types.
    for other in [
        "LazyDots<i32>",
        "&LazyDots",
        "Option<LazyDots>",
        "Missing<LazyDots>",
        "Dots",
        "&Dots",
    ] {
        assert!(!is_lazy_dots_marker(&ty(other)), "{other}");
    }
    // The misplaced shapes are recognised, so the macro can refuse them.
    for misplaced in [
        "&LazyDots",
        "Option<LazyDots<'_>>",
        "Vec<LazyDots>",
        "Missing<LazyDots>",
    ] {
        assert!(mentions_lazy_dots(&ty(misplaced)), "{misplaced}");
    }
    assert!(!mentions_lazy_dots(&ty("&Dots")));
    // It holds the wrapper's frame, so it pins the main thread.
    assert!(is_main_thread_bound_input(&ty("LazyDots<'_>")));
}

/// The inner type a parameter converts as, and the markers peeled off it.
fn peeled(s: &str) -> (Vec<ParamMarker>, String) {
    let t = ty(s);
    let (markers, inner) = peel_param_markers(&t).expect("peels");
    (markers, type_display(inner))
}

fn peel_error(s: &str) -> String {
    peel_param_markers(&ty(s))
        .expect_err("rejected")
        .to_string()
}

#[test]
fn param_markers_peel_the_outermost_marker() {
    assert_eq!(
        peeled("Checked<i32>"),
        (vec![ParamMarker::Checked], "i32".into())
    );
    assert_eq!(
        peeled("miniextendr_api::Unchecked<Vec<f64>>"),
        (vec![ParamMarker::Unchecked], "Vec<f64>".into())
    );
    assert_eq!(
        peeled("::miniextendr_api::precondition_marker::Checked<Option<i32>>"),
        (vec![ParamMarker::Checked], "Option<i32>".into())
    );
    assert_eq!(
        peeled("Checked<&'a [f64]>"),
        (vec![ParamMarker::Checked], "&'a [f64]".into())
    );
    // Not markers: no type argument, two of them, or another type.
    assert_eq!(peeled("Checked"), (vec![], "Checked".into()));
    assert_eq!(
        peeled("Checked<i32, f64>"),
        (vec![], "Checked<i32, f64>".into())
    );
    assert_eq!(peeled("Vec<i32>"), (vec![], "Vec<i32>".into()));
    assert_eq!(peeled("Missing<i32>"), (vec![], "Missing<i32>".into()));
}

#[test]
fn param_markers_reject_stacks_and_nesting() {
    for stacked in ["Checked<Unchecked<i32>>", "Checked<Checked<i32>>"] {
        let err = peel_error(stacked);
        assert!(
            err.contains("at most one precondition marker per parameter"),
            "{stacked}: {err}"
        );
    }
    for nested in [
        "Option<Checked<i32>>",
        "Missing<Unchecked<i32>>",
        "Vec<Checked<i32>>",
        "&Checked<i32>",
        "Option<Vec<Unchecked<i32>>>",
        "Checked<Option<Unchecked<i32>>>",
    ] {
        let err = peel_error(nested);
        assert!(err.contains("outermost"), "{nested}: {err}");
    }
    assert!(peel_error("Option<Checked<i32>>").contains("put `Checked` outermost"));
}

/// A parameter marker in return position is an error, also inside a
/// visibility marker; in argument position it is the parameter's to peel.
#[test]
fn param_markers_are_rejected_in_return_position() {
    for ret in [
        "Checked<i32>",
        "miniextendr_api::Unchecked<Vec<f64>>",
        "Invisible<Checked<i32>>",
    ] {
        let err = visibility_marker_error(&ty(ret), "return")
            .expect("rejected")
            .to_string();
        assert!(err.contains("marks a parameter only"), "{ret}: {err}");
    }
    assert!(visibility_marker_error(&ty("Checked<i32>"), "argument").is_none());
}

#[test]
fn type_display_uses_source_spacing() {
    let cases = [
        ("i32", "i32"),
        ("AsFromStrVec<i32>", "AsFromStrVec<i32>"),
        ("Either<i32, String>", "Either<i32, String>"),
        ("&[f64]", "&[f64]"),
        ("&mut [f64]", "&mut [f64]"),
        ("&str", "&str"),
        ("&'a str", "&'a str"),
        ("&'static [u8]", "&'static [u8]"),
        ("Option<Vec<String>>", "Option<Vec<String>>"),
        (
            "HashMap<String, Vec<(i32, f64)>>",
            "HashMap<String, Vec<(i32, f64)>>",
        ),
        ("::std::vec::Vec<i32>", "::std::vec::Vec<i32>"),
        ("[u8; 4]", "[u8; 4]"),
        ("Box<[Mode]>", "Box<[Mode]>"),
        ("()", "()"),
        ("*const T", "*const T"),
        ("RCow<'a, i32>", "RCow<'a, i32>"),
        (
            "Box<dyn Fn(i32) -> i32 + Send + 'static>",
            "Box<dyn Fn(i32) -> i32 + Send + 'static>",
        ),
        ("impl Iterator<Item = i32>", "impl Iterator<Item = i32>"),
        ("<T as Trait>::Assoc", "<T as Trait>::Assoc"),
        ("for<'a> fn(&'a str) -> bool", "for<'a> fn(&'a str) -> bool"),
    ];
    for (src, want) in cases {
        assert_eq!(type_display(&ty(src)), want, "rendering `{src}`");
    }
}

#[test]
fn r_value_noun_names_the_r_side() {
    let cases = [
        ("DataFrame", "a data frame"),
        ("miniextendr_api::DataFrame", "a data frame"),
        ("List", "a list"),
        ("HashMap<String, i32>", "a list"),
        ("String", "a string"),
        ("&str", "a string"),
        ("f64", "a number"),
        ("i32", "an integer"),
        ("bool", "TRUE or FALSE"),
        ("Vec<f64>", "a numeric vector"),
        ("&[i32]", "an integer vector"),
        ("Vec<Option<String>>", "a character vector"),
        ("SEXP", "any other R value"),
        ("ExternalPtr<Model>", "an external pointer"),
        ("Model", "a `Model`"),
        ("Vec<Model>", "a `Vec<Model>`"),
    ];
    for (src, want) in cases {
        assert_eq!(r_value_noun(&ty(src)), want, "noun for `{src}`");
    }
}

#[test]
fn choice_layers_peel_missing_option_either() {
    let layers = |src: &str| {
        let t = ty(src);
        let l = choice_layers(&t);
        (
            l.missing,
            l.nullable,
            l.either_right.map(type_display),
            type_display(l.value),
        )
    };
    assert_eq!(layers("Mode"), (false, false, None, "Mode".to_string()));
    assert_eq!(
        layers("Missing<Option<Either<Mode, DataFrame>>>"),
        (
            true,
            true,
            Some("DataFrame".to_string()),
            "Mode".to_string()
        )
    );
    assert_eq!(
        layers("Either<Mode, Option<f64>>"),
        (
            false,
            false,
            Some("Option<f64>".to_string()),
            "Mode".to_string()
        )
    );
    // A layer out of order stays in the value, where the classifier
    // rejects it.
    assert_eq!(
        layers("Option<Missing<Mode>>"),
        (false, true, None, "Missing<Mode>".to_string())
    );
    assert_eq!(choice_layer_name(&ty("Missing<Mode>")), Some("Missing"));
    assert_eq!(choice_layer_name(&ty("Either<Mode, f64>")), Some("Either"));
    assert_eq!(choice_layer_name(&ty("Either<Mode>")), None);
    assert_eq!(
        type_display(match_arg_choices_ty(&ty("Missing<Vec<Mode>>"), true)),
        "Mode"
    );
    assert_eq!(
        type_display(match_arg_choices_ty(
            &ty("Option<Either<Mode, List>>"),
            false
        )),
        "Mode"
    );
    // A `several_ok` list with another kind of value (#1612).
    assert_eq!(
        layers("Either<Vec<Mode>, DataFrame>"),
        (
            false,
            false,
            Some("DataFrame".to_string()),
            "Vec<Mode>".to_string()
        )
    );
    assert_eq!(
        type_display(match_arg_choices_ty(
            &ty("Missing<Either<Vec<Mode>, DataFrame>>"),
            true
        )),
        "Mode"
    );
}

#[test]
fn erase_lifetimes_replaces_every_lifetime() {
    let erased = |s: &str| type_display(&erase_lifetimes(&ty(s)));
    assert_eq!(erased("&'a [f64]"), "&'_ [f64]");
    assert_eq!(erased("&'static str"), "&'_ str");
    assert_eq!(erased("&mut [i32]"), "&mut [i32]");
    assert_eq!(erased("RCow<'a, Vec<&'b str>>"), "RCow<'_, Vec<&'_ str>>");
    assert_eq!(erased("&str"), "&str");
}
