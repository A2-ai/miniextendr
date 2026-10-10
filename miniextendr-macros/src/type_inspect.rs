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

/// Framework type names that hold R memory and are `!Send` by design.
///
/// Used only for thread-strategy selection: values of these types can neither
/// move into the worker closure nor cross back out of it (`run_on_worker`
/// requires `Send`), so any function touching one stays on the main thread
/// even under `worker-default`. Covers the raw `SEXP`, `AltrepSexp`, the
/// zero-copy R-backed views (`RDVector`, `RDMatrix`, `RndVec`, `RndMat`,
/// `ProtectedStrVec`), and the owned GC-rooted handles (`BuiltDataFrame`,
/// `DataFrameShape`). Arbitrary user `!Send` types can't be detected
/// syntactically — those need an explicit `no_worker`.
const MAIN_THREAD_BOUND: &[&str] = &[
    "SEXP",
    // Condition-call markers wrap the `.call` slot's SEXP (#1566).
    "Call",
    "CallerCall",
    // Unevaluated-argument markers hold the argument's expression and
    // environment (#1835); `Missing<..>` around them is checked by the caller.
    "Quoted",
    "Quosure",
    // The unforced dots hold the wrapper's frame and evaluate R code (#1892).
    "LazyDots",
    "AltrepSexp",
    "RDVector",
    "RDMatrix",
    "RndVec",
    "RndMat",
    "ProtectedStrVec",
    "BuiltDataFrame",
    "DataFrameShape",
];

/// Returns `true` if `ty` is an input type bound to the R main thread.
///
/// Checks only the outermost path segment: main-thread-bound inputs arrive
/// bare (`x: SEXP`, `v: RDVector<f64>`), never nested inside containers.
/// Return-type analysis keeps the narrower [`is_sexp_type`] and uses the
/// recursive [`is_main_thread_bound_return`] for thread selection.
#[inline]
pub(crate) fn is_main_thread_bound_input(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Path(p) if p
        .path
        .segments
        .last()
        .map(|s| MAIN_THREAD_BOUND.contains(&s.ident.to_string().as_str()))
        .unwrap_or(false))
}

/// Returns `true` if `ty` is (or contains) a main-thread-bound type anywhere
/// in a return position — e.g. `BuiltDataFrame`, `Result<BuiltDataFrame,
/// String>`, `Option<DataFrameShape>`, `Vec<BuiltDataFrame>`.
///
/// Unlike inputs, main-thread-bound returns routinely nest inside `Result` /
/// `Option` / containers, so this walks the whole type tree. Under
/// `worker-default` a function whose return type matches is forced onto the
/// main thread: the value owns R memory (`!Send`) and cannot cross back from
/// the worker (`run_on_worker` requires `T: Send`).
pub(crate) fn is_main_thread_bound_return(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Path(p) => p.path.segments.last().is_some_and(|seg| {
            if MAIN_THREAD_BOUND.contains(&seg.ident.to_string().as_str()) {
                return true;
            }
            if let syn::PathArguments::AngleBracketed(ab) = &seg.arguments {
                return ab.args.iter().any(|arg| {
                    matches!(arg, syn::GenericArgument::Type(t) if is_main_thread_bound_return(t))
                });
            }
            false
        }),
        syn::Type::Reference(r) => is_main_thread_bound_return(&r.elem),
        syn::Type::Paren(p) => is_main_thread_bound_return(&p.elem),
        syn::Type::Group(g) => is_main_thread_bound_return(&g.elem),
        syn::Type::Tuple(t) => t.elems.iter().any(is_main_thread_bound_return),
        syn::Type::Array(a) => is_main_thread_bound_return(&a.elem),
        syn::Type::Slice(s) => is_main_thread_bound_return(&s.elem),
        _ => false,
    }
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
/// wrapper wraps the value (`<ctor>::from_inner(x)`) before the call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ParamMarker {
    /// `Checked<T>`: keep the parameter's type-derived R-side checks.
    Checked,
    /// `Unchecked<T>`: drop them.
    Unchecked,
}

/// The decision family a [`ParamMarker`] belongs to: a parameter carries at
/// most one marker per family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ParamMarkerFamily {
    /// `Checked` / `Unchecked`: the type-derived R-side checks.
    Preconditions,
}

