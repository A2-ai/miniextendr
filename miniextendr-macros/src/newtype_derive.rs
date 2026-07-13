//! `#[derive(TryFromSexp)]` / `#[derive(IntoR)]` — forward R↔Rust conversions
//! from a single-field newtype to its inner type.
//!
//! For a newtype `struct UserId(Uuid)` (or `struct UserId { inner: Uuid }`):
//!
//! - `#[derive(TryFromSexp)]` generates the R → Rust direction: a scalar
//!   `TryFromSexp` impl that forwards to the inner type, plus a
//!   `miniextendr_api::FromRNewtype` marker impl. The marker lets
//!   the container blankets in `miniextendr_api::newtype` light up
//!   `Vec<UserId>` / `Option<UserId>` / `Vec<Option<UserId>>` — see issue #844.
//! - `#[derive(IntoR)]` generates the Rust → R direction: a scalar `IntoR` impl
//!   that forwards to the inner type, plus
//!   `miniextendr_api::IntoRNewtype` (for `Option` / `Vec<Option>`) and a
//!   concrete `miniextendr_api::IntoRVecElement`
//!   impl (for `Vec`).
//!
//! Direction is chosen by *which* derive you list — there are no attributes.
//! Some inner types are convertible in only one direction (a compiled
//! `regex::Regex` reads from R but cannot be written back): derive only
//! `TryFromSexp` for those.
//!
//! ```ignore
//! #[derive(TryFromSexp)]            // R -> Rust only
//! struct Pattern(regex::Regex);
//!
//! #[derive(TryFromSexp, IntoR)]     // round-trip
//! struct UserId(uuid::Uuid);
//! ```
//!
//! Do not derive both `IntoR` and `MatchArg` on the same type: both would feed
//! the single `IntoR for Vec<T>` blanket slot and collide (E0119).

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Type};

/// The single field of a newtype: its inner type plus how to wrap/unwrap it.
struct Newtype {
    inner: Type,
    /// `Some(field_ident)` for named newtypes, `None` for tuple newtypes.
    named_field: Option<syn::Ident>,
}

