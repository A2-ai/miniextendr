use super::*;

fn expand(
    derive: fn(syn::DeriveInput) -> syn::Result<TokenStream>,
    tokens: TokenStream,
) -> syn::Result<String> {
    derive(syn::parse2(tokens).unwrap()).map(|out| out.to_string())
}

fn error(derive: fn(syn::DeriveInput) -> syn::Result<TokenStream>, tokens: TokenStream) -> String {
    expand(derive, tokens).expect_err("an error").to_string()
}

fn has(output: &str, tokens: TokenStream) -> bool {
    output.contains(&tokens.to_string())
}

#[test]
fn an_array_field_gives_length_and_coerced_elements() {
    let out = expand(
        derive_altrep_integer,
        quote! { struct Dice<const N: usize> { value: [Die; N], _secret: () } },
    )
    .unwrap();
    assert!(has(&out, quote! { <[Die]>::len }), "{out}");
    assert!(has(
        &out,
        quote! { <Die as ::miniextendr_api::Coerce<i32>>::coerce }
    ));
    // An `i32` element is read in place; a `Die` one is copied.
    assert!(has(
        &out,
        quote! { ::miniextendr_api::altrep_data::native_view::<Die, i32> }
    ));
    assert!(has(
        &out,
        quote! { impl<const N: usize> ::miniextendr_api::altrep_data::AltrepDataptr<i32> for Dice<N> }
    ));
    assert!(has(
        &out,
        quote! { ::miniextendr_api::__impl_altvec_dataptr!({const N: usize} Dice<N> {}, i32) }
    ));
}

#[test]
fn a_generic_struct_gets_a_class_per_instantiation() {
    let out = expand(
        derive_altrep_integer,
        quote! { struct Dice<const N: usize> { value: [Die; N], _secret: () } },
    )
    .unwrap();
    assert!(has(
        &out,
        quote! { ::miniextendr_api::altrep::generic_altrep_class::<Self> }
    ));
    assert!(has(&out, quote! { ::std::string::String::from("Dice") }));
    assert!(has(&out, quote! { ::std::string::ToString::to_string(&N) }));
    // Registered on first conversion, not at package init.
    assert!(!out.contains("MX_ALTREP_REGISTRATIONS"), "{out}");
    assert!(has(
        &out,
        quote! { ::miniextendr_api::__impl_altrep_base!({const N: usize} Dice<N> {}, RUnwind) }
    ));
    assert!(has(&out, quote! { struct DiceRef<const N: usize> }));
}

