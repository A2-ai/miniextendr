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

/// A type without an R-facing expectation: the `Err` arm names the parameter
/// by its R formal (`_x` → `x`) in the `invalid '<p>' argument` prefix, passes
/// the Rust type (with source spacing) as `e$rust_type` rather than in the
/// message, probes without an expectation, and passes an empty crate class
/// list by default (#1591).
#[test]
fn test_conversion_err_arm_fallback_prefix_and_rust_type() {
    let s = conversion_text(&RustConversionBuilder::new(), "_nums: AsFromStrVec<i32>");
    assert!(s.contains("conversion_condition_value"), "{s}");
    assert!(s.contains("__mx_conversion_err_parts ! (e , false)"), "{s}");
    assert!(
        s.contains("\"invalid 'nums' argument\" , \"nums\" , :: core :: option :: Option :: Some (\"AsFromStrVec<i32>\") , & []"),
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
            ChoiceLeaf::Literal,
            format!(
                "{api}match_arg_either_or::<_,f64,_>(s,<Stringas::miniextendr_api::TryFromSexp>::try_from_sexp)"
            ),
        ),
    ];
    for (ty, leaf, want) in cases {
        assert_eq!(decoder(ty, leaf), want, "decoder for `{ty}`");
    }
}

/// A layered choice parameter's `Err` arm is the argument error of #1591:
/// the `invalid '<p>' argument` prefix (a choice type has no R-facing
/// expectation), the full Rust type as `e$rust_type`, and the crate class,
/// on every layer shape the decoder composes.
#[test]
fn test_layered_choice_err_arm_is_the_argument_error() {
    for (src, leaf, rust_type) in [
        (
            "mode: Missing<Option<Mode>>",
            ChoiceLeaf::MatchArg,
            "Missing<Option<Mode>>",
        ),
        (
            "mode: Missing<Vec<Mode>>",
            ChoiceLeaf::MatchArgSeveral,
            "Missing<Vec<Mode>>",
        ),
        (
            "mode: Either<Route, DataFrame>",
            ChoiceLeaf::MatchArg,
            "Either<Route, DataFrame>",
        ),
        (
            "mode: Option<Either<String, f64>>",
            ChoiceLeaf::Literal,
            "Option<Either<String, f64>>",
        ),
    ] {
        let builder = RustConversionBuilder::new()
            .with_layered_choice("mode".to_string(), leaf)
            .with_conversion_error_class(vec!["pkg_error_argument".to_string()]);
        let s = conversion_text(&builder, src);
        assert!(s.contains("match_arg_"), "{src}: {s}");
        assert!(
            s.contains(&format!(
                "\"invalid 'mode' argument\" , \"mode\" , :: core :: option :: Option :: Some (\"{rust_type}\") , & [\"pkg_error_argument\"]"
            )),
            "{src}: {s}"
        );
        assert!(
            s.contains("__mx_conversion_err_parts ! (e , false)"),
            "{src}: {s}"
        );
    }
}
