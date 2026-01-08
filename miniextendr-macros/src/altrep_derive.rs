//! Derive macros for ALTREP data traits.
//!
//! These macros auto-implement `AltrepLen` and `Alt*Data` traits for simple field-based
//! ALTREP types, reducing boilerplate for users.

use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;

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
}

impl AltrepAttrs {
    /// Parse #[altrep(...)] attributes from a struct.
    fn parse(input: &syn::DeriveInput) -> syn::Result<Self> {
        let mut len_field = None;
        let mut elt_field = None;
        let mut generate_lowlevel = true; // Default: generate
        let mut lowlevel_options = Vec::new();

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
                }
                Ok(())
            })?;
        }

        Ok(Self {
            len_field,
            elt_field,
            generate_lowlevel,
            lowlevel_options,
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

/// Derive AltrepInteger - auto-implements AltrepLen and AltIntegerData.
pub fn derive_altrep_integer(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let generics = &input.generics;
    let attrs = AltrepAttrs::parse(&input)?;
    let len_field = attrs.get_len_field(&input)?;

    let altrep_len_impl = generate_altrep_len(name, generics, &len_field);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Generate elt() implementation if elt_field is specified
    let elt_impl = if let Some(ref elt_field) = attrs.elt_field {
        quote! {
            fn elt(&self, _i: usize) -> i32 {
                self.#elt_field
            }
        }
    } else {
        // No elt field - generate stub that returns NA
        // Users must override this
        quote! {
            fn elt(&self, _i: usize) -> i32 {
                ::miniextendr_api::altrep_traits::NA_INTEGER
            }
        }
    };

    let alt_integer_impl = quote! {
        impl #impl_generics ::miniextendr_api::altrep_data::AltIntegerData for #name #ty_generics #where_clause {
            #elt_impl
        }
    };

    // Generate impl_altinteger_from_data! call if requested
    let lowlevel_impl = if !attrs.generate_lowlevel {
        quote! {}
    } else if attrs.lowlevel_options.is_empty() {
        quote! {
            ::miniextendr_api::impl_altinteger_from_data!(#name);
        }
    } else {
        let options = &attrs.lowlevel_options;
        quote! {
            ::miniextendr_api::impl_altinteger_from_data!(#name, #(#options),*);
        }
    };

    Ok(quote! {
        #altrep_len_impl
        #alt_integer_impl
        #lowlevel_impl
    })
}

/// Derive AltrepReal - auto-implements AltrepLen and AltRealData.
pub fn derive_altrep_real(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let generics = &input.generics;
    let attrs = AltrepAttrs::parse(&input)?;
    let len_field = attrs.get_len_field(&input)?;

    let altrep_len_impl = generate_altrep_len(name, generics, &len_field);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let elt_impl = if let Some(ref elt_field) = attrs.elt_field {
        quote! {
            fn elt(&self, _i: usize) -> f64 {
                self.#elt_field
            }
        }
    } else {
        quote! {
            fn elt(&self, _i: usize) -> f64 {
                f64::NAN
            }
        }
    };

    let alt_real_impl = quote! {
        impl #impl_generics ::miniextendr_api::altrep_data::AltRealData for #name #ty_generics #where_clause {
            #elt_impl
        }
    };

    let lowlevel_impl = if !attrs.generate_lowlevel {
        quote! {}
    } else if attrs.lowlevel_options.is_empty() {
        quote! {
            ::miniextendr_api::impl_altreal_from_data!(#name);
        }
    } else {
        let options = &attrs.lowlevel_options;
        quote! {
            ::miniextendr_api::impl_altreal_from_data!(#name, #(#options),*);
        }
    };

    Ok(quote! {
        #altrep_len_impl
        #alt_real_impl
        #lowlevel_impl
    })
}

/// Derive AltrepLogical - auto-implements AltrepLen and AltLogicalData.
pub fn derive_altrep_logical(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let generics = &input.generics;
    let attrs = AltrepAttrs::parse(&input)?;
    let len_field = attrs.get_len_field(&input)?;

    let altrep_len_impl = generate_altrep_len(name, generics, &len_field);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let elt_impl = if let Some(ref elt_field) = attrs.elt_field {
        quote! {
            fn elt(&self, _i: usize) -> ::miniextendr_api::altrep_data::Logical {
                self.#elt_field.into()
            }
        }
    } else {
        quote! {
            fn elt(&self, _i: usize) -> ::miniextendr_api::altrep_data::Logical {
                ::miniextendr_api::altrep_data::Logical::Na
            }
        }
    };

    let alt_logical_impl = quote! {
        impl #impl_generics ::miniextendr_api::altrep_data::AltLogicalData for #name #ty_generics #where_clause {
            #elt_impl
        }
    };

    let lowlevel_impl = if !attrs.generate_lowlevel {
        quote! {}
    } else if attrs.lowlevel_options.is_empty() {
        quote! {
            ::miniextendr_api::impl_altlogical_from_data!(#name);
        }
    } else {
        let options = &attrs.lowlevel_options;
        quote! {
            ::miniextendr_api::impl_altlogical_from_data!(#name, #(#options),*);
        }
    };

    Ok(quote! {
        #altrep_len_impl
        #alt_logical_impl
        #lowlevel_impl
    })
}

/// Derive AltrepRaw - auto-implements AltrepLen and AltRawData.
pub fn derive_altrep_raw(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let generics = &input.generics;
    let attrs = AltrepAttrs::parse(&input)?;
    let len_field = attrs.get_len_field(&input)?;

    let altrep_len_impl = generate_altrep_len(name, generics, &len_field);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let elt_impl = if let Some(ref elt_field) = attrs.elt_field {
        quote! {
            fn elt(&self, _i: usize) -> u8 {
                self.#elt_field
            }
        }
    } else {
        quote! {
            fn elt(&self, _i: usize) -> u8 {
                0
            }
        }
    };

    let alt_raw_impl = quote! {
        impl #impl_generics ::miniextendr_api::altrep_data::AltRawData for #name #ty_generics #where_clause {
            #elt_impl
        }
    };

    let lowlevel_impl = if !attrs.generate_lowlevel {
        quote! {}
    } else if attrs.lowlevel_options.is_empty() {
        quote! {
            ::miniextendr_api::impl_altraw_from_data!(#name);
        }
    } else {
        let options = &attrs.lowlevel_options;
        quote! {
            ::miniextendr_api::impl_altraw_from_data!(#name, #(#options),*);
        }
    };

    Ok(quote! {
        #altrep_len_impl
        #alt_raw_impl
        #lowlevel_impl
    })
}

/// Derive AltrepString - auto-implements AltrepLen and AltStringData.
pub fn derive_altrep_string(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let generics = &input.generics;
    let attrs = AltrepAttrs::parse(&input)?;
    let len_field = attrs.get_len_field(&input)?;

    let altrep_len_impl = generate_altrep_len(name, generics, &len_field);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // String elt() returns Option<&str>, so if elt_field is specified,
    // we assume it's a String or &str field
    let elt_impl = if let Some(ref elt_field) = attrs.elt_field {
        quote! {
            fn elt(&self, _i: usize) -> Option<&str> {
                Some(self.#elt_field.as_ref())
            }
        }
    } else {
        quote! {
            fn elt(&self, _i: usize) -> Option<&str> {
                None
            }
        }
    };

    let alt_string_impl = quote! {
        impl #impl_generics ::miniextendr_api::altrep_data::AltStringData for #name #ty_generics #where_clause {
            #elt_impl
        }
    };

    let lowlevel_impl = if !attrs.generate_lowlevel {
        quote! {}
    } else if attrs.lowlevel_options.is_empty() {
        quote! {
            ::miniextendr_api::impl_altstring_from_data!(#name);
        }
    } else {
        let options = &attrs.lowlevel_options;
        quote! {
            ::miniextendr_api::impl_altstring_from_data!(#name, #(#options),*);
        }
    };

    Ok(quote! {
        #altrep_len_impl
        #alt_string_impl
        #lowlevel_impl
    })
}

/// Derive AltrepComplex - auto-implements AltrepLen and AltComplexData.
pub fn derive_altrep_complex(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let generics = &input.generics;
    let attrs = AltrepAttrs::parse(&input)?;
    let len_field = attrs.get_len_field(&input)?;

    let altrep_len_impl = generate_altrep_len(name, generics, &len_field);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let elt_impl = if let Some(ref elt_field) = attrs.elt_field {
        quote! {
            fn elt(&self, _i: usize) -> ::miniextendr_api::ffi::Rcomplex {
                self.#elt_field
            }
        }
    } else {
        quote! {
            fn elt(&self, _i: usize) -> ::miniextendr_api::ffi::Rcomplex {
                ::miniextendr_api::ffi::Rcomplex {
                    r: f64::NAN,
                    i: f64::NAN,
                }
            }
        }
    };

    let alt_complex_impl = quote! {
        impl #impl_generics ::miniextendr_api::altrep_data::AltComplexData for #name #ty_generics #where_clause {
            #elt_impl
        }
    };

    let lowlevel_impl = if !attrs.generate_lowlevel {
        quote! {}
    } else if attrs.lowlevel_options.is_empty() {
        quote! {
            ::miniextendr_api::impl_altcomplex_from_data!(#name);
        }
    } else {
        let options = &attrs.lowlevel_options;
        quote! {
            ::miniextendr_api::impl_altcomplex_from_data!(#name, #(#options),*);
        }
    };

    Ok(quote! {
        #altrep_len_impl
        #alt_complex_impl
        #lowlevel_impl
    })
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

    let lowlevel_impl = if attrs.generate_lowlevel {
        quote! {
            ::miniextendr_api::impl_altlist_from_data!(#name);
        }
    } else {
        quote! {}
    };

    Ok(quote! {
        #altrep_len_impl
        #alt_list_impl
        #lowlevel_impl
    })
}
