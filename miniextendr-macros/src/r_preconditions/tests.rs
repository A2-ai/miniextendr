use super::*;

/// Helper to parse a type string into syn::Type
fn parse_type(s: &str) -> syn::Type {
    syn::parse_str(s).unwrap()
}

/// Helper to get assertions for a type.
fn assertions_for(ty_str: &str, param: &str) -> Vec<RAssertion> {
    let ty = parse_type(ty_str);
    r_check_for_type(&ty).unwrap().assertions(param)
}

#[test]
fn scalar_numeric_produces_two_assertions() {
    let asserts = assertions_for("i64", "x");
    assert_eq!(asserts.len(), 2);
    assert_eq!(asserts[0].message(), "'x' must be numeric, logical, or raw");
    assert_eq!(
        asserts[0].condition,
        "is.numeric(x) || is.logical(x) || is.raw(x)"
    );
    assert_eq!(asserts[1].message(), "'x' must have length 1");
    assert_eq!(asserts[1].condition, "length(x) == 1L");
}

#[test]
fn native_scalars_use_their_storage_gate() {
    // `i32` / `f64` convert from one SEXPTYPE, so the gate says so (#1112).
    let asserts = assertions_for("i32", "x");
    assert_eq!(asserts.len(), 2);
    assert_eq!(asserts[0].message(), "'x' must be integer");
    assert_eq!(asserts[0].condition, "is.integer(x)");
    let asserts = assertions_for("f64", "x");
    assert_eq!(asserts[0].message(), "'x' must be double");
    assert_eq!(asserts[0].condition, "is.double(x)");
    let asserts = assertions_for("Vec<f64>", "x");
    assert_eq!(asserts.len(), 1);
    assert_eq!(asserts[0].condition, "is.double(x)");
    let asserts = assertions_for("&[f64]", "x");
    assert_eq!(asserts[0].condition, "is.double(x)");
}

#[test]
fn all_signed_numeric_types_use_scalar_numeric() {
    for ty_str in &["f32", "i8", "i16", "i64", "isize"] {
        let asserts = assertions_for(ty_str, "x");
        assert_eq!(asserts.len(), 2, "{} should produce 2 assertions", ty_str);
        assert!(
            asserts[0].condition.contains("is.numeric(x)"),
            "{} type check",
            ty_str
        );
        assert!(
            asserts[0].condition.contains("is.logical(x)"),
            "{} accepts logical",
            ty_str
        );
        assert!(
            asserts[0].condition.contains("is.raw(x)"),
            "{} accepts raw",
            ty_str
        );
    }
}

#[test]
fn scalar_non_neg_produces_three_assertions() {
    let asserts = assertions_for("u32", "n");
    assert_eq!(asserts.len(), 3);
    assert_eq!(asserts[0].message(), "'n' must be numeric, logical, or raw");
    assert_eq!(asserts[1].message(), "'n' must have length 1");
    assert_eq!(asserts[2].message(), "'n' must be non-negative");
    assert_eq!(asserts[2].condition, "is.raw(n) || n >= 0");
}

#[test]
fn all_unsigned_types_use_scalar_non_neg() {
    for ty_str in &["u16", "u32", "u64", "usize"] {
        let asserts = assertions_for(ty_str, "x");
        assert_eq!(asserts.len(), 3, "{} should produce 3 assertions", ty_str);
        assert!(
            asserts[2].condition.contains(">= 0"),
            "{} non-neg check",
            ty_str
        );
    }
}

#[test]
fn scalar_logical() {
    let asserts = assertions_for("bool", "x");
    assert_eq!(asserts.len(), 2);
    assert_eq!(asserts[0].message(), "'x' must be logical");
    assert_eq!(asserts[0].condition, "is.logical(x)");
    assert_eq!(asserts[1].condition, "length(x) == 1L");
}

#[test]
fn scalar_character() {
    for ty_str in &["String", "char", "PathBuf"] {
        let asserts = assertions_for(ty_str, "s");
        assert_eq!(asserts.len(), 2);
        assert_eq!(asserts[0].message(), "'s' must be character");
        assert_eq!(asserts[0].condition, "is.character(s)");
    }
}

#[test]
fn ref_str() {
    let ty: syn::Type = syn::parse_str("& str").unwrap();
    let asserts = r_check_for_type(&ty).unwrap().assertions("s");
    assert_eq!(asserts.len(), 2);
    assert_eq!(asserts[0].condition, "is.character(s)");
}

#[test]
fn scalar_raw() {
    let asserts = assertions_for("u8", "x");
    assert_eq!(asserts.len(), 2);
    assert_eq!(asserts[0].message(), "'x' must be raw");
    assert_eq!(asserts[0].condition, "is.raw(x)");
}

#[test]
fn vector_float_stays_loose() {
    // `Vec<f32>` coerces from every numeric source, so it keeps the loose predicate.
    let asserts = assertions_for("Vec<f32>", "x");
    assert_eq!(asserts.len(), 1);
    assert_eq!(
        asserts[0].condition,
        "is.numeric(x) || is.logical(x) || is.raw(x)"
    );
}

#[test]
fn vector_i32_is_intsxp_strict() {
    // `Vec<i32>` is the only INTSXP-only integer vector: the native inbound
    // conversion rejects REALSXP, so the R gate is `is.integer(x)` — a clean
    // boundary rejection for any double (issue #616).
    let asserts = assertions_for("Vec<i32>", "x");
    assert_eq!(asserts.len(), 1);
    assert_eq!(asserts[0].condition, "is.integer(x)");
    assert_eq!(asserts[0].message(), "'x' must be an integer vector");
}

#[test]
fn vector_integer_wide_is_lossless() {
    // Every non-i32 integer element type uses the coercing inbound path that
    // accepts whole REALSXP and rejects fractional → lossless whole-number gate.
    for ty_str in &[
        "Vec<i8>",
        "Vec<i16>",
        "Vec<u16>",
        "Vec<u32>",
        "Vec<i64>",
        "Vec<u64>",
        "Vec<isize>",
        "Vec<usize>",
    ] {
        let asserts = assertions_for(ty_str, "x");
        assert_eq!(asserts.len(), 1, "{} should produce 1 assertion", ty_str);
        assert_eq!(
            asserts[0].condition,
            "is.integer(x) || is.logical(x) || is.raw(x) || \
                 (is.numeric(x) && all(is.na(x) | x == trunc(x)))",
            "{}",
            ty_str
        );
        assert_eq!(
            asserts[0].message(),
            "'x' must be integer or whole-number numeric"
        );
    }
}

#[test]
fn slice_int_is_strict() {
    let ty: syn::Type = syn::parse_str("& [i32]").unwrap();
    let asserts = r_check_for_type(&ty).unwrap().assertions("x");
    assert_eq!(asserts.len(), 1);
    assert_eq!(asserts[0].condition, "is.integer(x)");
}

#[test]
fn coerced_native_int_widens_to_whole_number_gate() {
    let ty = parse_type("i32");
    let asserts = coerce_widened(r_check_for_type(&ty).unwrap(), &ty).assertions("x");
    assert_eq!(asserts.len(), 2);
    assert_eq!(
        asserts[0].message(),
        "'x' must be integer or whole-number numeric"
    );
    assert!(asserts[0].condition.contains("x == trunc(x)"));
    assert_eq!(asserts[1].condition, "length(x) == 1L");

    let ty = parse_type("Vec<i32>");
    let asserts = coerce_widened(r_check_for_type(&ty).unwrap(), &ty).assertions("x");
    assert_eq!(asserts.len(), 1);
    assert!(asserts[0].condition.contains("x == trunc(x)"));

    // Native doubles move from `is.double` to the loose numeric gate;
    // borrowed slices have no mapping and stay strict.
    for name in ["f64", "Vec<f64>"] {
        let ty = parse_type(name);
        let actual = coerce_widened(r_check_for_type(&ty).unwrap(), &ty).assertions("x");
        assert_eq!(
            actual[0].condition, "is.numeric(x) || is.logical(x) || is.raw(x)",
            "{name}"
        );
    }
    let ty = parse_type("&[i32]");
    let actual = coerce_widened(r_check_for_type(&ty).unwrap(), &ty).assertions("x");
    assert_eq!(actual[0].condition, "is.integer(x)");
}

#[test]
fn coerced_numeric_checks_are_preserved() {
    for name in ["Vec<u16>", "Vec<f32>", "Option<u16>", "Vec<Option<i64>>"] {
        let ty = parse_type(name);
        let expected = r_check_for_type(&ty).unwrap().assertions("x");
        let actual = coerce_widened(r_check_for_type(&ty).unwrap(), &ty).assertions("x");
        assert_eq!(actual.len(), expected.len());
        for (actual, expected) in actual.iter().zip(&expected) {
            assert_eq!(actual.condition, expected.condition);
        }
    }
}

