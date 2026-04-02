//! R wrapper generation for trait methods across all class systems.
//!
//! Each class system (Env, S3, S4, S7, R6, Vctrs) has its own generator that
//! produces R code strings for instance methods, static methods, and associated
//! constants. The top-level [`generate_trait_r_wrapper`] dispatches to the
//! appropriate generator and applies post-processing for export/documentation control.

use super::{TraitConst, TraitMethod, trait_method_body_lines};
use crate::miniextendr_impl::ClassSystem;

/// Options controlling export visibility and documentation for trait R wrapper generation.
pub(super) struct TraitWrapperOpts {
    /// Which R class system to generate wrappers for (env, r6, s3, s4, s7, vctrs).
    pub(super) class_system: ClassSystem,
    /// Whether the impl block has `@noRd`, suppressing roxygen documentation output.
    /// For S3/vctrs, method registration tags are preserved even when this is true.
    pub(super) class_has_no_rd: bool,
    /// Whether `#[miniextendr(internal)]` is set, adding `@keywords internal` and
    /// suppressing `@export`/`@exportMethod`.
    pub(super) internal: bool,
    /// Whether `#[miniextendr(noexport)]` is set, suppressing `@export`/`@exportMethod`
    /// without adding `@keywords internal`.
    pub(super) noexport: bool,
}

/// Generate R wrapper code for trait methods and consts, dispatching by class system.
///
/// Calls the appropriate class-system-specific generator (env, s3, s4, s7, r6),
/// then applies post-processing for `@noRd`, `internal`, and `noexport` options:
///
/// - `class_has_no_rd`: Strips roxygen blocks (for S3/vctrs, keeps `@method`/`@export` tags)
/// - `internal`: Replaces `@export`/`@exportMethod` with `@keywords internal`
/// - `noexport`: Removes `@export`/`@exportMethod` entirely
///
/// Returns the complete R wrapper code as a string ready for embedding in a `const`.
pub(super) fn generate_trait_r_wrapper(
    type_ident: &syn::Ident,
    trait_name: &syn::Ident,
    methods: &[TraitMethod],
    consts: &[TraitConst],
    opts: TraitWrapperOpts,
) -> syn::Result<String> {
    let TraitWrapperOpts {
        class_system,
        class_has_no_rd,
        internal,
        noexport,
    } = opts;
    let result = match class_system {
        ClassSystem::Env => generate_trait_env_r_wrapper(type_ident, trait_name, methods, consts)?,
        ClassSystem::S3 => generate_trait_s3_r_wrapper(type_ident, trait_name, methods, consts),
        ClassSystem::S4 => generate_trait_s4_r_wrapper(type_ident, trait_name, methods, consts),
        ClassSystem::S7 => generate_trait_s7_r_wrapper(type_ident, trait_name, methods, consts),
        ClassSystem::R6 => generate_trait_r6_r_wrapper(type_ident, trait_name, methods, consts),
        // vctrs uses S3 under the hood, so use the S3 trait wrapper
        ClassSystem::Vctrs => generate_trait_s3_r_wrapper(type_ident, trait_name, methods, consts),
    };

    // When impl block has @noRd, suppress documentation generation.
    // For S3/vctrs impls we still keep S3 method registration tags so roxygen
    // can generate NAMESPACE entries without emitting missing-export warnings.
    if class_has_no_rd {
        if matches!(class_system, ClassSystem::S3 | ClassSystem::Vctrs) {
            let mut filtered = Vec::new();
            let mut roxygen_block: Vec<&str> = Vec::new();

            let flush_block = |block: &mut Vec<&str>, out: &mut Vec<String>| {
                if block.iter().any(|line| line.contains("@method ")) {
                    out.push("#' @noRd".to_string());
                    for &line in block.iter() {
                        if line.contains("@method ")
                            || line.contains("@param ")
                            || line.contains("@export")
                        {
                            out.push(line.to_string());
                        }
                    }
                }
                block.clear();
            };

            for line in result.lines() {
                if line.starts_with("#'") {
                    roxygen_block.push(line);
                    continue;
                }

                if !roxygen_block.is_empty() {
                    flush_block(&mut roxygen_block, &mut filtered);
                }
                filtered.push(line.to_string());
            }

            if !roxygen_block.is_empty() {
                flush_block(&mut roxygen_block, &mut filtered);
            }

            Ok(filtered.join("\n"))
        } else {
            Ok(result
                .lines()
                .filter(|line| !line.starts_with("#'"))
                .collect::<Vec<_>>()
                .join("\n"))
        }
    } else if !class_has_no_rd && (internal || noexport) {
        // internal → add @keywords internal + suppress @export/@exportMethod
        // noexport → suppress @export/@exportMethod only
        let has_export = result.lines().any(|line| line.contains("@export"));
        let mut processed: Vec<String> = result
            .lines()
            .flat_map(|line| {
                if line.contains("@export") {
                    // Replace @export/@exportMethod with @keywords internal (for internal)
                    // or just remove (for noexport)
                    if internal {
                        vec!["#' @keywords internal".to_string()]
                    } else {
                        vec![]
                    }
                } else {
                    vec![line.to_string()]
                }
            })
            .collect();
        // For class systems without @export (e.g., Env), insert @keywords internal
        // before the first roxygen tag if no @export line was found to replace.
        if internal
            && !has_export
            && let Some(pos) = processed.iter().position(|l| l.starts_with("#'"))
        {
            processed.insert(pos, "#' @keywords internal".to_string());
        }
        Ok(processed.join("\n"))
    } else {
        Ok(result)
    }
}

