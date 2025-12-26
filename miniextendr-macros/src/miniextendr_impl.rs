//! Impl-block parsing and wrapper generation for class-like Rust structs.
//!
//! This module provides shared infrastructure for all class system support:
//! - Env (environment-style with `$`/`[[` dispatch)
//! - R6 (`R6::R6Class`)
//! - S7 (`S7::new_class`)
//! - S3 (`structure()` with class attr)
//! - S4 (`setClass`)
//!
//! The impl-block parser extracts methods and categorizes them by env type,
//! then class-system adapters generate appropriate R wrapper code.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

/// Strip miniextendr attributes (and roxygen tags) from an impl block and its items.
fn strip_miniextendr_attrs_from_impl(mut item_impl: syn::ItemImpl) -> syn::ItemImpl {
    item_impl.attrs = crate::roxygen::strip_roxygen_from_attrs(&item_impl.attrs);
    item_impl
        .attrs
        .retain(|attr| !attr.path().is_ident("miniextendr"));
    for item in &mut item_impl.items {
        match item {
            syn::ImplItem::Fn(fn_item) => {
                fn_item.attrs = crate::roxygen::strip_roxygen_from_attrs(&fn_item.attrs);
                fn_item
                    .attrs
                    .retain(|attr| !attr.path().is_ident("miniextendr"));
            }
            syn::ImplItem::Const(const_item) => {
                const_item.attrs = crate::roxygen::strip_roxygen_from_attrs(&const_item.attrs);
                const_item
                    .attrs
                    .retain(|attr| !attr.path().is_ident("miniextendr"));
            }
            syn::ImplItem::Type(type_item) => {
                type_item.attrs = crate::roxygen::strip_roxygen_from_attrs(&type_item.attrs);
                type_item
                    .attrs
                    .retain(|attr| !attr.path().is_ident("miniextendr"));
            }
            syn::ImplItem::Macro(macro_item) => {
                macro_item.attrs = crate::roxygen::strip_roxygen_from_attrs(&macro_item.attrs);
                macro_item
                    .attrs
                    .retain(|attr| !attr.path().is_ident("miniextendr"));
            }
            _ => {}
        }
    }
    item_impl
}

/// Class system flavor for wrapper generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassSystem {
    /// Environment-style with `$`/`[[` dispatch
    Env,
    /// R6::R6Class
    R6,
    /// S7::new_class
    S7,
    /// S3 structure() with class attribute
    S3,
    /// S4 setClass
    S4,
}

impl std::str::FromStr for ClassSystem {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "env" => Ok(ClassSystem::Env),
            "r6" => Ok(ClassSystem::R6),
            "s7" => Ok(ClassSystem::S7),
            "s3" => Ok(ClassSystem::S3),
            "s4" => Ok(ClassSystem::S4),
            _ => Err(format!("unknown class system: {}", s)),
        }
    }
}

/// Receiver kind for methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReceiverKind {
    /// No env - static/associated function
    None,
    /// `&self` - immutable borrow
    Ref,
    /// `&mut self` - mutable borrow
    RefMut,
    /// `self` - consuming (not supported in v1)
    Value,
}

impl ReceiverKind {
    /// Returns true if this is an instance method (has self).
    pub fn is_instance(&self) -> bool {
        matches!(self, ReceiverKind::Ref | ReceiverKind::RefMut)
    }

    /// Returns true if this requires mutable access.
    #[allow(dead_code)]
    pub fn is_mut(&self) -> bool {
        matches!(self, ReceiverKind::RefMut | ReceiverKind::Value)
    }
}

/// Parsed method from an impl block.
///
/// # Default Parameters
///
/// Default parameters are specified using method-level syntax:
/// - `#[miniextendr(defaults(param = "value", ...))]` on the method
///
/// Note: Parameter-level `#[miniextendr(default = "...")]` syntax is only supported
/// for standalone functions, not impl methods (Rust language limitation).
///
/// Defaults cannot be specified for `self` parameters (compile error).
#[derive(Debug)]
pub struct ParsedMethod {
    /// Method identifier
    pub ident: syn::Ident,
    /// Receiver kind
    pub env: ReceiverKind,
    /// Method signature (without env)
    pub sig: syn::Signature,
    /// Visibility
    pub vis: syn::Visibility,
    /// Roxygen tag lines extracted from Rust doc comments
    pub doc_tags: Vec<String>,
    /// Per-method attributes for class system overrides
    pub method_attrs: MethodAttrs,
    /// Parameter default values from `#[miniextendr(default = "...")]`
    pub param_defaults: std::collections::HashMap<String, String>,
}

/// Per-method attributes for class system customization.
#[derive(Debug, Default)]
pub struct MethodAttrs {
    /// Skip this method
    pub ignore: bool,
    /// Mark as constructor
    pub constructor: bool,
    /// Mark as finalizer (R6)
    pub finalize: bool,
    /// Mark as private (R6)
    pub private: bool,
    /// Mark as active binding (R6)
    pub active: bool,
    /// Override generic name for S3/S4/S7 methods.
    ///
    /// Use this to implement methods for existing generics (like `print`, `format`, `length`)
    /// without creating a new generic. When set, the generated code:
    /// - Uses the specified generic name instead of the method name
    /// - Skips creating a new generic (assumes it already exists)
    /// - Creates only the method implementation (e.g., `print.MyClass`)
    ///
    /// # Example
    /// ```ignore
    /// #[miniextendr(s3)]
    /// impl MyType {
    ///     #[miniextendr(generic = "print")]
    ///     fn show(&self) -> String {
    ///         format!("MyType: {}", self.value)
    ///     }
    /// }
    /// ```
    /// This generates `print.MyType` that calls the `show` method.
    pub generic: Option<String>,
    /// Worker thread execution (default: auto-detect based on types)
    pub worker: bool,
    /// Force main thread execution (unsafe)
    pub unsafe_main_thread: bool,
    /// Enable R interrupt checking
    pub check_interrupt: bool,
    /// Enable coercion for this method's parameters
    pub coerce: bool,
    /// Parameter defaults from `#[miniextendr(defaults(param = "value", ...))]`
    pub defaults: std::collections::HashMap<String, String>,
}

/// Parsed impl block with all methods.
#[derive(Debug)]
pub struct ParsedImpl {
    /// Type being implemented
    pub type_ident: syn::Ident,
    /// Type generics (rejected in v1 unless 'static)
    #[allow(dead_code)]
    pub generics: syn::Generics,
    /// Class system to use
    pub class_system: ClassSystem,
    /// Override class name (else type name)
    pub class_name: Option<String>,
    /// Roxygen tag lines extracted from impl doc comments
    pub doc_tags: Vec<String>,
    /// All parsed methods
    pub methods: Vec<ParsedMethod>,
    /// Original impl item for re-emission
    pub original_impl: syn::ItemImpl,
    /// cfg attributes to propagate
    pub cfg_attrs: Vec<syn::Attribute>,
}

/// Attributes on the impl block itself.
#[derive(Debug)]
pub struct ImplAttrs {
    pub class_system: ClassSystem,
    pub class_name: Option<String>,
}

impl syn::parse::Parse for ImplAttrs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut class_system = ClassSystem::Env;
        let mut class_name = None;

        // Parse the first identifier (class system)
        if !input.is_empty() {
            let first: syn::Ident = input.parse()?;
            class_system = first
                .to_string()
                .parse()
                .map_err(|e| syn::Error::new(first.span(), e))?;

            // Parse optional key=value pairs
            while !input.is_empty() {
                let _: syn::Token![,] = input.parse()?;
                if input.is_empty() {
                    break;
                }
                let key: syn::Ident = input.parse()?;
                let _: syn::Token![=] = input.parse()?;

                match key.to_string().as_str() {
                    "class" => {
                        let value: syn::LitStr = input.parse()?;
                        class_name = Some(value.value());
                    }
                    _ => {
                        return Err(syn::Error::new(
                            key.span(),
                            format!("unknown option: {}", key),
                        ));
                    }
                }
            }
        }

        Ok(ImplAttrs {
            class_system,
            class_name,
        })
    }
}

impl ParsedMethod {
    /// Validate method attributes for the given class system.
    /// Returns an error if unsupported attributes are used.
    fn validate_method_attrs(
        attrs: &MethodAttrs,
        class_system: ClassSystem,
        span: proc_macro2::Span,
    ) -> syn::Result<()> {
        // #[...(active)] is only meaningful for R6, and not yet implemented
        if attrs.active {
            if class_system != ClassSystem::R6 {
                return Err(syn::Error::new(
                    span,
                    "#[r6(active)] is only valid for R6 class systems",
                ));
            }
            return Err(syn::Error::new(
                span,
                "#[r6(active)] active bindings are not yet implemented",
            ));
        }

        // Worker attribute is now supported on methods
        // (validation happens during wrapper generation based on return type)

        Ok(())
    }

