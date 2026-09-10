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

#[test]
fn worker_inputs_unwind_before_dispatch_and_rng_cleanup_precedes_r_resume() {
    let sig: syn::ItemFn = syn::parse_quote!(
        fn probe(input: OwnedInput) {}
    );
    let ctx = CWrapperContext::builder(sig.sig.ident.clone(), syn::parse_quote!(C_probe))
        .r_wrapper_const(syn::parse_quote!(R_WRAPPER_probe))
        .call_expr(quote::quote!(probe(input)))
        .inputs(sig.sig.inputs)
        .thread_strategy(ThreadStrategy::WorkerThread)
        .check_interrupt()
        .rng()
        .build();
    let wrapper = ctx.generate_worker_thread_wrapper().to_string();
    let ordered = [
        "catch_unwind",
        "InputConversionScope :: new",
        "R_CheckUserInterrupt",
        "try_from_sexp",
        "drop (__miniextendr_input_scope)",
        "run_on_worker",
        "PutRNGstate",
        "resume_input_error",
    ];
    let mut offset = 0;
    for needle in ordered {
        offset += wrapper[offset..]
            .find(needle)
            .unwrap_or_else(|| panic!("missing or misplaced {needle}: {wrapper}"))
            + needle.len();
    }
}
