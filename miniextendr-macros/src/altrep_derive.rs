//! Derive macros for ALTREP data traits.
//!
//! These macros auto-implement `AltrepLen` and `Alt*Data` traits for simple field-based
//! ALTREP types, reducing boilerplate for users.

use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;

/// Per-family configuration controlling low-level code generation for an ALTREP type family.
///
/// Each ALTREP family (integer, real, logical, raw, string, complex, list) has a distinct
/// set of runtime macros for trait implementations and different capabilities (e.g., only
/// some families support `dataptr` or `subset`). This struct captures those differences
/// so [`AltrepAttrs::generate_lowlevel`] can emit the correct code.
struct AltrepFamilyConfig<'a> {
    /// Name of the `impl_alt*_from_data!` runtime macro used on the simple (non-expanded) path.
    /// For example, `"impl_altinteger_from_data"` for the integer family.
    macro_base: &'a str,
    /// If this family supports a typed `dataptr` materialization macro, the tuple contains:
    /// - The macro name (e.g., `"__impl_altvec_dataptr"`)
    /// - An optional element type token stream (e.g., `i32` for integer, `f64` for real)
    ///
    /// `None` means this family does not have a typed dataptr macro (e.g., list).
    dataptr_macro: Option<(&'a str, Option<TokenStream>)>,
    /// Whether this family supports the string-specific dataptr materialization macro
    /// (`__impl_altvec_string_dataptr`). Only `true` for the String family.
    string_dataptr: bool,
    /// Whether this family supports the `subset` option for `Extract_subset` method
    /// registration. `false` for List (which rejects `subset` and `dataptr`).
    subset: bool,
    /// Name of the internal macro for type-specific method implementations
    /// (e.g., `"__impl_altinteger_methods"` for integer).
    methods_macro: &'a str,
    /// Name of the `impl_inferbase_*!` macro that provides `InferBase` for this family
    /// (e.g., `"impl_inferbase_integer"`).
    inferbase_macro: &'a str,
    /// Default guard mode for this family when no explicit guard is specified.
    /// String family uses `RUnwind` because elt/dataptr call R APIs (Rf_mkCharLenCE).
    /// All families now default to `RUnwind`.
    default_guard: &'a str,
}

/// Parsed `#[altrep(...)]` attributes controlling ALTREP derive code generation.
///
/// These attributes are placed on the struct and parsed by all ALTREP derive macros
/// (`AltrepInteger`, `AltrepReal`, etc.) to customize the generated trait implementations.
///
/// # Supported `#[altrep(...)]` keys
///
/// | Key | Type | Description |
/// |-----|------|-------------|
/// | `len = "field"` | `String` | Name of the struct field that holds the vector length. Auto-detected if a field is named `len` or `length`. |
/// | `elt = "field"` | `String` | Name of the struct field to return as the element value (produces a constant-value vector). If omitted, the default `elt()` returns `NA` / `NaN` / `0` / `None` depending on the family. |
/// | `no_lowlevel` | Flag | Suppress automatic `impl_alt*_from_data!` macro invocation. Use this when you want to provide your own `Altrep`, `AltVec`, and family-specific trait implementations. |
/// | `dataptr` | Flag | Enable `Dataptr` method registration, allowing R to get a direct pointer to the underlying data. Mutually exclusive with `subset`. Not supported for List. |
/// | `serialize` | Flag | Enable `Serialized_state` and `Unserialize` method registration for ALTREP serialization support. |
/// | `subset` | Flag | Enable `Extract_subset` method registration. Mutually exclusive with `dataptr`. Only supported for integer and complex families. Not supported for List. |
/// | `unsafe` | Flag | Set guard mode to `Unsafe` -- no panic protection on ALTREP callbacks. |
/// | `rust_unwind` | Flag | Set guard mode to `RustUnwind` -- uses `catch_unwind` only (unsafe if callbacks call R APIs). |
/// | `r_unwind` | Flag | Set guard mode to `RUnwind` (default) -- uses `with_r_unwind_protect` for safe R API calls. |
struct AltrepAttrs {
    /// Field name containing the vector length, set via `#[altrep(len = "field")]`.
    /// If `None`, auto-detection looks for fields named `len` or `length`.
    len_field: Option<syn::Ident>,
    /// Field name for constant-value element access, set via `#[altrep(elt = "field")]`.
    /// When set, `elt()` returns `self.{field}` for every index.
    elt_field: Option<syn::Ident>,
    /// Field name for delegated element access, set via `#[altrep(elt_delegate = "field")]`.
    /// When set, `elt()` calls `self.{field}.elt(i)`, delegating to the inner type's
    /// `AltIntegerData`/`AltRealData`/etc. implementation. Useful for wrapper types
    /// around `StreamingIntData`, `StreamingRealData`, etc.
    elt_delegate: Option<syn::Ident>,
    /// Whether to generate the `impl_alt*_from_data!` macro call. Defaults to `true`.
    /// Set to `false` by `#[altrep(no_lowlevel)]`.
    generate_lowlevel: bool,
    /// Collected option flags (`dataptr`, `serialize`, `subset`) passed to the runtime macro.
    lowlevel_options: Vec<syn::Ident>,
    /// Guard mode override for ALTREP trampoline callbacks. Maps to [`AltrepGuard`] variants:
    /// - `Unsafe` -- no protection
    /// - `RustUnwind` -- `catch_unwind` only
    /// - `RUnwind` -- `with_r_unwind_protect` (default)
    guard: Option<syn::Ident>,
}