#[test]
fn a_type_parameter_is_static_and_its_element_bounded() {
    let out = expand(
        derive_altrep_integer,
        quote! { struct Wrap<T, const N: usize>([T; N]); },
    )
    .unwrap();
    assert!(
        has(
            &out,
            quote! { where T: 'static, T: ::miniextendr_api::Coerce<i32> + ::core::clone::Clone }
        ),
        "{out}"
    );
    assert!(has(&out, quote! { ::core::any::type_name::<T>() }));
}

#[test]
fn a_tuple_index_selects_a_field() {
    let out = expand(
        derive_altrep_integer,
        quote! {
            #[altrep(len = "1", elt = "0")]
            struct Repeat(i32, usize);
        },
    )
    .unwrap();
    assert!(has(&out, quote! { self.1 }), "{out}");
    assert!(has(&out, quote! { self.0 }), "{out}");
}

#[test]
fn a_borrowed_slice_is_never_written() {
    let out = expand(
        derive_altrep_integer,
        quote! { struct Table(&'static [i32]); },
    )
    .unwrap();
    assert!(!out.contains("native_view_mut"), "{out}");
    assert!(has(
        &out,
        quote! {
            fn dataptr(&mut self, _writable: bool) -> ::core::option::Option<*mut i32> {
                ::core::option::Option::None
            }
        }
    ));
}

#[test]
fn logical_and_string_slices_copy_for_a_pointer() {
    let out = expand(
        derive_altrep_logical,
        quote! { struct Outcomes<const N: usize>([Outcome; N]); },
    )
    .unwrap();
    assert!(has(
        &out,
        quote! { <Outcome as ::miniextendr_api::Coerce<::miniextendr_api::Rboolean>>::coerce }
    ));
    assert!(has(
        &out,
        quote! { ::miniextendr_api::__impl_altvec_materializing_dataptr!({const N: usize} Outcomes<N> {}, ::miniextendr_api::RLogical) }
    ));

    let out = expand(derive_altrep_string, quote! { struct Names(Vec<String>); }).unwrap();
    assert!(has(&out, quote! { ::core::convert::AsRef::<str>::as_ref }));
    assert!(has(
        &out,
        quote! { ::miniextendr_api::__impl_altvec_string_dataptr!({} Names {}) }
    ));
}

#[test]
fn a_length_field_keeps_the_length_field_path() {
    let out = expand(
        derive_altrep_integer,
        quote! { struct Data { data: Vec<i32>, len: usize } },
    )
    .unwrap();
    assert!(has(&out, quote! { self.len }), "{out}");
    assert!(has(
        &out,
        quote! { ::miniextendr_api::__impl_altvec_materializing_dataptr!({} Data {}, i32) }
    ));
}

#[test]
fn field_conflicts_with_len_elt_and_manual() {
    let err = error(
        derive_altrep_integer,
        quote! {
            #[altrep(field = "data", len = "n")]
            struct Data { data: Vec<i32>, n: usize }
        },
    );
    assert!(
        err.contains("drop `len`, `elt` and `elt_delegate`"),
        "{err}"
    );
    let err = error(
        derive_altrep_integer,
        quote! {
            #[altrep(field = "data", manual)]
            struct Data { data: Vec<i32> }
        },
    );
    assert!(err.contains("use one"), "{err}");
}

#[test]
fn several_slice_fields_need_field() {
    let err = error(
        derive_altrep_real,
        quote! { struct Two { a: Vec<f64>, b: Vec<f64> } },
    );
    assert!(err.contains("#[altrep(field = \"a\")]"), "{err}");
}

#[test]
fn serialize_is_refused_on_generic_structs() {
    let err = error(
        derive_altrep_integer,
        quote! {
            #[altrep(serialize)]
            struct Dice<const N: usize>([i32; N]);
        },
    );
    assert!(err.contains("#2364"), "{err}");
    // A concrete struct keeps it.
    expand(
        derive_altrep_integer,
        quote! {
            #[altrep(serialize)]
            struct Dice([i32; 3]);
        },
    )
    .unwrap();
}

#[test]
fn an_enum_needs_manual() {
    let err = error(derive_altrep_integer, quote! { enum Die { One, Two } });
    assert!(err.contains("needs `#[altrep(manual)]`"), "{err}");
    assert!(err.contains("implement `Coerce<i32>` for `Die`"), "{err}");
    expand(
        derive_altrep_integer,
        quote! {
            #[altrep(manual)]
            enum Die { One, Two }
        },
    )
    .unwrap();
}

#[test]
fn lifetimes_and_unknown_fields_are_refused() {
    let err = error(
        derive_altrep_integer,
        quote! { struct View<'a>(&'a [i32]); },
    );
    assert!(err.contains("no lifetime parameters"), "{err}");
    let err = error(
        derive_altrep_integer,
        quote! {
            #[altrep(len = "n")]
            struct Data { len: usize }
        },
    );
    assert!(err.contains("`Data` has no field `n`"), "{err}");
}

#[test]
fn list_takes_generics_but_no_slice_field() {
    let out = expand(
        derive_altrep_list,
        quote! { struct Items<const N: usize> { len: usize, _n: [u8; N] } },
    )
    .unwrap();
    assert!(has(
        &out,
        quote! { ::miniextendr_api::impl_inferbase_list!({const N: usize} Items<N> {}) }
    ));
    let err = error(
        derive_altrep_list,
        quote! {
            #[altrep(field = "items")]
            struct Items { items: Vec<i32> }
        },
    );
    assert!(err.contains("not supported for AltrepList"), "{err}");
}
