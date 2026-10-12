//! Explicit owner-bearing trait receivers, rewritten for stable Rust.

use proc_macro2::TokenStream;
use quote::quote;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OwnerReceiver {
    Shared,
    Mutable,
}

impl OwnerReceiver {
    pub(crate) fn type_path(self) -> TokenStream {
        match self {
            Self::Shared => quote!(::miniextendr_api::ExternalPtrRef),
            Self::Mutable => quote!(::miniextendr_api::ExternalPtrMut),
        }
    }
}

pub(crate) fn owner_receiver(sig: &syn::Signature) -> Option<OwnerReceiver> {
    let receiver = sig.receiver()?;
    // Only an explicit `self: Type` receiver can name a handle.
    receiver.colon_token?;
    let syn::Type::Path(path) = receiver.ty.as_ref() else {
        return None;
    };
    if path.qself.is_some() {
        return None;
    }
    let segment = path.path.segments.last()?;
    let kind = match segment.ident.to_string().as_str() {
        "ExternalPtrRef" => OwnerReceiver::Shared,
        "ExternalPtrMut" => OwnerReceiver::Mutable,
        _ => return None,
    };
    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };
    let mut types = args.args.iter().filter_map(|arg| match arg {
        syn::GenericArgument::Type(ty) => Some(ty),
        _ => None,
    });
    match (types.next(), types.next()) {
        (Some(syn::Type::Path(ty)), None) if ty.qself.is_none() && ty.path.is_ident("Self") => {
            Some(kind)
        }
        _ => None,
    }
}

pub(crate) fn is_mut(sig: &syn::Signature) -> bool {
    match owner_receiver(sig) {
        Some(OwnerReceiver::Mutable) => true,
        Some(OwnerReceiver::Shared) => false,
        None => sig
            .receiver()
            .is_some_and(|receiver| match receiver.ty.as_ref() {
                syn::Type::Reference(reference) => reference.mutability.is_some(),
                _ => receiver.mutability.is_some(),
            }),
    }
}

/// Keep the source signature for wrapper metadata; rewrite only the emitted
/// Rust trait/impl. The associated function takes its borrowed handle by value.
pub(crate) fn rewrite(sig: &mut syn::Signature, body: Option<&mut syn::Block>) {
    if owner_receiver(sig).is_none() {
        return;
    }
    let receiver = sig.receiver().expect("owner receiver");
    let ty = receiver.ty.clone();
    let mutability = receiver.mutability;
    let receiver_ident = syn::Ident::new("__miniextendr_self", proc_macro2::Span::mixed_site());
    sig.inputs[0] = syn::parse_quote!(#receiver_ident: #ty);
    if let Some(body) = body {
        sig.inputs[0] = syn::parse_quote!(#mutability #receiver_ident: #ty);
        crate::receiver_body::rewrite(body, &receiver_ident);
    }
}