impl AltrepAttrs {
    /// Parses all `#[altrep(...)]` attributes from a derive input struct.
    ///
    /// Multiple `#[altrep(...)]` attributes are supported and their contents are merged.
    /// Unknown keys are silently ignored.
    ///
    /// # Errors
    ///
    /// Returns `Err` if an `#[altrep(...)]` attribute has malformed syntax.
    fn parse(input: &syn::DeriveInput) -> syn::Result<Self> {
        let mut len_field = None;
        let mut elt_field = None;
        let mut elt_delegate = None;
        let mut generate_lowlevel = true; // Default: generate
        let mut lowlevel_options = Vec::new();
        let mut guard = None;

        for attr in &input.attrs {
            if !attr.path().is_ident("altrep") {
                continue;
            }

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("len") {
                    let _: syn::Token![=] = meta.input.parse()?;
                    let field: syn::LitStr = meta.input.parse()?;
                    len_field = Some(syn::Ident::new(&field.value(), field.span()));
                } else if meta.path.is_ident("elt") {
                    let _: syn::Token![=] = meta.input.parse()?;
                    let field: syn::LitStr = meta.input.parse()?;
                    elt_field = Some(syn::Ident::new(&field.value(), field.span()));
                } else if meta.path.is_ident("elt_delegate") {
                    let _: syn::Token![=] = meta.input.parse()?;
                    let field: syn::LitStr = meta.input.parse()?;
                    elt_delegate = Some(syn::Ident::new(&field.value(), field.span()));
                } else if meta.path.is_ident("no_lowlevel") {
                    generate_lowlevel = false;
                } else if meta.path.is_ident("dataptr") {
                    lowlevel_options.push(syn::Ident::new("dataptr", meta.path.span()));
                } else if meta.path.is_ident("serialize") {
                    lowlevel_options.push(syn::Ident::new("serialize", meta.path.span()));
                } else if meta.path.is_ident("subset") {
                    lowlevel_options.push(syn::Ident::new("subset", meta.path.span()));
                } else if meta.path.is_ident("r#unsafe") || meta.path.is_ident("unsafe") {
                    guard = Some(syn::Ident::new("Unsafe", meta.path.span()));
                } else if meta.path.is_ident("rust_unwind") {
                    guard = Some(syn::Ident::new("RustUnwind", meta.path.span()));
                } else if meta.path.is_ident("r_unwind") {
                    guard = Some(syn::Ident::new("RUnwind", meta.path.span()));
                }
                Ok(())
            })?;
        }

        Ok(Self {
            len_field,
            elt_field,
            elt_delegate,
            generate_lowlevel,
            lowlevel_options,
            guard,
        })
    }

    /// Returns the length field identifier, either from the explicit `len = "..."` attribute
    /// or by auto-detecting a field named `len` or `length` on the struct.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the input is not a struct, or if no length field was specified
    /// and auto-detection fails.
    fn get_len_field(&self, input: &syn::DeriveInput) -> syn::Result<syn::Ident> {
        if let Some(ref field) = self.len_field {
            return Ok(field.clone());
        }

        // Try to auto-detect: look for field named "len" or "length"
        let fields = match &input.data {
            syn::Data::Struct(data_struct) => &data_struct.fields,
            _ => {
                return Err(syn::Error::new(
                    input.span(),
                    "Altrep derive only supports structs",
                ));
            }
        };

        for field in fields {
            if let Some(ident) = &field.ident
                && (ident == "len" || ident == "length")
            {
                return Ok(ident.clone());
            }
        }

        Err(syn::Error::new(
            input.span(),
            "no length field found; specify with #[altrep(len = \"field_name\")]",
        ))
    }

    /// Returns `true` if a non-default guard mode is set (i.e., `Unsafe` or `RUnwind`).
    ///
    /// The default guard is `RUnwind`, which uses the simple `impl_alt*_from_data!`
    /// macro path. Non-default guards (e.g., `RustUnwind`, `Unsafe`) require the
    /// expanded code generation path that emits individual internal macros with an
    /// explicit guard parameter.
    fn has_non_default_guard(&self) -> bool {
        match &self.guard {
            Some(g) => g != "RUnwind",
            None => false,
        }
    }

    /// Validates that the requested `#[altrep(...)]` option flags are compatible with
    /// the given ALTREP type family.
    ///
    /// Enforces two rules:
    /// 1. `subset` is only valid for families where `supports_subset` is `true`.
    /// 2. `dataptr` and `subset` are mutually exclusive.
    ///
    /// # Arguments
    ///
    /// * `family` -- A human-readable family name used in error messages (e.g., `"AltrepList"`).
    /// * `supports_subset` -- Whether this family supports the `Extract_subset` method.
    ///
    /// # Errors
    ///
    /// Returns `Err` with a span pointing to the offending option identifier.
    fn validate_options(&self, family: &str, supports_subset: bool) -> syn::Result<()> {
        let has_dataptr = self.lowlevel_options.iter().any(|o| o == "dataptr");
        let has_subset = self.lowlevel_options.iter().any(|o| o == "subset");

        if has_subset && !supports_subset {
            return Err(syn::Error::new(
                self.lowlevel_options
                    .iter()
                    .find(|o| *o == "subset")
                    .unwrap()
                    .span(),
                format!("`subset` is not supported for {family}"),
            ));
        }

        if has_dataptr && has_subset {
            return Err(syn::Error::new(
                self.lowlevel_options
                    .iter()
                    .find(|o| *o == "subset")
                    .unwrap()
                    .span(),
                "`dataptr` and `subset` are mutually exclusive",
            ));
        }

        Ok(())
    }

    /// Generates low-level ALTREP trait implementation code for a given type family.
    ///
    /// There are two code generation paths:
    ///
    /// 1. **Simple path** -- When using the default `RUnwind` guard and no `subset` option,
    ///    delegates to the `impl_alt*_from_data!` runtime macro which bundles `Altrep`,
    ///    `AltVec`, family-specific methods, and `InferBase` in a single expansion.
    ///
    /// 2. **Expanded path** -- When a non-default guard mode or `subset` is requested,
    ///    emits individual internal macros (`__impl_altrep_base!`, `__impl_altvec_*!`,
    ///    `__impl_alt*_methods!`, `impl_inferbase_*!`) with explicit guard parameters.
    ///
    /// # Arguments
    ///
    /// * `name` -- The struct identifier.
    /// * `family` -- The family-specific configuration controlling which macros to emit.
    ///
    /// # Returns
    ///
    /// A token stream containing the macro invocations, or an empty stream if
    /// `no_lowlevel` was specified.
    ///
    /// # Errors
    ///
    /// Returns `Err` if option validation fails (e.g., `subset` on an unsupported family).
    fn generate_lowlevel(
        &self,
        name: &syn::Ident,
        family: &AltrepFamilyConfig,
    ) -> syn::Result<TokenStream> {
        let AltrepFamilyConfig {
            macro_base,
            ref dataptr_macro,
            string_dataptr,
            subset,
            methods_macro,
            inferbase_macro,
            default_guard,
        } = *family;
        let altvec_dataptr_macro = dataptr_macro;
        let altvec_string_dataptr = string_dataptr;
        let altvec_subset = subset;
        if !self.generate_lowlevel {
            return Ok(quote! {});
        }

        self.validate_options(macro_base, altvec_subset)?;

        let has_serialize = self.lowlevel_options.iter().any(|o| o == "serialize");
        let has_dataptr = self.lowlevel_options.iter().any(|o| o == "dataptr");
        let has_subset = self.lowlevel_options.iter().any(|o| o == "subset");

        // Use the expanded path (individual internal macros) when:
        // - Non-default guard mode is set, OR
        // - `subset` is requested (the runtime from_data macros only have subset
        //   variants for integer and complex; other families expand manually)
        let needs_expanded_path = self.has_non_default_guard() || has_subset;

        if !needs_expanded_path {
            // Simple path: delegate to the impl_alt*_from_data! runtime macro
            let macro_ident = syn::Ident::new(macro_base, proc_macro2::Span::call_site());
            if self.lowlevel_options.is_empty() {
                return Ok(quote! {
                    ::miniextendr_api::#macro_ident!(#name);
                });
            } else {
                let options = &self.lowlevel_options;
                return Ok(quote! {
                    ::miniextendr_api::#macro_ident!(#name, #(#options),*);
                });
            }
        }

        // Expanded path: emit individual internal macros
        let guard = self
            .guard
            .as_ref()
            .cloned()
            .unwrap_or_else(|| syn::Ident::new(default_guard, proc_macro2::Span::call_site()));

        // 1. Altrep base (with or without serialize)
        let base_impl = if has_serialize {
            quote! { ::miniextendr_api::__impl_altrep_base_with_serialize!(#name, #guard); }
        } else {
            quote! { ::miniextendr_api::__impl_altrep_base!(#name, #guard); }
        };

        // 2. AltVec impl
        let vec_impl = if has_dataptr {
            if let Some((macro_name, elem_ty)) = altvec_dataptr_macro {
                let dp_macro = syn::Ident::new(macro_name, proc_macro2::Span::call_site());
                if let Some(elem) = elem_ty {
                    quote! { ::miniextendr_api::#dp_macro!(#name, #elem); }
                } else {
                    quote! { ::miniextendr_api::#dp_macro!(#name); }
                }
            } else if altvec_string_dataptr {
                quote! { ::miniextendr_api::__impl_altvec_string_dataptr!(#name); }
            } else {
                quote! { impl ::miniextendr_api::altrep_traits::AltVec for #name {} }
            }
        } else if has_subset && altvec_subset {
            quote! { ::miniextendr_api::__impl_altvec_extract_subset!(#name); }
        } else {
            quote! { impl ::miniextendr_api::altrep_traits::AltVec for #name {} }
        };

        // 3. Type-specific methods
        let methods_ident = syn::Ident::new(methods_macro, proc_macro2::Span::call_site());
        let methods_impl = quote! { ::miniextendr_api::#methods_ident!(#name); };

        // 4. InferBase
        let inferbase_ident = syn::Ident::new(inferbase_macro, proc_macro2::Span::call_site());
        let inferbase_impl = quote! { ::miniextendr_api::#inferbase_ident!(#name); };

        Ok(quote! {
            #base_impl
            #vec_impl
            #methods_impl
            #inferbase_impl
        })
    }
}

/// Generates an `impl AltrepLen for T` block that delegates to a named struct field.
///
/// The generated implementation returns `self.{len_field}` cast to `usize` as the
/// ALTREP vector length.
///
/// # Arguments
///
/// * `name` -- The struct identifier.
/// * `generics` -- Generic parameters for the struct.
/// * `len_field` -- The identifier of the field that holds the length value.
fn generate_altrep_len(
    name: &syn::Ident,
    generics: &syn::Generics,
    len_field: &syn::Ident,
) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics ::miniextendr_api::altrep_data::AltrepLen for #name #ty_generics #where_clause {
            fn len(&self) -> usize {
                self.#len_field
            }
        }
    }
}

/// Shared implementation for all non-list ALTREP derive macros.
///
/// Generates three items:
/// 1. `impl AltrepLen` -- delegates to the detected/specified length field
/// 2. `impl Alt*Data` -- the family-specific data trait with an `elt()` method
/// 3. Low-level trait impls via [`AltrepAttrs::generate_lowlevel`]
///
/// # Arguments
///
/// * `input` -- The `DeriveInput` from the proc-macro.
/// * `data_trait_path` -- The fully qualified path to the data trait
///   (e.g., `::miniextendr_api::altrep_data::AltIntegerData`).
/// * `gen_elt_impl` -- A closure that receives the optional `elt_field` and returns
///   a token stream for the `fn elt(...)` method body. If `elt_field` is `Some`, the
///   closure typically returns `self.{field}`; if `None`, it returns a family-appropriate
///   default (`NA_INTEGER`, `f64::NAN`, `0u8`, `Logical::Na`, `None`, or `Rcomplex { NAN, NAN }`).
/// * `family` -- The [`AltrepFamilyConfig`] for this type family.
///
/// # Errors
///
/// Returns `Err` if attribute parsing fails, no length field can be found, or
/// option validation fails.
fn derive_altrep_generic(
    input: syn::DeriveInput,
    data_trait_path: TokenStream,
    gen_elt_impl: impl FnOnce(Option<&syn::Ident>, Option<&syn::Ident>) -> TokenStream,
    family: &AltrepFamilyConfig,
) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let generics = &input.generics;
    let attrs = AltrepAttrs::parse(&input)?;
    let len_field = attrs.get_len_field(&input)?;

    let altrep_len_impl = generate_altrep_len(name, generics, &len_field);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let elt_impl = gen_elt_impl(attrs.elt_field.as_ref(), attrs.elt_delegate.as_ref());

    let data_trait_impl = quote! {
        impl #impl_generics #data_trait_path for #name #ty_generics #where_clause {
            #elt_impl
        }
    };

    let lowlevel_impl = attrs.generate_lowlevel(name, family)?;

    Ok(quote! {
        #altrep_len_impl
        #data_trait_impl
        #lowlevel_impl
    })
}

/// Derive macro entry point for `AltrepInteger`.
///
/// Auto-implements `AltrepLen` and `AltIntegerData` for a struct with a length field.
/// The `elt()` method returns `self.{elt_field}` as `i32` if `#[altrep(elt = "...")]`
/// is specified, or `NA_INTEGER` by default.
///
/// Supports `#[altrep(dataptr)]` for direct `i32` data pointer access and
/// `#[altrep(subset)]` for `Extract_subset`.
pub fn derive_altrep_integer(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    derive_altrep_generic(
        input,
        quote! { ::miniextendr_api::altrep_data::AltIntegerData },
        |elt_field, elt_delegate| {
            if let Some(d) = elt_delegate {
                quote! { fn elt(&self, i: usize) -> i32 { self.#d.elt(i) } }
            } else if let Some(f) = elt_field {
                quote! { fn elt(&self, _i: usize) -> i32 { self.#f } }
            } else {
                quote! { fn elt(&self, _i: usize) -> i32 { ::miniextendr_api::altrep_traits::NA_INTEGER } }
            }
        },
        &AltrepFamilyConfig {
            macro_base: "impl_altinteger_from_data",
            dataptr_macro: Some(("__impl_altvec_dataptr", Some(quote! { i32 }))),
            string_dataptr: false,
            subset: true,
            methods_macro: "__impl_altinteger_methods",
            inferbase_macro: "impl_inferbase_integer",
            default_guard: "RUnwind",
        },
    )
}

/// Derive macro entry point for `AltrepReal`.
///
/// Auto-implements `AltrepLen` and `AltRealData` for a struct with a length field.
/// The `elt()` method returns `self.{elt_field}` as `f64` if `#[altrep(elt = "...")]`
/// is specified, or `f64::NAN` (R's `NA_real_`) by default.
///
/// Supports `#[altrep(dataptr)]` for direct `f64` data pointer access and
/// `#[altrep(subset)]` for `Extract_subset`.
pub fn derive_altrep_real(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    derive_altrep_generic(
        input,
        quote! { ::miniextendr_api::altrep_data::AltRealData },
        |elt_field, elt_delegate| {
            if let Some(d) = elt_delegate {
                quote! { fn elt(&self, i: usize) -> f64 { self.#d.elt(i) } }
            } else if let Some(f) = elt_field {
                quote! { fn elt(&self, _i: usize) -> f64 { self.#f } }
            } else {
                quote! { fn elt(&self, _i: usize) -> f64 { f64::NAN } }
            }
        },
        &AltrepFamilyConfig {
            macro_base: "impl_altreal_from_data",
            dataptr_macro: Some(("__impl_altvec_dataptr", Some(quote! { f64 }))),
            string_dataptr: false,
            subset: true,
            methods_macro: "__impl_altreal_methods",
            inferbase_macro: "impl_inferbase_real",
            default_guard: "RUnwind",
        },
    )
}

/// Derive macro entry point for `AltrepLogical`.
///
/// Auto-implements `AltrepLen` and `AltLogicalData` for a struct with a length field.
/// The `elt()` method returns `self.{elt_field}.into()` as `Logical` if
/// `#[altrep(elt = "...")]` is specified, or `Logical::Na` by default.
///
/// Supports `#[altrep(dataptr)]` for direct `i32` data pointer access (logicals are
/// stored as `i32` in R) and `#[altrep(subset)]` for `Extract_subset`.
pub fn derive_altrep_logical(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    derive_altrep_generic(
        input,
        quote! { ::miniextendr_api::altrep_data::AltLogicalData },
        |elt_field, elt_delegate| {
            if let Some(d) = elt_delegate {
                quote! { fn elt(&self, i: usize) -> ::miniextendr_api::altrep_data::Logical { self.#d.elt(i) } }
            } else if let Some(f) = elt_field {
                quote! { fn elt(&self, _i: usize) -> ::miniextendr_api::altrep_data::Logical { self.#f.into() } }
            } else {
                quote! { fn elt(&self, _i: usize) -> ::miniextendr_api::altrep_data::Logical { ::miniextendr_api::altrep_data::Logical::Na } }
            }
        },
        &AltrepFamilyConfig {
            macro_base: "impl_altlogical_from_data",
            dataptr_macro: Some(("__impl_altvec_dataptr", Some(quote! { i32 }))),
            string_dataptr: false,
            subset: true,
            methods_macro: "__impl_altlogical_methods",
            inferbase_macro: "impl_inferbase_logical",
            default_guard: "RUnwind",
        },
    )
}

/// Derive macro entry point for `AltrepRaw`.
///
/// Auto-implements `AltrepLen` and `AltRawData` for a struct with a length field.
/// The `elt()` method returns `self.{elt_field}` as `u8` if `#[altrep(elt = "...")]`
/// is specified, or `0u8` by default.
///
/// Supports `#[altrep(dataptr)]` for direct `u8` data pointer access and
/// `#[altrep(subset)]` for `Extract_subset`.
pub fn derive_altrep_raw(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    derive_altrep_generic(
        input,
        quote! { ::miniextendr_api::altrep_data::AltRawData },
        |elt_field, elt_delegate| {
            if let Some(d) = elt_delegate {
                quote! { fn elt(&self, i: usize) -> u8 { self.#d.elt(i) } }
            } else if let Some(f) = elt_field {
                quote! { fn elt(&self, _i: usize) -> u8 { self.#f } }
            } else {
                quote! { fn elt(&self, _i: usize) -> u8 { 0 } }
            }
        },
        &AltrepFamilyConfig {
            macro_base: "impl_altraw_from_data",
            dataptr_macro: Some(("__impl_altvec_dataptr", Some(quote! { u8 }))),
            string_dataptr: false,
            subset: true,
            methods_macro: "__impl_altraw_methods",
            inferbase_macro: "impl_inferbase_raw",
            default_guard: "RUnwind",
        },
    )
}

/// Derive macro entry point for `AltrepString`.
///
/// Auto-implements `AltrepLen` and `AltStringData` for a struct with a length field.
/// The `elt()` method returns `Some(self.{elt_field}.as_ref())` as `Option<&str>` if
/// `#[altrep(elt = "...")]` is specified, or `None` (R's `NA_character_`) by default.
///
/// String ALTREP supports `#[altrep(dataptr)]` for materialized `STRSXP` dataptr
/// (via `__impl_altvec_string_dataptr`) and `#[altrep(subset)]` for `Extract_subset`.
/// Note: String dataptr materializes the entire vector into a cached `STRSXP` in the
/// data2 slot.
pub fn derive_altrep_string(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    derive_altrep_generic(
        input,
        quote! { ::miniextendr_api::altrep_data::AltStringData },
        |elt_field, elt_delegate| {
            if let Some(d) = elt_delegate {
                quote! { fn elt(&self, i: usize) -> Option<&str> { self.#d.elt(i) } }
            } else if let Some(f) = elt_field {
                quote! { fn elt(&self, _i: usize) -> Option<&str> { Some(self.#f.as_ref()) } }
            } else {
                quote! { fn elt(&self, _i: usize) -> Option<&str> { None } }
            }
        },
        &AltrepFamilyConfig {
            macro_base: "impl_altstring_from_data",
            dataptr_macro: None,
            string_dataptr: true,
            subset: true,
            methods_macro: "__impl_altstring_methods",
            inferbase_macro: "impl_inferbase_string",
            // String elt calls Rf_mkCharLenCE; dataptr calls Rf_allocVector + SET_STRING_ELT.
            // These R API calls can longjmp — must use RUnwind.
            default_guard: "RUnwind",
        },
    )
}

/// Derive macro entry point for `AltrepComplex`.
///
/// Auto-implements `AltrepLen` and `AltComplexData` for a struct with a length field.
/// The `elt()` method returns `self.{elt_field}` as `Rcomplex` if
/// `#[altrep(elt = "...")]` is specified, or `Rcomplex { r: NAN, i: NAN }` by default.
///
/// Supports `#[altrep(dataptr)]` for direct `Rcomplex` data pointer access and
/// `#[altrep(subset)]` for `Extract_subset`.
pub fn derive_altrep_complex(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    derive_altrep_generic(
        input,
        quote! { ::miniextendr_api::altrep_data::AltComplexData },
        |elt_field, elt_delegate| {
            if let Some(d) = elt_delegate {
                quote! { fn elt(&self, i: usize) -> ::miniextendr_api::ffi::Rcomplex { self.#d.elt(i) } }
            } else if let Some(f) = elt_field {
                quote! { fn elt(&self, _i: usize) -> ::miniextendr_api::ffi::Rcomplex { self.#f } }
            } else {
                quote! {
                    fn elt(&self, _i: usize) -> ::miniextendr_api::ffi::Rcomplex {
                        ::miniextendr_api::ffi::Rcomplex { r: f64::NAN, i: f64::NAN }
                    }
                }
            }
        },
        &AltrepFamilyConfig {
            macro_base: "impl_altcomplex_from_data",
            dataptr_macro: Some((
                "__impl_altvec_dataptr",
                Some(quote! { ::miniextendr_api::ffi::Rcomplex }),
            )),
            string_dataptr: false,
            subset: true,
            methods_macro: "__impl_altcomplex_methods",
            inferbase_macro: "impl_inferbase_complex",
            default_guard: "RUnwind",
        },
    )
}

/// Derive macro entry point for `AltrepList`.
///
/// Auto-implements `AltrepLen` and `AltListData` for a struct with a length field.
/// The `elt()` method returns `self.{elt_field}[i]` as `SEXP` if
/// `#[altrep(elt = "...")]` is specified (the field should be indexable and return `SEXP`),
/// or `R_NilValue` by default.
///
/// List ALTREP does **not** support `#[altrep(dataptr)]` or `#[altrep(subset)]` -- both
/// are rejected at compile time. `#[altrep(serialize)]` is supported but requires the
/// expanded code generation path with individual internal macros.
pub fn derive_altrep_list(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let generics = &input.generics;
    let attrs = AltrepAttrs::parse(&input)?;
    let len_field = attrs.get_len_field(&input)?;

    let altrep_len_impl = generate_altrep_len(name, generics, &len_field);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // List elt() returns SEXP - if elt_field specified, assume it's a Vec<SEXP> or similar
    let elt_impl = if let Some(ref elt_field) = attrs.elt_field {
        quote! {
            fn elt(&self, i: usize) -> ::miniextendr_api::ffi::SEXP {
                self.#elt_field[i]
            }
        }
    } else {
        quote! {
            fn elt(&self, _i: usize) -> ::miniextendr_api::ffi::SEXP {
                ::miniextendr_api::ffi::SEXP::null()
            }
        }
    };

    let alt_list_impl = quote! {
        impl #impl_generics ::miniextendr_api::altrep_data::AltListData for #name #ty_generics #where_clause {
            #elt_impl
        }
    };

    // List does not support dataptr or subset
    for opt in &attrs.lowlevel_options {
        if opt == "dataptr" || opt == "subset" {
            return Err(syn::Error::new(
                opt.span(),
                format!("`{opt}` is not supported for AltrepList"),
            ));
        }
    }

    let has_serialize = attrs.lowlevel_options.iter().any(|o| o == "serialize");

    let lowlevel_impl = if !attrs.generate_lowlevel {
        quote! {}
    } else if has_serialize {
        // Serialize requires expanding individual macros since impl_altlist_from_data!
        // does not have a serialize variant. Use __impl_altrep_base_with_serialize!
        // for the Altrep trait, then manually emit AltVec + AltList + InferBase.
        if attrs.has_non_default_guard() {
            let guard = attrs.guard.as_ref().unwrap();
            quote! {
                ::miniextendr_api::__impl_altrep_base_with_serialize!(#name, #guard);
                impl ::miniextendr_api::altrep_traits::AltVec for #name {}
                #[allow(clippy::not_unsafe_ptr_arg_deref)]
                impl ::miniextendr_api::altrep_traits::AltList for #name {
                    fn elt(x: ::miniextendr_api::ffi::SEXP, i: ::miniextendr_api::ffi::R_xlen_t) -> ::miniextendr_api::ffi::SEXP {
                        unsafe { ::miniextendr_api::altrep_data1_as::<#name>(x) }
                            .map(|d| <#name as ::miniextendr_api::altrep_data::AltListData>::elt(&*d, i.max(0) as usize))
                            .unwrap_or(::miniextendr_api::ffi::SEXP::null())
                    }
                }
                ::miniextendr_api::impl_inferbase_list!(#name);
            }
        } else {
            quote! {
                ::miniextendr_api::__impl_altrep_base_with_serialize!(#name);
                impl ::miniextendr_api::altrep_traits::AltVec for #name {}
                #[allow(clippy::not_unsafe_ptr_arg_deref)]
                impl ::miniextendr_api::altrep_traits::AltList for #name {
                    fn elt(x: ::miniextendr_api::ffi::SEXP, i: ::miniextendr_api::ffi::R_xlen_t) -> ::miniextendr_api::ffi::SEXP {
                        unsafe { ::miniextendr_api::altrep_data1_as::<#name>(x) }
                            .map(|d| <#name as ::miniextendr_api::altrep_data::AltListData>::elt(&*d, i.max(0) as usize))
                            .unwrap_or(::miniextendr_api::ffi::SEXP::null())
                    }
                }
                ::miniextendr_api::impl_inferbase_list!(#name);
            }
        }
    } else if attrs.has_non_default_guard() {
        let guard = attrs.guard.as_ref().unwrap();
        quote! {
            ::miniextendr_api::impl_altlist_from_data!(#name, #guard);
        }
    } else {
        quote! {
            ::miniextendr_api::impl_altlist_from_data!(#name);
        }
    };

    Ok(quote! {
        #altrep_len_impl
        #alt_list_impl
        #lowlevel_impl
    })
}
