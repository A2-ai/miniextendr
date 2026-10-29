use super::*;

// At unit-test runtime cargo does not forward CARGO_CRATE_NAME to the
// test binary's process env, so crate_prefix() takes the CARGO_PKG_NAME
// fallback: "miniextendr-macros" → "miniextendr_macros". Real macro
// expansion (inside rustc) always sees CARGO_CRATE_NAME. The insta
// snapshots pin the same value; if this test fails the snapshots are
// stale too.
#[test]
fn crate_prefix_fallback_is_normalized_pkg_name() {
    assert_eq!(crate_prefix(), "miniextendr_macros");
}

#[test]
fn c_symbols_are_crate_prefixed() {
    let ty = quote::format_ident!("Counter");
    let tr = quote::format_ident!("Resettable");
    let m = quote::format_ident!("reset");
    assert_eq!(
        bare_fn_c_wrapper_ident(&m).to_string(),
        "C_miniextendr_macros_reset"
    );
    assert_eq!(
        impl_method_c_wrapper_ident(&ty, &m).to_string(),
        "C_miniextendr_macros_Counter__reset"
    );
    assert_eq!(
        trait_member_c_wrapper_string(&ty, &tr, &m),
        "C_miniextendr_macros_Counter__Resettable__reset"
    );
    assert_eq!(
        trait_member_c_wrapper_ident(&ty, &tr, &m).to_string(),
        trait_member_c_wrapper_string(&ty, &tr, &m)
    );
    assert_eq!(
        sidecar_getter_c_name("Counter", "count"),
        "C_miniextendr_macros__mx_rdata_get_Counter_count"
    );
    assert_eq!(
        sidecar_setter_c_name("Counter", "count"),
        "C_miniextendr_macros__mx_rdata_set_Counter_count"
    );
    assert_eq!(
        altrep_reg_fn_ident(&ty).to_string(),
        "__mx_altrep_reg_miniextendr_macros_Counter"
    );
    assert_eq!(
        vtable_static_ident(&tr, "Counter").to_string(),
        "__VTABLE_miniextendr_macros_Resettable_FOR_Counter"
    );
    assert_eq!(
        vtshim_ident(&ty, &tr, &m).to_string(),
        "__vtshim_miniextendr_macros_Counter__Resettable__reset"
    );
}