impl Newtype {
    /// `Self(<val>)` or `Self { field: <val> }`.
    fn wrap(&self, val: &TokenStream) -> TokenStream {
        match &self.named_field {
            Some(field) => quote! { Self { #field: #val } },
            None => quote! { Self(#val) },
        }
    }

    /// `<binding>.0` or `<binding>.field`.
    fn unwrap(&self, binding: &TokenStream) -> TokenStream {
        match &self.named_field {
            Some(field) => quote! { #binding.#field },
            None => quote! { #binding.0 },
        }
    }
}

fn parse_newtype(input: &DeriveInput) -> syn::Result<Newtype> {
    let data = match &input.data {
        Data::Struct(data) => data,
        _ => {
            return Err(syn::Error::new_spanned(
                &input.ident,
                "this derive only works on single-field newtype structs",
            ));
        }
    };
    match &data.fields {
        Fields::Unnamed(fields) if fields.unnamed.len() == 1 => Ok(Newtype {
            inner: fields.unnamed.first().unwrap().ty.clone(),
            named_field: None,
        }),
        Fields::Named(fields) if fields.named.len() == 1 => {
            let field = fields.named.first().unwrap();
            Ok(Newtype {
                inner: field.ty.clone(),
                named_field: Some(field.ident.clone().unwrap()),
            })
        }
        _ => Err(syn::Error::new_spanned(
            &input.ident,
            "this derive requires a struct with exactly one field",
        )),
    }
}

/// Build a `where` clause that merges the struct's existing predicates with one
/// extra predicate (the `Inner: Trait` bound the forwarding impl requires).
fn where_with(base: &Option<syn::WhereClause>, extra: TokenStream) -> TokenStream {
    let mut preds: Vec<TokenStream> = base
        .iter()
        .flat_map(|w| w.predicates.iter().map(|p| quote! { #p }))
        .collect();
    preds.push(extra);
    quote! { where #(#preds),* }
}

/// `#[derive(TryFromSexp)]`: scalar forwarding `TryFromSexp` + `FromRNewtype` marker.
pub fn derive_try_from_sexp(input: DeriveInput) -> syn::Result<TokenStream> {
    let nt = parse_newtype(&input)?;
    let inner = &nt.inner;
    let name = &input.ident;
    let (impl_generics, ty_generics, _) = input.generics.split_for_impl();
    let base_where = &input.generics.where_clause;

    let wrap_val = nt.wrap(&quote! { val });
    let wrap_inner = nt.wrap(&quote! { inner });
    let from_where = where_with(
        base_where,
        quote! { #inner: ::miniextendr_api::TryFromSexp },
    );

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::miniextendr_api::TryFromSexp for #name #ty_generics #from_where {
            type Error = <#inner as ::miniextendr_api::TryFromSexp>::Error;
            #[inline]
            fn try_from_sexp(sexp: ::miniextendr_api::SEXP) -> ::core::result::Result<Self, Self::Error> {
                <#inner as ::miniextendr_api::TryFromSexp>::try_from_sexp(sexp).map(|val| #wrap_val)
            }
            #[inline]
            unsafe fn try_from_sexp_unchecked(sexp: ::miniextendr_api::SEXP) -> ::core::result::Result<Self, Self::Error> {
                unsafe { <#inner as ::miniextendr_api::TryFromSexp>::try_from_sexp_unchecked(sexp) }.map(|val| #wrap_val)
            }
        }

        #[automatically_derived]
        impl #impl_generics ::miniextendr_api::FromRNewtype for #name #ty_generics #base_where {
            type Inner = #inner;
            #[inline]
            fn from_inner(inner: #inner) -> Self {
                #wrap_inner
            }
        }
    })
}

/// `#[derive(IntoR)]`: scalar forwarding `IntoR` + `IntoRNewtype` marker (for
/// `Option`/`Vec<Option>` blankets) + concrete `IntoRVecElement` (for `Vec`).
pub fn derive_into_r(input: DeriveInput) -> syn::Result<TokenStream> {
    let nt = parse_newtype(&input)?;
    let inner = &nt.inner;
    let name = &input.ident;
    let (impl_generics, ty_generics, _) = input.generics.split_for_impl();
    let base_where = &input.generics.where_clause;

    let unwrap_self = nt.unwrap(&quote! { self });
    let unwrap_v = nt.unwrap(&quote! { v });
    let into_where = where_with(base_where, quote! { #inner: ::miniextendr_api::IntoR });
    // The Vec element impl forwards to `Vec<Inner>: IntoR`; gate it on that so a
    // newtype whose inner lacks a vector conversion still gets the scalar IntoR.
    let vec_elem_where = where_with(
        base_where,
        quote! { ::std::vec::Vec<#inner>: ::miniextendr_api::IntoR },
    );

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::miniextendr_api::IntoR for #name #ty_generics #into_where {
            type Error = <#inner as ::miniextendr_api::IntoR>::Error;
            #[inline]
            fn try_into_sexp(self) -> ::core::result::Result<::miniextendr_api::SEXP, Self::Error> {
                <#inner as ::miniextendr_api::IntoR>::try_into_sexp(#unwrap_self)
            }
            #[inline]
            unsafe fn try_into_sexp_unchecked(self) -> ::core::result::Result<::miniextendr_api::SEXP, Self::Error> {
                unsafe { <#inner as ::miniextendr_api::IntoR>::try_into_sexp_unchecked(#unwrap_self) }
            }
            #[inline]
            fn into_sexp(self) -> ::miniextendr_api::SEXP {
                <#inner as ::miniextendr_api::IntoR>::into_sexp(#unwrap_self)
            }
            #[inline]
            unsafe fn into_sexp_unchecked(self) -> ::miniextendr_api::SEXP {
                unsafe { <#inner as ::miniextendr_api::IntoR>::into_sexp_unchecked(#unwrap_self) }
            }
        }

        #[automatically_derived]
        impl #impl_generics ::miniextendr_api::IntoRNewtype for #name #ty_generics #base_where {
            type Inner = #inner;
            #[inline]
            fn into_inner(self) -> #inner {
                #unwrap_self
            }
        }

        #[automatically_derived]
        impl #impl_generics ::miniextendr_api::IntoRVecElement for #name #ty_generics #vec_elem_where {
            #[inline]
            fn elements_into_sexp(values: ::std::vec::Vec<Self>) -> ::miniextendr_api::SEXP {
                <::std::vec::Vec<#inner> as ::miniextendr_api::IntoR>::into_sexp(
                    values.into_iter().map(|v| #unwrap_v).collect::<::std::vec::Vec<#inner>>()
                )
            }
        }
    })
}
