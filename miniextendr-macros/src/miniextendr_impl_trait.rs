//! # `#[miniextendr]` on trait impls - Trait Implementation Registration
//!
//! This module handles `#[miniextendr]` applied to trait implementations,
//! generating the vtable static for cross-package trait dispatch, plus optional
//! R-callable wrappers for direct method access.
//!
//! ## Overview
//!
//! When `#[miniextendr]` is applied to an `impl Trait for Type` block, it:
//!
//! 1. **Detects the trait** from the impl syntax (no attribute args needed)
//! 2. **Generates vtable static** using declaration-provided layout and callbacks
//! 3. **Generates C wrappers** for each trait method (for R `.Call` access)
//! 4. **Generates R wrapper code** for the trait methods
//! 5. **Passes through** the original impl block unchanged
//!
//! ## Usage
//!
//! ```ignore
//! use miniextendr_api::miniextendr;
//!
//! // The trait must have been defined with #[miniextendr]
//! // which generates __Counter_build_vtable::<T>()
//!
//! struct MyCounter { value: i32 }
//!
//! #[miniextendr]
//! impl Counter for MyCounter {
//!     fn value(&self) -> i32 {
//!         self.value
//!     }
//!     fn increment(&mut self) {
//!         self.value += 1;
//!     }
//!     fn add(&mut self, n: i32) {
//!         self.value += n;
//!     }
//! }
//! ```
//!
//! Generates (conceptually):
//!
//! ```ignore
//! // Original impl block (passed through)
//! impl Counter for MyCounter {
//!     fn value(&self) -> i32 { self.value }
//!     fn increment(&mut self) { self.value += 1; }
//!     fn add(&mut self, n: i32) { self.value += n; }
//! }
//!
//! // Generated vtable static (crate-prefixed for webR cross-package symbol
//! // uniqueness — #1273; example assumes a crate named `mypkg`). Its local
//! // repr(C) type reproduces Counter's declared callback order and types.
//! pub static __VTABLE_mypkg_Counter_FOR_MyCounter: __VTABLE_mypkg_Counter_FOR_MyCounter_Layout =
//!     __VTABLE_mypkg_Counter_FOR_MyCounter_Layout {
//!         value: <MyCounter as Counter>::__Counter_value_shim,
//!         increment: <MyCounter as Counter>::__Counter_increment_shim,
//!         add: <MyCounter as Counter>::__Counter_add_shim,
//!     };
//! ```
//!
//! ## How It Works
//!
//! 1. Parse `impl Trait for Type` to extract trait path and concrete type
//! 2. Generate vtable static name: `__VTABLE_{crate}_{Trait}_FOR_{Type}`
//! 3. Obtain the declaration's ABI tag and ordered callback fields through its metadata macro
//! 4. Initialize the local layout with associated or concrete generic callbacks
//!
//! ## Trait Detection
//!
//! The macro reads the trait directly from the impl syntax:
//!
//! ```ignore
//! #[miniextendr]
//! impl path::to::Counter for MyType { ... }
//! //   ^^^^^^^^^^^^^^^^^ detected automatically
//! ```
//!
//! No extra arguments are needed; the trait path is explicit in the impl syntax.
//!
//! ## Name Generation
//!
//! - **Vtable static**: `__VTABLE_{crate}_{Trait}_FOR_{Type}` (case-preserving;
//!   `{crate}` is the consuming crate's name, prefixed for webR cross-package symbol
//!   uniqueness — #1273)
//! - **Local vtable type**: the static name with `_Layout` appended
//!
//! For `impl foo::Counter for my_mod::MyType` in a crate named `mypkg`:
//! - Static: `__VTABLE_mypkg_Counter_FOR_MyType`
//! - Callback: `<my_mod::MyType as foo::Counter>::__Counter_value_shim`
//!
//! ## Integration with ExternalPtr / TypedExternal
//!
//! The generated vtable static is automatically registered via linkme
//! distributed slices.
//!
//! ```ignore
//! #[derive(ExternalPtr)]
//! struct MyCounter { value: i32 }
//!
//! #[miniextendr]
//! impl Counter for MyCounter { /* ... */ }
//! ```
//!
//! `ExternalPtr<T>` provides the type identity for the external pointer.
//! Trait dispatch is wired automatically.
//!
//! ## Thread Safety
//!
//! The generated vtable is a static constant, safe to access from any thread.
//! Trait shims now mirror inherent impls: instance methods stay on the main
//! thread, while static trait methods run on the worker thread unless
//! `main_thread` is explicitly requested.

use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident};
use syn::ItemImpl;

use crate::miniextendr_impl::{ClassSystem, ImplAttrs};

