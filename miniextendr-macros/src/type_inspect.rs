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

/// Peel a return-visibility marker (`Invisible<T>` / `Visible<T>`, matched
/// on the last path segment like `Dots` / `Missing`).
///
/// Returns `(Some(true), &T)` for `Invisible<T>`, `(Some(false), &T)` for
/// `Visible<T>`, and `(None, ty)` for anything else, including a bare
/// `Invisible` with no type argument (left alone; the fail-safe direction is
/// "not a marker", and `IntoR` on the marker types keeps the conversion
/// correct). Nested markers are the caller's error to report; see
/// [`visibility_marker_error`].
pub(crate) fn peel_visibility_marker(ty: &syn::Type) -> (Option<bool>, &syn::Type) {
    let syn::Type::Path(p) = ty else {
        return (None, ty);
    };
    let Some(seg) = p.path.segments.last() else {
        return (None, ty);
    };
    let invisible = match seg.ident.to_string().as_str() {
        "Invisible" => true,
        "Visible" => false,
        _ => return (None, ty),
    };
    match first_type_argument(seg) {
        Some(inner) => (Some(invisible), inner),
        None => (None, ty),
    }
}

/// [`peel_visibility_marker`] over a whole `syn::ReturnType`: returns the
/// marker's decision and an owned return type with the marker removed, so the
/// rest of the analysis (`Option`/`Result` shapes, `Self`, thread strategy)
/// sees exactly what it would without the marker.
pub(crate) fn peel_return_visibility(output: &syn::ReturnType) -> (Option<bool>, syn::ReturnType) {
    match output {
        syn::ReturnType::Type(arrow, ty) => {
            let (marker, inner) = peel_visibility_marker(ty);
            match marker {
                Some(_) => (
                    marker,
                    syn::ReturnType::Type(*arrow, Box::new(inner.clone())),
                ),
                None => (None, output.clone()),
            }
        }
        syn::ReturnType::Default => (None, output.clone()),
    }
}

