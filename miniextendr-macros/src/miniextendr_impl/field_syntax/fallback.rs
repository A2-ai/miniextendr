//! Exact-name fallback callbacks for field readers (#2150).
//!
//! Like computed fields, these borrow the live struct through the sidecar
//! accessor path. NULL as the callback's name requests completion. A lookup
//! returns a protected one-element list for Some, NULL for None, so every
//! payload (including NULL, a missing argument or a condition) stays distinct
//! from absence and from the wrapper's error transport.
use super::{Helpers, ParsedImpl, guard};
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, Clone)]
pub(crate) struct Fallback {
    pub get: syn::Path,
    pub names: Option<syn::Path>,
}

fn c_ident(parsed: &ParsedImpl) -> syn::Ident {
    crate::naming::field_fallback_c_ident(&parsed.original_impl, &parsed.type_ident)
}

/// Computed getters and fallback paths are emitted outside the impl.
fn resolve_path(path: &syn::Path, parsed: &ParsedImpl) -> syn::Path {
    let mut path = path.clone();
    if path.segments[0].ident == "Self" {
        let syn::Type::Path(target) = parsed.original_impl.self_ty.as_ref() else {
            unreachable!("ParsedImpl requires a named type");
        };
        path.leading_colon = target.path.leading_colon;
        path.segments = target
            .path
            .segments
            .iter()
            .cloned()
            .chain(path.segments.into_iter().skip(1))
            .collect();
    }
    path
}

pub(crate) fn fallback_wrapper(parsed: &ParsedImpl) -> TokenStream {
    let Some(fallback) = &parsed.field_fallback else {
        return quote!();
    };
    let ty = &parsed.original_impl.self_ty;
    let get = resolve_path(&fallback.get, parsed);
    let names = if let Some(path) = &fallback.names {
        let path = resolve_path(path, parsed);
        quote! {
            let data = unsafe { ::miniextendr_api::externalptr::sidecar_r_struct::<#ty>(x) };
            let names: ::std::vec::Vec<::std::string::String> = #path(&data);
            ::miniextendr_api::IntoR::into_sexp(names)
        }
    } else {
        quote!(::miniextendr_api::IntoR::into_sexp(::std::vec::Vec::<
            ::std::string::String,
        >::new()))
    };
    let c_ident = c_ident(parsed);
    let cfg_attrs = &parsed.cfg_attrs;
    let def = quote::format_ident!("__MX_CALL_DEF_{c_ident}");
    let registration = crate::call_method_def::call_method_def(&def, &c_ident, 2);
    quote! {
        #(#cfg_attrs)*
        #[doc(hidden)]
        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        pub unsafe extern "C-unwind" fn #c_ident(
            x: ::miniextendr_api::SEXP,
            name: ::miniextendr_api::SEXP,
        ) -> ::miniextendr_api::SEXP {
            let mark = ::miniextendr_api::deferred_condition::mark();
            let value = ::miniextendr_api::unwind_protect::with_r_unwind_protect(|| {
                if name.is_nil() {
                    if !unsafe { ::miniextendr_api::externalptr::sidecar_r_live::<#ty>(x, false) } {
                        return ::miniextendr_api::SEXP::nil();
                    }
                    #names
                } else {
                    // Borrow first: a restored value is refused before any user code.
                    let data = unsafe { ::miniextendr_api::externalptr::sidecar_r_struct::<#ty>(x) };
                    let roots = unsafe { ::miniextendr_api::ProtectScope::new() };
                    let name = <&str as ::miniextendr_api::TryFromRoot<'_>>::try_from_root(unsafe {
                        ::miniextendr_api::ProtectScope::rooted(&roots, name)
                    }).expect("field fallback requires one non-missing name");
                    // Allocate the protected carrier before the user callback:
                    // it may return a freshly allocated, unrooted SEXP.
                    let result = unsafe { ::miniextendr_api::list::ListBuilder::new(&roots, 1) };
                    match #get(&data, name) {
                        ::core::option::Option::Some(value) => unsafe {
                            ::miniextendr_api::list::ListBuilder::set(
                                &result, 0, ::miniextendr_api::IntoR::into_sexp(value)
                            );
                            ::miniextendr_api::list::ListBuilder::into_sexp(result)
                        },
                        ::core::option::Option::None => ::miniextendr_api::SEXP::nil(),
                    }
                }
            }, ::core::option::Option::None);
            unsafe { ::miniextendr_api::deferred_condition::finish(
                mark, value, ::core::option::Option::None
            ) }
        }
        #(#cfg_attrs)*
        #registration
    }
}

/// Called only for an unknown, non-missing scalar name. Return immediately
/// for Some, including NULL; None goes through the existing custom error/hint.
pub(super) fn read(parsed: &ParsedImpl, receiver: &str, name: &str, indent: &str) -> Vec<String> {
    let h = Helpers::new(parsed);
    let c = c_ident(parsed);
    let class = crate::naming::r_string_escape(&parsed.class_name());
    let mut lines = vec![format!("{indent}.val <- .Call({c}, {receiver}, {name})")];
    lines.extend(guard(indent));
    lines.push(format!(
        "{indent}if (is.null(.val)) {}({name}, environment(), \"{class}\")",
        h.no_field
    ));
    lines.push(format!("{indent}return(.val[[1L]])"));
    lines
}

/// The native callback checks liveness even without restored(every_read).
/// NULL means no live struct; character(0) means a live struct with no names.
pub(super) fn completion(parsed: &ParsedImpl, receiver: &str, extra: &str) -> Vec<String> {
    let h = Helpers::new(parsed);
    let c = c_ident(parsed);
    let mut lines = vec![format!("  .val <- .Call({c}, {receiver}, NULL)")];
    lines.extend(guard("  "));
    lines.push("  if (is.null(.val)) return(character(0))".to_string());
    lines.push(format!(
        "  grep(pattern, c({}, .val{extra}), value = TRUE)",
        h.fields
    ));
    lines
}