/// `Option<T>` takes its scalar's widened check behind `is.null()`;
/// `Vec<Option<T>>` its vector's, which lets `NA` elements through and, like
/// `Vec<T>`'s, refuses `NULL` (#2274).
#[test]
fn coerced_optional_checks_widen_with_their_element() {
    for (name, scalar) in [
        ("Option<i32>", "i32"),
        ("Option<f64>", "f64"),
        ("Option<bool>", "bool"),
    ] {
        let ty = parse_type(name);
        let actual = coerce_widened(r_check_for_type(&ty).unwrap(), &ty).assertions("x");
        let scalar = parse_type(scalar);
        let expected = coerce_widened(r_check_for_type(&scalar).unwrap(), &scalar).assertions("x");
        assert_eq!(actual.len(), expected.len(), "{name}");
        for (actual, expected) in actual.iter().zip(&expected) {
            assert_eq!(
                actual.condition,
                format!("is.null(x) || {}", expected.condition)
            );
        }
    }
    for (name, vector) in [
        ("Vec<Option<i32>>", "Vec<i32>"),
        ("Vec<Option<f64>>", "Vec<f64>"),
        ("Vec<Option<bool>>", "Vec<bool>"),
    ] {
        let ty = parse_type(name);
        let actual = coerce_widened(r_check_for_type(&ty).unwrap(), &ty).assertions("x");
        let vector = parse_type(vector);
        let expected = coerce_widened(r_check_for_type(&vector).unwrap(), &vector).assertions("x");
        assert_eq!(actual.len(), 1, "{name}");
        assert_eq!(actual[0].condition, expected[0].condition, "{name}");
        assert!(!actual[0].condition.contains("is.null"), "{name}");
    }
    let ty = parse_type("Option<i32>");
    let check = coerce_widened(r_check_for_type(&ty).unwrap(), &ty);
    assert_eq!(check.expectation(), "NULL or a single whole number");
    assert_eq!(
        check.assertions("x")[0].message(),
        "'x' must be NULL or integer or whole-number numeric"
    );
}

#[test]
fn coerced_bool_checks_accept_logical_and_integer() {
    for name in ["bool", "Vec<bool>"] {
        let ty = parse_type(name);
        let actual = coerce_widened(r_check_for_type(&ty).unwrap(), &ty).assertions("x");
        assert_eq!(actual[0].condition, "is.logical(x) || is.integer(x)");
        assert_eq!(actual.len(), if name == "bool" { 2 } else { 1 });
    }
}

#[test]
fn build_checks_coerced_param_preserves_whole_number_check() {
    let sig: syn::Signature = syn::parse_str("fn f(x: Vec<u16>)").unwrap();
    let mut coerce_params = HashSet::new();
    coerce_params.insert("x".to_string());
    let opts = PreconditionOptions {
        coerce_params,
        ..Default::default()
    };
    let output = build_precondition_checks(&sig.inputs, &HashSet::new(), &opts);
    let joined = output.guards(None).join("\n");
    assert!(joined.contains("is.integer(x)"));
    assert!(joined.contains("trunc(x)"));
}

/// Every check is one guard raising the shared argument error (#1591):
/// the wrapper's own call by default, the captured `.mx_call` under
/// `call = caller` (#1548). The requirement leaves out the parameter name,
/// which the helper puts in front.
#[test]
fn guards_raise_the_argument_error_with_the_given_call() {
    let sig: syn::Signature = syn::parse_str("fn f(n: i32, xs: Vec<f64>)").unwrap();
    let output = build_precondition_checks(
        &sig.inputs,
        &HashSet::new(),
        &PreconditionOptions::default(),
    );
    assert_eq!(
        output.guards(None),
        vec![
            "if (!isTRUE(is.integer(n))) .miniextendr_arg_error(\"n\", \"must be integer\")",
            "if (!isTRUE(length(n) == 1L)) .miniextendr_arg_error(\"n\", \"must have length 1\")",
            "if (!isTRUE(is.double(xs))) .miniextendr_arg_error(\"xs\", \"must be double\")",
        ]
    );
    assert_eq!(
        output.guards(Some(".mx_call")),
        vec![
            "if (!isTRUE(is.integer(n))) .miniextendr_arg_error(\"n\", \"must be integer\", .mx_call)",
            "if (!isTRUE(length(n) == 1L)) .miniextendr_arg_error(\"n\", \"must have length 1\", .mx_call)",
            "if (!isTRUE(is.double(xs))) .miniextendr_arg_error(\"xs\", \"must be double\", .mx_call)",
        ]
    );
    let no_params: syn::Signature = syn::parse_str("fn g()").unwrap();
    let empty = build_precondition_checks(
        &no_params.inputs,
        &HashSet::new(),
        &PreconditionOptions::default(),
    );
    assert!(empty.guards(None).is_empty());
    assert!(empty.guards(Some(".mx_call")).is_empty());
}

/// The conversion-side expectation (#1591) reads the same classification:
/// scalars name one value, vectors the R type, `Option` adds `NULL`,
/// `coerce` widens with the gate, and types without a check have none.
#[test]
fn conversion_expectation_follows_the_type_check() {
    let exp = |ty: &str, coerced: bool| conversion_expectation(&parse_type(ty), coerced);
    let some = |s: &str| Some(s.to_string());
    assert_eq!(exp("AsNumericVec", false), some("numeric"));
    assert_eq!(exp("AsNumeric", false), some("a single number"));
    assert_eq!(
        exp("Option<AsNumeric>", false),
        some("NULL or a single number")
    );
    assert_eq!(exp("i32", false), some("a single integer"));
    assert_eq!(exp("i32", true), some("a single whole number"));
    assert_eq!(exp("f64", false), some("a single double"));
    assert_eq!(exp("f64", true), some("a single number"));
    assert_eq!(exp("i64", false), some("a single number"));
    assert_eq!(exp("u32", false), some("a single non-negative number"));
    assert_eq!(exp("bool", false), some("TRUE or FALSE"));
    assert_eq!(exp("Option<bool>", false), some("NULL, TRUE or FALSE"));
    assert_eq!(exp("bool", true), some("a single logical or integer"));
    assert_eq!(exp("String", false), some("a single string"));
    assert_eq!(exp("&str", false), some("a single string"));
    assert_eq!(exp("Cow<'_, str>", false), some("a single string"));
    assert_eq!(
        exp("Option<Cow<'_, str>>", false),
        some("NULL or a single string")
    );
    assert_eq!(exp("Cow<'_, [f64]>", false), some("double"));
    assert_eq!(exp("Vec<&str>", false), some("character"));
    assert_eq!(exp("Vec<Option<&str>>", false), some("character"));
    assert_eq!(exp("Vec<Cow<'_, str>>", false), some("character"));
    assert_eq!(exp("Vec<Option<Cow<'_, str>>>", false), some("character"));
    assert_eq!(exp("u8", false), some("a single raw value"));
    assert_eq!(exp("Rcomplex", false), some("a single complex value"));
    assert_eq!(exp("Vec<i32>", false), some("integer"));
    assert_eq!(
        exp("Vec<i32>", true),
        some("integer or whole-number numeric")
    );
    assert_eq!(
        exp("Vec<u16>", false),
        some("integer or whole-number numeric")
    );
    assert_eq!(exp("Vec<f64>", false), some("double"));
    assert_eq!(exp("&[f64]", false), some("double"));
    assert_eq!(exp("Vec<Option<String>>", false), some("character"));
    assert_eq!(exp("Vec<bool>", true), some("logical or integer"));
    assert_eq!(exp("HashMap<String, i32>", false), some("a list"));
    assert_eq!(exp("Missing<f64>", false), some("a single double"));
    assert_eq!(
        exp("AsCharacter", false),
        some("coercible to a single string")
    );
    assert_eq!(exp("AsCharacterVec", false), some("coercible to character"));
    assert_eq!(
        exp("Option<AsCharacter>", false),
        some("NULL or coercible to a single string")
    );
    // Types without an R-side check that still say what they accept.
    assert_eq!(exp("AsFromStr<IpAddr>", false), some("a single string"));
    assert_eq!(exp("AsFromStrVec<i32>", false), some("character"));
    assert_eq!(
        exp("Option<AsFromStr<IpAddr>>", false),
        some("NULL or a single string")
    );
    assert_eq!(
        exp("Either<i32, String>", false),
        some("a single integer or a single string")
    );
    assert_eq!(
        exp("Either<f64, Vec<i32>>", false),
        some("a single double or integer")
    );
    assert_eq!(
        exp("Either<bool, Either<i32, String>>", false),
        some("TRUE or FALSE or a single integer or a single string")
    );
    assert_eq!(
        exp("Option<Either<i32, String>>", false),
        some("NULL or a single integer or a single string")
    );
    assert_eq!(
        exp("Option<Either<f64, String>>", false),
        some("NULL or a single double or a single string")
    );
    assert_eq!(
        exp("Option<Either<AsNumericVec, DataFrame>>", false),
        some("NULL or numeric or a data frame")
    );
    assert_eq!(exp("(i32, String)", false), some("a list of length 2"));
    assert_eq!(exp("DataFrame", false), some("a data frame"));
    assert_eq!(
        exp("Option<DataFrame>", false),
        some("NULL or a data frame")
    );
    // Choice parameters, including the `Missing` / `Option` / `Either`
    // layers of #1551, have no static expectation: their conversion error
    // supplies it at run time (`one of "a", "b"`). An opaque side leaves
    // the whole `Either` without a static one; the wrapper words it side
    // by side at run time.
    for unknown in [
        "Hyperparams",
        "Either<i32, Hyperparams>",
        "SEXP",
        "()",
        "Mode",
        "Option<Mode>",
        "Missing<Option<Mode>>",
        "Missing<Vec<Mode>>",
        "Either<Route, DataFrame>",
        "Option<Either<Route, DataFrame>>",
        "Missing<Option<Either<Mode, List>>>",
        "Either<Vec<Mode>, DataFrame>",
        "Option<Either<Vec<Mode>, DataFrame>>",
    ] {
        assert_eq!(exp(unknown, false), None, "{unknown}");
    }
    // A `choices(...)` `several_ok` string list with another kind of value
    // (#1612).
    assert_eq!(
        exp("Either<Vec<String>, f64>", false),
        some("character or a single double")
    );
}

