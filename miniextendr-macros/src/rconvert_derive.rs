//! `#[derive(RConvert)]` — forward R↔Rust conversions from a newtype to its
//! inner type.
//!
//! For a single-field newtype `struct MyId(Uuid)` (or `struct MyId { inner: Uuid }`),
//! `#[derive(RConvert)]` generates `TryFromSexp` and `IntoR` impls that *forward*
//! to the inner type's existing conversions. The newtype therefore inherits the
//! inner type's exact behaviour — SEXPTYPE expectations, NA policy, error
//! messages — for free, with no per-type configuration.
//!
//! # Scalar only (orphan rule)
//!
//! The derive emits only the **scalar** impls (`MyId`), not the container shapes
//! (`Option<MyId>`, `Vec<MyId>`, `Vec<Option<MyId>>`). Those are impossible to
//! emit in the user's crate: `Option`/`Vec` are not `#[fundamental]`, so the
//! local newtype is "covered" by them and the orphan rule (E0117) forbids
//! `impl TryFromSexp for Vec<MyId>` anywhere except the crate that defines the
//! trait. This is the same constraint that stops `#[derive(RNativeType)]` from
//! granting `Vec<T>` conversions, and the reason the framework deliberately has
//! no `impl<T: RNativeType> IntoR for Vec<T>` blanket. To take a vector of
//! newtypes across the FFI boundary, accept the inner type's vector
//! (`Vec<Uuid>`) and wrap per element inside the function body.
//!
//! # Direction control
//!
//! Some inner types are convertible in only one direction (e.g. a compiled
//! `regex::Regex` reads from R but cannot be written back). Use the attribute to
//! drop the unsupported direction:
//!
//! ```ignore
//! #[derive(RConvert)]
//! #[rconvert(into = false)]   // emit TryFromSexp only
//! struct MyPattern(regex::Regex);
//! ```
//!
//! `forward` is accepted as an explicit (no-op) spelling of the default mode.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Type};

/// Which conversion directions the derive should emit.
struct RConvertOpts {
    /// Emit the `TryFromSexp` impl (R → Rust).
    emit_from: bool,
    /// Emit the `IntoR` impl (Rust → R).
    emit_into: bool,
}

impl Default for RConvertOpts {
    fn default() -> Self {
        RConvertOpts {
            emit_from: true,
            emit_into: true,
        }
    }
}

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

fn parse_opts(input: &DeriveInput) -> syn::Result<RConvertOpts> {
    let mut opts = RConvertOpts::default();
    for attr in &input.attrs {
        if !attr.path().is_ident("rconvert") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("forward") {
                // Explicit spelling of the default forwarding mode — accept.
                Ok(())
            } else if meta.path.is_ident("from") {
                opts.emit_from = meta.value()?.parse::<syn::LitBool>()?.value();
                Ok(())
            } else if meta.path.is_ident("into") {
                opts.emit_into = meta.value()?.parse::<syn::LitBool>()?.value();
                Ok(())
            } else {
                Err(meta.error(
                    "unknown #[rconvert] option; expected `forward`, `from = <bool>`, or `into = <bool>`",
                ))
            }
        })?;
    }
    if !opts.emit_from && !opts.emit_into {
        return Err(syn::Error::new_spanned(
            input,
            "#[rconvert]: at least one of `from` / `into` must remain enabled",
        ));
    }
    Ok(opts)
}

fn parse_newtype(input: &DeriveInput) -> syn::Result<Newtype> {
    let data = match &input.data {
        Data::Struct(data) => data,
        _ => {
            return Err(syn::Error::new_spanned(
                &input.ident,
                "#[derive(RConvert)] only works on newtype structs",
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
            "#[derive(RConvert)] requires a struct with exactly one field",
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

pub fn derive_rconvert(input: DeriveInput) -> syn::Result<TokenStream> {
    let opts = parse_opts(&input)?;
    let nt = parse_newtype(&input)?;
    let inner = &nt.inner;

    let name = &input.ident;
    let (impl_generics, ty_generics, _) = input.generics.split_for_impl();
    let base_where = &input.generics.where_clause;

    let mut out = TokenStream::new();

    if opts.emit_from {
        let val = quote! { val };
        let wrap = nt.wrap(&val);
        let w = where_with(
            base_where,
            quote! { #inner: ::miniextendr_api::TryFromSexp },
        );
        out.extend(quote! {
            #[automatically_derived]
            impl #impl_generics ::miniextendr_api::TryFromSexp for #name #ty_generics #w {
                type Error = <#inner as ::miniextendr_api::TryFromSexp>::Error;
                #[inline]
                fn try_from_sexp(sexp: ::miniextendr_api::SEXP) -> ::core::result::Result<Self, Self::Error> {
                    <#inner as ::miniextendr_api::TryFromSexp>::try_from_sexp(sexp).map(|val| #wrap)
                }
                #[inline]
                unsafe fn try_from_sexp_unchecked(sexp: ::miniextendr_api::SEXP) -> ::core::result::Result<Self, Self::Error> {
                    unsafe { <#inner as ::miniextendr_api::TryFromSexp>::try_from_sexp_unchecked(sexp) }.map(|val| #wrap)
                }
            }
        });
    }

    if opts.emit_into {
        let this = quote! { self };
        let unwrap_self = nt.unwrap(&this);
        let w = where_with(base_where, quote! { #inner: ::miniextendr_api::IntoR });
        out.extend(quote! {
            #[automatically_derived]
            impl #impl_generics ::miniextendr_api::IntoR for #name #ty_generics #w {
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
        });
    }

    Ok(out)
}