/// S4 and env classes with a fallback expose the same positional and
/// field-only collection readers as S3. Env's name dispatch still gives
/// its registered methods priority over a fallback.
pub(crate) fn additional_readers(parsed: &ParsedImpl, on_page: bool, export: bool) -> Vec<String> {
    use super::{ClassSystem, page_lines, s3_method_doc};
    let h = Helpers::new(parsed);
    let class = parsed.class_name();
    let class_lit = crate::naming::r_string_escape(&class);
    let s4 = parsed.class_system == ClassSystem::S4;
    let receiver = if s4 { "x@ptr" } else { "x" };
    let mut lines = Vec::new();
    let open = |lines: &mut Vec<String>, generic: &str, args: &str, params: &[(&str, &str)]| {
        if s4 {
            if on_page {
                lines.extend(page_lines(parsed, params));
            } else {
                lines.push("#' @noRd".to_string());
            }
            if export {
                lines.push(format!("#' @exportMethod {generic}"));
            }
            lines.push(format!(
                "methods::setMethod(\"{generic}\", \"{class}\", function({args}) {{"
            ));
        } else {
            s3_method_doc(lines, parsed, generic, on_page, export, params);
            let name = crate::naming::r_def_name(&format!("{generic}.{class}"));
            lines.push(format!("{name} <- function({args}) {{"));
        }
    };
    let close = |lines: &mut Vec<String>| {
        lines.push(if s4 { "})" } else { "}" }.to_string());
        lines.push(String::new());
    };
    let x = ("x", "An object.");
    let dots = ("...", "Additional arguments.");
    let i = (
        "i",
        "An exact field or fallback name, or a field position; assignment accepts field names only.",
    );
    let index_error = format!(
        ".miniextendr_field_index_error(\"{class_lit}\", environment(), {}, strict = TRUE)",
        h.error_class
    );
    if s4 {
        open(
            &mut lines,
            "[[",
            "x, i, j, ...",
            &[x, i, ("j", "Unused; only one index is accepted."), dots],
        );
        lines.extend(h.live_check(receiver, "  "));
        lines.push(format!("  if (!missing(j)) {index_error}"));
    } else {
        open(
            &mut lines,
            "[[",
            "self, name, ...",
            &[
                ("self", "The object instance."),
                (
                    "name",
                    "An exact field, method or fallback name, or a field position.",
                ),
                dots,
            ],
        );
        lines.extend(h.live_check("self", "  "));
    }
    let index = if s4 { "i" } else { "name" };
    lines.push(format!("  if (is.numeric({index}) && length({index}) == 1L && !is.na({index}) && {index} == trunc({index})) {{"));
    lines.push(format!(
        "    {index} <- .miniextendr_field_at({}, {index}, environment(), {})",
        h.fields, h.error_class
    ));
    lines.push(format!("  }} else if (!is.character({index}) || length({index}) != 1L || is.na({index})) {index_error}"));
    if s4 {
        lines.push(format!("  if (is.na(match(i, {}))) {{", h.fields));
        lines.extend(read(parsed, receiver, "i", "    "));
        lines.push("  }".to_string());
        lines.push(format!("  .val <- {}({receiver}, i)", h.get));
        lines.extend(guard("  "));
        lines.push("  .val".to_string());
    } else {
        lines.extend(super::super::env_class::reader_body(parsed));
    }
    close(&mut lines);

    open(&mut lines, "names", "x", &[x]);
    lines.extend(h.live_check(receiver, "  "));
    if !s4 {
        lines.push("  if (typeof(x) != \"externalptr\") return(NextMethod())".to_string());
    }
    lines.push(format!("  {}", h.fields));
    close(&mut lines);

    open(&mut lines, "as.list", "x, ...", &[x, dots]);
    lines.extend(h.live_check(receiver, "  "));
    if !s4 {
        lines.push("  if (is.environment(x)) return(as.list.environment(x, ...))".to_string());
        lines.push("  if (typeof(x) != \"externalptr\") return(NextMethod())".to_string());
    }
    lines.push(format!("  .out <- vector(\"list\", length({}))", h.fields));
    lines.push(format!("  names(.out) <- {}", h.fields));
    lines.push(format!("  for (.k in seq_along({})) {{", h.fields));
    lines.push(format!(
        "    .val <- {}({receiver}, {}[[.k]])",
        h.get, h.fields
    ));
    lines.extend(guard("    "));
    lines.push("    .out[.k] <- list(.val)".to_string());
    lines.push("  }".to_string());
    lines.push("  .out".to_string());
    close(&mut lines);
    if s4 && parsed.r_data_accessors.setters() {
        open(
            &mut lines,
            "[[<-",
            "x, i, j, ..., value",
            &[
                x,
                i,
                ("j", "Unused; only one index is accepted."),
                dots,
                ("value", "The new value of the field."),
            ],
        );
        lines.push(format!(
            "  if (!missing(j) || !is.character(i) || length(i) != 1L || is.na(i)) {index_error}"
        ));
        lines.push(format!(
            "  if (is.na(match(i, {}))) {}(i, environment(), \"{class_lit}\")",
            h.fields, h.no_field
        ));
        lines.push(format!("  .val <- {}({receiver}, i, value)", h.set));
        lines.extend(guard("  "));
        lines.push("  x".to_string());
        close(&mut lines);
    }
    lines
}
