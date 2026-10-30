//! Return-type rules shared by the `#[miniextendr]` generators.
//!
//! - [`returns_invisibly`]: whether a standalone fn's R wrapper returns
//!   invisibly, read from the return type's shape.
//! - [`output_is_result`]: the shallow `Result` check behind `unwrap_in_r`.
//! - Strict mode's checked conversions for lossy integer types, of a return
//!   value ([`strict_conversion_for_type`]) and of an argument
//!   ([`strict_input_conversion_for_type`]).
//!
//! The return value's conversion itself is
//! [`crate::c_wrapper_builder::detect_return_handling`]'s.

/// Whether a standalone fn's R wrapper returns invisibly, read from the
/// return type's shape (a visibility marker or attribute overrides it, see
/// [`crate::type_inspect::resolve_visibility`]).
///
/// Invisible: no return type, `()`, `Option<()>`, `Result<(), ()>`, and
/// `Result<(), E>` unless `unwrap_in_r` hands the `Result` to R. Everything
/// else is visible. Names are matched by the last path segment; aliases are
/// not resolved.
pub(crate) fn returns_invisibly(output: &syn::ReturnType, unwrap_in_r: bool) -> bool {
    let ty = match output {
        syn::ReturnType::Default => return true,
        syn::ReturnType::Type(_, ty) => ty.as_ref(),
    };
    if is_unit(ty) {
        return true;
    }
    let syn::Type::Path(path) = ty else {
        return false;
    };
    let Some(segment) = path.path.segments.last() else {
        return false;
    };
    let ok_is_unit = crate::first_type_argument(segment).is_some_and(is_unit);
    if segment.ident == "Option" {
        return ok_is_unit;
    }
    if segment.ident == "Result" {
        let err_is_unit = crate::second_type_argument(segment).is_some_and(is_unit);
        return ok_is_unit && (err_is_unit || !unwrap_in_r);
    }
    false
}

/// `()`.
fn is_unit(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Tuple(tuple) if tuple.elems.is_empty())
}

/// Returns `true` if the return type is `Result<T, E>` (shallow name check).
///
/// Does not resolve type aliases. Used to decide whether `unwrap_in_r` should
/// strip the `Result` wrapper before converting to SEXP.
pub(crate) fn output_is_result(output: &syn::ReturnType) -> bool {
    match output {
        syn::ReturnType::Type(_, ty) => matches!(
            ty.as_ref(),
            syn::Type::Path(p)
                if p.path
                    .segments
                    .last()
                    .map(|s| s.ident == "Result")
                    .unwrap_or(false)
        ),
        syn::ReturnType::Default => false,
    }
}

/// Scalar types whose conversion between R and Rust is potentially lossy.
///
/// R represents all integers as 32-bit and all reals as 64-bit doubles,
/// so types wider than `i32` (or unsigned) may overflow or lose precision.
/// In strict mode, these types use checked conversions that panic on overflow.
pub(crate) const LOSSY_SCALARS: &[&str] = &["i64", "u64", "isize", "usize"];

/// The lossy scalar `T` when `ty` is `Vec<T>` with `T` in [`LOSSY_SCALARS`].
fn lossy_vec_element(ty: &syn::Type) -> Option<String> {
    if last_segment_ident(ty)? != "Vec" {
        return None;
    }
    let elem = last_segment_ident(first_type_arg_from_type(ty)?)?.to_string();
    LOSSY_SCALARS.contains(&elem.as_str()).then_some(elem)
}

/// A contiguous return shape whose conversion can borrow its elements.
/// Vec has existing named helpers; these are the other owned/borrowed shapes.
fn lossy_slice_element(ty: &syn::Type) -> Option<&syn::Type> {
    let elem = match ty {
        syn::Type::Reference(reference) => return lossy_slice_element(&reference.elem),
        syn::Type::Slice(slice) => slice.elem.as_ref(),
        syn::Type::Array(array) => array.elem.as_ref(),
        syn::Type::Paren(paren) => return lossy_slice_element(&paren.elem),
        syn::Type::Group(group) => return lossy_slice_element(&group.elem),
        _ if last_segment_ident(ty).is_some_and(|name| name == "Box") => {
            return lossy_slice_element(first_type_arg_from_type(ty)?);
        }
        _ => return None,
    };
    let name = last_segment_ident(elem)?.to_string();
    LOSSY_SCALARS.contains(&name.as_str()).then_some(elem)
}