#[test]
fn strict_expectation_names_a_whole_number() {
    let exp = |ty: &str| strict_conversion_expectation(&parse_type(ty));
    let some = |s: &str| Some(s.to_string());
    assert_eq!(exp("i64"), some("a single whole number"));
    assert_eq!(exp("isize"), some("a single whole number"));
    assert_eq!(exp("u64"), some("a single non-negative whole number"));
    assert_eq!(exp("usize"), some("a single non-negative whole number"));
    assert_eq!(exp("Vec<i64>"), some("integer or whole-number numeric"));
    assert_eq!(
        exp("Vec<Option<u64>>"),
        some("integer or whole-number numeric")
    );
}

#[test]
fn vector_character() {
    let asserts = assertions_for("Vec<String>", "x");
    assert_eq!(asserts.len(), 1);
    assert_eq!(asserts[0].condition, "is.character(x)");
}

#[test]
fn vector_optional_string() {
    let asserts = assertions_for("Vec<Option<String>>", "x");
    assert_eq!(asserts.len(), 1);
    assert_eq!(asserts[0].condition, "is.character(x)");
}

#[test]
fn slice_u8() {
    let ty: syn::Type = syn::parse_str("& [u8]").unwrap();
    let asserts = r_check_for_type(&ty).unwrap().assertions("x");
    assert_eq!(asserts.len(), 1);
    assert_eq!(asserts[0].condition, "is.raw(x)");
}

#[test]
fn nullable_wraps_inner_assertions() {
    let asserts = assertions_for("Option<i64>", "x");
    assert_eq!(asserts.len(), 2);
    assert_eq!(
        asserts[0].message(),
        "'x' must be NULL or numeric, logical, or raw"
    );
    assert_eq!(
        asserts[0].condition,
        "is.null(x) || is.numeric(x) || is.logical(x) || is.raw(x)"
    );
    let asserts = assertions_for("Option<i32>", "x");
    assert_eq!(asserts[0].condition, "is.null(x) || is.integer(x)");
    assert_eq!(asserts[1].message(), "'x' must be NULL or have length 1");
    assert_eq!(asserts[1].condition, "is.null(x) || length(x) == 1L");
}

#[test]
fn nullable_character() {
    let asserts = assertions_for("Option<String>", "s");
    assert_eq!(asserts.len(), 2);
    assert_eq!(asserts[0].message(), "'s' must be NULL or character");
    assert_eq!(asserts[0].condition, "is.null(s) || is.character(s)");
    assert_eq!(asserts[1].message(), "'s' must be NULL or have length 1");
}

#[test]
fn as_numeric_markers_admit_numbers_text_and_factors() {
    let condition = "is.numeric(x) || is.logical(x) || is.character(x) || is.factor(x)";
    let asserts = assertions_for("AsNumeric", "x");
    assert_eq!(asserts.len(), 2);
    assert_eq!(
        asserts[0].message(),
        "'x' must be numeric, logical, character, or factor"
    );
    assert_eq!(asserts[0].condition, condition);
    assert_eq!(asserts[1].message(), "'x' must have length 1");
    assert_eq!(asserts[1].condition, "length(x) == 1L");

    // Path-qualified spellings resolve by their last segment.
    let asserts = assertions_for("miniextendr_api::AsNumericVec", "x");
    assert_eq!(asserts.len(), 1);
    assert_eq!(
        asserts[0].message(),
        "'x' must be numeric, logical, character, or factor"
    );
    assert_eq!(asserts[0].condition, condition);
}

#[test]
fn optional_as_numeric_is_nullable() {
    let asserts = assertions_for("Option<AsNumeric>", "x");
    assert_eq!(asserts.len(), 2);
    assert_eq!(
        asserts[0].message(),
        "'x' must be NULL or numeric, logical, character, or factor"
    );
    assert_eq!(
        asserts[0].condition,
        "is.null(x) || is.numeric(x) || is.logical(x) || is.character(x) || is.factor(x)"
    );
    assert_eq!(asserts[1].message(), "'x' must be NULL or have length 1");
    let asserts = assertions_for("Option<AsNumericVec>", "x");
    assert_eq!(asserts.len(), 1);
    assert!(asserts[0].condition.starts_with("is.null(x) || "));
}

#[test]
fn as_character_markers_admit_atomic_vectors() {
    let asserts = assertions_for("AsCharacter", "x");
    assert_eq!(asserts.len(), 2);
    assert_eq!(asserts[0].message(), "'x' must be atomic");
    assert_eq!(asserts[0].condition, "is.atomic(x)");
    assert_eq!(asserts[1].message(), "'x' must have length 1");
    assert_eq!(asserts[1].condition, "length(x) == 1L");

    // Path-qualified spellings resolve by their last segment.
    let asserts = assertions_for("miniextendr_api::AsCharacterVec", "x");
    assert_eq!(asserts.len(), 1);
    assert_eq!(asserts[0].message(), "'x' must be atomic");
    assert_eq!(asserts[0].condition, "is.atomic(x)");
}

#[test]
fn optional_as_character_is_nullable() {
    let asserts = assertions_for("Option<AsCharacter>", "x");
    assert_eq!(asserts.len(), 2);
    assert_eq!(asserts[0].message(), "'x' must be NULL or atomic");
    assert_eq!(asserts[0].condition, "is.null(x) || is.atomic(x)");
    assert_eq!(asserts[1].message(), "'x' must be NULL or have length 1");
    let asserts = assertions_for("Option<AsCharacterVec>", "x");
    assert_eq!(asserts.len(), 1);
    assert_eq!(asserts[0].message(), "'x' must be NULL or atomic");
    assert_eq!(asserts[0].condition, "is.null(x) || is.atomic(x)");
}

#[test]
fn as_character_vec_with_no_na_checks_na_then_type() {
    let out = explicit_output("fn f(x: AsCharacterVec)", &[("x", no_na())], false);
    assert_eq!(
        out.guards(None),
        vec![
            "if (!isTRUE(!anyNA(x))) .miniextendr_arg_error(\"x\", \"must not contain NA\")",
            "if (!isTRUE(is.atomic(x))) .miniextendr_arg_error(\"x\", \"must be atomic\")",
        ]
    );
}

#[test]
fn coerce_keeps_the_as_numeric_gate() {
    // `AsNumeric` has no coercion mapping: `coerce` leaves its gate alone.
    let ty = parse_type("AsNumericVec");
    let check = coerce_widened(r_check_for_type(&ty).unwrap(), &ty);
    let asserts = check.assertions("x");
    assert_eq!(asserts.len(), 1);
    assert!(asserts[0].condition.contains("is.factor(x)"));
}

#[test]
fn map_types() {
    for ty_str in &["HashMap<String, i32>", "BTreeMap<String, f64>"] {
        let ty = parse_type(ty_str);
        let asserts = r_check_for_type(&ty).unwrap().assertions("x");
        assert_eq!(asserts.len(), 1);
        assert_eq!(asserts[0].condition, "is.list(x)");
    }
}

#[test]
fn skip_types() {
    for ty_str in &["SEXP", "ExternalPtr<MyType>"] {
        let ty = parse_type(ty_str);
        assert!(
            r_check_for_type(&ty).is_none(),
            "{} should be skipped",
            ty_str
        );
    }
}

#[test]
fn scalar_param_produces_one_guard_per_check() {
    let sig: syn::Signature = syn::parse_str("fn f(n: i32)").unwrap();
    let output = build_precondition_checks(
        &sig.inputs,
        &HashSet::new(),
        &PreconditionOptions::default(),
    );
    let checks = output.guards(None);
    assert_eq!(checks.len(), 2);
    assert!(checks[0].contains("\"must be integer\""));
    assert!(checks[1].contains("\"must have length 1\""));
    assert!(output.fallback_params.is_empty());
}

#[test]
fn vector_param_single_guard() {
    // Vec<f64> produces 1 check → one guard line
    let sig: syn::Signature = syn::parse_str("fn f(x: Vec<f64>)").unwrap();
    let output = build_precondition_checks(
        &sig.inputs,
        &HashSet::new(),
        &PreconditionOptions::default(),
    );
    assert_eq!(
        output.guards(None),
        vec!["if (!isTRUE(is.double(x))) .miniextendr_arg_error(\"x\", \"must be double\")"]
    );
}

#[test]
fn two_scalar_params_produce_four_guards_in_order() {
    let sig: syn::Signature = syn::parse_str("fn f(a: i32, b: f64)").unwrap();
    let output = build_precondition_checks(
        &sig.inputs,
        &HashSet::new(),
        &PreconditionOptions::default(),
    );
    let checks = output.guards(None);
    // 2 checks per param
    assert_eq!(checks.len(), 4);
    assert!(checks[0].contains("(\"a\", ") && checks[0].contains("integer"));
    assert!(checks[1].contains("(\"a\", ") && checks[1].contains("length 1"));
    assert!(checks[2].contains("(\"b\", ") && checks[2].contains("double"));
    assert!(checks[3].contains("(\"b\", ") && checks[3].contains("length 1"));
}

#[test]
fn build_checks_skips_match_arg() {
    let sig: syn::Signature = syn::parse_str("fn f(n: i32, mode: String)").unwrap();
    let mut skip = HashSet::new();
    skip.insert("mode".to_string());
    let output = build_precondition_checks(&sig.inputs, &skip, &PreconditionOptions::default());
    // Only n's 2 checks remain
    let joined = output.guards(None).join("\n");
    assert!(joined.contains("(\"n\", "));
    assert!(!joined.contains("mode"));
}

