use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Fields, parse_quote, spanned::Spanned};

fn field_is_ignored(field: &syn::Field) -> syn::Result<bool> {
    let mut ignored = false;

    for attr in &field.attrs {
        if !attr.path().is_ident("into_list") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("ignore") {
                ignored = true;
                return Ok(());
            }

            Err(meta.error("unknown #[into_list(...)] option; supported: ignore"))
        })?;
    }

    Ok(ignored)
}

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
                if field_is_ignored(f)? {
                    continue;
                }
                let ty = &f.ty;
                bounds.push(parse_quote!(#ty: ::miniextendr_api::into_r::IntoR));
                names.push(ident.to_string());
                idents.push(ident);
            }

            let pat = if idents.is_empty() {
                quote! { { .. } }
            } else {
                quote! { { #(#idents),*, .. } }
            };
            // Use from_raw_pairs to allow heterogeneous field types
            let construction = quote! {
                ::miniextendr_api::list::List::from_raw_pairs(vec![ #( (#names, #idents.into_sexp()) ),* ])
            };
            (pat, construction)
        }

        // Tuple struct: create unnamed R list (positional access)
        Fields::Unnamed(fields) => {
            let mut pat_elems: Vec<proc_macro2::TokenStream> = Vec::new();
            let mut value_idents: Vec<syn::Ident> = Vec::new();

            for (idx, f) in fields.unnamed.iter().enumerate() {
                if field_is_ignored(f)? {
                    pat_elems.push(quote! { _ });
                    continue;
                }
                let ident = syn::Ident::new(&format!("_field{idx}"), f.span());
                let ty = &f.ty;
                bounds.push(parse_quote!(#ty: ::miniextendr_api::into_r::IntoR));
                pat_elems.push(quote! { #ident });
                value_idents.push(ident);
            }

            let pat = quote! { ( #(#pat_elems),* ) };
            let construction = quote! {
                ::miniextendr_api::list::List::from_raw_values(vec![ #( #value_idents.into_sexp() ),* ])
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
            let mut field_extractions: Vec<proc_macro2::TokenStream> = Vec::new();
            let mut field_inits: Vec<proc_macro2::TokenStream> = Vec::new();

            for f in fields.named.iter() {
                let ident = f.ident.as_ref().unwrap().clone();
                let ty = &f.ty;

                if field_is_ignored(f)? {
                    bounds.push(parse_quote!(#ty: ::core::default::Default));
                    field_inits.push(quote! { #ident: ::core::default::Default::default() });
                    continue;
                }

                bounds.push(parse_quote!(#ty: ::miniextendr_api::from_r::TryFromSexp<Error = ::miniextendr_api::from_r::SexpError>));

                let name_str = ident.to_string();
                field_extractions.push(quote! {
                    let #ident: #ty = list.get_named(#name_str)
                        .ok_or_else(|| ::miniextendr_api::from_r::SexpError::MissingField(#name_str.into()))?;
                });
                field_inits.push(quote! { #ident });
            }

            quote! {
                #(#field_extractions)*
                Ok(Self { #(#field_inits),* })
            }
        }

        // Tuple struct: extract by position
        Fields::Unnamed(fields) => {
            let mut field_extractions: Vec<proc_macro2::TokenStream> = Vec::new();
            let mut ctor_args: Vec<proc_macro2::TokenStream> = Vec::new();
            let mut ignored_fields: Vec<bool> = Vec::with_capacity(fields.unnamed.len());
            for f in fields.unnamed.iter() {
                ignored_fields.push(field_is_ignored(f)?);
            }
            let input_fields: usize = ignored_fields.iter().filter(|&&b| !b).count();
            let mut input_idx: usize = 0;

            for (idx, f) in fields.unnamed.iter().enumerate() {
                let ty = &f.ty;

                if ignored_fields[idx] {
                    bounds.push(parse_quote!(#ty: ::core::default::Default));
                    ctor_args.push(quote! { ::core::default::Default::default() });
                    continue;
                }

                let ident = syn::Ident::new(&format!("_field{idx}"), f.span());
                bounds.push(parse_quote!(#ty: ::miniextendr_api::from_r::TryFromSexp<Error = ::miniextendr_api::from_r::SexpError>));

                let idx_isize = input_idx as isize;
                field_extractions.push(quote! {
                    let #ident: #ty = list.get_index(#idx_isize)
                        .ok_or_else(|| ::miniextendr_api::from_r::SexpError::Length(
                            ::miniextendr_api::from_r::SexpLengthError {
                                expected: #input_fields,
                                actual: list.len() as usize,
                            }
                        ))?;
                });
                ctor_args.push(quote! { #ident });
                input_idx += 1;
            }

            quote! {
                #(#field_extractions)*
                Ok(Self( #(#ctor_args),* ))
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