/// Model `#[miniextendr(serialize)]` exactly like a declared `AsSerialize<T>`
/// return, after peeling an outer visibility marker. In particular, `Option`
/// and `Result` inside it are serde data, not boundary control flow.
pub(crate) fn serialize_return_type(output: &syn::ReturnType, serialize: bool) -> syn::ReturnType {
    if !serialize {
        return output.clone();
    }
    let ty: syn::Type = match output {
        syn::ReturnType::Default => syn::parse_quote!(()),
        syn::ReturnType::Type(_, ty) => (**ty).clone(),
    };
    syn::parse_quote!(-> ::miniextendr_api::serde::AsSerialize<#ty>)
}

/// Prepare the value to match the type used by return analysis. Both functions
/// and method/shim paths unwrap visibility before applying serde transport.
pub(crate) fn prepare_return_value(
    call: proc_macro2::TokenStream,
    visibility_marker: Option<bool>,
    serialize: bool,
) -> proc_macro2::TokenStream {
    let value = if visibility_marker.is_some() {
        quote::quote! { (#call).0 }
    } else {
        call
    };
    if serialize {
        quote::quote! { ::miniextendr_api::serde::AsSerialize(#value) }
    } else {
        value
    }
}

/// Resolve the wrapper's visibility from the three sources, most explicit
/// first: a return-type marker, then the `invisible` / `visible` attribute,
/// then the shape default. A marker and an attribute that disagree are an
/// error at `span`.
pub(crate) fn resolve_visibility(
    marker: Option<bool>,
    attr: Option<bool>,
    shape_default: bool,
    span: proc_macro2::Span,
) -> syn::Result<bool> {
    match (marker, attr) {
        (Some(m), Some(a)) if m != a => Err(syn::Error::new(
            span,
            "the return type's visibility marker and the `invisible` / `visible` attribute disagree; keep one of them (or make them agree)",
        )),
        (Some(m), _) => Ok(m),
        (None, Some(a)) => Ok(a),
        (None, None) => Ok(shape_default),
    }
}

/// The error for a marker that cannot be honoured: a nested marker
/// (`Invisible<Visible<T>>`), a marker in argument position, or, the mirror
/// case, a parameter marker (`Checked<T>` / `Unchecked<T>`) in return
/// position (also inside a visibility marker).
pub(crate) fn visibility_marker_error(ty: &syn::Type, what: &str) -> Option<syn::Error> {
    let (marker, inner) = peel_visibility_marker(ty);
    if what == "return"
        && let Some((param_marker, _)) = outer_param_marker(inner)
    {
        return Some(syn::Error::new_spanned(
            ty,
            format!(
                "`{}<T>` marks a parameter only; it cannot be used as a return type (return the \
                 inner `T`)",
                param_marker.name()
            ),
        ));
    }
    marker?;
    if what == "argument" {
        return Some(syn::Error::new_spanned(
            ty,
            "`Invisible<T>` / `Visible<T>` mark the return type only; they cannot be used as a parameter type",
        ));
    }
    if peel_visibility_marker(inner).0.is_some() {
        return Some(syn::Error::new_spanned(
            ty,
            "visibility markers cannot be nested; use a single `Invisible<T>` or `Visible<T>` around the return type",
        ));
    }
    None
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

// region: condition-call markers (#1566)

/// Detect a condition-call marker parameter type: `Call` selects `wrapper`
/// attribution, `CallerCall` selects `caller`. Matched on the last path
/// segment like the visibility markers; a segment carrying generic arguments
/// (`Call<T>`) is some other type. References and wrappers (`&Call`,
/// `Option<Call>`) are not markers: the C wrapper binds the marker by value
/// from its hidden call slot.
pub(crate) fn call_marker(ty: &syn::Type) -> Option<crate::r_wrapper_builder::CallAttribution> {
    let syn::Type::Path(p) = ty else {
        return None;
    };
    let seg = p.path.segments.last()?;
    if !matches!(seg.arguments, syn::PathArguments::None) {
        return None;
    }
    match seg.ident.to_string().as_str() {
        "Call" => Some(crate::r_wrapper_builder::CallAttribution::Wrapper),
        "CallerCall" => Some(crate::r_wrapper_builder::CallAttribution::Caller),
        _ => None,
    }
}

// endregion

// region: argument-count marker (#1860)

/// Detect the argument-count marker parameter type, `NArgs`: matched on the
/// last path segment like [`call_marker`], with no generic arguments.
/// References and wrappers (`&NArgs`, `Option<NArgs>`) are not the marker.
///
/// The parameter is no R formal: the R wrapper passes `nargs()` at its
/// position in the `.Call()` (`RArgumentBuilder::build_call_args_vec`), and the
/// C wrapper converts it like any parameter (`TryFromSexp for NArgs`).
pub(crate) fn is_nargs_marker(ty: &syn::Type) -> bool {
    let syn::Type::Path(p) = ty else {
        return false;
    };
    p.path
        .segments
        .last()
        .is_some_and(|seg| seg.ident == "NArgs" && seg.arguments.is_none())
}

// endregion

// region: RNG guard parameter (#2002)

/// Detect the RNG guard parameter type, `&RngGuard` (a shared reference to a
/// path whose last segment is `RngGuard`, with no generic arguments, any
/// lifetime). Under `#[miniextendr(rng)]` the parameter is no R formal: the
/// wrapper has loaded R's RNG state and passes its guard at that position.
pub(crate) fn is_rng_guard_param(ty: &syn::Type) -> bool {
    let syn::Type::Reference(reference) = ty else {
        return false;
    };
    reference.mutability.is_none()
        && matches!(reference.elem.as_ref(), syn::Type::Path(p) if p.qself.is_none() && p
            .path
            .segments
            .last()
            .is_some_and(|seg| seg.ident == "RngGuard" && seg.arguments.is_none()))
}

/// Whether `ty` names `RngGuard` anywhere: a parameter that does without
/// being [`is_rng_guard_param`] (`&mut RngGuard`, `RngGuard` by value,
/// `Option<&RngGuard>`) is a shape the wrapper cannot fill.
pub(crate) fn mentions_rng_guard(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Path(p) => p.path.segments.iter().any(|seg| {
            seg.ident == "RngGuard"
                || match &seg.arguments {
                    syn::PathArguments::AngleBracketed(ab) => ab.args.iter().any(
                        |arg| matches!(arg, syn::GenericArgument::Type(t) if mentions_rng_guard(t)),
                    ),
                    _ => false,
                }
        }),
        syn::Type::Reference(r) => mentions_rng_guard(&r.elem),
        syn::Type::Paren(p) => mentions_rng_guard(&p.elem),
        syn::Type::Group(g) => mentions_rng_guard(&g.elem),
        syn::Type::Tuple(t) => t.elems.iter().any(mentions_rng_guard),
        syn::Type::Array(a) => mentions_rng_guard(&a.elem),
        syn::Type::Slice(s) => mentions_rng_guard(&s.elem),
        _ => false,
    }
}

/// The argument expression for an `&RngGuard` parameter: a borrow of a guard
/// made at the call, which the wrapper has loaded the state for and saves
/// itself (the guard saves nothing when it drops). No binding, so no user
/// parameter name can capture it.
pub(crate) fn rng_guard_arg() -> proc_macro2::TokenStream {
    quote::quote! {
        &unsafe { ::miniextendr_api::rng::RngGuard::__mx_loaded_by_wrapper() }
    }
}

// endregion

// region: borrowed vector views (#1427)

/// Conversions selected from the rooted call argument rather than a raw SEXP.
/// The Cow vector cases are character vectors, not arbitrary lists of views.
pub(crate) fn is_rooted_view(ty: &syn::Type) -> bool {
    let syn::Type::Path(path) = ty else {
        return false;
    };
    if path.qself.is_some() {
        return false;
    }
    let Some(seg) = path.path.segments.last() else {
        return false;
    };
    if seg.ident == "StrVec" || seg.ident == "RCow" || seg.ident == "Cow" {
        return true;
    }
    seg.ident == "Vec"
        && first_type_argument(seg).is_some_and(|elem| {
            let elem = option_inner_type(elem).unwrap_or(elem);
            matches!(elem, syn::Type::Path(p) if p.qself.is_none() && p.path.segments.last()
            .is_some_and(|s| s.ident == "Cow"))
        })
}

/// Explicit shared reference forms whose main-thread conversions borrow a root:
/// the references, their `Option`s, and `Vec`s of either (nested `Vec`s too).
/// Mutable and custom references keep their own conversion path.
pub(crate) fn is_rooted_reference(ty: &syn::Type) -> bool {
    let ty = option_inner_type(ty).unwrap_or(ty);
    if let syn::Type::Path(path) = ty
        && path.qself.is_none()
        && let Some(segment) = path.path.segments.last()
        && segment.ident == "Vec"
    {
        return first_type_argument(segment).is_some_and(is_rooted_reference);
    }
    let syn::Type::Reference(reference) = ty else {
        return false;
    };
    if reference.mutability.is_some() {
        return false;
    }
    match reference.elem.as_ref() {
        syn::Type::Slice(_) => true,
        syn::Type::Path(path) if path.qself.is_none() => {
            path.path.segments.last().is_some_and(|seg| {
                seg.arguments.is_none()
                    && matches!(
                        seg.ident.to_string().as_str(),
                        "i32" | "f64" | "u8" | "RLogical" | "Rcomplex" | "str"
                    )
            })
        }
        _ => false,
    }
}

/// A `'static` borrowed parameter (`StrVec<'static>`, `Cow<'static, str>`,
/// `&'static [T]`, `Option<&'static str>`) can never be bound: its root is the
/// call's argument slot, or a copy that ends with the call. Returns the
/// statement that binds `ident` with a `compile_error!` naming the fix,
/// spanned at the lifetime, in place of a borrow-checker error about a
/// generated temporary. The binding keeps the code
/// that uses `ident` free of follow-on errors; it does not diverge, so that
/// code draws no unreachable warning.
pub(crate) fn static_rooted_view_binding(
    ident: &syn::Ident,
    ty: &syn::Type,
) -> Option<proc_macro2::TokenStream> {
    let lifetime = static_lifetime(ty)?;
    Some(quote::quote_spanned! {lifetime.span()=>
        ::core::compile_error!(
            "this parameter borrows the call's argument and cannot be `'static`; write `'_` or \
             elide the lifetime, or take an owned type (`ProtectedStrVec`, `Vec`, `String`) to \
             keep the data after the call"
        );
        let #ident: #ty = ::core::option::Option::<#ty>::None.unwrap();
    })
}

/// The first `'static` lifetime written anywhere in `ty`.
pub(crate) fn static_lifetime(ty: &syn::Type) -> Option<&syn::Lifetime> {
    let is_static = |lt: &&syn::Lifetime| lt.ident == "static";
    match ty {
        syn::Type::Reference(r) => r
            .lifetime
            .as_ref()
            .filter(is_static)
            .or_else(|| static_lifetime(&r.elem)),
        syn::Type::Path(p) => p.path.segments.iter().find_map(|seg| {
            let syn::PathArguments::AngleBracketed(args) = &seg.arguments else {
                return None;
            };
            args.args.iter().find_map(|arg| match arg {
                syn::GenericArgument::Lifetime(lt) => Some(lt).filter(is_static),
                syn::GenericArgument::Type(ty) => static_lifetime(ty),
                _ => None,
            })
        }),
        syn::Type::Slice(s) => static_lifetime(&s.elem),
        syn::Type::Array(a) => static_lifetime(&a.elem),
        syn::Type::Paren(p) => static_lifetime(&p.elem),
        syn::Type::Group(g) => static_lifetime(&g.elem),
        syn::Type::Tuple(t) => t.elems.iter().find_map(static_lifetime),
        _ => None,
    }
}

// endregion

// region: unforced dots (#1892)

/// Detect the unforced-dots parameter type, `LazyDots`: matched on the last
/// path segment like the other markers, bare or with lifetime arguments only
/// (`LazyDots<'_>`). Only the whole type by value is the parameter; any other
/// shape (`&LazyDots`, `Option<LazyDots>`, `Missing<LazyDots>`) is refused by
/// the callers through [`mentions_lazy_dots`].
///
/// The parameter is R's `...` at its position: the R wrapper passes
/// `environment()` there (`RArgumentBuilder::build_call_args_vec`), and the C
/// wrapper binds it with `LazyDots::from_wrapper_arg`.
pub(crate) fn is_lazy_dots_marker(ty: &syn::Type) -> bool {
    let syn::Type::Path(p) = ty else {
        return false;
    };
    p.path.segments.last().is_some_and(|seg| {
        seg.ident == "LazyDots"
            && match &seg.arguments {
                syn::PathArguments::None => true,
                syn::PathArguments::AngleBracketed(ab) => ab
                    .args
                    .iter()
                    .all(|arg| matches!(arg, syn::GenericArgument::Lifetime(_))),
                syn::PathArguments::Parenthesized(_) => false,
            }
    })
}

/// Whether `ty` names `LazyDots` anywhere, at any depth.
pub(crate) fn mentions_lazy_dots(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Path(p) => p.path.segments.iter().any(|seg| {
            seg.ident == "LazyDots"
                || match &seg.arguments {
                    syn::PathArguments::AngleBracketed(ab) => ab.args.iter().any(
                        |arg| matches!(arg, syn::GenericArgument::Type(t) if mentions_lazy_dots(t)),
                    ),
                    _ => false,
                }
        }),
        syn::Type::Reference(r) => mentions_lazy_dots(&r.elem),
        syn::Type::Paren(p) => mentions_lazy_dots(&p.elem),
        syn::Type::Group(g) => mentions_lazy_dots(&g.elem),
        syn::Type::Tuple(t) => t.elems.iter().any(mentions_lazy_dots),
        syn::Type::Array(a) => mentions_lazy_dots(&a.elem),
        syn::Type::Slice(s) => mentions_lazy_dots(&s.elem),
        _ => false,
    }
}

// endregion

// region: unevaluated-argument markers (#1835)

/// Which unevaluated-argument marker a parameter takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UnevaluatedKind {
    /// `Quoted`: base R's `substitute(x)` and `parent.frame()`.
    Quoted,
    /// `Quosure`: `rlang::enquo(x)`.
    Quosure,
}

impl UnevaluatedKind {
    /// The marker's type name, for diagnostics.
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Quoted => "Quoted",
            Self::Quosure => "Quosure",
        }
    }

    /// The `.Call()` argument the R wrapper passes for parameter `p`: the
    /// missing-argument sentinel, as for `Missing<T>`, else the capture. Never
    /// forces `p`.
    pub(crate) fn r_call_arg(self, p: &str) -> String {
        let capture = match self {
            Self::Quoted => format!("list(substitute({p}), parent.frame())"),
            Self::Quosure => format!("rlang::enquo({p})"),
        };
        format!("if (missing({p})) quote(expr=) else {capture}")
    }

    /// The marker's path in `miniextendr_api`.
    pub(crate) fn api_path(self) -> proc_macro2::TokenStream {
        match self {
            Self::Quoted => quote::quote!(::miniextendr_api::Quoted),
            Self::Quosure => quote::quote!(::miniextendr_api::Quosure),
        }
    }

    /// The marker a path segment names: `Quoted` / `Quosure`, bare or with
    /// lifetime arguments only (`Quoted<'_>`).
    fn of_segment(seg: &syn::PathSegment) -> Option<Self> {
        let lifetimes_only = match &seg.arguments {
            syn::PathArguments::None => true,
            syn::PathArguments::AngleBracketed(ab) => ab
                .args
                .iter()
                .all(|arg| matches!(arg, syn::GenericArgument::Lifetime(_))),
            syn::PathArguments::Parenthesized(_) => false,
        };
        if !lifetimes_only {
            return None;
        }
        match seg.ident.to_string().as_str() {
            "Quoted" => Some(Self::Quoted),
            "Quosure" => Some(Self::Quosure),
            _ => None,
        }
    }
}

/// An unevaluated-argument parameter: `Quoted` / `Quosure`, or either inside
/// `Missing<..>` (`optional`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct UnevaluatedParam {
    pub(crate) kind: UnevaluatedKind,
    /// `Missing<Quoted>` / `Missing<Quosure>`: an omitted argument is
    /// `Missing::Absent` instead of an error.
    pub(crate) optional: bool,
}

