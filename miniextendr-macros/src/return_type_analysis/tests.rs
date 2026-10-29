use super::strict_conversion_for_type;

/// The strict conversion emitted for return type `ty`, as a token string.
fn strict_expr(ty: &str) -> Option<String> {
    let ty: syn::Type = syn::parse_str(ty).expect("valid type");
    let result = syn::Ident::new("result", proc_macro2::Span::call_site());
    strict_conversion_for_type(&ty, &result).map(|ts| ts.to_string())
}

fn calls(ty: &str, helper: &str) -> bool {
    strict_expr(ty).is_some_and(|expr| expr.contains(helper))
}

#[test]
fn list_of_vectors_shapes_use_checked_row_helpers() {
    assert!(calls("Vec<Vec<usize>>", "checked_vec_vec_usize_into_sexp"));
    assert!(calls("Vec<Vec<i64>>", "checked_vec_vec_i64_into_sexp"));
    assert!(calls(
        "Vec<Option<Vec<u64>>>",
        "checked_vec_option_vec_u64_into_sexp"
    ));
    assert!(calls(
        "std::vec::Vec<std::vec::Vec<isize>>",
        "checked_vec_vec_isize_into_sexp"
    ));
}

#[test]
fn option_vec_lossy_checks_some_and_returns_null_for_none() {
    let expr = strict_expr("Option<Vec<usize>>").expect("strict arm");
    assert!(expr.contains("checked_vec_usize_into_sexp"), "{expr}");
    assert!(expr.contains("SEXP :: nil ()"), "{expr}");
}

#[test]
fn flat_lossy_shapes_keep_their_helpers() {
    assert!(calls("usize", "checked_into_sexp_usize"));
    assert!(calls("Option<i64>", "checked_option_i64_into_sexp"));
    assert!(calls("Vec<u64>", "checked_vec_u64_into_sexp"));
    assert!(calls(
        "Vec<Option<isize>>",
        "checked_vec_option_isize_into_sexp"
    ));
}

#[test]
fn slice_box_and_array_shapes_use_shared_checked_helpers() {
    for scalar in ["i64", "u64", "isize", "usize"] {
        for shape in [
            format!("&[{scalar}]"),
            format!("Box<[{scalar}]>"),
            format!("[{scalar}; 3]"),
        ] {
            assert!(calls(&shape, "checked_slice_into_sexp"), "{shape}");
            assert!(calls(
                &format!("Option<{shape}>"),
                "checked_slice_into_sexp"
            ));
            assert!(calls(&format!("Vec<{shape}>"), "checked_rows_into_sexp"));
            assert!(calls(
                &format!("Vec<Option<{shape}>>"),
                "checked_rows_into_sexp"
            ));
        }
    }
    assert!(calls(
        "std::vec::Vec<std::boxed::Box<[usize]>>",
        "checked_rows_into_sexp"
    ));
    assert!(calls("&'static [u64]", "checked_slice_into_sexp"));
    assert!(calls("&mut [i64]", "checked_slice_into_sexp"));
}

#[test]
fn slice_box_and_array_errors_name_the_type_as_written() {
    for (ty, name) in [
        ("Box<[i64]>", "Box<[i64]>"),
        ("&'static [u64]", "&'static [u64]"),
        ("Option<[usize; 2]>", "[usize; 2]"),
        ("Vec<Option<Box<[isize]>>>", "Vec<Option<Box<[isize]>>>"),
    ] {
        let expr = strict_expr(ty).expect("a strict conversion");
        assert!(expr.contains(&format!("{name:?}")), "{ty}: {expr}");
    }
}

#[test]
fn lossless_shapes_fall_through_to_into_r() {
    for ty in [
        "Vec<Vec<i32>>",
        "Vec<Vec<f64>>",
        "Option<Vec<String>>",
        "Vec<Option<Vec<bool>>>",
        "Vec<Vec<Vec<usize>>>",
    ] {
        assert_eq!(strict_expr(ty), None, "{ty}");
    }
}
