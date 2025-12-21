//! Impl-block parsing and wrapper generation for class-like Rust structs.
//!
//! This module provides shared infrastructure for all class system support:
//! - Receiver (env-style with `$`/`[[` dispatch)
//! - R6 (`R6::R6Class`)
//! - S7 (`S7::new_class`)
//! - S3 (`structure()` with class attr)
//! - S4 (`setClass`)
//!
//! The impl-block parser extracts methods and categorizes them by receiver type,
//! then class-system adapters generate appropriate R wrapper code.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

/// Check if an attribute is a class-system attribute that should be stripped.
fn is_class_system_attr(attr: &syn::Attribute) -> bool {
    let path = attr.path();
    path.is_ident("receiver")
        || path.is_ident("r6")
        || path.is_ident("s7")
        || path.is_ident("s3")
        || path.is_ident("s4")
}

/// Strip class-system attributes from an impl block to avoid unused_attributes warnings.
fn strip_class_system_attrs(mut item_impl: syn::ItemImpl) -> syn::ItemImpl {
    for item in &mut item_impl.items {
        if let syn::ImplItem::Fn(fn_item) = item {
            fn_item.attrs.retain(|attr| !is_class_system_attr(attr));
        }
    }
    item_impl
}

/// Class system flavor for wrapper generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassSystem {
    /// Environment-style with `$`/`[[` dispatch
    Receiver,
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
            "receiver" => Ok(ClassSystem::Receiver),
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
    /// No receiver - static/associated function
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
#[derive(Debug)]
pub struct ParsedMethod {
    /// Method identifier
    pub ident: syn::Ident,
    /// Receiver kind
    pub receiver: ReceiverKind,
    /// Method signature (without receiver)
    pub sig: syn::Signature,
    /// Visibility
    pub vis: syn::Visibility,
    /// Roxygen tag lines extracted from Rust doc comments
    pub doc_tags: Vec<String>,
    /// Per-method attributes for class system overrides
    pub method_attrs: MethodAttrs,
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
    /// Override generic name (S3/S4/S7)
    pub generic: Option<String>,
    /// Worker thread execution (default: auto-detect based on types)
    pub worker: bool,
    /// Force main thread execution (unsafe)
    pub unsafe_main_thread: bool,
    /// Enable R interrupt checking
    pub check_interrupt: bool,
    /// Enable coercion for this method's parameters
    pub coerce: bool,
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
        let mut class_system = ClassSystem::Receiver;
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
            // Check for old-style attributes (#[r6(...)], #[s3(...)], etc.) and reject
            let path = attr.path();
            let is_old_class_attr = path.is_ident("receiver")
                || path.is_ident("r6")
                || path.is_ident("s7")
                || path.is_ident("s3")
                || path.is_ident("s4");

            if is_old_class_attr {
                let class_system = path.get_ident().map(|i| i.to_string()).unwrap_or_default();
                return Err(syn::Error::new_spanned(
                    attr,
                    format!(
                        "#[{}(...)] is not supported; use #[miniextendr({}(...))] instead",
                        class_system, class_system
                    ),
                ));
            }

            // Parse new-style #[miniextendr(class_system(...))] attributes
            if !path.is_ident("miniextendr") {
                continue;
            }

