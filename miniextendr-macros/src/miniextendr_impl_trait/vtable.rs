//! Vtable static generation, C wrapper generation, and method attribute parsing for trait impls.
//!
//! This module contains the core codegen for `#[miniextendr]` on `impl Trait for Type` blocks.
//! It produces the vtable static constant, C-callable wrapper functions for each method and
//! associated constant, and delegates to [`super::r_wrappers`] for R wrapper code generation.

use proc_macro2::TokenStream;
use quote::format_ident;
use syn::ItemImpl;

use super::r_wrappers::{TraitWrapperOpts, generate_trait_r_wrapper};
use super::{TraitAbiMetadata, TraitConst, TraitMethod, type_to_symbol_name};
use crate::miniextendr_impl::{ClassSystem, ImplAttrs};

/// Generate the vtable static, C wrappers, R wrappers, and call defs for a trait implementation.
///
/// This is the main entry point for trait impl codegen. For a given
/// `impl Trait for Type { ... }` block, it produces:
///
/// - The cleaned original impl block (with `#[miniextendr]` attrs stripped from methods)
/// - A `static __VTABLE_{crate}_{Trait}_FOR_{Type}` with the declared repr(C) layout
/// - C wrapper functions and `R_CallMethodDef` entries for each method and const
/// - R wrapper code string (class-system specific)
/// - Two `const` items: `{TYPE}_{TRAIT}_CALL_DEFS` and `R_WRAPPERS_{TYPE}_{TRAIT}_IMPL`
///
/// # Arguments
///
/// - `impl_item`: The parsed `impl Trait for Type` block
/// - `trait_path`: Full path to the trait (e.g., `crate::Counter`)
/// - `concrete_type`: The implementing type (e.g., `MyCounter`)
/// - `impl_attrs`: The impl block's options. The class system picks the R
///   wrapper generator; `blanket` skips emitting the impl block (a blanket impl
///   already provides it); `internal` / `noexport` control the roxygen export
///   tags; `no_preconditions` drops the R-side type checks, as on an inherent
///   impl.
/// - `abi`: The canonical trait name, fingerprint and callback layout from the declaration.
pub(super) fn generate_vtable_static(
    impl_item: &ItemImpl,
    trait_path: &syn::Path,
    concrete_type: &syn::Type,
    impl_attrs: &ImplAttrs,
    abi: &TraitAbiMetadata,
    declaration: &[syn::TraitItemFn],
) -> TokenStream {
    let trait_name = &abi.name;

    // Extract type identifier (for simple types)
    let type_ident = match concrete_type {
        syn::Type::Path(type_path) => {
            let Some(last_seg) = type_path.path.segments.last() else {
                return syn::Error::new_spanned(
                    concrete_type,
                    "type path must have at least one segment",
                )
                .into_compile_error();
            };
            last_seg.ident.clone()
        }
        _ => format_ident!("Unknown"),
    };

    // Extract type name for naming (simplified - handles Path types)
    let type_name_str = type_to_symbol_name(concrete_type);

    // Generate names (crate-prefixed vtable static — #1273 — routed through the
    // shared naming.rs helper so this can't drift from the other two
    // reconstruction sites in `miniextendr_impl_trait.rs`)
    let vtable_static_name = crate::naming::vtable_static_ident(trait_name, &type_name_str);

    // The impl block renders no page under `@noRd` or a plain `noexport`
    // (`generate_trait_r_wrapper`'s `suppress_all_rd`); its methods' docs, of
    // which S3 / vctrs keep the `@param` lines, then keep no links (#1818).
    let impl_page = crate::roxygen::RdPage::suppressed_if(
        crate::roxygen::attrs_have_roxygen_tag(&impl_item.attrs, "noRd")
            || (impl_attrs.noexport && !impl_attrs.internal),
    );

    // Parse methods and consts from the impl block
    let all_methods = match extract_methods(impl_item, impl_page) {
        Ok(m) => m,
        Err(e) => return e.into_compile_error(),
    };
    let consts = extract_consts(impl_item);

    // Separate skipped methods: skipped methods are kept in the emitted impl block
    // but excluded from C wrappers, R wrappers and call defs. The declaration
    // alone determines vtable membership, including when an impl hides a wrapper.
    let methods: Vec<&TraitMethod> = all_methods.iter().filter(|m| !m.skip).collect();

    // Generate C wrappers and call defs for each non-skipped method
    let method_c_wrappers: Vec<TokenStream> = methods
        .iter()
        .map(|m| generate_trait_method_c_wrapper(m, &type_ident, trait_name, trait_path))
        .collect();

    // Generate C wrappers for consts
    let const_c_wrappers: Vec<TokenStream> = consts
        .iter()
        .map(|c| generate_trait_const_c_wrapper(c, &type_ident, trait_name, trait_path))
        .collect();

    // Combine C wrappers
    let c_wrappers: Vec<TokenStream> = method_c_wrappers
        .into_iter()
        .chain(const_c_wrappers)
        .collect();

    // Check if impl block has @noRd doc comment (strip @param from class-level tags)
    let raw_impl_tags = crate::roxygen::roxygen_tags_from_attrs(&impl_item.attrs, impl_page);
    let (impl_doc_tags, param_warnings) = crate::roxygen::strip_method_tags(
        &raw_impl_tags,
        &type_ident.to_string(),
        impl_item.impl_token.span,
    );
    let class_has_no_rd = crate::roxygen::has_roxygen_tag(&impl_doc_tags, "noRd");

    // Generate R wrapper code string based on class system (only non-skipped methods)
    let methods_owned: Vec<TraitMethod> = methods.iter().map(|m| (*m).clone()).collect();
    let r_wrapper_string = match generate_trait_r_wrapper(
        &type_ident,
        trait_name,
        &methods_owned,
        &consts,
        TraitWrapperOpts {
            class_system: impl_attrs.class_system,
            class_has_no_rd,
            internal: impl_attrs.internal,
            noexport: impl_attrs.noexport,
            impl_preconditions: impl_attrs.preconditions,
        },
    ) {
        Ok(s) => s,
        Err(e) => return e.into_compile_error(),
    };

    // Generate constant name for R wrapper registration
    let r_wrappers_const = crate::naming::trait_impl_wrappers_ident(&type_name_str, trait_name);

    // Generate trait dispatch entry name
    let dispatch_entry_name = crate::naming::trait_dispatch_ident(trait_name, &type_name_str);

    let trait_tag = &abi.tag;

    // Format R wrapper as raw string literal
    let r_wrapper_str = crate::r_wrapper_raw_literal(
        &r_wrapper_string,
        &crate::r_shadowing::formal_names(methods_owned.iter().map(|m| &m.sig.inputs)),
    );
    let source_loc_doc = crate::source_location_doc(type_ident.span());
    let source_start = type_ident.span().start();
    let source_line_lit = syn::LitInt::new(&source_start.line.to_string(), type_ident.span());
    let source_col_lit =
        syn::LitInt::new(&(source_start.column + 1).to_string(), type_ident.span());

    // Strip #[miniextendr(...)] attrs from methods before emitting,
    // so they don't trigger another macro expansion.
    //
    // Skip emitting the impl block when:
    // - Body is empty (no methods, no consts) — blanket impl provides it
    // - `blanket` flag is set — a blanket impl exists, methods are only for
    //   C wrapper signature extraction, not for actual trait implementation
    let has_items = !all_methods.is_empty() || !consts.is_empty();
    let clean_impl_tokens = if has_items && !impl_attrs.blanket {
        let mut clean_impl = impl_item.clone();
        for item in &mut clean_impl.items {
            if let syn::ImplItem::Fn(method) = item {
                method
                    .attrs
                    .retain(|attr| !attr.path().is_ident("miniextendr"));
                crate::trait_receiver::rewrite(&mut method.sig, Some(&mut method.block));
            }
        }
        quote::quote! { #clean_impl }
    } else {
        quote::quote! {}
    };

    let vtable_static_tokens =
        match generate_impl_vtable(abi, concrete_type, trait_path, &all_methods, declaration) {
            Ok(tokens) => tokens,
            Err(error) => return error.into_compile_error(),
        };

    quote::quote! {
        // Pass through the original impl block (with method attrs stripped)
        // — omitted when the body is empty (blanket impl covers it)
        #clean_impl_tokens

        // Warnings for @param tags on trait impl blocks
        #param_warnings

        #[doc = ::core::concat!(
            "Vtable for `",
            ::core::stringify!(#concrete_type),
            "` implementing `",
            ::core::stringify!(#trait_path),
            "`."
        )]
        #[doc = "Generated by `#[miniextendr]` on the trait impl block."]
        #[doc = #source_loc_doc]
        #[doc = ::core::concat!("Generated from source file `", ::core::file!(), "`.")]
        #[doc(hidden)]
        #vtable_static_tokens

        // C wrappers and call method defs for trait methods
        #(#c_wrappers)*

        // R wrapper registration via distributed slice
        #[doc = ::core::concat!(
            "R wrapper code for `",
            ::core::stringify!(#type_ident),
            "` implementing `",
            ::core::stringify!(#trait_name),
            "`."
        )]
        #[doc = #source_loc_doc]
        #[doc = ::core::concat!("Generated from source file `", ::core::file!(), "`.")]
        #[doc(hidden)]
        #[cfg_attr(not(target_arch = "wasm32"), ::miniextendr_api::linkme::distributed_slice(::miniextendr_api::registry::MX_R_WRAPPERS), linkme(crate = ::miniextendr_api::linkme))]
        #[allow(non_upper_case_globals)]
        static #r_wrappers_const: ::miniextendr_api::registry::RWrapperEntry =
            ::miniextendr_api::registry::RWrapperEntry {
                priority: ::miniextendr_api::registry::RWrapperPriority::TraitImpl,
                source_file: ::core::file!(),
                source_line: #source_line_lit,
                content: ::core::concat!(
                    "# Generated from Rust impl `",
                    ::core::stringify!(#trait_name),
                    "` for `",
                    ::core::stringify!(#type_ident),
                    "` (",
                    ::core::file!(),
                    ":",
                    #source_line_lit,
                    ":",
                    #source_col_lit,
                    ")",
                    #r_wrapper_str
                ),
            };

        // Trait dispatch entry for universal_query
        #[doc = ::core::concat!(
            "Trait dispatch entry: `",
            ::core::stringify!(#trait_name),
            "` for `",
            ::core::stringify!(#type_ident),
            "`."
        )]
        #[doc = #source_loc_doc]
        #[doc = ::core::concat!("Generated from source file `", ::core::file!(), "`.")]
        #[doc(hidden)]
        #[cfg_attr(not(target_arch = "wasm32"), ::miniextendr_api::linkme::distributed_slice(::miniextendr_api::registry::MX_TRAIT_DISPATCH), linkme(crate = ::miniextendr_api::linkme))]
        #[allow(non_upper_case_globals)]
        static #dispatch_entry_name: ::miniextendr_api::registry::TraitDispatchEntry =
            ::miniextendr_api::registry::TraitDispatchEntry {
                concrete_tag: ::miniextendr_api::abi::mx_tag_from_path(
                    ::core::concat!(::core::module_path!(), "::", ::core::stringify!(#type_ident))
                ),
                trait_tag: ::miniextendr_api::abi::mx_tag_from_path(#trait_tag),
                vtable: unsafe {
                    // SAFETY: vtable is a static reference valid for program lifetime
                    ::std::ptr::from_ref(&#vtable_static_name).cast::<::std::os::raw::c_void>()
                },
                vtable_symbol: ::core::stringify!(#vtable_static_name),
            };
    }
}

/// Reproduce the declaration's repr(C) layout locally, with its exact field
/// order and callback types. The versioned declaration tag guards the cast to
/// the consumer's nominal vtable type. The exported static's name and byte
/// representation stay the same for native and wasm registration.
///
/// Non-generic callbacks belong to the trait, keeping default methods and
/// declaration-local argument types in their original scope. Generic traits
/// retain the existing concrete-shim path to avoid recursive conversion bounds.
pub(super) fn generate_impl_vtable(
    abi: &TraitAbiMetadata,
    concrete_type: &syn::Type,
    trait_path: &syn::Path,
    methods: &[TraitMethod],
    declaration: &[syn::TraitItemFn],
) -> syn::Result<TokenStream> {
    let trait_name = &abi.name;
    let type_name = type_to_symbol_name(concrete_type);
    let static_name = crate::naming::vtable_static_ident(trait_name, &type_name);
    let layout_name = crate::naming::vtable_layout_ident(trait_name, &type_name);
    let fields = &abi.fields;
    let type_ident = match concrete_type {
        syn::Type::Path(path) => path.path.segments.last().unwrap().ident.clone(),
        _ => format_ident!("Unknown"),
    };
    let mut concrete_methods = Vec::new();
    let mut initializers = Vec::new();
    for field in &fields.named {
        let name = field.ident.as_ref().unwrap();
        let callback = if abi.generic {
            let Some(method) = methods.iter().find(|method| method.ident == *name) else {
                let message = format!(
                    "this impl must define `{name}`, even if the trait gives it a default: \
                     a generic `#[miniextendr]` trait's shims are generated from the impl's \
                     method signatures"
                );
                return Err(syn::Error::new_spanned(trait_path, message));
            };
            concrete_methods.push(method.clone());
            let shim = crate::naming::vtshim_ident(&type_ident, trait_name, name);
            quote::quote!(#shim)
        } else {
            let shim = crate::naming::trait_method_shim_ident(trait_name, name);
            quote::quote!(<#concrete_type as #trait_path>::#shim)
        };
        initializers.push(quote::quote!(#name: #callback));
    }
    let shims = generate_concrete_vtable_shims(
        &concrete_methods,
        &type_ident,
        trait_name,
        trait_path,
        concrete_type,
        declaration,
    );
    Ok(quote::quote! {
        #[doc(hidden)]
        #[repr(C)]
        #[allow(non_camel_case_types)]
        pub struct #layout_name #fields

        #shims
        #[doc(hidden)]
        #[unsafe(no_mangle)]
        #[allow(non_upper_case_globals)]
        pub static #static_name: #layout_name = #layout_name {
            #(#initializers),*
        };
    })
}

/// Generate concrete vtable shims for a generic trait impl.
///
/// For generic traits (e.g., `RExtend<T>`), shims and the vtable builder function
/// cannot be generated at the trait definition site because where clauses like
/// `Vec<T>: TryFromSexp` cause recursive trait resolution overflow. Instead, we
/// generate fully monomorphized shims at the impl site where `T` is known (e.g., `T = i32`).
///
/// Each instance method gets a concrete `unsafe extern "C"` shim named
/// `__vtshim_{crate}_{Type}__{Trait}__{method}` (crate-prefixed for webR
/// cross-package symbol uniqueness — #1273) that, inside
/// `with_r_unwind_protect_shim`, follows the declaration shim's steps:
/// 1. Checks argument arity
/// 2. Refuses aliased native borrows among the arguments
/// 3. Extracts SEXP arguments to concrete Rust types
/// 4. Calls the method via fully-qualified syntax `<Type as Trait>::method()`
/// 5. Converts the result back to SEXP
///
/// Static methods are skipped (not part of the vtable).
fn generate_concrete_vtable_shims(
    methods: &[TraitMethod],
    type_ident: &syn::Ident,
    trait_name: &syn::Ident,
    trait_path: &syn::Path,
    concrete_type: &syn::Type,
    declaration: &[syn::TraitItemFn],
) -> TokenStream {
    let mut shims = Vec::new();

    for method in methods {
        if !method.has_self {
            continue; // Static methods not in vtable
        }

        let method_ident = &method.ident;
        let shim_name = crate::naming::vtshim_ident(type_ident, trait_name, method_ident);
        // Match the declaration shim's hygiene: source argument names must
        // not shadow ABI parameters or the generated receiver reference.
        let data = syn::Ident::new("data", proc_macro2::Span::mixed_site());
        let argc = syn::Ident::new("argc", proc_macro2::Span::mixed_site());
        let argv = syn::Ident::new("argv", proc_macro2::Span::mixed_site());
        let self_ref = syn::Ident::new("self_ref", proc_macro2::Span::mixed_site());
        let cell = syn::Ident::new("__mx_cell", proc_macro2::Span::mixed_site());
        let guard = syn::Ident::new("__mx_guard", proc_macro2::Span::mixed_site());
        let owner_kind = crate::trait_receiver::owner_receiver(&method.sig);
        let owner = syn::Ident::new("owner", proc_macro2::Span::mixed_site());
        let owner_parameter = owner_kind.map(|_| {
            quote::quote! {
                #owner: ::miniextendr_api::SEXP,
            }
        });

        // Count non-self parameters
        let param_count = method
            .sig
            .inputs
            .iter()
            .filter(|a| !matches!(a, syn::FnArg::Receiver(_)))
            .count();
        let expected_argc = param_count as i32;

        let declared = declaration
            .iter()
            .find(|declared| declared.sig.ident == *method_ident)
            .expect("trait metadata includes each callback's argument names");
        let diagnostic_names: Vec<_> = declared
            .sig
            .inputs
            .iter()
            .map(|arg| {
                let syn::FnArg::Typed(arg) = arg else {
                    unreachable!("metadata has no receiver")
                };
                let syn::Pat::Ident(name) = arg.pat.as_ref() else {
                    unreachable!("metadata has named arguments")
                };
                crate::naming::ident_name(&name.ident)
            })
            .collect();

        // Generate argument extraction (concrete types, no generics)
        let arg_extractions: Vec<TokenStream> = method
            .sig
            .inputs
            .iter()
            .filter(|a| !matches!(a, syn::FnArg::Receiver(_)))
            .enumerate()
            .map(|(i, arg)| {
                if let syn::FnArg::Typed(pt) = arg {
                    let name = if let syn::Pat::Ident(pat_ident) = pt.pat.as_ref() {
                        pat_ident.ident.clone()
                    } else {
                        format_ident!("arg{}", i)
                    };
                    crate::trait_self_ref::argument(
                        &name,
                        &diagnostic_names[i],
                        &pt.ty,
                        concrete_type,
                        i,
                        &argv,
                    )
                } else {
                    quote::quote! {}
                }
            })
            .collect();

        // Collect param names for the method call
        let param_names: Vec<syn::Ident> = method
            .sig
            .inputs
            .iter()
            .filter(|a| !matches!(a, syn::FnArg::Receiver(_)))
            .enumerate()
            .map(|(i, arg)| {
                if let syn::FnArg::Typed(pt) = arg
                    && let syn::Pat::Ident(pat_ident) = pt.pat.as_ref()
                {
                    return pat_ident.ident.clone();
                }
                format_ident!("arg{}", i)
            })
            .collect();

        let native_borrow_checks = crate::trait_self_ref::native_borrow_checks(
            param_names
                .iter()
                .zip(method.sig.inputs.iter().filter_map(|arg| match arg {
                    syn::FnArg::Typed(pt) => Some(pt.ty.as_ref()),
                    syn::FnArg::Receiver(_) => None,
                }))
                .enumerate()
                .map(|(index, (_, ty))| (index, diagnostic_names[index].clone(), ty)),
            &argv,
        );

        let self_refs: Vec<_> = param_names
            .iter()
            .zip(method.sig.inputs.iter().filter_map(|arg| match arg {
                syn::FnArg::Typed(pt) => Some(pt.ty.as_ref()),
                syn::FnArg::Receiver(_) => None,
            }))
            .enumerate()
            .filter_map(|(index, (name, ty))| {
                crate::type_inspect::self_ref_mutability(ty).map(|mutable| (name, mutable, index))
            })
            .collect();
        let handles: Vec<_> = self_refs
            .iter()
            .map(|(name, mutable, index)| {
                (
                    crate::trait_self_ref::handle_ident(name),
                    *mutable,
                    diagnostic_names[*index].clone(),
                )
            })
            .collect();
        let alias_checks = crate::trait_self_ref::alias_checks(
            concrete_type,
            &handles,
            Some((crate::trait_self_ref::cell_address(&cell), method.is_mut)),
        );
        let self_borrows = self_refs
            .iter()
            .map(|(name, mutable, _)| crate::trait_self_ref::borrow(name, *mutable));

        // Generate method call using fully-qualified syntax to avoid ambiguity
        // with generic trait paths like `RExtend<i32>::method()` where `<` would
        // be parsed as a comparison operator in expression position.
        // A visibility marker (`Invisible<T>`, #1213) is a transparent newtype
        // around the real return; `method.sig.output` is already peeled, so
        // unwrap it here before the conversion below.
        let unwrap_marker = |call: TokenStream| -> TokenStream {
            let call =
                crate::type_inspect::prepare_return_value(call, method.invisible, method.serialize);
            match &method.return_wrap {
                Some(wrap) => wrap.prepare_value(call),
                None => call,
            }
        };
        let invocation = unwrap_marker(quote::quote! {
            <#concrete_type as #trait_path>::#method_ident(#self_ref, #(#param_names),*)
        });
        // The receiver borrows the value's RefCell for the rest of the shim,
        // through the result conversion.
        let receiver = if let Some(kind) = owner_kind {
            let handle = kind.type_path();
            quote::quote! {
                let #self_ref = unsafe {
                    #handle::<#concrete_type>::from_raw_parts(
                        &#owner,
                        #cell,
                    )
                };
            }
        } else {
            crate::trait_self_ref::receiver_borrow(&cell, &guard, &self_ref, method.is_mut)
        };

        // Generate result conversion
        let has_return = match &method.sig.output {
            syn::ReturnType::Default => false,
            syn::ReturnType::Type(_, ty) => {
                !matches!(ty.as_ref(), syn::Type::Tuple(t) if t.elems.is_empty())
            }
        };
        let result_conversion = if has_return {
            quote::quote! {
                unsafe { ::miniextendr_api::trait_abi::to_sexp(result) }
            }
        } else {
            quote::quote! {
                let _ = result;
                unsafe { ::miniextendr_api::trait_abi::nil() }
            }
        };

        let method_name_str = format!("{}::{}", trait_name, method_ident);

        shims.push(quote::quote! {
            #[doc(hidden)]
            #[allow(non_snake_case)]
            unsafe extern "C" fn #shim_name(
                #data: *mut ::std::os::raw::c_void,
                #owner_parameter
                #argc: i32,
                #argv: *const ::miniextendr_api::SEXP,
            ) -> ::miniextendr_api::SEXP {
                // with_r_unwind_protect_shim: returns tagged error SEXP on panic
                // so the View method wrapper can re-panic via repanic_if_rust_error,
                // allowing rust_* class layering (issue #345).
                ::miniextendr_api::unwind_protect::with_r_unwind_protect_shim(|| {
                    ::miniextendr_api::trait_abi::check_arity(#argc, #expected_argc, #method_name_str);
                    let #cell = unsafe { &*#data.cast::<::core::cell::RefCell<#concrete_type>>() };
                    #native_borrow_checks
                    #(#arg_extractions)*
                    #alias_checks
                    #(#self_borrows)*
                    #receiver
                    let result = #invocation;
                    #result_conversion
                })
            }
        });
    }

    quote::quote! { #(#shims)* }
}

/// Inherit only declaration-level R naming/export metadata. The implementation
/// owns its execution options, docs and signatures; an explicit impl rename
/// wins, while a declaration's skipped method is never exported accidentally.
pub(super) fn inherit_method_metadata(
    implementation: &mut ItemImpl,
    declaration: &[syn::TraitItemFn],
) -> syn::Result<()> {
    for item in &mut implementation.items {
        let syn::ImplItem::Fn(method) = item else {
            continue;
        };
        let Some(declared) = declaration
            .iter()
            .find(|declared| declared.sig.ident == method.sig.ident)
        else {
            continue; // rustc diagnoses methods that do not belong to the trait.
        };
        let inherited = parse_trait_method_attrs(&declared.attrs)?;
        let own = parse_trait_method_attrs(&method.attrs)?;
        if own.r_name.is_none()
            && let Some(r_name) = inherited.r_name
        {
            method
                .attrs
                .push(syn::parse_quote!(#[miniextendr(r_name = #r_name)]));
        }
        if inherited.skip && !own.skip {
            method.attrs.push(syn::parse_quote!(#[miniextendr(skip)]));
        }
    }
    Ok(())
}

/// Extract all methods from a trait impl block as [`TraitMethod`] structs.
///
/// Parses each `ImplItem::Fn` to determine receiver type, mutability,
/// `#[miniextendr(...)]` attributes (coerce, skip, r_name, defaults, etc.),
/// and roxygen `@param` tags from doc comments. `impl_page` is whether the
/// impl block renders a help page: the methods' docs keep no links when it
/// does not.
pub(super) fn extract_methods(
    impl_item: &ItemImpl,
    impl_page: crate::roxygen::RdPage,
) -> syn::Result<Vec<TraitMethod>> {
    let mut methods = Vec::new();
    for item in &impl_item.items {
        if let syn::ImplItem::Fn(method) = item {
            // Check receiver type
            let (has_self, is_mut) = method.sig.inputs.first().map_or((false, false), |arg| {
                if let syn::FnArg::Receiver(_) = arg {
                    (true, crate::trait_receiver::is_mut(&method.sig))
                } else {
                    (false, false)
                }
            });
            // The unforced dots are no trait-method parameter (#1892).
            crate::miniextendr_fn::refuse_lazy_dots_in_trait_method(&method.sig.inputs)?;
            let mut attrs = parse_trait_method_attrs(&method.attrs)?;
            // Every parameter `choices(...)` / `inherits(...)` /
            // `not_inherits(...)` / `no_na(...)` name must exist, as on
            // inherent methods; `Option<T>` scalar
            // choices params are the optional form (#1473).
            crate::miniextendr_fn::finalize_method_param_attrs(
                &mut attrs.per_param,
                &method.sig.inputs,
                &[],
                &attrs.defaults,
                attrs
                    .per_param_span
                    .unwrap_or_else(|| method.sig.ident.span()),
            )?;
            check_trait_method_defaults(
                &attrs.defaults,
                &method.sig.inputs,
                attrs
                    .defaults_span
                    .unwrap_or_else(|| method.sig.ident.span()),
            )?;

            // The author's tags; the R wrapper generators pick the `@param`
            // lines and the page tags from them (every tag but `@title` and a
            // bare `@export` on blocks without `\usage`,
            // `r_wrappers::forwarded_body_tag`).
            let doc_tags = crate::roxygen::roxygen_tags_from_attrs(&method.attrs, impl_page);

            // Validate and peel a return-visibility marker (#1213): the
            // codegen sees the inner type, `.0` unwraps the value.
            if let syn::ReturnType::Type(_, ty) = &method.sig.output
                && let Some(err) = crate::type_inspect::visibility_marker_error(ty, "return")
            {
                return Err(err);
            }
            let (invisible, output) =
                crate::type_inspect::peel_return_visibility(&method.sig.output);
            let mut sig = method.sig.clone();
            // Wrappers call the method positionally; patterns belong only to
            // the original impl body. Give every wrapper argument a binding,
            // including ignored and destructured parameters (#2029).
            let mut used_names: std::collections::HashSet<_> = sig
                .inputs
                .iter()
                .filter_map(|arg| match arg {
                    syn::FnArg::Typed(arg) => match arg.pat.as_ref() {
                        syn::Pat::Ident(pat) => Some(crate::naming::ident_name(&pat.ident)),
                        _ => None,
                    },
                    _ => None,
                })
                .collect();
            for (index, arg) in sig
                .inputs
                .iter_mut()
                .filter_map(|arg| match arg {
                    syn::FnArg::Typed(arg) => Some(arg),
                    _ => None,
                })
                .enumerate()
            {
                if !matches!(arg.pat.as_ref(), syn::Pat::Ident(_)) {
                    let mut name = format!("arg{index}");
                    while !used_names.insert(name.clone()) {
                        name.push('_');
                    }
                    let name = format_ident!("{name}");
                    arg.pat = syn::parse_quote!(#name);
                }
            }
            let (return_wrap, output) = crate::return_wrap::resolve(&output, attrs.wrap)?;
            if return_wrap
                .as_ref()
                .is_some_and(|wrap| wrap.conversion.is_some())
            {
                return Err(syn::Error::new_spanned(
                    &method.sig.output,
                    "ConvertTo/ConvertFrom require an inherent S7 impl method",
                ));
            }
            if return_wrap.is_some() && (attrs.serialize || attrs.unwrap_in_r) {
                return Err(syn::Error::new_spanned(
                    &method.sig.output,
                    "explicit class wrapping cannot be combined with serialize or unwrap_in_r",
                ));
            }
            sig.output = crate::type_inspect::serialize_return_type(&output, attrs.serialize);

            methods.push(TraitMethod {
                ident: method.sig.ident.clone(),
                sig,
                invisible,
                has_self,
                is_mut,
                worker: attrs.worker,
                unsafe_main_thread: attrs.unsafe_main_thread,
                coerce: attrs.coerce,
                check_interrupt: attrs.check_interrupt,
                rng: attrs.rng,
                unwrap_in_r: attrs.unwrap_in_r,
                serialize: attrs.serialize,
                return_wrap,
                param_defaults: attrs.defaults,
                doc_tags,
                skip: attrs.skip,
                r_name: attrs.r_name,
                strict: attrs.strict,
                lifecycle: attrs.lifecycle,
                r_entry: attrs.r_entry,
                r_post_checks: attrs.r_post_checks,
                r_on_exit: attrs.r_on_exit,
                no_shortcut: attrs.no_shortcut,
                generic: attrs.generic,
                receiver: attrs.receiver,
                no_dispatch_dots: attrs.no_dispatch_dots,
                per_param: attrs.per_param,
                preconditions: attrs.preconditions,
            });
        }
    }
    Ok(methods)
}

/// Reject a `defaults(...)` entry that names no parameter of the trait
/// method, with the inherent-impl wording (a typo would otherwise drop the
/// default without a word).
fn check_trait_method_defaults(
    defaults: &std::collections::HashMap<String, String>,
    inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
    span: proc_macro2::Span,
) -> syn::Result<()> {
    let names: std::collections::HashSet<String> = inputs
        .iter()
        .filter_map(|arg| match arg {
            syn::FnArg::Typed(pt) => match pt.pat.as_ref() {
                syn::Pat::Ident(pat_ident) => Some(crate::naming::ident_name(&pat_ident.ident)),
                _ => None,
            },
            syn::FnArg::Receiver(_) => None,
        })
        .collect();
    let mut invalid: Vec<&str> = defaults
        .keys()
        .map(String::as_str)
        .filter(|key| !names.contains(key.strip_prefix("r#").unwrap_or(key)))
        .collect();
    if invalid.is_empty() {
        return Ok(());
    }
    invalid.sort_unstable();
    Err(syn::Error::new(
        span,
        format!(
            "defaults(...) references non-existent parameter(s): {}",
            invalid.join(", ")
        ),
    ))
}

/// Parsed `#[miniextendr(...)]` attributes for a single trait method.
///
/// Extracted from method-level attributes to control C wrapper behavior,
/// threading, and R wrapper generation.
struct TraitMethodAttrs {
    /// Dispatch to worker thread. Set by explicit `#[miniextendr(worker)]` or `worker-default` feature.
    worker: bool,
    /// Force execution on R's main thread (overrides explicit or feature-selected worker dispatch).
    unsafe_main_thread: bool,
    /// Enable `Rf_coerceVector` for all parameters. Set by `coerce` /
    /// `no_coerce`, else the `coerce-default` feature.
    coerce: bool,
    /// Call `R_CheckUserInterrupt` before the method body.
    check_interrupt: bool,
    /// Wrap the call in `GetRNGstate`/`PutRNGstate` for reproducible random number generation.
    rng: bool,
    /// Return `Result<T, E>` to R without unwrapping (R wrapper receives the result variant).
    unwrap_in_r: bool,
    /// Serialize the complete return value through `AsSerialize<T>`.
    serialize: bool,
    wrap: Option<ClassSystem>,
    /// Exclude this method from all generated wrappers (C, R, vtable shims).
    skip: bool,
    /// Parameter default values: keys are parameter names, values are R expressions.
    defaults: std::collections::HashMap<String, String>,
    /// Override the R-facing method name.
    r_name: Option<String>,
    /// Strict output conversion: panic instead of lossy widening for i64/u64/isize/usize.
    /// Set by `strict` / `no_strict`, else the `strict-default` feature.
    strict: bool,
    /// Lifecycle specification for deprecation/experimental status.
    lifecycle: Option<crate::lifecycle::LifecycleSpec>,
    /// R code to inject at the very top of the wrapper body.
    r_entry: Option<String>,
    /// R code to inject after all checks, immediately before `.Call()`.
    r_post_checks: Option<String>,
    /// Register `on.exit()` cleanup code in the R wrapper.
    r_on_exit: Option<crate::miniextendr_fn::ROnExit>,
    /// Opt out of the S7 fast-dispatch shortcut (`#[miniextendr(s7(no_shortcut))]`).
    no_shortcut: bool,
    generic: Option<String>,
    receiver: Option<String>,
    no_dispatch_dots: bool,
    /// The method's bare `preconditions` / `no_preconditions` (#1017); the
    /// last one written wins. See `TraitMethod::preconditions`.
    preconditions: Option<bool>,
    /// Per-parameter `match_arg`/`choices`/`several_ok` attributes, keyed by
    /// Rust parameter name. See `TraitMethod::per_param`.
    per_param: std::collections::HashMap<String, crate::miniextendr_fn::ParamAttrs>,
    /// Span of the first `choices` / `choices_several_ok` / `no_default` /
    /// `no_na` / `inherits` / `not_inherits` option: where an unknown
    /// parameter name is reported.
    per_param_span: Option<proc_macro2::Span>,
    /// Span of the `defaults(...)` option: where an unknown parameter name in
    /// it is reported.
    defaults_span: Option<proc_macro2::Span>,
}

/// Parse `#[miniextendr(...)]` attributes from a trait method.
///
/// Supports two syntax styles:
/// - **Flat**: `#[miniextendr(worker, coerce, rng)]`
/// - **Nested class-system**: `#[miniextendr(env(worker, coerce))]`
///
/// Both styles can coexist. The `worker` flag controls whether static methods
/// dispatch to the worker thread (defaults to `cfg!(feature = "worker-default")`).
/// `coerce` and `strict` default to the `coerce-default` / `strict-default`
/// features, as on inherent methods, and `no_coerce` / `no_strict` opt out.
fn parse_trait_method_attrs(attrs: &[syn::Attribute]) -> syn::Result<TraitMethodAttrs> {
    use syn::spanned::Spanned;
    let mut worker = false;
    let mut unsafe_main_thread = false;
    let mut coerce: Option<bool> = None;
    let mut check_interrupt = false;
    let mut rng = false;
    let mut unwrap_in_r = false;
    let mut serialize = false;
    let mut wrap = None;
    let mut skip = false;
    let mut strict: Option<bool> = None;
    let mut defaults = std::collections::HashMap::new();
    let mut r_name: Option<String> = None;
    let mut lifecycle: Option<crate::lifecycle::LifecycleSpec> = None;
    let mut r_entry: Option<String> = None;
    let mut r_post_checks: Option<String> = None;
    let mut r_on_exit: Option<crate::miniextendr_fn::ROnExit> = None;
    let mut no_shortcut = false;
    let mut generic = None;
    let mut receiver = None;
    let mut no_dispatch_dots = false;
    let mut preconditions: Option<bool> = None;
    let mut per_param: std::collections::HashMap<String, crate::miniextendr_fn::ParamAttrs> =
        std::collections::HashMap::new();
    let mut per_param_span: Option<proc_macro2::Span> = None;
    let mut defaults_span: Option<proc_macro2::Span> = None;

    for attr in attrs {
        if !attr.path().is_ident("miniextendr") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            let is_class_meta = meta.path.is_ident("env")
                || meta.path.is_ident("r6")
                || meta.path.is_ident("s7")
                || meta.path.is_ident("s3")
                || meta.path.is_ident("s4");

            if is_class_meta {
                meta.parse_nested_meta(|inner| {
                    if inner.path.is_ident("worker") {
                        worker = true;
                        crate::deprecation::worker_option(inner.path.span());
                    } else if inner.path.is_ident("main_thread") {
                        unsafe_main_thread = true;
                    } else if inner.path.is_ident("coerce") {
                        coerce = Some(true);
                    } else if inner.path.is_ident("no_coerce") {
                        coerce = Some(false);
                    } else if inner.path.is_ident("check_interrupt") {
                        check_interrupt = true;
                    } else if inner.path.is_ident("unwrap_in_r") {
                        unwrap_in_r = true;
                    } else if inner.path.is_ident("wrap") {
                        let value: syn::LitStr = inner.value()?.parse()?;
                        wrap = Some(crate::return_wrap::parse_system(&value)?);
                    } else if inner.path.is_ident("serialize") {
                        serialize = true;
                    } else if inner.path.is_ident("no_shortcut") {
                        no_shortcut = true;
                    } else if (meta.path.is_ident("s3") || meta.path.is_ident("s4"))
                        && inner.path.is_ident("generic")
                    {
                        let value: syn::LitStr = inner.value()?.parse()?;
                        if generic.replace(value.value()).is_some() {
                            return Err(inner.error("generic may be specified only once"));
                        }
                    } else if (meta.path.is_ident("s3") || meta.path.is_ident("s4"))
                        && inner.path.is_ident("receiver")
                    {
                        let value: syn::LitStr = inner.value()?.parse()?;
                        if receiver.replace(crate::naming::parse_r_receiver(&value)?).is_some() {
                            return Err(inner.error("receiver may be specified only once"));
                        }
                    } else if (meta.path.is_ident("s3") || meta.path.is_ident("s4"))
                        && inner.path.is_ident("no_dots")
                    {
                        no_dispatch_dots = true;
                    } else {
                        return Err(inner.error(
                            "unknown nested option; expected `worker`, `main_thread`, `coerce`, \
                             `no_coerce`, `check_interrupt`, `unwrap_in_r`, `serialize`, or \
                             `no_shortcut` (S3/S4 also accept `generic`, `receiver`, `no_dots`)",
                        ));
                    }
                    Ok(())
                })?;
            } else if meta.path.is_ident("worker") {
                worker = true;
                crate::deprecation::worker_option(meta.path.span());
            } else if meta.path.is_ident("main_thread") {
                unsafe_main_thread = true;
            } else if meta.path.is_ident("coerce") {
                coerce = Some(true);
            } else if meta.path.is_ident("no_coerce") {
                coerce = Some(false);
            } else if meta.path.is_ident("check_interrupt") {
                check_interrupt = true;
            } else if meta.path.is_ident("rng") {
                rng = true;
            } else if meta.path.is_ident("unwrap_in_r") {
                unwrap_in_r = true;
            } else if meta.path.is_ident("wrap") {
                let value: syn::LitStr = meta.value()?.parse()?;
                wrap = Some(crate::return_wrap::parse_system(&value)?);
            } else if meta.path.is_ident("serialize") {
                serialize = true;
            } else if meta.path.is_ident("skip") {
                skip = true;
            } else if meta.path.is_ident("no_shortcut") {
                no_shortcut = true;
            } else if meta.path.is_ident("r_name") {
                let value: syn::LitStr = meta.value()?.parse()?;
                r_name = Some(value.value());
            } else if meta.path.is_ident("strict") {
                strict = Some(true);
            } else if meta.path.is_ident("no_strict") {
                strict = Some(false);
            } else if meta.path.is_ident("defaults") {
                // Parse defaults(param = "value", param2 = "value2", ...)
                defaults_span.get_or_insert(meta.path.span());
                meta.parse_nested_meta(|inner| {
                    let param_name = inner
                        .path
                        .get_ident()
                        .map(|i| i.to_string())
                        .unwrap_or_default();
                    let value: syn::LitStr = inner.value()?.parse()?;
                    defaults.insert(param_name, value.value());
                    Ok(())
                })?;
            } else if meta.path.is_ident("lifecycle") {
                if meta.input.peek(syn::Token![=]) {
                    // lifecycle = "stage"
                    let _: syn::Token![=] = meta.input.parse()?;
                    let value: syn::LitStr = meta.input.parse()?;
                    let stage = crate::lifecycle::LifecycleStage::from_str(&value.value())
                        .ok_or_else(|| {
                            syn::Error::new(
                                value.span(),
                                "invalid lifecycle stage; expected one of: experimental, stable, superseded, soft-deprecated, deprecated, defunct",
                            )
                        })?;
                    lifecycle = Some(crate::lifecycle::LifecycleSpec::new(stage));
                } else {
                    // lifecycle(stage = "deprecated", when = "0.4.0", ...)
                    let mut spec = crate::lifecycle::LifecycleSpec::default();
                    meta.parse_nested_meta(|inner| {
                        let key = inner.path.get_ident()
                            .ok_or_else(|| inner.error("expected identifier"))?
                            .to_string();
                        let _: syn::Token![=] = inner.input.parse()?;
                        let value: syn::LitStr = inner.input.parse()?;
                        match key.as_str() {
                            "stage" => {
                                spec.stage = crate::lifecycle::LifecycleStage::from_str(&value.value())
                                    .ok_or_else(|| syn::Error::new(value.span(), "invalid lifecycle stage"))?;
                            }
                            "when" => spec.when = Some(value.value()),
                            "what" => spec.what = Some(value.value()),
                            "with" => spec.with = Some(value.value()),
                            "details" => spec.details = Some(value.value()),
                            "id" => spec.id = Some(value.value()),
                            _ => return Err(inner.error(
                                "unknown lifecycle option; expected: stage, when, what, with, details, id"
                            )),
                        }
                        Ok(())
                    })?;
                    lifecycle = Some(spec);
                }
            } else if meta.path.is_ident("r_entry") {
                let _: syn::Token![=] = meta.input.parse()?;
                let value: syn::LitStr = meta.input.parse()?;
                r_entry = Some(value.value());
            } else if meta.path.is_ident("r_post_checks") {
                let _: syn::Token![=] = meta.input.parse()?;
                let value: syn::LitStr = meta.input.parse()?;
                r_post_checks = Some(value.value());
            } else if meta.path.is_ident("r_on_exit") {
                if meta.input.peek(syn::Token![=]) {
                    // Short form: r_on_exit = "expr"
                    let _: syn::Token![=] = meta.input.parse()?;
                    let value: syn::LitStr = meta.input.parse()?;
                    r_on_exit = Some(crate::miniextendr_fn::ROnExit {
                        expr: value.value(),
                        add: true,
                        after: true,
                    });
                } else {
                    // Long form: r_on_exit(expr = "...", add = false, after = false)
                    let mut expr = None;
                    let mut add = true;
                    let mut after = true;
                    meta.parse_nested_meta(|inner| {
                        if inner.path.is_ident("expr") {
                            let _: syn::Token![=] = inner.input.parse()?;
                            let value: syn::LitStr = inner.input.parse()?;
                            expr = Some(value.value());
                        } else if inner.path.is_ident("add") {
                            let _: syn::Token![=] = inner.input.parse()?;
                            let value: syn::LitBool = inner.input.parse()?;
                            add = value.value;
                        } else if inner.path.is_ident("after") {
                            let _: syn::Token![=] = inner.input.parse()?;
                            let value: syn::LitBool = inner.input.parse()?;
                            after = value.value;
                        } else {
                            return Err(inner.error(
                                "unknown r_on_exit option; expected `expr`, `add`, or `after`",
                            ));
                        }
                        Ok(())
                    })?;
                    let expr = expr.ok_or_else(|| {
                        meta.error("r_on_exit(...) requires `expr = \"...\"` specifying the R expression")
                    })?;
                    r_on_exit = Some(crate::miniextendr_fn::ROnExit { expr, add, after });
                }
            } else if meta.path.is_ident("choices") {
                per_param_span.get_or_insert(meta.path.span());
                // `choices(param = "a, b, c", param2 = "x, y")` — explicit string choice lists.
                //
                // NOTE: `match_arg`/`match_arg_several_ok` (choices derived from a
                // `#[derive(MatchArg)]` enum's `CHOICES` const) are intentionally NOT
                // supported here yet. The inherent-impl path additionally emits a
                // `MX_MATCH_ARG_CHOICES` registration
                // (`generate_method_match_arg_entries` in
                // `miniextendr_impl.rs`) that resolves the write-time placeholder
                // formal default into the enum's literal choice list — trait methods
                // have no equivalent metadata registration yet, so accepting `match_arg`
                // here would silently ship a formal default that never resolves
                // (`object '.__MX_MATCH_ARG_CHOICES_...__' not found` at call time).
                // `choices(...)` needs no such resolution (the literal list is baked in
                // at macro-expansion time), so it's safe to support today. Tracked in
                // the trait-method-emitter follow-up issue for full match_arg parity.
                meta.parse_nested_meta(|inner| {
                    let name = crate::miniextendr_fn::method_check_param(&inner)?;
                    let _: syn::Token![=] = inner.input.parse()?;
                    let value: syn::LitStr = inner.input.parse()?;
                    let choices = crate::r_wrapper_builder::split_choice_list(&value.value());
                    per_param.entry(name).or_default().choices = Some(choices);
                    Ok(())
                })?;
            } else if meta.path.is_ident("choices_several_ok") {
                per_param_span.get_or_insert(meta.path.span());
                // `choices_several_ok(param = "a, b, c")` — choices + several_ok.
                meta.parse_nested_meta(|inner| {
                    let name = crate::miniextendr_fn::method_check_param(&inner)?;
                    let _: syn::Token![=] = inner.input.parse()?;
                    let value: syn::LitStr = inner.input.parse()?;
                    let choices = crate::r_wrapper_builder::split_choice_list(&value.value());
                    let entry = per_param.entry(name).or_default();
                    entry.choices = Some(choices);
                    entry.several_ok = true;
                    Ok(())
                })?;
            } else if meta.path.is_ident("no_default") {
                per_param_span.get_or_insert(meta.path.span());
                // `no_default(p, q)` — choice params whose R formal has no
                // default (#1828).
                crate::miniextendr_fn::parse_method_no_default(&meta, &mut per_param)?;
            } else if meta.path.is_ident("preconditions") || meta.path.is_ident("no_preconditions")
            {
                // Bare: the whole method keeps / drops its type-derived R-side
                // checks. `preconditions(p)` / `no_preconditions(p)`: those
                // parameters. The last one written wins.
                let keep = meta.path.is_ident("preconditions");
                if meta.input.peek(syn::token::Paren) {
                    per_param_span.get_or_insert(meta.path.span());
                    crate::miniextendr_fn::parse_method_preconditions(&meta, &mut per_param, keep)?;
                } else {
                    preconditions = Some(keep);
                }
            } else if meta.path.is_ident("no_na") {
                per_param_span.get_or_insert(meta.path.span());
                // `no_na(p, q(message = "..."))` — R-side `!anyNA(p)` checks.
                crate::miniextendr_fn::parse_method_no_na(&meta, &mut per_param)?;
            } else if let Some(check) = crate::miniextendr_fn::ClassCheck::of(&meta.path) {
                per_param_span.get_or_insert(meta.path.span());
                // `inherits(p = "cls_a, cls_b", q(class = "cls", message = "..."))` —
                // R-side `inherits(p, c(...))` checks; `not_inherits(...)` the
                // same shape, `!inherits(p, c(...))`.
                crate::miniextendr_fn::parse_method_class_check(&meta, &mut per_param, check)?;
            } else {
                return Err(meta.error(
                    "unknown #[miniextendr] option on trait impl method; expected one of: \
                     `env`, `r6`, `s7`, `s3`, `s4`, `worker`, `main_thread`, `coerce`, `no_coerce`, \
                     `check_interrupt`, `rng`, `unwrap_in_r`, `serialize`, `skip`, `no_shortcut`, `r_name`, \
                     `defaults`, `strict`, `no_strict`, `lifecycle`, `r_entry`, `r_post_checks`, `r_on_exit`, \
                     `choices`, `choices_several_ok`, `no_default`, `inherits`, `not_inherits`, `no_na`, \
                     `preconditions`, `no_preconditions`",
                ));
            }
            Ok(())
        })?;
    }

    if serialize && unwrap_in_r {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "`serialize` cannot be combined with `unwrap_in_r`: it serializes the complete return value, including any Result variant",
        ));
    }

    Ok(TraitMethodAttrs {
        worker: worker || cfg!(feature = "worker-default"),
        unsafe_main_thread,
        coerce: coerce.unwrap_or(cfg!(feature = "coerce-default")),
        check_interrupt,
        rng,
        unwrap_in_r,
        serialize,
        wrap,
        skip,
        strict: strict.unwrap_or(cfg!(feature = "strict-default")),
        defaults,
        r_name,
        lifecycle,
        r_entry,
        r_post_checks,
        r_on_exit,
        no_shortcut,
        generic,
        receiver,
        no_dispatch_dots,
        preconditions,
        per_param,
        per_param_span,
        defaults_span,
    })
}

/// Extract associated constant items from a trait impl block.
///
/// Each `const NAME: Type = value;` in the impl block becomes a [`TraitConst`]
/// that will get its own zero-argument C wrapper for R access.
fn extract_consts(impl_item: &ItemImpl) -> Vec<TraitConst> {
    impl_item
        .items
        .iter()
        .filter_map(|item| {
            if let syn::ImplItem::Const(const_item) = item {
                Some(TraitConst {
                    ident: const_item.ident.clone(),
                    ty: const_item.ty.clone(),
                })
            } else {
                None
            }
        })
        .collect()
}

/// Generate a C wrapper function and `R_CallMethodDef` for a single trait method.
///
/// Uses `CWrapperContext` builder to produce:
/// - An `extern "C-unwind"` function callable from R via `.Call()`
/// - A `R_CallMethodDef` constant for symbol registration
///
/// Instance methods (`has_self`) extract the object via `ErasedExternalPtr::from_sexp`
/// and call with fully-qualified trait syntax `<Type as Trait>::method(self_ref, ...)`.
/// Static methods run on the worker thread when the `worker` flag is set.
///
/// `&Self` / `&mut Self` parameters are borrowed external arguments (#2106): the C wrapper
/// converts each to its `ExternalPtr<Type>` handle and borrows it with the declared
/// mutability for the call, after checking that no two of them alias while one is mutable.
pub(super) fn generate_trait_method_c_wrapper(
    method: &TraitMethod,
    type_ident: &syn::Ident,
    trait_name: &syn::Ident,
    trait_path: &syn::Path,
) -> TokenStream {
    use crate::c_wrapper_builder::{CWrapperContext, ReturnHandling, ThreadStrategy};

    let method_ident = &method.ident;
    let c_ident = method.c_wrapper_ident(type_ident, trait_name);
    let call_method_def_ident = method.call_method_def_ident(type_ident, trait_name);

    // Thread strategy: instance methods stay on main thread (self_ref can't cross threads);
    // static methods use worker thread only when worker=true (explicit or worker-default feature)
    // and, as for standalone functions, no R value crosses the call and no
    // `check_interrupt` asks for R's interrupt check.
    let thread_strategy = if method.has_self
        || method.unsafe_main_thread
        || method.check_interrupt
        || crate::type_inspect::signature_holds_r_values(&method.sig)
    {
        ThreadStrategy::MainThread
    } else if method.worker {
        ThreadStrategy::WorkerThread
    } else {
        ThreadStrategy::MainThread
    };

    // Build rust argument names from the signature (excluding self receiver)
    let rust_args: Vec<syn::Ident> = method
        .sig
        .inputs
        .iter()
        .filter_map(|arg| {
            if let syn::FnArg::Typed(pt) = arg {
                if let syn::Pat::Ident(pat_ident) = pt.pat.as_ref() {
                    Some(pat_ident.ident.clone())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    // Filter inputs to exclude the receiver (builder handles self separately with has_self()).
    // `&Self` / `&mut Self` parameters stay: the builder converts them to their handles and
    // borrows them for the call (#2106). Track them for the alias checks.
    let mut self_ref_params = Vec::new();
    let filtered_inputs: syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]> = method
        .sig
        .inputs
        .iter()
        .filter(|arg| !matches!(arg, syn::FnArg::Receiver(_)))
        .inspect(|arg| {
            if let syn::FnArg::Typed(pt) = arg
                && let Some(mutable) = crate::type_inspect::self_ref_mutability(&pt.ty)
                && let syn::Pat::Ident(pat_ident) = pt.pat.as_ref()
            {
                self_ref_params.push((pat_ident.ident.clone(), mutable));
            }
        })
        .cloned()
        .collect();
    let call_args = &rust_args;

    // Determine return handling
    let return_handling = if method.unwrap_in_r
        && crate::return_type_analysis::output_is_result(&method.sig.output)
    {
        ReturnHandling::IntoR
    } else {
        crate::c_wrapper_builder::detect_return_handling(&method.sig.output)
    };

    // Generate R wrapper const name (not actually used but needed by builder)
    let r_wrappers_const = crate::naming::trait_impl_wrappers_ident(
        &crate::naming::ident_name(type_ident),
        trait_name,
    );

    // Build the wrapper using the builder infrastructure
    // Use custom call_method_def_ident to avoid collisions with inherent impl methods
    // Messages R sees name the method as R calls it (`r_name`).
    let mut builder = CWrapperContext::builder(method_ident.clone(), c_ident)
        .r_name(method.r_method_name())
        .r_wrapper_const(r_wrappers_const)
        .inputs(filtered_inputs)
        .output(method.sig.output.clone())
        .thread_strategy(thread_strategy)
        .return_handling(return_handling)
        .type_context(type_ident.clone())
        .call_method_def_ident(call_method_def_ident);

    let mut post_conversion = Vec::new();
    let handles: Vec<_> = self_ref_params
        .iter()
        .map(|(name, mutable)| {
            (
                crate::externalptr_args::handle_ident(name),
                *mutable,
                crate::naming::ident_name(name),
            )
        })
        .collect();
    let receiver = method
        .has_self
        .then(|| (quote::quote!(__miniextendr_receiver_addr), method.is_mut));
    post_conversion.push(crate::trait_self_ref::alias_checks(
        type_ident, &handles, receiver,
    ));

    if method.has_self {
        let self_sexp = crate::naming::self_sexp_ident();
        let owner = syn::Ident::new("__mx_owner", proc_macro2::Span::mixed_site());
        let self_ptr = syn::Ident::new("self_ptr", proc_macro2::Span::mixed_site());
        let self_ref = syn::Ident::new("self_ref", proc_macro2::Span::mixed_site());
        let cell = syn::Ident::new("__mx_cell", proc_macro2::Span::mixed_site());
        let guard = syn::Ident::new("__mx_guard", proc_macro2::Span::mixed_site());
        let resolve = quote::quote! {
            let #owner =
                ::miniextendr_api::externalptr::ReceiverOwner::resolve::<#type_ident>(#self_sexp);
        };
        let self_extraction = if let Some(kind) = crate::trait_receiver::owner_receiver(&method.sig)
        {
            let receiver_mutability = method.is_mut.then(|| quote::quote!(mut));
            let borrow = match kind {
                crate::trait_receiver::OwnerReceiver::Shared => quote::quote!(borrow),
                crate::trait_receiver::OwnerReceiver::Mutable => quote::quote!(borrow_mut),
            };
            let address = (!self_ref_params.is_empty()).then(|| quote::quote! {
                let __miniextendr_receiver_addr = #self_ptr.as_ptr().cast::<::core::ffi::c_void>();
            });
            // The alias preflight above runs before constructing either an
            // exclusive view or any &Self argument reference. The view holds
            // the value's RefCell borrow for the rest of the call.
            post_conversion.push(quote::quote! {
                let #self_ref = #self_ptr.#borrow();
            });
            quote::quote! {
                #resolve
                let #receiver_mutability #self_ptr = unsafe {
                    ::miniextendr_api::externalptr::wrap_receiver::<#type_ident>(#owner.get())
                };
                #address
            }
        } else {
            let trait_method_name = format!("{}::{}()", trait_name, method.r_method_name());
            // Borrow after argument conversion and alias preflight: an R
            // error that longjmps out of a conversion would skip the
            // borrow's release.
            post_conversion.push(crate::trait_self_ref::receiver_borrow(
                &cell,
                &guard,
                &self_ref,
                method.is_mut,
            ));
            let self_setup = (!self_ref_params.is_empty()).then(|| {
                let address = crate::trait_self_ref::cell_address(&cell);
                quote::quote! {
                    let __miniextendr_receiver_addr = #address;
                }
            });
            quote::quote! {
                #resolve
                let #self_ptr = unsafe {
                    ::miniextendr_api::externalptr::ErasedExternalPtr::from_sexp(#owner.get())
                };
                let #cell = match #self_ptr.downcast_cell::<#type_ident>() {
                    ::core::option::Option::Some(cell) => cell,
                    ::core::option::Option::None => ::miniextendr_api::externalptr::handle_downcast_failed_in::<#type_ident>(
                        &#self_ptr, #trait_method_name,
                    ),
                };
                #self_setup
            }
        };

        // Call expression with self_ref and dereferenced &Self params
        // Use fully-qualified syntax to avoid ambiguity with generic traits
        // (e.g., `RExtend<i32>::method()` is ambiguous — `<` parsed as comparison)
        let call_expr = quote::quote! {
            <#type_ident as #trait_path>::#method_ident(#self_ref, #(#call_args),*)
        };
        // Unwrap a visibility-marker newtype (#1213); the output type is peeled.
        let call_expr = crate::type_inspect::prepare_return_value(
            call_expr,
            method.invisible,
            method.serialize,
        );
        let call_expr = match &method.return_wrap {
            Some(wrap) => wrap.prepare_value(call_expr),
            None => call_expr,
        };

        builder = builder
            .pre_call(vec![self_extraction])
            .call_expr(call_expr)
            .has_self();
    } else {
        // Static method: call directly without self
        let call_expr = quote::quote! {
            <#type_ident as #trait_path>::#method_ident(#(#call_args),*)
        };
        let call_expr = crate::type_inspect::prepare_return_value(
            call_expr,
            method.invisible,
            method.serialize,
        );
        let call_expr = match &method.return_wrap {
            Some(wrap) => wrap.prepare_value(call_expr),
            None => call_expr,
        };

        builder = builder.call_expr(call_expr);
    }

    // Apply coerce_all if the method has #[miniextendr(coerce)]
    if method.coerce {
        builder = builder.coerce_all();
    }

    // Apply check_interrupt if the method has #[miniextendr(check_interrupt)]
    if method.check_interrupt {
        builder = builder.check_interrupt();
    }

    // Apply rng if the method has #[miniextendr(rng)]
    if method.rng {
        builder = builder.rng();
    }

    // Apply strict mode for lossy type conversions
    if method.strict {
        builder = builder.strict();
    }

    // A `choices(...)` string under an `Either<.., R>` layer decodes with the
    // choice-or-other split instead of `TryFromSexp` (the other layers of a
    // `choices` string convert through `TryFromSexp` directly).
    // `no_na(..)` parameters also have their converted value checked, for what
    // the type reads as `NA` beyond the R guard's `anyNA()`.
    for (rust_name, attrs) in &method.per_param {
        if let Some(leaf) = attrs.layered_leaf() {
            builder = builder.layered_choice(rust_name.clone(), leaf);
        }
        if attrs.checks.no_na {
            builder = builder.no_na(rust_name.clone(), attrs.checks.no_na_message.clone());
        }
    }

    // The builder generates both the C wrapper and the R_CallMethodDef
    builder.post_conversion(post_conversion).build().generate()
}

/// Generate a C wrapper function and `R_CallMethodDef` for a trait associated constant.
///
/// The generated wrapper takes only the call-attribution slot and returns the constant value
/// converted to SEXP. Uses fully-qualified syntax `<Type as Trait>::CONST`
/// to access the value. Always runs on the main thread.
pub(super) fn generate_trait_const_c_wrapper(
    trait_const: &TraitConst,
    type_ident: &syn::Ident,
    trait_name: &syn::Ident,
    trait_path: &syn::Path,
) -> TokenStream {
    use crate::c_wrapper_builder::{CWrapperContext, ThreadStrategy};

    let const_ident = &trait_const.ident;
    let c_ident = trait_const.c_wrapper_ident(type_ident, trait_name);
    let call_method_def_ident = trait_const.call_method_def_ident(type_ident, trait_name);
    let const_ty = &trait_const.ty;

    // Generate R wrapper const name
    let r_wrappers_const = crate::naming::trait_impl_wrappers_ident(
        &crate::naming::ident_name(type_ident),
        trait_name,
    );

    // Build the call expression to access the const
    let call_expr = quote::quote! {
        <#type_ident as #trait_path>::#const_ident
    };

    // Determine return type handling - we need to convert the const to SEXP
    // The return type is `-> Type` not just `Type`
    let return_type: syn::ReturnType = syn::parse_quote!(-> #const_ty);
    let return_handling = crate::c_wrapper_builder::detect_return_handling(&return_type);

    // Build wrapper - no inputs, just returns the const value
    let builder = CWrapperContext::builder(const_ident.clone(), c_ident)
        .r_wrapper_const(r_wrappers_const)
        .inputs(Default::default()) // no inputs
        .output(return_type)
        .call_expr(call_expr)
        .thread_strategy(ThreadStrategy::MainThread)
        .return_handling(return_handling)
        .type_context(type_ident.clone())
        .call_method_def_ident(call_method_def_ident);

    builder.build().generate()
}
