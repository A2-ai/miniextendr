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
fn mut_slice_family_detection() {
    // &mut [T] -> flagged
    let mut_slice: syn::Type = syn::parse_quote!(&mut [i32]);
    assert!(is_mut_slice_family(&mut_slice));

    // Option<&mut [T]> -> flagged
    let opt_mut_slice: syn::Type = syn::parse_quote!(Option<&mut [f64]>);
    assert!(is_mut_slice_family(&opt_mut_slice));

    // &[T] (shared) -> not flagged
    let shared_slice: syn::Type = syn::parse_quote!(&[i32]);
    assert!(!is_mut_slice_family(&shared_slice));

    // Option<&[T]> (shared) -> not flagged
    let opt_shared: syn::Type = syn::parse_quote!(Option<&[i32]>);
    assert!(!is_mut_slice_family(&opt_shared));

    // &mut T (scalar reference) -> not flagged (only slices)
    let mut_scalar: syn::Type = syn::parse_quote!(&mut i32);
    assert!(!is_mut_slice_family(&mut_scalar));

    // Vec<T> -> not flagged
    let vec_ty: syn::Type = syn::parse_quote!(Vec<i32>);
    assert!(!is_mut_slice_family(&vec_ty));

    // Option<i32> -> not flagged
    let opt_scalar: syn::Type = syn::parse_quote!(Option<i32>);
    assert!(!is_mut_slice_family(&opt_scalar));
}

#[test]
fn alias_guard_emitted_only_for_multiple_mut_slices() {
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

    // Two &mut [T] params -> a pairwise debug_assert is emitted.
    let ctx = ctx_for(syn::parse_quote!(
        fn alias_probe(a: &mut [i32], b: &mut [i32]) {}
    ));
    let guard = ctx.build_alias_guard(&sexp_idents).to_string();
    assert!(guard.contains("debug_assert"), "guard = {guard}");
    assert!(guard.contains("arg_0"), "guard = {guard}");
    assert!(guard.contains("arg_1"), "guard = {guard}");

    // Two mut slices where one is Option-wrapped -> still guarded.
    let ctx = ctx_for(syn::parse_quote!(
        fn opt_mut(a: &mut [i32], b: Option<&mut [i32]>) {}
    ));
    assert!(!ctx.build_alias_guard(&sexp_idents).is_empty());

    // One &mut [T] + one &[T]: only one mutable slice -> no guard.
    let ctx = ctx_for(syn::parse_quote!(
        fn one_mut(a: &mut [i32], b: &[i32]) {}
    ));
    assert!(ctx.build_alias_guard(&sexp_idents).is_empty());

    // Single &mut [T] param -> no guard (nothing to alias with).
    let ctx = ctx_for(syn::parse_quote!(
        fn single(a: &mut [i32]) {}
    ));
    assert!(
        ctx.build_alias_guard(&[syn::parse_quote!(arg_0)])
            .is_empty()
    );
}
