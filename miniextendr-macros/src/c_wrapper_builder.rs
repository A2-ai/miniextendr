//! Unified C wrapper generation for standalone functions and impl methods.
//!
//! This module provides shared infrastructure for generating C wrappers that:
//! - Handle worker thread vs main thread execution strategies
//! - Perform parameter conversion from SEXP to Rust types
//! - Convert Rust return values back to SEXP
//! - Properly handle panics and R errors
//!
//! The same infrastructure is used by both `#[miniextendr]` on standalone functions
//! and `#[miniextendr(env|r6|s3|s4|s7)]` on impl blocks.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

/// Thread execution strategy for C wrappers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadStrategy {
    /// Execute on main R thread with `with_r_unwind_protect`.
    ///
    /// Required when:
    /// - Function takes SEXP inputs (not Send)
    /// - Function returns raw SEXP
    /// - Instance method (self_ptr isn't Send)
    /// - `#[miniextendr(unsafe(main_thread))]` explicitly set
    /// - `#[miniextendr(check_interrupt)]` used
    /// - Function uses variadic dots (Dots type isn't Send)
    MainThread,

    /// Execute on worker thread with panic catching.
    ///
    /// Provides better panic handling with proper destructor cleanup.
    /// Structure:
    /// 1. Argument conversion on main thread
    /// 2. Function execution on worker thread via `run_on_worker`
    /// 3. SEXP conversion on main thread with `with_r_unwind_protect`
    WorkerThread,
}

impl ThreadStrategy {
    /// Detect thread strategy based on function characteristics.
    ///
    /// Worker thread is the default - provides proper panic handling with destructor cleanup.
    /// Main thread is only used when explicitly requested.
    ///
    /// # Arguments
    /// * `force_main_thread` - Explicit `#[miniextendr(unsafe(main_thread))]`
    #[allow(dead_code)] // Will be used when lib.rs is refactored
    pub fn detect(force_main_thread: bool) -> Self {
        if force_main_thread {
            ThreadStrategy::MainThread
        } else {
            ThreadStrategy::WorkerThread
        }
    }
}

/// Return value handling strategy.
#[derive(Debug, Clone)]
pub enum ReturnHandling {
    /// Returns unit type () - use R_NilValue
    Unit,
    /// Returns raw SEXP - pass through
    RawSexp,
    /// Returns Self - wrap in ExternalPtr
    ExternalPtr,
    /// Returns other type - use IntoR::into_sexp
    IntoR,
    /// Returns Option<()> - check for None, return R_NilValue
    OptionUnit,
    /// Returns `Option<SEXP>` - check for None, pass through
    OptionSexp,
    /// Returns `Option<T>` - check for None, use IntoR
    OptionIntoR,
    /// Returns Result<(), E> - check for Err, return R_NilValue
    ResultUnit,
    /// Returns Result<SEXP, E> - check for Err, pass through
    ResultSexp,
    /// Returns Result<T, E> - check for Err, use IntoR
    ResultIntoR,
}

/// Context for generating a C wrapper function.
///
/// This struct holds all the information needed to generate a C wrapper,
/// abstracting over differences between standalone functions and impl methods.
pub struct CWrapperContext {
    /// Rust function/method identifier
    pub fn_ident: syn::Ident,
    /// C wrapper identifier (e.g., `C_foo` or `C_Type__method`)
    pub c_ident: syn::Ident,
    /// R wrapper const identifier for doc links
    pub r_wrapper_const: syn::Ident,
    /// Function inputs (without self receiver)
    pub inputs: syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
    /// Return type (stored for builder pattern but read via detect_return_handling)
    #[allow(dead_code)]
    pub output: syn::ReturnType,
    /// Pre-call statements (e.g., self extraction for methods)
    pub pre_call: Vec<TokenStream>,
    /// The call expression (e.g., `my_func(args)` or `self_ref.method(args)`)
    pub call_expr: TokenStream,
    /// Thread strategy
    pub thread_strategy: ThreadStrategy,
    /// Return handling strategy
    pub return_handling: ReturnHandling,
    /// Enable coercion for all parameters
    pub coerce_all: bool,
    /// Parameters with individual coercion enabled
    pub coerce_params: Vec<String>,
    /// Check interrupt before call
    pub check_interrupt: bool,
    /// Use RNG state management (GetRNGstate/PutRNGstate)
    pub rng: bool,
    /// cfg attributes to propagate
    pub cfg_attrs: Vec<syn::Attribute>,
    /// Type identifier for method context (for doc generation)
    pub type_context: Option<syn::Ident>,
    /// Whether this is an instance method (has self parameter)
    pub has_self: bool,
    /// Custom call_method_def identifier (if None, uses default naming)
    pub call_method_def_ident: Option<syn::Ident>,
}

