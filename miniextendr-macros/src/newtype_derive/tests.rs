/// `#[try_from_sexp(validate = path)]` gives the element marker a
/// `check_sexp` that calls the path, and the scalar impl runs it before
/// the inner conversion, with `SexpError` as its error. Without the
/// attribute the scalar impl keeps the inner error and no check is
/// emitted (the trait's default accepts every value).
#[test]
fn try_from_sexp_derive_validate_emits_the_check() {
    let validated = super::derive_try_from_sexp(syn::parse_quote! {
        #[try_from_sexp(validate = checks::plain_number)]
        struct Duration(f64);
    })
    .unwrap()
    .to_string();
    assert!(
        validated.contains(
            "fn check_sexp (sexp : :: miniextendr_api :: SEXP ,) -> :: core :: result :: Result < () , :: miniextendr_api :: from_r :: SexpError > { checks :: plain_number (sexp) . map_err (:: core :: convert :: Into :: into) }"
        ),
        "{validated}"
    );
    assert!(
        validated.contains("type Error = :: miniextendr_api :: from_r :: SexpError ;"),
        "{validated}"
    );
    assert!(
        validated.contains(
            "< f64 as :: miniextendr_api :: TryFromSexp > :: Error : :: core :: convert :: Into < :: miniextendr_api :: from_r :: SexpError >"
        ),
        "{validated}"
    );
    // Both scalar paths run the check first.
    let check = "< Self as :: miniextendr_api :: TryFromSexpElement > :: check_sexp (sexp) ? ;";
    assert_eq!(validated.matches(check).count(), 2, "{validated}");

    let plain = super::derive_try_from_sexp(syn::parse_quote! { struct Duration(f64); })
        .unwrap()
        .to_string();
    assert!(!plain.contains("check_sexp"), "{plain}");
    assert!(
        plain.contains("type Error = < f64 as :: miniextendr_api :: TryFromSexp > :: Error ;"),
        "{plain}"
    );
}

/// `validate` is the only key, given once, and only on the struct.
#[test]
fn try_from_sexp_derive_rejects_bad_options() {
    let error = |input: syn::DeriveInput| {
        super::derive_try_from_sexp(input)
            .expect_err("the options should be refused")
            .to_string()
    };
    assert!(
        error(syn::parse_quote! {
            #[try_from_sexp(check = plain_number)]
            struct Duration(f64);
        })
        .contains("unknown `#[try_from_sexp(...)]` key")
    );
    assert!(
        error(syn::parse_quote! {
            #[try_from_sexp(validate = a, validate = b)]
            struct Duration(f64);
        })
        .contains("duplicate `validate`")
    );
    assert!(
        error(syn::parse_quote! {
            struct Duration(#[try_from_sexp(validate = a)] f64);
        })
        .contains("goes on the struct")
    );
}

/// `#[derive(TryFromSexp)]` forwards the hidden `no_na` probes to the
/// inner type, for tuple and named newtypes alike: `__mx_has_na` for the
/// value, and `__mx_input_has_na` for the input of an `Either` arm.
#[test]
fn try_from_sexp_derive_forwards_no_na() {
    for (input, field) in [
        (syn::parse_quote! { struct Table(DataFrame); }, "self . 0"),
        (
            syn::parse_quote! { struct Table { value: DataFrame } },
            "self . value",
        ),
    ] {
        let out = super::derive_try_from_sexp(input).unwrap().to_string();
        let forward =
            |call: &str| format!("< DataFrame as :: miniextendr_api :: TryFromSexp > :: {call}");
        assert!(out.contains("fn __mx_has_na (& self) -> bool"), "{out}");
        assert!(
            out.contains(&forward(&format!("__mx_has_na (& {field})"))),
            "{out}"
        );
        assert!(
            out.contains(
                "fn __mx_input_has_na (& self , input : :: miniextendr_api :: SEXP) -> bool"
            ),
            "{out}"
        );
        assert!(
            out.contains(&forward(&format!("__mx_input_has_na (& {field} , input)"))),
            "{out}"
        );
    }
}

/// `#[derive(TryFromSexp)]` declares the inner type's R-facing
/// expectation: the macro's wording for an inner type it knows, else the
/// inner type's own constant (a newtype of a newtype, a generic newtype).
#[test]
fn try_from_sexp_derive_declares_the_inner_expectation() {
    let expectation = |input: syn::DeriveInput| {
        let out = super::derive_try_from_sexp(input).unwrap().to_string();
        let start = out
            .find("const __MX_EXPECTATION")
            .unwrap_or_else(|| panic!("no expectation constant: {out}"));
        let end = start + out[start..].find(';').unwrap();
        out[start..end].trim_end().to_string()
    };
    let declares = |text: &str| {
        format!(
            "const __MX_EXPECTATION : :: core :: option :: Option < & 'static str > = \
                 :: core :: option :: Option :: Some ({text:?})"
        )
    };
    assert_eq!(
        expectation(syn::parse_quote! { struct Dose(AsNumeric); }),
        declares("a single number")
    );
    assert_eq!(
        expectation(syn::parse_quote! { struct Numbers { values: AsNumericVec } }),
        declares("numeric")
    );
    assert_eq!(
        expectation(syn::parse_quote! { struct Table(DataFrame); }),
        declares("a data frame")
    );
    for (input, inner) in [
        (syn::parse_quote! { struct Wrapped(Table); }, "Table"),
        (syn::parse_quote! { struct Id(Uuid); }, "Uuid"),
        (syn::parse_quote! { struct Any<T>(T); }, "T"),
    ] {
        assert_eq!(
            expectation(input),
            format!(
                "const __MX_EXPECTATION : :: core :: option :: Option < & 'static str > = \
                     < {inner} as :: miniextendr_api :: TryFromSexp > :: __MX_EXPECTATION"
            )
        );
    }
}