/// Generate Env-style R wrapper code for trait methods.
///
/// Env-class trait methods use a namespace hierarchy: `Type$Trait$method(x, ...)`.
/// Instance methods take `x` as the first parameter (the self object) and are
/// stamped with `.__mx_instance__` attribute for `$` dispatch detection.
/// Void instance methods return `invisible(x)` for pipe-friendly chaining.
///
/// Static methods and constants also live under `Type$Trait$name`.
///
/// Returns an error if an instance method has a parameter named `x` (collides
/// with the self parameter in env-class dispatch).
fn generate_trait_env_r_wrapper(
    type_ident: &syn::Ident,
    trait_name: &syn::Ident,
    methods: &[TraitMethod],
    consts: &[TraitConst],
) -> syn::Result<String> {
    use crate::r_wrapper_builder::{DotCallBuilder, RoxygenBuilder};

    let mut lines = Vec::new();
    let type_str = type_ident.to_string();
    let trait_str = trait_name.to_string();

    // Header comment
    lines.push(format!(
        "# Trait methods and consts for {} implementing {}",
        type_ident, trait_name
    ));
    lines.push(format!(
        "# Generated by #[miniextendr] impl {} for {}",
        trait_name, type_ident
    ));
    lines.push(String::new());

    // Create trait namespace environment
    lines.push(format!(
        "{}${} <- new.env(parent = emptyenv())",
        type_ident, trait_name
    ));
    lines.push(String::new());

    for method in methods {
        let method_name = &method.ident;
        let r_name = method.r_method_name();

        // Build R formals with defaults applied
        let formals =
            crate::r_wrapper_builder::build_r_formals_from_sig(&method.sig, &method.param_defaults);

        // Collect param names for .Call() (without defaults)
        let params =
            crate::r_wrapper_builder::collect_param_idents(&method.sig.inputs, false, true);

        // Build roxygen tags
        let roxygen = RoxygenBuilder::new()
            .name(format!("{}${}${}", type_str, trait_str, r_name))
            .rdname(&type_str)
            .build();
        lines.extend(roxygen);

        // Check for 'x' parameter collision in instance methods
        if method.has_self {
            for input in &method.sig.inputs {
                if let syn::FnArg::Typed(pt) = input
                    && let syn::Pat::Ident(pat_ident) = pt.pat.as_ref()
                    && pat_ident.ident == "x"
                {
                    return Err(syn::Error::new_spanned(
                        &pat_ident.ident,
                        "trait instance method parameter cannot be named `x` \
                         (collides with self parameter in env-class dispatch)",
                    ));
                }
            }
        }

        // Build .Call() invocation — C name uses Rust ident, R name uses r_name
        let c_ident = format!("C_{}__{}__{}", type_ident, trait_name, method_name);
        let (full_params, call) = if method.has_self {
            let fp = if formals.is_empty() {
                "x".to_string()
            } else {
                format!("x, {}", formals)
            };
            let c = DotCallBuilder::new(&c_ident)
                .with_self("x")
                .with_args(&params)
                .build();
            (fp, c)
        } else {
            (
                formals.clone(),
                DotCallBuilder::new(&c_ident).with_args(&params).build(),
            )
        };

        // Generate method wrapper (R-facing name)
        lines.push(format!(
            "{}${}${} <- function({}) {{",
            type_ident, trait_name, r_name, full_params
        ));
        lines.extend(trait_method_body_lines(&call, method.error_in_r, "  "));
        if method.has_self && method.returns_unit() {
            lines.push("  invisible(x)".to_string());
        }
        lines.push("}".to_string());

        // Stamp instance methods with attribute for $ dispatch detection
        if method.has_self {
            lines.push(format!(
                "attr({}${}${}, \".__mx_instance__\") <- TRUE",
                type_ident, trait_name, r_name
            ));
        }

        lines.push(String::new());
    }

    // Generate const wrappers
    for trait_const in consts {
        let const_name = &trait_const.ident;
        let const_str = const_name.to_string();

        // Build roxygen tags
        let roxygen = RoxygenBuilder::new()
            .name(format!("{}${}${}", type_str, trait_str, const_str))
            .rdname(&type_str)
            .build();
        lines.extend(roxygen);

        // Build .Call() invocation
        let c_ident = format!("C_{}__{}__{}", type_ident, trait_name, const_name);
        let call = DotCallBuilder::new(&c_ident).build();

        // Generate const wrapper
        lines.push(format!(
            "{}${}${} <- function() {{",
            type_ident, trait_name, const_name
        ));
        lines.push(format!("  {}", call));
        lines.push("}".to_string());
        lines.push(String::new());
    }

    Ok(lines.join("\n"))
}