/// Detect an unevaluated-argument parameter type, matched on the last path
/// segment like the other markers: `Quoted`, `Quosure`, or one of them as the
/// type argument of `Missing<..>`. Any other position (`Option<Quoted>`,
/// `&Quoted`) is not one; [`mentions_unevaluated_marker`] lets the caller
/// refuse it.
pub(crate) fn unevaluated_param(ty: &syn::Type) -> Option<UnevaluatedParam> {
    let marker = |ty: &syn::Type| match ty {
        syn::Type::Path(p) => p.path.segments.last().and_then(UnevaluatedKind::of_segment),
        _ => None,
    };
    if let Some(kind) = marker(ty) {
        return Some(UnevaluatedParam {
            kind,
            optional: false,
        });
    }
    let inner = crate::miniextendr_fn::get_missing_inner_type(ty)?;
    marker(inner).map(|kind| UnevaluatedParam {
        kind,
        optional: true,
    })
}

/// Whether `ty` names `Quoted` / `Quosure` anywhere, at any depth.
pub(crate) fn mentions_unevaluated_marker(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Path(p) => p.path.segments.iter().any(|seg| {
            matches!(seg.ident.to_string().as_str(), "Quoted" | "Quosure")
                || match &seg.arguments {
                    syn::PathArguments::AngleBracketed(ab) => ab.args.iter().any(|arg| {
                        matches!(arg, syn::GenericArgument::Type(t) if mentions_unevaluated_marker(t))
                    }),
                    _ => false,
                }
        }),
        syn::Type::Reference(r) => mentions_unevaluated_marker(&r.elem),
        syn::Type::Paren(p) => mentions_unevaluated_marker(&p.elem),
        syn::Type::Group(g) => mentions_unevaluated_marker(&g.elem),
        syn::Type::Tuple(t) => t.elems.iter().any(mentions_unevaluated_marker),
        syn::Type::Array(a) => mentions_unevaluated_marker(&a.elem),
        syn::Type::Slice(s) => mentions_unevaluated_marker(&s.elem),
        _ => false,
    }
}

