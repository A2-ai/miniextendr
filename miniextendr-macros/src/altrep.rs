//! ALTREP registration code generation.
//!
//! This module generates the full ALTREP registration stack for data structs:
//! `TypedExternal`, `AltrepClass`, `RegisterAltrep`, `IntoR`, linkme entry,
//! and `Ref`/`Mut` accessor types.
//!
//! # Usage
//!
//! For types with field-based derives (auto-generates trait impls):
//! ```ignore
//! #[derive(AltrepInteger)]
//! #[altrep(len = "len", elt = "value", class = "MyConstInt")]
//! struct MyConstInt { value: i32, len: usize }
//! ```
//!
//! For types with manual trait impls (registration only):
//! ```ignore
//! #[derive(Altrep)]
//! #[altrep_derive_opts(class = "MyCustom")]
//! struct MyCustomData { ... }
//!
//! impl AltrepLen for MyCustomData { ... }
//! impl AltIntegerData for MyCustomData { ... }
//! impl_altinteger_from_data!(MyCustomData);
//! ```

/// Generates full ALTREP registration for a data struct.
///
/// Generates TypedExternal, AltrepClass, RegisterAltrep, IntoR, linkme entry, and Ref/Mut.
/// The struct must already implement the low-level ALTREP traits (via `impl_alt*_from_data!`
/// or `#[derive(AltrepInteger)]`) and `InferBase`.
pub(crate) fn generate_direct_altrep_registration(
    ident: &syn::Ident,
    generics: &syn::Generics,
    class_name: &str,
) -> syn::Result<proc_macro2::TokenStream> {
    let (_impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Class name as CStr literal
    let class_cstr = syn::LitCStr::new(&std::ffi::CString::new(class_name).unwrap(), ident.span());

    // TypedExternal constants — needed for ExternalPtr<T> to work
    let type_name_str = class_name;
    let type_name_bytes = format!("{}\0", type_name_str);
    let type_name_byte_lit = syn::LitByteStr::new(type_name_bytes.as_bytes(), ident.span());

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

    // For non-generic types, emit a distributed_slice ALTREP registration entry
    let altrep_reg_entry = if generics.params.is_empty() {
        let reg_fn_name =
            quote::format_ident!("__mx_altrep_reg_{}", ident.to_string().to_lowercase());
        quote::quote! {
            #[::miniextendr_api::linkme::distributed_slice(::miniextendr_api::registry::MX_ALTREP_REGISTRATIONS)]
            #[linkme(crate = ::miniextendr_api::linkme)]
            #[doc(hidden)]
            fn #reg_fn_name() {
                <#ident as ::miniextendr_api::altrep_registration::RegisterAltrep>::get_or_init_class();
            }
        }
    } else {
        quote::quote! {}
    };

    let source_loc_doc = crate::source_location_doc(ident.span());

    Ok(quote::quote! {
        // TypedExternal — enables ExternalPtr<T> storage.
        // NOTE: We intentionally do NOT implement IntoExternalPtr, because ALTREP types
        // have their own IntoR impl that creates an ALTREP SEXP (not a plain ExternalPtr).
        impl ::miniextendr_api::externalptr::TypedExternal for #ident #ty_generics #where_clause {
            const TYPE_NAME: &'static str = #type_name_str;
            const TYPE_NAME_CSTR: &'static [u8] = #type_name_byte_lit;
            const TYPE_ID_CSTR: &'static [u8] =
                concat!(module_path!(), "::", stringify!(#ident), "\0").as_bytes();
        }

        // AltrepClass — class name and base type
        #[doc = concat!("ALTREP class descriptor for [`", stringify!(#ident), "`].")]
        #[doc = #source_loc_doc]
        impl ::miniextendr_api::altrep::AltrepClass for #ident #ty_generics #where_clause {
            const CLASS_NAME: &'static ::core::ffi::CStr = #class_cstr;
            const BASE: ::miniextendr_api::altrep::RBase =
                <#ident #ty_generics as ::miniextendr_api::altrep_data::InferBase>::BASE;
        }

        // RegisterAltrep — OnceLock class registration via InferBase
        #[doc = concat!("Registration entry point for [`", stringify!(#ident), "`] ALTREP class.")]
        #[doc = #source_loc_doc]
        impl ::miniextendr_api::altrep_registration::RegisterAltrep for #ident #ty_generics #where_clause {
            fn get_or_init_class() -> ::miniextendr_api::ffi::altrep::R_altrep_class_t {
                use ::std::sync::OnceLock;
                static CLASS: OnceLock<::miniextendr_api::ffi::altrep::R_altrep_class_t> = OnceLock::new();
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
        }

        // IntoR — convert to R ALTREP SEXP (wraps self in ExternalPtr)
        #[doc = #into_r_doc]
        impl ::miniextendr_api::IntoR for #ident #ty_generics #where_clause {
            type Error = ::core::convert::Infallible;

            fn try_into_sexp(self) -> ::core::result::Result<::miniextendr_api::ffi::SEXP, Self::Error> {
                Ok(self.into_sexp())
            }

            unsafe fn try_into_sexp_unchecked(self) -> ::core::result::Result<::miniextendr_api::ffi::SEXP, Self::Error> {
                Ok(unsafe { self.into_sexp_unchecked() })
            }

            fn into_sexp(self) -> ::miniextendr_api::ffi::SEXP {
                use ::miniextendr_api::altrep_registration::RegisterAltrep;
                use ::miniextendr_api::externalptr::ExternalPtr;
                use ::miniextendr_api::ffi::{SEXP, Rf_protect, Rf_unprotect};

                let ext_ptr = ExternalPtr::new(self);
                let cls = Self::get_or_init_class();
                let data1 = ext_ptr.as_sexp();
                unsafe {
                    Rf_protect(data1);
                    let altrep = cls.new_altrep(data1, SEXP::nil());
                    Rf_unprotect(1);
                    altrep
                }
            }

            unsafe fn into_sexp_unchecked(self) -> ::miniextendr_api::ffi::SEXP {
                use ::miniextendr_api::altrep_registration::RegisterAltrep;
                use ::miniextendr_api::externalptr::ExternalPtr;
                use ::miniextendr_api::ffi::{Rf_protect_unchecked, Rf_unprotect_unchecked};

                let ext_ptr = ExternalPtr::new_unchecked(self);
                let cls = Self::get_or_init_class();
                let data1 = ext_ptr.as_sexp();
                unsafe {
                    Rf_protect_unchecked(data1);
                    let altrep = cls.new_altrep_unchecked(
                        data1,
                        ::miniextendr_api::ffi::SEXP::nil(),
                    );
                    Rf_unprotect_unchecked(1);
                    altrep
                }
            }
        }

        // Ref/Mut accessor types for receiving ALTREP back from R
        #[doc = #ref_doc]
        pub struct #ref_ident(::miniextendr_api::externalptr::ExternalPtr<#ident #ty_generics>);

        impl ::miniextendr_api::TryFromSexp for #ref_ident {
            type Error = ::miniextendr_api::SexpTypeError;

            fn try_from_sexp(sexp: ::miniextendr_api::ffi::SEXP) -> ::core::result::Result<Self, Self::Error> {
                use ::miniextendr_api::ffi::SEXPTYPE;

                if !::miniextendr_api::ffi::SexpExt::is_altrep(&sexp) {
                    return Err(::miniextendr_api::SexpTypeError {
                        expected: SEXPTYPE::INTSXP,
                        actual: ::miniextendr_api::ffi::SexpExt::type_of(&sexp),
                    });
                }

                match unsafe { ::miniextendr_api::altrep_ext::AltrepSexpExt::altrep_data1::<#ident #ty_generics>(&sexp) } {
                    Some(ptr) => Ok(#ref_ident(ptr)),
                    None => Err(::miniextendr_api::SexpTypeError {
                        expected: SEXPTYPE::EXTPTRSXP,
                        actual: ::miniextendr_api::ffi::SexpExt::type_of(&sexp),
                    }),
                }
            }
        }

        impl ::core::ops::Deref for #ref_ident {
            type Target = #ident #ty_generics;

            fn deref(&self) -> &Self::Target {
                &*self.0
            }
        }

        #[doc = #mut_doc]
        pub struct #mut_ident(::miniextendr_api::externalptr::ExternalPtr<#ident #ty_generics>);

        impl ::miniextendr_api::TryFromSexp for #mut_ident {
            type Error = ::miniextendr_api::SexpTypeError;

            fn try_from_sexp(sexp: ::miniextendr_api::ffi::SEXP) -> ::core::result::Result<Self, Self::Error> {
                use ::miniextendr_api::ffi::SEXPTYPE;

                if !::miniextendr_api::ffi::SexpExt::is_altrep(&sexp) {
                    return Err(::miniextendr_api::SexpTypeError {
                        expected: SEXPTYPE::INTSXP,
                        actual: ::miniextendr_api::ffi::SexpExt::type_of(&sexp),
                    });
                }

                match unsafe { ::miniextendr_api::altrep_ext::AltrepSexpExt::altrep_data1::<#ident #ty_generics>(&sexp) } {
                    Some(ptr) => Ok(#mut_ident(ptr)),
                    None => Err(::miniextendr_api::SexpTypeError {
                        expected: SEXPTYPE::EXTPTRSXP,
                        actual: ::miniextendr_api::ffi::SexpExt::type_of(&sexp),
                    }),
                }
            }
        }

        impl ::core::ops::Deref for #mut_ident {
            type Target = #ident #ty_generics;

            fn deref(&self) -> &Self::Target {
                &*self.0
            }
        }

        impl ::core::ops::DerefMut for #mut_ident {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut *self.0
            }
        }

        #altrep_reg_entry
    })
}