#[test]
fn unknown_type_produces_fallback() {
    let sig: syn::Signature = syn::parse_str("fn f(x: MyCustomType)").unwrap();
    let output = build_precondition_checks(
        &sig.inputs,
        &HashSet::new(),
        &PreconditionOptions::default(),
    );
    assert!(output.guards(None).is_empty());
    assert_eq!(output.fallback_params.len(), 1);
    assert_eq!(output.fallback_params[0].r_name, "x");
}

#[test]
fn mixed_known_and_unknown_types() {
    let sig: syn::Signature = syn::parse_str("fn f(a: i32, b: MyType, c: String)").unwrap();
    let output = build_precondition_checks(
        &sig.inputs,
        &HashSet::new(),
        &PreconditionOptions::default(),
    );
    // a (i32) and c (String) are known → checks
    let joined = output.guards(None).join("\n");
    assert!(joined.contains("(\"a\", "));
    assert!(joined.contains("(\"c\", "));
    assert!(!joined.contains("(\"b\", "));
    // b (MyType) is unknown → fallback
    assert_eq!(output.fallback_params.len(), 1);
    assert_eq!(output.fallback_params[0].r_name, "b");
}

#[test]
fn sexp_not_fallback() {
    let sig: syn::Signature = syn::parse_str("fn f(x: SEXP)").unwrap();
    let output = build_precondition_checks(
        &sig.inputs,
        &HashSet::new(),
        &PreconditionOptions::default(),
    );
    assert!(output.guards(None).is_empty());
    assert!(output.fallback_params.is_empty());
}

/// Build checks for `sig` with the given explicit checks keyed by R name.
/// The checks of `sig` with the named `explicit` checks; `no_type_checks`
/// drops every parameter's type-derived checks.
fn explicit_output(
    sig: &str,
    explicit: &[(&str, ExplicitChecks)],
    no_type_checks: bool,
) -> PreconditionOutput {
    let sig: syn::Signature = syn::parse_str(sig).unwrap();
    let unchecked = if no_type_checks {
        unchecked_params(
            &sig.inputs,
            &Default::default(),
            |_| Some(false),
            None,
            None,
            None,
        )
    } else {
        HashSet::new()
    };
    let opts = PreconditionOptions {
        explicit: explicit
            .iter()
            .map(|(name, checks)| (name.to_string(), checks.clone()))
            .collect(),
        unchecked,
        ..Default::default()
    };
    build_precondition_checks(&sig.inputs, &HashSet::new(), &opts)
}

/// The precedence table of `resolve_type_checks`: the first spelling that
/// speaks wins, from the parameter down to the feature; the framework
/// default keeps the checks. `(param, item, impl, crate, feature) → keep`.
#[test]
fn resolve_type_checks_follows_the_precedence_table() {
    /// `(param, item, impl, crate, feature, keep)`.
    type Case = (
        Option<bool>,
        Option<bool>,
        Option<bool>,
        Option<bool>,
        bool,
        bool,
    );
    let cases: &[Case] = &[
        // Rank 6: nothing speaks.
        (None, None, None, None, false, true),
        // Rank 5: the feature drops them.
        (None, None, None, None, true, false),
        // Rank 4 over rank 5: the crate default beats the feature.
        (None, None, None, Some(true), true, true),
        (None, None, None, Some(false), false, false),
        // Rank 3 over 4 and 5: the impl block.
        (None, None, Some(true), Some(false), true, true),
        (None, None, Some(false), Some(true), false, false),
        // Rank 2 over 3: the fn / method keyword.
        (None, Some(true), Some(false), Some(false), true, true),
        (None, Some(false), Some(true), Some(true), false, false),
        // Rank 1 over everything: marker, per-param keyword, method list.
        (
            Some(true),
            Some(false),
            Some(false),
            Some(false),
            true,
            true,
        ),
        (
            Some(false),
            Some(true),
            Some(true),
            Some(true),
            false,
            false,
        ),
    ];
    for &(param, item, impl_level, crate_default, feature, keep) in cases {
        assert_eq!(
            resolve_type_checks(param, item, impl_level, crate_default, feature),
            keep,
            "{param:?} {item:?} {impl_level:?} {crate_default:?} {feature}"
        );
    }
    // The item-level decision is ranks 2 to 6.
    assert!(!resolve_item_type_checks(None, None, Some(false), false));
    assert!(resolve_item_type_checks(
        Some(true),
        Some(false),
        None,
        true
    ));
    assert!(!resolve_item_type_checks(None, None, None, true));
}

/// `unchecked` drops one parameter's type-derived checks and keeps the
/// others', and keeps that parameter's named checks.
#[test]
fn unchecked_drops_one_parameter_and_keeps_its_named_checks() {
    let sig: syn::Signature = syn::parse_str("fn f(n: i32, xs: Vec<f64>)").unwrap();
    let unchecked = unchecked_params(
        &sig.inputs,
        &Default::default(),
        |name| (name == "xs").then_some(false),
        None,
        None,
        Some(true),
    );
    assert_eq!(unchecked, ["xs".to_string()].into_iter().collect());
    let opts = PreconditionOptions {
        explicit: [("xs".to_string(), no_na())].into_iter().collect(),
        unchecked,
        ..Default::default()
    };
    let guards = build_precondition_checks(&sig.inputs, &HashSet::new(), &opts).guards(None);
    assert_eq!(
        guards,
        vec![
            "if (!isTRUE(is.integer(n))) .miniextendr_arg_error(\"n\", \"must be integer\")",
            "if (!isTRUE(length(n) == 1L)) .miniextendr_arg_error(\"n\", \"must have length 1\")",
            "if (!isTRUE(!anyNA(xs))) .miniextendr_arg_error(\"xs\", \"must not contain NA\")",
        ]
    );
    // The set holds R names: `_n` is the R formal `n`.
    let sig: syn::Signature = syn::parse_str("fn f(_n: i32)").unwrap();
    let unchecked = unchecked_params(
        &sig.inputs,
        &Default::default(),
        |_| None,
        Some(false),
        None,
        None,
    );
    assert_eq!(unchecked, ["n".to_string()].into_iter().collect());
}

/// An item-level `no_preconditions` leaves `no_na` alone: a plain
/// parameter keeps its R guard, and an `Either` (which has no type-derived
/// check, so no rank-1 spelling) still gets none, its NA check running in
/// Rust after the conversion.
#[test]
fn item_no_preconditions_keeps_no_na_and_either_stays_rust_side() {
    let sig: syn::Signature =
        syn::parse_str("fn f(x: Either<AsNumeric, DataFrame>, y: f64)").unwrap();
    let opts = PreconditionOptions {
        explicit: [("x".to_string(), no_na()), ("y".to_string(), no_na())]
            .into_iter()
            .collect(),
        unchecked: unchecked_params(
            &sig.inputs,
            &Default::default(),
            |_| None,
            Some(false),
            None,
            None,
        ),
        ..Default::default()
    };
    let guards = build_precondition_checks(&sig.inputs, &HashSet::new(), &opts).guards(None);
    assert_eq!(
        guards,
        vec!["if (!isTRUE(!anyNA(y))) .miniextendr_arg_error(\"y\", \"must not be NA\")"]
    );
}

#[test]
fn has_type_check_is_false_for_unguarded_types() {
    for guarded in [
        "i32",
        "Option<f64>",
        "&str",
        "&[f64]",
        "Vec<String>",
        "u16",
        "List",
    ] {
        assert!(has_type_check(&parse_type(guarded)), "{guarded}");
    }
    for bare in [
        "SEXP",
        "Missing<i32>",
        "ExternalPtr<T>",
        "&Dots",
        "MyType",
        "Call",
        "Either<AsNumeric, DataFrame>",
    ] {
        assert!(!has_type_check(&parse_type(bare)), "{bare}");
    }
}

fn inherits(classes: &[&str]) -> ExplicitChecks {
    ExplicitChecks {
        inherits: Some(classes.iter().map(|c| c.to_string()).collect()),
        ..Default::default()
    }
}

fn no_na() -> ExplicitChecks {
    ExplicitChecks {
        no_na: true,
        ..Default::default()
    }
}

fn with_inherits_message(classes: &[&str], message: &str) -> ExplicitChecks {
    ExplicitChecks {
        inherits_message: Some(message.to_string()),
        ..inherits(classes)
    }
}

fn with_no_na_message(message: &str) -> ExplicitChecks {
    ExplicitChecks {
        no_na_message: Some(message.to_string()),
        ..no_na()
    }
}

#[test]
fn inherits_precedes_the_type_check() {
    let out = explicit_output("fn f(x: List)", &[("x", inherits(&["pkg_obj"]))], false);
    assert_eq!(
        out.guards(None),
        vec![
            "if (!isTRUE(inherits(x, \"pkg_obj\"))) .miniextendr_arg_error(\"x\", \"must inherit from 'pkg_obj'\")",
            "if (!isTRUE(is.list(x))) .miniextendr_arg_error(\"x\", \"must be a list\")",
        ]
    );
}

/// An `inherits` message is the message of every type check on the
/// parameter too: storage and length alike.
#[test]
fn inherits_message_covers_the_type_checks() {
    let out = explicit_output(
        "fn f(x: f64)",
        &[("x", with_inherits_message(&["pkg_unit"], "need a pkg_unit"))],
        false,
    );
    assert_eq!(
        out.guards(None),
        vec![
            "if (!isTRUE(inherits(x, \"pkg_unit\"))) .miniextendr_arg_error(\"x\", message = \"need a pkg_unit\")",
            "if (!isTRUE(is.double(x))) .miniextendr_arg_error(\"x\", message = \"need a pkg_unit\")",
            "if (!isTRUE(length(x) == 1L)) .miniextendr_arg_error(\"x\", message = \"need a pkg_unit\")",
        ]
    );
}