            // Parse the nested content: miniextendr(class_system(options...))
            attr.parse_nested_meta(|meta| {
                let is_class_meta = meta.path.is_ident("receiver")
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
                }
                Ok(())
            })?;
        }

        Ok(method_attrs)
    }

    /// Detect receiver kind from function signature.
    fn detect_receiver(sig: &syn::Signature) -> ReceiverKind {
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

    /// Create signature without receiver (for C wrapper generation).
    fn sig_without_receiver(sig: &syn::Signature) -> syn::Signature {
        let mut sig = sig.clone();
        if let Some(syn::FnArg::Receiver(_)) = sig.inputs.first() {
            sig.inputs = sig.inputs.into_iter().skip(1).collect();
        }
        sig
    }

    /// Parse a method from an impl item.
    pub fn from_impl_item(item: syn::ImplItemFn) -> syn::Result<Self> {
        let receiver = Self::detect_receiver(&item.sig);
        let method_attrs = Self::parse_method_attrs(&item.attrs)?;
        let doc_tags = crate::roxygen::roxygen_tags_from_attrs(&item.attrs);

        Ok(ParsedMethod {
            ident: item.sig.ident.clone(),
            receiver,
            sig: Self::sig_without_receiver(&item.sig),
            vis: item.vis,
            doc_tags,
            method_attrs,
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
    /// Inferred from: no receiver + named "new" + returns Self.
    pub fn is_constructor(&self) -> bool {
        self.method_attrs.constructor
            || (self.receiver == ReceiverKind::None && self.ident == "new" && self.returns_self())
    }

    /// Returns true if this is likely a finalizer.
    /// Inferred from: consumes self (by value) + doesn't return Self.
    pub fn is_finalizer(&self) -> bool {
        self.method_attrs.finalize || (self.receiver == ReceiverKind::Value && !self.returns_self())
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
    pub fn parse(attrs: ImplAttrs, item_impl: syn::ItemImpl) -> syn::Result<Self> {
        // Reject trait impls
        if item_impl.trait_.is_some() {
            return Err(syn::Error::new_spanned(
                &item_impl,
                "trait impls are not supported, use inherent impl blocks",
            ));
        }

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
                let method = ParsedMethod::from_impl_item(fn_item.clone())?;
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
            // Strip class-system attrs to avoid unused_attributes warnings
            original_impl: strip_class_system_attrs(item_impl),
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

    /// Get public instance methods (have receiver, not private).
    pub fn public_instance_methods(&self) -> impl Iterator<Item = &ParsedMethod> {
        self.methods.iter().filter(|m| {
            m.should_include()
                && m.receiver.is_instance()
                && !m.is_constructor()
                && !m.is_finalizer()
                && !m.is_private()
        })
    }

    /// Get private instance methods (have receiver, private visibility).
    pub fn private_instance_methods(&self) -> impl Iterator<Item = &ParsedMethod> {
        self.methods.iter().filter(|m| {
            m.should_include()
                && m.receiver.is_instance()
                && !m.is_constructor()
                && !m.is_finalizer()
                && m.is_private()
        })
    }

    /// Get instance methods (have receiver) - includes both public and private.
    pub fn instance_methods(&self) -> impl Iterator<Item = &ParsedMethod> {
        self.methods.iter().filter(|m| {
            m.should_include()
                && m.receiver.is_instance()
                && !m.is_constructor()
                && !m.is_finalizer()
        })
    }

    /// Get static methods (no receiver, not constructor, not finalizer).
    pub fn static_methods(&self) -> impl Iterator<Item = &ParsedMethod> {
        self.methods.iter().filter(|m| {
            m.should_include()
                && m.receiver == ReceiverKind::None
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
    let thread_strategy = if method.method_attrs.unsafe_main_thread || method.receiver.is_instance()
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
    let pre_call = if method.receiver.is_instance() {
        let self_extraction = if method.receiver == ReceiverKind::RefMut {
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
    let call_expr = if method.receiver.is_instance() {
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

    if method.receiver.is_instance() {
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

/// Generate R wrapper string for receiver-style class.
pub fn generate_receiver_r_wrapper(parsed_impl: &ParsedImpl) -> String {
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
        let params = build_r_formals(&ctor.sig);
        let args = build_r_call_args(&ctor.sig);
        let call = if args.is_empty() {
            format!(".Call({}, .call = match.call())", c_ident)
        } else {
            format!(".Call({}, .call = match.call(), {})", c_ident, args)
        };
        if !ctor.doc_tags.is_empty() {
            crate::roxygen::push_roxygen_tags(&mut lines, &ctor.doc_tags);
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
        let params = build_r_formals(&method.sig);
        let args = build_r_call_args(&method.sig);
        let call = if args.is_empty() {
            format!(".Call({}, .call = match.call(), self)", c_ident)
        } else {
            format!(".Call({}, .call = match.call(), self, {})", c_ident, args)
        };
        if !method.doc_tags.is_empty() {
            crate::roxygen::push_roxygen_tags(&mut lines, &method.doc_tags);
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
        let params = build_r_formals(&method.sig);
        let args = build_r_call_args(&method.sig);
        let call = if args.is_empty() {
            format!(".Call({}, .call = match.call())", c_ident)
        } else {
            format!(".Call({}, .call = match.call(), {})", c_ident, args)
        };
        if !method.doc_tags.is_empty() {
            crate::roxygen::push_roxygen_tags(&mut lines, &method.doc_tags);
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
    lines.push(format!("#' @rdname {}", class_name));
    lines.push("#' @export".to_string());
    lines.push(format!("`$.{}` <- function(self, name) {{", class_name));
    lines.push(format!("    func <- {}[[name]]", class_name));
    lines.push("    environment(func) <- environment()".to_string());
    lines.push("    func".to_string());
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
    lines.push(format!(
        "#' @field .ptr (private) External pointer to Rust `{}` struct",
        type_ident
    ));
    if !class_has_export {
        lines.push("#' @export".to_string());
    }
    lines.push(format!("{} <- R6::R6Class(\"{}\",", class_name, class_name));

    // Public list
    lines.push("    public = list(".to_string());

    // Constructor (initialize) - accepts either normal params or a pre-made .ptr
    // Add .ptr parameter if ANY method (instance or static) returns Self
    let has_self_returning_methods = parsed_impl
        .methods
        .iter()
        .filter(|m| m.should_include())
        .any(|m| m.returns_self());

    if let Some(ctor) = parsed_impl.constructor() {
        let c_ident = ctor.c_wrapper_ident(type_ident);
        let params = build_r_formals(&ctor.sig);
        let args = build_r_call_args(&ctor.sig);
        let call = if args.is_empty() {
            format!(".Call({}, .call = match.call())", c_ident)
        } else {
            format!(".Call({}, .call = match.call(), {})", c_ident, args)
        };
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
        let params = build_r_formals(&method.sig);
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
        let params = build_r_formals(&method.sig);
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
        let params = build_r_formals(&method.sig);
        let args = build_r_call_args(&method.sig);
        let call = if args.is_empty() {
            format!(".Call({}, .call = match.call())", c_ident)
        } else {
            format!(".Call({}, .call = match.call(), {})", c_ident, args)
        };
        lines.push(String::new());
        if !method.doc_tags.is_empty() {
            crate::roxygen::push_roxygen_tags(&mut lines, &method.doc_tags);
        }
        if !crate::roxygen::has_roxygen_tag(&method.doc_tags, "rdname") {
            lines.push(format!("#' @rdname {}", class_name));
        }
        lines.push(format!(
            "{}${} <- function({}) {{",
            class_name, method.ident, params
        ));

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
        let params = build_r_formals(&ctor.sig);
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
        let params = build_r_formals(&method.sig);
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
            // Use conditional creation to avoid overwriting existing generics
            lines.push(format!(
                "#' @source Generated by miniextendr from `{}::{}`",
                type_ident, method.ident
            ));
            lines.push("#' @export".to_string());
            lines.push(format!(
                "if (!exists(\"{generic_name}\", mode = \"function\")) {generic_name} <- function(x, ...) UseMethod(\"{generic_name}\")"
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
        let params = build_r_formals(&method.sig);
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
        let params = build_r_formals(&ctor.sig);
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
        let params = build_r_formals(&method.sig);
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
        let params = build_r_formals(&method.sig);
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
        let params = build_r_formals(&ctor.sig);
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
        let params = build_r_formals(&method.sig);
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
    let mut builder = crate::RArgumentBuilder::new(&sig.inputs);
    if matches!(sig.inputs.first(), Some(syn::FnArg::Receiver(_))) {
        builder = builder.skip_first(); // Skip self parameter for instance methods
    }
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

/// Expand a #[miniextendr(receiver|r6|s7|s3|s4)] impl block.
pub fn expand_impl(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let attrs = match syn::parse::<ImplAttrs>(attr) {
        Ok(a) => a,
        Err(e) => return e.into_compile_error().into(),
    };

    let item_impl = match syn::parse::<syn::ItemImpl>(item) {
        Ok(i) => i,
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
        ClassSystem::Receiver => generate_receiver_r_wrapper(&parsed),
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
            [#(#call_def_idents()),*];
    };

    expanded.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn receiver_wrappers_preserve_static_params() {
        let attrs = ImplAttrs {
            class_system: ClassSystem::Receiver,
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
        let wrapper = generate_receiver_r_wrapper(&parsed);

        assert!(wrapper.contains("ReceiverCounter$new <- function(initial)"));
        assert!(wrapper.contains("ReceiverCounter$add <- function(amount)"));
        assert!(wrapper.contains("ReceiverCounter$default_counter <- function(step)"));
    }
}
