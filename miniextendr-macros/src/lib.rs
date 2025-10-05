use proc_macro::TokenStream;
use quote::format_ident;
use syn::{
    FnArg, ItemFn, Pat, Signature, parse_macro_input, parse_quote,
    punctuated::{Pair, Punctuated},
};

struct ExtendrFunction {
    // function_name: Ident
    vis: syn::Visibility,
    abi: Option<syn::Abi>,
    ident: syn::Ident,
    generics: syn::Generics,
    inputs: syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
    output: syn::ReturnType,
}

impl syn::parse::Parse for ExtendrFunction {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let itemfn: ItemFn = input.parse()?;
        let signature: Signature = itemfn.sig;

        Ok(Self {
            vis: itemfn.vis,
            abi: signature.abi,
            ident: signature.ident,
            generics: signature.generics,
            inputs: signature.inputs,
            output: signature.output,
        })
    }
}

#[proc_macro_attribute]
pub fn miniextendr(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // let item: TokenStream = item.into();
    let original_item = item.clone();
    let original_item = parse_macro_input!(original_item as syn::Item);
    let extendr_function = parse_macro_input!(item as ExtendrFunction);
    // let extendr_function_vis = extendr_function.vis;
    // TODO: implement pass-through of abi extern "C"
    let ExtendrFunction {
        vis,
        abi: _,
        ident,
        generics,
        inputs,
        output,
    } = extendr_function;
    let ident = format_ident!("C_{ident}");

    let wrapper_inputs: Punctuated<syn::FnArg, _> = inputs
        .clone()
        .into_pairs()
        .map(|pair| {
            let punct = pair.punct().cloned();
            let mut arg = pair.into_value();
            if let syn::FnArg::Typed(ref mut pt) = arg {
                *pt.ty.as_mut() = parse_quote!(SEXP);
            }
            match punct {
                Some(c) => Pair::Punctuated(arg, c),
                None => Pair::End(arg),
            }
        })
        .collect();

    let input_names: Vec<syn::Ident> = inputs
        .pairs()
        .filter_map(|pair| match pair.value() {
            FnArg::Typed(pat_type) => match &*pat_type.pat {
                Pat::Ident(p) => Some(p.ident.clone()),
                _ => None,
            },
            FnArg::Receiver(_) => todo!(),
        })
        .collect();

    TokenStream::from(quote::quote! {
        #original_item

        #[unsafe(no_mangle)]
        #vis unsafe extern "C" fn #ident #generics(#wrapper_inputs) -> SEXP {
            with_r_unwind(move || unsafe {
                #[allow(unused_imports)]
                use std::borrow::{Borrow, BorrowMut};
                #(
                    let #input_names = *DATAPTR_RO(#input_names).cast();
                );*

                let result = add(left, right);
                Rf_ScalarInteger(result)
                // unsafe { R_NilValue }
            })
        }
    })
}
