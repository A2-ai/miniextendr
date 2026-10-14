//! The borrows a C wrapper holds for its `ExternalPtr<T>` arguments (#2092).
//!
//! An `ExternalPtr<T>` or `Option<ExternalPtr<T>>` parameter borrows the
//! value's `RefCell` the way a method receiver does (#1965): exclusively for a
//! `mut` binding, shared otherwise. The borrow is taken after argument
//! conversion, so an R error that longjmps out of a conversion can't leave
//! the object borrowed, and held through return conversion. Two arguments
//! that hold one object, or a callback into R that reaches the object while
//! the function runs, then raise the `miniextendr_borrow_conflict` error
//! instead of making a second reference.
//!
//! `into_inner`, `into_raw`, `into_non_null` and `leak` move the value out of
//! its pointer, so they refuse a borrowed value. A body that names one of
//! them takes no borrow for its handles ([`consumed_handles`]).

use proc_macro2::{TokenStream, TokenTree};
use quote::{format_ident, quote};

/// The `ExternalPtr` functions that move the value out of its pointer.
const CONSUMING: [&str; 4] = ["into_inner", "into_raw", "into_non_null", "leak"];

/// `Some(false)` for an `ExternalPtr<T>` parameter type, `Some(true)` for
/// `Option<ExternalPtr<T>>`, `None` for any other. Matched on the last path
/// segment: a type alias of either is not seen, and gets no borrow (#2104).
pub(crate) fn handle_kind(ty: &syn::Type) -> Option<bool> {
    if is_handle(ty) {
        return Some(false);
    }
    crate::type_inspect::option_inner_type(ty)
        .is_some_and(is_handle)
        .then_some(true)
}

fn is_handle(ty: &syn::Type) -> bool {
    let syn::Type::Path(path) = ty else {
        return false;
    };
    path.qself.is_none()
        && path.path.segments.last().is_some_and(|seg| {
            seg.ident == "ExternalPtr" && crate::type_inspect::first_type_argument(seg).is_some()
        })
}

/// The names of the handle parameters a body may move the value out of:
/// every one when `body` names a consuming function anywhere (in a macro's
/// arguments, or as a function value, too), none otherwise. The match is on
/// the name alone, so `Box::leak` counts: a missed borrow, never a false
/// conflict. The scan can't follow a handle into other code: a function that
/// consumes it there raises the `miniextendr_borrow_conflict` error.
pub(crate) fn consumed_handles<'a>(
    inputs: impl IntoIterator<Item = &'a syn::FnArg>,
    body: TokenStream,
) -> Vec<String> {
    if !names_consuming_fn(body) {
        return Vec::new();
    }
    inputs
        .into_iter()
        .filter_map(|arg| match arg {
            syn::FnArg::Typed(pt) if handle_kind(&pt.ty).is_some() => match pt.pat.as_ref() {
                syn::Pat::Ident(pat) => Some(crate::naming::ident_name(&pat.ident)),
                _ => None,
            },
            _ => None,
        })
        .collect()
}

fn names_consuming_fn(tokens: TokenStream) -> bool {
    tokens.into_iter().any(|token| match token {
        TokenTree::Ident(ident) => CONSUMING.iter().any(|name| ident == name),
        TokenTree::Group(group) => names_consuming_fn(group.stream()),
        _ => false,
    })
}

/// A C wrapper's argument borrows, in two parts.
#[derive(Default)]
pub(crate) struct ArgBorrows {
    /// Binds each handle argument's SEXP, before conversion: the conversion
    /// binding may shadow the slot (a standalone function keeps its
    /// parameter names).
    pub capture: TokenStream,
    /// Takes the borrows, after every conversion. Each holds its value's
    /// `RefCell` until the wrapper's closure ends, after return conversion.
    pub borrow: TokenStream,
}

