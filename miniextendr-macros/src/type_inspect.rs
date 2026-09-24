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
/// (`Invisible<Visible<T>>`) or a marker in argument position.
pub(crate) fn visibility_marker_error(ty: &syn::Type, what: &str) -> Option<syn::Error> {
    let (marker, inner) = peel_visibility_marker(ty);
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
/// from its hidden call slot. Whichever attribution `None` would spell has no
/// marker, so this never returns `CallAttribution::None`.
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
        call_marker, erase_lifetimes, is_main_thread_bound_input, is_main_thread_bound_return,
        type_display,
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

/// Resolve the `MatchArg`-bound type behind a `match_arg` parameter.
///
/// A `several_ok` parameter is a container (`Vec<T>`, `Box<[T]>`, `[T; N]`,
/// `&[T]`), so the element type is the one carrying `CHOICES`. A scalar
/// parameter may be `Option<T>` (the optional form, #1473), in which case `T`
/// is. Anything else is returned unchanged, and the `MatchArg` bound on the
/// generated code reports the mistake.
///
/// Shared by the standalone-fn path (`lib.rs`) and the impl-method path
/// (`miniextendr_impl.rs`) so the two cannot resolve the type differently.
pub(crate) fn match_arg_choices_ty(param_ty: &syn::Type, several_ok: bool) -> &syn::Type {
    if several_ok {
        classify_several_ok_container(param_ty)
            .map(|(_, inner)| inner)
            .unwrap_or(param_ty)
    } else {
        option_inner_type(param_ty).unwrap_or(param_ty)
    }
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