/// Parsed method from a trait impl block.
///
/// Stores everything needed to generate C wrappers, R wrappers, and vtable
/// shims for a single method in an `impl Trait for Type` block.
#[derive(Debug, Clone)]
struct TraitMethod {
    /// Signature copied from declaration metadata: infer its argument types
    /// from the real method so declaration-local aliases keep their identity.
    inherited_signature: bool,
    /// Rust method identifier (e.g., `value`, `increment`).
    ident: syn::Ident,
    /// Full method signature including self receiver and all parameters. A
    /// return-visibility marker is already peeled (see `invisible`).
    sig: syn::Signature,
    /// Return-type visibility marker (#1213): `Some(true)` for
    /// `-> Invisible<T>`, `Some(false)` for `-> Visible<T>`, stripped from
    /// `sig.output`. The C wrapper unwraps the newtype before conversion.
    invisible: Option<bool>,
    /// Whether the method has a receiver (`&self`, `&mut self`).
    /// False for static/associated methods.
    has_self: bool,
    /// Whether receiver is `&mut self` (vs `&self`). Only meaningful if `has_self` is true.
    is_mut: bool,
    /// When true, dispatches to the worker thread via `run_on_worker`.
    /// Set by explicit `#[miniextendr(worker)]` or the `worker-default` feature flag.
    worker: bool,
    /// When true, forces execution on R's main thread via `unsafe(main_thread)`.
    unsafe_main_thread: bool,
    /// Enable automatic type coercion for all parameters via `Rf_coerceVector`.
    coerce: bool,
    /// Check for R user interrupts (`R_CheckUserInterrupt`) before calling the method.
    check_interrupt: bool,
    /// Enable RNG state management (`GetRNGstate`/`PutRNGstate`) around the call.
    rng: bool,
    /// Return `Result<T, E>` to R without unwrapping -- R wrapper receives the result variant.
    unwrap_in_r: bool,
    /// Serialize the complete return value through `AsSerialize<T>`.
    serialize: bool,
    /// Explicit class-system return wrapping.
    return_wrap: Option<crate::return_wrap::ReturnWrap>,
    /// Parameter default values from `#[miniextendr(defaults(param = "value", ...))]`.
    /// Keys are parameter names, values are R expressions used as default values.
    param_defaults: std::collections::HashMap<String, String>,
    /// The author's roxygen tags from the method's doc comment. The wrapper
    /// generators forward the `@param` lines and the page tags (see
    /// `r_wrappers::own_block_page`): a method-level `@rdname other` or
    /// `@describeIn other ...` puts the method's own R wrapper block (S3
    /// method, S4 `setMethod`, env/R6 namespace member, S7 shortcut, static
    /// function) on that page instead of the type's shared `@rdname <Type>`
    /// page, `@name` replaces the generated topic name, `@order` the generated
    /// order, and `@inheritParams` / `@inherit` / `@inheritDotParams` fill the
    /// arguments. Generics and consts stay on the type page.
    doc_tags: Vec<String>,
    /// When true, this method is excluded from C wrappers, R wrappers, and vtable shims.
    /// The method is still kept in the emitted impl block (it's a real trait method).
    skip: bool,
    /// Override the R-facing method name. When set, the R wrapper uses this name
    /// instead of the Rust method name (e.g., `next` -> `next_item` to avoid R reserved words).
    r_name: Option<String>,
    /// Strict output conversion: panic instead of lossy widening for i64/u64/isize/usize.
    strict: bool,
    /// Lifecycle specification for deprecation/experimental status.
    lifecycle: Option<crate::lifecycle::LifecycleSpec>,
    /// R code to inject at the very top of the wrapper body.
    r_entry: Option<String>,
    /// R code to inject after all checks, immediately before `.Call()`.
    r_post_checks: Option<String>,
    /// Register `on.exit()` cleanup code in the R wrapper.
    r_on_exit: Option<crate::miniextendr_fn::ROnExit>,
    /// Opt out of the per-class `<ClassName>_<method>` S7 fast-dispatch shortcut
    /// (set via `#[miniextendr(s7(no_shortcut))]`). Only meaningful for S7 trait
    /// impls; ignored by the other class systems.
    no_shortcut: bool,
    /// Existing S3/S4 generic and its receiver/formals contract.
    generic: Option<String>,
    receiver: Option<String>,
    no_dispatch_dots: bool,
    /// Per-parameter `#[miniextendr(match_arg(...))]` / `choices(...)` /
    /// `several_ok` attributes, keyed by Rust parameter name. Mirrors
    /// `MethodAttrs::per_param` on the inherent-impl path (`miniextendr_impl.rs`)
    /// so trait methods get the same `match.arg()` validation prelude — see
    /// `TraitMethodContext::match_arg_prelude`.
    per_param: std::collections::HashMap<String, crate::miniextendr_fn::ParamAttrs>,
    /// The method's own `preconditions` / `no_preconditions` (bare, #1017):
    /// the type-derived R-side checks of the parameters that do not decide
    /// for themselves (`preconditions(p)` / `no_preconditions(p)` land in
    /// [`Self::per_param`]). `None` defers to the impl block.
    preconditions: Option<bool>,
}

impl TraitMethod {
    /// Returns the R-facing method name.
    ///
    /// Uses `r_name` override if set, otherwise falls back to the Rust identifier string.
    fn r_method_name(&self) -> String {
        self.r_name
            .clone()
            .unwrap_or_else(|| crate::naming::ident_name(&self.ident))
    }

    /// Generates the C wrapper function identifier: `C_{crate}_{Type}__{Trait}__{method}`.
    ///
    /// This is the symbol name registered with R via `R_CallMethodDef` for `.Call()` access.
    /// Crate-prefixed for webR cross-package symbol uniqueness (#1273); shares one
    /// formatter with `TraitConst` and with the string variant below so the shape
    /// can't drift across the two types × two forms.
    fn c_wrapper_ident(&self, type_ident: &syn::Ident, trait_name: &syn::Ident) -> syn::Ident {
        crate::naming::trait_member_c_wrapper_ident(type_ident, trait_name, &self.ident)
    }