impl CWrapperContext {
    /// Create a new builder for constructing CWrapperContext.
    pub fn builder(fn_ident: syn::Ident, c_ident: syn::Ident) -> CWrapperContextBuilder {
        CWrapperContextBuilder {
            fn_ident,
            c_ident,
            r_wrapper_const: None,
            inputs: syn::punctuated::Punctuated::new(),
            output: syn::ReturnType::Default,
            pre_call: Vec::new(),
            call_expr: None,
            thread_strategy: None,
            return_handling: None,
            coerce_all: false,
            coerce_params: Vec::new(),
            check_interrupt: false,
            rng: false,
            cfg_attrs: Vec::new(),
            type_context: None,
            has_self: false,
            call_method_def_ident: None,
        }
    }

    /// Generate the C wrapper function and call_method_def.
    pub fn generate(&self) -> TokenStream {
        let c_wrapper = match self.thread_strategy {
            ThreadStrategy::MainThread => self.generate_main_thread_wrapper(),
            ThreadStrategy::WorkerThread => self.generate_worker_thread_wrapper(),
        };

        let call_method_def = self.generate_call_method_def();

        let cfg_attrs = &self.cfg_attrs;

        quote! {
            #(#cfg_attrs)*
            #c_wrapper

            #(#cfg_attrs)*
            #call_method_def
        }
    }

