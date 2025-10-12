//!
//!
//!
//!

struct ExtendrFunction {
    pub vis: syn::Visibility,
    pub abi: Option<syn::Abi>,
    pub ident: syn::Ident,
    pub generics: syn::Generics,
    pub inputs: syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
    pub output: syn::ReturnType,
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
pub fn miniextendr(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut item = syn::parse_macro_input!(item as syn::ItemFn);

    let dots = if let Some(variadic) = item.sig.variadic {
        if let Some((pat, _)) = variadic.pat {
            if let syn::Pat::Ident(pat_ident) = pat.as_ref() {
                Some(pat_ident.ident.clone())
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };
    item.sig.variadic = None;
    // FIXME: ... is being replaced by () which gets replaced by SEXP...
    // do something else...
    if let Some(ident_dots) = dots {
        item.sig.inputs.push(syn::parse_quote!(#ident_dots: ()));
    }
    let original_item = item.clone();
    use quote::ToTokens;
    let item = item.into_token_stream();
    let extendr_function = syn::parse2(item).unwrap();
    // TODO: implement pass-through of abi extern "C"
    let ExtendrFunction {
        vis,
        abi,
        ident,
        generics,
        inputs,
        output,
    } = extendr_function;
    use syn::spanned::Spanned;
    let num_args = syn::LitInt::new(&inputs.len().to_string(), inputs.span());

    let rust_ident = &ident;
    let call_method_def = quote::format_ident!("call_method_def_{rust_ident}");
    let c_ident = if abi.is_none() {
        &quote::format_ident!("C_{rust_ident}")
    } else {
        rust_ident
    };

    let c_ident_name = syn::LitCStr::new(
        std::ffi::CString::new(c_ident.to_string())
            .expect("couldn't crate a C-string for the C wrapper name")
            .as_c_str(),
        ident.span(),
    );
    let mut func_ptr_def: syn::punctuated::Punctuated<syn::Ident, syn::Token![,]> =
        syn::punctuated::Punctuated::new();
    for _ in 0..inputs.len() {
        func_ptr_def.push(syn::parse_quote!(SEXP));
    }

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
    // dbg!(&rust_inputs);

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
    // dbg!(&wrapper_inputs);

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

    let return_statement = match &output {
        syn::ReturnType::Default => quote::quote! {unsafe { R_NilValue }},
        syn::ReturnType::Type(_rarrow, box_type) => {
            if let syn::Type::Path(type_path) = box_type.as_ref() {
                let last_segment = type_path.path.segments.last().unwrap();
                let is_result = last_segment.ident == "Result";
                if is_result {
                    // TODO: implement other Result<> -> SEXP interpretations here``
                    quote::quote! {
                        // TODO: set debug flag?
                        // let _ = dbg!(result);
                        Rf_ScalarInteger(result.unwrap())
                    }
                } else {
                    // TODO: type conversion for non-Result returns
                    quote::quote! {
                        Rf_ScalarInteger(result)
                    }
                }
            } else {
                // interpret () -> R_NilValue (R's NULL)
                quote::quote! {
                    R_NilValue
                }
            }
        }
    };

    let c_wrapper = if abi.is_some() {
        proc_macro2::TokenStream::new()
    } else {
        quote::quote! {
            // TODO: add the method it is wrapping as doc comment
            #[doc = "C wrapper method for TODO"]
            #[unsafe(no_mangle)]
            #vis unsafe extern "C" fn #c_ident #generics(#wrapper_inputs) -> SEXP {
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
                // FIXME: in case of a panic, the panic hook is never reset.
                std::panic::set_hook(old);
                result
            }
        }
    };

    let r_wrapper_args: Vec<_> = inputs
        .into_iter()
        .map(|x| {
            let syn::FnArg::Typed(pat_type) = &x else {
                unreachable!()
            };
            let syn::PatType {
                attrs: _,
                pat,
                colon_token: _,
                ty: _,
            } = &pat_type;
            let syn::Pat::Ident(pat_ident) = pat.as_ref() else {
                unreachable!()
            };
            pat_ident.ident.clone()
        })
        .collect();

    // region: R wrappers

    // mark return type invisible if (), Option<()> or Result<()>, or Result<(), _>
    let is_invisible_return_type = match &output {
        syn::ReturnType::Default => true,
        syn::ReturnType::Type(_, ty) => match ty.as_ref() {
            syn::Type::Tuple(t) if t.elems.is_empty() => true, // ()
            syn::Type::Path(p) => {
                let seg = &p.path.segments.last().unwrap().ident.to_string();
                match seg.as_str() {
                    "Result" | "Option" => {
                        if let syn::PathArguments::AngleBracketed(args) =
                            &p.path.segments.last().unwrap().arguments
                        {
                            let mut inner = args.args.iter();
                            match inner.next() {
                                Some(syn::GenericArgument::Type(syn::Type::Tuple(t)))
                                    if t.elems.is_empty() =>
                                {
                                    true
                                }
                                Some(syn::GenericArgument::Type(syn::Type::Array(a))) => {
                                    matches!(a.len, syn::Expr::Lit(ref l)
                                        if matches!(&l.lit, syn::Lit::Int(i) if i.base10_parse::<usize>().ok() == Some(0)))
                                }
                                _ => false,
                            }
                        } else {
                            true // Result<()> without explicit args
                        }
                    }
                    _ => false,
                }
            }
            syn::Type::Array(a) => {
                matches!(a.len, syn::Expr::Lit(ref l)
                    if matches!(&l.lit, syn::Lit::Int(i) if i.base10_parse::<usize>().ok() == Some(0)))
            }
            _ => false,
        },
    };

    let r_wrapper_return = if !is_invisible_return_type {
        quote::quote! {.Call(#c_ident #(,#r_wrapper_args)*)}
    } else {
        quote::quote! {invisible(.Call(#c_ident #(,#r_wrapper_args)*))}
    };
    let r_wrapper = quote::quote! {
        #rust_ident <- function(#(#r_wrapper_args),*) {

            #r_wrapper_return
        }
    };
    let r_wrapper_string = r_wrapper.to_string();
    let r_wrapper_cstring = std::ffi::CString::new(r_wrapper_string).unwrap();
    let r_wrapper_cstr = r_wrapper_cstring.as_c_str();
    let r_wrapper_cstr = syn::LitCStr::new(r_wrapper_cstr, r_wrapper.span());

    let r_wrapper_generator = quote::format_ident!("r_wrapper_{rust_ident}");
    // endregion

    quote::quote! {
        #original_item

        #c_wrapper

        const #r_wrapper_generator: &'static std::ffi::CStr = #r_wrapper_cstr;

        #[doc(hidden)]
        #[inline(always)]
        const fn #call_method_def() -> R_CallMethodDef {
            unsafe {
                R_CallMethodDef {
                    name: #c_ident_name.as_ptr(),
                    fun: Some(std::mem::transmute::<unsafe extern "C" fn(#func_ptr_def) -> SEXP, unsafe extern "C" fn(...) -> SEXP>(#c_ident)),
                    numArgs: #num_args,
                }
            }
        }
    }
    .into()
}

#[proc_macro_attribute]
pub fn r_tokens(
    _attr: proc_macro::TokenStream,
    _item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    proc_macro::TokenStream::new()
}

struct ExtendrModuleFunction {
    pub _abi: Option<syn::Abi>,
    _fn_token: syn::Token![fn],
    pub ident: syn::Ident,
}

impl syn::parse::Parse for ExtendrModuleFunction {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _abi = if input.peek(syn::Token![extern]) {
            Some(input.parse()?)
        } else {
            None
        };
        Ok(Self {
            _abi,
            _fn_token: input.parse()?,
            ident: input.parse()?,
        })
    }
}

struct ExtendrModuleStruct {
    _struct_token: syn::Token![struct],
    pub ident: syn::Ident,
}

impl syn::parse::Parse for ExtendrModuleStruct {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _struct_token: input.parse()?,
            ident: input.parse()?,
        })
    }
}

struct ExtendrModuleName {
    _mod_token: syn::Token![mod],
    pub ident: syn::Ident,
}

impl syn::parse::Parse for ExtendrModuleName {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _mod_token: input.parse()?,
            ident: input.parse()?,
        })
    }
}

struct ExtendrModuleUse {
    _use_token: syn::Token![use],
    pub use_name: syn::UseName,
}

impl syn::parse::Parse for ExtendrModuleUse {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        use syn::spanned::Spanned;
        let _use_token = input.parse()?;
        let use_name: syn::UseTree = input.parse()?;
        // dbg!(&use_name);
        let use_name = match use_name {
            syn::UseTree::Name(use_name) => use_name,
            // TODO: provide boilerplate error message here.
            syn::UseTree::Rename(use_rename) => {
                return Err(syn::Error::new(
                    use_rename.span(),
                    "it is not possible to rename wrappers in `miniextendr_module`",
                ));
            }
            syn::UseTree::Path(_) |
            syn::UseTree::Glob(_) | 
            syn::UseTree::Group(_) => return Err(syn::Error::new(use_name.span(), "syntax not supported"))
        };
        Ok(Self {
            _use_token,
            use_name,
        })
    }
}

struct ExtendrModule {
    pub extendr_module: ExtendrModuleName,
    pub extendr_use: Vec<ExtendrModuleUse>,
    pub extendr_fn: Vec<ExtendrModuleFunction>,
    pub extendr_struct: Vec<ExtendrModuleStruct>,
}

enum ExtendrItem {
    Module(ExtendrModuleName),
    Use(ExtendrModuleUse),
    Struct(ExtendrModuleStruct),
    Func(ExtendrModuleFunction),
}

impl syn::parse::Parse for ExtendrItem {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let look_ahead = input.lookahead1();
        if look_ahead.peek(syn::Token![mod]) {
            Ok(Self::Module(input.parse()?))
        } else if look_ahead.peek(syn::Token![use]) {
            Ok(Self::Use(input.parse()?))
        } else if look_ahead.peek(syn::Token![struct]) {
            Ok(Self::Struct(input.parse()?))
        } else if look_ahead.peek(syn::Token![fn]) || look_ahead.peek(syn::Token![extern]) {
            Ok(Self::Func(input.parse()?))
        } else {
            Err(look_ahead.error())
        }
    }
}

