use proc_macro::TokenStream;

struct ExtendrFunction {
    // function_name: Ident
    vis: syn::Visibility,
    // TODO: implement the extern "C" passthrough
    abi: Option<syn::Abi>,
    ident: syn::Ident,
    generics: syn::Generics,
    inputs: syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
    output: syn::ReturnType,
}

impl syn::parse::Parse for ExtendrFunction {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let itemfn: syn::ItemFn = input.parse()?;
        let signature: syn::Signature = itemfn.sig;

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
    let original_item = item.clone();
    let original_item = syn::parse_macro_input!(original_item as syn::Item);
    let extendr_function = syn::parse_macro_input!(item as ExtendrFunction);
    // TODO: implement pass-through of abi extern "C"
    let ExtendrFunction {
        vis,
        abi: _,
        ident,
        generics,
        inputs,
        output,
    } = extendr_function;
    let rust_ident = ident.clone();
    let ident = quote::format_ident!("C_{ident}");

    let rust_inputs: syn::punctuated::Punctuated<syn::Ident, _> = inputs
        .clone()
        .into_pairs()
        .flat_map(|pair| {
            let punct = pair.punct().cloned();
            let mut arg = pair.into_value();
            let arg = if let syn::FnArg::Typed(ref mut pat_type) = arg {
                if let syn::Pat::Ident(pat_ident) = pat_type.pat.as_mut() {
                    pat_ident.ident.clone()
                } else {
                    return None;
                }
            } else {
                return None;
            };
            Some(match punct {
                Some(c) => syn::punctuated::Pair::Punctuated(arg, c),
                None => syn::punctuated::Pair::End(arg),
            })
        })
        .collect();

    let wrapper_inputs: syn::punctuated::Punctuated<syn::FnArg, _> = inputs
        .clone()
        .into_pairs()
        .map(|pair| {
            let punct = pair.punct().cloned();
            let mut arg = pair.into_value();
            if let syn::FnArg::Typed(ref mut pt) = arg {
                *pt.ty.as_mut() = syn::parse_quote!(SEXP);
                if let syn::Pat::Ident(ident) = pt.pat.as_mut() {
                    ident.mutability = None;
                }
            }
            match punct {
                Some(c) => syn::punctuated::Pair::Punctuated(arg, c),
                None => syn::punctuated::Pair::End(arg),
            }
        })
        .collect();

    let input_names: Vec<_> = inputs
        .pairs()
        .filter_map(|pair| match pair.value() {
            syn::FnArg::Typed(pat_type) => {
                let is_unit_type = if let syn::Type::Tuple(type_tuple) = &*pat_type.ty {
                    type_tuple.elems.is_empty()
                } else {
                    false
                };
                match &*pat_type.pat {
                    syn::Pat::Ident(p) => {
                        let ident = p.ident.clone();
                        //TODO / FIXME: implement mutability here!
                        if is_unit_type {
                            Some(quote::quote! {let #ident = (); })
                        } else {
                            Some(quote::quote! {let #ident = *DATAPTR_RO(#ident).cast(); })
                        }
                    }
                    _ => None,
                }
            }
            syn::FnArg::Receiver(_) => todo!(),
        })
        .collect();

    let return_statement = match output {
        syn::ReturnType::Default => quote::quote! {unsafe { R_NilValue }},
        syn::ReturnType::Type(_rarrow, box_type) => {
            if let syn::Type::Path(type_path) = &*box_type {
                let last_segment = type_path.path.segments.last().unwrap();
                let is_result = last_segment.ident == "Result";
                if is_result {
                    quote::quote! {
                        // TODO: set debug flag?
                        // let _ = dbg!(result);
                        Rf_ScalarInteger(result.unwrap())
                    }
                } else {
                    quote::quote! {
                        Rf_ScalarInteger(result)
                    }
                }
            } else {
                todo!()
            }
        }
    };

    TokenStream::from(quote::quote! {
        #original_item

        #[unsafe(no_mangle)]
        #vis unsafe extern "C" fn #ident #generics(#wrapper_inputs) -> SEXP {
            let old = std::panic::take_hook();
            std::panic::set_hook(Box::new(|_| {}));
            let result = with_r_unwind(move || unsafe {
                #[allow(unused_imports)]
                // TODO: these borrows ought to be used based on the mutability requirements...
                use std::borrow::{Borrow, BorrowMut};
                #(#input_names)*
                // FIXME: shouldn't this borrow?
                // dbg!(#rust_inputs);
                let result = #rust_ident(#rust_inputs);
                #return_statement
            });
            std::panic::set_hook(old);
            result
        }
    })
}