    /// Generates the C wrapper identifier as a `String`, for use in R-side `.Call()` generation.
    ///
    /// Prefer `c_wrapper_ident()` for Rust token generation; use this variant
    /// when building R code strings (e.g., `".Call(C_crate_Type__Trait__method, ...)"`).
    fn c_wrapper_ident_string(&self, type_ident: &syn::Ident, trait_name: &syn::Ident) -> String {
        crate::naming::trait_member_c_wrapper_string(type_ident, trait_name, &self.ident)
    }

    /// Whether the R wrapper returns this method's value with `invisible()`
    /// (#1213): set by an `Invisible<T>` return-type marker on the method.
    /// Trait methods are visible by default, void ones included.
    fn is_invisible(&self) -> bool {
        self.invisible == Some(true)
    }

    /// Returns true if this method has no return type (returns unit `()`).
    ///
    /// Used to decide whether the R wrapper should hand back the receiver
    /// (`x` / `self`) for void instance methods (pipe-friendly chaining).
    fn returns_unit(&self) -> bool {
        match &self.sig.output {
            syn::ReturnType::Default => true,
            syn::ReturnType::Type(_, ty) => {
                matches!(ty.as_ref(), syn::Type::Tuple(t) if t.elems.is_empty())
            }
        }
    }

    /// Returns true if this method returns `Self` (a factory/clone method).
    ///
    /// Mirrors [`crate::miniextendr_impl::ParsedMethod::returns_self`] on the
    /// inherent-impl path. The C wrapper hands back a bare `ExternalPtr`
    /// (`ReturnHandling::ExternalPtr`); the R wrapper must re-wrap it into a
    /// classed object — see `TraitMethodContext::returns_self`.
    fn returns_self(&self) -> bool {
        matches!(&self.sig.output, syn::ReturnType::Type(_, ty)
            if matches!(ty.as_ref(), syn::Type::Path(p)
                if p.path.segments.last().map(|s| s.ident == "Self").unwrap_or(false)))
    }

    /// Returns true if this method returns `Result<Self, E>`. Treated exactly
    /// like a bare `Self` return on the R side (the C wrapper already raised on
    /// `Err`, so `.val` is a bare `ExternalPtr`). Mirrors
    /// [`crate::miniextendr_impl::ParsedMethod::returns_result_self`].
    fn returns_result_self(&self) -> bool {
        self.first_generic_arg_is_self("Result")
    }

    /// Returns true if this method returns `Option<Self>`. Treated exactly like
    /// a bare `Self` return on the R side (the C wrapper already raised on
    /// `None`). Mirrors
    /// [`crate::miniextendr_impl::ParsedMethod::returns_option_self`].
    fn returns_option_self(&self) -> bool {
        self.first_generic_arg_is_self("Option")
    }

    /// Shared helper for [`Self::returns_result_self`] /
    /// [`Self::returns_option_self`]: true when the return type is
    /// `wrapper<Self, ..>` (e.g. `Result<Self, E>` / `Option<Self>`).
    fn first_generic_arg_is_self(&self, wrapper: &str) -> bool {
        let syn::ReturnType::Type(_, ty) = &self.sig.output else {
            return false;
        };
        let syn::Type::Path(p) = ty.as_ref() else {
            return false;
        };
        let Some(seg) = p.path.segments.last() else {
            return false;
        };
        if seg.ident != wrapper {
            return false;
        }
        let syn::PathArguments::AngleBracketed(ab) = &seg.arguments else {
            return false;
        };
        matches!(ab.args.first(), Some(syn::GenericArgument::Type(syn::Type::Path(ip)))
            if ip.path.segments.last().map(|s| s.ident == "Self").unwrap_or(false))
    }

    /// Generates the `R_CallMethodDef` static identifier: `call_method_def_{Type}__{Trait}_{method}`.
    ///
    /// This constant holds the C function pointer and arity used by R's `.Call()` registration.
    fn call_method_def_ident(
        &self,
        type_ident: &syn::Ident,
        trait_name: &syn::Ident,
    ) -> syn::Ident {
        format_ident!(
            "call_method_def_{}__{}_{}",
            type_ident,
            trait_name,
            self.ident
        )
    }
}

/// Parsed associated constant from a trait impl block.
///
/// Trait constants are exposed to R as zero-argument `.Call()` wrappers
/// that simply return the constant value.
#[derive(Debug)]
struct TraitConst {
    /// Constant identifier (e.g., `MAX_SIZE`).
    ident: syn::Ident,
    /// Constant type (e.g., `i32`, `&str`). Used to determine SEXP conversion.
    ty: syn::Type,
}

impl TraitConst {
    /// Generates the C wrapper function identifier: `C_{crate}_{Type}__{Trait}__{CONST}`.
    ///
    /// This is the symbol name registered with R for `.Call()` access to the constant.
    /// Crate-prefixed for webR cross-package symbol uniqueness (#1273); shares one
    /// formatter with `TraitMethod` so the shape can't drift.
    fn c_wrapper_ident(&self, type_ident: &syn::Ident, trait_name: &syn::Ident) -> syn::Ident {
        crate::naming::trait_member_c_wrapper_ident(type_ident, trait_name, &self.ident)
    }

    /// Generates the C wrapper identifier as a `String`, for use in R-side `.Call()` generation.
    fn c_wrapper_ident_string(&self, type_ident: &syn::Ident, trait_name: &syn::Ident) -> String {
        crate::naming::trait_member_c_wrapper_string(type_ident, trait_name, &self.ident)
    }

