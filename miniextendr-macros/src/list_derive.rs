use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Fields, parse_quote, spanned::Spanned};

/// Derive `IntoList`, `TryIntoList`, and `TryFromList` for structs.
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

    // Collect field info
    let mut bounds: Vec<syn::WherePredicate> = Vec::new();
    let mut pair_exprs: Vec<(String, syn::Ident)> = Vec::new();
    let mut field_specs: Vec<(syn::Ident, syn::Type, String, usize)> = Vec::new();

    let destructure_pat: TokenStream = match &struct_data.fields {
        Fields::Named(fields) => {
            for (idx, f) in fields.named.iter().enumerate() {
                let ident = f.ident.as_ref().unwrap().clone();
                let ty = f.ty.clone();
                bounds.push(parse_quote!(#ty: ::miniextendr_api::into_r::IntoR));
                bounds.push(
                    parse_quote!(#ty: ::miniextendr_api::from_r::TryFromSexp<Error = ::miniextendr_api::from_r::SexpError>),
                );
                let name_str = ident.to_string();
                pair_exprs.push((name_str.clone(), ident.clone()));
                field_specs.push((ident, ty, name_str, idx));
            }
            let idents: Vec<_> = fields
                .named
                .iter()
                .map(|f| f.ident.as_ref().unwrap())
                .collect();
            quote! { { #(#idents),* } }
        }
        Fields::Unnamed(fields) => {
            let mut idents = Vec::new();
            for (idx, f) in fields.unnamed.iter().enumerate() {
                let ident = syn::Ident::new(&format!("_field{idx}"), f.span());
                let ty = f.ty.clone();
                bounds.push(parse_quote!(#ty: ::miniextendr_api::into_r::IntoR));
                bounds.push(
                    parse_quote!(#ty: ::miniextendr_api::from_r::TryFromSexp<Error = ::miniextendr_api::from_r::SexpError>),
                );
                let name_str = idx.to_string();
                pair_exprs.push((name_str.clone(), ident.clone()));
                field_specs.push((ident.clone(), ty, name_str, idx));
                idents.push(ident);
            }
            quote! { ( #(#idents),* ) }
        }
        Fields::Unit => quote! {},
    };

    // Extend where-clause
    let mut where_clause = where_clause.cloned().unwrap_or_else(|| syn::WhereClause {
        where_token: <syn::Token![where]>::default(),
        predicates: syn::punctuated::Punctuated::new(),
    });
    for b in bounds {
        where_clause.predicates.push(b);
    }

    let names: Vec<String> = pair_exprs.iter().map(|(n, _)| n.clone()).collect();
    let vars_for_into: Vec<_> = pair_exprs.iter().map(|(_, ident)| ident).collect();

    let expand = quote! {
        impl #impl_generics ::miniextendr_api::markers::IsIntoList for #name #ty_generics #where_clause {}

        impl #impl_generics ::miniextendr_api::list::IntoList for #name #ty_generics #where_clause {
            fn into_list(self) -> ::miniextendr_api::list::List {
                let Self #destructure_pat = self;
                ::miniextendr_api::list::List::from_pairs(vec![ #( (#names, #vars_for_into) ),* ])
            }
        }

    };

    Ok(expand)
}
