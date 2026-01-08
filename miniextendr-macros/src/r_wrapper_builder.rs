//! Shared utilities for building R wrapper code.
//!
//! This module provides builders for constructing R function signatures and call arguments
//! consistently across both standalone functions and impl methods.
//!
//! ## Key Components
//!
//! - [`RArgumentBuilder`]: Builds R formals and `.Call()` arguments from Rust signatures
//! - [`DotCallBuilder`]: Formats `.Call()` invocations with proper argument handling
//! - [`RoxygenBuilder`]: Generates roxygen2 documentation tags
//!
//! ## Usage
//!
//! ```ignore
//! // Build R function signature
//! let formals = build_r_formals_from_sig(&method.sig, &defaults);
//! let call_args = build_r_call_args_from_sig(&method.sig);
//!
//! // Build .Call() invocation
//! let call = DotCallBuilder::new("C_MyType__method")
//!     .with_self("self")
//!     .with_args(&["x", "y"])
//!     .build();
//!
//! // Build roxygen tags
//! let tags = RoxygenBuilder::new("MyType")
//!     .name("method")
//!     .rdname("MyType")
//!     .export()
//!     .build();
//! ```

/// Normalizes Rust argument identifiers for R.
///
/// - Leading `_` → stripped (Rust convention for unused params)
/// - Leading `__` → stripped
/// - Otherwise → unchanged
///
/// # Examples
/// - `_x` → `x`
/// - `_to` → `to`
/// - `__field` → `field`
/// - `value` → `value`
///
/// Note: We strip underscores rather than prefixing "unused" because R callers
/// (like vctrs) may use named arguments that must match the original name.
pub fn normalize_r_arg_ident(rust_ident: &syn::Ident) -> syn::Ident {
    let arg_name = rust_ident.to_string();
    let normalized = arg_name.trim_start_matches('_');
    // Handle edge case of just underscores
    let normalized = if normalized.is_empty() {
        "arg"
    } else {
        normalized
    };
    syn::Ident::new(normalized, rust_ident.span())
}

/// Builder for R function formal parameters and call arguments.
///
/// Handles:
/// - Underscore normalization (`_x` → `unused_x`)
/// - Unit type defaults (`()` → `= NULL`)
/// - Dots (`...`) with optional naming
/// - Consistent formatting across function and method wrappers
pub struct RArgumentBuilder<'a> {
    inputs: &'a syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
    /// If true, last parameter is treated as dots (`...`)
    has_dots: bool,
    /// Optional named binding for dots (e.g., `args = ...`)
    named_dots: Option<String>,
    /// If true, skip the first parameter (used for `self` in method wrappers)
    skip_first: bool,
    /// Parameter default values from `#[miniextendr(default = "...")]`
    defaults: std::collections::HashMap<String, String>,
}

impl<'a> RArgumentBuilder<'a> {
    /// Create a new builder for the given function inputs.
    pub fn new(inputs: &'a syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>) -> Self {
        Self {
            inputs,
            has_dots: false,
            named_dots: None,
            skip_first: false,
            defaults: std::collections::HashMap::new(),
        }
    }

    /// Add parameter defaults from `#[miniextendr(default = "...")]` attributes.
    pub fn with_defaults(mut self, defaults: std::collections::HashMap<String, String>) -> Self {
        self.defaults = defaults;
        self
    }

    /// Mark the last parameter as dots (`...`).
    pub fn with_dots(mut self, named_dots: Option<String>) -> Self {
        self.has_dots = true;
        self.named_dots = named_dots.map(|s| {
            normalize_r_arg_ident(&syn::Ident::new(&s, proc_macro2::Span::call_site())).to_string()
        });
        self
    }

    /// Skip the first parameter (for instance methods with `self`).
    pub fn skip_first(mut self) -> Self {
        self.skip_first = true;
        self
    }