#[test]
fn inherits_message_on_option() {
    let out = explicit_output(
        "fn f(x: Option<List>)",
        &[("x", with_inherits_message(&["pkg_obj"], "need a pkg_obj"))],
        false,
    );
    assert_eq!(
        out.guards(None),
        vec![
            "if (!isTRUE(is.null(x) || inherits(x, \"pkg_obj\"))) .miniextendr_arg_error(\"x\", message = \"need a pkg_obj\")",
            "if (!isTRUE(is.null(x) || is.list(x))) .miniextendr_arg_error(\"x\", message = \"need a pkg_obj\")",
        ]
    );
}

/// The `inherits` message does not cover the NA check, which keeps its
/// generated message and runs first.
#[test]
fn no_na_keeps_its_own_message() {
    let mut checks = with_inherits_message(&["pkg_num"], "need a pkg_num");
    checks.merge(no_na()).unwrap();
    let out = explicit_output("fn f(x: Vec<f64>)", &[("x", checks)], false);
    assert_eq!(
        out.guards(None),
        vec![
            "if (!isTRUE(!anyNA(x))) .miniextendr_arg_error(\"x\", \"must not contain NA\")",
            "if (!isTRUE(inherits(x, \"pkg_num\"))) .miniextendr_arg_error(\"x\", message = \"need a pkg_num\")",
            "if (!isTRUE(is.double(x))) .miniextendr_arg_error(\"x\", message = \"need a pkg_num\")",
        ]
    );
}

/// `coerce` widens the type checks; the widened checks carry the message.
#[test]
fn inherits_message_with_coerce() {
    let sig: syn::Signature = syn::parse_str("fn f(x: i32)").unwrap();
    let opts = PreconditionOptions {
        coerce_all: true,
        explicit: [(
            "x".to_string(),
            with_inherits_message(&["pkg_count"], "need a pkg_count"),
        )]
        .into_iter()
        .collect(),
        ..Default::default()
    };
    let out = build_precondition_checks(&sig.inputs, &HashSet::new(), &opts);
    let guards = out.guards(None);
    assert_eq!(guards.len(), 3, "{guards:?}");
    assert!(guards[0].starts_with("if (!isTRUE(inherits(x, \"pkg_count\")))"));
    assert!(
        guards[1].contains("is.integer(x) || is.logical(x) || is.raw(x)"),
        "{}",
        guards[1]
    );
    assert!(guards[2].contains("length(x) == 1L"), "{}", guards[2]);
    for guard in &guards {
        assert!(
            guard.ends_with(".miniextendr_arg_error(\"x\", message = \"need a pkg_count\")"),
            "{guard}"
        );
    }
}

/// `Missing<T>` has no R type check, so the class check is its only guard.
#[test]
fn inherits_message_on_missing() {
    let out = explicit_output(
        "fn f(x: Missing<List>)",
        &[("x", with_inherits_message(&["pkg_obj"], "need a pkg_obj"))],
        false,
    );
    assert_eq!(
        out.guards(None),
        vec![
            "if (!isTRUE(missing(x) || inherits(x, \"pkg_obj\"))) .miniextendr_arg_error(\"x\", message = \"need a pkg_obj\")"
        ]
    );
}

#[test]
fn inherits_any_of_several_classes() {
    let out = explicit_output("fn f(x: SEXP)", &[("x", inherits(&["a", "b", "c"]))], false);
    assert_eq!(
        out.guards(None),
        vec![
            "if (!isTRUE(inherits(x, c(\"a\", \"b\", \"c\")))) .miniextendr_arg_error(\"x\", \"must inherit from 'a', 'b' or 'c'\")"
        ]
    );
}

#[test]
fn inherits_escapes_quotes_and_backslashes() {
    let out = explicit_output("fn f(x: SEXP)", &[("x", inherits(&["a\"b\\c"]))], false);
    assert_eq!(
        out.guards(None),
        vec![
            "if (!isTRUE(inherits(x, \"a\\\"b\\\\c\"))) .miniextendr_arg_error(\"x\", \"must inherit from 'a\\\"b\\\\c'\")"
        ]
    );
}

fn not_inherits(classes: &[&str]) -> ExplicitChecks {
    ExplicitChecks {
        not_inherits: Some(classes.iter().map(|c| c.to_string()).collect()),
        ..Default::default()
    }
}

fn with_not_inherits_message(classes: &[&str], message: &str) -> ExplicitChecks {
    ExplicitChecks {
        not_inherits_message: Some(message.to_string()),
        ..not_inherits(classes)
    }
}

/// The #1815 case: `not_inherits` with a message on an `Option<AsNumeric>`
/// with `no_na`. The refusal runs before the marker's type checks
/// (`is.numeric()` is `FALSE` for these classes, so after them it would
/// never be reached), after the NA check, passes `NULL`, and leaves the
/// type and NA checks their generated messages.
#[test]
fn not_inherits_precedes_the_type_checks_and_keeps_their_messages() {
    let mut checks = with_not_inherits_message(
        &["difftime", "Date", "POSIXt"],
        "`tau` must be a plain number",
    );
    checks.merge(no_na()).unwrap();
    let out = explicit_output("fn f(tau: Option<AsNumeric>)", &[("tau", checks)], false);
    assert_eq!(
        out.guards(None),
        vec![
            "if (!isTRUE(is.null(tau) || !anyNA(tau))) .miniextendr_arg_error(\"tau\", \"must not be NA\")",
            "if (!isTRUE(is.null(tau) || !inherits(tau, c(\"difftime\", \"Date\", \"POSIXt\")))) .miniextendr_arg_error(\"tau\", message = \"`tau` must be a plain number\")",
            "if (!isTRUE(is.null(tau) || is.numeric(tau) || is.logical(tau) || is.character(tau) || is.factor(tau))) .miniextendr_arg_error(\"tau\", \"must be NULL or numeric, logical, character, or factor\")",
            "if (!isTRUE(is.null(tau) || length(tau) == 1L)) .miniextendr_arg_error(\"tau\", \"must be NULL or have length 1\")",
        ]
    );
}

/// Without a message, the refusal names the classes the way `inherits`
/// names the required ones.
#[test]
fn not_inherits_generated_message() {
    let one = explicit_output("fn f(x: SEXP)", &[("x", not_inherits(&["Date"]))], false);
    assert_eq!(
        one.guards(None),
        vec![
            "if (!isTRUE(!inherits(x, \"Date\"))) .miniextendr_arg_error(\"x\", \"must not inherit from 'Date'\")"
        ]
    );
    let several = explicit_output(
        "fn f(x: SEXP)",
        &[("x", not_inherits(&["difftime", "Date", "POSIXt"]))],
        false,
    );
    let guards = several.guards(None);
    assert_eq!(
        guards,
        vec![
            "if (!isTRUE(!inherits(x, c(\"difftime\", \"Date\", \"POSIXt\")))) .miniextendr_arg_error(\"x\", \"must not inherit from 'difftime', 'Date' or 'POSIXt'\")"
        ]
    );
    let asserts = not_inherits(&["a\"b"]).class_assertions("x", &parse_type("SEXP"));
    assert_eq!(asserts[0].message(), "'x' must not inherit from 'a\\\"b'");
}

/// `inherits` and `not_inherits` on one parameter: the required classes
/// first, then the refused ones, then the type checks, which take the
/// `inherits` message only.
#[test]
fn inherits_and_not_inherits_compose() {
    let mut checks = with_inherits_message(&["pkg_obj"], "need a pkg_obj");
    checks
        .merge(with_not_inherits_message(
            &["pkg_old"],
            "pkg_old is retired",
        ))
        .unwrap();
    let out = explicit_output("fn f(x: List)", &[("x", checks)], false);
    assert_eq!(
        out.guards(None),
        vec![
            "if (!isTRUE(inherits(x, \"pkg_obj\"))) .miniextendr_arg_error(\"x\", message = \"need a pkg_obj\")",
            "if (!isTRUE(!inherits(x, \"pkg_old\"))) .miniextendr_arg_error(\"x\", message = \"pkg_old is retired\")",
            "if (!isTRUE(is.list(x))) .miniextendr_arg_error(\"x\", message = \"need a pkg_obj\")",
        ]
    );
    // A `not_inherits` message alone leaves the type check's wording.
    let out = explicit_output(
        "fn f(x: List)",
        &[("x", with_not_inherits_message(&["pkg_old"], "retired"))],
        false,
    );
    assert_eq!(
        out.guards(None)[1],
        "if (!isTRUE(is.list(x))) .miniextendr_arg_error(\"x\", \"must be a list\")"
    );
}

/// The check is plain R on the formal, so it needs no type check: an
/// `Either` (no R guard at all, its `no_na` checked in Rust) gets it, and
/// so does a parameter under `no_preconditions`. `Missing<T>` passes an
/// omitted argument, `Option<Either<..>>` `NULL`. A bare `Either` arm
/// that reads `NULL` needs no prefix: `!inherits(NULL, cls)` is `TRUE`.
#[test]
fn not_inherits_without_type_checks() {
    let refused = || not_inherits(&["difftime"]);
    let guards = |sig: &str, no_type_checks: bool| {
        let mut checks = refused();
        checks.merge(no_na()).unwrap();
        explicit_output(sig, &[("x", checks)], no_type_checks).guards(None)
    };
    let either = guards("fn f(x: Either<Option<AsNumeric>, DataFrame>)", false);
    assert_eq!(
        either,
        vec![
            "if (!isTRUE(!inherits(x, \"difftime\"))) .miniextendr_arg_error(\"x\", \"must not inherit from 'difftime'\")"
        ]
    );
    let optional_either = guards("fn f(x: Option<Either<AsNumeric, DataFrame>>)", false);
    assert_eq!(optional_either.len(), 1);
    assert!(
        optional_either[0].starts_with("if (!isTRUE(is.null(x) || !inherits(x, \"difftime\")))"),
        "{optional_either:?}"
    );
    let unchecked = guards("fn f(x: Option<AsNumeric>)", true);
    assert_eq!(unchecked.len(), 2, "{unchecked:?}");
    assert!(unchecked[0].contains("!anyNA(x)"));
    assert!(unchecked[1].contains("!inherits(x, \"difftime\")"));
    let missing = explicit_output("fn f(x: Missing<f64>)", &[("x", refused())], false);
    assert_eq!(
        missing.guards(None)[0],
        "if (!isTRUE(missing(x) || !inherits(x, \"difftime\"))) .miniextendr_arg_error(\"x\", \"must not inherit from 'difftime'\")"
    );
}

