use super::*;

#[test]
fn wrapper_idents_match_between_attribute_and_module_macros() {
    let item_fn: syn::ItemFn = syn::parse2(quote::quote! { fn my_fn() {} }).unwrap();
    let f = ExtendrFunction::from_item_fn(&item_fn);

    let m: ExtendrModuleFunction = syn::parse2(quote::quote! { fn my_fn }).unwrap();

    assert_eq!(f.call_method_def_ident(), m.call_method_def_ident());
    assert_eq!(f.r_wrapper_const_ident(), m.r_wrapper_const_ident());
}

#[test]
fn parsed_fn_rewrites_unnamed_dots_to_dots_arg() {
    let parsed: ExtendrFunctionParsed =
        syn::parse2(quote::quote! { fn f(a: i32, ...) -> i32 { a } }).unwrap();

    assert!(parsed.has_dots);
    assert!(parsed.named_dots.is_none());
    assert!(parsed.original_item.sig.variadic.is_none());

    let last = parsed.original_item.sig.inputs.last().unwrap();
    let syn::FnArg::Typed(pat_type) = last else {
        panic!("expected a typed arg");
    };
    let syn::Pat::Ident(pat_ident) = pat_type.pat.as_ref() else {
        panic!("expected ident pattern");
    };
    assert_eq!(pat_ident.ident, "_dots");

    let syn::Type::Reference(r) = pat_type.ty.as_ref() else {
        panic!("expected reference type");
    };
    let syn::Type::Path(tp) = r.elem.as_ref() else {
        panic!("expected path type");
    };
    assert_eq!(
        tp.path
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect::<Vec<_>>(),
        vec!["miniextendr_api", "dots", "Dots"]
    );
}

#[test]
fn parsed_fn_rewrites_named_dots_to_named_dots_arg() {
    let parsed: ExtendrFunctionParsed =
        syn::parse2(quote::quote! { fn f(a: i32, dots: ...) -> i32 { a } }).unwrap();

    assert!(parsed.has_dots);
    assert_eq!(parsed.named_dots.as_ref().unwrap(), "dots");

    let last = parsed.original_item.sig.inputs.last().unwrap();
    let syn::FnArg::Typed(pat_type) = last else {
        panic!("expected a typed arg");
    };
    let syn::Pat::Ident(pat_ident) = pat_type.pat.as_ref() else {
        panic!("expected ident pattern");
    };
    assert_eq!(pat_ident.ident, "dots");
}

#[test]
fn parsed_fn_rewrites_wildcards_and_tracks_per_param_coerce() {
    let parsed: ExtendrFunctionParsed = syn::parse2(quote::quote! {
        fn f(#[miniextendr(coerce)] _: u16, _: i32) {}
    })
    .unwrap();

    assert!(parsed.per_param_coerce.contains("__unused0"));
    assert!(!parsed.per_param_coerce.contains("__unused1"));

    let args: Vec<&syn::FnArg> = parsed.original_item.sig.inputs.iter().collect();
    let syn::FnArg::Typed(first) = args[0] else {
        panic!("expected typed arg");
    };
    let syn::Pat::Ident(first_ident) = first.pat.as_ref() else {
        panic!("expected ident pattern");
    };
    assert_eq!(first_ident.ident, "__unused0");
    assert!(!first.attrs.iter().any(is_miniextendr_coerce_attr));
}

#[test]
fn parsed_fn_errors_on_unnamed_dots_conflicting_with_dots_arg_name() {
    let err = syn::parse2::<ExtendrFunctionParsed>(quote::quote! {
        fn f(_dots: i32, ...) {}
    })
    .err()
    .unwrap();

    assert!(
        err.to_string()
            .contains("conflicts with implicit dots parameter")
    );
}

#[test]
fn parsed_fn_errors_on_non_ident_dots_pattern() {
    let err = syn::parse2::<ExtendrFunctionParsed>(quote::quote! {
        fn f((a, b): ...) {}
    })
    .err()
    .unwrap();

    assert!(
        err.to_string()
            .contains("variadic pattern must be a simple identifier")
    );
}

#[test]
fn miniextendr_attr_rejects_unknown_options() {
    let err = syn::parse2::<MiniextendrFnAttrs>(quote::quote!(typo))
        .err()
        .unwrap();
    assert!(err.to_string().contains("unknown `#[miniextendr]` option"));
}

#[test]
fn miniextendr_attr_rejects_option_arguments() {
    let err = syn::parse2::<MiniextendrFnAttrs>(quote::quote!(invisible(true)))
        .err()
        .unwrap();
    assert!(err.to_string().contains("does not take any arguments"));
}

#[test]
fn miniextendr_attr_rejects_unknown_unsafe_options() {
    let err = syn::parse2::<MiniextendrFnAttrs>(quote::quote!(unsafe(oops)))
        .err()
        .unwrap();
    assert!(err.to_string().contains("unknown `unsafe(...)` option"));
}

#[test]
fn miniextendr_attr_accepts_multiple_flags() {
    let attrs = syn::parse2::<MiniextendrFnAttrs>(quote::quote!(coerce, invisible))
        .expect("should parse multiple flags");
    assert!(attrs.coerce_all);
    assert_eq!(attrs.force_invisible, Some(true));
}
