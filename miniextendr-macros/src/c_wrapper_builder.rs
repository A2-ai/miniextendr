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
    /// Execute on main R thread with `with_r_unwind_protect`. **Default.**
    ///
    /// All code runs on R's main thread. Combined with `error_in_r` (also default),
    /// errors are returned as tagged SEXP values and the R wrapper raises structured
    /// condition objects. Simpler execution model with better R integration.
    ///
    /// Also required when:
    /// - Function takes SEXP inputs (not Send)
    /// - Function returns raw SEXP
    /// - Instance method (self_ptr isn't Send)
    /// - Function uses variadic dots (Dots type isn't Send)
    /// - `#[miniextendr(check_interrupt)]` used
    MainThread,

    /// Execute on worker thread with panic catching. **Opt-in via `#[miniextendr(worker)]`.**
    ///
    /// Structure:
    /// 1. Argument conversion on main thread
    /// 2. Function execution on worker thread via `run_on_worker`
    /// 3. SEXP conversion on main thread with `with_r_unwind_protect`
    WorkerThread,
}

impl ThreadStrategy {}

/// Strategy for converting a Rust return value into an R `SEXP`.
///
/// Determined automatically by [`detect_return_handling`] from the function's return type,
/// or set explicitly via [`CWrapperContextBuilder::return_handling`]. Each variant
/// handles a different return type pattern, controlling how the C wrapper converts
/// the Rust value back to R and how errors/None values are surfaced.
#[derive(Debug, Clone)]
pub enum ReturnHandling {
    /// Returns unit type `()` -- emits `R_NilValue`.
    Unit,
    /// Returns raw `SEXP` -- passes the value through unchanged (no conversion).
    RawSexp,
    /// Returns `Self` -- wraps the value in an `ExternalPtr` via `ExternalPtr::new`.
    ExternalPtr,
    /// Returns an arbitrary type `T: IntoR` -- converts via `IntoR::into_sexp`.
    IntoR,
    /// Returns `Option<()>` -- raises an error on `None`, otherwise emits `R_NilValue`.
    OptionUnit,
    /// Returns `Option<SEXP>` -- raises an error on `None`, otherwise passes through.
    OptionSexp,
    /// Returns `Option<T>` -- raises an error on `None`, otherwise converts via `IntoR::into_sexp`.
    OptionIntoR,
    /// Returns `Result<(), E>` -- raises an error on `Err`, otherwise emits `R_NilValue`.
    ResultUnit,
    /// Returns `Result<SEXP, E>` -- raises an error on `Err`, otherwise passes through.
    ResultSexp,
    /// Returns `Result<T, E>` -- raises an error on `Err`, otherwise converts via `IntoR::into_sexp`.
    ResultIntoR,
}