/// Under `call = caller` the refusal names the caller's call: positional
/// after the generated requirement, by name after a message.
#[test]
fn not_inherits_takes_the_caller_call() {
    let plain = explicit_output("fn f(x: SEXP)", &[("x", not_inherits(&["Date"]))], false);
    assert!(
        plain.guards(Some(".mx_call"))[0]
            .ends_with(".miniextendr_arg_error(\"x\", \"must not inherit from 'Date'\", .mx_call)")
    );
    let custom = explicit_output(
        "fn f(x: SEXP)",
        &[("x", with_not_inherits_message(&["Date"], "no dates"))],
        false,
    );
    assert!(
        custom.guards(Some(".mx_call"))[0]
            .ends_with(".miniextendr_arg_error(\"x\", message = \"no dates\", call = .mx_call)")
    );
}

/// A class in both lists is a contradiction the parsers reject; any
/// other pairing, or one list alone, is fine. Merging keeps the two
/// lists and their messages apart.
#[test]
fn not_inherits_merge_and_conflict() {
    let mut checks = inherits(&["a", "b"]);
    checks.merge(not_inherits(&["c"])).unwrap();
    assert_eq!(checks.class_conflict("x"), None);
    checks
        .merge(with_not_inherits_message(&["d"], "m"))
        .unwrap();
    assert_eq!(checks.not_inherits, Some(vec!["c".into(), "d".into()]));
    assert_eq!(checks.not_inherits_message.as_deref(), Some("m"));
    assert!(checks.inherits_message.is_none());
    let err = checks
        .merge(with_not_inherits_message(&["e"], "n"))
        .unwrap_err();
    assert!(err.contains("`not_inherits`"), "{err}");
    checks.merge(not_inherits(&["b"])).unwrap();
    let err = checks.class_conflict("x").expect("b is in both");
    assert!(
        err.contains("class `b` is in both `inherits` and `not_inherits` on parameter `x`"),
        "{err}"
    );
    assert!(!not_inherits(&["a"]).is_empty());
    assert_eq!(not_inherits(&["a"]).class_conflict("x"), None);
}

// region: `when(...)` hints on `inherits` (#1824)

/// `inherits(classes, message, when(...) ...)`: `hints` are
/// `(classes, message)` pairs, in the order written.
fn with_hints(
    classes: &[&str],
    message: Option<&str>,
    hints: &[(&[&str], &str)],
) -> ExplicitChecks {
    ExplicitChecks {
        inherits_message: message.map(str::to_string),
        inherits_hints: hints
            .iter()
            .map(|(classes, message)| InheritsHint {
                classes: classes.iter().map(|c| c.to_string()).collect(),
                message: message.to_string(),
            })
            .collect(),
        ..inherits(classes)
    }
}

/// Each hint is one check ahead of the `inherits` check, in the order
/// written, so the first hint naming one of a refused value's classes is
/// raised. A value that passes `inherits` passes every hint
/// (`inherits(p, <required>) || ...`), so a `pkg_model` that is also a
/// `data.frame` is accepted. The type checks keep the `inherits` message.
#[test]
fn inherits_hints_precede_the_check_in_written_order() {
    let checks = with_hints(
        &["pkg_model"],
        Some("`model` must be a `pkg_model` object"),
        &[
            (&["data.frame"], "a data frame: use `col_dose()`"),
            (&["pkg_fit", "pkg_results"], "a fit: use `fit$model`"),
        ],
    );
    let out = explicit_output("fn f(model: List)", &[("model", checks)], false);
    assert_eq!(
        out.guards(None),
        vec![
            "if (!isTRUE(inherits(model, \"pkg_model\") || !inherits(model, \"data.frame\"))) .miniextendr_arg_error(\"model\", message = \"a data frame: use `col_dose()`\")",
            "if (!isTRUE(inherits(model, \"pkg_model\") || !inherits(model, c(\"pkg_fit\", \"pkg_results\")))) .miniextendr_arg_error(\"model\", message = \"a fit: use `fit$model`\")",
            "if (!isTRUE(inherits(model, \"pkg_model\"))) .miniextendr_arg_error(\"model\", message = \"`model` must be a `pkg_model` object\")",
            "if (!isTRUE(is.list(model))) .miniextendr_arg_error(\"model\", message = \"`model` must be a `pkg_model` object\")",
        ]
    );
    // Without its own message the check keeps the generated one; several
    // required classes are all accepted by every hint.
    let checks = with_hints(&["pkg_a", "pkg_b"], None, &[(&["data.frame"], "hint")]);
    let out = explicit_output("fn f(x: SEXP)", &[("x", checks)], false);
    assert_eq!(
        out.guards(None),
        vec![
            "if (!isTRUE(inherits(x, c(\"pkg_a\", \"pkg_b\")) || !inherits(x, \"data.frame\"))) .miniextendr_arg_error(\"x\", message = \"hint\")",
            "if (!isTRUE(inherits(x, c(\"pkg_a\", \"pkg_b\")))) .miniextendr_arg_error(\"x\", \"must inherit from 'pkg_a' or 'pkg_b'\")",
        ]
    );
}

/// A hint is an R string literal like any message: quotes, backslashes,
/// control characters and non-ASCII text are escaped, and so are its
/// class names.
#[test]
fn inherits_hints_are_escaped() {
    let checks = with_hints(
        &["pkg_model"],
        None,
        &[(
            &["caf\u{e9}", "a\"b\\c"],
            "say \"hi\" \\ back\nline caf\u{e9} \u{1F600}",
        )],
    );
    let out = explicit_output("fn f(x: SEXP)", &[("x", checks)], false);
    assert_eq!(
        out.guards(None)[0],
        r#"if (!isTRUE(inherits(x, "pkg_model") || !inherits(x, c("caf\u{e9}", "a\"b\\c")))) .miniextendr_arg_error("x", message = "say \"hi\" \\ back\nline caf\u{e9} \U{1f600}")"#
    );
}

/// Under `call = caller` a hint names the caller's call, by name after
/// its message.
#[test]
fn inherits_hints_take_the_caller_call() {
    let checks = with_hints(&["pkg_model"], None, &[(&["data.frame"], "hint")]);
    let out = explicit_output("fn f(x: SEXP)", &[("x", checks)], false);
    assert_eq!(
        out.guards(Some(".mx_call")),
        vec![
            "if (!isTRUE(inherits(x, \"pkg_model\") || !inherits(x, \"data.frame\"))) .miniextendr_arg_error(\"x\", message = \"hint\", call = .mx_call)",
            "if (!isTRUE(inherits(x, \"pkg_model\"))) .miniextendr_arg_error(\"x\", \"must inherit from 'pkg_model'\", .mx_call)",
        ]
    );
}

/// An `Option<T>` parameter passes `NULL`, and a `Missing<T>` parameter
/// an omitted argument, through the hints as through the check. Under
/// `no_preconditions` the hints stay with the check.
#[test]
fn inherits_hints_pass_null_and_missing() {
    let hinted = || with_hints(&["pkg_model"], None, &[(&["data.frame"], "hint")]);
    let optional = explicit_output("fn f(x: Option<List>)", &[("x", hinted())], true);
    assert_eq!(
        optional.guards(None)[0],
        "if (!isTRUE(is.null(x) || inherits(x, \"pkg_model\") || !inherits(x, \"data.frame\"))) .miniextendr_arg_error(\"x\", message = \"hint\")"
    );
    let missing = explicit_output("fn f(x: Missing<List>)", &[("x", hinted())], true);
    assert_eq!(
        missing.guards(None),
        vec![
            "if (!isTRUE(missing(x) || inherits(x, \"pkg_model\") || !inherits(x, \"data.frame\"))) .miniextendr_arg_error(\"x\", message = \"hint\")",
            "if (!isTRUE(missing(x) || inherits(x, \"pkg_model\"))) .miniextendr_arg_error(\"x\", \"must inherit from 'pkg_model'\")",
        ]
    );
}

/// Several `inherits` attributes on one parameter concatenate their
/// hints in the order written; every hint then accepts every required
/// class.
#[test]
fn inherits_hints_merge_in_order() {
    let mut checks = with_hints(&["pkg_a"], Some("need an a"), &[(&["df"], "one")]);
    checks
        .merge(with_hints(&["pkg_b"], None, &[(&["tbl"], "two")]))
        .unwrap();
    checks.merge(inherits(&["pkg_c"])).unwrap();
    let hints: Vec<&str> = checks
        .inherits_hints
        .iter()
        .map(|h| h.message.as_str())
        .collect();
    assert_eq!(hints, ["one", "two"]);
    assert_eq!(checks.class_conflict("x"), None);
    let out = explicit_output("fn f(x: SEXP)", &[("x", checks)], false);
    let guards = out.guards(None);
    assert_eq!(guards.len(), 3);
    assert!(
        guards[0].starts_with(
            "if (!isTRUE(inherits(x, c(\"pkg_a\", \"pkg_b\", \"pkg_c\")) || !inherits(x, \"df\")))"
        ),
        "{}",
        guards[0]
    );
    assert!(guards[1].contains("!inherits(x, \"tbl\"))) "));
    assert!(guards[1].ends_with("message = \"two\")"));
    assert!(guards[2].ends_with("message = \"need an a\")"));
}

