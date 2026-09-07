//! Shared utilities for converting R SEXP parameters to Rust types.
//!
//! This module provides a builder for generating Rust conversion code from R SEXP arguments,
//! ensuring consistent behavior across standalone functions and impl methods.

use crate::miniextendr_fn::CoercionMapping;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

/// Builder for generating Rust conversion statements from R SEXP parameters.
///
/// Handles:
/// - Unit types `()` → identity binding
/// - `&Dots` → special wrapper with storage
/// - Slices `&[T]` → TryFromSexp
/// - `&str` → String + Borrow (for worker thread compatibility)
/// - Scalar references → DATAPTR_RO_unchecked
/// - Coercion → multi-source numeric conversion (incl. widened native `i32`/`f64`) or logical/integer bool conversion
/// - Default → TryFromSexp
pub struct RustConversionBuilder {
    /// Wrapper parameter holding the R call, with the declaration's syntax context.
    /// Interpolate it into spanned conversions instead of inheriting user-type hygiene.
    call_context_ident: syn::Ident,
    /// Enable coercion for all parameters
    coerce_all: bool,
    /// Parameter names that should use coercion
    coerce_params: Vec<String>,
    /// Enable strict input conversion for lossy types
    strict: bool,
    /// Parameter names with `match_arg + several_ok` — use `match_arg_vec_from_sexp` instead of `TryFromSexp`.
    match_arg_several_ok_params: Vec<String>,
    /// Choice parameters whose type wraps the choice in `Missing` / `Option`
    /// layers (#1473, #1551), with the kind of their innermost value. Decoded
    /// by [`layered_choice_expr`] instead of `TryFromSexp`.
    layered_choice_params: Vec<(String, ChoiceLeaf)>,
    /// The crate's `conversion_error_class` (`[package.metadata.miniextendr]`),
    /// appended to every conversion condition's class vector.
    conversion_error_class: Vec<String>,
}

impl RustConversionBuilder {
    /// Create a conversion builder using the wrapper's call-context parameter,
    /// carrying the crate-level `conversion_error_class` from the manifest
    /// (empty when unset).
    pub fn new(call_context_ident: syn::Ident) -> Self {
        Self {
            call_context_ident,
            coerce_all: false,
            coerce_params: Vec::new(),
            strict: false,
            match_arg_several_ok_params: Vec::new(),
            layered_choice_params: Vec::new(),
            conversion_error_class: crate::crate_config::conversion_error_class(),
        }
    }

    /// Override the crate-level conversion-error classes (the manifest's
    /// `conversion_error_class` is read by [`Self::new`]).
    #[cfg(test)]
    pub fn with_conversion_error_class(mut self, classes: Vec<String>) -> Self {
        self.conversion_error_class = classes;
        self
    }

    /// Enable coercion for all parameters.
    pub fn with_coerce_all(mut self) -> Self {
        self.coerce_all = true;
        self
    }

    /// Add a single parameter name that should use coercion.
    ///
    /// `param_name` is matched against the identifier in the function signature.
    /// Can be called multiple times to add several parameters.
    pub fn with_coerce_param(mut self, param_name: String) -> Self {
        self.coerce_params.push(param_name);
        self
    }

    /// Enable strict input conversion for lossy types (i64/u64/isize/usize + Vec variants).
    pub fn with_strict(mut self) -> Self {
        self.strict = true;
        self
    }

    /// Mark a parameter as `match_arg + several_ok` — uses `match_arg_vec_from_sexp`
    /// instead of `TryFromSexp` for converting STRSXP → `Vec<EnumType>`.
    pub fn with_match_arg_several_ok(mut self, param_name: String) -> Self {
        self.match_arg_several_ok_params.push(param_name);
        self
    }

    /// Mark a choice parameter whose type wraps the choice in `Missing` /
    /// `Option` layers (#1473, #1551): it is decoded layer by layer with the
    /// `miniextendr_api::match_arg_*` helpers (see [`layered_choice_expr`])
    /// instead of `TryFromSexp`. There is no `TryFromSexp for Option<T>` a
    /// downstream crate could provide for its own enum (orphan rule), and a
    /// `T: MatchArg` blanket would collide with the newtype blanket in
    /// `miniextendr_api::newtype`, so the wrapper calls the helpers directly.
    pub fn with_layered_choice(mut self, param_name: String, leaf: ChoiceLeaf) -> Self {
        self.layered_choice_params.push((param_name, leaf));
        self
    }

    /// Check if a parameter should use coercion.
    ///
    /// Returns `true` if `coerce_all` is set or `param_name` appears in the per-parameter list.
    fn should_coerce(&self, param_name: &str) -> bool {
        self.coerce_all || self.coerce_params.contains(&param_name.to_string())
    }