// endregion

// region: parameter markers (#1566 §2)

/// A by-value parameter marker the macro peels from a parameter type before
/// any R-side analysis: the parameter converts as the inner type, and the C
/// wrapper wraps the value ([`ParamMarker::wrap`]) before the call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ParamMarker {
    /// `Checked<T>`: keep the parameter's type-derived R-side checks.
    Checked,
    /// `Unchecked<T>`: drop them.
    Unchecked,
    /// `NonNa<T>`: the parameter's `no_na` check, written in the type; the C
    /// wrapper also checks the argument before converting it.
    NonNa,
}

/// The decision family a [`ParamMarker`] belongs to: a parameter carries at
/// most one marker per family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ParamMarkerFamily {
    /// `Checked` / `Unchecked`: the type-derived R-side checks.
    Preconditions,
    /// `NonNa`: the `no_na` check.
    NoNa,
}

impl ParamMarker {
    /// The marker for a path segment's name, if it is one.
    fn from_ident(ident: &syn::Ident) -> Option<Self> {
        match ident.to_string().as_str() {
            "Checked" => Some(Self::Checked),
            "Unchecked" => Some(Self::Unchecked),
            "NonNa" => Some(Self::NonNa),
            _ => None,
        }
    }

    /// The type's name as written in source.
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Checked => "Checked",
            Self::Unchecked => "Unchecked",
            Self::NonNa => "NonNa",
        }
    }

    /// Which decision the marker makes.
    pub(crate) fn family(self) -> ParamMarkerFamily {
        match self {
            Self::Checked | Self::Unchecked => ParamMarkerFamily::Preconditions,
            Self::NonNa => ParamMarkerFamily::NoNa,
        }
    }

    /// The C wrapper's rebinding of the converted `value` as the marked
    /// type. A `NonNa` is built unchecked: the wrapper refused an argument
    /// holding an `NA` before converting it.
    pub(crate) fn wrap(self, value: &syn::Ident) -> proc_macro2::TokenStream {
        match self {
            Self::Checked => quote::quote!(::miniextendr_api::Checked::from_inner(#value)),
            Self::Unchecked => quote::quote!(::miniextendr_api::Unchecked::from_inner(#value)),
            // SAFETY (of the emitted `unsafe`): the C wrapper ran the
            // marker's `__mx_any_na` check on the argument before converting
            // it, and returned the argument error when it held an `NA`.
            Self::NonNa => {
                quote::quote!(unsafe { ::miniextendr_api::NonNa::new_unchecked(#value) })
            }
        }
    }

    /// The precondition decision: `Some(true)` keeps the type-derived
    /// checks, `Some(false)` drops them; `None` for a marker of another
    /// family.
    pub(crate) fn preconditions(self) -> Option<bool> {
        match self {
            Self::Checked => Some(true),
            Self::Unchecked => Some(false),
            Self::NonNa => None,
        }
    }

    /// The method attribute an impl spells for the marker of a trait
    /// method's parameter `param`: `preconditions(x)`, `no_preconditions(x)`
    /// or `no_na(x)`.
    pub(crate) fn impl_keyword(self, param: &str) -> String {
        match self {
            Self::Checked => format!("preconditions({param})"),
            Self::Unchecked => format!("no_preconditions({param})"),
            Self::NonNa => format!("no_na({param})"),
        }
    }
}

/// The marker a type's outermost segment names, with its single type
/// argument. Matched on the last path segment like [`peel_visibility_marker`];
/// a bare `Checked` (no type argument) or one with two is not a marker (the
/// fail-safe direction: the type then fails to convert).
fn outer_param_marker(ty: &syn::Type) -> Option<(ParamMarker, &syn::Type)> {
    let syn::Type::Path(p) = ty else {
        return None;
    };
    let seg = p.path.segments.last()?;
    let marker = ParamMarker::from_ident(&seg.ident)?;
    let syn::PathArguments::AngleBracketed(ab) = &seg.arguments else {
        return None;
    };
    let mut types = ab.args.iter().filter_map(|arg| match arg {
        syn::GenericArgument::Type(t) => Some(t),
        _ => None,
    });
    let inner = types.next()?;
    if types.next().is_some() {
        return None;
    }
    Some((marker, inner))
}

/// Peel the parameter markers off a parameter type, outermost first: returns
/// the markers in the order written and the inner type the parameter
/// converts as. `Checked<Strict<T>>`-style stacks of different families peel
/// in one pass; two markers of one family (`Checked<Unchecked<T>>`) are an
/// error, as is a marker nested inside `Option` / `Missing` / `Vec` / `&`
/// (at any depth of those), where the conversion has no marker to unwrap.
pub(crate) fn peel_param_markers(ty: &syn::Type) -> syn::Result<(Vec<ParamMarker>, &syn::Type)> {
    let mut markers: Vec<ParamMarker> = Vec::new();
    let mut inner = ty;
    while let Some((marker, next)) = outer_param_marker(inner) {
        if let Some(first) = markers.iter().find(|m| m.family() == marker.family()) {
            let message = match marker.family() {
                ParamMarkerFamily::Preconditions => format!(
                    "at most one precondition marker per parameter: `{}` already decides this \
                     parameter's R-side type checks, so drop `{}`",
                    first.name(),
                    marker.name()
                ),
                ParamMarkerFamily::NoNa => {
                    "`NonNa<NonNa<T>>`: one `NonNa` already refuses an `NA`, so drop the \
                     inner one"
                        .to_string()
                }
            };
            return Err(syn::Error::new_spanned(ty, message));
        }
        markers.push(marker);
        inner = next;
    }
    if markers.contains(&ParamMarker::NonNa) && is_option_type(inner) {
        return Err(syn::Error::new_spanned(
            ty,
            "`NonNa<Option<T>>`: a value without `NA` leaves `Option` only `NULL` to mean; \
             take `NonNa<T>`, or `Option<T>` with `#[miniextendr(no_na)]` to accept `NULL`",
        ));
    }
    if let Some(nested) = nested_param_marker(inner) {
        return Err(syn::Error::new_spanned(
            ty,
            format!(
                "put `{0}` outermost: `{0}<Option<T>>`, not `Option<{0}<T>>`; the marker \
                 applies to the whole parameter",
                nested.name()
            ),
        ));
    }
    Ok((markers, inner))
}

/// A marker inside `Option<..>` / `Missing<..>` / `Vec<..>` / `&..`, at any
/// depth of those wrappers.
fn nested_param_marker(ty: &syn::Type) -> Option<ParamMarker> {
    let arg = match ty {
        syn::Type::Reference(r) => r.elem.as_ref(),
        syn::Type::Paren(p) => p.elem.as_ref(),
        syn::Type::Path(p) => {
            let seg = p.path.segments.last()?;
            if !matches!(seg.ident.to_string().as_str(), "Option" | "Missing" | "Vec") {
                return None;
            }
            first_type_argument(seg)?
        }
        _ => return None,
    };
    match outer_param_marker(arg) {
        Some((marker, _)) => Some(marker),
        None => nested_param_marker(arg),
    }
}

/// Whether any of `markers` is in `family`, and which.
pub(crate) fn marker_in_family(
    markers: &[ParamMarker],
    family: ParamMarkerFamily,
) -> Option<ParamMarker> {
    markers.iter().copied().find(|m| m.family() == family)
}

// endregion

// region: type rendering for messages

/// Render a type the way it is written in Rust source, for user-facing
/// messages: `AsFromStrVec<i32>`, `Either<i32, String>`, `&[f64]`,
/// `&'a str`, `Option<Vec<String>>`.
///
/// `quote!(#ty).to_string()` puts a space between every token
/// (`AsFromStrVec < i32 >`); this walks the tokens and applies the usual
/// spacing instead: none around `<`, `>`, `::`, after `&` / `*` / `'`, or
/// inside brackets; a space after `,` / `;`, around `+` / `=` / `->`, between
/// two words, and between `mut` / `const` / `dyn` / `impl` or a lifetime and
/// a following bracket group (`&mut [T]`, `&'a [T]`).
pub(crate) fn type_display(ty: &syn::Type) -> String {
    let mut out = String::new();
    write_type_tokens(quote::quote!(#ty), &mut out);
    out
}

fn write_type_tokens(tokens: proc_macro2::TokenStream, out: &mut String) {
    use proc_macro2::{Delimiter, Spacing, TokenTree};

    /// What the previous token leaves for the next one to attach to.
    #[derive(Clone, Copy, PartialEq)]
    enum Prev {
        /// Start of the stream, or a token after which nothing is spaced
        /// (`<`, `&`, `*`, `::`, a trailing-space punct like `, `).
        Glue,
        /// A lifetime's `'`: the next ident attaches.
        Tick,
        /// An identifier, literal or closing `>`: a following word is spaced.
        Word,
        /// `mut` / `const` / `dyn` / `impl` or a lifetime: a following word
        /// or group is spaced.
        Keyword,
    }

    let mut prev = Prev::Glue;
    // `-` of a pending `->` and the first `:` of a `::`.
    let mut pending_arrow = false;
    let mut pending_path_sep = false;
    for tt in tokens {
        match tt {
            TokenTree::Ident(ident) => {
                let name = ident.to_string();
                if matches!(prev, Prev::Word | Prev::Keyword) {
                    out.push(' ');
                }
                out.push_str(&name);
                prev = if prev == Prev::Tick
                    || matches!(name.as_str(), "mut" | "const" | "dyn" | "impl")
                {
                    Prev::Keyword
                } else {
                    Prev::Word
                };
            }
            TokenTree::Literal(lit) => {
                if matches!(prev, Prev::Word | Prev::Keyword) {
                    out.push(' ');
                }
                out.push_str(&lit.to_string());
                prev = Prev::Word;
            }
            TokenTree::Group(group) => {
                if prev == Prev::Keyword
                    || (prev == Prev::Word && group.delimiter() == Delimiter::Bracket)
                {
                    out.push(' ');
                }
                let (open, close) = match group.delimiter() {
                    Delimiter::Parenthesis => ("(", ")"),
                    Delimiter::Bracket => ("[", "]"),
                    Delimiter::Brace => ("{", "}"),
                    Delimiter::None => ("", ""),
                };
                out.push_str(open);
                write_type_tokens(group.stream(), out);
                out.push_str(close);
                prev = Prev::Word;
            }
            TokenTree::Punct(punct) => {
                let c = punct.as_char();
                match c {
                    ',' | ';' => {
                        out.push(c);
                        out.push(' ');
                        prev = Prev::Glue;
                    }
                    '+' | '=' => {
                        out.push(' ');
                        out.push(c);
                        out.push(' ');
                        prev = Prev::Glue;
                    }
                    '-' if punct.spacing() == Spacing::Joint => {
                        out.push_str(" -");
                        pending_arrow = true;
                    }
                    '>' if pending_arrow => {
                        out.push_str("> ");
                        pending_arrow = false;
                        prev = Prev::Glue;
                    }
                    '>' => {
                        out.push('>');
                        prev = Prev::Word;
                    }
                    ':' if punct.spacing() == Spacing::Joint => {
                        out.push(':');
                        pending_path_sep = true;
                        prev = Prev::Glue;
                    }
                    ':' if pending_path_sep => {
                        out.push(':');
                        pending_path_sep = false;
                        prev = Prev::Glue;
                    }
                    ':' => {
                        // Associated-type bound: `Item: Clone`.
                        out.push_str(": ");
                        prev = Prev::Glue;
                    }
                    '\'' => {
                        if matches!(prev, Prev::Word | Prev::Keyword) {
                            out.push(' ');
                        }
                        out.push('\'');
                        prev = Prev::Tick;
                    }
                    _ => {
                        out.push(c);
                        prev = Prev::Glue;
                    }
                }
            }
        }
    }
}

/// `ty` with every lifetime replaced by `'_`, for a `let` binding annotation
/// in generated code: `&'a [f64]` → `&'_ [f64]`. The binding then names the
/// target type (so the error type of its conversion is known where the
/// wrapper inspects it) without depending on the user's lifetime parameters
/// being in scope.
pub(crate) fn erase_lifetimes(ty: &syn::Type) -> syn::Type {
    use proc_macro2::{Group, Ident, TokenStream, TokenTree};

    fn erase(tokens: TokenStream) -> TokenStream {
        let mut out = Vec::new();
        let mut iter = tokens.into_iter();
        while let Some(tt) = iter.next() {
            match tt {
                TokenTree::Punct(p) if p.as_char() == '\'' => {
                    out.push(TokenTree::Punct(p));
                    if let Some(TokenTree::Ident(lifetime)) = iter.next() {
                        out.push(TokenTree::Ident(Ident::new("_", lifetime.span())));
                    }
                }
                TokenTree::Group(g) => {
                    let mut erased = Group::new(g.delimiter(), erase(g.stream()));
                    erased.set_span(g.span());
                    out.push(TokenTree::Group(erased));
                }
                other => out.push(other),
            }
        }
        out.into_iter().collect()
    }

    syn::parse2(erase(quote::quote!(#ty))).unwrap_or_else(|_| ty.clone())
}

// endregion

#[cfg(test)]
mod tests;

/// `true` when the last path segment of `ty` is `Option<...>`.
pub(crate) fn is_option_type(ty: &syn::Type) -> bool {
    option_inner_type(ty).is_some()
}

/// Return `T` for an `Option<T>` type, `None` for anything else.
pub(crate) fn option_inner_type(ty: &syn::Type) -> Option<&syn::Type> {
    let syn::Type::Path(tp) = ty else {
        return None;
    };
    let seg = tp.path.segments.last()?;
    if seg.ident != "Option" {
        return None;
    }
    first_type_argument(seg)
}

/// Return `T` for an `ExternalPtr<T>` class handle, `ty` itself otherwise.
///
/// `IntoR for ExternalPtr<T>` produces the same bare `EXTPTRSXP` as a bare
/// `T`, so both return-wrapping paths (inferred and explicit `wrap` /
/// `WrapAs*`) name the class through the handle (#1375).
pub(crate) fn peel_external_ptr(ty: &syn::Type) -> &syn::Type {
    let syn::Type::Path(tp) = ty else {
        return ty;
    };
    match tp.path.segments.last() {
        Some(seg) if seg.ident == "ExternalPtr" => first_type_argument(seg).unwrap_or(ty),
        _ => ty,
    }
}

// region: choice-parameter layers (`match_arg` / `choices`)

/// The wrappers around the value of a `match_arg` / `choices` parameter,
/// peeled outermost first: `Missing<..>`, then `Option<..>`, then
/// `Either<T, R>` with the choice on the left (a scalar choice, or a
/// `several_ok` list: `Either<Vec<T>, R>`, #1612). A plain `T` (or a plain
/// `several_ok` container) has none.
///
/// Each layer changes one contract and leaves the rest to the layer below:
/// `Missing` reports an omitted argument as `Missing::Absent` and keeps the
/// choice vector as the R formal (#1551), `Option` turns `NULL` into `None`
/// (#1473), and `Either` sends every argument that is not character or
/// factor to its `R` arm instead of the choice check.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ChoiceLayers<'a> {
    /// `Missing<..>` is the outermost wrapper.
    pub missing: bool,
    /// `Option<..>` wraps the value (inside `Missing`, if present).
    pub nullable: bool,
    /// `Either<T, R>` wraps the value (inside the others): its `R` arm.
    pub either_right: Option<&'a syn::Type>,
    /// The type under the layers: the `MatchArg` type (or the string type of a
    /// `choices` parameter) for a scalar, the container for `several_ok`.
    pub value: &'a syn::Type,
}

/// Peel the [`ChoiceLayers`] of a choice parameter's type.
pub(crate) fn choice_layers(ty: &syn::Type) -> ChoiceLayers<'_> {
    let (missing, ty) = match crate::miniextendr_fn::get_missing_inner_type(ty) {
        Some(inner) => (true, inner),
        None => (false, ty),
    };
    let (nullable, ty) = match option_inner_type(ty) {
        Some(inner) => (true, inner),
        None => (false, ty),
    };
    let (either_right, value) = match either_arms(ty) {
        Some((left, right)) => (Some(right), left),
        None => (None, ty),
    };
    ChoiceLayers {
        missing,
        nullable,
        either_right,
        value,
    }
}

/// The layer name (`"Missing"` / `"Option"` / `"Either"`) when `ty` is itself
/// a layer type, i.e. when it sits where the choice value should be. Used to
/// reject layers in an unsupported order, such as `Option<Missing<T>>` or
/// `Either<Option<T>, R>`.
pub(crate) fn choice_layer_name(ty: &syn::Type) -> Option<&'static str> {
    if crate::miniextendr_fn::get_missing_inner_type(ty).is_some() {
        Some("Missing")
    } else if option_inner_type(ty).is_some() {
        Some("Option")
    } else if either_arms(ty).is_some() {
        Some("Either")
    } else {
        None
    }
}

/// `(L, R)` for an `Either<L, R>` type (last path segment `Either` with two
/// type arguments), `None` for anything else.
pub(crate) fn either_arms(ty: &syn::Type) -> Option<(&syn::Type, &syn::Type)> {
    let syn::Type::Path(tp) = ty else {
        return None;
    };
    let seg = tp.path.segments.last()?;
    if seg.ident != "Either" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(args) = &seg.arguments else {
        return None;
    };
    let mut types = args.args.iter().filter_map(|arg| match arg {
        syn::GenericArgument::Type(ty) => Some(ty),
        _ => None,
    });
    match (types.next(), types.next(), types.next()) {
        (Some(left), Some(right), None) => Some((left, right)),
        _ => None,
    }
}

/// How R documentation names a value of type `ty`, with its article: `"a data
/// frame"`, `"a list"`, `"a number"`, `"a character vector"`, ... Used for the
/// `R` arm of an `Either<T, R>` choice parameter in its auto-generated
/// `@param` line (`One of "a", "b", or a data frame.`). A type without an R
/// name falls back to its Rust spelling in code format.
pub(crate) fn r_value_noun(ty: &syn::Type) -> String {
    let named = match ty {
        syn::Type::Reference(r) => match r.elem.as_ref() {
            syn::Type::Slice(s) => vector_noun(&s.elem),
            elem => return r_value_noun(elem),
        },
        syn::Type::Slice(s) => vector_noun(&s.elem),
        syn::Type::Path(tp) => {
            tp.path
                .segments
                .last()
                .and_then(|seg| match seg.ident.to_string().as_str() {
                    "DataFrame" | "BuiltDataFrame" => Some("a data frame"),
                    "List" | "ListMut" | "NamedList" | "HashMap" | "BTreeMap" => Some("a list"),
                    "String" | "str" | "char" | "PathBuf" | "Path" => Some("a string"),
                    "f64" | "f32" => Some("a number"),
                    "i8" | "i16" | "i32" | "i64" | "isize" | "u16" | "u32" | "u64" | "usize" => {
                        Some("an integer")
                    }
                    "bool" | "Rbool" | "Rboolean" => Some("TRUE or FALSE"),
                    "Rcomplex" => Some("a complex number"),
                    "ExternalPtr" => Some("an external pointer"),
                    "SEXP" => Some("any other R value"),
                    "Vec" => first_type_argument(seg).and_then(vector_noun),
                    _ => None,
                })
        }
        _ => None,
    };
    named.map_or_else(|| format!("a `{}`", type_display(ty)), str::to_string)
}

/// The R name of a vector with elements of type `elem`, if it has one.
fn vector_noun(elem: &syn::Type) -> Option<&'static str> {
    let syn::Type::Path(tp) = elem else {
        return None;
    };
    let seg = tp.path.segments.last()?;
    match seg.ident.to_string().as_str() {
        "f64" | "f32" => Some("a numeric vector"),
        "i8" | "i16" | "i32" | "i64" | "isize" | "u16" | "u32" | "u64" | "usize" => {
            Some("an integer vector")
        }
        "String" | "str" => Some("a character vector"),
        "bool" | "Rbool" | "Rboolean" => Some("a logical vector"),
        "u8" => Some("a raw vector"),
        "Option" => first_type_argument(seg).and_then(vector_noun),
        _ => None,
    }
}

/// Resolve the `MatchArg`-bound type behind a `match_arg` parameter.
///
/// The [`ChoiceLayers`] are peeled first. A `several_ok` parameter is then a
/// container (`Vec<T>`, `Box<[T]>`, `[T; N]`, `&[T]`), so the element type is
/// the one carrying `CHOICES`; a scalar parameter's value is that type itself.
/// Anything else is returned unchanged, and the `MatchArg` bound on the
/// generated code reports the mistake.
///
/// Shared by the standalone-fn path (`lib.rs`) and the impl-method path
/// (`miniextendr_impl.rs`) so the two cannot resolve the type differently.
pub(crate) fn match_arg_choices_ty(param_ty: &syn::Type, several_ok: bool) -> &syn::Type {
    let value = choice_layers(param_ty).value;
    if several_ok {
        classify_several_ok_container(value)
            .map(|(_, inner)| inner)
            .unwrap_or(value)
    } else {
        value
    }
}

// endregion

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

/// Whether a bare `&Self` / `&mut Self` parameter is mutable.
/// `None` means this is not a Self-reference parameter.
pub(crate) fn self_ref_mutability(ty: &syn::Type) -> Option<bool> {
    if let syn::Type::Reference(reference) = ty
        && let syn::Type::Path(path) = reference.elem.as_ref()
        && path.path.is_ident("Self")
    {
        Some(reference.mutability.is_some())
    } else {
        None
    }
}
