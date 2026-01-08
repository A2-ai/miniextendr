//! ALTREP code generation for `miniextendr_module!`.
//!
//! When the module contains `impl AltIntegerData for Type;` (or similar ALTREP traits),
//! this generates:
//! - Macro invocation for trait implementations (via `impl_alt*_from_data!`)
//! - Registration infrastructure (RegisterAltrep)
//! - IntoR implementation for returning ALTREP values from functions
//!
//! The actual trait implementations are handled by existing macros in miniextendr-api,
//! which also implement `InferBase` for class creation and method registration.
//!
//! # Supported types
//!
//! Both simple types and generic types are supported:
//! - `impl AltIntegerData for MyType;` - simple type
//! - `impl AltIntegerData for Vec<i32>;` - generic type
//! - `impl AltIntegerData for Range<i32>;` - generic type

use crate::miniextendr_module::{AltrepBase, MiniextendrModuleTraitImpl};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

/// Information about an ALTREP type to generate code for.
pub(crate) struct AltrepTypeInfo<'a> {
    /// The concrete type implementing the ALTREP traits.
    /// This can be a simple type (`MyType`) or generic (`Vec<i32>`).
    pub impl_type: &'a syn::Type,
    /// The ALTREP base type (Integer, Real, etc.).
    pub base: AltrepBase,
    /// cfg attributes from the module declaration.
    pub cfg_attrs: Vec<&'a syn::Attribute>,
}

impl<'a> AltrepTypeInfo<'a> {
    /// Returns a sanitized class name for this ALTREP type.
    ///
    /// Converts generic types like `Vec<i32>` to `Vec_i32` for use as the ALTREP class name.
    pub(crate) fn class_name(&self) -> String {
        let type_str = self.impl_type.to_token_stream().to_string();
        // Replace special characters with underscores for valid class names
        type_str
            .replace(['<', '>', ' ', ':'], "_")
            .replace(',', "_")
            .replace("__", "_")
            .trim_matches('_')
            .to_string()
    }
}

