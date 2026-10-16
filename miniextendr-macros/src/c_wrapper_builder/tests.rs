use super::*;

#[test]
fn return_handling_detection() {
    // Default (no return type) -> Unit
    assert!(matches!(
        detect_return_handling(&syn::ReturnType::Default),
        ReturnHandling::Unit
    ));

    // -> () -> Unit
    let unit_ty: syn::ReturnType = syn::parse_quote!(-> ());
    assert!(matches!(
        detect_return_handling(&unit_ty),
        ReturnHandling::Unit
    ));

    // -> i32 -> IntoR
    let i32_ty: syn::ReturnType = syn::parse_quote!(-> i32);
    assert!(matches!(
        detect_return_handling(&i32_ty),
        ReturnHandling::IntoR
    ));

    // -> Self -> ExternalPtr
    let self_ty: syn::ReturnType = syn::parse_quote!(-> Self);
    assert!(matches!(
        detect_return_handling(&self_ty),
        ReturnHandling::ExternalPtr
    ));

    // -> Option<i32> -> OptionIntoRUnwrap (default: unwrap + error on None)
    // Use ReturnHandling::OptionIntoR explicitly when Option<T>: IntoR is known.
    let option_ty: syn::ReturnType = syn::parse_quote!(-> Option<i32>);
    assert!(matches!(
        detect_return_handling(&option_ty),
        ReturnHandling::OptionIntoRUnwrap
    ));

    // -> Option<()> -> OptionUnit
    let option_unit_ty: syn::ReturnType = syn::parse_quote!(-> Option<()>);
    assert!(matches!(
        detect_return_handling(&option_unit_ty),
        ReturnHandling::OptionUnit
    ));

    // -> Result<i32, E> -> ResultIntoR
    let result_ty: syn::ReturnType = syn::parse_quote!(-> Result<i32, E>);
    assert!(matches!(
        detect_return_handling(&result_ty),
        ReturnHandling::ResultIntoR
    ));

    // -> Result<(), E> -> ResultUnit
    let result_unit_ty: syn::ReturnType = syn::parse_quote!(-> Result<(), E>);
    assert!(matches!(
        detect_return_handling(&result_unit_ty),
        ReturnHandling::ResultUnit
    ));

    // -> Result<i32, ()> -> ResultNullOnErr (unit error is a sentinel, always returns NULL)
    let result_null_on_err_ty: syn::ReturnType = syn::parse_quote!(-> Result<i32, ()>);
    assert!(matches!(
        detect_return_handling(&result_null_on_err_ty),
        ReturnHandling::ResultNullOnErr
    ));

    // -> Result<(), ()> -> ResultNullOnErr (both unit: returns NULL regardless)
    let result_unit_unit_ty: syn::ReturnType = syn::parse_quote!(-> Result<(), ()>);
    assert!(matches!(
        detect_return_handling(&result_unit_unit_ty),
        ReturnHandling::ResultNullOnErr
    ));

    // -> Result<Self, E> -> ResultExternalPtr (audit A4: fallible constructor-shaped
    // methods like `from_r` wrap `Ok(Self)` in an ExternalPtr, not `IntoR`).
    let result_self_ty: syn::ReturnType = syn::parse_quote!(-> Result<Self, E>);
    assert!(matches!(
        detect_return_handling(&result_self_ty),
        ReturnHandling::ResultExternalPtr
    ));

    // -> Option<Self> -> OptionExternalPtr (#1164: lookup-shaped fallible constructors
    // like `try_find` wrap `Some(Self)` in an ExternalPtr, not `IntoR` — symmetric with
    // the Result<Self, E> case above).
    let option_self_ty: syn::ReturnType = syn::parse_quote!(-> Option<Self>);
    assert!(matches!(
        detect_return_handling(&option_self_ty),
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

/// A `no_na` parameter's converted value is checked in both wrappers; in the
/// worker wrapper the check runs on the main thread, before `run_on_worker`.
#[test]
fn no_na_check_renders_in_main_thread_and_worker_wrappers() {
    let sig: syn::ItemFn = syn::parse_quote!(
        fn dose(x: AsNumeric) -> f64 {}
    );
    let ctx = |strategy: ThreadStrategy| {
        CWrapperContext::builder(sig.sig.ident.clone(), syn::parse_quote!(C_dose))
            .r_wrapper_const(syn::parse_quote!(R_WRAPPER_dose))
            .call_expr(quote::quote!(dose(x)))
            .inputs(sig.sig.inputs.clone())
            .output(sig.sig.output.clone())
            .thread_strategy(strategy)
            .no_na("x".to_string(), None)
            .build()
            .generate()
            .to_string()
    };
    let check = ":: miniextendr_api :: TryFromSexp :: __mx_has_na (& x)";

    let main = ctx(ThreadStrategy::MainThread);
    assert!(main.contains(check), "{main}");
    assert!(main.contains("\"'x' must not be NA\""), "{main}");

    let worker = ctx(ThreadStrategy::WorkerThread);
    let at = worker.find(check).expect("the check in the worker wrapper");
    let dispatch = worker.find("run_on_worker").expect("the worker dispatch");
    assert!(at < dispatch, "{worker}");
}

#[test]
fn every_none_error_path_uses_the_r_name_on_both_threads() {
    for handling in [
        ReturnHandling::OptionUnit,
        ReturnHandling::OptionSexp,
        ReturnHandling::OptionIntoRUnwrap,
        ReturnHandling::OptionExternalPtr,
        ReturnHandling::SelfHandleOption,
    ] {
        let context =
            CWrapperContext::builder(syn::parse_quote!(rust_name), syn::parse_quote!(C_test))
                .r_name("visible.name".to_owned())
                .r_wrapper_const(syn::parse_quote!(R_WRAPPER_test))
                .call_expr(quote::quote!(rust_name()))
                .return_handling(handling)
                .build();
        let call = quote::quote!(rust_name());
        let main = context.generate_return_handling(&call).to_string();
        let mut outputs = vec![main];
        // Self-handle returns belong to instance methods, which cannot run on workers.
        if !matches!(context.return_handling, ReturnHandling::SelfHandleOption) {
            let (_, worker) = context.generate_worker_return_handling(&call);
            outputs.push(worker.to_string());
        }
        for tokens in outputs {
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
        .thread_strategy(ThreadStrategy::MainThread)
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
            .thread_strategy(ThreadStrategy::WorkerThread)
            .with_convert_param("x".into())
            .build();
        let code = context.generate().to_string();
        assert!(code.contains(converter), "{code}");
        assert!(code.contains("NATIVE_BORROW"), "{code}");
        assert!(code.contains("assert_no_aliases"), "{code}");
        assert!(!code.contains("__mx_external_arg"), "{code}");
        assert!(!code.contains("__mx_borrow"), "{code}");
        assert!(!code.contains("run_on_worker"), "{code}");
        if converter == "TryFromRoot" {
            assert!(code.contains("__mx_roots_x . rooted"), "{code}");
        }
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
            .call_expr(quote::quote!(f(x, y)))
            .thread_strategy(ThreadStrategy::MainThread);
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
