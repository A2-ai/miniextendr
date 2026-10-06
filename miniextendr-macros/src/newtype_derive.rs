//! `#[derive(TryFromSexp)]` / `#[derive(IntoR)]` — forward R↔Rust conversions
//! from a single-field newtype to its inner type.
//!
//! For a newtype `struct UserId(Uuid)` (or `struct UserId { inner: Uuid }`):
//!
//! - `#[derive(TryFromSexp)]` generates the R → Rust direction: a scalar
//!   `TryFromSexp` impl that forwards to the inner type, plus a
//!   `miniextendr_api::TryFromSexpElement` marker impl. The marker lets
//!   the container blankets in `miniextendr_api::newtype` light up
//!   `Vec<UserId>` / `Option<UserId>` / `Vec<Option<UserId>>` — see issues
//!   #844 and #1766.
//! - `#[derive(IntoR)]` generates the Rust → R direction: a scalar `IntoR` impl
//!   that forwards to the inner type, plus
//!   `miniextendr_api::IntoRNewtype` (for `Option` / `Vec<Option>`) and a
//!   concrete `miniextendr_api::IntoRVecElement`
//!   impl (for `Vec`).
//!
//! Direction is chosen by *which* derive you list. Some inner types are
//! convertible in only one direction (a compiled `regex::Regex` reads from R
//! but cannot be written back): derive only `TryFromSexp` for those.
//!
//! ```ignore
//! #[derive(TryFromSexp)]            // R -> Rust only
//! struct Pattern(regex::Regex);
//!
//! #[derive(TryFromSexp, IntoR)]     // round-trip
//! struct UserId(uuid::Uuid);
//! ```
//!
//! `#[derive(TryFromSexp)]` takes one attribute, on the struct:
//! `#[try_from_sexp(validate = path::to::check)]`, with `check` a
//! `fn(SEXP) -> Result<(), E>` and `E: Into<SexpError>` (an `RError` for a
//! classed refusal). It becomes `TryFromSexpElement::check_sexp`, which the
//! scalar impl and every container run on the R value before the inner type
//! reads it (#1815). The scalar impl's error is then `SexpError`, so the inner
//! type's error must convert into it.
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

