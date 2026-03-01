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

/// Collected information about a single ALTREP type to generate registration code for.
///
/// Extracted from `impl Alt*Data for Type;` entries in `miniextendr_module!` by
/// [`extract_altrep_impls`].
pub(crate) struct AltrepTypeInfo<'a> {
    /// The concrete type implementing the ALTREP data traits.
    /// This can be a simple type (`MyType`) or a generic type (`Vec<i32>`, `Range<i32>`).
    pub impl_type: &'a syn::Type,
    /// The ALTREP base type family, determined by which `Alt*Data` trait is declared
    /// (e.g., `AltIntegerData` maps to `AltrepBase::Integer`).
    pub base: AltrepBase,
    /// Any `#[cfg(...)]` attributes from the module entry, propagated to all generated
    /// items so they are conditionally compiled in sync with the declaration.
    pub cfg_attrs: Vec<&'a syn::Attribute>,
}

impl<'a> AltrepTypeInfo<'a> {
    /// Returns a sanitized ALTREP class name derived from the Rust type.
    ///
    /// Generic types are normalized into valid C identifier-like strings by replacing
    /// angle brackets, colons, commas, and spaces with underscores. For example:
    /// - `MyType` stays `MyType`
    /// - `Vec<i32>` becomes `Vec_i32`
    /// - `std::ops::Range<i32>` becomes `std__ops__Range_i32`
    ///
    /// Consecutive underscores are collapsed and leading/trailing underscores are trimmed.
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

/// Generates all ALTREP-related code for a `miniextendr_module!` expansion.
///
/// For each ALTREP type, this produces:
/// 1. A `impl_alt*_from_data!` macro invocation for the low-level trait implementations
///    (`Altrep`, `AltVec`, family-specific traits, and `InferBase`).
/// 2. An `impl RegisterAltrep` block with a `OnceLock`-based lazy class initializer
///    that calls `InferBase::make_class()` and `InferBase::install_methods()`.
/// 3. An `impl IntoR` block for converting the Rust type into an R ALTREP SEXP.
///
/// All generated items inherit `#[cfg(...)]` attributes from the module declaration.
///
/// # Arguments
///
/// * `altrep_impls` -- Slice of [`AltrepTypeInfo`] extracted from the module's trait impls.
///
/// # Returns
///
/// A tuple of:
/// - A [`TokenStream`] containing all macro invocations, `RegisterAltrep`, and `IntoR` impls.
/// - A `Vec<syn::Expr>` of registration expressions (`T::get_or_init_class()`) to call
///   during R package initialization.
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

/// Generates the `impl_alt*_from_data!` macro invocation for a single ALTREP type.
///
/// Selects the correct macro based on [`AltrepBase`]:
/// - `Integer` -- `impl_altinteger_from_data!`
/// - `Real` -- `impl_altreal_from_data!`
/// - `Logical` -- `impl_altlogical_from_data!`
/// - `Raw` -- `impl_altraw_from_data!`
/// - `String` -- `impl_altstring_from_data!`
/// - `Complex` -- `impl_altcomplex_from_data!`
/// - `List` -- `impl_altlist_from_data!`
///
/// These runtime macros generate `Altrep`, `AltVec`, family-specific trait impls,
/// and `InferBase` (for class creation and method installation).
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

/// Generates the `impl RegisterAltrep for T` block for a single ALTREP type.
///
/// The implementation uses a `static OnceLock<R_altrep_class_t>` to lazily create and
/// cache the R ALTREP class handle. On first access it calls:
/// 1. `InferBase::make_class()` -- creates the `R_altrep_class_t` via the appropriate
///    `R_make_alt*_class` C function.
/// 2. `InferBase::install_methods()` -- registers all ALTREP method trampolines
///    (Length, Elt, Dataptr, etc.) on the class handle.
///
/// The class name is derived from the Rust type via [`AltrepTypeInfo::class_name`]
/// and stored as a null-terminated byte string constant.
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
                            CLASS_NAME.as_ptr().cast::<std::ffi::c_char>(),
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

/// Generates the `impl IntoR for T` block for a single ALTREP type.
///
/// The conversion wraps the Rust value in a `TypedExternal` external pointer (stored
/// in the ALTREP data1 slot) and creates an ALTREP SEXP via `R_new_altrep`.
///
/// Both checked (`into_sexp`) and unchecked (`into_sexp_unchecked`) variants are
/// generated. The checked variant uses thread-checked FFI calls; the unchecked variant
/// bypasses thread assertions for use in contexts known to be on the R main thread.
///
/// The `Error` associated type is `Infallible` since ALTREP creation cannot fail.
fn generate_into_r(info: &AltrepTypeInfo<'_>) -> TokenStream {
    let ty = info.impl_type;

    quote! {
        impl ::miniextendr_api::IntoR for #ty {
            type Error = std::convert::Infallible;

            fn try_into_sexp(self) -> Result<::miniextendr_api::ffi::SEXP, Self::Error> {
                Ok(self.into_sexp())
            }

            unsafe fn try_into_sexp_unchecked(self) -> Result<::miniextendr_api::ffi::SEXP, Self::Error> {
                Ok(unsafe { self.into_sexp_unchecked() })
            }

            fn into_sexp(self) -> ::miniextendr_api::ffi::SEXP {
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

            unsafe fn into_sexp_unchecked(self) -> ::miniextendr_api::ffi::SEXP {
                let cls = <#ty as ::miniextendr_api::altrep_registration::RegisterAltrep>::get_or_init_class();
                let data1 = <#ty as ::miniextendr_api::externalptr::TypedExternal>::wrap(self);
                unsafe {
                    ::miniextendr_api::ffi::altrep::R_new_altrep_unchecked(
                        cls,
                        data1,
                        ::miniextendr_api::ffi::SEXP::null(),
                    )
                }
            }
        }
    }
}

/// Partitions the module's `impl Trait for Type;` entries into ALTREP and non-ALTREP groups.
///
/// An entry is classified as ALTREP if its trait name matches one of the `Alt*Data` traits
/// (determined by [`MiniextendrModuleTraitImpl::altrep_base`]). All other trait impls
/// (e.g., cross-package trait ABI entries) are returned in the `other_impls` vector.
///
/// # Arguments
///
/// * `trait_impls` -- All `impl Trait for Type;` entries from the module.
///
/// # Returns
///
/// A tuple of:
/// - `Vec<AltrepTypeInfo>` -- ALTREP entries with their base type and cfg attributes.
/// - `Vec<&MiniextendrModuleTraitImpl>` -- Non-ALTREP trait impl entries, passed through
///   unchanged for further processing by the module expansion.
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
