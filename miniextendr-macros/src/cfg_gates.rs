//! Resolve `cfg` / `cfg_attr` on the members of a `#[miniextendr]` item before
//! anything is generated from it.
//!
//! An attribute macro receives its item unconfigured: rustc has not evaluated
//! the gates on the item's methods, trait items, parameters, variants or fields.
//! Forwarding a member's gate to some generated items is not enough, because
//! item-wide analysis (constructor inference, S7 properties, name collisions,
//! factor levels, R formals) and the generated R text must see the same members
//! the compiled item has. So the macro re-enters itself on two gated copies of
//! the item, one per value of the first member gate it finds. rustc strips the
//! inactive copy before expanding it, so each pass resolves one gate and
//! configurations are never enumerated. The item stays whole: splitting an impl
//! into separately registered blocks would change constructor inference and the
//! ownership of S7 properties.
//!
//! Gates on the item itself need no re-entry: rustc evaluates them when they
//! precede `#[miniextendr]`, and the expansion forwards them otherwise.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Item, Meta, Token, punctuated::Punctuated};

/// Re-enter `#[miniextendr(attr)]` on both resolutions of the item's first
/// member gate, or return `None` when no member carries one.
pub(crate) fn select(attr: &TokenStream, item: &Item) -> syn::Result<Option<TokenStream>> {
    for member in 0..member_count(item) {
        let Some(attrs) = member_attrs(item, member) else {
            continue;
        };
        let Some(position) = attrs
            .iter()
            .position(|gate| gate.path().is_ident("cfg") || gate.path().is_ident("cfg_attr"))
        else {
            continue;
        };
        let gate = &attrs[position];
        let mut enabled = item.clone();
        let mut disabled = item.clone();
        let enabled_attrs = member_attrs_mut(&mut enabled, member);
        enabled_attrs.remove(position);
        let predicate = if gate.path().is_ident("cfg") {
            remove_member(&mut disabled, member);
            gate.parse_args::<Meta>()?
        } else {
            let mut args = gate
                .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?
                .into_iter();
            let predicate = args
                .next()
                .ok_or_else(|| syn::Error::new_spanned(gate, "expected a cfg_attr predicate"))?;
            enabled_attrs.splice(
                position..position,
                args.map(|meta| -> Attribute { syn::parse_quote!(#[#meta]) }),
            );
            member_attrs_mut(&mut disabled, member).remove(position);
            predicate
        };
        return Ok(Some(quote! {
            #[cfg(#predicate)]
            #[::miniextendr_api::miniextendr(#attr)]
            #enabled

            #[cfg(not(#predicate))]
            #[::miniextendr_api::miniextendr(#attr)]
            #disabled
        }));
    }
    Ok(None)
}

/// Number of gateable members: function parameters, impl and trait items, enum
/// variants, struct fields.
fn member_count(item: &Item) -> usize {
    match item {
        Item::Fn(item) => item.sig.inputs.len(),
        Item::Impl(item) => item.items.len(),
        Item::Trait(item) => item.items.len(),
        Item::Enum(item) => item.variants.len(),
        Item::Struct(item) => item.fields.len(),
        _ => 0,
    }
}

fn member_attrs(item: &Item, member: usize) -> Option<&Vec<Attribute>> {
    match item {
        Item::Fn(item) => Some(match &item.sig.inputs[member] {
            syn::FnArg::Receiver(receiver) => &receiver.attrs,
            syn::FnArg::Typed(typed) => &typed.attrs,
        }),
        Item::Impl(item) => match &item.items[member] {
            syn::ImplItem::Fn(member) => Some(&member.attrs),
            syn::ImplItem::Const(member) => Some(&member.attrs),
            syn::ImplItem::Type(member) => Some(&member.attrs),
            syn::ImplItem::Macro(member) => Some(&member.attrs),
            _ => None,
        },
        Item::Trait(item) => match &item.items[member] {
            syn::TraitItem::Fn(member) => Some(&member.attrs),
            syn::TraitItem::Const(member) => Some(&member.attrs),
            syn::TraitItem::Type(member) => Some(&member.attrs),
            syn::TraitItem::Macro(member) => Some(&member.attrs),
            _ => None,
        },
        Item::Enum(item) => Some(&item.variants[member].attrs),
        Item::Struct(item) => item.fields.iter().nth(member).map(|field| &field.attrs),
        _ => None,
    }
}

/// Only called for a member `member_attrs` returned.
fn member_attrs_mut(item: &mut Item, member: usize) -> &mut Vec<Attribute> {
    match item {
        Item::Fn(item) => match &mut item.sig.inputs[member] {
            syn::FnArg::Receiver(receiver) => &mut receiver.attrs,
            syn::FnArg::Typed(typed) => &mut typed.attrs,
        },
        Item::Impl(item) => match &mut item.items[member] {
            syn::ImplItem::Fn(member) => &mut member.attrs,
            syn::ImplItem::Const(member) => &mut member.attrs,
            syn::ImplItem::Type(member) => &mut member.attrs,
            syn::ImplItem::Macro(member) => &mut member.attrs,
            _ => unreachable!(),
        },
        Item::Trait(item) => match &mut item.items[member] {
            syn::TraitItem::Fn(member) => &mut member.attrs,
            syn::TraitItem::Const(member) => &mut member.attrs,
            syn::TraitItem::Type(member) => &mut member.attrs,
            syn::TraitItem::Macro(member) => &mut member.attrs,
            _ => unreachable!(),
        },
        Item::Enum(item) => &mut item.variants[member].attrs,
        Item::Struct(item) => &mut item.fields.iter_mut().nth(member).unwrap().attrs,
        _ => unreachable!(),
    }
}

fn remove_member(item: &mut Item, member: usize) {
    match item {
        Item::Fn(item) => remove_at(&mut item.sig.inputs, member),
        Item::Impl(item) => {
            item.items.remove(member);
        }
        Item::Trait(item) => {
            item.items.remove(member);
        }
        Item::Enum(item) => remove_at(&mut item.variants, member),
        Item::Struct(item) => match &mut item.fields {
            syn::Fields::Named(fields) => remove_at(&mut fields.named, member),
            syn::Fields::Unnamed(fields) => remove_at(&mut fields.unnamed, member),
            syn::Fields::Unit => unreachable!(),
        },
        _ => unreachable!(),
    }
}

fn remove_at<T, P: Default>(list: &mut Punctuated<T, P>, index: usize) {
    *list = std::mem::take(list)
        .into_iter()
        .enumerate()
        .filter_map(|(position, member)| (position != index).then_some(member))
        .collect();
}
