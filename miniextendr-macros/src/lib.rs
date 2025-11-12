// miniextendr-macros procedural macros

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

fn first_type_argument(seg: &syn::PathSegment) -> Option<&syn::Type> {
    if let syn::PathArguments::AngleBracketed(ab) = &seg.arguments {
        for arg in ab.args.iter() {
            if let syn::GenericArgument::Type(ty) = arg {
                return Some(ty);
            }
        }
    }
    None
}

fn is_unit_type(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Tuple(t) if t.elems.is_empty())
}

fn is_sexp_type(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Path(p) if p
        .path
        .segments
        .last()
        .map(|s| s.ident == "SEXP")
        .unwrap_or(false))
}

#[proc_macro_attribute]
pub fn miniextendr(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut item = syn::parse_macro_input!(item as syn::ItemFn);

    // dots support here
    //TODO: move to ExtendrFunction?
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
    let uses_internal_c_wrapper = abi.is_none();
    let rust_arg_count = inputs.len();
    let registered_arg_count = if uses_internal_c_wrapper {
        rust_arg_count + 1
    } else {
        rust_arg_count
    };
    let num_args = syn::LitInt::new(&registered_arg_count.to_string(), inputs.span());

    let rust_ident = &ident;
    let call_method_def = quote::format_ident!("call_method_def_{rust_ident}");
    let c_ident = if uses_internal_c_wrapper {
        &quote::format_ident!("C_{rust_ident}")
    } else {
        rust_ident
    };

    // name of the C-wrapper
    let c_ident_name = syn::LitCStr::new(
        std::ffi::CString::new(c_ident.to_string())
            .expect("couldn't crate a C-string for the C wrapper name")
            .as_c_str(),
        ident.span(),
    );
    // registration of the C-wrapper
    // these are needed to transmute fn-item to fn-pointer correctly.
    let mut func_ptr_def: Vec<syn::Pat> = Vec::new();
    if uses_internal_c_wrapper {
        func_ptr_def.push(syn::parse_quote!(::miniextendr_api::ffi::SEXP));
    }
    func_ptr_def.extend(
        (0..inputs.len())
            .map(|_| syn::parse_quote!(::miniextendr_api::ffi::SEXP))
            .collect::<Vec<_>>(),
    );

    // calling the rust function with
    let rust_inputs: Vec<syn::Ident> = inputs
        .iter()
        .filter_map(|arg| {
            if let syn::FnArg::Typed(pt) = arg
                && let syn::Pat::Ident(p) = pt.pat.as_ref()
            {
                return Some(p.ident.clone());
            }
            None
        })
        .collect();
    // dbg!(&rust_inputs);

    // calling the C-wrapper with
    let call_param_ident = syn::Ident::new("__miniextendr_call", proc_macro2::Span::call_site());
    let mut c_wrapper_inputs: Vec<_> = Vec::new();
    if uses_internal_c_wrapper {
        c_wrapper_inputs.push(syn::parse_quote!(#call_param_ident: ::miniextendr_api::ffi::SEXP));
    }
    c_wrapper_inputs.extend(inputs.clone().into_pairs().map(|pair| {
        let arg = pair.value();
        match arg {
            syn::FnArg::Receiver(receiver) => {
                syn::Error::new(receiver.span(), "impl-blocks not supported yet").to_compile_error()
            }
            syn::FnArg::Typed(pt) => {
                let syn::PatType {
                    attrs: _,
                    pat,
                    colon_token: _,
                    ty: _,
                } = pt;
                match pat.as_ref() {
                    syn::Pat::Ident(pat_ident) => {
                        let mut pat_ident = pat_ident.clone();
                        pat_ident.mutability = None;
                        pat_ident.by_ref = None;
                        let ident = pat_ident;
                        syn::parse_quote!(#ident: ::miniextendr_api::ffi::SEXP)
                    }
                    syn::Pat::Wild(_pat_wild) => {
                        todo!("what should c wrapper do with _ args?")
                    }
                    _ => todo!(),
                }
            }
        }
    }));
    // dbg!(&wrapper_inputs);
    let mut pre_call_statements: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut closure_statements: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut post_call_statements: Vec<proc_macro2::TokenStream> = Vec::new();
    for arg in inputs.iter() {
        let syn::FnArg::Typed(pat_type) = arg else {
            // TODO: no support for self!
            continue;
        };
        let syn::Pat::Ident(pat_ident) = pat_type.pat.as_ref() else {
            continue;
        };
        let ident = pat_ident.ident.clone();

        match pat_type.ty.as_ref() {
            syn::Type::Tuple(t) if t.elems.is_empty() => {
                if pat_ident.mutability.is_some() {
                    closure_statements.push(quote::quote! { let mut #ident = (); });
                } else {
                    closure_statements.push(quote::quote! { let #ident = (); });
                }
            }
            syn::Type::Reference(r) => {
                let send_ident = quote::format_ident!("__miniextendr_arg_{ident}");
                pre_call_statements.push(quote::quote! {
                    let #send_ident = unsafe { ::miniextendr_api::ffi::SendSEXP::new(#ident) };
                });
                let is_dots = matches!(
                    r.elem.as_ref(),
                    syn::Type::Path(tp)
                        if tp
                            .path
                            .segments
                            .last()
                            .map(|s| s.ident == "Dots")
                            .unwrap_or(false)
                );
                if is_dots {
                    let storage_ident = quote::format_ident!("{}_storage", ident);
                    closure_statements.push(quote::quote! {
                        let #storage_ident = ::miniextendr_api::dots::Dots { inner: #send_ident.get() };
                        let #ident = &#storage_ident;
                    });
                } else if pat_ident.mutability.is_some() {
                    closure_statements.push(quote::quote! {
                        let mut #ident = *::miniextendr_api::ffi::DATAPTR(#send_ident.get()).cast();
                    });
                } else {
                    closure_statements.push(quote::quote! {
                        let #ident = *::miniextendr_api::ffi::DATAPTR_RO(#send_ident.get()).cast();
                    });
                }
            }
            _ => {
                let send_ident = quote::format_ident!("__miniextendr_arg_{ident}");
                pre_call_statements.push(quote::quote! {
                    let #send_ident = unsafe { ::miniextendr_api::ffi::SendSEXP::new(#ident) };
                });
                if pat_ident.mutability.is_some() {
                    closure_statements.push(quote::quote! {
                        let mut #ident = *::miniextendr_api::ffi::DATAPTR(#send_ident.get()).cast();
                    });
                } else {
                    closure_statements.push(quote::quote! {
                        let #ident = *::miniextendr_api::ffi::DATAPTR_RO(#send_ident.get()).cast();
                    });
                }
            }
        }
    }

    //TODO: add an invisibility attribute to miniextendr(invisible)
    // after this block, otherwise it will be overwritten.
    let is_invisible_return_type: bool;
    let rust_result_ident =
        syn::Ident::new("__miniextendr_rust_result", proc_macro2::Span::mixed_site());
    let option_none_error = quote::quote! {
        || ::std::borrow::Cow::Borrowed(concat!(
            "miniextendr function `",
            stringify!(#rust_ident),
            "` returned None"
        ))
    };
    let result_err_mapper = quote::quote!(|err| ::std::borrow::Cow::Owned(format!("{err:?}")));
    let return_expression = match &output {
        // no arrow
        syn::ReturnType::Default => {
            is_invisible_return_type = true;
            quote::quote! { ::miniextendr_api::ffi::R_NilValue }
        }

        syn::ReturnType::Type(_, ty) => match ty.as_ref() {
            // -> ()
            syn::Type::Tuple(t) if t.elems.is_empty() => {
                is_invisible_return_type = true;
                quote::quote! { ::miniextendr_api::ffi::R_NilValue }
            }
            syn::Type::Path(_p) if is_sexp_type(ty.as_ref()) => {
                is_invisible_return_type = false;
                quote::quote! { #rust_result_ident }
            }

            // -> Option<...> cases
            syn::Type::Path(p)
                if p.path.segments.last().map(|s| &s.ident)
                    == Some(&syn::Ident::new("Option", p.path.span())) =>
            {
                let seg = p.path.segments.last().unwrap();
                // check ONLY the first type argument of Option<T>
                let inner_ty = first_type_argument(seg);
                let is_unit_inner = inner_ty.is_some_and(is_unit_type);
                let is_sexp_inner = inner_ty.is_some_and(is_sexp_type);

                if is_unit_inner {
                    // -> Option<()>
                    is_invisible_return_type = true;
                    post_call_statements.push(quote::quote! {
                        #rust_result_ident.ok_or_else(#option_none_error.clone())?;
                    });
                    quote::quote! { ::miniextendr_api::ffi::R_NilValue }
                } else {
                    is_invisible_return_type = false;
                    // -> Option<T>
                    post_call_statements.push(quote::quote! {
                        let #rust_result_ident =
                            #rust_result_ident.ok_or_else(#option_none_error.clone())?;
                    });
                    if is_sexp_inner {
                        quote::quote! { #rust_result_ident }
                    } else {
                        quote::quote! { ::miniextendr_api::ffi::Rf_ScalarInteger(#rust_result_ident) }
                    }
                }
            }

            // -> Result<...> cases
            syn::Type::Path(p)
                if p.path.segments.last().map(|s| &s.ident)
                    == Some(&syn::Ident::new("Result", p.path.span())) =>
            {
                let seg = p.path.segments.last().unwrap();
                // check ONLY the first type argument (Ok type) of Result<Ok, Err>
                let ok_ty = first_type_argument(seg);
                let ok_is_unit = ok_ty.is_some_and(is_unit_type);
                let ok_is_sexp = ok_ty.is_some_and(is_sexp_type);

                if ok_is_unit {
                    // -> Result<(), E>
                    is_invisible_return_type = true;
                    post_call_statements.push(quote::quote! {
                        #rust_result_ident.map_err(#result_err_mapper)?;
                    });
                    quote::quote! { ::miniextendr_api::ffi::R_NilValue }
                } else {
                    is_invisible_return_type = false;
                    // -> Result<T, E>
                    post_call_statements.push(quote::quote! {
                        let #rust_result_ident = #rust_result_ident.map_err(#result_err_mapper)?;
                    });
                    if ok_is_sexp {
                        quote::quote! { #rust_result_ident }
                    } else {
                        quote::quote! { ::miniextendr_api::ffi::Rf_ScalarInteger(#rust_result_ident) }
                    }
                }
            }

            // all other T
            _ => {
                is_invisible_return_type = false;
                quote::quote! { ::miniextendr_api::ffi::Rf_ScalarInteger(#rust_result_ident) }
            }
        },
    };
    //TODO: add an invisibility attribute to miniextendr(invisible)

    let c_wrapper = if abi.is_some() {
        proc_macro2::TokenStream::new()
    } else {
        quote::quote! {
            #[doc = "C wrapper method for TODO"]
            #[unsafe(no_mangle)]
            #vis unsafe extern "C" fn #c_ident #generics(#(#c_wrapper_inputs),*) -> ::miniextendr_api::ffi::SEXP {
                #(#pre_call_statements)*

                unsafe {
                        ::miniextendr_api::unwind::call_worker(#call_param_ident, move || {
                            #(#closure_statements)*
                            let #rust_result_ident = #rust_ident(#(#rust_inputs),*);
                            #(#post_call_statements)*
                            let __miniextendr_sexp_result = #return_expression;
                            let __miniextendr_sexp_result = ::miniextendr_api::ffi::SendSEXP::new(__miniextendr_sexp_result);
                            Ok(__miniextendr_sexp_result)
                        })
                }
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
                    if let Some(path_to_sexp) = type_path.path.segments.last().map(|x| &x.ident)
                        && path_to_sexp != "SEXP"
                    {
                        return syn::Error::new(path_to_sexp.span(), "output must be SEXP")
                            .into_compile_error()
                            .into();
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

    // region: R wrappers generation in `fn`
    // normalize `named_dots` for R (no leading underscore)
    if has_dots && let Some(named) = &mut named_dots {
            let mut arg_name = named.to_string();
            if arg_name.starts_with("__") {
                arg_name.insert_str(0, "private");
            } else if arg_name.starts_with('_') {
                arg_name.insert_str(0, "unused");
            }
            let arg_ident = syn::Ident::new(&arg_name, named.span());
            *named = arg_ident;
    }

    // Build both the .Call argument list and the formal parameter list in one pass
    let last_idx = inputs.len().saturating_sub(1);
    let mut r_call_args_strs: Vec<String> = Vec::new();
    if uses_internal_c_wrapper {
        r_call_args_strs.push(".call = match.call()".to_string());
    }
    let mut r_formals: Vec<proc_macro2::TokenStream> = Vec::new();
    for (idx, x) in inputs.iter().enumerate() {
        let syn::FnArg::Typed(pat_type) = x else {
            unreachable!()
        };
        let syn::PatType {
            attrs: _,
            pat,
            colon_token: _,
            ty,
        } = pat_type;

        // derive R argument name, applying leading-underscore rename
        let arg_ident = match pat.as_ref() {
            syn::Pat::Ident(pat_ident) => {
                let mut arg_name = pat_ident.ident.to_string();
                if arg_name.starts_with("__") {
                    arg_name.insert_str(0, "private");
                } else if arg_name.starts_with('_') {
                    arg_name.insert_str(0, "unused");
                }
                syn::Ident::new(&arg_name, pat_ident.ident.span())
            }
            _ => unreachable!(),
        };

        // call-site argument
        if has_dots && idx == last_idx {
            if let Some(named_dots) = &named_dots {
                r_call_args_strs.push(format!("list({})", named_dots));
            } else {
                r_call_args_strs.push("list(...)".to_string());
            }
        } else {
            r_call_args_strs.push(arg_ident.to_string());
        }

        // formal parameter (with defaults for unit types)
        if has_dots && idx == last_idx {
            if let Some(named_dots) = &named_dots {
                let named = syn::Ident::new(&named_dots.to_string(), named_dots.span());
                r_formals.push(syn::parse_quote!(#named = ...));
            } else {
                r_formals.push(syn::parse_quote!(...));
            }
        } else {
            match ty.as_ref() {
                syn::Type::Tuple(t) if t.elems.is_empty() => {
                    r_formals.push(syn::parse_quote!(#arg_ident = NULL));
                }
                _ => {
                    r_formals.push(arg_ident.into_token_stream());
                }
            }
        }
    }

    // region: R wrappers generation in `fn`
    // Build the R body string consistently
    let c_ident_str = c_ident.to_string();
    let call_args_joined = r_call_args_strs.join(", ");
    let call_expr = if r_call_args_strs.is_empty() {
        format!(".Call({})", c_ident_str)
    } else {
        format!(".Call({}, {})", c_ident_str, call_args_joined)
    };
    let r_wrapper_return_str = if !is_invisible_return_type {
        call_expr
    } else {
        format!("invisible({})", call_expr)
    };
    let r_wrapper_ident = if abi.is_some() {
        &quote::format_ident!("unsafe_{rust_ident}")
    } else {
        rust_ident
    };
    // Stable, consistent R formatting style: brace on same line, body indented, closing brace on its own line
    let formals_joined = r_formals
        .iter()
        .map(|t| t.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let r_wrapper_string = format!(
        "{} <- function({}) {{\n    {}\n}}",
        r_wrapper_ident, formals_joined, r_wrapper_return_str
    );
    let r_wrapper_str = syn::LitStr::new(&r_wrapper_string, r_wrapper_ident.span());

    let rust_ident_upper = rust_ident.to_string().to_uppercase();
    let r_wrapper_generator = quote::format_ident!("R_WRAPPER_{rust_ident_upper}");

    // endregion

    let abi = abi.unwrap_or(syn::parse_quote!(extern "C"));
    let expanded: proc_macro::TokenStream = quote::quote! {
        // rust function!
        #original_item

        // C wrapper
        #c_wrapper

        // R wrapper
        const #r_wrapper_generator: &str = #r_wrapper_str;


        // registration of C wrapper in R

        // TODO: unhide docs if you add the num_args and the rust-name, then the C wrapper name!
        // also handle the case where there is no rust-name because it is an `unsafe extern "C"` being exported!
        #[doc(hidden)]
        #[inline(always)]
        #[allow(non_snake_case)]
        const fn #call_method_def() -> ::miniextendr_api::ffi::R_CallMethodDef {
            unsafe {
                ::miniextendr_api::ffi::R_CallMethodDef {
                    name: #c_ident_name.as_ptr(),
                    fun: Some(std::mem::transmute::<
                        unsafe #abi fn(#(#func_ptr_def),*) -> ::miniextendr_api::ffi::SEXP,
                        unsafe #abi fn(...) -> ::miniextendr_api::ffi::SEXP
                    >(#c_ident)),
                    numArgs: #num_args,
                }
            }
        }
    }
    .into();

    expanded
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
    #[allow(dead_code)]
    pub _extendr_struct: Vec<ExtendrModuleStruct>,
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
            syn::punctuated::Punctuated::parse_terminated_with(input, ExtendrItem::parse)?;

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
            _extendr_struct: structs,
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
            let rust_ident_upper = rust_ident.to_string().to_uppercase();
            let r_wrapper_generator = quote::format_ident!("R_WRAPPER_{rust_ident_upper}");
            syn::parse_quote!(#r_wrapper_generator)
        })
        .collect();
    let r_wrappers_use_other_modules = miniextendr_module
        .extendr_use
        .iter()
        .map(|x| {
            let use_module_ident = &x.use_name.ident;
            let use_module_ident_upper = use_module_ident.to_string().to_uppercase();
            let r_wrappers_use_module =
                quote::format_ident!("R_WRAPPERS_PARTS_{use_module_ident_upper}");
            syn::parse_quote!(#use_module_ident::#r_wrappers_use_module)
        })
        .collect::<Vec<syn::Expr>>();

    let module_upper = module.to_string().to_uppercase();
    let r_wrappers_parts_ident = quote::format_ident!("R_WRAPPERS_PARTS_{module_upper}");
    let r_wrappers_deps_ident = quote::format_ident!("R_WRAPPERS_DEPS_{module_upper}");

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