    /// Generates the `R_CallMethodDef` static identifier: `call_method_def_{Type}__{Trait}_{CONST}`.
    fn call_method_def_ident(
        &self,
        type_ident: &syn::Ident,
        trait_name: &syn::Ident,
    ) -> syn::Ident {
        format_ident!(
            "call_method_def_{}__{}_{}",
            type_ident,
            trait_name,
            self.ident
        )
    }
}

/// Expand `#[miniextendr]` applied to a trait implementation.
///
/// # Arguments
///
/// * `attr` - Attribute arguments (currently unused)
/// * `item` - The impl block token stream
///
/// # Returns
///
/// Expanded token stream containing:
/// - Original impl block
/// - Vtable static constant
///
/// # Errors
///
/// Returns a compile error if:
/// - Not applied to a trait impl (`impl Trait for Type`)
/// - Applied to an inherent impl (`impl Type`)
pub fn expand_miniextendr_impl_trait(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // Parse class system from attribute (defaults to Env if empty)
    let attr_tokens = TokenStream::from(attr.clone());
    let impl_attrs: ImplAttrs = syn::parse_macro_input!(attr as ImplAttrs);
    let impl_item = syn::parse_macro_input!(item as ItemImpl);

    // Validate: must be a trait impl, not inherent impl
    let (trait_path, concrete_type) = match extract_trait_and_type(&impl_item) {
        Ok(result) => result,
        Err(e) => return e.into_compile_error().into(),
    };

    // TPIE: empty impl body → expand via macro_rules! helper from the trait definition
    if impl_item.items.is_empty() && !impl_attrs.blanket {
        // Only the `@noRd` flag and the `@param` warnings are read here; the
        // page rule is `generate_vtable_static`'s (#1818).
        let raw_tags = crate::roxygen::roxygen_tags_from_attrs(
            &impl_item.attrs,
            crate::roxygen::RdPage::suppressed_if(impl_attrs.noexport && !impl_attrs.internal),
        );
        let (doc_tags, param_warnings) = crate::roxygen::strip_method_tags(
            &raw_tags,
            &concrete_type.to_token_stream().to_string(),
            impl_item.impl_token.span,
        );
        let no_rd = crate::roxygen::has_roxygen_tag(&doc_tags, "noRd");
        let mut output = generate_tpie_invocation(
            &trait_path,
            &concrete_type,
            impl_attrs.class_system,
            no_rd,
            impl_attrs.internal,
            impl_attrs.noexport,
            // The trait's signatures carry no method or parameter spellings,
            // so the impl block, the crate default and the feature decide;
            // the `macro_rules!` contract carries the decision as a bool.
            !crate::r_preconditions::resolve_item_type_checks(
                None,
                impl_attrs.preconditions,
                crate::crate_config::preconditions_default(),
                cfg!(feature = "no-preconditions-default"),
            ),
        );
        output.extend(param_warnings);
        return output.into();
    }

    // Ask the declaration for its R metadata before generating wrappers.
    // Rust imports/re-exports the metadata macro beside the trait itself.
    let mut macro_path = trait_path;
    macro_path.segments.last_mut().unwrap().arguments = syn::PathArguments::None;
    quote::quote! {
        #macro_path! { @impl { #attr_tokens } { #impl_item } }
    }
    .into()
}

/// Extract the trait path and concrete type from an impl block.
///
/// For `impl path::Trait for concrete::Type`, returns:
/// - trait_path: `path::Trait`
/// - concrete_type: `concrete::Type`
fn extract_trait_and_type(impl_item: &ItemImpl) -> syn::Result<(syn::Path, syn::Type)> {
    // Check for trait impl
    let (_, trait_path, _) = impl_item.trait_.as_ref().ok_or_else(|| {
        syn::Error::new_spanned(
            impl_item,
            "#[miniextendr] must be applied to a trait implementation (impl Trait for Type), \
             not an inherent impl (impl Type)",
        )
    })?;

    let concrete_type = (*impl_item.self_ty).clone();

    Ok((trait_path.clone(), concrete_type))
}

// region: Sub-modules

mod defaults;
mod method_context;
mod r_wrappers;
mod vtable;

use r_wrappers::TraitWrapperOpts;
use r_wrappers::generate_trait_r_wrapper;
use vtable::generate_trait_method_c_wrapper;
use vtable::generate_vtable_static;

/// Generate R function body lines that capture the `.Call()` result in `.val`,
/// check for a tagged `rust_condition_value`, and return `.val`.
/// `raise_default` is the raise helper's fallback call
/// (`CallAttribution::raise_default`).
fn trait_method_body_lines(call_expr: &str, indent: &str, raise_default: &str) -> Vec<String> {
    let mut lines = vec![format!("{}.val <- {}", indent, call_expr)];
    lines.extend(
        crate::method_return_builder::condition_check_lines_with_default(indent, raise_default),
    );
    lines.push(format!("{}.val", indent));
    lines
}