/// Generate all ALTREP-related code for the module.
///
/// Returns a tuple of:
/// - Token stream with macro invocations and registration code
/// - List of registration expressions to call at init time
pub(crate) fn generate_altrep_code(
    altrep_impls: &[AltrepTypeInfo<'_>],
) -> (TokenStream, Vec<syn::Expr>) {
    let mut all_code = TokenStream::new();
    let mut registration_exprs = Vec::new();

    for info in altrep_impls {
        let impl_type = info.impl_type;
        let cfg_attrs = &info.cfg_attrs;

        // Generate macro invocation for trait implementations
        let macro_invocation = generate_macro_invocation(info);

        // Generate RegisterAltrep impl (uses InferBase from the macro)
        let register_impl = generate_register_altrep(info);

        // Generate IntoR impl
        let into_r_impl = generate_into_r(info);

        // Combine all generated code with cfg attributes
        all_code.extend(quote! {
            #(#cfg_attrs)*
            #macro_invocation

            #(#cfg_attrs)*
            #register_impl

            #(#cfg_attrs)*
            #into_r_impl
        });

        // Add registration expression
        let reg_expr: syn::Expr = syn::parse_quote! {
            <#impl_type as ::miniextendr_api::altrep_registration::RegisterAltrep>::get_or_init_class()
        };
        registration_exprs.push(reg_expr);
    }

    (all_code, registration_exprs)
}

/// Generate the macro invocation for trait implementations.
///
/// This delegates to the existing `impl_alt*_from_data!` macros which handle:
/// - Low-level trait impls (Altrep, AltVec, Alt*)
/// - InferBase impl (make_class, install_methods)
fn generate_macro_invocation(info: &AltrepTypeInfo<'_>) -> TokenStream {
    let ty = info.impl_type;

    match info.base {
        AltrepBase::Integer => quote! {
            ::miniextendr_api::impl_altinteger_from_data!(#ty);
        },
        AltrepBase::Real => quote! {
            ::miniextendr_api::impl_altreal_from_data!(#ty);
        },
        AltrepBase::Logical => quote! {
            ::miniextendr_api::impl_altlogical_from_data!(#ty);
        },
        AltrepBase::Raw => quote! {
            ::miniextendr_api::impl_altraw_from_data!(#ty);
        },
        AltrepBase::String => quote! {
            ::miniextendr_api::impl_altstring_from_data!(#ty);
        },
        AltrepBase::Complex => quote! {
            ::miniextendr_api::impl_altcomplex_from_data!(#ty);
        },
        AltrepBase::List => quote! {
            ::miniextendr_api::impl_altlist_from_data!(#ty);
        },
    }
}

/// Generate RegisterAltrep implementation.
///
/// This uses `InferBase::make_class()` and `InferBase::install_methods()` which are
/// provided by the `impl_inferbase_*!` macros (called by `impl_alt*_from_data!`).
fn generate_register_altrep(info: &AltrepTypeInfo<'_>) -> TokenStream {
    let ty = info.impl_type;
    // Use sanitized class name for generic types (e.g., "Vec_i32" for "Vec<i32>")
    let class_name = info.class_name();
    // Create a null-terminated byte string for the class name
    let class_name_cstr = format!("{}\0", class_name);
    let class_name_bytes = proc_macro2::Literal::byte_string(class_name_cstr.as_bytes());

    quote! {
        impl ::miniextendr_api::altrep_registration::RegisterAltrep for #ty {
            fn get_or_init_class() -> ::miniextendr_api::ffi::altrep::R_altrep_class_t {
                use std::sync::OnceLock;
                static CLASS: OnceLock<::miniextendr_api::ffi::altrep::R_altrep_class_t> = OnceLock::new();
                *CLASS.get_or_init(move || {
                    // Class name as null-terminated C string
                    const CLASS_NAME: &[u8] = #class_name_bytes;
                    let cls = unsafe {
                        <#ty as ::miniextendr_api::altrep_data::InferBase>::make_class(
                            CLASS_NAME.as_ptr() as *const std::ffi::c_char,
                            // Package name is set globally by C entrypoint
                            ::miniextendr_api::AltrepPkgName::as_ptr(),
                        )
                    };
                    unsafe {
                        <#ty as ::miniextendr_api::altrep_data::InferBase>::install_methods(cls);
                    }
                    cls
                })
            }
        }
    }
}

/// Generate IntoR implementation for ALTREP type.
fn generate_into_r(info: &AltrepTypeInfo<'_>) -> TokenStream {
    let ty = info.impl_type;

    quote! {
        impl ::miniextendr_api::IntoR for #ty {
            fn into_r(self) -> ::miniextendr_api::ffi::SEXP {
                let cls = <#ty as ::miniextendr_api::altrep_registration::RegisterAltrep>::get_or_init_class();
                let data1 = <#ty as ::miniextendr_api::externalptr::TypedExternal>::wrap(self);
                unsafe {
                    ::miniextendr_api::ffi::altrep::R_new_altrep(
                        cls,
                        data1,
                        ::miniextendr_api::ffi::SEXP::null(),
                    )
                }
            }
        }
    }
}

/// Extract ALTREP trait impls from the module's trait_impls.
pub(crate) fn extract_altrep_impls<'a>(
    trait_impls: &'a [MiniextendrModuleTraitImpl],
) -> (Vec<AltrepTypeInfo<'a>>, Vec<&'a MiniextendrModuleTraitImpl>) {
    let mut altrep_impls = Vec::new();
    let mut other_impls = Vec::new();

    for ti in trait_impls {
        if let Some(base) = ti.altrep_base() {
            altrep_impls.push(AltrepTypeInfo {
                impl_type: &ti.impl_type,
                base,
                cfg_attrs: ti
                    .attrs
                    .iter()
                    .filter(|a| a.path().is_ident("cfg"))
                    .collect(),
            });
        } else {
            other_impls.push(ti);
        }
    }

    (altrep_impls, other_impls)
}