    /// Build C wrapper parameter list.
    fn build_c_params(&self) -> (Vec<TokenStream>, Vec<syn::Ident>, Vec<syn::Ident>) {
        let mut c_params: Vec<TokenStream> = Vec::new();
        let mut rust_args: Vec<syn::Ident> = Vec::new();
        let mut sexp_idents: Vec<syn::Ident> = Vec::new();

        // First param is always __miniextendr_call for error context
        c_params.push(quote!(__miniextendr_call: ::miniextendr_api::ffi::SEXP));

        // For instance methods, add self_sexp parameter
        if self.has_self {
            c_params.push(quote!(self_sexp: ::miniextendr_api::ffi::SEXP));
        }

        // Add regular parameters
        for (idx, arg) in self.inputs.iter().enumerate() {
            if let syn::FnArg::Typed(pt) = arg
                && let syn::Pat::Ident(pat_ident) = pt.pat.as_ref()
            {
                let ident = &pat_ident.ident;
                let param_ident = format_ident!("arg_{}", idx);

                c_params.push(quote!(#param_ident: ::miniextendr_api::ffi::SEXP));
                rust_args.push(ident.clone());
                sexp_idents.push(param_ident);
            }
        }

        (c_params, rust_args, sexp_idents)
    }

    /// Generate conversion statements for parameters.
    fn build_conversion_stmts(&self, sexp_idents: &[syn::Ident]) -> Vec<TokenStream> {
        let mut builder = crate::RustConversionBuilder::new();
        if self.coerce_all {
            builder = builder.with_coerce_all();
        }
        for param in &self.coerce_params {
            builder = builder.with_coerce_param(param.clone());
        }
        builder.build_conversions(&self.inputs, sexp_idents)
    }

    /// Build conversion statements split for worker thread execution.
    ///
    /// Returns (pre_closure, in_closure) statements:
    /// - pre_closure: Run on main thread, produce owned values to move
    /// - in_closure: Run inside worker closure, create borrows
    fn build_conversion_stmts_split(
        &self,
        sexp_idents: &[syn::Ident],
    ) -> (Vec<TokenStream>, Vec<TokenStream>) {
        let mut builder = crate::RustConversionBuilder::new();
        if self.coerce_all {
            builder = builder.with_coerce_all();
        }
        for param in &self.coerce_params {
            builder = builder.with_coerce_param(param.clone());
        }

        let mut all_pre = Vec::new();
        let mut all_in = Vec::new();

        for (arg, sexp_ident) in self.inputs.iter().zip(sexp_idents.iter()) {
            if let syn::FnArg::Typed(pat_type) = arg {
                let (owned, borrowed) = builder.build_conversion_split(pat_type, sexp_ident);
                all_pre.extend(owned);
                all_in.extend(borrowed);
            }
        }

        (all_pre, all_in)
    }

    /// Generate the main thread wrapper body.
    fn generate_main_thread_wrapper(&self) -> TokenStream {
        let c_ident = &self.c_ident;
        let (c_params, _, sexp_idents) = self.build_c_params();
        let conversion_stmts = self.build_conversion_stmts(&sexp_idents);
        let pre_call = &self.pre_call;
        let call_expr = &self.call_expr;

        let pre_call_checks = if self.check_interrupt {
            quote! {
                unsafe { ::miniextendr_api::ffi::R_CheckUserInterrupt(); }
            }
        } else {
            TokenStream::new()
        };

        let return_handling = self.generate_return_handling(call_expr);

        let doc = self.generate_doc_comment("main thread");

        if self.rng {
            // RNG variant: wrap in catch_unwind so we can call PutRNGstate before error handling
            quote! {
                #[doc = #doc]
                #[unsafe(no_mangle)]
                extern "C-unwind" fn #c_ident(#(#c_params),*) -> ::miniextendr_api::ffi::SEXP {
                    unsafe { ::miniextendr_api::ffi::GetRNGstate(); }
                    let __result = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                        ::miniextendr_api::unwind_protect::with_r_unwind_protect(
                            || {
                                #pre_call_checks
                                #(#pre_call)*
                                #(#conversion_stmts)*
                                #return_handling
                            },
                            Some(__miniextendr_call),
                        )
                    }));
                    // PutRNGstate runs after catch_unwind, before error handling
                    unsafe { ::miniextendr_api::ffi::PutRNGstate(); }
                    match __result {
                        Ok(sexp) => sexp,
                        Err(payload) => ::std::panic::resume_unwind(payload),
                    }
                }
            }
        } else {
            // Non-RNG variant: direct call to with_r_unwind_protect
            quote! {
                #[doc = #doc]
                #[unsafe(no_mangle)]
                extern "C-unwind" fn #c_ident(#(#c_params),*) -> ::miniextendr_api::ffi::SEXP {
                    ::miniextendr_api::unwind_protect::with_r_unwind_protect(
                        || {
                            #pre_call_checks
                            #(#pre_call)*
                            #(#conversion_stmts)*
                            #return_handling
                        },
                        Some(__miniextendr_call),
                    )
                }
            }
        }
    }

    /// Generate the worker thread wrapper body.
    fn generate_worker_thread_wrapper(&self) -> TokenStream {
        let c_ident = &self.c_ident;
        let (c_params, _, sexp_idents) = self.build_c_params();
        let (pre_closure_stmts, in_closure_stmts) = self.build_conversion_stmts_split(&sexp_idents);
        let pre_call = &self.pre_call;
        let call_expr = &self.call_expr;

        let pre_call_checks = if self.check_interrupt {
            quote! {
                unsafe { ::miniextendr_api::ffi::R_CheckUserInterrupt(); }
            }
        } else {
            TokenStream::new()
        };

        let (worker_body, return_conversion) = self.generate_worker_return_handling(call_expr);

        let doc = self.generate_doc_comment("worker thread");

        // RNG state management: GetRNGstate at start, PutRNGstate before returning/error handling
        let (rng_get, rng_put) = if self.rng {
            (
                quote! { unsafe { ::miniextendr_api::ffi::GetRNGstate(); } },
                quote! { unsafe { ::miniextendr_api::ffi::PutRNGstate(); } },
            )
        } else {
            (TokenStream::new(), TokenStream::new())
        };

        quote! {
            #[doc = #doc]
            #[unsafe(no_mangle)]
            extern "C-unwind" fn #c_ident(#(#c_params),*) -> ::miniextendr_api::ffi::SEXP {
                #rng_get
                let __miniextendr_panic_result = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(move || {
                    #pre_call_checks
                    #(#pre_call)*
                    // Pre-closure: conversions on main thread (owned values to move)
                    #(#pre_closure_stmts)*

                    let __miniextendr_result = ::miniextendr_api::worker::run_on_worker(move || {
                        // In-closure: borrows from moved storage
                        #(#in_closure_stmts)*
                        #worker_body
                    });

                    #return_conversion
                }));
                // PutRNGstate runs after catch_unwind, before error conversion
                #rng_put
                match __miniextendr_panic_result {
                    Ok(sexp) => sexp,
                    Err(payload) => ::miniextendr_api::worker::panic_message_to_r_errorcall(
                        ::miniextendr_api::worker::panic_payload_to_string(&payload),
                        __miniextendr_call,
                    ),
                }
            }
        }
    }

    /// Generate return handling for main thread strategy.
    fn generate_return_handling(&self, call_expr: &TokenStream) -> TokenStream {
        let fn_ident = &self.fn_ident;

        match &self.return_handling {
            ReturnHandling::Unit => {
                quote! {
                    #call_expr;
                    unsafe { ::miniextendr_api::ffi::R_NilValue }
                }
            }
            ReturnHandling::RawSexp => {
                quote! {
                    #call_expr
                }
            }
            ReturnHandling::ExternalPtr => {
                quote! {
                    let __result = #call_expr;
                    ::miniextendr_api::into_r::IntoR::into_sexp(
                        ::miniextendr_api::externalptr::ExternalPtr::new(__result)
                    )
                }
            }
            ReturnHandling::IntoR => {
                quote! {
                    let __result = #call_expr;
                    ::miniextendr_api::into_r::IntoR::into_sexp(__result)
                }
            }
            ReturnHandling::OptionUnit => {
                let error_msg = quote! {
                    concat!("miniextendr function `", stringify!(#fn_ident), "` returned None")
                };
                quote! {
                    let __result = #call_expr;
                    if __result.is_none() {
                        panic!(#error_msg);
                    }
                    unsafe { ::miniextendr_api::ffi::R_NilValue }
                }
            }
            ReturnHandling::OptionSexp => {
                let error_msg = quote! {
                    concat!("miniextendr function `", stringify!(#fn_ident), "` returned None")
                };
                quote! {
                    let __result = #call_expr;
                    match __result {
                        Some(v) => v,
                        None => panic!(#error_msg),
                    }
                }
            }
            ReturnHandling::OptionIntoR => {
                let error_msg = quote! {
                    concat!("miniextendr function `", stringify!(#fn_ident), "` returned None")
                };
                quote! {
                    let __result = #call_expr;
                    let __result = match __result {
                        Some(v) => v,
                        None => panic!(#error_msg),
                    };
                    ::miniextendr_api::into_r::IntoR::into_sexp(__result)
                }
            }
            ReturnHandling::ResultUnit => {
                quote! {
                    let __result = #call_expr;
                    if let Err(e) = __result {
                        panic!("{:?}", e);
                    }
                    unsafe { ::miniextendr_api::ffi::R_NilValue }
                }
            }
            ReturnHandling::ResultSexp => {
                quote! {
                    let __result = #call_expr;
                    match __result {
                        Ok(v) => v,
                        Err(e) => panic!("{:?}", e),
                    }
                }
            }
            ReturnHandling::ResultIntoR => {
                quote! {
                    let __result = #call_expr;
                    let __result = match __result {
                        Ok(v) => v,
                        Err(e) => panic!("{:?}", e),
                    };
                    ::miniextendr_api::into_r::IntoR::into_sexp(__result)
                }
            }
        }
    }

    /// Generate return handling for worker thread strategy.
    /// Returns (worker_body, return_conversion).
    fn generate_worker_return_handling(
        &self,
        call_expr: &TokenStream,
    ) -> (TokenStream, TokenStream) {
        let fn_ident = &self.fn_ident;

        match &self.return_handling {
            ReturnHandling::Unit => {
                let worker = quote! {
                    #call_expr;
                };
                let convert = quote! {
                    unsafe { ::miniextendr_api::ffi::R_NilValue }
                };
                (worker, convert)
            }
            ReturnHandling::RawSexp => {
                // Raw SEXP can't use worker thread - this shouldn't happen
                // but handle it gracefully
                let worker = quote! {
                    #call_expr
                };
                let convert = quote! {
                    __miniextendr_result
                };
                (worker, convert)
            }
            ReturnHandling::ExternalPtr => {
                let worker = quote! {
                    #call_expr
                };
                let convert = quote! {
                    ::miniextendr_api::unwind_protect::with_r_unwind_protect(
                        || ::miniextendr_api::into_r::IntoR::into_sexp(
                            ::miniextendr_api::externalptr::ExternalPtr::new(__miniextendr_result)
                        ),
                        None,
                    )
                };
                (worker, convert)
            }
            ReturnHandling::IntoR => {
                let worker = quote! {
                    #call_expr
                };
                let convert = quote! {
                    ::miniextendr_api::unwind_protect::with_r_unwind_protect(
                        || ::miniextendr_api::into_r::IntoR::into_sexp(__miniextendr_result),
                        None,
                    )
                };
                (worker, convert)
            }
            ReturnHandling::OptionUnit => {
                let error_msg = quote! {
                    concat!("miniextendr function `", stringify!(#fn_ident), "` returned None")
                };
                let worker = quote! {
                    let __result = #call_expr;
                    if __result.is_none() {
                        panic!(#error_msg);
                    }
                };
                let convert = quote! {
                    unsafe { ::miniextendr_api::ffi::R_NilValue }
                };
                (worker, convert)
            }
            ReturnHandling::OptionSexp => {
                let error_msg = quote! {
                    concat!("miniextendr function `", stringify!(#fn_ident), "` returned None")
                };
                let worker = quote! {
                    let __result = #call_expr;
                    match __result {
                        Some(v) => v,
                        None => panic!(#error_msg),
                    }
                };
                let convert = quote! {
                    __miniextendr_result
                };
                (worker, convert)
            }
            ReturnHandling::OptionIntoR => {
                let error_msg = quote! {
                    concat!("miniextendr function `", stringify!(#fn_ident), "` returned None")
                };
                let worker = quote! {
                    let __result = #call_expr;
                    match __result {
                        Some(v) => v,
                        None => panic!(#error_msg),
                    }
                };
                let convert = quote! {
                    ::miniextendr_api::unwind_protect::with_r_unwind_protect(
                        || ::miniextendr_api::into_r::IntoR::into_sexp(__miniextendr_result),
                        None,
                    )
                };
                (worker, convert)
            }
            ReturnHandling::ResultUnit => {
                let worker = quote! {
                    let __result = #call_expr;
                    if let Err(e) = __result {
                        panic!("{:?}", e);
                    }
                };
                let convert = quote! {
                    unsafe { ::miniextendr_api::ffi::R_NilValue }
                };
                (worker, convert)
            }
            ReturnHandling::ResultSexp => {
                let worker = quote! {
                    let __result = #call_expr;
                    match __result {
                        Ok(v) => v,
                        Err(e) => panic!("{:?}", e),
                    }
                };
                let convert = quote! {
                    __miniextendr_result
                };
                (worker, convert)
            }
            ReturnHandling::ResultIntoR => {
                let worker = quote! {
                    let __result = #call_expr;
                    match __result {
                        Ok(v) => v,
                        Err(e) => panic!("{:?}", e),
                    }
                };
                let convert = quote! {
                    ::miniextendr_api::unwind_protect::with_r_unwind_protect(
                        || ::miniextendr_api::into_r::IntoR::into_sexp(__miniextendr_result),
                        None,
                    )
                };
                (worker, convert)
            }
        }
    }

    /// Generate call_method_def constant.
    fn generate_call_method_def(&self) -> TokenStream {
        let fn_ident = &self.fn_ident;
        let c_ident = &self.c_ident;
        let (c_params, _, _) = self.build_c_params();
        let num_args = c_params.len();
        let num_args_lit = syn::LitInt::new(&num_args.to_string(), proc_macro2::Span::call_site());

        let c_ident_name = syn::LitCStr::new(
            std::ffi::CString::new(c_ident.to_string())
                .expect("valid C string")
                .as_c_str(),
            c_ident.span(),
        );

        // Use custom call_method_def_ident if set, otherwise use default naming
        let call_method_def_ident = self.call_method_def_ident.clone().unwrap_or_else(|| {
            if let Some(ref type_ident) = self.type_context {
                format_ident!("call_method_def_{}_{}", type_ident, fn_ident)
            } else {
                format_ident!("call_method_def_{}", fn_ident)
            }
        });

        // Build func_ptr_def for transmute
        let func_ptr_def: Vec<syn::Type> = (0..num_args)
            .map(|_| syn::parse_quote!(::miniextendr_api::ffi::SEXP))
            .collect();

        let item_label = if let Some(ref type_ident) = self.type_context {
            format!("`{}::{}`", type_ident, fn_ident)
        } else {
            format!("`{}`", fn_ident)
        };
        let doc = format!(
            "R call method definition for {} (C wrapper: [`{}`]).",
            item_label, c_ident
        );
        let doc_example = format!(
            "Value: `R_CallMethodDef {{ name: \"{}\", numArgs: {}, fun: <DL_FUNC> }}`",
            c_ident, num_args
        );

        quote! {
            #[doc = #doc]
            #[doc = #doc_example]
            #[allow(non_upper_case_globals)]
            #[allow(non_snake_case)]
            const #call_method_def_ident: ::miniextendr_api::ffi::R_CallMethodDef = unsafe {
                ::miniextendr_api::ffi::R_CallMethodDef {
                    name: #c_ident_name.as_ptr(),
                    fun: Some(std::mem::transmute::<
                        unsafe extern "C-unwind" fn(#(#func_ptr_def),*) -> ::miniextendr_api::ffi::SEXP,
                        unsafe extern "C-unwind" fn() -> *mut ::std::os::raw::c_void
                    >(#c_ident)),
                    numArgs: #num_args_lit,
                }
            };
        }
    }

    /// Generate doc comment for the C wrapper.
    fn generate_doc_comment(&self, thread_info: &str) -> String {
        if let Some(ref type_ident) = self.type_context {
            format!(
                "C wrapper for [`{}::{}`] ({}). See [`{}`] for R wrapper.",
                type_ident, self.fn_ident, thread_info, self.r_wrapper_const
            )
        } else {
            format!(
                "C wrapper for [`{}`] ({}). See [`{}`] for R wrapper.",
                self.fn_ident, thread_info, self.r_wrapper_const
            )
        }
    }
}

/// Builder for CWrapperContext.
pub struct CWrapperContextBuilder {
    fn_ident: syn::Ident,
    c_ident: syn::Ident,
    r_wrapper_const: Option<syn::Ident>,
    inputs: syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
    output: syn::ReturnType,
    pre_call: Vec<TokenStream>,
    call_expr: Option<TokenStream>,
    thread_strategy: Option<ThreadStrategy>,
    return_handling: Option<ReturnHandling>,
    coerce_all: bool,
    coerce_params: Vec<String>,
    check_interrupt: bool,
    rng: bool,
    cfg_attrs: Vec<syn::Attribute>,
    type_context: Option<syn::Ident>,
    has_self: bool,
    /// Custom call_method_def identifier (if not set, uses default naming)
    call_method_def_ident: Option<syn::Ident>,
}

impl CWrapperContextBuilder {
    /// Set the R wrapper constant identifier.
    pub fn r_wrapper_const(mut self, ident: syn::Ident) -> Self {
        self.r_wrapper_const = Some(ident);
        self
    }

    /// Set the function inputs.
    pub fn inputs(
        mut self,
        inputs: syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
    ) -> Self {
        self.inputs = inputs;
        self
    }

    /// Set the return type.
    pub fn output(mut self, output: syn::ReturnType) -> Self {
        self.output = output;
        self
    }

    /// Add pre-call statements.
    pub fn pre_call(mut self, stmts: Vec<TokenStream>) -> Self {
        self.pre_call = stmts;
        self
    }

    /// Set the call expression.
    pub fn call_expr(mut self, expr: TokenStream) -> Self {
        self.call_expr = Some(expr);
        self
    }

    /// Set the thread strategy explicitly.
    pub fn thread_strategy(mut self, strategy: ThreadStrategy) -> Self {
        self.thread_strategy = Some(strategy);
        self
    }

    /// Set the return handling strategy explicitly.
    pub fn return_handling(mut self, handling: ReturnHandling) -> Self {
        self.return_handling = Some(handling);
        self
    }

    /// Enable coercion for all parameters.
    pub fn coerce_all(mut self) -> Self {
        self.coerce_all = true;
        self
    }

    /// Add a parameter to coerce.
    #[allow(dead_code)] // Will be used when lib.rs is refactored
    pub fn coerce_param(mut self, param: String) -> Self {
        self.coerce_params.push(param);
        self
    }

    /// Enable interrupt checking.
    pub fn check_interrupt(mut self) -> Self {
        self.check_interrupt = true;
        self
    }

    /// Enable RNG state management (GetRNGstate/PutRNGstate).
    pub fn rng(mut self) -> Self {
        self.rng = true;
        self
    }

    /// Set cfg attributes.
    pub fn cfg_attrs(mut self, attrs: Vec<syn::Attribute>) -> Self {
        self.cfg_attrs = attrs;
        self
    }

    /// Set type context for methods.
    pub fn type_context(mut self, type_ident: syn::Ident) -> Self {
        self.type_context = Some(type_ident);
        self
    }

    /// Mark this as an instance method (has self parameter).
    pub fn has_self(mut self) -> Self {
        self.has_self = true;
        self
    }

    /// Set a custom call_method_def identifier.
    ///
    /// If not set, the default naming is used:
    /// - With type_context: `call_method_def_{type}_{method}`
    /// - Without: `call_method_def_{method}`
    pub fn call_method_def_ident(mut self, ident: syn::Ident) -> Self {
        self.call_method_def_ident = Some(ident);
        self
    }

    /// Build the CWrapperContext.
    ///
    /// # Panics
    /// Panics if required fields (call_expr, r_wrapper_const) are not set.
    pub fn build(self) -> CWrapperContext {
        let call_expr = self
            .call_expr
            .expect("call_expr is required for CWrapperContext");
        let r_wrapper_const = self
            .r_wrapper_const
            .expect("r_wrapper_const is required for CWrapperContext");

        // Detect thread strategy if not explicitly set
        // Worker thread is the default for all methods
        let thread_strategy = self.thread_strategy.unwrap_or(ThreadStrategy::WorkerThread);

        // Detect return handling if not explicitly set
        let return_handling = self
            .return_handling
            .unwrap_or_else(|| detect_return_handling(&self.output));

        CWrapperContext {
            fn_ident: self.fn_ident,
            c_ident: self.c_ident,
            r_wrapper_const,
            inputs: self.inputs,
            output: self.output,
            pre_call: self.pre_call,
            call_expr,
            thread_strategy,
            return_handling,
            coerce_all: self.coerce_all,
            coerce_params: self.coerce_params,
            check_interrupt: self.check_interrupt,
            rng: self.rng,
            cfg_attrs: self.cfg_attrs,
            type_context: self.type_context,
            has_self: self.has_self,
            call_method_def_ident: self.call_method_def_ident,
        }
    }
}

/// Detect return handling strategy from return type.
pub fn detect_return_handling(output: &syn::ReturnType) -> ReturnHandling {
    match output {
        syn::ReturnType::Default => ReturnHandling::Unit,
        syn::ReturnType::Type(_, ty) => detect_return_handling_from_type(ty),
    }
}

fn detect_return_handling_from_type(ty: &syn::Type) -> ReturnHandling {
    match ty {
        // Unit tuple ()
        syn::Type::Tuple(t) if t.elems.is_empty() => ReturnHandling::Unit,

        // Self - wrap in ExternalPtr
        syn::Type::Path(p)
            if p.path
                .segments
                .last()
                .map(|s| s.ident == "Self")
                .unwrap_or(false) =>
        {
            ReturnHandling::ExternalPtr
        }

        // SEXP - pass through
        syn::Type::Path(p)
            if p.path
                .segments
                .last()
                .map(|s| s.ident == "SEXP")
                .unwrap_or(false) =>
        {
            ReturnHandling::RawSexp
        }

        // Option<T>
        syn::Type::Path(p)
            if p.path
                .segments
                .last()
                .map(|s| s.ident == "Option")
                .unwrap_or(false) =>
        {
            if let Some(inner_ty) = first_type_argument(p.path.segments.last().unwrap()) {
                match inner_ty {
                    syn::Type::Tuple(t) if t.elems.is_empty() => ReturnHandling::OptionUnit,
                    syn::Type::Path(ip)
                        if ip
                            .path
                            .segments
                            .last()
                            .map(|s| s.ident == "SEXP")
                            .unwrap_or(false) =>
                    {
                        ReturnHandling::OptionSexp
                    }
                    _ => ReturnHandling::OptionIntoR,
                }
            } else {
                ReturnHandling::OptionIntoR
            }
        }

        // Result<T, E>
        syn::Type::Path(p)
            if p.path
                .segments
                .last()
                .map(|s| s.ident == "Result")
                .unwrap_or(false) =>
        {
            if let Some(ok_ty) = first_type_argument(p.path.segments.last().unwrap()) {
                match ok_ty {
                    syn::Type::Tuple(t) if t.elems.is_empty() => ReturnHandling::ResultUnit,
                    syn::Type::Path(ip)
                        if ip
                            .path
                            .segments
                            .last()
                            .map(|s| s.ident == "SEXP")
                            .unwrap_or(false) =>
                    {
                        ReturnHandling::ResultSexp
                    }
                    _ => ReturnHandling::ResultIntoR,
                }
            } else {
                ReturnHandling::ResultIntoR
            }
        }

        // Everything else - use IntoR
        _ => ReturnHandling::IntoR,
    }
}

fn first_type_argument(seg: &syn::PathSegment) -> Option<&syn::Type> {
    if let syn::PathArguments::AngleBracketed(ab) = &seg.arguments {
        for arg in ab.args.iter() {
            if let syn::GenericArgument::Type(ty) = arg {
                return Some(ty);
            }
        }
    }
    None
}

/// Check if a type is SEXP.
#[allow(dead_code)] // Will be used when lib.rs is refactored
pub fn is_sexp_type(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Path(p) if p
        .path
        .segments
        .last()
        .map(|s| s.ident == "SEXP")
        .unwrap_or(false))
}

/// Check if any input parameter is SEXP.
#[allow(dead_code)] // Will be used when lib.rs is refactored
pub fn has_sexp_inputs(inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>) -> bool {
    inputs.iter().any(|arg| {
        if let syn::FnArg::Typed(pat_type) = arg {
            is_sexp_type(pat_type.ty.as_ref())
        } else {
            false
        }
    })
}

#[cfg(test)]
mod tests;
