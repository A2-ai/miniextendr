//! Infer inherited argument types from a concrete method item, not copied paths.

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};

pub(crate) fn argument_ident(name: &syn::Ident) -> syn::Ident {
    format_ident!(
        "__mx_type_{}",
        crate::naming::unraw(name),
        span = Span::mixed_site()
    )
}

/// This local generic function only inspects a callable's signature. It never
/// calls it, constructs an argument, or adds conversion bounds to the trait.
pub(crate) fn setup(
    inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
    receiver: bool,
    callable: &TokenStream,
) -> TokenStream {
    let names: Vec<_> = inputs
        .iter()
        .filter_map(|arg| match arg {
            syn::FnArg::Typed(arg) => match arg.pat.as_ref() {
                syn::Pat::Ident(name) => Some(argument_ident(&name.ident)),
                _ => None,
            },
            syn::FnArg::Receiver(_) => None,
        })
        .collect();
    if names.is_empty() {
        return TokenStream::new();
    }
    let args: Vec<_> = (0..names.len() + usize::from(receiver))
        .map(|i| format_ident!("__MxArg{i}", span = Span::mixed_site()))
        .collect();
    let output = syn::Ident::new("__MxOutput", Span::mixed_site());
    let helper = syn::Ident::new("__mx_argument_types", Span::mixed_site());
    let receiver_slot = receiver.then(|| quote!(_,));
    quote! {
        let (#receiver_slot #(#names,)*) = {
            fn #helper<#output, #(#args),*>(
                _: impl ::core::ops::FnOnce(#(#args),*) -> #output,
            ) -> (#(::miniextendr_api::trait_abi::signature::Argument<#args>,)*) {
                (#(::miniextendr_api::trait_abi::signature::Argument::<#args>::NEW,)*)
            }
            #helper(#callable)
        };
    }
}

/// Foreign-ABI methods are not FnOnce items. A never-called closure
/// presents their argument types to the same inference helper.
pub(crate) fn callable(sig: &syn::Signature, item: TokenStream) -> TokenStream {
    if sig.abi.is_none() {
        return item;
    }
    let args: Vec<_> = (0..sig.inputs.len())
        .map(|i| format_ident!("__mx_value{i}", span = Span::mixed_site()))
        .collect();
    let call = quote! { #item(#(#args),*) };
    quote! { |#(#args),*| #call }
}
