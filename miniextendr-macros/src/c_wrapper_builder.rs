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
    /// Return type (used for strict-mode type inspection)
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
    /// Strict conversion mode: use checked conversions for lossy return types.
    pub strict: bool,
    /// error_in_r mode: return tagged error values instead of calling r_stop.
    pub error_in_r: bool,
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
            strict: false,
            error_in_r: false,
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
        if self.strict {
            builder = builder.with_strict();
        }
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
        if self.strict {
            builder = builder.with_strict();
        }
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
        let source_loc_doc = crate::source_location_doc(self.fn_ident.span());

        // Select unwind protection function: error_in_r returns tagged error values on panic
        let unwind_protect_fn = if self.error_in_r {
            quote! { ::miniextendr_api::unwind_protect::with_r_unwind_protect_error_in_r }
        } else {
            quote! { ::miniextendr_api::unwind_protect::with_r_unwind_protect }
        };

        if self.rng {
            // RNG variant: wrap in catch_unwind so we can call PutRNGstate before error handling
            let rng_panic_handler = if self.error_in_r {
                quote! {
                    ::miniextendr_api::error_value::make_rust_error_value(
                        &::miniextendr_api::worker::panic_payload_to_string(&payload),
                        "panic",
                    )
                }
            } else {
                quote! { ::std::panic::resume_unwind(payload) }
            };
            quote! {
                #[doc = #doc]
                #[doc = #source_loc_doc]
                #[doc = concat!("Generated from source file `", file!(), "`.")]
                #[unsafe(no_mangle)]
                extern "C-unwind" fn #c_ident(#(#c_params),*) -> ::miniextendr_api::ffi::SEXP {
                    unsafe { ::miniextendr_api::ffi::GetRNGstate(); }
                    let __result = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                        #unwind_protect_fn(
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
                        Err(payload) => { #rng_panic_handler },
                    }
                }
            }
        } else {
            // Non-RNG variant: direct call to with_r_unwind_protect
            quote! {
                #[doc = #doc]
                #[doc = #source_loc_doc]
                #[doc = concat!("Generated from source file `", file!(), "`.")]
                #[unsafe(no_mangle)]
                extern "C-unwind" fn #c_ident(#(#c_params),*) -> ::miniextendr_api::ffi::SEXP {
                    #unwind_protect_fn(
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
        let source_loc_doc = crate::source_location_doc(self.fn_ident.span());

        // RNG state management: GetRNGstate at start, PutRNGstate before returning/error handling
        let (rng_get, rng_put) = if self.rng {
            (
                quote! { unsafe { ::miniextendr_api::ffi::GetRNGstate(); } },
                quote! { unsafe { ::miniextendr_api::ffi::PutRNGstate(); } },
            )
        } else {
            (TokenStream::new(), TokenStream::new())
        };

        // Panic error handling: in error_in_r mode, return tagged error value;
        // otherwise, raise R error via Rf_errorcall.
        let panic_error_handling = if self.error_in_r {
            quote! {
                ::miniextendr_api::error_value::make_rust_error_value(
                    &::miniextendr_api::worker::panic_payload_to_string(&payload),
                    "panic",
                )
            }
        } else {
            quote! {
                ::miniextendr_api::worker::panic_message_to_r_errorcall(
                    ::miniextendr_api::worker::panic_payload_to_string(&payload),
                    __miniextendr_call,
                )
            }
        };

        if self.error_in_r {
            // error_in_r: use run_on_worker_result to get Result<T, String> instead of diverging
            quote! {
                #[doc = #doc]
                #[doc = #source_loc_doc]
                #[doc = concat!("Generated from source file `", file!(), "`.")]
                #[unsafe(no_mangle)]
                extern "C-unwind" fn #c_ident(#(#c_params),*) -> ::miniextendr_api::ffi::SEXP {
                    #rng_get
                    let __miniextendr_panic_result = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(move || {
                        #pre_call_checks
                        #(#pre_call)*
                        #(#pre_closure_stmts)*

                        match ::miniextendr_api::worker::run_on_worker_result(move || {
                            #(#in_closure_stmts)*
                            #worker_body
                        }) {
                            Ok(__miniextendr_result) => {
                                #return_conversion
                            }
                            Err(__panic_msg) => {
                                ::miniextendr_api::error_value::make_rust_error_value(
                                    &__panic_msg, "panic",
                                )
                            }
                        }
                    }));
                    #rng_put
                    match __miniextendr_panic_result {
                        Ok(sexp) => sexp,
                        Err(payload) => {
                            #panic_error_handling
                        },
                    }
                }
            }
        } else {
            quote! {
                #[doc = #doc]
                #[doc = #source_loc_doc]
                #[doc = concat!("Generated from source file `", file!(), "`.")]
                #[unsafe(no_mangle)]
                extern "C-unwind" fn #c_ident(#(#c_params),*) -> ::miniextendr_api::ffi::SEXP {
                    #rng_get
                    let __miniextendr_panic_result = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(move || {
                        #pre_call_checks
                        #(#pre_call)*
                        #(#pre_closure_stmts)*

                        let __miniextendr_result = ::miniextendr_api::worker::run_on_worker(move || {
                            #(#in_closure_stmts)*
                            #worker_body
                        });

                        #return_conversion
                    }));
                    #rng_put
                    match __miniextendr_panic_result {
                        Ok(sexp) => sexp,
                        Err(payload) => {
                            #panic_error_handling
                        },
                    }
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
                let result_ident = format_ident!("__result");
                let conversion = self.sexp_conversion_expr(&result_ident);
                quote! {
                    let #result_ident = #call_expr;
                    #conversion
                }
            }
            ReturnHandling::OptionUnit => {
                let error_msg = format!("miniextendr function `{}` returned None", fn_ident);
                if self.error_in_r {
                    quote! {
                        let __result = #call_expr;
                        if __result.is_none() {
                            return ::miniextendr_api::error_value::make_rust_error_value(
                                #error_msg, "none_err"
                            );
                        }
                        unsafe { ::miniextendr_api::ffi::R_NilValue }
                    }
                } else {
                    quote! {
                        let __result = #call_expr;
                        if __result.is_none() {
                            ::miniextendr_api::error::r_stop(#error_msg);
                        }
                        unsafe { ::miniextendr_api::ffi::R_NilValue }
                    }
                }
            }
            ReturnHandling::OptionSexp => {
                let error_msg = format!("miniextendr function `{}` returned None", fn_ident);
                if self.error_in_r {
                    quote! {
                        let __result = #call_expr;
                        match __result {
                            Some(v) => v,
                            None => return ::miniextendr_api::error_value::make_rust_error_value(
                                #error_msg, "none_err"
                            ),
                        }
                    }
                } else {
                    quote! {
                        let __result = #call_expr;
                        match __result {
                            Some(v) => v,
                            None => ::miniextendr_api::error::r_stop(#error_msg),
                        }
                    }
                }
            }
            ReturnHandling::OptionIntoR => {
                let error_msg = format!("miniextendr function `{}` returned None", fn_ident);
                let result_ident = format_ident!("__result");
                let conversion = self.sexp_conversion_expr(&result_ident);
                if self.error_in_r {
                    quote! {
                        let __result = #call_expr;
                        let #result_ident = match __result {
                            Some(v) => v,
                            None => return ::miniextendr_api::error_value::make_rust_error_value(
                                #error_msg, "none_err"
                            ),
                        };
                        #conversion
                    }
                } else {
                    quote! {
                        let __result = #call_expr;
                        let #result_ident = match __result {
                            Some(v) => v,
                            None => ::miniextendr_api::error::r_stop(#error_msg),
                        };
                        #conversion
                    }
                }
            }
            ReturnHandling::ResultUnit => {
                if self.error_in_r {
                    quote! {
                        let __result = #call_expr;
                        if let Err(e) = __result {
                            return ::miniextendr_api::error_value::make_rust_error_value(
                                &format!("{:?}", e), "result_err"
                            );
                        }
                        unsafe { ::miniextendr_api::ffi::R_NilValue }
                    }
                } else {
                    quote! {
                        let __result = #call_expr;
                        if let Err(e) = __result {
                            ::miniextendr_api::error::r_stop(&format!("{:?}", e));
                        }
                        unsafe { ::miniextendr_api::ffi::R_NilValue }
                    }
                }
            }
            ReturnHandling::ResultSexp => {
                if self.error_in_r {
                    quote! {
                        let __result = #call_expr;
                        match __result {
                            Ok(v) => v,
                            Err(e) => return ::miniextendr_api::error_value::make_rust_error_value(
                                &format!("{:?}", e), "result_err"
                            ),
                        }
                    }
                } else {
                    quote! {
                        let __result = #call_expr;
                        match __result {
                            Ok(v) => v,
                            Err(e) => ::miniextendr_api::error::r_stop(&format!("{:?}", e)),
                        }
                    }
                }
            }
            ReturnHandling::ResultIntoR => {
                let result_ident = format_ident!("__result");
                let conversion = self.sexp_conversion_expr(&result_ident);
                if self.error_in_r {
                    quote! {
                        let __result = #call_expr;
                        let #result_ident = match __result {
                            Ok(v) => v,
                            Err(e) => return ::miniextendr_api::error_value::make_rust_error_value(
                                &format!("{:?}", e), "result_err"
                            ),
                        };
                        #conversion
                    }
                } else {
                    quote! {
                        let __result = #call_expr;
                        let #result_ident = match __result {
                            Ok(v) => v,
                            Err(e) => ::miniextendr_api::error::r_stop(&format!("{:?}", e)),
                        };
                        #conversion
                    }
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
                let unwind_fn = self.worker_conversion_unwind_fn();
                let convert = quote! {
                    #unwind_fn(
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
                let result_ident = format_ident!("__miniextendr_result");
                let conversion = self.sexp_conversion_expr(&result_ident);
                let unwind_fn = self.worker_conversion_unwind_fn();
                let convert = quote! {
                    #unwind_fn(
                        || #conversion,
                        None,
                    )
                };
                (worker, convert)
            }
            ReturnHandling::OptionUnit => {
                let error_msg = format!("miniextendr function `{}` returned None", fn_ident);
                if self.error_in_r {
                    // In error_in_r mode: return the Option from worker, check on main thread
                    let worker = quote! { #call_expr };
                    let convert = quote! {
                        if __miniextendr_result.is_none() {
                            ::miniextendr_api::error_value::make_rust_error_value(
                                #error_msg, "none_err"
                            )
                        } else {
                            unsafe { ::miniextendr_api::ffi::R_NilValue }
                        }
                    };
                    (worker, convert)
                } else {
                    let worker = quote! {
                        let __result = #call_expr;
                        if __result.is_none() {
                            ::miniextendr_api::error::r_stop(#error_msg);
                        }
                    };
                    let convert = quote! {
                        unsafe { ::miniextendr_api::ffi::R_NilValue }
                    };
                    (worker, convert)
                }
            }
            ReturnHandling::OptionSexp => {
                let error_msg = format!("miniextendr function `{}` returned None", fn_ident);
                if self.error_in_r {
                    let worker = quote! { #call_expr };
                    let convert = quote! {
                        match __miniextendr_result {
                            Some(v) => v,
                            None => ::miniextendr_api::error_value::make_rust_error_value(
                                #error_msg, "none_err"
                            ),
                        }
                    };
                    (worker, convert)
                } else {
                    let worker = quote! {
                        let __result = #call_expr;
                        match __result {
                            Some(v) => v,
                            None => ::miniextendr_api::error::r_stop(#error_msg),
                        }
                    };
                    let convert = quote! {
                        __miniextendr_result
                    };
                    (worker, convert)
                }
            }
            ReturnHandling::OptionIntoR => {
                let error_msg = format!("miniextendr function `{}` returned None", fn_ident);
                if self.error_in_r {
                    let worker = quote! { #call_expr };
                    let result_ident = format_ident!("__miniextendr_result");
                    let conversion = self.sexp_conversion_expr(&result_ident);
                    let unwind_fn = self.worker_conversion_unwind_fn();
                    let convert = quote! {
                        match __miniextendr_result {
                            Some(#result_ident) => #unwind_fn(
                                || #conversion,
                                None,
                            ),
                            None => ::miniextendr_api::error_value::make_rust_error_value(
                                #error_msg, "none_err"
                            ),
                        }
                    };
                    (worker, convert)
                } else {
                    let worker = quote! {
                        let __result = #call_expr;
                        match __result {
                            Some(v) => v,
                            None => ::miniextendr_api::error::r_stop(#error_msg),
                        }
                    };
                    let result_ident = format_ident!("__miniextendr_result");
                    let conversion = self.sexp_conversion_expr(&result_ident);
                    let convert = quote! {
                        ::miniextendr_api::unwind_protect::with_r_unwind_protect(
                            || #conversion,
                            None,
                        )
                    };
                    (worker, convert)
                }
            }
            ReturnHandling::ResultUnit => {
                if self.error_in_r {
                    let worker = quote! { #call_expr };
                    let convert = quote! {
                        match __miniextendr_result {
                            Ok(()) => unsafe { ::miniextendr_api::ffi::R_NilValue },
                            Err(e) => ::miniextendr_api::error_value::make_rust_error_value(
                                &format!("{:?}", e), "result_err"
                            ),
                        }
                    };
                    (worker, convert)
                } else {
                    let worker = quote! {
                        let __result = #call_expr;
                        if let Err(e) = __result {
                            ::miniextendr_api::error::r_stop(&format!("{:?}", e));
                        }
                    };
                    let convert = quote! {
                        unsafe { ::miniextendr_api::ffi::R_NilValue }
                    };
                    (worker, convert)
                }
            }
            ReturnHandling::ResultSexp => {
                if self.error_in_r {
                    let worker = quote! { #call_expr };
                    let convert = quote! {
                        match __miniextendr_result {
                            Ok(v) => v,
                            Err(e) => ::miniextendr_api::error_value::make_rust_error_value(
                                &format!("{:?}", e), "result_err"
                            ),
                        }
                    };
                    (worker, convert)
                } else {
                    let worker = quote! {
                        let __result = #call_expr;
                        match __result {
                            Ok(v) => v,
                            Err(e) => ::miniextendr_api::error::r_stop(&format!("{:?}", e)),
                        }
                    };
                    let convert = quote! {
                        __miniextendr_result
                    };
                    (worker, convert)
                }
            }
            ReturnHandling::ResultIntoR => {
                if self.error_in_r {
                    let worker = quote! { #call_expr };
                    let result_ident = format_ident!("__miniextendr_result");
                    let conversion = self.sexp_conversion_expr(&result_ident);
                    let unwind_fn = self.worker_conversion_unwind_fn();
                    let convert = quote! {
                        match __miniextendr_result {
                            Ok(#result_ident) => #unwind_fn(
                                || #conversion,
                                None,
                            ),
                            Err(e) => ::miniextendr_api::error_value::make_rust_error_value(
                                &format!("{:?}", e), "result_err"
                            ),
                        }
                    };
                    (worker, convert)
                } else {
                    let worker = quote! {
                        let __result = #call_expr;
                        match __result {
                            Ok(v) => v,
                            Err(e) => ::miniextendr_api::error::r_stop(&format!("{:?}", e)),
                        }
                    };
                    let result_ident = format_ident!("__miniextendr_result");
                    let conversion = self.sexp_conversion_expr(&result_ident);
                    let convert = quote! {
                        ::miniextendr_api::unwind_protect::with_r_unwind_protect(
                            || #conversion,
                            None,
                        )
                    };
                    (worker, convert)
                }
            }
        }
    }

    /// Returns the appropriate unwind protection function for worker-thread
    /// conversion steps. In error_in_r mode, uses the error_in_r variant that
    /// returns tagged error values on conversion panics instead of longjmping.
    fn worker_conversion_unwind_fn(&self) -> TokenStream {
        if self.error_in_r {
            quote! { ::miniextendr_api::unwind_protect::with_r_unwind_protect_error_in_r }
        } else {
            quote! { ::miniextendr_api::unwind_protect::with_r_unwind_protect }
        }
    }

    /// Returns the SEXP conversion expression for `result_ident`, using strict
    /// checked conversion if strict mode is on and the inner return type is lossy,
    /// otherwise falling back to `IntoR::into_sexp()`.
    fn sexp_conversion_expr(&self, result_ident: &syn::Ident) -> TokenStream {
        if self.strict {
            // Extract effective inner type from output
            let inner_ty = match &self.output {
                syn::ReturnType::Type(_, ty) => {
                    let ty = ty.as_ref();
                    // Check for Option<T> or Result<T, E> wrappers
                    if let syn::Type::Path(p) = ty
                        && let Some(seg) = p.path.segments.last()
                    {
                        let name = seg.ident.to_string();
                        if (name == "Option" || name == "Result")
                            && let Some(inner) = first_type_argument(seg)
                        {
                            Some(inner)
                        } else {
                            Some(ty)
                        }
                    } else {
                        Some(ty)
                    }
                }
                syn::ReturnType::Default => None,
            };

            if let Some(inner_ty) = inner_ty.and_then(|ty| {
                crate::return_type_analysis::strict_conversion_for_type(ty, result_ident)
            }) {
                return inner_ty;
            }
        }

        quote! { ::miniextendr_api::into_r::IntoR::into_sexp(#result_ident) }
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
        let source_loc_doc = crate::source_location_doc(self.fn_ident.span());

        quote! {
            #[doc = #doc]
            #[doc = #doc_example]
            #[doc = #source_loc_doc]
            #[doc = concat!("Generated from source file `", file!(), "`.")]
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
    /// Strict conversion mode.
    strict: bool,
    /// error_in_r mode.
    error_in_r: bool,
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

    /// Enable strict conversion mode for lossy return types.
    pub fn strict(mut self) -> Self {
        self.strict = true;
        self
    }

    /// Enable error_in_r mode: return tagged error values instead of r_stop.
    pub fn error_in_r(mut self) -> Self {
        self.error_in_r = true;
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
            strict: self.strict,
            error_in_r: self.error_in_r,
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

/// Detect return handling from a concrete return type.
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

/// Returns the first generic type argument from a path segment, if any.
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

#[cfg(test)]
mod tests;
