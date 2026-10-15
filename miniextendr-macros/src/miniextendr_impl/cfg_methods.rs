//! Resolve method gates through rustc before interpreting the complete class.
//!
//! An attribute macro receives methods whose inner `cfg` attributes have not
//! been evaluated. Forwarding their gates only to C wrappers is insufficient:
//! class-wide analysis and the generated R text must see the same methods.
//! Re-enter the macro on two gated copies of the impl, selecting one condition
//! at a time. Only the active copy expands again, so this does not enumerate
//! configurations. Keep the impl intact: splitting it into separately registered
//! blocks changes constructor inference and the ownership of S7 properties.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{ImplItem, ItemImpl, Meta, Token, punctuated::Punctuated};

pub(super) fn select(attr: TokenStream, item: &ItemImpl) -> syn::Result<Option<TokenStream>> {
    for (item_index, member) in item.items.iter().enumerate() {
        let ImplItem::Fn(method) = member else {
            continue;
        };
        for (attr_index, gate) in method.attrs.iter().enumerate() {
            if !gate.path().is_ident("cfg") && !gate.path().is_ident("cfg_attr") {
                continue;
            }
            let mut enabled = item.clone();
            let mut disabled = item.clone();
            let ImplItem::Fn(enabled_method) = &mut enabled.items[item_index] else {
                unreachable!()
            };
            enabled_method.attrs.remove(attr_index);
            let predicate = if gate.path().is_ident("cfg") {
                disabled.items.remove(item_index);
                gate.parse_args::<Meta>()?
            } else {
                let mut args = gate
                    .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?
                    .into_iter();
                let predicate = args.next().ok_or_else(|| {
                    syn::Error::new_spanned(gate, "expected a cfg_attr predicate")
                })?;
                enabled_method.attrs.splice(
                    attr_index..attr_index,
                    args.map(|meta| syn::parse_quote!(#[#meta])),
                );
                let ImplItem::Fn(disabled_method) = &mut disabled.items[item_index] else {
                    unreachable!()
                };
                disabled_method.attrs.remove(attr_index);
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
    }
    Ok(None)
}
