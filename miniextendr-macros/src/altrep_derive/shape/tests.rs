use super::*;

fn input(tokens: TokenStream) -> syn::DeriveInput {
    syn::parse2(tokens).unwrap()
}

fn selector(text: &str) -> Selector {
    Selector::parse(&syn::LitStr::new(text, Span::call_site())).unwrap()
}

#[test]
fn selectors_take_names_and_tuple_indices() {
    assert!(matches!(selector("value").member, syn::Member::Named(_)));
    assert!(matches!(
        selector("1").member,
        syn::Member::Unnamed(syn::Index { index: 1, .. })
    ));
    let err = Selector::parse(&syn::LitStr::new("not a field", Span::call_site()))
        .err()
        .unwrap();
    assert!(
        err.to_string()
            .contains("is not a field name or a tuple index")
    );
}

#[test]
fn a_selector_names_a_field_of_the_struct() {
    let tuple = input(quote! { struct Outcomes<const N: usize>([Outcome; N], u8); });
    assert!(selector("1").check(&tuple).is_ok());
    let err = selector("2").check(&tuple).err().unwrap().to_string();
    assert!(err.contains("`Outcomes` has no field `2`"), "{err}");
    assert!(err.contains("`0`, `1`"), "{err}");

    let named = input(quote! { struct Dice { value: Vec<i32> } });
    let err = selector("values").check(&named).err().unwrap().to_string();
    assert!(
        err.contains("has no field `values`; its fields are `value`"),
        "{err}"
    );
}

#[test]
fn slice_fields_are_arrays_vecs_boxed_and_static_slices() {
    for (ty, borrowed) in [
        (quote! { [Die; N] }, false),
        (quote! { Vec<Die> }, false),
        (quote! { ::std::vec::Vec<Die> }, false),
        (quote! { Box<[Die]> }, false),
        (quote! { &'static [Die] }, true),
    ] {
        let ty: syn::Type = syn::parse2(ty.clone()).unwrap();
        let (elem, is_borrowed) = slice_element(&ty).expect("a slice field");
        assert_eq!(elem.to_token_stream().to_string(), "Die");
        assert_eq!(is_borrowed, borrowed);
    }
    for ty in [
        quote! { &'a [Die] },
        quote! { &'static mut [Die] },
        quote! { Box<Die> },
        quote! { std::collections::VecDeque<Die> },
        quote! { usize },
    ] {
        let ty: syn::Type = syn::parse2(ty).unwrap();
        assert!(slice_element(&ty).is_none());
    }
}

#[test]
fn the_only_slice_field_is_chosen() {
    let dice = input(quote! { struct Dice<const N: usize> { value: [Die; N], _secret: () } });
    let field = slice_field(&dice, None, "Altrep").unwrap().unwrap();
    assert_eq!(member_name(&field.member), "value");

    let tuple = input(quote! { struct Outcomes<const N: usize>([Outcome; N]); });
    let field = slice_field(&tuple, None, "Altrep").unwrap().unwrap();
    assert_eq!(member_name(&field.member), "0");

    let none = input(quote! { struct Seq { start: i32, len: usize } });
    assert!(slice_field(&none, None, "Altrep").unwrap().is_none());
}

#[test]
fn several_slice_fields_need_a_selector() {
    let two = input(quote! { struct Two { a: Vec<i32>, b: [f64; 3] } });
    let err = slice_field(&two, None, "Altrep").err().unwrap().to_string();
    assert!(err.contains("several fields"), "{err}");
    assert!(err.contains("`a`, `b`"), "{err}");
    let field = slice_field(&two, Some(&selector("b")), "Altrep")
        .unwrap()
        .unwrap();
    assert_eq!(field.elem.to_token_stream().to_string(), "f64");
}

#[test]
fn a_selector_must_name_a_slice_field() {
    let dice = input(quote! { struct Dice { value: Vec<i32>, n: usize } });
    let err = slice_field(&dice, Some(&selector("n")), "AltrepInteger")
        .err()
        .unwrap()
        .to_string();
    assert!(err.contains("names a `usize` field"), "{err}");
}

#[test]
fn generics_lose_defaults_and_gain_static_bounds() {
    let wrap = input(quote! { struct Wrap<T: Copy = i32, const N: usize = 3>([T; N]); });
    let generics = altrep_generics(&wrap.generics).unwrap();
    let target = brace_target(&wrap.ident, &generics).to_string();
    assert_eq!(
        target,
        quote! { {T: Copy, const N: usize} Wrap<T, N> {T: 'static} }.to_string()
    );
    assert!(is_generic(&generics));
    assert!(has_type_params(&generics));

    let plain = input(quote! { struct Plain(Vec<i32>); });
    let generics = altrep_generics(&plain.generics).unwrap();
    assert_eq!(
        brace_target(&plain.ident, &generics).to_string(),
        quote! { {} Plain {} }.to_string()
    );
    assert!(!is_generic(&generics));
}

#[test]
fn lifetimes_are_refused() {
    let borrowed = input(quote! { struct View<'a>(&'a [i32]); });
    let err = altrep_generics(&borrowed.generics)
        .err()
        .unwrap()
        .to_string();
    assert!(err.contains("take no lifetime parameters"), "{err}");
}