fn strict_slice_conversion(
    ty: &syn::Type,
    result: &syn::Ident,
) -> Option<proc_macro2::TokenStream> {
    let elem = lossy_slice_element(ty)?;
    let container = crate::type_inspect::type_display(ty);
    Some(quote::quote! {
        ::miniextendr_api::strict::checked_slice_into_sexp::<#elem>(
            #container, ::core::convert::AsRef::as_ref(&#result)
        )
    })
}

/// Try to generate a strict conversion expression for a lossy return type.
///
/// Returns `Some(TokenStream)` if the type is a lossy scalar, `Option<lossy>`,
/// `Vec<lossy>`, `Vec<Option<lossy>>`, or one of the list-of-vectors shapes
/// `Option<Vec<lossy>>`, `Vec<Vec<lossy>>`, `Vec<Option<Vec<lossy>>>`.
/// Slices, boxed slices and arrays use the same checks, including optional
/// containers and rows. Otherwise `None` (falls through to standard `IntoR::into_sexp`).
pub(crate) fn strict_conversion_for_type(
    ty: &syn::Type,
    result_ident: &syn::Ident,
) -> Option<proc_macro2::TokenStream> {
    if let Some(conversion) = strict_slice_conversion(ty, result_ident) {
        return Some(conversion);
    }
    let type_name = last_segment_ident(ty)?;
    let name = type_name.to_string();

    // Check for scalar lossy types: i64, u64, isize, usize
    if LOSSY_SCALARS.contains(&name.as_str()) {
        let helper = quote::format_ident!("checked_into_sexp_{}", name);
        return Some(quote::quote! {
            ::miniextendr_api::strict::#helper(#result_ident)
        });
    }

    // Check for Option<lossy> or Option<Vec<lossy>>
    if name == "Option"
        && let Some(inner) = first_type_arg_from_type(ty)
    {
        let value = syn::Ident::new("__mx_strict_slice", proc_macro2::Span::mixed_site());
        if let Some(conversion) = strict_slice_conversion(inner, &value) {
            return Some(quote::quote! {
                match #result_ident {
                    ::core::option::Option::Some(#value) => #conversion,
                    ::core::option::Option::None => ::miniextendr_api::SEXP::nil(),
                }
            });
        }
        // Option<Vec<lossy>>: `Some` is the checked vector, `None` is NULL
        // (what the lax `Option<Vec<T>>` impl gives `None`).
        if let Some(elem) = lossy_vec_element(inner) {
            let helper = quote::format_ident!("checked_vec_{}_into_sexp", elem);
            return Some(quote::quote! {
                match #result_ident {
                    ::core::option::Option::Some(__mx_strict_vec) => {
                        ::miniextendr_api::strict::#helper(__mx_strict_vec)
                    }
                    ::core::option::Option::None => ::miniextendr_api::SEXP::nil(),
                }
            });
        }
        let inner_name = last_segment_ident(inner)?.to_string();
        if LOSSY_SCALARS.contains(&inner_name.as_str()) {
            let helper = quote::format_ident!("checked_option_{}_into_sexp", inner_name);
            return Some(quote::quote! {
                ::miniextendr_api::strict::#helper(#result_ident)
            });
        }
    }

    // Check for Vec<lossy>, Vec<Option<lossy>>, Vec<Vec<lossy>>, Vec<Option<Vec<lossy>>>
    if name == "Vec"
        && let Some(inner) = first_type_arg_from_type(ty)
    {
        let optional = last_segment_ident(inner).is_some_and(|name| name == "Option");
        let row = if optional {
            first_type_arg_from_type(inner)?
        } else {
            inner
        };
        if let Some(elem) = lossy_slice_element(row) {
            let rows = if optional {
                quote::quote! { ::core::iter::IntoIterator::into_iter(#result_ident) }
            } else {
                quote::quote! {
                    ::core::iter::Iterator::map(
                        ::core::iter::IntoIterator::into_iter(#result_ident),
                        ::core::option::Option::Some,
                    )
                }
            };
            let container = crate::type_inspect::type_display(ty);
            return Some(quote::quote! {
                ::miniextendr_api::strict::checked_rows_into_sexp::<#elem, _>(
                    #container, #rows
                )
            });
        }
        if let Some(elem) = lossy_vec_element(inner) {
            let helper = quote::format_ident!("checked_vec_vec_{}_into_sexp", elem);
            return Some(quote::quote! {
                ::miniextendr_api::strict::#helper(#result_ident)
            });
        }
        let inner_name = last_segment_ident(inner)?.to_string();
        if LOSSY_SCALARS.contains(&inner_name.as_str()) {
            let helper = quote::format_ident!("checked_vec_{}_into_sexp", inner_name);
            return Some(quote::quote! {
                ::miniextendr_api::strict::#helper(#result_ident)
            });
        }
        // Check for Vec<Option<lossy>> and Vec<Option<Vec<lossy>>>
        if inner_name == "Option"
            && let Some(option_inner) = first_type_arg_from_type(inner)
        {
            if let Some(elem) = lossy_vec_element(option_inner) {
                let helper = quote::format_ident!("checked_vec_option_vec_{}_into_sexp", elem);
                return Some(quote::quote! {
                    ::miniextendr_api::strict::#helper(#result_ident)
                });
            }
            let option_inner_name = last_segment_ident(option_inner)?.to_string();
            if LOSSY_SCALARS.contains(&option_inner_name.as_str()) {
                let helper =
                    quote::format_ident!("checked_vec_option_{}_into_sexp", option_inner_name);
                return Some(quote::quote! {
                    ::miniextendr_api::strict::#helper(#result_ident)
                });
            }
        }
    }

    None
}

/// Try to generate a strict conversion expression for a lossy input parameter type.
///
/// Returns `Some(TokenStream)` if the type is a lossy scalar, `Vec<lossy>`,
/// or `Vec<Option<lossy>>`, otherwise `None` (falls through to standard
/// `TryFromSexp`). The expression is a `Result<_, SexpError>`: the caller
/// binds it like any other conversion, so a rejected input is the argument
/// error (`kind = "conversion"`, `e$param`, `e$rust_type`), not a panic.
///
/// # Parameters
/// - `ty`: The Rust parameter type to check
/// - `sexp_ident`: Identifier for the SEXP variable holding the R value
pub(crate) fn strict_input_conversion_for_type(
    ty: &syn::Type,
    sexp_ident: &syn::Ident,
) -> Option<proc_macro2::TokenStream> {
    let type_name = last_segment_ident(ty)?;
    let name = type_name.to_string();

    // Check for scalar lossy types: i64, u64, isize, usize
    if LOSSY_SCALARS.contains(&name.as_str()) {
        let helper = quote::format_ident!("checked_try_from_sexp_{}", name);
        return Some(quote::quote! {
            ::miniextendr_api::strict::#helper(#sexp_ident)
        });
    }

    // Check for Vec<lossy> or Vec<Option<lossy>>
    if name == "Vec"
        && let Some(inner) = first_type_arg_from_type(ty)
    {
        let inner_name = last_segment_ident(inner)?.to_string();
        if LOSSY_SCALARS.contains(&inner_name.as_str()) {
            let helper = quote::format_ident!("checked_vec_try_from_sexp_{}", inner_name);
            return Some(quote::quote! {
                ::miniextendr_api::strict::#helper(#sexp_ident)
            });
        }
        // Check for Vec<Option<lossy>> — must apply the same input-SEXP-type
        // gate as Vec<lossy> (reject LGLSXP/RAWSXP), not fall through to the
        // coercing TryFromSexp path. NA elements still become `None`.
        if inner_name == "Option"
            && let Some(option_inner) = first_type_arg_from_type(inner)
        {
            let option_inner_name = last_segment_ident(option_inner)?.to_string();
            if LOSSY_SCALARS.contains(&option_inner_name.as_str()) {
                let helper =
                    quote::format_ident!("checked_vec_option_try_from_sexp_{}", option_inner_name);
                return Some(quote::quote! {
                    ::miniextendr_api::strict::#helper(#sexp_ident)
                });
            }
        }
    }

    None
}

/// Extract the last path segment identifier from a type.
pub(crate) fn last_segment_ident(ty: &syn::Type) -> Option<&syn::Ident> {
    if let syn::Type::Path(p) = ty {
        p.path.segments.last().map(|s| &s.ident)
    } else {
        None
    }
}

/// Extract the first generic type argument from a type (e.g., `T` from `Vec<T>`).
pub(crate) fn first_type_arg_from_type(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(p) = ty {
        crate::first_type_argument(p.path.segments.last()?)
    } else {
        None
    }
}

#[cfg(test)]
mod tests;
