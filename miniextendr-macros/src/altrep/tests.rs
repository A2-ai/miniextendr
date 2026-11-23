use super::*;

fn expand(tokens: proc_macro2::TokenStream) -> syn::Result<String> {
    derive_altrep(syn::parse2(tokens).unwrap()).map(|out| out.to_string())
}

fn has(output: &str, tokens: proc_macro2::TokenStream) -> bool {
    output.contains(&tokens.to_string())
}

#[test]
fn the_slice_field_names_the_element() {
    let out = expand(quote! { struct Dice<const N: usize>([Die; N]); }).unwrap();
    assert!(has(
        &out,
        quote! { impl<const N: usize> ::miniextendr_api::altrep_data::AltrepSlice for Dice<N> }
    ));
    assert!(has(&out, quote! { type Element = Die; }), "{out}");
    assert!(has(
        &out,
        quote! { ::core::option::Option::Some(::core::convert::AsMut::<[Die]>::as_mut(&mut self.0)) }
    ));
    // A factor element's attributes go on the new vector.
    assert!(has(
        &out,
        quote! { ::miniextendr_api::altrep_data::set_slice_altrep_attributes::<Self>(altrep) }
    ));
    assert!(has(
        &out,
        quote! { ::miniextendr_api::altrep::generic_altrep_class::<Self> }
    ));
}

#[test]
fn a_borrowed_slice_has_no_mutable_view() {
    let out = expand(quote! { struct Table { values: &'static [i32] } }).unwrap();
    assert!(has(
        &out,
        quote! {
            fn altrep_slice_mut(&mut self) -> ::core::option::Option<&mut [i32]> {
                ::core::option::Option::None
            }
        }
    ));
    // Concrete: registered at package init.
    assert!(out.contains("MX_ALTREP_REGISTRATIONS"), "{out}");
}

#[test]
fn a_type_parameter_element_is_bounded() {
    let out = expand(quote! { struct Wrap<T>(Vec<T>); }).unwrap();
    assert!(
        has(
            &out,
            quote! { where T: 'static, T: ::miniextendr_api::altrep_data::AltrepElement }
        ),
        "{out}"
    );
}

#[test]
fn field_picks_one_of_several() {
    let out = expand(quote! {
        #[altrep(field = "b")]
        struct Two { a: Vec<i32>, b: Box<[String]> }
    })
    .unwrap();
    assert!(has(&out, quote! { type Element = String; }), "{out}");
    let err = expand(quote! { struct Two { a: Vec<i32>, b: Box<[String]> } })
        .err()
        .unwrap()
        .to_string();
    assert!(err.contains("several fields"), "{err}");
}

#[test]
fn manual_registers_only() {
    let out = expand(quote! {
        #[altrep(manual, class = "Mine")]
        struct Mine { data: Vec<i32> }
    })
    .unwrap();
    assert!(!out.contains("AltrepSlice"), "{out}");
    assert!(!out.contains("set_slice_altrep_attributes"), "{out}");
    assert!(out.contains("RegisterAltrep"), "{out}");
    let err = expand(quote! {
        #[altrep(manual, field = "data")]
        struct Mine { data: Vec<i32> }
    })
    .err()
    .unwrap()
    .to_string();
    assert!(err.contains("use one"), "{err}");
}

#[test]
fn errors_point_at_the_way_out() {
    let err = expand(quote! { enum Die { One, Two } })
        .err()
        .unwrap()
        .to_string();
    assert!(err.contains("derive `RFactor` on `Die`"), "{err}");
    assert!(err.contains("`MatchArg`"), "{err}");

    let err = expand(quote! { struct Seq { start: i32, len: usize } })
        .err()
        .unwrap()
        .to_string();
    assert!(err.contains("`Seq` has none"), "{err}");
    assert!(err.contains("#[altrep(manual)]"), "{err}");

    let err = expand(quote! {
        #[altrep(serialize)]
        struct Dice([i32; 3]);
    })
    .err()
    .unwrap()
    .to_string();
    assert!(
        err.contains("expected `class`, `field` or `manual`"),
        "{err}"
    );
}