impl syn::parse::Parse for ExtendrModule {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let items: syn::punctuated::Punctuated<ExtendrItem, syn::Token![;]> =
            syn::punctuated::Punctuated::parse_terminated_with(&input, ExtendrItem::parse)?;

        let mut name = None;
        let mut uses = Vec::new();
        let mut funs = Vec::new();
        let mut structs = Vec::new();

        for it in items {
            match it {
                ExtendrItem::Module(m) => {
                    if name.is_some() {
                        return Err(syn::Error::new(m._mod_token.span, "duplicate `mod <name>`"));
                    }
                    name = Some(m);
                }
                ExtendrItem::Use(u) => uses.push(u),
                ExtendrItem::Struct(s) => structs.push(s),
                ExtendrItem::Func(f) => funs.push(f),
            }
        }

        let extendr_module =
            name.ok_or_else(|| syn::Error::new(input.span(), "missing `mod <name>`"))?;

        Ok(Self {
            extendr_module,
            extendr_use: uses,
            extendr_fn: funs,
            extendr_struct: structs,
        })
    }
}

#[proc_macro]
pub fn miniextendr_module(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let extendr_module = syn::parse_macro_input!(item as ExtendrModule);

    let module = extendr_module.extendr_module.ident;
    let module_entrypoint_ident = quote::format_ident!("R_init_{module}_miniextendr");
    let call_entries: Vec<syn::Expr> = extendr_module
        .extendr_fn
        .iter()
        .map(|x| {
            //TODO: put this in ExtendrFunction impl
            let rust_ident = &x.ident;
            let call_method_def = quote::format_ident!("call_method_def_{rust_ident}");
            syn::parse_quote!(#call_method_def())
        })
        .collect();
    let call_entries_len = call_entries.len();

    // call the R_init from all the submodules (given by `use`)
    let use_other_modules = extendr_module
        .extendr_use
        .iter()
        .map(|x| {
            let use_module_ident = &x.use_name.ident;
            let use_module_init = quote::format_ident!("R_init_{use_module_ident}_miniextendr");
            syn::parse_quote!(#use_module_ident::#use_module_init(dll))
        })
        .collect::<Vec<syn::Expr>>();

    let module_r_wrappers = quote::format_ident!("module_r_wrappers_{module}");
    let r_wrapper_generators: Vec<syn::Expr> = extendr_module
        .extendr_fn
        .iter()
        .map(|x| {
            //TODO: put this in ExtendrFunction impl
            let rust_ident = &x.ident;
            let r_wrapper_generator = quote::format_ident!("r_wrapper_{rust_ident}");
            syn::parse_quote!(#r_wrapper_generator)
        })
        .collect();

    quote::quote! {


        //TODO: still need to deal with modules and their respective wrappers..
        // what to do here?
         


        #[doc(hidden)]
        #[unsafe(no_mangle)]
        #[inline(always)]
        pub(crate) const extern "C" fn #module_r_wrappers() -> [&'static std::ffi::CStr; #call_entries_len] {
            [
                #(#r_wrapper_generators),*
            ]
        }

        #[doc(hidden)]
        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        pub(crate) extern "C" fn #module_entrypoint_ident(dll: *mut DllInfo) {
            static CALL_ENTRIES: [R_CallMethodDef; {#call_entries_len + 1}] = [
                #(#call_entries,)*
                R_CallMethodDef {
                    name: std::ptr::null(),
                    fun: None,
                    numArgs: 0,
                }
            ];

            #(#use_other_modules;)*

            unsafe {
                R_registerRoutines(dll, std::ptr::null(), CALL_ENTRIES.as_ptr(), std::ptr::null(), std::ptr::null());
                // these are already present in entrypoint.c!
                // R_useDynamicSymbols(dll, Rboolean::FALSE);
                // R_forceSymbols(dll, Rboolean::TRUE);
            }
        }
    }
    .into()
}