/// The borrows for the handle parameters of `inputs`, except those
/// `unborrowed` names. `sexp_idents` are the C wrapper's argument slots, in
/// parameter order.
pub(crate) fn arg_borrows(
    inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
    sexp_idents: &[syn::Ident],
    unborrowed: &[String],
) -> ArgBorrows {
    let mut borrows = ArgBorrows::default();
    let typed = inputs.iter().filter_map(|arg| match arg {
        syn::FnArg::Typed(pt) => Some(pt),
        syn::FnArg::Receiver(_) => None,
    });
    for (pt, sexp) in typed.zip(sexp_idents) {
        let syn::Pat::Ident(pat) = pt.pat.as_ref() else {
            continue;
        };
        let Some(optional) = handle_kind(&pt.ty) else {
            continue;
        };
        if unborrowed.contains(&crate::naming::ident_name(&pat.ident)) {
            continue;
        }
        let name = &pat.ident;
        let plain = crate::naming::unraw(name);
        let span = proc_macro2::Span::mixed_site();
        let slot = format_ident!("__mx_handle_sexp_{}", plain, span = span);
        let cell = format_ident!("__mx_handle_{}", plain, span = span);
        let guard = format_ident!("__mx_handle_guard_{}", plain, span = span);
        let handle = if optional {
            quote! { ::core::option::Option::as_ref(&#name) }
        } else {
            quote! { ::core::option::Option::Some(&#name) }
        };
        let borrow = if pat.mutability.is_some() {
            quote! { exclusive }
        } else {
            quote! { shared }
        };
        borrows.capture.extend(quote! {
            let #slot = #sexp;
        });
        // SAFETY (of the emitted `unsafe`): the wrapper runs on R's main
        // thread, and `#slot` is the argument the handle was converted from.
        borrows.borrow.extend(quote! {
            let #cell = unsafe {
                ::miniextendr_api::externalptr::ArgumentCell::new(#handle, #slot)
            };
            let #guard = ::miniextendr_api::externalptr::ArgumentCell::#borrow(&#cell);
        });
    }
    borrows
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inputs(sig: syn::Signature) -> syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]> {
        sig.inputs
    }

    #[test]
    fn handles_are_external_pointers_and_their_options() {
        let kinds = [
            (syn::parse_quote!(ExternalPtr<Counter>), Some(false)),
            (
                syn::parse_quote!(::miniextendr_api::externalptr::ExternalPtr<Counter>),
                Some(false),
            ),
            (syn::parse_quote!(Option<ExternalPtr<Counter>>), Some(true)),
            (syn::parse_quote!(Vec<ExternalPtr<Counter>>), None),
            (syn::parse_quote!(Vec<Option<ExternalPtr<Counter>>>), None),
            (syn::parse_quote!(CounterPtr), None),
            (syn::parse_quote!(ExternalPtr), None),
            (syn::parse_quote!(&ExternalPtr<Counter>), None),
        ];
        for (ty, kind) in kinds {
            let ty: syn::Type = ty;
            assert_eq!(handle_kind(&ty), kind, "{}", quote!(#ty));
        }
    }

    #[test]
    fn a_body_naming_a_consuming_function_consumes_every_handle() {
        let sig: syn::Signature = syn::parse_quote!(fn f(
            a: ExternalPtr<Counter>,
            mut b: Option<ExternalPtr<Counter>>,
            n: i32,
        ));
        for body in [
            quote! { ExternalPtr::into_inner(a).value },
            quote! { b.map(ExternalPtr::into_raw) },
            quote! { { let x = 1; ::std::println!("{}", ExternalPtr::leak(a).value); } },
        ] {
            assert_eq!(
                consumed_handles(&sig.inputs, body),
                ["a".to_owned(), "b".to_owned()]
            );
        }
        assert!(consumed_handles(&sig.inputs, quote! { a.value + n }).is_empty());
    }

    #[test]
    fn mut_bindings_borrow_exclusively_and_others_shared() {
        let sig: syn::Signature = syn::parse_quote!(fn f(
            mut a: ExternalPtr<Counter>,
            b: Option<ExternalPtr<Counter>>,
            n: i32,
            c: ExternalPtr<Counter>,
        ));
        let slots: Vec<syn::Ident> = (0..4_usize).map(|i| format_ident!("arg_{}", i)).collect();
        let borrows = arg_borrows(&inputs(sig.clone()), &slots, &["c".to_owned()]);
        let capture = borrows.capture.to_string();
        let borrow = borrows.borrow.to_string();
        assert!(capture.contains("= arg_0"), "{capture}");
        assert!(capture.contains("= arg_1"), "{capture}");
        assert!(
            !capture.contains("arg_2") && !capture.contains("arg_3"),
            "{capture}"
        );
        assert!(
            borrow.contains(":: core :: option :: Option :: Some (& a)"),
            "{borrow}"
        );
        assert!(
            borrow.contains(":: core :: option :: Option :: as_ref (& b)"),
            "{borrow}"
        );
        assert_eq!(
            borrow.matches("ArgumentCell :: exclusive").count(),
            1,
            "{borrow}"
        );
        assert_eq!(
            borrow.matches("ArgumentCell :: shared").count(),
            1,
            "{borrow}"
        );
        assert!(
            arg_borrows(&inputs(sig), &slots, &[])
                .borrow
                .to_string()
                .contains("(& c)")
        );
    }
}