/// Convert a type to a case-preserving identifier-safe name.
///
/// For non-generic types, the name is the last path segment with its spelling preserved:
/// - `MyType` → `MyType`
/// - `path::to::MyType` → `MyType`
///
/// For generic types, a 16-character lowercase hex suffix derived from a stable
/// FNV-1a-64 hash of the full canonical token stream is appended:
/// - `MyType<u32>` → `MyType_a1b2c3d4e5f60718`
/// - `MyType<f64>` → `MyType_0102030405060708` (different hash → no collision)
///
/// This prevents vtable static name collisions when the same base type is
/// monomorphised with different generic arguments in the same crate.
///
/// **Hash stability:** FNV-1a-64 with a fixed seed (offset basis = 0xcbf29ce484222325)
/// is deterministic across builds, rustc versions, and platforms.  We deliberately
/// avoid `std::collections::hash_map::DefaultHasher` and `RandomState` because both
/// are explicitly unspecified and can change across releases.
fn type_to_symbol_name(ty: &syn::Type) -> String {
    /// FNV-1a 64-bit hash of a byte slice.
    ///
    /// Chosen for simplicity (≈10 lines, no new deps) and cross-build stability.
    /// The FNV offset basis and prime are fixed constants defined in the FNV spec.
    fn fnv1a_64(data: &[u8]) -> u64 {
        const OFFSET_BASIS: u64 = 0xcbf29ce484222325;
        const PRIME: u64 = 0x00000100000001b3;
        let mut h = OFFSET_BASIS;
        for &b in data {
            h ^= b as u64;
            h = h.wrapping_mul(PRIME);
        }
        h
    }

    match ty {
        syn::Type::Path(type_path) => {
            let last = type_path.path.segments.last();
            let base = last
                .map(|s| crate::naming::ident_name(&s.ident))
                .unwrap_or_else(|| "UNKNOWN".to_string());

            // Only append a hash suffix when the type actually carries generic arguments
            // (e.g. `MyType<u32>`). Plain `MyType` keeps the clean `MyType` form.
            let has_generics = last
                .map(|s| !matches!(s.arguments, syn::PathArguments::None))
                .unwrap_or(false);

            if has_generics {
                // Canonical token string of the *full* type (including all generic args)
                // so that `MyType<u32>` and `MyType<f64>` produce distinct hashes.
                let token_str = quote::quote!(#ty).to_string();
                let hash = fnv1a_64(token_str.as_bytes());
                format!("{}_{:016x}", base, hash)
            } else {
                base
            }
        }
        _ => "UNKNOWN".to_string(),
    }
}
// endregion

// region: TPIE: Trait-Provided Impl Expansion

/// The declaration's ABI layout and tag, carried through its metadata macro.
/// An impl must not reconstruct these from an imported or re-exported path.
struct TraitAbiMetadata {
    name: syn::Ident,
    generic: bool,
    tag: syn::LitStr,
    fields: syn::FieldsNamed,
}

impl syn::parse::Parse for TraitAbiMetadata {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let keyword: syn::Ident = input.parse()?;
        if keyword != "abi" {
            return Err(syn::Error::new_spanned(
                keyword,
                "expected trait ABI metadata",
            ));
        }
        Ok(Self {
            name: input.parse()?,
            generic: input.parse::<syn::LitBool>()?.value,
            tag: input.parse()?,
            fields: input.parse()?,
        })
    }
}

/// Populated impl plus declaration metadata, supplied by the trait's macro.
/// Keep this as Rust syntax so method bodies, generics and author attributes
/// pass through without a second bespoke representation.
struct ImplWithMetadata {
    attrs: ImplAttrs,
    implementation: ItemImpl,
    methods: Vec<syn::TraitItemFn>,
    generics: syn::Generics,
    abi: TraitAbiMetadata,
}

impl syn::parse::Parse for ImplWithMetadata {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<syn::Token![@]>()?;
        input.parse::<syn::Token![impl]>()?;
        let attrs;
        syn::braced!(attrs in input);
        let attrs = attrs.parse()?;
        let implementation;
        syn::braced!(implementation in input);
        let implementation = implementation.parse()?;
        let metadata;
        syn::braced!(metadata in input);
        let mut methods = Vec::new();
        while !metadata.is_empty() {
            methods.push(metadata.parse()?);
        }
        let generics;
        syn::braced!(generics in input);
        Ok(Self {
            attrs,
            implementation,
            methods,
            generics: generics.parse()?,
            abi: input.parse()?,
        })
    }
}

impl ImplWithMetadata {
    fn expand(mut self) -> syn::Result<TokenStream> {
        vtable::inherit_method_metadata(&mut self.implementation, &self.methods)?;
        let (trait_path, concrete_type) = extract_trait_and_type(&self.implementation)?;
        let defaults = defaults::inherited_methods(
            &self.implementation,
            &self.methods,
            &self.generics,
            &trait_path,
        )?;
        Ok(generate_vtable_static(
            &self.implementation,
            &trait_path,
            &concrete_type,
            &self.attrs,
            &self.abi,
            &self.methods,
            &defaults,
        ))
    }
}

