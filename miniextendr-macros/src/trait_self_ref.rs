//! Shared pre-conversion alias checks and Self-reference lowering for trait shims.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

/// Check native arguments before conversion can create references. Arity
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
        metadata.push(
            if crate::type_inspect::is_rooted_view(ty)
                || crate::type_inspect::is_rooted_reference(ty)
            {
                let ty = crate::type_inspect::erase_lifetimes(ty);
                quote! { <#ty as ::miniextendr_api::TryFromRoot<'_>>::NATIVE_BORROW }
            } else {
                quote! { <#ty as ::miniextendr_api::TryFromSexp>::NATIVE_BORROW }
            },
        );
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

/// Convert one shim argument with the C wrapper's structured error arm.
/// Arity and native alias checks must run first. Self handles are borrowed
/// later, after their address checks; rooted views retain a local scope until
/// the method's return has been converted.
pub(crate) fn argument(
    name: &syn::Ident,
    diagnostic_name: &str,
    ty: &syn::Type,
    impl_ty: &impl quote::ToTokens,
    index: usize,
    argv: &syn::Ident,
) -> TokenStream {
    let input = quote! { unsafe { *#argv.add(#index) } };
    let r_name = crate::r_wrapper_builder::normalize_r_arg_string(diagnostic_name);
    let (setup, binding, read_ty, conversion, rooted) = if let Some(mutable) =
        crate::type_inspect::self_ref_mutability(ty)
    {
        let handle = handle_ident(name);
        let mutability = mutable.then(|| quote!(mut));
        let read_ty: syn::Type = syn::parse_quote!(::miniextendr_api::ExternalPtr<#impl_ty>);
        let conversion = quote! {
            <#read_ty as ::miniextendr_api::TryFromSexp>::try_from_sexp(#input)
        };
        (
            TokenStream::new(),
            quote!(#mutability #handle),
            read_ty,
            conversion,
            false,
        )
    } else if crate::type_inspect::is_rooted_view(ty)
        || crate::type_inspect::is_rooted_reference(ty)
    {
        if let Some(stmt) = crate::type_inspect::static_rooted_view_binding(name, ty) {
            return stmt;
        }
        let slot = format_ident!(
            "__mx_root_{}",
            crate::naming::unraw(name),
            span = proc_macro2::Span::mixed_site()
        );
        let read_ty = crate::type_inspect::erase_lifetimes(ty);
        let conversion = quote! {
            <#read_ty as ::miniextendr_api::TryFromRoot<'_>>::try_from_root(unsafe {
                #slot.rooted(*#argv.add(#index))
            })
        };
        (
            quote! { let #slot = unsafe { ::miniextendr_api::gc_protect::ProtectScope::new() }; },
            quote!(#name),
            read_ty,
            conversion,
            true,
        )
    } else {
        let conversion = quote! {
            <#ty as ::miniextendr_api::TryFromSexp>::try_from_sexp(#input)
        };
        (
            TokenStream::new(),
            quote!(#name),
            ty.clone(),
            conversion,
            false,
        )
    };
    let err_arm = crate::rust_conversion_builder::shim_conversion_err_arm(
        &r_name,
        ty,
        (!rooted).then_some(&read_ty),
    );
    quote! {
        #setup
        let #binding: #read_ty = match #conversion {
            ::core::result::Result::Ok(value) => value,
            #err_arm
        };
    }
}

/// A shim's receiver address for [`alias_checks`]: the value inside the
/// receiver's `RefCell`, comparable with `ExternalPtr::as_ptr` of a `&Self`
/// argument.
pub(crate) fn cell_address(cell: &syn::Ident) -> TokenStream {
    quote! { #cell.as_ptr().cast_const().cast::<::core::ffi::c_void>() }
}

/// Borrow a shim's receiver from its `RefCell` for the rest of the shim:
/// shared for `&self`, exclusive for `&mut self`. A call still running on the
/// same object makes this raise the `miniextendr_borrow_conflict` error.
pub(crate) fn receiver_borrow(
    cell: &syn::Ident,
    guard: &syn::Ident,
    self_ref: &syn::Ident,
    mutable: bool,
) -> TokenStream {
    if mutable {
        quote! {
            let mut #guard = ::miniextendr_api::externalptr::borrow_value_mut(#cell);
            let #self_ref = &mut *#guard;
        }
    } else {
        quote! {
            let #guard = ::miniextendr_api::externalptr::borrow_value(#cell);
            let #self_ref = &*#guard;
        }
    }
}

pub(crate) fn handle_ident(name: &syn::Ident) -> syn::Ident {
    format_ident!(
        "__extptr_{}",
        crate::naming::unraw(name),
        span = proc_macro2::Span::mixed_site()
    )
}

/// Borrow a `&Self` argument's value for the rest of the shim, exclusively
/// for `&mut Self`, as a C wrapper borrows a `&T` / `&mut T` argument
/// (#2106): a call still running on the object makes the borrow raise the
/// `miniextendr_borrow_conflict` error. The handle [`argument`] made roots the
/// pointer of a class-wrapped argument. Runs after [`alias_checks`].
pub(crate) fn borrow(name: &syn::Ident, mutable: bool) -> TokenStream {
    let handle = handle_ident(name);
    let guard = format_ident!(
        "__mx_guard_{}",
        crate::naming::unraw(name),
        span = proc_macro2::Span::mixed_site()
    );
    if mutable {
        quote! {
            let mut #guard = #handle.borrow_mut();
            let #name = &mut *#guard;
        }
    } else {
        quote! {
            let #guard = #handle.borrow();
            let #name = &*#guard;
        }
    }
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

    #[test]
    fn self_arguments_hold_their_borrow_for_the_shim() {
        let output = borrow(&format_ident!("r#type"), true).to_string();
        assert!(output.contains("__extptr_type . borrow_mut ()"), "{output}");
        assert!(
            output.contains("let r#type = & mut * __mx_guard_type"),
            "{output}"
        );
        let output = borrow(&format_ident!("other"), false).to_string();
        assert!(output.contains("__extptr_other . borrow ()"), "{output}");
        assert!(
            output.contains("let other = & * __mx_guard_other"),
            "{output}"
        );
    }
}