    /// Generate a conversion expression that returns a tagged condition SEXP on failure.
    ///
    /// The R wrapper inspects `.val` and raises a structured `rust_*` condition; the
    /// `return` happens from inside the C wrapper body before any further conversion.
    ///
    /// - `try_expr`: The `Result<T, E>`-producing expression
    /// - `ctx`: The argument's R-facing failure context ([`ArgContext`]): the
    ///   message prefix, `e$param` and `e$rust_type`
    /// - `ident`: The binding name for the converted value
    /// - `ty`: The target Rust type (for the `let` binding). The annotation is
    ///   load-bearing: the `Err` arm probes the error type by method call
    ///   ([`conversion_err_arm`]), which needs that type known at that point.
    /// - `span`: Source span for error reporting
    fn conversion_stmt(
        &self,
        try_expr: TokenStream,
        ctx: &ArgContext,
        ident: &syn::Ident,
        ty: &syn::Type,
        span: proc_macro2::Span,
    ) -> TokenStream {
        let err_arm = conversion_err_arm(
            ctx,
            &self.conversion_error_class,
            &self.call_context_ident,
            span,
        );
        quote_spanned! {span=>
            let #ident: #ty = match #try_expr {
                Ok(v) => v,
                #err_arm
            };
        }
    }

    /// Generate conversion statement for a single parameter.
    ///
    /// This is the non-split variant: owned conversions and borrow statements are
    /// concatenated into a single list, suitable for main-thread execution where
    /// everything runs in the same scope.
    ///
    /// - `pat_type`: the typed pattern from the function signature (e.g., `x: i32`).
    /// - `sexp_ident`: the identifier of the raw SEXP variable holding the R argument.
    ///
    /// Returns a flat list of `let` binding statements that convert `sexp_ident` into
    /// the Rust type declared in `pat_type`.
    pub fn build_conversion(
        &self,
        pat_type: &syn::PatType,
        sexp_ident: &syn::Ident,
    ) -> Vec<TokenStream> {
        // `build_conversion` is the single-scope (main-thread) path: every statement
        // runs inside the same `with_r_unwind_protect` closure where the argument
        // SEXPs are live for the whole call. So `&str` can borrow R's CHARSXP pool
        // directly — zero-copy — exactly like `&[T]` already does. The owning-`String`
        // detour only exists to satisfy `Send` when the value must cross the worker
        // boundary (`build_conversion_split`), which never applies here.
        let (owned, borrowed) = self.build_conversion_split_inner(pat_type, sexp_ident, true);
        owned.into_iter().chain(borrowed).collect()
    }

    /// Generate conversion statements split into two phases for worker thread execution.
    ///
    /// For reference types like `&str`, we need to:
    /// 1. Convert SEXP to owned type (String) -- runs on the main thread before the
    ///    worker closure, so the owned value can be moved into the closure.
    /// 2. Borrow from the owned type (`&str`) -- runs inside the worker closure.
    ///
    /// For non-reference types (scalars, `Vec`, etc.) everything goes into the first
    /// phase and the second vec is empty.
    ///
    /// - `pat_type`: the typed pattern from the function signature (e.g., `s: &str`).
    /// - `sexp_ident`: the identifier of the raw SEXP variable holding the R argument.
    ///
    /// Returns `(owned_conversions, borrow_statements)` where each element is a list
    /// of `let` binding token streams.
    pub fn build_conversion_split(
        &self,
        pat_type: &syn::PatType,
        sexp_ident: &syn::Ident,
    ) -> (Vec<TokenStream>, Vec<TokenStream>) {
        // Worker path: `&str` MUST be owned-then-borrowed because a borrowed view
        // over R's CHARSXP pool is `!Send` and cannot move into the worker closure.
        self.build_conversion_split_inner(pat_type, sexp_ident, false)
    }

    /// Inner implementation of [`Self::build_conversion_split`].
    ///
    /// `zero_copy_str` controls how a `&str` argument is lowered:
    /// - `true` (main-thread, single-scope): emit a direct zero-copy `&str` borrow
    ///   over R's CHARSXP pool via `TryFromSexp` — no `String` allocation. Sound
    ///   because the SEXP outlives the borrow inside the `with_r_unwind_protect`
    ///   closure, and the `&str`'s lifetime is tied to that scope (storing it beyond
    ///   the call is a borrow-checker error).
    /// - `false` (worker): convert to an owned `String` on the main thread, then
    ///   borrow `&str` inside the worker closure — the `String` is `Send`, the
    ///   borrowed view is not.
    fn build_conversion_split_inner(
        &self,
        pat_type: &syn::PatType,
        sexp_ident: &syn::Ident,
        zero_copy_str: bool,
    ) -> (Vec<TokenStream>, Vec<TokenStream>) {
        let syn::Pat::Ident(pat_ident) = pat_type.pat.as_ref() else {
            return (vec![], vec![]);
        };
        let ident = &pat_ident.ident;
        // `r#`-free spelling for synthesized bindings (`__storage_where`).
        let plain = crate::naming::unraw(ident);
        let ty = pat_type.ty.as_ref();
        // The R formal's name (`_x` → `x`, `r#type` → `type`): what a
        // conversion condition names in its message and in `e$param`.
        let r_name =
            crate::r_wrapper_builder::normalize_r_arg_string(&crate::naming::ident_name(ident));
        // What a conversion failure of this argument says (#1591): the R-facing
        // expectation (coerce-widened like the R-side check), `e$param` and
        // `e$rust_type`.
        let ctx = ArgContext::new(
            &r_name,
            ty,
            self.should_coerce(&crate::naming::ident_name(ident)),
        );

        // A `Call` / `CallerCall` marker (#1566) never reaches this builder from
        // a standalone fn: `lib.rs` removes it from the inputs and binds it from
        // the call slot at the call site. Seeing one here means a class or trait
        // method took it, where the wrapper's own call is the only attribution.
        if crate::type_inspect::call_marker(ty).is_some() {
            let span = ty.span();
            let call_context_ident = &self.call_context_ident;
            // The binding keeps the method's own use of the parameter from adding
            // a follow-on "cannot find value" to the diagnostic; the wrapper's call
            // slot is in scope in every C wrapper, so it also type-checks.
            let stmt = quote_spanned! {span=>
                ::core::compile_error!(
                    "`Call` / `CallerCall` parameters are supported on standalone `#[miniextendr]` \
                     functions only; class and trait methods attribute conditions to the wrapper's own call"
                );
                let #ident: #ty = <#ty>::from_sexp(#call_context_ident);
            };
            return (vec![stmt], vec![]);
        }

        match ty {
            // Unit type: ()
            // Note: We never generate `mut` on conversion bindings - the user's function
            // has its own parameter binding that will be `mut` if they specified it.
            syn::Type::Tuple(t) if t.elems.is_empty() => {
                let stmt = quote! { let #ident = (); };
                (vec![stmt], vec![])
            }

            // Reference types: &T, &mut T
            syn::Type::Reference(r) => {
                let param_name = crate::naming::ident_name(ident);
                let is_dots = matches!(
                    r.elem.as_ref(),
                    syn::Type::Path(tp)
                        if tp.path.segments.last()
                            .map(|s| s.ident == "Dots")
                            .unwrap_or(false)
                );
                let is_slice = matches!(r.elem.as_ref(), syn::Type::Slice(_));
                let is_str = matches!(
                    r.elem.as_ref(),
                    syn::Type::Path(tp) if tp.path.is_ident("str")
                );

                // &[T] / &mut [T] with match_arg + several_ok:
                // two-phase: pre-call Vec<T>, in-call borrow
                if is_slice
                    && self
                        .match_arg_several_ok_params
                        .contains(&param_name.to_string())
                    && let Some((crate::SeveralOkContainer::BorrowedSlice, inner_ty)) =
                        crate::classify_several_ok_container(ty)
                {
                    let is_mut = r.mutability.is_some();
                    let storage_ident = quote::format_ident!("__storage_{}", plain);
                    let vec_ty: syn::Type = syn::parse_quote!(::std::vec::Vec<#inner_ty>);
                    let span = ty.span();
                    let try_expr = quote_spanned! {span=>
                        ::miniextendr_api::match_arg_vec_from_sexp::<#inner_ty>(#sexp_ident)
                    };
                    // Emit owned Vec<T> binding.
                    // For &mut [T] the storage binding needs `mut`.
                    let owned_stmt = if is_mut {
                        // Need `let mut storage_ident: vec_ty = ...`; inline the mut variant.
                        let err_arm = conversion_err_arm(
                            &ctx,
                            &self.conversion_error_class,
                            &self.call_context_ident,
                            span,
                        );
                        quote_spanned! {span=>
                            let mut #storage_ident: #vec_ty = match #try_expr {
                                Ok(v) => v,
                                #err_arm
                            };
                        }
                    } else {
                        self.conversion_stmt(try_expr, &ctx, &storage_ident, &vec_ty, span)
                    };
                    let borrow_stmt = if is_mut {
                        quote_spanned! {span=>
                            let #ident: #ty = &mut #storage_ident;
                        }
                    } else {
                        quote_spanned! {span=>
                            let #ident: #ty = &#storage_ident;
                        }
                    };
                    return (vec![owned_stmt], vec![borrow_stmt]);
                }

                if is_dots {
                    // &Dots: create wrapper with storage (main thread only - requires SEXP)
                    let storage_ident = quote::format_ident!("{}_storage", plain);
                    let stmt = quote! {
                        let #storage_ident = ::miniextendr_api::dots::Dots { inner: #sexp_ident };
                        let #ident = &#storage_ident;
                    };
                    (vec![stmt], vec![])
                } else if is_slice {
                    // &[T]: use TryFromSexp (backed by DATAPTR_RO). The binding is
                    // annotated with the lifetime-erased type (`&'a [T]` → `&'_ [T]`):
                    // the user's lifetime need not be in scope, and the `Err` arm's
                    // probe needs the error type named.
                    let span = ty.span();
                    let try_expr = quote_spanned! {span=>
                        ::miniextendr_api::TryFromSexp::try_from_sexp(#sexp_ident)
                    };
                    let binding_ty = crate::type_inspect::erase_lifetimes(ty);
                    let stmt = self.conversion_stmt(try_expr, &ctx, ident, &binding_ty, span);
                    (vec![stmt], vec![])
                } else if is_str {
                    let span = ty.span();
                    if zero_copy_str {
                        // Main-thread path: borrow R's CHARSXP pool directly via the
                        // `&'static str` TryFromSexp impl — zero allocation. The SEXP
                        // is a live wrapper argument for the whole call, so the borrow
                        // is sound; its lifetime is tied to this scope, so storing it
                        // beyond the call is a borrow-checker error (no-store guarantee).
                        // The binding carries the lifetime-erased type, as for slices.
                        let try_expr = quote_spanned! {span=>
                            ::miniextendr_api::TryFromSexp::try_from_sexp(#sexp_ident)
                        };
                        let binding_ty = crate::type_inspect::erase_lifetimes(ty);
                        let stmt = self.conversion_stmt(try_expr, &ctx, ident, &binding_ty, span);
                        (vec![stmt], vec![])
                    } else {
                        // Worker path: convert to owned String, then borrow using the
                        // Borrow trait. The String moves into the worker closure (it is
                        // Send); a borrowed view over R's CHARSXP pool is not.
                        let owned_ident = quote::format_ident!("__owned_{}", plain);
                        // Owned conversion: SEXP -> String
                        let string_ty: syn::Type = syn::parse_quote!(String);
                        let try_expr = quote_spanned! {span=>
                            ::miniextendr_api::TryFromSexp::try_from_sexp(#sexp_ident)
                        };
                        let owned_stmt =
                            self.conversion_stmt(try_expr, &ctx, &owned_ident, &string_ty, span);
                        // Borrow: String -> &str (using Borrow trait)
                        let borrow_stmt = quote_spanned! {span=>
                            let #ident: &str = ::std::borrow::Borrow::borrow(&#owned_ident);
                        };
                        (vec![owned_stmt], vec![borrow_stmt])
                    }
                } else {
                    // &T for other types: use TryFromSexp for the reference type.
                    let span = ty.span();
                    let try_expr = quote_spanned! {span=>
                        ::miniextendr_api::TryFromSexp::try_from_sexp(#sexp_ident)
                    };
                    let stmt = self.conversion_stmt(try_expr, &ctx, ident, ty, span);
                    (vec![stmt], vec![])
                }
            }

            // All other types
            _ => {
                let param_name = crate::naming::ident_name(ident);

                // Strict mode: use checked input helpers for lossy types. A
                // rejected input is the same argument error as any other
                // conversion failure (#1594), worded against what strict
                // accepts (`a single whole number`).
                if self.strict
                    && let Some(strict_expr) =
                        crate::return_type_analysis::strict_input_conversion_for_type(
                            ty, sexp_ident,
                        )
                {
                    let span = ty.span();
                    let strict_ctx = ArgContext::strict(&r_name, ty);
                    let stmt = self.conversion_stmt(strict_expr, &strict_ctx, ident, ty, span);
                    return (vec![stmt], vec![]);
                }

                // A choice under `Missing` / `Option` layers (#1473, #1551):
                // decoded layer by layer, outermost first.
                if let Some((_, leaf)) = self
                    .layered_choice_params
                    .iter()
                    .find(|(name, _)| *name == param_name)
                {
                    let span = ty.span();
                    let try_expr = layered_choice_expr(ty, *leaf, sexp_ident, span);
                    let stmt = self.conversion_stmt(try_expr, &ctx, ident, ty, span);
                    return (vec![stmt], vec![]);
                }

                // match_arg + several_ok: use match_arg_vec_from_sexp for container types
                if self
                    .match_arg_several_ok_params
                    .contains(&param_name.to_string())
                    && let Some((container, inner_ty)) = crate::classify_several_ok_container(ty)
                {
                    let span = ty.span();
                    match container {
                        crate::SeveralOkContainer::Vec => {
                            let try_expr = quote_spanned! {span=>
                                ::miniextendr_api::match_arg_vec_from_sexp::<#inner_ty>(#sexp_ident)
                            };
                            let stmt = self.conversion_stmt(try_expr, &ctx, ident, ty, span);
                            return (vec![stmt], vec![]);
                        }
                        crate::SeveralOkContainer::BoxedSlice => {
                            let try_expr = quote_spanned! {span=>
                                ::miniextendr_api::match_arg_vec_from_sexp::<#inner_ty>(#sexp_ident)
                                    .map(|v| v.into_boxed_slice())
                            };
                            let stmt = self.conversion_stmt(try_expr, &ctx, ident, ty, span);
                            return (vec![stmt], vec![]);
                        }
                        crate::SeveralOkContainer::Array(n) => {
                            let span = ty.span();
                            // First extract the Vec via match_arg_vec_from_sexp (handles
                            // match_arg validation + error reporting), then the array
                            // conversion, whose only failure is the selection length:
                            // an argument error like any other (#1591).
                            let vec_ty: syn::Type = syn::parse_quote!(::std::vec::Vec<#inner_ty>);
                            let vec_ident = quote::format_ident!("__vec_{}", plain);
                            let try_expr = quote_spanned! {span=>
                                ::miniextendr_api::match_arg_vec_from_sexp::<#inner_ty>(#sexp_ident)
                            };
                            let vec_stmt =
                                self.conversion_stmt(try_expr, &ctx, &vec_ident, &vec_ty, span);
                            let len_ctx = ArgContext {
                                prefix: format!("'{r_name}' must be of length {n}"),
                                expected_known: true,
                                ..ctx.clone()
                            };
                            let len_arm = conversion_err_arm(
                                &len_ctx,
                                &self.conversion_error_class,
                                &self.call_context_ident,
                                span,
                            );
                            let arr_stmt = quote_spanned! {span=>
                                let #ident: #ty = match <[#inner_ty; #n]>::try_from(#vec_ident)
                                    .map_err(|v| ::std::format!("got length {}", v.len()))
                                {
                                    Ok(v) => v,
                                    #len_arm
                                };
                            };
                            return (vec![vec_stmt, arr_stmt], vec![]);
                        }
                        crate::SeveralOkContainer::BorrowedSlice => {
                            let storage_ident = quote::format_ident!("__storage_{}", plain);
                            let vec_ty: syn::Type = syn::parse_quote!(::std::vec::Vec<#inner_ty>);
                            let try_expr = quote_spanned! {span=>
                                ::miniextendr_api::match_arg_vec_from_sexp::<#inner_ty>(#sexp_ident)
                            };
                            let owned_stmt =
                                self.conversion_stmt(try_expr, &ctx, &storage_ident, &vec_ty, span);
                            let borrow_stmt = quote_spanned! {span=>
                                let #ident: #ty = &#storage_ident;
                            };
                            return (vec![owned_stmt], vec![borrow_stmt]);
                        }
                    }
                }

                let should_coerce = self.should_coerce(&param_name);
                let coercion_mapping = if should_coerce {
                    CoercionMapping::from_type(ty)
                } else {
                    None
                };

                let span = ty.span();
                let stmt = match coercion_mapping {
                    Some(mapping) => {
                        let try_expr = match mapping {
                            CoercionMapping::Numeric => quote_spanned! {span=>
                                ::miniextendr_api::TryFromSexp::try_from_sexp(#sexp_ident)
                            },
                            CoercionMapping::Bool => quote_spanned! {span=>
                                ::miniextendr_api::from_r::try_from_sexp_coerced_bool(#sexp_ident)
                            },
                            CoercionMapping::BoolVec => quote_spanned! {span=>
                                ::miniextendr_api::from_r::try_from_sexp_coerced_bool_vec(#sexp_ident)
                            },
                            CoercionMapping::NativeInt => quote_spanned! {span=>
                                ::miniextendr_api::from_r::try_from_sexp_coerced_i32(#sexp_ident)
                            },
                            CoercionMapping::NativeIntVec => quote_spanned! {span=>
                                ::miniextendr_api::from_r::try_from_sexp_coerced_i32_vec(#sexp_ident)
                            },
                            CoercionMapping::NativeReal => quote_spanned! {span=>
                                ::miniextendr_api::from_r::try_from_sexp_coerced_f64(#sexp_ident)
                            },
                            CoercionMapping::NativeRealVec => quote_spanned! {span=>
                                ::miniextendr_api::from_r::try_from_sexp_coerced_f64_vec(#sexp_ident)
                            },
                        };
                        self.conversion_stmt(try_expr, &ctx, ident, ty, span)
                    }
                    None => {
                        let try_expr = quote_spanned! {span=>
                            ::miniextendr_api::TryFromSexp::try_from_sexp(#sexp_ident)
                        };
                        self.conversion_stmt(try_expr, &ctx, ident, ty, span)
                    }
                };
                (vec![stmt], vec![])
            }
        }
    }

    /// Generate conversion statements for all parameters in a function signature.
    ///
    /// Iterates over `inputs` (the function's parameter list) paired with `sexp_idents`
    /// (the corresponding SEXP variable names), calling [`build_conversion`](Self::build_conversion)
    /// for each typed parameter. Receiver parameters (`self`) are silently skipped.
    ///
    /// Returns a flat list of all conversion statements, in parameter order.
    pub fn build_conversions(
        &self,
        inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
        sexp_idents: &[syn::Ident],
    ) -> Vec<TokenStream> {
        let mut all_statements = Vec::new();

        for (arg, sexp_ident) in inputs.iter().zip(sexp_idents.iter()) {
            if let syn::FnArg::Typed(pat_type) = arg {
                let statements = self.build_conversion(pat_type, sexp_ident);
                all_statements.extend(statements);
            }
        }

        all_statements
    }
}

// region: layered choice parameters

/// Where the innermost value of a layered choice parameter comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChoiceLeaf {
    /// A scalar `match_arg` type (`T: MatchArg`), decoded with
    /// `match_arg_from_sexp::<T>`.
    MatchArg,
    /// A `match_arg` + `several_ok` container (`Vec<T>` / `Box<[T]>`),
    /// decoded with `match_arg_vec_from_sexp::<T>`.
    MatchArgSeveral,
    /// The string type of a `choices(...)` parameter, decoded with its own
    /// `TryFromSexp` (the R prelude already matched it against the list).
    Literal,
}

/// A layer around a choice parameter's value, outermost first.
enum ChoiceLayer<'a> {
    /// `Missing<..>`: the missing-argument sentinel is `Missing::Absent`.
    Missing,
    /// `Option<..>`: `NULL` is `None`.
    Null,
    /// `Either<.., R>`: character or factor input is the choice (`Left`),
    /// anything else converts to `R` (`Right`).
    Either(&'a syn::Type),
}

/// The decoding expression for a choice parameter of type `ty` whose value
/// sits under `Missing` / `Option` / `Either` layers (#1473, #1551), applied
/// to the `SEXP` binding `sexp`. Each layer is one `miniextendr_api::match_arg_*`
/// helper that takes the decoder of the layers below it, so
/// `Missing<Option<Mode>>` becomes
///
/// ```ignore
/// ::miniextendr_api::match_arg_missing_or(sexp, ::miniextendr_api::match_arg_option_from_sexp::<Mode>)
/// ```
///
/// and `Either<Route, DataFrame>` becomes
///
/// ```ignore
/// ::miniextendr_api::match_arg_either_or::<_, DataFrame, _>(sexp, ::miniextendr_api::match_arg_from_sexp::<Route>)
/// ```
fn layered_choice_expr(
    ty: &syn::Type,
    leaf: ChoiceLeaf,
    sexp: &syn::Ident,
    span: proc_macro2::Span,
) -> TokenStream {
    let layers = crate::type_inspect::choice_layers(ty);
    let mut wraps = Vec::new();
    if layers.missing {
        wraps.push(ChoiceLayer::Missing);
    }
    if layers.nullable {
        wraps.push(ChoiceLayer::Null);
    }
    if let Some(right) = layers.either_right {
        wraps.push(ChoiceLayer::Either(right));
    }
    let value = LayeredValue::new(layers.value, leaf);
    let expr = value.apply(&wraps, &quote! { #sexp });
    quote_spanned! {span=> #expr }
}

/// The innermost value of a layered choice parameter.
struct LayeredValue {
    leaf: ChoiceLeaf,
    /// The decoded type: the `MatchArg` type of a scalar, the element type
    /// of a `several_ok` container, the string type of a `choices` value
    /// (lifetimes erased).
    choice_ty: syn::Type,
    /// A `Box<[T]>` container, converted from the decoded `Vec<T>`.
    boxed: bool,
}

impl LayeredValue {
    fn new(value: &syn::Type, leaf: ChoiceLeaf) -> Self {
        let (choice_ty, boxed) = match (leaf, crate::classify_several_ok_container(value)) {
            (ChoiceLeaf::MatchArgSeveral, Some((container, inner))) => (
                inner.clone(),
                matches!(container, crate::SeveralOkContainer::BoxedSlice),
            ),
            _ => (crate::type_inspect::erase_lifetimes(value), false),
        };
        Self {
            leaf,
            choice_ty,
            boxed,
        }
    }

    /// `layers` over this value, decoding the argument `sexp`: the outermost
    /// layer's helper called on `sexp` with the decoder of the rest.
    fn apply(&self, layers: &[ChoiceLayer], sexp: &TokenStream) -> TokenStream {
        let t = &self.choice_ty;
        match layers {
            [] if self.boxed => quote! {
                ::miniextendr_api::match_arg_vec_from_sexp::<#t>(#sexp)
                    .map(::std::vec::Vec::into_boxed_slice)
            },
            [] => {
                let decoder = self.decoder(&[]);
                quote! { #decoder(#sexp) }
            }
            // `Option<T>` over a scalar choice (#1473) keeps its dedicated helper.
            [ChoiceLayer::Null] if self.leaf == ChoiceLeaf::MatchArg => {
                quote! { ::miniextendr_api::match_arg_option_from_sexp::<#t>(#sexp) }
            }
            [first, rest @ ..] => {
                let inner = self.decoder(rest);
                match first {
                    ChoiceLayer::Missing => {
                        quote! { ::miniextendr_api::match_arg_missing_or(#sexp, #inner) }
                    }
                    ChoiceLayer::Null => {
                        quote! { ::miniextendr_api::match_arg_null_or(#sexp, #inner) }
                    }
                    ChoiceLayer::Either(right) => {
                        let right = crate::type_inspect::erase_lifetimes(right);
                        quote! {
                            ::miniextendr_api::match_arg_either_or::<_, #right, _>(#sexp, #inner)
                        }
                    }
                }
            }
        }
    }

    /// `layers` over this value as a decoder (`FnOnce(SEXP) -> Result<_, _>`):
    /// a function path where one exists, a closure otherwise.
    fn decoder(&self, layers: &[ChoiceLayer]) -> TokenStream {
        let t = &self.choice_ty;
        match (layers, self.leaf) {
            ([], ChoiceLeaf::MatchArg) => {
                quote! { ::miniextendr_api::match_arg_from_sexp::<#t> }
            }
            ([], ChoiceLeaf::MatchArgSeveral) if !self.boxed => {
                quote! { ::miniextendr_api::match_arg_vec_from_sexp::<#t> }
            }
            ([], ChoiceLeaf::Literal) => {
                quote! { <#t as ::miniextendr_api::TryFromSexp>::try_from_sexp }
            }
            ([ChoiceLayer::Null], ChoiceLeaf::MatchArg) => {
                quote! { ::miniextendr_api::match_arg_option_from_sexp::<#t> }
            }
            _ => {
                let arg = quote! { __mx_sexp };
                let body = self.apply(layers, &arg);
                quote! { |#arg| #body }
            }
        }
    }
}

// endregion

// region: conversion-failure conditions

/// The R-facing context of one argument's conversion failure (#1591).
#[derive(Clone)]
struct ArgContext {
    /// The message prefix: `'<p>' must be <expected>` when the type has an
    /// R-facing expectation ([`crate::r_preconditions::conversion_expectation`]),
    /// `invalid '<p>' argument` otherwise. The error's reason follows it.
    prefix: String,
    /// Whether `prefix` states the expectation. Passed to the probe
    /// (`__mx_conversion_err_parts!(e, expected_known)`), so a built-in error's
    /// reason does not repeat it (`got character` rather than
    /// `expected integer, got character`).
    expected_known: bool,
    /// The R formal's name, `e$param`.
    r_name: String,
    /// The Rust type as written in the signature, `e$rust_type`: kept for the
    /// package author, out of the user-facing message.
    rust_type: String,
    /// Whether the value may be `NULL` (an `Option<_>`, under any `Missing`):
    /// an expectation the error supplies at run time then reads
    /// `NULL or <expected>`. Only used when `expected_known` is `false`.
    nullable: bool,
}

impl ArgContext {
    /// The context of parameter `r_name` of type `ty`; `coerced` when the
    /// `coerce` knob applies to it (the expectation widens with the gate).
    fn new(r_name: &str, ty: &syn::Type, coerced: bool) -> Self {
        Self::with_expectation(
            r_name,
            ty,
            crate::r_preconditions::conversion_expectation(ty, coerced),
        )
    }

    /// The context of a `strict` lossy-integer parameter (#1594): the
    /// expectation names what strict accepts, a whole number, since a
    /// logical, raw or fractional input is refused.
    fn strict(r_name: &str, ty: &syn::Type) -> Self {
        Self::with_expectation(
            r_name,
            ty,
            crate::r_preconditions::strict_conversion_expectation(ty),
        )
    }

    fn with_expectation(r_name: &str, ty: &syn::Type, expected: Option<String>) -> Self {
        let prefix = match &expected {
            Some(expected) => format!("'{r_name}' must be {expected}"),
            None => format!("invalid '{r_name}' argument"),
        };
        let value_ty = crate::miniextendr_fn::get_missing_inner_type(ty).unwrap_or(ty);
        Self {
            prefix,
            expected_known: expected.is_some(),
            r_name: r_name.to_string(),
            rust_type: crate::type_inspect::type_display(ty),
            nullable: crate::type_inspect::is_option_type(value_ty),
        }
    }
}

/// The `Err(e)` arm of a conversion binding: return the tagged
/// `kind = "conversion"` value. `__mx_conversion_err_parts!` takes the class
/// vector, message and data from an `RConditionError` error type, the
/// R-worded reason from a built-in conversion error and the `Display` text
/// otherwise; `conversion_condition_value` puts the context's prefix before
/// it, appends the crate's `conversion_error_class` and adds `e$param` and
/// `e$rust_type`.
fn conversion_err_arm(
    ctx: &ArgContext,
    crate_class: &[String],
    call_context_ident: &syn::Ident,
    span: proc_macro2::Span,
) -> TokenStream {
    let ArgContext {
        prefix,
        expected_known,
        r_name,
        rust_type,
        nullable,
    } = ctx;
    let value = conversion_value_tokens(
        &ConversionSubject {
            static_prefix: expected_known.then_some(prefix.as_str()),
            quoted: r_name,
            param: r_name,
            nullable: *nullable,
            rust_type,
        },
        crate_class,
        &quote! { Some(#call_context_ident) },
        span,
    );
    // SAFETY (of the emitted `unsafe`): the arm runs inside the wrapper's
    // with_r_unwind_protect closure, on the R main thread.
    quote_spanned! {span=>
        Err(e) => return unsafe { #value },
    }
}

/// What a conversion failure is about: the value it names and how.
pub(crate) struct ConversionSubject<'a> {
    /// `'<p>' must be <expected>` when the macro knows the R-facing
    /// expectation, else `None` (asked of the error at run time).
    pub(crate) static_prefix: Option<&'a str>,
    /// The name quoted in the message: the parameter, or a sidecar's field.
    pub(crate) quoted: &'a str,
    /// `e$param`: the R formal that failed.
    pub(crate) param: &'a str,
    /// The value may be `NULL` (`Option<_>`): a run-time expectation reads
    /// `NULL or <expected>`.
    pub(crate) nullable: bool,
    /// `e$rust_type`: the Rust type as written.
    pub(crate) rust_type: &'a str,
}

/// The `conversion_condition_value(...)` expression for the error bound as
/// `e` on `subject`, with the crate class and `call`.
///
/// With a `static_prefix` (the macro knows the R-facing expectation,
/// `'<p>' must be <expected>`) the prefix is that literal. Without one, the
/// error may know what the value should have been (a `match_arg` choice
/// error: `one of "fast", "slow"`, #1594), so the prefix is built on the
/// failure path from `__mx_conversion_expectation!(e)` by
/// `condition::conversion_prefix` (`NULL or ...` when `nullable`), falling
/// back to `invalid '<p>' argument`. Shared by the argument conversions and
/// the sidecar setters.
pub(crate) fn conversion_value_tokens(
    subject: &ConversionSubject,
    crate_class: &[String],
    call: &TokenStream,
    span: proc_macro2::Span,
) -> TokenStream {
    let ConversionSubject {
        static_prefix,
        quoted,
        param,
        nullable,
        rust_type,
    } = subject;
    let rust_type = quote! { ::core::option::Option::Some(#rust_type) };
    match static_prefix {
        Some(prefix) => quote_spanned! {span=>
            ::miniextendr_api::error_value::conversion_condition_value(
                #prefix,
                #param,
                #rust_type,
                &[#(#crate_class),*],
                ::miniextendr_api::__mx_conversion_err_parts!(e, true),
                #call,
            )
        },
        None => quote_spanned! {span=> {
            let __mx_expected = ::miniextendr_api::__mx_conversion_expectation!(e);
            ::miniextendr_api::error_value::conversion_condition_value(
                &::miniextendr_api::condition::conversion_prefix(
                    #quoted,
                    #nullable,
                    __mx_expected.as_deref(),
                ),
                #param,
                #rust_type,
                &[#(#crate_class),*],
                ::miniextendr_api::__mx_conversion_err_parts!(e, __mx_expected.is_some()),
                #call,
            )
        } },
    }
}

// endregion

#[cfg(test)]
mod tests;
