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
    /// `no_na` parameters (Rust name) with the author's `no_na(message = ..)`,
    /// if any: their converted value is checked right after the conversion
    /// (see [`Self::with_no_na`]).
    no_na_params: Vec<(String, Option<String>)>,
    /// Parameters (Rust name) declared with parameter markers
    /// (`Checked<T>` / `Unchecked<T>`, #1566), outermost first: the inner
    /// type converts, then the value is wrapped (see
    /// [`Self::with_param_markers`]).
    param_markers: Vec<(String, Vec<crate::type_inspect::ParamMarker>)>,
    /// The crate's `conversion_error_class` (`[package.metadata.miniextendr]`),
    /// appended to every conversion condition's class vector.
    conversion_error_class: Vec<String>,
}

impl RustConversionBuilder {
    /// Create a new conversion builder, carrying the crate-level
    /// `conversion_error_class` from the manifest (empty when unset).
    pub fn new() -> Self {
        Self {
            coerce_all: false,
            coerce_params: Vec::new(),
            strict: false,
            match_arg_several_ok_params: Vec::new(),
            layered_choice_params: Vec::new(),
            no_na_params: Vec::new(),
            param_markers: Vec::new(),
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

    /// Mark a `#[miniextendr(no_na)]` parameter (`param_name` is its Rust
    /// name), with the author's `no_na(message = "...")` if given.
    ///
    /// The R guard `!anyNA(x)` runs first; a type can still read an input as
    /// missing that `anyNA()` does not see (the reading markers `AsNumeric*` /
    /// `AsCharacter*` read the text `"NA"` or a blank string as `NA`). So when
    /// the parameter converts through plain `TryFromSexp`, the converted value
    /// is asked `TryFromSexp::__mx_has_na` right after its binding, and a
    /// refusal returns the R guard's own condition (see
    /// [`no_na_value_stmt`]). No type is named: Rust resolves the call on the
    /// value's real type, so aliases and derived newtypes of a marker are
    /// checked too, and for any other type the default `false` optimises out.
    /// The reference arms (`&T`, `&[T]`, `&str`) emit no check and keep only
    /// the R guard: they convert through the reference type's own impl, and
    /// no reading marker converts by reference.
    ///
    /// An `Either` parameter (also under `Missing`) has no R guard
    /// (`r_preconditions::no_na_checked_after_conversion`). Its value is asked
    /// `TryFromSexp::__mx_input_has_na` with the input SEXP instead: the arm
    /// the value converted to makes the guard's `anyNA()` check on the input
    /// in Rust, then its own `__mx_has_na`, and a `DataFrame` arm refuses
    /// nothing (also behind a derived newtype or `Result<_, ()>`, which
    /// forward the call).
    pub fn with_no_na(mut self, param_name: String, message: Option<String>) -> Self {
        self.no_na_params.push((param_name, message));
        self
    }

    /// Mark a parameter (`param_name` is its Rust name) whose declared type
    /// wraps the one it converts as in parameter markers (`Checked<T>` /
    /// `Unchecked<T>`, #1566), outermost first. The `pat_type` this builder
    /// sees holds the inner type, so the conversion and its failure context
    /// (`e$rust_type`) are the inner type's; after the parameter's last
    /// statement (the borrow on the split worker path, else the conversion or
    /// its `no_na` check) the value is rebound through each marker's
    /// `from_inner`, innermost first, so the call receives the declared type.
    pub fn with_param_markers(
        mut self,
        param_name: String,
        markers: Vec<crate::type_inspect::ParamMarker>,
    ) -> Self {
        self.param_markers.push((param_name, markers));
        self
    }

    /// The rebinding of a marked parameter's converted value (see
    /// [`Self::with_param_markers`]), or `None` for an unmarked parameter.
    fn marker_rebind(&self, pat_type: &syn::PatType) -> Option<TokenStream> {
        let syn::Pat::Ident(pat_ident) = pat_type.pat.as_ref() else {
            return None;
        };
        let ident = &pat_ident.ident;
        let name = crate::naming::ident_name(ident);
        let (_, markers) = self.param_markers.iter().find(|(n, _)| *n == name)?;
        let span = pat_type.ty.span();
        let rebinds = markers.iter().rev().map(|marker| {
            let ctor = marker.ctor_path();
            quote_spanned! {span=> let #ident = #ctor::from_inner(#ident); }
        });
        Some(quote! { #(#rebinds)* })
    }

    /// Check if a parameter should use coercion.
    ///
    /// Returns `true` if `coerce_all` is set or `param_name` appears in the per-parameter list.
    fn should_coerce(&self, param_name: &str) -> bool {
        self.coerce_all || self.coerce_params.contains(&param_name.to_string())
    }

    /// Describe only the conversion actually used for this argument.
    ///
    /// Special paths can have no TryFromSexp impl for the declared type. Omit
    /// their metadata queries at codegen time; a runtime branch still typechecks.
    pub(crate) fn native_borrow_metadata(
        &self,
        pat_type: &syn::PatType,
        sexp_ident: &syn::Ident,
    ) -> Option<TokenStream> {
        let syn::Pat::Ident(pat_ident) = pat_type.pat.as_ref() else {
            return None;
        };
        let ty = pat_type.ty.as_ref();
        if crate::type_inspect::call_marker(ty).is_some() {
            return None;
        }
        let param_name = crate::naming::ident_name(&pat_ident.ident);
        match ty {
            syn::Type::ImplTrait(_) => return None,
            syn::Type::Tuple(t) if t.elems.is_empty() => return None,
            syn::Type::Reference(r) => {
                if let syn::Type::Path(tp) = r.elem.as_ref()
                    && (tp.path.is_ident("str")
                        || tp
                            .path
                            .segments
                            .last()
                            .is_some_and(|seg| seg.ident == "Dots"))
                {
                    return None;
                }
                if self.match_arg_several_ok_params.contains(&param_name)
                    && crate::classify_several_ok_container(ty).is_some()
                {
                    return None;
                }
            }
            _ => {
                if self.strict
                    && crate::return_type_analysis::strict_input_conversion_for_type(ty, sexp_ident)
                        .is_some()
                {
                    return None;
                }
                if self
                    .layered_choice_params
                    .iter()
                    .any(|(name, _)| *name == param_name)
                {
                    return None;
                }
                if self.match_arg_several_ok_params.contains(&param_name)
                    && crate::classify_several_ok_container(ty).is_some()
                {
                    return None;
                }
                // Numeric delegates to TryFromSexp, so retain that converter's metadata.
                if self.should_coerce(&param_name)
                    && let Some(mapping) = CoercionMapping::from_type(ty)
                    && !matches!(mapping, CoercionMapping::Numeric)
                {
                    return None;
                }
            }
        }
        Some(quote! { <#ty as ::miniextendr_api::TryFromSexp>::NATIVE_BORROW })
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
        let err_arm = conversion_err_arm(ctx, &self.conversion_error_class, span);
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
        owned
            .into_iter()
            .chain(borrowed)
            .chain(self.marker_rebind(pat_type))
            .collect()
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
        let (mut owned, mut borrowed) =
            self.build_conversion_split_inner(pat_type, sexp_ident, false);
        // A marker wraps the parameter's final binding: the borrow inside the
        // worker closure when there is one, else the owned value, which then
        // moves into the closure as the declared type.
        if let Some(rebind) = self.marker_rebind(pat_type) {
            if borrowed.is_empty() {
                owned.push(rebind);
            } else {
                borrowed.push(rebind);
            }
        }
        (owned, borrowed)
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
            // The binding keeps the method's own use of the parameter from adding
            // a follow-on "cannot find value" to the diagnostic; the wrapper's call
            // slot is in scope in every C wrapper, so it also type-checks. The slot
            // is built with plain `quote!` so it keeps the declaration's call-site
            // hygiene when the method comes from a `macro_rules!` expansion (#1489).
            let call_slot = quote! { __miniextendr_call };
            let stmt = quote_spanned! {span=>
                ::core::compile_error!(
                    "`Call` / `CallerCall` parameters are supported on standalone `#[miniextendr]` \
                     functions only; class and trait methods attribute conditions to the wrapper's own call"
                );
                let #ident: #ty = <#ty>::from_sexp(#call_slot);
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
                        let err_arm = conversion_err_arm(&ctx, &self.conversion_error_class, span);
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

                // A choice under `Missing` / `Option` / `Either` layers (#1473,
                // #1551): decoded layer by layer, outermost first. Under an
                // `Either<.., R>` layer, a value `R` refuses is reported against
                // the whole parameter, in the words of its `@param` line, and an
                // `R` that reads only character input is a compile error.
                if let Some((_, leaf)) = self
                    .layered_choice_params
                    .iter()
                    .find(|(name, _)| *name == param_name)
                {
                    let span = ty.span();
                    let try_expr = layered_choice_expr(ty, leaf.clone(), sexp_ident, span);
                    let choice_ctx = ArgContext::either_choice(&r_name, ty, leaf);
                    let stmt = self.conversion_stmt(
                        try_expr,
                        choice_ctx.as_ref().unwrap_or(&ctx),
                        ident,
                        ty,
                        span,
                    );
                    let mut stmts = either_arm_guards(&param_name, ty);
                    stmts.push(stmt);
                    return (stmts, vec![]);
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
                            let len_arm =
                                conversion_err_arm(&len_ctx, &self.conversion_error_class, span);
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
                        // `ty: TryFromSexp` here, so a failure can ask the
                        // type what it declares.
                        let stmt =
                            self.conversion_stmt(try_expr, &ctx.with_declared(ty), ident, ty, span);
                        // `ty: TryFromSexp` is proven here, so a `no_na`
                        // parameter's value can be asked after the binding.
                        // Owned vector: it runs on the main thread, before a
                        // worker closure, like the conversion's `Err` arm.
                        if let Some((_, message)) = self
                            .no_na_params
                            .iter()
                            .find(|(name, _)| *name == param_name)
                        {
                            let message = crate::r_preconditions::no_na_message(
                                &r_name,
                                ty,
                                message.as_deref(),
                            );
                            // An `Either` has no R guard: the arm it converted
                            // to reads the input instead. The input is bound
                            // first, since the conversion binding may shadow
                            // it (a C wrapper can keep the parameter's name).
                            if crate::r_preconditions::no_na_checked_after_conversion(ty) {
                                let input = quote::format_ident!("__no_na_input_{}", plain);
                                let bind = quote_spanned! {span=>
                                    let #input = #sexp_ident;
                                };
                                let check = no_na_value_stmt(
                                    ident,
                                    Some(&input),
                                    &r_name,
                                    &message,
                                    &self.conversion_error_class,
                                    span,
                                );
                                return (vec![bind, stmt, check], vec![]);
                            }
                            let check = no_na_value_stmt(
                                ident,
                                None,
                                &r_name,
                                &message,
                                &self.conversion_error_class,
                                span,
                            );
                            return (vec![stmt, check], vec![]);
                        }
                        stmt
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

impl Default for RustConversionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// region: layered choice parameters

/// Where the innermost value of a layered choice parameter comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChoiceLeaf {
    /// A scalar `match_arg` type (`T: MatchArg`), decoded with
    /// `match_arg_from_sexp::<T>`.
    MatchArg,
    /// A `match_arg` + `several_ok` container (`Vec<T>` / `Box<[T]>`),
    /// decoded with `match_arg_vec_from_sexp::<T>`: under `Missing<..>`, or
    /// on the left of an `Either<.., R>` (#1612).
    MatchArgSeveral,
    /// The string type of a `choices(...)` parameter, decoded with its own
    /// `TryFromSexp` (the R prelude already matched it against the list).
    Literal {
        /// The choice list, which the argument error of an `Either<.., R>`
        /// parameter names.
        choices: Vec<String>,
        /// `several_ok`: one or more of the choices.
        several: bool,
    },
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
/// `Either<Route, DataFrame>` becomes
///
/// ```ignore
/// ::miniextendr_api::match_arg_either_or::<_, DataFrame, _>(sexp, ::miniextendr_api::match_arg_from_sexp::<Route>)
/// ```
///
/// and a `several_ok` list with another kind of value (#1612),
/// `Either<Vec<Fill>, DataFrame>`, becomes
///
/// ```ignore
/// ::miniextendr_api::match_arg_either_or::<_, DataFrame, _>(sexp, ::miniextendr_api::match_arg_vec_from_sexp::<Fill>)
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

/// One compile-time assertion per leaf of the `R` arm of an `Either` choice
/// parameter `param` of type `ty` (nested `Either`s on the `R` side peeled):
/// compilation fails when the leaf's `TryFromSexp` is `CHARACTER_ONLY`, since
/// the choice check never sends character or factor input there. Read
/// through `EitherArmProbe`, so a leaf without `TryFromSexp` adds no error of
/// its own. Empty without an `Either` layer.
///
/// Each guard is a block holding only items (the fallback trait import and a
/// `const _` assertion), spanned on the leaf: the assertion is evaluated by
/// `cargo check` too, and the error points at the arm.
fn either_arm_guards(param: &str, ty: &syn::Type) -> Vec<TokenStream> {
    fn leaves<'a>(ty: &'a syn::Type, out: &mut Vec<&'a syn::Type>) {
        match crate::type_inspect::either_arms(ty) {
            Some((left, right)) => {
                leaves(left, out);
                leaves(right, out);
            }
            None => out.push(ty),
        }
    }

    let Some(right) = crate::type_inspect::choice_layers(ty).either_right else {
        return Vec::new();
    };
    let nested = crate::type_inspect::either_arms(right).is_some();
    let mut arms = Vec::new();
    leaves(right, &mut arms);
    arms.into_iter()
        .map(|leaf| {
            let shown = crate::type_inspect::type_display(leaf);
            let arm = if nested {
                format!(
                    "the other arm's `{shown}` (in `{}`)",
                    crate::type_inspect::type_display(right)
                )
            } else {
                format!("the other arm `{shown}`")
            };
            let message = format!(
                "`Either` choice parameter `{param}`: {arm} reads only character or factor \
                 input, and the choice check sends every character or factor argument to the \
                 choice arm, so at most NULL can reach `{shown}`. Give that arm a type that \
                 reads other input (a data frame, a number, a list); for a NULL alternative, \
                 declare the parameter as `Option<Either<..>>`."
            );
            // `assert!` reads a lone literal as a format string.
            let message = message.replace('{', "{{").replace('}', "}}");
            let probe = crate::type_inspect::erase_lifetimes(leaf);
            quote_spanned! {leaf.span()=>
                {
                    #[allow(unused_imports)]
                    use ::miniextendr_api::match_arg::EitherArmProbeFallback as _;
                    #[allow(clippy::assertions_on_constants)]
                    const _: () = ::core::assert!(
                        !::miniextendr_api::match_arg::EitherArmProbe::<#probe>::CHARACTER_ONLY,
                        #message
                    );
                }
            }
        })
        .collect()
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
        let (choice_ty, boxed) = match (&leaf, crate::classify_several_ok_container(value)) {
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
        match (layers, &self.leaf) {
            ([], ChoiceLeaf::MatchArg) => {
                quote! { ::miniextendr_api::match_arg_from_sexp::<#t> }
            }
            ([], ChoiceLeaf::MatchArgSeveral) if !self.boxed => {
                quote! { ::miniextendr_api::match_arg_vec_from_sexp::<#t> }
            }
            ([], ChoiceLeaf::Literal { .. }) => {
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
    /// An expression yielding the expectation (a `String`) on the failure
    /// path, when the macro knows it but not its text: the choices of a
    /// `match_arg` type under an `Either<.., R>` layer come from
    /// `MatchArg::CHOICES`. Takes the place of `prefix`.
    expected_at_run_time: Option<TokenStream>,
    /// An expression yielding what the argument's type declares (an
    /// `Option<String>`) on the failure path, when the macro has no wording
    /// for it: set by [`ArgContext::with_declared`] where the binding
    /// converts with `TryFromSexp`, and tried before the error's own
    /// expectation ([`declared_expectation`]).
    declared: Option<TokenStream>,
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
            expected_at_run_time: None,
            declared: None,
            r_name: r_name.to_string(),
            rust_type: crate::type_inspect::type_display(ty),
            nullable: crate::type_inspect::is_option_type(value_ty),
        }
    }

    /// This context for a binding that converts `ty` with `TryFromSexp`: when
    /// the macro has no wording for the type, the failure path asks the type
    /// what it declares ([`declared_expectation`]). Only there is
    /// `ty: TryFromSexp` known; the other conversion paths (a `several_ok`
    /// `Vec<Mode>`, a coercion helper) may name a type that has no impl. An
    /// `impl Trait` type gets no lookup: it cannot name a path, and its
    /// binding is already the compile error.
    fn with_declared(&self, ty: &syn::Type) -> Self {
        let mut ctx = self.clone();
        if !ctx.expected_known && ctx.expected_at_run_time.is_none() && !names_impl_trait(ty) {
            ctx.declared = Some(declared_expectation(ty));
        }
        ctx
    }

    /// The context of a choice parameter of type `ty` decoded as `leaf`, when
    /// the type has an `Either<.., R>` layer; `None` otherwise. A value `R`
    /// refuses is reported against the whole parameter, in the words of its
    /// `@param` line: `'route' must be one of "oral", "bolus", or a data
    /// frame` (`one or more of` for `several_ok`, `, a data frame, or NULL`
    /// under `Option`). The choices are `MatchArg::CHOICES` for a `match_arg`
    /// type and the literal list for `choices(...)`; the `R` arm is named by
    /// its `@param` noun ([`crate::type_inspect::r_value_noun`]). The error
    /// then supplies only the reason (`got integer`).
    fn either_choice(r_name: &str, ty: &syn::Type, leaf: &ChoiceLeaf) -> Option<Self> {
        let layers = crate::type_inspect::choice_layers(ty);
        let right = layers.either_right?;
        let suffix = crate::miniextendr_fn::choice_alternatives_suffix(
            Some(&crate::type_inspect::r_value_noun(right)),
            layers.nullable,
            "NULL",
        );
        let (choices, several) = match leaf {
            ChoiceLeaf::Literal { choices, several } => (quote! { &[#(#choices),*] }, *several),
            ChoiceLeaf::MatchArg | ChoiceLeaf::MatchArgSeveral => {
                let choice_ty = LayeredValue::new(layers.value, leaf.clone()).choice_ty;
                (
                    quote! { <#choice_ty as ::miniextendr_api::MatchArg>::CHOICES },
                    *leaf == ChoiceLeaf::MatchArgSeveral,
                )
            }
        };
        Some(Self {
            expected_at_run_time: Some(quote! {
                ::miniextendr_api::match_arg::choice_expectation(#choices, #several, #suffix)
            }),
            ..Self::with_expectation(r_name, ty, None)
        })
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
    span: proc_macro2::Span,
) -> TokenStream {
    let ArgContext {
        prefix,
        expected_known,
        expected_at_run_time,
        declared,
        r_name,
        rust_type,
        nullable,
    } = ctx;
    let expected = match expected_at_run_time {
        Some(expr) => Expected::RunTime(expr),
        None if *expected_known => Expected::Literal(prefix),
        None => Expected::FromError(declared.as_ref()),
    };
    let value = conversion_value_tokens(
        &ConversionSubject {
            expected,
            quoted: r_name,
            param: r_name,
            nullable: *nullable,
            rust_type,
        },
        crate_class,
        &quote! { Some(__miniextendr_call) },
        span,
    );
    // SAFETY (of the emitted `unsafe`): the arm runs inside the wrapper's
    // with_r_unwind_protect closure, on the R main thread.
    quote_spanned! {span=>
        Err(e) => return unsafe { #value },
    }
}

/// The post-conversion `no_na` check of the binding `ident` (R name
/// `r_name`): when the value holds what its type reads as `NA`
/// (`TryFromSexp::__mx_has_na`), return the tagged `kind = "conversion"`
/// value `no_na`'s R guard would raise, with `message` verbatim, the crate's
/// `conversion_error_class` and `e$param` (`arg_check_condition_value`). The
/// call is the wrapper's call slot, as for a conversion failure.
///
/// `input` is the argument's SEXP when no R guard ran (an `Either`, see
/// `r_preconditions::no_na_checked_after_conversion`): the value is then
/// asked `TryFromSexp::__mx_input_has_na`, which also makes the guard's
/// `anyNA()` check on the input, by the arm the value converted to.
fn no_na_value_stmt(
    ident: &syn::Ident,
    input: Option<&syn::Ident>,
    r_name: &str,
    message: &str,
    crate_class: &[String],
    span: proc_macro2::Span,
) -> TokenStream {
    let has_na = match input {
        Some(sexp) => quote_spanned! {span=>
            ::miniextendr_api::TryFromSexp::__mx_input_has_na(&#ident, #sexp)
        },
        None => quote_spanned! {span=>
            ::miniextendr_api::TryFromSexp::__mx_has_na(&#ident)
        },
    };
    // SAFETY (of the emitted `unsafe`): the check runs right after the
    // argument's conversion, where the conversion `Err` arm runs: on the R
    // main thread, inside the wrapper's with_r_unwind_protect closure or
    // before the worker closure.
    quote_spanned! {span=>
        if #has_na {
            return unsafe {
                ::miniextendr_api::error_value::arg_check_condition_value(
                    #message,
                    #r_name,
                    &[#(#crate_class),*],
                    Some(__miniextendr_call),
                )
            };
        }
    }
}

/// Where the `<expected>` of a conversion failure's `'<p>' must be <expected>`
/// comes from.
pub(crate) enum Expected<'a> {
    /// The macro knows it: the whole prefix, `'<p>' must be <expected>`.
    Literal(&'a str),
    /// An expression the macro wrote yields it on the failure path (the
    /// choices of a `match_arg` type): the prefix is `'<p>' must be <it>`.
    RunTime(&'a TokenStream),
    /// What the type declares, when an expression for it is given
    /// ([`declared_expectation`]), else what the error may know
    /// (`__mx_conversion_expectation!`), else the prefix is
    /// `invalid '<p>' argument`.
    FromError(Option<&'a TokenStream>),
}

/// What a conversion failure is about: the value it names and how.
pub(crate) struct ConversionSubject<'a> {
    /// Where the expectation in the message prefix comes from.
    pub(crate) expected: Expected<'a>,
    /// The name quoted in the message: the parameter, or a sidecar's field.
    pub(crate) quoted: &'a str,
    /// `e$param`: the R formal that failed.
    pub(crate) param: &'a str,
    /// The value may be `NULL` (`Option<_>`): an expectation the error
    /// supplies reads `NULL or <expected>` ([`Expected::FromError`] only).
    pub(crate) nullable: bool,
    /// `e$rust_type`: the Rust type as written.
    pub(crate) rust_type: &'a str,
}

/// The `conversion_condition_value(...)` expression for the error bound as
/// `e` on `subject`, with the crate class and `call`.
///
/// With [`Expected::Literal`] (the macro knows the R-facing expectation,
/// `'<p>' must be <expected>`) the prefix is that literal; with
/// [`Expected::RunTime`] it is `'<p>' must be ` and the expression's value.
/// With [`Expected::FromError`] the prefix is built on the failure path by
/// `condition::conversion_prefix` (`NULL or ...` when `nullable`) from what
/// the type declares (a `#[derive(TryFromSexp)]` newtype, the sides of an
/// `Either`), else what the error may know (a `match_arg` choice error: `one
/// of "fast", "slow"`, #1594, from `__mx_conversion_expectation!(e)`),
/// falling back to `invalid '<p>' argument`.
/// Shared by the argument conversions and the sidecar setters.
pub(crate) fn conversion_value_tokens(
    subject: &ConversionSubject,
    crate_class: &[String],
    call: &TokenStream,
    span: proc_macro2::Span,
) -> TokenStream {
    let ConversionSubject {
        expected,
        quoted,
        param,
        nullable,
        rust_type,
    } = subject;
    let rust_type = quote! { ::core::option::Option::Some(#rust_type) };
    match expected {
        Expected::Literal(prefix) => quote_spanned! {span=>
            ::miniextendr_api::error_value::conversion_condition_value(
                #prefix,
                #param,
                #rust_type,
                &[#(#crate_class),*],
                ::miniextendr_api::__mx_conversion_err_parts!(e, true),
                #call,
            )
        },
        Expected::RunTime(expr) => quote_spanned! {span=>
            ::miniextendr_api::error_value::conversion_condition_value(
                &::std::format!("'{}' must be {}", #quoted, #expr),
                #param,
                #rust_type,
                &[#(#crate_class),*],
                ::miniextendr_api::__mx_conversion_err_parts!(e, true),
                #call,
            )
        },
        Expected::FromError(declared) => {
            let from_error = quote! { ::miniextendr_api::__mx_conversion_expectation!(e) };
            let expected = match declared {
                Some(declared) => quote! { (#declared).or_else(|| #from_error) },
                None => from_error,
            };
            quote_spanned! {span=> {
                let __mx_expected: ::core::option::Option<::std::string::String> = #expected;
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
            } }
        }
    }
}

/// The expression, on a conversion failure's path, for what a parameter of
/// type `ty` declares when the macro has no wording for it: an
/// `Option<String>`, with the error bound as `e`. Only for a type the binding
/// converts with `TryFromSexp`, whose `__MX_EXPECTATION` it reads.
///
/// An `Either` (also under `Missing`) is worded side by side
/// (`from_r::either_expectation`, [`expected_arm`]), so one side the macro
/// does not know no longer leaves the whole parameter without an
/// expectation; any other type is asked for its `__MX_EXPECTATION` (a
/// `#[derive(TryFromSexp)]` newtype says what its inner type does). The
/// `NULL or` of an `Option<_>` parameter is the prefix's (`nullable`).
pub(crate) fn declared_expectation(ty: &syn::Type) -> TokenStream {
    let value = crate::miniextendr_fn::get_missing_inner_type(ty).unwrap_or(ty);
    let value = crate::type_inspect::option_inner_type(value).unwrap_or(value);
    if crate::type_inspect::either_arms(value).is_some() {
        let arms = expected_arm(value);
        quote! { ::miniextendr_api::from_r::either_expectation(&e, &#arms) }
    } else {
        let value = crate::type_inspect::erase_lifetimes(value);
        quote! {
            <#value as ::miniextendr_api::TryFromSexp>::__MX_EXPECTATION
                .map(::std::string::ToString::to_string)
        }
    }
}

/// The `from_r::ExpectedArm` of one side of an `Either`, of type `ty`: the
/// macro's wording when it has one
/// ([`crate::r_preconditions::conversion_expectation`]), a nested `Either`
/// side by side, `NULL or <T>` for an `Option<T>` or `Result<T, ()>` side,
/// else the type's `__MX_EXPECTATION`, with its `@param` noun
/// ([`crate::type_inspect::r_value_noun`]) for when neither the type nor its
/// error names it. Every side is a `TryFromSexp` type, as `Either`'s own
/// impl requires (and the `Option` / `Result` impls of their `T`).
fn expected_arm(ty: &syn::Type) -> TokenStream {
    let arm = quote! { ::miniextendr_api::from_r::ExpectedArm };
    if let Some(text) = crate::r_preconditions::conversion_expectation(ty, false) {
        return quote! { #arm::Known(#text) };
    }
    if let Some((left, right)) = crate::type_inspect::either_arms(ty) {
        let (left, right) = (expected_arm(left), expected_arm(right));
        return quote! { #arm::Either(&#left, &#right) };
    }
    if let Some(inner) =
        crate::type_inspect::option_inner_type(ty).or_else(|| unit_result_ok_type(ty))
    {
        let inner = expected_arm(inner);
        return quote! { #arm::Nullable(&#inner) };
    }
    let noun = crate::type_inspect::r_value_noun(ty);
    let ty = crate::type_inspect::erase_lifetimes(ty);
    quote! {
        #arm::Opaque {
            declared: <#ty as ::miniextendr_api::TryFromSexp>::__MX_EXPECTATION,
            noun: #noun,
        }
    }
}

/// Whether `ty` spells an `impl Trait` anywhere (`impl AsRef<str>`,
/// `Vec<impl Display>`).
fn names_impl_trait(ty: &syn::Type) -> bool {
    fn scan(tokens: TokenStream) -> bool {
        tokens.into_iter().any(|tt| match tt {
            proc_macro2::TokenTree::Ident(ident) => ident == "impl",
            proc_macro2::TokenTree::Group(group) => scan(group.stream()),
            _ => false,
        })
    }
    scan(quote! { #ty })
}

/// `T` for a `Result<T, ()>` type, which reads `NULL` as `Err(())`.
fn unit_result_ok_type(ty: &syn::Type) -> Option<&syn::Type> {
    let syn::Type::Path(tp) = ty else {
        return None;
    };
    let seg = tp.path.segments.last()?;
    if seg.ident != "Result" {
        return None;
    }
    let ok = crate::type_inspect::first_type_argument(seg)?;
    match crate::type_inspect::second_type_argument(seg)? {
        syn::Type::Tuple(unit) if unit.elems.is_empty() => Some(ok),
        _ => None,
    }
}

// endregion

#[cfg(test)]
mod tests;