/// A hint class that `inherits` requires could never be shown: an error,
/// also when another attribute requires it. A hint class that
/// `not_inherits` refuses is fine: a value of that class alone gets the
/// hint, one that also has a required class gets the refusal.
#[test]
fn inherits_hint_class_conflicts() {
    let checks = with_hints(&["pkg_model"], None, &[(&["df", "pkg_model"], "hint")]);
    let err = checks
        .class_conflict("model")
        .expect("pkg_model is required");
    assert!(
        err.contains(
            "class `pkg_model` is in both `inherits` and a `when(...)` hint on parameter `model`"
        ),
        "{err}"
    );
    let mut checks = with_hints(&["pkg_model"], None, &[(&["pkg_old"], "hint")]);
    assert_eq!(checks.class_conflict("model"), None);
    checks.merge(inherits(&["pkg_old"])).unwrap();
    assert!(checks.class_conflict("model").is_some());

    let mut checks = with_hints(&["pkg_model"], None, &[(&["pkg_old"], "hint")]);
    checks.merge(not_inherits(&["pkg_old"])).unwrap();
    assert_eq!(checks.class_conflict("model"), None);
    let out = explicit_output("fn f(x: SEXP)", &[("x", checks)], false);
    assert_eq!(
        out.guards(None),
        vec![
            "if (!isTRUE(inherits(x, \"pkg_model\") || !inherits(x, \"pkg_old\"))) .miniextendr_arg_error(\"x\", message = \"hint\")",
            "if (!isTRUE(inherits(x, \"pkg_model\"))) .miniextendr_arg_error(\"x\", \"must inherit from 'pkg_model'\")",
            "if (!isTRUE(!inherits(x, \"pkg_old\"))) .miniextendr_arg_error(\"x\", \"must not inherit from 'pkg_old'\")",
        ]
    );
}

// endregion

#[test]
fn no_na_wording_follows_the_shape() {
    let scalar = explicit_output("fn f(x: f64)", &[("x", no_na())], true);
    assert_eq!(
        scalar.guards(None),
        vec!["if (!isTRUE(!anyNA(x))) .miniextendr_arg_error(\"x\", \"must not be NA\")"]
    );
    let vector = explicit_output("fn f(x: Vec<f64>)", &[("x", no_na())], true);
    assert_eq!(
        vector.guards(None),
        vec!["if (!isTRUE(!anyNA(x))) .miniextendr_arg_error(\"x\", \"must not contain NA\")"]
    );
    let slice = explicit_output("fn f(x: &[f64])", &[("x", no_na())], true);
    assert!(slice.guards(None)[0].contains("must not contain NA"));
    // The vector markers are vectors too; their scalar forms are not.
    for (ty, verb) in [
        ("AsNumericVec", "contain"),
        ("AsCharacterVec", "contain"),
        ("Option<AsCharacterVec>", "contain"),
        ("AsNumeric", "be"),
        ("AsCharacter", "be"),
    ] {
        let out = explicit_output(&format!("fn f(x: {ty})"), &[("x", no_na())], true);
        assert!(
            out.guards(None)[0].contains(&format!("must not {verb} NA")),
            "{ty}: {:?}",
            out.guards(None)
        );
    }
}

/// Beyond the type table's vector checks (above): a `Missing<_>` marker,
/// maps, arrays, and a marker the table does not know, by its `...Vec`
/// name (`AsFromStrVec<T>`, `AsDisplayVec<T>`), hold several values too;
/// their scalar halves and opaque types do not.
#[test]
fn no_na_wording_knows_the_vector_markers() {
    let verb = |sig: &str| {
        let out = explicit_output(sig, &[("x", no_na())], false);
        let guards = out.guards(None);
        let guard = guards.first().expect("the no_na guard");
        if guard.contains("must not contain NA") {
            "contain"
        } else {
            assert!(guard.contains("must not be NA"), "{guard}");
            "be"
        }
    };
    for (sig, expected) in [
        ("fn f(x: Option<AsNumericVec>)", "contain"),
        ("fn f(x: Missing<AsNumericVec>)", "contain"),
        ("fn f(x: AsFromStrVec<std::net::IpAddr>)", "contain"),
        ("fn f(x: AsDisplayVec<u8>)", "contain"),
        ("fn f(x: HashMap<String, f64>)", "contain"),
        ("fn f(x: [f64; 3])", "contain"),
        // An optional data frame is worded as a bare one.
        ("fn f(x: Option<DataFrame>)", "be"),
        ("fn f(x: Option<AsNumeric>)", "be"),
        ("fn f(x: AsFromStr<std::net::IpAddr>)", "be"),
        ("fn f(x: String)", "be"),
        ("fn f(x: MyCustomType)", "be"),
        // Through `Result<T, _>` and `Either`, the layers `no_na` checks
        // through: an `Either` holds several when either arm does.
        ("fn f(x: Result<AsNumericVec, ()>)", "contain"),
        ("fn f(x: Result<AsNumeric, ()>)", "be"),
        ("fn f(x: Either<AsNumericVec, Vec<u8>>)", "contain"),
        ("fn f(x: Either<AsNumericVec, i32>)", "contain"),
        ("fn f(x: Either<i32, AsCharacterVec>)", "contain"),
        ("fn f(x: Either<AsNumeric, AsCharacter>)", "be"),
        // An `Option` arm is read by its value.
        ("fn f(x: Either<Option<AsNumeric>, DataFrame>)", "be"),
        (
            "fn f(x: Either<Option<AsNumericVec>, DataFrame>)",
            "contain",
        ),
        (
            "fn f(x: Option<Either<i32, Either<String, Vec<f64>>>>)",
            "contain",
        ),
    ] {
        let ty = match syn::parse_str::<syn::Signature>(sig).unwrap().inputs[0].clone() {
            syn::FnArg::Typed(pt) => *pt.ty,
            syn::FnArg::Receiver(_) => unreachable!(),
        };
        // An `Either` has only the Rust check; the others have the R guard
        // too, and the Rust check words it the same way.
        if !no_na_checked_after_conversion(&ty) {
            assert_eq!(verb(sig), expected, "{sig}");
        }
        assert_eq!(
            no_na_requirement(&ty),
            format!("must not {expected} NA"),
            "{sig}"
        );
    }
}

/// `no_na` on an `Either` (also under `Missing` or `Option`) emits no R
/// guard: the C wrapper checks the input by the arm the value converted
/// to. A plain parameter keeps the guard (`Option<DataFrame>` behind its
/// `is.null(x)` pass), and so does a choice `Either` (`match_arg` /
/// `choices`, also under `Option` / `Missing<Option<..>>`), whose
/// conversion has no Rust check.
#[test]
fn no_na_on_an_either_is_checked_after_the_conversion() {
    let guards = |sig: &str, skip: &[&str]| {
        let sig: syn::Signature = syn::parse_str(sig).unwrap();
        let opts = PreconditionOptions {
            explicit: [("x".to_string(), no_na())].into_iter().collect(),
            ..Default::default()
        };
        let skip: HashSet<String> = skip.iter().map(|s| s.to_string()).collect();
        build_precondition_checks(&sig.inputs, &skip, &opts).guards(None)
    };
    for sig in [
        "fn f(x: Either<AsNumeric, DataFrame>)",
        "fn f(x: Either<Option<AsNumeric>, DataFrame>)",
        "fn f(x: either::Either<Vec<f64>, List>)",
        "fn f(x: Missing<Either<AsNumeric, DataFrame>>)",
        "fn f(x: Option<Either<AsNumericVec, DataFrame>>)",
        "fn f(x: Missing<Option<Either<f64, String>>>)",
    ] {
        assert!(
            guards(sig, &[]).iter().all(|g| !g.contains("anyNA")),
            "{sig}: {:?}",
            guards(sig, &[])
        );
    }
    for (sig, skip) in [
        ("fn f(x: AsNumeric)", &[][..]),
        ("fn f(x: DataFrame)", &[]),
        ("fn f(x: Either<Mode, DataFrame>)", &["x"]),
    ] {
        let guards = guards(sig, skip);
        assert!(
            guards
                .first()
                .is_some_and(|g| g.contains("!isTRUE(!anyNA(x))")),
            "{sig}: {guards:?}"
        );
    }
    // A choice `Either` under `Option` / `Missing<Option<..>>` keeps the
    // guard too, after the layer's `is.null(x)` / `missing(x)` pass, as
    // does an optional data frame.
    for (sig, layer, skip) in [
        (
            "fn f(x: Option<Either<Mode, DataFrame>>)",
            "is.null(x)",
            &["x"][..],
        ),
        (
            "fn f(x: Missing<Option<Either<Mode, List>>>)",
            "missing(x)",
            &["x"],
        ),
        ("fn f(x: Option<DataFrame>)", "is.null(x)", &[]),
    ] {
        let guards = guards(sig, skip);
        let expected = format!("!isTRUE({layer} || !anyNA(x))");
        assert!(
            guards.first().is_some_and(|g| g.contains(&expected)),
            "{sig}: {guards:?}"
        );
    }
    // `Result<Either<..>, ()>` is not a layer the predicate peels: it
    // keeps the guard, and the plain Rust check.
    assert!(!no_na_checked_after_conversion(
        &syn::parse_str("Result<Either<f64, String>, ()>").unwrap()
    ));
}

