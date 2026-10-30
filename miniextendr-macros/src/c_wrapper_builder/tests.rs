use super::*;

#[test]
fn return_handling_detection() {
    // Default (no return type) -> Unit
    assert!(matches!(
        detect_return_handling(&syn::ReturnType::Default, None),
        ReturnHandling::Unit
    ));

    // -> () -> Unit
    let unit_ty: syn::ReturnType = syn::parse_quote!(-> ());
    assert!(matches!(
        detect_return_handling(&unit_ty, None),
        ReturnHandling::Unit
    ));

    // -> i32 -> IntoR
    let i32_ty: syn::ReturnType = syn::parse_quote!(-> i32);
    assert!(matches!(
        detect_return_handling(&i32_ty, None),
        ReturnHandling::IntoR
    ));

    // -> Self -> ExternalPtr
    let self_ty: syn::ReturnType = syn::parse_quote!(-> Self);
    assert!(matches!(
        detect_return_handling(&self_ty, None),
        ReturnHandling::ExternalPtr
    ));

    // -> Option<i32> -> OptionIntoR (None becomes NA_integer_, as from a function)
    let option_ty: syn::ReturnType = syn::parse_quote!(-> Option<i32>);
    assert!(matches!(
        detect_return_handling(&option_ty, None),
        ReturnHandling::OptionIntoR
    ));

    // -> Option<()> -> OptionUnit
    let option_unit_ty: syn::ReturnType = syn::parse_quote!(-> Option<()>);
    assert!(matches!(
        detect_return_handling(&option_unit_ty, None),
        ReturnHandling::OptionUnit
    ));

    // -> Result<i32, E> -> ResultIntoR
    let result_ty: syn::ReturnType = syn::parse_quote!(-> Result<i32, E>);
    assert!(matches!(
        detect_return_handling(&result_ty, None),
        ReturnHandling::ResultIntoR
    ));

    // -> Result<(), E> -> ResultUnit
    let result_unit_ty: syn::ReturnType = syn::parse_quote!(-> Result<(), E>);
    assert!(matches!(
        detect_return_handling(&result_unit_ty, None),
        ReturnHandling::ResultUnit
    ));

    // -> Result<i32, ()> -> ResultNullOnErr (unit error is a sentinel, always returns NULL)
    let result_null_on_err_ty: syn::ReturnType = syn::parse_quote!(-> Result<i32, ()>);
    assert!(matches!(
        detect_return_handling(&result_null_on_err_ty, None),
        ReturnHandling::ResultNullOnErr
    ));

    // -> Result<(), ()> -> ResultNullOnErr (both unit: returns NULL regardless)
    let result_unit_unit_ty: syn::ReturnType = syn::parse_quote!(-> Result<(), ()>);
    assert!(matches!(
        detect_return_handling(&result_unit_unit_ty, None),
        ReturnHandling::ResultNullOnErr
    ));

    // -> Result<Self, E> -> ResultExternalPtr (audit A4: fallible constructor-shaped
    // methods like `from_r` wrap `Ok(Self)` in an ExternalPtr, not `IntoR`).
    let result_self_ty: syn::ReturnType = syn::parse_quote!(-> Result<Self, E>);
    assert!(matches!(
        detect_return_handling(&result_self_ty, None),
        ReturnHandling::ResultExternalPtr
    ));

    // -> Option<Self> -> OptionExternalPtr (#1164: lookup-shaped fallible constructors
    // like `try_find` wrap `Some(Self)` in an ExternalPtr, not `IntoR` — symmetric with
    // the Result<Self, E> case above).
    let option_self_ty: syn::ReturnType = syn::parse_quote!(-> Option<Self>);
    assert!(matches!(
        detect_return_handling(&option_self_ty, None),
        ReturnHandling::OptionExternalPtr
    ));
}

