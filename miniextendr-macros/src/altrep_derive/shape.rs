//! What the ALTREP derives read from the struct: field selectors, the slice
//! field, and the generic parameters every emitted impl carries.

use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::spanned::Spanned;

/// The field kinds a derive reads as a slice, for messages.
pub(crate) const SLICE_KINDS: &str = "`[T; N]`, `Vec<T>`, `Box<[T]>` or `&'static [T]`";

// region: Selectors

/// A `len` / `elt` / `elt_delegate` / `field` selector: a field name, or a
/// tuple struct's index (`"0"`).
pub(crate) struct Selector {
    /// The field, spanned at the selector string.
    pub(crate) member: syn::Member,
    /// The selector string's span.
    pub(crate) span: Span,
}

impl Selector {
    /// Parse a selector string.
    pub(crate) fn parse(lit: &syn::LitStr) -> syn::Result<Self> {
        let member = lit.parse::<syn::Member>().map_err(|_| {
            syn::Error::new(
                lit.span(),
                format!(
                    "`{}` is not a field name or a tuple index",
                    lit.value().escape_default()
                ),
            )
        })?;
        Ok(Self {
            member,
            span: lit.span(),
        })
    }

    /// Check that the struct has the field.
    pub(crate) fn check(&self, input: &syn::DeriveInput) -> syn::Result<()> {
        self.field(input).map(drop)
    }

    /// The field the selector names.
    fn field<'a>(&self, input: &'a syn::DeriveInput) -> syn::Result<&'a syn::Field> {
        let syn::Data::Struct(data) = &input.data else {
            return Err(syn::Error::new(
                self.span,
                "a field selector needs a struct",
            ));
        };
        data.fields
            .iter()
            .zip(field_members(&data.fields))
            .find(|(_, member)| *member == self.member)
            .map(|(field, _)| field)
            .ok_or_else(|| {
                let known: Vec<String> = field_members(&data.fields)
                    .map(|member| format!("`{}`", member_name(&member)))
                    .collect();
                let fields = if known.is_empty() {
                    "it has no fields".to_owned()
                } else {
                    format!("its fields are {}", known.join(", "))
                };
                syn::Error::new(
                    self.span,
                    format!(
                        "`{}` has no field `{}`; {fields}",
                        input.ident,
                        member_name(&self.member)
                    ),
                )
            })
    }
}

/// The members of a struct's fields, in order: names, or tuple indices.
fn field_members(fields: &syn::Fields) -> impl Iterator<Item = syn::Member> + '_ {
    fields
        .iter()
        .enumerate()
        .map(|(index, field)| match &field.ident {
            Some(ident) => syn::Member::Named(ident.clone()),
            None => syn::Member::Unnamed(syn::Index {
                index: u32::try_from(index).expect("a struct has fewer than 2^32 fields"),
                span: field.span(),
            }),
        })
}

/// A member as the user writes it in a selector.
fn member_name(member: &syn::Member) -> String {
    match member {
        syn::Member::Named(ident) => crate::naming::ident_name(ident),
        syn::Member::Unnamed(index) => index.index.to_string(),
    }
}
// endregion

// region: The slice field

/// The struct field a slice ALTREP reads.
pub(crate) struct SliceField {
    /// The field, spanned at its type.
    pub(crate) member: syn::Member,
    /// The element type, as written in the field's type.
    pub(crate) elem: syn::Type,
    /// A `&'static [T]` field: R reads it in place, never writes it.
    pub(crate) borrowed: bool,
}