/// Input to the `__mx_trait_impl_expand!` proc macro.
///
/// TPIE (Trait-Provided Impl Expansion) allows empty `#[miniextendr] impl Trait for Type {}`
/// blocks to auto-expand C/R wrappers using metadata embedded in a `macro_rules!` helper
/// generated at the trait definition site.
///
/// Parsed from tokens like:
/// ```text
/// concrete_type = Point;
/// trait_path = miniextendr_api::adapter_traits::RDebug;
/// class_system = env;
/// no_rd = false;
/// internal = false;
/// noexport = false;
/// no_preconditions = false;
/// abi RDebug false "<declaration fingerprint>" {
///     pub debug_str: ::miniextendr_api::abi::mx_meth,
///     pub debug_str_pretty: ::miniextendr_api::abi::mx_meth,
/// }
/// method { r_name = debug_str; #[doc = " Get a compact debug string representation."] fn debug_str(&self) -> String; }
/// method { r_name = debug_str_pretty; fn debug_str_pretty(&self) -> String; }
/// ```
struct TpieInput {
    /// The concrete type implementing the trait (e.g., `Point`).
    concrete_type: syn::Type,
    /// Fully qualified path to the trait (e.g., `miniextendr_api::adapter_traits::RDebug`).
    trait_path: syn::Path,
    /// Which R class system to generate wrappers for (env, r6, s3, s4, s7).
    class_system: ClassSystem,
    /// Whether the impl block has `@noRd`, suppressing roxygen documentation.
    no_rd: bool,
    /// Whether the impl block has `#[miniextendr(internal)]`, adding `@keywords internal`.
    internal: bool,
    /// Whether the impl block has `#[miniextendr(noexport)]`, suppressing `@export`.
    noexport: bool,
    /// Whether the impl block drops the R-side type checks: its
    /// `ImplAttrs::preconditions`, the crate default and the
    /// `no-preconditions-default` feature, resolved in the implementing crate
    /// when the empty impl invoked the trait's `macro_rules!` helper.
    no_preconditions: bool,
    /// Method signatures and R-facing names from the trait definition.
    methods: Vec<TpieMethod>,
    abi: TraitAbiMetadata,
}

/// A single method entry in TPIE metadata.
///
/// Contains the R-facing name, the declaration's docs and the method
/// signature as declared in the trait.
struct TpieMethod {
    /// The R-facing method name (may differ from the Rust ident via `r_name`).
    r_name: String,
    /// The declaration's `#[doc]` attributes, which document the method's
    /// wrappers.
    docs: Vec<syn::Attribute>,
    /// The method signature (parameters and return type) from the trait definition.
    sig: syn::Signature,
}

impl syn::parse::Parse for TpieInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // concrete_type = Type;
        let kw: syn::Ident = input.parse()?;
        if kw != "concrete_type" {
            return Err(syn::Error::new_spanned(
                &kw,
                format!("expected 'concrete_type', got '{}'", kw),
            ));
        }
        input.parse::<syn::Token![=]>()?;
        let concrete_type: syn::Type = input.parse()?;
        input.parse::<syn::Token![;]>()?;

        // trait_path = some::Path;
        let kw: syn::Ident = input.parse()?;
        if kw != "trait_path" {
            return Err(syn::Error::new_spanned(
                &kw,
                format!("expected 'trait_path', got '{}'", kw),
            ));
        }
        input.parse::<syn::Token![=]>()?;
        let trait_path: syn::Path = input.parse()?;
        input.parse::<syn::Token![;]>()?;

        // class_system = env;
        let kw: syn::Ident = input.parse()?;
        if kw != "class_system" {
            return Err(syn::Error::new_spanned(
                &kw,
                format!("expected 'class_system', got '{}'", kw),
            ));
        }
        input.parse::<syn::Token![=]>()?;
        let cs_ident: syn::Ident = input.parse()?;
        input.parse::<syn::Token![;]>()?;

        let class_system = ClassSystem::from_ident(&cs_ident).ok_or_else(|| {
            syn::Error::new_spanned(&cs_ident, format!("unknown class system: {}", cs_ident))
        })?;

        // no_rd = true/false;
        let kw: syn::Ident = input.parse()?;
        if kw != "no_rd" {
            return Err(syn::Error::new_spanned(
                &kw,
                format!("expected 'no_rd', got '{}'", kw),
            ));
        }
        input.parse::<syn::Token![=]>()?;
        let no_rd_lit: syn::LitBool = input.parse()?;
        input.parse::<syn::Token![;]>()?;
        let no_rd = no_rd_lit.value;

        // internal = true/false;
        let kw: syn::Ident = input.parse()?;
        if kw != "internal" {
            return Err(syn::Error::new_spanned(
                &kw,
                format!("expected 'internal', got '{}'", kw),
            ));
        }
        input.parse::<syn::Token![=]>()?;
        let internal_lit: syn::LitBool = input.parse()?;
        input.parse::<syn::Token![;]>()?;
        let internal = internal_lit.value;

        // noexport = true/false;
        let kw: syn::Ident = input.parse()?;
        if kw != "noexport" {
            return Err(syn::Error::new_spanned(
                &kw,
                format!("expected 'noexport', got '{}'", kw),
            ));
        }
        input.parse::<syn::Token![=]>()?;
        let noexport_lit: syn::LitBool = input.parse()?;
        input.parse::<syn::Token![;]>()?;
        let noexport = noexport_lit.value;

        // no_preconditions = true/false;
        let kw: syn::Ident = input.parse()?;
        if kw != "no_preconditions" {
            return Err(syn::Error::new_spanned(
                &kw,
                format!("expected 'no_preconditions', got '{}'", kw),
            ));
        }
        input.parse::<syn::Token![=]>()?;
        let no_preconditions_lit: syn::LitBool = input.parse()?;
        input.parse::<syn::Token![;]>()?;
        let no_preconditions = no_preconditions_lit.value;

        let abi = input.parse()?;

        // method { ... } repeated
        let mut methods = Vec::new();
        while !input.is_empty() {
            let kw: syn::Ident = input.parse()?;
            if kw != "method" {
                return Err(syn::Error::new_spanned(
                    &kw,
                    format!("expected 'method', got '{}'", kw),
                ));
            }
            let content;
            syn::braced!(content in input);
            methods.push(content.parse::<TpieMethod>()?);
        }

        Ok(TpieInput {
            concrete_type,
            trait_path,
            class_system,
            no_rd,
            internal,
            noexport,
            no_preconditions,
            methods,
            abi,
        })
    }
}

