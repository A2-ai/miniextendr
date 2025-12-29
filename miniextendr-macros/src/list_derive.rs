use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Fields, parse_quote, spanned::Spanned};

/// Derive `IntoList` for structs (Rust → R).
///
/// - Named structs (`struct Foo { x: i32 }`) → named R list: `list(x = 1L)`
/// - Tuple structs (`struct Foo(i32, i32)`) → unnamed R list: `list(1L, 2L)`
/// - Unit structs (`struct Foo`) → empty R list: `list()`
pub fn derive_into_list(input: DeriveInput) -> syn::Result<TokenStream> {
    let struct_data = match input.data {
        syn::Data::Struct(data) => data,
        _ => {
            return Err(syn::Error::new(
                input.ident.span(),
                "IntoList can only be derived for structs",
            ));
        }
    };

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut bounds: Vec<syn::WherePredicate> = Vec::new();

    let (destructure_pat, list_construction) = match &struct_data.fields {
        // Named struct: create named R list
        Fields::Named(fields) => {
            let mut names: Vec<String> = Vec::new();
            let mut idents: Vec<syn::Ident> = Vec::new();

            for f in fields.named.iter() {
                let ident = f.ident.as_ref().unwrap().clone();
                let ty = &f.ty;
                bounds.push(parse_quote!(#ty: ::miniextendr_api::into_r::IntoR));
                names.push(ident.to_string());
                idents.push(ident);
            }

            let pat = quote! { { #(#idents),* } };
            let construction = quote! {
                ::miniextendr_api::list::List::from_pairs(vec![ #( (#names, #idents) ),* ])
            };
            (pat, construction)
        }

        // Tuple struct: create unnamed R list (positional access)
        Fields::Unnamed(fields) => {
            let mut idents: Vec<syn::Ident> = Vec::new();

            for (idx, f) in fields.unnamed.iter().enumerate() {
                let ident = syn::Ident::new(&format!("_field{idx}"), f.span());
                let ty = &f.ty;
                bounds.push(parse_quote!(#ty: ::miniextendr_api::into_r::IntoR));
                idents.push(ident);
            }

            let pat = quote! { ( #(#idents),* ) };
            let construction = quote! {
                ::miniextendr_api::list::List::from_raw_values(vec![ #( #idents.into_sexp() ),* ])
            };
            (pat, construction)
        }

        // Unit struct: empty list
        Fields::Unit => {
            let pat = quote! {};
            let construction = quote! {
                ::miniextendr_api::list::List::from_raw_values(vec![])
            };
            (pat, construction)
        }
    };

    // Extend where-clause with bounds
    let mut where_clause = where_clause.cloned().unwrap_or_else(|| syn::WhereClause {
        where_token: <syn::Token![where]>::default(),
        predicates: syn::punctuated::Punctuated::new(),
    });
    for b in bounds {
        where_clause.predicates.push(b);
    }

    let expand = quote! {
        impl #impl_generics ::miniextendr_api::list::IntoList for #name #ty_generics #where_clause {
            fn into_list(self) -> ::miniextendr_api::list::List {
                use ::miniextendr_api::into_r::IntoR;
                let Self #destructure_pat = self;
                #list_construction
            }
        }
    };

    Ok(expand)
}

/// Derive `TryFromList` for structs (R → Rust).
///
/// - Named structs: extract by field name from named R list
/// - Tuple structs: extract by position (index 0, 1, 2, ...)
/// - Unit structs: accept any list (no extraction needed)
pub fn derive_try_from_list(input: DeriveInput) -> syn::Result<TokenStream> {
    let struct_data = match input.data {
        syn::Data::Struct(data) => data,
        _ => {
            return Err(syn::Error::new(
                input.ident.span(),
                "TryFromList can only be derived for structs",
            ));
        }
    };

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut bounds: Vec<syn::WherePredicate> = Vec::new();

    let from_list_body = match &struct_data.fields {
        // Named struct: extract by field name
        Fields::Named(fields) => {
            let mut idents: Vec<syn::Ident> = Vec::new();
            let mut field_extractions: Vec<proc_macro2::TokenStream> = Vec::new();

            for f in fields.named.iter() {
                let ident = f.ident.as_ref().unwrap().clone();
                let ty = &f.ty;
                bounds.push(parse_quote!(#ty: ::miniextendr_api::from_r::TryFromSexp<Error = ::miniextendr_api::from_r::SexpError>));

                let name_str = ident.to_string();
                idents.push(ident.clone());

                field_extractions.push(quote! {
                    let #ident: #ty = list.get_named(#name_str)
                        .ok_or_else(|| ::miniextendr_api::from_r::SexpError::MissingField(#name_str.into()))?;
                });
            }

            quote! {
                #(#field_extractions)*
                Ok(Self { #(#idents),* })
            }
        }

        // Tuple struct: extract by position
        Fields::Unnamed(fields) => {
            let mut idents: Vec<syn::Ident> = Vec::new();
            let mut field_extractions: Vec<proc_macro2::TokenStream> = Vec::new();
            let n_fields = fields.unnamed.len();

            for (idx, f) in fields.unnamed.iter().enumerate() {
                let ident = syn::Ident::new(&format!("_field{idx}"), f.span());
                let ty = &f.ty;
                bounds.push(parse_quote!(#ty: ::miniextendr_api::from_r::TryFromSexp<Error = ::miniextendr_api::from_r::SexpError>));

                let idx_isize = idx as isize;
                field_extractions.push(quote! {
                    let #ident: #ty = list.get_index(#idx_isize)
                        .ok_or_else(|| ::miniextendr_api::from_r::SexpError::Length(
                            ::miniextendr_api::from_r::SexpLengthError {
                                expected: #n_fields,
                                actual: list.len() as usize,
                            }
                        ))?;
                });
                idents.push(ident);
            }

            quote! {
                #(#field_extractions)*
                Ok(Self( #(#idents),* ))
            }
        }

        // Unit struct: just return Self
        Fields::Unit => {
            quote! { Ok(Self) }
        }
    };

    // Extend where-clause with bounds
    let mut where_clause = where_clause.cloned().unwrap_or_else(|| syn::WhereClause {
        where_token: <syn::Token![where]>::default(),
        predicates: syn::punctuated::Punctuated::new(),
    });
    for b in bounds {
        where_clause.predicates.push(b);
    }

    let expand = quote! {
        impl #impl_generics ::miniextendr_api::list::TryFromList for #name #ty_generics #where_clause {
            type Error = ::miniextendr_api::from_r::SexpError;

            fn try_from_list(list: ::miniextendr_api::list::List) -> Result<Self, Self::Error> {
                #from_list_body
            }
        }
    };

    Ok(expand)
}

/// Derive `PrefersList`: add the marker and IntoR impl that routes through IntoList.
pub fn derive_prefer_list(input: DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expand = quote! {
        impl #impl_generics ::miniextendr_api::markers::PrefersList for #name #ty_generics #where_clause {}

        impl #impl_generics ::miniextendr_api::into_r::IntoR for #name #ty_generics #where_clause {
            #[inline]
            fn into_sexp(self) -> ::miniextendr_api::ffi::SEXP {
                ::miniextendr_api::list::IntoList::into_list(self).into_sexp()
            }

            #[inline]
            unsafe fn into_sexp_unchecked(self) -> ::miniextendr_api::ffi::SEXP {
                ::miniextendr_api::list::IntoList::into_list(self).into_sexp()
            }
        }
    };

    Ok(expand)
}

/// Derive `PrefersExternalPtr`: marker and IntoR impl that routes through ExternalPtr.
pub fn derive_prefer_externalptr(input: DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expand = quote! {
        impl #impl_generics ::miniextendr_api::markers::PrefersExternalPtr for #name #ty_generics #where_clause {}

        impl #impl_generics ::miniextendr_api::into_r::IntoR for #name #ty_generics #where_clause {
            #[inline]
            fn into_sexp(self) -> ::miniextendr_api::ffi::SEXP {
                ::miniextendr_api::externalptr::ExternalPtr::new(self).into_sexp()
            }

            #[inline]
            unsafe fn into_sexp_unchecked(self) -> ::miniextendr_api::ffi::SEXP {
                ::miniextendr_api::externalptr::ExternalPtr::new(self).into_sexp()
            }
        }
    };

    Ok(expand)
}

/// Derive `PreferRNativeType`: marker and IntoR impl that routes through native R vector allocation.
///
/// The type must also derive `RNativeType` for this to work.
pub fn derive_prefer_rnative(input: DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expand = quote! {
        impl #impl_generics ::miniextendr_api::markers::PrefersRNativeType for #name #ty_generics #where_clause {}

        impl #impl_generics ::miniextendr_api::into_r::IntoR for #name #ty_generics #where_clause {
            #[inline]
            fn into_sexp(self) -> ::miniextendr_api::ffi::SEXP {
                ::miniextendr_api::into_r::IntoR::into_sexp(
                    ::miniextendr_api::convert::AsRNative(self)
                )
            }

            #[inline]
            unsafe fn into_sexp_unchecked(self) -> ::miniextendr_api::ffi::SEXP {
                ::miniextendr_api::into_r::IntoR::into_sexp_unchecked(
                    ::miniextendr_api::convert::AsRNative(self)
                )
            }
        }
    };

    Ok(expand)
}