/// Generate S3-style R wrapper code (generic + method.Type).
///
/// For `impl Counter for SimpleCounter`, generates:
/// - S3 generic `value(x, ...)` (if not already defined)
/// - S3 method `value.SimpleCounter <- function(x, ...) { .Call(...) }`
/// - S7 method registration if the generic is an S7 generic
///
/// Static methods and constants use `Type$Trait$name` namespace (env-style).
/// Void instance methods return `invisible(x)` for pipe-friendly chaining.
///
/// Also used for `ClassSystem::Vctrs` since vctrs uses S3 under the hood.
fn generate_trait_s3_r_wrapper(
    type_ident: &syn::Ident,
    trait_name: &syn::Ident,
    methods: &[TraitMethod],
    consts: &[TraitConst],
) -> String {
    use crate::r_wrapper_builder::{DotCallBuilder, RoxygenBuilder};

    let mut lines = Vec::new();
    let type_str = type_ident.to_string();
    let trait_str = trait_name.to_string();

    // Header comment
    lines.push(format!(
        "# S3 trait methods for {} implementing {}",
        type_ident, trait_name
    ));
    lines.push(format!(
        "# Generated by #[miniextendr(s3)] impl {} for {}",
        trait_name, type_ident
    ));
    lines.push(String::new());

    // Separate instance methods (S3 dispatch) from static methods (namespace access)
    let instance_methods: Vec<_> = methods.iter().filter(|m| m.has_self).collect();
    let static_methods: Vec<_> = methods.iter().filter(|m| !m.has_self).collect();

    // Generate S3 generics + methods for instance methods
    for method in &instance_methods {
        let method_name = &method.ident;
        let generic_name = method.r_method_name();
        let s3_method_name = format!("{}.{}", generic_name, type_str);

        // Build R formals with defaults applied
        let formals =
            crate::r_wrapper_builder::build_r_formals_from_sig(&method.sig, &method.param_defaults);
        // Collect param names for .Call() (without defaults)
        let params =
            crate::r_wrapper_builder::collect_param_idents(&method.sig.inputs, false, true);

        // S3 generic roxygen (only create if doesn't exist)
        let generic_roxygen = RoxygenBuilder::new()
            .title(format!("S3 generic for `{}`", generic_name))
            .custom(format!("S3 generic for `{}`", generic_name))
            .name(&generic_name)
            .rdname(&type_str)
            .custom("@param x An object")
            .custom("@param ... Additional arguments passed to methods")
            .source(format!(
                "Generated by miniextendr from `impl {} for {}`",
                trait_name, type_ident
            ))
            .export()
            .build();
        lines.extend(generic_roxygen);

        // S3 generic definition
        lines.push(format!(
            "if (!exists(\"{generic_name}\", mode = \"function\")) {{"
        ));
        lines.push(format!(
            "  {generic_name} <- function(x, ...) UseMethod(\"{generic_name}\")"
        ));
        lines.push("}".to_string());
        lines.push(String::new());

        // S3 method roxygen (include @param tags from method doc comments)
        let mut method_roxygen = RoxygenBuilder::new()
            .rdname(&type_str)
            .export()
            .method(&generic_name, &type_str);
        for tag in &method.param_tags {
            method_roxygen = method_roxygen.custom(tag.clone());
        }
        lines.extend(method_roxygen.build());

        // S3 method: generic.class
        let full_params = if formals.is_empty() {
            "x, ...".to_string()
        } else {
            format!("x, {}, ...", formals)
        };

        // Build .Call() invocation
        let c_ident = format!("C_{}__{}__{}", type_ident, trait_name, method_name);
        let call = DotCallBuilder::new(&c_ident)
            .with_self("x")
            .with_args(&params)
            .build();

        // Always define the S3 method (roxygen expects it for NAMESPACE export)
        lines.push(format!(
            "{} <- function({}) {{",
            s3_method_name, full_params
        ));
        lines.extend(trait_method_body_lines(&call, method.error_in_r, "  "));
        // Void instance methods return invisible(x) for pipe-friendly chaining
        if method.returns_unit() {
            lines.push("  invisible(x)".to_string());
        }
        lines.push("}".to_string());

        // Additionally register as S7 method if the generic is S7
        // This ensures S7 dispatch works when the generic was defined by an S7 class
        lines.push(format!(
            "if (inherits(get0(\"{generic_name}\", mode = \"function\"), \"S7_generic\")) {{"
        ));
        lines.push(format!(
            "  S7::method({generic_name}, S7::new_S3_class(\"{type_str}\")) <- {s3_method_name}"
        ));
        lines.push("}".to_string());
        lines.push(String::new());
    }

    // Create trait namespace for static methods and consts BEFORE assigning to it
    if !static_methods.is_empty() || !consts.is_empty() {
        lines.push(format!(
            "{}${} <- new.env(parent = emptyenv())",
            type_ident, trait_name
        ));
        lines.push(String::new());
    }

    // Generate static methods in Type$Trait$ namespace
    for method in &static_methods {
        let method_name = &method.ident;
        let r_name = method.r_method_name();
        // Build R formals with defaults applied
        let formals =
            crate::r_wrapper_builder::build_r_formals_from_sig(&method.sig, &method.param_defaults);
        // Collect param names for .Call() (without defaults)
        let params =
            crate::r_wrapper_builder::collect_param_idents(&method.sig.inputs, false, true);

        // Static method roxygen
        lines.push(format!(
            "#' Static trait method {}::{}()",
            trait_name, r_name
        ));
        let roxygen = RoxygenBuilder::new()
            .name(format!("{}${}${}", type_str, trait_str, r_name))
            .rdname(&type_str)
            .build();
        lines.extend(roxygen);

        // Build .Call() invocation — C name uses Rust ident
        let c_ident = format!("C_{}__{}__{}", type_ident, trait_name, method_name);
        let call = DotCallBuilder::new(&c_ident).with_args(&params).build();

        lines.push(format!(
            "{}${}${} <- function({}) {{",
            type_ident, trait_name, r_name, formals
        ));
        lines.extend(trait_method_body_lines(&call, method.error_in_r, "  "));
        lines.push("}".to_string());
        lines.push(String::new());
    }

    // Generate const wrappers in Type$Trait$ namespace
    for trait_const in consts {
        let const_name = &trait_const.ident;
        let const_str = const_name.to_string();

        let roxygen = RoxygenBuilder::new()
            .name(format!("{}${}${}", type_str, trait_str, const_str))
            .rdname(&type_str)
            .build();
        lines.extend(roxygen);

        let c_ident = format!("C_{}__{}__{}", type_ident, trait_name, const_name);
        let call = DotCallBuilder::new(&c_ident).build();

        lines.push(format!(
            "{}${}${} <- function() {{",
            type_ident, trait_name, const_name
        ));
        lines.push(format!("  {}", call));
        lines.push("}".to_string());
        lines.push(String::new());
    }

    lines.join("\n")
}

