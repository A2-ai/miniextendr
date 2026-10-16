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

/// Read MatchArg constants without resolving a declaration-local type path in
/// the impl module. Like `setup`, this infers from a method item and never calls
/// it. Borrowing the callable keeps this const function free of generic drops.
pub(crate) fn match_arg_values(
    sig: &syn::Signature,
    parameter: &str,
    leaf: &syn::Type,
    callable: &TokenStream,
) -> TokenStream {
    use syn::visit_mut::VisitMut;
    struct ReplaceLeaf<'a>(&'a syn::Type);
    impl VisitMut for ReplaceLeaf<'_> {
        fn visit_type_mut(&mut self, ty: &mut syn::Type) {
            if ty == self.0 {
                *ty = syn::parse_quote!(__MxChoice);
            } else {
                syn::visit_mut::visit_type_mut(self, ty);
            }
        }
    }
    let mut generics = Vec::new();
    let args: Vec<syn::Type> = sig
        .inputs
        .iter()
        .enumerate()
        .map(|(index, arg)| {
            if let syn::FnArg::Typed(arg) = arg
                && let syn::Pat::Ident(name) = arg.pat.as_ref()
                && crate::naming::ident_name(&name.ident) == parameter
            {
                let mut ty = *arg.ty.clone();
                ReplaceLeaf(leaf).visit_type_mut(&mut ty);
                ty
            } else {
                let name = format_ident!("__MxArg{index}");
                generics.push(name.clone());
                syn::parse_quote!(#name)
            }
        })
        .collect();
    quote! {{
        const fn __mx_choices<__MxChoice, __MxOutput, __MxFn, #(#generics),*>(
            _: &__MxFn,
        ) -> (&'static [&'static str], &'static [(&'static str, &'static str)])
        where
            __MxChoice: ::miniextendr_api::match_arg::MatchArg,
            __MxFn: ::core::ops::FnOnce(#(#args),*) -> __MxOutput,
        {
            (<__MxChoice as ::miniextendr_api::match_arg::MatchArg>::CHOICES,
             <__MxChoice as ::miniextendr_api::match_arg::MatchArg>::ALIASES)
        }
        __mx_choices(&(#callable))
    }}
}
