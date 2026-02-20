//! Derive macros for ALTREP data traits.
//!
//! These macros auto-implement `AltrepLen` and `Alt*Data` traits for simple field-based
//! ALTREP types, reducing boilerplate for users.

use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;

/// Per-family configuration for ALTREP lowlevel codegen.
struct AltrepFamilyConfig<'a> {
    macro_base: &'a str,
    dataptr_macro: Option<(&'a str, Option<TokenStream>)>,
    string_dataptr: bool,
    subset: bool,
    methods_macro: &'a str,
    inferbase_macro: &'a str,
}

/// Common attributes for ALTREP derives.
struct AltrepAttrs {
    /// Field name containing the length
    len_field: Option<syn::Ident>,
    /// Field name containing the element value (for constant vectors)
    elt_field: Option<syn::Ident>,
    /// Whether to generate impl_alt*_from_data! call
    generate_lowlevel: bool,
    /// Options for impl_alt*_from_data! (dataptr, serialize, subset)
    lowlevel_options: Vec<syn::Ident>,
    /// Guard mode override: "unsafe" | "rust_unwind" | "r_unwind"
    guard: Option<syn::Ident>,
}

impl AltrepAttrs {
    /// Parse #[altrep(...)] attributes from a struct.
    fn parse(input: &syn::DeriveInput) -> syn::Result<Self> {
        let mut len_field = None;
        let mut elt_field = None;
        let mut generate_lowlevel = true; // Default: generate
        let mut lowlevel_options = Vec::new();
        let mut guard = None;

        for attr in &input.attrs {
            if !attr.path().is_ident("altrep") {
                continue;
            }

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("len") {
                    let _: syn::Token![=] = meta.input.parse()?;
                    let field: syn::LitStr = meta.input.parse()?;
                    len_field = Some(syn::Ident::new(&field.value(), field.span()));
                } else if meta.path.is_ident("elt") {
                    let _: syn::Token![=] = meta.input.parse()?;
                    let field: syn::LitStr = meta.input.parse()?;
                    elt_field = Some(syn::Ident::new(&field.value(), field.span()));
                } else if meta.path.is_ident("no_lowlevel") {
                    generate_lowlevel = false;
                } else if meta.path.is_ident("dataptr") {
                    lowlevel_options.push(syn::Ident::new("dataptr", meta.path.span()));
                } else if meta.path.is_ident("serialize") {
                    lowlevel_options.push(syn::Ident::new("serialize", meta.path.span()));
                } else if meta.path.is_ident("subset") {
                    lowlevel_options.push(syn::Ident::new("subset", meta.path.span()));
                } else if meta.path.is_ident("r#unsafe") || meta.path.is_ident("unsafe") {
                    guard = Some(syn::Ident::new("Unsafe", meta.path.span()));
                } else if meta.path.is_ident("rust_unwind") {
                    guard = Some(syn::Ident::new("RustUnwind", meta.path.span()));
                } else if meta.path.is_ident("r_unwind") {
                    guard = Some(syn::Ident::new("RUnwind", meta.path.span()));
                }
                Ok(())
            })?;
        }

        Ok(Self {
            len_field,
            elt_field,
            generate_lowlevel,
            lowlevel_options,
            guard,
        })
    }

    /// Get the length field or try to auto-detect it.
    fn get_len_field(&self, input: &syn::DeriveInput) -> syn::Result<syn::Ident> {
        if let Some(ref field) = self.len_field {
            return Ok(field.clone());
        }

        // Try to auto-detect: look for field named "len" or "length"
        let fields = match &input.data {
            syn::Data::Struct(data_struct) => &data_struct.fields,
            _ => {
                return Err(syn::Error::new(
                    input.span(),
                    "Altrep derive only supports structs",
                ));
            }
        };

        for field in fields {
            if let Some(ident) = &field.ident
                && (ident == "len" || ident == "length")
            {
                return Ok(ident.clone());
            }
        }

        Err(syn::Error::new(
            input.span(),
            "no length field found; specify with #[altrep(len = \"field_name\")]",
        ))
    }

    /// Returns true if a non-default guard mode is set (i.e. not RustUnwind).
    fn has_non_default_guard(&self) -> bool {
        match &self.guard {
            Some(g) => g != "RustUnwind",
            None => false,
        }
    }

    /// Validate option combinations for a given ALTREP type family.
    fn validate_options(&self, family: &str, supports_subset: bool) -> syn::Result<()> {
        let has_dataptr = self.lowlevel_options.iter().any(|o| o == "dataptr");
        let has_subset = self.lowlevel_options.iter().any(|o| o == "subset");

        if has_subset && !supports_subset {
            return Err(syn::Error::new(
                self.lowlevel_options
                    .iter()
                    .find(|o| *o == "subset")
                    .unwrap()
                    .span(),
                format!("`subset` is not supported for {family}"),
            ));
        }

        if has_dataptr && has_subset {
            return Err(syn::Error::new(
                self.lowlevel_options
                    .iter()
                    .find(|o| *o == "subset")
                    .unwrap()
                    .span(),
                "`dataptr` and `subset` are mutually exclusive",
            ));
        }

        Ok(())
    }

    /// Generate lowlevel impl code for a given ALTREP type family.
    fn generate_lowlevel(
        &self,
        name: &syn::Ident,
        family: &AltrepFamilyConfig,
    ) -> syn::Result<TokenStream> {
        let AltrepFamilyConfig {
            macro_base,
            ref dataptr_macro,
            string_dataptr,
            subset,
            methods_macro,
            inferbase_macro,
        } = *family;
        let altvec_dataptr_macro = dataptr_macro;
        let altvec_string_dataptr = string_dataptr;
        let altvec_subset = subset;
        if !self.generate_lowlevel {
            return Ok(quote! {});
        }

        self.validate_options(macro_base, altvec_subset)?;

        let has_serialize = self.lowlevel_options.iter().any(|o| o == "serialize");
        let has_dataptr = self.lowlevel_options.iter().any(|o| o == "dataptr");
        let has_subset = self.lowlevel_options.iter().any(|o| o == "subset");

        // Use the expanded path (individual internal macros) when:
        // - Non-default guard mode is set, OR
        // - `subset` is requested (the runtime from_data macros only have subset
        //   variants for integer and complex; other families expand manually)
        let needs_expanded_path = self.has_non_default_guard() || has_subset;

        if !needs_expanded_path {
            // Simple path: delegate to the impl_alt*_from_data! runtime macro
            let macro_ident = syn::Ident::new(macro_base, proc_macro2::Span::call_site());
            if self.lowlevel_options.is_empty() {
                return Ok(quote! {
                    ::miniextendr_api::#macro_ident!(#name);
                });
            } else {
                let options = &self.lowlevel_options;
                return Ok(quote! {
                    ::miniextendr_api::#macro_ident!(#name, #(#options),*);
                });
            }
        }

        // Expanded path: emit individual internal macros
        let guard = self
            .guard
            .as_ref()
            .cloned()
            .unwrap_or_else(|| syn::Ident::new("RustUnwind", proc_macro2::Span::call_site()));

        // 1. Altrep base (with or without serialize)
        let base_impl = if has_serialize {
            quote! { ::miniextendr_api::__impl_altrep_base_with_serialize!(#name, #guard); }
        } else {
            quote! { ::miniextendr_api::__impl_altrep_base!(#name, #guard); }
        };

        // 2. AltVec impl
        let vec_impl = if has_dataptr {
            if let Some((macro_name, elem_ty)) = altvec_dataptr_macro {
                let dp_macro = syn::Ident::new(macro_name, proc_macro2::Span::call_site());
                if let Some(elem) = elem_ty {
                    quote! { ::miniextendr_api::#dp_macro!(#name, #elem); }
                } else {
                    quote! { ::miniextendr_api::#dp_macro!(#name); }
                }
            } else if altvec_string_dataptr {
                quote! { ::miniextendr_api::__impl_altvec_string_dataptr!(#name); }
            } else {
                quote! { impl ::miniextendr_api::altrep_traits::AltVec for #name {} }
            }
        } else if has_subset && altvec_subset {
            quote! { ::miniextendr_api::__impl_altvec_extract_subset!(#name); }
        } else {
            quote! { impl ::miniextendr_api::altrep_traits::AltVec for #name {} }
        };

        // 3. Type-specific methods
        let methods_ident = syn::Ident::new(methods_macro, proc_macro2::Span::call_site());
        let methods_impl = quote! { ::miniextendr_api::#methods_ident!(#name); };

        // 4. InferBase
        let inferbase_ident = syn::Ident::new(inferbase_macro, proc_macro2::Span::call_site());
        let inferbase_impl = quote! { ::miniextendr_api::#inferbase_ident!(#name); };

        Ok(quote! {
            #base_impl
            #vec_impl
            #methods_impl
            #inferbase_impl
        })
    }
}

/// Generate impl AltrepLen for a struct.
fn generate_altrep_len(
    name: &syn::Ident,
    generics: &syn::Generics,
    len_field: &syn::Ident,
) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics ::miniextendr_api::altrep_data::AltrepLen for #name #ty_generics #where_clause {
            fn len(&self) -> usize {
                self.#len_field
            }
        }
    }
}

/// Shared implementation for all non-list ALTREP derive macros.
///
/// The `gen_elt_impl` closure receives `elt_field` (if specified via `#[altrep(elt = "...")]`)
/// and returns the `fn elt(...)` method body for the data trait implementation.
fn derive_altrep_generic(
    input: syn::DeriveInput,
    data_trait_path: TokenStream,
    gen_elt_impl: impl FnOnce(Option<&syn::Ident>) -> TokenStream,
    family: &AltrepFamilyConfig,
) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let generics = &input.generics;
    let attrs = AltrepAttrs::parse(&input)?;
    let len_field = attrs.get_len_field(&input)?;

    let altrep_len_impl = generate_altrep_len(name, generics, &len_field);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let elt_impl = gen_elt_impl(attrs.elt_field.as_ref());

    let data_trait_impl = quote! {
        impl #impl_generics #data_trait_path for #name #ty_generics #where_clause {
            #elt_impl
        }
    };

    let lowlevel_impl = attrs.generate_lowlevel(name, family)?;

    Ok(quote! {
        #altrep_len_impl
        #data_trait_impl
        #lowlevel_impl
    })
}

/// Derive AltrepInteger - auto-implements AltrepLen and AltIntegerData.
pub fn derive_altrep_integer(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    derive_altrep_generic(
        input,
        quote! { ::miniextendr_api::altrep_data::AltIntegerData },
        |elt_field| {
            if let Some(f) = elt_field {
                quote! { fn elt(&self, _i: usize) -> i32 { self.#f } }
            } else {
                quote! { fn elt(&self, _i: usize) -> i32 { ::miniextendr_api::altrep_traits::NA_INTEGER } }
            }
        },
        &AltrepFamilyConfig {
            macro_base: "impl_altinteger_from_data",
            dataptr_macro: Some(("__impl_altvec_dataptr", Some(quote! { i32 }))),
            string_dataptr: false,
            subset: true,
            methods_macro: "__impl_altinteger_methods",
            inferbase_macro: "impl_inferbase_integer",
        },
    )
}

/// Derive AltrepReal - auto-implements AltrepLen and AltRealData.
pub fn derive_altrep_real(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    derive_altrep_generic(
        input,
        quote! { ::miniextendr_api::altrep_data::AltRealData },
        |elt_field| {
            if let Some(f) = elt_field {
                quote! { fn elt(&self, _i: usize) -> f64 { self.#f } }
            } else {
                quote! { fn elt(&self, _i: usize) -> f64 { f64::NAN } }
            }
        },
        &AltrepFamilyConfig {
            macro_base: "impl_altreal_from_data",
            dataptr_macro: Some(("__impl_altvec_dataptr", Some(quote! { f64 }))),
            string_dataptr: false,
            subset: true,
            methods_macro: "__impl_altreal_methods",
            inferbase_macro: "impl_inferbase_real",
        },
    )
}

/// Derive AltrepLogical - auto-implements AltrepLen and AltLogicalData.
pub fn derive_altrep_logical(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    derive_altrep_generic(
        input,
        quote! { ::miniextendr_api::altrep_data::AltLogicalData },
        |elt_field| {
            if let Some(f) = elt_field {
                quote! { fn elt(&self, _i: usize) -> ::miniextendr_api::altrep_data::Logical { self.#f.into() } }
            } else {
                quote! { fn elt(&self, _i: usize) -> ::miniextendr_api::altrep_data::Logical { ::miniextendr_api::altrep_data::Logical::Na } }
            }
        },
        &AltrepFamilyConfig {
            macro_base: "impl_altlogical_from_data",
            dataptr_macro: Some(("__impl_altvec_dataptr", Some(quote! { i32 }))),
            string_dataptr: false,
            subset: true,
            methods_macro: "__impl_altlogical_methods",
            inferbase_macro: "impl_inferbase_logical",
        },
    )
}

/// Derive AltrepRaw - auto-implements AltrepLen and AltRawData.
pub fn derive_altrep_raw(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    derive_altrep_generic(
        input,
        quote! { ::miniextendr_api::altrep_data::AltRawData },
        |elt_field| {
            if let Some(f) = elt_field {
                quote! { fn elt(&self, _i: usize) -> u8 { self.#f } }
            } else {
                quote! { fn elt(&self, _i: usize) -> u8 { 0 } }
            }
        },
        &AltrepFamilyConfig {
            macro_base: "impl_altraw_from_data",
            dataptr_macro: Some(("__impl_altvec_dataptr", Some(quote! { u8 }))),
            string_dataptr: false,
            subset: true,
            methods_macro: "__impl_altraw_methods",
            inferbase_macro: "impl_inferbase_raw",
        },
    )
}

/// Derive AltrepString - auto-implements AltrepLen and AltStringData.
pub fn derive_altrep_string(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    derive_altrep_generic(
        input,
        quote! { ::miniextendr_api::altrep_data::AltStringData },
        |elt_field| {
            if let Some(f) = elt_field {
                quote! { fn elt(&self, _i: usize) -> Option<&str> { Some(self.#f.as_ref()) } }
            } else {
                quote! { fn elt(&self, _i: usize) -> Option<&str> { None } }
            }
        },
        &AltrepFamilyConfig {
            macro_base: "impl_altstring_from_data",
            dataptr_macro: None,
            string_dataptr: true,
            subset: true,
            methods_macro: "__impl_altstring_methods",
            inferbase_macro: "impl_inferbase_string",
        },
    )
}

/// Derive AltrepComplex - auto-implements AltrepLen and AltComplexData.
pub fn derive_altrep_complex(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    derive_altrep_generic(
        input,
        quote! { ::miniextendr_api::altrep_data::AltComplexData },
        |elt_field| {
            if let Some(f) = elt_field {
                quote! { fn elt(&self, _i: usize) -> ::miniextendr_api::ffi::Rcomplex { self.#f } }
            } else {
                quote! {
                    fn elt(&self, _i: usize) -> ::miniextendr_api::ffi::Rcomplex {
                        ::miniextendr_api::ffi::Rcomplex { r: f64::NAN, i: f64::NAN }
                    }
                }
            }
        },
        &AltrepFamilyConfig {
            macro_base: "impl_altcomplex_from_data",
            dataptr_macro: Some((
                "__impl_altvec_dataptr",
                Some(quote! { ::miniextendr_api::ffi::Rcomplex }),
            )),
            string_dataptr: false,
            subset: true,
            methods_macro: "__impl_altcomplex_methods",
            inferbase_macro: "impl_inferbase_complex",
        },
    )
}

/// Derive AltrepList - auto-implements AltrepLen and AltListData.
pub fn derive_altrep_list(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let generics = &input.generics;
    let attrs = AltrepAttrs::parse(&input)?;
    let len_field = attrs.get_len_field(&input)?;

    let altrep_len_impl = generate_altrep_len(name, generics, &len_field);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // List elt() returns SEXP - if elt_field specified, assume it's a Vec<SEXP> or similar
    let elt_impl = if let Some(ref elt_field) = attrs.elt_field {
        quote! {
            fn elt(&self, i: usize) -> ::miniextendr_api::ffi::SEXP {
                self.#elt_field[i]
            }
        }
    } else {
        quote! {
            fn elt(&self, _i: usize) -> ::miniextendr_api::ffi::SEXP {
                unsafe { ::miniextendr_api::ffi::R_NilValue }
            }
        }
    };

    let alt_list_impl = quote! {
        impl #impl_generics ::miniextendr_api::altrep_data::AltListData for #name #ty_generics #where_clause {
            #elt_impl
        }
    };

    // List does not support dataptr or subset
    for opt in &attrs.lowlevel_options {
        if opt == "dataptr" || opt == "subset" {
            return Err(syn::Error::new(
                opt.span(),
                format!("`{opt}` is not supported for AltrepList"),
            ));
        }
    }

    let has_serialize = attrs.lowlevel_options.iter().any(|o| o == "serialize");

    let lowlevel_impl = if !attrs.generate_lowlevel {
        quote! {}
    } else if has_serialize {
        // Serialize requires expanding individual macros since impl_altlist_from_data!
        // does not have a serialize variant. Use __impl_altrep_base_with_serialize!
        // for the Altrep trait, then manually emit AltVec + AltList + InferBase.
        if attrs.has_non_default_guard() {
            let guard = attrs.guard.as_ref().unwrap();
            quote! {
                ::miniextendr_api::__impl_altrep_base_with_serialize!(#name, #guard);
                impl ::miniextendr_api::altrep_traits::AltVec for #name {}
                #[allow(clippy::not_unsafe_ptr_arg_deref)]
                impl ::miniextendr_api::altrep_traits::AltList for #name {
                    fn elt(x: ::miniextendr_api::ffi::SEXP, i: ::miniextendr_api::ffi::R_xlen_t) -> ::miniextendr_api::ffi::SEXP {
                        unsafe { ::miniextendr_api::altrep_data1_as::<#name>(x) }
                            .map(|d| <#name as ::miniextendr_api::altrep_data::AltListData>::elt(&*d, i.max(0) as usize))
                            .unwrap_or(unsafe { ::miniextendr_api::ffi::R_NilValue })
                    }
                }
                ::miniextendr_api::impl_inferbase_list!(#name);
            }
        } else {
            quote! {
                ::miniextendr_api::__impl_altrep_base_with_serialize!(#name);
                impl ::miniextendr_api::altrep_traits::AltVec for #name {}
                #[allow(clippy::not_unsafe_ptr_arg_deref)]
                impl ::miniextendr_api::altrep_traits::AltList for #name {
                    fn elt(x: ::miniextendr_api::ffi::SEXP, i: ::miniextendr_api::ffi::R_xlen_t) -> ::miniextendr_api::ffi::SEXP {
                        unsafe { ::miniextendr_api::altrep_data1_as::<#name>(x) }
                            .map(|d| <#name as ::miniextendr_api::altrep_data::AltListData>::elt(&*d, i.max(0) as usize))
                            .unwrap_or(unsafe { ::miniextendr_api::ffi::R_NilValue })
                    }
                }
                ::miniextendr_api::impl_inferbase_list!(#name);
            }
        }
    } else if attrs.has_non_default_guard() {
        let guard = attrs.guard.as_ref().unwrap();
        quote! {
            ::miniextendr_api::impl_altlist_from_data!(#name, #guard);
        }
    } else {
        quote! {
            ::miniextendr_api::impl_altlist_from_data!(#name);
        }
    };

    Ok(quote! {
        #altrep_len_impl
        #alt_list_impl
        #lowlevel_impl
    })
}
