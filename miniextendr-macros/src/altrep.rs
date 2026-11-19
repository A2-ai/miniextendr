//! ALTREP registration code generation, and `#[derive(Altrep)]`.
//!
//! This module generates the full ALTREP registration stack for data structs:
//! `TypedExternal`, `AltrepClass`, `RegisterAltrep`, `IntoR`, linkme entry,
//! and `Ref`/`Mut` accessor types.
//!
//! # Usage
//!
//! `#[derive(Altrep)]` reads a struct's slice field, and the element type
//! picks the R vector (an `RFactor` enum makes a factor, `String` a
//! character vector, `i32` an integer vector R reads in place, ...):
//! ```ignore
//! #[derive(Altrep)]
//! struct Dice<const N: usize>([Die; N]);
//! ```
//!
//! For types with field-based family derives (auto-generates trait impls):
//! ```ignore
//! #[derive(AltrepInteger)]
//! #[altrep(len = "len", elt = "value", class = "MyConstInt")]
//! struct MyConstInt { value: i32, len: usize }
//! ```
//!
//! For types with manual trait impls (lowlevel + registration, user writes data traits):
//! ```ignore
//! #[derive(AltrepInteger)]
//! #[altrep(manual, class = "MyCustom", serialize)]
//! struct MyCustomData { ... }
//!
//! impl AltrepLen for MyCustomData { ... }
//! impl AltIntegerData for MyCustomData { ... }
//! // Family derived from AltrepInteger — generates Altrep, AltVec, AltInteger, InferBase.
//! ```

use crate::altrep_derive::shape::{self, Selector};
use quote::quote;

