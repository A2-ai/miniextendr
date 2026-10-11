//! Shared pre-conversion alias checks and Self-reference lowering for trait shims.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

/// Check native arguments before `extract_arg` can create references. Arity
/// must already be checked, and the caller must be inside the shim's R unwind
/// boundary. `&Self` uses ExternalPtr handles and its separate address check.
pub(crate) fn native_borrow_checks<'a>(
    args: impl IntoIterator<Item = (usize, String, &'a syn::Type)>,
    argv: &syn::Ident,
) -> TokenStream {
    let mut values = Vec::new();
    let mut metadata = Vec::new();
    for (index, name, ty) in args {
        if crate::type_inspect::self_ref_mutability(ty).is_some() {
            continue;
        }
        values.push(quote! { (*#argv.add(#index), #name) });
        metadata.push(quote! { <#ty as ::miniextendr_api::TryFromSexp>::NATIVE_BORROW });
    }
    if values.is_empty() {
        return TokenStream::new();
    }
    let borrows = syn::Ident::new(
        "__miniextendr_native_borrows",
        proc_macro2::Span::mixed_site(),
    );
    quote! {{
        let #borrows = [#(#metadata),*];
        if ::miniextendr_api::from_r::borrow::needs_check(&#borrows) {
            // SAFETY: arity was checked before reading argv; its arguments
            // remain rooted on R's thread inside the shim's unwind boundary.
            unsafe {
                ::miniextendr_api::from_r::borrow::assert_no_aliases(
                    [#(#values),*], #borrows,
                );
            }
        }
    }}
}

pub(crate) fn handle_ident(name: &syn::Ident) -> syn::Ident {
    format_ident!(
        "__extptr_{}",
        crate::naming::unraw(name),
        span = proc_macro2::Span::mixed_site()
    )
}

/// Extract handles first; references are created only after alias checks.
pub(crate) fn extract(
    name: &syn::Ident,
    ty: &impl quote::ToTokens,
    index: usize,
    mutable: bool,
    argc: &syn::Ident,
    argv: &syn::Ident,
) -> TokenStream {
    let handle = handle_ident(name);
    let mutability = mutable.then(|| quote!(mut));
    let name_str = crate::naming::ident_name(name);
    quote! {
        let #mutability #handle: ::miniextendr_api::ExternalPtr<#ty> = unsafe {
            ::miniextendr_api::trait_abi::extract_arg(#argc, #argv, #index, #name_str)
        };
    }
}

pub(crate) fn borrow(name: &syn::Ident, mutable: bool) -> TokenStream {
    let handle = handle_ident(name);
    let mutability = mutable.then(|| quote!(mut));
    quote! { let #name = &#mutability *#handle; }
}

/// Reject overlapping Self arguments before any of their references exist.
/// Shared/shared pairs are allowed. Reentrant access remains a separate concern.
pub(crate) fn alias_checks(
    ty: &impl quote::ToTokens,
    handles: &[(syn::Ident, bool, String)],
    receiver: Option<(TokenStream, bool)>,
) -> TokenStream {
    let mut pointers: Vec<_> = handles
        .iter()
        .map(|(handle, mutable, name)| {
            (
                quote!(#handle.as_ptr().cast::<::core::ffi::c_void>()),
                *mutable,
                name.clone(),
            )
        })
        .collect();
    if let Some((pointer, mutable)) = receiver {
        pointers.insert(0, (pointer, mutable, "self".to_owned()));
    }
    let mut checks = Vec::new();
    for (index, (left, left_mut, left_name)) in pointers.iter().enumerate() {
        for (right, right_mut, right_name) in &pointers[index + 1..] {
            if *left_mut || *right_mut {
                let message = format!(
                    "trait arguments '{left_name}' and '{right_name}' alias the same object while one is mutable"
                );
                checks.push(quote! {
                    ::core::assert!(!::core::ptr::eq(#left, #right), #message);
                });
            }
        }
    }
    if checks.is_empty() {
        TokenStream::new()
    } else {
        // Distinct zero-sized allocations can have identical data addresses.
        quote! { if ::core::mem::size_of::<#ty>() != 0 { #(#checks)* } }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_preflight_preserves_argument_positions_around_self_references() {
        let types: [syn::Type; 3] = [
            syn::parse_quote!(&Self),
            syn::parse_quote!((&mut i32, &i32)),
            syn::parse_quote!(&mut Self),
        ];
        let argv = format_ident!("argv");
        let output = native_borrow_checks(
            types
                .iter()
                .enumerate()
                .map(|(i, ty)| (i, format!("arg{i}"), ty)),
            &argv,
        );
        let expected =
            quote! { <(&mut i32, &i32) as ::miniextendr_api::TryFromSexp>::NATIVE_BORROW };
        let output = output.to_string();
        assert!(output.contains(&expected.to_string()));
        assert!(output.contains(&quote! { (*argv.add(1usize), "arg1") }.to_string()));
        assert!(!output.contains("arg0"));
        assert!(!output.contains("arg2"));
        assert!(native_borrow_checks([(0, "other".into(), &types[0])], &argv).is_empty());
    }

    #[test]
    fn shared_pairs_need_no_guard_but_every_mutable_pair_does() {
        let handles = vec![
            (format_ident!("left"), false, "left".into()),
            (format_ident!("r#type"), false, "type".into()),
        ];
        assert!(
            alias_checks(
                &quote!(Counter),
                &handles,
                Some((quote!(data.cast_const()), false))
            )
            .is_empty()
        );
        let output = alias_checks(
            &quote!(Counter),
            &handles,
            Some((quote!(data.cast_const()), true)),
        )
        .to_string();
        assert_eq!(output.matches(":: core :: assert !").count(), 2);
        assert!(output.contains("'self' and 'left'"));
        assert!(output.contains("'self' and 'type'"));
        let mut handles = handles;
        handles[1].1 = true;
        let output = alias_checks(
            &quote!(Counter),
            &handles,
            Some((quote!(data.cast_const()), true)),
        )
        .to_string();
        assert_eq!(output.matches(":: core :: assert !").count(), 3);
        assert!(output.contains("'left' and 'type'"));
    }
}
