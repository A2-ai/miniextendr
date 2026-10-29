use super::*;

#[test]
fn native_preflight_preserves_argument_positions_around_self_references() {
    let types: [syn::Type; 3] = [
        syn::parse_quote!(&Self),
        syn::parse_quote!((&mut i32, &i32)),
        syn::parse_quote!(&mut Self),
    ];
    let argv = format_ident!("argv");
    let output = native_borrow_checks(
        types
            .iter()
            .enumerate()
            .map(|(i, ty)| (i, format!("arg{i}"), ty)),
        &argv,
        None,
    );
    let expected = quote! { <(&mut i32, &i32) as ::miniextendr_api::TryFromSexp>::NATIVE_BORROW };
    let output = output.to_string();
    assert!(output.contains(&expected.to_string()));
    assert!(output.contains(&quote! { (*argv.add(1usize), "arg1") }.to_string()));
    assert!(!output.contains("arg0"));
    assert!(!output.contains("arg2"));
    assert!(native_borrow_checks([(0, "other".into(), &types[0])], &argv, None).is_empty());
}

#[test]
fn shared_pairs_need_no_guard_but_every_mutable_pair_does() {
    let handles = vec![
        (format_ident!("left"), false, "left".into()),
        (format_ident!("r#type"), false, "type".into()),
    ];
    assert!(
        alias_checks(
            &quote!(Counter),
            &handles,
            Some((quote!(data.cast_const()), false))
        )
        .is_empty()
    );
    let output = alias_checks(
        &quote!(Counter),
        &handles,
        Some((quote!(data.cast_const()), true)),
    )
    .to_string();
    assert_eq!(output.matches(":: core :: assert !").count(), 2);
    assert!(output.contains("'self' and 'left'"));
    assert!(output.contains("'self' and 'type'"));
    let mut handles = handles;
    handles[1].1 = true;
    let output = alias_checks(
        &quote!(Counter),
        &handles,
        Some((quote!(data.cast_const()), true)),
    )
    .to_string();
    assert_eq!(output.matches(":: core :: assert !").count(), 3);
    assert!(output.contains("'left' and 'type'"));
}

#[test]
fn self_arguments_hold_their_borrow_for_the_shim() {
    let output = borrow(&format_ident!("r#type"), true).to_string();
    assert!(output.contains("__extptr_type . borrow_mut ()"), "{output}");
    assert!(
        output.contains("let r#type = & mut * __mx_guard_type"),
        "{output}"
    );
    let output = borrow(&format_ident!("other"), false).to_string();
    assert!(output.contains("__extptr_other . borrow ()"), "{output}");
    assert!(
        output.contains("let other = & * __mx_guard_other"),
        "{output}"
    );
}
