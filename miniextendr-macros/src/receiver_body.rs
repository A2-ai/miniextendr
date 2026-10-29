//! Rewrite an explicit handle receiver without changing independent Rust scopes.

use proc_macro2::{Group, Ident, TokenStream, TokenTree};
use syn::visit_mut::{self, VisitMut};

pub(crate) fn rewrite(body: &mut syn::Block, receiver: &Ident) {
    ReceiverBody { receiver }.visit_block_mut(body);
}

struct ReceiverBody<'a> {
    receiver: &'a Ident,
}

impl VisitMut for ReceiverBody<'_> {
    fn visit_expr_path_mut(&mut self, expr: &mut syn::ExprPath) {
        if expr.qself.is_none() && expr.path.is_ident("self") {
            expr.path = self.receiver.clone().into();
        } else {
            visit_mut::visit_expr_path_mut(self, expr);
        }
    }

    fn visit_item_mut(&mut self, item: &mut syn::Item) {
        // Nested functions, impls and modules have independent receivers and
        // cannot capture this method's self. Local macros can capture it.
        if let syn::Item::Macro(item) = item {
            self.visit_macro_mut(&mut item.mac);
        }
    }

    fn visit_macro_mut(&mut self, mac: &mut syn::Macro) {
        // Macro arguments are opaque syntax. Preserve the macro's path and
        // module-qualified self:: paths inside its tokens.
        mac.tokens = rewrite_macro_tokens(mac.tokens.clone(), self.receiver);
    }
}

fn rewrite_macro_tokens(tokens: TokenStream, receiver: &Ident) -> TokenStream {
    let tokens: Vec<_> = tokens.into_iter().collect();
    tokens.iter().enumerate().map(|(i, token)| match token {
        TokenTree::Ident(ident) if ident == "self" => {
            let module_path = matches!(tokens.get(i + 1), Some(TokenTree::Punct(p)) if p.as_char() == ':')
                && matches!(tokens.get(i + 2), Some(TokenTree::Punct(p)) if p.as_char() == ':');
            if module_path {
                token.clone()
            } else {
                TokenTree::Ident(receiver.clone())
            }
        }
        TokenTree::Group(group) => {
            let mut rewritten = Group::new(group.delimiter(), rewrite_macro_tokens(group.stream(), receiver));
            rewritten.set_span(group.span());
            TokenTree::Group(rewritten)
        }
        _ => token.clone(),
    }).collect()
}

#[cfg(test)]
mod tests;