#[test]
fn trait_symbols_preserve_case_and_strip_raw_prefixes() {
    let upper = quote::format_ident!("Counter");
    let lower = quote::format_ident!("counter");
    let method = quote::format_ident!("value");
    assert_ne!(
        vtable_static_ident(&upper, "Foo"),
        vtable_static_ident(&lower, "Foo")
    );
    assert_ne!(
        vtable_static_ident(&upper, "Foo"),
        vtable_static_ident(&upper, "foo")
    );
    assert_ne!(trait_tag_ident(&upper), trait_tag_ident(&lower));
    assert_ne!(
        trait_vtable_builder_ident(&upper),
        trait_vtable_builder_ident(&lower)
    );
    assert_ne!(
        trait_method_shim_ident(&upper, &method),
        trait_method_shim_ident(&lower, &method)
    );
    assert_ne!(
        trait_impl_wrappers_ident("Foo", &upper),
        trait_impl_wrappers_ident("Foo", &lower)
    );
    assert_ne!(
        trait_dispatch_ident(&upper, "Foo"),
        trait_dispatch_ident(&lower, "Foo")
    );
    let raw: syn::Ident = syn::parse_quote!(r#match);
    assert_eq!(trait_tag_ident(&raw), "TAG_match");
    assert_eq!(trait_vtable_builder_ident(&raw), "__match_build_vtable");
    assert_eq!(
        trait_method_shim_ident(&upper, &raw),
        "__Counter_match_shim"
    );
}

#[test]
fn raw_identifiers_lose_their_prefix_in_names() {
    let raw: syn::Ident = syn::parse_quote!(r#where);
    let plain: syn::Ident = syn::parse_quote!(x);
    assert_eq!(raw.to_string(), "r#where", "Display keeps the prefix");
    assert_eq!(ident_name(&raw), "where");
    assert_eq!(ident_name(&plain), "x");
    assert_eq!(unraw(&raw).to_string(), "where");
    assert_eq!(
        quote::format_ident!("__storage_{}", unraw(&raw)),
        "__storage_where"
    );
    // The C symbol / const-name formatters unraw internally.
    assert_eq!(
        bare_fn_c_wrapper_ident(&syn::parse_quote!(r#match)).to_string(),
        "C_miniextendr_macros_match"
    );
    assert_eq!(
        r_wrapper_const_ident_for(&syn::parse_quote!(r#match)).to_string(),
        "R_WRAPPER_match"
    );
    let ty: syn::Ident = syn::parse_quote!(Row);
    assert_eq!(
        impl_method_c_wrapper_ident(&ty, &syn::parse_quote!(r#type)).to_string(),
        "C_miniextendr_macros_Row__type"
    );
}

#[test]
fn name_ident_makes_keywords_raw() {
    assert_eq!(name_ident("type").to_string(), "r#type");
    assert_eq!(name_ident("where").to_string(), "r#where");
    assert_eq!(name_ident("value").to_string(), "value");
    // Round-trips: a raw ident's name maps back to the same raw ident.
    let raw: syn::Ident = syn::parse_quote!(r#type);
    assert_eq!(name_ident(&ident_name(&raw)), raw);
}

#[test]
fn snake_case() {
    assert_eq!(to_snake_case("HelloWorld"), "hello_world");
    assert_eq!(to_snake_case("InProgress"), "in_progress");
    assert_eq!(to_snake_case("ABC"), "a_b_c");
    assert_eq!(to_snake_case("Red"), "red");
}

#[test]
fn kebab_case() {
    assert_eq!(to_kebab_case("HelloWorld"), "hello-world");
    assert_eq!(to_kebab_case("InProgress"), "in-progress");
}

#[test]
fn rename_all() {
    assert_eq!(
        apply_rename_all("HelloWorld", Some("snake_case")),
        "hello_world"
    );
    assert_eq!(
        apply_rename_all("HelloWorld", Some("kebab-case")),
        "hello-world"
    );
    assert_eq!(apply_rename_all("HelloWorld", Some("lower")), "helloworld");
    assert_eq!(apply_rename_all("HelloWorld", Some("upper")), "HELLOWORLD");
    assert_eq!(apply_rename_all("HelloWorld", None), "HelloWorld");
}

#[test]
fn namespace_names_quote_operators_only() {
    for name in ["get", "var", ".hidden", "s7_trait_Ops_value", "Foo_at"] {
        assert_eq!(r_namespace_name(name), name);
        assert_eq!(r_def_name(name), name);
    }
    for (name, ns, def) in [
        ("[", r#""[""#, "`[`"),
        ("[[", r#""[[""#, "`[[`"),
        ("$", r#""$""#, "`$`"),
        ("%custom%", r#""%custom%""#, "`%custom%`"),
        ("Foo_[[", r#""Foo_[[""#, "`Foo_[[`"),
        ("if", r#""if""#, "`if`"),
    ] {
        assert_eq!(r_namespace_name(name), ns);
        assert_eq!(r_def_name(name), def);
    }
    // The NAMESPACE form is an R string literal: escape `"` and `\`.
    assert_eq!(r_namespace_name(r#"a"b\c"#), r#""a\"b\\c""#);
}

#[test]
fn def_names_escape_backslashes_before_backticks() {
    // R reads backslash escapes inside backticks, so `\` must be doubled
    // (`` `a\\b` `` is the three-character name `a\b`) and a trailing one must
    // not escape the closing backtick.
    assert_eq!(r_def_name(r"a\b"), r"`a\\b`");
    assert_eq!(r_def_name(r"trail\"), r"`trail\\`");
    assert_eq!(r_def_name("a`b"), r"`a\`b`");
    // A backslash next to a backtick: each is escaped once, in order.
    assert_eq!(r_def_name(r"x\`y"), r"`x\\\`y`");
    assert_eq!(r_def_name(r"%\%"), r"`%\\%`");
}

#[test]
fn package_qualified_generics() {
    let parse = |s: &str| parse_generic(&syn::LitStr::new(s, proc_macro2::Span::call_site()));
    for ok in [
        "print",
        "[[",
        "dplyr::dplyr_reconstruct",
        "data.table::as.data.table",
    ] {
        assert_eq!(parse(ok).unwrap(), ok);
    }
    assert_eq!(
        split_generic("vctrs::vec_ptype2"),
        (Some("vctrs"), "vec_ptype2")
    );
    assert_eq!(split_generic("format"), (None, "format"));
    for (bad, why) in [
        ("dplyr:::dplyr_reconstruct", "not `pkg:::generic`"),
        ("a::b::c", "one `pkg::` qualifier"),
        ("dplyr::", "no generic name"),
        ("::print", "not an R package name"),
        ("x::print", "not an R package name"),
        ("2pkg::print", "not an R package name"),
        ("pkg.::print", "not an R package name"),
        ("my_pkg::print", "not an R package name"),
    ] {
        let message = parse(bad).unwrap_err().to_string();
        assert!(message.contains(why), "{bad}: {message}");
    }
}
