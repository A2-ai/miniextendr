use super::*;

fn policy(value: syn::Expr) -> DotsPolicy {
    match parse_dots_attr(&value) {
        Ok(DotsAttr::Policy(policy)) => policy,
        Ok(DotsAttr::TypedList(_)) => panic!("parsed as typed_list!"),
        Err(err) => panic!("{err}"),
    }
}

fn error(value: syn::Expr) -> String {
    match parse_dots_attr(&value) {
        Err(err) => err
            .into_iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join(" | "),
        Ok(_) => panic!("parsed"),
    }
}

#[test]
fn spellings() {
    let p = policy(syn::parse_quote!(empty));
    assert_eq!(
        (p.role, p.allow.len(), p.class.len()),
        (DotsRole::Empty, 0, 0)
    );
    let p = policy(syn::parse_quote!(empty()));
    assert_eq!((p.role, p.allow.len()), (DotsRole::Empty, 0));
    let p = policy(syn::parse_quote!(empty(allow = ["row.names", "optional"])));
    assert_eq!(p.allow, ["row.names", "optional"]);
    let p = policy(syn::parse_quote!(forward(
        allow = "digits",
        class = "pkg_unused"
    )));
    assert_eq!(
        (p.role, p.allow, p.class),
        (
            DotsRole::Forward,
            vec!["digits".into()],
            vec!["pkg_unused".into()]
        )
    );
    let p = policy(syn::parse_quote!(data));
    assert_eq!(p.role, DotsRole::Data);
    assert!(matches!(
        parse_dots_attr(&syn::parse_quote!(typed_list!(x => numeric()))),
        Ok(DotsAttr::TypedList(_))
    ));
}

#[test]
fn refusals_are_batched() {
    let msg = error(syn::parse_quote!(empty(
        allow = ["a", "", "a", 1, "..."],
        class = []
    )));
    assert!(msg.contains("the string is empty"), "{msg}");
    assert!(msg.contains("\"a\" is listed twice"), "{msg}");
    assert!(msg.contains("expected a string literal"), "{msg}");
    assert!(msg.contains("`...` is not an argument name"), "{msg}");
    assert!(msg.contains("the list is empty"), "{msg}");
    assert!(error(syn::parse_quote!(forward)).contains("`forward` passes the names it lists"));
    assert!(error(syn::parse_quote!(forward())).contains("`forward` passes the names it lists"));
    assert!(error(syn::parse_quote!(data(allow = "x"))).contains("`data` takes no options"));
    assert!(error(syn::parse_quote!(empty(allow = "a", allow = "b"))).contains("given twice"));
    assert!(error(syn::parse_quote!(empty(keep = "a"))).contains("unknown option"));
    assert!(error(syn::parse_quote!(nothing)).contains("dots expects"));
    assert!(error(syn::parse_quote!(other_list!(x))).contains("dots expects"));
}

#[test]
fn r_check_reads_names_only() {
    let p = policy(syn::parse_quote!(empty));
    assert_eq!(
        p.r_check("object, parm, level = 0.95, ...", None).unwrap(),
        "if (...length()) .miniextendr_check_dots(...names(), ...length(), \
             c(\"object\", \"parm\", \"level\"))"
    );
    let p = policy(syn::parse_quote!(empty(
        allow = ["optional"],
        class = ["a\"b"]
    )));
    assert_eq!(
        p.r_check(".mx_self, x, ..., .call = NULL", Some("environment()"))
            .unwrap(),
        "if (...length()) .miniextendr_check_dots(...names(), ...length(), \"x\", \
             allow = \"optional\", class = \"a\\\"b\", call = environment())"
    );
    // No `...` formal: R refuses extra arguments itself.
    assert!(p.r_check("x, y", None).is_none());
    // `forward` lets unnamed arguments through.
    assert_eq!(
        policy(syn::parse_quote!(forward(allow = ["digits", "nsmall"])))
            .r_check("x, ...", None)
            .unwrap(),
        "if (...length()) .miniextendr_check_dots(...names(), ...length(), \"x\", \
             allow = c(\"digits\", \"nsmall\"), forward = TRUE)"
    );
    // `data` checks nothing.
    assert!(
        policy(syn::parse_quote!(data))
            .r_check("x, ...", None)
            .is_none()
    );
    // A default holding a comma doesn't split a formal.
    assert_eq!(
        policy(syn::parse_quote!(empty))
            .r_check("x, mode = c(\"a\", \"b\"), ...", None)
            .unwrap(),
        "if (...length()) .miniextendr_check_dots(...names(), ...length(), c(\"x\", \"mode\"))"
    );
}

#[test]
fn r_check_takes_classes_as_an_r_expression() {
    // The field methods pass the type's `field_error` classes, a variable
    // of the derive's helpers, unquoted (#2174).
    let p = DotsPolicy::empty(Some(".rdata_error_class_T".to_string()));
    assert_eq!(
        p.r_check("x, i, ..., exact = TRUE", Some("environment()"))
            .unwrap(),
        "if (...length()) .miniextendr_check_dots(...names(), ...length(), \
             c(\"x\", \"i\", \"exact\"), class = .rdata_error_class_T, call = environment())"
    );
    assert_eq!(
        DotsPolicy::empty(None).r_check("x, ...", None).unwrap(),
        "if (...length()) .miniextendr_check_dots(...names(), ...length(), \"x\")"
    );
}