    /// Build R formal parameters string (for function signature).
    ///
    /// # Returns
    /// Comma-separated parameter list, e.g., `"x, y = NULL, ..."`
    ///
    /// This method handles R-style defaults (like `1L`, `c(1,2,3)`) that aren't
    /// valid Rust syntax by outputting them directly as strings.
    pub fn build_formals(&self) -> String {
        let mut formals = Vec::new();
        let last_idx = self.inputs.len().saturating_sub(1);

        for (idx, input) in self.inputs.iter().enumerate() {
            // Skip first if requested (for self in methods)
            if self.skip_first && idx == 0 {
                continue;
            }

            let pat_type = match input {
                syn::FnArg::Typed(pt) => pt,
                syn::FnArg::Receiver(_) => continue, // Skip self receivers
            };

            // Handle dots (must be last)
            // Note: In R, `...` cannot have a name/default in formals - it must be just `...`
            // The named_dots is only used on the Rust side. R formals always use plain `...`
            if self.has_dots && idx == last_idx {
                formals.push("...".to_string());
                continue;
            }

            // Extract and normalize argument name
            let arg_ident = match pat_type.pat.as_ref() {
                syn::Pat::Ident(pat_ident) => normalize_r_arg_ident(&pat_ident.ident),
                _ => continue,
            };

            // Check for user-specified default value
            if let Some(default_val) = self.defaults.get(&arg_ident.to_string()) {
                // User provided default via #[miniextendr(default = "...")]
                // Output directly as string - supports R-style defaults like "1L", "c(1,2,3)"
                formals.push(format!("{} = {}", arg_ident, default_val));
                continue;
            }

            // Add default for unit types
            match pat_type.ty.as_ref() {
                syn::Type::Tuple(t) if t.elems.is_empty() => {
                    formals.push(format!("{} = NULL", arg_ident));
                }
                _ => {
                    formals.push(arg_ident.to_string());
                }
            }
        }

        formals.join(", ")
    }

    /// Build R call arguments string (for `.Call()` invocation).
    ///
    /// # Returns
    /// Comma-separated argument list, e.g., `"x, y, list(...)"`
    pub fn build_call_args(&self) -> String {
        self.build_call_args_vec().join(", ")
    }

    /// Build R call arguments as `Vec<String>`.
    pub fn build_call_args_vec(&self) -> Vec<String> {
        let mut call_args = Vec::new();
        let last_idx = self.inputs.len().saturating_sub(1);

        for (idx, input) in self.inputs.iter().enumerate() {
            // Skip first if requested (for self in methods)
            if self.skip_first && idx == 0 {
                continue;
            }

            let syn::FnArg::Typed(pat_type) = input else {
                continue;
            };

            // Handle dots special case
            // Always use list(...) since R formals always have plain `...`
            if self.has_dots && idx == last_idx {
                call_args.push("list(...)".to_string());
                continue;
            }

            // Extract and normalize argument name
            let arg_ident = match pat_type.pat.as_ref() {
                syn::Pat::Ident(pat_ident) => normalize_r_arg_ident(&pat_ident.ident),
                _ => continue,
            };

            call_args.push(arg_ident.to_string());
        }

        call_args
    }
}

/// Build R formal parameters from a Rust signature, with optional defaults.
pub(crate) fn build_r_formals_from_sig(
    sig: &syn::Signature,
    defaults: &std::collections::HashMap<String, String>,
) -> String {
    let mut builder = RArgumentBuilder::new(&sig.inputs);
    if matches!(sig.inputs.first(), Some(syn::FnArg::Receiver(_))) {
        builder = builder.skip_first();
    }
    builder = builder.with_defaults(defaults.clone());
    builder.build_formals()
}

/// Build R .Call arguments from a Rust signature.
pub(crate) fn build_r_call_args_from_sig(sig: &syn::Signature) -> String {
    let mut builder = RArgumentBuilder::new(&sig.inputs);
    if matches!(sig.inputs.first(), Some(syn::FnArg::Receiver(_))) {
        builder = builder.skip_first();
    }
    builder.build_call_args()
}

/// Collect parameter identifiers from a function signature.
///
/// Skips receivers and optionally the first argument. Optionally normalizes
/// identifiers for R-friendly names.
pub(crate) fn collect_param_idents(
    inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
    skip_first: bool,
    normalize: bool,
) -> Vec<String> {
    let mut params = Vec::new();
    for (idx, arg) in inputs.iter().enumerate() {
        if skip_first && idx == 0 {
            continue;
        }
        let syn::FnArg::Typed(pt) = arg else {
            continue;
        };
        let syn::Pat::Ident(pat_ident) = pt.pat.as_ref() else {
            continue;
        };
        if normalize {
            params.push(normalize_r_arg_ident(&pat_ident.ident).to_string());
        } else {
            params.push(pat_ident.ident.to_string());
        }
    }
    params
}

// =============================================================================
// DotCallBuilder - .Call() invocation formatting
// =============================================================================

/// Builder for formatting `.Call()` invocations in R wrapper code.
///
/// Handles the common pattern of `.Call(C_ident, .call = match.call(), args...)`.
///
/// # Example
///
/// ```ignore
/// let call = DotCallBuilder::new("C_Counter__increment")
///     .with_self("self")
///     .build();
/// // => ".Call(C_Counter__increment, .call = match.call(), self)"
///
/// let call = DotCallBuilder::new("C_Counter__add")
///     .with_self("x")
///     .with_args(&["n"])
///     .build();
/// // => ".Call(C_Counter__add, .call = match.call(), x, n)"
/// ```
pub struct DotCallBuilder {
    c_ident: String,
    self_var: Option<String>,
    args: Vec<String>,
}

