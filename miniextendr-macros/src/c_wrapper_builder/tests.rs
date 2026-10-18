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
    for handling in [ReturnHandling::OptionUnit, ReturnHandling::OptionSexp] {
        let context =
            CWrapperContext::builder(syn::parse_quote!(rust_name), syn::parse_quote!(C_test))
                .r_name("visible.name".to_owned())
                .r_wrapper_const(syn::parse_quote!(R_WRAPPER_test))
                .call_expr(quote::quote!(rust_name()))
                .return_handling(handling)
                .build();
        let call = quote::quote!(rust_name());
        let main = context.generate_return_handling(&call).to_string();
        let (_, worker) = context.generate_worker_return_handling(&call);
        for tokens in [main, worker.to_string()] {
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
    let main = lookup.generate_return_handling(&call).to_string();
    let (_, worker) = lookup.generate_worker_return_handling(&call);
    for tokens in [main, worker.to_string()] {
        assert!(
            tokens.contains("map (:: miniextendr_api :: externalptr :: ExternalPtr :: new)"),
            "{tokens}"
        );
        assert!(!tokens.contains("NONE_ERR"), "{tokens}");
    }

    // Self-handle returns belong to instance methods, which cannot run on workers.
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

/// `OptionClassPayload` gives NULL for `None` on both threads, never the
/// absence error.
#[test]
fn optional_class_payload_none_is_null() {
    let context = CWrapperContext::builder(syn::parse_quote!(find), syn::parse_quote!(C_test))
        .r_wrapper_const(syn::parse_quote!(R_WRAPPER_test))
        .output(syn::parse_quote!(-> Option<Board>))
        .call_expr(quote::quote!(find()))
        .return_handling(ReturnHandling::OptionClassPayload)
        .build();
    let call = quote::quote!(find());
    let main = context.generate_return_handling(&call).to_string();
    let (_, worker) = context.generate_worker_return_handling(&call);
    for tokens in [main, worker.to_string()] {
        assert!(tokens.contains("SEXP :: nil ()"), "{tokens}");
        assert!(tokens.contains("into_sexp"), "{tokens}");
        assert!(!tokens.contains("NONE_ERR"), "{tokens}");
    }
}