impl syn::parse::Parse for TpieMethod {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // r_name = some_name;
        let kw: syn::Ident = input.parse()?;
        if kw != "r_name" {
            return Err(syn::Error::new_spanned(
                &kw,
                format!("expected 'r_name', got '{}'", kw),
            ));
        }
        input.parse::<syn::Token![=]>()?;
        let r_name_ident: syn::Ident = input.parse()?;
        input.parse::<syn::Token![;]>()?;

        // #[doc = "..."]* fn method_name(...) -> ReturnType;
        let docs = syn::Attribute::parse_outer(input)?;
        let sig: syn::Signature = input.parse()?;
        input.parse::<syn::Token![;]>()?;

        Ok(TpieMethod {
            r_name: crate::naming::ident_name(&r_name_ident),
            docs,
            sig,
        })
    }
}

/// Rewrite `Self` → concrete type in a method signature's parameters.
///
/// `&Self` params are left as-is because `generate_trait_method_c_wrapper`
/// detects them via `type_inspect::self_ref_mutability` and generates `ExternalPtr<T>` extraction.
///
/// The return type keeps `Self`, as in a trait impl with method bodies: the
/// C wrapper hands a `Self` / `Option<Self>` / `Result<Self, E>` back as an
/// `ExternalPtr` without naming the type, and the R wrapper re-wraps exactly
/// those returns into an object of the class (`TraitMethod::returns_self` and
/// its siblings look for `Self`).
fn rewrite_self_in_params(sig: &mut syn::Signature, concrete_type: &syn::Type) {
    for input in &mut sig.inputs {
        if let syn::FnArg::Typed(pt) = input {
            // Don't rewrite &Self — C wrapper generator handles it specially
            if crate::type_inspect::self_ref_mutability(&pt.ty).is_some() {
                continue;
            }
            let rewritten = rewrite_self_type(&pt.ty, concrete_type);
            *pt.ty = rewritten;
        }
    }
}

/// Recursively replace `Self` with the concrete type in a type tree.
fn rewrite_self_type(ty: &syn::Type, concrete_type: &syn::Type) -> syn::Type {
    match ty {
        syn::Type::Path(tp) => {
            if tp.path.is_ident("Self") {
                return concrete_type.clone();
            }
            let mut new_tp = tp.clone();
            for seg in &mut new_tp.path.segments {
                if let syn::PathArguments::AngleBracketed(args) = &mut seg.arguments {
                    for arg in &mut args.args {
                        if let syn::GenericArgument::Type(inner) = arg {
                            *inner = rewrite_self_type(inner, concrete_type);
                        }
                    }
                }
            }
            syn::Type::Path(new_tp)
        }
        syn::Type::Reference(r) => {
            let mut new_r = r.clone();
            new_r.elem = Box::new(rewrite_self_type(&r.elem, concrete_type));
            syn::Type::Reference(new_r)
        }
        syn::Type::Tuple(t) => {
            let mut new_t = t.clone();
            for elem in &mut new_t.elems {
                *elem = rewrite_self_type(elem, concrete_type);
            }
            syn::Type::Tuple(new_t)
        }
        _ => ty.clone(),
    }
}

/// Unwrap invisible Group tokens from `macro_rules!` `$t:ty` captures.
///
/// When a type passes through a `macro_rules!` pattern like `$concrete_type:ty`,
/// the compiler wraps it in a `Group` with invisible delimiters. This function
/// recursively unwraps those groups to get the underlying type.
fn unwrap_group_type(ty: &syn::Type) -> syn::Type {
    match ty {
        syn::Type::Group(g) => unwrap_group_type(&g.elem),
        _ => ty.clone(),
    }
}