impl SliceField {
    /// The elements, as `&[elem]`, in a `&self` method.
    pub(crate) fn values(&self) -> TokenStream {
        let Self { member, elem, .. } = self;
        quote! { ::core::convert::AsRef::<[#elem]>::as_ref(&self.#member) }
    }

    /// The elements, as `&mut [elem]`, in a `&mut self` method; `None` for
    /// a borrowed field.
    pub(crate) fn values_mut(&self) -> Option<TokenStream> {
        let Self {
            member,
            elem,
            borrowed,
        } = self;
        (!borrowed).then(|| quote! { ::core::convert::AsMut::<[#elem]>::as_mut(&mut self.#member) })
    }
}

/// The element type of a field the derives read as a slice, and whether it
/// is borrowed (`&'static [T]`).
fn slice_element(ty: &syn::Type) -> Option<(syn::Type, bool)> {
    match ty {
        syn::Type::Group(group) => slice_element(&group.elem),
        syn::Type::Paren(paren) => slice_element(&paren.elem),
        syn::Type::Array(array) => Some(((*array.elem).clone(), false)),
        syn::Type::Reference(reference)
            if reference.mutability.is_none()
                && reference
                    .lifetime
                    .as_ref()
                    .is_some_and(|lifetime| lifetime.ident == "static") =>
        {
            match &*reference.elem {
                syn::Type::Slice(slice) => Some(((*slice.elem).clone(), true)),
                _ => None,
            }
        }
        syn::Type::Path(path) if path.qself.is_none() => {
            let segment = path.path.segments.last()?;
            let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
                return None;
            };
            let mut types = args.args.iter().filter_map(|arg| match arg {
                syn::GenericArgument::Type(ty) => Some(ty),
                _ => None,
            });
            let (Some(inner), None) = (types.next(), types.next()) else {
                return None;
            };
            if segment.ident == "Vec" {
                Some((inner.clone(), false))
            } else if segment.ident == "Box" {
                match inner {
                    syn::Type::Slice(slice) => Some(((*slice.elem).clone(), false)),
                    _ => None,
                }
            } else {
                None
            }
        }
        _ => None,
    }
}

/// The struct's slice fields: not `#[r_data]` sidecars.
fn slice_candidates(input: &syn::DeriveInput) -> Vec<SliceField> {
    let syn::Data::Struct(data) = &input.data else {
        return Vec::new();
    };
    data.fields
        .iter()
        .zip(field_members(&data.fields))
        .filter(|(field, _)| !field.attrs.iter().any(|a| a.path().is_ident("r_data")))
        .filter_map(|(field, member)| {
            let (elem, borrowed) = slice_element(&field.ty)?;
            Some(SliceField {
                member: respan_member(member, field.ty.span()),
                elem,
                borrowed,
            })
        })
        .collect()
}

fn respan_member(member: syn::Member, span: Span) -> syn::Member {
    match member {
        syn::Member::Named(ident) => syn::Member::Named(ident),
        syn::Member::Unnamed(index) => syn::Member::Unnamed(syn::Index {
            index: index.index,
            span,
        }),
    }
}

/// The slice field a derive reads: the one `field = "..."` names, else the
/// struct's only slice field. `Ok(None)` when the struct has none.
///
/// `derive` names the derive in messages.
pub(crate) fn slice_field(
    input: &syn::DeriveInput,
    selector: Option<&Selector>,
    derive: &str,
) -> syn::Result<Option<SliceField>> {
    if let Some(selector) = selector {
        let field = selector.field(input)?;
        let Some((elem, borrowed)) = slice_element(&field.ty) else {
            return Err(syn::Error::new(
                selector.span,
                format!(
                    "`field = \"{}\"` names a `{}` field; `#[derive({derive})]` reads {SLICE_KINDS}",
                    member_name(&selector.member),
                    crate::type_inspect::type_display(&field.ty),
                ),
            ));
        };
        return Ok(Some(SliceField {
            member: selector.member.clone(),
            elem,
            borrowed,
        }));
    }
    let mut candidates = slice_candidates(input);
    match candidates.len() {
        0 => Ok(None),
        1 => Ok(candidates.pop()),
        _ => {
            let names: Vec<String> = candidates
                .iter()
                .map(|field| format!("`{}`", member_name(&field.member)))
                .collect();
            Err(syn::Error::new(
                input.ident.span(),
                format!(
                    "`{}` has several fields `#[derive({derive})]` can read ({}); choose one with \
                     `#[altrep(field = \"{}\")]`",
                    input.ident,
                    names.join(", "),
                    member_name(&candidates[0].member),
                ),
            ))
        }
    }
}
// endregion

// region: Generics

/// The struct's generics as every ALTREP impl takes them: no lifetimes (an
/// error), no parameter defaults, and `T: 'static` for each type parameter,
/// since R keeps the struct in an external pointer.
pub(crate) fn altrep_generics(generics: &syn::Generics) -> syn::Result<syn::Generics> {
    if let Some(lifetime) = generics.lifetimes().next() {
        return Err(syn::Error::new(
            lifetime.lifetime.span(),
            "ALTREP derives take no lifetime parameters: R keeps the struct in an external \
             pointer, which needs `'static` data; own the elements (`Vec<T>`, `Box<[T]>`, \
             `[T; N]`) or borrow `'static` ones (`&'static [T]`)",
        ));
    }
    let mut generics = generics.clone();
    let mut statics: Vec<syn::WherePredicate> = Vec::new();
    for param in &mut generics.params {
        match param {
            syn::GenericParam::Type(ty) => {
                ty.eq_token = None;
                ty.default = None;
                let ident = &ty.ident;
                statics.push(syn::parse_quote!(#ident: 'static));
            }
            syn::GenericParam::Const(constant) => {
                constant.eq_token = None;
                constant.default = None;
            }
            syn::GenericParam::Lifetime(_) => {}
        }
    }
    if !statics.is_empty() {
        generics.make_where_clause().predicates.extend(statics);
    }
    Ok(generics)
}

/// Whether the struct has type or const parameters: one ALTREP class per
/// instantiation.
pub(crate) fn is_generic(generics: &syn::Generics) -> bool {
    !generics.params.is_empty()
}

/// Whether the struct has type parameters, so the element's bounds must be
/// stated.
pub(crate) fn has_type_params(generics: &syn::Generics) -> bool {
    generics.type_params().next().is_some()
}

/// Add `ty: bound` to the where clause.
pub(crate) fn add_bound(generics: &mut syn::Generics, ty: &syn::Type, bound: TokenStream) {
    generics
        .make_where_clause()
        .predicates
        .push(syn::parse_quote!(#ty: #bound));
}

/// The `{params} Type {predicates}` invocation the api's generic macro arms
/// take: `{const N: usize} Dice<N> {}`.
pub(crate) fn brace_target(name: &syn::Ident, generics: &syn::Generics) -> TokenStream {
    let (_, ty_generics, _) = generics.split_for_impl();
    let params = generics.params.to_token_stream();
    let predicates = generics
        .where_clause
        .as_ref()
        .map(|clause| clause.predicates.to_token_stream());
    quote! { {#params} #name #ty_generics {#predicates} }
}
// endregion

#[cfg(test)]
mod tests;