/// Generate S4-style R wrapper code.
///
/// For `impl Counter for SimpleCounter`, generates:
/// - `setOldClass("SimpleCounter")` to register the S3 class for S4 dispatch
/// - S4 generic `s4_trait_Counter_value(x, ...)` via `setGeneric()`
/// - S4 method via `setMethod("s4_trait_Counter_value", "SimpleCounter", ...)`
///
/// Generic names are prefixed with `s4_trait_{Trait}_` to avoid collisions
/// with user-defined S4 generics. Static methods and constants are generated
/// as standalone exported functions: `{Type}_{Trait}_{method}()`.
fn generate_trait_s4_r_wrapper(
    type_ident: &syn::Ident,
    trait_name: &syn::Ident,
    methods: &[TraitMethod],
    consts: &[TraitConst],
) -> String {
    use crate::r_wrapper_builder::{DotCallBuilder, RoxygenBuilder};

    let mut lines = Vec::new();
    let type_str = type_ident.to_string();
    let trait_str = trait_name.to_string();

    // Header comment
    lines.push(format!(
        "# S4 trait methods for {} implementing {}",
        type_ident, trait_name
    ));
    lines.push(format!(
        "# Generated by #[miniextendr(s4)] impl {} for {}",
        trait_name, type_ident
    ));
    lines.push(String::new());

    // NOTE: We do NOT call setOldClass here. The inherent impl's class registration
    // (setClass for S4, or setOldClass for S3/env) takes care of that. Calling
    // setOldClass here would clobber a proper S4 setClass with slots.
    lines.push("#' @importFrom methods setGeneric setMethod".to_string());
    lines.push(String::new());

    // Separate instance methods from static methods
    let instance_methods: Vec<_> = methods.iter().filter(|m| m.has_self).collect();
    let static_methods: Vec<_> = methods.iter().filter(|m| !m.has_self).collect();

    // Generate S4 generics + methods for instance methods
    for method in &instance_methods {
        let method_name = &method.ident;
        let generic_name = format!("s4_trait_{}_{}", trait_name, method.r_method_name());

        // Build R formals with defaults applied
        let formals =
            crate::r_wrapper_builder::build_r_formals_from_sig(&method.sig, &method.param_defaults);
        // Collect param names for .Call() (without defaults)
        let params =
            crate::r_wrapper_builder::collect_param_idents(&method.sig.inputs, false, true);

        // Build full parameter list (x first, then others, then ...)
        let full_params = if formals.is_empty() {
            "x, ...".to_string()
        } else {
            format!("x, {}, ...", formals)
        };

        // S4 generic roxygen (include @param tags from method doc comments)
        let mut generic_roxygen = RoxygenBuilder::new()
            .custom(format!(
                "S4 generic for trait method `{}::{}`",
                trait_name, method_name
            ))
            .name(&generic_name)
            .rdname(&type_str)
            .source(format!(
                "Generated by miniextendr from `impl {} for {}`",
                trait_name, type_ident
            ))
            .custom(format!("@param x A `{}` object", type_str))
            .custom("@param ... Additional arguments passed to methods");
        for tag in &method.param_tags {
            generic_roxygen = generic_roxygen.custom(tag.clone());
        }
        lines.extend(generic_roxygen.export().build());

        // Define generic only if it doesn't already exist (avoid clearing methods)
        lines.push(format!(
            "if (!methods::isGeneric(\"{generic_name}\")) methods::setGeneric(\"{generic_name}\", function(x, ...) standardGeneric(\"{generic_name}\"))"
        ));
        lines.push(String::new());

        // S4 method roxygen + definition (include @param tags from method doc comments)
        lines.push(format!("#' @rdname {}", type_str));
        for tag in &method.param_tags {
            lines.push(format!("#' {}", tag));
        }
        lines.push(format!("#' @exportMethod {}", generic_name));

        // Build .Call() invocation
        let c_ident = format!("C_{}__{}__{}", type_ident, trait_name, method_name);
        let call = DotCallBuilder::new(&c_ident)
            .with_self("x")
            .with_args(&params)
            .build();

        lines.push(format!(
            "methods::setMethod(\"{}\", \"{}\", function({}) {{",
            generic_name, type_str, full_params
        ));
        // S4 objects store the ExternalPtr in x@ptr — extract it for .Call()
        lines.push("  .ptr <- x@ptr".to_string());
        let s4_call = call.replace(", x", ", .ptr");
        lines.extend(trait_method_body_lines(&s4_call, method.error_in_r, "  "));
        // Void instance methods return invisible(x) for pipe-friendly chaining
        if method.returns_unit() {
            lines.push("  invisible(x)".to_string());
        }
        lines.push("})".to_string());
        lines.push(String::new());
    }

    // Generate static methods as standalone functions
    for method in &static_methods {
        let method_name = &method.ident;
        let r_name = method.r_method_name();
        let fn_name = format!("{}_{}_{}", type_str, trait_str, r_name);
        // Build R formals with defaults applied
        let formals =
            crate::r_wrapper_builder::build_r_formals_from_sig(&method.sig, &method.param_defaults);
        // Collect param names for .Call() (without defaults)
        let params =
            crate::r_wrapper_builder::collect_param_idents(&method.sig.inputs, false, true);

        // Static method roxygen
        lines.push(format!(
            "#' Static trait method {}::{}() for {}",
            trait_name, r_name, type_str
        ));
        let roxygen = RoxygenBuilder::new()
            .name(&fn_name)
            .rdname(&type_str)
            .export()
            .build();
        lines.extend(roxygen);

        // Build .Call() invocation — C name uses Rust ident
        let c_ident = format!("C_{}__{}__{}", type_ident, trait_name, method_name);
        let call = DotCallBuilder::new(&c_ident).with_args(&params).build();

        lines.push(format!("{} <- function({}) {{", fn_name, formals));
        lines.extend(trait_method_body_lines(&call, method.error_in_r, "  "));
        lines.push("}".to_string());
        lines.push(String::new());
    }

    // Generate const wrappers as standalone functions
    for trait_const in consts {
        let const_name = &trait_const.ident;
        let fn_name = format!("{}_{}_{}", type_str, trait_str, const_name);

        let roxygen = RoxygenBuilder::new()
            .name(&fn_name)
            .rdname(&type_str)
            .export()
            .build();
        lines.extend(roxygen);

        let c_ident = format!("C_{}__{}__{}", type_ident, trait_name, const_name);
        let call = DotCallBuilder::new(&c_ident).build();

        lines.push(format!("{} <- function() {{", fn_name));
        lines.push(format!("  {}", call));
        lines.push("}".to_string());
        lines.push(String::new());
    }

    lines.join("\n")
}

