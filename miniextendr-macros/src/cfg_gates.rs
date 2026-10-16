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
    for member in members(item) {
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

/// A direct member, or a parameter inside an impl/trait method. Visiting the
/// method first lets a disabled method disappear without inspecting its inputs.
#[derive(Clone, Copy)]
struct Member {
    index: usize,
    parameter: Option<usize>,
}

fn members(item: &Item) -> Vec<Member> {
    let count = match item {
        Item::Fn(item) => item.sig.inputs.len(),
        Item::Impl(item) => item.items.len(),
        Item::Trait(item) => item.items.len(),
        Item::Enum(item) => item.variants.len(),
        Item::Struct(item) => item.fields.len(),
        _ => 0,
    };
    let mut members = Vec::new();
    for index in 0..count {
        members.push(Member {
            index,
            parameter: None,
        });
        if let Some(signature) = method_signature(item, index) {
            members.extend((0..signature.inputs.len()).map(|parameter| Member {
                index,
                parameter: Some(parameter),
            }));
        }
    }
    members
}

fn method_signature(item: &Item, index: usize) -> Option<&syn::Signature> {
    match item {
        Item::Impl(item) => match &item.items[index] {
            syn::ImplItem::Fn(method) => Some(&method.sig),
            _ => None,
        },
        Item::Trait(item) => match &item.items[index] {
            syn::TraitItem::Fn(method) => Some(&method.sig),
            _ => None,
        },
        _ => None,
    }
}

fn method_signature_mut(item: &mut Item, index: usize) -> &mut syn::Signature {
    match item {
        Item::Impl(item) => match &mut item.items[index] {
            syn::ImplItem::Fn(method) => &mut method.sig,
            _ => unreachable!(),
        },
        Item::Trait(item) => match &mut item.items[index] {
            syn::TraitItem::Fn(method) => &mut method.sig,
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}

fn parameter_attrs(arg: &syn::FnArg) -> &Vec<Attribute> {
    match arg {
        syn::FnArg::Receiver(receiver) => &receiver.attrs,
        syn::FnArg::Typed(typed) => &typed.attrs,
    }
}

fn parameter_attrs_mut(arg: &mut syn::FnArg) -> &mut Vec<Attribute> {
    match arg {
        syn::FnArg::Receiver(receiver) => &mut receiver.attrs,
        syn::FnArg::Typed(typed) => &mut typed.attrs,
    }
}

fn member_attrs(item: &Item, member: Member) -> Option<&Vec<Attribute>> {
    if let Some(parameter) = member.parameter {
        return Some(parameter_attrs(
            &method_signature(item, member.index)?.inputs[parameter],
        ));
    }
    let member = member.index;
    match item {
        Item::Fn(item) => Some(parameter_attrs(&item.sig.inputs[member])),
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
fn member_attrs_mut(item: &mut Item, member: Member) -> &mut Vec<Attribute> {
    if let Some(parameter) = member.parameter {
        return parameter_attrs_mut(
            &mut method_signature_mut(item, member.index).inputs[parameter],
        );
    }
    let member = member.index;
    match item {
        Item::Fn(item) => parameter_attrs_mut(&mut item.sig.inputs[member]),
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

fn remove_member(item: &mut Item, member: Member) {
    if let Some(parameter) = member.parameter {
        remove_at(
            &mut method_signature_mut(item, member.index).inputs,
            parameter,
        );
        return;
    }
    let member = member.index;
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
