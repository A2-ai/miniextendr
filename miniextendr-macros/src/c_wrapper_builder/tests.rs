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

    // -> Option<i32> -> OptionIntoR (None becomes NA_integer_).
    let option_ty: syn::ReturnType = syn::parse_quote!(-> Option<i32>);
    assert!(matches!(
        detect_return_handling(&option_ty),
        ReturnHandling::OptionIntoR
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
fn scalar_options_use_the_whole_option_conversion() {
    for inner in [
        "i32",
        "f64",
        "bool",
        "String",
        "Rboolean",
        "RLogical",
        "Rcomplex",
        "i8",
        "i16",
        "u16",
        "u32",
        "f32",
        "i64",
        "u64",
        "isize",
        "usize",
        "PathBuf",
        "OsString",
        "&str",
        "&'static str",
        "std::string::String",
        "std::path::PathBuf",
        "std::ffi::OsString",
        "miniextendr_api::Rcomplex",
        "std::primitive::i32",
    ] {
        let output = syn::parse_str::<syn::ReturnType>(&format!("-> Option<{inner}>"))
            .expect("valid scalar return type");
        assert!(
            matches!(detect_return_handling(&output), ReturnHandling::OptionIntoR),
            "{inner}"
        );
        assert!(
            matches!(
                detect_return_handling_standalone_fn(&output),
                ReturnHandling::OptionIntoR
            ),
            "{inner}"
        );
    }
}

#[test]
fn other_options_keep_their_existing_return_strategies() {
    for inner in [
        "u8",
        "Custom",
        "Self::Item",
        "<Self as Trait>::Item",
        "Vec<i32>",
        "&i32",
        "&mut str",
        "String<T>",
    ] {
        let output = syn::parse_str::<syn::ReturnType>(&format!("-> Option<{inner}>"))
            .expect("valid non-scalar return type");
        assert!(
            matches!(
                detect_return_handling(&output),
                ReturnHandling::OptionIntoRUnwrap
            ),
            "{inner}"
        );
    }
    let sexp: syn::ReturnType = syn::parse_quote!(-> Option<miniextendr_api::SEXP>);
    assert!(matches!(
        detect_return_handling(&sexp),
        ReturnHandling::OptionSexp
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

/// `ExternalPtr<T>` arguments are borrowed after every conversion and
/// post-conversion statement (#2092), on the main thread in both wrappers;
/// a consumed handle and a marked parameter are not.
#[test]
fn argument_borrows_follow_conversion_on_both_threads() {
    let sig: syn::ItemFn = syn::parse_quote!(
        fn pair(
            mut a: ExternalPtr<Counter>,
            b: Option<ExternalPtr<Counter>>,
            c: ExternalPtr<Counter>,
        ) {
        }
    );
    let ctx = |strategy: ThreadStrategy, consumed: Vec<String>| {
        CWrapperContext::builder(sig.sig.ident.clone(), syn::parse_quote!(C_pair))
            .r_wrapper_const(syn::parse_quote!(R_WRAPPER_pair))
            .call_expr(quote::quote!(pair(a, b, c)))
            .inputs(sig.sig.inputs.clone())
            .thread_strategy(strategy)
            .post_conversion(vec![quote::quote!(post_conversion_marker();)])
            .consumed_handles(consumed)
            .preserve_param_names()
            .build()
            .generate()
            .to_string()
    };

    let main = ctx(ThreadStrategy::MainThread, Vec::new());
    let capture = main
        .find("let __mx_handle_sexp_a = a")
        .expect("captured slot");
    let conversion = main.find("let a :").expect("conversion");
    let post = main
        .find("post_conversion_marker")
        .expect("post conversion");
    let exclusive = main
        .find("ArgumentCell :: exclusive")
        .expect("exclusive borrow");
    let call = main.find("pair (a , b , c)").expect("call");
    assert!(
        capture < conversion && post < exclusive && exclusive < call,
        "{main}"
    );
    assert_eq!(main.matches("ArgumentCell :: shared").count(), 2, "{main}");
    assert!(
        main.contains(":: core :: option :: Option :: as_ref (& b)"),
        "{main}"
    );

    let worker = ctx(ThreadStrategy::WorkerThread, Vec::new());
    let post = worker
        .find("post_conversion_marker")
        .expect("post conversion");
    let exclusive = worker
        .find("ArgumentCell :: exclusive")
        .expect("exclusive borrow");
    let dispatch = worker.find("run_on_worker").expect("the worker dispatch");
    assert!(post < exclusive && exclusive < dispatch, "{worker}");

    let consumed = ctx(
        ThreadStrategy::MainThread,
        vec!["a".into(), "b".into(), "c".into()],
    );
    assert!(!consumed.contains("ArgumentCell"), "{consumed}");
}