/// The message the Rust `no_na` check raises: generated from the
/// parameter's R name and type, or the author's message verbatim.
#[test]
fn no_na_message_is_generated_or_custom() {
    let ty = |s: &str| syn::parse_str::<syn::Type>(s).unwrap();
    assert_eq!(
        no_na_message("x", &ty("AsNumeric"), None),
        "'x' must not be NA"
    );
    assert_eq!(
        no_na_message("dv", &ty("Missing<Option<AsNumericVec>>"), None),
        "'dv' must not contain NA"
    );
    // A vector alias not named `...Vec` reads `be`, as its R guard does.
    assert_eq!(no_na_message("d", &ty("Doses"), None), "'d' must not be NA");
    let custom = "`x` can't be NA: it's \"required\" \\ 100%\ncaf\u{e9}()";
    assert_eq!(no_na_message("x", &ty("AsNumeric"), Some(custom)), custom);
}

#[test]
fn explicit_checks_pass_null_and_missing() {
    let optional = explicit_output(
        "fn f(x: Option<List>)",
        &[("x", inherits(&["pkg_obj"]))],
        true,
    );
    assert_eq!(
        optional.guards(None),
        vec![
            "if (!isTRUE(is.null(x) || inherits(x, \"pkg_obj\"))) .miniextendr_arg_error(\"x\", \"must inherit from 'pkg_obj'\")"
        ]
    );
    let missing = explicit_output("fn f(x: Missing<f64>)", &[("x", no_na())], true);
    assert_eq!(
        missing.guards(None),
        vec![
            "if (!isTRUE(missing(x) || !anyNA(x))) .miniextendr_arg_error(\"x\", \"must not be NA\")"
        ]
    );
    let optional_vec = explicit_output("fn f(x: Option<Vec<f64>>)", &[("x", no_na())], true);
    assert!(optional_vec.guards(None)[0].contains("must not contain NA"));
}

#[test]
fn explicit_checks_survive_no_type_checks() {
    let both = ExplicitChecks {
        inherits: Some(vec!["pkg_obj".into()]),
        no_na: true,
        ..Default::default()
    };
    let out = explicit_output("fn f(n: i32, x: List)", &[("x", both)], true);
    // `n`'s type checks and `x`'s `is.list()` are gone; the named checks stay,
    // the NA check first. A list holds several values, so its NA check says
    // `contain`.
    assert_eq!(
        out.guards(None),
        vec![
            "if (!isTRUE(!anyNA(x))) .miniextendr_arg_error(\"x\", \"must not contain NA\")",
            "if (!isTRUE(inherits(x, \"pkg_obj\"))) .miniextendr_arg_error(\"x\", \"must inherit from 'pkg_obj'\")",
        ]
    );
    let none = explicit_output("fn f(n: i32)", &[], true);
    assert!(none.guards(None).is_empty());
}

#[test]
fn explicit_checks_apply_to_match_arg_params() {
    let sig: syn::Signature = syn::parse_str("fn f(mode: String)").unwrap();
    let opts = PreconditionOptions {
        explicit: [("mode".to_string(), no_na())].into_iter().collect(),
        ..Default::default()
    };
    let skip: HashSet<String> = ["mode".to_string()].into_iter().collect();
    let out = build_precondition_checks(&sig.inputs, &skip, &opts);
    assert_eq!(
        out.guards(None),
        vec!["if (!isTRUE(!anyNA(mode))) .miniextendr_arg_error(\"mode\", \"must not be NA\")"]
    );
}

#[test]
fn explicit_checks_take_the_caller_call() {
    let out = explicit_output("fn f(x: f64)", &[("x", no_na())], true);
    assert_eq!(
        out.guards(Some(".mx_call")),
        vec!["if (!isTRUE(!anyNA(x))) .miniextendr_arg_error(\"x\", \"must not be NA\", .mx_call)"]
    );
}

#[test]
fn explicit_checks_merge() {
    let mut a = inherits(&["a"]);
    a.merge(no_na()).unwrap();
    a.merge(inherits(&["b"])).unwrap();
    assert_eq!(
        a,
        ExplicitChecks {
            inherits: Some(vec!["a".into(), "b".into()]),
            no_na: true,
            ..Default::default()
        }
    );
    assert!(ExplicitChecks::default().is_empty());

    // One message per check: it may come from either attribute, not both.
    let mut b = inherits(&["a"]);
    b.merge(with_inherits_message(&["b"], "one")).unwrap();
    assert_eq!(b.inherits_message.as_deref(), Some("one"));
    assert_eq!(b.inherits, Some(vec!["a".into(), "b".into()]));
    b.merge(with_no_na_message("two")).unwrap();
    assert_eq!(b.no_na_message.as_deref(), Some("two"));
    let err = b.merge(with_inherits_message(&["c"], "three")).unwrap_err();
    assert!(
        err.contains("`inherits`") && err.contains("more than one `message`"),
        "{err}"
    );
    let err = b.merge(with_no_na_message("four")).unwrap_err();
    assert!(err.contains("`no_na`"), "{err}");
}

/// The author's message replaces `'<p>' <requirement>` verbatim: the
/// helper gets it as `message = `, and the call, when there is one, by
/// name too (after a named `message` a positional call would bind to
/// `what`). One message covers every class of the check, and the
/// parameter's type checks.
#[test]
fn custom_messages_are_passed_verbatim_by_name() {
    let checks = ExplicitChecks {
        inherits: Some(vec!["pkg_a".into(), "pkg_b".into()]),
        inherits_message: Some("`model` must be a `pkg_model`; see pkg_model().".into()),
        no_na: true,
        no_na_message: Some("no NA in `model`".into()),
        ..Default::default()
    };
    let out = explicit_output("fn f(model: List)", &[("model", checks)], false);
    assert_eq!(
        out.guards(None),
        vec![
            "if (!isTRUE(!anyNA(model))) .miniextendr_arg_error(\"model\", message = \"no NA in `model`\")",
            "if (!isTRUE(inherits(model, c(\"pkg_a\", \"pkg_b\")))) .miniextendr_arg_error(\"model\", message = \"`model` must be a `pkg_model`; see pkg_model().\")",
            "if (!isTRUE(is.list(model))) .miniextendr_arg_error(\"model\", message = \"`model` must be a `pkg_model`; see pkg_model().\")",
        ]
    );
    let caller = out.guards(Some(".mx_call"));
    assert!(
        caller[0].ends_with(
            ".miniextendr_arg_error(\"model\", message = \"no NA in `model`\", call = .mx_call)"
        ),
        "{}",
        caller[0]
    );
    assert!(
        caller[1].ends_with(
            ".miniextendr_arg_error(\"model\", message = \"`model` must be a `pkg_model`; see pkg_model().\", call = .mx_call)"
        ),
        "{}",
        caller[1]
    );
    // A type check with the class message names the call too.
    assert!(
        caller[2].ends_with(
            ".miniextendr_arg_error(\"model\", message = \"`model` must be a `pkg_model`; see pkg_model().\", call = .mx_call)"
        ),
        "{}",
        caller[2]
    );
    // A type check without a class message keeps the positional call.
    let plain = explicit_output("fn f(model: List)", &[("model", no_na())], false);
    assert!(
        plain.guards(Some(".mx_call"))[1]
            .ends_with(".miniextendr_arg_error(\"model\", \"must be a list\", .mx_call)"),
        "{:?}",
        plain.guards(Some(".mx_call"))
    );
    // The `Option` / `Missing` guards are unchanged.
    let optional = explicit_output(
        "fn f(x: Option<List>)",
        &[("x", with_inherits_message(&["pkg_obj"], "need a pkg_obj"))],
        true,
    );
    assert_eq!(
        optional.guards(None),
        vec![
            "if (!isTRUE(is.null(x) || inherits(x, \"pkg_obj\"))) .miniextendr_arg_error(\"x\", message = \"need a pkg_obj\")"
        ]
    );
    let asserts = with_no_na_message("mine").value_assertions("x", &parse_type("f64"), false);
    assert_eq!(asserts[0].message(), "mine");
}

/// A message is an R string literal: quotes, backslashes, newlines, tabs
/// and control characters are escaped, and non-ASCII text becomes
/// `\u{..}` / `\U{..}` escapes (R code in a package must be ASCII). `%`
/// needs nothing: the message never goes through `sprintf()`.
#[test]
fn custom_messages_are_escaped_as_r_string_literals() {
    let message = "say \"hi\" \\ back\nline\ttab 100% caf\u{e9} \u{1F600} bell\u{7}";
    let out = explicit_output("fn f(x: f64)", &[("x", with_no_na_message(message))], true);
    assert_eq!(
        out.guards(None),
        vec![
            r#"if (!isTRUE(!anyNA(x))) .miniextendr_arg_error("x", message = "say \"hi\" \\ back\nline\ttab 100% caf\u{e9} \U{1f600} bell\u{7}")"#
        ]
    );
    assert_eq!(r_string_escape("a\rb"), r"a\rb");
    assert_eq!(r_string_escape("plain 'text'"), "plain 'text'");
    // Class names take the same escaping.
    let out = explicit_output("fn f(x: SEXP)", &[("x", inherits(&["caf\u{e9}"]))], true);
    assert_eq!(
        out.guards(None),
        vec![
            r#"if (!isTRUE(inherits(x, "caf\u{e9}"))) .miniextendr_arg_error("x", "must inherit from 'caf\u{e9}'")"#
        ]
    );
}