impl DotCallBuilder {
    /// Create a new builder with the C function identifier.
    pub fn new(c_ident: impl Into<String>) -> Self {
        Self {
            c_ident: c_ident.into(),
            self_var: None,
            args: Vec::new(),
        }
    }

    /// Add a self/x parameter (prepended to args).
    pub fn with_self(mut self, var: impl Into<String>) -> Self {
        self.self_var = Some(var.into());
        self
    }

    /// Add arguments after self (if any).
    pub fn with_args(mut self, args: &[impl AsRef<str>]) -> Self {
        self.args = args.iter().map(|s| s.as_ref().to_string()).collect();
        self
    }

    /// Build the `.Call()` string.
    pub fn build(&self) -> String {
        let mut all_args = Vec::new();

        if let Some(ref self_var) = self.self_var {
            all_args.push(self_var.clone());
        }
        all_args.extend(self.args.clone());

        if all_args.is_empty() {
            format!(".Call({}, .call = match.call())", self.c_ident)
        } else {
            format!(
                ".Call({}, .call = match.call(), {})",
                self.c_ident,
                all_args.join(", ")
            )
        }
    }
}

// =============================================================================
// RoxygenBuilder - roxygen2 documentation tag generation
// =============================================================================

/// Builder for generating roxygen2 documentation tags.
///
/// Provides a fluent API for building common roxygen tag patterns used
/// across all class systems.
///
/// # Example
///
/// ```ignore
/// let tags = RoxygenBuilder::new()
///     .name("Counter$increment")
///     .rdname("Counter")
///     .export()
///     .build();
/// // => vec!["#' @name Counter$increment", "#' @rdname Counter", "#' @export"]
/// ```
pub struct RoxygenBuilder {
    name: Option<String>,
    rdname: Option<String>,
    title: Option<String>,
    description: Option<String>,
    source: Option<String>,
    export: bool,
    export_method: Option<String>,
    method: Option<(String, String)>, // (generic, class)
    custom_tags: Vec<String>,
}

impl RoxygenBuilder {
    /// Create a new empty builder.
    pub fn new() -> Self {
        Self {
            name: None,
            rdname: None,
            title: None,
            description: None,
            source: None,
            export: false,
            export_method: None,
            method: None,
            custom_tags: Vec::new(),
        }
    }

    /// Set the `@name` tag.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the `@rdname` tag (groups docs into one page).
    pub fn rdname(mut self, rdname: impl Into<String>) -> Self {
        self.rdname = Some(rdname.into());
        self
    }

    /// Set the `@title` tag.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the `@description` tag.
    #[allow(dead_code)] // Public API for external consumers
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set the `@source` tag (typically "Generated by miniextendr...").
    pub fn source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Add `@export` tag.
    pub fn export(mut self) -> Self {
        self.export = true;
        self
    }

    /// Add `@exportMethod` tag (for S4).
    #[allow(dead_code)] // Public API for external consumers
    pub fn export_method(mut self, method: impl Into<String>) -> Self {
        self.export_method = Some(method.into());
        self
    }

    /// Add `@method` tag (for S3).
    pub fn method(mut self, generic: impl Into<String>, class: impl Into<String>) -> Self {
        self.method = Some((generic.into(), class.into()));
        self
    }

    /// Add a custom tag line (without the `#' ` prefix).
    pub fn custom(mut self, tag: impl Into<String>) -> Self {
        self.custom_tags.push(tag.into());
        self
    }

    /// Build the roxygen tag lines (each prefixed with `#' `).
    pub fn build(&self) -> Vec<String> {
        let mut lines = Vec::new();

        if let Some(ref title) = self.title {
            lines.push(format!("#' @title {}", title));
        }
        if let Some(ref desc) = self.description {
            lines.push(format!("#' @description {}", desc));
        }
        if let Some(ref name) = self.name {
            lines.push(format!("#' @name {}", name));
        }
        if let Some(ref rdname) = self.rdname {
            lines.push(format!("#' @rdname {}", rdname));
        }
        if let Some(ref source) = self.source {
            lines.push(format!("#' @source {}", source));
        }
        if let Some((ref generic, ref class)) = self.method {
            lines.push(format!("#' @method {} {}", generic, class));
        }
        for tag in &self.custom_tags {
            lines.push(format!("#' {}", tag));
        }
        if self.export {
            lines.push("#' @export".to_string());
        }
        if let Some(ref method) = self.export_method {
            lines.push(format!("#' @exportMethod {}", method));
        }

        lines
    }
}

impl Default for RoxygenBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests;
