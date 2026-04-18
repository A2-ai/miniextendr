//! Lightweight type-introspection helpers shared by parsing and codegen.

/// Returns the `n`-th generic type argument from a path segment.
pub(crate) fn nth_type_argument(seg: &syn::PathSegment, n: usize) -> Option<&syn::Type> {
    if let syn::PathArguments::AngleBracketed(ab) = &seg.arguments {
        let mut count = 0;
        for arg in ab.args.iter() {
            if let syn::GenericArgument::Type(ty) = arg {
                if count == n {
                    return Some(ty);
                }
                count += 1;
            }
        }
    }
    None
}

/// Returns the first generic type argument from a path segment.
pub(crate) fn first_type_argument(seg: &syn::PathSegment) -> Option<&syn::Type> {
    nth_type_argument(seg, 0)
}

/// Returns the second generic type argument from a path segment.
pub(crate) fn second_type_argument(seg: &syn::PathSegment) -> Option<&syn::Type> {
    nth_type_argument(seg, 1)
}

/// Returns `true` if `ty` is syntactically `SEXP`.
#[inline]
pub(crate) fn is_sexp_type(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Path(p) if p
        .path
        .segments
        .last()
        .map(|s| s.ident == "SEXP")
        .unwrap_or(false))
}

/// Container family for a `several_ok` parameter, returned by
/// [`classify_several_ok_container`].
#[derive(Debug, Clone)]
pub(crate) enum SeveralOkContainer {
    /// `Vec<T>`
    Vec,
    /// `Box<[T]>`
    BoxedSlice,
    /// `[T; N]` — the `usize` is the fixed array length N
    Array(usize),
    /// `&[T]` or `&mut [T]` — allocate `Vec<T>` then borrow
    BorrowedSlice,
}

/// Classify a `several_ok` parameter type into one of the four container
/// families and extract its inner element type `T`.
///
/// Returns `Some((container, inner_ty))` or `None` if the type is not one of
/// the four accepted container shapes.
pub(crate) fn classify_several_ok_container(
    ty: &syn::Type,
) -> Option<(SeveralOkContainer, &syn::Type)> {
    match ty {
        // Vec<T>
        syn::Type::Path(tp) => {
            let seg = tp.path.segments.last()?;
            if seg.ident == "Vec" {
                let inner = first_type_argument(seg)?;
                return Some((SeveralOkContainer::Vec, inner));
            }
            // Box<[T]>
            if seg.ident == "Box"
                && let syn::PathArguments::AngleBracketed(ab) = &seg.arguments
            {
                for arg in &ab.args {
                    if let syn::GenericArgument::Type(syn::Type::Slice(s)) = arg {
                        return Some((SeveralOkContainer::BoxedSlice, s.elem.as_ref()));
                    }
                }
            }
            None
        }
        // [T; N]
        syn::Type::Array(arr) => {
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(n),
                ..
            }) = &arr.len
            {
                let n = n.base10_parse::<usize>().ok()?;
                return Some((SeveralOkContainer::Array(n), arr.elem.as_ref()));
            }
            None
        }
        // &[T] or &mut [T]
        syn::Type::Reference(r) => {
            if let syn::Type::Slice(s) = r.elem.as_ref() {
                return Some((SeveralOkContainer::BorrowedSlice, s.elem.as_ref()));
            }
            None
        }
        _ => None,
    }
}