/// Generate S7-style R wrapper code.
///
/// For `impl Counter for SimpleCounter`, generates:
/// - S7 S3-class wrapper: `.s7_class_SimpleCounter <- S7::new_S3_class("SimpleCounter")`
/// - S7 generic: `s7_trait_Counter_value <- S7::new_generic(...)` (if not exists)
/// - S7 method registration: `S7::method(s7_trait_Counter_value, .s7_class_SimpleCounter) <- ...`
///
/// Generic names are prefixed with `s7_trait_{Trait}_` to avoid collisions.
/// Static methods and constants use `Type$Trait$name` namespace (env-style).
fn generate_trait_s7_r_wrapper(
    type_ident: &syn::Ident,
    trait_name: &syn::Ident,
    methods: &[TraitMethod],
    consts: &[TraitConst],
) -> String {
    use crate::r_wrapper_builder::{DotCallBuilder, RoxygenBuilder};

    let mut lines = Vec::new();
    let type_str = type_ident.to_string();
    let trait_str = trait_name.to_string();
    let s7_class_var = format!(".s7_class_{}", type_str);

    // Header comment
    lines.push(format!(
        "# S7 trait methods for {} implementing {}",
        type_ident, trait_name
    ));
    lines.push(format!(
        "# Generated by #[miniextendr(s7)] impl {} for {}",
        trait_name, type_ident
    ));
    lines.push(String::new());

    // Use the S7 class object directly for method dispatch.
    // new_S3_class("Foo") creates a descriptor for "Foo" but S7 new_class
    // creates instances with the namespaced class "pkg::Foo", so new_S3_class
    // wouldn't match. Using the class object directly works correctly.
    lines.push("#' @importFrom S7 new_generic method S7_dispatch".to_string());
    lines.push(format!("{} <- {}", s7_class_var, type_str));
    lines.push(String::new());

    // Separate instance methods from static methods
    let instance_methods: Vec<_> = methods.iter().filter(|m| m.has_self).collect();
    let static_methods: Vec<_> = methods.iter().filter(|m| !m.has_self).collect();

    // Generate S7 generics + methods for instance methods
    for method in &instance_methods {
        let method_name = &method.ident;
        let generic_name = format!("s7_trait_{}_{}", trait_name, method.r_method_name());

        // Build R formals with defaults applied
        let formals =
            crate::r_wrapper_builder::build_r_formals_from_sig(&method.sig, &method.param_defaults);
        // Collect param names for .Call() (without defaults)
        let params =
            crate::r_wrapper_builder::collect_param_idents(&method.sig.inputs, false, true);

        // Build full parameter list (x first, then others, then ...)
        let full_params = if formals.is_empty() {
            "x, ...".to_string()
        } else {
            format!("x, {}, ...", formals)
        };

        // S7 generic roxygen
        // Note: Don't include method-specific @param tags here since S7 methods
        // are assignments and won't appear in \usage, which would cause warnings
        let generic_roxygen = RoxygenBuilder::new()
            .custom(format!(
                "S7 generic for trait method `{}::{}`",
                trait_name, method_name
            ))
            .name(&generic_name)
            .rdname(&type_str)
            .source(format!(
                "Generated by miniextendr from `impl {} for {}`",
                trait_name, type_ident
            ))
            .export()
            .build();
        lines.extend(generic_roxygen);

        // S7 generic definition
        lines.push(format!(
            "if (!exists(\"{generic_name}\", mode = \"function\")) {{"
        ));
        lines.push(format!(
            "  {generic_name} <- S7::new_generic(\"{generic_name}\", \"x\", function(x, ...) S7::S7_dispatch())"
        ));
        lines.push("}".to_string());
        lines.push(String::new());

        // Build .Call() invocation
        let c_ident = format!("C_{}__{}__{}", type_ident, trait_name, method_name);
        let call = DotCallBuilder::new(&c_ident)
            .with_self("x")
            .with_args(&params)
            .build();

        // S7 method definition
        lines.push(format!(
            "S7::method({}, {}) <- function({}) {{",
            generic_name, s7_class_var, full_params
        ));
        // S7 objects store the ExternalPtr in x@.ptr — extract it for .Call()
        lines.push("  .ptr <- x@.ptr".to_string());
        let s7_call = call.replace(", x", ", .ptr");
        lines.extend(trait_method_body_lines(&s7_call, method.error_in_r, "  "));
        // Void instance methods return invisible(x) for pipe-friendly chaining
        if method.returns_unit() {
            lines.push("  invisible(x)".to_string());
        }
        lines.push("}".to_string());
        lines.push(String::new());
    }

    // Create trait namespace for static methods and consts.
    // For S7 classes, use a local variable + attr() to avoid S7's $<- interception.
    let trait_env_var = format!(".{}__{}", type_ident, trait_name);
    if !static_methods.is_empty() || !consts.is_empty() {
        lines.push(format!("{} <- new.env(parent = emptyenv())", trait_env_var));
        lines.push(String::new());
    }

    // Generate static methods in trait namespace
    for method in &static_methods {
        let method_name = &method.ident;
        let r_name = method.r_method_name();
        // Build R formals with defaults applied
        let formals =
            crate::r_wrapper_builder::build_r_formals_from_sig(&method.sig, &method.param_defaults);
        // Collect param names for .Call() (without defaults)
        let params =
            crate::r_wrapper_builder::collect_param_idents(&method.sig.inputs, false, true);

        lines.push(format!(
            "#' Static trait method {}::{}()",
            trait_name, r_name
        ));
        let roxygen = RoxygenBuilder::new()
            .name(format!("{}${}${}", type_str, trait_str, r_name))
            .rdname(&type_str)
            .build();
        lines.extend(roxygen);

        // C name uses Rust ident
        let c_ident = format!("C_{}__{}__{}", type_ident, trait_name, method_name);
        let call = DotCallBuilder::new(&c_ident).with_args(&params).build();

        lines.push(format!(
            "{}${} <- function({}) {{",
            trait_env_var, r_name, formals
        ));
        lines.extend(trait_method_body_lines(&call, method.error_in_r, "  "));
        lines.push("}".to_string());
        lines.push(String::new());
    }

    // Generate const wrappers in trait namespace
    for trait_const in consts {
        let const_name = &trait_const.ident;
        let const_str = const_name.to_string();

        let roxygen = RoxygenBuilder::new()
            .name(format!("{}${}${}", type_str, trait_str, const_str))
            .rdname(&type_str)
            .build();
        lines.extend(roxygen);

        let c_ident = format!("C_{}__{}__{}", type_ident, trait_name, const_name);
        let call = DotCallBuilder::new(&c_ident).build();

        lines.push(format!("{}${} <- function() {{", trait_env_var, const_name));
        lines.push(format!("  {}", call));
        lines.push("}".to_string());
        lines.push(String::new());
    }

    // Attach the trait env to the S7 class via attr() to bypass S7's $<- interception.
    // R's $ accessor on S7 objects falls through to attributes, so Type$Trait$method still works.
    if !static_methods.is_empty() || !consts.is_empty() {
        lines.push(format!(
            "attr({}, \"{}\") <- {}",
            type_ident, trait_name, trait_env_var
        ));
        lines.push(String::new());
    }

    lines.join("\n")
}

