use super::*;

fn arg(ty: syn::Type) -> Option<(String, bool, Mode)> {
    external_arg(&ty).map(|arg| {
        let value = &arg.value;
        (quote!(#value).to_string(), arg.optional, arg.mode)
    })
}

#[test]
fn references_to_types_without_conversions_and_views_are_external() {
    let some = |value: &str, optional, mode| Some((value.to_owned(), optional, mode));
    assert_eq!(
        arg(syn::parse_quote!(&Counter)),
        some("Counter", false, Mode::Shared)
    );
    assert_eq!(
        arg(syn::parse_quote!(&mut crate::Counter)),
        some("crate :: Counter", false, Mode::Exclusive)
    );
    assert_eq!(
        arg(syn::parse_quote!(&'a Self)),
        some("Self", false, Mode::Shared)
    );
    assert_eq!(
        arg(syn::parse_quote!(Option<&mut Counter>)),
        some("Counter", true, Mode::Exclusive)
    );
    assert_eq!(
        arg(syn::parse_quote!(ExternalPtrRef<'_, Counter>)),
        some("Counter", false, Mode::SharedView)
    );
    assert_eq!(
        arg(syn::parse_quote!(
            Option<::miniextendr_api::ExternalPtrMut<'a, Counter>>
        )),
        some("Counter", true, Mode::ExclusiveView)
    );
}

#[test]
fn native_string_dots_container_and_handle_types_are_not_external() {
    for ty in [
        quote!(&str),
        quote!(&Path),
        quote!(&i32),
        quote!(&mut f64),
        quote!(&RLogical),
        quote!(&[i32]),
        quote!(&mut [f64]),
        quote!(&Dots),
        quote!(&String),
        quote!(&Vec<i32>),
        quote!(&Option<Counter>),
        quote!(&ExternalPtr<Counter>),
        quote!(ExternalPtr<Counter>),
        quote!(Option<ExternalPtr<Counter>>),
        quote!(Vec<&Counter>),
        quote!(Counter),
    ] {
        let ty: syn::Type = syn::parse2(ty).unwrap();
        assert!(external_arg(&ty).is_none(), "{}", quote!(#ty));
    }
}

#[test]
fn borrows_follow_the_mode() {
    let sig: syn::Signature = syn::parse_quote!(fn f(
        a: &mut Counter,
        b: Option<&Counter>,
        n: i32,
        c: ExternalPtr<Counter>,
        d: ExternalPtrRef<'_, Counter>,
        e: Option<ExternalPtrMut<'_, Counter>>,
        m: &Counter,
    ));
    let borrow = arg_borrows(&sig.inputs, &["m".to_owned()], None).to_string();
    assert!(
        borrow.contains("let a = :: miniextendr_api :: externalptr :: __mx_exclusive :: < Counter > (& mut __mx_guard_a)"),
        "{borrow}"
    );
    assert!(borrow.contains("__mx_shared"), "{borrow}");
    assert!(
        borrow.contains(
            "let d = :: miniextendr_api :: externalptr :: __mx_borrow :: < Counter > (& __mx_ptr_d)"
        ),
        "{borrow}"
    );
    assert!(
        borrow.contains("let e = :: core :: option :: Option :: map"),
        "{borrow}"
    );
    assert_eq!(borrow.matches("__mx_borrow_mut").count(), 2, "{borrow}");
    assert!(
        !borrow.contains("__mx_ptr_c") && !borrow.contains("__mx_ptr_n"),
        "{borrow}"
    );
    assert!(!borrow.contains("__mx_ptr_m"), "{borrow}");
}

#[test]
fn handle_conversion_names_the_concrete_type_and_mutability() {
    let ident: syn::Ident = syn::parse_quote!(other);
    let sexp: syn::Ident = syn::parse_quote!(arg_0);
    let ty: syn::Type = syn::parse_quote!(&mut Self);
    let arg = external_arg(&ty).unwrap();
    let self_ty: syn::Type = syn::parse_quote!(Counter);
    let stmt = handle_conversion(&ident, &sexp, &ty, &arg, Some(&self_ty), quote! {}).to_string();
    assert!(stmt.contains("let mut __mx_ptr_other"), "{stmt}");
    assert!(
        stmt.contains("__mx_external_arg :: < Counter > (arg_0)"),
        "{stmt}"
    );

    let ty: syn::Type = syn::parse_quote!(&'static Counter);
    let arg = external_arg(&ty).unwrap();
    let stmt = handle_conversion(&ident, &sexp, &ty, &arg, None, quote! {}).to_string();
    assert!(stmt.contains("compile_error"), "{stmt}");
}
