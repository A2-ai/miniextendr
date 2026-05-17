//! MXL120: vctrs constructor returns `Self` / named type, or impl has an instance-method receiver.
//!
//! A `#[miniextendr(vctrs(...))]` impl has two hard invariants:
//!
//! 1. **Constructor return type** — the constructor (`fn new` or a method tagged
//!    `#[miniextendr(constructor)]`) must NOT return `Self`, `&Self`, `&mut Self`,
//!    `Box<Self>`, the named impl type, `Result<Self, _>`, or `Result<NamedType, _>`.
//!    The generated R wrapper passes the return value to `vctrs::new_vctr()` /
//!    `new_rcrd()` / `new_list_of()`, which require a plain vector payload, not an
//!    `ExternalPtr` (`EXTPTRSXP`).
//!
//! 2. **Instance receivers** — no method may carry any form of `self` receiver
//!    (`&self`, `&mut self`, `self`, `self: &ExternalPtr<Self>`, etc.).
//!    A vctrs object is an S3-classed base vector; there is no Rust `Self` stored
//!    inside the SEXP.  The C wrapper cannot reconstruct `Self` from a base vector,
//!    so calling an instance method would panic at runtime.
//!
//! ## Mirror
//!
//! The same checks fire as proc-macro hard errors in
//! `miniextendr-macros/src/miniextendr_impl.rs` (search `MXL120`).
//! This lint is defence-in-depth: it catches the mistake during the
//! build-time static-analysis pass (when the macro isn't being expanded,
//! e.g. lint-only IDE runs, third-party tooling).
//! Keep both implementations in sync: if the macro relaxes either check,
//! update this rule too.

use crate::crate_index::{CrateIndex, MethodReceiverKind};
use crate::diagnostic::Diagnostic;
use crate::lint_code::LintCode;

pub fn check(index: &CrateIndex, diagnostics: &mut Vec<Diagnostic>) {
    for (path, data) in &index.file_data {
        for (impl_type, methods) in &data.impl_methods {
            for entry in methods {
                if entry.class_system != "vctrs" {
                    continue;
                }

                // Check 1: constructor return type.
                //
                // A method is a vctrs constructor when it either:
                //   (a) is named `new` with no instance receiver, OR
                //   (b) carries `#[miniextendr(constructor)]`.
                //
                // Note: a method with an instance receiver named `new` is already
                // caught by Check 2 below, so we restrict the `new`-heuristic to
                // static methods only (receiver == None), matching the macro's
                // `method.env != ReceiverKind::Ref && method.env != ReceiverKind::RefMut`
                // guard in `miniextendr_impl.rs:2209-2212`.
                let is_ctor = (entry.method_name == "new"
                    && entry.receiver_kind == MethodReceiverKind::None)
                    || entry.has_constructor_attr;

                if is_ctor && return_type_is_self_or_named(&entry.return_type_str, impl_type) {
                    diagnostics.push(
                        Diagnostic::new(
                            LintCode::MXL120,
                            path,
                            entry.line,
                            format!(
                                "[MXL120] vctrs constructor `{}` must not return `Self` or `{}`.\n\
                                 \n\
                                 The generated R wrapper passes the constructor result to \
                                 `vctrs::new_vctr()` (or `new_rcrd`/`new_list_of`), which requires a \
                                 plain vector payload — not an ExternalPtr (`EXTPTRSXP`).\n\
                                 \n\
                                 Fix: return the vector payload directly instead of `Self`.\n\
                                 For example, return `Vec<f64>` (for vctr), a \
                                 `std::collections::HashMap` / named-list struct (for rcrd), or a \
                                 `Vec<Vec<T>>` (for list_of).",
                                entry.method_name, impl_type,
                            ),
                        )
                        .with_help(
                            "vctrs objects are S3-classed base vectors — there is no ExternalPtr \
                             to return; return the plain vector payload instead",
                        ),
                    );
                }

                // Check 2: instance receivers are not supported on vctrs impls.
                if entry.receiver_kind.is_instance() {
                    diagnostics.push(
                        Diagnostic::new(
                            LintCode::MXL120,
                            path,
                            entry.line,
                            format!(
                                "[MXL120] vctrs impl method `{}` uses a `{}` receiver, which is \
                                 not supported on `#[miniextendr(vctrs(...))]` impls.\n\
                                 \n\
                                 A vctrs object is an S3-classed base vector (REALSXP, INTSXP, \
                                 etc.). There is no Rust `Self` stored inside the R SEXP — the \
                                 vector payload IS the R object. The C wrapper cannot reconstruct \
                                 `Self` from a base vector, so calling an instance method would \
                                 panic at runtime.\n\
                                 \n\
                                 Fix: convert this method to a static method whose parameters \
                                 receive the vector data directly.",
                                entry.method_name,
                                entry.receiver_kind.spelling(),
                            ),
                        )
                        .with_help(
                            "convert instance methods to static methods; pass the vector data \
                             (e.g. `Vec<f64>`) as an explicit parameter instead of `self`",
                        ),
                    );
                }
            }
        }
    }
}

// region: Return-type predicate
//
// Mirror of `vctrs_ctor_returns_self_or_type` + `ty_is_self_or_named` in
// `miniextendr-macros/src/miniextendr_impl.rs`.
// Intentionally duplicated (not shared) — the lint crate must remain independent
// of the macros crate at the type level.  When the macro changes the predicate,
// update both.

/// Returns true if `return_type_str` represents `Self`, `&Self`, `&mut Self`,
/// `Box<Self>`, the named type, `Result<Self, _>`, or `Result<NamedType, _>`.
///
/// `return_type_str` is the stringified token form produced by
/// `quote::ToTokens::to_token_stream()` on a `syn::ReturnType::Type`.
/// An empty string means the return type is `()` (no explicit return) and
/// always returns `false`.
fn return_type_is_self_or_named(return_type_str: &str, type_name: &str) -> bool {
    if return_type_str.is_empty() {
        return false;
    }
    // Re-parse the stored token string.  If parsing fails (shouldn't happen for
    // well-formed Rust), conservatively return false so we don't emit spurious errors.
    let Ok(ty) = syn::parse_str::<syn::Type>(return_type_str) else {
        return false;
    };
    ty_is_self_or_named(&ty, type_name)
}

/// Recursively checks whether `ty` is `Self`, `&Self`, `&mut Self`, `Box<Self>`,
/// the named type, or `Result<(Self | NamedType), _>`.
fn ty_is_self_or_named(ty: &syn::Type, type_name: &str) -> bool {
    match ty {
        syn::Type::Path(p) => {
            let Some(last) = p.path.segments.last() else {
                return false;
            };
            // Plain `Self` or `TypeName`
            if last.ident == "Self" || last.ident == type_name {
                return true;
            }
            // `Result<Self, _>` or `Result<TypeName, _>`
            if last.ident == "Result"
                && let syn::PathArguments::AngleBracketed(ref args) = last.arguments
                && let Some(syn::GenericArgument::Type(first_ty)) = args.args.first()
            {
                return ty_is_self_or_named(first_ty, type_name);
            }
            // `Box<Self>` or `Box<TypeName>`
            if last.ident == "Box"
                && let syn::PathArguments::AngleBracketed(ref args) = last.arguments
                && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
            {
                return ty_is_self_or_named(inner, type_name);
            }
            false
        }
        // `&Self` or `&mut Self`
        syn::Type::Reference(r) => ty_is_self_or_named(r.elem.as_ref(), type_name),
        _ => false,
    }
}

// endregion
