use super::*;
use quote::quote;

#[test]
fn nested_types_consts_lifetimes_and_parameter_defaults() {
    let generics = syn::parse_quote!(<'a, T, const N: usize, U = Vec<T>>);
    let path = syn::parse_quote!(Example<'static, i32, 3>);
    let mut sub = Substitution::new(&generics, &path).unwrap();
    let mut ty: syn::Type = syn::parse_quote!((&'a [T; N], U, SizedBuffer<N>));
    sub.visit_type_mut(&mut ty);
    assert_eq!(
        quote!(#ty).to_string(),
        quote!((&'static [i32; 3], Vec<i32>, SizedBuffer<{ 3 }>)).to_string()
    );
}

#[test]
fn caller_types_are_not_reinterpreted_as_declaration_parameters() {
    let generics = syn::parse_quote!(<T, U>);
    let path = syn::parse_quote!(Example<i32, T>);
    let mut sub = Substitution::new(&generics, &path).unwrap();
    let mut ty: syn::Type = syn::parse_quote!((T, U));
    sub.visit_type_mut(&mut ty);
    assert_eq!(quote!(#ty).to_string(), quote!((i32, T)).to_string());
}

#[test]
fn only_omitted_defaults_get_synthetic_signatures() {
    let implementation = syn::parse_quote!(impl Example<i32> for Host {
        fn own(&self) -> i32 { 9 }
    });
    let declaration = vec![
        syn::parse_quote!(
            fn own(&self) -> T {}
        ),
        syn::parse_quote!(
            fn required(&self) -> T;
        ),
        syn::parse_quote!(
            #[miniextendr(r_name = "values")]
            fn inherited(&self, x: Vec<T>) -> T {}
        ),
    ];
    let methods = inherited_methods(
        &implementation,
        &declaration,
        &syn::parse_quote!(<T>),
        &syn::parse_quote!(Example<i32>),
    )
    .unwrap();
    assert_eq!(methods.len(), 1);
    let method = &methods[0];
    assert_eq!(
        quote!(#method).to_string(),
        quote!(
            #[miniextendr(r_name = "values")]
            fn inherited(&self, x: Vec<i32>) -> i32 {}
        )
        .to_string()
    );
}