#[test]
fn alias_guard_emission() {
    // Build a minimal CWrapperContext whose `inputs` are the parameters of the
    // given function signature.
    fn ctx_for(sig: syn::ItemFn) -> CWrapperContext {
        CWrapperContext::builder(sig.sig.ident.clone(), syn::parse_quote!(C_test))
            .r_wrapper_const(syn::parse_quote!(R_WRAPPER_test))
            .call_expr(quote::quote!(test()))
            .inputs(sig.sig.inputs)
            .build()
    }

    let sexp_idents: Vec<syn::Ident> = vec![syn::parse_quote!(arg_0), syn::parse_quote!(arg_1)];

    // Two &mut [T] params -> an unconditional pre-conversion guard is emitted.
    let ctx = ctx_for(syn::parse_quote!(
        fn alias_probe(a: &mut [i32], b: &mut [i32]) {}
    ));
    let guard = ctx.build_alias_guard(&sexp_idents).to_string();
    assert!(guard.contains("assert"), "guard = {guard}");
    assert!(!guard.contains("debug_assert"), "guard = {guard}");
    assert!(guard.contains("arg_0"), "guard = {guard}");
    assert!(guard.contains("arg_1"), "guard = {guard}");

    // Two mut slices where one is Option-wrapped -> still guarded.
    let ctx = ctx_for(syn::parse_quote!(
        fn opt_mut(a: &mut [i32], b: Option<&mut [i32]>) {}
    ));
    assert!(!ctx.build_alias_guard(&sexp_idents).is_empty());

    // One &mut [T] + one &[T]: a mutable and a shared borrow over the same
    // buffer is also UB, so a guard IS emitted (#1104 mixed-aliasing case).
    let ctx = ctx_for(syn::parse_quote!(
        fn one_mut(a: &mut [i32], b: &[i32]) {}
    ));
    assert!(!ctx.build_alias_guard(&sexp_idents).is_empty());

    // Shared-only metadata is filtered before entering the runtime check.
    let ctx = ctx_for(syn::parse_quote!(
        fn two_shared(a: &[i32], b: &[i32]) {}
    ));
    assert!(
        ctx.build_alias_guard(&sexp_idents)
            .to_string()
            .contains("needs_check")
    );

    // A single parameter can hide a mutable list behind a type alias.
    let ctx = ctx_for(syn::parse_quote!(
        fn single(a: BorrowAlias) {}
    ));
    let guard = ctx
        .build_alias_guard(&[syn::parse_quote!(arg_0)])
        .to_string();
    assert!(guard.contains("BorrowAlias as :: miniextendr_api :: TryFromSexp"));
}

#[test]
fn alias_guard_covers_borrowed_lists() {
    for sig in [
        syn::parse_quote!(
            fn probe(a: Vec<&mut [i32]>, b: &[i32]) {}
        ),
        syn::parse_quote!(
            fn probe(a: Vec<&[i32]>, b: &mut [i32]) {}
        ),
        syn::parse_quote!(
            fn probe(a: Vec<&mut [i32]>, b: Vec<&[i32]>) {}
        ),
        syn::parse_quote!(
            fn probe(a: Option<Vec<Option<&mut [i32]>>>, b: &[i32]) {}
        ),
        syn::parse_quote!(
            fn probe(a: Vec<Vec<&mut [i32]>>, b: &[i32]) {}
        ),
    ] {
        let sig: syn::ItemFn = sig;
        let ctx = CWrapperContext::builder(sig.sig.ident.clone(), syn::parse_quote!(C_probe))
            .r_wrapper_const(syn::parse_quote!(R_WRAPPER_probe))
            .call_expr(quote::quote!(probe()))
            .inputs(sig.sig.inputs)
            .build();
        assert!(
            !ctx.build_alias_guard(&[syn::parse_quote!(arg_0), syn::parse_quote!(arg_1)])
                .is_empty(),
            "borrowed list elements must participate in the pre-conversion alias guard"
        );
    }
}