    /// Parse method attributes in #[miniextendr(class_system(...))] format.
    ///
    /// Supported formats:
    /// - `#[miniextendr(r6(ignore, constructor, finalize, private, generic = "...")]`
    /// - `#[miniextendr(s3(ignore, constructor, generic = "..."))]`
    /// - `#[miniextendr(s7(ignore, constructor, generic = "..."))]`
    /// - etc.
    fn parse_method_attrs(attrs: &[syn::Attribute]) -> syn::Result<MethodAttrs> {
        let mut method_attrs = MethodAttrs::default();

        for attr in attrs {
            // Parse new-style #[miniextendr(class_system(...))] attributes
            if !attr.path().is_ident("miniextendr") {
                continue;
            }

            // Parse the nested content: miniextendr(class_system(options...)) or miniextendr(defaults(...))
            attr.parse_nested_meta(|meta| {
                let is_class_meta = meta.path.is_ident("env")
                    || meta.path.is_ident("r6")
                    || meta.path.is_ident("s7")
                    || meta.path.is_ident("s3")
                    || meta.path.is_ident("s4");

                if is_class_meta {
                    // Parse the inner options: r6(ignore, constructor, ...)
                    meta.parse_nested_meta(|inner| {
                        if inner.path.is_ident("ignore") {
                            method_attrs.ignore = true;
                        } else if inner.path.is_ident("constructor") {
                            method_attrs.constructor = true;
                        } else if inner.path.is_ident("finalize") {
                            method_attrs.finalize = true;
                        } else if inner.path.is_ident("private") {
                            method_attrs.private = true;
                        } else if inner.path.is_ident("active") {
                            method_attrs.active = true;
                        } else if inner.path.is_ident("worker") {
                            method_attrs.worker = true;
                        } else if inner.path.is_ident("main_thread") {
                            method_attrs.unsafe_main_thread = true;
                        } else if inner.path.is_ident("check_interrupt") {
                            method_attrs.check_interrupt = true;
                        } else if inner.path.is_ident("coerce") {
                            method_attrs.coerce = true;
                        } else if inner.path.is_ident("generic") {
                            let _: syn::Token![=] = inner.input.parse()?;
                            let value: syn::LitStr = inner.input.parse()?;
                            method_attrs.generic = Some(value.value());
                        }
                        Ok(())
                    })?;
                } else if meta.path.is_ident("defaults") {
                    // Parse defaults(param = "value", param2 = "value2", ...)
                    meta.parse_nested_meta(|inner| {
                        // Get parameter name
                        let param_name = inner
                            .path
                            .get_ident()
                            .ok_or_else(|| inner.error("expected parameter name"))?
                            .to_string();
                        // Parse = "value"
                        let _: syn::Token![=] = inner.input.parse()?;
                        let value: syn::LitStr = inner.input.parse()?;
                        method_attrs.defaults.insert(param_name, value.value());
                        Ok(())
                    })?;
                }
                Ok(())
            })?;
        }

        Ok(method_attrs)
    }

    /// Detect env kind from function signature.
    fn detect_env(sig: &syn::Signature) -> ReceiverKind {
        match sig.inputs.first() {
            Some(syn::FnArg::Receiver(r)) => {
                if r.reference.is_some() {
                    if r.mutability.is_some() {
                        ReceiverKind::RefMut
                    } else {
                        ReceiverKind::Ref
                    }
                } else {
                    ReceiverKind::Value
                }
            }
            _ => ReceiverKind::None,
        }
    }

    /// Create signature without env (for C wrapper generation).
    fn sig_without_env(sig: &syn::Signature) -> syn::Signature {
        let mut sig = sig.clone();
        if let Some(syn::FnArg::Receiver(_)) = sig.inputs.first() {
            sig.inputs = sig.inputs.into_iter().skip(1).collect();
        }
        sig
    }

    /// Parse a method from an impl item.
    ///
    /// Regular doc comments are auto-converted to `@description` for all class systems.
    pub fn from_impl_item(item: syn::ImplItemFn, _class_system: ClassSystem) -> syn::Result<Self> {
        let env = Self::detect_env(&item.sig);
        let method_attrs = Self::parse_method_attrs(&item.attrs)?;

        // Validate: no defaults on self parameter (any kind: &self, &mut self, self)
        if env != ReceiverKind::None && method_attrs.defaults.contains_key("self") {
            return Err(syn::Error::new(
                item.sig.ident.span(),
                "cannot specify default for self parameter in defaults(...)",
            ));
        }

        // Validate: all defaults reference existing parameters
        let param_names: std::collections::HashSet<String> = item
            .sig
            .inputs
            .iter()
            .filter_map(|input| {
                if let syn::FnArg::Typed(pat_type) = input
                    && let syn::Pat::Ident(pat_ident) = pat_type.pat.as_ref()
                {
                    Some(pat_ident.ident.to_string())
                } else {
                    None
                }
            })
            .collect();

        let mut invalid_params: Vec<String> = method_attrs
            .defaults
            .keys()
            .filter(|key| *key != "self" && !param_names.contains(*key))
            .cloned()
            .collect();
        invalid_params.sort();

        if !invalid_params.is_empty() {
            return Err(syn::Error::new(
                item.sig.ident.span(),
                format!(
                    "defaults(...) references non-existent parameter(s): {}",
                    invalid_params.join(", ")
                ),
            ));
        }

        // Auto-convert regular doc comments to @description for all class systems
        let doc_tags = crate::roxygen::roxygen_tags_from_attrs_for_r6_method(&item.attrs);

        // Check for @title/@description conflicts with implicit values
        crate::roxygen::warn_on_doc_conflicts(&item.attrs, item.sig.ident.span());

        // Get parameter defaults from method-level #[miniextendr(defaults(...))] attribute
        let param_defaults = method_attrs.defaults.clone();

        Ok(ParsedMethod {
            ident: item.sig.ident.clone(),
            env,
            sig: Self::sig_without_env(&item.sig),
            vis: item.vis,
            doc_tags,
            method_attrs,
            param_defaults,
        })
    }

    /// Returns true if this method should be included in the class.
    pub fn should_include(&self) -> bool {
        // Skip ignored methods
        !self.method_attrs.ignore
    }

    /// Returns true if this method should be private in R6.
    /// Inferred from Rust visibility: anything not `pub` is private.
    pub fn is_private(&self) -> bool {
        // Explicit attribute takes precedence
        if self.method_attrs.private {
            return true;
        }
        // Infer from visibility: anything not `pub` is private
        !matches!(self.vis, syn::Visibility::Public(_))
    }

    /// Returns true if this is likely a constructor.
    /// Inferred from: no env + named "new" + returns Self.
    pub fn is_constructor(&self) -> bool {
        self.method_attrs.constructor
            || (self.env == ReceiverKind::None && self.ident == "new" && self.returns_self())
    }

    /// Returns true if this is likely a finalizer.
    /// Inferred from: consumes self (by value) + doesn't return Self.
    pub fn is_finalizer(&self) -> bool {
        self.method_attrs.finalize || (self.env == ReceiverKind::Value && !self.returns_self())
    }

    /// C wrapper identifier for this method.
    pub fn c_wrapper_ident(&self, type_ident: &syn::Ident) -> syn::Ident {
        format_ident!("C_{}__{}", type_ident, self.ident)
    }

    /// Call method def identifier for registration.
    pub fn call_method_def_ident(&self, type_ident: &syn::Ident) -> syn::Ident {
        format_ident!("call_method_def_{}_{}", type_ident, self.ident)
    }

    /// Returns true if this method returns Self.
    pub fn returns_self(&self) -> bool {
        matches!(&self.sig.output, syn::ReturnType::Type(_, ty)
            if matches!(ty.as_ref(), syn::Type::Path(p)
                if p.path.segments.last().map(|s| s.ident == "Self").unwrap_or(false)))
    }

    /// Returns true if this method has no return type (returns unit `()`).
    pub fn returns_unit(&self) -> bool {
        match &self.sig.output {
            syn::ReturnType::Default => true,
            syn::ReturnType::Type(_, ty) => {
                matches!(ty.as_ref(), syn::Type::Tuple(t) if t.elems.is_empty())
            }
        }
    }
}

