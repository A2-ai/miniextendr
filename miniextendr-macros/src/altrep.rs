//! ALTREP struct expansion for `#[miniextendr]`.
//!
//! This module handles the expansion of `#[miniextendr]` when applied to structs,
//! generating the necessary trait implementations for ALTREP.
//!
//! # Usage
//!
//! Simple 1-field wrapper struct (recommended):
//! ```ignore
//! #[miniextendr]
//! pub struct MyInts(Vec<i32>);
//! ```
//!
//! With explicit class name (optional):
//! ```ignore
//! #[miniextendr(class = "CustomClassName")]
//! pub struct MyInts(Vec<i32>);
//! ```

pub fn expand_altrep_struct(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    use syn::spanned::Spanned;
    let input: syn::ItemStruct = match syn::parse(item) {
        Ok(it) => it,
        Err(e) => return e.into_compile_error().into(),
    };

    let ident = input.ident.clone();
    let generics = input.generics.clone();

    // Extract the inner type - must be a 1-field struct
    let data_ty: syn::Type = match &input.fields {
        syn::Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
            fields.unnamed.first().unwrap().ty.clone()
        }
        syn::Fields::Named(fields) if fields.named.len() == 1 => {
            fields.named.first().unwrap().ty.clone()
        }
        _ => {
            return syn::Error::new(
                input.span(),
                "#[miniextendr] ALTREP requires a 1-field wrapper struct, e.g., `struct MyInts(Vec<i32>);`",
            )
            .into_compile_error()
            .into();
        }
    };

    // Parse attr list: class = "..." (optional), base = "..." (optional)
    // pkg is no longer needed - we use ALTREP_PKG_NAME at runtime
    use syn::parse::Parser;
    let parser =
        syn::punctuated::Punctuated::<syn::MetaNameValue, syn::Token![,]>::parse_terminated;
    let args = match parser.parse(attr) {
        Ok(v) => v,
        Err(e) => return e.into_compile_error().into(),
    };
    let mut class_name = None::<String>;
    let mut base_name = None::<String>;
    let mut base_lit = None::<syn::LitStr>;
    for nv in args {
        let key = nv
            .path
            .get_ident()
            .map(|i| i.to_string())
            .unwrap_or_default();
        if let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(s),
            ..
        }) = &nv.value
        {
            match key.as_str() {
                "class" => class_name = Some(s.value()),
                "base" => {
                    base_name = Some(s.value());
                    base_lit = Some(s.clone());
                }
                // Silently ignore "pkg" for backwards compatibility, but it's no longer used
                "pkg" => {}
                _ => {}
            }
        }
    }

    // Default class name to struct name if not provided
    let class_name = class_name.unwrap_or_else(|| ident.to_string());
    // base is now OPTIONAL - inferred from InferBase if not provided

    // Validate base if provided, otherwise use InferBase inference
    let base_variant: syn::Expr = if let Some(ref base_name) = base_name {
        match base_name.as_str() {
            "Int" => syn::parse_quote!(::miniextendr_api::altrep::RBase::Int),
            "Real" => syn::parse_quote!(::miniextendr_api::altrep::RBase::Real),
            "Logical" => syn::parse_quote!(::miniextendr_api::altrep::RBase::Logical),
            "Raw" => syn::parse_quote!(::miniextendr_api::altrep::RBase::Raw),
            "String" => syn::parse_quote!(::miniextendr_api::altrep::RBase::String),
            "List" => syn::parse_quote!(::miniextendr_api::altrep::RBase::List),
            "Complex" => syn::parse_quote!(::miniextendr_api::altrep::RBase::Complex),
            _ => {
                return syn::Error::new_spanned(
                    base_lit.expect("base_lit set when base_name is Some"),
                    "base must be one of Int|Real|Logical|Raw|String|List|Complex",
                )
                .into_compile_error()
                .into();
            }
        }
    } else {
        // Infer from InferBase trait (auto-implemented via impl_inferbase_* macros)
        syn::parse_quote!(<#data_ty as ::miniextendr_api::altrep_data::InferBase>::BASE)
    };

    // The trampoline type is always the inner data type
    let tramp_ty = data_ty.clone();

    // Generate family setters and make_class based on the base type
    // If base is explicitly provided, generate type-specific code at macro time.
    // If base is not provided, use AltrepInstaller trait for compile-time dispatch.
    let (family_setters, make_class): (proc_macro2::TokenStream, proc_macro2::TokenStream) =
        if let Some(ref base_name) = base_name {
            // Explicit base: generate type-specific code at macro time
            let setters = match base_name.as_str() {
                "Int" => quote::quote! {
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltInteger>::HAS_ELT, R_set_altinteger_Elt_method, bridge::t_int_elt::<#tramp_ty>);
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltInteger>::HAS_GET_REGION, R_set_altinteger_Get_region_method, bridge::t_int_get_region::<#tramp_ty>);
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltInteger>::HAS_IS_SORTED, R_set_altinteger_Is_sorted_method, bridge::t_int_is_sorted::<#tramp_ty>);
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltInteger>::HAS_NO_NA, R_set_altinteger_No_NA_method, bridge::t_int_no_na::<#tramp_ty>);
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltInteger>::HAS_SUM, R_set_altinteger_Sum_method, bridge::t_int_sum::<#tramp_ty>);
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltInteger>::HAS_MIN, R_set_altinteger_Min_method, bridge::t_int_min::<#tramp_ty>);
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltInteger>::HAS_MAX, R_set_altinteger_Max_method, bridge::t_int_max::<#tramp_ty>);
                },
                "Real" => quote::quote! {
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltReal>::HAS_ELT, R_set_altreal_Elt_method, bridge::t_real_elt::<#tramp_ty>);
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltReal>::HAS_GET_REGION, R_set_altreal_Get_region_method, bridge::t_real_get_region::<#tramp_ty>);
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltReal>::HAS_IS_SORTED, R_set_altreal_Is_sorted_method, bridge::t_real_is_sorted::<#tramp_ty>);
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltReal>::HAS_NO_NA, R_set_altreal_No_NA_method, bridge::t_real_no_na::<#tramp_ty>);
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltReal>::HAS_SUM, R_set_altreal_Sum_method, bridge::t_real_sum::<#tramp_ty>);
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltReal>::HAS_MIN, R_set_altreal_Min_method, bridge::t_real_min::<#tramp_ty>);
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltReal>::HAS_MAX, R_set_altreal_Max_method, bridge::t_real_max::<#tramp_ty>);
                },
                "Logical" => quote::quote! {
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltLogical>::HAS_ELT, R_set_altlogical_Elt_method, bridge::t_lgl_elt::<#tramp_ty>);
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltLogical>::HAS_GET_REGION, R_set_altlogical_Get_region_method, bridge::t_lgl_get_region::<#tramp_ty>);
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltLogical>::HAS_IS_SORTED, R_set_altlogical_Is_sorted_method, bridge::t_lgl_is_sorted::<#tramp_ty>);
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltLogical>::HAS_NO_NA, R_set_altlogical_No_NA_method, bridge::t_lgl_no_na::<#tramp_ty>);
                },
                "Raw" => quote::quote! {
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltRaw>::HAS_ELT, R_set_altraw_Elt_method, bridge::t_raw_elt::<#tramp_ty>);
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltRaw>::HAS_GET_REGION, R_set_altraw_Get_region_method, bridge::t_raw_get_region::<#tramp_ty>);
                },
                "String" => quote::quote! {
                    unsafe { R_set_altstring_Elt_method(cls, Some(bridge::t_str_elt::<#tramp_ty>)); }
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltString>::HAS_IS_SORTED, R_set_altstring_Is_sorted_method, bridge::t_str_is_sorted::<#tramp_ty>);
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltString>::HAS_NO_NA, R_set_altstring_No_NA_method, bridge::t_str_no_na::<#tramp_ty>);
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltString>::HAS_SET_ELT, R_set_altstring_Set_elt_method, bridge::t_str_set_elt::<#tramp_ty>);
                },
                "List" => quote::quote! {
                    unsafe { R_set_altlist_Elt_method(cls, Some(bridge::t_list_elt::<#tramp_ty>)); }
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltList>::HAS_SET_ELT, R_set_altlist_Set_elt_method, bridge::t_list_set_elt::<#tramp_ty>);
                },
                "Complex" => quote::quote! {
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltComplex>::HAS_ELT, R_set_altcomplex_Elt_method, bridge::t_cplx_elt::<#tramp_ty>);
                    set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltComplex>::HAS_GET_REGION, R_set_altcomplex_Get_region_method, bridge::t_cplx_get_region::<#tramp_ty>);
                },
                _ => quote::quote! {},
            };
            let make = match base_name.as_str() {
                "Int" => quote::quote! { ::miniextendr_api::ffi::altrep::R_make_altinteger_class(
                    <#ident as ::miniextendr_api::altrep::AltrepClass>::CLASS_NAME.as_ptr(),
                    ::miniextendr_api::AltrepPkgName::as_ptr(),
                    core::ptr::null_mut(),
                ) },
                "Real" => quote::quote! { ::miniextendr_api::ffi::altrep::R_make_altreal_class(
                    <#ident as ::miniextendr_api::altrep::AltrepClass>::CLASS_NAME.as_ptr(),
                    ::miniextendr_api::AltrepPkgName::as_ptr(),
                    core::ptr::null_mut(),
                ) },
                "Logical" => {
                    quote::quote! { ::miniextendr_api::ffi::altrep::R_make_altlogical_class(
                        <#ident as ::miniextendr_api::altrep::AltrepClass>::CLASS_NAME.as_ptr(),
                        ::miniextendr_api::AltrepPkgName::as_ptr(),
                        core::ptr::null_mut(),
                    ) }
                }
                "Raw" => quote::quote! { ::miniextendr_api::ffi::altrep::R_make_altraw_class(
                    <#ident as ::miniextendr_api::altrep::AltrepClass>::CLASS_NAME.as_ptr(),
                    ::miniextendr_api::AltrepPkgName::as_ptr(),
                    core::ptr::null_mut(),
                ) },
                "String" => quote::quote! { ::miniextendr_api::ffi::altrep::R_make_altstring_class(
                    <#ident as ::miniextendr_api::altrep::AltrepClass>::CLASS_NAME.as_ptr(),
                    ::miniextendr_api::AltrepPkgName::as_ptr(),
                    core::ptr::null_mut(),
                ) },
                "List" => quote::quote! { ::miniextendr_api::ffi::altrep::R_make_altlist_class(
                    <#ident as ::miniextendr_api::altrep::AltrepClass>::CLASS_NAME.as_ptr(),
                    ::miniextendr_api::AltrepPkgName::as_ptr(),
                    core::ptr::null_mut(),
                ) },
                "Complex" => {
                    quote::quote! { ::miniextendr_api::ffi::altrep::R_make_altcomplex_class(
                        <#ident as ::miniextendr_api::altrep::AltrepClass>::CLASS_NAME.as_ptr(),
                        ::miniextendr_api::AltrepPkgName::as_ptr(),
                        core::ptr::null_mut(),
                    ) }
                }
                _ => quote::quote! { unreachable!() },
            };
            (setters, make)
        } else {
            // No explicit base: use InferBase trait for compile-time dispatch
            // This is auto-implemented via impl_inferbase_* macros alongside impl_alt*_from_data!
            let setters = quote::quote! {
                // SAFETY: Called during R initialization while holding exclusive R access
                unsafe { <#tramp_ty as ::miniextendr_api::altrep_data::InferBase>::install_methods(cls); }
            };
            let make = quote::quote! {
                <#tramp_ty as ::miniextendr_api::altrep_data::InferBase>::make_class(
                    <#ident as ::miniextendr_api::altrep::AltrepClass>::CLASS_NAME.as_ptr(),
                    ::miniextendr_api::AltrepPkgName::as_ptr(),
                )
            };
            (setters, make)
        };

    // Registration: per-type; create class handle then install methods via MethodRegistrar

    let (_impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    // Use LitCStr for proper C string literal generation (c"...")
    let class_cstr = syn::LitCStr::new(
        &std::ffi::CString::new(class_name.as_str()).unwrap(),
        ident.span(),
    );
    // pkg_name is now obtained at runtime via AltrepPkgName::as_ptr()

    // No trait forwarding: rely on trampoline type's trait impls.
    // The ALTREP trait implementations for the data type must already exist.
    // For standard types like Vec<i32>, Vec<f64>, these are provided by miniextendr_api.
    // For custom types, users must implement the data traits themselves.

    // Generate From, IntoR, and TryFromSexp wrappers
    let ref_ident = quote::format_ident!("{}Ref", ident);
    let mut_ident = quote::format_ident!("{}Mut", ident);
    let data_helper_impl: proc_macro2::TokenStream = {
        let ref_doc = format!(
            "Immutable reference wrapper for [`{}`] ALTREP data. Implements `TryFromSexp` and `Deref<Target = {}>`.",
            ident,
            quote::quote!(#data_ty)
        );
        let mut_doc = format!(
            "Mutable reference wrapper for [`{}`] ALTREP data. Implements `TryFromSexp`, `Deref`, and `DerefMut`.",
            ident
        );
        let from_doc = format!(
            "Create a [`{}`] ALTREP wrapper from the inner data type.",
            ident
        );
        let into_r_doc = format!(
            "Convert [`{}`] to an R ALTREP SEXP.\n\nIn debug builds, asserts that we're on R's main thread.",
            ident
        );
        quote::quote! {
            #[doc = #from_doc]
            impl From<#data_ty> for #ident {
                fn from(data: #data_ty) -> Self {
                    Self(data)
                }
            }

            #[doc = #into_r_doc]
            impl ::miniextendr_api::IntoR for #ident {
                fn into_sexp(self) -> ::miniextendr_api::ffi::SEXP {
                    use ::miniextendr_api::altrep_registration::RegisterAltrep;
                    use ::miniextendr_api::externalptr::ExternalPtr;
                    use ::miniextendr_api::ffi::altrep::R_new_altrep;
                    use ::miniextendr_api::ffi::R_NilValue;
                    use ::miniextendr_api::ffi::{Rf_protect, Rf_unprotect};

                    let ext_ptr = ExternalPtr::new(self.0);
                    let cls = Self::get_or_init_class();
                    // Protect data1 during R_new_altrep to prevent GC from collecting it.
                    // R_new_altrep allocates and can trigger GC.
                    let data1 = ext_ptr.as_sexp();
                    unsafe {
                        Rf_protect(data1);
                        let altrep = R_new_altrep(cls, data1, R_NilValue);
                        Rf_unprotect(1);
                        altrep
                    }
                }
            }

            #[doc = #ref_doc]
            pub struct #ref_ident(::miniextendr_api::externalptr::ExternalPtr<#data_ty>);

            #[allow(clippy::not_unsafe_ptr_arg_deref)]
            impl ::miniextendr_api::TryFromSexp for #ref_ident {
                type Error = ::miniextendr_api::SexpTypeError;

                fn try_from_sexp(sexp: ::miniextendr_api::ffi::SEXP) -> Result<Self, Self::Error> {
                    use ::miniextendr_api::ffi::SEXPTYPE;

                    // Check if it's an ALTREP object (ALTREP returns c_int, not bool)
                    if unsafe { ::miniextendr_api::ffi::ALTREP(sexp) } == 0 {
                        return Err(::miniextendr_api::SexpTypeError {
                            expected: SEXPTYPE::INTSXP, // placeholder - ALTREP check failed
                            actual: unsafe { ::miniextendr_api::ffi::TYPEOF(sexp) },
                        });
                    }

                    // Extract the ExternalPtr from data1
                    match unsafe { ::miniextendr_api::altrep_data1_as::<#data_ty>(sexp) } {
                        Some(ptr) => Ok(#ref_ident(ptr)),
                        None => Err(::miniextendr_api::SexpTypeError {
                            expected: SEXPTYPE::EXTPTRSXP,
                            actual: unsafe { ::miniextendr_api::ffi::TYPEOF(sexp) },
                        }),
                    }
                }
            }

            impl std::ops::Deref for #ref_ident {
                type Target = #data_ty;

                fn deref(&self) -> &Self::Target {
                    &*self.0
                }
            }

            #[doc = #mut_doc]
            pub struct #mut_ident(::miniextendr_api::externalptr::ExternalPtr<#data_ty>);

            #[allow(clippy::not_unsafe_ptr_arg_deref)]
            impl ::miniextendr_api::TryFromSexp for #mut_ident {
                type Error = ::miniextendr_api::SexpTypeError;

                fn try_from_sexp(sexp: ::miniextendr_api::ffi::SEXP) -> Result<Self, Self::Error> {
                    use ::miniextendr_api::ffi::SEXPTYPE;

                    // Check if it's an ALTREP object (ALTREP returns c_int, not bool)
                    if unsafe { ::miniextendr_api::ffi::ALTREP(sexp) } == 0 {
                        return Err(::miniextendr_api::SexpTypeError {
                            expected: SEXPTYPE::INTSXP, // placeholder - ALTREP check failed
                            actual: unsafe { ::miniextendr_api::ffi::TYPEOF(sexp) },
                        });
                    }

                    // Extract the ExternalPtr from data1
                    match unsafe { ::miniextendr_api::altrep_data1_as::<#data_ty>(sexp) } {
                        Some(ptr) => Ok(#mut_ident(ptr)),
                        None => Err(::miniextendr_api::SexpTypeError {
                            expected: SEXPTYPE::EXTPTRSXP,
                            actual: unsafe { ::miniextendr_api::ffi::TYPEOF(sexp) },
                        }),
                    }
                }
            }

            impl std::ops::Deref for #mut_ident {
                type Target = #data_ty;

                fn deref(&self) -> &Self::Target {
                    &*self.0
                }
            }

            impl std::ops::DerefMut for #mut_ident {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut *self.0
                }
            }
        }
    };

    // Generate doc strings for trait impls
    let base_doc = base_name
        .as_ref()
        .map(|b| b.to_string())
        .unwrap_or_else(|| "inferred".to_string());
    let altrep_class_doc = format!(
        "ALTREP class descriptor for [`{}`] (class: `{}`, base: `{}`).",
        ident, class_name, base_doc
    );
    let register_altrep_doc = format!("Registration entry point for [`{}`] ALTREP class.", ident);

    let method_registrar_install_body: proc_macro2::TokenStream = if base_name.is_some() {
        quote::quote! {
            // Base: length is ALWAYS required (no HAS_LENGTH check)
            unsafe { R_set_altrep_Length_method(cls, Some(bridge::t_length::<#tramp_ty>)); }

            // Base optional methods
            set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::Altrep>::HAS_SERIALIZED_STATE, R_set_altrep_Serialized_state_method, bridge::t_serialized_state::<#tramp_ty>);
            set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::Altrep>::HAS_UNSERIALIZE, R_set_altrep_Unserialize_method, bridge::t_unserialize::<#tramp_ty>);
            set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::Altrep>::HAS_DUPLICATE, R_set_altrep_Duplicate_method, bridge::t_duplicate::<#tramp_ty>);
            set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::Altrep>::HAS_COERCE, R_set_altrep_Coerce_method, bridge::t_coerce::<#tramp_ty>);
            set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::Altrep>::HAS_INSPECT, R_set_altrep_Inspect_method, bridge::t_inspect::<#tramp_ty>);

            // Vec-level setters
            set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltVec>::HAS_DATAPTR, R_set_altvec_Dataptr_method, bridge::t_dataptr::<#tramp_ty>);
            set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltVec>::HAS_DATAPTR_OR_NULL, R_set_altvec_Dataptr_or_null_method, bridge::t_dataptr_or_null::<#tramp_ty>);
            set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::AltVec>::HAS_EXTRACT_SUBSET, R_set_altvec_Extract_subset_method, bridge::t_extract_subset::<#tramp_ty>);

            // Family-specific
            #family_setters
        }
    } else {
        // Inferred base: the InferBase installer wires up base + vec + family methods.
        quote::quote! { #family_setters }
    };

    let expanded = quote::quote! {
        #input

        // Helper methods for creating ALTREP instances
        #data_helper_impl

        #[doc = #altrep_class_doc]
        #[allow(clippy::not_unsafe_ptr_arg_deref)]
        impl ::miniextendr_api::altrep::AltrepClass for #ident #ty_generics #where_clause {
            const CLASS_NAME: &'static std::ffi::CStr = #class_cstr;
            const BASE: ::miniextendr_api::altrep::RBase = #base_variant;
            unsafe fn length(x: ::miniextendr_api::ffi::SEXP) -> ::miniextendr_api::ffi::R_xlen_t {
                <#tramp_ty as ::miniextendr_api::altrep_traits::Altrep>::length(x)
            }
        }

        #[doc = #register_altrep_doc]
        impl ::miniextendr_api::altrep_registration::RegisterAltrep for #ident #ty_generics #where_clause {
            fn get_or_init_class() -> ::miniextendr_api::ffi::altrep::R_altrep_class_t {
                use std::sync::OnceLock;
                static CLASS: OnceLock<::miniextendr_api::ffi::altrep::R_altrep_class_t> = OnceLock::new();
                *CLASS.get_or_init(move || {
                    let cls = unsafe { #make_class };
                    // Install ALTREP methods
                    {
                        #[allow(unused_imports)]
                        use ::miniextendr_api::altrep_bridge as bridge;
                        #[allow(unused_imports)]
                        use ::miniextendr_api::ffi::altrep::*;
                        // Local helper to reduce boilerplate
                        #[allow(unused_macros)]
                        macro_rules! set_if { ($cond:expr, $setter:path, $tramp:expr) => { if $cond { unsafe { $setter(cls, Some($tramp)) } } } }
                        #method_registrar_install_body
                    }
                    cls
                })
            }
        }

    };

    expanded.into()
}
