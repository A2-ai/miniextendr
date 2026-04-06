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

/// Valid ALTREP base type names corresponding to [`miniextendr_api::altrep::RBase`] variants.
///
/// These are the short names accepted by the `base = "..."` attribute on `#[miniextendr]`
/// ALTREP structs and `#[altrep_derive_opts(base = "...")]`:
/// - `"Int"` -- integer vector (`INTSXP`)
/// - `"Real"` -- double vector (`REALSXP`)
/// - `"Logical"` -- logical vector (`LGLSXP`)
/// - `"Raw"` -- raw/byte vector (`RAWSXP`)
/// - `"String"` -- character vector (`STRSXP`)
/// - `"List"` -- generic list (`VECSXP`)
/// - `"Complex"` -- complex vector (`CPLXSXP`)
const VALID_BASES: &[&str] = &["Int", "Real", "Logical", "Raw", "String", "List", "Complex"];

/// Generates family-specific ALTREP method setter code for an explicit base type.
///
/// Each ALTREP family (Int, Real, Logical, etc.) has a set of R C API setter functions
/// that register trampolines for optional and required callbacks. This function produces:
///
/// - `set_if!(...)` statements for **conditional** methods -- only registered if the
///   user's type sets the corresponding `HAS_*` associated constant to `true` on the
///   relevant trait (e.g., `AltInteger::HAS_ELT`).
/// - `unsafe { setter(cls, ...) }` statements for **always-required** methods --
///   currently `Elt` for String and List families, which have no `HAS_ELT` guard.
///
/// # Arguments
///
/// * `base_name` -- One of the [`VALID_BASES`] names (e.g., `"Int"`, `"String"`).
/// * `tramp_ty` -- The inner data type of the ALTREP wrapper struct, used as the type
///   parameter for the bridge trampoline functions (e.g., `Vec<i32>`).
///
/// # Returns
///
/// A [`TokenStream`](proc_macro2::TokenStream) containing the setter invocations,
/// or an empty stream if `base_name` is not recognized.
fn generate_explicit_setters(base_name: &str, tramp_ty: &syn::Type) -> proc_macro2::TokenStream {
    let span = proc_macro2::Span::call_site();

    // (trait_name, [(has_const, setter_fn, trampoline)], [(always_setter, always_trampoline)])
    type Cond = [(&'static str, &'static str, &'static str)];
    type Always = [(&'static str, &'static str)];
    let (trait_name, cond, always): (&str, &Cond, &Always) = match base_name {
        "Int" => (
            "AltInteger",
            &[
                ("HAS_ELT", "R_set_altinteger_Elt_method", "t_int_elt"),
                (
                    "HAS_GET_REGION",
                    "R_set_altinteger_Get_region_method",
                    "t_int_get_region",
                ),
                (
                    "HAS_IS_SORTED",
                    "R_set_altinteger_Is_sorted_method",
                    "t_int_is_sorted",
                ),
                ("HAS_NO_NA", "R_set_altinteger_No_NA_method", "t_int_no_na"),
                ("HAS_SUM", "R_set_altinteger_Sum_method", "t_int_sum"),
                ("HAS_MIN", "R_set_altinteger_Min_method", "t_int_min"),
                ("HAS_MAX", "R_set_altinteger_Max_method", "t_int_max"),
            ][..],
            &[][..],
        ),
        "Real" => (
            "AltReal",
            &[
                ("HAS_ELT", "R_set_altreal_Elt_method", "t_real_elt"),
                (
                    "HAS_GET_REGION",
                    "R_set_altreal_Get_region_method",
                    "t_real_get_region",
                ),
                (
                    "HAS_IS_SORTED",
                    "R_set_altreal_Is_sorted_method",
                    "t_real_is_sorted",
                ),
                ("HAS_NO_NA", "R_set_altreal_No_NA_method", "t_real_no_na"),
                ("HAS_SUM", "R_set_altreal_Sum_method", "t_real_sum"),
                ("HAS_MIN", "R_set_altreal_Min_method", "t_real_min"),
                ("HAS_MAX", "R_set_altreal_Max_method", "t_real_max"),
            ][..],
            &[][..],
        ),
        "Logical" => (
            "AltLogical",
            &[
                ("HAS_ELT", "R_set_altlogical_Elt_method", "t_lgl_elt"),
                (
                    "HAS_GET_REGION",
                    "R_set_altlogical_Get_region_method",
                    "t_lgl_get_region",
                ),
                (
                    "HAS_IS_SORTED",
                    "R_set_altlogical_Is_sorted_method",
                    "t_lgl_is_sorted",
                ),
                ("HAS_NO_NA", "R_set_altlogical_No_NA_method", "t_lgl_no_na"),
            ][..],
            &[][..],
        ),
        "Raw" => (
            "AltRaw",
            &[
                ("HAS_ELT", "R_set_altraw_Elt_method", "t_raw_elt"),
                (
                    "HAS_GET_REGION",
                    "R_set_altraw_Get_region_method",
                    "t_raw_get_region",
                ),
            ][..],
            &[][..],
        ),
        "String" => (
            "AltString",
            &[
                (
                    "HAS_IS_SORTED",
                    "R_set_altstring_Is_sorted_method",
                    "t_str_is_sorted",
                ),
                ("HAS_NO_NA", "R_set_altstring_No_NA_method", "t_str_no_na"),
                (
                    "HAS_SET_ELT",
                    "R_set_altstring_Set_elt_method",
                    "t_str_set_elt",
                ),
            ][..],
            &[("R_set_altstring_Elt_method", "t_str_elt")][..],
        ),
        "List" => (
            "AltList",
            &[(
                "HAS_SET_ELT",
                "R_set_altlist_Set_elt_method",
                "t_list_set_elt",
            )][..],
            &[("R_set_altlist_Elt_method", "t_list_elt")][..],
        ),
        "Complex" => (
            "AltComplex",
            &[
                ("HAS_ELT", "R_set_altcomplex_Elt_method", "t_cplx_elt"),
                (
                    "HAS_GET_REGION",
                    "R_set_altcomplex_Get_region_method",
                    "t_cplx_get_region",
                ),
            ][..],
            &[][..],
        ),
        _ => return quote::quote! {},
    };

    let trait_ident = syn::Ident::new(trait_name, span);

    let always_stmts = always.iter().map(|(setter, tramp)| {
        let s = syn::Ident::new(setter, span);
        let t = syn::Ident::new(tramp, span);
        quote::quote! { unsafe { #s(cls, Some(bridge::#t::<#tramp_ty>)); } }
    });

    let cond_stmts = cond.iter().map(|(has, setter, tramp)| {
        let h = syn::Ident::new(has, span);
        let s = syn::Ident::new(setter, span);
        let t = syn::Ident::new(tramp, span);
        quote::quote! {
            set_if!(<#tramp_ty as ::miniextendr_api::altrep_traits::#trait_ident>::#h, #s, bridge::#t::<#tramp_ty>);
        }
    });

    quote::quote! {
        #(#always_stmts)*
        #(#cond_stmts)*
    }
}

/// Generates the `R_make_alt*_class()` call and `validate_altrep_class()` assertion
/// for an explicit base type.
///
/// The returned expression creates the R ALTREP class handle via the appropriate
/// `R_make_alt*_class` C API function and immediately validates that the returned
/// handle is non-null (panics at registration time if R fails to create the class).
///
/// # Arguments
///
/// * `base_name` -- One of the [`VALID_BASES`] names (e.g., `"Real"`, `"List"`).
/// * `ident` -- The Rust struct identifier for the ALTREP wrapper type.
///
/// # Returns
///
/// A [`TokenStream`](proc_macro2::TokenStream) containing a block expression that
/// evaluates to an `R_altrep_class_t`.
fn generate_explicit_make_class(base_name: &str, ident: &syn::Ident) -> proc_macro2::TokenStream {
    let span = proc_macro2::Span::call_site();

    let make_fn_name = match base_name {
        "Int" => "R_make_altinteger_class",
        "Real" => "R_make_altreal_class",
        "Logical" => "R_make_altlogical_class",
        "Raw" => "R_make_altraw_class",
        "String" => "R_make_altstring_class",
        "List" => "R_make_altlist_class",
        "Complex" => "R_make_altcomplex_class",
        _ => unreachable!("validated by VALID_BASES check"),
    };

    let make_fn = syn::Ident::new(make_fn_name, span);
    let base_ident = syn::Ident::new(base_name, span);

    quote::quote! {{
        let __cls = ::miniextendr_api::ffi::altrep::#make_fn(
            <#ident as ::miniextendr_api::altrep::AltrepClass>::CLASS_NAME.as_ptr(),
            ::miniextendr_api::AltrepPkgName::as_ptr(),
            core::ptr::null_mut(),
        );
        ::miniextendr_api::altrep::validate_altrep_class(
            __cls,
            <#ident as ::miniextendr_api::altrep::AltrepClass>::CLASS_NAME,
            ::miniextendr_api::altrep::RBase::#base_ident,
        )
    }}
}

/// Reusable core for generating ALTREP trait implementations.
///
/// This is the shared code-generation backend called by both:
/// - [`expand_altrep_struct`] (the `#[miniextendr]` attribute macro path)
/// - [`derive_altrep`] (the `#[derive(AltrepClass)]` derive macro path)
///
/// It generates the following items for the wrapper struct:
/// - `impl AltrepClass` -- class name and base type constants
/// - `impl RegisterAltrep` -- lazy `OnceLock`-based class registration with method installation
/// - `impl IntoR` -- conversion from Rust to R ALTREP SEXP (both checked and unchecked)
/// - `impl From<DataTy>` -- construction from the inner data type
/// - `impl TryFromSexp` for `{Ident}Ref` -- immutable borrow wrapper with `Deref`
/// - `impl TryFromSexp` for `{Ident}Mut` -- mutable borrow wrapper with `Deref` + `DerefMut`
///
/// # Arguments
///
/// * `ident` -- The name of the ALTREP wrapper struct (e.g., `MyInts`).
/// * `generics` -- Any generic parameters on the struct.
/// * `data_ty` -- The inner data type (the single field's type, e.g., `Vec<i32>`).
/// * `class_name` -- The ALTREP class name string registered with R.
/// * `base_name` -- If `Some`, an explicit base type from [`VALID_BASES`]. If `None`,
///   the base is inferred at compile time via `InferBase`.
///
/// # Errors
///
/// Returns `Err` if `base_name` is `Some` but not one of the [`VALID_BASES`].
pub(crate) fn generate_altrep_impls(
    ident: &syn::Ident,
    generics: &syn::Generics,
    data_ty: &syn::Type,
    class_name: &str,
    base_name: Option<&str>,
) -> syn::Result<proc_macro2::TokenStream> {
    // Validate base if provided, otherwise use InferBase inference
    let base_variant: syn::Expr = if let Some(base_name) = base_name {
        if !VALID_BASES.contains(&base_name) {
            return Err(syn::Error::new(
                ident.span(),
                "base must be one of Int|Real|Logical|Raw|String|List|Complex",
            ));
        }
        let base_ident = syn::Ident::new(base_name, proc_macro2::Span::call_site());
        syn::parse_quote!(::miniextendr_api::altrep::RBase::#base_ident)
    } else {
        syn::parse_quote!(<#data_ty as ::miniextendr_api::altrep_data::InferBase>::BASE)
    };

    let tramp_ty = data_ty.clone();

    // Generate family setters and make_class based on the base type.
    let (family_setters, make_class): (proc_macro2::TokenStream, proc_macro2::TokenStream) =
        if let Some(base_name) = base_name {
            let setters = generate_explicit_setters(base_name, &tramp_ty);
            let make = generate_explicit_make_class(base_name, ident);
            (setters, make)
        } else {
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

    let (_impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let class_cstr = syn::LitCStr::new(&std::ffi::CString::new(class_name).unwrap(), ident.span());

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
                type Error = std::convert::Infallible;

                fn try_into_sexp(self) -> Result<::miniextendr_api::ffi::SEXP, Self::Error> {
                    Ok(self.into_sexp())
                }

                unsafe fn try_into_sexp_unchecked(self) -> Result<::miniextendr_api::ffi::SEXP, Self::Error> {
                    Ok(unsafe { self.into_sexp_unchecked() })
                }

                fn into_sexp(self) -> ::miniextendr_api::ffi::SEXP {
                    use ::miniextendr_api::altrep_registration::RegisterAltrep;
                    use ::miniextendr_api::externalptr::ExternalPtr;
                    use ::miniextendr_api::ffi::altrep::R_new_altrep;
                    use ::miniextendr_api::ffi::{SEXP, Rf_protect, Rf_unprotect};

                    let ext_ptr = ExternalPtr::new(self.0);
                    let cls = Self::get_or_init_class();
                    let data1 = ext_ptr.as_sexp();
                    unsafe {
                        Rf_protect(data1);
                        let altrep = R_new_altrep(cls, data1, SEXP::nil());
                        Rf_unprotect(1);
                        altrep
                    }
                }

                unsafe fn into_sexp_unchecked(self) -> ::miniextendr_api::ffi::SEXP {
                    use ::miniextendr_api::altrep_registration::RegisterAltrep;
                    use ::miniextendr_api::externalptr::ExternalPtr;
                    use ::miniextendr_api::ffi::altrep::R_new_altrep_unchecked;
                    use ::miniextendr_api::ffi::{Rf_protect_unchecked, Rf_unprotect_unchecked};

                    let ext_ptr = ExternalPtr::new(self.0);
                    let cls = Self::get_or_init_class();
                    let data1 = ext_ptr.as_sexp();
                    unsafe {
                        Rf_protect_unchecked(data1);
                        let altrep = R_new_altrep_unchecked(
                            cls,
                            data1,
                            ::miniextendr_api::ffi::SEXP::nil(),
                        );
                        Rf_unprotect_unchecked(1);
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

                    if unsafe { ::miniextendr_api::ffi::ALTREP(sexp) } == 0 {
                        return Err(::miniextendr_api::SexpTypeError {
                            expected: SEXPTYPE::INTSXP,
                            actual: ::miniextendr_api::ffi::SexpExt::type_of(&sexp),
                        });
                    }

                    match unsafe { ::miniextendr_api::altrep_data1_as::<#data_ty>(sexp) } {
                        Some(ptr) => Ok(#ref_ident(ptr)),
                        None => Err(::miniextendr_api::SexpTypeError {
                            expected: SEXPTYPE::EXTPTRSXP,
                            actual: ::miniextendr_api::ffi::SexpExt::type_of(&sexp),
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

                    if unsafe { ::miniextendr_api::ffi::ALTREP(sexp) } == 0 {
                        return Err(::miniextendr_api::SexpTypeError {
                            expected: SEXPTYPE::INTSXP,
                            actual: ::miniextendr_api::ffi::SexpExt::type_of(&sexp),
                        });
                    }

                    match unsafe { ::miniextendr_api::altrep_data1_as::<#data_ty>(sexp) } {
                        Some(ptr) => Ok(#mut_ident(ptr)),
                        None => Err(::miniextendr_api::SexpTypeError {
                            expected: SEXPTYPE::EXTPTRSXP,
                            actual: ::miniextendr_api::ffi::SexpExt::type_of(&sexp),
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

    let base_doc = base_name
        .map(|b| b.to_string())
        .unwrap_or_else(|| "inferred".to_string());
    let altrep_class_doc = format!(
        "ALTREP class descriptor for [`{}`] (class: `{}`, base: `{}`).",
        ident, class_name, base_doc
    );
    let register_altrep_doc = format!("Registration entry point for [`{}`] ALTREP class.", ident);
    let source_loc_doc = crate::source_location_doc(ident.span());

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

    Ok(quote::quote! {
        // Helper methods for creating ALTREP instances
        #data_helper_impl

        #[doc = #altrep_class_doc]
        #[doc = #source_loc_doc]
        #[doc = concat!("Generated from source file `", file!(), "`.")]
        #[allow(clippy::not_unsafe_ptr_arg_deref)]
        impl ::miniextendr_api::altrep::AltrepClass for #ident #ty_generics #where_clause {
            const CLASS_NAME: &'static std::ffi::CStr = #class_cstr;
            const BASE: ::miniextendr_api::altrep::RBase = #base_variant;
        }

        #[doc = #register_altrep_doc]
        #[doc = #source_loc_doc]
        #[doc = concat!("Generated from source file `", file!(), "`.")]
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
                        #[allow(unused_macros)]
                        macro_rules! set_if { ($cond:expr, $setter:path, $tramp:expr) => { if $cond { unsafe { $setter(cls, Some($tramp)) } } } }
                        #method_registrar_install_body
                    }
                    cls
                })
            }
        }

        #altrep_reg_entry
    })
}

/// Expands `#[miniextendr]` on a one-field wrapper struct into ALTREP plumbing.
///
/// This is the attribute macro entry point for ALTREP structs. The struct must have
/// exactly one field whose type is the ALTREP data container (e.g., `Vec<i32>`).
///
/// # Supported attributes
///
/// ```ignore
/// #[miniextendr]                              // class name = struct name, base inferred
/// #[miniextendr(class = "CustomName")]        // explicit ALTREP class name
/// #[miniextendr(base = "Int")]                // explicit base type (Int|Real|Logical|Raw|String|List|Complex)
/// #[miniextendr(class = "Name", base = "Real")] // both
/// ```
///
/// The `pkg` attribute is silently ignored for backwards compatibility.
///
/// # Arguments
///
/// * `attr` -- The attribute arguments (e.g., `class = "...", base = "..."`).
/// * `item` -- The struct definition token stream.
///
/// # Returns
///
/// The original struct definition followed by all generated ALTREP trait implementations.
/// On error, returns a compile error token stream.
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
                // Silently ignore "pkg" for backwards compatibility
                "pkg" => {}
                _ => {}
            }
        }
    }

    let class_name = class_name.unwrap_or_else(|| ident.to_string());

    // Validate base early with span if we have the literal
    if let Some(ref base_name) = base_name
        && !VALID_BASES.contains(&base_name.as_str())
    {
        return syn::Error::new_spanned(
            base_lit.expect("base_lit set when base_name is Some"),
            "base must be one of Int|Real|Logical|Raw|String|List|Complex",
        )
        .into_compile_error()
        .into();
    }

    match generate_altrep_impls(
        &ident,
        &generics,
        &data_ty,
        &class_name,
        base_name.as_deref(),
    ) {
        Ok(impls) => {
            let expanded = quote::quote! {
                #input
                #impls
            };
            expanded.into()
        }
        Err(e) => e.into_compile_error().into(),
    }
}

/// Entry point for `#[derive(AltrepClass)]`.
///
/// Parses optional `#[altrep_derive_opts(...)]` helper attributes and generates
/// ALTREP registration, `IntoR`, `TryFromSexp`, and Ref/Mut wrappers using the
/// shared [`generate_altrep_impls`] core.
///
/// # Helper attributes
///
/// ```ignore
/// #[altrep_derive_opts(class = "CustomName")]  // override the ALTREP class name (default: struct name)
/// #[altrep_derive_opts(base = "Real")]         // explicit base type (default: inferred via InferBase)
/// ```
///
/// The struct must have exactly one field. Both tuple structs (`struct X(T)`) and
/// named-field structs (`struct X { data: T }`) are accepted.
///
/// # Errors
///
/// Returns `Err` if the input is not a single-field struct or if an unknown
/// `altrep_derive_opts` key is provided.
pub fn derive_altrep(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    use syn::spanned::Spanned;

    let ident = &input.ident;

    // Extract the inner type - must be a 1-field struct
    let data_ty: syn::Type = match &input.data {
        syn::Data::Struct(data) => match &data.fields {
            syn::Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                fields.unnamed.first().unwrap().ty.clone()
            }
            syn::Fields::Named(fields) if fields.named.len() == 1 => {
                fields.named.first().unwrap().ty.clone()
            }
            _ => {
                return Err(syn::Error::new(
                    input.span(),
                    "#[derive(AltrepClass)] requires a 1-field wrapper struct, e.g., `struct MyInts(Vec<i32>);`",
                ));
            }
        },
        _ => {
            return Err(syn::Error::new(
                input.span(),
                "#[derive(AltrepClass)] can only be applied to structs",
            ));
        }
    };

    // Parse #[altrep_derive_opts(class = "...", base = "...")] helper attributes
    let mut class_name = None::<String>;
    let mut base_name = None::<String>;

    for attr in &input.attrs {
        if attr.path().is_ident("altrep_derive_opts") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("class") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    class_name = Some(value.value());
                } else if meta.path.is_ident("base") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    base_name = Some(value.value());
                } else {
                    return Err(meta.error(
                        "unknown altrep_derive_opts attribute; expected `class` or `base`",
                    ));
                }
                Ok(())
            })?;
        }
    }

    let class_name = class_name.unwrap_or_else(|| ident.to_string());

    generate_altrep_impls(
        ident,
        &input.generics,
        &data_ty,
        &class_name,
        base_name.as_deref(),
    )
}