impl ParsedImpl {
    /// Parse an impl block with class system attribute.
    ///
    /// Note: Trait impls (`impl Trait for Type`) are handled by `expand_impl`
    /// before this function is called, so we only handle inherent impls here.
    pub fn parse(attrs: ImplAttrs, item_impl: syn::ItemImpl) -> syn::Result<Self> {
        // Extract type identifier
        let type_ident = match item_impl.self_ty.as_ref() {
            syn::Type::Path(p) => p
                .path
                .segments
                .last()
                .map(|s| s.ident.clone())
                .ok_or_else(|| syn::Error::new_spanned(&item_impl.self_ty, "expected type path"))?,
            _ => {
                return Err(syn::Error::new_spanned(
                    &item_impl.self_ty,
                    "expected type path",
                ));
            }
        };

        // Reject all generics until codegen fully supports them.
        // The wrapper generation uses `type_ident` without generic args, which would
        // fail to compile or mis-resolve types for generic impls.
        if !item_impl.generics.params.is_empty() {
            return Err(syn::Error::new_spanned(
                &item_impl.generics,
                "generic impl blocks are not yet supported by #[miniextendr]",
            ));
        }

        // Reject unsupported attributes on the impl block
        for attr in &item_impl.attrs {
            if attr.path().is_ident("export_name") {
                return Err(syn::Error::new_spanned(
                    attr,
                    "#[export_name] is not supported with #[miniextendr]; \
                     the macro generates its own C symbol names",
                ));
            }
        }

        // Parse methods and validate attributes
        let mut methods = Vec::new();
        for item in &item_impl.items {
            if let syn::ImplItem::Fn(fn_item) = item {
                let method = ParsedMethod::from_impl_item(fn_item.clone(), attrs.class_system)?;
                // Validate method attributes for this class system
                ParsedMethod::validate_method_attrs(
                    &method.method_attrs,
                    attrs.class_system,
                    fn_item.sig.ident.span(),
                )?;
                methods.push(method);
            }
        }

        // Extract cfg attributes
        let cfg_attrs: Vec<_> = item_impl
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("cfg"))
            .cloned()
            .collect();
        let doc_tags = crate::roxygen::roxygen_tags_from_attrs(&item_impl.attrs);

        Ok(ParsedImpl {
            type_ident,
            generics: item_impl.generics.clone(),
            class_system: attrs.class_system,
            class_name: attrs.class_name,
            doc_tags,
            methods,
            // Strip miniextendr attributes (and roxygen tags) before re-emitting.
            original_impl: strip_miniextendr_attrs_from_impl(item_impl),
            cfg_attrs,
        })
    }

    /// Get the class name (override or type name).
    pub fn class_name(&self) -> String {
        self.class_name
            .clone()
            .unwrap_or_else(|| self.type_ident.to_string())
    }

    /// Get methods that should be included.
    pub fn included_methods(&self) -> impl Iterator<Item = &ParsedMethod> {
        self.methods.iter().filter(|m| m.should_include())
    }

    /// Get the constructor method (fn new() -> Self), if included.
    /// Respects `#[...(ignore)]` and visibility filters.
    pub fn constructor(&self) -> Option<&ParsedMethod> {
        self.methods
            .iter()
            .find(|m| m.should_include() && m.is_constructor())
    }

    /// Get public instance methods (have env, not private).
    pub fn public_instance_methods(&self) -> impl Iterator<Item = &ParsedMethod> {
        self.methods.iter().filter(|m| {
            m.should_include()
                && m.env.is_instance()
                && !m.is_constructor()
                && !m.is_finalizer()
                && !m.is_private()
        })
    }

    /// Get private instance methods (have env, private visibility).
    pub fn private_instance_methods(&self) -> impl Iterator<Item = &ParsedMethod> {
        self.methods.iter().filter(|m| {
            m.should_include()
                && m.env.is_instance()
                && !m.is_constructor()
                && !m.is_finalizer()
                && m.is_private()
        })
    }

    /// Get instance methods (have env) - includes both public and private.
    pub fn instance_methods(&self) -> impl Iterator<Item = &ParsedMethod> {
        self.methods.iter().filter(|m| {
            m.should_include()
                && m.env.is_instance()
                && !m.is_constructor()
                && !m.is_finalizer()
        })
    }

    /// Get static methods (no env, not constructor, not finalizer).
    pub fn static_methods(&self) -> impl Iterator<Item = &ParsedMethod> {
        self.methods.iter().filter(|m| {
            m.should_include()
                && m.env == ReceiverKind::None
                && !m.is_constructor()
                && !m.is_finalizer()
        })
    }

    /// Get the finalizer method, if any.
    pub fn finalizer(&self) -> Option<&ParsedMethod> {
        self.methods
            .iter()
            .find(|m| m.should_include() && m.is_finalizer())
    }

    /// Module constant identifier for all call method defs.
    pub fn call_defs_const_ident(&self) -> syn::Ident {
        format_ident!("{}_CALL_DEFS", self.type_ident.to_string().to_uppercase())
    }

    /// Module constant identifier for R wrapper parts.
    pub fn r_wrappers_const_ident(&self) -> syn::Ident {
        format_ident!(
            "R_WRAPPERS_IMPL_{}",
            self.type_ident.to_string().to_uppercase()
        )
    }
}