/// All information needed to generate a C wrapper function for an R-exported Rust item.
///
/// This struct abstracts over the differences between standalone `#[miniextendr]` functions
/// and `impl` block methods (R6, S3, S4, S7, Env). It is constructed via
/// [`CWrapperContextBuilder`] and consumed by [`CWrapperContext::generate`], which emits
/// both the `extern "C-unwind"` wrapper and the corresponding `R_CallMethodDef` constant.
pub struct CWrapperContext {
    /// Identifier of the original Rust function or method being wrapped.
    pub fn_ident: syn::Ident,
    /// Identifier for the generated C wrapper (e.g., `C_foo` or `C_Type__method`).
    pub c_ident: syn::Ident,
    /// Identifier of the `R_WRAPPER_*` or `R_WRAPPERS_IMPL_*` const that holds the
    /// generated R wrapper code string. Used for rustdoc cross-references.
    pub r_wrapper_const: syn::Ident,
    /// Function parameters (excluding the `self` receiver for methods).
    /// Each parameter becomes a `SEXP` argument in the C wrapper signature.
    pub inputs: syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
    /// The original Rust return type. Used by strict-mode to inspect whether the inner
    /// type is lossy (e.g., `i64`, `u64`) and needs checked conversion.
    pub output: syn::ReturnType,
    /// Statements emitted before the call expression. For instance methods, this
    /// includes extracting `self` from the `ExternalPtr` SEXP.
    pub pre_call: Vec<TokenStream>,
    /// The actual Rust call expression (e.g., `my_func(arg0, arg1)` or
    /// `self_ref.method(arg0)`). Inserted into the wrapper body after conversions.
    pub call_expr: TokenStream,
    /// Whether to run on the main R thread or dispatch to the worker thread.
    pub thread_strategy: ThreadStrategy,
    /// How to convert the Rust return value into a `SEXP` for R.
    pub return_handling: ReturnHandling,
    /// When `true`, all parameters use coercing conversion (`Rf_coerceVector`) instead
    /// of strict type-matching. Set by `#[miniextendr(coerce)]`.
    pub coerce_all: bool,
    /// Names of individual parameters that use coercing conversion.
    /// Set by `#[miniextendr(coerce = "param_name")]`.
    pub coerce_params: Vec<String>,
    /// When `true`, emits `R_CheckUserInterrupt()` before the call expression.
    /// Set by `#[miniextendr(check_interrupt)]`.
    pub check_interrupt: bool,
    /// When `true`, wraps the call in `GetRNGstate()`/`PutRNGstate()` for R's
    /// random number generator state management. Set by `#[miniextendr(rng)]`.
    pub rng: bool,
    /// `#[cfg(...)]` attributes from the original item, propagated to the C wrapper
    /// and `call_method_def` constant so they are conditionally compiled.
    pub cfg_attrs: Vec<syn::Attribute>,
    /// For methods: the type identifier (e.g., `MyStruct`). Used in doc comments
    /// and default `call_method_def` naming. `None` for standalone functions.
    pub type_context: Option<syn::Ident>,
    /// Whether the original method has a `self` receiver. When `true`, the C wrapper
    /// includes a `self_sexp` parameter before the regular arguments.
    pub has_self: bool,
    /// Override for the `call_method_def` constant name. If `None`, defaults to
    /// `call_method_def_{type}_{method}` (methods) or `call_method_def_{fn}` (standalone).
    pub call_method_def_ident: Option<syn::Ident>,
    /// When `true`, uses `checked_into_sexp_*` for lossy return types (`i64`, `u64`,
    /// `isize`, `usize` and their `Vec` variants) instead of regular `IntoR::into_sexp`.
    /// Set by `#[miniextendr(strict)]`.
    pub strict: bool,
    /// When `true`, Rust panics and errors return tagged `SEXP` error values
    /// (via `make_rust_error_value`) instead of raising an R error directly.
    /// The R wrapper then raises a structured condition object.
    /// This is the default for standalone functions.
    pub error_in_r: bool,
}

impl CWrapperContext {
    /// Creates a new [`CWrapperContextBuilder`] with the given function and C wrapper identifiers.
    ///
    /// All other fields start at their defaults (empty/false/None). Use the builder methods
    /// to configure the context, then call [`CWrapperContextBuilder::build`] to finalize.
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

    /// Generates the complete output for this wrapper: an `extern "C-unwind"` function
    /// and an `R_CallMethodDef` constant, both decorated with `#[cfg(...)]` attributes
    /// if present.
    ///
    /// Dispatches to [`generate_main_thread_wrapper`](Self::generate_main_thread_wrapper) or
    /// [`generate_worker_thread_wrapper`](Self::generate_worker_thread_wrapper) based on
    /// [`thread_strategy`](Self::thread_strategy).
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

    /// Builds the C wrapper's parameter list from the Rust function signature.
    ///
    /// Returns a tuple of:
    /// - `c_params`: `SEXP` parameter declarations for the C wrapper signature. Always
    ///   starts with `__miniextendr_call` (the R call object for error context), followed
    ///   by `self_sexp` for instance methods, then `arg_0`, `arg_1`, ... for each input.
    /// - `rust_args`: The original Rust parameter identifiers (used in the call expression).
    /// - `sexp_idents`: The generated `arg_N` identifiers (used in SEXP-to-Rust conversions).
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