/// Entry point for the `__mx_trait_impl_expand!` proc macro.
///
/// Parses TPIE metadata tokens and generates C wrappers, R wrappers,
/// and call defs — the same outputs as a manual trait impl with method bodies.
pub fn expand_tpie(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    if matches!(input.clone().into_iter().next(), Some(proc_macro::TokenTree::Punct(p)) if p.as_char() == '@')
    {
        let metadata = syn::parse_macro_input!(input as ImplWithMetadata);
        return metadata
            .expand()
            .unwrap_or_else(syn::Error::into_compile_error)
            .into();
    }
    let tpie_input = syn::parse_macro_input!(input as TpieInput);

    // Unwrap Group tokens (macro_rules! wraps $concrete_type:ty and $trait_path:path
    // in invisible groups)
    let concrete_type = unwrap_group_type(&tpie_input.concrete_type);
    let trait_path = &tpie_input.trait_path;
    let class_system = tpie_input.class_system;

    let trait_name = &tpie_input.abi.name;

    let type_ident = match &concrete_type {
        syn::Type::Path(tp) => tp
            .path
            .segments
            .last()
            .map(|s| s.ident.clone())
            .unwrap_or_else(|| format_ident!("Unknown")),
        _ => format_ident!("Unknown"),
    };

    // Convert TpieMethod → TraitMethod, rewriting Self → ConcreteType
    let methods: Vec<TraitMethod> = tpie_input
        .methods
        .iter()
        .map(|tm| {
            let mut sig = tm.sig.clone();
            rewrite_self_in_params(&mut sig, &concrete_type);

            let (has_self, is_mut) = sig.inputs.first().map_or((false, false), |arg| {
                if let syn::FnArg::Receiver(_) = arg {
                    (true, crate::trait_receiver::is_mut(&sig))
                } else {
                    (false, false)
                }
            });

            // Peel a return-visibility marker (#1213) so the return-shape
            // predicates and the C wrapper see the inner type.
            let (invisible, sig) = {
                let (marker, output) = crate::type_inspect::peel_return_visibility(&sig.output);
                let mut sig = sig;
                sig.output = output;
                (marker, sig)
            };
            TraitMethod {
                inherited_signature: true,
                ident: sig.ident.clone(),
                sig,
                invisible,
                has_self,
                is_mut,
                worker: cfg!(feature = "worker-default"),
                unsafe_main_thread: false,
                coerce: false,
                check_interrupt: false,
                rng: false,
                unwrap_in_r: false,
                serialize: false,
                return_wrap: None,
                param_defaults: Default::default(),
                // The declaration's docs, as for an inherited default.
                doc_tags: crate::roxygen::declaration_roxygen_tags(&tm.docs),
                skip: false,
                strict: false,
                lifecycle: None,
                r_entry: None,
                r_post_checks: None,
                r_on_exit: None,
                no_shortcut: false,
                generic: None,
                receiver: None,
                no_dispatch_dots: false,
                per_param: Default::default(),
                preconditions: None,
                r_name: if tm.sig.ident == tm.r_name {
                    None // r_name matches ident → no override
                } else {
                    Some(tm.r_name.clone())
                },
            }
        })
        .collect();

    // Generate C wrappers
    let c_wrappers: Vec<TokenStream> = methods
        .iter()
        .map(|m| generate_trait_method_c_wrapper(m, &type_ident, trait_name, trait_path))
        .collect();

    // Generate R wrappers
    let r_wrapper_string = match generate_trait_r_wrapper(
        &type_ident,
        trait_name,
        &methods,
        &[], // no consts in TPIE
        TraitWrapperOpts {
            class_system,
            class_has_no_rd: tpie_input.no_rd,
            internal: tpie_input.internal,
            noexport: tpie_input.noexport,
            impl_preconditions: Some(!tpie_input.no_preconditions),
        },
    ) {
        Ok(s) => s,
        Err(e) => return e.into_compile_error().into(),
    };

    // Generate const names (same pattern as generate_vtable_static)
    let type_name_str = type_to_symbol_name(&concrete_type);
    let r_wrappers_const = crate::naming::trait_impl_wrappers_ident(&type_name_str, trait_name);

    // Generate trait dispatch entry name
    let dispatch_entry_name = crate::naming::trait_dispatch_ident(trait_name, &type_name_str);

    let trait_tag = &tpie_input.abi.tag;

    // Build vtable static name (same as generate_vtable_static — routed through
    // the shared naming.rs helper so the three reconstruction sites can't drift, #1273)
    let vtable_static_name = crate::naming::vtable_static_ident(trait_name, &type_name_str);

    let vtable = match vtable::generate_impl_vtable(
        &tpie_input.abi,
        &concrete_type,
        trait_path,
        &methods,
        &[],
    ) {
        Ok(tokens) => tokens,
        Err(error) => return error.into_compile_error().into(),
    };

    // Format R wrapper as raw string literal
    let r_wrapper_str = crate::r_wrapper_raw_literal(
        &r_wrapper_string,
        &crate::r_shadowing::formal_names(methods.iter().map(|m| &m.sig.inputs)),
    );
    let source_start = type_ident.span().start();
    let source_line_lit = syn::LitInt::new(&source_start.line.to_string(), type_ident.span());
    let source_col_lit =
        syn::LitInt::new(&(source_start.column + 1).to_string(), type_ident.span());

    let expanded = quote::quote! {
        #vtable
        // C wrappers and call method defs for trait methods
        #(#c_wrappers)*

        // R wrapper registration via distributed slice
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
                    ::std::ptr::from_ref(&#vtable_static_name).cast::<::std::os::raw::c_void>()
                },
                vtable_symbol: ::core::stringify!(#vtable_static_name),
            };
    };

    expanded.into()
}

/// Request declaration metadata for an empty trait impl.
///
/// When `#[miniextendr] impl Trait for Type {}` has no method bodies, this
/// delegates to the trait's metadata macro
/// (generated at the trait definition site) to expand C/R wrappers.
fn generate_tpie_invocation(
    trait_path: &syn::Path,
    concrete_type: &syn::Type,
    class_system: ClassSystem,
    class_has_no_rd: bool,
    internal: bool,
    noexport: bool,
    no_preconditions: bool,
) -> TokenStream {
    // Resolve metadata through the same path as the trait. The declaration
    // exports both in separate Rust namespaces.
    let mut macro_path = trait_path.clone();
    macro_path.segments.last_mut().unwrap().arguments = syn::PathArguments::None;
    let class_system_ident = class_system.to_ident();
    let no_rd_ident = if class_has_no_rd {
        format_ident!("true")
    } else {
        format_ident!("false")
    };
    let internal_ident = if internal {
        format_ident!("true")
    } else {
        format_ident!("false")
    };
    let noexport_ident = if noexport {
        format_ident!("true")
    } else {
        format_ident!("false")
    };
    let no_preconditions_ident = if no_preconditions {
        format_ident!("true")
    } else {
        format_ident!("false")
    };

    quote::quote! {
        #macro_path !(#concrete_type, #trait_path, #class_system_ident, #no_rd_ident, #internal_ident, #noexport_ident, #no_preconditions_ident);
    }
}

#[cfg(test)]
mod tests;
// endregion
