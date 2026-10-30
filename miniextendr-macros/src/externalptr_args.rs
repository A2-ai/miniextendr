//! Borrowed external arguments (#2106): `&T` / `&mut T` for a `TypedExternal`
//! type, the views `ExternalPtrRef<'_, T>` / `ExternalPtrMut<'_, T>`, and an
//! `Option` of either.
//!
//! The wrapper converts the argument to its `ExternalPtr<T>` handle with the
//! other arguments (`from_r::__mx_external_arg`, which unwraps and roots a
//! class handle), then borrows the value's `RefCell` the way a method receiver
//! is borrowed: shared for `&T` and `ExternalPtrRef`, exclusive for `&mut T`
//! and `ExternalPtrMut`. The borrow is taken after every conversion and held
//! through return conversion, and the body gets the plain reference (or the
//! view). One object passed twice with a `&mut T`, or a callback into R that
//! reaches the object while the function runs, then raises the
//! `miniextendr_borrow_conflict` error instead of making a second reference.
//!
//! An `ExternalPtr<T>` parameter is a handle the body keeps or consumes: it
//! takes no borrow, and reads through it are `borrow()` / `borrow_mut()`.

use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;

/// How a borrowed external argument holds its value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Mode {
    /// `&T`.
    Shared,
    /// `&mut T`.
    Exclusive,
    /// `ExternalPtrRef<'_, T>`.
    SharedView,
    /// `ExternalPtrMut<'_, T>`.
    ExclusiveView,
}

impl Mode {
    fn is_exclusive(self) -> bool {
        matches!(self, Mode::Exclusive | Mode::ExclusiveView)
    }
}

/// A borrowed external argument's type, taken apart.
#[derive(Debug)]
pub(crate) struct ExternalArg {
    /// `T`, as written (`Self` included).
    pub value: syn::Type,
    /// Under `Option`: `NULL` is `None`.
    pub optional: bool,
    pub mode: Mode,
}

/// Names that are never an external value behind `&` / `&mut`: R's native
/// borrows, `&str` / `&Path`, the dots, the RNG guard an `rng` wrapper passes
/// (#2002), and containers with conversions of their own.
const NOT_EXTERNAL: &[&str] = &[
    "str",
    "Path",
    "Dots",
    "LazyDots",
    "RngGuard",
    "SEXP",
    "i32",
    "f64",
    "u8",
    "RLogical",
    "Rcomplex",
    "Option",
    "Vec",
    "Box",
    "Rc",
    "Arc",
    "Cow",
    "Result",
    "ExternalPtr",
    "ExternalPtrRef",
    "ExternalPtrMut",
    "Missing",
];

/// The reference selected by an explicit conversion, including Option.
pub(crate) fn conversion_reference(ty: &syn::Type) -> Option<&syn::TypeReference> {
    let ty = crate::type_inspect::option_inner_type(ty).unwrap_or(ty);
    match ty {
        syn::Type::Reference(reference) => Some(reference),
        _ => None,
    }
}

/// The borrowed external argument `ty` describes, if any: `&T` / `&mut T`
/// where `T` names a type with no R-side conversion of its own (no type
/// check, not in [`NOT_EXTERNAL`]), or an `ExternalPtrRef<'_, T>` /
/// `ExternalPtrMut<'_, T>` view, either one optionally under `Option`.
/// Matched on the written type: a `&T` whose `T` isn't `TypedExternal` is a
/// compile error at the type (`ExternalPtr<T>` requires it).
pub(crate) fn external_arg(ty: &syn::Type) -> Option<ExternalArg> {
    let (inner, optional) = match crate::type_inspect::option_inner_type(ty) {
        Some(inner) => (inner, true),
        None => (ty, false),
    };
    let (value, mode) = match inner {
        syn::Type::Reference(reference) if is_external_value(&reference.elem) => {
            let mode = if reference.mutability.is_some() {
                Mode::Exclusive
            } else {
                Mode::Shared
            };
            (reference.elem.as_ref().clone(), mode)
        }
        syn::Type::Path(path) if path.qself.is_none() => {
            let seg = path.path.segments.last()?;
            let mode = if seg.ident == "ExternalPtrRef" {
                Mode::SharedView
            } else if seg.ident == "ExternalPtrMut" {
                Mode::ExclusiveView
            } else {
                return None;
            };
            (crate::type_inspect::first_type_argument(seg)?.clone(), mode)
        }
        _ => return None,
    };
    Some(ExternalArg {
        value,
        optional,
        mode,
    })
}

fn is_external_value(ty: &syn::Type) -> bool {
    let syn::Type::Path(path) = ty else {
        return false;
    };
    path.qself.is_none()
        && path.path.segments.last().is_some_and(|seg| {
            !NOT_EXTERNAL.contains(&seg.ident.to_string().as_str())
                && !crate::r_preconditions::has_type_check(ty)
        })
}

/// The binding of the handle a borrowed argument `param` converts to.
/// Mixed-site: user parameters may use any spelling.
pub(crate) fn handle_ident(param: &syn::Ident) -> syn::Ident {
    format_ident!(
        "__mx_ptr_{}",
        crate::naming::unraw(param),
        span = proc_macro2::Span::mixed_site()
    )
}

