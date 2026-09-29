use super::*;

fn parse_param(s: &str) -> syn::FnArg {
    let sig: syn::Signature = syn::parse_str(&format!("fn test({})", s)).unwrap();
    sig.inputs.into_iter().next().unwrap()
}

#[test]
fn test_unit_type() {
    let builder = RustConversionBuilder::new();
    let param = parse_param("_unused: ()");
    if let syn::FnArg::Typed(pat_type) = param {
        let sexp_ident = syn::Ident::new("arg_0", proc_macro2::Span::call_site());
        let stmts = builder.build_conversion(&pat_type, &sexp_ident);
        assert_eq!(stmts.len(), 1);
        assert!(stmts[0].to_string().contains("let"));
    }
}

#[test]
fn test_basic_conversion() {
    let builder = RustConversionBuilder::new();
    let param = parse_param("x: i32");
    if let syn::FnArg::Typed(pat_type) = param {
        let sexp_ident = syn::Ident::new("arg_0", proc_macro2::Span::call_site());
        let stmts = builder.build_conversion(&pat_type, &sexp_ident);
        assert_eq!(stmts.len(), 1);
        assert!(stmts[0].to_string().contains("TryFromSexp"));
    }
}

#[test]
fn test_slice_conversion() {
    let builder = RustConversionBuilder::new();
    let param = parse_param("x: &[i32]");
    if let syn::FnArg::Typed(pat_type) = param {
        let sexp_ident = syn::Ident::new("arg_0", proc_macro2::Span::call_site());
        let stmts = builder.build_conversion(&pat_type, &sexp_ident);
        assert_eq!(stmts.len(), 1);
        assert!(stmts[0].to_string().contains("TryFromSexp"));
    }
}

#[test]
fn test_str_conversion_main_thread_borrows_zero_copy() {
    // Main-thread path (`build_conversion`): `&str` borrows R's CHARSXP pool
    // directly — a single zero-copy `TryFromSexp` binding, no `String` allocation.
    let builder = RustConversionBuilder::new();
    let param = parse_param("s: &str");
    if let syn::FnArg::Typed(pat_type) = param {
        let sexp_ident = syn::Ident::new("arg_0", proc_macro2::Span::call_site());
        let stmts = builder.build_conversion(&pat_type, &sexp_ident);
        assert_eq!(stmts.len(), 1); // single zero-copy borrow
        let s = stmts[0].to_string();
        assert!(s.contains("TryFromSexp"));
        // No owning-String detour and no Borrow trait on the main-thread path.
        assert!(!s.contains("String"));
        assert!(!s.contains("Borrow"));
    }
}

#[test]
fn test_str_conversion_worker_copies_then_borrows() {
    // Worker path (`build_conversion_split`): `&str` must be owned-then-borrowed
    // because a borrowed view over R's CHARSXP pool is `!Send`.
    let builder = RustConversionBuilder::new();
    let param = parse_param("s: &str");
    if let syn::FnArg::Typed(pat_type) = param {
        let sexp_ident = syn::Ident::new("arg_0", proc_macro2::Span::call_site());
        let (owned, borrowed) = builder.build_conversion_split(&pat_type, &sexp_ident);
        assert_eq!(owned.len(), 1); // String storage on main thread
        assert_eq!(borrowed.len(), 1); // borrow inside worker closure
        assert!(owned[0].to_string().contains("String"));
        assert!(borrowed[0].to_string().contains("Borrow"));
    }
}