/// Generate C wrapper for a method.
pub fn generate_method_c_wrapper(
    parsed_impl: &ParsedImpl,
    method: &ParsedMethod,
    r_wrappers_const: &syn::Ident,
) -> TokenStream {
    use crate::c_wrapper_builder::{CWrapperContext, ReturnHandling, ThreadStrategy};

    let type_ident = &parsed_impl.type_ident;
    let method_ident = &method.ident;
    let c_ident = method.c_wrapper_ident(type_ident);

    // Determine thread strategy
    // Instance methods must use main thread because self_ref is a borrow that can't cross threads
    // Static methods use worker thread by default, main thread only when explicitly requested
    let thread_strategy = if method.method_attrs.unsafe_main_thread || method.env.is_instance()
    {
        ThreadStrategy::MainThread
    } else {
        ThreadStrategy::WorkerThread
    };

    // Build rust argument names from the signature
    let rust_args: Vec<syn::Ident> = method
        .sig
        .inputs
        .iter()
        .filter_map(|arg| {
            if let syn::FnArg::Typed(pt) = arg
                && let syn::Pat::Ident(pat_ident) = pt.pat.as_ref()
            {
                Some(pat_ident.ident.clone())
            } else {
                None
            }
        })
        .collect();

    // Generate self extraction for instance methods
    // SEXP is now Send+Sync, so this works for both main and worker threads
    let pre_call = if method.env.is_instance() {
        let self_extraction = if method.env == ReceiverKind::RefMut {
            quote! {
                let mut self_ptr = unsafe {
                    ::miniextendr_api::externalptr::ErasedExternalPtr::from_sexp(self_sexp)
                };
                let self_ref = self_ptr.downcast_mut::<#type_ident>()
                    .expect(concat!("expected ExternalPtr<", stringify!(#type_ident), ">"));
            }
        } else {
            quote! {
                let self_ptr = unsafe {
                    ::miniextendr_api::externalptr::ErasedExternalPtr::from_sexp(self_sexp)
                };
                let self_ref = self_ptr.downcast_ref::<#type_ident>()
                    .expect(concat!("expected ExternalPtr<", stringify!(#type_ident), ">"));
            }
        };
        vec![self_extraction]
    } else {
        vec![]
    };

    // Generate call expression
    let call_expr = if method.env.is_instance() {
        quote! { self_ref.#method_ident(#(#rust_args),*) }
    } else {
        quote! { #type_ident::#method_ident(#(#rust_args),*) }
    };

    // Determine return handling strategy
    let return_handling = if method.returns_self() {
        ReturnHandling::ExternalPtr
    } else {
        crate::c_wrapper_builder::detect_return_handling(&method.sig.output)
    };

    // Build the context using the builder
    let mut builder = CWrapperContext::builder(method_ident.clone(), c_ident)
        .r_wrapper_const(r_wrappers_const.clone())
        .inputs(method.sig.inputs.clone())
        .output(method.sig.output.clone())
        .pre_call(pre_call)
        .call_expr(call_expr)
        .thread_strategy(thread_strategy)
        .return_handling(return_handling)
        .cfg_attrs(parsed_impl.cfg_attrs.clone())
        .type_context(type_ident.clone());

    if method.env.is_instance() {
        builder = builder.has_self();
    }

    if method.method_attrs.coerce {
        builder = builder.coerce_all();
    }

    if method.method_attrs.check_interrupt {
        builder = builder.check_interrupt();
    }

    builder.build().generate()
}

/// Generate R wrapper string for env-style class.
pub fn generate_env_r_wrapper(parsed_impl: &ParsedImpl) -> String {
    let class_name = parsed_impl.class_name();
    let type_ident = &parsed_impl.type_ident;
    let class_doc_tags = &parsed_impl.doc_tags;
    let class_has_title = crate::roxygen::has_roxygen_tag(class_doc_tags, "title");
    let class_has_name = crate::roxygen::has_roxygen_tag(class_doc_tags, "name");
    let class_has_rdname = crate::roxygen::has_roxygen_tag(class_doc_tags, "rdname");
    let class_has_export = crate::roxygen::has_roxygen_tag(class_doc_tags, "export");

    let mut lines = Vec::new();

    // Class environment
    if !class_has_title {
        lines.push(format!("#' @title {} Class", class_name));
    }
    if !class_has_name {
        lines.push(format!("#' @name {}", class_name));
    }
    if !class_has_rdname {
        lines.push(format!("#' @rdname {}", class_name));
    }
    crate::roxygen::push_roxygen_tags(&mut lines, class_doc_tags);
    lines.push(format!(
        "#' @source Generated by miniextendr from Rust type `{}`",
        type_ident
    ));
    if !class_has_export {
        lines.push("#' @export".to_string());
    }
    lines.push(format!("{} <- new.env(parent = emptyenv())", class_name));
    lines.push(String::new());

    // Constructor
    if let Some(ctor) = parsed_impl.constructor() {
        let c_ident = ctor.c_wrapper_ident(type_ident);
        let params = build_r_formals_with_defaults(&ctor.sig, &ctor.param_defaults);
        let args = build_r_call_args(&ctor.sig);
        let call = if args.is_empty() {
            format!(".Call({}, .call = match.call())", c_ident)
        } else {
            format!(".Call({}, .call = match.call(), {})", c_ident, args)
        };
        if !ctor.doc_tags.is_empty() {
            crate::roxygen::push_roxygen_tags(&mut lines, &ctor.doc_tags);
        }
        // Add @name and @rdname to combine methods into one doc page
        if !crate::roxygen::has_roxygen_tag(&ctor.doc_tags, "name") {
            lines.push(format!("#' @name {}$new", class_name));
        }
        if !crate::roxygen::has_roxygen_tag(&ctor.doc_tags, "rdname") {
            lines.push(format!("#' @rdname {}", class_name));
        }
        lines.push(format!("{}$new <- function({}) {{", class_name, params));
        lines.push(format!("    self <- {}", call));
        lines.push(format!("    class(self) <- \"{}\"", class_name));
        lines.push("    self".to_string());
        lines.push("}".to_string());
        lines.push(String::new());
    }

    // Instance methods
    for method in parsed_impl.instance_methods() {
        let c_ident = method.c_wrapper_ident(type_ident);
        let params = build_r_formals_with_defaults(&method.sig, &method.param_defaults);
        let args = build_r_call_args(&method.sig);
        let call = if args.is_empty() {
            format!(".Call({}, .call = match.call(), self)", c_ident)
        } else {
            format!(".Call({}, .call = match.call(), self, {})", c_ident, args)
        };
        if !method.doc_tags.is_empty() {
            crate::roxygen::push_roxygen_tags(&mut lines, &method.doc_tags);
        }
        // Add @name and @rdname to combine methods into one doc page
        if !crate::roxygen::has_roxygen_tag(&method.doc_tags, "name") {
            lines.push(format!("#' @name {}${}", class_name, method.ident));
        }
        if !crate::roxygen::has_roxygen_tag(&method.doc_tags, "rdname") {
            lines.push(format!("#' @rdname {}", class_name));
        }
        lines.push(format!(
            "{}${} <- function({}) {{",
            class_name, method.ident, params
        ));

        // Use shared return builder
        let strategy = crate::ReturnStrategy::for_method(method);
        let return_builder = crate::MethodReturnBuilder::new(call)
            .with_strategy(strategy)
            .with_class_name(class_name.clone());
        lines.extend(return_builder.build());

        lines.push("}".to_string());
        lines.push(String::new());
    }

    // Static methods
    for method in parsed_impl.static_methods() {
        let c_ident = method.c_wrapper_ident(type_ident);
        let params = build_r_formals_with_defaults(&method.sig, &method.param_defaults);
        let args = build_r_call_args(&method.sig);
        let call = if args.is_empty() {
            format!(".Call({}, .call = match.call())", c_ident)
        } else {
            format!(".Call({}, .call = match.call(), {})", c_ident, args)
        };
        if !method.doc_tags.is_empty() {
            crate::roxygen::push_roxygen_tags(&mut lines, &method.doc_tags);
        }
        // Add @name and @rdname to combine methods into one doc page
        if !crate::roxygen::has_roxygen_tag(&method.doc_tags, "name") {
            lines.push(format!("#' @name {}${}", class_name, method.ident));
        }
        if !crate::roxygen::has_roxygen_tag(&method.doc_tags, "rdname") {
            lines.push(format!("#' @rdname {}", class_name));
        }
        lines.push(format!(
            "{}${} <- function({}) {{",
            class_name, method.ident, params
        ));

        // Use shared return builder (static methods use same logic)
        let strategy = crate::ReturnStrategy::for_method(method);
        let return_builder = crate::MethodReturnBuilder::new(call)
            .with_strategy(strategy)
            .with_class_name(class_name.clone());
        lines.extend(return_builder.build());

        lines.push("}".to_string());
        lines.push(String::new());
    }

    // $ dispatch - export as S3 methods
    // Handles both functions (inherent methods) and environments (trait namespaces)
    lines.push(format!("#' @rdname {}", class_name));
    lines.push("#' @export".to_string());
    lines.push(format!("`$.{}` <- function(self, name) {{", class_name));
    lines.push(format!("    obj <- {}[[name]]", class_name));
    lines.push("    if (is.environment(obj)) {".to_string());
    lines.push("        # Trait namespace - bind self to all methods".to_string());
    lines.push("        bound <- new.env(parent = emptyenv())".to_string());
    lines.push("        for (method_name in names(obj)) {".to_string());
    lines.push("            method <- obj[[method_name]]".to_string());
    lines.push("            if (is.function(method)) {".to_string());
    lines.push("                environment(method) <- environment()".to_string());
    lines.push("                bound[[method_name]] <- method".to_string());
    lines.push("            }".to_string());
    lines.push("        }".to_string());
    lines.push("        bound".to_string());
    lines.push("    } else {".to_string());
    lines.push("        environment(obj) <- environment()".to_string());
    lines.push("        obj".to_string());
    lines.push("    }".to_string());
    lines.push("}".to_string());
    lines.push(format!("#' @rdname {}", class_name));
    lines.push("#' @export".to_string());
    lines.push(format!("`[[.{}` <- `$.{}`", class_name, class_name));

    lines.join("\n")
}

/// Generate R wrapper string for R6-style class.
///
/// Creates an R6::R6Class with:
/// - `initialize` method that calls the Rust `new` function (or accepts `.ptr` directly)
/// - Public methods for all instance methods
/// - Private `.ptr` field holding the ExternalPtr
pub fn generate_r6_r_wrapper(parsed_impl: &ParsedImpl) -> String {
    let class_name = parsed_impl.class_name();
    let type_ident = &parsed_impl.type_ident;
    let class_doc_tags = &parsed_impl.doc_tags;
    let class_has_title = crate::roxygen::has_roxygen_tag(class_doc_tags, "title");
    let class_has_name = crate::roxygen::has_roxygen_tag(class_doc_tags, "name");
    let class_has_rdname = crate::roxygen::has_roxygen_tag(class_doc_tags, "rdname");
    let class_has_export = crate::roxygen::has_roxygen_tag(class_doc_tags, "export");

    // Check if .ptr parameter will be added to initialize (for static methods returning Self)
    let has_self_returning_methods = parsed_impl
        .methods
        .iter()
        .filter(|m| m.should_include())
        .any(|m| m.returns_self());

    let mut lines = Vec::new();

    // Start R6Class definition
    if !class_has_title {
        lines.push(format!("#' @title {} R6 Class", class_name));
    }
    if !class_has_name {
        lines.push(format!("#' @name {}", class_name));
    }
    if !class_has_rdname {
        lines.push(format!("#' @rdname {}", class_name));
    }
    crate::roxygen::push_roxygen_tags(&mut lines, class_doc_tags);
    lines.push(format!(
        "#' @source Generated by miniextendr from Rust type `{}`",
        type_ident
    ));
    lines.push("#' @importFrom R6 R6Class".to_string());
    // Document .ptr param if initialize will have it (for static methods returning Self)
    if has_self_returning_methods && !crate::roxygen::has_roxygen_tag(class_doc_tags, "param .ptr")
    {
        lines.push(
            "#' @param .ptr Internal pointer (used by static methods, not for direct use)."
                .to_string(),
        );
    }
    if !class_has_export {
        lines.push("#' @export".to_string());
    }
    lines.push(format!("{} <- R6::R6Class(\"{}\",", class_name, class_name));

    // Public list
    lines.push("    public = list(".to_string());

    // Constructor (initialize) - accepts either normal params or a pre-made .ptr
    // Note: has_self_returning_methods was calculated above for @param .ptr documentation
    if let Some(ctor) = parsed_impl.constructor() {
        let c_ident = ctor.c_wrapper_ident(type_ident);
        let params = build_r_formals_with_defaults(&ctor.sig, &ctor.param_defaults);
        let args = build_r_call_args(&ctor.sig);
        let call = if args.is_empty() {
            format!(".Call({}, .call = match.call())", c_ident)
        } else {
            format!(".Call({}, .call = match.call(), {})", c_ident, args)
        };

        // Add inline roxygen documentation for initialize method
        for tag in &ctor.doc_tags {
            for line in tag.lines() {
                lines.push(format!("        #' {}", line));
            }
        }

        if has_self_returning_methods {
            let full_params = if params.is_empty() {
                ".ptr = NULL".to_string()
            } else {
                format!("{}, .ptr = NULL", params)
            };
            lines.push(format!("        initialize = function({}) {{", full_params));
            lines.push("            if (!is.null(.ptr)) {".to_string());
            lines.push("                private$.ptr <- .ptr".to_string());
            lines.push("            } else {".to_string());
            lines.push(format!("                private$.ptr <- {}", call));
            lines.push("            }".to_string());
            lines.push("        },".to_string());
        } else {
            lines.push(format!("        initialize = function({}) {{", params));
            lines.push(format!("            private$.ptr <- {}", call));
            lines.push("        },".to_string());
        }
    }

    // Public instance methods
    let public_methods: Vec<_> = parsed_impl.public_instance_methods().collect();
    for (i, method) in public_methods.iter().enumerate() {
        let c_ident = method.c_wrapper_ident(type_ident);
        let params = build_r_formals_with_defaults(&method.sig, &method.param_defaults);
        let args = build_r_call_args(&method.sig);
        let call = if args.is_empty() {
            format!(".Call({}, .call = match.call(), private$.ptr)", c_ident)
        } else {
            format!(
                ".Call({}, .call = match.call(), private$.ptr, {})",
                c_ident, args
            )
        };
        let comma = if i < public_methods.len() - 1 {
            ","
        } else {
            ""
        };

        // Add inline roxygen documentation for this method
        for tag in &method.doc_tags {
            for line in tag.lines() {
                lines.push(format!("        #' {}", line));
            }
        }

        lines.push(format!(
            "        {} = function({}) {{",
            method.ident, params
        ));

        // Use shared return builder (R6-specific)
        let strategy = crate::ReturnStrategy::for_method(method);
        let return_builder = crate::MethodReturnBuilder::new(call)
            .with_strategy(strategy)
            .with_class_name(class_name.clone())
            .with_indent(12); // R6 methods have 12-space indent
        lines.extend(return_builder.build_r6_body());

        lines.push(format!("        }}{}", comma));
    }

    lines.push("    ),".to_string());

    // Private list - includes .ptr and any private methods
    lines.push("    private = list(".to_string());

    // Private instance methods
    let private_methods: Vec<_> = parsed_impl.private_instance_methods().collect();
    for method in &private_methods {
        let c_ident = method.c_wrapper_ident(type_ident);
        let params = build_r_formals_with_defaults(&method.sig, &method.param_defaults);
        let args = build_r_call_args(&method.sig);
        let call = if args.is_empty() {
            format!(".Call({}, .call = match.call(), private$.ptr)", c_ident)
        } else {
            format!(
                ".Call({}, .call = match.call(), private$.ptr, {})",
                c_ident, args
            )
        };
        lines.push(format!(
            "        {} = function({}) {{",
            method.ident, params
        ));

        let strategy = crate::ReturnStrategy::for_method(method);
        let return_builder = crate::MethodReturnBuilder::new(call)
            .with_strategy(strategy)
            .with_class_name(class_name.clone())
            .with_indent(12);
        lines.extend(return_builder.build_r6_body());

        lines.push("        },".to_string());
    }

    // Finalizer (if any)
    if let Some(finalizer) = parsed_impl.finalizer() {
        let c_ident = finalizer.c_wrapper_ident(type_ident);
        lines.push(format!(
            "        finalize = function() .Call({}, .call = match.call(), private$.ptr),",
            c_ident
        ));
    }

    // .ptr field (always last, no trailing comma)
    lines.push("        .ptr = NULL".to_string());
    lines.push("    ),".to_string());

    // Class options
    lines.push("    lock_objects = TRUE,".to_string());
    lines.push("    lock_class = FALSE,".to_string());
    lines.push("    cloneable = FALSE".to_string());
    lines.push(")".to_string());

    // Static methods as separate functions on the class object
    for method in parsed_impl.static_methods() {
        let c_ident = method.c_wrapper_ident(type_ident);
        let params = build_r_formals_with_defaults(&method.sig, &method.param_defaults);
        let args = build_r_call_args(&method.sig);
        let call = if args.is_empty() {
            format!(".Call({}, .call = match.call())", c_ident)
        } else {
            format!(".Call({}, .call = match.call(), {})", c_ident, args)
        };
        // Static method name like "R6Counter$default_counter"
        let static_method_name = format!("{}${}", class_name, method.ident);
        lines.push(String::new());
        if !method.doc_tags.is_empty() {
            crate::roxygen::push_roxygen_tags(&mut lines, &method.doc_tags);
        }
        // Add @name so roxygen knows what the block documents (attached to Class$method assignment)
        if !crate::roxygen::has_roxygen_tag(&method.doc_tags, "name") {
            lines.push(format!("#' @name {}", static_method_name));
        }
        if !crate::roxygen::has_roxygen_tag(&method.doc_tags, "rdname") {
            lines.push(format!("#' @rdname {}", class_name));
        }
        lines.push(format!("{} <- function({}) {{", static_method_name, params));

        // Use shared return builder (R6-specific)
        let strategy = crate::ReturnStrategy::for_method(method);
        let return_builder = crate::MethodReturnBuilder::new(call)
            .with_strategy(strategy)
            .with_class_name(class_name.clone());
        lines.extend(return_builder.build_r6_body());

        lines.push("}".to_string());
    }

    lines.join("\n")
}

/// Generate R wrapper string for S3-style class.
///
/// Creates:
/// - Constructor function `new_<class>()` that returns an ExternalPtr with class attribute
/// - S3 generic methods `<method>.<class>` for each instance method
pub fn generate_s3_r_wrapper(parsed_impl: &ParsedImpl) -> String {
    let class_name = parsed_impl.class_name();
    let type_ident = &parsed_impl.type_ident;
    // S3 convention: lowercase constructor name
    let ctor_name = format!("new_{}", class_name.to_lowercase());
    let class_doc_tags = &parsed_impl.doc_tags;

    let mut lines = Vec::new();

    // Constructor
    if let Some(ctor) = parsed_impl.constructor() {
        let c_ident = ctor.c_wrapper_ident(type_ident);
        let params = build_r_formals_with_defaults(&ctor.sig, &ctor.param_defaults);
        let args = build_r_call_args(&ctor.sig);
        let call = if args.is_empty() {
            format!(".Call({}, .call = match.call())", c_ident)
        } else {
            format!(".Call({}, .call = match.call(), {})", c_ident, args)
        };
        let mut ctor_doc_tags = Vec::new();
        ctor_doc_tags.extend(class_doc_tags.iter().cloned());
        ctor_doc_tags.extend(ctor.doc_tags.iter().cloned());
        let has_title = crate::roxygen::has_roxygen_tag(&ctor_doc_tags, "title");
        let has_name = crate::roxygen::has_roxygen_tag(&ctor_doc_tags, "name");
        let has_rdname = crate::roxygen::has_roxygen_tag(&ctor_doc_tags, "rdname");
        let has_export = crate::roxygen::has_roxygen_tag(&ctor_doc_tags, "export");
        if !has_title {
            lines.push(format!("#' @title {} S3 Class", class_name));
        }
        if !has_name {
            lines.push(format!("#' @name {}", class_name));
        }
        if !has_rdname {
            lines.push(format!("#' @rdname {}", class_name));
        }
        crate::roxygen::push_roxygen_tags(&mut lines, &ctor_doc_tags);
        lines.push(format!(
            "#' @source Generated by miniextendr from `{}::new`",
            type_ident
        ));
        if !has_export {
            lines.push("#' @export".to_string());
        }
        lines.push(format!("{} <- function({}) {{", ctor_name, params));
        lines.push(format!(
            "    structure({}, class = \"{}\")",
            call, class_name
        ));
        lines.push("}".to_string());
        lines.push(String::new());
    }

    // Instance methods as S3 generics + methods
    for method in parsed_impl.instance_methods() {
        let c_ident = method.c_wrapper_ident(type_ident);
        let params = build_r_formals_with_defaults(&method.sig, &method.param_defaults);
        let args = build_r_call_args(&method.sig);

        // Use generic override if provided, otherwise use method name
        let generic_name = method
            .method_attrs
            .generic
            .clone()
            .unwrap_or_else(|| method.ident.to_string());

        // S3 method: generic.class
        let s3_method_name = format!("{}.{}", generic_name, class_name);

        // Build full parameter list (x first, then others)
        let full_params = if params.is_empty() {
            "x, ...".to_string()
        } else {
            format!("x, {}, ...", params)
        };

        let call = if args.is_empty() {
            format!(".Call({}, .call = match.call(), x)", c_ident)
        } else {
            format!(".Call({}, .call = match.call(), x, {})", c_ident, args)
        };

        // Only create the S3 generic if no generic override was provided
        // (i.e., we're creating a new generic, not implementing a method for an existing one)
        let skip_generic_creation = method.method_attrs.generic.is_some();

        if !skip_generic_creation {
            // Create the S3 generic (only for custom generics, not base R overrides)
            // Use conditional definition to avoid overwriting existing generics.
            // @name is required because roxygen2 can't parse conditional definitions.
            lines.push(format!("#' S3 generic for `{}`", generic_name));
            lines.push(format!("#' @name {}", generic_name));
            lines.push("#' @param x An object".to_string());
            lines.push("#' @param ... Additional arguments passed to methods".to_string());
            lines.push(format!(
                "#' @source Generated by miniextendr from `{}::{}`",
                type_ident, method.ident
            ));
            lines.push("#' @export".to_string());
            lines.push(format!(
                "if (!exists(\"{generic_name}\", mode = \"function\")) {{
    {generic_name} <- function(x, ...) UseMethod(\"{generic_name}\")
}}"
            ));
            lines.push(String::new());
        }

        // Then create the S3 method
        if !method.doc_tags.is_empty() {
            crate::roxygen::push_roxygen_tags(&mut lines, &method.doc_tags);
        }
        if !crate::roxygen::has_roxygen_tag(&method.doc_tags, "rdname") {
            lines.push(format!("#' @rdname {}", class_name));
        }
        if !crate::roxygen::has_roxygen_tag(&method.doc_tags, "export") {
            lines.push("#' @export".to_string());
        }
        lines.push(format!("#' @method {} {}", generic_name, class_name));
        lines.push(format!(
            "{} <- function({}) {{",
            s3_method_name, full_params
        ));

        // Use shared return builder (S3-specific)
        let strategy = crate::ReturnStrategy::for_method(method);
        let return_builder = crate::MethodReturnBuilder::new(call)
            .with_strategy(strategy)
            .with_class_name(class_name.clone())
            .with_chain_var("x".to_string());
        lines.extend(return_builder.build_s3_body());

        lines.push("}".to_string());
        lines.push(String::new());
    }

    // Static methods as regular functions
    for method in parsed_impl.static_methods() {
        let c_ident = method.c_wrapper_ident(type_ident);
        let params = build_r_formals_with_defaults(&method.sig, &method.param_defaults);
        let args = build_r_call_args(&method.sig);
        let call = if args.is_empty() {
            format!(".Call({}, .call = match.call())", c_ident)
        } else {
            format!(".Call({}, .call = match.call(), {})", c_ident, args)
        };

        // Static methods get a prefix to avoid naming conflicts
        let fn_name = format!("{}_{}", class_name.to_lowercase(), method.ident);

        if !method.doc_tags.is_empty() {
            crate::roxygen::push_roxygen_tags(&mut lines, &method.doc_tags);
        }
        if !crate::roxygen::has_roxygen_tag(&method.doc_tags, "rdname") {
            lines.push(format!("#' @rdname {}", class_name));
        }
        lines.push(format!(
            "#' @source Generated by miniextendr from `{}::{}`",
            type_ident, method.ident
        ));
        if !crate::roxygen::has_roxygen_tag(&method.doc_tags, "export") {
            lines.push("#' @export".to_string());
        }
        lines.push(format!("{} <- function({}) {{", fn_name, params));

        // Use shared return builder (S3-specific)
        let strategy = crate::ReturnStrategy::for_method(method);
        let return_builder = crate::MethodReturnBuilder::new(call)
            .with_strategy(strategy)
            .with_class_name(class_name.clone());
        lines.extend(return_builder.build_s3_body());

        lines.push("}".to_string());
        lines.push(String::new());
    }

    lines.join("\n")
}

/// Generate R wrapper string for S7-style class.
///
/// Creates:
/// - S7::new_class with constructor and .ptr property
/// - S7::new_generic + S7::method for each instance method
pub fn generate_s7_r_wrapper(parsed_impl: &ParsedImpl) -> String {
    let class_name = parsed_impl.class_name();
    let type_ident = &parsed_impl.type_ident;
    let class_doc_tags = &parsed_impl.doc_tags;
    let class_has_title = crate::roxygen::has_roxygen_tag(class_doc_tags, "title");
    let class_has_name = crate::roxygen::has_roxygen_tag(class_doc_tags, "name");
    let class_has_rdname = crate::roxygen::has_roxygen_tag(class_doc_tags, "rdname");
    let class_has_export = crate::roxygen::has_roxygen_tag(class_doc_tags, "export");

    let mut lines = Vec::new();

    // Class definition
    if !class_has_title {
        lines.push(format!("#' @title {} S7 Class", class_name));
    }
    if !class_has_name {
        lines.push(format!("#' @name {}", class_name));
    }
    if !class_has_rdname {
        lines.push(format!("#' @rdname {}", class_name));
    }
    crate::roxygen::push_roxygen_tags(&mut lines, class_doc_tags);
    lines.push(format!(
        "#' @source Generated by miniextendr from Rust type `{}`",
        type_ident
    ));
    lines.push(
        "#' @importFrom S7 new_class class_any new_object S7_object new_generic method".to_string(),
    );
    if !class_has_export {
        lines.push("#' @export".to_string());
    }
    lines.push(format!(
        "{} <- S7::new_class(\"{}\",",
        class_name, class_name
    ));

    // Properties - .ptr holds the ExternalPtr
    lines.push("    properties = list(".to_string());
    lines.push("        .ptr = S7::class_any".to_string());
    lines.push("    ),".to_string());

    // Constructor - add .ptr param if ANY method returns Self
    let has_self_returning_methods = parsed_impl
        .methods
        .iter()
        .filter(|m| m.should_include())
        .any(|m| m.returns_self());

    if let Some(ctor) = parsed_impl.constructor() {
        let c_ident = ctor.c_wrapper_ident(type_ident);
        let params = build_r_formals_with_defaults(&ctor.sig, &ctor.param_defaults);
        let args = build_r_call_args(&ctor.sig);
        let call = if args.is_empty() {
            format!(".Call({}, .call = match.call())", c_ident)
        } else {
            format!(".Call({}, .call = match.call(), {})", c_ident, args)
        };
        if has_self_returning_methods {
            let params_with_ptr = if params.is_empty() {
                ".ptr = NULL".to_string()
            } else {
                format!("{}, .ptr = NULL", params)
            };
            lines.push(format!(
                "    constructor = function({}) {{",
                params_with_ptr
            ));
            lines.push("        if (!is.null(.ptr)) {".to_string());
            lines.push("            S7::new_object(S7::S7_object(), .ptr = .ptr)".to_string());
            lines.push("        } else {".to_string());
            lines.push(format!(
                "            S7::new_object(S7::S7_object(), .ptr = {})",
                call
            ));
            lines.push("        }".to_string());
            lines.push("    }".to_string());
        } else {
            lines.push(format!("    constructor = function({}) {{", params));
            lines.push(format!(
                "        S7::new_object(S7::S7_object(), .ptr = {})",
                call
            ));
            lines.push("    }".to_string());
        }
    }

    lines.push(")".to_string());
    lines.push(String::new());

    // Instance methods as S7 generics + methods
    for method in parsed_impl.instance_methods() {
        let c_ident = method.c_wrapper_ident(type_ident);
        let params = build_r_formals_with_defaults(&method.sig, &method.param_defaults);
        let args = build_r_call_args(&method.sig);

        // Use generic override if provided, otherwise use method name
        let generic_name = method
            .method_attrs
            .generic
            .clone()
            .unwrap_or_else(|| method.ident.to_string());

        // Check if this references an external generic (e.g., "base::print" or "pkg::func")
        let is_external_generic = method.method_attrs.generic.is_some();

        let call = if args.is_empty() {
            format!(".Call({}, .call = match.call(), x@.ptr)", c_ident)
        } else {
            format!(".Call({}, .call = match.call(), x@.ptr, {})", c_ident, args)
        };

        // Build full parameter list (x first, then others, then ...)
        let full_params = if params.is_empty() {
            "x, ...".to_string()
        } else {
            format!("x, {}, ...", params)
        };

        // Define generic (only if doesn't exist)
        // Note: The second arg to new_generic is the dispatch argument name (e.g., "x"), not the class
        // Add @name so roxygen knows what the block documents (it's attached to an if statement)
        if !method.doc_tags.is_empty() {
            crate::roxygen::push_roxygen_tags(&mut lines, &method.doc_tags);
        }
        if !crate::roxygen::has_roxygen_tag(&method.doc_tags, "name") {
            lines.push(format!("#' @name {}", generic_name));
        }
        if !crate::roxygen::has_roxygen_tag(&method.doc_tags, "rdname") {
            lines.push(format!("#' @rdname {}", class_name));
        }
        lines.push(format!(
            "#' @source Generated by miniextendr from `{}::{}`",
            type_ident, method.ident
        ));
        if !crate::roxygen::has_roxygen_tag(&method.doc_tags, "export") {
            lines.push("#' @export".to_string());
        }

        if is_external_generic {
            // Parse "pkg::name" format for external generics
            // If no "::" present, assume base R
            let (pkg, gen_name) = if generic_name.contains("::") {
                let parts: Vec<&str> = generic_name.split("::").collect();
                (parts[0].to_string(), parts[1].to_string())
            } else {
                ("base".to_string(), generic_name.clone())
            };

            // Use S7::new_external_generic for existing generics from other packages
            lines.push(format!(
                "if (!exists(\"{gen_name}\", mode = \"function\")) {gen_name} <- S7::new_external_generic(\"{pkg}\", \"{gen_name}\")"
            ));

            // Define method using the resolved generic name
            let strategy = crate::ReturnStrategy::for_method(method);
            let return_expr = crate::MethodReturnBuilder::new(call.clone())
                .with_strategy(strategy)
                .with_class_name(class_name.clone())
                .build_s7_inline();
            lines.push(format!(
                "S7::method({gen_name}, {class_name}) <- function({full_params}) {return_expr}"
            ));
        } else {
            // Create new S7 generic if it doesn't exist
            lines.push(format!(
                "if (!exists(\"{generic_name}\", mode = \"function\")) {generic_name} <- S7::new_generic(\"{generic_name}\", \"x\", function(x, ...) S7::S7_dispatch())"
            ));

            // Define method
            let strategy = crate::ReturnStrategy::for_method(method);
            let return_expr = crate::MethodReturnBuilder::new(call.clone())
                .with_strategy(strategy)
                .with_class_name(class_name.clone())
                .build_s7_inline();
            lines.push(format!(
                "S7::method({generic_name}, {class_name}) <- function({full_params}) {return_expr}"
            ));
        }
        lines.push(String::new());
    }

    // Static methods as regular functions
    for method in parsed_impl.static_methods() {
        let c_ident = method.c_wrapper_ident(type_ident);
        let params = build_r_formals_with_defaults(&method.sig, &method.param_defaults);
        let args = build_r_call_args(&method.sig);
        let call = if args.is_empty() {
            format!(".Call({}, .call = match.call())", c_ident)
        } else {
            format!(".Call({}, .call = match.call(), {})", c_ident, args)
        };

        let fn_name = format!("{}_{}", class_name, method.ident);

        if !method.doc_tags.is_empty() {
            crate::roxygen::push_roxygen_tags(&mut lines, &method.doc_tags);
        }
        if !crate::roxygen::has_roxygen_tag(&method.doc_tags, "rdname") {
            lines.push(format!("#' @rdname {}", class_name));
        }
        lines.push(format!(
            "#' @source Generated by miniextendr from `{}::{}`",
            type_ident, method.ident
        ));
        if !crate::roxygen::has_roxygen_tag(&method.doc_tags, "export") {
            lines.push("#' @export".to_string());
        }
        lines.push(format!("{} <- function({}) {{", fn_name, params));

        // Use shared return builder (S7-specific inline)
        let strategy = crate::ReturnStrategy::for_method(method);
        let return_expr = crate::MethodReturnBuilder::new(call)
            .with_strategy(strategy)
            .with_class_name(class_name.clone())
            .build_s7_inline();
        lines.push(format!("    {}", return_expr));

        lines.push("}".to_string());
        lines.push(String::new());
    }

    lines.join("\n")
}

/// Generate R wrapper string for S4-style class.
///
/// Creates:
/// - setClass with ptr slot
/// - Constructor function
/// - setMethod for each instance method
pub fn generate_s4_r_wrapper(parsed_impl: &ParsedImpl) -> String {
    let class_name = parsed_impl.class_name();
    let type_ident = &parsed_impl.type_ident;
    let class_doc_tags = &parsed_impl.doc_tags;
    let class_has_title = crate::roxygen::has_roxygen_tag(class_doc_tags, "title");
    let class_has_name = crate::roxygen::has_roxygen_tag(class_doc_tags, "name");
    let class_has_rdname = crate::roxygen::has_roxygen_tag(class_doc_tags, "rdname");
    // S4 uses setClass() which doesn't need @export on the class definition
    let _class_has_export = crate::roxygen::has_roxygen_tag(class_doc_tags, "export");

    let mut lines = Vec::new();

    // Class definition
    if !class_has_title {
        lines.push(format!("#' @title {} S4 Class", class_name));
    }
    if !class_has_name {
        lines.push(format!("#' @name {}", class_name));
    }
    if !class_has_rdname {
        lines.push(format!("#' @rdname {}", class_name));
    }
    crate::roxygen::push_roxygen_tags(&mut lines, class_doc_tags);
    lines.push(format!(
        "#' @source Generated by miniextendr from Rust type `{}`",
        type_ident
    ));
    lines.push("#' @importFrom methods setClass setGeneric setMethod new isGeneric".to_string());
    lines.push(format!(
        "#' @slot ptr External pointer to Rust `{}` struct",
        type_ident
    ));
    lines.push(format!(
        "methods::setClass(\"{}\", slots = c(ptr = \"externalptr\"))",
        class_name
    ));
    lines.push(String::new());

    // Constructor function
    if let Some(ctor) = parsed_impl.constructor() {
        let c_ident = ctor.c_wrapper_ident(type_ident);
        let params = build_r_formals_with_defaults(&ctor.sig, &ctor.param_defaults);
        let args = build_r_call_args(&ctor.sig);
        let call = if args.is_empty() {
            format!(".Call({}, .call = match.call())", c_ident)
        } else {
            format!(".Call({}, .call = match.call(), {})", c_ident, args)
        };
        if !ctor.doc_tags.is_empty() {
            crate::roxygen::push_roxygen_tags(&mut lines, &ctor.doc_tags);
        }
        if !crate::roxygen::has_roxygen_tag(&ctor.doc_tags, "rdname") {
            lines.push(format!("#' @rdname {}", class_name));
        }
        lines.push(format!(
            "#' @source Generated by miniextendr from `{}::new`",
            type_ident
        ));
        if !crate::roxygen::has_roxygen_tag(&ctor.doc_tags, "export") {
            lines.push("#' @export".to_string());
        }
        lines.push(format!("{} <- function({}) {{", class_name, params));
        lines.push(format!(
            "    methods::new(\"{}\", ptr = {})",
            class_name, call
        ));
        lines.push("}".to_string());
        lines.push(String::new());
    }

    // Instance methods as S4 methods
    for method in parsed_impl.instance_methods() {
        let c_ident = method.c_wrapper_ident(type_ident);
        let method_name = if let Some(ref generic) = method.method_attrs.generic {
            generic.clone()
        } else {
            format!("s4_{}", method.ident)
        };
        let params = build_r_formals(&method.sig);
        let args = build_r_call_args(&method.sig);

        let call = if args.is_empty() {
            format!(".Call({}, .call = match.call(), x@ptr)", c_ident)
        } else {
            format!(".Call({}, .call = match.call(), x@ptr, {})", c_ident, args)
        };

        // Build full parameter list (x first, then others, then ...)
        let full_params = if params.is_empty() {
            "x, ...".to_string()
        } else {
            format!("x, {}, ...", params)
        };

        // Define generic if needed (setGeneric is idempotent for existing generics)
        // Use roxygen block to export the generic
        // Add @name so roxygen knows what the block documents (it's attached to an if statement)
        if !method.doc_tags.is_empty() {
            crate::roxygen::push_roxygen_tags(&mut lines, &method.doc_tags);
        }
        if !crate::roxygen::has_roxygen_tag(&method.doc_tags, "name") {
            lines.push(format!("#' @name {}", method_name));
        }
        if !crate::roxygen::has_roxygen_tag(&method.doc_tags, "rdname") {
            lines.push(format!("#' @rdname {}", class_name));
        }
        lines.push(format!(
            "#' @source Generated by miniextendr from `{}::{}`",
            type_ident, method.ident
        ));
        if !crate::roxygen::has_roxygen_tag(&method.doc_tags, "export") {
            lines.push("#' @export".to_string());
        }
        lines.push(format!(
            "if (!methods::isGeneric(\"{}\")) methods::setGeneric(\"{}\", function(x, ...) standardGeneric(\"{}\"))",
            method_name, method_name, method_name
        ));

        // Define method with @exportMethod for proper S4 dispatch
        lines.push(format!("#' @exportMethod {}", method_name));

        // Use shared return builder (S4-specific inline)
        let strategy = crate::ReturnStrategy::for_method(method);
        let return_expr = crate::MethodReturnBuilder::new(call)
            .with_strategy(strategy)
            .with_class_name(class_name.clone())
            .build_s4_inline();
        lines.push(format!(
            "methods::setMethod(\"{}\", \"{}\", function({}) {})",
            method_name, class_name, full_params, return_expr
        ));
        lines.push(String::new());
    }

    // Static methods as regular functions
    for method in parsed_impl.static_methods() {
        let c_ident = method.c_wrapper_ident(type_ident);
        let params = build_r_formals_with_defaults(&method.sig, &method.param_defaults);
        let args = build_r_call_args(&method.sig);
        let call = if args.is_empty() {
            format!(".Call({}, .call = match.call())", c_ident)
        } else {
            format!(".Call({}, .call = match.call(), {})", c_ident, args)
        };

        let fn_name = format!("{}_{}", class_name, method.ident);

        if !method.doc_tags.is_empty() {
            crate::roxygen::push_roxygen_tags(&mut lines, &method.doc_tags);
        }
        if !crate::roxygen::has_roxygen_tag(&method.doc_tags, "rdname") {
            lines.push(format!("#' @rdname {}", class_name));
        }
        lines.push(format!(
            "#' @source Generated by miniextendr from `{}::{}`",
            type_ident, method.ident
        ));
        if !crate::roxygen::has_roxygen_tag(&method.doc_tags, "export") {
            lines.push("#' @export".to_string());
        }
        lines.push(format!("{} <- function({}) {{", fn_name, params));

        // Use shared return builder (S4-specific inline)
        let strategy = crate::ReturnStrategy::for_method(method);
        let return_expr = crate::MethodReturnBuilder::new(call)
            .with_strategy(strategy)
            .with_class_name(class_name.clone())
            .build_s4_inline();
        lines.push(format!("    {}", return_expr));

        lines.push("}".to_string());
        lines.push(String::new());
    }

    lines.join("\n")
}

/// Build R formal parameters from a Rust signature.
/// For impl methods, this skips the self parameter.
fn build_r_formals(sig: &syn::Signature) -> String {
    build_r_formals_with_defaults(sig, &std::collections::HashMap::new())
}

/// Build R formal parameters from a Rust signature with default values.
/// For impl methods, this skips the self parameter.
fn build_r_formals_with_defaults(
    sig: &syn::Signature,
    defaults: &std::collections::HashMap<String, String>,
) -> String {
    let mut builder = crate::RArgumentBuilder::new(&sig.inputs);
    if matches!(sig.inputs.first(), Some(syn::FnArg::Receiver(_))) {
        builder = builder.skip_first(); // Skip self parameter for instance methods
    }
    builder = builder.with_defaults(defaults.clone());
    builder.build_formals()
}

/// Build R .Call arguments from a Rust signature.
/// For impl methods, this skips the self parameter.
fn build_r_call_args(sig: &syn::Signature) -> String {
    let mut builder = crate::RArgumentBuilder::new(&sig.inputs);
    if matches!(sig.inputs.first(), Some(syn::FnArg::Receiver(_))) {
        builder = builder.skip_first(); // Skip self parameter for instance methods
    }
    builder.build_call_args()
}

/// Expand a #[miniextendr(env|r6|s7|s3|s4)] impl block.
///
/// This handles two cases:
/// 1. **Inherent impls** (`impl Type`): Generate class-system wrappers
/// 2. **Trait impls** (`impl Trait for Type`): Generate vtable static for trait ABI
pub fn expand_impl(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item_impl = match syn::parse::<syn::ItemImpl>(item.clone()) {
        Ok(i) => i,
        Err(e) => return e.into_compile_error().into(),
    };

    // Check if this is a trait impl (impl Trait for Type)
    if item_impl.trait_.is_some() {
        // Delegate to trait ABI vtable generator
        return crate::miniextendr_impl_trait::expand_miniextendr_impl_trait(attr, item);
    }

    // Otherwise, this is an inherent impl - parse class system attrs
    let attrs = match syn::parse::<ImplAttrs>(attr) {
        Ok(a) => a,
        Err(e) => return e.into_compile_error().into(),
    };

    let parsed = match ParsedImpl::parse(attrs, item_impl) {
        Ok(p) => p,
        Err(e) => return e.into_compile_error().into(),
    };

    // Generate constants for module registration (needed for doc links)
    let type_ident = &parsed.type_ident;
    let cfg_attrs = &parsed.cfg_attrs;
    let r_wrappers_const = parsed.r_wrappers_const_ident();

    // Generate C wrappers for all included methods
    let c_wrappers: Vec<TokenStream> = parsed
        .included_methods()
        .map(|m| generate_method_c_wrapper(&parsed, m, &r_wrappers_const))
        .collect();

    // Generate R wrapper string based on class system
    let r_wrapper_string = match parsed.class_system {
        ClassSystem::Env => generate_env_r_wrapper(&parsed),
        ClassSystem::R6 => generate_r6_r_wrapper(&parsed),
        ClassSystem::S3 => generate_s3_r_wrapper(&parsed),
        ClassSystem::S7 => generate_s7_r_wrapper(&parsed),
        ClassSystem::S4 => generate_s4_r_wrapper(&parsed),
    };
    let call_defs_const = parsed.call_defs_const_ident();

    let call_def_idents: Vec<syn::Ident> = parsed
        .included_methods()
        .map(|m| m.call_method_def_ident(type_ident))
        .collect();
    let call_defs_len = call_def_idents.len();
    let call_defs_len_lit =
        syn::LitInt::new(&call_defs_len.to_string(), proc_macro2::Span::call_site());

    let original_impl = &parsed.original_impl;

    let r_wrapper_str: TokenStream = {
        use std::str::FromStr;
        let indented = r_wrapper_string.replace('\n', "\n    ");
        let raw = format!("r#\"\n    {}\n\"#", indented);
        TokenStream::from_str(&raw).expect("valid raw string literal")
    };

    // Generate doc comment linking to R wrapper constant
    let r_wrapper_doc = format!(
        "See [`{}`] for the generated R wrapper code.",
        r_wrappers_const
    );

    let expanded = quote! {
        // Original impl block with doc link to R wrapper
        #[doc = #r_wrapper_doc]
        #original_impl

        // C wrappers and call method defs
        #(#c_wrappers)*

        // R wrapper constant
        #(#cfg_attrs)*
        pub const #r_wrappers_const: &str = #r_wrapper_str;

        // Call method def array for module registration
        #(#cfg_attrs)*
        #[doc(hidden)]
        pub const #call_defs_const: [::miniextendr_api::ffi::R_CallMethodDef; #call_defs_len_lit] =
            [#(#call_def_idents),*];
    };

    expanded.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_wrappers_preserve_static_params() {
        let attrs = ImplAttrs {
            class_system: ClassSystem::Env,
            class_name: None,
        };

        let item_impl: syn::ItemImpl = syn::parse_quote! {
            impl ReceiverCounter {
                pub fn new(initial: i32) -> Self {
                    unimplemented!()
                }

                pub fn add(&self, amount: i32) -> i32 {
                    amount
                }

                pub fn default_counter(step: i32) -> Self {
                    unimplemented!()
                }
            }
        };

        let parsed = ParsedImpl::parse(attrs, item_impl).expect("failed to parse impl");
        let wrapper = generate_env_r_wrapper(&parsed);

        assert!(wrapper.contains("ReceiverCounter$new <- function(initial)"));
        assert!(wrapper.contains("ReceiverCounter$add <- function(amount)"));
        assert!(wrapper.contains("ReceiverCounter$default_counter <- function(step)"));
    }
}