/// Generates full ALTREP registration for a data struct.
///
/// Generates TypedExternal, AltrepClass, RegisterAltrep, IntoR, linkme entry, and Ref/Mut.
/// The struct must already implement the low-level ALTREP traits (via `impl_alt*_from_data!`
/// or `#[derive(AltrepInteger)]`) and `InferBase`. `input.generics` are the
/// ones [`shape::altrep_generics`] returns.
///
/// A concrete type registers its class at package init (a linkme entry); a
/// generic one registers a class per instantiation on first conversion,
/// named `class_name` and the parameters (`Dice_5`), through
/// `generic_altrep_class`.
///
/// `slice` is set for a `#[derive(Altrep)]` slice owner, whose conversion
/// gives the new vector its element's attributes (a factor's `levels` and
/// `class`).
pub(crate) fn generate_direct_altrep_registration(
    input: &syn::DeriveInput,
    class_name: &str,
    slice: bool,
) -> syn::Result<proc_macro2::TokenStream> {
    let ident = &input.ident;
    let visibility = &input.vis;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let storage = crate::externalptr_derive::generate_altrep_storage(input, class_name)?;

    // Class name as CStr literal
    let class_cstr = syn::LitCStr::new(&std::ffi::CString::new(class_name).unwrap(), ident.span());

    let ref_ident = quote::format_ident!("{}Ref", ident);
    let mut_ident = quote::format_ident!("{}Mut", ident);

    let into_r_doc = format!(
        "Convert [`{}`] to an R ALTREP SEXP.\n\nIn debug builds, asserts that we're on R's main thread.",
        ident
    );
    let ref_doc = format!(
        "Immutable reference wrapper for [`{}`] ALTREP data. Implements `TryFromSexp` and `Deref<Target = {}>`.",
        ident, ident
    );
    let mut_doc = format!(
        "Mutable reference wrapper for [`{}`] ALTREP data. Implements `TryFromSexp`, `Deref`, and `DerefMut`.",
        ident
    );

    // For non-generic types, emit a registration fn + a distributed_slice
    // entry pairing the fn pointer with its `#[no_mangle]` symbol name.
    //
    // The function is `pub extern "C-unwind"` with `#[unsafe(no_mangle)]` so that a
    // separate compilation unit (the WASM snapshot codegen path) can reference it
    // through a matching extern declaration. The unwind ABI lets registration
    // panics reach package initialization's guard instead of aborting at this shim.
    // The symbol is crate-prefixed for webR cross-package uniqueness (#1273).
    // The entry static carries the symbol string for the host-time snapshot
    // writer, so it does not have to recover the name from a fn pointer.
    //
    // The ident is used verbatim (no `.to_lowercase()`) to avoid case-collision footguns:
    // `MyType` vs `MYType` would both produce the same symbol name if lowercased.
    let altrep_reg_entry = if shape::is_generic(generics) {
        quote! {}
    } else {
        let reg_fn_name = crate::naming::altrep_reg_fn_ident(ident);
        let entry_ident = quote::format_ident!("__MX_ALTREP_REG_ENTRY_{}", ident);
        quote! {
            #[doc(hidden)]
            #[unsafe(no_mangle)]
            pub extern "C-unwind" fn #reg_fn_name() {
                <#ident as ::miniextendr_api::altrep_registration::RegisterAltrep>::get_or_init_class();
            }

            #[cfg_attr(not(target_arch = "wasm32"), ::miniextendr_api::linkme::distributed_slice(::miniextendr_api::registry::MX_ALTREP_REGISTRATIONS), linkme(crate = ::miniextendr_api::linkme))]
            #[doc(hidden)]
            #[allow(non_upper_case_globals)]
            static #entry_ident: ::miniextendr_api::registry::AltrepRegistration =
                ::miniextendr_api::registry::AltrepRegistration {
                    register: #reg_fn_name,
                    symbol: ::core::stringify!(#reg_fn_name),
                };
        }
    };

    // One class per instantiation of a generic type: a `static` in a generic
    // fn would be shared by all of them.
    let get_or_init_class = if shape::is_generic(generics) {
        let parts = generics.params.iter().map(|param| match param {
            syn::GenericParam::Type(ty) => {
                let ty = &ty.ident;
                quote! { ::std::string::String::push_str(&mut __mx_name, ::core::any::type_name::<#ty>()); }
            }
            syn::GenericParam::Const(constant) => {
                let constant = &constant.ident;
                quote! { ::std::string::String::push_str(&mut __mx_name, &::std::string::ToString::to_string(&#constant)); }
            }
            syn::GenericParam::Lifetime(_) => quote! {},
        });
        quote! {
            ::miniextendr_api::altrep::generic_altrep_class::<Self>(|| {
                let mut __mx_name = ::std::string::String::from(#class_name);
                #(
                    ::std::string::String::push(&mut __mx_name, '_');
                    #parts
                )*
                __mx_name
            })
        }
    } else {
        quote! {
            use ::std::sync::OnceLock;
            static CLASS: OnceLock<::miniextendr_api::sys::altrep::R_altrep_class_t> = OnceLock::new();
            *CLASS.get_or_init(move || {
                let cls = unsafe {
                    <#ident as ::miniextendr_api::altrep_data::InferBase>::make_class(
                        <#ident as ::miniextendr_api::altrep::AltrepClass>::CLASS_NAME.as_ptr(),
                        ::miniextendr_api::AltrepPkgName::as_ptr(),
                    )
                };
                unsafe {
                    <#ident as ::miniextendr_api::altrep_data::InferBase>::install_methods(cls);
                }
                cls
            })
        }
    };

    // A slice owner's new vector takes its element's attributes (a factor's
    // `levels` and `class`). The ALTREP holds data1, so only it is protected.
    let attributes = slice.then(|| {
        quote! {
            unsafe {
                ::miniextendr_api::altrep_data::set_slice_altrep_attributes::<Self>(altrep);
            }
        }
    });

    let source_loc_doc = crate::source_location_doc(ident.span());

    Ok(quote! {
        // Shared ExternalPtr storage, including sidecars. No IntoExternalPtr
        // marker: conversion must produce the ALTREP, not its data1 pointer.
        #storage

        // AltrepClass — class name and base type. A generic type's classes
        // add their parameters to the name.
        #[doc = ::core::concat!("ALTREP class descriptor for [`", ::core::stringify!(#ident), "`].")]
        #[doc = #source_loc_doc]
        impl #impl_generics ::miniextendr_api::altrep::AltrepClass for #ident #ty_generics #where_clause {
            const CLASS_NAME: &'static ::core::ffi::CStr = #class_cstr;
            const BASE: ::miniextendr_api::altrep::RBase =
                <#ident #ty_generics as ::miniextendr_api::altrep_data::InferBase>::BASE;
        }

        // RegisterAltrep — class registration via InferBase
        #[doc = ::core::concat!("Registration entry point for [`", ::core::stringify!(#ident), "`] ALTREP class.")]
        #[doc = #source_loc_doc]
        impl #impl_generics ::miniextendr_api::altrep_registration::RegisterAltrep for #ident #ty_generics #where_clause {
            fn get_or_init_class() -> ::miniextendr_api::sys::altrep::R_altrep_class_t {
                #get_or_init_class
            }
        }

        // IntoR — convert to R ALTREP SEXP (wraps self in ExternalPtr)
        #[doc = #into_r_doc]
        impl #impl_generics ::miniextendr_api::IntoR for #ident #ty_generics #where_clause {
            type Error = ::core::convert::Infallible;

            fn try_into_sexp(self) -> ::core::result::Result<::miniextendr_api::SEXP, ::core::convert::Infallible> {
                ::core::result::Result::Ok(self.into_sexp())
            }

            unsafe fn try_into_sexp_unchecked(self) -> ::core::result::Result<::miniextendr_api::SEXP, ::core::convert::Infallible> {
                ::core::result::Result::Ok(unsafe { self.into_sexp_unchecked() })
            }

            fn into_sexp(self) -> ::miniextendr_api::SEXP {
                use ::miniextendr_api::altrep_registration::RegisterAltrep;
                use ::miniextendr_api::externalptr::ExternalPtr;
                use ::miniextendr_api::SEXP;
                use ::miniextendr_api::sys::{Rf_protect, Rf_unprotect};

                let ext_ptr = ExternalPtr::new(self);
                let cls = Self::get_or_init_class();
                let data1 = ext_ptr.as_sexp();
                let altrep = unsafe {
                    Rf_protect(data1);
                    let altrep = cls.new_altrep(data1, SEXP::nil());
                    Rf_unprotect(1);
                    altrep
                };
                #attributes
                altrep
            }

            unsafe fn into_sexp_unchecked(self) -> ::miniextendr_api::SEXP {
                use ::miniextendr_api::altrep_registration::RegisterAltrep;
                use ::miniextendr_api::externalptr::ExternalPtr;
                use ::miniextendr_api::sys::{Rf_protect_unchecked, Rf_unprotect_unchecked};

                let ext_ptr = ExternalPtr::new_unchecked(self);
                let cls = Self::get_or_init_class();
                let data1 = ext_ptr.as_sexp();
                let altrep = unsafe {
                    Rf_protect_unchecked(data1);
                    let altrep = cls.new_altrep_unchecked(
                        data1,
                        ::miniextendr_api::SEXP::nil(),
                    );
                    Rf_unprotect_unchecked(1);
                    altrep
                };
                #attributes
                altrep
            }
        }

        // Ref/Mut accessor types for receiving ALTREP back from R
        #[doc = #ref_doc]
        #visibility struct #ref_ident #impl_generics (::miniextendr_api::externalptr::ExternalPtr<#ident #ty_generics>) #where_clause;

        impl #impl_generics ::miniextendr_api::TryFromSexp for #ref_ident #ty_generics #where_clause {
            type Error = ::miniextendr_api::SexpTypeError;

            fn try_from_sexp(sexp: ::miniextendr_api::SEXP) -> ::core::result::Result<Self, ::miniextendr_api::SexpTypeError> {
                use ::miniextendr_api::SEXPTYPE;

                if !::miniextendr_api::SexpExt::is_altrep(&sexp) {
                    return ::core::result::Result::Err(::miniextendr_api::SexpTypeError {
                        expected: <#ident #ty_generics as ::miniextendr_api::altrep::AltrepClass>::BASE.sexptype(),
                        actual: ::miniextendr_api::SexpExt::type_of(&sexp),
                    });
                }

                match unsafe { ::miniextendr_api::altrep_data1_as::<#ident #ty_generics>(sexp) } {
                    ::core::option::Option::Some(ptr) => ::core::result::Result::Ok(#ref_ident(ptr)),
                    ::core::option::Option::None => ::core::result::Result::Err(::miniextendr_api::SexpTypeError {
                        expected: SEXPTYPE::EXTPTRSXP,
                        actual: ::miniextendr_api::SexpExt::type_of(&sexp),
                    }),
                }
            }
        }

        // ALTREP data1 reads take no `RefCell` borrow until #2103 measures
        // the checked read.
        impl #impl_generics ::core::ops::Deref for #ref_ident #ty_generics #where_clause {
            type Target = #ident #ty_generics;

            fn deref(&self) -> &Self::Target {
                unsafe { self.0.as_ref_unchecked() }
            }
        }

        #[doc = #mut_doc]
        #visibility struct #mut_ident #impl_generics (::miniextendr_api::externalptr::ExternalPtr<#ident #ty_generics>) #where_clause;

        impl #impl_generics ::miniextendr_api::TryFromSexp for #mut_ident #ty_generics #where_clause {
            type Error = ::miniextendr_api::SexpTypeError;

            fn try_from_sexp(sexp: ::miniextendr_api::SEXP) -> ::core::result::Result<Self, ::miniextendr_api::SexpTypeError> {
                use ::miniextendr_api::SEXPTYPE;

                if !::miniextendr_api::SexpExt::is_altrep(&sexp) {
                    return ::core::result::Result::Err(::miniextendr_api::SexpTypeError {
                        expected: <#ident #ty_generics as ::miniextendr_api::altrep::AltrepClass>::BASE.sexptype(),
                        actual: ::miniextendr_api::SexpExt::type_of(&sexp),
                    });
                }

                match unsafe { ::miniextendr_api::altrep_data1_as::<#ident #ty_generics>(sexp) } {
                    ::core::option::Option::Some(ptr) => ::core::result::Result::Ok(#mut_ident(ptr)),
                    ::core::option::Option::None => ::core::result::Result::Err(::miniextendr_api::SexpTypeError {
                        expected: SEXPTYPE::EXTPTRSXP,
                        actual: ::miniextendr_api::SexpExt::type_of(&sexp),
                    }),
                }
            }
        }

        impl #impl_generics ::core::ops::Deref for #mut_ident #ty_generics #where_clause {
            type Target = #ident #ty_generics;

            fn deref(&self) -> &Self::Target {
                unsafe { self.0.as_ref_unchecked() }
            }
        }

        impl #impl_generics ::core::ops::DerefMut for #mut_ident #ty_generics #where_clause {
            fn deref_mut(&mut self) -> &mut Self::Target {
                unsafe { self.0.as_mut_unchecked() }
            }
        }

        #altrep_reg_entry
    })
}