    /// Generates `TryFromSexp` conversion statements for each parameter.
    ///
    /// Each statement converts an `arg_N: SEXP` into the corresponding Rust type
    /// declared in the original function signature. Respects `strict` and `coerce` settings.
    ///
    /// Used by the main-thread wrapper where all conversions happen inline.
    fn build_conversion_stmts(&self, sexp_idents: &[syn::Ident]) -> Vec<TokenStream> {
        let mut builder = crate::RustConversionBuilder::new();
        if self.strict {
            builder = builder.with_strict();
        }
        if self.error_in_r {
            builder = builder.with_error_in_r();
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
        if self.error_in_r {
            builder = builder.with_error_in_r();
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

    /// Generates an `extern "C-unwind"` wrapper that runs entirely on the R main thread.
    ///
    /// The wrapper body is enclosed in `with_r_unwind_protect` (or its `_error_in_r` variant),
    /// which catches both Rust panics and R longjmps. When `rng` is enabled, the call is
    /// additionally wrapped in `catch_unwind` so that `PutRNGstate()` runs even on panic.
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
            // RNG variant: wrap in catch_unwind so we can call PutRNGstate before error handling.
            // rng always implies error_in_r (validated at parse time), so we always return
            // a tagged error value on panic instead of resume_unwind.
            let rng_panic_handler = quote! {
                ::miniextendr_api::error_value::make_rust_error_value(
                    &::miniextendr_api::unwind_protect::panic_payload_to_string(&*payload),
                    "panic",
                    Some(__miniextendr_call),
                )
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

    /// Generates an `extern "C-unwind"` wrapper that dispatches to the worker thread.
    ///
    /// Structure:
    /// 1. `GetRNGstate()` (if `rng` enabled)
    /// 2. `catch_unwind` around the entire body
    /// 3. Pre-closure conversions on the main thread (produces owned values)
    /// 4. `run_on_worker` (returns `Result<T, String>`) with a
    ///    `move` closure containing in-closure conversions and the call expression
    /// 5. Return conversion back on the main thread via `with_r_unwind_protect`
    /// 6. `PutRNGstate()` (if `rng` enabled)
    /// 7. Panic handling: either tagged error value or `Rf_errorcall`
    fn generate_worker_thread_wrapper(&self) -> TokenStream {
        let c_ident = &self.c_ident;
        let (c_params, _, sexp_idents) = self.build_c_params();
        let (pre_closure_stmts, in_closure_stmts) = self.build_conversion_stmts_split(&sexp_idents);
        let pre_call = &self.pre_call;
        let call_expr = &self.call_expr;

        // Compile-time check: worker dispatch requires the `worker-thread` feature.
        // Check both `worker-thread` (direct) and `default-worker` (implies worker-thread
        // via miniextendr-api, but the user crate may only have the latter in its features).
        let fn_name = self.fn_ident.to_string();
        let feature_msg = format!(
            "`#[miniextendr(worker)]` on `{fn_name}` requires the `worker-thread` cargo feature. \
             Add `worker-thread = [\"miniextendr-api/worker-thread\"]` to your [features] in Cargo.toml."
        );
        let worker_feature_check = quote! {
            #[cfg(not(any(feature = "worker-thread", feature = "default-worker")))]
            compile_error!(#feature_msg);
        };

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
                    &::miniextendr_api::unwind_protect::panic_payload_to_string(&*payload),
                    "panic",
                    Some(__miniextendr_call),
                )
            }
        } else {
            quote! {
                ::miniextendr_api::worker::panic_message_to_r_error(
                    ::miniextendr_api::unwind_protect::panic_payload_to_string(&*payload),
                    Some(__miniextendr_call),
                )
            }
        };

        if self.error_in_r {
            // error_in_r: run_on_worker returns Result; Err → tagged error value
            quote! {
                #worker_feature_check

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

                        match ::miniextendr_api::worker::run_on_worker(move || {
                            #(#in_closure_stmts)*
                            #worker_body
                        }) {
                            Ok(__miniextendr_result) => {
                                #return_conversion
                            }
                            Err(__panic_msg) => {
                                ::miniextendr_api::error_value::make_rust_error_value(
                                    &__panic_msg, "panic", Some(__miniextendr_call),
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
            // run_on_worker returns Result; Err → R error via Rf_errorcall
            quote! {
                #worker_feature_check

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

                        match ::miniextendr_api::worker::run_on_worker(move || {
                            #(#in_closure_stmts)*
                            #worker_body
                        }) {
                            Ok(__miniextendr_result) => {
                                #return_conversion
                            }
                            Err(__panic_msg) => {
                                ::miniextendr_api::worker::panic_message_to_r_error(__panic_msg, Some(__miniextendr_call))
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
        }
    }

    /// Generates the inline return-handling code for the main-thread wrapper.
    ///
    /// Emits the call expression followed by conversion logic based on [`ReturnHandling`].
    /// For `Option`/`Result` variants, also emits error-path code: either a tagged error
    /// value (`error_in_r`) or an R error.
    fn generate_return_handling(&self, call_expr: &TokenStream) -> TokenStream {
        let fn_ident = &self.fn_ident;

        match &self.return_handling {
            ReturnHandling::Unit => {
                quote! {
                    #call_expr;
                    ::miniextendr_api::ffi::SEXP::nil()
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
                                #error_msg, "none_err", Some(__miniextendr_call),
                            );
                        }
                        ::miniextendr_api::ffi::SEXP::nil()
                    }
                } else {
                    quote! {
                        let __result = #call_expr;
                        if __result.is_none() {
                            ::miniextendr_api::error::r_stop(#error_msg);
                        }
                        ::miniextendr_api::ffi::SEXP::nil()
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
                                #error_msg, "none_err", Some(__miniextendr_call),
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
                                #error_msg, "none_err", Some(__miniextendr_call),
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
                                &format!("{:?}", e), "result_err", Some(__miniextendr_call),
                            );
                        }
                        ::miniextendr_api::ffi::SEXP::nil()
                    }
                } else {
                    quote! {
                        let __result = #call_expr;
                        if let Err(e) = __result {
                            ::miniextendr_api::error::r_stop(&format!("{:?}", e));
                        }
                        ::miniextendr_api::ffi::SEXP::nil()
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
                                &format!("{:?}", e), "result_err", Some(__miniextendr_call),
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
                                &format!("{:?}", e), "result_err", Some(__miniextendr_call),
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

    /// Generates return-handling code split between worker and main threads.
    ///
    /// Returns `(worker_body, return_conversion)`:
    /// - `worker_body`: Runs inside the `run_on_worker` closure. Contains the call
    ///   expression and, for non-`error_in_r` mode, error checking (`Option::None` /
    ///   `Result::Err` handling).
    /// - `return_conversion`: Runs back on the main thread after the worker returns.
    ///   Converts the Rust value to SEXP (via `with_r_unwind_protect`). In `error_in_r`
    ///   mode, error checking also happens here since the worker returns the raw
    ///   `Option`/`Result` for the main thread to inspect.
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
                    ::miniextendr_api::ffi::SEXP::nil()
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
                                #error_msg, "none_err", Some(__miniextendr_call),
                            )
                        } else {
                            ::miniextendr_api::ffi::SEXP::nil()
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
                        ::miniextendr_api::ffi::SEXP::nil()
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
                                #error_msg, "none_err", Some(__miniextendr_call),
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
                                #error_msg, "none_err", Some(__miniextendr_call),
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
                            Ok(()) => ::miniextendr_api::ffi::SEXP::nil(),
                            Err(e) => ::miniextendr_api::error_value::make_rust_error_value(
                                &format!("{:?}", e), "result_err", Some(__miniextendr_call),
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
                        ::miniextendr_api::ffi::SEXP::nil()
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
                                &format!("{:?}", e), "result_err", Some(__miniextendr_call),
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
                                &format!("{:?}", e), "result_err", Some(__miniextendr_call),
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

    /// Generates the `R_CallMethodDef` constant for R's `.Call` interface registration.
    ///
    /// The constant contains the C symbol name, a `DL_FUNC` pointer to the wrapper
    /// (obtained via `transmute`), and the argument count. R uses this at package load
    /// time (via `R_registerRoutines`) to register the native routine.
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
            #[::miniextendr_api::linkme::distributed_slice(::miniextendr_api::registry::MX_CALL_DEFS)]
                #[linkme(crate = ::miniextendr_api::linkme)]
            #[allow(non_upper_case_globals)]
            #[allow(non_snake_case)]
            static #call_method_def_ident: ::miniextendr_api::ffi::R_CallMethodDef = unsafe {
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

    /// Generates a rustdoc comment string for the C wrapper function.
    ///
    /// Includes the original function/method name, thread strategy label, and a
    /// cross-reference to the R wrapper constant.
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

/// Builder for [`CWrapperContext`].
///
/// Created via [`CWrapperContext::builder`]. All fields except `fn_ident` and `c_ident`
/// (provided at construction) default to empty/false/None. Required fields (`call_expr`,
/// `r_wrapper_const`) must be set before calling [`build`](Self::build) or it will panic.
///
/// Optional fields like `thread_strategy` and `return_handling` are auto-detected from
/// the function signature if not explicitly set.
pub struct CWrapperContextBuilder {
    /// Rust function/method identifier (set at construction).
    fn_ident: syn::Ident,
    /// C wrapper function identifier (set at construction).
    c_ident: syn::Ident,
    /// R wrapper constant identifier for doc cross-references. **Required.**
    r_wrapper_const: Option<syn::Ident>,
    /// Function parameters (excluding `self`). Defaults to empty.
    inputs: syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
    /// Rust return type. Defaults to `()` (no return type annotation).
    output: syn::ReturnType,
    /// Pre-call statements emitted before the call expression. Defaults to empty.
    pre_call: Vec<TokenStream>,
    /// The Rust call expression. **Required.**
    call_expr: Option<TokenStream>,
    /// Thread strategy override. If `None`, defaults to [`ThreadStrategy::MainThread`].
    thread_strategy: Option<ThreadStrategy>,
    /// Return handling override. If `None`, auto-detected from `output` via [`detect_return_handling`].
    return_handling: Option<ReturnHandling>,
    /// Enable coercing conversion for all parameters.
    coerce_all: bool,
    /// Names of individual parameters with coercing conversion enabled.
    coerce_params: Vec<String>,
    /// Emit `R_CheckUserInterrupt()` before the call.
    check_interrupt: bool,
    /// Wrap call in `GetRNGstate()`/`PutRNGstate()`.
    rng: bool,
    /// `#[cfg(...)]` attributes to propagate to generated items.
    cfg_attrs: Vec<syn::Attribute>,
    /// Type identifier for method context (e.g., `MyStruct`). `None` for standalone functions.
    type_context: Option<syn::Ident>,
    /// Whether the original method has a `self` receiver.
    has_self: bool,
    /// Custom `call_method_def` constant name override.
    call_method_def_ident: Option<syn::Ident>,
    /// Enable strict checked conversions for lossy return types.
    strict: bool,
    /// Enable error_in_r mode (tagged error values instead of raising R errors).
    error_in_r: bool,
}

impl CWrapperContextBuilder {
    /// Sets the R wrapper constant identifier (e.g., `R_WRAPPER_my_func`).
    /// **Required** -- [`build`](Self::build) panics if not set.
    pub fn r_wrapper_const(mut self, ident: syn::Ident) -> Self {
        self.r_wrapper_const = Some(ident);
        self
    }

    /// Sets the function parameters (excluding `self` receiver).
    /// Each input becomes a `SEXP` argument in the C wrapper.
    pub fn inputs(
        mut self,
        inputs: syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
    ) -> Self {
        self.inputs = inputs;
        self
    }

    /// Sets the Rust return type. Used for auto-detecting [`ReturnHandling`]
    /// and for strict-mode type inspection.
    pub fn output(mut self, output: syn::ReturnType) -> Self {
        self.output = output;
        self
    }

    /// Sets pre-call statements emitted before the call expression.
    /// Typically used for self-extraction in instance methods.
    pub fn pre_call(mut self, stmts: Vec<TokenStream>) -> Self {
        self.pre_call = stmts;
        self
    }

    /// Sets the Rust call expression (e.g., `my_func(arg0)` or `self_ref.method(arg0)`).
    /// **Required** -- [`build`](Self::build) panics if not set.
    pub fn call_expr(mut self, expr: TokenStream) -> Self {
        self.call_expr = Some(expr);
        self
    }

    /// Overrides the thread strategy. If not called, defaults to [`ThreadStrategy::MainThread`].
    pub fn thread_strategy(mut self, strategy: ThreadStrategy) -> Self {
        self.thread_strategy = Some(strategy);
        self
    }

    /// Overrides the return handling strategy. If not called, auto-detected from `output`
    /// via [`detect_return_handling`].
    pub fn return_handling(mut self, handling: ReturnHandling) -> Self {
        self.return_handling = Some(handling);
        self
    }

    /// Enables coercing conversion for all parameters via `Rf_coerceVector`.
    pub fn coerce_all(mut self) -> Self {
        self.coerce_all = true;
        self
    }

    /// Enables `R_CheckUserInterrupt()` before the call expression.
    pub fn check_interrupt(mut self) -> Self {
        self.check_interrupt = true;
        self
    }

    /// Enable RNG state management (GetRNGstate/PutRNGstate).
    pub fn rng(mut self) -> Self {
        self.rng = true;
        self
    }

    /// Sets `#[cfg(...)]` attributes to propagate to the C wrapper and `call_method_def`.
    pub fn cfg_attrs(mut self, attrs: Vec<syn::Attribute>) -> Self {
        self.cfg_attrs = attrs;
        self
    }

    /// Sets the type context for methods (e.g., `MyStruct`). Used in doc comments
    /// and default `call_method_def` naming.
    pub fn type_context(mut self, type_ident: syn::Ident) -> Self {
        self.type_context = Some(type_ident);
        self
    }

    /// Marks this as an instance method with a `self` receiver.
    /// Causes the C wrapper to include a `self_sexp` parameter.
    pub fn has_self(mut self) -> Self {
        self.has_self = true;
        self
    }

    /// Enables strict checked conversions for lossy return types (`i64`, `u64`, `isize`,
    /// `usize` and their `Vec` variants).
    pub fn strict(mut self) -> Self {
        self.strict = true;
        self
    }

    /// Enables error_in_r mode: panics and errors return tagged `SEXP` values
    /// (via `make_rust_error_value`) instead of raising an R error directly.
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

    /// Consumes the builder and returns a fully configured [`CWrapperContext`].
    ///
    /// If `thread_strategy` was not set, defaults to [`ThreadStrategy::MainThread`].
    /// If `return_handling` was not set, auto-detects from the `output` type via
    /// [`detect_return_handling`].
    ///
    /// # Panics
    ///
    /// Panics if `call_expr` or `r_wrapper_const` was not set.
    pub fn build(self) -> CWrapperContext {
        let call_expr = self
            .call_expr
            .expect("call_expr is required for CWrapperContext");
        let r_wrapper_const = self
            .r_wrapper_const
            .expect("r_wrapper_const is required for CWrapperContext");

        // Detect thread strategy if not explicitly set
        // Main thread is the default for all methods (safer, error_in_r compatible)
        let thread_strategy = self.thread_strategy.unwrap_or(ThreadStrategy::MainThread);

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

/// Detects the appropriate [`ReturnHandling`] strategy from a function's return type.
///
/// Inspects the `syn::ReturnType`:
/// - No return type annotation (`Default`) maps to [`ReturnHandling::Unit`].
/// - An explicit type is analyzed by [`detect_return_handling_from_type`].
pub fn detect_return_handling(output: &syn::ReturnType) -> ReturnHandling {
    match output {
        syn::ReturnType::Default => ReturnHandling::Unit,
        syn::ReturnType::Type(_, ty) => detect_return_handling_from_type(ty),
    }
}

/// Determines the [`ReturnHandling`] variant for a concrete `syn::Type`.
///
/// Recognition rules:
/// - `()` -> [`Unit`](ReturnHandling::Unit)
/// - `Self` -> [`ExternalPtr`](ReturnHandling::ExternalPtr)
/// - `SEXP` -> [`RawSexp`](ReturnHandling::RawSexp)
/// - `Option<T>` -> recurses into `T` for `OptionUnit`, `OptionSexp`, or `OptionIntoR`
/// - `Result<T, E>` -> recurses into `T` for `ResultUnit`, `ResultSexp`, or `ResultIntoR`
/// - Anything else -> [`IntoR`](ReturnHandling::IntoR)
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

/// Extracts the first generic type argument from a path segment's angle-bracketed arguments.
///
/// For example, given `Option<String>`, returns `Some(&String)`.
/// Returns `None` if the segment has no angle-bracketed arguments or no type arguments.
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