/// Build a `where` clause that merges the struct's existing predicates with
/// extra predicates (the `Inner: Trait` bounds the forwarding impl requires).
fn where_with(base: &Option<syn::WhereClause>, extra: &[TokenStream]) -> TokenStream {
    let mut preds: Vec<TokenStream> = base
        .iter()
        .flat_map(|w| w.predicates.iter().map(|p| quote! { #p }))
        .collect();
    preds.extend(extra.iter().cloned());
    quote! { where #(#preds),* }
}

/// The `#[try_from_sexp(...)]` options of `#[derive(TryFromSexp)]`.
#[derive(Default)]
struct TryFromSexpOptions {
    /// `validate = path`: the newtype's `TryFromSexpElement::check_sexp`.
    validate: Option<syn::Path>,
}

/// Read the struct's `#[try_from_sexp(...)]` attributes. The only key is
/// `validate = path`; any other key, a repeated `validate`, or the attribute on
/// the field is a spanned error.
fn parse_try_from_sexp_options(input: &DeriveInput) -> syn::Result<TryFromSexpOptions> {
    let mut options = TryFromSexpOptions::default();
    for attr in input
        .attrs
        .iter()
        .filter(|a| a.path().is_ident("try_from_sexp"))
    {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("validate") {
                if options.validate.is_some() {
                    return Err(meta.error("duplicate `validate` in `#[try_from_sexp(...)]`"));
                }
                options.validate = Some(meta.value()?.parse()?);
                Ok(())
            } else {
                Err(meta.error(
                    "unknown `#[try_from_sexp(...)]` key; the only key is `validate = path::to::check`",
                ))
            }
        })?;
    }
    if let Data::Struct(data) = &input.data {
        for field in &data.fields {
            if let Some(attr) = field
                .attrs
                .iter()
                .find(|a| a.path().is_ident("try_from_sexp"))
            {
                return Err(syn::Error::new_spanned(
                    attr,
                    "`#[try_from_sexp(...)]` goes on the struct, not on its field",
                ));
            }
        }
    }
    Ok(options)
}

/// `#[derive(TryFromSexp)]`: scalar forwarding `TryFromSexp` + `TryFromSexpElement` marker.
///
/// With `#[try_from_sexp(validate = path)]` the marker's `check_sexp` calls
/// `path`, and the scalar impl runs it before the inner conversion, with
/// `SexpError` as its error (the inner type's error converts into it).
pub fn derive_try_from_sexp(input: DeriveInput) -> syn::Result<TokenStream> {
    let nt = parse_newtype(&input)?;
    let options = parse_try_from_sexp_options(&input)?;
    let inner = &nt.inner;
    let name = &input.ident;
    let (impl_generics, ty_generics, _) = input.generics.split_for_impl();
    let base_where = &input.generics.where_clause;

    let wrap_val = nt.wrap(&quote! { val });
    let wrap_inner = nt.wrap(&quote! { inner });
    let field = nt.unwrap(&quote! { self });
    let inner_converts = quote! { #inner: ::miniextendr_api::TryFromSexp };
    let inner_try_from = quote! { <#inner as ::miniextendr_api::TryFromSexp>::try_from_sexp(sexp) };
    let inner_try_from_unchecked = quote! {
        unsafe { <#inner as ::miniextendr_api::TryFromSexp>::try_from_sexp_unchecked(sexp) }
    };

    // Without `validate` the scalar impl is the inner type's, error and all.
    // With it, the check runs first and both errors become `SexpError`, the
    // error every container of the newtype has.
    let (from_where, error, try_from, try_from_unchecked, check_sexp) = match &options.validate {
        None => (
            where_with(base_where, &[inner_converts]),
            quote! { <#inner as ::miniextendr_api::TryFromSexp>::Error },
            quote! { #inner_try_from.map(|val| #wrap_val) },
            quote! { #inner_try_from_unchecked.map(|val| #wrap_val) },
            TokenStream::new(),
        ),
        Some(validate) => {
            let check = quote! {
                <Self as ::miniextendr_api::TryFromSexpElement>::check_sexp(sexp)?;
            };
            (
                where_with(
                    base_where,
                    &[
                        inner_converts,
                        quote! {
                            <#inner as ::miniextendr_api::TryFromSexp>::Error:
                                ::core::convert::Into<::miniextendr_api::from_r::SexpError>
                        },
                    ],
                ),
                quote! { ::miniextendr_api::from_r::SexpError },
                quote! {
                    #check
                    #inner_try_from.map(|val| #wrap_val).map_err(::core::convert::Into::into)
                },
                quote! {
                    #check
                    #inner_try_from_unchecked.map(|val| #wrap_val).map_err(::core::convert::Into::into)
                },
                quote! {
                    #[inline]
                    fn check_sexp(
                        sexp: ::miniextendr_api::SEXP,
                    ) -> ::core::result::Result<(), ::miniextendr_api::from_r::SexpError> {
                        #validate(sexp).map_err(::core::convert::Into::into)
                    }
                },
            )
        }
    };

    // `__mx_has_na` forwards too, so `no_na` on a newtype of a reading marker
    // (`struct Dose(AsNumeric)`) refuses what the marker reads as `NA`. So
    // does `__mx_input_has_na`, so a newtype arm of an `Either` is checked as
    // its inner type: `struct Table(DataFrame)` keeps its `NA` cells.
    let expectation = expectation_const(inner);
    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::miniextendr_api::TryFromSexp for #name #ty_generics #from_where {
            type Error = #error;
            const NATIVE_BORROW: ::core::option::Option<::miniextendr_api::from_r::NativeBorrow> =
                <#inner as ::miniextendr_api::TryFromSexp>::NATIVE_BORROW;
            const CHARACTER_ONLY: bool = <#inner as ::miniextendr_api::TryFromSexp>::CHARACTER_ONLY;
            const __MX_EXPECTATION: ::core::option::Option<&'static str> = #expectation;
            #[inline]
            fn __mx_has_na(&self) -> bool {
                <#inner as ::miniextendr_api::TryFromSexp>::__mx_has_na(&#field)
            }
            #[inline]
            fn __mx_input_has_na(&self, input: ::miniextendr_api::SEXP) -> bool {
                <#inner as ::miniextendr_api::TryFromSexp>::__mx_input_has_na(&#field, input)
            }
            #[inline]
            fn try_from_sexp(sexp: ::miniextendr_api::SEXP) -> ::core::result::Result<Self, #error> {
                #try_from
            }
            #[inline]
            unsafe fn try_from_sexp_unchecked(sexp: ::miniextendr_api::SEXP) -> ::core::result::Result<Self, #error> {
                #try_from_unchecked
            }
        }

        #[automatically_derived]
        impl #impl_generics ::miniextendr_api::TryFromSexpElement for #name #ty_generics #base_where {
            type Inner = #inner;
            #[inline]
            fn from_inner(inner: #inner) -> Self {
                #wrap_inner
            }
            #check_sexp
        }
    })
}

/// The newtype's `TryFromSexp::__MX_EXPECTATION`: what an argument of the
/// inner type must be, in R terms, so a newtype parameter or `Either` arm
/// reads as its inner type does (`struct Dose(AsNumeric)`: `a single
/// number`). The macro's own wording when it knows the inner type
/// ([`crate::r_preconditions::conversion_expectation`]), else the inner
/// type's constant, which a newtype of a newtype carries through.
fn expectation_const(inner: &Type) -> TokenStream {
    match crate::r_preconditions::conversion_expectation(inner, false) {
        Some(expected) => quote! { ::core::option::Option::Some(#expected) },
        None => quote! { <#inner as ::miniextendr_api::TryFromSexp>::__MX_EXPECTATION },
    }
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
    let into_where = where_with(base_where, &[quote! { #inner: ::miniextendr_api::IntoR }]);
    // The Vec element impl forwards to `Vec<Inner>: IntoR`; gate it on that so a
    // newtype whose inner lacks a vector conversion still gets the scalar IntoR.
    let vec_elem_where = where_with(
        base_where,
        &[quote! { ::std::vec::Vec<#inner>: ::miniextendr_api::IntoR }],
    );

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::miniextendr_api::IntoR for #name #ty_generics #into_where {
            type Error = <#inner as ::miniextendr_api::IntoR>::Error;
            #[inline]
            fn try_into_sexp(self) -> ::core::result::Result<::miniextendr_api::SEXP, <#inner as ::miniextendr_api::IntoR>::Error> {
                <#inner as ::miniextendr_api::IntoR>::try_into_sexp(#unwrap_self)
            }
            #[inline]
            unsafe fn try_into_sexp_unchecked(self) -> ::core::result::Result<::miniextendr_api::SEXP, <#inner as ::miniextendr_api::IntoR>::Error> {
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

#[cfg(test)]
mod tests {
    /// `#[try_from_sexp(validate = path)]` gives the element marker a
    /// `check_sexp` that calls the path, and the scalar impl runs it before
    /// the inner conversion, with `SexpError` as its error. Without the
    /// attribute the scalar impl keeps the inner error and no check is
    /// emitted (the trait's default accepts every value).
    #[test]
    fn try_from_sexp_derive_validate_emits_the_check() {
        let validated = super::derive_try_from_sexp(syn::parse_quote! {
            #[try_from_sexp(validate = checks::plain_number)]
            struct Duration(f64);
        })
        .unwrap()
        .to_string();
        assert!(
            validated.contains(
                "fn check_sexp (sexp : :: miniextendr_api :: SEXP ,) -> :: core :: result :: Result < () , :: miniextendr_api :: from_r :: SexpError > { checks :: plain_number (sexp) . map_err (:: core :: convert :: Into :: into) }"
            ),
            "{validated}"
        );
        assert!(
            validated.contains("type Error = :: miniextendr_api :: from_r :: SexpError ;"),
            "{validated}"
        );
        assert!(
            validated.contains(
                "< f64 as :: miniextendr_api :: TryFromSexp > :: Error : :: core :: convert :: Into < :: miniextendr_api :: from_r :: SexpError >"
            ),
            "{validated}"
        );
        // Both scalar paths run the check first.
        let check = "< Self as :: miniextendr_api :: TryFromSexpElement > :: check_sexp (sexp) ? ;";
        assert_eq!(validated.matches(check).count(), 2, "{validated}");

        let plain = super::derive_try_from_sexp(syn::parse_quote! { struct Duration(f64); })
            .unwrap()
            .to_string();
        assert!(!plain.contains("check_sexp"), "{plain}");
        assert!(
            plain.contains("type Error = < f64 as :: miniextendr_api :: TryFromSexp > :: Error ;"),
            "{plain}"
        );
    }

    /// `validate` is the only key, given once, and only on the struct.
    #[test]
    fn try_from_sexp_derive_rejects_bad_options() {
        let error = |input: syn::DeriveInput| {
            super::derive_try_from_sexp(input)
                .err()
                .expect("the options should be refused")
                .to_string()
        };
        assert!(
            error(syn::parse_quote! {
                #[try_from_sexp(check = plain_number)]
                struct Duration(f64);
            })
            .contains("unknown `#[try_from_sexp(...)]` key")
        );
        assert!(
            error(syn::parse_quote! {
                #[try_from_sexp(validate = a, validate = b)]
                struct Duration(f64);
            })
            .contains("duplicate `validate`")
        );
        assert!(
            error(syn::parse_quote! {
                struct Duration(#[try_from_sexp(validate = a)] f64);
            })
            .contains("goes on the struct")
        );
    }

    /// `#[derive(TryFromSexp)]` forwards the hidden `no_na` probes to the
    /// inner type, for tuple and named newtypes alike: `__mx_has_na` for the
    /// value, and `__mx_input_has_na` for the input of an `Either` arm.
    #[test]
    fn try_from_sexp_derive_forwards_no_na() {
        for (input, field) in [
            (syn::parse_quote! { struct Table(DataFrame); }, "self . 0"),
            (
                syn::parse_quote! { struct Table { value: DataFrame } },
                "self . value",
            ),
        ] {
            let out = super::derive_try_from_sexp(input).unwrap().to_string();
            let forward = |call: &str| {
                format!("< DataFrame as :: miniextendr_api :: TryFromSexp > :: {call}")
            };
            assert!(out.contains("fn __mx_has_na (& self) -> bool"), "{out}");
            assert!(
                out.contains(&forward(&format!("__mx_has_na (& {field})"))),
                "{out}"
            );
            assert!(
                out.contains(
                    "fn __mx_input_has_na (& self , input : :: miniextendr_api :: SEXP) -> bool"
                ),
                "{out}"
            );
            assert!(
                out.contains(&forward(&format!("__mx_input_has_na (& {field} , input)"))),
                "{out}"
            );
        }
    }

    /// `#[derive(TryFromSexp)]` declares the inner type's R-facing
    /// expectation: the macro's wording for an inner type it knows, else the
    /// inner type's own constant (a newtype of a newtype, a generic newtype).
    #[test]
    fn try_from_sexp_derive_declares_the_inner_expectation() {
        let expectation = |input: syn::DeriveInput| {
            let out = super::derive_try_from_sexp(input).unwrap().to_string();
            let start = out
                .find("const __MX_EXPECTATION")
                .unwrap_or_else(|| panic!("no expectation constant: {out}"));
            let end = start + out[start..].find(';').unwrap();
            out[start..end].trim_end().to_string()
        };
        let declares = |text: &str| {
            format!(
                "const __MX_EXPECTATION : :: core :: option :: Option < & 'static str > = \
                 :: core :: option :: Option :: Some ({text:?})"
            )
        };
        assert_eq!(
            expectation(syn::parse_quote! { struct Dose(AsNumeric); }),
            declares("a single number")
        );
        assert_eq!(
            expectation(syn::parse_quote! { struct Numbers { values: AsNumericVec } }),
            declares("numeric")
        );
        assert_eq!(
            expectation(syn::parse_quote! { struct Table(DataFrame); }),
            declares("a data frame")
        );
        for (input, inner) in [
            (syn::parse_quote! { struct Wrapped(Table); }, "Table"),
            (syn::parse_quote! { struct Id(Uuid); }, "Uuid"),
            (syn::parse_quote! { struct Any<T>(T); }, "T"),
        ] {
            assert_eq!(
                expectation(input),
                format!(
                    "const __MX_EXPECTATION : :: core :: option :: Option < & 'static str > = \
                     < {inner} as :: miniextendr_api :: TryFromSexp > :: __MX_EXPECTATION"
                )
            );
        }
    }
}
