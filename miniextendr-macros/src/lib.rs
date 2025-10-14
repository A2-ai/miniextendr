//!
//!
//!
//!

struct ExtendrFunction {
    pub attrs: Vec<syn::Attribute>,
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
            attrs: itemfn.attrs,
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

    // dots support here
    let has_dots = item.sig.variadic.is_some();
    let mut named_dots: Option<syn::Ident> = if has_dots {
        let dots = item.sig.variadic.as_ref().unwrap();
        if let Some(named_dots) = dots.pat.as_ref() {
            if let syn::Pat::Ident(named_dots_ident) = named_dots.0.as_ref() {
                Some(named_dots_ident.ident.clone())
            } else {
                // FIXME: maybe an error? what could lead to here?
                None
            }
        } else {
            // unnamed dots
            None
        }
    } else {
        None
    };
    item.sig.variadic = None;
    // instead of ... replace with Dots type!
    //TODO: investigate why this needs to be &Dots.
    if has_dots {
        item.sig
            .inputs
            .push(if let Some(named_dots) = named_dots.as_ref() {
                syn::parse_quote!(#named_dots: &::miniextendr_api::dots::Dots)
            } else {
                // cannot use `_` as variable name, thus cannot use it as a placeholder for `...``
                // FIXME: check that no other parameter is called `_dots`!
                syn::parse_quote!(_dots: &::miniextendr_api::dots::Dots)
            });
    }
    let original_item = item.clone();
    use quote::ToTokens;
    let item = item.into_token_stream();
    let extendr_function = syn::parse2(item).unwrap();
    let ExtendrFunction {
        attrs,
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
    // these are needed to transmute fn-item to fn-pointer correctly.
    let func_ptr_def: Vec<syn::Pat> = (0..inputs.len())
        .map(|_| syn::parse_quote!(::miniextendr_api::ffi::SEXP))
        .collect();

    // calling the rust function with

    let rust_inputs: Vec<syn::Ident> = inputs
        .iter()
        .filter_map(|arg| {
            if let syn::FnArg::Typed(pt) = arg {
                if let syn::Pat::Ident(p) = pt.pat.as_ref() {
                    return Some(p.ident.clone());
                }
            }
            None
        })
        .collect();
    // dbg!(&rust_inputs);

    let c_wrapper_inputs: syn::punctuated::Punctuated<syn::FnArg, _> = inputs
        .clone()
        .into_pairs()
        .map(|pair| {
            let punct = pair.punct().cloned();
            let mut arg = pair.into_value();
            if let syn::FnArg::Typed(ref mut pt) = arg {
                *pt.ty.as_mut() = syn::parse_quote!(::miniextendr_api::ffi::SEXP);
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
            syn::FnArg::Typed(pat_type) => match (pat_type.pat.as_ref(), pat_type.ty.as_ref()) {
                // () param
                (syn::Pat::Ident(p), syn::Type::Tuple(t)) if t.elems.is_empty() => {
                    let ident = p.ident.clone();
                    if p.mutability.is_some() {
                        Some(quote::quote! { let mut #ident = (); })
                    } else {
                        Some(quote::quote! { let #ident = (); })
                    }
                }

                // non-() return
                (syn::Pat::Ident(p), _) => {
                    let ident = p.ident.clone();
                    if p.mutability.is_some() {
                        Some(quote::quote! {
                            let mut #ident = *::miniextendr_api::ffi::DATAPTR(#ident).cast();
                        })
                    } else {
                        Some(quote::quote! {
                            let #ident = *::miniextendr_api::ffi::DATAPTR_RO(#ident).cast();
                        })
                    }
                }
                _ => None,
            },
            // TODO: no support for self!
            syn::FnArg::Receiver(_) => todo!(),
        })
        .collect();

    //TODO: add an invisibility attribute to miniextendr(invisible)
    // after this block, otherwise it will be overwritten.
    let is_invisible_return_type: bool;
    let return_statement = match &output {
        // no arrow
        syn::ReturnType::Default => {
            is_invisible_return_type = true;
            quote::quote! { unsafe { ::miniextendr_api::ffi::R_NilValue } }
        }

        syn::ReturnType::Type(_, ty) => match ty.as_ref() {
            // -> ()
            syn::Type::Tuple(t) if t.elems.is_empty() => {
                is_invisible_return_type = true;
                quote::quote! { unsafe { ::miniextendr_api::ffi::R_NilValue } }
            }

            // -> Option<...> cases
            syn::Type::Path(p)
                if p.path.segments.last().map(|s| &s.ident)
                    == Some(&syn::Ident::new("Option", p.path.span())) =>
            {
                let seg = p.path.segments.last().unwrap();
                let is_unit_inner = match &seg.arguments {
                syn::PathArguments::AngleBracketed(ab) => ab.args.iter().any(|ga| {
                    matches!(ga, syn::GenericArgument::Type(syn::Type::Tuple(t)) if t.elems.is_empty())
                }),
                _ => false,
            };

                if is_unit_inner {
                    // -> Option<()>
                    is_invisible_return_type = true;
                    quote::quote! {
                        let _ = result.unwrap();
                        unsafe { ::miniextendr_api::ffi::R_NilValue }
                    }
                } else {
                    is_invisible_return_type = false;
                    // -> Option<T>
                    quote::quote! { ::miniextendr_api::ffi::Rf_ScalarInteger(result.unwrap()) }
                }
            }

            // -> Result<...> cases
            syn::Type::Path(p)
                if p.path.segments.last().map(|s| &s.ident)
                    == Some(&syn::Ident::new("Result", p.path.span())) =>
            {
                let seg = p.path.segments.last().unwrap();
                let is_unit_inner = match &seg.arguments {
                syn::PathArguments::AngleBracketed(ab) => ab.args.iter().any(|ga| {
                    matches!(ga, syn::GenericArgument::Type(syn::Type::Tuple(t)) if t.elems.is_empty())
                }),
                _ => false,
            };

                if is_unit_inner {
                    // -> Result<(), _>
                    is_invisible_return_type = true;
                    quote::quote! {
                        let _ = result.unwrap();
                        unsafe { ::miniextendr_api::ffi::R_NilValue }
                    }
                } else {
                    is_invisible_return_type = false;
                    // -> Result<T, _>
                    quote::quote! { ::miniextendr_api::ffi::Rf_ScalarInteger(result.unwrap()) }
                }
            }

            // all other T
            _ => {
                is_invisible_return_type = false;
                quote::quote! { ::miniextendr_api::ffi::Rf_ScalarInteger(result) }
            }
        },
    };
    //TODO: add an invisibility attribute to miniextendr(invisible)
    let is_invisible_return_type = is_invisible_return_type;

    let c_wrapper = if abi.is_some() {
        proc_macro2::TokenStream::new()
    } else {
        quote::quote! {
            // TODO: add the method it is wrapping as doc comment
            #[doc = "C wrapper method for TODO"]
            #[unsafe(no_mangle)]
            #vis unsafe extern "C" fn #c_ident #generics(#c_wrapper_inputs) -> ::miniextendr_api::ffi::SEXP {
                let old = std::panic::take_hook();
                std::panic::set_hook(Box::new(|_| {}));
                let result = ::miniextendr_api::unwind::with_r_unwind_protect(move || unsafe {
                    // TODO: these borrows ought to be used based on the mutability requirements...
                    #[allow(unused_imports)]
                    use std::borrow::{Borrow, BorrowMut};
                    #(#input_names)*
                    // FIXME: shouldn't this borrow?
                    // dbg!(#rust_inputs);
                    let result = #rust_ident(#(#rust_inputs),*);
                    #return_statement
                }, move || {
                    std::panic::set_hook(old);
                });
                result
            }
        }
    };

    // check the validity of the provided C-function!
    if abi.is_some() {
        // check that #[no_mangle] or #[unsafe(no_mangle)] is present!
        let has_no_mangle = attrs.iter().any(|attr| {
            attr.path().is_ident("no_mangle")
                || attr
                    .parse_nested_meta(|meta| {
                        if meta.path.is_ident("no_mangle") {
                            Err(meta.error("found #[no_mangle]"))
                        } else {
                            Ok(())
                        }
                    })
                    .is_err()
        });

        if !has_no_mangle {
            return syn::Error::new(
                attrs
                    .first()
                    .map(|attr| attr.span())
                    .unwrap_or_else(|| abi.span()),
                "missing #[no_mangle] (edition 2021) or #[unsafe(no_mangle)] (edition 2024).",
            )
            .into_compile_error()
            .into();
        }

        // TODO: check that the return type is SEXP;
        match output {
            non_return_type @ syn::ReturnType::Default => {
                return syn::Error::new(non_return_type.span(), "output must be SEXP")
                    .into_compile_error()
                    .into();
            }
            syn::ReturnType::Type(_rarrow, output_type) => match output_type.as_ref() {
                syn::Type::Path(type_path) => {
                    if let Some(path_to_sexp) = type_path.path.segments.last().map(|x| &x.ident) {
                        if path_to_sexp != "SEXP" {
                            return syn::Error::new(path_to_sexp.span(), "output must be SEXP")
                                .into_compile_error()
                                .into();
                        }
                    }
                }
                _ => {
                    return syn::Error::new(output_type.span(), "output must be SEXP")
                        .into_compile_error()
                        .into();
                }
            },
        }
    }

    let r_wrapper_args: Vec<_> = inputs
        .into_iter()
        .map(|x| {
            let syn::FnArg::Typed(pat_type) = &x else {
                // FIXME: convert this to an error
                unreachable!()
            };
            let syn::PatType {
                // TODO: use `attrs` to add Default value!
                attrs: _,
                pat,
                colon_token: _,
                ty: _,
            } = &pat_type;
            // dbg!(&pat);
            match pat.as_ref() {
                syn::Pat::Ident(pat_ident) => {
                    let mut arg_name = pat_ident.ident.to_string();
                    if arg_name.starts_with('_') {
                        arg_name.insert_str(0, "unused");
                    }
                    let arg_name = syn::Ident::new(&arg_name, pat_ident.ident.span());

                    arg_name
                }
                _ => todo!(),
            }
        })
        .collect();

    // need to change the leading underscore of a dots variable to match
    // r's requirement of non _ as leading character in alias/variable names.
    if has_dots {
        if let Some(named_dots) = &mut named_dots {
            // TODO: promote to a helper method, see `r_wrapper_args` processing
            let mut arg_name = named_dots.to_string();
            if arg_name.starts_with('_') {
                arg_name.insert_str(0, "unused");
            }
            let arg_name = syn::Ident::new(&arg_name, named_dots.span());
            *named_dots = arg_name;
        }
    }
    let named_dots = named_dots;

    // region: R wrappers generation in `fn`
    let mut r_wrapper_args: Vec<_> = r_wrapper_args
        .into_iter()
        .map(|x| x.into_token_stream())
        .collect();
    if has_dots {
        r_wrapper_args.pop();
        if let Some(named_dots) = &named_dots {
            r_wrapper_args.push(syn::parse_quote!(list(#named_dots)));
        } else {
            r_wrapper_args.push(syn::parse_quote!(list(...)));
        }
    }
    let r_wrapper_return = if !is_invisible_return_type {
        quote::quote! {.Call(#c_ident #(, #r_wrapper_args)*)}
    } else {
        quote::quote! {invisible(.Call(#c_ident #(, #r_wrapper_args)*))}
    };
    let r_wrapper_ident = if abi.is_some() {
        &quote::format_ident!("unsafe_{rust_ident}")
    } else {
        rust_ident
    };
    if has_dots {
        r_wrapper_args.pop();
        if let Some(named_dots) = named_dots {
            r_wrapper_args.push(syn::parse_quote!(#named_dots = ...));
        } else {
            r_wrapper_args.push(syn::parse_quote!(...));
        }
    }
    let r_wrapper = quote::quote! {
        #r_wrapper_ident <- function(#(#r_wrapper_args),*) {

            #r_wrapper_return
        }
    };
    let r_wrapper_string = r_wrapper.to_string();
    let r_wrapper_str = syn::LitStr::new(&r_wrapper_string, r_wrapper.span());

    let r_wrapper_generator = quote::format_ident!("r_wrapper_{rust_ident}");

    // endregion

    let abi = abi.unwrap_or(syn::parse_quote!(extern "C"));
    quote::quote! {
        #original_item

        #c_wrapper

        const #r_wrapper_generator: &'static str = #r_wrapper_str;

        // TODO: unhide docs if you add the num_args and the rust-name, then the C wrapper name!
        // also handle the case where there is no rust-name because it is an `unsafe extern "C"` being exported!
        #[doc(hidden)]
        #[inline(always)]
        const fn #call_method_def() -> ::miniextendr_api::ffi::R_CallMethodDef {
            unsafe {
                ::miniextendr_api::ffi::R_CallMethodDef {
                    name: #c_ident_name.as_ptr(),
                    fun: Some(std::mem::transmute::<unsafe #abi fn(#(#func_ptr_def),*) -> ::miniextendr_api::ffi::SEXP, unsafe #abi fn(...) -> ::miniextendr_api::ffi::SEXP>(#c_ident)),
                    numArgs: #num_args,
                }
            }
        }
    }
    .into()
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
    #[allow(dead_code)]
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
            syn::UseTree::Path(_) | syn::UseTree::Glob(_) | syn::UseTree::Group(_) => {
                return Err(syn::Error::new(use_name.span(), "syntax not supported"));
            }
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
    // TODO: add extendr_impl: Vec<ExtendrImpl>
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

// TODO: Currently, miniextendr_module does not distinguish between
// `extern "C" fn` and `fn` items.. they are treated alike.
#[proc_macro]
pub fn miniextendr_module(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let miniextendr_module = syn::parse_macro_input!(item as ExtendrModule);

    let module = miniextendr_module.extendr_module.ident;
    let module_entrypoint_ident = quote::format_ident!("R_init_{module}_miniextendr");
    let call_entries: Vec<syn::Expr> = miniextendr_module
        .extendr_fn
        .iter()
        .map(|miniextendr_fn| {
            //TODO: put this in ExtendrFunction impl
            let rust_ident = &miniextendr_fn.ident;
            let call_method_def = quote::format_ident!("call_method_def_{rust_ident}");
            syn::parse_quote!(#call_method_def())
        })
        .collect();
    let call_entries_len = call_entries.len();

    // call the R_init from all the submodules (given by `use`)
    let use_other_modules = miniextendr_module
        .extendr_use
        .iter()
        .map(|x| {
            let use_module_ident = &x.use_name.ident;
            let use_module_init = quote::format_ident!("R_init_{use_module_ident}_miniextendr");
            syn::parse_quote!(#use_module_ident::#use_module_init(dll))
        })
        .collect::<Vec<syn::Expr>>();

    // region: r wrapper generation in `mod`

    let r_wrapper_generators: Vec<syn::Expr> = miniextendr_module
        .extendr_fn
        .iter()
        .map(|x| {
            //TODO: put this in ExtendrFunction impl
            let rust_ident = &x.ident;
            let r_wrapper_generator = quote::format_ident!("r_wrapper_{rust_ident}");
            syn::parse_quote!(#r_wrapper_generator)
        })
        .collect();
    let r_wrappers_use_other_modules = miniextendr_module
        .extendr_use
        .iter()
        .map(|x| {
            let use_module_ident = &x.use_name.ident;
            let r_wrappers_use_module = quote::format_ident!("R_WRAPPERS_PARTS_{use_module_ident}");
            syn::parse_quote!(#use_module_ident::#r_wrappers_use_module)
        })
        .collect::<Vec<syn::Expr>>();
    let r_wrappers_parts_ident = quote::format_ident!("R_WRAPPERS_PARTS_{module}");
    let r_wrappers_deps_ident = quote::format_ident!("R_WRAPPERS_DEPS_{module}");

    // endregion
    quote::quote! {

        //TODO: still need to deal with modules and their respective wrappers..
        // what to do here?

        #[doc(hidden)]
        pub const #r_wrappers_parts_ident: &[&str] = &[#(#r_wrapper_generators),*];
        #[doc(hidden)]
        pub const #r_wrappers_deps_ident: &[&[&str]] = &[#(#r_wrappers_use_other_modules),*];

        //TODO: add the use-modules and their entry point docs to the doc!

        // #[doc(hidden)]
        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        /// Internal function that is used by R to register the exported
        /// miniextendr items.
        pub(crate) extern "C" fn #module_entrypoint_ident(dll: *mut ::miniextendr_api::ffi::DllInfo) {
            static CALL_ENTRIES: [::miniextendr_api::ffi::R_CallMethodDef; {#call_entries_len + 1}] = [
                #(#call_entries,)*
                ::miniextendr_api::ffi::R_CallMethodDef {
                    name: std::ptr::null(),
                    fun: None,
                    numArgs: 0,
                }
            ];

            #(#use_other_modules;)*

            unsafe {
                ::miniextendr_api::ffi::R_registerRoutines(dll, std::ptr::null(), CALL_ENTRIES.as_ptr(), std::ptr::null(), std::ptr::null());
                // these are already present in entrypoint.c!
                // R_useDynamicSymbols(dll, Rboolean::FALSE);
                // R_forceSymbols(dll, Rboolean::TRUE);
            }
        }
    }
    .into()
}