#[test]
fn alias_guard_also_rejects_conflicts_in_release() {
    let sig: syn::ItemFn = syn::parse_quote!(
        fn probe(a: &mut [i32], b: &[i32]) {}
    );
    let ctx = CWrapperContext::builder(sig.sig.ident.clone(), syn::parse_quote!(C_probe))
        .r_wrapper_const(syn::parse_quote!(R_WRAPPER_probe))
        .call_expr(quote::quote!(probe()))
        .inputs(sig.sig.inputs)
        .build();
    let guard = ctx
        .build_alias_guard(&[syn::parse_quote!(arg_0), syn::parse_quote!(arg_1)])
        .to_string();
    assert!(
        !guard.contains("debug_assert"),
        "release wrappers must reject aliasing too"
    );
}

#[test]
fn alias_guard_covers_native_scalar_and_boxed_borrows() {
    for sig in [
        syn::parse_quote!(
            fn probe(a: &mut i32, b: &i32) {}
        ),
        syn::parse_quote!(
            fn probe(a: &mut i32, b: &[i32]) {}
        ),
        syn::parse_quote!(
            fn probe(a: Vec<&mut i32>, b: &i32) {}
        ),
        syn::parse_quote!(
            fn probe(a: Box<[&mut [i32]]>, b: &[i32]) {}
        ),
        syn::parse_quote!(
            fn probe(a: ScalarAlias, b: BoxedAlias) {}
        ),
    ] {
        let sig: syn::ItemFn = sig;
        let ctx = CWrapperContext::builder(sig.sig.ident.clone(), syn::parse_quote!(C_probe))
            .r_wrapper_const(syn::parse_quote!(R_WRAPPER_probe))
            .call_expr(quote::quote!(probe()))
            .inputs(sig.sig.inputs)
            .build();
        assert!(
            !ctx.build_alias_guard(&[syn::parse_quote!(arg_0), syn::parse_quote!(arg_1)])
                .is_empty(),
            "scalar, boxed, and aliased conversion types need native borrow preflight"
        );
    }
}

/// A `no_na` parameter's converted value is checked in the wrapper.
#[test]
fn no_na_check_renders_in_the_wrapper() {
    let sig: syn::ItemFn = syn::parse_quote!(
        fn dose(x: AsNumeric) -> f64 {}
    );
    let main = CWrapperContext::builder(sig.sig.ident.clone(), syn::parse_quote!(C_dose))
        .r_wrapper_const(syn::parse_quote!(R_WRAPPER_dose))
        .call_expr(quote::quote!(dose(x)))
        .inputs(sig.sig.inputs.clone())
        .output(sig.sig.output.clone())
        .no_na("x".to_string(), None)
        .build()
        .generate()
        .to_string();
    let check = ":: miniextendr_api :: TryFromSexp :: __mx_has_na (& x)";
    assert!(main.contains(check), "{main}");
    assert!(main.contains("\"'x' must not be NA\""), "{main}");
}

#[test]
fn every_none_error_path_uses_the_r_name() {
    for handling in [ReturnHandling::OptionUnit, ReturnHandling::OptionSexp] {
        let context =
            CWrapperContext::builder(syn::parse_quote!(rust_name), syn::parse_quote!(C_test))
                .r_name("visible.name".to_owned())
                .r_wrapper_const(syn::parse_quote!(R_WRAPPER_test))
                .call_expr(quote::quote!(rust_name()))
                .return_handling(handling)
                .build();
        let call = quote::quote!(rust_name());
        let tokens = context.generate_return_handling(&call).to_string();
        assert!(
            tokens.contains("`visible.name()` returned no value"),
            "{tokens}"
        );
        assert!(
            !tokens.contains("`rust_name()` returned no value"),
            "{tokens}"
        );
    }
}