/// `ty` with `Self` replaced by `self_ty` (an impl method's wrapper is a free
/// function, where `Self` names nothing).
pub(crate) fn replace_self(ty: &syn::Type, self_ty: Option<&syn::Type>) -> syn::Type {
    match (ty, self_ty) {
        (syn::Type::Path(path), Some(self_ty)) if path.path.is_ident("Self") => self_ty.clone(),
        _ => ty.clone(),
    }
}

/// The statement converting the borrowed argument `ident` (of type `arg`) to
/// its handle, given its R-facing failure arm (`err_arm`, an `Err(e) =>`
/// match arm). `self_ty` replaces `Self`.
pub(crate) fn handle_conversion(
    ident: &syn::Ident,
    sexp_ident: &syn::Ident,
    ty: &syn::Type,
    arg: &ExternalArg,
    self_ty: Option<&syn::Type>,
    err_arm: TokenStream,
) -> TokenStream {
    let handle = handle_ident(ident);
    if let Some(lifetime) = crate::type_inspect::static_lifetime(ty) {
        return quote_spanned! {lifetime.span()=>
            ::core::compile_error!(
                "this parameter borrows its object for the call and cannot be `'static`; \
                 elide the lifetime, or take `ExternalPtr<T>` to keep a handle"
            );
            let #ident: #ty = ::core::option::Option::<#ty>::None.unwrap();
        };
    }
    let value = replace_self(&arg.value, self_ty);
    let span = arg.value.span();
    let convert = if arg.optional {
        quote_spanned! {span=> __mx_optional_external_arg }
    } else {
        quote_spanned! {span=> __mx_external_arg }
    };
    let mutability = arg.mode.is_exclusive().then(|| quote! { mut });
    // Spanned on `T`, like the borrows: every use of `T: TypedExternal` is a
    // free function's bound there, so a `T` without it is one error.
    quote_spanned! {span=>
        let #mutability #handle = match ::miniextendr_api::from_r::#convert::<#value>(#sexp_ident) {
            ::core::result::Result::Ok(v) => v,
            #err_arm
        };
    }
}

/// The borrows of the borrowed external parameters of `inputs`, except those
/// `unborrowed` names (a parameter marker rebinds them to another type).
/// Each binds the parameter to its reference or view, holding the value's
/// `RefCell` until the wrapper's closure ends, after return conversion.
pub(crate) fn arg_borrows(
    inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
    unborrowed: &[String],
    self_ty: Option<&syn::Type>,
) -> TokenStream {
    let mut borrows = TokenStream::new();
    for arg in inputs {
        let syn::FnArg::Typed(pt) = arg else {
            continue;
        };
        let syn::Pat::Ident(pat) = pt.pat.as_ref() else {
            continue;
        };
        let Some(external) = external_arg(&pt.ty) else {
            continue;
        };
        if unborrowed.contains(&crate::naming::ident_name(&pat.ident)) {
            continue;
        }
        let name = &pat.ident;
        let handle = handle_ident(name);
        let guard = format_ident!(
            "__mx_guard_{}",
            crate::naming::unraw(name),
            span = proc_macro2::Span::mixed_site()
        );
        let value = replace_self(&external.value, self_ty);
        let span = external.value.span();
        let api = quote_spanned! {span=> ::miniextendr_api::externalptr };
        borrows.extend(match (external.mode, external.optional) {
            (Mode::Shared, false) => quote_spanned! {span=>
                let #guard = #api::__mx_borrow::<#value>(&#handle);
                let #name = #api::__mx_shared::<#value>(&#guard);
            },
            (Mode::Exclusive, false) => quote_spanned! {span=>
                let mut #guard = #api::__mx_borrow_mut::<#value>(&mut #handle);
                let #name = #api::__mx_exclusive::<#value>(&mut #guard);
            },
            (Mode::SharedView, false) => quote_spanned! {span=>
                let #name = #api::__mx_borrow::<#value>(&#handle);
            },
            (Mode::ExclusiveView, false) => quote_spanned! {span=>
                let #name = #api::__mx_borrow_mut::<#value>(&mut #handle);
            },
            (Mode::Shared, true) => quote_spanned! {span=>
                let #guard = ::core::option::Option::map(
                    ::core::option::Option::as_ref(&#handle), #api::__mx_borrow::<#value>,
                );
                let #name = ::core::option::Option::map(
                    ::core::option::Option::as_ref(&#guard), #api::__mx_shared::<#value>,
                );
            },
            (Mode::Exclusive, true) => quote_spanned! {span=>
                let mut #guard = ::core::option::Option::map(
                    ::core::option::Option::as_mut(&mut #handle), #api::__mx_borrow_mut::<#value>,
                );
                let #name = ::core::option::Option::map(
                    ::core::option::Option::as_mut(&mut #guard), #api::__mx_exclusive::<#value>,
                );
            },
            (Mode::SharedView, true) => quote_spanned! {span=>
                let #name = ::core::option::Option::map(
                    ::core::option::Option::as_ref(&#handle), #api::__mx_borrow::<#value>,
                );
            },
            (Mode::ExclusiveView, true) => quote_spanned! {span=>
                let #name = ::core::option::Option::map(
                    ::core::option::Option::as_mut(&mut #handle), #api::__mx_borrow_mut::<#value>,
                );
            },
        });
    }
    borrows
}

#[cfg(test)]
mod tests;
