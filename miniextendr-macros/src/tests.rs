use crate::miniextendr_fn::{
    MiniextendrFnAttrs, MiniextendrFunctionParsed, is_miniextendr_coerce_attr,
};
use crate::miniextendr_module::MiniextendrModuleFunction;

#[test]
fn wrapper_idents_match_between_attribute_and_module_macros() {
    // Parse via MiniextendrFunctionParsed (attribute macro path)
    let parsed: MiniextendrFunctionParsed = syn::parse2(quote::quote! { fn my_fn() {} }).unwrap();

    // Parse via MiniextendrModuleFunction (module macro path)
    let m: MiniextendrModuleFunction = syn::parse2(quote::quote! { fn my_fn }).unwrap();

    assert_eq!(parsed.call_method_def_ident(), m.call_method_def_ident());
    assert_eq!(parsed.r_wrapper_const_ident(), m.r_wrapper_const_ident());
}

#[test]
fn parsed_fn_rewrites_unnamed_dots_to_dots_arg() {
    let parsed: MiniextendrFunctionParsed =
        syn::parse2(quote::quote! { fn f(a: i32, ...) -> i32 { a } }).unwrap();

    assert!(parsed.has_dots());
    assert!(parsed.named_dots().is_none());
    assert!(parsed.item().sig.variadic.is_none());

    let last = parsed.inputs().last().unwrap();
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
    let parsed: MiniextendrFunctionParsed =
        syn::parse2(quote::quote! { fn f(a: i32, dots: ...) -> i32 { a } }).unwrap();

    assert!(parsed.has_dots());
    assert_eq!(parsed.named_dots().unwrap(), "dots");

    let last = parsed.inputs().last().unwrap();
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
    let parsed: MiniextendrFunctionParsed = syn::parse2(quote::quote! {
        fn f(#[miniextendr(coerce)] _: u16, _: i32) {}
    })
    .unwrap();

    assert!(parsed.has_coerce_attr("__unused0"));
    assert!(!parsed.has_coerce_attr("__unused1"));

    let args: Vec<&syn::FnArg> = parsed.inputs().iter().collect();
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
    let err = syn::parse2::<MiniextendrFunctionParsed>(quote::quote! {
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
    let err = syn::parse2::<MiniextendrFunctionParsed>(quote::quote! {
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

#[test]
fn miniextendr_attr_accepts_unwrap_in_r() {
    let attrs = syn::parse2::<MiniextendrFnAttrs>(quote::quote!(unwrap_in_r))
        .expect("should parse unwrap_in_r");
    assert!(attrs.unwrap_in_r);
}

#[test]
fn parsed_fn_adds_inline_never_for_rust_abi() {
    let mut parsed: MiniextendrFunctionParsed = syn::parse2(quote::quote! { fn f() {} }).unwrap();
    parsed.add_inline_never_if_needed();

    let has_inline_never = parsed.item().attrs.iter().any(|attr| {
        attr.path().is_ident("inline")
            && matches!(&attr.meta, syn::Meta::List(list)
                if list.tokens.to_string() == "never")
    });
    assert!(
        has_inline_never,
        "should add #[inline(never)] to Rust ABI functions"
    );
}

#[test]
fn parsed_fn_preserves_explicit_inline() {
    let mut parsed: MiniextendrFunctionParsed =
        syn::parse2(quote::quote! { #[inline(always)] fn f() {} }).unwrap();
    parsed.add_inline_never_if_needed();

    // Should not add inline(never) since inline(always) is already present
    let inline_count = parsed
        .item()
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("inline"))
        .count();
    assert_eq!(
        inline_count, 1,
        "should preserve existing #[inline] attribute"
    );
}

#[test]
fn parsed_fn_no_inline_for_extern_c() {
    let mut parsed: MiniextendrFunctionParsed =
        syn::parse2(quote::quote! { extern "C-unwind" fn f() {} }).unwrap();
    parsed.add_inline_never_if_needed();

    let has_inline = parsed
        .item()
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("inline"));
    assert!(
        !has_inline,
        "should not add #[inline] to extern C functions"
    );
}

fn normalize_tokens(ts: proc_macro2::TokenStream) -> String {
    ts.to_string()
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect()
}

#[test]
fn derive_into_list_skips_ignored_named_fields() {
    let input: syn::DeriveInput = syn::parse2(quote::quote! {
        struct Foo {
            a: i32,
            #[into_list(ignore)]
            b: i32,
        }
    })
    .unwrap();

    let expanded = crate::list_derive::derive_into_list(input).unwrap();
    let s = normalize_tokens(expanded);

    assert!(s.contains("\"a\""));
    assert!(!s.contains("\"b\""));
}

#[test]
fn derive_try_from_list_defaults_ignored_named_fields() {
    let input: syn::DeriveInput = syn::parse2(quote::quote! {
        struct Foo {
            a: i32,
            #[into_list(ignore)]
            b: i32,
        }
    })
    .unwrap();

    let expanded = crate::list_derive::derive_try_from_list(input).unwrap();
    let s = normalize_tokens(expanded);

    assert!(s.contains("get_named(\"a\")"));
    assert!(!s.contains("get_named(\"b\")"));
    assert!(s.contains("b:::core::default::Default::default()"));
}

#[test]
fn derive_into_list_skips_ignored_tuple_fields() {
    let input: syn::DeriveInput = syn::parse2(quote::quote! {
        struct Foo(i32, #[into_list(ignore)] i32, i32);
    })
    .unwrap();

    let expanded = crate::list_derive::derive_into_list(input).unwrap();
    let s = normalize_tokens(expanded);

    assert!(s.contains("_field0"));
    assert!(s.contains("_field2"));
    assert!(!s.contains("_field1"));
}

#[test]
fn derive_try_from_list_defaults_ignored_tuple_fields() {
    let input: syn::DeriveInput = syn::parse2(quote::quote! {
        struct Foo(i32, #[into_list(ignore)] i32, i32);
    })
    .unwrap();

    let expanded = crate::list_derive::derive_try_from_list(input).unwrap();
    let s = normalize_tokens(expanded);

    assert!(s.contains("expected:2"));
    assert!(s.contains("get_index(0"));
    assert!(s.contains("get_index(1"));
    assert!(!s.contains("get_index(2"));
    assert!(s.contains("Self(_field0,::core::default::Default::default(),_field2)"));
}

#[test]
fn list_attrs_error_on_unknown_options() {
    let input: syn::DeriveInput = syn::parse2(quote::quote! {
        struct Foo {
            #[into_list(typo)]
            a: i32,
        }
    })
    .unwrap();

    let err = crate::list_derive::derive_into_list(input).unwrap_err();
    assert!(err.to_string().contains("unknown #[into_list(...)] option"));
}

// =============================================================================
// ALTREP derive macro tests
// =============================================================================

#[test]
fn test_derive_altrep_integer_basic() {
    let input: syn::DeriveInput = syn::parse2(quote::quote! {
        pub struct TestData {
            len: usize,
        }
    })
    .unwrap();

    let output = crate::altrep_derive::derive_altrep_integer(input).unwrap();
    let output_str = output.to_string();

    // Should generate AltrepLen impl
    assert!(output_str.contains("AltrepLen"));
    assert!(output_str.contains("fn len"));

    // Should generate AltIntegerData impl
    assert!(output_str.contains("AltIntegerData"));
    assert!(output_str.contains("fn elt"));

    // Should call impl_altinteger_from_data!
    assert!(output_str.contains("impl_altinteger_from_data"));
}

#[test]
fn test_derive_altrep_integer_with_elt_field() {
    let input: syn::DeriveInput = syn::parse2(quote::quote! {
        #[altrep(elt = "value")]
        pub struct ConstantData {
            value: i32,
            len: usize,
        }
    })
    .unwrap();

    let output = crate::altrep_derive::derive_altrep_integer(input).unwrap();
    let output_str = output.to_string();

    // Should use field for elt()
    assert!(output_str.contains("self . value"));
}

#[test]
fn test_derive_altrep_integer_with_options() {
    let input: syn::DeriveInput = syn::parse2(quote::quote! {
        #[altrep(dataptr, serialize)]
        pub struct VecData {
            data: Vec<i32>,
            len: usize,
        }
    })
    .unwrap();

    let output = crate::altrep_derive::derive_altrep_integer(input).unwrap();
    let output_str = output.to_string();

    // Should pass options to macro
    assert!(output_str.contains("dataptr"));
    assert!(output_str.contains("serialize"));
}

#[test]
fn test_derive_altrep_logical_basic() {
    let input: syn::DeriveInput = syn::parse2(quote::quote! {
        pub struct TestLogical {
            len: usize,
        }
    })
    .unwrap();

    let output = crate::altrep_derive::derive_altrep_logical(input).unwrap();
    let output_str = output.to_string();

    // Should generate AltrepLen impl
    assert!(output_str.contains("AltrepLen"));
    assert!(output_str.contains("fn len"));

    // Should generate AltLogicalData impl with default NA
    assert!(output_str.contains("AltLogicalData"));
    assert!(output_str.contains("Logical :: Na"));

    // Should call impl_altlogical_from_data!
    assert!(output_str.contains("impl_altlogical_from_data"));
}

#[test]
fn test_derive_altrep_logical_with_elt_field() {
    let input: syn::DeriveInput = syn::parse2(quote::quote! {
        #[altrep(elt = "value")]
        pub struct LogicalValue {
            value: miniextendr_api::altrep_data::Logical,
            len: usize,
        }
    })
    .unwrap();

    let output = crate::altrep_derive::derive_altrep_logical(input).unwrap();
    let output_str = normalize_tokens(output);

    // Should use field conversion via Into<Logical>
    assert!(output_str.contains("self.value.into()"));
}

#[test]
fn test_derive_altrep_logical_with_options() {
    let input: syn::DeriveInput = syn::parse2(quote::quote! {
        #[altrep(dataptr)]
        pub struct LogicalVecData {
            value: bool,
            len: usize,
        }
    })
    .unwrap();

    let output = crate::altrep_derive::derive_altrep_logical(input).unwrap();
    let output_str = output.to_string();

    // Should pass options to macro
    assert!(output_str.contains("dataptr"));
}