/// `Option<Self>` and the fallible self-handle step give `NULL` for `None`, as
/// every other `Option` return does; neither raises.
#[test]
fn option_self_returns_give_null_for_none() {
    let context = |handling| {
        CWrapperContext::builder(syn::parse_quote!(rust_name), syn::parse_quote!(C_test))
            .r_wrapper_const(syn::parse_quote!(R_WRAPPER_test))
            .call_expr(quote::quote!(rust_name()))
            .return_handling(handling)
            .build()
    };
    let call = quote::quote!(rust_name());

    let lookup = context(ReturnHandling::OptionExternalPtr);
    let tokens = lookup.generate_return_handling(&call).to_string();
    assert!(
        tokens.contains("map (:: miniextendr_api :: externalptr :: ExternalPtr :: new)"),
        "{tokens}"
    );
    assert!(!tokens.contains("NONE_ERR"), "{tokens}");

    let step = context(ReturnHandling::SelfHandleOption);
    let main = step.generate_return_handling(&call).to_string();
    assert!(
        main.contains("return :: miniextendr_api :: SEXP :: nil ()"),
        "{main}"
    );
    assert!(!main.contains("NONE_ERR"), "{main}");
}

/// Borrowed external arguments (`&T` / `&mut T` / views, #2106) convert to
/// their handles with the other arguments and are borrowed after every
/// conversion and post-conversion statement; an `ExternalPtr<T>` handle is
/// not borrowed, and a method's `Self` names the impl's type.
#[test]
fn argument_borrows_follow_conversion() {
    let sig: syn::ItemFn = syn::parse_quote!(
        fn pair(
            a: &mut Self,
            b: Option<&Counter>,
            c: ExternalPtr<Counter>,
            d: ExternalPtrRef<'_, Counter>,
        ) {
        }
    );
    let main = CWrapperContext::builder(sig.sig.ident.clone(), syn::parse_quote!(C_pair))
        .r_wrapper_const(syn::parse_quote!(R_WRAPPER_pair))
        .call_expr(quote::quote!(pair(a, b, c, d)))
        .inputs(sig.sig.inputs.clone())
        .post_conversion(vec![quote::quote!(post_conversion_marker();)])
        .type_context(syn::parse_quote!(Counter))
        .preserve_param_names()
        .build()
        .generate()
        .to_string();

    let conversion = main
        .find("let mut __mx_ptr_a = match :: miniextendr_api :: from_r :: __mx_external_arg :: < Counter >")
        .expect("handle conversion");
    let handle = main.find("let c :").expect("handle argument");
    let post = main
        .find("post_conversion_marker")
        .expect("post conversion");
    let exclusive = main
        .find("__mx_borrow_mut :: < Counter > (& mut __mx_ptr_a)")
        .expect("exclusive borrow");
    let call = main.find("pair (a , b , c , d)").expect("call");
    assert!(
        conversion < handle && handle < post && post < exclusive && exclusive < call,
        "{main}"
    );
    assert!(
        main.contains("__mx_optional_external_arg :: < Counter > (b)"),
        "{main}"
    );
    assert!(
        main.contains(
            "let d = :: miniextendr_api :: externalptr :: __mx_borrow :: < Counter > (& __mx_ptr_d)"
        ),
        "{main}"
    );
    assert!(!main.contains("__mx_ptr_c"), "{main}");
}