/// Entry point for `#[derive(Altrep)]`.
///
/// Reads the length and elements from the struct's slice field (`[T; N]`,
/// `Vec<T>`, `Box<[T]>`, `&'static [T]`; `#[altrep(field = "name")]` picks
/// one of several, a tuple index works), and implements the hidden
/// `AltrepSlice` for it: miniextendr-api's impls over `AltrepSlice` give the
/// low-level ALTREP traits, with the R vector type chosen by the element's
/// `AltrepElement` impl. Then the registration, as for the family derives.
///
/// `#[altrep(manual)]` generates the registration only (TypedExternal,
/// AltrepClass, RegisterAltrep, IntoR, linkme entry, Ref/Mut accessor
/// types), for a struct that implements the low-level ALTREP traits itself.
///
/// # Helper attributes
///
/// ```ignore
/// #[altrep(class = "CustomName")]  // override ALTREP class name (default: struct name)
/// #[altrep(field = "values")]      // the slice field, when there are several
/// #[altrep(manual)]                // registration only
/// ```
pub fn derive_altrep(mut input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let ident = &input.ident;
    match &input.data {
        syn::Data::Struct(_) => {}
        syn::Data::Enum(_) => {
            return Err(syn::Error::new(
                crate::util::data_keyword_span(&input.data),
                format!(
                    "`#[derive(Altrep)]` makes an R vector from a struct's slice field, and an \
                     enum is an element: derive `RFactor` on `{ident}` for a factor or \
                     `MatchArg` for a character vector, and put it in a field such as \
                     `[{ident}; N]` or `Vec<{ident}>`"
                ),
            ));
        }
        syn::Data::Union(_) => {
            return Err(syn::Error::new(
                crate::util::data_keyword_span(&input.data),
                "#[derive(Altrep)] can only be applied to structs",
            ));
        }
    }

    let mut class_name = None::<String>;
    let mut field = None::<Selector>;
    let mut manual = false;
    for attr in &input.attrs {
        if !attr.path().is_ident("altrep") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("class") {
                let value: syn::LitStr = meta.value()?.parse()?;
                class_name = Some(value.value());
            } else if meta.path.is_ident("field") {
                let value: syn::LitStr = meta.value()?.parse()?;
                field = Some(Selector::parse(&value)?);
            } else if meta.path.is_ident("manual") {
                manual = true;
            } else {
                return Err(meta.error(
                    "unknown #[altrep(...)] attribute; expected `class`, `field` or `manual` \
                     (the family derives, `#[derive(AltrepInteger)]` ..., take the other options)",
                ));
            }
            Ok(())
        })?;
    }

    input.generics = shape::altrep_generics(&input.generics)?;
    let class_name = class_name.unwrap_or_else(|| crate::naming::ident_name(&input.ident));

    if manual {
        if let Some(field) = field {
            return Err(syn::Error::new(
                field.span,
                "`field` reads the struct's slice, and `manual` means you implement the ALTREP \
                 traits yourself: use one",
            ));
        }
        return generate_direct_altrep_registration(&input, &class_name, false);
    }

    let Some(slice) = shape::slice_field(&input, field.as_ref(), "Altrep")? else {
        return Err(syn::Error::new(
            input.ident.span(),
            format!(
                "`#[derive(Altrep)]` reads a struct's slice field, and `{}` has none: give it a \
                 {} field, or implement the ALTREP traits yourself and add `#[altrep(manual)]`",
                input.ident,
                shape::SLICE_KINDS
            ),
        ));
    };
    if shape::has_type_params(&input.generics) {
        shape::add_bound(
            &mut input.generics,
            &slice.elem,
            quote! { ::miniextendr_api::altrep_data::AltrepElement },
        );
    }

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let elem = &slice.elem;
    let values = slice.values();
    let values_mut = match slice.values_mut() {
        Some(values_mut) => quote! { ::core::option::Option::Some(#values_mut) },
        None => quote! { ::core::option::Option::None },
    };
    let slice_impl = quote! {
        impl #impl_generics ::miniextendr_api::altrep_data::AltrepSlice for #name #ty_generics #where_clause {
            type Element = #elem;

            fn altrep_slice(&self) -> &[#elem] {
                #values
            }

            fn altrep_slice_mut(&mut self) -> ::core::option::Option<&mut [#elem]> {
                #values_mut
            }
        }
    };
    let registration = generate_direct_altrep_registration(&input, &class_name, true)?;
    Ok(quote! {
        #slice_impl
        #registration
    })
}

#[cfg(test)]
mod tests;