/// Entry point for `#[derive(Altrep)]`.
///
/// Generates ALTREP registration for a data struct (TypedExternal, AltrepClass,
/// RegisterAltrep, IntoR, linkme entry, Ref/Mut accessor types).
///
/// The struct must already have low-level ALTREP traits implemented (via
/// `impl_alt*_from_data!` or a family-specific derive like `#[derive(AltrepInteger)]`).
///
/// # Helper attributes
///
/// ```ignore
/// #[altrep(class = "CustomName")]  // override ALTREP class name (default: struct name)
/// ```
///
/// The legacy `#[altrep_derive_opts(class = "...")]` syntax is also accepted.
pub fn derive_altrep(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    use syn::spanned::Spanned;

    let ident = &input.ident;

    if !matches!(input.data, syn::Data::Struct(_)) {
        return Err(syn::Error::new(
            input.span(),
            "#[derive(Altrep)] can only be applied to structs",
        ));
    }

    // Parse class name from either #[altrep(class = "...")] or #[altrep_derive_opts(class = "...")]
    let mut class_name = None::<String>;

    for attr in &input.attrs {
        let is_altrep = attr.path().is_ident("altrep");
        let is_opts = attr.path().is_ident("altrep_derive_opts");
        if !is_altrep && !is_opts {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("class") {
                let value: syn::LitStr = meta.value()?.parse()?;
                class_name = Some(value.value());
            } else {
                return Err(meta.error("unknown attribute; expected `class`"));
            }
            Ok(())
        })?;
    }

    let class_name = class_name.unwrap_or_else(|| ident.to_string());

    generate_direct_altrep_registration(ident, &input.generics, &class_name)
}