/// Build the conversion statements of one parameter, joined as a string.
fn conversion_text(builder: &RustConversionBuilder, src: &str) -> String {
    let syn::FnArg::Typed(pat_type) = parse_param(src) else {
        unreachable!()
    };
    let sexp_ident = syn::Ident::new("arg_0", proc_macro2::Span::call_site());
    builder
        .build_conversion(&pat_type, &sexp_ident)
        .iter()
        .map(|t| t.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

/// A type without an R-facing expectation (an opaque custom type): the `Err`
/// arm names the parameter by its R formal (`_x` → `x`) in the
/// `invalid '<p>' argument` prefix, passes the Rust type (with source spacing)
/// as `e$rust_type` rather than in the message, probes without an
/// expectation, and passes an empty crate class list by default (#1591).
#[test]
fn test_conversion_err_arm_fallback_prefix_and_rust_type() {
    let s = conversion_text(&RustConversionBuilder::new(), "_nums: Hyperparams<i32>");
    assert!(s.contains("conversion_condition_value"), "{s}");
    // The error may still know its expectation (a `match_arg` choice error,
    // #1594): the prefix is built on the failure path, `invalid 'nums'
    // argument` when it does not.
    assert!(s.contains("__mx_conversion_expectation ! (e)"), "{s}");
    assert!(
        s.contains("conversion_prefix (\"nums\" , false , __mx_expected . as_deref () ,)"),
        "{s}"
    );
    assert!(
        s.contains("__mx_conversion_err_parts ! (e , __mx_expected . is_some ())"),
        "{s}"
    );
    assert!(
        s.contains("\"nums\" , :: core :: option :: Option :: Some (\"Hyperparams<i32>\") , & []"),
        "{s}"
    );
    assert!(!s.contains("failed to convert"), "{s}");
}

/// A type with an R-facing expectation: `'<p>' must be <expected>`, the same
/// classification as the R-side check, and the probe told the expectation is
/// stated.
#[test]
fn test_conversion_err_arm_states_the_r_expectation() {
    let builder = RustConversionBuilder::new();
    for (src, prefix) in [
        ("dv: AsNumericVec", "'dv' must be numeric"),
        ("num: AsNumeric", "'num' must be a single number"),
        ("x: i32", "'x' must be a single integer"),
        ("x: Option<f64>", "'x' must be NULL or a single double"),
        ("s: &str", "'s' must be a single string"),
        ("flag: bool", "'flag' must be TRUE or FALSE"),
        ("xs: &[f64]", "'xs' must be double"),
        ("addr: AsFromStr<IpAddr>", "'addr' must be a single string"),
        ("addrs: AsFromStrVec<IpAddr>", "'addrs' must be character"),
        (
            "value: Either<i32, String>",
            "'value' must be a single integer or a single string",
        ),
        ("pair: (i32, String)", "'pair' must be a list of length 2"),
    ] {
        let s = conversion_text(&builder, src);
        assert!(s.contains(&format!("\"{prefix}\"")), "{src}: {s}");
        assert!(
            s.contains("__mx_conversion_err_parts ! (e , true)"),
            "{src}: {s}"
        );
    }
    // `coerce` widens the expectation with the gate.
    let s = conversion_text(
        &RustConversionBuilder::new().with_coerce_param("x".to_string()),
        "x: i32",
    );
    assert!(s.contains("\"'x' must be a single whole number\""), "{s}");
}

/// A `several_ok` fixed-size array whose selection has the wrong length is an
/// argument error too (#1591), not a panic.
#[test]
fn test_several_ok_array_length_is_an_argument_error() {
    let builder = RustConversionBuilder::new().with_match_arg_several_ok("modes".to_string());
    let s = conversion_text(&builder, "modes: [Mode; 2]");
    assert!(!s.contains("panic"), "{s}");
    assert!(s.contains("\"'modes' must be of length 2\""), "{s}");
    assert!(s.contains("got length {}"), "{s}");
    assert_eq!(s.matches("conversion_condition_value").count(), 2, "{s}");
}

/// Strict input rejections are argument errors (#1594), not panics: the
/// checked helper returns a `Result` bound like any other conversion, with
/// an expectation naming what strict accepts.
#[test]
fn test_strict_input_rejection_is_an_argument_error() {
    let builder = RustConversionBuilder::new().with_strict();
    for (src, helper, prefix) in [
        (
            "n: i64",
            "checked_try_from_sexp_i64",
            "'n' must be a single whole number",
        ),
        (
            "n: usize",
            "checked_try_from_sexp_usize",
            "'n' must be a single non-negative whole number",
        ),
        (
            "xs: Vec<u64>",
            "checked_vec_try_from_sexp_u64",
            "'xs' must be integer or whole-number numeric",
        ),
        (
            "xs: Vec<Option<isize>>",
            "checked_vec_option_try_from_sexp_isize",
            "'xs' must be integer or whole-number numeric",
        ),
    ] {
        let s = conversion_text(&builder, src);
        assert!(
            s.contains(&format!("strict :: {helper} (arg_0)")),
            "{src}: {s}"
        );
        assert!(s.contains(&format!("\"{prefix}\"")), "{src}: {s}");
        assert!(s.contains("conversion_condition_value"), "{src}: {s}");
        assert!(
            s.contains("__mx_conversion_err_parts ! (e , true)"),
            "{src}: {s}"
        );
        assert!(!s.contains("panic"), "{src}: {s}");
    }
}

/// Every conversion site binds a typed `let`, which the `Err` arm's probe
/// needs; borrowed slices and `&str` carry the lifetime-erased type.
#[test]
fn test_borrowed_bindings_are_typed_with_erased_lifetimes() {
    let builder = RustConversionBuilder::new();
    for (src, binding) in [
        ("x: &'a [f64]", "let x : & '_ [f64]"),
        ("s: &'a str", "let s : & '_ str"),
        ("x: &[f64]", "let x : & [f64]"),
    ] {
        let param = parse_param(src);
        let syn::FnArg::Typed(pat_type) = param else {
            unreachable!()
        };
        let sexp_ident = syn::Ident::new("arg_0", proc_macro2::Span::call_site());
        let s = builder.build_conversion(&pat_type, &sexp_ident)[0].to_string();
        assert!(s.contains(binding), "{src}: {s}");
    }
}

/// The crate-level `conversion_error_class` is emitted into every arm.
#[test]
fn test_conversion_err_arm_carries_the_crate_class() {
    let builder = RustConversionBuilder::new().with_conversion_error_class(vec![
        "pkg_error_argument".to_string(),
        "pkg_error".to_string(),
    ]);
    for src in ["x: i32", "x: &[f64]", "s: &str"] {
        let param = parse_param(src);
        let syn::FnArg::Typed(pat_type) = param else {
            unreachable!()
        };
        let sexp_ident = syn::Ident::new("arg_0", proc_macro2::Span::call_site());
        let s = builder.build_conversion(&pat_type, &sexp_ident)[0].to_string();
        assert!(
            s.contains("& [\"pkg_error_argument\" , \"pkg_error\"]"),
            "{src}: {s}"
        );
    }
}

#[test]
fn test_coercion() {
    let builder = RustConversionBuilder::new().with_coerce_param("x".to_string());
    let param = parse_param("x: u16");
    if let syn::FnArg::Typed(pat_type) = param {
        let sexp_ident = syn::Ident::new("arg_0", proc_macro2::Span::call_site());
        let stmts = builder.build_conversion(&pat_type, &sexp_ident);
        assert_eq!(stmts.len(), 1);
        assert!(stmts[0].to_string().contains("TryFromSexp"));
        assert!(stmts[0].to_string().contains("u16"));
    }
}

/// The decoder of a layered choice parameter (#1473, #1551): one
/// `match_arg_*` helper per layer, outermost first, around the choice's own
/// decoder.
#[test]
fn test_layered_choice_decoders() {
    let decoder = |ty: &str, leaf: ChoiceLeaf| {
        let ty: syn::Type = syn::parse_str(ty).unwrap();
        let sexp = syn::Ident::new("s", proc_macro2::Span::call_site());
        layered_choice_expr(&ty, leaf, &sexp, proc_macro2::Span::call_site())
            .to_string()
            .replace(' ', "")
    };
    let api = "::miniextendr_api::";
    // The list does not enter the decoder (the R prelude matched it).
    let literal = ChoiceLeaf::Literal {
        choices: vec!["low".into()],
        several: false,
    };
    let cases = [
        (
            "Option<Mode>",
            ChoiceLeaf::MatchArg,
            format!("{api}match_arg_option_from_sexp::<Mode>(s)"),
        ),
        (
            "Missing<Mode>",
            ChoiceLeaf::MatchArg,
            format!("{api}match_arg_missing_or(s,{api}match_arg_from_sexp::<Mode>)"),
        ),
        (
            "Missing<Option<Mode>>",
            ChoiceLeaf::MatchArg,
            format!("{api}match_arg_missing_or(s,{api}match_arg_option_from_sexp::<Mode>)"),
        ),
        (
            "Missing<Vec<Mode>>",
            ChoiceLeaf::MatchArgSeveral,
            format!("{api}match_arg_missing_or(s,{api}match_arg_vec_from_sexp::<Mode>)"),
        ),
        (
            "Missing<Box<[Mode]>>",
            ChoiceLeaf::MatchArgSeveral,
            format!(
                "{api}match_arg_missing_or(s,|__mx_sexp|{api}match_arg_vec_from_sexp::<Mode>(__mx_sexp).map(::std::vec::Vec::into_boxed_slice))"
            ),
        ),
        (
            "Either<Route, DataFrame>",
            ChoiceLeaf::MatchArg,
            format!(
                "{api}match_arg_either_or::<_,DataFrame,_>(s,{api}match_arg_from_sexp::<Route>)"
            ),
        ),
        (
            "Option<Either<Route, DataFrame>>",
            ChoiceLeaf::MatchArg,
            format!(
                "{api}match_arg_null_or(s,|__mx_sexp|{api}match_arg_either_or::<_,DataFrame,_>(__mx_sexp,{api}match_arg_from_sexp::<Route>))"
            ),
        ),
        (
            "Either<String, f64>",
            literal.clone(),
            format!(
                "{api}match_arg_either_or::<_,f64,_>(s,<Stringas::miniextendr_api::TryFromSexp>::try_from_sexp)"
            ),
        ),
        // `several_ok` lists with another kind of value (#1612).
        (
            "Either<Vec<Route>, DataFrame>",
            ChoiceLeaf::MatchArgSeveral,
            format!(
                "{api}match_arg_either_or::<_,DataFrame,_>(s,{api}match_arg_vec_from_sexp::<Route>)"
            ),
        ),
        (
            "Either<Box<[Route]>, DataFrame>",
            ChoiceLeaf::MatchArgSeveral,
            format!(
                "{api}match_arg_either_or::<_,DataFrame,_>(s,|__mx_sexp|{api}match_arg_vec_from_sexp::<Route>(__mx_sexp).map(::std::vec::Vec::into_boxed_slice))"
            ),
        ),
        (
            "Missing<Either<Vec<Route>, DataFrame>>",
            ChoiceLeaf::MatchArgSeveral,
            format!(
                "{api}match_arg_missing_or(s,|__mx_sexp|{api}match_arg_either_or::<_,DataFrame,_>(__mx_sexp,{api}match_arg_vec_from_sexp::<Route>))"
            ),
        ),
        (
            "Option<Either<Vec<Route>, DataFrame>>",
            ChoiceLeaf::MatchArgSeveral,
            format!(
                "{api}match_arg_null_or(s,|__mx_sexp|{api}match_arg_either_or::<_,DataFrame,_>(__mx_sexp,{api}match_arg_vec_from_sexp::<Route>))"
            ),
        ),
        (
            "Either<Vec<String>, f64>",
            literal.clone(),
            format!(
                "{api}match_arg_either_or::<_,f64,_>(s,<Vec<String>as::miniextendr_api::TryFromSexp>::try_from_sexp)"
            ),
        ),
    ];
    for (ty, leaf, want) in cases {
        assert_eq!(decoder(ty, leaf), want, "decoder for `{ty}`");
    }
}

/// A layered choice parameter's `Err` arm is the argument error of #1591:
/// the full Rust type as `e$rust_type` and the crate class, on every layer
/// shape the decoder composes. Without an `Either` layer a choice type has no
/// static R-facing expectation, so the prefix is built on the failure path
/// from the error (`'mode' must be one of ...` for a `match_arg` choice error,
/// `NULL or` under `Option`, #1594). With one, see
/// `test_either_choice_err_arm_names_the_whole_parameter`.
#[test]
fn test_layered_choice_err_arm_is_the_argument_error() {
    for (src, leaf, rust_type, nullable) in [
        (
            "mode: Missing<Option<Mode>>",
            ChoiceLeaf::MatchArg,
            "Missing<Option<Mode>>",
            true,
        ),
        (
            "mode: Missing<Vec<Mode>>",
            ChoiceLeaf::MatchArgSeveral,
            "Missing<Vec<Mode>>",
            false,
        ),
    ] {
        let builder = RustConversionBuilder::new()
            .with_layered_choice("mode".to_string(), leaf)
            .with_conversion_error_class(vec!["pkg_error_argument".to_string()]);
        let s = conversion_text(&builder, src);
        assert!(s.contains("match_arg_"), "{src}: {s}");
        assert!(
            s.contains("__mx_conversion_expectation ! (e)"),
            "{src}: {s}"
        );
        assert!(
            s.contains(&format!(
                "conversion_prefix (\"mode\" , {nullable} , __mx_expected . as_deref () ,)"
            )),
            "{src}: {s}"
        );
        assert!(
            s.contains(&format!(
                "\"mode\" , :: core :: option :: Option :: Some (\"{rust_type}\") , & [\"pkg_error_argument\"]"
            )),
            "{src}: {s}"
        );
        assert!(
            s.contains("__mx_conversion_err_parts ! (e , __mx_expected . is_some ())"),
            "{src}: {s}"
        );
    }
}

/// A choice parameter with an `Either<.., R>` layer is refused against the
/// whole parameter, in the words of its `@param` line: the choices
/// (`MatchArg::CHOICES` of a `match_arg` type, the literal list of
/// `choices(...)`, `one or more of` for `several_ok`), then the `R` arm's
/// noun and `NULL` under `Option`. The error supplies only the reason.
#[test]
fn test_either_choice_err_arm_names_the_whole_parameter() {
    let literal = |several| ChoiceLeaf::Literal {
        choices: vec!["low".into(), "mid".into()],
        several,
    };
    let routes = "< Route as :: miniextendr_api :: MatchArg > :: CHOICES";
    for (src, leaf, choices, several, suffix) in [
        (
            "mode: Either<Route, DataFrame>",
            ChoiceLeaf::MatchArg,
            routes,
            false,
            ", or a data frame",
        ),
        (
            "mode: Either<Vec<Route>, DataFrame>",
            ChoiceLeaf::MatchArgSeveral,
            routes,
            true,
            ", or a data frame",
        ),
        (
            "mode: Option<Either<Route, DataFrame>>",
            ChoiceLeaf::MatchArg,
            routes,
            false,
            ", a data frame, or NULL",
        ),
        (
            "mode: Missing<Option<Either<Vec<Route>, DataFrame>>>",
            ChoiceLeaf::MatchArgSeveral,
            routes,
            true,
            ", a data frame, or NULL",
        ),
        (
            "mode: Either<String, f64>",
            literal(false),
            "& [\"low\" , \"mid\"]",
            false,
            ", or a number",
        ),
        (
            "mode: Option<Either<String, f64>>",
            literal(false),
            "& [\"low\" , \"mid\"]",
            false,
            ", a number, or NULL",
        ),
        (
            "mode: Missing<Either<Vec<String>, f64>>",
            literal(true),
            "& [\"low\" , \"mid\"]",
            true,
            ", or a number",
        ),
    ] {
        let builder = RustConversionBuilder::new()
            .with_layered_choice("mode".to_string(), leaf)
            .with_conversion_error_class(vec!["pkg_error_argument".to_string()]);
        let s = conversion_text(&builder, src);
        assert!(s.contains("match_arg_either_or"), "{src}: {s}");
        assert!(
            s.contains(&format!(
                "format ! (\"'{{}}' must be {{}}\" , \"mode\" , :: miniextendr_api :: match_arg :: choice_expectation ({choices} , {several} , \"{suffix}\"))"
            )),
            "{src}: {s}"
        );
        assert!(
            s.contains("__mx_conversion_err_parts ! (e , true)"),
            "{src}: {s}"
        );
        assert!(!s.contains("__mx_conversion_expectation"), "{src}: {s}");
        let rust_type = src.trim_start_matches("mode: ");
        assert!(
            s.contains(&format!(
                "\"mode\" , :: core :: option :: Option :: Some (\"{rust_type}\") , & [\"pkg_error_argument\"]"
            )),
            "{src}: {s}"
        );
    }
}

#[test]
fn native_metadata_matches_selected_conversion_paths() {
    let arg = syn::Ident::new("arg_0", proc_macro2::Span::call_site());
    for (ty, specialized) in [
        ("()", true),
        ("&Dots", true),
        ("&str", true),
        ("Call", true),
        ("miniextendr_api::CallerCall", true),
        ("i64", true),
        ("u16", false),
        ("Vec<Mode>", true),
        ("Box<[Mode]>", true),
        ("[Mode; 2]", true),
        ("&[Mode]", true),
        ("&mut i32", false),
        ("HiddenBorrow", false),
    ] {
        let builder = RustConversionBuilder::new()
            .with_strict()
            .with_coerce_all()
            .with_match_arg_several_ok("x".into());
        let syn::FnArg::Typed(param) = parse_param(&format!("x: {ty}")) else {
            unreachable!()
        };
        assert_eq!(
            builder.native_borrow_metadata(&param, &arg).is_none(),
            specialized,
            "{ty}"
        );
    }
    // A layered choice parameter (#1551) decodes through `match_arg_*` helpers.
    let builder =
        RustConversionBuilder::new().with_layered_choice("x".into(), ChoiceLeaf::MatchArg);
    let syn::FnArg::Typed(param) = parse_param("x: Option<Mode>") else {
        unreachable!()
    };
    assert!(builder.native_borrow_metadata(&param, &arg).is_none());
    // Numeric coercion now delegates to TryFromSexp, as do these fallback paths.
    for ty in [
        "u16",
        "Vec<u16>",
        "&mut i32",
        "Option<&mut i32>",
        "Box<[&mut i32]>",
        "HiddenBorrow",
    ] {
        let builder = RustConversionBuilder::new().with_coerce_all();
        let syn::FnArg::Typed(param) = parse_param(&format!("x: {ty}")) else {
            unreachable!()
        };
        assert!(
            builder.native_borrow_metadata(&param, &arg).is_some(),
            "{ty}"
        );
    }
}

// region: post-conversion no_na check

/// The emitted post-conversion `no_na` check of binding `x`.
const NO_NA_CHECK: &str = ":: miniextendr_api :: TryFromSexp :: __mx_has_na (& x)";

/// A `no_na` parameter converted through plain `TryFromSexp` gets the check
/// right after its binding, returning the R guard's condition with the
/// generated message: `be` for a scalar marker, `contain` for a vector one.
#[test]
fn no_na_checks_the_converted_value() {
    for (src, message) in [
        ("x: AsNumeric", "'x' must not be NA"),
        ("x: AsNumericVec", "'x' must not contain NA"),
        ("x: Option<AsCharacterVec>", "'x' must not contain NA"),
        ("_x: Missing<AsCharacter>", "'x' must not be NA"),
    ] {
        let name = if src.starts_with('_') { "_x" } else { "x" };
        let builder = RustConversionBuilder::new().with_no_na(name.to_string(), None);
        let s = conversion_text(&builder, src);
        let check = NO_NA_CHECK.replace("& x", &format!("& {name}"));
        assert!(s.contains(&check), "{src}: {s}");
        assert!(
            s.contains(&format!(
                "arg_check_condition_value (\"{message}\" , \"x\" , & [] , Some (__miniextendr_call) ,)"
            )),
            "{src}: {s}"
        );
        // The check follows the conversion binding.
        let binding = s.find(&format!("let {name}")).expect("the binding");
        assert!(binding < s.find(&check).unwrap(), "{src}: {s}");
    }
}

/// No name gate: the check is emitted for a type the macro does not know (an
/// alias of a marker) and for a plain type, where the trait default `false`
/// optimises it out. Rust resolves `__mx_has_na` on the real type.
#[test]
fn no_na_check_is_type_driven_not_name_matched() {
    let builder = RustConversionBuilder::new().with_no_na("x".to_string(), None);
    for src in ["x: MyDoseAlias", "x: f64", "x: HashMap<String, Dose>"] {
        let s = conversion_text(&builder, src);
        assert!(s.contains(NO_NA_CHECK), "{src}: {s}");
    }
}

/// `no_na(message = "...")` reaches the check verbatim, and the crate class is
/// passed like on a conversion failure.
#[test]
fn no_na_check_uses_the_custom_message_and_crate_class() {
    let builder = RustConversionBuilder::new()
        .with_no_na("dose".to_string(), Some("need a dose".to_string()))
        .with_conversion_error_class(vec!["pkg_error_argument".to_string()]);
    let s = conversion_text(&builder, "dose: AsNumeric");
    assert!(
        s.contains(
            "arg_check_condition_value (\"need a dose\" , \"dose\" , & [\"pkg_error_argument\"] , Some (__miniextendr_call) ,)"
        ),
        "{s}"
    );
}

/// Without `no_na`, or for another parameter, no check is emitted.
#[test]
fn no_na_check_only_for_listed_params() {
    let s = conversion_text(&RustConversionBuilder::new(), "x: AsNumeric");
    assert!(!s.contains("__mx_has_na"), "{s}");
    let builder = RustConversionBuilder::new().with_no_na("y".to_string(), None);
    let s = conversion_text(&builder, "x: AsNumeric");
    assert!(!s.contains("__mx_has_na"), "{s}");
}

/// Under the worker split the check is an owned (pre-closure) statement: it
/// runs on the main thread, where it may allocate the condition.
#[test]
fn no_na_check_is_in_the_owned_vector_of_the_split() {
    let builder = RustConversionBuilder::new().with_no_na("x".to_string(), None);
    let syn::FnArg::Typed(pat_type) = parse_param("x: AsNumericVec") else {
        unreachable!()
    };
    let sexp_ident = syn::Ident::new("arg_0", proc_macro2::Span::call_site());
    let (owned, borrowed) = builder.build_conversion_split(&pat_type, &sexp_ident);
    assert_eq!(owned.len(), 2);
    assert!(borrowed.is_empty());
    assert!(owned[1].to_string().contains(NO_NA_CHECK), "{}", owned[1]);
}

/// The special conversion arms never carry the check: a coerced type (one
/// with a coercion mapping), a borrowed slice, a `&T` borrow and a strict
/// lossy integer are converted by their own helpers and keep only the R guard.
/// A marker with `coerce` has no coercion mapping, so it takes the plain arm
/// and is checked.
#[test]
fn no_na_check_skips_the_special_arms() {
    let coerced = RustConversionBuilder::new()
        .with_coerce_param("x".to_string())
        .with_no_na("x".to_string(), None);
    assert!(!conversion_text(&coerced, "x: f64").contains("__mx_has_na"));
    assert!(conversion_text(&coerced, "x: AsNumeric").contains(NO_NA_CHECK));

    let builder = RustConversionBuilder::new().with_no_na("x".to_string(), None);
    assert!(!conversion_text(&builder, "x: &[f64]").contains("__mx_has_na"));
    assert!(!conversion_text(&builder, "x: &i32").contains("__mx_has_na"));

    let strict = RustConversionBuilder::new()
        .with_strict()
        .with_no_na("x".to_string(), None);
    assert!(!conversion_text(&strict, "x: i64").contains("__mx_has_na"));
}

// endregion