impl ParamMarker {
    /// The marker for a path segment's name, if it is one.
    fn from_ident(ident: &syn::Ident) -> Option<Self> {
        match ident.to_string().as_str() {
            "Checked" => Some(Self::Checked),
            "Unchecked" => Some(Self::Unchecked),
            _ => None,
        }
    }

    /// The type's name as written in source.
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Checked => "Checked",
            Self::Unchecked => "Unchecked",
        }
    }

    /// Which decision the marker makes.
    pub(crate) fn family(self) -> ParamMarkerFamily {
        match self {
            Self::Checked | Self::Unchecked => ParamMarkerFamily::Preconditions,
        }
    }

    /// The runtime type, for the rebinding the C wrapper emits.
    pub(crate) fn ctor_path(self) -> proc_macro2::TokenStream {
        match self {
            Self::Checked => quote::quote!(::miniextendr_api::Checked),
            Self::Unchecked => quote::quote!(::miniextendr_api::Unchecked),
        }
    }

    /// The precondition decision: `Some(true)` keeps the type-derived
    /// checks, `Some(false)` drops them.
    pub(crate) fn preconditions(self) -> Option<bool> {
        match self {
            Self::Checked => Some(true),
            Self::Unchecked => Some(false),
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
            return Err(syn::Error::new_spanned(
                ty,
                format!(
                    "at most one precondition marker per parameter: `{}` already decides this \
                     parameter's R-side type checks, so drop `{}`",
                    first.name(),
                    marker.name()
                ),
            ));
        }
        markers.push(marker);
        inner = next;
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
mod tests {
    use super::{
        ParamMarker, UnevaluatedKind, UnevaluatedParam, call_marker, choice_layer_name,
        choice_layers, erase_lifetimes, is_lazy_dots_marker, is_main_thread_bound_input,
        is_main_thread_bound_return, is_nargs_marker, match_arg_choices_ty, mentions_lazy_dots,
        mentions_unevaluated_marker, peel_param_markers, r_value_noun, type_display,
        unevaluated_param, visibility_marker_error,
    };
    use crate::r_wrapper_builder::CallAttribution;

    fn ty(s: &str) -> syn::Type {
        syn::parse_str(s).unwrap()
    }

    #[test]
    fn main_thread_bound_return_detects_nested_positions() {
        // Bare and fully-qualified paths
        assert!(is_main_thread_bound_return(&ty("BuiltDataFrame")));
        assert!(is_main_thread_bound_return(&ty(
            "miniextendr_api::dataframe::BuiltDataFrame"
        )));
        assert!(is_main_thread_bound_return(&ty("DataFrameShape")));
        assert!(is_main_thread_bound_return(&ty("SEXP")));
        // Nested inside Result / Option / containers / tuples
        assert!(is_main_thread_bound_return(&ty(
            "Result<BuiltDataFrame, String>"
        )));
        assert!(is_main_thread_bound_return(&ty(
            "Result<DataFrameShape, std::string::String>"
        )));
        assert!(is_main_thread_bound_return(&ty("Option<BuiltDataFrame>")));
        assert!(is_main_thread_bound_return(&ty("Vec<BuiltDataFrame>")));
        assert!(is_main_thread_bound_return(&ty("(i32, BuiltDataFrame)")));
        // Send-safe returns stay worker-eligible
        assert!(!is_main_thread_bound_return(&ty("i32")));
        assert!(!is_main_thread_bound_return(&ty(
            "Result<Vec<f64>, String>"
        )));
        assert!(!is_main_thread_bound_return(&ty("ExternalPtr<MyType>")));
        assert!(!is_main_thread_bound_return(&ty("DataFrame")));
    }

    #[test]
    fn unevaluated_param_matches_bare_and_missing_markers_only() {
        let required = |kind| {
            Some(UnevaluatedParam {
                kind,
                optional: false,
            })
        };
        let optional = |kind| {
            Some(UnevaluatedParam {
                kind,
                optional: true,
            })
        };
        assert_eq!(
            unevaluated_param(&ty("Quoted")),
            required(UnevaluatedKind::Quoted)
        );
        assert_eq!(
            unevaluated_param(&ty("Quoted<'_>")),
            required(UnevaluatedKind::Quoted)
        );
        assert_eq!(
            unevaluated_param(&ty("::miniextendr_api::Quosure<'a>")),
            required(UnevaluatedKind::Quosure)
        );
        assert_eq!(
            unevaluated_param(&ty("Missing<Quoted>")),
            optional(UnevaluatedKind::Quoted)
        );
        assert_eq!(
            unevaluated_param(&ty("miniextendr_api::Missing<Quosure<'_>>")),
            optional(UnevaluatedKind::Quosure)
        );
        for other in [
            "Option<Quoted>",
            "&Quoted",
            "Vec<Quosure>",
            "Quoted<i32>",
            "Missing<Option<Quoted>>",
            "SEXP",
        ] {
            assert_eq!(unevaluated_param(&ty(other)), None, "{other}");
        }
        // The misplaced shapes are still recognised as mentioning a marker, so
        // the macro can refuse them by name.
        for misplaced in [
            "Option<Quoted>",
            "&Quoted",
            "Vec<Quosure<'_>>",
            "(i32, Quoted)",
        ] {
            assert!(mentions_unevaluated_marker(&ty(misplaced)), "{misplaced}");
        }
        assert!(!mentions_unevaluated_marker(&ty("Vec<i32>")));
        // Bare markers are main-thread-bound inputs; `Missing<..>` is checked
        // by the caller.
        assert!(is_main_thread_bound_input(&ty("Quoted")));
        assert!(is_main_thread_bound_input(&ty("Quosure<'_>")));
    }

    #[test]
    fn unevaluated_kind_r_call_args_never_force_the_argument() {
        assert_eq!(
            UnevaluatedKind::Quoted.r_call_arg("cond"),
            "if (missing(cond)) quote(expr=) else list(substitute(cond), parent.frame())"
        );
        assert_eq!(
            UnevaluatedKind::Quosure.r_call_arg("cols"),
            "if (missing(cols)) quote(expr=) else rlang::enquo(cols)"
        );
    }

    #[test]
    fn call_marker_matches_bare_marker_types_only() {
        assert_eq!(call_marker(&ty("Call")), Some(CallAttribution::Wrapper));
        assert_eq!(
            call_marker(&ty("miniextendr_api::CallerCall")),
            Some(CallAttribution::Caller)
        );
        assert_eq!(
            call_marker(&ty("::miniextendr_api::call_marker::Call")),
            Some(CallAttribution::Wrapper)
        );
        // Not markers: generics, references, wrappers, other types.
        assert_eq!(call_marker(&ty("Call<i32>")), None);
        assert_eq!(call_marker(&ty("&Call")), None);
        assert_eq!(call_marker(&ty("Option<Call>")), None);
        assert_eq!(call_marker(&ty("SEXP")), None);
        assert_eq!(call_marker(&ty("Caller")), None);
        // A marker holds the `.call` slot's SEXP, so it pins the main thread.
        assert!(is_main_thread_bound_input(&ty("Call")));
        assert!(is_main_thread_bound_input(&ty(
            "miniextendr_api::CallerCall"
        )));
    }

    #[test]
    fn nargs_marker_matches_the_bare_type_only() {
        assert!(is_nargs_marker(&ty("NArgs")));
        assert!(is_nargs_marker(&ty("miniextendr_api::NArgs")));
        assert!(is_nargs_marker(&ty("::miniextendr_api::nargs::NArgs")));
        // Not the marker: generics, references, wrappers, other types.
        assert!(!is_nargs_marker(&ty("NArgs<i32>")));
        assert!(!is_nargs_marker(&ty("&NArgs")));
        assert!(!is_nargs_marker(&ty("Option<NArgs>")));
        assert!(!is_nargs_marker(&ty("Missing<NArgs>")));
        assert!(!is_nargs_marker(&ty("Nargs")));
        // A plain count: it does not pin the main thread.
        assert!(!is_main_thread_bound_input(&ty("NArgs")));
    }

    #[test]
    fn lazy_dots_marker_matches_the_whole_type_by_value_only() {
        assert!(is_lazy_dots_marker(&ty("LazyDots")));
        assert!(is_lazy_dots_marker(&ty("LazyDots<'_>")));
        assert!(is_lazy_dots_marker(&ty("miniextendr_api::LazyDots<'a>")));
        assert!(is_lazy_dots_marker(&ty(
            "::miniextendr_api::lazy_dots::LazyDots"
        )));
        // Not the parameter: generics, references, wrappers, other types.
        for other in [
            "LazyDots<i32>",
            "&LazyDots",
            "Option<LazyDots>",
            "Missing<LazyDots>",
            "Dots",
            "&Dots",
        ] {
            assert!(!is_lazy_dots_marker(&ty(other)), "{other}");
        }
        // The misplaced shapes are recognised, so the macro can refuse them.
        for misplaced in [
            "&LazyDots",
            "Option<LazyDots<'_>>",
            "Vec<LazyDots>",
            "Missing<LazyDots>",
        ] {
            assert!(mentions_lazy_dots(&ty(misplaced)), "{misplaced}");
        }
        assert!(!mentions_lazy_dots(&ty("&Dots")));
        // It holds the wrapper's frame, so it pins the main thread.
        assert!(is_main_thread_bound_input(&ty("LazyDots<'_>")));
    }

    /// The inner type a parameter converts as, and the markers peeled off it.
    fn peeled(s: &str) -> (Vec<ParamMarker>, String) {
        let t = ty(s);
        let (markers, inner) = peel_param_markers(&t).expect("peels");
        (markers, type_display(inner))
    }

    fn peel_error(s: &str) -> String {
        peel_param_markers(&ty(s))
            .expect_err("rejected")
            .to_string()
    }

    #[test]
    fn param_markers_peel_the_outermost_marker() {
        assert_eq!(
            peeled("Checked<i32>"),
            (vec![ParamMarker::Checked], "i32".into())
        );
        assert_eq!(
            peeled("miniextendr_api::Unchecked<Vec<f64>>"),
            (vec![ParamMarker::Unchecked], "Vec<f64>".into())
        );
        assert_eq!(
            peeled("::miniextendr_api::precondition_marker::Checked<Option<i32>>"),
            (vec![ParamMarker::Checked], "Option<i32>".into())
        );
        assert_eq!(
            peeled("Checked<&'a [f64]>"),
            (vec![ParamMarker::Checked], "&'a [f64]".into())
        );
        // Not markers: no type argument, two of them, or another type.
        assert_eq!(peeled("Checked"), (vec![], "Checked".into()));
        assert_eq!(
            peeled("Checked<i32, f64>"),
            (vec![], "Checked<i32, f64>".into())
        );
        assert_eq!(peeled("Vec<i32>"), (vec![], "Vec<i32>".into()));
        assert_eq!(peeled("Missing<i32>"), (vec![], "Missing<i32>".into()));
    }

    #[test]
    fn param_markers_reject_stacks_and_nesting() {
        for stacked in ["Checked<Unchecked<i32>>", "Checked<Checked<i32>>"] {
            let err = peel_error(stacked);
            assert!(
                err.contains("at most one precondition marker per parameter"),
                "{stacked}: {err}"
            );
        }
        for nested in [
            "Option<Checked<i32>>",
            "Missing<Unchecked<i32>>",
            "Vec<Checked<i32>>",
            "&Checked<i32>",
            "Option<Vec<Unchecked<i32>>>",
            "Checked<Option<Unchecked<i32>>>",
        ] {
            let err = peel_error(nested);
            assert!(err.contains("outermost"), "{nested}: {err}");
        }
        assert!(peel_error("Option<Checked<i32>>").contains("put `Checked` outermost"));
    }

    /// A parameter marker in return position is an error, also inside a
    /// visibility marker; in argument position it is the parameter's to peel.
    #[test]
    fn param_markers_are_rejected_in_return_position() {
        for ret in [
            "Checked<i32>",
            "miniextendr_api::Unchecked<Vec<f64>>",
            "Invisible<Checked<i32>>",
        ] {
            let err = visibility_marker_error(&ty(ret), "return")
                .expect("rejected")
                .to_string();
            assert!(err.contains("marks a parameter only"), "{ret}: {err}");
        }
        assert!(visibility_marker_error(&ty("Checked<i32>"), "argument").is_none());
    }

    #[test]
    fn type_display_uses_source_spacing() {
        let cases = [
            ("i32", "i32"),
            ("AsFromStrVec<i32>", "AsFromStrVec<i32>"),
            ("Either<i32, String>", "Either<i32, String>"),
            ("&[f64]", "&[f64]"),
            ("&mut [f64]", "&mut [f64]"),
            ("&str", "&str"),
            ("&'a str", "&'a str"),
            ("&'static [u8]", "&'static [u8]"),
            ("Option<Vec<String>>", "Option<Vec<String>>"),
            (
                "HashMap<String, Vec<(i32, f64)>>",
                "HashMap<String, Vec<(i32, f64)>>",
            ),
            ("::std::vec::Vec<i32>", "::std::vec::Vec<i32>"),
            ("[u8; 4]", "[u8; 4]"),
            ("Box<[Mode]>", "Box<[Mode]>"),
            ("()", "()"),
            ("*const T", "*const T"),
            ("RCow<'a, i32>", "RCow<'a, i32>"),
            (
                "Box<dyn Fn(i32) -> i32 + Send + 'static>",
                "Box<dyn Fn(i32) -> i32 + Send + 'static>",
            ),
            ("impl Iterator<Item = i32>", "impl Iterator<Item = i32>"),
            ("<T as Trait>::Assoc", "<T as Trait>::Assoc"),
            ("for<'a> fn(&'a str) -> bool", "for<'a> fn(&'a str) -> bool"),
        ];
        for (src, want) in cases {
            assert_eq!(type_display(&ty(src)), want, "rendering `{src}`");
        }
    }

    #[test]
    fn r_value_noun_names_the_r_side() {
        let cases = [
            ("DataFrame", "a data frame"),
            ("miniextendr_api::DataFrame", "a data frame"),
            ("List", "a list"),
            ("HashMap<String, i32>", "a list"),
            ("String", "a string"),
            ("&str", "a string"),
            ("f64", "a number"),
            ("i32", "an integer"),
            ("bool", "TRUE or FALSE"),
            ("Vec<f64>", "a numeric vector"),
            ("&[i32]", "an integer vector"),
            ("Vec<Option<String>>", "a character vector"),
            ("SEXP", "any other R value"),
            ("ExternalPtr<Model>", "an external pointer"),
            ("Model", "a `Model`"),
            ("Vec<Model>", "a `Vec<Model>`"),
        ];
        for (src, want) in cases {
            assert_eq!(r_value_noun(&ty(src)), want, "noun for `{src}`");
        }
    }

    #[test]
    fn choice_layers_peel_missing_option_either() {
        let layers = |src: &str| {
            let t = ty(src);
            let l = choice_layers(&t);
            (
                l.missing,
                l.nullable,
                l.either_right.map(type_display),
                type_display(l.value),
            )
        };
        assert_eq!(layers("Mode"), (false, false, None, "Mode".to_string()));
        assert_eq!(
            layers("Missing<Option<Either<Mode, DataFrame>>>"),
            (
                true,
                true,
                Some("DataFrame".to_string()),
                "Mode".to_string()
            )
        );
        assert_eq!(
            layers("Either<Mode, Option<f64>>"),
            (
                false,
                false,
                Some("Option<f64>".to_string()),
                "Mode".to_string()
            )
        );
        // A layer out of order stays in the value, where the classifier
        // rejects it.
        assert_eq!(
            layers("Option<Missing<Mode>>"),
            (false, true, None, "Missing<Mode>".to_string())
        );
        assert_eq!(choice_layer_name(&ty("Missing<Mode>")), Some("Missing"));
        assert_eq!(choice_layer_name(&ty("Either<Mode, f64>")), Some("Either"));
        assert_eq!(choice_layer_name(&ty("Either<Mode>")), None);
        assert_eq!(
            type_display(match_arg_choices_ty(&ty("Missing<Vec<Mode>>"), true)),
            "Mode"
        );
        assert_eq!(
            type_display(match_arg_choices_ty(
                &ty("Option<Either<Mode, List>>"),
                false
            )),
            "Mode"
        );
        // A `several_ok` list with another kind of value (#1612).
        assert_eq!(
            layers("Either<Vec<Mode>, DataFrame>"),
            (
                false,
                false,
                Some("DataFrame".to_string()),
                "Vec<Mode>".to_string()
            )
        );
        assert_eq!(
            type_display(match_arg_choices_ty(
                &ty("Missing<Either<Vec<Mode>, DataFrame>>"),
                true
            )),
            "Mode"
        );
    }

    #[test]
    fn erase_lifetimes_replaces_every_lifetime() {
        let erased = |s: &str| type_display(&erase_lifetimes(&ty(s)));
        assert_eq!(erased("&'a [f64]"), "&'_ [f64]");
        assert_eq!(erased("&'static str"), "&'_ str");
        assert_eq!(erased("&mut [i32]"), "&mut [i32]");
        assert_eq!(erased("RCow<'a, Vec<&'b str>>"), "RCow<'_, Vec<&'_ str>>");
        assert_eq!(erased("&str"), "&str");
    }
}

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