/// One rule for every `Option<T>` return, whoever returns it (#1415, #2088):
/// the whole `Option` converts through `IntoR`. Only `Option<()>`,
/// `Option<SEXP>` and `Option<Self>` have their own shapes.
#[test]
fn every_option_converts_through_its_own_into_r() {
    for output in [
        syn::parse_quote!(-> Option<i32>),
        syn::parse_quote!(-> Option<f64>),
        syn::parse_quote!(-> Option<bool>),
        syn::parse_quote!(-> Option<String>),
        syn::parse_quote!(-> Option<std::string::String>),
        // Scalars and their qualified spellings (from #1490's #1415 list).
        syn::parse_quote!(-> Option<Rboolean>),
        syn::parse_quote!(-> Option<RLogical>),
        syn::parse_quote!(-> Option<Rcomplex>),
        syn::parse_quote!(-> Option<miniextendr_api::Rcomplex>),
        syn::parse_quote!(-> Option<i8>),
        syn::parse_quote!(-> Option<i16>),
        syn::parse_quote!(-> Option<u16>),
        syn::parse_quote!(-> Option<u32>),
        syn::parse_quote!(-> Option<f32>),
        syn::parse_quote!(-> Option<i64>),
        syn::parse_quote!(-> Option<u64>),
        syn::parse_quote!(-> Option<isize>),
        syn::parse_quote!(-> Option<usize>),
        syn::parse_quote!(-> Option<std::primitive::i32>),
        syn::parse_quote!(-> Option<PathBuf>),
        syn::parse_quote!(-> Option<std::path::PathBuf>),
        syn::parse_quote!(-> Option<OsString>),
        syn::parse_quote!(-> Option<std::ffi::OsString>),
        syn::parse_quote!(-> Option<&str>),
        syn::parse_quote!(-> Option<&mut str>),
        syn::parse_quote!(-> Option<String<T>>),
        syn::parse_quote!(-> Option<Custom>),
        syn::parse_quote!(-> Option<u8>),
        syn::parse_quote!(-> Option<Self::Item>),
        syn::parse_quote!(-> Option<<Self as Trait>::Item>),
        syn::parse_quote!(-> Option<Vec<i32>>),
        syn::parse_quote!(-> Option<std::vec::Vec<String>>),
        syn::parse_quote!(-> Option<Vec<Option<f64>>>),
        syn::parse_quote!(-> Option<Box<[String]>>),
        syn::parse_quote!(-> Option<[i32; 3]>),
        syn::parse_quote!(-> Option<(i32, String)>),
        syn::parse_quote!(-> Option<HashMap<String, i32>>),
        syn::parse_quote!(-> Option<BTreeSet<String>>),
        syn::parse_quote!(-> Option<&'static i32>),
        syn::parse_quote!(-> Option<&'static str>),
        syn::parse_quote!(-> Option<&'static [i32]>),
        // Class shapes too: the R wrapper's class wrap lets NULL through.
        syn::parse_quote!(-> Option<Board>),
        syn::parse_quote!(-> Option<ExternalPtr<Board>>),
        syn::parse_quote!(-> Option<Vec<Board>>),
        syn::parse_quote!(-> Option<Vec<ExternalPtr<Board>>>),
        syn::parse_quote!(-> Option<Vec<Self>>),
    ] {
        assert!(
            matches!(
                detect_return_handling(&output, None),
                ReturnHandling::OptionIntoR
            ),
            "{}",
            quote::quote!(#output)
        );
    }
    // The special shapes are recognised by their last path segment.
    let sexp: syn::ReturnType = syn::parse_quote!(-> Option<miniextendr_api::SEXP>);
    assert!(matches!(
        detect_return_handling(&sexp, None),
        ReturnHandling::OptionSexp
    ));
}

/// An explicit class wrap names a class, and no `IntoR for Option<Class>` can
/// exist: `Option<Class>` converts its payload (None is NULL). `Option<Vec<Class>>`
/// has its `Option` impl and keeps the common rule.
#[test]
fn explicit_class_wrap_converts_an_optional_payload() {
    let wrapped = |output: &str| -> (syn::ReturnType, crate::return_wrap::ReturnWrap) {
        let (wrap, output) =
            crate::return_wrap::resolve(&syn::parse_str(output).unwrap(), None).unwrap();
        (output, wrap.unwrap())
    };
    for output in [
        "-> Option<WrapAsR6<Board>>",
        "-> Option<WrapAsS7<ExternalPtr<Board>>>",
    ] {
        let (output, wrap) = wrapped(output);
        assert!(matches!(
            detect_return_handling(&output, Some(&wrap)),
            ReturnHandling::OptionClassPayload
        ));
    }
    for (output, expected) in [
        ("-> Option<Vec<WrapAsR6<Board>>>", "OptionIntoR"),
        ("-> WrapAsR6<Board>", "IntoR"),
        ("-> Result<WrapAsR6<Board>, String>", "ResultIntoR"),
        ("-> Vec<WrapAsR6<Board>>", "IntoR"),
    ] {
        let (output, wrap) = wrapped(output);
        let handling = format!("{:?}", detect_return_handling(&output, Some(&wrap)));
        assert_eq!(handling, expected);
    }
    // The attribute spelling selects the same plan.
    let output: syn::ReturnType = syn::parse_quote!(-> Option<Board>);
    let (wrap, output) =
        crate::return_wrap::resolve(&output, Some(crate::miniextendr_impl::ClassSystem::S4))
            .unwrap();
    assert!(matches!(
        detect_return_handling(&output, wrap.as_ref()),
        ReturnHandling::OptionClassPayload
    ));
}

/// `OptionClassPayload` gives NULL for `None`, never the absence error.
#[test]
fn optional_class_payload_none_is_null() {
    let context = CWrapperContext::builder(syn::parse_quote!(find), syn::parse_quote!(C_test))
        .r_wrapper_const(syn::parse_quote!(R_WRAPPER_test))
        .output(syn::parse_quote!(-> Option<Board>))
        .call_expr(quote::quote!(find()))
        .return_handling(ReturnHandling::OptionClassPayload)
        .build();
    let call = quote::quote!(find());
    let tokens = context.generate_return_handling(&call).to_string();
    assert!(tokens.contains("SEXP :: nil ()"), "{tokens}");
    assert!(
        tokens.contains("IntoR :: try_into_sexp (__result)"),
        "{tokens}"
    );
    assert!(!tokens.contains("NONE_ERR"), "{tokens}");
}

/// Every arm that converts a value converts it with `try_into_sexp` and turns
/// a failed conversion into the `result_err` condition of the error (its
/// classes or its `Display` text, `__mx_into_r_err_parts!`) on the wrapper's
/// call, never into a panic of `into_sexp` (#2272).
#[test]
fn every_return_conversion_raises_its_error() {
    let handlings = [
        ("-> Board", ReturnHandling::ExternalPtr),
        ("-> i32", ReturnHandling::IntoR),
        ("-> Option<i32>", ReturnHandling::OptionIntoR),
        ("-> Option<Board>", ReturnHandling::OptionClassPayload),
        ("-> Option<Board>", ReturnHandling::OptionExternalPtr),
        ("-> Result<i32, E>", ReturnHandling::ResultIntoR),
        ("-> Result<i32, ()>", ReturnHandling::ResultNullOnErr),
        ("-> Result<Board, E>", ReturnHandling::ResultExternalPtr),
        ("-> Board", ReturnHandling::AsListOf),
        ("-> Board", ReturnHandling::AsExternalPtrOf),
        ("-> u8", ReturnHandling::AsNativeOf),
    ];
    for (output, handling) in handlings {
        let context = CWrapperContext::builder(syn::parse_quote!(f), syn::parse_quote!(C_test))
            .r_wrapper_const(syn::parse_quote!(R_WRAPPER_test))
            .output(syn::parse_str(output).unwrap())
            .call_expr(quote::quote!(f()))
            .return_handling(handling.clone())
            .build();
        let tokens = context
            .generate_return_handling(&quote::quote!(f()))
            .to_string();
        assert!(
            tokens.contains(":: miniextendr_api :: into_r :: IntoR :: try_into_sexp ("),
            "{handling:?}: {tokens}"
        );
        assert!(!tokens.contains(":: into_sexp ("), "{handling:?}: {tokens}");
        assert!(
            tokens.contains(
                "result_err_condition_value (:: miniextendr_api :: __mx_into_r_err_parts ! \
                 (__mx_error) , :: core :: option :: Option :: Some (__miniextendr_call) ,)"
            ),
            "{handling:?}: {tokens}"
        );
    }
}

/// A trait vtable shim or a sidecar getter, which have no call slot, convert
/// the same way with `None` for the call.
#[test]
fn return_conversion_without_a_call_slot() {
    let tokens = return_conversion(
        &quote::quote!(result),
        &quote::quote!(::core::option::Option::None),
    )
    .to_string();
    assert!(
        tokens.starts_with("match :: miniextendr_api :: into_r :: IntoR :: try_into_sexp (result)"),
        "{tokens}"
    );
    assert!(
        tokens.contains("(__mx_error) , :: core :: option :: Option :: None ,)"),
        "{tokens}"
    );
}

#[test]
fn converted_references_keep_metadata_and_roots_without_external_guards() {
    for (ty, converter) in [
        ("&Name", "TryFromRoot"),
        ("Option<&Name>", "TryFromRoot"),
        ("&mut Values", "TryFromSexp"),
        ("Option<&mut Values>", "TryFromSexp"),
    ] {
        let ty: syn::Type = syn::parse_str(ty).unwrap();
        let context = CWrapperContext::builder(syn::parse_quote!(f), syn::parse_quote!(C_f))
            .inputs(syn::parse_quote!(x: #ty))
            .call_expr(quote::quote!(f(x)))
            .with_convert_param("x".into())
            .build();
        let code = context.generate().to_string();
        assert!(code.contains(converter), "{code}");
        assert!(code.contains("NATIVE_BORROW"), "{code}");
        assert!(code.contains("assert_no_aliases"), "{code}");
        assert!(!code.contains("__mx_external_arg"), "{code}");
        assert!(!code.contains("__mx_borrow"), "{code}");
        if converter == "TryFromRoot" {
            assert!(code.contains("__mx_roots_x . rooted"), "{code}");
        }
        // A `&mut` converter must declare its metadata: the preflight skips
        // `None`, the default (checked at compile time).
        assert_eq!(
            code.contains("NATIVE_BORROW . is_some ()"),
            converter == "TryFromSexp",
            "{code}"
        );
    }
}

/// `convert` on a reference the macro already reads from R keeps that
/// reference's own conversion: shared ones through the rooted `TryFromRoot`
/// path, mutable ones through `TryFromSexp` (spelled with the type).
#[test]
fn convert_on_a_native_reference_keeps_its_conversion() {
    let conversion_of_x = |ty: &syn::Type, convert: bool| {
        let mut builder = CWrapperContext::builder(syn::parse_quote!(f), syn::parse_quote!(C_f))
            .inputs(syn::parse_quote!(x: #ty, y: i32))
            .call_expr(quote::quote!(f(x, y)));
        if convert {
            builder = builder.with_convert_param("x".into());
        }
        let code = builder.build().generate().to_string();
        let start = code
            .find("let __mx_roots_x")
            .or_else(|| code.find("let x :"))
            .expect("x is converted");
        let end = start + code[start..].find("let y :").expect("y follows x");
        code[start..end].to_owned()
    };
    for ty in [
        "&str",
        "Option<&str>",
        "&[i32]",
        "Option<&[f64]>",
        "&i32",
        "&mut [i32]",
        "Option<&mut [i32]>",
        "&mut f64",
    ] {
        let ty: syn::Type = syn::parse_str(ty).unwrap();
        let qualified = format!(
            "< {} as :: miniextendr_api :: TryFromSexp > :: try_from_sexp",
            quote::quote!(#ty)
        );
        assert_eq!(
            conversion_of_x(&ty, true).replace(
                &qualified,
                ":: miniextendr_api :: TryFromSexp :: try_from_sexp"
            ),
            conversion_of_x(&ty, false),
            "{}",
            quote::quote!(#ty)
        );
    }
}