/// Generate R6-style R wrapper code.
///
/// R6 classes are defined monolithically (all methods in `R6Class()`), so trait
/// methods cannot be injected into the class definition. Instead, they are
/// generated as standalone exported functions that accept the R6 object.
///
/// For `impl Counter for SimpleCounter`, generates:
/// - `r6_trait_Counter_value(x)` -- exported standalone function
/// - `r6_trait_Counter_increment(x)` -- exported standalone function
///
/// Instance method names are prefixed with `r6_trait_{Trait}_` to avoid collisions.
/// Static methods and constants use `Type$Trait$name` namespace (env-style).
fn generate_trait_r6_r_wrapper(
    type_ident: &syn::Ident,
    trait_name: &syn::Ident,
    methods: &[TraitMethod],
    consts: &[TraitConst],
) -> String {
    use crate::r_wrapper_builder::{DotCallBuilder, RoxygenBuilder};

    let mut lines = Vec::new();
    let type_str = type_ident.to_string();
    let trait_str = trait_name.to_string();

    // Header comment
    lines.push(format!(
        "# R6 trait methods for {} implementing {}",
        type_ident, trait_name
    ));
    lines.push(format!(
        "# Generated by #[miniextendr(r6)] impl {} for {}",
        trait_name, type_ident
    ));
    lines.push("# Note: R6 trait methods are standalone functions".to_string());
    lines.push(String::new());

    // Separate instance methods from static methods
    let instance_methods: Vec<_> = methods.iter().filter(|m| m.has_self).collect();
    let static_methods: Vec<_> = methods.iter().filter(|m| !m.has_self).collect();

    // Generate standalone functions for instance methods
    for method in &instance_methods {
        let method_name = &method.ident;
        let fn_name = format!("r6_trait_{}_{}", trait_name, method.r_method_name());

        // Build R formals with defaults applied
        let formals =
            crate::r_wrapper_builder::build_r_formals_from_sig(&method.sig, &method.param_defaults);
        // Collect param names for .Call() (without defaults)
        let params =
            crate::r_wrapper_builder::collect_param_idents(&method.sig.inputs, false, true);

        // Build parameter list (x first, then others)
        let full_params = if formals.is_empty() {
            "x".to_string()
        } else {
            format!("x, {}", formals)
        };

        // R6 trait method roxygen (include @param tags from method doc comments)
        let mut roxygen = RoxygenBuilder::new()
            .custom(format!(
                "R6 trait method `{}::{}` for {}",
                trait_name, method_name, type_str
            ))
            .name(&fn_name)
            .rdname(&type_str)
            .source(format!(
                "Generated by miniextendr from `impl {} for {}`",
                trait_name, type_ident
            ))
            .custom(format!("@param x A `{}` object", type_str));
        for tag in &method.param_tags {
            roxygen = roxygen.custom(tag.clone());
        }
        lines.extend(roxygen.export().build());

        // Build .Call() invocation
        let c_ident = format!("C_{}__{}__{}", type_ident, trait_name, method_name);
        let call = DotCallBuilder::new(&c_ident)
            .with_self("x")
            .with_args(&params)
            .build();

        lines.push(format!("{} <- function({}) {{", fn_name, full_params));
        // R6 objects store the ExternalPtr in private$.ptr — extract it for .Call()
        lines.push("  .ptr <- x$.__enclos_env__$private$.ptr".to_string());
        // Replace x with .ptr in the .Call
        let r6_call = call.replace(", x", ", .ptr");
        lines.extend(trait_method_body_lines(&r6_call, method.error_in_r, "  "));
        // Void instance methods return invisible(x) for pipe-friendly chaining
        if method.returns_unit() {
            lines.push("  invisible(x)".to_string());
        }
        lines.push("}".to_string());
        lines.push(String::new());
    }

    // Create trait namespace for static methods and consts
    if !static_methods.is_empty() || !consts.is_empty() {
        lines.push(format!(
            "{}${} <- new.env(parent = emptyenv())",
            type_ident, trait_name
        ));
        lines.push(String::new());
    }

    // Generate static methods in Type$Trait$ namespace
    for method in &static_methods {
        let method_name = &method.ident;
        let r_name = method.r_method_name();
        // Build R formals with defaults applied
        let formals =
            crate::r_wrapper_builder::build_r_formals_from_sig(&method.sig, &method.param_defaults);
        // Collect param names for .Call() (without defaults)
        let params =
            crate::r_wrapper_builder::collect_param_idents(&method.sig.inputs, false, true);

        lines.push(format!(
            "#' Static trait method {}::{}()",
            trait_name, r_name
        ));
        let roxygen = RoxygenBuilder::new()
            .name(format!("{}${}${}", type_str, trait_str, r_name))
            .rdname(&type_str)
            .build();
        lines.extend(roxygen);

        // C name uses Rust ident
        let c_ident = format!("C_{}__{}__{}", type_ident, trait_name, method_name);
        let call = DotCallBuilder::new(&c_ident).with_args(&params).build();

        lines.push(format!(
            "{}${}${} <- function({}) {{",
            type_ident, trait_name, r_name, formals
        ));
        lines.extend(trait_method_body_lines(&call, method.error_in_r, "  "));
        lines.push("}".to_string());
        lines.push(String::new());
    }

    // Generate const wrappers in Type$Trait$ namespace
    for trait_const in consts {
        let const_name = &trait_const.ident;
        let const_str = const_name.to_string();

        let roxygen = RoxygenBuilder::new()
            .name(format!("{}${}${}", type_str, trait_str, const_str))
            .rdname(&type_str)
            .build();
        lines.extend(roxygen);

        let c_ident = format!("C_{}__{}__{}", type_ident, trait_name, const_name);
        let call = DotCallBuilder::new(&c_ident).build();

        lines.push(format!(
            "{}${}${} <- function() {{",
            type_ident, trait_name, const_name
        ));
        lines.push(format!("  {}", call));
        lines.push("}".to_string());
        lines.push(String::new());
    }

    lines.join("\n")
}
